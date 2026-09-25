## [2026-09-25] #161 이 떨군 LGT 행 3개 복원 — `0x415` memmove · WIPIC 412(upstream `3e203809` 채택) · WIPIC 207 GetContext (wie-2026-09-24-lgt-hard-fail-14-axes-adopt-p0)

채택 제안 `2026-09-24-lgt-hard-fail-14-axes#p0`·`#p1`·`#p2`(0230 의 B·C·D 축)의 회차다. 행마다 커밋을 따로 뒀다.
★**p1 의 전제 하나를 뒤집었다.** 412 는 「LGT 전용 0 스텁」이 아니라 upstream 이 2026-09-20 에 넣은 공용 매핑으로 복원했다(아래 ⒝).

### ⒜ 현 main 에서 각 타이틀이 그 SVC 로 멈추는가 — 6/6 그렇다

`origin/main` `1b4944de` · debug `wie_validate`(loadavg 150~210). `--timeout 15` 로는 3건이 SVC 에 닿기 전에 데드라인에 걸려서, 그 3건은 60~150초로 다시 쟀다.

| 타이틀 | 벽(원문) |
|---|---|
| `(LGT)알바타이쿤2` | `Unknown lgt stdlib import: 0x415`(150s · ticks 64) |
| `데몬헌터` | `Unknown lgt stdlib import: 0x415`(ticks 1) |
| `(LGT)리듬페스티발` · `리듬페스티발` | `Unknown LGT WIPIC SVC id 412`(ticks 4) |
| `하이브리드` | `Unknown LGT WIPIC SVC id 412`(60s · ticks 7) |
| `바이오크로니클` | `Unknown LGT WIPIC SVC id 207`(60s · ticks 9) |

### ⒝ `d70b93f8` 행과 의미 대조

- **`0x415`**: 교체 전 `StdlibSvcId::Memmove => stdlib::memmove`. 같은 함수가 `wie-core-arm` 에 그대로 있다 ⇒ **같은 의미**다.
- **207**: 교체 전 `GetContext => graphics::get_context`. 그 `graphics` 는 교체 전에도 공용 `wie_wipi_c::api::graphics` 였다 ⇒ **같은 대상**이다. upstream 에는 이 행이 없다.
- **412** — ★제안의 전제와 다르다. 교체 전 행은 LGT 로컬 스텁(`0` = 「DB 없음」)이었다. 그런데 **upstream `dlunch/wie` `3e203809`(2026-09-20 · 「Map LGT SVC 412 to available database storage」)가 같은 번호를 공용 `database::list_databases`(남은 저장 바이트)로 매핑했다.** 우리 main 에는 아직 없다. 즉 「공용 구현은 KTF 의미라 LGT 에 틀린 값」은 upstream 관리자가 내린 판단과 반대다.
  ⇒ 두 의미를 둘 다 빌드해 짝지어 쟀다(교대 2회):

  | | `(LGT)리듬페스티발` | `리듬페스티발` | `하이브리드` |
  |---|---|---|---|
  | upstream 매핑 | PASS · PASS | PASS · PASS | FAIL(blank) ×1 |
  | 교체 전 0 스텁 | PASS · PASS | PASS · PASS | FAIL(blank) ×1 |

  타이틀은 두 의미를 가르지 않는다. `하이브리드` 의 blank 는 0230 이 교체 **전**에도 FAIL(blank)로 적은 별개 벽이다.
  ⇒ **upstream 3줄을 그대로 가져왔다**(cherry-pick). 제안의 tradeoff 였던 「upstream 과 갈라지는 줄」이 **0** 이 되는 쪽이고, 다음 upstream 동기에서 같은 변경으로 만난다.
  ★한계: 저장 공간 크기에 따라 분기하는 LGT 타이틀이 있다면 두 의미가 갈린다. 이 3건에서는 그런 분기를 보지 못했다.
  ★**형제 감사(#290 · 0254)와 판정이 다르다.** 그 감사는 412 를 「LGT 로컬 스텁으로 복원」하라고 권했다. 그러나 그 표는 upstream `3e203809` 를 대조하지 않았다(「공용 = KTF 의미」까지만 적었다). 이 회차는 그 커밋을 찾았고 두 의미를 짝지어 쟀다. 결과가 같아서 upstream 쪽을 골랐다.
  그 감사가 더 넣으라고 한 표본 `제노니아1`(working·broken)·`하이브리드2`·`놈ZERO` 도 #288 착지 뒤의 main 과 이 브랜치로 짝지어 60초씩 2회 쟀다. 모두 양쪽 `no frame` 이었고, main 도 412 오류를 내지 않았다 ⇒ **이 예산 안에서 412 에 닿지 않는다.** 두 의미를 가를 표본은 아직 없다.

### ⑴ 행별 before / after · 행 제거 변이

before 는 main, after 는 최종 트리, 변이는 최종 트리에서 그 행 한 줄만 뺀 빌드다.

| 행 | 타이틀 | before | after | 변이(행 제거) |
|---|---|---|---|---|
| `0x415` | `(LGT)알바타이쿤2` | FAIL error · 0x415 | **PASS** · paints 23 | FAIL error · `0x415` |
| `0x415` | `데몬헌터` | FAIL error · 0x415 | 다음 벽 207 → 행 3 뒤 **PASS ×2** · paints 3 / 33 | FAIL error · `0x415` |
| 412 | `(LGT)리듬페스티발` | FAIL error · 412 | **PASS ×2** · paints 17 / 22 | FAIL error · `412` |
| 412 | `리듬페스티발` | FAIL error · 412 | **PASS ×2** · paints 21 / 128 | FAIL error · `412` |
| 412 | `하이브리드` | FAIL error · 412 | FAIL · blank(paints 30) — 다음 벽 | FAIL error · `412` |
| 207 | `바이오크로니클` | FAIL error · 207 | **PASS ×2** · paints 6 / 8 | FAIL error · `207` |
| 207 | `데몬헌터` | (0x415 뒤) FAIL error · 207 | **PASS ×2** | FAIL error · `207` |

ticks 는 부하(150~210)에 따라 흔들려서 판정에 쓰지 않았다. 판정은 result 와 벽 문면으로 했다.
**6건 중 5건이 PASS 로 돌아왔다.** `하이브리드` 는 행 뒤의 blank 벽이 남는다.

**추가 표본(0254 권고)**: `게임빌2010프로야구`(broken/lgt)는 #288 착지 뒤 main 에서 `Unknown lgt stdlib import: 0x415` 로 2/2 멈춘다. 이 브랜치에서는 2/2 그 벽을 넘는다(뒤는 60초 no frame). 0x415 행이 여는 일곱 번째 타이틀이다.

### 시험

- `stdlib_import_0x415_is_memmove_through_the_svc_table` — 0x415 **SVC 스텁을 거쳐** 겹치는 복사(dst = src + 2)가 `ababcdef` 로 나오는지 본다. match 행을 빼면 `FatalError("Unknown lgt stdlib import: 0x415")` 로 FAILED 가 난다. `memmove` 를 직접 부르는 시험이었다면 #161 동안에도 green 이었을 것이다.
- `wipic_svc_412_list_databases_is_in_the_table` · `wipic_svc_207_get_context_is_in_the_table` — `try_from` 표 행을 확인한다(기존 `0x581` 시험과 같은 모양).

### 문면

`docs/upstream-realign-p3-slices.md` §D 에 1줄을 더했다: 공용 graphics 배선 27 → **28**(GetContext). ⒝ 결정의 연장이고, ⒜ 로 되돌릴 때 이 줄도 함께 센다.

### ⑵ `working/lgt` 54 전/후 — 짝지은 측정

타이틀마다 main 과 최종 바이너리를 번갈아 돌렸다(순서도 타이틀마다 교대). 판정은 smoke_gate 와 같다(boot+render · `--timeout 15` · 최대 3회 · 워치독 50s). loadavg 는 130~215 였다.

| | PASS | FAIL |
|---|---|---|
| main | 6 | 48 |
| 최종 | **8** | 46 |

- 뒤집힌 4건: `바이오크로니클` FAIL→PASS(행 3)가 이 회차의 고침이다. `무한신맞고2009`·`블레이드마스터4` FAIL→PASS · `영웅서기3` PASS→FAIL.
- 뒤의 셋을 `--timeout 40` 으로 짝지어 다시 쟀다. **양쪽 모두 매 쌍 PASS** 였으므로 15초 데드라인 잡음이다 ⇒ **회귀 0**.
- ★절대 수(6/54)는 0230 의 25(load 55~65)보다 훨씬 낮다. 부하가 3배이기 때문이다. 그리고 15초로는 되찾은 `알바타이쿤2`·`리듬페스티발` 도 첫 그림까지 가지 못한다(이 부하에서 60~120초가 필요했다) ⇒ 이득은 위 ⑴ 표가 보이고, 이 게이트는 회귀 0 만 말한다. 이 수를 코퍼스 상태로 인용하지 마라.

### 검증

- `cargo fmt --check` rc=0 · `cargo clippy --all -D warnings` rc=0 · wasm clippy rc=0 · `cargo +beta clippy --all -D warnings` rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 46 스위트 · **439 passed · 0 failed**(리베이스 후 트리 · 리베이스 전 436).
- `cargo clippy --workspace --all-targets`: 경고 16건은 전부 `wie-backend`·`wie-jvm-support`·`wie_cli` 테스트 코드에서 나왔다. `wie-lgt` 는 0건 ⇒ 경고 증가 0.
- 형제 #288(`…-adopt-p3`, 같은 두 파일의 `0x12c~0x130` 행)과 `git merge-tree` rc=0 이다. 겹치는 행은 없다.
- ★**리베이스**: 작업 중 형제 #288(`…-adopt-p3` · 같은 두 파일의 `0x12c~0x130` 행)과 #290(감사) 이 착지했다(ⓕ·ⓖ). 푸시 전이라 그 위로 리베이스했고 충돌은 0이다. 위 네 게이트는 리베이스된 트리에서 다시 돌렸다. ⑴·⑵ 의 타이틀 측정은 리베이스 전 바이너리로 쟀다. 추가 표본만 리베이스 후 바이너리로 쟀다.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 의 대상은 이 브랜치가 바꾼 6파일이다.
결과는 **BOUNDED 27쌍 · SUFFIX-ATTACHED 2쌍 · PREFIX 0** 이다.
- BOUNDED 는 전부 이 회차가 잰 타이틀 이름이다(대상 6건 · 게이트에서 뒤집힌 3건 · 0254 권고 추가 표본). 예외 하나는 §D 문서에 원래 있던 주석 속 이름이다. 이름일 뿐 바이트는 0이다.
- SUFFIX-ATTACHED 2쌍은 손으로 갈랐다. worklog 두 곳은 stem 에 조사(가·는)가 붙은 «진짜 언급»이다. 회차 문서 한 곳은 stem 뒤에 `2` 가 붙은 «더 긴 다른 제목»이고, 그 제목은 추가 표본으로 BOUNDED 에 있다 ⇒ 새 이름은 0이다.

