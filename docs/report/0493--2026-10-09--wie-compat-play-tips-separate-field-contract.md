## [2026-10-09] 진행 요령을 «알려진 문제»와 가른다 — compat `playTips_ko` (wie-compat-play-tips-separate-field-contract)

### 무엇을
- 계약 `docs/contracts/featurephone-public-data.md` §1 에 행 필드 `playTips_ko?: string[]`(선택 · 없으면 키 생략) 정의. `knownIssues_ko` = 안 되는 것만.
- `compat.json` **19행**의 진행 요령 1줄씩을 `knownIssues_ko` → `playTips_ko` 로 옮겼다(문장 무변경). 0482 의 14 + 걷기 요령 5(`61ed69520fd3` `c107462e5f8a` `d1dce4a36141` `7089dec0e8df` `ccb45e6b8d80` — 티켓은 3이라 했으나 `HAND_NOTE` 와 compat 에 5행이 있다). 그중 16행은 `knownIssues_ko` 가 빈 배열이 됐다(전부 playable).
- 남긴 것: `287af341dac8` «껐다 켜야 시작»(멈춤이 본문 — 문제)·소리 줄들.
- 전수 조사 `HAND_NOTE` 에서 요령을 `HAND_TIP` 으로 갈라 `report` 가 `playTips_ko` 로 내게 했다 — 재생성해도 «알려진 문제»로 돌아가지 않는다. 잠김·통신망 벽 행은 요령을 싣지 않는다.
- `player-data.mjs` 검사: 키가 있으면 비지 않은 문자열 배열(빈 배열 거부). selftest 사례 3 + 정상 1. `check-compat-revert` 필드 목록에 `playTips_ko` 추가.

### 왜
요령이 «알려진 문제»로 보이고, 플레이 전 안내(`knownIssues_ko[0]`)가 첫 줄만 보여 소리 줄 뒤의 요령이 묻혔다(0482 §4).

### 사용자 영향
셸이 `playTips_ko` 를 그리기 전까지(otterpebble `otterpebble-featurephone-play-tips-near-progress-badge-and-preplay` 착지 전) 19행의 요령이 화면에서 빠진다 — 두 착지 사이 몇 시간의 공백.

### 확인
- `node scripts/player-data.mjs` OK(429) · `--selftest` 35 규칙 · 비지 않음 규칙을 지우면 «NOT rejected — empty playTips_ko array» 로 red(되돌림 확인).
- `node scripts/playability-census.mjs selftest` 65/65 · `check-compat-revert` OK.
- 유입: 이 회차가 더한 줄에 새 제목 0 — 브랜치 전체 BOUNDED 330 · SUFFIX-ATTACHED 15 는 기존 `compat.json` title(0482 와 같은 값).

