## [2026-07-22] main 브랜치 보호 — 계약 게이트 강제력 (wie-main-branch-protection)
- **무엇을**: ①`engine-contract.yml` 의 `contract` 잡을 **always-run 래퍼**로 전환(트리거 `paths:` 제거 + 잡 내부 `dorny/paths-filter` 감지 — 경로 미해당 시 즉시 성공, 잡 이름 `contract` 안정 유지). ②`web.yml` 프로덕션 배포를 Rust CI 에 배선하지 **않기로** 판단하고 근거를 워크플로 주석으로 문서화. ③운영자용 branch-protection **제안**(ruleset JSON)은 done 에 human-step 으로 분리(워커 미적용).
- **왜**: `main` 이 완전 무방비(`branches/main/protection`→404, `rulesets`→[])라 engine-contract 가 red 여도 머지되고 직push 도 열려 있었다. required check 로 걸려면 잡이 항상 상태를 보고해야 하는데, `paths:` 필터 잡을 그대로 required 지정하면 경로 미해당 PR 이 "Waiting for status" 로 영구 교착 — 그래서 래퍼가 필요.
- **정정(과장 시정)**: 종전 REPORT 의 "계약을 깨는 엔진 변경이 **PR 단계에서 차단**" 은 과장이었다. 현재 PR 단계 차단력은 0이고(래퍼+보호설정 적용 전), fail-closed 인 것은 릴리스 게이트(`publish-artifact.yml`)뿐 — featurephone **사용자 도달 경로는 이미 차단**돼 있으나 PR 조기 차단은 이 티켓의 보호설정(human-step)을 적용해야 성립.
- **배포 판단**: `web.yml` 의 D1 마이그레이션+Pages 배포는 같은 `build-web` 잡의 후속 스텝이라 이미 자기 빌드 성공에 의존. Rust CI 에 `workflow_run` 배선은 하지 않음 — 올바른 통제점은 **머지 게이트**(보호설정)이고, 크로스-워크플로 배선은 배포 중단 위험만 키운다. 최소·가역 원칙에 따라 무변경 + 주석 문서화.
- **후속 추천**: ①운영자가 done 의 ruleset JSON 을 한 번 적용(required checks + PR-before-merge, **리뷰 승인 필수는 제외** — 단독 소유자 교착 방지) ②`enforce_admins` 상당(bypass_actors) 옵션은 긴급 핫픽스 경로 장단 검토 후 선택.

