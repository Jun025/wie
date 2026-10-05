## [2026-10-05] compat.json 되돌림 검사 — 값이 아니라 계보로 잰다 (wie-compat-json-unintended-row-revert-contract-check)

### 무엇을
- `scripts/check-compat-revert.mjs` 신설 · `engine-contract.yml` `contract` 잡(상시 실행)에 `--selftest` 와 함께 배선.
- 판정: fork(브랜치가 main 에서 처음 갈라진 점) ≠ mb(merge-base) 인 필드 = 브랜치가 main 머지로 «받은» 변경. 그중
  착지값(`git merge-tree main 브랜치`) == fork 값 · ≠ mb 값 · ≠ main 값이면 RED(행 sha12 · 필드 출력).
  의도한 되돌림은 그 PR 이 더한/고친 worklog 의 `intendedCompatReverts: ["<sha12>:<필드>"|"<sha12>:*"]` 로만 통과.
- 성공 줄에 «착지 기준 바뀐 행 N»(main ↔ merge 결과)을 낸다 — 검수가 인용할 수.
- `AGENTS.md` §Landing paperwork 1줄 · `docs/contracts/featurephone-public-data.md` §4 1줄.

### 왜 — 그리고 티켓 전제가 틀렸다는 실측
티켓은 #475 F1(17행)·#478 F1(23행)을 «green 인 채 착지할 뻔한 되돌림»으로 봤다. 재측 결과 **둘 다 되돌림이 아니다**:

| 비교 | 다른 행 |
|---|---|
| main `48137e48` ↔ #475 head `875f3a71` (2점 diff — 검수가 쓴 것) | 30 |
| merge-base `9b4eb62e` ↔ `875f3a71` (PR 이 쓴 것) | 14 |
| main ↔ `git merge-tree 48137e48 875f3a71` (실제 착지할 내용) | **14** |

- #475 의 merge-base 는 `48137e48` 이 아니라 `9b4eb62e`(#473 이전)다. 브랜치는 #473 을 받은 적이 없고, git 3-way 머지는
  #473 의 17행을 그대로 남긴다. 2점 diff 가 main 쪽 변경을 «뒤집어» 보인 것이다. 현 main 도 #473 값을 갖고 있다
  (예: `7089dec0e8df` `sound=no` + 설정 안내 문구).
- #478 F1 의 23행도 회신 스스로 「main → 875f3a71 의 차이와 정확히 같다」고 적었다 — 같은 2점 비교다.
- ⇒ 티켓 Acceptance 의 «#475 옛 head `875f3a71` 에서 red(17행)»는 **맞출 수 없고 맞춰서도 안 된다** — 그 head 는 착지해도
  아무것도 되돌리지 않는다. 대신 그 형상이 «실제로» 되돌림이 되는 경우를 합성해 재현했다(아래 ⑵).

### v1 을 버린 이유(값 기준)
첫 판은 «착지값이 main 최근 10판 중 어느 판의 값이면 red»였다. 최근 compat 착지 6건 중 **3건**(#465·#471·#476)이 red —
엔진 수정이 `limited → playable` 로 옛 값을 «되살리는» 것과 낡은 재생성은 값만으로 구분되지 않는다. 가르는 것은
«브랜치가 그 변경을 받은 적이 있는가», 즉 계보다.

### 검증
- `--selftest` 6형: ⑴정상 0 · ⑵받은 행을 받기 전 값으로 1 · ⑵'받은 행 삭제 1 · ⑶선언 0 · ⑷v1 오탐 형 0 · ⑸main 이 같은 값으로 되돌림 0 — 전부 기대값.
- 실사례: `--head 875f3a71 --base 48137e48` → OK «받은 적 없다 · 착지 기준 바뀐 행 14» · `--head 55b01999 --base d4a0f330`(고친 head ·
  당시 main) → OK(fork `9b4eb62e` → mb `d4a0f330` · 14행) · `ddbd8e84`(#478) ↔ `d4a0f330` → «미대조 — 충돌»(GitHub 도 머지 불가).
- 합성 위험형: `875f3a71` 이 `48137e48` 을 머지(=#473 수용)한 뒤 `875f3a71` 의 compat 로 다시 씀 → **RED 17행 28필드**
  (검수 F1 의 17행과 같은 집합). CI 형(그 브랜치와 main 의 머지 커밋을 HEAD 로 체크아웃 · HEAD^2 판독)에서도 같은 RED.
- 오탐 측정: main first-parent 의 compat 착지 머지 28건(`--head M^2 --base M^1`) → **red 0**. 그중 9건이 «main 을 받은» 경로를 탔다.

### 한계
- 판정은 «브랜치가 받은 main 변경»만 본다. 브랜치가 받지 않은 main 변경은 git 머지가 지키므로 볼 필요가 없다.
- main 과 충돌하면 «미대조»로 green — 충돌은 GitHub 가 이미 막는다. 충돌을 푼 뒤 다시 잰다.
- 판정 로직·compat 데이터 무변경.

### 사용자 영향
없음(CI 검사 추가). 검수가 compat 행 변경을 2점 diff 로 세어 생기는 오반려가 줄어든다.
