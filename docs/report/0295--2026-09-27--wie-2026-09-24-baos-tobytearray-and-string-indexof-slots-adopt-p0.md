## [2026-09-27] OutputStream 14 = close()V — 서든어택포켓 부팅 통과 (wie-2026-09-24-baos-tobytearray-and-string-indexof-slots-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` 의 `java/io/OutputStream` 블록에 `close ()V = 14` 한 줄과 근거 주석을 더했다.

**왜**: 직전 회차에서 ByteArrayOutputStream 16(toByteArray)을 연 뒤, 턴·서든어택포켓 4파일이 모두 `Unimplemented: java/io/DataOutputStream vtable index 14` 에서 멈췄다.
현 main `1ebafd2e` 에서 다시 재현했다. 무주입 · `RUST_LOG=info wie_validate --timeout 90` 조건이고, 부팅 중(tick 2~12) 이 칸에 닿는다.
형제 회차가 적은 «60초 무주입 예산에서는 저장 경로에 닿지 않는다»와 조건이 다르다. 이번에는 90초 예산이고, 두 타이틀 모두 부팅 단계(턴 `GameMIDlet.<init>`, 서든어택포켓 `SAttack.startApp`)에서 세이브를 쓴다.

**판별 근거**(capstone 으로 binary.mod 를 전수 스캔했다. `ldr ip,[r3,#0x44]` 뒤 60워드 안에 `ldr ip,[r3,#0x3c]` 가 2개 이상 오는 자리):
- 턴 4곳, 서든어택포켓 5곳 모두 같은 모양이다. 서든어택포켓은 별개 바이너리이므로 이것이 두 번째 호출부 교차다.
- toByteArray(16) 직후 `fp-0x24`(DataOutputStream, baos 를 감싼 바깥)에 먼저, `fp-0x20`(baos)에 그다음으로 14를 호출한다. 두 호출 모두 r0 만 세팅하고 반환값은 쓰지 않는다.
- 그 뒤 두 스트림은 참조되지 않은 채 함수가 반환한다. 즉 `dos.close(); baos.close();` 이다.
- 선언 순서(10 + write(I) write([B) write([BII) flush() close())에서도 14 는 close() 다.
- flush()(13) 는 모양이 같지만, toByteArray 로 이미 꺼낸 배열을 바꿀 수 없으므로 이 위치에서는 의미가 없다.

**결과**(before → after, 무주입 `--timeout 90`):

| 파일 | before | after |
|---|---|---|
| 턴 / (LGT)턴 | FAIL ticks 2/5 paints 0 · DataOutputStream 14 | FAIL ticks 2/3 paints 1 · host panic `class_definition.rs:801` `InvalidMemoryAccess(0)` |
| 서든어택포켓 / lgt 서든어택 포켓 | FAIL ticks 6/12 paints 0 · DataOutputStream 14 | **PASS** paints 112/118 · booted + rendered |

턴의 다음 벽은 이번 칸과 무관한 별건이다. `wie-lgt-class-name-unwrap-guest-boundary-close`(#317) 계열이 다룬다.

**변이**(각각 턴·서든어택포켓):
- 행 제거 → 둘 다 `DataOutputStream vtable index 14` 로 되돌아간다.
- index 13 으로 옮기기 → 같은 벽이 나온다.

**한계**: close 와 flush 는 둘 다 no-op 이라 이름을 바꿔 넣는 변이로는 가를 수 없다. 판별은 호출 위치 하나에 기댄다.

**검증**: 다음 게이트가 전부 통과했다.
- `cargo fmt --check` · `clippy --all -D warnings` · wasm clippy · `cargo +beta clippy`
- `RUST_MIN_STACK=4194304 cargo test --all` 478 passed / 0 failed
- `--workspace --all-targets` clippy 경고 17 (main 17, 증가 0)
- 러너 블록 6줄 전건 PASS

**사용자 영향**: 서든어택포켓이 부팅 세이브를 지나 화면을 그린다.
