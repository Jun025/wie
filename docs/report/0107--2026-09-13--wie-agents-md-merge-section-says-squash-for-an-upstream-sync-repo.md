## [2026-09-13] `-merge` 절이 스쿼시를 «지시»하던 것을 «정본을 가리키게» 바꿨다 (wie-agents-md-merge-section-says-squash-for-an-upstream-sync-repo)

- **무엇을**: `AGENTS.md` §Git Workflow → 「For the `-merge` task only」의 두 불릿을 **교체**했다.
  ⑴착지 방식은 **이 파일이 정하지 않는다** — `-merge` 티켓의 `merge_strategy:` frontmatter 를 읽고,
  그 집행 정본은 `templates/merge-ticket.tpl` **§4-A** 다(티켓 선언이 여기 적힌 어떤 절차보다 **이긴다**).
  이 repo 의 값은 `merge` · ★**never `squash`** + **왜** 1줄(upstream 계보 = 재정렬 오버레이가 얹히는 바로 그것 ·
  `rustjava` **4회 연속** 소실). ⑵`--delete-branch` **금지**(원격은 이 repo 의 `deleteBranchOnMerge` 가 지우므로
  플래그가 얻는 것이 없고, **로컬 브랜치까지** 지운다 — 남의 세션이 체크아웃 중일 수 있다).
  ★**절차·플래그 사본 0**(계약 1) · **검사기 0**(계약 4) · 러스트·계약·템플릿 **무접촉**(계약 3).

- **★★ⓐ 반증 실패 — 그리고 결함의 «이름»이 더 나쁜 쪽으로 바뀌었다: 문서 대 계약이 아니라 «문서 대 자기 자신»이다.**
  그 절은 조건·예외 **없이** `gh pr merge --squash --delete-branch` 를 지시했다(두 회차의 신고가 옳았다).
  ★**그런데 같은 파일이 §Definition of Done 에서 이미 이렇게 적는다**:
  「**This repo is registered as an upstream-sync fork and must *not* squash-merge**, so landings arrive
  as merge commits: PR #69 landed that way on 2026-09-03」.
  ⇒ ★**한 파일 안에서 정면으로 어긋나 있었고, 그 상태가 «몇 주» 살아 있었다.** 이것이 이 회차의 실제 발견이다 —
  티켓이 세운 「문서 ↔ 계약」 프레임보다 **고치기 쉽고 더 부끄러운** 형태다(외부 계약을 몰라서가 아니라,
  자기가 이미 적어 둔 것을 **자기 다른 절이 무시**했다).

- **ⓑ 등재 확인**: `~/orchestrator/contracts/upstream-sync-repos.conf` 에 **`repo wie`** 지금도 실재
  (`# dlunch/wie · behind 1067`) ⇒ 충돌의 전제는 **살아 있다**.

- **★ⓒ 읽는 주체 — 「아무도 안 읽는다」가 아니었다. 값이 높다.**
  | 축 | 실측 | 뜻 |
  |---|---|---|
  | 에이전트(사람 포함) | `CLAUDE.md:1` = **`@AGENTS.md`** | ★이 repo 의 **모든 세션이 자동 적재**한다 — 머지 회차 포함 |
  | 기계 집행 경로 | `templates/merge-ticket.tpl` 4-A 스니펫은 **티켓 frontmatter** 만 읽는다(`AGENTS.md` 참조 1건은 무관한 worklog-owner 결정 서술) | 문면은 **집행에 배선돼 있지 않다** |
  ⇒ ★**기계는 옳게 집행하는데 «읽히는 문면»이 틀렸다** — 그래서 실패 양식은 「잘못 착지한다」가 아니라
  ★**「매 회차가 그 모순을 스스로 알아차려 이겨 내야 한다」**였다. 실제로 **두 회차가 각자 알아차렸다**
  (`…-align-with-upstream-merge` 게이트③ · `…-paints-55-…`). ★**알아차리지 못한 회차가 나올 때가 사고다.**

- **⑴ 스쿼시 «지시» 잔존 0 — `grep` 출력 그대로**:
  ```
  546:  "squash".** This repo is registered as an upstream-sync fork and must *not* squash-merge, so
  605:  `merge` — **never `squash`**: it is a registered upstream-sync fork, so a squash folds the two parents
  609:  and must *not* squash-merge"); what used to stand in this spot was a `--squash --delete-branch` recipe
  611:- **Do not pass `--delete-branch`**, for the reasons §4-A measured: the remote branch is deleted by this
  ```
  4건 전부 ★**금지 또는 과거 서술**이다(:546 선재 금지 · :605 새 금지 · :609 걷어낸 사본의 서술 · :611 플래그 금지).
  ⇒ **지시문 0건.** ※`git diff --numstat AGENTS.md` = **`15 2`** ⇒ ★**삭제행 2 실재**(«치환»이 일어났다 —
  스쿼시 불릿 2줄이 사라졌다). 삭제 0 이면 조용한 롤백을 의심해야 하는 자리다.

- **★계약 2(대가) — 한 홉을 값하게 했다**: 「정본을 가리킨다」는 독자가 티켓/템플릿으로 한 번 더 가야 한다는 뜻이라,
  그 자리에 **왜**를 한 줄로 남겼다(계보가 접히면 재정렬 오버레이가 얹힐 대상이 사라지고 **되돌릴 수 없다** ·
  `rustjava` 4회). ★**값은 복사하지 않았다** — `merge_strategy: merge` 토큰만 쓰고 **명령·플래그 사본은 두지 않았다**
  (계약 1 이 허용한 그 경계 그대로).

- **부수 정정 1건(같은 불릿 안이라 함께 고쳤다)**: 걷어낸 사본은 `git branch -D` 를 「스쿼시 브랜치는 `-d` 로
  인식되지 않으므로」로 **정당화**했다 — ★그 이유 자체가 «스쿼시 전제»의 하류다(머지 커밋이면 `-d` 가 듣는다).
  그리고 「Sync local `main` afterwards」는 **항상 가능하지 않다**: 2026-09-13 게이트③ 실측으로
  `git checkout main` 과 `git branch -D` 가 **둘 다 거부**됐다(각각 다른 워킹트리가 점유) ⇒ 그 정리는
  **그 트리의 소관**임을 같은 자리에 적었다.

- **⑶ 문서 검사기 전건 통과**: `check-doc-liveness-parity` **OK(25줄 전건 커버 · nothing extra)** —
  ★편집분이 **fenced `sh` 블록이 아니라** 리스트·산문이라 패리티 무영향 · `check-worklog-json` OK(88) ·
  `check-docs-report-serial` OK(중복 0) · `check-worklog-coverage` **9/10 · 약속 최신**(landed 86 / 기록 83) ·
  `npm run audit` **PASSED**.
  **⑷ 러스트 무접촉** ⇒ 4게이트 전량 불요(diff 4파일 전부 `.md`/`.json`).

- ★**연번은 도구에 물었다**(디렉터리 `max+1` 금지 규율): `--next-serial` → **0107**
  — 디스크 기준 0106 이지만 열린 PR 이 **0105(#151)·0106(#152)** 를 claim 하고 있었다. ⇒ 그 규율이 이번에 값했다.

- **범위 밖 무접촉 확인**: `contracts/upstream-sync-repos.conf` · `templates/merge-ticket.tpl` ·
  `AGENTS.md` 의 다른 절(§DoD :555 문장은 **인용만** 했다) · 착지한 PR·태그 · 러스트 코드 전부.

- **남긴 값**: worklog 제안 **1건** — 「문서가 계약과 어긋나도 아무것도 울지 않는다」. ★**일부러 만들지 않았고**
  (계약 4), 제안 본문에 ★**어휘 술어의 함정**을 실측으로 적어 뒀다(`--squash` 를 금지어로 넣으면 이번 새 문면의
  `never squash`·과거 서술까지 문다 — 잔존 4건 전부 정당하다). ⇒ 만들려면 술어가 «어휘»가 아니어야 한다.
