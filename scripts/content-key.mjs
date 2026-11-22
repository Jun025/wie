#!/usr/bin/env node
// compat.json's `contentKey` — written from, and checked against, the local corpus.
// LOCAL ONLY: it reads game files (game_lab/ is git-ignored — Constraint 9), never writes one, and
// prints sha12 and counts only — no file name, no bytes.
//
//   node scripts/content-key.mjs write [--corpus <dir>] [--bin <wie_identify>]
//       each compat.json row whose file is in the corpus gets the key the engine computes for it
//   node scripts/content-key.mjs check [--corpus <dir>] [--bin <wie_identify>] [--wasm <dir>]
//       committed value == native `wie_identify` == the browser's `identify()`, row by row, and
//       both agree with the row's platform/model
//       rc 0 all equal · 1 a row differs · 2 could not measure (a row's file is not in the corpus)
//   node scripts/content-key.mjs --selftest
//       native == wasm == md5 computed here, on the committed fixtures (no game bytes)
//
// The computation lives in one place, `wie_identify::identify` (Rust). Nothing here re-derives it:
// `write` runs the native binary, `check` runs that and the wasm export (web/src/wasm — build it
// with `npm run build:wasm`). Contract: docs/contracts/featurephone-public-data.md §1.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const ROOT = new URL('..', import.meta.url).pathname;
const COMPAT = join(ROOT, 'docs/player-data/compat.json');
const [cmd, ...rest] = process.argv.slice(2);
const opt = (name, fallback) => (rest.includes(name) ? rest[rest.indexOf(name) + 1] : fallback);

/** sha256 -> path of the first .zip/.jar under `dir` with that content, for the hashes in `wanted`. */
function corpus(dir, wanted) {
  const found = new Map();
  for (const e of readdirSync(dir, { recursive: true, withFileTypes: true })) {
    if (!e.isFile() || !/\.(zip|jar)$/.test(e.name)) continue;
    const path = join(e.parentPath, e.name);
    const sha = createHash('sha256').update(readFileSync(path)).digest('hex');
    if (wanted.has(sha) && !found.has(sha)) found.set(sha, path);
  }
  return found;
}

const blank = (v) => (v === '-' ? null : v);
/** One identity per path, from the native binary (`--bin`, else a release `cargo run`). */
function native(paths) {
  const bin = opt('--bin');
  const [exe, ...pre] = bin ? [bin] : ['cargo', 'run', '-q', '--release', '-p', 'wie_identify', '--'];
  const lines = execFileSync(exe, [...pre, ...paths], { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 26 }).trimEnd().split('\n');
  if (lines.length !== paths.length) throw new Error(`wie_identify printed ${lines.length} lines for ${paths.length} files`);
  return lines.map((l) => l.split('\t').map(blank)).map(([contentKey, platform, model]) => ({ contentKey, platform, model }));
}

/** The same, from the wasm export the shell calls. */
async function wasm(paths) {
  const dir = opt('--wasm', join(ROOT, 'web/src/wasm'));
  const glue = await import(pathToFileURL(join(dir, 'wie_web.js')).href);
  glue.initSync({ module: readFileSync(join(dir, 'wie_web_bg.wasm')) });
  return paths.map((p) => glue.identify(p, readFileSync(p)) ?? { contentKey: null, platform: null, model: null });
}

const same = (a, b) => a.contentKey === b.contentKey && a.platform === b.platform && (a.model ?? null) === (b.model ?? null);

if (cmd === 'write' || cmd === 'check') {
  const compat = JSON.parse(readFileSync(COMPAT, 'utf8'));
  const found = corpus(opt('--corpus', join(ROOT, 'game_lab')), new Set(compat.entries.map((x) => x.sha256)));
  const rows = compat.entries.filter((x) => found.has(x.sha256));
  const paths = rows.map((x) => found.get(x.sha256));
  const fromNative = native(paths);
  if (cmd === 'write') {
    const keyOf = new Map(rows.map((x, i) => [x.sha256, fromNative[i].contentKey]));
    const entries = compat.entries.map(({ sha256, contentKey, ...x }) => (keyOf.has(sha256) ? { sha256, contentKey: keyOf.get(sha256), ...x } : { sha256, ...(contentKey === undefined ? {} : { contentKey }), ...x }));
    writeFileSync(COMPAT, JSON.stringify({ ...compat, entries }, null, 1) + '\n');
    console.log(`content-key write: ${rows.length} of ${compat.entries.length} rows keyed from the corpus (${compat.entries.length - rows.length} not on disk, left as they were) -> docs/player-data/compat.json`);
  } else {
    const fromWasm = await wasm(paths);
    const sha12 = (list) => (list.length ? ` (${list.slice(0, 10).map((i) => rows[i].sha256.slice(0, 12)).join(' ')}${list.length > 10 ? ' …' : ''})` : '');
    const differ = (pred) => rows.map((_, i) => i).filter((i) => !pred(rows[i], fromNative[i], fromWasm[i]));
    const checks = [
      ['committed contentKey == native', differ((row, n) => row.contentKey === n.contentKey)],
      ['native == wasm identify() (contentKey · platform · model)', differ((_, n, w) => same(n, w))],
      ['row platform/model == identify()', differ((row, n) => row.platform === n.platform && (row.model ?? null) === n.model)],
    ];
    console.log(`content-key check: ${compat.entries.length} rows · ${rows.length} on disk`);
    for (const [label, bad] of checks) console.log(`  ${label}: ${rows.length - bad.length}/${rows.length}${sha12(bad)}`);
    if (checks.some(([, bad]) => bad.length)) process.exit(1);
    if (rows.length < compat.entries.length) {
      console.log(`UNMEASURED: ${compat.entries.length - rows.length} rows have no file in the corpus`);
      process.exit(2);
    }
  }
} else if (cmd === '--selftest') {
  const dir = join(ROOT, 'test_data');
  const paths = readdirSync(dir).filter((f) => /\.(zip|jar)$/.test(f)).sort().map((f) => join(dir, f));
  const fromNative = native(paths);
  const fromWasm = await wasm(paths);
  let bad = 0;
  paths.forEach((p, i) => {
    if (!same(fromNative[i], fromWasm[i])) bad++, console.error(`selftest: native and wasm differ on ${p} — ${JSON.stringify([fromNative[i], fromWasm[i]])}`);
    // A bare jar's key is the MD5 of the file itself — computed here, not by the engine.
    if (p.endsWith('.jar') && fromNative[i].contentKey !== `jar:${createHash('md5').update(readFileSync(p)).digest('hex')}`) bad++, console.error(`selftest: ${p} is not keyed by its own MD5`);
  });
  if (!fromNative.some((x) => x.contentKey?.startsWith('zip:')) || !fromNative.some((x) => x.model)) bad++, console.error('selftest: no fixture produced a zip key / an LGT model — the comparison proved nothing');
  if (bad) process.exit(1);
  console.log(`content-key selftest: native == wasm on ${paths.length} fixtures`);
} else {
  console.error('usage: content-key.mjs [write | check | --selftest] [--corpus <dir>] [--bin <wie_identify>] [--wasm <dir>]');
  process.exit(2);
}
