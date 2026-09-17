## [2026-09-17] 「이 검사기는 어디서 도나」를 기계가 센다 — 37개 중 호출자 0이 4 · 1이 15, 그리고 ★**결함은 0** (wie-count-checkers-with-only-one-caller)

**무엇을**: 채택 제안 `2026-09-17-upstream-guard-wired#p0` 의 이행. `scripts/checker-census.mjs` 신설 +
`engine-contract.yml` 의 **상시 스텝**으로 배선(★`continue-on-error: true` — **판정을 만들지 않는다**) +
`AGENTS.md` §Web-surface commands 에 진입점 1문단. ★**제품 동작 0줄 · Rust 0줄 · 기존 검사기 무접촉 ·
기존 배선 변경 0.**

**왜**: 「이건 어디서 도나?」가 **파일마다 손 `git grep`** 이었다. 이 저장소는 그 질문에 두 번 답했는데
둘 다 **한 검사기를 위한 전용 가드**였고(`check-parity-lock-wired.mjs` 2026-09-05 ·
`check-upstream-guard-wired.mjs` 2026-09-17), `AGENTS.md` 는 세 번째를 **산문**으로 답한다
(「Measured 2026-09-06 across all 8 workflow files」) — 워크플로가 움직이는 순간 낡는 측정이다.

**사용자 영향**: 없다(제품 0줄). PR 마다 **~2.8초** node 스텝 1개.

---

### ⑴ 수 — ★**a 와 b 를 따로** (Acceptance 1)

| 트리 | N | 호출자 **0 = a** | 호출자 **1 = b** | 2+ |
|---|---|---|---|---|
| base `f7a1d022`(이 회차 «전») | **36** | **4** | **14** | 18 |
| 이 회차 후 | 37 | 4 | 15 | 18 |

★**+1 은 `checker-census.mjs` 자신**이고 **버킷 1**에 들어간다 — ★**이 회차가 새 고아를 만들지 않았다는
것이 그 줄의 뜻이다**(그 실패형이 정확히 이 티켓의 맥락 ⒜⒝⒞ 다). 모집단·표는 도구 출력이 정본이고,
`node scripts/checker-census.mjs` 로 언제든 재현된다.

**★`scripts/check-*.mjs` 만 따로 세면 13개 중 «단일 호출자 9 · 0 호출자 1 · 2+ 3».**
⇒ 제안이 스스로 건 임계(「2~3개면 지금 형태(개별 가드)가 더 싸다」)는 **수로는 넘었다**.
★**그런데 그 수로 결정하면 안 된다 — 아래 ⑵-ⓑ 가 그 이유다.**

### ⑵ ★대전제 ⓑ — 「호출자 1」은 **결함이 아니다**. 이 트리에서 **19건 중 결함 0건**

호출자 0·1 인 **19행 전건**을 손으로 분류했다(★도구는 이 열을 갖지 않는다 — 티켓 ⓑ 그대로).

**호출자 0 (4건) — 검사기는 2건이고, 둘 다 «사유가 적혀 있다»**
| 파일 | 검사기인가 | 판정 |
|---|---|---|
| `scripts/check-branch-protection-claim.mjs` | ○ | ★**일부러 미배선**. Actions `github.token` 이 protection·rulesets 에 **403**(run 35181122022 · 스텝 rc=2 = 「못 쟀다」) · `permissions: administration: read` 는 **부여 불가 키**라 워크플로가 파싱 단계에서 죽는다(run 35180786771). 사유가 `doc-liveness.yml` 꼬리 주석에 있다 ⇒ **정상** |
| `scripts/smoke_gate.sh` | ○ | ★**구조적으로 CI 불가** — 코퍼스가 게임 바이트(Constraint 9) · `AGENTS.md` 가 「local only, and structurally so」로 이미 선언 ⇒ **정상** |
| `scripts/lgt_render_probe.sh` | ✕ | 외부 zip 을 받는 «측정 하네스» — 합격/불합격 술어가 없다 |
| `scripts/make-wipi-keydraw-fixture.sh` | ✕ | 픽스처 «생성기» · 산출물은 커밋돼 있다 |

**호출자 1 (15건) — 「하나면 충분」이 14건, 나머지 1건은 이 회차의 도구**
- ★**PR 게이트 8건**(`check-doc-liveness-parity`·`check-engine-runner-fixtures`·`check-linux-system-deps`·
  `check-parity-lock-wired`·`check-parked-workflows`·`check-upstream-guard-wired`·`check-worklog-json`
  + `checker-census`) — 전부 `engine-contract.yml` 의 **상시 스텝**. ★**게이트의 호출처가 하나인 것은 설계다.**
- **주기형 2건**: `check-audit-warnings`(`rust-audit.yaml` = `schedule,workflow_dispatch` = **일 1회 크론 ·
  PR 게이트가 아니다**) · `check-upstream-new-workflows`(`doc-liveness.yml` 주간 — 그 파일 머리주석이
  **주간 cadence 를 일부러 골랐다**고 적는다).
- **배포용 3건**: `.github/scripts/release/*.sh` — 호출처 `release.yaml` 이 ★**`workflow_dispatch` 단독(parked)**
  ⇒ 사람이 누르기 전엔 안 돈다. `check-parked-workflows.mjs` 가 그 상태를 **의도로 잠그고 있다** ⇒ **정상**.
- **검사기 아님 2건**: `make-resize-fixture.mjs`(생성기) · `wie_cli/tests/support/dod_ci_parity.rs`
  (검사기 «몸통» · `#[path]` 로 1곳에서 결합되는 것이 설계이고 `check-parity-lock-wired` 가 그것을 잠근다).

⇒ ★★**a+b 19건 중 «고쳐야 할 것» 0건.** 단일 호출자를 술어로 red 를 만들면 **9건 전부 오탐**이고,
그것이 이 저장소가 반복해 규탄한 「아무도 안 보는 red」다. ⇒ ★**차단을 만들지 않은 것은 계약 준수이자
측정 결과다**(계약 3⒜).

★**그렇다면 두 개의 전용 가드는 왜 정당했나** — 그 둘의 술어는 「호출자가 1인가」가 **아니다**.
「**그 하나를 지워도 아무것도 red 가 되지 않는가**」다. 그 축은 별 회차
(`wie-count-deletable-checks-that-stay-green-repo-wide`, PR #94 · A=9·B=23·차집합 14)가 이미 세었고,
★**이 회차의 수와 그 회차의 수는 «곱해야» 가드 판단이 된다** — 어느 한쪽만으로는 안 된다.
⇒ 세 번째 가드 제안은 **수가 아니라 그 결합 술어**로 써야 한다(worklog 제안 카드 #p0).

### ⑶ ★세는 방식과 그 천장 (Acceptance 2)

**방식 = 배선에서 역방향**(계약 1 권고 그대로). 이름 규칙(`check-*`)에 기대지 **않았다** —
그 규칙 밖 검사기가 실재한다(`audit-no-leak.sh`·`verify-browser.mjs`·`smoke_gate.sh`), 반대로
규칙 «안»에도 검사기가 아닌 것이 있다(`check-docs-report-serial.mjs` 는 주로 일련번호 할당기다).
⇒ 모집단은 **위치+확장자**, 호출자는 **실행 위치를 읽어서** 센다.

**★호출자 판정에서 걸러낸 두 가지 — 이 둘을 안 걸렀으면 수가 틀린다**
- **주석**: `contract-roundtrip.mjs` 는 시험 파일 이름을 주석에서 부르고(선행 회차가 같은 함정을 기록),
  `doc-liveness.yml` 의 `# NOT-RUN:` 은 **안 도는 명령을 이름으로 적는 줄**이다.
- ★**«데이터로서의 경로»**: `check-parity-lock-wired.mjs` 의 `test:`/`checker:` 상수 ·
  `check-upstream-guard-wired.mjs` 의 `GUARD.script` · `check-engine-contract.mjs` 의 오류 문구
  「run scripts/build-wasm.sh first」. 실측: 단순 부분문자열이면 **호출자 0이 3 · 1이 11** 로 나오고
  **7행이 잘못된 버킷에 앉는다**. ⇒ 호출은 **실행 형태**(`bash|sh|node|npx|python`·`./`·
  `import/require`·`execFileSync` 류·`#[path]`)로만 세고, 나머지는 **`named-not-run` 열**로 따로 낸다
  (그 열도 버린 정보가 아니다 — 이름을 바꾸면 그 자리들이 깨진다).

**★글롭 호출자는 «규칙으로 선언»했다** — `cargo test --all` / `cargo tarpaulin --workspace` 는
`*/tests/*.rs` 를 **이름 없이** 문다. 실측: 그 규칙을 빼면 **호출자 0이 4 → 14**, 즉 규칙 하나가
**36행 중 10행**을 지탱한다. 도구가 이 수를 **기본 출력에 항상 찍는다**(플래그 뒤에 숨기지 않았다 —
숨기면 인용하는 사람이 어느 모드의 수인지 모른다).

**★천장 — 무엇을 «못 보는가»**
1. ★**워크플로 «인라인» 검사는 모집단 밖**이다(파일이 아니다). 실측 **6종·12개 호출처**:
   `cargo fmt --all -- --check` ×2 · `cargo clippy --all -D warnings` ×2 · wasm clippy ×2 ·
   `cargo test --all` ×3 · `cargo tarpaulin` ×1 · `cargo audit` ×2. ⇒ 이것들은 **정의가 곧 호출처**라
   언제나 「호출자 1」이고, 세어도 정보가 없다.
2. ★**소스 «안»의 `#[cfg(test)] mod tests` 는 모집단 밖** — 실측 `.rs` **101파일**(그중 `*/src/` **96**).
   같은 글롭이 줍지만 파일 단위가 아니라 **모듈 단위**라 파일 술어로 못 센다.
   ※선행 회차가 같은 천장을 **31파일**로 적었다 — upstream base swap 후 **101**이다.
3. **`uses:`/`with:` 의 액션 입력은 호출이 아니다** — `dorny/paths-filter` 가
   `scripts/make-wipi-keydraw-fixture.sh` 를 **트리거 경로**로 들고 있으나 «실행»은 아니다.
   ⇒ `named-not-run` 으로 낸다. 재사용 워크플로·composite action 은 이 트리에 **0건**(실측)이라 손실 없다.
4. ★**cadence 는 워크플로 «전체»의 `on:` 이지 스텝 조건이 아니다** — `doc-liveness.yml` 의
   `pull_request` 는 **자기 파일에만 걸린 paths 필터**인데 표기는 `[schedule,workflow_dispatch,pull_request]`
   로 나온다. `if:` 조건(`npm run verify` 의 `github.event_name == 'schedule'`)도 안 읽는다.
   ⇒ ★**cadence 표기는 «상한»이다 — 「이보다 자주 돌지는 않는다」로만 읽어라.**

### ⑷ 양방향 — 배선을 끊으면 **나타난다** (Acceptance 3)

격리 워크트리(`git worktree add --detach` · 커밋 0)에서 **배선 3개만** 끊고 재실행:

| 대상 | 끊은 것 | before | after |
|---|---|---|---|
| `scripts/check-worklog-json.mjs` | `engine-contract.yml` 의 **유일한** run 줄 | 버킷 **1** | ★**버킷 0** |
| `scripts/audit-no-leak.sh` | `engine-contract.yml` run 줄(둘 중 하나) | 버킷 **2+** | ★**버킷 1** |
| `scripts/verify-browser.mjs` | `package.json` 의 `verify` 별칭 | 버킷 **2+** | ★**버킷 1** |

요약 줄도 함께 움직였다: `0 = 4 · 1 = 15 · 2+ = 18` → ★**`0 = 5 · 1 = 16 · 2+ = 16`**.
★워크트리는 `git worktree remove --force` 로 제거 · 본 트리 `git status` 클린.
★**«red 축»은 없다** — 이 회차는 차단을 만들지 않으므로 억지로 만들지 않았다(티켓 지시 그대로).

### ⑸ ★`bin/suite-orphan-check` 를 빌렸나 (Acceptance 4)

**빌린 것 = 두 가지 · 못 빌린 것 = 두 가지.** 도구 자체는 **못 빌린다**(다른 트리·다른 소관이기도 하고,
아래 ⒞ 가 구조적 이유다).

- ⒜★**«판정을 재구현하지 않는다»를 빌렸다.** 그 도구는 스위트에서 판정 블록을 **런타임에 떼어** 쓴다.
  여기서 대응물은 **워크플로·`package.json` 자신**이다 ⇒ 배선 목록을 상수로 **복제하지 않고** 매번 읽는다.
  ★그래서 워크플로가 움직이면 수가 **저절로** 따라간다(`AGENTS.md` 산문이 못 하는 바로 그것).
- ⒝★**«못 쟀다»를 «깨끗하다»로 읽지 않는다를 빌렸다** — rc=2 = UNMEASURABLE(git 부재·모집단 0) ·
  읽지 못한 표면은 `★UNREAD` 로 찍고 **「아래 수는 하한」**이라고 말한다.
- ⒞★**«3택 처방»(등재/장부/머리주석 선언)은 «일부러» 안 빌렸다.** 그 셋은 **게이트를 조용히 시키는 장치**이고
  이 산출물은 **게이트가 아니다**. 면제 목록을 두면 **분류가 사는 곳이 둘**이 되는데, 그것이 이 저장소가
  반복해 이름 붙인 실패형이다. ⇒ ★**판정은 사람에게 남겼다**(계약 ⓑ).
- ⒟**`_T90_LIST` 같은 «기대 목록»도 안 빌렸다** — 그 도구의 모집단은 «스위트가 부르기로 한 것»인데,
  여기엔 그런 정본 목록이 **존재하지 않는다**(정본은 워크플로 자신이다).

### ⑹ 차단 0 · rc 변경 0 (Acceptance 5)

- 도구는 **rc=1 이 없다** — 실패 상태가 없으니 fail-open 도 없다(rc 0 / 2 뿐).
- CI 스텝에 ★**`continue-on-error: true`** — `engine-contract.yml` 의 `contract` 잡은 이 저장소가
  **required check** 로 적어 둔 잡이라, 그 플래그가 「rc 변경 0」을 **주장이 아니라 기계**로 만든다.
  선행 형제(`check-engine-runner-fixtures`)가 같은 자리에 같은 이유로 그 플래그를 쓴다.
  ★**승격은 그 한 줄 삭제**이고, 그것은 별 티켓 몫이다.
- **기존 검사기 배선·삭제 0** · **`bin/suite-orphan-check` 무접촉**(다른 트리).

### ⑺ 게이트

`cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm clippy ·
★`cargo +beta clippy --all -- -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` —
결과는 회신(`reports/wie-count-checkers-with-only-one-caller.done.md`)에 수치로 적었다.
노드 검사기 **8종 전건 rc=0**(`check-doc-liveness-parity`·`check-parked-workflows`·
`check-linux-system-deps`·`check-worklog-json`·`check-upstream-guard-wired`·`check-parity-lock-wired`·
`check-engine-runner-fixtures`·`check-worklog-coverage` **10/10=100% · 착지 120 · 최근 기록 114 ⇒ 미도래**).
★`AGENTS.md` 편집은 **fenced `sh` 블록을 늘리지 않았다** ⇒ `check-doc-liveness-parity` 의 의무
(문서↔`doc-liveness.yml` 복사 상등)가 늘지 않는다 — 실측 `25 documented line(s)` 불변.
