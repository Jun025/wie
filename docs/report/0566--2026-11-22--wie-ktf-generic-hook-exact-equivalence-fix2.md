## [2026-11-22] 범용 훅 — «꼴이 다르면 걸지 않는다»와 «훅 쪽이 돌아오지 못한 발화»를 시험과 도구로 잠근다 (wie-ktf-generic-hook-exact-equivalence-fix2)

PR #598 게이트② 반려 R1 · R2 의 승계 회차다. 앞 회차 기록은 `0565`. 훅 동작은 바꾸지 않았다 — 더한 것은 시험 3개, 그림자 대조의 계수 1개, 스윕 스크립트의 판정이다.

**무엇을**

- R1: 스택 인라인 루프(`inline_copy`)의 몸통 패턴은 검사 꼴이 다른 루프에도 맞는다. 그런 자리에 훅을 걸지 않는다는 것을 시험으로 잠갔다(`inline_copy_is_not_installed_on_another_loop_shape`).
  두 꼴 모두 «설치 수 0 · 결과가 루프 자신과 같다»를 단언한다.
  - `while (--len > 0)`: 같은 칸을 쓰고 한 번 덜 돈다(len 8 → 7바이트).
  - 검사는 `len-- > 0` 그대로인데 몸통 바로 앞이 아닌 꼴: 검사와 몸통 사이에서 `len` 이 한 번 더 준다(len 8 → 4바이트).
- R2: 그림자 대조가 «훅 쪽이 재개 지점에 닿지 못한 발화»를 `hook_fault` 로 센다(보고 JSON 의 계수 + `notes` 의 `HOOK-FAULT <종류> site=… stopped=…`).
  종전에는 그런 발화가 대조 수에도 불일치 수에도 들지 않았다.
- `scripts/hook-shadow-sweep.sh` 가 exit 1 을 내는 경우를 넓혔다.
  - `hook_fault` 가 있다.
  - 발화했는데 대조도 건너뜀도 아닌 호출이 있다(`fires − compared − skipped > 0`). 계수가 없는 옛 결과 파일도 이 항으로 잡힌다.
  - 결과 파일에 그림자 보고가 없다(실행이 죽었다) · 결과가 한 건도 없다.
  - `bash scripts/hook-shadow-sweep.sh selftest` 가 지어낸 결과로 종료코드를 확인한다(게임 파일 불요).
  - 표에 `handed to the title's own routine:` 줄을 더했다 — 겹침 호출에 실제로 닿은 판과 횟수다.
- 검수 §6 에서 같이 닫은 것: F1(레지스터 루프 횟수 −1 시험) · F2(뒤쪽 겹침도 게임 루틴에 넘긴다는 것을 `own` 수로 단언) · F3(스크립트 4건 — 위) · F4(장부 · toml 머리 문장).

**왜** — 검수가 제품 줄을 개악해도 시험과 도구가 green 인 자리를 둘 찾았다.
`inline_copy` 를 «빼지 않고 고친» 근거가 루프 꼴 확인인데 그 확인을 꺼도 시험 61개가 통과했고, 코퍼스에 다른 꼴이 없어 전수 대조도 그 축을 못 본다.
또 `0565` 가 새로 만든 길(겹치는 호출을 게임 루틴에 넘김)이 깨져 훅 쪽이 엉뚱한 곳으로 가도 스윕은 exit 0 이었다 — 그 PR 이 관문으로 선언한 도구에 최악의 실패가 안 보였다.

**사용자 영향** — 없다. 게임이 도는 결과는 `0565` 와 같다(아래 §3 · `compat.json` 변화 0 · 소식 항목 추가 0).

### 1. 고친 곳

| 파일 | 내용 |
|---|---|
| `wie-core-arm/src/binary_patches/hook/shadow.rs` | 계수 `hook_fault` · `notes` 줄. 제품 줄 변경은 이것뿐이다(그림자 모드에서만 실행 · `wasm32` 제외). |
| `wie-core-arm/src/binary_patches/hook.rs` | 시험만(`#[cfg(test)]`). 새 시험 2개 · 넓힌 시험 2개. |
| `scripts/hook-shadow-sweep.sh` | `bad` 판정 · `selftest` · 표 2줄. |
| `wie-core-arm/data/binary_patches.toml` | 주석만 — «맞는 자리마다 2바이트를 덮어쓴다»에 꼴이 다른 `inline_copy` 자리는 예외라고 적음 · 확인 줄에 `0566`. |
| `docs/contracts/runtime-adjustments.tsv` | 주석만 — 범용 훅은 그림자 대조 불일치 0 일 때만 남긴다(`0565` 할 일 6 의 «있으면 장부에»). 행 변화 0. |
| `AGENTS.md` · `docs/report/0565` | 도구 항 한 문장 · «장부가 이 repo 에 없다» 문장 정정(#595 로 들어왔다). |

main 을 머지로 당겼다(`e50af53be` · 충돌 0 · 리베이스 아님) — F4 의 두 문장이 #595 가 들여온 파일에 있다.

### 2. 시험 — 개악하면 red

`cargo test -p wie-core-arm --lib binary_patches`: **64 passed · 0 failed**(`0565` 의 61 + 새 시험 2 + main 이 들여온 장부 가드 1).

개악은 제품 줄 한 곳씩이다. 전부 red 를 확인하고 되돌렸다.

| 개악 | 결과 | red 가 된 시험 · 출력 |
|---|---|---|
| R1 검수 개악: 루프 꼴 확인을 `if false && !…` 로 | 63 passed · 1 failed | `inline_copy_is_not_installed_on_another_loop_shape` — `hooks installed (left) vs expected (right)` · `left: 1` · `right: 0` |
| R1 절반: 검사 16바이트 대조만 끔 | 63 · 1 | 같은 시험(`--len > 0` 꼴) |
| R1 절반: «검사 + 18 == 몸통» 만 끔 | 63 · 1 | 같은 시험(검사가 몸통 바로 앞이 아닌 꼴) |
| R2 검수 개악: `resume_original` 의 `bx pc` 갈래를 `site + 8` 로 | 62 · 2 | `shadow_passes_the_right_hook_and_catches_a_wrong_one` — `"memcpy":{"fires":3,"compared":1,…,"own":2,…,"hook_fault":2}` · `memcpy_forward_overlap_gives_the_routines_own_result` |
| `shadow.rs` 가 `hook_fault` 를 세지 않음 | 63 · 1 | `shadow_counts_a_hook_that_never_reaches_the_resume_point` — `"inline_copy":{"fires":1,"compared":0,…,"hook_fault":0}` |
| F1: 레지스터 루프의 `max(0)` 제거 | 63 · 1 | `reg_inline_copy_matches_the_loop` — `count -1: hooked (left) vs the loop itself (right)` |
| F2: 뒤쪽 겹침(dst 가 src 앞) 판정 제거 | 63 · 1 | `shadow_passes_…` — `"memcpy":{"fires":3,"compared":3,…,"own":1,…}`(기대 2) |

`shadow_counts_a_hook_that_never_reaches_the_resume_point` 는 제품 개악 없이도 계수가 오르는 것을 본다: 재개 주소가 틀린 `inline_copy` 훅을 스택 루프에 걸면 `fires 1 · compared 0 · hook_fault 1`.

스크립트(`selftest` · 사본을 한 줄씩 개악):

| 개악 | `selftest` 출력 |
|---|---|
| `bad` 에서 `fires − compared − skipped` 항 제거 | `selftest FAIL: fires 10 · compared 3 · skipped 0 — exit 0, expected 1` |
| `bad` 를 종전 줄로 | 같은 줄 |
| `bad` 에서 `hook_fault` 제거 | `selftest FAIL: hook_fault 1 — exit 0, expected 1` |
| 결과 0건 통과 | `selftest FAIL: no results — exit 0, expected 1` |
| 보고 없는 결과 파일을 세지 않음 | `selftest FAIL: a result file with nothing in it — exit 0, expected 1` |

검수의 합성 결과 그대로(옛 서식 · `fires 10 · compared 3 · skipped 0` · `hook_fault` 칸 없음)를 표 모드에 넣으면 rc=1 이다. 지금 트리는 `selftest ok`.

### 3. 그림자 대조 재실행 — 차이가 났던 3판 + 인라인 루프 패턴이 있는 14판

새 계수를 넣은 릴리스 빌드(이 회차 트리)로 16판(3판 중 `f80ca6d2cb48` 은 14판에도 든다)을 돌렸다. 인자는 `0565` 와 같다(census 탐침 A · 30초).
겹침이 있는 2판은 30초 · 100초로 세 번씩 더 돌렸다. `build-slot --long` 임대 7건 · 동시 3개 이하 · 09:22 ~ 09:31 · load1 21.7 ~ 125.7.

**합계: 16판 · 28회 실행 · 발화 18,856,947 = 대조 18,856,947 · 불일치 0 · `hook_fault` 0 · `own_fault` 0 · 건너뜀 0 · `notes` 0줄 · 게임 루틴에 넘김 6.** 스크립트 exit 0(7회 모두).

| 판 | 실행 | 발화 | 대조 | 불일치 | `own_fault` | `hook_fault` | 그중 루프 훅 발화 | 게임 루틴에 넘김(닿은 실행) |
|---|---|---|---|---|---|---|---|---|
| `1fe6c1c897d7` | 7 | 9357511 | 9357511 | 0 | 0 | 0 | 0 | 2 (1회) |
| `2b1ed0c8d061` | 1 | 1746 | 1746 | 0 | 0 | 0 | 0 | 0 |
| `2bafbffeb742` | 1 | 4191 | 4191 | 0 | 0 | 0 | 101 | 0 |
| `3151fdc167b6` | 1 | 5977 | 5977 | 0 | 0 | 0 | 0 | 0 |
| `4166acd8fc62` | 1 | 3577 | 3577 | 0 | 0 | 0 | 0 | 0 |
| `79d9bc10be82` | 1 | 9283 | 9283 | 0 | 0 | 0 | 107 | 0 |
| `7da00ecd4804` | 1 | 25332 | 25332 | 0 | 0 | 0 | 0 | 0 |
| `7eb95dc3b02c` | 1 | 18371 | 18371 | 0 | 0 | 0 | 274 | 0 |
| `93d5b6b8ceb5` | 1 | 22443 | 22443 | 0 | 0 | 0 | 0 | 0 |
| `94a34abdea21` | 1 | 24008 | 24008 | 0 | 0 | 0 | 0 | 0 |
| `9d52de42e7f9` | 1 | 570 | 570 | 0 | 0 | 0 | 0 | 0 |
| `a603436b4ec4` | 1 | 498 | 498 | 0 | 0 | 0 | 90 | 0 |
| `d9384b388ea5` | 7 | 9368367 | 9368367 | 0 | 0 | 0 | 0 | 4 (2회) |
| `dbc45612d910` | 1 | 329 | 329 | 0 | 0 | 0 | 190 | 0 |
| `e70c0c1c282c` | 1 | 14530 | 14530 | 0 | 0 | 0 | 92 | 0 |
| `f80ca6d2cb48` | 1 | 214 | 214 | 0 | 0 | 0 | 176 | 0 |

- 겹침 호출 도달: `1fe6c1c897d7` 은 7회 중 1회(30초) · `d9384b388ea5` 는 7회 중 2회(30초 2). 100초 실행 3회는 두 판 모두 닿지 않았다 — 검수 F10 과 같다(도달은 실행마다 갈리고 길이로 정해지지 않는다).
  닿은 3회에서 넘긴 호출 6건은 전부 대조됐고 다름 0 · 미도달 0 이다.
- 인라인 루프 패턴이 있는 14판 중 루프 훅이 30초 안에 발화한 것은 7판이다(레지스터 루프 6 · 스택 루프 1 · `0565` 전수와 같은 7판). 나머지 7판의 루프 자리는 이번에도 대조되지 않았다.
- 전수 264판은 다시 돌리지 않았다(티켓). `0565` 의 전수 표는 `hook_fault` 계수가 없는 빌드의 값이지만 264행 전부 발화 = 대조였다 — 새 판정의 `fires − compared − skipped` 항으로도 0 이다.

### 4. 게이트 — 이 회차 트리(main `80f314e1a` 를 당긴 뒤)

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | rc=0 |
| `cargo clippy --all -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=0 |
| 같은 줄에 `-p wie-core-arm -p wie_featurephone` | rc=0(2분 16초 — 그 crate 를 실제로 컴파일) |
| `cargo +beta clippy --all -- -D warnings`(`1.101.0-beta.2`) | rc=0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | rc=0 · 54 스위트 · 865 passed · 0 failed |
| 러너 블록(문서대로 debug) | 6종 PASS — `keydraw_ktf` rc=0 · paints 79 · 27/27 · `keydraw_lgt` rc=0 · paints 55 · 27/27. 나머지 4종의 종료코드는 따로 받지 않았다(판정 PASS 만 읽음). |
| `npm run audit` · `node scripts/player-data.mjs` | rc=0 · `429 titles … 183 updates` |
| `node scripts/check-compat-revert.mjs` | 되돌린 필드 0 · 착지 기준 바뀐 행 0 |

`WIE_HOOK_MODE` 미설정: 결과 줄 키 23개 · `hook_shadow` 없음 · `WIE_HOOK_MODE=off` 와 같고, `shadow` 일 때의 키에서 `hook_shadow` 를 뺀 것과 같다(릴리스 · `keydraw_ktf`). 이 회차는 `wie_validate.rs` 와 훅의 실행 경로(`dispatch` 이하)를 건드리지 않았다.

### 5. 남기는 목록 — 이 회차가 닫지 않은 것

- F5: 루프 2종 뒤에 몸통 임시값(r0–r7 중 일부 · 레지스터 루프는 플래그)이 게임 루프와 다르다. 대조하지 않는다(`shadow.rs` 머리). 닫으려면 훅이 그 값을 써야 한다 — 제품 줄.
- F6: `spill_back = false` 는 이제 항상 틀린 설정이다(훅이 돌아가는 검사가 `len` 칸을 다시 읽는다). 필드를 없애거나 파서가 거절해야 한다 — 제품 줄.
- F7: `bx pc` 넘김의 재개 주소가 `site + 4` 고정이다. 썽크가 4바이트 정렬이 아니면 다르다(패턴상 코퍼스에서는 정렬돼 있다) — 제품 줄.
- F8: 릴리스 `wie_validate` 로 `keydraw_*` 를 러너 블록 인자로 돌리면 `UNMEASURED`(max-ticks). main 과 대조하지 않았다. `keydraw_lgt` 에서도 같다고 했는데 LGT 에는 범용 훅을 설치하지 않으므로 훅 코드 탓은 아닐 것이다 — 추론이고 재지 않았다.
- F9: 앞 회차 done 회신의 미치환 자리표시자 — 그 회신 파일은 고치지 않았다.
- 그림자 대조의 새 항 `fires − compared − skipped` 는 «엔진 루틴이 오류를 내고 넘길 수도 없는» 훅 자리(첫 명령이 `bx pc` · `push` 가 아닌 자리에 pc 로 건 훅 — 지금 표에는 없다)의 발화도 잡는다. 범용 패턴 4종은 전부 넘길 수 있는 첫 명령(`bx pc` · `push`)이라 해당이 없다.
- `memset` 은 이번에도 0회다(`0565` §5 · worklog 제안 그대로).
- 네 게이트의 wasm clippy 가 `wie-core-arm` 을 보지 않는 것(`0565` §5)도 그대로다.

후속 제안은 새로 내지 않았다(worklog 없음).

증적(개수와 sha12 만): `~/orchestrator/reports/evidence/wie-ktf-generic-hook-exact-equivalence-fix2/`.

유입(`node scripts/corpus-name-inflow.mjs`, 이 브랜치 대 `origin/main`): BOUNDED 48회/40쌍 · SUFFIX-ATTACHED 0회/0쌍이다. 바뀐 파일 «전체»를 훑은 수다.
48 중 44 는 `docs/contracts/runtime-adjustments.tsv`(#595 가 main 에 들인 장부의 기존 행 — 이 회차가 그 파일에 더한 것은 이름 없는 주석 2줄)이고 4 는 `wie_cli/src/bin/wie_validate.rs` 에 이미 있던 주석 줄이다(`0565` 와 같은 4).
이 브랜치가 더한 줄에 걸린 것은 0 이다. `0565` 에 있던 표식은 이 회차가 그 파일과 대상 파일들을 고쳐 낡았으므로 떼고 아래에 새로 찍는다.
