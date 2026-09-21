// Enforces the *promise* AGENTS.md §Landing paperwork made — "re-measure every
// 10 landed rounds" — not the threshold judgment itself.
//
// The distinction is the whole design. Coverage below 70% is a signal to REOPEN
// a decision, and reopening is a human call: the metric has known, legitimate
// misses baked in (a round that leaves no follow-ups owes no worklog and still
// counts against it; an upstream-sync merge lands as one round with no worklog
// at all). Failing CI on the ratio would gate PRs on a *conditional* obligation
// — exactly the mandate 2026-09-01 declined. Being overdue, by contrast, is not
// a judgment: it is a fact.
//
// That sentence used to end "and it has no false positives", and it was wrong —
// measured 2026-09-21, a branch with an old base reported an overdue that did not
// exist on main. The claim is true again only because of the union read below; it
// is stated as a consequence there, not as a property of the idea.
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
// ── Which COPY of the record — the union of `head`'s and this tree's ────────
// The landed count comes from `head` (origin/main). The record used to come from
// the working tree ALONE, and that pairing is what the header above got wrong when
// it called OVERDUE "a fact, with no false positives": landed moves with main, the
// tree's copy does not, so a branch whose base is 10+ landings old reports an
// overdue that does not exist on main. Measured 2026-09-21 on PR #233 (`cb53573d`,
// base `4acb1631`): landed 159 · tree's copy 145 · `✗ OVERDUE` — while a real merge
// with the then-current main read 155 and `OK`.
//
// It is not a second design axis to be argued about: this file is a union-merged
// ledger file (AGENTS.md §Landing paperwork — "Never take one side wholesale"), so
// the record that will exist on main once this branch lands is BOTH copies. Reading
// the tree alone models a take-ours merge that same rule forbids. So `last` is the
// highest `landedRounds` across the two copies.
//
// What that costs, because it is not free: a branch that DELETES rows from its copy
// no longer reads as overdue — `head`'s copy restores the number and the check goes
// green. That regression is invisible here by construction, and this is the one
// place it used to be visible.
//
// Both copies are still read, rather than just `head`'s, because `--record` writes
// the working tree: a round that has just discharged the obligation must read as
// current before it lands, or it could never discharge it at all.
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
const highest = (measurements) => measurements.reduce((a, b) => (b.landedRounds >= a.landedRounds ? b : a));
const lastHere = highest(record.measurements);

// `head`'s copy, unioned with the tree's — see the header. A failure here is not
// fatal: fall back to the tree alone, and SAY so, because that fallback is the old
// behaviour and can still read a stale base as overdue.
let lastCanon = null;
try {
  lastCanon = highest(JSON.parse(git("show", `${head}:${recordPath}`)).measurements);
} catch {
  lastCanon = null;
}
// Only `origin/main` is a cross-check. In the `HEAD` fallback above there is no
// independent copy to compare against — `git show HEAD:` is this same branch.
const crossChecked = head === "origin/main" && lastCanon !== null;
const last = lastCanon && lastCanon.landedRounds > lastHere.landedRounds ? lastCanon : lastHere;
const behind = lastCanon !== null && lastCanon.landedRounds > lastHere.landedRounds;

console.log(`worklog coverage: ${num}/${den} = ${pct.toFixed(1)}% over the last ${den} landed round(s)`);
console.log(
  `  window ${oldest}..${newest} · landed rounds since ${SINCE}: ${landed} · last recorded at: ${last.landedRounds}` +
    (behind ? ` (from ${head}; this working tree's copy stops at ${lastHere.landedRounds} — its base is behind)` : ""),
);
if (!crossChecked) {
  console.log(
    `  NOTE: ${recordPath} was not cross-checked against origin/main (` +
      (lastCanon === null ? `it could not be read from \`${head}\`` : `the landed count came from \`${head}\`, not origin/main`) +
      `), so an old base can still read as OVERDUE when main is current.`,
  );
}
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
  } else if (behind) {
    // Guard 3: due, but this tree's copy is older than `head`'s. Appending here
    // writes a file that is missing `head`'s rows, and landing it needs the union
    // resolution to put them back. Pull base instead — that is one command.
    console.log(
      `  --record: this working tree's ${recordPath} stops at ${lastHere.landedRounds} but ${head} is at ` +
        `${lastCanon.landedRounds} — pull base first; appending here writes a copy missing ${head}'s rows.`,
    );
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
  // The OWNER instruction is conditional on this reading being cross-checked against
  // origin/main. Uncross-checked, "run --record and bundle it" is the wrong thing to
  // do — it records a deadline that has not arrived, which is the same pollution the
  // "do NOT hand-append" sentence exists to prevent.
  const lead = `OVERDUE: ${landed - last.landedRounds} landed rounds since the last recorded measurement (cadence is ${WINDOW}). `;
  problems.push(
    crossChecked
      ? lead +
          `Run \`node scripts/check-worklog-coverage.mjs --record\` — it appends the entry printed above, idempotently. ` +
          `Do NOT hand-append: every round that pulls base gets this same rc=1, and hand-appending produced three identical rows on 2026-09-06. ` +
          `OWNER: the gate3 round — bundle that one file into the PR before merging (AGENTS.md §Landing paperwork). ` +
          `It is a ledger file, so touching it here is authorized; nothing else in the round changes.`
      : lead +
          `But the landed count came from \`${head}\` and ${recordPath} was not cross-checked against origin/main, so this rc=1 may be ` +
          `an artifact of an old base rather than a real overdue. Re-measure on a REAL merge of current \`main\` into this branch — ` +
          `NOT on \`refs/pull/N/merge\`, whose cached base goes stale (measured 2026-09-21: PR #233's merge ref carried base \`4acb1631\`, ` +
          `14 landings behind). Do NOT run \`--record\` on this reading; no owner is named because there may be nothing to own.`,
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
