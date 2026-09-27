## [2026-09-27] java/util/Vector 29 = addElement(Object)V — 월드장기체스 벽이 Vector 16 으로 이동 (wie-2026-09-27-wec-vector-vtable-29-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Vector 블록에 `addElement(Ljava/lang/Object;)V = 29` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: Vector 31(#0311) 뒤 월드장기체스가 3/3 `java/util/Vector vtable index 29` 에서 멈췄다.

**사용자 영향**: 월드장기체스가 한 칸 더 가지만 화면은 아직 없다(다음 벽 `java/util/Vector` 16).

### 29 가 addElement(Object)V 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

#0311 과 같은 방식: missing-entry 스텁이 lr·r0..r3 과 lr-0x280 부터 0x300 바이트를 출력하게 하고 ARM 모드로 풀었다.
실측: lr=0x2b598 · r0=0x48847180(Vector) · r1=0x4884a240 · Java 스택 `d.paint(Lorg/kwis/msp/lcdui/Graphics;)V`.

```
0x2b538 mov ip,sp ; push {r4,r5,fp,ip,lr,pc}         ; 인자 1개짜리 게스트 메서드(this=r0, arg=r1)
0x2b544 mov r4,r0 ; … mov r5,r1                      ; 인자를 r5 에 보관
0x2b55c ldr r3,[r4,#8] ; mov r1,r5 ; ldr r4,[r3]      ; r4 = this.<필드>[0], r1 = 그 인자
0x2b568 subs r0,r4,#0 ; bne 0x2b584                   ; null 이면 예외 경로
0x2b584 mov r0,r4 ; ldr r3,[r4] ; ldr ip,[r3,#0x78] ; bx ip   ; Vector.<29>(arg) — 0x78 = 4 + 4×29
0x2b598 mov r0,r5                                     ; 호출 결과를 읽지 않고 덮어쓴다 — 인자를 그대로 반환
0x2b59c ldm … ; bx lr
```

- 인자: 객체 1개(r1 = 게스트 메서드가 받은 인자. 값이 Vector 와 같은 힙 대역).
- 반환: 쓰이지 않는다 — 바로 다음 명령이 r0 를 덮어쓴다. 「원소를 넣고 그 원소를 돌려준다」 모양.
- 후보(CLDC 1.1 Vector 의 객체 1개 인자): `copyInto([Object)V`(10) · `contains(Object)Z`(18) · `indexOf(Object)I`(19) ·
  `lastIndexOf(Object)I`(21) · `addElement(Object)V`(29) · `removeElement(Object)Z`(30).
  선언 순서가 29 를 addElement 로 가리키고, 기존 5행(15·23·27·28·31)이 전부 그 순서 위치에 있다.
- **반증 가능성**: 결과가 쓰였다면 void 인 29 는 틀린 것이다 — 쓰이지 않았으므로 반증되지 않았다.
  단 Java 는 boolean 결과를 버릴 수 있어 removeElement(30)도 같은 모양이다. **모양만으로는 갈리지 않고 판정은 순서가 한다.**
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/vector.rs:36`)이 `addElement (Ljava/lang/Object;)V` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --timeout 20` · release · loadavg 290~390)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `84418eea`(계측만) ×1 | FAIL | — | 0 | `java/util/Vector vtable index 29` |
| 이 브랜치 ×3 | FAIL | 49295 / 8368 / 9084 | 0 | `java/util/Vector vtable index 16` |
| 변이: addElement 행만 제거 | FAIL | — | 0 | `java/util/Vector vtable index 29` · 핀 테스트 FAILED(`jvm_support.rs:1849`) |

ticks 가 #0311(60438~63909)보다 작은 것은 퇴보가 아니다 — 이 머신 loadavg 가 그때 40~98 이었고 지금은 290~390 이다.
판정은 첫 예외가 바뀐 것이다(3/3 동일).

### 한계

- 호출부 1곳(한 타이틀). addElement 와 removeElement 는 모양으로 갈리지 않는다 — 판정은 선언 순서다.
- 다음 벽 Vector 16(순서상 `isEmpty()Z`)은 등재하지 않았다(범위 밖) — worklog 후속 제안.

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

도구는 바뀐 파일의 본문 전체를 센다 — `jvm_support.rs`·`lgt_java_abi.toml` 의 제목 언급은 대부분 원래 있던 주석이다.
이 회차가 새로 더한 제목은 `월드장기체스` 하나(ABI 주석 1 · 핀 테스트 주석 1 · 이 문서 · worklog)이고, 선례(#0311)와 같은 자리다.
SUFFIX-ATTACHED 는 `간호사타이쿤2`·`서든어택포켓` 같은 «더 긴 다른 제목»과 원래 있던 주석이다(#0313 이 손으로 가른 것과 같은 줄).
최종 수는 아래 표식이 갖는다.
<!-- corpus-name-inflow v1 subjects=4 tree=ecbf43ccb51f0ac8 B=95/32 P=0/0 S=19/8 -->
