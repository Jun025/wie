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

import { readdirSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = join(dirname(fileURLToPath(import.meta.url)), '..', 'docs', 'worklog');
// The seven strings scanRepoSimple pulls out of each proposals[] element.
const KEYS = ['title', 'plainSummary', 'userBenefit', 'why', 'tradeoff', 'effort', 'target'];
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
    });

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
console.log(`worklog .json contract: OK (${files.length} file(s) in docs/worklog/)`);
