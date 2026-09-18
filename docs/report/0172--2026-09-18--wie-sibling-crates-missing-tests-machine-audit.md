## [2026-09-18] 개명이 떨어뜨린 시험을 «기계로» 물었다 — ★**파일 축은 0건이고, 함수 축이 10건을 냈다** (wie-sibling-crates-missing-tests-machine-audit)

**무엇을**: 채택 제안 `2026-09-18-restore-lgt-reach-tests#p0` 의 이행. ★**조사 회차다 — 아무것도 되살리지 않았다**(목록이 산출물).
제품 코드 **0줄** · 산출 = 이 문서 + 감사 술어(코드)와 전체 출력은 `~/orchestrator/reports/evidence/wie-sibling-crates-missing-tests-machine-audit/`.

**왜**: 되살린 두 파일(`wie-lgt/tests/test_{key,resource}_reach.rs`)은 ★**사람이 눈으로** 찾았다. 같은 방식으로 떨어진 것이 더 있는지 기계가 답해야 한다.

**사용자 영향**: 지금은 **0**(고친 것이 없다). 바뀐 것은 ★**「더 없다」를 말할 수 있는 근거**와, ★**그 근거가 원래 못 보던 축 하나**다.

### ★결론 두 줄

⑴★**base swap 이 «파일»로 떨어뜨린 시험은 정확히 2개**였고 **둘 다 이미 복원됐다** ⇒ 파일 축의 남은 결손 **0**.
⑵★★**그런데 «함수» 축을 재니 20개가 이름째 사라져 있었고, 그중 «대상 코드가 살아 있는데 단언만 없어진» 것이 10개다.**
  ⇒ ★**파일 축만 보고 「0건」이라 적었으면 그것은 거짓이었다.** 이 회차의 실제 산출물은 그 10개 목록이다.

### 술어 — 다섯 축. ★하나가 다른 하나를 대신하지 못한다

| 축 | 무엇을 보나 | 결과 |
|---|---|---|
| **A** 수집 안 됨 | 디스크의 `tests/*.rs` ↔ `cargo metadata` 의 test 타깃 | **3건**(전건 `0153` 이 이미 처분) |
| **B** swap 손실 | swap 부모 트리 → 합류 트리, `-M` 으로 rename 제외 | **3건 DROPPED** · 그중 지금도 없는 것 **0** |
| **C** 역사 전수 | 전 역사에서 rename 아닌 삭제 | **11건**(전건 처분 확인 — 아래) |
| **D** 빈 껍데기 | 수집되는데 `#[test]` 0개 | **0건** |
| ★**E** 함수 이름 | swap 직전 트리 ↔ HEAD 의 `#[test]` 함수 **이름 집합** | ★**20건 소실** |

★**A~D 는 전부 «파일»을 본다.** `#[cfg(test)] mod` 안의 유닛시험이 통째로 빠지면 **네 축 다 침묵한다** — E 가 그 사각이다.

### ★E 의 20건 — 대상 코드 생존 여부로 갈랐다

| 잃은 시험 | 당시 위치 | 대상 코드 | 판정 |
|---|---|---|---|
| `test_argb_abgr_roundtrip` · `test_rgb332_roundtrip` · `test_rgb565_roundtrip` · `test_rgb8_roundtrip` · `test_decode_image_png` · `test_decode_image_short_input_returns_err` · `test_decode_lbmp_rgb332` · `test_decode_lbmp_truncated_header_returns_err` | `wie_backend/src/canvas.rs` | ★**생존** — `fn decode_image` 가 `wie-backend/src/canvas.rs:902` 에 있고 `canvas/lbmp.rs`·`canvas/res.rs` 도 있다 | ★★**진짜 결손 8건.** HEAD `canvas.rs` 의 시험 **28건은 전부 draw/clip/arc/xor 계열**이고 decode·roundtrip **0** · `lbmp.rs` 시험 **0** · `res.rs` 1건(`decodes_static_icon_after_animation`) |
| `wipic_svc_0x581_maps_and_table_stays_fail_closed` | `wie_lgt/src/runtime/svc_ids.rs` | ★**생존** — `Unk16 = 0x581` 과 `try_from` 의 `Err` 경로(fail-closed)가 그대로다 | ★**진짜 결손 1건.** 그 파일의 현재 시험 **0** ⇒ 「표가 fail-closed 로 남는다」가 **아무 데서도 단언되지 않는다** |
| `test_helloworld_jar_named_application` | `wie_lgt/tests/test_helloworld.rs` | 생존 — HEAD 동명 파일에 `test_helloworld`·`test_helloworld_jar_under_p_prefix` **2건** | ★**진짜 결손 1건.** ★**LGT 시험 «세 번째»가 남아 있었다** — 눈으로 복원한 회차가 못 본 자리다 |
| `misc_unk9_error_names_the_module_index_and_arguments` | `wie_lgt/src/runtime/wipi_c.rs` | ★**소멸** — `misc_unk9_error` HEAD **0건** · `Unk9`(=`0x195`)는 이제 `database::list_record_info` 로 **구현**됐다 | 적용 불가(대상 함수가 없다) |
| `binary_names_do_not_match_by_design` · `internal_names_from_class_definition_match` · `unrelated_cards_do_not_match` | `wie_wipi_java/.../card_canvas.rs` | ★**소멸** — HEAD `card_canvas.rs` 에 이름 매칭 함수 0(`from_raw`·`from_midp_raw`·`as_proto`·이벤트 핸들러뿐) | 적용 불가 |
| `char_array_guest_layout` · `field_layout_inherited_first` · `per_class_override_slots` · `vtable_reserved_slot_zero` · `parse_descriptor_fixture` | `wie_lgt/src/runtime/java/native_jvm.rs`·`native_class.rs` | **파일 소멸** → `wie-lgt/src/runtime/java/jvm_support/*` 로 재구성 | 재구성 — 새 구조가 **시험 8건**을 자체 보유(`jvm_support.rs` 3 · `method.rs` 3 · `vtable.rs` 1 · `array_class_instance.rs` 1). ★**같은 «성질»을 여전히 단언하는지는 이 감사가 확정하지 않는다**(이름 축의 한계) |
| `read_null_terminated_string_handles_four_byte_boundaries` | `wie_util/src/lib.rs` | 생존 | 개명·재작성 — HEAD 에 `terminated_string_reads_stop_at_the_reader_boundary`·`terminated_string_reads_preserve_short_reads_and_errors` |

⇒ ★**대상 코드가 살아 있는데 단언만 사라진 것 = 8 + 1 + 1 = 10건.**

### A·B·C — 파일 축의 처분

**A(수집 안 됨) 3건**: `wie_jvm_support/tests/absent_string_buffer_insert.rs` · `…/absent_timer_schedule.rs` · `wie_midp/tests/create_image_missing_name_message.rs`.
★**새 사실이 아니다** — `docs/report/0153` 의 고아 전수(4건)가 이미 처분했고 그중 1건은 형제 회차가 되살렸다. 남은 3건은 ★**「되살리면 red」로 «판정된» 것**이지 방치가 아니다.
※그 3파일이 든 시험 함수 `string_buffer_insert`·`timer_schedule_one_shot` 은 **2026-09-16 이후 한 번도 돌지 않았다**(cargo 타깃 0).

**B(swap) 3건 DROPPED**: `wie_lgt/tests/test_{helloworld,key_reach,resource_reach}.rs`. 대칭으로 KTF 3건은 **rename 으로 이어졌다**(`R088`·`R100`·`R100`).
`test_helloworld.rs` 는 **다른 부모(upstream)가 같은 자리에 자기 판본을 넣어** 파일로는 메워졌고(그 대가가 위 E 의 `…_jar_named_application`),
나머지 2건이 눈으로 복원된 그 둘이다 ⇒ ★**파일 축 잔여 0.**

**C(역사 전수) 11건** — 삭제 커밋과 함수 생존으로 전건 처분:

| 파일 | 삭제 | 처분 |
|---|---|---|
| `wie-arm-wasm/tests/compile.rs`·`execution.rs` | 2026-09-10 `Replace ARM JIT with startup AOT` / `Simplify ARM JIT packaging and tests` | ★**크레이트째 소멸**(HEAD 파일 0) — 시험 51개가 함께 갔고 대상 코드도 갔다 |
| `wie_backend/tests/test_canvas.rs` | 2024-02-01 `Move canvas test to unit test` | 이동 확인 — `fn test_canvas` 가 `wie-backend/src/canvas.rs:961` 에 **실재** |
| `wie_core_arm/tests/allocator.rs` | 2024-01-07 `Move arm allocator to unit test` | 이동 확인 — `fn test_allocator` 가 `wie-core-arm/src/allocator/bucket.rs` 에 **2곳** |
| `wie_impl_java/tests/graphics.rs` | 2024-01-07 `Move java test to unit test` | 이동 확인 — `fn test_graphics` 가 `wie-midp/…/lcdui/graphics.rs:952` 에 |
| `wie_impl_java/tests/string.rs` | 2024-01-07 `Take out java runtime into external crate` | ★**코드와 함께 외부로** — 이 repo 는 `java/lang/String` 을 **구현하지 않는다**(`JavaLangString` 은 핀된 RustJava 것) |
| `wie_wipi_c/tests/kernel.rs` | 2024-09-16 `Reorganize test` | 이동 확인 — `fn test_sprintk` 가 `wie-wipi-c/src/api/kernel.rs:350` 에 |
| `wie_jvm_support/tests/absent_methods.rs` | 2026-09-04 (우리 회차) `시험 2건을 «한 바이너리»에서 갈라 놓는다` | **의도된 분할** — 그 둘이 위 A 의 고아 2건이다 |
| `wie_lgt/tests/*` 3건 | 2026-09-16 base swap | 위 B |

### ★술어가 «못 보는 것» — 이 목록이 「없다」의 증거가 되려면 함께 읽어야 한다

1. ★**E 는 «이름»으로 센다** — 시험을 **개명**하면 «잃음 1 + 새것 1»로 보인다. 그래서 20건을 전부 «대상 코드 생존»으로 한 번 더 갈랐고, 그 갈래가 10 대 10 이다.
   반대로 ★**이름이 같은데 본문이 비워진 것**은 E 가 못 본다.
2. ★**E 의 창은 swap 직전(`ac6e0705`)↔HEAD 하나**다. 그 이전의 손실은 C(파일 축)만 덮는다 — 전 역사 함수 축은 **재지 않았다**(4,649 커밋 × 트리 스캔).
3. ★**B 의 rename 판정은 git 유사도 휴리스틱**이라 **크게 다시 쓰인 이동은 «삭제»로 보인다**. 실측: `B-2` 가 `wie_lgt/tests/test_{helloworld,key_reach}.rs` 를 `STILL-GONE` 으로 냈는데 ★**둘 다 HEAD 에 있다**(복원본이 KTF 판본에서 다시 쓰여 유사도 미달). ⇒ ★**B 는 «상한»이지 «목록»이 아니다** — A(존재 여부)와 교차해서 읽어라.
4. ★**`git log --diff-filter=D -- <path>` 를 쓰지 마라 — 두 겹으로 거짓말한다.** ⒜history simplification 이 기본이라 곁가지를 지우고(`--full-history` 필요) ⒝**머지 커밋 «안»의 삭제는 `-m` 없이 아예 안 보인다.**
   ★**실측: base swap 은 머지 커밋이라 두 함정에 다 걸려 그 명령이 «0건»을 답한다.** 이 감사는 그래서 **트리 대 트리**로 묻는다.
5. 다른 시험 관용(`#[rstest]`·`proptest!`·`quickcheck`·doc-test)은 술어에 없다. ★**가정하지 않고 셌다** — HEAD 전수:
   `#[test]` **349** · `#[futures_test::test]` **46** · `rstest`/`proptest!`/`quickcheck` **0건** ·
   doc-test 는 `cargo test --all --doc` 이 **`Doc-tests` 블록 16개를 내고 전건 `0 passed`** 다.
   ⇒ 술어의 어휘 두 개가 이 repo 의 **전부**다. (반대로, 새 관용을 들이면 이 감사가 **조용히** 그만큼 눈멀게 된다.)

### ★후속(이 회차에서 하지 않은 것)

- ★**되살리기 0.** 10건은 **목록**이고, 되살리기는 별 회차다(티켓이 명시).
  ★되살릴 때 «되살리면 red」인지는 `0153` 이 고아 3건에 대해 한 그대로 **먼저 재야** 한다 — 8건의 canvas 시험은 2026-09-16 이전 API 위에 쓰였다.
- ★**이 감사를 «상설 검사기»로 만들지 않았다.** 그것은 별 결정이다(호출자·CI 배선·`checker-census` 의 no-caller 버킷 비용) — 제안으로 남긴다.
