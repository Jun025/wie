## [2026-10-02] lwc 폼 입력 — 셸 키를 자식 위젯으로 넘기고 버튼 동작을 부른다 (wie-lwc-form-key-dispatch-to-child-widgets-adopt-p0)

**무엇을**: 포커스를 기록하고, 맨 `ShellComponent` 가 키를 포커스 위젯에 넘기게 했다. 버튼은 FIRE 에서 `ActionListener.action` 을 부르고, 텍스트 칸은 숫자를 받고, ChoiceText 는 선택을 가진다. 폼은 자기 위젯을 그린다(`wie-wipi-java`).
**왜**: `0c67145b11df`(KTF AOT)는 이름 입력 폼에서 멈춰 있었다(0397 §4). 셸 `keyNotify` 가 stub 이라 모든 키가 거기서 끝났고, 버튼 리스너는 한 번도 불리지 않았다.
**사용자 영향**: 이름 입력 화면에서 위·아래로 칸을 옮기고 숫자를 넣고, 성별을 좌·우로 고르고, OK 를 눌러 게임에 들어간다. 넣은 숫자가 게임 안 이름으로 나온다(「12 오는 중~」). 이름에는 아직 숫자만 넣을 수 있다.

### 1. 이 층의 구조 — 바꾸기 전과 후
| 무엇 | 전 | 후 |
|---|---|---|
| 포커스 | `setFocus` stub, 저장 없음 | `net.wie.ShellCard` 의 정적 필드 `focus` 에 기록. lwc 클래스에 인스턴스 필드를 더하지 않았다(ShellCard 머리 주석의 LGT AOT 배치 이유) |
| 셸 키 | `Component.keyNotify` stub → true | 맨 `ShellComponent.keyNotify`: 포커스가 이 셸의 잎(컨테이너가 아닌 자식, 추가 순)에 있을 때만 동작. UP/DOWN 누름은 다음/이전 잎으로 포커스 이동(순환), 나머지 키는 포커스 잎의 `keyNotify` 로. 누름마다 `repaint`. 반환은 언제나 true(전과 같다) |
| 버튼 | 리스너 보관만 | FIRE 누름 → `action(이 버튼, o)`(javadoc `setActionListener`). `ActionListener` 에 추상 `action` 선언(없으면 `NoSuchMethodError` — 실측) |
| ChoiceText | 선택지 버림 | 선택지와 선택 보관 · `getSelectedIndex()I`(리스너가 읽는다 — 실측 벽) · LEFT/RIGHT 누름으로 선택 이동(순환) |
| 텍스트 칸 | `getString` = 늘 `"temp"` | 숫자 누름은 덧붙이고 CLR 은 지운다. 정본 버퍼 필드 `m_td` 에 둔다. 입력이 있으면 `getString` 이 그것을, 없으면 전처럼 `"temp"` |
| 그리기 | 셸·위젯 전부 no-op | 맨 `ShellComponent.paint` 가 자식 `paint` 를 부른다. `GFormComponent` 는 `addComponent(…IIII)` 의 상자를 보관하고 그린다: 흰 상자 + 글자/선택지, 버튼 이미지, 포커스는 빨간 테두리 |
| 배치 | 없음 | 없음(그대로). 상자는 게임이 준 값만 쓴다 |

- 위젯 `keyNotify` 는 처리하지 않은 키에도 true 를 돌려준다. 옛 stub 의 답과 같게 두었다 — 이 타이틀은 폼을 보이기 전에 텍스트 칸에 `keyNotify(1, LSOFT)` 를 직접 세 번 부른다.
- 상자를 지우는 이유(실측): 게임 쪽 카드는 다시 그릴 때 폼 영역을 덧칠하지 않는다. 지우지 않으면 지난 포커스 테두리와 지난 선택지가 남는다.
- `hasFocus` 는 여전히 false 다. 정직하게 답하면 «false 를 보고 키를 직접 처리하던» 타이틀이 바뀔 수 있다. 그건 따로 할 일이다.

### 2. 전/후 — `--inject` (release · `--max-ticks 1e11` · 전·후 바이너리를 같은 시각에 짝으로 · 2회)
전 = 이 브랜치의 base `5cbd635e`, 후 = 이 브랜치. 13:26 측정 직전 `host-load-guard --status --recovered` rc=0(load 27). 측정 중 load 는 116~130 까지 올랐다(다른 레인). 동시 실행은 2개.

| 키 순서 | 전 paints · frozen_tail | 후 paints · frozen_tail | 전 마지막 화면 | 후 마지막 화면 |
|---|---|---|---|---|
| 기본 27키 | 331·264 · 26·25 | 348·262 · 0·0 | 이름 폼 (같은 md5) | 게임 안 |
| 폼 경로(숫자·DOWN·RIGHT·OK) 18 | 289·256 · 17·17 | 282·237 · 0·0 | 이름 폼 | 게임 안 |
| OK×10 | 133·110 · 8·8 | 139·112 · 8·8 | 이름 폼 | 폼(첫 칸 포커스) |
| LSOFT+OK×9 | 143·106 · 8·8 | 148·113 · 8·8 | 이름 폼 | 폼(첫 칸 포커스) |
| NUM5×10 | 136·119 · 8·8 | 139·115 · 0·0 | 이름 폼 | 폼(첫 칸 «55555…») |

- 오류 0 · 예외 0(전·후 모두).
- OK 만 누르면 넘어가지 않는다 — 포커스가 첫 칸에 있고, FIRE 는 버튼의 것이다. 버튼까지 가려면 DOWN 세 번이 필요하다. 기본 27키에는 DOWN 이 있어서 넘어간다.
- 텍스트 칸 길이 제한은 없다. `setMaxLength` 는 아직 no-op 이다. NUM5×10 은 다섯 자 넘게 들어간다.

### 3. 대조 · 퇴행
- 단위 시험 `shell_walks_focus_to_the_button_and_fire_calls_its_listener`(shell_card.rs — 표시 장치 설치를 기존 시험과 함께 쓴다). 게임 순서대로 폼을 만든다: 칸 2 · ChoiceText · 버튼 → GFormComponent → ShellComponent, 첫 칸에 setFocus. 확인하는 것: 첫 칸에서 FIRE 하면 리스너 0회 · `NUM1 NUM2 CLR NUM3` 뒤 `getString` = "13" · DOWN DOWN RIGHT 뒤 `getSelectedIndex` = 1 · DOWN FIRE 하면 리스너 1회(인자는 버튼과 등록한 객체) · DOWN 이 순환해 FIRE 무반응 · UP FIRE 하면 2회.
- 변이 red: §6 에 적었다.
- 대상 밖 lwc 타이틀 15개. 코퍼스 문자열 스캔에서 `setFocus`·`ButtonComponent`·`ActionListener` 중 하나를 lwc `ShellComponent` 와 함께 갖는 것 + `85e94babc247`. 기본 27키로 전·후를 짝 2회 쟀다. 결과·오류 수·첫 예외가 전부 같다.
  - 마지막 화면 md5 는 같은 쪽 두 회차끼리도 다르다(예: `08b868809366` 전 2회). 그래서 md5 로는 가르지 않았다. 대신 후 빌드로 디버그 추적을 1회씩 돌렸다.
  - 새 코드에 닿는 타이틀은 `09a6a300994d`·`33f3e7669599` 2개뿐이다. 둘 다 `setFocus` 0회라 셸 `keyNotify` 는 일찍 돌아간다(전과 같은 답). 새 셸 `paint` 는 자식이 0개(`09a6…`)이거나 자식 `paint` 가 no-op stub(`33f3…`)이다.
  - 나머지 13개는 새 코드 진입 0. 대상 밖 동작 변경 0.

### 4. 게이트
- `cargo fmt --check` 0 · `cargo clippy --all -D warnings` 0 · wasm32 clippy 0 · `cargo +beta clippy --all -D warnings` 0 · `RUST_MIN_STACK=4194304 cargo test --all` 639 통과 · 실패 0.
- runner 블록: draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS · keydraw_ktf/lgt `--inject --expect-last-frame` PASS(paints 80 · 55 · rc=0).

### 5. 범위 밖으로 남긴 것
- 한글·영문 멀티탭 입력. 칸은 숫자만 받는다.
- `setMaxLength`·`maxLength` 기록. LGT 타이틀이 그 필드를 직접 읽는다(text_component.rs 주석). 그래서 이 회차에서 쓰지 않았다.
- 일반 lwc 배치·그리기. `GFormComponent` 밖의 위젯은 여전히 그리지 않는다.
- `compat.json` 의 조작 축(`no`)은 손대지 않았다. 그 축은 census 가 정한다(«키 넣은 실행이 무키 실행에 없던 프레임을 보이는가»). 다음 census 가 다시 잰다.

### 6. 변이 red (실측 5/5 — 위 단위 시험이 각각 실패)
| 변이 | 실패 단언 |
|---|---|
| `setFocus` 가 기록하지 않음 | `getString` = "13" |
| `ButtonComponent.keyNotify` 등재 제거 | 리스너 1회 |
| `ShellComponent.keyNotify` 등재 제거 | `getString` = "13" |
| `getString` 이 입력을 무시 | `getString` = "13" |
| `ChoiceText.keyNotify` 등재 제거 | `getSelectedIndex` = 1 |

### 7. 게임 파일명 유입
<!-- filled below -->
