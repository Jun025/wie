## [2026-10-05] 측정 스윕은 `build-slot run --long` — 스윕 하나 = long 임대 하나, 안쪽 병렬은 그 안에서 (otterpebble-wie-sweeps-use-build-slot-long-pool)

**무엇을**: `CLAUDE.md` 「측정 스윕」⒜ · `AGENTS.md` census 실행 줄 · `scripts/playability-census.mjs` `--jobs` 주석. 스윕(census `run` · 짝 재측 묶음 · 수 분 이상 장주행)은 **스윕 전체를 한 번의 `build-slot run --long`** 으로 감싼다. 러너 안의 실행은 맨 명령이다. 짧은 빌드·테스트와 단발 확인 1회는 종전대로 short(`build-slot run --`)다.

**왜**: orchestrator PR #1285(머지 `6dffb87b`)가 빌드 슬롯을 short 3 · long 1 의 2급으로 나눴다. 근거는 7일 대기의 53% 가 ≥10분이고, 나쁜 시간대의 점유자가 전부 wie 스윕이었다는 것이다. 그런데 이 repo 헌장은 스윕을 맨 `build-slot run --`(= short)로 보냈다 ⇒ 그 PR 이 착지해도 행동 변화가 0 이었다(게이트② major 1).

**사용자 영향**: 없다(개발 규율).

### 1. 호출부 전수
`git grep build-slot`(코드·헌장 · `docs/report` 의 서술 제외):

| 자리 | 전 | 후 |
|---|---|---|
| `CLAUDE.md` 빌드·테스트 줄 | `build-slot run --` | 그대로(short) |
| `CLAUDE.md` 측정 스윕 ⒜ | «한 번에 하나씩 `build-slot run --`» | 스윕 전체를 `build-slot run --long --` 한 번 · 안쪽은 맨 명령 · 단발 1회만 short |
| `AGENTS.md` census 줄 | `build-slot run -- node … census run` | `build-slot run --long -- …` |
| `scripts/playability-census.mjs` | 주석만(«3 = 빌드 슬롯 수») | 주석: short 슬롯 수 · run 은 long 임대 하나 · jobs 는 그 안 |

- 러너가 `build-slot` 을 «직접» 부르는 곳은 0 이다. census 는 바깥에서 감싼다.
- 이 repo 의 `wie_validate` DoD 러너 블록(픽스처 5~20초)은 build-slot 을 쓰지 않는다 — 바꾸지 않았다.

### 2. 처리량 판정 — long 1 유지 · 스윕 내부 병렬은 한 임대 안에서

| 형태 | 실측 | long=1 에서 |
|---|---|---|
| census `run` | 6차(`docs/report/0432` · `~/scratch/w6census/progress.log`): 회차마다 **임대 1개 · `--jobs 3`** — 주 단계 7h21 · 6h26 · 6h27 · 2h30, 진도 4h37, P2 3h20 | **직렬화 0**. 이미 한 임대 안에서 3병렬이다. census 둘은 호스트 잠금(`/tmp/wie-playability-census.lock`)이 원래 겹치지 못하게 한다 |
| 손 스윕(실행마다 감쌈) | `state/build-slot/stats-2026-10-05.tsv` 7열 행 2.0h 창: 스윕 임대 **226회** · 보유 중앙 30s · p90 60s · 합 **125분** · 동시 1/2/3 = 14/28/18분 | 실행마다 `--long` 이면 long 1칸에 줄을 선다: 합 125분이 직렬 ⇒ 관측된 «스윕이 돈 시간» 60분의 **약 2.1배** |

⇒ 고른 것: **«스윕 내부 병렬은 한 임대 안에서 유지» + long 1 유지**.
- 실행마다 따로 임대하면 2.1배 느려진다. 한 임대 안에 ≤3 을 두면 지금 속도 그대로 short 풀만 비워진다.
- 비용: 다른 레인의 스윕이 동시에 오면 long 칸을 기다린다. build-slot 은 30분 상한 뒤 «그냥 실행»하므로 최악은 스윕 시작 30분 지연이다.
- ★7열(보유초) 기록은 PR #1285 이후 2시간분뿐이다. 동시 스윕 레인 수는 이 표본으로 판정할 수 없다. long 값 재판정은 회신의 제안으로 남긴다(값은 orchestrator `contracts/host-load-gate.conf` 소관).

### 3. 검증
- 라이브 `~/orchestrator-live/bin/build-slot run --long -- true` → rc=0 · `status` 에 `long=1` 이 보인다.
- 문서·주석만 바꿨다. 네 게이트는 돌렸다(회신 참조).
