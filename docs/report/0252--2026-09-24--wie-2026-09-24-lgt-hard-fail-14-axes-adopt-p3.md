## [2026-09-24] LGT WIPIC `0x12d`(unk4) = `MC_imGetSupportedModes` — IM 모드 목록을 돌려주고 `0x130` 행을 등록했다 (wie-2026-09-24-lgt-hard-fail-14-axes-adopt-p3)

**무엇을**: `0x12c..0x130` 을 입력기(IM) 묶음으로 판정하고, `0x12c`(모드 수)·`0x12d`(모드 이름 `char**`)를 구현, `0x130`(`MC_imHandleInput`)을 no-op 행으로 등록했다. 제품 코드 2파일(`wie-lgt/src/runtime/{svc_ids,wipi_c}.rs`).

**왜**: 채택 제안 `2026-09-24-lgt-hard-fail-14-axes#p3`(원문 `docs/worklog/2026-09-24-lgt-hard-fail-14-axes.json` `proposals[3]`). 가드가 아니라 정체 규명이 요구였다.

**사용자 영향**: 5건 전부 `address: 0` 벽을 넘었다. `하이브리드2` 는 한 번 PASS(233 paints)했고, `놈ZERO` 는 처음으로 프레임을 낸다(아직 빈 화면). 나머지 3건은 이제 축 C(`412` ListDatabases) 에서 멈춘다.

## 1. 전제 반증 — 서 있다

- ⒜ 현 main(`2e356dfc`) release `wie_validate`: **5/5 FAIL**, 전건 `stub unk4(0x0, …)` 직후 `Invalid memory access; address: 0` · paints 0 · ticks 2/2/3/5/6(제노니아1·2·하이브리드2·놈ZERO·게임빌).
- ⒝ 벤더 헤더 + 게스트 호출부 대조(아래 §2). KTF 는 같은 함수를 `gen_stub(41, "MC_imGetSupportedModes")` 로만 둔다(`wie-ktf/src/runtime/wipi_c/method_table.rs:617`) — 대응 구현이 없어 대조 근거로는 이름뿐이다.

## 2. 판정: `0x12c..0x130` = IM 묶음, `0x12d` = `char** MC_imGetSupportedModes()`

근거 셋, 독립이다:

1. **헤더 순서** — `docs/reference/WIPIHeader.h:1442-1446`: `imGetSurpportModeCount` · `imGetSupportedModes`(`char**`) · `imSetCurrentMode(M_Int32)` · `imGetCurrentMode` · `imHandleInput(char, M_Int32, char*, M_Int32*, char*, M_Int32*)`. LGT 의 다른 묶음도 헤더 순서를 따른다(graphics `0xc8..` 가 `GetImageProperty` 부터 한 칸씩 일치).
2. **호출부 모양**(capstone 디스어셈블, `binary.mod` ELF `.text` VMA 0x1000):
   - `제노니아1` Thumb `0x607c`: `0x12c()` → `[this+0xc]` · `r4 = 0x12d()` · `ldr r0,[r4]`(**`modes[0]` 역참조**) · `strstr(modes[0], "/L")` 없으면 `strstr(modes[0], "/S")` · `0x12e(2)`. 스텁이 0 을 주면 `ldr r0,[r4]` 가 0 을 읽고 다음 `strstr(0, …)` 이 벽이다.
   - `아니마` ARM `0x53aac`: `r7 = 0x12c()` · `r8 = 0x12d()` · `for i < r7: strcpy(buf, r8[i])` 후 원하는 이름(`EN/S`·`EN/L`·`KO`·`N123`)과 비교, 맞으면 **`0x12e(i)`** — 인덱스를 SetCurrentMode 에 넘긴다.
   - `그랜드체이스` Thumb `0x38ef8`: `0x130('1', 0x1f6, buf1, &size1=6, buf2, &size2=6)` — 헤더의 6-인자 모양 그대로.
3. **코퍼스 어휘** — `game_lab/*/lgt` 전수(git-ignored)에서 `0x12d` 를 import 하는 binary.mod **33개**, 그 호출부 리터럴 풀에서 모드 이름으로 찾는 문자열은 `KO` · `EN` · `EN/L` · `EN/S` · `N123` · `/L` · `/S` 뿐이다.

## 3. 구현

- `0x12c` → 4 · `0x12d` → 4개 이름 + NULL 종결 `char**`(호출마다 `alloc_raw`).
- **순서는 측정값이 아니다**: `["EN/S","EN/L","KO","N123"]` 은 `아니마` 자신의 enum(0 EN/S · 1 EN/L · 2 KO · 3 N123)이다. 실기 순서를 댈 근거가 없다. `제노니아` 류는 `modes[0]` 에 `/L`·`/S`·둘 다 없음 세 경우를 모두 처리하는 코드라 어느 순서든 벽은 없다. 코드에 `ponytail:` 로 적었다.
- `0x130`(`MC_imHandleInput`)은 **로그 + 0 반환 no-op**. 필요해진 이유가 측정이다 — 아래 §4 변이 ②.

## 4. 전/후와 양방향 변이 (Acceptance ⑵)

5건, `--timeout 60`, load 105~200:

| 타이틀 | main `2e356dfc` | 이 브랜치 |
|---|---|---|
| 제노니아1 | FAIL `address: 0` · ticks 2 ×2 | FAIL `Unknown LGT WIPIC SVC id 412` · ticks 183/164 |
| 제노니아2 | FAIL `address: 0` · ticks 2 ×2 | FAIL `SVC id 412` · ticks 189/144/217 |
| 하이브리드2 | FAIL `address: 0` · ticks 7/4 | **PASS paints 233** · 1회 FAIL `no frame rendered`(ticks 402 · 부하) |
| 놈ZERO | FAIL `address: 0` · ticks 5/14 | FAIL `only blank/uniform frames` · paints 178/205 |
| 게임빌2010슈퍼사커 | FAIL `address: 0` · ticks 15/17 | FAIL `SVC id 412` · ticks 22/20/16 |

변이 두 축:

- ① **diff 제거 ⇒ 원래 벽 재현**: main 바이너리 = 이 diff 를 뺀 상태, 위 왼쪽 열 5/5 가 `stub unk4(0x0,…)` → `address: 0` 원문 그대로. **diff 적용 ⇒ 벽 소멸** 5/5(오른쪽 열 — `address: 0` 0건).
- ② **`0x130` 행 제거 ⇒ 회귀 재현**: 중간 빌드(`0x12c/0x12d` 만, `0x130` 없음)에서 `그랜드체이스` 가 main PASS(paints 183) → **FAIL `Unknown LGT WIPIC SVC id 304`**. `0x12c` 가 4 를 주자 모드 루프가 돌고 `0x130` 까지 간다. 행을 넣은 최종 빌드에서 **PASS ×2**(paints 106/98 · `MC_imHandleInput` 로그 2줄), main 대조 PASS ×2(82/85).
- **못 한 것**: `0x12d` 행 **하나만** 뺀 빌드는 만들지 않았다. 이 부하에서 release 링크가 142~165분이라서다. ①이 diff 전체를 되돌린 대조다.

## 5. 회귀 — `working/lgt` 중 `0x12d` import 20건 짝지은 측정

- 기본 예산(load ~120)에서 main PASS → 이 브랜치 FAIL 인 행 **0**. 기아 FAIL(`no frame rendered`)은 양쪽에 같이 나왔다(그랜드체이스·리듬스타1·2·미니게임천국4·아니마).
- 그 5건을 `--timeout 60` 으로 재측: 리듬스타1·2·아니마 main/브랜치 각 2/2 PASS · 그랜드체이스 위 ② · 미니게임천국4 는 **양쪽 모두 가끔 FAIL** — main 1/3 FAIL, 브랜치 2/7 FAIL, 서명 동일(ticks 632~656 · 무프레임) ⇒ 부하 기아이지 회귀가 아니다.

## 6. 게이트

`cargo fmt --check` OK · `cargo clippy --all -D warnings` rc=0 · wasm clippy rc=0 · `+beta` clippy rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 **46 suites · 432 passed · 0 failed**. `cargo clippy --workspace --all-targets` 경고 16건 전부 이 diff 밖(`wie-backend/src/canvas.rs` · `wie-jvm-support/src/hardening.rs` · `wie_cli/tests/support/dod_ci_parity.rs`) ⇒ 증가 0.
러너 블록(release `wie_validate` 로 실행): draw_j2me · helloworld_ktf/lgt PASS · keydraw_ktf 53 / keydraw_lgt 54 paints `--inject --expect-last-frame` rc=0 · text_j2me PASS.

## 7. 남은 것

- 제노니아1·2 · 게임빌의 다음 벽은 **축 C(`412`)** — `2026-09-24-lgt-hard-fail-14-axes#p1` 이 진다(이 3건이 그 제안의 대상에 새로 들어간다). 새로 발권하지 않는다.
- `놈ZERO` 빈 화면은 새 축이다(후속 제안).
- `0x130` 은 IME 가 없다. 글자 입력이 필요한 타이틀이 나오면 그때 구현한다(코드의 `ponytail:`).
