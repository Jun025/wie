## [2026-09-28] java/lang/String 22 = indexOf(II)I — 월드장기체스 · 벽이 Stack 33 으로 이동 (wie-2026-09-28-wec-vector-vtable-24-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` String 블록에 `indexOf(II)I = 22` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: Vector 24(#376 · 0344) 뒤 월드장기체스가 3/3 `java/lang/String vtable index 22` 에서 멈췄다(paints 5).

**사용자 영향**: 월드장기체스가 한 벽을 더 넘는다. 아직 플레이는 안 된다(다음 벽 `java/util/Stack` 33).

### 22 가 indexOf(II) 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

0344 와 같은 방식: missing-entry 스텁이 lr·r0~r3 과 lr-0x80 부터 0x200 바이트를 오류 문구에 싣게 하고 capstone ARM 모드로 풀었다.
실측: lr=0x7ac0 · r0=0x48841c40(String) · r1=0x26 · r2=0x0.

```
0x7a48 ldr r6,[fp,#-0x44] ; ldr r7,[fp,#-0x60] ; cmp r6,#0      ; 같은 String · null 검사 · r7 = from
0x7a54 mov r5,#0x26                                             ; '&'
0x7a5c mov r1,r5 ; mov r0,r6 ; mov r2,r7 ; ldr ip,[r3,#0x5c] ; bx ip   ; String.<22>('&', from) — 0x5c = 4 + 4×22
0x7a78 ldr r4,[fp,#-0x50] ; cmn r0,#1 ; movne r4,r0             ; 결과가 -1 이 아니면 채택
0x7a84 add r5,r4,#1 ; str r5,[fp,#-0x50]                        ; from = i + 1
0x7a90 … mov r5,#0x26 ; … mov r1,r5 ; mov r2,r7 ; ldr ip,[r3,#0x5c]    ; String.<22>('&', from) 두 번째
0x7ac0 cmn r0,#1 ; … bne                                        ; ← lr · 결과를 -1 과 비교
```

- 인자: r1 = 문자 상수 `'&'`(String 아님), r2 = int 시작 위치. 반환: int — `-1` 과 비교된다.
- 후보(CLDC 1.1 String 의 int 두 개 · int 반환): `indexOf(II)`(22) · `lastIndexOf(II)`(24).
- **호출부가 가른다**: 첫 결과 `i` 가 -1 이 아니면 `from = i + 1` 로 **앞으로** 옮겨 다시 찾는다 — 연속된 `'&'` 를 차례로 찾는 순방향 훑기다.
  lastIndexOf(ch, i+1) 이라면 방금 찾은 `i` 를 다시 돌려준다.
- r1 이 String 이 아니므로 이미 핀된 `indexOf(Ljava/lang/String;I)I`(26)도 아니다.
- CLDC 1.1 선언 순서 규칙도 indexOf(II) 를 22 에 둔다 — 핀된 indexOf(I)(21) 와 lastIndexOf(I)(23) 사이. 호출부 판정과 일치.
- 런타임(`dlunch/RustJava@5b84dd1` `java/lang/string.rs:90`)이 `indexOf (II)I` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --timeout 20` · release · loadavg 190~250)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `b29d043e`(계측만) ×1 | FAIL | 103466 | 5 | `java/lang/String vtable index 22` |
| 이 브랜치 ×3 | FAIL | 85080 / 16901 / 71652 | 5 / 5 / 5 | `java/util/Stack vtable index 33` |
| 변이: indexOf(II) 행만 제거 | — | — | — | 핀 테스트 FAILED(«String vtable index 22 is empty») |

ticks 편차는 부하 때문이다 — 판정은 첫 예외가 바뀐 것(3/3 동일)이다.

### 한계

- 호출부 1곳(한 타이틀).
- 다음 벽 Stack 33 은 등재하지 않았다(범위 밖) — worklog 후속 제안.
- player-updates 는 추가하지 않았다 — 타이틀은 여전히 FAIL(선례 #353·#367·#376 과 같은 판단).

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

유입 30건(BOUNDED) + 판단 필요 6건(SUFFIX-ATTACHED). 도구는 바뀐 파일의 본문 전체를 센다 — `jvm_support.rs`·`lgt_java_abi.toml`
의 제목 언급은 대부분 원래 있던 주석이다. 이 회차가 새로 더한 제목은 `월드장기체스` 하나(ABI 주석 1 · 핀 테스트 주석 1 · 이 문서 · worklog)이고,
선례(0333·0344)와 같은 자리다. SUFFIX-ATTACHED 6건은 `간호사타이쿤2`·`서든어택포켓` — «더 긴 다른 제목»이 든 원래 있던 주석이다.
<!-- corpus-name-inflow v1 subjects=4 tree=350753c385376d16 B=100/32 P=0/0 S=19/8 -->
