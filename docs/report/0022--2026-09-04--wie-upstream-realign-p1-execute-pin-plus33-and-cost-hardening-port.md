## [2026-09-04] P1 집행 — `Jun025/RustJava` fork 이탈(핀 `dlunch/RustJava@5b84dd1`) + 하드닝 3축 wie 로 이식 (wie-upstream-realign-p1-execute-pin-plus33-and-cost-hardening-port)
- **무엇을**: `Cargo.toml`(`[patch]` 표 **삭제** + base 5줄에 `rev` 핀 · 왜/한계 주석)·`Cargo.lock` · 호출부 **11개소 / 7파일** 수정 · **신규 `wie_jvm_support/src/hardening.rs`**(본문 103 · 시험 99) + 배선 19줄 · 정본 `docs/upstream-realign-verdict.md` **§9 추가** · `STATE.md`·`REPORT.md`·워크로그.
- **왜**: 총괄 결정(2026-09-04)이 §8-6 권고를 **채택**했다 — 갈래 ⒝ · 핀 **`5b84dd1`(+33)**. 근거 셋: ⑴+16 이 「산다」던 16건은 이미 fork main 안에 있었다 ⑵칸 F 는 +33 과 대가가 같은데 fork 의존을 유지한다 ⑶P2 가 이 머신에서 영구 불가라 계속 미루면 무기한 대기다.
- **★⑴ API 파열 — 예상 ≥7 ↔ 실제 «11개소 / 7파일»**: ⑴`from_classfile` 오류형 1 ⑵`Runtime::exit` 1 ⑶`attach_thread` async+`Option` 3 ⑷`ClassDefinition::{interface_names, prepare}` 2 impl ★⑸**`ClassInstance::{identity, shallow_clone}` 3 impl**(예상에 없었다) ★⑹**`ArrayClassInstance: ClassInstance` 승격** 1 impl 재구조화(예상에 없었다). ★**「≥7」이 하한 표기였던 것이 옳았다** — ⑸⑹은 `wie_jvm_support` 가 통과한 «뒤»에야 보이는 자리다(§8-7-2 가 그 성질을 미리 적었다).
- **★⑵ `shallow_clone` 은 시그니처를 채우는 일이 아니었다**: KTF·LGT 인스턴스는 **게스트 메모리**에 산다 ⇒ Rust 구조체 복제는 같은 주소를 가리켜 «복제본에 쓰면 원본이 바뀐다». **새 게스트 객체를 할당하고 필드 블록을 복사**하도록 구현했고, LGT 는 `instantiate` 안의 동기 할당 클로저를 `alloc_guest_object` 로 빼내 두 경로가 같은 모양을 쓰게 했다. 반대로 `identity` 는 공짜였다 — 게스트 주소가 곧 정체성이다.
- **★★⑶ 하드닝 6축: ⒤전부 사라졌다(프로브 재실행) · ⒥⒦를 재고 «3축 이식 · 2축 미이식 · 1축 불가»를 골랐다**:

  | 축 | ⒥이식 비용 | ⒦코퍼스 없이 잠금 | 처분 |
  |---|---|---|---|
  | `ByteArrayInputStream`/`arraycopy`/`StringBuffer.append([CII)` null NPE | 원본 합 13줄 | ✔ | ★**이식** |
  | `StringBuffer.insert` | 53줄(+시험 36) | ✔ | 미이식 |
  | `Timer.schedule(TimerTask,long)` | 17줄(+시험 26) | △ | 미이식 |
  | pending-thread GC 루트 | 34줄 중 **25줄이 `jvm` 크레이트 내부** | ✘ | ★**불가** |

  ★**고른 기준은 줄 수가 아니라 «실패의 등급»이다**: 이식한 3축은 null 이면 ★**호스트 프로세스가 패닉**하고(개악 대조로 `jvm/src/class_instance.rs:108` `Option::unwrap()` 재현), 미이식 2축은 **메서드 부재**라 Java 레벨에서 시끄럽게 잡힌다. ★축 5 는 wie 가 닿을 이음매가 없어 **fork 없이 영구 미복구** — 이 회차가 갚지 못한 유일한 값이다.
- **★★⑷ 이식이 «가능»했던 이유(구조적 발견)**: `JvmRuntime::find_rustjar_class`(**wie 쪽 코드**)가 `java_runtime::get_runtime_class_proto` 의 프로토를 JVM 에 넘기고 `JavaClassProto` 필드는 전부 `pub` 다 ⇒ ★**wie 가 그 사이에서 메서드 본문을 감쌀 수 있다.** ⇒ ★**「하드닝을 되돌리려면 fork 가 필요하다」는 참이 아니었다** — §8-6 은 그 가능성을 재지 않았다.
- **★⑸ 잠금**: `harden()` 은 **적용 개수를 돌려주고** 못 찾으면 `tracing::error!` 를 찍는다(★조용히 안 걸리는 것이 이 하드닝이 처음 사라진 방식이다). 시험 3종 전부 `run_jvm_test` = **게임 파일 0**. 개악(`harden` → `return 0`) 시 ⒜적용 수 `0 vs 1` red ⒝★**그 패닉이 그대로 재현**된다.
- **★⑹ fork 의존 소멸(실측)**: `Cargo.toml` `[patch]` **0** · `Cargo.lock` 의 `Jun025` **0건** · `Cargo.lock` RustJava source 6행 전건 `dlunch/RustJava?rev=5b84dd1c…` · `cargo tree` 상 5개 크레이트 전건 동일. (`Cargo.toml` 에 남은 `Jun025` 1건은 «무엇이 왜 없어졌나»를 적은 주석이다.)
- **사용자 영향**: 엔진 의존이 fork → upstream 핀으로 바뀌었고 upstream 33커밋을 얻었다. 게임 동작은 **미검증**(아래) — ktf·lgt helloworld 는 ok.
- **★★[2026-09-04 정정 · 게이트② `request-changes` 승계 — 같은 PR #71]** 위 서술 중 셋이 «참이 아니었다». ⒜★**축 5 「불가·영구 미복구」는 거짓** — 새 핀이 같은 창을 **다른 설계로 이미 닫아 놓았다**(`Thread::start` 의 `GlobalRef<Thread>` + `determine_garbage` 의 `global_references` 루트 순회 + `Drop` 해제). ★**왜 틀렸나: 13행 프로브는 «`pending` 이라는 fork 의 식별자»를 세지 «보호»를 세지 않는다.** ⇒ 상실은 6축이 아니라 **5축**이고 갚지 못한 값은 **미이식 2축뿐**이다. ⒝★**「ktf·lgt 둘 다 ok — 부팅 경로를 깨뜨리지 않았다」는 «macOS 한정»**이었다 — Windows 에서 `wie_ktf::test_helloworld` 가 **호스트 패닉**했다(`java.class.path` 를 하드코딩 `:` 로 만드는데 새 핀은 `File.pathSeparator`(Windows `;`)로 쪼갠다). ⒞★★**J2ME 게스트가 «아예 못 떴다»** — `wie_validate` 로 1명령 재현되는데 이 회차가 **repo 자신의 러너를 돌리지 않았다.** ⒟`ci-presence` 를 묻지 않았고 실제로 **CI red 인 PR 을 「완료」로 냈다.** 승계 회차가 넷 다 고쳤다.
- **★★남는 구멍 — green 을 하드닝 보존의 증거로 읽지 마라**: 4게이트 green · `cargo test --all` **133 passed**(직전 130 + 3)이지만 §8-6 이 「4게이트도 `cargo test --all` 도 green 인 채로 사라진다」고 이미 못박았다. ⇒ ★**green 은 «회귀 없음»이지 «하드닝 보존»이 아니다.** 보존의 증거는 프로브와 개악 대조뿐이고 ★**축 5·8·9 에는 그 증거가 없다.** 그리고 ★**코퍼스 축(P2)은 여전히 이 머신에서 불가**다 — 이 회차가 바꾸지 않았다.

