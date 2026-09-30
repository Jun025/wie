## [2026-10-01] `a23f3c9fc2cb` LGT 행 limited → playable — #420 착지 뒤 main 장시간 재측 3/3 무오류 (wie-kroisen-a23f3c9fc2cb-restore-playable-after-420)

**무엇을**: `docs/player-data/compat.json` 의 `a23f3c9fc2cb` 행 1개를 `limited · longplay no` 에서 `playable · longplay ok` 로 되돌리고 knownIssue 「플레이 도중에 게임이 멈추거나 꺼질 수 있어요.」를 지웠다. 엔진 코드는 바꾸지 않았다.

**왜**: #420 이 이 행을 limited 로 적은 근거(`Unknown LGT WIPIC SVC id 2000`)는 #419(`449e470e`)를 조상으로 갖지 않는 #420 head 빌드에서 났다(#426 · `docs/report/0394`). #419·#420 이 모두 들어간 `origin/main 414233f3` 빌드로 다시 쟀다.

| 회차 | 시각 | host-load-guard | A (30초·27키) | L (600초) | 오류 | census 판정 |
|---|---|---|---|---|---|---|
| 1 | 04:57 | rc=0 · load 10.5 | PASS | 854/900키 · deadline · 오류 0 | 0 | playable · longplay ok |
| 2 | 05:08 | rc=0 · load 8.4 | PASS | 854/900키 · deadline · 오류 0 | 0 | playable · longplay ok |
| 3 | 05:20 | rc=0 · load 8.3 | PASS | 854/900키 · deadline · 오류 0 | 0 | playable · longplay ok |

- 방법: `wie_validate` release(`414233f3`) · `playability-census.mjs run --jobs 1` · build-slot 경유 · 회차마다 `report` 로 판정.
- L 의 `UNMEASURED · rc=2` 는 600초 예산 끝(`stop: deadline`)이지 오류가 아니다 — census 의 longplay 는 FAIL 줄만 error 로 센다. java 예외 0 · stderr 의 Unknown/panic 0.
- ★sound 축은 3/3 모두 census 가 `ok` 로 쟀지만(재생 42회) 이 회차 범위 밖이라 행의 `sound: no` 와 「소리가 나지 않을 수 있어요.」는 그대로 두었다.

**사용자 영향**: 지원 현황 playable 366 → 367 · limited 43 → 42. (티켓 문안의 354 → 355 · 49 → 48 은 #420 시점 수이고, 그 뒤 착지분이 수를 옮겼다.)

**게임 파일명 유입**: BOUNDED 330쌍 · SUFFIX-ATTACHED 15쌍 — 전부 `compat.json` 파일 단위 스캔이 잡은 기존 `title`·`fileTitle` 값이다. 이 회차가 더한 줄(`"status": "playable"` · `"longplay": "ok"` · player-updates · 이 문서)의 게임명은 0.

<!-- corpus-name-inflow v1 subjects=3 tree=6911bbe5a31f779f B=718/330 P=0/0 S=35/15 -->
