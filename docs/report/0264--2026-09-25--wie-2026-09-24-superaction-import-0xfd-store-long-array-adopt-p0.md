## [2026-09-25] DataInputStream 27 = readChar()C · 32 = readUTF()String — 스파이더맨3·슈퍼액션히어로 벽을 한 칸씩 넘겼다 (wie-2026-09-24-superaction-import-0xfd-store-long-array-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` 의 `java/io/DataInputStream` 블록에 두 행을 더했다 — `readChar()C = 27`,
`readUTF()Ljava/lang/String; = 32` — 와 근거 주석. 코드 변경 0. 채택 제안
`2026-09-24-superaction-import-0xfd-store-long-array#p0` · `2026-09-24-lgt-wide-field-second-word-link#p1`.

**왜**: 두 타이틀이 각각 이 두 빈 칸에서 `Unimplemented` 로 멈췄다. 두 칸 모두 선언 순서 규칙이 정하는 자리이고,
호출부가 그 자리와 맞는다(아래 ⒝).

**사용자 영향**: 아직 없다 — 두 타이틀 모두 여전히 균일색이다. 벽이 한 칸씩 뒤로 옮겨 갔다
(슈퍼액션히어로 → `java/util/Random vtable index 10`, 스파이더맨3 → `DataInputStream vtable index 19`).

### ⒜ 전제 재측 — `origin/main` `2efea821` release `wie_validate --inject`, 3회씩
| 타이틀 | stop | input | ticks | paints | colors | 게임 스레드의 죽음 |
|---|---|---|---|---|---|---|
| 슈퍼액션히어로 | deadline ×3 | 27/27 | 5.27M·6.34M·5.87M | 2·3·3 | 1 | `Unimplemented: java/io/DataInputStream vtable index 32` 3/3 |
| 스파이더맨3 | error ×3 | 1·0·0/27 | 82,407·101,883·38,080 | 1·1·1 | 1 | `Unimplemented: java/io/DataInputStream vtable index 27` 3/3 |

전제가 선다. (`2efea821` 과 착지 기준 `1d562b34` 의 차이는 같은 블록의 `readBoolean = 22` 한 행뿐이고, 변이 검사가
`1d562b34` 위에서 같은 두 벽을 재현한다 — 아래.)

### ⒝ 칸별 판정 — 호출부 디스어셈블(binary.mod · Thumb · 기호 없음)
LR 은 `handle_missing_java_vtable_entry` 에 임시로 `dump_reg_stack` 을 붙인 디버그 빌드로 쟀다(커밋하지 않았다).
★물리 오프셋은 `4 × (index + 1)` 이다 — 스파이더맨3 의 LR `0x1b0a` 직전이 `ldr r3, [r3, #0x70]` 이고 런타임은 그것을
index 27 로 불렀다. 형제 회차의 readBoolean(22 = `#0x5c`)과 같은 축이다.

| index | 판정 | 근거 |
|---|---|---|
| 27 | `readChar()C` | 스파이더맨3 `0x1ac6~0x1b86`: 한 스트림에 `#0x70`(27) → `#0x60`(readByte 23) → `#0x68`(readShort 25, 결과−4 로 `byte[]` 할당) → `#0x70` ×2 → `#0x50`(readFully([B) 19, 그 배열). 27 의 인자는 r0 뿐(r1 = 0x1afb 는 낡은 값). 결과 하나를 `(v >> i) & 1` 로 **정확히 16회**(`i <= 15`) 훑는다 ⇒ 16비트 값. 위치는 측정 앵커 25(readShort)·28(readInt) 사이의 규칙 자리. |
| 32 | `readUTF()Ljava/lang/String;` | 슈퍼액션히어로 `0x1a04`: `adds r3, #0x84; ldr r3, [r3]`, 인자 r0 뿐. 결과를 null 검사한 뒤 그 객체에 **String.getBytes()[B**(index 14, `#0x3c` — 표에 이미 있는 행)를 부르고 `byte[len+1]` 에 복사한다 ⇒ String 반환. no-arg·String 반환인 DataInputStream 메서드는 readUTF() 하나. CLDC 1.1 순서 readLong 29 · readFloat 30 · readDouble 31 · readUTF 32 가 그 자리를 준다(1.0 이면 30 — Thread 계수가 이미 1.1 을 정했다). |

★**27 이 가르지 못하는 것**: readUnsignedShort()I(26)는 같은 두 바이트를 읽어 같은 부호 없는 값을 돌려준다 — 값으로는 둘을
못 가르고 슬롯 순서만 가른다. 틀려도 게스트가 받는 수는 같다.

**after 교차 확인**(`RUST_LOG=debug`, 한 판): 스파이더맨3 는 `readChar` 3 · `readByte` 1 · `readShort` 1 — 디스어셈블의
호출 순서(27, 23, 25, 27, 27, 그리고 19)와 **개수까지** 같다. 슈퍼액션히어로는 `readUTF` **196**회이고 `UTFDataFormatException`
0 — 호출 루프 `0x43be` 가 `cmp r5, #0xc3; ble` 로 정확히 196(0..195)번 도는 것과 같은 수다. 길이 워드를 잘못 읽으면
다음 readFully 가 EOF 로 터진다; 196 연속으로 터지지 않았다.

### ⒞ before / after (release · `--inject` · 3회씩)
| | 타이틀 | stop | input | ticks | paints | colors | 죽음 |
|---|---|---|---|---|---|---|---|
| before | 슈퍼액션히어로 | deadline ×3 | 27/27 | 5.27M·6.34M·5.87M | 2·3·3 | 1 | DataInputStream **32** |
| after | 슈퍼액션히어로 | deadline ×3 | 27/27 | 14.35M·13.61M·14.08M | 3·3·3 | 1 | `java/util/Random vtable index 10` |
| before | 스파이더맨3 | error ×3 | 1·0·0 | 82,407·101,883·38,080 | 1 | 1 | DataInputStream **27** |
| after | 스파이더맨3 | error ×3 | 0·0·0 | 134,633·4·181,471 | 1 | 1 | DataInputStream **19** |

`ticks` 는 처리량 척도가 아니다(AGENTS.md) — 비교 축은 벽 문면이다.

### ⒟ 변이 검사 (양방향 · debug 빌드 · `1d562b34` + 이 diff 에서 한 행씩 제거)
| 변이 | 슈퍼액션히어로 | 스파이더맨3 |
|---|---|---|
| readChar 27 행 제거 | `java/util/Random vtable index 10`(무관) | `Unimplemented: java/io/DataInputStream vtable index 27` ← 원래 벽 |
| readUTF 32 행 제거 | `Unimplemented: java/io/DataInputStream vtable index 32` ← 원래 벽 | `DataInputStream vtable index 19`(무관) |

각 행이 자기 타이틀의 벽 하나만 연다.

### 회귀
- 4게이트: `fmt` rc0 · `clippy --all -D warnings` 출력 0 · wasm clippy 출력 0 · `RUST_MIN_STACK=4194304 cargo test --all`
  **444 passed / 0 failed**(결과 블록 46) · `cargo +beta clippy --all -D warnings` 출력 0.
- `cargo clippy --workspace --all-targets` 경고: after **16** · base(`git stash`) **16** — 증가 0(diff 는 Rust 0줄).
- runner(LGT, release after): `helloworld_lgt` PASS · `keydraw_lgt --inject --expect-last-frame` PASS.
- 부하: loadavg 43~153(측정 중 변동) — before/after 는 같은 시간대에 짝지어 돌리지 못했다(빌드 사이 약 1시간).

### 다음 벽 (이 회차 범위 밖 — 제안으로 남겼다)
- 스파이더맨3 → `DataInputStream` 19. 같은 함수 `0x1b82` 가 `#0x50` 을 r1 = 방금 할당한 `byte[]` 한 개로 부른다 —
  readFully([B)V 의 규칙 자리와 맞는다.
- 슈퍼액션히어로 → `java/util/Random` 10. 이 회차는 재지 않았다.
