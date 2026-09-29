## [2026-09-29] 다시 그려 달라고 한 영역을 paint 의 클립으로 — 월드장기체스가 흰 화면 루프를 벗어난다 (wie-2026-09-29-wec-vector-vtable-19-adopt-p0)

**무엇을**
- `wie-midp` `Display.repaint(x,y,w,h)` 가 요청 영역을 버리던 것을, 다음 paint 까지의 **합집합**으로 모아 `handlePaintEvent` 의 클립으로 쓴다
  (필드 4개 · `paint_clip` · 요청 없는 paint 와 음수 크기(엔진의 «전체» 요청)는 종전대로 콘텐츠 전체).
- `wie-wipi-java` `CardCanvas.paint` 가 카드마다 `reset` 하며 그 클립을 전체 화면으로 되돌리던 것을, 들어온 클립(절대좌표)을 카드 좌표로 다시 건다(`clipRect`).
  카드 **자기 경계**로는 여전히 자르지 않는다 — upstream `7304facd`(「Avoid clipping WIPI cards during paint」)의 의도는 그대로 둔다.
- `wie_validate` 에 보고 전용 `frozen_tail_steps` — 마지막 몇 개 키가 화면을 **한 픽셀도** 바꾸지 못했는가.
- 테스트: `paint_clip_is_the_union_of_repaint_areas_since_the_last_paint`(신규) · `test_card_canvas_offsets_clips_and_isolates_graphics_state`(기대값 교정 — 아래) · `frozen_tail_counts_trailing_unchanged_steps_test`(신규).

**왜**: 0363 뒤 `--inject` 27/27 PASS 인데 키 이후 프레임이 전부 같았다(worklog `2026-09-29-wec-vector-vtable-19#p0`).

**사용자 영향**: 월드장기체스가 요금 안내문 뒤 흰 화면에서 멈추지 않고 제작사 로고(애니메이션)까지 넘어간다. **아직 플레이는 안 된다** — 곧바로 다음 벽(`java/util/Vector vtable index 17`)에서 멈춘다.
변경은 MIDP·WIPI Java 전반의 부분 repaint 에 걸린다(명세: paint 의 클립 = 다시 그릴 영역). 같은 방식으로 클립 폭을 확인하는 타이틀이 더 있다면 같은 경로로 풀리겠지만, 측정한 것은 이 타이틀 하나다.

### 원인 — 좁힌 과정(`RUST_LOG=wie_wipi_java=debug,rustjava_runtime=debug,wie_midp=debug` · release · 1회)

후보 넷(그리기 경로 · 이미지 로드 · 키 처리 · 입력 대기) 중 로그가 **그리기 경로**를 가리켰다. 멈춘 뒤 한 프레임의 호출 전부:

```
CardCanvas::paint → Graphics::reset → translate(0,0) → getClipWidth()        ← 아무것도 그리지 않는다
Display::callSerially(card) → Card::repaint(60, 138, 120, 46) → Thread::sleep(20)
(다음 paint 에서 반복 · 425회)
```

- 직전 프레임은 정상이다: 3,454바이트 이미지를 읽어(`Image::createImage`) 오프스크린에 두 조각(`setClip 60,0,120,160` · `60,160,120,24`)을 그리고 화면에 한 번 올린 뒤
  **부분 영역** `repaint(60,138,120,46)` 을 요청한다. 그 첫 조각이 캡처의 «흰 바탕 · 청록 표식 하나»였다(로고 왼쪽 끝).
- 그 뒤 paint 는 `getClipWidth()` 만 읽고 그리지 않은 채 같은 영역을 다시 요청한다 — 자기 부분 repaint 가 왔는지를 **클립 폭으로** 가린다.
- 엔진은 그 영역을 버렸다: `Display.repaint` 는 `repaintPending` 만 세우고, `handlePaintEvent` 는 늘 `setClip(0,0,240,320)`,
  `CardCanvas.paint` 는 카드마다 `reset`(클립 = 전체)한다 ⇒ 게임은 영원히 240 을 본다.
- 배제한 후보: **이미지 로드**(createImage 성공 · 예외 0) · **키 처리**(멈춘 뒤에도 키가 카드에 닿아 `Display.getGameAction` 이 21회 불리고 반환된다 — 루프는 키와 무관하게 돈다) ·
  **입력 대기**(`Thread.sleep(20)` 이 매 프레임 끝나고 paint 가 계속 온다 = 스레드는 산다).

### 전/후 (`wie_validate --inject` · release · 캡처 해시)

| 트리 | result | input_steps | paints | 캡처 | frozen_tail_steps |
|---|---|---|---|---|---|
| 전(`57506d11`) ×2 · load1 106~370 | PASS · rc=0 | 27/27 | 552 / 498 | 00_boot 만 다르고 **01~27·t5·t10·t15 전부 같은 프레임** | (필드 없음 · 같은 술어로 캡처 해시에서 계산하면 **26**) |
| 후 ×4 · load1 537~ | **FAIL** · stop error | 2~3 | 86~165 | 키마다 다른 프레임(로고 애니메이션) | 0 |

후 4판 모두 같은 새 벽 `java/util/Vector vtable index 17` 에서 멈춘다. PASS → FAIL 은 **후퇴가 아니라 교정**이다 — 전의 PASS 는 «멈추지 않았다»였지 «반응했다»가 아니었다.

### 변이(각 반쪽이 테스트에 잡히는가)

- `CardCanvas` 의 `clipRect` 를 전체 영역으로 → `test_card_canvas_offsets_clips_and_isolates_graphics_state` **FAILED**
- `paint_clip` 이 늘 전체를 돌려주게 → `paint_clip_is_the_union_of_repaint_areas_since_the_last_paint` **FAILED**

### 바꾼 기대값 하나 — 숨기지 않는다

`test_card_canvas_offsets_and_isolates_graphics_state` 는 upstream `7304facd` 이 «들어온 클립 밖 픽셀도 칠해진다»로 뒤집어 둔 테스트다.
그 커밋 당시 upstream `Display` 는 **늘 전체 클립**을 넘겼으므로(지금 upstream `main` 도 dirty 영역 추적이 없다 — 실측) 실제 실행에서 그 커밋이 바꾼 것은
**카드 경계 자르기**뿐이었다. 이 회차는 그 부분을 그대로 두고, «들어온 클립 = repaint 영역»만 지킨다. 테스트 이름을 커밋 이전 이름(`…_offsets_clips_and_…`)으로 되돌리고
클립 밖 세 픽셀의 기대값을 검정으로 고쳤다. **upstream 과의 차이**: `display.rs`·`card_canvas.rs` 두 파일 — realign 시 이 회차를 다시 얹어야 한다.

### validator — `frozen_tail_steps`(보고 전용)

`--inject` 의 스텝 캡처(`00_boot` + 스텝마다 1장 · `--shot-every` 제외)를 해시해, **끝에서부터** 직전 스텝과 같은 프레임이 몇 개 이어지는지 센다. `--shotdir` 없이도 잰다.
판정은 바꾸지 않는다: 스크립트 끝의 키 몇 개를 무시하는 타이틀은 정상이고(`keydraw_ktf` 도 1 이다), 문턱 없는 수는 실패를 만들 수 없다(`last_frame_*` 선례).
이번 멈춤은 26/27 이라 «PASS 인데 26 이면 의심» 정도로 읽힌다.

### 게이트 · 러너 · smoke

- 4게이트: `fmt` 0 · `clippy --all -D warnings` 0 · wasm32 clippy 0 · beta clippy 0 · `RUST_MIN_STACK=4194304 cargo test --all` **589 passed / 0 failed**.
- 러너: `draw_j2me` PASS · `helloworld_ktf`/`_lgt` PASS(clean exit) · `text_j2me` PASS · `keydraw_ktf --inject --expect-last-frame` PASS rc=0.
  `keydraw_lgt` 는 **UNMEASURED(max-ticks · 6~11/27)** ×4 — load1 530~683. ⑶ 대조: `target-pre`(2026-09-27 빌드 · 이 변경 이전 · main 과 정확히 같은 트리는 아니다)도 ×3 **같은 UNMEASURED(4~5/27)** ⇒ 부하 굶주림이지 이 변경이 아니다.
- smoke gate(`scripts/smoke_gate.sh` · 로컬 working 카탈로그 · boot+render): debug 바이너리로 working 카탈로그 **294판**(ktf 190 · lgt 54 · skt 50) 실행 — PASS 289 · FAIL 5
  (전부 `ktf/`). 5건 모두 release 로 다시 돌리면 **PASS**, `target-pre` 도 PASS(paints 비슷)이고,
  debug 로는 `--timeout 60` 에서 PASS(1건 확인 · 첫 paint 가 15초 뒤) ⇒ 부하 아래 debug 의 첫 paint 지연이지 이 변경이 아니다(load1 120~680).
  ★**스크립트의 «0 regressions» 는 인용하지 않는다** — `checked 1 baseline titles, 291 absent` 였다. 이 Mac 의 코퍼스 파일명은 NFD 이고 기준선은 NFC 라
  기준선 대조가 **1건**만 맞았다(실측: `working/skt` 파일명 `unicodedata.is_normalized('NFC')` = False). 위 294판 PASS/FAIL 은 스크립트의 실행 줄을 직접 센 값이다.

### 한계

- 원인을 측정한 타이틀은 하나다. 다른 타이틀의 개선은 주장하지 않는다.
- player-updates 는 추가하지 않았다 — 이용자에게는 아직 «플레이 불가»다.
- 다음 벽 `Vector 17` 은 이 회차 범위 밖(worklog 후속 제안 1).
- `smoke_gate.sh` 의 기준선 대조가 NFD 코퍼스에서 사실상 비어 있다(위 절) — 이 회차에서 고치지 않았다.

### 게임 파일명 유입 (`scripts/corpus-name-inflow.mjs --corpus <로컬 game_lab>`)

유입 BOUNDED + 판단 필요 SUFFIX-ATTACHED 수는 아래 표식 줄이 정본이다. 이 회차가 새로 더한 제목은 `월드장기체스` 하나
(display.rs 테스트 주석 1 · wie_validate.rs 주석 2 · 이 문서 · worklog)이고 0351·0358·0363 과 같은 자리다. BOUNDED 의 나머지
세 제목은 이 회차가 고친 파일(display.rs · wie_validate.rs)에 **원래 있던** 주석이다.
SUFFIX-ATTACHED 는 전부 `월드장기체스가`(이 회차가 쓴 조사 붙은 언급 — 실제 언급)다.
<!-- corpus-name-inflow v1 subjects=6 tree=2138db815a1aae62 B=11/8 P=0/0 S=4/2 -->
