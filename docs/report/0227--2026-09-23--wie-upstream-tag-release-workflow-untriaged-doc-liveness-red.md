## [2026-09-23] upstream `tag-release.yaml` 을 park 로 분류 — `doc-liveness` 주간 런 red 해소 (wie-upstream-tag-release-workflow-untriaged-doc-liveness-red)

**무엇을**: upstream 워크플로 `tag-release.yaml` 을 **park** 했다. upstream 원본을 바이트 그대로 가져오고, 머리에
결정 주석 21줄을 붙이고, `scripts/check-parked-workflows.mjs` 의 `PARKED` 에 올렸다. 가드 두 개의 주석 한 줄씩도 맞췄다
(`check-parked-workflows.mjs` 머리 · `engine-contract.yml` 비용 줄).

**왜**: 주간 런 `35473367963`(2026-09-19T22:27Z · schedule)이 마지막 스텝에서 red 였다 —
`upstream 7, ours 10, declared-absent 0, new 1` · `tag-release.yaml`. 문서화 명령이 썩은 것이 아니라
**아직 분류하지 않은 upstream 워크플로**가 있다는 뜻이다. upstream `ead444fb`(#1434, 2026-09-18)가 추가했다.

**그 워크플로가 이 fork 에서 하는 일** (`git show upstream/main:.github/{workflows/tag-release.yaml,scripts/release/tag-version.sh}`):
- 트리거는 `workflow_dispatch` 하나이고, job 은 `github.ref == 'refs/heads/main'` 일 때만 돈다. upstream 에서 이미 수동 전용이다.
- 본문 `tag-version.sh`: 워크스페이스 전체에 `cargo set-version` → `v*` 태그와 `release/v*` 브랜치를 push →
  ★`gh workflow run release.yaml` → 버전 올림 PR 을 연다.
- ⇒ **2026-09-16 에 park 한 Tauri 릴리스 파이프라인(`release.yaml`)의 앞쪽 절반**이다. `release.yaml` 에 남은 유일한 트리거가
  `workflow_dispatch` 라서, 이 파일은 그 파이프라인을 여기서 시작할 수 있는 **유일한 경로**다.
- 다음 버전은 `select(.name == "wie-app")` 에서 읽는다. `wie-app` 은 `Cargo.toml` 이 워크스페이스에서 **제외한** 크레이트다.
  그리고 버전 올림은 `**/Cargo.toml` 을 건드리는데, 그 PR 이 착지하면 `publish-artifact.yml` 의 릴리스 트리거가 된다.

**분류 = park, adopt·KNOWN_ABSENT 는 기각**:
- adopt: `release.yaml` 을 adopt 하지 않기로 한 결정(데스크톱·모바일 호스트는 우리 것이 아니다)을 그대로 따른다. 따로 떼어 결정할 수 없다.
- `KNOWN_ABSENT`: 다음 `git merge upstream/main` 에서 이 파일은 upstream 쪽 **추가**로 충돌 없이 들어온다. 그러면
  결정 머리 주석도, 트리거 가드도 없이 워크플로가 생긴다. 그 스크립트가 적어 둔 「지우는 결정은 동기화마다 되돌아온다」와 같은 모양이다.
- park: 트리거는 바꿀 것이 없다(이미 dispatch 전용). 이 결정이 막는 것은 **upstream 이 나중에 트리거를 «더하는» 경우**다.
  `check-parked-workflows.mjs` 는 이미 always-run 이라 그 머지 PR 에서 red 가 된다.

**검증** (이 브랜치 · 로컬):
- `node scripts/check-upstream-new-workflows.mjs` → `upstream 7, ours 12, declared-absent 0, new 0` · rc=0.
  (`ours` 가 10 이 아니라 12 인 이유: 주간 런 이후 `pr-audit.yml` 이 `main` 에 착지했고, 이 파일이 하나를 더했다.)
- `node scripts/check-parked-workflows.mjs` → `3 parked, 0 violation(s)`.
- 변이 테스트: 복사본에 `push:` 트리거를 넣으면 → `tag-release.yaml is no longer parked: triggers = [push, workflow_dispatch]`,
  `2 parked, 1 violation(s)`. 되돌린 뒤 다시 green.
- upstream 원본과 바이트 일치: `diff <(git show upstream/main:.github/workflows/tag-release.yaml) <(tail -n +22 …)` → 차이 0.

**아는 한계**:
- 수동 dispatch 는 **여전히 가능하다** — park 는 트리거 표면을 지키는 것이지 버튼을 막는 것이 아니다(`release.yaml` 과 같은 선택).
  지금은 `tag-version.sh` 가 우리 트리에 없어서 스크립트 단계에서 실패한다. 다음 upstream 머지가 그 스크립트를 가져오면 그 실패도 사라진다.
  머리 주석은 이 파일만 따로 다시 열지 말고 `release.yaml` 과 함께 결정하라고 적어 두었다.
- 주간 런이 green 인지는 이 PR 이 **머지된 뒤** `gh workflow run doc-liveness.yml -R Jun025/wie` 로 확인해야 한다.
  PR self-test 는 이 PR 에서 돌지 않는다 — 그 트리거는 `doc-liveness.yml` 파일 자체가 바뀔 때만 발화하고, 이 PR 은 그 파일을 건드리지 않는다.

**사용자 영향**: 없음. CI 파일과 가드 목록만 바꿨고 제품 코드는 0줄이다. 다음 주간 `doc-liveness` 런의 red 한 건이 사라진다.
