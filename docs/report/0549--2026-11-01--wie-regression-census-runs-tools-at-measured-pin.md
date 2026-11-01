## [2026-11-01] 주간 퇴행 감시가 판정 도구를 측정 핀 트리에서 돌린다 — 도구 ↔ 기준 핀 어긋남 위양성 0 (wie-regression-census-runs-tools-at-measured-pin)

**무엇을**: `scripts/regression-census.sh` 를 고쳤다. 이제 `census-drive.sh`(→ `playability-census.mjs run`)·`playability-census.mjs report`·`regression-diff.mjs` 를 `$BUILD/scripts/` 에서 부른다. `$BUILD` 는 바이너리를 빌드한 worktree 이고, 회차마다 `$PIN` 으로 다시 checkout 한다(이어 돌리는 회차도 같다). 그 뒤 `$BUILD` 의 HEAD 가 `$PIN` 이 아니거나 `scripts/` 에 로컬 수정이 있으면 `WIE_REGRESSION_INCOMPLETE stage=tool-pin` 으로 멈춘다. 처방 ⒜와 ⒝를 둘 다 넣었다.

**왜**: 2026-11-01 회차의 `SUSPECT n=4` 는 4건 모두 측정 결함이었다(`~/orchestrator/reports/wie-weekly-regression-census-20261101-suspect-triage.done.md`). 바이너리와 기준 compat 는 `1ca15ebf` 에서 왔다. 판정 도구는 레인 checkout 의 `5ceff33f` 에서 돌았다. 그 사이의 #580·#581 이 `HAND_WALL` 에서 뺀 4개 sha 를, 낡은 도구가 «벽 → limited» 로 판정했다. ⒜만 있으면 «도구·기준·바이너리가 한 핀»이 구조로 선다. ⒝는 그 checkout 이 실패한 경우를 맡는다(알 수 없는 sha · 충돌하는 로컬 수정 · 트리 없음). 이때 낡은 도구로 조용히 판정하지 않고 멈춘다.

**사용자 영향**: 없다. compat.json 은 고치지 않았다. 주간 원장 줄에 위양성 `WIE_REGRESSION_SUSPECT` 가 사라진다.

### 재생 (샌드박스 · 측정 0회)
2026-11-01 회차 디렉터리를 APFS 복제로 떴다. 거기에 triage 재측 ②의 L 결과(같은 바이너리 `1ca15ebf` · 4건)를 넣었다. 회차의 `wie_validate` 는 호출을 기록하고 exit 1 하는 스텁으로 바꿨다. 네 경우 모두 스텁 호출은 **0** 이었다(전부 디스크에 있는 결과의 재판정이다). HOME·원장·증적·`$BUILD` 는 샌드박스였고, 실제 원장과 `~/worktrees/wie-regression-build` 는 손대지 않았다.

| 경우 | 레인 트리 도구 | `$BUILD` 시작 HEAD | 결과 |
|---|---|---|---|
| 대조(고치기 전 스크립트) | `5ceff33f` | — | `WIE_REGRESSION_SUSPECT n=4` · 도구 핀 `5ceff33f` — 2026-11-01 원 회차를 그대로 재현 |
| 고친 스크립트 · 핀 어긋남 개악 | `5ceff33f` | `5ceff33f` | `WIE_REGRESSION_CLEAR` · 의심 0 · 후보 1(`44a4228cca51`, 짝 불일치) · 도구 핀 = `1ca15ebf` |
| 고친 스크립트 · 받을 수 없는 핀(`deadbeef…`) | `5ceff33f` | `5ceff33f` | `WIE_REGRESSION_INCOMPLETE stage=tool-pin` · rc 1 |
| 고친 스크립트 · `$BUILD/scripts` 로컬 수정 | `5ceff33f` | `5ceff33f` + 수정 | `WIE_REGRESSION_INCOMPLETE stage=tool-pin` (sha 만 보면 통과했다 — 수정이 checkout 을 넘어간다) |

`bash scripts/regression-census.sh selftest` → `selftest: 21/21`.

### 남는 것
- `scripts/perf-watch.sh` 는 여전히 레인 checkout 에서 돈다. 그것은 A/B 두 핀을 `git archive` 로 따로 빌드하므로, 판정 표가 엔진 핀을 따라 움직이는 이 결함은 없다. 범위 밖이다.
- 드라이버 `regression-census.sh` 자신은 레인 checkout 의 것이다(launchd 래퍼가 그 경로를 exec 한다). 그래서 이 수정은 레인 checkout 이 이 커밋 이후일 때부터 효력이 있다.
