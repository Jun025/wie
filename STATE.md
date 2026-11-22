# STATE

> 실측 기준일 **2026-11-22**(`wie-state-md-baseline-refresh-2026-11-22`).
> ★**이 «기준일 + 날짜» 꼴은 기계가 읽는다** — `~/orchestrator-live/bin/state-baseline-lag-scan` 이 이 파일 머리 15줄 안의
> 첫 `기준일 … YYYY-MM-DD` 를 `origin/main` 마지막 착지일과 견준다(14일 넘게 벌어지면 원장에 `STATE_BASELINE_LAG`).
> 낱말을 바꾸면(예: «갱신») 판정에서 조용히 빠진다 — 낱말은 두고 날짜만 올려라.
> 2026-08-27 판본의 머리 블록과 위치 표, 옛 `## 다음` 본문(①~④ 전부 끝남)은 **`docs/state-next-history.md`** 에 있다
> (2026-11-06 이관 · `docs/report/0550`).
> ★**`## 다음` 이 낡으면 이 레인은 굶는다** — 착지할 때마다 갱신하라.
> ★★**새로 쓰는 항목도 diff·실측을 열고 써라** — 낡은 줄을 지운 자리에 새 부정확을 심으면 병은 그대로다.
> ★`## 진행중` 과 `## 완료` 는 고정 안내다(정본 = 열린 PR 목록 · `docs/report/`).

## fork 의 현재 위치 (2026-11-22 실측 · ★조회 시각과 함께 읽어라)

| 축 | 값 |
|---|---|
| `git merge-base origin/main upstream/main` | `44fbf265`(2026-09-16 조각 D · PR #161 착지 이후 **불변** · 67일째) |
| behind / ahead | **244** / 3,034 (`origin/main` `64ee6998` ↔ `upstream/main` `10620f82` 2026-11-21) · 11-06 에는 173 / 2,970 |
| upstream 쪽 델타 | 243커밋(병합 제외) · 384파일 **+42,087/−11,755** · `git merge-tree` 충돌 파일 **136**(11-06 에는 93) |
| RustJava | 우리 = crates.io `jvm*` `^0.4.1`(lock 0.4.1 · 조각 S1 · `docs/report/0572`) ↔ upstream = `^0.5.1`(11-22 #1630·#1631 · 11-06 예행 때는 `^0.4.1`) |
| compat (`docs/player-data/compat.json`) | 429행 · playable **426** · not-yet 2 · limited 1 · `progress=stuck` **90** · enginePin `4ac38566` |
| GitHub Actions | 2026-11-04T05:40Z 뒤 run 0 → 11-19T14:59Z 에 첫 run(GitHub 쪽 정지 · `docs/report/0553`). 그 뒤 push·PR run 은 돈다. ★`schedule` 이벤트 run 은 11-04 이후 **0건**(아래 §다음 «상시») |

## 진행중

> ★★**이 절은 «회차마다 고쳐 쓰지 않는다» — 정본은 열린 PR 목록이다**(2026-09-08 이관 · 티켓
> `wie-state-in-progress-top-insert-conflicts-every-landing` · 채택 제안
> `2026-09-07-state-completed-split-decision#p0`).
>
> ```sh
> gh pr list -R Jun025/wie --state open        # ← 「지금 무엇이 떠 있나」의 정본
> ```
>
> ★**왜 «고정 안내»인가**: 회차마다 이 절 **맨 위**에 자기 항목을 끼워 넣으면 형제 회차가 전부
> ★**«같은 삽입 지점»** 을 다투므로 착지 1회가 **열린 PR 전건의 충돌**로 증폭된다.
> `REPORT.md` 가 2026-09-07 에 같은 병으로 고정 안내가 됐고, **남은 절반이 정확히 이 절이었다.**
>
> ★★**«맨 아래 추가»(append)로는 안 고쳐진다 — 실측이다.** git 의 3-way 병합은 두 삽입 지점 사이에
> ★**단 1줄**만 있으면 깨끗이 붙고, **같은 지점이면 위든 아래든 충돌**한다(0줄 → CONFLICT · 1줄 → clean).
> 「맨 위」가 문제가 아니라 ★**«지점이 하나»** 인 것이 문제다. ⇒ 지점을 **없앴다**.
>
> ★**정렬키를 PR 번호로 두는 안도 이 저장소에선 안 통한다** — 형제 회차의 PR 번호가 **연속**이라
> (#120~#133 실측) 삽입 지점이 다시 서로 인접한다.
>
> ★**대가(숨기지 않는다)**: ⑴**오프라인에서 못 읽는다** — `gh` 없이는 「지금 뜬 것」을 알 수 없다.
> 그 대신 **착지한 것**은 아래 「완료」와 `docs/report/` 에 그대로 남으므로 **기록은 잃지 않는다.**
> ⑵**진행 중 회차의 «서술»이 이 파일에 남지 않는다** — 그 서술의 정본은 PR 본문과
> `~/orchestrator/tasks/<티켓>.md` 이고, 착지 시점에 「완료」로 들어온다.
>
> ★**되돌리는 법**: 이 인용 블록을 지우고 회차 항목을 다시 손으로 적으면 된다(코드·검사기 0).

## 완료 (최근)

> ★★**이 절은 «회차마다 고쳐 쓰지 않는다» — 착지 기록의 정본은 `docs/report/` 의 회차별 파일이다**
> (2026-09-18 이관 · 티켓 `wie-remove-state-md-completed-insertion-point` · 선행 =
> `wie-state-md-insertion-point-conflicts-every-open-pr`(PR #199)가 백필로 **기각 사유를 없앴다**).
>
> ```sh
> grep -H '^## \[' docs/report/*.md | sort -r     # ← 「무엇이 착지했나」의 정본(연번 내림차순)
> ```
>
> ★**왜 «고정 안내»인가 — §진행중 과 «같은 병»이다.** 회차마다 이 절 맨 위에 자기 항목을 끼워 넣으면
> 형제 회차가 전부 ★**«같은 삽입 지점»** 을 다투므로 **착지 1회가 열린 PR 전건의 충돌로 증폭**된다.
> ★**«맨 아래 추가»로는 안 고쳐진다** — 위 §진행중 절의 실측 그대로 **지점이 «하나»인 것**이 문제다.
> ⇒ 지점을 **없앴다.** `docs/report/NNNN--…` 는 ★**회차마다 «새 파일»** 이라 공유 지점이 아예 없다.
>
> ★**언제 가능해졌나**: 「이 절에만 있는 내용이 소실된다」가 **유일한 기각 사유**였고
> (`AGENTS.md` §Landing paperwork 의 SUPERSEDED 배너가 그 결정의 측정 기록이다),
> 2026-09-18 재측에서 ★**149 항목 «전건»이 `docs/report/` 사본을 갖는다**(사본 없음 **0**).
> ※티켓 id 가 없는 옛 3줄도 각각 `0092`·`0094`·`0159` 가 **원문을 그대로 인용**해 두었다.
>
> ★**대가(숨기지 않는다)**: ⑴★**한 홉 멀어진다** — 이 파일만 열면 보이던 «최근 착지 요약»이 이제
> `docs/report/` 를 읽어야 보인다(위 한 줄이 그 인덱스다). 오프라인에서도 읽히지만 **한 번에 한 파일**이다.
> ⑵**정렬이 «날짜»가 아니라 «연번»** 이 된다 — 대체로 일치하지만 **보장이 아니다**(그래서 `-H` 로 경로를
> 붙여 연번으로 정렬한다 · `AGENTS.md` 가 그 이유를 적는다). ⑶★**이 회차 자신이 열린 PR 전건을
> 한 번 더 무효화했다** — 그 비용은 «지금 한 번»이고, **안 하면 «착지마다 계속»** 이다.
>
> ★**이 절을 읽는 기계는 0 이다**(2026-09-18 재측 · 추적 코드·설정 전수): `## 완료` 파서 **0** ·
> `STATE.md` 언급 **1**(`wie_midp/tests/create_image_missing_name_message.rs` 의 doc 주석 —
> 그 줄번호 인용을 이 회차가 `docs/report/0047--…` 로 **함께 고쳤다**).
>
> ★**되돌리는 법**: 이 인용 블록을 지우고 `docs/report/` 항목을 다시 손으로 옮겨 적으면 된다(코드·검사기 0).
> ★**되돌릴 때 알아야 할 것**: 되돌리는 순간 위 ⑶의 비용이 **착지마다** 다시 붙는다.

## 다음

> 형식: «무엇 · 왜 지금 · 근거 · size». 끝난 것은 아래 «완료로 접은 항목» 에 1줄씩만 둔다.
> 사료(옛 본문 · 측정 서술 전문) = `docs/state-next-history.md`.

다음은 아래 ⑴ 이다(발권 전 · 발권은 총괄 몫). ⑵ 는 ⑴ 과 독립이다.

**⑴ upstream 2차 동기화 — 조각 S1b(RustJava `^0.4.1` → upstream 이 쓰는 `^0.5.1`) · S2(executor·pacing)**
- **무엇**: `docs/report/0551` §7 의 조각 계획 그대로다. S1 의 첫 조각(`^0.1.1` → `^0.4.1` · 필드 owner 인자 · `patches/classfile` 삭제 ·
  zip 9)은 `docs/report/0572` 이 했다. 남은 S1b = upstream 이 11-22 에 올린 `^0.5.1`(#1630 · #1631)까지 — 0.4.2 가 더한
  Java 스택 깊이 상한(512 프레임 → `StackOverflowError`)이 여기 든다. S2 = `wie-backend` 의 `executor.rs`·
  `system.rs`·`task.rs` 를 upstream 의 sleep 마감·협동 폴링 의미로 옮기고 우리 pacing API 는 지킨다.
  둘 다 우리 쪽만 바꾸고 서로 독립이라 병렬로 열 수 있다. 병합 커밋(S3)은 둘이 착지한 뒤다.
- **왜 지금**: «한 번에 병합»은 no-go 로 판정됐고 조각 계획은 go 다(0551 §0). 기다린 값이 수로 나왔다 —
  11-06 → 11-22(16일) 사이 behind 173 → **244** · 충돌 파일 93 → **136**. merge-base 는 67일째 그대로다.
- **근거**: 0551 §7(공통 검증식 V0 + 조각별 검증식) · 열린 제안 `2026-11-22-upstream-sync2-s1-rustjava-pin-0-4-1#p0`(S1b)·`2026-11-06-upstream-sync2-dry-run-classify#p1`(S2) ·
  병합 전략 = `merge`(squash 금지 · `AGENTS.md` §For the `-merge` task only).
- **size**: S1b = M(upstream #1630 은 15파일 +43/−36) · S2 = M.
- ★**0551 의 93행 분류표는 11-06 트리의 것이다.** S3 을 열 때 표 전체를 그날의 `upstream/main` 에서 다시 잰다(0551 §1).

**⑵ 「오래 하기」 이른 종료(`short`) 14행 — 한 번 더 재서 같은 값이 나오는 행만 `compat.json` 에 반영**
- **무엇**: `docs/report/0561` §4 의 14행(전부 `playable → limited` 후보 · 네 축 `ok/ok/ok/short`)을 현행 핀에서 다시 재고,
  이번에도 요청 시간 전에 끝난 행만 내린다. 내리는 행에는 이용자 소식(`docs/player-updates/`)을 쓴다.
- **왜 지금**: 판정 규칙은 2026-11-22 에 착지했고(PR #594) 행은 일부러 안 고쳤다. 지금 `compat.json` 은 7~204초에 스스로 끝나는
  14판을 `playable` 로 알린다. 그중 `4ece6eeeaa04` 는 주간 퇴행 감시의 가드 7행 중 하나다.
- **근거**: 0561 §4 와 그 「반영할 때의 주의」(보관 기록의 이른 종료 20행 중 7행이 재측에서 600초를 채웠다 — 그래서 한 번 더 잰다).
  14 는 하한이다. 나머지 389행은 보관 기록에 기댄다(전수 재측은 약 22시간이라 이 조각에 넣지 않는다).
- **size**: M. 실행은 `bash scripts/census-drive.sh …`(phase 단위 `--long` 임대 · `--jobs` 기본).
- ★`compat.json` 의 행을 고친다. 같은 파일에 `runtimeAdjustments` 칸을 더한 PR #595 는 2026-11-22 에 착지했다(11-21T23:25Z) — 그 위에서 연다.

**총괄 발권 대기**(레인이 칠 수 있는 일이다 · 사람 결정이 아니다 · 티켓을 내는 것만 총괄 몫)
- **PR #589 에 CI 를 받는다**(0556 · 브라우저 틱 호스트 깨움 opt-in · 2026-11-11 개설 · S) — Actions 정지 중에 열려 check 가 **0건**이고
  required 미보고로 머지가 막혀 있다(11-22 조회 · `gh pr checks 589` = `no checks reported`). 새 커밋으로 head 를 바꿔야 run 이 생긴다
  (`gh run rerun` 으로는 안 온다).

**이미 발권돼 있다 — 다시 발권하지 마라**(2026-11-22 `~/orchestrator/tasks/` 조회 · 정본은 그 원장이고 이 줄은 금방 낡는다)
- `wie-rust-audit-main-red-since-1119-advisory-batch`(M) — `main` 의 `cargo audit` red · advisory 13건 처분.
- `wie-ktf-generic-hook-exact-equivalence-fix`(M).
- hold 2건: `wie-ktf-local-net-fold-into-fake-server`(M) · `wie-featurephone-content-fingerprint-for-user-supplied-files`(L).
  선행이던 PR #595 는 2026-11-22 에 착지했다(11-21T23:25Z). hold 를 푸는 것은 총괄이다.
- `wie-schedule-events-stopped-since-1104-reregister`(S) — 아래 «상시»의 schedule 미발화 조사.

**상시 · 정해진 날**
- **주간 퇴행 감시**(`scripts/regression-census.sh`) — launchd 가 매주 일요일 05:00 에 돌린다. 마지막 완주 = **2026-11-15**
  (측정 핀 `2e66f9d6` · 퇴행 의심 0). ★**2026-11-22 회차는 돌지 않았다** — 05:00 에 census 잠금을 다른 실행이 쥐고 있어
  launchd 래퍼가 건너뛰었다(`~/logs/wie-regression-census.err` 끝줄 `skipped census-lock`). 그래서 11-22 에 착지한 10건
  (엔진을 고친 0557·0559 포함)은 이 감시를 아직 한 번도 안 거쳤다. 정시로는 **2026-11-29(일) 05:00** 이 처음이다.
  그 전에 손으로 1회 돌릴지는 총괄 판단이다(1회 약 2.8시간 · long 풀).
- **CI 의 주 1회 축**(0553 · PR #586) — `rust.yml` 6다리 전부와 `coverage.yml` 은 schedule·수동 실행에서만 돈다(토 19:17Z · 19:47Z).
  ★**이 저장소의 `schedule` 이벤트는 2026-11-04T05:40Z 의 Security audit 를 끝으로 한 번도 발화하지 않았다.** Actions 가 다시 돈 뒤
  (첫 run 11-19T14:59Z)로도 **0건**이다(2026-11-21T23:29Z 조회 ·
  `gh api 'repos/Jun025/wie/actions/runs?event=schedule&created=>=2026-11-05' --jq .total_count`).
  - 매일 00:00Z 의 Security audit 는 11-20 · 11-21 두 번을 통째로 걸렀다. 이 워크플로는 예정보다 5시간 21분~6시간 1분 늦게 뜨곤 했는데
    (11-01~11-04 의 네 run) 그 폭을 한참 넘겼다 ⇒ «늦는 중»이 아니라 «안 뜬다»다.
  - 0553 뒤 첫 토요일 cron 셋도 조회 시각까지 발화하지 않았다: `rust.yml` 11-21 19:17Z(4시간 12분 경과) · `coverage.yml` 19:47Z
    (3시간 42분) · `doc-liveness` 20:35Z(2시간 54분 · 이 워크플로의 지연 전례는 10-10~10-31 네 run 에서 2시간 1분~3시간 12분).
  - ⇒ 주 1회 그물(6다리 · coverage)은 아직 한 번도 안 돌았고, 일일 감사(마지막 11-04)와 `doc-liveness`(마지막 10-31)도 서 있다.
    원인 조사·재등록 티켓 `wie-schedule-events-stopped-since-1104-reregister` 가 발권돼 있다(위 «이미 발권» 줄).
    그래도 0 이면 `AGENTS.md` «A red weekly run has an owner» 의 ⑶(8일 넘으면 손으로 1회 + 운영자에게 올림)이 걸린다.

**사람 몫**(워커가 정할 수 없다)
- **동작이 갈리는 6자리 — S3 전에 운영자가 고른다**(0551 §8): `HandsetProperty` `VOLUMELEVEL` · WIPI `Player.stop(null clip)` ·
  `Player.stop` 반환 · MIDP `Canvas` GAME_A..D · LGT SVC 4개 · 파일시스템 이름공간(aid ↔ pid — ★저장 키가 바뀐다).
  어느 쪽이든 합칠 수는 있다. 고르는 순간 게임이 보는 값이 바뀐다.
- **upstream `AGENTS.md` 의 Attribution 조항을 받을지**(0551 §7 S4).

**보류 · 재개 조건이 따로 있는 축**(발권하지 마라 — 조건이 «새로» 생기면 연다)
- **P4 / S5 — 엔진 고유분을 upstream PR 로**: 2026-09-13 dlunch/wie #1419 1건 착지 · 그 밖 미착수. 0551 의 분류에서 «발신» **29**
  파일이 나왔다. 재개 조건 = 운영자 확인(외부 저장소에 보낸다). IP 방침 선 안쪽만(공개 API 스텁 · 단위 시험 —
  `docs/lgt_abi.md`·`docs/reference/` 는 보내지 마라) · 보내기 전에 주석·시험의 sha 접두와 타이틀 언급을 지운다(0551 §7 S5).
- **진도 ⒟ `confirm-only-entry` 남은 14행** — 0552 가 원인을 전부 «넘는 키 미발견»으로 판정했다(엔진 벽 0 · 게임 설계 0).
  재개 조건 = 그 판을 넘는 키 경로(`--keys`)가 새로 나올 때. 그 회차의 열린 제안 1건(`2026-11-06-progression-stuck-confirm-only-15-remeasure#p0` · S)은
  14행을 넘기는 제안이 아니다 — 그중 한 판(`a0436eb1ddbb`)이 놀이 중에 내는 배열 범위 밖 예외의 자리를 재는 별개의 조사다.
  ⒟ 의 나머지 22행(`key-path-not-found` 20 · `play-reached` 2)은 `docs/report/0525`(10-22) 판정 그대로다 —
  11-06 대조에서 한 행도 안 움직였고 그 뒤 이 행들을 다룬 회차가 없다.
- **무거운 장면의 ARM 해석기 처리량**(L · 열린 제안 `2026-11-13-lgt-cadence-verdict-and-frames-painted#p0`) — 0559 가 싼 것 둘만 넣었다.
  upstream 의 ARM AOT(#1415)가 S3 으로 들어온다. 그 뒤에 재서 남는 몫만 여는 순서가 싸다(이 회차의 읽기다 · 측정한 것이 아니다).
- **60Hz 죽은 구간 14 → 15 ms**(0559 §5 · 이득 +4.7% 는 섰다) — 프레임 여백 0.5 ms 의 위험을 잴 실기기가 없어 넣지 않았다.
  재개 조건 = 기기 측정.
- **upstream #1260 후속(LGT misc index 9 · `Unknown SVC id 1409`)** — 재발권 금지. 해제 조건:
  ⑴실기 덤프 ⑵다른 구현체의 LGT misc 표 ⑶게임 바이너리 호출부 디스어셈 중 하나가 새로 생길 때.
  근거 3개 전문은 `docs/state-next-history.md`.
- **upstream #1122(스테이지 5 정지)** — 종결(`docs/report/0509`). 재개 조건: 우리 빌드에서 정지를 새로 보고받을 때.

### 완료로 접은 항목 (다시 발권하지 마라 · 옛 `## 다음` 전문은 `docs/state-next-history.md`)
- **2026-11-06 후보 ⑴ upstream 2차 동기화 «예행·분류»** — 완료(`docs/report/0551` · PR #584 · 2026-11-22 착지).
  충돌 93 = 폐기 11 · 재적용 53 · 발신 29 · 예행 트리 게이트 전부 red · «한 번에 병합» no-go · 조각 계획 S1~S5.
- **2026-11-06 후보 ⑵ 진도 ⒟ `confirm-only-entry` 15행 현행 핀 재측** — 완료(`docs/report/0552` · PR #585 · 2026-11-22 착지).
  1행 `ok` · 14행 `stuck` 유지(위 보류 축).
- **① P1 RustJava `[patch]` fork 이탈** — 완료 2026-09-04(`docs/report/0022`). 핀 계단은 조각 D 에서 crates.io 로 끝났다.
- **① P2 측정 게이트** — 완료 2026-09-03 · 재측 2026-09-16 (러너 이식 21줄 · 코퍼스는 이 머신 `game_lab/`에 있음 · `docs/report/0019`).
- **① P3 조각 A~E** — A `0115` · B PR #158 · C `0113` · D **PR #161 착지**(merge-base 이동 — 위 위치 표) ·
  E PR #162 착지 없이 닫음(`docs/report/0508`). ⒜/⒝ 배선 결정은 코퍼스 LGT 54건 차이 0 으로 ⒝ 유지(`docs/report/0222`).
- **③ upstream #1122 발권 판단** — 종결 2026-10-11(`docs/report/0509`).
- **④ 그리는 픽스처** — 해소 2026-08-27(`draw_j2me.jar` · Scenario C 실단언).
- **2026-08 «해소» 목록**(PR #54·#46·#52·#53·#55 · 로컬 main 분기) — 전건 해소 · 판정 기록은 사료 파일 끝.
