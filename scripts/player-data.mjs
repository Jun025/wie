#!/usr/bin/env node
// featurephone public data — the «지원 게임 목록» (compat.json) and «업데이트 소식» (updates.json)
// the shell shows players. Contract: docs/contracts/featurephone-public-data.md.
//
//   node scripts/player-data.mjs                      check the committed data (CI: engine-contract.yml)
//   node scripts/player-data.mjs --selftest           each rule must reject its mutation
//   node scripts/player-data.mjs import <census.json> census `report` output -> docs/player-data/compat.json
//   node scripts/player-data.mjs normalize            re-derive names/groups, order and runtimeAdjustments in docs/player-data/compat.json
//   node scripts/player-data.mjs build --out <dir>    write the shipped compat.json + updates.json
//                                                     (publish-artifact.yml; needs full history)
//
// Sources: docs/player-data/compat.json (one census run, changes stripped) and
// docs/player-updates/<YYYY-MM-DD>-<slug>.json — one change per file, so two PRs never touch
// the same line. `build` derives each title's `changes` from those files: one source, not two.
// docs/contracts/runtime-adjustments.tsv (the ledger of per-title engine adjustments) is the one
// source of each row's `runtimeAdjustments`; the engine tables are held to it by `cargo test`.
// Each row's `contentKey` is what the engine computes from the file (`wie_identify`), written by
// scripts/content-key.mjs from the local corpus; here it is held to the ledger's jar keys.
// ★No game bytes: titles, content hashes and sentences only.
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, realpathSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const ROOT = new URL('..', import.meta.url).pathname;
const COMPAT = join(ROOT, 'docs/player-data/compat.json');
const UPDATES = join(ROOT, 'docs/player-updates');
const LEDGER = join(ROOT, 'docs/contracts/runtime-adjustments.tsv');

// Same values as the shell importer (otterpebble apps/featurephone/scripts/compat-import.mjs).
export const STATUSES = ['playable', 'limited', 'not-yet'];
export const AXES = ['boot', 'render', 'input', 'longplay', 'sound', 'speed'];
export const AXIS_VALUES = ['ok', 'partial', 'no', 'unknown'];
// Axes beyond the six: carried only on rows a census measured, never required (the shell importer
// checks AXES only and spreads the rest through). `progress` = the census's progress axis, drawn by
// the shell's PROGRESS_UI (otterpebble apps/featurephone/lib/compat.ts).
export const EXTRA_AXES = ['progress'];
// Each extra axis carries its OWN public vocabulary — the one its consumer draws. The shell's
// PROGRESS_UI (otterpebble #1244) knows `ok` and `stuck` and hides any other value, so mapping stuck
// to the six-axis `no` would show the 22 `ok` rows and hide every stuck one — a display biased to
// the good side. A census `error` (the run crashed) is not progress either, so it ships as `stuck`;
// `n/a` (not measured) ships as no key at all.
export const EXTRA_AXIS_VALUES = { progress: ['ok', 'stuck'] };
const EXTRA_AXIS_MAP = { progress: { ok: 'ok', stuck: 'stuck', error: 'stuck' } };
export const PLATFORMS = ['KTF', 'SKT', 'LGT', 'J2ME'];
export const KINDS = ['new-support', 'fix', 'improvement', 'sound', 'speed'];
// `runtimeAdjustments` vocabulary (§1) — what the engine does for this one title after loading its file.
export const ADJUSTMENTS = ['lock-lift', 'owner-file-skip', 'owner-file-bytes', 'orphan-flag', 'device-bound-save', 'local-server', 'display-size'];
const LEDGER_CHANGES = ['memory-code', 'file-not-mounted', 'file-bytes', 'response', 'engine-setting'];
const LEDGER_SCOPES = ['any', 'zip', 'aid'];
const LEDGER_CLASSES = ['operator-directed', 'adopted', 'ratified-2026-11-20'];
const HEX40 = /^[0-9a-f]{40}$/;
const HEX64 = /^[0-9a-f]{64}$/;
// `contentKey` (§1): the container the engine loaded, and the MD5 of the jar it runs.
const CONTENT_KEY = /^(zip|jar):[0-9a-f]{32}$/;
const DATE = /^\d{4}-\d{2}-\d{2}$/;
const PR = /^https:\/\/github\.com\/Jun025\/wie\/pull\/\d+$/;
const FILE = /^(\d{4}-\d{2}-\d{2})-[a-z0-9][a-z0-9-]*\.json$/;
// Punctuation the shell's site copy refuses — same set as otterpebble scripts/user-copy-glyph-check.mjs
// (r2), whose importer rejects the whole updates file on one hit (otterpebble#1279).
const FORBIDDEN_PUNCT = /[—―–·・ㆍ‧∙•]/;
const str = (v) => typeof v === 'string' && v.trim() !== '';

// Display names come from file names, which carry file markers: bracket tags ([큰화], [SKVM], the
// «[1]» of a re-download), carrier labels (platform is its own field), edit/patch tags, file version
// numbers, a numeric id prefix. stripMarkers() drops them and returns what it dropped as `label`:
// the screen-size variant if there was one, else the version — the shortest marker that can tell
// two files apart. A label ships as `variant` only inside a same-name group, and only when unique there (§1).
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

// `game` is one spelling per game: names equal under gameKey() (the shell's normalizeGameKey — NFKC,
// lower case, no spaces or hyphens) share the group's representative spelling, on every carrier (§1).
export const gameKey = (s) => s.normalize('NFKC').toLowerCase().replace(/[\s-]/g, '');
const count = (s, re) => s.match(re)?.length ?? 0;
// Representative spelling: more capitals («KBO», «EX»), then more spaces, then fewer hyphens, then code order.
const bySpelling = (a, b) =>
  count(b, /\p{Lu}/gu) - count(a, /\p{Lu}/gu) || count(b, / /g) - count(a, / /g) || count(a, /-/g) - count(b, /-/g) || (a < b ? -1 : a > b ? 1 : 0);

// Judged from archive metadata only (PID·Name·Ver·DisplaySize of __adf__/app_info/.msd, entry list,
// main-jar hash — docs/report/0555, 0560). sha12 -> what the file name cannot say: the game's real
// name (`title` — a different game behind the same file name, or the same game under another name),
// or the label the census file name lost (`variant`). A repackage of a build that already carries the
// label gets none: (platform, title, variant) stays unique, and the shell names that row by its hash.
const SAME_NAME_HAND = {
  '1cdea1985955': { title: '메이플스토리 마법사편' }, // PD001295 «메이플-마법사편» — not the same product as ↓
  '3b5b98afa890': { title: '메이플스토리 전사편' }, // PD002198, file name «(전사편)»
  '52edc54e2b23': { title: '메이플스토리 마법사편' }, // SKT port of 1cdea1985955: .msd MIDlet-Name «메이플스토리마법사편» · 135/138 shared entries identical
  '1f0d7e81336a': { title: '영웅서기 1' }, // SKT port of KTF «영웅서기 1»: both descriptors name the same subtitle, 345/350 shared entries identical
  f981d228b757: { title: '물가에돌튕기기3' }, // __adf__ Name «물가에돌튕기기3» · PD005691 · same jar as 4a4d2ac046e5
  d1fba7ad5d1f: { title: '영웅전설3 하얀마녀' }, // same product as b907b0faf483: PD004460 · Name «영웅전설3» · Ver 01.00.02 both, 243/244 jar entries identical; both Desc carry the subtitle (0585)
  '306dcdb03842': { variant: '큰화면' }, // DisplaySize 240*320 · Ver 01.00.05
  e159b9727dbd: { variant: '작은화면' }, // 176*220 · 01.00.01
  '4288d8c1c6ac': { variant: '큰화면' }, // 240*320 (file name «(큰화면)»)
  '3ccc6cf147d2': { variant: '작은화면' }, // 176*220
  '0bc9cc28c05a': { variant: '01.00.02' }, // app_info Ver
  '6f170f692586': { variant: '01.00.04' },
};
// (platform, game) groups whose rows were judged the same game; any other group of ≥2 rows ships `undetermined`.
const SAME_GAME = new Set(
  [
    ...['거상', '던파거너편', '리얼비트', '리얼싸커2009', '마린블루스', '마이프린세스2', '영웅전설4 주홍물방울', '이노티아1', '이노티아연대기2'],
    ...['지크', '질주쾌감 스케쳐', '추억의달고나', '크아bnb2007', '투스워즈', '마스터오브소드2', '붕어빵타이쿤3', '영어뇌습격', '절묘한타이밍', '프린세스메이커4'],
    ...['크로노스 윙', 'KBO프로야구2009', '드래곤나이트EX', '물가에돌튕기기3'], // 0560
    ...['영웅전설3 하얀마녀'], // 0585
  ]
    .map((t) => `KTF\t${t}`)
    .concat(['LGT\t영웅서기3', 'SKT\t웰루시아', 'SKT\t프린세스메이커4'])
    .map(gameKey), // over the whole «platform\tgame» string — retitle() builds its lookup key the same way
);
export const SAME_TITLE = ['same-game', 'undetermined'];

/** sha12 -> kinds in ADJUSTMENTS order, from the ledger's rows (one row per engine key × package). */
export function adjustmentsOf(ledger) {
  const by = Map.groupBy(ledger.rows ?? [], (r) => r.package);
  return new Map([...by].map(([sha12, rows]) => [sha12, ADJUSTMENTS.filter((k) => rows.some((r) => r.kind === k))]));
}
const LEDGER_COLUMNS = ['device', 'key', 'package', 'platform', 'title', 'kind', 'changes', 'scope', 'class', 'basis'];
/** Tab-separated, `#` lines are comments, the first other line is the header. A row keeps its line number for messages. */
export function parseLedger(text) {
  const [header, ...body] = text.split(/\r?\n/).map((raw, i) => ({ line: i + 1, raw, cells: raw.split('\t') })).filter((x) => x.raw !== '' && !x.raw.startsWith('#'));
  return { header: header?.cells ?? [], rows: body.map(({ line, cells }) => ({ line, cells, ...Object.fromEntries(LEDGER_COLUMNS.map((k, i) => [k, cells[i]])) })) };
}
const readLedger = () => parseLedger(readFileSync(LEDGER, 'utf8'));

/** The ledger's own rules, and that each row names one compat row by its real platform and title. */
export function validateLedger(l, entries) {
  if (l.header.join('\t') !== LEDGER_COLUMNS.join('\t')) return [`ledger: header row must be ${LEDGER_COLUMNS.join(' <tab> ')}`];
  const e = [];
  const seen = new Set();
  for (const r of l.rows) {
    const at = `ledger line ${r.line} (${r.device} ${r.package})`;
    if (r.cells.length !== LEDGER_COLUMNS.length) e.push(`${at}: ${r.cells.length} tab-separated cells, expected ${LEDGER_COLUMNS.length}`);
    for (const k of LEDGER_COLUMNS) if (!str(r[k])) e.push(`${at}: empty ${k}${k === 'basis' ? ' — a row must say where it came from' : ''}`);
    if (!ADJUSTMENTS.includes(r.kind)) e.push(`${at}: kind not in ${ADJUSTMENTS}`);
    if (!LEDGER_CHANGES.includes(r.changes)) e.push(`${at}: changes not in ${LEDGER_CHANGES}`);
    if (!LEDGER_SCOPES.includes(r.scope)) e.push(`${at}: scope not in ${LEDGER_SCOPES}`);
    if (!LEDGER_CLASSES.includes(r.class)) e.push(`${at}: class not in ${LEDGER_CLASSES}`);
    const id = `${r.device}\t${r.key}\t${r.package}`;
    if (seen.has(id)) e.push(`${at}: repeats`);
    seen.add(id);
    const hit = /^[0-9a-f]{12}$/.test(r.package ?? '') ? entries.filter((x) => x.sha256?.startsWith(r.package)) : [];
    if (hit.length !== 1) e.push(`${at}: package must be the first 12 hex of exactly one compat.json row (found ${hit.length})`);
    else if (hit[0].platform !== r.platform || hit[0].title !== r.title) e.push(`${at}: platform/title differ from compat.json (${hit[0].platform} ${JSON.stringify(hit[0].title)})`);
    // A zip-scoped table keys on the MD5 of the package's jar — the same value as that row's contentKey.
    else if (r.scope === 'zip' && 'contentKey' in hit[0] && hit[0].contentKey !== `zip:${r.key}`)
      e.push(`${at}: the engine table keys this package by jar ${String(r.key).slice(0, 12)}, compat.json contentKey says ${JSON.stringify(hit[0].contentKey)} — never typed in: \`node scripts/content-key.mjs write\``);
  }
  return e;
}

/** Sets `fileTitle` (kept for search/tracing), a marker-free `title`, the carrier-free group key `game`, `variant`, and the ledger's `runtimeAdjustments`. */
export function retitle(entries, adjustments = adjustmentsOf(readLedger())) {
  const out = entries.map(({ sameTitle, variant, runtimeAdjustments, ...x }) => {
    const fileTitle = x.fileTitle ?? x.title;
    const { title, label } = stripMarkers(fileTitle);
    const hand = SAME_NAME_HAND[x.sha256.slice(0, 12)] ?? {};
    return { ...x, fileTitle, title: hand.title ?? title, label: hand.variant ?? label };
  });
  for (const g of Map.groupBy(out, (x) => gameKey(x.title)).values()) {
    const spelling = g.map((x) => x.title).sort(bySpelling)[0];
    for (const x of g) x.title = x.game = spelling;
  }
  for (const g of Map.groupBy(out, (x) => `${x.platform}\t${x.game}`).values())
    for (const x of g) {
      x.variant = g.length > 1 && x.label && !g.some((y) => y !== x && y.label === x.label) ? x.label : null;
      if (g.length > 1) x.sameTitle = SAME_GAME.has(gameKey(`${x.platform}\t${x.game}`)) ? 'same-game' : 'undetermined';
    }
  return sortEntries(
    out.map(({ label, ...x }) => {
      const kinds = adjustments.get(x.sha256.slice(0, 12));
      return kinds ? { ...x, runtimeAdjustments: kinds } : x; // no key = no per-title adjustment (§1)
    }),
  );
}
const DISAMBIGUATOR = / \((\d+|작은화면|큰화면|v?\d+(?:\.\d+)+)\)$/;

// The group's representative comes first — the shell reads a (platform, game) group's first row (§1).
const STATUS_RANK = Object.fromEntries(STATUSES.map((s, i) => [s, i]));
const okAxes = (x) => AXES.filter((a) => x.axes?.[a] === 'ok').length;
const byRepresentative = (a, b) =>
  STATUS_RANK[a.status] - STATUS_RANK[b.status] || okAxes(b) - okAxes(a) || (b.changes?.length ?? 0) - (a.changes?.length ?? 0) || a.sha256.localeCompare(b.sha256);
const sortEntries = (es) => [...es].sort((a, b) => a.platform.localeCompare(b.platform) || a.game.localeCompare(b.game, 'ko') || byRepresentative(a, b));

export function validateCompat(d) {
  const e = [];
  if (!d || typeof d !== 'object') return ['compat: not an object'];
  if (d.schema !== 1) e.push(`compat: schema must be 1 (got ${JSON.stringify(d.schema)})`);
  if (!str(d.generatedAt) || !/^\d{4}-\d{2}-\d{2}/.test(d.generatedAt)) e.push('compat: generatedAt must start YYYY-MM-DD');
  if (!HEX40.test(d.enginePin ?? '')) e.push('compat: enginePin must be 40 lowercase hex');
  if (!Array.isArray(d.entries)) return [...e, 'compat: entries must be an array'];
  const seen = new Set();
  const names = new Set();
  for (const [k, g] of Map.groupBy(d.entries.filter((x) => str(x?.game)), (x) => gameKey(x.game))) {
    const spellings = [...new Set(g.map((x) => x.game))];
    if (spellings.length > 1) e.push(`compat: game ${JSON.stringify(k)} is spelled ${spellings.map((s) => JSON.stringify(s)).join(' / ')} — one game, one key (§1; \`node scripts/player-data.mjs normalize\`)`);
  }
  const perTitle = Map.groupBy(d.entries, (x) => `${x?.platform}\t${x?.title}`);
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
      if (stripMarkers(x.title).title !== x.title) e.push(`${at}: title carries a file marker — ${JSON.stringify(x.title)}`);
      if (DISAMBIGUATOR.test(x.title)) e.push(`${at}: title carries an ordinal or variant label (that is \`variant\`, §1) — ${JSON.stringify(x.title)}`);
      const key = `${x.platform}\t${x.title}\t${x.variant ?? ''}`;
      if (x.variant != null && names.has(key)) e.push(`${at}: (platform, title, variant) ${JSON.stringify(key)} repeats`);
      names.add(key);
      if (!SAME_TITLE.includes(x.sameTitle) && perTitle.get(`${x.platform}\t${x.title}`).length > 1)
        e.push(`${at}: title ${JSON.stringify(x.title)} repeats within ${x.platform} — a repeated title must be one game's files (sameTitle ${SAME_TITLE})`);
    }
    if (!str(x.game)) e.push(`${at}: empty game (the carrier-free group key)`);
    if (!(x.variant === null || str(x.variant))) e.push(`${at}: variant must be a string or null`);
    const prev = d.entries[i - 1];
    if (prev?.platform === x.platform && prev?.game === x.game && STATUSES.includes(prev.status) && STATUSES.includes(x.status) && byRepresentative(prev, x) > 0)
      e.push(`${at}: comes after a worse row of its group ${JSON.stringify(x.game)} — the representative goes first (§1; \`node scripts/player-data.mjs normalize\`)`);
    if (!str(x.fileTitle)) e.push(`${at}: empty fileTitle (the file-derived name, kept for search)`);
    if (!STATUSES.includes(x.status)) e.push(`${at}: status not in ${STATUSES}`);
    for (const a of AXES) if (!AXIS_VALUES.includes(x.axes?.[a])) e.push(`${at}: axes.${a} not in ${AXIS_VALUES}`);
    for (const a of EXTRA_AXES)
      if (x.axes && a in x.axes && !EXTRA_AXIS_VALUES[a].includes(x.axes[a])) e.push(`${at}: axes.${a} not in ${EXTRA_AXIS_VALUES[a]}`);
    if (!Array.isArray(x.knownIssues_ko) || !x.knownIssues_ko.every(str)) e.push(`${at}: knownIssues_ko must be non-empty strings`);
    // A row the shell marks limited/not-yet with no sentence tells the player «화면만» and not why.
    else if (x.status !== 'playable' && !x.knownIssues_ko.length) e.push(`${at}: status ${x.status} needs at least one knownIssues_ko sentence (why it is not playable)`);
    // Optional: how to move a working title forward. Absent when there is none — an empty array is refused
    // so «no tips» has one spelling the shell can test.
    if ('playTips_ko' in x && !(Array.isArray(x.playTips_ko) && x.playTips_ko.length && x.playTips_ko.every(str)))
      e.push(`${at}: playTips_ko must be omitted or a non-empty array of non-empty strings`);
    // Optional, and never hand-written: `build` refuses a value the ledger does not derive (below). No key = none.
    if ('runtimeAdjustments' in x && !(Array.isArray(x.runtimeAdjustments) && x.runtimeAdjustments.length && x.runtimeAdjustments.every((k, j, a) => ADJUSTMENTS.includes(k) && a.indexOf(k) === j)))
      e.push(`${at}: runtimeAdjustments must be omitted or a non-empty array of distinct ${ADJUSTMENTS}`);
    // Optional for a consumer; computed from the file, never typed in (the check below and the ledger hold it).
    if ('contentKey' in x && !(x.contentKey === null || CONTENT_KEY.test(x.contentKey))) e.push(`${at}: contentKey must be null or <zip|jar>:<32 lowercase hex>`);
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

/** The committed list carries a contentKey on every row (null = the engine finds no jar in that file): a row without one was never measured. */
export function missingContentKeys(entries) {
  const missing = entries.filter((x) => !('contentKey' in x));
  return missing.length
    ? [`compat: ${missing.length} rows have no contentKey (${missing.slice(0, 5).map((x) => x.sha256.slice(0, 12)).join(' ')}${missing.length > 5 ? ' …' : ''}) — \`node scripts/content-key.mjs write\` (reads the local corpus; never typed in)`]
    : [];
}

/** contentKey groups: how many keys, how many shared by ≥2 rows, and the shared ones whose rows differ in status. */
export function contentKeyGroups(entries) {
  const groups = [...Map.groupBy(entries.filter((x) => x.contentKey), (x) => x.contentKey).values()];
  const shared = groups.filter((g) => g.length > 1);
  return { distinct: groups.length, shared, split: shared.filter((g) => new Set(g.map((x) => x.status)).size > 1) };
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
    else if (FORBIDDEN_PUNCT.test(u.summary_ko)) e.push(`${name}: summary_ko has ${u.summary_ko.match(FORBIDDEN_PUNCT)[0]} — the site copy refuses —―–·・ㆍ‧∙•`);
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
// `short` (a long run the guest ended early) measured nothing about the full time: unknown, never ok.
const AXIS_MAP = { ok: 'ok', 'n/a': 'unknown', short: 'unknown', fail: 'no', none: 'no', uniform: 'no', error: 'no', silent: 'no', slow: 'no', stall: 'no' };
export function fromCensus(c) {
  const entries = c.entries.map((x) => ({
    ...x,
    fileTitle: x.title.replace(/_+/g, ' ').replace(/\s+/g, ' ').trim(), // census titles are file names
    axes: Object.fromEntries([
      ...AXES.map((a) => [a, AXIS_MAP[x.axes?.[a]] ?? `?${x.axes?.[a]}`]),
      ...EXTRA_AXES.filter((a) => a in (x.axes ?? {}) && x.axes[a] !== 'n/a').map((a) => [a, EXTRA_AXIS_MAP[a][x.axes[a]] ?? `?${x.axes[a]}`]),
    ]),
    changes: [],
  }));
  return { schema: 1, generatedAt: c.generatedAt, enginePin: c.enginePin, entries: retitle(entries) };
}

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

// «마지막 확인» (the shell's /support header reads generatedAt): rounds edit compat rows by hand and
// leave generatedAt at the last full census, so it stood still while rows changed daily (10-04 vs
// #471…#512). Bumping it in each PR would make every compat PR conflict on one line, so the shipped
// value is derived here instead: the later of the census time and the last first-parent landing that
// touched compat.json. The committer date keeps its own offset, so slice(0, 10) is the landing's
// local date. A shallow clone keeps the census time: there `git log -- <path>` returns the boundary
// commit (it has no parent, so it looks like it adds every file) — i.e. HEAD, whatever it touched.
export function checkedAt(generatedAt, cwd = ROOT) {
  if (execFileSync('git', ['rev-parse', '--is-shallow-repository'], { cwd, encoding: 'utf8' }).trim() === 'true') return generatedAt;
  const landed = execFileSync('git', ['log', '--first-parent', '-1', '--format=%cI', '--', 'docs/player-data/compat.json'], { cwd, encoding: 'utf8' }).trim();
  return landed && new Date(landed) > new Date(generatedAt) ? landed : generatedAt;
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
    compat: { ...compat, entries: sortEntries(compat.entries.map((x) => ({ ...x, changes: byTitle.get(x.sha256) ?? [] }))) },
    updates: { schema: 1, generatedAt: new Date().toISOString(), wieHead, entries },
  };
}

// What `build` ships, before validation. A function so the selftest runs the same wiring build does.
export function buildOutput(compat, files, cwd = ROOT) {
  const head = execFileSync('git', ['rev-parse', 'HEAD'], { cwd, encoding: 'utf8' }).trim();
  // An update with no explicit pin and no landing commit (shallow clone, or not on main yet)
  // is left out rather than shipped with a made-up pin.
  const pinOf = (name) => landedPin(join('docs/player-updates', name), cwd);
  compat = { ...compat, generatedAt: checkedAt(compat.generatedAt, cwd) };
  const out = assemble(compat, files, pinOf, head);
  const missing = out.updates.entries.filter((u) => !u.enginePin);
  for (const u of missing) console.log(`::warning::player-data: ${u.id} has no landing commit — left out of this build`);
  if (missing.length) Object.assign(out, assemble(compat, files.filter(([n]) => !missing.some((u) => `${u.id}.json` === n)), pinOf, head));
  return out;
}

function selftest() {
  const sha = 'a'.repeat(64);
  const good = {
    schema: 1,
    generatedAt: '2026-09-27',
    enginePin: 'b'.repeat(40),
    entries: [{ sha256: sha, contentKey: `zip:${'9'.repeat(32)}`, platform: 'KTF', model: null, title: 't', game: 't', variant: null, fileTitle: '[큰화]t', status: 'playable', axes: Object.fromEntries(AXES.map((a) => [a, 'ok'])), knownIssues_ko: [], changes: [] }],
  };
  const pair = [good.entries[0], { ...good.entries[0], sha256: 'c'.repeat(64) }].map((x) => ({ ...x, sameTitle: 'undetermined' }));
  const upd = { date: '2026-09-27', kind: 'fix', titles: [sha], summary_ko: '고쳤어요.', pr: 'https://github.com/Jun025/wie/pull/1' };
  const cases = [
    ['compat schema 2', { ...good, schema: 2 }, null],
    ['unknown status', { ...good, entries: [{ ...good.entries[0], status: 'boots' }] }, null],
    ['census axis value leaks through', { ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, boot: 'fail' } }] }, null],
    ['six-axis value on progress', { ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, progress: 'no' } }] }, null],
    ['unmeasured progress shipped as a value', { ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, progress: 'unknown' } }] }, null],
    ['limited row with no reason', { ...good, entries: [{ ...good.entries[0], status: 'limited' }] }, null],
    ['not-yet row with no reason', { ...good, entries: [{ ...good.entries[0], status: 'not-yet' }] }, null],
    ['empty playTips_ko array', { ...good, entries: [{ ...good.entries[0], playTips_ko: [] }] }, null],
    ['playTips_ko with an empty line', { ...good, entries: [{ ...good.entries[0], playTips_ko: [''] }] }, null],
    ['playTips_ko as a string', { ...good, entries: [{ ...good.entries[0], playTips_ko: '키를 누르세요.' }] }, null],
    ['duplicate sha', { ...good, entries: [good.entries[0], good.entries[0]] }, null],
    ['short enginePin', { ...good, enginePin: 'abc' }, null],
    ['title keeps a bracket tag', { ...good, entries: [{ ...good.entries[0], title: '[큰화]t' }] }, null],
    ['title keeps a carrier label', { ...good, entries: [{ ...good.entries[0], title: 't kt' }] }, null],
    ['title keeps a file version', { ...good, entries: [{ ...good.entries[0], title: 't 01.00.05' }] }, null],
    ['title keeps an edit tag', { ...good, entries: [{ ...good.entries[0], title: 't 2억에디트1' }] }, null],
    ['same title twice on one platform', { ...good, entries: [good.entries[0], { ...good.entries[0], sha256: 'c'.repeat(64) }] }, null],
    ['title keeps an ordinal', { ...good, entries: [{ ...good.entries[0], title: 't (2)' }] }, null],
    ['one game spelled two ways', { ...good, entries: [good.entries[0], { ...good.entries[0], sha256: 'c'.repeat(64), platform: 'SKT', title: 'T', game: 'T' }] }, null],
    ['game dropped', { ...good, entries: [{ ...good.entries[0], game: undefined }] }, null],
    ['representative not first', { ...good, entries: [{ ...pair[0], status: 'not-yet', knownIssues_ko: ['멈춰요.'] }, pair[1]] }, null],
    ['same title, different-game verdict', { ...good, entries: pair.map((x) => ({ ...x, sameTitle: 'different-game' })) }, null],
    ['same title and variant twice', { ...good, entries: pair.map((x) => ({ ...x, variant: '작은화면' })) }, null],
    // A same-game verdict does not open the rule: relabelling the small-screen build «큰화면» next to the real one must fail.
    ['one label on two files of a same-game group', { ...good, entries: pair.map((x) => ({ ...x, sameTitle: 'same-game', variant: '큰화면' })) }, null],
    ['fileTitle dropped', { ...good, entries: [{ ...good.entries[0], fileTitle: undefined }] }, null],
    ['update: title not in compat', good, [['2026-09-27-x.json', { ...upd, titles: ['c'.repeat(64)] }]]],
    ['update: date differs from name', good, [['2026-09-26-x.json', upd]]],
    ['update: bad kind', good, [['2026-09-27-x.json', { ...upd, kind: 'misc' }]]],
    ['update: bad file name', good, [['x.json', upd]]],
    ['update: pr is not a wie PR', good, [['2026-09-27-x.json', { ...upd, pr: 'https://example.com/1' }]]],
    ['update: forbidden punctuation', good, [['2026-09-27-x.json', { ...upd, summary_ko: '위·아래 키' }]]],
    ['update: unknown key', good, [['2026-09-27-x.json', { ...upd, summary: '' }]]],
  ];
  const errs = (c, u) => [...validateCompat(c), ...validateUpdates(u ?? [], new Set((c.entries ?? []).map((x) => x.sha256)))];
  let bad = 0;
  if (errs(good, [['2026-09-27-x.json', upd]]).length) bad++, console.error('selftest: the good fixture is rejected');
  if (validateCompat({ ...good, entries: [{ ...good.entries[0], playTips_ko: ['확인 키로 시작해요.'] }] }).length) bad++, console.error('selftest: a row with playTips_ko is rejected');
  if (validateCompat({ ...good, entries: pair }).length) bad++, console.error('selftest: a judged same-title pair is rejected');
  const re = retitle([{ ...pair[1], title: 'x 1.04', fileTitle: undefined }, { ...pair[0], status: 'limited', knownIssues_ko: ['멈춰요.'], title: 'x', fileTitle: undefined }]);
  if (re.map((x) => `${x.title}|${x.game}|${x.variant}|${x.sameTitle}|${x.sha256[0]}`).join(' ') !== 'x|x|1.04|undetermined|c x|x|null|undetermined|a')
    bad++, console.error(`selftest: retitle must keep the name, move the label to variant and put the representative first, got ${JSON.stringify(re)}`);
  const sp = retitle([
    { ...good.entries[0], title: 'kbo 야구-x', fileTitle: undefined },
    { ...good.entries[0], sha256: 'c'.repeat(64), platform: 'LGT', title: 'KBO야구 x', fileTitle: undefined },
    { ...good.entries[0], sha256: 'd'.repeat(64), platform: 'SKT', title: 'kbo야구x', fileTitle: undefined },
  ]);
  if (sp.some((x) => x.game !== 'KBO야구 x' || x.title !== x.game || x.sameTitle))
    bad++, console.error(`selftest: names equal under gameKey must share the representative spelling on every carrier, got ${JSON.stringify(sp.map((x) => x.game))}`);
  const hy = retitle([{ ...good.entries[0], title: 'a-b', fileTitle: undefined }, { ...good.entries[0], sha256: 'c'.repeat(64), platform: 'SKT', title: 'ab', fileTitle: undefined }]);
  if (hy.some((x) => x.game !== 'ab')) bad++, console.error(`selftest: with capitals and spaces equal the spelling with fewer hyphens wins, got ${JSON.stringify(hy.map((x) => x.game))}`);
  for (const [label, c, u] of cases) if (!errs(c, u).length) bad++, console.error(`selftest: NOT rejected — ${label}`);
  // runtimeAdjustments: derived from the ledger, never typed in; a ledger row needs its basis and a real compat row.
  const lrow = { line: 2, device: 'd', key: '9'.repeat(32), package: sha.slice(0, 12), platform: 'KTF', title: 't', kind: 'owner-file-bytes', changes: 'file-bytes', scope: 'zip', class: 'adopted', basis: 'PR #1' };
  const tsv = (rows) => [LEDGER_COLUMNS, ...rows.map((r) => LEDGER_COLUMNS.map((k) => r[k]))].map((r) => r.join('\t')).join('\n');
  const led = parseLedger(`# a comment\n${tsv([lrow, { ...lrow, key: 'k2', scope: 'any', kind: 'lock-lift' }])}\n`);
  if (led.rows.length !== 2 || validateLedger(led, good.entries).length) bad++, console.error(`selftest: the good ledger is rejected — ${validateLedger(led, good.entries)}`);
  const ledgerCases = [
    ['ledger row with no basis', tsv([{ ...lrow, basis: ' ' }])],
    ['ledger row for a file not in compat', tsv([{ ...lrow, package: 'f'.repeat(12) }])],
    ['ledger row with another title', tsv([{ ...lrow, title: 'u' }])],
    ['ledger row with an unknown kind', tsv([{ ...lrow, kind: 'patch' }])],
    ['ledger row with an unknown scope', tsv([{ ...lrow, scope: 'jar' }])],
    ['ledger row twice', tsv([lrow, lrow])],
    ['ledger row with spaces for tabs', tsv([lrow]).replace(/\t(?=KTF)/, '    ')],
    ['ledger with another header', tsv([lrow]).replace('device', 'table')],
    ['ledger jar key that is not the row\'s contentKey', tsv([{ ...lrow, key: '8'.repeat(32) }])],
  ];
  for (const [label, text] of ledgerCases) if (!validateLedger(parseLedger(text), good.entries).length) bad++, console.error(`selftest: NOT rejected — ${label}`);
  const adj = retitle([{ ...good.entries[0], runtimeAdjustments: ['display-size'] }, { ...good.entries[0], sha256: 'c'.repeat(64), title: 'u', runtimeAdjustments: ['lock-lift'] }], adjustmentsOf(led));
  if (JSON.stringify(adj.map((x) => x.runtimeAdjustments ?? null)) !== '[["lock-lift","owner-file-bytes"],null]')
    bad++, console.error(`selftest: runtimeAdjustments must be the ledger's kinds in vocabulary order, and absent off the ledger — got ${JSON.stringify(adj.map((x) => x.runtimeAdjustments ?? null))}`);
  for (const [label, v] of [['empty runtimeAdjustments', []], ['unknown runtimeAdjustments kind', ['patch']], ['runtimeAdjustments kind twice', ['lock-lift', 'lock-lift']], ['runtimeAdjustments as a string', 'lock-lift']])
    if (!validateCompat({ ...good, entries: [{ ...good.entries[0], runtimeAdjustments: v }] }).length) bad++, console.error(`selftest: NOT rejected — ${label}`);
  // contentKey: a shape, on every committed row, and the group counts the OK line prints.
  for (const [label, v] of [['contentKey without its container', '9'.repeat(32)], ['contentKey in capitals', `zip:${'A'.repeat(32)}`], ['contentKey of a whole-file sha256', `zip:${sha}`], ['empty contentKey', '']])
    if (!validateCompat({ ...good, entries: [{ ...good.entries[0], contentKey: v }] }).length) bad++, console.error(`selftest: NOT rejected — ${label}`);
  if (validateCompat({ ...good, entries: [{ ...good.entries[0], contentKey: null }] }).length || missingContentKeys([{ ...good.entries[0], contentKey: null }]).length)
    bad++, console.error('selftest: a row whose file holds no jar (contentKey null) is rejected');
  const { contentKey: _, ...unkeyed } = good.entries[0];
  if (missingContentKeys(good.entries).length || !missingContentKeys([unkeyed]).length) bad++, console.error('selftest: a row with no contentKey must be reported, and a keyed one must not');
  const groups = contentKeyGroups([...pair, { ...pair[0], sha256: 'd'.repeat(64), contentKey: `zip:${'7'.repeat(32)}` }, { ...pair[0], sha256: 'e'.repeat(64), contentKey: null }]);
  if (groups.distinct !== 2 || groups.shared.length !== 1 || groups.split.length !== 0 || contentKeyGroups([pair[0], { ...pair[1], status: 'limited' }]).split.length !== 1)
    bad++, console.error(`selftest: contentKeyGroups must count keys, shared keys and shared keys with differing status, got ${JSON.stringify(groups)}`);
  if (fromCensus({ ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, render: 'uniform', speed: 'n/a' } }] }).entries[0].axes.render !== 'no')
    bad++, console.error('selftest: census uniform must map to no');
  if (fromCensus({ ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, longplay: 'short' } }] }).entries[0].axes.longplay !== 'unknown')
    bad++, console.error('selftest: census short (the long run ended early) must map to unknown');
  const withProgress = (v) => fromCensus({ ...good, entries: [{ ...good.entries[0], axes: { ...good.entries[0].axes, ...v } }] }).entries[0].axes;
  if (
    withProgress({ progress: 'stuck' }).progress !== 'stuck' ||
    withProgress({ progress: 'error' }).progress !== 'stuck' ||
    'progress' in withProgress({}) ||
    'progress' in withProgress({ progress: 'n/a' })
  )
    bad++, console.error('selftest: progress ships stuck/error as «stuck» (the shell hides «no»), and n/a as no key');
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
    // checkedAt: a row edit made on a branch and merged --no-ff moves «마지막 확인» to the MERGE time
    // (--first-parent; without it git follows the branch and picks the earlier branch commit); a later
    // landing that does not touch compat.json does not; a census newer than every landing wins;
    // build ships it; a shallow clone keeps the census time.
    const at = (d, ...a) => execFileSync('git', ['-c', 'user.name=t', '-c', 'user.email=t@t', ...a], { cwd: repo, env: { ...process.env, GIT_COMMITTER_DATE: d } });
    mkdirSync(join(repo, 'docs/player-data'), { recursive: true });
    writeFileSync(join(repo, 'docs/player-data/compat.json'), '{"generatedAt":"2026-10-04T01:26:49.632Z"}\n');
    at('2026-10-04T12:00:00+09:00', 'add', '-A');
    at('2026-10-04T12:00:00+09:00', 'commit', '-q', '-m', 'census');
    git('checkout', '-q', '-b', 'rows');
    writeFileSync(join(repo, 'docs/player-data/compat.json'), '{"generatedAt":"2026-10-04T01:26:49.632Z","row":1}\n');
    at('2026-10-06T09:00:00+09:00', 'commit', '-q', '-am', 'row edit');
    git('checkout', '-q', 'main');
    at('2026-10-08T21:00:46+09:00', 'merge', '-q', '--no-ff', '-m', 'land rows', 'rows');
    at('2026-10-09T09:00:00+09:00', 'commit', '-q', '--allow-empty', '-m', 'unrelated');
    const c1 = checkedAt('2026-10-04T01:26:49.632Z', repo);
    if (c1.slice(0, 10) !== '2026-10-08') bad++, console.error(`selftest: checkedAt must follow the last compat.json landing (2026-10-08 merge), got ${c1}`);
    const c2 = checkedAt('2026-10-10T00:00:00.000Z', repo);
    if (c2 !== '2026-10-10T00:00:00.000Z') bad++, console.error(`selftest: checkedAt must keep a census newer than every landing, got ${c2}`);
    const b = buildOutput({ ...good, generatedAt: '2026-10-04T01:26:49.632Z' }, [], repo).compat.generatedAt;
    if (b.slice(0, 10) !== '2026-10-08') bad++, console.error(`selftest: build must ship checkedAt as generatedAt (2026-10-08), got ${b}`);
    const shallow = join(repo, 'shallow');
    git('clone', '-q', '--depth', '1', pathToFileURL(repo).href, shallow);
    const c3 = checkedAt('2026-10-04T01:26:49.632Z', shallow);
    if (c3 !== '2026-10-04T01:26:49.632Z') bad++, console.error(`selftest: a shallow clone must keep the census time, got ${c3}`);
    // Run through a symlink, the script must still run: an unknown command is rc=2, not a silent rc=0.
    symlinkSync(fileURLToPath(import.meta.url), join(repo, 'link.mjs'));
    const viaLink = spawnSync(process.execPath, [join(repo, 'link.mjs'), 'no-such-command']).status;
    if (viaLink !== 2) bad++, console.error(`selftest: called through a symlink the script must still run (usage rc=2), got rc=${viaLink}`);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
  if (bad) process.exit(1);
  console.log(`player-data selftest: ${cases.length + ledgerCases.length + 4 + 15 + 7} rules each reject their mutation`);
}

const [cmd, ...args] = process.argv.slice(2);
// Real paths on both sides: called through a symlinked path (macOS /var -> /private/var) the two
// spellings differed, and the script printed nothing and exited 0 — a check that looked green.
const isMain = () => {
  try {
    return realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url));
  } catch {
    return false;
  }
};
if (!isMain());
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
} else if (cmd === 'normalize') {
  // Re-derives title/game/variant/sameTitle from fileTitle and puts each group's representative first.
  const c = JSON.parse(readFileSync(COMPAT, 'utf8'));
  writeFileSync(COMPAT, JSON.stringify({ ...c, entries: retitle(c.entries) }, null, 1) + '\n');
  console.log(`normalized ${c.entries.length} titles -> docs/player-data/compat.json`);
} else if (!cmd || cmd === 'build') {
  const compat = JSON.parse(readFileSync(COMPAT, 'utf8'));
  const files = readUpdates();
  const ledger = readLedger();
  const e = [...validateCompat(compat), ...missingContentKeys(compat.entries), ...validateUpdates(files, new Set(compat.entries.map((x) => x.sha256))), ...validateLedger(ledger, compat.entries)];
  // The committed names and runtimeAdjustments are what retitle() derives, row for row — a hand edit, a naming rule
  // changed without `normalize`, or a ledger row added without it, shows here.
  const drift = e.length ? -1 : retitle(compat.entries, adjustmentsOf(ledger)).findIndex((x, i) => JSON.stringify(x) !== JSON.stringify(compat.entries[i]));
  if (drift >= 0) e.push(`compat.entries[${drift}]: differs from what retitle() derives (title/game/variant/sameTitle, order, runtimeAdjustments from the ledger) — \`node scripts/player-data.mjs normalize\``);
  if (e.length) {
    console.error(`player-data: ${e.length} violations (docs/contracts/featurephone-public-data.md):\n  ${e.join('\n  ')}`);
    process.exit(1);
  }
  const count = (k) => JSON.stringify(compat.entries.reduce((m, x) => ((m[x[k]] = (m[x[k]] ?? 0) + 1), m), {}));
  const adjusted = compat.entries.filter((x) => x.runtimeAdjustments);
  console.log(`player-data OK — ${compat.entries.length} titles · status ${count('status')} · platform ${count('platform')} · ${files.length} updates · runtimeAdjustments on ${adjusted.length} rows (ledger ${ledger.rows.length} rows)`);
  // A key shared by rows of differing status cannot hand a status to a file known only by its key — list them (§1).
  const keys = contentKeyGroups(compat.entries);
  const sha12s = (g) => g.map((x) => `${x.sha256.slice(0, 12)}=${x.status}`).join('/');
  console.log(`contentKey — ${keys.distinct} distinct on ${compat.entries.filter((x) => x.contentKey).length} rows · ${keys.shared.length} shared by ${keys.shared.flat().length} rows · ${keys.split.length} shared with differing status${keys.split.length ? `: ${keys.split.map(sha12s).join(' · ')}` : ''}`);
  if (cmd === 'build') {
    const oi = args.indexOf('--out');
    if (oi < 0) process.exit(2);
    const out = buildOutput(compat, files);
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
  console.error('usage: player-data.mjs [--selftest | import <census compat.json> | normalize | build --out <dir>]');
  process.exit(2);
}
