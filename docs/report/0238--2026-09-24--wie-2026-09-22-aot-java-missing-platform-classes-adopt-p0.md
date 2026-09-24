## [2026-09-24] `wec/SYSTheme`·`wec/OEMDevice` — 「실기 트레이스 없이는 못 짓는다」를 «뒤집었다»: 게임은 두 상수로 분기하지 않는다 (wie-2026-09-22-aot-java-missing-platform-classes-adopt-p0)

채택 제안 `2026-09-22-aot-java-missing-platform-classes#p0`(0214 가 남긴 것)의 판정 회차다.
★**판정 = «최소 스텁은 지을 수 있다»** — 근거는 월드장기체스 `binary.mod` 디스어셈블이다. 값은 **하나도 지어내지 않았다.**

### ⒜ 코퍼스 교차 — `wec/` 참조는 월드장기체스 밖에도 있다(그러나 상수는 없다)

`game_lab/` 전체(working·broken·_dup·vendor_sdk·inbox · 7,647파일 · zip 안쪽 3단까지 227,445엔트리) 바이트 검색:

| 타이틀 | 참조 | 두 상수 |
|---|---|---|
| 월드장기체스(LGT · `_dup` 사본 포함 2파일 = 같은 sha256) | `wec/OEMDevice`·`wec/SYSTheme`·`getSYSTheme`·`saveItem` | 이름 **있음** |
| ★**귀신사냥2007 (KTF)** | `wec/OEMDevice`·`()Lwec/SYSTheme;+getSYSTheme`·필드 `Lwec/SYSTheme;+m_theme` | **0건**(`saveItem` 도 0건) |
| 미니러비·(KTF)미니러비 | `wec/GatewayIP` 만 | 0 |
| 해적왕2007 (KTF) | `wec/DMInfo` 만 | 0 |
| 더팜1 (KTF · working) | `wec/AppManager` 만 | 0 |

⇒ **0214 의 「고유 1타이틀」은 «두 클래스를 함께 쓰는» 기준으로는 틀렸다** — KTF 에 한 종이 더 있다. 그러나
★**두 정적 필드를 참조하는 것은 여전히 월드장기체스뿐**이라 **상수 값의 교차 복원은 불가**하다. `wec` 패키지는
단말(OEM) 확장 네임스페이스로 적어도 5클래스(`OEMDevice`·`SYSTheme`·`GatewayIP`·`DMInfo`·`AppManager`)가 쓰인다.

### ★타입은 복원된다 — 0214 의 「인접 서술자가 없다」는 «문자열 풀 스캔»의 착시였다

정적 필드 import 표(`.text` `0x65d9c`)는 `(이름 포인터, 서술자 포인터)` 쌍이고, 두 필드의 서술자 포인터는
**둘 다 `0x6725a` = 한 바이트 문자열 `"I"`** 다(게임 전체가 공유하는 `I`). 풀을 «이름 옆 문자열»로 읽으면 안 보인다.
링크 레코드(`LgtJavaClassLink` = 이름 포인터 + u16×10)도 디버그 로그와 바이트가 일치한다:

```
wec/SYSTheme  @0x6613c  static 0+2 · virtual 7+1 (saveItem) · direct 12+2 (클래스 getter 둘)
wec/OEMDevice @0x66154  direct 14+3 (getter 둘 + getSYSTheme)   ← <init> 없음 ⇒ getSYSTheme 는 static
정적 필드 import: [0] FORMAT_SOUND_MA3 : I   [1] ITEM_GROUP_SOUND_DEFAULT : I
```

### ⒝ 게임은 두 상수로 «분기하지 않는다» — 디스어셈블 원문

링크 인자(호출부 리터럴 풀 `0x648c`)에서 `static_field_word_indices = 0x1500684`(u16×2 = 정확히 4바이트),
`virtual_method_indices = 0x1500734`, `non_virtual_method_targets = 0x1500474` 를 얻고, `.text` 전체(104,057명령)를
리터럴 추적으로 훑어 그 슬롯을 읽는 곳을 셌다:

| 슬롯 | 사용처 |
|---|---|
| 정적 필드 idx 0·1 | **`0x436a8`·`0x436ac` 단 1곳** (게임 전체 정적 필드 사용이 이 2개뿐) |
| non-virtual 14·16·12 (`OEMDevice` getter · `getSYSTheme` · `SYSTheme` getter) | `0x43678`·`0x43684`·`0x43694` 단 1곳 |
| virtual idx 7 (`saveItem`) | `0x43788` 단 1곳 |

★**추적기가 눈먼 게 아니다** — 같은 코드가 모든 인덱스를 허용하면 virtual **524**·non-virtual **613** 사용처를 잡는다.

그 한 곳(`fn 0x42ab8` 안):

```
43684: ldr ip, [r4, #0x40]      ; OEMDevice.getSYSTheme()
4368c: bx ip                    ; → r0
43690: mov r5, r0               ; r5 = theme
43694: ldr ip, [r4, #0x30]      ; SYSTheme 클래스(초기화) getter
436a8: ldrsh r2, [r1]           ; 정적 idx0 word index
436ac: ldrsh r1, [r1, #2]       ; 정적 idx1 word index
436b8: ldr r7, [r2, #0x14]      ; r7 = FORMAT_SOUND_MA3
436bc: ldr r6, [r3, #0x14]      ; r6 = ITEM_GROUP_SOUND_DEFAULT
  …(사이 61명령: r6·r7 쓰기 0 · 읽기는 아래 두 mov 뿐 · 분기 5개는 전부 다른 레지스터의 null 검사)…
43774: cmp r5, #0 / beq 0x438f8 ; theme 가 null 이면 NPE 경로
43788: ldrsh r3, [r3, #0xe]     ; virtual idx 7 = saveItem
43798: mov r1, r6               ; 인자1 = ITEM_GROUP_SOUND_DEFAULT
437a4: mov r2, r7               ; 인자2 = FORMAT_SOUND_MA3
437b4: bx ip                    ; saveItem(group, format, String, byte[], int)
437b8: ldr r3, [r5]             ; r5 = sl 로 이미 갈아끼움
437bc: mov r0, r5               ; ★반환값 r0 을 읽기 전에 덮는다 ⇒ 버려진다
437d0: mov r6, #3               ; r6 재정의 · r7 은 이후 미사용(에필로그로 분기)
```

⇒ ★**두 상수는 `cmp`·분기에 한 번도 쓰이지 않고, `saveItem` 인자로만 흘러가며, 그 반환값은 버려진다.**
값이 의미를 갖는 곳은 **우리가 구현하는 `saveItem` 안쪽뿐**이다. 티켓 계약 1(「⒝가 분기 없음으로 확정될 때만 최소 스텁」)의
조건이 성립해 **지었다.**

### 무엇을 지었나 — 값 발명 0

`wie-wipi-java/src/classes/wec/{oem_device,sys_theme}.rs`(신규) · `get_protos` 51 → 53.

- `wec/OEMDevice.getSYSTheme()Lwec/SYSTheme;` — `PUBLIC | STATIC`, **null 이 아닌** 새 인스턴스를 준다(호출부가 null 이면 NPE 로 간다).
- `wec/SYSTheme` — 정적 `ITEM_GROUP_SOUND_DEFAULT : I`·`FORMAT_SOUND_MA3 : I` 를 **선언만** 하고 **값을 넣지 않는다**
  (`<clinit>` 없음 ⇒ JVM 기본값). `saveItem(IILjava/lang/String;[BI)I` 는 아무것도 저장하지 않고 0 을 돌려준다 —
  반환값이 버려짐을 위에서 쟀으므로 0 은 «실기 반환값에 대한 주장»이 아니다. `<init>()V` 는 `PRIVATE`(게임은 참조 0 ·
  `getSYSTheme` 만 쓴다).
- 부르지 않는 멤버는 없다. `wec/GatewayIP`·`DMInfo`·`AppManager` 는 **만들지 않았다**(이 티켓 범위 밖 · 각 타이틀의 현재 벽 미확인).

### 전·후 (release · `--inject` · 타이틀당 3회 · loadavg 31~37)

| 타이틀 | ticks 전→후 | paints | 멈춘 자리 전 → **후 (문면 그대로)** |
|---|---|---|---|
| 월드장기체스 (LGT) | 1 → 1 (3/3) | 0 → 0 | `NoClassDefFoundError: wec/SYSTheme` → ★`net.wie.WieError: Unimplemented: java/lang/String vtable index 16` (3/3) |
| 귀신사냥2007 (KTF) | 8·8·9 → 12·11·14 | 0 → 0 | `No such class: wec/OEMDevice` → `JavaException` panic ⇒ ★`java.lang.StringIndexOutOfBoundsException: begin 10, end 11, length 10 at ghost.startApp` (3/3) |

★**렌더를 켰다고 쓰지 않는다** — 둘 다 `paints 0`. 벽이 «클래스 해결 실패»에서 **다른 계급의 벽**으로 옮겨 갔을 뿐이다.
★월드장기체스는 `saveItem` 경로에 **아직 도달하지 않는다**(`RUST_LOG=warn` 에서 `wec.` 스텁 경고 **0줄**) — 닫힌 벽은 0214 가
본 그대로 «사용»이 아니라 «링크 시 해결»이었다. 귀신사냥2007 은 `getSYSTheme` 를 **실제로 1회 호출**한다(경고 1줄).
★귀신사냥2007 의 새 벽이 스텁 탓인지는 **재지 않았다** — 그 타이틀은 두 상수·`saveItem` 참조가 0이고 `m_theme` 필드에 저장만 한다.

### 검증

- 단위 시험 `get_sys_theme_then_save_item_with_the_static_fields` — 게임 경로 그대로(`getSYSTheme` → 두 정적 읽기 → `saveItem`).
- **양방향 변이**: 원본 **ok** · M1(두 프로토 등재 제거) **FAILED** · M2(`getSYSTheme` 가 null 반환) **FAILED** — 복원 후 다시 ok.
  게임 수준 M1 은 «전» 바이너리 그 자체다(`NoClassDefFoundError` 3/3).
- 로컬 게이트·회귀 수치는 회신(`~/orchestrator/reports/wie-2026-09-22-aot-java-missing-platform-classes-adopt-p0.done.md`)에 있다.

### 한계 — 숨기지 않는다

- ★**이 스텁은 «월드장기체스의 호출부»에 대해서만 옳다.** 두 상수로 분기하는 호출자가 나타나면 이 스텁은 그 호출자에게 틀린다 —
  그때 값은 실기 트레이스나 OEM 문서에서 와야 한다(소스 주석에 같은 문장을 남겼다).
- `saveItem` 은 «단말 시스템 테마에 소리 저장»이라 저장하지 않아도 게임 상태는 변하지 않는다 — 그러나 실기에서 게임이 그 효과를
  **나중에 읽는** 경로(다른 API 로)가 있는지는 이 바이너리 안에서 `SYSTheme` 멤버 사용이 위 3곳뿐이라는 것까지만 쟀다.
- 전 estate 계측이 아니라 이 Mac 의 로컬 코퍼스(`game_lab/`, git 무시 · Constraint 9)다.
