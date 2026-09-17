## [2026-09-17] 제안은 참인데 «이 보드의 일이 아니다» — 카드 스키마는 orchestrator 소유다 (wie-adopt-corpus-card-first-question-p0)

**무엇을**: 제안 `2026-09-16-corpus-card-first-question#p0` 의 전제를 재측해 **여전히 참**임을 확인하고,
**집행하지 않았다.** 고칠 자리가 `~/orchestrator/humansteps/README.md` — **wie 밖**이기 때문이다.
이 회차의 산출물은 ⑴재측값 ⑵그 자리에 넣을 **문안 초안**(그대로 붙이면 되게) ⑶wie 보드의 처분 기록
⑷라우팅 결함 1건이다. ★**wie 제품 코드 0줄 · Rust 0줄 · `~/orchestrator` 파일 0줄 변경.**

### ⓐ 전제 재측 — 제안은 지금도 참이다
| 축 | 제안 기재 | 2026-09-17 재측 | 판정 |
|---|---|---|---|
| `bin/humanstep-scan` 이 본문을 읽는가 | 「문자열 `body` 0건」 | `grep -c 'body'` → **0** | 참 |
| 본문 3줄 이상 카드 | 57장 | **57장** | 참 |
| 그 본문의 합계 줄 수 | 2,254줄 | **2,269줄**(비어 있지 않은 줄) | 참(15줄 증가 — 그 사이 새 카드 서사) |
| 카드 총수 | 99장 | **99장**(`README.md`·`sample-*` 제외) | 참 |

⇒ 「운영자 화면에 안 닿는 서사가 57장 2,269줄」은 **오늘도 사실**이고, 스키마 문서(`humansteps/README.md`)의
해당 줄은 `(본문은 선택 — 배경·주의사항)` 그대로다 — **닿지 않는다는 사실을 한 글자도 적고 있지 않다.**

### 왜 집행하지 않았나 — 근거 넷, 마지막 하나가 결정적이다
⑴★**제안 스스로 `"target": "orchestrator"` 라고 적고 있다.** 이 티켓의 `repo: wie` 는 그 필드가 아니라
**worklog 파일이 놓인 위치**에서 왔다(아래 「라우팅 결함」).
⑵★**wie 트리에 이 제안이 닿을 표면이 «0» 이다** — `humanstep` 문자열 보유 파일 **4건이 전부 원장**
(`STATE.md`·worklog·report). 제품·스크립트·CI 어디에도 없다.
⑶★**그 파일은 99장의 카드와 estate 3곳(main·dodu·tria)을 지배하는 «전역 작성 규약»이다.**
어떤 독법으로도 「`wie` 만 수정」 안에 들어오지 않는다.
⑷★★**결정적 — `humansteps/README.md` 는 orchestrator-ops 에 «untracked» 다**(`humansteps/` 101파일 중
tracked **5**). ⇒ 거기에 손을 대면 **PR 도 게이트②도 revert 경로도 없다.** wie PR 이 approve 돼도
그 편집은 **어느 리뷰에도 실리지 않는다** — 형식상 wie 회차인데 실질은 **3게이트 밖의 전역 계약 변경**이다.
★**형제 회차와 갈리는 지점이 정확히 여기다**: 직전 `…-triage-r2-p0` 도 `~/orchestrator/humansteps/` 를
고쳤지만 그것은 **자기 티켓을 막고 있던 카드 «한 장»(leaf)** 이었고, 이번 것은 **그 카드들을 규정하는 계약**이다.

★그리고 **wie 안에 적는 우회로는 wie 자신이 금지한다** — `AGENTS.md:39`
「**An external contract is referenced, never copied.** … nothing here can check that, because CI never sees `~/orchestrator`」.
⇒ wie 쪽에 사본을 만드는 것은 규율 위반이자 **두 번째 진실원**이다.

### 산출물 — 그 자리에 넣을 문안(초안 · 이 회차는 «적용하지 않았다»)
대상: `~/orchestrator/humansteps/README.md` · 자리: `## 스키마` 코드펜스 마지막 줄
`(본문은 선택 — 배경·주의사항)` 를 아래로 치환 + 펜스 «뒤»에 한 절 신설.

```
(본문은 선택 — ★운영자 화면에는 닿지 않는다. 바로 아래 절을 읽어라)
```

```markdown
## ★본문(markdown)은 운영자에게 닿지 않는다 — 넣을 곳은 front-matter 다

`bin/humanstep-scan` 은 **front-matter 만** 파싱한다(문자열 `body` **0건** · `--json` 방출 키 15종
`id,title,why,how,blocks,origin,status,cost,since,whereLabel,what,readiness,blocked_by,age_days,verify`
에 본문 **없음**). 그 payload 를 `bin/pipeline-feed` 가 `human_steps` 로 그대로 싣는다.
⇒ ★**본문에 적은 지시는 무증상으로 무효다** — 파일에는 남고, 화면에는 없다.

★**이것은 「본문을 쓰지 마라」가 아니다.** 본문은 **사람이 파일을 열었을 때의 서사**로 여전히 값한다
(2026-09-17 실측: 99장 중 **57장이 본문 3줄 이상 · 합 2,269줄** — 그것을 지우라는 뜻이 아니다).
규율은 하나뿐이다:

> ★**운영자가 «행동하기 위해» 읽어야 하는 문장은 `title`·`why`·`how`·`what` 에 넣는다.
> 본문에는 «배경»만 남긴다.**

판별법: 그 문장이 없으면 운영자가 **잘못 행동하는가?** 그렇다면 본문이 아니라 front-matter 다.
※실사례(2026-09-16 `…-triage-r2-p0`): 「제일 먼저 확인할 첫 질문」을 본문에 적었으면 집행은
명목상 끝나고 실효는 **0** 이었다 — 그 회차가 개악 대조로 그 형태를 **RED** 로 재현했다.
```

### 라우팅 결함 1건(이 회차가 새로 잰 것)
cockpit 이 패널 payload 에 `target` 을 **싣기는 한다**(`~/tower/bin/cockpitd.js:2727`·`:3095` —
`target: p.target || ''`). 그런데 주입 지시문
`directives/processed/20260916T220412Z-cockpit-2026-09-16-corpus-card-first-question-p0.md` 는
`제목·쉬운 설명·노력도` 만 찍고 ★**`target` 을 찍지 않으며**, `repo` 를 **worklog 가 놓인 위치**로 정한다
(`- 제안 ref: … (repo: wie)` · `배송: main 채널 — 이 estate 가 queue/wie 를 갖는다`).
⇒ ★**제안이 자기 소유 보드를 명시해도 라우팅이 그것을 보지 않는다.** 후속 제안으로 뺐다.

### 잃는 것 / 안 하면 무엇이 나쁜가
**잃는 것**: ★**왕복 1회**(총괄이 `orch-*` 로 재발권해야 실제 문안이 들어간다). 그 사이 57장 2,269줄은
그대로 화면 밖이고, **새 카드가 계속 같은 형태로 쓰인다**(오늘 15줄이 늘어난 것이 그 증거다).
★**「잃는 것이 없다」고 적지 않는다** — 지연은 실재하고, 그 값을 치르고 산 것이 ⑷(전역 계약을 게이트 밖에서
고치지 않는 것)이다.
**안 하면(=재발권도 안 하면)**: 규약이 계속 `(본문은 선택 — 배경·주의사항)` 이라고 «가르치므로»
다음 회차도 운영자용 지시를 본문에 적고, 그것은 **오류도 경고도 없이 무효**다.

### 판정식이 이 회차에 어떻게 적용되나(빈 칸을 두지 않는다)
- **⑴ 개악 대조**: ★**못 쟀다 — 대상이 없다.** 이 회차의 wie diff 는 **원장 3파일**뿐이라 제품 호출부가 0이다.
  그리고 제안이 요구하는 변경 자체가 **순수 문서**라 ★**어느 repo 에서 집행하더라도 ⑴은 구조적으로 불가**하다
  (기계 소비자가 없다). 물게 하려면 「본문 3줄 이상이면 `humanstep-scan` 이 WARN」 같은 **코드 축**이 필요한데
  그것은 제안을 **넓히는 것**이라 Contract 3 이 금지한다.
- **⑵ 회귀 0**: 전 스위트 돌렸다(아래).
- **⑶ 처분 기록**: `declinedProposals` 에 기록했다 — ★**`adoptedProposals` 가 아니다**(wie 가 집행하지 않았으므로).
  ★**기록 자체는 «필수»다**: 이 ref 를 처분한 worklog 가 **0건**이라, 안 적으면 패널이 **같은 제안을 wie 로 다시** 내민다.

### 전 스위트(문서 전용 diff 도 면제가 아니다 — AGENTS.md)
- `cargo fmt --all -- --check` OK · `clippy --all -D warnings` rc=0 · wasm clippy rc=0 · `+beta clippy` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` — **전건 합산 384 passed / 0 failed**(result 줄 42개 · `FAILED` 0 · `panicked at` 0)
- 검사기: `check-worklog-json` OK · `check-docs-report-serial` OK · `check-doc-liveness-parity` OK · `npm run audit` PASSED
