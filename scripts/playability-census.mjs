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
//   node scripts/playability-census.mjs run --bin <wie_validate> --out <dir> [--jobs 8]
//        [--secs 30] [--long 600] [--only probe|long] <corpus dir>...
//   node scripts/playability-census.mjs report --out <dir> --pin <wie sha>
//        [--compat <compat.json>] [--changes <changes.json>] [--prs <gh-merged.json>]
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
//           with a shot every 20 s: error = FAIL line; stall = 4 identical shots in a row
//           (>= 60 s unchanged while keys keep arriving). A title whose 30 s probe already
//           failed after painting is `error` without the long run.
//   sound   ok = a Play with events reached the sink; silent = none did. This is the
//           engine side only: a command the browser host drops is #348's axis, not this one.
//   speed   1 - (sleep lateness + timer lateness + GC) / window, the #347 ratio, headless.
//           slow < 0.9. ★Wall-clock: host load inflates lateness — record load1 beside it.
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { cpus, loadavg } from 'node:os';
import { basename, join, resolve } from 'node:path';

const EXCLUDED = ['_dup', '_nongame', 'vendor_sdk'];
const PROBE_KEYS_AT = 8; // pacing window opens after boot
// Never CLR / soft keys: those quit many titles, and a clean exit ends the run early.
const LONG_KEYS = 'OK:1 UP:0.5 UP:0.5 OK:1 DOWN:0.5 RIGHT:0.5 NUM5:1 LEFT:0.5 NUM5:1 OK:1 NUM2:0.5 NUM8:0.5 NUM4:0.5 NUM6:0.5 OK:1';

const [cmd, ...rest] = process.argv.slice(2);
const opt = { jobs: Math.max(1, Math.floor(cpus().length * 0.8)), secs: 30, long: 600, dirs: [] };
for (let i = 0; i < rest.length; i++) {
  const a = rest[i];
  if (a.startsWith('--')) opt[a.slice(2)] = rest[++i];
  else opt.dirs.push(a);
}
opt.jobs = Number(opt.jobs);
opt.secs = Number(opt.secs);
opt.long = Number(opt.long);
if (!opt.out || !['run', 'report'].includes(cmd)) {
  console.error('usage: playability-census.mjs run|report --out <dir> …  (see the header)');
  process.exit(2);
}
const out = resolve(opt.out);
mkdirSync(out, { recursive: true });
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
    let partial = '';
    child.stdout.on('data', (b) => (stdout += b));
    child.stderr.on('data', (b) => {
      const lines = (partial + b).split('\n');
      partial = lines.pop();
      for (const l of lines) {
        if (/\bWARN\b|panicked/.test(l)) warns++;
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
    const args = ['--inject', '--keep-timeout', '--timeout', String(opt.secs), '--shotdir', join(d, name), ...extra, t.path];
    const r = await validate(args, opt.secs + 120, join(d, `${name}.stderr`));
    r.shots = shotHashes(join(d, name));
    writeFileSync(f, JSON.stringify(r));
  }
}

async function longplay(t) {
  const d = join(out, t.sha);
  const f = join(d, 'L.json');
  if (existsSync(f)) return;
  mkdirSync(join(d, 'L'), { recursive: true });
  const keys = join(d, 'long.keys');
  const reps = Math.ceil(opt.long / 10);
  writeFileSync(keys, Array(reps).fill(LONG_KEYS).join('\n'));
  // --max-ticks: the 50M default is an infinite-loop backstop sized for a boot, and a fast title
  // burns it in minutes — measured on this run's first pass, which ended runs at 3 of 10 minutes.
  const args = ['--inject', '--keys', keys, '--keep-timeout', '--timeout', String(opt.long), '--max-ticks', '100000000000', '--shotdir', join(d, 'L'), '--shot-every', '20', t.path];
  const r = await validate(args, opt.long + 300, join(d, 'L.stderr'));
  // `--keys` also shoots once per key step; only the `tNNN.N` timer shots are evenly spaced.
  const timed = existsSync(join(d, 'L'))
    ? readdirSync(join(d, 'L'))
        .filter((n) => /__t\d+\.\d\.png$/.test(n))
        .sort()
    : [];
  r.shots = timed.map((n) => sha256(readFileSync(join(d, 'L', n))));
  writeFileSync(f, JSON.stringify(r));
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

function judge(sha) {
  const d = join(out, sha);
  const A = read(join(d, 'A.json'));
  const B = read(join(d, 'B.json'));
  const L = read(join(d, 'L.json'));
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
  if (probeErr) ax.longplay = 'error';
  else if (!L) ax.longplay = 'n/a';
  else if (L.result === 'FAIL') ax.longplay = 'error';
  else ax.longplay = maxRun(L.shots) >= 4 ? 'stall' : 'ok';
  const au = A.audio ?? B.audio;
  ax.sound = ax.boot !== 'ok' || !au ? 'n/a' : au.plays - (au.empty_plays ?? 0) > 0 ? 'ok' : 'silent';
  const p = A.pacing;
  const windowMs = (opt.secs - PROBE_KEYS_AT) * 1000;
  let ratio = null;
  if (p && p.sleeps + p.timers > 0) ratio = 1 - (p.sleep_late_sum + p.timer_late_sum + p.gc_ms) / windowMs;
  ax.speed = ratio === null || !ok2 ? 'n/a' : ratio >= 0.9 ? 'ok' : 'slow';
  return { A, B, L, ax, ratio, novel, baselineDistinct: baseline.size };
}

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
  'sound:silent': '소리가 나지 않아요.',
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
// A PR names a title when the name stands alone ("놈3" is not "놈") …
const names = (text, title) => title.length >= 2 && new RegExp(`(^|[^\\p{L}\\p{N}])${title.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}($|[^\\p{L}\\p{N}])`, 'u').test(text);
// … and does not name a different carrier (one name, two carriers' builds).
const otherCarrier = (text, platform) => ['KTF', 'SKT', 'LGT'].some((c) => c !== platform && new RegExp(`\\b${c}\\b`, 'i').test(text)) && !new RegExp(`\\b${platform}\\b`, 'i').test(text);

// NFC: macOS hands back file names decomposed (NFD), and PR titles are composed.
const displayTitle = (p) =>
  basename(p)
    .normalize('NFC')
    .replace(/\.(zip|jar)$/i, '')
    .replace(/^\s*\((KTF|SKT|LGT|J2ME)\)\s*/i, '')
    .replace(/\s*[[(（][^\])）]*[\])）]\s*$/, '')
    .trim();

if (cmd === 'run') {
  if (!opt.bin || opt.dirs.length === 0) {
    console.error('run needs --bin <wie_validate> and at least one corpus dir');
    process.exit(2);
  }
  const pop = population(opt.dirs);
  writeFileSync(join(out, 'population.json'), JSON.stringify({ ...pop, dirs: opt.dirs.map((d) => resolve(d)) }));
  console.error(`population: ${pop.files} files -> ${pop.titles.length} unique · excluded dirs ${JSON.stringify(pop.excluded)} · jobs ${opt.jobs}`);
  if (opt.only !== 'long') await pool(pop.titles, opt.jobs, probe);
  if (opt.only !== 'probe') {
    const cand = pop.titles.filter((t) => {
      const j = judge(t.sha);
      return j && j.ax.input === 'ok' && j.ax.longplay === 'n/a';
    });
    console.error(`longplay: ${cand.length} candidates × ${opt.long}s`);
    await pool(cand, opt.jobs, longplay);
  }
} else {
  const pop = read(join(out, 'population.json'));
  const prs = opt.prs ? read(resolve(opt.prs)) : [];
  const extra = opt.changes ? read(resolve(opt.changes)) : {};
  const entries = [];
  const rows = [['sha256', 'platform', 'model', 'title', 'status', 'boot', 'render', 'input', 'longplay', 'sound', 'speed', 'ratio', 'paints', 'distinct', 'novel', 'base_distinct', 'audio', 'reasonA', 'reasonL', 'load1']];
  const clusters = new Map();
  for (const t of pop.titles) {
    const j = judge(t.sha);
    if (!j) continue;
    const title = displayTitle(t.path);
    const platform = (j.A.platform && j.A.platform !== 'unknown' ? j.A.platform : t.path.split('/').at(-2)).toUpperCase();
    const st = status(j.ax);
    const issues = Object.entries(j.ax)
      .map(([k, v]) => ISSUE_KO[`${k}:${v}`])
      .filter(Boolean);
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
      j.A.paints,
      j.A.distinct_colors,
      j.novel,
      j.baselineDistinct,
      j.A.audio ? `${j.A.audio.plays}/${j.A.audio.wave_events}w/${j.A.audio.midi_events}m` : '',
      j.A.reason,
      j.L?.reason ?? '',
      j.A.load1?.toFixed(0),
    ]);
    // A title joins ONE playability cluster — its first failing axis — plus sound/speed ones.
    const first = ['boot', 'render', 'input', 'longplay'].find((a) => !['ok', 'n/a'].includes(j.ax[a]));
    for (const [axis, v] of Object.entries(j.ax)) {
      if (['ok', 'n/a'].includes(v) || (axis !== first && !['sound', 'speed'].includes(axis))) continue;
      const src = axis === 'longplay' && j.L?.result === 'FAIL' ? j.L : j.A;
      const key = `${axis}:${v}\t${axis === 'sound' || axis === 'speed' || axis === 'input' ? '' : wallOf(src.reason, join(out, t.sha, `${src === j.L ? 'L' : 'A'}.stderr`))}`;
      if (!clusters.has(key)) clusters.set(key, []);
      clusters.get(key).push(`${t.sha.slice(0, 12)}(${platform.toLowerCase()})`);
    }
  }
  const count = (k) => entries.reduce((m, e) => ((m[e[k]] = (m[e[k]] ?? 0) + 1), m), {});
  writeFileSync(join(out, 'census.tsv'), rows.map((r) => r.map((c) => String(c ?? '').replace(/\s+/g, ' ')).join('\t')).join('\n') + '\n');
  const compat = { schema: 1, generatedAt: new Date().toISOString(), enginePin: opt.pin ?? null, entries };
  writeFileSync(opt.compat ? resolve(opt.compat) : join(out, 'compat.json'), JSON.stringify(compat, null, 1));
  const axes = ['boot', 'render', 'input', 'longplay', 'sound', 'speed'];
  const md = [
    `# playability census — pin ${opt.pin ?? '?'} · ${entries.length} titles`,
    '',
    `population: ${pop.files} files -> ${pop.titles.length} unique · excluded dirs ${JSON.stringify(pop.excluded)}`,
    `status: ${JSON.stringify(count('status'))}`,
    '',
    '| axis | ' + ['ok', 'n/a', 'fail', 'uniform', 'none', 'error', 'stall', 'silent', 'slow'].join(' | ') + ' |',
    '|---|' + '---|'.repeat(9),
    ...axes.map((a) => {
      const c = entries.reduce((m, e) => ((m[e.axes[a]] = (m[e.axes[a]] ?? 0) + 1), m), {});
      return `| ${a} | ` + ['ok', 'n/a', 'fail', 'uniform', 'none', 'error', 'stall', 'silent', 'slow'].map((k) => c[k] ?? '').join(' | ') + ' |';
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
