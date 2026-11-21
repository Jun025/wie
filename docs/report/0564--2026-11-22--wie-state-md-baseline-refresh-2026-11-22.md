## [2026-11-22] STATE.md 기준일·«다음» 을 2026-11-22 사실로 — 열린 PR #583(0550)을 받아 그 위에 잇는다 (wie-state-md-baseline-refresh-2026-11-22)

**무엇을**: `STATE.md` 의 머리 기준일을 2026-11-22 로 올리고, 「fork 의 현재 위치」 표와 `## 다음` 을 오늘 실측으로 다시 썼다.
`## 진행중`·`## 완료` 고정 안내는 그대로다. 코드 변경 0.
이 브랜치는 열린 PR #583(`wie-state-next-refresh-and-next-round-proposal` · 0550 · 게이트② approve · 미착지)의 head `d93561bf` 를
일반 병합으로 받았다(rebase·force-push 0). 그래서 diff 에 그 PR 의 4파일(`docs/state-next-history.md` · `docs/report/0550` ·
worklog 1건 · `STATE.md`)이 함께 보인다. 그 위에 이 회차가 더한 것은 `STATE.md` 재작성, 이 파일, 그리고 #583 검수의 비차단 지적 3건
(m1·m2·m3 — 사료 머리와 0550 의 «그대로» 문장에 예외 1행 명시 · worklog 의 머지 시각 `07:17:19Z`)이다.

**왜**: 원장 2026-11-22 07:37 `STATE_BASELINE_LAG wie 기준일=2026-08-27 착지=2026-11-22 lag=86d`. `origin/main` 의 `STATE.md` 는
2026-08-27 판본 그대로였다. 11-06 에 같은 일을 한 PR #583 은 approve 를 받았지만 그때 이 저장소의 Actions 가 멈춰 있어
(`docs/report/0553`) 착지하지 못했다. 그 사이 #583 이 «다음 후보»로 적은 2건이 둘 다 착지했다(0551 · 0552). 그 문안도 낡았다.

- **#583 을 따로 두지 않고 받은 까닭**: 둘 다 `STATE.md` 의 같은 절을 통째로 고친다. 따로 쓰면 어느 쪽이 먼저 착지하든 다른 쪽이
  충돌한다. 받으면 순서와 무관하다 — 이 PR 이 먼저 착지하면 #583 의 커밋이 전부 `main` 의 조상이 되고, #583 이 먼저 착지해도
  이 브랜치에는 이미 그 커밋이 있다. `check-docs-report-serial` 은 0550 을 «같은 파일»로 보아 충돌로 세지 않았다(rc=0).
- **#583 의 머리 줄을 되돌린 까닭**: #583 은 머리를 「갱신 **2026-11-06**」 으로 썼다. `state-baseline-lag-scan` 은 머리 15줄 안의
  `기준일 … YYYY-MM-DD` 만 읽는다. 그 낱말이 없으면 `NO_BASELINE` 으로 판정에서 빠진다. «기준일» 낱말을 되살리고 그 사실을 머리에 적었다.

**사용자 영향**: 없다(문서만).

### 실측 (2026-11-22 07:5x KST · `git fetch origin` · `git fetch upstream` 직후)

| 축 | 11-06 (#583 · 0551) | 11-22 |
|---|---|---|
| merge-base | `44fbf265` | `44fbf265`(불변) |
| behind / ahead | 173 / 2,970 | **244** / 3,034 (`64ee6998` ↔ `10620f82`) |
| upstream 델타(병합 제외) | 172커밋 · 309파일 +32,181/−9,779 | 243커밋 · 384파일 +42,087/−11,755 |
| `git merge-tree --write-tree --name-only` 충돌 파일 | 93 | **136** (rc=1) |
| RustJava(우리 ↔ upstream) | `^0.1.1` ↔ `^0.4.1` | `^0.1.1` ↔ `^0.4.2` |
| compat | 429 · playable 426 · stuck 91 | 429 · playable 426 · not-yet 2 · limited 1 · stuck **90** |

- 기준일 이후 `origin/main` first-parent 착지 **523건**(`git log origin/main --first-parent --since=2026-08-27 --oneline | wc -l`).
  이 중 11-06 이후는 10건(PR #584~#588 · #590~#594 · 전부 2026-11-21T18:41Z~22:45Z 머지)이고 회차 기록은 0551~0555 · 0557~0561 이다.
- 열린 PR 3건(이 PR 을 열기 전): #583(이 브랜치가 받음) · #589(0556 · check 0건 · `MERGEABLE` · `BLOCKED`) ·
  #595(0562 · required 5종 pass · `pr_audit` fail — `main` 의 audit red 와 같은 축으로 보이나 이 회차는 그 로그를 열지 않았다).
- Actions: `gh run list` 로 11-04T06Z 이후 run 을 event 별로 세면 `workflow_dispatch` 2(11-19T14:59Z) · `pull_request` 58 · `push` 38 ·
  **`schedule` 0**. `gh api 'repos/Jun025/wie/actions/runs?event=schedule&created=>=2026-11-05' --jq .total_count` → **0**(11-21T23Z 조회).
  매일 00:00Z 인 Security audit 가 11-20·11-21 에 안 떴다. 토요일 cron 셋(11-21 19:17Z · 19:47Z · 20:35Z)은 조회 시각에 run 이 없었는데
  `doc-liveness` 의 지난 두 run 이 cron 보다 2시간 6분 · 3시간 3분 늦게 생겼으므로 이것은 판정하지 않았다.
- 주간 퇴행 감시: `~/logs/wie-regression-census.err` 의 마지막 두 사건 = `2026-11-15.md · suspects 0 · candidates 0 · 10056s` 다음에
  `cc-wie-regression-census: skipped census-lock(pid=53116)`(파일 수정 시각 11-22 05:00). `evidence/wie-weekly-regression/` 에
  `2026-11-22.md` 는 없다. 07:5x 에 그 pid 는 살아 있지 않았고 census 프로세스도 없었다.
- LANE_IDLE 판독: `dispatcher` 의 `lane_idle_next_source` 술어를 새 `STATE.md` 에 그대로 적용했다. 배너 포인터 0 · 동결 0 ⇒ legacy.
  «다음» 절 84행 · `다음은` 지목 줄 1행(id·sha 0) · 절 안 id 형 토큰 중 `tasks/` 에 실재하는 것 4개(전부 «이미 발권» 줄의 미착지 티켓).

### `## 다음` 을 어떻게 고쳤나

`origin/main`(2026-08-27 판본) 대비 — 옛 항목은 ①(P1 · P2 · P3 · P4) · ② · ③ · ④ · «해소» 목록의 8개다.
- **완료로 접음 6**: P1 · P2 · P3(조각 A~E) · ③ · ④ · «해소» 목록. 전문은 `docs/state-next-history.md` 로 옮겨져 있다(#583 의 몫).
- **유지 3**(보류 축 · 재개 조건과 함께): P4(→ 0551 의 S5 와 묶음) · ②(#1260) · ③의 재개 조건(#1122).
- **추가**: 다음 후보 2(⑴ S1·S2 · ⑵ `short` 14행) · 사람 몫 3 · 보류 축 3(진도 14행 · ARM 처리량 · 60Hz 죽은 구간) ·
  상시 2(주간 감시 · CI 주 1회 축) · 이미 발권된 티켓 4의 한 줄 · 완료 2(#583 의 11-06 후보 둘).

#583 판본 대비로는: 후보 2건을 완료로 내리고 새 후보 2건으로 갈았다. 보류 3축은 유지했고 P4 문안만 고쳤다.

### 한계

- ⑵ 의 «#595 착지 뒤가 싸다»와 ARM 처리량 축의 «S3 뒤에 재라»는 이 회차의 읽기다. 잰 것이 아니다.
- «이미 발권돼 있다» 줄은 `~/orchestrator/tasks/` 의 11-22 스냅숏이다. 그 원장이 정본이고 이 줄은 낡는다.
- 0551 의 93행 분류가 136 으로 늘어난 지금 어떻게 갈리는지는 재지 않았다(S3 착수 때의 몫 — 0551 §1).
- 게임 파일명 유입(`node scripts/corpus-name-inflow.mjs`, 이 브랜치 대 `origin/main`): BOUNDED 0 · SUFFIX-ATTACHED 0.

### `-fix` (2026-11-22 · `wie-state-md-baseline-refresh-2026-11-22-fix` · 게이트② 반려 F1 · 핀 `9dbe7572`)

- **F1**: 첫 판의 `STATE.md` 24~34행에 편집 잔해가 남았다 — 머리 인용문의 앞머리가 잘린 줄이 H2 로 읽혔고, #583 판본의 11-06 위치 표가
  새 11-22 표 아래에 그대로 있었다(«현재 위치» 절 2개 · 값이 서로 달랐다). 그 12줄(빈 줄 포함)을 지웠다. 위 «위치 표를 다시 썼다»는
  이 커밋부터 사실이다. `grep -n '^## ' STATE.md` = 현재 위치 1 · 진행중 1 · 완료 1 · 다음 1.
- **m1**: PR #589 항목을 «사람 몫»에서 새 줄 «총괄 발권 대기»로 옮겼다(레인이 칠 수 있는 일이다).
- **m2**: 보류 축 «진도 ⒟ 남은 14행»의 열린 제안이 14행용이 아니라 그중 한 판의 예외 조사임을 한 구절로 적었다.
- **m3**: «상시»의 schedule 문장을 다시 쟀다(2026-11-21T23:29Z). `schedule` run 은 여전히 0 이고 토요일 cron 셋도 조회 시각까지 안 떴다.
  ★위 실측의 «2시간 6분 · 3시간 3분 늦게»는 `doc-liveness` 의 지난 두 run 만 본 값이다. 네 run 으로는 2시간 1분~3시간 12분이고,
  일일 Security audit 는 5시간 21분~6시간 1분 늦게 뜨곤 했다(11-01~11-04). 그래서 판정의 근거는 «토요일 cron 이 4시간 12분 지났다»가
  아니라 «일일 감사가 이틀을 통째로 걸렀다»다. 토요일 셋은 `rust.yml` 기준 4시간 12분 경과로, 일일 감사의 지연 폭 안쪽이다.
- **범위 밖에서 고친 2줄**: 작업 중 PR #595 가 착지했다(11-21T23:25Z). «⑵ 는 #595 착지 뒤가 싸다»와 «hold 2건(선행 = #595 착지)»가
  그 순간 낡아서 사실로 고쳤고, «이미 발권» 줄에 schedule 조사 티켓 1건을 더했다. 위치 표의 `origin/main` `64ee6998` 과 ahead 3,034 는
  첫 판 측정 시각의 값 그대로 두었다(다시 재지 않았다).
- 코드 변경 0 · 병합 구조(`f1bb4718`) 그대로 · rebase·force-push 0.

<!-- corpus-name-inflow v1 subjects=5 tree=3453b36d33b4b6a7 B=0/0 P=0/0 S=0/0 -->
