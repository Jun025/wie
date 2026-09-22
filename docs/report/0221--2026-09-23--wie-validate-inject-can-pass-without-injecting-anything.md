## [2026-09-23] `--inject` 가 키를 한 번도 주지 않고 PASS 를 찍던 자리를 닫았다 — 「못 쟀다」를 세 번째 판정으로 (wie-validate-inject-can-pass-without-injecting-anything)

### 무엇을

`wie_validate --inject` 는 입력 스텝을 **0회** 실행하고도
`PASS · "survived input sequence"` 를 냈다. 그 PASS 가 등재 증거로 쓰일 뻔했다(출처 회차 자수).
판정을 **셋**으로 늘려 그 조합을 `UNMEASURED`(rc 2)로 돌리고, 출력이 **무엇을 했는지**를
말하게 했다 — `input_steps` / `input_steps_total` / `stop`.

| 파일 | 무엇 |
|---|---|
| `wie_cli/src/bin/wie_validate.rs` | 판정 3종 · `inject_unmeasured` · `stop_cause` · JSON 3필드 · 시험 4건 |
| `AGENTS.md` | 러너 블록 옆 9줄 — 「세 번째 판정이 있고, 이 줄에서는 보이면 안 된다」 |

★**엔진 무접촉**(`wie-lgt`·`wie-core-arm` 등 0줄) · 등재 목록 무접촉 · otterpebble 무접촉.

### ⑴ 공허한 PASS — 내 손으로 받았다(3축 한 출력)

★**먼저 재현이 «안 됐다»는 사실부터 적는다.** 출처 회차의 명령 그대로
(`--inject --boot-secs 15 --action-secs 1.2 --timeout 240`, 기본 백스톱)는 **재현되지 않았다**:

```
result PASS · ticks 22,293,707 · ms 48,764 · shots 28     ← 28스텝 전건 실행. 결함 아님
```

★**고쳐져서가 아니라 «머신이 느려서»다.** 백스톱은 **틱** 예산이고 주입 일정은 **벽시계**라,
둘 중 무엇이 먼저 닿는지는 **틱 처리율**이 정한다. 출처 회차는 **50M 틱 / 6.2초 ≈ 8.0M tick/s**,
이 회차는 load 60~81 에서 **22.3M / 48.8초 ≈ 0.46M tick/s**(**17배** 느리다) ⇒ 일정이 먼저 끝난다.

⇒ ★**같은 «비»를 만들어 재현했다 — 백스톱은 «기본값 그대로»(50,000,000) 두고 부팅 창만 넓혔다**:

```
$ wie_validate --inject --boot-secs 600 --action-secs 1.2 --timeout 900 \
    --shotdir <dir> game_lab/broken/lgt/체스마스터.zip

{"result":"PASS","reason":"booted + rendered + survived input sequence (visual correctness NOT checked)",
 "ticks":50000000,"paints":2,"content":true,"ms":43460}
rc=0   shots=0
```

계약이 요구한 3축이 **한 출력에** 있다:
⒜`survived input sequence` ⒝`ms 43,460` ≪ `--boot-secs` 600,000ms ⒞`--shotdir` **0장**.
★그리고 `ticks` 가 **정확히 기본 백스톱**인데 — ★**도구는 그것을 말하지 않는다.**
읽는 사람이 기본값을 외우고 있어야 눈치챈다. 그것이 계약 2⒝가 겨냥한 자리다.

### ⑵ 고른 처방 — `FAIL` 이 아니라 «측정 불가»(rc 2)

★**두 오답의 «크기»가 다르다.** `FAIL` 은 「이 게임이 입력에 죽는다」는 **주장**이다.
입력을 한 번도 받지 않은 타이틀에 대해 그 주장은 **자기가 대체하려는 PASS 와 똑같이 틀렸다** —
PASS 는 근거 없이 등재시키고, FAIL 은 근거 없이 **차단**한다. 이 출력은 실제로 등재 판정
(`CONFIRMED_AOT_JAVA_SHA256`)의 근거로 쓰이므로 그 대칭이 값이다.

★**rc 2 는 발명이 아니라 이 트리의 기존 관용이다** — `AGENTS.md` 가 `ktf-image-sweep.py` 에 대해
「Exit 2 is "could not measure", never "found nothing"」이라고 이미 못박았다. 같은 뜻에 같은 수를 썼다.

★**두 축 모두 fail-closed 다**(하나만으로는 부족하다):
⒜rc 가 **0이 아니다** ⒝`result` 가 **`PASS` 가 아니다** — 사내 호출부 **3곳 전부**
(`smoke_gate.sh` · `lgt_render_probe.sh` · `game-lab-recensus.sh`)가 rc 가 아니라
`"result":"PASS"` **문자열**을 grep 하므로, 문자열 축이 실효 축이다.

★**백스톱 기본값을 올려서 닫지 «않았다»** — 계약이 금지했고, 금지가 옳다.
오늘 8.0M tick/s 인 타이틀이 막히면 내일 12M 인 타이틀이 같은 자리를 연다.
★**결함의 본질은 «값이 작다»가 아니라 «틱 예산이 벽시계 일정을 조용히 잘라낸다»**이다.

집행(계약 2 전 3항):

| 항 | 무엇 | 어디 |
|---|---|---|
| ⒜ | 0스텝 PASS → `UNMEASURED` rc 2 | `inject_unmeasured(inject, passed, steps)` — 단방향(PASS 만 강등) |
| ⒝ | 종료 원인 4종 구별 | `stop` = `clean exit` \| `max-ticks` \| `deadline` \| `error`, ★**전 런에** 출력 |
| ⒞ | 실행된 스텝 «수» | `input_steps` / `input_steps_total` |

★`inject_unmeasured` 를 **단방향**으로 둔 이유는 `last_frame_gate_fails` 와 같다 —
0스텝 **FAIL** 은 부팅 중 크래시라는 **실제 발견**이고, 그것을 「못 쟀다」로 바꾸면 발견을 버린다.
★`--expect-last-frame` 보다 **먼저** 판정한다(`else if`): 0스텝 런에 「마지막 프레임이 비었다」는
더 좁고 더 오도하는 사유다.

### ⑶ 고친 뒤 — 같은 명령, 같은 형상

```
$ wie_validate --inject --boot-secs 600 --action-secs 1.2 --timeout 900 \
    --shotdir <dir> game_lab/broken/lgt/체스마스터.zip

{"result":"UNMEASURED",
 "reason":"--inject delivered 0/27 input steps (run ended: max-ticks) — input survival NOT measured
           (otherwise: booted + rendered + survived input sequence (visual correctness NOT checked))",
 "stop":"max-ticks","input_steps":0,"input_steps_total":27,
 "ticks":50000000,"paints":2,"content":true,"ms":48963}
rc=2   shots=0
```

★종전 사유를 `(otherwise: …)` 로 **보존**한다 — 무엇이 강등됐는지가 보여야 한다.

★**커밋된 픽스처로도 같은 계급을 «코퍼스 없이» 재현할 수 있다**(다음 사람 몫이 싸진다):
`wie_validate --inject test_data/helloworld_ktf.zip` → 종전 `PASS "clean exit"` rc 0 ·
지금 `UNMEASURED · stop "clean exit" · 0/27` rc 2. 1초면 끝난다.

### ⑷ 개악 — 2건, 둘 다 red. ★첫 번째는 «처음엔 통과했다»

★**M1 이 이 회차에서 가장 값한 실측이다.** 계약이 부른 그 형상
(「스텝 계수를 세되 0 을 통과로 접기」)을 호출부 한 자리로 만들었다:

```
-    if inject_unmeasured(args.inject, outcome.passed, outcome.input_steps) {
+    if inject_unmeasured(args.inject, outcome.passed, outcome.input_steps_total) {
```

이름·필드·계수 **전부 옳다**. 그리고 `input_steps_total` 은 `--inject` 런에서 **항상 27** 이라 게이트가 **영영 안 돈다**:

```
{"result":"PASS","reason":"clean exit","stop":"clean exit","input_steps":0,"input_steps_total":27, …}
rc=0                                    ← 0 을 «찍어 놓고» PASS 다. 결함이 그대로 돌아왔다
test result: ok. 14 passed; 0 failed    ← ★그런데 cargo test 는 «초록»이었다
```

★**술어 시험이 술어를 시험하기 때문이다 — 틀린 것은 술어가 아니라 «인자»였다.**
`run()` 은 실제 에뮬레이터가 있어야 돌아 이 파일의 어떤 행위 시험도 그 호출부에 닿지 못한다.
⇒ ★**이 파일에 이미 있는 관용으로 닫았다** — `richness_is_recorded_before_the_gate_judges_test` 가
같은 이유로 `include_str!` 로 자기 소스를 읽는다. `the_gate_is_handed_the_delivered_count_test` 를
추가한 뒤 같은 개악을 다시 얹으니:

```
---- tests::the_gate_is_handed_the_delivered_count_test stdout ----
assertion `left == right` failed: the gate call site is not unique (or no longer reads
`outcome.input_steps`) — a gate fed `input_steps_total` never fires: that field is 27 on every --inject run
  left: 0
 right: 1
test result: FAILED. 14 passed; 1 failed
```

**M2 — 술어 자신의 부호 뒤집기**(`passed` → `!passed`, 한 글자). FAIL 을 강등하고 PASS 는 놓아준다:

```
---- tests::zero_injected_steps_is_not_a_pass_test stdout ----
assertion failed: inject_unmeasured(true, true, 0)
test result: FAILED. 14 passed; 1 failed
```

두 개악 모두 **원본 복원 후** 재확인 — 복원 상태 `15 passed; 0 failed`.

### ⑸ 타이틀별 실패율 — ★계약 2 를 닫은 «뒤에» 쟀다

고친 도구(release) · `--inject` 기본 예산 · **각 12회**(계약 최소 5회) · load 26~64:

| 타이틀 | 실패율 | 무엇 |
|---|---|---|
| 놈3 (`broken/lgt`) | ★**3 / 12 (25%)** | `panic during '<step>': InvalidMemoryAccess(273416884)` |
| 체스마스터 (`broken/lgt`) | **0 / 12** | 이 회차에서는 재현 안 됨(아래) |

★**놈3 의 세 실패는 스텝이 다르고 주소가 같다**:

| 회차 | 스텝 | `input_steps` | shots | ms | 주소 |
|---|---|---|---|---|---|
| 이번 #4 | `11_LEFT` | 11 | 11 | 9,283 | `273416884` = **`0x104C02B4`** |
| 이번 #8 | `09_UP` | 9 | 9 | 7,627 | 동일 |
| 이번 #9 | `09_UP` | 9 | 9 | 7,809 | 동일 |
| 게이트② | `06_OK` | — | 6 | — | 동일 |
| 게이트② | `18_DOWN` | — | 18 | — | 동일 |

⇒ ★**서로 다른 «네» 스텝에서 죽는데 주소는 «하나»다(5/5).** 게이트②의 「타이밍 의존 실결함」
판정이 **더 큰 표본에서 재확인**됐고, 합산하면 **5 / 16**.
★**그 주소가 다음 회차의 표적이다** — 스텝이 아니라 `0x104C02B4` 를 쫓아라.

★**체스마스터 0/12 은 게이트②의 1/3 을 «반증하지 않는다»**(과장하지 마라):
그 실패 모드는 **stack overflow** 이고 ★그것은 `catch_unwind` 로 못 잡는 **프로세스 abort** 라
**JSON 이 아예 안 나온다**. 이번 12회는 전건 JSON 을 냈으므로 「0회 관측」이지 「없다」가 아니다.
(그것을 가리려고 이 회차 harness 는 JSON 부재를 `*** NO JSON ***` 으로 따로 찍게 만들었다 — 0건.)

★**곁가지 1건(기록만)**: 체스마스터 #10 이 `steps 27/27` 인데 `shots 27`(28 아님)이었다.
마지막 Shot 이벤트가 데드라인과 경합한다 ⇒ ★**shots 는 스텝 수의 대용물이 아니다.**
이 회차가 `input_steps` 를 **따로** 센 이유가 정확히 이것이다.

★**엔진은 손대지 않았다**(Non-goal) — 이 절의 산출물은 **수**다.

### ⑹ 하위호환 — 뒤집히는 것이 «있다». 2건이고 둘 다 픽스처다

**축 ⒜ 주입 없는 런**(= `smoke_gate.sh` 의 **게이트 판정**이 쓰는 그 축): ★**판정 변화 0.**
`inject_unmeasured` 는 `inject` 가 거짓이면 절대 발화하지 않는다(시험이 12행 전수로 고정).
JSON 은 **키 3개가 늘었을 뿐**이고, 사내 파서 3종은 전부 키 단위 `grep -o` 또는 `d.get()` 이라 무해.

**축 ⒝ `--inject` 런**: 7월 `--inject` 전수 센서스(`game_lab/reports/` · 452 JSON)로 대조했다.
판별자 = `ms < 2,800`(첫 키가 `boot 2.5 + 0.3` 에 예정되므로 그보다 짧은 런은 **0스텝**이다).

| | 수 |
|---|---|
| `--inject` 모드로 식별된 JSON | 236 |
| 그중 PASS | 185 |
| ★그중 **0스텝**(⇒ `UNMEASURED` 로 뒤집힌다) | ★**2** (1.1%) |

뒤집히는 2건은 **`helloworld_ktf` · `helloworld_lgt`** — 게임이 아니라 **커밋된 픽스처**이고,
`AGENTS.md` 가 이미 「draws nothing at all and exits cleanly」라고 적은 그 둘이다.
★**러너 블록은 그 둘에 `--inject` 를 쓰지 않는다** ⇒ ★**문서화된 러너 명령의 판정 변화 0.**
실행으로 확인: 러너 블록 6줄 전건 종전과 동일(§검증).

★★**그리고 센서스가 «더 큰 것»을 하나 말해 준다 — 이 회차가 닫지 «않은» 몫이다.**
같은 236건 중 **21건**이 기본 백스톱(50,000,000)에 닿았고, 전건 `ms 13.2~17.5초`(데드라인 20.0초)다.
⇒ ★**0스텝은 아니지만 «일정을 다 못 돌고» 「survived input sequence」를 받았다** —
대략 18~25 / 27 스텝. ★**지금은 그 수가 «보인다»**(`input_steps: 22/27`)는 것이 이 회차의 몫이고,
**몇 스텝부터 «쟀다»로 칠지는 임계를 «지어내야» 하므로 정하지 않았다** —
이 파일이 richness 지표에 대해 이미 세운 규율 그대로다(「a number needs a threshold to fail
anything. No threshold is defined here on purpose」). ★**이제 그 임계를 «고를 데이터»가 생겼다**(제안 등재).

★**대가 1건(숨기지 않는다)**: `smoke_gate.sh` 의 `run_one` 은 PASS 아닌 것을 전부 `FAIL` 로 접는다
⇒ `INJECT=1` **권고** 줄에서 `UNMEASURED` 가 `input-advisory=FAIL` 로 보인다.
★**게이트 판정에는 닿지 않는다**(그 줄은 비-게이팅이고, 게이트는 주입 없이 돈다).
고치려면 그 스크립트가 3값을 읽어야 하는데, **이 회차의 범위 밖**이라 제안으로 올렸다.

### 검증

- 4관문 + beta: `cargo fmt --check` / `clippy --all -D warnings` / `clippy --target wasm32 -D warnings`
  / `RUST_MIN_STACK=4194304 cargo test --all` / `cargo +beta clippy --all -D warnings` — 전건 rc 0.
- `wie_validate` 단위시험 **15건** 전건 통과(신규 4건: `zero_injected_steps_is_not_a_pass_test` ·
  `the_gate_is_handed_the_delivered_count_test` · `unmeasured_outranks_passed_in_both_readings_test` ·
  `stop_cause_names_the_terminator_test`).
- AGENTS.md 러너 블록 전건 실행 — fenced `sh` 블록 **추가 0** 이므로
  `check-doc-liveness-parity.mjs` 대상 변화 없음.

### 게임 파일명 유입 — 도구를 «실행해서» 적는다

`node scripts/corpus-name-inflow.mjs --corpus <이 머신의 game_lab>` (모집단 451 stem · 대상 4파일):

**BOUNDED 20회 / 11쌍** · SUFFIX-ATTACHED **5회 / 5쌍** · PREFIX-EMBEDDED 2회/1쌍.
★**두 수를 함께 적는 것이 규약이라 함께 적는다.**

★**갈라 적는다 — 11쌍 중 이 회차가 «새로 들인» 것은 5쌍이다.**
stem 별 20회: `체스마스터` 8 · `놈3` 5 · `놈ZERO` 2 · 나머지 5종 각 1.
파일별 20회: 이 round doc **10** · 이 worklog **4** · `AGENTS.md` **5** · `wie_validate.rs` **1**.

- ★**새 유입 5쌍 / 14회** — `놈3`·`체스마스터` × (round doc · worklog) **4쌍**, 그리고 이 문단이
  `wie_validate.rs` 의 기존 주석어 `놈ZERO` 를 인용하며 생긴 (`놈ZERO`, round doc) **1쌍**.
  ⒸConstraint 9 는 **바이트**를 금하고 이름은 이미 `docs/report/0216` · `docs/lgt.md` ·
  `docs/project-kb/02_status.md` 가 쓰는 축이다. 실패율을 **분수로** 적으라는 것이 계약 4 이므로
  타이틀을 지목하지 않고는 그 수를 적을 수 없다.
- **선재 6쌍 / 6회** — 내가 «고친» 파일 안의 **기존 본문**이다(도구는 수정 파일의 전문을 센다):
  `AGENTS.md` 5쌍은 이 도구 자신을 설명하는 문단의 예시어이고, `wie_validate.rs` 1쌍(`놈ZERO`)은
  기존 시험 주석이다. ★**내 diff 가 더한 «코드» 줄에는 하나도 없다.**
- **SUFFIX-ATTACHED 5쌍은 «전건 선재»** — 다섯 다 `AGENTS.md` 의 그 같은 문단이다. ★내 몫의 판단 대상 0.
- ★**이 문단 자신이 수를 움직였다**(17/10 → 20/11) — 첫 측정 뒤 산문을 더했기 때문이다.
  ★**그것이 이 규약이 겨냥하는 바로 그 실패형**이라 다시 재고 위 수로 갈아 적었다.

### 사용자 영향

없다(엔진 무접촉). 도구 판정만 바뀐다 — 다만 ★**`--inject` 를 쓰는 회차는 이제
`input_steps` 를 읽고 「몇 스텝 돌았는지」를 리포트에 적을 수 있다**. 종전에는 그 수가 없어서
출처 회차가 「28스텝 전건 실행」을 적으려고 `--shotdir` 장수를 세야 했다.
