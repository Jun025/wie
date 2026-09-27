## [2026-09-27] java/util/Vector 31 = removeAllElements()V — 월드장기체스 벽이 Vector 29 로 이동 (wie-2026-09-26-wec-vector-vtable-31-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Vector 블록에 `removeAllElements()V = 31` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: Random 10(#0287) 뒤 월드장기체스가 3/3 `java/util/Vector vtable index 31` 에서 멈췄다.

**사용자 영향**: 월드장기체스가 약 3배 더 오래 부팅을 진행하지만 화면은 아직 없다(다음 벽 `java/util/Vector` 29).

### 31 이 removeAllElements()V 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

missing-entry 스텁에 lr·r0..r3 과 lr-0x280 부터 0x300 바이트 덤프를 달아 ARM 모드로 풀었다(vtable 오프셋 = 4 + 4×31 = 0x80).
실측: lr=0x2aab0 · r0 = Vector 인스턴스 · r1=0 · Java 스택 `d.paint(Lorg/kwis/msp/lcdui/Graphics;)V`.

```
0x2aa64 subs r3,r0,#0 ; mov r1,#0 ; ldrne … [r3,#0x48] ; bxne ip   ; 조건부 호출 (r1=0 은 이 호출의 인자)
0x2aa7c ldr r3,[r4,#8] ; ldr r3,[r3] ; subs r0,r3,#0 ; bne 0x2aa9c ; r3 = this.<필드>[0], null 이면 예외 경로
0x2aa9c mov r0,r3 ; ldr r3,[r3] ; ldr ip,[r3,#0x80] ; bx ip         ; Vector.<31>() — r0 외 인자 레지스터 기록 없음
0x2aab0 ldr r2,[r4,#8] ; mov r3,#0 ; str r3,[r2,#4]                ; r0 를 읽지 않고 옆 int 필드를 0 으로
0x2aabc ldm … ; bx lr
```

- 인자: 없음. r1=0 은 앞선 **조건부** 호출용이라 합류점 이후 살아 있다고 가정할 수 없다(그 호출이 실행되면 r1 은 파괴된다).
- 반환: 쓰이지 않는다(void). 호출 직후 같은 블록의 int 필드를 0 으로 되돌린다 — 비우기와 커서 리셋의 모양.
- 후보(CLDC 1.1 Vector 의 인자 없는 void): `trimToSize()V`(순서 11) · `removeAllElements()V`(순서 31).
  기존 4행(15·23·27·28)이 모두 선언 순서 위치에 있으므로 31 은 removeAllElements 이고, trimToSize 는 31 에 올 수 없다.
  (`clear()V` 는 런타임에 있으나 CLDC Vector 메서드가 아니다.)
- 같은 함수의 `0x2ab0c ldr ip,[r3,#0x40]`(= 15) 호출 결과를 루프 상한으로 쓴다 — size()I 행과 일치(교차 확인).
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/vector.rs`)이 `removeAllElements ()V` 를 구현한다.

### 전/후 + 역변이 (`wie_validate --timeout 20` · loadavg 40~98)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `origin/main`(계측만) ×1 | FAIL | 19123 | 0 | `java/util/Vector vtable index 31` |
| 이 브랜치 ×3 | FAIL | 63909 / 60438 / 62016 | 0 | `java/util/Vector vtable index 29` |
| 변이: removeAllElements 행만 제거 | FAIL | — | 0 | `java/util/Vector vtable index 31` · 핀 테스트 FAILED |

### 회귀

`cargo fmt --check` · `clippy --all -D warnings` · wasm clippy · `+beta clippy` 전부 경고 0 · `RUST_MIN_STACK=4194304 cargo test --all` 495 passed / 0 failed.

### 한계

- 호출부 1곳(한 타이틀). 모양(인자 0 · void)만으로는 trimToSize 와 갈리지 않는다 — 판정은 선언 순서가 한다(기존 4행이 그 순서를 실측으로 지지).
- 다음 벽 Vector 29 는 등재하지 않았다(범위 밖) — worklog 후속 제안.
