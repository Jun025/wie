## [2026-09-26] 9/21 메타 도구 정리 — 쓰이지 않는 도구 삭제 · 필수 체크에서 메타 게이트 제외 (wie-meta-gates-trim-after-0921-audit)

**무엇을**
- 삭제: `scripts/audit-missing-tests.sh`(191줄 · 호출자 0).
- 삭제: `scripts/check-docs-report-serial.mjs` 의 worklog↔report 인용 검사 경로(#236 `CITE_RE`·`citedSerials`·`exclusionCounts`·`danglingCitations`·`myWorklogFiles`·`baseSerials`, 호출부, #249 가 더한 셀프테스트 8건). 594 → 351줄 · 셀프테스트 27 → 19건. 파일명 축(중복 연번 · 열린 PR claim)은 그대로다.
- `check-worklog-coverage.mjs` 단계를 필수 `contract` 잡에서 뺐다. 스크립트는 OVERDUE 를 `::warning` 으로 찍고 rc=0 을 낸다. 주간 비필수 `doc-liveness.yml`(DOC-COPY 로 이미 실행 중)에만 남는다. 70% 미만인데 아무도 답하지 않은 측정은 계속 rc=1 이다.
- `check-inflow-marker.mjs` 단계는 `continue-on-error: true` 로 바꿨다(권고용).
- AGENTS.md: 유입 도구 절 두 곳을 10줄짜리 한 절로 합쳤다(경위는 `0195`–`0196` 참조로 대신). 커버리지 절에서 gate③ 소유/red 서술을 걷어 냈다.

**왜** — 2026-09-25 총괄 감사. 커버리지 OVERDUE 는 09-20 이후 `main` 을 8회 red 로 만들었고, inflow 마커는 무관한 PR 3건을 red 로 만들었다. 인용 검사는 인용 123건에서 진탐 1 · 오탐 1 이었고, 그 셀프테스트가 정상 PR 을 red 로 만들어 #249 가 따로 고쳐야 했다.

**재현(OVERDUE 가 더는 red 가 아니다)** — 인위적인 카운트 조작 없이 재현했다. 이 회차 base(`5434ab0a`)가 실제로 OVERDUE 상태다(착지 235 · 마지막 기록 225).
- `origin/main` 판 스크립트: `✗ OVERDUE …` · **rc=1**
- 이 브랜치 판: `::warning … OVERDUE …` · `OK (with warning)` · **rc=0**

**전후 수치**
- `contract` 잡 단계: 24 → 23(커버리지 단계 제거). 이 잡에서 결과를 좌우하던 검사 중 빠진 것: `check-worklog-coverage`(제거), `check-inflow-marker`(권고용 전환). census 단계는 `wie-meta-machinery-cleanup-census-step-orphan-tests-ruleset-shape` 소관이라 건드리지 않았다.
- 제안 정리: 9/21 착지 21건(#212–#216, #227–#240, #243, #249)의 worklog 제안 **44건 모두 이미 채택·기각된 상태**였다. 이 회차가 닫은 것 0 · 남은 것 0.

**사용자 영향** — 없음. 엔진 코드와 사이트는 건드리지 않았다. 무관한 PR 과 `main` 이 기록 관리 기한 때문에 red 가 되는 일이 없어진다.
