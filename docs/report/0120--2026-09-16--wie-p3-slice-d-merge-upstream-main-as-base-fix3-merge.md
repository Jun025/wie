## [2026-09-16] 조각 D 착지 — upstream/main 을 base 로 삼는다 (wie-p3-slice-d-merge-upstream-main-as-base-fix3-merge)

**무엇을**: PR #161(`wie-p3-slice-d-merge-upstream-main-as-base` 리니지 4회차)을 게이트③에서
`--merge`(parents=2)로 `main` 에 착지시켰다. 코드 변경 0 — 이 회차가 만진 것은 원장 3파일뿐이다
(이 리포트 · `STATE.md` 완료 항 · `docs/worklog/2026-09-16-slice-d-gate3-merge.json`).

**왜 `--merge` 인가 (★`--squash` 금지)**: `wie` 는 `contracts/upstream-sync-repos.conf` 등재 repo 다.
스쿼시하면 부모 2개가 1개로 접혀 `merge-base(main, upstream/main)` 이 `fa641a8a`(2026-06-10 이후 불변)로
**되돌아가고**, 조각 D 가 산 것 — 즉 「upstream/main 을 base 로 삼는다」 그 자체 — 이 통째로 사라진다.
회차·게이트② 검수자가 독립으로 같은 근거를 적었고, 머지 티켓 frontmatter 의 `merge_strategy: merge` 가
그 선언이다(집행 스니펫이 그 줄을 **읽어** 명령을 만든다 — 고를 것이 없다).
그리고 이 별도 `-merge` 티켓이 존재하는 이유가 정확히 그것이다: 등재 repo 는 게이트③을 묶을 수 없다
(묶음 경로엔 `merge_strategy:` 를 담을 파일이 없어 `queue-lint` 검사 22 와 집행 STOP 두 방어선이 함께 사라진다).

**사용자 영향**: 이 착지는 **배포를 수반한다** — 아래 3-a 예측 참조. 엔진 아티팩트가 새로 발행되고
(`publish-artifact.yml` → GitHub Release + `repository_dispatch` → otterpebble featurephone 전파),
Cloudflare Pages 배포가 돈다(`web.yml`). 브라우저 호스트 `wie_featurephone` 의 export 표면은
직전 회차들이 계약(`docs/contracts/featurephone-engine-contract.json`)과 동봉해 맞춰 두었다.

### 게이트③ 실측

| 축 | 값 |
|---|---|
| 검수 핀(review.md 줄2) | `7a439bd040dc64b64d5cdb286823e1aba6bc5f31` |
| 동봉 «전» PR head | `7a439bd040dc64b64d5cdb286823e1aba6bc5f31` — ★**핀과 일치**(원격 조회) |
| 2-b⒟ 사전 `ci-presence` | `rc=0 CI_GREEN` — 검사 5건(name·event) 전건 완료·성공 ⇒ 갈래 판정 = 「선재 red 없음」 |
| 2-b-2 주기 의무 | 검사기 **1건**(`scripts/check-worklog-coverage.mjs`) · `landed 96 / last recorded 93` ⇒ `(96+1)−93 = 4 < 10` ★**내 착지는 주기를 넘기지 않는다** |
| 3-b base 낡음 | 판별 도구 **0건**(빈 출력) ⇒ **건너뜀**. `mergeable MERGEABLE` · `mergeStateStatus CLEAN` |
| 계약 5 자식 PR | `--base wie-p3-slice-d-merge-upstream-main-as-base --state open --limit 200` → 아래 회신 참조 |

### 3-a 배포 예측 — 원천은 `origin/main...HEAD`(착지 diff), **460파일**

`ls .github/workflows/` 전수 대조(도구 탐침 빈 출력 ⇒ ⒝ 수동 대조):

| 워크플로 | `on` | 착지 시 | 성격 |
|---|---|---|---|
| `publish-artifact.yml` | push main + paths | ★**발화** | **배포** — 엔진 아티팩트 Release + `repository_dispatch` → otterpebble |
| `web.yml` | push `[main,…]` · paths 없음 | ★**발화** | **배포** — Cloudflare Pages + 배포 후 `verify-browser.mjs` |
| `engine-contract.yml` | push main · paths 없음 | 발화 | 검사(계약·round-trip) |
| `rust.yml` | push main | 발화 | 검사(3 OS × 2 툴체인) |
| `coverage.yml` | `[push]` | 발화 | 검사 |
| `release.yaml` · `web.yaml` | `workflow_dispatch` **전용** | 미발화 | ★이번 리니지가 «주차»한 upstream 워크플로 둘 |
| `doc-liveness.yml` | schedule · dispatch · PR-paths | 미발화 | — |
| `rust-audit.yaml` | schedule | 미발화 | — |
| `dependabot.yaml` | pull_request | 미발화 | — |

★**`publish-artifact.yml` 은 paths 대조 없이도 발화한다** — 그 파일 **자신**이 자기 `paths` 의 원소이고
이 착지가 그것을 `M` 으로 만진다(계약 3-a⒞ 의 「공유 앵커」 형상 그대로). 그것을 빼고 세어도
`**/*.rs`(수백) · `Cargo.lock` · `**/Cargo.toml` · `assets/neodgm.ttf` 가 각각 독립으로 매치한다.
★**착지 diff 로 셌다** — 마지막 회차 delta(`Cargo.toml`·`Cargo.lock`·원장 = 5파일)로 세면
`web.yml` 과 `publish-artifact.yml` 을 **둘 다 놓친다**.

### 한계 — 숨기지 않는다

- **배포 잡은 PR 에서 돌아 본 적이 없다.** `web.yml` 의 배포·검증 스텝은 전부
  `github.event_name == 'push'` 게이트 뒤에 있다 ⇒ PR 의 `build-web pass` 는 **빌드까지만** 증명한다.
  배포 자체의 판정은 착지 후 `web.yml` 의 `verify-browser.mjs` 스텝과 self-verify 가 낸다.
- **`wie-app` 단독 빌드는 여전히 깨져 있다**(직전 회차가 `exclude` 로 뺀 대가 · `0119` 에 기록).
  이 착지가 바꾸지 않는다.
