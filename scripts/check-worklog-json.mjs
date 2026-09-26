#!/usr/bin/env node
// Locks the worklog `.json` convention (AGENTS.md §Landing paperwork).
//
// Why: the cockpit 「후속 작업 추천」 panel reads `docs/worklog/*.json` and nothing else
// (`/api/proposals` → `scanRepoSimple`). A malformed or key-short file is SILENT — it just
// produces zero cards, or a card with blank fields, and nobody finds out. This turns that
// silence red. Measured 2026-08-26: wie was `json:1 · proposals:0` while otterpebble was 824.
//
// Scope: every `.json` in docs/worklog/, checked against the keys the consumer actually reads.
// Other keys are free-form and are NOT policed here. No `.md` sibling is required — wie's
// worklogs are `.json`-only (that is also dodu's shape: md:0), so requiring one would mean
// retroactively converting history, which this convention explicitly does not do.
//
// Two rules added 2026-09-26 (ticket wie-worklog-proposal-threshold-and-kind — AGENTS.md
// §Proposal threshold):
//   ⒜ `proposals[].kind`, when present, is "product" or "meta". Absent = unclassified (fine).
//   ⒝ a worklog this branch ADDS, or a changed one whose proposal count GREW, may carry at most
//      MAX_PROPOSALS. Measured against the merge-base with origin/main, so the 41 existing
//      worklogs with 3+ proposals stay green and a disposition-only edit to one of them does too.
//      If origin/main is missing, ⒝ is NOT measured and the success line says so.
//
//   node scripts/check-worklog-json.mjs              # the gate
//   node scripts/check-worklog-json.mjs --selftest   # discrimination, through the CLI itself

import { readdirSync, readFileSync, writeFileSync, mkdirSync, mkdtempSync, rmSync } from 'node:fs';
import { execFileSync, spawnSync } from 'node:child_process';
import { join, dirname } from 'node:path';
import { tmpdir } from 'node:os';
import { fileURLToPath } from 'node:url';

const SELF = fileURLToPath(import.meta.url);
const ROOT = process.env.WORKLOG_JSON_ROOT || join(dirname(SELF), '..');
const REL = 'docs/worklog';
const dir = join(ROOT, REL);
// The seven strings scanRepoSimple pulls out of each proposals[] element.
const KEYS = ['title', 'plainSummary', 'userBenefit', 'why', 'tradeoff', 'effort', 'target'];
const KINDS = ['product', 'meta'];
const MAX_PROPOSALS = 2;

if (process.argv.includes('--selftest')) selftest();

const git = (args) => execFileSync('git', args, { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });

// Worklogs this tree changed vs the merge-base: name -> base proposal count (null = added).
// Compares against the WORK TREE, and includes untracked files, so it answers before a commit too.
function changedWorklogs() {
  try {
    const mb = git(['merge-base', 'origin/main', 'HEAD']).trim();
    const changed = new Map();
    for (const line of git(['diff', '--name-status', '--diff-filter=AM', mb, '--', REL]).split('\n')) {
      const [st, path] = line.split('\t');
      if (!path || !path.endsWith('.json')) continue;
      let base = null;
      if (st === 'M') {
        try {
          const p = JSON.parse(git(['show', `${mb}:${path}`])).proposals;
          base = Array.isArray(p) ? p.length : 0;
        } catch {
          base = 0;
        }
      }
      changed.set(path.slice(REL.length + 1), base);
    }
    for (const path of git(['ls-files', '--others', '--exclude-standard', '--', REL]).split('\n'))
      if (path.endsWith('.json')) changed.set(path.slice(REL.length + 1), null);
    return changed;
  } catch (e) {
    return { error: (e.stderr || e.message || '').toString().trim().split('\n')[0] };
  }
}

const changed = changedWorklogs();
const measured = changed instanceof Map;
const violations = [];

const files = readdirSync(dir).filter((f) => f.endsWith('.json')).sort();
for (const f of files) {
  let j;
  try {
    j = JSON.parse(readFileSync(join(dir, f), 'utf8'));
  } catch (e) {
    violations.push(`${f}: JSON parse failed — the consumer skips the whole file (${e.message})`);
    continue;
  }
  if (j === null || typeof j !== 'object' || Array.isArray(j)) {
    violations.push(`${f}: top level is not an object`);
    continue;
  }
  if (!/^\d{4}-\d{2}-\d{2}/.test(f)) violations.push(`${f}: filename does not start with YYYY-MM-DD`);
  else if (j.date !== f.slice(0, 10)) violations.push(`${f}: date (${j.date}) != filename date (${f.slice(0, 10)}) — the sort would lie`);

  const proposals = j.proposals ?? [];
  if (!Array.isArray(proposals)) violations.push(`${f}: proposals is not an array`);
  else
    proposals.forEach((p, i) => {
      if (p === null || typeof p !== 'object' || Array.isArray(p)) {
        violations.push(`${f}#p${i}: element is not an object — the consumer skips it`);
        return;
      }
      for (const k of KEYS)
        if (typeof p[k] !== 'string' || !p[k].trim())
          violations.push(`${f}#p${i}: '${k}' missing — that field renders blank on the card`);
      if ('kind' in p && !KINDS.includes(p.kind))
        violations.push(`${f}#p${i}: kind ${JSON.stringify(p.kind)} — must be "product" or "meta", or omitted`);
    });

  if (measured && changed.has(f) && Array.isArray(proposals) && proposals.length > MAX_PROPOSALS) {
    const base = changed.get(f);
    if (base === null || proposals.length > base)
      violations.push(
        `${f}: ${proposals.length} proposals (${base === null ? 'new file' : `was ${base}`}) — at most ${MAX_PROPOSALS} per worklog (AGENTS.md §Proposal threshold; 0 is the normal value)`,
      );
  }

  for (const key of ['adoptedProposals', 'declinedProposals']) {
    const refs = j[key] ?? [];
    if (!Array.isArray(refs)) {
      violations.push(`${f}: ${key} is not an array`);
      continue;
    }
    for (const r of refs)
      if (typeof r !== 'string' || !r.includes('#p'))
        violations.push(`${f}: ${key} entry ${JSON.stringify(r)} — must be '<basename>#p<index>' or the disposition does not apply`);
  }
}

if (violations.length) {
  console.error(`worklog .json contract: ${violations.length} violation(s) across ${files.length} file(s)`);
  for (const v of violations) console.error(`  - ${v}`);
  process.exit(1);
}
const cap = measured
  ? `proposal cap checked on ${changed.size} changed file(s)`
  : `★proposal cap NOT measured — no merge-base with origin/main (${changed.error})`;
console.log(`worklog .json contract: OK (${files.length} file(s) in docs/worklog/ · ${cap})`);

// ── selftest: every case runs THIS script as a CLI against a throwaway git repo ──────────────
function selftest() {
  const prop = (extra = {}) => ({ title: 't', plainSummary: 's', userBenefit: 'u', why: 'w', tradeoff: 'o', effort: 'S', target: 'wie', ...extra });
  const wl = (date, n, extra = {}) => ({ date, proposals: Array.from({ length: n }, () => prop()), ...extra });
  const cases = [
    // [name, base files, head files (written after the base commit), expected rc]
    ['past worklog with 3 proposals, unchanged → green', { '2026-01-01-old.json': wl('2026-01-01', 3) }, {}, 0],
    ['new worklog with 3 proposals → red', {}, { '2026-01-02-new.json': wl('2026-01-02', 3) }, 1],
    ['new worklog with 2 proposals (product+meta) → green', {}, { '2026-01-02-new.json': { date: '2026-01-02', proposals: [prop({ kind: 'product' }), prop({ kind: 'meta' })] } }, 0],
    ['kind "x" → red', {}, { '2026-01-02-new.json': { date: '2026-01-02', proposals: [prop({ kind: 'x' })] } }, 1],
    ['kind "x" in a PAST file → red (kind is checked everywhere)', { '2026-01-01-old.json': { date: '2026-01-01', proposals: [prop({ kind: 'x' })] } }, {}, 1],
    ['disposition-only edit to a past 3-proposal worklog → green', { '2026-01-01-old.json': wl('2026-01-01', 3) }, { '2026-01-01-old.json': wl('2026-01-01', 3, { declinedProposals: ['2026-01-01-old#p0'] }) }, 0],
    ['past worklog grown 3 → 4 → red', { '2026-01-01-old.json': wl('2026-01-01', 3) }, { '2026-01-01-old.json': wl('2026-01-01', 4) }, 1],
  ];
  let bad = 0;
  for (const [name, base, head, want] of cases) {
    const tmp = mkdtempSync(join(tmpdir(), 'wl-selftest-'));
    try {
      const g = (...a) => execFileSync('git', ['-c', 'user.name=t', '-c', 'user.email=t@t', ...a], { cwd: tmp, stdio: 'ignore' });
      const put = (files) => { for (const [n, j] of Object.entries(files)) writeFileSync(join(tmp, REL, n), JSON.stringify(j)); };
      mkdirSync(join(tmp, REL), { recursive: true });
      g('init', '-q');
      writeFileSync(join(tmp, 'README'), 'x');
      put(base);
      g('add', '-A');
      g('commit', '-qm', 'base');
      g('update-ref', 'refs/remotes/origin/main', 'HEAD');
      put(head);
      const r = spawnSync(process.execPath, [SELF], { env: { ...process.env, WORKLOG_JSON_ROOT: tmp }, encoding: 'utf8' });
      const ok = r.status === want && (want !== 0 || /proposal cap checked/.test(r.stdout));
      if (!ok) bad++;
      console.log(`${ok ? 'selftest PASS' : 'selftest FAIL'} — ${name} (rc=${r.status}, want ${want})`);
      if (!ok) console.log((r.stdout + r.stderr).replace(/^/gm, '    '));
    } finally {
      rmSync(tmp, { recursive: true, force: true });
    }
  }
  console.log(`\ncheck-worklog-json --selftest: ${cases.length} case(s)${bad ? `, ${bad} FAILED` : ', all as expected'}`);
  process.exit(bad ? 1 : 0);
}
