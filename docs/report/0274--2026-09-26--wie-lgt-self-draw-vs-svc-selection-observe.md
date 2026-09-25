## [2026-09-26] LGT 타이틀의 «자기가 그릴지 · SVC 에 맡길지» 분기는 없다 — `0262` 의 «⒜ FillRect 0» 은 로그 공백이었다 (wie-lgt-self-draw-vs-svc-selection-observe)

**무엇을** — 측정과 주석 추가만 했다(**동작 코드 0줄**). `0262` §2 가 남긴 질문 「게스트가 무엇을 보고 자기가 그릴지 SVC 에 맡길지 고르나」를
SVC 디스패치 층에서 직접 관측했다. `wie-lgt/src/runtime/wipi_c.rs` 의 «⒜ 가 `keydraw_lgt` 를 깨는 이유» 주석에 컨텍스트 레코드 축을 더했다.

**왜** — 채택 제안 `2026-09-25-lgt-graphics-context-record-refuted#p0`·`#p1`(운영자 cockpit 채택 2026-09-25T12:05Z).

**사용자 영향** — 없다. 그 타이틀(`0236` §2-2 의 뒤집힌 1건)은 지금도 흰 바탕에 거의 흰 글자다.
이 회차는 다음 수리가 어디를 봐야 하는지를 좁혔다.

## 0. 대전제

- 같은 처방: `ledger-grep` tasks·queue·running 에서 이 티켓만 걸린다. `gh pr list` 에 같은 관측을 하는 PR 은 없다.
- ★인접 티켓 **`wie-2026-09-25-lgt-hybrid-blank-bpp-accessor-adopt-p0`**(queue/wie · 미착수)은 **같은 타이틀의 글자색 층**을 맡는다.
  이 회차는 그쪽 Contract 2(글자색을 읽는 필드 특정)를 하지 않았다. 아래 §3 이 그 티켓의 전제를 고정한다.
- 기준은 `origin/main` `6dfca9e6` 이다. #301(BPP 를 LGT 접근자로 · 머지 `4d5d639b`)이 들어 있다.
  ★#301 §4 가 이미 이 타이틀의 첫 층을 BPP 라고 밝혔다. 그래서 **BPP 가 분기를 고르는가**를 먼저 쟀다.

## 1. 관측 — 두 바이너리 · 디스패치 층 계수

release `wie_validate` 두 개 · `--timeout 60` · `RUST_LOG=wie_wipi_c=debug,wie_lgt=debug` · load1 77~126.

| 바이너리 | 만든 법 |
|---|---|
| `main` | `origin/main` 무수정 |
| `⒜+probe` | 공용 graphics 26팔을 LGT `graphics::` 로 교체(`GetContext` 는 LGT 판이 없어 공용 유지) · BPP 는 이미 LGT · `handle_wipic_svc` 첫 줄에 `tracing::debug!("PROBE_SVC {:#x}", id.0)` → 빌드 → `git checkout --`(`status --porcelain` 빈 출력) |

대조쌍 `keydraw_lgt --inject --expect-last-frame --max-ticks 5000000000`: `⒜+probe` **FAIL** rc=1(`Undefined instruction` in `CletWrapperCard.paint` · paints 0) ↔
`main` **PASS** rc=0 · paints 55. `0222` §2-3 서명과 같다. 따라서 두 바이너리는 서로 다르다.

★**왜 디스패치 층에 탐침을 달았나** — LGT `graphics.rs` 의 그리기·이미지 함수 15개(`put_pixel`·`fill_rect`·`draw_line`·`draw_rect`·`draw_arc`·`fill_arc`·
`draw_image`·`copy_frame_buffer`·`copy_area`·`draw_string`·`get_rgb_pixels`·`set_rgb_pixels`·`create_image`·`get_image_framebuffer`·`get_image_property`)에는 `tracing::debug!` 가 **없다**(`awk` 로 함수별 확인). ⇒ `0262` §2 의 ⒜ 열은 그리기 SVC 를 **셀 수 없는 계수기**로 센 값이었다.

프레임당 호출(데이터베이스·메모리 제외):

| SVC | `main` (59 프레임) | `⒜+probe` (862 프레임 · id) | 프레임당 |
|---|---|---|---|
| `GetFramebufferPointer` | 534 | 7,765 (`0x32`) | 9 · 9 |
| `GetPixelFromRgb` | 241 | 3,453 (`0xdf`) | 4 · 4 |
| `SetContext` | 240 | 3,452 (`0xce`) | 4 · 4 |
| `InitContext` | 123 | 1,729 (`0xcd`) | 2 · 2 |
| ★`FillRect` | **60** | **863** (`0xd3`) | **1 · 1** |
| `FlushLcd` | 59 | 862 (`0xde`) | 1 · 1 |
| `GetFramebufferBpp` | 1 (→ 16) | 2 (`0x36`) | 부팅 때만 |

★**판정: 분기는 없다.** 두 배선에서 게스트는 **같은 SVC 를 같은 순서로** 부른다.
바탕은 두 배선 모두 `FillRect` SVC 로 칠하고, 글자는 두 배선 모두 `GetFramebufferPointer` 로 받은 버퍼에 게스트가 직접 쓴다.
⇒ `0262` §2 의 「⒜ 는 흰 바탕을 SVC 없이 자기 경로로 채운다 · 게스트가 스스로 판단한다」는 **로그 공백을 읽은 것**이다. BPP 도 이 선택에 관여하지 않는다
(BPP 가 공용이던 `0262` ⒝ 도, LGT 인 `main` 도 `FillRect` 가 프레임당 1회다).

## 2. 출력 — 같은 글자 픽셀, 다른 색

`--screenshot` 마지막 프레임(240×320)의 색 계수(`uv run --with pillow`):

| 바이너리 | 바탕 | 글자 |
|---|---|---|
| `main` | `#ffffff` 74,675 | `#fffbff` **2,125** |
| `⒜+probe` | `#ffffff` 74,675 | `#ff0000` **2,125** |

★글자 픽셀 **수와 자리**가 같고 **색만** 다르다(#301 §4 의 `#fffbff` 2,125 px 를 재현했다).
⇒ 게스트가 직접 쓰는 글자는 `main` 에서도 **제자리에 떨어진다**. 픽셀 포인터를 주는 프레임버퍼 레코드는 이 타이틀에서 **문제가 아니다**.
틀린 것은 **글자 색 값 하나**다. `SetContext(FgPixelIdx, 0xf800)` 은 두 배선에서 똑같이 불린다.
그러므로 색이 갈리는 곳은 그 값이 **기록되는 컨텍스트 레코드의 자리**다. 공용은 `fgpxl` 을 +12 에 쓰고, LGT 는 `foreground` 를 +16 에 쓴다.
게스트가 정확히 어느 오프셋을 읽는지는 **재지 않았다** — 그것은 인접 티켓의 Contract 2 다.

## 3. 뜻 — `0262` §4 의 추론을 고친다

- `0262` §4 는 「남은 차이는 프레임버퍼·화면 레코드 계열」이라고 추론했다. ★이 타이틀에서는 **반증됐다**: 글자 픽셀 2,125 개가 같은 자리에 쓰인다.
- 남은 층은 **컨텍스트 레코드 하나**다. `0262` §3 어댑터 F 가 이 층을 «원인 아님»으로 본 것은 **BPP 가 공용이던 트리**에서였다(#301 §4 가 이미 지적했다).
  그 반증은 BPP 층에 가려져 있었다.
- ★설계 결정인가: 이 층도 `keydraw_lgt` SDK 가 공용 48B 레이아웃으로 **직접 읽는다**(`0262` §3 F · 이 회차 주석).
  «공용 레코드 하나로 둘 다 연다»가 가능한지는 **게스트가 읽는 오프셋이 겹치는지**에 달렸다. 오프셋이 특정되기 전에는 판정할 수 없다.
  그 특정과 판정은 인접 티켓 `…-bpp-accessor-adopt-p0` 의 Contract 2·3 이 이미 맡았다.
  ⇒ **새 설계 메모도, 새 제안도 내지 않는다**(번식 0).

## 4. 주석(제안 p1)

`wie-lgt/src/runtime/wipi_c.rs` 머리 주석에 6줄을 더했다. 내용: 프레임버퍼 레코드만이 축이 아니다.
LGT 컨텍스트(`LgtGraphicsContext` 56B)만 어댑터로 주고 프레임버퍼는 공용으로 둬도 `keydraw_lgt` 가 같은 `Undefined instruction` 으로 깨진다(`0262` §3).
그러므로 재배선은 레코드 ABI 둘을 바꾼다. 코드 변화는 0 이다.

## 한계

- 표본은 이 타이틀 1건이고, 바이너리마다 1회씩 쟀다. 판정축(호출 순서·프레임당 수·글자 픽셀 수)은 부하와 무관하게 정수로 같다. paints 절대값(59 ↔ 862)은 부하 값이니 인용하지 마라.
- `⒜+probe` 는 `0262` ⒜ 와 **같지 않다**. `0262` ⒜ 는 BPP 가 공용이고 27팔이었다. 이 회차는 BPP 가 LGT 이고 26팔이며 `GetContext` 는 공용이다.
  `0262` ⒜ 의 FillRect 수를 다시 재지는 않았다. 다만 그 회차의 계수기로는 어떤 값이든 0 으로 보였다(§1).
- 글자색 필드 오프셋은 미측정이다(인접 티켓 소관).

## 검증 — 실행한 것

- release 두 개 빌드 · 이 타이틀 `main`·`⒜+probe` 각 1회 + 스크린샷 색 계수 · `keydraw_lgt` 대조쌍 · 소스 복원 `git status --porcelain` 빈 출력.
- 네 게이트 + beta clippy — 결과는 PR 본문과 회신에 인용한다.

## 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 결과는 ★**BOUNDED 11회 / 6쌍** · PREFIX-EMBEDDED 0 · ★**SUFFIX-ATTACHED 0** 이다.
6쌍은 전부 `wie-lgt/src/runtime/wipi_c.rs` 에 **원래 있던** 주석 속 이름이다. 도구는 바뀐 파일 전체를 센다. 이 회차가 그 파일에 더한 6줄에는 이름이 없다.
이 문서와 worklog 도 타이틀 이름 대신 «그 타이틀»(`0236` §2-2)로 적었다. ⇒ 이 회차가 새로 들인 이름은 0 이다. 게임 바이트 유입 0 · 경로 유입 0.


<!-- corpus-name-inflow v1 subjects=3 tree=30dbbdf12e064f03 B=11/6 P=0/0 S=0/0 -->
