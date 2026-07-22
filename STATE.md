# STATE

## 진행중
- `wie-featurephone-engine-contract-selftest` — 구현·로컬 검증 완료, PR 게이트② 리뷰 대기.
  브랜치 `feat/wie-featurephone-engine-contract-selftest`. 리뷰 approve 시 squash 머지
  (`gh pr merge --squash --delete-branch`, 머지 커밋에 `[wie-featurephone-engine-contract-selftest]` 태그).

## 완료 (최근)
- 2026-07-22: main 브랜치 보호 코드 준비 (`wie-main-branch-protection`) — `engine-contract.yml`
  `contract` 잡을 always-run 래퍼로 전환(`paths:` 제거 + 내부 `dorny/paths-filter` 감지, 경로
  미해당 시 즉시 성공·잡 이름 안정)해 required check 로 걸 수 있게 준비. `web.yml` 배포는 머지
  게이트가 통제점이라 무배선 판단(주석 문서화). REPORT.md 의 "PR 단계 차단" 과장 정정.
  ★human-step 잔여: 운영자가 branch-protection ruleset 1회 적용(리뷰 승인 필수 제외) — 상세는
  `~/orchestrator/reports/wie-main-branch-protection.done.md` C항.
- 2026-07-22: Security audit schedule 상시 red 정정 (`wie-security-audit-schedule-red`) —
  fork+Issues 비활성으로 `rustsec/audit-check` 의 Issue 생성이 매 schedule 런 실패하던 것을
  `cargo audit` 직접 실행으로 전환(무효 `issues/checks: write` 권한 제거). 취약점=red / 경고=green
  부류 분리 확립(로컬 실증). KB `02_status.md` 의 오독 green 기록도 실측대로 정정.
- 2026-07-22: featurephone 소비 계약 드리프트 가드 구현 — `docs/contracts/` 계약 핀 +
  정적/브라우저 왕복 검사기 + `engine-contract.yml`(PR CI) + `publish-artifact.yml` 릴리스
  fail-closed 게이트. 상세: `docs/worklog/2026-07-22--featurephone-engine-contract-selftest.json`.

## 다음
- (게이트② 후) PR 머지 + 로컬 main 동기화.
- 로컬 main 이 origin/main 과 분기 상태(로컬 전용 커밋 `0f13ab87` chore: 자율운영 하드닝, ahead 1 / behind 12).
  별도 소티켓으로 해당 커밋을 브랜치→PR 경유 착지시키고 로컬 main 을 origin 에 재동기 필요.
- (선택) 화면을 실제로 그리는 초소형 픽스처를 추가하면 왕복 검사가 blit 회귀까지 커버 가능(현재 한계).
