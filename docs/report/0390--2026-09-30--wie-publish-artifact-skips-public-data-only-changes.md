## [2026-09-30] 공개 데이터만 바뀐 머지도 엔진 릴리스를 만든다 (wie-publish-artifact-skips-public-data-only-changes)

**무엇을** — `publish-artifact.yml` 의 `on.push.paths` 에 공개 데이터 원천 3개를 넣었다:
`scripts/player-data.mjs` · `docs/player-data/**` · `docs/player-updates/**`.

**왜** — 릴리스는 wasm 과 함께 `compat.json`·`updates.json` 을 싣는데(「Build public data」 단계),
트리거는 엔진 입력만 보고 있었다. 그래서 #417(`67c3f5d8` · 빌더의 landedPin «+}» 수정)이 09-29 에
머지된 뒤로 publish run 이 **0** 이었고, 마지막 릴리스 `engine-6514e9af` 은 옛 빌더 산출이라
셸 수신(run 36593728310)에서 compat 35 · updates 18 건이 거부된 채 남아 있다.

**engine-contract.yml 필터에는 넣지 않았다(의도)** — 그 필터는 wasm 계약 검사를 켤지 정하는 것이고,
이 세 입력은 같은 잡의 상시 실행 `player-data.mjs --selftest` · `player-data.mjs` 단계가 이미 매 PR 검사한다.
넣으면 데이터만 바뀐 PR 마다 wasm 빌드 + 브라우저 왕복이 헛돈다. `docs/contracts/**` 는 빌더가 읽지 않아(실측) 트리거에 넣지 않았다.

**데이터만 바뀐 경우 wasm 재빌드 생략은 하지 않았다** — 릴리스 한 번 몇 분이고, 분기를 두면 wasm 없는 릴리스가 생긴다.

**사전 검증(머지 전 · 이 브랜치)** — `node scripts/player-data.mjs build` → otterpebble `origin/main` 의
`compat-import.mjs`·`updates-import.mjs` 에 스크래치 출력으로 통과: 429종 · 47건 · **둘 다 rc=0 · 거부 0**.

**사용자 영향** — 이 PR 이 머지되면(워크플로 파일 자신이 paths 에 있다) 새 릴리스가 나가고 /support 가 최신 수치로 바뀐다.
이후로는 업데이트 소식만 추가한 PR 도 사이트에 바로 닿는다.
