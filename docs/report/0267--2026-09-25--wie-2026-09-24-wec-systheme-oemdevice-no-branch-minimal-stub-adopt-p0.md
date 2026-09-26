## [2026-09-25] java/lang/String 16 = compareTo(String) — 월드장기체스 벽이 Random 10 으로 이동 (wie-2026-09-24-wec-systheme-oemdevice-no-branch-minimal-stub-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` String 블록에 `compareTo(Ljava/lang/String;)I = 16` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 16 행 1줄.

**왜**: wec 스텁(#273) 뒤 월드장기체스가 `java/lang/String vtable index 16` 에서 멈췄다.

**사용자 영향**: 월드장기체스가 부팅에서 한 tick 더 가지만 여전히 화면은 없다(다음 벽 `java/util/Random` 10).

### ⒜ 현 main 에서 이 칸인가 — 예

`origin/main` `595de2b6`, `wie_validate --timeout 20`: 3/3 `FAIL` · ticks 1 · paints 0 ·
`Unimplemented: java/lang/String vtable index 16` · `at CCC.<init>()V`.

### ⒝ 16 이 compareTo(String) 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

missing-entry 스텁에 lr·r1..r3·r1 의 클래스명·lr-0x300 부터 0x400 바이트 덤프를 달아 ARM 모드로 풀었다
(vtable 오프셋 = 4 + 4×16 = 0x44). 실측: lr=0x21320 · r1 클래스 = `java/lang/String`.

```
0x212ec mov r0,#0x30 ; bx r6        ; c1 = 상수 문자열 (r6 = 상수→String 팩토리, 0x2f..0x33 로 반복 호출)
0x212f8 cmp r5,#0 ; beq 0x2145c     ; s == null 이면 예외 경로
0x21308 mov r0,r5 ; mov r1,r8       ; s.<16>(c1)
0x21314 ldr ip,[r3,#0x44] ; bx ip
0x21320 cmp r0,#0 ; beq 0x213b8     ; 0 이면 플래그로
 …같은 모양 3회 더(c2·c3·c4), 모두 beq/bne 로 0x213b8 …
0x213b8 mov r8,#1 ; str r8,[r3,#0xc] ; 플래그 = 1
0x213c4 …두 번째 사슬, 같은 모양(0x213ec ldr ip,[r3,#0x44])
```

- 인자: String 1개(힙에서 클래스명 확인). 반환: 0 과만 비교.
- 의미: 「s 가 상수 넷 중 하나와 같으면 플래그 = 1」 — 0 = 같음인 compareTo 와 맞는다.
  이웃 String 인자 메서드 중 equalsIgnoreCase(15)는 불일치에 0 을 돌려주므로 「아무 0 이면 플래그」는
  서로 다른 상수 넷에 대해 항상 참이 되어 사슬이 무의미하다. startsWith(S)/endsWith/indexOf(S) 는 19/20/25.
- 위치도 규칙(10 + CLDC 선언 순서: equalsIgnoreCase 15 · compareTo 16)과 같다.

### ⑴ 전/후 + 양방향 변이 (`--timeout 20` · loadavg 105~312)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `595de2b6` ×3 | FAIL | 1 | 0 | `java/lang/String vtable index 16` |
| 이 브랜치 ×3 | FAIL | 2 | 0 | `java/util/Random vtable index 10` |
| 변이: 16 행만 제거 | FAIL | 1 | 0 | `java/lang/String vtable index 16` |

### 한계

- 호출부는 한 타이틀 안의 두 사슬·8곳이다. 다른 타이틀의 두 번째 호출부 교차는 없다(제안 tradeoff 미해소분).
- 다음 벽 Random 10 은 후속 제안으로 남겼다(worklog `2026-09-25-wec-string-16-compareto.json`).
