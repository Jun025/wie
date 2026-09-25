## [2026-09-25] LGT `JavaClassDefinition::from_raw` 게스트 경계 전수 — 5곳 중 막힌 1 · 남은 구멍 1곳은 패닉 실측(값 코덱) (wie-2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured-adopt-p2)

**무엇을**: 게스트 레지스터·메모리에서 온 포인터로 LGT `JavaClassDefinition` 을 만든 뒤, 오류 통로가 없는 `ClassDefinition` trait 메서드(`name`·`super_class_name`·`interface_names` — `class_definition.rs:796-821`, 전부 `descriptor().unwrap()` 계열)에 닿는 호출부를 셌다. **코드 변경 0** — 고칠 자리는 worklog `proposals` 로 넘겼다.

**왜**: 제안 `2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured#p2` — #264 는 `java_is_class_assignable` 한 곳만 막았고 같은 모양이 몇 곳인지 센 적이 없었다.

**계수 명령**(@`origin/main` `7e97a7db`):

```text
git grep -n 'JavaClassDefinition::from_raw' -- 'wie-lgt/*.rs'   # 4 (그중 1 = 테스트 class_definition.rs:983)
git grep -n 'class_from_raw' -- wie-lgt                          # 래퍼 jvm_support.rs:70 + 호출 3
git grep -nE '\.class\(\)' -- wie-lgt                            # JavaClassInstance::class() 소비자 9 (테스트 1)
git grep -n 'object_from_raw' -- '*.rs'                          # 값 코덱 진입 3
git grep -n 'register_generated_class' -- wie-lgt                # 호출 2 (+ 테스트 3)
```

매크로·간접 호출: `from_raw` 는 매크로로 생성되지 않는다(정의는 `class_definition.rs:41` 하나). 간접 경로는 래퍼 `LgtJvmSupport::class_from_raw` 와 `JavaClassInstance::class()` 둘뿐이고, 둘 다 위 명령으로 끝까지 따라갔다.

**표 — 게스트 경계 5곳**:

| # | 파일:줄 | 포인터 출처 | 지금 | 정직한 답 |
|---|---|---|---|---|
| A | `interface.rs:231` `java_is_class_assignable` | 게스트 레지스터(던져진 객체의 클래스 워드) | **막힘**(#264 · `try_name` → 0) | 있음 — 「할당 불가」(적용 완료) |
| B | `interface.rs:354` `java_register_class` | 게스트 레지스터 | 클래스·디스크립터 두 홉은 `descriptor()?` 로 오류 전파 → C 로 | — (C 에 흡수) |
| C | `jvm_support.rs:240` `register_generated_class` 의 `ClassDefinition::name` | B(게스트 레지스터) · `lgt_class_loader.rs:105`(게스트 해시 표) | **열림** — B 경로에서 셋째 홉(`ptr_name` 문자열·UTF-8)이 깨지면 패닉. 로더 경로는 `find_raw_class`(`:72-88`)가 같은 세 홉을 `?` 로 먼저 읽어 안전 | 있음 — `try_name()?`(함수가 `Result` 반환) |
| D | `interface.rs:375` `java_resolve_class` | 게스트 레지스터(B 와 같은 값) | 사실상 도달 불가 — 직전 B→C 가 같은 이름을 방금 읽었다 | 있음 — `try_name()?`(C 와 같은 줄 교체) |
| E | `value.rs:33` `JavaValueCodec::object_from_raw` 의 `ClassDefinition::name` | 게스트 메모리 — `class_instance.rs:61` `class()` 가 인스턴스 헤더 → 디스패치 표 첫 워드로 읽은 클래스 포인터 | **열림 · 패닉 실측**(아래) | 있음 — `None`(함수 자신의 null-on-invalid 계약 · `:26` 의 `class()` 실패와 같은 처분) |

E 에 닿는 진입: `wie-jvm-support/src/native.rs:30`(게스트가 넘기는 **모든** Class/Array 워드) · `jvm_support.rs:79` `class_instance_from_raw` · `method.rs:164`(게스트 예외 디코드). ⇒ 호출 빈도로는 A 보다 E 가 압도적으로 넓다.

**E 패닉 실측**(스크래치 테스트 · 커밋 안 함 · 실행 후 `git checkout` 원복): `value.rs` 테스트 모듈의 `live_instance` 로 살아 있는 인스턴스를 만들고, 디스패치 표를 첫 워드 = `0x104c_02b4`(놈3 이 던진 클래스 워드) 인 새 4바이트 칸으로 바꾼 뒤 `codec.decode_word(live, Class("java/lang/Object"))` ⇒

```text
panicked at wie-lgt/src/runtime/java/jvm_support/class_definition.rs:797:25:
called `Result::unwrap()` on an `Err` value: InvalidMemoryAccess(273416884)
```

(273416884 = 0x104c02b4.) `class()` 는 성공하므로 `:26` 의 방어가 못 본다 — #264 가 고친 «`ptr_class_name` 은 `?` 인데 `ptr_class` 는 패닉» 과 같은 비대칭이다.

**경계로 세지 않은 것 — 이유와 함께**:
- `class_instance.rs:94,107` · `array_class_instance.rs:69,73,99,108` — `class().unwrap()` 후 trait 소비. 여기 도달한 인스턴스는 E 를 통과한 객체다(E 가 고쳐지면 상류에서 걸러진다). `class_definition()` 은 `Box<dyn ClassDefinition>` 반환이라 오류 통로가 없다 ⇒ 정직한 답 없음 · 패닉 유지가 정직.
- `class_instance.rs:74` `storage_size` — `?` 전파. 안전.
- trait 내부 `super_class_name`(`:800`)·`interface_names`(`:811`) — 등록된 생성 클래스의 디스크립터(게스트 데이터)를 두 번째 홉으로 따라간다. 반환형이 `Option<String>`·`Vec<String>` 이라 «읽을 수 없음» 을 말할 자리가 없고 `None` 은 «부모가 없다(= Object 직계)» 라는 거짓이 된다 ⇒ 정직한 답 없음 · 패닉 유지. C 가 등록 전에 이름을 검증하는 것이 이쪽의 유일한 상류 방어다.
- KTF — 구현이 별개(`wie-ktf/.../class_definition.rs:289-290` 의 trait `name()` 도 `unwrap`). `from_raw` 11곳 · 이번 target 밖 ⇒ worklog 제안으로 넘김.

**사용자 영향**: 없음(문서만). 다음 회차가 E·C 를 닫으면, 놈3 과 같은 «엉뚱한 클래스 워드» 가 예외 경로가 아닌 일반 인자 경로로 들어와도 에뮬레이터 전체가 죽지 않는다.

**후속**: `docs/worklog/2026-09-25-lgt-class-from-raw-guest-boundary-census.json` — p0(E) · p1(C·D) · p2(KTF 계수).
