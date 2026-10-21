## [2026-10-21] 주간 퇴행 감시 보정 — 정책 잠금 행의 매주 오탐 제거 · 걸린 시간 오기 (wie-regression-census-locked-rows-and-elapsed-time)

**무엇을**: `scripts/regression-diff.mjs` 가 «정책 행»의 축 내림을 퇴행 후보로 세지 않게 했다. 증적에는 «정책 행 축 변화(참고)» 절로 따로 적는다. 정책 행의 `status` 변화는 그대로 센다. 증적의 «걸린 시간»은 회차의 첫 시작부터 잰다. compat 2행의 축 값을 실측에 맞췄다. 엔진 코드 변경은 0이다.

**왜**: 첫 실행(`docs/report/0522` §2)의 퇴행 의심 2행(`44b6356d13f8` input · `eefc947d8337` render)은 둘 다 «구매한 휴대폰에서만 켜지도록 잠긴 파일»(not-yet)이다. 운영자 정책(2026-10-02)상 해제하지 않는다. 그 행의 축은 거절 화면을 잰 값이고, 이용자 상태는 바뀌지 않았다. 지금 규칙이면 매주 `WIE_REGRESSION_SUSPECT` 가 울리는 영구 오탐이 된다.

**사용자 영향**: 지원 현황에서 두 잠긴 행의 축 한 칸씩이 실측대로 바뀐다(`44b6356d13f8` input ok → no · `eefc947d8337` render ok → no). status(not-yet)와 안내 문장은 그대로다. 플레이 방식은 바뀌지 않으므로 `docs/player-updates/` 항목은 없다.

### 1. 정책 행 판별
- 술어: `status ≠ playable` 이고 `knownIssues_ko` 가 **한 줄뿐**이며 그 줄이 벽 문장(`WALL = /서버|인증|잠긴/` · 같은 파일의 기존 상수)이다.
- 근거: census 는 잠김(`LOCK_KO` drm·phone)·통신망 벽(`NET_KO`)·손 기재 서버 벽(`HAND_WALL`) 행에 그 안내 한 줄만 싣는다(`featurephone-public-data.md` §knownIssues_ko «잠김·통신망 벽 행은 그 안내 한 줄뿐»). 현행 compat 에서 18행이 걸린다(잠김 8 · 서버/인증 벽 10). 모두 한 줄짜리다.
- 좁힌 범위: 벽 문장 옆에 다른 문제가 한 줄이라도 있는 행, `playable` 행(벽 안내가 붙은 «온라인 등록» 류)은 정책 행이 아니다. 일반 행의 내림 판정은 그대로다.
- 정책 행은 짝 재측 대상에서도 빠진다(`candidates` 가 같은 `down` 을 읽는다).

### 2. compat 2행 축 갱신 — 판정: 고친다
- 짝 재측까지 두 번 같은 값이 나왔다. 축은 그 핀의 실측과 맞아야 한다. 이용자 문장·status 는 불변이다.
- 바꾼 칸은 짝이 재현한 축 하나씩뿐이다. 이번에 안 잰 축(`n/a`: longplay·speed·progress)은 손대지 않았다.
- `node scripts/check-compat-revert.mjs` → `OK · 착지 기준 바뀐 행 2`. `node scripts/player-data.mjs` rc=0.

### 3. 걸린 시간
- 원인: 10-20 회차는 01:14 에 시작해 03:58 에 짝 재측까지 끝났다(9,799초). 05:12 에 같은 날 이어 돌리기로 다시 실행됐다. 그 실행은 디스크의 결과를 모두 재사용해 3초 만에 끝났고, `T0` 는 그 실행의 시작이었다.
- 수정: `regression-census.sh` 가 회차 디렉터리에 첫 시작 epoch(`started`)를 한 번만 쓴다. `report --started <epoch>` 가 거기서부터 잰다. 6시간 상한(`CENSUS_DEADLINE`)은 종전대로 실행 단위다.

### 4. 검증
- `node scripts/regression-diff.mjs selftest` 21/21. 추가 7건: 잠금 행 축 내림 → 후보 0 · 잠금 행 status 변화 → 후보 · 정책 행 올림은 일반 참고 · 일반 행 축 내림 → 후보 · 벽 문장 + 다른 문제 행은 정책 행 아님 · playable 행은 정책 행 아님 · 걸린 시간이 회차 첫 시작 기준(9,799s → `2.72h`).
- **되돌리면 red**: `policyRow` 를 항상 false 로 → 20/21 rc=1. 경과를 고정값으로 → 20/21 rc=1.
- **10-20 회차 재판정**(같은 `base-compat.json` · `out` · `pair`): 종전 도구 의심 **2** → 이번 도구 의심 **0**, 정책 행 축 변화(참고) 2.
- 4 게이트: fmt · clippy stable/wasm32/beta `-D warnings`(beta 는 `wie_cli` manifest 경고 7 · 기존) · `RUST_MIN_STACK=4194304 cargo test --all` 784/0 · `npm run audit` PASS · `check-docs-report-serial` OK.
- 게임 파일명 유입: 유입 330건(BOUNDED) + 판단 필요 16건(SUFFIX-ATTACHED) — 전부 `compat.json` 의 기존 `title` 필드다(그 파일만 넣어 재도 330·16 · 스크립트 2 + 이 문서만 넣으면 0·0).
