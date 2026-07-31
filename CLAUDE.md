@AGENTS.md

## 착수 규율

- ★**티켓 없는 착수 금지**(2026-07-26 dispatch-guardrail-scope-fix-002): 착수 지시에 대응하는
  **티켓 파일이 `~/orchestrator/tasks/` 에 없으면 편집·커밋·push 하지 않는다.**
  읽기전용 조사까지만 하고 **총괄에게 확인을 구한다.**
  (dispatcher 를 거치지 않고 생성된 세션 — `Dispatch(Cowork)` 등 — 의 **원장 밖 변경 방지**.)

## 에이전트 개발환경

정본은 `AGENTS.md`(위 `@AGENTS.md` 로 이미 로드됨)다. 이 절은 진입점만 가리킨다.

- **커밋 전 게이트**: `AGENTS.md` §CI-parity commands. `cargo clippy --workspace` 만으로는
  CI 를 예측하지 못한다 — `-D warnings`·wasm 타깃·`RUST_MIN_STACK=4194304` 까지 맞춰 돌려라.
- ★**축소 금지 목록**: `AGENTS.md` §Hard Requirements. 게이트 4종·커버리지 배선·featurephone
  계약 핀과 always-run 래퍼·무-ignore `cargo audit`·`no_std`·`wie_web` cfg 게이팅·정확버전
  핀과 `[patch]` 표·게임바이트 금지·시크릿 규율·D1 마이그레이션 정책·브랜치 규율.
  **ponytail 은 명시되지 않은 요구사항을 범위 밖으로 취급한다** — 그 목록에 걸리는 제안은
  false positive 로 간주하고, 집행이 필요하면 별도 티켓으로 올려라.
- **ponytail 모드는 `full` 고정**(`ultra` 전환 금지). `/ponytail-audit` 결과는 **리포트일 뿐**이며
  적용은 별도 티켓이다.
- **MCP 3종 사용 시점**: `AGENTS.md` §MCP servers — context7(외부 crate API 확인)·
  serena(심볼 단위 탐색)·playwright(이 repo 는 `web/` 웹 표면이 있으므로 **해당됨**).
