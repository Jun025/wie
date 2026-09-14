## [2026-09-14] 「외부 계약은 가리킨다, 복사하지 않는다」 — 한 줄 (wie-agents-md-external-contract-is-referenced-never-copied)

- **무엇을**: `AGENTS.md` §Constraints 「Held by you, not by a machine」에 **불릿 1개**.
  ★`git diff --numstat AGENTS.md` = **`1 0`** ⇒ **추가 1줄 · 삭제 0 · 그 절의 기존 줄 무접촉**.
  ```
  - **An external contract is referenced, never copied.** Point at the canon (path + section) and let
    the `-merge` ticket's `merge_strategy:` frontmatter bind; nothing here can check that, because CI
    never sees `~/orchestrator`.
  ```

- **★왜 이 자리인가 — ⓐ 를 «읽어서» 확인했다**: 그 절의 머리가 「**no gate catches these**, and a table row
  would delete the working part」이고, 기존 불릿 4개가 전부 **「굵은 주장.」 + 「무엇이 (부분적으로) 집행하는가
  / 왜 기계가 못 잡는가」** 형태다. ⇒ ★**계약 2(「왜 기계가 안 잡는지를 같은 줄에」)는 이 절의 «문형» 그 자체**라
  따로 덧붙일 것이 없었다.

- **★★ⓑ 중복 점검 — 없었다(있었으면 더하지 않고 끝냈다)**: 그 절 안의 `contract|canon|orchestrator` 히트 **0**.
  저장소에 이미 있는 유사 문장은 ⒜`AGENTS.md` §Git Workflow 「The landing strategy is **not this file's to
  state**」 ⒝`CONTRIBUTING.md` 「how that merge lands is **not this file's to state**」인데, ★**둘 다 «한 사례»
  (착지 방식)에 대한 것이지 «일반 규율»이 아니다.** ⇒ 일반형이 없어서 매번 사례마다 다시 발견해야 했다 —
  그것이 이 줄이 메우는 자리다.

- **⑵ 이웃과 문형 대조**(세 줄 나란히 · ★`Secrets` 줄 **바로 다음**에 놓았다):
  ```
  - **Secrets are referenced, never embedded or printed.** `.dev.vars*` is git-ignored …
  - **An external contract is referenced, never copied.** Point at the canon (path + section) …
  - **Never rewrite published history**: no `git push --force`, no rebasing a branch that has been pushed. …
  ```
  ★**배치가 논거다** — 두 줄이 **`X is referenced, never Y`** 라는 **같은 구문**을 이루어, 읽는 사람이
  「이건 시크릿 규율과 같은 계급이구나」를 **문장 형태만으로** 알게 된다.

- **★대가를 알고 썼다**(계약 3 · 제안이 스스로 적은 것): ⒜**규율은 기계가 아니다** — 읽지 않는 회차에는 무력하다
  ⒝**이 파일은 이미 길다** ⇒ ★**새 절 0 · 한 줄 · 사례 서술 0**(9일 모순의 이야기는 `docs/report/0107`·`0109` 가
  이미 담고 있어 **여기 옮기지 않았다** — 옮기면 그것이야말로 이 규율이 금지하는 «복사»다).

- **★선행 판정 ⒥ 불변**: 검사기 **신설 0**. 이 줄은 그 판정이 **남긴 방어선**을 적은 것이지 그것을 뒤집지 않는다.

- **게이트**: ★러스트 무접촉(diff 3파일 전부 `.md`/`.json`) ⇒ 4게이트 전량 불요. 문서 검사기 5종 전건 통과.
  채택 기록 = worklog `adoptedProposals: ["2026-09-13-doc-contract-divergence-verdict#p0"]`
  (★**0-기반 대조 완료** — 그 파일 `proposals[0]` 의 `target` 이 「`AGENTS.md` §Constraints → 「Held by you,
  not by a machine」」로 이 회차가 고친 자리와 **일치**한다).
