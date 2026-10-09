## [2026-10-09] LGT Clet 타이머 콜백의 쓰레기 수거 호출을 시험으로 잠근다 (wie-2026-10-09-lgt-0266ca417880-timer-gc-call-test-lock-adopt-p0)

**무엇을** — `wie-lgt/src/runtime/wipi_c/context.rs` 에 시험 `a_timer_callback_collects_garbage` 1개. 실제 `LgtWIPICContext`(`init_jvm` 으로 만든 JVM·ARM 코어)에서 `set_timer` 를 부르고, 큐에서 그 타이머 이벤트를 꺼내 실행한 뒤 그 에뮬레이터의 `GcClock` 이 수거 1회를 기록했는지 단언한다. 제품 코드 변경 0.

**왜** — 0490 §1·§5: `set_timer` 의 이벤트 안 `collect_garbage_if_due` 호출만 빼면 0266ca417880 이 404.7초에 힙 고갈로 죽는데 `cargo test --all` 은 green 이었다 — 타이머를 거는 LGT 픽스처가 없다. 채택 제안 `2026-10-09-lgt-0266ca417880-heap-exhaustion#p0` 의 싼 경로(단언)다.

**개악 대조** — 그 호출을 `Ok(())` 로 바꾼 판: `cargo test -q -p wie-lgt a_timer_callback` → rc=101 · `panicked … the timer callback did not collect`. 원판: rc=0 · 1 passed.

**시험 설계에서 잰 것 두 가지** — ⑴ `TestPlatform::new()` 의 기본 시계는 0 근처에서 시작해 첫 수거가 아직 «때가 아니다»(`GC_INTERVAL_MS` 1000) ⇒ 원판에서도 red 였다. ⑵ `TestClock` 을 50,000 에 «멈춰» 두면 시험이 끝나지 않았다(16분 대기 후 중단). 그래서 `TestClock::stepping(1)` 을 50,000 에서 시작한다.

**한계** — 호출 자리만 잠근다. 수거 주기는 기존 `timer_collection_waits_a_second_and_for_its_cost` 가, 실제 타이틀의 장주행은 census 가 본다(픽스처 경로 M 은 하지 않았다).

**사용자 영향** — 없음(시험만). 7분 힙 고갈의 재발을 PR 단계에서 막는다.
