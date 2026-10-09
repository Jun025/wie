## [2026-10-10] WIPI 타이머 «틱당 1회» 상한을 배속에 비례 (wie-wipi-timer-per-tick-cap-scale-with-speed)

### 무엇을

- `MC_knlSetTimer`(wie-wipi-c `kernel.rs`)가 다시 걸린 타이머를 «이번 틱에서 이미 발화했으면 다음 호스트 프레임으로» 미루던 규칙을
  «이번 틱에서 **몫만큼** 발화했으면»으로 바꿨다. 몫 = `wie_backend::timer_fires_per_tick(speed, tick)`
  = `floor(S·(t+1)) − floor(S·t)` — 1배 1 · 2배 2 · 1.5배 1·2 번갈아(60틱에 90) · 1.25배 60틱에 75.
  틱마다 독립이다: 덜 쓴 틱이 다음 틱에 몫을 넘기지 않는다(상한이지 목표치가 아니므로 쌓아 두면 한 틱에 몰아 쏜다).
- `EventQueue` 의 타이머별 기록이 «마지막 발화 틱»에서 «(마지막 발화 틱, 그 틱 발화 횟수)»로 — `timer_fired_in → timer_fires_in(…) -> u32`.
- 배속을 엔진까지 내린 길: `Platform::speed()`(기본 구현 1.0). `WebPlatform`·`HeadlessPlatform` 은 이미 쥔 `SpeedClock` 의 `speed()` 를 돌려준다.
  ★티켓 문안은 «System 필드 1개»였으나, 그러면 호스트가 시계와 System 양쪽에 배속을 써야 해 진실원이 둘이 된다. `now()` 를 이미 플랫폼이
  답하므로 같은 자리에서 배속도 답하게 했다 — System 필드 0 · wasm export 0.
- `scripts/audio-probe.mjs`: `--speed x`(반복 가능 · 빌드×게임×배속 실행) · `--paint-window a,b`(게스트 a~b 초 = 벽 a/x~b/x 초의 그리기 수 ·
  실행을 b/x 초에 끝냄). 0494 의 «게스트 시간 창 맞춤» 방법을 브라우저 프레임 위에서 그대로 쓴다.

### 1배 무회귀 — 구조로

1배에서 몫은 모든 `t` 에 대해 정확히 1이다(`S·t` 가 정수라 부동소수 오차 없음 · `u32::MAX × 7` 틱에서도 시험). 그때
`timer_fires_in(…) >= 1` ⇔ 종전 `timer_fired_in(…)` 이라 `pace_from` 이 바이트 단위로 같다. 실측도 같다(아래 표 1배 열 · 소음 범위).
러너 블록 전건 PASS(keydraw 둘 rc=0 · text_j2me `--timeout 5` PASS) · `cargo test --all` 771 pass.

### 실측 — 브라우저 60Hz 경로 (headless Chromium · rAF 당 `emu.tick()` 1회)

방법: `node scripts/audio-probe.mjs --wasm <전> --wasm <후> --speed 1 --speed 1.5 --speed 2 --paint-window 12,24 --key-ms 0 --jobs 3`.
전 = `4bf649ff`(origin/main) · 후 = 이 브랜치. 지표 = 게스트 [12 s, 24 s] 창의 `putImageData` 수. 달성 배속 = S × 창(S) / 창(1배, 같은 빌드).
틱 수 ≈ 59.6/s(1배 24 s 에 1,427~1,434) — 60Hz rAF 가 실제로 걸렸다. load1 31~67(10코어 · 다른 레인 스윕 동시). 제목은 sha256 앞 8자.
배속 범위는 운영자 지시(2026-10-10 01:48 «1~2배»)에 맞춰 [1, 2].

| 플랫폼 | 표본 | 빌드 | 1배 | 1.5배 | 2배 | 달성 1.5배 | 달성 2배 |
|---|---|---|---|---|---|---|---|
| LGT | `287af341` | 전 | 719 · 718 | 480 · 480 | 360 · 360 | 1.00 · 1.00 | 1.00 · 1.00 |
| LGT | `287af341` | **후** | 718 · 720 | 593 · 600 | 552 · 562 | **1.24 · 1.25** | **1.54 · 1.56** |
| LGT | `863b8ab6` | 전 | 450 | 361 | 316 | 1.20 | 1.40 |
| LGT | `863b8ab6` | 후 | 467 | 365 | 304 | 1.17 | 1.30 |
| SKT | `38277d63` | 전 | 721 | 480 | 360 | 1.00 | 1.00 |
| SKT | `38277d63` | 후 | 720 | 480 | 360 | 1.00 | 1.00 |
| SKT | `abc0c578` | 전 | 1249 | 925 | 701 | 1.11 | 1.12 |
| SKT | `abc0c578` | 후 | 1266 | 939 | 715 | 1.11 | 1.13 |

(`287af341` 은 같은 조건 2회 — 두 수를 다 적었다.)

- **`287af341` 이 이 상한에 묶여 있던 표본이다.** 전 빌드는 배속과 무관하게 «틱당 그리기 1회»(1,411/1,428 · 698/708)에 정확히 붙어
  달성 1.00 — 배속 슬라이더가 이 게임에는 아무것도 안 했다. 후 빌드는 2배에서 틱당 1.5회(1,082/709)까지 올라 **1.54~1.56배**.
  2.0 에 못 미치는 것은 상한이 아니라 틱 안의 일감이다: 후 2배의 tick p99 16~17 ms 로 프레임 예산을 다 쓴다(전 2배는 10 ms — 일이 없어 쉬었다).
- `863b8ab6` 은 1배에서 이미 틱당 0.31회 그린다(450/1,429) — 타이머 상한에 닿지 않는 게임이라 바뀌지 않는다(차이는 소음 범위 · 같은 부하에서 1회).
  0494 의 «LGT 1.2~1.7배» 중 이 표본의 원인은 이 상한이 아니다 — 그쪽은 worklog `2026-10-09-play-speed-scale-api#p1` 소관 그대로.
- SKT 두 표본은 WIPI-C 타이머로 돌지 않는다(SKVM) — 이 변경과 무관하고, 수도 전·후 같다. 0494 의 «SKT 가 배속을 거의 못 먹는다»는 이 회차로 풀리지 않는다.
- ★`wie_validate` 표는 만들지 않았다: 그 루프는 틱 사이에 쉬지 않아(`--frame-hz` 도움말 «this loop never sleeps») 미뤄진 타이머가
  곧바로 다음 틱에 발화한다 — 이 상한이 거기서는 구조적으로 안 보인다(0494 가 같은 이유로 후속 제안을 남겼다). 비교는 위 브라우저 표가 대신한다.

### 검증

- 단위 시험: `speed_clock.rs` `timer_fires_per_tick_is_speed_on_average_and_one_at_1x`(1배 전부 1 · 2배 전부 2 · 1.5배 1/2 만 · 60틱 합 90 ·
  1.25배 75 · 큰 틱 · NaN→1) · `kernel.rs` `test_a_timer_may_fire_speed_times_per_tick`(2배: 첫 발화 뒤 `pace_from` 0 · 둘째 뒤 `tick+1` · 다음 틱 0).
  기존 `test_when_a_timer_may_keep_a_tick_alive`(1배) 그대로 통과.
- 되돌리기 돌연변이: `>= share` → `>= 1` — `test_a_timer_may_fire_speed_times_per_tick` **CAUGHT**(1 failed).
- 4게이트(fmt · clippy `-D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all` 771 pass) + beta clippy rc0.
- `wie_web.js` 39,598 → **39,602 B**(상한 40,000 · +4 B — 새 export 없음).

### 사용자 영향

1배는 그대로다. 배속을 올리면, 1배에서 이미 화면 주사율만큼 타이머를 돌리던 LGT 게임(표본 `287af341`)이 처음으로 실제로 빨라진다
(2배 설정 ≈ 1.55배). 그 밖의 표본은 바뀌지 않는다 — 다른 원인에 묶여 있다.

게임 파일명 유입: 유입 5건(BOUNDED · 전건 이번에 고친 파일 4개의 «기존» 주석 — 이 회차가 쓴 줄에는 0) + 판단 필요 0건(SUFFIX-ATTACHED).

<!-- corpus-name-inflow v1 subjects=13 tree=369b0d1a0053ee03 B=6/5 P=0/0 S=0/0 -->
