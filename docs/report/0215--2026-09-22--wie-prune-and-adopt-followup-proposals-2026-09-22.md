## [2026-09-22] 추천 후속작업 2차 정리 — **adopted 5 · declined 19** (★`adopted` 가 중복발권을 막는다) (wie-prune-and-adopt-followup-proposals-2026-09-22)

운영자 지시(2026-09-21) 「주요도가 낮은 작업들은 미진행 … 추천 작업 목록에서 삭제해 정리」 +
(09-22) 「배틀몬스터 … 플레이 가능하게 · 후속을 완전자율주행으로」의 **2차** 이행.
1차(`wie-prune-declined-followup-proposals-2026-09-21` · 머지됨)가 63 → **34** 로 줄였는데,
줄어든 만큼 ★**어제 회차들이 새 제안을 냈다**(정상 동작이다). 이 회차가 그 신규분을 처분한다.
★**선별은 총괄이 했고 이 회차는 «목록 그대로» 집행했다** — 다시 고르지 않았다.

### ⒜ 기준선 — ★**주장하지 않고 «소비자의 파생식»을 재현해서 쟀다**

tower 의 `/api/proposals` 는 `bin/cockpitd.js` 의 `scanRepoSimple` + `collectProposals.settle` 로
목록을 만든다. 그 술어를 **그대로 재현**했다(`origin/main` 아카이브에서 전 워크로그 `.json` 을 걸으며
`adoptedProposals`·`declinedProposals` 를 **전역 합집합**으로 모으고, `proposals[i]` 를 `<파일명>#p<i>` 로
키잉한 뒤 `open = 전체 − adopted − declined − injected − dismissed`).

| 축 | `origin/main`(`ae778a74`) 실측 |
|---|---|
| 전체 `proposals` | **284** |
| `adopted` | 168 |
| `declined` | 34 |
| `injected`(= 이미 지시로 나간 것 · tower `data/injected.json`) | 52 |
| `dismissed` | 0 |
| ★**열린 제안** | ★**34** |

⇒ ★**티켓이 인용한 34 와 정확히 일치한다** — 수를 옮겨 적은 것이 아니라 독립으로 닿았다.

### ⒝ ★세 목록이 실측 집합을 «정확히 분할»하는가 — 눈이 아니라 집합 연산으로

| 질문 | 답 |
|---|---|
| adopted 5 ∪ declined 19 ∪ 남길 10 **==** 열린 34 | ★**True** |
| 세 목록 사이의 겹침 | **0** |
| 목록에 있는데 실재하지 않는 `ref` | **0** |
| ★**열려 있는데 세 목록 어디에도 없는 것** | **0** |

★**마지막 줄이 중요하다** — 그것이 0 이 아니면 「24 를 처분했다」가 「목록이 현실을 덜 덮는다」를 감춘다.

### ⒞ ★★`adopted` 5건이 이 티켓의 «급한» 절반이다 — `declined` 와 **뜻이 반대다**

★**이 5건은 이미 총괄이 티켓으로 발권했다.** 열린 채 두면 ★**다른 레인이 같은 일을 다시 발권한다**
(이 집의 「중복 발권 방지」 규율이 정확히 이 형태를 경계한다). 채택해서 **닫히는 것**이지 기각이 아니다.

| ref | 무엇 | 발권된 티켓 |
|---|---|---|
| `2026-09-21-battlemonster-aot-java-first-wall#p0` | `Invalid memory access; address: 0` 지배 벽 | `wie-aot-java-invalid-memory-access-address-zero-cluster`(P0 · 배차됨) |
| `2026-09-21-aot-java-render-rebaseline#p0` | 배틀몬스터 현 벽(tick 2 · 같은 문면) | ★**위와 같은 티켓**(두 제안이 같은 축이다) |
| `2026-09-21-battlemonster-aot-java-first-wall#p1` | `DialogComponent` 미구현 2종 | `wie-aot-java-missing-platform-classes-dialogcomponent-cluster`(P1 · 배차됨) |
| `2026-09-21-aot-java-render-rebaseline#p1` | `NoClassDefFoundError` 3종 | ★**위와 같은 티켓** |
| `2026-09-21-aot-java-render-rebaseline#p5` | `lgt_compile_model` 일괄차단을 타이틀 단위로 | ★**이미 착지**: `Jun025/otterpebble#989`(게이트② 진행) + 선행 `#980` 머지 |

### ⒟ 집행 — ★**«선언»이고 «삭제»가 아니다**

★**`proposals` 배열의 원소를 하나도 지우지 않았다.** `ref` 는 **0-기반 순서 키**(`<파일명>#p<i>`)라,
원소를 지우면 뒤 원소의 `#pN` 이 밀려 ★**이미 채택·인용된 ref 가 조용히 다른 것을 가리킨다.**
⇒ 닫는 방법은 `adoptedProposals`·`declinedProposals` 에 **이름을 적는 것**이다.

★**어디에 적었나 — 이 repo 의 «기존 관용구»를 먼저 읽고 따랐다**(새 서식 발명 0).
선례가 전부 ★**«판정하는 회차의 워크로그가 «남의 파일»의 ref 를 선언한다»**는 형태이고,
바로 어제의 1차 정리(`2026-09-21-followup-backlog-pruned-for-battlemonster-focus.json`)가
`dispositionNotes` + `kept-open` 키까지 쓴 그 서식이다 — **그대로 썼다.**

⇒ **1파일 신규** `docs/worklog/2026-09-22-followup-backlog-second-pass-adopt-five-decline-nineteen.json`
(`adoptedProposals` 5 · `declinedProposals` 19 · `dispositionNotes` 24 + `kept-open`) ·
★**기존 워크로그 41파일 diff 0**.

★**그래서 소비자가 이걸 «본다»** — `scanRepoSimple` 이 두 목록을 **파일 단위가 아니라 전역 Set** 으로
모으기 때문이다(`for (const r of j.adoptedProposals || []) adopted.add(String(r))`). 코드로 확인했다.

★**사유는 «분류»로 적었다 — 19줄을 같은 문장으로 채우지 않았다.**

| 분류 | 수 | 무엇 |
|---|---|---|
| A 절차·게이트 문서 | **2** | `slice-d-resume-gate-reversibility#p0`·`#p1` |
| B 검사기 자기정비 | **2** | `serial-selftest-live-repo-coupling#p0`·`#p1` |
| C 인용 규약 통지 | **1** | `merge-contract-enumeration-quote-rot#p0`(대상 = `orchestrator`) |
| D 문서 수치 한정 | **3** | `lgt-load-dependent-trace-counts#p0`·`#p1`·`#p2` |
| E 측정·기록 규약 | **2** | `no-frame-paired-remeasure-window-infeasible#p1`(대상 = `orchestrator`)·`#p2` |
| F 시험·감사 축 | **7** | `restore-canvas-decode-roundtrip-tests` 2 · `lgt-svcid-restored-…` 2 · `audit-standing-checker-…` 3 |
| ★G KTF 조사의 «이름·중복» 곁가지 | **2** | `svc-selector5-function0-…#p1` · `knl-reserved-is-ktf-extension-space#p1` |

★**D 와 G 를 숨기지 않는다** — 둘 다 «남긴 것»과 **같은 파일**에서 갈렸다.
D 는 `no-frame-…#p0`(부하 둔감화)을 **남기면서** 닫혔다: 차이는 「문서 서술을 고치는가」 ↔
「**측정이 부하에 흔들리지 않게 만드는가**」이고 **후자만 오판을 실제로 막는다**.
G 는 두 파일에서 `#p0`(관측·추적)을 남기고 `#p1`(이름 변경·중복 술어 합치기)만 닫았다.

### ⒠ 검증 — ★**닫는 쪽만 세지 않았고, `adopted` 와 `declined` 를 «따로» 셌다**

| 축 | 전 | 후 |
|---|---|---|
| ★**`adopted` 5 중 «아직 열린» 것** | 5 | ★**0** |
| ★**`declined` 19 중 «아직 열린» 것** | 19 | ★**0** |
| ★**열린 제안(wie)** | **34** | ★**10** |
| ★**남기기로 한 10건이 그대로 열려 있는가** | — | ★**10/10** |
| 5건이 전역 `adopted` Set 에 들어갔는가 / `declined` 에 잘못 섞였는가 | — | ★**5/5 · 0** |
| 19건이 전역 `declined` Set 에 / `adopted` 에 잘못 섞였는가 | — | ★**19/19 · 0** |
| 남긴 10건이 두 Set «어디에도» 없는가 | — | ★**10/10** |
| 기존 워크로그 41파일 diff | — | ★**0** ⇒ 기존 `#pN` 인덱스 **불변** |
| `scripts/check-worklog-json.mjs` | — | **rc=0**(189파일) — ★검사기가 이 repo 에 **있다**(`engine-contract.yml` 이 매 PR 실행) |

★**남는 쪽을 세는 것이 과잉 차단을 잡는 유일한 축이다** — 닫는 쪽만 세면 「24 를 처분했다」가
「34 를 닫았다」와 화면에서 구별되지 않는다.

### ⒡ ★티켓의 수 하나를 정정한다 — 「전체 36 → 12(RustJava 2 포함)」는 **37 → 13** 이다

RustJava 의 열린 제안을 같은 술어로 쟀더니 **2 가 아니라 ★3** 이었다:
`2026-09-20-string-array-hiding-overflows-stack#p0` · `2026-09-20-name-the-missing-bootstrap-class#p0` ·
★**`2026-09-18-bootstrap-argument-index-and-tag#p0`**(티켓 목록에 이름이 없다).
⇒ 전체는 **34+3 = 37 → 10+3 = 13**.
★**집행은 바뀌지 않는다** — RustJava 는 범위 밖이고 3건 전부 **무접촉**이다. **수만 정정해 둔다.**

### ⒢ 되돌리는 법 — ★**한 줄이다**

`docs/worklog/2026-09-22-followup-backlog-second-pass-adopt-five-decline-nineteen.json` 의
`adoptedProposals`(또는 `declinedProposals`)에서 **그 `ref` 한 줄을 빼면** 다음 `/api/proposals`
파생에서 **다시 열린다**(`open = 전체 − adopted − declined − injected − dismissed`).
전량 원복은 그 파일을 지우면 된다.
★**제안 본문은 원래 자리에 그대로 있으므로 잃은 서술이 0 이다** — 그것이 «삭제»가 아니라 «선언»을 고른 이유다.

### ⒣ 대가 — 「잃는 것이 없다」가 아니다

1. ★**«닫힘»은 착지해야 보인다.** tower 는 `origin/main` **아카이브 캐시**를 읽는다(`worklogDirOf`).
   ⇒ 이 PR 이 열려 있는 동안 화면의 **34 는 그대로**다. 머지가 곧 집행이다.
2. ★**`adopted` 5건은 «발권됐다»는 뜻이지 «끝났다»는 뜻이 아니다.** 그 티켓들이 실패하면
   그 축은 화면에서 **«사라진 채로» 멈춘다.** 되살리는 길은 ⒢ 한 줄이다.
3. ★**이 회차는 «판단을 기록»했을 뿐 24건의 내용을 검토하지 않았다** — 선별의 책임은 총괄에 있고,
   이 파일은 그 사실을 **추적 가능하게** 남긴 것이다(사유 문면에 지시 일자와 문구를 박았다).
4. ★**`injected` 52건은 건드리지 않았다** — 이미 지시로 나간 것이라 «추천 목록»이 아니다.
   화면의 부담이 그만큼 남는다는 사실은 그대로다.
5. ★**제품 코드 0줄** ⇒ 에뮬레이터가 더 돌게 되지 않는다. 이 회차가 사는 것은
   **«무엇을 안 할지»와 «무엇이 이미 발권됐는지»의 기록**뿐이다.

### 범위 판정 — 제안자 추정 `S` ↔ ★내 실측도 `S`

한 회차에 들어갔다. ★**넓히지 않았다** — 검사기·CI·워크플로 신설 **0** · 새 `proposals` 원소 **0** ·
제안 본문 **무접촉** · 다른 repo·tower·원장 **무접촉** · 머지 **0**.
★**좁히지도 않았다** — 24건 전수를 처분했고 남긴 10건까지 실측으로 셌다.
