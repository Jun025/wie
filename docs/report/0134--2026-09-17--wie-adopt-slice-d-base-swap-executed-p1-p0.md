## [2026-09-17] SetDisplayProperty SVC 경로에 픽스처는 여전히 0 — 배선을 유닛 테스트가 잡게 했다 (wie-adopt-slice-d-base-swap-executed-p1-p0)

**무엇을**: `InitSvcId::SetDisplayProperty` SVC 경로(import table `(0x1f8, 0x16)` → INIT SVC 디스패치 →
`graphics::set_display_property`)를 실제로 밟는 유닛 테스트 1건을 추가했다. 제품 동작 0줄
— `init.rs` 의 두 함수를 `pub(crate)` 로 넓힌 것이 제품 코드의 전부다.

**왜**: 제안 `2026-09-16-adopt-slice-d-base-swap-executed-p1#p0` 이 「그 SVC 는 정적으로 도달 가능한데
어느 픽스처도 켜지 않는다」를 실측으로 제기했다. **2026-09-17 재측으로 그 전제는 «여전히 참»이다**:
`set_display_property` 머리에 `panic!` 1줄(`git diff --numstat` = `1 0`)을 넣어도
`helloworld_lgt` PASS·paints 0 · `keydraw_lgt --inject --expect-last-frame` PASS·paints 55·content true.

**제안의 처방을 그대로 따르지 않은 이유(적어 둔다)**: 제안의 tradeoff 는 「새 픽스처가 필요할 수 있고
게스트 SDK 로 만들어야 한다 · 먼저 어느 LGT 타이틀이 이 SVC 를 쓰는지 조사가 순서다」였다.
그 조사는 **292 코퍼스 없이는 이 회차에서 못 한다**(게이트가 조용한 것과 별개 사실이다).
그러나 「게이트가 조용하다」의 원인은 **둘로 갈린다**:
⒜ `set_display_property` 의 **본문 로직** — 이것은 이미 잡혀 있었다
   (`display_properties_update_physical_display_state`, 함수를 직접 호출한다).
⒝ **배선** — import table 항목 `(0x1f8, 0x16)` 과 `handle_init_svc` 의 `SetDisplayProperty` 갈래.
   ★**이 축은 픽스처도 유닛 테스트도 «아무것도» 잡고 있지 않았다.**
⒝ 는 픽스처 없이 잡을 수 있고(같은 파일의 `compiler_array_helpers_build_guest_arrays` 가 이미 그 형태다 —
SVC 스텁을 만들어 `core.run_function` 으로 실행한다), ⒜ 는 이미 잡혀 있다.
⇒ **이 회차는 ⒝만 닫았고, ⒜⒝ 어디에도 속하지 않는 「실제 LGT 타이틀이 이 SVC 를 쓰는가」는 열어 뒀다.**

**사용자 영향**: 없다(제품 동작 0줄). LGT 게임이 화면 폭·높이·주석창 설정을 바꾸는 통로의 **배선이
끊어지면** 이제 `cargo test --all` 이 6개 매트릭스 레그 전부에서 red 가 된다 — 종전에는 조용했다.

**잃는 것**: `init.rs` 의 `register_init_svc_handler`·`get_import_function` 이 `pub(crate)` 로 넓어졌다
(crate 밖에는 안 나간다). 테스트 1건 + 약 0.07초. 그 외 없다.
**안 하면 무엇이 나쁜가**: 배선 두 곳 중 하나가 바뀌어도 5픽스처 전건 PASS 이고 `dead_code` 도 울지 않는다
(함수가 도달 «가능»하므로). 즉 LGT 게임의 화면 속성 설정이 통째로 죽어도 **CI 가 green 이다.**

### 개악 대조(양방향 · 제품 «호출부»)
| 개악 | numstat | 결과 |
|---|---|---|
| `init.rs` import table `(0x1f8, 0x16)` → `(0x1f8, 0x15)` | `3 3` | **FAILED** 1 failed |
| `init.rs` 디스패치 갈래 `SetDisplayProperty` → `get_import_table` | `3 3` | **FAILED** `assertion left == right` @ graphics.rs:1027 |
| 복원 | `2 2`(= `pub(crate)` 2줄만) | **ok** 1 passed |

### 4게이트 + 5픽스처
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 ·
  `cargo clippy --target wasm32-unknown-unknown --all -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` — **전건 합산 385 passed / 0 failed**(result 줄 42개 합산 · `tail` 미사용)
- 5픽스처: `draw_j2me` PASS/1/true rc=0 · `helloworld_ktf` PASS/0/false rc=0 · `helloworld_lgt` PASS/0/false rc=0 ·
  `keydraw_ktf --inject --expect-last-frame` PASS/35/true rc=0 · `keydraw_lgt` 동일 플래그 PASS/52/true rc=0

### ★부하로 «FAIL» 이 났다 — AGENTS.md 의 현 문면이 덮지 못하는 형태라 적어 둔다
첫 회차에서 `keydraw_ktf` **FAIL/13/false rc=1** · `keydraw_lgt` **FAIL/23/false rc=1** 이 났다.
그 실행은 **7분 54초짜리 wasm clippy 직후**였고 `load average 160.84` 였다. 즉시 재측 **4/4 PASS**
(ktf 29·35 · lgt 41·52 · 전건 content true · rc=0).
★**AGENTS.md 는 「count 는 floor 이고 FAIL·blank·non-zero rc 는 회귀다」라고 못박는다** — 그런데 여기서는
**부하가 FAIL 을 만들었다.** 그 문면의 측정(28~36 @ 30동시)은 «count 하락»까지만 관측했고 «verdict 반전»은
`42/42` 로 불변이었다(그 회차의 최대 동시 30). 이번 load 160 은 그 범위 밖이다.
★**내 변경의 회귀가 «아니다»라는 근거 두 개**: ⑴함께 FAIL 한 `keydraw_ktf` 는 **KTF 캐리어**이고
이 회차의 diff 는 **`wie-lgt` 밖으로 나가지 않는다**(테스트 모듈 + `pub(crate)` 2줄 + 주석) ·
⑵이 회차 착수 시 **무변경 origin/main 트리**에서 잰 기준선이 ktf·lgt 둘 다 PASS/55/true 였다.
⇒ ★후속 제안으로 뺐다(AGENTS.md 의 그 블록에 「극단 부하에서는 verdict 도 뒤집힌다」를 실측으로 넣는 일).
