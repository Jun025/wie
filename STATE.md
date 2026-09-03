# STATE

> 실측 기준일 **2026-08-27**(`wie-upstream-realign-verdict`).
> ★★**이 회차로 이 repo 의 «위치»가 바뀌었다 — 위 절들보다 이 줄을 먼저 읽어라.**
> `origin/main`(`250d7e4c`)은 `upstream/main`(`dlunch/wie` `73938944`)보다 **1,067커밋 뒤**이고
> **192커밋 앞**이며, 공통조상 `fa641a8a` 는 **2026-06-10 이후 한 번도 움직인 적이 없다**
> (우리 192커밋에 upstream 동기화 시도 **0건** — 실측).
> 판정 정본 = **`docs/upstream-realign-verdict.md`**. 채택 갈래 = ★**⒟ — fork 를 배포·제품
> 오버레이로 남기고 엔진은 upstream 을 쓴다.** 집행은 이 회차가 하지 않는다(P1~P4 초안은 `REPORT.md`).
> ★**직전 갱신 시점의 `## 진행중` 1건(PR #65)은 착지**했다(squash `250d7e4c`) — `## 완료` 로 옮겼다.
> ★**`## 다음` 이 낡으면 이 레인은 굶는다** — 착지할 때마다 갱신하라(`AGENTS.md` §Session Discipline).
> ★★**그리고 «새로 쓰는 항목»도 diff 를 열고 써라** — 초판의 ①②는 PR 제목·개설일만 보고 작성돼
> **둘 다 사실과 어긋났다**(게이트② 반려). 낡음을 지운 자리에 새 부정확을 심으면 병은 그대로다.
> ★★★**`## 진행중` 과 「열린 PR 0건」 실측이 어긋나면 «절이 낡은 것»이다 — 착지 즉시 `## 완료` 로 옮겨라.**
> 2026-08-19 게이트②가 잡은 형태가 그것이다: 이미 머지된 PR 이 `## 진행중` 에 남아 바로 아래의
> 「열린 PR 0건」 줄과 서로를 반증했고, 앞줄만 읽은 총괄은 **죽은 `-merge` 를 발권할 수 있었다.**

## fork 의 현재 위치 (2026-08-27 실측 · 정본 `docs/upstream-realign-verdict.md`)

| 축 | 값 |
|---|---|
| 공통조상 | `fa641a8a` **2026-06-10** (이후 **불변**) |
| behind / ahead | **1,067** / **192** |
| 우리 고유 변경 | 220파일 **+25,690/−164** (문서·CI 제외 시 148파일 +17,890/−127) |
| ① 중복(upstream 이 자기 구현으로 착지) | Rust **3,437줄 = 우리 Rust 의 64.3%** |
| ② 엔진 고유·가치 | Rust 1,134줄 (21.2%) |
| ③ 로컬 스캐폴딩(웹 제품·문서·CI) | **20,291줄 = 전체의 79.0%** |
| LGT Java import `0x64` 구현 | ours **8** ↔ upstream **31** · ★**ours 에만 있는 것 0** |
| `Jun025/RustJava` `[patch]` fork | ★**upstream RustJava 의 진부분집합**(우리에만 있는 `.rs` **0**) |

★**핵심**: fork 가 private 이라서가 아니라 **전진하지 않아서** 문제다(`Jun025/wie` 는 이미 public).
★**그리고 진짜 사슬은 `Jun025/RustJava` `[patch]` 표다** — 재정렬과 **독립적으로 지금 끊을 수 있다**(P1).

## 진행중
- 2026-09-04: **키 입력 «도달»을 행동으로 단언 — 왕복 검사 Scenario D 신설**
  (`wie-featurephone-keypress-reaches-guest-behavioral-axis` · 채택 제안
  `2026-07-22--featurephone-engine-contract-selftest#p0`) — 종전에 키 축을 보던 것은 둘뿐이었다:
  ⒜왕복 검사 Scenario A 의 「어휘 **20종**을 눌러도 **예외가 안 났다**」 ⒝`check-engine-contract.mjs`
  §4 의 **소스 핀**(`wie_web/src/lib.rs` 의 `fn parse_key` 본문에서 `"UP" => KeyCode::UP` **쌍**을 읽는다).
  ★**둘 다 «게스트에 도달했는가»는 보지 않는다.** ⇒ 픽스처의 `keyPressed()` 가 **받은 MIDP 코드만큼 넓은 막대**를
  그리게 해서 캔버스가 **어느 코드가 도달했는지**를 말하게 했다(대표 키 **3종** — 소프트/숫자/방향).
  ★**개악 대조**: `key_down` 이 이벤트를 **버리게** 하면 소스 핀은 **48 pass / 0 위반**, Scenario A 도
  **✓ 20 codes** — ★**둘 다 못 잡는다.** Scenario D 만 **3건 red**. 오탐 0(기존 26건 전건 통과 · **29/29**).
  ★**제품 코드 변경 0**(`wie_web/src/lib.rs` 무접촉 — 개악은 되돌렸다) · CI 워크플로 변경 0.
- 2026-09-01: **워크로그 «회차 의무» 기각 + 2026-07-22 백필** (`wie-worklog-mandate-decision-and-2026-07-22-backfill`) —
  ★**결정: 의무화하지 «않는다».** 규약 착지(`92c25276`) 후 착지한 **3회차 전건**이
  워크로그를 썼고(**3/3 = 100%** · `-fix` 승계까지 세는 커밋 축으로는 4/4), 직전 19회차는 **0/19** 였다 ⇒ 무조건 의무를 얹어 얻을 커버리지가 남아 있지
  않고, 이미 있는 **조건부** 문장이 면제까지 포함한다. ★그래서 `AGENTS.md`·검사기 **무접촉**이다.
  백필은 `limits` 3줄 중 **1줄만** 승격했다(1줄은 `1853d49e` 가 이미 닫았고 1줄은 Constraint 9 의 영구 경계).
  ★**분모가 3회차뿐**이라 기각은 영구 판정이 아니다 — 재측정 시점·임계는 이 회차의 유일한 후속 제안이다.
  ★★**[2026-09-04 · `wie-worklog-mandate-rejection-needs-a-reopen-threshold`] 그 제안이 채택돼
  «되돌릴 조건»이 박혔다 — 정본은 `AGENTS.md` §Landing paperwork **한 곳**이다.**
  ★여기에 주기·임계·방법을 옮겨 적지 마라(두 곳에 있으면 갈린다). ★위 「3회차 전건」 근거는 **사료로 그대로 둔다** —
  결정 자체는 바뀌지 않았고, 붙은 것은 **언제 다시 재고 얼마면 뒤집는가**뿐이다.

## 완료 (최근)
- 2026-08-27: **upstream 재정렬 판정 착지** (PR #66 `0cb309b4`, `wie-upstream-realign-verdict`
  + 반려 승계 `-fix`) — `docs/upstream-realign-verdict.md` 신설(3분류 표 + 갈래 판정 + LGT 코드
  대조 + RustJava fork 실측). ★**제품 코드 변경 0 · upstream 발신 0.** 채택 갈래 ⒟, 집행은 후속
  P1~P4(위 `## 다음` ①). ★**이 줄은 2026-09-01 회차가 옮겼다** — 착지 후에도 `## 진행중` 에 남아
  있어 바로 위 머리글의 「착지 즉시 `## 완료` 로 옮겨라」와 어긋났다(그 시점 열린 PR **0건** 실측).
- 2026-08-27: **외부 apt 장애가 착지를 막던 경로 차단 + `fail-fast: false` 착지**
  (PR #65 `250d7e4c`, `wie-rust-ci-beta-leg-blocks-gate-on-external-outage`) — `rust.yml` 1파일.
  ubuntu 스텝이 `bash -e` 아래에서 `apt update` 의 100(우리가 쓰지 않는 `packages.microsoft.com` 403)에
  죽어 `apt install` 에 도달조차 못 하던 것을 「update 는 best-effort · install 은 fatal」로 갈랐다.
  ★게이트를 무르게 하지 않았다 — `libasound2-dev` 를 못 받으면 여전히 red 다. 제품 코드 변경 0.
- 2026-08-27: **화면을 그리는 초소형 픽스처 + 픽셀 계수 실단언 승격 착지** (PR #64 `1853d49e`,
  `wie-drawing-fixture-makes-pixel-count-a-real-assertion`) — `scripts/make-draw-fixture.mjs`
  (JDK 없이 class 파일 바이트를 직접 찍는다 · jar 는 **커밋하지 않고** 메모리에서 서빙 —
  `*.jar` 는 git-ignore + 유출 감사가 tracked jar 를 거부한다) + `contract-roundtrip.mjs`
  **Scenario C**(J2ME · `nonBlackPixels() > 0` 실단언). 26/26 green · 개악 시 25/26 red.
  ★**게이트③ 이 1회차는 `blocked`** 였다 — 외부 apt 장애로 `ci-presence` rc=1(그 red 가 위 `## 진행중`
  회차를 낳았다). 2회차가 `rerun` **1회**로 green(rc=0)을 받아 착지시켰고, 머지 후 main 4런 전건 green ·
  Pages prod 배포 + D1 원격 마이그레이션 실집행 · self-verify `https://wie-web.pages.dev` **200 · 콘솔 0에러**.
- 2026-08-26: **회차 워크로그 `.json` + `proposals` 규약 이식 착지** (PR #63 `92c25276`,
  `wie-worklog-json-proposals-convention`) — cockpit 「후속 작업 추천」이 wie 를 구조적으로 0건으로
  읽던 것을 풀었다(착수 실측 `/api/proposals` `derived.coverage`: wie `json:1 · md:0 · proposals:0`).
  `AGENTS.md` §Landing paperwork 에 워크로그 `.json` 의무 + 소비 키 표, `scripts/check-worklog-json.mjs`
  5축 잠금, `engine-contract.yml` 의 항상 도는 잡에 편입. ★**소급 변환 0** · 코드·의존성 변경 0.
- 2026-08-21: **문서의 맨손 원격 변이 wrangler 명령 + `web.yml` 판본 핀 단일화 착지**
  (PR #62 `4cd0f43e`, `wie-cf-setup-bare-d1-create-and-wrangler-pin-consolidate`) —
  ①`docs/CLOUDFLARE_SETUP.md` 의 `wrangler d1 create` 를 계정 핀 접두로 + `docs/COMPLIANCE.md` 의
  원격 변이 7건도 블록마다 핀 ②`web.yml` 판본 리터럴 **5곳 → job env `WRANGLER_VERSION` 1곳**
  ③`CLOUDFLARE_ACCOUNT_ID` 가 비면 배포 전에 시끄럽게 죽는 가드 스텝. 코드·의존성·판본 변경 0.
- 2026-08-20: **Pages 가 거부하는 `account_id` — 핀을 env 로 이전 착지** (PR #61 `eef08184`,
  `wie-wrangler-pages-account-id-breaks-prod-deploy-fix`) — `wrangler.toml` 의 `account_id` 제거 +
  `package.json` 의 `deploy`·`db:migrate:remote` 에 `CLOUDFLARE_ACCOUNT_ID` 가드 + `web.yml` 에
  PR 에서도 도는 Pages 설정 검증 스텝. ★**「고쳤다고 본다」가 「고쳤다」로 확정됐다** — 머지 후
  `web.yml` on main run **32375925645 success**(2026-08-20T13:43Z), 직전 red run 32364381443 대비.
- 2026-08-20: **wrangler `account_id` 고정 착지** (PR #59, `wie-wrangler-account-id-pin`) —
  `wrangler.toml` 최상단에 otterpebble 계정을 박아 «어느 계정으로 배포되는지»를 파일이
  선언하게 했다. 게이트②는 08-20 에 approve 였으나 아래 beta clippy red 가 게이트③ `ci-presence` 를
  막고 있었고(1회차 `-merge` 는 그 rc=1 로 **정확히 거부**했다), 그 red 착지 후 base 를 당겨 착지했다.
  ★★**그리고 이 착지가 prod Pages 배포를 red 로 만들었다** — Pages 는 `account_id` 키를 문법으로
  거부한다(`web.yml` run 32364381443). ⇒ 위 「파일이 선언한다」는 **더 이상 참이 아니다**(env 핀으로
  이전 · 위 `## 진행중`). 사료로 남긴다.
- 2026-08-20: **beta clippy `double_must_use` repo 전역 CI red 해소 착지** (PR #60 `7a49aff0`,
  `wie-rust-ci-beta-clippy-double-must-use-red`) — `async-trait` 0.1.89 → 0.1.92(`Cargo.lock`) +
  `wie_ktf` `find_java_method` 국소 `allow` 1줄. 코드 동작 변경 0.
- 2026-08-18: **`STATE.md` 착지 잔재 정리 + `misc_unk9` 에러 문면 잠금 착지** (PR #58 `6f9dbae7`,
  `wie-state-landed-pr56-residue-and-misc-unk9-error-lock`).
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

**★① upstream 재정렬 집행 — 이 회차의 판정을 잇는 축**(정본 `docs/upstream-realign-verdict.md`).
채택 갈래 **⒟**. ★★**[2026-08-27 정정] 초판의 「순서는 P1 → P2 → P3/P4 이고 P1 은 나머지와 독립이라
먼저 간다」는 «순서» 부분이 자명하지 않다** — P1 의 «목적»(fork 이탈)은 여전히 독립이지만 **«크기»가
갈래에 따라 `S`↔`L` 로 달라져** 순서가 그에 딸린다(아래 P1·P2). ★**이 레인은 순서를 결정하지 않는다.**
- **P1**(★**갈래 둘 · risk med** · 선행없음) `Jun025/RustJava` **fork 이탈**. ★★**[2026-08-27 정정 ·
  게이트② 반려] 초판의 「S·low·13줄 제거」는 «참이 아니다»** — `Cargo.toml:49-53` base 의존에 `rev` 가
  없어 `[patch]` 를 지우면 의존이 `dlunch/RustJava` HEAD 로 **47커밋 전진**하고, 그 면에 우리 호출부
  ★**218곳**을 깨뜨리는 공개 API 변경 **3종**이 실재한다(⑴`invoke_virtual` 인자 추가 **209곳/37파일** ·
  ⑵`current_class_loader` **비공개화 6곳**(공개 대체 API **없음**) · ⑶`attach_thread` arity+async **3곳**).
  ⇒ ★**⒜ 파열 포함 = `L`** / ★**⒝ base 를 `bee850f` 로 `rev` 핀 = `S` 이나 «12커밋 하드닝 상실»이
  대가이고 ★그 손실은 4게이트 green 인 채로 «조용히» 일어난다**(Java 레벨이라 코퍼스에서만 드러난다).
  ★**선택은 총괄 몫 — 정본 `docs/upstream-realign-verdict.md` §4-B·§6-P1 에 둘을 나란히 적었다.**
  ★부기: Constraint 8 반증 자체는 **참**이고 게이트②가 재현했다. 단 「진부분집합」은 ★**«기능·파일 축»
  한정**이다 — `jvm` 공개 API 축에서는 우리 fork 가 `pub fn` 4종을 **추가**했다(wie 호출 0곳).
- **P2**(M·측정전용·★**선행 배치 재검토 대상**) ★**⒟ 전체의 go/no-go.**
  ★P1 이 ⒜(=`L`)로 확정되면 **가장 싼 판정 측정이 가장 비싼 선행 뒤에 서게 된다** — 총괄이 순서를
  재검토하라(게이트② §6 지적). ★이 레인은 결정하지 않는다.
  `upstream/main` 체크아웃에 `scripts/smoke_gate.sh` 를 걸어 **코퍼스가 있는 머신에서**
  `ktf 190 / lgt 52 / skt 50` 재측정 → 차이표 1장. ★`game_lab/` 은 **이 머신에 없다**(실측) —
  게임 바이트는 Constraint 9 로 repo 에 들어올 수 없으므로 **구조적으로** 여기선 못 잰다.
- **P3**(L·med·선행 P2) `wie_web` → `wie_featurephone` **개명**(upstream 이 같은 이름을 자기 용도로 쓴다)
  후 upstream 을 base 로 ③ 오버레이 재적용 + `compile_model.rs` **122줄 이식** + ★**엔트리포인트 규약 정합**
  (upstream `LgtEmulator` 는 `application.jar` 를 찾고 우리는 `00000000.jar` 를 넘긴다 — ★«부수 발견»이
  아니라 **작업목록 리터럴 항목**이다. 조용히 깨지는 것은 목록에 없으면 잊힌다). ★여러 회차로 쪼개라.
- **P4**(M·low·P3 와 병행) ② 를 upstream PR 로. ★**IP 방침 선 안쪽만**(#1239 2026-06-29
  「공개 문서 기반으로만 구현 · 펌웨어 리버스 계획 없음」) — `wipi_java` 공개 API 스텁 10종 +
  `canvas.rs` 단위테스트 9개는 **보낼 수 있고**, `docs/lgt_abi.md`·`docs/reference/` 는 **보내지 마라**.
★★**P3 의 DoD 에 리터럴로 박아라**: 머지 후 `git merge-base origin/main upstream/main` 이
`fa641a8a` 가 **아니어야** 한다. 그대로면 그 회차는 **실패**다 — 게이트③ `--squash` 가 upstream 계보를
평평하게 만들어 다음 회차를 **또 1,067커밋 뒤**에서 시작시킨다(동시 발권
`rustjava-upstream-sync-squash-defeats-convergence` 가 RustJava 에서 실측한 바로 그 형태).

**② upstream #1260 후속 — ★«재발권 금지» 축이다(해제 조건 있음)**. 영웅서기5 LGT `Unknown SVC id 1409`.
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

**③ upstream #1122 발권 판단** (대장 B · 실질 개발 후보) — 컴투스 삼국지 촉, 스테이지 5 부근 정지.
2026-08-15 실측: upstream **OPEN** 유지(2026-05-10 이후 정체), upstream 오너도 «에뮬레이터 버그로
추정 · 디버깅 난해»로만 답했다. ★**착수 전에 재현 가능성부터 판정하라** — 현 회귀 게이트는
**부팅+렌더까지만** 판정하므로(`scripts/smoke_gate_baseline.tsv` 의 `ktf/컴삼촉.zip PASS` 도 그 의미다)
스테이지 5 심도는 **기존 자동화로 도달하지 못한다.** ①보다 난도가 한 단계 높다.

**④ ~~(선택) 화면을 실제로 그리는 초소형 픽스처~~** — **해소**(2026-08-27,
`wie-drawing-fixture-makes-pixel-count-a-real-assertion` · 아래 `## 진행중`).
`scripts/make-draw-fixture.mjs` 가 사각형 하나를 칠하는 J2ME MIDlet jar 를 만들고,
왕복 검사 **Scenario C** 가 그 픽스처에 대해 `nonBlackPixels() > 0` 을 **실단언**한다
(실측 **1024 px / 2 frames**). ⇒ 왕복 검사가 **blit 회귀까지** 커버한다 — 코어가 프레임을
합성하지 못하거나 `WebScreen::paint` 가 캔버스에 닿지 못하면 CI 가 red 다.
★**남는 것 3가지(사실만)**: ⑴`helloworld_*.zip` 은 **그대로**이고 그쪽 픽셀 수는 여전히
**info-only** 다(전역 승격이 아니라 픽스처별 승격이다) ⑵새 픽스처는 **J2ME 경로**를 지난다 —
KTF·LGT 의 그리기 경로는 여전히 왕복 검사의 단언 밖이다 ⑶픽스처가 **exit 하지 않는다**
(Scenario C 는 첫 페인트에서 멈춘다) — 그리기 + 정상 종료를 한 픽스처로 함께 보진 않는다.

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
