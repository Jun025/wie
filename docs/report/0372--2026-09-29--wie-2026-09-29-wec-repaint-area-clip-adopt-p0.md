## [2026-09-29] java/util/Vector 17 = elements()Ljava/util/Enumeration; — 월드장기체스 로고 다음 · 벽이 java import 0x64 로 이동 (wie-2026-09-29-wec-repaint-area-clip-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Vector 블록에 `elements()Ljava/util/Enumeration; = 17` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: repaint 클립(#400 · 0368) 뒤 월드장기체스가 제작사 로고까지 가서 키 2~3 에서 `java/util/Vector vtable index 17` 로 멈췄다(4/4).

**사용자 영향**: 월드장기체스가 한 벽을 더 넘는다. 아직 플레이는 안 된다(다음 벽 `Unknown lgt java import: 0x64`).

### 17 이 elements() 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

0344·0351 과 같은 방식: missing-entry 스텁이 lr·r0~r3 과 lr-0x100 부터 0x800 바이트를 오류 문구에 싣게 하고 capstone ARM 모드로 풀었다.
실측: lr=0xd3b8 · r0=0x488421a0(Vector).

```
0xd398 ldr r8,[sl,#4] ; cmp r8,#0 ; beq …                        ; this.<필드>+4 = Vector · null 검사
0xd3a4 mov r0,r8 ; ldr r3,[r8] ; ldr ip,[r3,#0x48] ; bx ip       ; Vector.<17>() — 0x48 = 4 + 4×17, r0 외 인자 없음
0xd3b8 str r0,[fp,#-0x58] ; b 0xd7f8                             ; ← lr · 결과를 지역에 두고 루프 머리로
0xd7f8 ldr r8,[fp,#-0x58] ; cmp r8,#0 ; …                        ; 결과 null 검사
0xd80c ldr r3,[pc,#…] ; ldr ip,[r3,#0x44] ; bx ip                ; 인터페이스 서술자
0xd81c … ldr r1,[r3,#8] ; mov r0,r8 ; ldr r6,[pc,#…] ; bx r6     ; 결과 객체에서 인터페이스 메서드 표 찾기
0xd838 ldr r3,[r0,#4] ; … mov r0,r8 ; bx r3                      ; 인터페이스 1번 슬롯(결과 객체에)
0xd864 cmp r0,#0 ; beq 0xd8f0                                    ; 거짓이면 루프 탈출 — 플래그
0xd3cc ldr r8,[fp,#-0x58] ; … mov r0,r8 ; … bx r6                ; 본문: 같은 인터페이스 표 다시
0xd3ec ldr r3,[r0,#8] ; … mov r0,r8 ; bx r3                      ; 인터페이스 2번 슬롯 → 객체
0xd404 subs r8,r0,#0 ; … (클래스 비교 호출 두 번) ; beq → 예외      ; 게스트 클래스 캐스트 검사
```

- 인자: 없음. 반환: 객체 — 원소로 쓰이지 않고 **인터페이스 디스패치의 수신자**가 된다.
- 후보(CLDC 1.1 Vector 의 인자 없는 객체 반환): `elements()`(17) · `firstElement()`(24) · `lastElement()`(25).
- **호출부가 가른다**: 루프 머리마다 결과 객체에 인터페이스 1번 슬롯(참/거짓 → 탈출)을, 본문에서 2번 슬롯(객체 → 캐스트)을 부른다 —
  `for (Enumeration e = v.elements(); e.hasMoreElements();) x = (T) e.nextElement();` 이다.
  firstElement·lastElement 는 원소 자신을 돌려주므로 그 위에서 hasMoreElements/nextElement 모양의 인터페이스 호출이 나올 수 없다.
- CLDC 1.1 선언 순서도 elements() 를 17 에 둔다 — 핀된 isEmpty(16) 바로 뒤. 호출부 판정과 일치.
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/vector.rs:32`)이 `elements ()Ljava/util/Enumeration;` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --inject` · release · load1 515~686)

| 트리 | result | ticks | paints | 키 | 첫 예외 |
|---|---|---|---|---|---|
| main `94100f99`(계측만) ×2 | FAIL | 2936758 / 2773116 | 159 / 159 | 2/27 | `java/util/Vector vtable index 17` |
| 이 브랜치 ×4 | FAIL | 1930571 / 2233201 / 2221458 / 2617672 | 133 / 139 / 132 / 138 | 2/27 | `Unknown lgt java import: 0x64` |
| 변이: elements 행만 제거 | — | — | — | — | 핀 테스트 FAILED(«Vector vtable index 17 is empty») |

전은 2판이다 — #400(0368)의 후 4판이 같은 main 코드에서 4/4 같은 벽이었다. 판정은 첫 예외가 바뀐 것(4/4 동일)이다.
paints 는 부하에 따라 움직인다(0368 의 86~165). `frozen_tail_steps` 전·후 모두 0.

### 한계

- 호출부 1곳(한 타이틀).
- 다음 벽 import 0x64 는 등재하지 않았다(범위 밖 · 지연 해결되는 java 시스템 import 로 0x5b 와 같은 계열) — 호출부는 아직 모른다. worklog 후속 제안.
- player-updates 는 추가하지 않았다 — 타이틀은 여전히 FAIL(선례와 같은 판단).

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

유입 32건(BOUNDED) + 판단 필요 8건(SUFFIX-ATTACHED). 이 회차가 새로 더한 제목은 `월드장기체스` 하나(ABI 주석 1 · 핀 테스트 주석 1 · 이 문서 · worklog)이고,
선례(0344·0351)와 같은 자리다. SUFFIX-ATTACHED 중 `월드장기체스가`(이 문서·worklog)는 조사 붙은 실제 언급이고, 나머지 `간호사타이쿤2`·`서든어택포켓` 은
«더 긴 다른 제목»이 든 원래 있던 주석이다.
<!-- corpus-name-inflow v1 subjects=4 tree=38fabbbc3bf6453d B=106/32 P=0/0 S=20/8 -->
