## [2026-10-06] SKT XDisplay.drawImageEx — 로고에서 멈추던 1종이 게임 화면까지 (wie-skt-xdisplay-drawimageex-implement)

**무엇을**: `com/xce/lcdui/XDisplay.drawImageEx(Graphics, Image, int, int, Image, int×5)` 를 등록했다. 몸체는 `Graphics2D.getGraphics2D(gfx).drawImage(tx, ty, srcImage, sx, sy, sw, sh, mode)` 위임이고 둘째 인자(Image)는 쓰지 않는다. 모드 처리(COPY·XOR 그림 · AND·OR 경고 후 무동작)는 기존 `Graphics2D.drawImage` 한 곳에 그대로 둔다.

**왜**: 제안 `2026-10-06-input-none-17-recheck#p1`. `fb80e97cbc57` 이 `java.lang.NoSuchMethodError: com/xce/lcdui/XDisplay.drawImageEx` 로 로고에서 멈췄다(0453 §2 «엔진 — 없는 API»).

**사용자 영향**: 그 SKT 게임 1종이 로고를 지나 인트로와 게임 화면까지 간다. 키를 누르면 화면이 바뀐다.

### 1. 원인 계급
엔진 — 없는 API(0453 판정 그대로). base 로그에 `NoSuchMethodError … drawImageEx` 1줄, head 로그에 0줄.

### 2. 인자 의미 — 근거 둘
⑴ **KEmulator 구현**(`game_lab/vendor_sdk/ezi/` 의 SK-VM 에뮬레이터 `home/KEmulator-mmpp.jar` → `com/xce/lcdui/XDisplay.class`, `javap -c -p -l`). LocalVariableTable 이름이 `gfx, image, tx, ty, srcImage, sx, sy, sw, sh, mode` 이고 몸체는 로그 1줄 후 `com/skt/m/Graphics2D.getGraphics2D(gfx).drawImage(tx, ty, srcImage, sx, sy, sw, sh, mode)` 다. `image`(slot 1)는 읽지 않는다. 같은 클래스의 `clear`·`copyLCD` 도 이 저장소의 스텁과 서술자가 같다.
⑵ **게임 호출부**(`fb80e97cbc57` 의 `f.class`, 13곳). 첫 호출부는 둘째 인자에 `aconst_null` 을 넘긴다. 따라서 그 자리는 필수 마스크가 아니다. 넘기는 mode 는 `0`(12곳) · `1`(1곳) = `DRAW_COPY` · `DRAW_AND`.
코퍼스 전체에서 `drawImageEx` 를 부르는 타이틀은 이 1종뿐이다(전 zip/jar 의 class 바이트 검색 · 나머지 2건은 vendor_sdk 의 KEmulator 자신).
남김: KEmulator 는 제3자 재구현이고 원 단말 문서는 이 저장소에 없다. 둘째 Image 가 원 단말에서 무엇이었는지는 확인하지 못했다. 다만 게임이 null 을 넘기는 호출이 있으므로 «쓰지 않음»이 그 호출과 모순되지 않는다. `DRAW_AND` 가 무동작인 것은 이 회차 이전부터의 `Graphics2D` 한계다(그 호출부는 1픽셀 높이 띠를 반복해서 그린다).

### 3. 측정 — 같은 타이틀 base ↔ head
바이너리: `wie_validate` release — base `21b1aef9`(origin/main) · head = base + 이 회차. 명령 `wie_validate --inject --shotdir <d> <file>`(기본 20초 · 27키) · 짝 2회 · build-slot short 단발 순차 · load1 14.6.

| | result | paints | frozen_tail_steps | center_nonuniform | 마지막 화면 |
|---|---|---|---|---|---|
| base 1·2 | PASS · PASS | 14 · 14 | 27 · 27 | 5.7% | 제작사 로고 |
| head 1·2 | PASS · PASS | 1107 · 1093 | 0 · 0 | 81.3% | 게임 화면(맵 · 상태바) |

base 의 PASS 는 «로고 한 장을 그리고 안 죽었다»일 뿐이다 — 27키 내내 화면이 그대로다(`frozen_tail_steps 27`).

### 4. 개악 대조
`draw_image_ex` 몸체를 `return Ok(())` 로 바꾸면 새 테스트 `draw_image_ex_draws_the_source_region_at_the_target_and_ignores_the_second_image` 가 **red**(`left: (0, 0, 0)` · `right: (171, 205, 239)`). 원복 후 green.

### 5. compat
head 바이너리로 `scripts/playability-census.mjs run`(이 1종 · probe A/B 30초 + longplay 600초 · build-slot `--long` — long 풀이 만석이라 대기 상한 1800초 뒤 `over` 로 돌았고 census 호스트 잠금을 다른 회차 뒤에서 기다렸다 · load1 8) → `report`. 판정: boot ok · render ok · input ok(A 새 그림 26) · longplay ok(600초 생존 · 854/900키 · `stop: deadline`) · sound silent · speed ok(ratio 0.999).
`compat.json` 은 이 1행만 바꿨다: status limited → **playable** · input no → ok · longplay unknown → ok · speed unknown → ok · knownIssues_ko 에서 «키를 눌러도 화면이 바뀌지 않을 수 있어요.» 삭제. sound 는 `no` 그대로(census `silent` = 계약 어휘 `no`). progress 는 이 회차에 재지 않아 키를 넣지 않았다.
★`report --compat <경로>` 는 그 경로를 **측정한 행만으로 덮어쓴다**(1행 파일이 된다). 저장소 파일에 바로 주지 말고 사본에 받아 행을 옮겨라.
`node scripts/player-data.mjs` OK(429 · playable 401 · limited 14 · not-yet 14).
CHECK_COMPAT_REVERT

### 6. 게이트
fmt · clippy `-D warnings`(stable · wasm32 · beta) rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(715 passed · 0 failed). 러너 블록: `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf` PASS rc=0(paints 79) · `keydraw_lgt` PASS rc=0(paints 55) · `text_j2me` PASS.
