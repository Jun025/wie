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
| 8 | The exact version pins and the RustJava `rev` pin are deliberate | `Cargo.toml` — comment above the `rev` lines; full rationale in the ledger |
| 9 | No game bytes, ever | `.gitignore` blocklist + `scripts/audit-no-leak.sh`, run on every PR by `engine-contract.yml` — full text below |
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

<!-- COMMIT-GATES:BEGIN — rust.yml's header comment points at this marked region; keep both markers -->

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

**The matrix has a second toolchain. Run the lint gate on it too.**

```sh
rustup toolchain install beta --component clippy   # re-run whenever beta rolls — install IS update
cargo +beta clippy --all -- -D warnings            # rust.yml: the lint gate, again on beta
```

<!-- COMMIT-GATES:END — everything the BEGIN marker encloses is the commit-gate list; nothing
     outside it is. The `sh` blocks BELOW this marker (wie_validate runner, gh pr checks) are
     conditional extras. Read the enclosed region; do not count blocks. -->

`rust.yml`'s matrix is `[macos, ubuntu, windows] × [stable, beta]`, so all four gates above already
run twice up there — but only the lint gate has ever caught anything on beta that stable missed.
Measured over every `rust.yml` run to date, `ac4ce1aa` (2026-06-24) through `11a35252`
(2026-09-04): **246 runs, 34 failed, and in 9 of those a beta leg was the only failing job. All 9
were `cargo clippy --all -- -D warnings`** — 8 × `clippy::chunks_exact_to_as_chunks` (2026-07-06 →
07-13) and 1 × `clippy::double_must_use` (2026-08-20). Zero on `cargo fmt`, zero on the wasm lint
gate, zero on the tests. That is why this is one line and not four; add another only when a run
makes you. `beta` also rolls every six weeks and CI takes *that day's* beta, so a locally pinned
one silently stops covering the newest lint — which is the whole class this block exists for.

**A new lint reddens code you did not write, so "I only touched config" does not exempt you.** The
2026-08-20 red landed on a PR whose entire diff was six added lines in `wrangler.toml` and no Rust
at all: the `double_must_use` violations were pre-existing and repo-wide. The 2026-07 cluster is
the same shape — it reddened `main` itself on **6 pushes over about 36 hours** (`2026-07-06T18:24`
→ `07-08T05:51`, all at `wie_midp/.../lcdui/image.rs:310`), until `37e3e4f6` (PR #21) replaced the
`chunks_exact` call. The same lint fired once more on 07-13, but at a **different site**
(`wie_lgt/src/compile_model.rs:113`) and only on a feature branch — `e3cbaa08` (PR #33) fixed
*that*, and it never reddened `main`. The two are separate events; do not merge them.

**When beta reddens code you did not touch, split it off — do not fold it into your round.** Both
incidents were cleared by a dedicated round (`double_must_use` → PR #60; the 07-06 cluster →
PR #21), which is the right shape: the fix is repo-wide, it is unrelated to your diff, and merging
it in makes your change unreviewable. Open a separate ticket, cite the failing run in your report,
and say plainly that CI is red for a pre-existing reason.

**Cost, measured on `11a35252`.** The first `cargo +beta clippy` in a target dir holding only
stable artifacts is **36.3s**; after that, following an engine-crate edit, it adds **7.6–7.7s** to
the round. Cargo keys artifacts by rustc version and keeps both sets side by side, so alternating
toolchains does *not* thrash — going back to stable right after beta took 0.50s. No separate
`CARGO_TARGET_DIR` is needed, and none is configured.

The sibling `RustJava` repo hit this same gap and went further, mechanically diffing its documented
gates against its workflows (`scripts/check-dod-ci-parity.py`). **That is now ported** — the marked
region above is diffed against `rust.yml` by `wie_cli/tests/dod_ci_parity.rs`, which `cargo test
--all` runs in all six legs, so a matrix change or a dropped gate reddens the next PR instead of
waiting for someone to notice. It compares two sets and nothing else: the cargo commands, and the
toolchains. What it deliberately does **not** see — the OS axis, the gate × toolchain cross
product, the other workflows, and its own deletion — it prints on every run; read that block
rather than trusting the word "green".

**Touching engine code? The four gates are not enough — run the repo's own runner.**

<!-- ENGINE-RUNNER:BEGIN — scripts/check-engine-runner-fixtures.mjs diffs the fixtures named
     inside this region against `git ls-files test_data/`, both directions. Keep both markers;
     the checker fails if either goes missing rather than passing on an empty region. -->

```sh
node scripts/make-draw-fixture.mjs                                    # builds the J2ME fixture
for f in test_data/draw_j2me.jar test_data/helloworld_ktf.zip test_data/helloworld_lgt.zip; do
  cargo run -q -p wie_cli --bin wie_validate -- "$f"                  # each must report "result":"PASS"
done
for f in test_data/keydraw_ktf.zip test_data/keydraw_lgt.zip; do      # key-driven — --inject is REQUIRED
  cargo run -q -p wie_cli --bin wie_validate -- --inject --expect-last-frame "$f"   # PASS *and* rc=0
done
```

**A fixture that this runner deliberately does not touch is named here, not omitted** — write
`NOT-RUN: test_data/<name> — <why>` inside this marked region. That keeps the classification in the
same document as the list instead of in the checker, which is the one thing the proposal behind this
check warned about: a checker that knows which fixtures are "runner fixtures" becomes a second source
of truth and drifts from this block. **There are none today** (the diff is 0 in both directions), so
this paragraph is the syntax, not a list.

<!-- ENGINE-RUNNER:END -->

**`keydraw_*` without `--inject` reports FAIL, and that is the CORRECT result — you did not break it.**
Those two fixtures paint only in response to a key, so with no injected input the screen stays black
and the validator is right to say so. Measured 2026-09-06 on both carriers: without the flag
`result FAIL · content false · paints 1`, with it `result PASS · content true · paints 55`. The
misread is not hypothetical — a round chasing an unrelated change stopped on exactly this, took the
FAIL for its own regression, and only cleared it by reproducing the same FAIL on an untouched tree.

**`--expect-last-frame` is on the `keydraw_*` line and deliberately NOT on the one above it.** It
turns `last_frame_content` from a reported field into an exit code, which is the only thing that
catches "the emulator ran fine and the last frame is black" — the 2026-09-05 LGT failure, where
`result` stayed PASS and `paints` went *up* (55 → 83, the blank MIDP overpaint). It cannot go on the
line above because that is **one loop over three fixtures** and `helloworld_ktf`/`helloworld_lgt`
fail it by construction — they are *expected* to end blank, and with the flag they exit 1 (measured).
**`draw_j2me` does not fail it** — that fixture ends with content (`last_frame_content true`, rc=0
with the flag), so the reason it goes unflagged is the shared loop, not the fixture. Splitting it
onto its own line would flag it correctly and cost an extra runner line, which is the one thing this
block cannot afford. The expectation is per fixture *and mode*, which is why it lives on the command
line — see `wie_validate.rs`'s header for that reasoning (its table covers `helloworld_*` and
`keydraw_*`; `draw_j2me` is measured here).

**This is the local net, not the CI one.** The browser round-trip's Scenario F is what gates that
failure in CI; this line makes the same class visible in ~20 s with no wasm build, before you push.
Scenario F is not unconditional either — it sits behind `engine-contract.yml`'s `dorny/paths-filter`
`engine` gate, so a diff that touches no engine path reports "Reporting success without rebuilding"
and never runs it. Do not read this line as CI enforcement — nothing in `.github/` runs
`wie_validate` (measured: 0 hits across all workflow files).

**And it stays that way: promoting `--expect-last-frame` into CI was decided against on 2026-09-07,
measured rather than assumed.** The question is not "is it in CI" but "is the class caught", and it
is — twice over, by things that already run:

| what | fixtures | frame it reads | predicate | host | runs when |
|---|---|---|---|---|---|
| Scenarios **E + F** | `keydraw_ktf` + `keydraw_lgt` | **the first frame that reaches the expected pixel count** — `tickLoop` breaks on match | **exact pixel count**, 3 keys each = 6 assertions | `WebScreen` | every PR touching the `engine` filter |
| `--expect-last-frame` | same two | the run's **final** paint | non-blank (boolean) | `HeadlessScreen` | never in CI |

**They differ on two axes, not one, and only the host axis is vacuous.** The *host* axis is
empty: `HeadlessScreen::paint` is a **pure sink** — it stores the frame and updates counters, with no
drawing, compositing or ordering of its own, so everything that can blank a frame happens *above* the
`Screen` boundary, in the shared engine both hosts drive. The *time* axis is **not** empty:
`tickLoop` exits the moment `until` matches, so E+F assert "the expected count was observed at some
tick boundary" and stop looking; the flag asserts "the run's last paint is non-blank". After the
**last** key matches, F does `key_up` → `free()` and sees nothing further, while the headless
`--inject` run continues through its 27-step schedule (measured 2026-09-07: `paints` 55 on both
carriers). So a regression that blanks the screen *after* the final key assertion — a follow-up
action, a stop/shutdown path — passes F and fails the flag. Earlier keys are not exposed: the next
key's assertion re-reads the canvas.

Separately, the flag's own logic is already CI-covered — `last_frame_gate_fails` has an 8-row
truth-table test that `cargo test --all` runs on all six matrix legs.

**So the detection delta is narrow, not zero**, and that is the actual reason not to promote: it is
confined to the window after the last key assertion, **zero incidents have ever been observed in
it**, and buying it costs either a new `cargo` build in the node-only `contract` job or six
redundant runs on `rust.yml`'s matrix. The 2026-09-05 LGT failure hides this window rather than
demonstrating it — that overpaint happened *within* a tick, so the canvas never showed 424 at any
boundary and the early break never fired.

The paths-filter caveat above is real but bounded: its first entry is `**/*.rs`, so any diff that
could regress the engine's last frame does fire it (verified on `a4fda020`, a comment-only `.rs`
landing — step "Contract check — browser boot round-trip" ran and succeeded). A diff that trips
neither has no engine to regress.

**Reopen this if any of three things happen** — otherwise a later round will re-propose it from the
same starting point. First and most likely: **a blank-screen regression lands in the window E+F
structurally cannot see** — after the last key assertion, where the loop has already broken. It will
not arrive as a CI failure, by construction; it arrives from the local runner line above, or from
someone running a game, and *that* is the signal to re-price the delta. Second: `HeadlessScreen`
stops being a pure sink, which would open the host axis too. Third: the shipped native host
(`wie_cli`'s `WindowHandle`) needs covering — **note that promoting this flag would not do that
either**, since it exercises `HeadlessScreen`, not `WindowHandle`. That host is covered by neither
net today, and saying so is the honest version of "the local runner is enough".

**Do not try to shorten these two runs with `--timeout`.** On the `--inject` path that flag is
overwritten: the deadline is rebuilt from the injection schedule (`--boot-secs 2.5` + 0.3 + 27
steps × `--action-secs 0.6` + 1.0 = **20.0 s**), so `--timeout 5` and `--timeout 20` both take ~20 s
(measured). The knobs that do move it are `--boot-secs`/`--action-secs`, and shortening them drops
paints (`--boot-secs 1.0` → 18.7 s, paints 55 → 37), i.e. it buys time by seeing less. Measured wall
time over six runs each: KTF **20.1–26.1 s**, LGT **20.2–21.4 s** — the spread above 20.0 is tick
overrun under load, not budget starvation.

`cargo test --all` boots KTF and LGT but **nothing in it boots a J2ME guest**. 2026-09-04 shipped a
RustJava pin bump whose four gates were all green while `draw_j2me.jar` failed with
`NoClassDefFoundError` on the first tick — one `wie_validate` line reproduced it locally, and the
round had not run it. Every fixture here is committed; no game files are involved.

**Then read the PR's CI — a local pass is not a CI pass, and CI is the last gate.**

```sh
gh pr checks <n> -R Jun025/wie          # every check, with its conclusion
```

Local green predicts CI green for the four gates and no further. Three jobs run only up there and
have each gone red on a locally-green branch: `coverage` runs the tests under `cargo tarpaulin`
(a different execution engine — 2026-09-04 a test that passes natively segfaulted under it),
`rust.yml` runs the matrix on Windows and Ubuntu (2026-09-04 a `:`-vs-`;` path separator passed on
macOS and panicked on Windows), and `engine-contract` diffs the WASM export surface. **Quote the
result in the report even when it is red** — twice on this branch's lineage a CI-red PR was
reported as complete, which is what makes this a gate and not a suggestion.

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

**Which of these CI actually runs — "the check exists" is not "the check runs".** Measured
2026-09-06 across all 8 workflow files: `check-engine-contract.mjs` and `contract-roundtrip.mjs`
run in `engine-contract.yml`; `build-wasm.sh` and the frontend build run in `web.yml`; **`npm run
audit` now runs on every PR** as an always-run step of `engine-contract.yml`; **`verify-browser.mjs`
runs after every deploy** (below). The one after that does **not** run in CI and is **local-only by
design** — do not "fix" that by wiring it:

- **`npm run verify` (`scripts/verify-browser.mjs`) — runs post-deploy, never on a PR.** Since
  2026-09-07 it is the last step of `web.yml`, driving `steps.deploy.outputs.deployment-url` — the
  immutable per-deploy URL, live when the action returns. It is **not** a deploy gate: it runs after
  the bytes are up, so it reports a bad deploy rather than blocking one.

  > **If that step goes red, the gate③ round that landed the merge owns it** — it is already running
  > the same script against production for merge-contract 4-C, so it re-runs it against the URL the
  > failed step printed and either files a ticket or records in its reply that the deploy is bad.

  That owner is not a formality: this repo has already had a check go red with nobody named
  (`check-worklog-coverage`, 2026-09-07), and that one blocked every open PR. This one cannot —
  it is push-only, so PR runs skip it and gate③ reads the PR's checks — which is also why it is
  allowed to fail hard instead of hiding behind `continue-on-error`.

  **It still does not run on a PR, and that part of the old reasoning stands**: `web.yml`'s deploy
  steps are all gated on `github.event_name == 'push'`, so a PR build produces `web/dist` as an
  artifact and deploys nothing, and pointing this script at a PR would mean standing up
  `wrangler pages dev` with D1 bindings inside CI — a workflow-sized change, not a step.

  **Two things it does not tell you.** It reads the per-deploy URL, not the `wie-web.pages.dev`
  alias, whose swing-over delay nothing here measures — so keep running it by hand against
  production after a deploy, which is a *different* assertion: `WIE_BASE=https://wie-web.pages.dev
  node scripts/verify-browser.mjs test_data/helloworld_ktf.zip`. And `rc=0` does not mean the screen
  rendered — it exits non-zero on a leak (2) and on the flow not completing, but `nonBlack: 0` is a
  pass, which for the helloworld fixtures is correct since they are expected to end blank. Read it
  as "booted, took a file, leaked nothing". It needs no game file (its default argument is the
  committed `test_data/helloworld_ktf.zip`).
- **`scripts/smoke_gate.sh` — local only, and structurally so.** It regresses the working game
  catalog against `scripts/smoke_gate_baseline.tsv`, reading titles from `WORKING_DIR`
  (default `game_lab/working`). `game_lab/` is git-ignored and holds real game bytes, which
  **Constraint 9 forbids from ever entering the repo, the build output, or any log**. There is no
  version of this check that runs in CI without breaking the constraint it sits beside; the
  committed baseline is identifiers and expected status only, never paths or bytes.

### Landing paperwork

- **`STATE.md` and `docs/report/` are tracked files, not scratch**: keep `STATE.md`'s 진행중/완료/다음 current as a task starts and lands, and write a dated 무엇을·왜·사용자 영향 entry when it lands. **Round entries go in a new `docs/report/NNNN--YYYY-MM-DD--<ticket-id>.md` — do not append to `REPORT.md`**, which is now a fixed pointer (2026-09-07; every round appending to one file's top made every open PR conflict — 5/5 at migration time, 4 of them on the ledger files *only*). `NNNN` is the global sequence, largest + 1:

  ```sh
  N=$(node scripts/check-docs-report-serial.mjs --next-serial)   # ask the tool, not the directory
  $EDITOR docs/report/$N--$(date +%F)--<ticket-id>.md   # first line: ## [YYYY-MM-DD] title (<ticket-id>)
  grep -H '^## \[' docs/report/*.md | sort -r            # reading it back: the directory is the index
  ```

  **Ask the tool for `N`; do not compute `max + 1` from the directory.** The directory is the
  *merged* tree, so two open PRs computing it independently pick the same serial — the filenames
  differ, git merges both cleanly, and nothing reddens. Measured 2026-09-07: `main` held `0056`,
  open PR #112 held `0056`, open PR #111 held `0057`; the directory said `0057` and the free number
  was `0058`. `--next-serial` consults open PRs (`gh api …/pulls/<n>/files`) and prints the number
  on stdout; if the network is unavailable it warns and falls back to the directory rather than
  blocking you. The post-hoc half runs in CI (`engine-contract.yml`) and reddens a tree that already
  holds a duplicate — **it does not renumber anything, and neither should you renumber a landed
  file**; move the side that has not landed yet.

  **`-H` is load-bearing, not cosmetic.** It prefixes the path, so `sort -r` keys on the *sequence number*; `-h` keys on the title text, which is the date, and this repo lands up to six rounds a day. Measured over 54 files: the `-h` form is **52 lines out of place**, the `-H` form is **0**. Sort by the **sequence number, not the date** — the ledger's date-monotonicity is a coincidence, not a guarantee. `REPORT.md` explains the rest; `docs/report-migration-revert.md` reverts it.
- **The ledger files of this repo are `STATE.md`, `REPORT.md`, `docs/report/**`, `docs/worklog/**`,
  and `docs/worklog-coverage-remeasures.json`.**
  Resolve a merge conflict in any of them by **union** — keep both sides' entries, ordered by the
  authoring time of each entry's round. Never take one side wholesale; the other side's entries
  vanish silently and the gates stay green.

  **`docs/report/**` is on that list because the round entries moved there** on 2026-09-07 (merge
  `a5091df6`, ticket `wie-report-md-per-round-files-port-from-otterpebble`). `REPORT.md` stays on it
  too — the file still exists as the fixed pointer, and a round that edits the pointer is editing a
  ledger file. **The merge contract's own enumeration (`STATE`·`REPORT`·`docs/worklog/**`·`reports/`·
  `tasks/`) predates that move and does not name `docs/report/**`** — it is rendered from
  `~/orchestrator/templates/merge-ticket.tpl`, outside this repo, so a round that needs the authority
  cannot find it there. This line records the judgement already made rather than making each round
  re-derive it: 2026-09-07 a merge round reasoned it out and chose to *move* the entry (appending to
  `REPORT.md` knowingly breaks a convention that landed 20 minutes earlier; dropping the entry loses
  it), which is the answer — but nothing guaranteed the next round would reach it.

  **`docs/worklog-coverage-remeasures.json` is on that list for the same reason, plus one of its
  own.** It is append-only evidence, so union is the only correct resolution — taking one side drops
  a recorded measurement, and the checker reads `measurements.at(-1)`, so order is load-bearing too.
  The other reason is authority: the merge contract's enumeration names `docs/worklog/**`, and this
  file is a *sibling* of that directory, not inside it. Without this line a gate③ round that must
  discharge an overdue re-measure (see below) has no rule saying it may touch the file, which is
  exactly the gap that left `main` red on 2026-09-07.
- **Follow-up proposals go in a `docs/worklog/*.json`, or they do not exist.** When a task leaves
  follow-up recommendations (or adopts/declines earlier ones), write
  `docs/worklog/YYYY-MM-DD-<slug>.json` in the same PR. The cockpit 「후속 작업 추천」 panel reads
  `.json` in that directory and nothing else — a proposal left only in prose (`REPORT.md`, the done
  reply, a `.md` worklog) never reaches the screen. Schema below.
- **That rule stays *conditional* — and the decision to keep it conditional expires.** 2026-09-01
  declined to make a worklog a per-round mandate, on one number: every round since the convention
  had written one. A number with no re-measure date quietly becomes a permanent rule, so:

  > **Re-measure every 10 landed rounds since `92c25276`, over the most recent 10. Below 70%,
  > re-open the mandate decision.** A landed round is one **first-parent** commit on `main`.

  ```sh
  node scripts/check-worklog-coverage.mjs            # prints the numbers; fails if the promise is overdue
  node scripts/check-worklog-coverage.mjs --record   # discharges it — idempotent, never off-schedule
  ```

  **The commands live in that script, not here** — a second copy would drift from the one CI runs.
  It also mechanizes the *promise*, not the ratio: `engine-contract.yml` fails when 10+ rounds have
  landed with no recorded re-measure, or when a recorded measurement is under the line and nobody
  answered it. It deliberately does **not** fail on the ratio itself, because that obligation is
  conditional — gating PRs on it would rebuild the per-round mandate 2026-09-01 declined. The
  record of each re-measure is `docs/worklog-coverage-remeasures.json`; appending the entry the
  script prints *is* the re-measurement — but **append it with `--record`, not by hand.** The
  obligation is keyed to `origin/main`, so once the cadence is crossed *every* round that pulls base
  gets the same failure and every one of them discharges it honestly: measured 2026-09-06, three
  rounds wrote the same entry (six fields identical, only `decision` differed) and a human stopped
  two of them by hand. `--record` scans the whole record for that `landedRounds` and writes nothing
  if it is already there, so running it twice — or on a base that already carries it — is a no-op.

  **The overdue re-measure belongs to the gate③ round, and "every round handles it honestly" is not
  an owner.** Idempotent `--record` landed on 2026-09-07 (`be37ca7d`) and closed the *duplicate* side
  of this; the same day the other side arrived — landing #53 crossed the cadence, nobody recorded it,
  and `main` went red. Both workflows checkout at `fetch-depth: 0`, so `origin/main` is present in
  `pull_request` runs too: while overdue, **every open PR is red as well**, not just main's badge. So
  the rule is:

  > **A gate③ round that sees `check-worklog-coverage` overdue runs `--record` and bundles that one
  > file into the PR before merging.** Nothing else in the round changes.

  Gate③ is the owner because it is the only role that is *already there* at the moment the obligation
  fires — the crossing round is a landing, and the next thing to touch the repo is another gate③,
  which is also the role the red blocks. It costs one conditional step, once per ten landings, and
  needs no new machinery: the tool exists and is idempotent, so two gate③ rounds racing produce one
  row. **The cost of the alternatives is what rules them out.** A scheduled workflow opening a
  recording PR (⒝) adds an automated PR that itself needs CI and its own gate③ round — more
  machinery for a slower answer. Writing at landing time (⒞) is the only option that closes the
  window completely, and it requires pushing to `main`, which this repo forbids; routed through a PR
  instead it collapses into ⒝.

  **The hole in this choice, in numbers, because it is real.** Gate③ can only act when a gate③ runs,
  so the red persists from the crossing landing until the next one. Measured 2026-09-07 over the 53
  landings since `92c25276`: the cadence of ten took **9h 18m** (#43 04:17 → #53 13:36 KST), the gap
  between consecutive landings is **70.5m median** (34.1m over the last 15) — but the **maximum gap is
  115.2h**. So on a quiet stretch `main` can stay red for days. That is accepted rather than fixed,
  on one observation: the check only goes overdue *by landing*, and the only thing it blocks is
  landing, so during a quiet stretch nothing is waiting on it — and the first round back is the owner.
  If that stops being true (a quiet stretch that blocks something real), the answer is ⒝, not a
  wider tolerance in the checker.

  **`--first-parent` is load-bearing in every one of the script's three counts, and the definition says "first-parent", not
  "squash".** This repo is registered as an upstream-sync fork and must *not* squash-merge, so
  landings arrive as merge commits: PR #69 landed that way on 2026-09-03. Without the flag the
  commands walk every reachable commit — branch commits and, once upstream is merged, thousands of
  upstream commits — and count them as "rounds". Measured on 2026-09-04, one landed merge already
  moved the unflagged answer to **6/5 = 83.3%**, and simulating a single `upstream/main` merge
  takes it to **204/7 = 3.4%**, below the 70% line in one round. The window breaks the same way:
  7 of its top 10 become upstream commits, so `OLD` stops being the 10th landed round.

  **Never let the window reach past `92c25276`** — the 19 rounds before it are 0/19 by
  construction and would trip the rule on history. Baseline 2026-09-04: **5/5 = 100%**; only 5
  rounds exist since the convention, so the first re-measure is due at round 10.
  That first re-measure ran on schedule (round 13, **10/10 = 100%**) and is recorded in
  `docs/worklog-coverage-remeasures.json` — read the numbers there, not here; this paragraph is the
  baseline it started from.
  *Why 10*: the original denominator was **3**, where one round moves the number 33pp and no
  threshold separates a habit from a coincidence; 10 rounds is ≈20 days at the measured cadence
  (15 landed rounds in the 30 days to 2026-09-04). *Why 70 and not higher*: the count above is
  mechanical but the obligation is conditional — a round that left no follow-ups owes no worklog
  and still counts as a miss, so a tighter line fires on rounds that obeyed.

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
disagree. `tracing-attributes = "<0.1.29"` is pinned for a no_std compile error.

**The RustJava `[patch]` fork is gone (2026-09-04); the `rev` pin that replaced it is not
arbitrary.** This row used to justify a `[patch]` table pointing at `Jun025/RustJava`, on the
grounds that it "is not stale duplication". That claim was measured and disproven — the fork was a
subset of upstream on every axis it named, frozen at 2026-07-07 (`docs/upstream-realign-verdict.md`
§4). P1 removed it and pinned `dlunch/RustJava@5b84dd1` (§9). **`5b84dd1` is the last rev before
`current_class_loader` goes private and before `invoke_virtual` gains a `class_name` argument (209
call sites)**, so bumping the pin is a design task, not a one-line edit — §8-4⑶ has the cost per
step. Six hardening axes the fork carried do not exist at that rev; the three whose absence
*panics the host* were re-added in `wie_jvm_support/src/hardening.rs`, which wraps method bodies as
they pass through `find_rustjar_class`. **If you move the pin, re-run that module's tests** — they
assert the guards still attach, and a guard that silently stops attaching is how this hardening was
lost the first time.

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
- `wie_jvm_support` — bridge onto the pinned RustJava JVM; also `hardening.rs`, the null guards that pin does not carry.
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
