// Branch-protection claim guard — does what this repo's docs SAY about required
// checks still match what GitHub actually enforces?
//
// Adopting docs/worklog/2026-09-17-adopt-slice-d-base-swap-fix3-p1.json #p1.
//
// ── Why this exists, measured rather than argued ─────────────────────────────
// The round that proposed it had just corrected five paragraphs that claimed
// `contract` was a required check, because it was not: `branches/main/protection`
// answered 404 and `rulesets` answered `[]`. Its own proposal then said the
// correction would rot the moment the operator flipped protection on, and that
// nothing watched for it.
//
// ★It rotted the SAME DAY. Measured 2026-09-17 12:4x KST, hours after that round
//   landed: a repo RULESET named "main protection" exists, `enforcement: active`,
//   targeting `~DEFAULT_BRANCH`, carrying `required_status_checks` with five
//   contexts. Its `created_at` is 2026-09-17T10:17:54+09:00 — after the measurement
//   the docs were written from.
//
// ── The trap this guard is shaped around ────────────────────────────────────
// ★Rulesets DO NOT show up in the endpoint those docs measured. Today, live:
//     GET /branches/main/protection  -> 404 "Branch not protected"   (classic only)
//     GET /branches/main             -> protected: true, protection.enabled: FALSE
//     GET /rulesets                  -> 1 active, required_status_checks x5
//   So the two signals the docs quoted are exactly the two a ruleset leaves
//   untouched, and "GitHub-enforced required checks: zero" read as measured fact
//   while five checks were enforced. A guard that asks only the classic endpoint
//   would have inherited the same blind spot, so this one asks all three.
//
// ── What it compares (no prose parsing) ─────────────────────────────────────
// AGENTS.md carries ONE machine-readable claim between REQUIRED-CHECKS:BEGIN /
// REQUIRED-CHECKS:END — the same marked-region shape check-engine-runner-fixtures.mjs
// uses. This diffs that list against the API, both directions. Prose elsewhere points
// at the marked region instead of restating it (§Constraints, "reference, never copy").
//
// ── Exit codes ──────────────────────────────────────────────────────────────
//   0  claim == reality
//   1  drift — the marked claim and GitHub disagree (prints both directions)
//   2  ★COULD NOT MEASURE — the token cannot read what it needs. NOT green:
//      "absence is not a pass" is this repo's oldest lesson, and a protection
//      endpoint that 403s must never read as "nothing is enforced".
//
// ── Who runs it: you, by hand. Nothing in CI does, and that is measured ─────
// In Actions `github.token` gets 403 "Resource not accessible by integration" on
// both the protection and the rulesets endpoint (run 35181122022 — this guard
// exited 2, correctly refusing to read 403 as "nothing enforced"), and
// `permissions: administration: read` does not fix it: that key is not grantable,
// so GitHub rejects the workflow at parse time (run 35180786771, startup_failure).
// Wiring it would need a PAT — a different cost class, not yet decided.
// ★So there is NO scheduled caller. AGENTS.md §Incident ledger names the two
// moments to run it: when you edit its REQUIRED-CHECKS block, and when a merge
// behaves unlike what that section says.
//
// Usage: node scripts/check-branch-protection-claim.mjs
// Needs a `gh` that can read /branches/main/protection and /rulesets — an
// authenticated local `gh` with admin on the repo, or a PAT carrying that scope.

import { readFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const BEGIN = "REQUIRED-CHECKS:BEGIN";
const END = "REQUIRED-CHECKS:END";

/** Claimed contexts = the `- ` bullets inside AGENTS.md's marked region. */
export function claimedContexts(doc) {
  const b = doc.indexOf(BEGIN);
  const e = doc.indexOf(END);
  if (b < 0 || e < 0 || e < b) return null; // caller turns this into a hard failure
  return doc
    .slice(b, e)
    .split("\n")
    .map((l) => /^\s*-\s+`([^`]+)`\s*$/.exec(l)?.[1])
    .filter(Boolean)
    .sort();
}

/** Set difference both ways — the whole verdict is these two arrays. */
export function drift(claimed, actual) {
  const a = new Set(actual);
  const c = new Set(claimed);
  return {
    claimedNotEnforced: claimed.filter((x) => !a.has(x)),
    enforcedNotClaimed: actual.filter((x) => !c.has(x)),
  };
}

const gh = (args) => execFileSync("gh", args, { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });

// ── Axis ⑵: the WHOLE ruleset, not one rule inside it ───────────────────────
// The axis above compares ONE thing: the required-check list. Everything else the
// same ruleset carries — `bypass_actors`, `required_approving_review_count`,
// `allowed_merge_methods`, `deletion`, `non_fast_forward`, which refs it targets —
// could change and this file would still print OK. Measured 2026-09-18: the live
// ruleset has FOUR rules and the loop below `continue`s past three of them.
//
// ★So this axis does not enumerate fields. Enumerating is how the next field
// added by GitHub slips through, and this repo has named that failure often
// enough. It fingerprints the normalized OBJECT, so a field added at the TOP
// level of a rule's `parameters` moves the fingerprint without anyone listing it.
//
// ★That is NOT the same as "any field, present or future" — an earlier revision
// of this comment said that, and it was false. Measured 2026-09-18 (gate② C1):
// adding `integration_id: 15368` to a `required_status_checks[]` entry left rc=0.
// The reason is one layer down, in `sortRules`: array elements are projected
// through `x.context ?? x`, so for an array OF OBJECTS every key except `context`
// is dropped. Live exposure today is zero (each entry is `{context}` and nothing
// else), which is why this is recorded rather than patched here — the successor
// ticket that noticed it is scoped to output and is forbidden from touching the
// fingerprint predicate. ★The one-line fix is known and was verified by the
// reviewer: sort via `JSON.stringify` round-trip, the way `bypass_actors` already
// does. Doing it also means RESEEDING the baseline in the same PR, because the
// stored `required_status_checks` goes from strings to `{context: …}` objects.
//
// ★The fingerprint is stored as READABLE JSON, not a hash. A hash answers "it
// changed" and stops there; the expected file is its own diff, so the failure can
// say WHICH line moved — which is what decides "operator meant this" vs "drift".
//
// ★WHAT NORMALIZATION DISCARDS, stated because a guard whose blind spot is
// undocumented gets trusted past its limits: `id`, `node_id`, `created_at`,
// `updated_at`, `source`, `source_type`, `current_user_can_bypass`, `_links`, and
// ★every key except `context` on an object inside a rule-parameter ARRAY (today
// that is `required_status_checks[]`, whose `integration_id` is therefore not
// compared — see the paragraph above). Consequences:
// deleting the ruleset and recreating it with identical content is INVISIBLE here
// (new id, same content), and "can *I* bypass right now" is not compared — it is a
// property of the caller, not of the repo, so comparing it would make the answer
// depend on who ran the tool. The `bypass_actors` LIST is compared; only the
// caller-relative view is dropped.
const RULESET_EXPECTED = ".github/branch-protection-expected.json";

const sortRules = (rules) =>
  [...(rules ?? [])]
    .map((r) => {
      const p = r.parameters;
      if (!p) return { type: r.type };
      const q = {};
      for (const k of Object.keys(p).sort()) {
        const v = p[k];
        // Required-check contexts arrive in operator order; sort so a cosmetic
        // reorder in the GitHub UI does not read as drift.
        q[k] = Array.isArray(v) ? [...v].map((x) => (x && typeof x === "object" ? x.context ?? x : x)).sort() : v;
      }
      return { type: r.type, parameters: q };
    })
    .sort((a, b) => a.type.localeCompare(b.type));

/** The comparable shape of one ruleset — see the block above for what is dropped. */
export function normalizeRuleset(full) {
  return {
    name: full.name,
    target: full.target,
    enforcement: full.enforcement,
    conditions: {
      ref_name: {
        include: [...(full.conditions?.ref_name?.include ?? [])].sort(),
        exclude: [...(full.conditions?.ref_name?.exclude ?? [])].sort(),
      },
    },
    bypass_actors: [...(full.bypass_actors ?? [])].map((a) => JSON.stringify(a)).sort().map((s) => JSON.parse(s)),
    rules: sortRules(full.rules),
  };
}

/** Enforced contexts = classic branch protection ∪ every ACTIVE branch ruleset on the default branch. */
function enforcedContexts(slug, branch) {
  const out = new Set();
  const notes = [];
  const shapes = [];

  // ⑴ classic protection — 404 means "no classic protection", which is NOT "nothing enforced".
  try {
    const p = JSON.parse(gh(["api", `repos/${slug}/branches/${branch}/protection`]));
    for (const c of p.required_status_checks?.contexts ?? []) out.add(c);
    notes.push("classic protection: present");
  } catch (e) {
    const msg = String(e.stderr || e.message || e);
    if (/Branch not protected|HTTP 404/.test(msg)) notes.push("classic protection: none (404)");
    else return { ok: false, why: `classic protection unreadable — ${msg.split("\n")[0]}`, notes };
  }

  // ⑵ rulesets — the axis the docs' original measurement could not see.
  try {
    const sets = JSON.parse(gh(["api", `repos/${slug}/rulesets`]));
    const active = sets.filter((s) => s.enforcement === "active" && s.target === "branch");
    notes.push(`rulesets: ${sets.length} (${active.length} active branch)`);
    for (const s of active) {
      const full = JSON.parse(gh(["api", `repos/${slug}/rulesets/${s.id}`]));
      const include = full.conditions?.ref_name?.include ?? [];
      const hitsDefault = include.includes("~DEFAULT_BRANCH") || include.includes(`refs/heads/${branch}`);
      if (!hitsDefault) continue;
      shapes.push(normalizeRuleset(full));
      for (const r of full.rules ?? []) {
        if (r.type !== "required_status_checks") continue;
        for (const c of r.parameters?.required_status_checks ?? []) out.add(c.context);
      }
    }
  } catch (e) {
    // ★This is the branch the proposal said to measure first: can the Actions token
    //   read /rulesets? If it cannot, we must NOT fall back to the classic answer —
    //   that is precisely how "zero required checks" became a false measured fact.
    return { ok: false, why: `rulesets unreadable — ${String(e.stderr || e.message || e).split("\n")[0]}`, notes };
  }

  return { ok: true, out: [...out].sort(), notes, shapes };
}

// ── `--print-current`: the material the drift message tells you to paste ─────
// The drift path used to say "copy the LIVE shape" while printing only the lines
// that differ, so the person told to update the baseline had nothing to copy. The
// full shape WAS printed, but only on the rc=2 (no baseline) path — reachable only
// by deleting the file first.
//
// ★Chosen over "always print the whole shape on drift" because the drift message
// is the thing a human actually reads, and burying a 55-line object in it is how
// people stop reading it; and over "write an artifact file" because that ties the
// material to CI artifact lifetime, while this tool has no CI caller at all (it is
// run by hand). ★Its cost, stated: one more `gh api` round trip, and the live
// shape may have MOVED between the run that failed and this one — so re-run the
// plain check after reseeding. That re-run is the verification anyway.
//
// In this mode stdout carries ONLY the JSON, so `> .github/branch-protection-
// expected.json` is safe; the informational lines go to stderr.
const PRINT_CURRENT = process.argv.includes("--print-current");
const say = (...a) => (PRINT_CURRENT ? console.error(...a) : console.log(...a));

const slug = (() => {
  try {
    return gh(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"]).trim();
  } catch {
    return "";
  }
})();
if (!slug) {
  say("::error::cannot resolve the repo slug (`gh repo view`) — COULD NOT MEASURE, and that is not a pass.");
  process.exit(2);
}

// In --print-current mode the AGENTS.md claim is irrelevant: you are asking for the
// live shape, and a missing marker block must not stand between you and it.
const claimed = PRINT_CURRENT ? [] : claimedContexts(readFileSync(path.join(root, "AGENTS.md"), "utf8"));
if (claimed === null) {
  say(`::error file=AGENTS.md::${BEGIN}/${END} markers missing or reversed — the claim cannot be read, so nothing can be compared.`);
  process.exit(2);
}

const measured = enforcedContexts(slug, "main");
if (!measured.ok) {
  say(`::error::COULD NOT MEASURE — ${measured.why}. ${measured.notes.join(" · ")}`);
  say("A token that cannot read protection must never read as 'nothing is enforced' — run this locally with a `gh` that has admin on the repo, or supply a PAT that can read protection/rulesets. `permissions: administration: read` is NOT an option: that key is not grantable and the workflow is rejected at parse time (run 35180786771).");
  process.exit(2);
}

if (PRINT_CURRENT) {
  console.error(`live ruleset shape: ${measured.shapes.length} active ruleset(s) on the default branch · ${measured.notes.join(" · ")}`);
  console.error(`  ⇒ review this before saving it — you are declaring this state correct.`);
  process.stdout.write(`${JSON.stringify({ rulesets: measured.shapes }, null, 2)}\n`);
  process.exit(0);
}

const d = drift(claimed, measured.out);
console.log(`branch-protection claim: AGENTS.md claims ${claimed.length} · GitHub enforces ${measured.out.length} · ${measured.notes.join(" · ")}`);
console.log(`  claimed:  ${claimed.join(" | ") || "(none)"}`);
console.log(`  enforced: ${measured.out.join(" | ") || "(none)"}`);

for (const c of d.enforcedNotClaimed)
  console.log(`::error file=AGENTS.md::\`${c}\` is REQUIRED on main but the ${BEGIN} block does not list it — the docs understate what blocks a merge.`);
for (const c of d.claimedNotEnforced)
  console.log(`::error file=AGENTS.md::\`${c}\` is listed in the ${BEGIN} block but GitHub does not require it — the docs overstate the gate.`);

// ★Axis ⑴ records its verdict instead of exiting, so a round whose required-check
//   list drifted still gets told about a ruleset-shape drift in the same run. The
//   predicate is unchanged; only the moment of exit moved (gate② C4).
let failed = false;
if (d.enforcedNotClaimed.length || d.claimedNotEnforced.length) {
  console.log("Update the marked region in AGENTS.md to the measured list (that block is the one copy; prose elsewhere points at it).");
  failed = true;
}
// ── Axis ⑵ — whole-ruleset fingerprint (see the block above normalizeRuleset) ──
const expPath = path.join(root, RULESET_EXPECTED);
let expected;
try {
  expected = JSON.parse(readFileSync(expPath, "utf8"));
} catch (e) {
  // FAIL CLOSED: no baseline means "cannot judge", and "cannot judge" is not a pass.
  console.log(`::error file=${RULESET_EXPECTED}::cannot read the expected ruleset shape — ${String(e.message).split("\n")[0]}. COULD NOT MEASURE, and that is not a pass.`);
  // Seeding it by hand would re-type the normalization and drift from it, so the
  // live shape is printed here instead: review it, then save it as that file.
  console.log(`  ⇒ Seed it: node ${path.relative(process.cwd(), fileURLToPath(import.meta.url))} --print-current > ${RULESET_EXPECTED}`);
  console.log(`  ⇒ …or copy the live shape below (review before saving — this is the state you are declaring correct):`);
  console.log(JSON.stringify({ rulesets: measured.shapes }, null, 2));
  process.exit(2);
}
const liveShape = JSON.stringify(measured.shapes, null, 2);
const wantShape = JSON.stringify(expected.rulesets ?? expected, null, 2);
console.log(`ruleset shape: ${measured.shapes.length} active ruleset(s) on the default branch · baseline ${RULESET_EXPECTED}`);
if (liveShape !== wantShape) {
  const a = wantShape.split("\n");
  const b = liveShape.split("\n");
  console.log(`::error file=${RULESET_EXPECTED}::branch-protection ruleset DRIFTED — a field outside the required-check list changed on GitHub.`);
  // ★The pairing below is POSITIONAL, so an inserted or removed field shifts every
  //   line after it and each pair then reads as a substitution rather than as the
  //   insert/delete it is (gate② C3 — a removed `non_fast_forward` rule reported as
  //   `non_fast_forward ↔ pull_request`). Say so when the line counts differ, which
  //   is exactly when that misreading is possible; the reseed command below is the
  //   escape hatch that does not depend on reading this diff correctly at all.
  if (a.length !== b.length)
    console.log(
      `    ★fields were ADDED or REMOVED (baseline ${a.length} lines ↔ live ${b.length}) — the pairing below is positional, so everything after the first shift reads as a substitution. Diff the reseeded file instead of trusting these pairs.`,
    );
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    if (a[i] !== b[i]) console.log(`    line ${i + 1}:  expected ${a[i] ?? "(absent)"}   ↔   live ${b[i] ?? "(absent)"}`);
  }
  console.log(`  ⇒ If the operator meant it, reseed in the same PR and say why in the round's report:`);
  console.log(`       node ${path.relative(process.cwd(), fileURLToPath(import.meta.url))} --print-current > ${RULESET_EXPECTED}`);
  console.log(`       git diff -- ${RULESET_EXPECTED}   # ← read THIS as the change, then re-run this check (it must print OK)`);
  console.log("  ⇒ If nobody meant it: the repo's merge gate rests on these values (bypass_actors, approval count, allowed merge methods) — escalate before touching anything.");
  console.log("  ⇒ This tool never writes the ruleset; it only reads. Changing GitHub is an operator step.");
  failed = true;
}

if (failed) process.exit(1);
console.log("OK — the documented required-check list matches what GitHub enforces, and the ruleset shape matches its baseline.");
process.exit(0);
