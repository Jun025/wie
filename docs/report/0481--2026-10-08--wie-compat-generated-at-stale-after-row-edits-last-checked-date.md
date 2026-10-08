## [2026-10-08] 지원 현황 «마지막 확인» 날짜 — 행 단위 갱신을 따라가게 (wie-compat-generated-at-stale-after-row-edits-last-checked-date)

**무엇을**: `scripts/player-data.mjs build` 가 내보내는 `compat.json` 의 `generatedAt` 을
max(커밋된 전수 조사 시각, `docs/player-data/compat.json` 을 건드린 마지막 first-parent 착지의 커밋 시각)으로 정한다(`checkedAt`).

**왜**: 라이브 `/support` 머리가 「마지막 확인 2026-10-04」에 멈춰 있었다. 그 뒤 #471·#473·#476·#493·#502·#512 등이
행을 고쳤고 라이브 수치도 그 값인데, `generatedAt` 은 전면 재생성(10-04)에서만 바뀌고 `build` 는 그것을 그대로 통과시켰다.
행을 고치는 PR 마다 `generatedAt` 을 올리게 하는 안(검사 강제)은 버렸다 — 모든 compat PR 이 같은 한 줄을 고치게 되어
형제 PR 이 서로 충돌한다(이 repo 가 STATE.md·REPORT.md 에서 없앤 바로 그 «공유 삽입점»이다). 행별 `checkedAt` 필드 안도
버렸다 — 셸 변경이 필요하고, 셸이 보여 주는 것은 머리 날짜 하나다.

**검사**: `--selftest` 가 임시 git 저장소에서 ⑴전수 조사(10-04) 뒤 브랜치에서 행을 고쳐(10-06) `--no-ff` 로 착지(10-08)하면
날짜가 착지 시각 10-08 로 옮겨지는지 ⑵compat 를 안 건드린 뒤 착지(10-09)는 안 옮기는지 ⑶착지보다 새 전수 조사 시각은 그대로인지
⑷`build` 가 내보내는 값(`buildOutput`)이 그것인지 ⑸`--depth 1` 클론은 전수 조사 시각을 그대로 내는지 잰다.
개악 실측(각각 rc=1): `checkedAt` 을 `return generatedAt` 으로 · 얕은 분기 삭제 · `buildOutput` 의 `checkedAt` 호출 한 줄 삭제 · `--first-parent` 제거(→ 브랜치 커밋 10-06 을 고른다).
게이트② 1차 반려(얕은 클론은 HEAD 시각을 낸다 — 아래 한계의 종전 문구가 거짓이었다)를 같은 PR 에서 고쳤다.

**사용자 영향**: 다음 엔진 릴리스(이 PR 착지 = `scripts/player-data.mjs` 경로 → `publish-artifact.yml` → 셸 import)부터
「마지막 확인」이 마지막 행 갱신 착지일을 보인다. 현 main 으로 로컬 build 한 값 = `2026-10-08T21:00:46+09:00`(#512).
셸 변경 0 — 필드 이름·뜻(«마지막으로 확인한 시각»)은 그대로이고, 셸 import 의 날짜 검사(`^\d{4}-\d{2}-\d{2}`)와 `slice(0, 10)` 이 오프셋 붙은 시각을 그대로 받는다.

**한계**: 얕은 클론이면 `--is-shallow-repository` 로 알아보고 전수 조사 시각을 그대로 낸다(그대로 두면 `git log -- <path>` 가 얕은 경계 = HEAD 시각을 돌려준다 · publish 는 `fetch-depth: 0`). compat.json 을 서식만 바꾼 착지도 날짜를 옮긴다.
