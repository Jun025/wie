## [2026-09-16] 제안 `#p1` 을 «기록»하려다 그 전제를 반증했다 — LGT graphics 는 «죽지 않았고», 게이트도 «조용하지 않다» (wie-adopt-slice-d-base-swap-executed-p1)

**무엇을**: 제안 `2026-09-16-slice-d-base-swap-executed#p1`(「upstream LGT graphics 1,095줄이 «아무 테스트도
밟지 않는» 상태를 기록으로 남긴다」)을 채택했다. ★**제안이 적으라고 한 «사실»을 먼저 쟀고, 그것이 «틀렸다».**
⇒ 산출물은 「그 사실의 기록」이 아니라 ★**「그 사실의 정정」**이다. 제품 동작 변경 **0줄**(주석·문서만).

**왜**: 티켓 대전제 ⓒ(「제안은 그 회차의 «관측»이지 «판정»이 아니다 — 틀렸다고 판단되면 근거를 대고 기각하라」).
제안의 전제 문장이 **코드 4곳에 그대로 복사**돼 있었고(`STATE.md` · `docs/upstream-realign-p3-slices.md` §D ·
`docs/report/0117` · 그리고 `wipi_c.rs` 주석의 「NOT WIRED」), ★**되살리는 회차가 처음 읽는 것이 정확히 그 문장**이라
그대로 두면 제안이 막으려던 비용을 **제안 자신이 만든다.**

---

### ⑴ 반증 1 — 「배선을 끊었다」는 **graphics SVC 27개에만** 걸린다

`wie-lgt/src/` 에서 이 모듈로 들어가는 **살아 있는 호출부 3개**가 남아 있다(치환 대상 27줄과 무관한 경로다):

| 호출부 | 대상 |
|---|---|
| `wie-lgt/src/runtime/wipi_c.rs` `clet_register` | `graphics::init_process_state` · `graphics::set_use_annunciator` |
| `wie-lgt/src/runtime/init.rs` `InitSvcId::SetDisplayProperty` | `graphics::set_display_property` |

`clet_register` 는 **LGT clet 등록 경로**라 모든 LGT 게스트 부팅이 지난다.

### ⑵ 반증 2 — 「어느 픽스처도 한 줄도 실행하지 않는다」는 거짓 · ★**실측(`panic!` 프로브)**

각 함수 첫 줄에 `panic!` **1줄**을 넣고(`git diff --numstat` = `1 0`) 두 LGT 픽스처를 돌렸다.
★**정상값과 나란히 적는다 — 한 방향만으로는 상수 pass 와 구별되지 않는다.**

| 프로브 위치 | `keydraw_lgt --inject --expect-last-frame` | `helloworld_lgt --inject` | 판정 |
|---|---|---|---|
| **없음(정상)** | ★`PASS · paints 55 · content true · rc=0` | `PASS · paints 0`(clean exit) | 기준선 |
| `init_process_state` (`:70-98`) | ★**`FAIL · paints 0`**(`panic during 'boot'`) | — | ★**실행된다** |
| `set_use_annunciator` (`:152-157`) | ★**`FAIL · paints 0`** | ★**`FAIL · paints 0`** | ★**실행된다** |
| `set_display_property` (`:121-150`) | `PASS · paints 55` | `PASS · paints 0` | ★**도달 가능하나 미실행** |

⇒ ★**그 35줄(`:70-98` + `:152-157`)은 «모든 LGT 부팅마다» 돈다.** 반대로 `set_display_property` 는
그 SVC 를 켜는 픽스처가 없어 **정적으로만 살아 있다**.

### ⑶ 반증 3 — 「썩어도 게이트가 조용하다」는 **컴파일 축에서 거짓**

`#[allow(dead_code)]` 는 ★**린트를 끄지 컴파일을 끄지 않는다.** 모듈은 여전히 빌드·타입검사된다
⇒ ★**1,095줄 전건이 4게이트 전부를 지난다.** upstream 이 `wie_wipi_c::api::graphics::{FrameBuffer,
decode_image_framebuffer, primitives}` 의 모양을 바꾸면 ★**빌드가 붉어진다.**

★**진짜로 안 잡히는 것은 «동작»이고, 그 범위는 «죽은 45개 항목»이다** — 1,095줄 전체가 아니다.
`#[allow(dead_code)]` 를 떼고 잰 값: ★**경고 45건 · 전건 `wipi_c/graphics.rs`** · 최상위 항목 **57개 중**.
살아 있는 12개 = 위 3함수 + `LgtGraphicsState`·`LgtDisplayProperties` + `GRAPHICS_STATE_ROOT` 등 상수 6.

> ★**측정 중 «거짓 0» 을 한 번 밟았고 숨기지 않는다**: 처음 `cargo clippy -p wie_lgt` 로 쟀는데 패키지명은
> **`wie-lgt`**(하이픈)라 cargo 가 `did not match any packages` 로 **죽었고 경고 0건**이 나왔다. 그 0 을
> 「죽은 코드가 없다」로 읽을 뻔했다. ⇒ ★**0건은 «없다»가 아니라 «못 쟀다»일 수 있다.**
> 같은 형태로 `eprintln!` 마커 실험도 폐기했다 — `wie-lgt` 는 `no_std`(Constraint 6)라 컴파일이 실패했고,
> ★**내 `grep` 이 «런타임 출력»이 아니라 «컴파일러가 되울린 소스 줄»을 세고 있었다**(위 표는 `panic!` 재측값이다).

---

### 고친 자리 — 「한 자리」는 **그 문장이 복사된 4곳**이다

| 파일 | 고친 것 |
|---|---|
| `wie-lgt/src/runtime/wipi_c.rs` | 모듈 선언 주석에 ★**「NOT WIRED 의 범위」 + 살아 있는 3경로 + 45/57 + 컴파일↔동작 구분** 추가 |
| `docs/upstream-realign-p3-slices.md` §D | 거짓 문장에 정정 블록 |
| `STATE.md` | 같은 문장에 정정 블록 |
| `docs/report/0117--….md` | 같은 문장에 정정 포인터(§D 가 «정본»으로 가리키는 파일이라 여기를 안 고치면 재인용된다) |

★**`graphics.rs` 자체는 «한 글자도» 안 건드렸다** — `git diff upstream/main -- <그 파일>` = **0줄**(바이트 동일).
조각 D 가 그 성질을 일부러 지켰고(보고 ⑶), 주석 한 줄이라도 넣으면 그것이 깨진다.
⇒ ★**그래서 기록은 «그 파일»이 아니라 «그 파일을 가리키는 선언»에 붙였다.**

### ★대가 — Contract 2

⒜**무엇을 잃는가**: ⑴`wipi_c.rs` 주석이 **8줄 → 25줄**로 늘었다(읽는 비용). ⑵★**이 수들은 «오늘의 수»다** —
`upstream/main` 이 움직이면 45/57 도 35줄도 바뀐다. ★**상수로 인용되면 다음 사람을 속인다**(이 회차가 정정한
문장이 정확히 그 형태였다). ⑶제품 동작은 **0** 이라 잃는 성능·차단은 없다 — ★**「잃는 것이 없다」가 아니라
「잃는 것이 문서 유지비다」**가 정확하다.

⒝**안 하면 무엇이 나쁜가**: ★**거짓 문장이 4곳에 있고 그중 하나가 «되살리는 회차가 처음 읽는 코드 주석»이다.**
그 회차는 「1,095줄 전부가 검증 없이 썩었을 수 있다」로 시작해 **전수 재검증을 설계**하는데,
실제로 필요한 것은 ★**죽은 45개 항목의 «동작» 검토뿐**이고 컴파일 정합은 이미 지켜져 있다. 반대 방향도 나쁘다 —
「배선이 완전히 끊겼다」를 믿고 `clet_register` 경로를 건드리면 ★**모든 LGT 부팅이 깨지는데** 그 사실이 어디에도 없었다.

### 사용자 영향

**없다.** 제품 동작 0줄 · 주석과 문서만. 프로브는 전부 되돌렸고 `graphics.rs` 는 upstream 과 바이트 동일하다.
