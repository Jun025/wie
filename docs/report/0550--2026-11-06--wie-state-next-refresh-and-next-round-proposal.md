## [2026-11-06] STATE.md «다음» 현행화 — 옛 ①~④ 를 완료 1줄로 접고 사료는 `docs/state-next-history.md` 로 · 다음 후보 2건 (wie-state-next-refresh-and-next-round-proposal)

**무엇을**: `STATE.md` 의 `## 다음`(약 300줄)과 머리 블록·「fork 의 현재 위치」 표(2026-08-27 기준)를
`docs/state-next-history.md` 로 **그대로** 옮겼다(내용 수정 0). `STATE.md` 에는 2026-11-06 실측 위치 표와
«무엇 · 왜 지금 · 근거 · size» 형식의 후보 2건, 보류 축 3건, 완료로 접은 항목 6줄만 남겼다.
`## 진행중`·`## 완료` 고정 안내는 그대로다. 코드 변경 0.

**왜**: LANE_IDLE(2026-11-06 11:30 · 큐 0 · 66h 조용). 옛 «다음» 의 ①(P1·P2·P3)·③·④ 는 전부 끝났거나 닫혔는데
절이 착지분 서술로 길어져 지금 칠 것이 안 보였다.

**사용자 영향**: 없다(문서만).

### 실측 (2026-11-06)
- `git merge-base origin/main upstream/main` = `44fbf265` — 조각 D(PR #161, 2026-09-16) 이후 불변.
  behind **173** / ahead 2,970. upstream 쪽 172커밋(병합 제외, 09-16~11-06) · 309파일 +32,181/−9,779.
  `git merge-tree --write-tree --name-only origin/main upstream/main` → rc=1 · 충돌 파일 **93**.
- RustJava: `Cargo.toml` 이 crates.io `jvm*` `^0.1.1` 를 쓴다 — 옛 «다음 칸 = ba5797b(+47)» 는 대상이 사라졌다.
- compat 429행: playable 426 · not-yet 2 · limited 1 · `progress=stuck` 91. `0525` 원장의 ⒟ 37행은 전부 아직 `stuck`.
- 주간 퇴행 감시: launchd 매주 일요일 05:00 → 다음 **2026-11-08 05:00**.
- 열린 PR 0(`gh pr list -R Jun025/wie --state open`).

### 다음 후보 (발권 안 함 — 총괄 몫)
1. upstream 2차 동기화 예행·분류(M · `main` 머지 0).
2. 진도 ⒟ `confirm-only-entry` 15행 현행 핀 재측 + 원인 판정(M).
상세는 `STATE.md` §다음.

### 제안 처분 1건
- 제안 `2026-09-28-modified-utf8-classfile-constants#p0`(classfile 수정을 upstream 에 제안)은 dlunch/RustJava **#208 MERGED 2026-09-28** 로
  이미 이뤄졌다 → `docs/worklog/2026-11-06-state-next-refresh.json` 의 `declinedProposals` 로 처분했다.
  `patches/classfile` 제거 조건(crates.io 새 판) 확인은 이 회차 범위 밖이다.
