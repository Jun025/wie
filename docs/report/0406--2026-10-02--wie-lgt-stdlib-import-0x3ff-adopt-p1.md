## [2026-10-02] LGT stdlib import 0x3ff = strtol — acc4215b7ec0 의 도움말 화면 벽 (wie-lgt-stdlib-import-0x3ff-adopt-p1)

**무엇을**: `wie-lgt` stdlib 표에 **0x3ff = strtol** 을 넣었다(`StdlibSvcId::Strtol` · `strtol`/`parse_long`). 키 경로 `docs/keys/lgt-acc4215b7ec0-help.keys` 를 더했다.
**왜**: 채택 제안 `2026-10-01-lgt-annunciator-height-296-decide#p1` — 0405 §3 에서 `acc4215b7ec0` 가 `Unknown lgt stdlib import: 0x3ff` 로 2/2 FAIL.
증적: `~/orchestrator/reports/evidence/wie-lgt-stdlib-import-0x3ff-adopt-p1/`. 타이틀은 sha12 로만 적는다.
**사용자 영향**: 그 게임에서 메인 메뉴 → 도움말/기타 → 게임설명을 열면 멈추던 것이 열린다. 설명 글의 색 글씨(주황)도 그 값으로 칠해진다.

### 1. 호출부 — 의미를 정한 근거

`binary.mod` 의 `.data` 에서 0x3ff 썽크는 `0x140acb4` 하나다(`push {lr}; bl 해석기; .word 1, 0x3ff` · 앞뒤가 atoi 0x3fb · strcpy 0x405). `.text` 에서 그 주소를 가진 리터럴은 `0x363b0` 하나이고, 읽는 곳도 하나다:

- `0x36172`–`0x3618c`: 글 `tbl->strings[i]` 에서 `|` 와 `|` 사이 바이트를 `sp+0xb8` 로 복사하고 NUL 로 닫는다.
- `0x3618e`–`0x36194`: **`f(sp+0xb8, sp+0x44, 0x10)`** — 글 · 끝 위치를 받을 칸 · 16.
- `0x36198`–`0x361a0`: 반환값을 `sprintk(sp+0xac, "%d", v)` 의 인자로 쓴다(서식 `0x5a3e0` = `"%d"` · `0x363b8` = WIPIC 썽크 `0x1fb, 0x65`).

jar 의 글 자원 3개(`0.BAR` `1.BAR` `7.BAR`)에서 그 칸은 `|0xFFAE08|` `|0xFFFFFF|` 꼴이다 — 색 값. 세 인자 모양(글 · `char**` · 진법)과 `0x` 머리를 받아들이는 16진 해석은 **strtol** 이다.
strtoul 과는 이 타이틀이 넘기는 값(전부 `≤ 0xFFFFFF`)에서 결과가 같아 갈리지 않는다 — `0x7fffffff` 를 넘는 값을 넘기는 호출부가 나오면 다시 정한다(`svc_ids.rs` 주석).

### 2. 결정적 재현 — 키 경로

0405 의 FAIL 은 기본 27키가 «느릴 때만» 닿는 자리였다(이 회차 기본 27키 재측 main 1/3 · 1/3 · 0/6 FAIL — 실패 회차는 그리기 수가 180~183 으로 통과 회차 350~500 보다 적다). 느리면 앞쪽 `UP` 이 메인 메뉴 커서를 옮겨 놓고, 12·14번째 `OK` 가 **도움말/기타**로 들어간다.
메인 메뉴에서 `UP` 하나가 연습모드 → 도움말/기타로 감아 돈다. 처음 시도(메뉴 뒤 8초 · 기본 누름 0.15초)는 `UP` 이 회차마다 먹히기도 하고 빠지기도 해서 같은 경로가 부하에 따라 0/3 · 3/3 으로 갈렸다(`UP UP` 은 첫 `UP` 이 빠질 때만 맞는다).
메뉴가 다 들어오도록 15초 기다리고 키를 0.5초 누르니 갈림이 사라졌다 — 그것을 `docs/keys/lgt-acc4215b7ec0-help.keys` 로 남겼다(6번째 `OK` = 게임설명).

### 3. 전/후

같은 키 경로 · release `wie_validate --inject --keys <경로> --max-ticks 1e11` · 동시 3개. 전 = `origin/main` `5cbd635e` · 후 = 이 브랜치.

시작·끝 `host-load-guard --status --recovered` rc=0(`hl-start.txt`·`hl-end.txt` · load 24).

| | 판정 | 키 | strtol 호출 |
|---|---|---|---|
| 전 ×3 | **FAIL ×3** · `06_OK` · `Unknown lgt stdlib import: 0x3ff` | 5/6 | 0 |
| 후 ×3 | **PASS ×3** | 6/6 | 830 · 827 · 830 |

- 후의 strtol 호출 수는 `RUST_LOG=wie_lgt::runtime::stdlib=debug` 로 셌다(전부 `(…, …, 16)`) — 통과가 «안 닿아서»가 아니다.
- 후 화면: 게임설명 1/7 쪽이 열리고 강조 낱말이 주황(`0xFFAE08`)으로 칠해진다.

### 4. 짝 재측 — 같은 import 를 가진 다른 LGT

`game_lab` LGT 78종(sha 중복 제외)의 `binary.mod` 를 걸었다. `(1, 0x3ff)` 바이트 쌍은 76종에 있지만 **썽크로 정렬된 자리(데이터 구간)는 둘**뿐이다 — 나머지는 우연한 바이트 일치다.
- `acc4215b7ec0` — §1.
- `393d359d0815`(ARM 링커 배치 · `ER_RW` `0x14005f0`) — 0x3fe · 0x3ff · 0x400 썽크가 나란히 있으나 **어느 썽크 주소도 코드·데이터 어디에도 없다**(4바이트 리터럴 · Thumb `bl`/`blx` 대상 전수 0). 묶여 들어온 죽은 썽크로 본다. 기본 27키 60초 짝(`--pacing 8 --relaunch 1`): 조용한 호스트(guard 시작·끝 rc=0)에서 main 3/3 · 후 3/3 PASS · 27/27키 · 그리기 수 897~899 로 같다. 부하 중 한 판(guard rc=1 · 참고만)도 main 3/3 · 후 3/3 PASS.
⇒ 0x3ff 의 의미를 다르게 쓰는 타이틀은 이 코퍼스에 없다. 다른 경로는 0x3ff 에 닿지 않으므로(구성상) 동작이 같다.

### 5. 되돌리면 red

- ⒜ `StdlibSvcId::Strtol` 디스패치 줄 삭제 → `stdlib_import_0x3ff_is_strtol_through_the_svc_table` `Unknown lgt stdlib import: 0x3ff`
- ⒝ `0x` 머리 건너뛰기 삭제 → `parse_long_follows_c_strtol` `(0, 1) ≠ (16756232, 8)` · 위 시험도 함께 red
- 게임 층 변이 = §3 의 «전» 열.

### 6. 게이트

이 worktree 의 target · `build-slot` 경유.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` · `cargo +beta clippy --all -- -D warnings` · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` 전부 rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 51 묶음 · 640 통과 · 실패 0
- 러너 블록(`cargo run`): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt` `--inject --expect-last-frame` PASS rc=0(paints 79 / 55) · `text_j2me --timeout 5` PASS
- `check-worklog-json` · `player-data` · `check-docs-report-serial` OK

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus <game_lab>`: BOUNDED 6회/6쌍 · SUFFIX-ATTACHED 0회/0쌍. BOUNDED 6회는 전부 이 회차가 손댄 `wie-lgt` `stdlib.rs`·`svc_ids.rs` 의 **기존 줄**이다 — `git diff origin/main` 의 `+` 줄에는 그 여섯 이름이 0회다. 타이틀은 sha12 로만 적었고, 키 경로 파일에는 화면 이름(메뉴 글자)만 있다.
