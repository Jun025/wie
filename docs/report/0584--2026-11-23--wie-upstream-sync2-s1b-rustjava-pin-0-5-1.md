## [2026-11-23] 조각 S1b — RustJava 핀 =0.4.1 → =0.5.1 (upstream 이 쓰는 판 · 우리 쪽만 · 병합 커밋 아님) (wie-upstream-sync2-s1b-rustjava-pin-0-5-1)

**무엇을**: `docs/report/0551` §7 의 S1 가운데 남은 조각이다. 루트 `Cargo.toml` 의 `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` 을 `=0.4.1` 에서 **`=0.5.1`** 로 올리고, 그 판이 바꾼 API(`JavaType` 의 자리 · 날짜 라이브러리)를 upstream #1630 · #1631 이 고친 꼴로 따라갔다. `git merge upstream/main` 은 하지 않았다.

**왜**: 티켓 `wie-upstream-sync2-s1b-rustjava-pin-0-5-1`(채택 제안 `2026-11-22-upstream-sync2-s1-rustjava-pin-0-4-1#p0`). upstream 이 2026-11-22 에 `^0.5.1` 로 갔다. 병합 커밋(S3)이 Java 실행기 판 차이까지 지지 않게 병합 전에 우리 쪽에서 먼저 올린다. 0.4.2 가 더한 Java 스택 깊이 상한(512 프레임 → `StackOverflowError`)이 지원 목록에 닿는지가 이 조각이 재야 할 것이었다.

**사용자 영향**: 지원 목록의 판정이 바뀐 행은 전/후 짝으로 잰 146행(SKT 82 전부 · KTF · LGT 표본 57 · 가드 7)에서 **0**이고, 깊이 상한(512)에 닿은 행도 **0**이다(가장 깊은 값 45 · §7-1). ★**이용자에게 보이는 변화가 하나 있다 — 일부 판이 빨라진다.** 전 · 후를 같은 순간에 돌려 그리기(paints)가 두 번 모두 +10% 이상인 행이 SKT 6(+12 ~ 45%) · KTF 3(+28 ~ 91%) · LGT 2(+19 ~ 33%)이고, 열에 여덟(SKT 66/80 · KTF · LGT 표본 44/57)은 ±3% 안이다(§7-4). CPU 초는 전/후가 같다 — 0.5.0 의 조회 캐시가 한 바퀴의 일을 줄였다. 그래서 `docs/player-updates/2026-11-23-some-games-run-faster-after-java-engine-update.json`(`kind: speed`)을 더했다. `compat.json` 은 바뀌지 않는다. `Cargo.lock` 과 엔진 `*.rs` 가 바뀌므로 착지하면 otterpebble 핀 범프가 뜬다. wasm 아티팩트는 +0.71% 이고(§6) 감사 경고 하나(`chrono` unmaintained)가 빠진다.

| 축 | 전 | 후 |
|---|---|---|
| `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` | `=0.4.1` | **`=0.5.1`**(upstream `^0.5.1` · lock 은 같은 0.5.1) |
| `classfile`(간접) | 0.4.1 | 0.5.1 |
| 날짜 라이브러리 | `chrono 0.4.45`(`rustjava-runtime` + `wie-lgt`) | `jiff 1.0.2`(둘 다) — chrono 는 lock 에서 빠졌다 |
| lock 패키지 수 | 560 | 567 |
| `patches/` · `[patch.crates-io]` | 없음 | 없음 |

바뀐 파일(코드 커밋 `cbb7bcb5d` 의 `--numstat` · 그 뒤 문서 커밋이 `Cargo.toml` 의 주석 1줄 · `STATE.md` · 이 기록 · worklog · 고지를 더한다):

```
112	43	Cargo.lock
10	9	Cargo.toml
2	1	wie-jvm-support/src/native.rs
2	1	wie-ktf/src/runtime/java/jvm_support/array_class_definition.rs
2	1	wie-ktf/src/runtime/java/jvm_support/array_class_instance.rs
2	2	wie-ktf/src/runtime/java/jvm_support/class_definition.rs
2	2	wie-ktf/src/runtime/java/jvm_support/class_instance.rs
2	2	wie-ktf/src/runtime/java/jvm_support/method.rs
1	1	wie-lgt/Cargo.toml
2	1	wie-lgt/src/runtime/java/interface.rs
2	1	wie-lgt/src/runtime/java/jvm_support/array_class_definition.rs
2	1	wie-lgt/src/runtime/java/jvm_support/array_class_instance.rs
2	2	wie-lgt/src/runtime/java/jvm_support/class_definition.rs
2	2	wie-lgt/src/runtime/java/jvm_support/class_instance.rs
3	3	wie-lgt/src/runtime/java/jvm_support/field.rs
2	2	wie-lgt/src/runtime/java/jvm_support/method.rs
2	1	wie-lgt/src/runtime/java/jvm_support/value.rs
32	8	wie-lgt/src/runtime/stdlib.rs
```

### 1. 현행 — 고칠 것이 있었다

| 잰 것 | 값 |
|---|---|
| `origin/main` `3c3986da9` 의 핀(루트 `Cargo.toml` · `Cargo.lock`) | `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` **`=0.4.1`** · lock 0.4.1 · `classfile` 0.4.1(간접) |
| `upstream/main` `ea5640ec1`(2026-11-23 조회 · behind 248) 의 핀 | 같은 다섯 **`^0.5.1`** · lock 0.5.1 · `classfile` 0.5.1 |
| crates.io 의 최신(`cargo search`) | 0.5.1 — upstream 이 그 사이 더 올린 것은 없다 |
| upstream 이 올린 커밋 | `263e6ac9b`(#1630 · 0.4.2 → 0.5.0 · 15파일 +43/−36) · `24b670780`(#1631 · 0.5.0 → 0.5.1 + LGT `localtime` 의 chrono → jiff · 4파일) |

### 2. 핀이 바꾼 API 와 우리가 따라간 곳

| RustJava 의 변화 | 우리 쪽 |
|---|---|
| 0.5.0 — `JavaType` 이 `jvm` 에서 `jvm_types` 로 옮겨 갔다(`jvm::JavaType` 재수출 없음) | import 14파일(`wie-jvm-support/native.rs` · KTF `jvm_support` 5 · LGT `interface.rs` + `jvm_support` 7). upstream #1630 과 같은 꼴이다. `wie-jvm-support` 는 이미 `jvm-types` 에 의존하므로 매니페스트 줄은 더하지 않았다 |
| 0.5.0 — `JavaType::default()` 가 없어지고 `JavaValue::default_of(&JavaType)` | 1곳(LGT `JavaHostField::get` — upstream 에 없는 우리 코드) |
| 0.5.1 — `rustjava-runtime` 이 날짜 계산을 chrono 에서 jiff 로 | LGT `localtime` 을 #1631 과 같은 꼴로 옮겼다(`wie-lgt` 의 `chrono ^0.4` → `jiff ^1.0`). 옮기지 않으면 chrono 는 `wie-lgt` 하나를 위해 그래프에 남는다. 옮긴 뒤 lock 에서 chrono 는 빠졌다. 시험 `localtime_fills_tm_in_kst` 를 더했다(종전 시험 0) |

- 그 밖에 손댄 호출부는 **0**이다 — 위 15곳을 고친 뒤 `cargo check --all --all-targets` 가 통과했다. 필드 owner 인자 · `NativeFrame` 가드 · `collect_garbage` 의 `anyhow` 결과는 S1(0572)이 0.4.1 에서 이미 옮겼고 0.5.1 에서 서명이 같다.
- **`=0.5.1` 이지 `^0.5.1` 이 아니다.** 티켓의 문구는 «upstream 이 쓰는 `^0.5.1`» 이다. 판은 맞췄고(0.5.1 · upstream lock 과 같다) 연산자는 우리 것(`=`)을 지켰다 — 0572 §10-1(F5)이 정한 포크 차이다: dependabot 이 이 매니페스트를 매일 읽으므로 캐럿이면 `Cargo.lock` 한 줄만이 판을 붙든다. 오늘은 crates.io 최신이 0.5.1 이라 두 표기가 같은 판으로 풀린다.
- lock 에 새로 든 것: `jiff 1.0.2` · `jiff-core 1.0.1` · `jiff-static 1.0.2` · `parking_lot 0.13.0` · `parking_lot_core 0.10.0` · `lock_api 0.5.0`(RustJava 0.4.2 가 `parking_lot ^0.13` 으로 갔다 · 0.12 는 다른 의존이 여전히 쓴다) · `portable-atomic 1.15.0` · `portable-atomic-util 0.2.8`. 빠진 것: `chrono 0.4.45`. `cargo update -p <다섯 크레이트>` 가 같이 올린 것: `libc 0.2.189 → 0.2.191` · `redox_syscall 0.9.0 → 0.9.5`.

### 3. `patches/` 처분

| 대상 | 전 | 후 | 근거 |
|---|---|---|---|
| `patches/` 디렉터리 | **없다**(S1 이 `patches/classfile` 을 지웠다 — 0572 §3) | 없다 | `ls patches` → No such file or directory |
| 루트 `Cargo.toml` 의 `[patch.crates-io]` | 없다 | 없다 | `git grep -n '^\[patch' -- '*.toml'` 0줄 |
| `classfile`(S1 이 패치를 지운 그 크레이트) | crates.io 0.4.1 | crates.io **0.5.1** | `wie-jvm-support/tests/modified_utf8.rs`(`class_constants_decode_as_modified_utf8` — S1 이 패치의 시험 4종을 옮긴 것)가 0.5.1 에서 통과한다(§5) |

하나씩 확인할 «남은 패치»는 0건이다. 패치는 아니지만 «핀이 이러해서 둔다»고 적힌 우리 쪽 덧댐은 넷이고, 새 판에서의 처분은 §4 의 표에 있다.

### 4. 0.4.2 ~ 0.5.1 이 바꾼 동작과 우리 포크의 차이 — 소스로 대조

`~/.cargo/registry` 의 0.4.1 · 0.4.2 · 0.5.1 소스를 `diff -ru` 로 읽었다(jvm 0.4.1→0.4.2 62줄 · 0.4.2→0.5.1 502줄 · jvm-bytecode 314줄 · rustjava-runtime 258 + 183줄).

| 판 | 바뀐 동작 | 우리 쪽에서 닿는 곳 | 읽은 결과 |
|---|---|---|---|
| 0.4.2 | **Java 프레임 깊이 상한** — `execute_method` 가 프레임을 민 뒤 `thread.iter_frame().count() == 512` 이면 `StackOverflowError` 를 던진다 | 모든 통신사. 세는 것은 Java 프레임 + 네이티브 프레임(`push_native_frame`)이다 | §7 에서 쟀다. ★판정이 «== 512» 라서 네이티브 프레임이 511 → 512 를 밟으면 다음 Java 프레임은 513 이 되어 상한을 건너뛴다(우리 코드의 `push_native_frame` 호출 20곳이 그 자리다) — 상한은 «보장»이 아니다 |
| 0.4.2 | `Jvm::exception` 이 예외 객체를 만들다 실패하면 그 실패를 돌려준다(종전 `unwrap`) | 모든 `jvm.exception(..)` 호출 | 패닉 하나가 Java 예외로 바뀐다 |
| 0.4.2 | `java.lang.ref.Reference` · `WeakReference` 추가 · GC 가 약한 참조를 비운다 | GC(`collect_garbage`) | `classes.get("java/lang/ref/Reference")` 가 등록돼 있을 때만 돈다. KTF · LGT 는 이 클래스를 우리 proto 에서 받는다 |
| 0.4.2 | `Timer.scheduleAtFixedRate(TimerTask, Date, long)` 추가 · 음수 실행 시각 거부 · `FileInputStream.read` 의 범위 검사 · `ZipFile` 이 아카이브를 통째로 복사하지 않고 제자리에서 읽는다 · `ArrayIndexOutOfBoundsException(int)` | SKT · J2ME | 러너 블록 · 표본으로만 확인(§5 · §7) |
| 0.5.0 | **조회 캐시** — 필드 · 메서드 해소와 가상 호출의 선택 결과를 (수신 클래스 이름, 클래스, 이름, 서술자) 키로 `Jvm` 에 붙든다. 전제는 «등록된 클래스는 바뀌지 않고 정의는 불변» | ⑴ `InheritedMethods`(상속 메서드를 하위 클래스에서 답한다) · `Preemptible` · `ForcedBoolMethod`(`class-patches` 5행) — `method()` 가 부를 때마다 새 래퍼를 만든다. ⑵ KTF · LGT 의 `method()` · `field()` 는 부를 때마다 게스트 메모리의 표를 훑는다 | ⑴ 래퍼는 상태가 없거나 공유 계수기(`GuestCalls`)만 들어서 처음 것을 계속 써도 같다 — `a_guest_that_never_yields_is_preempted_between_method_calls` · `class_patches` 시험이 그 길을 탄다. ⑵ KTF · LGT 의 `JavaMethod` · `JavaField` 는 «게스트 주소»를 든 값이고 쓸 때마다 메모리에서 읽으므로, 표가 «제자리에서» 바뀌는 것은 캐시 뒤에도 보인다. 표 자체가 다른 주소로 갈리는 경우는 코드에서 찾지 못했고 표본으로 확인했다(§7). 캐시는 성공한 해소만 담는다 |
| 0.5.0 | **필드 저장이 칸(slot)** — `jvm-bytecode` 의 인스턴스 · 정적 필드가 `BTreeMap` 에서 «준비(prepare) 때 정한 배치의 칸»으로. 배치에 없는 필드를 읽으면 `unwrap` 패닉(종전에는 기본값) | `DefinedClasses::sever`(JVM 을 버릴 때 정적에서 닿는 객체의 필드를 비운다) · `InheritedMethods::prepare`(안쪽 정의에 넘긴다) | `sever` 가 읽는 필드는 그 객체의 클래스 사슬이 선언한 것뿐이고 그것이 곧 배치다. 준비 전에 만든 인스턴스는 `Jvm` API 로는 생기지 않는다(`instantiate_class` 가 `ensure_initialized` 를 먼저 부른다). 우리 코드가 `ClassDefinition::instantiate` 를 직접 부르는 곳은 LGT 의 자기 정의 1곳뿐이다 |
| 0.5.0 | 클래스 초기화에서 `prepare` 가 상위 클래스 초기화보다 «먼저»(JVMS 5.5) | KTF · LGT 의 `prepare` 는 빈 본문 | 영향 없음 |
| 0.5.0 | **GC 표시가 재귀에서 명시 스택으로** · 클래스별 참조 필드 목록을 한 수거 안에서 한 번만 구한다 · 정적 필드는 그 클래스가 선언한 것만(상위 것은 상위에서) | `guest_roots`(KTF · LGT) — `GuestRootsClass::fields()` 가 «수거 한 번에 한 번» 불린다는 가정 위에 있다 | 가정은 그대로 성립한다(`GuestRoots` 인스턴스는 JVM 당 하나이고 클래스 캐시는 수거마다 새로 만든다). `guest_roots` 시험 통과(§5). ★덤: 긴 참조 사슬에서 호스트 스택이 넘치던 길이 없어졌다 |
| 0.5.0 | `find_calling_class` 가 `this` 를 프레임의 첫 참조(`local_variables[0]`)에서 얻는다 | `current_class_loader`(클래스 로딩) | 인스턴스 메서드의 프레임은 인자 0 번이 수신자다 — 우리 호출은 전부 `invoke_virtual` · `invoke_special` 을 지난다 |
| 0.5.0 | `is_instance` 가 등록부가 아니라 인스턴스 «자신의» 정의에서 사슬을 탄다 · `java/lang/Object` 는 바로 참 | `InheritedMethods` 로 싼 클래스의 인스턴스는 안쪽 정의를 든다 | 이름 · 상위 · 인터페이스가 같아 결과가 같다 |
| 0.5.0 | 해석기: `tableswitch` 가 표를 훑지 않고 색인한다 · `iinc` 가 제자리에서 · 배열 저장의 형 변환이 명령으로 정해진다 · 예외가 난 pc 를 `next_index − 1` 에서 얻는다 | SKT · J2ME | 러너 블록 · 표본(§5 · §7) |
| 0.5.1 | `StringBuffer.append(boolean/int/long)` · `Vector.add*` 가 자기 가상 메서드를 다시 부르지 않고 직접 쓴다 · `Integer.equals` 가 필드를 읽는다 · 날짜가 jiff | SKT · J2ME(KTF · LGT 의 엔진 Java 클래스도 같은 proto 다) | 게임이 `StringBuffer` · `Vector` 를 상속해 `append(String)` · `ensureCapacity` 를 덮어쓴 경우에만 보이는 차이다. 그런 판을 코퍼스에서 세지는 않았다(§9) |

**핀에 기대어 둔 우리 쪽 덧댐 넷의 처분**

| 덧댐 | 왜 뒀나 | 0.5.1 에서 | 이 회차의 처분 |
|---|---|---|---|
| `wie-jvm-support/src/hardening.rs`(null 가드 · `InputStreamReader.read` 대체 본문) | 0.1.1 이 안 가진 가드 | 시험 8개 통과 — `every_guard_is_actually_applied` 가 가드가 실제로 붙는지 단언한다 | **둔다**(필요 여부는 가드별로 upstream 본문을 읽어야 한다 — 이 조각의 일이 아니다) |
| `InheritedMethods` 의 «상속 메서드를 하위 클래스에서 답하기» | 0.1.1 의 `invoke_special` 이 이름 붙은 클래스만 봤다 | 0.5.1 의 `resolve_method_ref` 는 상위 클래스를 탄다 — 제거 조건은 0.4.1 에서 이미 섰다(0572 §8) | **둔다**. 같은 구조체가 `Preemptible` · `ForcedBoolMethod` 를 싣고 있어 지우는 것은 «동작을 하나 더 바꾸는 일»이다 |
| `DefinedClasses::sever`(JVM 을 버릴 때 순환 끊기) | `jvm` 이 마지막 `Arc` 가 갈 때만 객체를 놓는다 | 그대로다. 새 조회 캐시 · 프레임의 `Arc<ResolvedMethod>` 는 `JvmInner` 안이라 JVM 과 같이 간다 | **둔다** |
| `wie_validate` 의 `java_exceptions` 집계 | `Jvm::exception` 의 로그 줄을 센다 | ★`StackOverflowError` 는 `Jvm::exception` 을 지나지 않는다(`new_class` 로 직접 만든다) — **이 집계는 깊이 상한에 닿은 것을 못 본다** | 고치지 않았다. 그래서 §7 은 스크래치 계측으로 쟀다 |

### 5. V0 — 네 게이트 · beta · 러너 블록 · CI

**게이트**(코드 머리 `cbb7bcb5d` · 한 번에 이어 돌림 · 그 전 트리에서도 같은 결과): `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — **58 스위트 · 880 passed · 0 failed**(첫 실행에 통과 · 고친 시험 0 · 더한 시험 1) · `cargo +beta clippy --all -- -D warnings` rc=0. beta 는 `unused dependency` 매니페스트 경고 9줄(`test-utils` · `wie-util` · `wie_cli`)을 찍는다 — 이 조각이 건드리지 않은 매니페스트이고 rc 는 0 이다. 그 뒤 `Cargo.toml` 의 주석 한 줄(이 기록의 번호 0583 → 0584 — 같은 번호가 main 에 먼저 착지했다)을 고친 트리에서 다섯 가지를 다시 돌렸고 값이 같다(880 passed).

**러너 블록**(`AGENTS.md` · 커밋된 픽스처 · debug 빌드 · 측정 스윕이 끝난 뒤):

| 픽스처 | 명령 | 결과 |
|---|---|---|
| `test_data/draw_j2me.jar` | `wie_validate <f>` | `PASS` · rc=0 · booted + rendered |
| `test_data/helloworld_ktf.zip` | 〃 | `PASS` · rc=0 · clean exit |
| `test_data/helloworld_lgt.zip` | 〃 | `PASS` · rc=0 · clean exit |
| `test_data/keydraw_ktf.zip` | `--inject --expect-last-frame` | `PASS` · rc=0 · `input_steps` 27/27 · `paints` 79 · `content` true · `last_frame_content` true |
| `test_data/keydraw_lgt.zip` | 〃 | `PASS` · rc=0 · 27/27 · `paints` 55 · true · true |
| `test_data/text_j2me.jar` | `--timeout 5` | `PASS` · rc=0 · booted + rendered |

load1 6.9(17:47). `keydraw_*` 의 `paints` 79 · 55 는 S1(0572 §6)의 값과 같다.

**PR 의 CI**(`gh pr checks 616 -R Jun025/wie`): 코드 커밋 `cbb7bcb5d` 에서 `build-web` pass(5m59s) · `contract` pass(7m32s — 실제 브라우저 왕복 포함) · `pr_audit` pass(12s — `Cargo.lock` 이 움직여 돈다) · `rust_ci (ubuntu-latest, stable)` pass(6m17s) · `dependabot` skipping. 나머지 `rust_ci` 다섯 레그는 2 ~ 5초의 빈 통과다(2026-11-07 부터 PR 에서는 일을 하지 않는다 — 플랫폼 보장이 아니다). `coverage` 는 PR 에서 돌지 않는다. 이 기록을 실은 문서 커밋의 값은 회신에 적는다.

### 6. 조각 검증식 (0551 §7 의 S1 행)

| 검증식 | 결과 |
|---|---|
| `cargo test -p wie-jvm-support` 의 `hardening` 시험(`AGENTS.md` 사건대장 «핀을 옮기면 다시 돌려라») | `cargo test --all` 안에서 `hardening::` **8개 통과**(`every_guard_is_actually_applied` 포함) · `reader_read_tests` 4개 통과 |
| `patches/classfile` 의 시험 4종(S1 이 `wie-jvm-support/tests/modified_utf8.rs` 로 옮긴 것) | `class_constants_decode_as_modified_utf8` 통과 — crates.io `classfile` 0.5.1 에서 |
| 장부 가드 · 조정 표 | `the_ledger_lists_exactly_*` 7개 · `class_patches` 3개 통과. 엔진 표 · 장부 · `compat.json` 을 고치지 않았다 |
| `guest_roots`(GC 루트 · KTF · LGT) · pacing | `guest_roots` 2개 · pacing 4개(`wie-j2me/tests/test_pacing.rs` 포함) 통과 |
| `node scripts/check-engine-contract.mjs`(wasm 표면 불변) | 이 브랜치에서 `npm run build:wasm` rc=0 → `engine contract static check — 143 pass, 0 violation(s)` rc=0 |
| wasm 아티팩트 크기(`web/src/wasm/wie_web_bg.wasm` · `wasm-opt -Oz` 뒤) | 전(`origin/main` `3c3986da9` · 스크래치 트리에서 같은 스크립트) 17,893,072 바이트 → 후 **18,019,325** 바이트(+126,253 · +0.71%) |
| `cargo audit`(lock 이 움직였다) | 취약점 전 0 → 후 0. 경고(unmaintained) 전 2 → 후 **1** — `chrono 0.4.45` 의 `RUSTSEC-2026-0543` 이 chrono 와 함께 빠졌다. 남은 하나는 `ttf-parser 0.25.1`(`RUSTSEC-2026-0192`)이고 이 조각과 무관하다 |
| `smoke_gate.sh` 를 ktf · lgt · skt 각각 전/후로 | ★**그 스크립트로는 돌리지 않았다.** 대신 §7 의 «전 · 후 · 계측 판 세 프로세스 동시» 실행이 같은 것(부팅 + 그림)과 그 이상(키 27개 · Java 예외 · 깊이)을 SKT 82행 전부 · KTF · LGT 표본 57행 · 가드 7행에서 쟀다. `smoke_gate.sh` 는 한 판씩 차례로 도는 단일 바이너리 실행이라 포화된 호스트에서 전/후를 «그 순간의 비»로 견줄 수 없다. KTF · LGT 의 나머지 283행은 «후» 바이너리로 그 스크립트와 같은 실행(`wie_validate --timeout 15`)을 돌렸다(§7-2) |
| 조각 뒤 예행으로 충돌 수 | §8 |

### 7. 전 · 후 · 계측 판을 같은 순간에 — 깊이 상한 · 판정 · 속도

**방법.** 세 바이너리를 행마다 «같은 순간에» 세 프로세스로 돌렸다(동시 에뮬레이터 3 — 상한 그대로).

- **전** = `origin/main` `3c3986da9` 의 릴리스 `wie_validate`(스크래치 트리 · 자기 `target/`).
- **후** = 이 브랜치의 릴리스 `wie_validate`.
- **계측 판** = 후와 같은 트리에 스크래치 `[patch.crates-io] jvm = { path = … }` 를 건 것. `jvm 0.5.1` 사본의 `execute_method` 에 여덟 줄을 더했다 — 프레임을 민 직후의 깊이가 그 프로세스의 최고값을 넘으면 `WARN WIE_DEPTH <n>`, 512 에 닿으면 `WARN WIE_SOE`. 저장소에는 없다(증적의 `jvm-depth-instrumentation.diff`). 엔진은 후와 같으므로 그 paints 는 «후»의 같은 순간 두 번째 표본이기도 하다.
- 인자는 census 프로브 A 와 같다: `--inject --keep-timeout --timeout 30 --max-ticks 100000000000 --relaunch 1 --pacing 8`(키 27개). 가드 7행은 census 오래 하기의 키 고리를 120초(`--keys`). CPU 초는 `/usr/bin/time -l`.
- phase(≈14분) 하나에 `build-slot run --long` 임대 하나 · 15:40 ~ 17:25 · 임대 8건. 행을 시작할 때의 load1 은 SKT 묶음이 6.9 ~ 56(중앙 12), 뒤 묶음들은 7 ~ 10 이다.
- 잰 행: **가드 7**(KTF 1 · LGT 6 · 2회) · **SKT 82 전부** · **KTF · LGT 표본 57**(가드를 뺀 KTF · LGT 340행을 sha 순으로 늘어놓고 6번째마다 · KTF 41 · LGT 16) — 합 **146행**. 움직인 19행은 한 번 더 쟀다.

#### 7-1. 깊이 상한(512) — 닿은 행 0

| 통신사 | 잰 행 | 최대 깊이 | 중앙 | p95 | 64 이상인 행 | 512 에 닿은 행 |
|---|---:|---:|---:|---:|---:|---:|
| SKT | 82 | 39 | 24 | 30 | 0 | 0 |
| KTF | 42 | 45 | 40 | 44 | 0 | 0 |
| LGT | 22 | 44 | 36 | 40 | 0 | 0 |
| **합계** | **146** | **45** | 29 | 43 | 0 | **0** |

- `StackOverflowError` 가 난 행 **0**(계측 판의 `WIE_SOE` 0건 · 전 · 후의 stderr 에 그 이름 0건). 가장 깊은 값이 45 로 상한의 9% 다.
- KTF · LGT 가 SKT 보다 깊은 것은 게임 Java 가 깊어서가 아니라 «엔진 Rust → 게스트 ARM → 엔진 Rust» 왕복이 한 번에 Java 프레임 하나씩을 쌓기 때문으로 읽는다(이 회차의 읽기다 — 프레임의 내용을 찍어 보지는 않았다).
- ★이 표가 못 보는 것: 키 27개가 30초(가드는 120초) 안에 닿는 장면까지다. 그 뒤 장면에서 더 깊이 재귀하는 판이 있을 수 있다. 종전에는 그런 판이 호스트 스택이 넘칠 때까지 돌았고(릴리스 빌드의 여유는 재지 않았다) 이제는 512 에서 Java 예외로 끝난다.
- ★`wie_validate` 의 `java_exceptions` 는 이 예외를 세지 못한다(§4 의 넷째 덧댐). 그래서 계측으로 쟀다.

#### 7-2. 판정 — 전/후가 달라진 행 0

| 묶음 | 행 | `result` 가 전/후 다른 행 | 후(또는 계측 판)에서 패닉 | 게스트 스레드의 죽음이 전/후 다른 행 | 첫 Java 예외가 다른 행 |
|---|---:|---:|---:|---:|---:|
| 가드(120초 · 2회) | 7 | 0 | 0 | 0 | 0 |
| SKT | 82 | 0 | 0 | 0 | 0 |
| KTF · LGT 표본 | 57 | 0 | 0 | 0 | 0 |
| 움직인 행의 2회째 | 19 | 0 | 0 | 0 | 0 |

- SKT 82행은 전/후 모두 `PASS`. KTF · LGT 표본은 56행이 양쪽 `PASS`, 1행(`689c491586b9`)이 양쪽 `UNMEASURED`(키를 다 받기 전에 끝났다 — 전/후 같다).
- 가드의 `result` 는 14회 모두 양쪽 `UNMEASURED` 다. 판정이 아니라 **이 회차 키 스크립트의 길이** 탓이다: 고리 13바퀴(195걸음)를 넣었는데 120초에 168걸음이 들어간다. 전/후가 같은 수(168 · `4ece6eeeaa04` 는 29)를 받았다. `4ece6eeeaa04` 는 스스로 일찍 끝나는 판이다(`STATE.md` «다음» ⑵).

**KTF · LGT 의 나머지 283행 — «후» 바이너리의 부팅 + 그림만**(`smoke_gate.sh` 가 재는 것과 같은 실행: `wie_validate --timeout 15 <f>` · 세 프로세스 · 임대 3건 · 17:25 ~ 17:46):

| | 행 | `PASS` | `FAIL` |
|---|---:|---:|---:|
| KTF | 227 | 225 | 2 |
| LGT | 56 | 56 | 0 |

`FAIL` 두 행(`60bd6cbc5936` · `dab2d537f3ef`)은 `compat.json` 이 이미 `not-yet`(부팅 `no`)로 적은 그 두 행이고, «전» 바이너리로도 같은 줄(`ZipException: invalid Zip archive`)로 실패한다. 즉 `compat.json` 이 부팅 `ok` 로 적은 427행은 후 바이너리에서 전부 부팅하고 그린다(짝으로 잰 146 + 이 281). 패닉 0.

#### 7-3. 가드 7행 — 전/후 · 두 번

1회(15:40 ~ 15:53 · 다른 레인의 릴리스 빌드 셋과 겹쳐 load1 17 ~ 74):

| 행 | 통신사 | load1 | result 전 / 후 | paints 전 / 후 | CPU 초 전 / 후 | 키 전 / 후 | Java 예외 수 전 / 후 | 최대 깊이(계측 판) | 상한 도달 |
|---|---|---|---|---|---|---|---|---:|---:|
| `13d7e3c21856` | LGT | 40.58→73.73 | UNMEASURED / UNMEASURED | 1244 / 1439 (16%) | 41.5 / 41.8 | 168 / 168 | 0 / 0 | 38 | 0 |
| `1b107b96bf4e` | LGT | 73.73→64.38 | UNMEASURED / UNMEASURED | 1751 / 1744 (-0%) | 64.1 / 64.0 | 168 / 168 | 6 / 6 | 40 | 0 |
| `49ade89578c5` | KTF | 64.38→26.19 | UNMEASURED / UNMEASURED | 3803 / 3395 (-11%) | 91.9 / 91.5 | 168 / 168 | 0 / 0 | 39 | 0 |
| `4ece6eeeaa04` | LGT | 26.19→27.61 | UNMEASURED / UNMEASURED | 32 / 32 (0%) | 15.2 / 15.2 | 29 / 29 | 0 / 0 | 37 | 0 |
| `a30bbe008b5e` | LGT | 27.61→41.15 | UNMEASURED / UNMEASURED | 2161 / 2224 (3%) | 89.6 / 89.9 | 168 / 168 | 2 / 2 | 37 | 0 |
| `b475b6399684` | LGT | 42.50→25.99 | UNMEASURED / UNMEASURED | 2837 / 2993 (5%) | 96.1 / 95.9 | 168 / 168 | 1 / 1 | 33 | 0 |
| `ddd885583b15` | LGT | 25.99→17.37 | UNMEASURED / UNMEASURED | 5368 / 5435 (1%) | 116.8 / 116.6 | 168 / 168 | 0 / 0 | 36 | 0 |

2회(17:03 ~ 17:17 · load1 7 ~ 9):

| 행 | 통신사 | load1 | result 전 / 후 | paints 전 / 후 | CPU 초 전 / 후 | 키 전 / 후 | Java 예외 수 전 / 후 | 최대 깊이(계측 판) | 상한 도달 |
|---|---|---|---|---|---|---|---|---:|---:|
| `13d7e3c21856` | LGT | 7.39→7.62 | UNMEASURED / UNMEASURED | 1776 / 1779 (0%) | 119.2 / 119.2 | 168 / 168 | 0 / 0 | 38 | 0 |
| `1b107b96bf4e` | LGT | 7.62→7.96 | UNMEASURED / UNMEASURED | 1999 / 2205 (10%) | 119.3 / 119.4 | 168 / 168 | 6 / 6 | 40 | 0 |
| `49ade89578c5` | KTF | 7.96→9.25 | UNMEASURED / UNMEASURED | 4761 / 4764 (0%) | 119.3 / 119.3 | 168 / 168 | 0 / 0 | 39 | 0 |
| `4ece6eeeaa04` | LGT | 9.25→8.47 | UNMEASURED / UNMEASURED | 32 / 34 (6%) | 22.3 / 22.3 | 29 / 29 | 0 / 0 | 37 | 0 |
| `a30bbe008b5e` | LGT | 8.47→8.52 | UNMEASURED / UNMEASURED | 2287 / 2288 (0%) | 119.7 / 119.6 | 168 / 168 | 2 / 2 | 37 | 0 |
| `b475b6399684` | LGT | 8.52→8.41 | UNMEASURED / UNMEASURED | 3141 / 3193 (2%) | 119.3 / 119.3 | 168 / 168 | 1 / 1 | 33 | 0 |
| `ddd885583b15` | LGT | 8.41→8.09 | UNMEASURED / UNMEASURED | 5442 / 5481 (1%) | 119.2 / 119.2 | 168 / 168 | 0 / 0 | 36 | 0 |

- 1회에서 ±10% 를 넘은 두 행(`13d7e3c21856` +16% · `49ade89578c5` −11%)은 2회에서 둘 다 0% 다. 1회의 그 두 행은 세 프로세스가 CPU 를 41 ~ 92초(120초 가운데)밖에 못 받은 구간이었다.
- 2회에서는 `1b107b96bf4e` 가 +10%(1,999 → 2,205)이고 1회에는 0% 였다. 두 번 다 같은 쪽으로 움직인 가드 행은 없다.
- 일곱 행 모두 Java 예외 수 · 받은 키 수 · 최대 깊이가 전/후 · 두 회에서 같다.

#### 7-4. 속도 — 일부 판이 빨라진다 (CPU 초는 같다)

paints(30초에 그린 장수)의 후/전 비 · 1회 · paints 20 미만인 행은 뺐다:

| 묶음 | −10% 이하 | −10 ~ −3% | ±3% 안 | +3 ~ +10% | +10% 이상 | (paints < 20) |
|---|---:|---:|---:|---:|---:|---:|
| SKT 82 | 0 | 3 | **66** | 5 | **6** | 2 |
| KTF · LGT 표본 57 | 0 | 2 | **44** | 6 | **5** | 0 |

움직인 행을 한 번 더 쟀다(SKT 는 ±3% 를 넘은 14행 전부 · KTF · LGT 는 ±10% 를 넘은 5행 · 17:15 ~ 17:25 · load1 6 ~ 10):

| 행 | 통신사 | 1회 paints 전 → 후 | 1회 CPU 초 전 / 후 | 1회 load1 | 2회 paints 전 → 후 | 2회 CPU 초 전 / 후 | 2회 load1 | 두 회 모두 |
|---|---|---|---|---:|---|---|---:|---|
| `4288d8c1c6ac` | KTF | 223 → 396 (+78% · 계측 판 +78%) | 29.8 / 29.8 | 8.48 | 221 → 391 (+77% · 계측 판 +77%) | 29.6 / 29.6 | 8.00 | **+10% 이상** |
| `9babd9789bae` | KTF | 11974 → 22876 (+91% · 계측 판 +91%) | 29.8 / 29.8 | 7.39 | 12002 → 22965 (+91% · 계측 판 +91%) | 29.9 / 29.8 | 6.98 | **+10% 이상** |
| `d1e0badfce82` | KTF | 68 → 112 (+65% · 계측 판 +35%) | 29.9 / 29.9 | 8.37 | 75 → 96 (+28% · 계측 판 +49%) | 29.9 / 29.9 | 7.32 | **+10% 이상** |
| `1a69522a7d43` | LGT | 1087 → 1445 (+33% · 계측 판 +33%) | 29.8 / 29.8 | 8.47 | 1076 → 1431 (+33% · 계측 판 +33%) | 29.9 / 29.9 | 7.85 | **+10% 이상** |
| `2a57e33133b5` | LGT | 1488 → 1773 (+19% · 계측 판 +14%) | 29.9 / 29.8 | 7.33 | 1464 → 1774 (+21% · 계측 판 +16%) | 29.9 / 29.9 | 8.81 | **+10% 이상** |
| `1367261bc3ee` | SKT | 294 → 280 (-5% · 계측 판 -3%) | 13.3 / 13.5 | 25.14 | 302 → 304 (+1% · 계측 판 +0%) | 30.1 / 30.4 | 7.68 | 갈린다 |
| `2b6d78567795` | SKT | 2247 → 2987 (+33% · 계측 판 +34%) | 13.4 / 13.5 | 27.95 | 5616 → 7551 (+34% · 계측 판 +35%) | 29.9 / 29.8 | 8.52 | **+10% 이상** |
| `36c859b7df28` | SKT | 64 → 61 (-5% · 계측 판 -2%) | 9.6 / 9.9 | 41.25 | 57 → 57 (+0% · 계측 판 +2%) | 30.1 / 30.5 | 8.29 | 갈린다 |
| `38277d63b0ba` | SKT | 4924 → 5808 (+18% · 계측 판 +18%) | 10.1 / 10.3 | 40.46 | 14604 → 16760 (+15% · 계측 판 +16%) | 29.6 / 29.5 | 7.94 | **+10% 이상** |
| `66959afab216` | SKT | 1509 → 1812 (+20% · 계측 판 +20%) | 29.8 / 29.8 | 9.36 | 1514 → 1820 (+20% · 계측 판 +20%) | 29.8 / 29.8 | 8.74 | **+10% 이상** |
| `71d1d8235bd1` | SKT | 5723 → 6414 (+12% · 계측 판 +12%) | 29.5 / 29.4 | 9.45 | 5653 → 6370 (+13% · 계측 판 +13%) | 29.7 / 29.7 | 8.64 | **+10% 이상** |
| `78bd51675574` | SKT | 18031 → 23249 (+29% · 계측 판 +29%) | 28.2 / 28.1 | 10.84 | 20197 → 24957 (+24% · 계측 판 +24%) | 29.9 / 29.9 | 8.24 | **+10% 이상** |
| `916aea39fa36` | SKT | 27 → 26 (-4% · 계측 판 +0%) | 26.6 / 26.7 | 42.45 | 27 → 26 (-4% · 계측 판 -4%) | 29.9 / 29.9 | 7.33 | −3% 이하 |
| `a0436eb1ddbb` | SKT | 857 → 886 (+3% · 계측 판 +4%) | 29.9 / 29.9 | 16.07 | 865 → 892 (+3% · 계측 판 +3%) | 29.9 / 29.9 | 6.48 | +3% 이상 |
| `b42e4242866b` | SKT | 23892 → 25785 (+8% · 계측 판 +8%) | 29.8 / 29.9 | 10.14 | 24295 → 25958 (+7% · 계측 판 +7%) | 29.9 / 29.9 | 7.27 | +3% 이상 |
| `eefc947d8337` | SKT | 60 → 62 (+3% · 계측 판 +3%) | 29.5 / 29.5 | 22.03 | 59 → 62 (+5% · 계측 판 +5%) | 29.9 / 29.9 | 6.76 | +3% 이상 |
| `f2ae515201f2` | SKT | 1787 → 1900 (+6% · 계측 판 +6%) | 29.9 / 29.9 | 13.84 | 1821 → 1940 (+7% · 계측 판 +7%) | 29.9 / 29.8 | 6.63 | +3% 이상 |
| `f81658d4ad1a` | SKT | 14724 → 21293 (+45% · 계측 판 +45%) | 29.9 / 29.9 | 7.93 | 13572 → 19567 (+44% · 계측 판 +43%) | 29.2 / 29.3 | 7.10 | **+10% 이상** |
| `fb7a86ce425b` | SKT | 358 → 372 (+4% · 계측 판 +4%) | 29.7 / 29.7 | 7.16 | 357 → 372 (+4% · 계측 판 +4%) | 28.8 / 28.7 | 9.95 | +3% 이상 |

- **두 번 모두 +10% 이상인 행이 11** 이다 — SKT 6(`f81658d4ad1a` +45 · +44% · `2b6d78567795` +33 · +34% · `78bd51675574` +29 · +24% · `66959afab216` +20 · +20% · `38277d63b0ba` +18 · +15% · `71d1d8235bd1` +12 · +13%) · KTF 3(`9babd9789bae` +91 · +91% · `4288d8c1c6ac` +78 · +77% · `d1e0badfce82` +65 · +28%) · LGT 2(`1a69522a7d43` +33 · +33% · `2a57e33133b5` +19 · +21%). 계측 판(후와 같은 엔진)의 비도 같은 값이다 — 한 순간에 «후» 두 표본이 같이 움직였다.
- 두 번 모두 +3 ~ +8% 인 SKT 행이 5 다. 두 번 모두 내린 행은 `916aea39fa36` 하나이고 27장 → 26장이다(한 장).
- 1회에 −5% 였던 SKT 두 행(`1367261bc3ee` · `36c859b7df28`)은 2회에 +1% · 0% 다. KTF · LGT 표본의 −8% · −5% 두 행은 같은 순간의 계측 판이 0% · +3% 였다(다시 재지는 않았다).
- **CPU 초는 전/후가 같다**(위 표 — 행마다 전 · 후가 같은 값이다. 대부분 30초 가운데 29.x초이고, 1회의 SKT 두 행은 호스트가 바빠 10 ~ 13초였다 — 그 두 행도 2회에서는 29.x초이고 비가 같다). 줄어든 것은 CPU 가 아니라 한 바퀴의 일이다: 0.5.0 의 조회 캐시(§4)가 필드 · 메서드 해소를 처음 한 번으로 줄였다. KTF · LGT 에서는 엔진 Rust 가 게스트 클래스의 필드 · 메서드를 이름으로 찾을 때마다 게스트 메모리의 표를 문자열로 훑던 것이 없어진다 — 그래서 SKT 만 빨라진 S1(0572 §10-9)과 달리 이번에는 KTF · LGT 도 움직인다.
- 빨라지는 것은 «한 코어를 다 쓰고도 제 속도에 못 미치던 판»과 «쉬지 않고 그리는 판»이다. 게스트가 청한 sleep 횟수(게임 고리)로 보면 `4288d8c1c6ac` 는 70 → 144 로 고리 자체가 두 배가 됐고, `9babd9789bae` 는 314 → 314 로 고리는 그대로인데 그리기만 는다.
- ★**이용자에게 보이는 변화다.** `docs/player-updates/2026-11-23-some-games-run-faster-after-java-engine-update.json`(`kind: speed`)을 더했다. `compat.json` 의 행은 고치지 않았다(판정이 바뀐 행이 없다).
- ★못 잰 것: KTF · LGT 의 나머지 283행에서 몇 행이 빨라지는지(표본의 비율 5/57 을 그대로 늘리면 25행 안팎이지만 잰 수가 아니다) · 브라우저에서의 값 · «빨라진 것이 그 판의 제 속도에 가까워진 것인지 넘어선 것인지»(실기 기준이 없다).

### 8. upstream 병합 예행 — 충돌 수는 이번에도 줄지 않았다

`git merge-tree --write-tree --name-only <ours> upstream/main`. `upstream/main` = `ea5640ec1`(behind **248** · merge-base `44fbf265` 그대로).

| 잰 것 | 전(`origin/main` `3c3986da9`) | 후(이 브랜치의 코드 머리 `cbb7bcb5d`) |
|---|---:|---:|
| 충돌 파일 | 146 | **147** |
| 충돌 덩어리(`<<<<<<<`) | 666 | **663** |

- 늘어난 한 파일은 `wie-ktf/src/runtime/java/jvm_support/class_instance.rs` 다. 종전에는 upstream 만 그 import 줄을 고쳐 조용히 들어왔다. 이제 우리도 같은 줄을 (같은 내용으로) 고쳤고, upstream 은 바로 아랫줄(`wipi_types` import)도 고쳤으므로 git 은 겹친 수정으로 적는다. import 순서를 upstream 과 글자까지 같게 맞춰도 남는다 — 0572 §7 이 적은 것과 같은 현상이다.
- 이 조각이 고친 18파일 가운데 전에도 충돌하던 것은 10개이고 그 10개는 후에도 충돌한다(`Cargo.toml` · `Cargo.lock` 포함).
- 이 조각이 치운 것은 충돌 «수»가 아니라 병합 뒤에 질 일이다: 병합 커밋이 RustJava 판을 0.4.1 → 0.5.1 로 옮기면서 깨지는 호출부 15곳과, 그 판의 동작 변화(§4)를 병합의 다른 변화와 섞어서 재야 하는 부담이 없어졌다. `Cargo.toml` 의 RustJava 다섯 줄은 병합 때 «우리 `=0.5.1` ↔ upstream `^0.5.1`» 로 한 번 더 충돌한다 — 우리 것을 고른다.

### 9. 남긴 것 · 못 잰 것

- **병합 커밋(S3)은 하지 않았다**(티켓 경계). `wie-backend` 의 executor · pacing · `compat.json` 의 행도 건드리지 않았다. GC 루트 설계(우리 `guest_roots` ↔ upstream `ExternalRoots`)는 그대로이고 `Jvm::new` 의 `external_roots` 는 여전히 `None` 이다.
- **`InheritedMethods` · `hardening.rs` 는 지우지 않았다**(§4 의 처분 표).
- **깊이 상한은 «== 512» 판정이라 네이티브 프레임을 사이에 두면 건너뛸 수 있다**(§4). upstream 의 코드이고 우리 쪽에서 고칠 자리가 아니다 — 발신 후보로만 적는다(S5).
- 제안 카드는 0건이다. 이 회차가 본 것 가운데 지금 바로 칠 수 있고 이미 열린 것과 겹치지 않는 «게임이 달라지는» 일이 없다. `java_exceptions` 집계가 `StackOverflowError` 를 못 보는 것은 도구(meta) 쪽이라 카드로 올리지 않았다.

| 못 잰 것 | 왜 · 무엇이 대신하나 |
|---|---|
| KTF · LGT 나머지 283행의 전/후 짝 · 깊이 | 세 프로세스 동시 실행은 한 행에 30초라 340행이면 약 3시간이다. 표본 57행 + 가드 7행을 쟀고, 나머지는 «후» 바이너리의 부팅 + 그림만 쟀다(§7-2) |
| 오래 하기(600초) · 소리 · 입력 축(census 의 판정) | 키 27개 · 30초 한 번(가드는 120초)이다. census 전수 짝은 15초 프로브로도 약 2.4시간이었다(0572 §10-3). 일요일 05:00 의 주간 퇴행 감시가 착지 뒤 첫 전수다 |
| 브라우저에서의 속도 · 동작 | wasm 은 빌드했고 표면 계약은 통과했다(§6). 실제 브라우저 부팅은 PR 의 `contract` 잡(`contract-roundtrip.mjs`)이 잰다. `audio-probe.mjs` 의 paints 는 재지 않았다 |
| 게임이 `StringBuffer` · `Vector` 를 상속해 `append(String)` · `ensureCapacity` 를 덮어쓴 판의 수 | 0.5.1 이 그 내부 호출을 직접 쓰기로 바꿨다(§4). 코퍼스에서 세지 않았다 — SKT 82행의 첫 Java 예외 · 판정은 전/후 같다 |
| J2ME 코퍼스 | `compat.json` 에 J2ME 행이 없다. 러너 블록의 `draw_j2me.jar` · `text_j2me.jar` 가 유일한 신호다 |
| 릴리스 빌드에서 종전에 호스트 스택이 넘치던 Java 깊이 | 재지 않았다. 상한 512 가 «종전에 돌던 깊이»를 자르는지는 깊이가 45 를 넘는 판이 나와야 물을 수 있다 |

### 10. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치가 main 대비 더하거나 고친 22파일): **유입 14건(BOUNDED)** + **판단 필요 1건(SUFFIX-ATTACHED)**. 열다섯 건 모두 `wie-lgt` 의 `.rs` 다섯 파일(`interface.rs` · `jvm_support/class_definition.rs` · `method.rs` · `value.rs` · `stdlib.rs`)에 main 부터 있던 주석이고, 이 조각은 그 파일들의 import 줄 · `localtime` · 시험만 고쳤다 — 이 조각이 더한 `.rs` · `.toml` 69줄에 한글은 0줄이다. 이 회차가 쓴 문서(이 기록 · worklog · 고지 · `STATE.md`)와 `Cargo.toml` 만 넣어 돌리면 BOUNDED 0 · SUFFIX-ATTACHED 0 이다.

<!-- corpus-name-inflow v1 subjects=22 tree=80a9080137a685ab B=18/14 P=6/1 S=1/1 -->
