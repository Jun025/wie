# AGENTS.md

## Goal

Keep the wie emulator engine correct and shippable on two hosts at once: the native `wie_cli`
desktop host, and the `wie_featurephone` browser host that otterpebble's featurephone shell consumes as a
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
| 7 | `wie_featurephone` is an empty library off `wasm32`. Do not "clean up" the `cfg(target_arch = "wasm32")` gates | `wie_featurephone/Cargo.toml` header; native jobs in `rust.yml` |
| 8 | The exact version pins and the RustJava `rev` pin are deliberate | `Cargo.toml` — comment above the `rev` lines; full rationale in the ledger |
| 9 | No game bytes, ever | `.gitignore` blocklist + `scripts/audit-no-leak.sh`, run on every PR by `engine-contract.yml` — full text below |
| 10 | Secrets are referenced, never embedded or printed | `.dev.vars*` git-ignored + `.claude/settings.json` read-deny — full text below |
| 11 | D1 migrations auto-apply to prod on `main`, destructive statements included — author accordingly | `web.yml:97-100`; `docs/CLOUDFLARE_SETUP.md` |
| 12 | Never commit to `main`; branch → PR, and stop. Merge and branch deletion are a separate approved task | **Nothing machine-locks this** — see Definition of Done |

### Held by you, not by a machine

Quoted in full on purpose — no gate catches these, and a table row would delete the working part.

- **No game bytes, ever.** Game binaries/saves must never enter the repo, the build output, or any log (`.gitignore` blocklist + `scripts/audit-no-leak.sh`). Server-side, the file vault is per-owner isolated with no cross-user identity path — `npm run audit` encodes those checks.
- **Secrets are referenced, never embedded or printed.** `.dev.vars*` is git-ignored (`.dev.vars.example` is the committed template); CI reads tokens from `secrets.*` and gates steps on presence flags rather than echoing values.
- **An external contract is referenced, never copied.** Point at the canon (path + section) and let the `-merge` ticket's `merge_strategy:` frontmatter bind; nothing here can check that, because CI never sees `~/orchestrator`.
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
cargo run -q -p wie_cli --bin wie_validate -- --timeout 5 test_data/text_j2me.jar   # the ONLY fixture that draws text
```

**`text_j2me.jar` is the one fixture that reaches `Platform::font()`, and it exists because nothing
did.** `wie_validate`'s `HeadlessPlatform` shipped for two months with `font()` as `unimplemented!()`:
every guest that drew a string panicked the *validator*, and `classify.sh` recorded that as the
game's fault — the largest failure signature in `game_lab/broken/`. The whole runner block stayed
green through all of it, because `drawString` appeared in **0 of the 6 committed fixtures and 0 of
the 3 generators** (measured). The corpus that did catch it is `game_lab/`, which is git-ignored
under Constraint 9 and can never be in CI. So the guard had to be a fixture, and this is it.

**Why it is a separate jar rather than one more call inside `draw_j2me`, and why `--timeout 5`** —
both measured, neither a preference. Adding the `drawString` to `draw_j2me` moved its pixel stats
(`distinct_colors` 2 → 3, `nondominant_pct` 1.5 → 1.7), and `contract-roundtrip.mjs` asserts *exact*
non-black counts derived from `make-draw-fixture.mjs`'s exports — an amount of glyph ink the
generator cannot predict and so cannot export. A second jar keeps `drawFixtureJar()` byte-identical
(verified by md5) and every existing count intact. The timeout is a budget, not a guess: the paint
lands in about a second, `--timeout 1` is flaky (1/3 FAIL) and 2, 3 and 5 all passed 3/3 at loadavg
124, so 5 is ~5× the observed paint time. Left at the default it would cost **~21 s**; at 5 it costs
**~5.1 s**. Read a FAIL here the way §The four gates says to read any FAIL — re-run before blaming
your diff.

**A fixture that this runner deliberately does not touch is named here, not omitted** — write
`NOT-RUN: test_data/<name> — <why>` inside this marked region. That keeps the classification in the
same document as the list instead of in the checker, which is the one thing the proposal behind this
check warned about: a checker that knows which fixtures are "runner fixtures" becomes a second source
of truth and drifts from this block. There are exactly two today, and they are the same exclusion
for the same reason — `wie_validate` has no real screen to resize:

NOT-RUN: test_data/resize_ktf.zip — it exists to prove `Screen::resize` reaches a real screen, and
`wie_validate`'s own `resize` is a no-op that returns `Ok(())`, so running it here would assert
nothing. Its assertion lives in the browser round-trip (Scenario G), where the canvas is real.
Built by `node scripts/make-resize-fixture.mjs` — the same guest as `helloworld_ktf.zip` plus one
`DisplaySize:` line, byte-stable on regeneration.

NOT-RUN: test_data/resize_draw_ktf.zip — same exclusion, same generator, and the pair to the one
above: that fixture SHRINKS and this one GROWS. The growth is what makes the back buffer visible.
`WebScreen`'s back canvas is created inside wasm and never enters the DOM, so Scenario G — which can
only read `HTMLCanvasElement.width/height` — passes whether or not it moved; a stale back buffer
clips the blit rather than throwing. On a shrink that miss is invisible by construction (an
oversized back canvas is clipped by the front one, and the region copied is exactly the frame), so
only growing exposes it: the area past the OLD size is copied from nothing. Its assertion is
Scenario G2, which samples ALPHA there — `WebScreen::paint` forces alpha opaque across the guest
frame while a freshly sized canvas is transparent, so the probe holds even where the guest draws
nothing. That is also why this one derives from `keydraw_ktf.zip` rather than `helloworld_ktf.zip`:
helloworld never paints, so nothing would be blitted and both branches would read alpha 0.

<!-- ENGINE-RUNNER:END -->

**`keydraw_*` without `--inject` reports FAIL, and that is the CORRECT result — you did not break it.**
Those two fixtures paint only in response to a key, so with no injected input the screen stays black
and the validator is right to say so. Measured 2026-09-06 on both carriers: without the flag
`result FAIL · content false · paints 1`, with it `result PASS · content true · paints 55`. The
misread is not hypothetical — a round chasing an unrelated change stopped on exactly this, took the
FAIL for its own regression, and only cleared it by reproducing the same FAIL on an untouched tree.

**Read the count as a floor, not an equality: the verdict is `PASS` · `content true` · rc=0, never
the number.** What the 1-vs-dozens gap proves is that the flag reached the guest — that gap is the
signal, and it is enormous. There is no exact upper figure, because `paints` counts the ticks that
fit a **fixed** ~20 s budget (the `--timeout` note below derives that budget, and says the wall-time
spread above it is tick overrun under load) — so concurrent work pulls the count straight down.
Measured 2026-09-13 on `keydraw_lgt`: **48–55 idle**, **38–41** with twelve concurrent runs, **28–36**
with thirty (n=30). Across all 42 of those runs the verdict never moved once — **42/42 `PASS` ·
`content true` · rc=0** — and that invariance, not the count, is why the floor is the rule. A lower
count is therefore not by itself a regression; a `FAIL`, a blank last frame, or a non-zero rc is —
**but that invariance has a ceiling, and past it the verdict flips too. Do not trust your own `FAIL`
until you have run the four steps below.** The
same command reported **45 on both carriers** earlier that day, right after `cargo test --all` and
`cargo +beta clippy`: that sits inside the measured range, which is consistent with load and is not,
on its own, evidence of anything in the engine.

**Past that ceiling the verdict is not invariant — measured 2026-09-17 on an untouched `origin/main`
(`75ca3451`), with the documented command and no flags: `keydraw_ktf` **FAILED 4 of 6** runs
(`paints` 11–33) while `keydraw_lgt` passed **6 of 6** (44–55), at load 121–150.** Nothing was
changed, so there was nothing to regress. The 42/42 above was measured at **thirty** concurrent
runs; this is several times that, and it is outside what that sample can speak to.

**The mechanism, so you can reason about it instead of memorising a number.** `paints` counts the
ticks that fit a *fixed wall-clock* budget, so load cuts ticks-per-second while the deadline stays
put. Enough load and the **last key's paint never lands before the deadline** — `last_frame_content`
goes false and `--expect-last-frame` exits 1. Count and verdict are therefore **not independent**:
the floor rule holds only while enough ticks still fit. You can starve the same budget from the
other side with no load at all — `--action-secs 0.02` on an idle-ish tree gave **5/5 FAIL** on
`keydraw_lgt` (`paints` 4–11), and `0.01` passed again, so it is a **race, not a threshold**. That
is also the cheap way to reproduce this class without slowing the machine down for everyone else.

> **So when your run says `FAIL`, do these four before touching your diff:**
> ⑴ **Re-run it several times** — a starved run is not reproducible, a real regression is.
> ⑵ **Compare `paints` to the idle range** (48–55 for `keydraw_lgt`). A `FAIL` at 11 was starved; a
>   `FAIL` at a *healthy* count is the dangerous one — the 2026-09-05 LGT regression had `paints`
>   going **up** (55 → 83) with a blank frame.
> ⑶ **Reproduce on an untouched tree.** This is the only conclusive step.
> ⑷ **Read the load** (`uptime`) — and per §Host performance, read `idle`/`sys`, not the load figure alone.

**One heuristic that looks right and is not: "the other carrier passed, so it is a real bug."** One
carrier failing alone is the *ordinary* starvation signature here, not evidence against it. Use
⑴–⑷, not the cross-carrier comparison.

**That advice is unchanged, but the reason first given for it was wrong, and the correction is the
more useful fact.** This paragraph used to say the fragility is *carrier-specific*, on the strength
of the 4/6-vs-0/6 split above. Re-measured 2026-09-17 with the two fixtures **alternating inside one
loop**, so both see the same load minute: `keydraw_ktf` **5/18 FAIL**, `keydraw_lgt` **5/18 FAIL** —
identical, across three load levels (0/6 and 0/6 unloaded, then 2/6 vs 3/6 and 3/6 vs 2/6). A
carrier-specific effect of the original size would have put ktf near 12/18 and lgt near 0/18, so
that magnitude is excluded; a small difference is not, at this n. **So a lone-carrier failure is
sampling noise, not a property of the carrier** — which makes the heuristic *more* wrong, not less.
The original split is best explained by unpaired sampling: the background load on this machine moves
far more than any knob here, measured swinging between loadavg 50 and 201 *between* legs of one run.

**Two corollaries worth keeping, because both cost a round to learn.** `ticks` is not a throughput
measure — it counts executor spins while the guest is blocked, and across five identical
`keydraw_lgt` runs it read 9,630,471 / 912,303 / 36 / 103,786 / 35. Do not derive "this carrier is
N× slower" from it. And `--action-secs` is not a stand-in for real load: sweeping it 0.60 → 0.05
left both fixtures at 0/5 until 0.05, where both collapse together (ktf 5/5, lgt 4/5).

**Boot is not what runs out, and it cannot be.** The deadline is *defined* as
`min(boot_secs + 0.3 + 27 × action_secs + 1.0, 120)` (`wie_validate.rs`, the `--inject` schedule), so
the slack left after boot is `27 × action_secs + 1.0` — `boot_secs` cancels. (The 120 s cap is a
runaway guard; the cancellation holds while the sum is under it, which every knob setting in this
file is. `27` is the length of that schedule's key array — it is transcribed here, not derived, so
re-count it with the method recorded in `docs/worklog/2026-09-17-keydraw-ktf-load-fragility-refuted.json`
before trusting it if the array has moved.) At `--action-secs 0.05` that is
**+2.35 s no matter what `--boot-secs` says**. An earlier revision of this paragraph said the budget
"no longer covers boot"; that was not off by a margin, it was the wrong category. The algebra is what
settles it, and it had better be — a 2×2 over `boot {2.5, 0.3} × action {0.05, 0.6}`, order-balanced,
**cannot** settle it either way: 48 runs at loadavg 13–95 had *every* cell pass, and a second pass of
the same design at loadavg 105–148 had every cell fail part of the time, the documented `0.6`
**included (2/6)**. A null result and a noise floor; neither attributes anything to boot. Do not cite
that experiment as evidence about `--boot-secs`. Cite it for what it does show, below.

**And the sweep does not show a *different* failure from the load one — it may well be the same one.**
The signature matches on every field the validator reports: same `reason` string, same blank last
frame, overlapping `paints`. What settles it is that *one* configuration produces both outcomes with
only the machine changing under it: measured 2026-09-18, the runs at `--action-secs` **0.05 and below
passed 32/32 at loadavg 13–95** (the 2×2's two low-`action` cells, 12 + 12, plus 8 more at 0.02/0.01),
while at loadavg 105–148 even the documented `0.6` failed **2/6**. *(An earlier revision of this
sentence said 48/48 — that is the 2×2's **whole** run count, and half of it is at `0.6`. The sentence
narrows the population to `≤0.05` but reached for the experiment's headline total; if you cite a
subset, count the subset. The composition is spelled out above so the next reader can check it
against the table in `docs/report/0154`.)* So the knob and
real load push on the same race. That is still a reason not to use the knob as a stand-in — a better
one than "different failure", because it says what the knob actually does: it moves the odds along
the axis you were already on, so a green sweep buys you nothing about the loaded regime.

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
and never runs it. Do not read this line as CI enforcement — **no PR-triggered workflow runs
`wie_validate`**. The `.github/` hits are 3, all of them in `doc-liveness.yml`, whose `pull_request`
trigger is `paths`-scoped to that workflow file: on every other PR it is zero. (This paragraph said
"0 hits across all workflow files" for eight days; it was written 2026-09-07 and `4e39dfaa` put the
runner block into `doc-liveness.yml` on 2026-09-10. Count with `grep -rn 'wie_validate' .github/`,
and read the triggers — the number alone answers the wrong question.)

**One method out of this path IS covered per-PR, and only one.** `HeadlessPlatform::font()` — the
`unimplemented!()` that caused all of the above — is asserted by
`headless_platform_font_measures_text_test` in `wie_validate.rs`, so `cargo test --all` reddens on
all six matrix legs if it is re-broken. **That test guards one method, not the text path**: it goes
through `Platform::font()` and `text_layout::minimum_width` (the call a guest's `drawString`
actually makes) and no further. A regression anywhere else between `drawString` and the screen is
still weekly-only.

**Promoting the runner line itself to per-PR was priced and declined on 2026-09-18**
(`wie-text-drawing-fixture-cheap-tier-vs-broad-tier-decision`), on the same axis as the
2026-09-07 decision below. Two ways to do it, both measured here:

| how | what it costs |
|---|---|
| runner block in `rust.yml`'s legs | the block is **168 s** warm (measured, loadavg 180) × **6 legs** ≈ 17 min of runner time *per PR*, on a self-hosted runner siblings queue behind |
| runner block in `contract` | that job's toolchain is `if: engine == 'true'` and targets **wasm32** with a wasm-keyed cache, so this needs a *native* build. Inside the filter it misses docs-only PRs (the diffs that break wiring); outside it, every PR pays a native cargo build in a job that currently finishes in 12 s on a doc-only diff |

versus the test above: **~10 ms** in an already-compiled target, no new dependency, no new workflow.
**Reopen if** a regression lands in the text path *outside* `font()` and the weekly job is the thing
that catches it — that is the evidence this trade is wrong, and nothing short of it is.

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

**That question is now answered mechanically for the whole tree, so stop grepping it out by
hand: `node scripts/checker-census.mjs`.** It lists every executable artifact in `scripts/`,
`.github/scripts/` and `*/tests/` next to the places that actually run it and the triggers those
places fire on, and the always-run `contract` job prints it on every PR. Two things it is
deliberately not. It is **not a check** — it has no failing state, a zero caller count is a
question and not a defect (this repo ships two checkers that are correctly uncalled, below), and
`continue-on-error` on its step makes that mechanical rather than promised. And it is **not a
replacement for the paragraph below**: it counts call sites, it does not know which of them
matter. Baseline at adoption (`f7a1d022`, re-measured 2026-09-18): **37 artifacts — 8 with no
caller, 14 with exactly one, 15 with two or more**. **That is a reading of one commit, not a
constant** — by `origin/main` of 2026-09-18 the no-caller bucket is already **7**, because a sibling
round revived one orphaned test (`docs/report/0153`). Re-run it rather than quoting this line. It
costs **0.6-2.0 s** in the `contract` job. **Three runs of byte-identical code and output** (PR
#195: `35257783718` **2.04 s**, `35268371028` **1.49 s**, `35270887798` **0.64 s**) — a 3.2x
spread that is runner load, not code. **So do not quote one reading, and do not derive a ratio
from two.** An earlier revision of this paragraph said 0.6 s, then 2.04 s, then 1.5-2.0 s; each
was a true reading and each was wrong as a claim. In particular the 0.6 s predates the
`cargo metadata` subprocess this script now runs — yet it sits *inside* the post-cargo spread, so
the runner figure cannot separate the two versions at all. A dev Mac under load takes 0.9-1.3 s,
which is inside the same band. If you need the cost of the cargo call, measure that call.
**The first published figures — 36/4/14/18 — were wrong and are recorded here as wrong**, because
the census asked a path regex which files `cargo test --all` reaches instead of asking cargo: it
credited four `tests/*.rs` files under directories that carry no `Cargo.toml` (orphans of the base
swap, so cargo compiles none of them) and it dropped `tests/font.rs`, which is a real target of the
root package. The 0-caller bucket was therefore understated by exactly half. The per-row disposition is
`docs/report/0155--2026-09-17--wie-count-checkers-with-only-one-caller.md`, which is also where
its four measured blind spots are written down. Prefer it over a fresh `git grep` when you need
to know where something runs — a hand grep counts prose and comments as wiring, which is how the
count below went stale.

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

  > **If that step goes red, the gate③ round that landed the merge owns it** — it re-runs the script
  > against the URL the failed step printed and either files a ticket or records in its reply that
  > the deploy is bad.

  **That owner rule presumes the round already ran this script once, and measurement says it often
  did not.** This paragraph used to assert the round "is already running the same script against
  production for merge-contract 4-C" — that claim was false and is gone. **The command itself was
  never missing from this file** (the last bullet of this section has carried it all along); what was
  missing is a *verdict* — which output lines count as a pass — and that is what the blockquote below
  adds. **Compliance is roughly half, and the first measurement of it used the wrong denominator —
  read the second row, not the first.** Measured **2026-09-17T08:25Z** over 142 `wie-*merge*.done.md`
  replies — **a snapshot, not a constant; the trigger and owner for redoing it are below the table**:

  | denominator | how it is chosen | n | cite `WIE_BASE` |
  |---|---|---|---|
  | replies containing the literal `Deploy to Cloudflare Pages` | **wording** | 40 | 22 (55%) |
  | replies that **actually landed** (`merged:` is a sha) | **fact** | **112** | **62 (55%)** |

  **Every one of those cells has already been shown to rot — the same predicates, run by three
  different rounds inside one day, moved all of them.** First: 137 · 36/18 (50%) · 107/57 (53%);
  ~9 h later: 138 · 37/19 (51%) · 108/58 (54%); ~6 h after that: the row above. So the trigger is
  keyed to a cadence this file already runs, rather than a new one:

  > **Trigger — two, and the cheap one is unconditional. ⑴ Re-measure before you cite either row
  > anywhere.** ⑵ **Independently, when the worklog-coverage re-measure comes due — the same
  > ten-landing cadence — these two rows are due with it.** Both denominators are one pass over
  > `~/orchestrator/reports/wie-*merge*.done.md`: *wording* is the replies containing the literal
  > `Deploy to Cloudflare Pages`; *fact* is those whose **first 12 lines** carry a `merged:` matching
  > `^[0-9a-fA-F]{7,40}$`; each numerator is the subset that also contains `WIE_BASE`. Overwrite the
  > table and the timestamp — there is no append-only record for this one, by choice.
  >
  > **Owner — deliberately *not* the gate③ round, unlike the cadence it borrows.** That round owns
  > `--record` because `docs/worklog-coverage-remeasures.json` is a ledger file; this table is in
  > `AGENTS.md`, which the merge contract does **not** put in the set a merge round may touch, so
  > naming gate③ here would name someone who is not allowed to do it. The gate③ round that trips the
  > cadence is the **noticer** — it says so in its reply, and the orchestrator tickets a wie round to
  > do the edit. Trigger ⑴ needs no owner at all: the round citing the number is the one re-measuring.

  **Both triggers are prose, and neither is checked. That gap is the cost of writing it this way.**
  The ten landings are enforced by `check-worklog-coverage.mjs` for a *different* metric; nothing
  reads these two rows, so a round can discharge that obligation, leave these cells untouched, and
  nothing reddens. The failure mode is silent and is precisely the one this paragraph exists to name.
  It is left that way on purpose: a checker keyed to reply wording would rebuild row one, and one
  keyed to the `merged:` sha belongs to `orchestrator`, not here — so that axis is carried as a
  proposal instead of only in this prose
  (`docs/worklog/2026-09-17-adopt-gate3-deploy-selfverify-instruction-p0.json`, `target:
  orchestrator`), which is what keeps it on the recommendations panel.

  **The first row is the original 45%-of-33 measurement, and it cannot answer the question it was
  built for.** Two failures, both measured: ⑴ the two gate③ rounds of 2026-09-17 that landed `#177`
  and `#178` **did** run the browser verify and **do** cite `WIE_BASE`, yet neither writes that exact
  phrase — so the predicate drops them from numerator *and* denominator, and compliance can improve
  without the ratio moving. ⑵ `web.yml`'s `push` trigger on `main` carries **no `paths` filter**
  (verified by parsing it), so **every** landing here deploys — confirmed 12/12 on the most recent
  merge replies by asking GitHub whether a `Web` run exists for each reply's `merged:` sha. There is
  no "deploy-bearing" subset in this repo, so a denominator of 40 out of 112 landings is wrong by
  construction.

  **So the obligation is unconditional: every gate③ landing here owes the browser run.** The ~45%
  that did not do it still *reported* a self-verify — by quoting the in-CI step's conclusion and
  curling the alias — which passes "운영 URL" but **not** the "콘솔 0에러" half of merge-contract 4-C,
  because console errors and off-origin requests need the browser run. **If a checker is ever built
  for this, key it off the `merged:` sha and GitHub's run list, never off reply wording** — row one is
  what wording-based predicates do.

  > **So, concretely — every gate③ round that lands runs this and quotes it** (there is no
  > deploy-less landing here — see the table above):
  > `WIE_BASE=https://wie-web.pages.dev node scripts/verify-browser.mjs test_data/helloworld_ktf.zip`
  > — deliberately *inline*, not a fenced `sh` block: fenced blocks in this file are parity-checked
  > against `doc-liveness.yml`, and this line is the alias variant of a command that job already runs
  > on its schedule. Fencing it would add a duplicate obligation, and fencing it *inside a blockquote*
  > would dodge the checker only because its fence regex is `^\s*` (a `>` is not whitespace) — a trap
  > for whoever un-indents it later.
  > Pass is **rc=0**, printed as `NO-LEAK AUDIT: ✅` with `off-origin requests: 0`, `requests whose
  > body contains the game header bytes: 0`, and **`console errors (console.error + pageerror): 0`**.
  > `nonBlack: 0` is **also** a pass here (below). That third line is the "콘솔 0에러" half of 4-C and
  > is exactly the half the measurement above found unmet — so as of 2026-09-17 it is not a line to
  > read but a line the script *judges*: `verify-browser.mjs` exits **3** on a non-zero console-error
  > count, next to **2** for `NO-LEAK AUDIT: ❌ POSSIBLE LEAK`. Rewired rather than left as prose
  > because prose here is unenforced discipline, and the change cost nothing: the last 10 deploy runs
  > and the by-hand alias run all report 0, so no previously-green deploy turns red. Quoting the workflow's own
  > step is necessary but not sufficient: that step reads the **per-deploy** URL, so it cannot see an
  > alias that never swung over — which is the entire reason this by-hand run exists.

  **Why this is written here and not in the merge ticket.** The gate③ obligation lives in
  `~/orchestrator/templates/merge-ticket.tpl` §4-C, which says only "main 자동배포 수반 시
  self-verify(운영 URL·콘솔 0에러) 증빙" — one line, no command, and it is **outside this repo**, so
  no round here can edit it. The *how* is repo-specific and therefore belongs in this file; the
  template's line resolves to this block. Do not duplicate the command into the template's wording
  from here — point at this section instead (§Constraints' "An external contract is referenced,
  never copied").

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
  production after a deploy, which is a *different* assertion; **the blockquote above holds that
  command and its pass criteria, and is the one copy** (this bullet used to repeat the command, which
  is the same two-sources-of-truth trap §Constraints names). And `rc=0` does not mean the screen
  rendered — it exits non-zero on a leak (2), on a console error (3), and on the flow not
  completing, but `nonBlack: 0` is a
  pass, which for the helloworld fixtures is correct since they are expected to end blank. Read it
  as "booted, took a file, leaked nothing". It needs no game file (its default argument is the
  committed `test_data/helloworld_ktf.zip`).

  **It writes two screenshots into the repo root, and you do not commit them**:
  `verify_<label>_screen.png` and `verify_<label>_page.png`, where `<label>` defaults to the game
  file's basename — so the default run leaves `verify_helloworld_ktf.zip_screen.png` and
  `verify_helloworld_ktf.zip_page.png` beside `Cargo.toml`. `.gitignore`'s `/verify_*.png` already
  covers them, so `git status` stays clean and nothing can reach a PR by accident; they are
  transient debugging output, overwritten on the next run. Screenshots meant to be *kept* live in
  `docs/verification/`. This is said here because the surprise is real: a gate② reviewer hit it,
  deleted the two files by hand, and filed it — the script had never announced the side effect.
- **`scripts/make-wipi-keydraw-fixture.sh` — local only, and it must never be wired as a
  regenerate-and-compare check.** It rebuilds `test_data/keydraw_{ktf,lgt}.zip` by cloning
  `dlunch/wipi@068312d`, injecting a guest, and building it with nightly + `rust-src` +
  `-Zbuild-std`. **Execution call sites: 0, on purpose.** Three measurements, in the order that
  matters:

  **⑴ The output cannot be byte-reproduced anywhere, least of all on a runner.** The script's own
  header has said so since it landed ("the output is not byte-reproducible (the build embeds
  paths)... Verify a regeneration by re-running the tests, not by diffing the zips"), and the
  mechanism is visible in the committed artifact: `strings` over the ARM binaries inside those zips
  finds **41 path-shaped strings in `keydraw_ktf.zip` and 39 in `keydraw_lgt.zip`**, including
  `/Users/<dev>/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/src`. A `ubuntu-latest`
  runner embeds `/home/runner/...` instead, so a regenerate-and-compare step is not flaky — it is
  **red by construction, forever**. A permanently red check gets switched off, and then the repo is
  worse off than with no check.

  **⑵ Byte comparison is the wrong question anyway, and the right one is already asked.** What the
  fixture owes is *behaviour*, and both zips are booted and asserted against exact integers:
  `wie-ktf/tests/test_key_reach.rs` and `test_resource_reach.rs` `include_bytes!` the KTF zip
  (so `cargo test --all` reads its bytes on all six matrix legs), and `contract-roundtrip.mjs`
  serves **both** over HTTP and boots them — Scenario E/E-res for KTF, **F/F-res for LGT**.

  **⑶ The generator's constants are fail-closed against the contract.** `contract-roundtrip.mjs`
  parses `BAR_H`, the `KeyCode::X => N` table and the `res.bin` payload back out of this script and
  throws on drift. Verified by mutation, 2026-09-18: renaming `BAR_H` → rc=1 "cannot read `const
  BAR_H: i32 = N;`"; setting `KeyCode::Key1 => 999` → rc=1 "paints 999 ... but
  `contract.keyWipiCodes` is N". That is the live defence, and it is the one to keep working.

  **What none of this catches — say it plainly rather than calling the declaration safety.** The
  guarded property is behaviour, so a regeneration that changes the bytes *without* changing what
  the guest prints or paints is invisible, by design. And one carrier is thinner than the other:
  **`keydraw_lgt.zip` has no `cargo test` coverage at all.** `b52ed661` created
  `wie_lgt/tests/test_key_reach.rs` alongside the KTF one, and the 2026-09-16 crate rename
  (`wie_lgt` → `wie-lgt`) did not carry it over — `wie-lgt/tests/` holds only `test_helloworld.rs`,
  and the two `keydraw_lgt` hits under `wie-lgt/src/` are comments. So LGT's zip is covered by
  Scenario F and the local runner, both of which sit behind a paths filter or a weekly schedule,
  and by nothing that runs unconditionally. **Restoring that test is a separate round** (it is a
  new test file, not a declaration), and this bullet exists so the gap is written down rather than
  rediscovered.

- **`scripts/smoke_gate.sh` — local only, and structurally so.** It regresses the working game
  catalog against `scripts/smoke_gate_baseline.tsv`, reading titles from `WORKING_DIR`
  (default `game_lab/working`). `game_lab/` is git-ignored and holds real game bytes, which
  **Constraint 9 forbids from ever entering the repo, the build output, or any log**. There is no
  version of this check that runs in CI without breaking the constraint it sits beside; the
  committed baseline is identifiers and expected status only, never paths or bytes.

- **`scripts/game-lab-recensus.sh` + `scripts/game-lab-census-map.mjs` — local only, same reason, and
  they are a pair.** The runner re-validates an already-sorted corpus **read-only** (it never moves a
  file, unlike the ingest path `game_lab/classify.sh` — which is itself inside the ignored tree, so
  `git ls-files game_lab` is 0 and nothing there is reviewable) and leaves one JSON per game in
  `game_lab/reports-YYYY-MM-DD/`; the generator turns that directory into a per-game bucket map via
  `--reports`. **Pointing the runner's `--out` at `game_lab/reports/` is refused (exit 3) — in any
  spelling**, because the guard compares resolved paths (symlinks followed) rather than strings; that
  is the July baseline the census's 7/7 reconciliation is checked against, and overwriting it in place
  is the one irreversible thing here. *This sentence used to promise only what the first version
  delivered: gate 2 measured `./game_lab/reports`, `game_lab/./reports`,
  `game_lab/reports/../reports` and the absolute path all sailing past it. The line was true of one
  spelling out of four while reading as if it covered them all — if you widen a claim here, widen the
  predicate in the same PR.* They exist because the 2026-09-18 census wrote its per-game rows to
  a `mktemp -d` it then deleted, which cost the next round an hour and made a low-load re-measure
  impossible; `docs/report/0173` has the numbers. Start it as `bash scripts/game-lab-recensus.sh
  --dry-run` to see what a full run would cost before spending 45 minutes of the self-hosted runner.

- **`scripts/corpus-name-inflow.mjs` — local only (same reason), and ★every round in this lineage
  that reports a "게임 파일명 유입" number RUNS IT rather than re-deriving the predicate.** Call it
  as `node scripts/corpus-name-inflow.mjs` (defaults to the files this branch changed against
  `origin/main`; pass paths, or `--all-tracked`, to widen) and quote its three buckets. **The number
  to report is `BOUNDED`, and it is not reportable alone** — `SUFFIX-ATTACHED` must be quoted beside
  it, because that bucket provably mixes a longer *different* title (`<stem>2`, `<stem>1.04`) with a
  real mention carrying a Korean particle (`<stem>의 …`), and nothing in the shape separates them.

  **Nothing checks that you ran it, and building that checker was priced and declined on 2026-09-20**
  (`wie-corpus-name-inflow-token-boundary-p1`, `docs/report/0195`). Two measurements decided it.
  First, **running a script leaves no trace** — this tool writes **0 files** — so "did this round call
  it" is not observable after the fact; any check must proxy through wording or invent a new artifact.
  Second, the obvious proxy is **wrong where it was tested**: requiring the literal string
  `corpus-name-inflow` flags **3 of the 12** round-doc files that report a number since the tool
  landed, and **all 3 ran it** and said so in their own words ("도구를 실행해서 적었다"). Zero true
  positives. The disease it targets has **never been observed**: 3 of 3 rounds that reported a number
  ran the tool. What *has* been observed, 4 times out of 4, is the **other** failure — the number was
  measured and then invalidated by later edits — and every one was caught before landing (3 by a
  gate② `-fix`, 1 by a self re-measure). That is a different proposal with its own ticket; do not
  solve it here. **Reopen when a round that is NOT in this lineage reports an inflow number** — the
  compliance above is all from the rounds that built the tool, which is the weakest possible sample —
  **or when any round is found to have hand-derived one.** ★This trigger is prose and nothing
  enforces it, the same unchecked-obligation shape the `WIE_BASE` rows above carry; that is the cost
  of not building the checker, and mechanizing the trigger would rebuild the same wording proxy one
  level up.

  ★**Writing "유입 0" while that bucket is non-empty is the exact claim this lineage was rejected
  for.** It exists because the predicate lived only in prose — `docs/report/0173` says "코퍼스 고유
  stem 184개와 NFC 완전일치로 전수 대조", `0170` and `0174` say it in their own words, and nothing
  executed any of them; a plain substring test is what produced 35 hits for a one-syllable stem that
  merely sits inside `인스턴스` and `패턴`. Measured 2026-09-19 over 838 tracked text files: 392
  occurrences split **328 / 48 / 16**. `docs/report/0187` has the split and the false-negative audit.

  **The population is the WHOLE corpus, not `game_lab/broken` — widened 2026-09-20, and that number
  above is from before the widening.** It read `broken/` only, so 266 of the 451 game stems were
  invisible and the honest answer to "is this name in the corpus" was wrong for 59% of it. Not
  hypothetical: `0187`'s own hand-split dismissed 엑스맨3 · 크로이센1.04 · 하이브리드2 ·
  일지매영웅전기2 · 붕어빵타이쿤3작은화면 as "a longer *different* title, measured not to be a corpus
  stem", and **all five are archives in `game_lab/working/`**. **The cost that argued against
  widening was measured at zero where it would be paid**: over the 25 most recent landed rounds, in
  the default mode, `SUFFIX-ATTACHED` — the bucket a human splits by hand — is **identical under
  both populations in all 25**; `BOUNDED` grows by a median of 1 pair (mean 1.44), which is printed
  lines, not work. `vendor_sdk/` is the one excluded bucket, because it is emulator/SDK jars rather
  than games and its stem `agent` is an ordinary word here — it alone adds 21 false
  `SUFFIX-ATTACHED` pairs, more than all of `broken/` produces. The tool prints what it excluded.
  `docs/report/0194` has the per-round table.

- **`scripts/ktf-image-sweep.py` — local only (it needs a KTF client image, which comes out of the
  git-ignored corpus), and it is the only Python in `scripts/`.** Three sweeps behind one entry
  point: `slots` (every indirect call through an interface table, all slot offsets, with the global
  the table came from), `refs` (who reads/writes given sl-relative globals), `window` (a
  *synchronised* Thumb window ending at an address, literals resolved). Run it as
  `uv run --with capstone python3 scripts/ktf-image-sweep.py <sub> …` — ★`capstone` is **not
  installed** for any `python3` on this machine (measured 2026-09-20), so the bare invocation exits
  **2** and prints that line for you rather than dying at the import. Exit 2 is "could not measure",
  never "found nothing"; there is no failing state on findings, so it is the same class as
  `smoke_gate.sh` above and shows up in `checker-census` with zero callers on purpose.
  ★**Read the header before quoting the `slots` argument column**: it is a straight-line model with
  no register liveness and there is a *measured* counterexample in it (`01031C0A:0x128f4a`, where the
  column names the token and the real argument is the path). Confirm anything load-bearing with
  `window`. It exists because these sweeps lived only under `~/orchestrator/reports/evidence/…` until
  2026-09-20, which is the shape this lineage was once rejected for — and the round that moved them
  had itself re-implemented one of the three from scratch a round earlier without noticing it existed.

### Documented-command liveness — one weekly scheduled job, decided 2026-09-10

**Where "the commands this file names still run" gets checked was decided on 2026-09-10: a weekly
scheduled workflow — not a pre-commit hook, not a PR-gated CI step, not a per-landing hand run**
(ticket `wie-doc-named-commands-liveness-placement-decision`, choosing among the four placements
priced in `docs/report/0100`). **The job landed 2026-09-10 as `.github/workflows/doc-liveness.yml`**
(ticket `wie-doc-liveness-weekly-job-implementation`): schedule + `workflow_dispatch` + a
`pull_request` self-test scoped to the workflow file itself (a job edit runs once before landing —
GitHub refuses `workflow_dispatch` for a workflow not yet on the default branch, measured 404;
**never make this a required check**, the paths-filter deadlock in the Incident ledger). It executes
the executable lines of this file's fenced `sh` blocks **verbatim — the alias, never the script
behind it**. The copy question was decided as ⒜ copy-plus-diff (the `dod_ci_parity.rs` precedent; ⒝
extract-and-execute was rejected because a markdown-parsing executor is new machinery whose failure
mode is running garbage or silently skipping, where a diff's failure mode is a red check):
`scripts/check-doc-liveness-parity.mjs` diffs this file's fenced `sh` lines against the workflow's
`DOC-COPY` regions on every PR (an `engine-contract.yml` always-run step), both directions.
**A new fenced `sh` block in this file is therefore parity-checked**: add its lines to the job — or
a `# NOT-RUN: <line> — <why>` declaration beside the copy — in the same PR, or the next PR reddens.
Deliberate non-execution lives as NOT-RUN lines *in the workflow*, next to the copy (the
runner-block NOT-RUN precedent — the checker holds no classification); today: `gh pr checks <n>`
(placeholder argument) and the `$EDITOR` line (interactive). `npm run verify` runs **only on the
schedule event**, against production via `WIE_BASE` — one external touch a week is the cap;
`workflow_dispatch` runs skip it.

Why this placement, in report 0100's numbers — it is the only one that can hold the whole gap. The
gap is the runner block (3 commands, led by a work-tree-writing fixture builder) **plus the alias
layer (≥6: `npm run audit`/`build:wasm`/`frontend`/`verify`, `rustup toolchain install beta`,
`cargo +beta clippy`)** — side-effect commands a hook or PR job cannot contain. A scheduled runner
executes the *documented text verbatim*, which is the entire point: the check asks "does the
documented invocation run", so it must call the alias, never the script behind it — calling the
script is how `matrix.rust: [stable, beta]` stays green while the documented `cargo +beta` line
dies. Blocking power is not being given up where it matters: engine regressions behind the runner
block are already CI-caught twice over (Scenarios E+F — the 2026-09-07 decision above), and what
remains is command-surface rot, whose damage is a later round's time rather than shipped code, so a
≤1-week detection latency is priced correctly. The rejected three: **⒜ pre-commit hook** — 413s +
writer per commit at 10–25 landings/day, and it still cannot hold the side-effect commands, so the
alias layer stays uncovered (maximum price, incomplete coverage). **⒝ paths-filtered CI step** —
buys blocking only for the runner block, at the cargo-build-in-the-node-job price the 2026-09-07
decision already declined, and covering the alias layer would mean CI calling aliases, which
perturbs the `dod_ci_parity.rs` correspondence; plus a mis-scoped filter fails silent (the disease
#130 just fixed). **⒟ gate③ hand-run** — 75s+ × every landing (87–218 min/week against one weekly
run), hand-executes a work-tree-writing command each time (the same objection that barred those
commands from ⒜), relies on unforced discipline, and still leaves the alias 6 uncovered.

**The red has an owner** (active since the job landed): **the first gate③ round that runs after a
red weekly run owns it** — read the latest scheduled run alongside the PR's checks
(`gh run list --workflow=doc-liveness.yml -L1`); if red, file a ticket
naming the failing command — do not fix inline, merge tickets do not change code. Same shape as the
two owner rules nearby (verify-browser red → the landing gate③; worklog-coverage overdue → the next
gate③): the owner is the role already there. A scheduled red with no owner is how a check dies
(2026-09-07, `check-worklog-coverage`).

**Move off this placement if**: runner-block or alias rot lands and burns a round before the weekly
run catches it, twice in a quarter → re-price ⒝ for the runner block only; or a red weekly run sits
unticketed past the next two landings → the owner rule failed; block on it or kill the job rather
than let it be ignored — a periodically-red check that people scroll past is worse than no check.

### Landing paperwork

- **`STATE.md` and `docs/report/` are tracked files, not scratch**: keep `STATE.md`'s 다음 current as a task lands, and write a dated 무엇을·왜·사용자 영향 entry when it lands. **Do not write a 진행중 entry** — that section became a fixed pointer to `gh pr list` on 2026-09-08, for the reason below. **Do not write a 완료 entry either** — §완료 became a fixed pointer to `docs/report/` on 2026-09-18 (ticket `wie-remove-state-md-completed-insertion-point`), so the round file below *is* the landing record and `STATE.md` is no longer touched by an ordinary landing at all. That is the whole point: a landing that touches no shared line cannot invalidate a sibling PR. **Round entries go in a new `docs/report/NNNN--YYYY-MM-DD--<ticket-id>.md` — do not append to `REPORT.md`**, which is now a fixed pointer (2026-09-07; every round appending to one file's top made every open PR conflict — 5/5 at migration time, 4 of them on the ledger files *only*). `NNNN` is the global sequence, largest + 1:

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

  **That answer goes stale while you work, so the same check also compares your serial against the
  *other* open PRs and reddens before anything lands** (2026-09-17). `--next-serial` is true at the
  moment you ask, and the file is committed at the end of the round: measured twice, #164/#165 both
  took `0122` eight minutes apart, and #176/#177 both took `0134` five minutes apart. Widening the
  claim query to pushed-but-unopened branches would not have caught either — the commit→PR gap is
  **p50 28s, max 70s** over 32 PRs, while the gap that bites is ask→commit, i.e. the length of your
  round. So the bare mode now also asks: *does another open PR already hold a serial I added?* If
  yes it exits 1 and names both sides and the move rule — **the side that claimed later moves**. That
  axis is not cosmetic: check runs are pinned to commits, so the later claimer goes red on its *next*
  run while the earlier one **does not know until its own CI runs again**. Keying the rule to the PR
  number instead would tell the side that is already red to sit still and the side that cannot see it
  to act — and the two axes genuinely disagree (2026-09-17: #177 claimed `0134` first, yet #176 is the
  lower number; gate③ moved #176, i.e. the later claimer). Read a red here as "you are probably the
  later claimer — move"; if you know you claimed first, tell the PR the message names. It costs
  nothing on a `main` push (nothing added → no API call) and, like `--next-serial`, a network or git
  failure **says so in the success line** rather than reporting a comparison it never made.

  **`-H` is load-bearing, not cosmetic.** It prefixes the path, so `sort -r` keys on the *sequence number*; `-h` keys on the title text, which is the date, and this repo lands up to six rounds a day. Measured over 54 files: the `-h` form is **52 lines out of place**, the `-H` form is **0**. Sort by the **sequence number, not the date** — the ledger's date-monotonicity is a coincidence, not a guarantee. `REPORT.md` explains the rest; `docs/report-migration-revert.md` reverts it.

  **What actually conflicts is a *shared insertion point*, not a "top".** Measured 2026-09-08 on the
  10 most recent landings that are replayable (the branch tip before it pulled base, merged against
  the `main` it pulled): **10/10 conflicted, and `STATE.md` was the only conflicting file in all
  10** — the `REPORT.md` half is gone, which is what the 2026-09-07 migration bought. Stripping
  §진행중 from all three sides clears **7/10**; stripping §완료 clears **3/10**; stripping both
  clears **10/10**. So the two sections are *each* a contention point and neither alone is
  sufficient — and stripping only the shared "열린 형제 PR: #…" enumeration line clears **0/10**, so
  it is the round *entries* that collide, not that line.

  **Those two numbers are a partition, not two independent readings — quote them together.** The
  seven and the three are disjoint and exhaust the ten: **7 + 3 = 10**, and the set that §완료-
  stripping clears is *exactly* the set that survives §진행중-stripping (measured:
  `6ed4ee8e 480e8654 c2c9552d`). That is forced, not a coincidence: stripping both sections clears
  10/10, so every conflict lives in §진행중 ∪ §완료; nothing is cleared by stripping neither, so
  no pair conflicts outside them. Hence `|A ∪ B| = 10` with `|A| = 7`, `|B| = 3`, and inclusion-
  exclusion gives `|A ∩ B| = 0`.

  **So the pair carries its own check, and you should run it before believing a re-measure:
  `C1_clean + C2_clean ≤ 10`, with equality exactly when the two sets are disjoint.** This is not
  decorative — the first version of this block put §완료's clear count at **four**, which makes
  `4 + 7 = 11` and is arithmetically impossible against its own other three cells. Nobody had to
  re-measure to know it was wrong; the review caught it by arithmetic alone, before measuring.
  Quote one number without the other and that check disappears.

  **This is why `STATE.md`'s 진행중 is a pointer and not an append-only list.** git's three-way
  merge needs exactly **one** unchanged line between two insertions: measured, 0 lines apart →
  CONFLICT, 1 line apart → clean. Appending every round to the *bottom* is still one shared point,
  so it conflicts identically — a same-position insert collides whether the position is the top or
  the bottom. Keying the position off the PR number does not save it either, because sibling rounds
  here carry **consecutive** numbers (#120–#133 measured), which puts their slots back-to-back. The
  only thing that removes the collision is removing the point, which is what the pointer does.
  §완료 had one too, and that was the measured **3/10 residual** — **removed on 2026-09-18**; see the
  superseded-decision banner below.

  > **★SUPERSEDED 2026-09-18 — §완료 is now a pointer too** (ticket
  > `wie-remove-state-md-completed-insertion-point`). The block that follows is kept as the
  > measurement record, not as live instruction: its numbers are still how the collision was priced,
  > but its *verdict* ("keep the insertion point") no longer holds. **What changed is exactly the one
  > thing it named as the blocker** — `docs/report/` was not a superset of §완료, and now it is:
  > re-measured 2026-09-18 over **149 §완료 entries**, every one has a `docs/report/` copy
  > (**사본 없음 0**; the 3 id-less legacy lines are quoted verbatim inside `0092`, `0094`, `0159`).
  > Read the rest for *why* the collision costs what it costs; do not read it as "leave §완료 alone".

  **§완료 keeps its shared insertion point on purpose. That is a decision, not an oversight**
  (2026-09-08, ticket `wie-state-completed-top-insert-residual-three`, adopting
  `2026-09-08-state-in-progress-pointer#p0` as *examined and declined*). Two numbers first, because
  both correct the proposal that asked for it:

  - **The forward rate is not 3/10, it is 1/1.** The 3/10 was a property of a historical sample, not
    a prediction. Measured on `b0a08d73` in a throwaway worktree: two rounds branched independently
    off `main`, each inserting one entry at the §완료 head, **conflict** (`merge-tree` rc=1); move
    one insertion **1 line** down and it is **clean** (rc=0 at 1, 2 and 3 lines). So *any* two
    concurrent rounds collide here, and this repo lands 10–25 rounds a day. Quote 1/1 forward and
    3/10 historical; they answer different questions.
  - **`docs/report/` is not a superset of §완료.** Of 87 entries, 83 carry a ticket id and **3 of
    those have no `docs/report/` file** (`wie-pr45-orphan-close-and-remnant-land-r2`,
    `wie-state-landed-pr56-residue-and-misc-unk9-error-lock`,
    `wie-supply-chain-cargo-updates-a2-a3`); 4 more are short legacy entries carrying no id. So
    pointer-ising §완료 the way §진행중 was pointer-ised would **drop 7 entries that exist nowhere
    else** unless they are backfilled first.

  **That gap is the whole difference between the two sections, and it is why the same prescription
  does not transfer.** §진행중 was a *stale mirror* of `gh pr list` — the round that replaced it
  measured "§진행중 항목 1 → 0, 잃은 서술 0". §완료 is not a mirror; it is the only place 7 of its
  entries live. Removing a point that holds unique content is a migration, not a pointer swap.

  **Distributing the point instead (per-year/per-month subheadings) was rejected by measurement, not
  taste**: the separation needed is 1 line *per concurrent round*, and at 10–25 landings a day the
  rounds that race are in the same day, let alone the same month — so every bucket coarser than
  per-round puts them back on the same line. Per-round files are the only thing that separates them,
  and that is the `docs/state/<round>.md` shape this repo already declined (PR #114).

  **What the collision actually costs, so the next round can re-price it rather than re-derive it.**
  Nothing mechanical: **no checker, workflow, or script reads `STATE.md` at all** — measured over
  344 tracked code/config files, `## 완료` parsers **0**, `STATE.md` mentions **1**, and that one is
  prose in a Rust doc comment. The cost is one union edit inside a base pull the round performs
  anyway. The real risk is that its failure mode is **silent** — take one side wholesale and the
  entries vanish with every gate green — which is why the union rule above demands a both-ways
  preservation count and not "the conflict markers are gone".

  **Reopen this if any of three things change.** First, a machine consumer of §완료 appears — then
  the format question stops being free. Second, someone backfills those 7 entries into
  `docs/report/`; the migration blocker disappears and ⒜ becomes a cheap pointer swap. Third, a
  round loses an entry for real — the silent failure stops being hypothetical and the 1/1 rate makes
  it a matter of time. Absent those, the union edit is cheaper than the migration.

  **★Trigger two has now fired twice, and the blocker is gone — do not re-derive the "7".** PR #140
  (2026-09-08) backfilled 5 and reported the residual as 0; **that 0 was incomplete**, because its
  sweep assumed one §완료 line is one round and the 2026-07-22 line bundles **four** (PRs #36·#39·#42·#43).
  Re-measured 2026-09-18 over **144 entries / 148 report files**: the true residual was **3**
  (`wie-contract-gate-paths-key-mapping`, plus #140's own entry and the same-day
  `wie-main-red-worklog-coverage-overdue-73`), and this round backfilled all three ⇒ **0**. The gap
  also does **not** regenerate: of the **69** entries landed since 2026-09-08, only those 2 same-day
  in-flight ones lacked a file — **67/67 since 09-09 have one**. So "backfill, then it silently
  refills" is not a live objection; what remains is only the migration itself, which is a separate
  round because it too collides with every open PR (landing order is the operator's call).

  **★That migration landed 2026-09-18** (`wie-remove-state-md-completed-insertion-point`). It paid
  exactly the price named above — one final invalidation of every open PR that touches `STATE.md` —
  and in exchange an ordinary landing now touches `STATE.md` **not at all**. Two numbers to inherit
  rather than re-derive: **149/149** §완료 entries had a `docs/report/` copy at migration time
  (so the "drop 7 entries" objection was fully retired, not waived), and the machine-consumer count
  was **still 0** — the single `STATE.md` mention in a code file
  (`wie_midp/tests/create_image_missing_name_message.rs`, a doc comment citing `STATE.md:349`) was
  repointed at `docs/report/0047--…` in the same commit, because a line citation into a section that
  no longer exists is worse than a stale one.

  **★And do not propose `.gitattributes` `STATE.md merge=union` as the cheap way out — it was tried
  and measured on 2026-09-18, and it does not fix the reported symptom.** The symptom is
  `mergeable: CONFLICTING`, and that is decided by **GitHub's server-side merge, which ignores the
  attribute**: with `.gitattributes` committed on the merge base *and* inherited by both sides — the
  exact state you would be in after landing it — `POST /repos/:owner/:repo/merges` still answers
  **HTTP 409 Merge conflict** (measured twice, 2026-09-17 and again 2026-09-18 in that base-inherited
  shape). Git itself resolves that same pair cleanly, which is the point: the capability exists and
  the server does not use it.

  > **Do not read a `git merge-tree` exit code as evidence either way — it does not know about the
  > branches' attributes.** It reads them from **your working tree** (or `--attr-source`), so the
  > same pair flips on whether a file you are not even merging is sitting on disk. Measured on
  > `origin/main` × PR #196: no `.gitattributes` in the worktree → **rc=1**; drop an **untracked**
  > one-line `.gitattributes` there, absent from both merged commits → **rc=0**; delete it → **rc=1**
  > again. An earlier revision of this block cited "`merge-tree` rc=1 with the attribute committed on
  > both branches" as a second proof that union is ignored; that was a measurement error — the rc
  > reported the worktree, not the branches — and it is struck.

  It works in a worktree `git merge`, which buys a cheaper hand-resolution during a base pull, not an
  unblocked PR. Its costs are real too: two differing edits to the *same* line both
  survive **silently** (measured — one `- 열린 형제 PR:` line became two contradictory ones, rc=0, no
  warning), the surviving order is ours-before-theirs rather than the by-authoring-time order this
  ledger's union rule requires, and the merge that *introduces* the attribute still conflicts once.

  **★When you quote open-PR mergeability, re-fetch the PR refs in the same command that builds the
  table** — `git fetch origin '+refs/pull/*/head:refs/remotes/origin/pr/*'`. A cached ref is how the
  first version of this block reported a PR as conflicting that had been clean for 15 minutes.

  **One measured wrinkle worth knowing before you cite `STATE.md` by line.** Top-insert moves every
  line below it, so line-number citations into §완료 rot. `wie_midp/tests/create_image_missing_name_message.rs`
  cited `STATE.md:349`; by 2026-09-08 that line held an unrelated entry and the content it meant had
  moved to 510. **2026-09-18 that citation stopped resolving at all** — §완료 became a pointer and the
  entry it meant lives only in `docs/report/0047--…`, which the same comment already cited, so the
  round that migrated §완료 repointed it there. The rule is unchanged and now unavoidable: **cite the
  per-round file, not `STATE.md:<line>`.**
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

- **The landing strategy is not this file's to state.** Read it off the `-merge` ticket's
  `merge_strategy:` frontmatter; the canon for how that is executed is
  `~/orchestrator/templates/merge-ticket.tpl` §4-A — **outside this repo**, like the other reference to
  it above — and the ticket's declaration wins over any procedure written here. For this repo the value is
  `merge` — **never `squash`**: it is a registered upstream-sync fork, so a squash folds the two parents
  into one and the upstream lineage that every realign round rebases its overlay onto is gone, with no
  way back. That cost is measured, not hypothetical — the sibling `rustjava` lost it on four landings
  running. **This file already said so** under §Definition of Done ("registered as an upstream-sync fork
  and must *not* squash-merge"); what used to stand in this spot was a `--squash --delete-branch` recipe
  that contradicted it, and two gate③ rounds had to override the recipe to land correctly.
- **Do not pass `--delete-branch`**, for the reasons that same §4-A measured: the remote branch is deleted by this
  repo's own `deleteBranchOnMerge` setting, so the flag buys nothing, and it *also* deletes the local
  branch — which may be checked out by another session. Leave no stale merged branches behind, but
  `main` and the merged branch can each be held by a different worktree (measured 2026-09-13: both
  `git checkout main` and `git branch -D` refused), and tidying those is that tree's business, not the
  merge round's.

## Incident ledger (사건 대장)

Measured causes. These are narrative on purpose — a rule without its cause gets reverted by the
next person who finds it inconvenient. Do not compress them into the table above.

**Self-merge, five times.** The cause behind §Git Workflow: five landed changes bypassed the
review gate this exact way. The rule is not "merging is discouraged".

**A paths-filtered required check deadlocks the merge forever.** A required check whose triggers
carry a `paths:` filter never reports on a PR that misses those paths — GitHub shows "Expected —
Waiting for status" indefinitely and the merge button never unlocks. That is why
`engine-contract.yml` filters *inside* the job instead. Adding a `paths:` filter to it looks like
an obvious optimization and is the outage.

**`contract` IS a required check — the switch was thrown on 2026-09-17, and this entry has now been
wrong in both directions.** It said "required" while nothing was; it then said "nothing is required"
for about two hours after a ruleset made five checks required. The list below is the one copy, and
`node scripts/check-branch-protection-claim.mjs` diffs it against the live API both ways.

**Run that guard by hand — CI cannot, and the reason is measured, not assumed.** In Actions the
`github.token` gets **403 "Resource not accessible by integration"** on the branch-protection and
rulesets endpoints (run `35181122022`; the guard exited **2 = COULD NOT MEASURE** rather than reading
a 403 as "nothing enforced"), and `permissions: administration: read` does not help because that key
is not grantable — GitHub rejects the workflow at parse time (run `35180786771`, startup_failure).
Wiring it would need a PAT, a different cost class. So it runs where `gh` is the owner: **when you
edit the block below, and when a merge behaves unlike what this section says.**

**That guard now checks two things, and the second one is not a list.** The block below is one rule
*inside* a ruleset that carries four; the other three — `deletion`, `non_fast_forward`, and
`pull_request` (which holds `required_approving_review_count`, `allowed_merge_methods`, …) — plus
`bypass_actors` and the ref condition could all change while this section stayed true. So the guard
also fingerprints the **whole normalized ruleset** against `.github/branch-protection-expected.json`
and prints the differing line. Measured 2026-09-18: **27 compared leaf fields, 21 of which nothing
watched before**. It is deliberately *not* an enumerated field list — a list is how the next field
GitHub adds slips through (verified: injecting a `required_signatures` rule that does not exist today
still fires). **If the operator changed the ruleset on purpose, update that JSON in the same PR and
say why** — reseed it rather than hand-editing, with
`node scripts/check-branch-protection-claim.mjs --print-current > .github/branch-protection-expected.json`,
then `git diff` that file and re-run the plain check until it prints OK. The guard never writes GitHub,
it only reads. What the normalization drops is stated at the
top of the script — chiefly `id`/timestamps, so **deleting and recreating the ruleset with identical
content is invisible here**; **it also drops every key but `context` inside a rule-parameter array**, so
a `required_status_checks[]` entry gaining an `integration_id` is not compared (exposure today: zero,
each entry carries `context` alone).

<!-- REQUIRED-CHECKS:BEGIN — scripts/check-branch-protection-claim.mjs diffs this list against
     classic protection ∪ active branch rulesets, both directions. Edit this block, not the prose
     around it; other files point here rather than restating (§Constraints). -->

Required on `main` (repo ruleset **"main protection"**, `enforcement: active`, `~DEFAULT_BRANCH`,
created 2026-09-17T10:17:54+09:00 — rules `deletion`, `non_fast_forward`, `pull_request`,
`required_status_checks`; `bypass_actors: []`; `required_approving_review_count: 0`):

- `contract`
- `build-web`
- `rust_ci (ubuntu-latest, stable)`
- `rust_ci (macos-latest, stable)`
- `rust_ci (windows-latest, stable)`

<!-- REQUIRED-CHECKS:END -->

**The three signals disagree, and the disagreement is the trap.** Measured 2026-09-17 12:4x:
`branches/main/protection` still answers **404 "Branch not protected"** and `branches/main` reports
`protection.enabled: false` — because **both report classic protection only, and this is a ruleset**.
`protected: true` and `rulesets` → **1 active** are the signals that see it. An earlier round quoted
the first two and concluded "GitHub-enforced required checks: zero" while five were enforced; ask all
three or do not answer the question.

**This is why the always-run wrapper was worth keeping.** Paths-filtering `contract` now deadlocks
every PR that misses the filter — no longer a hypothetical cost on a future switch-on day. The other
four required contexts are safe for the same reason: `rust.yml` and `web.yml` carry **no `paths:`
filter** on their `pull_request` triggers (measured), so all five always report. **Never make
`doc-liveness` required** — it is paths-scoped by design.

**The rollout that got applied.** The 2026-07-22 round did the code half and left the settings half
as an explicit human-step — *"★human-step (워커 적용 금지 · repo 설정 변경)"*, ruleset JSON included,
PR-before-merge + required checks with review approval deliberately excluded (a sole-owner repo
deadlocks the moment approvals are required, which is why the live ruleset requires **0** approvals).
Report `0005` recorded it unapplied on 2026-08-02; it sat about eight weeks and was applied
2026-09-17. The whole of it lives in `~/orchestrator/reports/wie-main-branch-protection.done.md` §C —
outside this repo, and outside what any round here can execute.

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
- `wie_cli` — native host (also `wie_validate`, a headless triage runner); `wie_featurephone` — browser host, empty library off `wasm32` (Constraint 7; renamed from `wie_web` 2026-09-11 — upstream uses that name).
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
