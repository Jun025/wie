## [2026-11-22] 조각 S1 — RustJava 핀 0.1.1 → 0.4.1 (upstream 병합 전 · 우리 쪽만) · `patches/classfile` 삭제 · zip 9 · 감사 억제 8건 해제 (wie-upstream-sync2-s1-rustjava-pin-0-4-1-before-upstream-sync)

**무엇을**: `docs/report/0551` §7 의 S1 이다. 루트 `Cargo.toml` 의 `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` 을 `^0.1.1` 에서 `=0.4.1` 로 올렸고(처음에는 `^0.4.1` + lock 0.4.1 이었고 §10 에서 매니페스트를 `=0.4.1` 로 고정했다), 그 판이 바꾼 API 를 우리 코드 전부에서 따라갔다. `git merge upstream/main` 은 하지 않았다.

| 축 | 전 | 후 |
|---|---|---|
| `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` | 0.1.1 | **0.4.1** |
| `classfile` | 0.1.1 + 우리 패치(`patches/classfile` · `[patch.crates-io]`) | crates.io **0.4.1** 그대로 — 패치 디렉터리와 `[patch]` 줄을 지웠다 |
| `zip` | 8.6.0(`wie-backend` 직접 + `rustjava-runtime` 간접) | **9.8.1** 하나(`wie-backend` `^8.6` → `^9.0`) |
| 감사 워크플로의 `--ignore` | 8개(zip · 대장 §D 3행) | **0개**(§D → C-16) |
| lock 에 새로 든 크레이트 | — | `constant_time_eq 0.6.1` · `typed-path 0.13.0` · `zeroize_derive 1.5.0`(zip 9 의 의존) |

**왜**: 티켓 `wie-upstream-sync2-s1-rustjava-pin-0-4-1-before-upstream-sync`(운영자 채택 제안 `2026-11-06-upstream-sync2-dry-run-classify#p0`). upstream 병합에서 Java 표면 파일이 손 수정이 되는 가장 큰 원인이 RustJava 0.3.0 의 «필드 접근에 owner 클래스 인자»였다. 그 몫을 병합 전에 우리 쪽에서 치운다.

**사용자 영향**: 지원 목록의 판정이 바뀐 판은 표본 68판에서 **0**이고(§6), 전수 429행의 전/후 짝(부팅 · 그림 · 입력 · 소리 · 15초 프로브)에서도 다시 재어 남는 변화가 **0**이다(§10-3). 엔진 쪽에서 바뀌는 것은 셋이다. ⑴ 엔진의 Java 클래스가 이제 «자기 필드»를 읽는다 — 게임 클래스가 같은 이름의 필드를 선언해 엔진 값을 가리던 종류의 오동작(0382)이 9곳이 아니라 전부에서 막힌다. ⑵ Java 로 도는 판은 필드 · 메서드 조회가 싸졌다(0.1.1 은 조회마다 후보의 이름 · 서술자를 `String` 으로 새로 만들었고 0.4.1 은 빌려 읽는다). CPU 초는 전/후가 같고, 같은 CPU 몫으로 하는 일이 는다: 정해진 시간만 자고 나머지를 일로 채우는 판(SKT 표본 하나)은 한 바퀴가 짧아져 CPU 가 넉넉해도 화면을 더 그리고(최종 트리 · CPU 한 코어를 다 쓸 때 +52 ~ 53%), 프레임 마감에 맞춰 자는 판(J2ME 표본)은 CPU 가 넉넉하면 전/후가 같고(30.3 / 30.3) 모자라면 더 그린다(§6 속도 · §10-2 · 최종 트리의 수는 §10-7). 처음 판의 이 줄 «CPU 가 3분의 1 로 줄었다» 는 틀렸다 — 준 것은 CPU 가 아니라 지표의 분모(tick)였다. ⑶ zip 9 로 가면서 감사 억제 8건이 0건이 됐다. 이용자에게 보이는 동작 변화로 확인된 것이 없어 `docs/player-updates/` 항목은 더하지 않았다. `compat.json` 은 바뀌지 않는다. `Cargo.lock` 과 엔진 `*.rs` 가 바뀌므로 착지하면 otterpebble 핀 범프가 뜬다.

### 1. 핀이 바꾼 API 와 우리가 따라간 곳

| RustJava 의 변화 | 우리 쪽 |
|---|---|
| 0.2.0 — `ClassDefinition::name` · `Field::name/descriptor` · `Method::name/descriptor` · `ArrayClassDefinition::element_type_name` 이 `String` 대신 `Cow<'_, str>` | KTF · LGT · `wie-jvm-support` 의 구현 15곳과 그 값을 키로 쓰던 호출부 |
| 0.3.0 — `Jvm::get_field` · `put_field` 가 필드를 선언한 클래스 이름을 받는다 | owner 없는 호출 **1,774곳** + 종전 도우미 9곳(아래 §2) |
| 0.4.0 — `Jvm::new` 가 `external_roots` 를 받는다 | `None`. GC 루트는 우리 `guest_roots` 그대로다(설계 고르기는 S3) |
| 0.4.0 — `push_native_frame(&[..])` 가 가드(`NativeFrame`)를 돌려주고 공개 `pop_frame` 이 없다 | 짝이던 push/pop 20쌍을 가드 + `drop` 으로. 조건부 프레임(KTF `init.rs` · `relocated.rs`)은 `Option<NativeFrame>` |
| 0.4.0 — `collect_garbage` 가 `anyhow::Result` | `Display` 페인트 뒤 수거는 upstream 과 같은 `net/wie/WieError`, KTF · LGT 문맥은 `WieError::FatalError` |

upstream 이 같은 일을 한 커밋(`5fb65ee82` #1430 · `228e3ff48` #1444 · `7951304c9` #1489 · `28c182434` · `b149acdc2` · `0c338e2b7` · `9fd11d483` #1542 · `a6dca709d`)을 본으로 썼다. #1542 의 `ExternalRoots` 구현(KTF · LGT)과 #1489 의 매니페스트 읽기 변경은 가져오지 않았다 — 앞은 GC 루트 설계를 바꾸는 일이고 뒤는 우리 코드가 그 API 를 쓰지 않는다.

런타임 클래스가 선언한 필드 · 메서드를 0.1.1 과 0.4.1 에서 전수로 뽑아 대조했다: 필드는 `HashMap$HashIterator` 의 2개가 늘었고, 메서드는 `java.util.jar.Attributes` 계열 17개와 HashMap 반복자 2개가 늘고 1개가 빠졌다. `Object` · `Thread` · `Runtime` · `TimerTask` 는 선언이 같다 — `wie-lgt/data/lgt_java_abi.toml` 의 유도 주석(「rustjava-runtime 0.1.1 은 …」)이 기대는 사실은 0.4.1 에서도 성립한다.

### 2. 필드 owner 인자 — 1,783곳을 어떻게 정했나

`origin/main` 의 owner 없는 호출 1,774곳과 owner 를 이미 받던 도우미 호출 9곳이다. 그중 LGT 시험의 7곳은 나중에 필드 객체로 직접 읽게 바꿨으므로(§5) 지금 트리의 `Jvm::get_field`/`put_field` 호출은 1,776곳이다.

| 정한 법 | 곳 |
|---|---:|
| 같은 파일의 `upstream/main` 에 같은 (인스턴스 · 이름 · 서술자) 호출이 있고 owner 가 하나 → 그 값(우리 클래스 표에서도 풀리는지 확인) | 1,196 |
| 인스턴스가 `this` → 그 메서드를 가진 클래스 | 394 |
| (이름 · 서술자)를 선언한 클래스가 트리에 하나뿐 → 그 클래스 | 68 |
| 둘러싼 클래스에서 풀리는 선언 클래스 — 인스턴스 형을 읽어 확인했다(시험 코드 4곳이 틀려 고쳤다) | 30 |
| 손으로(시험의 지역 클래스 · 이름이 변수인 도우미 · 다른 클래스의 인스턴스) | 86 |
| 종전 `get_declared_field`/`put_declared_field`(이미 owner 를 받던 도우미) → 새 API | 9 |

- 변환은 스크래치 스크립트로 했고 저장소에 두지 않았다. 변환 뒤 모든 호출의 owner 를 우리 proto 표 + `rustjava-runtime 0.4.1` 의 proto 표에서 부모 사슬로 풀어 보았다: 리터럴 owner 1,707곳 중 못 푼 것 10곳은 전부 필드 이름이나 서술자가 상수인 선언(`List` · `MIDlet` · `DialogComponent`)이라 표가 못 읽은 것이고, 시험이 그 경로를 탄다.
- `static` 으로 선언된 필드를 인스턴스 접근으로 읽는 곳은 **0**이다(새 판은 그 경우 `IncompatibleClassChangeError` 를 던진다).
- ★**의미가 바뀐다(둘).** ⑴ 종전 `get_field` 는 «인스턴스의 실행 중 클래스»에서 이름을 찾았다. 그래서 게임 클래스가 같은 이름 · 같은 형의 필드를 선언하면 엔진 코드가 게임의 필드를 읽었다(`docs/report/0374` · `0382` — `Card` 의 x · w, `MIDlet` · `Jlet` 의 display). 그때 고친 자리는 `wie-jvm-support/src/declared_field.rs` 의 도우미를 쓰는 9곳뿐이었다. 이제는 모든 호출이 owner 로 적은 클래스의 필드를 읽는다 — 엔진 코드에서는 «엔진 클래스 자신의 필드»다. 그 도우미는 새 API 와 같은 일을 하므로 지웠고, 가림 시험 3개(`card_bounds_are_cards_own_fields_not_a_subclasss` · `display_is_the_midlets_own_field_not_a_subclasss` · `display_is_the_jlets_own_field_not_a_subclasss`)는 owner 를 하위 클래스로 적어 그대로 남겼다.
  ⑵ `jvm-bytecode` 0.4.1 은 인스턴스 필드의 저장 키에 선언 클래스를 넣었다(`FieldInner.declaring_class` — `jvm-bytecode-0.4.1/src/field.rs`). 0382 가 «upstream 크레이트라 못 고친다»고 남긴 SKT 의 private 필드 «공유»(엔진 `MIDlet.display` 와 게임의 private `display` 가 같은 칸이던 것)가 이 핀으로 풀린다 — 이제 두 필드는 다른 칸이다. KTF · LGT 는 ARM 오프셋 저장이라 해당 없다. 0382 뒤에 생긴 «가릴 수 있는» 엔진 필드의 코퍼스 계수는 §10 에 있다.

### 3. `patches/classfile` 삭제 — 지우기 전후의 문자열 해석

패치의 제거 조건은 «배포된 classfile 이 `constant_pool.rs` 의 시험 4종을 통과할 때»였다. 0.4.1 의 `parse_utf8` 은 `jvm_types::modified_utf8::decode` → `String::from_utf16_lossy` 다. 시험 4종 가운데 셋(`C0 80` = NUL · 서로게이트 쌍 · ASCII/한글)은 그 규칙으로 성립한다. 넷째(`edges_of_the_wider_decoder`)의 한 줄 — 표준 4바이트 UTF-8(`F0 …`)을 받아들인다 — 은 0.4.1 에서 **성립하지 않는다**(1 · 2 · 3바이트 규칙만 받는다). javac 는 클래스 파일에 4바이트 꼴을 쓰지 않으므로 표본으로 갈랐다.

스크래치 대조기(저장소 밖 — 패치판 0.1.1 과 crates.io 0.4.1 을 한 바이너리에 같이 묶었다)로 코퍼스 꾸러미 527개와 그 안의 jar 에 든 `.class` 전부를 두 판으로 읽었다.

| 잰 것 | 값 |
|---|---:|
| 클래스 | 14,730 |
| Utf8 상수(두 판이 다 읽은 클래스의 것) | 1,027,138 · 그중 비 ASCII 22,298 |
| 상수 문자열이 하나라도 다른 클래스 | **0** |
| 패치판만 읽는 클래스 / 0.4.1 만 읽는 클래스 | **0** / **0** |
| 두 판이 똑같이 못 읽는 `.class` | 391 |

★**이 표가 센 범위**(게이트② F6 — 처음 판은 적지 않았다): `game_lab/` 의 `working` · `broken` · `_dup` · `_nongame` · `vendor_sdk` 전부이고, 같은 클래스가 여러 꾸러미에 들어 있으면 그때마다 셌다. 14,730 가운데 **12,950 이 `vendor_sdk`(SDK jar 47개)** 다 — 게임 꾸러미의 클래스가 아니다. §10-4 에서 대조기를 다시 써서 증적에 넣고(처음 판의 스크래치는 지워졌다) 디렉터리별로 다시 셌다: 게임 꾸러미(`working` + `broken` · 464 경로 · 안의 jar 포함 아카이브 936)의 `.class` 는 **1,778**, 두 판이 다르게 읽는 상수 0 이다.

4바이트 꼴을 가진 클래스는 코퍼스에 없다. 그 차이는 `wie-jvm-support/tests/modified_utf8.rs` 에 단언으로 남겼다 — 패치의 시험 4종을 옮긴 것이고, 넷째의 그 한 줄만 «거부한다»로 뒤집혀 있다.

### 4. zip 8.6.0 → 9.8.1 과 감사 억제

`rustjava-runtime 0.4.1` 은 `zip ^9.0.5` 를 요구한다. `wie-backend` 의 직접 의존(`^8.6`)을 그대로 두면 lock 에 zip 이 둘 남고 자문 8건도 그대로라서 같이 올렸다. `extract_zip` 의 엔트리 이름 읽기만 고쳤다(zip 9 의 `name()` 은 `Result`).

- ★zip 9 는 엔트리 이름을 다르게 읽는다. zip 8 은 «UTF-8 플래그가 서 있으면 UTF-8(손실 허용), 아니면 CP437», zip 9 는 «바이트가 UTF-8 로 유효하면 UTF-8, 아니면 CP437» 이다. 그래서 같은 스크래치 대조기로 **zip 8.6.0 과 9.8.1 을 실제로 나란히** 돌렸다: 코퍼스 꾸러미 527개와 그 안의 jar · zip 484개, 합 **1,011개**에서 파일 엔트리의 (이름 · 바이트) 목록이 다른 것 **0** · 한쪽만 여는 것 **0**.
- 두 규칙이 다르게 읽는 이름은 코퍼스 전체에 하나다 — LGT `b44b5fbe29e6` 의 jar 안, 플래그 없이 UTF-8 로 적힌 **디렉터리** 엔트리. `extract_zip` 은 디렉터리 엔트리를 올리지 않으므로 게임이 보는 파일 목록은 같다. 그 판은 §6 표본에 넣었다.
- 이용자 업로드는 코퍼스 밖이다. 플래그 없이 UTF-8 이름을 가진 **파일** 엔트리가 든 꾸러미는 이제 이름이 깨지지 않고 읽힌다 — 종전에 깨진 이름으로 우연히 맞던 조회가 있었다면 달라진다(코퍼스에는 그런 파일이 없다).
- 무-플래그 `cargo audit`(cargo-audit 0.22.2 · advisory-db 1,555 · lock 559 패키지): 취약점 **0** · 경고 2(`chrono` · `ttf-parser` — 대장 §A 그대로). 두 워크플로의 `--ignore` 8개를 지웠고 대장 §D 의 세 행을 **C-16** 으로 옮겼다. `node scripts/check-audit-warnings.mjs` rc=0 · `node scripts/check-ledger-c-validity-scope.mjs` rc=0. 억제를 지우기 전에는 같은 검사기가 «억제했는데 더는 내지 않는 자문 8건» 으로 red 였다.

### 5. 하드닝 · 조정 표가 새 판에서도 붙는가

- **하드닝**(`wie-jvm-support/src/hardening.rs` · `AGENTS.md` 사건대장 «핀을 옮기면 이 모듈의 시험을 다시 돌려라»): 시험 8개 전부 통과. `every_guard_is_actually_applied` 가 가드가 `find_rustjar_class` 를 지나는 메서드에 실제로 붙는지를 단언한다. `InputStreamReader.read` 대체 본문의 필드 접근 12곳은 owner 를 `java/io/InputStreamReader` 로 적었고 `reader_read_tests` 4개가 그 길을 탄다.
- **장부 가드**(`the_ledger_lists_exactly_*`): `class-patches` · `binary-patches` · `fake-servers` · `ktf-local-servers` · `owner_files` 3표 · `orphan-entries` · `display-overrides` — 전부 통과. 엔진 표와 장부의 키 집합은 그대로다(조정 행 추가 · 삭제 0).
- **`class-patches` 5행(SKT · 클래스 파일 md5 키)**: `class_patches` 시험 3개 통과. 표본 실행은 §6 의 표에 있다 — 다섯 판 모두 새 판에서도 «Purchase-lock patch matched» 가 한 번씩 찍힌다.
- **필드 가림 시험**: `card_bounds_are_cards_own_fields_not_a_subclasss` · `card_display_is_cards_own_field_not_a_subclasss` · `display_is_the_midlets_own_field_not_a_subclasss` · `display_is_the_jlets_own_field_not_a_subclasss` 통과.
- **`this` 접근 1,240곳의 owner 를 둘러싼 `impl` 블록과 대조**했다(스크래치): `impl` 안에 proto 가 하나 있는 블록에서 owner 가 그 클래스나 그 조상이 아닌 곳은 **0**이다. 블록 밖(자유 함수 · 시험 지역 클래스)의 34곳은 눈으로 읽었고 그중 2곳(`event_queue.rs` 시험의 `RecurringCallback`)이 틀려 있었다 — 첫 `cargo test --all` 이 `NoClassDefFoundError` 로 잡은 것과 같은 자리다.
- **등록하지 않은 클래스**: 새 `get_field`/`put_field` 는 owner 이름으로 클래스를 찾으므로 JVM 에 등록하지 않은 정의의 인스턴스에는 쓸 수 없다. LGT 시험 2개(`test_native_jvm_runtime` · `wide_instance_field_is_stored_high_word_first`)가 그 꼴이라 필드 객체로 직접 읽고 쓰게 고쳤다(`ClassInstance::get_field`/`put_field` — 종전 `Jvm` 호출이 안에서 하던 일). 제품 코드에 그런 호출은 없다.

### 6. 회귀

**게이트**(최종 트리): `4fbd7657` 에서 한 임대로 이어 돌렸다. `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — **52 스위트 · 851 passed · 0 failed**(첫 실행은 848 passed · 3 failed 였고 셋 다 시험 쪽 owner · 등록 문제였다 — §5). `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · 같은 것에 `-p wie-core-arm -p wie_featurephone` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0. 그 뒤 더한 것은 시험 파일 하나(`wie-jvm-support/tests/modified_utf8.rs`)와 문서뿐이고, 그 시험은 `cargo test -p wie-jvm-support` 로 따로 돌렸다(rc=0 · lib 29 passed · `class_constants_decode_as_modified_utf8` ok) — 즉 최종 트리의 `cargo test --all` 은 851 + 이 1개다. `cargo fmt --all -- --check` 는 그 뒤 다시 rc=0.

**러너 블록**(`AGENTS.md` · 커밋된 픽스처): `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS rc=0 · `keydraw_ktf.zip` `--inject --expect-last-frame` PASS rc=0(`input_steps` 27 · `paints` 79 · `content` true) · `keydraw_lgt.zip` PASS rc=0(27 · 55 · true) · `text_j2me.jar --timeout 5` PASS rc=0. load1 121 → 80.

**코퍼스 표본 — 전/후 짝**. 전 = `origin/main` `1853f3b6f` 의 릴리스 `wie_validate`, 후 = 이 브랜치의 릴리스 `wie_validate`. `scripts/smoke_gate.sh`(부팅 + 그림 · `--timeout 15` · 실패는 2회 재시도)를 두 바이너리로 같은 임대 안에서 나란히 돌렸다(`build-slot run --long` 1건 · 12:47 ~ 13:08 · load1 177 → 175 · 동시 실행 2).

| 표본 | 판 | 전 PASS | 후 PASS | 판정이 바뀐 판 |
|---|---:|---:|---:|---:|
| 조정 행이 걸린 판 전부(장부 `runtime-adjustments.tsv` 의 꾸러미 49개 · KTF 30 · LGT 7 · SKT 12) | 49 | 49 | 49 | **0** |
| 통신사별 추가(`compat.json` 의 playable 행을 sha 순으로 6개씩 · KTF 6 · LGT 6 · SKT 6) | 18 | 18 | 18 | **0** |
| zip 9 가 jar 엔트리 이름 하나를 다르게 읽는 LGT `b44b5fbe29e6` | 1 | 1 | 1 | **0** |
| **합계** | **68** | **68** | **68** | **0** |

- `smoke_gate.sh` 자신의 종료 코드는 두 쪽 다 2(`UNMEASURED: 252 of 292 baseline titles absent`)다 — 표본이라 baseline 대조가 성립하지 않는 것이고, 위 표는 그 스크립트가 판마다 찍는 PASS/FAIL 줄을 sha 로 짝지은 것이다.
- **`class-patches` 5행**(`0ed66634d3dc` · `44b6356d13f8` · `a42f77f44955` · `c33090c12755` · `eefc947d8337`): `RUST_LOG=info wie_validate --timeout 20` 의 «Purchase-lock patch matched» 줄이 전 1 · 후 1 로 다섯 판 모두 같고 결과도 다섯 판 모두 PASS 다. 새 판에서도 같은 클래스(md5 키)에 걸린다.
- ★**이 표가 못 보는 것**: 판정은 부팅 + 그림뿐이다. 입력 · 오래 하기 · 소리 축(census 의 probe · long)은 재지 않았다 — 이 시각 census 의 호스트 잠금(`/tmp/wie-playability-census.lock`)을 다른 레인의 전수 실행이 쥐고 있었고, 임대 예산 안에서 68판의 probe 를 끼워 넣을 수 없었다. 전수(429행)도 아니다.

**속도**: - `wie-j2me/tests/test_pacing.rs`(1ms 걸음 시계 — 부하와 무관한 수치): 2개 통과. `wie-backend` 의 `pacing` 시험 3개 통과. 주기 · 페인트 수 단언이 그대로 성립한다.
- `WIE_PERF_AXES=native bash scripts/perf-watch.sh compare <전 바이너리> <후 바이너리>`(고정 표본 7 · 3회 · 50M tick 당 CPU ms · 회마다 `--long` 임대 1건 · load1 80 ~ 171): 도구의 판정은 퇴행 0 · 1차 경보 0 · 미측정 0/7 · 편차에 묻힘 0 이다. ★**이 표는 KTF · LGT 다섯 표본에서만 «퇴행 0» 으로 읽을 수 있다.** J2ME · SKT 표본(`835acd535a5f` · `d1234e0ddb75`)의 «tick 당 CPU» 는 이 핀을 사이에 두고 같은 단위가 아니다(아래 · §10-2).

| 표본 | 회별 «후가 전보다 느린 %» | 중앙값 | 판정 |
|---|---|---:|---|
| `0606d43702e7` | −1.3 · −1.8 · 0.6 | −1.3 | ok |
| `1065985081fc` | −0.5 · −0.2 · −1.1 | −0.5 | ok |
| `155972cac664` | 9.3 · −1.8 · −0.8 | −0.8 | ok |
| `3cc7a9b4cb15` | −0.8 · −12 · 5.5 | −0.8 | ok |
| `8238f5449347` | −0.1 · −0.1 · −0.3 | −0.1 | ok |
| `835acd535a5f` | −66.7 · −67.3 · −71.2 | **−67.3** | ok |
| `d1234e0ddb75` | −27.2 · −49 · −2.5 | −27.2 | ok(편차 19) |

★**`835acd535a5f` 의 −67% 는 CPU 가 준 것이 아니다**(게이트② F2 · 처음 판의 이 문단은 «CPU 가 3분의 1 로 줄었다» 고 적었다 — 틀렸다). `wie_validate` 의 루프는 쉬지 않으므로(게스트가 자는 동안에도 빈 tick 을 돈다) 프로세스의 CPU 초는 스케줄러가 준 몫이고 전/후가 같다(§10-2 의 표: 22짝 가운데 20짝이 ±5% 안). 달라진 것은 지표의 분모다 — 후는 같은 일을 더 빨리 끝내고 자는 시간이 길어 빈 tick 이 는다(이 표본 1.5 ~ 2.2배, SKT `abc0c5789259` 27 ~ 50배). 따라서 «50M tick 당 CPU» 는 Java 로 도는 판에서 이 핀의 전/후를 재지 못하고, tick 이 부풀면 실제 CPU 퇴행도 가려진다. 그 두 표본은 §10-2 에서 CPU 초 · paints/초 · 게임이 청한 sleep 으로 다시 쟀다. `d1234e0ddb75` 는 회마다 값이 100배씩 흔들려 이 표의 수로 쓰지 않는다. 브라우저 축은 재지 않았다(§8).

### 7. upstream 병합 예행 — 충돌 수는 줄지 않았다

`git merge-tree --write-tree origin/main upstream/main`(전) 과 `… <이 브랜치> upstream/main`(후). `origin/main` = `1853f3b6f` · `upstream/main` = `24b670780`(behind **247** · merge-base `44fbf265` 그대로).

| 잰 것 | 전 | 후 |
|---|---:|---:|
| 충돌 파일 | 137 | **143** |
| 충돌 덩어리(`<<<<<<<`) | 590 | **659** |
| └ 한쪽이 빈 덩어리(그쪽을 그대로 받으면 되는 것) | 43 | 87 |
| └ 양쪽이 다 찬 덩어리 | 408 | 422 |
| └ 그중 «우리 쪽에 owner 없는 필드 접근이 남아 손으로 API 를 옮겨야 하는» 덩어리 | 69(300줄) | **2**(2줄 · 둘 다 `Jvm` 호출이 아닌 줄) |
| 트리 전체의 owner 없는 `get_field`/`put_field` 호출(병합 뒤 컴파일이 안 되는 곳) | 1,774 | **0** |

★**0551 §7 이 S1 에 건 기대 `N < 93`(지금 수로는 `< 137`)은 성립하지 않았다.** 파일 수는 6 늘었다(`clet_wrapper_card.rs` · `displayable.rs` · `ticker.rs` · `event_queue.rs` · `wipi_midlet.rs` · wipi `image.rs`). 까닭은 이렇다: 종전에는 upstream 만 고친 줄(owner 인자를 붙인 줄)이 충돌 없이 들어왔다. 이제 우리도 같은 줄을 고쳤으므로, upstream 이 그 줄 «옆»에 다른 것을 더한 자리는 전부 겹친 수정이 된다 — 내용이 같아도 git 은 충돌로 적는다. 늘어난 69 덩어리 가운데 44 가 한쪽이 빈 덩어리다.

이 조각이 실제로 치운 것은 충돌 «수»가 아니라 병합 뒤의 **이식 일**이다: 병합 커밋이 owner 를 정해야 하는 호출이 1,774 → 0 이 됐고, 충돌 덩어리 안에서 API 를 옮겨야 하는 곳이 69 → 2 가 됐다. S3 의 비용을 «충돌 파일 수»로 읽던 0551 · `STATE.md` 의 표는 이 점에서 고쳐 읽어야 한다.

### 8. 남긴 것

- **upstream 은 오늘 `^0.5.1` 로 갔다**(`263e6ac9b` #1630 · `24b670780` #1631). 이 조각은 티켓이 이름으로 건 0.4.1 까지다. lock 을 0.4.2 가 아니라 0.4.1 로 둔 것은 0.4.2 가 Java 프레임 깊이 512 에서 `StackOverflowError` 를 던지기 시작하기 때문이다(동작 변화 · 따로 재야 한다). 다음 조각은 worklog 제안으로 남겼다.
- **`InheritedMethods` 의 제거 조건이 섰다.** 그 주석은 «고정한 크레이트의 `invoke_special` 이 상위 클래스를 찾게 되면 지워라»이고, 0.4.1 의 `invoke_special` 은 `<init>` 이 아니면 `resolve_method` 로 상위 클래스를 찾는다. 이 조각에서는 지우지 않았다(동작을 하나 더 바꾸는 일이다).
- GC 루트 설계(우리 `guest_roots` ↔ upstream `ExternalRoots`) · KTF `InitSvcId` 7 충돌은 0551 대로 S3 의 몫이다.
- **못 잰 것**:

| 못 잰 것 | 왜 · 무엇이 대신하나 |
|---|---|
| 전수(429행) census · 입력 · 오래 하기 · 소리 축 | census 호스트 잠금을 다른 레인의 전수 실행이 쥐고 있었다. 표본 68판의 부팅 + 그림 짝만 쟀다(§6) |
| 브라우저 쪽(wasm 빌드 · `check-engine-contract.mjs` · `contract-roundtrip.mjs` · perf-watch 의 browser 축) | 로컬에서 wasm 을 빌드하지 않았다. wasm32 clippy(`-p wie-core-arm -p wie_featurephone`)는 통과했고, 아티팩트와 표면 계약은 PR 의 `contract` · `build-web` 이 잰다. zip 9 가 wasm 그래프에 `zeroize` · `constant_time_eq` · `typed-path` 를 들여오므로 아티팩트 크기는 달라진다 — 수는 재지 않았다 |
| J2ME 코퍼스 | `compat.json` 에 J2ME 행이 없다. 러너 블록의 `draw_j2me.jar` · `text_j2me.jar` 가 유일한 신호다 |
| 0.4.1 이 바꾼 실행기 의미 가운데 표본이 타지 않았을 수 있는 것 | `invokespecial` 의 `AbstractMethodError` · 인터페이스 메서드 해소(가장 구체적인 선언 고르기) · GC 가 활성 모니터를 루트로 치는 것. 시험 851개와 표본 68판이 거친 만큼만 확인됐다 |

### 9. PR 을 연 뒤 — main 병합 1회 (#599)

PR 의 CI 가 도는 사이 `origin/main` 이 #599(`contentKey` · 새 크레이트 `wie_identify`)로 전진해 PR 이 CONFLICTING 이 됐다. `git merge origin/main` 으로 받았다(rebase · force-push 없음).

- 충돌 2파일: `AGENTS.md` · `Cargo.toml` — 둘 다 같은 자리다(main 이 `patches/classfile` 줄 옆에 `wie_identify` 줄을 더했다). `wie_identify` 줄은 받고 `patches/classfile` 줄은 뺐다.
- ★충돌 밖에서 고친 것 하나: `wie_identify/Cargo.toml` 의 dev 의존 `zip ^8.6` → `^9.0`. 그대로 두면 lock 에 zip 8.6.0 이 되살아나 §4 에서 푼 자문 8건이 다시 뜬다. 병합 뒤 lock 의 zip 은 9.8.1 하나이고 `cargo audit` 은 취약점 0 이다.
- §1 ~ §8 의 수는 병합 «전» 머리(`1b52120e7`)에서 잰 것이다. 병합 트리의 게이트 · 러너 블록 결과는 PR 본문에 적었다. 코퍼스 표본 · perf-watch · 병합 예행은 다시 재지 않았다 — #599 는 엔진의 Java 실행 경로를 건드리지 않고(로더가 jar 항목을 고르는 함수 하나를 거치게 한 것 · 새 크레이트 · 스크립트), 받은 뒤 `compat.json` 의 착지 기준 바뀐 행은 0 이다(`check-compat-revert.mjs`).

### 10. 게이트② 반려 뒤 — main 을 다시 받고 F1 ~ F7 을 닫는다 (`…-fix` · 2026-11-22 15:50 ~)

게이트②(`1b52120e` 를 본 검수 · request-changes · F1 ~ F9)의 필수 수정 셋과 그 밖의 발견 F4 ~ F7 이다. F8 · F9 는 기록만이라 손대지 않았다. §10-1 ~ §10-6 의 수는 코드 트리 `ad44df027`(= PR 머리 `01680fa5c` + `origin/main` `76cc27c57` 일반 병합 + `=0.4.1` 고정)에서 쟀다. ★그 사이 main 이 또 전진해 한 번 더 받았다 — **최종 코드 트리는 `b32eb6dcd`** 이고 거기서 다시 잰 것은 §10-7 에 있다.

#### 10-1. main 병합 한 번 더 · lock · 감사 (F1) · `=0.4.1` (F5)

- `origin/main` 이 #601(가짜 서버 표 한 곳으로) · #604(네이티브 릴리스에서 `log` 의 trace 를 컴파일 단계에서 뺀다)로 전진해 `git merge origin/main` 으로 받았다(병합 커밋 `8dc9f44fc` · 충돌 0 · rebase · force-push 없음). #604 는 ARM 명령마다 돌던 검사를 빼는 변경이라 **전/후 짝의 «전» 은 이 절에서 전부 `76cc27c57`** 이다(검수의 «전» `1853f3b6` 과 다르다).
- F1 은 §9 의 병합이 이미 닫아 둔 상태였다(`wie_identify` dev 의존 `zip ^9.0`). 새 머리에서 다시 잰 값:

| 잰 것 | 값 |
|---|---|
| `cargo metadata --locked --offline` | rc=0 |
| `Cargo.lock` 의 `zip` | 1개 — 9.8.1 |
| 무-플래그 `cargo audit` | rc=0 · 취약점 **0** · 경고 2(`chrono` · `ttf-parser` — 대장 §A 그대로) |
| `node scripts/check-audit-warnings.mjs` | rc=0(억제 0건 · §D 행 0건) |
| `node scripts/content-key.mjs check`(네이티브 `wie_identify` + 이 머리에서 새로 빌드한 wasm) | rc=0 — `429 rows · 429 on disk` · committed == native 429/429 · native == wasm `identify()` 429/429 · 행의 platform/model == `identify()` 429/429 |
| `node scripts/content-key.mjs --selftest` | rc=0(픽스처 11개에서 native == wasm) |
| `node scripts/check-engine-contract.mjs`(같은 wasm) | rc=0 — 140 pass · 0 violation |
| 러너 블록(`AGENTS.md` 의 여섯 줄 그대로 · 16:35 · load1 39 ~ 45) | 6종 PASS rc=0 — `keydraw_ktf` `paints` 79 · `keydraw_lgt` `paints` 55 · 둘 다 `content` true · `input_steps` 27/27 |

- **F5**: 루트 `Cargo.toml` 의 다섯 크레이트(`jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime`)를 `^0.4.1` 에서 **`=0.4.1`** 로 고정하고 그 까닭을 그 줄 위 주석으로 옮겼다. `jvm` 0.4.2 는 `MAX_STACK_DEPTH = 512` 를 넘으면 `StackOverflowError` 를 던지고(`jvm-0.4.2/src/jvm.rs:44`), `.github/dependabot.yml` 이 cargo 를 매일 본다 — `^` 로 두면 막는 것이 lock 한 줄뿐이었다. `classfile` 은 직접 의존이 아니라(`jvm-bytecode` 가 끌어온다) 매니페스트로 고정할 수 없고 lock 이 잡는다(주석에 적었다). lock 은 바뀌지 않았다.
- 게이트(이 트리 · 한 줄에 임대 1건 · 16:11 ~ 16:22 · load1 29 ~ 53): `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · 같은 것에 `-p wie-core-arm -p wie_featurephone` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — **57 스위트 · 858 passed · 0 failed**(§9 의 857 + #604 의 `log_static_level` 1).

#### 10-2. 속도를 다시 쟀다 (F2) — CPU 초는 같고, 달라지는 것은 «같은 몫으로 하는 일»

`perf-watch` 의 native 축은 «50M tick 당 CPU ms» 다. `wie_validate` 의 루프는 쉬지 않아서 프로세스의 CPU 초는 스케줄러가 준 몫이고, tick 은 게스트가 자는 동안에도 는다. 그래서 J2ME · SKT 표본 넷(`perf-watch` 의 native 표본 `835acd535a5f` · `8238f5449347` · `d1234e0ddb75` + 검수가 짚은 `abc0c5789259`)을 CPU 초 · paints/초 · 게임이 청한 sleep 으로 다시 쟀다. 전(`76cc27c57`) · 후(이 머리) 릴리스 바이너리를 **같은 순간에 나란히**(2 프로세스 · `--timeout 30 --pacing 0` · `/usr/bin/time -l` · `--long` 임대). 세 묶음이다 — 16:01 ~ 16:07(네 판 · 3회씩), 그리고 load1 이 내려간 두 틈 17:08 ~ 17:10 · 17:31 ~ 17:33(두 판 · census 의 3 프로세스가 같이 돌았다). 표의 회 번호는 판마다 잰 순서다. CPU 몫 = CPU 초 ÷ 벽시계 30초.

| 판 | 회 | load1 | CPU 초 전 / 후 | CPU 몫 전 / 후 | tick 전 / 후 | paints/초 전 / 후 | 게임이 청한 sleep 횟수 전 / 후 | 그 길이 p50 ms |
|---|---:|---:|---|---|---|---|---|---|
| J2ME `835acd535a5f` | 1 | 29 → 36 | 14.89 / 14.56 | 0.48 / 0.45 | 13,340,936 / 25,510,769 | 22.6 / 22.9 | 706 / 738 | 28 / 30 |
| J2ME `835acd535a5f` | 2 | 36 → 42 | 14.03 / 13.43 | 0.47 / 0.45 | 12,410,715 / 24,525,073 | 20.9 / 23.2 | 627 / 693 | 28 / 30 |
| J2ME `835acd535a5f` | 3 | 42 → 31 | 24.03 / 23.94 | 0.80 / 0.80 | 25,472,349 / 44,930,860 | 34.6 / 32.8 | 1,041 / 989 | 28 / 30 |
| J2ME `835acd535a5f` | 4 | 15 → 24 | 24.88 / 24.65 | 0.83 / 0.82 | 27,038,395 / 47,895,608 | 35.7 / 33.6 | 1,075 / 1,013 | 28 / 30 |
| J2ME `835acd535a5f` | 5 | 24 → 20 | 27.16 / 27.13 | 0.91 / 0.90 | 38,293,909 / 58,242,187 | 35.1 / 32.8 | 1,052 / 988 | 28 / 30 |
| J2ME `835acd535a5f` | 6 | 20 → 46 | 12.97 / 13.17 | 0.43 / 0.44 | 9,567,960 / 21,212,045 | 24.4 / 27.4 | 732 / 822 | 27 / 29 |
| J2ME `835acd535a5f` | 7 | 12 → 19 | 26.62 / 26.60 | 0.89 / 0.89 | 24,213,416 / 49,956,744 | 36.6 / 34.1 | 1,104 / 1,027 | 27 / 30 |
| J2ME `835acd535a5f` | 8 | 19 → 21 | 25.56 / 25.60 | 0.85 / 0.85 | 24,966,304 / 48,247,777 | 36.1 / 33.7 | 1,085 / 1,012 | 27 / 30 |
| SKT `abc0c5789259` | 1 | 31 → 25 | 21.48 / 21.51 | 0.72 / 0.72 | 27,056 / 1,176,099 | 65.9 / 90.8 | 1,049 / 1,630 | 10 / 10 |
| SKT `abc0c5789259` | 2 | 25 → 20 | 26.88 / 27.44 | 0.90 / 0.91 | 40,932 / 2,051,478 | 90.5 / 110.7 | 1,388 / 2,006 | 10 / 10 |
| SKT `abc0c5789259` | 3 | 20 → 21 | 26.74 / 26.68 | 0.89 / 0.89 | 52,435 / 2,604,030 | 89.9 / 105.8 | 1,386 / 1,948 | 10 / 10 |
| SKT `abc0c5789259` | 4 | 46 → 61 | 14.69 / 26.38 | 0.49 / 0.88 | 13,694 / 2,038,530 | 48.9 / 107.2 | 777 / 1,981 | 10 / 10 |
| SKT `abc0c5789259` | 5 | 61 → 56 | 22.33 / 22.41 | 0.74 / 0.75 | 45,593 / 1,498,994 | 74.2 / 95.0 | 1,176 / 1,744 | 10 / 10 |
| SKT `abc0c5789259` | 6 | 56 → 46 | 23.81 / 23.89 | 0.79 / 0.80 | 47,302 / 1,291,440 | 82.0 / 102.8 | 1,266 / 1,820 | 10 / 10 |
| SKT `abc0c5789259` | 7 | 8 → 9 | 29.27 / 29.36 | 0.97 / 0.98 | 59,493 / 1,821,026 | 100.2 / 123.1 | 1,522 / 2,137 | 10 / 10 |
| SKT `abc0c5789259` | 8 | 9 → 12 | 26.58 / 28.57 | 0.89 / 0.95 | 39,513 / 1,795,805 | 88.7 / 119.8 | 1,379 / 2,121 | 10 / 10 |
| SKT `d1234e0ddb75` | 1 | 21 → 25 | 22.49 / 22.69 | 0.75 / 0.75 | 925,093 / 595,449 | 19.2 / 19.2 | 2 / 2 | 200 / 200 |
| SKT `d1234e0ddb75` | 2 | 25 → 26 | 20.42 / 20.24 | 0.68 / 0.67 | 439,995 / 1,679 | 19.1 / 19.0 | 2 / 2 | 200 / 200 |
| SKT `d1234e0ddb75` | 3 | 26 → 26 | 27.69 / 27.50 | 0.92 / 0.92 | 788,344 / 897,564 | 19.8 / 19.8 | 2 / 2 | 200 / 200 |
| J2ME `8238f5449347` | 1 | 26 → 26 | 27.62 / 27.64 | 0.92 / 0.92 | 198,008,409 / 198,138,648 | 0.0 / 0.0 | 0 / 0 | — / — |
| J2ME `8238f5449347` | 2 | 26 → 25 | 21.90 / 21.77 | 0.73 / 0.73 | 155,942,842 / 153,692,832 | 0.0 / 0.0 | 0 / 0 | — / — |
| J2ME `8238f5449347` | 3 | 25 → 27 | 25.93 / 25.53 | 0.86 / 0.85 | 186,313,676 / 182,770,341 | 0.0 / 0.0 | 0 / 0 | — / — |

- **CPU 초는 22짝 가운데 20짝이 전/후 같다**(차이 최대 4.3%). 「CPU 가 3분의 1」 은 없다. 두 짝(`abc0c5789259` 4회 — 전 14.69 · 후 26.38, 8회 — 전 26.58 · 후 28.57)은 CPU 몫 자체가 갈려서 아래 비교에서 뺐다.
- **`835acd535a5f`(J2ME) — 프레임 마감에 맞춰 자는 판.** 청한 sleep 의 p50 이 전 27 ~ 28 ms · 후 29 ~ 30 ms 다(여덟 회 모두 2 ~ 3 ms 차이) — 게임이 잰 한 프레임의 일이 약 2 ms 줄어 그만큼 길게 잔다.
  - CPU 가 넉넉할 때(몫 0.80 ~ 0.91 · 3 · 4 · 5 · 7 · 8회): paints/초 34.6 → 32.8 · 35.7 → 33.6 · 35.1 → 32.8 · 36.6 → 34.1 · 36.1 → 33.7, **후가 5 ~ 7% 덜 그린다**. 프레임 주기로는 약 28 ms → 30 ms 이고, 30 ms 는 게임이 청하는 sleep 의 상한(겨냥한 주기)이다.
  - CPU 가 모자랄 때(몫 0.43 ~ 0.48 · 1 · 2 · 6회): 22.6 → 22.9 · 20.9 → 23.2 · 24.4 → 27.4, **후가 1 ~ 12% 더 그린다**. 검수가 본 +33 ~ 99% 는 몫 0.22(60초에 13초)에서의 값이다.
  - tick 이 1.5 ~ 2.2배인 것은 일을 빨리 끝내고 자는 동안 빈 tick 이 늘어서다.
  - 전의 주기가 30 이 아니라 28 ms 였던 까닭(청한 28 ms + 일 2 ms 면 30 이어야 한다)은 이 측정에서는 가르지 않았다. `executor.rs` 는 전/후 같은 코드라 이 핀이 만든 차이는 아니다 — 답은 §10-7 에 있다(종전 실행기가 잠을 «일한 만큼» 줄였고, 그 사이 main 에 들어온 0573 이 그것을 없앴다).
- **`abc0c5789259`(SKT) — 마감 없이 도는 판. 걸음이 빨라진다. 원인은 우리 pacing 이 아니라 Java 실행 비용이다.** CPU 초가 같은 여섯 짝(몫 0.72 ~ 0.98)에서 paints/초가 +38% · +22% · +18% · +28% · +25% · +23%, 게임 루프(청한 sleep 횟수)가 +55% · +45% · +41% · +48% · +44% · +40% 다. **가장 한가했던 짝(7회 · load1 8 · 몫 0.97 / 0.98)에서도 +23% · 루프 +40%** 다. 이 판은 `sleep(10)` 만 하고(p05 · p50 · p95 가 전/후 모두 10 ms) 나머지를 일로 채우므로, 한 바퀴의 일이 싸지면 그만큼 빨리 돈다.
  - 프로파일(`sample` 10초 · 전/후 따로): 전은 워커 스레드 표본 7,280 가운데 7,273(99.9%)이 게임 Java(이벤트 디스패치 · 페인트) 안이다 — 쉬는 구간이 없다. 그중 **3,068(42%)이 malloc/free 이고 2,571 이 `jvm_bytecode` 의 `ClassDefinitionImpl::field` · `method` 조회**에서 온다. 후는 malloc/free 가 1,195(17%)이고 그 자리는 상위 8곳에 없다.
  - 소스: 그 조회의 본문은 두 판이 글자까지 같다(후보마다 `field.name() == name && field.descriptor() == descriptor`). 바뀐 것은 `name()` · `descriptor()` 의 반환형이다 — 0.1.1 은 `String`(후보마다 할당 2번), 0.2.0 부터 `Cow<'_, str>`(빌려 읽기 · §1 표의 첫 줄).
  - 우리 pacing 쪽은 그대로다: `wie-backend/src/executor.rs` · `pacing.rs` · `wie_validate.rs` 는 전/후 diff 0 줄이고, `rustjava-runtime` 의 `Thread.sleep` · `yield` 본문도 0.1.1 과 0.4.1 이 같다. `test_pacing` 2개 · `wie-backend` pacing 시험은 통과한다(걸음 시계라 이 차이를 보지 못한다 — 검수 지적대로).
- `d1234e0ddb75`(SKT)는 paints/초가 세 회 모두 같다(19.2 · 19.1 · 19.8). `8238f5449347`(J2ME · 한 번 그리고 쉰다)은 tick 까지 같다.
- **판정 한 줄**: 티켓이 물은 «부하가 낮을 때 paints/초가 전/후 같은가» 의 답은 **같지 않다** 이다 — 다만 엔진의 pacing 의미(executor · sleep · 타이머)는 바뀌지 않았고, 바뀐 것은 Java 필드 · 메서드 조회 비용이다. 그 결과 ⑴ 마감에 맞춰 자는 판은 CPU 가 넉넉하면 게임 자신의 계산대로 조금 길게 자고(표본: 주기 28 → 30 ms) 모자라면 더 그리며 ⑵ **마감 없이 도는 판은 CPU 가 넉넉해도 빨라진다**(표본: 루프 +40 ~ 55%). ⑵ 는 이용자에게 「일부 Java 판이 빨라진다」 로 보일 수 있는 변화다 — 브라우저에서는 재지 않았고, 그래서 `docs/player-updates/` 항목은 더하지 않았다(회신에 열린 판단으로 올렸다).
- ★가르지 못한 것: 가장 한가한 값이 load1 8 · CPU 몫 0.97 의 한 짝이다(census 가 같이 돌았다) — 아무것도 돌지 않는 호스트의 값은 없다. 코퍼스의 다른 판 가운데 ⑵ 꼴이 몇인지는 세지 않았다. 표본은 판마다 3 ~ 8짝이다.
- 도구 · 원자료 표: `~/orchestrator/reports/evidence/wie-upstream-sync2-s1-rustjava-pin-0-4-1-before-upstream-sync-fix/`(`tools/f2.sh` · `f2-table.md` · `f2-sample-alloc.txt` — 저장소 밖).

#### 10-3. 전수 census 전/후 짝 (F3) — 429행에서 다시 재도 남는 판정 변화 0

`scripts/playability-census.mjs run --only probe` 를 전(`76cc27c57`) · 후(이 머리) 릴리스 `wie_validate` 로 **429행 전부**에 돌렸다. 프로브 A(키 27개) · B(키 없음) · `--jobs` 기본(3) · 호스트 잠금 · `scripts/census-drive.sh` 를 쪽마다 한 호출(임대 `--long` 1건 · 15분)씩 **전 → 후 → 전 → 후** 로 번갈아 불렀다(임대 10건 · 16:07 ~ 18:30 · 프로브가 찍은 load1 7 ~ 81). 같은 행의 전/후는 15 ~ 30분 사이에 잰 것이다.

- ★`--secs 15` 다(기본 30). 30초로 두 쪽을 돌리면 약 4.8시간이라 이 회차(240분)에 들어가지 않는다. 그래서 여기 수는 `compat.json` 의 판정과 견주는 수가 아니라 **전/후끼리만** 견주는 수다.
- 미측 0 · 굶어서 기록되지 않은 프로브 0 · 두 쪽 모두 429행.

| 축 | 전 | 후 |
|---|---|---|
| 부팅 | ok 427 · fail 2 | ok 427 · fail 2 |
| 그림 | ok 427 · none 2 | ok 427 · none 2 |
| 입력 | ok 425 · none 2 · n/a 2 | ok 425 · none 2 · n/a 2 |
| 소리(프로브가 들은 것) | ok 402 · silent 25 · n/a 2 | ok 404 · silent 23 · n/a 2 |
| 통신사별 (부팅 · 그림 · 입력 모두 ok) | KTF 266/269 · LGT 78/78 · SKT 81/82 | KTF 266/269 · LGT 78/78 · SKT 81/82 |

| 행 단위 대조 | 행 |
|---|---:|
| 부팅 · 그림 · 입력 · 소리가 전/후 같다 | **425** |
| 1차에서 그 넷 중 하나가 달랐다 | 4 |
| └ 그 4행을 두 바이너리로 2번 더 쟀을 때 다시 달랐다 | **0** |
| `wie_validate` 의 `result` 가 다른 행(A 또는 B) | 0 |
| 첫 Java 예외가 다른 행 | 0 |
| Java 예외 «수» 가 다른 행 | 3 |

1차에서 달랐던 4행(sha12 · 올림 3 · 내림 1):

| 행 | 통신사 | 1차 | 재측 1 | 재측 2 |
|---|---|---|---|---|
| `0606d43702e7` | LGT | 소리 silent → ok | 전/후 같다(다섯 축 ok) | 전/후 같다 |
| `23919eb33365` | KTF | 입력 none → ok | 전/후 같다 | 전/후 같다 |
| `3d38a46becd9` | KTF | 입력 ok → none | 전/후 같다 | 전/후 같다 |
| `5028b8a5d19f` | KTF | 소리 silent → ok | 전/후 같다 | 전/후 같다 |

- 재측은 그 4행만 같은 도구 · 같은 예산으로 두 바이너리에 다시 돌린 것이다(18:31 ~ 18:35 · load1 약 10). 네 행 모두 두 번 다 전/후의 다섯 축이 ok 로 같았다 — 1차의 차이는 재현되지 않았다.
- **속도 축만 다른 행은 10** 이다(ok → n/a 4: `0a6f495f0b35` · `0bc9cc28c05a` · `2d5cada03004` · `31c90441f639` / n/a → ok 6: `14a62a8521a0` · `249e655147a1` · `2f84b8cc870d` · `71d1d8235bd1` · `7218e8720f8c` · `e9ea67541b01`). headless 의 속도 축은 벽시계 늦음 비율(0.9 이상이면 ok · 아니면 n/a)이라 호스트 부하를 따라 움직이고, 주간 감시(`regression-diff.mjs`)도 이 축은 견주지 않는다. 방향이 4 : 6 으로 갈려 한쪽으로 쏠리지 않았다. 다시 재지 않았다.
- F4 표시: 게임 클래스가 lwc 컴포넌트를 상속하는 19판(§10-5)은 **19판 모두 다섯 축이 전/후 같다**. 위 14행(4 + 10)에 든 것은 0판이다.
- ★이 표가 못 보는 것: 오래 하기(600초) · 진행 축은 재지 않았다. 판정 `status` 는 오래 하기가 없어 전 행이 `limited` 로 나오므로 축으로만 견줬다. 15초 프로브는 30초 프로브보다 얕다 — 키 27개가 7초 창에 들어간다.
- 도구 · 출력(sha12 만): 증적 디렉터리의 `tools/census-pair.sh`(저장소의 `census-drive.sh` 를 번갈아 부르는 고리) · `tools/census-diff.py` · `census-diff-first-pass.txt` · `census-diff-recheck-{1,2}.txt` · `census-pair.log`.

#### 10-4. `patches/classfile` 대조를 다시 셌다 (F6)

처음 판의 스크래치 대조기는 지워져 수를 재현할 수 없었다. 다시 써서 증적에 넣었다(`…/evidence/wie-upstream-sync2-s1-rustjava-pin-0-4-1-before-upstream-sync-fix/classfile-compare/` — `Cargo.toml` · `src/main.rs` · `output.txt`. 패치판은 `git archive 1853f3b6f patches/classfile` 로 꺼낸다). 패치판 0.1.1 과 crates.io 0.4.1 을 한 바이너리에 묶어 같은 바이트를 두 판으로 읽고, 두 파서와 무관하게 상수 풀을 직접 걸어 4바이트 UTF-8 선두(`F0`+) · `C0 80` · 서로게이트 반쪽을 센다.

| 디렉터리(`game_lab/` 아래) | 꾸러미(경로) | 아카이브(안의 jar 포함) | `.class` 엔트리 | 그중 서로 다른 내용 | 두 판이 다 읽음 | 한쪽만 읽음 | 둘 다 못 읽음 | Utf8 상수 | 비 ASCII | 다르게 읽은 상수 | `F0`+ | `C0 80` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `working` | 294 | 596 | 1,148 | 1,025 | 1,025 | 0 | 0 | 112,178 | 12,909 | **0** | 0 | 0 |
| `broken` | 170 | 340 | 630 | 628 | 628 | 0 | 0 | 68,079 | 8,330 | **0** | 0 | 10 |
| `_dup` | 17 | 34 | 2 | 2 | 2 | 0 | 0 | 78 | 0 | **0** | 0 | 0 |
| `_nongame` | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| `vendor_sdk` | 47 | 47 | 12,950 | 8,090 | 7,904 | 0 | 186 | 551,809 | 16 | **0** | 0 | 70 |

- 처음 판의 «클래스 14,730» 은 다섯 디렉터리의 `.class` 엔트리 합(1,148 + 630 + 2 + 0 + 12,950)이다 — **12,950 이 `vendor_sdk`** 다. Utf8 상수 1,027,138 · 못 읽는 391 은 같은 내용을 꾸러미마다 다시 센 수로 읽힌다(그 스크립트가 남아 있지 않아 확인은 못 했다). 위 표는 디렉터리 안에서 내용이 같은 클래스를 한 번만 센다 — 그래서 상수 수가 작다. 꾸러미 527 ↔ 여기 529 의 차이 2 는 가르지 않았다.
- 검수의 독립 계수(`working` + `broken` · 고유 꾸러미 432 · 클래스 1,768 · Utf8 188,031 · 비 ASCII 21,764 · `C0 80` 10 · `F0`+ 0)와 맞춰 보면: 여기는 경로 464(중복 꾸러미 포함)의 `.class` 엔트리 **1,778** · 디렉터리별 고유 1,653 · Utf8 180,257 · 비 ASCII 21,239 · `C0 80` 10 · `F0`+ **0**. 셈의 단위(고유 꾸러미 ↔ 경로 · 디렉터리별 고유)가 달라 수가 조금 다르고, 결론은 같다.
- ★SKT 꾸러미의 jar 는 앞에 머리가 붙어 `PK` 로 시작하지 않는다. 처음에 «`PK` 로 시작하는 엔트리만 연다» 로 썼다가 `.class` 가 20개만 나와서 알았다 — zip 읽기는 디렉터리를 끝에서 찾으므로 이름(`.jar` · `.zip`)으로도 열게 고쳤다. 같은 실수가 §10-5 의 문자열 스캔에도 있었고 같이 고쳤다.
- 결론은 그대로다: 두 판이 다르게 읽는 상수 0 · 한쪽만 읽는 클래스 0 · 4바이트 꼴 0.

#### 10-5. 0382 뒤에 생긴 «가릴 수 있는» 엔진 필드 (F4) — 코퍼스에 0판

0382(09-29) 뒤로 `org/kwis/msp/lwc/Component` 에 인스턴스 필드 `x` · `y` · `w` · `h`(protected I), `javax/microedition/lcdui/Canvas` 에 `standardKeyCodes` 가 생겼다.

- **`standardKeyCodes`**: `static` 필드이고 엔진은 `get_static_field("javax/microedition/lcdui/Canvas", …)` 로 클래스 이름을 적어 읽는다 — 인스턴스의 실행 중 클래스를 타던 자리가 아니라서 이 핀으로 읽는 칸이 바뀌지 않는다. 그 문자열을 가진 꾸러미도 코퍼스에 **0**이다(`working` + `broken` 고유 432 · 안의 jar 까지 풀어서 · SKT 의 머리 붙은 jar 포함).
- **`Component.x/y/w/h`**: KTF · LGT 는 클래스가 ARM 이미지 안이라 정적으로 못 센다. 0382 의 등록 탐침을 0.4.1 에 다시 넣었다(스크래치의 `jvm` 0.4.1 사본 · `[patch.crates-io]` · 저장소에 넣지 않음 — `register_class_internal` 에서 «자기 인스턴스 필드 ∩ 조상 사슬의 인스턴스 필드» 를 (이름 · 서술자)로 기록).
  - 대상: `org/kwis/msp/lwc/` 문자열을 가진 꾸러미 **213**(KTF 195 · LGT 17 · SKT 1 — 그 클래스를 상속하려면 이름이 꾸러미 안에 있어야 한다) · 각 `wie_validate --inject` 1판(키 27개 일정 · tick 상한 해제) · 등록된 클래스 KTF 23,339 · LGT 2,211 · SKT 99.
  - 게임 클래스가 lwc 컴포넌트 아래에 있는 판: **19**(KTF 17 · LGT 2 — `ShellComponent` 6 · `TextFieldComponent` 5 · `FormComponent` 4 · `TextBoxComponent` 4 · `AnnunciatorComponent` 2 · `Component` 2 · `TextComponent` 1 · `ContainerComponent` 1).
  - 그 가운데 lwc 조상과 (이름 · 서술자)가 겹치는 인스턴스 필드를 선언한 판: **0**. 즉 `Component.x/y/w/h` 를 가리는 판은 코퍼스에 없다.
  - 탐침이 제대로 보는지: 같은 실행에서 0382 의 수가 그대로 나온다 — `Card.display` KTF 13 · `Card.x` 10 · `Card.y` 7 · `Card.w` 6 + SKT 1 · `Card.h` 6 + SKT 1 · `Jlet.dis` 2. 엔진 조상과 겹친 것은 이 여섯 가지뿐이다.
- 한계는 0382 와 같다: 그 판에서 **등록된 클래스만** 본다. 키 27개가 닿지 않는 화면에서만 올라오는 클래스는 빠진다. 213판 가운데 11판은 스스로 끝났고(`clean exit` · `UNMEASURED`) 1판은 마감까지 돌고도 `UNMEASURED` 였다 — 그 12판은 올라온 클래스만 봤다.
- 도구: 증적 디렉터리의 `tools/jvm-0.4.1-shadowprobe.patch` · `tools/f4.sh` · `tools/f4-count.py` · `tools/strscan.py` · `f4-count.txt`.

#### 10-6. 이 회차의 경계

- 에뮬레이터 실행은 전부 `build-slot run --long` 임대 안이다. ★census(3 프로세스)가 도는 사이 다른 임대로 겹쳐 돌린 것이 셋 있다(긴 임대 2칸을 내가 같이 쥔 시간이다) — 속도 짝 2 · 3차(2 프로세스 · 3분 · 2분 — load1 이 낮은 틈을 쓰려고), 가림 탐침(2 프로세스 · 17:54 ~ 18:30). 그 사이 내 에뮬레이터 프로세스는 최대 5개였다. 러너 블록 6회 · 탐침 확인 1회 · `sample` 2회는 임대 없이 한 프로세스씩 돌렸다.
- 가림 탐침은 처음에 tick 상한(기본 5천만)을 둔 채 돌려, 끝난 12판 중 8판이 키를 다 넣기 전에 `max-ticks` 로 끝났다. 멈추고 상한을 풀어 처음부터 다시 돌렸다(위 수는 다시 돌린 것이다).
- 스크래치(`/tmp/s1fix` — 바이너리 넷 · census 출력 · 탐침 트리)와 «전» 을 빌드한 임시 워크트리는 회차 끝에 지웠다. 증적에 남긴 것은 도구와 sha12 만 든 출력이다.

#### 10-7. census 가 도는 사이 main 이 또 전진했다 — 한 번 더 받고 최종 트리에서 다시 쟀다

§10-1 ~ §10-5 를 재는 2시간 반 사이 `origin/main` 이 7건(`d65954359` #606 · `2dfff4f2d` 0570 · `0b50fb735` #608 · `e843ce009` #609 · `b52f6c46f` 0573 · `90861ad7a` #610 · `afd4b0627` #607) 더 전진했다. 그 가운데 **0573(조각 S2)은 실행기의 시간 의미를 바꾼다** — `sleep(n)` 의 마감을 «그 step 이 시작한 때 + n» 에서 «부른 때 + n» 으로 센다. `git merge origin/main` 으로 받았다(병합 커밋 `b32eb6dcd` · 충돌 0). **최종 코드 트리 = `b32eb6dcd`** 이고 그 뒤 더한 것은 이 문서뿐이다. 이 트리에서 다시 잰 것:

| 잰 것(최종 트리 `b32eb6dcd` · 18:39 ~ 18:56 · load1 5 ~ 15) | 값 |
|---|---|
| `cargo metadata --locked --offline` · lock 의 `zip` | rc=0 · 9.8.1 하나 |
| 무-플래그 `cargo audit` · `check-audit-warnings.mjs` | rc=0 · 취약점 0 · 경고 2 / rc=0 |
| `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm32 clippy(기본 + `-p wie-core-arm -p wie_featurephone`) · `cargo +beta clippy --all -- -D warnings` | 전부 rc=0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | rc=0 — **58 스위트 · 864 passed · 0 failed** |
| 러너 블록 6종 | PASS rc=0 — `keydraw_ktf` `paints` 79 · `keydraw_lgt` `paints` 55 · 둘 다 `content` true · `input_steps` 27/27 |
| `content-key.mjs check`(이 트리의 네이티브 + wasm) | rc=0 — 429/429 · 429/429 · 429/429 |
| `check-engine-contract.mjs` | rc=0 — 140 pass · 0 violation |

**속도 — 최종 트리의 전(`afd4b0627`) · 후(`b32eb6dcd`)**. 같은 방법 · 2짝씩 · 18:44 ~ 18:46 · load1 10 ~ 15. 이번에는 CPU 몫 0.98 ~ 1.00 의 짝이 나왔다.

| 판 | 회 | CPU 초 전 / 후 | CPU 몫 전 / 후 | paints/초 전 / 후 | 게임이 청한 sleep 횟수 전 / 후 | 그 길이 p50 ms |
|---|---:|---|---|---|---|---|
| J2ME `835acd535a5f` | 1 | 19.02 / 18.93 | 0.59 / 0.61 | 22.9 / 24.4 | 734 / 756 | 28 / 30 |
| J2ME `835acd535a5f` | 2 | 29.84 / 29.85 | 0.99 / 0.99 | **30.3 / 30.3** | 909 / 909 | 28 / 30 |
| SKT `abc0c5789259` | 1 | 29.51 / 29.53 | 0.98 / 0.98 | **70.8 / 108.2** | 1,065 / 1,732 | 10 / 10 |
| SKT `abc0c5789259` | 2 | 29.93 / 29.93 | 1.00 / 1.00 | **72.0 / 109.5** | 1,084 / 1,755 | 10 / 10 |

- **마감형 판(`835acd535a5f`)은 CPU 가 넉넉하면 전/후가 똑같다**(30.3 / 30.3 · 주기 33.0 ms — 0573 이 적은 «요청한 주기» 그대로). §10-2 에서 후가 5 ~ 7% 덜 그리던 것은 종전 실행기가 «자기 전에 한 일만큼» 잠을 줄이던 것의 그림자였고(그래서 일이 2 ms 줄자 주기가 2 ms 늘었다), 0573 이 들어온 지금은 없다. §10-2 가 «가르지 않았다» 고 남긴 28 ms 의 까닭이 이것이다. CPU 가 모자라면(몫 0.6) 후가 여전히 더 그린다(+7%).
- **마감 없이 도는 판(`abc0c5789259`)은 CPU 한 코어를 다 쓰는 상태에서도 +52 ~ 53%**(루프 +62 ~ 63%)다. §10-2 의 +18 ~ 38% 보다 큰 것은 0573 뒤로 «전» 이 느려졌기 때문이다(전 100 → 72/초 · 후 123 → 109/초 — sleep 이 일한 만큼 줄지 않게 되어 한 바퀴가 «일 + 10 ms» 가 됐고, 일이 큰 쪽이 더 느려진다). 원인(§10-2 의 조회 할당)은 같다.
- 판정은 §10-2 그대로이고 수만 최종 트리 것으로 읽으면 된다: **이 핀은 마감형 판의 걸음을 바꾸지 않고, 마감 없이 도는 Java 판을 빠르게 한다.**

**코퍼스 — 최종 트리의 전/후 짝(표본 28행)**. 전수를 한 번 더 돌릴 시간은 없었다. `compat.json` 의 playable 행을 통신사마다 sha 순 한가운데서 8개씩(KTF 8 · LGT 8 · SKT 8) + §10-3 의 1차에서 달랐던 4행 + `abc0c5789259`(그 5행 중 1행은 24개와 겹쳐 합 28행 — KTF 11 · LGT 9 · SKT 8)를 같은 도구 · `--secs 15` 로 쟀다(18:47 ~ 18:56 · load1 6 ~ 10): **28행 모두 부팅 · 그림 · 입력 · 소리 · 속도가 전/후 같다** · `result` 다른 행 0 · 첫 예외 다른 행 0 · 예외 수 다른 행 1.

- ★남는 틈: §10-3 의 전수 429행 · §10-5 의 탐침은 `ad44df027` ↔ `76cc27c57` 에서 잰 것이다. 최종 트리(이 핀 + 0573 의 실행기)의 코퍼스 실행은 위 28행이 전부다. 0573 은 그 PR 에서 따로 쟀고(`docs/report/0573`) 이 핀과 겹치는 파일은 `event_queue.rs` 하나(병합 충돌 0 · 시험 통과)다.

**게임 파일명 유입**: `node scripts/corpus-name-inflow.mjs` — 유입 65건(BOUNDED) · 판단 필요 8건(SUFFIX-ATTACHED) · 대상 115파일. 둘 다 이 브랜치가 건드린 파일에 «이미 있던» 언급이다(`docs/project-kb/02_status.md` · LGT `jvm_support.rs` 의 주석 등 — 도구는 바뀐 파일의 본문 전체를 센다). 이 브랜치가 «더한 줄»에 든 것은 0건이다(도구가 낸 72쌍을 `git diff origin/main...HEAD` 의 추가 줄과 대조). SUFFIX-ATTACHED 8쌍도 같은 대조에서 0 이다.

<!-- corpus-name-inflow v1 subjects=116 tree=57fd904b8e9e13a8 B=115/65 P=17/2 S=10/8 -->
