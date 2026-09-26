## [2026-09-26] jvm_support pin 테스트 게임 목록 — 머리 주석 한 문장 → 행별 주석 (wie-jvm-support-pin-test-header-comment-merge-hotspot)

**무엇을** — `wie-lgt/src/runtime/java/jvm_support.rs` 의 `abi_rows_cover_the_indexes_titles_actually_dispatch_on`
머리 주석에 «, <게임> <클래스> index N» 으로 이어 붙이던 게임 목록을, 단언 표의 각 행 바로 위 `// <게임>` 한 줄로 옮겼다.
머리 주석은 «행을 더해라, 이 주석을 고치지 마라» 규칙 3줄 + 기존 CLDC 경고만 남긴다. 내용(현재 합집합 16개 행 귀속)은 전부 보존.
동작 코드·단언 변경 0.

**왜** — ABI 행을 더하는 PR 마다 같은 문장 끝을 고쳐 형제 PR 이 반드시 충돌했다(#299·#292 두 번 다 해소 = 목록 합집합뿐).
행 «위» 줄을 택한 이유: 행 «끝» 주석은 rustfmt 가 이웃 행끼리 열 정렬해서, 긴 행 하나가 이웃 행을 다시 고쳐 같은 온상을 되살린다(실측 — 첫 시도에서 정렬이 걸렸다).

**측정** — 두 가상 브랜치(A: `DataInputStream 19` 를 DIS 22 앞에 · B: `String 30` 을 String 27 뒤에, 각자 옛 방식대로 머리 주석에도 덧붙임):
새 배치 위 `git merge-tree --write-tree A B` → **rc=0**, 같은 두 편집을 `origin/main`(옛 배치) 위에서 → **rc=1** (`jvm_support.rs` stage 1/2/3).

**한계** — 두 PR 이 표의 «같은 자리»(바로 인접 행)에 넣으면 여전히 충돌한다(git 은 두 삽입 사이에 불변 줄 1개가 필요하다). 그건 표 자체의 성질이고 이 라운드 범위 밖.
열린 형제 PR 중 이 주석을 고친 것은 이 변경과 한 번 더 충돌한다 — 해소는 «자기 게임을 자기 행 위 한 줄로» 옮기는 것.

**사용자 영향** — 없음(테스트 주석).
