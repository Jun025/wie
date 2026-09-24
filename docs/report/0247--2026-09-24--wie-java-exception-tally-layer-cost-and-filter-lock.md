## [2026-09-24] java_exceptions 레이어 — keydraw_lgt UNMEASURED 는 레이어 비용이 아니다 · 전역 필터 퇴행 잠금 (wie-java-exception-tally-layer-cost-and-filter-lock)

**무엇을** — #277(게이트② F1·F2·F3) 후속.
⑴ F2 를 재측정했다. 결론은 잡음이다. 코드 변경은 없다.
⑵ F1 을 잠갔다. `main` 의 구독자 조립을 `validator_subscriber(stderr_filter, tally)` 로 뽑아냈다. 그 조립을 `RUST_LOG` 미설정 필터로 돌려 계수를 확인하는 시험을 추가했다.
⑶ F3: doc 주석 문구를 정정했다.

**왜** — 게이트②에서 두 가지가 나왔다.
- F1: 변이 M4(레이어별 필터 → 전역 `EnvFilter`)가 기존 시험 4개를 전부 통과했다. 이 변이가 들어가면 `RUST_LOG` 가 없을 때 `count:0` 이 조용히 나온다.
- F2: `keydraw_lgt` 에서 head 가 11회 중 2회 UNMEASURED 였고, base 는 11/11 PASS 였다(confidence low).

**F2 재측정** — 두 바이너리를 비교했다.
- 대조 1: 현 main `ef65adb0` debug 빌드(head).
- 대조 2: 같은 트리에서 `.with(java_exception_layer(..))` 한 줄만 뺀 빌드(nolayer).
- 명령은 `--inject --expect-last-frame test_data/keydraw_lgt.zip` 이고 기본값을 그대로 썼다.
- 짝 안에서는 두 바이너리를 연달아 돌렸다. 순서는 짝마다 HN/NH 로 번갈았다. 매 실행 직전 load1 을 기록했고, 짝 안에 load1 ≥ 40 이 있으면 그 짝은 무효로 쳤다.

| 회차 | 유효 짝 | 무효 | head max-ticks(UNMEAS) | nolayer max-ticks(UNMEAS) | FAIL | 유효 load1 |
|---|---|---|---|---|---|---|
| 1 | 18 | 2 | 2 (1) — HN 1 · NH 1 | 0 (0) | 0 | 15.7–38.1 |
| 2 | 17 | 3 | 1 (1) — HN 1 | 2 (2) — HN 2 | 0 | 7.9–39.97 |
| 계 | 35 | 5 | **3 (2)** | **2 (2)** | 0 | |

(무효 짝의 nolayer 1회는 load 46 에서 FAIL 이었다. paints 24 로, AGENTS.md 에 적힌 부하 굶김 형태다.)

- 대조 바이너리도 같은 UNMEASURED 를 낸다. 2회차 9번 짝에서는 **두 바이너리가 연달아** 23/27 과 22/27 에서 max-ticks 로 멈췄다.
- max-ticks 정지는 전부 **저부하**(load1 11.0–29.5)에서 났다. 5,000만 틱이 16–18초 만에 소진됐다. 2회차 `ticks` 는 load 에 따라 35 에서 4,900만까지 흔들렸다.
- 레이어에 «비용»이 있다면 틱이 느려져야 하는데, 관측은 반대로 틱이 «너무 빨라서» 백스톱에 닿은 것이다.
- 판정: **부하 잡음이다. 레이어 비용이 아니다.** 게이트② 표본의 비대칭(2/11 대 0/11)도 이 기제로 설명된다.

**F1 잠금** — 새 시험 `java_exception_tally_counts_under_main_subscriber_with_rust_log_unset_test`.
- `validator_subscriber(EnvFilter::new(""), tally)` 로 실제 `Jvm::exception` 을 1회 던지고 count 1 을 단언한다.
- 변이 M4(`validator_subscriber` 안에서 `.with(stderr_filter)` 를 전역으로)를 넣으면 **5개 중 1개 FAILED**(이 시험)가 된다. 원상 복구하면 5/5 green 이다.
- source-shape 시험의 needle 은 `main` 이 `validator_subscriber(…from_default_env(), …)` 를 부르는지 보도록 바꿨다.
- 실제 바이너리로도 확인했다(`RUST_LOG` 미설정): draw_j2me count 2 · helloworld_ktf count 0.

**사용자 영향** — 없음. 검증기 출력과 판정이 모두 같다.
