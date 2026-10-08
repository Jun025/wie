## [2026-10-08] census 임대를 «스윕 전체»가 아니라 phase(≈15분) 단위로 (wie-census-drivers-lease-per-phase-not-per-driver)

**무엇을**
- `scripts/playability-census.mjs run --budget <secs>`: 프로세스 시작(census 잠금 대기 포함)부터 `<secs>` 가 지나면 새 게임을 더 내주지 않는다. 진행 중이던 게임은 끝까지 돈다. 남은 게임이 있으면 **exit 3** 으로 끝난다. `run` 은 원래 이어서 돌릴 수 있으므로(이미 있는 JSON 은 건너뛴다) 잘린 다음 호출이 그 자리부터 잇는다. ★게임별 측정과 판정은 바뀌지 않는다. 바뀌는 것은 «언제 재느냐»뿐이다.
- `scripts/census-drive.sh <run 인자…>`: 호출 1회마다 `build-slot run --long` 임대를 1건 잡는다(기본 `CENSUS_BUDGET=900`). exit 3 인 동안 다시 부르고, 호출은 최대 `CENSUS_MAX_CALLS=200` 회다. 호스트 부하 폴링은 넣지 않았다(임대 대기가 그 역할을 한다).
- `CLAUDE.md` «측정 스윕»: 「스윕 전체를 한 번의 `--long` 으로」 → 「phase 마다 임대 · 1건 ≤ 30분」. «ONE lease» 는 이제 **금지 예**로만 남는다. 「long 1」 → 「long 2」로 고쳤다. ⒞의 host-load-guard 대기를 «드라이버에 넣지 마라»로 바꿨다. «드라이버는 repo `scripts/` 에 둔다 · 스크래치 복제 금지» 규칙을 추가했다. `AGENTS.md` census 항목의 «one lease for the whole run» 도 같은 내용으로 고쳤다.
- 드라이버 원형: repo 에는 **없었다**. 스크래치에만 있었다(`drive*.sh` · 실행 중이던 `bash -c` 묶음들 — 전부 «한 임대 안에 phase 여러 개 + host-load-guard» 형상). 그래서 원형을 새로 만들었다.

**왜**: ops 실측(10-05~08)에서 long 임대 held_max 는 26974s·19861s·17942s 였다. 점유자는 census `run` 과 scratch 드라이버였다. 이 헌장 줄이 그 형상을 지시하고 있었다. 이 회차 중에도 `wie-3` 의 한 임대 스윕이 **10817s** 째 long 1칸을 쥐고 있었다. 그래서 이 회차의 첫 임대는 2181s 를 기다렸다.

**실측**(`~/orchestrator/state/build-slot/stats-2026-10-08.tsv` · long 행 · cwd `wie-2`): 수정된 드라이버를 썼다. 코퍼스는 `~/tmp/w5/corpus` 54개(git 밖)이고 `--jobs 2` · 개인 census 잠금이다. 기본 모드(probe → longplay)로 돌렸다.

| 호출 | 내용 | 대기s | 보유s |
|---|---|---|---|
| 1 | probe 30/54 | 2181 | **907** |
| 2 | probe 나머지 24 + longplay 시작 | 0 | **1322** |
| 3 | longplay | 1 | **1196** |
| 4 | longplay — 내가 중단(48×600s ≈ 4h 라 회차 범위 밖) | 1 | 292 |

⇒ 전건 ≤ 1800s 다. 대조군은 같은 날 같은 worktree 에서 돈 scratch `drive.sh` 1건이다: **16101s**.
상한: 호출 1건 = budget + 진행 중인 가장 긴 게임이다(longplay 600s → ≈1500s). progress 레시피의 `secs` 가 1800 을 넘는 게임은 그 1건만으로 30분을 넘는다. 이것은 쪼갤 수 없는 단위다.

**사용자 영향**: 없음(측정 도구·운영 규칙만 바뀌었다). 다른 레인의 long 대기가 줄어든다.

게이트: fmt · clippy · wasm clippy · `RUST_MIN_STACK` test 전부 rc=0 · beta clippy rc=0 · census `selftest` 62/62 · doc-liveness parity OK.
