## [2026-09-29] LGT import 0x64 = 수신자의 인터페이스 메서드 표 — 월드장기체스 메뉴·연습 대국 화면 · `--inject` 27/27 PASS (wie-2026-09-29-wec-vector-vtable-17-adopt-p0)

**무엇을**: LGT Java 시스템 import `0x64` 를 `GetInterfaceMethodTable`(SVC id 37)로 등재했다.
`0x64(receiver, interface_name)` 는 수신자 클래스가 그 인터페이스 메서드를 구현한 표를 돌려준다 — 0번 워드 = 인터페이스 raw class,
`1 + i` 번 워드 = 인터페이스 메서드 `i` 에 대한 **수신자의** 구현 target(없으면 0). (수신자 클래스, 인터페이스)마다 한 번 만들고 캐시한다.

**왜**: Vector 17(#405 · 0373) 뒤 월드장기체스가 키 2 에서 `Unknown lgt java import: 0x64` 로 멈췄다(4/4).

**사용자 영향**: 월드장기체스가 제작사 로고 → 타이틀 → 메인 메뉴 → 연습 → 체스판(중수급 AI 대국)까지 간다. 그 뒤 보드 화면에서 키가 화면을 바꾸지 못한다(아래 한계).

### 0x64 판정 — binary.mod(ELF · ARM · 기호 없음) 디스어셈블 + 실행 1회 계측

- 트램폴린 `{push {lr}; bl resolver; 0x64; 0x64}` = `.data` `0x1403a74`. `.text` 리터럴 풀 참조 3곳(`0xd950`·`0x261c8`·`0x26f08`) → 호출부 3곳.
  이 바이너리는 `0x0a`(`GetInterfaceDispatchTable`)를 import 하지 **않는다**. 널 슬롯 대비책은 import `0x40` 이다.
- 세 호출부가 같은 모양이다(`0xd82c`, #405 의 elements() 루프 머리):

```
0xd80c ldr r3,=0x1500474 ; ldr ip,[r3,#0x44] ; bx ip   ; 인터페이스의 클래스 getter → r0 = raw class
0xd81c ldr r3,[r0,#8] ; ldr r1,[r3,#8]                 ; r1 = class.descriptor(+8).name(+8) — 인터페이스 이름
0xd828 mov r0,r8                                       ; r0 = 수신자(elements() 의 결과 · 직전에 null 검사)
0xd82c ldr r6,=0x1403a74 ; bx r6                       ; import 0x64
0xd838 ldr r3,[r0,#4] ; cmp r3,#0 ; bne 0xd858          ; ← lr · 표의 1번 워드 = 인터페이스 메서드 0
0xd844 mov r0,r3 ; ldr r3,=0x1403c14 ; bx r3            ;   0 이면 import 0x40 (대비책)
0xd858 mov r0,r8 ; bx r3                               ;   아니면 수신자를 r0 로 그 target 호출
0xd864 cmp r0,#0 ; beq 0xd8f0                           ; hasMoreElements() 거짓 → 루프 탈출
```

  `0x260f4`·`0x26df4` 는 슬롯 번호를 상수 대신 링크 산출물(`ldrsh r3,[=0x150072e, #4/#2]` — 링커가 `interface_method_indices` 에 쓰는 u16)에서 읽고
  `add r0,r0,r3,lsl #2 ; ldr r3,[r0,#4]` 로 같은 표를 찾는다 ⇒ 표는 **인터페이스 메서드 순서로 색인**된다(링커가 인터페이스에 대해 `virtual_method_index` 로 내주는 순서).
- 인자: r0 = 수신자 객체, r1 = 인터페이스 이름(UTF-8 C 문자열). 반환: 워드 표 포인터. 표의 워드는 **수신자를 r0 에 넣고 호출되는 target**이다.
- **왜 `0x0a` 와 같은 답이 아닌가**: `0x0a` 는 인터페이스 **자신의** vtable 을 돌려주는데, 그 슬롯은 추상 메서드 target 이라 호출하면 `AbstractMethodError` 다(`abstract_method_target_throws_abstract_method_error`).
  호출부가 그 워드를 수신자 위에서 **직접 호출**하므로, 필요한 것은 수신자 클래스의 구현이다. 변이 ⒝가 이것을 잰다.
- 실행 계측(임시 `warn!` · 커밋 안 함, release `--inject` 1회): `lr=0xd838 recv=java/util/Hashtable$Enumerator iface=java/util/Enumeration` · 1·2번 워드 모두 0 아님 · 표 생성 1회(이후 캐시).
  lr 은 디스어셈블한 `0xd82c` 호출의 복귀점이고, r1 은 실제로 인터페이스 이름이었다. 수신자는 RustJava `Vector.elements()` 가 돌려주는 클래스다.
- RustJava `java/util/Enumeration`(`dlunch/RustJava@5b84dd1` `java_runtime/src/classes/java/util/enumeration.rs`) 선언 순서가 `hasMoreElements`·`nextElement` 라 호출부의 고정 오프셋 `+4`(루프 조건)·`+8`(본문, 캐스트 대상 객체 · 0373)과 일치한다.
- 코퍼스: ARM 트램폴린을 쓰는 LGT 16타이틀 **전부**가 `0x64` 를 import 하고 `0x0a` 는 0 이다. 2026-09-28 전수 조사(`reports-2026-09-28-playability-census-1dd9f81c-r2`)에서 `import: 0x64` 로 죽은 타이틀은 0 — 이 경로에 닿은 것은 이번 타이틀이 처음이다.

### 실측 — release `wie_validate --inject`

| 트리 | result | stop | ticks | paints | 키 | frozen_tail | 첫 예외 |
|---|---|---|---|---|---|---|---|
| 변이 ⒜(0x64 행만 제거 = main 동작) ×2 · load1 99~101 | FAIL | error | 2607908 / 3332983 | 161 / 164 | 2/27 | 0 | `Unknown lgt java import: 0x64` |
| 이 브랜치 ×4 · load1 60~65 | **PASS** | deadline | 2306102 / 8632256 / 6286651 / 8977667 | 278 / 485 / 465 / 485 | **27/27** | 12 | 없음(`FileNotFoundException` 1 — 첫 실행 저장 파일) |

- 첫 판은 `--shotdir … --shot-every 2` 로 찍었다(git 무시 스크래치): 안내문 → MANASTONE 로고 → 타이틀(Ver. 1.0.4) → 메인 메뉴 → 메인메뉴>연습 → 체스판 「중수급 연습」(사용자 vs 중수급 AI).
- #405 의 후 4판(같은 main 코드)도 4/4 `0x64` 였다.

### 시험(되돌리면 red)

`interface_method_table_dispatches_on_the_receiver_in_interface_order`: 원소 1개인 Vector 의 `elements()` 수신자에 import 0x64 를 부르고,
1번 워드(hasMoreElements) = 1 → 2번 워드(nextElement) = 그 원소 → 1번 워드 = 0 을 **게스트 호출(`run_function`)로** 확인한다. 같은 키의 두 번째 호출은 같은 표.
변이 모두 red(실측):
- ⒜ 표에서 `0x64` 행 삭제 → `FatalError("Unknown lgt java import: 0x64")`
- ⒝ 0x64 가 `0x0a` 와 같은 표(인터페이스 자신의 vtable)를 돌려주게 → `JavaException(…)`(추상 target 호출)

SVC id 왕복 시험의 마지막 id 도 `GetInterfaceMethodTable` 로 옮겼다.

### 한계

- **보드 화면에서 멈춘다**: 4/4 `frozen_tail_steps` 12 — 마지막 12키 동안 프레임이 바뀌지 않고, 화면의 「사용시간 0:00:01」도 그대로다(paints 는 계속 는다). 원인은 모른다 — worklog 후속 제안.
- 실행으로 확인한 호출부는 `0xd82c` 1곳(Enumeration)이다. `0x260f4`·`0x26df4` 는 디스어셈블로만 봤다.
- 수신자가 메서드를 구현하지 않으면 워드를 0 으로 두어 게스트가 import `0x40` 을 부르게 한다 — `0x40` 은 아직 미등재라 그 경우 `Unknown lgt java import: 0x40` 로 드러난다(조용히 넘어가지 않는다).

### 회귀

- 네 게이트 + beta: fmt · clippy stable/wasm32/beta `-D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 51 스위트 **592 passed 0 failed**(자기 target `wie-2/target`).
- 러너(AGENTS §The four gates): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt --inject --expect-last-frame` PASS · paints 55 · rc=0 · `text_j2me --timeout 5` PASS(load1 ~155).
- 이 코드는 import 0x64 가 해석될 때만 돈다 — 전에는 그 해석이 곧 치명 오류였고, 전수 조사에서 0x64 로 죽은 타이틀은 0 이었다(위). 다른 타이틀의 기존 판정을 바꿀 경로는 없다.

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

유입 27건(BOUNDED) + 판단 필요 5건(SUFFIX-ATTACHED). 도구는 바뀐 파일의 본문 전체를 센다 — `interface.rs`·`jvm_support.rs`·`svc_ids.rs` 의 제목은 전부 원래 있던 주석이다(이 회차의 Rust 주석에는 제목이 없다).
이 회차가 새로 쓴 제목은 `월드장기체스`(이 문서 · worklog) 하나다. SUFFIX-ATTACHED 중 `월드장기체스가`(이 문서 · worklog — 2쌍)는 조사 붙은 실제 언급이고,
나머지 3쌍은 Rust 파일에 원래 있던 주석의 «더 긴 다른 제목»이다. player-updates 항목은 제목 대신 compat sha256 만 쓴다.

<!-- corpus-name-inflow v1 subjects=6 tree=5f22883a029d4bfd B=58/27 P=0/0 S=10/5 -->
