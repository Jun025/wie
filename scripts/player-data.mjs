#!/usr/bin/env node
// featurephone public data — the «지원 게임 목록» (compat.json) and «업데이트 소식» (updates.json)
// the shell shows players. Contract: docs/contracts/featurephone-public-data.md.
//
//   node scripts/player-data.mjs                      check the committed data (CI: engine-contract.yml)
//   node scripts/player-data.mjs --selftest           each rule must reject its mutation
//   node scripts/player-data.mjs import <census.json> census `report` output -> docs/player-data/compat.json
//   node scripts/player-data.mjs build --out <dir>    write the shipped compat.json + updates.json
//                                                     (publish-artifact.yml; needs full history)
//
// Sources: docs/player-data/compat.json (one census run, changes stripped) and
// docs/player-updates/<YYYY-MM-DD>-<slug>.json — one change per file, so two PRs never touch
// the same line. `build` derives each title's `changes` from those files: one source, not two.
// ★No game bytes: titles, content hashes and sentences only.
import { execFileSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

const ROOT = new URL('..', import.meta.url).pathname;
const COMPAT = join(ROOT, 'docs/player-data/compat.json');
const UPDATES = join(ROOT, 'docs/player-updates');

// Same values as the shell importer (otterpebble apps/featurephone/scripts/compat-import.mjs).
export const STATUSES = ['playable', 'limited', 'not-yet'];
export const AXES = ['boot', 'render', 'input', 'longplay', 'sound', 'speed'];
export const AXIS_VALUES = ['ok', 'partial', 'no', 'unknown'];
// Axes the shell does not know yet: carried when a census measured them, never required (the shell
// importer checks AXES only and spreads the rest through). `progress` = the census's 30-min axis.
export const EXTRA_AXES = ['progress'];
export const PLATFORMS = ['KTF', 'SKT', 'LGT', 'J2ME'];
export const KINDS = ['new-support', 'fix', 'improvement', 'sound', 'speed'];
const HEX40 = /^[0-9a-f]{40}$/;
const HEX64 = /^[0-9a-f]{64}$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;
const PR = /^https:\/\/github\.com\/Jun025\/wie\/pull\/\d+$/;
const FILE = /^(\d{4}-\d{2}-\d{2})-[a-z0-9][a-z0-9-]*\.json$/;
const str = (v) => typeof v === 'string' && v.trim() !== '';

// Display names come from file names, which carry file markers: bracket tags ([큰화], [SKVM], the
// «[1]» of a re-download), carrier labels (platform is its own field), edit/patch tags, file version
// numbers, a numeric id prefix. stripMarkers() drops them and returns what it dropped as `label`:
// the screen-size variant if there was one, else the version — the shortest marker that can tell
// two files apart. Only a name that collides after stripping gets that label, else « (2)» (§1).
const SCREEN = /[\s-]*(작은화면|큰화면|큰화)$/;
const VERSION = /[\s-]*v?(\d+(?:\.\d+)+)$/i;
export function stripMarkers(raw) {
  let t = raw.replace(/\+/g, ' ');
  let screen = '';
  t = t.replace(/\[(작은화면|큰화면|큰화)\]/g, (_, m) => ((screen = m), ' '));
  t = t.replace(/\[[^\]]*\]/g, (m) => (/^\[\d+\]$/.test(m) ? '' : ' ')); // 1[1].2 -> 1.2
  t = t.replace(/^\d{6,}-/, '');
  t = t.replace(/(^|\s)(ktf|kt|lgt|skt|skvm)(?=\s|$)/gi, ' ').replace(/(?<=[0-9가-힣])(KTF|LGT|SKT)$/, '');
  t = t.replace(/[\s-]*[^\s-]*(에디트|수정판|추가다운완료)[^\s-]*/g, '');
  let version = '';
  for (let changed = true; changed; ) {
    t = t.replace(/\s+/g, ' ').trim();
    const before = t;
    t = t.replace(SCREEN, (_, m) => ((screen = m), '')).replace(VERSION, (_, m) => ((version = m), ''));
    changed = t !== before;
  }
  const label = screen ? (screen === '큰화' ? '큰화면' : screen) : version;
  return { title: t.replace(/\s+/g, ' ').trim(), label };
}

/** Sets `fileTitle` (kept for search/tracing) and a marker-free, per-platform-unique `title`. */
export function retitle(entries) {
  const out = entries.map((x) => ({ ...x, fileTitle: x.fileTitle ?? x.title, ...stripMarkers(x.fileTitle ?? x.title) }));
  const groups = Map.groupBy(out, (x) => `${x.platform}\t${x.title}`);
  for (const g of groups.values()) {
    if (g.length < 2) continue;
    g.sort((a, b) => a.label.length - b.label.length || a.label.localeCompare(b.label) || a.sha256.localeCompare(b.sha256));
    const taken = new Set();
    g.forEach((x, i) => {
      let t = x.label && !g.some((y) => y !== x && y.label === x.label) ? `${x.title} (${x.label})` : x.title;
      for (let n = 2; taken.has(t); n++) t = `${x.title} (${n})`;
      taken.add(t);
      x.title = t;
    });
  }
  return sortEntries(out.map(({ label, ...x }) => x));
}
const DISAMBIGUATOR = / \((\d+|작은화면|큰화면|v?\d+(?:\.\d+)+)\)$/;

export function validateCompat(d) {
  const e = [];
  if (!d || typeof d !== 'object') return ['compat: not an object'];
  if (d.schema !== 1) e.push(`compat: schema must be 1 (got ${JSON.stringify(d.schema)})`);
  if (!str(d.generatedAt) || !/^\d{4}-\d{2}-\d{2}/.test(d.generatedAt)) e.push('compat: generatedAt must start YYYY-MM-DD');
  if (!HEX40.test(d.enginePin ?? '')) e.push('compat: enginePin must be 40 lowercase hex');
  if (!Array.isArray(d.entries)) return [...e, 'compat: entries must be an array'];
  const seen = new Set();
  const names = new Set();
  d.entries.forEach((x, i) => {
    const at = `compat.entries[${i}]`;
    if (!x || typeof x !== 'object') return e.push(`${at}: not an object`);
    if (!HEX64.test(x.sha256 ?? '')) e.push(`${at}: sha256 must be 64 lowercase hex`);
    else if (seen.has(x.sha256)) e.push(`${at}: duplicate sha256 ${x.sha256.slice(0, 12)}`);
    else seen.add(x.sha256);
    if (!PLATFORMS.includes(x.platform)) e.push(`${at}: platform not in ${PLATFORMS}`);
    if (!(x.model === null || str(x.model))) e.push(`${at}: model must be a string or null`);
    if (!str(x.title)) e.push(`${at}: empty title`);
    else {
      const base = x.title.replace(DISAMBIGUATOR, '');
      if (stripMarkers(base).title !== base) e.push(`${at}: title carries a file marker — ${JSON.stringify(x.title)}`);
      if (names.has(`${x.platform}\t${x.title}`)) e.push(`${at}: title ${JSON.stringify(x.title)} repeats within ${x.platform}`);
      names.add(`${x.platform}\t${x.title}`);
    }
    if (!str(x.fileTitle)) e.push(`${at}: empty fileTitle (the file-derived name, kept for search)`);
    if (!STATUSES.includes(x.status)) e.push(`${at}: status not in ${STATUSES}`);
    for (const a of AXES) if (!AXIS_VALUES.includes(x.axes?.[a])) e.push(`${at}: axes.${a} not in ${AXIS_VALUES}`);
    for (const a of EXTRA_AXES) if (x.axes && a in x.axes && !AXIS_VALUES.includes(x.axes[a])) e.push(`${at}: axes.${a} not in ${AXIS_VALUES}`);
    if (!Array.isArray(x.knownIssues_ko) || !x.knownIssues_ko.every(str)) e.push(`${at}: knownIssues_ko must be non-empty strings`);
    if (!Array.isArray(x.changes)) e.push(`${at}: changes must be an array`);
    else
      x.changes.forEach((c, j) => {
        if (!DATE.test(c?.date ?? '')) e.push(`${at}.changes[${j}]: date must be YYYY-MM-DD`);
        if (!(c?.enginePin === null || HEX40.test(c?.enginePin ?? ''))) e.push(`${at}.changes[${j}]: enginePin must be 40 lowercase hex or null`);
        if (!str(c?.summary_ko)) e.push(`${at}.changes[${j}]: empty summary_ko`);
      });
  });
  return e;
}

/** `files` = [[filename, parsed]]; `shas` = the compat sha256 set. */
export function validateUpdates(files, shas) {
  const e = [];
  for (const [name, u] of files) {
    const m = FILE.exec(name);
    if (!m) e.push(`${name}: file name must be <YYYY-MM-DD>-<slug>.json`);
    if (!u || typeof u !== 'object') {
      e.push(`${name}: not an object`);
      continue;
    }
    for (const k of Object.keys(u))
      if (!['date', 'kind', 'titles', 'summary_ko', 'pr', 'enginePin'].includes(k)) e.push(`${name}: unknown key ${k}`);
    if (!DATE.test(u.date ?? '')) e.push(`${name}: date must be YYYY-MM-DD`);
    else if (m && m[1] !== u.date) e.push(`${name}: date ${u.date} differs from the file name`);
    if (!KINDS.includes(u.kind)) e.push(`${name}: kind not in ${KINDS}`);
    if (!Array.isArray(u.titles)) e.push(`${name}: titles must be an array (empty = every title)`);
    else for (const t of u.titles) if (!shas.has(t)) e.push(`${name}: title ${String(t).slice(0, 12)} is not in compat.json`);
    if (!str(u.summary_ko)) e.push(`${name}: empty summary_ko`);
    if (!PR.test(u.pr ?? '')) e.push(`${name}: pr must be https://github.com/Jun025/wie/pull/<n>`);
    if ('enginePin' in u && !HEX40.test(u.enginePin ?? '')) e.push(`${name}: enginePin must be 40 lowercase hex`);
  }
  return e;
}

/** The built updates.json, by the shell importer's rules (otterpebble updates-import.mjs). */
export function validateBuiltUpdates(d) {
  const e = HEX40.test(d.wieHead ?? '') ? [] : ['updates: wieHead must be 40 lowercase hex'];
  const ids = new Set();
  for (const u of d.entries) {
    if (ids.has(u.id)) e.push(`updates ${u.id}: duplicate id`);
    ids.add(u.id);
    if (!HEX40.test(u.enginePin ?? '')) e.push(`updates ${u.id}: enginePin must be 40 lowercase hex (got ${JSON.stringify(u.enginePin)})`);
  }
  return [...e, ...validateUpdates(d.entries.map(({ id, ...u }) => [`${id}.json`, u]), new Set())].filter((x) => !/is not in compat/.test(x));
}

// Census axis vocabulary (scripts/playability-census.mjs judge()) -> contract vocabulary.
const AXIS_MAP = { ok: 'ok', 'n/a': 'unknown', fail: 'no', none: 'no', uniform: 'no', error: 'no', silent: 'no', slow: 'no', stall: 'no', stuck: 'no' };
export function fromCensus(c) {
  const entries = c.entries.map((x) => ({
    ...x,
    fileTitle: x.title.replace(/_+/g, ' ').replace(/\s+/g, ' ').trim(), // census titles are file names
    axes: Object.fromEntries([...AXES, ...EXTRA_AXES.filter((a) => a in (x.axes ?? {}))].map((a) => [a, AXIS_MAP[x.axes?.[a]] ?? `?${x.axes?.[a]}`])),
    changes: [],
  }));
  return { schema: 1, generatedAt: c.generatedAt, enginePin: c.enginePin, entries: retitle(entries) };
}
const sortEntries = (es) => [...es].sort((a, b) => a.platform.localeCompare(b.platform) || a.title.localeCompare(b.title, 'ko') || a.sha256.localeCompare(b.sha256));

function readUpdates() {
  return readdirSync(UPDATES)
    .filter((f) => f.endsWith('.json'))
    .sort()
    .map((f) => [f, JSON.parse(readFileSync(join(UPDATES, f), 'utf8'))]);
}

// The first-parent commit that added the file = the landing that shipped the change.
// ★--no-patch is load-bearing: --diff-merges=<format> also turns the patch on (git 2.55), and the last
// output line was then the file's closing «+}» — 2026-09-29 the shell refused every such update.
export function landedPin(file, cwd = ROOT) {
  const out = execFileSync('git', ['log', '--first-parent', '--diff-merges=first-parent', '--diff-filter=A', '--no-patch', '--format=%H', '--', file], {
    cwd,
    encoding: 'utf8',
  }).trim();
  return out.split('\n').pop() || null;
}

export function assemble(compat, files, pinOf, wieHead) {
  const entries = files
    .map(([name, u]) => ({ id: name.replace(/\.json$/, ''), ...u, enginePin: u.enginePin ?? pinOf(name) }))
    .sort((a, b) => b.date.localeCompare(a.date) || b.id.localeCompare(a.id));
  const byTitle = new Map();
  for (const u of entries)
    for (const t of u.titles) // an all-titles update (titles: []) is news in updates.json, not a per-title row
      byTitle.set(t, [...(byTitle.get(t) ?? []), { date: u.date, enginePin: u.enginePin, summary_ko: u.summary_ko, kind: u.kind, pr: u.pr }]);
  return {
    compat: { ...compat, entries: compat.entries.map((x) => ({ ...x, changes: byTitle.get(x.sha256) ?? [] })) },
    updates: { schema: 1, generatedAt: new Date().toISOString(), wieHead, entries },
  };
}

function selftest() {
  const sha = 'a'.repeat(64);
  const good = {
    schema: 1,
    generatedAt: '2026-09-27',
    enginePin: 'b'.repeat(40),
    entries: [{ sha256: sha, platform: 'KTF', model: null, title: 't', fileTitle: '[큰화]t', status: 'playable', axes: Object.fromEntries(AXES.map((a) => [a, 'ok'])), knownIssues_ko: [], changes: [] }],
  };
  const upd = { date: '2026-09-27', kind: 'fix', titles: [sha], summary_ko: '고쳤어요.', pr: 'https://github.com/Jun025/wie/pull/1' };
  const cases = [
    ['compat schema 2', { ...good, schema: 2 }, null],
    ['unknown status', { ...good, entries: [{ ...good.entries[0], status: 'boots' }] }, null],
    ['census axis value leaks through', { ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, boot: 'fail' } }] }, null],
    ['census progress value leaks through', { ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, progress: 'stuck' } }] }, null],
    ['duplicate sha', { ...good, entries: [good.entries[0], good.entries[0]] }, null],
    ['short enginePin', { ...good, enginePin: 'abc' }, null],
    ['title keeps a bracket tag', { ...good, entries: [{ ...good.entries[0], title: '[큰화]t' }] }, null],
    ['title keeps a carrier label', { ...good, entries: [{ ...good.entries[0], title: 't kt' }] }, null],
    ['title keeps a file version', { ...good, entries: [{ ...good.entries[0], title: 't 01.00.05' }] }, null],
    ['title keeps an edit tag', { ...good, entries: [{ ...good.entries[0], title: 't 2억에디트1' }] }, null],
    ['same title twice on one platform', { ...good, entries: [good.entries[0], { ...good.entries[0], sha256: 'c'.repeat(64) }] }, null],
    ['fileTitle dropped', { ...good, entries: [{ ...good.entries[0], fileTitle: undefined }] }, null],
    ['update: title not in compat', good, [['2026-09-27-x.json', { ...upd, titles: ['c'.repeat(64)] }]]],
    ['update: date differs from name', good, [['2026-09-26-x.json', upd]]],
    ['update: bad kind', good, [['2026-09-27-x.json', { ...upd, kind: 'misc' }]]],
    ['update: bad file name', good, [['x.json', upd]]],
    ['update: pr is not a wie PR', good, [['2026-09-27-x.json', { ...upd, pr: 'https://example.com/1' }]]],
    ['update: unknown key', good, [['2026-09-27-x.json', { ...upd, summary: '' }]]],
  ];
  const errs = (c, u) => [...validateCompat(c), ...validateUpdates(u ?? [], new Set((c.entries ?? []).map((x) => x.sha256)))];
  let bad = 0;
  if (errs(good, [['2026-09-27-x.json', upd]]).length) bad++, console.error('selftest: the good fixture is rejected');
  for (const [label, c, u] of cases) if (!errs(c, u).length) bad++, console.error(`selftest: NOT rejected — ${label}`);
  if (fromCensus({ ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, render: 'uniform', speed: 'n/a' } }] }).entries[0].axes.render !== 'no')
    bad++, console.error('selftest: census uniform must map to no');
  const withProgress = (v) => fromCensus({ ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, ...v } }] }).entries[0].axes;
  if (withProgress({ progress: 'stuck' }).progress !== 'no' || 'progress' in withProgress({}))
    bad++, console.error('selftest: progress must map stuck to no, and stay absent when the census did not measure it');
  const { compat } = assemble(good, [['2026-09-27-x.json', upd]], () => 'd'.repeat(40), 'e'.repeat(40));
  if (compat.entries[0].changes[0]?.summary_ko !== upd.summary_ko) bad++, console.error('selftest: an update must reach the title it names');
  const shipped = { ...compat, entries: [{ ...compat.entries[0], changes: [{ ...compat.entries[0].changes[0], enginePin: '+}' }] }] };
  if (!validateCompat(shipped).length) bad++, console.error('selftest: NOT rejected — shipped change with enginePin «+}»');
  const built = { wieHead: 'e'.repeat(40), entries: [{ id: '2026-09-27-x', ...upd, enginePin: '+}' }] };
  if (!validateBuiltUpdates(built).length) bad++, console.error('selftest: NOT rejected — built update with enginePin «+}»');
  if (validateBuiltUpdates({ ...built, entries: [{ ...built.entries[0], enginePin: 'd'.repeat(40) }] }).length) bad++, console.error('selftest: a good built update is rejected');
  // landedPin against a real repo where a merge commit adds the file (the 2026-09-29 «+}» case).
  const repo = mkdtempSync(join(tmpdir(), 'player-data-'));
  try {
    const git = (...a) => execFileSync('git', ['-c', 'user.name=t', '-c', 'user.email=t@t', ...a], { cwd: repo, encoding: 'utf8' }).trim();
    git('init', '-q', '-b', 'main');
    git('commit', '-q', '--allow-empty', '-m', 'root');
    git('checkout', '-q', '-b', 'pr');
    writeFileSync(join(repo, 'u.json'), '{\n  "a": 1\n}\n');
    git('add', 'u.json');
    git('commit', '-q', '-m', 'add');
    git('checkout', '-q', 'main');
    git('merge', '-q', '--no-ff', '-m', 'land', 'pr');
    const pin = landedPin('u.json', repo);
    if (pin !== git('rev-parse', 'HEAD')) bad++, console.error(`selftest: landedPin must be the merge commit, got ${JSON.stringify(pin)}`);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
  if (bad) process.exit(1);
  console.log(`player-data selftest: ${cases.length + 6} rules each reject their mutation`);
}

const [cmd, ...args] = process.argv.slice(2);
if (import.meta.url !== pathToFileURL(process.argv[1] ?? '').href);
else if (cmd === '--selftest') selftest();
else if (cmd === 'import') {
  const c = fromCensus(JSON.parse(readFileSync(args[0], 'utf8')));
  const e = validateCompat(c);
  if (e.length) {
    console.error(`import refused — ${e.length} violations, nothing written:\n  ${e.slice(0, 20).join('\n  ')}`);
    process.exit(1);
  }
  mkdirSync(join(ROOT, 'docs/player-data'), { recursive: true });
  writeFileSync(COMPAT, JSON.stringify(c, null, 1) + '\n');
  console.log(`imported ${c.entries.length} titles (pin ${c.enginePin.slice(0, 8)}) -> docs/player-data/compat.json`);
} else if (!cmd || cmd === 'build') {
  const compat = JSON.parse(readFileSync(COMPAT, 'utf8'));
  const files = readUpdates();
  const e = [...validateCompat(compat), ...validateUpdates(files, new Set(compat.entries.map((x) => x.sha256)))];
  if (e.length) {
    console.error(`player-data: ${e.length} violations (docs/contracts/featurephone-public-data.md):\n  ${e.join('\n  ')}`);
    process.exit(1);
  }
  const count = (k) => JSON.stringify(compat.entries.reduce((m, x) => ((m[x[k]] = (m[x[k]] ?? 0) + 1), m), {}));
  console.log(`player-data OK — ${compat.entries.length} titles · status ${count('status')} · platform ${count('platform')} · ${files.length} updates`);
  if (cmd === 'build') {
    const oi = args.indexOf('--out');
    if (oi < 0) process.exit(2);
    const head = execFileSync('git', ['rev-parse', 'HEAD'], { cwd: ROOT, encoding: 'utf8' }).trim();
    // An update with no explicit pin and no landing commit (shallow clone, or not on main yet)
    // is left out rather than shipped with a made-up pin.
    const pinOf = (name) => landedPin(join('docs/player-updates', name));
    const out = assemble(compat, files, pinOf, head);
    const missing = out.updates.entries.filter((u) => !u.enginePin);
    for (const u of missing) console.log(`::warning::player-data: ${u.id} has no landing commit — left out of this build`);
    if (missing.length) Object.assign(out, assemble(compat, files.filter(([n]) => !missing.some((u) => `${u.id}.json` === n)), pinOf, head));
    // Ship nothing the shell would refuse: it drops the whole file on one violation.
    const shipErr = [...validateCompat(out.compat), ...validateBuiltUpdates(out.updates)];
    if (shipErr.length) {
      console.error(`player-data build: ${shipErr.length} violations in the built output, nothing written:\n  ${shipErr.slice(0, 20).join('\n  ')}`);
      process.exit(1);
    }
    mkdirSync(args[oi + 1], { recursive: true });
    writeFileSync(join(args[oi + 1], 'compat.json'), JSON.stringify(out.compat) + '\n');
    writeFileSync(join(args[oi + 1], 'updates.json'), JSON.stringify(out.updates) + '\n');
    console.log(`built -> ${args[oi + 1]}/{compat,updates}.json (${out.updates.entries.length} updates)`);
  }
} else {
  console.error('usage: player-data.mjs [--selftest | import <census compat.json> | build --out <dir>]');
  process.exit(2);
}
