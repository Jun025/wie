## [2026-10-10] SKT 14a62a8521a0 캐릭터 이름 상자 — 처리기로만 오는 키를 포커스 입력칸에 넘긴다 (wie-skt-14a62a8521a0-name-box-no-input-adopt-p0)

**무엇을**: `com.xce.lcdui.XTextField` 가 `setFocus(true)` 때 자기를 정적 `focused` 에 둔다. `TextComponentHandler.keyPressed` 는 등록된 `TextComponent` 가 없으면 그 입력칸의 키 처리(`take_key` · 0489 의 멀티탭·CLEAR 그대로)에 키를 넘기고, 글자가 바뀌었으면 `true` 다. 포커스 입력칸이 없거나 포커스를 잃었으면 종전처럼 `false` 다.

**왜**: 0502 가 «이름 입력 상자에서 숫자 멀티탭·OK·방향키 모두 글자 0 · stub_hits 에 XTextField 없음»(⒝ 엔진)으로 남겼다. 0489(XTextField.keyPressed)와 0410(TextComponentHandler + setTextComponent)이 둘 다 이 경로에 닿지 않았다.

**사용자 영향**: 이 게임에서 캐릭터 이름을 숫자 키로 넣을 수 있다(영문 대문자). 확인 키로 넘어가 이야기 → 거리 → 댄스 연습 화면까지 간다. 종전에는 상자에 글자가 안 들어가 시작할 수 없었다.

### 1. 어느 클래스가 그 상자를 그리나 (바이트코드 실측 · 스크래치 `javap`)

게임 jar 에는 `XTextField` 를 감싸는 클래스가 둘 있다.

- `n`: 키를 `XTextField.keyPressed` 로 넘긴다 — 0489 경로.
- ★`m`: 생성자가 `TextComponentHandler.getTextComponentHandler()` 를 받아 둔다. `keyPressed` 는 모드 키 130·194·195 만 거르고 나머지를 전부 `handler.keyPressed(key); pop` 한다. 모드 키 경로는 `getInputMode` 를 보며 `XTextField.setFocus(false)` → `XEventHandler.restoreDisplay()` → `setFocus(true)` 한다. `XTextField.keyPressed` 호출이 **0**, `setTextComponent` 호출도 **0** 이다.

실행 로그(`RUST_LOG=wie_skvm=debug` · 이름 경로)로 이 상자가 `m` 쪽인 것을 확인했다. `XTextField(<"">, 64, 0, canvas)` 1개 생성 · `setFocus(true)` · 이후 키 53 53 50 50 50 8 54 가 전부 `TextComponentHandler::keyPressed` 로만 왔다. 등록이 없으니 0410 처리기는 `false` 를 돌려줬고, 입력칸은 키를 한 번도 받지 못했다.

★실기에서는 포커스를 가진 XTextField 가 처리기의 입력 대상이라고 읽었다(이 게임은 처리기에만 키를 주므로 그렇지 않으면 이름을 넣을 길이 없다). XCE 문서는 저장소에 없다.

### 2. 다른 타이틀에 닿는가 (SKT 코퍼스 84 파일 · 안쪽 jar 클래스 단위 셈)

| 축 | 타이틀 |
|---|---|
| `XTextField` 사용 · `handler.keyPressed` 호출 | ★`14a62a8521a0` 만 |
| `XTextField` 사용 · `XTextField.keyPressed` 만 | 13종(`2d66945008c1` `2f5246006bd8` `31c90441f639` `47fe675bfffd` `640428a9cf9e` `8fec741a782d` `9a2cf5ffc9d3` `b42e4242866b` `ca1132f2e7bc` `ccb45e6b8d80` `ec2f8f2e02a2` `f2ae515201f2` `fb80e97cbc57`) |
| `handler.keyPressed` · `setTextComponent` 호출 · `XTextField` 미사용 | 8종(`85f03ca7389e` `2f84b8cc870d` `36c859b7df28` `9e2307e4fa88` `ccdd9295c5c4` `d1b55085bddc` `d6abd6258d08` `dae153ad28ba`) |

구조상 바뀌는 것은 첫 행뿐이다. 13종은 처리기 `keyPressed` 를 부르지 않으므로 키가 입력칸에 지금처럼 한 번만 간다. 처리기 쪽 타이틀은 등록이 있으면 종전 경로이고, 등록 전에는 포커스 XTextField 가 없어 `false` 다.

### 3. 전/후 (release `wie_validate` · origin/main `a3da330d` ↔ 이 브랜치 · 같은 시각 짝 · 동시 ≤ 3 · phase 마다 long 임대)

시작 시 `host-load-guard --status --recovered` rc=0. 실행 중 load1 은 15~37 이었다.

- 대상 `14a62a8521a0` `--keys`(시작 → 이야기 → 이름 상자 → 숫자 → OK):
  - 전: 30컷 내내 «이름 입력» 상자이고 글자는 0 이다. PASS 25/25 · 예외 9(RecordStoreNotFound option).
  - 후: `J` → `K` → `KA` → `KAA` → `KAB` → CLR `KA` → `KAM`(같은 키 1초 안 = 순환). OK 로 로딩 → 이야기 → 거리 → 댄스 연습 화면까지 간다. PASS 25/25 · 예외 9(같음) · stderr 새 오류 0.
  - 최종 커밋 바이너리로 다시 쟀다: `JAM` → 같은 경로 · PASS.
- `--inject` 27키(14종 = XTextField 13 + `85f03ca7389e`): 전/후 모두 **14/14 PASS 27/27** · 예외 수 14/14 같음 · rc 0.
  - 캡처 바이트 동일은 9종이 25~28/28 이다.
  - `fb80e97cbc57` 1/28 · `8fec741a782d` 10/28 · `2d66945008c1` 18/28 은 나란히 놓고 봤다. 같은 화면이 같은 순서로 나오고, 차이는 타자 연출과 팝업이 한 컷 앞뒤로 밀린 것이다(0489 §3 의 `fb80e97cbc57` 와 같은 서명).
- 이름 경로 레시피(`game_lab/recipes-progress/<sha>.keys` 의 첫 줄 · 비커밋):
  - `ec2f8f2e02a2` 25/27 동일. `2d66945008c1` 은 같은 경로이고 이름 칸 안내 문구의 깜박임 위상만 다르다. 둘 다 PASS · 예외 같음.
  - `f2ae515201f2` 는 같은 경로였다. paints 는 1876 → 3546 이었다. 짝을 두 번 더 쟀더니 3532·3566 / 3933·3959 였다. 첫 «전» 판이 굶은 것이고 회귀가 아니다.

### 4. 시험

- `handler_keys_reach_the_focused_field`(wie-skvm): 이 게임의 모양(`XTextField("", 64, 0)` · 키는 처리기에만)이다.
  - 포커스 전 `5` → `false`. 포커스 뒤 `5 5 2 CLEAR 6` → 전부 `true` · `KM` · canvas repaint 5회. `UP` → `false`. `setFocus(false)` 뒤 `2` → `false` · 글자 그대로.
- 개악 red: 처리기 폴백을 종전 `Ok(false)` 로 되돌리면 실패한다.
  - 같은 방식으로 `setFocus(false)` 가 정적 필드를 비우는 가지를 빼 봤더니 green 이었다. `take_key` 가 포커스를 이미 보기 때문이다. 그래서 그 가지는 지웠다.
- 기존 `registered_field_takes_multitap_and_digits` 의 «미등록 → false» 는 포커스 XTextField 가 없을 때의 단언으로 그대로 남았다.

### 5. 한계

- 영문 대문자·숫자만이다(0489·0410 과 같은 `wie_util::keypad`). 한글 조합·입력 모드 표시는 없다. `getInputMode` 는 0 그대로다.
- 진도 축(`--only progress` P/P2)은 재지 않았다. `compat.json` 의 `progress: stuck` 은 그대로 두었다. 이름 상자 뒤의 600초는 다음 진도 회차의 몫이다.
- `#`·`*` 는 0489 의 직접 경로와 같이 글자로 들어간다(32..=126 문자). 이 게임의 모드 키 130·194·195 는 `m` 이 먼저 거른다.

증적(캡처 띠 · 짝 비교표)은 저장소 밖 `~/orchestrator/reports/evidence/wie-skt-14a62a8521a0-name-box-no-input-adopt-p0/` 에 있다.

<!-- corpus-name-inflow v1 subjects=5 tree=d60ee0212749accb B=0/0 P=0/0 S=0/0 -->
