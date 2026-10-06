## [2026-10-06] LGT 투명 이미지도 RGB565 로 저장 — 게임이 투명색을 스스로 키로 쓴다 · 알파는 8bpp 마스크로 (wie-lgt-transparent-image-direct-read-stripes-three)

**무엇을**: LGT `MC_grpCreateImage` 가 투명 화소가 있는 이미지도 RGB565(16bpp)로 저장한다. 투명 화소는 색(대개 마젠타)을 그대로 두고, 알파는 `WIPICImage.mask` 에 8bpp(화소당 1바이트)로 곁에 둔다. 공용 `MC_grpDrawImage` 와 픽셀-op 블릿이 그 마스크를 적용한다.
**왜**: #483(`docs/report/0450`)이 불투명 이미지만 고치고 남긴 3종(worklog `2026-10-06-lgt-title-screen-stripes#p0`).
**사용자 영향**: `0baceb1dfe9c` 의 메뉴 뒤 줄무늬가 사라졌다. `a23f3c9fc2cb` 는 제작사 로고 세 장이 회색·갈색 덩어리로 나오다가 제대로 나오고, 타이틀 화면까지 간다. `0606d43702e7` 의 타이틀 탱크 위 얼룩 무늬도 사라졌다. `63332c51d514` 는 캐릭터 선택 화면의 초상 다섯 장이 검게 빠지던 것이 다 나온다.

### 1. 원인 계급 — 화면 버퍼(이미지 프레임버퍼의 깊이). #483 과 같은 계급이다

남은 질문은 «투명 이미지를 실기가 어떤 형식으로 넘기나»였다. 세 타이틀의 게스트 코드가 답한다. 모두 **투명도를 스스로 처리**하고, 그 키를 **이미지 화소에서 읽어** 온다.

호출부는 `PROBE`(svc id · lr) 로그로 찾고 `binary.mod` 를 Thumb 로 역어셈블해 읽었다(`objdump --triple=thumbv5te`). 증거: `~/orchestrator/reports/evidence/wie-lgt-transparent-image-direct-read-stripes-three/`(레포 밖).

- **`0baceb1dfe9c`** — 블릿 루틴 `0x259c4`.
  - 전역 `[0x1503d54]` 를 깊이로 쓴다. 이 값은 `MC_GRP_GET_FRAME_BUFFER_BPP` 의 답이다(lr `0x26d95` · 우리 답 16).
  - 32 이면 `ldr` 로 4바이트씩, 아니면 `ldrh` 로 2바이트씩 이미지 화소를 읽는다(`0x25bb4`~`0x25bd6`). 읽은 값이 키 `[0x15038e4]`(16비트)와 같으면 건너뛰고, 아니면 화면에 `strh` 한다.
  - 키는 `0x24ec4` 가 정한다. 키 이미지 하나를 `MC_grpGetImageFrameBuffer` → `MC_GRP_GET_FRAME_BUFFER_POINTER` 로 받아 **첫 화소**를 읽는다. 호출부는 `0x2446a` 이고, 그 이미지는 `0x2b580` 의 경로다. 그 PNG 의 (0,0)은 팔레트 투명 항목 `(255,0,255,α0)` 이다.
  - ⇒ 실기는 투명 화소를 **마젠타의 RGB565(0xf81f)** 로 저장했다. 게임은 그 값을 키로 잡아 건너뛴다. 우리는 32bpp 로 저장했다. 그래서 16비트로 읽으면 화소가 반으로 쪼개져 줄무늬가 나고, 키도 `0x00ff` 가 된다.
- **`a23f3c9fc2cb`** — `0x26686` 이하.
  - 이미지 화소를 RGB565 로 읽는다(`[0x150e565] == 2` 일 때만 32비트 변환 갈래로 간다).
  - `0x26736` 에서 키 `[0x15108e4]` 와 같은 화소를 다른 값으로 바꾼다.
  - 그 뒤 RGB 바이트로 만든 RGB565 팔레트(`0x26654`: `r>>3<<11 | g>>2<<5 | b>>3`)와 맞춰 8비트 색인을 만든다. 32bpp 로 두면 팔레트에 맞는 값이 없다. 그래서 로고가 단색 덩어리였다.
- **`63332c51d514`** — `0x438d8`. 이미지 첫 4바이트에 마스크 `[0x1504378]` 를 AND 한 값을 키로 둔다. head 의 `unk0` 인자가 base `0xff` → head **`0xf81f`** 로 바뀐 것이 로그에 보인다.
- 세 게임의 이미지는 전부 8비트 팔레트 PNG 다. 투명 항목의 알파는 0/255 두 값뿐이다(`0baceb1dfe9c` 1,139장 중 tRNS 912장). 로그에서 투명 화소의 RGB 는 **모두 `0xff00ff`** 였다.

⇒ 형식 근거: 실기 이미지 프레임버퍼는 **16bpp RGB565** 이고, 투명 화소도 **자기 색을 RGB565 로** 갖는다. 투명 처리는 게임 몫이다. 이 회차는 그대로 맞췄다.

### 2. 수정

- `wie-lgt/src/runtime/wipi_c.rs` `create_image`: 32bpp 로 디코드된 이미지를 전부 RGB565 로 다시 쓴다. 알파가 0xff 가 아닌 화소가 있으면 같은 크기의 8bpp 마스크를 만들고 `image.mask` 에 둔다.
- `wie-wipi-c/src/api/graphics.rs`
  - `image_alpha` 를 더했다. 마스크가 `bpp 8` 이고 크기가 이미지와 같을 때만 알파를 돌려준다.
  - `draw_image` 는 마스크가 있으면 RGB565 색에 그 알파를 얹어 그린다. 그래서 투명 이미지를 `MC_grpDrawImage` 로 그린 결과는 종전과 같다. 16bpp 화면에서는 888 → 565 차이도 없다.
  - `blit_with_pixel_op` 는 인자 `source_is_image` 를 받는다. 이미지일 때 마스크 알파 0 인 화소를 건너뛴다(종전 32bpp `a == 0` 건너뛰기와 같은 규칙).
- KTF 는 마스크를 만들지 않는다(`create_wipi_image` 가 빈 마스크). 그래서 공용 경로 변경은 KTF 에 닿지 않는다.
- **알고 남긴 것**
  - 마스크 형식(8bpp)은 우리 표현이다. 실기 `mask` 필드의 형식은 모른다. 이번 표본에서 마스크를 읽는 게스트는 관측하지 못했다(전부 `img` 만 `MC_grpGetImageFrameBuffer` 로 받는다).
  - `MC_grpCopyFrameBuffer` 에 이미지 핸들을 넘기면 마스크를 보지 않는다. 표본 20종 head 로그에서 마스크 있는 이미지를 그 경로로 복사한 횟수는 **0** 이다.

### 3. 짝 측정 — base `382e64fc`(= `origin/main`) ↔ head, release `wie_validate`

- 조건: `--inject`(기본 27키) · `--max-ticks 4000000000` · `--shot-every 3`. 타이틀마다 base·head 를 같은 순간에 나란히 돌렸다(동시 2개).
- 대상: #483 표본 LGT 78종 중 `MC_grpCreateImage` 를 부르는 **20종 전부**, 그리고 부르지 않는 6종(대조군)이다.
- 시작 때 `host-load-guard --status --recovered` 는 rc=1 이었다. 그래서 폭을 2로 줄였다. 스윕 중 load1 은 23~38 이었다.
- 줄무늬 점수는 #483 과 같은 정의다(시계열 화면 중 최댓값).

| 타이틀 | 판정 base→head | paints | 줄무늬 % | 이미지 / 마스크 / 직접읽기(마스크) |
|---|---|---|---|---|
| `0baceb1dfe9c` | PASS→PASS | 217→204 | **12.7→1.5** | 171 / 159 / 22(19) |
| `a23f3c9fc2cb` | PASS→PASS | 155→143 | 2.1→2.1 | 71 / 42 / 71(42) — 로고 3장·타이틀 화면이 바로 나온다(그림 비교) |
| `0606d43702e7` | PASS→PASS | 327→325 | **11.1→6.4** | 8 / 2 / 0 — 타이틀 탱크 위 얼룩이 사라졌다 |
| `63332c51d514` | PASS→UNMEASURED(이 base) · 새 base 에서 PASS→PASS 3/3 | 199→199 | — | 3 / 3 / 1(1) — §4 |
| 직접 읽는 나머지 7종(`01eef742dd0a` · `1eaa92092bee` · `34c48bbae783` · `6b515884dbc1` · `863b8ab6a21d` · `b9bfcaf42722` · `e68b1c8aef85`) | 전부 PASS→PASS | ±6% 안 | 전부 같음 | 마스크 0~4 · 직접 읽는 이미지의 마스크 0 |
| CreateImage 를 부르는 나머지 9종 | PASS→PASS 7 · UNMEASURED→UNMEASURED 2 | PASS 7종 ±1% 안 | 같거나 낮음 | — |
| 대조군 6종(CreateImage 0) | 전부 PASS→PASS | — | 같음(1종 0.4→1.0) | 0 |

- 대조군은 이 수정이 구조상 닿지 않는다. 그런데도 마지막 화면 차이가 0~85%, paints 가 471→407 까지 움직였다(`d01275d61fa9`). 마지막 화면 차이와 paints 차는 이 부하에서는 잡음이다. 그래서 판단은 그림 비교와 판정으로 했다.
- 그림은 사람 눈으로 봤다. `0baceb1dfe9c` 는 1초 간격 짝 촬영도 봤다. 메뉴 배경 나무의 금색 세로줄이 사라지고 메뉴 커서(캐릭터)가 보인다. 타이틀 문구가 늦게 뜨는 것은 base·head 같은 애니메이션이다.

### 4. `63332c51d514` — 첫 스윕의 PASS→UNMEASURED 는 0449 §3 의 스택 찌꺼기였다 · #490 착지 뒤 재측

- 첫 스윕(base `382e64fc`)의 짝 재측(3회씩)은 base PASS 3/3 · head UNMEASURED 3/3(clean exit · 1키)이었다. head 첫 화면에 게임 자신의 «데이터 저장 실패로 게임을 종료합니다»가 나왔다.
- 갈림은 **이미지를 만들기 전**이었다. `MC_dbListRecordInfo(SAVsetup)` 뒤에 base 는 `MC_dbCloseDataBase`, head 는 `db.stream_read(…, 22067244)` 였다. 빈 DB 의 크기 칸을 쓰지 않아 스택 찌꺼기가 크기가 되는 결함이다(`docs/report/0449` §3). base 의 create_image 로직을 그대로 둔 탐침 빌드도 똑같이 실패했다(2/2 · 같은 `22067244`). 찌꺼기 값은 빌드마다 갈린다.
- 그 사이 #490(`55447f05` · 빈 DB 정보는 크기 0 항목)이 착지했다. 이 브랜치를 `origin/main` `9c8d9729` 로 올리고 그 base 와 다시 짰다(3회씩 · 같은 조건).
  - base PASS 3/3 · head PASS 3/3 · paints 199/199 · 27키 전부.
  - 34장 중 32장이 바이트 단위로 같다. 다른 두 장(`09_UP` · `10_OK`)은 캐릭터 선택 화면이다. base 는 초상 다섯 장 중 대부분이 검게 빠지고, head 는 다 나온다(3회 모두 같다). 키 값이 base `0xff` → head `0xf81f` 로 바뀐 결과다(§1).
- 그 사이 착지한 세 PR 의 Rust 변경은 `wie-skvm`(#492) · `wie-wipi-c/src/api/database.rs` 와 LGT `OpenDatabase` 배선 1줄(#490)뿐이다. 이 회차가 고친 그리기 경로에는 닿지 않는다. §3 의 나머지 25종은 base·head 가 같은 `382e64fc` 위에서 짝을 이뤘으므로 그 비교를 그대로 쓴다.

### 5. 검증

- 개악 red 3종. 신규 시험 `wipic_create_image_stores_images_as_rgb565_with_an_alpha_mask`.
  ⑴ 투명 이미지는 32bpp 유지로 되돌리면 → `left: (32, 8)` · `right: (16, 4)` FAILED
  ⑵ `draw_image` 의 마스크 적용을 끄면 → 마젠타가 그려져 `0x11aaf81f ≠ 0x11aaaaaa` FAILED
  ⑶ 픽셀-op 블릿의 마스크 건너뛰기를 끄면 → 숨은 화소가 proc 를 거쳐 `0xf81f11aa ≠ 0xf81faaaa` FAILED
  셋 다 복원하면 ok.
- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm32 clippy · `cargo +beta clippy --all -- -D warnings` 모두 rc=0. beta 의 `unused dependency` 경고 9줄은 이 diff 밖이고 rc 에 영향이 없다.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0(714 passed · 0 failed · `CARGO_INCREMENTAL=0`).
- 러너 블록 6줄 PASS. `keydraw_ktf`/`keydraw_lgt` 는 `--inject --expect-last-frame` 에서 rc=0 · paints 79/55.
- 짝 스윕과 6333 재측은 각각 `build-slot run --long` 임대 1개로 감쌌다. 둘 다 long 슬롯이 만석이라 대기한 뒤 돌았다(다른 레인이 쥐고 있었다). 회차 밖 프로세스는 0이다.
- `compat.json` 은 바꾸지 않았다. `render` 축은 줄무늬를 보지 못한다(0450 §4 와 같은 판단).
- 게임 파일명 유입: `corpus-name-inflow` BOUNDED 12회/7쌍 · SUFFIX-ATTACHED 0. 12회 모두 이 회차가 손댄 두 파일(`wie-lgt/src/runtime/wipi_c.rs` · `wie-wipi-c/src/api/graphics.rs`)에 이미 있던 주석이다(`origin/main` 판에서 같은 6개 이름이 12회). 이 회차가 더한 줄에는 0건이다.

<!-- corpus-name-inflow v1 subjects=6 tree=a972949868a563bf B=12/7 P=0/0 S=0/0 -->
