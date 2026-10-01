## [2026-10-01] LGT `java/lang/Integer` index 12 = `intValue()I` — 월드장기체스 대국 메뉴 「계속하기」 정지 해소 (wie-wec-continue-integer-vtable-index-12-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` 에 `java/lang/Integer` 행 1개(`intValue()I` = 12)를 더했다. 엔진 코드 변경 0.

**왜**: 0383 이 남긴 다음 벽 — 대국 중 `STAR` 로 메뉴를 열고 `NUM1`(1.계속하기)을 고르면 `d.paint` 안에서 `Unimplemented: java/lang/Integer vtable index 12` 로 끝난다.

**사용자 영향**: 대국 중 메뉴를 열었다가 「계속하기」를 고르면 판으로 돌아와 이어 둘 수 있다(종전: 게임이 멈춤).

### 재현 — 기본 27키는 이제 이 메뉴에 닿지 않는다

- main `1becaae2`(#416 타이머 · #426 착지 뒤) 기본 `--inject` 4판은 **4/4 PASS · 27/27 · 오류 없음** — 0383 의 「4/4 key 25 에서 Integer 12」는 재현되지 않았다. 캡처: `23_STAR` 가 메뉴를 열지 않고 `25_NUM1` 이 「한 수 쉬시겠습니까?」 모달을 연다 — 같은 27키가 판 위 다른 상태에 떨어진다.
- 그래서 경로를 `docs/keys/wec-match-menu-continue.keys` 로 적었다(연습 → 장기연습 → 상차림 1 → 판 → `STAR` → `NUM1`, 이어 커서 이동 → 메뉴 → 계속하기 한 번 더). main 에서 **2/2** `19_NUM1` 에서 `Unimplemented: java/lang/Integer vtable index 12`.
- 미구현 경고 줄: `java/lang/Integer vtable index 12: this=0x488551f0 r1=0x4904f180 r2=0x4d863200 r3=0x4a85aa00 lr=0x3a8e0`.

### 12 가 어느 메서드인가 — 호출부 디스어셈블(binary.mod · `objdump --triple=armv5te` · .text 0x1000)

호출 함수 `0x3a834`(`this` 하나):

```
3a854 ldr r3,[r6,#0x8] ; ldr r5,[r3,#0x14]      ; s = this.<f8>.<0x14>, null 이면 예외
3a864 ldr r3,[r5] ; ldr ip,[r3,#0x88] ; bx ip    ; s.<33>()  = Stack.pop (ABI 행 33)
3a874 subs r5,r0,#0 ; beq …                      ; 결과 null 검사
3a87c … 0x3a8a8                                  ; 클래스 비교 → 실패면 0x3a8fc 경로(캐스트 예외)
3a8cc ldr r3,[r5] ; mov r0,r5 ; ldr ip,[r3,#0x34] ; bx ip   ; o.<12>()  — r0 만 설정
3a8e0 ldr r3,[r6,#0x8] ; str r0,[r3]              ; this.<f8>.<0> = 결과 한 워드
```

짝 함수 `0x3a904`(`this, v`):

```
3a928 r6 = this.<f8>.<0x14>                       ; 같은 Stack
3a934 … 3a948 → r4 = 새 객체
3a94c ldr r1,[this.<f8>,#0]  ; 생성자류 호출(r0 = r4, r1 = 현재 값)
3a984 r6.<32>(r4)  (`[r3,#0x84]` = Stack.push)
3a99c this.<f8>.<0> = v
```

⇒ `stack.push(new Integer(cur)); cur = v;` 와 `cur = ((Integer) stack.pop()).intValue();` — 같은 int 필드를 감쌌다 푼다.
- 인자 없음(r0 만) · 결과 **한 워드**를 int 필드에 저장 ⇒ `longValue`·`doubleValue`(두 워드)와 Object 행(hashCode 2 등은 순번이 다르다) 배제.
- CLDC 1.1 `Integer` 는 Object 직계라 자기 메서드가 10 부터: `byteValue 10 · shortValue 11 · intValue 12 · longValue 13 · floatValue 14 · doubleValue 15`. 순서도 12 = `intValue`.
- 모양만으로 못 가르는 것: `byteValue`/`shortValue`/`floatValue` 도 한 워드다. int 필드 왕복이라는 용도와 순서가 `intValue` 를 고른다(틀렸다면 0..127 밖의 값에서만 드러난다).
- 런타임 수신자 클래스명이 `java/lang/Integer`(위 경고 줄)라 호출부 해석과 맞는다.

### 실측 — release `wie_validate`, 전(main `1becaae2`)·후 짝 동시 실행

| 트리 · 경로 | result | stop | 키 | frozen_tail | paints | 끝 |
|---|---|---|---|---|---|---|
| 전 · 키 경로 ×2(load1 99) | FAIL ×2 | error | 11/15 | 1 | 1855 / 2765 | `19_NUM1` 에서 Integer index 12 |
| 후 · 키 경로 ×2(load1 99) | **PASS** ×2 | deadline | **15/15** | 0 | 2773 / 3783 | 판 복귀 · 메뉴 재진입·재계속 정상 |
| 전 · 기본 `--inject` ×4(load1 27~37) | PASS ×4 | deadline | 27/27 | 0 | 618~787 | 메뉴에 닿지 않음 |
| 후 · 기본 `--inject` ×4(같은 짝) | PASS ×4 | deadline | 27/27 | 0 | 610~787 | 같음 |

- 후 · 키 경로 캡처: `18_WAIT` 대국 메뉴(1.계속하기 … 4.나가기 · 연습경기 VS 중수급 AI) → `19_NUM1` 판 복귀 → 커서 이동 → 다시 메뉴 → 계속하기 → 판. 사용자 시계 0:00:19 → 0:00:31 로 흐른다.
- 기본 스크립트는 전·후 차이가 없다 — 이 회차 변경은 그 27키 범위의 동작을 바꾸지 않는다.
- 첫 `java_exceptions` 는 전·후 모두 `FileNotFoundException`(첫 실행 저장 파일) 1건.

### 시험(되돌리면 red)

`abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 `("java/lang/Integer", 12, "intValue", "()I")` 를 더했다. `every_abi_vtable_row_is_the_slot_linking_resolves_to` 는 파일 전체를 걸어 새 행도 링크 index·원시 표 대상과 대조한다(ok).
- 변이(실측): TOML 행을 지우면 `abi_rows_cover…` FAILED — `java/lang/Integer vtable index 12 is empty`. 복원 후 ok.

### 회귀

- 네 게이트 + beta: `cargo fmt --check` rc=0 · `clippy --all -D warnings` rc=0 · wasm32 clippy rc=0 · `+beta clippy` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 51 스위트 **626 passed 0 failed**(스크래치 자기 target — 공유 `target/` 에서는 다른 프로세스와 incremental 이 겹쳐 rustc ICE(LTO coordinator panic)가 났고, 코드와 무관하다).
- 러너(AGENTS §The four gates): `draw_j2me`·`helloworld_ktf`·`helloworld_lgt` PASS · `keydraw_ktf`(paints 79)/`keydraw_lgt`(paints 55) `--inject --expect-last-frame` PASS · `text_j2me --timeout 5` PASS(load1 ~38).

### 한계

- `byteValue`/`shortValue`/`floatValue` 와의 구별은 순서·용도 근거다(위).
- 장기 상차림에서 AI 시계는 0:00:00 그대로였다 — 사용자가 수를 두지 않는 경로라 AI 차례는 이 경로에서 보지 않았다.

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs`)

유입 31건(BOUNDED) + 판단 필요 4쌍 14회(SUFFIX-ATTACHED). 도구는 바뀐 파일의 본문 전체를 센다 — `lgt_java_abi.toml`·`jvm_support.rs` 의 제목 대부분은 원래 있던 주석이다.
이 회차가 새로 쓴 제목은 `월드장기체스`(이 문서 · worklog · keys 파일 · `lgt_java_abi.toml` 주석 · 시험 주석) 하나다.
SUFFIX-ATTACHED 4쌍(`간호사타이쿤`→`간호사타이쿤2` · `서든어택`→`서든어택포켓`)은 전부 바뀐 파일에 원래 있던 주석의 «더 긴 다른 제목»이고 이 회차가 쓴 언급이 아니다.
player-updates 항목은 제목 대신 compat sha256 만 쓴다.
