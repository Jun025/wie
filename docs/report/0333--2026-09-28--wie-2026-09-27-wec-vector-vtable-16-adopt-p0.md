## [2026-09-28] java/util/Vector 16 = isEmpty()Z — 월드장기체스 벽이 Vector 24 로 이동 (wie-2026-09-27-wec-vector-vtable-16-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Vector 블록에 `isEmpty()Z = 16` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: Vector 29(#0319) 뒤 월드장기체스가 3/3 `java/util/Vector vtable index 16`(스택 d.paint)에서 멈췄다.

**사용자 영향**: 월드장기체스가 한 칸 더 가지만 화면은 아직 없다(다음 벽 `java/util/Vector` 24).

### 16 이 isEmpty()Z 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

#0311·#0319 와 같은 방식: missing-entry 스텁이 lr·r0..r3 과 lr-0x280 부터 0x300 바이트를 오류 문구에 싣게 하고 ARM 모드로 풀었다.
실측: lr=0x21de0 · r0=0x488468d0(Vector) · r1=r2=0x488468c0.

```
0x21db8 ldr r2,[fp,#-0x24] ; ldr r3,[r2,#8] ; ldr r5,[r3,#0x18]  ; r5 = this.<필드>
0x21dc4 cmp r5,#0 ; beq 0x21e70                                ; null 이면 예외 경로
0x21dcc mov r0,r5 ; ldr r3,[r5] ; ldr ip,[r3,#0x44] ; bx ip    ; Vector.<16>() — 0x44 = 4 + 4×16, r0 외 인자 없음
0x21de0 cmp r0,#0 ; bne 0x21eb8                                ; ★결과가 분기 조건
0x21de8 … ldr r6,[r3,#0x18] ; cmp r6,#0 ; beq …                ; 같은 필드를 다시 읽고 null 검사
0x21dfc mov r0,r6 ; ldr r3,[r6] ; ldr ip,[r3,#0x64] ; bx ip    ; 0 쪽에서만 Vector.<24>() — 인자 없음
0x21e10 subs r6,r0,#0 ; beq … ; ldr r3,[r6] ; ldr r4,[r3]      ; 반환 객체의 클래스를 읽어 타입 검사
```

- 인자: 없음. 반환: 0/비0 으로 분기한다 — 값이 쓰인다.
- 후보(CLDC 1.1 Vector 의 인자 없는 비-void): `capacity()I`(14) · `size()I`(15) · `isEmpty()Z`(16) · `elements()`(17) ·
  `firstElement()`(24) · `lastElement()`(25). 객체 반환 셋은 분기 뒤에 객체를 다루지 않으므로 빠진다.
- 남는 셋을 **호출부가 가른다**: 0 이 나온 경로만 24 번(firstElement 자리)으로 원소를 꺼낸다.
  size()·capacity() 를 0 과 비교한 것이라면 «비어 있을 때» firstElement 를 부르게 된다(NoSuchElementException).
  isEmpty() 면 «비어 있지 않을 때만» 첫 원소를 꺼낸다 — 이 모양만 성립한다. size 는 이미 15 로 등재·실측돼 있다.
- CLDC 1.1 선언 순서도 isEmpty 를 16(size 바로 다음)에 둔다 — 호출부 판정과 일치.
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/vector.rs:42`)이 `isEmpty ()Z` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --timeout 20` · release · loadavg 220~340)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `bb4d8931`(계측만) ×1 | FAIL | 11158 | 0 | `java/util/Vector vtable index 16` |
| 이 브랜치 ×3 | FAIL | 11 / 20048 / 5956 | 0 | `java/util/Vector vtable index 24` |
| 변이: isEmpty 행만 제거 | — | — | — | 핀 테스트 FAILED(`jvm_support.rs:1851` «Vector vtable index 16 is empty») |

ticks 편차는 부하 때문이다(loadavg 220~340) — 판정은 첫 예외가 바뀐 것이다(3/3 동일). 새 벽 24 는 위 디스어셈블의
0x21dfc 가 부르는 바로 그 자리라, 16 을 채운 뒤 실행이 그 다음 호출에 닿았다는 교차 확인이기도 하다.

### 한계

- 호출부 1곳(한 타이틀).
- 다음 벽 Vector 24 는 등재하지 않았다(범위 밖) — worklog 후속 제안. 인자 0 · 객체 반환 후보(elements 17 · firstElement 24 · lastElement 25)가 모양으로 안 갈린다.

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

유입 32건(BOUNDED) + 판단 필요 8건(SUFFIX-ATTACHED). 도구는 바뀐 파일의 본문 전체를 센다 — `jvm_support.rs`·`lgt_java_abi.toml`
의 제목 언급은 대부분 원래 있던 주석이다. 이 회차가 새로 더한 제목은 `월드장기체스` 하나(ABI 주석 1 · 핀 테스트 주석 1 · 이 문서 · worklog)이고,
선례(#0311·#0319)와 같은 자리다. SUFFIX-ATTACHED 8건은 `간호사타이쿤2`·`서든어택포켓` — «더 긴 다른 제목»이 든 원래 있던 주석이다.
