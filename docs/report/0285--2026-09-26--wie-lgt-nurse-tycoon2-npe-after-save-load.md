## [2026-09-26] 간호사타이쿤2 로드 직후 NPE 는 게스트 자신의 것 · invalid memory access 의 이름 = `java/lang/Class` 필드 블록 word 0 오염 (wie-lgt-nurse-tycoon2-npe-after-save-load)

채택 제안 `2026-09-25-datainputstream-readboolean-slot#p0` 의 회차다.

★**판정 두 줄.**
- **null 의 출처는 우리 스텁도, 예외 전달 기구도 아니다.** 게스트 자신의 재귀다. 세이브 로드 메서드가 `Record not found` 를 잡은 catch 본문에서 **자기 자신을 다시 부르고**, 안쪽 호출이 정상 종료하며 static 스트림 필드를 `null` 로 지운다. 바깥 호출의 `finally` 가 그 필드에 `close()` 를 부르다 게스트의 암묵 null 검사가 `RaiseNullPointerException` 을 부른다. 그 NPE 는 게스트가 스스로 잡고 메서드는 정상 반환한다.
- **실제 벽(`Invalid memory access 0x706f6e50`)은 우리 런타임의 `java/lang/Class` 레이아웃 결함이다.** 게스트의 static 필드 접근자는 클래스 객체 필드 블록 **word 0 의 하위 halfword 를 플래그로** 읽는다. 그런데 그 자리에 RustJava `Class` 의 첫 필드 `nameBytes` **포인터**가 놓여 있었다. 포인터의 비트 13(`0x2000`)이 켜진 클래스는 static 기준 주소를 엉뚱한 곳에서 읽는다. 행 2개로 고쳤다.

### ⒜ 재현 — origin/main 에서도 같은 벽

`origin/main` `ed36be98` · `wie_validate --timeout 120`:
`FAIL · no frame rendered · stop deadline · paints 0` · `java_exceptions.count 4`.
예외 순서는 `InvalidRecordIDException` · `DataBaseRecordException` · `NullPointerException` · `WieError Invalid memory access; address: 1886342224(0x706f6e50)` 이다. 0250 의 after 와 같다.

### ⒝ null 의 출처 — 게스트의 자기 재귀 (임시 계측, 커밋 안 함)

`lr=0x297f0` 는 공유 꼬리 `0x297e8 mov lr, pc; bx r3` 의 복귀 주소다. 이 꼬리는 `Throw`(`0x297e4`)와 `RaiseNPE`(`0x27970 → b 0x297e8`)가 **같이 쓴다**. 그래서 lr 만으로는 어느 쪽인지 알 수 없다. 이 꼬리로 오는 분기는 함수(`0x27690`~`0x29838`) 안에 14곳이다.
엔진 루프에 PC 링을 넣어 실제 경로를 떴다. 세이브 로드 메서드 `0x27690` 의 진입·종료에서 fp 를 찍었다:

```
pc=0x27690 fp=0x400fffbc lr=0x1de90            ; 바깥 load 진입
pc=0x27840 fp=0x400fff80 r2=0x488498b0         ; static 스트림 = 바깥 스트림
pc=0x2962c fp=0x400fff80 ip=0x27690 lr=0x29630 ; catch 본문이 가상 호출로 «자기 자신» 호출
pc=0x27690 fp=0x400fff80 lr=0x29630            ; 안쪽 load 진입
pc=0x27840 fp=0x400fff3c r2=0x48840f30         ; static 스트림 = 안쪽 스트림(덮어씀)
pc=0x29814 fp=0x400fff3c r2=0x0                ; 안쪽 정상 종료: static 스트림 = null
pc=0x29630 fp=0x400fff80                       ; 바깥으로 복귀
pc=0x296b0 fp=0x400fff80 r3=0x0                ; 바깥 finally: static 스트림 읽기 → 0
                                               ; → 0x29790 → 0x27970 RaiseNPE
pc=0x29814 fp=0x400fff80                       ; NPE 는 게스트가 잡는다 · 바깥도 정상 종료
pc=0x29838 fp=0x400fffbc lr=0x1de90            ; 호출자로 반환(r0=0)
```

catch 는 `IsClassAssignable` 을 통과한 뒤 들어간다(`0x27770` → `0x2777c`). 잡은 예외는 첫 실행의 `Record not found` 다. 안쪽 호출은 0250 이 본 819회 읽기를 완주한다.
⇒ 이 NPE 는 **게스트 코드의 의미 그대로**다. 실기에서도 같은 순서면 같은 NPE 가 나고, 게스트가 스스로 삼킨다. 우리 스텁 반환값도, `exception::unwind` 도 null 을 만들지 않았다. 형제 티켓(`wie-lgt-unwind-handler-frame-reentry-rethrow-loop` · #292)에 합류할 축이 아니다.

### ⒞ 진짜 벽 — `<clinit>` 이 클래스 이름 문자열을 포인터로 읽는다

NPE 뒤 약 0.3초에 `WorldMap` 의 `<clinit>`(`0x1462ac`)이 돈다. `InitializeClass` 가 콜백으로 부르는 코드다. 폴트 지점:

```
0x1462ac ldr  r1, [r0, #8]      ; r0 = 클래스 객체 → r1 = 필드 블록
0x1462b0 ldrh r3, [r1]          ; word 0 하위 halfword
0x1462b4 tst  r3, #0x2000
0x1462b8 ldrne r3, [r1, #8]     ; 켜졌으면 word 2 …
0x1462c0 ldrne r3, [r3, #8]     ; … 의 +8 …
0x1462c4 addne r2, r3, #0x4c    ; … +0x4c 가 static 기준
0x1462bc addeq r2, r1, #0x14    ; 꺼졌으면 블록+0x14 (= 우리 레이아웃의 static 시작)
0x1462d4 str  r3, [r2, #4]      ; ← PROBE MEMERR addr=0x706f4c50 r2=0x706f4c4c
```

클래스별 필드 블록 앞 5워드를 `<clinit>` 직전에 떴다:

| 클래스 | word 0 | `& 0x2000` | word 1 | word 2 | word 3 | word 4 | 결과 |
|---|---|---|---|---|---|---|---|
| GameCanvas | `0x48845b60` | 0 | `0x488424f0` | name | 0 | 5 | 통과 |
| SoundManager | `0x48848250` | 0 | `0x488424f0` | name | 0 | 5 | 통과 |
| **WorldMap** | `0x48842140` | **켜짐** | `0x488424f0` | `0x176789` | 0 | 5 | word 2 = `nativeClassName` → 이름 문자열 +8 = ASCII → 폴트 |

word 0 은 RustJava `java/lang/Class` 의 첫 인스턴스 필드 `nameBytes`(`[B`) 참조다. word 1 은 `classLoader` 다. ABI 가 고정하는 것은 `nativeClassName`=2, `initializationState`=4 뿐이다. 나머지 둘은 선언 순서대로 0·1 에 떨어진다.
⇒ **벽은 힙 주소에 달렸다.** `nameBytes` 배열 주소의 비트 13 이 켜지면 그 클래스의 static 접근이 전부 틀린 곳으로 간다. 폴트 주소가 실행마다 다른 것도 그래서다(`0x706f6e50` · `0x706f4c50`). `0x706f....` 는 이름 문자열 바이트다.

이 검사는 한 곳이 아니다. 이 바이너리의 `tst …, #0x2000` 은 **310곳**이다. `0x2000` 검사 앞뒤 8명령 안에서 `[클래스객체+8]` 로 얻은 블록을 읽는 자리를 휴리스틱으로 셌다. word 0 halfword 가 47곳, word 2 가 있다. **word 1·3 을 읽는 자리는 0곳**이다(이 맥락 밖은 세지 않았다).

### ⒟ 수리 — `lgt_java_abi.toml` `java/lang/Class` 에 `field` 2행

```
{ name = "classLoader", descriptor = "Ljava/lang/ClassLoader;", index = 1 },
{ name = "nameBytes",   descriptor = "[B",                      index = 3 },
```

RustJava 의 두 필드를 게스트가 읽지 않는 word 1·3 에 고정했다. word 0 은 0 으로 남는다. `classLoader` 도 고정해야 한다. `nameBytes` 만 옮기면 고정되지 않은 `classLoader` 가 다음 빈 인덱스인 word 4 로 밀려 `initializationState` 와 겹친다(`class_definition.rs` 의 `instance_field_word_index` 규칙).
**upstream(`dlunch/wie` main `33d4935a`)도 같은 두 행만 가진다.** 이 결함은 물려받은 것이다.

★**값을 발명하지 않았다.** word 0 의 실기 의미(플래그 전체)는 모른다. 알아낸 것은 두 가지다. ⑴`0x2000` 이 «static 을 다른 블록에서 찾아라»는 뜻이다. ⑵우리 레이아웃은 static 을 항상 블록+0x14 에 둔다. 그러니 0 이 우리 레이아웃과 맞는 유일한 값이다.

### before / after

간호사타이쿤2, `--timeout 120`:

| | result · stop | paints | `<clinit>` 완료 | 마지막 예외 |
|---|---|---|---|---|
| before (debug) | FAIL · deadline | 0 | GameCanvas · SoundManager | `WieError Invalid memory access` (`WorldMap` `<clinit>`) |
| before (release) | FAIL · deadline | 0 | — | 같음 |
| after (debug) | FAIL · deadline | 0 | 위 2 + **WorldMap · Clinic · Patient · Room · LobbyNPC · XFont · Logo · GameMode** | `WieError Unimplemented: java/lang/Object vtable index 9` (thread 3) |

after 의 필드 블록 앞 5워드는 모든 클래스에서 `[0, classLoader, name, nameBytes, 5]` 다.
**다음 벽은 `java/lang/Object vtable index 9` 다.** 표 위 주석이 «wait() 로 관측됐으나 도달한 실행이 없어 쓰지 않았다»고 적은 바로 그 칸이다. 이제 도달하는 실행이 생겼다. 범위 밖이라 제안으로 남긴다.

### 코퍼스 짝 회귀 — 91건 · 귀속 가능한 변화 0

release(`lto = true`) `wie_validate --inject`, `game_lab/working/lgt` 54건 + `broken/lgt` 37건 = **91건**이다. 제목마다 base(`origin/main` `ed36be98`) 다음에 fix 를 **연달아** 돌려 같은 부하를 받게 했다. 4병렬 · loadavg 130~155.

| base → fix | 건수 |
|---|---|
| PASS → PASS | 54 |
| FAIL → FAIL | 30 |
| FAIL → PASS | 3 (나는마왕이다2 · 이터니티-천상의화원 · 제노니아1) |
| PASS → FAIL | 3 (이터널사가3 · 레전드오브마스터 · 제노니아2) |
| rc 134 → 134 | 1 (크로이센 abort — 0261 과 같다) |

뒤집힌 6건은 **전부** `no frame rendered` ↔ PASS 경계였다. 방향도 3:3 으로 대칭이었다. 그래서 6건을 짝으로 3회씩 다시 돌렸다(3병렬):

| 제목 | base | fix |
|---|---|---|
| 이터널사가3 | P F P | P F P |
| 이터니티-천상의화원 | P P P | P P P |
| 나는마왕이다2 | F F **P** | F F F |
| 제노니아1 | F F P | F F P |
| 제노니아2(working·broken 두 파일 × 3 — 스윕에서 뒤집힌 것은 broken) | F F F F P P | F F F F P P |
| 레전드오브마스터 | F P P | F P P |

⇒ 모든 제목이 **같은 바이너리 안에서** 양쪽 판정을 다 낸다. 짝끼리도 18짝 중 17짝이 같다. 어긋난 1짝(나는마왕이다2 3회차)은 첫 스윕과 **반대 방향**이다. 이 변경에 귀속되는 판정 변화는 **0** 이다.
나머지 84건은 두 판정 모두 같다. 그중 6건(영웅서기3 · 메이플스토리 도적편 · 블레이드마스터4 · 아니마 · 당신은골프왕 · 뮤직팩토리)은 `reason` 문면만 다르다. 키 단계 이름이나 blank/no-frame 문구 차이이고, 벽의 이름(`SVC id 904` · `/ by zero` · `SVC id 1100` · `JavaException`)은 같다.
★**한계**: 이 91건에서 static 이 실제로 엉뚱한 곳에 쓰였던 제목이 또 있는지는 재지 않았다. 비트 13 은 힙 배치에 달려 있어서, 폴트 없이 조용히 잘못 쓰는 경우는 판정에 드러나지 않는다.

### 단위 시험 · 개악

`jvm_support.rs` 의 클래스 객체 static 저장 시험에 두 단언을 더했다. ⑴블록 word 0 == 0 ⑵`classLoader`=1 · `nameBytes`=3.

변이: 두 행(`classLoader`·`nameBytes`)을 지우고 `cargo test -p wie-lgt` → `test_native_jvm_runtime` **FAILED** (`jvm_support.rs:1038` `assert_eq!(block_word_0, 0)`), 37 passed · 1 failed. 되돌리면 38 passed.

### 게이트

`cargo fmt --check` OK · `cargo clippy --all -D warnings` rc 0 · wasm32 clippy rc 0 · `+beta` clippy rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` rc 0(448 passed · 0 failed).
러너 블록: `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf` PASS paints 47 rc 0 · `keydraw_lgt` PASS paints 37 rc 0 · `text_j2me` PASS.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 를 돌렸다. 대상은 이 브랜치가 바꾼 4파일이다.
BOUNDED 와 SUFFIX-ATTACHED 는 아래 표식 줄의 값 그대로다. SUFFIX-ATTACHED 는 손으로 갈랐다. 전부 stem 뒤에 숫자나 `포켓` 이 붙은 «더 긴 다른 제목»이고, 그 제목들은 코퍼스에 있는 파일 이름이다. 이 회차가 새로 쓴 이름은 대상 타이틀과 코퍼스 짝 회귀 표의 제목들이다. 모두 `game_lab/working/lgt`·`broken/lgt` 의 파일 이름이다. 게임 바이트·경로는 쓰지 않았다.

<!-- corpus-name-inflow v1 subjects=4 tree=d9071cde9e94d308 B=76/38 P=1/1 S=17/7 -->
