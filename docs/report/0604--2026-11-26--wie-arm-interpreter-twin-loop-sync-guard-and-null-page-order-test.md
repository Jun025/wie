## [2026-11-26] ARM 해석기 쌍둥이 루프 — upstream 루프 본문 안 표지 · 멈춤 갈래 수 단언 · 널 페이지 안의 끝 주소 1건 · 주석 수치 정정 (wie-arm-interpreter-twin-loop-sync-guard-and-null-page-order-test)

**무엇을**: `wie-core-arm/src/engine/arm32_cpu.rs` 한 파일. ① upstream 의 `run` 루프 «본문 안»(널 페이지 검사 바로 위 · 이미 fork 가 고친 덩어리)에
«`run_interpreter` 가 이 루프의 쌍둥이다 — 여기 멈춤을 더하면 그쪽에도» 주석 2줄. ② 동기화 시험
`the_loop_without_a_backend_stops_where_the_loop_with_one_does` 에 끝 주소가 널 페이지 안(SVC 벡터 0x08)인 경우 1건(7 → 8가지)과
두 루프의 «멈춤 갈래 수» 단언(`run_interpreter` 6 · upstream 루프 7). ③ `run_interpreter` 머리 주석의 「−7%」를 0603 §3 의 평균 −10.7% 로 고치고
−7.3% 는 세 회 중 최소라고 적었다. 두 루프의 동작은 그대로다(제품 쪽 변경은 주석뿐).

**왜**: PR #635 게이트② 검수의 minor F1 · F2 · F3. 0603 이 루프를 둘로 갈랐고 둘이 어긋나는 것을 잡는 장치는 시험 1건이었는데, 그 시험에 구멍이 둘 있었다 —
«널 페이지 검사가 끝 주소 검사보다 먼저»라는 순서를 붙드는 경우가 없었고(검수의 개악 Q5 가 green 으로 살아남았다), upstream 이 루프에 «새» 멈춤을
더하면 있는 경우들은 그것을 밟지 않아 green 인 채 어긋난다.

**사용자 영향**: 없다. 그리는 내용 · 속도 · 판정 · 지원 목록 그대로. 고지 없음(`docs/player-updates` 0건).

### 1. 멈춤 갈래 수를 어떻게 세나

실행해서는 «갈래가 몇 개인가»를 셀 수 없다 — 시험이 밟는 것은 있는 갈래뿐이다. 그래서 소스를 센다: 시험이 자기 파일을 `include_str!` 로 읽어
각 함수의 `let stop_reason = loop {` 부터 그 루프를 닫는 `};` 까지에서, 주석 줄을 빼고 `break ` · `return ` · `?` 가 있는 줄 수를 센다.

| 루프 | 줄 수 | 그 줄들 |
|---|---|---|
| `run_interpreter` | 6 | SVC 결과(`break …?`) · 널 분기 오류 · 끝 · 예산 · 미정의 명령 · 메모리 오류 |
| upstream `run` 의 루프 | 7 | 위 6 + 컴파일된 코드의 게스트 폴트(백엔드가 없으면 닿지 않는다) |

수가 달라지면 단언 문구가 할 일을 말한다: 그 하나 말고는 두 루프 모두에 넣고, 시험의 경우에 더하고, 그 다음에 두 수를 고친다.

### 2. 개악 — 한쪽만 고치면 red 인가

`cargo test -p wie-core-arm --lib the_loop_without_a_backend`(작업 트리에 차례로 적용 · 회마다 원본 복원 · short 임대 1건).

| 개악 | 결과 |
|---|---|
| M1 `run_interpreter` 에만 멈춤 1개 추가 | **red** — `(7, 7)` ≠ `(6, 7)` |
| M2 upstream 루프에만 멈춤 1개 추가(병합 뒤 어긋남 모의) | **red** — `(6, 8)` ≠ `(6, 7)` |
| M3 `run_interpreter` 에서 끝 검사를 널 페이지 검사 앞으로(검수의 Q5) | **red** — `"end after 2"` ≠ `"svc 42 lr=0x1004 spsr=0x3f after 2"` |
| M4 upstream 루프에서 같은 순서 바꿈 | **red** — 두 루프의 결과가 갈린다(`svc …` ≠ `end after 2`) |
| M5 두 루프에 같은 멈춤을 더하고 수는 그대로 | **red** — `(7, 8)` ≠ `(6, 7)`(경우를 더하라는 문구로 간다) |

고치지 않은 트리: 크레이트 179 passed(시험 수는 그대로 — 경우와 단언을 있는 시험 안에 넣었다).

### 3. 한계

- 수는 «줄»을 센다. 한 줄에 나감이 둘이면 하나로 세고, 루프를 닫는 줄의 들여쓰기나 `let stop_reason = loop {` 문구가 바뀌면 수가 틀어져 red 다
  (조용히 통과하는 쪽이 아니라 시끄러운 쪽으로 틀린다 — 표지를 못 찾으면 0 이거나 파일 끝까지 센다).
- 세는 것은 «멈춤»뿐이다. upstream 이 멈추지 않는 갈래(상태를 바꾸고 계속 도는 것)를 더하면 수는 그대로다 — 그것은 있는 8가지 경우와 본문 안 표지의 몫이다.
- 널 페이지 안의 끝 주소는 제품 호출자가 넘기지 않는 값이다(`RUN_FUNCTION_LR` 0x7f000000 · 게스트 주소). 이 경우는 지금 도는 경로가 아니라 두 루프의 순서를 붙든다.
- upstream 충돌 자리는 늘지 않았다: 표지 2줄은 fork 가 이미 고친 덩어리(널 페이지 검사 위 주석)에 이어 붙였다.

### 4. 검사

- 네 게이트 + beta(로컬 · 자기 `target` · `build-slot` 경유): `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 ·
  `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` rc 0(`test result: ok` 72줄 ·
  1054 passed · FAILED 0 — 0603 과 같은 수) · `cargo +beta clippy --all -- -D warnings` rc 0.
- 러너 블록 6건 전부 `PASS` · rc 0(`keydraw_ktf` paints 79 · `keydraw_lgt` paints 55 · 둘 다 `content` true). 제품 쪽 변경이 주석뿐이라 기대한 대로다.
- `check-worklog-json` OK · `check-docs-report-serial` OK(0604 · 열린 PR claim 충돌 0).
- 하지 않은 것: wasm 빌드 · 계약 왕복 · `smoke_gate` · 프로파일 — 제품 코드가 주석만 바뀌어 다시 잴 것이 없다. PR 의 CI 결과는 회신에 적는다.

### 5. 그 밖

- 검수 F5 의 스크래치 `~/scratch/wie-aotcheck`(179M · 796 파일)를 지웠다 — 지우기 전에 본 것: 전 파일의 수정 시각이 그 회차 창(11-26 09:24 ~ 11:10) 안 ·
  심링크 0 · `.git` 0 · 워크트리 등록 0 · 그 경로를 쥔 프로세스 0.
- 제안 0건. 검수 F6(upstream 루프 안에 남은 fork 검사 3곳)은 0603 이 이미 문턱 아래로 적었고 이 회차 범위 밖이다.
