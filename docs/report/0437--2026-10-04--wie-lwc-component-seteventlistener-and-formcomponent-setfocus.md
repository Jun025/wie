## [2026-10-04] lwc `Component.setEventListener` · `FormComponent.setFocus(Component)` — 리스너가 키·포커스를 먼저 듣고, 폼이 포커스를 옮긴다 (wie-lwc-component-seteventlistener-and-formcomponent-setfocus)

### 무엇을·왜

`docs/report/0436` §5 의 KTF 전수 짝에서 두 타이틀이 «메서드를 찾을 수 없다»로 끝났다 — `65ef7052f528`(`Component.setEventListener`) · `0c67145b11df`(`FormComponent.setFocus(Component)`). 둘 다 키가 그 화면에 닿을 때만 난다.

| 바꾼 것 | 어디 | 흉내 낸 범위 |
|---|---|---|
| `Component.setEventListener(EventListener,Object)` | `lwc/component.rs` | 등록은 `Component` 의 정적 Vector(구성요소·리스너·obj 세 칸씩). 정본 필드 `evtListener`/`evtListenerObj` 를 인스턴스에 두지 않은 이유는 기존 주석 그대로 — lwc 인스턴스 필드는 LGT AOT 하위 클래스의 필드 오프셋을 민다. null 리스너는 등록을 지운다 |
| 리스너가 먼저 듣는다 | `ShellCard.keyNotify`(셸 자신) · `ShellComponent.keyNotify`(포커스 잎) · `Component.setFocus` | 키는 `eventNotify(KEY_NOTIFY=3, 키 종류, 키, 0, obj)` — true 면 잎·포커스 이동까지 건너뛴다(javadoc «true 를 돌려주면 이벤트 처리를 하지 않으며»). 포커스 이동은 잃는 쪽 `FOCUS_NOTIFY=1, 0` · 얻는 쪽 `1, 1`(답은 읽지 않는다 — 뒤에 막을 `focusNotify` 호출이 없다) |
| `FormComponent.setFocus(Component)` | `lwc/form_component.rs` | `c.setFocus()` 를 가상 호출(게스트 재정의 유지). javadoc 에 본문이 없어 «c 에게 포커스»로 읽었다(클래스에 `cmpFocus` 가 있다). null 은 무시 |
| `com.ktf.kfc.GFormComponent` 의 부모 `ContainerComponent` → `FormComponent` | `kfc/g_form_component.rs` | `0c67145b11df` 가 이 폼에 `FormComponent.setFocus` 를 부른다. KTF 에서 수신 클래스가 상속하지 않은 메서드는 vtable 칸이 없어 **메서드를 넣은 뒤에도 주소 0 으로 뛰었다**(`Invalid memory access; address: 0`). 부모 추정(«추론이지 문서가 아니다»)을 이 호출이 확정했다 |
| 보여진 셸의 첫 포커스 | `ShellComponent.show` | 잎 중 포커스를 가진 것이 없으면 입력을 받는 첫 잎(라벨 제외)에 `setFocus`. `65ef7052f528` 의 ID 입력은 `setFocus` 를 한 번도 부르지 않고 리스너가 OK 를 듣기만 기다린다 — 이게 없으면 리스너는 등록돼도 불리지 않는다. 스스로 포커스를 준 타이틀은 그대로 |

**미구현으로 남긴 것**: `SHOW_NOTIFY`·`POINTER_NOTIFY` 전달(lwc 자식은 여기서 보여지거나 눌리지 않는다) · `focusNotify` 호출(종전대로 스텁) · `hasFocus` 는 여전히 false · `processEvent` · 텍스트 필드에 친 글자는 그려지지 않는다(lwc 위젯은 `GFormComponent` 말고 그리지 않는다 — 종전과 같다).

### 실측 — 두 타이틀이 그 화면을 넘어가는가 (release · 같은 키 레시피 · `build-slot`)

- **`0c67145b11df`** — 레시피 `OK:2 OK:3 DOWN:1 DOWN:1 DOWN:1 OK:3 WAIT:2 NUM4:1 OK:3 WAIT:2`(빈 이름 칸으로 OK 버튼까지 내려가 누른다).

  | | 결과 | 끝 |
  |---|---|---|
  | main | FAIL rc=2 · 6/10 키 | `06_OK`: `setFocus(Lorg/kwis/msp/lwc/Component;)V@240 not found from FormComponent` |
  | 이 브랜치(메서드만) | FAIL | `06_OK`: `Invalid memory access; address: 0` — 위 vtable 칸 |
  | 이 브랜치 | **PASS** · 8/8 · 예외 0 | OK 뒤 포커스가 빈 «고치» 칸으로 옮겨지고 `NUM4` 가 그 칸에 «ㄱ»으로 찍힌다 |

- **`65ef7052f528`** — 레시피 `OK:2 OK:2 LSOFT:2 NUM5:2 DOWN:2 OK:3 DOWN:1 DOWN:1 RIGHT:1 OK:4 OK:3 HASH:3 NUM1:1 NUM2:1 NUM3:1 OK:3 WAIT:3 OK:3 WAIT:2`(랭킹 → 점수보기 → `#` = ID 입력). 메뉴의 첫 커서 위치가 매 실행 달라(같은 키로 다른 아이콘에 선다) **9회씩** 돌렸다.

  | | 그 화면 도달 | 도달한 실행의 결과 |
  |---|---|---|
  | main | 2/9 | 2/2 FAIL `12_HASH`: `setEventListener(…)V not found from Component` |
  | 이 브랜치 | 2/9 | 2/2 PASS · 예외 0 — ID 칸이 포커스를 받고(`setFocus`), 숫자 3개가 칸에 들어가고, OK(-5)는 칸의 `keyNotify` 에 닿기 전에 게스트 `eventNotify` 가 가져가 셸을 `hide()` 하고 `getString` 을 읽는다 → 다음 화면(오프라인이라 «네트웍 오류» 안내) → 점수 목록 |

  도달하지 못한 실행은 양쪽 모두 PASS(다른 메뉴로 갔다). 캡처는 저장소 밖(`/tmp/lwcbase.17dN/{fb_base,fb_fix,ra3,ra9,rb5,rb7}`, sha 만 · 게임 바이트 없음).

### 나빠짐 0 — lwc KTF 타이틀 짝 (main `9c1c8c92` ↔ 이 브랜치 · 기본 27키 `--inject` · 각 2회 · 순서 교대 · 동시 ≤3)

대상: `game_lab/working/ktf` 의 lwc 참조 타이틀 sha 130종 중, 바뀐 경로에 닿을 수 있는 **23종**(`ShellComponent` · `setEventListener` · `FormComponent` 참조) 전부 + `lwc` 만 참조하는 107종에서 10종(대조). 107종이 닿는 lwc 는 `AnnunciatorComponent` 이고 그 `show()` 는 `ShellComponent.show` 를 거치지 않는다.

| | main | 이 브랜치 |
|---|---|---|
| PASS | 64 | 64 |
| UNMEASURED(`clean exit` · 같은 1종) | 2 | 2 |
| FAIL | 0 | 0 |
| Java 예외 수가 달라진 타이틀 | — | 0 |
| paints 비(브랜치/main) | — | 중앙 1.000 · p10 0.997 · p90 1.009 |

- 갈린 것은 **`33f3e7669599` 하나**, 좋아진 쪽이다: paints 265 → 275(2회 모두). 따로 캡처해 보면 main 은 12번째 키부터 끝까지 «STEP 1. 별명»에 머물고, 이 브랜치는 별명 칸이 show 때 포커스를 받아 «STEP 2. 성별» → «STEP 3. 생년(1990)»까지 간다.
- 측정 규율: 착수 `host-load-guard --status --recovered` rc=0(load 7.9) · 에뮬레이터 실행 전부 `build-slot run` · 동시 ≤3 · `nohup` 없음 · 끝에 내 프로세스 0 · 인공 부하 없음.

### 되돌리면 red (각 하나만 red)

`shell_card.rs` 시험 2종 — `event_listener_on_a_shown_text_field_hears_focus_and_keys_first` · `form_component_set_focus_on_a_g_form_moves_the_keys_to_that_box`.

| 되돌림 | red |
|---|---|
| `setEventListener` 메서드 | 앞 시험(메서드 해석 실패) |
| `show` 의 첫 포커스 | 앞 시험(`focusGained` 0) |
| 잎 키를 리스너 먼저 | 앞 시험(가져간 키가 칸에 찍힌다) |
| `FormComponent.setFocus` 메서드 | 뒤 시험(메서드 해석 실패) |
| `GFormComponent` 부모 → `ContainerComponent` | 뒤 시험(메서드 해석 실패) |

### 게이트

- `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(`1.100.0-beta.3`) · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 **687 pass / 0 fail**.
- 러너 블록: draw · helloworld ×2 · text PASS · keydraw ×2 `--inject --expect-last-frame` PASS rc=0(paints 79 · 55).

### 알고 남긴 것

- `0c67145b11df` 의 OK 버튼은 포커스가 떠난 뒤에도 빨간 오른쪽 테두리가 남는다 — main 에서도 같다(`DOWN` 으로 버튼을 돌아 나가도 남는다). 이 회차 범위 밖.
- 리스너 등록은 게스트가 지우지 않으면 구성요소보다 오래 산다(정적 Vector · 선형 탐색) — 화면당 한두 개라 `ponytail:` 주석으로 상한을 적었다.

### 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 2회 / 2쌍 · SUFFIX-ATTACHED 0회 / 0쌍. 둘 다 이 회차가 고친 파일(`shell_card.rs` · `component.rs`)에 **이미 있던** 주석 줄이다. 이 회차가 더한 줄의 게임 이름은 0이다(`git diff origin/main...HEAD` 의 `+` 줄 대조).

<!-- corpus-name-inflow v1 subjects=9 tree=d928d9b456981321 B=2/2 P=2/1 S=0/0 -->
