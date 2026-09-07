## [2026-09-07] 리소스 폴백 가지는 ★**두 타깃 «모두»에서 도달 0** — 그 «아래»를 단위시험으로 덮었다 (wie-resource-fallback-branch-unexercised-in-both-targets)

**무엇을**: 단위시험 **7건**(`wie_cli/src/filesystem.rs` **5** · `wie_backend/.../file_system.rs` **2**).
★**제품 코드 무접촉** · ★**첫 경로 무접촉** · ★**기존 픽스처 기대값 무접촉** · ★**새 픽스처 0**.

**F1 — «둘 다» 재현했다(추정 0)**. 폴백 자리는 `wie_lgt/src/runtime/wipi_c/context.rs` 의 **두 곳**
(`get_resource_size` 의 `Ok(self.system.filesystem().size(name).await)` · `read_resource` 의
`let Some(size) = … else { FatalError }` + `read(...)`) — 둘 다 **클래스로더 stream 이 `None` 일 때만** 지난다.

| 타깃 | 프로브 | 결과 |
|---|---|---|
| **네이티브** | 두 자리에 `unreachable!()` 심고 `cargo test --all` | ★**167 passed / 0 failed · PROBE 히트 0** ⇒ **도달 0** |
| ★**브라우저(wasm)** | 두 자리에 `panic!()` 심고 **wasm 재빌드 후** `contract-roundtrip.mjs` | ★**46/46 checks passed · rc=0 · PROBE 히트 0** · ★**E-res·F-res 둘 다 green** ⇒ **도달 0** |

★**둘 다 원복**했고 `git status --porcelain` 이 **의도한 두 파일만** 남는 것을 확인했다(깨끗한 wasm 산출물은 스냅샷에서 복원).

**★그래서 무엇을 덮었나 — 「지나게 하기」가 아니라 「지나면 무엇이 옳은가」**:
폴백을 실제로 지나게 하려면 ★**클래스로더가 못 찾는 이름을 게스트가 요청**해야 하고 그건 **새 픽스처**(제안 tradeoff ⑴)다.
⇒ 제안이 「★**그쪽이 먼저다**」로 지목한 tradeoff ⑵ — ★**호스트가 갈리는 «아래쪽»의 단위시험** — 을 했다.

| 자리 | 전 | 후 | 무엇을 단언하나(F3) |
|---|---:|---:|---|
| `wie_cli/src/filesystem.rs` | ★**0** | **5** | `path_for` 의 **정규화·거부 의미**(정상/`.` 제거/`..`·빈 경로 거부/`aid` 위생) |
| `wie_backend/.../file_system.rs` | 7 | **9** | ★**폴백이 부르는 «그 호출쌍»**(`size` → `read(0,size)`) · ★**`\`·`..` 를 두 백엔드가 보기 «전»에 오버레이가 막는다** |
| `wie_web/src/filesystem.rs` | 0 | ★**0(불가)** | `#![cfg(target_arch = "wasm32")]` 라 네이티브 `cargo test` 가 **컴파일조차 하지 않는다** |

**★호스트 divergence 를 «읽어서»가 아니라 «돌려서» 적었다**(제안이 지적한 그 결함):
`/save.dat` → CLI **거부**(`Component::RootDir`) ↔ Web **수용**(`save.dat`) · `a\b` → CLI(unix) **한 파일명** ↔ Web **`a/b`**.
★**오늘 둘 다 도달 불가인 이유**는 `FilesystemOverlay::normalize_guest_path` 가 앞에서 막기 때문이고,
★**그 masking 자체를 시험으로 잠갔다** — 마스킹이 풀리면 무엇이 달라지는지 두 시험이 말한다.

**★양방향**: M1(오버레이 가상층 폴백 제거) → `fallback_size_then_read_pair_round_trips` **FAILED**(20 passed / 2 failed) ·
M2(CLI 의 traversal 거부 제거) → `traversal_and_empty_are_rejected` + `differs_from_web_on_absolute_and_backslash` **FAILED**(3/2).
원복 **바이트 동일** · `cargo test --all` **167 → 174**(+7 = 넣은 시험 수와 일치).

**사용자 영향**: 없다(시험만 · 동작 무접촉).

**★안 한 것 — 재개 지점**: 폴백을 **실제로 지나는** 시험(브라우저 시나리오 포함)은 **하지 않았다**.
필요한 것은 「클래스로더가 못 찾고 파일시스템에는 있는 이름」이고 ⇒ **픽스처 재생성** 또는 `add_virtual` 로 심은 이름을 쓰는
**새 시나리오**다(제안 tradeoff ⑴ · 노력도 **M**). ★**제안 tradeoff ⑶(「죽은 코드면 제거 결정이 값한다」)은 이 회차가 «닫지 않았다»** —
도달 0 은 **두 타깃에서 확인됐지만** 그것이 「죽은 코드」인지 「픽스처가 안 건드리는 산 코드」인지는 **별 결정**이다.
