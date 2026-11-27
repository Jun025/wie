## [2026-11-27] AOT 기본 켬(#645) 뒤 남은 두 자리 — 재센서스를 해석기에 고정 · `--profile-out` 문서에 엔진을 적는다 (wie-aot-default-on-followup-recensus-pin-off-and-profile-doc)

**무엇을**: ⑴ `scripts/game-lab-recensus.sh` 가 `wie_validate` 를 `WIE_AOT=off`(해석기)로 띄운다 — `WIE_AOT` 를 주지 않으면 끔이고, `WIE_AOT=native` 를 주면 켠 팔을
`reports-<날짜>-aot/` 로 잰다. 첫 출력 줄이 엔진을 적고, 그 밖의 값은 게임을 돌리기 전에 exit 2 로 거절한다. ⑵ `wie_validate --profile-out` 의 플래그 문서에 «어느 엔진으로
도는가 · 두 엔진에서 표본이 뜻하는 바»를 적었다 — 실제 두 판으로 재서 적었다(§2). ⑶ `Cargo.toml` 의 주석 «under `WIE_AOT=native`» 를 기본값이 켬인 지금의 말로 고쳤다.

**왜**: PR #645 검수(게이트② · 2026-11-27)의 비차단 지적 1 · 2 · 3. #645 는 «저장된 기준선이 해석기로 잰 수면 끔으로 고정한다»는 기준으로 도구 넷을 고정했는데, 7월 기준선
`game_lab/reports/`(해석기로 잰 것 — 그때는 엔진이 하나였다) 옆에 새 열을 놓고 견주는 재센서스가 그 목록에서 빠져 기본값(켬)을 따르고 있었다.

**사용자 영향**: 없다. 로컬 전용 스크립트 하나의 기본 엔진과 문서다. 제품 기본값(`aot: None`) · 브라우저 · wasm 산출물 · export 표면 · `compat.json` 무변경 · 고지 0건 ·
게임 파일 무수정 · `Cargo.lock` 무변경 · 새 의존 0. `wie_validate` 의 동작은 바뀌지 않았다(`--help` 의 글만 는다). **착지하면 릴리스는 한 번 선다** — `*.rs`(플래그 문서)와
`Cargo.toml`(주석) 둘 다 `publish-artifact.yml` 의 경로 필터에 든다(§3).

### 1. 재센서스 — 해석기에 고정

| | 종전(#645 뒤) | 이제 |
|---|---|---|
| `WIE_AOT` 없음 · 빈 값 | 러너의 기본값 = 켬 | **끔**(`WIE_AOT=off` 를 넘긴다) → `game_lab/reports-<날짜>/` |
| `WIE_AOT=native` | 켬 | 켬 → 기본 출력이 `game_lab/reports-<날짜>-aot/`(두 엔진이 한 열에 섞이지 않는다) |
| 그 밖의 값(`OFF` · `0` · `on` …) | 러너가 판마다 exit 2 · 결과 줄 없음 → 스크립트가 **판마다 «hang» 으로 기록** | 게임을 돌리기 전에 거절 · exit 2 · 출력 디렉터리도 만들지 않는다 |
| 첫 출력 줄 | 엔진 표시 없음 | `engine=interpreter (WIE_AOT=off)` / `engine=native AOT (WIE_AOT=native)` |

고정하는 꼴은 `hook-shadow-sweep.sh` · `arm-interp-profile.sh` 와 같다(`${WIE_AOT:-off}`). **켬으로 재는 길은 별 플래그 없이 그 환경 변수로 남겼다** — 같은 뜻의 스위치를
도구마다 다른 이름으로 두지 않으려는 것이고, 켠 팔을 재야 할 때(생성기를 고친 PR 이 `broken/` 에서 달라지는 판을 볼 때)가 있다. 다만 같은 날 두 팔의 기본 출력이 한
디렉터리면 `<stem>.json` 을 서로 덮고 `summary.tsv` 에 두 엔진의 줄이 섞이므로 켠 팔의 기본 출력에만 `-aot` 를 붙였다. `--out` 을 직접 주면 그 경로를 그대로 쓴다.

셋째 줄은 #645 가 만든 구멍이다: 러너가 틀린 값을 exit 2 · 결과 줄 없음으로 거절하게 됐는데, 이 스크립트는 결과 줄이 없으면 그 판을 «hang: SIGKILLed» 로 적는다. 셸에
`WIE_AOT=OFF` 가 남아 있으면 187판이 전부 hang 으로 기록됐을 것이다 — main 의 스크립트 사본으로 픽스처 한 판을 돌려 재현했다(`FAIL` · `"reason":"hang: SIGKILLed after 50s (infinite tick loop)"` ·
ticks 0 · 실제로는 러너가 곧바로 exit 2 한 것이다). 지금은 스크립트가 먼저 본다.

**근거**(커밋된 픽스처 `test_data/keydraw_ktf.zip` · 디버그 바이너리 · 출력은 repo 밖 스크래치 `$S`):

```
$ echo test_data/keydraw_ktf.zip | env -u WIE_AOT WIE_AOT_CACHE_DIR=$S/rc-cache-off RUST_LOG=info bash scripts/game-lab-recensus.sh --from-stdin --out $S/rc-off
game-lab-recensus: 1 game(s) · engine=interpreter (WIE_AOT=off) · corpus=game_lab/broken · out=$S/rc-off · timeout=20s kill=50s nice=15
   PASS (ktf) ticks=17705086 paints=79  keydraw_ktf.zip
=== done: 1 PASS / 0 FAIL / 0 skipped ===
```

| 팔 | 첫 줄의 엔진 | rc | 판정 | 캐시 디렉터리 | `.log` 의 «ARM AOT» 줄(`RUST_LOG=info`) |
|---|---|---|---|---|---|
| 변수 없음 | `interpreter (WIE_AOT=off)` | 0 | PASS · paints 79 | **생기지 않는다** | **0** |
| `WIE_AOT=native` | `native AOT (WIE_AOT=native)` | 0 | PASS · paints 79 | 기록 1건 | 2(«ARM AOT installed» · «…cache») |
| `WIE_AOT=`(빈 값) | `interpreter (WIE_AOT=off)`(`--dry-run`) | 0 | — | — | — |
| `WIE_AOT=OFF` · `0` · `on` | — | **2** | 돌지 않는다 | 출력 디렉터리 생성 안 됨 | — |

러너는 기본 로그 수준에서 엔진을 적지 않는다(끔 · 켬 모두 «ARM AOT» 0줄 — §2 의 여섯 실행). 그래서 엔진 표시는 스크립트의 첫 줄이 맡고, 위 표의 캐시 · 로그 칸은 그
표시가 실제로 넘어간 값과 맞는지를 따로 본 것이다. 기준선 가드는 그대로다: `WIE_AOT=native … --out ./game_lab/reports --dry-run` → rc 3 · «refusing --out».

**7월 기준선과 같은 형식인가** — 같다:

- `summary.tsv` 머리: 기준선 `result|platform|file|reason|ticks|paints|ms` = 이번 실행 `result|platform|file|reason|ticks|paints|ms`.
- 판마다 `<stem>.json` 한 개. 기준선 452개의 키 모음은 다섯 가지(411개가 11키 · 나머지는 그 부분집합이거나 `last_frame_*` 가 붙은 2개)이고, 그 키 가운데 **이번 실행의
  JSON 에 없는 것은 0** 이다(이번 것은 23키 — 7월 뒤 러너에 는 칸이다 · 엔진과 무관하다: 끔 · 켬의 JSON 은 `ticks` · `ms` 값만 다르다).
- 짝 도구가 그대로 읽는다: `node scripts/game-lab-census-map.mjs --reports $S/rc-off --corpus test_data --out $S/map-off.tsv` → rc 0 · `1 PASS`(그 픽스처 · 출처
  `summary.tsv`) · `10 NO-REPORT`(돌리지 않은 나머지 픽스처).

### 2. `--profile-out` — 두 엔진에서 표본이 뜻하는 바

**표본은 두 엔진에서 같은 것을 센다.** 표본은 엔진 실행 조각이 끝날 때마다 1개다(`wie-core-arm/src/core.rs` 의 `run` 루프 → `sample_profile` · 0276): 조각은 SVC(호스트 호출) ·
함수 반환 · 명령 예산 10,000 소진에서 끝난다. 컴파일된 코드도 같은 예산을 같은 명령에서 다 쓴다(0609 · 0612 — «슬라이스가 끝나는 명령은 그대로다»). 곧 시간으로 뜨는 표본이
아니라 **게스트가 한 일**(호출 수 · 명령 수)을 세는 표본이고, 엔진이 빨라져도 달라질 까닭이 없다.

검수는 `helloworld_ktf.zip` 으로 세 팔이 같다(5줄 · 표본 1934)는 것까지 쟀고 «실제 게임에서 무슨 뜻인지는 재지 않았다»고 적었다. 실제 두 판으로 쟀다 — 디버그 바이너리 ·
`--timeout 20 --profile-out <파일>` · 키 없음(부팅 화면) · 판마다 세 팔을 차례로 · load1 4.5 ~ 5.5:

| 판 | 팔 | 표본 | 조각 끝 `0x8`(SVC) | `0x7f000000`(반환) | 게스트 PC(예산 소진) | 서로 다른 스택 | paints | ticks |
|---|---|---|---|---|---|---|---|---|
| LGT `3cc7a9b4cb15` | 끔 | 229,489 | 179,580 | 32,318 | 17,591(7.7%) | 1,897 | 315 | 319,553 |
| | 켬 · 빈 캐시 | 227,631 | 177,350 | 33,005 | 17,276(7.6%) | 1,887 | 309 | 5,257,746 |
| | 켬 · 적중 | 233,097 | 182,221 | 33,082 | 17,794(7.6%) | 1,902 | 320 | 4,874,837 |
| KTF `155972cac664` | 끔 | 186,850 | 166,360 | 14,556 | 5,934(3.2%) | 740 | 468 | 2,560 |
| | 켬 · 빈 캐시 | 177,536 | 157,504 | 14,448 | 5,584(3.1%) | 732 | 441 | 406,387 |
| | 켬 · 적중 | 185,126 | 164,720 | 14,536 | 5,870(3.2%) | 762 | 463 | 93,702 |

| 스택 분포의 겹침(스택마다 두 팔의 몫 가운데 작은 쪽을 더한 값) | LGT | KTF |
|---|---|---|
| 끔 ↔ 켬 · 빈 캐시 | 98.7% | 99.2% |
| 끔 ↔ 켬 · 적중 | 98.9% | 99.6% |
| **끔 ↔ 끔**(같은 팔을 한 번 더 — 잡음 바닥) | **99.0%** | **99.6%** |

- **같은 스택이 같은 몫으로 나온다 — 해석기를 두 번 돌린 것만큼 같다.** 끔 ↔ 켬의 겹침(98.7 ~ 99.6%)이 끔 ↔ 끔 바닥(99.0 · 99.6%)과 같은 자리다. 끔을 한 번 더 돌린
  값은 LGT 표본 226,296 · 스택 1,878 · paints 310 / KTF 187,193 · 744 · 469 이고, 표본 수도 팔 사이 ±2% 안이다(빈 캐시 팔만 −1 ~ −5% — 컴파일을 기다리는 동안 게스트가
  덜 돈다 · paints 도 같이 준다). 조각 끝의 세 갈래 몫도 같다. 두 장면 다 게스트가 타이머에 맞춰 도는 장면이라 20초 동안 게스트가 한 일이 두 엔진에서 같다.
- **호스트 시간은 이 파일에 없다 — 어느 엔진에서도.** 같은 LGT 판의 프레임당 실행 루프는 끔 14.71 ms ↔ 켬 1.86 ms 다(0612 · release · 다른 장면) — 그 8배가 위 표본에는
  보이지 않는다. (`ticks` 칸은 그 근거가 아니다: 끔끼리도 319,553 ↔ 102,189 로 벌어진다 — `AGENTS.md` «ticks 는 처리량이 아니다».) 해석기에서는 «명령 수 ≈ 호스트 시간»이라 이 표본을 시간처럼 읽어도 크게 틀리지 않았지만, 켠 실행에서는
  그 읽기가 성립하지 않는다 — 게스트 PC 표본이 몰린 자리는 «게스트가 명령을 많이 쓴 자리»이지 «실행이 시간을 쓴 자리»가 아니다. 호스트 시간은
  `scripts/arm-interp-profile.sh`(xctrace)가 재는 것이다.
- 플래그 문서(`wie_validate --help`)에 위 두 가지와 엔진(기본 = 네이티브 AOT · `WIE_AOT=off` = 해석기)을 적었다. 러너 머리 주석의 «스스로 끔을 넘기는 도구» 목록 ·
  `AGENTS.md` 의 같은 목록과 재센서스 항 · `STATE.md` «다음» 의 같은 목록에 `game-lab-recensus.sh` 를 더했다.

### 3. `Cargo.toml` 주석 — 넣었다

넣지 않을 이유는 «주석만 고쳐도 착지하면 릴리스가 선다»(`AGENTS.md` 사건 대장 — `**/Cargo.toml` 이 `publish-artifact.yml` 의 경로 필터에 있다)였다. 그런데 이 PR 은 ⑵ 때문에
`wie_cli/src/bin/wie_validate.rs` 를 고치고, `**/*.rs` 도 같은 필터에 있다 — **릴리스는 어차피 한 번 서고, 주석을 같이 넣어도 두 번이 되지 않는다**(push 한 번 = 실행 한 번).
그래서 넣었다. 바뀐 줄은 전부 `#` 줄이다(`git diff origin/main -- Cargo.toml` 의 `+`/`-` 줄 가운데 `#` 로 시작하지 않는 것 0) · `Cargo.lock` 무변경. 그 릴리스의 wasm 은
내용이 같다 — 고친 `*.rs` 는 `wie_cli/` 아래(wasm 의존 그래프 밖)이고 프로파일 절의 값은 그대로다.

### 4. 검사

`*.rs` 가 최종인 나무에서 돌렸다(그 뒤 바뀐 것은 이 문서와 worklog 뿐이다). 무거운 것은 `build-slot` 으로 감쌌다.

- `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc 0 ·
  `cargo +beta clippy --all -- -D warnings` rc 0.
- `RUST_MIN_STACK=4194304 cargo test --all`(`WIE_AOT` · `WIE_AOT_CACHE_DIR` 해제) rc 0 — 결과 줄 73개 · **1075 passed · 0 failed**(main 과 같은 수 — 시험을 더하지도 빼지도
  않았다). 플래그 문서를 다듬기 전 나무에서 한 번, 최종 나무에서 한 번 — 두 번 같은 수다(load1 4.1 ~ 6.1). `profile_out_writes_folded_stacks_only_when_asked_test` ·
  `aot_is_on_unless_wie_aot_is_off_and_a_mistyped_switch_is_refused_test` ok. 시험 뒤 사용자 캐시 디렉터리의 기록 0.
- 재센서스 스크립트에는 자가 시험이 없다(종전에도 없었다) — §1 의 표가 손으로 돌린 것이다. `bash -n` rc 0 · `shellcheck -S warning` 은 종전부터 있던 1건(`content` 미사용)뿐.
- 문서 검사: `check-engine-runner-fixtures.mjs` OK · `check-doc-liveness-parity.mjs` OK(26줄 — fenced `sh` 블록은 고치지 않았다 · 워크플로 무변경) ·
  `check-worklog-json.mjs` OK · `check-docs-report-serial.mjs` OK(0614 · 열린 PR claim 충돌 0) · `player-data.mjs` rc 0 · `npm run audit` PASSED. `AGENTS.md` 840줄(상한 850).
- 측정 바이너리(§1 · §2)는 플래그 문서를 다듬기 전의 디버그 빌드다 — 그 뒤 `*.rs` 에서 바뀐 것은 `///` 주석 네 줄(`--help` 의 글)뿐이다.
- 게임 파일명 유입(`node scripts/corpus-name-inflow.mjs`): **6회 / 4쌍(BOUNDED)** · 판단 필요 **0회 / 0쌍(SUFFIX-ATTACHED)**. 여섯 다 main 부터 있던 줄이다 — 도구는 고친
  파일의 본문 전체를 본다(`game-lab-recensus.sh` 머리 주석의 예시 둘 · `wie_validate.rs` 주석 넷). 이 PR 이 더한 줄에 든 것은 0 이다(`git diff origin/main...HEAD` 의 `+`
  줄에서 그 네 낱말을 세어 0 · 0 · 0 · 0).
- 이 회차가 띄운 것은 전부 차례로 돌았다(동시 실행 1) · 지속 캐시는 `WIE_AOT_CACHE_DIR` 로 스크래치에 두었다 · 출력은 전부 repo 와 `game_lab/` 밖이다.

### 5. 한계 · 하지 않은 것

- **켠 팔로 `broken/` 을 돌려 보지 않았다.** 이 회차가 재센서스로 돌린 것은 픽스처 한 판(끔 · 켬)뿐이다 — 187판 재센서스(이 Mac 45분)는 티켓 밖이다.
- §2 의 두 장면은 **타이머에 맞춰 도는 부팅 화면**이다. 해석기가 제 속도를 못 내는 장면(끔에서는 게스트가 밀리고 켬에서는 안 밀리는 장면)에서는 같은 시간 창에 켠 팔의
  표본이 더 많을 것이다 — 재지 않았고, 그래서 플래그 문서에도 적지 않았다. 표본 하나의 뜻(조각 끝 1개)은 그때도 같다.
- §2 는 판마다 1회 · 디버그 바이너리다. 겹침 98.7 ~ 99.6% 를 «같다»로 읽는 근거는 끔 ↔ 끔 바닥과 견준 것이다(표의 마지막 줄).
- `lgt_render_probe.sh` 는 여전히 기본값(켬)을 따른다 — 저장된 기준선과 견주는 도구가 아니라서 #645 의 기준에 들지 않는다(검수도 들지 않았다).
- 재센서스의 `-aot` 접미는 **기본 출력에만** 붙는다. `--out` 을 직접 주고 두 팔을 같은 디렉터리에 쓰면 섞인다 — 첫 줄의 엔진 표시가 유일한 기록이다(JSON 에는 엔진 칸이 없다).

<!-- corpus-name-inflow v1 subjects=7 tree=9b516d713d1c5c29 B=6/4 P=0/0 S=0/0 -->
