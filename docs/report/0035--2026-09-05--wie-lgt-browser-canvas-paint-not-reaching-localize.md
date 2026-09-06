## [2026-09-05] LGT 화면은 «닿지 않은» 것이 아니라 «닿은 뒤 덮인다» — 슬래시/점 한 글자에서 끊긴다 (wie-lgt-browser-canvas-paint-not-reaching-localize)
- **무엇을**: 양쪽 호스트에 **같은 단계 프로브 6종**을 넣어 LGT 브라우저 검은 화면을 좁혔다. 남긴 것은 `scripts/contract-roundtrip.mjs` 헤더의 **CORRECTION 주석**뿐 — ★**제품 코드 변경 0**(프로브·반증 실험 전건 원복 · `WIEPROBE` 잔재 0).
- **왜**: 운영자 채택 제안 `2026-09-05-roundtrip-ktf-key-reach-scenario#p0`. ★**규명 회차이고 수정 회차가 아니다**(범위를 넓히면 회귀 책임을 못 가린다).
- **★★⑴ 선행 서술이 반증됐다**: 「LGT paint → WebScreen → canvas 에서 끊긴다」는 **틀렸다.** 브라우저에서 LGT 프레임은 **실제로 캔버스에 그려진다** — `WebScreen::paint incoming_nonblack=424` · `draw_image ok=true 240x320`. ★오히려 LGT 가 KTF보다 **두 번 더** 닿는다(P5 5 ↔ 3).
- **★★⑵ 끊기는 지점 = 그 «직후»의 MIDP 덮어쓰기**(브라우저 순서 추적 그대로): `P2 MC_grpFlushLcd` → `P5 incoming_nonblack=424` → `P5c ok` → `P3 disable_paint=false` → ★**`P5 incoming_nonblack=0`** → `P5c ok`. ⇒ 좋은 WIPI 프레임 위에 **비어 있는 MIDP `screenImage`** 가 덮여 **마지막 프레임이 검정**이다. KTF 는 같은 자리가 `disable_paint=true` 라 뒤따르는 blank blit 이 **없다**.
- **★★⑶ 근인 — 슬래시/점**: `wie_wipi_java/…/card_canvas.rs` 가 `class_name_str == "net/wie/CletWrapperCard"`(**슬래시**)를 보는데 `Class.getName()` 은 **`net.wie.CletWrapperCard`**(**점**)를 준다 ⇒ `disablePaint()` 가 **영원히** 호출되지 않는다. ★KTF 카드는 `CletCard`(패키지 없음)라 두 형식이 같아 **우연히** 통과한다.
- **★⑷ 반증 실험(주입 → 측정 → 원복)**: 점 형식 1개 추가 → 브라우저 LGT ★**0 px → 424 px**(KTF 와 동일 추적) · 네이티브 `disable_paint` false→**true**(28회) · `incoming_nonblack=0` paint **30 → 2** · `paints` **83 → 55**(KTF 와 정확히 같은 수). ⇒ ★지목이 틀렸다면 이 실험이 아무것도 바꾸지 않았어야 한다.
- **★⑸ 「네이티브는 정상」도 정확히 고쳐 적었다**: 네이티브에서도 **같은 덮어쓰기가 일어난다**(LGT blank paint 30건). `wie_validate` 가 PASS 인 것은 판정 축이 **«어느 한 프레임이라도» 내용이 있었나**(any/max)라서이고 ★**마지막 프레임을 보지 않기 때문**이다. ⇒ 호스트 차이가 아니라 **판정 축 차이**였다.
- **사용자 영향**: 이 회차는 없음(주석 1개). 대신 ★**「LGT 게임은 브라우저에서 화면이 안 나온다」가 «추정»에서 «지목된 한 줄»로** 바뀌었다.
- **★★[게이트② 반려 `-fix`] KTF 대조군의 «앞으로»를 적었다** — 종전에는 「KTF 는 패키지가 없어 **우연히** 통과한다」라는 «현재 사실»만 있었다. ★`CletCard` 는 **게스트에서 오는 이름**이므로(픽스처 `client.bin52` 상수풀에 실재) ★**카드 클래스에 패키지가 붙는 순간 KTF 도 «같은 형태로» 깨진다** ⇒ ★**처방 ⒜(점 형식 «추가»)는 그 «형식 취약성»을 그대로 남긴다.** 그 문장을 **제안 1 의 `tradeoff`**(수정 회차가 처방을 고르며 읽는 자리)와 `contract-roundtrip.mjs` 헤더(그 「우연히」 문장이 사는 자리)에 넣었다.
- **★남는 구멍**: 잰 것은 `keydraw_lgt.zip` **하나**(상용 코퍼스 0 · Constraint 9) · **수정안을 고르지 않았다**(점 추가 / 정규화 / 비교 폐기 — 수정 회차 판단) · 슬래시 리터럴이 «한때 맞았는지»는 미조사.

