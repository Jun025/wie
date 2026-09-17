## [2026-09-17] 8주째 보이지 않던 human-step 을 화면이 읽는 자리로 올렸다 — 그리고 검사 목록을 «오늘로» 다시 쟀다 (wie-adopt-slice-d-base-swap-fix3-p1-p0)

**무엇을**: `~/orchestrator/humansteps/wie-main-branch-protection.md` **신설 1장**.
2026-07-22 회차가 done 리포트 §C «본문 안»에만 남겨 둔 human-step(main 브랜치 보호 ruleset 적용)을
운영자 화면이 읽는 자리로 옮겼다. ★**wie 제품 코드 0줄 · Rust 0줄 · wie diff 는 원장 3파일**.

### ⓐ 전제 재측 — 제안은 지금도 참이다
| 축 | 2026-09-17 재측 |
|---|---|
| `humansteps/` 에 branch-protection 카드 | ★**0장**(`branch|protect` 본문 일치 **0건**) |
| `gh api repos/Jun025/wie/branches/main/protection` | **404 Branch not protected** |
| `gh api repos/Jun025/wie/rulesets --jq length` | **0** |
| `branches/main .protected` | **false** |
⇒ ★**강제되는 required check 0개** · 미적용 상태 **무변화**. 카드 신설 후 `humanstep-scan` 이 찍은
`age_days` = ★**57** — 제안의 「약 두 달」이 이제 **기계가 세는 수**가 됐다.

### ⓒ 제안의 처방을 «그대로» 쓰지 않았다 — 검사 목록을 오늘로 다시 쟀다
정본 §C 의 권고 5종을 그대로 베끼면 **틀릴 수 있었다**. 2026-07-22 이후 워크플로가 움직였기 때문이다.
실제 PR 의 check 이름을 전수 조회해 대조했다(PR #180 head · `actions/runs` + `check-runs`):

| context | PR 트리거 | `paths` 필터 | 판정 |
|---|---|---|---|
| `contract` · `build-web` · `rust_ci (ubuntu/macos/windows-latest, stable)` | 있음 | **없음** | ✅ **required 안전**(모든 PR 에 반드시 보고) |
| ★`doc-liveness (weekly)` | 있음 | ★**`['.github/workflows/doc-liveness.yml']`** | ❌ ★**넣으면 모든 PR 영구 교착** |
| `coverage` | ★**없음**(push 전용) | — | ❌ 제외 |
| `dependabot` | 있음 | 없음 | ❌ 제외(일반 PR 에서 `skipped`) |
| `rust_ci (…, beta)` 3종 | 있음 | 없음 | ⚠️ 제외 — 상류 lint 회귀가 **머지를 막는다** |

★★**`doc-liveness (weekly)` 는 2026-09-10 에 생겨 §C 목록에 «존재하지 않는다».** 그것을 required 에 넣으면
`Expected — Waiting for status` 로 영원히 남는다 — 이 저장소 사건 대장이 이름 붙인 바로 그 교착이고,
그 워크플로 자신도 머리에 「★This workflow must NEVER become a required check」라고 적어 두었다.
⇒ 카드의 `how` 에 **«넣으면 안 되는 것» 절**을 리터럴로 넣었다. ★**이것이 이 회차가 §C 에 «더한» 전부다**
— JSON 본문은 §C 와 같다(제안을 넓히지 않았다).

### 왜 «wie 레인이» 이 카드를 만들었나 — 30분 전 회차와 갈리는 선을 명시한다
직전 티켓(`…-corpus-card-first-question-p0`)에서 나는 `humansteps/README.md` 수정을 **거부**했다.
같은 디렉터리인데 이번엔 집행했다. **선은 하나이고 그때와 같다**:

> ★**그 편집이 «다른 레인·estate 가 의존하는 계약»을 리뷰 없이 바꾸는가?**

- `README.md` = **99장 · estate 3곳**을 지배하는 작성 규약 ⇒ **YES** ⇒ 거부(3게이트 밖의 전역 계약 변경).
- 이 카드 = **wie 한 건**짜리 **leaf**. 내용 전부가 wie 지식(wie 의 main · wie 의 check 이름 · wie 의 §C)이고,
  **추가·가역**(파일 1개 삭제로 원상)이며, 최악의 오류도 **운영자 화면에 카드 한 장이 잘못 뜨는 것**이다 ⇒ **NO** ⇒ 집행.
- ★**선례도 그 선 위에 있다**: 직전 형제 회차(`…-triage-r2-p0`)가 wie 레인에서
  `humansteps/wie-p2-corpus-placement.md`(wie 전용 카드)를 고쳐 **착지했다**(mtime 2026-09-16 22:50).
- ★**제안 원문이 「제품 repo 회차가 고칠 수 없다」고 적은 것은 알고 있다** — 그 판단을 뒤집은 근거가 위 셋이다.
  ★그리고 **내용의 권위는 wie 에 있다**: check 이름·`paths` 필터·`doc-liveness` 함정은 orchestrator 레인이
  wie 를 읽어야만 알 수 있는 것이고, 실제로 그 재측이 이 카드의 **가장 값나가는 부분**이다.

### 개악 대조(양방향 · 제품 «소비자» 경유 · 격리 사본)
술어 = `ORCH_OPS=$S bin/humanstep-scan --json`(= `bin/pipeline-feed` 가 `human_steps` 로 싣는 그 생산자).

| 상태 | 결과 |
|---|---|
| 기준선 | `entries=1 how_ruleset_marker=true status=open` **GREEN** |
| ★M1 — `how:` 의 운영자 지시를 markdown **본문**으로 옮김(이 리니지가 이름 붙인 바로 그 실패형) | `how_ruleset_marker=false` **RED** |
| M2 — 카드 파일 제거 | `ABSENT` **RED** |
| 복원 | `how_ruleset_marker=true` **GREEN** |

★**상수 대 상수가 아니다** — 단언이 실제 소비자 출력을 통과한다. ★**라이브 카드는 무접촉**(격리 `ORCH_OPS` 사본).
라이브 검증: `humanstep-scan --json` rc=0 · 이 카드 **WARN 0** · 전체 WARN **0** ·
`--verify` → `wie-main-branch-protection  OPEN — verify 미통과`(**참**이다 — 보호가 아직 꺼져 있다).

### `verify` 술어는 양방향으로 쟀다
```
[ "$(gh api repos/Jun025/wie/rulesets --jq "length" 2>/dev/null || echo 0)" != "0" ] || gh api repos/Jun025/wie/branches/main/protection >/dev/null 2>&1
```
오늘 **rc=1(OPEN)** · 참이 되는 형태 **rc=0(RESOLVED)**. ruleset·classic **둘 다** 받는다.
★`grep -q` 를 파이프 끝에 두지 않았다(`humansteps/README.md` 가 SIGPIPE 오판으로 금지한 관용구).
★**`rules/branch/main` 은 쓰지 않았다** — 이 토큰으로 **404** 라 실측으로 기각했다.

### 잃는 것 / 안 하면 무엇이 나쁜가
**잃는 것**: ⑴★**운영자 화면에 카드가 한 장 는다** — 차단 카드가 아닌데도 목록을 차지한다(그래서 제목·`why` 첫 줄에
「★차단 아님」을 박았다. `blocks: []`). ⑵★**카드가 «반대 방향»으로 낡을 수 있다** — 운영자가 적용하면 `why` 의
실측표가 즉시 거짓이 된다. 그것을 막는 것이 `verify` 이고, 통과하는 순간 `status` 가 `done` 으로 바뀐다.
⑶★**근본 처방이 아니다** — 제안 자신이 적었듯 「done 본문에만 있는 human-step 이 몇 건 더 있는가」는
**세지 않았다**(전수 조사는 이 티켓의 S 범위 밖이고, 넓히면 다른 티켓이다). 후속 제안으로 뺐다.
★**「잃는 것이 없다」고 적지 않는다** — ⑶이 실질이다. 이 회차는 **한 건을 건졌지 병을 고치지 않았다**.
**안 하면**: 준비가 끝난 스위치가 **계속 보이지 않는다**. 이미 8주(=`age_days` **57**)였고, 그 공백이
문서 5곳의 거짓 단언과 게이트② 반려 1건을 낳았다. 그리고 `main` 은 **오늘도 무방비**다 — 지키는 것은 사람뿐이다.

### 인접 관측 1건(이 회차의 산출물은 아니다 · 숨기지 않는다)
검사 이름을 재려고 PR 들의 check-run 을 훑다가 잡았다: ★**PR #177 은 `mergeable=CONFLICTING`(DIRTY)이고
그 head 의 workflow run 이 `coverage`(push 이벤트) **1건뿐**이다** — `pull_request` 이벤트 워크플로 4종이
**하나도 돌지 않았다**. 충돌 PR 은 GitHub 이 병합 ref 를 만들지 못해 PR 이벤트 검사가 생성되지 않는다.
⇒ ★**그 PR 을 「CI 대기 중」으로 읽으면 영원히 기다린다** — 해소는 base 를 당겨 충돌을 푸는 것이다.
(대조군: 같은 날 PR #180 은 workflow run **5종 · check 10개 전건 success**.)

### 전 스위트(문서/외부 원장 diff 도 면제가 아니다 — AGENTS.md)
- `cargo fmt --all -- --check` OK · `clippy --all -D warnings` rc=0 · wasm clippy rc=0 · `+beta clippy` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` — **전건 합산 384 passed / 0 failed**(result 줄 42개 · `FAILED` 0 · `panicked at` 0)
- 검사기: `check-worklog-json` · `check-docs-report-serial` · `check-doc-liveness-parity` · `check-parked-workflows` · `npm run audit` · `check-worklog-coverage` **전건 rc=0**

### ★내 검증이 «불완전했다» — 축 하나를 못 봤고, 소유 레인이 몇 분 만에 잡았다
카드를 올린 직후 `bin/humanstep-scan` 이 **09:33:56** 에 이것을 냈다:
`HUMANSTEP_BLOCKED_BY_UNDECLARED wie-main-branch-protection — 열린 human-step 이 «왜 사람만 할 수 있는지»를
선언하지 않았다(frontmatter `blocked_by`)`. 총괄이 **09:40** 에 `blocked_by: decision` 을 선언하고
사유를 카드에 적었다(그 편집은 그대로 두었다 — 소유 레인의 정당한 보정이고, 되돌리지 않았다).

★★**왜 내가 못 봤나 — 이 한 줄이 이 절의 값이다**: 나는 `humanstep-scan --json` 의 **stderr WARN 을 0으로**
확인하고 「전 축 통과」로 읽었다. 그런데 ★**`blocked_by` 축은 stderr 로 나가지 않는다 — `events/inbox.log`
에 «쓴다»**(그리고 `events/.blockedby-<id>` 마커로 dedup 한다). 실측: `--json`·`--audit`·`--alert`·`--check`·
`--verify` **전 모드에서 그 문자열 0건**이다(격리 사본에서 `blocked_by` 를 지우고 다시 돌려도 0건 —
출력 채널이 애초에 다르다).
⇒ ★**규율**: 카드를 신설하면 `--json` WARN 만 보지 말고 **`events/inbox.log` 의 `HUMANSTEP_*` 를 확인하라.**

★**그리고 이것을 «사고»로 읽지 마라 — 파이프라인이 설계대로 돈 것이다.** 카드가 기계에 닿았고, 탐지 축이
7분 만에 울었고, 소유 레인이 닫았다. 카드가 없었으면 그 축은 **울 대상 자체가 없었다**(그것이 8주였다).

### 총괄이 카드에 더한 판단(여기 요약만 — 정본은 카드 본문)
`blocked_by: decision` 이지 「도구가 안 된다」가 아니다(`gh auth status` 실측: 계정 `Jun025` · 스코프에 `repo`
포함 · 본인 소유 저장소 ⇒ 총괄이 칠 수 «있을» 가능성이 높다). 사람 몫인 이유는 **성격**이다:
⒜저장소 정책 스위치라 이후 **모든 머지**의 동작이 바뀐다 ⒝★**지금 켜면 살아 있는 위험** — 게이트③ 머지
티켓이 큐에 여러 건(wie 2건 포함)이라 required check 가 하나라도 어긋나면 그 회차들이 **그 자리에서 선다**.
⇒ ★**켤 시점은 머지 큐가 빈 때가 싸고, 그 선택이 곧 «결정»이다.**
