## [2026-09-25] KTF 영웅서기4 프레임 절반 — 원인은 ARM 선점 × 8ms tick 예산 · 고정 sleep(16) (wie-ktf-frame-pacing-regression-eventqueue-sleep-after-base-swap)

**무엇을**: 두 줄짜리 수정 + 시험 3개.
⑴`wie-backend` executor 의 tick 예산 8ms → **14ms**(`TICK_BUDGET_MS` 상수 · 근거 주석).
⑵`net/wie/EventQueue.getNextEvent` 의 빈 큐 대기를 고정 `sleep(16)` 대신 **가장 이른 대기 타이머까지**(1~16ms)로.
단 callSerially 반복 작업이 남아 있으면 종전처럼 16ms 를 유지한다(12e451cc 가 잠근 콜백 박자 보존).

**왜**: 09-16 base 교체 뒤 KTF 영웅서기4 가 약 35fps → 약 17fps(프레임 간격 p50 17ms → 50ms)로 떨어졌다.
티켓 후보 ⒜ `12e451cc`(EventQueue 콜백 재배치) · ⒝ 동기 `serviceRepaints` 는 **둘 다 원인이 아니다**(되돌려도 3/3 불변).
원인은 upstream 의 **ARM 선점**(`cf13a388` 1000명령마다 yield → `400c3a22` 10,000 → `45d4791e` SVC 를 넘어 예산 유지)이다.
선점 전에는 게임 한 프레임의 계산(부하 하에서 약 12~16ms)이 한 번의 poll 안에서 끝났다. 선점 뒤에는 executor 의
8ms 예산에서 끊겨 **rAF 두 번**에 걸쳐 나뉜다. 여기에 계산이 끝난 뒤의 `sleep(16)` 이 이미 기한이 된 다음 타이머를
**한 tick 더** 늦춘다 ⇒ 게임 프레임마다 3 tick(약 50ms).
(영웅서기4 는 Clet(`client.bin`)이다. 프레임은 `MC_knlSetTimer` 가 돌리고 blit 은 `MC_grpFlushLcd` 가 직접 한다 — 계측 추적으로 확인.)

**사용자 영향**: 영웅서기4(KTF)가 원래 속도(약 35fps)로 돌아온다. 배틀몬스터 · 영웅서기3 · 영웅서기 제로 는 빨라졌고, 라이브 4종은 불변이다(아래 표).
대가: 쉬지 않는(바쁘게 도는) 게스트는 호스트 프레임의 약 84%를 쓴다(종전 약 48%). 그래서 그런 타이틀은 fps 는 그대로인데 CPU 를 더 쓴다(아래 f/cpu 열).

### 측정 — 하네스 = otterpebble 이분탐색 회신의 `harness.html`/`run.mjs` 그대로 · LaunchServices Chrome(`open -na`) · 20초 창 · 교대 짝

원인 확정(영웅서기4 KTF · 짝마다 `local-main` = 수정 전 `origin/main` `1b4944de` 로컬 빌드 · load1 97~265):

| 변형 | 짝 수 | fps (변형) | fps (main) | 간격 p50 |
|---|---|---|---|---|
| ⒜ `12e451cc` EventQueue 되돌림 | 3 | 17.9 · 17.6 · 17.9 | 17.1 · 18.4 · 17.0 | 49.4~49.5 |
| ⒝ `Canvas.serviceRepaints` 동기 페인트 되돌림 | 4 | 17.5 · 16.6 · 17.7 · 17.5 | 17.6 · 17.2 · 17.2 · 18.3 | 49.4~50.0 |
| C1 ARM 선점 끔(`INSTRUCTIONS_PER_YIELD = u32::MAX`) | 4 | **32.9 · 33.0 · 33.5 · 33.8** | 17.1 · 17.9 · 17.5 · 17.5 | **17.3~17.6** |
| C2 sleep → 다음 타이머까지 | 1 | 25.2 | 18.2 | 33.0 |
| C3 tick 예산 14ms | 1 | 22.1 | 17.4 | 34.0 |
| **C2+C3 (= 이 수정)** | 1 | **35.5** | 17.9 | **17.2** |

수정 전/후(최종 diff 빌드 `fix1`):

| 타이틀 | 전 fps | 후 fps | 전 간격 p50/p95/max | 후 간격 p50/p95/max |
|---|---|---|---|---|
| **영웅서기4 KTF** (3짝) | 17.8 · 17.7 · 18.1 | **34.8 · 35.8 · 35.6** | 49.3~49.7 / 116.4~116.7 / 265~310 | **17.1~17.2** / 99.6~99.7 / 231~270 |
| 놈3 (LGT) | 3.0 · 3.0 | 3.1 · 3.0 | 34.7~35.2 / 66.7~68.3 | 34.6~34.9 / 67.9~68.2 |
| 메이플스토리2007 | 11.5 · 11.5 | 11.9 · 11.9 | 83.4~83.6 / 116.5~116.6 | 83.3~83.4 / 100.7~101.0 |
| 현영맞고2006 | 1.9 · 1.9 | 2.0 · 2.0 | 정지 화면(게임이 적게 그림) | 동일 |
| 체스마스터 | 0.8 · 0.8 | 0.8 (1회 — 2회차는 CDP 연결 시간초과로 측정 실패) | 정지 화면 | 동일 |
| **배틀몬스터 (LGT)** | 14.1 · 14.1 | **15.8 · 16.0** | 81.8~82.2 / 99.9~100.1 / 333~351 | **65.7~66.0** / 95.2~98.5 / 251~265 |
| 영웅서기3 KTF | 9.6 · 9.6 | **13.9 · 14.0** | 68.1~68.4 | 51.1~51.2 |
| 영웅서기 제로 KTF | 13.3 | 13.7 · 13.7 | 81.7 | 68.5 |
| 영웅전설4 KTF | `RuntimeError: unreachable` (전 · 후 동일 — 수정 무관한 기존 결함) | | | |

- 부팅 판정 불변: 모든 타이틀의 전 · 후 오류 유무가 같다. 끝 화면 비흑 픽셀은 같은 화면끼리 같다.
- 최대 간격 꼬리(영웅서기4 231~270ms · 기준선 117ms)는 **남았다**. 계측 추적 결과 이 꼬리는 대기 방식과 무관하다.
  **한 프레임의 계산이 약 290ms**(키 입력 직후 1회)이고, 그 사이 tick 하나가 49~72ms 로 예산을 넘는다. 즉 선점되지 않는 구간이다.
  선점을 끈 C1 에서도 max 가 302~317ms 다 ⇒ 이 수정의 범위 밖(후속 제안 p0).

### 시험

- `executor::tests::test_tick_gives_a_yielding_task_most_of_a_60hz_frame` — 예산이 반 프레임(8ms)으로 되돌아가면 red.
  ★**첫 판은 이 줄이 거짓이었다**(게이트② 반려): 단언이 `assert_eq!(polls, TICK_BUDGET_MS)` 라 상수 자신과 비교했고 예산 8 변이에서도 green(검수자 4/4). `-fix` 회차가 리터럴 `14` 로 고쳤다 — 무변이 4/4 ok · `TICK_BUDGET_MS = 8` 변이 → `left: 8 right: 14` FAILED(`mut.log`).
- `event_queue::test::timer_due_mid_slice_fires_without_waiting_for_the_whole_slice` — 변이(고정 16ms 복원) 시 **FAILED** 확인.
- `event_queue::test::recurring_callback_keeps_the_slice_when_a_timer_is_pending` — 변이(콜백 가드 제거) 시 **FAILED** 확인.
- 입력 밀림: `12e451cc` 의 `input_precedes_recurring_callbacks_and_pending_paint` · `recurring_callback_waits_while_the_backend_queue_is_empty`
  · 백엔드 `prioritizes_input_and_coalesces_pending_redraws` · `new_input_precedes_timers_and_notifications_without_reordering_them` **모두 green**.
- 네 게이트 + `cargo +beta clippy` green. runner: draw_j2me · helloworld 2종 · text_j2me PASS.
  keydraw 2종은 `UNMEASURED`(stop=max-ticks · last_frame_content true)였다. **손대지 않은 `origin/main` 트리에서도 교대 3회 모두 똑같이 나왔다**(6/6 · load1 140~260) ⇒ 이 수정과 무관한 부하 현상이다.

### upstream 경계

두 변경 모두 upstream 파일(`wie-backend/src/executor.rs` · `wie-midp/src/classes/net/wie/event_queue.rs`)의 최소 diff 이고 포크 전용 분기가 없다.
제안문 초안은 회신(`reports/…done.md`)에 적었다.
