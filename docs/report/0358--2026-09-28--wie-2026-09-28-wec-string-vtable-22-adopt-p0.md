## [2026-09-28] java/util/Stack 33 = pop()Ljava/lang/Object; — 월드장기체스가 첫 화면을 그린다 (wie-2026-09-28-wec-string-vtable-22-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Stack 블록에 `pop()Ljava/lang/Object; = 33` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: String 22(#383 · 0351) 뒤 월드장기체스가 3/3 `java/util/Stack vtable index 33` 에서 멈췄다(paints 5).

**사용자 영향**: 월드장기체스가 처음으로 부팅을 넘어 첫 안내 화면(요금 안내문)을 그리고 20초 동안 멈추지 않는다.
키를 한 번 누르면 다음 벽(`java/util/Vector` 19)에서 멈추므로 아직 플레이는 안 된다.

### 33 이 Stack 자기 메서드 pop 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

0351 과 같은 방식: missing-entry 스텁이 lr·r0~r3 과 lr-0x100 부터 0x200 바이트를 오류 문구에 싣게 하고 capstone ARM 모드(skipdata)로 풀었다.
실측: lr=0x15368 · r0=0x48846800(Stack) · r1=0x48849fa0 · r2=0x5 · r3=0x4d85d600.

```
0x15318 push {r4,r5,r6,fp,ip,lr,pc} ; mov r4,r0 ; … ; mov r6,r1          ; r1 → r6 (함수 자기 인자)
0x15340 ldr r3,[r4,#8] ; ldr r5,[r3,#0x20] ; cmp r5,#0 ; beq …           ; Stack = this.<field>+8 → +0x20 · null 검사
0x15354 mov r0,r5 ; ldr r3,[r5] ; ldr ip,[r3,#0x88] ; bx ip               ; Stack.<33>() — 0x88 = 4 + 4×33 · r1 은 안 씀
0x15368 subs r5,r0,#0 ; beq …                                            ; ← lr · 결과 null 검사
0x15370 … bx r3 (클래스 검사) ; cmp r0,#0 ; ldreq … (ClassCastException) ; 결과를 게스트 클래스로 캐스트
0x153b4 ldr r3,[r1,#8] ; ldr r1,[r3] ; ldr r5,[r3,#4] ; ldr r4,[r3,#8] … ; 꺼낸 객체의 필드들을
0x153f8 … mov r0,r6 ; … ldr ip,[r2,#4] ; bx ip                           ; 인자 객체의 메서드에 넘긴다 — 저장된 상태 복원
```

바로 위 함수(0x152c4~0x152f4)는 같은 모양의 필드(`[r8,#8]` → +0x20)에서 Stack 을 꺼내 `[r3,#0x84]`(= 32, 핀된 push)로 **인자 하나를 push** 한다.

- 인자 없음(r1 은 이 함수의 인자가 남은 값 — 호출 직전에 다시 쓰지 않는다). 반환: 객체 — null 검사 후 게스트 클래스로 캐스트된다.
- **Vector 칸이 아니다**: Vector 는 CLDC 1.1 순서로 copyInto(10)~removeAllElements(31)이고 핀된 행(15·16·23·24·27·28·29·31)이 전부 그 자리에 맞는다. Stack 자기 메서드는 핀된 push(32)부터 시작한다.
- 후보(인자 없음 · 객체 반환): `pop`(33) · `peek`(34). **모양으로는 둘을 가르지 못한다** — 순서가 가른다.
  보조 근거: 바로 위 함수가 같은 Stack 에 push 하고 이 함수가 꺼내 복원하는 save/restore 짝이다. peek 이라면 저장된 상태가 Stack 에서 빠지지 않는다.
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/stack.rs:21`)이 `pop ()Ljava/lang/Object;` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --timeout 20` · release)

| 트리 | result | ticks | paints | stop | 첫 예외 |
|---|---|---|---|---|---|
| main `766ce2a6`(계측만) ×1 · load1 129 | FAIL | 85841 | 5 | error | `java/util/Stack vtable index 33` |
| 이 브랜치 ×3 · load1 111~118 | **PASS** | 17399934 / 17451348 / 11268918 | 920 / 953 / 876 | deadline | 없음 |
| 변이: pop 행만 제거 | — | — | — | — | 핀 테스트 FAILED(«Stack vtable index 33 is empty») |

후 3판 모두 `content true` · `last_frame_content true` · distinct_colors 4 · nondominant 23.8% — 첫 안내 화면이 그려진 채 유지된다.

### 다음 벽 (`--inject` · 1회)

`wie_validate --inject` → FAIL · stop error · input_steps 1/27 · paints 109 · `Unimplemented: java/util/Vector vtable index 19`.
첫 키 하나에 멈춘다. 같은 실행의 `java_exceptions` 첫 항목 `java/io/FileNotFoundException: File not found` 는 잡힌 예외다(첫 실행 저장 파일 부재 모양 — 이 회차에서 확인하지 않았다).
worklog 후속 제안 1.

### 한계

- 호출부 1곳(한 타이틀). pop/peek 은 모양이 같아 순서 + save/restore 짝으로 판정했다.
- Stack push(32) 행은 여전히 핀 테스트에 없다(이 회차 범위 밖 — 33 만 핀).
- player-updates 는 추가하지 않았다 — 헤드리스 판정은 PASS 가 됐지만 첫 키에서 멈춰 이용자에게는 아직 «플레이 불가»다.
  첫 키 벽이 넘어가는 회차가 쓸 몫이다.

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

유입 BOUNDED + 판단 필요 SUFFIX-ATTACHED 수는 아래 표식 줄이 정본이다(B=바구니 총/파일 · S=같음). 도구는 바뀐 파일의 본문 전체를 센다 —
`jvm_support.rs`·`lgt_java_abi.toml` 의 제목 언급은 대부분 원래 있던 주석이다. 이 회차가 새로 더한 제목은 `월드장기체스` 하나
(ABI 주석 1 · 핀 테스트 주석 1 · 이 문서 · worklog)이고 선례(0344·0351)와 같은 자리다. SUFFIX-ATTACHED 는 두 종류다:
`월드장기체스가`(이 회차가 쓴 조사 붙은 언급 — 실제 언급)와 `간호사타이쿤2`·`서든어택포켓`(«더 긴 다른 제목»이 든 원래 있던 주석).
<!-- corpus-name-inflow v1 subjects=4 tree=33f798228b320fdb B=101/32 P=0/0 S=21/8 -->
