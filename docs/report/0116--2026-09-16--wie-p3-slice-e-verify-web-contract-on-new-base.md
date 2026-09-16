## [2026-09-16] 조각 E — 새 base 에서 웹 계약·아티팩트를 확인했다 (wie-p3-slice-e-verify-web-contract-on-new-base)

**무엇을** — 조사 전용(제품 코드 **0줄**). 채택 제안 `2026-09-16-p3-remaining-slices-plan#p4`.
★**결론: 새 base 에서 «웹 표면 전체가 서 있지 않다».** 브라우저 호스트 크레이트가 **컴파일되지 않고**,
그래서 아티팩트·계약검사·브라우저 왕복이 **하나도 돌지 못한다.** ★**고치지 않았다**(Contract 3).

**왜** — 조각 D 가 `wie_cli` 는 새 트레이트 표면에 맞췄지만 **브라우저 호스트는 맞추지 않았다**. 그리고
★**우리 4게이트는 그것을 «구조적으로 못 본다»** — 그 사실이 이 회차의 실제 산출이다.

### ★측정 대상 — «어느 트리인가»를 먼저 박는다

| | sha | 무엇 |
|---|---|---|
| ★**새 base(피검체)** | **`f533ba54`** | PR **#161** head = 조각 D 산출물. ★**착지하지 않았다**(`bin/landed …-fix` **rc=1 UNLANDED** · `mergeable: CONFLICTING`) |
| **구 base(대조군)** | **`d70b93f8`** | 오늘의 `origin/main`(조각 A 착지 직후) |

★**대조군을 «같은 명령으로» 돌린 것이 이 회차의 무는 축이다** — 구 base 가 전건 green 이므로
아래 실패는 ★**내 툴체인이 아니라 «새 base» 의 것**이다. 프로브는 격리 `git worktree` 에서 돌고 **제거**했다.

### ★F1 [차단] — `wie_featurephone` 이 새 base 에서 «컴파일되지 않는다»

```
$ npm run build:wasm            # = scripts/build-wasm.sh
error: could not compile `wie_featurephone` (lib) due to 12 previous errors     rc=101
```

**12건 · 4파일 · 트레이트 표면 4축**(전수):

| 축 | 오류 | 자리 |
|---|---|---|
| `Platform::font` | `E0046` 미구현 | `wie_featurephone/src/platform.rs:59` |
| `Screen::resize` | `E0046` 미구현 | `wie_featurephone/src/screen.rs:63` |
| `AudioSink` | `E0046` `send` 미구현 **+** `E0407` **5건**(`play_wave`·`midi_note_on`·`midi_note_off`·`midi_program_change`·`midi_control_change` 가 트레이트 멤버가 아니다) | `wie_featurephone/src/audio.rs:84-96` |
| `DatabaseRepository` | `E0046` `usage` 미구현 **+** `E0050` **3건**(`open`·`exists`·`delete` 가 인자 4개인데 트레이트는 3개) | `wie_featurephone/src/database.rs:35-49` |

★★**이것은 «새로운 종류»가 아니다 — 조각 A 검수자가 `wie_cli` 에서 이미 잰 «그 21줄 어댑터»와 같은 축이다**
(`Screen::resize` · `AudioSink 5→send` · `DatabaseRepository` · `Platform::font`).
⇒ ★**D 는 `wie_cli` 를 맞췄고 `wie_featurephone` 을 맞추지 않았다.** 빠뜨린 것은 «어려운 것»이 아니라 «두 번째 호스트»다.

**딸린 결과 — 웹 표면이 «통째로» 선다**(전부 같은 한 지점에서 죽는다):

| 검증식(계획 §E-⒟ 그대로) | 새 base | 구 base(대조군) |
|---|---|---|
| `npm run build:wasm` | ★**rc=101** | **rc=0** |
| `ls web/src/wasm/wie_web.js wie_web_bg.wasm` | ★**디렉터리 자체가 없다** | **둘 다 실재** — `wie_web.js` **26,287B** · `wie_web_bg.wasm` **9,205,987B** |
| `node scripts/check-engine-contract.mjs` | ★**rc=1**(F4 — 아티팩트 이전에 크래시) | **rc=0** — `107 pass, 0 violation` |
| `node scripts/contract-roundtrip.mjs` | ★**돌릴 수 없다**(아티팩트 부재) | **rc=0** — 시나리오 **A~F** 전건, E·F 키 6단언 포함 |
| `npm run frontend`(web/ `npm ci` → `npm run build`) | ★**rc=101**(같은 자리) | — |

### ★F2 [게이트 사각] — 문서화된 wasm 게이트가 그 크레이트를 «검사하지 않는다»

★**새 base 에서 그 게이트는 «green 이다»** — 크레이트가 깨져 있는데도:

```
$ cargo clippy --target wasm32-unknown-unknown -- -D warnings      # AGENTS.md 4게이트 중 wasm 레그
rc=0        ★검사된 패키지: `wie` «하나»뿐(= upstream 루트 패키지)
```

근인은 **패키지 선택**이다 — `--all` 이 없으므로 cargo 는 `default-members` 를 고른다:

| | `default-members` | 그 게이트가 «실제로» 보는 것 | `wie_featurephone` 포함? |
|---|---|---|---|
| 구 base | `["wie_cli"]` | `wie_cli` | ★**아니다** |
| ★**새 base** | `["."]` ← upstream 루트 `[package] name = "wie"` | `wie` | ★**아니다** |

⇒ ★★**두 base «어느 쪽에서도» 이 게이트는 브라우저 호스트를 린트한 적이 없다.**
★**「upstream 이 사각을 만들었다」가 아니다** — 사각은 **원래 있었고**, 새 base 가 그 사각 «안에» 실제 결함을 넣었을 뿐이다.
★`AGENTS.md` Constraint 6 은 이 게이트를 「`std` 로 손 뻗으면 웹 빌드가 깨진다」의 잠금으로 적는데,
★**정작 «웹 빌드 그 자체»인 크레이트는 그 잠금 밖에 있다.**
※`cargo clippy --all` (네이티브 레그)은 `wie_featurephone` 을 **본다** — 단 그 크레이트는 비-wasm 에서
**빈 라이브러리**(Constraint 7)라 ★**볼 것이 없다.** 두 레그가 «각자 놓치는» 형상이다.

### ★F3 [측정되지 않았다] — 그것을 «볼 수 있는» CI 는 #161 에서 «돌지 않았다»

★**기계가 눈이 없는 것은 아니다**: `web.yml` 의 `Build frontend (wasm + React/Vite)` 스텝은 `if:` 가 **없어서**
`pull_request` 에서도 돈다 ⇒ ★**이 결함을 잡을 수 있다.** 그런데 실측:

```
$ gh run list -R Jun025/wie --commit f533ba54…
coverage · push · completed failure          ← ★이 «한 줄»이 전부다
```
⇒ ★**`pull_request` 워크플로가 «0건»이다**(web.yml·rust.yml·engine-contract 전건 부재).
근인 = **#161 이 `CONFLICTING`** 이라 머지 ref 가 계산되지 않아 `pull_request` 가 애초에 발화하지 못한다.
⇒ ★★**새 base 의 웹 축은 «이 형상에서 측정되지 않았다».** ★**구조적 부재가 아니다** — 볼 수 있는 축(`web.yml Build frontend`)이
실재하고 `pull_request` 에서 도는데, #161 이 `CONFLICTING` 이라 **그 run 자체가 생기지 않았을 뿐**이다.
조각 D 회신의 `CI_RED … coverage` 는 «돌아간 것»에 대해
정확하지만, ★**웹 축은 red 도 green 도 아니고 «미측정»이다.**

### ★F4 [검사기가 낡았다] — `check-engine-contract.mjs` 가 옛 레이아웃 경로를 박아 둔다

새 base 에서 그 검사기는 **위반을 보고하기 전에 크래시**한다:
```
ENOENT … /base/wie_midp/src/classes/net/wie/event_queue.rs
```
경로 로케이터 **전수**(`bin/ledger-grep -nE` · 실행 경로만):

| 줄 | 로케이터 | 새 base 에서 |
|---|---|---|
| `:111` | `wie_midp/src/classes/net/wie/event_queue.rs` | ★**없다** → 실제 위치 `wie-midp/src/classes/net/wie/event_queue.rs` |
| `:146` | `wie_wipi_java/src/classes/net/wie/card_canvas.rs` | ★**없다** → 실제 위치 `wie-wipi-java/src/classes/net/wie/card_canvas.rs` |
| `:79`·`:224` | `wie_featurephone/src/lib.rs` | **산다**(우리 크레이트는 밑줄 이름을 유지했다) |

⇒ **낡음 2 · 정상 2.** ★**fail-closed 라는 점은 지켜졌다**(rc≠0) — 단 실패 형태가 «위반 1줄»이 아니라
**스택 트레이스**라, 읽는 사람이 「계약 위반」이 아니라 「검사기 고장」으로 읽는다.
★**이것은 F1 과 «독립»이다** — F1 을 고쳐 아티팩트가 생겨도 이 두 줄은 그대로 크래시한다.

### ★F5 [커버리지 삭제] — 구 base 에서 «돌던» 시험 **5건**이 스위트에서 빠졌다 (4디렉터리)

> ★**[정정 2026-09-16 게이트②]** 초판은 이 절을 「고아 `wie_midp/` **1건**」으로 적고 절 제목에 **「전수 계수」**를 달았다.
> ★**전수가 아니었다 — 5건 중 1건(20%)** 이다. 그리고 ★**귀속이 뒤집혀 있었다**: 초판은 이것을 **upstream 쪽에서 비롯한 것**처럼
> 적었으나, 실제로는 ★**구 base 에서 «우리 것»이었고 base 교체로 넷이 «동시에» 고아가 됐다.**

★**선별 술어를 먼저 적는다**(이것이 없으면 다음 회차가 또 세고 또 틀린다):
> **새 base `f533ba54` 의 top-level `wie*` 중 ⑴workspace `members` 밖이고 ⑵`Cargo.toml` 이 없는 디렉터리** = 전수.

| 디렉터리 | 새 base 추적 파일 | ★**구 base `d70b93f8`** |
|---|---|---|
| `wie_j2me` | **1** — `tests/test_boot.rs` | 추적 **4** · `Cargo.toml` · ★members **등재** |
| `wie_jvm_support` | **2** — `tests/absent_string_buffer_insert.rs` · `tests/absent_timer_schedule.rs` | 추적 **9** · `Cargo.toml` · ★members **등재** |
| `wie_midp` | **1** — `tests/create_image_missing_name_message.rs` | 추적 **40** · `Cargo.toml` · ★**암묵 member**(아래 ※) |
| `wie_wipi_java` | **1** — `tests/preload_classes_come_from_the_runtime.rs` | 추적 **59** · `Cargo.toml` · ★members **등재** |
| **계** | ★**4디렉터리 · 5파일** | — |

★**성격 = «남겨진 파일»이 아니라 «커버리지 삭제»다.** 그 5개는 구 base 에서 **실제로 돌던 통합 시험**이고,
★**로그로 확인했다** — 구 base `cargo test --all` 이 다섯 바이너리를 전건 `Running tests/…` 로 띄운다
(`test_boot` · `absent_string_buffer_insert` · `absent_timer_schedule` · `create_image_missing_name_message` ·
`preload_classes_come_from_the_runtime`). 새 base 의 `cargo test --all` 은 **더는 포함하지 않는다.**

★**이어받아지지도 않았다** — 5파일 전건이 새 base 트리에 **정확히 1회씩** 나타난다
(`git ls-tree -r --name-only f533ba54 | ledger-grep -cE <이름>` → **1·1·1·1·1**) ⇒ 하이픈 후속 크레이트가 같은 시험을 갖지 않는다.

※★**「members 등재」를 한 칸 더 정확히 적는다**: 구 base 의 명시 `members` 목록에 있는 것은 **셋**(`wie_j2me`·`wie_jvm_support`·`wie_wipi_java`)이고
`wie_midp` 는 **`[workspace.dependencies]` 의 path 의존**이다 — cargo 는 워크스페이스 디렉터리 안의 path 의존을 **암묵 member** 로 넣으므로
★**넷 다 `cargo test --all` 의 대상이었고, 위 실행 로그가 그 증거다**(「명시 등재」가 아니라 「실제로 돌았다」가 결정적 근거다).
※`wie_midp` 의 그 시험은 `AGENTS.md` 가 「`STATE.md:<줄>` 이 아니라 회차 파일을 인용하라」의 실례로 지목한 바로 그 파일이다.

### ★깨지지 «않은» 것 — 이 티켓이 실제로 물은 축

★**산출물 이름 계약은 온전하다.** 「개명됐나」가 이 회차의 1차 질문이었고 답은 **아니오**다:

| 축 | 새 base 실측 |
|---|---|
| `scripts/build-wasm.sh` | ★**`--out-name wie_web` 그대로** · 그 위 「ON PURPOSE … otterpebble 소비자 계약 · 2026-09-11 개명은 일부러 아티팩트를 안 건드렸다」 주석도 그대로 |
| `docs/contracts/featurephone-engine-contract.json` | `artifacts.files` = **`["wie_web.js","wie_web_bg.wasm"]`** · `glueFetchesWasmByName` = **`wie_web_bg.wasm`** |
| 입력 크레이트 | `WASM_IN=target/wasm32-unknown-unknown/release/**wie_featurephone**.wasm` ⇒ 크레이트명↔아티팩트명 **분리 유지** |
| upstream 충돌 | upstream 이 **자기 `wie-web` 크레이트**를 들고 왔고 우리 `wie_featurephone` 과 **둘 다 members 에 있다** ⇒ 이름 충돌 **0** |

⇒ ★**otterpebble 리시버가 «이름으로» 받는 축은 안전하다.** 오늘 깨진 것은 «이름»이 아니라 **«빌드»** 다.
★**교차 repo 조율은 필요 없다**(Contract 3 의 그 갈래가 아니다) — 필요한 것은 **wie 안의 어댑터 이식**이다.

### 사용자 영향

★**지금 당장은 없다** — 새 base 는 **착지하지 않았고**(#161 CONFLICTING · 미착지), 운영 중인 `main` 은 건강하다
(이 회차가 `d70b93f8` 에서 build·계약·왕복 **전건 green** 으로 확인했다).
★**착지했다면 있었다** — featurephone 셸에 나가는 WASM 이 **아예 만들어지지 않으므로** 웹 배포가 통째로 깨진다.

### 다음 조각에 넘기는 것

⑴★**어댑터 이식**(`wie_featurephone` 4파일) — 별 회차. ★**여기서 고치지 않았다**(Contract 3 · 「조사면 「없더라」도 산출물」).
⑵★**게이트가 브라우저 호스트를 보게 하라**(F2) — 이것을 먼저 하면 ⑴의 재발을 «기계»가 잡는다.
⑶`check-engine-contract.mjs` 로케이터 2줄(F4) · ⑷★**커버리지 삭제 5건 처분**(F5 · 4디렉터리 —
`wie_j2me`·`wie_jvm_support`·`wie_midp`·`wie_wipi_java`). ★**이식할지 버릴지는 그 회차가 정한다** — 이 회차는 정하지 않았다.
★★**⑷는 시한이 있다**: 조각 D `-fix2`(PR #161)가 착지하는 **그 순간** 그 5건이 `main` 의 스위트에서 빠진다.
★**문서 정정으로는 막을 수 없다** — 막으려면 `-fix2` 안에서 처분되거나 그 직후 회차가 받아야 한다.
★**⑴~⑷ 전부 «새 base 위»의 일이다** — `main` 에서는 오늘 할 일이 아니다.
