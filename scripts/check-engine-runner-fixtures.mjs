// Diffs AGENTS.md's engine-runner list against the committed fixture set, both
// directions. It answers one question — "a fixture appeared or vanished; does
// the list that tells you to run it still match?" — which nothing else asks:
// the DoD parity lock reads only the COMMIT-GATES region, and its END marker
// says so itself.
//
// ── Why stems and not paths, measured rather than assumed ────────────────────
// On 2026-09-07 the two sets were **5 and 5** and a count check would have said
// "fine". The paths were not the same set:
//     named in the block, not tracked : test_data/draw_j2me.jar
//     tracked, not named in the block : test_data/draw_j2me.zip
// Neither is drift. The block's own first line builds the .jar from the .zip
// (scripts/make-draw-fixture.mjs) and `*.jar` is gitignored, so the committed
// fixture and the file the runner opens are the same fixture under two
// extensions. Comparing basenames-without-extension collapses exactly that and
// nothing else: `draw_j2me_v2.zip` does NOT match `draw_j2me`, because the
// comparison is set equality on stems, not a substring test.
//
// So this file makes two claims and you can check both: counting is not enough
// (5 == 5 above), and stems are enough (the diff is 0 both ways at that commit).
//
// ── What it deliberately does not know ──────────────────────────────────────
// It has no idea which fixtures "should" be runner fixtures, and it must not
// learn: that classification is the second source of truth the proposal behind
// this check called out. A fixture the runner should skip is excused IN the
// marked region with `NOT-RUN: test_data/<name> — <why>`, so the list and its
// exceptions stay in one document.
//
// Usage: node scripts/check-engine-runner-fixtures.mjs

import { readFile } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const BEGIN = "ENGINE-RUNNER:BEGIN";
const END = "ENGINE-RUNNER:END";

const doc = await readFile(path.join(root, "AGENTS.md"), "utf8");
const b = doc.indexOf(BEGIN);
const e = doc.indexOf(END);
// Fail rather than pass on a missing marker. A checker that silently reads an
// empty region is worse than no checker: it reports OK forever.
if (b < 0 || e < 0 || e < b) {
  console.log(`::error file=AGENTS.md::engine-runner markers missing or reversed (${BEGIN} at ${b}, ${END} at ${e}) — the fixture-drift check cannot read the list.`);
  process.exit(1);
}
const region = doc.slice(b, e);

const stem = (p) => path.basename(p).replace(/\.[^.]*$/, "");
const paths = (s) => [...new Set(s.match(/test_data\/[A-Za-z0-9_.-]+/g) ?? [])].sort();

// `NOT-RUN:` lines are excuses, not runner entries — pull them out first so a
// fixture named only as an excuse does not read as "the runner covers it".
const excused = paths(
  region
    .split("\n")
    .filter((l) => l.includes("NOT-RUN:"))
    .join("\n"),
);
const named = paths(region).filter((p) => !excused.includes(p));

const tracked = execFileSync("git", ["ls-files", "test_data/"], { cwd: root, encoding: "utf8" })
  .split("\n")
  .filter(Boolean)
  .sort();

const namedStems = new Set(named.map(stem));
const excusedStems = new Set(excused.map(stem));
const trackedStems = new Set(tracked.map(stem));

// Direction 1: a committed fixture nobody runs and nobody excused.
const uncovered = tracked.filter((p) => !namedStems.has(stem(p)) && !excusedStems.has(stem(p)));
// Direction 2: the list names something that is not in the repo any more.
const dangling = named.filter((p) => !trackedStems.has(stem(p)));

console.log(`engine-runner fixture drift: ${tracked.length} tracked · ${named.length} named${excused.length ? ` · ${excused.length} excused` : ""}`);
console.log(`  tracked: ${tracked.join(" ") || "(none)"}`);
console.log(`  named:   ${named.join(" ") || "(none)"}`);
if (excused.length) console.log(`  excused: ${excused.join(" ")}`);

if (uncovered.length === 0 && dangling.length === 0) {
  console.log("OK — every committed fixture is named or excused, and every named fixture exists.");
  process.exit(0);
}
for (const p of uncovered) {
  console.log(`::error file=AGENTS.md::${p} is committed but the engine-runner block neither runs nor excuses it — add it to the runner, or add \`NOT-RUN: ${p} — <why>\` inside the marked region.`);
}
for (const p of dangling) {
  console.log(`::error file=AGENTS.md::the engine-runner block names ${p}, which is not a committed fixture (and not built from one) — drop the line or restore the fixture.`);
}
process.exit(1);
