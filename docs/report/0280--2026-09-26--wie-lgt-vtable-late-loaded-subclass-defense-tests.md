## [2026-09-26] LGT vtable: 나중에 로드되는 컴파일 하위 방어 두 곳 + ABI 고정 칸 전수 대조 시험 (wie-lgt-vtable-late-loaded-subclass-defense-tests)

**무엇을**: #281(부모에 덧붙인 vtable 칸을 하위로 전파) 게이트② 검수(`~/orchestrator/reports/wie-2026-09-22-lgt-vtable-index-family-runtime-thread-string-adopt-p1.review.md` F1·F3·F4)의 후속.
시험 3개를 `wie-lgt/src/runtime/java/jvm_support.rs` 에 더했고, `generatedClasses` 버킷 순회 사본을 헬퍼 1개로 합쳤다.
제품 동작은 바뀌지 않는다.

**왜**: #281 은 «이미 로드된 하위»가 칸을 받는지는 잠갔다. «아직 로드되지 않은 컴파일 하위» 쪽 방어 두 곳은 잠그지 않았다.
하나는 미로드 컴파일 하위 bound(M3)이고, 다른 하나는 `build_from_compiler_vtable` 의 부모 꼬리 상속(M4)이다.
검수 때 둘 다 지워도 전 스위트가 green 이었다. 이 방어가 무너지면 벽(Unimplemented)이 나지 않는다. 대신 **엉뚱한 메서드가 조용히 호출된다.**

**사용자 영향**: 없음(시험 + 동작 불변 리팩터). 앞으로 이 두 방어나 ABI 행 적용이 깨지면 CI 가 red 가 된다.

### 더한 시험

| 시험 | 합성 입력(게임 바이트 0) | 잠그는 것 |
|---|---|---|
| `virtual_method_appended_to_a_parent_skips_slots_an_unloaded_compiled_subclass_owns` | `Component` 하위 가짜 생성 클래스. 컴파일 표 길이 = 부모+4 · 부모 길이 칸에 자기 메서드 · 실제 `LgtClassLoader` 의 `generatedClasses` 버킷에 심고 **미등록** | 링크 index ≥ 컴파일 길이 · 나중 로드 뒤 자기 칸 보존 + index 칸 = `getWidth` |
| `compiled_subclass_loaded_after_a_parent_append_inherits_the_appended_slot` | 같은 하위. 컴파일 표 길이 = 부모 길이 · 전 칸 0 | 링크 **뒤** 로드된 하위의 표가 index 를 덮고 그 칸이 `getWidth` |
| `every_abi_vtable_row_is_the_slot_linking_resolves_to` | `data/lgt_java_abi.toml` 전 행(인터페이스 제외) | 행 index = 이름 링크 index(`virtual_method_index`) = raw 표 target 의 메서드 |

### 변이(원복 후 green)

| # | 개악 | 결과 |
|---|---|---|
| M3 | `unloaded_compiled_subclass_vtable_bound(…)` → `.max(0)` | **1 FAILED**(첫 시험: `index 28 lands inside …'s compiled range 0..32`) · 40 passed |
| M4 | `build_from_compiler_vtable` 꼬리 상속 3줄 삭제 | **2 FAILED**(둘째: `built with 28 slots; index 28 is past its table` · 첫째: index 32 칸 불일치) · 39 passed |
| A1 | `build_methods` 가 `java/util/Random` 행만 무시(`confirmed_index` 필터) | **1 FAILED**(ABI 시험만: `java/util/Random nextInt()I: row index 12 · linked 12 · slot 12 holds nextInt()I (raw target differs)`) · 40 passed |
| A0 | toml 행 `Random.nextInt` 12 → 13 | **통과** — 아래 한계 ⑴ |
| A2 | 전 행 index +1 | 스택 오버플로(red 이지만 JVM 부팅부터 무너져 시험 판별이 아님) |

### ABI 대조 결과 — 어긋난 행 0

수리 대상 행은 **없다**. 비교에서 뺀 행이 1건 있다. `java/lang/Runnable run()V`(index 10)이다.
이 행은 인터페이스 자기 표의 칸이 아니다. **구현 클래스 컴파일 표의 칸 이름**이다.
`prepare_generated` 가 AOT 메서드에 이름을 붙일 때만 읽는다. 인터페이스 자기 표에서는 `run` 이 0 번에 온다(`build_interface_methods`).
그래서 「linked 0 ≠ 10」은 드리프트가 아니라 비교 축이 다르다는 뜻이다. 그 경로는 기존 `generated_class_exposes_compiler_vtable_methods_to_jvm` 이 잡는다. 이 시험은 `run` 을 index 10 에 두고 인터페이스로 디스패치한다.
시험은 뺀 목록을 `["java/lang/Runnable"]` 로 단언한다. 인터페이스 행이 새로 생기면 조용히 빠지지 않고 red 가 된다.

### F3 — 추출함

검수 회신은 「사본 4곳」이라 했지만 실측은 **3곳**이다. `interface.rs:364` 는 필드를 한 번 읽을 뿐 순회가 아니다.
순회하는 곳은 `lgt_class_loader.rs` `find_raw_class`, `class_definition.rs` 인터페이스 참조 해석, `jvm_support.rs` `generated_class_pointers` 셋이다.
이 셋을 `jvm_support::find_generated_class(core, generated_classes, predicate) -> Result<Option<u32>>` 하나로 합쳤다.
술어가 참이면 **첫 일치에서 멈춘다**. 그래서 옛 두 사본의 조기 종료(그 뒤 체인을 읽지 않음)가 그대로 유지된다.
동작 불변의 증거는 기존 시험이다. `generated_class_lookup_includes_last_bucket`(마지막 버킷), 인터페이스 참조 해석(`generated_class_exposes_compiler_vtable_methods_to_jvm`), 그리고 위 세 시험이 모두 green 이다.

### 한계

- ⑴ **ABI 시험은 «행이 참인가»를 재지 못한다.** 표가 행에서 만들어진다. 그래서 행을 옮기면 칸도 따라 옮겨지고 시험은 통과한다(A0).
  이 시험이 잠그는 것은 **행이 런타임에 실제로 적용되는가**다. 같은 이름의 앞 칸이나 폴백 덧붙임으로 index 가 갈리지 않는지도 잠근다.
  행이 게스트 ABI 와 맞는지는 여전히 실측(PROBE·objdump)만 답한다.
- ⑵ **F4(비용)**: 폴백 링크 1회마다 부모 + 로드된 하위 전부의 vtable 을 새로 할당한다(옛 것은 해제하지 않는다 — 기존 패턴). 생성 클래스 전량도 2회 순회한다(`loaded_subclasses`·bound). 폴백 빈도가 낮아 영향이 실측된 적은 없다. 이 회차도 코드를 바꾸지 않았다.
- ⑶ M3 은 검수와 같은 자리를 `.max(0)` 으로 개악했다. 부속 함수까지 지우는 형태는 dead-code 경고만 늘 뿐 시험 결과는 같다.
