## [2026-09-26] 09-19 채택분 감사 정리 — 토큰 판정 통합 · 역공학 노트 이전 · 문구 고정 테스트 완화 (wie-2026-09-19-adoption-audit-prune)

**무엇을**
- 계약 2: 토큰 인용 판정 두 벌(`database.rs` `SLOT8_TOKEN_MAX`/`slot8_token` ↔ `method_table.rs` `UNNAMED_TABLE_TOKEN_MAX`/`unnamed_table_token`/`token_from_bytes`)을 `wie_util::{QUOTABLE_TOKEN_MAX, quotable_token, read_quotable_token}` 하나로 합쳤다. 상한 16 · 인쇄 가능 ASCII 술어는 그대로, 경계 테스트(리터럴 17 · `assert_eq!(MAX, 16)`)는 `wie_util` 로 옮겼다.
- 계약 3: `sort_records` doc 주석 212줄 → 0175 §부록, `ktf_kernel_extension_message` doc 주석 35줄 → 0178 §부록(원문 그대로). 소스에는 요약 + 링크만 남겼다.
- 계약 4: `ktf_kernel_extension_message_says_it_is_ours_not_the_spec_s` → `…_names_the_slot_and_placeholder`(`slot 36` · `MC_knlReserved4` 만) · `unnamed_table_message_carries_the_whole_coordinate` 에서 영문 문장 needle 2개를 빼고 셀렉터·함수·레지스터·토큰 값만 남겼다.
- 계약 5: `game-lab-census-map.mjs` 에서 7월 `summary.tsv` 반사실 재측정(`byCarrier` · `cf*` 카운터 · 70줄 주석)을 걷었다. ★`summary.tsv` 경로 키 조회 자체는 **남겼다** — 오늘의 `game-lab-recensus.sh` 가 쓰는 무손실 입력이라서다. 실 코퍼스(170파일)에서 신·구 판본의 데이터 행을 비교했더니 **171줄 IDENTICAL**이고, 달라진 것은 머리 2줄이다 — `inputs:` 줄, 그리고 스템 충돌 비대칭 줄의 괄호 문구(「measured and refused — see the inputs line」 → 「docs/report/0191」).
- 계약 8: `wie_validate.rs` 의 `run` 꼬리(계수 기록 + 두 게이트)를 `judge()` 로 뽑아 `the_gate_is_handed_the_delivered_count_test` 를 행동 단언(0/27·10/27 → UNMEASURED, 27/27 → PASS)으로 바꿨다. `judge` 안의 개악 두 가지(M1 기록 줄 `= input_steps_total` · M3 `judge` 안 `inject_unmeasured` 호출에 total 먹이기)는 **둘 다 FAILED** 를 냈다. `run` 이 `judge` 를 부르는 호출은 여기에 들지 않았다. 첫 판본에서 그 호출은 `u64` 위치 인자 두 개를 받았고, 두 값을 바꿔 넣어도 스위트가 green 이었다(게이트② M2). 그래서 계수 지점에서 `InputCount { delivered, scripted }` 를 한 번 만들어 넘기게 고쳤다(wie-2026-09-19-adoption-audit-prune-fix). 이제 같은 스왑은 컴파일 에러(E0061)가 난다. 상수·술어 개악(M4 `QUOTABLE_TOKEN_MAX` 16→64 · M5 `0x20..0x7f`→`0x01..0xff`)은 둘 다 `wie_util` 경계 테스트가 FAILED 로 잡는다. `include_str!` 테스트 2종(`java_exception_layer_is_installed…` · `richness_is_recorded_before…`)은 순수 함수로 뽑을 수 없어서 삭제했다. `inject_unmeasured` 이력 주석은 13줄에서 2줄로 줄였다.
- 계약 7: census·검사기 다듬기 계열 열린 제안 12건을 declined 로 접었다(worklog). slot 8 계열에는 남은 열린 제안이 없다.

**왜** — 09-19 채택분 10건에서 PASS 로 바뀐 게임이 0이었다. 무게는 테스트 커버리지가 아니라 주석과 메타 장치에 몰려 있었다.

**사용자 영향** — 없다. 제품 동작 변경 0, 오류 메시지 문면 불변.

**수치** — 코드(`.rs`·`.mjs`) +98 / −637, **순감 539줄**. 문서는 +257(이전분).
**게이트** — fmt · clippy `-D warnings` · wasm clippy · `+beta` clippy · `RUST_MIN_STACK=4194304 cargo test --all` 전건 rc=0.
**건드리지 않은 것** — `audit-missing-tests.sh` · `check-inflow-marker`(형제 티켓 소관) · `wie-lgt/data/lgt_java_abi.toml` · 게임 호환 활성 티켓들.

**한계** — `run` 이 `InputCount` 를 이름 붙여 잘못 채우면(`InputCount { delivered: inputs.scripted, ..inputs }`) 스위트도 CI 도 잡지 못한다(실측: `wie_cli` 24+11+3 passed). 이름 있는 필드는 그런 편집을 한 토큰 스왑이 아니라 눈에 보이는 편집으로 만들 뿐이다. `main` 이 tally 레이어를 설치하는지도 더 이상 테스트가 잡지 않는다(삭제의 대가).

**게임명 유입** — 이 diff 가 더한 줄에는 **0**(BOUNDED·SUFFIX 전 stem 을 추가 줄에 대조). 도구는 변경 파일 본문 전체를 재므로 아래 표식의 B/S 는 `origin/main` 기존분이다.

<!-- corpus-name-inflow v1 subjects=10 tree=239a3942aca10d98 B=16/10 P=3/1 S=1/1 -->
