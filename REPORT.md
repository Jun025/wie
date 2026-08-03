# REPORT

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
