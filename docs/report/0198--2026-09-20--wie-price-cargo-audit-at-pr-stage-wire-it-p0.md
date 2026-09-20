## [2026-09-20] PR 단계 `cargo audit` 을 «배선»했다 — paths 필터 + 비-required, 그리고 「required 로 올리지 마라」를 두 곳에 못박았다 (wie-price-cargo-audit-at-pr-stage-wire-it-p0)

채택 제안 `2026-09-20-price-cargo-audit-at-pr-stage#p0` 의 이행.
선행 리니지(`…-recommend-only{,-fix,-fix2}`)는 **가격만 재고 권고로 끝났다** — 그 회신이 스스로 「권고가 채택되면 **구현 티켓**」이라 적었고(`…-recommend-only.done.md:172`), 이 회차가 그 권고를 집행한다.

### ⒞ 먼저 — 이미 돌고 있나

아니다. 워크플로 **10개 전수**에서 `cargo audit` 문자열은 **`rust-audit.yaml` 한 곳(5회)** 뿐이고, 그 워크플로의 트리거는 **`schedule` + `workflow_dispatch`** 다(`pull_request` **0**). ⇒ **PR 단계 감사는 존재하지 않았다.**

### ⒝ 선행 회차가 «왜 배선을 미뤘는지» — 그 이유는 «범위»였고, 지금은 서 있지 않다

원 회신 한계 ⑴: 「★**권고이지 구현이 아니다** — 「`paths` 필터 + 비-required」가 실제로 그렇게 도는지는 **구현 회차가 실측**해야 한다」.
⇒ 미룬 것은 **기술적 장애가 아니라 산출물 경계**(그 티켓의 산출은 「권고 문서 1편」)였다. ★**그러므로 배선을 막는 전제는 없다.** 그 회차가 기각한 두 대안(⑵전면 기각 ⑶조건부)도 **뒤집지 않았다** — 이 회차는 채택안 ⑴만 집행한다.

### ★★가장 중요한 발견 — `rust-audit.yaml` 에 `pull_request` 를 «붙이면 안 된다»

가장 싼 배선은 기존 워크플로에 트리거 한 줄을 더하는 것이다. ★**그런데 그 파일이 자기 안에 반대 결정을 적어 두었다**(`:106-109`, `check-audit-warnings.mjs` 단계 주석):

> NOT a required check and cannot become one by accident: this workflow has no `pull_request` / `push` trigger (schedule + workflow_dispatch only), so it never reports on a PR. **Promoting it is a deliberate act — do it after the ledger check has a track record, not before.**

⇒ 거기에 `pull_request` 를 붙이면 `cargo audit` 만이 아니라 ★**대장 §A 검사기까지** PR 에 올라가고, 그것은 **이 회차가 뒤집을 결정이 아니다**.
⇒ ★**별 워크플로 `.github/workflows/pr-audit.yml` 로 지었고, 그 안에서 `cargo audit` «만» 돈다.** 일일 잡은 두 단계를 그대로 유지하고 **경보 채널로 남는다**.

### 계약 1 — ★「required 로 올리지 마라」를 못박은 «경로 두 곳»

| 파일 | 무엇을 적었나 |
|---|---|
| **`.github/workflows/pr-audit.yml`**(머리주석) | ★**NEVER MAKE THIS A REQUIRED CHECK** + 이유 **둘** |
| **`AGENTS.md`**(§Incident ledger · `Never make doc-liveness required` 바로 뒤) | ★**Never make `pr-audit` required either** + 「이 검사에만 있는 둘째 이유」 |

★**이유를 «둘» 적은 것이 요점이다.** `doc-liveness` 가 가진 것은 ⑴뿐이다:
- ⑴ **paths 필터 + required = 교착.** 그 경로를 안 건드린 PR 이 영원히 「Expected — Waiting for status」에 선다(AGENTS.md Incident ledger 의 그 사고).
- ⑵ ★**이 검사에만 있는 이유**: `cargo audit` 은 ★**새 자문이 «공개되는 것»만으로** 이 repo 변경 0 인 채 red 가 된다. **남이 밤새 걸 수 있는 게이트**는 사람이 우회하는 물건이 되고, Constraint 5 가 그 우회로(`--ignore`·`continue-on-error`)를 이미 막아 두었다.

★**비-게이팅을 «`continue-on-error` 로 만들지 않았다»** — Constraint 5 금지 대상이다. 방법은 ★**「required 목록에 넣지 않는 것」** 뿐이고, 현행 required 5개(`contract`·`build-web`·`rust_ci (…, stable)`×3 · AGENTS.md 정본 실측)에 **새 컨텍스트는 저절로 들어가지 않는다** ⇒ **배선만으로 성립한다.**

### 계약 2 — paths 필터

```
Cargo.toml · Cargo.lock · **/Cargo.toml · .github/workflows/pr-audit.yml
```
(YAML 파싱으로 확인 — 네 항목 · `types: [opened, synchronize]` · `continue-on-error` 키 **잡·스텝 전건 없음**)

넷째 항목은 **자기 시험**이다: ★GitHub 은 기본 브랜치에 아직 없는 워크플로의 `workflow_dispatch` 를 거부하므로(`doc-liveness.yml` 이 실측으로 적어 둔 404), 그 항목이 없으면 **이 잡은 시험되지 않은 채 착지한다**. 다른 PR 은 **0초**를 문다.
`docs/report/0185` 의 실측: 이 필터로 최근 60 PR 중 **2건(3.3%)** 만 발화하고, ★**그 2건 중 하나가 정확히 이번 사고(PR #161 의 base swap)** 다.

### 계약 4 — 개악 양방향 ★둘 다 «돌려서»

★**실제 CVE 를 심었다 — `--deny` 류 대체가 필요 없었다.** 심은 것은 가상의 결함이 아니라 ★**이 배선이 존재하는 이유가 된 그 되돌림**이다(base swap `36df9c31` 이 실제로 한 것).

| | 무엇을 했나 | 결과 |
|---|---|---|
| ⒝ **green** | 손대지 않은 트리에서 `cargo audit` | **rc=0** · `2 allowed warnings found`(ttf-parser unmaintained · chacha20 yanked) |
| ⒜ **red** | `cargo update -p rtrb --precise 0.3.4`(= RUSTSEC-2026-0274 · §C 의 C-1 이 해소한 판본으로 되돌림) | ★**rc=1** · `error: 1 vulnerability found!` |

★`Cargo.lock` 은 **백업에서 원복**했고 `rtrb = 0.3.5` 로 돌아온 것을 확인했다(작업트리 청결 · 커밋 0).
★**그리고 그 개악은 `Cargo.lock` 을 건드리므로 위 paths 필터에도 걸린다** — 두 축이 같은 사건에서 맞물린다.

### 왜 지금인가 · 대가 (각 한 줄)

- **왜 지금인가**: 선행 회차가 가격을 다 재고 「구현 티켓」이라 적은 채 멈췄고, 그 사이 이 배선이 막았을 사고가 **3일 지연 + 경계 이분 탐색**으로 처리됐다(`docs/report/0181`).
- **대가**: ★**러너 시간은 실질 0**(p50 **17s** · 툴체인 설치 0 · 컴파일 0 · 공개 repo 라 분 과금 0 · 60 PR 중 **2건**만 발화). ★**진짜 대가는 «아무도 안 봐도 되는 red»가 신호를 마모시키는 것**이고, 그것을 받아들이는 근거는 **일일 잡이 여전히 경보 채널이고 이 잡은 «귀속»일 뿐**이라는 것이다(제안이 적은 tradeoff 그대로).

### 한계 — 숨기지 않는다

1. ★**`Cargo.lock` 을 안 바꾸고 취약해지는 갈래는 못 본다** — `[patch]` 항목이나 git `rev` 핀의 내용이 같은 ref 아래서 바뀌는 경우. **일일 잡이 계속 그 갈래를 진다. 이것은 대체가 아니라 귀속이다.**
2. ★**대장 §A 검사기(`check-audit-warnings.mjs`)는 PR 에 올리지 «않았다»** — 그 단계 자신의 결정 문면(「track record 가 쌓인 뒤에」)을 존중했다. 그 승격은 별 회차의 몫이다.
3. ★**CI 에서 이 잡이 도는 것은 «이 PR 에서 처음» 본다** — 자기 시험 경로가 그것을 보장하지만, 착지 전 관측은 이 PR 한 번뿐이다.
4. ★**비-required 는 «지금의 ruleset»에 의존한다** — 누가 이 컨텍스트를 required 로 올리면 ⑴의 교착이 즉시 난다. 그것을 막는 것은 **위 두 문서의 문장**이고, ★**기계가 아니다**(`scripts/check-branch-protection-claim.mjs` 는 required 목록의 «드리프트»를 잡지만, 그것은 사람이 손으로 돌리는 가드다).
