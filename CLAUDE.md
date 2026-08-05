@AGENTS.md

## 착수 규율

- ★**티켓 없는 착수 금지**(2026-07-26 dispatch-guardrail-scope-fix-002): 착수 지시에 대응하는
  **티켓 파일이 `~/orchestrator/tasks/` 에 없으면 편집·커밋·push 하지 않는다.**
  읽기전용 조사까지만 하고 **총괄에게 확인을 구한다.**
  (dispatcher 를 거치지 않고 생성된 세션 — `Dispatch(Cowork)` 등 — 의 **원장 밖 변경 방지**.)

## 완주 규율

- ★**완주 = PR 을 열어 둔 상태이지 머지가 아니다.** 브랜치 push + PR 생성까지가 네 몫이고,
  **머지와 브랜치 삭제는 검수 approve 후 별도 `-merge` 티켓의 몫**이다.
  **네 PR 을 네가 머지하지 마라** — CI green 은 필요조건일 뿐 승인이 아니다.
  (정본 = `AGENTS.md` §Git Workflow · Constraint 12. 위 `@AGENTS.md` 로 이미 로드됨.)
  ★이 조항이 리터럴로 적힌 이유: 종전 헌장이 «완주 = main 에 머지» 라고 가르쳐
  **동일 실패형이 5건** 났다(`wie-agents-md-gate2-contradiction-fix`, 2026-08-02).

## 에이전트 개발환경

정본은 `AGENTS.md`(위 `@AGENTS.md` 로 이미 로드됨)다. 이 절은 진입점만 가리킨다.

- **커밋 전 게이트**: `AGENTS.md` §Definition of Done 의 4종. `cargo clippy --workspace` 만으로는
  CI 를 예측하지 못한다 — `-D warnings`·wasm 타깃·`RUST_MIN_STACK=4194304` 까지 맞춰 돌려라.
- ★**축소 금지 목록 = `AGENTS.md` §Constraints 표**(12행 + «Held by you» 절).
  ★**여기에 재열거하지 않는다** — 같은 사실을 두 곳에 적으면 한쪽이 낡는다.
  **ponytail 은 명시되지 않은 요구사항을 범위 밖으로 취급하므로** 그 표에 걸리는 제안은
  false positive 로 간주하고, 집행이 필요하면 별도 티켓으로 올려라.
- **ponytail 모드는 `full` 고정**(`ultra` 전환 금지). `/ponytail-audit` 결과는 **리포트일 뿐**이며
  적용은 별도 티켓이다.
- **MCP 는 현재 0개 등록**(2026-08-05 실측 — `claude mcp list` · 4개 설정파일 전부 빈 `mcpServers`).
  종전의 «3종 상시 사용 가능» 기재는 사실이 아니었다. 실측 근거와 재등록 시 용처는
  `AGENTS.md` §Agent Environment.
