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
// Usage: node scripts/check-branch-protection-claim.mjs
// Needs `gh` with `administration: read` for /rulesets (the weekly job grants it).

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

/** Enforced contexts = classic branch protection ∪ every ACTIVE branch ruleset on the default branch. */
function enforcedContexts(slug, branch) {
  const out = new Set();
  const notes = [];

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

  return { ok: true, out: [...out].sort(), notes };
}

const slug = (() => {
  try {
    return gh(["repo", "view", "--json", "nameWithOwner", "-q", ".nameWithOwner"]).trim();
  } catch {
    return "";
  }
})();
if (!slug) {
  console.log("::error::cannot resolve the repo slug (`gh repo view`) — COULD NOT MEASURE, and that is not a pass.");
  process.exit(2);
}

const claimed = claimedContexts(readFileSync(path.join(root, "AGENTS.md"), "utf8"));
if (claimed === null) {
  console.log(`::error file=AGENTS.md::${BEGIN}/${END} markers missing or reversed — the claim cannot be read, so nothing can be compared.`);
  process.exit(2);
}

const measured = enforcedContexts(slug, "main");
if (!measured.ok) {
  console.log(`::error::COULD NOT MEASURE — ${measured.why}. ${measured.notes.join(" · ")}`);
  console.log("A token that cannot read protection must never read as 'nothing is enforced' — grant `administration: read`, or run this where `gh` is authenticated.");
  process.exit(2);
}

const d = drift(claimed, measured.out);
console.log(`branch-protection claim: AGENTS.md claims ${claimed.length} · GitHub enforces ${measured.out.length} · ${measured.notes.join(" · ")}`);
console.log(`  claimed:  ${claimed.join(" | ") || "(none)"}`);
console.log(`  enforced: ${measured.out.join(" | ") || "(none)"}`);

for (const c of d.enforcedNotClaimed)
  console.log(`::error file=AGENTS.md::\`${c}\` is REQUIRED on main but the ${BEGIN} block does not list it — the docs understate what blocks a merge.`);
for (const c of d.claimedNotEnforced)
  console.log(`::error file=AGENTS.md::\`${c}\` is listed in the ${BEGIN} block but GitHub does not require it — the docs overstate the gate.`);

if (d.enforcedNotClaimed.length || d.claimedNotEnforced.length) {
  console.log("Update the marked region in AGENTS.md to the measured list (that block is the one copy; prose elsewhere points at it).");
  process.exit(1);
}
console.log("OK — the documented required-check list matches what GitHub enforces.");
process.exit(0);
