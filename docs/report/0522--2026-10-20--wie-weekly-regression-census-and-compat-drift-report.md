## [2026-10-20] 주간 퇴행 감시 — origin/main 전수 프로브 → compat 대조 → 내림만 짝 재측 (wie-weekly-regression-census-and-compat-drift-report)

**무엇을**: `scripts/regression-census.sh` 와 `scripts/regression-diff.mjs` 를 더했다. 앞의 것은 현행 `origin/main` 의 release `wie_validate` 로 전 타이틀을 잰다. 뒤의 것은 그 결과를 공개 지원 현황 `docs/player-data/compat.json` 과 행·축 단위로 대조한다. 그리고 `census-drive.sh` 에 `CENSUS_DEADLINE` 을 더했다(회차 6시간 상한). 엔진 코드 변경은 0이다.

**왜**: compat 는 행 단위로 고쳐 왔다. 한 핀에서 전 행을 잰 것은 6차(10-04 · `4ac38566`)가 마지막이다. 그 뒤 엔진 PR 들은 걸린 타이틀과 가드 2 · 라이브 LGT 5 만 다시 잰다. 그래서 나머지 400여 행의 퇴행은 아무도 보지 않았다. 게임 파일은 저장소 밖에 있어 CI 로는 잴 수 없다. 그래서 이 일은 호스트 로컬 주기 작업으로 둔다. 주기 등록(launchd)은 ops 쪽 별도 티켓이 한다.

**사용자 영향**: 없다. compat 를 쓰지 않으므로 지원 현황 페이지는 그대로다. 내림이 재현되면 원장에 한 줄이 남는다. 그 줄이 다음 티켓의 출발점이 된다.

### 1. 판정 규칙(`regression-diff.mjs` 머리 주석이 정본)
- 내림은 측정된 값이 더 나쁜 측정값으로 바뀐 것이다(ok > partial > no). `unknown`(`n/a`) 은 어느 쪽으로도 세지 않는다.
- boot · render · input 은 모든 행에서 비교한다. longplay 는 이번 회차에 600초 실행이 실제로 돈 행만 비교한다. sound 는 그 행들만 양방향으로 비교한다. 나머지 행은 올림만 센다. 프로브가 소리를 못 들었다는 것은 타이틀 화면 뒤에서 음악이 시작되는 타이틀에 대해 아무것도 말하지 않기 때문이다.
- status 도 비교한다. 장주행 밖의 행은 longplay `n/a` 때문에 `limited` 로 읽힌다. 그것만 모자란 경우에는 compat 의 status 를 그대로 둔다. 단, compat 에 없던 서버·인증·잠금 문장이 새로 나타났으면 그 문장이 status 를 묶는다. 실측 회차에서 `1a69522a7d43` 이 오탐이 됐다. compat 에 이미 있던 «온라인 등록» 서버 안내가 새 벽으로 읽힌 것이다. 그래서 `55cb621e` 에서 고쳤다.
- 내림 후보는 같은 바이너리로 짝 재측을 1회 한다(`pair-corpus` 는 sha12 이름의 심링크다). 같은 축에서 다시 내려간 것만 «퇴행 의심»이 된다. 올림은 참고 표로만 남긴다.

### 2. 수동 1회 실전(2026-10-20 01:14 → 03:58 KST)
- 측정 바이너리 `13aee5b5f6c87fa1d5d8fcda96d2bb4ccbf0f7a2`(origin/main)와 대조 기준 compat(enginePin `4ac38566` · 429행)을 썼다.
- **걸린 시간은 9,799초(2.72h)다.** 내역: 프로브 A·B 429종(`--budget 900` 임대 10회 · `--jobs` 3 · 표시 load1 8–24) → 장주행 9행 중 7행 실행. 두 행은 input 이 ok 가 아니어서 census 가 장주행 후보로 고르지 않았다. 그다음 짝 재측 2행.
- 측정 429 · 미측 0 · 굶은 프로브 0.
- **퇴행 의심 2행 · 올림 0.** 두 행 모두 compat 에서 «구매한 휴대폰에서만 켜지도록 잠긴 파일»(`not-yet`)이고, 행 status 는 `not-yet → not-yet` 그대로다:
  - `44b6356d13f8` input ok → no (짝 no)
  - `eefc947d8337` render ok → no (짝 no)
  - 다음 행동: 잠긴 파일이라(운영자 정책 2026-10-02 · 해제 금지) 플레이어가 보는 상태는 바뀌지 않는다. 다만 축 값은 실제로 내려갔다. 거절 화면을 그리기 전에 끝나는 쪽으로 바뀌었는지, `4ac38566` 과 `13aee5b5` 를 같은 시각 짝으로 재서 좁힌다. 이것은 별도 티켓의 몫이다. compat 축 갱신도 그 티켓이 한다.
- 원장: `WIE_REGRESSION_SUSPECT n=2 pin=13aee5b5`.
- 증적: `~/orchestrator/reports/evidence/wie-weekly-regression/2026-10-20.md`(sha12 만). 측정 원본(프레임 · 이름이 든 census.tsv)은 `~/wie-regression/2026-10-20/` 에 있다. 저장소 밖이고, 한 회차가 1.8 GB 다. 최근 2회차만 통째로 남긴다.
- 도구 핀: 회차는 `dead0c0f` 에서 시작했다. 대조 단계(`changed` 이후)는 `55cb621e` 의 `regression-diff.mjs` 로 돌았다. 증적 md 는 이 PR 의 head 로 다시 만들었다(같은 입력에 «행 상태» 열을 더했다).

### 3. 검증
- `bash scripts/regression-census.sh selftest`(= `regression-diff.mjs selftest`) 결과는 14/14 다. 가짜 compat 2판에서 «내림 1 · 올림 1 · 불변 1»을 판정하고, 장주행·벽·짝 규칙도 본다. **되돌리면 red**: 내림 판정을 빼는 변이에서 6/11 FAIL · rc=1 이 났다(그 시점 케이스 11개).
- 고정 fixture 코퍼스(`test_data/`)로 스모크를 돌렸다. 원장은 끄고 돌렸다. 빌드 → 프로브 → 장주행 → 대조가 rc 0 으로 끝났다. base 에 합성 내림 1행을 넣으면 짝 재측 → «재현»까지 간다.
- 4 게이트: fmt · clippy stable/wasm32/beta `-D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` 784/0 · `npm run audit` PASS.
- 회차 종료 시 남은 프로세스 0 · census 락 해제 확인.

### 4. 한계
- 장주행 밖의 행에서 longplay 와 sound 의 퇴행은 보지 못한다. 직전 회차보다 프로브 축이 바뀐 행만 장주행을 받기 때문이다. progress 축(30분 정책)은 재지 않는다.
- 짝은 «재현성»을 확인할 뿐 «원인»을 확인하지 않는다. compat 행이 손으로 고쳐졌을 수 있다(레시피 실행 등). 그런 행은 프로브가 매번 같은 값을 내므로 의심으로 남는다. 위 2행이 그런 행인지는 아직 가르지 않았다.
- 같은 날 다시 돌리면 이어서 잰다. 그때 «걸린 시간»은 그 호출의 시간만 센다.
