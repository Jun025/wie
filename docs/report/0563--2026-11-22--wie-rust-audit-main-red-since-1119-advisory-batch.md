## [2026-11-22] Security audit red — 자문 16건(억제 뒤 13건) + 경고 5건 처분: 4 크레이트 상향 · zip 5건 이름 억제 (wie-rust-audit-main-red-since-1119-advisory-batch)

**무엇을**: `rust-audit.yaml` 최신 run 44549764012(11-19 · `2e66f9d6`)이 `13 vulnerabilities found` · `5 allowed warnings` 로 failure 였다. 그 13 은 기존 억제 3건을 뺀 수이고, 무-플래그 `cargo audit`(cargo-audit 0.22.2 · advisory-db 1,555건 `55e1c3df` · `Cargo.lock` 557 패키지)은 **취약점 16건 · 경고 5건**을 낸다. 전수 처분했다.

**actix-http·actix-web 이 어디서 오나**: 워크스페이스 루트 패키지 `wie`(= 네이티브 데스크톱 호스트 `wie_cli`)가 쓰는 `sentry 0.49.2` 의 **선택 의존** `sentry-actix` 를 통해서다. 그 feature 를 켜지 않으므로 ★**lockfile 에만 있고 컴파일되지 않는다**(`cargo tree -i actix-http -e all --target all` → `nothing to print`). 배포되는 WASM(`wie_featurephone`)에도 개발 도구에도 없다 — 우리는 HTTP 서버를 띄우지 않는다. `quinn-udp`·`tokio` 도 같은 형상(`sentry → reqwest` 계열의 선택 의존)이다.

| ID | 크레이트 | 현재 → 수정 판 | 우리 도달 경로 | 처분 |
|---|---|---|---|---|
| RUSTSEC-2026-0446 · 0448 · 0455 · 0552 · 0553 · 0554 | actix-http | 3.13.5 → **3.22.0**(하한 최고 `>=3.21.3`) | 빌드 그래프 0(lockfile 에만) | `cargo update -p` |
| RUSTSEC-2026-0447 | actix-web | 4.15.0 → **4.20.0**(`>=4.18.2`) | 빌드 그래프 0 | `cargo update -p` |
| RUSTSEC-2026-0538 | quinn-udp | 0.5.15 → **0.5.16**(`>=0.5.16, <0.6.0`) | 빌드 그래프 0 | `cargo update -p` |
| RUSTSEC-2026-0429 · 0546 · 0548 | zip | 8.6.0 → (9.0.0 · 9.0.5 에만 패치) | ★없음 — `extract`·`ZipStreamReader`·`read_zipfile_from_stream` 호출 0(우리·rustjava-runtime 둘 다 메모리 읽기만) | ★이름 억제 ×3(§D D-2) |
| RUSTSEC-2026-0545 · 0547 | zip | 8.6.0 → (9.0.0 에만 패치) | ★없음 — 결함 코드가 `aes-crypto`·`legacy-zip` feature 뒤에 있고 둘 다 꺼져 있어 컴파일되지 않는다 | ★이름 억제 ×2(§D D-3) |
| RUSTSEC-2026-0367 · 0368 · 0369 | zip | (10-23 처분분) | §D D-1 그대로 — 이번에 호출 0건을 재측했다 | 기존 억제 유지 |
| (경고·unsound) RUSTSEC-2026-0486 | rustix | 1.1.4 → **1.1.6** | 리눅스 데스크톱 호스트만(winit 백엔드) · WASM 0 · 우리 xattr 호출 0 | `cargo update -p` |
| (경고·unsound) RUSTSEC-2026-0386 | tokio | 1.53.1 → **1.54.1**(`>=1.53.3`) | 빌드 그래프 0 | `cargo update -p` |
| (경고·yanked) | chacha20 | 0.10.1 → **0.10.2** | 네이티브만(rodio → rand) · 기존 A-4 판정 «도달 불가» | `cargo update -p` → A-4 를 C-10 으로 |
| (경고·unmaintained) RUSTSEC-2026-0543 | chrono | 0.4.45 · `patched = []` | ★실린다 — `wie-lgt`(직접) · `rustjava-runtime`. 결함 경로가 아니라 «유지보수 종료» 신호 | 수용 — 기대 목록 + §A **A-5** 등재 |
| (경고·unmaintained) RUSTSEC-2026-0192 | ttf-parser | 0.25.1 | 기존 A-1 | 변경 없음 |

**왜 zip 은 여전히 억제인가**: 10-23 과 같은 이유다. 다섯 건 모두 패치가 zip 9 에만 있고 `rustjava-runtime` 은 0.3.2 까지 `zip ^8.6`, `^9.0.5` 는 0.3.3 ~ 0.4.2 다(crates.io 인덱스 2026-11-22 재측). RustJava 핀 이동(Constraint 8 · 설계 사안)은 별 회차의 몫이다. 억제는 ID 로만 했고(크레이트·일괄 억제 0) 논증과 해제 조건은 `docs/project-kb/02_status.md` §D 의 D-2·D-3 행에 있다. D-3 의 전제는 판본이 아니라 **feature 조합**이다 — 해소된 zip feature 는 `cargo metadata` 실측으로 deflate 계열 다섯뿐이고, 어디서든 `aes-crypto`·`legacy-zip` 을 켜면 그 행은 무효다.

**★`cargo update -p` 가 무관한 `windows-sys` 간선 7개를 낮췄다 — 되돌렸다**: cargo 1.99.0 에서 sentry 계열 아무 패키지나 `cargo update -p` 하면(같은 판으로의 무변경 `--precise` 포함) `dirs-sys`·`errno`·`nu-ansi-term`·`quinn-udp`·`rustix`·`rustls-platform-verifier`·`winapi-util` 의 `windows-sys 0.61.2` 간선이 lockfile 에 이미 있던 `0.52.0`/`0.59.0` 으로 내려간다. 자문과 무관하고, PR 이 재지 않는 Windows 빌드만 바꾸는 변경이라 일곱 간선을 원래 값으로 되돌렸다. 되돌린 lock 은 `cargo metadata --locked` rc=0 으로 유효함을 확인했다(cargo 가 다시 쓰지 않는다). 다음에 이 저장소에서 `cargo update -p` 를 돌리는 회차는 같은 것을 만난다.

**사용자 영향**: 없다. 배포 WASM 에 실리는 크레이트 가운데 판본이 바뀐 것은 0이다(zip·chrono 는 그대로). 네이티브에서 실제로 컴파일되는 변경은 `chacha20`(macOS·Windows·Linux)과 `rustix`(Linux)뿐이다.

**10-23 처분 뒤 일일 실행은 «하루도» green 이 아니었고, 11-05 부터는 아예 돌지 않는다**(run 로그 직독):
- 10-24 ~ 11-02(10일): 취약점 **0건**인데 failure 다. 원인은 audit 단계가 아니라 둘째 단계의 LEDGER-A 절 — 새 **경고** `tokio@1.53.1 RUSTSEC-2026-0386` 가 «§A 에 행이 없다» 로 red 를 냈다. 기대 목록 절은 새 경고를 «red 가 아니다» 로 다루지만 §A 집합 대조는 양방향 red 라(둘 다 `rust-audit.yaml` 주석에 적힌 설계다), 새 정보성 자문 하나가 일일 잡을 붉힌다. 이번 처분 뒤에도 같다 — 다음 새 경고가 뜨는 날 다시 붉어진다.
- 11-03 · 11-04: `1 vulnerability found`(zip RUSTSEC-2026-0429 도착).
- **11-04T05:40Z 뒤로 schedule 이벤트는 이 저장소 전 워크플로에서 0건**이다(주간 `doc-liveness` 도 10-31 이 마지막). 11-19 의 두 run 은 `workflow_dispatch` 다. 워크플로 상태는 전부 `active`, Actions 는 켜져 있고(`enabled: true`), 원인은 재지 못했다. 그래서 이 PR 이 착지해도 일일 run 이 스스로 green 을 보여 주지 않는다 — 착지 뒤 확인은 `gh workflow run rust-audit.yaml` 손 실행이어야 한다.

**실측**:
- `Cargo.lock` **41줄**(21+/20−): 6 패키지 × 판본+체크섬 24줄 · `rustix 1.1.4 → 1.1.6` 을 가리키는 간선 8쌍 16줄 · actix-web 4.20.0 의 새 간선 `tokio` 1줄. 패키지 수 557 → 557 · `Cargo.toml` 손대지 않음.
- 워크플로와 같은 명령(`cargo audit --ignore` ×8) **rc=0**(`2 allowed warnings found`) · 무-플래그 rc=1(zip 8건 — 의도된 것) · `node scripts/check-audit-warnings.mjs` rc=0(기대 목록 · §A · IGNORES 세 절) · 음성 대조: `pr-audit.yml` 에서 0548 하나를 빼면 rc=1, 원복 뒤 rc=0 · `check-ledger-c-validity-scope.mjs` OK(C-10 ~ C-15 추가 · C-4 와 C-5 사이 빈 줄을 지워 끊긴 표를 이었다).
- 4 게이트(`cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm32 clippy · `cargo +beta clippy --all -- -D warnings` 1.101.0-beta.2) 전부 rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — 54 스위트 838 통과 0 실패 · 빌드 뒤 `Cargo.lock` 재작성 0.
- 러너 블록 6줄 전부 PASS rc=0(draw_j2me · helloworld_ktf · helloworld_lgt · keydraw_ktf `--inject --expect-last-frame` paints 79 · keydraw_lgt 같음 paints 55 · text_j2me `--timeout 5`) ⇒ 회귀 0 · `npm run audit` PASS · `check-doc-liveness-parity` OK.
- 로컬 stable 은 1.99.0 이고 CI stable 은 그보다 새 판이다 — stable 축은 PR 의 `rust_ci (ubuntu-latest, stable)` 가, 그 위 lint 는 위 beta 줄이 덮는다. PR 의 macOS·Windows 다리는 빈 통과라 `rustix`(Linux 전용)·`chacha20` 의 Windows 빌드는 이 PR 의 green 이 덮지 않는다.
