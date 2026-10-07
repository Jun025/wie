## [2026-10-07] KTF 그래픽 컨텍스트 배치 — SDK 문자열 없는 3종 재확인 · paint 붙들기 — 컨텍스트 안 쓰는 KTF 10종 짝 측정 (wie-ktf-descriptor-array-loader-mninterface-three-boot)

**무엇을**: `docs/report/0463` §9 후속 표의 측정 2행.
⑴ 컨텍스트를 쓰지만 `WIPICX_incMemInterface` 가 없는 KTF 3종(`2b1ed0c8d061` `4166acd8fc62` `444821c513ac`)의 배치를 «판별 후 빌드»로 다시 쟀다.
⑵ 컨텍스트를 쓰지 않는 KTF 타이틀에서 paint 붙들기의 영향을 가드 밖 표본 10종으로 짝 측정했다.
엔진 코드 변경은 0이다.

**왜**: 0463 은 ⑴을 판별 «전» 빌드(KTF 전부 64바이트 워드)로 쟀다. ⑵는 가드 `49ade89578c5` 1종만 쟀다. 운영자 지시(2026-09-30 · 10-07)에 따른 것이다.
티켓의 할 일 1(서술자 배열 적재기 · `MNInterface` · `fp` 문맥)은 #499(`92784647` · `docs/report/0466`)가 먼저 끝냈다. 남은 `83fc429f9cbe` 는 `wie-ktf-kfc-gprogressbar`(#501)가 맡는다.

**사용자 영향**: 없음. 판정이 바뀐 타이틀이 0이라 compat·업데이트 소식은 고치지 않았다.

### 1. 방법

- 빌드: `origin/main` `f5c02609` release `wie_validate` 에 스크래치 스위치 2개를 넣었다(커밋 0).
  - `WIE_X_WIDE=1`: KTF 판별을 무시하고 64바이트 워드 배치를 켠다.
  - `WIE_X_NOHOLD=1`: `Display` 의 paint 붙들기(0463 §1)를 끈다.
  - 둘 다 켜지 않으면 main 과 같은 코드다. 같은 바이너리로 짝을 지었다.
- 스위치가 실제로 듣는지 양성 대조로 확인했다.
  - `WIE_X_WIDE=1` 이면 `keydraw_ktf.zip` 이 부팅 중 `Invalid memory access; address: 0` 으로 FAIL 한다(끄면 PASS). 0463 이 판별을 넣은 이유와 같은 증상이다.
  - `WIE_X_NOHOLD=1` 이면 `5267badf20b3` 이 그림 4,863 → 1 · 소리 9 → 0 · input ok → none · playable → limited 가 된다(0463 §1 의 벽이 되살아난다).
- 측정: `playability-census.mjs run`(probe A·B 30초 + longplay 600초 · 인자 그대로) · `report` 로 판정 · `build-slot run --long` 한 임대.
  - ★이탈: census 호스트 잠금은 다른 레인 셋(`r4`·`w4k`·`w4sl` 의 600초 진도 census)이 번갈아 쥐고 있었다. 그 셋이 long 슬롯도 쥔 채 기다리고 있어서, 잠금 순번을 기다리면 끝이 없었다. 그래서 0438·0463 선례대로 잠금 경로만 사적으로 바꿔 직접 실행했다.
  - `host-load-guard --status --recovered` 는 90분 내내 rc=1(idle 0%)이었다. 그래서 폭을 2로 줄였다(CLAUDE.md 측정 스윕 ⒞). 짝의 두 팔(main ↔ 스위치)을 **나란히** 하나씩 돌려 같은 부하 창을 나누게 했다.
  - 시각·load1: 20:03 → 22:34(load1 16 → 19 · 각 실행 18~37).

### 2. ⑴ SDK 문자열 없는 3종 — 배치가 동작에 드러나지 않는다

| sha12 | packed(main) | wide(강제) |
|---|---|---|
| `2b1ed0c8d061` | playable · A 그림 362 · ratio 0.990 | playable · 365 · 0.991 |
| `4166acd8fc62` | playable · 89 · 0.973 | playable · 88 · 0.971 |
| `444821c513ac` | playable · 612 · 0.996 | playable · 611 · 0.997 |

- 6축이 모두 같다. 다른 것은 소리 재생 횟수(`2b1e` 17 ↔ 13)와 그림 수 ±1% 뿐이다(타이밍 차).
- 원인(스크래치 진단 빌드 · 각 20초 · `read_grp_ctx` 마다 원시 64바이트를 찍었다):
  - 세 타이틀은 각자 컨텍스트 하나만 쓴다(`0x1acb20` · `0x1394a8` · 힙 하나).
  - 부르는 API 는 `MC_grpInitContext` 와 `MC_grpSetContext`(FG · CLIP)뿐이다. `MC_grpGetContext` 는 0회다.
  - 원시 바이트의 모든 변화가 우리 쓰기로 설명된다. 게임이 필드를 직접 쓴 흔적은 없다.
  - +52..+63(wide 만 쓰는 12바이트)은 packed 실행에서 늘 0이다. wide 쓰기가 남의 데이터를 덮은 흔적도 없다.
- 판정: 이 셋에게 배치는 엔진 내부 사정이다. 읽기와 쓰기가 같은 배치를 쓰는 한 결과가 같다. main 의 판별(packed)을 그대로 둔다.
  - 한계: 게임 코드가 컨텍스트를 **직접 읽는지**는 이 방법으로 보이지 않는다. 다만 그런 읽기가 있었다면 두 배치의 그림이 갈렸어야 한다.

### 3. ⑵ paint 붙들기 — 컨텍스트 안 쓰는 KTF 10종

표본: KTF 266종 중 컨텍스트를 안 쓰는 195종에서, compat playable 이고 가드(`49ade89578c5` `ddd885583b15`)가 아닌 것을 sha 순으로 10종 골랐다. 전부 `client.bin` 이다(KTF 의 Java 를 ARM 으로 컴파일한 형식).

| sha12 | 붙듦(main) | 안 붙듦 | 다른 칸 |
|---|---|---|---|
| `01e2715ba07a` | playable | playable | 없음 |
| `023c935aea8e` | playable | playable | 소리 18 ↔ 19 |
| `0392263fbb85` | playable | playable | 그림 574 ↔ 585 |
| `070daa5b552c` | playable · speed ok 0.992 | playable · speed **n/a 0.733** | §3-1 |
| `0865be217bde` | playable | playable | still 14 ↔ 12 |
| `08b868809366` | playable | playable | 소리 13 ↔ 7 |
| `09a6a300994d` | playable | playable | 없음 |
| `0a6f495f0b35` | playable | playable | 소리 16 ↔ 19 |
| `0a8a31b3c018` | playable | playable | still 29 ↔ 7 |
| `0c67145b11df` | playable | playable | still 3 ↔ 9 |

- 상태·boot·render·input·longplay·sound 축은 10종 모두 같다. speed 는 1종(`070daa5b552c`)만 갈렸고, 붙든 쪽이 빠르다(§3-1). 그림 수 차 ±2% 이내다.
- 소리 횟수와 `still`(장시간 같은 화면 최장 연속) 차는 양방향으로 흩어져 있다. 키 시점 차로 본다.

#### 3-1. `070daa5b552c` — 붙들기가 오히려 속도를 지킨다

- census 의 speed 축만 갈렸다(ok 0.992 → n/a 0.733). 속도는 벽시계 축이라 짝 3+3 으로 다시 쟀다. probe A 인자 그대로 · 두 팔을 나란히 · 23:06–23:07 · load1 29.
  | 실행 | ratio | sleep 늦음 합 | sleeps | redraw p95 | 틱을 넘은 redraw |
  |---|---|---|---|---|---|
  | 붙듦 ×3 | 0.990 · 0.988 · 0.988 | 87~125ms | 609~820 | 1ms | 0 |
  | 안 붙듦 ×3 | 0.721 · 0.730 · 0.715 | 5,834~6,200ms | 5,475~7,015 | 82~125ms | 127~173 |
- 그림 수는 짝마다 같다(2,390 ↔ 2,311 · 2,978 ↔ 2,948 · 2,891 ↔ 2,920). 다른 것은 «한 그림을 그리는 시간»이다.
  - 붙들기가 없으면 paint 가 게임 스레드와 섞여 여러 틱에 걸친다(redraw p95 1 → 82~125ms).
  - 그동안 게임 스레드의 sleep 은 늦게 깬다(늦음 합 약 50배).
- 판정: 이 타이틀에서 붙들기는 퇴행이 아니라 이득이다(재현 6/6). 이 표본에서 붙들기 때문에 나빠진 축은 없다.

### 4. 검증

- `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · wasm32 clippy rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` **730/0**(`build-slot` · 이 브랜치 = `origin/main` `f5c02609` + 이 문서).
- 엔진 코드 변경이 없어 «되돌리면 red» 시험은 없다. 기존 `paint_runs_with_the_other_guest_threads_held`·`a_wide_context_is_read_where_the_title_wrote_it` 가 그대로 지킨다.
- 회차 밖 프로세스 0. 스크래치 빌드·패치는 커밋 0이고 워킹트리에 남은 것도 0이다.

### 5. 후속 표

| 수 | 벽 | 계급 | 크기 |
|---|---|---|---|
| — | ⑴ 3종 | 판정(배치 무관 · 바꾸지 않음) | — |
| — | ⑵ paint 붙들기 | 판정(10종 퇴행 0 · `070daa5b552c` 는 붙들기 쪽이 빠르다) | — |
| 1 | `83fc429f9cbe` `GProgressBar` | 엔진 · #501 진행 중 | — |

### 6. 게임 파일명 유입

- 도구 표기 BOUNDED 0쌍 · SUFFIX-ATTACHED 0쌍. 타이틀은 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=1 tree=90af90e018bb7f79 B=0/0 P=0/0 S=0/0 -->
