## [2026-09-24] `org.kwis.msp.io.File` 이 `FileNotFoundException` 을 덮지 않는다 (wie-2026-09-23-aot-java-dataoutputstream-and-gregoriancalendar-anchors-adopt-p1)

### 무엇을

`init_with_flag` 의 `RandomAccessFile` 실패 경로가 예외 종류를 보지 않고 전부
`IOException("Invalid filename")` 으로 덮던 것을 **원인 그대로 전파**(`.await?`)로 바꿨다.
`// TODO check exception type` 이 없어졌다. 새 테스트
`test_missing_file_keeps_file_not_found_exception_type` 가 그 종류를 잠근다.

### 왜 — 규격이 먼저였고, 규격이 답했다

티켓이 요구한 반증 두 축을 착수 전에 쟀다.

**⒜ 덮어쓰기는 현행 체크아웃에 살아 있었다** — `wie-wipi-java/src/classes/org/kwis/msp/io/file.rs:140-143`
(`origin/main` `7e40b2f4` 기준):

```rust
if raf.is_err() {
    // TODO check exception type
    return Err(jvm.exception("java/io/IOException", "Invalid filename").await);
}
```

**⒝ 규격 — 제안이 「확인이 먼저다」라고 적은 tradeoff 는 이 경우 «존재하지 않는다».**
권위 API(`docs/reference/AromaWIPI_javadoc.zip` → `org/kwis/msp/io/File.html`, cp949)의 생성자 선언:

```
public File(String filename, int mode) throws IOException
  Throws: IOException - 파일을 열 수 없을 경우
public File(String filename, int mode, int flag) throws IOException, SecurityException
```

그리고 이 트리가 쓰는 JVM 에서 `java/io/FileNotFoundException` 의 `parent_class` 는
**`java/io/IOException`** 이다(RustJava 핀 `5b84dd1` ·
`java_runtime/src/classes/java/io/file_not_found_exception.rs:15`).
⇒ ★**하위형이므로 선언된 `throws IOException` 을 만족하고, `catch (IOException)` 게스트는 그대로 잡는다.**
「종류를 통과시키면 지금 IOException 만 잡던 게스트가 못 잡는다」는 우려는 JVM 의 catch 의미론상
**성립하지 않는다**. 같은 판정을 이 repo 가 이미 한 번 내렸다 — `docs/report/0060` 의 개악 M1
(`IOException`→`FileNotFoundException`)이 **green** 이었고, 그 회차는 그것을 「시험의 구멍이 아니라
JVM 의미론」으로 기록했다.

### 사용자 영향 — ★**측정했고, 이 타이틀의 벽은 «움직이지 않았다». 흐리지 않는다.**

제안이 약속한 편익은 「첫 실행에 세이브가 만들어지지 않는 타이틀이 정상적으로 파일을 생성하게 된다」였다.
서든어택포켓 2파일을 **짝지은 before/after**(같은 probe 를 심은 두 release 바이너리 · 같은 분 · `--timeout 60`)로 쟀다:

| 축 | before (재라벨 존치) | after (이 변경) |
|---|---|---|
| `File::<init>` 호출 | `(…, mode=1, flag=0)` **1회** | 동일 **1회** |
| 원인 예외(probe) | `java/io/FileNotFoundException` · `File not found` | 동일 |
| ★게스트가 «받는» 예외 | `java/io/IOException` · **`Invalid filename`** | ★`java/io/FileNotFoundException` · **`File not found`** |
| 쓰기 모드 재오픈 | **0** | **0** |
| 세이브 파일 생성 | **없음** | **없음** |
| 벽 | `FAIL` · `no frame rendered (hang/black screen)` · `stop=deadline` · `paints 0` | **동일** |

⇒ ★**고쳐진 것은 「게스트에게 전달되는 사실」이고, 「게스트의 분기」는 바뀌지 않았다.**
이 타이틀은 그 예외를 받은 뒤 쓰기 모드로 다시 열지 않는다 — `mode=1`(READ_ONLY) 한 번이 전부다.
⇒ **제안의 편익 문장은 이 타이틀에서 «관측되지 않았다».** 세이브 미생성의 근인은 다른 곳에 있다
(이 두 파일은 여전히 `paints 0` 으로 데드라인에 닿는다 = 저장 로직 앞에서 막혀 있다).
남는 값은 **규격 준수**와 **다음 라운드가 같은 곳에서 오진하지 않는 것**이다.

probe 는 일회용이고 최종 diff 에 **0줄** 남았다(`grep -c PROBE` = 0).

### 검증

- ★**양방향 변이**: 재라벨을 되살리면 새 테스트가 **red**
  (`assertion failed: jvm.is_instance(&*exception, "java/io/FileNotFoundException")` · 3중 1 failed —
  기존 2건은 green 이라 단언이 그 축만 잡는다), 원상태 **green**(3 passed).
- 네 게이트: `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 ·
  `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 ·
  `cargo +beta clippy --all -- -D warnings` rc=0 ·
  `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · **421 passed / 0 failed**.
- `cargo clippy --workspace --all-targets` rc=0 · 경고 16(전건 기존 · 다른 크레이트의 test 타깃) ·
  ★**변경 파일 귀속 경고 0**(`grep -c 'kwis/msp/io/file.rs'` = 0) ⇒ 증가 0.
- AGENTS.md 러너 블록 6종: `draw_j2me` PASS(paints 1) · `helloworld_ktf`/`helloworld_lgt` PASS ·
  `keydraw_lgt` **PASS**(paints 55 · `last_frame_content` true) · `text_j2me` PASS(paints 1).
  ★`keydraw_ktf` 는 **첫 회 FAIL**(paints 23) 뒤 **재실행 3/3 PASS**(rc=0 · paints 29/46/35 · lfc true).
  loadavg **98~105** 였고 idle 구간(48–55)보다 낮은 count 에서의 FAIL 이므로 AGENTS.md 가 이름 붙인
  **부하 기아**다(⑴재실행 ⑵count 대조 ⑷load 로 해소 · ⑶untouched tree 재현은 돌리지 않았다 —
  ⑴이 이미 갈랐고 이 변경은 `keydraw_*` 가 건드리지 않는 파일시스템 경로다).

### 게임 파일명 유입 — 도구를 실행해서 적었다

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab`(이 워크트리에는
`game_lab/` 이 없어 `--corpus` 로 코퍼스를 가리켰다) · subjects 3:
★**BOUNDED 10회/6쌍** · PREFIX-EMBEDDED 0 · ★**SUFFIX-ATTACHED 7회/3쌍**.
**0건이 아니다** — 이 회차는 라이브 타이틀 2파일을 측정한 회차라 그 파일명이 본문에 «의도적으로» 들어 있다
(`서든어택포켓` · `lgt 서든어택 포켓`).
SUFFIX-ATTACHED 7건은 전부 **`서든어택`**(working/ktf 의 별개 타이틀)이 `서든어택포켓` 의 접두로 걸린 것이고,
조사가 붙은 언급이 아니라 ★**«더 긴 다른 제목»** 쪽이다 — 도구 주석이 이름 붙인 그 두 갈래 중 후자.

### 범위 밖으로 둔 것 — 같은 함수에 규격 불일치가 «하나 더» 있다

같은 javadoc 이 「위 모드외에 다른 모드로 열고자 하면 **IllegalArgumentException**을 발생킵니다」라고
적는데, 이 파일은 `IOException("Invalid mode")` 를 던진다(`file.rs:124`).
**고치지 않았다** — 티켓 범위 밖이고, 그쪽은 이 건과 달리 **상하위형 관계가 없어**
지금 `catch (IOException)` 으로 잡던 게스트가 **못 잡게 된다**(진짜 tradeoff 다).
후속 제안으로 올렸다.

<!-- corpus-name-inflow v1 subjects=3 tree=1ca4f0620b875534 B=10/6 P=0/0 S=7/3 -->
