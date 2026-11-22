## [2026-11-22] 네이티브 릴리스 빌드에서 `log` 의 trace 수준을 컴파일 단계에서 뺀다 — ARM 명령마다 돌던 수준 검사 2회 (wie-native-release-build-compile-out-arm32-cpu-trace)

**무엇을**: `wie_cli/Cargo.toml` 에 `log = { features = ["release_max_level_debug"] }` 한 줄(네이티브 타깃 전용)과 그 줄을
잠그는 시험 1건. 릴리스로 빌드한 `wie_cli` · `wie_validate` 에서 `log` 크레이트의 `trace!` 가 컴파일 단계에서 사라진다.
해석기 코드 · `arm32_cpu` · wasm 빌드 설정은 건드리지 않았다.

**왜**: `arm32_cpu` 는 게스트 명령 하나마다 `trace!` 를 2번 부르고, 로그를 꺼 두어도 그때마다 수준을 검사한다. 네이티브
프로파일에서 ARM 실행 루프 표본의 23.1% 가 그 검사였다(0559 §4). wasm 쪽은 #592 가 이미 뺐다. 운영자 채택 제안
`2026-11-13-heavy-scene-frame-cost-cheap-wins#p0`.

**사용자 영향**: 브라우저(피처폰 셸)로 하는 사람에게는 없다 — wasm 산출물이 같다(§6). 네이티브 실행기와 검증 러너에서
해석기가 바쁜 장면(LGT `3cc7a9b4cb15`)의 CPU 비용이 50M 틱당 **중앙값 −11.5%**(−16.0 · −11.4 · −11.5). 타이머를 기다리며
쉬는 시간이 대부분인 판은 그대로다(±3% 안). 전수 조사 · 퇴행 감시의 판정은 표본 10판에서 바뀌지 않았다(§5).

### 1. 바꾼 것 — `Cargo.*` 범위

| 파일 | 변경 |
|---|---|
| `wie_cli/Cargo.toml` | `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` 절 + `log` 한 줄(+8) |
| `Cargo.lock` | `wie_cli` 의 의존 목록에 `"log"` 한 줄(+1) — 버전 · 체크섬 변화 0 |
| `wie_cli/tests/log_static_level.rs` | 시험 1건(새 파일) |

- 수단은 wasm 과 같다(`log` 의 `release_max_level_*`). wasm 은 `…_off`(로거가 없다), 네이티브는 `…_debug`(로거가 있고
  debug · info 는 남겨야 한다).
- ★네이티브 타깃 전용 절에 둔 이유: `log` 는 `release_max_level_*` 가 둘 켜지면 `compile_error!` 를 낸다
  (`log` 0.4.34 `lib.rs:382~399` · `debug_assertions` 와 무관). 타깃 조건 없이 넣었더니
  `cargo tree --workspace --target wasm32-unknown-unknown -i log -e features` 에 `…_debug` 와 `…_off` 가 함께 나왔다 —
  `wie_cli` 와 `wie_featurephone` 을 같이 고르는 wasm 빌드가 깨진다. CI 의 wasm 린트는 `wie_cli` 를 고르지 않아 초록인 채로
  남았을 자리다. 타깃 조건을 단 뒤 같은 명령에 `…_off` 만 나온다.

### 2. 빠지는 것은 `log` 의 trace 뿐 — 확인 3건

| 확인 | 근거 |
|---|---|
| 디버그 빌드는 그대로 | `cargo test --all`(디버그)의 `release_builds_compile_out_log_trace_only` 가 `log::STATIC_MAX_LEVEL == Trace` 를 단언 · 디버그 `wie_validate` 에 `RUST_LOG=trace` 를 주면 `arm32_cpu` 의 명령 추적 줄이 9,836줄 나온다(main 릴리스와 같은 수 · 명령 4,918개 × 2줄) |
| wie 자신의 `tracing` 로그는 무관 | `tracing` 은 `log` 를 거치지 않는다(`cargo tree -p wie_cli -e features` 에 `tracing feature "log"` 0건) · 릴리스 `wie_validate` 에 `RUST_LOG=trace` 를 주면 `arm32_cpu` 추적 줄만 9,836 → 0 이 되고 wie 자신의 TRACE 12,668줄 · DEBUG 1,084줄 · INFO 11줄은 main 릴리스와 같은 수로 남는다 |
| debug · info 는 릴리스에도 남는다 | 릴리스 시험(`cargo test --release -p wie_cli --test log_static_level`)이 `STATIC_MAX_LEVEL == Debug` 를 단언 · 릴리스 바이너리 안의 `arm32_cpu` 포맷 문자열: trace 2종(`THM: pc` · `cond not met`) 1 → 0, info 2종 · error 1종은 1 → 1 |

런타임 수는 커밋된 `test_data/helloworld_ktf.zip` 에 `RUST_LOG=trace --timeout 2` 를 준 stderr 의 수준별 줄 수다:

| 빌드 | `arm32_cpu` trace | wie 자신의 TRACE | DEBUG | INFO |
|---|---|---|---|---|
| main 릴리스 | 9,836 | 12,668 | 1,084 | 11 |
| 이 브랜치 릴리스 | 0 | 12,668 | 1,084 | 11 |
| 이 브랜치 디버그 | 9,836 | 12,668 | 1,084 | 11 |

릴리스 `wie_validate` 는 31,524,560 → 31,490,016 B(−34,544).

같이 빠지는 것: `log` 로 찍는 다른 크레이트(`gdbstub` · `midir` · `symphonia-*` · `zopfli`)의 trace 도 릴리스에서 사라진다.

### 3. 효과 — `perf-watch.sh compare`(네이티브 축 · 50M 틱당 CPU ms · A 와 B 를 같은 분에 · 3회)

`WIE_PERF_AXES=native bash scripts/perf-watch.sh compare <main 빌드> <이 브랜치 빌드> <out>`. A = `origin/main` `b6ef84444`,
B = 이 브랜치(같은 명령 `cargo build --release -p wie_cli --bin wie_validate`). load1 124~341. 벽시계가 아니라 CPU 시간인
이유는 그 도구 머리말(부하에서 벽시계는 2.5배 퍼지고 CPU 시간은 ±3%)이다.

| 표본 | 플랫폼 | A(3회) | B(3회) | B/A − 1 (%) | 중앙값 |
|---|---|---|---|---|---|
| `3cc7a9b4cb15` | LGT · 해석기가 바쁜 장면 | 67,606 · 159,976 · 72,507 | 56,816 · 141,765 · 64,182 | −16.0 · −11.4 · −11.5 | **−11.5** |
| `155972cac664` | KTF | 25,812 · 48,274 · 37,644 | 25,143 · 51,936 · 32,141 | −2.6 · +7.6 · −14.6 | −2.6 |
| `1065985081fc` | KTF | 13,993 · 52,957 · 17,664 | 13,923 · 50,825 · 18,003 | −0.5 · −4.0 · +1.9 | −0.5 |
| `0606d43702e7` | LGT | 11,160 · 14,279 · 12,718 | 11,220 · 14,208 · 12,666 | +0.5 · −0.5 · −0.4 | −0.4 |
| `835acd535a5f` | J2ME(대조) | 1,702,801 · 1,814,589 · 1,464,539 | 1,688,596 · 4,302,500 · 1,236,015 | −0.8 · +137.1 · −15.6 | −0.8 |
| `8238f5449347` | J2ME(대조) | 7,440 · 7,681 · 7,775 | 7,409 · 7,665 · 7,788 | −0.4 · −0.2 · +0.2 | −0.2 |
| `d1234e0ddb75` | SKT(대조) | 218,181,818 · 215,853,659 · 225,233,645 | 214,721,845 · 225,190,840 · 235,376,045 | −1.6 · +4.3 · +4.5 | +4.3 |

도구 판정: 퇴행 0 · 1차 경보 0 · 미측정 0/7 · 편차에 묻힘 0.

- 제안이 적은 «최대 약 23%»보다 작다. 23.1% 는 «ARM 실행 루프 표본» 대비였고 그 루프는 그 프로파일 전체의 39.1% 였다
  (0559 §4) — 틱당 CPU 는 러너가 타이머를 기다리며 헛도는 몫까지 센다. 해석기가 틱 예산을 채우는 `3cc7a9b4cb15` 만 두
  자릿수로 움직였고, 나머지 ARM 3판은 잡음 안이다.
- 대조 3판(J2ME · SKT)은 `arm32_cpu` 를 타지 않는다(`cargo tree -i arm32_cpu` 의 뿌리는 `wie-ktf` · `wie-lgt` 뿐) — 이
  변경으로 움직일 이유가 없는 판이라 그 수는 이 측정의 잡음 바닥으로 읽는다. SKT 의 +4.3 은 도구 문턱(5.7) 아래이고,
  J2ME `835acd535a5f` 의 +137 한 회는 load1 267 에서 난 한 쌍이다.

### 4. `perf-watch` 의 기준 — 갱신할 것 없음

기준은 저장된 수가 아니라 «지난 회차가 잰 핀»이다(`perf-watch.sh`: A = 마지막으로 판정이 난 회차의 `b-pin` · 지금
`2e66f9d6`). 도구는 «느려짐»만 경보하므로, 이 변경이 착지한 뒤 첫 주간 비교는 `3cc7a9b4cb15` 의 네이티브 축을 약 −11% 로
읽고 `WIE_PERF_CLEAR` 를 낸 다음 그 회차의 `origin/main` 을 다음 기준으로 삼는다. 손댈 파일이 없다.
그대로 두면 생기는 일은 하나다: **그 한 회차에 한해**, 같은 주에 들어온 네이티브 퇴행이 이 이득만큼(해석기가 바쁜
표본에서 약 11% 까지) 상쇄돼 안 보일 수 있다. 다음 회차부터는 새 핀이 기준이라 감도가 돌아온다. `compare` 로 이 커밋을
가로질러 이분 탐색하면 여기서 한 계단이 보인다 — 그 계단이 이 회차다.

### 5. 전수 조사 · 퇴행 판정의 시간 축 — 벽시계다 · 표본 10판에서 판정 변화 0

- `wie_validate` 의 종료 조건은 `elapsed > deadline || ticks >= max_ticks`(`wie_validate.rs:1476`)이고 게스트 시계도
  `SystemTime::now()`(`:638`) · 키 일정도 벽시계다. `playability-census.mjs` 는 모든 실행에 `--timeout <초>` 와
  `--max-ticks 100000000000`(틱 상한을 사실상 끔)을 준다 — 탐침 30초 · longplay 600초 · progress 전부. `short` 는 «600초
  전에 `deadline` 아닌 사유로 끝남»(`endedEarly`)이다. 즉 **시간 판정은 전부 벽시계**이고 틱은 판정에 쓰이지 않는다.
- 틱 상한: 표본에서 600초에 가장 많이 돈 판이 461,590,283틱 — 상한 1e11 의 0.46%. 빨라져도 닿지 않는다.
- 빨라진 빌드가 벽시계 창에서 바꾸는 것: 해석기가 바쁜 판이 같은 초 안에 더 멀리 간다(덜 바쁜 호스트에서 돌린 것과 같은
  방향). 타이머 박자로 도는 판은 그대로다.

표본 10판 = KTF 5 · LGT 5(`arm32_cpu` 를 타는 플랫폼만): `perf-watch` 의 ARM 표본 4판 + `compat.json` 의 `playable` 행 중
sha256 오름차순 KTF 3 · LGT 3. `bash scripts/census-drive.sh --bin <빌드> --out <dir> <표본 10판>`(탐침 30초 ×2 ·
longplay 600초 · `--jobs` 3)을 main 빌드(11:16~11:51) → 이 브랜치 빌드(11:51~12:26)로 잇달아 돌렸다.

| | main 빌드 | 이 브랜치 빌드 |
|---|---|---|
| `status` | playable 9 · limited 1 | playable 9 · limited 1 (같은 판) |
| boot · render · input · longplay · sound (50칸) | — | 50칸 모두 같음 |
| longplay `short` | `0093012b8c36`(게스트가 스스로 끝냄) | 같은 판 · 같은 사유 |
| speed | ok 10 | ok 7 · n/a 3 → 도구의 재측(`--only speed`) 뒤 ok 8 · n/a 2 |

speed 가 갈린 것은 이 변경이 아니라 호스트 부하다:
- speed 는 «늦음 합 ÷ 창» 이라 부하가 더하기만 한다(그 도구의 주석). main 탐침은 load1 101~105, 이 브랜치 탐침은
  121~178 · 재측은 297~355 에서 돌았다. 빠른 빌드가 같은 30초 탐침에서 10판 중 9판이 main 의 0.28~0.93배만 틱했다 —
  그만큼 굶었다.
- 남은 2판을 **main 과 이 브랜치를 같은 순간에** 3회씩 다시 쟀다(전수 조사의 speed 탐침과 같은 인자 · load1 140~149):

  | 판 | main(늦음 비 · 0.9 이상이 ok) | 이 브랜치 |
  |---|---|---|
  | `3cc7a9b4cb15` | 0.477 · 0.433 · 0.340 | 0.517 · 0.471 · 0.320 |
  | `0392263fbb85` | 0.720 · 0.640 · 0.650 | 0.696 · 0.663 · 0.596 |

  main 빌드도 지금 부하에서는 `n/a` 다. 두 빌드의 차(중앙값 +0.04 · +0.01)는 회차 간 흔들림 안이다.
- speed 는 `status` 에 들어가지 않는다(`playable` 판정은 boot · render · input · longplay).

크기 비교: 같은 판의 600초 틱 수가 두 census 사이에 0.38~1.65배로 흔들렸다(부하). 이 변경이 옮기는 폭(최대 16%)은 그
아래다. 원리상 남는 것은 «빨라진 빌드가 600초 안에 전에는 닿지 못한 지점의 오류에 닿는» 경우인데 표본 10판에서는 0건이다.

### 6. wasm 산출물 — 바이트가 바뀔 입력이 없다

- 이 PR 의 diff 는 `wie_cli/Cargo.toml` · `wie_cli/tests/` · `Cargo.lock`(`wie_cli` 줄 1) · 문서뿐이고, `wie_cli` 는 wasm
  빌드 그래프에 없다(`cargo tree --target wasm32-unknown-unknown -p wie_featurephone` 에 `wie_cli` 0건).
- 발행 경로(`scripts/build-wasm.sh` = `cargo build --target wasm32-unknown-unknown --release -p wie_featurephone`)의 유닛
  그래프(`--unit-graph` · 207 유닛 · 패키지 · 버전 · feature · 프로파일)가 `origin/main` 과 이 브랜치에서 경로 정규화 뒤
  바이트 동일하다. `log` 유닛의 feature 는 양쪽 다 `["release_max_level_off"]`.
- wasm 을 두 번 빌드해 바이트를 대조하지는 않았다 — 위는 «입력이 같다»는 증명이다.
- 착지하면 `publish-artifact.yml` 은 뜬다(`**/Cargo.toml` · `Cargo.lock` · `**/*.rs` 가 경로 필터에 있다): 새 Release 와
  otterpebble 핀 범프가 생기고 그 안의 엔진 바이트는 같다. `compat.json` 변화 0.

### 7. 한계

- 호스트 1대(M1 Max) · 회차 내내 load1 100~355 · idle 0%. 속도 수는 CPU 시간 · 같은 분 짝으로 쟀고 벽시계 수는 쓰지 않았다.
- 프로파일(xctrace)로 «그 두 검사의 표본 비율»이 0 이 됐는지는 다시 재지 않았다 — 바이너리에서 문자열이 사라진 것과
  틱당 CPU 로 대신했다.
- 루트 패키지 `wie`(upstream 의 실행기 · `cargo build` 기본 멤버)는 `wie_cli` 에 기대지 않아 이 feature 를 받지 않는다.
  티켓 범위(`wie_cli` · `wie_validate`) 밖이라 두었다.
- `docs/player-updates/` 항목은 쓰지 않았다 — 피처폰 사이트에서 판이 도는 방식은 바뀌지 않는다.

<!-- corpus-name-inflow v1 subjects=5 tree=c54f12dd658b14d6 B=0/0 P=0/0 S=0/0 -->
