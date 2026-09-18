## [2026-09-18] ruleset 의 «나머지 21개 필드»를 보게 했다 — 목록이 아니라 «객체 지문»으로 (wie-branch-protection-ruleset-drift-beyond-required-checks)

**무엇을**: 채택 제안 `2026-09-17-branch-protection-claim-guard#p0` 의 이행.
`scripts/check-branch-protection-claim.mjs` 에 **축 ⑵**(정규화된 ruleset 객체 ↔ 베이스라인)를 더하고
`.github/branch-protection-expected.json` 을 신설했다.
★**새 검사기 0 · 새 CI 스텝 0 · PAT 0 · ruleset 무접촉**(계약 3 전건 준수).

**왜**: 기존 가드는 `if (r.type !== "required_status_checks") continue;` 로 ★**네 rule 중 셋을 건너뛴다**.
`bypass_actors`·승인 수·허용 머지 방식·삭제 금지가 바뀌어도 **OK 를 찍는다** — 그리고 이 저장소의 3게이트가 그 값들 «위»에 서 있다.

**사용자 영향**: 없다(제품 0줄). 바뀐 것은 **그 가드가 보는 범위**다.

### ⓐ 주장을 «내가» 먼저 확인했다 — 재현됨
라이브 ruleset(`id=23572944` `main protection` `active` `~DEFAULT_BRANCH`)의 rules 는 **4개**:
`deletion` · `non_fast_forward` · `pull_request` · `required_status_checks`.
기존 가드가 읽는 것은 ★**마지막 하나뿐**이다.

### ⓑ 리니지 — 새로 만들지 않고 «얹었다»
`tasks/` 에 `wie-branch-protection-drift-watch-pat-decision`(= **#p1 PAT 결정** · ★계약 3⒜ 가 범위 밖으로 못박은 그것) ·
`wie-main-branch-protection`(설정 적용) 실재. **이 축의 중복 0건.**
★그리고 **새 파일을 만들지 않고 기존 가드를 확장**했다 — 둘은 같은 `gh` fetch·같은 fail-closed 의미·**같은 «손으로 돌리는» 제약**을 공유하므로,
가르면 사람이 **두 도구를 기억해야** 하고 이 저장소는 「손으로 기억해야 하는 것은 죽는다」를 이미 측정했다.

### ⓒ 가정법 ⇒ 픽스처를 만들었다(계약이 시킨 그것)
제안은 「…바뀌어도 아무도 모른다」라 **오늘 재현 입력이 없다**. 가짜 `gh` 를 PATH 에 끼워
`api …/rulesets/<id>` **응답만** 가로채 개악했다 — ★**읽기 경로만 건드렸고 실 ruleset 은 무접촉**
(사후 실측: 승인 수 **0** · `bypass_actors` **0** · rules **4** 불변).

### ★★양방향(Acceptance ⑴) — 4종 전건 발화
| 개악(required-check «가 아닌» 축) | rc | 가드가 지목한 줄 |
|---|---|---|
| (무개악) | **0** | `OK — … and the ruleset shape matches its baseline.` |
| `approval` 승인 수 0 → 2 | ★**1** | line 34 `required_approving_review_count` |
| `bypass` `bypass_actors` 채움 | ★**1** | line 14 `bypass_actors` |
| ★`futurefield` — **오늘 없는** rule type `required_signatures` 추가 | ★**1** | line 40 |
| `dropdeletion` — `deletion` rule 제거 | ★**1** | line 17 |

★★**`futurefield` 가 이 설계의 증거다.** 계약 1 은 「항목을 손으로 열거하지 마라 — 다음 항목이 생기면 또 샌다」였고,
★**오늘 존재하지 않는 필드로 그것을 확인했다**. 열거식 검사였으면 **통과**했을 것이다.

### 설계 — 왜 «해시»가 아니라 «읽을 수 있는 JSON» 인가
계약 1 은 «지문»을 요구했다. 해시는 「바뀌었다」에서 멈춘다 ⇒ ★**정규화 JSON 자체를 베이스라인으로** 두면
그 파일이 **곧 diff** 라 실패가 **어느 줄이 움직였는지**를 말한다 — 그것이 「운영자가 의도했다 ↔ 드리프트다」를 가르는 정보다.
★**베이스라인을 손으로 쓰지 않았다**: 파일이 없으면 가드가 **rc=2(fail-closed)** 로 멈추면서 **라이브 형상을 찍어 준다**
⇒ 검토 후 저장했다(정규화 로직을 **두 벌로 만들지 않으려고**). ★그 rc=2 를 **이 회차가 실제로 받았다**.

### Acceptance ⑵ — PAT 없이 어디까지 보이나(수)
```
비교되는 잎 필드          27
  그중 종전에도 보던 축      6   (required_status_checks)
  ★종전에 «아무도 안 보던»  21
로컬 admin `gh`          27 / 27 보인다
★CI(`github.token`)        0 / 27   — 두 엔드포인트에서 403(run 35181122022) ·
                                      `administration: read` 는 «줄 수 없는 키»(run 35180786771)
```
⇒ ★**이 수가 #p1(PAT 결정)의 입력이다** — 「PAT 없이 주기 감시는 0」이 숫자로 섰다.

### Acceptance ⑶ — 정규화가 버리는 것(1줄)
`id` · `node_id` · `created_at` · `updated_at` · `source` · `source_type` · `current_user_can_bypass`
⇒ ★**ruleset 을 지우고 «같은 내용»으로 다시 만들면 이 가드는 못 본다**(id 만 바뀐다).
`current_user_can_bypass` 는 **호출자에 따라 달라져** 비교하면 답이 «누가 돌렸는가»에 의존하므로 버렸다 — `bypass_actors` **목록은 비교한다**.

### 계약 2 — 잃는 것
⒜★**무해한 변경에도 운다**: 그래서 배열을 정렬했다(required-check 순서·`allowed_merge_methods` 순서). 대가는 «순서 자체가 의미인 필드»를 못 보는 것(오늘 그런 필드 0).
⒝★**PAT 는 도입하지 않았다**(계약 3⒜) ⇒ 여전히 **손으로** 돌린다. 주기 호출자 **0**.
⒞★**정당한 변경 때마다 red** 다 ⇒ 실패 문구가 **무엇을 고쳐야 하는지**를 세 줄로 말한다
(의도했으면 같은 PR 에서 JSON 갱신 + 이유 · 아니면 에스컬레이트 · ★이 도구는 GitHub 에 **쓰지 않는다**).
★그 세 줄이 없으면 다음 사람이 **검사를 끈다** — 이 저장소가 아는 결말이다.

### 회귀 0 — 전건 합산
4게이트 + `cargo +beta clippy` **rc=0** · `cargo test --all` **44타깃 · 388 passed · 0 failed** · `npm run audit` PASSED.
node 검사기 **13개 중 11개 rc=0** — rc=1 **2개는 선재**(`check-worklog-coverage` OVERDUE · `check-audit-warnings`, 둘 다 내 변경 이전부터).
