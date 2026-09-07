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
// ── Append with --record, not by hand, and that is a correctness rule ────────
// The obligation is keyed to `origin/main`, so once the cadence is crossed EVERY
// round that pulls base gets the same rc=1 and every one of them handles it
// honestly. Measured 2026-09-06: three rounds each wrote the SAME entry (date,
// landedRounds, window, num, den, pct identical; only `decision` differed) and a
// human had to intervene twice to stop two of them from landing. Nobody was
// wrong — the obligation's shape produced the duplicates.
//
// `--record` is the idempotent way to discharge it: it scans the WHOLE record
// for the same `landedRounds` and, if it is already there, changes nothing and
// exits 0. So a round whose base already carries the entry writes nothing, and
// running it twice is the same as running it once. It also refuses to write when
// the cadence is not yet due, so it cannot invent an off-schedule row.
//
// What it does NOT fix, stated so nobody assumes otherwise: two branches that
// both append BEFORE either lands still produce two rows, because each tree is
// individually correct. That residual is about the checker's tolerance, not the
// recorder's.
//
// ── Which row is "the last recorded measurement" — highest, not last-appended ─
// It reads the row with the largest `landedRounds`, not `at(-1)`. Those differ
// only when rows arrive out of order, which is exactly what the residual above
// produces: two branches append independently and the merge decides the order.
// Under `at(-1)`, `[…, 50, 40]` reports "last recorded at 40" and goes OVERDUE
// even though a measurement at 50 is sitting right there — a false red, because
// the promise is "re-measure every 10 landed rounds", not "append in order".
//
// **Not a relaxation, and that distinction was measured, not assumed** (all three
// re-runnable by editing a copy of the record and running this script):
//   [ …, 50, 40 ]  order-reversed, same set   at(-1) rc=1  ->  max rc=0   fixed
//   [ …, 40, 50 ]  in order, same set         at(-1) rc=0  ->  max rc=0   unchanged
//   [ …, 40, 45 ]  in order, genuinely due    at(-1) rc=1  ->  max rc=1   STILL RED
// The last line is the one that matters: `max` moves the out-of-order case and
// nothing else. It also makes BELOW-UNANSWERED stricter rather than looser — that
// check reads the same row, so a later-appended *older* entry can no longer mask a
// sub-threshold measurement.
//
// Ties (two rows with the same `landedRounds`) resolve to the later-appended one:
// same number, so the newer statement about that round wins. That case is real —
// measured 2026-09-07, commits `80809604` and `ed70b279` each appended
// `landedRounds: 53` from separate branches before either landed, and they merged
// into one row only because the two additions were byte-identical.
//
// This file is the single source of the measurement. AGENTS.md keeps the *why*
// (the threshold, the cadence, why --first-parent is load-bearing) and points
// here for the *how*; two copies of the commands would drift.
//
// Usage: node scripts/check-worklog-coverage.mjs            (check only; what CI runs)
//        node scripts/check-worklog-coverage.mjs --record   (idempotently append when due)

import { readFile, writeFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// Values owned by AGENTS.md §Landing paperwork — do not retune them here.
const SINCE = "92c25276"; // the commit that landed the worklog convention
const WINDOW = 10; // rounds per measurement window, and the re-measure cadence
const THRESHOLD = 70; // percent; below this the mandate decision reopens

const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();

const RECORD = process.argv.includes("--record");

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
// Highest `landedRounds`, ties to the later-appended row — see the header for why
// this is not `at(-1)` and what it does and does not change.
const last = record.measurements.reduce((a, b) => (b.landedRounds >= a.landedRounds ? b : a));

console.log(`worklog coverage: ${num}/${den} = ${pct.toFixed(1)}% over the last ${den} landed round(s)`);
console.log(`  window ${oldest}..${newest} · landed rounds since ${SINCE}: ${landed} · last recorded at: ${last.landedRounds}`);
// Hoisted so --record writes the SAME object the check prints — one source, so
// the printed line and the appended row cannot drift.
const entry = {
  // Local date, to match how every other dated artifact in this repo is stamped.
  date: new Date().toLocaleDateString("en-CA"),
  landedRounds: landed,
  window: `${oldest}..${newest}`,
  num,
  den,
  pct: Number(pct.toFixed(1)),
};
console.log(`  entry for ${recordPath}:`);
console.log("  " + JSON.stringify(entry));

const due = landed - last.landedRounds >= WINDOW;

// --record: discharge the obligation idempotently. Two guards, and both matter.
//   1. Not due -> write nothing. Keeps the recorder from inventing off-schedule
//      rows just because someone ran it.
//   2. Already recorded -> write nothing. The scan is over the WHOLE array, not
//      just the last entry, so a round whose base already carries the row (from
//      a sibling that landed first) is a no-op rather than a duplicate.
// The append preserves the file's own shape: 2-space JSON, trailing newline,
// entries appended in order — this file is append-only evidence, so nothing
// existing is ever rewritten.
if (RECORD) {
  const already = record.measurements.find((m) => m.landedRounds === landed);
  if (already) {
    console.log(`  --record: landedRounds ${landed} is already recorded (window ${already.window}) — nothing to do.`);
  } else if (!due) {
    console.log(`  --record: not due (${landed - last.landedRounds} landed since ${last.landedRounds}, cadence ${WINDOW}) — nothing to do.`);
  } else {
    const decisionArg = process.argv[process.argv.indexOf("--decision") + 1];
    record.measurements.push({
      ...entry,
      decision:
        process.argv.includes("--decision") && decisionArg
          ? decisionArg
          : `임계 이상 여부는 pct 가 말한다(${entry.pct}%). 착지 ${landed} ≥ ${last.landedRounds}+${WINDOW} 로 기한이 차서 ` +
            `\`--record\` 로 기록했다 — 손으로 붙이면 회차마다 중복이 난다(2026-09-06 실측 3건).`,
    });
    await writeFile(path.join(root, recordPath), `${JSON.stringify(record, null, 2)}\n`);
    console.log(`  --record: appended landedRounds ${landed} to ${recordPath}.`);
  }
}

const problems = [];
if (due && !(RECORD && record.measurements.some((m) => m.landedRounds === landed))) {
  problems.push(
    `OVERDUE: ${landed - last.landedRounds} landed rounds since the last recorded measurement (cadence is ${WINDOW}). ` +
      `Run \`node scripts/check-worklog-coverage.mjs --record\` — it appends the entry printed above, idempotently. ` +
      `Do NOT hand-append: every round that pulls base gets this same rc=1, and hand-appending produced three identical rows on 2026-09-06. ` +
      `OWNER: the gate3 round — bundle that one file into the PR before merging (AGENTS.md §Landing paperwork). ` +
      `It is a ledger file, so touching it here is authorized; nothing else in the round changes.`,
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
