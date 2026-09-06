## [2026-09-07] 카드 신원을 `class_definition().name()` 으로 — 「되돌릴 것」 자체를 없앴다 (wie-clet-card-identity-use-class-definition-name)

**무엇을**: `wie_wipi_java/.../net/wie/card_canvas.rs` 한 파일(**+44 / −24**). `push_card` 가 쓰던
`getClass()` → `getName()` **`invoke_virtual` 2회 + `JavaLangString::to_rust_string`** 을
★**`c.as_ref().class_definition().name()` 한 줄**로 바꾸고, `is_clet_card` 에서 **`replace('.', "/")` 를 지웠다.**
쓰이지 않게 된 import 3개(`Class`·`String`·`JavaLangString`)도 함께 빠졌다.

**왜**: 2026-09-05 검은 화면의 근인은 **바이너리 형식(`net.wie.CletWrapperCard`) ↔ 슬래시 리터럴** 불일치였고,
1차 처방은 그 형식을 **되돌려서** 맞췄다. ★**이 회차는 되돌릴 것 자체를 없앤다** — 형식 불일치가
«방어 대상»이 아니라 ★**«비존재»** 가 된다.

**F1 — 두 경로를 «내가» 프로브했다**(이 변경의 안전성 전부가 여기 걸려 있다):

| 경로 | 픽스처 | `class_definition().name()` | `getClass().getName()` |
|---|---|---|---|
| **LGT** | `helloworld_lgt.zip` | ★**`net/wie/CletWrapperCard`** | `net.wie.CletWrapperCard` |
| **KTF** | `keydraw_ktf.zip` | ★**`CletCard`** | `CletCard` |

★**둘 다 «내부 형식»** ⇒ 전제 성립. ※`helloworld_ktf.zip` 은 `pushCard` 에 **도달하지 않는다** — KTF 프로브는
`keydraw_ktf.zip` 으로 얻었다(한쪽만 재고 넘어가지 말라는 요구가 여기서 값했다).

**개악 대조 2종**:
⒜**M1 — 정규화 재도입**(`replace('.', "/")`) → ★`binary_names_do_not_match_by_design` **FAILED**
(`assertion failed: !is_clet_card("net.wie.CletWrapperCard")` · 2 passed / 1 failed).
⒝**M2 — 호출부를 바이너리로 되돌림** → ★**2026-09-05 서명이 그대로 재현**: LGT `last_frame_content=false` ·
`paints=83`(= 55 + MIDP 덮어쓰기 28) ↔ KTF 무영향(55 · 패키지가 없어 두 형식이 같다).
게이트 형태 `--expect-last-frame` 로는 **LGT rc=1** · 원복 후 **rc=0**.

**사용자 영향**: 없다(동작 동일 · 러너 5픽스처 전건 PASS · `paints` LGT/KTF **55/55** = 기준선 그대로).

**★핀 결합을 적었다(고치지 않았다)**: `ClassInstance::class_definition()`·`ClassDefinition::name()` 은
**핀된 RustJava(`dlunch/RustJava@5b84dd1`)** 의 API 다 ⇒ ★**핀을 옮기면 「`name()` 이 두 캐리어 모두에서
내부 형식을 주는가」를 다시 재야 한다.** 그 문장을 `is_clet_card` 주석에 **PIN COUPLING** 으로 박았다.

**★하드코딩 두 이름은 그대로다** — 그 축(`#p1`)은 선행 회차(PR #95)가 **기각**했고 이 회차는 다시 열지 않았다.

**★측정 실수 하나 — 숨기지 않는다**: 중간에 `paints` 를 **45/43** 으로 읽고 「제거한 JVM 호출 탓」이라고
결론지었는데, ★**틀렸다.** `wie_validate` 의 예산은 **wall-clock**(`--timeout`)이라 빌드 중 측정이 값을 낮췄다.
같은 트리에서 3회 반복하면 **55/55** 로 기준선과 같다. ⇒ ★**단발 측정으로 인과를 적지 마라.**
