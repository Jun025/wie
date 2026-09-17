## [2026-09-17] 가드는 옳게 섰는데 그 가드가 자기 가동 방식을 거짓으로 말했다 — 「주 1회」를 걷어냈다 (wie-adopt-slice-d-base-swap-fix3-p1-p1-fix)

**무엇을**: 게이트② `request-changes` 승계. `scripts/check-branch-protection-claim.mjs` 는 **고치지 않았다** —
고친 것은 **그 도구가 자기에 대해 하는 서술**뿐이다. 「`…diffs it against the API weekly`」·
「`the weekly job grants it`」·「주 1회 API 와 대조한다(이 문단이 또 낡으면 **그쪽이 운다**)」를
**「손으로 돌린다 · CI 는 못 읽는다(run `35181122022` · `35180786771`)」** 로 바꿨다.

**왜**: 이 저장소의 병은 「틀린 판단」이 아니라 **「근거 없이 맞아 보이는 문장」**이고,
이번 거짓은 그 병의 **가장 나쁜 형태**였다 — ⒜가드 파일 **머리 주석**이라 다음 사람이 가장 먼저 읽고
⒝이 가드의 유일한 기동 축이 **사람의 규율**인데 그 문장이 정확히 그 규율을 해제시킨다
(「기계가 본다니 안 돌려도 되겠군」) ⒞그러면 `AGENTS.md` 가 기록한 **8주 / 2시간** 부패가 그대로 돌아온다.
★**이 회차가 고치려던 결함과 같은 형태다** — 「문서가 감시를 실제보다 크게 말하고 아무도 안 본다」.

**사용자 영향**: 없다(제품 코드 0줄 · 도구 로직 0줄 · CI 배선 0줄). 바뀐 것은 **다음 회차가 읽는 문장**이다.

### 고친 자리 — 검수 §4 의 넷 + ★**전수 조회가 찾은 다섯째**

| 파일 | 전 | 후 |
|---|---|---|
| `scripts/check-branch-protection-claim.mjs` 머리 | ``Needs `gh` with `administration: read` … (the weekly job grants it).`` | 「Who runs it: you, by hand」 절 신설 — 주기 호출자 **0** · 두 run 번호 · 필요한 것은 **로컬 admin `gh` 또는 PAT** |
| 같은 파일 rc=2 출력(`:141`) | ``grant `administration: read`, or run this where `gh` is authenticated`` | `administration: read` 는 ★**줄 수 없는 키**임을 명시(run `35180786771`) · 「로컬 admin `gh` 또는 PAT」로 좁힘 |
| `.github/workflows/engine-contract.yml` | `…diffs it against the API weekly.` | `…BY HAND, not on a schedule.` + 못 하는 이유 2run |
| `.github/workflows/web.yml` | 동일 문장 | 동일 정정 + 「nothing machine-watches this paragraph」 |
| `docs/upstream-realign-p3-slices.md` | `주 1회 API 와 대조한다(이 문단이 또 낡으면 그쪽이 운다)` | ★**「우는 기계는 «없다»」** — 잡는 것은 사람이고, 그 자리는 `REQUIRED-CHECKS` 블록을 고치는 회차다 |
| ★`docs/worklog/2026-09-17-branch-protection-claim-guard.json` `limits[2]`·`limits[3]`·`#p0.tradeoff` | 「**주 1회 잡**이 설정 감사기로」 · 「**주 1회 잡의 red 소유자 규칙**이 그대로 적용된다」 | 「손으로 도는 이 검사기가」 · ★**「그 red 소유자 규칙은 doc-liveness 잡의 것이고 이 검사기에는 붙지 «않는다»」** |

★**다섯째가 이 회차의 실익이다** — 같은 파일의 `limits[0]` 은 「★주기 실행이 «없다»」라고 **정확히** 적어 놓고
`limits[3]` 이 「주 1회 잡의 red 소유자 규칙이 적용된다」로 **자기를 반증**하고 있었다.

### 전수 조회 — 남은 거짓 **0건**
```sh
git grep -n -i -e 'weekly' -e '주 1회' -e '주1회' -- .     # 34건
```
34건을 **전건 분류**했다: `doc-liveness` 자신·그 잡의 스텝·배치 결정 문서 = **참**(그 잡은 실제로 주 1회 돈다) ·
`STATE.md:74`·`docs/report/0144:42,70`·worklog `summary` = **참**(「시도했고 실측이 막아 되돌렸다 · **지금은 손으로 도는 도구다**」) ·
`wie-wipi-c/src/lib.rs:34` = 무관(핀 이동 주기).
교차 술어로 재조회해 **이 검사기에 붙은 주기 주장 0건**을 확인:
```sh
git grep -n -i -e 'weekly' -e '주 1회' -- . | grep -i 'branch-protection-claim'
# → 남는 1줄은 worklog summary 의 「CI(주 1회)에 얹으려 했으나 … 되돌리고 손으로 도는 도구로 남겼다」 = 참
```

### 검수자가 통과시킨 세 축은 **건드리지 않았다**
§1 독립 실행 · §2 반증(개악 10종) · §3 「CI 배선은 실측이 막았다」 — 이 셋을 건드리면 그 통과가 무효가 된다.
★**대전제(반증)도 내가 다시 쟀다**: `git grep check-branch-protection-claim` **10건 전부 주석** ·
`package.json` scripts **미등재** · `doc-liveness.yml` 의 유일한 언급은 **NOT-RUN 선언 주석**(`:189`) ·
schedule 보유 워크플로 2개(`doc-liveness`·`rust-audit`) 어느 쪽도 이 검사기를 부르지 않는다
⇒ ★**검수자가 옳다. 반증 실패.**

### 수치 동일성 — 서술만 고쳤음을 «값»으로 보인다
```
고치기 전 / 후 — 같은 명령 `node scripts/check-branch-protection-claim.mjs`
claims 5 · enforces 5 · classic protection: none (404) · rulesets: 1 (1 active branch)   rc=0   (전/후 ★글자 단위 동일)
```
