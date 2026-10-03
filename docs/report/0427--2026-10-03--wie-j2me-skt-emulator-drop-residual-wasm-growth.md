## [2026-10-03] JVM 해제 시 자바 객체 순환 절단 — J2ME·SKT 부팅당 ~0.5 MiB 잔류 제거 (wie-j2me-skt-emulator-drop-residual-wasm-growth)

0426 후속 제안 `2026-10-03-emulator-drop-cycles#p0`. 0426 이 작업·코어 순환을 끊은 뒤에도 J2ME 는 부팅당 남았다.

### 잔여 보유자 — 무엇이 붙잡고 있었나

임시 계측(커밋 안 함 · `jvm`/`jvm-bytecode` 사본을 `[patch]` 로 물려 생존 수를 셈 · `draw_j2me.jar` 5부팅):

| 잰 것 | 결과 |
|---|---|
| 플랫폼(→ `System`) 해제 | **안 됨** — 1틱 부팅부터 |
| `JvmInner` 생존 | **0** — JVM 자체는 풀린다 |
| 부팅당 잔류(전역 할당자 계수) | **~512 KB** · `ClassDefinitionImpl` 12 · 인스턴스 18 |
| 남은 클래스 | `Display`·`Graphics`·`Image`·`Font`·`DrawCanvas`·`PrintStream`·`OutputStreamWriter`·`FileOutputStream`·`FileDescriptor`·`String`·`Vector`·`net/wie/EventQueue` |

⇒ **자바 객체 순환**이다. `jvm` 크레이트는 객체를 마지막 `Arc` 가 갈 때만 푼다(`jvm-bytecode` 의 `ClassInstance::destroy` 는 no-op, `JvmInner` 에 `Drop` 없음) —
정적 필드가 자기 클래스 인스턴스를 쥐면(인스턴스는 클래스를 쥔다) · `Writer.lock = this` · `Display` ↔ 현재 `Canvas` 같은 순환은 JVM 보다 오래 산다.
그 클래스의 context(`WieJvmContext`·`JvmRuntime`)가 `System` 을, `System` 이 플랫폼을 쥐어 함께 남았다.
**원인은 상류 크레이트의 객체 모델이지만, 고치는 데 상류 변경은 필요 없었다** — wie 가 모든 클래스를 정의하는 자리(`JvmRuntime`)에 있다.

- 정적 필드만 비웠을 때: 12 → 11 클래스(`EventQueue` 만 풀림). 인스턴스 순환까지 끊어야 했다.
- 정적 필드에서 닿는 객체의 객체 필드·배열 원소까지 비웠을 때: 플랫폼 해제 · 잔류 **16 B**(계측 자체).

### 무엇을

`wie-jvm-support`:
- `DefinedClasses` — `JvmRuntime` 이 정의한 클래스(`find_rustjar_class`·`define_class` 가 돌려준 `InheritedMethods`)를 기록.
- `sever()` — 모든 정적 객체 필드와, 거기서 닿는 `jvm-bytecode` 객체의 객체 필드·객체 배열 원소를 `null` 로. 목록은 `take` 해 비운다(클래스 context 가 목록을 쥐는 순환).
- `SeverOnDrop` 을 `Jvm::new` 의 현재 작업 id 클로저에 넣어 **JVM 이 단독 소유** — `JvmInner` 가 drop 될 때 돈다. 이미 그때는 자바를 실행할 수 없다.
  에뮬레이터·`System` 에 새 훅을 달지 않았다.
- **KTF·LGT 무접촉**: 그 둘은 클래스·객체를 게스트 메모리에 정의한다 — 기록은 `InheritedMethods` 만, 걷기는 `ClassDefinitionImpl`/`ArrayClassDefinitionImpl` 객체만.
- SKT 는 J2ME 와 같은 경로(`JvmSupport::new_jvm` + `RustJavaJvmImplementation`)라 같이 고쳐진다. **SKT 픽스처가 없어 직접 재지 않았다.**

### 수치

**브라우저**(0426 과 같은 방식 · headless Chromium · `wie_featurephone` wasm · 한 페이지 · 전환 12 = 생성 → 120틱 → `free()` · 가져오기 12 = 6틱 → `free()` ·
`memory.buffer.byteLength` · base = `origin/main` `0e9fbaeb` · `host-load-guard --recovered` rc=0 에서):

| 파일 | base 전환1 → 전환12 → 가져오기12 | PR |
|---|---|---|
| draw_j2me.jar | 6.1 → 11.1 → **16.4 MiB** | 내내 **6.1 MiB** |
| text_j2me.jar | 6.1 → 11.1 → **16.4 MiB** | 내내 **6.1 MiB** |
| keydraw_lgt | 262.4 평탄 | 262.4 평탄 |
| keydraw_ktf | 263.1 평탄 | 263.1 평탄 |

⇒ 0426 의 J2ME 숫자(6.1 → 16.4)를 그대로 재현했고 PR 은 평탄. LGT·KTF 는 바이트 단위로 같다.

**되돌리면 red**: `wie-j2me/tests/test_emulator_freed.rs` — `sever()` 호출을 빼면 `the platform outlived its emulator after 1 ticks`.
플랫폼 해제를 보는 이유: `System` 이 풀렸다는 가장 싼 증거이고, 붙잡는 쪽이 무엇이든 그 끝에 플랫폼이 있다.

### 한계

- 실행 **중** GC 가 쓰레기로 판정한 순환도 같은 이유로 남는다(`destroy` no-op) — 상류 크레이트 몫이고 이번에 재지 않았다.
- 정적 필드에서 닿지 않는 순환(스레드 스택·전역 참조에서만 닿던 것)은 이 걷기가 못 본다 — 위 픽스처 둘에서는 잔류 0 으로 재었다.

### 사용자 영향

J2ME 게임을 한 탭에서 바꿔 띄우거나 가져올 때 쌓이던 메모리(부팅당 약 0.5 MiB)가 이제 쌓이지 않는다. LGT·KTF 는 0426 그대로.
