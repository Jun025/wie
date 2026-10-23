## [2026-10-23] achieved-speed 상한 문구 제안 — otterpebble 이행분 채택 기록 (wie-worklog-record-cross-repo-adoption-achieved-speed-p0)

**무엇을**: `2026-10-22-achieved-speed-host-slowness#p0`(셸 «약 N배» 문구를 상한으로)을
`docs/worklog/2026-10-23-record-cross-repo-adoption-achieved-speed-p0.json` 의 `adoptedProposals` 로 닫았다.
이행은 otterpebble PR #1360(머지 `9545910c` · 2026-10-23)이다. `AGENTS.md` §Proposal threshold 에 이 방향
(wie 제안 → 다른 저장소 이행)의 기록 자리를 한 줄 더했다.

**왜**: tower `~/tower/bin/cockpitd.js` 의 `collectProposals` 는 저장소마다 `scanRepoSimple` 을 돌려 그
저장소 안의 `adoptedProposals` 로만 `settle` 한다 — otterpebble 의 worklog 는 wie ref 를 닫을 수 없다.
otterpebble `.claude/rules/worklog.md` §교차 레포 채택은 반대 방향(otterpebble 원장이 밖에서 이행된 것을
기록)만 다룬다.

**사용자 영향**: 없음(문서·원장만). tower 「추천 후속 작업」의 접수됨(pending)에서 이 제안 1건이 빠진다 —
착지 전 실측 `GET /api/proposals` 에서 `pending` 에 `wie 2026-10-22-achieved-speed-host-slowness#p0` 이 있었다.
