# REPORT

## [2026-08-21] 문서의 맨손 원격 변이 wrangler 명령 + `web.yml` 판본 핀 5곳 단일화 (wie-cf-setup-bare-d1-create-and-wrangler-pin-consolidate)
- **무엇을**: ①`docs/CLOUDFLARE_SETUP.md` 의 **`npx wrangler d1 create wie-db`** 를 `CLOUDFLARE_ACCOUNT_ID=…` 접두 형태로 바꾸고 0-1 을 가리키는 사유 3줄 추가(0번 표의 「`wrangler d1 create`」 안내도 「3-1 (계정 고정 필수)」로 교체) ②★**전수 grep 이 같은 형태를 `docs/COMPLIANCE.md` 에서 7건 더 찾았다** — `d1 execute --remote` 로 `UPDATE` 6건 + `r2 object delete` 1건. 3개 `sh` 블록 **전부의 첫 줄에 계정 핀**을 넣고 「Before running any command here」 절 신설 ③`web.yml` 의 wrangler 판본 리터럴 **5곳 → job env `WRANGLER_VERSION` 1곳** ④`web.yml` 에 계정 id 부재 시 **시끄럽게 죽는** 가드 스텝 1개. **3파일 · 코드·의존성·판본 변경 0**.
- **왜 (축1)**: `create` 는 원격에 리소스를 **만드는** 명령이다. 핀이 없으면 로컬 OAuth 로그인이 속한 **엉뚱한 계정에 `wie-db` 가 조용히 생기고 실패하지 않는다** — 직전 회차가 env 핀으로 옮기며 문서의 d1 «조회» 2곳은 가드된 npm 스크립트로 돌렸으나 이 «생성» 한 줄은 그 밖이었다. ★**「2곳만 봤다」가 이 결함을 낳은 형태라, 이번에는 `*.md` 전수를 원격 변이 동사(`create`·`delete`·`execute --remote`·`deploy`·`publish`·`secret put`)로 재grep 했고 COMPLIANCE.md 7건이 그렇게 나왔다.** 그쪽은 존재하지 않는 DB/버킷을 때려 대개 시끄럽게 죽으므로 위험도는 낮지만, 운영자가 **블록이 아니라 한 줄만 복사**하는 것이 실제 사고 경로라 블록마다 핀을 넣었다.
- **왜 (축2)**: 판본 `4.104.0` 이 설정 **검증기**(`npx` 2곳)와 실제 **배포기**(`wrangler-action` 2곳) 그리고 프로젝트 생성(`npx` 1곳)에 흩어져 있었다(티켓은 3곳으로 봤으나 실측 **5곳**). 갱신 때 한 곳만 올리면 **검증이 통과시킨 설정을 배포기가 거부**하고, 그것이 정확히 2026-08-20 사고의 모양이다. ⇒ 값은 **그대로 두고** 자리만 하나로 접었다. `env` 컨텍스트는 `steps.*.with` 와 `run:` 셸 양쪽에서 읽히므로 `wranglerVersion: ${{ env.WRANGLER_VERSION }}` 과 `npx --yes "wrangler@$WRANGLER_VERSION"` 이 각각 실제로 먹는다.
- **왜 (축3)**: 배포·마이그레이션 스텝의 `if:` 가 `HAS_CF_TOKEN` 만 봐서 **`CLOUDFLARE_ACCOUNT_ID` 가 비어도 게이트가 열렸다**. ★**「건너뛴다」가 아니라 「죽는다」를 골랐다** — 건너뛰면 **아무것도 배포하지 않은 green run** 이 남고, 그것은 이 리니지가 이미 한 번 당한 «조용한 실패»다. 토큰이 없는 fork PR 은 종전대로 그냥 skip 된다(가드가 `HAS_CF_TOKEN == 'true'` 를 함께 요구한다).
- **사용자 영향**: 없음. 에뮬레이터·배포 대상·판본 전부 무변경이고, 바뀐 것은 **문서의 문면**과 **워크플로가 판본을 한 곳에서 읽는다**는 것뿐이다.
- **★남는 구멍(숨기지 않는다)**: 축1 은 **문서 규율일 뿐 기계가 아니다** — 운영자가 핀 줄을 지우고 복사하면 그대로 통과한다. 기계로 닫으려면 wrangler 래퍼나 `--profile` 이 필요하고 그것은 직전 회차가 이미 「별건」으로 남긴 자리다. 축3 의 가드는 **secret 이 «비었는지»만** 본다 — **틀린 계정 id** 는 여전히 wrangler 의 인증 오류에 맡긴다(그쪽은 시끄럽게 죽는 것이 실측됐다).

## [2026-08-20] Pages 가 거부하는 `account_id` 로 prod 배포가 red — 핀을 env 로 이전 (wie-wrangler-pages-account-id-breaks-prod-deploy-fix)
- **무엇을**: ①`wrangler.toml` 의 `account_id` 한 줄 **제거**(사유 주석으로 대체) ②`package.json` 의 원격을 건드리는 두 스크립트(`deploy`·`db:migrate:remote`)를 `: ${CLOUDFLARE_ACCOUNT_ID:?…}` 가드로 감싸 **env 가 비면 wrangler 를 실행조차 하지 않게** 함 ③`web.yml` 에 **PR 에서도 도는** Pages 설정 검증 스텝 1개 추가(배포 없음) ④`docs/CLOUDFLARE_SETUP.md` 에 「0-1. 로컬 wrangler 계정 고정」 신설. **4파일 · 코드·의존성 변경 0**.
- **왜**: 직전 회차(PR #59)가 박은 `account_id` 를 **Pages 가 문법으로 거부**한다 — `Configuration file for Pages projects does not support "account_id"`. 그 착지 직후 `web.yml` run **32364381443** 이 배포 스텝에서 failure 였고, 앞 4회는 전건 success 였다. ★**`wrangler d1` 은 같은 키를 받아들여서**(같은 run 의 D1 마이그레이션 스텝은 성공) d1 로 시험하면 이 고장이 보이지 않는다 — 이번 회차가 d1 경로를 건드리지 않은 이유다.
- **★CI 가 못 잡은 이유(구조적)**: `web.yml` 의 배포 스텝은 `if: push && main` 게이트라 **PR 에서 한 번도 실행되지 않는다**. ⇒ 이 계열 결함은 항상 «PR green → main 착지 → red» 로만 드러난다. 그래서 추가한 검증 스텝은 **인증도 업로드도 하지 않는 자리**를 노린다: `wrangler pages deploy` 는 **설정 파일 검증을 인증·디렉터리 읽기보다 먼저** 하므로, 존재하지 않는 디렉터리를 가리키고 토큰을 주지 않으면 **검증기까지만 도달**한다. 실측 양방향 — 개악(`account_id` 재삽입) 시 **red**, 현 트리에서 **green**(인증 오류로 죽고 «validation for Pages» 는 나오지 않는다).
- **오계정 방지는 어디로 갔나**: 원 티켓의 목적(로컬 wrangler 가 **다른 계정**에 조용히 작업하는 것 방지)은 버리지 않고 **env 로 이전**했다. 실측 2건 — ⑴핀 없이 `wrangler pages deploy` 를 태우면 **에러 없이 로컬 로그인 계정을 그대로 사용**한다(에러 메시지가 사용 중인 계정을 그 계정으로 지목했다) ⑵`CLOUDFLARE_ACCOUNT_ID` 를 **틀린 값**으로 주면 `Authentication error [code: 10000]` 로 **시끄럽게** 죽고 로그인 계정으로 **되돌아가지 않는다**. 여기에 npm 가드가 «비어 있음»까지 막는다(unset → rc≠0, wrangler 미실행).
- **★남는 구멍(숨기지 않는다)**: 파일 핀과 달리 env 핀은 **맨손 `npx wrangler …` 를 구속하지 못한다**. 가드가 서는 것은 npm 스크립트 경로뿐이다. 파일 핀으로 되돌리는 길은 Pages 가 막혀 있으므로, 더 조이려면 `--profile`/래퍼가 필요하고 그것은 별건이다.
- **사용자 영향**: 없음(에뮬레이터 동작 무변경). 사이트도 내려간 적이 없다 — Pages 는 직전 성공 배포를 계속 서빙했고 이번 착지 diff 에 `migrations/**` 0파일이라 데이터 영향도 0. 되살아나는 것은 **신규 배포 반영**이다.
- **★판정의 한계**: 배포 스텝은 PR 에서 돌지 않으므로, 착지 전 이 회차는 「고쳤다」가 아니라 **「고쳤다고 본다」**다. 확정은 머지 후 `web.yml` on main 이 green 인지 실측해야 한다.

## [2026-08-20] wrangler `account_id` 를 otterpebble 계정으로 고정 (wie-wrangler-account-id-pin)
- **무엇을**: `wrangler.toml` 최상단에 `account_id = "17024dfe5a8ff38798c35942d116026b"` 를 사유 주석과 함께 박았다. **1파일 +6/-0** — 코드·워크플로·의존성 무변경.
- **왜**: 이 맥에는 Cloudflare 계정이 둘(otterpebble·dodu) 있고, `wrangler` 는 계정이 명시되지 않으면 **자격증명이 가리키는 아무 계정**으로 붙는다. `wie-db`(D1)·R2 버킷은 otterpebble 계정 소유이므로 잘못된 계정으로 붙으면 조용히 «빈 프로젝트에 배포»가 된다. 게이트② 검수가 `wrangler d1 info wie-db` 를 이 자격증명으로 돌려 **핀한 계정이 실제로 `wie-db` 를 소유함**을 양성 대조로 확인했다.
- **★권고 — 핀이 env 를 덮는다**: `wrangler.toml` 의 `account_id` 는 `CLOUDFLARE_ACCOUNT_ID` 환경변수보다 **우선한다**. ⇒ 이후 CI 에서 «다른 계정으로 배포»하려 해도 **`CLOUDFLARE_ACCOUNT_ID` 시크릿 교체만으로는 계정이 바뀌지 않는다** — 이 파일의 핀을 함께 고쳐야 한다(고정의 목적 자체가 그것이므로 의도된 동작이다).
- **사용자 영향**: 없음(에뮬레이터 동작·배포 대상 무변경). 바뀐 것은 «어느 계정인지»가 자격증명이 아니라 **파일에 적혀 있다**는 것뿐이다.

## [2026-08-20] beta clippy `double_must_use` 로 인한 repo 전역 CI red 해소 (wie-rust-ci-beta-clippy-double-must-use-red)
- **무엇을**: ①`async-trait` **0.1.89 → 0.1.92**(`Cargo.lock` 만 — 워크스페이스 요구사항은 이미 `^0.1`). 0.1.92 의 `expand.rs` 는 트레이트 메서드에 `#[must_use]` 를 **더 이상 붙이지 않는다**(0.1.89 `expand.rs:69` 의 `method.attrs.push(parse_quote!(#[must_use]))` 가 삭제됨). ②`wie_ktf/src/runtime/java/interface.rs` 의 `find_java_method` 에 `#[allow(clippy::double_must_use)]` 1줄 + 사유 주석 3줄. **코드 동작 변경 0** — 어트리뷰트와 lockfile 뿐이다.
- **왜**: `rust_ci (macos-latest, beta)` 의 `cargo clippy --all -- -D warnings` 가 **`wie_backend` lib 에서 15건**의 `double_must_use` 로 죽었고, fail-fast 가 나머지 5잡을 cancelled 로 끊었다. ★**이 red 는 특정 PR 의 결함이 아니라 repo 전역**이라 다음 wie PR 이 전부 같은 red 를 문다(PR #59 가 approve 인데도 게이트③ `ci-presence` 에 막힌 형태). 근인은 **우리 코드의 `#[must_use]` 가 아니다** — 15건 전부 `async_trait::async_trait` 매크로 확장에서 나왔고, clippy 가 `1.98.0-beta.1`(2026-07-06)→**`1.99.0-beta.1`(2026-08-17)** 로 넘어오면서 이 패턴을 새로 잡기 시작했다(로컬 구 beta 로는 **재현되지 않았다** — 툴체인을 올려서 재현시킨 뒤 고쳤다).
- **왜 `allow` 가 1건 남았나**: `async-trait` 을 올리면 15건이 0건이 되지만 **`wie_ktf` 의 `async_recursion` 1건**이 같은 이유로 남는다. `async-recursion` 은 **1.1.1 이 최신**이라 올릴 곳이 없고, 문제의 `#[must_use]` 는 우리가 쓴 것이 아니라 매크로가 찍은 것이다. ⇒ 호출부 1곳(`find_java_method` 유일 사용처)에 국소 `allow` 를 달고 사유를 주석으로 남겼다. **crate 전역·워크스페이스 전역 억제는 쓰지 않았다.**
- **사용자 영향**: 없음(에뮬레이션 동작 무변경). 바뀐 것은 CI 가 다시 green 이 되어 **wie 레인의 PR 이 머지 가능해졌다**는 것뿐이다.
- **부수 실측**: ⒜`cargo audit` 취약점 **0건**(정보성 경고 2건 = `ttf-parser` unmaintained / `arrayref` yanked, 둘 다 기존분·비게이팅) — 새로 들어온 전이 의존성은 `syn 3.0.3` 하나다. ⒝`gh repo set-default Jun025/wie` 를 repo 로컬에 설정했다. **이 repo 는 `dlunch/wie` 의 포크**라 그 전까지 맨 `gh pr view N` 이 upstream 을 조회했다(실측: #59 조회가 2023년 dependabot PR 로 갔다). ⒞`Cargo.lock` 과 `**/*.rs` 는 둘 다 `publish-artifact.yml` 의 `on.push.paths` 라 **이 PR 이 main 에 들어가면 아티팩트 발행 + GitHub Release + otterpebble dispatch 가 발화한다**(사건 대장 「`paths-filter` reads paths, not content」 참조). WASM export surface 변경 0 이므로 `docs/contracts/featurephone-engine-contract.json` 은 손대지 않았다.

## [2026-08-19] LGT SVC 0x581 — 정체 미확정이라 «미지원»으로 끊었다 + 낡은 `## 다음` ①② 해소 (wie-lgt-svc-1409-unknown-and-state-stale-next)
- **무엇을**: ①`wie_lgt/src/runtime/svc_ids.rs` 의 `WIPICSvcId` 표에 **`0x581`(=1409)** 을 `MiscUnk9` 로 등재하고, `wipi_c.rs` 의 디스패치를 **`WieError::Unimplemented`(모듈·인덱스·인자 4개를 로그에 실는다)** 로 연결했다. **값을 지어내 반환하지 않는다.** ②`STATE.md` `## 다음` ①②(PR #54·#46)를 «다시 발권하지 마라» 절로 옮겼다.
- **왜**: 영웅서기5 LGT(upstream `dlunch/wie#1260`)가 `CletWrapper.startApp` 에서 `Unknown LGT WIPIC SVC id 1409` 로 즉사한다. **`0x581` 이 misc 모듈이라는 것까지는 실측으로 확정된다** — LGT 는 WIPIC 모듈마다 100 의 배수를 기저로 쓰고 `BackLight = 0x578 = 1400+0` 이 misc 를 고정하므로 `0x581` 은 **misc index 9** 다. ★**그러나 index 9 «가 무슨 함수인지»는 확정되지 않는다**: 이 repo 의 유일한 misc 참조표(KTF `get_misc_method_table`)가 **index 4 에서 끝나고**, LGT 표는 **다른 스펙 개정판**이다(graphics 가 index 14 부터 KTF 대비 **+1** — LGT `CopyArea 0xd7`=+15 ↔ KTF 14). ⇒ 이름을 붙이면 **근거 없이 맞아 보이는 문장**이 된다. 그래서 티켓 Contract 2 의 «등재 + 미지원 예외» 갈래로 잘랐다.
- **사용자 영향**: 영웅서기5 LGT 는 **여전히 이 지점에서 진행하지 못한다.** 바뀐 것은 로그다 — 「알 수 없는 SVC」가 「misc index 9 미구현, 인자 4개는 이러함」이 되고, 그것이 다음 회차의 입력이 된다. 다른 타이틀 영향 0(`scripts/smoke_gate_baseline.tsv` 기준선 변화 0).
- **후속**: misc index 9 의 정체는 **이 repo 안의 근거로는 못 푼다** — 실기/다른 구현체의 LGT misc 테이블이나 게임 바이너리 호출부 디스어셈이 있어야 한다. 새 근거 없이 재발권하지 마라.

## [2026-08-15] wie 레인 재가동 — 낡은 `## 다음` 이 25시간 공백의 근인이었다 (wie-lane-restart-upstream-carryover-and-main-divergence)
- **무엇을**: `STATE.md` 를 2026-08-15 실측으로 전면 갱신했다. ①`## 진행중` 5행이 **전건 착지 완료분**(PR #52·#53·#55, 그리고 브랜치조차 없는 `feat/wie-featurephone-engine-contract-selftest`)이었으므로 `## 완료` 로 이관 ②`## 다음` 4항을 **유효/해소/무효**로 재판정하고 해소분은 «다시 발권하지 마라» 절로 분리해 명시 보존 ③적체 PR **#54·#46** 을 `## 다음` ①②로 신규 등재 — 이 둘은 종전 `STATE.md` **어디에도 없었다** ④upstream #1260·#1122 를 실측 재확인하고 착수 브리프로 승격. **로컬 `main` 은 fast-forward 재동기만 했다**(ahead 1 / behind 12 → **0 / 0**). 코드·워크플로·lockfile 무변경.
- **왜**: `cc-wie` pane 이 2026-08-14 21:02 기동 후 **25시간 동안 지시를 한 번도 받지 못했다**(큐 0 · running 0). 할 일이 없어서가 아니라 **`## 다음` 이 08-08 자 사실에 멈춰 있어서** 아무도 발권하지 않았다. 그 4항 중 2항은 이미 조용히 해소돼 있었고, 정작 살아 있는 축(적체 PR 2건)은 문서에 **적혀 있지도 않았다** — 즉 문서는 낡은 만큼 «없는 일»을 가리키고 «있는 일»을 감췄다.
- **판정 — 로컬 main 분기(`0f13ab87`)는 «착지 필요» 가 아니라 «이미 처분됨»**: 실측상 그 커밋은 `main@{11}: reset: moving to origin/main` 으로 **버려졌고** 어떤 ref 에도 없다. 다만 그 내용은 브랜치 `chore/claude-autonomy-hardening` → **PR #45 로 올라갔다가 2026-07-31 미머지 종결**됐고, 살릴 값만 골라 **PR #50** 이 착지시켰다. 세 헝크 대조 결과 `.claude/settings.json` deny 블록은 origin/main 에 **있고**(차이는 장식용 `$schema` 1줄뿐), `.gitignore` 2줄(`.direnv/`·`.claude/settings.local.json`)도 **둘 다 있다**(PR #51 등). ★남은 `CLAUDE.md` «자율운영 SOP» 블록은 **되살리면 안 된다** — 4개 조항이 전부 `AGENTS.md` §Session Discipline 에 이미 있고, 첫 조항 「시작 시 … **확인 없이 이어서 완료한다**」는 현행 `CLAUDE.md` §착수 규율(**티켓 없는 착수 금지**)과 **정면 충돌**한다. 즉 PR 로 착지시키는 것이 옳은 조치가 아니었다.
- **판정 — upstream #1260 은 유효하고 착수 가능**: OPEN 유지. `wie_lgt/src/runtime/svc_ids.rs` 의 `WIPICSvcId` 변환표에 **`0x581`(=1409) 부재**를 직접 확인했다(인접 등재 최대치는 `0x578 BackLight`) — 추정이 아니라 **미구현 확정**이다. 대상 게임(영웅서기5)이 `MClass: Clet` 이라 이 repo 가 confirmed 로 지원하는 서브셋에 속한다.
- **판정 — upstream #1122 는 유효하되 «재현»이 먼저**: OPEN 유지이나 2026-05-10 이후 정체이고 upstream 오너도 «에뮬레이터 버그 추정 · 디버깅 난해»로만 답했다. ★현 회귀 게이트는 **부팅+렌더까지만** 판정하므로 `scripts/smoke_gate_baseline.tsv` 의 `ktf/컴삼촉.zip PASS` 는 **스테이지 5 도달을 뜻하지 않는다** — 기존 자동화로는 증상 지점에 닿지 못한다. 착수 전 재현 경로 확보가 선행 조건이다.
- **사용자 영향**: 없음(문서 전용). 다만 이 레인이 다음에 다시 굶지 않는다 — `## 다음` 이 **오늘 실행 가능한 5개 축**을 가리키고, 그중 2개는 이미 CI green 인 채 검수만 기다리는 PR 이다.
- ★**정정 (2026-08-16 · 게이트② 반려 승계 `-fix`)**: 초판의 `## 다음` **①②가 둘 다 틀렸다.** 옛 4항은 diff 로 검증했는데 **새로 등재한 두 항목은 PR 제목·개설일만 보고 썼다** — 즉 이 항목이 진단한 병(«문서가 낡아 없는 일을 가리키고 있는 일을 감췄다»)을 새 문장을 쓰면서 그대로 재현했다. ⒜**#46**: 「경로를 `~/dev/wie` 로 고치는 PR 이라 의심스럽다」→ **정반대다.** `gh pr diff 46` 전문은 **1파일 1줄**로 `~/Documents/dev/wie` → **`~/work/otterpebble/wie`**(=실제 경로)로 **옳게** 고친다. 낡은 것은 **PR 제목**뿐이고 `origin/main` 이 지금도 틀린 경로를 담고 있다. ⒝**#54**: 「#54 개설 뒤 #53·#49 가 착지해 `AGENTS.md` 충돌」→ **틀렸다.** 둘 다 `merge-base 78f40a6f` 의 **조상**이고(`--is-ancestor` 양쪽 YES), base 이후 착지분은 **PR #55 하나**다. `merge-tree` 실측상 충돌 파일은 **`STATE.md` 단 1건**이며 `AGENTS.md` 는 충돌하지 않는다. ⒞★그리고 **이 PR 이 그 충돌을 키운다** — #56 head 기준 `merge-tree` 는 충돌을 **`STATE.md`+`REPORT.md` 2건**으로 보고한다(초판에 이 부작용 언급 0). ⇒ **#54 재작업은 #56 착지 «후»에.** ★옛 4항 재판정과 「해소」 2항은 검수가 자력 재현해 **참으로 확인**됐다 — 그대로 유지한다.
- **후속 추천**: ①PR #54 재작업 티켓 — ★충돌 파일은 `AGENTS.md` 가 아니라 **`STATE.md` 하나**이고 원인은 **PR #55** 다(#53·#49 는 `merge-base 78f40a6f` 의 **조상**이라 원인일 수 없다). ★**이 PR(#56)이 착지하면 충돌이 `STATE.md`+`REPORT.md` 2건으로 커지므로 #54 재작업은 #56 착지 «후»에** ②PR #46 은 **검수 상신만 하면 된다** — diff 전문 확인 결과 **1파일 1줄**로 `~/Documents/dev/wie` → **`~/work/otterpebble/wie`**(=실제 경로)로 고치는 **옳은 PR** 이고, 낡은 것은 **PR 제목**뿐이다 ③#1260 구현 티켓 발권 ④#1122 는 «구현» 이 아니라 «재현 경로 조사» 티켓으로 먼저 발권.

## [2026-08-05] 에이전트 지침 선언형 재구조화 — 잠금 테이블 + 지도 이관 (wie-agents-md-declarative-restructure)
- **무엇을**: `AGENTS.md` 를 **Goal / Constraints / Definition of Done / 사건 대장** 골격으로 재배치했다.
  ①Hard Requirements 12항 산문 → **12행 잠금 테이블**(각 항 = 한 줄 + «무엇이 잡는가» 포인터)
  ②아키텍처 지도 4종(레이어 다이어그램·crate roles 표·non-Rust surfaces 표, ~44줄)을
  `docs/architecture.md` 로 **이관**(삭제 아님) + 한 줄 crate 목록과 포인터만 잔류
  ③ponytail 절의 do-not-cut 재열거 제거 → 「Constraints 표가 그 목록이다」
  ④`CLAUDE.md` 축소금지 목록도 포인터 1줄로 대체(같은 목록이 **3벌**이었다)
  ⑤Hard Req 1·2 의 «왜» 를 `rust.yml`·`coverage.yml` **헤더 주석으로 이관**.
  **실측: 16,888 → 13,099 바이트(−22.4%)**, 상시 로드 합계(AGENTS+CLAUDE) 19,209 → 15,360(−20.0%).
  **코드·빌드설정·CI 동작 무변경** — 워크플로는 주석만 늘었고 파싱된 YAML 이 main 과 동일함을 확인했다.
- **왜**: 2026-08-05 전 프로젝트 지침 감사(`reports/audit-agent-instructions-2026-08-05.md` §3-1)가
  이 repo 의 Hard Req 12항 **대부분이 이미 CI 워크플로로 잠겨 있는데 산문이 한 번 더 적고 있다**고
  지목했다. ★지표도 바꿨다 — **줄 수가 아니라 바이트다**. 종전 189줄은 줄당 89바이트라 줄 수로는
  부하가 보이지 않았다.
- **★보존한 것(축약 금지)**: 「Completion is an open PR, not a merge」 전문과 §Git Workflow 전체,
  `CLAUDE.md` §완주 규율의 **리터럴 반복**(중복이지만 **의도된 것**), 게임바이트·시크릿·로컬전용
  3종 전문, `RUST_MIN_STACK` «not decorative», always-run 래퍼 교착 인과, 정확버전 핀 근거.
  뒤의 둘은 새 **사건 대장** 절로 옮겼다 — 서술형이 보호되는 자리다.
- **★부수 실측 ①**: `**/Cargo.toml` 이 `publish-artifact.yml` 의 `on.push.paths` 에 있다 —
  **paths-filter 는 내용이 아니라 경로만 본다**(otterpebble `free-tier.md` 기실측의 재확인). 즉
  **주석만 고친 Cargo.toml 도** 머지되면 릴리스 발행 + otterpebble `repository_dispatch` 를 발화시킨다.
  ⇒ Hard Req 8 의 «왜» 를 Cargo.toml 주석으로 내리려던 계획을 **철회**하고 `AGENTS.md` 에 남겼다.
  이 인과를 사건 대장에 등재했다.
- **★부수 실측 ②**: **MCP 등록 0개**(`claude mcp list` 무 · `~/.claude.json`·`~/.claude/settings.json`·
  `$CLAUDE_CONFIG_DIR` 2개 파일 전부 `mcpServers` 빈 값). `AGENTS.md` 는 「3종이 매 세션 사용 가능」이라
  적고 있었다 — **사실이 아니었다**. 문안을 실측대로 정정했고, 이로써 티켓 C2 의 「serena 로 대체
  가능한가」는 **불가**로 확정 → 지도는 삭제가 아니라 읽을 수 있는 문서로 이관했다.
- **★plan mode 시뮬레이션이 잡은 결함 1건**: 신·구 지침으로 각각 대표 작업 2건을 계획시켜 비교했더니,
  신 지침을 읽은 에이전트가 **순수 `web/` CSS 수정에서 Rust 게이트 4종을 누락**했다(«.rs 무변경이니
  web 명령만» 으로 읽음). 구 지침에는 «Pre-commit (MANDATORY)» 문장이 그 일을 하고 있었는데 게이트
  블록에 접으면서 «every commit» 이 사라진 것이다. ⇒ 「4종은 무엇을 고쳤든 **매 커밋 전**에 돌린다 ·
  web 명령은 **추가**이지 대체가 아니다」를 명시해 수정했다. 나머지 6개 점검 문항은 신·구 동등하거나
  신 지침이 우세했다(구 지침은 playwright MCP 가 쓸 수 있다고 **오판**).
- **사용자 영향**: 없음(문서 전용). 간접 영향은 매 세션 상시 로드가 약 3.8KB 줄어 같은 컨텍스트로
  실제 작업에 쓸 여지가 늘고, 잠금 포인터 덕에 「이 규칙을 누가 강제하나」를 파일을 뒤지지 않고 안다.
- **경계 / 미달**: 목표 바이트는 **≤12,288 이었고 결과는 13,099 — 811 초과**다. 파일의 약 6,500바이트가
  C3 무접촉 문안이거나 이번 산출물인 잠금 테이블이라, 남은 초과분을 없애려면 **보호 대상 서술을
  압축**해야 했다. 감사 자신의 결론(「선언화하면 가장 비싼 정보가 증발한다」)에 반하므로 멈추고 수치를
  보고한다. 시뮬레이션 수정분 214바이트도 이 초과에 포함돼 있다 — 정확성을 바이트에 양보하지 않았다.
- **후속 추천**: ①`CONTRIBUTING.md` 가 §Git Workflow 를 또 한 벌 갖고 있는지 재점검(#53 이 2곳을
  고쳤으나 이번 재구조화로 절 이름이 바뀌었다) ②감사 §4 훅(H2 「티켓 없는 착수 금지」)이 도입되면
  `CLAUDE.md` §착수 규율도 포인터로 축약 가능 — 현재는 강제력이 산문뿐이라 유지했다.

## [2026-08-02] 헌장이 머지를 «지시» 하던 문장 제거 — 게이트② 우회의 근인 (wie-agents-md-gate2-contradiction-fix)
- **무엇을**: 워커에게 머지·브랜치 삭제를 **지시**하던 문장 **5곳**을 개정했다 — `AGENTS.md` **3곳**(§Git Workflow «완주 = merge into main» · «Clean up merged branches (MANDATORY)» + `gh pr merge --delete-branch` 권장 · Hard Req 12 «squash-merge → delete the branch»)과 `CONTRIBUTING.md` **2곳**(§Git Workflow 17·18행). 더해 ①`AGENTS.md` 에 «For the `-merge` task only» 절을 신설해 브랜치 정리 절차를 **머지 티켓 소유로 이관**(지식은 보존, 수행 주체만 변경) ②`CLAUDE.md` 에 «완주 규율» 절 신설 ③`STATE.md` 의 머지 지시 문안 2곳 정정. **코드·워크플로 로직 무변경.**
- **왜**: 게이트② 사후 감사(`wie-rustsec-advisory-sweep-batch1.review.md`, severity **critical**)가 근인으로 확정한 것이 이 문장들이다. 헌장이 «완주 = `main` 에 머지» 라고 가르치니 **워커는 규율을 어긴 게 아니라 자기 repo 의 헌장을 따랐고**, 그 결과 검수 approve 전 머지가 **5건**(#48·#43·#42·#39·#38) 났다. 07-22 사후 회신들이 이미 «wie 4건 동일 실패형» 이라 적어 뒀는데 **헌장을 안 고쳐서 8일 만에 5건째**가 났다 — «알고 있었다» 가 «고쳤다» 를 대체하지 못한다.
- **핵심 개정**: 완주의 정의를 **«PR 을 열어 둔 상태»** 로 바꿨다. 머지와 브랜치 삭제는 검수 approve 후 **별도 `-merge` 티켓**의 몫이며, «CI green 은 필요조건일 뿐 승인이 아니다» 를 명시했다. 종전 문안은 «changes made 에서 멈추지 마라» 는 압력만 있고 상한이 없어 워커를 머지까지 밀어냈으므로, 하한(«push·PR 안 하면 미완주»)과 상한(«그 이상 가지 마라»)을 **양쪽 다** 적었다.
- **★감사가 놓친 2곳**: 감사는 `AGENTS.md` 3곳만 지목했으나, 전수 스캔에서 **`CONTRIBUTING.md` 에 같은 지시가 2곳 더** 있었다. blame 결과 이 절은 upstream `dlunch/wie` 에 **없는 우리 자작**이고 커밋 `1960d9c9`(«codify git workflow — **merge to main**», 2026-07-12)이 **두 파일에 동시에** 심은 것이다. 근인 커밋이 하나인데 사본이 둘이었으므로, 3곳만 고쳤으면 살아 있는 사본이 남았다.
- **사용자 영향**: 없음(문서 전용). 다만 검수 전 머지가 구조적으로 막히면 **미검수 변경이 `main` 에 들어가 featurephone 으로 자동 전파될 확률이 낮아진다** — `publish-artifact.yml` 이 `main` push 에서 발행·dispatch 하므로 게이트② 우회는 곧 사용자 도달 경로의 우회였다.
- **후속 추천**: ①(운영자) `main` branch-protection 에 «PR 승인 필수» 를 걸면 문서가 아니라 **플랫폼이** 강제 — 현재 강제력은 여전히 문서뿐이다(`wie-main-branch-protection` human-step 미적용 상태) ②`-merge` 티켓 문안 표준화 시 `AGENTS.md` §For the `-merge` task only 를 참조점으로 사용.

## [2026-08-01] RUSTSEC 이월분 등재 + 신규 권고 2건 판정 (wie-rustsec-advisory-sweep-batch2)
- **무엇을**: ①`docs/project-kb/02_status.md` 에 **«공급망 추적 대장»** 신설 — 배치1(2026-07-31)이 완료 보고서 본문에만 남겼던 이월분을 KB 로 옮겨 등재(**A. 권고·공급망 3건** + **B. upstream 이슈 9건**, 총 **12행**). ②신규 경고 2건 판정: `event-listener` **RUSTSEC-2026-0221**(unsound) · `spin` **0.12.0 yanked**. ③upstream 9건 재조회. ④로드맵에 발권 대기 항목 6·7번 추가. **문서 전용 — 코드·`Cargo.lock`·워크플로 무변경.**
- **왜**: 배치1 게이트② 사후 감사가 «이월 목록이 done 본문 단 한 곳에만 존재 = 사실상 유실»(severity major)로 지목했다. 티켓·큐·KB 어디에도 없어 추적 대기열 밖에 있었고, 이 저장소의 반복 실패형이다. 등재가 곧 최소 산출물.
- **판정 — `event-listener` 5.4.1 (A-2)**: 권고 `patched = [">= 5.4.2"]`·`unaffected = ["< 5.1.0"]` 이므로 **버전축으로는 affected 구간**이다. 그럼에도 **도달 불가** — 결함 대상은 `listener!` 매크로가 만드는 `StackSlot` 이고 전제는 `Event::with_tag` 의 `!Send` 태그인데, 유일 소비자 `jvm`(RustJava fork `c66f08d4`)은 `Event::new()`(태그 `()`=Send+Sync) + 힙 `EventListener` 만 쓴다. fork 전수 grep 에서 `listener!` **0** · `StackSlot` **0** · `Event::with_tag` **0**(`with_tag` 2건은 상수풀 파서 `parse_with_tag` 로 무관). 우리 repo 의 `EventListener` grep **22건은 전부 에뮬레이트되는 자바 클래스명**(`org/kwis/msp/lwc/EventListener` 등)이라 Rust 크레이트와 무관 — ★순진한 grep 이 «22건 도달» 로 오독되는 자리.
- **판정 — `spin` 0.12.0 yanked (A-3)**: ★**yanked 는 취약점이 아니다** — RustSec 의 `spin` 권고 3건은 전부 미해당(`>=0.5.2`·`>=0.9.8` 이미 상회 · 나머지는 `unaffected=[">= 0"]` 무력화 이력)이고, yanked 는 crates.io 인덱스 상태라 `cargo audit` 도 exit 0 으로만 보고한다. **그럼에도 무시하면 안 되는 이유**: 2026-07-13 하루에 6개 라인 패치본이 동시 발행되고 선행본이 일괄 yanked 된 형상이라 건전성 결함이 의심됐다. 추정하지 않고 `.crate` 2본을 내려받아 diff 해 사유를 확정 — `0.12.1`=`lock_api` 경로 `RwLock::try_upgrade` unsoundness, `0.12.2`=`Once::*into_inner*` unsoundness. **우리는 `default-features=false, features=["spin_mutex","rwlock"]`** 라 `once`·`lock_api`·`lazylock` 이 전부 미활성 = **두 결함 모두 컴파일조차 되지 않는다**(기계 확증: `Cargo.lock` 의 `spin` 블록에 의존 항목 0). 소스도 `spin::` 13건 전부 `Mutex`/`RwLock` 뿐.
- **upstream 9건 재조회 결과**: **#1292 가 2026-07-31 CLOSED/COMPLETED**(작성자 자진 종결) — 배치1 의 «실질 개발 = 1292·1260·1122» 는 **1260·1122 2건**으로 갱신. ★단 종결이 우리 블로커 해소를 뜻하지 않는다: 같은 문제(트랙② §7 `0x64` ordinal 표)에 대해 upstream 메인테이너가 **저작권 사유로 펌웨어/공식 에뮬레이터 리버싱을 하지 않는다**고 명시했으므로 «해결됨» 이 아니라 «upstream 이 다루지 않음» 이다 — 트랙② 동결 판단은 그대로 유효. #1240 은 REOPENED(upstream PR #1291 진행 중). 문의 2건(#1253·#1127) 분리 유지.
- **사용자 영향**: 없음(문서 전용). 다만 **매일 도는 `cargo audit` 의 비게이팅 경고 3건이 이제 KB 대장과 1:1 대응**하므로, 경고가 늘거나 줄면 어느 쪽이 낡았는지 즉시 드러난다 — 경고가 잊혀 쌓이던 실패형이 닫힌다.
- **경계**: **억제 수단 신설 0**(`--ignore`·`audit.toml` 없음 — 현재의 «취약점 0» 은 억제로 만든 0이 아니다) · **구현 0**(lockfile 무변경) · upstream 외부 발신 0(코멘트·이슈 작성 없음, 읽기 조회만).
- **후속 추천**: ①대장 A-2·A-3 상향 소티켓 2건 발권(각 dry-run **1패키지** 이동 실측 — `event-listener` 5.4.1→5.4.2, `spin` 0.12.0→0.12.2. 긴급도 0, 위생 부채) ②대장 B 의 #1260·#1122 건별 발권 ③(운영자 결정) upstream 이슈에 근거 코멘트를 남길지 — 외부 발신이라 미실행.

## [2026-07-22] main 브랜치 보호 — 계약 게이트 강제력 (wie-main-branch-protection)
- **무엇을**: ①`engine-contract.yml` 의 `contract` 잡을 **always-run 래퍼**로 전환(트리거 `paths:` 제거 + 잡 내부 `dorny/paths-filter` 감지 — 경로 미해당 시 즉시 성공, 잡 이름 `contract` 안정 유지). ②`web.yml` 프로덕션 배포를 Rust CI 에 배선하지 **않기로** 판단하고 근거를 워크플로 주석으로 문서화. ③운영자용 branch-protection **제안**(ruleset JSON)은 done 에 human-step 으로 분리(워커 미적용).
- **왜**: `main` 이 완전 무방비(`branches/main/protection`→404, `rulesets`→[])라 engine-contract 가 red 여도 머지되고 직push 도 열려 있었다. required check 로 걸려면 잡이 항상 상태를 보고해야 하는데, `paths:` 필터 잡을 그대로 required 지정하면 경로 미해당 PR 이 "Waiting for status" 로 영구 교착 — 그래서 래퍼가 필요.
- **정정(과장 시정)**: 종전 REPORT 의 "계약을 깨는 엔진 변경이 **PR 단계에서 차단**" 은 과장이었다. 현재 PR 단계 차단력은 0이고(래퍼+보호설정 적용 전), fail-closed 인 것은 릴리스 게이트(`publish-artifact.yml`)뿐 — featurephone **사용자 도달 경로는 이미 차단**돼 있으나 PR 조기 차단은 이 티켓의 보호설정(human-step)을 적용해야 성립.
- **배포 판단**: `web.yml` 의 D1 마이그레이션+Pages 배포는 같은 `build-web` 잡의 후속 스텝이라 이미 자기 빌드 성공에 의존. Rust CI 에 `workflow_run` 배선은 하지 않음 — 올바른 통제점은 **머지 게이트**(보호설정)이고, 크로스-워크플로 배선은 배포 중단 위험만 키운다. 최소·가역 원칙에 따라 무변경 + 주석 문서화.
- **후속 추천**: ①운영자가 done 의 ruleset JSON 을 한 번 적용(required checks + PR-before-merge, **리뷰 승인 필수는 제외** — 단독 소유자 교착 방지) ②`enforce_admins` 상당(bypass_actors) 옵션은 긴급 핫픽스 경로 장단 검토 후 선택.

## [2026-07-22] Security audit schedule 상시 red 정정 (wie-security-audit-schedule-red)
- **무엇을**: 매일 도는 `Security audit`(`rust-audit.yaml`) 잡을 `rustsec/audit-check` 액션 → `cargo audit` 직접 실행으로 전환. 무효 권한 `issues/checks: write` 제거. KB `02_status.md` 의 "green(2026-07-10 해소)" 오독 기록을 실측(schedule 28건 전건 failure)대로 정정.
- **왜**: 이 repo 는 `dlunch/wie` fork 라 Issues 가 기본 비활성이고 repo-레벨 disable 은 토큰으로 못 넘긴다. 그런데 audit-check 이 경고 2건(ttf-parser unmaintained·spin yanked)을 Issue 로 올리려다 `Issues has been disabled` 로 매 schedule 런에서 죽었다. 선행 커밋 `7405c50b` 의 `issues: write` 는 성립 불가한 처방이었고 같은 커밋이 KB 를 red→green 으로 오기록. 2026-07-10 "green" 은 check-run 경로를 타는 dispatch 런 오독.
- **성격 규정(과장 금지)**: 이것은 공급망 **차단 게이트가 아니다**(workflow_run 소비 참조 0건, publish-artifact 독립). 탐지(cargo audit)는 정상 작동했고 red 로 보였으므로 무성 실패도 아님 — 죽었던 것은 **알림 채널과 신호 대 잡음비**.
- **부류 분리(이 티켓의 실질)**: 취약점(count>0) → `cargo audit` exit 1 = **red**. 경고(unmaintained/yanked) → exit 0 = **green**(비게이팅, 로그엔 보임). `continue-on-error`·전면 억제 없이 종료코드로 구분. quick-xml 2건은 개별 `--ignore` 유지(제거 시 red — 실증됨).
- **후속 추천**: ①ab_glyph→skrifa 이행으로 ttf-parser 경고 실제 해소(미래 트랙, 02_status 5번) ②schedule 최초 야간 런(다음 00:00 UTC)이 green 이면 정정 최종 확증.

## [2026-07-22] featurephone 소비 계약 드리프트 가드 (wie-featurephone-engine-contract-selftest)
- **무엇을**: featurephone 웹이 의존하는 엔진 계약(아티팩트 쌍·glue API·키 어휘·세이브 블롭·clean-exit 체인·dispatch payload)을 `docs/contracts/` 에 핀하고, 정적 검사 + 실브라우저 부팅 왕복 검사를 PR CI(`engine-contract.yml`)와 릴리스 게이트(`publish-artifact.yml`)에 이중 편입.
- **왜**: 웹 셸의 부팅 셀프테스트 제거(2026-07-20)로 사라진 커버리지를 엔진 쪽 CI 가 인수 — 엔진 변경이 웹을 깨면 엔진 레포에서 먼저 실패(운영자 지시, 제안 #p2 채택). 웹 레포는 무변경.
- **사용자 영향**: main 에 계약 파손이 들어가도 릴리스 게이트가 발행·전파를 fail-closed 차단 — 사용자가 깨진 화면을 볼 확률↓. (★정정: PR 단계 조기 차단은 `wie-main-branch-protection` 의 보호설정 적용 후에만 성립. 이 티켓만으로는 PR 차단력 없음.)
- **후속 추천**: ①화면을 그리는 초소형 픽스처 추가로 blit 회귀까지 커버 확장(현 한계) ②로컬 main 분기(로컬 전용 커밋 0f13ab87) 브랜치·PR 경유 정리.
