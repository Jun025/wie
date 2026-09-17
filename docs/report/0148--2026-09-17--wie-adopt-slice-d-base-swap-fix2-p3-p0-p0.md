## [2026-09-17] upstream 감시 가드를 «지웠는데 green» 이 되지 않게 했다 — 그리고 순진한 술어는 공허했다 (wie-adopt-slice-d-base-swap-fix2-p3-p0-p0)

**무엇을**: 채택 제안 `2026-09-17-adopt-slice-d-base-swap-fix2-p3-p0#p0` 의 이행.
`scripts/check-upstream-guard-wired.mjs` 신설 + `engine-contract.yml` 의 **상시(always-run) 스텝**으로 배선.
★**제품 동작 0줄 · Rust 0줄 · 기존 검사기 무접촉.**

**왜**: `scripts/check-upstream-new-workflows.mjs` 는 `doc-liveness.yml` 의 스텝 **하나**로만 돈다.
그 스텝을 지우면 **파일은 남고 · `git status` 는 깨끗하고 · 전 검사가 green** 이며,
바뀌는 것은 **아무도 upstream 을 안 본다**는 사실뿐이다. 이 저장소가 이미 겪은 병이고
(`check-parity-lock-wired.mjs` 가 그것 때문에 있다), 그 라이더에는 **외부 참조가 하나도 없었다**.

**사용자 영향**: 없다(제품 0줄). PR 마다 **~40ms** 짜리 node 스텝 1개가 늘어난다.

### ★설계 급소 — 「순진한 포함 검사」는 **공허하다**
`doc-liveness.yml` 머리 주석(`:34`)이 그 경로 문자열을 **그대로** 들고 있다.
⇒ 호출 줄을 지워도 `grep -q 'check-upstream-new-workflows'` 는 **통과한다**(실측).
★**주석이 만족시키는 가드는 가드가 아니다.** 그래서 이 검사기는 **«주석 아닌 줄»에서만** 센다.

### ★두 번째 설계 결정 — 워크플로 «이름»을 박지 않았다
제안이 물려준 교훈: `check-parity-lock-wired` 가 2026-09-06 에 **스펠링을 박아** 정당한 리팩터를
red 로 만들었고, **그것이 가드가 삭제되는 경로**다. ⇒ load-bearing 한 성질은
「**어떤** 워크플로가 부르는가」이지 「`doc-liveness.yml` 이 부르는가」가 아니다.
⇒ `.github/workflows/` **전수**를 스캔하고, 어디서 걸렸는지는 **요구가 아니라 보고**다.

### 대전제 — 먼저 반증하려 했고, 세 축 다 실패했다(= 제안이 옳다)
- ⒜**지금도 참인가**: **참**. `origin/main edfe108e` 전수 `git grep check-upstream-new-workflows` **6건** =
  `doc-liveness.yml` **주석 1** + **실행 줄 1** + `STATE.md`·report·worklog 산문 3 + 그 스크립트 자신의 Usage 1
  ⇒ ★**실행 참조는 «하나»뿐.**
- ⒝**이미 같은 축이 있나**: **없다**. `check-doc-liveness-parity.mjs` 는 **DOC-COPY 구간만** 집합 대조하고
  라이더는 그 **밖에 «일부러»** 있다(그 워크플로 머리주석이 그렇게 선언한다) ·
  `check-parity-lock-wired.mjs` 는 `LOCK` 상수로 **Rust 파리티 락에만** 묶여 있다.
- ⒞**제안이 틀렸나**: 아니다 — 위 둘이 그것을 확정한다.

### 무엇을 잃는가 / 안 하면 무엇이 나쁜가
- **잃는 것**: ⑴**가드를 지키는 가드가 하나 더 늘었다** — 무한퇴행이 남는다(아래 천장에 명시).
  ⑵PR 마다 상시 스텝 1개(**~40ms** · node-only · 빌드 없음).
  ⑶★**느슨함을 «선택»했다** — 워크플로 이름을 안 박았으므로 「스케줄 잡에서 PR 잡으로 옮겨도 green」이다.
  그것은 결함이 아니라 **오탐과 맞바꾼 값**이고, 그 대가를 천장에 적었다.
- **안 하면**: 라이더 스텝 한 줄이 사라져도 **아무 신호가 없다**. ★그리고 그 라이더가 지키는 것은
  「upstream 이 새 워크플로를 들고 오는 것」이라 **침묵의 대가가 늦게·크게** 온다(base swap 회차가 그 값을 치렀다).

### ★개악 대조 — 양방향 · 제품 호출부에서
| 개악 | 자리 | 결과 |
|---|---|---|
| **M1** 호출 줄만 삭제(★머리 주석은 남긴다) | `doc-liveness.yml:185` | 내 검사기 ★**rc=1** · ↔ 순진한 `grep -q` 는 ★**통과(공허)** ⇒ 「주석 아닌 줄」이 load-bearing |
| **M2** 가드 스크립트 자신을 치움 | `scripts/check-upstream-new-workflows.mjs` | ★**rc=1** 「가드 «자신»이 사라졌다」 |
| **M3** ★**정당한 이동**(오탐 방지축) | 호출을 `rust-audit.yaml` 로 옮김 | ★**rc=0 GREEN** · 보고 줄이 `rust-audit.yaml:115` 로 따라간다 |

복원 후 `doc-liveness.yml` 은 `HEAD` 와 **바이트 동일**, 전 축 **rc=0** 복귀, `git status` 잔재 **0**.
★**M3 이 이 회차의 핵심 방어다** — 그것이 없으면 이 가드는 다음 리팩터에서 삭제될 종류의 가드다.

### 회귀 0 — 전건 합산(★`tail` 미사용)
4게이트 + `cargo +beta clippy` **전부 rc=0** · `RUST_MIN_STACK=4194304 cargo test --all` **rc=0**
— `^test result:` **전수 합산** = ★**43타깃 · 386 passed · 0 failed · 0 ignored**.
node 검사기 **9/9 rc=0**(신설분 포함) · `npm run audit` **PASSED** · 두 워크플로 **YAML 파싱 OK**.
