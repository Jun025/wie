## [2026-09-27] 놈3 «같은 프레임 4만 번 unwind → 스택 넘침»은 origin/main 에서 재현 0 — ec28e2fe 가 닫았다 · 호출부 시험 추가 (wie-2026-09-26-battlemonster-field-input-scene-thread-death-adopt-p0)

**무엇을**: 추천 `2026-09-26-battlemonster-field-input-is-scene-thread-death#p0`(#292 위에서 놈3 이 `--boot-secs 15 --action-secs 1.2` 일정에서 main 스레드 스택 넘침)을 현 origin/main 으로 재측했다. 재현 0 — 코드 수리 없이 측정으로 닫고, 그 수리의 호출부를 지키는 시험 1건을 더했다.
**왜**: 증적의 overflow 는 4646c3d8(#292 초기 head)에서 쟀다. 같은 #292 계열의 뒤 커밋 ec28e2fe(fix3)가 바로 이 trace 를 고쳤다 — 그 커밋의 `release_to` 단위 시험이 증적 trace 의 lr 셋(0x21a07·0x20b87·0x1f867)을 그대로 쓴다. #292(`7bac9acd`)는 ec28e2fe 를 담아 착지했다.
**사용자 영향**: 없음(시험만 추가). 놈3 은 해당 일정·기본 일정 모두 통과한다.

### 측정 — 놈3 · 같은 명령 · release(LTO)
| 빌드 | 결과 | 비고 |
|---|---|---|
| 4646c3d8(증적 원본) | **stack overflow 2/2**(rc=134) | 대조 재현 |
| ec28e2fe | PASS 2/2 | |
| origin/main `156c5c56` | **PASS 4/4** · 27/27 키 · overflow 0 | load 35~245 에서 두 창 |
| main 기본 일정(`--inject --expect-last-frame --timeout 300`) | PASS 2/2 · `last_frame_content true` | |
| main − String 26 행(가드 유지 · 트리거 존재) | **PASS 4/4** · String 26 hit 4/4 | 가드 단독으로 견딘다 |
| main − String 26 행 − 가드 | **FAIL 3/4** · `guest heap exhausted` → `JavaException` panic(입력 20~24) | 개악 red |
| main − 가드(String 26 구현 유지) | PASS 2/2 | 트리거가 사라져 가드가 안 불린다 |
| 824f07f3(ec28e2fe 부모) | PASS 2/2 | load 156 — 타이밍 의존 · 원인 귀속에 쓰지 않는다 |

기전(ec28e2fe 커밋 본문과 일치): Rust 가 부른 `paint` 가 세 단 깊이에서 `Unimplemented: java/lang/String vtable index 26` 으로 중단되면 그 호출들이 push 한 프레임이 체인에 남고, 호출자의 throw 가 죽은 프레임으로 unwind — 재시도 고리가 매 회 새 예외를 만들며 게스트 힙을 소진한다. 지금은 그 끝이 overflow 가 아니라 `guest heap exhausted`(6c9bdeb7 이 재귀를 1회에서 끊는다)라서 가드를 빼면 overflow 대신 FAIL 로 드러난다. 같은 커밋이 String 26 = `indexOf(S,I)` 도 구현해 **트리거 자체가 사라졌다** — 그래서 현 main 에서 가드만 빼면 PASS 이고, 가드의 값은 트리거를 되살려야(String 26 행 제거) 보인다.

### 시험 공백 — 실측
`JavaMethod::run` 의 `mark`/`release_to` 호출부를 bare `run_function` 으로 되돌려도 기존 **495 시험 전부 green**이었다(`release_to` 는 단위로만 시험됨). 새 시험 `host_error_in_rust_invoked_guest_call_does_not_leave_its_frames_on_the_chain`(wie-lgt `jvm_support/method.rs`): 게스트가 try 두 개에 들어간 뒤 호스트 오류로 중단되는 SVC 를 `JavaMethod::run` 으로 부르고, 그 뒤 호출자 sp 의 throw 가 호출자 프레임(lr 0x4001)에 닿는지 본다. 호출부 제거 변이 → `left: Some(0x6001) right: Some(0x4001)` **red**.

### 게이트
fmt rc0 · clippy `-D warnings` stable/wasm32/beta rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **496 passed / 0 failed**(배틀몬스터 ARM·현영맞고/놈3 Thumb 시퀀스 시험 `catch_that_*`·`cx_thumb_*`·`rethrow_*`·`release_to_*` 포함 green). unwind 규칙은 건드리지 않았다.

증적: `~/orchestrator/reports/evidence/wie-2026-09-26-battlemonster-field-input-scene-thread-death-adopt-p0/`(README 에 빌드 대응표).

### 게임 파일명 유입
`scripts/corpus-name-inflow.mjs` 실행값: 유입 4쌍(BOUNDED — 놈3·배틀몬스터, 이 리니지가 이미 제목·시험 주석에 쓰는 이름) · 판단 필요 0건(SUFFIX-ATTACHED). 게임 바이트·키 스크립트 커밋 0.
