## [2026-11-27] 검증 러너의 네이티브 ARM AOT 를 기본 켬으로 — `WIE_AOT=off` 로 끈다 · 해석기를 재는 도구 넷은 끔으로 고정 (wie-arm-aot-native-validate-runner-default-on-with-off-switch)

**무엇을**: `wie_validate` 가 KTF · LGT 게스트를 기본으로 컴파일해 돈다(네이티브 ARM AOT · `wie-core-arm-native`). `WIE_AOT=off` 면 종전대로 해석기만 쓴다.
해석기의 수를 재는 도구 넷은 스스로 끔을 넘긴다. 문서(`AGENTS.md` 러너 블록 · `smoke_gate` 항 · `wie_validate.rs` 머리 · `STATE.md`)를 그 기본값에 맞췄다.

**왜**: `STATE.md` «다음» ⑴(티켓 `wie-arm-aot-native-validate-runner-default-on-with-off-switch`). 막던 까닭 — 디버그 바이너리가 빈 캐시로 도는 첫 실행이
`smoke_gate` 의 기한을 넘는다 — 은 0611 이 풀었고, 그 회차가 적은 범위(0611 §4)를 그대로 했다.

**사용자 영향**: 없다. 제품 기본값은 `aot: None` 그대로다(`wie_featurephone/src/lib.rs:233`). 브라우저 배선 0 · export 표면 변화 0 · `compat.json` 무변경 · 고지 0건 ·
게임 파일 무수정. 바뀌는 것은 이 저장소에서 검증 러너를 부르는 사람과 스크립트가 보는 엔진이다. `*.rs` 가 바뀌므로 착지하면 릴리스는 한 번 선다
(`publish-artifact.yml` 의 `**/*.rs` — 바뀐 `*.rs` 넷은 전부 `wie_cli/` 아래이고 wasm 산출물의 의존 그래프 밖이다 · §6).

| | 끔(`WIE_AOT=off`) | **기본(켬)** |
|---|---|---|
| `smoke_gate` 디버그 244판(LGT 54 · KTF 190) | 244 PASS · 0 FAIL | **244 PASS · 0 FAIL**(빈 캐시에서 시작) |
| `smoke_gate` release 244판 | 244 PASS · 0 FAIL | **244 PASS · 0 FAIL**(빈 캐시에서 시작) |
| 끔과 판정이 다른 판 · 기준선 대비 regressions | — | **0 · 0**(디버그 · release 둘 다 · 재시도로 PASS 가 된 판도 0 — §3) |
| 러너 블록 여섯 픽스처(디버그 · 문서의 명령 그대로) | 6/6 `PASS` · rc 0 | **6/6 `PASS` · rc 0**(빈 캐시 · 그 캐시로 다시 — §4) |
| doc-liveness(ubuntu x86-64 · 브랜치 손 실행) | — | run `45926844387` · **success** — 러너 블록 6/6 `PASS` · `cargo test --all` 1073 passed(§5) |

### 1. 스위치 — `WIE_AOT`

| `WIE_AOT` | 동작 | 종전 |
|---|---|---|
| 없음 · 빈 값 | **켬** — `NativeExecutor` + 지속 캐시(`AotCache`) | 끔 |
| `native` | 켬(종전의 opt-in — 0607 ~ 0611 의 명령이 그대로 돈다) | 켬 |
| `off` | 끔 — `aot: None`(해석기만 · 캐시 디렉터리도 만들지 않는다) | 끔(그 값은 «native 가 아님»으로 읽혔다) |
| 그 밖의 값(`0` · `OFF` · `false` …) | **거절** — stderr 한 줄 · exit 2 · 결과 줄 없음 | 끔 |

- **그 밖의 값을 거절하는 까닭**: 끄는 스위치를 잘못 치면(`WIE_AOT=0`) 조용히 켠 채로 돈다 — 재려던 엔진이 아닌 것을 재고도 아무 표시가 없다. exit 2 는 잘못된 플래그에
  `clap` 이 내는 값이고 이 나무에서 «재지 못했다»의 값이다(`UNMEASURED`). 결과 줄을 내지 않으므로 `"result":"PASS"` 를 찾는 호출자 셋에게도 PASS 가 아니다.
  `FAIL` 줄로 내지 않은 까닭: `FAIL` 은 그 판에 대한 말이다.
- 바꾼 코드는 `build_emulator` 의 `aot:` 한 식과 그 위의 판독 함수 `aot_switch`(순수 함수)다. 캐시(`AotCache`)는 0610 의 것 그대로다 — 자리(사용자 캐시
  디렉터리의 `validate-native-aot/` · `WIE_AOT_CACHE_DIR`) · 상한 8 GiB · 바이너리 표식 모두 무변경.
- **`wie_validate.rs` 의 단위 시험은 해석기로 돈다**(`cfg!(test)` 한 가지). 그 시험들은 `run()` 을 프로세스 안에서 불러 픽스처를 돌리고 틱 수를 단언한다
  («첫 틱에 끝나는 게스트»). 켠 실행에서는 **컴파일을 기다리는 동안에도 틱이 돈다** — `helloworld_*` 의 틱이 끔 1 → 켬 1.4만(ubuntu · §5) ~ 40만이다.
  켠 채로 둔 `cargo test --all` 다섯 번 가운데 한 번이 그 단언에서 떨어졌다(`relaunch_reboots_after_a_clean_exit_and_keeps_the_database` — `97365 vs 402411`:
  한 번 부팅한 실행의 틱이 세 번 부팅한 실행보다 많았다 · 시험 스레드들이 한꺼번에 컴파일하는 첫 실행이 늦다). 또 켠 채로는 시험이 사용자 캐시 디렉터리에
  기록을 썼다(뒤집은 직후의 실행이 2건 — 4.7 MB · 47 KB · 이름 머리의 길이가 시험 바이너리의 것이었다 · 지웠다). 해석기로 돌린 뒤: 연달아 세 번
  1073 passed · 그 디렉터리에 생긴 파일 0. 기본값(켬)은 진짜 바이너리를 띄우는 시험 셋이 지킨다(아래 두 줄).
- **시험**(`wie_cli/tests/validate_aot_switch.rs` — 진짜 바이너리 · 커밋된 픽스처 `helloworld_ktf.zip` · 캐시는 시험의 임시 디렉터리 · `--timeout 20` — 픽스처는 스스로 끝난다): 결과 줄은 어느 엔진이 돌았는지
  말하지 않으므로 **캐시 기록**을 본다 — 없음 → 기록 1 · `off` → 기록 0(디렉터리도 없다) · `native` → 기록 1 · `OFF` → 기록 0 · exit 2 · stdout 빈 줄.
  이 시험은 `cargo test --all` 에 들어 있어 PR 의 `rust_ci (ubuntu-latest, stable)` 에서도 돈다 — 검증 러너가 x86-64 리눅스에서 네이티브 AOT 를 도는 첫 자리다(§5).
- 러너 바이너리를 띄우는 기존 시험 둘(`validate_stub_hits.rs` · `validate_profile_out.rs`)은 켠 채로 그대로 통과한다(빈 캐시로 3회 · `--profile-out` 의 표본도 나온다).
  기록을 시험의 임시 디렉터리로 돌리는 줄만 더했다(`WIE_AOT_CACHE_DIR`).

손으로 한 번씩(디버그 바이너리 · `helloworld_ktf.zip` · `--timeout 2` · `RUST_LOG=warn,wie_core_arm_native=info` · 캐시는 스크래치 디렉터리):

| 실행 | rc | 결과 줄 | 로그 | 캐시 |
|---|---|---|---|---|
| 없음(빈 캐시) | 0 | `PASS` | «Stored native ARM AOT cache» 1줄 | 기록 1건(84 KB) |
| 없음(다시) · `native` · 빈 값 | 0 · 0 · 0 | `PASS` ×3 | «persistent cache hit» 1줄씩 | 그대로 |
| `off`(새 디렉터리) | 0 | `PASS` | AOT 줄 0 | 디렉터리가 생기지 않는다 |
| `0` · `OFF` | 2 · 2 | 없음 | `wie_validate: WIE_AOT="0" is not a value: …` | 디렉터리가 생기지 않는다 |

### 2. 끔으로 고정한 도구 — 티켓이 든 둘 + 같은 기준으로 둘 더

기준은 티켓의 것이다: 그 도구가 내는 수가 «해석기의 수»이고, 기본값이 바뀌면 재는 것이 조용히 달라지는가.

| 도구 | 고정 | 왜 | 근거(명령) |
|---|---|---|---|
| `perf-watch.mjs` 네이티브 축(티켓) | **항상 끔** — `nativeOne()` 이 `WIE_AOT=off` 를 넘긴다 | 기준선(`judge.json`)과 A 쪽(지난 회차의 핀)이 전부 해석기 값이다. 옛 A 바이너리는 그 변수를 «native 가 아님»으로 읽어 같은 끔이다 | `node scripts/perf-watch.mjs selftest` 30/30 — 새 경우 «네이티브 축의 실행은 전부 WIE_AOT=off 로 뜬다» |
| `arm-interp-profile.sh`(티켓) | **기본 끔**(`WIE_AOT=${WIE_AOT:-off}`) | 이름대로 해석기의 프로파일이다. 0607 ~ 0610 이 켠 팔을 이 스크립트로 쟀으므로 `WIE_AOT=native` 를 주면 켠 팔을 잰다(머리 주석) | `LC_ALL=C /usr/bin/grep -n 'WIE_AOT' scripts/arm-interp-profile.sh` → 실행 줄 1 · 주석 2 |
| `playability-census.mjs`(더한 것) | **항상 끔** — `validate()` 의 `spawn` 환경 | `compat.json`(지원 현황)은 제품이 싣는 엔진에 대한 말이고 그것은 해석기다. 주간 퇴행 감시(`regression-census.sh`)도 이 길로 `compat.json` 과 견준다 — 켠 채로 재면 해석기만의 퇴행을 못 본다 | 가짜 러너(받은 변수를 적는 스크립트)로 `run --only probe` 1회 → `2 WIE_AOT=off`(탐침 A · B) |
| `hook-shadow-sweep.sh`(더한 것) | **기본 끔**(`WIE_AOT=${WIE_AOT:-off}`) | 켠 채로는 shadow 가 훅 자리를 불릴 때마다 두 번 고쳐 쓰고 그 영역을 버려, 같은 실행에서 닿는 호출이 절반이었다(0607 §6 — `memcpy` 366,150 ↔ 175,740). 이 표는 `fires` 를 먼저 읽는다 | 같은 가짜 러너(`WIE_VALIDATE=`)로 1회씩 → `1 WIE_AOT=off` · `WIE_AOT=native` 를 주면 `1 WIE_AOT=native` · `bash scripts/hook-shadow-sweep.sh selftest` ok |

- **뒤의 둘은 티켓에 없다.** 넣은 까닭은 위 «왜» 칸이고, 빼려면 각각 한 줄이다(`playability-census.mjs` 의 `WIE_AOT: 'off'` · `hook-shadow-sweep.sh` 의 `WIE_AOT=${WIE_AOT:-off}`).
  census 의 고정은 브라우저에 AOT 가 붙는 PR 이 푸는 자리다(`STATE.md` «다음»에 적었다).
- **고정하지 않은 호출자**(기본값 = 켬을 따른다): `AGENTS.md` 의 러너 블록 · `smoke_gate.sh` · `lgt_render_probe.sh` · `game-lab-recensus.sh` · 손으로 치는 `wie_validate`.
  `perf-watch` 의 브라우저 축 · `audio-probe.mjs` · `contract-roundtrip.mjs` 는 wasm 을 돈다 — 이 스위치와 무관하다.
- `perf-watch` 의 `compare <A> <B>` 로 «끔 ↔ 켬»을 재던 길(0607 · 0608 — B = 스위치를 켠 래퍼 스크립트)은 그대로 된다: 래퍼가 자기 안에서 변수를 다시 정한다.

### 3. `smoke_gate` — 디버그 · release 를 끔 / 기본(켬)으로 나란히

`scripts/smoke_gate.sh` 그대로(`TIMEOUT` 15 · `RETRY` 2 · 무수정) · 줄기 3개(에뮬레이터 ≤ 3) · (빌드, 팔, 플랫폼)마다 long 임대 1건(8건 · 4분 9초 ~ 16분 46초).
플랫폼마다 끔을 먼저, 기본(켬)을 그 다음 임대로 돌렸다. **끔 = `WIE_AOT=off` · 켬 = `WIE_AOT` 도 `WIE_AOT_CACHE_DIR` 도 없는 환경** — 기록은 진짜 기본 자리
(`~/Library/Caches/net.dlunch.wie/validate-native-aot/` · 이 회차 전에는 없던 디렉터리)에 쌓였다. 바이너리 표식 때문에 두 빌드 모두 빈 캐시에서 시작한다.

| 244판(LGT 54 · KTF 190) | 디버그 · 끔 | **디버그 · 기본(켬)** | release · 끔 | **release · 기본(켬)** |
|---|---|---|---|---|
| PASS / FAIL | 244 / 0 | **244 / 0** | 244 / 0 | **244 / 0** |
| 끔과 판정이 다른 판(판마다 대조) | — | **0** | — | **0** |
| 기준선(`smoke_gate_baseline.tsv` 242행) 대비 regressions · absent | 0 · 0 | **0 · 0** | 0 · 0 | **0 · 0** |
| 줄기 6개의 rc · «OK: no regressions vs baseline.» | 전부 0 · 6줄 | 전부 0 · 6줄 | 전부 0 · 6줄 | 전부 0 · 6줄 |
| 러너를 부른 횟수 · 그중 `PASS` 가 아닌 것 | 244 · 0 | **244 · 0** | 244 · 0 | **244 · 0** |
| 이 팔이 쓴 캐시 기록 | 0 | 209건 · 4.36 GiB(LGT 45 · KTF 164) | 0 | 209건 · 4.36 GiB(LGT 45 · KTF 164) |
| 실행을 끝낸 것(`stop`) | deadline 244 | deadline 244 | deadline 104 · max-ticks 140 | deadline 128 · max-ticks 116 |
| 벽시계(LGT + KTF) | 292 + 1006 초 | 290 + 999 초 | 249 + 790 초 | 255 + 887 초 |
| 임대를 끝낼 때의 load1(LGT · KTF) | 16.4 · 5.1 | 10.3 · 7.1 | 5.8 · 14.2 | 8.1 · 12.3 |

- **재시도에 기댄 PASS 가 없다.** `smoke_gate.sh` 는 판마다 마지막 판정만 찍는다(0607 · 0608 의 한계 줄). 이번에는 `BIN=` 에 러너를 그대로 부르고 실행마다 결과를 한 줄
  적는 껍데기를 물렸다: 네 팔 모두 러너 호출이 정확히 244회이고 전부 `PASS` 다 — 첫 시도에 FAIL 이 나서 다시 돈 판이 0 이다.
- **기록 수는 0610 · 0611 과 같다**(209건 · 4.36 GiB — LGT 45 · KTF 164 · 게스트 이미지가 같은 판들이 기록 하나를 나눠 쓴다). 건당 5.3 ~ 69.4 MB · 평균 21.3 MB.
- **상한 8 GiB 가 실제 디렉터리에서 일했다**(0610 §8 이 «시험으로만 봤다»고 남긴 자리). 디버그의 209건 4.36 GiB 가 있는 채로 release 켬 팔이 4.36 GiB 를 더 썼다:
  30초 간격으로 본 합의 최댓값이 8,389,364 KB(8.001 GiB · 상한 8,388,608 KB)이고, 넘는 동안 **가장 오래 전에 쓴 기록부터**(디버그 바이너리의 것) 36건이 나갔다.
  끝난 뒤 7.99 GiB — release 209건은 전부 남고 디버그는 173건. 그 사이의 실행 190회는 전부 `PASS` 다(지우는 것이 도는 실행을 건드리지 않았다).
- release 켬 팔의 KTF 가 끔보다 97초 길다(887 ↔ 790초). 빈 캐시의 켬은 판마다 컴파일을 한 번 하고(0610 §3 — release 빗나감의 첫 화면이 끔 + 0.9 ~ 1.8초),
  그 임대의 앞부분에는 다른 레인의 `cargo test --all` 이 겹쳤다(load1 12 ~ 14). 디버그는 두 팔 모두 판마다 기한 15초를 다 써서 벽시계가 같다.
  release 에서 `stop` 의 몫이 팔마다 다른 것은 틱 상한(5천만)에 먼저 닿는 쪽이 플랫폼마다 다르기 때문이다(0607 §6 — 판정이 아니다 · 네 팔 모두 244 PASS).
- **0611 의 같은 표(디버그 · `WIE_AOT=native` 로 켬)와 같은 결과다**: 244 PASS · 달라진 판 0 · 기록 209건 4.36 GiB. 달라진 것은 켜는 길(변수 없이)과 기록의 자리(기본 디렉터리)뿐이다.

**`WIE_AOT=off` 가 종전의 끔과 같은가** — 같다:
1. 코드: `off` 는 `aot: None` 이다 — main 이 변수가 `native` 가 아닐 때 만들던 `Options` 와 같은 값이다. 그 식과 판독 함수 밖에 바뀐 실행 코드는 없다
   (`git diff origin/main...HEAD -- wie_cli/src/bin/wie_validate.rs` — 나머지는 주석과 `cfg!(test)` 한 줄).
2. 판정표: 이 회차의 끔 두 팔(디버그 · release)이 판마다 기준선 242행과 같고(regressions 0 · absent 0), 0611 의 디버그 끔(244 PASS) · 0610 의 release 끔(244 PASS)과 같다.
3. 부작용: `off` 로 돈 488회가 캐시 디렉터리에 쓴 것 0(끔 임대 넷의 앞뒤 기록 수가 같다 — 0 → 0 · 45 → 45 · 209 → 209 · 254 → 254).

표 · 임대별 줄 · 줄기의 rc · 실행별 결과 집계는 저장소 밖 `~/orchestrator/reports/evidence/wie-arm-aot-native-validate-runner-default-on-with-off-switch/` 에 있다
(`smoke-compare.txt` · `driver1.log` · `driver2.log` · `attempts-by-phase.txt` · `cache-during-release-ktf-on.txt` — 판 이름은 싣지 않았다).

### 4. 러너 블록 — 문서의 명령 그대로

`AGENTS.md` 의 블록을 글자 그대로(`cargo run -q -p wie_cli --bin wie_validate -- …` · 디버그 · 코드 커밋 `5ef684df8` 로 지은 바이너리) 세 번, 임대 1건 안에서 차례로:
변수 없이(이 바이너리의 첫 실행 = 빈 캐시) → 변수 없이 다시 → `WIE_AOT=off`. `RUST_LOG=warn,wie_core_arm_native=info` 만 더해 캐시 줄을 읽었다.

| 픽스처 | 기본(켬) · 첫 실행 | 기본(켬) · 다시 | `WIE_AOT=off` | paints · 키(세 번 같다) |
|---|---|---|---|---|
| `draw_j2me.jar` | PASS · rc 0 · 캐시 줄 없음(J2ME) | PASS · rc 0 | PASS · rc 0 | 1 · — |
| `helloworld_ktf.zip` | PASS · rc 0 · «Stored» | PASS · rc 0 · «cache hit» | PASS · rc 0 · AOT 줄 0 | 0 · —(clean exit) |
| `helloworld_lgt.zip` | PASS · rc 0 · «Stored» | PASS · rc 0 · «cache hit» | PASS · rc 0 · AOT 줄 0 | 0 · —(clean exit) |
| `keydraw_ktf.zip` `--inject --expect-last-frame` | PASS · rc 0 · «Stored» | PASS · rc 0 · «cache hit» | PASS · rc 0 · AOT 줄 0 | 79 · 27/27 |
| `keydraw_lgt.zip` 〃 | PASS · rc 0 · «Stored» | PASS · rc 0 · «cache hit» | PASS · rc 0 · AOT 줄 0 | 55 · 27/27 |
| `text_j2me.jar` `--timeout 5` | PASS · rc 0 | PASS · rc 0 | PASS · rc 0 | 1 · — |

- 18회 전부 `PASS` · rc 0. ARM 픽스처 넷이 첫 실행에 기록 4건을 쓰고(84 KB ~ 4.7 MB) 둘째 실행은 넷 다 적중이다. `keydraw_*` 는 세 번 모두 `content true` ·
  `last_frame_content true` 이고 paints 가 같다(79 · 55 — ubuntu 실행의 값과도 같다 · §5).
- 벽시계(`cargo run` 포함)는 세 번이 같은 자리다 — `keydraw_*` 22.4 ~ 22.7초(키 주입 일정이 정한다) · `helloworld_*` 2.5 ~ 2.8초 · `text_j2me` 7.4초.
  픽스처는 작아서 빈 캐시의 컴파일이 벽시계에 드러나지 않는다. 끝낼 때의 load1 5.9 ~ 7.8.

### 5. doc-liveness — ubuntu x86-64 에서 러너 블록

**돈다 — 워크플로에 `WIE_AOT=off` 를 넣지 않았다.** `gh workflow run doc-liveness.yml -R Jun025/wie --ref wie-arm-aot-native-validate-runner-default-on-with-off-switch` 1회:
run **`45926844387`** · `workflow_dispatch` · head `5ef684df8`(이 PR 의 코드 커밋 — 그 뒤에 바뀐 `*.rs` 는 시험 빌드에서만 갈리는 줄과 주석이다 · §7) · **conclusion `success`** · 2026-11-27T02:11:18Z → 02:36:16Z ·
러너 `ubuntu-26.04` · 툴체인 `stable-x86_64-unknown-linux-gnu`.

| 단계 | 결과 | 걸린 시간 |
|---|---|---|
| The four gates + the beta axis | success — `cargo test --all` 결과 줄 73개 · **1073 passed · 0 failed** · `aot_is_on_unless_wie_aot_is_off_and_a_mistyped_switch_is_refused_test ... ok` | 515초 |
| The engine runner block | success — 아래 여섯 줄 | 183초 |
| Web-surface commands | success | 721초 |
| `npm run verify`(운영) | skipped — schedule 이벤트에서만 돈다(주 1회 상한) | — |
| Landing-paperwork commands | success | 1초 |

러너 블록의 결과 줄(그 로그에서 뽑은 것 · 워크플로는 `RUST_LOG` 를 주지 않아 «AOT installed» 줄은 없다):

| 픽스처 | 결과 | `stop` | 키 | paints | `content` · `last_frame_content` |
|---|---|---|---|---|---|
| `draw_j2me.jar` | PASS | deadline | — | 1 | true · true |
| `helloworld_ktf.zip` | PASS | clean exit | — | 0 | false · false |
| `helloworld_lgt.zip` | PASS | clean exit | — | 0 | false · false |
| `keydraw_ktf.zip` `--inject --expect-last-frame` | PASS | deadline | 27/27 | 79 | true · true |
| `keydraw_lgt.zip` 〃 | PASS | deadline | 27/27 | 55 | true · true |
| `text_j2me.jar` `--timeout 5` | PASS | deadline | — | 1 | true · true |

- **그 호스트에서 컴파일이 실제로 돌았다는 근거는 같은 실행의 시험이다**: `validate_aot_switch` 는 변수가 없을 때 캐시 기록이 1건 생기는 것을 본다 — 기록은 컴파일을
  마친 실행만 쓴다. `wie-core-arm-native` 자신의 시험(생성기 · 실행기)은 그 전부터 이 다리에서 돌았다. 검증 러너가 그 실행기를 붙이고 x86-64 에서 돈 것은 이번이 처음이다.
- `keydraw_lgt` 의 paints 55 는 `AGENTS.md` 가 적은 한가한 범위(48 ~ 55) 안이다. 그 범위는 고치지 않았다.
- 이 실행은 손 실행이라 주간 그물의 «마지막 schedule 실행»을 갈음하지 않는다(그 소유 규칙은 게이트③ 회차의 것이다 · 마지막 schedule 은 2026-10-31 success).
- 로그 사본: 저장소 밖 증거 디렉터리의 `doc-liveness-45926844387.log`.

### 6. 무변경 근거

| 무엇 | 명령 | 결과 |
|---|---|---|
| 제품 기본값 | `LC_ALL=C /usr/bin/grep -n 'aot:' wie_featurephone/src/lib.rs` | `233: aot: None, // interpreter only: …` (그 파일은 diff 에 없다) |
| 제품 · 웹 · 계약 · 판별 데이터 · 매니페스트 | `git diff --name-only origin/main...HEAD -- wie_featurephone wie-web web functions docs/player-data docs/contracts Cargo.toml Cargo.lock '**/Cargo.toml'` | 0줄 |
| 바뀐 `*.rs` | `git diff --name-only origin/main...HEAD -- '*.rs'` | `wie_cli/src/bin/wie_validate.rs` · `wie_cli/tests/validate_{aot_switch,profile_out,stub_hits}.rs` |
| wasm 산출물의 의존 그래프 | `cargo tree -p wie_featurephone --target wasm32-unknown-unknown -e normal` · 같은 것 `-p wie-web` 에서 `wie_cli` · `cranelift` · `wie-core-arm-native` 를 센다 | 0 · 0 |
| `compat.json` | `git diff --stat origin/main...HEAD -- docs/player-data/` | 0줄 — census 는 돌리지 않았고 이제 끔으로 고정이다(§2) |

`Cargo.toml` 둘의 주석(`WIE_AOT=native` 를 이름으로 든 두 줄)은 고치지 않았다 — 그 값은 여전히 받는 값이고 문장은 지금도 참이다.

### 7. 검사

`*.rs` 가 최종인 나무에서 돌렸다(그 뒤 바뀐 것은 문서뿐이다). 무거운 것은 `build-slot` 으로 감쌌다.

- `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc 0 ·
  `cargo +beta clippy --all -- -D warnings` rc 0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc 0 — 결과 줄 73개 · **1073 passed · 0 failed**(main 의 1072 + 새 시험 1건 · 결과 줄은 새 시험 파일 하나만큼 는다).
  최종 나무에서 연달아 세 번 같은 수다(load1 8.5 ~ 9.8). 시험 뒤 사용자 캐시 디렉터리에 생긴 기록 0(§1). ubuntu x86-64 에서도 같은 수다(§5 의 실행 — `5ef684df8`).
- 스크립트의 자가 시험: `node scripts/perf-watch.mjs selftest` 30/30(종전 29 + 새 경우 1) · `node scripts/playability-census.mjs selftest` 70/70 ·
  `bash scripts/hook-shadow-sweep.sh selftest` ok · `bash scripts/test-smoke-gate.sh` ALL OK.
- 문서 검사: `check-engine-runner-fixtures.mjs` OK · `check-doc-liveness-parity.mjs` OK(26줄 — fenced `sh` 블록은 고치지 않았다 · 워크플로 무변경) ·
  `check-worklog-json.mjs` OK · `check-docs-report-serial.mjs` OK · `player-data.mjs` OK · `audit-no-leak.sh` PASSED. `AGENTS.md` 838줄(상한 850).
- **연번**: 이 기록은 0612 로 쓰기 시작했다. 올리기 전의 검사에서 PR #644(다른 레인 · 생성기)가 같은 번호를 먼저 claim 한 것이 나와 0613 으로 옮겼다
  (`check-docs-report-serial.mjs --next-serial`). 이미 올린 첫 커밋 `5ef684df8` 의 제목에는 «(0612)» 가 남아 있다 — 공개 이력은 고쳐 쓰지 않는다.
- **측정 바이너리와 doc-liveness 는 `5ef684df8` 이다.** 그 커밋 뒤에 `*.rs` 에서 바뀐 것: 단위 시험을 해석기로 돌리는 `cfg!(test)` 가지(§1 — 시험이 아닌 빌드에서는
  같은 코드다 · 그 커밋에 있던 «시험은 기록을 남기지 않는다» 가지를 갈음했다) · 새 시험의 기한 2 → 20초 · 주석의 연번.
- 게임 파일명 유입(`node scripts/corpus-name-inflow.mjs`): **4회 / 3쌍(BOUNDED)** · 판단 필요 **1회 / 1쌍(SUFFIX-ATTACHED)**. BOUNDED 넷은 `wie_validate.rs` 에
  main 부터 있던 주석 줄이고, SUFFIX-ATTACHED 하나는 `playability-census.mjs` 에 main 부터 있던 주석 속 메뉴 낱말이다(제목 언급이 아니다) — 도구는 고친 파일의
  본문 전체를 본다. 이 PR 이 더한 줄에 든 것은 0 이다(`git diff origin/main...HEAD` 의 `+` 줄에서 그 네 낱말을 세어 0 · 0 · 0 · 0).
- **이 PR 의 CI**(PR #645 · head `0f53496ca` — `*.rs` 가 최종인 커밋): `gh pr checks 645 -R Jun025/wie` 전부 pass — `rust_ci (ubuntu-latest, stable)` 6분 46초 ·
  `contract` 7분 16초 · `build-web` 6분 24초 · 나머지 `rust_ci` 다섯 다리는 PR 에서 빈 통과다(`AGENTS.md`). 그 green 은 다른 OS 를 덮지 않으므로
  **여섯 다리 전부를 손으로 1회 돌렸다**: `gh workflow run rust.yml -R Jun025/wie --ref <이 브랜치>` → run **`45938137002`** · `success` — ubuntu · macOS · Windows 의
  stable · beta 여섯 다리가 `cargo test --all` 을 돌았고(357 ~ 1365초) 여섯 로그 모두에 `aot_is_on_unless_wie_aot_is_off_and_a_mistyped_switch_is_refused_test ... ok` 가 있다.
  곧 검증 러너가 세 OS 에서 변수 없이 픽스처를 컴파일해 기록을 쓰고, `off` 에서는 쓰지 않는다. 이 뒤의 커밋은 이 문서뿐이다.

### 8. 한계 · 하지 않은 것

- **`playability-census.mjs` 와 `hook-shadow-sweep.sh` 의 고정은 티켓이 이름으로 든 것이 아니다**(§2). 넣은 기준은 티켓의 것(그 수가 해석기의 수인가)이고, 빼는 것은 각각 한 줄이다.
  두 도구를 진짜 판으로 돌려 보지는 않았다 — 넘기는 값만 가짜 러너로 봤다. `perf-watch` 의 주간 경로도 돌리지 않았다(selftest 뿐이다).
- **`lgt_render_probe.sh` · `game-lab-recensus.sh` 는 기본값(켬)을 따르고, 켠 채로 돌려 보지 않았다.** 둘 다 `smoke_gate` 와 같은 디버그 바이너리 · 기한 12 ~ 20초다.
- **디버그 스윕의 바이너리는 코드 커밋 `5ef684df8` 직전의 작업 나무로 지은 것이다.** 그 뒤에 `*.rs` 에 든 것은 시험 빌드에서만 갈리는 `cfg!(test)` 가지와
  머리 주석의 문장 하나다. release 스윕 · 러너 블록 · doc-liveness 는 그 커밋으로 지은 바이너리다(스윕 도중에 바꿔 넣었다 — release 팔이 시작하기 전).
- **켠 실행의 `ticks` 는 컴파일 대기를 포함한다**(§1 — 틱은 원래도 처리량의 자가 아니다 · `AGENTS.md`). 틱 상한(`--max-ticks` 기본 5천만)이 그만큼 먼저 온다:
  release 스윕에서 상한으로 끝난 실행은 끔 140 · 켬 116 이라 이 코퍼스의 기한 15초에서는 드러나지 않았다. 켠 채 `--inject` 로 `UNMEASURED`(`max-ticks`)를 보면
  상한을 올리는 것이 먼저다(0607 §6 이 그 모양을 봤다).
- **켬 팔은 «빈 캐시에서 시작» 하나다.** 채운 캐시로 한 바퀴 더(적중 팔)는 돌리지 않았다 — 0610 §4 가 release 에서 244 PASS · 쓴 기록 0 으로 봤고, 적중의 첫 화면은
  빗나감보다 이르다(0611 §1). 러너 블록의 둘째 실행(§4)이 이 회차의 적중이다.
- **스윕 내내 다른 레인이 같이 돌았다**(long 풀이 한 칸이라 다른 레인의 `smoke_gate` 스윕과 임대를 번갈아 썼다 · short 풀의 빌드 · 시험). 임대를 끝낼 때의 load1 5.1 ~ 16.4.
  «부하가 높을 때도 전부 통과한다»까지는 이 표가 말하지 않는다(0611 §3 과 같은 한계).
- **게이트의 `cargo test --all` 한 번이 디버그 LGT 끔 팔과 겹쳐 돌았다**(커밋을 일찍 올려 doc-liveness 를 스윕과 나란히 돌리려고). 그 시험이 띄우는 러너는 픽스처 실행
  여섯(각 2초 이하)이다 — 그 팔은 54 PASS 였다.
- **이 회차가 기본 자리에 쓴 기록은 지웠다**(두 표식 386건 · 8.0 GiB — 스크래치 사본과 다시 지어질 `target/debug` 의 것이라 다시 읽히지 않는다). `validate-native-aot/` 는
  비어서 같이 지웠고, 그 위 `net.dlunch.wie/` 에는 이 회차가 만들지 않은 빈 파일 `.metadata_never_index` 하나가 생겨 있어 그대로 뒀다.
- **캐시의 상한(8 GiB)과 바이너리 표식은 그대로다.** 기본 켬이 되면 그 디렉터리는 이 Mac 의 레인들이 같이 쓴다 — 새로 지은 바이너리마다 `smoke_gate` 한 바퀴가 4.4 GiB 를
  쓰므로 두 바이너리째부터 앞 것의 기록이 나간다(§3 이 그 모양이다). 표식을 «생성기 소스의 해시»로 좁히는 것은 0610 이 남긴 자리 그대로다(`ponytail:` 주석).
- **wie-3 의 `wie-arm-aot-native-budget-check-at-guest-basic-block-head`(생성기)와 파일이 겹치지 않는다** — `wie-core-arm-native/` · `wie-core-arm/` 의 diff 0.
  다만 뜻은 닿는다: 이 PR 이 착지하면 그 PR 의 생성기가 검증 러너의 기본 엔진이 된다. `STATE.md` «다음»에 «`smoke_gate` 를 기본값과 `WIE_AOT=off` 로 나란히»를 적었다.
- 스윕 래퍼는 스크래치에 뒀다(0609 ~ 0611 과 같은 꼴 — 줄기를 나누는 것 · 임대를 잡는 것 · 러너 호출을 세는 껍데기 · `scripts/smoke_gate.sh` 는 그대로 불렀다).
  임대는 (빌드, 팔, 플랫폼)마다 1건이고 스윕 전체를 한 임대로 감싼 것은 0 이다. 처음 여섯 줄기(디버그 LGT)는 0611 과 같은 «세 조각» 드라이버로 돌다가, long 풀이
  한 칸이라 임대마다 5분씩 기다리는 것을 보고 KTF 를 임대 1건(13 ~ 17분)으로 묶은 드라이버로 바꿨다(바꿀 때 도는 실행은 없었다).
- `Cargo.toml` 둘의 주석은 고치지 않았다(§6). upstream 에 PR 을 보내지 않았다. 머지 0 · force-push 0 · 게임 파일 무수정.
