## [2026-09-25] #161 이 떨군 LGT 수리분 3건 처분 — 604/605 행 복원 · free(NULL) 가드 시험 · 0x581 은 코퍼스 91건 중 호출 0 ⇒ 유지 (wie-lgt-swap-dropped-repairs-restore-and-judge)

채택 제안 `2026-09-25-lgt-swap-dropped-repairs-audit#p0`·`#p1`·`#p2`(0254 감사)의 회차다.

### 대전제 재측(origin/main `802652db`)

- `wie-lgt/src/runtime/svc_ids.rs` 에 `0x25c`·`0x25d` 행 **0**, `0x25e => Self::SocketClose` 는 있다. 공용 `wie-wipi-c/src/api/net.rs` 의 `pub async fn` 은 `connect`·`close`·`socket_close` **셋뿐**(쓰기·읽기 없음) ⇒ 공용 배선 대체 불가, 제안 그대로다.
- `context.rs::free` 의 `memory.0 == 0` 가드는 있고(#265), wie-lgt 에 그 가드를 부르는 시험은 **0**.
- `0x581 => Self::Unk16` · `unk16` = `warn + Ok(0)` 확인.
- 중복: 열린 PR #294 가 같은 감사의 **다른 행**(0x415 · 412 · 207)을 복원한다. 604/605 · free · 0x581 은 거기 없다. 같은 파일 두 개(`svc_ids.rs`·`wipi_c.rs`)를 만지지만 삽입 지점은 서로 한 줄 이상 떨어져 있고, 시험은 #294 가 모듈 끝에 붙이므로 이쪽은 `java_system…` 시험 바로 뒤에 넣었다.

### p0 — WIPIC 604/605 행 복원

`02ad8b5c` 와 같은 모양: `SocketWrite = 0x25c` · `SocketRead = 0x25d` enum·`try_from`·디스패치 행 + LGT 로컬 `net_socket_write`/`net_socket_read`(`-1`).
시험 `wipic_svc_604_605_socket_write_read_are_in_the_table` — `try_from(604/605)` 가 그 변형으로 돌아오는지 본다.
**개악**: `0x25c => Self::SocketWrite` 한 줄을 지우면 `svc_ids.rs:411` 에서 panic(FAILED). 디스패치 행을 지우면 match 가 비완전해 컴파일이 안 된다.
★한계(제안 그대로): 헤드리스 부팅은 네트워크 경로에 닿지 않는다 — 테라-영원의혼돈이 오프라인으로 넘어가는지는 이 회차가 재지 않았다.

### p1 — free(NULL) 가드 시험

`LgtWIPICContext` 는 `Jvm` 을 요구해 단위 시험이 어렵다(0230). 그래서 새 인프라 대신 **`alloc`/`free` 본문을 `ArmCore` 만 받는 모듈 함수 두 개(`alloc_indirect`·`free_indirect`)로 뺐다** — trait 메서드는 그것을 부르기만 한다(동작 변경 0).
시험 `free_null_handle_is_a_no_op_and_real_handle_still_frees` — `ArmCore::new` + `Allocator::init` 만으로 `free(0)` 이 `Ok` 이고, 비-null 경로는 alloc → free → 같은 블록 재할당으로 그대로인지 본다.
**개악**: 가드를 `if false && …` 로 죽이면 `context.rs:33` 의 `0 - 4` 가 underflow panic(FAILED).
★release 빌드에서는 underflow 가 wrap 되어 panic 대신 `0xfffffffc` 읽기가 된다 — 시험은 debug(`cargo test`·tarpaulin)에서 돌므로 잡힌다.

### p2 — WIPIC 0x581 관측 판정

영웅서기5 LGT 는 로컬 코퍼스에 없다(0254). 그래서 **LGT 전 코퍼스**를 돌려 0x581 을 부르는 타이틀이 있는지 물었다.
release `wie_validate --inject`(이 브랜치 · `RUST_LOG=wie_lgt=warn,wie_wipi_c=warn`), `game_lab/working/lgt` 54 + `broken/lgt` 37 = **91건**, loadavg 95~133.

| 항목 | 값 |
|---|---|
| 실행 완료 | 91/91 (rc 0 ×35 · 1 ×55 · 134 ×1 = `broken/크로이센` abort) |
| `stub unk16`(= 0x581) 로그가 난 타이틀 | **0** |
| `Unknown LGT WIPIC SVC id 1409` / `0x581` | **0** |
| 채널이 살아 있다는 대조 | 같은 실행에서 `stub unk0` 9타이틀 · `unk15` 6 · `unk13` 5 등 **다른 스텁 11종**이 로그에 났다 |
| 입력 도달 | `input_steps` 27 인 실행 69건 |

⇒ **코퍼스에 0x581 호출자가 없다 → 현 상태(`unk16` warn + Ok(0)) 유지**(티켓 기본값). 「전후 결과」 열은 호출 타이틀이 없어 **비교 대상이 없다**.
★한계: `--inject` 약 20 초 예산 안의 도달 범위다. 영웅서기5 는 `CletWrapper.startApp` 에서 불렀으니(0008) 부팅 직후 경로라 이 예산이 닿는 곳이지만, 더 깊은 곳에서 부르는 타이틀은 못 본다. 근거가 생기면(호출 타이틀 입수) 그때 재판정한다.
★첫 스캔은 `timeout` 이 이 맥에 없어 91건 전부 rc=127 이었고 `unk16` 0건은 **아무것도 안 돌린 0** 이었다 — rc 분포를 보고 버렸다.

### 게이트

`cargo fmt --check` · `cargo clippy --all -D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all`(실패 0) 통과.
