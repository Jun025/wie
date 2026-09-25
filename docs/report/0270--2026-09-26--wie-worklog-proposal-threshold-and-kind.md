## [2026-09-26] 후속 제안 입구 좁히기 — 문턱·`kind`·상한 2 (wie-worklog-proposal-threshold-and-kind)

**무엇을**
- `AGENTS.md` §Worklog schema 에 `proposals[].kind`(`product`|`meta`, 선택) 행 1줄 + §Proposal threshold 신설(본문 13줄).
  otterpebble 추천도 3문은 요지 + 경로(`otterpebble/.claude/rules/autonomy.md` §제안 등재 문턱)만 옮겼다.
- `scripts/check-worklog-json.mjs`: ⒜`kind` 값 검증(전 파일 — 현재 `kind` 보유 파일 0이라 소급 red 0)
  ⒝origin/main merge-base 대비 **추가된** worklog, 또는 **제안 수가 늘어난** 변경 worklog 에서 제안 3개 이상이면 red.
  ⒞`--selftest` 7케이스 — 각 케이스가 임시 git 저장소에 대해 **이 스크립트를 CLI 로 실행**해 rc 를 단언한다.
- `engine-contract.yml` 기존 스텝에 `git fetch origin main` + `--selftest` 추가(새 잡 0).

**왜** — 09-14~24 채택 1건당 새 제안 1.40(09-24 배치 1.45). 병합 PR 123건 중 38건이 제안 채택·기각만 했다.

**설계 판단**
- 「변경된 과거 파일」을 전부 재면 기존 3개 이상 worklog 41건(235건 중) 에 `declinedProposals` 만 더해도 red 가 된다.
  그래서 변경 파일은 **제안 수가 base 보다 늘었을 때만** 잰다(selftest 의 disposition-only 케이스가 이를 고정한다).
- origin/main 이 없으면 ⒝를 건너뛰고 성공 줄에 `★proposal cap NOT measured` 라고 적는다 — 조용한 통과 금지.
- `resolved-by #<PR>` 사유는 ref 에 붙이지 않는다(소비자가 ref 를 그대로 대조하므로 붙이면 기각이 먹지 않는다) —
  자유형 `declinedReasons` 객체에 둔다.

**판별력** — `MAX_PROPOSALS=99` 변이 → selftest 2건 FAIL · `kind` 검사 제거 변이 → 2건 FAIL.

**사용자 영향** — 없음(엔진·웹 무접촉). 추천 패널에 새로 올라오는 카드 수가 worklog 당 2개 이하로 묶인다.
