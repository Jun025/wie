#!/usr/bin/env node
// Weekly speed watch — the measuring and judging half. `scripts/perf-watch.sh` builds the two engines and
// leases each phase; this file runs one phase, judges, writes the evidence. Local only: the samples are
// game bytes (Constraint 9), so no CI can run it.
//
//   node scripts/perf-watch.mjs resolve --out <dir> <corpus dir>...       sample sha12 -> path, into <dir>/samples.json
//   node scripts/perf-watch.mjs run --out <dir> --axis native|browser --rep <n> --a <dir> --b <dir> [--only <sha12,…>]
//   node scripts/perf-watch.mjs judge --out <dir> [--reps <from-to>] [--axis <a>]   prints the alarmed sha12s, comma-joined
//   node scripts/perf-watch.mjs report --out <dir> --md <file> --a-pin <sha> --b-pin <sha> --tool-pin <sha> --date <d>
//   node scripts/perf-watch.mjs selftest
//
// A build dir holds `wie_validate` (native) and `wasm/` (wie_web.js + wie_web_bg.wasm). Side A is the
// reference, side B the build under test. Every sample runs A and B AT THE SAME TIME, so a pair shares
// one load minute; nothing else runs beside them (2 processes or 2 browsers — under the ≤ 3 cap).
//
// Two axes, both «how much slower is B than A», in percent:
//   native   CPU ms (user+sys) per 50M ticks, `wie_validate --timeout 60` without keys. CPU time, not the
//            wall clock: under load the wall clock of one build spread 2.5× over 5 runs while CPU time held
//            ±3% (docs/report/0500 §근거 ⑵). A run counts when it stopped at max-ticks or the deadline with
//            ≥ 100 ticks (SKT ticks are paced: ~950 in 30 s). 60 s, not 30: under load the idle-bound samples
//            stopped at the deadline short of 50M, and a deadline run's cost per tick moves with where the game
//            got to (stickfight, one binary against itself in the same minute: +7 % and +18 %) — that spread lands in the threshold.
//            slower = B/A − 1. Both sides run with WIE_AOT=off (the interpreter — nativeOne()).
//            ★The Java-run samples that draw (`per: 'paint'` in SAMPLES) are judged on another unit: CPU ms per 1000
//            paints over the whole 60 s (no tick backstop, so A and B run the same window). A tick is not a unit of
//            work there — this loop never sleeps, so ticks also pass while the game sleeps, and a change in what
//            Java costs moves how many: across #605 CPU per tick read −67 % on 835acd535a5f with the CPU seconds
//            equal (docs/report/0572 §10-2). Paints move with the host's load, so such a pair counts only when
//            both sides started together (each side stamps its own launch — nativeOne(), judge()). What that
//            unit cannot see, and what its `ok` does not mean: docs/report/0588 §4 (report() prints it every round).
//   browser  paints in 30 s, `audio-probe.mjs` (the shell's own wasm path in headless Chromium, its default
//            key cycle — docs/report/0495 §⒜). A run that errored or whose guest exited counts for nothing.
//            slower = A/B − 1. A title that does not fill the frame budget is capped by its own sleeps, so
//            this axis only moves on the budget-saturated samples (3cc7a9b4cb15 · 155972cac664).
// Judging, per axis × sample, over the paired reps: alarm when the median slowdown ≥ max(5 %, 2 × the
// standard deviation of the paired slowdowns), with ≥ 2 valid pairs (fewer = unmeasured, never clear).
// A median ≥ 5 % under a threshold its own spread raised is `noisy`, not ok: the spread hid it (classify()).
// The shell driver re-measures an alarmed sample once more (the pair, as the compat census does) and
// only a sample that alarms again is a regression; an alarm the pair could not measure is unmeasured (pairUp()).
// A round owes every SAMPLES row on each axis it ran. Fewer than half judged is no round: report exits 3,
// writes no judge.json, and the next round keeps this round's A — a missing corpus never moves the baseline (tally()).
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { existsSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync, appendFileSync, mkdirSync } from 'node:fs';
import { loadavg, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');
// Platforms 4 × 2 per axis, sha256 prefixes only. The six non-J2ME are docs/report/0495's samples; 3cc7a9b4cb15 and
// 155972cac664 are the two the shell named as filling the frame budget. J2ME comes from the redistributable OSS
// corpus (docs/oss-corpus/, jars outside the repo): stickfight (0495) on both axes; natively an idle title beside
// it (8238f5449347 paints once and idles to 50M ticks — the scheduler's own cost, as 0500's samples), in the
// browser one that keeps drawing (bf5b031018aa: natively 1.3k ticks in 30 s, too few for the native axis).
// `per: 'paint'` = the native unit of a Java-run sample that draws (header). 8238f5449347 is Java-run too and stays
// per tick: it paints once and asks for no sleep, so it has no other unit, and every one of its ticks is the same
// empty pass on both sides (docs/report/0588 §2).
export const SAMPLES = [
  { sha12: '835acd535a5f', platform: 'j2me', axes: ['native', 'browser'], per: 'paint' },
  { sha12: '8238f5449347', platform: 'j2me', axes: ['native'] },
  { sha12: 'bf5b031018aa', platform: 'j2me', axes: ['browser'] },
  // Browser only: SKT ticks are paced, so CPU per tick did not move under a +54 % engine (0.7 · 0 · 0.5 — docs/report/0527 §7).
  { sha12: '976141a9525b', platform: 'skt', axes: ['browser'] },
  { sha12: 'd1234e0ddb75', platform: 'skt', axes: ['native', 'browser'], per: 'paint' },
  { sha12: '1065985081fc', platform: 'ktf', axes: ['native', 'browser'] },
  { sha12: '155972cac664', platform: 'ktf', axes: ['native', 'browser'] },
  { sha12: '0606d43702e7', platform: 'lgt', axes: ['native', 'browser'] },
  { sha12: '3cc7a9b4cb15', platform: 'lgt', axes: ['native', 'browser'] },
];
const FLOOR = 5; // percent
const TICKS = 50e6;
const TOGETHER_MS = 1000; // a per-paint pair's two launches, each stamped by its own nativeOne()

const median = (a) => {
  const s = [...a].sort((x, y) => x - y);
  return s.length % 2 ? s[(s.length - 1) / 2] : (s[s.length / 2 - 1] + s[s.length / 2]) / 2;
};
const stdev = (a) => {
  const m = a.reduce((p, c) => p + c, 0) / a.length;
  return Math.sqrt(a.reduce((p, c) => p + (c - m) ** 2, 0) / a.length);
};
const r1 = (v) => Math.round(v * 10) / 10;

// slow: the per-rep slowdowns of one axis × sample, in percent.
export function classify(slow) {
  if (slow.length < 2) return { verdict: 'unmeasured' };
  const med = median(slow);
  const dev = stdev(slow);
  const threshold = Math.max(FLOOR, 2 * dev);
  return { median: r1(med), dev: r1(dev), threshold: r1(threshold), verdict: med >= threshold ? 'alarm' : med >= FLOOR ? 'noisy' : 'ok' };
}

// rows: { axis, rep, sample, side: 'a'|'b', metric|null } -> per axis × sample verdict.
export function judge(rows, reps = null) {
  const keep = rows.filter((r) => !reps || (r.rep >= reps[0] && r.rep <= reps[1]));
  const groups = new Map();
  for (const r of keep) {
    const k = `${r.axis} ${r.sample}`;
    if (!groups.has(k)) groups.set(k, new Map());
    const byRep = groups.get(k);
    byRep.set(r.rep, { ...byRep.get(r.rep), [r.side]: r });
  }
  const out = [];
  for (const [k, byRep] of groups) {
    const [axis, sample] = k.split(' ');
    const slow = [];
    let per;
    for (const { a, b } of byRep.values()) {
      per = b?.per ?? a?.per ?? per; // also when no pair counts: an unmeasured per-paint sample still names its unit
      if (!(a?.metric > 0) || !(b?.metric > 0)) continue;
      // Paints move with the host's load: a per-paint pair that was not one side-by-side launch says nothing.
      if ((a.per === 'paint' || b.per === 'paint') && !(a.per === b.per && Math.abs(a.t0 - b.t0) <= TOGETHER_MS)) continue;
      slow.push(100 * (axis === 'native' ? b.metric / a.metric - 1 : a.metric / b.metric - 1));
    }
    out.push({ axis, sample, per, pairs: slow.length, slow: slow.map(r1), ...classify(slow) });
  }
  return out.sort((x, y) => (x.axis + x.sample).localeCompare(y.axis + y.sample));
}

// first = judge(reps 1–3), pair = judge(reps 4–6).
export function pairUp(first, pair) {
  const again = (v) => pair.find((p) => p.axis === v.axis && p.sample === v.sample)?.verdict;
  const alarms = first.filter((v) => v.verdict === 'alarm');
  return { regressions: alarms.filter((v) => again(v) === 'alarm'), unconfirmed: alarms.filter((v) => !again(v) || again(v) === 'unmeasured') };
}

// What the round owes (SAMPLES × the axes it ran) against what it judged.
export function tally(first, pair, axes) {
  const expected = SAMPLES.flatMap((s) => s.axes.filter((a) => axes.includes(a)).map((a) => [a, s.sha12]));
  const { regressions, unconfirmed } = pairUp(first, pair);
  const judged = (a, s) => first.find((v) => v.axis === a && v.sample === s && v.verdict !== 'unmeasured');
  const unmeasured = expected.filter(([a, s]) => !judged(a, s)).length + unconfirmed.length;
  return {
    regressions,
    unmeasured,
    expected: expected.length,
    noisy: first.filter((v) => v.verdict === 'noisy').length,
    complete: 2 * (expected.length - unmeasured) >= expected.length && expected.length > 0,
  };
}

// --- measuring -------------------------------------------------------------------------------------------
function sh(cmd, args, opts = {}) {
  return new Promise((res) => {
    const p = spawn(cmd, args, { stdio: ['ignore', 'pipe', 'pipe'], ...opts });
    let out = '';
    let err = '';
    p.stdout.on('data', (d) => (out += d));
    p.stderr.on('data', (d) => (err = (err + d).slice(-20000)));
    p.on('close', (code) => res({ code, out, err }));
  });
}

// r: { stop, ticks, paints, cpuMs } of one run -> its native value (null = the run does not count).
export function nativeMetric(r, per) {
  if (!(r.cpuMs > 0)) return null;
  // Per paint: only a run that lasted the whole window beside its pair, and drew enough to divide by.
  if (per === 'paint') return r.stop === 'deadline' && r.paints >= 100 ? Math.round((r.cpuMs * 1000) / r.paints) : null;
  return (r.stop === 'max-ticks' || r.stop === 'deadline') && r.ticks >= 100 ? Math.round((r.cpuMs * TICKS) / r.ticks) : null;
}

async function nativeOne(bin, game, per, exec) {
  const unit = per && { per, t0: Date.now() }; // its own launch time, not a value handed in: judge() holds the pair to it
  // WIE_AOT=off: this axis is the interpreter's — the engine the product ships and the one every stored baseline was
  // measured on. wie_validate compiles ARM guests by default since docs/report/0613; an older side A ignores the variable.
  const { out, err } = await exec('/usr/bin/time', ['-p', bin, '--timeout', '60', ...(per === 'paint' ? ['--max-ticks', '100000000000'] : []), game], { env: { ...process.env, WIE_AOT: 'off' } });
  const line = out.trim().split('\n').reverse().find((l) => l.startsWith('{'));
  const user = Number(/^user\s+([\d.]+)/m.exec(err)?.[1]);
  const sys = Number(/^sys\s+([\d.]+)/m.exec(err)?.[1]);
  if (!line) return { metric: null, note: 'no result line', ...unit };
  const j = JSON.parse(line);
  const r = { stop: j.stop, ticks: j.ticks, paints: j.paints, cpuMs: Math.round((user + sys) * 1000), ms: j.ms };
  return { metric: nativeMetric(r, per), ...unit, ...r };
}

// One sample's native pair: its unit, and both sides launched in one call. The unit is read from SAMPLES here and
// nowhere else (not samples.json's copy: a resumed round may predate the field). `exec` is the process — selftest
// passes a stand-in, so the lookup and the launch it checks are the ones run() uses.
export function nativePair(a, b, sha12, game, exec = sh) {
  const per = SAMPLES.find((x) => x.sha12 === sha12)?.per;
  return Promise.all([a, b].map((dir) => nativeOne(join(dir, 'wie_validate'), game, per, exec)));
}

async function browserPair(a, b, game) {
  const { out, err, code } = await sh(process.execPath, [join(ROOT, 'scripts/audio-probe.mjs'), '--json', '--secs', '30', '--jobs', '2', '--wasm', join(a, 'wasm'), '--wasm', join(b, 'wasm'), game]);
  const runs = out.trim().split('\n').filter((l) => l.startsWith('{')).map((l) => JSON.parse(l));
  const pick = (dir) => {
    const r = runs.find((x) => x.wasm === resolve(dir, 'wasm'));
    if (!r) return { metric: null, note: `audio-probe rc=${code} ${err.trim().split('\n').at(-1) ?? ''}`.slice(0, 200) };
    return { metric: r.error ? null : r.paints, paints: r.paints, error: r.error ?? null, tickP99Ms: r.tickP99Ms };
  };
  return [pick(a), pick(b)];
}

function readRows(out) {
  const rows = [];
  for (const axis of ['native', 'browser']) {
    const f = join(out, `${axis}.jsonl`);
    if (existsSync(f)) for (const l of readFileSync(f, 'utf8').split('\n')) if (l) rows.push(JSON.parse(l));
  }
  return rows;
}

async function run(o) {
  const samples = JSON.parse(readFileSync(join(o.out, 'samples.json'), 'utf8'));
  const only = o.only ? new Set(o.only.split(',')) : null;
  const rep = Number(o.rep);
  const done = new Set(readRows(o.out).filter((r) => r.axis === o.axis && r.rep === rep).map((r) => r.sample));
  const file = join(o.out, `${o.axis}.jsonl`);
  for (const s of samples) {
    if (!s.axes.includes(o.axis) || (only && !only.has(s.sha12)) || done.has(s.sha12)) continue;
    if (!s.path) continue; // resolve recorded it as missing; judge reports it unmeasured
    const load1 = r1(loadavg()[0]);
    const [ra, rb] = o.axis === 'native' ? await nativePair(o.a, o.b, s.sha12, s.path) : await browserPair(o.a, o.b, s.path);
    for (const [side, r] of [['a', ra], ['b', rb]]) appendFileSync(file, JSON.stringify({ axis: o.axis, rep, sample: s.sha12, side, load1, ...r }) + '\n');
    console.error(`perf-watch: ${o.axis} rep ${rep} ${s.sha12} a=${ra.metric} b=${rb.metric} load1=${load1}`);
  }
}

function resolveSamples(out, dirs) {
  // A resumed round keeps its list unless a sample was missing (or moved): those are looked for again.
  const prev = join(out, 'samples.json');
  if (existsSync(prev) && JSON.parse(readFileSync(prev, 'utf8')).every((s) => s.path && existsSync(s.path) && SAMPLES.some((x) => x.sha12 === s.sha12))) return 0;
  const want = new Map(SAMPLES.map((s) => [s.sha12, s]));
  const found = new Map();
  const walk = (d) => {
    for (const n of readdirSync(d)) {
      const p = join(d, n);
      const st = statSync(p);
      if (st.isDirectory()) walk(p);
      else if (/\.(zip|jar|jad)$/i.test(n) && found.size < want.size) {
        const sha = createHash('sha256').update(readFileSync(p)).digest('hex').slice(0, 12);
        if (want.has(sha) && !found.has(sha)) found.set(sha, p);
      }
    }
  };
  for (const d of dirs) if (existsSync(d)) walk(resolve(d));
  mkdirSync(out, { recursive: true });
  const list = SAMPLES.map((s) => ({ ...s, path: found.get(s.sha12) ?? null }));
  writeFileSync(join(out, 'samples.json'), JSON.stringify(list, null, 1) + '\n');
  return list.filter((s) => !s.path).length;
}

// --- evidence ------------------------------------------------------------------------------------------
function report(o) {
  const rows = readRows(o.out);
  const first = judge(rows, [1, 3]);
  const pair = judge(rows, [4, 6]);
  const t = tally(first, pair, (o.axes ?? 'native,browser').split(','));
  const loads = rows.map((r) => r.load1).filter((v) => v >= 0);
  const pin = (p) => (/^[0-9a-f]{40}$/.test(p) ? p.slice(0, 8) : p); // a build dir name (…-slow64) is kept whole
  const L = [];
  L.push(`# wie 주간 속도 감시 — ${o.date}`, '');
  L.push(`- 기준(A) \`${pin(o['a-pin'])}\` → 대상(B) \`${pin(o['b-pin'])}\` · 도구 \`${o['tool-pin'].slice(0, 8)}\` · load1 ${loads.length ? `${Math.min(...loads)}~${Math.max(...loads)}` : '–'}`);
  if (!t.complete) L.push(`- ★**미완 — 판정하지 않았다.** 미측정 ${t.unmeasured}/${t.expected}(절반 넘게 못 잼). 다음 회차의 기준(A)은 이 회차로 옮기지 않는다.`);
  L.push(`- 판정: 느려짐 중앙값 ≥ max(${FLOOR}%, 2×편차) 이고 짝 재측(회 4~6)에서도 같으면 퇴행. **퇴행 ${t.regressions.length}** · 1차 경보 ${first.filter((v) => v.verdict === 'alarm').length} · 미측정 ${t.unmeasured}/${t.expected} · 편차에 묻힘(noisy) ${t.noisy}`);
  L.push('- 축: native = 50M tick 당 CPU ms(높을수록 느림) · browser = 30 s paints(낮을수록 느림). 느려짐 % 는 «B 가 A 보다 얼마나 느린가».');
  L.push('- ★native 의 판정 단위는 둘이다: Java 로 돌며 그리는 표본(표의 `native/paint`)은 tick 이 아니라 «1000 paints 당 CPU ms» — tick 은 게임이 자는 동안에도 늘어, Java 실행 비용이 바뀌면 같은 단위가 아니다(`perf-watch.mjs` 머리).');
  L.push('- ★Java 표본의 `ok` 가 뜻하지 않는 것(`docs/report/0588` §4): `d1234e0ddb75` 의 ok 는 «Java 가 안 느려졌다» 가 아니다 — 타이머 박자에 묶인 판이라 엔진이 그 박자를 놓칠 만큼 느려지기 전에는 값이 안 움직인다 · tick 으로 남긴 `8238f5449347` 은 빈 걸음 값이라 Java 실행 비용을 보지 못한다.');
  L.push('- ★`835acd535a5f` 는 같은 빌드 짝의 부호가 호스트 부하로 바뀐다(`docs/report/0588` §4 ⑴ · ⑸): 한가하면 게임이 자는 길이(박자)를, 바쁘면 같은 CPU 몫으로 그린 장수를 읽는다 — 1차와 짝 재측이 같은 부하 형상이면 Java 실행 비용이 늘지 않았어도 거짓 퇴행(박자를 읽은 경보)이 설 수 있다. 이 표본의 경보 · ok 는 원자료의 CPU ms(60초 가운데 받은 몫)와 load1 을 같이 읽어라.', '');
  L.push('| 축 | 표본 | 쌍 | 회별 느려짐 % | 중앙값 | 편차 | 문턱 | 판정 | 짝 재측 |', '|---|---|---|---|---|---|---|---|---|');
  for (const v of first) {
    const p = pair.find((x) => x.axis === v.axis && x.sample === v.sample);
    L.push(`| ${v.axis}${v.per ? `/${v.per}` : ''} | \`${v.sample}\` | ${v.pairs} | ${v.slow.join(' · ')} | ${v.median ?? '–'} | ${v.dev ?? '–'} | ${v.threshold ?? '–'} | ${v.verdict} | ${p ? `${p.verdict} ${p.median ?? ''}` : '–'} |`);
  }
  L.push('', '<details><summary>원자료(회 · 표본 · 쪽 · 값)</summary>', '', '| 축 | 회 | 표본 | 쪽 | 값 | CPU ms · ticks · paints | stop/error | load1 |', '|---|---|---|---|---|---|---|---|');
  for (const r of rows) L.push(`| ${r.axis}${r.per ? `/${r.per}` : ''} | ${r.rep} | \`${r.sample}\` | ${r.side} | ${r.metric ?? '–'} | ${[r.cpuMs, r.ticks, r.paints].map((x) => x ?? '–').join(' · ')} | ${r.stop ?? r.error ?? r.note ?? ''} | ${r.load1} |`);
  L.push('', '</details>', '');
  writeFileSync(o.md, L.join('\n'));
  if (t.complete) writeFileSync(join(o.out, 'judge.json'), JSON.stringify({ first, pair, regressions: t.regressions }, null, 1) + '\n');
  return t;
}

// --- selftest ------------------------------------------------------------------------------------------
async function selftest() {
  const rows = [];
  const add = (axis, sample, pairs, rep0 = 1) =>
    pairs.forEach(([a, b], i) => {
      rows.push({ axis, rep: rep0 + i, sample, side: 'a', metric: a });
      rows.push({ axis, rep: rep0 + i, sample, side: 'b', metric: b });
    });
  add('native', 'slow10', [[6200, 6840], [6300, 6900], [6150, 6790]]); // +10 % each time
  add('native', 'same', [[6200, 6230], [6300, 6260], [6150, 6170]]); // ±1 %
  add('native', 'noisy', [[6000, 6900], [6000, 5500], [6000, 6400]]); // median +6.7 but spread ±10
  add('native', 'faster', [[6600, 6200], [6700, 6250], [6650, 6300]]);
  add('native', 'onevalid', [[6200, null], [null, 6300], [6100, 6800]]);
  add('browser', 'fewer', [[390, 350], [395, 352], [388, 349]]); // paints −10 %: slower
  add('browser', 'more', [[390, 430], [395, 433], [388, 429]]);
  add('native', 'under', [[6000, 6294], [6000, 6294], [6000, 6294]]); // +4.9 % every time, no spread: the floor holds
  const v = Object.fromEntries(judge(rows).map((x) => [`${x.axis}/${x.sample}`, x]));
  const cases = [
    ['개악 +10% → 경보', v['native/slow10'].verdict === 'alarm'],
    ['원상 ±1% → 무경보', v['native/same'].verdict === 'ok'],
    ['중앙값 +6.7% 라도 편차 ±10% 면 경보 아님 — noisy(ok 와 다름 · 문턱 = 2×편차)', v['native/noisy'].verdict === 'noisy' && v['native/noisy'].threshold > 10],
    ['빨라짐 → 무경보', v['native/faster'].verdict === 'ok'],
    ['유효 쌍 1 → 미측정(무경보 아님)', v['native/onevalid'].verdict === 'unmeasured'],
    ['browser paints −10% → 경보', v['browser/fewer'].verdict === 'alarm'],
    ['browser paints +10% → 무경보', v['browser/more'].verdict === 'ok'],
    // The threshold itself, on exactly representable percentages (a ratio like 420/400 lands a hair above 5).
    ['문턱 경계: 중앙값 = 5 · 편차 0 → 경보(≥)', classify([5, 5, 5]).verdict === 'alarm'],
    ['문턱 경계: 중앙값 = 2×편차 = 6 → 경보(≥)', classify([3, 9]).verdict === 'alarm' && classify([3, 9]).threshold === 6],
    ['편차 0 · +4.9% → 무경보(문턱 하한 5%)', v['native/under'].verdict === 'ok'],
  ];
  // The pair, through the same pairUp() report() calls.
  add('native', 'flake', [[6200, 7000], [6200, 7000], [6200, 7000]]);
  add('native', 'flake', [[6200, 6200], [6200, 6250], [6200, 6180]], 4);
  add('native', 'slow10', [[6200, 6840], [6300, 6900], [6150, 6790]], 4);
  add('native', 'lostpair', [[6200, 7000], [6200, 7000], [6200, 7000]]);
  add('native', 'lostpair', [[6200, null], [6200, null], [6200, null]], 4);
  const { regressions, unconfirmed } = pairUp(judge(rows, [1, 3]), judge(rows, [4, 6]));
  const reg = regressions.map((x) => x.sample);
  cases.push(
    ['짝 재측이 재현 안 하면 퇴행 아님', !reg.includes('flake')],
    ['짝 재측도 경보면 퇴행', reg.includes('slow10')],
    ['짝 재측이 미측정이면 퇴행도 무경보도 아님 — unconfirmed', !reg.includes('lostpair') && unconfirmed.some((x) => x.sample === 'lostpair')],
  );
  // The two native units on the SAME runs, through nativePair() — the unit lookup and the launch are the ones run()
  // uses; only the process is a stand-in that answers after `ms`. A side is [cpuMs, ticks, paints] of a 60 s deadline
  // run. `sha12` picks the unit (a real SAMPLES row), `sample` only names the case. gap: b's stamp moved, as a
  // hand-joined row would have it.
  const jrows = [];
  const aot = new Set();
  const stand = (p, ms) => (_cmd, args, opts) =>
    new Promise((res) => {
      aot.add(opts?.env?.WIE_AOT);
      const [cpuMs, ticks, paints] = p[args[1].startsWith('a') ? 0 : 1];
      setTimeout(res, ms, { out: JSON.stringify({ stop: 'deadline', ticks, paints, ms: 60000 }) + '\n', err: `real 60.00\nuser ${cpuMs / 1000}\nsys 0.00\n` });
    });
  const raw = async (sample, sha12, pairs, rep0 = 1, gap = 0, ms = 0) => {
    for (const [i, p] of pairs.entries())
      (await nativePair('a', 'b', sha12, 'game', stand(p, ms))).forEach((r, k) => jrows.push({ axis: 'native', rep: rep0 + i, sample, side: 'ab'[k], ...r, ...(gap && { t0: r.t0 + k * gap }) }));
  };
  // B does the same work sooner and sleeps longer: ticks up 1.5–1.8×, CPU seconds and paints equal (0572 §10-2's shape).
  const inflated = [[[24030, 25.47e6, 1038], [23940, 44.93e6, 1040]], [[24880, 27.04e6, 1071], [24650, 47.9e6, 1066]], [[27160, 38.29e6, 1053], [27130, 58.24e6, 1050]]];
  // B burns +20 % CPU for the same paints, and its ticks doubled: per tick it reads as −40 %.
  const hidden = [[[20000, 20e6, 1000], [24000, 40e6, 1000]], [[21000, 21e6, 1050], [25300, 43e6, 1052]], [[19500, 19e6, 980], [23300, 39e6, 978]]];
  await raw('inflated', '835acd535a5f', inflated);
  await raw('inflated-tick', '1065985081fc', inflated); // the same runs under a KTF row: per tick
  for (const rep0 of [1, 4]) await raw('hidden', 'd1234e0ddb75', hidden, rep0);
  await raw('hidden-tick', '8238f5449347', hidden); // the Java row that stays per tick
  await raw('apart', '835acd535a5f', hidden, 1, 5000); // the same numbers, b stamped 5 s after a
  await raw('starved', 'd1234e0ddb75', [[[20000, 20e6, 40], [20000, 20e6, 40]], [[20000, 20e6, 40], [20000, 20e6, 40]]]); // too few paints: no pair counts
  // Two reps whose processes outlast TOGETHER_MS: launched side by side their stamps still agree, one after the other they could not.
  await Promise.all([1, 2].map((rep) => raw('together', '835acd535a5f', [hidden[0]], rep, 0, TOGETHER_MS + 100)));
  const jv = Object.fromEntries(judge(jrows, [1, 3]).map((x) => [x.sample, x]));
  cases.push(
    ['Java 표본(per paint): tick 만 부풀고 CPU 초 · paints 같음 → «개선» 없음(|중앙값| < 5) · 무경보', jv.inflated.verdict === 'ok' && Math.abs(jv.inflated.median) < FLOOR],
    ['… 같은 입력을 tick 으로 나누면 −30% 아래(옛 분모의 거짓 개선)', jv['inflated-tick'].median < -30],
    ['Java 표본(per paint): CPU 초 +20% · tick 2배 · paints 같음 → 경보', jv.hidden.verdict === 'alarm' && jv.hidden.median >= 19],
    ['… 짝 재측도 경보 → 퇴행', pairUp(judge(jrows, [1, 3]), judge(jrows, [4, 6])).regressions.some((x) => x.sample === 'hidden')],
    ['… 같은 입력을 tick 으로 나누면 무경보(옛 분모는 가린다)', jv['hidden-tick'].verdict === 'ok' && jv['hidden-tick'].median < 0],
    ['per paint 짝이 같은 순간에 뜬 것이 아니면 미측정(경보도 무경보도 아님)', jv.apart.verdict === 'unmeasured' && jv.apart.pairs === 0],
    ['per paint 는 마감까지 돈 실행 · paints ≥ 100 만 센다', nativeMetric({ stop: 'max-ticks', cpuMs: 9000, ticks: 5e7, paints: 900 }, 'paint') === null && nativeMetric({ stop: 'deadline', cpuMs: 9000, ticks: 5e7, paints: 1 }, 'paint') === null],
    ['per paint 표본은 정확히 835acd535a5f · d1234e0ddb75 둘(8238f5449347 과 KTF · LGT 는 tick 그대로)', SAMPLES.filter((s) => s.per).map((s) => `${s.sha12}:${s.per}:${s.axes.includes('native')}`).sort().join() === '835acd535a5f:paint:true,d1234e0ddb75:paint:true'],
    ['전 짝이 무효인 per paint 표본도 미측정 행에 단위(per)를 싣는다', jv.starved.verdict === 'unmeasured' && jv.starved.pairs === 0 && jv.starved.per === 'paint'],
    ['네이티브 축의 실행은 전부 WIE_AOT=off 로 뜬다(해석기 — 러너의 기본값은 AOT 다)', [...aot].join() === 'off'],
    ['두 쪽은 각자 띄운 시각을 찍는다 — 나란히 띄운 짝은 프로세스가 1초 넘게 돌아도 센다(순차로 띄우면 미측정)', jv.together.pairs === 2],
  );
  const lost = rows.filter((r) => r.sample === 'lostpair').map((r) => ({ ...r, sample: '0606d43702e7' }));
  const t1 = tally(judge(lost, [1, 3]), judge(lost, [4, 6]), ['native']);
  cases.push(['그 unconfirmed 는 원장의 unmeasured 로 센다', t1.unmeasured === t1.expected && t1.regressions.length === 0]);
  // The round, through report(): a missing corpus or a broken validator is INCOMPLETE and leaves no judge.json.
  const round = (name, metric) => {
    const out = join(tmpdir(), `perf-watch-selftest-${process.pid}-${name}`);
    rmSync(out, { recursive: true, force: true });
    mkdirSync(out, { recursive: true });
    const lines = [];
    if (metric !== undefined)
      for (const s of SAMPLES.filter((x) => x.axes.includes('native')))
        for (const rep of [1, 2, 3]) for (const side of ['a', 'b']) lines.push(JSON.stringify({ axis: 'native', rep, sample: s.sha12, side, load1: 1, metric: metric && (side === 'a' ? 6000 : 6060) }));
    writeFileSync(join(out, 'native.jsonl'), lines.join('\n') + (lines.length ? '\n' : ''));
    const t = report({ out, md: join(out, 'perf.md'), 'a-pin': 'a'.repeat(40), 'b-pin': 'b'.repeat(40), 'tool-pin': 'c'.repeat(40), date: 'selftest', axes: 'native' });
    const r = { ...t, judged: existsSync(join(out, 'judge.json')), md: readFileSync(join(out, 'perf.md'), 'utf8') };
    rmSync(out, { recursive: true, force: true });
    return r;
  };
  const empty = round('empty');
  const allnull = round('allnull', null);
  const full = round('full', 1);
  const expected = SAMPLES.filter((x) => x.axes.includes('native')).length;
  cases.push(
    [`코퍼스 비움(행 0) → 미완 · unmeasured ${expected}/${expected} · judge.json 없음`, !empty.complete && empty.unmeasured === expected && empty.expected === expected && !empty.judged],
    ['행 0 의 증적 md 에 Infinity 없음', !empty.md.includes('Infinity')],
    ['측정 행 전부 null → 미완 · judge.json 없음', !allnull.complete && allnull.unmeasured === expected && !allnull.judged],
    ['전 표본 +1% → 완료 · unmeasured 0 · judge.json 있음', full.complete && full.unmeasured === 0 && full.regressions.length === 0 && full.judged],
    ['증적 md 머리(표 앞)가 Java 세 표본의 ok 가 뜻하지 않는 것 · 부하로 부호가 바뀜 · 0588 §4 를 적는다', ['835acd535a5f', 'd1234e0ddb75', '8238f5449347', '부하로 바뀐다', '거짓 퇴행', '`docs/report/0588` §4'].every((w) => full.md.split('| 축 |')[0].includes(w))],
  );
  let fail = 0;
  for (const [name, ok] of cases) {
    if (!ok) fail++;
    console.log(`${ok ? 'ok  ' : 'FAIL'} ${name}`);
  }
  console.log(`perf-watch selftest ${cases.length - fail}/${cases.length}`);
  return fail ? 1 : 0;
}

// --- cli -----------------------------------------------------------------------------------------------
const [cmd, ...rest] = process.argv.slice(2);
const o = { _: [] };
for (let i = 0; i < rest.length; i++) {
  if (rest[i].startsWith('--')) o[rest[i].slice(2)] = rest[++i];
  else o._.push(rest[i]);
}
if (cmd === 'selftest') process.exit(await selftest());
else if (cmd === 'resolve') {
  const missing = resolveSamples(o.out, o._);
  if (missing) console.error(`perf-watch: ${missing} sample(s) not found in the corpus`);
} else if (cmd === 'run') await run(o);
else if (cmd === 'judge') {
  // Prints the alarmed samples of one axis, comma-joined: the --only list of the pair re-measure.
  const v = judge(readRows(o.out), o.reps ? o.reps.split('-').map(Number) : null).filter((x) => !o.axis || x.axis === o.axis);
  console.log(v.filter((x) => x.verdict === 'alarm').map((x) => x.sample).join(','));
} else if (cmd === 'report') {
  // stdout: <regressions> <unmeasured> <expected> <noisy>; exit 3 = incomplete (no judge.json written).
  const t = report(o);
  console.log(`${t.regressions.length} ${t.unmeasured} ${t.expected} ${t.noisy}`);
  if (!t.complete) process.exit(3);
}
else {
  console.error('usage: perf-watch.mjs resolve|run|judge|report|selftest — see the header');
  process.exit(2);
}
