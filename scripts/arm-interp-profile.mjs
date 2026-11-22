#!/usr/bin/env node
// Where the ARM interpreter's time goes, from an xctrace Time Profiler export of a native `wie_validate`
// built with line tables. `scripts/arm-interp-profile.sh` records and exports; this file only aggregates.
//
//   node scripts/arm-interp-profile.mjs <time-profile.xml>
//   node scripts/arm-interp-profile.mjs selftest     the parse and the buckets, on a made-up export (no game, no xctrace)
//
// Every sample under `ArmEngine::run` is given to its innermost frame that is not the standard library,
// and that frame's source line is put in a bucket. The buckets are found by READING the sources the
// frames point at (the decode table, the `trace!` statements, our memory shim), not by line numbers
// written here — the files this measures are the ones the next rounds change. A source that cannot be
// read (build tree deleted) falls back to one bucket per file and the output says so.
// Shares are of the samples under the run loop; the header line gives that loop's share of all samples.
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';

const OURS = 'wie-core-arm/src/engine/arm32_cpu.rs';

// ── xctrace XML: elements carry `id` once and are referenced by `ref` afterwards ─────────────────
function parse(xml) {
  const root = { tag: '', attrs: {}, kids: [], text: '' };
  const stack = [root];
  const ids = new Map();
  const re = /<(\/?)([\w-]+)((?:\s+[\w:-]+=(?:"[^"]*"|'[^']*'))*)\s*(\/?)>|([^<]+)/g;
  for (let m; (m = re.exec(xml)); ) {
    const top = stack.at(-1);
    if (m[5] !== undefined) top.text += m[5];
    else if (m[1]) stack.pop();
    else {
      const attrs = {};
      for (const a of m[3].matchAll(/([\w:-]+)=(?:"([^"]*)"|'([^']*)')/g)) attrs[a[1]] = a[2] ?? a[3];
      const el = { tag: m[2], attrs, kids: [], text: '' };
      top.kids.push(el);
      if (attrs.id !== undefined) ids.set(attrs.id, el);
      if (!m[4]) stack.push(el);
    }
  }
  const deref = (el) => (el && el.attrs.ref !== undefined ? ids.get(el.attrs.ref) : el);
  const kid = (el, tag) => deref(el.kids.find((k) => k.tag === tag));
  const rows = [];
  const walk = (el) => {
    if (el.tag === 'row') rows.push(el);
    else el.kids.forEach(walk);
  };
  walk(root);
  return { rows, deref, kid };
}

// ── buckets, read off the sources ────────────────────────────────────────────────────────────────
const sources = new Map(); // path -> (line -> bucket) | null when unreadable
const unreadable = new Set();
function classifier(path) {
  if (sources.has(path)) return sources.get(path);
  let fn = null;
  if (!existsSync(path)) unreadable.add(path.split('/').slice(-3).join('/'));
  else {
    const lines = readFileSync(path, 'utf8').split('\n');
    const at = (re, from = 0) => {
      const i = lines.findIndex((l, n) => n >= from && re.test(l));
      return i < 0 ? Infinity : i + 1; // 1-based
    };
    // Statement spans of `trace!(…);` / `log_enabled!(…)`: the level check lives on those lines.
    const logLines = new Set();
    for (let i = 0; i < lines.length; i++) {
      if (!/\b(trace|log_enabled)!\(/.test(lines[i])) continue;
      for (let j = i; j < lines.length; j++) {
        logLines.add(j + 1);
        if (/\)\s*;|\)\s*\{/.test(lines[j])) break;
      }
    }
    if (path.endsWith(OURS)) {
      const shim = at(/^struct Arm32CpuMemory/);
      fn = (n) => (n >= shim ? '게스트 메모리 shim (read_page · get_page · r/w)' : '실행 루프 (ArmEngine::run)');
    } else if (/arm32_cpu[^/]*\/src\/(thumb|arm)\.rs$/.test(path)) {
      const mode = path.endsWith('thumb.rs') ? 'Thumb' : 'ARM';
      const table = at(/^const INST_MATCH_ORDER/);
      const impl = at(/^impl Cpu \{/);
      const body = at(/^\s*match inst_type \{/, impl);
      fn = (n) =>
        logLines.has(n) ? 'log 수준 검사 (trace!)' : n >= table && n < impl ? `${mode} 해독 (패턴 표 선형 대조)` : n >= impl && n < body ? `${mode} 선행부 (인출 · CPSR · PC 전진)` : `${mode} 실행 (명령별)`;
    }
  }
  sources.set(path, fn);
  return fn;
}
function bucket(path, line) {
  const byLine = classifier(path);
  if (byLine) return byLine(line);
  const file = path.split('/').pop();
  if (path.includes('/log-0.')) return 'log 수준 검사 (trace!)';
  if (path.endsWith(OURS)) return 'wie-core-arm arm32_cpu.rs (소스 못 읽음 — 루프와 shim 미분리)';
  if (/arm32_cpu[^/]*\/src\//.test(path))
    return { 'reg.rs': '레지스터 파일 (뱅크 표 REG_MAP 경유)', 'lib.rs': 'Cpu::step 본체', 'alignment.rs': '정렬 래퍼', 'util.rs': '플래그 · 시프트 · 조건 (util.rs)' }[file] ?? `arm32_cpu ${file}`;
  return `그 밖 ${file}`;
}

// ── aggregate ────────────────────────────────────────────────────────────────────────────────────
function aggregate(xml) {
  const { rows, deref, kid } = parse(xml);
  const count = (m, k) => m.set(k, (m.get(k) ?? 0) + 1);
  const r = { total: 0, loop: 0, by: new Map(), lines: new Map(), outside: new Map() };
  for (const row of rows) {
    const bt = kid(row, 'tagged-backtrace') ?? kid(row, 'backtrace');
    const frames = (bt?.kids.some((k) => k.tag === 'frame') ? bt : bt && kid(bt, 'backtrace'))?.kids.filter((k) => k.tag === 'frame').map(deref) ?? [];
    if (!frames.length) continue;
    r.total++;
    const where = frames.map((f) => {
      const src = f.kids.find((k) => k.tag === 'source');
      const s = deref(src);
      const p = s && kid(s, 'path');
      return p ? [p.text, Number((src.attrs.line ?? s.attrs.line) || 0)] : null;
    });
    // Under the run loop = some frame is in our engine file above the memory shim (the shim is also reached from host calls).
    if (!where.some((w) => w && w[0].endsWith(OURS) && !bucket(w[0], w[1]).startsWith('게스트 메모리'))) {
      count(r.outside, (frames[0].attrs.name ?? '?').replaceAll('&lt;', '<').replaceAll('&gt;', '>').replaceAll('&amp;', '&').slice(0, 80));
      continue;
    }
    r.loop++;
    const inner = where.find((w) => w && !w[0].includes('/rustc/') && !w[0].includes('/rustlib/'));
    if (!inner) {
      count(r.by, '표준 라이브러리만');
      continue;
    }
    count(r.by, bucket(inner[0], inner[1]));
    count(r.lines, `${inner[0].split('/src/')[0].split('/').pop()}/${inner[0].split('/').pop()}:${inner[1]}`);
  }
  return r;
}

function selftest() {
  const d = mkdtempSync(join(tmpdir(), 'arm-interp-profile-'));
  const ours = join(d, OURS);
  const thumb = join(d, 'arm32_cpu-9.9.9/src/thumb.rs');
  const write = (f, lines) => {
    mkdirSync(dirname(f), { recursive: true });
    writeFileSync(f, lines.join('\n'));
  };
  write(ours, ['fn run() {', '    step();', '}', 'struct Arm32CpuMemory;', 'fn read_page() {}']); // run = 1-3, shim = 4+
  write(thumb, ['const INST_MATCH_ORDER: [u8; 1] = [0];', 'fn decode() {}', 'impl Cpu {', '    trace!("x");', '    let pc = 0;', '    match inst_type {', '        A => {}', '    }', '}']);
  const frame = (id, path, line) => `<frame id="${id}" name="f${id}"><source line="${line}"><path id="p${id}">${path}</path></source></frame>`;
  const run = frame(1, ours, 2);
  const row = (bt) => `<row><weight ref="w"/>${bt}</row>`;
  const xml = `<trace-query-result><node>${[
    row(`<tagged-backtrace id="a">${frame(2, thumb, 2)}${run}</tagged-backtrace>`), // decode
    row('<tagged-backtrace ref="a"/>'), //                                              decode again, by ref
    row(`<tagged-backtrace id="b">${frame(3, thumb, 4)}<frame ref="1"/></tagged-backtrace>`), // trace!
    row(`<tagged-backtrace id="c">${frame(4, thumb, 7)}<frame ref="1"/></tagged-backtrace>`), // an instruction's arm
    row(`<tagged-backtrace id="e">${frame(5, ours, 5)}<frame ref="1"/></tagged-backtrace>`), // our shim, under the loop
    row(`<tagged-backtrace id="g">${frame(6, ours, 5)}</tagged-backtrace>`), //                 our shim from a host call: outside
    row(`<tagged-backtrace id="h">${frame(7, '/x/rustc/core/num.rs', 1)}<frame ref="1"/></tagged-backtrace>`), // std above the loop
  ].join('')}</node></trace-query-result>`;
  const r = aggregate(xml);
  rmSync(d, { recursive: true, force: true });
  const got = [...r.by].map(([k, n]) => `${k.split(' (')[0]}=${n}`).sort().join(' ');
  const want = ['Thumb 실행=1', 'Thumb 해독=2', 'log 수준 검사=1', '게스트 메모리 shim=1', '실행 루프=1'].sort().join(' ');
  const ok = r.total === 7 && r.loop === 6 && got === want;
  console.log(ok ? 'arm-interp-profile selftest: ok (7 samples · 6 under the loop · 5 buckets)' : `arm-interp-profile selftest: FAIL — total ${r.total} loop ${r.loop} · ${got}`);
  process.exit(ok ? 0 : 1);
}

const file = process.argv[2];
if (file === 'selftest') selftest();
if (!file) {
  console.error('usage: node scripts/arm-interp-profile.mjs <time-profile.xml> | selftest   (see the header)');
  process.exit(2);
}
const { total, loop, by, lines, outside } = aggregate(readFileSync(file, 'utf8'));
if (!total) {
  console.error('no samples with a backtrace in that export');
  process.exit(2);
}
const pct = (n, d) => ((100 * n) / d).toFixed(1).padStart(5);
const sorted = (m) => [...m].sort((a, b) => b[1] - a[1]);
console.log(`samples ${total} · under the ARM run loop ${loop} (${pct(loop, total).trim()}%)`);
console.log('outside the loop, top self symbols (% of all samples):');
for (const [k, n] of sorted(outside).slice(0, 5)) console.log(`  ${pct(n, total)}  ${k}`);
console.log('inside the loop (% of loop samples):');
for (const [k, n] of sorted(by)) console.log(`  ${pct(n, loop)}  ${k}`);
console.log('top source lines (% of loop samples):');
for (const [k, n] of sorted(lines).slice(0, 16)) console.log(`  ${pct(n, loop)}  ${k}`);
if (unreadable.size) console.log(`sources not readable (coarser buckets): ${[...unreadable].join(' · ')}`);
