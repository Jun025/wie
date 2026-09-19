## [2026-09-20] `cargo audit` 을 PR 단계에서도 돌릴까 — ★**가격을 쟀다. 권고 = 「채택, 단 비게이팅 + lockfile 변경 시에만」** (wie-price-cargo-audit-at-pr-stage-recommend-only)

채택 제안 `2026-09-19-security-audit-schedule-red-three-days#p1`. **산출은 판정이다** —
`.github/workflows/**` 변경 **0** · 의존성 **0** · `--ignore` 추가 **0**.

★**이 문서가 여기 있는 이유**: `AGENTS.md` §Landing paperwork 가 회차 기록을 `docs/report/NNNN--YYYY-MM-DD--<ticket-id>.md`
로 못박았고, **구현 없는 판정 회차**의 선례가 같은 자리에 있다(`0109` 「경보를 만들지 않는다」 · `0100` doc-liveness 배치 결정).

---

## ⒜ 대전제 재측 — ★**세 개 중 하나가 틀렸고, 그 하나가 결론을 바꾼다**

### ⑴ 「push CI 4종 중 `cargo audit` 0개」 → ★**5종**이고 `cargo audit` 은 **0개**가 맞다

`main` push 로 도는 워크플로를 `on:` 블록에서 전수로 걸었다 — **5종**:
`coverage.yml`(`on: [push]` · 전 브랜치) · `engine-contract.yml` · `publish-artifact.yml` · `rust.yml` · `web.yml`.
★**PR #211 착지 커밋의 실제 run 목록이 정확히 그 5개**였다(독립 확인).
제안의 「4종」은 **1 적다** — `coverage.yml` 이 `push: branches:[main]` 형식이 아니라 `on: [push]` 여서 빠진 것으로 보인다.
★**`cargo audit` 부분은 참이다**: 워크플로 10개 전수에서 `cargo audit` 은 **`rust-audit.yaml` 한 곳**뿐이고
그 트리거는 `schedule` + `workflow_dispatch` ⇒ ★**push 0 · pull_request 0.**

### ⑵ `AGENTS.md` 의 두 기각 — 축자

> **2026-09-18** — 「**Promoting the runner line itself to per-PR was priced and declined on 2026-09-18**
> (`wie-text-drawing-fixture-cheap-tier-vs-broad-tier-decision`), on the same axis as the 2026-09-07 decision below.」
> 표의 두 행: 「the block is **168 s** warm (measured, loadavg 180) × **6 legs** ≈ 17 min of runner time *per PR*,
> **on a self-hosted runner siblings queue behind**」 · 「every PR pays a native cargo build in a job that
> currently finishes in 12 s on a doc-only diff」

> **2026-09-07** — 「**And it stays that way: promoting `--expect-last-frame` into CI was decided against on 2026-09-07,
> measured rather than assumed.**」 근거는 「the detection delta is narrow, not zero … **zero incidents have ever been
> observed in it**, and buying it costs either a new `cargo` build in the node-only `contract` job or six redundant
> runs on `rust.yml`'s matrix」

### ⑶ Constraint 5 — 축자

> `| 5 | `cargo audit` with no ignores. A suppression needs a named advisory ID and a written reachability argument
> — never blanket, never `continue-on-error` | `rust-audit.yaml:39-54` |`

★그 지목 자리(`rust-audit.yaml:39-54`)의 실제 내용은 **`--ignore` 금지가 아니라 «두 계급을 종료코드로 가른다»**는 선언이다:
「Vulnerabilities (vulnerabilities.count > 0) **FAIL** the job … Informational warnings (unmaintained / yanked) are
printed but **NON-gating** — cargo-audit exits 0 on warnings unless `--deny warnings` is passed, which we
deliberately do not … the two classes are distinguished by **exit code, not squashed by `continue-on-error`**」.
★**이 사실이 아래 「`--ignore` 압력」 축의 절반을 미리 닫는다** — 새 «경고»는 빨갛게 만들지 못한다. **«취약점»만 만든다.**

### ★★⑷ 브리프가 물려준 전제 하나가 **틀렸다** — 「러너 1대가 병목」은 **wie 에 해당하지 않는다**

| 잰 것 | 값 |
|---|---|
| 워크플로 10개의 `runs-on` | ★**전건 GitHub 호스티드**(`ubuntu-latest` · `macos-latest` · `windows-latest`) · self-hosted 라벨 **0** |
| 실제 잡의 러너 | `runner_group_name` = **`GitHub Actions`**(rust-audit·rust_ci·contract·coverage 전건 실측) |
| `.github/` 이력에 `self-hosted` | ★**0건**(`git log -S'self-hosted' -- .github/` 무출력) |
| repo 공개 여부 | **`private=false`** ⇒ 표준 러너 Actions 분 **과금 0** |
| 큐 대기(run 생성 → job 시작) | 잡 **50개** · **p50 2s · p90 7s · max 38s** · ≤2s가 **62%** |

⇒ ★★**`AGENTS.md` 2026-09-18 행의 「on a self-hosted runner siblings queue behind」는 이 저장소의 CI 를 서술하지 않는다.**
그 문장이 참인 곳은 **local Mac estate**(otterpebble·orchestrator 의 self-hosted 러너)이고, **`wie` 의 CI 는 거기 있지 않다.**
★**그 기각의 «수»(168s×6=17min)는 여전히 참이다** — 틀린 것은 «희소 자원» 서술이고, 그래서 **그 기각 자체는 이 회차가 뒤집지 않는다**(아래 ⒠).

---

## ⒝ 가격 — ★**네 축을 수로**

### ⒜축 `cargo audit` 자체

| 형상 | 값 |
|---|---|
| CI(`rust-audit.yaml`)의 `cargo audit` **스텝** | ★**5 s**(n=2 · 성공/실패 런 모두 5 s) |
| 로컬 · advisory-db **증분** fetch 포함 | **6.8 / 6.4 / 5.4 s**(n=3 · `real 6.82` · CPU 는 0.83 s ⇒ **거의 전부 네트워크**) |
| 로컬 · advisory-db **콜드**(`--db <빈 경로>`) | **28.7 s**(47 MB 클론) → 같은 DB 재사용 **7.3 s** |

★**CI 값이 콜드인데 5 s 인 것은 러너 네트워크가 빠르기 때문**이다(그 워크플로는 advisory-db 를 캐시하지 않는다 —
캐시하는 것은 cargo registry·index 다). ⇒ 제안의 「수 초」는 **맞다**.

### ⒝축 그 job 에 «붙는» 비용 — ★**진짜 값**

`rust-audit.yaml` 잡의 스텝별 실측(초 · 최근 2런):

```
Set up job 1~2 · Cache cargo registry 0~1 · Cache cargo index 0~1 · checkout 1~2 · setup-node 1
★Install cargo-audit (taiki-e/install-action) 0     ← 프리빌트 바이너리 · 툴체인 빌드 없음
★cargo audit 5                                       ← 위 ⒜
expected-warning 대조(node) 2 · post 스텝 합 1~2
────────────────────────────────────────────────
잡 «전체» 벽시계: p50 17 s · max 49 s (n=30, schedule 런)
```

★★**제안이 걱정한 「cargo 툴체인 + install-action 이 붙는다」는 «측정으로 반증»된다** — `install-action` 은 **0 s**이고,
**Rust 툴체인 설치 스텝이 아예 없다**(`ubuntu-latest` 기본 Rust + `cargo audit` 은 `Cargo.lock` 만 읽고 **워크스페이스를 컴파일하지 않는다**).
⇒ ★**17 s 가 «붙는 비용을 전부 포함한» 값이다.** 2026-09-18 기각이 잰 **168 s × 6 legs ≈ 17 min** 과 **60배** 차이다.

### ⒞축 docs-only PR 이 무는 값 — 「헛돈 시간」

최근 **PR 60건** 전수(파일 목록을 API 로 받아 분류):

| 분류 | 건수 | 비율 |
|---|---|---|
| **docs-only**(`.md` 또는 `docs/` 만) | **22** | **36.7%** |
| ★**`Cargo.lock`/`Cargo.toml` 을 만짐** | ★**2** | ★**3.3%**(#217 · **#161**) |
| `.rs` 를 만짐 | 16 | 26.7% |
| `.github/` 를 만짐 | 12 | 20.0% |

PR 당 push 횟수 = **2.27회**(브랜치 44개 · `pull_request` run 100회 · 2026-09-17~19).

| 설계 | 60 PR 표본의 총비용 | 그중 docs-only 가 무는 «헛돈» |
|---|---|---|
| **무조건 PR 마다** | 60 × 2.27 × 17 s ≈ **38.6 분** | 22/60 × 그것 = **14.2 분**(36.7%) |
| **`paths` 필터(lockfile 변경 시만)** | 2 × 2.27 × 17 s ≈ **1.3 분** | ★**0 분** |
| 「항상 도는 래퍼 + 안에서 필터」(engine-contract 관용) | 60 × 2.27 × **13 s** ≈ 29.5 분 | 22/60 × 그것 ≈ **10.8 분** |

※13 s = `contract` 잡의 **최솟값 실측**(docs-only diff · n=10 중 min 13 s · p50 303 s) = `AGENTS.md` 의 「12 s on a doc-only diff」와 정합.

### ⒟축 러너 여력

위 ⒜⑷ 그대로 — **GitHub 호스티드 · 공개 repo(분 과금 0) · 큐 p50 2 s**.
그리고 ★**임계경로에 0 초를 더한다**: PR 한 바퀴의 최장 잡이 `coverage` **p50 677 s** · `rust.yml` **p50 575 s** ·
`doc-liveness` 990 s 인데, **17 s 짜리 잡은 그들과 «병렬»로 돈다** ⇒ **PR 완료 시각이 늦어지지 않는다.**

| 워크플로 | PR 한 바퀴 소요 |
|---|---|
| `coverage.yml`(push) | n=20 · p50 **677 s** · max 1354 s |
| `rust.yml` | n=20 · p50 **575 s** · max 892 s |
| `web.yml` / `engine-contract.yml` | p50 301 s / 299 s |
| ★**`rust-audit.yaml`** | n=30 · p50 ★**17 s** · max 49 s |

---

## ⒞ `--ignore` 압력 축 — ★**이것이 설계를 정한다**

「새 자문이 뜨면 무관한 PR 이 빨개진다」가 **실제로 몇 번 일어났는가**를 schedule 런 **87회**(2026-06-25 ~ 09-19)로 쟀다:

| 구간 | 일수 | red 원인 |
|---|---|---|
| 2026-06-25 .. 07-22 | **28일 연속** | ★**인프라**(fork 에서 Issues 비활성 → `rustsec/audit-check` 가 Issue 생성 시도하다 사망). 자문과 무관 — `02_status.md` 가 기록한 그것 |
| 2026-08(31런) | **0일** | — |
| 2026-09-02 .. 09-06 | **5일** | ★★**새 자문** — `rtrb` **RUSTSEC-2026-0274**(실패 런 로그 직접 확인 · `error: 1 vulnerability found!`). ★**의존성은 그대로였고 자문이 «새로 떴다»** |
| 2026-09-17 .. 09-19 | **3일** | ★**우리 lockfile 교체** — base swap `36df9c31`(= **PR #161**)이 C-1·C-2 를 되돌렸다 |

⇒ ★★**워크플로가 고쳐진 뒤(2026-07-23~) red 8일 중 «5일(62.5%)이 PR 의 잘못이 아니다».**
★**게이팅이면**: 그 5일 동안 **열린 PR 전건이 «자기가 만들지 않은 이유로» 머지 불가** ⇒ `--ignore` 압력이 정확히 그 자리에서 생긴다.
그리고 Constraint 5 는 그 압력의 출구(`--ignore` 남발 · `continue-on-error`)를 **금지**한다 ⇒ ★**막다른 길이 된다.**

★**그런데 비게이팅을 «어떻게» 만드느냐가 함정이다**:
- `continue-on-error: true` ⇒ ★**Constraint 5 의 문면과 정면 충돌**(「never `continue-on-error`」). **쓰면 안 된다.**
- ★**required 컨텍스트에 넣지 않는 것** ⇒ 잡은 **정직하게 red 를 낸다**(종료코드 그대로)지만 **머지를 막지 않는다**.
  현행 required 5개는 `contract`·`build-web`·`rust_ci (ubuntu|macos|windows-latest, stable)` 이고
  ★**새 컨텍스트는 자동으로 required 가 «되지 않는다»** ⇒ 배선만으로 비게이팅이 성립한다. **이쪽이 정답이다.**

★**「아무도 안 보면 조용한 0 아닌가」에 대한 답**: ★**일일 schedule 잡을 그대로 둔다.** 경보 채널은 **바뀌지 않는다** —
PR 단계 잡은 «경보»가 아니라 **«귀속(attribution)»** 이다. 이번 사고의 실제 비용은 24시간 지연이 아니라
★**3일 방치 + 경계 이분 탐색**이었고, PR 에서 빨개지면 **범인이 그 PR 로 즉시 지목**된다.

---

## ⒟ 권고 — ★**⑴ 채택. 단 «비게이팅» + «lockfile 변경 시에만».**

| 축 | 값 |
|---|---|
| 형태 | ★**`pull_request` 에서 `paths: ['**/Cargo.toml','Cargo.lock']` 로만 발화** · `cargo audit` **무-플래그**(Constraint 5 그대로) |
| 게이팅 | ★**required 컨텍스트에 «넣지 않는다»** — `continue-on-error` **금지**(Constraint 5) · 잡은 정직하게 red |
| 가격 | 60 PR 표본에 ★**1.3 분**(docs-only 헛돈 **0**) · 임계경로 **+0 s** · 분 과금 **0**(공개 repo) |
| 이득 | 표본의 **3.3%**(2/60)가 발화 대상이고, ★**그 둘 중 하나가 이번 사고(#161)** 였다 |
| 남는 것 | 새 외부 자문(62.5% 계급)은 **일일 잡이 계속 진다** — PR 단계는 그 계급을 **보지도 않는다**(lockfile 안 바뀌면 안 돈다) |

★★**왜 「무조건 PR 마다」가 아닌가** — `paths` 필터가 **약화가 아니라 «옳은 술어»**이기 때문이다:
PR 단계 잡의 **고유 가치는 «이 PR 이 lockfile 을 바꿨다»** 하나이고, **자문이 새로 뜨는 계급은 일일 잡의 일**이다.
무조건 돌리면 비용이 **30배**(1.3분 → 38.6분)가 되면서 ★**늘어나는 것은 «무관한 PR 의 red» 뿐이다.**

★★**함정 명시 — 이 컨텍스트를 나중에 required 로 «올리지 마라».**
`paths` 필터가 붙은 required 체크는 **그 경로를 안 건드린 PR 에서 영원히 «Expected — Waiting for status»** 가 된다
(`AGENTS.md` Incident ledger 의 그 사고 · `doc-liveness` 가 같은 이유로 「never make this required」다).
⇒ 구현 회차는 그 한 줄을 **워크플로 머리주석과 `AGENTS.md` 에 함께** 박아야 한다.

### ★선행 두 기각과 «같은 근거인가» — ★**아니다**(같았으면 세 번째 기각이었다)

| | 2026-09-07 | 2026-09-18 | **이 회차** |
|---|---|---|---|
| 비용 | node 전용 잡에 **네이티브 cargo 빌드** 추가 · 또는 matrix **6 legs 중복** | **168 s × 6 legs ≈ 17 min/PR** | ★**17 s · 1 leg · 컴파일 0** |
| 탐지 델타 | 「narrow, not zero · ★**zero incidents ever observed**」 | 텍스트 경로 회귀가 **아직 없다** | ★**관측된 사고 1건**(3일 red + 이분 탐색 · PR #161) |
| 자원 | self-hosted 러너 경합 | self-hosted 러너 경합 | ★**GitHub 호스티드 · 큐 p50 2 s · 과금 0** |

⇒ ★**두 기각의 근거는 «비싸고 잡은 적이 없다»** 였고, 이 축은 ★**«싸고 이미 한 번 잡혔어야 했다»** 다. **다른 근거다.**
※그 둘을 뒤집자는 말이 **아니다** — 그쪽의 수(168 s × 6)는 지금도 참이고, 이 회차는 그 결정을 **건드리지 않는다**.
★**다만 그 표의 「self-hosted runner」 서술은 `wie` CI 에 대해 틀렸다** — 그 한 줄은 별 회차가 고칠 일이다(아래 후속).

---

## ⒠ 경계 · 이 회차가 하지 «않은» 것

`.github/workflows/**` 변경 **0**(`git diff --name-status` 로 증명) · 의존성 **0** · `--ignore` 추가 **0** ·
schedule 워크플로 **무접촉** · 형제 `#p0`(§C 되돌림 탐지) **무접촉** · `AGENTS.md` **무접촉**.
★**머지하지 않는다** — `wie` 는 upstream 동기 repo.
★로컬 `cargo audit` 실행은 **읽기 전용**이고 advisory-db 콜드 측정은 `mktemp -d` 에서만 했다(디렉터리째 삭제).

## 한계

1. ★**권고이지 구현이 아니다** — 「`paths` 필터 + 비-required」가 실제로 그렇게 도는지는 **구현 회차가 실측**해야 한다.
2. ★**표본 60 PR·87 schedule 런은 «이 시간창»의 값**이다. 특히 **docs-only 36.7%·lockfile 3.3%** 는 이 저장소의
   현재 작업 성격(문서·판정 회차가 많다)을 반영하고, 성격이 바뀌면 같이 바뀐다 — **인용하지 말고 다시 재라.**
3. ★**「새 자문 62.5%」는 red «일수» 기준**이지 «자문 건수»가 아니다. 클러스터 2개(5일·3일)가 전부라
   **n=2 의 비율**이고, 그 사실이 이 수의 가장 큰 약점이다.
4. ★**`paths` 필터는 «lockfile 을 안 바꾸고 취약해지는» 갈래를 못 본다** — 예: `[patch]`/git rev 핀이 같은 ref 에서
   내용만 바뀌는 경우. 그 갈래는 **일일 잡이 계속 진다**(그래서 일일 잡을 없애자는 권고가 아니다).
5. ★**2026-09-18 표의 「self-hosted」 서술 정정은 이 회차가 하지 않았다** — `AGENTS.md` 무접촉 경계가 우선한다.
