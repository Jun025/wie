## [2026-09-27] 배틀몬스터 `/ by zero` — 현 main 60회 재현 0 · 조건부 종결 · 코드 변경 0 (wie-2026-09-25-lgt-by-zero-divisor-source-adopt-p0)

**무엇을**: 제안 `2026-09-25-lgt-unwind-consumes-catch-frame#p0`(마을 직후 `ArithmeticException: / by zero` 의 나눗수 출처)를 현 main 에서 재현하려 했다.
**왜**: 게임이 0 으로 나눈 것인지, 우리 스텁이 0 을 돌려준 것인지 가르려면 먼저 재현해야 한다.
**사용자 영향**: 없음. 코드 변경이 없다.

### 계측
- 트리: `origin/main` `96c9f243` + 스크래치 훅(커밋 안 함)
  - `java_raise_arithmetic_exception` 첫 줄에 `tracing::error!("TEMP-DIV0 {}", core.dump_reg_stack(0))` 를 넣었다.
  - 이 SVC(import `0x64,0x25`)는 LGT 에서 `/ by zero` 를 내는 유일한 곳이다(`git grep "by zero"` 기준).
- 양성 대조: 붕어빵타이쿤3 `--inject --timeout 60` 1회에서 훅이 **7회** 걸렸다(LR `0x49fe1` 등).
  - ⇒ 훅은 살아 있다. 0 은 «못 잰 0»이 아니다.

### 재현 시도 — 60회 · 훅 0 · 미처리 `/ by zero` 0

| 경로 | 명령 | 횟수 | 동시 실행 | 시작 load1 | 결과 |
|---|---|---|---|---|---|
| 기본 27키 | `-fix2` run6 명령 그대로(`--inject --boot-secs 6 --action-secs 2 --max-ticks 2000000000 --timeout 300`) | 56 | 4 | 15.3–113.6 | PASS 56/56 · 27/27 단계 · 훅 0 |
| 마을 | #331 `--keys docs/keys/battlemonster-village.keys` + 끝에 `WAIT:60` · 트리에 #331(`64a278dd`) 병합 | 4 | 4(+기본 4) | 15.3 | PASS 4/4 · 97/97 단계 · 789s · 훅 0 |

- 기본 경로의 선택 근거: 과거 유일한 재현은 `-fix2` 증거 `bm_full_run6.txt` 다(load1 135.9 · 61s).
  - 그 명령은 7분 경로가 아니라 기본 27키였다. 그래서 1분짜리 명령으로 횟수를 채웠다.
- 한계 ⑴: 이번 load1 최고치는 113.6 이다. 유일한 재현 판의 135.9–160.3 에는 닿지 못했다.
- 한계 ⑵: 마을 경로는 이 회차에서 스크린샷을 남기지 못했다(`--shotdir` 디렉터리가 생기지 않았다).
  - 그래서 «마을 도달»은 단계 수(97/97)와 #331 의 도달 기록(7/7 · load1 11.6–129.1)에 기댄다.
- 형제 기록과 합치면 #292 이후 재현 시도는 **0/87** 이다.
  - 형제 #306 이 27회 · 이번 회차가 60회.
  - #331 의 마을 7회도 미처리 `/ by zero` 가 0 이었다(단, 훅 없음).

### 판정
- **재현 불가 — 조건부 종결.** 나눗수 출처는 판정하지 않았다(억지 수정 금지).
- 제안이 든 후보 `Font::getDefaultFont`(메트릭 0)는 현 main 기준으로 읽었다(`wie-wipi-java/.../lcdui/font.rs`).
  - `getHeight` 는 MIDP `HEIGHT` 로 위임한다.
  - 0 을 돌려줄 수 있는 것은 `getBaselinePosition`(스텁 0)과 기본 폰트의 `getSize`(필드 0)다.
  - LR 없이 이를 원인으로 지목하지는 않는다.
- **재개 조건**: 어떤 실행이든 `ArithmeticException: / by zero at a.run()V` 가 다시 보이면 된다.
  - 그 실행에 증거의 스크래치 훅을 얹으면 호출부 LR 이 바로 나온다.
- 증적(게임 바이트 0): `~/orchestrator/reports/evidence/wie-2026-09-25-lgt-by-zero-divisor-source-adopt-p0/`
  - 두 묶음 요약 · 양성 대조 덤프 · 훅 패치 · 실행 스크립트
