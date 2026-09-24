## [2026-09-24] «틱당 벽시계»는 느림과 멈춤을 가르지 못한다 — 반증, 코드 변경 0 (wie-2026-09-20-no-frame-paired-remeasure-window-infeasible-adopt-p0)

**무엇을**: `no frame rendered` 를 «느림»과 «멈춤»으로 가르는 축으로 제안된 «틱당 벽시계»(`ms / ticks`)를 착수 전에 반증했다. 새 필드·새 버킷은 만들지 않았다.

- ⒜ catch-all 은 맞다 — `wie_cli/src/bin/wie_validate.rs` classify 의 최종 `else` 가 `run_err` 없음 · 미종료 · magenta 아님 · `content` 거짓 · `paints == 0` 을 이유 없이 모두 `no frame rendered (hang/black screen)` 로 받는다.
- ⒝ `ticks`·`ms` 는 이미 모든 JSON 줄에 있다 ⇒ 틱당 벽시계는 소비자 쪽 나눗셈 한 번이다. 이 축을 새 필드로 내면 같은 값을 두 번 적는 것이 된다.
- ⒞ **그 축은 묻는 질문에 답하지 못한다** — `Executor::tick`(`wie-backend/src/executor.rs`)은 8 ms 예산의 슬라이스인데, 모든 태스크가 잠들어 있으면 **즉시 반환**하고, 아니면 poll 하나가 끝날 때까지 예산을 넘겨 돈다. 따라서 `ms/tick` 은 «게스트가 계속 runnable 인가(바쁨) / 잠들어 있나(한가)»를 잰다. 진행 여부는 재지 못한다. 돌면서 멈춘 경우는 느린 부팅과 같은 값이 나오고, 기다리며 멈춘 경우(깨어날 일 없는 sleep·pending)는 한가한 정상 게임과 같은 값이 나온다.

| 실측 | 결과 | ticks | paints | ms | ms/tick |
|---|---|---|---|---|---|
| 이번 회차 main `e7ef1eda` · `--inject` · 12개 동시 · load 35→48 | | | | | |
| 일지매영웅전기 ×4 | FAIL `no frame rendered` | 815–843 | 0 | 20028–20120 | **23.9–24.6** |
| 놈3 ×4 | PASS | 81,695–588,528 | 24–108 | 20040–20110 | **0.034–0.246** |
| keydraw_lgt ×4 (정상 픽스처) | PASS | 22.7M–23.6M | 55 | 20013–20019 | **0.001** |
| 2026-09-21 rebaseline(`reports-2026-09-21-aot-java-render-evidence/rebaseline-0210`) | | | | | |
| 놈3 rel3 | **PASS** | **56** | 2 | 20724 | **370** |
| 일지매영웅전기 rel1/rel2/rel3 | FAIL | 502 / 185 / 119 | 0 | ~20,000 | 40 / 109 / 172 |

이번 표본에서는 두 타이틀이 100배 차로 갈린다. 하지만 같은 놈3 가 rel3 에서 **370 ms/tick 으로 PASS** 했다 — 일지매 FAIL 세 번보다 모두 비싸다. 같은 타이틀의 ticks 가 56 → 8.7M(rel2)으로 다섯 자릿수를 오가므로(AGENTS.md 「`ticks` is not a throughput measure」와 같은 현상), 임계를 어디에 두든 한쪽을 오분류한다. 게다가 한 틱이 예산을 넘는 정도는 poll 한 번의 벽시계라서 호스트 부하에 비례한다. 부하에 둔감하지 않다.

**판정**: 전제 ⒝ 는 성립한다(파생값이 이미 있다). 목표 축(부하 둔감 · 느림↔멈춤)은 **이 계기로는 만들 수 없다** ⇒ 티켓 규정대로 반증을 적고 멈춘다. 게이트 판정 변경 0 · 기존 버킷 정의 무접촉이다(⑶ 전/후 불변은 변경이 없으므로 자명하다). 필요한 것은 «진행» 계기이고, 후속 제안으로 넘긴다(worklog `2026-09-24-no-frame-tick-cost-axis-falsified.json`).

**사용자 영향**: 없음(코드 변경 0).
