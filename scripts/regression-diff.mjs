#!/usr/bin/env node
// Weekly regression watch — the comparison half. `scripts/regression-census.sh` measures; this file
// only reads two or three `compat.json` and says which rows went DOWN. Local only, like the census.
//
//   node scripts/regression-diff.mjs changed --base <compat> --new <compat>          sha12 per line: probe axes moved
//   node scripts/regression-diff.mjs candidates --base <compat> --new <compat> --long <titles>
//        --population <out/population.json> --pair-corpus <dir> --pair-long <file>  links the downgraded rows for the pair
//   node scripts/regression-diff.mjs report --base <compat> --new <compat> --long <titles> [--pair <compat> --pair-long <file>]
//        --pin <sha> --tool-pin <sha> --elapsed <secs> --date <YYYY-MM-DD> --out <md>                   prints the suspect count
//   node scripts/regression-diff.mjs selftest
//
// Either vocabulary reads: the census's (`ok`/`n/a`/`none`/…, scripts/playability-census.mjs) and the
// shipped one (`ok`/`no`/`unknown`, docs/player-data/compat.json) — mapped as scripts/player-data.mjs does.
// A row goes DOWN on an axis only from a measured value to a worse measured one; `unknown` is never a side.
//   boot · render · input   every row (the 30 s probes A·B run on every title).
//   longplay                only rows in --long whose 600 s run happened this round.
//   sound                   --long rows both ways; elsewhere only UP — a probe that heard nothing says
//                           nothing about a title whose music starts past the title screen (census header).
//   speed                   both ways, but headless never says `slow`, so in practice only `ok` -> `unknown`, ignored.
//   status                  playable > limited > not-yet. A row outside --long reads `limited` for want of a
//                           long run; when that is all it lacks it keeps its base status (lifted()).
//   progress                not measured here (opt-in census axis, 30 min a title) — never compared.
// The compat is NEVER written: an UP is a note, a DOWN is a candidate, and a candidate the pair (same
// binary, re-measured once) shows again is a «suspect».
import { lstatSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { extname, join } from 'node:path';

const MAP = { 'n/a': 'unknown', fail: 'no', none: 'no', uniform: 'no', error: 'no', silent: 'no', slow: 'no', stall: 'no' };
const norm = (v) => MAP[v] ?? v ?? 'unknown';
const RANK = { ok: 2, partial: 1, no: 0 };
const STATUS = { playable: 2, limited: 1, 'not-yet': 0 };
const PROBE_AXES = ['boot', 'render', 'input'];
const AXES = [...PROBE_AXES, 'longplay', 'sound', 'speed'];
// A NEW wall (server, purchase check, lock) caps a row whatever its axes say — keep its status as measured.
// One base already carried is a note on a row that stayed playable (1a69522a7d43's «온라인 등록»).
// ponytail: matched on the shipped sentence; a census constant to import is the upgrade if a wall is reworded.
const WALL = /서버|인증|잠긴/;

const rows = (c) => new Map(c.entries.map((e) => [e.sha256, e]));
const statusOf = (ax) =>
  ax.boot === 'ok' && ax.render === 'ok' && ax.input === 'ok' && ax.longplay === 'ok' ? 'playable' : ax.boot === 'ok' && ax.render === 'ok' ? 'limited' : 'not-yet';
// Status of a row outside --long: `limited` only because longplay was not run this round keeps base's.
function lifted(b, n) {
  const newWall = (n.knownIssues_ko ?? []).some((s) => WALL.test(s) && !(b.knownIssues_ko ?? []).includes(s));
  if (n.status !== 'limited' || b.status !== 'playable' || newWall) return n.status;
  const ax = Object.fromEntries(AXES.map((a) => [a, norm(n.axes[a])]));
  return statusOf({ ...ax, longplay: norm(b.axes.longplay) });
}

// One row, base -> new: [{ axis, before, after, dir }] with dir 'down' | 'up'.
function rowDrift(b, n, long) {
  const out = [];
  const step = (axis, before, after, rank, upOnly = false) => {
    if (!(before in rank) || !(after in rank) || rank[before] === rank[after]) return;
    const dir = rank[after] < rank[before] ? 'down' : 'up';
    if (dir === 'down' && upOnly) return;
    out.push({ axis, before, after, dir });
  };
  for (const a of AXES) {
    if (a === 'longplay' && !long) continue;
    step(a, norm(b.axes[a]), norm(n.axes[a]), RANK, a === 'sound' && !long);
  }
  step('status', b.status, long ? n.status : lifted(b, n), STATUS);
  return out;
}

export function drift(base, cur, long = new Set()) {
  const B = rows(base);
  const down = [];
  const up = [];
  const unmeasured = [];
  for (const [sha, b] of B) {
    const n = rows(cur).get(sha);
    if (!n) {
      unmeasured.push(sha.slice(0, 12));
      continue;
    }
    // In --long and the run happened (a title whose input is not ok never gets one).
    const ran = long.has(sha.slice(0, 12)) && norm(n.axes.longplay) !== 'unknown';
    const rowStatus = `${b.status} → ${n.status}`;
    for (const d of rowDrift(b, n, ran)) (d.dir === 'down' ? down : up).push({ sha12: sha.slice(0, 12), rowStatus, ...d });
  }
  const added = [...rows(cur).keys()].filter((s) => !B.has(s)).map((s) => s.slice(0, 12));
  return { down, up, unmeasured, added };
}

// Rows whose probe axes differ from the previous round: they get this round's long run.
export const changed = (prev, cur) => {
  const P = rows(prev);
  return [...rows(cur)].filter(([s, n]) => P.has(s) && PROBE_AXES.some((a) => norm(P.get(s).axes[a]) !== norm(n.axes[a]))).map(([s]) => s.slice(0, 12));
};

// A candidate stands as a suspect when the pair shows the same row going down on the same axis.
export const confirm = (cands, pairDown) => cands.map((c) => ({ ...c, pair: pairDown.find((p) => p.sha12 === c.sha12 && p.axis === c.axis) ?? null }));
// The pair needs the long run when the downgrade is one only a long run can show.
const suspectRows = (cands) => new Set(cands.filter((c) => c.pair).map((c) => c.sha12)).size;
const needsLong = (c, long) => long.has(c.sha12) && ['longplay', 'sound', 'status'].includes(c.axis);

const sha12s = (file) => new Set(readFileSync(file, 'utf8').split('\n').map((l) => l.split('#')[0].trim().split(/\s+/)[0]).filter(Boolean).map((s) => s.slice(0, 12)));
const readJson = (p) => JSON.parse(readFileSync(p, 'utf8'));

function markdown({ base, cur, long, res, pin, toolPin, elapsed, date }) {
  const sus = suspectRows(res.cands);
  const t = (xs, head, f) => (xs.length ? [head, `|${'---|'.repeat(head.split('|').length - 2)}`, ...xs.map(f)] : ['(없음)']);
  return [
    `# wie 주간 퇴행 감시 — ${date}`,
    '',
    `- 측정 바이너리 핀: \`${pin}\` (origin/main · release \`wie_validate\`)`,
    `- 도구 핀: \`${toolPin}\` · 대조 기준: \`docs/player-data/compat.json\` (enginePin \`${base.enginePin}\` · ${base.entries.length}행)`,
    `- 걸린 시간: ${elapsed}s (${(elapsed / 3600).toFixed(2)}h) · 이번 회차 측정 행 ${cur.entries.length} · 미측 ${res.unmeasured.length} · 장주행(L 600s) 대상 ${long.size}행`,
    `- ★퇴행 의심(짝 재현) **${sus}행** · 내림 후보 ${new Set(res.cands.map((c) => c.sha12)).size}행(${res.cands.length}건) · 올림(참고) ${res.up.length} — compat 는 고치지 않았다`,
    '',
    '## 퇴행 의심 · 내림 후보 (sha12 · 축 · 전 → 후 · 짝 재측 · 재현)',
    '',
    ...t(res.cands, '| sha12 | 축 | 전 | 후 | 짝 | 재현 | 행 상태(compat → 이번) |', (c) => `| \`${c.sha12}\` | ${c.axis} | ${c.before} | ${c.after} | ${c.pair?.after ?? (res.paired ? '같지 않음' : '미측')} | ${c.pair ? '**재현**' : '아니오'} | ${c.rowStatus} |`),
    '',
    ...(sus
      ? ['다음 행동: 의심 행마다 compat 핀과 이번 핀 사이를 같은 시각 짝으로 좁혀(`wie_validate` 전/후 바이너리) 원인 PR 을 찾고 티켓을 연다 — compat 행은 그 티켓이 고친다.', '']
      : []),
    '## 올림 (참고 — compat 는 자동으로 고치지 않는다)',
    '',
    ...t(res.up, '| sha12 | 축 | 전 | 후 |', (c) => `| \`${c.sha12}\` | ${c.axis} | ${c.before} | ${c.after} |`),
    '',
    `## 미측 행 (${res.unmeasured.length} — 굶은 프로브 등, 판정 없음)`,
    '',
    res.unmeasured.length ? res.unmeasured.map((s) => `\`${s}\``).join(' ') : '(없음)',
    ...(res.added.length ? ['', `## compat 에 없는 행 (${res.added.length})`, '', res.added.map((s) => `\`${s}\``).join(' ')] : []),
    '',
  ].join('\n');
}

const [cmd, ...rest] = process.argv.slice(2);
const opt = {};
for (let i = 0; i < rest.length; i += 2) opt[rest[i].replace(/^--/, '')] = rest[i + 1];

if (cmd === 'changed') {
  for (const s of changed(readJson(opt.base), readJson(opt.new))) console.log(s);
} else if (cmd === 'candidates') {
  const long = sha12s(opt.long);
  const { down } = drift(readJson(opt.base), readJson(opt.new), long);
  const paths = new Map(readJson(opt.population).titles.map((t) => [t.sha.slice(0, 12), t.path]));
  rmSync(opt['pair-corpus'], { recursive: true, force: true });
  mkdirSync(opt['pair-corpus'], { recursive: true });
  const linked = new Set();
  for (const c of down) {
    if (linked.has(c.sha12)) continue;
    linked.add(c.sha12);
    // Named by sha12, so the pair corpus spells no title.
    const p = paths.get(c.sha12);
    const link = join(opt['pair-corpus'], c.sha12 + extname(p));
    if (!lstatSync(link, { throwIfNoEntry: false })) symlinkSync(p, link);
  }
  writeFileSync(opt['pair-long'], [...new Set(down.filter((c) => needsLong(c, long)).map((c) => c.sha12))].map((s) => s + '\n').join(''));
  console.log(linked.size);
} else if (cmd === 'report') {
  const base = readJson(opt.base);
  const cur = readJson(opt.new);
  const long = sha12s(opt.long);
  const d = drift(base, cur, long);
  const pairDown = opt.pair ? drift(base, readJson(opt.pair), sha12s(opt['pair-long'])).down : [];
  const res = { ...d, cands: confirm(d.down, pairDown), paired: Boolean(opt.pair) };
  writeFileSync(opt.out, markdown({ base, cur, long, res, pin: opt.pin, toolPin: opt['tool-pin'], elapsed: Number(opt.elapsed), date: opt.date }));
  console.log(suspectRows(res.cands));
} else if (cmd === 'selftest') {
  const ax = (o = {}) => ({ boot: 'ok', render: 'ok', input: 'ok', longplay: 'ok', sound: 'ok', speed: 'ok', ...o });
  const row = (n, status, axes, extra = {}) => ({ sha256: String(n).repeat(64).slice(0, 64), status, axes, knownIssues_ko: [], ...extra });
  // Base in the shipped vocabulary, «new» in the census's — the two a real round compares.
  const base = { enginePin: 'x', entries: [row(1, 'playable', ax()), row(2, 'limited', ax({ render: 'no', input: 'unknown', longplay: 'unknown' })), row(3, 'playable', ax())] };
  const cur = { entries: [row(1, 'limited', ax({ input: 'none', longplay: 'n/a' })), row(2, 'limited', ax({ longplay: 'n/a' })), row(3, 'limited', ax({ longplay: 'n/a', sound: 'silent', speed: 'n/a' }))] };
  const d = drift(base, cur);
  const k = (xs) => xs.map((x) => `${x.sha12.slice(0, 1)}:${x.axis}`).sort().join(' ');
  const L1 = new Set(['111111111111']);
  const cases = [
    ['one row down: input ok -> none, and its status with it', k(d.down) === '1:input 1:status'],
    ['one row up: render no -> ok (a note, not a candidate)', k(d.up) === '2:render'],
    ['an unchanged row with no long run this round is unchanged', !d.down.concat(d.up).some((x) => x.sha12.startsWith('3'))],
    ['a long-run row: longplay error is down', k(drift(base, { entries: [row(1, 'limited', ax({ longplay: 'error' }))] }, L1).down) === '1:longplay 1:status'],
    ['a long row whose long run never happened: silent is not down', drift(base, { entries: [row(1, 'limited', ax({ input: 'none', longplay: 'n/a', sound: 'silent' }))] }, L1).down.every((x) => x.axis !== 'sound')],
    ['suspects count rows, not axes', suspectRows(confirm(d.down, d.down)) === 1],
    ['a long-run row: silent is down', k(drift(base, { entries: [row(1, 'playable', ax({ sound: 'silent' }))] }, L1).down) === '1:sound'],
    ['a wall keeps the measured status', k(drift(base, { entries: [row(3, 'limited', ax({ longplay: 'n/a' }), { knownIssues_ko: ['그 서버가 지금은 없어'] })] }).down) === '3:status'],
    ['a wall line base already carried does not cap', (() => {
      const note = { knownIssues_ko: ['랭킹은 게임사 서버가 지금은 없어 쓸 수 없어요.'] };
      return drift({ entries: [row(3, 'playable', ax(), note)] }, { entries: [row(3, 'limited', ax({ longplay: 'n/a' }), note)] }).down.length === 0;
    })()],
    ['n/a is never a side', drift({ entries: [row(1, 'playable', ax({ speed: 'unknown' }))] }, { entries: [row(1, 'playable', ax({ speed: 'n/a' }))] }, L1).down.length === 0],
    ['a row missing from the round is unmeasured, not down', drift(base, { entries: [] }).unmeasured.length === 3 && drift(base, { entries: [] }).down.length === 0],
    ['changed: probe axes only', changed(base, cur).join() === '111111111111,222222222222'],
    ['the pair confirms the same axis only', confirm(d.down, [{ sha12: '111111111111', axis: 'input', after: 'no' }]).filter((c) => c.pair).length === 1],
    ['a status-only downgrade needs the long run in the pair only for a long row', needsLong({ sha12: '111111111111', axis: 'status' }, L1) && !needsLong({ sha12: '111111111111', axis: 'input' }, L1)],
  ];
  const bad = cases.filter(([, ok]) => !ok);
  for (const [n] of bad) console.error(`selftest FAIL: ${n}`);
  console.log(`selftest: ${cases.length - bad.length}/${cases.length}`);
  process.exit(bad.length ? 1 : 0);
} else {
  console.error('usage: regression-diff.mjs changed|candidates|report|selftest … (see the header)');
  process.exit(2);
}
