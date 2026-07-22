# REPORT

## [2026-07-22] Security audit schedule 상시 red 정정 (wie-security-audit-schedule-red)
- **무엇을**: 매일 도는 `Security audit`(`rust-audit.yaml`) 잡을 `rustsec/audit-check` 액션 → `cargo audit` 직접 실행으로 전환. 무효 권한 `issues/checks: write` 제거. KB `02_status.md` 의 "green(2026-07-10 해소)" 오독 기록을 실측(schedule 28건 전건 failure)대로 정정.
- **왜**: 이 repo 는 `dlunch/wie` fork 라 Issues 가 기본 비활성이고 repo-레벨 disable 은 토큰으로 못 넘긴다. 그런데 audit-check 이 경고 2건(ttf-parser unmaintained·spin yanked)을 Issue 로 올리려다 `Issues has been disabled` 로 매 schedule 런에서 죽었다. 선행 커밋 `7405c50b` 의 `issues: write` 는 성립 불가한 처방이었고 같은 커밋이 KB 를 red→green 으로 오기록. 2026-07-10 "green" 은 check-run 경로를 타는 dispatch 런 오독.
- **성격 규정(과장 금지)**: 이것은 공급망 **차단 게이트가 아니다**(workflow_run 소비 참조 0건, publish-artifact 독립). 탐지(cargo audit)는 정상 작동했고 red 로 보였으므로 무성 실패도 아님 — 죽었던 것은 **알림 채널과 신호 대 잡음비**.
- **부류 분리(이 티켓의 실질)**: 취약점(count>0) → `cargo audit` exit 1 = **red**. 경고(unmaintained/yanked) → exit 0 = **green**(비게이팅, 로그엔 보임). `continue-on-error`·전면 억제 없이 종료코드로 구분. quick-xml 2건은 개별 `--ignore` 유지(제거 시 red — 실증됨).
- **후속 추천**: ①ab_glyph→skrifa 이행으로 ttf-parser 경고 실제 해소(미래 트랙, 02_status 5번) ②schedule 최초 야간 런(다음 00:00 UTC)이 green 이면 정정 최종 확증.

## [2026-07-22] featurephone 소비 계약 드리프트 가드 (wie-featurephone-engine-contract-selftest)
- **무엇을**: featurephone 웹이 의존하는 엔진 계약(아티팩트 쌍·glue API·키 어휘·세이브 블롭·clean-exit 체인·dispatch payload)을 `docs/contracts/` 에 핀하고, 정적 검사 + 실브라우저 부팅 왕복 검사를 PR CI(`engine-contract.yml`)와 릴리스 게이트(`publish-artifact.yml`)에 이중 편입.
- **왜**: 웹 셸의 부팅 셀프테스트 제거(2026-07-20)로 사라진 커버리지를 엔진 쪽 CI 가 인수 — 엔진 변경이 웹을 깨면 엔진 레포에서 먼저 실패(운영자 지시, 제안 #p2 채택). 웹 레포는 무변경.
- **사용자 영향**: 계약을 깨는 엔진 변경이 PR 단계에서 차단되고, main 에 들어가도 릴리스 게이트가 발행·전파를 fail-closed 차단 — 사용자가 깨진 화면을 볼 확률↓.
- **후속 추천**: ①화면을 그리는 초소형 픽스처 추가로 blit 회귀까지 커버 확장(현 한계) ②로컬 main 분기(로컬 전용 커밋 0f13ab87) 브랜치·PR 경유 정리.
