# STATE

> 실측 기준일 **2026-08-19**(`wie-state-landed-pr56-residue-and-misc-unk9-error-lock`).
> 열린 PR **0건** · `## 진행중` 의 착지분(PR #56·#57)은 `## 완료` 로 이관 완료.
> 직전 갱신은 2026-08-08 이었고 그 사이 `## 다음` 4항 중 2항이 조용히 해소돼 레인이 25시간 굶었다.
> ★**`## 다음` 이 낡으면 이 레인은 굶는다** — 착지할 때마다 갱신하라(`AGENTS.md` §Session Discipline).
> ★★**그리고 «새로 쓰는 항목»도 diff 를 열고 써라** — 초판의 ①②는 PR 제목·개설일만 보고 작성돼
> **둘 다 사실과 어긋났다**(게이트② 반려). 낡음을 지운 자리에 새 부정확을 심으면 병은 그대로다.
> ★★★**`## 진행중` 과 「열린 PR 0건」 실측이 어긋나면 «절이 낡은 것»이다 — 착지 즉시 `## 완료` 로 옮겨라.**
> 2026-08-19 게이트②가 잡은 형태가 그것이다: 이미 머지된 PR 이 `## 진행중` 에 남아 바로 아래의
> 「열린 PR 0건」 줄과 서로를 반증했고, 앞줄만 읽은 총괄은 **죽은 `-merge` 를 발권할 수 있었다.**

## 진행중
- `wie-state-landed-pr56-residue-and-misc-unk9-error-lock` — 본 절의 착지 잔재 정리 + `misc_unk9` 에러 문면
  테스트 잠금. 브랜치 `docs/wie-state-pr56-residue-misc-unk9-lock`. **게이트② 리뷰 대기.**
- ★**그 밖에 진행중 0 · 열린 PR 0건** — 2026-08-19 실측 `gh pr list -R Jun025/wie --state open` = `[]`.

## 완료 (최근)
- 2026-08-18: **LGT SVC 0x581 등재 착지** (PR #57 `bccf11f1`, `wie-lgt-svc-1409-unknown-and-state-stale-next`) —
  `WIPICSvcId::MiscUnk9 = 0x581` + `WieError::Unimplemented`(모듈·인덱스·인자 4개). 영웅서기5 LGT 는 여전히
  이 지점에서 멈추지만 로그가 「알 수 없는 SVC」에서 「misc index 9 미구현」으로 바뀌었다.
- 2026-08-15: **`STATE.md` 재판정 착지** (PR #56 `225392e8`, `wie-lane-restart-upstream-carryover-and-main-divergence`) —
  `## 진행중` 5행을 `## 완료` 로 이관 + `## 다음` 4항 재판정. 이 레인의 25시간 공백을 끝낸 회차다.
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

**① upstream #1260 후속 — ★«재발권 금지» 축이다(해제 조건 있음)**. 영웅서기5 LGT `Unknown SVC id 1409`.
1차 착수분(표 등재 + 미지원 예외)은 **PR #57 `bccf11f1` 로 착지**했다. 남은 것은 **misc index 9 의 정체 규명**이고,
★**이 repo 안의 근거로는 닫혀 있다** — 2026-08-19 게이트② 검수가 총괄 질의에 «동의한다 — 재발권 금지에 찬성»으로
답하며 근거 3개를 실측으로 댔다:
- repo 안에 **LGT misc 표가 없다**. KTF 쪽에는 `WIPICMiscMethodId` enum **자체가 없고**,
  유일한 misc 자료 `wie_ktf/.../method_table.rs` 의 `get_misc_method_table()` 은 **index 4 에서 끝난다**.
- ★**유추 경로도 닫혀 있다** — 두 구현의 인덱스 오프셋이 **모듈마다 다르다**(graphics **+1** / kernel **−3**).
  ⇒ KTF 인덱스 산술로 LGT index 9 를 옮겨 적으면 «근거 없이 맞아 보이는 문장»이 된다.
★**해제 조건 — 아래 셋 중 하나가 «새로» 생기기 전에는 이 축을 다시 열지 마라**:
⑴실기 덤프 ⑵다른 구현체의 LGT misc 표 ⑶게임 바이너리 호출부 디스어셈.
★조건 없이 발권하면 다음 사람이 **같은 벽에 다시 부딪힌다** — 그 왕복을 막으려고 여기 적어 둔다.

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

### «해소»로 내린 항목 — 다시 발권하지 마라 (판정일은 항목마다 표기)
> ★절 제목에 날짜를 박지 않는다. 회차마다 항목이 붙는데 제목 날짜는 안 따라와서
> 「언제 판정됐나」가 어긋났다(2026-08-19 게이트② 지적). 판정일은 **각 항목의 접두**로 읽어라.
- **[판정 2026-08-19]** ~~① PR #54 «게이트② 검수 상신 대기»~~ — **해소.** `feat/wie-agents-md-declarative-restructure` 는
  **MERGED 2026-08-16T14:13:11Z** · 머지커밋 `41721671f1b906a76e9298d08f2737c10cba7416`.
- **[판정 2026-08-19]** ~~② PR #46 «검수 상신»~~ — **해소.** `kb-path-update-2026-07-25` 는
  **MERGED 2026-08-17T04:14:07Z** · 머지커밋 `7514d5527263ba539a8f30e109bd8a2dbdbed0a8`.
  ⇒ 2026-08-19 실측 `gh pr list -R Jun025/wie --state open` = `[]`(**열린 PR 0**).
  ★**`-R Jun025/wie` 를 반드시 붙여라** — 이 워킹트리에서 `gh pr view 54` 는 fork 부모(`dlunch/wie`)로
  해석돼 2023년 dependabot PR 을 돌려준다. repo 를 못박지 않은 조회는 **다른 repo 를 잰 값**이다.
- **[판정 2026-08-15]** ~~게이트② approve 후 `-merge` 티켓~~ — **해소.** 당시 대기하던 PR #52·#53·#55 전건 머지 완료.
  `feat/wie-featurephone-engine-contract-selftest` 는 브랜치조차 없다 — 내용은 PR #36·#39(07-22)로 착지했다.
- **[판정 2026-08-15]** ~~로컬 main 분기(`0f13ab87`, ahead 1 / behind 12)~~ — **해소·무효.** 2026-08-15 실측 **0 / 0**.
  ★그 커밋을 «PR 로 착지시킬» 필요는 **없다**: PR #45 로 올라갔다가 미머지 종결됐고, 세 헝크 중
  `.claude/settings.json` deny 와 `.gitignore` 2줄은 **이미 origin/main 에 있다**(PR #50·#51 경유).
  남은 `CLAUDE.md` «자율운영 SOP» 블록은 **되살리면 안 된다** — 그 4개 조항은 전부
  `AGENTS.md` §Session Discipline 에 있고, 첫 조항 「확인 없이 이어서 완료한다」는 현행
  `CLAUDE.md` §착수 규율(**티켓 없는 착수 금지**)과 **정면으로 충돌**한다.
