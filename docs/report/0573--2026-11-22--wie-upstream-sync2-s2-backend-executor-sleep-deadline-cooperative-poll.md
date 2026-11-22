## [2026-11-22] upstream 의 실행기 시간 의미를 우리 pacing 아래로 — sleep 마감은 «부른 때»부터 · 깨워진 태스크만 틱 안에서 다시 돈다 (wie-upstream-sync2-s2-backend-executor-sleep-deadline-cooperative-poll)

★**판정: 지금 상태로는 착지시키지 않는다 — 운영자 결정이 먼저다.** 티켓의 기준(«`perf-watch compare` 가 그대로 · 허용 오차 밖이면 착지 불가»)에 걸린다. `perf-watch compare` 15칸 중 14칸은 ok 이고 **1칸이 퇴행**이다: 브라우저 축의 J2ME 표본 `835acd535a5f` 가 30초 그리기 1244~1319회 → 879~882회(느려짐 중앙값 42% · 짝 재측 41.7%). 원인은 결함이 아니라 **옮긴 의미(#1491) 그 자체**다 — 이 판은 지금까지 제 프레임 시간보다 빨리 돌고 있었고, 이 PR 뒤에는 부하와 무관하게 34.1 ms 주기(29.3 fps)로 돈다(§4-B). 그래서 «고쳐서 그대로로 만들 수 있는 차이»가 아니다. 받아들일지(= sleep 으로 박자를 잡는 J2ME 판이 설계 속도로 느려진다)는 운영자가 정할 일이고, upstream 병합(S3)이 같은 의미를 가져오므로 그때도 같은 결정이 필요하다. KTF · LGT · SKT 표본과 무거운 장면 4판은 바뀌지 않았다.

**무엇을**: upstream 이 실행기에 넣은 두 의미를 `wie-backend` 의 `executor.rs` · `system.rs` · `task.rs` 에 옮겼다.
- **sleep 마감(upstream `49078c6b6` · #1491)**: `sleep(n)` 의 깨는 시각을 «부른 때 + n» 으로 센다. 그전에는 «그 step 이 시작한 때 + n» 이라, 자기 전에 한 일만큼 잠이 짧았다.
- **협동 폴링(upstream `d63d6022d` · #1429)**: 틱 안에서 다시 도는 것은 «다시 돌려 달라고 한» 태스크뿐이다. 모니터 `wait` 처럼 걸려 있는 태스크는 누가 깨우거나 다음 틱이 올 때 돈다. 그전에는 걸린 태스크가 하나라도 있으면 틱이 예산(14 ms) 끝까지 헛돌았다.

우리 pacing API(`tick_for` · `wants_cpu` · `next_due` · `halt` · `clear` · `sleep_toward` · `TICK_BUDGET_MS`)의 서명과 기본값은 그대로다. 스폰 순서(`BTreeMap`)도 그대로다. 더한 공개 함수는 `System::poll_until` 하나다(§3).

**왜**: 티켓 `wie-upstream-sync2-s2-backend-executor-sleep-deadline-cooperative-poll`(운영자 채택 제안 `2026-11-06-upstream-sync2-dry-run-classify#p1` · 계획 = 0551 §5·§7 의 S2 행). upstream 2차 병합 예행의 ours 탐침이 맨 처음 죽는 곳이 `wie-backend` 이고, 그 오류 4건 중 2건이 이 의미 차이다(§6).

**사용자 영향**: 게임 스레드의 `sleep(n)` 이 요청한 만큼 잔다(그전에는 자기 전에 한 일만큼 — 한 조각, 길어야 수 ms — 짧았다). 마감에 맞춰 자는 게임은 주기가 그대로이고(§3), 자기 전에 한 일을 빼고 자는 게임(`sleep(프레임 시간 − 걸린 시간)`)과 고정 길이로 자는 게임은 그만큼 느려져 제 박자가 된다 — 측정한 J2ME 표본 하나는 브라우저에서 42 fps → 29 fps 다(§4-B). 걸려 있는 스레드만 남은 판은 틱이 예산 끝까지 헛돌지 않는다. ★착지한다면 이용자에게 보이는 변화라 `docs/player-updates/` 항목이 같이 가야 한다 — 결정 전이라 이 PR 에는 넣지 않았다.

### 1. 옮긴 의미 — upstream 줄 ↔ 우리 줄

| upstream | 무엇 | 우리 줄(이 PR 뒤) |
|---|---|---|
| #1491 `executor.rs` — `last_now` 필드 삭제 · `step` 이 더는 적지 않음 | sleep 이 step 시작 시각에 기대지 않는다 | `executor.rs` `ExecutorInner` 에서 `last_now` 삭제 · `step` 의 대입 삭제 |
| #1491 `executor.rs` — `pub fn sleep(&self, until: Instant)` | 실행기는 «언제까지»만 받는다 | `Executor::sleep(until)`(같은 서명) · `Executor::sleep_toward(until, pace)` |
| #1491 `task.rs` — `SleepFuture::new(until: Instant, …)` | 같은 뜻 | `SleepFuture::new` 두 줄이 upstream 과 글자까지 같다 · `SleepFuture::toward(until, pace, …)` |
| #1491 `system.rs` — `SleepFuture::new(self.platform.now() + timeout, …)` | 마감을 부른 때의 시계로 만든다 | `System::sleep` → `sleep_from(now, timeout)` · `System::sleep_toward` · `System::guest_sleep`(시계를 한 번만 읽어 마감과 «늦음» 기준을 같게 둔다) |
| #1491 `wie-backend/tests/system.rs`(새 파일) | 회귀 시험 `sleep_waits_from_the_call_time_after_task_work` | upstream/main 의 파일을 그대로 실었다(바이트 동일). ★고치기 전 트리(`origin/main`)에서 이 시험은 red 가 아니라 **멈춘다** — §8 |
| #1491 `wie-backend/Cargo.toml` — dev 의존 `test-utils` | 위 시험용 | 이미 있었다(변경 0) |
| #1429 `executor.rs` — `TaskWake`(깃발 waker) | no-op waker 를 깃발로 | 0524 에서 이미 `Woken` 으로 들어와 있었다(`wants_cpu` 용) |
| #1429 `executor.rs` — `tick`: `step` 뒤 깃발이 안 섰으면 `break` | 깨워진 태스크가 없으면 그 틱은 더 돌지 않는다 | `tick_for` 의 `idle` — 아래 §2 ⑴ |
| #1429 `executor.rs` — `spawn` 이 깃발을 세운다 | 새 태스크는 스폰된 틱 안에서 돈다 | `Executor::spawn` 끝의 `self.woken.wake_by_ref()` |
| #1429 `executor.rs` — `sleep` 이 깃발을 세운다 | 0 ms sleep 이 같은 틱에서 이어진다 | 깃발은 세우지 않는다 — 아래 §2 ⑵. 같은 결과를 `idle` 경로의 «깰 때가 된 태스크가 있으면 step» 이 낸다 |
| #1429 시험 2개 · `YieldOnce` 가 waker 를 부름 | `pending_tasks_only_repeat_within_a_tick_when_woken` · `zero_duration_sleeps_and_newly_spawned_tasks_resume_in_the_same_tick` | 둘 다 그대로 옮겨 통과 · `YieldOnce` 도 같게 |
| #1429 `wie-ktf/…/class_instance.rs`(필드 읽기 최적화) | 실행기와 무관 | 옮기지 않았다 — 0551 §6 의 «조용한 병합» 목록에 있는 파일이고 S3 몫이다 |

### 2. 우리 pacing 을 지키려고 upstream 과 다르게 둔 두 곳

⑴ **깨워진 태스크가 없을 때 틱을 바로 끝내지 않는다.** upstream 은 `break` 한다. 우리는 «모두 잠든 틱»과 같은 길로 보낸다: 예산 안에 드는 paced 깨움이 있으면 그때까지 기다렸다가 step, 없으면 끝낸다. 걸린 태스크 옆에서 자는 게임 스레드의 `sleep(60)` 이 호스트 프레임 격자(66.7/83.3 ms)로 밀리지 않게 하는 우리 규칙(`test_tick_waits_for_a_wake_inside_its_budget`)이 걸린 태스크가 있을 때도 서야 하기 때문이다. 새 시험 `test_tick_waits_for_a_wake_inside_its_budget_beside_a_parked_task` 가 잠근다(걸린 태스크는 그 틱에 2번만 폴링된다 — 재우는 step 과 깨우는 step).

⑵ **`sleep` 은 깃발을 세우지 않는다.** 그 깃발은 `wants_cpu`(«예산을 다 쓰고도 더 달라는 게스트» → `starved_ticks` → `achieved_speed()`)가 읽는다. 잠드는 것은 CPU 를 달라는 것이 아니다. 세우면 모두 잠든 틱이 «굶었다»로 읽힌다.

그 밖에 자리만 옮긴 것: «1 ms 이하 sleep 은 폴링이라 틱을 붙잡지 않는다»(`POLL_SLEEP_MS`)는 판정이 `Executor` 에서 `System::sleep` 으로 갔다. 실행기가 더는 `timeout` 을 받지 않아서다. 값(1)과 뜻은 그대로다.

### 3. 측정 — `test_pacing`(전/후)

`wie-j2me/tests/test_pacing.rs` 의 픽스처(`pace_j2me` · 50 ms 마감 루프 · 읽을 때마다 1 ms 가는 시계). 워밍업 2 s 뒤 10 s 창의 `Pacing::summary_json`. «전» = `origin/main` `1853f3b6f` 를 따로 꺼낸 워크트리, «후» = 이 PR.

| 칸 | 전 | 후 | |
|---|---|---|---|
| `ticks` | 720 | 720 | 같음 |
| `paints` · `redraws` | 200 · 200 | 200 · 200 | 같음 — 프레임 주기 50.0 ms |
| `redraw_p50` · `p95` · `sum` | 4 · 4 · 800 | 4 · 4 · 800 | 같음 |
| `redraw_cross_tick` | 0 | 0 | 같음 |
| `yields` · `sleeps` | 400 · 200 | 400 · 200 | 같음 |
| `gcs` · `starved_ticks` | 10 · 0 | 10 · 0 | 같음 |
| `sleep_ms_p05` · `p50` · `p95` | 39 · 39 · 40 | 37 · 37 · 38 | **−2** |
| `sleep_late_p50` · `p95` · `sum` | 0 · 0 · 0 | 2 · 2 · 300 | **+2**(상한 5) |

틱 하나하나를 적어 본 것(스크래치 시험 · 커밋하지 않음 · 창 260 ms = 16틱)도 같다. 틱 길이의 분포(18 · 16 · 16 ms 가 프레임마다 반복 · 19 ms 1회)와 틱 끝 `due` 의 앞섬(34 · 18 · 2 ms · 35/19/3 각 1회)이 전/후 한 줄도 다르지 않다. 달라진 것은 무늬 전체가 2 ms 뒤로 간 것뿐이다(첫 틱 2000..2018 → 2002..2020).

바뀐 두 칸은 옮긴 의미 그 자체다. 이 게스트는 `sleep(다음 마감 − 지금)` 을 부른다. «전»은 그 잠을 step 시작 시각에서 세어 마감보다 시계 읽기 2번만큼 **일찍** 깼다 — 그래서 «늦음»이 0 으로 읽혔고, 일찍 깬 만큼 다음 잠이 길었다(39). «후»는 마감에 깬다. 읽기 한 번이 1 ms 인 이 시계에서 그 뒤 첫 읽기까지가 2 ms 다. 주기 · 그리기 수 · repaint→paint · 틱을 넘긴 그리기는 그대로다. 그래서 «박자는 그대로, 잠의 기준점만 옮겨졌다»로 판정했다. 시험 주석의 실측값 한 줄을 고쳤다(p95 `4 and 0` → `4 and 2`). 단언의 상한은 건드리지 않았다.

★**이 PR 의 첫 판은 이 시험이 red 였다 — S3 가 같은 자리를 밟는다.** upstream 의 꼴(`System::sleep` 이 부를 때마다 `platform.now()` 를 읽는다)을 그대로 옮기면 MIDP 이벤트 스레드의 1 ms 폴링이 한 바퀴에 시계를 두 번 읽는다(대기 조건 1 + sleep 1). 읽기가 곧 시간인 이 시계에서 그 판은 `redraw_p95` 4 → 9 · `redraw_cross_tick` 0 → 76/201 · `sleep_late_p95` 0 → 7 · `starved_ticks` 0 → 102 로 두 시험 모두 red 였다(`paints` 는 201 로 주기는 섰다). 그래서 대기 루프가 제 조건으로 읽은 시각을 폴링의 시작으로 그대로 쓰게 했다(`System::poll_until` · `wie-midp` `event_queue.rs`). 폴링당 읽기 1회 — 고치기 전과 같다. 실기기 시계에서는 읽기 한 번이 수십 ns 라 이 차이가 박자에 닿지 않는다. 이것은 자(읽기 수를 세는 시계)를 지키는 고침이다. S3 에서 `system.rs` 의 `sleep` 충돌을 upstream 쪽으로 풀면 이 red 가 다시 난다.

### 4. 측정 — `perf-watch compare` · 무거운 장면 표본

두 엔진의 release 빌드: A = `origin/main` `1853f3b6f`(`perf-watch.sh build`) · B = 이 PR 의 코드(작업 트리 스냅숏 `c30a645788a2` 를 같은 절차로 빌드 — 커밋 `cddf25a4b` 와 엔진 소스가 같다. 다른 것은 시험 파일 2개와 그 뒤 고친 주석뿐이다).

#### 4-A. 무거운 장면 표본 — 오늘 착지한 0559 · 0565 의 측정 판 4개

0559 의 LGT 판과 0565 의 KTF 3판. census 탐침 A 의 인자(`--inject --keep-timeout --timeout 30 --max-ticks 100000000000 --pacing 8`)로, 판마다 A 와 B 를 **같은 분에 동시에** 돌렸다(프로세스 2개 · `build-slot --long` 임대 1건 · 2회 반복). 16회 모두 PASS · 키 27/27 · `stop=deadline`. 칸은 «A · B».

| 판 | 회 | 그리기(30 s) | B/A | 타이머 발화(창 22 s) | 타이머 늦음 p50/p95 ms | 굶은 틱 | 틱 | load1 |
|---|---|---|---|---|---|---|---|---|
| `3cc7a9b4cb15` LGT | 1 | 354 · 361 | +2.0% | 237 · 239 | 0/19 · 0/18 | 653 · 629 | 48.5M · 47.9M | 79.5 |
| `3cc7a9b4cb15` LGT | 2 | 387 · 392 | +1.3% | 272 · 272 | 0/3 · 0/5 | 557 · 533 | 72.1M · 73.3M | 76.2 |
| `f80ca6d2cb48` KTF | 1 | 283 · 277 | −2.1% | 222 · 220 | 0/7 · 0/13 | 294 · 284 | 68.8M · 67.9M | 79.6 |
| `f80ca6d2cb48` KTF | 2 | 297 · 296 | −0.3% | 227 · 226 | 0/1 · 0/2 | 243 · 241 | 122.9M · 123.2M | 62.8 |
| `1fe6c1c897d7` KTF | 1 | 255 · 261 | +2.4% | 211 · 219 | 0/2 · 0/2 | 1160 · 1167 | 2.3M · 2.2M | 88.4 |
| `1fe6c1c897d7` KTF | 2 | 398 · 404 | +1.5% | 305 · 310 | 0/0 · 0/0 | 1116 · 1128 | 15.9M · 16.0M | 48.1 |
| `d9384b388ea5` KTF | 1 | 212 · 213 | +0.5% | 174 · 172 | 0/4 · 0/2 | 1126 · 1135 | 4.6M · 3.5M | 83.9 |
| `d9384b388ea5` KTF | 2 | 405 · 412 | +1.7% | 308 · 314 | 0/0 · 0/0 | 1151 · 1140 | 13.7M · 14.8M | 43.1 |

- 그리기 수의 B/A 는 중앙값 +1.4%(−2.1 ~ +2.4)다. 느려진 쌍도 빨라진 쌍도 있고 폭이 같다 — 프레임 박자가 바뀌었다는 신호가 없다. 같은 판의 1회 ↔ 2회 차이(부하 80 → 45 에서 255 → 398 까지)가 쌍 안의 차이보다 훨씬 크다. 쌍을 동시에 돌린 이유다.
- 타이머 늦음 p50 은 16회 모두 0 이다. p95 는 부하를 따라 양쪽으로 움직인다(0/19 ↔ 0/18 · 0/7 ↔ 0/13).
- ★이 네 판은 모두 WIPI 타이머로 박자를 잡는다(창 안의 게스트 `Thread.sleep` 0회). 그래서 이 표가 본 것은 «타이머 · 이벤트 대기 경로가 그대로다»이고, sleep 마감 쪽은 §3 의 픽스처와 아래 `perf-watch` 의 J2ME 표본이 본다.

#### 4-B. `perf-watch compare`

`bash scripts/perf-watch.sh compare <A> <B> <out>` — 13:44~14:17 KST · load1 16~134 · 축마다 3회 + 경보 표본 짝 재측 3회. 도구의 판정 줄: **퇴행 1 · 1차 경보 1 · 미측정 0/15 · noisy 0**. 느려짐 % 는 «B 가 A 보다 얼마나 느린가»다.

| 축 | 표본 | 회별 느려짐 % | 중앙값 | 문턱 | 판정 |
|---|---|---|---|---|---|
| browser | `835acd535a5f` j2me | 47 · 42.1 · 41.8 | **42.1** | 5 | **alarm → 짝 재측 41.2 · 49.5 · 41.7 = 퇴행** |
| browser | `bf5b031018aa` j2me | 1.8 · 0.9 · 1.6 | 1.6 | 5 | ok |
| browser | `976141a9525b` skt | 1 · 0.2 · −0.3 | 0.2 | 5 | ok |
| browser | `d1234e0ddb75` skt | −0.2 · 0 · −0.3 | −0.2 | 5 | ok |
| browser | `1065985081fc` ktf | 0 · 0 · 0 | 0 | 5 | ok |
| browser | `155972cac664` ktf | 0.2 · 1 · −1.4 | 0.2 | 5 | ok |
| browser | `0606d43702e7` lgt | −0.6 · −0.6 · −0.2 | −0.6 | 5 | ok |
| browser | `3cc7a9b4cb15` lgt | 0 · −24.2 · 0 | 0 | 22.9 | ok |
| native | `835acd535a5f` j2me | −52.9 · −75.3 · −73 | −73 | 20.1 | ok |
| native | `8238f5449347` j2me | 1.1 · 0.5 · 0.1 | 0.5 | 5 | ok |
| native | `d1234e0ddb75` skt | −59.4 · −1 · −0.3 | −1 | 55.4 | ok |
| native | `1065985081fc` ktf | 0.9 · −0.6 · 0.3 | 0.3 | 5 | ok |
| native | `155972cac664` ktf | −0.1 · −16.2 · −1.5 | −1.5 | 14.6 | ok |
| native | `0606d43702e7` lgt | 2.3 · 0.8 · −0.9 | 0.8 | 5 | ok |
| native | `3cc7a9b4cb15` lgt | −0.7 · −9 · −1.3 | −1.3 | 7.6 | ok |

**퇴행 1건 — `835acd535a5f`(재배포 가능한 OSS J2ME 표본) 의 브라우저 그리기 수(30 s):**

| 회 | 1 | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|---|
| A | 1292 | 1249 | 1246 | 1244 | 1319 | 1247 |
| B | 879 | 879 | 879 | 881 | 882 | 880 |
| load1 | 68 | 88.9 | 46.3 | 25.1 | 21.5 | 24.9 |

- **B 는 부하(21 ~ 89)와 무관하게 879 ~ 882 회다 — 주기 34.1 ms · 29.3 fps.** A 는 1244 ~ 1319 회(22.7 ~ 24.1 ms · 42 ~ 44 fps)로 흔들린다. 부하에 안 흔들리는 수는 호스트가 아니라 게스트 자신의 시간표에서 나온다.
- native 축(같은 판 · 60 s · 키 없음)의 그리기 속도도 같은 쪽이다: 35.1 → 29.1회/초(B 주기 34.4 ms) · 6.0 → 4.9 · 26.0 → 21.8. 그 축의 «−73%»는 느려짐이 아니라 틱 수가 는 것이다(걸린 태스크가 있는 틱이 헛돌지 않는다 — 1회차에 A 는 60초에 3,300만 틱, B 는 40.7초에 상한 5,000만 틱에 닿았다 · 값은 «틱당 CPU»다).
- **이 판의 루프 꼴**(native · 두 엔진을 같은 분에 동시에 · 30 s · 창 22 s 의 pacing 계수): 게스트가 요청한 sleep 이 A 6 / 24 / 28 ms · B 5 / 23 / 28 ms(p05 / p50 / p95)다. 한 값이 아니라 퍼져 있다 — `sleep(프레임 시간 − 걸린 시간)` 꼴이다. 이런 루프는 sleep 이 요청한 만큼이면 주기가 프레임 시간에 선다(B 의 34.1 ms 고정이 그것이다 — 프레임 시간 ≈ 33 ms 로 읽힌다). **A 의 23 ms 주기는 그 프레임 시간보다 짧다. 걸린 시간을 빼고 자는 루프가 제 프레임 시간보다 빨리 도는 길은 sleep 이 일찍 끝나는 것뿐이다.** 창 안 sleep 수는 A 624 · B 505, 굶은 틱은 158 → 6 이다.
- **옮긴 두 의미 중 어느 쪽인가**: 주기가 느는 쪽은 #1491 이다. 종전 식은 깨는 시각을 «그 step 이 시작한 때 + n» 으로 셌다. 게임 스레드가 한 step 안에서 프레임 일을 하고 나서 `sleep` 을 부르면, 그 일에 걸린 시간이 잠에서 빠졌다 — upstream 의 회귀 시험이 재는 바로 그 꼴이다(일 35 ms 뒤의 `sleep(115)` 는 1150 에 깨야 한다 — 종전 식으로는 1000 + 115 = 1115 다). #1429 쪽(걸린 태스크를 다시 돌리지 않음)은 깨는 시각을 옮기지 않는다. ★두 의미를 따로 넣은 변형 빌드로 가르지는 않았다 — 위는 식과 측정값에서 나온 설명이다.
- 다른 J2ME 표본 `bf5b031018aa` 는 1.6%(0.9 ~ 1.8) 느리다. 문턱 안이지만 세 번 다 같은 부호다 — 같은 원인의 작은 몫으로 읽는다.
- KTF · LGT · SKT 6표본은 −0.6 ~ +0.2%(중앙값)다. `3cc7a9b4cb15` 의 −24.2 한 번은 2회차에 A 300 · B 396 으로 갈린 것이다(1회차 404 · 404 · 3회차 301 · 301 — 판이 두 속도 사이를 오가고 그 회에만 쪽이 엇갈렸다).

**이것을 어떻게 읽을 것인가(결정할 것)**

| 선택 | 결과 |
|---|---|
| ⒜ 받아들인다(upstream 과 같은 의미) | sleep 으로 박자를 잡는 J2ME 판이 설계 속도로 돈다. 표본 1건은 42 → 29 fps. 이용자에게는 «느려졌다»로 보일 수 있다 — `player-updates` 항목과, 느려지는 판이 몇 개인지의 census(이 회차는 표본 2개만 봤다)가 같이 가야 한다 |
| ⒝ 받지 않는다(#1491 은 빼고 #1429 만) | 속도는 그대로다. 예행 탐침의 `task.rs` 오류는 남고, S3 에서 `system.rs`·`task.rs`·`executor.rs` 를 우리 쪽으로 풀어야 한다. upstream 의 `tests/system.rs` 는 우리 트리에서 red 라 뺀다 |
| ⒞ S3(병합) 때 같이 정한다 | 이 PR 은 열어 둔다. 병합이 같은 결정을 다시 요구한다 |

이 회차의 권고는 ⒜ 다 — 틀린 쪽은 종전 동작이고(요청한 만큼 자지 않았다), upstream 과 갈라진 채로 두면 병합 때마다 이 자리를 다시 푼다. 다만 «얼마나 많은 판이 얼마나 느려지는가»를 모른 채 착지시킬 일은 아니다.

### 5. 게이트

모두 이 PR 의 코드 트리에서 돌렸다(`build-slot` 경유 · 2026-11-22 13:03~13:11 KST · load1 126~208).

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | rc=0 |
| `cargo clippy --all -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -p wie-core-arm -p wie_featurephone -- -D warnings` | rc=0 |
| `cargo +beta clippy --all -- -D warnings` | rc=0 · `1.101.0-beta.2`(`rustup check`: up to date) |
| `RUST_MIN_STACK=4194304 cargo test --all --no-fail-fast` | rc=0 · 시험 바이너리 55 · **869 passed · 0 failed** |
| 그중 `wie-backend` | 139 passed + `tests/system.rs` 1 passed |

러너 블록(문서대로 debug · load1 173):

| 픽스처 | 결과 |
|---|---|
| `draw_j2me.jar` | PASS · rc=0 · paints 1 |
| `helloworld_ktf.zip` · `helloworld_lgt.zip` | PASS · rc=0(둘 다 `clean exit`) |
| `keydraw_ktf.zip` `--inject --expect-last-frame` | PASS · rc=0 · `content true` · paints 79 · 27/27 |
| `keydraw_lgt.zip` `--inject --expect-last-frame` | PASS · rc=0 · `content true` · paints 55 · 27/27 |
| `text_j2me.jar` `--timeout 5` | PASS · rc=0 · paints 1 |

위 표는 첫 커밋(`cddf25a4b`) 앞의 실행이다. 그 커밋 직전(13:29~13:30)과 이 문서를 마지막으로 고친 뒤(14:39~14:42)에 같은 묶음(fmt · clippy 4종 · `cargo test --all --no-fail-fast`)을 한 번씩 더 돌렸고 둘 다 전부 rc=0 · 869 passed 다. 그 뒤 연번을 한 번 더 옮기고(0571 → 0573 · #604 가 0571 을 먼저 잡았다) 같은 묶음을 다시 돌렸다(14:48~14:50 · 전부 rc=0 · 869 passed). 마지막 실행 뒤에 바뀐 것은 이 문단의 그 시각과 끝의 유입 표식 한 줄뿐이다.

### 6. upstream 병합 예행 탐침 — 전/후

0551 §5 의 P1(ours) 탐침을 오늘 트리로 다시 만들었다: `git merge --no-commit --no-ff upstream/main`(`24b670780` · behind 247) → UU·AA 는 `--ours` · UD 는 우리 판 · 매니페스트 4개(`Cargo.toml` · `wie-backend` · `wie-j2me` · `wie-skt`)는 양쪽 합집합 · 루트 `workspace.dependencies` 에 `md5` 복원. 그 뒤 `cargo check -p wie-backend --all-targets`.

| | 전(`origin/main` `1853f3b6f`) | 후(이 PR) |
|---|---|---|
| 충돌 경로 | 137(UU 106 · AA 27 · UD 4) | 139(UU 108 · AA 27 · UD 4) |
| `wie-backend` 오류 | **4** — `system.rs:31` E0603 · `task.rs:44` E0308 · `lib.rs:99` E0277 · `lib.rs:102` E0599 | **2** — `lib.rs:99` E0277 · `lib.rs:102` E0599 |

- 충돌이 2 는 것은 `task.rs` 와 `system.rs` 가 «조용히 (틀리게) 합쳐지는 파일»에서 «보이는 충돌»로 옮겨 온 것이다. 둘 다 덩어리 1개이고 답은 우리 쪽이다(`git merge-file` 로 따로 확인: `task.rs` 1 · `system.rs` 1 · `executor.rs` 10 — 전에도 10).
- ★`system.rs` 의 그 덩어리 **밖**에서는 upstream 의 두 줄이 여전히 조용히 들어온다: `pub use … normalize_guest_path` 와 `FilesystemOverlay::new(…, pid)`(aid → pid). 탐침은 파일을 통째로 `--ours` 로 풀어 못 본다. 0551 §8 의 «파일시스템 이름공간» 사람 결정이 그 줄이다 — S3 몫이다.
- `wie-midp` `event_queue.rs` 는 이 PR 뒤에도 충돌 없이 합쳐지고(같은 `merge-file` 확인 rc=0) 대기 루프가 그대로 남는다.
- **남은 2건은 zip 8 → 9 다**(`file.name()` 이 `Result` 를 돌려준다). 실행기와 무관하고, S1 레인(`wie-4` · 브랜치 `wie-upstream-sync2-s1-rustjava-pin-0-4-1` · 미푸시 WIP `4fbd7657d`)이 `wie-backend/Cargo.toml` 과 `lib.rs` 에서 이미 고치고 있다(읽기로 확인). 그래서 이 PR 에 넣지 않았다. 0551 §7 이 S2 에 적은 기대 «탐침 P1 에서 `wie-backend` 오류 0»은 S2 만으로는 서지 않고 S1 이 같이 착지해야 선다.
- 다음 벽: 스크래치 탐침에만 zip 고침(`lib.rs` 의 `file.name()` 두 곳 — S1 WIP 과 같은 꼴)을 얹고 `cargo check --workspace --all-targets --keep-going` 을 돌렸다. **`wie-backend` 오류 0 — 탐침이 `wie-backend` 를 지난다.** 다음에 죽는 곳은 셋이다(오류 38 · 그 아래 크레이트는 이 셋에 막혀 컴파일까지 못 간다).
  - `wie-jvm-support` 27 — RustJava 0.4.1 API(`name()` · `descriptor()` 가 `Cow<str>` · 인자 수). S1 축이다.
  - `wie-core-arm` 7 — `binary_patches/{parser,patch}.rs` 없음 · `md5` · AOT 엔진 트레이트(`record_image` 외) · `instructions_executed`. 0551 §7 S3 ⑴.
  - `wie-wipi-c` 4 — upstream 쪽 코드가 부르는 `decode_image_framebuffer` · `DatabaseRepository::usage` · `FilesystemOverlay::{remove,usage}` 가 `--ours` 로 푼 backend 파일에 없다. S3 의 3-way 몫이다.

### 7. S1 · 열린 PR 과의 겹침

- **S1**(RustJava 핀 · 위 WIP `4fbd7657d` · PR 아직 없음): 이 PR 과 같은 파일은 `wie-midp/src/classes/net/wie/event_queue.rs` 하나다. `git merge-tree --write-tree` rc=0(충돌 0). `wie-backend` 에서 S1 이 만지는 것은 `Cargo.toml` · `lib.rs` 이고 이 PR 은 `executor.rs` · `system.rs` · `task.rs` · `speed_clock.rs` · `tests/system.rs` 다. 착지 순서 제약 없음 — 단 S1 이 WIP 라 PR 이 열리면 다시 재야 한다.
- **PR #589**(`wake_tick` · 11-11 부터 열림): `wie-backend/src/executor.rs` 에서 충돌한다(`merge-tree` rc=1 · 그 파일 하나). #589 는 `tick_for` 에 `max_wait` 인자를 더하고 paced 판정 한 줄을 고친다. 나중에 착지하는 쪽이 그 한 줄과 `tick_for` 서명, 시험 도우미(`sleeper`)를 맞춰야 한다. `system.rs` · `speed_clock.rs` 는 깨끗이 합쳐진다.
- PR #599 · #601: `merge-tree` rc=0.

### 8. 한계

- ★**upstream 의 `tests/system.rs` 는 고치기 전 실행기에서 끝나지 않는다.** `origin/main` 워크트리에 그 파일만 넣고 돌렸더니 52분 동안 돌고 있어 손으로 끊었다(단언 실패가 아니다). 종전 실행기는 태스크가 하나도 없으면 예산이 다할 때까지 step 을 돌리는데, 이 시험의 시계는 멈춰 있어 예산이 다하지 않는다. 식으로 따지면 1115 에서 일찍 깬 태스크가 끝난 뒤의 틱이다(어느 틱에서 멈췄는지 찍어 보지는 않았다). 이 PR 의 `idle` 이 그 틱을 끝낸다. ⒝(#1491 을 빼는 선택)를 고르면 이 시험은 같이 빼야 한다. ★그 확인 실행은 시간 제한 없이 돌린 내 실수로 short 빌드 슬롯 한 칸을 54분 쥐고 있었다.
- **B 에서 «요청 다음 틱에 시작한 그리기»가 늘었다** — 위 판의 native 계수로 368/513 → 500/500. 프레임 주기는 프레임 시간에 서 있으므로 박자를 늦추는 것은 아니다. 다만 종전에 같은 틱 안에서 그려지던 28% 가 어느 경로였는지(걸린 태스크가 틱을 붙잡던 동안의 전달인지)는 가르지 않았다. ⒜ 로 가기 전에 브라우저에서 한 번 볼 자리다.
- 느려지는 판이 몇 개인지 모른다. 이 회차가 본 sleep 박자 판은 표본 2개(42% · 1.6%)뿐이다.

- 걸린 태스크만 있는 틱이 이제 바로 끝난다. `wie_validate` 처럼 틱을 쉬지 않고 부르는 호스트에서는 그런 판의 틱 수가 빨리 는다(잠만 자는 판이 이미 그렇듯). 기본 `--max-ticks`(5천만)로 키를 넣는 실행은 대본이 끝나기 전에 그 상한에 닿을 수 있다 — census 는 이미 상한을 풀고 돌린다(`NO_TICK_CAP`). 러너 블록의 `keydraw_*` 는 1,040만 · 1,657만 틱에 `deadline` 으로 끝났다(27/27).
- `Executor::sleep` 의 깃발(§2 ⑵)과 예산 안 기다림(§2 ⑴)은 upstream 과 다르다. S3 에서 `executor.rs` 는 우리 쪽으로 풀고 upstream 시험 2개가 남아 있는지 본다.
- 브라우저에서 «걸린 태스크가 있는 판의 틱이 실제로 짧아졌는지»(발열)는 따로 재지 않았다. PR #589 의 `tick_occupancy` 가 그 자다.

<!-- corpus-name-inflow v1 subjects=9 tree=b171dfa00a3f8400 B=6/5 P=0/0 S=0/0 -->
