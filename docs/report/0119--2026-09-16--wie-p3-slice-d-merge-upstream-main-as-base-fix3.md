## [2026-09-16] 착지하면 `main` 이 빨개진다 — 안 쓰는 데스크톱 크레이트를 workspace 에서 뺐다 (wie-p3-slice-d-merge-upstream-main-as-base-fix3)

**무엇을** — base swap 이 들여온 upstream 의 Tauri 데스크톱 셸 `wie-app` 을 workspace `members` 에서
빼고 `exclude` 로 **선언**했다. ★**그것이 GTK/WebKit2GTK/libsoup3 의 «유일한» 뿌리**였고, 우리 Linux CI 는
그 라이브러리를 깔지 않는다. ⇒ `rust_ci (ubuntu × stable·beta)` 와 `coverage` **3검사가 fail** 이었다.

**왜** — ★**「구조적 red」가 아니라 «회귀 red»다.** swap **전** head `7dd70da5` 는 Rust CI·coverage
**success** 였고 `main` 최근 5회도 **전건 success** 다. ⇒ ★**이 PR 이 착지하면 `main` 의 push CI 둘이
즉시 빨개진다.** ★**로컬 green 이 이것을 못 잡는다** — 직전 회차의 `384 passed`(검수자도 재현했다)는
**참이면서 이 red 를 보지 못하는 자**다.

**사용자 영향** — 없다(제품 코드 **0줄**). 지키는 것은 **CI 의 신뢰성**이다: `main` 이 상시 red 면
다음 회차부터 red 가 «배경»이 되고, 그것이 이 저장소가 반복해 규탄한 «조용한 실패»의 입구다.

### 처방은 ⒜ — 근거를 수로 적는다 (「둘 다」는 답이 아니다)

검수자가 두 갈래를 열었다: ⒜`wie-app` 을 members 에서 뺀다 · ⒝ubuntu 레그에 GTK/WebKit/soup3 를 깐다.
★**⒜를 골랐다.** 근거 넷, 전부 실측이다:

| # | 근거 | 측정 |
|---|---|---|
| ⑴ | ★**채택한 적이 없다** | `wie-app`·`wie_app` 참조 전수 — 우리 것은 **`Cargo.toml` members 한 줄뿐**. 나머지는 upstream 의 `release.yaml`+`scripts/release/*.sh`(★**이미 주차됨**) · 우리 «분석» 문서(`upstream-realign-verdict.md` 등) · ★**오탐 1건**(`wie-web/src/ts/app_library_store.ts:14` = `indexedDB.open("wie_app_library")` — 크레이트가 아니다) |
| ⑵ | ★**비대칭이 이미 절반 해소돼 있었다** | 직전 회차가 `release.yaml` 을 `workflow_dispatch` 로 주차했다 = ★**그 크레이트를 빌드하는 «유일한» 파이프라인**. ⇒ 오늘 상태는 「파이프라인은 주차하고 크레이트는 싣는다」이고 ⒜가 그것을 **정합**으로 만든다. ★**⒝는 반대로 «주차한 앱을 빌드하려고 돈을 쓰는» 형태**가 된다 |
| ⑶ | ⒝의 비용이 **영구·반복**이다 | upstream 자기 `release.yaml` 의 linux job 이 요구하는 목록이 **5개**(`libasound2-dev libayatana-appindicator3-dev librsvg2-dev libwebkit2gtk-4.1-dev patchelf`) × **3레인**(rust_ci ubuntu 2 + coverage 컨테이너). 그리고 ★**다음 upstream 당김이 또 호스트를 들여오면 같은 일이 반복**된다(그 파일에 이미 Android·iOS 잡이 있다) |
| ⑷ | 저장소 관용 | `AGENTS.md` **Constraint 7** = `wie_featurephone` 은 wasm32 밖에서 **빈 라이브러리**다. ★**안 쓰는 호스트를 빌드 밖에 두는 것이 이 저장소의 기존 형태**다 |

★**`exclude` 로 «선언»한 이유**: members 에서만 빼면 cargo 자신이
「to keep it out of the workspace, add the package to the `workspace.exclude` array」라고 **말한다**(실측).

### 판정식 — Linux 타깃에서 뿌리가 사라졌는가

```
cargo tree --workspace --all-features -i <crate> --target x86_64-unknown-linux-gnu
```
(★`--all-features`·`--workspace` 는 **`coverage.yml` 의 실제 invocation**이다 — `cargo tarpaulin
--all-features --skip-clean --workspace`)

| 크레이트 | 조치 전 | ★조치 후 |
|---|---|---|
| `glib-sys` · `soup3-sys` · `webkit2gtk-sys` · `gtk-sys` · `tauri` | **PRESENT** | ★**전건 «did not match any packages»** |

★**뿌리는 하나였다**(조치 전 역의존 트리 실측):
`glib-sys ← atk-sys ← atk ← gtk ← muda ← tauri 2.11.5 ← wie-app` — ★**workspace 안의 유일한 소비자**.
※검수자 기재 `tauri 2.10.2` 는 `wie-app/Cargo.toml` 의 **요구 범위**이고, `Cargo.lock` 해소값은 **2.11.5** 다.

### 실패의 정체 — 검수자 인용을 내가 다시 쟀다

```
rust_ci (ubuntu-latest, stable·beta) · cargo clippy --all -- -D warnings
  error: failed to run custom build command for `glib-sys v0.18.1`
  pkg-config exited with status code 1
    Package glib-2.0 was not found in the pkg-config search path.
  The system library `glib-2.0` required by crate `glib-sys` was not found.
coverage · cargo tarpaulin --all-features --workspace
  soup3-sys — Package 'libsoup-3.0', required by 'virtual:world', not found
```
★**우리가 깔는 것**: `rust.yml:96` = **`libasound2-dev` 하나** · `coverage.yml:48` =
`libgtk-3-dev libasound2-dev`(★gtk3 는 **libsoup2** 라 soup3 를 못 준다).
★**두 파일 다 이 PR 이 건드리지 않았다** — `git diff origin/main` **0줄**.
★**`--all`/`--workspace` 가 결정적이다**: `rust.yml:103` `cargo clippy --all` ·
`coverage.yml:53` `--workspace` ⇒ **members 에 있으면 반드시 빌드된다**(`default-members = ["."]` 는 이 둘을 못 막는다).

### 대조군 — 「회귀」의 증거

| head | Rust CI | coverage |
|---|---|---|
| `7dd70da5` (swap **전**) | ★**success** | ★**success** |
| `f533ba54` (swap 후) | — (★`CONFLICTING` 이라 `pull_request` 미발화) | **failure** |
| `d35c2902` · `8e870e95` | **failure** | **failure** |
| `main` 최근 5회 | success | success |

### 잃는 것 — 숨기지 않는다

★**`wie-app` 은 이제 «단독으로도» 빌드되지 않는다.** 실측: 조치 **전** `cd wie-app && cargo check` →
**성공(2m33s)** · 조치 **후** → `error inheriting 'edition' from workspace root manifest`.
근인은 그 크레이트가 `version/edition/license.workspace = true` 로 **루트에서 상속**하는데 `exclude` 가 그 링크를 끊는 것이다.
★**단독 빌드를 살리려면 upstream 자기 매니페스트를 리터럴로 고쳐야** 하고 그것은 **당김마다 영구 충돌**이다 ⇒ 더 싼 쪽을 택했다.
★**되돌리기는 members 한 줄**이고, 그때는 «의식적 채택»이라 오히려 옳은 시점이다.
★**`Cargo.lock` 이 2,281줄 줄었다**(80 삽입 · 2,361 삭제) — tauri 트리가 빠진 몫이고 **의도된 산출**이다.
★**형제 충돌 위험**: `Cargo.toml` 을 만지지만 ★**#162 는 문서 4파일뿐**(실측)이라 충돌 0.

### ★CI 상태를 정확히 승계한다 — 직전 회차 §7 이 «coverage» 를 빠뜨렸다

직전 회차 §7 은 「직전 head 에서 **6건**(그중 `web_ci` fail)이었고, 주차 후 **5건**이다」로 적었다.
★**`f533ba54` 의 `coverage` 도 fail 이었고**(run `35049681954`), ★**그 직전 게이트② 회신이 그것을
`rc=1 CI_RED … failure coverage` 로 이미 적었다.** ⇒ ★**`web_ci` 만 세고 `coverage` 를 빠뜨린 결과
「red 는 주차로 해소됐다」로 읽힌다 — 그것이 거짓이다.** 주차가 없앤 것은 `web_ci` **하나**이고
`coverage`·`rust_ci` 는 **남아 있었다**.
★**그리고 직전 회차의 `rc=4 PENDING` 은 «그 시점에» 참이었다** — `rust_ci` 가 **04:52:35 UTC** 에 죽었고
그 관측은 **04:52:23 UTC** 였다 ⇒ ★**12초 차이로 못 봤다.** 「적고 넘어간다」는 그때 옳았고,
★**지금은 판정이 바뀌었다** — 그 차이를 흐리지 않는다.

### ★정정 — 「`contract` 는 `main` 의 required status check」는 실측과 어긋난다

```
gh api repos/Jun025/wie/branches/main/protection → 404 "Branch not protected"
gh api repos/Jun025/wie/rulesets               → []
gh pr view 161 --json mergeStateStatus         → UNSTABLE   (BLOCKED 이 아니다)
```
⇒ ★**GitHub 강제 required check 는 «0개»** 이고 `contract` 가 red 여도 머지 버튼은 열린다.
직전 회차는 **문서를 인용했다**(`engine-contract.yml:22` 헤더) — ★**문서와 실측이 어긋날 때는 실측이 이긴다.**
그 헤더가 적은 것은 «설계 의도»이고 브랜치 보호의 «현재 상태»가 아니다.
★**행동은 그대로 옳았고 근거만 갈아치운다**: 크래시한 계약 검사기는 ★**fail-closed 계약을 «아무것도
검증하지 않는다»**(Constraint 3) ⇒ **강제 여부와 무관하게** 결함이다. `docs/report/0118` 에도 그 정정을 얹었다.

### 게이트

```
cargo fmt --all -- --check                                      rc=0
cargo clippy --all -- -D warnings                               rc=0
cargo clippy --target wasm32-unknown-unknown -- -D warnings     rc=0
cargo +beta clippy --all -- -D warnings                         rc=0
RUST_MIN_STACK=4194304 cargo test --all                         rc=0   384 passed · 0 failed (불변)
npm run build:wasm                                              rc=0
node scripts/check-engine-contract.mjs                          rc=0   107 pass · 0 violation
npm run audit / check-worklog-json / check-worklog-coverage / check-docs-report-serial(--selftest·bare)
  / check-doc-liveness-parity / check-parity-lock-wired / check-engine-runner-fixtures   전건 rc=0
5픽스처: draw_j2me PASS(1/content) · helloworld_ktf PASS(0) · helloworld_lgt PASS(0)
         keydraw_ktf PASS(55·content·rc0) · keydraw_lgt PASS(55·content·rc0)
```
★`cargo test --all` **384** 는 조치 전후 **불변**이다(`wie-app` 은 시험을 갖지 않는다) ⇒
★**이 조치가 커버리지를 «줄여서» green 을 산 것이 아니다.**

### 개악 대조 — 양방향

| 축 | 개악(`wie-app` 을 members 로 되돌림) | 정상 |
|---|---|---|
| `glib-sys` · `soup3-sys` · `webkit2gtk-sys` (linux · `--all-features`) | ★**전건 PRESENT** | ★**전건 GONE** |

복원 후 `Cargo.toml` **바이트 동일**(`exclude` 줄 1) · `HEAD` **불변**.
★**macOS·Windows 4레인은 «깨뜨릴 것이 없다»** — 이 조치는 **의존을 빼는 방향**이고,
실측으로 그 호스트는 조치 «전»에도 `wie-app` 을 빌드했다(위 2m33s) ⇒ 순 손실이 아니다.
※그 4레인의 green 은 **CI 가 다시 잰다**(§CI).

### 한계

- ★**ubuntu·coverage 의 green 을 «로컬에서» 보일 수 없다** — 리눅스 러너가 없다. 내가 보인 것은
  ★**기전**(그 3검사가 죽던 의존이 workspace 그래프에서 사라졌다)이고 **판정은 CI 가 낸다**(§CI).
- ★**`wie-app` 단독 빌드 상실**(위 「잃는 것」) · ★**다음 upstream 당김이 `Cargo.toml` members 를 되살릴 수 있다**
  — 조각 B 가 분류한 «폐기» 계급과 같은 성질이고, 그때 이 `exclude` 줄과 그 주석이 근거로 남는다.
- ★**`npm run frontend`·`contract-roundtrip.mjs` 는 이번에도 돌리지 않았다**(직전 회차 한계 그대로 · CI 몫).
