## [2026-10-09] 게임 배속 1.0~3.0배 연속 설정 — 엔진 API (wie-featurephone-play-speed-scale-api)

### 무엇을

- `wie_featurephone::WieEmulator::set_speed(speed: f64) -> f64` · `speed() -> f64` (wasm_bindgen).
  `[1.0, 3.0]` clamp · NaN·무한은 1.0 · 양자화 없음 · 반환값 = 실제 적용값. 계약 `methods` 에 두 이름 추가 + `setSpeedNote`.
- `wie_backend::SpeedClock` (새 파일 `wie-backend/src/speed_clock.rs`): 게스트 시계 = 마지막 배속 변경 시점의
  (벽시계, 게스트 시각) 닻 + 경과 × 배속. 바꾸는 순간 재닻이라 시각이 튀거나 되감기지 않고, 벽시계가 뒤로 가면 붙잡는다.
  배속을 한 번도 바꾸지 않으면 `now` = 벽시계 그대로(`untouched_clock_is_the_wall_clock`).
- `WebPlatform::now()` 가 그 시계를 읽고, `WieEmulator::tick` 은 `pacer` 예산(벽시계 ms)을 `clock.budget()` 으로 게스트 ms 로 바꿔 넘긴다.
  executor 는 예산을 게스트 시계로 재므로, 이렇게 해야 틱 한 번의 «벽시계» 몫이 배속과 무관하게 같다(브라우저 프레임을 더 쥐지 않는다).
- `wie_validate --speed X`: 같은 시계를 `HeadlessPlatform` 에 넣는 측정용 플래그. 기본 1.0 = 종전과 같다.

### 왜 플랫폼 시계 하나로 충분한가

게스트가 보는 모든 시간은 `Platform::now()` 를 지난다(실측 `grep '\.now()'` — WIPI `MC_knlGetCurrentTime`·`MC_knlSetTimer`,
MIDP `currentTimeMillis`·`Display` 타이머, LGT stdlib `time`, executor `sleep`/`tick_for`, GC 주기). 그래서 KTF·LGT·SKT·J2ME 를 따로 고치지 않았다.

### 소리 — 피치·템포 1배 유지 (권고안 그대로)

게스트 시계를 따르는 «타이밍»(Play 호출·효과음 시작 시점)은 배속을 따라간다. 워클릿이 실시간으로 합성하는 «음악 자체»는 1배 피치·템포로
울린다. 근거: ⑴피치를 올리면 3배에서 한 옥타브 반 위가 되어 듣기 어렵고 ⑵시퀀스 재생 속도를 바꾸려면 워클릿 렌더(`audio_worklet.js`)를
고쳐야 해 이번 범위(엔진 시계)를 넘는다 ⑶배속 중 곡이 끝나기 전에 게스트가 다음 Play 를 부르면 그 곡은 평소처럼 바뀐다 — 어긋남은 «곡 길이»뿐이다.

### 실측 — 플랫폼 4종 × {1, 2, 3}배 달성률

방법: release `wie_validate --frame-hz 60 --speed S --pacing 12/S --timeout 24/S`. 창을 «게스트 시간 [12 s, 24 s]» 로 맞췄으므로
게스트가 배속을 따라가면 창 안 횟수가 배속과 무관하게 «같다». 달성 배속 = S × 횟수(S) / 횟수(1). 지표는 그 플랫폼이 실제로 내는
진행 신호(그리기 `paints` · LGT 는 `paints` 계수에 안 잡혀 WIPI 타이머 발화 `timers`). 동시 3개 · 호스트 10코어 · load1 11→17.
제목은 sha256 앞 8자(로컬 코퍼스 · git 밖).

| 플랫폼 | 표본 | 지표 | 1배 | 2배 | 3배 | 달성 2배 | 달성 3배 |
|---|---|---|---|---|---|---|---|
| J2ME | `test_data/pace_j2me.zip` | paints | 240 | 240 | 239 | **2.00** | **2.99** |
| KTF (Java) | `7218e872` | paints | 149 | 150 | 139 | **2.01** | **2.80** |
| LGT | `287af341` | timers | 765 | 537 | 440 | 1.40 | 1.73 |
| LGT | `863b8ab6` | timers | 469 | 291 | 183 | 1.24 | 1.17 |
| SKT | `38277d63` | paints | 790 | 414 | 275 | 1.05 | 1.04 |
| SKT | `abc0c578` | paints | 1050 | 652 | 388 | 1.24 | 1.11 |
| KTF (clet) | `4b1c1947` | 화면 | — | — | — | 일치 | 일치 |

- J2ME·KTF Java = 배속을 다 낸다. 3배의 KTF 2.80 은 sleep 지연 p95 33 게스트 ms(= 벽 11 ms) — 동시 실행 부하에서 CPU 몫이 모자란 만큼이다.
- **SKT 두 표본은 배속이 «거의 안 먹는다»(1.04~1.24).** `38277d63` 은 sleep 0회(그리기 전달에 묶여 도는 루프 — 빠르기가 시계가 아니라
  호스트 틱 수로 정해진다), `abc0c578` 은 1 ms 폴 sleep 뿐이라 1배에서 이미 CPU 를 다 쓴다(1배 tick 48,925 짧은 틱 → 2배 655 긴 틱).
- LGT 는 1.2~1.7배 — 창 안에서도 tick 이 대부분 짧아(CPU 포화 아님) 원인을 이번 회차에 다 가르지 못했다. 정직하게 «낼 수 있는 만큼» 이 이 수다.
- KTF clet `4b1c1947` 은 `timers` 가 지표로 못 쓴다(창 안 500 → 49 → 28 인데 게임은 앞서 간다: 전체 화면 수 66 → 119).
  대신 화면을 게스트 시각에 맞춰 찍어 비교했다 — 게스트 6 s·12 s 프레임 해시가 1·2·3배 **셋 다 같다**(`6f151f2a`·`254c32b1`).

★**브라우저 한계(이 표가 못 보는 것)**: `wie_validate` 는 틱을 쉬지 않고 잇지만 브라우저는 rAF 당 1틱이다. WIPI `MC_knlSetTimer` 는
한 틱 안에서 이미 발화한 타이머의 두 번째 발화를 다음 호스트 프레임으로 미루고(`pace_from = tick + 1`), `MC_knlSetTimer(1)` 은 원래 프레임에
묶인다. ⇒ 1배에서 이미 60회/s 가까이 도는 타이머 루프는 브라우저에서 배속을 올려도 60Hz 화면이면 60회/s 를 못 넘는다. 1배 20 fps 짜리는 3배 60 fps 까지 간다.
후속 제안 = 그 상한을 배속에 비례시키기(worklog).

### 저장·타이머 표본 (1배 ↔ 3배, `--restart-at 5`)

| 플랫폼 | 표본 | 결과 | DB writes/reads (1배 · 3배) |
|---|---|---|---|
| KTF | `7218e872` | PASS · 재기동 2회 | 0/0 · 0/0 |
| LGT | `287af341` | PASS · 재기동 2회 | 3/1 · 3/1 |
| SKT | `abc0c578` | PASS · 재기동 2회 | 0/0 · 0/0 |
| J2ME | `pace_j2me` | PASS · 재기동 2회 | 0/0 · 0/0 |

타이머 지연(게스트 ms p95, 위 표의 창): J2ME 0~1 · KTF sleep 1/5/33 · LGT timer 0~10 — 깨짐 없음. 저장 경로는 시간과 무관한 데이터 쓰기라 1·3배 수가 같다.

### 검증

- 4게이트(fmt · clippy `-D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all` 769 pass) + beta clippy + `wie_featurephone` wasm clippy rc0.
- 단위 시험(`speed_clock.rs`): clamp 0.5→1.0 · 3.7→3.0 · NaN·∞→1.0 · 1.37 그대로 / 배속 6회 변경 동안 단조 증가 · 벽시계 역행 붙잡음 /
  구간 누적 / executor 위에서 `sleep(100)` 이 1배 ≈100 · 2배 ≈50 · 3배 ≈33 벽 ms 에 깸.
- 러너 블록 전건 PASS(keydraw 둘 rc=0) · `check-engine-contract.mjs` 137 pass 0 violation · `contract-roundtrip.mjs` OK.
- ★`wie_web.js` 39,598 B / 상한 40,000 B — 여유 402 B. 이번 두 메서드가 글루를 수백 바이트 늘렸고, 다음 export 하나가 상한을 넘길 수 있다.

### 사용자 영향

호출하지 않으면 1.0 = 종전과 같다. 셸(`otterpebble-featurephone-play-speed-slider`)이 핀 범프 뒤 슬라이더로 이 API 를 부르면
게임이 1~3배 사이 원하는 빠르기로 돈다 — J2ME·KTF Java 는 그대로, SKT·LGT 일부는 CPU·프레임에 묶여 덜 빨라진다.

게임 파일명 유입: 유입 7건(BOUNDED · 전건 이번에 고친 파일 3개의 «기존» 주석 — 이 회차가 쓴 줄에는 0) + 판단 필요 0건(SUFFIX-ATTACHED).
