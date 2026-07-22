# REPORT

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
