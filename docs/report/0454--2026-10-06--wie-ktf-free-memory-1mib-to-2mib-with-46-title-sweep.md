## [2026-10-06] KTF 남은 메모리 답 1MiB → 2MiB — 호출 49종 전/후 회귀 0 (wie-ktf-free-memory-1mib-to-2mib-with-46-title-sweep)

**무엇을**: 공유 `wie-wipi-c` 의 `MC_knlGetFreeMemory`·`MC_knlGetTotalMemory` 고정 답을 `0x100000`(1MiB) → `0x200000`(2MiB)로 올렸다(상수 `KTF_MEMORY` 하나).
이 두 스텁을 쓰는 것은 KTF 뿐이다 — LGT 는 0341 이후 `wie-lgt` 의 `get_memory`(4MiB)로 답한다. 시험 1개(`test_free_memory_clears_the_ktf_low_memory_notice`).

**왜**: `4b8c8f5ff7d6`(KTF)이 1MiB 를 받으면 부팅 직후 «메모리가 부족합니다. 팝업어플 등을 종료 후 재실행» 한 장에서 멈춘다(0453 §3). 제안 `2026-10-06-input-none-17-recheck#p0`.

**사용자 영향**: 그 타이틀이 안내 없이 로고·타이틀 화면으로 들어간다. 호출하는 다른 KTF 48종은 판정 변화 없음(아래).

### 1. 원인 계급 — 게임의 남은 메모리 검사 · 엔진 답이 너무 작다
- 재현: `RUST_LOG=wie_wipi_c::api::kernel=warn wie_validate --inject --keep-timeout --timeout 30 --max-ticks 100000000000 --shotdir <d> --shot-every 2 <file>`(release · 30초).
- 전(`544a0643`): 마지막 화면 = 위 안내 문구 · 다른 그림 **1**(1·2회 모두). 후: 타이틀 화면 · 다른 그림 **14 · 16**. 스텁 호출 1회(부팅).
- 임계는 식별하지 않았다 — 0453 실측으로 1,048,576 은 안내, 1,310,720 이상은 진행이므로 (1MiB, 1.25MiB] 안이다. 2MiB 는 그 위이고 LGT 4MiB 보다 아래다.
- **0341 과 정반대인 점**: 0341(`419dd2d1`)은 같은 타이틀이 1.2MB 이상이면 부팅 중 `Invalid memory access; address: 0` 이라 공유 값을 1MiB 에 묶었다. `544a0643` 에서는 2MiB 4회 전부 그 오류가 없다.
  그 사이 어느 변경이 그 오류를 없앴는지는 찾지 않았다(남김).

### 2. 호출 타이틀 — 전 실행에서 스텁 경고 계수
KTF 코퍼스 중복 제거 **266** sha(`game_lab/{working,broken}/ktf`) 전부를 전(base) 바이너리로 돌려 `stub MC_knlGet*` 경고를 셌다: **49** 종이 부른다(0341 은 46 — 같은 축, 그 뒤 더 깊이 가는 타이틀이 늘었다).
나머지 217 종은 부르지 않으므로 이 변경이 닿을 수 없다.

### 3. 전/후 — 49종 × base↔head(같은 인자 · build-slot `--long` 임대 안 · 동시 3)
판정 범주(오류/종료/빈 화면/진행)가 같은 것 **47/49**. 범주가 바뀐 2종과 그림 수가 크게 바뀐 2종을 짝으로 다시 쟀다:

| sha12 | 1회 전 → 후 | 재측(전 / 후) | 판정 |
|---|---|---|---|
| `4b8c8f5ff7d6` | 진행·그림 1 → 진행·그림 14 | 2회: 그림 1 / 16 | **의도한 변화** |
| `30c7bd6fb01b` | 진행 → 종료@6(load1 71) | 3·4회: 진행·진행 / 진행·진행 · 2회: 진행 / 진행 | 흔들림(0341 도 이 타이틀을 흔들림으로 적었다) — 후 4회 중 1회 |
| `db8ef04a6504` | 진행 → 종료@12 | 3·4회: 종료@12·종료@12 / 종료@12·종료@15 | 흔들림 — 전에서도 종료@12 가 난다(키 순환이 종료 메뉴에 닿는 타이틀) |
| `d5e996a53118` | 진행·그림 13 → 30 | 5·6회: 35·33 / 31·31 | 흔들림 — 전 1회만 메뉴에 머물렀다 |

0341 이 값에 따라 흔들렸다고 적은 나머지 둘(짝 2회): `a20c2044305c` 진행 27키·그림 18 = 전·후 2회 모두 같다 · `ab3d0020d7bc` 종료@8·그림 11 = 전·후 2회 모두 같다.
⇒ **회귀 0**. 측정 load1: 전 1회 12–331 · 후 1회 21–73 · 재측 13–27 — 조용한 호스트가 아니라 범주와 반복 재측만 인용한다.
화면·JSON: `~/scratch/kfm/out/{base,head}-{1..6}/`(로컬 · 게임 이름이 든 파일명이라 커밋 안 함).

### 4. compat — 이번 회차는 바꾸지 않았다
`4b8c8f5ff7d6` 행(limited · input no · longplay unknown)은 다음 census 가 다시 잰다. 이 회차의 census 실행은 다른 레인의 census(호스트 잠금)를 기다리느라 build-slot long 임대를 붙잡아 중단했다 — 손으로 축을 적지 않았다.

### 5. 게이트
fmt · clippy `-D warnings`(stable · wasm32 · beta) rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(707 passed · 0 failed).
러너 블록: draw_j2me/helloworld_ktf/helloworld_lgt/text_j2me PASS · keydraw_ktf/keydraw_lgt `--inject --expect-last-frame` PASS · deadline(paints 79 · 55).

### 6. 되돌리면 red
`KTF_MEMORY` 를 `0x100000` 으로 → `test_free_memory_clears_the_ktf_low_memory_notice` FAILED «free 1048576 shows a KTF title's low-memory notice».
