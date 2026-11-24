## `AGENTS.md` 는 기동 때 실리지 않는다 — 필요한 절을 그때 읽어라

(2026-11-24 · `wie-claude-md-stop-importing-agents-md-wholesale-load-on-demand` — 종전 이 파일 첫 줄의 통째 import 가 65KB 를 모든 세션에 실었다. 정본은 그대로 `AGENTS.md` 이고, 이 절은 «언제 어느 절을 읽는가»만 적는다.)

- ★**세션 첫 행동: `AGENTS.md` 를 맨 위부터 «## Definition of Done» 제목 앞까지 읽어라** — «## Goal» · «## Constraints» · «### Held by you, not by a machine»(맨 앞 40여 줄). 그 표가 축소 금지 목록의 정본이다.
- 그 읽기가 빠졌을 때를 대비한 **제목 목록**이다 — 본문·근거는 `AGENTS.md` 에만 있으니 이 문구로 판단하지 말고 그 절을 읽어라:
  1 커밋 전 게이트 4종 · 2 coverage 배선 · 3 featurephone 엔진 계약 · 4 `contract` 잡 always-run · 5 `cargo audit` 무시 0 · 6 엔진 크레이트 `no_std` · 7 `wie_featurephone` 의 wasm32 게이트 · 8 버전 핀 · RustJava `rev` 핀 · 9 게임 바이트 0 · 10 시크릿 · 11 D1 마이그레이션 · 12 `main` 직접 커밋 금지.
  Held by you: 게임 바이트 0 · 시크릿 · 외부 계약은 참조만 · 공개 이력 재작성 금지 · 로컬 표면만.
- **절 색인** — 왼쪽 일을 «하기 전에» 오른쪽 절을 읽는다. 제목으로 찾는다(`grep -n '^##' AGENTS.md` · 줄번호는 매주 바뀐다). 컨텍스트가 압축된 뒤에는 읽었던 절이 남아 있지 않다 — 다시 읽어라.
  - 커밋 · PR 전(문서만 고쳤어도) · 엔진 코드를 만졌다 · `wie_validate` 의 FAIL/UNMEASURED 를 읽는다 · PR 의 CI 를 읽는다 → «## Definition of Done» 의 «### The four gates»
  - `web/` · `functions/` · `scripts/` · `migrations/` 를 만진다 · local-only 스크립트(census · smoke_gate · audio-probe · perf-watch …)를 돌린다 · 회신에 «게임 파일명 유입» 수를 적는다(다시 세지 말고 `corpus-name-inflow.mjs` 를 돌린다) → «### Web-surface commands»
  - PR 을 내기 전(모든 회차 — 회차 기록은 회차마다의 의무다) · 회차 기록(`docs/report/`) · worklog · player-updates · 원장 파일 충돌 · 후속 제안 → «### Landing paperwork»(그 아래 «#### Worklog» · «#### Proposal threshold»)
  - 브랜치 · 커밋 · PR 을 연다 → «### Git Workflow»
  - `-merge` 티켓 → «### For the `-merge` task only»(머지 방식 · `--delete-branch`). ★그 절이 전부가 아니다 — 착지시키는 회차(게이트③)는 아래 셋도 읽고, 한 일을 회신에 인용한다:
    - «### Web-surface commands» 의 `npm run verify` 항 — 착지 뒤 운영 별칭(`WIE_BASE=https://wie-web.pages.dev`)에 `scripts/verify-browser.mjs` 를 손으로 돌린다(착지마다 · 명령과 통과 기준은 그 항에). 배포 스텝이 red 면 착지시킨 회차가 소유한다.
    - «### The four gates» 말미 — 주간 `rust.yml` · `coverage.yml` 의 schedule 실행을 PR 검사 옆에서 «보러 간다». red 면 그 뒤 첫 게이트③ 회차가 소유한다(티켓 발권 · 손 dispatch 는 그 절의 세 경우대로).
    - «### Documented-command liveness» — 주간 doc-liveness 의 마지막 schedule 실행도 같은 소유 규칙이다.
  - `AGENTS.md` 에 fenced `sh` 블록을 더한다 · 주간 doc-liveness 가 red → «### Documented-command liveness»
  - 같은 증상을 전에 겪었는지 · required check · 버전 핀 · paths-filter · `Cargo.toml` 을 만진다(주석만이어도 착지하면 릴리스가 선다) → «## Incident ledger (사건 대장)»
  - 코드 스타일 · 크레이트 지도 · ponytail/MCP → «## Reference»(«### Code Style» · «### Layout» · «### Agent Environment»)

## 착수 규율

- ★**티켓 없는 착수 금지**(2026-07-26 dispatch-guardrail-scope-fix-002): 착수 지시에 대응하는
  **티켓 파일이 `~/orchestrator/tasks/` 에 없으면 편집·커밋·push 하지 않는다.**
  읽기전용 조사까지만 하고 **총괄에게 확인을 구한다.**
  (dispatcher 를 거치지 않고 생성된 세션 — `Dispatch(Cowork)` 등 — 의 **원장 밖 변경 방지**.)

## 게임 파일 무수정 원칙

(2026-11-22 · 운영자 지시 2026-11-16 · 처분 결정 2026-11-20 — 「이용자가 가진 원본이 그대로 도는가」가 기준이다.)

- ⑴ **디스크의 게임 파일 · 코퍼스(`game_lab/`) · 이용자 업로드는 바이트를 고치지 않는다.** 다시 묶기 · 한 바이트 수정 · 이름 바꾸기 전부 해당한다. 대조 실험용 수정 사본은 repo 와 코퍼스 밖 스크래치에만 둔다.
- ⑵ **지원 판정은 원본 파일로만 한다** — `compat.json` 행의 `sha256` 은 고치지 않은 파일의 것이고, 측정도 그 파일로 한다.
- ⑶ **새 게임을 돌리는 길은 엔진 일반 구현이 먼저다.** 한 판에만 걸리는 키 행(해시 · AID 로 고른 표의 행)은 일반 구현이 왜 불가한지 회신에 근거를 적고, **같은 PR 에서 장부 `docs/contracts/runtime-adjustments.tsv` 에 올릴 때만** 더한다. 장부에 없는 엔진 행 · 엔진에 없는 장부 행은 `cargo test`(`the_ledger_lists_exactly_*`)가 red 로 내고, 장부와 어긋난 `compat.json` 의 `runtimeAdjustments` 칸은 `node scripts/player-data.mjs` 가 red 로 낸다 — 장부를 고친 뒤 `node scripts/player-data.mjs normalize`(칸 손 기입 금지).
- ⑷ **잠금 · 인증 해제 행(`lock-lift`)은 운영자가 그 판을 이름으로 지시했을 때만 더한다.**
- 지금 걸리는 행 · 표별 범위(zip 꾸러미에서만 걸리는 표 등) · 메모리에서 고쳐 쓰는 코드 자리 수의 정본은 그 장부다. ★여기에 재열거하지 않는다.

## 완주 규율

- ★**완주 = PR 을 열어 둔 상태이지 머지가 아니다.** 브랜치 push + PR 생성까지가 네 몫이고,
  **머지와 브랜치 삭제는 검수 approve 후 별도 `-merge` 티켓의 몫**이다.
  **네 PR 을 네가 머지하지 마라** — CI green 은 필요조건일 뿐 승인이 아니다.
  (정본 = `AGENTS.md` §Git Workflow · Constraint 12.)
  ★이 조항이 리터럴로 적힌 이유: 종전 헌장이 «완주 = main 에 머지» 라고 가르쳐
  **동일 실패형이 5건** 났다(`wie-agents-md-gate2-contradiction-fix`, 2026-08-02).

## 에이전트 개발환경

정본은 `AGENTS.md` 다(기동 때 실리지 않는다 — 맨 위 색인). 이 절은 진입점만 가리킨다.

- **커밋 전 게이트**: `AGENTS.md` §Definition of Done 의 4종. `cargo clippy --workspace` 만으로는
  CI 를 예측하지 못한다 — `-D warnings`·wasm 타깃·`RUST_MIN_STACK=4194304` 까지 맞춰 돌려라.
- 무거운 빌드·테스트(cargo build/test · vitest · next build · tsc -p · wrangler build)는 `~/orchestrator-live/bin/build-slot run -- <cmd>` 로 감싼다 — 머신당 슬롯을 넘으면 기다렸다 돈다(상한 30분 · rc 는 명령 그대로 · 라이브에 없으면 맨 명령).
- ★**측정·재생 스윕도 같다**(2026-10-02 · 회차 측정으로 `wv_main`/`wie_validate` **20개 동시 · 합 223% CPU** · load1 240 · idle 0% — 호스트 부하 가드는 «신규 배차»만 막아 회차 «안»에서 흩뿌린 프로세스를 못 본다 · 선례 otterpebble #1249):
  ⒜에뮬레이터 실행(`wie_validate`·그 사본 `wv_*` · `--inject` 짝 재측 포함)은 동시 실행 상한(**≤ 3**)을 둔 러너로만 돌린다 — 조합마다 `&` 로 한꺼번에 띄우지 마라. 전체 스윕은 `scripts/playability-census.mjs`(기본 `--jobs` ≤ 3 · 호스트 잠금)를 쓴다.
  ★**스윕은 «long 풀»이다 — 드라이버는 phase(게임/조합 묶음) 마다 `~/orchestrator-live/bin/build-slot run --long -- <phase 명령>` 을 잡고 반납한다 · 임대 1건 ≤ 30분 목표.** census 는 `bash scripts/census-drive.sh <run 인자…>` 가 그 루프다(`run --budget 900` 1회 = 임대 1건 · exit 3 이면 다음 임대). phase 명령 «안»의 실행은 맨 명령이다 — 안에서 다시 `build-slot` 으로 감싸지 마라(스윕 하나가 short 풀까지 먹는다). 단발 확인 1회(수십 초)만 맨 `build-slot run -- <cmd>`(short).
  ★**금지 예 — «ONE lease»: 스윕 «전체»를 한 번의 `build-slot run --long` 으로 감싸기**(종전 이 줄의 지시였다). 2026-10-05~08 long 임대 held_max **26974s(7.5h)**·19861s·17942s 의 점유자가 전부 그 형상(census `run` · scratch `drive*.sh`)이었고 그 사이 long 수요 max 8.
  ★왜: 빌드 슬롯은 short 3 · long 2 의 2급이다(orchestrator PR #1285 · #1302). 스윕을 맨 `build-slot run --` 로 보내면 short 풀을 차지하고, 짧은 빌드가 30분을 기다린다(7일 대기의 53% · 나쁜 시간대 점유자 전부가 wie 스윕). 실행(게임 1개)마다 `--long` 을 따로 받으면 long 칸에서 직렬이 된다(실측 2.1배 — `docs/report/0439`) — 그래서 단위는 «실행»도 «스윕 전체»도 아닌 «phase(≈15분 묶음)»다.
  ★**드라이버는 repo `scripts/` 에 둔다 — 스크래치에 복제하지 마라**(scratch `drive*.sh` 사본들이 낡은 «ONE lease» 형상을 계속 베꼈다).
  ⒝`nohup … &` 로 회차 밖에 떨어뜨리지 마라(부모가 launchd 가 되면 회차 종료·STUCK 판정이 못 본다) — 회차를 끝낼 때 **자기가 띄운 프로세스 0** 을 확인한다.
  ⒞드라이버 안에 `host-load-guard --status --recovered` 대기 폴링을 넣지 마라 — 임대 대기가 그 역할이다(hold-mode 는 observe 라 그 폴링은 아무것도 기다리지 않는다). 폭은 census 기본 `--jobs`(≤ 3)를 따른다.
- ★**축소 금지 목록 = `AGENTS.md` §Constraints 표**(12행 + «Held by you» 절).
  ★**여기에 재열거하지 않는다**(맨 위의 것은 제목뿐이다) — 같은 사실을 두 곳에 적으면 한쪽이 낡는다.
  **ponytail 은 명시되지 않은 요구사항을 범위 밖으로 취급하므로** 그 표에 걸리는 제안은
  false positive 로 간주하고, 집행이 필요하면 별도 티켓으로 올려라.
- **ponytail 모드는 `full` 고정**(`ultra` 전환 금지). `/ponytail-audit` 결과는 **리포트일 뿐**이며
  적용은 별도 티켓이다.
- **MCP 는 현재 0개 등록**(2026-08-05 실측 — `claude mcp list` · 4개 설정파일 전부 빈 `mcpServers`).
  종전의 «3종 상시 사용 가능» 기재는 사실이 아니었다. 실측 근거와 재등록 시 용처는
  `AGENTS.md` §Agent Environment.
