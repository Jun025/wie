## [2026-09-19] canvas 의 decode·roundtrip 단언 8건 — «먼저 쟀고», 제안이 예고한 red 는 오지 않았다 (2026-09-18-sibling-crates-missing-tests-machine-audit-p0)

**무엇을**: 채택 제안 `2026-09-18-sibling-crates-missing-tests-machine-audit#p0` 의 이행.
`0172` 의 E 축이 낸 「대상은 살아 있고 단언만 사라진」 10건 중 **최대 군집인 canvas 8건**을 되살렸다.
파일 **1개**(`wie-backend/src/canvas.rs`) · **+143 / -2 행** · ★**제품 코드 0행**(전부 `#[cfg(test)] mod tests` 안 + `use` 2행).
`#[test]` **28 → 36** · `wie-backend --lib` **55 → 63 passed**.

**왜**: `decode_image` 는 `canvas.rs:902` 에 살아 있고 `canvas/lbmp.rs`·`res.rs` 도 있는데, HEAD 의 28시험은 **전부** draw/clip/arc/xor 계열이라 **디코딩·색변환 경로의 단언이 0** 이었다. 게임 화면이 그려지는 경로다.

**사용자 영향**: 지금 당장 고쳐지는 버그는 **0**(제품 코드 무접촉). 바뀐 것은 ★**그 경로가 조용히 깨지면 «게임에서만» 드러나던 것이 이제 `cargo test` 에서 드러난다**는 것이다 — 아래 «7/8» 이 그 크기다.

---

### ★★결론 먼저 — 제안이 스스로 적은 tradeoff 가 «틀렸다»

제안 원문의 tradeoff:

> ★되살리면 red 일 수 있다 — 그 8건은 2026-09-16 이전 API 위에 쓰였고, upstream 이 canvas 를 갈아끼웠다. ⇒ `0153` 이 고아 3건에 한 그대로 «되살려 컴파일해 보는 것»이 첫 걸음이고, red 면 그 자체가 판정이다(되살리기가 아니라 재작성이 필요하다는 뜻).

★**재서 답한다: red 는 «0» 이다.** `ac6e0705`(swap 직전) 의 본문을 **한 글자도 고치지 않고** 붙여 **8/8 green**.
★**본문 수정 0행** — 바꾼 것은 그 `mod tests` 의 `use` **2행**뿐이다(`decode_image`·`AbgrPixel`·`PixelType`·`Rgb8Pixel`·`Rgb565Pixel` 를 스코프에 들임).

★**왜 안 깨졌나 — 추측하지 않고 «대조해서» 답한다.** `ac6e0705` ↔ HEAD 를 면 단위로 diff 했다:

| 이 8건이 무는 면 | `ac6e0705` ↔ HEAD |
|---|---|
| `decode_image` 본문(24행) | ★**바이트 동일** |
| `wie-backend/src/canvas/lbmp.rs` 전체 | ★**바이트 동일**(swap 전에도 이미 별 파일이었다 — «이사»가 아니다) |
| 다섯 `PixelType` 의 `from_color`/`to_color` | ★**전건 바이트 동일** — diff 29행은 **전부 «`xor_color` 가 새로 추가됐다»** 뿐이다 |

⇒ upstream 이 갈아끼운 것은 **그리기 계층**(`Canvas` 에 `draw_arc`·`fill_round_rect`·`invert_rect`·`set_xor_mode`, `PixelType` 에 `xor_color`)이고,
★**「upstream 이 canvas 를 갈아끼웠다」는 참이지만 「이 시험들이 무는 면을 갈아끼웠다」는 거짓이었다.** 제안의 tradeoff 는 전자에서 후자를 **추론**했고, 그 추론이 틀렸다.

---

### ⒞ 양방향 개악 — ★제품 호출부에 넣었다(픽스처 사본 아님)

8건 각각에 대해 **제품 코드 1곳**을 개악하고 되돌렸다. 개악 지점은 전부 `wie-backend/src/canvas.rs` 의 `decode_image`·`PixelType` 구현부와 `wie-backend/src/canvas/lbmp.rs` 의 `decode_lbmp` — ★**시험 파일이 아니다.**

| # | 개악한 제품 코드 | 되살린 시험 | 개악 시 | 원형상 |
|---|---|---|---|---|
| M1 | `decode_image` 채널 순서 `[2,1,0,3]` → `[0,1,2,3]` | `test_decode_image_png` | **FAILED** | ok |
| M2 | `decode_image` 의 LBMP 매직 가드 `data.len() >= 4 &&` **삭제** | `test_decode_image_short_input_returns_err` | **FAILED** | ok |
| M3 | `decode_lbmp` 의 `data.len() < 24` → `< 4` | `test_decode_lbmp_truncated_header_returns_err` | **FAILED** | ok |
| M4 | `decode_lbmp` 의 `header.r#type == 8` → `== 9` | `test_decode_lbmp_rgb332` | **FAILED** | ok |
| M5 | `Rgb332Pixel::to_color` 의 `b: b * 85` → `b * 84` | `test_rgb332_roundtrip` | **FAILED** | ok |
| M6 | `Rgb565Pixel::to_color` 의 `(g * 255 + 31) / 63` → `+ 0` | `test_rgb565_roundtrip` | **FAILED** | ok |
| M7 | `Rgb8Pixel::to_color` 의 `(raw >> 16)` → `(raw >> 15)` | `test_rgb8_roundtrip` | **FAILED** | ok |
| M8 | `AbgrPixel::to_color` 의 `b = (raw >> 16)` → `(raw >> 8)` | `test_argb_abgr_roundtrip` | **FAILED** | ok |

**8/8 개악 red · 8/8 원형상 green.**

---

### ★★그런데 이 표만으로는 «값»을 증명하지 못한다 — 중복 단언 여부를 따로 쟀다

제안이 요구한 두 번째 선행 조건:

> ★그리고 upstream 이 자기 시험 28건을 넣었으므로 «중복 단언»을 만들지 않게 대조가 선행돼야 한다.

★**같은 8개 개악을 «내 시험이 없는 원형 트리»(`origin/main` · 55 passed)에 그대로 넣고 전 스위트를 돌렸다.**

| 개악 | 원형 트리 55시험의 반응 |
|---|---|
| M1 M2 M3 M4 M6 M7 M8 | ★**ok. 55 passed — 아무도 못 잡는다** |
| M5 (`Rgb332` `b*85`→`b*84`) | FAILED 1 — `canvas::res::tests::decodes_static_icon_after_animation` |

⇒ ★★**8건 중 7건은 «이 저장소의 어떤 시험도 잡지 못하던» 개악이다.** 중복 단언이 아니라 **실공백**이었다.
★**1건(M5)은 부분 중복**이다 — 다만 그것도 **roundtrip 단언이 아니라** RES 아이콘 디코딩 시험이 `Rgb332::to_color` 를 지나가다 **우연히** 무는 것이고, 같은 계열의 M6·M7·M8 은 **못 잡는다**. ⇒ 우연한 경유는 축이 되지 못한다.

---

### ⒟ ★대가 — 「잃는 것이 없다」로 적지 않는다

- ★**`canvas.rs` 가 1,652 → 1,793행이 된다.** 이 파일은 이미 **시험이 본문보다 긴** 상태였고(`mod tests` 가 949행부터), 이번 회차가 그 비대칭을 **더 키운다.** 파일 분할(`canvas/tests/`)은 하지 **않았다** — 이 회차의 범위를 「8건 복원」으로 좁혔고, 분할은 upstream 병합면을 넓혀 base swap 비용을 키운다.
- ★**양자화 상수를 «고정»한다.** `test_rgb332_roundtrip` 은 `36`·`85`, `test_rgb565_roundtrip` 은 `+15)/31`·`+31)/63` 을 기대값으로 박는다. ⇒ ★**색 정확도를 «개선»하려는 정당한 변경이 앞으로 red 를 낸다** — 3개 시험의 기대값 편집이 그 변경의 일부가 된다. 그것이 이 단언의 목적이지만, 비용은 비용이다.
- ★**78바이트 PNG 상수(`PNG_2X2`)가 `image` 크레이트 디코더의 동작에 묶인다.** 그 크레이트가 RGBA 변환을 바꾸면 `test_decode_image_png` 가 **엔진 결함이 아닌 이유로** red 가 된다. (오늘 기준 pin 고정이라 즉시 위험은 아니다.)
- ★★**upstream 동기 repo 에 «우리 overlay» 143행을 더한다.** 이 8건은 upstream 에 없다 ⇒ **다음 base swap·realign 이 이것을 이월해야 한다.** ★그리고 `0172` 가 기록한 바로 그 형태 — **개명·swap 이 조용히 떨어뜨린다** — 의 새 표적이 하나 늘어난 것이다. (그 손실을 기계로 잡을지는 `#p2` 가 지는 별건이고, 이 회차가 그 제안의 **값을 올렸다**는 점은 적어 둔다.)
- ★**M5 는 부분 중복이다**(위 표) — 8건 전부가 «새 커버리지»는 아니다.

---

### ⒠ 게이트 — 내 출력

```
cargo fmt --all -- --check                                    → OK (무출력)
cargo clippy --all -- -D warnings                             → Finished `dev` profile … in 1m 39s (경고 0)
cargo clippy --target wasm32-unknown-unknown -- -D warnings   → Finished `dev` profile … in 7.72s (경고 0)
RUST_MIN_STACK=4194304 cargo test --all                       → exit 0 · FAILED 0 · wie-backend 63 passed(55→63)
cargo +beta clippy --all -- -D warnings                       → Finished `dev` profile (경고 0)
```
★**전부 캐시가 아니다** — stable clippy 는 `wie`·`wie-web` 를 실제로 checking 했고(1m 39s), `cargo test --all` 의 최장 바이너리는 **135.79s** 로 돌았다. wasm 레그(7.72s)만 대체로 캐시다.

★**엔진 크레이트를 건드렸으므로 `AGENTS.md` 의 러너 블록도 돌렸다** — 출력은 회신에.

---

### ★범위를 스스로 좁힌 곳

- 제안 표기 노력도는 `M`. ★**실측은 그보다 작았다** — 되살리기가 **본문 수정 0행**이었기 때문이다. 대신 그 절감분을 **중복 단언 대조**(원형 트리 8회 전 스위트 실행)에 썼다. 그쪽이 이 제안이 실제로 물은 질문(「값이 있는가」)이다.
- ★**`0172` 의 나머지 2건(LGT `svc_ids` 0x581 · `test_helloworld` 세 번째 케이스)은 이 회차에 넣지 않았다** — 별 제안(`#p1`)이고, 그쪽은 제안 자신이 「번호 체계가 바뀌었을 수 있다」·「픽스처가 필요한지부터」라고 적은 **다른 성격의 작업**이다. 섞으면 이 회차의 판정(「red 0」)이 흐려진다.
- ★**`canvas/lbmp.rs` 에 시험을 «새로» 쓰지 않았다.** 되살린 2건(M3·M4)은 `decode_image` 라는 **공개 진입점**을 지나므로 디스패치까지 함께 문다 — 원래 자리가 옳았다.

### 증적

`~/orchestrator/reports/evidence/2026-09-18-sibling-crates-missing-tests-machine-audit-p0/`
(개악 매트릭스 스크립트 + 두 실행 출력 — 제품 트리에 두지 않았다).
