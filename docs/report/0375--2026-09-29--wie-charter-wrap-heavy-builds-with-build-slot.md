## [2026-09-29] CLAUDE.md — 무거운 빌드는 build-slot 으로 감싼다 (wie-charter-wrap-heavy-builds-with-build-slot)

**무엇을**: `CLAUDE.md` §에이전트 개발환경에 1줄 — 무거운 빌드·테스트는 `~/orchestrator-live/bin/build-slot run -- <cmd>` 로 감싼다
(머신당 슬롯 초과 시 대기 · 상한 30분 · rc 투과 · 라이브에 도구가 없으면 맨 명령). 문안은 정본 `templates/repo-CLAUDE.md`(orchestrator) 그대로.

**왜**: 이 Mac 은 여러 레인이 동시에 cargo/vite 빌드를 돌려 부하가 겹친다. 슬롯이 동시 실행 수를 제한한다.

**사용자 영향**: 없음(에이전트 작업 규율 · 코드·CI·배포 무접촉).

**남은 것**: 줄의 열거는 `wie_validate` 러너 루프·`scripts/smoke_gate.sh`·`cargo clippy`(+wasm·beta)를 이름으로 잡지 않는다
(게이트② 발견 1 · 처분 소관 = 정본 템플릿, orchestrator). 이 회차 파일은 게이트③ 이 동봉했다(구현 회차 Acceptance 가 diff 를 CLAUDE.md 1파일로 한정).
