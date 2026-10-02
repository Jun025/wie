## [2026-10-02] 측정 스윕 동시 실행 상한 — build-slot 경유 + census 기본 `--jobs` ≤ 3 (wie-heavy-measure-sweeps-must-go-through-build-slot)

**무엇을**: repo `CLAUDE.md` 「에이전트 개발환경」에 «측정·재생 스윕도 build-slot 경유 또는 동시 ≤ 3 · `nohup … &` 금지 · 착수 전 `host-load-guard --status --recovered` rc=0» 조항을 넣고(otterpebble #1249 문안·구조 그대로), `scripts/playability-census.mjs` 의 기본 `--jobs` 를 `ncpu/2`(10코어 = 5) → `min(3, ncpu/2)` 로 낮췄다. `AGENTS.md` 의 census 기본값 서술도 맞췄다.
**왜**: 2026-10-02 09:5x 총괄 실측 — 회차들이 손으로 띄운 `wv_main`/`wie_validate` 20개(합 223% CPU)로 load1 238~242 · idle 0%. 그 스윕은 build-slot «밖»이었고, 부하 가드는 신규 배차만 막아 못 잡았다.
**사용자 영향**: 없음(개발 도구·규칙만 · 측정값·판정 로직 무변경 — 동시 실행 수만).

### 진입점 목록 (에뮬레이터를 띄우는 것)

| 진입점 | 병렬 방식 | 처분 |
|---|---|---|
| `scripts/playability-census.mjs` | `pool(jobs)` · 호스트 잠금 | 기본 `--jobs` 5 → **3**(`--only progress` 기본은 ncpu/4 = 2 그대로) |
| `scripts/smoke_gate.sh` · `scripts/game-lab-recensus.sh` · `scripts/lgt_render_probe.sh` | 한 번에 1개(`&` 는 타임아웃 감시용 · 곧바로 `wait`) | 무변경 |
| `scripts/audio-probe.mjs` | 빌드 × 게임 쌍을 동시에(한 부하 구간 공유가 설계) | 무변경 — 쌍 수는 호출자가 정한다. 조항 ⒜가 덮는다 |
| 워커가 손으로 띄우는 `wv_* --inject` 루프 | repo 밖(스크래치) | 조항 ⒜⒝⒞ |

### 검증

- `node scripts/playability-census.mjs selftest` → 37/37(기본값 단언을 `=== 3`·4코어 `=== 2` 로 바꿨다).
- 상한 위반 시도: 격리 상태(`ORCH_BUILD_SLOT_STATE`=임시 · 슬롯 3)에서 `build-slot run -- sleep 4` 4개 동시 → status `held=3 waiting=1` · 4번째가 「슬롯 3개 만석 — 대기」 후 실행.
