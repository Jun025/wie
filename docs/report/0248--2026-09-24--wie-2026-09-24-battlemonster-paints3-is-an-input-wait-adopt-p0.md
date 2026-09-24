## [2026-09-24] wie_validate: --inject-keys · --keep-timeout · --shot-every (wie-2026-09-24-battlemonster-paints3-is-an-input-wait-adopt-p0)

### 무엇을
`wie_validate` 에 opt-in 플래그 3개. `--inject-keys N` = 주입 스크립트 앞 N키만 · `--keep-timeout` = 데드라인을 스케줄 대신
`--timeout` 으로 · `--shot-every S` = S초마다 `--shotdir` 프레임(`<stem>__tNNN.N.png`). 스케줄 조립을 `plan_schedule()` 로 떼어
단위 테스트 3건으로 잠갔다(기본값 불변 · 새 플래그 · 짝 플래그 누락 거부). 0233 의 13줄 스크래치 패치(`WIE_*` env)를 정식화한 것.

### 왜
`--inject` 는 27키 전체와 스케줄 데드라인을 강제해 무주입 기준과 같은 예산으로 짝지을 수 없었다. 0233 은 그 판정에 스크래치 패치 +
release 재링크 약 20분을 썼다.

### 실측
- 반증 ⒜: `origin/main` `2e356dfc` 에 플래그·env 없음(`WIE_*` 는 0233·worklog 산문에만).
- ⑴ LGT 배틀몬스터, `--inject --inject-keys {0|1} --keep-timeout --boot-secs 20 --timeout 60 --max-ticks 100000000000 --shot-every 5`, 동시 기동:
  N=0 → paints **3** · `ms` 60052 · t≥20 프레임 8장 전부 `411c7503`(0233 과 같은 해시) · ERROR 0 /
  N=1 → paints **362** · `ms` 60036 · t≥20 프레임 8장 전부 다름 · ERROR 0. 플래그만으로 0233 판정(입력 대기)이 재현된다.
- ⑵ 러너 5줄, origin/main 빌드 ↔ 이 브랜치 빌드(둘 다 release · 동시): 타이밍 필드(ticks·paints·ms·richness) 제외 **동일**,
  유일한 차이는 keydraw_ktf `input_steps` 25 ↔ 26. 두 빌드 모두 keydraw_* 가 UNMEASURED(max-ticks) — release 빌드가 5천만 tick
  백스톱을 20초 안에 태우는 기존 현상이고 이 변경과 무관(구 빌드에서도 같다).
- 게이트: fmt OK · clippy(stable/wasm/beta) `-D warnings` OK · `cargo test --all` 46 suites 435 passed 0 failed ·
  `cargo clippy --workspace --all-targets` 경고 16 → 16.

### 사용자 영향
없음(검증 도구 전용 · 기본 동작 불변). 다음 «paints 3» 타이틀은 명령 두 줄로 입력 대기/벽을 가른다.

### 한계
`--inject-keys 0` 은 `input_steps_total 0` 이라 UNMEASURED 게이트(`input_steps < input_steps_total`)에 걸리지 않고 PASS 로 나온다 —
0키 실행의 판정은 입력에 대해 아무것도 말하지 않는다(AGENTS.md 에 적음).
판별은 **release 빌드**로만 선다 — debug 는 같은 60 s 안에 판별에 필요한 게스트 진행에 닿지 못한다(게이트② 실측: debug N=0·N=1 둘 다 paints 3 · `--timeout 180` 에도 동일 ↔ release 3 vs 82). AGENTS.md 문단에 release 명령을 적었다.
