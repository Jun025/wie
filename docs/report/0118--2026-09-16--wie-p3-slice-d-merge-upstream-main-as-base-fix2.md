## [2026-09-16] 조각 D 게이트② 반려 처분 — 「동작 회귀」는 없었고, 두 번째 호스트가 안 서 있었다 (wie-p3-slice-d-merge-upstream-main-as-base-fix2)

**무엇을** — 게이트② `request-changes` 4건과 형제 조각 E 의 F1 을 한 회차에 닫았다. ★**제품 코드 회귀는
없었다** — 직전 회차가 「실제 동작 회귀(후보 = WIPI 키코드 매핑)」로 **영구 기록 4곳**에 박은 것은
★**그 회차가 «직접 쓴» 시험의 루프 탈출 조건 1줄**이었다. 그리고 ★**두 번째 호스트
(`wie_featurephone` = 브라우저)는 base swap 이후 통째로 빌드가 깨져 있었다**(`npm run build:wasm` rc=101).

**왜** — 틀린 진단은 그 자체로 **없는 버그를 쫓는 회차 1개**(worklog 제안 `#p0` · effort M)를 발권시킨다.
F1 은 더 나쁘다: 조각 E 가 잰 대로 ★**CI 가 웹 축을 한 번도 재지 않으므로** 고치지 않고 착지하면
`main` 에 «빌드가 깨진 웹 표면»이 **조용히** 올라간다.

**사용자 영향** — 브라우저에서 게임을 켜는 표면이 **다시 빌드된다**(그 전에는 아티팩트가 아예 안 나왔다).
MIDP 텍스트를 그리는 타이틀은 폰트가 붙고, 게스트가 화면 크기를 지정하는 타이틀(KTF ADF `display_size`)은
캔버스가 따라간다. 네이티브 CLI 는 무접촉.

### 한 일 — 일곱 항 + 착지를 막던 둘

| # | 내용 | 무는 것 |
|---|---|---|
| ⑴ | `git merge origin/main`(리베이스 0) · `STATE.md` 1파일 충돌을 **합집합**으로 | 양방향 잔여 **0**(각 부모의 줄이 해소본에 전건 존재) |
| ⑵ | `test_key_reach` 탈출 조건 **1줄** 교정 | 종전 **FAILED** ↔ 교정 **ok. 1 passed** |
| ⑶ | 「동작 회귀」 서술 **4곳** 정정 + 제안 `#p0` 철회 | 리터럴 잔여 **0**(이 회차의 정정 문구 제외) |
| ⑷ | `27 27` → **`35 27`** **4곳**(티켓은 3곳이라 했다) | `git diff --numstat upstream/main` 실측 |
| ⑸ | 연번 `0115` → **`0117`** 재번호 + 포인터 3곳 | 중복 **0** · `--next-serial` 이 준 값 |
| ⑹ | **F1** — `wie_featurephone` 을 새 트레이트 4축에 맞췄다 | `npm run build:wasm` **rc=101 → rc=0** |
| ⑺ | `hardening.rs` 주석 썩음(2건 → **3건**) · `git worktree prune` | 등록 워크트리 2개(전부 정당) |
| ★+ | `check-engine-contract.mjs` 로케이터 2 + 계약 JSON `file` 핀 2 | ENOENT 크래시 → **107 pass · 0 violation** |
| ★+ | 워크플로 `paths` 의 죽은 `fonts/**` → `assets/**`(2파일 동기) | Constraint 4 |

### ⑵ — 「1 failed」의 정체. 두 갈래 «어느 쪽도» 아니었다

```
git rev-list --parents -n1 36df9c31 → 36df9c31 ac6e0705(우리) 44fbf265(upstream)
git log --all --oneline -- wie-ktf/tests/test_key_reach.rs → 36df9c31 «한 줄»
⇒ 두 부모 «모두»에 대해 신규 추가 = «회귀할 이전»이 없다
```

★**근인**: `seen.contains("key:")` 는 ★**접두사만 담긴 stdout write 에서 이미 참**이라 게스트가 숫자를
쓰기 «전»에 break 하고 잘린 버퍼로 assert 한다. 내 실측 재현:

| 형상 | 결과 |
|---|---|
| 종전 `seen.contains("key:")` | **FAILED** · `guest stdout was "res:9:602\nkey:"` |
| 교정 `seen.split("key:").nth(1).is_some_and(\|t\| t.contains('\n'))` | ★**ok. 1 passed** |
| 교정판에 `println!` 을 붙여 실물 확인(찍고 즉시 제거) | ★**`REVIEW-PROBE guest stdout = "res:9:602\nkey:53\n"`** |

⇒ ★**NUM5 는 WIPI 코드 53 으로 도달해 있다 — 제품은 멀쩡하다.** 제품 코드 **0줄**.
★**반증은 직전 회차가 이미 손에 쥐고 있었다** — 같은 리포트에 적힌 `keydraw_ktf` **PASS(paints 55)** 이고
그 픽스처는 «코드 폭만큼 막대를 그린다» ⇒ 코드가 안 닿으면 그 막대가 없다.

★★**그리고 같은 결함이 «형제 시험»에 하나 더 있었다 — base 를 당기자 드러났다.**
`test_resource_reach`(별 티켓 `wie-system-class-loader-spi-resource-fixture` 로 `origin/main` 에 착지)가
`seen.contains("res:")` 로 끊는다. ★**구 base 에서는 PASS 하고 새 base 에서 FAIL 한다**(격리 워크트리로 대조):
`guest stdout was "res:"`. ⇒ **술어가 언제나 경쟁 상태였고, upstream 이 stdout write 를 쪼개 그것을 노출했다.**
⇒ ★**같은 1줄 교정**을 함께 했다(ponytail 근인 규율 — 티켓이 지목한 한 곳만 고치면 형제가 깨진 채 남는다).

★★**수 정정 — `cargo test --all` 은 «384 passed · 0 failed» 다.** 티켓이 기대한 `202`, 검수자가 잰 `201`,
직전 회차의 `201` 은 전부 ★**`cargo test` 가 «첫 실패 타깃»에서 멈춘 부분 계수**다
(`^test result` 행 **14 ↔ 45** · `--no-fail-fast` 재측 **384/0** 동일). ★**「상수로 인용하지 말고 그때 세라」가
이 자리에서 세 번 연속 적용됐다.**

### ⑹ — F1. `wie_cli` 가 아니라 **upstream 자기 호스트**가 선례였다

12오류 · 4파일 · 4축. ★**트레이트 정의 무접촉**(upstream 것이다). 티켓은 `wie_cli` 어댑터를 선례로 지목했는데,
★**그 트리에는 더 가까운 선례가 이미 있다** — upstream 의 `wie-web` 크레이트가 members 에 **공존**하며
**같은 4축을 새 트레이트에 대해 이미 구현**한다. 그쪽을 읽고 맞췄다.

| 축 | 오류 | 처분 |
|---|---|---|
| `Platform::font` | E0046 | `assets/neodgm.ttf` 를 **`include_bytes!`**(upstream `src/lib.rs:174` 와 같은 형태). ★**JS 생성자에 인자를 더하지 않았다** — 더하면 계약이 핀한 export 표면이 바뀐다(Constraint 3). `wie_cli` 의 `unimplemented!()` 는 **채택하지 않았다**: 헤드리스는 텍스트를 안 그리지만 브라우저는 그린다 ⇒ MIDP 텍스트 타이틀이 패닉한다 |
| `Screen::resize` | E0046 | 앞·뒤 캔버스를 **함께** 옮기고 치수는 `Cell`(서명이 `&self`). ★**no-op 으로 두지 않았다** — `wie-ktf/src/emulator.rs:71`(ADF `display_size`)과 `wie-lgt/…/graphics.rs:195` 가 실제로 부른다 |
| `AudioSink` | E0046 `send` + **E0407 ×5** | 5메서드 → `send(AudioCommand)`. ★**기존 WebAudio PCM 경로를 보존**했고(지우면 동작 회귀다) 이벤트 `time` 을 스케줄 오프셋으로 쓴다 |
| `DatabaseRepository` | E0046 `usage` + **E0050 ×3**(인자 4↔3) | `&System` 제거(시그니처 변경이 아니라 «구현을 새 시그니처에») + `usage` 신설 |

★**대가 — 어댑터가 두 벌이 됐다**(`wie_cli` · `wie_featurephone`). ★**공통화하지 않았다**: 두 호스트의 정답이
실제로 다르고(헤드리스 `font` = 불요 ↔ 브라우저 = 필수 · 헤드리스 `resize` = no-op ↔ 브라우저 = 캔버스 이동),
묶으면 upstream 이 트레이트를 또 바꿀 때 **한 번에 둘 다** 깨진다. 선례도 «두 벌»을 택했고 upstream 자신은 **세 벌**이다.

★**남긴 천장(`ponytail:` 주석으로 명시)**: `AudioCommand::Stop` 과 `repeat`, MIDI 이벤트는 **버린다** —
★**이 호스트가 그 API 이전에도 하지 않던 것들**이다(MIDI 는 언제나 무음 스텁이었고 stop·loop 은 개념 자체가 없었다)
⇒ **회귀가 아니라 «그대로»다.** 업그레이드 경로는 upstream 의 JS 측 `AudioPlayer`(`midi.ts`)이고 그것은
**JS 표면 추가**이지 어댑터 변경이 아니다.

### ★+ 착지를 막던 둘 — 티켓 밖이지만 «이 회차 base swap 의 잔재»다

★**티켓은 F4 를 조각 E 소관으로 못박았고, 나는 그 울타리를 «절반만» 넘었다 — 이유를 수로 적는다.**

```
node scripts/check-engine-contract.mjs → ENOENT  wie_midp/src/classes/net/wie/event_queue.rs
```
⒜`contract` 는 ★**`main` 의 required status check** 다(`engine-contract.yml` 헤더·사건 대장 양쪽이 명시).
⒝내 diff 는 `**/*.rs` 를 건드리므로 그 job 의 `engine` 필터가 **발화한다**.
⇒ ★**그 크래시를 두면 이 PR 은 «게이트③이 머지할 수 없는» 상태로 제출된다**(직전 회차에서 안 보였던 이유는
#161 이 `CONFLICTING` 이라 `pull_request` 워크플로가 **애초에 안 돌았기** 때문이다 — 조각 E 의 F3).

★**전수 census 로 세 곳을 찾았다**(E 의 F4 는 「낡음 2」로 셌는데 ★**셋이다** — 세 번째는 **스크립트가 아니라
계약 JSON 안**에 있어 문자열 grep 에 안 걸린다):

| 자리 | 낡은 값 |
|---|---|
| `check-engine-contract.mjs:111` | `wie_midp/src/classes/net/wie/event_queue.rs` |
| `check-engine-contract.mjs:146` | `wie_wipi_java/src/classes/net/wie/card_canvas.rs` |
| ★`featurephone-engine-contract.json` `gameActionTables.{midp,wipi}.file` | `wie_midp/…/canvas.rs` · `wie_wipi_java/…/display.rs` |

⇒ **크레이트 개명(`wie_x` → `wie-x`)의 잔재 = 이 회차 base swap 의 산물**이므로 여기서 닫았다.
★**E 의 나머지 절반은 «손대지 않았다»** — 「실패가 위반 1줄이 아니라 스택 트레이스라 «검사기 고장»으로 읽힌다」는
**오류 처리 설계**이고 그쪽 소관이다. ★**결과**: `107 pass, 0 violation(s)` = ★**구 base 에서 E 가 잰 수와 동수**
⇒ **export 표면은 표류하지 않았다**(Constraint 3 은 같은 PR 에서 충족).

★**둘째** — 두 워크플로 `paths` 의 `fonts/**` 가 **죽은 줄**이다(그 디렉터리 추적 파일 **0** · 코드 참조 **0**).
그리고 ⑹이 `assets/neodgm.ttf` 를 **아티팩트 입력으로 만들었다** ⇒ 그대로 두면 폰트 교체가
`publish-artifact` 를 **발화시키지 못한다**. `assets/**` 로 바꾸고 **두 파일을 동기**시켰다(Constraint 4).

### ⑺ — `hardening.rs` 는 «둘»이 아니라 **셋**이 썩어 있었다

⒤`_ => 0` 위 고아 주석이 「one-shot `schedule` 이 핀에 없다」 — ★그 분기를 **삭제한** 측정의 정반대.
⒥잠금 시험 「StringBuffer carries two」 ↔ 바로 아래 `expected = 1`.
★⒦**모듈 헤더도 같은 썩음이다**(검수 지적에 없었으나 **같은 사실·같은 파일**): 「absent method 둘을 re-add 한다 ·
**1 of 9** · **3 of 5**」 ⇒ 핀 실소스 실측 — `Timer` 는 `schedule` **4/4**+`cancel`(`timer.rs:23`) ·
`StringBuffer` 는 `insert` **12개**(`string_buffer.rs:116` · Aroma 의 **9**보다 많다) ⇒ ★**지금 re-add 하는 것은 «0» 이다.**
★이 모듈의 계약이 「핀이 움직이면 **조용히 사라지지 말고 시끄럽게 실패하라**」이므로 주석이 사실과 어긋나는 것은
**그 계약 자체의 침식**이다. 코드 **0줄** · 16 passed 불변.

### ⑸ — 연번은 `0116` 이 아니라 **`0117`** 이다

```
node scripts/check-docs-report-serial.mjs --next-serial
→ 디스크 기준 0116 · 열린 PR claim [0114(#161) 0115(#161) 0116(#162)] ⇒ 0117
```
★**티켓이 적은 `0116` 은 «그 시점의» 값이고, 형제 PR #162(조각 E)가 그 사이 0116 을 claim 했다.**
⇒ ★**같은 결함이 한 계단 위에서 재발한 것**이고, 그래서 티켓 ⓑ 가 「손으로 세지 마라 · 도구에 물어라」라고
못박은 그대로 **도구의 답을 썼다**. 착지한 `0115`(조각 A)는 **건드리지 않았다**(`AGENTS.md`: 착지한 파일을
재번호하지 마라 — 옮기는 것은 미착지 쪽이다). 이 리포트는 `0118`.

### 게이트

```
cargo fmt --all -- --check                                      rc=0
cargo clippy --all -- -D warnings                               rc=0
cargo clippy --target wasm32-unknown-unknown -- -D warnings     rc=0
cargo +beta clippy --all -- -D warnings                         rc=0
RUST_MIN_STACK=4194304 cargo test --all                         rc=0   ★384 passed · 0 failed
npm run build:wasm                                              rc=0   ★F1 의 판정식(직전 rc=101)
node scripts/check-engine-contract.mjs                          rc=0   107 pass · 0 violation
npm run audit                                                   rc=0
check-docs-report-serial(--selftest·bare) / check-worklog-json / check-worklog-coverage
  / check-doc-liveness-parity / check-parity-lock-wired               전건 rc=0
5픽스처: draw_j2me PASS(1/content) · helloworld_ktf PASS(0) · helloworld_lgt PASS(0)
         keydraw_ktf PASS(55·content·rc0) · keydraw_lgt PASS(53·content·rc0)
```

★`keydraw_lgt` **53** 은 `AGENTS.md` 가 실측한 대역(28~55) 안이고 **판정축은 `result`·`content`·rc** 다.

### 개악 대조 — 셋 다 양방향

| 대상 | 개악 | 정상 |
|---|---|---|
| ⑵ 시험 탈출 조건 | 종전 `contains("key:")` → ★**FAILED** | 교정 → **ok. 1 passed** |
| ⑹ 어댑터(`Screen::resize` 제거) | ★**`npm run build:wasm` rc=101** · `E0046 missing: resize` | 복원 → **rc=0** |
| ⒠ LGT 배선 27줄 역치환(★**제품 호출부**) | ★**FAIL · paints 0 · content false · rc=1** | **PASS · paints 55 · content true · rc=0** |

★역치환 개악은 픽스처 사본이 아니라 실제로 분기하는 `wie-lgt/src/runtime/wipi_c.rs` 에 넣었고,
복원 후 `git status` **빈 값** · `HEAD` **불변** · 신 술어 **27** 재확인.
★★**그 개악의 자기 numstat 이 정확히 `27 27` 이다** ⇒ ★**직전 회차의 `27 27` 이 «어디서 왔는지»가 이것으로 설명된다** —
`git diff upstream/main`(=`35 27`)을 잰 것이 아니라 **치환 자체의 diff** 를 옮긴 것이다.

### ★★새로 찾았다 — base swap 이 upstream 워크플로 둘을 들여왔고, 하나는 «나갈» 채비였다

★**CI 를 읽었더니 28초 만에 `web_ci (stable)` 가 red 였다** — `npm error Missing script: "build:dev"`.
★**아무도 이것을 볼 수 없었다**: #161 이 `CONFLICTING` 이던 동안 `pull_request` 워크플로가 **한 번도 안 돌았다**
(조각 E 의 F3 가 그 기전을 적었다) ⇒ 게이트② 검수자도, 직전 회차도 못 봤다. 충돌을 푼 **첫 실행**이 이것이다.

| 파일 | 어디서 왔나 | 무엇 |
|---|---|---|
| `.github/workflows/web.yaml` | ★**upstream 만**(구 `main` 에 없다) | `npm run build:dev` — ★우리 `package.json` 에 **없는** 스크립트 |
| `.github/workflows/release.yaml` | ★**upstream 만** | ★**야간 cron `17 0 * * *`** + `pages deploy --project-name=wie`·`wie-dev`(★**우리 토큰으로**) + 이 repo 에 **GitHub 릴리스 발행** |

★★**`release.yaml` 이 오늘 아무것도 배포하지 않은 것은 «가드»가 아니라 «운»이다** — `web` job 이
`npm run build:prod`(역시 부재)에서 죽고 나머지 전 job 이 그것을 `needs:` 한다. ★**나머지 기계는 전부 실재한다**
(`.github/scripts/release/*.sh` **3건** · `wie-app`(`Cargo.toml` members 5행) · favicon) ⇒
★**`package.json` 에 스크립트 한 줄이 생기면 사슬 전체가 무장된다.** 우리 Pages 프로젝트는 **`wie-web`** 이고
`wie`·`wie-dev` 는 upstream 것이며, 릴리스는 **`publish-artifact.yml` 이 이미 소유**한다
(otterpebble 이 `repository_dispatch` 로 소비) ⇒ ★**한 repo 에 릴리스 발행자가 둘**이 된다.

⇒ ★**처분 = «주차»다 — 채택도 삭제도 «아니다».** 두 파일의 트리거를 **`workflow_dispatch` 만으로** 줄였다
(cron 잔여 **0** · YAML 파싱 확인 · 파일 본문 무접촉). ★**왜 삭제하지 않았나**: 「upstream 워크플로 중 무엇을
채택하나」는 ★**P3 계획이 «묻지 않은» 질문**이고, `-fix2` 회차가 제품 결정을 대신 내릴 자리가 아니다.
★**되돌리기는 트리거 두 줄**(원본 = `git show upstream/main:.github/workflows/{web,release}.yaml`).
★**왜 그냥 보고하고 두지 않았나**: 그러면 ★**착지와 동시에 야간 cron 이 우리 자격증명으로 무장된다** —
경계 ⓐ-2(비가역·외부 노출 작업)와 이 티켓 자신의 F1 논거(「고치지 않고 착지하면 조용하다」)가 같은 방향을 가리킨다.
★**결정은 후속 제안으로 발행**했다(`#p3`) — 그래야 «아무도 소유하지 않은 red» 로 남지 않는다.

### 한계 — 숨기지 않는다

- ★**착지 순서**: 이 PR 이 **먼저**다(조각 E 의 F1 이 이 회차로 들어온다). ★#162 는 **구 base 기준 측정 회차**이고
  이 PR 을 **피검체**로 삼았으므로 **서로 브랜치 무접촉**으로 뒀다.
- ★**F1 의 「12」는 첫 크레이트에서 멈춘 수**라는 E 의 한계는 **해소됐다** — `npm run build:wasm` 이 rc=0 이므로
  하류(`wasm-bindgen`·`wasm-opt`)까지 실제로 통과한다. ★**다만 `npm run frontend`(`tsc -b`·`vite build`)는
  이 회차가 돌리지 않았다** — TS 측 소비 코드는 무접촉이지만 **미측정이다**.
- ★**아티팩트가 커졌다**: `wie_web_bg.wasm` **9,205,987B(구 base · E 실측) → 15,425,955B**(+67.6%).
  ★**이 회차가 만든 것이 아니라 base swap 이 만든 것**이고(내 delta 는 폰트 665,904B 가 상한),
  ★**소비자 다운로드 비용이므로 기록한다.** 판정·처분은 이 회차 범위 밖이다.
- ★**브라우저에서 실제로 돌려 보지 않았다** — `contract-roundtrip.mjs`(Playwright)는 툴체인 fetch 가 필요해
  돌리지 않았고, 그것이 **CI 의 몫**이다. ⇒ `Screen::resize`·`Platform::font` 의 **실화면 검증은 미측정**이다.
- ★**MIDI·`Stop`·`repeat` 은 여전히 무음/무동작**이다(위 `ponytail:` 천장). 회귀는 아니지만 «구현»도 아니다.
- ★`STATE.md` §완료의 **공유 삽입점**은 그대로다 — `AGENTS.md` 가 「알고 남긴 값」으로 못박은 자리이고
  이 회차도 그 union 편집 1회를 치렀다.
