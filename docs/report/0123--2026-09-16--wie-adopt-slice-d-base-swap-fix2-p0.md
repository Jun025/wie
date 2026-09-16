## [2026-09-16] «완성된 줄» 술어를 한 곳으로 모았다 — 제안이 미룬 근거가 실측으로 없었다 (wie-adopt-slice-d-base-swap-fix2-p0)

### 무엇을

채택 제안 `2026-09-16-slice-d-base-swap-fix2#p0`(원문 =
`docs/worklog/2026-09-16-slice-d-base-swap-fix2.json` `proposals[0]`)을 집행했다.

`test-utils/src/platform.rs` 에 **`pub fn guest_line_complete(seen, prefix)`** 하나를 만들고,
그 술어를 독립으로 두 번 쓰고 두 번 틀렸던 호출부 2곳을 그것으로 바꿨다.

```
test-utils/src/lib.rs                |  2 +-
test-utils/src/platform.rs           | 29 +++++++++++++++++++++++++++++
wie-ktf/tests/test_key_reach.rs      | 12 +++++-------
wie-ktf/tests/test_resource_reach.rs | 15 ++++++---------
4 files changed, 41 insertions(+), 17 deletions(-)
```

삽입 29줄의 대부분은 **두 시험이 왜 틀렸는지의 서사**이고, 그것을 두 부에서 한 부로 줄인 것이 이 회차다.
제품 코드 **0줄**(시험 지원 크레이트 + 통합 시험 2개).

### 왜 «지금»인가 — 제안이 스스로 미룬 근거를 반증했다

제안의 `tradeoff` 는 이렇게 미뤘다: 「n=2 에 공용 헬퍼는 과설계일 수 있다 … **지금 만들면 통합 시험
바이너리 사이 모듈 공유(`#[path]` 또는 `tests/common/mod.rs`)라는 새 구조가 생긴다**」.

★**그 비용은 실재하지 않는다.** `test-utils` 는 이미 workspace 크레이트(`Cargo.toml:117`
`test-utils = { path = "test-utils" }`)이고 `wie-ktf` 의 `[dev-dependencies]` 에 등재돼 있으며,
★**두 시험이 이미 그것을 `use` 하고 있었다**(`use test_utils::{TestPlatform, TestPlatformEvent};`).
⇒ 이번 변경이 만든 새 구조는 **0** 이고, 늘어난 것은 `use` 목록의 이름 하나와 재수출 1행이다.

⇒ 미룰 근거가 없으면 남는 것은 **비용**뿐이고, 그 비용은 이미 두 번 지불됐다(아래).
★**제안을 기각하지 않았다** — 기각했어야 할 것은 제안의 *결론*이 아니라 그 *tradeoff 문장*이다.

### 대전제 재고

| 제안이 선 전제 | 재측(2026-09-16 · `origin/main` `3a1723bf`) | 판정 |
|---|---|---|
| 두 시험이 같은 결함을 독립으로 가졌다 | 두 파일에 같은 식이 각각 인라인 · 주석도 각각 6줄/7줄 | ★참(그대로) |
| 아직 안 모았다 | 공유 술어 **0건** | ★참 — 해소되지 않았다 |
| 「모으면 새 모듈 공유 구조가 생긴다」 | ★**거짓** — `test-utils` 가 이미 있고 둘 다 이미 쓴다 | ★반증 |
| `test_resource_reach` 는 **구 base 에서** 잠복했다 | ★**오늘은 잠복이 아니다** — 이 base 에서 순진한 술어로 **실제 FAILED** | ★강화 |

ⓑ **같은 것을 하는 축 0**(공유 술어 부재). ⓒ **기각 안 함** — 위 셋째 줄만 반증했다.

**세 번째 소비자는 오늘 없다**(TestPlatformEvent::Stdout 사용 6파일 전수):
`wie-ktf/tests/test_helloworld.rs` · `wie-lgt/tests/test_helloworld.rs`(2케이스)는 `while !exited` —
픽스처가 **스스로 종료**하므로 절단 경쟁이 없다. `wie_j2me/tests/test_boot.rs` 는 `paints` 폴링이고,
게다가 **고아**다(workspace member 는 `wie-j2me` — 하이픈. 조각 E 소관이라 무접촉).

### 검증 — 양방향 개악 대조

개악은 **시험 파일이 아니라 두 시험이 공유하는 «구현 한 곳»**에 넣었다(`guest_line_complete` 본문을
종전의 순진한 `seen.contains(prefix)` 로 되돌림). 판정은 게스트의 **실물 stdout** 이다 — 상수 대 상수가 아니다.

| 회차 | `test_key_reach` | `test_resource_reach` |
|---|---|---|
| 정상 | **ok** | **ok** |
| ★**개악**(공유 구현 1곳) | **FAILED** | **FAILED** |
| 복원 | **ok** | **ok** |

★**한 번의 개악이 두 소비자를 동시에 물었다 — 그것이 이 회차가 산 것의 전부다.**
잔여 개악 0(`/usr/bin/grep -c 'seen.contains(prefix)'` → **0**).

★★**`--no-fail-fast` 가 없으면 이 표를 못 만든다** — 붙이지 않으면 cargo 가 첫 실패 타깃에서 멈춰
**두 번째 소비자가 보이지도 않는다**(실측: 1건만 RED 로 세어졌다). 조각 D `-fix2` 가 `201·202` 를
부분 계수로 잡았던 것과 **같은 함정**이다.

회귀 0(★`tail` 아님 — `test result:` **42줄 전부**를 `awk` 합산):

```
cargo fmt --all -- --check                                    rc=0
cargo clippy --all -- -D warnings                             rc=0
cargo clippy --target wasm32-unknown-unknown -- -D warnings   rc=0
cargo +beta clippy --all -- -D warnings                       rc=0
RUST_MIN_STACK=4194304 cargo test --all --no-fail-fast   42 스위트 · 384 passed · 0 failed · 0 ignored
npm run audit                                                 rc=0
check-worklog-json / check-doc-liveness-parity / check-worklog-coverage   전건 rc=0
```

384 는 `origin/main` 기준과 **동수**다(조각 D `-fix2` 가 기록한 그 수). 엔진 무접촉이라
`wie_validate` 러너 블록은 돌리지 않았다 — 안 돈 것을 green 으로 적지 않는다.

### 무엇을 잃는가 / 안 하면 무엇이 나쁜가

★**「잃는 것이 없다」가 아니다.** 네 가지를 실제로 지불했다.

- **⑴ n=2 에 간접이 생겼다.** `test_key_reach.rs` 를 읽는 사람이 술어 **식**을 그 자리에서 못 본다.
  포인터 주석으로 **줄였을 뿐 없애지 못했다** — 이것이 제안이 미루려던 바로 그 비용이고, 여전히 참이다.
- **⑵ `test-utils` 의 소임이 번졌다.** 그 크레이트는 호스트 대역(`platform`·`filesystem`·`jvm`)인데
  문자열 술어가 하나 붙었다. `TestPlatformEvent::Stdout` **바로 옆**에 두고 **새 파일을 만들지 않은** 것이 그 완화다.
- **⑶ 공개 API 가 됐다.** 앞으로 이 술어를 고치면 `wie-ktf` 밖 소비자까지 함께 움직인다(오늘 0 —
  그러나 «세 번째»를 받겠다는 것이 이 제안의 목적이므로 이 비용은 **의도된 것**이다).
- **⑷ 빌드 결합.** `test-utils` 를 건드리면 그것을 dev-dep 으로 쓰는 크레이트가 함께 다시 돈다(수 초 단위).

**안 하면**: 세 번째 stdout 폴링 시험이 같은 줄을 세 번째로 틀린다. 가정이 아니다 —
**독립 저자 둘이 이미 틀렸고**, 한 번은 게이트② **critical 반려**를 냈으며,
없는 버그를 쫓느라 **영구 기록 4곳**과 **effort M 제안 하나**가 만들어졌다. 그리고 그 오류는
**조용하다** — 통과할 때도 값이 잘려 있을 수 있어, 다음에도 「엔진 회귀」로 읽힌다.

### 범위 밖 — 일부러 안 한 것

- 형제 `#p1`(아티팩트 크기 예산) · `#p2`(브라우저 실부팅) · `#p3`(upstream 워크플로 채택) **무접촉** — 각자 별 티켓이다.
- 유닛 시험을 **넣지 않았다**: 두 통합 시험이 「접두사 있음+개행 없음」·「접두사 없음」 두 경로를 모두 밟고
  개악 대조가 그것을 RED 로 보인다. n=1 유닛 단언은 **상수 대 상수**라 이 회차의 증거로도 못 쓴다.
- `tests/common/mod.rs`·`#[path]` 를 **만들지 않았다**(필요가 없다 — 위 §왜).
- 리팩터·이름 변경·주변 정리 **0**. 고아 `wie_j2me/tests/` **무접촉**.
