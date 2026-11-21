## [2026-11-07] Actions 사용량 줄이기 — GitHub 가 이 포크의 Actions 를 «사용량 규모»로 멈춤 (wie-actions-usage-cut-after-github-fork-throttle)

**무엇을** — 워크플로 트리거를 줄였다. 필수 검사 5종의 이름은 그대로 PR 에 보고된다.

| 워크플로 | 전 트리거 | 후 트리거 | 주간 잡분 전(실측) → 후(추정) |
|---|---|---|---|
| Rust CI (`rust.yml`) | PR·main push · 6다리 전부 | PR·main push 는 ubuntu/stable 만 실행 · 나머지 5다리는 스텝 전부 skip(빈 통과) · 주 1회 schedule·`workflow_dispatch` 는 6다리 전부 · PR 동시성 취소 | 854 → 약 160 |
| coverage | `on: [push]`(모든 브랜치) | 주 1회 schedule·`workflow_dispatch` | 401 → 약 19 |
| Web (`web.yml`) | push main+`feat/cf-web-service`+`feat/ux-redesign` · PR | push main · PR(빌드만 · 배포는 이미 push 전용) · PR 동시성 취소(main push 는 안 끊음) | 137 → 약 130 |
| Engine contract | 그대로(이미 동시성 취소) | 그대로 | 88 → 88 |
| 나머지 | 그대로 | 그대로 | 약 53 → 약 53 |
| 합 | | | **약 1,530 → 약 450** |

**왜** — `Jun025/wie` 의 Actions 가 2026-11-04T05:40Z 이후 아무 이벤트에도 run 을 만들지 않는다. Actions 탭 배너:
«Workflows are not currently being run on this fork due to the scale of GitHub Actions usage.» 다시 켜는 것은
운영자 몫이고, 줄이지 않고 다시 켜면 같은 정지가 되풀이된다.

**실측(2026-10-28~11-04 · 잡 시작~끝 합)** — `gh run list --created 2026-10-28..2026-11-04` 의 run 마다 `actions/runs/<id>/jobs`:
Rust CI 21 run(PR 15 · push 6) **854 잡분** · coverage 21(전부 push) **401** · web 21 **137** · contract 21 **88**.
Rust CI 다리별 평균: windows stable/beta 9.7 · macos stable 6.3 · beta 5.9 · ubuntu stable 4.6 · beta 4.5 분.
후 추정: Rust CI = 21 × (4.6 + 빈 다리 5개 × 약 0.2) ≈ 118 + 주간 전체 1회 ≈ 41 ⇒ 약 160. coverage = 주 1회 × 약 19.
web 은 죽은 두 브랜치(원격에 없음 — `git ls-remote --heads` 빈 출력) 분이 0 이었으므로 줄어드는 몫은 PR 재push 취소뿐이다.

**필수 검사(경계)** — `AGENTS.md` 의 REQUIRED-CHECKS 목록(ruleset «main protection»): `contract` · `build-web` ·
`rust_ci (ubuntu-latest, stable)` · `rust_ci (macos-latest, stable)` · `rust_ci (windows-latest, stable)`.
`branches/main/protection` 은 ruleset 을 못 보므로(404) 그 목록을 경계로 썼다.
- `contract`(engine-contract.yml) · `build-web`(web.yml): 트리거의 `pull_request` 그대로 · `paths:` 없음.
- `rust_ci (…)` 세 이름: 매트릭스(`os` × `rust`)를 그대로 두고 스텝마다 `if: env.FULL == 'true'` 를 붙였다.
  잡 수준 `if:` 는 쓰지 않았다 — 건너뛴 매트릭스 잡은 펼쳐지지 않은 이름 하나로 보고돼 필수 검사가 «대기»에 남는다.
  스텝이 전부 skip 된 잡은 success 로 끝난다. `rust: [stable, beta]` 줄을 그대로 둔 것은 DoD 파리티 락(`dod_ci_parity.rs`)이 그 줄을 읽기 때문이다.

**사용자 영향** — 없다(플레이어 화면 변화 0). 개발 쪽 대가: macOS·Windows·beta 의 회귀는 PR 이 아니라 주간 run 에서 보인다
(Windows CRLF 가 대표례 — `dod_ci_parity.rs` 의 CRLF 시험). 플랫폼에 닿는 PR 은 `gh workflow run rust.yml -R Jun025/wie --ref <branch>` 로
전체 매트릭스를 손으로 돌린다. coverage 도 같다(테스트를 지우거나 옮기는 PR).

**검증** — 4게이트 + `cargo +beta clippy --all -- -D warnings` 로컬 green(`cargo test --all` 837 passed · 0 failed) ·
`check-parked-workflows` · `check-doc-liveness-parity` · `check-linux-system-deps` · `check-parity-lock-wired` rc=0 ·
워크플로 12개 YAML 파싱 OK. Actions 가 꺼져 있어 이 PR 의 CI 는 다시 켜지기 전까지 돌지 않는다.

**[2026-11-22] 재배차 — Actions 가 다시 돈다** — `gh workflow run rust-audit.yaml --ref main` 이 run `44549723803`
(2026-11-19T14:59:26Z)을 만들었다(종전 HTTP 422 사라짐 · 그 run 의 failure 는 11-01~11-04 의 Security audit 와 같은 기존 red).
`origin/main` 은 11-07 이후 그대로(`2e66f9d68`)라 병합할 것이 없었고, 이 절을 담은 커밋이 정지 뒤 이 저장소의 첫 PR run 을 깨운다.
그 run 이 위 «스텝이 전부 skip 된 잡은 success 로 끝난다»의 첫 실측이다 — 결과는 `gh pr checks 586 -R Jun025/wie` 와
이 티켓의 회신에 있다(여기 다시 적으면 그 커밋이 run 을 한 번 더 만든다).

**[2026-11-22] 반려 수정(검수 F1~F4 · `wie-actions-usage-cut-after-github-fork-throttle-fix`)** — 게이트를 PR 에서 주간으로 옮긴 대가를
에이전트가 읽는 자리에 적고, 주간 red 에 소유자를 붙이고, `FULL` 한 줄을 락에 넣었다.
- **대가의 자리(F1)**: `rust.yml` 머리에 손 실행 명령 · `AGENTS.md` REQUIRED-CHECKS 블록 바로 아래에 «macOS·Windows 필수 검사의 PR green 은
  아무것도 재지 않는다» · 「Then read the PR's CI」 문단을 현 사실로(coverage·5다리는 PR 에서 안 돈다 · 플랫폼에 닿는 PR 은 손 dispatch 결과를 인용).
- **주간 red 소유(F2)**: doc-liveness 와 같은 꼴 — red 뒤 첫 게이트③ 회차가 `rust.yml`·`coverage.yml` 의 최근 schedule run 을 읽고 red 면 발권.
  «목록이 비었거나 8일 넘게 낡았다»는 green 이 아니다(재개 뒤 schedule 이벤트 0건 — 이 날짜 `gh run list --event schedule` 최신이 11-04).
  ★첫 주간 run 은 red 가 예정이다: beta clippy(`wie_cli/src/window.rs` · 분리 PR #591 미착지).
- **`FULL` 락(F3)**: `dod_ci_parity` 에 축 C — `FULL:` 식에 `(matrix.os == 'ubuntu-latest' && matrix.rust == 'stable')` 가 `||` 대안으로 남아 있고
  매트릭스에 그 다리가 있어야 green. 검수 개악 M-B(`FULL: ${{ github.event_name == 'schedule' }}`): 종전 rc=0 · 11 passed → **rc=101 · 2 failed**.
  스텝 `if:` 는 여전히 안 본다(M-C) — 천장 10항에 적었고 1·9항을 «주간 run» 사실로 고쳤다.
- **동시성(F4)**: `rust.yml` 그룹에 `github.event_name` — schedule run 의 ref 도 `refs/heads/main` 이라 main push 와 한 그룹이면 대기 중인
  주간 run 이 다음 main push 에 대체된다. `web.yml` 은 schedule·dispatch 트리거가 없어 그룹은 그대로 두고 주석만 «진행 중인 run 은»으로 좁혔다.
- **안 한 것**: 빈 다리의 `runs-on` 을 ubuntu 로 돌리기(검수 S5). `runs-on` 은 `env` 를 못 읽어 이벤트 조건을 한 번 더 적어야 하고, 두 식이
  어긋나면 주간 run 의 macOS 다리가 ubuntu 에서 돌고도 그 이름으로 green 이 된다 — 착지 전에 돌려 볼 수 없는 경로에 조용한 고장점을 하나 더 만든다.
- 분모 `~1,480`(4종 합)은 위 표의 `약 1,530`(전체)으로 맞췄다(`rust.yml`·`coverage.yml` 주석).
- `wie_validate.rs` 의 doc 주석 "runs it on all six matrix legs" 는 그대로다 — 제품 소스 파일이고, 그 파일을 건드리면 유입 도구가
  파일에 원래 있던 제목 3종을 이 PR 의 유입으로 센다(재 봄: B=4/3). 사실은 `AGENTS.md` 의 그 문단이 이제 말한다.

<!-- corpus-name-inflow v1 subjects=6 tree=79d717ad7ba392b2 B=0/0 P=0/0 S=0/0 -->
