## [2026-10-22] featurephone: «실제로 낸 배속» 조회 API (wie-featurephone-achieved-speed-api)

### 무엇을

- `WieEmulator::achieved_speed() -> f64`(wasm export 1개 · 계약 `methods` + `achievedSpeedNote`). 최근 2초 틱 창의 «실제로 낸 배속».
- 정의(`wie_backend::AchievedSpeed` 문서 주석): 게스트 시간 ÷ 벽시계. 단 시계는 **게스트가 굶는 동안 멈춘다**.
  굶음 = 틱이 예산을 다 썼는데 게스트가 CPU 를 더 원했던 경우이고, 그 틱이 끝난 때부터 다음 틱이 시작할 때까지가 굶는 구간이다.
  틱 자체는 언제나 센다. 모든 태스크가 잔 틱 뒤의 틈은 «제때»로 다 센다.
- ★`SpeedClock` 을 그대로 재면 안 되는 이유: 게스트 시계는 벽시계 × `speed` 라서 CPU 가 모자라도 그 비가 **정의상 항상 `speed`** 다.
  티켓 문안의 «게스트가 본 가상 시간 ÷ 벽시계»는 이 시계로는 아무것도 재지 못한다. 그래서 «굶는 동안 멈추는» 시계로 정의했다.
- 굶음 판별 = `Executor::wants_cpu()`. 지금까지 no-op 이던 waker 를 `alloc::task::Wake` 깃발로 바꿨다(no-op vtable 삭제).
  `YieldFuture`(ARM 코어가 `INSTRUCTIONS_PER_YIELD` 마다, JVM 인터프리터가 조각 사이에 기다린다)는 `wake_by_ref` 를 부른다.
  JVM 모니터 `wait`·sleep 은 부르지 않는다.
  ★«틱 끝에 잠들지 않은 태스크가 있다»로 판별하면 틀린다: `Object.wait()`(무기한)·`Thread.join` 으로 걸린 태스크도 executor 에는
  «안 자는» 태스크라 매 틱 예산까지 돈다. 그러면 따라가는 게임도 굶는다고 읽힌다(시험 `a_guest_parked_on_a_monitor_is_not_starved`).
- 전달 경로: `System::tick_for` → `Pacing::tick_ended(starved)` → `Pacing::starved_ticks`(창 카운터 · `summary_json` 끝에
  `"starved_ticks"`) → featurephone 이 매 틱 `take_pacing()` 으로 읽는다. `Emulator` 트레이트 변경 0.
  featurephone 에서는 그전까지 아무도 pacing 을 읽지 않았다 — 매 틱 take 하므로 그 표본 Vec(상한 4×65,536)도 더는 쌓이지 않는다.
- `set_speed` 가 창을 비운다. 비운 뒤(생성 직후 포함) 틱 2회 전까지는 `speed()` 를 돌려준다.

### 한계 (ponytail 주석)

- 굶은 게스트는 «얼마나 더 원했든» `speed × 예산 ÷ 프레임` 아래로 내려가지 않는다(60Hz ≈ 0.84 × speed).
  executor 는 «더 원했다»만 보고 «얼마나»는 못 본다. 깊은 측정은 여전히 0498 의 그리기 수 비교(`audio-probe.mjs --paint-window`)다.
  예: 0498 의 `287af341` 2배 실측 1.54~1.56 을 이 API 는 ≈1.68~1.72 로 읽을 것이다(브라우저에서 재지 않았다 — 아래).
- `Thread.yield` 스핀으로 기다리는 게스트는 굶는다고 읽힌다 — CPU 를 실제로 다 쓰므로 틀린 말은 아니다.

### 검증

- 단위 시험(`speed_clock.rs` · 60Hz 모의 호스트 · 14 ms 예산 · 3 s):
  - 여유 → `speed` 와 정확히 같음(1배·2배).
  - 모니터에 걸린 태스크만 있음 → 2.0.
  - 예산 포화(`YieldFuture` 무한) → 1.72(< 2).
  - 배속 변경 직후 창 초기화: reset 직후 새 speed 그대로, 굶은 틱이 창에서 빠짐.
  - 창이 2초인지.
- 4게이트: fmt · clippy stable/wasm32(+`-p wie_featurephone`)/beta `-D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` 789 pass / 0 fail.
- 러너 블록 전건 PASS(draw_j2me · helloworld ×2 · keydraw ×2 `--inject --expect-last-frame` rc=0 · text_j2me `--timeout 5`) · load1 56.
- `wie_web.js`: 긴 문서 주석을 단 첫 빌드가 **40,452 B**(상한 40,000 초과)였다 — wasm-bindgen 이 export 의 문서 주석을 JS 로 복사한다.
  export 주석을 한 줄로 줄이고 정의는 `AchievedSpeed` 에 뒀다. 상한은 올리지 않았다 → **39,859 B** · `check-engine-contract.mjs` 138 pass · 0 violation.
- 브라우저에서 실게임 수치는 재지 않았다(게임 파일 필요 · 셸 표시 티켓 `otterpebble-featurephone-achieved-speed-indicator` 가 화면에서 본다).

### 게이트② 반려 승계 (wie-featurephone-achieved-speed-api-fix)

- **틱 사이 공백 > 0.5 s 는 창에 넣지 않고 창을 비운다**(`HOST_PAUSE_MS`). 셸은 `visibilitychange` 에서 `pause()` 하지 않으므로
  rAF 만 멈춘 백그라운드 탭·앱 전환·화면 잠금 뒤 첫 틱이 30 s 공백을 «간격 하나»로 넣었다 — 직전 틱이 굶었으면 약 2 s 동안 ≈0배,
  안 굶었으면 약 2 s 동안 굶음을 가려 ≈speed. 0.5 s = 2 fps 아래면 «페이싱»이 아니라 «호스트가 안 부름»으로 본다.
- 시험 `a_host_pause_starts_a_new_window`: 30 s 공백 뒤 첫 값 = speed, 0.5 s 뒤 = 기기 값(2×14/16.7) — 직전 틱 굶음/안 굶음 두 갈래.
  개악 대조(가드를 `if false && …` 로 끔): `last_starved=true` 갈래 첫 단언 red, `false` 갈래만 돌리면 1.9947 ≠ 1.6766 red.
- 계약 `achievedSpeedNote` 에 그 한 문장을 더했다(export 표면 무변경). 하한 ≈0.84×speed 고지는 그대로다(확인만).
- ★0348 계열 측정에 주의: 이제 featurephone 이 **매 틱 `take_pacing()` 으로 Pacing 창을 비운다**. 0348 §1 하네스처럼 스크래치 export
  `take_pacing` 으로 15 s 창을 재면 1틱짜리 창만 보인다 — `achieved_speed()` 를 읽거나 `lib.rs` 의 그 줄을 빼고 재라.
  (0348 본문에 덧붙이지 않은 까닭: 그 회차의 유입 표식이 STALE 로 바뀐다 — 착지한 회차 표식은 고치지 않는다.)

### 사용자 영향

- 지금은 없다(엔진 API만). 셸이 «고른 배속 vs 실제 배속»을 보여 줄 수 있게 된다. 1배 동작 변경 0 — waker 는 깃발만 세우고 폴링 순서·횟수는 그대로다.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: 유입 7건(BOUNDED · 10회) + 판단 필요 0건(SUFFIX-ATTACHED). 전부 이 회차가 손댄 파일
(`executor.rs` · `system.rs` · `wie_featurephone/src/lib.rs`)에 **원래 있던** 주석이다. 이 회차가 더한 줄에 든 것은 0이다.

<!-- corpus-name-inflow v1 subjects=9 tree=a66194a2662731ee B=10/7 P=1/1 S=0/0 -->
