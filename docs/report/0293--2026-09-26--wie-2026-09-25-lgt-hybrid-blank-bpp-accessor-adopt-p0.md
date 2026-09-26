## [2026-09-26] LGT 컨텍스트 레코드(둘째 층) — 글자색은 네이티브 +16 에서 읽힌다 · 공용 레코드 위에 네이티브 세 칸을 겹쳐 연다 (wie-2026-09-25-lgt-hybrid-blank-bpp-accessor-adopt-p0)

**무엇을** — `wie-lgt` 의 WIPIC `InitContext`/`SetContext` 두 줄을 공용 구현에서 LGT 쪽 얇은 겹침(`graphics::init_shared_context`/`set_shared_context`)으로 옮겼다.
레코드는 **여전히 공용 52B** 다. 겹침은 네이티브 칸 셋(전경 +16 · 배경 +20 · 알파 +24)만 채운다. SVC 스텁을 거치는 시험 1개를 더했다.

**왜** — 채택 제안 `2026-09-25-lgt-hybrid-blank-bpp-accessor#p0`: #301(BPP) 뒤에도 `0236` §2-2 의 타이틀은 글자가 `#fffbff` 라 흰 바탕에서 보이지 않았다.

**사용자 영향** — 그 타이틀의 이용안내 글자가 빨간색으로 보인다. `working/lgt` 54 에서 회귀 0 이고, 두 타이틀이 덤으로 더 그린다(§4).

## 0. 전제 재측 — 아직 열려 있다

`origin/main` `978c7b3d` · release · `--timeout 60` · 마지막 프레임:
PASS · distinct **2** · 흰 바탕 `#ffffff` 74,675 + 글자 `#fffbff` **2,125** ⇒ 제안 문면 그대로다. 닫히지 않았다.

## 1. 글자색을 읽는 자리 — 센티넬로 특정했다

`#309`(`0274`)는 두 배선에서 글자 **자리와 수(2,125)가 같고 색만 다르다**고 쟀다. 그래서 공용 `init_context` 만 바꾼 바이너리로 칸을 하나씩 표시했다:
`alpha = 255` · `bgpxl`(+16) = `0x001f`(파랑) · `transpxl`(+20) = `0x07e0`(초록). 나머지는 기본값이다. `SetContext` 는 공용 그대로다.

| 바이너리 | 글자 픽셀 |
|---|---|
| `origin/main` | `#fffbff` 2,125 |
| 센티넬 | ★**`#0000ff` 2,125** |

⇒ **게스트는 글자색을 +16 에서 읽는다.** 공용 레이아웃에서 +16 은 `bgpxl` 이고, 네이티브 `LgtGraphicsContext` 에서는 `foreground` 다.
⇒ `#fffbff` 는 알파 0 의 흔적이다. 공용 기본 `alpha`(+24) 가 0 이라 글자가 흰 바탕에 거의 다 섞였다. 센티넬에서 알파를 255 로 두자 칸의 값이 그대로 나왔다.
+24 는 두 레이아웃에서 모두 `alpha` 다.

## 2. 왜 레코드를 통째로 바꾸지 않았나 — `keydraw_lgt` 의 SDK

`dlunch/wipi`(`4e3d8d3` · `wipi/src/framebuffer.rs`)의 `Framebuffer` 는 `WIPICGraphicsContext`(52B)를 **필드로 품는다**. `init_context` 로 채운 뒤 `fgpxl`(+12)을 **직접** 쓰고, `SetContext` 는 부르지 않는다.
네이티브 레코드는 56B 라 그 필드를 넘어 쓴다. `0262` §3 의 `keydraw_lgt` 파손(`Undefined instruction` in `CletWrapperCard.paint`)과 맞는 형태다.

그런데 공용 호스트 판독자(`fill_rect`·`draw_*`·`put_pixel` …)는 `bgpxl`·`transpxl`·`alpha` 로 **그리지 않는다**. 세 필드는 `SetContext`/`GetContext` 에만 나온다(`wie-wipi-c/src/api/graphics.rs` grep).
⇒ **한 레코드에 두 ABI 가 들어간다.** 호스트가 읽는 자리(+4 clip · +12 fgpxl · +28 offset)는 공용 필드이고, 타이틀이 읽는 자리(+16 · +20 · +24)는 네이티브 필드다.
게스트마다 레코드를 고르는 분기(판별자)가 **필요 없다.**

- `InitContext`: 공용 초기화 + `alpha` = 255(네이티브 초기값).
- `SetContext` fg: 공용 +12 **와** 네이티브 +16.
- `SetContext` bg: 네이티브 +20 만. 공용은 +16 에 써서 글자색을 덮는다. 시험의 변이가 정확히 그 값을 잡는다(§5).
- `SetContext` trans: 무시(네이티브도 무시한다). 공용은 +20 에 써서 네이티브 배경을 덮는다.
- 나머지 op: 공용 그대로.

## 3. before / after — 대상 타이틀과 fixture

release · `--timeout 60`:

| | 결과 | distinct | 마지막 프레임 |
|---|---|---|---|
| `origin/main` | PASS | 2 | `#ffffff` 74,675 · `#fffbff` 2,125 |
| 이 브랜치 | PASS | 2 | `#ffffff` 74,675 · ★**`#ff0000` 2,125** |

`#309` 의 ⒜(레코드 전면 LGT)도 `#ff0000` 2,125 였다 ⇒ 같은 화면이다.

| fixture | `origin/main` | 이 브랜치 |
|---|---|---|
| `keydraw_lgt --inject --expect-last-frame --max-ticks 5000000000`(release) | PASS · 27/27 · paints 55 · rc=0 | PASS · 27/27 · paints 55 · rc=0 |
| `helloworld_lgt`(release) | PASS · rc=0 | PASS · rc=0 |

## 4. `working/lgt` 54 짝지은 게이트

`--timeout 15` · 50s 워치독 · 순서는 타이틀마다 교대 · 3병렬 · load1 **28~99**.

| | PASS | FAIL |
|---|---|---|
| `origin/main` | 54 | 0 |
| 이 브랜치 | **54** | 0 |

마지막 프레임 색 수가 어긋난 7건은 `--timeout 30` 으로 짝지어 스크린샷을 떴다:

| 타이틀 | 판정 |
|---|---|
| `(LGT)알바타이쿤2` · `놈ZERO` | 팔레트·픽셀 수 **동일** ⇒ 15초 프레임 차이 |
| `제노니아1` · `하이브리드` | 같은 두 색 · 빨강 픽셀 수만 다르다(깜박이는 안내 줄) ⇒ 프레임 차이 |
| `블레이드마스터4` | ★main 은 검은 바탕에 글자만 있다. 이 브랜치는 **대화상자 틀(주황 테두리·남색 바탕)까지** 그린다 ⇒ 개선 |
| `판타지포에버3` | 이 브랜치만 아래쪽 빨간 안내 줄이 있다(깜박임일 수도 있다 · 회귀 아님) |
| `게임빌2010슈퍼사커` · `메탈슬러그 서바이벌` | 색 수 5=5 · 189↔194 ⇒ 프레임 차이 |

⇒ **회귀 0.**

## 5. 시험 — 양방향

`wipic_context_keeps_native_foreground_and_alpha`: SVC 스텁으로 `InitContext` → `SetContext(fg 0xf800)` → `SetContext(bg 0x1234)` 를 부르고 +12 · +16 · +20 · +24 를 단언한다.
- 이 브랜치: ok.
- 변이(두 줄을 공용으로 되돌림): **FAILED** `native foreground, read by the title` · `left: 4660 (0x1234) · right: 63488 (0xf800)`. 공용 bg 가 네이티브 전경 칸을 덮는 바로 그 충돌이다. 복원하면 ok.

## 한계

- 겹친 것은 세 칸뿐이다. 네이티브 `pixel_param`·`font`·`xor_enabled`·`offset`(+32~+52)은 여전히 공용 필드와 겹친다. 타이틀이 `SetContext(font)` 를 부르면 공용이 +44 에 쓰고, 그 칸은 네이티브 `xor_enabled` 다.
  `GetContext(bg)` 는 +16(전경)을 돌려준다. 어느 것도 관측된 타이틀은 없다. 코드에 `ponytail:` 주석으로 적었다.
- 글자색 칸은 센티넬 실험으로 특정했다. 게스트 글리프 루틴의 디스어셈블리는 뜨지 않았다. 다만 칸을 바꾸면 2,125 픽셀 **전부**가 그 값을 따른다.
- 디버그 러너 줄의 `keydraw_ktf`/`keydraw_lgt` 는 기본 `--max-ticks` 에서 둘 다 `UNMEASURED` rc=2(`stop: max-ticks`)였다. KTF 는 이 diff 가 닿지 않는다. 백스톱을 올리면 둘 다 PASS 27/27 · rc=0 이다.

## 검증

- `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · wasm clippy rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 47 스위트 · **458 passed · 0 failed**.
- 러너: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me` PASS · keydraw 두 개는 위 한계 참조.
- `docs/upstream-realign-p3-slices.md` §D: 공용 배선 27 → **25**줄.

## 게임 파일명 유입 — 도구를 «실행해서» 적는다

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 의 대상은 이 브랜치가 바꾼 파일이다. 수는 아래 표식이 최종이다.
BOUNDED 는 §4 게이트에서 스크린샷을 뜬 타이틀 이름과 `wipi_c.rs`·`graphics.rs` 에 원래 있던 주석 속 이름이다. 대상 타이틀은 이름 대신 «`0236` §2-2 의 타이틀»로 적었다.
SUFFIX-ATTACHED 1쌍은 `docs/upstream-realign-p3-slices.md` 에 원래 있던 조사 붙은 진짜 언급이다 ⇒ 판단이 더 필요한 것 0.
게임 바이트 유입 0 · 경로 유입 0.




<!-- corpus-name-inflow v1 subjects=6 tree=425161612bab68a7 B=21/16 P=0/0 S=1/1 -->
