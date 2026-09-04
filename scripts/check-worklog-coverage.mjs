// Enforces the *promise* AGENTS.md §Landing paperwork made — "re-measure every
// 10 landed rounds" — not the threshold judgment itself.
//
// The distinction is the whole design. Coverage below 70% is a signal to REOPEN
// a decision, and reopening is a human call: the metric has known, legitimate
// misses baked in (a round that leaves no follow-ups owes no worklog and still
// counts against it; an upstream-sync merge lands as one round with no worklog
// at all). Failing CI on the ratio would gate PRs on a *conditional* obligation
// — exactly the mandate 2026-09-01 declined. Being overdue, by contrast, is not
// a judgment: it is a fact, and it has no false positives.
//
// So this script fails on two things only:
//   OVERDUE            — 10+ landed rounds since the last recorded measurement
//   BELOW-UNANSWERED   — the last recorded measurement is under the threshold
//                        and nobody has recorded that the decision was reopened
// and otherwise prints today's numbers so recording one is a copy-paste.
//
// The record lives in docs/worklog-coverage-remeasures.json. Clearing OVERDUE
// means appending the entry this script prints — that IS the re-measurement.
//
// This file is the single source of the measurement. AGENTS.md keeps the *why*
// (the threshold, the cadence, why --first-parent is load-bearing) and points
// here for the *how*; two copies of the commands would drift.
//
// Usage: node scripts/check-worklog-coverage.mjs

import { readFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// Values owned by AGENTS.md §Landing paperwork — do not retune them here.
const SINCE = "92c25276"; // the commit that landed the worklog convention
const WINDOW = 10; // rounds per measurement window, and the re-measure cadence
const THRESHOLD = 70; // percent; below this the mandate decision reopens

const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();

// A landed round is one first-parent commit on the default branch. `origin/main`
// is not always present (a plain clone of a fork checkout may only have `main`),
// so fall back rather than fail on a naming detail.
let head = "origin/main";
try {
  git("rev-parse", "--verify", "--quiet", head);
} catch {
  head = "HEAD";
}

// Shallow clones silently truncate history, which would make every count a lie.
// Refuse rather than report a number nobody can trust.
if (git("rev-parse", "--is-shallow-repository") === "true") {
  console.log("worklog coverage: UNMEASURABLE — shallow clone (need full history back to " + SINCE + ")");
  console.log("  CI: give the checkout step `fetch-depth: 0`.  Local: `git fetch --unshallow`.");
  process.exit(1);
}

const landed = Number(git("rev-list", "--count", "--first-parent", `${SINCE}..${head}`));
const oldest = git("log", "--first-parent", "--format=%h", "-n", String(WINDOW), `${SINCE}..${head}`).split("\n").at(-1);
const den = Number(git("rev-list", "--count", "--first-parent", `${oldest}^..${head}`));
const num = Number(git("rev-list", "--count", "--first-parent", `${oldest}^..${head}`, "--", "docs/worklog"));
const pct = den === 0 ? 100 : (100 * num) / den;
const newest = git("log", "--first-parent", "--format=%h", "-n", "1", head);

const recordPath = "docs/worklog-coverage-remeasures.json";
const record = JSON.parse(await readFile(path.join(root, recordPath), "utf8"));
const last = record.measurements.at(-1);

console.log(`worklog coverage: ${num}/${den} = ${pct.toFixed(1)}% over the last ${den} landed round(s)`);
console.log(`  window ${oldest}..${newest} · landed rounds since ${SINCE}: ${landed} · last recorded at: ${last.landedRounds}`);
console.log(`  entry for ${recordPath}:`);
console.log(
  "  " +
    JSON.stringify({
      // Local date, to match how every other dated artifact in this repo is stamped.
      date: new Date().toLocaleDateString("en-CA"),
      landedRounds: landed,
      window: `${oldest}..${newest}`,
      num,
      den,
      pct: Number(pct.toFixed(1)),
    }),
);

const problems = [];
if (landed - last.landedRounds >= WINDOW) {
  problems.push(
    `OVERDUE: ${landed - last.landedRounds} landed rounds since the last recorded measurement (cadence is ${WINDOW}). ` +
      `Append the entry printed above to ${recordPath} — that is the re-measurement AGENTS.md promised.`,
  );
}
if (last.pct < THRESHOLD && last.reopened !== true) {
  problems.push(
    `BELOW-UNANSWERED: the last recorded measurement is ${last.pct}% (< ${THRESHOLD}%), and AGENTS.md says that reopens the ` +
      `per-round-mandate decision. Record the outcome by setting "reopened": true on that entry (with a "decision" note).`,
  );
}

if (problems.length > 0) {
  for (const p of problems) console.log(`  ✗ ${p}`);
  process.exit(1);
}
console.log("OK — the re-measure promise is current.");
