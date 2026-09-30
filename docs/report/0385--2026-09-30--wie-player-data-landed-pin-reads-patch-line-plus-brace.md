## [2026-09-30] 공개 데이터 빌드의 개선 이력 핀이 «+}» 로 실려 셸이 compat·updates 를 통째로 거부하던 것 (wie-player-data-landed-pin-reads-patch-line-plus-brace)

### 무엇을
- `scripts/player-data.mjs` `landedPin()` 에 `--no-patch`. `git log --diff-merges=first-parent` 가 git 2.55 에서 패치 출력을 함께 켜, `.pop()` 한 마지막 줄이 JSON 파일 diff 의 `+}` 였다.
- `build` 끝에서 산출물 전체를 셸 규칙(otterpebble `compat-import.mjs`·`updates-import.mjs`)으로 검증 — `validateCompat` 가 `changes[]`(date · enginePin 40hex|null · summary_ko)를 보게 했고, `validateBuiltUpdates`(wieHead · enginePin 40hex · id 중복). 위반이면 아무것도 쓰지 않고 rc=1.
- `--selftest` 에 실제 git 저장소 픽스처(merge 커밋이 파일을 들임)로 `landedPin` = 머지 커밋 단언 + 위 두 검사의 `+}` 변이 거부.

### 왜
라이브 지원 현황이 1차 조사(261)에 멈춰 있었다. 릴리스 `engine-5c32532b` 의 compat 는 2차(349/50/30)였으나 셸 수신이 `compat 거부 — 위반 35건`·`updates 거부 — 위반 18건`(전건 `enginePin: "+}"`)으로 버리고 엔진 핀만 범프했다. 빌드 쪽 검사는 **원천** 파일만 쟀고 **산출물**은 재지 않았다.

### 실측 (로컬 git 2.55.0)
- 수정 전 `build`: updates 46건 중 18건 `enginePin: "+}"`(명시 핀 없는 전건).
- 수정 후 `build`: 비-40hex 0 · 셸 `validateCompat`/`validateUpdates`(otterpebble origin/main 사본으로 실행) 위반 **0 / 0**.
- 되돌리기: `--no-patch` 제거 → selftest `landedPin must be the merge commit, got "+}"` rc=1 · build 는 산출물 검사에서 rc=1, 출력 디렉터리 비어 있음.

### 사용자 영향
머지 뒤 publish 가 새 릴리스를 내면 사이트 지원 현황·업데이트 소식이 2차 조사 결과로 바뀐다.
