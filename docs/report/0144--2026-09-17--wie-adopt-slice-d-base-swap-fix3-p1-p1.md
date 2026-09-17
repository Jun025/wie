## [2026-09-17] 「required check 는 0개」가 **2시간 만에** 거짓이 됐다 — 대조 도구를 세웠고, CI 배선은 실측이 막았다 (wie-adopt-slice-d-base-swap-fix3-p1-p1)

**무엇을**: `scripts/check-branch-protection-claim.mjs` 신설 — `AGENTS.md` 의 **`REQUIRED-CHECKS` 표시 영역**과
**GitHub 이 실제로 강제하는 목록**을 양방향으로 대조한다. ★**CI 에 얹으려 했고, 실측이 «안 된다»고 답해 손으로 도는 도구로 남겼다**(ⓒ).
함께 **이미 거짓이 된 문서 4곳**을 실측으로 갈았다. ★**제품 코드 0줄.**

**왜**: 채택 제안 `2026-09-17-adopt-slice-d-base-swap-fix3-p1#p1` — 「문서를 실측에 맞췄지만,
설정을 켜는 순간 **반대 방향으로** 틀리게 되는데 그것을 **아무도 보지 않는다**」.

---

## ⓐ 제안이 «지금도» 참인가 — ★★**참인 정도가 아니라 «이미 일어났다»**

제안이 예고한 회귀가 **같은 날** 발생했다(실측 2026-09-17 12:4x KST):

| 무엇 | 문서(직전 회차가 실측으로 적은 것) | ★**오늘 실측** |
|---|---|---|
| `branches/main/protection` | 404 "Branch not protected" | **404**(변화 없음) |
| `branches/main` → `.protected` | `false` | ★**`true`** |
| `rulesets` | `[]` | ★**1건 · `enforcement: active`** |
| **강제 required check** | **0개** | ★**5개**(`contract`·`build-web`·`rust_ci` stable ×3) |

ruleset **「main protection」** `created_at` = **2026-09-17T10:17:54+09:00** — 직전 회차의 측정 «뒤», 이 회차 «앞».
★**그 사이 약 2시간 동안 문서 5곳이 「강제 0개」라고 말하고 있었다.**

## ⓑ ★함정 — **왜 그 회차가 틀렸는지가 이 회차 설계의 전부다**
★**ruleset 은 «문서가 인용한 두 신호»에 나타나지 않는다**: `branches/main/protection` 은 **classic 보호만** 답하고
(오늘도 404), `protection.enabled` 도 **false 그대로**다. 보이는 신호는 **`protected: true` 와 `rulesets`** 뿐이다.
⇒ 직전 회차는 **하필 안 보이는 두 축만 읽고** 「0개」로 판정했다. **틀린 것이 아니라 «덜 물었다».**
⇒ 그래서 이 검사기는 **세 축을 다 묻고**, 못 읽으면 ★**「없다」가 아니라 「못 쟀다」(rc=2)** 로 끝낸다.

## ⓒ ★제안의 «선행 확인» — **실제로 돌려서 답을 받았고, 답은 «안 된다»였다**
제안: 「먼저 확인할 것은 **`administration: read` 로 그 엔드포인트가 읽히는가** 하나이고, 안 되면 PAT 라 비용 계급이 달라진다」.
**두 번 측정했고 둘 다 CI 런이 증거다**:

| 시도 | 결과 |
|---|---|
| `permissions: administration: read` 선언 | ★**GitHub 이 워크플로를 «파싱 단계»에서 거부**(run `35180786771` · `startup_failure` · "workflow file issue") ⇒ **그 스코프는 줄 수 없다** |
| `contents: read` 만으로 rider 실행 | ★**403 `Resource not accessible by integration`** — 검사기가 **rc=2 「COULD NOT MEASURE」**로 종료(run `35181122022` 스텝 16) |

⇒ ★**제안이 말한 «안 되면» 갈래가 성립했다** — CI 배선은 **PAT 없이는 불가**이고 PAT 는 비용 계급이 다르다(시크릿 관리).
⇒ ★**배선을 되돌렸다**(주 1회 스텝 제거). ★**영구 red 를 남기지 않는다** — 이 저장소에서 «상시 빨간 검사»는 죽은 검사다.
★**대신 두 가지를 남겼다**: ⑴`doc-liveness.yml` 에 «왜 안 얹었는지»를 **런 번호와 함께** 주석으로 박았다(다음 사람이 다시 시도하지 않게)
⑵`AGENTS.md` 가 **언제 손으로 돌리는지**를 지시한다(표시 영역을 고칠 때 · 머지가 문서와 다르게 굴 때).
★★**그리고 «설계가 값을 했다»**: 403 을 만난 순간 검사기는 ★**«강제 0개»가 아니라 «못 쟀다»**로 멈췄다 —
직전 회차가 진 형태(덜 물어 보고 0개로 단정)를 **그 자리에서 재현하지 않았다**.

## ⓓ 산출물 — 「표시 영역 ↔ API」 양방향 diff(산문 파싱 0)
- `AGENTS.md` 에 **`REQUIRED-CHECKS:BEGIN/END`** 표시 영역 **1곳**(목록 + ruleset 메타). ★**이 저장소의 기존 관용구**다
  (`ENGINE-RUNNER:BEGIN/END` ↔ `check-engine-runner-fixtures.mjs` 와 **같은 형태**) — 새 기구가 아니다.
- 검사기는 **claimed − enforced** 와 **enforced − claimed** 를 각각 찍는다(어느 방향으로 틀렸는지가 문안에 나온다).
- 다른 문서는 **목록을 restate 하지 않고 그 영역을 가리킨다**(`§Constraints` 의 "reference, never copy").

## ⓔ ★개악 대조 — **네 방향**(둘은 red · 둘은 «못 쟀다»)
| 개악 | rc | 출력 |
|---|---|---|
| **A** 표시 영역에서 `build-web` 삭제(과소 주장) | **1** | `` `build-web` is REQUIRED … but the block does not list it — 문서가 과소 `` |
| **B** 없는 `phantom-check` 추가(과대 주장) | **1** | `` `phantom-check` is listed … but GitHub does not require it — 문서가 과대 `` |
| **C** `REQUIRED-CHECKS:BEGIN` 마커 훼손 | ★**2** | 「마커 없음 ⇒ 비교 불가」 — ★**green 이 아니다** |
| **D** `gh` 가 protection/rulesets 를 못 읽는 형상(토큰 부족 모사) | ★**2** | 「COULD NOT MEASURE … 못 읽는 토큰이 «강제 없음» 으로 읽히면 안 된다」 |
| 복원 | **0** | `claims 5 · enforces 5` |

★**C·D 가 이 회차의 급소다** — 직전 회차가 진 형태(신호를 덜 읽고 «0개»로 단정)가 **다시 나면 rc=2 로 운다**.

## ⓕ 함께 고친 것 — **이미 거짓이 된 문서 4곳**(범위 판단은 회신에 적었다)
`AGENTS.md`(사건 대장 항목 재작성 + 표시 영역) · `.github/workflows/engine-contract.yml`(머리 + `:233` 주석) ·
`.github/workflows/web.yml`(「THAT ONCE HAS NOT HAPPENED」 → 「HAPPENED」) · `docs/upstream-realign-p3-slices.md`(재정정).
★**전부 «목록을 옮겨 적지» 않고 표시 영역을 가리키게** 바꿨다 — 그래야 다음 변화 때 **한 곳만** 고친다.

## ⓕ-2 ★«주 1회 CI» 를 못 쓰게 된 대가 — 숨기지 않는다
★**지금 이 축은 «사람이 돌려야 도는» 축이다.** 제안이 원한 「아무도 안 보는 상태의 종료」는 **절반만** 달성됐다 —
도구와 **단일 사본(표시 영역)** 은 생겼지만 **주기 실행이 없다**. ⇒ PAT 도입 여부를 결정해야 그 절반이 닫힌다(후속 제안 1건).

## ⓖ 부수 관측(고치지 않았다 · 기록만)
- ★**required 5개가 전부 «paths 필터 없음»이다** — `rust.yml`·`web.yml` 의 `pull_request` 트리거에 `paths:` 가 **없고**
  `contract` 는 상시 실행 래퍼다 ⇒ ★**사건 대장의 「paths 필터된 required check 는 머지를 영구 교착시킨다」에 걸리는 조합이 «현재 0»** 이다.
  그 래퍼를 지킨 선택이 오늘 값을 했다.
- `pull_request` 규칙의 `required_approving_review_count` = **0** ⇒ 승인 강제는 아니다(3게이트는 그대로 사람·절차).
- ★**`--record` 를 이 PR 에 동봉했다** — 착수 시 `check-worklog-coverage` 가 **OVERDUE(114 − 104 = 10)** 였고,
  `contract` 가 **required** 가 된 지금 그 red 는 **모든 PR 의 머지를 막는다**. 검사기 문안이 「ledger file 이라 여기서 만져도 된다」고
  명시하고 멱등이라, 이 회차가 그 한 파일을 실었다(사유는 done 에 적었다).

## ⓗ 검증
검사기 **9종 rc=0**(신규 포함 · `worklog-coverage` 는 `--record` 후 rc=0) · `fmt`·`clippy --all -D warnings`·wasm·**`+beta`** rc=0 ·
`RUST_MIN_STACK=4194304 cargo test --all` **rc=0**(42타깃 · **385 passed · 0 failed**) · `npm run audit` **PASSED** ·
`doc-liveness` parity **rc=0**.

**사용자 영향**: 「이 검사가 필수인가」를 문서만 보고 잘못 판단하면 **한 명령이 그 자리에서 양방향으로 말해 준다** —
오늘 그 오판은 한쪽으로 **2시간**, 반대쪽으로 **8주**였다. ★**자동 주기 실행은 아직 없다**(PAT 결정이 남았다).
