## [2026-09-28] 한 색/무화면 14종 첫 벽 재분류 · invokespecial 상위 클래스 탐색 · RMS 첫 실행 · LGT DataInputStream 24 (wie-census-blank-screen-14-and-invokespecial-lookup)

**무엇을**
- `wie-jvm-support`: 우리가 JVM 에 넘기는 클래스 정의(Rust 프로토 · 게스트 바이트코드 둘 다)를 `InheritedMethods` 로 감싸
  `method()` 가 상속받은 인스턴스 메서드도 답하게 했다 — 핀 `jvm-0.1.1` 의 `invoke_special` 이 이름 붙은 클래스만 보던 것(JVMS §5.4.3.3 위반)을
  crates.io 크레이트를 건드리지 않고 푼다. `<init>`/`<clinit>`·`static`·`private` 은 넓히지 않는다.
- #350 의 `Canvas.getWidth/getHeight` 재선언 우회를 걷어냈다(같은 시험이 이제 일반 경로로 green).
- `javax.microedition.rms.RecordStore.openRecordStore(name, false)`: 없는 저장소면 `RecordStoreNotFoundException`(MIDP 규격) ·
  `true` 면 그 자리에서 저장소를 만든다. 클래스 `RecordStoreNotFoundException` 추가. KTF `DataBase.openDataBase` 는 종전 동작 유지(아래).
- `wie-lgt/data/lgt_java_abi.toml`: `java/io/DataInputStream` 24 = `readUnsignedByte()I`.

**왜**: 전수 점검(0321) 후속 군집 «render uniform 8 · no frame rendered 6» 과 «invokespecial 이 이름 붙은 클래스에서만 찾는다».

**사용자 영향**: SKT 4종(14종 안 `14a62a8521a0` + 14종 밖 `2b6d78567795` `9aa31965cd10` `f81658d4ad1a` — 같은 첫 실행 RMS 병)·LGT 1종(`73f3a21e981c`)이
한 색 화면 또는 부팅 실패에서 실제 화면·조작까지 간다. 나머지 14종 안 타이틀은 아래 «다음 벽».

### 14종 — 첫 벽으로 다시 나눔(수 순)

측정 = release(LTO 끔) `wie_validate`, `RUST_LOG=debug` 15초 + 필요 시 임시 계측(커밋 0).

| 수 | 첫 벽 | sha12 | 처분 |
|---|---|---|---|
| 4 (앱 2개 × 판 2) | KTF clet · 게임 자체 소프트 타이머 3칸이 끝까지 비어 있다. 5ms 콜백이 `MC_knlCurrentTime` → 빈 칸 확인 → `MC_knlSetTimer` 만 반복, 화면 0 | `d4188f8ef8c4` `ed6ad7318ac9`(같은 앱) · `8b01d4ae6af4` `d60c34ffc9f6`(같은 앱) | **못 고침**. 아래 «KTF 타이머 군» |
| 2 (앱 1개 × 판 2) | KTF(`dnff`) · 로딩 스레드가 `monitor_exit` 뒤 멈추고 게임 스레드는 `sleep` + 클래스 초기화 도우미만 반복(`js_commonResInvokeNativeClinit` 로그 수천 줄). 첫 흰 화면만 | `4120f27288ab` `739c7657c1f2` | **못 고침**. 제안 등재 |
| 2 | 이미 풀림 — 측정 기준(`f0045fba` = main + #350) 에서 화면 ok. `be08d047cbae` 는 #349 의 이용자 기록이 이미 있다 | `be08d047cbae` `b1ec149b354c` | 조치 없음 |
| 1 | SKT · `openRecordStore(name,false)` 가 빈 저장소를 열어 줘서 게임이 «첫 실행 아님»으로 읽고 레코드 1 을 null 로 받은 뒤 paint 마다 NPE | `14a62a8521a0` | **고침** |
| 1 | LGT aot-java · `Unimplemented: java/io/DataInputStream vtable index 24` | `73f3a21e981c` | **고침** |
| 1 | SKT · LBMP `Unsupported grayscale type 2` → `Image` null(= `f6fe2adc8cce` 의 첫 원인) | `d2957348ddf4` | #361(열림) 이 고친다 — 이 PR 무접촉. #361 착지 뒤 재측 |
| 1 | KTF clet · 게임이 화면 프레임버퍼(`MC_grpGetScreenFrameBuffer`)에 그리고 `MC_grpRepaint` 만 부르며 `MC_grpFlushLcd` 를 한 번도 부르지 않는다. clet 모드는 MIDP paint 를 꺼 두므로 화면 0 | `ab3d0020d7bc` | **못 고침** — «clet 카드 paint 뒤 화면 FB 자동 반영» 은 다른 clet 전부에 걸리는 의미 결정이라 이 회차에서 넣지 않았다 |
| 1 | KTF · 첫 실행 `DataBase.selectRecord` 예외를 게임이 잡은 직후 `jump native address is null` | `8b899f410f5d` | **못 고침**(예외 경로 · 식별 근거 없음) |
| 1 | LGT clet · 부팅 후 타이머 0 · `CletWrapperCard.paint` 에서 WIPIC `unk1(0xa600,…)` → `unk11(0,0xa600)` 뒤 아무 일도 없음 | `01f05f8231f4` | **못 고침**(두 스텁 식별 근거 없음) |

★`unk12-1`(KTF Interface12) 은 `ab3d0020d7bc` 의 flush 가 아니다 — 재생 가능한 타이틀 `f44271803135` 가 같은 함수를 부른 **직후** `MC_grpFlushLcd` 를 따로 부른다(246 대 246).

#### KTF 타이머 군(4) — 잰 것

- 콜백 `0x1088cc`(앱 `010209E1`): `now-last` 계산 → 게임 표 3칸(보폭 0x14)의 `+0xc == 1` 을 찾고 없으면 5ms 재무장. 칸을 켜는 함수 `0x10bae0` 의
  호출부 5곳 중 4곳이 게임 내부 메시지 처리기(`0x1013c0`, 메시지 0xe~0x19) 안이다.
- 상태 변수(임시 계측 · 1,106틱 동안 불변): 주 상태 `1`, `[sl+0x18]→1`. 메시지 처리 루프(`0x100eb4`)의 상태 분기는 3·7·8·9·0xa·0x18 만 일을 하고,
  상태 3 으로 가는 함수(`0x10aa78`)는 게임 코드 어디에서도 호출되지 않고 큰 콜백 표의 `+0x384` 칸에만 등록된다 ⇒ **플랫폼이 부르는 콜백**으로 보이나 무엇인지 모른다.
- 키 27개 주입(`--inject`)으로도 화면 0 — 입력 대기 화면이 아니다.

### invokespecial — JVMS §5.4.3.3

- 핀(`jvm-0.1.1` · crates.io): `invoke_virtual`/`invoke_static` 은 `resolve_method` 로 상위 클래스를 걷는데 `invoke_special` 만
  `class.definition.method(name, descriptor, false)` 한 줄이다. 크레이트는 `[patch]` 없이 crates.io 에서 오므로 거기서 고치지 않았다.
- 우회 지점을 일반화: wie 가 JVM 에 정의를 건네는 **두 곳**(`RustJavaJvmImplementation::define_class_rust` · `JvmRuntime::define_class`)에서만 감싼다.
  KTF/LGT 는 자기 `ClassDefinition` 을 쓰고 그 형은 다운캐스트되므로 감싸지 않는다(SKT·J2ME 경로).
- 대가(주석에 명시): 상속 메서드를 가상 호출하면 프레임이 선언 클래스 대신 하위 클래스를 가리킨다 — 스택 트레이스 표기와, 그 프레임에서 고르는 클래스 로더가 하위 클래스의 것이 된다(상위로 위임하므로 찾는 결과는 같다).
- 코퍼스 429종 전수 로그에서 invokespecial 모양 NoSuchMethodError 는 `Canvas.getHeight` 2종뿐이었다(`Image.<init>()V` 는 진짜 부재). 즉 이 일반화가 **새로 살리는** 타이틀은 측정상 0이고, 값은 #350 의 우회를 걷어낸 것과 다음 같은 모양을 미리 막는 것이다.
- #350 두 타이틀: `916aea39fa36` 전·후 모두 부팅·화면·조작 ok(우회 삭제 후 같음) · `f6fe2adc8cce` 전·후 모두 getHeight 통과(NoSuchMethodError 0) → 다음 벽 LBMP type 2 + 한 tick 미종료(#361 이 고친다).

### `f6fe2adc8cce` 한 tick 무한

#361(열림) 이 원인을 판정하고 고쳤다(음악 스레드 `while(a) clip.play();` + LBMP type 2) — `docs/report/0329`. 이 PR 에서는 다시 하지 않았다.

### RMS 첫 실행 — 근거

게임 클래스 `j.a(String,Z)Z`(바이트코드): `create=false` 로 `openRecordStore` → 성공하면 `getRecord(1)` 을 필드에 담고 **예외를 잡으면 true** 를 돌린다.
MIDP 는 없는 저장소에 `RecordStoreNotFoundException` 을 던지고, 그때 게임은 false → 기본값을 만든다. 종전 wie 는 `create` 를 무시해 빈 저장소를 열었다
⇒ `getRecord(1)` 이 `InvalidRecordIDException` → 잡혀서 true · 레코드 null → `j.a([BI)I` NPE(paint 마다).
KTF `org.kwis.msp.db.DataBase.openDataBase(…, create=false)` 는 이 경로를 쓰지만 **항상 create 로** 부르게 바꿨다 — KTF 게임은 `DataBaseException` 을 잡지 RMS 예외를 모른다. 종전 동작 그대로다.

### DataInputStream 24 — 근거

누락 칸 핸들러 임시 덤프(커밋 0): LR `0x76fe4` · 디스패치 `ldr ip, [r3, #0x64]`(= 4 × (24+1)) · r0 = 스트림, 다른 인자 없음 · 결과는 `r0 & 0xff` 만 쓴다 ·
같은 스트림에 바로 다시(LR `0x77004`). 23 = readByte 가 실측 앵커이고 슬롯 순서가 24 = readUnsignedByte 를 가리킨다. 둘 다 같은 하위 바이트를 돌려주므로 마스크하는 게스트엔 같은 값이다.

### 전/후 — 같은 시각 짝(전수 점검 프로브 A/B 그대로)

전 = `f0045fba`(main `bf298b4e` + #350 head `7d26bd38`) · 후 = 이 브랜치 · 둘 다 release·LTO 끔 · 프로브 A = `--inject --keep-timeout --timeout 30 --pacing 8`,
B = `--inject-keys 0 --shot-every 1` · 축 = 부팅/화면/조작 · p = A,B paints · 2026-09-28 04:0x KST · **load1 307~364**.

| sha12 | 전 | 후 | 다음 벽 |
|---|---|---|---|
| `14a62a8521a0` | ok/uniform/n/a p1,1 | **ok/ok/ok** p313,277 | 장시간 아래 |
| `73f3a21e981c` | ok/uniform/n/a p1,1 | **ok/ok/ok** p50,125 | 장시간 아래 |
| `b1ec149b354c` | ok/ok/none p2,2 | ok/ok/ok p2,2 | (조작 축 흔들림 · 코드 무관) |
| `be08d047cbae` | ok/ok/ok p2,199 | ok/ok/ok p2,196 | – |
| `916aea39fa36` | ok/ok/ok p26,7 | ok/ok/ok p26,7 | – |
| `f6fe2adc8cce` | getHeight 통과 · tick 미종료 | 같음 | #361 |
| `d2957348ddf4` | ok/uniform/n/a p1,1 | 같음 | LBMP type 2(#361) |
| `4120f27288ab` | ok/uniform/n/a p8,8 | ok/uniform/n/a p8,8 | 로딩 스레드 정지 |
| `739c7657c1f2` | ok/uniform/n/a p8,8 | ok/uniform/n/a p7,8 | 같음 |
| `8b899f410f5d` | ok/uniform/n/a p1,1 | 같음 | null 점프 |
| `d4188f8ef8c4` `ed6ad7318ac9` `8b01d4ae6af4` `d60c34ffc9f6` | fail/none p0,0 ×4 | 같음 ×4 | 타이머 칸 비어 있음 |
| `ab3d0020d7bc` | fail/none p0,0 | 같음 | FlushLcd 없음 |
| `01f05f8231f4` | fail/none p0,0 | 같음 | LGT unk1/unk11 |

라이브 LGT 5종 + 퇴행 가드 2종(같은 짝):

| sha12 | 전 | 후 |
|---|---|---|
| `13d7e3c21856` | ok/ok/ok p222,388 | ok/ok/ok p212,384 |
| `1b107b96bf4e` | ok/ok/ok p129,275 | ok/ok/ok p306,273 |
| `4ece6eeeaa04` | ok/ok/ok p12,2 | ok/ok/ok p12,2 |
| `a30bbe008b5e` | ok/ok/ok p237,335 | ok/ok/ok p237,321 |
| `b475b6399684` | ok/ok/ok p50,104 | ok/ok/ok p203,107 |
| `49ade89578c5` | ok/ok/ok p72,101 | ok/ok/ok p72,98 |
| `ddd885583b15` | ok/ok/ok p36,69 | ok/ok/ok p44,71 |

퇴행 0. (`4ece6eeeaa04` A 는 전·후 모두 8/27 키에서 메뉴 «게임종료»로 끝난다 — #350 의 의도된 동작.)

SKT/J2ME 82종 짝 스윕(invokespecial 변경이 SKT·J2ME 의 모든 클래스 정의를 지나므로 · `--inject --keep-timeout --timeout 20` · 같은 시각):
PASS 62 → **67** · FAIL 10 → 6 · UNMEASURED 9 → 8 · **나빠진 타이틀 0**. 바뀐 5종 = 위 RMS 4종 + `f483ba078c14`(전 UNMEASURED 23/27 clean exit → 후 PASS — 종료 시점 흔들림, 코드 무관).
14종 밖 3종의 프로브 A/B(같은 짝): `2b6d78567795` fail/none → **ok/ok/ok** p641,480 · `9aa31965cd10` fail/none → **ok/ok/ok** p393,363 ·
`f81658d4ad1a` fail/none → **ok/ok/ok** p678,896. 셋 다 전수 점검의 «`NullPointerException: Array is null` @ Launcher» / «`src or dest is null` @ arraycopy» 군이었다
(후 로그: `openRecordStore(…, false)` → `RecordStoreNotFoundException` 뒤 정상 진행).

장시간(후 · 전수 점검 L 과 같은 키 반복 · `--timeout 600`):

| sha12 | 결과 | 비고 |
|---|---|---|
| `73f3a21e981c` | 600초 오류 0 · 854/900 키(부하로 일정 초과 → UNMEASURED) · paints 45 | 장시간 ok |
| `14a62a8521a0` | **220번째 키(약 146초)에서 FAIL** — `NoClassDefFoundError: com/xce/lcdui/TextComponentHandler` | **다음 벽** = #361 이 그 클래스를 넣는다 |
| `2b6d78567795` | 600초 오류 0 · 854/900 · paints 10,661 | 장시간 ok |
| `9aa31965cd10` | 600초 오류 0 · 854/900 · paints 7,638 | 장시간 ok |
| `f81658d4ad1a` | 600초 오류 0 · 854/900 · paints 14,529 | 장시간 ok |

### «되돌리면 red» — 변이 실측(각각 원복 후 green)

| 변이 | 결과 |
|---|---|
| `InheritedMethods::method` 의 상위 탐색을 `None` 으로 | `invokespecial_finds_inherited_methods_but_not_constructors` · `canvas_super_size_resolves_on_canvas` **둘 다 FAILED** |
| `name.starts_with('<')` 제외 삭제 | `invokespecial_…` **FAILED**(«an inherited <init> was found») |
| `openRecordStore` 의 `!exists` → throw 삭제 | `open_without_create_throws_until_the_store_exists` **FAILED** |
| ABI 24 행 삭제 | `abi_rows_cover_the_indexes_titles_actually_dispatch_on` **FAILED**(«vtable index 24 is empty») |

`private`·`static` 제외는 시험이 없다(한 줄 필터 · 넓히면 틀리는 방향이 분명해 남겨 둔 선).

### 게이트

fmt · clippy `-D warnings`(stable · beta · wasm32) · `RUST_MIN_STACK=4194304 cargo test --all`(49 스위트 · **536 passed** · 0 failed) · `npm run build:wasm` ·
`check-engine-contract.mjs` · `npm run audit` · 러너 줄(draw_j2me · helloworld_ktf/lgt · keydraw_ktf/lgt `--inject --expect-last-frame` · text_j2me) 전부 PASS·rc=0.
(`draw_j2me`·`text_j2me` 의 `Failed to decode image` 로그 1줄은 기준 빌드에서도 같다.)
- 게임 파일명 유입: 이 회차가 **더한 줄**에는 0건이다(코드 주석은 sha12 로만 적었다). 도구 표기 BOUNDED·SUFFIX-ATTACHED 는 고친 파일에 **이미 있던** 주석과,
  이 브랜치가 아래에 깔고 있는 #350 의 파일에서 온다.

### -fix2 — origin/main(da6c7a6a) 병합 해소 (wie-census-blank-screen-14-and-invokespecial-lookup-fix2)

#359·#367·#361·#368 착지 뒤 병합 충돌 3파일을 합집합으로 풀었다(새 기능 0).
- `rms.rs` — mod/pub use 에 main 의 `RecordComparator`·`RecordEnumeration(Impl)`·`RecordFilter` 와 이 PR 의 `RecordStoreNotFoundException` **전부**.
- `record_store.rs` — 시험 모듈의 양쪽 추가분(이 PR `open_without_create_…` · main 의 열거 시험) **둘 다 보존**.
- `lib.rs` `get_protos()` — 길이 **45**(base 40 + PR 1 + main 4) · `as_proto()` 행 45 · 중복 0.

게이트 재측: fmt · clippy `-D warnings`(stable · beta · wasm32) · `cargo test --all` 0 failed · `npm run build:wasm` · 러너 줄 전부 PASS·rc=0(keydraw paints 55/55).
대상 재측(release · LTO 끔 · `--inject --keep-timeout --timeout 30 --pacing 8` · load1 112): `14a62a8521a0` **PASS** rc=0 · 27/27 · paints 483 ·
`73f3a21e981c`(`--max-ticks 400000000`) **PASS** rc=0 · 27/27 · paints 69 · `DataInputStream vtable index 24` 로그 0.
- 게임 파일명 유입(-fix2 재측): BOUNDED 90회/28쌍 · SUFFIX-ATTACHED 14회/4쌍. 이 회차 문서 추가분은 0건(도구 적중에 이 파일 없음) · 88→90 증가는 쌍 수 불변(28)이고 병합으로 들어온 기존 줄 몫이다.
<!-- corpus-name-inflow v1 subjects=16 tree=9dc930e9d465f221 B=90/28 P=1/1 S=14/4 -->
