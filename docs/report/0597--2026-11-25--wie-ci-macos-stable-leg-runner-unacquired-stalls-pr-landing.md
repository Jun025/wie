## [2026-11-25] PR 의 빈 통과 CI 다리를 ubuntu 러너에서 돌린다 — macOS 러너 미배정이 머지를 막지 않게 (wie-ci-macos-stable-leg-runner-unacquired-stalls-pr-landing)

**무엇을** — `rust.yml` 의 `runs-on` 을 `${{ (schedule || workflow_dispatch) && matrix.os || 'ubuntu-latest' }}` 로 바꿨다. PR · main push 에서 `FULL` 이 거짓인 다섯 다리(모든 스텝을 건너뛰는 다리)는 이제 ubuntu 러너를 받는다. 잡 이름은 `runs-on` 이 아니라 매트릭스 값에서 나오므로 required 이름 `rust_ci (macos-latest, stable)` · `rust_ci (windows-latest, stable)` 은 그대로 보고된다. 주간 schedule 과 손 dispatch 는 종전대로 실제 OS 에서 여섯 다리를 다 돈다.

**왜** — 11-23 부터 GitHub 호스티드 macOS arm64 러너가 모자라 macOS 다리가 러너를 못 받고 15분 뒤 취소됐다. 그런데 PR 의 macOS 다리는 #586(2026-11-07) 이후 스텝을 하나도 돌리지 않는다. 러너를 쓰지도 않는 다리가 러너를 기다리다 머지를 막은 셈이다. 머지 티켓 1건(#622)이 CI_PARK 에 걸렸고 총괄 재실행 3회도 듣지 않았다.

**사용자 영향** — 없다(CI 배선만 바꿨다). PR 착지가 macOS 공급 상태와 무관해진다.

### 1. 원인 측정(읽기 조회만)

- **계정 축 — 기각.** 저장소가 public 이라 Actions 분은 무료다. `gh api /users/Jun025/settings/billing/usage?year=2026&month=11` 의 wie 행은 `Actions macOS 3-core min=288 net=0` · `Actions Linux min=2671 net=0` · `Actions Windows min=352 net=0` 이다(과금 0). `settings/billing/actions` 는 410(옮겨짐)이고 `settings/billing/budgets` 는 404 다(지출 한도 조회 불가).
- **설정 축 — 기각.** `rust.yml` 에는 잡 수준 `if:` · `timeout-minutes` 가 없다. 취소 사유는 timeout · concurrency 가 아니라 아래 annotation 이다.
- **공급 축 — 채택.** Rust CI 최근 60 run 의 macOS 잡(모든 attempt 포함) 164건 중 **49건이 `cancelled` · steps 0 · runner 공란**이다. 날짜별로 11-22 는 0/16, 11-23 은 42/96, 11-24 는 7/48 이다. 취소된 잡의 annotation 은 `The job was not acquired by Runner of type hosted even after multiple attempts` 와 `Due to capacity constraints, jobs targeting macOS arm64 runners may experience longer queue times.` 다(run 45430319889). 취소는 생성 후 900~903초에 일정하게 난다. 10-21 에 측정한 형상(`~/orchestrator/reports/wie-ci-macos-runner-starvation-measure-and-propose.done.md`)과 같다.
- **«왜 beta 는 받고 stable 은 못 받나» — 그런 축은 없다.** 49건 중 stable 과 beta 가 고르게 섞여 있다. main push run 45430319889 는 둘 다 취소됐고, #622 의 attempt 1 에서는 beta 도 10분 늦게 배정됐다. 같은 쪽이 반복되는 것은 `rerun --failed` 가 실패한 다리만 다시 돌리기 때문이다(10-21 측정과 같은 결론). stable 만 머지를 막는 이유는 ruleset «main protection» 의 required 목록에 stable 만 있기 때문이다(`gh api repos/Jun025/wie/rules/branches/main`).

### 2. 처방을 고른 이유

티켓의 «공급 축이면 코드 변경 없이 근거만» 은 일시 장애를 전제한 처방이다. 이번 공급 부족은 이틀째 이어지고, 재실행 3회가 모두 같은 결론이었다. 그리고 PR 다리는 아무것도 재지 않으므로 macOS 러너를 요구할 이유가 없다. ruleset 을 고치는 길(10-21 의 P3)은 사람 손이 필요하고, 매트릭스에서 다리를 빼는 길은 required 이름이 보고되지 않아 교착이 난다(사건 대장). `runs-on` 만 바꾸면 둘 다 피한다.

대가는 없다. 그 다섯 다리는 종전에도 스텝 0개였다. 플랫폼 검증은 지금처럼 주간 run 과 손 dispatch 가 맡는다. 덤으로 PR 마다 Windows 러너 2대 · macOS 러너 2대를 받지 않게 된다.

`runs-on` 의 이벤트 목록(`schedule` · `workflow_dispatch`)은 `FULL` 의 이벤트 목록과 같아야 한다. 둘이 어긋나면 FULL 다리가 남의 OS 에서 돈다. 그 사실은 `runs-on` 위 주석에 적었다.

### 3. 검증

- 게이트 4종과 beta clippy 를 로컬에서 돌렸다. 모두 green(`cargo test --all` FAILED 0)이다. `dod_ci_parity` 와 `check-parity-lock-wired.mjs` · `check-linux-system-deps.mjs` 도 OK 다.
- 이 PR 의 CI 결과는 PR 본문과 회신에 인용한다.
