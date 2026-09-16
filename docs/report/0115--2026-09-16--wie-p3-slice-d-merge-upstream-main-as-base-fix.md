## [2026-09-16] 조각 D — base swap 집행 + LGT graphics 배선 27줄 공용 복귀 (wie-p3-slice-d-merge-upstream-main-as-base-fix)

**무엇을** — ★**`upstream/main` 을 base 로 삼는 머지를 «했다».** 총괄이 고른 ⒝(공용 구현 복귀)를 같은 회차에서 집행했다.
★**DoD 리터럴 충족**: `git merge-base HEAD upstream/main` = **`44fbf265`**(≠ `fa641a8a`).

**왜** — blocked 회차가 「중심 결정이 양쪽 다 검증 불가」로 섰고, 총괄이 ⒝를 **명시로** 골라 재개 조건 ⑹-2 가 충족됐다.

### 한 일

| # | 내용 | 무는 것 |
|---|---|---|
| ⑴ | `git merge upstream/main`(리베이스 0) · 미해결 **67** 전건 해소 | `#159` 착지로 74 → **67**(그 8건이 그 PR 이 지운 파일) |
| ⑵ | `wie-lgt/src/runtime/wipi_c.rs` 의 `=> graphics::` **27건** → `=> wie_wipi_c::api::graphics::` | ★`git diff --numstat upstream/main` = **`27 27`**(삽입 27·삭제 27 = «치환»의 서명) |
| ⑶ | upstream 의 LGT 전용 `graphics.rs`(**1,095줄**)를 **지우지 않았다** | `git diff upstream/main -- <그 파일>` = ★**0줄**(바이트 동일) |

★**⑶ 의 부작용을 숨기지 않는다**: 배선을 끊었으므로 그 1,095줄은 **dead code** 가 되어 `-D warnings` 가 **46건**으로 울었다.
⇒ ★**모듈 «선언»에 `#[allow(dead_code)]` + 사유 주석**을 달았다 — ★**파일 자체는 안 건드린다**(Contract 3).

### ★대가 — Contract 2 가 요구한 세 줄

⒤★**benefit 은 «철회»가 아니라 «연기»다.** 제안이 적은 「LGT 전용 그래픽 0→1,095줄」은
★**트리에는 들어왔고 «배선되지 않았다»** 가 참이다. 코드가 사라진 것이 아니라 **호출되지 않는다**.

⒥★**되돌리는 조건과 비용 — 리터럴로**: ⑴**292 코퍼스(특히 LGT 52건)가 배치되거나**
⑵**upstream SDK 가 `wipi/src/framebuffer.rs` 에 LGT 분기를 넣으면** ⇒ ★**배선 27줄을 되돌려 재판정한다.**
★**비용은 «27줄 재배선»이다** — `sed 's/=> wie_wipi_c::api::graphics::/=> graphics::/'` 한 줄이 그 되돌림이고,
이 회차가 **개악 대조로 실제로 돌려 봤다**(아래).

⒦★**`keydraw_lgt` 가 지키는 것** — ★**이 결정의 «유일한» 검증이다**: LGT clet 이
`MC_grpGetScreenFrameBuffer` 가 준 레코드에서 픽셀 포인터를 **직접 읽어** 그리는 경로 전체.
그것이 깨지면 `result` 가 PASS→FAIL 로 뒤집히고 `paints` 가 0 이 된다.

★★**그리고 잃는 것을 축소하지 않는다 — upstream 의 LGT 전용 경로는 이 회차 뒤로 «아무 테스트도 밟지 않는다».**
1,095줄이 트리에 있으나 `#[allow(dead_code)]` 아래에 있고, 우리 5픽스처·`cargo test --all` 어느 것도 그 코드를
**한 줄도 실행하지 않는다**. ⇒ ★**그 코드가 썩어도 우리 게이트는 조용하다.** 되돌리는 회차는 그 사실을 안고 시작하라.

### ★양방향 — 개악 대조 (Acceptance)

| 형상 | `keydraw_lgt --inject --expect-last-frame` |
|---|---|
| ★**개악**(27줄 되돌림 = ⒜를 골랐을 때) | ★**FAIL · paints 0 · rc=1** |
| **정상**(이 회차) | ★**PASS · paints 51 · content true · rc=0** |

★복원 후 `HEAD` sha **불변**(`36df9c31`) · 치환 건수 **27** 재확인.
⇒ ★**이 한 쌍이 «⒜의 결과»를 기계로 남긴다** — 문서가 그 측정을 근거로 삼는다.

### 부수로 드러난 것 (숨기지 않는다)

- ★**hardening 사본 2건이 upstream 에 «이미 있다»**: `StringBuffer.insert(I,String)` · `Timer.schedule(TimerTask,J)`.
  우리 `add()` 가 스스로 `"now exists upstream — drop the wie-side copy"` 를 출력한다 ⇒ **사본과 죽은 구현을 제거**하고
  기대값 **2 → 1**. ★**가드(`append([CII)` null)는 그대로 붙는다** — 잠금 시험이 **먼저** 말했고,
  ★**green 을 보존의 증거로 쓰지 않았다**(그 시험이 red 로 이것을 잡아냈다).
- ★`invoke_virtual` **4곳**을 `+47` API(`class_name` 인자)로 갱신(우리 시험·hardening).
- ★`test_data/helloworld_{ktf,lgt}.zip` 이 머지 rename 으로 upstream 레이아웃(`wie-*/tests/data/`)으로 옮겨져
  러너가 `read error` 를 냈다 ⇒ **`test_data/` 에 복원**(upstream 사본 존치 · `AGENTS.md` 무접촉 = Contract 3).
  ★**엔진 회귀가 «아니었다»** — 경로 이동이었다.

### 게이트 · 남은 것

`fmt` **OK** · `clippy` **0** · ★`beta clippy` **0** · `wasm clippy` **0**.
`cargo test --all` = ★**201 passed · 1 failed**.
5픽스처: `draw_j2me` PASS · `helloworld_ktf` PASS · `helloworld_lgt` PASS · `keydraw_ktf` PASS(55) ·
★**`keydraw_lgt` PASS(51) rc=0** ← **결정 ⒝ 의 목적이 달성됐다**.

★★**남은 1건 — `wie-ktf` `test_key_reach::key_press_reaches_the_ktf_guest`**:
`NUM5 did not reach the KTF guest as WIPI code 53 — guest stdout was "res:9:602\nkey:"`.
★**경로 문제가 아니다**(그 시험은 `include_bytes!` 로 픽스처를 품는다) — **실제 동작 회귀**다.
★**그런데 `keydraw_ktf` 는 PASS(paints 55)** 다 ⇒ 키가 게스트에 **닿기는 한다**.
⇒ ★**좁혀진 후보는 «WIPI 키코드 매핑»** 이고, 그 규명은 이 회차가 하지 못했다(§아래).

### 사용자 영향

엔진이 upstream `44fbf265` 위로 옮겨졌다 — LGT Java import 8→31, canvas 709→1,647 등이 들어왔다.
★**단 LGT 전용 그래픽은 «연기»** 이고, ★**KTF 키코드 축 1건이 열려 있다.**
