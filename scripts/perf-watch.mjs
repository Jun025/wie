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
//            slower = B/A − 1.
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
export const SAMPLES = [
  { sha12: '835acd535a5f', platform: 'j2me', axes: ['native', 'browser'] },
  { sha12: '8238f5449347', platform: 'j2me', axes: ['native'] },
  { sha12: 'bf5b031018aa', platform: 'j2me', axes: ['browser'] },
  // Browser only: SKT ticks are paced, so CPU per tick did not move under a +54 % engine (0.7 · 0 · 0.5 — docs/report/0527 §7).
  { sha12: '976141a9525b', platform: 'skt', axes: ['browser'] },
  { sha12: 'd1234e0ddb75', platform: 'skt', axes: ['native', 'browser'] },
  { sha12: '1065985081fc', platform: 'ktf', axes: ['native', 'browser'] },
  { sha12: '155972cac664', platform: 'ktf', axes: ['native', 'browser'] },
  { sha12: '0606d43702e7', platform: 'lgt', axes: ['native', 'browser'] },
  { sha12: '3cc7a9b4cb15', platform: 'lgt', axes: ['native', 'browser'] },
];
const FLOOR = 5; // percent
const TICKS = 50e6;

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
    byRep.set(r.rep, { ...byRep.get(r.rep), [r.side]: r.metric });
  }
  const out = [];
  for (const [k, byRep] of groups) {
    const [axis, sample] = k.split(' ');
    const slow = [];
    for (const p of byRep.values()) {
      if (!(p.a > 0) || !(p.b > 0)) continue;
      slow.push(100 * (axis === 'native' ? p.b / p.a - 1 : p.a / p.b - 1));
    }
    out.push({ axis, sample, pairs: slow.length, slow: slow.map(r1), ...classify(slow) });
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

async function nativeOne(bin, game) {
  const { out, err } = await sh('/usr/bin/time', ['-p', bin, '--timeout', '60', game]);
  const line = out.trim().split('\n').reverse().find((l) => l.startsWith('{'));
  const user = Number(/^user\s+([\d.]+)/m.exec(err)?.[1]);
  const sys = Number(/^sys\s+([\d.]+)/m.exec(err)?.[1]);
  if (!line) return { metric: null, note: 'no result line' };
  const j = JSON.parse(line);
  const cpuMs = Math.round((user + sys) * 1000);
  const ok = (j.stop === 'max-ticks' || j.stop === 'deadline') && j.ticks >= 100 && cpuMs > 0;
  return { metric: ok ? Math.round((cpuMs * TICKS) / j.ticks) : null, stop: j.stop, ticks: j.ticks, paints: j.paints, cpuMs, ms: j.ms };
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
    const [ra, rb] = o.axis === 'native' ? await Promise.all([nativeOne(join(o.a, 'wie_validate'), s.path), nativeOne(join(o.b, 'wie_validate'), s.path)]) : await browserPair(o.a, o.b, s.path);
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
  L.push('- 축: native = 50M tick 당 CPU ms(높을수록 느림) · browser = 30 s paints(낮을수록 느림). 느려짐 % 는 «B 가 A 보다 얼마나 느린가».', '');
  L.push('| 축 | 표본 | 쌍 | 회별 느려짐 % | 중앙값 | 편차 | 문턱 | 판정 | 짝 재측 |', '|---|---|---|---|---|---|---|---|---|');
  for (const v of first) {
    const p = pair.find((x) => x.axis === v.axis && x.sample === v.sample);
    L.push(`| ${v.axis} | \`${v.sample}\` | ${v.pairs} | ${v.slow.join(' · ')} | ${v.median ?? '–'} | ${v.dev ?? '–'} | ${v.threshold ?? '–'} | ${v.verdict} | ${p ? `${p.verdict} ${p.median ?? ''}` : '–'} |`);
  }
  L.push('', '<details><summary>원자료(회 · 표본 · 쪽 · 값)</summary>', '', '| 축 | 회 | 표본 | 쪽 | 값 | stop/error | load1 |', '|---|---|---|---|---|---|---|');
  for (const r of rows) L.push(`| ${r.axis} | ${r.rep} | \`${r.sample}\` | ${r.side} | ${r.metric ?? '–'} | ${r.stop ?? r.error ?? r.note ?? ''} | ${r.load1} |`);
  L.push('', '</details>', '');
  writeFileSync(o.md, L.join('\n'));
  if (t.complete) writeFileSync(join(o.out, 'judge.json'), JSON.stringify({ first, pair, regressions: t.regressions }, null, 1) + '\n');
  return t;
}

// --- selftest ------------------------------------------------------------------------------------------
function selftest() {
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
if (cmd === 'selftest') process.exit(selftest());
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
