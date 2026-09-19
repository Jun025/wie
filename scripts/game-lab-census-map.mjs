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
// ── Staleness: what this tool now says, and what it still cannot reach ───────
// Because no check can ever run here, the only remaining lever is that the tool
// itself gets loud. It does, on three surfaces: a bounded stderr block, a stdout
// line, and two lines in the written TSV (one near the top, one immediately above
// the data). The verdict is CURRENT / STALE / UNMEASURED, and the axis is engine
// commits after the newest report — see the engine-paths block below for why that
// replaced a day threshold, and how the path set is measured rather than listed.
//
// ★It deliberately does NOT change the exit code. Regenerating a map from an OLD
// reports directory is a legitimate, documented use — it is exactly how the two
// columns of a census get compared (`--reports`, below) — and a non-zero rc would
// break that caller to warn it about something it already knows.
//
// ★★The hole that remains, stated rather than papered over: a reader who queries
// the TSV with `awk -F'\t' '$6 ~ /…/'` never sees a comment line, so none of the
// three surfaces reaches them. Closing that would mean putting the marker in the
// ROWS, which corrupts the data for every consumer to warn about one. The honest
// position is that this covers the reader who opens the file and the caller who
// runs the tool, and not the reader who greps past the header.
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
import { execFileSync } from "node:child_process";
import path from "node:path";
import { cargoMetadata, workspaceRelative } from "./cargo-metadata.mjs";

// ── Staleness is measured against the ENGINE, not against a calendar ─────────
// The proposal that asked for this left the threshold open ("a reason for N is
// needed separately"). Measured on 2026-09-19 over the last 90 days, against the
// path set cargo reports (below): commits touching it land on 56 of 90 days (165
// commits), and the gaps between consecutive engine-active days are p50 1 day,
// p90 3, MAX 6. ★The calendar is the wrong axis: the question a reader actually
// has is "could the verdicts below have changed since they were taken", and that
// is answered exactly by counting engine commits after the newest report.
// For the July baseline the answer is not a threshold call — it is 147 commits.
//
// ★Those four numbers REPLACED a set measured against the old literal globs
// (65/90 days, 304 commits, p50 1 / p90 2 / MAX 4, July 263) and they are lower
// on purpose: the literal also matched PRE-RENAME directory names (`wie_lgt/`,
// `wie_wipi_java/`, …) and two base-swap orphans, so 43 of 116 commits in one
// sampled window — 37% — came from paths cargo does not build. Asking cargo means
// asking about TODAY's layout, and today's layout cannot see history filed under
// yesterday's names. ★The cost is depth, not safety: a rename commit is recorded
// at the NEW path too (verified on `a56e72f4`, the `wie_web -> wie_featurephone`
// rename — it is counted by `wie_featurephone` as well as by `wie_web`), so a
// report older than a rename still sees at least that commit and can never read
// CURRENT because of it. What is lost is how far back the count reaches, which is
// informational; the verdict is not.
//
// Wall-clock age is still printed, because it is what a human recognises and
// because it is the only thing left when git or cargo cannot answer.
// ★The crate list is ASKED OF CARGO, not written down here. It used to be the literal
// `["wie_*", "wie-*", …]`, and the two globs are themselves the scar: this tree renamed
// `wie_web/ -> wie_featurephone/` (`a56e72f4`) and moved crates to hyphenated names, so a
// second glob had to be bolted on. Measured 2026-09-19 against `cargo metadata --no-deps`,
// the literal missed FOUR real source locations — `src` and `tests` (the ROOT is itself a
// package, `[package] name = "wie"`), `test-utils/Cargo.toml` and `test-utils/src` — i.e.
// 1 of 18 workspace members plus the root package's own sources. `checker-census.mjs` was
// bitten by exactly the same path-guess and reached the same answer: ask cargo.
//
// ★The leak has NOT yet produced a wrong verdict, and saying so is the honest grade: over
// the last 90 days, 9 commits touched those four paths and ALL 9 also touched a path the
// literal covered, so the count never went silent. This is prevention, not a live bug — and
// it is free, because widening to the measured set changes 0 of those 90 days' verdicts.
//
// ★When cargo cannot answer, this returns null and the verdict becomes UNMEASURED. It does
// NOT fall back to a literal list: a fallback list is a second copy of the truth, and the
// day it drifts it drifts SILENTLY toward "CURRENT" — the one direction this whole file
// exists to forbid. Frequency, measured rather than assumed: this generator has 0 callers
// (`checker-census.mjs`) and 0 references in `.github/` — it only runs by hand, on a machine
// that holds the git-ignored `game_lab/` corpus, i.e. a checkout of this Rust workspace. The
// environment where cargo is missing and this script is running is not one we could find.
// `Cargo.lock` and `data/` stay literal because cargo cannot report them: neither is a
// workspace member, and both are fixed repo-level paths that no rename touches.
const REPO_LEVEL_ENGINE_PATHS = ["Cargo.lock", "data"];

function enginePathsFromCargo() {
  const meta = cargoMetadata(process.cwd());
  if (!meta) return null;
  const rel = workspaceRelative(meta);
  const out = new Set(REPO_LEVEL_ENGINE_PATHS);
  for (const pkg of meta.packages) {
    const manifest = rel(pkg.manifest_path);
    if (manifest) out.add(manifest);
    for (const t of pkg.targets ?? []) {
      const src = rel(t.src_path);
      // ★Target source DIRECTORIES, not the package directory. The root package's directory
      // is the repo root, and passing "." to `git rev-list -- <paths>` would count EVERY
      // commit as an engine commit — a staleness check that always says STALE is a staleness
      // check nobody reads.
      if (src) out.add(path.dirname(src));
    }
  }
  out.delete(".");
  return out.size > REPO_LEVEL_ENGINE_PATHS.length ? [...out].sort() : null;
}

// ★A bare local date is ambiguous the moment the table leaves this machine: the SAME instant
// renders as three different days under three timezones (measured 2026-09-19 on one report
// file — `Asia/Seoul` 2026-09-19, `UTC` 2026-09-18, `America/Los_Angeles` 2026-09-18), and
// the header says only "LOCAL", so a reader comparing two columns cannot tell whether the
// input moved or the machine did. Printing the offset makes the label verifiable instead of
// merely asserted. ★The previous failure here was the opposite one and is already fixed
// (`3b73c63c` replaced `mtime.toISOString()`, which printed UTC under a LOCAL label); this
// closes the half that fix left open rather than re-fixing it.
const tzOffset = (d) => {
  const mins = -d.getTimezoneOffset(); // minutes EAST of UTC; per-date, so DST is handled
  const sign = mins < 0 ? "-" : "+";
  const abs = Math.abs(mins);
  return `${sign}${String(Math.floor(abs / 60)).padStart(2, "0")}:${String(abs % 60).padStart(2, "0")}`;
};
const localDay = (d) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}${tzOffset(d)}`;

// ★Returns {ok:false} rather than 0 when git OR cargo cannot answer. "Could not
// measure" must never render as "current" — that is the failure shape this whole
// lineage keeps naming, and it is why the caller prints UNMEASURED and not CURRENT.
// ★`why` is carried out so the warning can name the tool that went missing: "could not
// measure" with no cause is a message a reader cannot act on, and the two causes have
// different fixes (run inside the repo vs install the toolchain).
function engineCommitsSince(epochMs) {
  const enginePaths = enginePathsFromCargo();
  if (!enginePaths) return { ok: false, why: "cargo did not answer" };
  const opts = { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] };
  try {
    const count = execFileSync(
      "git",
      ["rev-list", "--count", `--since=${new Date(epochMs).toISOString()}`, "HEAD", "--", ...enginePaths],
      opts,
    ).trim();
    const last = execFileSync("git", ["log", "-1", "--format=%h %cs", "--", ...enginePaths], opts).trim();
    if (!/^\d+$/.test(count)) return { ok: false, why: "git did not answer" };
    return { ok: true, commits: Number(count), last, paths: enginePaths.length };
  } catch {
    return { ok: false, why: "git did not answer" };
  }
}

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

// ── The staleness verdict, computed BEFORE --bucket exits ───────────────────
// It is computed here and not next to the header because `--bucket` is the mode
// that feeds paths straight into a re-run: emitting a verdict only on the
// table-writing path would leave the machine-consumed path silent, which is the
// louder half of the problem.
//
// ★Dates are LOCAL, not `toISOString()`. The line this replaces said it was "THE
// date of the verdicts" while printing UTC, so a directory written at 05:26 KST
// reported the previous day — measured 2026-09-19, `reports-2026-09-19` printed as
// `2026-09-18`. A staleness defence that is itself a day out is worse than none.
const reportMtimes = readdirSync(reportsDir)
  .filter((f) => f.endsWith(".json"))
  .map((f) => statSync(path.join(reportsDir, f)).mtime.getTime())
  .sort((a, b) => a - b);

// An empty reports dir is a real state (a partial re-census in progress), and the
// previous code rendered it as `undefined .. undefined`.
const haveMtimes = reportMtimes.length > 0;
const newestMs = haveMtimes ? reportMtimes[reportMtimes.length - 1] : null;
const rangeStr = haveMtimes
  ? `${localDay(new Date(reportMtimes[0]))} .. ${localDay(new Date(newestMs))}`
  : "(no reports in that directory)";
const ageDays = haveMtimes ? (Date.now() - newestMs) / 86_400_000 : null;
const engine = haveMtimes ? engineCommitsSince(newestMs) : { ok: false };

const verdict = !haveMtimes || !engine.ok ? "UNMEASURED" : engine.commits > 0 ? "STALE" : "CURRENT";
const verdictLine =
  verdict === "STALE"
    ? `★★ STALE — the engine moved ${engine.commits} commit(s) after these verdicts were taken (newest report ${ageDays.toFixed(1)} days old; last engine commit ${engine.last})`
    : verdict === "CURRENT"
      ? `CURRENT — no engine commit after the newest report (${ageDays.toFixed(1)} days old)`
      : `★★ UNMEASURED — could not compare against the engine${haveMtimes ? ` (${engine.why ?? "no answer"})` : " (no reports to date)"}. Do NOT read this as current.`;

// stderr, loud and bounded, on both paths. Silent on CURRENT so that the one
// noisy case stays legible; a warning that fires every time gets scrolled past.
if (verdict !== "CURRENT") {
  console.error(`\n  ┌─ game-lab-census-map: ${verdict}`);
  console.error(`  │ ${verdictLine}`);
  console.error(`  │ reports=${reportsDir}  range(local)=${rangeStr}`);
  // ★No file is named here on purpose: the re-census runner lands in a SIBLING
  // round, and a warning that points at a path this tree may not have is worse
  // than one that describes the action.
  console.error(`  │ These buckets describe THAT run, not today. Re-validate the corpus and`);
  console.error(`  │ point --reports at the fresh directory.`);
  console.error(`  └─ (this does not change the exit code — comparing an OLD column against a`);
  console.error(`     new one is a legitimate use, so staleness warns and never blocks)\n`);
}

if (onlyBucket) {
  const hits = rows.filter((x) => x.bucket === onlyBucket);
  // Paths, not stems: this output is meant to be fed straight to wie_validate.
  for (const h of hits) console.log(h.file);
  console.error(`# ${hits.length} file(s) in bucket ${JSON.stringify(onlyBucket)} (input: ${reportsDir}) — ${verdict}`);
  process.exit(0);
}

const header =
  `# game_lab census map — regenerate with: node scripts/game-lab-census-map.mjs\n` +
  `# corpus=${corpusDir}  reports=${reportsDir}\n` +
  `# ${verdictLine}\n` +
  // ★The UTC offset is part of the value, not decoration — see the `tzOffset` block. A
  // table written before 2026-09-19 carries a BARE date on this line; that is the older
  // format, not a different input. Do not read `2026-09-18` vs `2026-09-18+09:00` as "the
  // reports moved" when diffing an old column against a new one.
  `# ★reports mtime range (local, with UTC offset): ${rangeStr}  ← THIS is the date of the verdicts below\n` +
  `# files=${corpus.length}  stems=${new Set(corpus.map((c) => c.stem)).size}  reports=${reports.size}\n` +
  `# ★col 6 is the reason's FIRST line, cut at 120 chars — not a search index.\n` +
  `#   Measured 2026-09-18: one excerpt value covers 99/187 rows across several buckets,\n` +
  `#   164/452 reasons are multi-line, 4 first lines exceed 120 chars. A signature that\n` +
  `#   sits on line 2 cannot be grepped here; re-read the report JSON for those.\n` +
  // ★Repeated immediately above the data, because that is the last line a reader
  // sees before the rows start. It does NOT reach a reader who greps column 6 —
  // `awk -F'\t' '$6 ~ /…/'` never matches a comment line. That hole is real and is
  // named in the block at the top of this file rather than papered over here.
  `# ★${verdict}${verdict === "STALE" ? ` — ${engine.commits} engine commit(s) newer than this table` : ""}\n` +
  `file\tstem\tcarrier\tresult\tbucket\tfirst_line_of_reason\n`;
writeFileSync(outFile, header + rows.map((r) => `${r.file}\t${r.stem}\t${r.carrier}\t${r.result}\t${r.bucket}\t${r.excerpt}`).join("\n") + "\n");

const totals = {};
for (const r of rows) totals[r.bucket] = (totals[r.bucket] || 0) + 1;
console.log(`game-lab-census-map: ${rows.length} files -> ${outFile}`);
console.log(`  corpus files ${corpus.length} · stems ${new Set(corpus.map((c) => c.stem)).size} · reports matched ${rows.filter((r) => r.result !== "NO-REPORT").length}`);
console.log(`  ${verdictLine}`);
console.log(`  ★reports mtime range (local, with UTC offset) ${rangeStr}`);
for (const [k, v] of Object.entries(totals).sort((a, b) => b[1] - a[1])) console.log(`  ${String(v).padStart(4)}  ${k}`);
