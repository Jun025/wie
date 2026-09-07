## [2026-09-07] 형제 `-18` 두 자리에 시험을 넣었다 — 「원래 옳던 자리」가 지는 것이 없었다 (wie-sibling-shortbuf-sites-have-no-tests)

**무엇을**: `wie_wipi_c/src/api/kernel.rs` 의 시험 모듈에 **2건**(`get_system_property` · `get_program_name` 의
버퍼-부족 분기). ★**제품 코드 무접촉** — diff hunk **3건이 전부 `mod test` 안**이고 제품 구간 hunk **0**이며,
`return Ok(-18)` 세 줄이 **그대로** 있다.

**왜**: 직전 회차(`wie-get-resource-shortbuf-code-is-not-in-the-abi` · 착지 `a862dfb4`)가 `get_resource` 를
`-1 → -18` 로 통일하며 시험을 얻었는데, ★**같은 조건에 이미 `-18` 을 쓰던 형제 두 자리는 여전히 «지는 것이 없었다».**
⇒ ★**「고친 자리」는 지켜지고 「원래 옳던 자리」는 안 지켜지는** 비대칭이었다.

**F1 — 내가 셌다**:
⑴ 두 분기 모두 `return Ok(-18); // M_E_SHORTBUF` 로 같은 형태다(`get_system_property` 는 `bytes.len() + 1 > buf_size`,
`get_program_name` 은 `buf_size < aid.len() + 1`).
⑵ ★**시험 0건 확인** — `kernel.rs` 의 시험 5건 중 그 둘을 부르는 것은 `test_get_system_property_min` 하나이고
그것은 **성공 경로**(`buf_size 16` → `0`)만 단언한다. `get_program_name` 은 저장소 전체에서 시험 참조가 **0**이다.
⑶ 본 = `test_resource_larger_than_buffer_is_rejected` — `TestContext` 로 **짧은 버퍼 → `-18`** 과 **정확한 버퍼 → 성공**을
한 시험에서 함께 단언한다. ★**그 형태를 그대로 따랐다**(새 형태 발명 0).

**기대값을 «수»로 적었다** — `M_E_SHORTBUF` 는 이 저장소·핀 저장소 어디에도 **정의가 0건**이고 주석에만 산다
(직전 게이트②가 확인한 사실). ⇒ 상수 이름을 쓰면 두 번째 진실원이 된다.

**★F3 개악 대조 — 두 자리가 «각각» red 이고 서로 «독립»이다**:

| 개악 | 결과 |
|---|---|
| M1 `get_system_property` `-18 → -1` | ★`test_system_property_larger_than_buffer_is_rejected` **FAILED**(`left: -1 · right: -18`) · **12 passed / 1 failed** — ★형제 시험은 **green** |
| M2 `get_program_name` `-18 → -9` | ★`test_program_name_larger_than_buffer_is_rejected` **FAILED**(`left: -9 · right: -18`) · **12 passed / 1 failed** — ★형제 시험은 **green** |
| 원복 | ★`cmp` **바이트 동일** · **13 passed / 0 failed** |

⇒ ★**각 시험이 «자기 자리»만 문다**(위양성 0 · 축이 겹치지 않는다).

**사용자 영향**: 없다 — 반환값·동작 **무접촉**. `cargo test --all` **161 → 163**(+2 = 이 회차가 넣은 시험 수와 일치).
