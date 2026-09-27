## [2026-09-27] featurephone 공개 데이터 — 지원 게임 목록(compat.json) · 게임별 변경 이력(updates.json) · 엔진 릴리스로 배달 (wie-featurephone-public-compat-catalog-and-player-changelog)

**무엇을**
- `docs/contracts/featurephone-public-data.md` — 두 JSON 의 계약 · 등급 술어 · 배달 경로.
- `scripts/player-data.mjs` — 검사(기본) · `--selftest` · `import <전수 조사 compat.json>` · `build --out <dir>`.
- `docs/player-data/compat.json` — 전수 조사(#354) 결과 429종 첫 판(축 어휘를 계약 어휘로 바꾸고 `_` 를 띄어쓰기로).
- `docs/player-updates/*.json` 24건 — 한 항목 한 파일 · 소급분.
- `engine-contract.yml` `contract` 잡에 검사 단계 · `publish-artifact.yml` 이 릴리스에 두 파일을 싣고 `client_payload.publicData` 로 넘긴다.
- `AGENTS.md` §Landing paperwork — 게임 동작을 바꾸는 PR 은 항목 파일 1개.

**왜**: 운영자 지시 — 게임 파일은 주지 않되 지원 게임 목록과 게임별 수정 이력을 보여 «계속 관리되고 있음»이 느껴지게. 엔진은 수백 종을 재는데 이용자에게 보이는 면이 0 이었다.

**사용자 영향**: 이 PR 만으로는 화면이 바뀌지 않는다. 셸 수신부가 `publicData` 를 받아야 보인다(아래 «셸 변경 필요»).

### 스키마를 새로 정하지 않은 이유
전수 조사(#354 `report`)와 셸 가져오기(otterpebble #1141 `compat-import.mjs`)가 이미 같은 스키마를 쓴다. 두 벌을 만들지 않았다.
단 **축 값 어휘는 서로 달랐다** — 전수 조사는 `fail|none|uniform|error|silent|n/a`, 셸은 `ok|partial|no|unknown`.
전수 조사 산출을 셸에 그대로 넣으면 셸 검사기가 거부한다. `import` 가 그 사이를 바꾼다(계약 §1). 빌드 산출물은 셸 `validateCompat` 위반 **0**(실측).

### 첫 판 수치(핀 = 전수 조사 스크래치 `19effc06` = main `aeb09129` + #347·#348 head)
- 타이틀 **429**(sha256 고유) · playable **261** · limited **81** · not-yet **87**.
- 통신사: KTF 269(167/51/51) · LGT 78(40/26/12) · SKT 82(54/4/24) · J2ME 0.
- `unknown`(재지 않음): input 87 · longplay 114 · sound 68 · speed 267. 이유 — 앞 축이 ok 가 아니라 재지 않았거나(input·longplay·sound), 부하 속 속도는 하한이라 «느림»을 판정하지 않는다(speed).
- 재측정은 하지 않았다: 전수 1회 약 4시간이고, 같은 날 같은 도구가 잰 값이다. 핀 스크래치 커밋은 GitHub 에 없다 — 셸은 배포 핀과 달라 «새 판에서 다시 확인 전» 안내를 띄운다(의도대로).

### upstream 호환성 표(wie-site 19종)와 겹침
| 통신사 | 이름 | Api | upstream | 우리 |
|---|---|---|---|---|
| KTF | 짜요짜요타이쿤 | WIPI/Java | 플레이 가능 | playable |
| KTF | 짜요짜요타이쿤2 | WIPI/Java | 플레이 가능 | playable |
| KTF | 미니게임천국1 | WIPI/C | 플레이 가능 | playable |
| KTF | 영웅서기1 | WIPI/Java | 플레이 가능 | playable |
| KTF | 슈퍼액션히어로 | WIPI/C | 플레이 가능 | playable |
| KTF | 프린세스메이커4 | WIPI/Java | 플레이 가능 | playable |
| KTF | 미니게임천국2 | WIPI/C | 플레이 가능 | playable |
| KTF | 영웅서기2 | WIPI/Java | 플레이 가능 | playable |
| KTF | 액션퍼즐패밀리 | WIPI/C | 플레이 가능 | playable |
| KTF | 리듬스타1 | WIPI/C | 플레이 가능 | limited |
| KTF | 팝픈뮤직 | WIPI/Java | 플레이 가능 | playable |
| KTF | 미니게임천국4 | WIPI/C | 메뉴까지 | playable |
| SKT | 문명 | SKVM | 플레이 가능 | (코퍼스에 없음) |
| SKT | 킹덤언더파이어 | SKVM | 메뉴까지 | playable |
| SKT | 에이지오브엠파이어2 | WIPI/Java | 플레이 가능 | playable |
| LGT | 테일즈위버 막시민편 | WIPI/C | 인트로까지 | playable |
| LGT | 검은방2 | WIPI/C | 플레이 가능 | playable |
| LGT | 그랜드체이스 | WIPI/C | 플레이 가능 | playable |
| LGT | 검은방3 | WIPI/C | 플레이 가능 | playable |

- 겹침 18/19. 티켓의 «18종»은 실측 19행이다(플레이 가능 16 · 메뉴까지 2 · 인트로까지 1).
- 우리가 높게 본 3건(메뉴까지/인트로까지 → playable)은 등급 뜻이 다르다: 우리 playable 은 «키로 화면이 바뀌고 10분 조작에서 안 멈췄다»이고 끝까지 해 본 것이 아니다. 리듬스타1 은 우리 쪽 longplay 오류로 limited.

### 소급 항목(24건)
전수 조사가 PR 제목 ↔ 게임 이름으로 이은 42 PR 에서 골랐다. 뺀 22건: 원인 판정만 한 PR(«판정만» · «코드 변경 0» · «닫지 않음» 등)과, 다음 벽으로 한 칸 옮겼을 뿐 게임이 아직 안 되는 PR(월드장기체스·학교가는길 중간 단계 등). 게임 이름으로 잇다가 다른 통신사 판까지 붙은 것은 PR 이 말하는 통신사로 좁혔다. 전수 조사가 못 이은 #343·#345·#348·#349 는 손으로 더했다.
09-21 이전 병합분에는 플레이어 체감 변경이 없었다(테스트·검사기·문서 · #29 는 기준선 승격이라 동작 변경이 아니다).
목록은 `docs/player-updates/` 가 정본이다(날짜 · 종류 · 대상 · 요약 · PR).

### 배달
`publish-artifact.yml`: `player-data.mjs build`(전체 이력 필요 → `fetch-depth: 0`) → 릴리스 `engine-<sha>` 자산에 `compat.json`·`updates.json` 추가 →
`client_payload.publicData = { compatUrl, compatSha256, updatesUrl, updatesSha256 }`(최상위 키는 8개 — 10개 제한 안). 셸 수신부는 필수 키만 검사하므로 이 키가 더해져도 깨지지 않는다(실측 `wie-artifact-receive.yml`).
이 PR 의 착지는 `publish-artifact.yml` 자신을 바꾸므로 릴리스가 한 번 돈다 — 그 릴리스가 두 파일의 첫 배달이다.

### 셸 변경 필요(여기서 고치지 않음)
1. `wie-artifact-receive.yml`: `publicData` 가 있으면 두 URL 을 받아 sha256 대조 → `compat.json` 은 `compat:import` 로 `lib/compat.json` 에, `updates.json` 은 새 위치에 쓰고 같은 핀 범프 커밋에 싣는다.
2. `updates.json` 소비(«업데이트 소식» 피드) — 지금 셸 `recentChanges()` 는 게임별 `changes` 만 모은다. `titles: []` 인 전체 소식(예 #348 소리)은 `updates.json` 에만 있다.
3. `changes[].kind`·`pr` 은 새 필드다(셸 검사기는 무시한다) — 뱃지·링크에 쓰려면 셸이 읽어야 한다.

### 한계
- 표시 이름은 파일 이름에서 왔다. `[큰화]`·`에디트`·판 번호 같은 파일 표식이 남은 이름이 있다(41종에 괄호·밑줄·점이 있었고 밑줄만 고쳤다). 공개 목록에 보일 이름을 정리하는 일은 별 회차다.
- 한 게임의 여러 판(같은 이름 · 다른 sha)이 각자 한 행이다(같은 통신사·같은 이름 15묶음).
- 되돌리면 red: `--selftest` 13규칙. 실측 변이 3종(제목 실재 검사 · 등급 검사 · 파일명 날짜 검사를 끔) 모두 rc=1.

### 게임 이름 유입
`corpus-name-inflow`: 유입 349건(BOUNDED) · 판단 필요 16건(SUFFIX-ATTACHED). **의도된 유입이다** — 공개 «지원 게임 목록»이 이 PR 의 산출물이고 이름은 `docs/player-data/compat.json` 과 위 upstream 겹침표에만 있다. 항목 파일(`docs/player-updates/`)의 문장에는 이름이 0 이다(대상은 sha256 으로 가리킨다). 게임 바이트 · 화면 캡처 · 키 스크립트 커밋 0.

<!-- corpus-name-inflow v1 subjects=31 tree=e08894b78f1b9f3d B=394/349 P=0/0 S=20/16 -->
