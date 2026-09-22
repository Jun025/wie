## [2026-09-22] 없는 플랫폼 클래스 2종을 만들었다 — 4타이틀의 `NoClassDefFoundError` 가 닫혔다, `wec/SYSTheme` 는 «짓지 않았다» (wie-aot-java-missing-platform-classes-dialogcomponent-cluster)

### 무엇을

LGT AOT-Java 타이틀이 부팅 첫 tick 에서 `NoClassDefFoundError` 로 죽던 **없는 클래스 2종**을 만들었다.
파일 5개 · 본문 +262 / −4. 세 번째 클래스 `wec/SYSTheme` 는 **소유 판정만 하고 만들지 않았다** — 사유는 아래 별 절.

| 파일 | 추가 |
|---|---|
| `org/kwis/msp/lwc/dialog_component.rs` (신규) | `<init>(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V` · `setButtonString(ILjava/lang/String;)V` · `doModal()I` · 필드 `type I` + 시험 2 |
| `org/kwis/msf/io/message.rs` (신규) | `<init>(Ljava/lang/String;[B)V` · `getData()[B` · 필드 `addr`·`data` + 시험 1 |
| `lwc.rs` · `msf/io.rs` · `lib.rs` | 모듈 선언 · 프로토 등재(`get_protos` **49 → 51**) |

### 왜 — 시그니처를 «게임에게 묻고, 그다음 권위 문서로 확인했다»

0208 은 게임의 `binary.mod` 참조 풀 **하나**로 시그니처를 골랐다. 이 회차는 축을 **하나 더** 썼다:
`docs/reference/AromaWIPI_classes.zip`(cp56 · 권위 LGT ez-i API)에 **두 클래스가 다 실재**하고,
그 `.class` 의 상수풀을 직접 파싱해 **게임이 부른 서술자와 한 글자씩 대조**했다. 전건 일치다.

| 만든 것 | 게임 참조 풀 (출처) | AromaWIPI 권위 클래스 | 일치 |
|---|---|---|---|
| `DialogComponent.<init>(Component;String;I)V` | 붕어빵타이쿤3 `@462754` · 당신은골프왕 `@501946` · 슈퍼액션히어로 `@304493` (**3타이틀 전건**) | `<init>(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V` PUBLIC | ★ |
| `DialogComponent.setButtonString(ILjava/lang/String;)V` | 당신은골프왕 `@501997`+`@502020` · 슈퍼액션히어로 `@304544`+`@304567` (**2타이틀**) | `setButtonString(ILjava/lang/String;)V` PUBLIC | ★ |
| `DialogComponent.doModal()I` | 3타이틀 전건(`@462805`·`@502036`·`@304583`) — ★**서술자는 풀에 인접하지 않다**(공용 서술자라 다른 자리에 interning) | `doModal()I` PUBLIC | ★ 권위 문서가 `()I` 를 확정했다 |
| `Message.<init>(Ljava/lang/String;[B)V` | 간호사타이쿤2 `@1529392` | `<init>(Ljava/lang/String;[B)V` PUBLIC | ★ |
| `Message.getData()[B` | 간호사타이쿤2 `@1529416`+`@1529421` | `getData()[B` PUBLIC | ★ |

★**상속 관계도 추측이 아니다**: AromaWIPI javadoc 이 `Object → Component → ContainerComponent →
ShellComponent → DialogComponent` 를 명시한다. `parent_class` 는 그대로 `ShellComponent` 다.

★**`doModal` 의 «반환 상수»도 지어내지 않았다.** `DialogComponent.class` 의 `ConstantValue` 속성을
직접 읽었다: `TYPE_NONE 0 · TYPE_OK 1 · TYPE_OK_CANCEL 2 · DLG_TIMEOUT 10 · DLG_OK 11 · DLG_CANCEL 12 ·
OK_BUTTON 20 · CANCEL_BUTTON 21 · TIMEOUT_INFINITE -1`. ★**`TYPE_*` 와 `DLG_*` 는 번호대가 다르다** —
둘을 같은 수로 가정했으면 틀렸다.

### 측정 — 벽이 어디로 옮겨 갔나 (release `wie_validate` · `--inject` 기본 예산 · **타이틀당 3회**)

★**«전» 열은 이 회차가 다시 재지 않았다 — `docs/report/0210` 의 재기준선 값이다**(같은 release 구성 ·
타이틀당 3~4회). 이 회차가 그 시점의 바이너리를 갖고 있지 않아서이고, 흐리지 않고 적어 둔다.
★**그러나 인과는 «전» 열에 기대지 않는다** — `NoClassDefFoundError: X` 는 X 가 없을 때만 나오고,
내가 한 일은 정확히 X 를 등재한 것이다.

| 타이틀 | ticks 전→후 | paints 전→후 | 닫은 벽 (전) | **멈춘 자리 (후) — 문면 그대로** |
|---|---|---|---|---|
| **붕어빵타이쿤3** | 1 → **1** | 0 → 0 | `NoClassDefFoundError: org/kwis/msp/lwc/DialogComponent` | `net.wie.WieError: Invalid memory access; address: 0` (3/3 동일 · `R0: 0x48845100`) |
| **당신은골프왕** | 1 → **1** | 0 → 0 | 같은 벽 | `net.wie.WieError: Invalid memory access; address: 0` (3/3 · `R0: 0x48844df0`) |
| ★**슈퍼액션히어로** | 1 → ★**14,545,817 ~ 19,196,881** | 0 → ★**2~3** | 같은 벽 | `only blank/uniform frames (black screen)` (3/3 · `distinct_colors 1`) |
| **간호사타이쿤2** | 2 → **2** | 0 → 0 | `NoClassDefFoundError: org/kwis/msf/io/Message` | `Unable to resolve non-virtual method org/kwis/msp/lcdui/Jlet.getCurrentJlet()Lorg/kwis/msp/lcdui/Jlet;` (3/3) |
| 월드장기체스 | 1 → 1 | 0 → 0 | — (손대지 않음) | `java.lang.NoClassDefFoundError: wec/SYSTheme` (3/3 · **불변**) |

★★**슈퍼액션히어로가 이 회차의 실제 산출이다** — 부팅 tick 1 에서 죽던 것이 **1,450만~1,920만 ticks**
를 돌며 **페인트를 2~3회** 한다. ★**그래도 `distinct_colors 1` = 균일색이라 «렌더한다»고 쓰지 않는다.**
벽이 «치명 오류»에서 «빈 프레임 루프»로 계급을 바꾼 것이고, 그 이상은 재지 않았다.

★**나머지 3종은 `paints 0 → 0` 이다.** 0208 과 같은 정직함을 유지한다 — **이 회차도 렌더를 켜지 않았다.**
붕어빵타이쿤3·당신은골프왕은 티켓이 «거기서 멈춰라»고 지정한 `Invalid memory access` 계급으로
옮겨 갔으므로 **멈췄다**(P0 별 티켓 `wie-aot-java-invalid-memory-access-address-zero-cluster` 소관).

★★**한 가지를 숨기지 않는다 — 세 타이틀 중 «어느 것도 DialogComponent 를 실제로 만들지 않는다».**
`RUST_LOG=warn` 으로 3타이틀을 돌려 내 `tracing::warn!` 을 센 값이 **전건 0줄**이다. ⇒ ★**닫힌 벽은
«사용»이 아니라 «해결(resolution)»이었다** — 클래스가 링크 시점에 풀리지 않아 죽던 것이다.
⇒ ★**그러므로 `doModal` 의 반환값 선택은 «오늘 관측 불가»다**(0208 이 `TextBoxComponent` 의 두 int
순서에 대해 적은 것과 같은 계급의 한계다). 그대로 적어 둔다.

### ★★`wec/SYSTheme` 소유 판정 — «게임 것이 아니다. 그러나 짓지 않았다»

티켓이 「판정 없이 만들지 마라」고 한 자리다. **판정 = 단말 OEM(벤더) 확장 API이지 게임 클래스가 아니다.**
근거 다섯, 전부 실측이다:

1. ★**권위 API 에 없다.** `AromaWIPI_classes.zip`·`AromaWIPI_javadoc.zip` 에서 `SYSTheme` **0건** ·
   `wec` **0건**. 반면 `DialogComponent`·`msf/io/Message` 는 **둘 다 실재**했다(위 표).
2. ★**벤더 SDK 어디에도 없다.** `game_lab/vendor_sdk/`(LGT MIDP Emulator · AromaWIPI Emulator ·
   SK-VM ez-i 에뮬레이터 · BizHawk) 전수 **이진 안전 검색** → `SYSTheme` **0건** · `wec/OEMDevice` **0건**.
3. ★★**게임 자신의 클래스가 아니다 — 이것이 결정타다.** `binary.mod` 문자열 풀은 두 구역으로 갈린다.
   월드장기체스의 «외부 참조» 구역은 `org/kwis/msp/lcdui/Jlet @417394` 에서 끝나고, **게임 자신의 클래스**는
   `CCC @417418`(= `app_info` 의 `MClass:CCC`)부터 `bb`·`af`·`ai`·`n`·`c`·`k`·`f`·`b`·`g` 로 이어진다 —
   ★**전부 «패키지 없는» 난독화 이름**이다. `wec/OEMDevice @416955`·`wec/SYSTheme @417047` 은
   그 **앞**, 즉 `java/lang/System`·`org/kwis/msp/lcdui/Display` 와 **같은 외부 참조 구역**에 있다.
   ★**코퍼스 46종 전수로 확인했다 — 자기 클래스를 Java 패키지에 두는 게임이 «0»이다**
   (간호사타이쿤2 도 `NurseTycoon2`·`MyInfo`·`Rival`·`MainMenu` 처럼 패키지가 없다).
   ⇒ **패키지가 붙은 이름은 예외 없이 «단말이 주는 것»이고, `wec/*` 가 그 자리에 있다.**
4. ★**API 모양이 «단말 설정»이다**: `wec/OEMDevice.getSYSTheme()Lwec/SYSTheme;` ·
   `SYSTheme.saveItem(IILjava/lang/String;[BI)I` · 정적 `ITEM_GROUP_SOUND_DEFAULT`·`FORMAT_SOUND_MA3`
   (MA-3 = Yamaha SMAF 오디오 포맷). 「폰의 시스템 테마에 사운드 항목을 저장한다」는 **기기 기능**이지
   게임 로직이 아니다.
5. 코퍼스 46종 중 **고유 1타이틀**만 참조한다(2파일 = 같은 sha256 `70d709c4`).

⇒ ★★**그러므로 P0 `LgtClassLoader` 축(클래스 로딩·경로 문제)의 소관이 «아니다»** — 게임이 자기 jar 에
넣어 왔어야 하는 것이 아니기 때문이다. 이 「없는 플랫폼 클래스」 계열이 맞다.

★★**그런데도 만들지 않았다 — 사유 넷.** 이 저장소가 `pause(Clip)Z` 를 기각한 그 규율의 연장이다:

- ⒜**권위 서명 출처가 «없다»**(근거 1·2). 한 게임의 참조 풀 말고는 대조할 문서가 없다.
- ⒝★**두 정적 필드의 «타입도 값도» 복원 불가다.** 전 토큰(1글자 포함)으로 그 블록을 다시 떠도
  `ITEM_GROUP_SOUND_DEFAULT`·`FORMAT_SOUND_MA3` 에 **인접 서술자가 없다**. 그리고 이름이 풀에 남아
  있다는 것은 **javac 가 인라인하지 않았다**는 뜻이라 게임이 런타임에 읽는다 ⇒ 값을 지어내면
  게임이 그 값으로 분기한다. ★`DialogComponent` 쪽은 정반대였다 — 필드 이름이 풀에 **0건**이라
  («`static final int` 는 인라인된다») 권위 문서의 상수를 쓰면 됐다.
- ⒞**한 클래스로 끝나지 않는다** — `getSYSTheme` 는 `wec/OEMDevice` 의 메서드이고 **그 클래스도 없다**
  (트리 전수 `OEMDevice` **0건**). 즉 **두 번째 클래스까지 발명**해야 도달한다.
- ⒟`saveItem` 의 반환 `I` 가 성공코드인지 항목 id 인지도 모른다.

⇒ **월드장기체스의 벽은 «불변»이고**, 그것이 이 회차의 정직한 산출이다.

### ★넣지 «않은» 것과 그 이유

- ★**`DialogComponent` 의 나머지 공개 멤버 전부.** 권위 클래스는 생성자 **3개**와
  `setType`·`setTimeout`·`getTimeout`·`getActionState`·`show`·`layout`·`paintFrame` 을 갖지만,
  코퍼스 46종 어디에서도 **참조가 0**이다. 상수 `DLG_CANCEL`·`OK_BUTTON` 등도 필드로 만들지 않았다
  (게임이 인라인해 들고 있어 **이름이 풀에 나타나지 않는다** ⇒ 필드로 둘 이유가 없다).
- ★**`Message` 의 나머지 17개 접근자**(`getAddress`·`getLength`·`getDate` …)와 생성자 2개. 같은 이유다.
- ★**`Component.getWidth` 류의 «대칭» 추가를 하지 않았다.** 0208 은 그 한 건만 근거 등급이 달랐다고
  적었는데, 이 회차는 **전 항목이 «런타임이 문면으로 지목했거나 참조 풀에 실재»** 다.
- ★**`Message.addr` 필드는 예외이니 등급을 밝힌다** — 읽는 타이틀이 **없다**. 그런데도 둔 것은
  «생성자가 받은 인자가 갈 자리»여서이고(권위 클래스의 같은 이름 필드), 빼면 주소를 **조용히 버린다**.
  `data` 는 `getData` 가 읽으므로 근거가 다르다.
- ★**`Jlet.getCurrentJlet()` 을 «이 회차에서» 만들지 않았다** — 아래 새 벽 절.

### ★옮겨 간 새 벽 — 그대로 적는다

- **붕어빵타이쿤3 · 당신은골프왕** → `net.wie.WieError: Invalid memory access; address: 0`.
  ★티켓이 지정한 정지선이다. **손대지 않았고 원인 조사도 하지 않았다.**
- **간호사타이쿤2** → `Unable to resolve non-virtual method org/kwis/msp/lcdui/Jlet.getCurrentJlet()Lorg/kwis/msp/lcdui/Jlet;`
  ★**계급이 다르다 — 「없는 클래스」가 아니라 「있는 클래스의 없는 메서드」다.**
  실측: 트리 전체에 `getCurrentJlet` **0건**이고, `Jlet` 은 `getActiveJlet()`(static)만 갖는다.
  간호사타이쿤2 참조 풀에도 `()Lorg/kwis/msp/lcdui/Jlet;`+`getCurrentJlet` 이 실재한다(`@1529219`·`@1529247`).
  ★**AromaWIPI 에 `Jlet` 이 «없어»**(ez-i 생명주기 클래스라 그 API 집합 밖) 권위 대조가 안 된다 ⇒
  「`getActiveJlet` 의 별명인가」는 **재지 않았다.** 후속 몫으로 넘긴다.
- **슈퍼액션히어로** → `only blank/uniform frames (black screen)`, `ticks` 1,450만~1,920만 · `paints` 2~3.

### 회귀 — 없다. 단 «검사 범위»를 정확히 적는다

★**이 변경은 `get_protos()` 에 클래스를 «더하기»만 한다** — 기존 클래스는 한 줄도 건드리지 않았다.
그러나 그 목록은 **KTF·SKT·LGT 가 공유**하므로 범위를 실측했다.

- `broken/lgt` **46종 전수**(압축 해제 후 검색): 두 클래스를 참조하는 것은 **대상 5파일뿐**.
- ★**`working/` 코퍼스 298파일 · 내부 엔트리 146,179개를 압축 해제해** 검색 → `DialogComponent` 참조
  **9타이틀**(전부 KTF · 전건 baseline `PASS`). ★**첫 시도는 «무효»였다** — 압축된 zip 을 날바이트로
  훑어 `0건` 이 나왔다. 그 0 은 의미가 없어 폐기하고 해제 검색으로 다시 했다.
- 그 **9종 전건 재실행**: `스맥vs로우` · `kt 졸라맨액션학원` · `미니고치` · `정무문2` ·
  `프린세스메이커4` · `졸라맨액션학원` · `(KTF)정무문2` · `소울카드마스터2` **8종 PASS**.
- ★**`얼음낚시` 1종이 FAIL 했다 — 조사했고, 회귀가 아니다.**
  ⑴3회 재실행 전건 `no frame rendered (hang/black screen)` · `ticks 74~87`.
  ⑵★**`RUST_LOG=warn` 으로 두 신규 클래스의 등장 횟수를 셌다 — `DialogComponent` 0줄 · `Message` 0줄.**
  ★**즉 이 타이틀은 두 클래스를 런타임에 «건드리지도 않는다»** ⇒ 내 변경이 원인일 수 없다.
  (그 로그가 보여 준 실제 모습은 `wie_midp` 의 `Font` 스텁 루프 = MIDP 경로 타이틀이다.)
  ⑶★**예산을 늘리니 PASS 한다** — `--timeout 60` → **PASS · 273 paints · 283색**, `--timeout 120` →
  **PASS · 657 paints**. ⇒ 기본 20초 예산에 비해 «느린 타이틀»이지 회귀가 아니다.
  ★**티켓의 측정 위생 조항(「FAIL 이면 예산을 늘려 한 번 더」)이 정확히 이 자리에서 값을 했다.**
  ※★**한계**: 「변경 전 바이너리로 재현」(AGENTS.md 4단계의 ⑶)은 **하지 않았다** — 이 회차가 그
  바이너리를 갖고 있지 않다. ⑵의 「경로를 타지 않는다」가 그보다 강한 증거라고 판단했고, 그 판단을 적어 둔다.

### 시험 — 「있다」가 아니라 「지킨다」를 확인했다

새 시험 3개(`dialog_component.rs` 2 · `message.rs` 1). ★**변이 시험**: `get_protos()` 에서 두 프로토
등재를 지우고(`51 → 49`) 돌리니 ★**정확히 그 3건만 FAILED, 나머지 30건은 ok**.
⇒ 기존 시험만으로는 이 결손이 잡히지 않았다는 뜻이다. 확인 후 원복했다.

- `do_modal_returns_timeout_only_for_the_button_less_type` — TYPE_NONE 은 버튼이 없어 `DLG_TIMEOUT`
  말고 다른 결말이 없다. 나머지 두 타입이 거기로 무너지지 않는 것까지 함께 잠근다.
- `dialog_component_is_a_component` — 부모 사슬이 깨지면 게스트가 넘겨 다니는 `Component` 로 취급되지 않는다.
- `get_data_returns_the_payload_the_constructor_was_given` — `getData` 가 null 을 주면 호출자는
  «빈 메시지»로 오독한다.

### 게이트

`cargo fmt --all -- --check` **OK** · `cargo clippy --all -- -D warnings` **OK** ·
`cargo clippy --target wasm32-unknown-unknown -- -D warnings` **OK** ·
`RUST_MIN_STACK=4194304 cargo test --all` ★**rc=0 · `test result: ok` 46줄 · 0 failed** ·
`cargo +beta clippy --all -- -D warnings` **OK**(beta 재설치 후).

★★**wasm 게이트의 사각을 이번에도 밟았다 — 0208·0210 이 적은 그대로다.** 문서대로의 맨 명령은
기본 멤버만 봐서 **3.40초**에 끝났다(내 변경을 **보지 못한다**). 그래서 상위집합으로 따로 돌렸다:
`cargo clippy --target wasm32-unknown-unknown -p wie-wipi-java -p wie-lgt -p wie-ktf -p wie-web -p wie_featurephone -- -D warnings`
→ **OK**(4m51s · 네 크레이트가 실제로 `Checking` 되는 것을 출력으로 확인).

★★**그리고 «게이트가 통과했다»를 한 번 잘못 읽었다 — 적어 둔다.** 첫 `cargo test --all` 을
`| tail -60` 으로 파이프해 놓고 그 **`tail` 의 exit 0** 을 성공으로 읽었다. 실제로는
`dialog_component.rs` 가 **컴파일 에러**(`as_ref().unwrap()` — `ClassInstanceRef::as_ref` 는
`Option` 이 아니라 `&dyn ClassInstance` 를 준다)였다. ⇒ ★**파이프 뒤의 rc 는 마지막 명령의 것이다.
게이트는 파이프하지 말고 rc 를 직접 받아라.**

### ★AGENTS.md 러너 블록 — 엔진 변경이라 전건 돌렸다

`node scripts/make-draw-fixture.mjs` → `draw_j2me.jar`·`text_j2me.jar` 재생성.

| 픽스처 | 결과 |
|---|---|
| `draw_j2me.jar` | PASS · ticks 3,195,939 · paints 1 |
| `helloworld_ktf.zip` | PASS |
| `helloworld_lgt.zip` | PASS |
| `keydraw_ktf.zip` `--inject --expect-last-frame` | ★**PASS · paints 54 · content true · rc=0** |
| `keydraw_lgt.zip` `--inject --expect-last-frame` | ★**PASS · paints 55 · content true · rc=0** |
| `text_j2me.jar --timeout 5` | PASS |

★`keydraw_*` 의 54·55 는 AGENTS.md 가 적은 **유휴 범위 48~55** 안이다. ★**그 수는 바닥이지 등식이
아니다** — 판정은 `PASS`·`content true`·rc=0 이고, 그 셋이 전건 섰다.

저장소 검사기: `check-docs-report-serial --next-serial` → **0214**(디스크 0214 · 열린 PR claim
`0116(#162)`·`0137(#180)` 회피).

### 한계 — 숨기지 않는다

- ★**「전」 열은 이 회차의 재측이 아니다**(0210 값). 위 측정 절에 이유를 적었다.
- ★**`doModal` 의 반환값은 오늘 관측할 수 없다** — 세 타이틀 중 누구도 다이얼로그를 만들지 않는다.
  TYPE_NONE→`DLG_TIMEOUT` 은 정의상 정확하지만, 버튼 타입에서 `DLG_OK` 를 고른 것은 **선택**이고
  그 대가(「사용자가 누르지 않았는데 눌렀다고 읽는다」)를 소스 주석에 적어 두었다.
- ★**`Message.addr` 는 읽는 타이틀이 없다**(위 «넣지 않은 것» 절의 등급 표기).
- ★**`얼음낚시` 는 «변경 전 바이너리»로 교차 확인하지 않았다**(위 회귀 절 ※).
- ★**부하가 높았다 — `uptime` load1 은 회차 내내 33.65 ~ 144.28 이었다**(최종 측정 시각 04:34 KST 에 **33.65**).
  CPU 는 같은 구간에 **53.5% idle**(load 119 시점) — 즉 I/O 포화이지 CPU 포화가 아니다.
  대상 5타이틀은 전건 **3회 반복**했고 판정·문면·`ticks` 자릿수가 흔들리지 않았다.
- ★**`ticks`·`distinct_colors` 를 절대값으로 인용하지 마라**(0210 이 세운 규율) — 슈퍼액션히어로의
  1,450만~1,920만은 **3회 실측 범위**이지 상수가 아니다.
- **엔진 동작 변경의 범위**: 새 클래스 2개 등재뿐. 기존 클래스·기존 메서드 **수정 0**.

### 코퍼스 파일명 유입 — 0 이 아니다 (도구 실행값)

`node scripts/corpus-name-inflow.mjs --corpus <로컬 코퍼스>` 실행값 —
★**유입 BOUNDED 42회 / 22쌍** · ★**판단 필요 SUFFIX-ATTACHED 17회 / 7쌍** · PREFIX-EMBEDDED 0.
두 수를 함께 적는다. 「0건」이 아니다.
※`--corpus` 를 준 것은 이 워크트리에 코퍼스가 없어서다(git-ignored · 메인 체크아웃에 있다).

★**의도된 유입이다.** 이 티켓은 «어느 타이틀의 어느 벽인가»를 좇으므로, 이름을 적지 않으면
벽 이동표도 소유 판정도 후속 티켓도 성립하지 않는다. 코드 쪽 유입은 두 신규 파일의 **주석뿐**이고
(어느 게임이 그 멤버를 부르는지의 근거), ★**게임 바이트 커밋 0 · `game_lab/` 커밋 0** 이다.

★**SUFFIX-ATTACHED 7쌍은 기계가 못 가르는 바구니라 눈으로 전건 봤다** — 두 갈래뿐이고 오분류는 없다:
⑴**조사가 붙은 진짜 언급**(`…은`·`…의`·`…가`·`…는`·`…를`) ⑵**더 긴 실제 제목**(짧은 stem 에 `2` 가
붙은 속편으로, 그 속편 자신이 이 회차의 대상 타이틀이다). 어느 쪽도 «잘못 새어 든 이름»이 아니다.

<!-- corpus-name-inflow v1 subjects=7 tree=a5357b3bf143b6a6 B=42/22 P=0/0 S=17/7 -->
