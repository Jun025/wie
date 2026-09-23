## [2026-09-23] 배틀몬스터가 스플래시를 넘어 «마을»까지 간다 — `java/io/InputStream` index 13 = `skip(J)J` (wie-battlemonster-past-splash-bytearrayinputstream-skip-slot)

### 무엇을

`wie-lgt/data/lgt_java_abi.toml` 의 `java/io/InputStream` 행에 **한 줄**(`skip (J)J = 13`) + 그 행을 고정하는
시험 1행(`("java/io/ByteArrayInputStream", 13, "skip", "(J)J")` — 게스트가 실제로 디스패치한 클래스로 고정).
본문은 데이터 1행·시험 1행이고 나머지는 주석이다. 다른 칸(`mark`·`reset`·`markSupported` 16~18)은 비워 뒀다.

### 왜

PR #261(`Object` index 5 = `notify()`) 위에서 배틀몬스터는 이용안내 → 개발사 스플래시까지 그리고
**흰 화면에서 멈췄다**. 그 순간 단 1건:

```
ERROR rustjava_runtime::classes::java::lang::thread: Uncaught exception in thread 1622147107:
net.wie.WieError: Unimplemented: java/io/ByteArrayInputStream vtable index 13
```

**도출**: `InputStream` 의 12(`read([BII)I`)와 14(`available()I`)가 둘 다 측정값이고, CLDC 1.1 선언 순서에서
그 사이 메서드는 `skip(J)J` 하나뿐이다.

**호출부 관측**(임시 계측 후 되돌림): 누락 스텁 진입 시 `r0=this · r1=0x26 · r2=0x0`, 게스트 코드는

```
mov r2, sb ; asr r3, r2, #31 ; mov r1, r2 ; mov r2, r3   ; int → long 부호확장(i2l) → r1:r2
ldr r3, [r5] ; ldr ip, [r3, #0x38] ; mov lr, pc ; bx ip  ; 디스패치
ldr r1, [fp, #-0x38] ; ldr r2, [fp, #-0x34] ; cmp r1, r2 ; 반환 r0:r1 은 즉시 덮인다 (문장형 `in.skip(n);`)
```

⇒ long 인자(38바이트)가 이 런타임의 순차 워드 규약대로 **r1:r2** 에 오고, 반환값은 버려진다.
(티켓이 적은 「AAPCS r2:r3」 는 #261 검수 F1 이 정정한 그 오류다 — 옳은 규약은 r1:r2.)
`rustjava-runtime@5b84dd1` 은 `ByteArrayInputStream.skip(J)J` 를 **이미 구현**한다(`byte_array_input_stream.rs:151`).

### 사용자 영향

같은 명령(`--inject --boot-secs 6 --action-secs 2 --max-ticks 2000000000 --timeout 300`, 짝지은 실행):

| | before(#261 핀) | after |
|---|---|---|
| 스플래시 뒤 | 흰 화면 22장 동일 | 타이틀 → 메뉴(「새로하기」 대화상자) → LOADING → 스토리 인트로 → 월드맵 |
| `paints` | 36 | 222 |
| ERROR / Uncaught | 위 1건 | 0 |

`--action-secs 4` 로 더 길게 돌리면 **플레이어 캐릭터가 선 마을 화면**까지 간다. 그 직후 다음 벽:

```
ERROR rustjava_runtime::classes::java::lang::thread: Uncaught exception in thread 1622147107:
net.wie.WieError: Invalid memory access; address: 0
```

이후 프레임이 멈춘다(마지막 5장 동일). ⇒ **판정선: 타이틀 ✓ · 메뉴 ✓ · 게임 화면 도달 ✓ · 게임 화면에서 입력 반응 ✗(미관측).**

### 회귀

짝지은 실행(before·after 동시 기동, `--timeout 30`): 메이플스토리2007 · 현영맞고2006 · 체스마스터 · 배틀몬스터 2파일 **양쪽 PASS**.
놈3 은 load 64~124 에서 양쪽 다 `p0 · deadline` 을 낸다(before 3회 · after 5회 / 13쌍). `--timeout 90` 에서
after 3/3 PASS · before 2/3 — **손대지 않은 쪽에서도 재현**되고 before 로그에 누락 슬롯 진입이 0건이므로
이 변경의 산물이 아니다. 4게이트 + `cargo +beta clippy` 통과.
