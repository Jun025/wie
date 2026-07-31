# AGENTS.md

## Build/Test/Lint Commands
- **Build**: `cargo build` (default member: `wie_cli`)
- **Test all**: `cargo test --workspace`
- **Test single**: `cargo test -p wie_ktf test_helloworld` or `cargo test -p <crate> <test_name>`
- **Lint**: `cargo clippy --workspace`
- **Format**: `cargo fmt` (uses rustfmt.toml: max_width=150, use_field_init_shorthand=true)
- **Pre-commit (MANDATORY)**: Always run `cargo fmt` and `cargo clippy --workspace` before every commit. CI will reject unformatted or lint-failing code.

### CI-parity commands (what `rust.yml` actually runs)

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

Web-surface commands (only when touching `web/`, `functions/`, `scripts/`, or `migrations/`):

```sh
node scripts/check-engine-contract.mjs   # static featurephone-contract surface check (node only; needs web/src/wasm present)
npm run audit                            # scripts/audit-no-leak.sh: no game bytes / no cross-user leak (offline)
npm run build:wasm                       # scripts/build-wasm.sh: cargo wasm32 + wasm-bindgen + wasm-opt -> web/src/wasm
npm run frontend                         # cd web && npm install && npm run build (wasm + tsc -b + vite build) -> web/dist
node scripts/contract-roundtrip.mjs      # real-browser boot round-trip; needs `npx playwright install chromium`
npm run verify                           # scripts/verify-browser.mjs: browser verification pass
```

`npm run audit` runs offline against the checked-out tree; the contract check is offline too but
assumes the WASM artifact already exists in `web/src/wasm/` (run `npm run build:wasm` first, or
reuse an earlier local build) — on a clean checkout it fails with missing-artifact violations.
CI runs them in that order (`engine-contract.yml`: `build-wasm.sh` at :116, then the check at :125).
Together they are the cheap pre-push check for any `functions/` or `web/src/lib` change.
The rest need a toolchain fetch (`wasm-bindgen-cli`,
`binaryen`, npm install, Playwright browsers), so run them when the change actually touches the
built artifact or the UI.

## Git Workflow
- **Run every task to completion**: branch → commit → PR → merge into `main`. Do not stop at "changes made" — land it on `main` unless genuinely blocked.
- **Never commit directly to `main`**: always work on a short-lived branch.
- **Clean up merged branches (MANDATORY)**: once a branch's work is complete and merged into `main`, delete it — remote and local — and sync local `main`. Prefer `gh pr merge --delete-branch` (deletes the remote), then locally `git branch -D <branch>` and `git fetch --prune`. Use `-D` (force) because squash-merged branches aren't recognized as merged by `-d`. Leave no stale merged branches behind — only `main` and in-progress work remain. Never re-merge or re-PR an already-merged branch.
- **GitHub CLI**: scope `gh` commands with `-R Jun025/wie`.
- **Commit trailer**: end commit messages with the `Co-Authored-By:` trailer.

## Session Discipline
- **Never rewrite published history**: no `git push --force`, no rebasing a branch that has been pushed. Before a risky change (bulk deletion, migration/schema edit, deploy wiring), commit a checkpoint first, so recovering the previous state never needs a force-push.
- **Local surfaces only**: work this repo from a local session. Do not move the work onto cloud surfaces (`claude --remote`, Cowork/Dispatch, app chat/Projects) — game bytes and secrets must never leave this machine, which is what the `.gitignore` blocklist, `scripts/audit-no-leak.sh`, and the `.dev.vars` read-deny in `.claude/settings.json` exist to enforce.
- **`STATE.md` and `REPORT.md` are tracked files, not scratch**: keep `STATE.md`'s 진행중/완료/다음 current as a task starts and lands, and append a dated 무엇을·왜·사용자 영향 entry to the top of `REPORT.md` when it lands.

## Code Style Guidelines
- **Edition**: Rust 2024
- **no_std**: Most crates are `#![no_std]` with `extern crate alloc`
- **Imports**: Group by source (std/alloc → external crates → local crate → workspace crates), alphabetized
- **Error handling**: Use `wie_util::Result<T>` / `WieError` enum. Propagate with `?`, no panics in library code
- **Naming**: snake_case for functions/variables, PascalCase for types, SCREAMING_CASE for constants
- **Types**: Explicit types preferred. Never use `as any` equivalents or suppress errors
- **Async**: Use `async-trait` for async trait methods

## Project Layout
- `wie_backend`: System-level services for APIs
- `wie_cli`: CLI for local testing
- `wie_core_arm`: ARM emulation
- `wie_jvm_support`: JVM support
- `wie_midp`, `wie_wipi_*`, `wie_skvm`: API implementations
- `wie_j2me`, `wie_skt`, `wie_ktf`, `wie_lgt`: Platform-specific logic

## Architecture

The repo is one Cargo workspace (the emulator engine) plus a Cloudflare Pages web service that
embeds the engine as WASM. Nothing above the engine is allowed to reach back into it.

```
                    ┌─ hosts ────────────────────────────────────────┐
  wie_cli  (native desktop: winit/softbuffer/rodio/midir)            │
  wie_web  (browser: wasm-bindgen; empty lib on non-wasm targets)    │
                    └───────────────┬────────────────────────────────┘
                                    │ implements wie_backend::Platform
                    ┌───────────────▼────────────────────────────────┐
  wie_backend       │ Platform / Screen / AudioSink / Filesystem /   │
                    │ Database traits, canvas, executor, event queue │
                    └───────────────┬────────────────────────────────┘
        ┌───────────────────────────┼───────────────────────────┐
  wie_ktf / wie_lgt / wie_skt / wie_j2me   ← per-carrier emulator entry points
        └───────────┬───────────────┴──────────────┬────────────┘
  wie_core_arm (ARM32 CPU + binary patches)   wie_jvm_support (RustJava bridge)
  wie_wipi_c / wie_wipi_java / wie_midp / wie_skvm  ← the emulated API surfaces
                                    │
                              wie_util (Result/WieError, byte read/write helpers)
```

### Crate roles

| Crate | Role |
|---|---|
| `wie_util` | `Result`/`WieError`, `ByteRead`/`ByteWrite`, generic memory read/write helpers. Bottom of the stack — depends on nothing in-tree. |
| `wie_backend` | The host-abstraction boundary. Owns the `Platform`, `Screen`, `AudioSink`, `Filesystem`, `Database`, `TaskRunner`, `Emulator` traits plus canvas/font/zip/audio services. Every host implements these. |
| `wie_core_arm` | ARM32 interpretation (`arm32_cpu`), the emulated-function calling convention, and the `data/binary_patches.toml`-driven per-game patch table. |
| `wie_jvm_support` | Bridges the emulated Java world onto the patched RustJava JVM (`JvmImplementation`, class-proto plumbing). |
| `wie_wipi_c` | WIPI C API (`WIPICContext`): graphics, kernel, media, network shims called from ARM code. |
| `wie_wipi_java`, `wie_midp`, `wie_skvm` | The emulated Java class libraries — WIPI (`org.kwis.msp.*`), MIDP (`javax.microedition.*`), and SK-VM respectively. |
| `wie_ktf`, `wie_lgt`, `wie_skt`, `wie_j2me` | Per-platform entry points: archive layout, boot/relocation, and which API surfaces get wired. `wie_ktf`/`wie_lgt` carry the heavy reverse-engineered runtimes. |
| `wie_cli` | Native host. Also ships `src/bin/wie_validate.rs`, a headless PASS/FAIL triage runner for batch game validation. |
| `wie_web` | Browser host. Compiles to an empty library off `wasm32` on purpose, so native workspace jobs stay green. |
| `wie_ktf_dump` | Dev-only binary: dumps a KTF game's relocated `client.bin` for IDA/Ghidra. |
| `test_utils` | Shared in-memory `Platform`/`Filesystem`/`Database`/JVM fixtures for tests. |

### Non-Rust surfaces

| Path | Role |
|---|---|
| `web/` | React 19 + Vite + Tailwind frontend. `web/src/wasm/` holds the engine artifact (`wie_web.js` glue + `wie_web_bg.wasm`) that `build-wasm.sh` generates — git-ignored, never committed (`web/.gitignore`). |
| `functions/` | Cloudflare Pages Functions (the API). `functions/_lib/` is import-only (Pages does not route `_`-prefixed paths); `functions/api/` is the routed surface. |
| `migrations/` | D1 SQL migrations, auto-applied to the prod DB on `main` pushes. |
| `scripts/` | Build and gate scripts: `build-wasm.sh`, `check-engine-contract.mjs`, `contract-roundtrip.mjs`, `audit-no-leak.sh`, `verify-browser.mjs`, `smoke_gate.sh`, `lgt_render_probe.sh`. |
| `docs/` | `architecture.md`, per-platform RE notes (`ktf.md`, `lgt.md`, `lgt_abi.md`), `contracts/` (the featurephone consumer pin), `project-kb/`, `verification/` screenshots, `worklog/`. |
| `data/`, `fonts/`, `test_data/` | Binary-patch table, the bundled `neodgm.ttf`, and the hello-world fixtures the tests and the round-trip gate boot. |

## Hard Requirements

These are load-bearing. They look like ceremony and are not — each one exists because something
broke without it. Do not remove, weaken, or "simplify" any of them without an explicit ticket
saying so, and treat any refactor that trips one as wrong until proven otherwise.

**Gates**
1. `cargo fmt --all -- --check`, `cargo clippy --all -- -D warnings`, `cargo clippy --target wasm32-unknown-unknown -- -D warnings`, and `RUST_MIN_STACK=4194304 cargo test --all` must all pass before every commit. `rust.yml` runs the full matrix (macOS/Ubuntu/Windows × stable/beta); a warning is a build failure there.
2. **Coverage stays wired.** `coverage.yml` runs `cargo tarpaulin --all-features --skip-clean --workspace --timeout 120 --out xml` on every push and uploads to Codecov (upload is skipped, not failed, when `CODECOV_TOKEN` is absent — forks stay green). `codecov.yml` is intentionally empty: Codecov defaults apply, and the file is the hook for adding thresholds later. Keep the job green — do not delete tests to make coverage "simpler".
3. **The featurephone engine contract is fail-closed.** `docs/contracts/featurephone-engine-contract.json` pins the surface otterpebble consumes. `scripts/check-engine-contract.mjs` (static) and `scripts/contract-roundtrip.mjs` (real-browser boot) gate both the PR (`engine-contract.yml`) and the release (`publish-artifact.yml`). Changing the exported WASM surface means updating the pin deliberately, in the same PR.
4. **`engine-contract.yml`'s `contract` job must stay an always-run wrapper.** It has no `paths:` filter on its triggers; relevance is detected *inside* the job via `dorny/paths-filter`. This shape is required because `contract` is a required status check on `main` — a paths-filtered required check sits at "Expected — Waiting for status" forever and deadlocks the merge. Keep its filter list in sync with `publish-artifact.yml`'s `on.push.paths`.
5. **`cargo audit` runs with no ignores** (`rust-audit.yaml`). Vulnerabilities fail the job; unmaintained/yanked warnings do not. Any future suppression must be a named advisory ID with a written reachability argument — never a blanket ignore, never `continue-on-error`.

**Invariants**
6. **`no_std` is not optional.** Thirteen crates are `#![no_std] + extern crate alloc` so the engine builds for wasm32. Reaching for `std` to shorten code breaks the web build.
7. **`wie_web` compiles to an empty library off `wasm32`.** The `cfg(target_arch = "wasm32")` gating in `wie_web/Cargo.toml` and its sources is what keeps native `cargo build`/`test`/`clippy` green. Do not "clean up" the gates.
8. **Exact version pins are deliberate.** `wasm-bindgen = "=0.2.108"` (and `js-sys`/`web-sys` `=0.3.85`) must match the `wasm-bindgen-cli@0.2.108` CI pin, or the generated glue and the runtime disagree. `tracing-attributes = "<0.1.29"` is pinned for a no_std compile error. The `[patch]` table redirecting RustJava to the `Jun025/RustJava` fork at a fixed rev carries KTF panic→exception hardening — it is not stale duplication.
9. **No game bytes, ever.** Game binaries/saves must never enter the repo, the build output, or any log (`.gitignore` blocklist + `scripts/audit-no-leak.sh`). Server-side, the file vault is per-owner isolated with no cross-user identity path — `npm run audit` encodes those checks.
10. **Secrets are referenced, never embedded or printed.** `.dev.vars*` is git-ignored (`.dev.vars.example` is the committed template); CI reads tokens from `secrets.*` and gates steps on presence flags rather than echoing values.
11. **D1 migrations auto-apply to prod on `main`, including destructive statements.** This is a documented pre-launch policy (`docs/CLOUDFLARE_SETUP.md`, `web.yml`). Author migrations accordingly.
12. **Branch discipline** (see Git Workflow above): never commit to `main`; branch → PR → squash-merge → delete the branch.

## Agent Environment

### ponytail

This repo is worked with the **ponytail** plugin active in **`full` mode**. Keep it there —
do not switch to `ultra` or change the mode config for this repo.

ponytail's job is to cut over-engineering, and it treats anything not stated as out of scope.
The **Hard Requirements** section above is therefore the explicit do-not-cut list: gates,
coverage wiring, the contract pin and its always-run wrapper, the no-ignore audit, `no_std`,
the `wie_web` cfg gating, the exact-version and `[patch]` pins, the no-game-bytes and secret
rules, the migration policy, and branch discipline. Read that list before acting on any
ponytail suggestion; a finding that lands on one of those is a false positive.

`/ponytail-audit` output is a *report*. Applying it is a separate, ticketed change.

### MCP servers

Three MCP servers are registered at user scope and available in every session here.

- **context7** — pull real API docs for an external crate/package before writing against it. Use it for `winit`, `wasm-bindgen`/`web-sys`, `image`, `zip`, `rodio`, and the RustJava/`jvm` fork surfaces, where guessing a signature costs a full rebuild cycle.
- **serena** — symbol-level navigation (find symbol, find references) instead of reading whole files. Default to it in `wie_core_arm` (5.3k lines), `wie_ktf` (4.4k), `wie_wipi_java` (64 files), and `docs/lgt_abi.md` (1.5k lines), where a full read is pure token waste.
- **playwright** — **applicable to this repo**: `web/` is a real browser surface, and the featurephone contract gate already drives headless Chromium (`scripts/contract-roundtrip.mjs`, `scripts/verify-browser.mjs`). Use it to verify UI/boot behavior interactively; the committed scripts remain the authority for CI.

`.serena/` is a local tool workspace and is git-ignored — do not commit it.
