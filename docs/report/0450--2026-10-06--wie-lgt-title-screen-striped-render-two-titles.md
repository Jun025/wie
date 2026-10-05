## [2026-10-06] LGT 타이틀 화면 줄무늬 = 32bpp 이미지를 게임이 RGB565 로 읽었다 — 불투명 이미지는 16bpp 로 저장 (wie-lgt-title-screen-striped-render-two-titles)

**무엇을**: LGT 의 `MC_grpCreateImage` 를 LGT 래퍼로 바꿨다. 공용 디코더가 만든 이미지가 불투명하면 화소를 RGB565(16bpp)로 다시 저장한다. 투명 화소가 하나라도 있으면 종전처럼 32bpp ARGB 로 둔다.
**왜**: 진도 3차 1회차(`docs/report/0442` · worklog `2026-10-05-progression-wave3-r1#p0`)가 `e68b1c8aef85` · `b9bfcaf42722` 의 타이틀이 줄무늬로 깨진 것을 봤다.
**사용자 영향**: 이 두 타이틀의 타이틀 화면이 제대로 보인다. 같은 원인이던 LGT 3종(`01eef742dd0a` · `1eaa92092bee` · `863b8ab6a21d`)의 화면도 함께 고쳐졌다.

### 1. 원인 계급 판정 — 화면 버퍼다. 이미지 형식이 아니다

- 증상: 타이틀 그림의 직사각형 조각 안에서 **한 열 걸러 한 열**이 쓰레기다. 짝수 열 값이 두 칸씩 반복되고, 홀수 열은 빨강이 높은 값(`#ffc26b` · `#d6baa5` 류)이다. 대화상자·글자는 멀쩡하다.
- 두 타이틀의 이미지 원본은 형식이 다르다(한쪽은 BMP 404개, 다른 쪽은 자체 형식). 그런데 증상이 같다 ⇒ 공통 층을 봤다.
- 로그(`RUST_LOG=wie_wipi_c::api::graphics=debug`): 두 타이틀 다 `MC_grpCreateImage` 로 이미지를 만들고, `MC_grpGetImageFrameBuffer(X)` 다음 `MC_GRP_GET_FRAME_BUFFER_POINTER(X)` 로 **화소 메모리를 직접** 받는다(`b9bfcaf42722` 25초에 각 15,359 · 36,169회).
- LGT 의 `MC_GRP_GET_FRAME_BUFFER_BPP` 는 인자와 무관하게 16 을 돌려준다(`docs/report/0266`). 게임은 이 값으로 모든 프레임버퍼를 RGB565 로 읽는다.
- 공용 `create_image` 는 디코드한 이미지를 **32bpp ARGB** 로 저장한다. RGB565 로 읽으면 한 화소가 두 칸이 된다: 낮은 반쪽(G·B) 한 칸, 높은 반쪽(`0xffRR`) 한 칸. 홀수 열이 빨강이 높은 값인 것이 이 때문이다.
- ⇒ **계급: 화면 버퍼(이미지 프레임버퍼의 깊이).** 이미지 디코드 자체는 맞다.
- 처음에는 `wie-lgt` 의 `graphics::create_image` 를 고쳤으나 그 함수는 배선되지 않은 함수였다(`allow(dead_code)` 모듈). 실측 배선은 `wipi_c.rs` 의 `WIPICSvcId::CreateImage => wie_wipi_c::api::graphics::create_image` 였다. 그쪽을 바꿨다.

### 2. 수정

- `wie-lgt/src/runtime/wipi_c.rs` `create_image`: 공용 `create_image` 를 부른 뒤, 결과 이미지가 32bpp 이고 모든 화소의 α 가 0xff 이면 RGB565 로 다시 쓰고 32bpp 버퍼를 해제한다.
- 불투명만인 이유: 공용 그리기 경로(`MC_grpDrawImage`)가 투명 이미지의 α 를 쓴다. 불투명 이미지는 16bpp 화면에 그려질 때 어차피 565 로 잘리므로 잃는 것이 없다.
- KTF·공용 디코더는 건드리지 않았다(LGT 행만).

### 3. 짝 측정 — base `9f8b0ae2`(= `origin/main`) ↔ head, release `wie_validate`

줄무늬 점수 = 화면에서 «오른쪽 이웃과 크게 다르고(RGB 차 합 > 96) 두 칸 옆과는 거의 같은(< 24)» 화소의 비율. 시계열 화면(2초 간격) 중 최댓값.

| 타이틀 | 조건 | base | head | 판정 · paints (base → head) |
|---|---|---|---|---|
| `e68b1c8aef85` | 키 `WAIT:3 OK:3×6 WAIT:3` · 40초 | 51.5% (12/19장 ≥5%) | **2.1%** (0/19) | PASS → PASS · 658 → 693 |
| `b9bfcaf42722` | 같음 | 68.2% (2/19) | **1.6%** (0/19) | PASS → PASS · 1908 → 1916 |
| `1eaa92092bee` | 기본 27키 | 85.0% (6/9) | **0.9%** (0/9) | PASS → PASS · 500 → 500 |
| `863b8ab6a21d` | 같음 | 51.0% (3/9) | **1.7%** (0/9) | PASS → PASS · 401 → 410 |
| `01eef742dd0a` | 같음 | 20.9% (1/9) | **2.1%** (0/9) | PASS → PASS · 2071 → 2724 |
| `0baceb1dfe9c` | 같음 | 13.8% (4/9) | 13.7% (4/9) | 그대로 — 남김(§4) |
| `a23f3c9fc2cb` | 같음 | 3.0% | 2.1% | 차이 없음 |

- 두 빌드는 같은 순간에 나란히 돌렸다(타이틀마다 2개). 화면은 사람 눈으로도 봤다. 위 4종 다 타이틀·본편 그림이 온전히 보인다.
- 짝 측정 시작 때 `host-load-guard --status --recovered` 는 rc=1 이었다. 그래서 동시 실행을 2개(base+head)로 줄였다. load1 은 32~36 이었다.
- 화면 캡처와 수치: `~/orchestrator/reports/evidence/wie-lgt-title-screen-striped-render-two-titles/`(레포 밖 · 게임 화면이라 커밋하지 않는다).

### 4. 같은 형상의 표본 수 — 로컬 LGT 78종(고유 sha)

head 빌드 · `--inject`(기본 27키) · `--max-ticks 4000000000` · graphics 디버그 로그. 이미지 핸들 `X` 로 `MC_grpGetImageFrameBuffer(X)` 와 `MC_GRP_GET_FRAME_BUFFER_POINTER(X)` 가 둘 다 불리면 «게임이 이미지 화소를 직접 읽는다»로 셌다.

| 항목 | 수 |
|---|---|
| 잰 타이틀 | 78 (PASS 63 · clean exit 14 · FAIL 1) |
| `MC_grpCreateImage` 를 부른 타이틀 | 19 |
| 그중 불투명 이미지가 있어 16bpp 로 바뀐 타이틀 | 17 |
| **이미지 화소를 직접 읽는 타이틀** | **8** |
| 그중 직접 읽는 이미지가 16bpp 로 바뀐 타이틀(이 수정이 닿는다) | 7 |
| 투명(32bpp) 이미지를 직접 읽는 타이틀(남는다) | 3 — `0baceb1dfe9c` · `a23f3c9fc2cb` · `63332c51d514` |

- 첫 스윕은 29종이 기본 `--max-ticks` 에서 키가 들어가기 전에 멈췄다(UNMEASURED). 그 29종을 tick 상한을 올려 다시 쟀다(28 PASS · 1 clean exit). 표는 다시 잰 값이다.
- 직접 읽는 7종 중 눈에 띄게 바뀐 것은 5종이다(§3). `a23f3c9fc2cb` 는 줄무늬 점수가 원래 낮았다(3.0%). 직접 읽는 이미지 중 투명한 것이 42개, 불투명한 것이 29개다.
- **남김**: 투명 이미지를 직접 읽는 3종. `0baceb1dfe9c` 는 메뉴 배경 귀퉁이가 여전히 줄무늬다. LGT 단말이 투명 이미지를 어떤 형식(16bpp + 마스크?)으로 넘기는지 모르는 채 바꾸면 «안 깨지게만» 하는 처방이 된다. 그래서 손대지 않았다(worklog 제안).
- compat 의 `render` 축은 이 7종 모두 이미 `ok` 였다. 이 축은 줄무늬를 보지 못한다. 그래서 `compat.json` 은 바꾸지 않았다.
- 표본은 로컬 `game_lab/` 의 LGT 뿐이다.

### 5. 검증

- 개악 red: `wipic_create_image_stores_opaque_images_as_rgb565`. 배선을 공용 `create_image` 로 되돌리면 `left: (32, 8)` · `right: (16, 4)` 로 FAILED, 복원하면 ok.
- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm32 clippy · `cargo +beta clippy --all -- -D warnings` 모두 rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0. 처음 두 번은 rustc ICE 로 rc=101 이었다. `target/debug/incremental/wie_core_arm-*` 의 LTO bitcode 파일이 빌드 도중 사라졌다(코드 실패가 아니다). `CARGO_INCREMENTAL=0` 으로 돌려 통과했다.
- 러너 블록 6줄 PASS. `keydraw_ktf`/`keydraw_lgt` 는 `--inject --expect-last-frame` 에서 rc=0 · paints 79/55.
- 스윕 2번과 짝 측정 묶음은 각각 `build-slot run --long` 임대 1개로 감쌌다. 첫 스윕은 long 슬롯 대기가 상한 1800초를 넘어 `over=1` 로 돌았다(다른 레인이 2시간 넘게 쥐고 있었다). 회차 밖 프로세스는 0이다.
