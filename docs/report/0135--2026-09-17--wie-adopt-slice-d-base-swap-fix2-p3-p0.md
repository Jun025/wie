## [2026-09-17] upstream 이 «새» 워크플로를 들고 오면 이제 기계가 본다 (wie-adopt-slice-d-base-swap-fix2-p3-p0)

**무엇을**: `scripts/check-upstream-new-workflows.mjs` 신설 — `upstream/main` 의 `.github/workflows/`
**파일 집합**에서 우리 트리의 것을 뺀 차분이 비어 있는지 본다. `.github/workflows/doc-liveness.yml` 의
**마지막 스텝**(주간 스케줄)으로 배선. 제품 동작 0줄 · Rust 0줄.

**왜**: 제안 `2026-09-16-adopt-slice-d-base-swap-fix2-p3#p0`. 형제 검사기 `check-parked-workflows.mjs` 는
**이미 아는 2건**만 보고 그 `PARKED` 배열은 **손으로 관리한다** — base swap 때 2건이 한꺼번에 들어왔고
둘 다 **사람 눈**으로 발견됐다. 셋째가 오면 같은 경로에 의존한다. 그중 `release.yaml` 은 야간 `cron` +
우리 `CLOUDFLARE_API_TOKEN` 으로 pages deploy + 이 repo 에 릴리스 발행까지 무장돼 있었고, 멈춰 있던
이유는 `npm run build:prod` 부재 — 그 회차 표현으로 **「운이지 가드가 아니다」**.

**전제 재측(2026-09-17)**: 제안은 **여전히 참**이다. `scripts/`·`.github/` 전수에서 upstream 워크플로
집합을 비교하는 축 **0건** · `check-parked-workflows.mjs:41` 의 `PARKED` 는 리터럴 2원소 배열 그대로.
오늘의 실측 차분은 **0** 이다(upstream 6 · ours 10 · upstream−ours = ∅) ⇒ 지금 넣으면 green 에서 출발한다.

**사용자 영향**: 없다(제품 동작 0줄). upstream 동기 머지가 «새» 워크플로를 들고 오면 늦어도 **1주 안에**
그 사실이 red 로 뜬다 — 종전에는 아무 신호도 없었다.

### 왜 «손으로 관리하는 목록»을 또 만들지 않았나
차분식이 성립하는 이유는, upstream 워크플로의 **처분 전건이 파일을 우리 트리에 남기기** 때문이다 —
채택하면 우리 것이 되고, 주차해도(이 repo 가 정한 형태 · 삭제는 동기마다 재발) 트리거만 줄인 채 우리 것이 된다.
⇒ 차분은 **스스로** 비어 있는 상태로 돌아온다. 예외는 「의도적 삭제·개명」 하나뿐이고 그것만
`KNOWN_ABSENT`(**빈 배열**)에 선언한다. ★그 배열을 «지금» 채우지 않았다 — 채울 처분이 오늘 0건이다.

### 왜 PR 이 아니라 주간인가
신호는 **upstream 이 움직일 때** 바뀌지 우리가 움직일 때가 아니다. PR 마다 돌리면 upstream 이 워크플로를
하나 올린 날 **열린 PR 전건이 동시에 red** 가 된다 — 그 PR 들이 만지지도 않은 것 때문에.
★이 저장소가 이미 그 형태로 값을 치렀다(2026-09-07 `check-worklog-coverage` red 가 열린 PR 전건을 막았다).
그리고 `doc-liveness.yml` 에는 **red 의 주인**(첫 gate③ 회차)이 이미 리터럴로 박혀 있어, 같은 job 안의
스텝이면 **새 규칙 없이 그 주인을 상속**한다. ★**마지막 스텝**으로 둔 이유: 먼저 두면 이 가드의 red 가
그 job 의 «문서 명령 계측»을 통째로 중단시킨다(호스트 job 의 기능을 깎는다).

### 개악 대조(양방향 · 제품 «호출부» = 실제 트리와 실제 `upstream/main`)
| 개악 | 결과 |
|---|---|
| 우리 트리에서 `.github/workflows/coverage.yml` 제거(= upstream 이 새 파일을 들고 온 것과 «집합적으로 동일») | **rc=1** `upstream 6, ours 9, new 1` + `::error` |
| 같은 상태에서 `KNOWN_ABSENT = ["coverage.yml"]` 선언(탈출구가 실제로 작동하는가) | **rc=0** `declared-absent 1, new 0` |
| `UPSTREAM_REF` 를 없는 ref 로(fail-open 하지 않는가) | **rc=1** `cannot resolve` + 복구 명령 2줄 |
| 전부 복원 | **rc=0** `upstream 6, ours 10, new 0` |

### 4게이트 + 검사기
- `cargo fmt --all -- --check` OK · `clippy --all -D warnings` rc=0 · `clippy --target wasm32-unknown-unknown --all -D warnings` rc=0 · `+beta clippy --all -D warnings` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` — **전건 합산 384 passed / 0 failed**(result 줄 42개 · `FAILED` 0 · `panicked at` 0 · `tail` 미사용)
- `check-doc-liveness-parity` **OK 25줄**(가드 스텝은 `DOC-COPY` 밖이라 parity 대상이 아니다 — 의도) ·
  `check-parked-workflows` rc=0 · `check-worklog-json` OK · `check-docs-report-serial` OK · `npm run audit` PASSED
- YAML 파싱: `doc-liveness.yml` 스텝 **14** · 마지막 = 이 가드 · `on` = `[schedule, workflow_dispatch, pull_request]`(불변).
  ★스텝 이름에 `#p0` 가 들어가 **따옴표가 필요했다** — 무따옴표 스칼라에서 ` #` 는 YAML 주석이라 이름이 잘렸다(실측 후 인용부호로 고침).

### 잃는 것 / 안 하면 무엇이 나쁜가
**잃는 것**: 주간 job 에 네트워크 1회(`git fetch --depth=1 upstream main`)와 스텝 1개. 그리고 ★**호스트 job 의
정체성이 한 칸 흐려진다** — 「AGENTS.md 가 이름 붙인 명령을 돌린다」는 job 에 그렇지 «않은» 스텝이 하나 붙었다
(그래서 파일 머리에 `One rider that is NOT a documented command` 절로 명시 선언했다).
오탐 여지: upstream 이 워크플로를 «개명»하면 옛 이름이 새것으로 보인다 — `KNOWN_ABSENT` 가 그 출구다.
**안 하면**: upstream 동기 머지가 새 워크플로를 들고 와도 **아무 신호가 없다.** `check-parked-workflows.mjs` 는
자기 `PARKED` 2건 밖을 보지 못하고, 그 배열을 갱신하는 주체는 사람 눈이다. 그 눈이 한 번 빗나가면
야간 cron 짜리 배포가 우리 토큰으로 조용히 돌기 시작한다 — 2026-09-16 에 실제로 그 직전까지 갔다.

### 이 가드가 «못 보는 것»(적어 둔다)
- **자기 배선의 삭제**: 누가 `doc-liveness.yml` 에서 이 스텝을 지우면 아무도 모른다(`check-doc-liveness-parity`
  는 `DOC-COPY` 구간만 본다). 이 저장소에 그 형태의 처방이 이미 있다(`check-parity-lock-wired.mjs`) — **후속 제안으로 뺐다.**
- **upstream 의 «수정»**: 의도적으로 안 본다(제안의 트레이드 그대로 — 주간 소음이 된다). 그 축은
  `check-parked-workflows.mjs` 가 트리거 면으로 이미 잡는다.
- **워크플로 «본문»**: 이 가드는 파일 내용을 한 글자도 읽지 않는다.
