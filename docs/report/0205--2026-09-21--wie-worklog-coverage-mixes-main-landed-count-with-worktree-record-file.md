## [2026-09-21] 착지 수는 `origin/main`, 기록은 «내 트리» — 그 짝이 없는 적체를 만들고, 그 rc=1 이 «따르면 오염되는» 지시를 찍었다 (wie-worklog-coverage-mixes-main-landed-count-with-worktree-record-file)

### A1 — 3지점 재현(내 시각 2026-09-21T23:0x~23:3xZ · 브리프의 수는 그 사이 158→**159** 로 움직였다)

| 어디서 재나 | landed | recorded | rc |
|---|---|---|---|
| `cb53573d`(PR #233 head · 미머지 · base `4acb1631`) | 159 | **145** | **1 `✗ OVERDUE`**(14 > 10) |
| `refs/pull/233/merge` = `654e1940` · **부모 `4acb1631 cb53573d`** | 159 | **145** | **1** — 캐시 ref 의 base 가 14착지 낡았다 |
| 현 `main`(`9c3c2e13`)과의 **실제** 머지 `46c875f1`(부모 `9c3c2e13 cb53573d`) | 159 | **155** | **0 `OK`** |

경계는 **165**(= 155+10)로 **여전히 유효**하다 — 159 이므로 6착지 남았다. ⇒ 브리프의 「기한 미도래」 전제는 내 시각에도 참이다.

★**PR #233 의 `contract` 는 지금 «pass» 다** — 그 체크런이 main 이 155 행을 싣기 «전»에 돌았기 때문이다. ⇒ ★**이 거짓 양성은 시간의존이고, 재실행 한 번이면 «required 체크»가 빨개진다.** 잠복이지 부재가 아니다.

### A2 — 결함인가 설계인가: ★**결함이다**(단, «두 축이 다른 ref 를 본다»는 것 자체는 설계다)

두 축이 갈리는 것은 **필요**하다 — `--record` 는 작업트리에 쓰므로, 방금 기록한 회차가 착지 전에 «현재»로 읽히려면 트리 사본을 읽어야 한다. 갈린 것을 **어떻게 합치느냐**가 틀렸다.

- `AGENTS.md:960-964` — 이 파일은 **ledger file** 이고 「Resolve a merge conflict in any of them by **union** … **Never take one side wholesale**」.
- `:977-983` — 「append-only evidence, so **union is the only correct resolution** — taking one side drops a recorded measurement」.

⇒ ★**검사기는 «내 트리 사본만» 읽어 «take-ours 머지»를 모델링했고, 그것은 이 파일에 대해 같은 문서가 금지한 해결이다.** 브리프 ⓑ 의 갈래로는 ⒝「잘못 쓰고 있는 것」 ⇒ 처방은 **코드**(Contract 1 의 «같은 ref» 갈래)다.
부수적으로 헤더의 자기주장 「it has no false positives」가 반증됐다 — 그 문장을 **그 자리에서 정정**했다(고치지 않으면 다음 사람이 그 문장을 근거로 이 결함을 다시 닫는다).

### 고친 것 — 한 파일, 세 갈래

1. **union read**: `last` = `origin/main` 사본 ∪ 작업트리 사본의 최대 `landedRounds`. 두 사본을 **다 읽는다**(트리만 읽으면 위 결함, main 만 읽으면 `--record` 로 기한을 해소할 길이 사라진다).
2. **Contract 2 — OWNER 지시를 조건부로**: `origin/main` 과 교차확인된 읽기에서만 「`--record` 하라 · OWNER: 게이트③」이 나온다. 교차확인이 **안 된** 국면(= `origin/main` 부재/판독불가 ⇒ `HEAD` 폴백)에서는 대신 ★「이 rc=1 은 낡은 base 의 인공물일 수 있다 — **현 `main` 을 이 브랜치에 실제로 머지해** 다시 재라(★`refs/pull/N/merge` 로 재지 마라 — 캐시 base 가 낡는다) · `--record` 하지 마라 · **주인을 지목하지 않는다**」가 나온다.
3. **`--record` 가드 3**: 기한이 찼어도 트리 사본이 `head` 보다 뒤처졌으면 **쓰지 않고** 「base 를 먼저 당겨라」를 찍는다(그대로 쓰면 main 의 행이 빠진 파일을 만든다).

### A3 — 양방향 실증

| 국면 | 산출 | rc |
|---|---|---|
| 낡은 base(`cb53573d`) · 기한 미도래 | `OK` · **OWNER 지시 없음** · 「(from origin/main; 트리 사본은 145 — base 가 뒤처졌다)」 | **0**(종전 1) |
| 낡은 base + `--record` | `not due (4 landed since 155, cadence 10) — nothing to do` · **파일 변경 0** | 0 |
| ★기한이 «실제로» 찬 · 교차확인됨(픽스처: 클론에서 `origin/main` 을 «기록 145 로 잘라낸» 커밋으로 `update-ref`) | **종전 OWNER 문면 그대로** | **1** |
| ★기한이 찼고 교차확인 **안 됨**(같은 픽스처 + `update-ref -d refs/remotes/origin/main`) | **인공물 경고 문면** · OWNER 없음 · `NOTE:` 한 줄 | **1** |
| 기한이 찼고 트리가 뒤처짐(가드 3) | `pull base first; appending here writes a copy missing origin/main's rows` · **파일 변경 0** | — |

★**⑶ 의 오염이 «산문이 아니라 기계»임을 쟀다** — 낡은 트리(`cb53573d`)에서 **종전** `--record` 를 돌리면 실제로 `landedRounds: 159` 행을 **썼다**(9행 추가 · 결과 파일은 17행/최대 159인데 ★**main 이 이미 가진 155 행이 없다**). 즉 한 번의 쓰기가 ⒜없는 기한을 기록하고 ⒝main 의 행을 떨어뜨린다. 고친 판본은 같은 자리에서 **아무것도 쓰지 않는다**.

### A4 — 회귀 0

`origin/main` 트리에서 **stdout 바이트 동일**(`diff -q` 통과) · old rc=0 ↔ new rc=0. 현재 green 을 red 로 만들지 않는다.

### Contract 3 — 무엇을 잃는가(대조군 포함 실측)

★**자기 사본에서 «행을 지운» 브랜치를 더는 못 잡는다** — `head` 사본이 그 수를 복원해 green 이 된다.

| 트리 상태 | old rc | new rc |
|---|---|---|
| `landedRounds:155` 행 **삭제** | **1** | ★**0** ← 잃은 것 |
| 삭제 없음(대조군) | 0 | 0 |

⇒ 잃은 범위는 **정확히 «행 삭제» 한 부류**이고 그 이상이 아니다(대조군이 그것을 가둔다). 그 퇴행은 여기 말고는 보이는 자리가 없었다 — 그래서 헤더에 적었다.

### 게이트

fmt · clippy(stable) · clippy(wasm32) · `cargo +beta clippy` · `RUST_MIN_STACK=4194304 cargo test --all`(FAILED **0**) · `npm run audit` PASS · `check-worklog-json` OK(165) · `check-doc-liveness-parity` OK(26줄) · `check-docs-report-serial --next-serial` → **0205**.

### 한계 · 안 한 것

- ★**`refs/pull/N/merge` 가 낡는 것은 고치지 않았다**(GitHub 캐시 · 브리프 ⓒ). 한 것은 **안내가 그 ref 를 가리키지 않게** 한 것뿐이다.
- ★**형제 무접촉**: `check-docs-report-serial.mjs`·`check-worklog-json.mjs` diff **0** · `AGENTS.md` 정책 **무접촉** · `docs/worklog-coverage-remeasures.json` **무접촉**(기한 미도래 — 건드리면 이 회차가 자기가 고치는 결함을 재현한다).
- ★**`AGENTS.md:979` 의 「the checker reads `measurements.at(-1)`」는 낡았다**(검사기는 이미 최대값을 읽는다 · 그 파일 헤더가 그렇게 적는다). Contract 4 의 「`AGENTS.md` 정책 무접촉」 경계가 우선해 **고치지 않았다** — 별 회차 몫(후속 제안 `#p0`).
- 교차확인 실패 갈래(`crossChecked === false`)는 실사용에서 드물다(`origin/main` 부재 클론 등). 픽스처로만 실증했고, 그 갈래가 **평시에 도는 것은 보지 못했다**.
