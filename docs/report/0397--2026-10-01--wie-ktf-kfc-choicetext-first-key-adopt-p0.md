## [2026-10-01] KTF com/ktf/kfc ChoiceText — 첫 키 벽 (wie-ktf-kfc-choicetext-first-key-adopt-p0)

**무엇을**: `com/ktf/kfc/ChoiceText`·`com/ktf/kfc/GFormComponent` 를 새로 넣고 lwc `Component.isShown()Z` 를 더했다(`wie-wipi-java`).
**왜**: `0c67145b11df`(KTF AOT)는 첫 키에서 이름 입력 폼을 만든다. 그 폼의 부품이 없어 `NoClassDefFoundError: com/ktf/kfc/ChoiceText` 로 멈췄다(0388 §3).
**사용자 영향**: 첫 키에서 굳던 화면이 이름 입력 화면으로 넘어간다. 아직 이름을 넣고 넘어갈 수는 없다(§4 — 다음 벽).

### 1. 넣은 모양 — 전부 실측(get_java_method 로그)
API 문서가 이 repo 에 없어서, 게임이 실제로 찾는 서술자만 넣었다. 클라이언트는 AOT(client.bin)라 바이트코드가 없다.
| 클래스 · 메서드 | 근거 |
|---|---|
| `ChoiceText.<init>([Ljava/lang/String;)V` | 게임이 찾는 유일한 ChoiceText 메서드 |
| `ChoiceText` 부모 = lwc `Component` | 인스턴스가 바로 `GFormComponent.addComponent(Lorg/kwis/msp/lwc/Component;IIII)I` 의 첫 인자로 간다 |
| `GFormComponent.<init>()V` · `addComponent(Component;IIII)I` | ChoiceText 다음 줄의 벽(`NoClassDefFoundError: com/ktf/kfc/GFormComponent` → `Method addComponent(…IIII)I not found`) |
| `GFormComponent` 부모 = `ContainerComponent` | 자식 4개를 받고, 폼 자신이 `ShellComponent.addComponent(Component)I` 로 넘어간다. 부모 클래스는 **추론**이다 |
| `Component.isShown()Z` | 그다음 벽: 게임 `paint` 가 셸에 묻는다(`Method isShown()Z not found from org/kwis/msp/lwc/ShellComponent`). 판정은 `repaint` 와 같은 `ShellCard::find` |
- 네 int 는 (x, y, w, h)로 보인다: (110,135,60,17) (110,160,60,17) (120,183,36,15) (107,300,22,8). 이 층은 배치를 하지 않아서 버린다.
- 선택지 배열은 보관하지 않는다. 읽는 메서드를 게임이 찾지 않았다.
- ChoiceText 하나만 넣으면 벽이 한 줄 뒤(GFormComponent)로 옮겨질 뿐이었다. 같은 생성 경로의 세 벽을 함께 넣었다.

### 2. 전/후 — `--inject` 장수 (release · `--max-ticks 1e11` · 같은 시각 짝 2회)
| 키 순서 | 전 paints | 전 오류 | 후 paints | 후 오류 | 후 마지막 화면 |
|---|---|---|---|---|---|
| 무키(20초) | 227 · 256 | 0 | 245 · 253 | 0 | 타이틀 |
| 기본 27키 | 16 · 17 | ChoiceText | 269 · 240 | 0 | 이름 입력 폼 |
| OK×10 | 18 · 14 | ChoiceText | 102 · 109 | 0 | 이름 입력 폼 |
| LSOFT+OK×9 | 16 · 16 | ChoiceText | 117 · 115 | 0 | 이름 입력 폼 |
| NUM5×10 | 17 · 12 | ChoiceText | 119 · 106 | 0 | 이름 입력 폼 |
- 네 키 순서 모두 전에는 첫 키에서 같은 오류, 후에는 오류 0. 키를 넣은 8회 전부 마지막 프레임이 같은 폼 화면(같은 md5)이다.
- 호스트 load1 32~39(12:39~12:42). 절대 수치는 하한이다.

### 3. 대조 · 퇴행
- 단위 시험 `choice_text_goes_into_a_g_form_component`: 게임 순서 그대로(생성 → 폼 → addComponent(…,120,183,36,15) → getComponent(0)). 기존 `shown_shell_component_is_painted_through_the_display` 에 isShown 단언 3개(보이기 전 false · show 뒤 true · hide 뒤 false).
- 변이 red(실측 4/4): ChoiceText 등재 제거 · ChoiceText 부모를 Object 로 · isShown 을 상수 true · 상수 false.
- 같은 툴킷을 참조하는 다른 타이틀(코퍼스 문자열 스캔 4종): `bfa8ec352451`(GFormComponent 포함) 전/후 2회 모두 `NoClassDefFoundError: com/ktf/kfc/GMenubarForm` 로 같다. `83fc429f9cbe` 는 client.bin 배치 미지원으로 전/후 같다. `f07cbc782828` 는 GForm 계열이라 이번 클래스를 쓰지 않는다.
- runner 블록: draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS · keydraw_ktf/lgt `--inject --expect-last-frame` PASS(paints 80 · 55).
- `isShown` 은 그 전에 찾으면 치명 오류였던 메서드라, 지금 도는 타이틀의 경로를 바꾸지 않는다.

### 4. 다음 벽 — com/ktf/kfc 부재 목록과 이 타이틀의 다음 벽
- 이 타이틀: 폼은 뜨지만 **키가 폼 안으로 가지 않는다**. 셸의 `keyNotify` 는 stub 이고(true 반환), lwc 위젯(TextBox·Button·ChoiceText)은 그려지지 않으며, 버튼의 ActionListener 는 불리지 않는다. 이름을 넣고 넘어갈 길이 없다. 그래서 compat.json 의 조작 축(`no`)은 바꾸지 않았다.
- 코퍼스의 com/ktf/kfc 참조(이번 회차 구현 0): `GForm`·`GFormBase`·`GMenuBar`·`GMenubarForm`·`GTextField`(`bfa8ec352451`) · `GForm`·`GMenubarForm`·`GMsgBox`·`GTextField`·`GTextListener`(`f07cbc782828`) · `GProgressBar`(`83fc429f9cbe` — 그 전에 client.bin 배치 벽).

### 5. 게임 파일명 유입
- 유입 2건(BOUNDED) + 판단 필요 0건(SUFFIX-ATTACHED). 2건 모두 이번 회차가 고친 파일(`shell_card.rs`·`component.rs`)에 **원래 있던** 주석 속 제목이다 — 이 PR 이 더한 줄에서는 0(`git diff origin/main` 의 `+` 줄 0).
- 표식 줄은 도구 출력 그대로다.

<!-- corpus-name-inflow v1 subjects=12 tree=79cc6210bb03615d B=2/2 P=1/1 S=0/0 -->
