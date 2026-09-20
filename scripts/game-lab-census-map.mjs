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
// four surfaces reaches them. Closing that would mean putting the marker in the
// ROWS, which corrupts the data for every consumer to warn about one.
//
// ★What changed 2026-09-19 is that the query itself now has a home inside the
// tool — `--signature <regex>`, which greps the same column and prints the same
// verdict block that `--bucket` does. That does not shrink the hole for someone
// who still runs the awk (nothing can, short of corrupting the rows); it means
// the documented path is no longer the silent one. The honest position is
// unchanged for raw-awk readers and improved for everyone who asks the tool.
//
// ── Where a verdict comes from: summary.tsv first, <stem>.json second ────────
// ★The `<stem>.json` layer LOSES DATA and always has. Measured 2026-09-19 (and
// again by this round): the corpus is 187 files over 184 stems — `놈3`,
// `다크슬레이어2` and `이노티아연대기2` each exist under two carriers. The
// re-census runner writes `<stem>.json` because that is the key this generator
// used to look up, so the second carrier's run OVERWRITES the first's, and this
// generator then attributed the surviving verdict to BOTH files. Three rows of
// every map were a copy of another row rather than a measurement.
//
// The runner already writes the lossless thing next to it: `summary.tsv`, one row
// per FILE keyed by PATH. So the fix is precedence, not a new artifact — read
// `summary.tsv` when it is there, fall back to `<stem>.json` when it is not.
//
// ★The fallback is NOT vestigial and must not be deleted. The proposal behind
// this change said the July baseline `game_lab/reports/` has no path key; measured
// 2026-09-19 that is wrong in letter and right in effect — the directory DOES hold
// a `summary.tsv` (73 rows, written 2026-07-01), but its `file` column is bare
// BASENAMES, so it cannot be joined to a corpus path and is not accepted as one.
// All 187 July verdicts therefore come from `<stem>.json`, and deleting that layer
// would make the column this whole lineage compares against unreadable.
//
// ★Because the two inputs can disagree, WHICH ONE WON is printed rather than left
// to be re-derived: per row in column 7, and as a total in the header, on stdout,
// and on the `--bucket` / `--signature` stderr line.
//
// ★What this does NOT repair: the July column is still stem-keyed, so those three
// stems carry one verdict across two files THERE, for good — the losing run was
// never written down and cannot be recovered. Comparing July against a fresh
// column therefore leaves exactly those rows asymmetric, and the header says so
// when the corpus contains colliding stems. Fixing the future is all this can do.
//
// ★And column 6 does not mean quite the same thing on both paths. The runner
// flattens newlines to spaces before it writes `summary.tsv`, so a summary-sourced
// excerpt is the first 120 chars of the WHOLE reason, while a json-sourced one is
// the first 120 chars of the FIRST LINE. Identical for a single-line reason;
// different — and, for signature search, more inclusive — for a multi-line one.
//
// ── Usage ────────────────────────────────────────────────────────────────────
//   node scripts/game-lab-census-map.mjs                       # write the TSV, print totals
//   node scripts/game-lab-census-map.mjs --bucket unimpl-stub  # file paths, one per line
//   node scripts/game-lab-census-map.mjs --signature 'no frame rendered'   # ditto, by col 6
//   node scripts/game-lab-census-map.mjs --reports <dir> --corpus <dir> --out <file>
//
// ★`--bucket` takes a BUCKET, and the buckets are exactly the left column of
// RULES plus PASS / UNCLASSIFIED / NO-REPORT. It is not a free-text search: a
// failure *signature* like `no frame rendered` lives INSIDE `UNCLASSIFIED`, so
// asking for it returns `# 0 file(s)` with rc=0 — no error, no warning. An
// earlier revision of this block advertised exactly that, which is the "absence
// reads as a pass" shape this repo keeps naming.
//
// ★That is what `--signature <regex>` is for, and it exists because the documented
// alternative bypassed every warning this tool emits. The one-liner this block
// used to send readers to —
//
//   awk -F'\t' '$6 ~ /no frame rendered/ {print $1}' game_lab/census-map.tsv
//
// — is still correct awk, but measured 2026-09-19 it prints 1 path and ZERO of
// the four staleness surfaces, because all four are comment lines and `$6 ~ /…/`
// cannot match a comment. The likeliest query was the one path that saw none of
// the warnings. Putting the marker in the ROWS was priced and rejected (it
// corrupts the data for every consumer to warn about one), so the remaining move
// was to bring the query inside the tool, where `--bucket` already demonstrates
// that a query and its warnings travel together.
//
// ★The cost, stated rather than discovered later: this makes a classifier into a
// query tool, so its answers now carry the tool's authority. Column 6 does not
// deserve that authority — see below — which is why `--signature` prints the
// limitation on every call instead of only in this comment. The scope stops here:
// one regex against one column, no joins, no second index.
//
// ★Column 6 is the reason's FIRST LINE truncated to 120 chars, and that is not a
// general search index: measured 2026-09-18 and re-measured 2026-09-19 (identical
// on all three), a single excerpt value (`tick error during 'boot': Fatal error: `)
// covers 99 of 187 rows and spans several buckets, 164 of 452 reports have a
// multi-line reason, and 4 first lines exceed 120 chars. So a signature that sits
// on line 2 is invisible to that grep. It works for `no frame rendered` because
// that reason is a single 36-char line. ★`--signature` recomputes those three
// numbers against the input it was actually given rather than quoting these.
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
// needed separately"). Measured on 2026-09-20 over the last 90 days, against the
// path set this file counts (cargo's, PLUS the pre-rename names derived below):
// commits touching it land on 65 of 90 days (294 commits), and the gaps between
// consecutive engine-active days are p50 1 day, p90 2, MAX 4. ★The calendar is
// the wrong axis: the question a reader actually has is "could the verdicts below
// have changed since they were taken", and that is answered exactly by counting
// engine commits after the newest report. For the July baseline the answer is not
// a threshold call — it is 259 commits.
//
// ★Those numbers have now moved TWICE, in opposite directions, and both moves are
// worth keeping because the second one repairs the cost the first one accepted.
//
//   literal globs (until 2026-09-19)   65/90 days · 304 commits · p50 1 / p90 2 / MAX 4 · July 269
//   cargo only    (2026-09-19)         56/90 days · 168 commits · p50 1 / p90 3 / MAX 6 · July 153
//   cargo + derived historic (here)    65/90 days · 294 commits · p50 1 / p90 2 / MAX 4 · July 259
//
// (All three rows are re-measured on today's HEAD, so the first two sit a little above
// what `docs/report/0182` published on 2026-09-19 — July 269 vs its 263 and 153 vs its
// 147 are six days of landings. Its 90-day cells are 304 and 165/168. Do not read the
// drift between the two documents as a disagreement; re-measure instead.)
//
// The middle row dropped the literal because it also matched PRE-RENAME directory
// names (`wie_lgt/`, `wie_wipi_java/`, …) and dead base-swap orphans, so 43 of 116
// commits in one sampled window — 37% — came from paths cargo does not build. That
// was right. What it also threw away was every commit filed under a name this tree
// has since renamed, and the price was measured and accepted as "depth, not safety":
// a rename commit is recorded at the NEW path too (verified on `a56e72f4`, the
// `wie_web -> wie_featurephone` rename), so a report older than a rename still sees
// at least that commit and can never read CURRENT because of it.
//
// ★The third row buys the depth back WITHOUT the literal, because git records the
// renames itself — see `historicEnginePaths` below. The proposal that asked for this
// (`2026-09-19-census-map-provenance-offset-and-engine-paths#p1`) predicted the
// opposite ("adding the old names IS the literal coming back, and git cannot --follow
// several paths, so tracking them automatically is not cheap either"); measured, both
// halves are false. Asking git for its own rename records is a second DERIVATION, not
// a second copy — nobody types a crate name — and it costs one `git log` pass.
//
// ★The recovered set is a strict SUBSET of what the literal caught. Measured over the
// last 90 days: literal 304, this 294, commits this counts that the literal did not
// **0**, commits the literal counted that this does not **10**. Those 10 were read one
// by one rather than characterised in bulk, and they are four things, not one:
//
//   4  `wie-web/src`'s TypeScript/CSS/HTML half — `0182` limits[]: cargo reports only
//      `src/rust`, and narrowing to it was that round's decision
//   4  `wie_jvm_support/tests`, `wie_midp/tests` — the base-swap orphans `0182` named as
//      what the literal WRONGLY bit; 3 files under directories with no `Cargo.toml`
//   1  `wie_tauri` — `0182` limits[]: `wie_app` is held out of the workspace by `7a439bd0`
//   1  ★`bfa1ef2f`, and this one is NOT a narrowing — it is a MERGE, and `git rev-list`
//      drops a merge that is TREESAME to a parent for the given paths. The coarse glob
//      `wie_*` is not TREESAME there and `wie_cli/src/bin` is, so the difference is
//      pathspec granularity on merge simplification, not coverage. Nobody predicted this
//      one; it is written down because "all 10 are deliberate exclusions" was the
//      tempting summary and it would have been false.
//
// ★"Strict subset" is the claim to re-run if this changes — a NON-zero "counts that the
// literal did not" is the signal that this has started biting something new, and it is
// the cheap check (`git rev-list` both sets, `comm`).
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

// ★The names this tree used to file the engine under, ASKED OF GIT rather than written
// down. `git log --diff-filter=R -M` is git's own record of every rename it detected; a
// pair whose DESTINATION is inside the cargo-derived set names a path that WAS the engine.
// Run to a fixed point, because this tree has renamed the same crate more than once
// (`crates/wie_vendor_ktf` → `wie_ktf` → `wie-ktf`) — measured, it converges in 2 passes over
// 998 pairs and reaches 285 candidates, 280 of which survive the HEAD filter below. They span
// four naming eras (`crates/wie_*`, `wie_impl_*`/`wie_vendor_*`, `wie_*`, today's `wie-*`).
// No list of that shape would have been written by hand, and none of it is typed here.
//
// ★A `--diff-filter=R` scan is possible only WITHOUT a pathspec. With one, git filters the
// diff before rename detection can pair the halves, so a rename from outside the pathspec
// into it reads as an add: asking `git log --diff-filter=R -- <the 45 cargo paths>` returns
// 6 pairs and misses `wie_web -> wie_featurephone` entirely (measured). That is the same
// trap as `--follow`, and it is why this scans everything and filters afterwards.
//
// ★A candidate that STILL EXISTS at HEAD is dropped, and that rule is the whole reason this
// does not re-open what `0182` closed. A surviving name is not history — it is a live path
// that cargo deliberately does not build, which is exactly the class the literal was removed
// for biting. Measured: 5 of 285 candidates survive (`src`, `wie_cli/src` — already cargo
// paths, so dropping them changes nothing — plus `test_data`, `wie_jvm_support`, `wie_midp`),
// and the three real drops cost 11 commits of July depth, 270 → 259.
//
// ★It is a PATH test, not a directory-name test, and the difference is load-bearing:
// `wie_jvm_support/` survives only as 3 orphaned `tests/*.rs` files, while
// `wie_jvm_support/src/` is genuinely gone — so the source half is counted as history and the
// surviving test half is not. Judging the whole directory by any surviving descendant would
// throw away 11 commits that really are pre-rename engine work.
//
// ★It cannot flip a verdict on a fresh table: every path here is gone from HEAD, so a commit
// can only touch one by predating the rename that removed it. Measured against the cargo-only
// count — since 2026-09-18 delta 0, 09-16 0, 09-15 +1, 09-10 +3, 09-01 +34, 08-01 +55,
// 07-01 +105. The correction grows with how far back the table reaches, which is what "depth"
// means. ★Stated as a measurement and not as a proof, because the same "renames are behind
// us" reasoning is what the `--since` note below records as having been measured FALSE.
//
// ★Returns null when git will not answer, and the caller says so. Degrading silently to the
// cargo-only count would make the number SMALLER, i.e. bias toward CURRENT, and that is the
// one direction this file exists to forbid.
function historicEnginePaths(current) {
  const opts = { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"], maxBuffer: 1 << 28 };
  const under = (p, set) => set.some((e) => p === e || p.startsWith(e + "/"));
  let raw;
  try {
    // ★The scan is UNBOUNDED, and the obvious optimisation was tried and measured wrong.
    // Passing the count's own `--since` looks exactly equivalent — a path renamed away before
    // the window opened should hold no commit inside it — and it is several times cheaper
    // (5 runs each at loadavg ~75: unbounded 0.35–1.15 s, bounded to July 0.06–0.19 s; the
    // same pair at loadavg ~205 read 2.7–5.0 s and 0.9–1.5 s, so read the RATIO, not the
    // seconds). It is not equivalent: measured against the cargo-only count, the correction
    // it recovers falls from +3 to +1 (since 2026-09-10) and from +34 to +2 (since
    // 2026-09-01), while the two older windows are unaffected. The reasoning fails because
    // history here is not linear — this tree took an upstream base swap, so a commit
    // reachable from HEAD can carry a commit-date INSIDE the window while sitting on a
    // lineage where the rename had not happened. `--since` filters by date; ancestry does not
    // follow. Do not re-apply that bound without re-running those two deltas.
    raw = execFileSync("git", ["log", "--diff-filter=R", "-M", "--name-status", "--format=", "HEAD"], opts);
  } catch {
    return null;
  }
  const pairs = [];
  for (const line of raw.split("\n")) {
    if (!line.startsWith("R")) continue;
    const f = line.split("\t");
    if (f.length >= 3) pairs.push([f[1], f[2]]);
  }
  // ★Fixed point over rename chains, and the ONLY test against the growing set is on the
  // destination. That is what makes the answer independent of the order git printed the pairs
  // in: adding a path can only ever enable more pairs, never disable one, so the loop has a
  // least fixed point and reaches it from any order. It is not a style point — the first
  // version of this also skipped pairs whose SOURCE was already in the set, which is not
  // monotone, and it measurably gave two different answers on one repo: newest-first (git's
  // default) 35 paths / 248 commits, oldest-first 78 / 259. Both orders now give 280 / 259.
  //
  // The 10 is a guard against a cycle in the pair list, not a tuning knob — the loop exits on
  // the first pass that adds nothing.
  //
  // ★Read the COMMIT count, not the path count. The path count is an implementation figure
  // and it moves without the answer moving: capping this loop at one pass leaves it at 280 on
  // git's own ordering, drops it to 125 on the reversed one, and the commit count is 259 in
  // all three — the paths it loses are sub-paths of paths it keeps. The loop is what makes the
  // SET reproducible; it is not what makes the number right.
  const set = [...current];
  const found = new Set();
  for (let i = 0; i < 10; i++) {
    let grew = false;
    for (const [from, to] of pairs) {
      if (!under(to, set)) continue;
      const dir = path.dirname(from);
      if (dir === "." || found.has(dir)) continue;
      found.add(dir);
      set.push(dir);
      grew = true;
    }
    if (!grew) break;
  }
  if (found.size === 0) return [];
  // One `ls-tree`, not one `cat-file` per candidate: this runs on every invocation.
  let live = "";
  try {
    live = execFileSync("git", ["ls-tree", "-r", "--name-only", "HEAD", "--", ...found], opts);
  } catch {
    return null;
  }
  const surviving = live.split("\n").filter(Boolean);
  return [...found].filter((p) => !surviving.some((f) => f === p || f.startsWith(p + "/"))).sort();
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
  // ★The historic names are an ENRICHMENT of the count, not a precondition for it: losing
  // them costs depth, losing cargo costs the set itself. So a null here is carried into the
  // verdict line as "depth unmeasured" rather than collapsing the whole answer to UNMEASURED.
  const historic = historicEnginePaths(enginePaths);
  const countPaths = historic ? [...enginePaths, ...historic] : enginePaths;
  const opts = { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] };
  try {
    const since = `--since=${new Date(epochMs).toISOString()}`;
    const count = execFileSync("git", ["rev-list", "--count", since, "HEAD", "--", ...countPaths], opts).trim();
    // ★The CURRENT paths only. `last` answers "when did the engine last move", and a dead
    // pre-rename path can only ever answer that with a date before its own rename.
    const last = execFileSync("git", ["log", "-1", "--format=%h %cs", "--", ...enginePaths], opts).trim();
    if (!/^\d+$/.test(count)) return { ok: false, why: "git did not answer" };
    // ★Split out rather than folded in, because the two halves answer different questions:
    // `commits` is how much the engine moved, `viaHistoric` is how much of that this table
    // would have MISSED before 2026-09-20 — i.e. how far past a rename boundary it reaches.
    const nowOnly = execFileSync("git", ["rev-list", "--count", since, "HEAD", "--", ...enginePaths], opts).trim();
    return {
      ok: true,
      commits: Number(count),
      last,
      paths: countPaths.length,
      viaHistoric: historic && /^\d+$/.test(nowOnly) ? Number(count) - Number(nowOnly) : null,
      historicPaths: historic ? historic.length : null,
    };
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
const signature = flag("signature", null);

// ★Refused rather than silently ordered. Both are filters over the same rows, so a
// caller passing both has a question this tool cannot answer without guessing which
// one they meant — and guessing would return a subset that looks like an answer.
if (onlyBucket && signature) {
  console.error(`game-lab-census-map: --bucket and --signature are both filters — pass one. (got --bucket ${JSON.stringify(onlyBucket)} --signature ${JSON.stringify(signature)})`);
  process.exit(2);
}

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

// ── The path-keyed input, which is the one that does not lose rows ───────────
// `summary.tsv` is written by `scripts/game-lab-recensus.sh`, one row per file:
//   result \t platform \t file \t reason \t ticks \t paints \t ms
// Only `result` and `file` and `reason` are read here — the rest is the runner's
// telemetry and this tool has no use for it.
//
// ★TWO keys per row, and the second one is not decoration. The exact key is the
// resolved absolute path, which matches when the runner and this tool were told
// the same corpus. They often are not: the runner defaults to `game_lab/broken`
// relative to the repo root while an analyst points this tool at an absolute path
// (that is exactly how this round measured it), and a bare `resolve()` against a
// different cwd then misses every row and falls back to the stem layer — i.e. the
// bug would look fixed while doing nothing. The second key is `<carrier>/<file>`,
// the last two components, which uniquely identifies all 187 files precisely
// BECAUSE the colliding stems sit under different carriers.
//
// ★The false-join it could cause is named rather than hidden: two corpora that
// share a carrier name and a filename would join across them. That needs someone
// to point `--reports` at one corpus's run and `--corpus` at another's, which is
// already a mis-use; the exact key is tried first, so the tail only ever answers
// when the exact key did not.
//
// ★Resolved LEXICALLY (`path.resolve`), never `realpathSync`: two corpus entries
// may legitimately be links onto the same bytes, and collapsing them would merge
// two rows that this whole change exists to keep apart.
const pathKey = (p) => nfc(path.resolve(p));
const tailKey = (p) => nfc(`${path.basename(path.dirname(p))}/${path.basename(p)}`);
const summaryFile = path.join(reportsDir, "summary.tsv");
const byPath = new Map();
const byTail = new Map();
let summaryRows = 0;
if (existsSync(summaryFile)) {
  for (const line of readFileSync(summaryFile, "utf8").split("\n")) {
    if (!line || line.startsWith("result\t")) continue;
    const cols = line.split("\t");
    if (cols.length < 4) continue;
    const [result, , file, reason] = cols;
    if (!file) continue;
    const rec = { result, reason };
    byPath.set(pathKey(file), rec);
    // First writer wins on the tail key: if two rows collapse onto one tail, the
    // exact key is the only correct answer for both and this ambiguous fallback
    // must not pick a side by ordering.
    const t = tailKey(file);
    byTail.set(t, byTail.has(t) ? null : rec);
    summaryRows++;
  }
}

const bucketOf = (r) => {
  if (r.result === "PASS") return "PASS";
  const reason = r.reason || "";
  for (const [name, re] of RULES) if (re.test(reason)) return name;
  return "UNCLASSIFIED";
};

const SRC_SUMMARY = "summary.tsv";
const SRC_JSON = "stem.json";
const SRC_NONE = "none";

// Precedence, in one place so it can be read and changed as one thing.
// ★`byTail` can hold `null` for an ambiguous tail; `??` treats that as "no answer"
// and falls through to the stem layer, which is the intended behaviour.
const lookup = (file, stem) => {
  const exact = byPath.get(pathKey(file));
  if (exact) return { r: exact, source: SRC_SUMMARY };
  const tail = byTail.get(tailKey(file));
  if (tail) return { r: tail, source: SRC_SUMMARY };
  const js = reports.get(stem);
  if (js) return { r: js, source: SRC_JSON };
  return { r: null, source: SRC_NONE };
};

const rows = [];
for (const { file, stem } of corpus) {
  const { r, source } = lookup(file, stem);
  const carrier = path.basename(path.dirname(file));
  if (!r) {
    rows.push({ file, stem, carrier, result: "NO-REPORT", bucket: "NO-REPORT", excerpt: "", source });
    continue;
  }
  const excerpt = (r.reason || "").split("\n")[0].slice(0, 120).replace(/\t/g, " ");
  rows.push({ file, stem, carrier, result: r.result, bucket: bucketOf(r), excerpt, source });
}

const sourceTotals = {};
for (const r of rows) sourceTotals[r.source] = (sourceTotals[r.source] || 0) + 1;
// ★A summary.tsv whose rows match NOTHING is announced, not left to be inferred
// from a total. Measured 2026-09-19: the July baseline DOES contain a
// `summary.tsv` (73 rows) — the proposal behind this change assumed it did not —
// but its `file` column holds BARE BASENAMES (`(SKT) 교실이데아.zip`), so no row
// joins and all 187 verdicts correctly come from the stem layer. That is the
// right outcome and it must not be silent: "I read an input and used none of it"
// is indistinguishable from "the input was absent" unless the tool says so.
//
// ★Basenames are deliberately NOT accepted as a third key. A basename cannot tell
// the colliding stems apart either, so honouring it would relabel a lossy row as
// `summary.tsv` — and it would re-bucket 73 rows of the July column, which is a
// retroactive change to the baseline this lineage compares against.
const summaryUsed = sourceTotals[SRC_SUMMARY] || 0;
const sourceLine =
  `inputs: ` +
  [SRC_SUMMARY, SRC_JSON, SRC_NONE]
    .filter((k) => sourceTotals[k])
    .map((k) => `${k}=${sourceTotals[k]}`)
    .join(" · ") +
  (!summaryRows
    ? ` (no summary.tsv in ${reportsDir} — stem-keyed fallback only)`
    : summaryUsed
      ? ` (summary.tsv had ${summaryRows} row(s))`
      : ` ★(summary.tsv has ${summaryRows} row(s) and NONE of them matched a corpus file — its 'file' column is not corpus-relative; the July baseline stores bare basenames. Falling back to the stem layer, which loses one verdict per colliding stem.)`);

// ── The asymmetry this cannot repair, printed only when it applies ───────────
// Stem collisions are a property of the CORPUS, so they are computed here and not
// from the inputs: the warning is about what a stem-keyed column can express, and
// that is true whether or not today's run happens to have a summary.tsv.
const stemCounts = new Map();
for (const c of corpus) stemCounts.set(c.stem, (stemCounts.get(c.stem) || 0) + 1);
const collidingStems = [...stemCounts].filter(([, n]) => n > 1).map(([s]) => s).sort();
const asymmetryLine = collidingStems.length
  ? `★${collidingStems.length} stem(s) exist under 2+ carriers (${collidingStems.length * 2 <= 8 ? collidingStems.join(", ") : `${collidingStems.slice(0, 3).join(", ")}, …`}). A stem-keyed column gives every copy ONE verdict; only the summary.tsv path tells them apart. Rows from a stem-keyed input are therefore not comparable 1:1 with rows from a path-keyed one — and the July baseline is stem-keyed here (its summary.tsv is basename-keyed, which cannot tell them apart either), so that asymmetry is permanent for it.`
  : null;

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
// ★What the count's DEPTH is worth, said next to the count. A number that silently stops at
// the last rename reads as an equality when it is a floor, and the reader has no way to tell
// the two apart — which is the whole of the proposal this answers. Three states, never
// silence: `viaHistoric` non-zero names how much of the count lives past a rename boundary,
// zero says no commit in this window touches one, and null says git would not hand over its
// rename records, so the count IS a floor.
//
// ★The zero branch says "none of them touch", NOT "this table is newer than every rename".
// The second is an inference the measurement does not carry — what was measured is that no
// commit inside THIS window touched a pre-rename path, which is weaker and is the thing a
// reader can check.
const depthNote =
  !engine.ok || engine.historicPaths === null
    ? " · ★depth UNMEASURED — git did not hand over its rename records, so this count stops at the newest rename and is a FLOOR"
    : engine.viaHistoric
      ? ` · ${engine.viaHistoric} of them found under ${engine.historicPaths} pre-rename path(s) this tree no longer uses`
      : ` · none of them touch any of the ${engine.historicPaths} pre-rename path(s) checked`;
const verdictLine =
  verdict === "STALE"
    ? `★★ STALE — the engine moved ${engine.commits} commit(s) after these verdicts were taken (newest report ${ageDays.toFixed(1)} days old; last engine commit ${engine.last})${depthNote}`
    : verdict === "CURRENT"
      ? // ★On CURRENT the depth note is attached ONLY when depth could not be measured. A
        // CURRENT that reaches every path says nothing interesting about renames (there are
        // no commits to attribute), but a CURRENT computed from a count that is a FLOOR is a
        // claim the floor cannot support, and that has to be visible.
        `CURRENT — no engine commit after the newest report (${ageDays.toFixed(1)} days old)${
          engine.historicPaths === null ? depthNote : ""
        }`
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

// ── What column 6 can and cannot answer, measured on THIS input ─────────────
// ★Recomputed per run rather than quoting the numbers in the header block: the
// point of printing a limitation next to an answer is that it describes the
// answer, and a frozen number stops doing that the first time the corpus moves.
// The multi-line and over-120 counts are properties of the JSON reports, so they
// are reported as unmeasurable when the input has none — which is exactly the
// case a summary.tsv-only directory presents, and saying "0" there would be a lie
// in the direction this file forbids.
function columnSixLimits() {
  const counts = new Map();
  for (const r of rows) if (r.excerpt) counts.set(r.excerpt, (counts.get(r.excerpt) || 0) + 1);
  let topN = 0;
  for (const n of counts.values()) if (n > topN) topN = n;

  let multi = 0;
  let over = 0;
  for (const r of reports.values()) {
    const reason = r.reason || "";
    if (reason.replace(/\n+$/, "").includes("\n")) multi++;
    if (reason.split("\n")[0].length > 120) over++;
  }
  const jsonPart = reports.size
    ? `${multi}/${reports.size} json reason(s) are multi-line · ${over} first line(s) exceed 120 chars`
    : `multi-line and >120 counts unmeasurable here (0 json report(s) in this input)`;
  return `# ★LIMIT col 6 is one truncated line, not a search index — measured on THIS input: top excerpt covers ${topN}/${rows.length} row(s) · ${jsonPart}. A signature on line 2 cannot match; re-read the report JSON for those.`;
}

if (onlyBucket) {
  const hits = rows.filter((x) => x.bucket === onlyBucket);
  // Paths, not stems: this output is meant to be fed straight to wie_validate.
  for (const h of hits) console.log(h.file);
  console.error(`# ${hits.length} file(s) in bucket ${JSON.stringify(onlyBucket)} (input: ${reportsDir}) — ${verdict}`);
  console.error(`# ${sourceLine}`);
  process.exit(0);
}

if (signature) {
  // ★Invalid regex is an exit-2, not a literal-string fallback. Falling back would
  // answer a different question than the one asked and look like it worked.
  let re;
  try {
    re = new RegExp(signature);
  } catch (e) {
    console.error(`game-lab-census-map: --signature ${JSON.stringify(signature)} is not a valid regex (${e.message})`);
    process.exit(2);
  }
  const hits = rows.filter((x) => re.test(x.excerpt));
  // Same shape as --bucket: paths on stdout, everything else on stderr, rc=0 even
  // for zero hits — "no file matches" is an answer, not an error.
  for (const h of hits) console.log(h.file);
  console.error(`# ${hits.length} file(s) whose col 6 matches /${signature}/ (input: ${reportsDir}) — ${verdict}`);
  console.error(`# ${sourceLine}`);
  console.error(columnSixLimits());
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
  // ★Which input each verdict came from, in total here and per row in column 7.
  // summary.tsv is path-keyed and lossless; <stem>.json gives one verdict to every
  // file sharing a stem. A column built from the second is not row-comparable with
  // one built from the first — see the next line when it is present.
  `# ★${sourceLine}\n` +
  (asymmetryLine ? `# ${asymmetryLine}\n` : "") +
  `# ★col 6 is the reason's FIRST line, cut at 120 chars — not a search index.\n` +
  `#   Measured 2026-09-18, re-measured 2026-09-19 (identical): one excerpt value covers\n` +
  `#   99/187 rows across several buckets, 164/452 reasons are multi-line, 4 first lines\n` +
  `#   exceed 120 chars. A signature that sits on line 2 cannot be grepped here; re-read\n` +
  `#   the report JSON for those. ★\`--signature <regex>\` runs that query inside this tool\n` +
  `#   and recomputes those numbers for the input it was given.\n` +
  `#   ★On summary.tsv-sourced rows col 6 is the first 120 chars of the WHOLE reason: the\n` +
  `#   runner flattens newlines before writing that file, so "first line" is a json-path\n` +
  `#   property only. Identical for single-line reasons.\n` +
  // ★Repeated immediately above the data, because that is the last line a reader
  // sees before the rows start. It does NOT reach a reader who greps column 6 —
  // `awk -F'\t' '$6 ~ /…/'` never matches a comment line. That hole is real and is
  // named in the block at the top of this file rather than papered over here.
  `# ★${verdict}${verdict === "STALE" ? ` — ${engine.commits} engine commit(s) newer than this table` : ""}\n` +
  // ★`source` is APPENDED as column 7. The documented query is `$6 ~ /…/` and
  // `$1` is the path, so columns 1..6 keep their meaning and every existing
  // reader keeps working; a column inserted in the middle would silently
  // re-point both.
  `file\tstem\tcarrier\tresult\tbucket\tfirst_line_of_reason\tsource\n`;
writeFileSync(
  outFile,
  header + rows.map((r) => `${r.file}\t${r.stem}\t${r.carrier}\t${r.result}\t${r.bucket}\t${r.excerpt}\t${r.source}`).join("\n") + "\n",
);

const totals = {};
for (const r of rows) totals[r.bucket] = (totals[r.bucket] || 0) + 1;
console.log(`game-lab-census-map: ${rows.length} files -> ${outFile}`);
console.log(`  corpus files ${corpus.length} · stems ${new Set(corpus.map((c) => c.stem)).size} · reports matched ${rows.filter((r) => r.result !== "NO-REPORT").length}`);
console.log(`  ${verdictLine}`);
console.log(`  ★reports mtime range (local, with UTC offset) ${rangeStr}`);
console.log(`  ★${sourceLine}`);
if (asymmetryLine) console.log(`  ${asymmetryLine}`);
for (const [k, v] of Object.entries(totals).sort((a, b) => b[1] - a[1])) console.log(`  ${String(v).padStart(4)}  ${k}`);
