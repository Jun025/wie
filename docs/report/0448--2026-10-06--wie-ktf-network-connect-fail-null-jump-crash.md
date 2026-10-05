## [2026-10-06] KTF finally 가 다시 던진 예외가 자기 핸들러로 되돌아오던 무한 루프 — 연결 실패 뒤 멈춤 1종 (wie-ktf-network-connect-fail-null-jump-crash)

제안 `2026-10-05-progression-wave3-r2#p1` 채택. 대상 `dbd078113b97`(KTF): 시작 직후 랭킹 메뉴에서 OK → `Network.connect()` = -1 → NPE → (옛 빌드) `jump native address is null` · (현 main) 화면 정지.

### 1. 원인 계급 — 엔진 벽(예외 디스패치)이다. `connect()` 의 -1 은 옳다

**-1 은 게임이 기다리는 실패 값이다 — 바이트코드 근거.** `connectToServer()Z`(이미지 `0x12ef60`) 는
`net = new Network(); r = Network.connect(); if (r == -1) { this.<상태> = 3; return false; }` 이다
(`0x12efe8` `bl` connect → `0x12efee..0x12eff6` `movs r3,#1; rsbs r3,r3,#0; cmp r0,r3; bne`) — 실패를 **-1 로 비교**한다.
그러니 스텁의 `Ok(-1)` 은 바꾸지 않았다. 예외로 알렸다면 이 비교가 있을 이유가 없다.

**NPE 는 게임 자신의 것이다.** `sendData([C)Z`(메서드 `0x14ce5c`) 는 `connectToServer()` 의 false 를 버리고 소켓 필드에 대고 호출한다 —
호출 도우미 `0x135ca8` 의 null 검사가 `java_throw("java/lang/NullPointerException")` 를 낸다(호출 지점 `0x12fece`, 실측 호출 스택).
실기에서 연결이 실패해도 같은 NPE 가 난다. 문제는 그 뒤다.

**엔진 벽.** `sendData` 의 예외 표(런타임 실측):

| from | to | target | class |
|---|---|---|---|
| 0x2 | 0x35 | 0x41 | (어떤 예외 클래스 — NPE 아님) |
| 0x2 | 0x35 | 0x50 | any (`finally`) |
| 0x41 | 0x44 | 0x50 | any |

NPE 는 `finally`(0x50) 로 잡히고, 그 본문은 `e` 를 다시 던진다(`0x12ffa2` `bl 0x1345a8` = `java_throw_instance`).
그런데 핸들러 레코드의 `current_pc` 는 여전히 try 범위 안(0x2)이라 다시 던진 예외가 **같은 레코드의 같은 0x50** 에 또 잡혔다 — 끝없는 throw/catch.

★실기가 무엇을 했는지는 컴파일된 코드가 말한다: catch 블록(target 0x41, `0x12ff5e`)은 들어오자마자 `current_pc = 0x41` 을 **스스로** 쓴다
(그 범위 `[0x41,0x44)` 를 `finally` 가 덮으니 필요하다). 0x50 의 `finally` 는 아무것도 쓰지 않는다 — 덮는 범위가 없으니까.
즉 레코드는 catch 뒤에도 등록된 채이고(풀었다면 `[0x41,0x44)` 행이 무의미하다), `finally` 의 재던짐이 빠져나가려면 런타임이 `current_pc` 를 범위 밖(= 핸들러 위치)으로 옮겨야 한다.

### 2. 고친 것

`JavaMethod::handle_exception` 이 잡는 순간 레코드의 `current_pc` 에 `entry.target` 을 쓴다(`wie-ktf/src/runtime/java/jvm_support/method.rs`). 한 줄.
이제 `finally` 의 재던짐은 그 프레임에 맞는 행이 없어 `ptr_old_handler` 로 호출자 프레임을 걷는다(0445 의 사슬 걷기).

### 3. 측정

| 항목 | base(`9f8b0ae2`) | head |
|---|---|---|
| 개악 red — `test_catch_handler_receives_thrown_exception` 새 단언 | **FAIL** (`current_pc` left 5, right 0x1dd) | PASS |
| `java_throw_instance` 수 · 키 6개 20초 | **1,648,751** | **1** |
| P 정책 한 바퀴(키 26) 60초 · `paints` | 66 (`05_OK` 뒤 프레임 md5 4장 동일 = 정지) | 599 (프레임 계속 바뀜) |
| 화면 | 랭킹 «접속 시 통화료» 확인창에서 정지 | 게임 자신의 «통신장애입니다 · 재시도해주세요 · ok» → 랭킹 메뉴로 복귀 |

명령(scratch, 게임 파일은 `game_lab/`): `wie_validate --inject --keys <P.keys 첫 줄> --keep-timeout --timeout 60 --shotdir … --shot-every 5 <title>`.
base·head 를 같은 분에 나란히 돌렸다(load1 ~20). long 풀이 다른 레인 스윕에 2시간 넘게 잡혀 있어 1분짜리 단발 짝으로 short 풀에서 쟀다.

**다른 KTF 통신 타이틀 표본.** census 출력에서 `Network::connect` 를 부른 나머지 2종(`5bff8d2168f7` · `b1ec149b354c`), 기본 27키 60초 base↔head:
둘 다 PASS · 27/27 · paints 1025↔1030 · 1678↔1672 · `java_throw`/`java_throw_instance` 0 — -1 을 예외 없이 처리하는 타이틀이라 이 수정의 영향이 없다.

게이트: fmt · clippy(stable·wasm·beta) · `RUST_MIN_STACK=4194304 cargo test --all` rc=0. 러너 블록: draw/helloworld×2/text PASS.
`keydraw_*` 는 base·head **같이** `UNMEASURED`(`stop: max-ticks`, 5/27) — 이 diff 와 무관(LGT 포함). `--max-ticks 100000000000` 이면 둘 다 PASS rc=0.

### 4. 남김

- `compat.json` 은 건드리지 않았다. 이 행의 `progress: stuck` 은 정책 600초 짝 측정으로만 바꿀 수 있고, 그 측정은 long 풀 몫이다(다음 진도 회차).
- 인접 `30c7bd6fb01b`(KTF Net 표 슬롯 34 `Unimplemented`)는 **다른 축**이다 — WIPI-C 네이티브 인터페이스 표이고 Java 예외 디스패치가 아니다. 이 회차는 판정만 하고 남겼다.

### 5. 게임 파일명 유입

- `corpus-name-inflow`: BOUNDED 1 + SUFFIX-ATTACHED 0. 그 1건은 이 PR 이 건드린 파일의 기존 줄(`jvm_support.rs:1012`, main 에 이미 있다)이고, 이 PR 이 더한 줄에서는 0 이다.

<!-- corpus-name-inflow v1 subjects=5 tree=ff0dba90a0b8d158 B=1/1 P=0/0 S=0/0 -->
