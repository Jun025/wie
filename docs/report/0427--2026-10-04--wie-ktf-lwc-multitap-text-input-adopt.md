## [2026-10-04] KTF lwc 글자 입력기 — 이름 칸에 한글·영문을 넣는다 · 자판 하나를 세 입력기가 같이 쓴다 (wie-ktf-lwc-multitap-text-input-adopt)

**무엇을**: 숫자 키를 글자로 바꾸는 자판을 `wie-util::keypad` 하나로 만들었다. 같은 키를 다시 누르면 글자가 바뀌는 영문 멀티탭과 천지인 한글 조합이 들어 있다. lwc `TextComponent.keyNotify`, lcdui `InputMethodHandler.notifyKeyInput`, SKVM `TextComponentHandler.keyPressed` 셋이 모두 이 자판을 쓴다. 입력 모드 3 은 한글이다. 새 입력기는 한글 모드로 시작한다.
**왜**: `9789fec50f39` 의 «시장님 존함» 칸은 숫자를 넣을 때마다 지웠다(0425 §2). 원인은 «글자 입력기가 없다»만이 아니었다. 게임은 이 칸에 `setCurrentMode(3)` 을 부른다. 그런데 wie 는 3 을 숫자 모드로 다뤘다. 한글을 넣자 게임은 지우지 않았고 이름을 받아 다음 대사로 넘어갔다(§1).
**사용자 영향**: `9789fec50f39` 는 이름 칸에 한글 이름을 넣고 본편 대사로 넘어간다. `0c67145b11df` 는 고치와 본인 이름에 한글이 들어간다(종전에는 숫자뿐이었다). `*` 로 영문 대문자 → 소문자 → 숫자 → 한글 순서로 바뀐다. 글자 입력을 쓰는 다른 KTF 게임도 기본 입력이 한글이 된다(§4).

증적: `~/orchestrator/reports/evidence/wie-ktf-lwc-multitap-text-input-adopt/`. 타이틀은 sha12 로만 적는다.

### 1. 재현 — 숫자를 지운 이유는 «모드 3»이었다

- `9789fec50f39`(release `wie_validate --inject --keys`): 이름 칸은 `TextFieldComponent(null, 0)` 이다. 만든 직후 게임이 `InputMethodHandler.setCurrentMode(3)` 을 부른다. 이 칸에서 숫자 키 7회를 넣었다. 7회 모두 그 뒤에 `setString("")` 이 왔다. 계측 로그로 `getString` 이 `"4"` 를 돌려준 직후 지우는 것을 확인했다.
- 같은 칸에 3 = 한글로 바꾼 빌드를 넣었다. `getString` 이 `ㄱ → 기 → 가 → 강 → 감 → 감ㅅ → 감시` 로 바뀌었다. `setString("")` 은 0회였다. OK 를 누르면 «감시입니다. 4년의 임기동안 최선을…» 대사가 나오고 다음 질문(«출신학교»)으로 넘어간다.
- 「12 오는 중~」(`0c67145b11df`) 도 같은 경로다. 키는 셸을 거쳐 `TextComponent.keyNotify(1, 53)` 으로 들어온다. 이 칸은 `TextBoxComponent(…, 0)` 이고, 이 게임은 모드를 정하지 않는다.
- 모드 3 을 쓰는 게임이 하나 더 있다. `b22a7fcfb406` 은 이름 입력 화면에서 `setCurrentMode(0)` 다음 곧바로 `setCurrentMode(3)` 을 부른다. 게임이 직접 그리는 모드 표시줄 «가 A a 1» 에서 **가** 에 불이 켜진다(증적 `b22a-mode-strip.png`).

### 2. 고른 규칙과 출처 — 근거가 없는 것은 «가정»이다

| 항목 | 고른 값 | 근거 |
|---|---|---|
| 모드 3 | 한글 | **실측** — §1 의 두 게임 |
| 모드 0 · 1 · 2 | 영대 · 영소 · 숫자 | **가정**. `b22a7fcfb406` 표시줄의 순서 «가 A a 1» 을 `changeCurrentModeToNext` 가 3 → 0 → 1 → 2 → 3 으로 돈다고 놓았다 |
| 시작 모드 | 한글(3) | **가정** — 단말은 한글로 시작한다고 보았다. 모드를 정하는 두 게임은 결과가 같다 |
| 숫자 제한자 1 · 2 · 5 | 숫자 그대로 | javadoc(`TextComponent` CONSTRAINT_*) · 종전 코드와 같다 |
| 한글 배열 | 천지인: 1 ㅣ · 2 ㆍ · 3 ㅡ 로 모음을 획으로 만든다. 4 ㄱㅋㄲ · 5 ㄴㄹ · 6 ㄷㅌㄸ · 7 ㅂㅍㅃ · 8 ㅅㅎㅆ · 9 ㅈㅊㅉ · 0 ㅇㅁ 는 다시 누르면 바뀐다 | **가정**. 이 repo 에는 배열 문서가 없다. 휴대폰에서 널리 쓰인 천지인을 따랐다. KTF 단말이 무엇을 썼는지는 모른다 |
| 영문 배열 | ITU-T E.161, 마지막에 숫자(2 → A B C 2) | SKVM 입력기(0410)가 이미 쓰던 표다. 그 표를 공용으로 옮겼다 |
| 모드 키 | lwc 칸의 `*` | **가정**. lcdui `notifyKeyInput` 은 `*` 를 종전처럼 먹지 않는다. `b22a7fcfb406` 은 `*` 를 넘기지만, 게임이 자기 표시줄을 따로 그리므로 건드리지 않았다 |
| CLR | 조합 중이면 마지막 키(획·자모·글자)를 되돌린다. 아니면 한 글자를 지운다 | 천지인의 획 단위 지우기를 따랐다(**가정**) |
| 멀티탭 시간창 | 1초 | 종전 두 입력기의 값 그대로 |

- `docs/reference/AromaWIPI_classes.zip` 의 `InputMethodHandler`·`TextComponent` 를 직접 열어 봤다. 필드와 메서드 이름은 있다(`processInputKO`, `currentMode`, `inputLanguages`). 그러나 **메서드 본문이 모두 0 바이트(nop)로 비어 있다.** 모드 번호는 네이티브 `getSurpportModes` 에 맡겨져 있다. 그래서 여기서 번호를 얻을 수 없었다.
- 조합 방식: 키마다 그동안 누른 키 전체를 다시 그리고, 바뀐 꼬리만 고친다. 그래서 «각 + ㅏ → 가가»처럼 받침이 다음 글자로 넘어가는 경우도 따로 처리하지 않아도 된다. 한 키가 «한 글자 바꾸기 + 한 글자 넣기»가 될 수 있다. lcdui 리스너에는 `notifyTextChanged` 를 글자마다 한 번씩 부른다.
- lwc 칸은 `m_td` 를 읽을 수 있다. 칸의 글이 조합 중인 글로 끝나지 않으면 조합을 버리고 새로 시작한다. 게임이 `setString` 으로 글을 바꾼 경우다(시험 단언 있음).

### 3. 공유 — 두 구현을 남기지 않았다

- 자판은 `wie-util/src/keypad.rs` 에 있다. JVM 을 모르는 순수 함수(`press`·`back`·`render`)다. 결과 `Edit` 를 `Op::{Insert, Replace, Delete}` 로 풀면 lcdui 리스너와 SKVM `TextComponent` 가 받는 모양이 된다.
- 종전에는 표가 둘이었다. lcdui 는 `" "·".,-"·"ABC"…`, SKVM 은 E.161 이었다. 이제 하나다. lcdui 영문 모드는 숫자까지 돈다(ABC2). 그 밖의 동작은 같다.
- SKVM 은 영대문자 모드로 고정했다. 이 핸들러에 글자를 넣는 게임은 `85f03ca7389e` 하나이고 영문 이름을 받는다. 한글 모드를 켤 근거가 없다(`ponytail:` 주석).
- lwc `TextComponent` 에는 필드를 더하지 않았다. 게스트 하위 클래스가 이 클래스의 필드를 오프셋으로 읽는다(0425 §4). 조합 상태는 이미 있던 `imHandler`(`InputMethodHandler`)에 둔다. 단말의 구조도 javadoc 이 말하는 것과 같다. `TextFieldComponent`·`TextBoxComponent` 생성자는 받은 제한자로 그 핸들러를 만든다.

### 4. 전/후

바이너리: 전 = `origin/main 0e9fbaeb` release, 후 = 이 브랜치 release. 전·후를 같은 시각에 짝으로 돌렸다(동시 2개). 짝마다 `host-load-guard --status --recovered` rc=0 을 기다린 뒤 시작했다(증적 `pair/guard.log`).

**글자 입력이 있는 타이틀 62종 · 기본 27키 `--inject`**(말뭉치에서 `TextFieldComponent`·`TextBoxComponent`·`InputMethodHandler`·`TextComponentHandler` 문자열이 나오는 전부):

| | 전 | 후 |
|---|---|---|
| PASS · UNMEASURED · FAIL | 27 · 34 · 1 | 27 · 34 · 1 |
| 판정이 바뀐 행 | — | **0** |
| 기본 27키로 글자 입력에 닿은 타이틀 | 3 | 3 |

- UNMEASURED 34 는 `max-ticks` 에 닿은 것이다. 전·후 조건이 같다(0425 §6 과 같다). 그중 6종은 받은 키 수가 후에서 1 적다(`36b82cb67723` 16 → 15 등). 6종 모두 글자 입력에 닿지 않았다. 동시에 띄운 두 실행의 tick 경합이다. 이 변경이 바꿀 수 있는 경로가 아니다.
- 글자 입력에 닿은 3종:
  - `9789fec50f39`: PASS · paints 190 → 193 · 예외 2 → 2(«image is null» · 전·후 같다). 이 칸에 닿는 글자 키는 `*` 와 `1` 뿐이다. 전은 `1` 이 숫자로 들어가 지워진다. 후는 `*` 가 영문으로 바꾸고 `1` 이 «.» 으로 들어가 역시 지워진다. 이 칸은 한글만 남긴다.
  - `0c67145b11df`: 이름 칸에 «5» 대신 «ㄴ» 이 들어간다(화면 확인). 판정·paints 같다.
  - `b22a7fcfb406`: `notifyKeyInput` 로 `1` 이 들어간다. 전 «.» → 후 «ㅣ». 마지막 화면 색 수 469 → 487.

**이름 경로 `--keys`**(증적 `name2.keys`·`c.keys`):

| sha12 | 전 | 후 |
|---|---|---|
| `9789fec50f39` `4 2 3 0 0 8 1` 뒤 OK×5 | 칸이 빈 채로 남는다 · 마지막 화면 = 이름 칸 | 칸에 «감시» → «감시입니다…» → 다음 질문 «출신학교» |
| `0c67145b11df` 두 칸 입력 뒤 DOWN×3 OK | 칸 «423991»·«001551» → 게임 안 | 칸 «고치»·«미리» → 게임 안 · 예외 0 → 0 |
| `85f03ca7389e`(SKVM · 0410 의 키) | PASS · paints 391 · 마지막 색 139 | 같다 · 이름 입력 화면 md5 같다 |

- `1e43e2e0055f`(가게 이름)와 `d448aee68157`(모드 함수)은 기본 키로 입력 화면에 닿지 않았다. 이번 회차는 그 경로를 다시 재지 않았다. 두 게임의 글자는 이제 영문 대신 한글로 들어간다. `1e43e2e0055f` 의 조건(«한 글자 이상»)은 한글도 채운다. `d448aee68157` 은 `getCurrentInputMode` 가 처음에 0 대신 3 을 받는다. 화면 확인은 하지 않았다.

### 5. 시험

- `wie-util` `keypad` 4건: 영문 순환·숫자 되감기 · 천지인 조합(받침 이동 «가기»·«가거», 겹받침 «낤», «와», «ㄸ» 은 받침이 되지 않음, 반쯤 친 모음 «ㄱㆍ») · CLR 되돌리기 · `Edit` → `Op` 변환.
- `wie-wipi-java`: `keys_reach_the_listener_as_text`(모드 3 한글 «가거» — 한 키가 바꾸기 + 넣기 · 시작 모드 3 · 숫자 모드) · `keys_compose_hangul_and_a_rewritten_text_starts_a_new_word`(`setString("")` 뒤 새 조합 · 숫자 제한자) · 셸 시험 `shell_walks_focus…`(«가A» — 한글, `*` 뒤 영문).
- 되돌리면 red(실측, 4/4): `m_td` 꼬리 확인 끔 → FAILED · 모드 표를 종전 순서로 → FAILED · `TextFieldComponent` 제한자 버림 → FAILED · 시작 모드 한글 삭제 → FAILED.
- 네 관문: `cargo fmt --check` · `clippy --all -D warnings` · wasm32 clippy · `cargo +beta clippy` 모두 rc=0. `RUST_MIN_STACK=4194304 cargo test --all` 677 통과 0 실패.
- 러너 블록: draw · helloworld ×2 · text PASS. keydraw ktf/lgt `--inject --expect-last-frame` PASS 27/27 · rc=0(paints 80 · 55).

### 6. 범위 밖으로 둔 것

- 글자 수 제한: `setMaxLength` 는 여전히 값만 기록한다(javadoc 은 넘치면 입력을 무시한다고 한다). 숫자만 넣던 종전과 같게 두었다.
- 띄어쓰기·기호 카드·`#`·커서 이동(좌우): 측정된 화면이 필요로 하지 않는다. 영문 모드의 `0` 이 띄어쓰기다.
- `TextBoxComponent(String, I, I)` 의 인자 순서는 아직 확인되지 않았다. 제한자가 이제 모드를 고르므로, 순서가 반대라면 «글자 칸에 숫자»로 드러난다. 이 생성자를 쓰는 게임에 글자를 넣어 본 적은 없다.

### 7. 측정 조건

- 짝 62쌍은 `host-load-guard` rc=0 일 때만 시작했다(23:52 ~ 00:08 · load1 11~17). 이름 경로 짝도 rc=0 에서 돌렸다.
- census 락은 다른 레인이 쥐고 있었다(21:44 부터). `playability-census.mjs` 는 쓰지 않았다. `wie_validate` 를 `build-slot` 경유로 직접 돌렸고, 동시 실행은 2개다.
- `nohup … &` 는 쓰지 않았다. 회차 끝에 자기 프로세스는 0 이다.

### 8. compat.json

바꾸지 않았다. 진도 축(600초)은 다시 재지 않았다. `9789fec50f39` 는 이름 칸을 넘지만, 다음 질문도 글자 입력이다. 진도 정책 키가 그 칸에 이름을 넣는지는 재지 않았다. 게임별 `changes` 는 `docs/player-updates/2026-10-04-name-input-letters.json` 에서 파생된다.

### 9. 게임 이름 유입

`node scripts/corpus-name-inflow.mjs`(14파일): 유입 5건(BOUNDED 5회/5쌍) · 판단 필요 1건(SUFFIX-ATTACHED 1회/1쌍). **이 diff 가 새로 들인 이름은 0이다.** 6건 모두 이 diff 가 건드린 파일(`shell_card.rs`·`text_box_component.rs`·`text_component.rs`)에 원래 있던 주석의 이름이다. SUFFIX-ATTACHED 1건은 같은 주석의 더 긴 제목이다. `git diff origin/main` 의 `+` 줄에서 코퍼스 제목과 맞는 것은 0건이다. 새 주석·문서·소식은 sha12 만 쓴다.
