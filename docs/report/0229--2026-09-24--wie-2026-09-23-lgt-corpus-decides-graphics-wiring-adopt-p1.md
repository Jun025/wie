## [2026-09-24] LGT 하드 실패 14건 분류: 근인은 하나(#161 이 우리 LGT 수리분을 떨궜다), 축은 여섯. 가장 싼 축 `free(NULL)` 만 닫았다 (wie-2026-09-23-lgt-corpus-decides-graphics-wiring-adopt-p1)

**무엇을**: `docs/report/0222` §4 의 「baseline PASS 12 + 코퍼스 전용 2 = 14건 하드 실패」를 원인 축별로 나눴다.
그중 가장 싼 축 하나(`LgtWIPICContext::free` 의 NULL 가드)만 고쳤다. 제품 코드 +7줄, 1파일(`wie-lgt/src/runtime/wipi_c/context.rs`).

**왜**: 채택 제안 `2026-09-23-lgt-corpus-decides-graphics-wiring#p1`.

**사용자 영향**: `메탈슬러그 서바이벌` LGT 두 사본이 다시 부팅·렌더한다(호스트 panic → PASS).

## 1. 전제 반증부터

- ⒜ **현 main(`24a920c4`)에서 14건 × 2회 = 28/28 FAIL**(paints 전건 0). 줄어든 것이 없다.
- ⒝ **baseline 행은 실제로 과거에 PASS 였다.** `smoke_gate_baseline.tsv` 의 해당 LGT 행은 2026-06-30~07-06
  `UPDATE_BASELINE` 라이브 실행으로 들어갔다(`99a66f0c`·`0942b0fb`·`97f764e2`). 그리고 **기반 교체 직전 트리
  `d70b93f8` 를 빌드해 같은 14건을 돌리면 8건이 2/2 PASS** 다(load ~100). ⇒ 그 8건은 기록 정정이 아니라 **회귀**다.
- 나머지 6건(`제노니아1`·`제노니아2`·`하이브리드2`·`하이브리드`·`놈ZERO`·`나는마왕이다2`)은 **교체 전 트리에서도
  FAIL** 했다(hang 또는 blank). 이 부하에서는 회귀인지 판정할 수 없다. 데드라인에 걸린 타이틀이기 때문이다(`0222` §2-1).

## 2. 회귀 경계: 인접 두 커밋

| first-parent 커밋 | `Memmove=0x415` | `GetContext=0xcf` | `ListDatabases=0x19c` | `free` NULL 가드 |
|---|---|---|---|---|
| `d70b93f8`(#160, 교체 직전) | 1 | 1 | 1 | 1 |
| `37734e74`(#161 조각 D, 기반 교체) | 0 | 0 | 0 | 0 |

⇒ **#161 이 `wie_lgt` 를 업스트림 `wie-lgt` 로 갈아끼우면서 우리 LGT 수리분이 함께 사라졌다.** 이 둘은 인접한
first-parent 커밋이라 이분 탐색을 더 할 필요가 없다. 대신 양 끝(`d70b93f8` 빌드, main 빌드)을 실제로 돌려 확인했다.

## 3. 분류 (Contract 1): main `24a920c4` · 각 2/2 · paints 전건 0

| 축 | 타이틀 | main 문면 | 교체 전 `d70b93f8` | 근거 |
|---|---|---|---|---|
| **A** `free(NULL)` 가드 소실 | `(LGT) 메탈슬러그 서바이벌` · `메탈슬러그 서바이벌` | panic `attempt to subtract with overflow` | PASS 2/2 · PASS 2/2 | 백트레이스 `context.rs:42` ← `destroy_offscreen_framebuffer`. 교체 전 가드 = `0942b0fb`. **★이번에 고침** |
| **B** stdlib `0x415` 행 소실 | `(LGT)알바타이쿤2` · `데몬헌터` | `Unknown lgt stdlib import: 0x415` | PASS · PASS | 교체 전 `Memmove = 0x415`. 구현 `wie-core-arm::stdlib::memmove` 는 지금도 있다 |
| **C** WIPIC `412`(`0x19c` ListDatabases) 행 소실 | `(LGT)리듬페스티발` · `리듬페스티발` · `하이브리드` | `Unknown LGT WIPIC SVC id 412` | PASS · PASS · FAIL(blank) | 교체 전 = LGT 전용 스텁(「DB 없음」→0). 공용 `wie-wipi-c` 의 `list_databases` 는 **KTF 의미**(남은 저장 바이트)라 행만 추가해서는 안 된다 |
| **D** WIPIC `207`(`0xcf` GetContext) 행 소실 | `바이오크로니클` | `Unknown LGT WIPIC SVC id 207` | PASS | 공용 `graphics::get_context` 는 있다. graphics 배선이라 조각 D 영역에 걸린다 |
| **E** stdlib `0x410` 이 이제 진짜 `strstr`, 게스트가 NULL 을 넘김 | `게임빌2010슈퍼사커` · `제노니아1` · `제노니아2` · `하이브리드2` · `놈ZERO` | `Invalid memory access; address: 0` | PASS · FAIL · FAIL · FAIL · FAIL(blank) | 5/5 추적 결과 직전 줄이 `stub unk4(0x0,…)`(WIPIC `0x12d`), 그 뒤 `strstr(0x0, …)`. 교체 전 `0x410` = no-op `unk5` 였고 r0=0 을 남겼다 |
| **F** 렌더 데드라인 | `나는마왕이다2` | `no frame rendered` | FAIL | 못 쟀다(`0222` 의 잡음 타이틀) |

**E 는 패치를 실제로 써서 쟀다. 그리고 싣지 않았다.** `strstr` 에 NULL 가드를 넣으면 4건(`제노니아1/2`·`하이브리드2`·`놈ZERO`)이
교체 전과 같은 상태(hang / blank)로 돌아가고, `게임빌` 은 다음 소실 행(축 C, 412)에 걸린다 ⇒ **단독으로는 PASS 회복 0**.
그래서 PASS 를 실제로 되찾는 A 를 골랐다.

## 4. 고친 축 A: 전/후와 양방향 변이 (Acceptance ⑵)

| 바이너리 | `(LGT) 메탈슬러그 서바이벌` ×3 | `메탈슬러그 서바이벌` ×3 |
|---|---|---|
| main `24a920c4`(= 가드를 뺀 상태) | FAIL ×3 · paints 0 · ticks 1~4 · panic | FAIL ×3 · paints 0 · ticks 1~3 · panic |
| main + 가드 `c31ab833` | **PASS ×3** · paints 12/12/8 · ticks 90,127~248,126 | **PASS ×3** · paints 4/3/3 · ticks 25,351~55,160 |

변이 검사: 수정을 되돌린 트리가 곧 main 이다(diff = 이 가드 7줄뿐). ⇒ 가드를 빼면 6/6 재실패, 넣으면 6/6 통과.

**가드가 다른 타이틀을 바꿀 수 없는 이유**: 바뀌는 곳은 `memory == 0` 경로 하나이고, main 에서 그 경로는 **언제나 panic** 한다.
따라서 main 에서 panic 하지 않은 타이틀은 그 경로를 한 번도 지나지 않았다.

## 5. `working/lgt` 54 전/후 (Acceptance ⑶): 짝지은 측정

타이틀마다 두 바이너리를 번갈아 돌렸다. 각각 smoke_gate 와 같은 판정(boot+render · `--timeout 15` · 재시도 2 · 워치독 50s)을 썼고, load 는 55~65 였다.

| | PASS | FAIL |
|---|---|---|
| main | 25 | 29 |
| main + 가드 | **29** | 25 |

뒤집힌 6건: 메탈 ×2 FAIL→PASS(이번 고침) · `(LGT)검은방3`·`아니마`·`일지매영웅전기2` FAIL→PASS · `레이카르나` PASS→FAIL.
**`레이카르나` 는 회귀가 아니다.** 번갈아 3회씩 재측하니 main 3/3 · 가드 3/3 이 똑같이 `no frame rendered` 였다(데드라인 잡음).
panic 경로가 없는 타이틀이라 위 §4 의 논리로도 영향을 받을 수 없다. ⇒ **회귀 0**.
★절대 수(25/29)는 `0222` 의 40(load 6~20)보다 낮다. 부하 탓이며, 이 수를 코퍼스 상태로 인용하지 마라.

## 6. 남긴 축: 후속 제안(worklog)

B(행 2줄), C(LGT 전용 스텁 복원), D(graphics 배선 한 행), E(`unk4` 정체 또는 `strstr` NULL 가드. 패치는 쟀다),
그리고 **교체가 떨군 LGT 수리분 전수 감사**. 전수 감사가 이번 여섯 축의 상위 질문이다. `d70b93f8` 대비 `wie-lgt` 에서 사라진 SVC 행과 가드를
한 번에 세면, 나머지 코퍼스(`broken/lgt`)에서도 같은 병을 미리 찾는다.

## 검증

- 네 게이트: `cargo fmt --check` rc=0 · `cargo clippy --all -D warnings` rc=0 · wasm clippy rc=0 ·
  `RUST_MIN_STACK=4194304 cargo test --all` rc=0(46 suites · 420 passed · 0 failed). `cargo +beta clippy --all -D warnings` rc=0.
- `cargo clippy --all --all-targets -D warnings` 는 rc=101 이다. 원인은 이 회차가 건드리지 않은 `wie-backend` 테스트 코드에 이미 있던 lint 11건
  (`unusual_byte_groupings` 8 · byte-str 3)이다. `-p wie-lgt --all-targets` 는 rc=0 ⇒ **경고 증가 0**.
- 러너 블록: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt` `--inject --expect-last-frame`
  PASS rc=0(paints 35/36, 27/27, load ~60) · `text_j2me --timeout 5` PASS.
- 단위 테스트는 없다. `LgtWIPICContext` 는 완전한 `Jvm` 이 있어야 만들어진다. 교체 전 가드(`0942b0fb`)도 테스트가 없었고,
  그래서 #161 에서 조용히 사라졌다. 이 공백은 §6 의 전수 감사 제안이 진다.

## 한계

- 판정 축은 boot+render 뿐이다. 되찾은 메탈슬러그가 무엇을 그리는지는 대조하지 않았다.
- 6건(§1 ⒝)은 이 부하에서 교체 전에도 실패해 회귀 여부를 가르지 못했다. `놈ZERO` 의 `docs/lgt_abi.md` cp47
  「PASS · 153 paints」는 교체 전 트리에서도 이번에 blank(paints 14/20)였다.

## 게임 파일명 유입 (도구를 실행해 적는다)

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 결과(이 절을 쓰기 전):
**BOUNDED 28쌍** · PREFIX-EMBEDDED 0 · **SUFFIX-ATTACHED 6쌍**. 수는 아래 표식이 최종이다.
SUFFIX 는 손으로 갈랐다. `일지매영웅전기2`·`하이브리드2` 는 코퍼스의 **별개 실제 타이틀**이다. 나머지
(`게임빌2010슈퍼사커도`·`데몬헌터가`·`바이오크로니클이`·`하이브리드가`)는 조사가 붙은 **진짜 언급**이다. ⇒ 판단이 더 필요한 것 0.
유입된 이름은 전부 분류 표의 식별자다. 게임 바이트 유입 0 · 경로 유입 0.
