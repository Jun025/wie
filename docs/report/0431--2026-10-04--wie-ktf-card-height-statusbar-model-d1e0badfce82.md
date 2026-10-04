## [2026-10-04] KTF 상태 표시줄 자리 — `show()` 뒤의 Card 는 24행 아래 · 높이 296 (wie-ktf-card-height-statusbar-model-d1e0badfce82)

**무엇을**: KTF 게임이 `AnnunciatorComponent.show()` 를 부르면, 240폭 화면에서 위쪽 24행을 상태 표시줄 자리로 비운다. 그 뒤 기본 생성자로 만든 Card 는 `y=24 · 높이 296` 이고, `org.kwis.msp.lcdui.Display.getHeight()` 도 296 을 답한다. 표시줄 자체는 그리지 않는다.
**왜**: `d1e0badfce82` 은 띠 수를 `getHeight()/10 + 1` 로 정하고 32칸 표를 읽는다. 높이가 320 이면 33번째 칸에서 매 paint `ArrayIndexOutOfBoundsException` 이 났다(0422 §4). 경계 검사는 게임의 AOT 이미지 안에 있다. 그러니 실기도 높이 ≤ 319 를 줬어야 한다.
**사용자 영향**: 그 보드게임의 판·말·정보창이 그려진다. 표시줄을 보이는 다른 240폭 KTF 게임은 화면이 24행 아래로 내려앉는다. 짝 전수에서 판정이 나빠진 타이틀은 0 이다(§3).

증적: `~/orchestrator/reports/evidence/wie-ktf-card-height-statusbar-model-d1e0badfce82/`. 타이틀은 sha12 로만 적는다.

### 1. 근거 — 코퍼스 실측(문서 정본은 찾지 못했다)

`game_lab/{working,broken}/ktf` 을 sha256 으로 중복 제거하면 KTF 261종이다. 그중 `DisplaySize:240*320` 은 164종이다. ADF 에는 표시줄을 말하는 키가 없다.

- **전폭 그림 높이**: 타이틀마다 폭이 240인 그림 중 가장 큰 높이를 PNG·GIF·JPEG 헤더에서 쟀다. 결과는 아래 표다.
  - 296 이 최빈값이다. 이름은 `title`·`bg`·`MainTitle`·`menubg`·`BACKGROUND` 류다.
  - 한 패키지는 폴더 이름이 `img/292/`, 한 패키지는 `_240x306` 이다. 단말마다 표시줄이 달랐던 흔적이다.

  | 최대 높이 | 290 | 292·294·297·301·304·306 | **296** | 298 | 300 | 320 | 전폭 그림 없음 |
  |---|---|---|---|---|---|---|---|
  | 타이틀 | 7 | 각 1 | **10** | 2 | 3 | 5 | 128 |

- **176폭 대조**: 202(= 220 − 18)가 7종으로 최빈값이고, 220 은 4종이다. 이번 모델에는 넣지 않았다(worklog 제안).
- **API 흔적**(`client.bin` 문자열): 164종 중 `org/kwis/msp/lwc/AnnunciatorComponent` 를 가진 타이틀이 119종이다. `d1e0badfce82` 은 `new AnnunciatorComponent(false)` → `show()` → `new Card()` 순서로 부른다(런타임 로그).
- **SDK**: 레퍼런스 에뮬레이터 설치본(NSIS)은 이 맥에서 풀 수 없었다. 릴리스 노트에는 «Annunciator 를 추가하면 Layout 오류» 수정 기록만 있고, 높이 수치는 없다.
- 24 는 LGT 의 240폭 표시줄(`wie-lgt` WIPI-C graphics)과도 같다.

### 2. 구현

- `wie-wipi-java/.../lwc/annunciator_component.rs`
  - 정적 필드 `shownHeight` 를 더했다. 인스턴스 필드는 LGT AOT 하위 클래스의 필드 오프셋을 움직이므로 쓰지 않았다.
  - `show()` 는 화면 폭이 240 일 때만 24 를 쓴다.
  - 표시줄 자신의 `getWidth/getHeight` 는 종전대로 0 이다(`b475b6399684` 주석 · 무변경).
- `lcdui/display.rs`: `getHeight` = 화면 높이 − `shownHeight`.
- `lcdui/card.rs`: 기본 생성자 `(Z)`·`(Display)` 의 y 를 `shownHeight` 로 둔다. `CardCanvas` 가 Card 의 x·y 로 translate 하므로 그림은 그만큼 아래에 그려진다.
- `show()` 를 부르지 않는 타이틀은 빼는 값이 0 이다. 그래서 동작이 그대로다.
- 시험 `shown_strip_moves_new_cards_below_it_on_a_240_wide_screen`
  - 240×320 에서 show 전 카드는 `[y 0, w 240, h 320]` 이다.
  - show 후 Display 높이는 296, 새 카드는 `[24, 240, 296]` 이다.

### 3. 짝 전수 — 판정 뒤집힘 0

`scripts/playability-census.mjs run --only probe` 를 썼다(probe A 27키 · B 키 없음 · 각 30초 · `--jobs` 기본 3 · census 락 대기 후 · host gate rc=0 확인 후 · `build-slot` 경유). base 는 `d1ba7687`, 후는 같은 트리에 모델을 더한 빌드다. 축은 `status`·`boot`·`render`·`input`·`sound` 이다.

| 집합 | 수 | 뒤집힘(나빠짐) | 좋아짐 | 측정 |
|---|---|---|---|---|
| KTF `DisplaySize:240*320` 전수 | 164 | **0** | 1(`1b3b4868d46e` sound silent→ok) | base 04:58~05:30 · 후 05:39~06:27 · load1 12~27 |
| `AnnunciatorComponent` 를 가진 LGT·SKT | 18(LGT 17 · SKT 1) | **0** | 1(`517ed32c92d6` sound silent→ok) | base 09:24~09:45(★load1 최고 109) · 후 09:45~10:22 · load1 16~19 |

- 분포는 전·후 같다. KTF 는 `limited 159 · not-yet 5` 다. boot·render 의 `fail/none 5` 는 전·후 같은 타이틀이다.
- sound 의 «좋아짐» 2건은 고침이라고 주장하지 않는다. 키 일정·부하에 따라 흔들리는 축이다.
- LGT·SKT 짝은 base 가 높은 부하에서 돌았다. 나빠짐 0 은 그 부하에서도 성립했다는 뜻이다. paints 수는 부하에 따라 움직이므로 비교하지 않았다.
- `d1e0badfce82`(60초 · 27키 · 같은 인자): base 는 PASS · 마지막 프레임 32색이고 판이 없다. 후는 PASS · AIOOBE 0 · 마지막 프레임 146색이고 판·말·정보창이 보인다(`d1e0-h320-base.png` · `d1e0-strip24-model.png`).
  - census 프로브는 두 빌드 다 `limited` 로 같다. 프로브 축은 게스트 예외로 끝난 paint 를 «그렸다»로 센다.

### 4. 한계

- 24 는 최빈값이다. 단말마다 290~306 이 섞여 있다. 그 단말들의 실제 높이를 가리는 근거는 없다.
- 표시줄을 그리지 않으므로 위 24행은 비어 있다. 전폭 320 그림을 쓰는 5종이 `show()` 를 부르면 아래 24행이 잘린다. 프로브 축은 그대로였다.
- longplay·progress 축은 다시 재지 않았다(`--only probe`).

### 5. 게이트

- fmt OK.
- clippy stable·beta·wasm32 `-D warnings` rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0.
- 러너 블록 전건 PASS(`keydraw_ktf` paints 79 · `keydraw_lgt` 55).
- 스크래치 타깃에서 `build-slot` 경유로 돌렸다.

### 6. 유입

- `node scripts/corpus-name-inflow.mjs` 결과: BOUNDED 1건, SUFFIX-ATTACHED 0건.
- 그 1건은 `shell_card.rs` 에 **main 에 이미 있던** 한 줄이다(`origin/main` 본문에 1회 · 이 PR 의 diff 에 0회). 파일을 고친 탓에 대상에 잡혔다.
- ⇒ 이 회차가 들인 게임 파일명은 **0건**이다.

<!-- corpus-name-inflow v1 subjects=7 tree=80985cfe5abf14cd B=1/1 P=1/1 S=0/0 -->
