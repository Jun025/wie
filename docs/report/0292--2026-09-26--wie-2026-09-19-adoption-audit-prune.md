## [2026-09-26] 09-19 채택분 감사 정리 — 토큰 판정 통합 · 역공학 노트 이전 · 문구 고정 테스트 완화 (wie-2026-09-19-adoption-audit-prune)

**무엇을**
- 계약 2: 토큰 인용 판정 두 벌(`database.rs` `SLOT8_TOKEN_MAX`/`slot8_token` ↔ `method_table.rs` `UNNAMED_TABLE_TOKEN_MAX`/`unnamed_table_token`/`token_from_bytes`)을 `wie_util::{QUOTABLE_TOKEN_MAX, quotable_token, read_quotable_token}` 하나로 합쳤다. 상한 16 · 인쇄 가능 ASCII 술어는 그대로, 경계 테스트(리터럴 17 · `assert_eq!(MAX, 16)`)는 `wie_util` 로 옮겼다.
- 계약 3: `sort_records` doc 주석 212줄 → 0175 §부록, `ktf_kernel_extension_message` doc 주석 35줄 → 0178 §부록(원문 그대로). 소스에는 요약 + 링크만 남겼다.
- 계약 4: `ktf_kernel_extension_message_says_it_is_ours_not_the_spec_s` → `…_names_the_slot_and_placeholder`(`slot 36` · `MC_knlReserved4` 만) · `unnamed_table_message_carries_the_whole_coordinate` 에서 영문 문장 needle 2개를 빼고 셀렉터·함수·레지스터·토큰 값만 남겼다.
- 계약 5: `game-lab-census-map.mjs` 에서 7월 `summary.tsv` 반사실 재측정(`byCarrier` · `cf*` 카운터 · 70줄 주석)을 걷었다. ★`summary.tsv` 경로 키 조회 자체는 **남겼다** — 오늘의 `game-lab-recensus.sh` 가 쓰는 무손실 입력이라서다. 실 코퍼스(170파일)에서 신·구 판본의 데이터 행을 비교했더니 **171줄 IDENTICAL**이고, 달라진 것은 머리줄 `inputs:` 문구뿐이다.
- 계약 8: `wie_validate.rs` 의 `run` 꼬리(계수 기록 + 두 게이트)를 `judge()` 로 뽑아 `the_gate_is_handed_the_delivered_count_test` 를 행동 단언(0/27·10/27 → UNMEASURED, 27/27 → PASS)으로 바꿨다. 개악 두 가지(기록 줄 `= input_steps_total` · 호출부에 total 먹이기)는 **둘 다 FAILED** 를 냈다. `include_str!` 테스트 2종(`java_exception_layer_is_installed…` · `richness_is_recorded_before…`)은 순수 함수로 뽑을 수 없어서 삭제했다. `inject_unmeasured` 이력 주석은 13줄에서 2줄로 줄였다.
- 계약 7: census·검사기 다듬기 계열 열린 제안 12건을 declined 로 접었다(worklog). slot 8 계열에는 남은 열린 제안이 없다.

**왜** — 09-19 채택분 10건에서 PASS 로 바뀐 게임이 0이었다. 무게는 테스트 커버리지가 아니라 주석과 메타 장치에 몰려 있었다.

**사용자 영향** — 없다. 제품 동작 변경 0, 오류 메시지 문면 불변.

**수치** — 코드(`.rs`·`.mjs`) +98 / −637, **순감 539줄**. 문서는 +257(이전분).
**게이트** — fmt · clippy `-D warnings` · wasm clippy · `+beta` clippy · `RUST_MIN_STACK=4194304 cargo test --all` 전건 rc=0.
**건드리지 않은 것** — `audit-missing-tests.sh` · `check-inflow-marker`(형제 티켓 소관) · `wie-lgt/data/lgt_java_abi.toml` · 게임 호환 활성 티켓들.
