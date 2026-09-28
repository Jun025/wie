## [2026-09-28] LGT MC_knlGetFreeMemory/GetTotalMemory = 4MiB 고정 — «메모리 부족» 안내 해소 · KTF 는 1MiB 유지 (wie-2026-09-27-wie-census-lgt-first-run-notice-unk13-0x68-adopt-p1)

**무엇을**: `wie-lgt` WIPIC `0x78`·`0x79`(`GetTotalMemory`·`GetFreeMemory`)를 공유 커널 스텁 대신 LGT 전용 `get_memory`(둘 다 `0x400000`)로 보낸다.
공유 `wie-wipi-c` 스텁(1MiB)은 그대로 — KTF 경로는 코드상 불변. 시험 1개(`wipic_memory_queries_clear_the_low_memory_notice`).

**왜**: `34c48bbae783`(LGT)가 남은 메모리 ≤ 1,500,000(`0x16e360`)이면 «메모리 부족 — 단말기 리셋» 후 종료한다(0325 표). 스텁은 항상 1MiB.

**사용자 영향**: 그 타이틀이 첫 키에서 끝나지 않고 게임으로 들어간다(아래). 다른 LGT·KTF 타이틀 판정 변화 없음.

### 결정 — 실제 힙 잔량이 아니라 상수, 그리고 «LGT 만»

1. **실제 힙 잔량을 답하지 않는다**: 에뮬레이터 힙은 `HEAP_SIZE` 256MiB(`wie-core-arm/src/core.rs`)라 잔량은 ~128MiB 대 — 실기 값이 아니고,
   할당 이력에 따라 달라져 재현성이 없다. 아래 KTF 사례처럼 잔량으로 풀 크기를 정하는 타이틀이 있어 큰 값은 그 자체가 위험하다.
2. **공유 상수를 올리지 않는다 — KTF 가 실측으로 깨진다**: `4b8c8f5ff7d6`(KTF)은 부팅 때 `GetFreeMemory` 1회 → `MC_knlCalloc(free − 100KiB)`로
   자체 풀을 잡는다. 값별 실측(같은 바이너리 · 환경변수로 값만 바꾼 임시 빌드 · 커밋 안 함):

   | 답한 값 | 결과 |
   |---|---|
   | 500,000 · 900,000 · 1,000,000 · 1MiB · 1,060,000 · 1,100,000 | PASS |
   | 1,200,000 · 1,310,720 · 1,500,001 · 1,572,864 · 1,835,008 · 2MiB · 4MiB · 8MiB | 부팅 중 `Invalid memory access; address: 0`(KtfClassLoader::loadClass 직후) |

   ⇒ 이 타이틀은 풀 ≤ 1MiB(답 ≲ 1.15MB)를 전제한다. LGT 임계(> 1,500,000)와 **겹치는 값이 없다** — 공유 상수로는 둘 다 만족 불가.
   원인은 게임 쪽 전제로 보고 파고들지 않았다(엔진이 KTF 에 1MiB 를 계속 답하면 닿지 않는다).
3. **LGT 값 = 4MiB**: 실기 사양 근거는 로컬에 없다(명시). 근거는 LGT 호출 타이틀 30종의 값 스윕(아래) — 2·4·8MiB 모두
   `34c48bbae783` 외 판정 변화 없음 · 새 오류 0. 관측된 LGT 임계는 1,500,000 하나이고, 4MiB 는 그 위에서 스윕이 무변화를 보인 구간의 가운데다.
   Total 도 같은 값 — free ≤ total 을 지키려고(스윕도 둘을 같이 바꿔 쟀다).

### 호출 타이틀 — 전수 «전» 실행에서 스텁 경고 계수

KTF+LGT 코퍼스 중복 제거 344 sha(`game_lab/working|broken/{ktf,lgt}`) 전부 `--inject` 로 돌려 `RUST_LOG=wie_wipi_c::api::kernel=warn` 로 셌다:
두 함수 중 하나라도 부른 것 **77**(KTF 46 · LGT 30 · 해석 불가 1 = `a23f3c9fc2cb` 기존 스택 넘침 abort) · Total 을 부른 것 26.
나머지 267 은 두 함수를 부르지 않는다 — 그 실행은 첫 호출 전까지 같으므로 이 변경이 닿을 수 없다(아래 전수 «후» 로도 확인).

### 값 스윕 — 호출 77종 × {1,2,4,8}MiB(free=total) · 판정 범주(오류/종료/빈 화면/진행)가 바뀐 것

| sha12 | 플랫폼 | 전(1MiB) | 1MiB 재측 | 2MiB | 4MiB | 8MiB |
|---|---|---|---|---|---|---|
| `34c48bbae783` | lgt | 종료@1 | 종료@1 | 진행 | 진행 | 진행 |
| `4b8c8f5ff7d6` | ktf | 진행 | 진행 | 부팅 오류 | 부팅 오류 | 부팅 오류 |
| `320a5360a0f3` | lgt | 종료@15 | 종료@15 | 종료@15 | 진행 | 진행 |
| `601556233e71` | lgt | 종료@15 | 진행 | 진행 | 진행 | 진행 |
| `30c7bd6fb01b` | ktf | 진행 | 오류@8 | 오류@12 | 오류@10 | 오류@12 |
| `a20c2044305c` | ktf | 진행 | 진행 | 진행 | 종료@15 | 진행 |
| `e159b9727dbd` | ktf | 종료@10 | 진행 | 진행 | 진행 | 진행 |
| `ab3d0020d7bc` | ktf | 빈 화면 | 빈 화면 | 빈 화면 | 빈 화면 | 종료@8 |

- `320a5360a0f3` 는 0325 가 적은 «키 순환이 종료 메뉴에 닿느냐» 흔들림 타이틀이다. 1/2/4MiB 각 3회 재측: 1MiB 는 max-ticks 1·종료@15 2,
  2·4MiB 는 전부 max-ticks@13 — 15키째에 닿기 전에 끝나 판정 불가. 메모리 탓이라 주장하지 않는다.
- `601556233e71` `e159b9727dbd` 는 같은 1MiB 끼리도 바뀐다 = 부하 흔들림. `30c7bd6fb01b` 오류는 전 값에서 같은 오류 · 키 번호만 흔들림.
- 나머지 변화(키 개수 · max-ticks ↔ deadline)는 판정 범주를 바꾸지 않는 부하 흔들림이다(측정 load1 30–160).

### 전수 전/후 — 344 sha · 전 = `origin/main 419dd2d1` · 후 = 이 브랜치

범주가 바뀐 것 24건(KTF 14 · LGT 10).
- **KTF 14건은 전부 잡음이다 — 구성상**: KTF 크레이트는 `wie-lgt` 에 의존하지 않고 이 변경은 `wie-lgt` 한 파일이다. 즉 KTF 14건은
  같은 코드에서 이 시각의 부하가 만드는 흔들림 크기(잡음 바닥)다.
- LGT 10건: `34c48bbae783` 종료@1 → 진행(의도한 변화) · 나머지 9건 중 4건(`236c7da689f6` `2a57e33133b5` `2dbde9acca99` `b475b6399684`)은
  두 함수를 부르지 않는 타이틀(닿을 수 없음) · 부르는 5건 중 `320a5360a0f3` `601556233e71` 은 종료 → 진행(위 스윕 절의 흔들림 타이틀),
  `1eaa92092bee` 는 같은 panic 의 키 번호만 · `4fdbd64c9fbd` 는 마지막 프레임 흔들림 · `caf9d76ffd13`(진행 → 종료@14)은 재측:
  전·후 각 3회 = 전 {종료@14, max-ticks, max-ticks} · 후 {max-ticks, 종료@14, max-ticks} — 같은 분포.
  부르지 않는 쪽의 가장 큰 변화 `2a57e33133b5`(진행 → panic@6)도 전·후 각 3회 전부 PASS 27키.
- ⇒ **회귀 0**. 측정 load1 19–160 — 조용한 호스트가 아니었으므로 판정 범주와 반복 재측만 인용한다.

### 대상 타이틀 — `34c48bbae783`

| | `--inject --relaunch 1` |
|---|---|
| 전 | UNMEASURED · clean exit · 2/27키 · 재시작 1 · distinct 27 |
| 후 | **PASS** · deadline · 27/27키 · 재시작 0 · distinct 359 · paints 2783 |

### 게이트
fmt · clippy `-D warnings`(stable · beta · wasm32) · `RUST_MIN_STACK=4194304 cargo test --all`(545 passed · 0 failed) · `npm run build:wasm` rc0 ·
러너 블록: draw_j2me/helloworld_ktf/helloworld_lgt/text_j2me PASS · keydraw_ktf PASS rc0 · keydraw_lgt 기본 `--max-ticks` 로는 전·후 둘 다
UNMEASURED(max-ticks · load 147) → `--max-ticks 500000000` 로 전·후 둘 다 PASS rc0 · last_frame_content true.

### 되돌리면 red
`LGT_MEMORY` 를 `0x100000` 으로 → `wipic_memory_queries_clear_the_low_memory_notice` FAILED «free 1048576 would show the low-memory notice».

### 게임명 유입
`node scripts/corpus-name-inflow.mjs`(대상 4파일): BOUNDED **11회/6쌍** · SUFFIX-ATTACHED **0**. 11회는 전부 `wie-lgt/src/runtime/wipi_c.rs`
의 **기존 줄**이다 — `git diff origin/main` 의 추가 줄에서 같은 이름을 세면 0. 이 회차가 새로 들인 게임명은 0, 타이틀은 sha12 로 적었다.

<!-- corpus-name-inflow v1 subjects=4 tree=55f5b0e4d498ec89 B=11/6 P=0/0 S=0/0 -->
