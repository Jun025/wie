#!/usr/bin/env node
// Playability census: every unique title in a local game corpus × six axes
// (boot · render · input · longplay · sound · speed), and the `compat.json` the
// featurephone shell's support-status page is built from.
//
// ── Local only, and it commits nothing about a game ─────────────────────────
// The corpus is git-ignored game bytes (Constraint 9), so this runs on this Mac and
// never in CI, like `game-lab-recensus.sh`. Every path comes from the command line and
// every output lands where `--out` points (keep it OUTSIDE the repo): titles are read
// from file names at run time and written only there. The one output meant to be
// committed is `clusters.md`, which names titles by sha256 prefix only.
//
// ── Usage ────────────────────────────────────────────────────────────────────
//   node scripts/playability-census.mjs run --bin <wie_validate> --out <dir> [--jobs <= ncpu, default min(3, ncpu/2)]
//        [--secs 30] [--long 600] [--only probe|long|speed] <corpus dir>...
//   node scripts/playability-census.mjs run … --only progress [--progress 1800] [--titles <file>] [--as P|P2] [--policy v1]
//   node scripts/playability-census.mjs run … --only long --titles <file>   the long run, recipe-prefixed
//        (`<sha12> [secs] [recipe keys file]`; keep the recipes in game_lab/ — they spell a title's menus)
//   node scripts/playability-census.mjs report --out <dir> --pin <wie sha>
//        [--compat <compat.json>] [--changes <changes.json>] [--prs <gh-merged.json>]
//        [--speed <browser runs.jsonl>]
//   node scripts/playability-census.mjs selftest     each speed/starvation rule against its counter-case
//
// `run` is resumable: a (title, phase) whose JSON is already in --out is skipped, so
// re-running after a pin bump means deleting --out (or pointing at a new one). `report`
// re-reads --out and writes census.tsv (private) + clusters.md (sha only) + compat.json.
// `--prs` is `gh pr list -R Jun025/wie --state merged -L 1000 --json number,title,mergedAt,mergeCommit`;
// a PR whose title names a title's display name becomes a `changes[]` row for it.
// `--changes` (sha256 -> [{date,enginePin,summary_ko}]) is merged on top, for rows a
// PR title does not name.
//
// ── What each axis measures, and its ceiling ─────────────────────────────────
// probe A = `--inject` 27 keys on a fixed budget (--secs) with `--pacing 8`;
// probe B = the same budget with 0 keys and a shot every second (the unkeyed baseline).
//   boot    fail when neither probe painted and A failed; else ok.
//   render  ok = some frame had >=2 colours; uniform = painted but never did; none = 0 paints.
//   input   ok when A shows a frame B never did (PNG bytes are a pure function of pixels).
//           ponytail: an attract-mode title that animates on its own reads as "ok" —
//           a per-shot diff against a time-aligned baseline is the upgrade if that bites.
//   longplay  titles with boot/render/input ok get --long seconds of a looping key script
//           with a shot every 20 s: error = FAIL line. Never `stall` (see judge()); the longest run
//           of identical shots goes to census.tsv as `still`. A title whose 30 s probe already
//           failed after painting is `error` without the long run.
//   sound   ok = a Play with events reached the sink in ANY run (probes, long, speed); silent = none did. This is the
//           engine side only: a command the browser host drops is #348's axis, not this one.
//   speed   1 - (sleep lateness + timer lateness + GC) / window, the #347 ratio, headless: `ok` at
//           >= 0.9, else `n/a` — never `slow` (see judge()). load1 is recorded beside it.
//           `--speed` overrides that with browser runs (the #347 harness, one JSON line per run:
//           `game` "<sha12>.zip", `loopHz`, `win`, `pacing`): two runs within 10% of each other
//           give `ok` (>= 0.9) or `slow`; runs that disagree leave the headless verdict standing.
//   progress  «does it keep moving forward», not «did it survive»: the progress policy for
//           --progress seconds, `stuck` when no never-seen frame came in the last third (pair-confirmed
//           by `--as P2`). Outside `status`. See progress() below.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { cpus, loadavg } from 'node:os';
import { inflateRawSync, inflateSync } from 'node:zlib';
import { basename, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const EXCLUDED = ['_dup', '_nongame', 'vendor_sdk'];
const PROBE_KEYS_AT = 8; // pacing window opens after boot
// Never CLR / soft keys: those quit many titles, and a clean exit ends the run early.
// A title that ends its FIRST run on purpose (a «다시 실행해 주세요» notice after writing a marker
// database) is relaunched once, database kept — what a player does. Without it such a title reads
// as boot/render fail on every census (7da00ecd4804, 2026-09-29).
const RELAUNCH = ['--relaunch', '1'];
// Every run here is bounded by --timeout. wie_validate's 50M-tick default is an infinite-loop backstop
// sized for a boot, and a fast title burns it before the key schedule is done: on 12 of 17 input `none`
// titles the 30 s probe A stopped at `max-ticks` after 2–17 of 27 keys, and 7 of them read `ok` once it
// was lifted (docs/report/0449). So no run here keeps the backstop.
const NO_TICK_CAP = ['--max-ticks', '100000000000'];
const LONG_KEYS = 'OK:1 UP:0.5 UP:0.5 OK:1 DOWN:0.5 RIGHT:0.5 NUM5:1 LEFT:0.5 NUM5:1 OK:1 NUM2:0.5 NUM8:0.5 NUM4:0.5 NUM6:0.5 OK:1';
// The progress policy (~30 s a cycle): confirm through notices and menus (OK/5, one left soft key,
// one NUM1 for «1. 예» notices), then play — directions tapped and held, fire repeatedly. Never CLR
// or the right soft key (back/quit on most titles); a title the left soft key quits is relaunched.
const PROGRESS_KEYS = 'OK:1 NUM5:1 OK:1 NUM1:1 OK:1 LSOFT:1.5 OK:1 NUM5:1 UP:0.5 DOWN:0.5 DOWN:0.5 OK:1 RIGHT:0.5:1.5 LEFT:0.5:1.5 NUM6:0.5:1 NUM4:0.5:1 NUM2:0.5 NUM8:0.5 NUM5:0.4 NUM5:0.4 NUM5:0.4 UP:0.5:1 NUM5:0.4 DOWN:0.5:1 NUM5:0.4 OK:1';
const PROGRESS_SHOT = 10; // seconds between timed shots
// Policy v2 (2026-10-02): the v1 cycle as-is, plus — only once the screen has stopped bringing
// anything new for PROGRESS_STALL s — one escape, rotating through these. They are the keys v1 never
// presses (back, the right soft key) and «the next menu item» that its OK-spam cannot reach. v1's ⒜
// cluster (35 of 82 stuck) was mostly «no way back» out of help, info and empty-save screens.
// A key that quits the title is answered by a relaunch, database kept.
const PROGRESS_STALL = 60;
const PROGRESS_ESCAPES = ['CLR:2', 'RSOFT:2', 'DOWN:0.5 OK:2', 'CLR:1 CLR:2', 'DOWN:0.5 DOWN:0.5 OK:2', 'UP:0.5 OK:2', 'NUM0:1 HASH:1 STAR:2'];
// After the progress window the guest is booted again, database kept (`--restart-at`), and the
// policy runs RESUME s more: «does a save made in play come back?» (`db.resumed_reads`). The
// progress curve is judged on the window before the restart only, so v1 and v2 compare.
const PROGRESS_RESUME = 120;

const [cmd, ...rest] = process.argv.slice(2);
// Default half the cores but at most 3, never more than all of them: each job is a CPU-bound
// wie_validate, so past ncpu the extra ones only add context switches (2026-09-29: --jobs 32 on 10
// cores -> load1 450, sys 80%+, idle 0%) and starve the wall-clock probes into UNMEASURED. The 3 is
// the host's short build-slot count on 10 cores: a census shares the Mac with every other lane's builds
// (2026-10-02: 20 emulator processes at load1 240 — CLAUDE.md «측정 스윕»). The run itself takes ONE
// `build-slot run --long` lease and these jobs run inside it — they do not take slots of their own.
function jobsFor(requested, ncpu) {
  if (requested === undefined) return Math.max(1, Math.min(3, Math.floor(ncpu / 2)));
  const n = Math.max(1, Math.floor(Number(requested)) || 1);
  return Math.min(n, ncpu);
}
// One `run` per host. The --jobs cap only bounds one run; 2026-09-29 two runs from two scratch
// copies (10 + 10 jobs on 10 cores) put load1 back at 528-592. So the lock is a FIXED path, not
// one derived from the script's location, and a second run WAITS rather than refusing — a ticket
// must not fail quietly because another lane got there first. mkdir is the atomic test-and-set.
const LOCK = process.env.WIE_CENSUS_LOCK || '/tmp/wie-playability-census.lock';
const alive = (pid) => {
  try {
    process.kill(pid, 0);
    return true;
  } catch (e) {
    return e.code === 'EPERM';
  }
};
// ponytail: a dead holder whose pid was reused reads as alive (waits until that pid exits), and two
// waiters reclaiming the same stale lock at once can both proceed; pid+start-time is the upgrade.
async function hostLock(dir, pollMs = 2000) {
  for (let told = false; ; ) {
    try {
      mkdirSync(dir);
      writeFileSync(join(dir, 'pid'), String(process.pid));
      process.on('exit', () => {
        try {
          if (readFileSync(join(dir, 'pid'), 'utf8') === String(process.pid)) rmSync(dir, { recursive: true, force: true });
        } catch {}
      });
      return;
    } catch (e) {
      if (e.code !== 'EEXIST') throw e;
    }
    let pid = NaN;
    try {
      pid = Number(readFileSync(join(dir, 'pid'), 'utf8'));
    } catch {}
    let age = 0;
    try {
      age = Date.now() - statSync(dir).mtimeMs;
    } catch {}
    // No pid file yet is a holder between mkdir and write — unless it has been like that for 10 s.
    if (pid ? !alive(pid) : age > 10000) {
      console.error(`census lock ${dir}: holder ${pid || '?'} is gone — reclaiming`);
      rmSync(dir, { recursive: true, force: true });
      continue;
    }
    if (!told) console.error(`census lock ${dir}: another run (pid ${pid || '?'}) holds this host — waiting`);
    told = true;
    await new Promise((r) => setTimeout(r, pollMs));
  }
}
const opt = { secs: 30, long: 600, dirs: [] };
for (let i = 0; i < rest.length; i++) {
  const a = rest[i];
  if (a.startsWith('--')) opt[a.slice(2)] = rest[++i];
  else opt.dirs.push(a);
}
{
  const ncpu = cpus().length;
  const want = opt.jobs;
  // A progress run holds a core for 10–30 minutes, not 30 s: default to a quarter of the cores so a
  // bare `--only progress` leaves room for the other lanes (2026-09-30: 8–10 jobs starved a sibling
  // lane's host-load gate for an hour).
  opt.jobs = want === undefined && opt.only === 'progress' ? Math.max(1, Math.floor(ncpu / 4)) : jobsFor(want, ncpu);
  if (want !== undefined && Number(want) > ncpu) console.error(`--jobs ${want} > ${ncpu} cores — capped to ${opt.jobs}`);
}
opt.secs = Number(opt.secs);
opt.long = Number(opt.long);
opt.progress = Number(opt.progress ?? 1800);
if (cmd !== 'selftest' && (!opt.out || !['run', 'report'].includes(cmd))) {
  console.error('usage: playability-census.mjs run|report --out <dir> … | selftest  (see the header)');
  process.exit(2);
}
const out = cmd === 'selftest' ? null : resolve(opt.out);
if (out) mkdirSync(out, { recursive: true });
const sha256 = (buf) => createHash('sha256').update(buf).digest('hex');

// ── population: one entry per sha256, first path wins ──────────────────────
function population(dirs) {
  const seen = new Map();
  const excluded = {};
  let files = 0;
  const walk = (d) => {
    for (const e of readdirSync(d, { withFileTypes: true })) {
      const p = join(d, e.name);
      if (e.isDirectory()) {
        if (EXCLUDED.includes(e.name)) excluded[e.name] = (excluded[e.name] ?? 0) + 1;
        else walk(p);
      } else if (/\.(zip|jar)$/i.test(e.name)) {
        files++;
        const h = sha256(readFileSync(p));
        if (!seen.has(h)) seen.set(h, p);
      }
    }
  };
  for (const d of dirs) walk(resolve(d));
  return { titles: [...seen].map(([sha, path]) => ({ sha, path })).sort((a, b) => a.sha.localeCompare(b.sha)), files, excluded };
}

function validate(args, killAfterSecs, stderrPath) {
  return new Promise((done) => {
    const child = spawn(opt.bin, args, { env: { ...process.env, RUST_LOG: 'warn', RUST_MIN_STACK: '4194304' } });
    let stdout = '';
    const tail = [];
    let warns = 0;
    let nets = 0;
    let deaths = 0;
    let partial = '';
    child.stdout.on('data', (b) => (stdout += b));
    child.stderr.on('data', (b) => {
      const lines = (partial + b).split('\n');
      partial = lines.pop();
      for (const l of lines) {
        if (/\bWARN\b|panicked/.test(l)) warns++;
        nets += (l.match(NET_CONNECT) ?? []).length; // the whole stream: the tail below keeps 200 lines
        if (UNCAUGHT.test(l)) deaths++;
        tail.push(l);
        if (tail.length > 200) tail.shift();
      }
    });
    const timer = setTimeout(() => child.kill('SIGKILL'), killAfterSecs * 1000);
    child.on('close', (code, signal) => {
      clearTimeout(timer);
      writeFileSync(stderrPath, tail.join('\n'));
      const line = stdout.trim().split('\n').pop() ?? '';
      let r;
      try {
        r = JSON.parse(line);
      } catch {
        // A signal here is the process dying (SIGABRT = an abort inside the engine), or our own
        // SIGKILL at killAfterSecs; the reason says which.
        r = { result: 'FAIL', reason: signal ? `died on ${signal}${signal === 'SIGKILL' ? ` (census kill at ${killAfterSecs}s)` : ''}` : `no result line (rc=${code})`, paints: 0 };
      }
      r.rc = code;
      r.stderr_warns = warns;
      r.net_connects = nets;
      r.uncaught_threads = deaths;
      r.load1 = loadavg()[0];
      done(r);
    });
  });
}

const shotHashes = (dir) =>
  existsSync(dir)
    ? readdirSync(dir)
        .filter((f) => f.endsWith('.png'))
        .sort()
        .map((f) => sha256(readFileSync(join(dir, f))))
    : [];

async function probe(t) {
  const d = join(out, t.sha);
  mkdirSync(d, { recursive: true });
  for (const [name, extra] of [
    ['A', ['--pacing', String(PROBE_KEYS_AT)]],
    ['B', ['--inject-keys', '0', '--shot-every', '1']],
  ]) {
    const f = join(d, `${name}.json`);
    if (existsSync(f)) continue;
    mkdirSync(join(d, name), { recursive: true });
    const args = ['--inject', '--keep-timeout', '--timeout', String(opt.secs), ...NO_TICK_CAP, '--shotdir', join(d, name), ...RELAUNCH, ...extra, t.path];
    const r = await validate(args, opt.secs + 120, join(d, `${name}.stderr`));
    // A probe the host starved is not a measurement: it is left unrecorded, so the next `run` retries
    // it, instead of reading as `boot: fail`. Measured 2026-09-28: next to two Interactive-priority
    // browsers, 296 of 429 probes ended at the deadline having ticked < 100 times with 0 paints; the
    // previous run's real boot failures that ended at the deadline had all ticked >= 100.
    if (starvedProbe(r)) {
      starved++;
      return;
    }
    r.shots = shotHashes(join(d, name));
    writeFileSync(f, JSON.stringify(r));
  }
}
let starved = 0;
const starvedProbe = (r) => r.stop === 'deadline' && !r.paints && (r.ticks ?? 0) < 100;

// Speed is wall-clock, so a starved host reads as a slow game. `--only speed` re-measures the
// titles that read below 0.9, alone (--jobs 1-2 is the point), into S.json; the judge takes the
// better of the two readings, since each is a lower bound.
async function speed(t) {
  const d = join(out, t.sha);
  const f = join(d, 'S.json');
  if (existsSync(f)) return;
  const args = ['--inject', '--keep-timeout', '--timeout', String(opt.secs), ...NO_TICK_CAP, '--pacing', String(PROBE_KEYS_AT), t.path];
  writeFileSync(f, JSON.stringify(await validate(args, opt.secs + 120, join(d, 'S.stderr'))));
}

const longKeys = (recipe, reps) => [recipe, ...Array(reps).fill(LONG_KEYS)].join('\n');
async function longplay(t, spec = {}) {
  const d = join(out, t.sha);
  const f = join(d, 'L.json');
  if (existsSync(f)) return;
  mkdirSync(join(d, 'L'), { recursive: true });
  const keys = join(d, 'long.keys');
  const reps = Math.ceil(opt.long / 10);
  // A `--titles` recipe is a prefix here too: the loop alone never leaves some logos and menus, and
  // a title whose music starts in play then reads `silent` (15 such, docs/report/0388 §1).
  writeFileSync(keys, longKeys(spec.keys ? readFileSync(spec.keys, 'utf8') : '', reps));
  // The backstop ended this run's first pass at 3 of 10 minutes (NO_TICK_CAP).
  const args = ['--inject', '--keys', keys, '--keep-timeout', '--timeout', String(opt.long), ...NO_TICK_CAP, '--shotdir', join(d, 'L'), '--shot-every', '20', ...RELAUNCH, t.path];
  const r = await validate(args, opt.long + 300, join(d, 'L.stderr'));
  // `--keys` also shoots once per key step; only the `tNNN.N` timer shots are evenly spaced.
  const timed = existsSync(join(d, 'L'))
    ? readdirSync(join(d, 'L'))
        .filter((n) => shotTime(n) !== null)
        .sort(byShotTime)
    : [];
  r.shots = timed.map((n) => sha256(readFileSync(join(d, 'L', n))));
  r.recipe = spec.keys ? basename(spec.keys) : null;
  writeFileSync(f, JSON.stringify(r));
}

// ── progress: does the title keep moving FORWARD, not merely stay alive ─────
// `--only progress` runs the progress policy (PROGRESS_KEYS, or a per-title recipe) for --progress
// seconds with a shot every PROGRESS_SHOT s, into `<as>.json` (`--as P`, `--as P2` for the pair
// re-measure). The measurement is the curve of NEW screens and `stall` = seconds from the last new
// screen to the end of the run. A screen is a 16×16 grid of mean luminance (fingerprint()), and a
// shot is new when it differs from EVERY screen seen so far in >= 8 cells by > 32 levels.
// Why not the PNG hash (the first version): measured on this run, a slot picker with a glowing
// cursor produced 27 distinct hashes in 30 min and a text-entry screen cycling letters 76, both read
// `ok` while going nowhere. On the same 6 titles 32/8 calls those two and the frozen menu stuck and
// keeps the two that play (new screens until 1300 s and 430 s of their runs) ok; 24/3 still let the
// slot picker through.
//   ponytail: a board game whose cursor walks the board without a move reads `ok` (a chess title,
//   measured) — screen novelty cannot tell a cursor from play. Per-title recipes are the answer.
// `--titles <file>` limits the run to listed titles, one per line: `<sha12> [secs] [recipe keys file]`.
const titleList = () => {
  if (!opt.titles) return null;
  const m = new Map();
  for (const l of readFileSync(resolve(opt.titles), 'utf8').split('\n')) {
    const [sha, secs, keys] = l.split('#')[0].trim().split(/\s+/);
    if (sha) m.set(sha.slice(0, 12), { secs: secs ? Number(secs) : null, keys: keys ? resolve(keys) : null });
  }
  return m;
};
async function progress(t, spec = {}) {
  const stem = opt.as ?? 'P';
  const d = join(out, t.sha);
  const f = join(d, `${stem}.json`);
  if (existsSync(f)) return;
  const secs = spec.secs ?? opt.progress;
  const shots = join(d, stem);
  rmSync(shots, { recursive: true, force: true });
  mkdirSync(shots, { recursive: true });
  const keys = join(d, `${stem}.keys`);
  const v2 = opt.policy !== 'v1';
  const total = v2 ? secs + PROGRESS_RESUME : secs;
  // A recipe is a PREFIX (the path to where play starts — an ⒜ unlock), then the policy as usual.
  writeFileSync(keys, [spec.keys ? readFileSync(spec.keys, 'utf8') : '', ...Array(Math.ceil(total / 25)).fill(PROGRESS_KEYS)].join('\n'));
  const policy = v2 ? ['--stall-secs', String(PROGRESS_STALL), ...PROGRESS_ESCAPES.flatMap((e) => ['--stall-keys', e]), '--restart-at', String(secs), '--relaunch', '8'] : ['--relaunch', '3'];
  const args = ['--inject', '--keys', keys, '--keep-timeout', '--timeout', String(total), ...NO_TICK_CAP, '--shotdir', shots, '--shot-every', String(PROGRESS_SHOT), ...policy, t.path];
  const r = await validate(args, total + 300, join(d, `${stem}.stderr`));
  const all = readdirSync(shots).filter((n) => n.endsWith('.png'));
  const timed = all.filter((n) => shotTime(n) !== null).sort(byShotTime);
  // Keep the timed frames (the curve's evidence, and the stuck frame to look at); drop per-key ones.
  for (const n of all) if (!timed.includes(n)) rmSync(join(shots, n));
  r.shots = timed.map((n) => sha256(readFileSync(join(shots, n))));
  r.fp = timed.map((n) => fingerprint(readFileSync(join(shots, n))).map(Math.round));
  r.shot_names = timed;
  r.secs = secs;
  r.policy = v2 ? 'v2' : 'v1';
  r.recipe = spec.keys ? basename(spec.keys) : null;
  writeFileSync(f, JSON.stringify(r));
}
// `--shot-every` frame time from its name (`<stem>__t1000.0.png` -> 1000), null for a per-key shot.
const shotTime = (n) => {
  const m = /__t(\d+\.\d)\.png$/.exec(n);
  return m ? Number(m[1]) : null;
};
const byShotTime = (a, b) => shotTime(a) - shotTime(b);
// A run recorded before `fp` existed is fingerprinted from its kept frames.
function readProgress(d, stem) {
  const r = read(join(d, `${stem}.json`));
  if (r && !r.fp) r.fp = r.shot_names.map((n) => fingerprint(readFileSync(join(d, stem, n))).map(Math.round));
  return r;
}
// Seconds of a run that brought no never-seen frame. A run that ended early (the guest quit and
// used up its relaunches) counts the missing time as stalled, which is what a player would see.
// The time is read off the name, never the index: names sort as text, and `t1000.0` < `t990.0`.
// Until 2026-10-02 the frames were sorted that way, so every run past 1000 s (the 30-min reps of
// docs/report/0393) was judged on frames out of order. A run recorded then is re-ordered here.
function progressCurve(r) {
  const seen = [];
  const uniq = [];
  let lastNew = 0;
  const at = r.fp.map((f, i) => [r.shot_names ? shotTime(r.shot_names[i]) : (i + 1) * PROGRESS_SHOT, f]).sort((a, b) => a[0] - b[0]);
  for (const [t, f] of at) {
    if (t > r.secs) continue; // the resume tail after `--restart-at` is not progress
    if (seen.every((s) => s.filter((v, k) => Math.abs(v - f[k]) > 32).length >= 8)) {
      seen.push(f);
      lastNew = t;
    }
    uniq.push(seen.length);
  }
  return { uniq, lastNew, stall: r.secs - lastNew, distinct: seen.length };
}
// 16×16 mean luminance of an 8-bit RGBA, non-interlaced PNG — what wie_validate's --shotdir writes
// (`png::ColorType::Rgba`, `BitDepth::Eight`). Anything else throws rather than guessing.
function fingerprint(png, G = 16) {
  const w = png.readUInt32BE(16);
  const h = png.readUInt32BE(20);
  if (png[24] !== 8 || png[25] !== 6 || png[28] !== 0) throw new Error('fingerprint: not an 8-bit RGBA non-interlaced PNG');
  const idat = [];
  for (let p = 8; p < png.length; ) {
    const n = png.readUInt32BE(p);
    if (png.toString('latin1', p + 4, p + 8) === 'IDAT') idat.push(png.subarray(p + 8, p + 8 + n));
    p += 12 + n;
  }
  const raw = inflateSync(Buffer.concat(idat));
  const stride = w * 4;
  const px = Buffer.alloc(h * stride);
  for (let y = 0; y < h; y++) {
    const f = raw[y * (stride + 1)];
    const src = y * (stride + 1) + 1;
    const dst = y * stride;
    for (let x = 0; x < stride; x++) {
      const a = x >= 4 ? px[dst + x - 4] : 0;
      const b = y ? px[dst - stride + x] : 0;
      const c = x >= 4 && y ? px[dst - stride + x - 4] : 0;
      const pa = Math.abs(b - c);
      const pb = Math.abs(a - c);
      const pc = Math.abs(a + b - 2 * c);
      const pred = [0, a, b, (a + b) >> 1, pa <= pb && pa <= pc ? a : pb <= pc ? b : c][f];
      px[dst + x] = (raw[src + x] + pred) & 255;
    }
  }
  const sum = new Float64Array(G * G);
  const cnt = new Float64Array(G * G);
  for (let y = 0; y < h; y++)
    for (let x = 0; x < w; x++) {
      const i = y * stride + x * 4;
      const k = Math.floor((y * G) / h) * G + Math.floor((x * G) / w);
      sum[k] += 0.299 * px[i] + 0.587 * px[i + 1] + 0.114 * px[i + 2];
      cnt[k]++;
    }
  return Array.from(sum, (v, k) => v / cnt[k]);
}
// What happened to the guest's saves in a v2 run: `resume` = a record written in play was read back
// after the restart · `saved` = it wrote, nothing came back · `none` = no write at all. '' = not recorded.
const saveOf = (r) => (!r?.db ? '' : r.db.resumed_reads > 0 ? 'resume' : r.db.writes > 0 ? 'saved' : 'none');
// ok | stuck | error for one run. The stall line is a third of the run (10 min of 30), floored at 3 min.
function progressRun(r) {
  if (!r) return null;
  if (r.result === 'FAIL') return 'error';
  const { stall } = progressCurve(r);
  return stall >= Math.max(180, r.secs / 3) ? 'stuck' : 'ok';
}
// A `stuck` stands only when the pair re-measure (P2) agrees: one slow run on a loaded host looks
// exactly like a stall. `stuck` without P2 is `n/a` — not measured, not guessed.
function progressAxis(P, P2) {
  const a = progressRun(P);
  if (!a) return 'n/a';
  if (a !== 'stuck') return a;
  // Only a pair that actually moved overrules the stall; a pair that crashed or is missing proves nothing.
  const b = progressRun(P2);
  return b === 'ok' ? 'ok' : b === 'stuck' ? 'stuck' : 'n/a';
}

async function pool(items, jobs, fn) {
  let i = 0;
  let n = 0;
  await Promise.all(
    Array.from({ length: jobs }, async () => {
      while (i < items.length) {
        const t = items[i++];
        await fn(t);
        if (++n % 10 === 0) console.error(`  ${n}/${items.length}  load1=${loadavg()[0].toFixed(0)}`);
      }
    }),
  );
}

const read = (p) => (existsSync(p) ? JSON.parse(readFileSync(p, 'utf8')) : null);

// #347's ratio for one browser run: 1 - engine-added ms per frame / actual frame period. The game
// thread waits on its own wake lateness, timer lateness and GC always, and on the paint only when it
// spins on yield. It is a ratio against a period the game ASKED for, so a run with no requested wait
// says nothing (a yield spin painting 246/s read 0.07; `sleep(0)` asks for nothing, so its "lateness"
// is the yield's cost — one painting 200/s read 0.75), and neither does lateness past the whole frame
// (a fixed-rate timer that has fallen behind read -6.2). Several waits or several blits per frame can
// only push the ratio DOWN (lateness summed over waits that may overlap; a frame counted twice), so
// outside one wait per frame a reading >= 0.9 still stands and one below it is dropped. Measured 2026-09-28.
function browserRatio(r) {
  const p = r.pacing;
  if (r.err || !p || !r.loopHz) return null;
  const frames = r.loopHz * r.win;
  const sleeps = p.sleep_ms_p95 > 0 ? p.sleeps : 0;
  const waits = sleeps + p.timers;
  if (!waits) return null;
  const spin = p.paints && p.yields > p.paints * 20;
  const added = ((sleeps ? p.sleep_late_sum : 0) + p.timer_late_sum + (spin ? p.redraw_sum : 0) + p.gc_ms) / frames;
  const ratio = 1 - added / (1000 / r.loopHz);
  const oneWaitPerFrame = waits >= frames / 2 && waits <= frames * 2;
  return ratio < 0 || (ratio < 0.9 && !oneWaitPerFrame) ? null : ratio;
}
const browserVerdict = (xs) => (xs.length >= 2 && Math.min(...xs) >= 0.9 * Math.max(...xs) ? [Math.min(...xs), Math.min(...xs) >= 0.9 ? 'ok' : 'slow'] : null);
const browser = new Map();
if (opt.speed)
  for (const l of readFileSync(resolve(opt.speed), 'utf8').split('\n')) {
    if (!l.startsWith('{')) continue;
    const r = JSON.parse(l);
    const x = r.game && browserRatio(r);
    if (x === null || x === undefined) continue;
    const k = r.game.slice(0, 12);
    browser.set(k, [...(browser.get(k) ?? []), x]);
  }

function judge(sha) {
  const d = join(out, sha);
  const A = read(join(d, 'A.json'));
  const B = read(join(d, 'B.json'));
  const L = read(join(d, 'L.json'));
  const S = read(join(d, 'S.json'));
  const P = readProgress(d, 'P');
  const P2 = readProgress(d, 'P2');
  if (!A || !B) return null;
  const painted = (A.paints ?? 0) + (B.paints ?? 0) > 0;
  const content = A.content || B.content;
  const ax = {};
  ax.boot = !painted && A.result === 'FAIL' ? 'fail' : 'ok';
  ax.render = content ? 'ok' : painted ? 'uniform' : 'none';
  const ok2 = ax.boot === 'ok' && ax.render === 'ok';
  const baseline = new Set(B.shots);
  const novel = A.shots.filter((h) => !baseline.has(h)).length;
  const inputPanic = /panic on input/.test(A.reason ?? '');
  ax.input = !ok2 ? 'n/a' : novel > 0 && !inputPanic ? 'ok' : 'none';
  const probeErr = painted && A.result === 'FAIL' && /panic|error|killed|SVC stub/i.test(A.reason ?? '');
  ax.longplay = longplayVerdict(probeErr, L, join(d, 'L.stderr'));
  ax.sound = soundVerdict(ax.boot, [A, B, L, S]);
  // Lateness is wall-clock, so host load only ever ADDS to it: a ratio measured on a busy host is a
  // lower bound. >= 0.9 there is a real `ok`; below it says nothing (measured 2026-09-27 at load1
  // ~300: one title read 0.22 and 0.85 on two runs, another 0.49 headless and 0.927 in a quiet
  // browser). So headless never says `slow` — it says `n/a`, and the ratio stays in census.tsv.
  const lateRatio = (p) => (p && p.sleeps + p.timers > 0 ? 1 - (p.sleep_late_sum + p.timer_late_sum + p.gc_ms) / windowMs : null);
  const windowMs = (opt.secs - PROBE_KEYS_AT) * 1000;
  const ratios = [A, S].map((r) => lateRatio(r?.pacing)).filter((r) => r !== null);
  let ratio = ratios.length ? Math.max(...ratios) : null;
  ax.speed = ok2 && ratio !== null && ratio >= 0.9 ? 'ok' : 'n/a';
  // A browser pair decides only when it agrees with itself: the host moves one run, not two alike.
  const br = browser.get(sha.slice(0, 12)) ?? [];
  const bv = ok2 ? browserVerdict(br) : null;
  if (bv) [ratio, ax.speed] = bv;
  // Not part of `status` (see status()): whether a stuck title should read `limited` is the
  // operator's wording to decide, so the axis is shipped beside the six, not folded into them.
  ax.progress = progressAxis(P, P2);
  return { A, B, L, S, P, P2, ax, ratio, br, novel, baselineDistinct: baseline.size };
}

// Sound is judged over EVERY run of the title, not the 30 s probes alone: many titles start their
// music only past the title screen, which the 27-key probe often never leaves. Measured 2026-09-30 on
// the 3c34efee census: 10 of 75 `silent` titles played sound in the 600 s long run (up to 508 plays).
const soundVerdict = (boot, runs) => {
  const au = runs.map((r) => r?.audio).filter(Boolean);
  if (boot !== 'ok' || !au.length) return 'n/a';
  return au.some((a) => a.plays - (a.empty_plays ?? 0) > 0) ? 'ok' : 'silent';
};

function maxRun(hs) {
  let best = 0;
  let run = 0;
  for (let i = 0; i < hs.length; i++) {
    run = i > 0 && hs[i] === hs[i - 1] ? run + 1 : 1;
    best = Math.max(best, run);
  }
  return best;
}

const status = (ax) =>
  ax.boot === 'ok' && ax.render === 'ok' && ax.input === 'ok' && ax.longplay === 'ok' ? 'playable' : ax.boot === 'ok' && ax.render === 'ok' ? 'limited' : 'not-yet';

const ISSUE_KO = {
  'boot:fail': '아직 실행되지 않아요(시작하는 도중에 멈춰요).',
  'render:uniform': '화면이 한 가지 색으로만 보여요.',
  'render:none': '게임 화면이 아직 나오지 않아요.',
  'input:none': '키를 눌러도 화면이 바뀌지 않을 수 있어요.',
  'longplay:error': '플레이 도중에 게임이 멈추거나 꺼질 수 있어요.',
  'longplay:stall': '오래 플레이하면 화면이 멈춘 채로 있을 수 있어요.',
  'sound:silent': '소리가 나지 않을 수 있어요.',
  'speed:slow': '원래보다 조금 느리게 움직일 수 있어요.',
};

// Numbers, addresses and hex are what differ between two titles hitting the same wall.
// First line + the first platform frame; a panic gets its source location from stderr.
// ★Only platform paths survive: an app's own class names can spell its title.
const PLATFORM = /^(java|javax|org\/kwis|com\/skt|com\/ktf|com\/lgt|com\/xce|net\/wie|mmpp|wipi)\//;
function wallOf(reason, stderrPath) {
  const lines = (reason ?? '').split('\n').map((l) => l.trim());
  // A register dump (`R0: 0x…`) follows the message on ARM errors; it differs per title.
  const head = lines.filter((l) => !l.startsWith('at ')).join(' ').replace(/\s+R\d+:.*$/, '');
  const frame = lines.filter((l) => l.startsWith('at ')).map((l) => l.slice(3)).find((l) => PLATFORM.test(l));
  let w = head.replace(/\b(?!(?:java|javax|org|com|net)\b)[\w$]+(\.[\w$<>]+\()/g, '<app>$1') + (frame ? ` @ ${frame}` : '');
  // Two spellings of one death (the first pass wrote `killed (SIGABRT) after Ns`); stderr says why.
  if (/SIGABRT/.test(head)) {
    w = 'died on SIGABRT';
    if (stderrPath && existsSync(stderrPath) && /overflowed its stack/.test(readFileSync(stderrPath, 'utf8'))) w += ' @ host stack overflow';
  }
  if (/panic/.test(head) && stderrPath && existsSync(stderrPath)) {
    const at = /panicked at (?:.*\/registry\/src\/[^/]+\/)?(\S+?):\d+:\d+/.exec(readFileSync(stderrPath, 'utf8'));
    if (at) w += ` @ ${at[1]}`;
  }
  return w
    .replace(/0x[0-9a-f]+/gi, '0x#')
    .replace(/\d+/g, '#')
    .replace(/'[^']*'/g, "'…'")
    .slice(0, 160);
}

const summaryKo = (prTitle) =>
  /소리|audio|MIDI|BGM|효과음/i.test(prTitle)
    ? '소리가 나도록 고쳤어요.'
    : /속도|fps|느리|빠르|GC|sleep/i.test(prTitle)
      ? '게임 속도를 원래대로 맞췄어요.'
      : /멈|정지|hang|stall|freeze|교착/i.test(prTitle)
        ? '도중에 멈추던 문제를 고쳤어요.'
        : /키|입력|input|key/i.test(prTitle)
          ? '키 입력이 제대로 먹히도록 고쳤어요.'
          : /화면|렌더|render|paint|그리|검은|black|blank|글자|text|font/i.test(prTitle)
            ? '화면이 제대로 나오도록 고쳤어요.'
            : '게임이 더 안정적으로 실행되도록 고쳤어요.';

// "(KTF) 제목 [태그].zip" -> "제목"
// A PR names a title when the name stands alone ("X3" does not name "X") …
const names = (text, title) => title.length >= 2 && new RegExp(`(^|[^\\p{L}\\p{N}])${title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}($|[^\\p{L}\\p{N}])`, 'u').test(text);
// … and does not name a different carrier (one name, two carriers' builds).
const otherCarrier = (text, platform) => ['KTF', 'SKT', 'LGT'].some((c) => c !== platform && new RegExp(`\\b${c}\\b`, 'i').test(text)) && !new RegExp(`\\b${platform}\\b`, 'i').test(text);

// When the engine could not route an archive, name the carrier from the marker the zip's
// central directory spells (file names are stored as plain bytes there).
function sniffPlatform(path) {
  const bytes = readFileSync(path).toString('latin1');
  return /__adf__/.test(bytes) ? 'KTF' : /app_info/.test(bytes) ? 'LGT' : /\.msd/.test(bytes) ? 'SKT' : 'J2ME';
}

// ── locked files: titles that cannot run anywhere but the original buyer's handset ────────────
// Operator policy 2026-10-02: neither is to be unlocked (no phone-number recovery, no check bypass,
// no decryption) — the census says so in the player's words instead of «아직 실행되지 않아요».
//   drm    an OMA DRM container (`odcf` magic) where the jar should be — encrypted, decided statically.
//   phone  an SKT purchase check (XCE `SecureUtil`: MD5 of carrier + the buyer's phone number +
//          SERVICE_ID == `MIDlet-Key`, docs/report/0386) AND the title quit on its own.
//          The class alone is not enough: measured 2026-10-02, 41 titles carry it and 36 play — only
//          the ones whose check runs at start and fails end with `clean exit`. Some paint first: the
//          failing check draws «인증 되지 않은 컨텐츠» and then calls `System.exit` (docs/report/0420),
//          so the rule is «every run quit, the unkeyed one included», not «0 paints». On the bd2337ff
//          census that adds 2 of the 41 and no title that plays (those all end on `deadline`).
const LOCK_KO = {
  drm: '암호로 잠긴 파일이라 여기서는 실행할 수 없어요.',
  phone: '구매한 휴대폰에서만 켜지도록 잠긴 파일이라 여기서는 실행할 수 없어요.',
};
// Central directory only; tolerates bytes prepended to the archive (some SKT jars carry 32).
function zipEntries(buf) {
  const e = buf.lastIndexOf(Buffer.from('PK\x05\x06', 'latin1'));
  if (e < 0) return [];
  const shift = e - buf.readUInt32LE(e + 12) - buf.readUInt32LE(e + 16);
  const out = [];
  for (let p = buf.readUInt32LE(e + 16) + shift, i = buf.readUInt16LE(e + 10); i > 0 && buf.readUInt32LE(p) === 0x02014b50; i--) {
    const [method, size, nl] = [buf.readUInt16LE(p + 10), buf.readUInt32LE(p + 20), buf.readUInt16LE(p + 28)];
    const lo = buf.readUInt32LE(p + 42) + shift;
    out.push({
      name: buf.toString('latin1', p + 46, p + 46 + nl),
      data: () => {
        const s = lo + 30 + buf.readUInt16LE(lo + 26) + buf.readUInt16LE(lo + 28);
        return method === 0 ? buf.subarray(s, s + size) : inflateRawSync(buf.subarray(s, s + size));
      },
    });
    p += 46 + nl + buf.readUInt16LE(p + 30) + buf.readUInt16LE(p + 32);
  }
  return out;
}
const PHONE_CHECK = ['MIDlet-Key', 'SERVICE_ID=', 'MIDlet-Jar-URL'];
function lockOf(path) {
  const buf = readFileSync(path);
  const jars = /\.jar$/i.test(path) ? [buf] : zipEntries(buf).filter((x) => /\.jar$/i.test(x.name)).map((x) => x.data());
  if (jars.some((j) => j.toString('latin1', 0, 4) === 'odcf')) return 'drm';
  const check = jars.some((j) => zipEntries(j).some((c) => c.name.endsWith('.class') && PHONE_CHECK.every((k) => c.data().includes(k))));
  return check ? 'phone' : null;
}
// The run half of `phone`: the title ended itself — a run either quit or never painted.
const quitOnItsOwn = (runs) => runs.every((r) => r && (r.stop === 'clean exit' || !r.paints)) && runs.some((r) => r.stop === 'clean exit');
const lockVerdict = (lock, runs) => (lock === 'drm' || (lock === 'phone' && quitOnItsOwn(runs)) ? lock : null);

// ── titles that cannot go on without the original carrier's server ─────────────────────────────
// Operator policy 2026-10-02: the server is gone and its data is not ours to make up, so the census
// says so in the player's words, and `status` stays where the title really gets to (at most limited).
// The rule is the run, not the code: measured 2026-10-02 on the bd2337ff census, 30 titles reach a
// connect call and 29 of them play — they connect once (at most 5 times a run) and go on. A title
// that cannot go on keeps asking: f44271803135 retried 416 to 2,463 times in a 30 s probe. Two more that read
// as «network» on screen were engine walls with the data already shipped (docs/report/0414).
// Only the 30 s probe A counts: the 600 s key loop reconnects in titles that play (wave6 at pin
// 4ac38566 · L 66 · 66 · 29 · 28 in four of them) (docs/report/0432).
// The count is every connect line of the probe (`net_connects`, counted by validate() over the whole
// stream). Until 2026-10-04 it was read off A.stderr, which keeps only the last 200 lines — a density,
// not a count: the wall read 100 of its 2,463, and a wall logging 4+ other lines per retry would have
// read < 50. A probe recorded before the field falls back to `stub_hits.first` (the real count, but
// only for a top-5 stub — the wall's is) or that tail, whichever is larger; both are lower bounds.
// The count moves with host load: a 30 s window holds as many retries as the machine lets it run. The
// wall asked 2,438 and 2,463 times alone (load1 10–24) but 416–1,005 in the seven real censuses on disk
// (load1 160–630, 416 at the lowest). The busiest live title asked 21, 25, 33, 34 and 113 times alone
// and ≤ 2 in those censuses, so the old 50 would flag it one run in five. 200 is the line: 1.8× over
// 113, 2.1× under 416 — and on all ten censuses on disk it flags the same titles the 50-line tail did
// (500 missed the wall at 416). Reopen if a title lands in 113–416 (docs/report/0433).
const NET_KO = '게임을 시작하려면 옛 통신사 서버에서 데이터를 받아야 하는데, 그 서버가 지금은 없어 여기서는 진행할 수 없어요.';
const NET_CONNECT = /MC_netConnect|MC_netSocketConnect|MC_netHttpConnect|Network::connect\(/g;
const tailConnects = (text) => (text.match(NET_CONNECT) ?? []).length;
const stubConnects = (A) => (A.stub_hits?.first ?? []).filter((x) => /MC_net(Socket|Http)?Connect$|Network::connect$/.test(x.name)).reduce((n, x) => n + x.count, 0);
const netConnects = (A, stderrPath) =>
  A.net_connects ?? Math.max(stubConnects(A), existsSync(stderrPath) ? tailConnects(readFileSync(stderrPath, 'latin1')) : 0);
const netWall = (connects) => connects >= 200;
// A server wall the count cannot see: the title asks once per answer, fails, and loops on the same question
// (2 connects in probe A). Named one at a time from the frames and the stderr — docs/report/0445.
const NET_HAND = {
  b9bfcaf42722: '처음 실행할 때 게임사 서버에서 인증서를 받아야 하는데, 그 서버가 지금은 없어 여기서는 진행할 수 없어요.',
};

// ── longplay: a guest Java thread that dies uncaught did not survive ───────────────────────────
// rustjava logs `Uncaught exception in thread N:` when a guest thread's run() throws; the run then
// spins to the deadline with nothing left to drive the game, so its result is UNMEASURED or PASS,
// never FAIL. One heap exhaustion split one title into FAIL (OOM on the main tick) and «ok» (OOM on
// the game thread) and read as a regression (8d8c24b7c198, wave6). On the wave6 corpus all 4 non-FAIL
// longplays with the line had a frozen tail (21–29 of 29 timer shots identical, frozen_tail_steps
// 603–850) — the frozen tail alone would also flag 54 titles parked on a menu (docs/report/0434).
//   ponytail: any thread counts. A title whose helper thread dies while the game plays on would read
//   `error`; none on disk. Name the thread here if one turns up.
// The count is `uncaught_threads` (whole stream, validate()); a run recorded before it falls back to
// the 200-line L.stderr tail — a lower bound.
const UNCAUGHT = /uncaught exception in thread/i;
const guestDied = (L, stderrPath) => (L.uncaught_threads ?? +(existsSync(stderrPath) && UNCAUGHT.test(readFileSync(stderrPath, 'latin1')))) > 0;
const longplayVerdict = (probeErr, L, stderrPath) =>
  probeErr ? 'error' : !L ? 'n/a' : L.result === 'FAIL' || guestDied(L, stderrPath) ? 'error' : 'ok';
// ponytail: no `stall` verdict. Measured on this census's first pass: of 40 runs with 4+ identical
// 20 s shots, the ones opened were a sub-menu the key loop never backs out of (no CLR) and a
// notice waiting for NUM1 — the script's ceiling, not a frozen engine — and on a starved host a
// live title paints too rarely to tell. `still` in census.tsv keeps the count for a human.
// The exception block after the line, up to the next timestamped log line — a cluster wall.
const uncaughtOf = (text) => /Uncaught exception in thread -?\d+:\n([\s\S]*?)(?=\n[^\n]*\d{4}-\d\d-\d\dT\d\d:|$)/.exec(text)?.[1] ?? null;

// NFC: macOS hands back file names decomposed (NFD), and PR titles are composed.
const displayTitle = (p) =>
  basename(p)
    .normalize('NFC')
    .replace(/\.(zip|jar)$/i, '')
    .replace(/^\s*\((KTF|SKT|LGT|J2ME)\)\s*/i, '')
    .replace(/\s*[[(（][^\])）]*[\])）]\s*$/, '')
    .trim();

if (cmd === 'selftest') {
  // One browser run whose ratio is 1 - added/period: 10 frames/s over 10 s, one 100 ms sleep per frame.
  const run = (p, extra = {}) => ({ loopHz: 10, win: 10, err: null, ...extra, pacing: { sleeps: 100, sleep_ms_p95: 100, sleep_late_sum: 0, timers: 0, timer_late_sum: 0, paints: 100, yields: 100, redraw_sum: 0, gc_ms: 0, ...p } });
  const near = (a, b) => a !== null && Math.abs(a - b) < 1e-9;
  const cases = [
    ['on time reads 1', near(browserRatio(run({})), 1)],
    ['20 ms late per 100 ms frame reads 0.8', near(browserRatio(run({ sleep_late_sum: 2000 })), 0.8)],
    ['a GC counts', near(browserRatio(run({ gc_ms: 500 })), 0.95)],
    ['an errored run says nothing', browserRatio(run({}, { err: 'exited' })) === null],
    ['sleep(0) asks for no period', browserRatio(run({ sleep_ms_p95: 0, sleep_late_sum: 2000 })) === null],
    ['a timer is a wait', near(browserRatio(run({ sleeps: 0, timers: 100, timer_late_sum: 1000 })), 0.9)],
    ['lateness past the frame says nothing', browserRatio(run({ sleep_late_sum: 20000 })) === null],
    ['5 waits per frame: 0.95 stands', near(browserRatio(run({ sleeps: 500, sleep_late_sum: 500 })), 0.95)],
    ['5 waits per frame: 0.8 is dropped', browserRatio(run({ sleeps: 500, sleep_late_sum: 2000 })) === null],
    ['one pair within 10% is a verdict', browserVerdict([0.8, 0.85])?.[1] === 'slow' && browserVerdict([0.95, 0.9])?.[1] === 'ok'],
    ['a pair that disagrees is not', browserVerdict([0.7, 0.85]) === null && browserVerdict([0.8]) === null],
    ['the verdict takes the lower reading', browserVerdict([0.95, 0.9])?.[0] === 0.9],
    ['a starved probe is not recorded', starvedProbe({ stop: 'deadline', paints: 0, ticks: 3 })],
    ['a probe that painted is', !starvedProbe({ stop: 'deadline', paints: 1, ticks: 3 })],
    ['a deadline after 100 ticks is a real hang', !starvedProbe({ stop: 'deadline', paints: 0, ticks: 100 })],
    ['an error is a real failure', !starvedProbe({ stop: 'error', paints: 0, ticks: 3 })],
    ['--jobs defaults to half the cores, at most 3', jobsFor(undefined, 10) === 3 && jobsFor(undefined, 4) === 2],
    ['--jobs default is at least 1', jobsFor(undefined, 1) === 1],
    ['--jobs above ncpu is capped', jobsFor('32', 10) === 10],
    ['--jobs within ncpu is kept', jobsFor('3', 10) === 3],
    ['sound heard only in the long run is ok', soundVerdict('ok', [{ audio: { plays: 0 } }, { audio: { plays: 0 } }, { audio: { plays: 3, empty_plays: 0 } }, null]) === 'ok'],
    ['empty plays everywhere are silent', soundVerdict('ok', [{ audio: { plays: 2, empty_plays: 2 } }, null, { audio: { plays: 0 } }]) === 'silent'],
    ['no audio record is n/a', soundVerdict('ok', [{}, null]) === 'n/a' && soundVerdict('fail', [{ audio: { plays: 1 } }]) === 'n/a'],
    ['--jobs 0 / garbage becomes 1', jobsFor('0', 10) === 1 && jobsFor('x', 10) === 1],
    ['a long-run recipe comes before the loop', longKeys('NUM2:3', 2).split('\n')[0] === 'NUM2:3' && longKeys('NUM2:3', 2).split('\n').length === 3],
  ];
  // Progress: 60 shots over 600 s. New frames until 400 s, then the same two alternating (a blink).
  {
    // A screen = one cell-block lit per index; the blink toggles 4 cells (< 8), so it is never new.
    const block = (k, b) => Math.floor(k / 8) === b % 32;
    const screen = (i) => Array.from({ length: 256 }, (_, k) => (block(k, i) || (i >= 32 && block(k, i + 16)) ? 255 : 0));
    const blinkOf = (i) => Array.from({ length: 256 }, (_, k) => (k < 4 && i % 2 ? 100 : 0));
    const run = (lastNewAt, extra = {}) => ({ result: 'UNMEASURED', secs: 600, ...extra, fp: Array.from({ length: 60 }, (_, i) => ((i + 1) * PROGRESS_SHOT <= lastNewAt ? screen(i) : blinkOf(i))) });
    const blink = run(400);
    cases.push(
      ['a blink after the last new frame is a stall', progressCurve(blink).stall === 600 - 410 && progressCurve(blink).distinct === 41],
      ['a stall under a third of the run is ok', progressRun(run(500)) === 'ok'],
      ['a stall of a third is stuck', progressRun(run(380)) === 'stuck'],
      ['a crash is error, whatever the curve', progressRun(run(600, { result: 'FAIL' })) === 'error'],
      ['stuck needs its pair', progressAxis(run(100), null) === 'n/a' && progressAxis(run(100), run(100)) === 'stuck'],
      ['a pair that moved overrules one slow run', progressAxis(run(100), run(600)) === 'ok'],
      ['a pair that crashed confirms nothing', progressAxis(run(100), run(100, { result: 'FAIL' })) === 'n/a'],
      ['ok and error need no pair', progressAxis(run(600), null) === 'ok' && progressAxis(run(600, { result: 'FAIL' }), null) === 'error'],
      ['not measured is n/a', progressAxis(null, null) === 'n/a'],
      // v2 runs PROGRESS_RESUME s past the window; new screens after the restart are not progress.
      ['the resume tail is outside the curve', (() => {
        const r = { result: 'UNMEASURED', secs: 480, fp: Array.from({ length: 60 }, (_, i) => ((i + 1) * PROGRESS_SHOT <= 200 || (i + 1) * PROGRESS_SHOT > 480 ? screen(i) : blinkOf(i))) };
        return progressCurve(r).stall === 480 - 210 && progressRun(r) === 'stuck';
      })()],
      ['frames are ordered by their time, not their name', (() => {
        // New screens at 10..990 s, then a blink to 1200 s: by name `t1000.0`.. sort before `t110.0`.
        const names = Array.from({ length: 120 }, (_, i) => `x__t${(i + 1) * 10}.0.png`).sort();
        const r = { result: 'UNMEASURED', secs: 1200, shot_names: names, fp: names.map((n) => (shotTime(n) <= 990 ? screen(shotTime(n) / 10 - 1) : blinkOf(shotTime(n) / 10))) };
        return progressCurve(r).lastNew === 1000 && progressRun(r) === 'ok';
      })()],
      ['a save read back after the restart is resume', saveOf({ db: { writes: 3, resumed_reads: 1 } }) === 'resume'],
      ['a save never read back is saved', saveOf({ db: { writes: 3, resumed_reads: 0 } }) === 'saved' && saveOf({ db: { writes: 0, resumed_reads: 0 } }) === 'none' && saveOf({}) === ''],
    );
  }
  // Locked files, on synthetic archives (stored entries; CRC is not read).
  {
    const storedZip = (files) => {
      const loc = [];
      const cen = [];
      let off = 0;
      for (const [name, body] of Object.entries(files)) {
        const n = Buffer.from(name, 'latin1');
        const data = Buffer.from(body, 'latin1');
        const l = Buffer.alloc(30);
        l.writeUInt32LE(0x04034b50, 0);
        l.writeUInt32LE(data.length, 18);
        l.writeUInt32LE(data.length, 22);
        l.writeUInt16LE(n.length, 26);
        const c = Buffer.alloc(46);
        c.writeUInt32LE(0x02014b50, 0);
        c.writeUInt32LE(data.length, 20);
        c.writeUInt32LE(data.length, 24);
        c.writeUInt16LE(n.length, 28);
        c.writeUInt32LE(off, 42);
        loc.push(l, n, data);
        cen.push(c, n);
        off += 30 + n.length + data.length;
      }
      const cd = Buffer.concat(cen);
      const e = Buffer.alloc(22);
      e.writeUInt32LE(0x06054b50, 0);
      e.writeUInt16LE(Object.keys(files).length, 8);
      e.writeUInt16LE(Object.keys(files).length, 10);
      e.writeUInt32LE(cd.length, 12);
      e.writeUInt32LE(off, 16);
      return Buffer.concat([...loc, cd, e]);
    };
    const tmp = join('/tmp', `wie-census-lock-${process.pid}`);
    mkdirSync(tmp, { recursive: true });
    const title = (name, jar, pad = 0) => {
      const p = join(tmp, name);
      writeFileSync(p, storedZip({ 'a.msd': 'MIDlet-Key: x', 'a.jar': Buffer.concat([Buffer.alloc(pad), jar]).toString('latin1') }));
      return p;
    };
    const check = storedZip({ 'b.class': `..${PHONE_CHECK.join('..')}..`, 'M.class': 'startApp' });
    const drm = title('drm.zip', Buffer.from('odcf\0\x02odrm', 'latin1'));
    const phone = title('phone.zip', check, 32);
    const plain = title('plain.zip', storedZip({ 'M.class': 'MIDlet-Key SERVICE_ID=' }));
    const quit = { stop: 'clean exit', paints: 0 };
    cases.push(
      ['an odcf jar is drm, whatever the run did', lockOf(drm) === 'drm' && lockVerdict('drm', [{ paints: 5 }, { paints: 5 }]) === 'drm'],
      ['the purchase check is found past prepended bytes', lockOf(phone) === 'phone'],
      ['a jar without all three properties is not locked', lockOf(plain) === null],
      ['the check that quit before painting is a lock', lockVerdict('phone', [quit, { stop: 'deadline', paints: 0 }]) === 'phone'],
      ['the check in a title that keeps running is not', lockVerdict('phone', [quit, { stop: 'deadline', paints: 3 }]) === null],
      ['the check that paints its refusal and quits is a lock', lockVerdict('phone', [{ stop: 'clean exit', paints: 7 }, { stop: 'clean exit', paints: 8 }]) === 'phone'],
      ['the wall under census load (416 connects) is a network wall', netWall(tailConnects('WARN stub MC_netConnect(0x1, 0xa)\n'.repeat(416)))],
      ['a live title asking 113 times while it starts is not', !netWall(tailConnects('WARN stub MC_netConnect(0x1, 0xa)\n'.repeat(113)))],
      ['a Java connect loop counts too', netWall(tailConnects('stub org.kwis.msf.io.Network::connect()\n'.repeat(416)))],
      ['a probe recorded with net_connects is read by it, not by its tail', netConnects({ net_connects: 2463 }, '/nonexistent') === 2463],
      ['an older probe is read off stub_hits, whose count is the whole run', netWall(netConnects({ stub_hits: { first: [{ name: 'MC_netConnect', count: 2463 }] } }, '/nonexistent'))],
      ['the check in a title that crashed is not', lockVerdict('phone', [{ stop: 'error', paints: 0 }, { stop: 'error', paints: 0 }]) === null],
    );
    rmSync(tmp, { recursive: true, force: true });
  }
  // netWall through validate(): a wall that logs four other stubs per retry has 600 connects, but
  // only 40 of them in the 200-line tail the old reading counted — under any line it could hold.
  {
    const tmp = join('/tmp', `wie-census-netwall-${process.pid}`);
    rmSync(tmp, { recursive: true, force: true });
    mkdirSync(tmp, { recursive: true });
    const bin = join(tmp, 'fake.sh');
    const retry = 'WARN stub MC_netConnect(0x1, 0xa)\\n' + 'WARN stub unk12-1\\n'.repeat(4);
    writeFileSync(bin, `#!/bin/sh\ni=0; while [ $i -lt 600 ]; do printf '${retry}' >&2; i=$((i+1)); done\necho '{"result":"PASS","paints":1}'\n`, { mode: 0o755 });
    const keep = opt.bin;
    opt.bin = bin;
    const r = await validate([], 30, join(tmp, 'A.stderr'));
    opt.bin = keep;
    const tail = tailConnects(readFileSync(join(tmp, 'A.stderr'), 'latin1'));
    cases.push(
      ['a wall with four stubs per retry has 40 connects in its tail — under the line', tail === 40 && !netWall(tail)],
      ['and validate() counts all 600 of them, so it is a wall', r.net_connects === 600 && netWall(netConnects(r, join(tmp, 'A.stderr')))],
    );
    rmSync(tmp, { recursive: true, force: true });
  }
  // longplay through validate(): a game thread that dies early, then 300 lines of a run spinning to
  // the deadline — the death is out of the 200-line tail, and the result line still says PASS.
  {
    const tmp = join('/tmp', `wie-census-death-${process.pid}`);
    rmSync(tmp, { recursive: true, force: true });
    mkdirSync(tmp, { recursive: true });
    const bin = join(tmp, 'fake.sh');
    const death = 'ERROR rustjava_runtime::classes::java::lang::thread: Uncaught exception in thread -1258909751:\\njava.lang.NullPointerException\\n\\tat java/lang/Thread.run()V\\n';
    writeFileSync(bin, `#!/bin/sh\nprintf '${death}' >&2\ni=0; while [ $i -lt 300 ]; do echo 'WARN stub unk12-1' >&2; i=$((i+1)); done\necho '{"result":"PASS","paints":1}'\n`, { mode: 0o755 });
    const keep = opt.bin;
    opt.bin = bin;
    const r = await validate([], 30, join(tmp, 'L.stderr'));
    opt.bin = keep;
    const stderr = join(tmp, 'L.stderr');
    writeFileSync(join(tmp, 'old.stderr'), death.replaceAll('\\n', '\n').replaceAll('\\t', '\t') + '2026-10-03T05:30:31.016169Z WARN next\n');
    cases.push(
      ['a guest thread death is counted over the whole stream', r.uncaught_threads === 1 && !UNCAUGHT.test(readFileSync(stderr, 'latin1'))],
      ['a run whose game thread died is not a survivor', longplayVerdict(false, r, stderr) === 'error'],
      ['a run with no death survives', longplayVerdict(false, { result: 'UNMEASURED', uncaught_threads: 0 }, '/nonexistent') === 'ok'],
      ['a FAIL is still an error, no record is n/a', longplayVerdict(false, { result: 'FAIL', uncaught_threads: 0 }, '/nonexistent') === 'error' && longplayVerdict(false, null, '') === 'n/a'],
      ['an older run is read off its tail', longplayVerdict(false, { result: 'UNMEASURED' }, join(tmp, 'old.stderr')) === 'error'],
      ['the wall is the exception, not the next log line', uncaughtOf(readFileSync(join(tmp, 'old.stderr'), 'latin1')) === 'java.lang.NullPointerException\n\tat java/lang/Thread.run()V'],
    );
    rmSync(tmp, { recursive: true, force: true });
  }
  // The host lock, through the real `run` path (an empty corpus, so nothing is validated): a
  // version that dropped the hostLock() call exits at once and fails the first case.
  {
    const tmp = join('/tmp', `wie-census-selftest-${process.pid}`);
    rmSync(tmp, { recursive: true, force: true });
    mkdirSync(join(tmp, 'corpus'), { recursive: true });
    const lock = join(tmp, 'lock');
    const runOnce = () =>
      new Promise((res) => {
        const env = { ...process.env, WIE_CENSUS_LOCK: lock };
        const args = [fileURLToPath(import.meta.url), 'run', '--bin', '/nonexistent', '--out', join(tmp, 'out'), join(tmp, 'corpus')];
        const c = spawn(process.execPath, args, { env });
        let err = '';
        c.stderr.on('data', (d) => (err += d));
        const r = { err: () => err, done: false, code: null };
        c.on('exit', (code) => Object.assign(r, { done: true, code }));
        res(r);
      });
    const until = async (f, ms) => {
      for (const t = Date.now(); Date.now() - t < ms && !f(); ) await new Promise((r) => setTimeout(r, 50));
      return f();
    };
    // Held by a live pid (this one): the run waits, then proceeds once the holder lets go.
    mkdirSync(lock);
    writeFileSync(join(lock, 'pid'), String(process.pid));
    const w = await runOnce();
    const waited = (await until(() => w.err().includes('waiting'), 5000)) && !(await until(() => w.done, 1000));
    rmSync(lock, { recursive: true, force: true });
    const proceeded = (await until(() => w.done, 10000)) && w.code === 0;
    cases.push(['a second run waits while the lock is held', waited], ['and proceeds once it is released', proceeded]);
    // Held by a dead pid: reclaimed, run completes without waiting on it.
    const dead = spawn(process.execPath, ['-e', '']);
    await new Promise((r) => dead.on('exit', r));
    mkdirSync(lock);
    writeFileSync(join(lock, 'pid'), String(dead.pid));
    const d = await runOnce();
    cases.push(['a dead holder is reclaimed', (await until(() => d.done, 10000)) && d.code === 0 && d.err().includes('reclaiming')]);
    cases.push(['the run releases the lock on exit', !existsSync(lock)]);
    rmSync(tmp, { recursive: true, force: true });
  }
  const bad = cases.filter(([, ok]) => !ok);
  for (const [name] of bad) console.error(`selftest FAIL: ${name}`);
  console.log(`selftest: ${cases.length - bad.length}/${cases.length}`);
  process.exit(bad.length ? 1 : 0);
}

if (cmd === 'run') {
  if (!opt.bin || opt.dirs.length === 0) {
    console.error('run needs --bin <wie_validate> and at least one corpus dir');
    process.exit(2);
  }
  await hostLock(LOCK);
  const pop = population(opt.dirs);
  // A second `run` into the same --out (another corpus slice) adds to the population, never replaces it.
  const prev = read(join(out, 'population.json'));
  const known = new Set(pop.titles.map((t) => t.sha));
  const all = [...pop.titles, ...(prev?.titles ?? []).filter((t) => !known.has(t.sha))].sort((a, b) => a.sha.localeCompare(b.sha));
  writeFileSync(join(out, 'population.json'), JSON.stringify({ ...pop, titles: all, dirs: [...new Set([...(prev?.dirs ?? []), ...opt.dirs.map((d) => resolve(d))])] }));
  console.error(`population: ${pop.files} files -> ${pop.titles.length} unique · excluded dirs ${JSON.stringify(pop.excluded)} · jobs ${opt.jobs}`);
  if (!opt.only || opt.only === 'probe') await pool(pop.titles, opt.jobs, probe);
  if (starved) console.error(`★${starved} probes starved (deadline, < 100 ticks, 0 paints) — not recorded; run again when the host is quieter`);
  if (opt.only === 'progress') {
    // Targets: playable, or limited with a clean longplay — the titles a player can get into.
    const list = titleList();
    const cand = pop.titles.filter((t) => {
      if (list && !list.has(t.sha.slice(0, 12))) return false;
      const j = judge(t.sha);
      return j && j.ax.boot === 'ok' && j.ax.render === 'ok' && j.ax.longplay === 'ok';
    });
    // Longest budgets first, so a 60-minute run does not start last and set the wall time alone.
    const spec = (t) => list?.get(t.sha.slice(0, 12)) ?? {};
    cand.sort((a, b) => (spec(b).secs ?? opt.progress) - (spec(a).secs ?? opt.progress));
    console.error(`progress: ${cand.length} titles × ${opt.progress}s default (as ${opt.as ?? 'P'})`);
    await pool(cand, opt.jobs, (t) => progress(t, spec(t)));
  } else if (opt.only === 'speed') {
    const slow = pop.titles.filter((t) => {
      const j = judge(t.sha);
      return j && j.ax.render === 'ok' && j.ax.speed === 'n/a' && j.ratio !== null;
    });
    console.error(`speed: ${slow.length} titles below 0.9, re-measured at --jobs ${opt.jobs}`);
    await pool(slow, opt.jobs, speed);
  } else if (opt.only !== 'probe' && opt.only !== 'progress') {
    const list = titleList();
    const cand = pop.titles.filter((t) => {
      if (list && !list.has(t.sha.slice(0, 12))) return false;
      const j = judge(t.sha);
      return j && j.ax.input === 'ok' && j.ax.longplay === 'n/a';
    });
    console.error(`longplay: ${cand.length} candidates × ${opt.long}s`);
    await pool(cand, opt.jobs, (t) => longplay(t, list?.get(t.sha.slice(0, 12)) ?? {}));
  }
} else {
  const pop = read(join(out, 'population.json'));
  const prs = opt.prs ? read(resolve(opt.prs)) : [];
  const extra = opt.changes ? read(resolve(opt.changes)) : {};
  const entries = [];
  const rows = [['sha256', 'platform', 'model', 'title', 'status', 'boot', 'render', 'input', 'longplay', 'sound', 'speed', 'progress', 'ratio', 'browser', 'paints', 'distinct', 'novel', 'base_distinct', 'audio', 'still', 'p_stall', 'p_distinct', 'p2_stall', 'p_policy', 'p_escapes', 'p_save', 'reasonA', 'reasonL', 'reasonP', 'load1']];
  const clusters = new Map();
  for (const t of pop.titles) {
    const j = judge(t.sha);
    if (!j) continue;
    const title = displayTitle(t.path);
    const platform = j.A.platform && j.A.platform !== 'unknown' ? j.A.platform.toUpperCase() : sniffPlatform(t.path);
    const net = netWall(netConnects(j.A, join(out, t.sha, 'A.stderr'))) ? NET_KO : (NET_HAND[t.sha.slice(0, 12)] ?? null);
    // A locked file says only that: the other lines would read as «not fixed yet». Its status is never
    // better than not-yet — a check that paints its refusal box would otherwise read as playable.
    const lock = lockVerdict(lockOf(t.path), [j.A, j.B]);
    const st = lock ? 'not-yet' : net && status(j.ax) === 'playable' ? 'limited' : status(j.ax);
    const issues = Object.entries(j.ax)
      .filter(([k]) => !(k === 'render' && j.ax.boot === 'fail')) // one line for a title that never started
      .map(([k, v]) => ISSUE_KO[`${k}:${v}`])
      .filter(Boolean);
    if (lock) issues.splice(0, issues.length, LOCK_KO[lock]);
    else if (net) issues.splice(0, issues.length, net);
    const changes = prs
      .filter((pr) => names(pr.title, title) && !otherCarrier(pr.title, platform))
      .map((pr) => ({ date: pr.mergedAt.slice(0, 10), enginePin: pr.mergeCommit?.oid ?? null, summary_ko: summaryKo(pr.title), pr: pr.number }));
    changes.push(...(extra[t.sha] ?? []));
    changes.sort((a, b) => b.date.localeCompare(a.date));
    entries.push({
      sha256: t.sha,
      platform,
      model: j.A.lgt_compile_model ?? null,
      title,
      status: st,
      axes: j.ax,
      knownIssues_ko: issues,
      changes: changes.map(({ date, enginePin, summary_ko }) => ({ date, enginePin, summary_ko })),
    });
    rows.push([
      t.sha,
      platform,
      j.A.lgt_compile_model ?? '',
      title,
      st,
      ...Object.values(j.ax),
      j.ratio?.toFixed(3) ?? '',
      j.br.map((x) => x.toFixed(3)).join('/'),
      j.A.paints,
      j.A.distinct_colors,
      j.novel,
      j.baselineDistinct,
      j.A.audio ? `${j.A.audio.plays}/${j.A.audio.wave_events}w/${j.A.audio.midi_events}m` : '',
      j.L ? maxRun(j.L.shots) : '',
      j.P ? progressCurve(j.P).stall : '',
      j.P ? progressCurve(j.P).distinct : '',
      j.P2 ? progressCurve(j.P2).stall : '',
      j.P?.policy ?? (j.P ? 'v1' : ''),
      j.P?.stall_escapes ?? '',
      saveOf(j.P),
      j.A.reason,
      j.L?.reason ?? '',
      j.P?.reason ?? '',
      (j.S ?? j.A).load1?.toFixed(0),
    ]);
    // A title joins ONE playability cluster — its first failing axis — plus sound/speed ones.
    const first = ['boot', 'render', 'input', 'longplay'].find((a) => !['ok', 'n/a'].includes(j.ax[a]));
    for (const [axis, v] of Object.entries(j.ax)) {
      if (['ok', 'n/a'].includes(v) || (axis !== first && !['sound', 'speed', 'progress'].includes(axis))) continue;
      const died = axis === 'longplay' && j.L && j.L.result !== 'FAIL' && guestDied(j.L, join(out, t.sha, 'L.stderr'));
      const src = axis === 'progress' ? j.P : axis === 'longplay' && (j.L?.result === 'FAIL' || died) ? j.L : j.A;
      const stem = src === j.P ? 'P' : src === j.L ? 'L' : 'A';
      const reason = died ? `guest thread died: ${uncaughtOf(readFileSync(join(out, t.sha, 'L.stderr'), 'latin1')) ?? ''}` : src.reason;
      const key = `${axis}:${v}\t${axis === 'sound' || axis === 'speed' || axis === 'input' || (axis === 'progress' && v === 'stuck') ? '' : wallOf(reason, join(out, t.sha, `${stem}.stderr`))}`;
      if (!clusters.has(key)) clusters.set(key, []);
      clusters.get(key).push(`${t.sha.slice(0, 12)}(${platform.toLowerCase()})`);
    }
  }
  const count = (k) => entries.reduce((m, e) => ((m[e[k]] = (m[e[k]] ?? 0) + 1), m), {});
  writeFileSync(join(out, 'census.tsv'), rows.map((r) => r.map((c) => String(c ?? '').replace(/\s+/g, ' ')).join('\t')).join('\n') + '\n');
  const compat = { schema: 1, generatedAt: new Date().toISOString(), enginePin: opt.pin ?? null, entries };
  writeFileSync(opt.compat ? resolve(opt.compat) : join(out, 'compat.json'), JSON.stringify(compat, null, 1));
  const axes = ['boot', 'render', 'input', 'longplay', 'sound', 'speed', 'progress'];
  const md = [
    `# playability census — pin ${opt.pin ?? '?'} · ${entries.length} titles`,
    '',
    `population: ${pop.files} files -> ${pop.titles.length} unique · excluded dirs ${JSON.stringify(pop.excluded)}`,
    `status: ${JSON.stringify(count('status'))}`,
    '',
    '| axis | ' + ['ok', 'n/a', 'fail', 'uniform', 'none', 'error', 'stall', 'silent', 'slow', 'stuck'].join(' | ') + ' |',
    '|---|' + '---|'.repeat(10),
    ...axes.map((a) => {
      const c = entries.reduce((m, e) => ((m[e.axes[a]] = (m[e.axes[a]] ?? 0) + 1), m), {});
      return `| ${a} | ` + ['ok', 'n/a', 'fail', 'uniform', 'none', 'error', 'stall', 'silent', 'slow', 'stuck'].map((k) => c[k] ?? '').join(' | ') + ' |';
    }),
    '',
    '## clusters (first wall, by title count)',
    '',
    '| n | axis | first wall | titles (sha12) |',
    '|---|---|---|---|',
    ...[...clusters]
      .sort((a, b) => b[1].length - a[1].length)
      .map(([k, v]) => {
        const [axis, wall] = k.split('\t');
        return `| ${v.length} | ${axis} | ${wall.replace(/\|/g, '\\|')} | ${v.join(' ')} |`;
      }),
  ];
  writeFileSync(join(out, 'clusters.md'), md.join('\n') + '\n');
  console.log(`${entries.length} entries · ${JSON.stringify(count('status'))}`);
}
