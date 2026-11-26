## [2026-11-26] upstream ARM AOT 를 검증 러너에서 켜서 쟀다 — 지금 배선 그대로는 느려지고 선점이 사라진다 · 브라우저 배선은 열지 않는다 (wie-arm-aot-native-executor-on-validate-runner-measure-go-no-go)

**무엇을**: 측정 기록이다. 엔진 · 검증 러너의 코드는 main 과 같다(이 PR 의 `*.rs` · `Cargo.toml` · `Cargo.lock` 차이 0줄).
`wie_validate` 에 `WIE_AOT=native` 스위치(코드 1줄 + 의존 1줄)를 브랜치에 커밋해 끔/켬을 같은 순간 짝으로 쟀고, 그 커밋을 같은 브랜치에서 되돌렸다.

**왜**: `STATE.md` «다음»의 후보 ⑵(총괄 채택 2026-11-26). upstream 의 ARM AOT 는 S3(PR #617)로 트리에 들어왔지만 우리 두 호스트는 끄고 돈다.
`docs/report/0586` §9 가 «AOT 를 켠 실행은 재지 않았다»고 남겼고, 해석기 쪽에 남은 조각 ⒟(L)를 열기 전에 이미 있는 것을 재기로 했다.

**사용자 영향**: 없다(제품 줄 0 · wasm 산출물 불변 · 고지 0건).

**판정: no-go — 브라우저 배선은 지금 열지 않는다.** 켜기만 해서는 쓸 수 없다. 본 것은 이렇다.

| | 끔(해석기) | 켬(upstream 배선 그대로) |
|---|---|---|
| 무거운 장면의 프레임당 실행 루프(LGT `3cc7a9b4cb15` 성 대화 · 짝 3회) | 14.51 ms | **20.09 ms(+38.5%)** — 세 짝 모두 느려짐 |
| 첫 화면까지(같은 판 · 짝 3회) | 0.45 s | **1.33 s** |
| 피크 RSS(〃) | 38 MB | **273 MB** |
| 컴파일된 코드 안의 선점 | 10,000 명령마다 내준다 | **내주지 않는다** — 붙든 스레드 · 선점 시험 11건 중 8건 red |
| 널 페이지 읽기(`ldr r0,[r0]` · r0 = 0) | 0 을 읽는다 | **`Invalid memory access; address: 0`** 로 끝난다 |
| `smoke_gate` KTF · LGT | (기준 목록) | 244판 244 PASS · 퇴행 0 |

**그런데 천장은 높다.** 느려지는 원인은 한 자리(버린 영역 목록을 매번 훑는 줄)이고, 그 줄을 건너뛰게 한 스크래치 변형은 같은 장면이
**1.70 ms(−88.3%)** 였다(§4). 그래서 다음 조각으로는 ⒟ 가 아니라 «네이티브에서 AOT 를 우리 규칙에 맞추는 일»을 권한다(§7).

### 1. 만든 것 — 측정용 스위치 (되돌린 커밋)

```rust
// wie_cli/src/bin/wie_validate.rs · build_emulator
aot: (std::env::var("WIE_AOT").as_deref() == Ok("native")).then(|| Box::new(wie_core_arm_native::NativeExecutor::new(None)) as _),
```

```toml
# wie_cli/Cargo.toml
wie-core-arm-native = { workspace = true }
```

- 환경 변수가 없으면 종전과 같은 `aot: None` 이다. 같은 바이너리 하나로 두 팔을 돌렸다(빌드 차이가 측정에 끼지 않는다).
  스위치를 환경 변수로 둔 것은 이 파일의 선례(`WIE_HOOK_MODE`)와 `smoke_gate.sh` 가 인자를 넘기지 않아서다.
- 실행기는 upstream 의 `NativeExecutor`(cranelift)이고 **지속 캐시는 붙이지 않았다**(`new(None)`) — 실행마다 컴파일 비용을 전부 낸다.
  upstream 의 CLI(`src/cli.rs`)는 sqlite 캐시를 붙인다. 두 번째 실행부터의 값은 이 회차에 없다.
- **약한 곳으로 적혀 있던 cranelift 의존은 문제가 아니었다.** `wie_validate` 는 이미 cranelift 를 링크하고 있다
  (`cargo tree -p wie_cli -i cranelift-jit` → `wie-core-arm-native` ← `test-utils` ← `wie_cli`). 더한 한 줄은 간선 하나이고
  `Cargo.lock` 은 그 한 줄만 는다(패키지 추가 0). wasm lint 게이트는 루트 패키지만 빌드하므로 닿지 않는다. 스위치를 얹은 채 네 게이트 + beta 전부 rc 0(§8).
- **넣지 않는다.** 판정이 no-go 이고, `**/*.rs` · `**/Cargo.toml` · `Cargo.lock` 은 발행 경로 필터에 있어 착지하면 wasm 이 같은 채로
  릴리스가 한 번 선다. 다음 조각이 채택되면 엔진을 고치는 그 PR 이 위 두 줄을 같이 싣는다.

### 2. 끔/켬 — 같은 순간 짝 3회 (할 일 0 · 2)

자는 0603 §1 그대로다: `scripts/arm-interp-profile.sh` · LGT `3cc7a9b4cb15` 성 대화 첫 쪽(`docs/keys/lgt-3cc7a9b4cb15-intro-castle.keys`) ·
`xctrace` Time Profiler 8초 창(`<at>` 130) · «창 안의 루프 표본 ÷ 프레임 수». 바이너리 = main `e01999434` + 스위치(release + 줄 표).
끔 → 켬 → 변형(§4) 순서로 3바퀴 · 9회 · 실행 1회마다 long 임대 1건.

| | 짝 1 | 짝 2 | 짝 3 | 평균 |
|---|---|---|---|---|
| 시작 때 load1(끔 · 켬 · 변형) | 5.0 · 4.1 · 6.3 | 6.5 · 10.9 · 8.6 | 9.6 · 7.3 · 7.5 | |
| 창 안 프레임 수 | 157 · 156 · 156 | 156 · 153 · 158 | 152 · 154 · 153 | |
| **끔(해석기)** | 14.28 | 14.87 | 14.37 | **14.51 ms** |
| **켬** | 19.82(+38.8%) | 20.10(+35.2%) | 20.36(+41.7%) | **20.09 ms(+38.5%)** |
| 켬에서 루프 표본 중 `Aot::recompile` 의 훑기(`aot.rs:122` · `:125`) | 90.1% | 90.8% | 89.0% | |
| **변형(§4 — 그 훑기만 건너뜀)** | 1.42(−90.1%) | 2.04(−86.3%) | 1.64(−88.6%) | **1.70 ms(−88.3%)** |

- 끔의 값은 0603 의 «가름» 15.12 ms 와 같은 자리다(그 변경이 착지한 main).
- 9회 전부 같은 화면이었다 — 프로세스 130초 · 140초 캡처가 끔 1회차와 98.3 ~ 99.6% 같은 바이트다(인트로 화면은 38.2%).
- 켬 · 변형에서 컴파일된 코드의 표본은 루프 안으로 센다. JIT 프레임은 줄 정보가 없어 그것을 부른 줄
  (`wie-core-arm-native/src/lib.rs:208`)에 붙고, 그 위에 `run` 이 있다. 변형의 루프 표본 중 53 ~ 62% 가 그 줄이다.

**첫 화면 · 메모리**(같은 판 · 키 없이 25초 · `/usr/bin/time -l` · 끔 → 켬 3짝 · load1 2.9 ~ 4.4):

| | 끔 | 켬 |
|---|---|---|
| 첫 `MC_grpFlushLcd` 까지 | 0.48 · 0.45 · 0.42 s | 1.34 · 1.30 · 1.36 s |
| 〃 그중 «AOT installed» 까지(컴파일을 기다린 시간) | — | 0.51 · 0.50 · 0.55 s |
| 피크 RSS | 38 · 38 · 38 MB | 280 · 271 · 268 MB |
| CPU(user · 50M 틱에서 멈춘 약 12초) | 11.88 · 11.92 · 11.70 s | 17.25 · 16.81 · 16.68 s |

**이 저장소의 속도 감시 자**(`perf-watch.mjs run --axis native` · 50M 틱당 CPU ms · A = 끔 · B = 같은 바이너리에 스위치를 켠 래퍼 ·
표본마다 A 와 B 가 같은 순간에 돈다 · 3회 · ARM 표본 넷):

| 표본 | 회별 느려짐 % | 비고 |
|---|---|---|
| KTF `1065985081fc` | +40.3 · +40.4 · +42.4 | |
| KTF `155972cac664` | +33.9 · +31.1 · +34.1 | 그리기 수 359 → 328 · 467 → 421 · 356 → 326 |
| LGT `0606d43702e7` | +17.5 · +17.4 · +19.8 | |
| LGT `3cc7a9b4cb15` | +44.7 · +20.9 · +44.6 | |

- 넷 다 주간 감시의 문턱(5%)을 넘는다. 이 축은 프로세스 전체의 CPU 이고 창이 부팅 12초라, 컴파일 스레드(rayon)가 쓴 약 5초가 그대로 실린다 —
  «부팅 직후의 값»이지 장면의 값이 아니다. 장면의 값은 위 프로파일 표와 §3 의 KTF 표다.

### 3. 왜 느려지나 — LGT 는 명령 캐시를 통째로 비우고, 그 뒤로는 놓칠 때마다 버린 목록을 훑는다

스크래치 진단 빌드(로그 몇 줄 — §9)로 같은 판을 150초 돌렸다.

1. 부팅 0.5초에 3,864 영역(진입 PC 249,706개)이 컴파일돼 깔린다.
2. **4 ms 뒤 게스트가 명령 캐시를 통째로 비운다.** `pc=0x5dbec` 의 `0xee070f15`(`mcr p15, 0, r0, c7, c5, 0` — I-cache 전체 무효화)이고
   부르는 자리는 하나다(`lr=0x5e185` · 이미지 안). 엔진은 그 명령을 `0..4GB` 무효화로 받는다(`arm32_cpu.rs` 의
   `InstructionCacheInvalidation::All`) — 3,864 영역이 전부 «버린 목록»으로 간다. 150초 동안 70번(부팅 2초에 53번 · 그 뒤 장면이 바뀔 때).
3. 버린 영역은 그 PC 에 다시 들어올 때 하나씩 다시 컴파일된다. 성 대화에서 살아 있는 것은 **464 영역**(진입 27,390개)이고 버린 목록에 3,526개가 남아 있다.
4. **진입 PC 가 표에 없을 때마다 `Aot::recompile` 이 그 3,526개를 처음부터 끝까지 훑는다**(`wie-core-arm/src/aot.rs:121-128` · 선형 `position`).
   컴파일될 수 없는 주소(`0x7100xxxx` — SVC 스텁)는 영영 표에 없으니 호스트 호출마다 훑는다: 126초에 419만 번. 켬의 루프 표본 90% 가 이 두 줄이다.

- **LGT 는 전부 그렇다.** `game_lab/working/lgt` 54개 파일을 12초씩 돌려 셌다 — **54개 전부**가 명령 캐시를 비운다
  (판당 21 ~ 55번 · 중앙값 36 · 판마다 부르는 PC 는 하나).
- **KTF 는 비우지 않는다**: `game_lab/working/ktf` 190개 중 다섯에 하나(38개)를 같은 식으로 — **38개 전부 0번**. §6 ⑵ 의 패치된 KTF 두 판도 0번이다.
- upstream 에서 이 훑기가 싼 것은 버린 목록이 보통 비어 있어서일 것이다(재지 않았다 — upstream 의 LGT 길은 우리와 다르다: 0586 부록 B).

**KTF 의 장면 값**(같은 자 · KTF `155972cac664` · 키 없이 부팅 화면 · 프로세스 30초부터 8초 창 · 프레임 표지 = `MC_grpFlushLcd` · 끔 → 켬 3짝):

| | 짝 1 | 짝 2 | 짝 3 | 평균 |
|---|---|---|---|---|
| 창 안 프레임 수(끔 · 켬) | 216 · 212 | 210 · 212 | 211 · 211 | |
| 끔 | 1.48 | 1.48 | 1.53 | **1.50 ms** |
| 켬 | 0.70(−52.7%) | 0.92(−37.8%) | 0.73(−52.3%) | **0.78 ms(−47.7%)** |

- **캐시를 비우지 않는 KTF 에서는 배선 그대로도 빨라진다.** 다만 이 화면은 가볍다 — 안내 창 하나가 떠 있는 정지 화면이고(6회 모두 30초 · 40초
  캡처가 100% 같은 바이트) 루프가 전체 표본의 3.7% 다. KTF 의 무거운 장면은 재지 않았다(키 경로와 프레임 표지가 선 KTF 장면이 이 자에 없다).

### 4. 훑기만 건너뛴 변형 — 천장을 재려고 만든 것 (넣지 않는다)

`Aot` 에 «버린 영역이 덮었던 주소의 범위» 하나를 두고, `recompile` 이 그 범위 밖의 PC 면 훑지 않고 돌아간다(스크래치 패치 +8줄 · §9).
SVC 스텁은 이미지 밖이라 전부 걸러진다. 그것 말고는 켬과 같다 — 캐시를 비울 때마다 영역을 버리고 다시 컴파일하는 것도 그대로다.

- 성 대화 **14.51 → 1.70 ms(−88.3%)** · 세 번 모두 −86 ~ −90%(§2 표). 같은 화면이다(캡처 98.3 ~ 99.6%).
- ⒟(해석기 묶음 실행)의 어림은 «위로 −6.7 ms»였다(0575 §5 · 재지 않은 값). 이 변형이 실제로 걷어낸 것은 12.8 ms 다.
- **이 값을 브라우저로 옮겨 읽지 마라.** 브라우저는 다른 코드 생성기(`wie-core-arm-wasm`)이고 영역마다 JS 를 건너 부른다
  (`wie-web/src/rust/aot.rs` 의 `executeRegion`). 재지 않았다.
- 이 변형도 §6 의 두 자리(선점 · 널 읽기)는 그대로 깨져 있다. 쓸 수 있는 물건이 아니라 «얼마나 남아 있나»를 본 것이다.

### 5. `smoke_gate` — 켠 채로 KTF · LGT (할 일 3)

`scripts/smoke_gate.sh`(뜨고 그린다 · `--timeout 15` · 재시도 2) · `BIN=` 스위치 바이너리 · `WIE_AOT=native` · 플랫폼의 파일을 두 줄기로 나눠
같이 돌렸다(줄기마다 그 몫의 기준 행만 건넨다). 임대 4건(LGT 1 · KTF 3).

| | 돌린 판 · PASS | 기준 대조 |
|---|---|---|
| LGT | 54 · 54 | checked 52 · absent 0 · **regressions 0** |
| KTF | 190 · 190(63 · 64 · 63) | checked 190 · absent 0 · **regressions 0** |

- **끔과 달라진 판 0.** 끔 팔은 같은 순간에 나란히 돌리지 않았다 — 기준 목록(끔으로 만든 것)과 오늘 0603 §6 의 같은 코퍼스 244 PASS 가 그 자리다.
- 이 그물은 부팅 15초의 «뜨고 그린다»만 본다. §6 의 두 자리는 여기서 한 판도 걸리지 않았다 — 걸리지 않는다는 것이 안전하다는 뜻은 아니다.
- 스크립트는 판마다 마지막 판정만 찍는다(재시도로 PASS 가 된 판이 있었는지는 모른다).

### 6. 미리 알던 충돌 자리 세 곳 (할 일 3)

**⑴ 널 페이지 읽기 0 규칙 — 성립하지 않는다(확인).** 스크래치 시험: `ldr r0, [r0]; bx lr` 을 널 페이지가 매핑되지 않은 곳에 올리고 r0 = 0 으로 부른다.

| | 결과 |
|---|---|
| 해석기 | `Ok(0)` |
| AOT | `Err("Invalid memory access; address: 0")` |

컴파일된 코드는 페이지 표를 직접 읽고, 없는 페이지면 `GuestFault` 로 끝난다(`wie-core-arm-native/src/codegen/memory.rs:163`). 0586 §9 가 읽기로
적은 그대로다. ★처음 만든 시험은 틀렸다 — 코드를 `0x1000` 에 올렸더니 두 팔 다 0 을 읽었다. 호스트 페이지가 64 KB 라 그 코드가 널 페이지와
같은 페이지에 실려 «매핑된 0»을 읽은 것이다. 코드를 `0x30000` 으로 옮긴 것이 위 표다.
실제 판(`2b1ed0c8d061` — 0391)의 그 읽기는 긴 키 경로 뒤라 돌리지 않았다.

**⑵ 바이트 패치 · 범용 훅 — 스냅숏 전에 들어가고, 켠 채로 일한다.** 장부의 `binary-patches` 세 행을 진단 빌드로 끔/켬 돌렸다(`--inject` 기본 27키).

| 꾸러미 | 끔 | 켬 | 패치 로그 · 훅 설치 | 잠금 화면 |
|---|---|---|---|---|
| KTF `974e0df9ab1e` | PASS · 27/27 키 | UNMEASURED · 20/27(`max-ticks`) | 3 · 3 = 두 팔 같음 | 두 팔 다 지나감(캡처) |
| KTF `f770b15f8876` | PASS · 27/27 | UNMEASURED · 19/27(`max-ticks`) | 3 · 3 | 〃 |
| LGT `01f05f8231f4` | UNMEASURED · 22/27(`max-ticks`) | PASS · 27/27 | 2 · 0 | 〃 |

- 패치 로그(«Patch applied»)가 «ARM AOT installed» 앞에 찍힌다 — 0586 §3 ⑴ 의 순서 그대로다(KTF `974e0df9ab1e`: 패치 3 · 훅 3 설치 0.8초 뒤
  4,974 영역이 깔린다 / LGT `01f05f8231f4`: 패치 2 의 1.3초 뒤 6,423 영역).
- `UNMEASURED` 는 틱 상한(기본 5천만)에 먼저 닿은 것이다 — 판정이 아니라 `--max-ticks` 를 올리라는 뜻이고, 어느 팔이 먼저 닿는지가 플랫폼마다
  반대다(KTF 는 켬이, LGT 는 끔이 먼저 닿는다). §3 의 방향과 맞지만 — 빨리 도는 팔에서 빈 틱이 빨리 쌓인다 — 그 까닭을 따로 확인하지는 않았다.
- 범용 훅: KTF `974e0df9ab1e` 를 `WIE_HOOK_MODE=shadow` 로 끔/켬(`--max-ticks` 4억 · 둘 다 PASS 27/27). 훅이 켠 채로 불리고 게임 자신의 코드와
  어긋남이 0 이다 — 끔 `memcpy` 366,150 · `strlen` 9,023 · `strcpy` 12 / 켬 175,740 · 4,445 · 8(전부 `mem_mismatch` · `result_mismatch` 0).
  켬의 횟수가 적은 것은 shadow 가 훅 자리를 불릴 때마다 두 번 고쳐 쓰고 그때마다 그 영역을 버리기 때문이다(무효화 360,386번) — 진단 모드의 값이다.

**⑶ `hold_others` · 선점 예산 — 블록 단위가 되는 것이 아니라, 컴파일된 코드에서는 예산이 없다.**
`run` 은 해석한 명령에서만 예산을 센다(`arm32_cpu.rs` 의 `budget_consumed += 1` 은 `step` 뒤 한 곳). 컴파일된 영역은 안에서 뒤로 뛰는 분기를 그대로
돌고(`has_backedge`), 실행기는 다음 영역이 표에 있으면 돌아오지 않고 이어 부른다(`wie-core-arm-native/src/lib.rs:203-228` 의 `loop`).
`RunFrame` 에 예산 칸이 없다.

| 스크래치 시험(AOT 를 붙이고 `prepare_execution` 을 부른 판) | 해석기 | AOT |
|---|---|---|
| 20만 명령 카운트다운 루프 — 끝나기 전에 스레드가 내준 횟수 | 20 | **0** |
| 0598 · 0601 · 0602 의 `core.rs` 시험 11건 | 11 green | **8 red · 3 green** |

- red 8: `yields_every_ten_thousand_instructions_across_svc` · `a_holder_sliced_out_keeps_the_others_off_guest_code` ·
  `another_thread_taking_a_hold_leaves_the_holders_in_place` · `another_thread_letting_go_leaves_the_holders_in_place` ·
  `only_a_thread_sliced_out_mid_code_counts_as_preempted` · `a_budget_that_runs_out_on_an_svc_counts_as_preempted` ·
  `a_thread_kept_off_by_a_holder_still_counts_as_preempted` · `two_sliced_holders_do_not_wait_for_each_other`.
  전부 같은 까닭이다 — «코드 한가운데서 잘린 스레드»가 생기지 않는다(`assert!(….poll(..).is_pending())` 에서 선다).
- green 3 은 긴 게스트 코드를 돌리지 않는 시험이다(`a_holder_parked_in_a_host_call_keeps_nobody_off` · `a_nested_hold_reports_the_outer_one` ·
  `a_thread_that_ends_holding_leaves_no_hold_behind`).
- 0598 의 `wie-wipi-c` · `wie-ktf` · `wie-lgt` 쪽 시험(`a_spawned_callback_*`)은 붙이지 않았다 — 그 시험들이 읽는 것이 위 `core.rs` 의 표시들이다.
- **뜻**: 다른 스레드나 타이머가 값을 바꿔 주기를 기다리며 도는 게스트 루프가 통째로 컴파일되면 그 루프는 내주지 않는다. 브라우저는 스레드가
  하나라 그 동안 탭이 선다. 이 문장은 위 두 관측에서 나온 추론이고, 그런 판을 실제로 보지는 못했다(`smoke_gate` 244판 중 0).

### 7. 판정 · 다음 조각 (할 일 4)

**no-go: 브라우저 배선(`WasmExecutor` · `wie-core-arm-wasm`)은 지금 열지 않는다.** 네이티브에서조차 켜면 LGT 54판이 느려지고(§2 · §3),
선점과 널 읽기 규칙이 깨진 채(§6) 브라우저로 가면 그 둘을 재는 자가 없다. 제품 기본값은 `aot: None` 그대로다.

**그리고 ⒟ 도 지금 열지 않기를 권한다.** 재개 조건(«AOT 를 켠 실행의 수»)은 섰고 그 수가 말하는 것은 이렇다:

| | ⒟ 해석기 묶음 실행 | AOT 를 우리 규칙에 맞추기 |
|---|---|---|
| 성 대화에서 걷어낼 몫 | −6.7 ms(어림 · 0575 §5) | **−12.8 ms(잰 값 · §4 의 변형)** |
| 새로 지어야 하는 것 | PC 키 블록 캐시 · 쓰기 두 길의 무효화 · 블록 단위 예산 · 블록 중간의 SVC · 두 해석기 대조 도구 | 예산 · 널 읽기 · 훑기 — 무효화 · 스냅숏 · 패치 순서 · 훅 구멍은 이미 있고 켠 채로 일한다(§6 ⑵) |
| 가장 어려운 자리 | 블록 단위 예산 | 같은 자리(컴파일된 코드의 예산) |
| 값 | 우리 계층이 하나 는다 | upstream 의 파일을 고친다(병합 때마다 충돌) · 코드 생성기가 둘이다 |

**다음 조각 초안 — `wie-arm-aot-native-fork-rules-budget-null-read-recompile-scan`** (size L · risk high · 네이티브만 · 제품 기본값 무변경)

- **Goal**: 검증 러너에서 AOT 를 켠 실행이 우리 규칙 셋을 지키게 하고, 같은 자로 다시 잰다. 브라우저 배선은 그 수를 보고 따로 연다.
- **할 일**
  1. 스위치 두 줄(§1)을 되살린다.
  2. **훑기**: `Aot::recompile` 이 놓칠 때마다 버린 목록 전체를 보지 않게 한다(§4 의 범위 하나면 SVC 스텁은 걸러진다 — 이미지 안의 구멍까지 걸러야 하는지는 프로파일로 정한다).
  3. **예산**: 컴파일된 코드가 쓴 명령 수가 `budget_consumed` 에 실리고, 예산이 다하면 `Yield` 로 나온다. 두 길을 재서 고른다 —
     ⒜ 코드 생성기가 뒤로 뛰는 분기와 영역 경계에서 예산을 깎는다(생성기를 고친다) / ⒝ 영역에서 뒤로 뛰는 분기를 빼고(`wie-core-arm/src/aot.rs` 의
     정책 상수 쪽) 실행기가 영역을 이어 부르지 않게 해 `run` 이 영역마다 센다(생성기는 안 고친다 · 루프가 느려진다).
  4. **널 읽기**: 널 페이지(< 0x1000 · 매핑 없음)의 읽기가 0 을 읽는다. 폴트를 해석기로 넘기는 길과 영역을 만들 때 가르는 길 중 재서 고른다.
  5. 다시 잰다: §2 의 표(끔 · 켬) · 첫 화면 · 피크 RSS(지속 캐시를 붙인 두 번째 실행 포함) · `perf-watch` 네이티브 축 · `smoke_gate` · KTF 장면 1개.
- **Acceptance**: §6 ⑶ 의 11건과 널 읽기 시험이 AOT 를 붙인 채 green(시험을 두 엔진으로 돌리는 꼴로 넣는다) · 성 대화가 끔보다 빠르다(값을 적는다) ·
  `smoke_gate` 244판 퇴행 0 · `yields_every_ten_thousand_instructions_across_svc` 가 AOT 로도 5,000 · 10,000 에서 내준다.
- **경계**: `wie_featurephone` 의 `aot: None` 무변경 · 브라우저 배선 0 · upstream 파일을 고치는 줄마다 `fork:` 주석 · 게임 파일 무수정.
- **약한 곳**: 3 의 ⒜ 는 코드 생성기 둘(`wie-core-arm-native` 1,079줄 · `wie-core-arm-wasm` 1,733줄) 중 하나만 고치는 것이라, 브라우저로 갈 때 같은 일을
  wasm 쪽에 한 번 더 한다. 크기 L 은 어림이다 — 3 을 ⒝ 로 풀 수 있으면 M 이다.

**그 뒤의 브라우저 배선**(지금 발권하지 않는다 · 범위만): upstream `wie-web/src/rust/aot.rs`(325줄 · `WasmExecutor`) 와 `wie-web/src/ts/arm-compiler.ts`
(140줄 · `WebAssembly.compile` · IndexedDB 캐시)를 `wie_featurephone` 으로 옮긴다. 걸리는 것 — ⑴ `wasm-encoder` 가 산출물에 들어온다(크기)
⑵ `wie_aot_pages` · `wie_aot_word_range` · `wie_aot_resolve` 가 export 표면에 선다(Constraint 3 — 계약 파일을 같은 PR 에서)
⑶ `arm-compiler.ts` 는 번들러 별칭(`@ts/`)으로 들어오는데 우리 빌드는 `wasm-bindgen --target web` 뿐이다(JS 로 옮겨야 한다)
⑷ 실행 중 `WebAssembly.compile` 을 셸의 CSP 가 허락하는지(otterpebble 쪽 · 확인 안 함) ⑸ wasm 코드 생성기에 위 3 · 4 를 한 번 더. 크기 L.

### 8. 게이트

| 게이트 | 스위치를 얹은 트리(되돌리기 전) | 되돌린 뒤 · 문서를 더한 트리 |
|---|---|---|
| `cargo fmt --all -- --check` | 0 | 0 |
| `cargo clippy --all -- -D warnings` | 0 | 0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | 0 | 0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | 0 — `test result: ok` 72줄 · 1054 passed · 0 failed | 0 — 72줄 · 1054 passed · 0 failed |
| `cargo +beta clippy --all -- -D warnings` | 0 | 0 |

- 스위치 쪽은 두 번 돌았다: 할 일 1 직후 main `e01999434` 위에서(전부 rc 0 · 72줄 · 1054 passed · 0 failed), 그리고 main 이 `780842ecb` 로
  움직인 뒤 그 위에서(위 표). PR 의 beta 다리는 빈 통과라 beta 의 그물은 이 로컬 값이다.
- 러너 블록 · wasm 계약은 돌리지 않았다 — 이 PR 이 싣는 엔진 차이가 0줄이다.
- `node scripts/check-worklog-json.mjs` · `node scripts/check-docs-report-serial.mjs` · `bash scripts/audit-no-leak.sh` · `node scripts/check-inflow-marker.mjs`: 전부 OK(494 파일 · 0607 충돌 0 · AUDIT PASSED · 유입 표식은 이 파일 끝).
- 오른쪽 열은 문서 커밋 직전의 작업 트리에서 돌렸다. 그 뒤 더한 것은 이 표의 숫자와 유입 표식 줄뿐이다(코드 0줄).

### 9. 하지 않은 것 · 한계

- **장면은 LGT 1판과 KTF 1판(부팅 화면)** · 기기 1대 · 3바퀴. 측정 내내 다른 레인과 내 게이트의 부하가 있었다(load1 2.9 ~ 12).
- **브라우저(wasm)는 한 번도 돌리지 않았다.** §4 의 천장은 네이티브 cranelift 의 값이다.
- **지속 캐시 없이 쟀다.** 첫 화면 · 피크 RSS · `perf-watch` 의 값은 «처음 여는 판»의 값이다.
- **`smoke_gate` 의 끔 팔을 나란히 돌리지 않았다**(§5). 켬에서 FAIL 이 하나도 없어 끔으로 다시 볼 판이 없었다.
- **스크래치 빌드 넷**: 진단 로그 2종(무효화 수 · 무효화한 PC) · §4 의 변형 · §6 의 시험 패치. 전부 작업 트리에서 지었다가 `git checkout` 으로 되돌렸고
  커밋하지 않았다. 패치 · 표 · 캡처 · 게이트 로그는 저장소 밖 `~/orchestrator/reports/evidence/wie-arm-aot-native-executor-on-validate-runner-measure-go-no-go/` 에 있다
  (2 MB 넘는 원 로그와 `x.trace` 는 지웠다 — 표는 남아 있다).
- §6 ⑶ 의 «탭이 선다»는 추론이다. 통째로 컴파일되는 대기 루프를 가진 판을 찾지 않았다.
- upstream 이 LGT 의 캐시 비우기를 어떻게 겪는지는 보지 않았다(upstream 의 호스트는 실행하지 않았다 — 0586 §9).
- 드라이버를 `scripts/` 에 두지 않았다: 프로파일 · `smoke_gate` · `perf-watch` 는 저장소의 스크립트를 그대로 불렀고, 스크래치에 둔 것은 줄기를 나누는
  래퍼와 로그를 세는 몇 줄이다(0603 §8 과 같은 꼴). 임대는 실행 · phase 마다 1건이다(가장 긴 것 약 8분).
