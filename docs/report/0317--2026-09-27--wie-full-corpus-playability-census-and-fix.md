## [2026-09-27] 보유 게임 전수 플레이 가능성 점검 — 6축 · compat.json · 재생성 스크립트 (wie-full-corpus-playability-census-and-fix)

**무엇을**: 보유 게임 전부를 여섯 축으로 쟀다. 축은 부팅·화면·조작·10분 연속 플레이·소리·속도다.
그 결과로 셸의 «게임별 지원 현황» 안내 기능이 읽을 `compat.json` 을 만들었다.
핀이 바뀌면 같은 명령으로 다시 만들 수 있게 `scripts/playability-census.mjs` 를 넣었다.
`wie_validate` 는 LGT 결과 줄에 `lgt_compile_model` 키를 더한다(LGT 한정 · 다른 줄은 그대로).
**왜**: 운영자 지시(2026-09-27). 종전 표(`census-map.tsv`)는 STALE 이었고 부팅 한 축뿐이었다.
**사용자 영향**: 이 PR 은 측정 도구다. 고친 결과는 형제 PR(`docs/report/0318`)에 있다.

증적(이름 포함 · repo 밖):
- 전수 표: `game_lab/reports-2026-09-27-playability-census/census.tsv`
- compat: `~/orchestrator/reports/evidence/wie-full-corpus-playability-census-and-fix/compat.json`

핀: 스크래치 `19effc06` = `origin/main` `aeb09129` + #347 head `cedac0e9` + #348 head `dc9eb13e` + 이 PR 의 `lgt_compile_model` 키.
#347·#348 은 측정 뒤에 착지했다. 그러니 **소리·속도 축은 «PR 빌드 기준»** 이다.

### 모집단
- `working/` + `broken/` 에 zip 이 **464개** 있고, sha256 로 중복을 없애면 **429종**이다(같은 바이트 35개).
- 제외(재지 않음): `_dup` 17 zip(바이트가 같은 쌍둥이) · `_nongame` 1 · `vendor_sdk` 45 아카이브(SDK).

### 축과 판정 — 재기 전에 정했다
| 축 | 재는 법 | 판정 |
|---|---|---|
| 부팅 | 프로브 A(`--inject` 27키, 30초) · B(키 0, 같은 30초) | 둘 다 0 페인트이고 A 가 FAIL 이면 fail |
| 화면 | `content`(2색 이상 프레임) | ok · uniform(그렸지만 한 색뿐) · none |
| 조작 | A 의 프레임 중 B(무입력 대조)에 없는 것 | 있으면 ok. 입력 중 panic 은 none |
| 장시간 | 부팅·화면·조작이 ok 인 타이틀만 **600초**. 키 15개 순환(CLR·소프트키는 뺐다 — 종료 키다) · 20초마다 샷 | FAIL 줄이면 error. 30초 프로브에서 이미 그린 뒤 죽었으면 error |
| 소리 | #348 `audio` 계수(엔진 → 싱크) | 이벤트가 든 Play ≥1 이면 ok, 아니면 silent |
| 속도 | #347 `--pacing 8` · 비율 = 1 − (sleep 늦음 + timer 늦음 + GC) / 창 | ≥ 0.9 면 ok. 그 밖은 **n/a**(아래 «한계») — 0.9 미만은 낮은 동시성으로 1회 더 쟀다(`--only speed`). 둘 중 큰 값을 쓴다 |

`status`: 부팅·화면·조작·장시간이 모두 ok 면 playable. 부팅·화면만 ok 면 limited. 그 밖은 not-yet.

### 결과
**429종 → playable 261 · limited 81 · not-yet 87**.

| 플랫폼 | playable | limited | not-yet | 계 |
|---|---|---|---|---|
| KTF | 167 | 51 | 51 | 269 |
| LGT | 40 | 26 | 12 | 78 |
| SKT | 54 | 4 | 24 | 82 |

축별(종):

| axis | ok | n/a | fail | uniform | none | error | stall | silent | slow |
|---|---|---|---|---|---|---|---|---|---|
| boot | 361 |  | 68 |  |  |  |  |  |  |
| render | 342 |  |  | 15 | 72 |  |  |  |  |
| input | 301 | 87 |  |  | 41 |  |  |  |  |
| longplay | 261 | 114 |  |  |  | 54 |  |  |  |
| sound | 257 | 68 |  |  |  |  |  | 104 |  |
| speed | 162 | 267 |  |  |  |  |  |  |  |

이 수치는 **수정 전** 핀 기준이다. 형제 PR(0318)의 수정 빌드로 전수를 다시 재면(부팅·화면·조작만) 달라진다.
부팅 fail→ok 16종 · 한 색→그림 4종 · 입력 중 panic→ok 3종이다. 장시간 축은 다시 재지 않았다.
`compat.json` 의 `enginePin` 은 스크래치 머지 커밋이다(GitHub 에 없다). 형제 PR 착지 후 다시 만들면 그 머지 sha 가 들어간다.

### 다시 만들기
```
node scripts/playability-census.mjs run --bin <wie_validate> --out <dir> --jobs 12 <game_lab>/working <game_lab>/broken
node scripts/playability-census.mjs run … --only speed --jobs 3     # 0.9 미만만, 낮은 동시성으로
node scripts/playability-census.mjs report --out <dir> --pin <sha> --prs <gh-merged.json> [--changes <json>] --compat <compat.json>
```
`run` 은 이어 받는다(이미 있는 JSON 은 건너뛴다). 핀을 바꾸면 새 `--out` 을 쓴다.
이 회차는 전수 1회에 약 **4시간**이 들었다(load1 300 안팎 · 프로브 55분 · 10분 실행 281종).

### 측정의 한계 — 숫자와 같이 읽어라
- **호스트 부하**: load1 **250~415**, idle **0%**. 10분 실행 하나가 CPU **3~13%** 만 받았다. 벽시계 축(장시간·속도)은 게임이 덜 진행된 상태로 쟀다.
- **조작 축**: 스스로 움직이는 타이틀(데모 화면)은 입력이 없어도 새 프레임이 나오므로 ok 로 읽힌다(과대). 한 번만 재서 흔들린다. 전수 재측에서 12건이 뒤집혀 보였는데, 전·후 빌드를 짝지어 다시 재니 같았다(0318).
- **속도 축은 `slow` 를 말하지 않는다**: 늦음은 벽시계라 부하가 **더하기만** 한다. 그러니 부하 속 측정값은 하한이다.
  - ≥0.9 면 진짜 ok 다.
  - 0.9 미만은 아무것도 말하지 않는다. 실측 예: 같은 타이틀이 두 번에 0.22 와 0.85 였다. 다른 타이틀은 여기서 0.49, 조용한 브라우저에서(#347) 0.927 이었다.
  - 원인 계급(R·G)은 #347 이 브라우저 29종으로 이미 갈랐다. 나머지는 조용한 호스트에서 브라우저로 재야 한다(후속).
- **`stall` 은 판정하지 않는다**: 첫 회차에 «20초 샷 4장 동일» 40건을 열어 봤다.
  - 하나는 키 순환이 빠져나오지 못하는 하위 메뉴였다(CLR 없음).
  - 하나는 NUM1 을 기다리는 안내창이었다.
  - 굶은 호스트에서는 살아 있는 타이틀도 거의 그리지 않는다.
  - 그래서 가장 긴 동일 구간은 `census.tsv` 의 `still` 로만 남긴다.
- **소리 축은 엔진 쪽만 본다**: 30초 안에 소리를 내지 않는 타이틀(메뉴가 무음이거나 기본값이 꺼짐)도 silent 다. 브라우저가 떨어뜨리는 소리는 이 축이 아니다.
- 첫 회차 장시간 실행 74건은 `--max-ticks` 기본값(5천만)에서 멈췄다(3~4분). 기본값을 1000억으로 올려 다시 쟀다. 스크립트가 이제 이 값을 넘긴다.

### 문제 군집 — 첫 벽 · 걸린 타이틀 수 순 · sha 앞 12자
| 수 | 축:판정 | 첫 벽 | 타이틀(sha12) |
|---|---|---|---|
| 104 | sound:silent |  | 01e2715ba07a(ktf) 04159045a7ea(ktf) 0865be217bde(ktf) 0e72b6bc12bb(ktf) 0ed66634d3dc(skt) 1045007289d8(lgt) 1352b27a7898(lgt) 14a62a8521a0(skt) 155972cac664(ktf) 1793f87924d4(ktf) 182fa44210dc(ktf) 1b3b4868d46e(ktf) …(전체: 증적 clusters.md) |
| 41 | input:none |  | 04159045a7ea(ktf) 0c67145b11df(ktf) 1352b27a7898(lgt) 155972cac664(ktf) 182fa44210dc(ktf) 1cd151222bde(lgt) 229291e20b13(ktf) 23919eb33365(ktf) 2520654be6de(lgt) 287af341dac8(lgt) 2cbd63e4427a(ktf) 2f5246006bd8(skt) …(전체: 증적 clusters.md) |
| 10 | longplay:error | died on SIGABRT @ host stack overflow | 1cdea1985955(ktf) 3b5b98afa890(ktf) 61ba00511b84(ktf) 6c96c5050b2b(ktf) 7521052a0aa4(ktf) 8ac1d682de08(ktf) c01e041fa782(ktf) c654601e389f(ktf) c94d64777926(ktf) e9fac881e602(ktf) |
| 8 | boot:fail | panic during '…': called `Result::unwrap()` on an `Err` value: JavaException(#x#) @ wie-ktf/src/runtime/java/jvm_support.rs | 0392263fbb85(ktf) 1d5831e42a8a(ktf) 60bd6cbc5936(ktf) 83fc429f9cbe(ktf) b907b0faf483(ktf) bfa8ec352451(ktf) d552e095ddcf(ktf) dab2d537f3ef(ktf) |
| 8 | render:uniform | only blank/uniform frames (black screen) | 14a62a8521a0(skt) 4120f27288ab(ktf) 739c7657c1f2(ktf) 73f3a21e981c(lgt) 8b899f410f5d(ktf) b1ec149b354c(ktf) be08d047cbae(lgt) d2957348ddf4(skt) |
| 6 | boot:fail | no frame rendered (hang/black screen) | 01f05f8231f4(lgt) 8b01d4ae6af4(ktf) ab3d0020d7bc(ktf) d4188f8ef8c4(ktf) d60c34ffc9f6(ktf) ed6ad7318ac9(ktf) |
| 5 | longplay:error | panic during '…': called `Option::unwrap()` on a `None` value @ jvm-#.#.#/src/class_instance.rs | 0cc4ef7ede37(ktf) 249e655147a1(ktf) 85e94babc247(ktf) c7f543c73b91(ktf) cbe4cfe098c6(ktf) |
| 4 | boot:fail | tick error during '…': Fatal error: java.lang.NoSuchMethodError: javax/microedition/lcdui/Image.<init>:()V  @ net/wie/Launcher.startMIDlet(Ljavax/microedition/m | 08aa799c11b0(skt) 7089dec0e8df(skt) e3276ce8557c(skt) f78334f270be(skt) |
| 4 | render:none | --inject delivered #/# input steps (run ended: clean exit) — input survival NOT measured (otherwise: clean exit) | 0ed66634d3dc(skt) a42f77f44955(skt) c33090c12755(skt) eefc947d8337(skt) |
| 3 | boot:fail | panic during '…': called `Option::unwrap()` on a `None` value @ jvm-#.#.#/src/class_instance.rs | 0262a4fe3389(skt) 155586ece7f8(ktf) 339130e1eb47(ktf) |
| 3 | boot:fail | tick error during '…': Fatal error: java.lang.NumberFormatException: For input string: "" @ net/wie/WIPIMIDlet.startApp()V | 0eb19d9bbe7a(ktf) 4a7e489bf6ab(ktf) bc94ba53677b(ktf) |
| 3 | boot:fail | load error: unrecognized zip archive (no __adf__/app_info/.msd) | 1cf2e6076079(ktf) 94a34abdea21(ktf) f770b15f8876(ktf) |
| 3 | boot:fail | tick error during '…': Fatal error: java.lang.Error @ net/wie/WIPIMIDlet.startApp()V | 1e43e2e0055f(ktf) ae212f5390ce(ktf) f07cbc782828(ktf) |
| 3 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Unknown LGT WIPIC SVC id # @ net/wie/EventQueue.getNextEvent([I)V | 2a8a3dcd07eb(lgt) 320a5360a0f3(lgt) 6b515884dbc1(lgt) |
| 3 | boot:fail | panic during '…': called `Option::unwrap()` on a `None` value @ wie-ktf/src/runtime/java/jvm_support.rs | 3c658a46bbfb(ktf) 5a59f62d1f1a(ktf) c4b90400f9fe(ktf) |
| 3 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Unknown LGT WIPIC SVC id # @ net/wie/CletWrapper.startApp([Ljava/lang/String;)V | 87b04639cdfe(lgt) 8f7758fa43b6(lgt) acc4215b7ec0(lgt) |
| 2 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Invalid memory access; address: # @ net/wie/EventQueue.getNextEvent([I)V | 2b1ed0c8d061(ktf) 30c7bd6fb01b(ktf) |
| 2 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Unknown LGT WIPIC SVC id # @ net/wie/CletWrapperCard.keyNotify(II)Z | 2d5cada03004(lgt) caf9d76ffd13(lgt) |
| 2 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Unknown lgt stdlib import: #x# @ net/wie/CletWrapper.startApp([Ljava/lang/String;)V | 2dbde9acca99(lgt) b7699c10dfd1(lgt) |
| 2 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Invalid memory access; address: # @ net/wie/WIPIMIDlet.startApp()V | 362c57e2b2b7(ktf) c3057f46c59b(ktf) |
| 2 | longplay:error | tick error during '…': Fatal error: Unknown LGT WIPIC SVC id # | 3ff5948e235e(lgt) 863b8ab6a21d(lgt) |
| 2 | render:uniform | panic during '…': called `Option::unwrap()` on a `None` value @ jvm-#.#.#/src/class_instance.rs | 41466fc7f709(ktf) 52f1f32e3f72(ktf) |
| 2 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: #: KTF database slot # (header name MC_dbSortRecords) — argument layout unknown, measured r | 59263295de74(ktf) e085e193211d(ktf) |
| 2 | boot:fail | tick error during '…': Fatal error: java.lang.NullPointerException: Method java/lang/String::equals:(Ljava/lang/Object;)Z is called on null  @ net/wie/Launcher. | 66959afab216(skt) c6cadf75c454(skt) |
| 2 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: #: unidentified WIPI-C table — SVC selector # (struct slot Interface#), function #; entry r | 6af589a88cf9(ktf) 7da00ecd4804(ktf) |
| 2 | render:uniform | tick error during '…': Fatal error: java.lang.NumberFormatException: For input string: "" @ net/wie/WIPIMIDlet.startApp()V | 89c214dbd15d(ktf) aa4542c94ce4(ktf) |
| 2 | boot:fail | tick error during '…': Fatal error: java.lang.NoSuchMethodError: javax/microedition/lcdui/Canvas.getHeight:()I  @ net/wie/Launcher.startMIDlet(Ljavax/microediti | 916aea39fa36(skt) f6fe2adc8cce(skt) |
| 2 | boot:fail | died on SIGABRT @ host stack overflow | 9babd9789bae(ktf) a23f3c9fc2cb(lgt) |
| 1 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Double free at #x# @ net/wie/EventQueue.getNextEvent([I)V | 0093012b8c36(lgt) |
| 1 | longplay:error | panic during '…': called `Result::unwrap()` on an `Err` value: JavaException(#x#) @ wie-lgt/src/runtime/java/jvm_support/class_definition.rs | 1eaa92092bee(lgt) |
| 1 | longplay:error | panic during '…': called `Result::unwrap()` on an `Err` value: InvalidMemoryAccess(#) @ wie-lgt/src/runtime/java/jvm_support/array_class_instance.rs | 2a57e33133b5(lgt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NullPointerException: Array is null  @ net/wie/Launcher.start(Ljava/lang/String;)V | 2b6d78567795(skt) |
| 1 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Invalid memory access; address: # @ net/wie/CardCanvas.keyPressed(I)V | 306dcdb03842(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: #: unidentified KTF kernel extension — `MC_knlReserved#` is this repo's placeholder name fo | 3151fdc167b6(ktf) |
| 1 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Invalid memory access; address: # @ net/wie/CardCanvas.paint(Ljavax/microedition/lcdui/Graphi | 32c3c91305f7(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: #: MC_grpEncodeImage @ net/wie/WIPIMIDlet.startApp()V | 33801c1ba14f(ktf) |
| 1 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Method <init>(Ljava/lang/String;)V@# not found from org/kwis/msp/lwc/LabelComponent @ net/wie | 33f3e7669599(ktf) |
| 1 | longplay:error | only blank/uniform frames (black screen) | 34ab350dc98a(ktf) |
| 1 | render:uniform | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: #: unidentified WIPI-C table — SVC selector # (struct slot Interface#), function #; entry r | 4166acd8fc62(ktf) |
| 1 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Unknown lgt stdlib import: #x# @ net/wie/CletWrapperCard.keyNotify(II)Z | 4fdbd64c9fbd(lgt) |
| 1 | longplay:error | panic during '…': index out of bounds: the len is # but the index is # @ wie-core-arm/src/allocator/bucket.rs | 517ed32c92d6(lgt) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Fatal error: jump native address is null @ net/wie/WIPIMIDlet.startApp()V | 5267badf20b3(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Invalid memory access; address: # @ net/wie/EventQueue.getNextEvent([I)V | 568c339a8c07(ktf) |
| 1 | longplay:error | render: magenta color-key not applied (#% of frame) | 5814101b8010(lgt) |
| 1 | longplay:error | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: java/io/DataOutputStream vtable index # @ net/wie/CardCanvas.keyPressed(I)V | 61ed69520fd3(lgt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.ArithmeticException: Division by zero  @ net/wie/Launcher.startMIDlet(Ljavax/microedition/midlet/MIDlet;)V | 6e9991f08650(skt) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Unimplemented: java/util/Vector vtable index # @ net/wie/CardCanvas.paint(Ljavax/microedition/lcdui/Graphi | 70d709c40e10(lgt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NoClassDefFoundError: m/V#  @ net/wie/Launcher.start(Ljava/lang/String;)V | 71d1d8235bd1(skt) |
| 1 | longplay:error | tick error during '…': Fatal error: Unimplemented: #: MC_netSocket | 7d007391e4a1(ktf) |
| 1 | longplay:error | panic during '…': called `Result::unwrap()` on an `Err` value: AllocationFailure @ wie-ktf/src/runtime/java/jvm_support/jvm_implementation.rs | 7e2247bdf565(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NullPointerException @ net/wie/WIPIMIDlet.startApp()V | 96dc32e781d3(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.ClassFormatError: Invalid class file  @ java/lang/ClassLoader.defineClass(Ljava/lang/String;[BII)Ljava/lang/Class; | 990ae27f67e6(skt) |
| 1 | longplay:error | tick error during '…': Fatal error: java.lang.NoClassDefFoundError: com/xce/lcdui/TextComponentHandler  @ javax/microedition/lcdui/Canvas.handlePaintEvent(Ljava | 9a2cf5ffc9d3(skt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NullPointerException: Array is null  @ net/wie/Launcher.startMIDlet(Ljavax/microedition/midlet/MIDlet;)V | 9aa31965cd10(skt) |
| 1 | longplay:error | panic during '…': not implemented: Unsupported pixel format: # @ wie-midp/src/classes/javax/microedition/lcdui/image.rs | 9e8bc87708dc(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Method layout()V@# not found from org/kwis/msp/lwc/AnnunciatorComponent @ net/wie/WIPIMIDlet. | a10a1f02b41b(ktf) |
| 1 | render:uniform | panic during '…': called `Result::unwrap()` on an `Err` value: InvalidMemoryAccess(#) @ wie-lgt/src/runtime/java/jvm_support/class_instance.rs | a16f08d025eb(lgt) |
| 1 | boot:fail | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Method getX()I@# not found from org/kwis/msp/lwc/ShellComponent @ org/kwis/msp/lcdui/Main.mai | ae749cc5a777(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.ArithmeticException: / by zero @ net/wie/WIPIMIDlet.startApp()V | b2da04c55cd4(lgt) |
| 1 | render:uniform | tick error during '…': Fatal error: net.wie.WieError: Fatal error: Method <init>(Ljava/lang/String;)V@# not found from org/kwis/msp/lwc/LabelComponent @ net/wie | ca7fa8ade8ad(ktf) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NoSuchMethodError: javax/microedition/lcdui/Image.<init>:()V  @ net/wie/Launcher.start(Ljava/lang/String;)V | d1dce4a36141(skt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NullPointerException: Method java/lang/String::length:()I is called on null  @ net/wie/Launcher.startMIDlet(Ljavax | f2ae515201f2(skt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NullPointerException: src or dest is null  @ java/lang/System.arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V | f81658d4ad1a(skt) |
| 1 | boot:fail | tick error during '…': Fatal error: java.lang.NoClassDefFoundError: com/xce/io/ByteToCharEUC_KR  @ net/wie/Launcher.startMIDlet(Ljavax/microedition/midlet/MIDle | fb80e97cbc57(skt) |
| 1 | longplay:error | panic during '…': called `Result::unwrap()` on an `Err` value: JavaException(#x#) @ wie-lgt/src/runtime/wipi_c/context.rs | fe76e641bb3d(lgt) |

조작·소리 축 군집은 벽 문구가 없다(판정이 대조·계수라서). 속도는 판정이 `ok`/`n/a` 뿐이라 군집이 없다.
