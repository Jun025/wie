# docs/report/0106

## [2026-09-13] LGT 엔트리포인트 jar 가 `P/` 아래 있으면 클래스패스만 접두를 안 뗐다 (wie-lgt-entrypoint-jar-under-p-prefix-never-opens)

### 무엇을
`LgtEmulator::load` 는 아카이브의 모든 파일을 가상 파일시스템에 넣을 때
`trim_start_matches("P/")` 로 접두를 떼는데, JVM 클래스패스에 넘기는 `jar_filename` 은
떼지 않았다. 탐색(0104 회차의 내용 탐색)이 찾은 이름이 `P/foo.jar` 면 파일시스템에는
`foo.jar` 만 있고 클래스패스는 `P/foo.jar` 를 가리켜 **부팅이 즉시 실패**한다.
처방 = `load()` 안에서 `jar_filename` 에도 **같은 술어** 한 번:
`jar_filename.trim_start_matches("P/").to_owned()`. 두 호출자(`from_archive`·`from_jar`)가
전부 이 지점을 지나므로 정규화는 한 곳이다.

### 실증 (티켓 대전제 ⓐ — 실행으로)
- **수정 전 red**: `helloworld_lgt.zip` 픽스처의 `00000000.jar` 를 `P/00000000.jar` 로 옮겨
  `from_archive` 에 넘기면
  `FatalError("Missing binary.mod in P/00000000.jar")` — 0104 회차가 심은 오류문이 정확히
  이 어긋남을 이름으로 보여 준다.
- **수정 후 green**: 같은 아카이브가 `Hello, world!` 까지 완주.
- 시험 = `wie_lgt/tests/test_helloworld.rs::test_helloworld_jar_under_p_prefix` —
  바로 옆 `test_helloworld_jar_named_application` 과 같은 관용(픽스처 키 개명)이고 새 틀 없음.

### 대전제 ⓑ·ⓒ
- ⓑ 갈리는 두 지점은 **같은 함수 `load()` 안**이다(파일시스템 키 :99 · 클래스패스 이름 :110) —
  중간 정규화 없음.
- ⓒ 접두는 관측된 `P/` 하나만 닫았다(티켓이 일반화를 금지). 술어도 upstream 이 이미 쓰는
  그 `trim_start_matches("P/")` 를 재사용했다.

### upstream 대가 (Contract 2)
- 갈린 지점에 주석 4줄: 왜 갈랐는가 · upstream 대응 위치(`wie-lgt/src/emulator.rs:118-120`) ·
  「재정렬 시 이 헝크를 보존하라」.
- **올릴 수 있는 형태다**: `upstream/main:wie-lgt/src/emulator.rs` 의 `load()` 에 동일한
  `let jar_filename = jar_filename.to_owned();` 줄이 같은 문맥으로 실재함을 확인 — 헝크가
  그대로 얹힌다. upstream PR 발행 자체는 범위 밖(운영자 결정).

### 검증
- 4게이트: `cargo fmt --all -- --check` ok · `cargo clippy --all -- -D warnings` ok ·
  `--target wasm32-unknown-unknown` ok · `RUST_MIN_STACK=4194304 cargo test --all`
  **178 passed / 0 failed**(직전 177 + 신규 1). `cargo +beta clippy --all -- -D warnings` ok.
- 러너: `draw_j2me.jar`·`helloworld_ktf.zip`·`helloworld_lgt.zip` 전건 `"result":"PASS"` ·
  `keydraw_ktf.zip`·`keydraw_lgt.zip` `--inject --expect-last-frame` 둘 다
  `"result":"PASS"` · `last_frame_content:true` · **rc=0(파이프 없이 측정)**.

### 한계
- 이 형태(`P/` 아래 엔트리포인트)의 **실아카이브는 여전히 본 적이 없다** — 제안 원문의 그
  단서 그대로다. 이번 회차는 코드가 스스로 모순이던 것(같은 함수가 같은 이름을 두 규약으로
  쓴다)을 실행 실증 위에서 닫은 것이다.
