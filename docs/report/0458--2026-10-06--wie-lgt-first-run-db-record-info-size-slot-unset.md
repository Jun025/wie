## [2026-10-06] LGT 첫 실행 «데이터 저장 실패» — 모드 8 열기는 DB 를 만든다 · 빈 DB 정보는 크기 0 항목 (wie-lgt-first-run-db-record-info-size-slot-unset)

**무엇을**: LGT `MC_dbOpenDataBase` 를 모드 8 로 열면 없는 DB 를 그 자리에서 만들게 했다(`open_database_lgt`). LGT 정보 질의(SVC `0x195` · `list_record_info`)는 이제 있는 DB 에 레코드가 없으면 크기 0 항목 1개를 쓰고 0 을 돌려준다.

**왜**: 추천 후속 작업 `2026-10-06-ktf-free-storage-16mb#p0` 채택. 0449 §3 이 원인만 재고 남긴 `63332c51d514` 다.

**사용자 영향**: `63332c51d514` 가 첫 실행에서 «데이터 저장 실패로 게임을 종료합니다» 안내 뒤 꺼지던 것이, 이제 타이틀과 메뉴로 들어간다.

### 1. 원인 (재측)

저장 로더 `0xf248`(Thumb · `.text` vaddr) 의 흐름:
1. `open(name, 8, 1)` 을 부른다.
2. 결과가 −11 · −12 · −13 · −24 · −3 · −9 · −1 중 하나면 «저장 없음»(크기 0)으로 간다.
   **0449 §3 은 −12 를 빠뜨렸다** — `adds r3, r0, #13; bgt` 범위 비교가 −12 를 «없음»으로 보낸다. 같은 7개 집합이 다른 타이틀 `6bc7f65e3022` 의 비트마스크 `0xa0b801` 에도 그대로 있다.
3. 열기가 성공하면 `0x195(name, sp, 1)` 을 부르고, **반환값을 보지 않고** `[sp+8]` 을 크기로 쓴다.
4. 우리 쪽: 모드 8 은 핸들을 주지만 DB 는 첫 쓰기 때에야 만든다. 그래서 `0x195` 는 «없음» −12 를 돌려주고 칸을 쓰지 않는다 ⇒ 크기 = 스택 찌꺼기 ⇒ 읽기 실패 ⇒ 오류 깃발 ⇒ 안내 후 종료.

### 2. 어느 계약인가 — 코퍼스 LGT 호출부 (정적 · capstone · ELF `.text`/`ER_RO`)

LGT 91개 zip 중 `0x195` 가져오기 스텁(`.data` 안 `[0x1fb][id]` 직전)을 가진 고유 타이틀은 **43종**이다. 스텁 주소를 리터럴로 싣는 `ldr` 호출부를 모두 모았다.

| 축 | 측정 |
|---|---|
| `0x195` 호출부 | 120곳 — 호출 직후 결과로 갈라지는 곳 85 · 결과를 보지 않고 버퍼를 읽는 곳 30 · 해석 못 함 5 |
| `0x190` 열기 모드(r1) | 1: 192 · **8: 100** · 4: 50 · 2: 16 · 불명 16. r2 는 전부 1 |
| 모드 8 다음 동작 | 쓰기 60 · 읽기 3 · 그 밖 37 |
| 모드 8 을 쓰는 타이틀 | 22종. 그중 **8종은 모드 1·8 로만 연다**(`0606d43702e7` `863b8ab6a21d` `6bc7f65e3022` `1eaa92092bee` `6b515884dbc1` `63a4859013be` `2520654be6de` `acc4215b7ec0`) |

판정: **«크기 0 항목» 쪽이다. «열기 실패»가 아니다.**
- 모드 1·8 로만 여는 8종은 모드 8 말고는 저장 파일을 만들 길이 없다. 그러니 실기에서 모드 8 열기는 없는 DB 를 만든다. 열기가 성공한 순간 그 DB 는 «있는 빈 파일»이다.
- `0606d43702e7 0x15e96` 도 같은 모양이다: `0x195` 결과를 버리고 → `open(8)` → 쓰기 → 닫기.
- 그러므로 «모드 8 이 없는 DB 에서 실패한다»는 계약은 그 8종의 첫 저장을 막는다. 63332 의 오류 7종 분기는 그 경로가 아니라 저장 공간 부족 같은 실패를 위한 방어로 본다.
- 열린 빈 파일에 대한 정보 질의는 성공해야 하고 크기는 0 이어야 한다. 결과로 갈라지는 85곳은 모두 «0 이면 항목을 읽는다» 꼴이다. 그런데 종전 `list_record_info` 는 **있는 빈 DB 에도 0 을 돌려주며 아무것도 쓰지 않았다** — 이 85곳 계약도 어기고 있었다.

**범위를 LGT 로 묶었다.** KTF 도 `open_database` 를 같이 쓰지만 모드 8 근거는 LGT 호출부뿐이다. 그래서 LGT 디스패치만 `open_database_lgt` 로 바꿨고, 일반 경로의 모드 8 은 지연 생성 그대로다(시험 `generic_mode_8_still_creates_lazily`).
모드 2 도 손대지 않았다. 생성 시점 근거가 없다.

### 3. 측정 (base `544a0643` ↔ head `034d61e8` · release `wie_validate`)

**대상 짝**(`--inject --keep-timeout --timeout 45 --max-ticks 100000000000`):

| | 결과 | stop | 키 | paints | 화면 |
|---|---|---|---|---|---|
| base | UNMEASURED | clean exit | 1/27 | 28 | «데이터 저장 실패로 게임을 종료합니다» 뒤 종료 |
| head | PASS | deadline | 27/27 | 449 | 안내 → 로고 → 타이틀 → 메뉴(게임 시작 · 도움말 · 환경 설정) |

**퇴행 짝**: `playability-census.mjs run --only probe`(long 풀 · 호스트 잠금 · `--jobs 3` · load1 9~40)로 49종을 쟀다. 대상 = `0x195` 를 가져오는 LGT 43종(모드 8 22종 전부 포함) + 라이브 LGT `13d7e3c21856` `b475b6399684` `4ece6eeeaa04` `1b107b96bf4e` `a30bbe008b5e` + 가드 `49ade89578c5` `ddd885583b15`.
- 축(status · boot · render · input · sound)이 바뀐 것은 `63332c51d514` **1종**이다: input none → ok · sound silent → ok.
- A/B 결과 · stop · content · Java 예외 수 98쌍 중 다른 것도 그 1종의 A 뿐이다.

compat.json 은 이 1행만 바꿨다: input·sound `ok` 이고, 두 knownIssue(키 · 소리)를 지웠다. status 는 `limited` 그대로다 — longplay·speed 를 재지 않았다.

### 4. 되돌리면 red

모드 8 생성 제거 · 빈 DB 의 크기 칸 미기입. 둘 다 `lgt_write_mode_8_creates_the_database_and_its_info_is_one_empty_entry` 가 FAILED 이고, 원상 green 이다.

### 검증

`cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all`(708 passed · 0 failed) · `npm run build:wasm` · `check-engine-contract` · `npm run audit` 전부 rc=0.
러너 줄(head release): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 기본 틱 상한에서 `UNMEASURED · max-ticks` 였고, `--max-ticks 2000000000` 로 head·base 모두 PASS · rc=0(paints 79/80 · 55/55)이었다.
증적(repo 밖): `~/orchestrator/reports/evidence/wie-lgt-first-run-db-record-info-size-slot-unset/`(짝 스크린샷 · 호출부 창 · census.tsv 두 벌).
게임 파일명 유입: 이 회차가 **더한 줄**에는 0건이다. 도구 기본 실행은 BOUNDED 336쌍 · SUFFIX-ATTACHED 15쌍이다. 그중 compat.json(계약상 제목 목록) 밖에서 걸린 것은 `wie-lgt/src/runtime/wipi_c.rs` 11회 · 6쌍인데, 이 회차가 그 파일에서 바꾼 것은 디스패치 한 줄(`open_database` → `open_database_lgt`)뿐이다. 즉 이미 있던 주석이다. compat.json 을 뺀 실행의 SUFFIX-ATTACHED 는 0 이다.

<!-- corpus-name-inflow v1 subjects=6 tree=ce9483579efbee9c B=729/336 P=0/0 S=35/15 -->
