## [2026-10-02] 플레이어 소식 문구의 금지 구두점 제거 + 검사 (wie-player-updates-forbidden-punct-name-form-keys)

- **무엇을**: `docs/player-updates/2026-10-02-name-form-keys.json` 의 「위·아래 키로」→「위아래 키로」(otterpebble#1279 와 같은 문구). `scripts/player-data.mjs` `validateUpdates` 가 `summary_ko` 의 `—―–·・ㆍ‧∙•` 를 거부한다(집합 출처 otterpebble `scripts/user-copy-glyph-check.mjs` r2) · selftest 사례 1 추가 · 계약 문서 1줄.
- **왜**: 그 가운뎃점이 엔진 핀 범프로 otterpebble `apps/featurephone/lib/updates.json` 에 실려 otterpebble main 의 `user-copy-glyph` 래칫을 red 로 만들었다. otterpebble 쪽 가져오기는 이제 금지 구두점이 있으면 소식 파일째 거부하므로, 원문을 고치지 않으면 다음 핀 범프의 소식이 사이트에 못 들어간다.
- **사용자 영향**: 다음 핀 범프부터 업데이트 소식이 다시 사이트에 실린다. 새 소식에 같은 구두점이 들어오면 wie PR 의 `contract` 잡에서 먼저 막힌다.
- **실측**: 금지 구두점 grep 1→0 · `node scripts/player-data.mjs` rc=0 · 원문만 되돌리면 rc=1(위반 1건) · `--selftest` 26 규칙 통과.
- `compat.json`(`knownIssues_ko`·`changes`)도 같은 사이트로 가지만 현재 0건이라 검사를 넓히지 않았다 — 걸리면 그때 같은 정규식을 얹는다.
