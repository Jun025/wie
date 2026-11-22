## [2026-11-22] 일일 감사 — 대장에 없는 새 «경고»는 red 가 아니라 잡 요약 한 줄 · C-6 의 없는 패키지 이름 정정 (wie-rust-audit-new-warning-reds-ledger-a-and-c6-package-name)

**무엇을**: `scripts/check-audit-warnings.mjs` 의 LEDGER-A 절(대장 §A ↔ `cargo audit` 경고 집합)을 양방향 red 에서 **한 방향 red** 로 갈랐다.

| 형상 | 종전 | 지금 |
|---|---|---|
| 경고는 있는데 §A 에 행이 없다(새 unsound·yanked·unmaintained) | red | **rc 0** + stdout 과 잡 요약(`GITHUB_STEP_SUMMARY`)에 «대장에 없는 새 경고 N건: <목록>» |
| §A 에 행은 있는데 경고가 없다(낡은 행) | red | red(그대로) |
| 절 머리의 건수 ≠ 행 수 · 표를 못 읽음 | red | red(그대로) |
| 억제되지 않은 취약점(새 취약점) | 앞 단계 `cargo audit` 만 red | 앞 단계 red(그대로) + **이 검사기도 red** |

**왜**: 새 정보성 자문은 이 repo 의 변경 없이 뜬다. 0563 이 run 로그로 확인한 대로 10-24 ~ 11-02 의 10일 red 는 취약점 0 인 채 새 경고 1건(`tokio@1.53.1` RUSTSEC-2026-0386)이 «§A 에 행이 없다» 로 낸 것이었고, #596 착지 뒤에도 다음 새 경고가 같은 red 를 만들 설계였다. 같은 스크립트의 첫 절(기대 목록)은 처음부터 «새 것은 red 가 아니다» 였으므로 두 절이 서로 다른 답을 하고 있었다. 총괄 결정: 새 경고는 «보이게» 하되 붉히지 않는다 · 새 취약점은 완화하지 않는다.

**새 취약점 줄을 검사기에도 둔 이유**: 워크플로에서는 `cargo audit` 단계가 먼저 붉어 이 줄이 발화할 일이 없다. 그래도 넣은 것은, 경고를 풀어 준 검사기의 판정이 «앞 단계가 있다» 는 사실에 기대지 않게 하고 자가시험이 «취약점 = red» 를 묶을 수 있게 하려는 것이다(억제 목록은 이미 그 절이 읽고 있어 4줄이다).

**자가시험**: `node scripts/check-audit-warnings.mjs --selftest` — 임시 디렉터리에 최소 대장·기대 목록·두 워크플로를 쓰고 합성 `cargo audit --json` 을 물려 스크립트 자신을 CLI 로 돌린다(cargo-audit·네트워크 불요). 4케이스: 기준 rc 0 · ⒜새 경고 rc 0 + 요약 줄 · ⒝새 취약점 rc 1 · ⒞낡은 행 rc 1(기대 목록도 비워 red 가 LEDGER-A 에서 나는지를 가른다). `rust-audit.yaml` 의 검사 단계가 본 검사 앞에 이것을 돌린다. PR 단계에는 물리지 않았다 — 그 워크플로는 schedule·dispatch 전용이고, `contract` 에 올리는 것은 이 티켓의 범위가 아니다.

**C-6**: 인용 명령의 `-p wie-featurephone` 을 실명 `wie_featurephone` 으로 고쳤다. ★고치고 돌려 보니 출력 꼴이 대장 서술과 **달랐다** — 서술도 함께 고쳤다.
- 실명: `cargo tree -p wie_featurephone --target wasm32-unknown-unknown -i rustls` → **rc=101** `error: package ID specification 'rustls' did not match any packages`(`--target all` 도 같다). `wie_featurephone` 의 해소 그래프에 `rustls` 가 아예 없다는 뜻이라 «WASM 아티팩트에 없다» 는 결론은 그대로이고 오히려 더 강하다.
- 없는 이름: 같은 명령이 rc=0 `nothing to print` 를 낸다. cargo 1.99.0 은 맞는 멤버가 없는 `-p` 를 오류로 내지 않고 **다른(더 넓은) 범위**를 잰다 — 양성 대조로 `-i zip` 을 넣으면 없는 이름 쪽 트리에 `test-utils`(이 패키지의 의존이 아니다)가 4줄 나오고 실명 쪽은 0줄이다. 종전의 `nothing to print` 는 이 패키지에 대한 답이 아니었다.

**문서**: 대장 머리(`docs/project-kb/02_status.md` «공급망 추적 대장»)에 2026-11-22 줄을 더해 09-08 의 «양방향 red» 가 어느 날까지의 동작이었는지와 지금 red 인 넷을 적었다. `rust-audit.yaml` 의 (2) 주석도 같은 내용으로 고쳤다.

**사용자 영향**: 없다. 엔진·웹·`Cargo.*` 변경 0 · 억제 목록(`--ignore` 8건)과 기대 목록 변경 0.

**실측**(cargo-audit 0.22.2 · advisory-db 1,555건 · lock 557 패키지 · cargo 1.99.0):
- 워크플로 파일에서 `run:` 을 추출해 그대로 실행 — `cargo audit --ignore` ×8 → `2 allowed warnings found` · `--selftest` 4/4 · 본 검사 세 절 OK ⇒ **rc=0**. `actionlint` rc=0.
- 실 대장 + 실 `cargo audit --json` 에 1건씩 주입(스크래치):
  - ⒜ `tokio@1.53.1` RUSTSEC-2026-0386(10-27 run 의 형상 그대로) → **rc=0** · 잡 요약 파일 = `대장에 없는 새 경고 1건: tokio@1.53.1 RUSTSEC-2026-0386`.
  - ⒝ 취약점 `RUSTSEC-2099-0001` → **rc=1** `억제되지 않은 취약점 1건`.
  - ⒞ 경고에서 `chrono` 를 뺌 → **rc=1** `§A 에만 있다(경고가 없다 = 해소됐다): A-5 chrono@0.4.45 RUSTSEC-2026-0543`.
- 개악 대조(스크립트의 스크래치 사본 4종에 `--selftest`): 새 경고를 다시 red 로 → ⒜ FAIL(rc=1) · 취약점 red 삭제 → ⒝ FAIL · 낡은 행 red 삭제 → ⒞ FAIL · 요약 줄 삭제 → ⒜ FAIL. 넷 다 rc=1 이고 나머지 케이스는 PASS.
- `check-ledger-c-validity-scope.mjs` rc=0(C 행 15건 · C-6 6칸 유지).
- 게이트: `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm32 clippy · `cargo +beta clippy --all -- -D warnings`(1.101.0-beta.2) 전부 rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — 54 스위트 856 통과 0 실패. PR 의 beta 다리는 빈 통과라 beta 는 이 로컬 줄이 유일한 망이다.
- `npm run audit` PASS · `check-docs-report-serial` OK · `check-doc-liveness-parity` OK. `check-engine-contract.mjs` 는 로컬 rc=1(`frames_painted() missing`) — git-ignore 된 `web/src/wasm/` 가 10-22 빌드라 낡은 것이고 이 변경과 무관하다(Rust·export 표면 무접촉). 그 축은 PR 의 `contract` 가 새로 빌드해 잰다. 러너 블록은 돌리지 않았다(엔진 코드 무접촉).

**남긴 것**:
- 대장에 없는 새 경고는 이제 누가 잡 요약을 읽어야 보인다. 등재하지 않은 경고가 나중에 해소되면 아무 신호 없이 사라진다 — 총괄 결정의 값이다.
- `rust-audit.yaml` 은 PR 에서 돌지 않으므로 이 PR 의 CI green 은 고친 워크플로 단계를 재지 않는다. 위 로컬 실행이 그 대신이고, 착지 뒤 확인은 일일 실행이나 손 실행이다(schedule 이벤트가 11-04 뒤 멈춰 있던 것은 0563 에 있다).
