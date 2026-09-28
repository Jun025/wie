## [2026-09-28] java/util/Vector 24 = firstElement()Ljava/lang/Object; — 월드장기체스 첫 화면(paints 0 → 5) · 벽이 String 22 로 이동 (wie-2026-09-28-wec-vector-vtable-16-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Vector 블록에 `firstElement()Ljava/lang/Object; = 24` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: Vector 16(#367 · 0333) 뒤 월드장기체스가 3/3 `java/util/Vector vtable index 24` 에서 멈췄다.

**사용자 영향**: 월드장기체스가 처음으로 프레임을 그린다(paints 0 → 5). 아직 플레이는 안 된다(다음 벽 `java/lang/String` 22).

### 24 가 firstElement 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

0333 과 같은 방식: missing-entry 스텁이 lr·r0 과 lr-0x40 부터 0x180 바이트를 오류 문구에 싣게 하고 capstone ARM 모드로 풀었다.
실측: lr=0x21e10 · r0=0x48846900(Vector).

```
0x21de0 cmp r0,#0 ; bne 0x21eb8                                ; isEmpty()(16) 가 거짓일 때만 아래로
0x21de8 ldr r5,[fp,#-0x24] ; ldr r3,[r5,#8] ; ldr r6,[r3,#0x18] ; 같은 Vector 필드 · null 검사
0x21dfc mov r0,r6 ; ldr r3,[r6] ; ldr ip,[r3,#0x64] ; bx ip    ; Vector.<24>() — 0x64 = 4 + 4×24, r0 외 인자 없음
0x21e10 subs r6,r0,#0 ; beq 0x21e50                            ; null 이 아니면
0x21e18 ldr r3,[r6] ; ldr r4,[r3] ; … bx r3 ; … bx r3          ; 반환 객체의 클래스로 게스트 클래스 캐스트 검사
0x21e44 cmp r0,#0 ; beq → 예외 경로                             ; 캐스트 실패 = ClassCastException
0x21e5c str r6,[r3,#0x28]                                      ; this.<필드>+0x28 = 꺼낸 객체
0x21e60 ldr r5,[r2,#0x18] ; mov r6,#0 ; …                      ; 같은 Vector 다시 · null 검사
0x21e7c mov r0,r5 ; mov r1,r6(=0) ; ldr ip,[r3,#0x70] ; bx ip  ; Vector.<27>(0) — 0x70 = 4 + 4×27 = removeElementAt(I)V (이미 핀)
```

- 인자: 없음. 반환: 객체 — 게스트 클래스로 캐스트돼 필드에 저장된다.
- 후보(CLDC 1.1 Vector 의 인자 없는 객체 반환): `elements()`(17) · `firstElement()`(24) · `lastElement()`(25).
- **호출부가 가른다**: 꺼낸 직후 같은 Vector 에서 **인덱스 0** 을 지운다(`removeElementAt(0)`) — `if (!v.isEmpty()) { x = (T) v.firstElement(); v.removeElementAt(0); }` 의 큐 pop 이다.
  lastElement 라면 꺼낸 것과 지운 것이 다른 원소가 된다. elements() 의 Enumeration 은 런타임 클래스라 게스트 클래스 캐스트를 통과하지 못한다.
- CLDC 1.1 선언 순서도 firstElement 를 24 에 둔다 — 이미 핀된 elementAt(23)·removeElementAt(27) 사이. 호출부 판정과 일치.
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/vector.rs:52`)이 `firstElement ()Ljava/lang/Object;` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --timeout 20` · release · loadavg 290~520)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `1dd9f81c`(계측만) ×1 | FAIL | 1348 | 0 | `java/util/Vector vtable index 24` |
| 이 브랜치 ×3 | FAIL | 80937 / 32455 / 43096 | **5 / 5 / 5** | `java/lang/String vtable index 22` |
| 변이: firstElement 행만 제거 | — | — | — | 핀 테스트 FAILED(«Vector vtable index 24 is empty») |

ticks 편차는 부하 때문이다 — 판정은 첫 예외가 바뀐 것(3/3 동일)과 paints 가 0 에서 5 로 선 것이다.

### 한계

- 호출부 1곳(한 타이틀).
- 다음 벽 String 22 는 등재하지 않았다(범위 밖) — worklog 후속 제안.
- player-updates 는 추가하지 않았다 — 타이틀은 여전히 FAIL(선례 #353·#367 과 같은 판단).
