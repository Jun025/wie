// inflow-marker.mjs — the one definition of the corpus-name-inflow freshness marker.
//
// ── Why a shared module and not two copies ──────────────────────────────────
// Two sides compute the same hash: `corpus-name-inflow.mjs` when it EMITS a marker, and
// `check-inflow-marker.mjs` when it VERIFIES one. If those two drift by one byte the check
// reddens working rounds, and a guard that reddens working code is a guard the next person
// deletes (`check-parity-lock-wired` learned that on 2026-09-06). So the hash lives HERE,
// once, and both sides import it.
//
// ── What the marker asserts, and what it deliberately does NOT ──────────────
// It asserts FRESHNESS: "these three counts were produced by the tool over exactly the
// content that is in the tree right now". That is the failure this lineage actually had —
// measured 4 times out of 4, never once "forgot to run the tool":
//   · 0187        measured 16회/8쌍, then wrote the evidence list (game names) into the same
//                 docs, which ADDS occurrences, and did not re-measure. Real: 29회/17쌍.
//   · 0188 worklog  same shape — a `-fix` added sentences and the count moved.
//   · 0189 worklog  measured BEFORE committing, when the subject set was 0 files.
//   · 0190 worklog  the cited justification was wrong (NOT a staleness failure — this
//                   marker would not have caught it, and saying so is the point).
//
// ★It does NOT assert TRUTH. Nothing here re-derives the buckets, because that needs the
// corpus — git-ignored real game bytes (Constraint 9), absent on every runner and different
// on every machine. A number hand-typed into a marker over unchanged content passes. That
// axis ("did the round call the tool at all") was measured and declined on 2026-09-20
// (`docs/report/0195`); it is not this file's job and pretending otherwise would be the
// wording-proxy that decision rejected.
//
// ── The self-reference, which is the one non-obvious part ───────────────────
// The marker is pasted INTO a subject file, so hashing the file naively would make every
// marker invalidate itself the moment it is written. Marker lines are therefore STRIPPED
// before hashing — by both sides. That also means re-running the tool with an older marker
// already in the tree reproduces the same hash, which is what makes "paste, then verify"
// terminate instead of chasing its own tail.
// ★Stripping alone is not enough, and this file learned that on its own first green case: see
// `normalize` below. Whitespace has to go too, or the blank line a Markdown author types above
// the marker reddens the round that just did everything right.

import { createHash } from "node:crypto";

export const MARKER_VERSION = "v1";
// `<!-- … -->` so it is invisible in rendered Markdown, and a plain substring inside a JSON
// string so a worklog can carry it too. The regex is deliberately anchored on the whole line:
// stripping is line-granular, and a marker sharing a line with prose would take that prose
// out of the hash.
export const MARKER_RE = /^.*<!--\s*corpus-name-inflow\s+v1\b[^>]*-->.*$/;

const nfc = (s) => s.normalize("NFC");

/**
 * Normalise to "the part of the text that can move the counts".
 *
 * ⑴ Marker lines are dropped, so a marker never invalidates itself.
 * ⑵ ★Whitespace is normalised too — blank lines dropped, every line trimmed — and that is NOT
 *    tidiness, it is required for ⑴ to work at all. The first version of this file did only ⑴
 *    and its own "paste the marker" green case came out RED: pasting `\n<marker>\n` leaves the
 *    surrounding blank line behind after the marker line is removed, so the digest moved by
 *    exactly the whitespace a Markdown author naturally types.
 *
 * ★Safe, and here is the argument rather than the assurance: the counter classifies a hit by the
 * single character on each side, and whitespace is a non-word character on every side. Trimming
 * only touches line-leading and line-trailing runs — a stem at line start already had `\n`
 * before it, and one at line end already had whitespace or `\n` after it, so the verdict is the
 * same before and after. Dropping a blank line turns `\n\n` into `\n`: non-word either way. A
 * stem can never match across a newline, so no removal can create or destroy a hit.
 */
export const normalize = (text) =>
  text
    .split("\n")
    .filter((l) => !MARKER_RE.test(l))
    .map((l) => l.trim())
    .filter((l) => l.length > 0)
    .join("\n");

/**
 * Fingerprint of the measured content.
 * `files` is a Map<repo-relative path, raw text>. Paths are sorted so the digest does not
 * depend on the order git happened to list them; NFC so a macOS NFD path/content round-trip
 * cannot move the hash (the same normalisation the counter itself applies).
 */
export function treeDigest(files) {
  const h = createHash("sha256");
  for (const p of [...files.keys()].sort()) {
    h.update(nfc(p));
    h.update("\0");
    h.update(nfc(normalize(files.get(p))));
    h.update("\0");
  }
  return h.digest("hex").slice(0, 16);
}

export function formatMarker({ subjects, digest, b, p, s }) {
  return `<!-- corpus-name-inflow ${MARKER_VERSION} subjects=${subjects} tree=${digest} B=${b} P=${p} S=${s} -->`;
}

/** Parse every marker in a text. Returns [] when there is none — never throws. */
export function parseMarkers(text) {
  const out = [];
  for (const line of text.split("\n")) {
    if (!MARKER_RE.test(line)) continue;
    const kv = {};
    for (const m of line.matchAll(/(\w+)=([^\s>]+)/g)) kv[m[1]] = m[2];
    out.push(kv);
  }
  return out;
}
