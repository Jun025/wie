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
- 무거운 빌드·테스트(cargo build/test · vitest · next build · tsc -p · wrangler build)는 `~/orchestrator-live/bin/build-slot run -- <cmd>` 로 감싼다 — 머신당 슬롯을 넘으면 기다렸다 돈다(상한 30분 · rc 는 명령 그대로 · 라이브에 없으면 맨 명령).
- ★**측정·재생 스윕도 같다**(2026-10-02 · 회차 측정으로 `wv_main`/`wie_validate` **20개 동시 · 합 223% CPU** · load1 240 · idle 0% — 호스트 부하 가드는 «신규 배차»만 막아 회차 «안»에서 흩뿌린 프로세스를 못 본다 · 선례 otterpebble #1249):
  ⒜에뮬레이터 실행(`wie_validate`·그 사본 `wv_*` · `--inject` 짝 재측 포함)은 동시 실행 상한(**≤ 3**)을 둔 러너로만 돌린다 — 조합마다 `&` 로 한꺼번에 띄우지 마라. 전체 스윕은 `scripts/playability-census.mjs`(기본 `--jobs` ≤ 3 · 호스트 잠금)를 쓴다.
  ★**스윕은 «long 풀»이다 — 스윕 «전체»를 한 번의 `~/orchestrator-live/bin/build-slot run --long -- <러너>` 로 감싼다**(census `run`·짝 재측 묶음·`--max-ticks`/`--timeout` 수 분 이상 장주행). 러너 «안»의 실행은 맨 명령이다 — 안에서 다시 `build-slot` 으로 감싸지 마라(스윕 하나가 short 풀까지 먹는다). 단발 확인 1회(수십 초)만 맨 `build-slot run -- <cmd>`(short).
  ★왜: 빌드 슬롯은 short 3 · long 1 의 2급이다(orchestrator PR #1285). 스윕을 맨 `build-slot run --` 로 보내면 short 풀을 차지하고, 짧은 빌드가 30분을 기다린다(7일 대기의 53% · 나쁜 시간대 점유자 전부가 wie 스윕). 실행마다 `--long` 을 따로 받으면 long 1칸에서 직렬이 된다(실측 2.1배 — `docs/report/0439`).
  ⒝`nohup … &` 로 회차 밖에 떨어뜨리지 마라(부모가 launchd 가 되면 회차 종료·STUCK 판정이 못 본다) — 회차를 끝낼 때 **자기가 띄운 프로세스 0** 을 확인한다.
  ⒞스윕 착수 전 `~/orchestrator-live/bin/host-load-guard --status --recovered` 를 친다 — **rc=0 일 때만** 전 폭으로 돌린다. rc≠0 이면 폭을 줄이거나(≤2) 기다린다(출력 글자로 판정하지 마라).
- ★**축소 금지 목록 = `AGENTS.md` §Constraints 표**(12행 + «Held by you» 절).
  ★**여기에 재열거하지 않는다** — 같은 사실을 두 곳에 적으면 한쪽이 낡는다.
  **ponytail 은 명시되지 않은 요구사항을 범위 밖으로 취급하므로** 그 표에 걸리는 제안은
  false positive 로 간주하고, 집행이 필요하면 별도 티켓으로 올려라.
- **ponytail 모드는 `full` 고정**(`ultra` 전환 금지). `/ponytail-audit` 결과는 **리포트일 뿐**이며
  적용은 별도 티켓이다.
- **MCP 는 현재 0개 등록**(2026-08-05 실측 — `claude mcp list` · 4개 설정파일 전부 빈 `mcpServers`).
  종전의 «3종 상시 사용 가능» 기재는 사실이 아니었다. 실측 근거와 재등록 시 용처는
  `AGENTS.md` §Agent Environment.
