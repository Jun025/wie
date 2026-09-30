## [2026-09-29] playability-census 호스트 단일 실행 잠금 (wie-playability-census-host-single-instance-lock)

### 무엇을
- `scripts/playability-census.mjs` `run` 이 시작할 때 호스트 전역 잠금 `/tmp/wie-playability-census.lock`(고정 경로 · `mkdir` 원자 획득 · 안에 보유 pid)을 잡는다.
  이미 잡혀 있으면 stderr 1줄(`… holds this host — waiting`)을 남기고 2초 간격으로 **기다린다**(거절 아님). 보유 pid 가 죽었으면 회수한다(`reclaiming`). 종료 시 자기 pid 일 때만 푼다.
- selftest 4건 추가 — 실제 `run` 경로(빈 corpus)를 자식 프로세스로 띄워 잰다: 살아 있는 보유자 → 대기 후 해제되면 진행 · 죽은 보유자 → 회수 · 종료 시 잠금 해제.
- AGENTS.md census 절 1줄.

### 왜
wie#410(jobs 캡) 뒤에도 load1 528~592 재포화(총괄 2026-09-29 21:1x 실측). 원인은 스크래치 사본 두 곳(`~/tmp/wie-census-313ddcd8`·`-3c34efee`)에서 census 가 동시에 돌아 10+10 jobs(ncpu 10). 캡은 한 회차 안만 막는다. 잠금 경로를 스크립트 위치가 아니라 고정 경로로 둔 이유가 이것이다(사본끼리도 같은 잠금). `os.tmpdir()` 은 `$TMPDIR` 을 따라 세션마다 갈릴 수 있어 `/tmp` 로 고정했다(시험용 `WIE_CENSUS_LOCK` 만 덮어쓴다).

### 검증
- `node scripts/playability-census.mjs selftest` → **25/25** rc=0(약 3초).
- 개악 대조: `await hostLock(LOCK);` 줄 제거본 → **22/25 rc=1**(대기·회수·해제 3건 FAIL).

### 한계(코드에 `ponytail:` 주석)
- 죽은 보유자의 pid 가 재사용되면 살아 있는 것으로 읽혀 그 pid 가 끝날 때까지 기다린다. 두 대기자가 같은 낡은 잠금을 동시에 회수하면 둘 다 진행할 수 있다(= 종전 상태). pid+기동시각이 상향 경로.
- 부모가 SIGKILL 되고 `wie_validate` 자식이 살아남으면, 다음 회차는 부모 pid 가 죽었으니 회수하고 진행한다.

### 사용자 영향
없음(로컬 측정 도구). 다른 레인은 census 두 회차가 겹쳐 생기는 포화를 더 겪지 않는다.
