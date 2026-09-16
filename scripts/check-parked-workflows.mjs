// Parked-upstream-workflow guard (node-only, no deps).
//
// Two of upstream's workflows arrived with the 2026-09-16 base swap and were
// DECIDED against (wie-adopt-slice-d-base-swap-fix2-p3, adopting proposal
// 2026-09-16-slice-d-base-swap-fix2#p3): we do not ship upstream's Tauri
// desktop/mobile release pipeline, and our web CI is web.yml. The files are kept
// rather than deleted — deleting them recurs on every upstream sync, and the
// originals are the reference for what upstream does — so the decision lives as
// "triggers reduced to workflow_dispatch". Each file's header says why.
//
// THAT is what this script guards, and the reason it exists at all: the parking
// is a LOCAL edit to an UPSTREAM file. The next `git merge upstream/main` can
// restore the original triggers, and nothing else in the repo would notice.
// Upstream's release.yaml carries `schedule: cron "17 0 * * *"` plus a tag-push
// trigger; re-armed on main it would, every night and unasked: `pages deploy`
// to the Cloudflare projects `wie`/`wie-dev` (ours is `wie-web`) using OUR
// CLOUDFLARE_API_TOKEN, and cut GitHub releases here, where publish-artifact.yml
// already owns releases and otterpebble consumes them by repository_dispatch.
// Two release publishers on one repo is a consumer-visible hazard.
//
// What stops it today WITHOUT this guard is that `npm run build:prod` is absent
// from our package.json and every later job `needs:` the job that runs it. The
// parking round called that "luck, not a guard" — adding one package.json script
// arms the whole chain. This script is the guard.
//
// Scope, deliberately narrow: it asserts the trigger surface and nothing else.
// It does not read job bodies, does not care what the workflows contain, and has
// no opinion on any workflow not listed here. Un-parking is a legitimate act —
// it just has to be a deliberate one, which means editing this list in the same
// commit and defending it in review.
//
// Usage: node scripts/check-parked-workflows.mjs

import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

// The decision record for each entry is the file's own header comment.
const PARKED = [".github/workflows/release.yaml", ".github/workflows/web.yaml"];
const ALLOWED = "workflow_dispatch";

const violations = [];

for (const rel of PARKED) {
  let src;
  try {
    src = await readFile(path.join(root, rel), "utf8");
  } catch {
    // Fail-closed: a parked file that vanished is a decision reversed silently.
    violations.push(`${rel} is missing — if it was deleted on purpose, drop it from PARKED in this script too`);
    continue;
  }

  // `on:` is a top-level key: from its line to the next column-0 key.
  const m = /^on:[ \t]*(.*)$\n?([\s\S]*?)(?=^\S)/m.exec(src);
  if (!m) {
    violations.push(`${rel}: could not locate the top-level \`on:\` block — refusing to fail-open; fix this locator if the file moved`);
    continue;
  }

  // Flow form (`on: [push]`) puts the triggers on the SAME line, where a
  // block-mapping parser sees an empty block and reports no triggers at all.
  // Measured 2026-09-16 on coverage.yml, which uses exactly that form.
  const inline = m[1].trim();
  const triggers = inline
    ? inline.replace(/^\[|\]$/g, "").split(",").map((s) => s.trim()).filter(Boolean)
    : [...m[2].matchAll(/^ {2}([A-Za-z_][A-Za-z0-9_]*):/gm)].map((x) => x[1]);

  if (triggers.length === 0) {
    violations.push(`${rel}: parsed zero triggers from its \`on:\` block — refusing to fail-open (a zero here would read as "parked")`);
  } else if (triggers.length === 1 && triggers[0] === ALLOWED) {
    console.log(`  ✓ ${rel}: ${ALLOWED} only`);
  } else {
    violations.push(
      `${rel} is no longer parked: triggers = [${triggers.join(", ")}], expected [${ALLOWED}] only. ` +
        `If an upstream merge restored them, re-park the file (its header says why). ` +
        `If adopting it is intended, that is a decision — remove it from PARKED in this script in the same commit ` +
        `and settle what the header lists: Pages project name, release ownership vs publish-artifact.yml, ` +
        `the missing package.json script, and wie-app's workspace exclusion.`,
    );
  }
}

console.log(`parked-workflow guard — ${PARKED.length - violations.length} parked, ${violations.length} violation(s)`);
for (const v of violations) console.log(`::error title=parked upstream workflow re-armed::${v}`);
if (violations.length > 0) process.exit(1);
console.log("OK — the parked upstream workflows are still dispatch-only.");
