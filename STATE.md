# STATE

> 실측 기준일 **2026-08-15**. `## 다음` 은 **2026-08-19 재실측**
> (`wie-lgt-svc-1409-unknown-and-state-stale-next` — 종전 ①② PR #54·#46 은 둘 다 머지돼 해소).
> 직전 갱신은 2026-08-08 이었고 그 사이 `## 다음` 4항 중 2항이 조용히 해소돼 레인이 25시간 굶었다.
> ★**`## 다음` 이 낡으면 이 레인은 굶는다** — 착지할 때마다 갱신하라(`AGENTS.md` §Session Discipline).
> ★★**그리고 «새로 쓰는 항목»도 diff 를 열고 써라** — 초판의 ①②는 PR 제목·개설일만 보고 작성돼
> **둘 다 사실과 어긋났다**(게이트② 반려). 낡음을 지운 자리에 새 부정확을 심으면 병은 그대로다.

## 진행중
- `wie-lane-restart-upstream-carryover-and-main-divergence` — 본 문서 갱신 + `## 다음` 재판정.
  브랜치 `docs/wie-lane-restart-state-refresh`. **문서 전용 · 게이트② 리뷰 대기.**
- `wie-lgt-svc-1409-unknown-and-state-stale-next` — 영웅서기5 LGT `Unknown SVC id 1409`(upstream #1260)를
  **표 등재 + 미지원 예외**까지 처리 + 본 문서 `## 다음` ①② 해소 표기. 브랜치 `feat/wie-lgt-svc-1409`.
  **게이트② 리뷰 대기.**
- ★**적체 PR 0건** — 2026-08-19 실측 `gh pr list -R Jun025/wie --state open` = `[]`.

## 완료 (최근)
- 2026-08-16: `AGENTS.md` **재구조화 착지** (PR #54 `41721671`, `wie-agents-md-declarative-restructure`) —
  Goal/Constraints/DoD/사건 대장 골격 + Hard Req 12항 → 잠금 테이블. **16,888 → 13,099 바이트**(−22.4%).
  지도 4종은 `docs/architecture.md` 로 **이관**(삭제 아님). ★부수 실측 2건: ①`**/Cargo.toml` 은
  `publish-artifact.yml` 발행 경로라 **주석만 고쳐도 릴리스+dispatch 발화** ②**MCP 등록 0개**.
- 2026-08-17: KB 실행 경로 정정 착지 (PR #46 `7514d552`) — `~/Documents/dev/wie` → `~/work/otterpebble/wie`.
- 2026-08-08: 공급망 대장 **A-2·A-3 착지** (PR #55, `wie-supply-chain-cargo-updates-a2-a3`) —
  `event-listener 5.4.1→5.4.2` · `spin 0.12.0→0.12.2`. `Cargo.lock` 만 변경 · 코드 변경 0.
- 2026-08-03: **완주의 정의를 «PR 을 열어 둔 상태»로 개정** (PR #53, `wie-agents-md-gate2-contradiction-fix`) —
  헌장이 워커에게 머지를 «지시» 하던 문장 5곳(`AGENTS.md` 3 + `CONTRIBUTING.md` 2) 개정 + `CLAUDE.md` «완주 규율» 신설.
- 2026-08-03: **공급망 추적 대장 등재** (PR #52, `wie-rustsec-advisory-sweep-batch2`) —
  `docs/project-kb/02_status.md` 에 권고·공급망 3건 + upstream 이슈 9건 표. `#1292` 종결로 실질 개발 후보 3→2건.
- 2026-08-01: `.direnv/` gitignore (PR #51).
- 2026-07-31: **PR #45 잔재 착지** (PR #50, `wie-pr45-orphan-close-and-remnant-land-r2`) —
  `.dev.vars` read-deny + session discipline. ★**PR #45 본체는 미머지 종결**(2026-07-31 CLOSED);
  살릴 값만 골라 옮긴 것이 #50 이다. 이 경위가 아래 «해소된 항목» ③의 근거다.
- 2026-07-22: featurephone 소비 계약 드리프트 가드 (PR #36·#39) · main 브랜치 보호 코드 준비 (PR #43) ·
  security audit schedule red 정정 (PR #42). 상세는 `REPORT.md` 및
  `docs/worklog/2026-07-22--featurephone-engine-contract-selftest.json`.
  ★human-step 잔여: 운영자가 branch-protection ruleset 1회 적용 —
  `~/orchestrator/reports/wie-main-branch-protection.done.md` C항.

## 다음

**① upstream #1260 후속** — 영웅서기5 LGT `Unknown SVC id 1409`. **1차 착수분은 `## 진행중` 으로 이동**
(`wie-lgt-svc-1409-unknown-and-state-stale-next`: 표 등재 + 미지원 예외까지). 남은 것은
**misc index 9 의 정체 규명**이고, 그것은 **이 repo 안의 근거로는 못 푼다** — 실기/다른 구현체의
LGT misc 테이블 또는 게임 바이너리의 호출부 디스어셈이 있어야 한다. 새 근거가 생기기 전에는 재발권하지 마라.

**② upstream #1122 발권 판단** (대장 B · 실질 개발 후보) — 컴투스 삼국지 촉, 스테이지 5 부근 정지.
2026-08-15 실측: upstream **OPEN** 유지(2026-05-10 이후 정체), upstream 오너도 «에뮬레이터 버그로
추정 · 디버깅 난해»로만 답했다. ★**착수 전에 재현 가능성부터 판정하라** — 현 회귀 게이트는
**부팅+렌더까지만** 판정하므로(`scripts/smoke_gate_baseline.tsv` 의 `ktf/컴삼촉.zip PASS` 도 그 의미다)
스테이지 5 심도는 **기존 자동화로 도달하지 못한다.** ①보다 난도가 한 단계 높다.

**③ (선택) 화면을 실제로 그리는 초소형 픽스처** — 여전히 유효. `scripts/contract-roundtrip.mjs` 는
`nonBlackPixels()` 를 세지만 `test_data/helloworld_*.zip` 이 아무것도 그리지 않아 픽셀 수가
**info-only** 로만 보고된다 — 같은 파일 상단 주석 «the fixtures never draw, so canvas blit is reported
as info» 와 `check()` 호출의 «fixture draws nothing — pixels are info only» 문구가 그 한계를 명시한다.
그리는 픽스처를 넣으면 왕복 검사가 blit 회귀까지 커버한다.

### 2026-08-15 재판정에서 «해소»로 내린 항목 (다시 발권하지 마라)
- ~~① PR #54 «게이트② 검수 상신 대기»~~ — **해소.** `feat/wie-agents-md-declarative-restructure` 는
  **MERGED 2026-08-16T14:13:11Z** · 머지커밋 `41721671f1b906a76e9298d08f2737c10cba7416`.
- ~~② PR #46 «검수 상신»~~ — **해소.** `kb-path-update-2026-07-25` 는
  **MERGED 2026-08-17T04:14:07Z** · 머지커밋 `7514d5527263ba539a8f30e109bd8a2dbdbed0a8`.
  ⇒ 2026-08-19 실측 `gh pr list -R Jun025/wie --state open` = `[]`(**열린 PR 0**).
  ★**`-R Jun025/wie` 를 반드시 붙여라** — 이 워킹트리에서 `gh pr view 54` 는 fork 부모(`dlunch/wie`)로
  해석돼 2023년 dependabot PR 을 돌려준다. repo 를 못박지 않은 조회는 **다른 repo 를 잰 값**이다.
- ~~게이트② approve 후 `-merge` 티켓~~ — **해소.** 당시 대기하던 PR #52·#53·#55 전건 머지 완료.
  `feat/wie-featurephone-engine-contract-selftest` 는 브랜치조차 없다 — 내용은 PR #36·#39(07-22)로 착지했다.
- ~~로컬 main 분기(`0f13ab87`, ahead 1 / behind 12)~~ — **해소·무효.** 2026-08-15 실측 **0 / 0**.
  ★그 커밋을 «PR 로 착지시킬» 필요는 **없다**: PR #45 로 올라갔다가 미머지 종결됐고, 세 헝크 중
  `.claude/settings.json` deny 와 `.gitignore` 2줄은 **이미 origin/main 에 있다**(PR #50·#51 경유).
  남은 `CLAUDE.md` «자율운영 SOP» 블록은 **되살리면 안 된다** — 그 4개 조항은 전부
  `AGENTS.md` §Session Discipline 에 있고, 첫 조항 「확인 없이 이어서 완료한다」는 현행
  `CLAUDE.md` §착수 규율(**티켓 없는 착수 금지**)과 **정면으로 충돌**한다.
