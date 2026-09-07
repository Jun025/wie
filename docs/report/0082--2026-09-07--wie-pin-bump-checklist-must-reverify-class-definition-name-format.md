## [2026-09-07] 핀 이동 비용 목록에 「`class_definition().name()` 형식 재검증」 1항 — ★**표가 세는 것은 «컴파일이 잡는 파열»뿐이다** (wie-pin-bump-checklist-must-reverify-class-definition-name-format)

- **무엇을**: `docs/upstream-realign-verdict.md` §8-4⑶ 의 비용 계단 표 **직후**에 항목 **1건**을 넣었다(+20/−0 · 삭제 **0줄** · hunk 1개). 내용은 ⒜그 표가 «컴파일 측정»만 센다는 것 ⒝카드 신원(`card_canvas.rs:272` → `is_clet_card`)이 그 표가 **구조적으로 못 보는** 결합 위에 서 있다는 것 ⒞★**두 캐리어 확인 명령과 오늘 나오는 값**.
- **왜**: `ClassDefinition::name()` 의 «형식»이 ★**계약이 아니라 관찰**이기 때문이다. 핀 `5b84dd1` 실측 — `jvm/src/class_definition.rs:12` 의 `fn name(&self) -> String;` 위에 doc 주석 **없음**(그 트레이트 11개 메서드 전부) · ★`jvm` 크레이트 전체 `///` **0건** · `README.md` 의 `name()` 언급 **0건** · `internal form` 을 말하는 유일한 줄(`jvm/src/type.rs:60`)은 `CONSTANT_Class_info`(JVMS 4.4.1) 이야기다. ⇒ ★**핀이 반환을 이진 형식으로 바꿔도 컴파일도 시험도 통과하고 신원 판정만 조용히 어긋난다** — 2026-09-05 검은 화면(브라우저에서만 보였고 네이티브 `result` 는 PASS)이 그 형상이다.
- **사용자 영향**: 없다(문서 1파일). 간접 영향은 핀 범프가 그 사고를 재현하는 경로에 **사람이 볼 항목**이 생겼다는 것.

★**확인 명령을 «적기만» 하지 않고 돌렸다** — `RUST_LOG=jvm=debug` + `Register class …Clet…`: LGT(`helloworld_lgt.zip`) → `net/wie/CletWrapper`·`net/wie/CletWrapperCard` · KTF(`keydraw_ktf.zip`) → `Clet`·`CletCard`. **전부 내부 형식** ⇒ 오늘 값이 `is_clet_card` 의 두 리터럴과 정합한다. ★**코드 변경 0** 으로 잰다.

★★**함정 하나를 찾아 항목에 못박았다**: `RUST_LOG=debug`(전체)로 재면 LGT 는 ★**점 형식 `net.wie.CletWrapperCard` 도 함께** 찍어 판정이 갈린다. 그래서 `jvm=debug` 로 좁히고 **JVM 자신의 `Register class` 줄**만 보라고 적었다 — 그 줄은 정의상 내부 형식이다.

★**핀 무접촉 · 신원 판정 코드 무접촉 · 기존 항목 무접촉 · 새 시험 0.** 상세 = `docs/worklog/2026-09-07-pin-bump-class-name-format-check.json`.
