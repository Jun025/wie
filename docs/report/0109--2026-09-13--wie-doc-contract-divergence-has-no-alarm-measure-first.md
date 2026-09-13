## [2026-09-13] 문서↔계약 어긋남 경보 — ★**만들지 않는다(⒥)**. CI 가 그 계약 파일을 «볼 수 없다» (wie-doc-contract-divergence-has-no-alarm-measure-first)

- **산출물은 «판정»이다**(계약 1). 코드·검사기 **구현 0** · `AGENTS.md` 재수정 **0** · 남의 정본(`contracts/`·`templates/`) **무접촉**.

- **★★판정 — ⒥ 「만들 값이 없다」. 근거를 수로 적는다.**
  | 축 | 측정값 | 판정에 기여하는 바 |
  |---|---:|---|
  | ⓐ 지금 잡는 기계 | **0** | 「울지 않는다」는 **참** |
  | ⓐ-2 repo 내 정합성 검사기 | **9** | 전부 살아 있다 — 그러나 ★**9/9 가 «repo 안 ↔ repo 안»** |
  | ⓑ 외부 계약을 **«값»으로 복사한 자리** | ★**2** | 티켓이 정한 임계 「1~2곳이면 사람 규율로 충분 · 10곳 이상이면 기계가 값한다」의 ★**아래쪽 끝** |
  | ⓑ 같은 계약의 **재진술**(규칙 복창) | 3 | 값이 아니라 규칙 — 낡을 여지가 낮다 |
  | ⓑ **포인터**(경로·절 앵커) | 3 | 복사가 아니다 — 정본이 옮기면 깨지되 «조용하지» 않다 |
  | ⓒ CI 러너가 그 계약을 볼 수 있는가 | ★**볼 수 없다**(7/7 GitHub 호스티드 · self-hosted **0**) | ★**게이트가 될 수 없다** |
  | ⓒ 참조 대상의 버전 추적성 | ★**없다**(아래) | ★**검사기가 «통과시키며 틀릴» 수 있다** |

- **★★ⓒ 가 이 판정의 «결정타»다 — 검사기의 «참조 쪽»이 이 저장소에서 검증 불가능하다.**
  ★**트리를 밝힌다**(헌장 「인용 트리 명시」): `~/orchestrator`@live, HEAD `13b88a0c`(2026-08-17) ·
  ★**`origin/main` 보다 2,473 커밋 뒤**.
  | 참조 대상 | 그 트리의 «커밋된» 판 | «라이브» 파일 |
  |---|---|---|
  | `templates/merge-ticket.tpl` | **63줄 · `4-A` 0건** | **1,146줄 · `4-A` 11건**(mtime 09-11) |
  | `contracts/upstream-sync-repos.conf` | ★**추적되지 않음**(그 트리의 git 이 이 계약을 **모른다**) | `repo` 2건 등재(mtime 08-28) |
  ⇒ ★**검사기가 «커밋된 판»을 읽으면 «§4-A 가 없다»로 오탐**하고(문서는 옳은데 red),
  ★**«라이브 파일»을 읽으면 출처 불명의 바이트와 비교**한다(한쪽은 untracked).
  ★★**그리고 후자가 더 나쁘다 — «틀린 문서»를 «낡은 계약»이 추인해 «통과»시킬 수 있다.**
  이 집이 반복해 이름 붙인 「가드가 자기가 막으려던 것을 통과시킨다」가 정확히 그 형태다.

- **ⓐ 전수 — 잡는 기계 0**: repo 검사기 **9종**(`check-audit-warnings`·`check-doc-liveness-parity`·
  `check-docs-report-serial`·`check-engine-contract`·`check-engine-runner-fixtures`·`check-parity-lock-wired`·
  `check-worklog-coverage`·`check-worklog-json`·`dod_ci_parity.rs`) 중 ★**repo 밖 경로를 읽는 것 0**.
  `~/orchestrator/bin/` 에서 `AGENTS.md` 를 읽는 도구 **0**. ⇒ 양쪽 어디에도 이 축이 **없다**(중복 위험 0).
  ★**그 9종이 «할 수 있는» 이유가 대비를 만든다** — 전부 **한 PR 안에서 양쪽이 함께 움직이는** 축이다
  (문서 ↔ 워크플로 · 문서 ↔ 픽스처 · 원장 ↔ 원장). 문서↔외부계약은 ★**양쪽이 다른 저장소에서 따로 움직인다.**

- **ⓑ 복사 자리 전수(목록)** — 판정은 **문장 단위**로 했다:
  | # | 자리(문구로 지목) | 분류 |
  |---|---|---|
  | 1 | `AGENTS.md` 「This repo is **registered as an upstream-sync fork** and must *not* squash-merge」 | ★**값 복사**(등재 사실) |
  | 2 | `AGENTS.md` 「For this repo **the value is `merge`** — never `squash`」 | ★**값 복사**(전략 값) |
  | 3 | `AGENTS.md` Constraint 12 「Merge and branch deletion are a separate approved task」 | 규칙 재진술 |
  | 4 | `AGENTS.md` 「**Never merge your own PR** …」 | 규칙 재진술 |
  | 5 | `CLAUDE.md` 「머지와 브랜치 삭제는 … 별도 `-merge` 티켓의 몫」 | 규칙 재진술 |
  | 6 | `AGENTS.md` ×2 「`~/orchestrator/templates/merge-ticket.tpl` … outside this repo」 | 포인터 |
  | 7 | `CLAUDE.md` 「티켓 파일이 `~/orchestrator/tasks/` 에 없으면」 | 포인터 |
  ⇒ ★**«조용히 어긋날 수 있는» 것은 1·2 뿐이고, 둘은 «같은 계약»의 «같은 불릿»에 있다**(오늘 `9ef24b43` 이 고친 그 자리).
  ★**3~5 는 값이 아니라 규칙**이고 이 repo 의 사건 대장(자기 머지 **5회**)이 **독립적으로** 떠받친다 —
  orchestrator 가 그 규칙을 바꿔도 이 repo 는 여전히 그렇게 하고 싶어 한다 ⇒ **어긋남이 «결함»이 아니다.**
  ★**6~7 은 복사가 아니라 포인터**라 깨지면 **시끄럽다**(경로가 없으면 사람이 즉시 막힌다).

- **ⓓ 더 싼 대안 — ★이미 착지했고, 값이 증명됐다**: 「**값을 복사하지 말고 정본을 가리켜라**」
  (오늘 `9ef24b43`). ★**복사가 없으면 어긋날 것이 없다** ⇒ 검사기가 지킬 표면 자체가 사라진다.
  | | 비용 | 커버리지 | 오탐 |
  |---|---|---|---|
  | ⒤검사기 | 신규 술어 + ★**CI 에서 실행 불가**(로컬 전용 ⇒ «기억해서 돌려야» 한다) | 값 복사 2곳 | ★위 ⓒ 두 갈래 |
  | ⒥가리키기 | **0**(이미 착지) | ★**값 복사 자체를 없앤다** | 없음 |
  ★★**⒤의 치명상은 «로컬 전용»이다** — 그러면 그것은 게이트가 아니라 **또 하나의 «매 회차가 기억해야 하는 것»**이고,
  ★**이 티켓이 없애려던 실패 양식(「매 회차가 스스로 알아차려야 한다」)을 그대로 재생산한다.**

- **⒥ 처방 — 사람 규율 한 줄을 «어디에» 둘 것인가**(계약 3 · ★**이 회차에서 쓰지 않았다** — `AGENTS.md`
  재수정이 범위 밖이라 **후속 제안**으로 남긴다):
  > 자리 = `AGENTS.md` §Constraints 의 **「Held by you, not by a machine」** 절(기계가 안 잡는 것만 모아 둔
  > 바로 그 자리 · 이미 「No game bytes」·「Secrets are referenced, never embedded」가 같은 성격으로 산다).
  > 문안 = **「An external contract is referenced, never copied — point at the canon (path + section) and let
  > the ticket's frontmatter bind. Nothing machine-checks this: CI cannot see `~/orchestrator`.」**
  ★**「Secrets are referenced, never embedded」와 «같은 문형»을 고른 것은 우연이 아니다** — 그 절의 기존 항목들도
  전부 「기계가 못 잡으니 사람이 진다」를 명시한 것들이고, ★**«못 잡는 이유»까지 적는 것이 그 절의 관례**다.

- **★재개 조건**(기각은 영구가 아니다): ⑴**self-hosted 러너가 도입**되어 CI 가 `~/orchestrator` 를 볼 수 있게 된다
  ⑵ⓑ의 **«값 복사» 자리가 5곳 이상**으로 는다(오늘 2) ⑶계약 파일이 **이 repo 안으로 벤더링**된다
  (그러면 in-repo↔in-repo 가 되어 기존 9종과 같은 축이 된다 — ★그때는 싸다).

- **게이트**: ★**러스트 무접촉**(diff 3파일 전부 `.md`/`.json`) ⇒ 4게이트 전량 불요. 문서 검사기 5종 전건 통과.
  채택 기록 = worklog `adoptedProposals: ["2026-09-13-merge-section-points-at-the-canon#p0"]`.
