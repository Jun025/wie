## [2026-09-16] 리눅스가 못 빌드하는 의존이 되돌아오면 PR 에서 빨개진다 — members 가 아니라 lock 을 본다 (wie-adopt-slice-d-base-swap-fix3-p0)

### 무엇을

채택 제안 `2026-09-16-slice-d-base-swap-fix3#p0`(원문 =
`docs/worklog/2026-09-16-slice-d-base-swap-fix3.json` `proposals[0]`)을 집행했다.

- `scripts/check-linux-system-deps.mjs` **신설** — `Cargo.lock` 에 `glib-sys`·`soup3-sys`·`gtk-sys`·
  `webkit2gtk-sys`·`tauri` 중 하나라도 있으면 **rc=1**.
- `.github/workflows/engine-contract.yml` 에 **상시 스텝**(여섯째) 배선.

**제품 코드 0줄.**

### 제안의 처방을 쓰지 않았다 — 그리고 그 근거는 제안 자신이 적어 두었다

제안의 `tradeoff`: 「members 화이트리스트를 검사로 박으면 **정당한 크레이트 신설마다 그 목록을 함께
고쳐야 한다**(검사기와 매니페스트가 **두 벌의 진실**이 되는 형태)」.

★그 비용은 실재하고, **그 축은 게다가 새지도 한다**: 화이트리스트는 「members 가 늘었다」라는 **대리 신호**라
★**기존 member 가 GTK 의존을 새로 얻는 경로**(예: `wie-web` 에 tauri 가 붙는다)를 **못 본다**.

⇒ **술어를 «해악 자체»로 바꿨다**: 리눅스 러너가 빌드할 수 없는 크레이트가 **해결된 그래프에 있는가**.
그 그래프의 정본은 **커밋된 `Cargo.lock`** 이다.

| 축 | members 화이트리스트 | ★`Cargo.lock` 금지 크레이트 |
|---|---|---|
| upstream 이 member 를 되살림 | 잡는다 | **잡는다** |
| 기존 member 가 GTK 의존을 얻음 | ★**못 잡는다** | **잡는다** |
| 정당한 크레이트 신설 비용 | 두 곳 수정 | ★**0** |
| 두 벌의 진실 | members 전체 사본 | 「이미 덴 시스템 의존」 5줄(정당 신설과 무관) |

★**공짜가 아니다** — 5줄은 여전히 사람이 유지한다. 그러나 그 목록이 늘어나는 순간은 **새 시스템 의존이
등장하는 순간**이고, 그때는 사람이 봐야 한다. 화이트리스트는 **평범한 크레이트 하나**에도 세금을 물린다.

### 왜 `Cargo.lock` 이 «믿을 수 있는 증인»인가 — 쟀다

격리 worktree(`origin/main`)에서 **양방향**:

| 상태 | `glib-sys` | `gtk-sys` | `soup3-sys` | `webkit2gtk-sys` | `tauri` |
|---|---|---|---|---|---|
| 현재(=`wie-app` 이 `exclude`) | 0 | 0 | 0 | 0 | 0 |
| ★`wie-app` 을 `members` 로 되돌림 | **1** | **1** | **1** | **1** | **1** |

⇒ 락파일은 그래프의 **대리**가 아니라 **증인**이다.

### 검증 — 양방향 개악 대조 (★실제 기전으로)

개악은 **upstream 당김이 하는 그대로**다: `members` 에 `wie-app` 한 줄 + `exclude` 비우기 → `cargo metadata`.

| 회차 | 결과 |
|---|---|
| 정상(`origin/main` + 새 검사기) | **rc=0** GREEN |
| ★**개악**(`members` 복원) | ★**FAIL · 감시 5종 전건 검출** |
| 복원(`git checkout -- Cargo.toml Cargo.lock`) | **rc=0** GREEN |

종료코드는 따로 못박았다(격리 스크래치 · cargo 무관): 크래프트한 lock → **rc=0** ↔
`[[package]] name = "tauri"` 추가 → **rc=1**.

★**위양성 한 종류를 닫았고 그것도 쟀다**: `dependencies = [ "glib-sys" ]` 줄이 있어도 **rc=0** 이다.
앵커 `^name = "…"$` 가 그 줄을 물지 않기 때문이고, 앵커 없는 부분문자열 판정이었다면 **이 저장소의 현재
lock 이 곧바로 오탐**이 됐다.

### 왜 «상시 스텝»인가 (필터 안이 아니라)

`Cargo.lock` 과 `**/Cargo.toml` 은 **이미 engine 필터 안에 있다**. 그런데도 필터 밖에 둔 이유는 하나다 —
★**이 가드를 침묵시키려면 지울 것이 «스크립트»이고, 그 경로는 필터에 없다.** 그 PR 은 필터 내 스텝을 전부
건너뛰고 green 을 보고한다. `check-parity-lock-wired`·`audit-no-leak`·`docs-report-serial` 세 스텝이
**같은 이유로** 상시다. 비용은 node 가 파일 하나를 읽는 것(네트워크·`node_modules` 0).

배선 확인: PyYAML 파싱 → `contract` 잡 스텝 **19개** 중 새 스텝이 `dorny/paths-filter` **앞**에 있다.

### AGENTS.md 를 건드리지 않았다 — 선례로 결정했다

같은 계급(제안 회차가 만든 상시 가드)의 선례 **`check-parity-lock-wired` 는 AGENTS.md 언급이 0건**이다
(실측: parity-lock-wired **0** ↔ worklog-json 1 · docs-report-serial 1 · engine-runner-fixtures 1 ·
doc-liveness-parity 1 · audit-no-leak 4). ⇒ 이 계급은 **스크립트 헤더 + 스텝 주석**이 집이다.
★부수 효과 하나가 더 있다 — **새 fenced `sh` 줄을 만들지 않았으므로 doc-liveness 패리티 의무
(`doc-liveness.yml` 의 DOC-COPY 동기)가 발생하지 않는다.**

### 무엇을 잃는가 / 안 하면 무엇이 나쁜가

★**「잃는 것이 없다」가 아니다.**
- **⑴ 목록은 사람이 유지한다.** 새 이름의 시스템 의존(`openssl-sys`·`dbus-sys` …)은 누가 적기 전까지
  **통과한다**. ★숨기지 않고 **매 실행 천장으로 출력**한다.
- **⑵ 러너 apt 목록과 어긋날 수 있다.** 누가 `libsoup3` 를 러너에 깔면 이 목록은 **틀린 게 아니라
  보수적**이 된다 — 그때 그 줄을 지우라고 **실패 메시지가 직접 안내**한다.
- **⑶ 타깃 독립성.** lock 항목은 타깃을 가리지 않으므로, 우리가 안 빌드하는 타깃에서만 닿는 크레이트도 문다.
  이 5종에는 **의도된 동작**이지만, 목록을 넓히려면 그 판단을 다시 해야 한다.
- **⑷ 리눅스 레그를 돌려 본 것이 아니다.** 보인 것은 **기전**이고 「ubuntu·coverage 가 green 이다」는
  여전히 **CI 가 낸다**(fix3 회차의 한계 그대로).

**안 하면**: 다음 upstream 당김이 같은 한 줄로 같은 red 를 만든다. ★그리고 **로컬 4게이트는 구조적으로
눈멀어 있다** — macOS 는 그 의존을 **컴파일조차 하지 않는다**. 이번에는 **게이트② 검수자가 사람 눈으로**
잡았고, 그 축을 지키는 기계는 **0** 이었다.

### 범위 밖 — 일부러 안 한 것

- 형제 `#p1`(문서의 «required status check» ↔ 실제 브랜치 보호 일치) **무접촉** — 별 티켓이고,
  제안 자신이 「운영자·총괄 결정이 필요할 수 있다」고 적었다.
- `members` 화이트리스트를 **만들지 않았다**(위 §처방).
- AGENTS.md·`doc-liveness.yml` **무접촉**(위 §선례).
- 리팩터·이름 변경·주변 정리 **0**.
