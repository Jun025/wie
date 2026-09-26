## [2026-09-27] 게스트 힙 고갈 시 오류 생성 무한 재귀 → 호스트 오류로 종료 (wie-2026-09-26-nom3-heap-exhaustion-array-error-recursion-adopt-p0)

채택 제안: `2026-09-26-nom3-overflow-is-dead-frames-after-host-error#p0`.

### 무엇을
- **반증 먼저**: 현 main(`0dd3faab` 기준 분기)에서 합성 시험 — `init_jvm` 뒤 list 절반을 채우고 128B 버킷을 채운 다음
  `instantiate_array("I", 256)` → **스택 넘침 SIGABRT**(`has overflowed its stack`). 경로는 여전히 열려 있었다.
  원인: `instantiate_array`/`instantiate` 가 할당 실패를 `Jvm::exception` 으로 알리는데 그 생성이 메시지 `char[]` 를 할당하고,
  같은 버킷이 차 있어 또 실패 → 또 `Jvm::exception` … (놈3 의 `Jvm::exception ↔ instantiate_array` 교대 프레임 그대로).
- `exception.rs`: `JavaSupportContext` 에 `host_error` 상태 1워드(IDLE/BUILDING/UNBUILDABLE). `begin_host_error` 가
  빌드 중 재진입이면 UNBUILDABLE 로 굳힌다. `unwind` 는 UNBUILDABLE 이면 게스트 catch 대신
  `FatalError("LGT host error unbuildable: … (guest heap exhausted)")` 를 낸다.
- `jvm_support.rs::host_error`: `Jvm::exception` 과 같은 것을 **fallible** 로 빌드(업스트림은 `unwrap`). 재진입이면 할당 없이
  raw 0 자리표시 인스턴스를 돌려 사슬을 끊는다(게스트는 못 본다 — unwind 가 먼저 호스트 오류를 낸다).
  `jvm_support/` 의 6곳(instantiate · instantiate_array · define_class ×2 · JavaMethod::run ×2)을 이것으로 바꿨다.
- `wie-jvm-support::to_wie_err`: 스택 트레이스 포맷이 할당하므로 힙 고갈에서 `unwrap` 패닉 → 실패 시 `FatalError` 로 폴백.

### 왜
업스트림 `JavaError` 에 예외 말고 변형이 없어(RustJava `5b84dd1`) 호스트 오류를 `JvmResult` 로 실어 보낼 수 없다 —
상태는 코어 쪽(지원 컨텍스트 1워드)에 두고, 모든 SVC 가 거치는 `exception::unwind` 한 곳에서 호스트 오류로 바꾼다.

### 측정
| 축 | 값 |
|---|---|
| 합성 시험 main | 스택 넘침 SIGABRT |
| 합성 시험 수정 후 | PASS (여유 힙: WieError 인스턴스 비-0 · 상태 불변 / 고갈: UNBUILDABLE · unwind=FatalError · to_wie_err=FatalError) |
| 개악① 재진입 표지 제거(`entered = true`) | 스택 넘침 SIGABRT (red) |
| 개악② unwind 검사 제거 | assert 실패 (red) |
| `cargo test --all` | 484 passed / 0 failed |
| clippy(stable·wasm·beta) `-D warnings` | rc 0 |
| runner 블록 | draw_j2me·helloworld_ktf·helloworld_lgt·text_j2me PASS · keydraw_ktf/lgt: 기본 max-ticks 에서 UNMEASURED(stop=max-ticks · load 46) → `--max-ticks 400000000` 으로 PASS 27/27 rc=0 |

### 사용자 영향
게스트 힙이 차면 창이 스택 넘침으로 통째로 죽는 대신 «host error unbuildable (guest heap exhausted)» 한 줄로 멈춘다.
힙 여유가 있는 평소 경로는 같은 예외를 게스트에 던진다(합성 시험 첫 단이 잠근다).

### 한계
- 놈3 실측 없음: 이 머신 `game_lab/` 에 놈3 바이너리가 없다(보고서 png 만). 게다가 main 은 #292 의 재시도 고리가 없어 그 경로에 안 닿는다.
- `interface.rs`(NPE·AIOOBE·ArithmeticException·ArrayStoreException 4곳)·`classes/net/wie/*`(7곳)는 여전히 `Jvm::exception` —
  힙 고갈 중 그 경로가 먼저 불리면 재귀 대신 업스트림 `unwrap` 패닉으로 끝난다(재귀는 instantiate 쪽 표지가 막는다).
- 상태는 스레드가 아니라 코어당 하나(`ponytail:` 주석). 빌드 중 스레드 전환이 있으면 오판 가능 — 실측 사례 없음.
- 합성 시험은 128B 버킷 131,072 슬롯을 채우느라 debug 에서 약 9~12 s 걸린다.
