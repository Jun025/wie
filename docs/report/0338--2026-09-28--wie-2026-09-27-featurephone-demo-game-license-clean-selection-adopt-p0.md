## [2026-09-28] 클래스 상수의 NUL(C0 80)·보조평면 문자 — modified UTF-8 디코딩 (wie-2026-09-27-featurephone-demo-game-license-clean-selection-adopt-p0)

**무엇을**: crates.io `classfile` 0.1.1 을 `patches/classfile/` 에 옮겨 두고 루트 `[patch.crates-io]` 로 갈아 끼웠다.
바꾼 것은 `src/constant_pool.rs` 의 `parse_utf8` 하나다 — `String::from_utf8` 대신 `decode_modified_utf8`(JVMS §4.4.7).
나머지 소스는 tarball 과 바이트 동일(`diff -r` 로 확인 · 그 함수·import·시험만 다르다).

**왜**: 자바 클래스 파일은 문자열 상수를 modified UTF-8 로 적는다 — NUL 은 `C0 80`, 보조평면 문자는 서로게이트 쌍을
3바이트씩(6바이트). 표준 UTF-8 파서는 둘 다 거부하므로 `"\0"` 이나 이모지를 상수로 가진 클래스가 부팅 단계에서
`ClassFormatError: Invalid class file` 로 죽는다. Pebble Snake 첫 빌드가 이걸 밟았고(`0327`) 데모는 태그 바이트를 따로 써서 우회했다.
upstream `dlunch/RustJava` main(`a8bc80e2`, 2026-09-28)도 아직 `String::from_utf8` 이다 — upstream PR 은 이 repo 가 할 일이 아니라 후속으로 남긴다.

**사용자 영향**: 문자열에 `\0` 이나 보조평면 문자가 든 J2ME/WIPI 게임이 클래스를 불러오다 죽지 않는다.

### 디코더가 받는 것
- 표준 UTF-8 로 유효한 입력은 **종전 경로 그대로**(빠른 길). 느린 길만 새로 생겼다.
- 느린 길은 1·2·3바이트 형(`C0 80`·서로게이트 반쪽 포함)에 더해 **표준 4바이트 형도 받는다** — javac 는 안 내지만 종전 파서는 받았으므로 막으면 퇴행이다.
- 짝 없는 서로게이트(자바 문자열로는 합법, Rust `String` 으로는 불가)는 **U+FFFD** 로 바꾼다. 클래스 전체를 거부하는 것보다 낫다고 봤고, 이것만 손실이 있다.
- 잘린 시퀀스·떠돌이 연속 바이트·`F5..FF` 선두 바이트는 여전히 거부.

### 실측
| 무엇 | 결과 |
|---|---|
| 새 단위 시험 4종(C0 80 · 서로게이트 쌍 · ASCII/한글/빈 문자열 · 경계) | `cargo test -p classfile` 14 passed(upstream 기존 단위 시험 10 포함) |
| 변이: `from_utf8` 복원 | **3 FAILED**(`nul_is_c0_80`·`supplementary_is_a_surrogate_pair`·`edges_of_the_wider_decoder`) · ASCII/한글 대조군은 green 유지 |
| Pebble Snake 변형 jar(`"MTR\0"` 리터럴 + `"🐍"` 상수, 스크래치 빌드·커밋 안 함) — **전**(patch 끔 · Cargo.lock origin/main 과 동일) | `FAIL` · `java.lang.ClassFormatError: Invalid class file` at `SnakeCanvas.<init>` · paints 0 · rc=1 |
| 같은 jar — **후** | `PASS` · 입력 10/10 · paints 71–73 · java_exceptions 0 · audio plays 7 · midi_events 31 · rc=0 |
| 출하 jar(`build.sh` sha256 `5ed67caa…05c68` 동일) — 후 | `PASS` · paints 72 · audio plays 7 · midi_events 31 |
| 네 게이트 + `cargo +beta clippy --all -- -D warnings` | 전부 rc=0 |
| `npm run build:wasm` · `check-engine-contract.mjs` · `npm run audit` | rc=0 · 109 pass / 0 violation · AUDIT PASSED |

변형 jar 의 `SNAKE.length() != 2` 검사는 서로게이트 쌍이 자바 문자열에서 두 char 로 되돌아오는지를 본다 — 예외 0 이므로 왕복이 맞다.
출하 데모는 건드리지 않았다(README 의 sha256 이 그 jar 를 가리킨다). 데모를 `"MTR\0"` 리터럴로 되돌리는 것은 선택 사항으로 남긴다.

### 한계
- 느린 길의 속도는 재지 않았다 — 표준 UTF-8 이 아닌 상수에서만 탄다.
- `patches/classfile` 은 upstream 새 판이 나오면 떼야 할 짐이다. 떼는 조건은 `patches/classfile/README.md` 에 적었다.
