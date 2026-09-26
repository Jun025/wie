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

**A new lint reddens code you did not write, so "I only touched config" does not exempt you.**
**When beta reddens code you did not touch, split it off — do not fold it into your round.** Open a
separate ticket, cite the failing run in your report, and say plainly that CI is red for a
pre-existing reason.

The marked region above is diffed against `rust.yml` by `wie_cli/tests/dod_ci_parity.rs`, which
`cargo test --all` runs in all six legs. Of the four gates only the lint gate has ever caught anything
on beta that stable missed — that is why beta is one line; add another only when a run makes you.
What the parity test deliberately does not see it prints on every run;
read that block rather than trusting the word "green". Why beta is one line, what it costs, and the
incidents behind it: `docs/report/0277` §A.
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
Those two fixtures paint only in response to a key.

**There is a third verdict, and on this line you should never see it: `UNMEASURED` · rc=2.** It means
`--inject` delivered **zero** keys; `input_steps` / `input_steps_total` say how many landed and `stop`
says which of four things ended the run (`clean exit`, `max-ticks`, `deadline`, `error`). If you see
it here, read `stop`: on `max-ticks` raise `--max-ticks`; on `clean exit` the guest quit during boot
and `--inject` has nothing to say about it. **Do not read it as a FAIL** — it is not a claim about the title.

**Read the count as a floor, not an equality: the verdict is `PASS` · `content true` · rc=0, never
the number.** A lower count is therefore not by itself a regression; a `FAIL`, a blank last frame, or
a non-zero rc is — **but that invariance has a ceiling, and past it the verdict flips too. Do not
trust your own `FAIL` until you have run the four steps below.** Past that ceiling the last key's
paint never lands before the deadline, so count and verdict are **not independent** — it is a **race,
not a threshold**.
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

`ticks` is not a throughput measure — do not derive "this carrier is N× slower" from it; and
`--action-secs` is not a stand-in for real load. A `--boot-secs` × `--action-secs` sweep is a null
result and a noise floor: do not cite that experiment as evidence about `--boot-secs`. Why, with the
numbers: `docs/report/0277` §B.

**`--expect-last-frame` is on the `keydraw_*` line and deliberately NOT on the one above it** — that
line is one loop over three fixtures and `helloworld_*` end blank by construction.

**This is the local net, not the CI one.** Do not read this line as CI enforcement — **no PR-triggered
workflow runs `wie_validate`**. Count with `grep -rn 'wie_validate' .github/`, and read the triggers —
the number alone answers the wrong question.

Two promotions were priced and declined: the runner line to per-PR (2026-09-18 — only
`HeadlessPlatform::font()` is covered per-PR, by `headless_platform_font_measures_text_test`) and
`--expect-last-frame` into CI (2026-09-07). Pricing: `docs/report/0277` §C.
**Reopen if** a regression lands in the text path *outside* `font()` and the weekly job is the thing
that catches it — that is the evidence this trade is wrong, and nothing short of it is.

For the 2026-09-07 `--expect-last-frame` decision:

**Reopen this if any of three things happen** — otherwise a later round will re-propose it from the
same starting point. First and most likely: **a blank-screen regression lands in the window E+F
structurally cannot see** — after the last key assertion, where the loop has already broken. It will
not arrive as a CI failure, by construction; it arrives from the local runner line above, or from
someone running a game, and *that* is the signal to re-price the delta. Second: `HeadlessScreen`
stops being a pure sink, which would open the host axis too. Third: the shipped native host
(`wie_cli`'s `WindowHandle`) needs covering — **note that promoting this flag would not do that
either**, since it exercises `HeadlessScreen`, not `WindowHandle`. That host is covered by neither
net today, and saying so is the honest version of "the local runner is enough".

**This repo's CI runs on GitHub-hosted runners — all of it, and it always has.** Two different
things are called "the runner" around here — keep them apart. ⑴ CI runners are
GitHub's (one fresh VM per job); ⑵ this Mac runs §The four gates and every `local only` script by
hand and is the sibling repos' self-hosted runner. Cost arguments that say "siblings queue behind us"
are about ⑵ and must not be written as if they were about ⑴.

**Do not try to shorten these two runs with `--timeout`** (the `keydraw_*` `--inject` lines). On the `--inject` path the deadline is
rebuilt from the injection schedule (~20 s); `--boot-secs`/`--action-secs` move it, and shortening
them buys time by seeing less.

**To pair a keyed run against an unkeyed one on the same budget, use the three opt-in flags — not
`--timeout` alone:** `--inject --keep-timeout --timeout 60 --shotdir <dir> --shot-every 5`, once with
`--inject-keys 0` and once with `--inject-keys 1`, **on a release build** — a debug run reads an input
wait as a wall (`docs/report/0233`, PR #283; the flags: `wie_validate.rs` header).

**To reach a specific screen, write the path as `--keys <file|list>` — do not carry a scratch patch.**
The fixed 27-key script lands on a different screen under load; three rounds each re-applied the same
uncommitted `WIE_KEYS` patch before the flag existed. Example: `docs/keys/battlemonster-village.keys`.

`cargo test --all` boots KTF and LGT but **nothing in it boots a J2ME guest**. 2026-09-04 shipped a
RustJava pin bump whose four gates were all green while `draw_j2me.jar` failed with
`NoClassDefFoundError` on the first tick — one `wie_validate` line reproduced it locally, and the
round had not run it. Every fixture here is committed; no game files are involved.

**Then read the PR's CI — a local pass is not a CI pass, and CI is the last gate.**

```sh
gh pr checks <n> -R Jun025/wie          # every check, with its conclusion
```

Local green predicts CI green for the four gates and no further — `coverage` (tarpaulin), the
Windows/Ubuntu matrix and `engine-contract` run only up there. **Quote the
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

**There is no caller census any more** — `scripts/checker-census.mjs` and its always-run step were
removed 2026-09-26: it had no failing state, and the 0-caller list it printed was never dispositioned.
To answer "where does this one run?", `git grep -n <script> .github/` and read the triggers.

**Which of these CI actually runs — "the check exists" is not "the check runs".**
`check-engine-contract.mjs` and `contract-roundtrip.mjs` run in `engine-contract.yml`; `build-wasm.sh`
and the frontend build run in `web.yml`; `npm run audit` runs on every PR; `verify-browser.mjs` runs
after every deploy. The one after that does **not** run in CI and is **local-only by design** — do not
"fix" that by wiring it:

- **`npm run verify` (`scripts/verify-browser.mjs`) — runs post-deploy, never on a PR.** It is the last
  step of `web.yml` against the per-deploy URL, and it reports a bad deploy rather than blocking one.

  > **If that step goes red, the gate③ round that landed the merge owns it** — it re-runs the script
  > against the URL the failed step printed and either files a ticket or records in its reply that
  > the deploy is bad.

  The compliance measurement this section once carried, with its re-measure trigger and owner, lives
  in `docs/report/0277` §D — re-measure before you cite it. If a checker is ever built for this, key it off the
  `merged:` sha and GitHub's run list, never off reply wording.

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

  Do not duplicate the command into the template's wording from here — point at this section instead
  (§Constraints' "An external contract is referenced, never copied"). `rc=0` does not mean the screen
  rendered — read it as "booted, took a file, leaked nothing".

  **It writes two screenshots into the repo root, and you do not commit them**:
  `verify_<label>_screen.png` and `verify_<label>_page.png` land beside `Cargo.toml`; `.gitignore`'s
  `/verify_*.png` covers them. Screenshots meant to be *kept* live in `docs/verification/`.

- **`scripts/make-wipi-keydraw-fixture.sh` — local only, and it must never be wired as a
  regenerate-and-compare check.** Its output embeds build paths, so such a step is red by
  construction; what the fixture owes is *behaviour*, which `test_key_reach.rs`/`test_resource_reach.rs`
  and Scenarios E/F assert, and `contract-roundtrip.mjs` fails closed on generator-constant drift.
  Verify a regeneration by re-running the tests, not by diffing the zips. **`keydraw_lgt.zip` has no
  `cargo test` coverage at all** — restoring that test is a separate round. Measurements: `docs/report/0277` §E.

- **`scripts/smoke_gate.sh` — local only, and structurally so.** It regresses the working game
  catalog against `scripts/smoke_gate_baseline.tsv`, reading titles from `WORKING_DIR`
  (default `game_lab/working`). `game_lab/` is git-ignored and holds real game bytes, which
  **Constraint 9 forbids from ever entering the repo, the build output, or any log**. There is no
  version of this check that runs in CI without breaking the constraint it sits beside; the
  committed baseline is identifiers and expected status only, never paths or bytes.

- **`scripts/game-lab-recensus.sh` + `scripts/game-lab-census-map.mjs` — local only, same reason, and
  they are a pair.** The runner re-validates an already-sorted corpus **read-only** and leaves one JSON
  per game in `game_lab/reports-YYYY-MM-DD/`; the generator turns that directory into a per-game bucket
  map via `--reports`. **Pointing the runner's `--out` at `game_lab/reports/` is refused (exit 3) — in
  any spelling.** If you widen a claim here, widen the predicate in the same PR. Start it as
  `bash scripts/game-lab-recensus.sh --dry-run` to see what a full run would cost before spending 45
  minutes **of this Mac** (`docs/report/0173`).

- **`scripts/corpus-name-inflow.mjs` — local only (same reason); a round that reports a "게임
  파일명 유입" number RUNS IT rather than re-deriving the predicate.** `node
  scripts/corpus-name-inflow.mjs` (defaults to this branch's changes against `origin/main`; pass
  paths or `--all-tracked` to widen) reads the whole corpus except `vendor_sdk/` and prints three
  buckets. Report `BOUNDED` **with** `SUFFIX-ATTACHED` beside it — that bucket mixes longer
  *different* titles with real mentions carrying a particle, so "유입 0" while it is non-empty is
  wrong. Commit first (an uncommitted round has 0 subjects), then paste the tool's trailing marker
  line LAST and repaste after any later edit: `scripts/check-inflow-marker.mjs` re-checks its digest
  in `contract` as an **advisory** step (`continue-on-error`, since 2026-09-26) — it proves
  freshness, not truth, and a round without a marker passes. History: `docs/report/0195`–`0196`.

- **`scripts/ktf-image-sweep.py` — local only (it needs a KTF client image, which comes out of the
  git-ignored corpus), and it is the only Python in `scripts/`.** Three sweeps behind one entry
  point: `slots` (every indirect call through an interface table, all slot offsets, with the global
  the table came from), `refs` (who reads/writes given sl-relative globals), `window` (a
  *synchronised* Thumb window ending at an address, literals resolved). Run it as
  `uv run --with capstone python3 scripts/ktf-image-sweep.py <sub> …` — ★`capstone` is **not
  installed** for any `python3` on this machine (measured 2026-09-20), so the bare invocation exits
  **2** and prints that line for you rather than dying at the import. Exit 2 is "could not measure",
  never "found nothing"; there is no failing state on findings, so it is the same class as
  `smoke_gate.sh` above and has zero callers on purpose.
  ★**Read the header before quoting the `slots` argument column**: it is a straight-line model with
  no register liveness and there is a *measured* counterexample in it (`01031C0A:0x128f4a`, where the
  column names the token and the real argument is the path). Confirm anything load-bearing with
  `window`. It exists because these sweeps lived only under `~/orchestrator/reports/evidence/…` until
  2026-09-20, which is the shape this lineage was once rejected for — and the round that moved them
  had itself re-implemented one of the three from scratch a round earlier without noticing it existed.

### Documented-command liveness — one weekly scheduled job, decided 2026-09-10

`.github/workflows/doc-liveness.yml` (weekly schedule + `workflow_dispatch` + a `pull_request`
self-test scoped to the workflow file — **never make this a required check**) executes the executable
lines of this file's fenced `sh` blocks **verbatim — the alias, never the script behind it**.
`scripts/check-doc-liveness-parity.mjs` diffs this file's fenced `sh` lines against the workflow's
`DOC-COPY` regions on every PR, both directions.
**A new fenced `sh` block in this file is therefore parity-checked**: add its lines to the job — or
a `# NOT-RUN: <line> — <why>` declaration beside the copy — in the same PR, or the next PR reddens.
Deliberate non-execution lives as NOT-RUN lines *in the workflow*, next to the copy (the
runner-block NOT-RUN precedent — the checker holds no classification); today: `gh pr checks <n>`
(placeholder argument) and the `$EDITOR` line (interactive). `npm run verify` runs **only on the
schedule event**, against production via `WIE_BASE` — one external touch a week is the cap;
`workflow_dispatch` runs skip it.

Why this placement and not a hook, a PR step, or a gate③ hand-run: `docs/report/0100`, `docs/report/0277` §F.

**The red has an owner** (active since the job landed): **the first gate③ round that runs after a
red weekly run owns it** — read the latest scheduled run alongside the PR's checks
(`gh run list --workflow=doc-liveness.yml -L1`); if red, file a ticket
naming the failing command — do not fix inline, merge tickets do not change code. Same shape as the
owner rule nearby (verify-browser red → the landing gate③): the owner is the role already there. A scheduled red with no owner is how a check dies
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

  **AGENTS.md holds standing rules only** — a round's findings, refutations and measurements go in
  its `docs/report/` file, with at most a one-line pointer here. `engine-contract.yml` caps this file
  at 850 lines.

  **Ask the tool for `N`; do not compute `max + 1` from the directory** — two open PRs computing it
  independently pick the same serial. The post-hoc half runs in CI and reddens a tree that already
  holds a duplicate — **it does not renumber anything, and neither should you renumber a landed
  file**; move the side that has not landed yet. The same check also reddens when another open PR
  holds a serial you added — **the side that claimed later moves**. Read a red here as "you are
  probably the later claimer — move"; if you know you claimed first, tell the PR the message names.

  **`-H` is load-bearing, not cosmetic** (`-h` sorts by the title's date). Sort by the **sequence
  number, not the date** — the ledger's date-monotonicity is a coincidence, not a guarantee.
  `REPORT.md` explains the rest; `docs/report-migration-revert.md` reverts it.

  Why `STATE.md` §진행중 and §완료 are pointers (the shared-insertion-point measurements and the
  2026-09-18 migration): `docs/report/0277` §G.

  **★And do not propose `.gitattributes` `STATE.md merge=union` as the cheap way out** — GitHub's
  server-side merge ignores the attribute, so the PR stays `CONFLICTING` (`docs/report/0277` §G).
  **Do not read a `git merge-tree` exit code as evidence either way — it does not know about the
  branches' attributes.**

  **★When you quote open-PR mergeability, re-fetch the PR refs in the same command that builds the
  table** — `git fetch origin '+refs/pull/*/head:refs/remotes/origin/pr/*'`. A cached ref is how the
  first version of this block reported a PR as conflicting that had been clean for 15 minutes.

  **Cite the per-round file, not `STATE.md:<line>`.**

- **The ledger files of this repo are `STATE.md`, `REPORT.md`, `docs/report/**`, `docs/worklog/**`,
  and `docs/worklog-coverage-remeasures.json`.**
  Resolve a merge conflict in any of them by **union** — keep both sides' entries, ordered by the
  authoring time of each entry's round. Never take one side wholesale; the other side's entries
  vanish silently and the gates stay green.

  **`docs/report/**` is on that list because the round entries moved there** (2026-09-07); `REPORT.md`
  stays on it as the fixed pointer. The merge contract's enumeration (`~/orchestrator/templates/merge-ticket.tpl`
  §2-c⒜) names the same paths as of 2026-09-21. **Keep the date when you
  re-read the canon**: nothing here can detect that it moved (§Constraints — CI never sees
  `~/orchestrator`), so this quote was false for two weeks and stayed green the whole time.

  **`docs/worklog-coverage-remeasures.json` is on that list for the same reason, plus one of its
  own.** It is append-only evidence, so union is the only correct resolution — taking one side drops
  a recorded measurement, and the checker reads `measurements.at(-1)`, so order is load-bearing too.
  The other reason *was* authority: until 2026-09-21 the merge contract's enumeration named
  `docs/worklog/**`, and this file is a *sibling* of that directory, not inside it — so a gate③
  round that had to discharge an overdue re-measure (see below) had no rule there saying it may
  touch the file, which is exactly the gap that left `main` red on 2026-09-07. **That half is
  closed**: the enumeration quoted above now names this file itself. The union reason is not, and
  it is why the line stays.
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
  node scripts/check-worklog-coverage.mjs            # prints the numbers; warns if the promise is overdue
  node scripts/check-worklog-coverage.mjs --record   # discharges it — idempotent, never off-schedule
  ```

  **The commands live in that script, not here** — a second copy would drift. It tracks the
  *promise*, not the ratio, and since 2026-09-26 **overdue is a warning, not a red**: the step left
  the required `contract` job (`wie-meta-gates-trim-after-0921-audit`) after it reddened `main` 8
  times from 09-20 for a deadline no PR had caused. It now runs only in the weekly, non-required
  `doc-liveness.yml`, where overdue prints a `::warning` and an unanswered sub-70% measurement is
  still rc=1. The record is `docs/worklog-coverage-remeasures.json`; **append with `--record`, never
  by hand** — it is idempotent (three hand-appended duplicate rows on 2026-09-06 are why), refuses
  off-schedule rows, and any round may bundle it since the file is a ledger file.

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
| `proposals[].kind` | `"product"` \| `"meta"` (optional) | not read by the consumer; classifies the card. `product` = something a user sees changes; `meta` = checker, guard, census, self-test, ratchet, ledger/doc rule, CI wiring. Omitted = unclassified — **do not backfill past worklogs** |

Locked by `scripts/check-worklog-json.mjs` (run it directly; `engine-contract.yml` runs it on every
PR). It validates every `.json` in `docs/worklog/` and nothing else — **existing worklogs are not
retroactively converted**, and no `.md` sibling is required (wie's worklogs are `.json`-only).

#### Proposal threshold (2026-09-26)

Measured 09-14~24: every adopted proposal spawned **1.40** new ones (highest of 5 repos; above 1 the
chain grows by itself), and 38 of 123 merged PRs did nothing but adopt/decline proposals.

- **0 proposals is the normal value.** Before listing one, answer otterpebble's three questions
  (`otterpebble/.claude/rules/autonomy.md` §제안 등재 문턱): *observed* this round, not a question;
  *actionable now*, not waiting on another landing or a policy call; *not a duplicate* of an open one.
  Any "no" → one line in the done reply, not a card.
- **At most 2 per worklog; census/audit worklogs at most 1** — the rest of the list stays in that
  round's own doc. The 2 is enforced (`check-worklog-json.mjs`, on files a PR adds or grows).
- **A `meta` proposal defaults to 0** unless it changes how battlemonster/KTF/LGT titles run, and
  none at the third generation of a meta chain or later.
- **A proposal a sibling PR already resolved is closed by the round that notices**, in
  `declinedProposals` — keep the ref bare (the consumer matches it verbatim) and put the reason in the
  free-form `"declinedReasons": { "<ref>": "resolved-by #<PR>" }`.

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

**It checks the required-check list only.** A ruleset-shape fingerprint (`.github/branch-protection-expected.json`)
was removed 2026-09-26: only the person who can edit the ruleset could trip it, so it was a reseed
chore, not an alarm.

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

**★Never make `pr-audit` required either, and it carries a second reason `doc-liveness` does not.**
`.github/workflows/pr-audit.yml` runs `cargo audit` on PRs that move `Cargo.toml`/`Cargo.lock`
(wired 2026-09-20, `docs/report/0198`; priced in `0185`: p50 17s, and only 2 of the last 60 PRs
would have fired it). It is paths-filtered, so promoting it deadlocks every PR that misses the
filter — that is reason one, the same as above. **Reason two is specific to this check: `cargo
audit` goes red when a NEW ADVISORY IS PUBLISHED, with no change to this repo at all.** A gating
check a stranger can trip overnight is one people route around, and Constraint 5 has already
closed the usual exits (`--ignore`, `continue-on-error`). ★Non-gating here means **"not in the
required list above"**, not `continue-on-error`: the job's exit code stays honest, it simply does
not hold the merge button. The daily `rust-audit.yaml` remains the alarm channel; the PR job only
buys **attribution** — and it cannot see a dependency that turns vulnerable without `Cargo.lock`
moving (a `[patch]` entry or a git `rev` whose content shifts under the same ref).

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
