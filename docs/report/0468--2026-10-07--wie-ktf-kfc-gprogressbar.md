## [2026-10-07] KTF `com/ktf/kfc/GProgressBar` — 시작 화면 벽 (wie-ktf-kfc-gprogressbar)

**무엇을**: `com/ktf/kfc/GProgressBar` 를 새로 넣었다(`wie-wipi-java`). 메서드는 `<init>()V` · `setMaximum(I)Z` · `setValue(I)Z` 셋이다.
**왜**: `83fc429f9cbe`(KTF · 재배치 client.bin)는 적재 후 이 클래스를 찾지 못했다. 게임이 그 예외를 잡은 뒤 null 진행 막대로 `startApp` 에서 NPE 가 났다(0466 §7).
**사용자 영향**: 그 타이틀이 이제 첫 화면과 메인 메뉴까지 나온다. 메뉴에서 게임을 고르면 불러오기 화면에서 게임 스레드가 죽는다(§4 — 다른 벽).

### 1. 모양 — 실행 추적에서만 정했다

head 바이너리에 `RUST_LOG=wie_ktf::runtime::java::interface=debug` 를 걸고 이 클래스(`0x4904fba0`)로 들어온 `get_java_method` 를 셌다. 전부 `startApp` 안에서 온다.

| 메서드 | 호출 | 값 |
|---|---|---|
| `<init>()V` | 1 | — |
| `setMaximum(I)Z` | 1 | 8 |
| `setValue(I)Z` | 8 | 1, 2, … 8 (적재 단계마다 1씩) |

- 처음에는 메서드 없는 빈 클래스로 돌렸다. 그때 실패가 `Method setMaximum(I)Z@87 not found` 였다. 생성자는 부모 `Component.<init>()V` 로 풀렸다. 그래서 이 클래스도 `()V` 생성자를 갖는다.
- 그 밖의 메서드는 부르지 않는다. 이미지의 이름표에도 이 둘 말고 진행 막대용으로 보이는 이름은 없다.
- **그리기는 넣지 않았다.** 게임이 막대를 컨테이너에 넣지 않고 `paint` 도 부르지 않는다. 불러오기 화면의 검은 막대는 게임이 직접 그린다.
- **추정 2건**: ⑴ 부모를 `org/kwis/msp/lwc/Component` 로 둔 것. 부모 쪽으로 가는 호출이 없어 근거가 없다. 같은 패키지의 lwc 위젯(`ChoiceText`)을 따랐다. ⑵ 두 setter 가 `true` 를 돌려주는 것. 추적 범위에서 게임이 그 값으로 갈리지 않았다.

### 2. 부팅 단계 — base `f5c02609` ↔ head

`wie_validate --timeout 20`(release · 키 없음):

| | base | head |
|---|---|---|
| 판정 | FAIL · stop error · paints 1 | **PASS · deadline · paints 165** |
| 멈춘 곳 | `NoClassDefFoundError: com/ktf/kfc/GProgressBar` → `startApp` NPE | 없음 — 20초 내내 첫 화면 |
| Java 예외 | 1 | 0 |

`--inject`(기본 27키)에서는 첫 화면 → 메인 메뉴 → 불러오기 화면까지 갔다. 메뉴 첫 칸(이어하기)과 둘째 칸(새로하기) 둘 다 같은 불러오기 화면에서 멈춘다(§4).

### 3. 퇴행 — 프로브 A/B 20종

`playability-census.mjs run --only probe --jobs 2`(long 풀 · 호스트 잠금) 으로 base → head 순서로 쟀다. load1 12~20 이다.
대상은 이 타이틀, `com/ktf/kfc` 를 쓰는 다른 3종(`0c67145b11df` `bfa8ec352451` `f07cbc782828`), 표준 KTF playable 16종(sha 순 앞 16)이다.

| 묶음 | 종 | 판정·stop·content·소리가 바뀐 것 |
|---|---|---|
| `83fc429f9cbe` | 1 | A·B 모두 FAIL(error) → PASS(deadline) · paints 1 → 33 / 245 · 예외 1 → 0 |
| kfc 3 · 표준 KTF 16 | 19 | 0 |

- Java 예외 수가 다른 칸이 셋 있었다(`01e2715ba07a` 8/6 · `070daa5b552c` 0/102 · `0e6cd188729e` 0/102). 같은 셋을 head → base 순서로 두 번 더 쟀다. base 끼리도 0 과 102 를 오갔고(`070daa5b552c` · `0e6cd188729e`), `01e2715ba07a` 는 두 번 다 6/6 이었다. 시점 차로 본다(0466 §6 과 같은 형태).
- 러너 줄(head): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 첫 실행이 `UNMEASURED · stop max-ticks` 였다. AGENTS 지시대로 `--max-ticks` 를 올리자 base·head 모두 PASS · rc=0 이었다(paints 80/79 · 55/55).

### 4. 남은 벽

| 대상 | 벽 | 크기 |
|---|---|---|
| `83fc429f9cbe` | 메뉴에서 게임을 고르면 불러오기 화면에서 게임 스레드(`c.run`)가 죽는다. 문자열 줄바꿈 코드(`substring` · `Font.stringWidth` 반복) 직후에 null 포인터 `+0x6c` 를 읽고 주소 0 으로 분기한다(`lr=0x1489f9`). 진행 막대 호출은 그 전에 끝났다. 그래서 이 클래스와 이어지는 근거는 없다 | 미정 — 원인 미조사 |

### 5. compat

`83fc429f9cbe` 행은 바꾸지 않았다. 프로브는 이 타이틀을 limited · input ok 로 계산한다. 하지만 그 판정은 주 스레드만 본다. 실제로는 게임을 시작하면 멈추므로 「시작하는 도중에 멈춰요」가 지금도 플레이어가 겪는 일이다. 첫 화면까지 나오게 된 것은 업데이트 소식(`docs/player-updates/2026-10-07-ktf-progress-bar-title-screen.json`)에 적었다.

### 6. 검증

- 4게이트(fmt · clippy stable/beta/wasm32 `-D warnings` · `cargo test --all`) 통과.
- 시험 `g_progress_bar_takes_maximum_and_value`: 생성 · 두 setter 의 반환값과 저장값을 본다. `setValue` 를 proto 에서 빼면 FAILED 였고, 원상에서 통과했다.
