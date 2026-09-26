## [2026-09-26] java/util/Random 10 = setSeed(J)V — 월드장기체스 벽이 Vector 31 로 이동 (wie-2026-09-25-wec-random-vtable-10-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` Random 블록에 `setSeed(J)V = 10` 1행과 근거 주석,
ABI 핀 테스트(`abi_rows_cover_the_indexes_titles_actually_dispatch_on`)에 1줄.

**왜**: String 16(#0267) 뒤 월드장기체스가 `java/util/Random vtable index 10` 에서 멈췄다.

**사용자 영향**: 월드장기체스가 부팅에서 수천 tick 더 가지만 화면은 아직 없다(다음 벽 `java/util/Vector` 31).

### 반증 먼저

- ⓐ 「Random 블록이 없다」는 **틀렸다** — `origin/main` `0c93f2f4` 에 블록이 있고 `nextInt()I = 12` 1행뿐이다. 칸 10 은 비어 있다(전제의 요지는 선다).
- ⓑ 벽 재현: `wie_validate --timeout 20` 3/3 `Unimplemented: java/util/Random vtable index 10`.

### 10 이 setSeed(J)V 인 근거 — 호출부 디스어셈블(임시 계측 · 커밋 안 함)

missing-entry 스텁에 lr·r0..r3·lr-0x200 부터 0x280 바이트 덤프를 달아 ARM 모드로 풀었다(vtable 오프셋 = 4 + 4×10 = 0x2c).
실측: lr=0x1c74 · r1=0xdcc55a04 · r2=0x1a0.

```
0x1c18 ldr r5,[r2,r3,lsl #2]         ; r5 = this.<Random 필드>
0x1c1c ldr ip,[r4,#0x1e0] ; bx ip     ; 네이티브 헬퍼 1
0x1c28 ldr ip,[r4,#0x1ec] ; bx ip     ; 네이티브 헬퍼 2 → r0:r1 (long)
0x1c38 mov r3,r1 ; mov r2,r0 ; mov r1,r2 ; mov r0,r5 ; mov r2,r3   ; r1:r2 = 그 long
0x1c4c bne 0x1c60                     ; r5 == null 이면 예외 경로
0x1c64 ldr r3,[r5] ; ldr ip,[r3,#0x2c] ; bx ip   ; r5.<10>(long)
0x1c74 ldm sp,{r4,r5,fp,sp,lr} ; bx lr           ; 반환값을 읽지 않고 에필로그
```

- 인자: long 1개. r1:r2 = 0x1a0:0xdcc55a04 = 1790410316292 ms = 2026-09-26 17:11:56 — 그 실행의 벽시계 시각(시드 = 현재 시각).
- 반환: 쓰이지 않는다(void).
- CLDC Random 의 메서드 중 long 인자는 setSeed 뿐이고, 규칙(10 + 선언 순서)도 첫 메서드 setSeed 를 가리킨다.
- 런타임(`dlunch/RustJava@5b84dd1` `java/util/random.rs`)이 `setSeed (J)V` 를 구현한다.

### 전/후 + 역변이 (`--timeout 20` · loadavg 126~135)

| 트리 | result | ticks | paints | 첫 예외 |
|---|---|---|---|---|
| main `0c93f2f4` ×3 | FAIL | — | 0 | `java/util/Random vtable index 10` |
| 이 브랜치 ×3 | FAIL | 13263 / 44119 / 3748 | 0 | `java/util/Vector vtable index 31` |
| 변이: setSeed 행만 제거 | FAIL | 2 | 0 | `java/util/Random vtable index 10` |

### 회귀

`cargo fmt --check` · `clippy --all -D warnings` · wasm clippy · `+beta clippy` 전부 통과 · `RUST_MIN_STACK=4194304 cargo test --all` 454 passed / 0 failed.

### 한계

- 호출부 1곳(한 타이틀). 다른 타이틀의 교차 호출부는 없다 — 다만 인자가 실제 현재 시각이라는 값 증거가 모양 증거에 더해진다.
- 두 네이티브 헬퍼(`[r4,#0x1e0]`·`[r4,#0x1ec]`)의 정체는 확인하지 않았다(값이 현재 시각이라 판정에 불필요).
- 다음 벽 Vector 31 은 등재하지 않았다(범위 밖) — worklog 후속 제안.
