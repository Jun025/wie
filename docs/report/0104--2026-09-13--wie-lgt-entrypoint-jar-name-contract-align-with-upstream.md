## [2026-09-13] LGT 엔트리포인트 jar — 이름을 «지어내지» 말고 내용으로 찾는다 (wie-lgt-entrypoint-jar-name-contract-align-with-upstream)

- **무엇을**: `LgtEmulator::from_archive` 가 엔트리포인트 컨테이너 이름을 `format!("{}.jar", app_info.aid)`
  로 **만들던** 것을, upstream 과 **같은 3줄**로 바꿨다 — 아카이브에서 `*.jar` 중 zip 안에 `binary.mod` 가
  있는 첫 파일(`Self::loadable_jar`)을 고른다. 덧붙여 `do_start` 의 `binary.mod` 스트림 `Option::unwrap()`
  을 `WieError::FatalError("Missing binary.mod in {jar_filename}")` 로 바꿨다(이 결함이 실제로 표면화되는
  자리이고, 옛 문면은 jar 이름을 말하지 않았다). 시험은 `wie_lgt/tests/test_helloworld.rs` 에
  `test_helloworld_jar_named_application` 1건 추가 — 픽스처의 `00000000.jar` 를 `application.jar` 로
  개명해 넘긴다(upstream 테스트와 같은 형태).

- **★왜 — 티켓의 전제를 실측이 정정했다**: 티켓 Goal 은 「upstream `LgtEmulator` 는 `application.jar` 를
  찾는데 우리는 `00000000.jar` 를 넘긴다」였다. ★**upstream 은 어떤 이름도 하드코딩하지 «않는다»** —
  `git grep 'application\.jar' upstream/main` 의 **유일한 일치는 테스트 1줄**
  (`wie-lgt/tests/test_helloworld.rs:35`)이고, 코드는 내용 탐색이다
  (`upstream/main:wie-lgt/src/emulator.rs:50-53`). 2026-08-23 upstream PR **#1368**(`9a88423b`)이
  우리와 **같았던** `format!("{}.jar", app_info.aid)` 를 그 형태로 바꾸면서, **같은 커밋에서** 테스트가
  픽스처 jar 를 `application.jar` 로 개명하도록 고쳤다 — 즉 그 개명은 규약이 아니라 ★**탐색이 실제로
  도는지 증명하는 장치**다(AID 이름 그대로면 옛 규칙으로도 우연히 통과한다).
  ⇒ 결함의 정확한 형태는 「이름이 둘이다」가 아니라 ★**「우리는 이름을 지어내고 upstream 은 찾는다」**이고,
  그래서 ⒤(`application.jar` 로 갈아끼우기)도 ⒥(두 이름 폴백)도 **고를 필요가 없었다** — 내용 탐색이
  두 이름을 **모두** 받으면서 규약을 **하나로** 남긴다.

- **불일치 실재 증명 (Acceptance ⑴ · 파일:줄)**

  | | 위치 | 하는 일 |
  |---|---|---|
  | ours(착수 시점 `5130d15f`) | `wie_lgt/src/emulator.rs:39` | `let jar_filename = format!("{}.jar", app_info.aid);` |
  | upstream(`580da150` · 2026-09-12) | `wie-lgt/src/emulator.rs:50-53` | `files.iter().find_map(|(filename, data)| (filename.ends_with(".jar") && Self::loadable_jar(data)).then_some(filename))` |
  | upstream 테스트 | `wie-lgt/tests/test_helloworld.rs:34-35` | `archive.remove("00000000.jar")` → `archive.insert("application.jar", …)` |
  | 갈린 커밋 | upstream `9a88423b` (PR #1368 · 2026-08-23) | 위 두 변경을 **같이** 넣었다 |

  ★**이 불일치는 지금 픽스처로는 «보이지 않는다»** — `helloworld_lgt.zip`·`keydraw_lgt.zip` 둘 다
  `app_info` 의 `AID:00000000` 이고 jar 가 `00000000.jar` 라 **두 규칙이 우연히 일치**한다. 그것이
  「조용히 깨진다」의 기전이다.

- **★양방향 증명 (Acceptance ⑵ · 둘 다 «실행»했다)**

  | 트리 | `test_helloworld` | `test_helloworld_jar_named_application` |
  |---|---|---|
  | 고치기 **전**(시험만 먼저 넣음) | **ok** | ★**FAILED** — `panicked at wie_lgt/src/emulator.rs:118: called Option::unwrap() on a None value` |
  | 고친 뒤 | ok | ★**ok** |
  | 고친 뒤 **탐색 3줄만** 되돌림(개악 대조) | **ok** | ★**FAILED** — `FatalError("Missing binary.mod in 00000000.jar")` |

  ★셋째 행이 두 가지를 한꺼번에 잠근다 — ⑴상수/술어가 **한 곳**임(그 한 곳을 되돌리면 red) ⑵새 진단
  문면이 **실제로 jar 이름을 말한다**(옛 `Option::unwrap()` 은 아무것도 말하지 않았다).

- **★`00000000` 전수 계수 (Acceptance ⑸ · ⒤/⒥ 판단 근거)**: 추적 파일 중 `00000000` 문자열 보유 **10건**
  이지만 **jar 이름/AID 로서**는 ⒜픽스처 **4건**(`helloworld_lgt`·`keydraw_lgt`·`helloworld_ktf`·`keydraw_ktf`
  전부 내부에 `00000000.jar`) ⒝생성기 `scripts/make-wipi-keydraw-fixture.sh:175`(AID 인자 `00000000`) ⒞
  이번 회차가 건드린 두 파일. 나머지(`wie_core_arm`·`wie_skt`·`wie_skvm`·`wie_wipi_c`·`.dev.vars.example`)는
  16진 0·전화번호·부동소수 상수로 **무관**.
  ⇒ ★**0 이 아니다** ⇒ 티켓 규칙대로면 ⒥(폴백)이었으나, ★**규약이 둘로 남는 대가를 치르지 않고** 같은
  결과를 얻는 길(내용 탐색)이 upstream 에 이미 있었으므로 그것을 골랐다. ★**KTF 는 손대지 않았다** —
  `wie_ktf/src/emulator.rs:44` 는 여전히 `format!("{}.jar", adf.aid)` 이고 ★**upstream 도 같다**
  (`upstream/main:wie-ktf/src/emulator.rs:76`) ⇒ 거기엔 불일치가 **없다**(있는 척하지 않는다).

- **사용자 영향**: AID 와 jar 이름이 어긋난 LGT 아카이브가 **뜬다**(전에는 호스트 패닉). 기존 자산
  무영향 — `00000000.jar` 는 탐색으로도 여전히 첫 후보다. WASM export 표면 **무변경**(공개 시그니처 0 변경)
  이라 featurephone 계약도 무접촉.

- **게이트 실측**: `cargo fmt --all -- --check` ok · `cargo clippy --all -D warnings` ok ·
  `cargo clippy --target wasm32-unknown-unknown -D warnings` ok · `cargo +beta clippy --all -D warnings` ok ·
  `RUST_MIN_STACK=4194304 cargo test --all` **177 passed / 0 failed**(직전 회차 기준 133 이상 · 신규 1건 포함).
  러너: `draw_j2me.jar`·`helloworld_ktf`·`helloworld_lgt` **PASS** · `keydraw_ktf`·`keydraw_lgt`
  `--inject --expect-last-frame` **PASS · last_frame_content true · rc=0**.
  `npm run audit` PASSED · `check-worklog-json` OK · `check-doc-liveness-parity` OK ·
  `check-worklog-coverage` **9/10 = 90% · 약속 최신**(85 landed / 마지막 기록 83 ⇒ 이 회차는 기록 의무 없음).
  ★**러너 부수 관측**: `keydraw_*` 의 `paints` 가 **45** 다 — `AGENTS.md` 는 2026-09-06 실측으로 **55** 를
  적어 두었다. 판정 축(PASS · content true · rc=0)은 전부 충족이고, 그 수는 문서 자신이 적은 대로
  **부하에 따른 틱 오버런**에 흔들린다(20.0s 고정 예산 안에서 몇 프레임을 보는지). ★회귀로 읽지 마라 —
  다만 문서의 그 수가 **재측 없이 인용되면 낡는다**는 사실은 남긴다.

- **범위 밖(손대지 않았다)**: `compile_model.rs` 122줄 이식 · upstream base 오버레이 재적용 ·
  `wie_web`→`wie_featurephone` 개명(2026-09-11 착지) · 산출물 이름 `wie_web.js`/`wie_web_bg.wasm`(의도적 보존) ·
  upstream 핀 이동 · upstream PR(P4) · KTF 이름 규칙(위 참조).
