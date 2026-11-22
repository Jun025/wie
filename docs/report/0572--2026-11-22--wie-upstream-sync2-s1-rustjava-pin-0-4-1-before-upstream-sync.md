## [2026-11-22] 조각 S1 — RustJava 핀 0.1.1 → 0.4.1 (upstream 병합 전 · 우리 쪽만) · `patches/classfile` 삭제 · zip 9 · 감사 억제 8건 해제 (wie-upstream-sync2-s1-rustjava-pin-0-4-1-before-upstream-sync)

**무엇을**: `docs/report/0551` §7 의 S1 이다. 루트 `Cargo.toml` 의 `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` 을 `^0.1.1` 에서 `^0.4.1` 로 올렸고(lock 은 정확히 0.4.1), 그 판이 바꾼 API 를 우리 코드 전부에서 따라갔다. `git merge upstream/main` 은 하지 않았다.

| 축 | 전 | 후 |
|---|---|---|
| `jvm` · `jvm-bytecode` · `jvm-class-proto` · `jvm-types` · `rustjava-runtime` | 0.1.1 | **0.4.1** |
| `classfile` | 0.1.1 + 우리 패치(`patches/classfile` · `[patch.crates-io]`) | crates.io **0.4.1** 그대로 — 패치 디렉터리와 `[patch]` 줄을 지웠다 |
| `zip` | 8.6.0(`wie-backend` 직접 + `rustjava-runtime` 간접) | **9.8.1** 하나(`wie-backend` `^8.6` → `^9.0`) |
| 감사 워크플로의 `--ignore` | 8개(zip · 대장 §D 3행) | **0개**(§D → C-16) |
| lock 에 새로 든 크레이트 | — | `constant_time_eq 0.6.1` · `typed-path 0.13.0` · `zeroize_derive 1.5.0`(zip 9 의 의존) |

**왜**: 티켓 `wie-upstream-sync2-s1-rustjava-pin-0-4-1-before-upstream-sync`(운영자 채택 제안 `2026-11-06-upstream-sync2-dry-run-classify#p0`). upstream 병합에서 Java 표면 파일이 손 수정이 되는 가장 큰 원인이 RustJava 0.3.0 의 «필드 접근에 owner 클래스 인자»였다. 그 몫을 병합 전에 우리 쪽에서 치운다.

**사용자 영향**: 지원 목록의 판정이 바뀐 판은 표본 68판에서 **0**이다(§6). 엔진 쪽에서 바뀌는 것은 셋이다. ⑴ 엔진의 Java 클래스가 이제 «자기 필드»를 읽는다 — 게임 클래스가 같은 이름의 필드를 선언해 엔진 값을 가리던 종류의 오동작(0382)이 9곳이 아니라 전부에서 막힌다. ⑵ Java 로 도는 표본 하나의 CPU 가 3분의 1 로 줄었고 나머지 여섯은 ±1% 안이다(§6 속도). ⑶ zip 9 로 가면서 감사 억제 8건이 0건이 됐다. 이용자에게 보이는 동작 변화로 확인된 것이 없어 `docs/player-updates/` 항목은 더하지 않았다. `compat.json` 은 바뀌지 않는다. `Cargo.lock` 과 엔진 `*.rs` 가 바뀌므로 착지하면 otterpebble 핀 범프가 뜬다.

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
- ★**의미가 바뀐다.** 종전 `get_field` 는 «인스턴스의 실행 중 클래스»에서 이름을 찾았다. 그래서 게임 클래스가 같은 이름 · 같은 형의 필드를 선언하면 엔진 코드가 게임의 필드를 읽었다(`docs/report/0374` · `0382` — `Card` 의 x · w, `MIDlet` · `Jlet` 의 display). 그때 고친 자리는 `wie-jvm-support/src/declared_field.rs` 의 도우미를 쓰는 9곳뿐이었다. 이제는 모든 호출이 owner 로 적은 클래스의 필드를 읽는다 — 엔진 코드에서는 «엔진 클래스 자신의 필드»다. 그 도우미는 새 API 와 같은 일을 하므로 지웠고, 가림 시험 3개(`card_bounds_are_cards_own_fields_not_a_subclasss` · `display_is_the_midlets_own_field_not_a_subclasss` · `display_is_the_jlets_own_field_not_a_subclasss`)는 owner 를 하위 클래스로 적어 그대로 남겼다.

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
- `WIE_PERF_AXES=native bash scripts/perf-watch.sh compare <전 바이너리> <후 바이너리>`(고정 표본 7 · 3회 · 50M tick 당 CPU ms · 회마다 `--long` 임대 1건 · load1 80 ~ 171): **퇴행 0 · 1차 경보 0 · 미측정 0/7 · 편차에 묻힘 0**.

| 표본 | 회별 «후가 전보다 느린 %» | 중앙값 | 판정 |
|---|---|---:|---|
| `0606d43702e7` | −1.3 · −1.8 · 0.6 | −1.3 | ok |
| `1065985081fc` | −0.5 · −0.2 · −1.1 | −0.5 | ok |
| `155972cac664` | 9.3 · −1.8 · −0.8 | −0.8 | ok |
| `3cc7a9b4cb15` | −0.8 · −12 · 5.5 | −0.8 | ok |
| `8238f5449347` | −0.1 · −0.1 · −0.3 | −0.1 | ok |
| `835acd535a5f` | −66.7 · −67.3 · −71.2 | **−67.3** | ok |
| `d1234e0ddb75` | −27.2 · −49 · −2.5 | −27.2 | ok(편차 19) |

`835acd535a5f` 는 세 회 모두 CPU 가 3분의 1 로 줄었다(전 136,353 ~ 273,407 · 후 44,631 ~ 91,039 · 둘 다 `deadline` 종료). 새 판의 실행기 쪽 변화(#1542 의 «runtime optimizations»)가 이 표본이 타는 길에 걸린 것으로 읽었고, 어느 변화인지는 가르지 않았다. `d1234e0ddb75` 는 회마다 값이 100배씩 흔들려 수로 쓰지 않는다. 브라우저 축은 재지 않았다(§8).

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

**게임 파일명 유입**: `node scripts/corpus-name-inflow.mjs` — 유입 65건(BOUNDED) · 판단 필요 8건(SUFFIX-ATTACHED) · 대상 115파일. 둘 다 이 브랜치가 건드린 파일에 «이미 있던» 언급이다(`docs/project-kb/02_status.md` · LGT `jvm_support.rs` 의 주석 등 — 도구는 바뀐 파일의 본문 전체를 센다). 이 브랜치가 «더한 줄»에 든 것은 0건이다(도구가 낸 72쌍을 `git diff origin/main...HEAD` 의 추가 줄과 대조). SUFFIX-ATTACHED 8쌍도 같은 대조에서 0 이다.

<!-- corpus-name-inflow v1 subjects=116 tree=19e2e1e8680905c9 B=115/65 P=11/2 S=10/8 -->
