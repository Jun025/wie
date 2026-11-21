## [2026-11-22] 한 판에만 걸리는 «실행 중 조정» — 장부 한 장 · 엔진 표와의 양방향 가드 · 지원 목록의 `runtimeAdjustments` 칸 (wie-featurephone-runtime-adjustment-manifest-guard-and-compat-field)

**무엇을** — 세 가지다. 조정 행의 추가 · 삭제 · 동작 변경은 없다.
- 장부 `docs/contracts/runtime-adjustments.tsv`: 엔진 표 11개의 행 전부를 (키 × 꾸러미) 한 줄씩 54줄로 모았다. 줄마다 표 · 키 · 꾸러미 sha12 · 통신사 · 제목 · 종류 · 무엇을 바꾸나 · 언제 걸리나 · 분류 · 도입 근거.
- 가드: 표마다 그 crate 의 `cargo test` 가 «컴파일된 표의 키 집합 = 장부의 키 집합»을 양방향으로 본다(`test-utils/src/ledger.rs` 의 `assert_ledger_lists` · 시험 9개). 도입 근거 칸이 비어도 red 다.
- 공개 칸: `docs/player-data/compat.json` 의 49행에 `runtimeAdjustments`(종류 배열)를 실었다. `scripts/player-data.mjs` 가 장부에서 만들고, 커밋된 값이 장부와 다르면 `contract` 잡이 red 다.

함께 고친 것: `CLAUDE.md` «게임 파일 무수정 원칙» 문단 · `binary_patches.toml` 머리 주석(디스크 0 · 메모리 코드 자리 수) · `owner_files.rs` 주석의 「지크」 둘째 꾸러미 · 공개 데이터 계약 §1.

**왜** — 운영자 지시(2026-11-16)의 기준은 「이용자가 가진 원본이 그대로 도는가」다. 전수 조사와 독립 검수는 디스크의 게임 파일을 고치는 경로를 못 찾았고, 적재 뒤 메모리 안에서 한 판에만 걸리는 장치 8종이 지원 목록 49행에 걸린다는 것을 확인했다. 처분 결정(2026-11-20)은 장치를 유지하되 숨어 있던 것을 드러내고 조용히 늘지 못하게 막는 것이었다. 종전에는 이 행들이 소스 6개 crate 의 표 11개에 흩어져 있었고, 한 꾸러미(`704839f1a6c8`)는 주석에도 없었고, 지원 목록에는 표식 칸이 없었다.

**사용자 영향** — 지금 화면에서 달라지는 것은 없다. 셸은 모르는 칸을 그대로 통과시킨다(§5). 셸이 이 칸을 그리면(otterpebble 쪽 티켓) 이용자는 어느 게임이 「잠금을 풀어서」「이전 주인의 파일 일부를 올리지 않고」「서버 답을 에뮬레이터 안에서 만들어서」 도는지 볼 수 있다.

### 1. 표별 수 — 엔진 표와 장부

엔진 표의 수는 `origin/main` `e06a35abc` 의 소스를 읽어 셌고, 키가 어느 꾸러미에 걸리는지는 코퍼스 꾸러미 안쪽의 MD5 · AID 와 대조했다(스크래치 스크립트 · 출력은 해시와 수뿐).

| 장부 `device` | 엔진 표 | 엔진 행 | 엔진 키 | 장부 키 | 장부 줄(꾸러미) |
|---|---|---|---|---|---|
| `binary-patches` | `wie-core-arm/data/binary_patches.toml` 의 `hash =` 항목 | 2 | 2 | 2 | 3 |
| `class-patches` | `wie-jvm-support/src/class_patches.rs` `PATCHES` | 5 | 5 | 5 | 5 |
| `owner-sound-off-settings` | `wie-backend/src/owner_files.rs` `OWNER_SOUND_OFF_SETTINGS` | 17 | 17 | 17 | 18 |
| `owner-sound-off-bytes` | 같은 파일 `OWNER_SOUND_OFF_BYTES` | 7 | 7 | 7 | 7 |
| `owner-sound-off-xor` | 같은 파일 `OWNER_SOUND_OFF_XOR` | 1 | 1 | 1 | 1 |
| `orphan-entries` | `wie-lgt/src/emulator.rs` `ORPHAN_ENTRIES` | 1 | 1 | 1 | 1 |
| `display-overrides` | `wie-skt/src/emulator.rs` `DISPLAY_OVERRIDES` | 1 | 1 | 1 | 1 |
| `device-bound-saves` | `wie-ktf/src/emulator.rs` `DEVICE_BOUND_SAVES` | 1 | 1 | 1 | 1 |
| `owner-progress-saves` | 같은 파일 `OWNER_PROGRESS_SAVES` | 11 | 11 | 11 | 11 |
| `fake-servers` | `wie-backend/src/fake_server.rs` `SERVERS` | 7 | 5 | 5 | 5 |
| `ktf-local-servers` | `wie-ktf/src/runtime/wipi_c/local_net.rs` `SERVERS` | 1 | 1 | 1 | 1 |
| 합 | | 54 | 52 | 52 | 54 |

- 엔진 행과 장부 줄이 다른 세 곳은 전부 설명된다. `binary-patches` 는 `client.bin` 하나를 두 꾸러미(`974e0df9ab1e` · `f770b15f8876`)가 함께 쓴다. `owner-sound-off-settings` 는 jar 하나를 두 꾸러미(`8bfd08fe4370` · `704839f1a6c8` 「지크」)가 함께 쓴다. `fake-servers` 는 표가 (AID, 호스트, 포트)마다 한 행이라 AID 5개에 7행이다.
- `owner_files.rs` 세 표의 시험용 `WIRING_PROBE_JAR` 행은 제외했다(게임이 아니다).
- 티켓은 「9개 표」로 세었다. `owner_files.rs` 의 세 표를 하나로 묶은 수이고, 장부와 가드는 셋을 따로 본다.
- 서로 다른 꾸러미는 49개다(KTF 30 · SKT 12 · LGT 7). 54줄 가운데 5줄은 한 꾸러미가 두 표에 걸린 것이다: `3185174d2121` · `5e53e490c6f1` · `6a885f89343c`(소리 설정 + 진도 저장) · `c361632541a7`(소리 설정 + 화면 크기) · `01f05f8231f4`(잠금 해제 + 소리 XOR).

가드 통과 출력(`RUST_MIN_STACK=4194304 cargo test --all` · 54묶음 · 통과 847 · 실패 0 가운데 해당 줄):

```
test ledger::tests::the_ledger_names_only_tables_the_guard_reads ... ok
test fake_server::tests::the_ledger_lists_exactly_this_table ... ok
test owner_files::tests::the_ledger_lists_exactly_these_tables ... ok
test binary_patches::tests::the_ledger_lists_exactly_this_table ... ok
test class_patches::tests::the_ledger_lists_exactly_this_table ... ok
test emulator::tests::the_ledger_lists_exactly_these_tables ... ok
test runtime::wipi_c::local_net::tests::the_ledger_lists_exactly_this_table ... ok
test emulator::tests::the_ledger_lists_exactly_this_table ... ok
test emulator::tests::the_ledger_lists_exactly_this_table ... ok
```

### 2. 개악 대조 — 넷 다 red 를 보고 원복했다

⑴ 엔진 표에 가짜 행 1개(`wie-ktf` `DEVICE_BOUND_SAVES` 에 `([0xaa; 16], &["fake.sav"])`):

```
$ cargo test -p wie-ktf --lib ledger
test emulator::tests::the_ledger_lists_exactly_these_tables ... FAILED
device-bound-saves: engine rows the ledger does not list ["aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"], ledger rows the engine does not have [] — docs/contracts/runtime-adjustments.tsv (CLAUDE.md «게임 파일 무수정 원칙»)
test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 54 filtered out; finished in 0.00s
```

⑵ 장부에서 실재 행 1개 삭제(`class-patches` · `44b6356d13f8`):

```
$ cargo test -p wie-jvm-support --lib ledger
test class_patches::tests::the_ledger_lists_exactly_this_table ... FAILED
class-patches: engine rows the ledger does not list ["1556bd6fd47112c01ce35f6495b79256"], ledger rows the engine does not have [] — docs/contracts/runtime-adjustments.tsv (CLAUDE.md «게임 파일 무수정 원칙»)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.00s
$ node scripts/player-data.mjs
player-data: 1 violations (docs/contracts/featurephone-public-data.md):
  compat.entries[393]: differs from what retitle() derives (title/game/variant/sameTitle, order, runtimeAdjustments from the ledger) — `node scripts/player-data.mjs normalize`
```

⑵-b 키를 두 꾸러미가 나눠 쓰는 줄 삭제(`owner-sound-off-settings` · `704839f1a6c8`). 키는 `8bfd08fe4370` 줄에 남으므로 엔진 가드는 통과하고, 공개 데이터 검사가 잡는다:

```
$ cargo test -p wie-backend --lib ledger
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 125 filtered out; finished in 0.00s
$ node scripts/player-data.mjs
player-data: 1 violations (docs/contracts/featurephone-public-data.md):
  compat.entries[198]: differs from what retitle() derives (title/game/variant/sameTitle, order, runtimeAdjustments from the ledger) — `node scripts/player-data.mjs normalize`
```

⑶ 도입 근거 칸 비움(`orphan-entries` · `a30bbe008b5e`):

```
$ cargo test -p wie-lgt --lib ledger
test emulator::tests::the_ledger_lists_exactly_this_table ... FAILED
orphan-entries 06de2f8c4cf632e0784f9503b5b42023: a ledger row must say where it came from (`basis`) — docs/contracts/runtime-adjustments.tsv (CLAUDE.md «게임 파일 무수정 원칙»)
$ node scripts/player-data.mjs
  ledger line 84 (orphan-entries a30bbe008b5e): empty basis — a row must say where it came from
```

### 3. `compat.json` — 칸 추가 외 변화 0

`origin/main` 의 파일과 행 단위로 대조했다(행에서 `runtimeAdjustments` 를 뺀 나머지를 JSON 으로 비교).

| | 값 |
|---|---|
| 행 수 | 429 → 429 |
| 행 순서가 달라진 곳 | 0 |
| `runtimeAdjustments` 외 칸이 달라진 행 | 0 |
| `runtimeAdjustments` 가 실린 행 | **49**(KTF 30 · SKT 12 · LGT 7 · 전부 `playable`) |

49는 조사 · 검수의 수와 같다. 종류별: `owner-file-skip` 26 · `lock-lift` 8 · `owner-file-bytes` 8 · `local-server` 6 · `device-bound-save` 1 · `orphan-flag` 1 · `display-size` 1(두 종류가 실린 행이 2개라 합은 51). 조합별: `owner-file-skip` 25 · `lock-lift` 7 · `owner-file-bytes` 7 · `local-server` 6 · `device-bound-save` 1 · `orphan-flag` 1 · `lock-lift`+`owner-file-bytes` 1 · `owner-file-skip`+`display-size` 1.

종류 이름은 티켓의 기본 7개를 그대로 썼다. 표 11개가 7종류로 접힌다: 잠금 해제 두 표 → `lock-lift` · 소리 설정과 진도 저장 미장착 → `owner-file-skip` · 소리 바이트와 XOR → `owner-file-bytes` · 가짜 서버 두 표 → `local-server`.

### 4. 「고쳐 쓰는 곳」의 수 — 문구 정정

- 저장소 안 상시 문서에는 「게임 파일 바이트 0」으로만 읽히는 문장이 없었다(`docs/report` 밖 검색 0건 · 그 문구는 발권 줄의 불변 조건이었다). 잠금 해제 회차 문서(0540 · 0546 · 0547)는 「바이트 패치 N건을 넣었다」고 사실대로 적고 있어 고치지 않았다.
- 고친 자리는 `binary_patches.toml` 머리 주석이다. 이제 「디스크 0 · 메모리에 올린 코드」를 수와 함께 적는다: 해시 키 패치 표 5행(바이너리 2개 · 3판에 8곳) · 해시 없는 범용 훅 패턴 6종(KTF 에서 패턴이 맞는 자리마다 코드 2바이트 · 2026-11-21 실측 264판 872곳).
- 5 · 8 · 3 · 6 은 표에서 센 값이고 장부 머리 주석의 두 줄(`lock-lift patches: …` · `generic hooks: …`)을 `binary_patches.rs` 의 시험이 표와 대조한다. 264판 872곳은 독립 검수의 실측을 인용한 값이다. 이 회차에 다시 재지 않았다.
- 범용 훅은 한 판 전용 조정이 아니므로 49행 표식에 섞지 않고 장부 머리 주석에 따로 적었다. 키 없이 전 타이틀에 걸리는 일반 경로 11개는 같은 주석의 «참고» 절에 출처만 걸었다.

### 5. 착지 사전 증거 — 셸이 이 칸을 받아도 green 인가

절차와 출력은 회신 파일에 있다(otterpebble 트리 쓰기 0 · 스크래치 사본에서 실행).

### 6. 한계

- 가드가 보는 것은 «키»다. 장부의 `package` 열(그 키가 어느 꾸러미에 걸리는가)은 게임 바이트가 있어야 확인할 수 있어 CI 가 못 본다. 이 회차는 코퍼스와 대조해 채웠고(§1), 공개 데이터 검사는 그 꾸러미가 지원 목록의 한 행인지 · 통신사와 제목이 그 행과 같은지만 본다. 키를 나눠 쓰는 둘째 꾸러미를 장부와 `compat.json` 에서 함께 빼고 `normalize` 하면 어느 검사도 red 가 아니다.
- `fake-servers` 는 키가 AID 다. 이미 장부에 있는 AID 에 (호스트, 포트) 행을 더하는 것은 가드가 잡지 않는다(같은 판 · 같은 종류).
- 표가 하나 더 생기면(12번째) 가드는 모른다. `test-utils/src/ledger.rs` 의 `DEVICES` 와 그 표의 시험을 함께 더해야 한다.
- 범위 사실은 장부의 `scope` 열과 계약 §1 에 적었다: jar 키 표 7개는 zip 꾸러미에서만 걸리고, AID 키 2개는 맨 `.jar` 면 파일 이름이 AID 와 같을 때만 걸린다. 지원 목록의 파일은 전부 zip 이라 표식은 그 파일에 대해 참이다. 같은 게임을 맨 `.jar` 로 올린 이용자에게는 표식과 실제가 다를 수 있다.
- 엔진을 돌려 재지 않았다. 표 · 코퍼스 해시 대조와 저장소 러너(합성 고정물 6종 PASS)뿐이다. 조정 행을 바꾸지 않았으므로 게임 동작은 재지 않았다.

### 7. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치가 바꾼 파일 전체가 대상) ⇒ BOUNDED 388종 · SUFFIX-ATTACHED 16종.
수가 큰 까닭은 `docs/player-data/compat.json` 이 바꾼 파일에 들기 때문이다. 그 파일의 제목 429행은 이미 공개된 것이고 이 PR 은 제목을 한 글자도 바꾸지 않았다(§3).
이 PR 이 새로 적은 제목은 장부의 `title` 열 47종(54줄)과 이 문서의 「지크」다. 전부 `compat.json` 에 이미 있는 표시 이름 그대로다(검사기가 같음을 강제한다).
SUFFIX-ATTACHED 16종은 `compat.json` 안의 더 긴 다른 제목(예: 뒤에 판 번호가 붙은 후속작)과 안내 문장 속 낱말이고, 이 PR 이 더한 줄에서 나온 것은 없다.
게임 바이트 · 꾸러미 안 파일 본문 · 경로 유입 0. 장부에 든 해시는 꾸러미 sha256 앞 12자와 엔진 표에 이미 있던 MD5 · AID 뿐이다.


