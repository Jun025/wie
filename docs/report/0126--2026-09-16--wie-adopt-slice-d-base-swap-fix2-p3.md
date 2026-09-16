## [2026-09-16] upstream 워크플로 둘 — «채택하지 않는다»로 «결정»했고, 그 결정을 기계로 잠갔다 (wie-adopt-slice-d-base-swap-fix2-p3)

**무엇을**: 제안 `2026-09-16-slice-d-base-swap-fix2#p3`(「채택할지 **정한다**」)을 채택하고 ★**결정을 냈다: 둘 다 채택하지 않는다.**
파일은 **지우지 않고** «주차»를 **«유예»에서 «결정»으로 승격**했다(두 헤더 문안 교체) + ★**그 결정을 지키는 검사기 1개**를 추가했다.
★**제품 동작 0줄** — 엔진·웹 표면 무접촉. 바뀐 것은 **주석 2개 · 검사기 1개 · CI 스텝 1개**다.

**왜**: 제안이 「⒜삭제 / ⒝채택 / 지금은 주차(어느 쪽도 아님)」를 나란히 놓고 **정하라**고 했다.
★**주차는 안전하지만 «결정»이 아니다** — 그리고 아래 ⑷에서 보듯 **저절로 풀린다**.

---

## ⓐ 제안이 지금도 유효한가 — ★**유효하다**(실측)

| 축 | 실측 |
|---|---|
| 두 파일 실재·주차 상태 | `release.yaml`·`web.yaml` 모두 `on:` = ★**`workflow_dispatch:` 단독** |
| 결정이 내려졌는가 | ★**아니다** — 두 헤더가 「the adoption decision **stays open**」·「a decision … **this round is not the place to make it**」이라고 **명시**하고 있었다 |
| 멈춰 있는 이유 | `package.json` scripts = `audit build:wasm db:migrate:local db:migrate:remote deploy dev frontend verify` ⇒ ★**`build:prod`·`build:dev` 둘 다 부재**(제안 문면 그대로) |
| 「나머지 기계는 실재한다」 | ★**참이다** — `.github/scripts/release/{resolve-context,publish,configure-android-signing}.sh` **3건** · `wie-app` 디렉터리 **실재** |

## ⓑ 같은 것을 하는 축이 있는가 — ★**웹은 있고, 릴리스도 있다. 그것이 «채택 안 함»의 근거다**

- `web.yaml` ↔ ★**우리 `web.yml`**(job `build-web` · Pages 프로젝트 `wie-web`) — **같은 트리거 면**(push main + PR).
- `release.yaml` 의 `publish` job ↔ ★**우리 `publish-artifact.yml`** — 릴리스를 **이미 소유**하고 otterpebble 이 `repository_dispatch` 로 소비한다.

## ⓒ 제안이 틀렸는가 — ★**틀리지 않았다. 다만 «한 축»을 몰랐다**

제안의 ⒝(채택) 대가 목록은 「package.json 스크립트 · Pages 프로젝트명 · 릴리스 소유권 · wie-app Tauri 유지」였다.
★**그 사이 `Cargo.toml` 이 `exclude = ["wie-app"]` 을 넣었다** — 그 주석의 실측: `wie-app` 은
★**gtk/webkit2gtk/libsoup3 의 «유일한» 루트**라 Linux CI 에서 `cargo clippy --all` 과 `tarpaulin` 을 **죽였고**
macOS·Windows 는 green 이었다. ⇒ ★**⒝ 를 고르면 그 breakage 를 «일부러» 되가져온다** — 제안이 적은 것보다 비싸다.

---

# 결정 — ★**둘 다 채택하지 않는다**

## `release.yaml` — 채택 안 함

★**job 그래프 실측**: `context → web` + **`windows`·`linux`·`macos`·`android`·`ios`** + `deploy_web` + `publish`
⇒ ★**Tauri 로 5개 데스크톱/모바일 타깃**을 만든다.
★**결정 근거는 «취향»이 아니라 `AGENTS.md` 의 Goal 이다** — 「shippable on **two hosts**: `wie_cli` (native) · `wie_featurephone` (browser)」.
그 다섯은 **우리가 배포하는 호스트가 아니다.**
그리고 ⓒ의 `wie-app` 축 · 릴리스 발행자 이중화(소비자 가시 위험) · Pages 프로젝트명(`wie`/`wie-dev` ↔ 우리 `wie-web`)이 **우리 토큰으로** 돈다는 점이 겹친다.

## `web.yaml` — 채택 안 함

★**우리 `web.yml` 의 중복**이고 트리거 면이 **같다**. 게다가 브랜치 집합이 **한 세대 낡았고**(우리 `web.yml` 은 `feat/*` 프리뷰 브랜치를 포함), `build:dev` 가 **없다**.
⇒ ★**「채택」의 어떤 형태도 «웹 CI 를 둘 돌리는데 하나는 깨져 있다»가 된다.**

## 삭제가 아니라 «주차 유지» — ★**왜 ⒜(삭제)를 고르지 않았나**

제안 자신이 적었다: 「삭제하면 **다음 upstream 동기마다 그 삭제를 다시 해야 한다**」.
★**그리고 이 저장소에 «같은 트레이드»의 선례가 바로 옆에 있다** — `Cargo.toml` 의
`exclude = ["wie-app"]` 이 「Excluded rather than deleted: adopting the desktop shell later is a **deliberate act**」라고 적었다.
⇒ ★**같은 형태로 통일했다: 지우지 말고 · 무력화하고 · 이유를 적는다.** 원본은 `git show upstream/main:…` 로 언제든 나온다.

---

# ⑷ ★**그런데 «주차»는 저절로 풀린다 — 그래서 검사기를 붙였다**

★★**이 회차가 새로 잰 것이고, 이 산출물의 핵심이다.** 주차는 ★**upstream 파일에 가한 «우리 로컬 편집»**이다
⇒ ★**다음 `git merge upstream/main` 이 원본 트리거를 되돌릴 수 있고, 그것을 볼 축이 «0»이었다.**

`upstream/main` 원본 실측:
```
release.yaml  on: workflow_dispatch · schedule(cron "17 0 * * *") · push(tags v*)
web.yaml      on: push(main) · pull_request(opened, synchronize)
```
⇒ 되살아나면 ★**매일 밤** 우리 `CLOUDFLARE_API_TOKEN` 으로 `wie`/`wie-dev` 에 `pages deploy` 하고
`publish-artifact.yml` 이 소유한 자리에 **릴리스를 또 발행**한다.

★**오늘 그것을 막는 것은 `build:prod` 부재뿐**이고 — 주차 회차 자신이 ★**「That is luck, not a guard」**라고 적었다.
⇒ ★**그 문장을 그대로 두고 결정만 적으면, 결정은 «다음 머지»에 조용히 뒤집힌다.**

**신설**: `scripts/check-parked-workflows.mjs` + `engine-contract.yml` **always-run 스텝**(여섯째).
★**always-run 인 이유가 이 검사의 전부다** — 재무장시키는 diff 는 **`.github/**` 만 만지는 upstream 머지**라
★**엔진 keyed paths 필터가 정확히 그 PR 을 건너뛴다**(형제 두 스텝이 같은 논거로 always-run 이다).
★**범위는 «트리거 면»뿐**이다 — job 본문을 읽지 않고, 목록 밖 워크플로에 의견이 없다.
★**해제는 정당한 행위다** — 다만 `PARKED` 배열을 같은 커밋에서 고쳐야 하므로 ★**«의도»가 되고 검수에 걸린다.**

## ★양방향 — 개악 대조 (Acceptance ⑴)

★**제품 호출부(워크플로 파일 자신)를 «upstream 원본대로» 되돌린 개악**이다 — 상수 대 상수가 아니다.

| 형상 | `node scripts/check-parked-workflows.mjs` |
|---|---|
| **정상** | ★`2 parked, 0 violation` · **rc=0** |
| ★**개악 1** `release.yaml` 에 원본 `schedule`+`push(tags)` 복원 | ★**rc=1** — `triggers = [workflow_dispatch, schedule, push]` |
| ★**개악 2** `web.yaml` 에 원본 `push(main)`+`pull_request` 복원 | ★**rc=1** — `triggers = [push, pull_request]` |
| ★**개악 3** 흐름열 `on: [push, pull_request]` | ★**rc=1** — 같은 판정 |
| ★**개악 4** 파일 삭제(fail-closed) | ★**rc=1** — `is missing — … drop it from PARKED` |
| **복원** | ★`2 parked, 0 violation` · **rc=0** |

★★**개악 3 은 «내가 오늘 실제로 밟은 함정»이라 일부러 넣었다.** 직전 회차에서 나는 `on:` 의 **블록 매핑**만 보는 술어로
main-push 워크플로를 열거했다가 ★**`coverage.yml` 의 `on: [push]`(흐름열)를 놓쳤다.**
⇒ 이 검사기는 **두 형식을 다 파싱**하고, ★**파싱 결과가 0개면 «주차»가 아니라 «위반»으로 접는다**
(「0을 통과로 읽는다」는 이 저장소가 반복해 이름 붙인 실패형이다).

---

## ★대가 — Contract 2

⒜**무엇을 잃는가**: ⑴★**결정을 «뒤집는 비용»이 올라간다** — 데스크톱/모바일 호스트를 실제로 원하게 되면
이제 헤더 + `PARKED` 배열 + `Cargo.toml exclude` **세 곳**을 함께 풀어야 한다. ★**그것이 의도다**(그 순간은 제품 결정이지 CI 청소가 아니다) —
그러나 **비용은 실재**하고, 「채택 안 함」이 틀린 결정이면 이 회차가 그 전환을 **비싸게 만든 것**이 맞다.
⑵★**죽은 설정이 트리에 남는다** — 제안이 지적한 그 비용을 **치르기로 한 것**이고, 대신 헤더가 「왜 여기 있나」를 말한다.
⑶CI 스텝 1개(node 가 파일 **둘** 읽기 · 네트워크·node_modules 0) — 측정 불가 수준.
⑷★**검사기의 «천장»**: 트리거 면만 본다 ⇒ upstream 머지가 **job 본문**을 바꿔 놓아도 **조용하다**.
★**그건 알고 남긴 값이다** — 본문까지 잠그면 upstream 파일을 통째로 핀하는 것이라 **동기마다 충돌**한다.

⒝**안 하면 무엇이 나쁜가**: ⑴★**결정이 없으면 다음 회차가 같은 질문을 다시 연다** — 이미 두 회차가
「이 라운드가 정할 자리가 아니다」로 미뤘다. ⑵★**더 나쁜 쪽은 «저절로 풀림»이다** — 주차는 로컬 편집이고
upstream 머지가 되돌릴 수 있는데 ★**그것을 보는 축이 0이었다.** 되살아나면 피해는 **우리 계정의 야간 배포 + 이중 릴리스 발행**이고,
★**소비자(otterpebble)가 엉뚱한 파일을 받는 형태**라 조용히 퍼진다.

## 사용자 영향

**없다**(이번 PR 만으로는). 제품 코드 0줄 · 두 워크플로의 **동작은 이전과 동일**(여전히 `workflow_dispatch` 전용).
바뀐 것은 ★**그 상태가 «지켜진다»는 것**이다.

## 한계 — 숨기지 않는다

- ★**검사기는 트리거 면만 본다**(위 ⒜⑷). job 본문 드리프트는 미판정이다.
- ★**`PARKED` 목록은 손으로 관리한다** — upstream 이 **새 워크플로**를 들고 오면 이 검사기는 **모른다**.
  「upstream 신규 워크플로를 자동 탐지」는 별 축이고, 이 제안이 묻지 않았다(후속 제안으로 뺐다).
- ★**이 회차는 두 파일을 «지우지 않기로» 했을 뿐 «영원히»를 뜻하지 않는다** — 헤더에 REOPEN 조건을 리터럴로 적었다.
