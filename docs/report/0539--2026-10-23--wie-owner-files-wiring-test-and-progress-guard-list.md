## [2026-10-23] 동봉 «소리 끔» 처리 — 통신사 마운트 배선 시험 · 진도 가드를 2행 목록으로 (wie-owner-files-wiring-test-and-progress-guard-list)

**무엇을**: #570 검수(minor 1·3)가 짚은 시험 공백 2개를 닫았다. 동작은 바뀌지 않았고, 이 회차는 그 동작을 시험으로 고정한다.
- ⑴ **배선**: `wie-backend::owner_files` 에 `WIRING_PROBE_JAR` 를 더했다. 이것은 어느 게임도 싣지 않는 합성 «jar» 바이트이고, 그 MD5 로 두 목록에 탐침 행을 하나씩 넣었다(`probe.sav` 빼기 · `probe.opt` +0 → 1). KTF·SKT 의 `emulator.rs` 에 시험 `from_archive_turns_the_owner_sound_on` 을 넣었다. 이 시험은 탐침 아카이브를 `from_archive` 로 올린 뒤 가상 파일시스템을 대조한다(`probe.sav` 없음 · `probe.opt == [1]`). `wie-skt` 에는 dev-dependency 로 `test-utils` 를 더했다.
- ⑵ **진도 가드**: `a_file_holding_progress_is_rewritten_never_dropped` 가 `(MD5, 파일)` 2행 목록을 돈다. 두 행은 `abc0c5789259` `Rms` 와 `6e93f26fa2f5` `data.sav` 다. 행마다 두 가지를 본다. 빼기 목록의 같은 MD5 행이 그 파일을 이름하지 않아야 하고, 바이트 행이 그 파일을 가리켜야 한다.

**왜**: 종전 시험이 고정한 것은 «행»과 «함수 동작»뿐이었다. 두 `from_archive` 의 호출을 지워도 green 이었다(검수 M3). 진도 가드도 `6e93f26fa2f5` 한 MD5 만 막았다. 같은 이유로 바이트 방식을 고른 `abc0c5789259` 는 빼기 목록으로 옮겨져도 잡히지 않았다.

**사용자 영향**: 없다(엔진 동작 변경 0). 탐침 행의 열쇠는 문자열 `wie owner-files wiring probe` 의 MD5 다. 이 MD5 와 같은 jar 를 가진 실제 게임은 없다.

### 개악 실측 (`cargo test -q -p wie-skt -p wie-ktf -p wie-backend`, 각 개악 뒤 `git checkout` 원복)

| 개악 | 결과 |
|---|---|
| KTF `from_archive` 의 `turn_owner_sound_on` 호출 주석 처리 | `wie-ktf --lib` FAILED 1 (52 passed) · rc 101 |
| SKT `from_archive` 의 같은 호출 주석 처리 | `wie-skt --lib` FAILED 1 (7 passed) · rc 101 |
| `abc0c5789259` `Rms` 행을 바이트 → 빼기 목록으로 옮김 | 진도 가드 + 행 고정 시험 FAILED 2 |
| `6e93f26fa2f5` `data.sav` 행을 같은 식으로 옮김 | 진도 가드 + 행 고정 시험 FAILED 2 |
| `abc0c5789259` 에 `Rms` 빼기 행만 «더함»(바이트 행 유지 · 검수 M1 형상) | 이 브랜치: 진도 가드만 FAILED 1 · `origin/main` 같은 개악: **4 passed (못 잡음)** |
| 원복 | 전부 green |

### 게이트
- `cargo fmt --check` · `clippy --all -D warnings` · wasm clippy · `+beta clippy` 모두 rc 0 이다. `RUST_MIN_STACK=4194304 cargo test --all` 은 rc 0 · **816 passed / 0 failed** 다(813 에 새 시험 3개: KTF·SKT 배선, 탐침 행 고정).
- 엔진 runner 블록 6줄은 모두 `PASS` · rc 0 이다(load1 6.2).

<!-- corpus-name-inflow v1 subjects=6 tree=eb6107b0d34506f1 B=0/0 P=0/0 S=0/0 -->
