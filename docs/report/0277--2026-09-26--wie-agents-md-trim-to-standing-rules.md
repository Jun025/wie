## [2026-09-26] AGENTS.md 를 상시 규칙으로 — 1,353 → 842줄, 옮긴 644줄은 이 파일에 원문 그대로 · 줄 수 상한 850 (wie-agents-md-trim-to-standing-rules)

**무엇을** — 모든 wie 세션이 기동 때 읽는 `AGENTS.md`(`CLAUDE.md` = `@AGENTS.md`)에서 회차 발견·반증·측정 이력을
걷어 이 파일 아래 «옮긴 원문»으로 옮겼다(19구간 · 644줄 · **원문 그대로**). 원자리에는 상시 규칙과 `§A`~`§H` 포인터만 남겼다.
`engine-contract.yml` `contract` 잡에 줄 수 상한 단계 1개(850)를 더했다. 규칙 «내용» 변경 0 — 옮기기만 했다.

**왜** — `origin/main 5434ab0a` 에서 **1,353줄 · 110,541B**(티켓 발권 시 1,320줄 · 107,670B 에서 더 늘었다). 매 세션이 그 전부를 읽는다.

### 측정

| 축 | 전 (`5434ab0a`) | 후 |
|---|---|---|
| 줄 · 바이트 | 1,353 · 110,541 | **842** · `wc -c` 는 PR 참조 |
| 표식 블록 `COMMIT-GATES` · `ENGINE-RUNNER` · `REQUIRED-CHECKS` | 23 · 60 · 15줄 | **바이트 동일**(블록 추출 diff 빈 출력) |
| fenced `sh` 블록 | — | **바이트 동일** · `check-doc-liveness-parity` 26줄 OK |

### AGENTS.md 를 «읽는» 기계 전수 (`git grep -n 'AGENTS.md' origin/main -- scripts .github '*.rs'`)

| 읽는 자리 | 읽는 것 |
|---|---|
| `wie_cli/tests/dod_ci_parity.rs` | `COMMIT-GATES:BEGIN/END` 구간(과 그 안의 명령 문자열 2개로 개악 대조) |
| `scripts/check-engine-runner-fixtures.mjs` | `ENGINE-RUNNER:BEGIN/END` 구간 · `NOT-RUN:` 줄 |
| `scripts/check-branch-protection-claim.mjs` | `REQUIRED-CHECKS:BEGIN/END` 구간의 `- ` 줄 |
| `scripts/check-doc-liveness-parity.mjs` | **파일 전체의 fenced `sh`/`bash` 블록**(위치 무관) ⇒ `sh` 블록은 옮길 수 없다 |
| 워크플로 단계 이름·스크립트 메시지 | 절 제목(§The four gates · §Web-surface commands · §Documented-command liveness · §Landing paperwork · §Proposal threshold · Incident ledger) — 전부 유지 |

### 건드리지 않은 절 — 형제 두 회차 소관(착지 전)

`wie-meta-gates-trim-after-0921-audit`(queue/wie-2) · `wie-meta-machinery-cleanup-census-step-orphan-tests-ruleset-shape`(queue/wie-4)
가 도구를 지우며 같은 절을 고친다 ⇒ 동시 편집 충돌을 피해 **원문 그대로** 두었다: checker-census 30줄 · corpus-name-inflow 47줄 ·
inflow 표식 19줄 · worklog-coverage 76줄 · branch-protection 가드/ruleset 지문(REQUIRED-CHECKS 포함) 47줄 = **219줄**.
★500줄 목표에 못 닿은 몫의 대부분이 이것이다 — 두 회차가 착지하면 상한을 그 회차가 함께 낮춘다.
사건 대장(Incident ledger)은 자기 규칙 「narrative on purpose … Do not compress them」 때문에 그대로 두었다.

### 규칙 보존 대조

명령형 문장 추출(must/never/do not/don't/always/하라/마라 · 명령형 동사 시작) — 전 160 · 후 132. 전에만 있는 36건 분류:
- 절이 그대로 남음(문장 경계·마침표만 다름) 6 — M06·M18·M21·M22·M23·M24
- 관찰(«never/must» 가 든 서술 — 규칙 아님) 16
- 옮긴 측정·이력의 «인용법» 규칙(그 측정과 함께 옮겨졌다) 12
- 티켓 지시로 줄인 문면 2 — #283 짝 레시피 ≤3줄(«on a release build» 유지) · 합집합 규칙은 「Never take one side wholesale」로 유지
- 은퇴 도구로 사라진 규칙 0
전체 목록은 회신 증적(`rules-missing.txt` · `rules-added.txt`).

### 옮기다 본 모순 — 고치지 않았다(목록만)

1. §Landing paperwork 첫 줄 「write a dated 무엇을·왜·사용자 영향 entry when it lands」 ↔ 같은 줄 「Do not write a 완료 entry either … `STATE.md` is no longer touched by an ordinary landing at all」.
2. 같은 줄 「`NNNN` is the global sequence, largest + 1」 ↔ 바로 아래 「do not compute `max + 1` from the directory」.
3. `docs/worklog-coverage-remeasures.json` 절 「the checker reads `measurements.at(-1)`」 — 2026-09-21 worklog 가 이미 낡았다고 적었다(동결 절 인접이라 무접촉).
4. 남긴 WIE_BASE 인용 블록의 「see the table above」·「the measurement above」 와 「Keep the date …」 문장의 「this quote was false for two weeks」 — 그 표·이력은 이제 이 파일 §D·§H 에 있다.
5. Incident ledger 의 「Do not compress them」 ↔ 이 티켓의 「이력은 report 로」.

### 검증

상한 단계 양방향(워크플로의 `run:` 본문을 파일에서 뽑아 실행): 그대로 rc=0 · 851줄 rc=1(`::error`) · 되돌림 rc=0 · 정확히 850 rc=0.
`check-doc-liveness-parity` · `check-engine-runner-fixtures` · `check-branch-protection-claim` rc=0.

---

### 옮긴 원문 (verbatim · AGENTS.md@5434ab0a)

#### A · The four gates — the beta axis, its cost, and the parity lock — AGENTS.md@5434ab0a:77-116

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

#### B · Reading the runner — keydraw without --inject; UNMEASURED; floor — AGENTS.md@5434ab0a:181-199

Those two fixtures paint only in response to a key, so with no injected input the screen stays black
and the validator is right to say so. Measured 2026-09-06 on both carriers: without the flag
`result FAIL · content false · paints 1`, with it `result PASS · content true · paints 55`. The
misread is not hypothetical — a round chasing an unrelated change stopped on exactly this, took the
FAIL for its own regression, and only cleared it by reproducing the same FAIL on an untouched tree.

**There is a third verdict, and on this line you should never see it: `UNMEASURED` · rc=2.** It
means `--inject` delivered **zero** keys, so the run has no opinion about input survival — the
`input_steps` / `input_steps_total` fields on every JSON line say how many actually landed, and
`stop` says which of four things ended the run (`clean exit`, `max-ticks`, `deadline`, `error`).
The case that produced it was a title fast enough to burn the `--max-ticks` backstop before the
first key was due, which reported `PASS ... survived input sequence` over 0 keys and 0 `--shotdir`
frames (2026-09-22, nearly used as registration evidence). If you see it here, read `stop`: on
`max-ticks` raise `--max-ticks`; on `clean exit` the guest quit during boot and `--inject` has
nothing to say about it. **Do not read it as a FAIL** — it is not a claim about the title.

**Read the count as a floor, not an equality: the verdict is `PASS` · `content true` · rc=0, never
the number.** What the 1-vs-dozens gap proves is that the flag reached the guest — that gap is the
signal, and it is enormous. There is no exact upper figure, because `paints` counts the ticks that

#### B · Reading the runner — floor ranges, ceiling, mechanism — AGENTS.md@5434ab0a:200-226

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

#### B · Reading the runner — carrier claim refuted, corollaries, boot algebra, sweep, --expect-last-frame placement — AGENTS.md@5434ab0a:239-297

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

#### C · local net vs CI, font() test, runner-line promotion priced, GitHub-hosted runners, --expect-last-frame into CI — AGENTS.md@5434ab0a:298-381

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
| runner block in `rust.yml`'s legs | the block is **168 s** warm (measured, loadavg 180) × **6 legs** ≈ 17 min of runner time *per PR* |
| runner block in `contract` | that job's toolchain is `if: engine == 'true'` and targets **wasm32** with a wasm-keyed cache, so this needs a *native* build. Inside the filter it misses docs-only PRs (the diffs that break wiring); outside it, every PR pays a native cargo build in a job that currently finishes in 12 s on a doc-only diff |

versus the test above: **~10 ms** in an already-compiled target, no new dependency, no new workflow.
**Reopen if** a regression lands in the text path *outside* `font()` and the weekly job is the thing
that catches it — that is the evidence this trade is wrong, and nothing short of it is.

> **★This repo's CI runs on GitHub-hosted runners — all of it, and it always has.** Measured
> 2026-09-20 over every workflow: **18 `runs-on:` values**, all `ubuntu-latest` / `macos-latest` /
> `windows-latest` (`rust.yml`'s `${{ matrix.os }}` is those three), **`self-hosted` 0**, and
> `git log -S'self-hosted' -- .github/` is **empty across the entire history** — it was never true
> here, not once. The row above used to end "on a self-hosted runner siblings queue behind"; ★**the
> decision it supports is unchanged and so is its number (168 s × 6 legs ≈ 17 min) — only the
> resource clause was wrong.** It came from the sibling repo, which genuinely does run self-hosted
> (measured the same day: otterpebble has `self-hosted` in 6 workflow files and 37 jobs behind a
> `vars.CI_RUNS_ON` switch), and a sentence true there was copied into a repo where it is not.
> ★**Two different things are called "the runner" around here — keep them apart.** ⑴ **CI runners**:
> GitHub's, one fresh VM per job, so a long job here does *not* make a sibling PR wait. ⑵ **This
> Mac**: where §The four gates and every `local only` script above are run *by hand*, and which
> *is* a self-hosted runner — for the sibling repos. Saturating it costs you and them; it does not
> queue this repo's PRs. Cost arguments that say "siblings queue behind us" are about ⑵ and must
> not be written as if they were about ⑴.

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

#### C · --timeout on the --inject path; pairing recipe (PR #283) in full — AGENTS.md@5434ab0a:392-414

**Do not try to shorten these two runs with `--timeout`.** On the `--inject` path that flag is
overwritten: the deadline is rebuilt from the injection schedule (`--boot-secs 2.5` + 0.3 + 27
steps × `--action-secs 0.6` + 1.0 = **20.0 s**), so `--timeout 5` and `--timeout 20` both take ~20 s
(measured). The knobs that do move it are `--boot-secs`/`--action-secs`, and shortening them drops
paints (`--boot-secs 1.0` → 18.7 s, paints 55 → 37), i.e. it buys time by seeing less. Measured wall
time over six runs each: KTF **20.1–26.1 s**, LGT **20.2–21.4 s** — the spread above 20.0 is tick
overrun under load, not budget starvation.

**To pair a keyed run against an unkeyed one on the same budget, use the three opt-in flags — not
`--timeout` alone.** `--inject-keys N` injects only the first N keys of the script, `--keep-timeout`
ends at `--timeout` instead of the schedule-derived deadline, and `--shot-every SECS` adds a
`--shotdir` frame every SECS (`<stem>__tNNN.N.png`). Answering "is a 3-paint title waiting for a
key, or stuck?" is `--inject --keep-timeout --timeout 60 --shotdir <dir> --shot-every 5` run twice,
with `--inject-keys 0` and `--inject-keys 1`: only the key differs. **Run it on a release build** —
`cargo run --release -q -p wie_cli --bin wie_validate -- …` or `target/release/wie_validate` — not the
debug `cargo run -q` of the runner lines above: measured on 배틀몬스터 at gate② (PR #283), debug gave
paints 3 for **both** N=0 and N=1 (and still 3/3 at `--timeout 180`) while release split them 3 vs 82,
because debug does not reach the guest progress this pairing needs in 60 s — so a debug run reads an
input wait as a wall, the exact misread this recipe exists to prevent. That question used to need an
uncommitted 13-line patch and a release relink (`docs/report/0233`). All three are off by default,
so the runner lines above are unchanged; `--inject-keys 0` has nothing to deliver, so it reports
`input_steps_total 0` and the verdict says nothing about input.

#### C · jobs that only run in CI — AGENTS.md@5434ab0a:426-430

Local green predicts CI green for the four gates and no further. Three jobs run only up there and
have each gone red on a locally-green branch: `coverage` runs the tests under `cargo tarpaulin`
(a different execution engine — 2026-09-04 a test that passes natively segfaulted under it),
`rust.yml` runs the matrix on Windows and Ubuntu (2026-09-04 a `:`-vs-`;` path separator passed on
macOS and panicked on Windows), and `engine-contract` diffs the WASM export surface. **Quote the

#### D · which web-surface commands CI runs; verify-browser — where it runs — AGENTS.md@5434ab0a:483-494

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

#### D · verify-browser — gate③ compliance measurement, triggers, owner, first-row critique — AGENTS.md@5434ab0a:499-559

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

#### D · verify-browser — why here not in the template, PR stance, what it does not tell you — AGENTS.md@5434ab0a:579-607

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

#### D/E · verify-browser screenshots; make-wipi-keydraw-fixture.sh head — AGENTS.md@5434ab0a:609-618

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

#### E · make-wipi-keydraw-fixture.sh — why it is never a regenerate-and-compare check — AGENTS.md@5434ab0a:619-655

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

#### E · game-lab-recensus — read-only runner, --out guard history — AGENTS.md@5434ab0a:663-681

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
  --dry-run` to see what a full run would cost before spending 45 minutes **of this Mac** — which is
  where it runs, not a CI runner (this repo's CI is entirely GitHub-hosted; see the note under
  §Definition of Done's cost table).

#### F · doc-liveness — placement decision and landing — AGENTS.md@5434ab0a:748-761

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

#### F · doc-liveness — why this placement (report 0100's numbers) — AGENTS.md@5434ab0a:770-789

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

#### G · report serial; STATE.md shared insertion points, §진행중/§완료 pointer migration — AGENTS.md@5434ab0a:813-948

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

#### G · merge=union measured; merge-tree reads the worktree — AGENTS.md@5434ab0a:949-972

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

#### G · STATE.md line citations rot — AGENTS.md@5434ab0a:976-983


  **One measured wrinkle worth knowing before you cite `STATE.md` by line.** Top-insert moves every
  line below it, so line-number citations into §완료 rot. `wie_midp/tests/create_image_missing_name_message.rs`
  cited `STATE.md:349`; by 2026-09-08 that line held an unrelated entry and the content it meant had
  moved to 510. **2026-09-18 that citation stopped resolving at all** — §완료 became a pointer and the
  entry it meant lives only in `docs/report/0047--…`, which the same comment already cited, so the
  round that migrated §완료 repointed it there. The rule is unchanged and now unavoidable: **cite the
  per-round file, not `STATE.md:<line>`.**

#### H · ledger path list — authority history — AGENTS.md@5434ab0a:1009-1025

  **`docs/report/**` is on that list because the round entries moved there** on 2026-09-07 (merge
  `a5091df6`, ticket `wie-report-md-per-round-files-port-from-otterpebble`). `REPORT.md` stays on it
  too — the file still exists as the fixed pointer, and a round that edits the pointer is editing a
  ledger file. **The merge contract's own enumeration has since caught up and now names both, so a
  round that needs the authority *can* find it there.** Measured 2026-09-21 at
  `~/orchestrator/templates/merge-ticket.tpl` §2-c⒜ — outside this repo, which is why this is a
  quote with a date on it and not a check — it reads `STATE`·`REPORT`·`docs/report/**`·
  `docs/worklog/**`·`docs/worklog-coverage-remeasures.json`·`reports/`·`tasks/`.

  **It did not when this paragraph was written, and the dates are the point.** `docs/report/**`
  arrived 2026-09-07 (`orch-merge-template-ledger-path-list-lacks-docs-report`, adopting a proposal
  this repo raised) and `docs/worklog-coverage-remeasures.json` only on 2026-09-21
  (orchestrator-ops PR #1087, merge `74fac0820`). Through that gap this line was the only written
  authority, and it records a judgement already made rather than making each round re-derive it:
  2026-09-07 a merge round reasoned it out and chose to *move* the entry (appending to `REPORT.md`
  knowingly breaks a convention that landed 20 minutes earlier; dropping the entry loses it), which
  is the answer — but nothing guaranteed the next round would reach it. **Keep the date when you


**게임 파일명 유입**(`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` 실행값): BOUNDED·SUFFIX-ATTACHED 는 아래 표식 그대로다.
전건 `AGENTS.md@5434ab0a` 에 이미 있던 문장이 «옮긴 원문»으로 이 파일에 들어온 것이다 — 새 이름·파일 바이트·경로 유입은 없다.


<!-- corpus-name-inflow v1 subjects=3 tree=e8f6dcbb684abd6b B=6/6 P=2/1 S=5/5 -->
