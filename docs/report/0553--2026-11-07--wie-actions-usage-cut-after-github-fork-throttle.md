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

<!-- corpus-name-inflow v1 subjects=6 tree=4cded8869ea231b5 B=0/0 P=0/0 S=0/0 -->
