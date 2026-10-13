## [2026-10-13] playable 중 속도 «알 수 없음» 42종 재측 — 21종 ok · 남은 21종은 부하가 아니라 측정 구조 탓 (wie-playable-speed-unknown-42-quiet-host-remeasure)

**무엇을**: `compat.json` 의 playable 411종 가운데 speed `unknown` 42종을 `origin/main` `d7059f1f` 바이너리로 다시 쟀다. 쓴 것은 6차와 같은 도구·같은 기준이다(`playability-census.mjs run --only probe` → `--only speed --jobs 1` · 판정 `judge` = A·S 두 읽기 중 나은 쪽 ≥ 0.9). 그 결과 21행을 `unknown → ok` 로 바꿨다.

**왜**: 운영자 지시(2026-09-30·10-13): 지원 현황 정확도. 6차(0432 §2)는 이 축을 «부하 속 하한이라 `n/a`» 로 남겼다.

**사용자 영향**: 21종의 지원 현황 페이지에 «속도 정상»이 표시된다. status 수는 그대로다(411/10/8 — speed 는 status 에 들어가지 않는다).

### 1. 측정 창 — 조용한 창은 끝내 오지 않았다
| 단계 | 시각 | load1 | idle | host-load-guard |
|---|---|---|---|---|
| probe A·B 42종(jobs 3) | 08:43–08:58 | 12–30 | 2–40% | 시작 rc=0 · 09:00 판정은 `saturated`(20분 4표본 전건 < 5%) |
| speed S1(jobs 1) | 09:00–09:02 | 8–12 | 0.1–3% | rc=1 saturated |
| speed S2(jobs 1, 같은 6종) | 09:22–09:25 | 8–12 | 11% | rc=1 recovering(포화 11분 전) |

09:00 부터 09:22 까지 가드가 rc=0 으로 돌아오지 않았다(5분 표본 6개 중 idle ≥ 15% 는 1개). ★다만 결론에는 영향이 없다:
- **ok 21**: 부하는 지연을 «더하기»만 하므로, 바쁜 호스트에서 ≥ 0.9 면 그대로 ok 다(`judge` 주석).
- **비율이 있는 n/a 6**: A·S1·S2 세 읽기(load1 8~25)가 서로 ±3% 안이다(`1cd151222bde` 는 A 한 읽기뿐)(§3). 부하가 원인이면 이렇게 고정되지 않는다.
- **비율이 없는 n/a 15**: 부하와 상관이 없다(§3 N·X).

### 2. 42행 전/후
| sha12 | 플랫폼 | 전 | 후 | 비율(나은 쪽) | 계급 |
|---|---|---|---|---|---|
| `070daa5b552c` | KTF | unknown | ok | 0.994 | — |
| `0c67145b11df` | KTF | unknown | ok | 0.991 | — |
| `0cc4ef7ede37` | KTF | unknown | ok | 0.990 | — |
| `0e6cd188729e` | KTF | unknown | ok | 0.995 | — |
| `31c90441f639` | SKT | unknown | ok | 0.920 | — |
| `33f3e7669599` | KTF | unknown | ok | 0.993 | — |
| `3c658a46bbfb` | KTF | unknown | ok | 0.999 | — |
| `517ed32c92d6` | LGT | unknown | ok | 0.996 | — |
| `59263295de74` | KTF | unknown | ok | 1.000 | — |
| `5d3ba49eccf7` | KTF | unknown | ok | 0.996 | — |
| `7218e8720f8c` | KTF | unknown | ok | 0.978 | — |
| `79d9bc10be82` | KTF | unknown | ok | 0.991 | — |
| `7f40060723fc` | SKT | unknown | ok | 0.953 | — |
| `9789fec50f39` | KTF | unknown | ok | 0.965 | — |
| `990ae27f67e6` | SKT | unknown | ok | 1.000 | — |
| `a16f08d025eb` | LGT | unknown | ok | 0.995 | — |
| `bc94ba53677b` | KTF | unknown | ok | 0.997 | — |
| `bf54c05e58a9` | SKT | unknown | ok | 0.998 | — |
| `d552e095ddcf` | KTF | unknown | ok | 0.999 | — |
| `f07cbc782828` | KTF | unknown | ok | 0.996 | — |
| `f78334f270be` | SKT | unknown | ok | 0.957 | — |
| `0eb19d9bbe7a` | KTF | unknown | unknown | — | N |
| `1b3b4868d46e` | KTF | unknown | unknown | — | N |
| `1cd151222bde` | LGT | unknown | unknown | -197.374 | D |
| `2cbd63e4427a` | KTF | unknown | unknown | — | N |
| `38277d63b0ba` | SKT | unknown | unknown | — | N |
| `4892a1abc0f8` | KTF | unknown | unknown | -408.374 | D |
| `4a7e489bf6ab` | KTF | unknown | unknown | — | N |
| `55aadf368b8e` | KTF | unknown | unknown | — | N |
| `6103e87874c6` | KTF | unknown | unknown | — | N |
| `61ed69520fd3` | LGT | unknown | unknown | — | N |
| `689c491586b9` | KTF | unknown | unknown | — | X |
| `7089dec0e8df` | SKT | unknown | unknown | — | N |
| `71d1d8235bd1` | SKT | unknown | unknown | 0.865 | W |
| `7f73422ba19b` | KTF | unknown | unknown | — | N |
| `86132b2ed76f` | KTF | unknown | unknown | — | N |
| `9acb5e0387eb` | LGT | unknown | unknown | 0.374 | T |
| `ca1132f2e7bc` | SKT | unknown | unknown | — | N |
| `cdd3eb5f9142` | KTF | unknown | unknown | -413.646 | D |
| `d5e996a53118` | KTF | unknown | unknown | 0.779 | T |
| `ea35907b22a4` | KTF | unknown | unknown | — | N |
| `f81658d4ad1a` | SKT | unknown | unknown | — | N |

- 6차 `out`(4ac38566)에서 이 42종 중 비율이 있던 것은 8종뿐이었고 34종은 `null` 이었다(창 안에 sleep·타이머가 0, 또는 프로브 A 가 창에 닿지 못함). 이 핀에서는 그중 19종에 비율이 생겼고 전부 ≥ 0.92 다. 엔진이 더 깊이 들어가 대기 경로에 닿은 것이다.

### 3. 남은 21종 군집표
| 계급 | 수 | 실측 | 판정 | sha12 |
|---|---|---|---|---|
| **D** 타이머 중복 arm 누적 | 3 | 콜백 하나가 같은 타이머를 `MC_knlSetTimer` 로 **두 번** 걸고, Unset 은 0회다(15초 debug 로그에서 Set 1,735/6,215회 · <1ms 간격 쌍 864/3,104). `arm_timer` 는 «두 Set = 두 콜백»을 유지하므로 대기열이 콜백마다 1개씩 늘어난다. 그래서 `timer_late` 가 창 안에서 선형으로 커진다(p50 11 s · p95 21 s) ⇒ 비율 −197~−430 | ★**느림이 아니다** — 콜백은 명목(21 ms ⇒ 47.6/s)보다 빠르게 돈다(57~64/s). 비율은 이 형상에서 무의미하다 ⇒ `unknown` 유지 | `cdd3eb5f9142` `4892a1abc0f8` `1cd151222bde` |
| **T** 타이머 지연(부하 무관) | 2 | `9acb5e0387eb`: 40 ms 타이머 · late p50 41 / p95 97 ms · 22 s 창에 370회 = 16.8/s(명목 25/s ⇒ **0.67**) · paints ≈ 발화 수 · 세 읽기 0.341/0.374/0.340. `d5e996a53118`: 5 ms 타이머 · late p50 0 / p95 8 · 155/s(명목 200/s ⇒ 0.78) · 세 읽기 0.763/0.779/0.775 | 느림 **후보**. 헤드리스는 도구 규칙상 `slow` 를 말하지 않는다(브라우저 짝만 말한다) ⇒ `unknown` 유지. `9acb` 는 키 없이 15초 돌린 debug 런에서 24.7/s(명목) — 키 뒤 화면에서만 늦다 | `9acb5e0387eb` `d5e996a53118` |
| **W** 주 루프가 아닌 sleep | 1 | sleep(20) 이 11 ms 늦게 깬다(T1 격자) · 다만 sleep 은 11/s 이고 paints 는 75/s(yield 루프) | 비율 0.86~0.87 은 프레임 대기가 아닌 sleep 을 센 값이다. 느림 근거가 아니다 ⇒ `unknown` 유지 | `71d1d8235bd1` |
| **N** 대기 없음 | 14 | 창 안 sleep 0 · 타이머 0. yield 스핀(`7089dec0e8df` `6103e87874c6` `1b3b4868d46e` — yields 1.2M~2.1M) · repaint 주도(`4a7e489bf6ab` `86132b2ed76f` `38277d63b0ba` `2cbd63e4427a` — yields ≈ paints) · 시계 바쁜 루프(ticks 10⁷~10⁸) 나머지 | 비율의 분모(대기)가 없어 **이 도구로는 원리상 잴 수 없다** ⇒ `unknown` 이 정확하다 | `0eb19d9bbe7a` `1b3b4868d46e` `2cbd63e4427a` `38277d63b0ba` `4a7e489bf6ab` `55aadf368b8e` `6103e87874c6` `61ed69520fd3` `7089dec0e8df` `7f73422ba19b` `86132b2ed76f` `ca1132f2e7bc` `ea35907b22a4` `f81658d4ad1a` |
| **X** 창 미도달 | 1 | 프로브 A 가 키 5/27 뒤 clean exit(6차도 pacing 없음) | 잴 창이 없다 | `689c491586b9` |

**slow 확정: 0.** 계급 D·W 는 «느림 아님»이 실측이고, T 2종만 느림 후보다.

### 4. 갱신 방법(행 단위)
1. `cargo build --release -p wie_cli --bin wie_validate`(`d7059f1f` · `build-slot`).
2. 42종을 sha12 이름 심링크로 모은 코퍼스에 `bash scripts/census-drive.sh --bin <bin> --out <out> --only probe <corp>` → `… --only speed --jobs 1 <corp>`(S 를 `S0.json` 으로 옮긴 뒤 두 번째 읽기).
3. `node scripts/playability-census.mjs report --out <out> --pin d7059f1f` 의 `census.tsv` 에서 speed `ok` 인 행만, `compat.json` 의 그 행 `axes.speed` 를 `unknown → ok` 로 바꿨다. 다른 필드·행은 손대지 않았다. `enginePin` 은 4ac38566 그대로다(행 전체 재생성이 아니다).
4. `node scripts/player-data.mjs` OK(429 · 411/10/8) · `node scripts/check-compat-revert.mjs` OK.
- 이용자 소식은 내지 않았다. 엔진을 바꾸지 않은 측정 갱신이다(wave3·6차 선례).

### 5. 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs`: BOUNDED 330쌍 + SUFFIX-ATTACHED 16쌍. 전부 이 PR 이 건드린 `compat.json` 의 기존 제목 값이다. 이 PR 은 그 파일에서 `axes.speed` 값만 바꿨다(제목 문자열 차이 0). 그 밖 파일 0. 타이틀은 sha12 로만 적었다.


<!-- corpus-name-inflow v1 subjects=3 tree=5d7199ceb4b041a5 B=718/330 P=0/0 S=36/16 -->
