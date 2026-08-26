# AGENTS.md

## Goal

Keep the wie emulator engine correct and shippable on two hosts at once: the native `wie_cli`
desktop host, and the `wie_web` browser host that otterpebble's featurephone shell consumes as a
prebuilt WASM artifact. Most constraints below exist because those hosts share one workspace — a
change that is fine natively can break the wasm build, the published artifact, or the consumer
that boots it. Your task ends with an **open PR**, not a merge.

## Constraints

Violating one means the task failed. Each exists because something broke without it — do not
remove, weaken, or "simplify" any without a ticket saying so, and treat a refactor that trips one
as wrong until proven otherwise. **This table is the do-not-cut list.** Each row's *why* lives in
the file that enforces it; causes no file enforces are in the ledger.

| # | Constraint | Locked by |
|---|---|---|
| 1 | The four gates pass before every commit (see Definition of Done) | `rust.yml` — header comment |
| 2 | Coverage stays wired. Never raise coverage by deleting tests; `codecov.yml` is deliberately empty | `coverage.yml` — header comment |
| 3 | The featurephone engine contract is fail-closed. A changed WASM export surface means updating `docs/contracts/featurephone-engine-contract.json` **in the same PR** | `check-engine-contract.mjs` + `contract-roundtrip.mjs`, via `engine-contract.yml` (PR) and `publish-artifact.yml` (release) |
| 4 | `engine-contract.yml`'s `contract` job stays an always-run wrapper — no `paths:` on its triggers; relevance is detected inside the job, and its filter list stays in sync with `publish-artifact.yml`'s `on.push.paths` | `engine-contract.yml:21-30`; why → ledger |
| 5 | `cargo audit` with no ignores. A suppression needs a named advisory ID and a written reachability argument — never blanket, never `continue-on-error` | `rust-audit.yaml:39-54` |
| 6 | `no_std` + `extern crate alloc` in the engine crates — reaching for `std` breaks the web build | wasm clippy gate in `rust.yml`; `docs/architecture.md` |
| 7 | `wie_web` is an empty library off `wasm32`. Do not "clean up" the `cfg(target_arch = "wasm32")` gates | `wie_web/Cargo.toml:1-11`; native jobs in `rust.yml` |
| 8 | The exact version pins and the `[patch]` table are deliberate | `Cargo.toml:70-75`; full rationale in the ledger |
| 9 | No game bytes, ever | `.gitignore` blocklist + `audit-no-leak.sh` — full text below |
| 10 | Secrets are referenced, never embedded or printed | `.dev.vars*` git-ignored + `.claude/settings.json` read-deny — full text below |
| 11 | D1 migrations auto-apply to prod on `main`, destructive statements included — author accordingly | `web.yml:97-100`; `docs/CLOUDFLARE_SETUP.md` |
| 12 | Never commit to `main`; branch → PR, and stop. Merge and branch deletion are a separate approved task | **Nothing machine-locks this** — see Definition of Done |

### Held by you, not by a machine

Quoted in full on purpose — no gate catches these, and a table row would delete the working part.

- **No game bytes, ever.** Game binaries/saves must never enter the repo, the build output, or any log (`.gitignore` blocklist + `scripts/audit-no-leak.sh`). Server-side, the file vault is per-owner isolated with no cross-user identity path — `npm run audit` encodes those checks.
- **Secrets are referenced, never embedded or printed.** `.dev.vars*` is git-ignored (`.dev.vars.example` is the committed template); CI reads tokens from `secrets.*` and gates steps on presence flags rather than echoing values.
- **Never rewrite published history**: no `git push --force`, no rebasing a branch that has been pushed. Before a risky change (bulk deletion, migration/schema edit, deploy wiring), commit a checkpoint first, so recovering the previous state never needs a force-push.
- **Local surfaces only**: work this repo from a local session. Do not move the work onto cloud surfaces (`claude --remote`, Cowork/Dispatch, app chat/Projects) — game bytes and secrets must never leave this machine, which is what the `.gitignore` blocklist, `scripts/audit-no-leak.sh`, and the `.dev.vars` read-deny in `.claude/settings.json` exist to enforce.

## Definition of Done

### The four gates (what `rust.yml` actually runs)

Run these — not just `cargo clippy --workspace` — when you want local green to predict CI green.
`--workspace` checks lib targets only in the default profile; CI additionally denies warnings and
lints the wasm target, and the test run needs a raised stack.

```sh
cargo fmt --all -- --check                       # rust.yml: formatting gate
cargo clippy --all -- -D warnings                # rust.yml: lint gate (warnings are errors)
cargo clippy --target wasm32-unknown-unknown -- -D warnings   # rust.yml: wasm lint gate
RUST_MIN_STACK=4194304 cargo test --all          # rust.yml: tests (the env var is required)
```

`RUST_MIN_STACK=4194304` is not decorative — CI sets it on every platform because the JVM/ARM
interpreter recursion overflows the default test-thread stack without it.

**All four run before every commit, whatever you changed** — a docs- or `web/`-only diff is not an
exemption. The web-surface commands below are *additional* to these, never an alternative.

Narrower commands are conveniences, not gates: `cargo build` (default member `wie_cli`),
`cargo test -p <crate> <test_name>`, `cargo fmt` to fix formatting (`rustfmt.toml`: max_width=150).

### Web-surface commands (on top of the four gates, when touching `web/`, `functions/`, `scripts/`, or `migrations/`)

```sh
node scripts/check-engine-contract.mjs   # static featurephone-contract surface check (node only; needs web/src/wasm present)
npm run audit                            # scripts/audit-no-leak.sh: no game bytes / no cross-user leak (offline)
npm run build:wasm                       # scripts/build-wasm.sh: cargo wasm32 + wasm-bindgen + wasm-opt -> web/src/wasm
npm run frontend                         # cd web && npm install && npm run build (wasm + tsc -b + vite build) -> web/dist
node scripts/contract-roundtrip.mjs      # real-browser boot round-trip; needs `npx playwright install chromium`
npm run verify                           # scripts/verify-browser.mjs: browser verification pass
```

The first two are the cheap offline pre-push check for any `functions/` or `web/src/lib` change.
The contract check needs the WASM artifact already in `web/src/wasm/` — build it first or reuse a
local build, else it fails with missing-artifact violations (CI order: `engine-contract.yml:116`
then `:125`). The rest need a toolchain fetch — run them only when the artifact or UI changes.

### Landing paperwork

- **`STATE.md` and `REPORT.md` are tracked files, not scratch**: keep `STATE.md`'s 진행중/완료/다음 current as a task starts and lands, and append a dated 무엇을·왜·사용자 영향 entry to the top of `REPORT.md` when it lands.
- **Follow-up proposals go in a `docs/worklog/*.json`, or they do not exist.** When a task leaves
  follow-up recommendations (or adopts/declines earlier ones), write
  `docs/worklog/YYYY-MM-DD-<slug>.json` in the same PR. The cockpit 「후속 작업 추천」 panel reads
  `.json` in that directory and nothing else — a proposal left only in prose (`REPORT.md`, the done
  reply, a `.md` worklog) never reaches the screen. Schema below.

#### Worklog `.json` schema (2026-08-26)

**Do not invent keys** — these are the ones the consumer (`/api/proposals` → `scanRepoSimple`)
actually reads. Everything else in the file (`schema`, `task`, `title`, `summary`, `changes`,
`verification`, `limits`, …) is free-form: the consumer ignores it, so it is for humans and for
the next task.

| key | type | what the consumer does with it |
|---|---|---|
| `date` | `"YYYY-MM-DD"` | sort axis; must equal the filename's first 10 chars (it falls back to them, so a mismatch makes the sort lie) |
| `proposals[]` | array of objects | one array element = one card. Its `ref` is derived as `<basename>#p<0-based index>` |
| `proposals[].title` `plainSummary` `userBenefit` `why` `tradeoff` `effort` `target` | string | the card body — fill **all seven**; an empty one renders as an empty field |
| `adoptedProposals[]` / `declinedProposals[]` | array of `ref` strings | removes that `ref` from the open recommendations (disposition record) |

Locked by `scripts/check-worklog-json.mjs` (run it directly; `engine-contract.yml` runs it on every
PR). It validates every `.json` in `docs/worklog/` and nothing else — **existing worklogs are not
retroactively converted**, and no `.md` sibling is required (wie's worklogs are `.json`-only).

### Git Workflow

**Completion is an open PR, not a merge.** Your task ends when the branch is pushed and the PR is
open awaiting review approval. Merging and branch deletion are a *separate* `-merge` task that runs
only after the review gate approves. This is not a formality: merging your own PR bypasses the
review gate, and this repo has been burned by exactly that five times.

- **Run every task to completion**: branch → commit → push → **open a PR, and stop there**. Do not stop earlier at "changes made" — an unpushed branch or an unopened PR is an unfinished task. But do not go further either.
- **Never merge your own PR**, and never merge on the strength of green CI alone. CI passing is necessary, not sufficient — approval is what authorizes the merge, and it is not yours to grant.
- **Never commit directly to `main`**: always work on a short-lived branch.
- **Leave your branch in place**: deleting it is the merge task's job (see below). Never re-merge or re-PR an already-merged branch.
- **GitHub CLI**: scope `gh` commands with `-R Jun025/wie`.
- **Commit trailer**: end commit messages with the `Co-Authored-By:` trailer.

### For the `-merge` task only

Recorded so the knowledge is not lost — **not** something to do at the end of an implementation task.

- Squash-merge, then delete the remote branch (`gh pr merge --squash --delete-branch`) and the local one (`git branch -D <branch>`, then `git fetch --prune`). `-D` is required because squash-merged branches aren't recognized as merged by `-d`.
- Leave no stale merged branches behind — only `main` and in-progress work remain. Sync local `main` afterwards.

## Incident ledger (사건 대장)

Measured causes. These are narrative on purpose — a rule without its cause gets reverted by the
next person who finds it inconvenient. Do not compress them into the table above.

**Self-merge, five times.** The cause behind §Git Workflow: five landed changes bypassed the
review gate this exact way. The rule is not "merging is discouraged".

**A paths-filtered required check deadlocks the merge forever.** `contract` is a required status
check on `main`. A required check whose triggers carry a `paths:` filter never reports on a PR
that misses those paths — GitHub shows "Expected — Waiting for status" indefinitely and the merge
button never unlocks. That is why `engine-contract.yml` filters *inside* the job instead. Adding a
`paths:` filter to it looks like an obvious optimization and is the outage.

**`paths-filter` reads paths, not content** (measured in otterpebble's `free-tier.md`, re-confirmed
here 2026-08-05). A comment-only or docs-only edit to a filtered path still fires the workflow.
Concretely: `**/Cargo.toml` is in `publish-artifact.yml`'s `on.push.paths:21`, so a comment-only
`Cargo.toml` change, once merged to `main`, builds a fresh artifact, cuts a GitHub Release, and
`repository_dispatch`es otterpebble. Treat every `Cargo.toml` as deploy-triggering even when your
edit is a comment.

**Exact version pins are deliberate.** `wasm-bindgen = "=0.2.108"` (and `js-sys`/`web-sys`
`=0.3.85`) must match the `wasm-bindgen-cli@0.2.108` CI pin, or the generated glue and the runtime
disagree. `tracing-attributes = "<0.1.29"` is pinned for a no_std compile error. The `[patch]`
table redirecting RustJava to the `Jun025/RustJava` fork at a fixed rev carries KTF
panic→exception hardening — it is not stale duplication.

**`RUST_MIN_STACK=4194304` is not decorative.** Without it `cargo test --all` stack-overflows
rather than failing an assertion. See the four-gates block.

## Reference

### Code Style

Rust 2024. Standard Rust naming and hygiene otherwise; repo-specific points:

- **Errors**: `wie_util::Result<T>` / `WieError`, propagated with `?`. No panics in library code.
- **Async**: `async-trait` for async trait methods.
- **Imports**: grouped std/alloc → external → local → workspace, alphabetized.
- Never suppress a type error instead of fixing it.

### Layout

One Cargo workspace (the engine) plus a Cloudflare Pages web service embedding it as WASM. Nothing
above the engine reaches back into it.

- `wie_util` — `Result`/`WieError`, byte read/write helpers. Bottom of the stack.
- `wie_backend` — host-abstraction boundary: `Platform`/`Screen`/`AudioSink`/`Filesystem`/`Database`, canvas, executor, event queue.
- `wie_core_arm` — ARM32 emulation + the `data/binary_patches.toml` per-game patch table.
- `wie_jvm_support` — bridge onto the patched RustJava JVM.
- `wie_wipi_c`, `wie_wipi_java`, `wie_midp`, `wie_skvm` — the emulated API surfaces.
- `wie_ktf`, `wie_lgt`, `wie_skt`, `wie_j2me` — per-carrier entry points (`wie_ktf`/`wie_lgt` hold the heavy reverse-engineered runtimes).
- `wie_cli` — native host (also `wie_validate`, a headless triage runner); `wie_web` — browser host, empty library off `wasm32` (Constraint 7).
- `web/`, `functions/`, `migrations/`, `scripts/`, `docs/`, `data/`, `fonts/`, `test_data/` — non-Rust surfaces.

**Full map: `docs/architecture.md`** — layer diagram, a role for every crate, and what each
non-Rust directory holds. Read it to find where something lives.

### Agent Environment

**ponytail** is active in **`full` mode** — keep it there, do not switch to `ultra`. Its job is to
cut over-engineering and it treats anything unstated as out of scope, so the **Constraints table
above is the explicit do-not-cut list**: a finding that lands on one of those rows is a false
positive. `/ponytail-audit` output is a *report*; applying it is a separate, ticketed change.

**MCP servers: none are registered** (measured 2026-08-05 — `claude mcp list` reports none and
`mcpServers` is empty in `~/.claude.json`, `~/.claude/settings.json`, and both `$CLAUDE_CONFIG_DIR`
files). Earlier revisions of this file claimed three were available at user scope; they are not.
Navigate with `Glob`/`Grep`/`Read` and `docs/architecture.md`, and check external crate APIs
against the source in `~/.cargo` rather than guessing — a wrong signature costs a full rebuild.
`.serena/` in the tree is residue from when serena *was* registered; it is git-ignored, do not
commit it.
