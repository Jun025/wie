## [2026-09-21] 머지계약 열거 인용을 «지금의» 계약과 맞췄다 — 결론 문장까지 (wie-agents-md-quotes-a-merge-contract-enumeration-that-has-since-gained-two-entries)

**무엇을** — `AGENTS.md` §Landing paperwork 의 원장 파일 블록에서, 외부 정본
(`~/orchestrator/templates/merge-ticket.tpl` §2-c⒜)을 인용한 **두 문단**을 현행 문면으로 갱신했다.
인용 열거를 현재의 7항목(`STATE`·`REPORT`·`docs/report/**`·`docs/worklog/**`·
`docs/worklog-coverage-remeasures.json`·`reports/`·`tasks/`)으로 고치고, **그 열거에 딸린 결론 문장**
(「does not name `docs/report/**`」·「cannot find it there」·「this file is a *sibling* … no rule
saying it may touch the file」)을 함께 고쳤다. 열거만 갈고 결론을 두면 본문이 자기 열거와 모순된다.

**왜** — 인용이 낡아 **결론이 거짓**이 돼 있었다. 실측(2026-09-21):

| 축 | 인용이 단언한 것 | 정본의 현재 문면 |
|---|---|---|
| `docs/report/**` | 「does not name」 | **있다** — 2026-09-07 추가(`orch-merge-template-ledger-path-list-lacks-docs-report`) |
| `docs/worklog-coverage-remeasures.json` | 인용에 **아예 없음** · 「sibling 이라 권위가 없다」 | **있다** — 2026-09-21 추가(orchestrator-ops PR #1087 · 머지 `74fac0820`) |

같은 파일 `:978~979` 의 **정본 열거는 이미 옳았다** — 갈린 것은 인용뿐이다(정본 무접촉).

**사료는 지우지 않고 시점을 밝혀 보존했다.** 2026-09-07 어느 머지 회차가 스스로 따져 내린 판단
(「`REPORT.md` 에 덧붙이면 20분 전 착지한 규약을 깬다 · 항목을 버리면 잃는다 ⇒ 옮긴다」)이 이 문단의
존재 이유다. 계약이 따라잡았다고 그 기록을 지우면, 그 공백기에 이 줄이 **유일한 written authority**
였다는 사실이 사라진다. 이 repo 가 반복해 규탄한 형태가 「낡은 수를 권위 있게 인용한다」이고 그 해법은
삭제가 아니라 **시점 명시**다 — 그래서 「언제부터 참이고 그 전에는 무엇이었나」로 다시 썼다.

**사용자 영향** — 없음(산문). 게이트③ 회차가 `docs/report/**`·`docs/worklog-coverage-remeasures.json`
을 만질 권위를 **계약에서 직접** 찾게 된다 — 종전에는 이 문단이 「거기서는 못 찾는다」고 잘못 가르쳤다.

### 측정

```
                              before(origin/main)   after
grep -c 'worklog-coverage-remeasures'      5          7
grep -c 'does not name'                    1          0
```

★티켓 Acceptance 의 「현재는 **1**」은 **작성 시점에 이미 거짓**이었다 — 실측 **5**(정본 1 + §완료 2회차
서술 등 4). 기준(≥2)은 before 에서도 이미 충족돼 있어 **이 축은 이 회차를 판정하지 못한다**. 실제로
움직인 축은 두 번째 줄(1 → 0)과, 인용 열거 자체가 7항목이 된 것이다.

### 검사기는 만들지 않았다 — 그 결정의 이유

인용 대상이 **다른 repo 의 파일**이라 이 repo 의 CI 가 읽을 수 없다(§Constraints 「CI never sees
`~/orchestrator`」). 읽게 만들면 외부 트리 의존이 생긴다. 기존 검사 유무도 실측했다 — `scripts/` 의
실행 가능 산출물 중 `orchestrator`·`merge-ticket` 을 참조하는 것 **0건**이므로 「왜 안 잡았는가」가
아니라 「애초에 그런 축이 없다」가 정답이다. 탐지 축은 **정본 쪽(orchestrator)** 소관이라
`docs/worklog/2026-09-21-merge-contract-enumeration-quote-rot.json` 에 `target: orchestrator` 제안으로
올렸다 — 이 파일이 이미 쓰는 형태다(§verify-browser 의 `…-deploy-selfverify-instruction-p0` 선례).

### 범위 밖 — 세었으나 고치지 않았다

`AGENTS.md` 가 같은 외부 정본을 인용하는 나머지 **3곳은 전부 유효**하다(실측):

- `:39` 「An external contract is referenced, never copied」 — 원칙 서술, 열거 없음.
- `:503` 「`AGENTS.md`, which the merge contract does **not** put in the set a merge round may touch」
  — **여전히 참**이다(7항목에 `AGENTS.md` 없음).
- `:555` §4-C 「self-verify(운영 URL·콘솔 0에러)」 / `:1120~1121` §4-A — 정본에 **실재**
  (`merge-ticket.tpl:908` · `:773`).

⇒ 낡은 인용은 이번에 고친 **두 문단뿐**이었다.
