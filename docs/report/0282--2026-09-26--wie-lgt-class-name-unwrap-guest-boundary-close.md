## [2026-09-26] LGT `ClassDefinition::name` 패닉 게스트 경계 2곳 닫기(값 코덱 → null · 등록/해석 → Err) + KTF `from_raw` 11곳 계수 (wie-lgt-class-name-unwrap-guest-boundary-close)

**무엇을**: `docs/report/0257` 의 표에서 «열림»으로 남았던 LGT 게스트 경계 E·C·D 를 닫았다. 모두 게스트 포인터에 대해 오류 통로가 없는 `ClassDefinition::name`(`class_definition.rs:801` `try_name().unwrap()`)을 부르던 자리다. 그리고 KTF 의 `JavaClassDefinition::from_raw` 11곳을 같은 형식의 표로 셌다. KTF 코드는 바꾸지 않았다.

**왜**: 채택 제안 `2026-09-25-lgt-class-from-raw-guest-boundary-census#p0`·`#p1`·`#p2`(운영자 cockpit 채택 2026-09-25T02:41Z).

**전제 재측**(@`origin/main` `5434ab0a`): 제안 근거는 그대로 참이다. 줄번호만 이동했다.
- E `value.rs:33` `ClassDefinition::name(&class)`: 같은 줄에 있다.
- C `register_generated_class` 의 `ClassDefinition::name(&definition)`: `jvm_support.rs:240` → **`:409`** 로 이동.
- D `java_resolve_class`: `interface.rs:375`, 같은 줄이다.
- 패닉 지점 `class_definition.rs:797` → **`:801`** 로 이동.
- 같은 처방의 티켓·PR 은 없다(`/usr/bin/grep -rlaF try_name` tasks·queue · 열린 PR 제목). 인접한 `wie-2026-09-26-class-definition-801-unwrap-panic-adopt-p1` 은 이 티켓에 `depends_on` 으로 걸린 «원인 추적» 티켓이다. 몫이 다르다(그쪽은 디스크립터 0 클래스 객체의 출처).

### 변경

| 자리 | 전 | 후 |
|---|---|---|
| E `wie-lgt/.../jvm_support/value.rs` `object_from_raw` | `ClassDefinition::name(&class)` 호출 시 패닉 | `class.try_name()` 이 실패하면 `None`(함수의 null-on-invalid 계약). 첫 1회만 `warn!`(참조 주소·클래스 워드·오류), 이후는 `trace!`. 이는 #264 `java_is_class_assignable` 의 1회 보고 패턴이다 |
| C `wie-lgt/.../jvm_support.rs` `register_generated_class` | `ClassDefinition::name(&definition)` | `definition.try_name()?` |
| D `wie-lgt/.../interface.rs` `java_resolve_class` | `ClassDefinition::name(&class_from_raw(..))` | `class_from_raw(..).try_name()?`. 이제 쓰지 않는 `ClassDefinition` import 도 뺐다 |

E 의 1회 보고는 `AtomicU32` 발생 계수로 한다(값이 0 에서 움직이는 1회만 warn). #264 는 `AtomicBool` 을 썼다. 계수로 둔 것은 시험이 «두 번 접혔다»를 셀 수 있게 하려는 것이고, 로그 동작은 같다. 프로세스 전역이며 타이틀별이 아니다(#264 와 같은 ponytail 판정).

### 시험과 개악 대조

- `value::tests::a_reference_whose_class_word_is_unreadable_decodes_to_null`
  - 살아 있는 인스턴스를 만들고, 디스패치 표 첫 워드를 놈3 의 `0x104c02b4` 로 바꾼다. 그 뒤 `decode_word(live, Class)` 는 두 번 모두 `Object(None)` 이고, 계수는 +2 다. 바꾸기 전 `Some` 도 함께 단언해서 «항상 null» 로 통과할 수 없게 했다.
  - 0257 에서 미커밋으로 남긴 스크래치 재현을 시험으로 고정한 것이다.
- `jvm_support::tests::register_generated_class_reports_an_unreadable_class_name`
  - 클래스·디스크립터는 읽히고 `ptr_name` 만 `0x104c02b4` 인 클래스를 만든다. 결과는 `Err(InvalidMemoryAccess(0x104c02b4))` 다.
  - p1 의 «이름 문자열 읽기 실패 → Err 전파» 시험이다.
  - D(`java_resolve_class`)는 SVC 경계라 따로 시험하지 않았다. 진입 직후 B→C 가 같은 이름을 먼저 읽고 실패하면 `?` 로 끝나므로, D 의 새 줄은 C 시험이 이미 덮는 입력에서만 실행된다.
- **개악 대조**: E 를 `Ok(ClassDefinition::name(&class))` 로, C 를 `ClassDefinition::name(&definition)` 으로 되돌리고 돌렸다(`cargo test -p wie-lgt --lib -- unreadable`). 새 시험 **2건 모두 FAILED** 였고, 둘 다 `class_definition.rs:801:25` 에서 패닉했다. 되돌리지 않은 기존 `try_name_reports_…` 1건만 ok 였다. 대조가 끝난 뒤 원복했다.
- 한계: warn 이 «정확히 1회» 라는 것은 로그를 잡아서 단언하지 않았다(`wie-lgt` dev-dependency 에 tracing 구독자가 없다). 단언하는 것은 계수와 `fetch_add(..) == 0` 분기다.

### KTF `JavaClassDefinition::from_raw` 11곳(p2 · 코드 변경 0)

계수 명령(@`5434ab0a`) `git grep -n 'JavaClassDefinition::from_raw' -- 'wie-ktf/*.rs'` 의 결과는 **11** 이고, 제안 수치와 같다.

KTF 는 LGT 와 구조가 다르다. 고유 메서드 `JavaClassDefinition::name() -> Result<String>`(`class_definition.rs:214`)이 따로 있어서 클래스·디스크립터·문자열 세 홉을 `?` 로 읽는다. 다만 **UTF-8 디코드는 `:220` `String::from_utf8(bytes).unwrap()`** 이다. 패닉하는 판은 trait `ClassDefinition::name`(`:289-290` `self.name().unwrap()`)·`super_class_name`(`:294`)·`interface_names`(`:317`)다.

| # | 파일:줄 | 입력 출처 | 이미 가드? |
|---|---|---|---|
| 1 | `interface.rs:398` `java_check_type` | 게스트 레지스터 `ptr_class` | 예 — 고유 `name()?`(UTF-8 만 unwrap) |
| 2 | `jvm_support.rs:196` 정적 필드 보유 클래스 등록 스캔 | client.bin 클래스 표를 게스트 술어 함수로 걷는다 | 예 — `name()?` 뒤 `register_class`(UTF-8 만 unwrap) |
| 3 | `jvm_support.rs:230` `KtfJvmSupport::class_from_raw`(래퍼) | 호출 5곳 `interface.rs:148·197·258·355·373` 전부 게스트 레지스터 | 예 — 5곳 모두 고유 `name()?` 또는 `field(..)?` 를 쓴다(UTF-8 만 unwrap). `:373` 의 `name()?[1..]` 은 빈 이름에서 슬라이스 패닉이 가능하다 |
| 4 | `jvm_support.rs:453` | 시험 코드(`#[cfg(test)]` `:289` 이후) | 해당 없음 |
| 5 | `array_class_definition.rs:29` `JavaArrayClassDefinition::from_raw`(래퍼) | `array_class_instance.rs:61·67·91` 이 `class_instance.class()` 로 읽은 게스트 메모리 | 부분 — `:61·67` 은 `class()?`, `:91` 은 `class().unwrap()` |
| 6 | `class_definition.rs:157` `read_class_hierarchy` | 게스트 디스크립터의 부모 사슬 | 예 — 매 홉 `read_generic(..)?` |
| 7 | `class_definition.rs:235` `parent_class` | 게스트 디스크립터 `ptr_parent_class` | 함수 자신은 예. 그러나 trait `super_class_name`(`:294`)이 `parent_class().unwrap()…name().unwrap()` 으로 소비한다 |
| 8 | `class_definition.rs:317` trait `interface_names` | 게스트 디스크립터 인터페이스 배열 | **아니오** — 읽기·`name()` 전부 unwrap(trait 가 `Vec<String>` 반환) |
| 9 | `class_instance.rs:56` `JavaClassInstance::class` | 게스트 인스턴스 헤더 `ptr_class` | 예 — `read_raw()?`. KTF 값 코덱(`value.rs:27`)은 `class().and_then(is_array)` 로 **세 홉을 모두 오류로 받는다**. ⇒ **LGT E 에 해당하는 KTF 자리는 이미 닫혀 있다** |
| 10 | `classes/net/wie/ktf_class_loader.rs:150` `findClass` | 게스트 `fn_get_class` 반환값 | **아니오** — `jvm.register_class(Box::new(class))` 로 그대로 넘겨 jvm 이 trait `name()` 을 부른다. 앞 줄 `run_function(..).await.unwrap()` 도 unwrap 이다 |
| 11 | `method.rs:239` 예외 catch 타입 대조 | 게스트 `ptr_class` | 예 — `name()?`(UTF-8 만 unwrap) |

**판정**:
- KTF 에서 LGT E(값 코덱) 급의 넓은 구멍은 **없다**.
- 가드되지 않은 게스트 경계는 #8(trait `interface_names`)과 #10(`findClass`) 두 곳이다. 여기에 공통 잔여로 고유 `name()` 의 UTF-8 unwrap(`:220`)이 있다.
- 이 셋 모두 코퍼스에서 도달이 관측된 적은 없다(코드 판독).
- 그래서 제안 문턱(«이번 회차 관측»)에 못 미친다고 보고 **worklog 제안 카드로 올리지 않았다**. 이 표가 기록이다. KTF 타이틀에서 같은 패닉이 관측되면 이 표의 #8·#10·`:220` 부터 본다.

### 사용자 영향

- LGT 타이틀이 깨진 클래스 워드를 가진 참조를 네이티브 인자·예외 디코드로 넘겨도 에뮬레이터 전체가 죽지 않는다. 그 참조는 null 로 읽히고(게스트 NPE), 콘솔에는 경고 1줄만 남는다.
- 게스트가 이름을 읽을 수 없는 클래스의 등록·해석을 요청하면, 패닉 대신 그 SVC 호출의 오류로 끝난다.
- 인접 티켓 `…-class-definition-801-unwrap-panic-adopt-p1` 참고: 배틀몬스터의 `:801` `InvalidMemoryAccess(0)` 패닉이 E·C·D 경로였다면, 이제 패닉이 아니라 이 warn 이나 Err 로 보인다. 그쪽 재현 판정은 warn 발화도 세야 한다.

### 검증
- 4게이트(@`5434ab0a` 위 이 브랜치): `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · wasm clippy rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` **456 passed · 0 failed** · `cargo +beta clippy --all -- -D warnings` rc=0.
- 러너 블록(엔진 변경 · loadavg 98): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf` PASS rc=0 paints 37 · `keydraw_lgt` PASS rc=0 paints 33(부하 하한 안) · `text_j2me` PASS.
- 게스트 파일은 쓰지 않았다(시험 입력은 전부 시험 안에서 합성했다).
