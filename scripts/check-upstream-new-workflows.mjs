// New-upstream-workflow guard (node-only, no deps).
//
// Adopting docs/worklog/2026-09-16-adopt-slice-d-base-swap-fix2-p3.json #p0.
//
// ── What it watches, and why that is not check-parked-workflows.mjs ──────────
// Its sibling `check-parked-workflows.mjs` guards two workflows we ALREADY
// triaged — its PARKED array is hand-maintained, and that is precisely its
// blind spot: it cannot see a THIRD upstream workflow arriving. The base swap
// brought two at once and both were found by eye; one of them (release.yaml)
// carried a nightly `cron` that would have deployed to Cloudflare with our
// token and cut GitHub releases on this repo. The parking round wrote that what
// stopped it was "luck, not a guard" — the missing `npm run build:prod` script.
// A third arrival gets the same luck or does not.
//
// So this script holds the OTHER half: the set difference. It never reads a
// file's contents and has no list of workflows to keep current.
//
//     upstream's .github/workflows/  −  ours  =  never triaged here
//
// That subtraction needs no hand-maintained inventory because every disposition
// of an upstream workflow but one puts the file in OUR tree: adopt it and it is
// ours; park it (the decided shape — deleting recurs on every sync) and it is
// ours with reduced triggers. Either way the difference returns to empty on its
// own. The one exception is KNOWN_ABSENT below.
//
// ── Deliberately NOT watched ────────────────────────────────────────────────
// Upstream EDITING a workflow we already have. That is the proposal's own
// trade: upstream touches its workflows often (dependabot alone moves them), so
// watching edits makes this a weekly noise generator, and the edit case already
// has a guard with teeth — `check-parked-workflows.mjs` asserts the trigger
// surface of the two files where an upstream edit could actually arm something.
//
// ── Where this runs, and why not on a PR ────────────────────────────────────
// Weekly, as a step of .github/workflows/doc-liveness.yml. The signal moves when
// UPSTREAM moves, not when we do: run it per-PR and a new upstream workflow
// reddens every open PR at once, for a thing none of them touched. This repo has
// already paid for that shape — `check-worklog-coverage` went red on 2026-09-07
// and blocked every open PR. Weekly also inherits that job's owner rule (the
// first gate③ round after a red run files a ticket), so the red has a name
// without inventing a second rule.
//
// Usage: node scripts/check-upstream-new-workflows.mjs
// Exit:  0 = nothing new upstream · 1 = new workflow(s), or could not measure.

import { readdir } from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DIR = ".github/workflows";
const UPSTREAM_REF = "upstream/main";

// Upstream workflows we looked at and deliberately do NOT carry a file for.
// Empty on purpose: this repo's decided disposition is to keep-and-park, not to
// delete (deleting recurs on every upstream sync — see check-parked-workflows.mjs).
// It exists so that a deliberate deletion, or a rename of an upstream workflow,
// has somewhere to be declared instead of leaving this check permanently red with
// no exit. An entry without a reason beside it is not a disposition.
const KNOWN_ABSENT = [];

const isWorkflow = (name) => name.endsWith(".yml") || name.endsWith(".yaml");

function fail(msg) {
  console.log(`::error title=new upstream workflow::${msg}`);
  process.exit(1);
}

let upstream;
try {
  upstream = execFileSync("git", ["ls-tree", "--name-only", UPSTREAM_REF, `${DIR}/`], { cwd: root, encoding: "utf8" })
    .split("\n")
    .map((line) => line.trim())
    .filter(Boolean)
    .map((p) => path.posix.basename(p))
    .filter(isWorkflow);
} catch {
  // Fail-closed: "could not measure" is not "nothing new" (the ci-presence rule).
  fail(
    `cannot resolve \`${UPSTREAM_REF}\`. Fetch it first:\n` +
      `  git remote add upstream https://github.com/dlunch/wie.git   # once\n` +
      `  git fetch --depth=1 upstream main:refs/remotes/upstream/main`,
  );
}

if (upstream.length === 0) {
  fail(`\`git ls-tree ${UPSTREAM_REF} ${DIR}/\` listed zero workflows — refusing to fail-open (a zero here reads as "nothing new")`);
}

const ours = new Set((await readdir(path.join(root, DIR))).filter(isWorkflow));
const declared = new Set(KNOWN_ABSENT);
const fresh = upstream.filter((name) => !ours.has(name) && !declared.has(name));

console.log(`new-upstream-workflow guard — upstream ${upstream.length}, ours ${ours.size}, declared-absent ${declared.size}, new ${fresh.length}`);

if (fresh.length > 0) {
  fail(
    `${UPSTREAM_REF} ships ${fresh.length} workflow(s) this repo has never triaged: ${fresh.join(", ")}. ` +
      `Read each one's triggers BEFORE the next \`git merge upstream/main\` — upstream's release.yaml arrived with a nightly cron, ` +
      `a Cloudflare deploy on our token and a GitHub release publisher. Then pick one: adopt it (the file lands here), ` +
      `park it (carry the file with \`on: workflow_dispatch\` only, plus a header saying why, and add it to PARKED in ` +
      `scripts/check-parked-workflows.mjs), or declare it in KNOWN_ABSENT in this script with the reason.`,
  );
}

console.log("OK — upstream ships no workflow this repo has not triaged.");
