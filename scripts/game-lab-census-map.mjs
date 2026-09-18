#!/usr/bin/env node
// Per-game census mapping for `game_lab/` — stem -> bucket, for the whole corpus.
//
// ── Why this exists ──────────────────────────────────────────────────────────
// The 2026-09-18 census published bucket TOTALS and wrote its per-game rows to a
// `mktemp -d` scratch that it then deleted. So the totals survived and the
// mapping did not, and "which games are the 32 `no frame rendered` ones" became
// unanswerable — the only way back was re-running all 187 games (~45 min of a
// self-hosted runner). One round's scratch directory cost the next round an hour.
//
// ── Why the predicate lives HERE and not in the ledger ───────────────────────
// It used to live only in `~/orchestrator/reports/wie-game-lab-census-*.done.md`,
// which this repo cannot see: the round that went looking for it grepped the repo,
// found nothing, and concluded "the rules are recorded nowhere" — a claim its own
// successor had to retract. RULES below is therefore the authoritative copy, and
// the ledger path is cited as PROVENANCE, not as a second source: if the two ever
// disagree, this file is what classified the games.
//
// ── Why the OUTPUT does not live here ────────────────────────────────────────
// Measured 2026-09-18: `game_lab/broken/**` holds 187 files / 184 distinct stems.
// The three homes the ticket priced, with what each actually costs:
//
//   (1) inside game_lab/ only  — repo cannot see it; it rots with nobody watching
//   (2) inside the repo        — 187 rows of titles enter history, un-undoable
//   (3) hash in repo + names beside corpus — a second mapping to keep in sync
//
// ★An earlier revision of this block priced (2) as "commercial game titles enter
// git history, which this repo does not do". That premise was wrong and gate 2
// measured it: `scripts/smoke_gate_baseline.tsv` is TRACKED and already commits
// 292 rows / 283 distinct stems, every row carrier-prefixed
// (`ktf/(KTF) 건담시드.zip`), including the very trademark words that argument
// leaned on. And Constraint 9 is "No game *bytes*, ever" — not names; AGENTS.md's
// smoke_gate bullet explicitly blesses committing "identifiers and expected
// status only". So (2)'s real cost is 187 rows of churn in a file nothing
// regenerates, not a policy breach.
//
// This is (3) reshaped: what lives in the repo is the GENERATOR, not a table —
// and the two reasons that survive measurement are:
//   * REGENERABLE — the corpus moves, so a committed table goes stale with no
//     check watching it, while this is one command;
//   * REVIEWABLE  — what CI and a reviewer can act on is the 5-line predicate
//     that produced the 187 rows, not the rows.
// The layer (3) was charged for — a hash<->name table — does not exist, because
// nothing needs to be joined: you re-run this and get the names.
//
// ★What this does NOT claim: that no game names reach the repo. Measured after
// the fact, the three files this round added carry 12 of the corpus's 184 stems
// in their prose (this file: 3). That is allowed — see the smoke_gate precedent
// above — but it is not zero, and an earlier revision said zero.
//
// ★What that does NOT buy: the output is still invisible to CI, so a STALE
// mapping cannot be detected by any check. The mitigation is that regenerating is
// one command, not that staleness is impossible. Do not read this as safety.
//
// ── Usage ────────────────────────────────────────────────────────────────────
//   node scripts/game-lab-census-map.mjs                       # write the TSV, print totals
//   node scripts/game-lab-census-map.mjs --bucket unimpl-stub  # file paths, one per line
//   node scripts/game-lab-census-map.mjs --reports <dir> --corpus <dir> --out <file>
//
// ★`--bucket` takes a BUCKET, and the buckets are exactly the left column of
// RULES plus PASS / UNCLASSIFIED / NO-REPORT. It is not a free-text search: a
// failure *signature* like `no frame rendered` lives INSIDE `UNCLASSIFIED`, so
// asking for it returns `# 0 file(s)` with rc=0 — no error, no warning. An
// earlier revision of this block advertised exactly that, which is the "absence
// reads as a pass" shape this repo keeps naming. To query by signature, grep the
// TSV's 6th column instead — that is the working path, and it is one command:
//
//   awk -F'\t' '$6 ~ /no frame rendered/ {print $1}' game_lab/census-map.tsv
//
// ★Column 6 is the reason's FIRST LINE truncated to 120 chars, and that is not a
// general search index: measured 2026-09-18, a single excerpt value
// (`tick error during 'boot': Fatal error: `) covers 99 of 187 rows and spans
// several buckets, 164 of 452 reports have a multi-line reason, and 4 first lines
// exceed 120 chars. So a signature that sits on line 2 is invisible to that grep.
// It works for `no frame rendered` because that reason is a single 36-char line.
//
// ★The `--reports` flag is the whole point of the re-application discipline below:
// changing RULES means running this against BOTH inputs (the old reports and the
// new ones) and re-publishing both columns, because a predicate edit silently
// re-buckets history otherwise. That is a rule a person keeps, not a check — the
// only thing mechanised here is that doing it correctly costs one extra flag.

import { readdirSync, readFileSync, writeFileSync, existsSync, statSync } from "node:fs";
import path from "node:path";

// The classification predicate. PROVENANCE: reverse-engineered from the July
// published table by `wie-game-lab-census-is-from-a-different-jvm`, and the
// CURRENT form of that work is `docs/report/0165` in this tree — read that, not
// the first ledger reply (`~/orchestrator/reports/wie-game-lab-census-is-from-a-
// different-jvm.done.md`), which its own `-fix` round superseded on two points
// that matter here. Both are cited because the ledger is where the derivation is
// narrated; `0165` is where the corrections landed.
//
// ★What that `-fix` corrected, restated so this file does not re-publish the
// retracted version: the reconstruction reproduces 5 of the July table's 15
// buckets and folds the other 10 into UNCLASSIFIED. It is NOT "6 of 6", and the
// matching total (186) is not evidence of anything — every partition of 186 sums
// to 186. Rule 4 was likewise widened from a hardcoded `lgt` to `(\w+)` by that
// round; see the note on that line.
//
// Order matters: first match wins.
const RULES = [
  ["NoSuchMethod", /NoSuchMethodError|Method .*? not found/],
  ["panic-unwrap", /unwrap\(\)/],
  ["unimpl-stub", /WieError: Unimplemented/],
  // ★`(\w+)`, not `lgt`. Today's tree can only emit the `lgt` form (the single
  // producer is `wie-lgt/src/runtime/stdlib.rs`), so on today's input the two
  // spellings classify identically — measured, 0 rows differ across all 187. The
  // widening is for the day another carrier adds the same line: with `lgt`
  // hardcoded those rows fall silently into UNCLASSIFIED instead of into this
  // bucket. `docs/report/0165` records the revert; do not narrow it back.
  ["unknown-stdlib-import", /Unknown (\w+) stdlib import/],
  ["panic-unreachable", /unreachable code/],
];

const nfc = (s) => s.normalize("NFC");

const argv = process.argv.slice(2);
const flag = (name, fallback) => {
  const i = argv.indexOf(`--${name}`);
  return i >= 0 && argv[i + 1] ? argv[i + 1] : fallback;
};
const corpusDir = flag("corpus", "game_lab/broken");
const reportsDir = flag("reports", "game_lab/reports");
const outFile = flag("out", "game_lab/census-map.tsv");
const onlyBucket = flag("bucket", null);

for (const d of [corpusDir, reportsDir]) {
  if (!existsSync(d)) {
    console.error(`game-lab-census-map: ${d} does not exist — this needs the corpus, which is git-ignored and local-only.`);
    process.exit(2);
  }
}

// Population: every archive under the corpus dir, keyed by NFC stem.
// ★NFC is load-bearing: macOS stores these names decomposed, so a literal typed
// in a script is NFC and will match NOTHING against the raw filenames. A round
// measured 0 hits that way before normalising.
const walk = (dir) =>
  readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const p = path.join(dir, e.name);
    return e.isDirectory() ? walk(p) : [p];
  });

// ★One row per FILE, not per stem. Measured 2026-09-18: 187 files share 184
// stems — `놈3`, `다크슬레이어2` and `이노티아연대기2` each exist under two
// carriers. Keying by stem loses one copy of each, and the consumer of this map
// re-runs PATHS: a stem-keyed list would silently skip a file it was meant to
// cover. It also reconciles the totals — file-keyed counting reproduces the
// census's July column 7/7 exactly (37·37·15·6·4·80·7 = 186), where stem-keyed
// counting is short by exactly those three (35·79 = 183).
const corpus = [];
for (const p of walk(corpusDir)) {
  if (!/\.(zip|jar|kdp)$/i.test(p)) continue;
  corpus.push({ file: p, stem: nfc(path.basename(p).replace(/\.[^.]+$/, "")) });
}
corpus.sort((a, b) => (a.file < b.file ? -1 : 1));

const reports = new Map();
for (const f of readdirSync(reportsDir)) {
  if (!f.endsWith(".json")) continue;
  try {
    reports.set(nfc(f.replace(/\.json$/, "")), JSON.parse(readFileSync(path.join(reportsDir, f), "utf8")));
  } catch {
    /* a malformed report is a missing report here, not a crash */
  }
}

const bucketOf = (r) => {
  if (r.result === "PASS") return "PASS";
  const reason = r.reason || "";
  for (const [name, re] of RULES) if (re.test(reason)) return name;
  return "UNCLASSIFIED";
};

const rows = [];
for (const { file, stem } of corpus) {
  const r = reports.get(stem);
  const carrier = path.basename(path.dirname(file));
  if (!r) {
    rows.push({ file, stem, carrier, result: "NO-REPORT", bucket: "NO-REPORT", excerpt: "" });
    continue;
  }
  const excerpt = (r.reason || "").split("\n")[0].slice(0, 120).replace(/\t/g, " ");
  rows.push({ file, stem, carrier, result: r.result, bucket: bucketOf(r), excerpt });
}

if (onlyBucket) {
  const hits = rows.filter((x) => x.bucket === onlyBucket);
  // Paths, not stems: this output is meant to be fed straight to wie_validate.
  for (const h of hits) console.log(h.file);
  console.error(`# ${hits.length} file(s) in bucket ${JSON.stringify(onlyBucket)} (input: ${reportsDir})`);
  process.exit(0);
}

const mtimes = readdirSync(reportsDir)
  .filter((f) => f.endsWith(".json"))
  .map((f) => statSync(path.join(reportsDir, f)).mtime.toISOString().slice(0, 10))
  .sort();

const header =
  `# game_lab census map — regenerate with: node scripts/game-lab-census-map.mjs\n` +
  `# corpus=${corpusDir}  reports=${reportsDir}\n` +
  `# ★reports mtime range: ${mtimes[0]} .. ${mtimes[mtimes.length - 1]}  ← THIS is the date of the verdicts below\n` +
  `# files=${corpus.length}  stems=${new Set(corpus.map((c) => c.stem)).size}  reports=${reports.size}\n` +
  `# ★col 6 is the reason's FIRST line, cut at 120 chars — not a search index.\n` +
  `#   Measured 2026-09-18: one excerpt value covers 99/187 rows across several buckets,\n` +
  `#   164/452 reasons are multi-line, 4 first lines exceed 120 chars. A signature that\n` +
  `#   sits on line 2 cannot be grepped here; re-read the report JSON for those.\n` +
  `file\tstem\tcarrier\tresult\tbucket\tfirst_line_of_reason\n`;
writeFileSync(outFile, header + rows.map((r) => `${r.file}\t${r.stem}\t${r.carrier}\t${r.result}\t${r.bucket}\t${r.excerpt}`).join("\n") + "\n");

const totals = {};
for (const r of rows) totals[r.bucket] = (totals[r.bucket] || 0) + 1;
console.log(`game-lab-census-map: ${rows.length} files -> ${outFile}`);
console.log(`  corpus files ${corpus.length} · stems ${new Set(corpus.map((c) => c.stem)).size} · reports matched ${rows.filter((r) => r.result !== "NO-REPORT").length}`);
console.log(`  ★reports mtime range ${mtimes[0]} .. ${mtimes[mtimes.length - 1]} — the buckets below describe THAT run, not today`);
for (const [k, v] of Object.entries(totals).sort((a, b) => b[1] - a[1])) console.log(`  ${String(v).padStart(4)}  ${k}`);
