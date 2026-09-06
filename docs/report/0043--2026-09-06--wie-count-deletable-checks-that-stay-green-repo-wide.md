## [2026-09-06] 「감시를 지웠는데 green」을 저장소 전체에서 세었다 — 「다섯」은 출처가 없고 실측은 8+3 이다 (wie-count-deletable-checks-that-stay-green-repo-wide)
- **무엇을**: ★**세기만 했다 — 가드 0 · 코드 0 · 워크플로 무접촉.** 산출물은 `docs/worklog/2026-09-06-deletable-checks-census.json` 과 이월된 「다섯 번」 3자리의 인라인 정정뿐이다.
- **왜**: 운영자 채택 제안 `2026-09-06-parity-lock-self-deletion-guard#p0`.
- **★⑴ 술어와 수**: 「워크플로 스텝이 «경로로» 부르는 검사 파일 집합 **A**」 ↔ 「`scripts/`·`*/tests/` 에 실재하는 파일 집합 **B**」의 차집합. ★**A=9 · B=23 · B\A=14.** 재현은 `git grep` 두 줄이고 워크로그 `sets.reproduce` 에 그대로 실었다.
- **★★⑵ 14를 전건 분류했다 — 「지워도 green」은 «한 형태»가 아니라 «두 형태»였다**:
  ⒜★**「CI 에서 돌고 있는데 지워도 green」 8건** — 전건 rust 통합시험. `cargo test --all`·`tarpaulin --workspace` 가 **glob 으로 줍고** 워크플로가 이름을 부르는 자리가 **0** 이다.
  ⒝★**「애초에 CI 에서 안 도는 검사」 3건** — `audit-no-leak.sh`·`verify-browser.mjs`·`smoke_gate.sh`. ★**지울 필요도 없다. 이미 안 돈다.** 특히 `audit-no-leak.sh` 는 Constraint 9·10 의 «기계 절반»인데 ★**어느 워크플로도 npm 스크립트도 부르지 않는다**(전수 0건).
  ⒞검사 아님 2건(`lgt_render_probe.sh` 측정 하네스 · `smoke_gate_baseline.tsv` 데이터) ⒟★**술어의 오탐 1건** = `wie_cli/tests/support/dod_ci_parity.rs` — 직전 회차가 만든 `check-parity-lock-wired.mjs`(집합 A 원소)가 이 경로를 물어 red 가 된다 ⇒ ★**그 가드가 여기서 작동을 증명했다.**
- **★★⑶ 분류를 «주장»이 아니라 «실행»으로 냈다**: 격리 워크트리에서 ⒜의 하나(`wie_jvm_support/tests/absent_timer_schedule.rs`)를 **실제로 지우고** `RUST_MIN_STACK=4194304 cargo test --all` → ★**기준선 rc=0 · 40줄 · 156 passed · 이름 1회** ↔ ★**삭제 후 rc=0 · 39줄 · 155 passed · 이름 «0회»**. ★**커밋 0 · 워크트리 제거.** 같은 트리에서 `cargo fmt` rc=0 · 두 node 가드 rc=0.
  ★**커버리지 게이트도 못 잡는다**: `codecov.yml` 이 **0바이트**(Constraint 2 가 «일부러 비워 둔다»고 적은 그것)라 임계가 없다 — `fail_ci_if_error: true` 는 업로드 오류용이다.
- **★★⑷ 회차를 낳은 수가 «출처 없음»이었다**: 「이 저장소에서 다섯 번 났다」는 `2026-09-05-dod-ci-parity-checker.json` 에서 처음 나와 두 곳으로 인용됐을 뿐 ★**다섯 자리를 열거한 곳이 없다.** 저장소가 실제로 센 「five times」는 `AGENTS.md:234,255` 의 **셀프머지 5건**(다른 형태)이다 ⇒ ★**다른 대장 항목에서 빌려 온 수로 보인다.** 세 자리에 상호참조 정정을 붙였고 **원문은 사료로 보존**했다.
- **사용자 영향**: 없음(조사·문서). 대신 총괄이 발권 계획을 세울 «수»가 생겼다.
- **★남는 구멍**: ⒜★**술어가 «파일»을 보지 «모듈»을 안 본다** — 인라인 `#[cfg(test)]` 를 가진 `.rs` 가 **31개** 더 있어 진짜 모집단은 8보다 크다(다만 `mod tests` 삭제는 소스 diff 라 더 눈에 띈다) ⒝스텝«까지» 지우면 A 원소도 사라진다(무한 후퇴 — 직전 회차가 이미 적었다) ⒞`web/`·`functions/` 는 술어 밖이다(현재 시험 파일 **0건** ⇒ 손실 없음).
- **★권하지 않는 것**: ⒜의 8건에 **각각 가드를 다는 것**. 직전 회차의 가드가 정당했던 이유는 락이 **2파일 구성**이고 **CI 자신을 감시**하기 때문이며, 보통의 시험 1건 삭제는 **PR diff 에 그대로 보인다**. ⇒ ★**값하는 자리는 ⒝의 3건**이다.

