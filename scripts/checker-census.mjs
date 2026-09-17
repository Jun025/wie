#!/usr/bin/env node
// checker-census.mjs — answers "where does this one run?" for every checker in the tree,
// mechanically, so nobody has to grep it out by hand one file at a time.
//
// ── What this is, and what it deliberately is NOT ────────────────────────────
// ★It is a CENSUS, not a gate. It prints a caller count per artifact and exits 0 even
//   when that count is zero. A zero is not a verdict: this repo ships checkers that are
//   correctly uncalled — scripts/smoke_gate.sh cannot run in CI at all (its corpus is
//   game bytes, Constraint 9), and scripts/check-branch-protection-claim.mjs was BUILT
//   for doc-liveness.yml and then measured out of it (github.token gets 403 on the
//   protection endpoints; the reason is written next to where the step would have gone).
//   Turning either into a red check would be wrong, so the judgement stays with the
//   reader and this file holds none of it. Whether any row deserves action is a separate
//   ticket — see docs/report/0155--2026-09-17--wie-count-checkers-with-only-one-caller.md.
// ★Therefore it has no exemption mechanism. An allow-list only exists to keep a gate
//   quiet, and there is no gate here; adding one would create a second place where the
//   classification lives, which is the failure this repo keeps naming.
//
// ── Why callers are counted from the WIRING side, not by name ────────────────
// ★A predicate that guesses from names (`check-*`) misses every checker that does not
//   follow the convention and miscounts the ones that do — scripts/verify-browser.mjs and
//   scripts/audit-no-leak.sh are checkers; scripts/check-docs-report-serial.mjs is mostly
//   a serial allocator. So the population is "executable artifact" (by location and
//   extension, below) and the caller count comes from reading the things that actually
//   invoke them.
// ★A caller must be an EXECUTABLE position, never prose. AGENTS.md, STATE.md and
//   REPORT.md name these paths dozens of times and run nothing; comments inside scripts
//   do the same (contract-roundtrip.mjs names two test files in comments). Both are
//   stripped, because counting them would report "wired" for things nothing runs — the
//   exact lie this tool exists to remove.
// ★Two callers are invisible to any name match and are declared as rules below instead:
//   `cargo test --all` and `cargo tarpaulin --workspace` reach integration tests by glob,
//   never by name. Without that rule every integration test reads as an orphan, so the
//   rule carries a large share of the output and has to be right.
// ★★WHICH files that glob reaches is asked of `cargo metadata --no-deps`, NOT inferred
//   from the path. The first version of this file inferred it — `^[^/]+/tests/[^/]+\.rs$`
//   — and was wrong in both directions on this tree:
//     · `wie_j2me/`, `wie_jvm_support/`, `wie_midp/` hold a `tests/` directory but NO
//       `Cargo.toml` (the workspace members are the hyphenated `wie-j2me` &c.; these are
//       orphans left by the upstream base swap). cargo compiles none of their 4 files,
//       yet the path rule handed each of them 4 callers and filed them under "2+" — the
//       best-wired bucket. That is exactly the lie the paragraph above says this tool
//       exists to remove, produced by this tool.
//     · the ROOT is itself a package (`[package] name = "wie"`), so `tests/font.rs` is a
//       real target — and the crate-dir prefix in the old population regex excluded it.
//   ⇒ "does cargo know this file as a target" is the only authority for "does cargo run
//   it", it costs one subprocess, and this file already shells out to git. There was no
//   cost argument for guessing.
// ★The price, stated rather than hidden: this census now depends on `cargo`. Where cargo
//   is absent or refuses (offline, no toolchain), the glob coverage CANNOT be measured —
//   and it is reported as UNMEASURABLE, never folded into "0 callers". Folding it would
//   understate callers and over-report orphans, i.e. fail toward a confident wrong number,
//   which is the failure this whole round exists to correct.
//
// Usage: node scripts/checker-census.mjs   (no flags — one output, so there is no mode
//   whose numbers someone quotes without saying which mode produced them)
// Exit: 0 = census printed · 2 = could not measure (no git / no files / no cargo target
//   list). There is no 1: a census has no failing state, so it cannot fail open.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

function die(msg) {
  console.error(`checker-census: ${msg}`);
  process.exit(2);
}

// ── Population ──────────────────────────────────────────────────────────────
// Location + extension, not name. `tests/**.rs` under a crate directory OR at the repo
// root — the root is a package too (`[package] name = "wie"`), so `tests/font.rs` is a
// target and an earlier crate-dir-only regex silently dropped it. Files under
// `tests/support/` stay in the population on purpose: they are not targets themselves,
// and the by-name pass shows how they are reached (`#[path]`), which is the useful answer.
// Data files (smoke_gate_baseline.tsv) are out: they are inputs, not things that run.
const POPULATION = [
  /^scripts\/[^/]+\.(?:sh|mjs|js)$/,
  /^\.github\/scripts\/.+\.(?:sh|mjs|js)$/,
  /^(?:[^/]+\/)?tests\/.+\.rs$/,
];

// Files read for caller evidence. Workflows and package.json are parsed structurally
// (below); the rest are read as comment-stripped source.
const SOURCE_EXT = /\.(?:sh|mjs|js|rs|ts|tsx|toml)$/;
const WORKFLOW = /^\.github\/workflows\/.+\.(?:yml|yaml)$/;
const PKG_JSON = /^(?:[^/]+\/)?package\.json$/;

// ── The glob callers: the INVOCATION is declared, the COVERAGE is measured ──
// `invokes` is a declared rule (which commands start a workspace-wide cargo run).
// `covers` is NOT a rule — it is the integration-test target set cargo itself reports.
// `cargoTestTargets` is null when cargo could not be asked; every consumer below must
// treat null as "unmeasured", never as "covers nothing".
const GLOB_CALLERS = [
  {
    label: "cargo test/tarpaulin over the whole workspace (glob, never by name)",
    invokes: /\bcargo\s+(?:\+\S+\s+)?(?:test|tarpaulin)\b[^\n]*?(?:--all\b|--workspace\b)/,
    // ★asks cargo; returns false for a path cargo does not compile, and the caller of
    //   this function must have already checked `cargoTestTargets !== null`.
    covers: (p) => cargoTestTargets !== null && cargoTestTargets.has(p),
  },
];

let root;
try {
  root = execFileSync("git", ["rev-parse", "--show-toplevel"], { encoding: "utf8" }).trim();
} catch {
  die("not a git work tree — cannot enumerate the population");
}

let tracked;
try {
  tracked = execFileSync("git", ["ls-files", "-z"], { cwd: root, encoding: "utf8" })
    .split("\0")
    .filter(Boolean);
} catch {
  die("git ls-files failed — cannot enumerate the population");
}

// ★MUST come after `root` — an earlier revision put this above it and the TDZ throw was
//   swallowed by the try/catch, so the tool reported `covers 0 artifacts` as if measured.
//   That is the same fail-open this rewrite exists to remove; hence the guard below.
const cargoTestTargets = (() => {
  let raw;
  try {
    raw = execFileSync("cargo", ["metadata", "--no-deps", "--format-version", "1"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      stdio: ["ignore", "pipe", "ignore"],
    });
  } catch {
    return null; // absent / offline / refused — reported, never folded into a count
  }
  try {
    const meta = JSON.parse(raw);
    const prefix = meta.workspace_root.endsWith("/") ? meta.workspace_root : meta.workspace_root + "/";
    const out = new Set();
    for (const pkg of meta.packages ?? [])
      for (const t of pkg.targets ?? [])
        if ((t.kind ?? []).includes("test") && typeof t.src_path === "string" && t.src_path.startsWith(prefix))
          out.add(t.src_path.slice(prefix.length));
    return out.size ? out : null; // an empty set here means the shape changed, not "no tests"
  } catch {
    return null;
  }
})();

const population = tracked.filter((f) => POPULATION.some((re) => re.test(f))).sort();
if (population.length === 0) die("population is empty — the location rules no longer match this tree");

const read = (f) => {
  try {
    return readFileSync(path.join(root, f), "utf8");
  } catch {
    return null;
  }
};

// ── Surface extraction ──────────────────────────────────────────────────────
// Every surface is a list of {line, text} of EXECUTABLE text only.

// "Where does it run" is half the question; "when" is the other half, and it is the half a
// path match cannot answer. .github/workflows/release.yaml names three of these scripts and
// fires on `workflow_dispatch` alone — a caller that never starts by itself. So each
// workflow's top-level `on:` keys are read and printed next to its call sites, from the
// file itself rather than from a list someone has to maintain.
function workflowTriggers(text) {
  const m = /^on:[ \t]*(.*)$\n?([\s\S]*?)(?=^\S|\Z)/m.exec(text);
  if (!m) return "on:?";
  const inline = m[1].trim();
  if (inline && !inline.startsWith("#")) return inline.replace(/[[\]]/g, "");
  const keys = [...m[2].matchAll(/^ {2}([A-Za-z_]+):/gm)].map((k) => k[1]);
  return keys.length ? keys.join(",") : "on:?";
}

// A workflow's executable text is the `run:` scalars. Block scalars (`run: |`) continue
// while the indentation exceeds the run: line's own; shell comments inside them are
// dropped, which is what keeps doc-liveness.yml's `# NOT-RUN:` declarations from reading
// as calls (they name commands precisely because those commands do NOT run).
function workflowSurface(text) {
  const lines = text.split("\n");
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    const m = lines[i].match(/^(\s*)(?:-\s+)?run:[ \t]*(.*)$/);
    if (!m) continue;
    const indent = lines[i].search(/\S/);
    const rest = m[2].trim();
    const blockScalar = rest === "" || /^[|>][+-]?\d*$/.test(rest);
    if (!blockScalar) out.push({ line: i + 1, text: rest });
    if (!blockScalar && rest !== "") continue;
    for (let j = i + 1; j < lines.length; j++) {
      if (lines[j].trim() === "") continue;
      if (lines[j].search(/\S/) <= indent) break;
      if (lines[j].trim().startsWith("#")) continue;
      out.push({ line: j + 1, text: lines[j] });
    }
  }
  return out;
}

// package.json's executable text is the values of `scripts` — the alias layer. A missing
// or unparsable file is reported, never silently treated as "no callers here".
function pkgSurface(f, text) {
  let json;
  try {
    json = JSON.parse(text);
  } catch {
    unreadable.push(`${f} (not valid JSON)`);
    return [];
  }
  const scripts = json?.scripts;
  if (!scripts || typeof scripts !== "object") return [];
  const lines = text.split("\n");
  return Object.entries(scripts).map(([k, v]) => ({
    line: lines.findIndex((l) => l.includes(`"${k}"`)) + 1 || 1,
    text: String(v),
  }));
}

// Source text with FULL-LINE comments blanked and /* */ spans removed. Only full-line
// `//` is dropped on purpose: a trailing `//` may be part of a URL or a string, and
// blanking it would hide a real call — undercounting is the worse error here, because it
// reports "orphan" for something that runs.
function sourceSurface(f, text) {
  const hash = /\.(?:sh|toml)$/.test(f);
  const out = [];
  let inBlock = false;
  text.split("\n").forEach((raw, i) => {
    let line = raw;
    if (!hash) {
      if (inBlock) {
        const end = line.indexOf("*/");
        if (end === -1) return;
        line = line.slice(end + 2);
        inBlock = false;
      }
      const open = line.indexOf("/*");
      if (open !== -1 && line.indexOf("*/", open) === -1) {
        inBlock = true;
        line = line.slice(0, open);
      }
      const t = line.trim();
      if (t.startsWith("//") || t.startsWith("*")) return;
    } else if (line.trim().startsWith("#")) {
      return;
    }
    if (line.trim() !== "") out.push({ line: i + 1, text: line });
  });
  return out;
}

const unreadable = [];
const surfaces = [];
for (const f of tracked) {
  if (!(WORKFLOW.test(f) || PKG_JSON.test(f) || SOURCE_EXT.test(f))) continue;
  const text = read(f);
  if (text === null) {
    unreadable.push(f);
    continue;
  }
  const kind = WORKFLOW.test(f) ? "workflow" : PKG_JSON.test(f) ? "npm" : "source";
  const entries = kind === "workflow" ? workflowSurface(text) : kind === "npm" ? pkgSurface(f, text) : sourceSurface(f, text);
  const when = kind === "workflow" ? workflowTriggers(text) : null;
  surfaces.push({ file: f, kind, entries, when });
  // A workflow's non-`run:` body still WIRES paths without running them — dorny/paths-filter
  // lists scripts/make-wipi-keydraw-fixture.sh as a trigger path. That belongs in the
  // named-not-run column, never in the caller count, so it is a ref-only surface.
  if (kind === "workflow") {
    const runLines = new Set(entries.map((e) => e.line));
    surfaces.push({
      file: f,
      kind: "workflow-config",
      when,
      entries: text
        .split("\n")
        .map((t, i) => ({ line: i + 1, text: t }))
        .filter((e) => !runLines.has(e.line) && e.text.trim() !== "" && !e.text.trim().startsWith("#")),
    });
  }
}

// ── Counting ────────────────────────────────────────────────────────────────
// ★Naming a path is NOT calling it, even outside a comment. Measured on this tree: of
//   the source-file hits a plain substring match produces, most are the path written as
//   DATA — check-parity-lock-wired.mjs holds `test:`/`checker:` constants naming the two
//   files it audits, check-upstream-guard-wired.mjs holds the path it requires a workflow
//   to name, check-engine-contract.mjs prints "run scripts/build-wasm.sh first" in an
//   error string. Counting those as callers reports "wired" for a file nothing executes,
//   which is precisely the lie this census exists to remove. So a hit must be in an
//   INVOCATION form, enumerated below, and everything else is reported separately as a
//   reference (still useful: a rename breaks those sites too).
// ★The path is matched repo-relative AND relative to the surface's own directory — the
//   two forms this tree uses (web/package.json: `bash ../scripts/build-wasm.sh`;
//   wie_cli/tests/dod_ci_parity.rs: `#[path = "support/dod_ci_parity.rs"]`).
const esc = (s) => s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
const CMD_HEAD = String.raw`(?:^|[;&|(]|\$\(|&&|\|\|)\s*(?:[A-Za-z_]\w*=\S*\s+)*`;
function invocationForms(form) {
  const f = esc(form);
  return [
    // shell: `bash scripts/x.sh`, `node scripts/x.mjs`, `VAR=1 npx foo scripts/x.mjs`
    new RegExp(`${CMD_HEAD}(?:bash|sh|zsh|node|npx|python3?|source)\\s+(?:-{1,2}\\S+\\s+)*${f}(?![\\w./-])`),
    // shell: `./scripts/x.sh` at command position
    new RegExp(`${CMD_HEAD}\\./${f}(?![\\w./-])`),
    // JS module load — an import RUNS the module's top level
    new RegExp(`(?:\\bfrom|\\bimport|\\brequire)\\s*\\(?\\s*["'\`](?:\\./)?${f}["'\`]`),
    // JS child process
    new RegExp(`(?:execFileSync|execSync|spawnSync|spawn)\\([^)]{0,160}${f}`),
    // Rust: a module declared at this path is compiled INTO the target
    new RegExp(`#\\[path\\s*=\\s*"${f}"`),
  ];
}

const callers = new Map(population.map((p) => [p, []]));
const refs = new Map(population.map((p) => [p, []]));

for (const s of surfaces) {
  const dir = path.dirname(s.file);
  for (const p of population) {
    if (p === s.file) continue; // a file is not its own caller
    const rel = path.relative(dir, p);
    const forms = [p];
    if (rel !== p) forms.push(rel); // includes `../scripts/x.sh` — web/package.json spells it that way
    const invokes = forms.flatMap(invocationForms);
    for (const e of s.entries) {
      if (!forms.some((form) => e.text.includes(form))) continue;
      const where = { file: s.file, line: e.line, kind: s.kind, when: s.when, how: "name" };
      if (s.kind !== "workflow-config" && invokes.some((re) => re.test(e.text))) callers.get(p).push(where);
      else refs.get(p).push(where);
    }
  }
}

// ★The glob rule cannot be applied without the target set. Say so loudly and exit 2 —
//   folding it into the counts would understate callers and over-report orphans.
const globUnmeasured = cargoTestTargets === null;

const globHits = [];
const globCovered = new Set();
{
  for (const s of surfaces) {
    if (s.kind !== "workflow" && s.kind !== "npm") continue; // only a run step or an alias starts cargo
    for (const e of s.entries) {
      for (const g of GLOB_CALLERS) {
        if (!g.invokes.test(e.text)) continue;
        globHits.push({ file: s.file, line: e.line, rule: g.label });
        for (const p of population) {
          if (!g.covers(p)) continue;
          callers.get(p).push({ file: s.file, line: e.line, kind: s.kind, when: s.when, how: "glob" });
          globCovered.add(p);
        }
      }
    }
  }
}

// One surface can name the same artifact on several lines; the question is "how many
// places run this", so distinct file:line is the unit.
const uniq = (hits) => {
  const seen = new Map();
  for (const h of hits) seen.set(`${h.file}:${h.line}`, h);
  return [...seen.values()].sort((a, b) => (a.file + a.line).localeCompare(b.file + b.line));
};

const rows = population.map((p) => ({ path: p, hits: uniq(callers.get(p)), refs: uniq(refs.get(p)) }));
const bucket = (n) => (n === 0 ? "0" : n === 1 ? "1" : "2+");
const groups = { 0: [], 1: [], "2+": [] };
for (const r of rows) groups[bucket(r.hits.length)].push(r);

// ── Output ──────────────────────────────────────────────────────────────────
const w = Math.max(...rows.map((r) => r.path.length));
console.log(
  `checker-census: ${rows.length} executable artifacts · callers 0 = ${groups[0].length} · 1 = ${groups[1].length} · 2+ = ${groups["2+"].length}`,
);
// `[crate/]tests/**.rs`, with the crate segment OPTIONAL — the repo root is itself a package, so
// `tests/font.rs` is in the population. This line used to read `*/tests/**.rs`, which describes the
// regex the -fix round REPLACED (it dropped the root target); the regex was corrected and the
// sentence describing it was not. Same defect class as the numbers this round is correcting, so it
// is corrected here rather than left for a reader to trip over.
console.log(`  population: scripts/*.{sh,mjs,js} · .github/scripts/**.{sh,mjs,js} · [crate/]tests/**.rs (crate segment optional — the root is a package)`);
console.log(`  surfaces:   ${surfaces.filter((s) => s.kind === "workflow").length} workflows · ${surfaces.filter((s) => s.kind === "npm").length} package.json · ${surfaces.filter((s) => s.kind === "source").length} source files (comments stripped)`);
for (const g of GLOB_CALLERS) {
  const n = globHits.filter((h) => h.rule === g.label).length;
  console.log(`  glob rule:  ${g.label}`);
  if (globUnmeasured) {
    console.log(`  ★UNMEASURABLE  \`cargo metadata --no-deps\` could not be read, so which files this rule reaches is UNKNOWN.`);
    console.log(`              ★The caller counts below are a FLOOR and the 0-caller bucket is OVER-stated. This is not "no glob callers".`);
  } else {
    const covered = [...globCovered].filter(g.covers).length;
    console.log(`              ${n} call site(s) · coverage asked of cargo (${cargoTestTargets.size} integration-test target(s)) → ${covered} artifact(s) in this population`);
    console.log(`              ★those ${covered} have NO by-name caller at all; drop this rule and they all read as orphans`);
  }
}
if (unreadable.length) console.log(`  ★UNREAD:    ${unreadable.length} surface(s) could not be read — the counts below are a floor: ${unreadable.join(", ")}`);

for (const b of ["0", "1", "2+"]) {
  console.log(`\n── callers ${b} ── ${groups[b].length} ────────────────────────────────`);
  if (b === "0") console.log("  ★a zero here is a question, not a defect — see this file's header.");
  for (const r of groups[b]) {
    const where = r.hits.map((h) => `${h.file}:${h.line}${h.when ? ` [${h.when}]` : ""}${h.how === "glob" ? " (glob)" : ""}`).join(", ");
    console.log(`  ${r.path.padEnd(w)}  ${where || "—"}`);
    if (r.refs.length) console.log(`  ${"".padEnd(w)}  named-not-run: ${r.refs.map((h) => `${h.file}:${h.line}`).join(", ")}`);
  }
}
if (globUnmeasured) {
  console.error("checker-census: ★UNMEASURABLE — glob coverage unknown (cargo metadata unavailable); the printed counts are a floor.");
  process.exit(2);
}
process.exit(0);
