## [2026-09-21] 추천 후속작업 **31건**을 `declinedProposals` 로 닫는다 — ★**«삭제»가 아니라 «선언»이다** (wie-prune-declined-followup-proposals-2026-09-21)

운영자 지시(2026-09-21): 「효율적인 수행을 위해 **주요도가 낮은 작업들은 미진행**하려고 한다 … **불필요하거나
우선순위가 낮은 작업들은 추천 작업 목록에서** 너의 판단 하에 **삭제해 정리**해 달라」.
★**선별은 총괄이 했고 이 회차는 «목록 그대로» 닫았다** — 다시 고르지 않았다.

### ⒜ 기준선 — ★**주장하지 않고 «소비자의 파생식»을 재현해서 쟀다**

tower 의 `/api/proposals` 는 `bin/cockpitd.js` 의 `scanRepoSimple` + `settle` 로 목록을 만든다.
그 술어를 **그대로 재현**했다(전 워크로그 `.json` 을 걸으며 `adoptedProposals`·`declinedProposals` 를
**전역 합집합**으로 모으고, `proposals[i]` 를 `<파일명>#p<i>` 로 키잉한 뒤
`open = 전체 − adopted − declined − injected − dismissed`).

| 축 | `origin/main`(`c2ac05d2`) 실측 |
|---|---|
| 전체 `proposals` | **254** |
| `adopted` | 158 |
| `declined` | 3 |
| `injected`(= 이미 지시로 나간 것 · tower `data/injected.json`) | 58 |
| `dismissed` | 0 |
| ★**열린 제안** | ★**35** |

⇒ ★**티켓이 인용한 35 와 정확히 일치한다** — 티켓의 수를 옮겨 적은 것이 아니라 독립으로 닿았다.

### ⒝ ★목록이 실측 집합을 «정확히 분할»하는가 — 눈이 아니라 집합 연산으로

| 질문 | 답 |
|---|---|
| 닫을 31 ∪ 남길 4 **==** 열린 35 | ★**True** |
| 닫을 ∩ 남길 (겹침) | **0** |
| 목록에 있는데 실재하지 않는 `ref` | **0** |
| ★**열려 있는데 두 목록 어디에도 없는 것** | **0** |

★**마지막 줄이 중요하다** — 그것이 0 이 아니면 「31 을 닫았다」가 「목록이 현실을 덜 덮는다」를 감춘다.

### ⒞ 집행 — ★**«선언»이고 «삭제»가 아니다**

★**`proposals` 배열의 원소를 하나도 지우지 않았다.** `ref` 는 **0-기반 순서 키**(`<파일명>#p<i>`)라,
원소를 지우면 뒤 원소의 `#pN` 이 밀려 ★**이미 채택·인용된 ref 가 조용히 다른 것을 가리킨다.**
⇒ 닫는 방법은 `declinedProposals` 에 **이름을 적는 것**이다.

★**어디에 적었나 — 이 repo 의 «기존 관용구»를 먼저 읽고 따랐다**(새 서식을 발명하지 않았다).
선례 3건이 전부 ★**«판정하는 회차의 워크로그가 «남의 파일»의 ref 를 선언한다»**는 형태다:

```
2026-08-27-upstream-realign-verdict.json          declined: 2026-08-26-worklog-json-proposals-convention#p2
2026-09-01-worklog-mandate-decision-and-backfill  declined: 2026-08-26-worklog-json-proposals-convention#p0
2026-09-16-slice-d-base-swap-fix2.json            declined: 2026-09-16-slice-d-base-swap-executed#p0
```

그리고 그중 하나(`2026-08-27`)는 ★**`dispositionNotes`** 로 사유를 남기고 ★**`kept-open`** 키로
«닫지 않은 것과 그 이유»까지 적어 두었다 — **그 두 관용구를 그대로 썼다.**

⇒ **1파일 신규** `docs/worklog/2026-09-21-followup-backlog-pruned-for-battlemonster-focus.json`
(`declinedProposals` 31 · `dispositionNotes` 31 + `kept-open`) · ★**기존 16파일 diff 0**.

★**그래서 소비자가 이걸 «본다»** — `scanRepoSimple` 이 `declined` 를 **파일 단위가 아니라 전역 Set** 으로
모으기 때문이다(`for (const r of j.declinedProposals || []) declined.add(String(r))`). 코드로 확인했다.

★**사유는 «분류»로 적었다 — 31줄을 같은 문장으로 채우지 않았다.** 총괄의 기준 ⒝ 를 여섯 갈래로 나눴다:

| 분류 | 수 | 무엇 |
|---|---|---|
| A 센서스·검사기 자기정비 | **9** | `census-*` 4계열 + `checker-census-py-extension` |
| B 코퍼스 이름 유입 계측기 | **7** | `corpus-name-inflow-*` · `inflow-marker-*` · `inflow-tool-call-*` |
| C 공급망 대장·CI 배선 | **4** | `ledger-c-validity-scope` · `pr-stage-cargo-audit` |
| D 문서 인용 규약·옛 문서 정정 | **4** | `self-hosted-runner-claim` · `worklog-report-citation-axis` |
| E 워크로그 커버리지 기록 술어 | **3** | `worklog-coverage-union-record-read` |
| ★F KTF 조사의 «도구·기록» 곁가지 | **4** | `ktf-slot8-h2-baseline-degenerate#p1` · `ktf-slot8-third-image-search#p1` · `ktf-sweep-unresolved-drop-is-said#p0·#p1` |

★**F 를 숨기지 않는다** — 이 넷은 배틀몬스터 레인에 **가장 가깝다**. 그런데도 닫히는 이유는 총괄 기준 그대로
「타이틀을 돌리는 능력이 아니라 조사 도구의 **출력·기록 규약**을 다듬는 일」이기 때문이다.
★특히 `ktf-sweep-unresolved-drop-is-said#p0`(`resolve_global` 근인 처방)은 조사 도구의 **정확도**를 올리는 일이라,
★**그 도구의 오독이 실제로 조사를 막는 순간 이 판단을 «먼저» 되돌려라** — `dispositionNotes` 에도 그렇게 적었다.

### ⒟ 검증 — ★**닫는 쪽만 세지 않았다**

| 축 | 전 | 후 |
|---|---|---|
| ★**닫기로 한 31 중 «아직 열린» 것** | 31 | ★**0** |
| ★**남기기로 한 4건이 그대로 열려 있는가** | — | ★**4/4** (`ktf-slot8-third-image-search#p0` · `ktf-slot8-h2-noop-arm-reach-gate#p0`·`#p1`·`#p2`) |
| 내 `declinedProposals` **==** 그 31 (집합 동일) | — | ★**True** — 더 닫지도, 덜 닫지도 않았다 |
| 기존 워크로그 16파일 diff | — | ★**0** ⇒ 기존 `#pN` 인덱스 **불변** |
| `scripts/check-worklog-json.mjs` | — | **rc=0**(176파일) — ★검사기가 이 repo 에 **있다**(`engine-contract.yml` 이 매 PR 실행) |

★**남는 쪽을 세는 것이 과잉 차단을 잡는 유일한 축이다** — 닫는 쪽만 세면 「31 을 닫았다」가
「35 를 닫았다」와 화면에서 구별되지 않는다.

★★**그리고 「35 → 4」라고 단정하지 않는다 — 재측하면 «5»가 나온다. 그 5번째를 숨기지 않고 적는다.**
집행 «후» 재측에서 열린 제안이 **5**였고, 그 다섯째는 **`2026-09-18-sibling-crates-missing-tests-machine-audit#p1`**
(「LGT svc_ids 의 fail-closed 단언과 test_helloworld 세 번째 케이스」)이다.
★**이 회차가 연 것이 아니다** — 기준선 시점에 그 ref 는 tower 의 `injected`(= 이미 지시로 나간 것)에 있어
**애초에 35 에 들어 있지 않았고**, 작업 중(`data/injected.json` mtime **18:33:47**) 그 원장에서 빠지며
열린 쪽으로 돌아왔다. ⇒ ★**이 repo 밖 · 이 diff 밖의 움직임**이다(그 ref 는 PR #215 가 이행 중인 그것이다).
★**티켓의 두 목록 어디에도 없으므로 «닫지 않았다»** — 「회차가 다시 고르지 마라」가 이 자리에도 적용된다.
⇒ **이 회차의 효과는 «35 → 4»이고, 화면이 보일 수는 «그때의 `injected` 에 달려 있다.»**

### ⒠ 되돌리는 법 — ★**한 줄이다**

`docs/worklog/2026-09-21-followup-backlog-pruned-for-battlemonster-focus.json` 의 `declinedProposals` 에서
**그 `ref` 한 줄을 빼면** 다음 `/api/proposals` 파생에서 **다시 열린다**
(`open = 전체 − adopted − declined − injected − dismissed`). 전량 원복은 그 파일을 지우면 된다.
★**제안 본문은 원래 자리에 그대로 있으므로 잃은 서술이 0 이다** — 그것이 «삭제»가 아니라 «선언»을 고른 이유다.

### ⒡ 대가 — 「잃는 것이 없다」가 아니다

1. ★**«닫힘»은 착지해야 보인다.** tower 는 `origin/main` **아카이브 캐시**를 읽는다(`worklogDirOf`).
   ⇒ 이 PR 이 열려 있는 동안 화면의 **35 는 그대로**다. 머지가 곧 집행이다.
2. ★**이 회차는 «판단을 기록»했을 뿐 31건의 내용을 검토하지 않았다.** 선별의 책임은 총괄에 있고,
   이 파일은 그 사실을 **추적 가능하게** 남긴 것이다(사유 문면에 지시 일자와 문구를 박았다).
3. ★**분류 F 의 위험**(위 ⒞) — 닫은 것 중 유일하게 «목표 레인을 느리게 만들 수 있는» 갈래다.
4. ★**`injected` 58건은 건드리지 않았다** — 이미 지시로 나간 것이라 «추천 목록»이 아니다.
   화면의 부담이 그만큼 남는다는 사실은 그대로다.
5. ★**제품 코드 0줄** ⇒ 에뮬레이터가 더 돌게 되지 않는다. 이 회차가 사는 것은 **«무엇을 안 할지»의 기록**뿐이다.

### 범위 판정 — 제안자 추정 `S` ↔ ★내 실측도 `S`

한 회차에 들어갔다. ★**넓히지 않았다** — 검사기·CI·워크플로 신설 **0** · 새 `proposals` 원소 **0** ·
제안 본문 **무접촉** · 다른 repo·tower·원장 **무접촉**. ★**좁히지도 않았다** — 31건 전수를 닫았고 실측으로 셌다.

### 범위·경계

제품 코드 **0줄** · `game_lab/`·`.gitignore`·`STATE.md`·`AGENTS.md`·**CI 워크플로 무접촉** ·
신규 2파일(worklog 1 · 이 보고서 1) · 기존 파일 수정 **0**.

### ★게임 파일명 유입 — **0 이 아니다. 「42회 / 2쌍」이고, 그렇게 적는다**

★**도구를 돌려 적었다**(`scripts/corpus-name-inflow.mjs` · 코퍼스 고유 stem **451**개 × 이 회차의 **2파일**):

| 바구니 | 수 |
|---|---|
| ★**BOUNDED**(= 유입 수) | ★**42회 / 2쌍** |
| PREFIX-EMBEDDED | 0회 / 0쌍 |
| ★**SUFFIX-ATTACHED**(기계가 못 가르는 것) | **0회 / 0쌍** |

★**「유입 0」이라고 쓰지 않았다** — 그것이 이 리니지가 반려됐던 바로 그 형태다.
**2쌍의 정체는 «한 stem × 두 파일»**이다: stem 은 **`배틀몬스터`**, 파일은 이 보고서와 이 회차의 worklog.
42회의 대부분은 ★**티켓이 «명령한» 추적 문구**(「2026-09-21 운영자 지시 — 배틀몬스터 플레이 집중을 위한
우선순위 정리」)가 `dispositionNotes` **31개 항목마다** 한 번씩 들어갔기 때문이다.

★**그리고 이것은 «새 이름이 들어온 것»이 아니다** — 실측: `배틀몬스터` 는 `origin/main` 의 **추적 파일 8개에
26회** 이미 있다(`docs/lgt_abi.md` 19 · `docs/lgt.md` · `docs/project-kb/02_status.md` ·
`10_deep-assets.md` · `docs/report/0016` · `docs/upstream-realign-verdict.md` ·
`docs/worklog/2026-08-27-*.json` · `scripts/lgt_render_probe.sh`). 이 타이틀은 이 저장소의 **조사 대상 자체**다.
★**그래도 수를 깎지 않았다** — 「이미 있으니 0으로 적자」가 정확히 이 도구가 막으려는 행동이다.
★**게임 «바이트»는 0 이다**(Constraint 9 의 대상) — 들어온 것은 산문 속 **이름**뿐이고, 그 사실을 여기 적는다.


<!-- corpus-name-inflow v1 subjects=2 tree=423a837cfe3f62d2 B=42/2 P=0/0 S=0/0 -->
