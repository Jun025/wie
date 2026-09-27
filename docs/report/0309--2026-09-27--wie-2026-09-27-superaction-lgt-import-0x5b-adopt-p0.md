## [2026-09-27] LGT import 0x5b = long[] 원소 읽기(`laload`) — 슈퍼액션히어로 첫 게임 화면 · `--inject` 27/27 PASS (wie-2026-09-27-superaction-lgt-import-0x5b-adopt-p0)

**무엇을**: LGT Java 시스템 import `0x5b` 를 `LoadLongArray`(checked `laload`)로 등재했다. 처리기 1개 + SVC id 36 + 반환 쓰개(`LongReturn`, r0 = 하위 · r1 = 상위).
**왜**: 슈퍼액션히어로의 게임 스레드가 `Fatal error: Unknown lgt java import: 0x5b` 로 죽어 균일색 화면에 머물렀다(0301).
**사용자 영향**: 슈퍼액션히어로가 컴투스 로고 → 튜토리얼 전투 화면까지 그린다. 3판 모두 입력 27/27 을 받고 살아남는다.

### 0x5b 판정 — binary.mod(ELF · Thumb · 기호 없음) 디스어셈블
- 트램폴린 `{push {lr}; bl resolver; 0x64; 0x5b}` = `0x1401a74`. `.text` 리터럴 풀 참조 2곳(`0x8a70`·`0x9958`) → 호출부 3곳(`0x883c`·`0x88b4`·`0x987c`). 호출은 `bl 0x4ac08`(`bx r3`)/`0x4ac0c`(`bx r4`).
- 모양은 셋 다 `r0` = 정적 필드 배열 · `r1` = 인덱스 → 64비트 반환(r0·r1 둘 다 저장).
- ★**`0x987c` 가 결정한다 — CRC-32 한 걸음**: `r1 = ((byte)b ^ crc_lo) & 0xff` 로 인덱스를 만들고 0x5b 를 부른 뒤, `crc >>> 8`(시프트 도우미 `r2 = 8`)과 **r0 ↔ 하위 · r1 ↔ 상위**로 xor 한다(`0x98c2 eors r5,r3` · `0x98c8 eors r6,r4`). 배열은 클래스 정적 필드 `+0xb8` 이고, 같은 필드를 `0x97f2` 가 **0xfd(`lastore`)** 로 256칸 채운다(`cmp r1, #0xff` 루프 · 다항식 xor `0x97aa`). ⇒ long[] 읽기.
- `0x8842`: 반환 r1 을 먼저 부호 비교(`cmp r0, r6; bgt` — r0 에 옮겨 둔 반환 r1) ⇒ r1 = 상위 워드. 위와 일치.
- ★**워드 순서가 0xfd 와 반대다**: 0xfd 는 값을 r2 = 상위 · r3 = 하위로 받고(0240), 0x5b 는 r0 = 하위 · r1 = 상위로 돌려준다(AAPCS). 같은 바이너리의 같은 배열에서 둘 다 읽었다.
- 어느 호출부도 long[] 을 null·범위 검사하지 않는다 ⇒ 처리기가 NPE · AIOOBE 를 던진다(0xfd 와 같은 규율).

### 실측 — release `wie_validate --inject`, 3회씩
| | stop | input | paints | colors | 게임 스레드 |
|---|---|---|---|---|---|
| origin/main `156c5c56` | deadline ×3 | 27/27 | 3·2·2 | 1 | `Unknown lgt java import: 0x5b` 3/3 |
| 이 브랜치(기본 max-ticks) | max-ticks ×3 | 19·6·11/27 | 99·12·52 | 39·23·38 | 오류 0 · `UNMEASURED` |
| 이 브랜치(`--max-ticks 1000000000`) | deadline ×3 | **27/27** | 161·190·193 | 93·46·46 | 오류 0 · **PASS · content true** 3/3 |

- 첫 판은 기본 `--max-ticks` 에 걸린 `UNMEASURED`(§The four gates: «on max-ticks raise --max-ticks»)라 올려서 다시 쟀다.
- 스크린샷(`--shot-every 5`, git 무시 스크래치): t=5s 컴투스 로고 · 27번째 키 뒤 튜토리얼(「모든 공격은 ez-i 버튼으로 시작되네」).
- ★**다음 벽: 이 판 안에서는 없다.** 이 바이너리가 import 하는 Java 시스템 번호 중 표에 없는 것이 `0x40`·`0x64` 둘 남아 있지만 27키 안에서는 풀리지 않았다(제안 카드로 올리지 않았다 — 관측된 실패가 아니다).

### 시험(되돌리면 red)
`load_long_array_returns_low_word_in_r0_and_checks_its_array`: import 0x5b 해석 · 0xfd 로 넣은 `0x1234_5678_9abc_def0` 을 읽어 r0 = `0x9abc_def0` · r1 = `0x1234_5678` · 범위 밖 2 · `u32::MAX` · null 은 게스트 예외.
변이 2건 모두 red(실측): 쓰개의 워드 순서 뒤집기 → `left: (305419896, 2596069104)` · 표에서 `0x5b` 행 삭제 → `Unknown lgt java import: 0x5b`. SVC id 왕복 시험의 마지막 id 도 `LoadLongArray` 로 옮겼다.

### 회귀
- 코퍼스 LGT binary.mod 100 중 import 0x5b 보유: 슈퍼액션히어로 · 월드장기체스(+ `_dup` 사본) · 메이플스토리2007 · 학교가는길. 나머지 셋 `--timeout 20` 2회씩 base ↔ 이 브랜치 판정·문면 동일(메이플스토리2007 PASS · 월드장기체스 FAIL error · 학교가는길 FAIL — 뒤 둘은 기존 벽, 0x5b 에 닿지 않는다).
- 러너(LGT): `helloworld_lgt` PASS · `keydraw_lgt --inject --expect-last-frame` 기본 max-ticks 에서 base·이 브랜치 모두 `UNMEASURED max-ticks`(load1 70~113) → `--max-ticks 1000000000` 에서 둘 다 **PASS · paints 55 · rc0**.
- 네 게이트 + beta: fmt OK · clippy/wasm clippy/beta clippy rc0 · `RUST_MIN_STACK=4194304 cargo test --all` 48 스위트 **496 passed 0 failed**.
