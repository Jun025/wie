## [2026-09-07] `verify-browser` 를 배포 «뒤» 스텝으로 — ⒜ 채택 (wie-verify-browser-as-post-deploy-step-decision)

### 무엇을
`scripts/verify-browser.mjs` 가 저장소에 있는데 **자동으로는 한 번도 실행되지 않았다**. 그것을
`web.yml` 의 **마지막 스텝**(배포 «뒤»)으로 옮겼다. 함께: 실 Chrome 하드코딩을 번들 chromium 으로
바꾸고, **red 를 누가 받는지**를 `AGENTS.md` 에 적었다.

### 왜 — F1 실측

**⑴ 지금 어디서 불리는가 = 0건.** `.github/` 전수:

```
grep -rn 'verify-browser'      .github/   → rc=1 (0건)
grep -rn 'npm run verify'      .github/   → rc=1 (0건)
대조군: 같은 술어가 contract-roundtrip·check-engine-contract·audit-no-leak 은 잡는다
        → engine-contract.yml · publish-artifact.yml
```

**대조군을 붙인 이유**: 이 저장소는 「0건」이 «검사기가 깨져서» 나온 적이 있다(zsh 단어분할 ·
`wie-expect-last-frame-…-fix2`). 술어가 살아 있음을 먼저 보였다.

**⑵ 배포 스텝은 정말 `event_name == 'push'` 뒤인가 — 그렇다.** `web.yml` 의 배포 3형제가 전부
같은 게이트다:

```
:126  Require the account pin …   if: github.event_name == 'push' && HAS_CF_TOKEN == 'true'
:144  Apply D1 migrations …       if: github.event_name == 'push' && github.ref_name == 'main' && …
:159  Ensure Pages project exists if: github.event_name == 'push' && …
:170  Deploy to Cloudflare Pages  if: github.event_name == 'push' && …   ← 파일의 마지막 스텝
on.push.branches: [main, feat/cf-web-service, feat/ux-redesign]   ·  --branch=${{ github.ref_name }}
```

⇒ PR 에는 가리킬 주소가 없고 **push 에는 생긴다** — 제안의 전제가 그대로 맞다.

**⑶ 번들 chromium 이 되는가 — 된다. ⇒ 대가 ⑴(러너에 실 Chrome)이 사라졌다.**

```
contract-roundtrip.mjs:277-280   const { chromium } = await import("playwright")
                                 if (WIE_CHROME_CHANNEL) launchOpts.channel = …   ← 기본 = 번들
verify-browser.mjs:18            chromium.launch({ channel: "chrome" })           ← 하드코딩(고쳤다)
engine-contract.yml:239          npx playwright install --with-deps chromium
```

**⑷ 손 실행 — rc=0.** 운영 URL(`https://wie-web.pages.dev`) 1회, `15597d1d` 착지 배포본:
`rc=0` · off-origin **0** · POST/PUT **(none)** · **게임 헤더 바이트 0** ⇒ `NO-LEAK AUDIT ✅`.
★**«1회» 제약을 지켰다** — 이 값은 이 세션에서 내가 직접 받은 것이고 **다시 부르지 않았다**.
★게임 바이트는 싣지 않는다(형태만).

### ★★그 실행에서 나온 «예정에 없던» 발견 — `rc=0` 은 「화면이 그려졌다」가 아니다

같은 실행의 `canvas pixel stats: {"w":240,"h":320,"nonBlack":0,...}` 이 **rc=0 으로 통과**했다.
코드가 그렇다 — `const leak = offOrigin.length > 0 || bodyCarryingGame.length > 0; process.exit(leak ? 2 : 0)`.

| 실패하는 것 | 실패하지 «않는» 것 |
|---|---|
| 누출(off-origin · 본문에 게임 헤더) → **rc=2** | `console.error` · `pageerror` (기록만 한다) |
| 흐름 미완주(셀렉터 부재 → throw) → **rc=1** | ★**검은 화면**(`nonBlack: 0`) |

★**이 fixture 에서 `nonBlack: 0` 은 «옳다»** — `AGENTS.md` 가 `helloworld_*` 는 **끝이 비는 것이
정상**이라고 이미 적었다(`--expect-last-frame` 결정 블록). ⇒ 스크립트 결함이 아니다.
★**그러나 게이트③ 회차들이 `rc=0` 을 «배포 건강»으로 인용해 왔고**(오늘만 4회 · 그중 넷째가 나다),
**그 인용은 「부팅했고 파일을 받았고 아무것도 새지 않았다」까지만 참이다.** 이 문장을
`AGENTS.md` 와 스크립트 머리주석에 박았다.

### F2 — ⒜ 채택 · 버린 셋의 이유

**⒜ `web.yml` 배포 뒤 스텝(채택).** 번들 chromium 으로 대가 ⑴ 소멸 · 배관 0 · 사람 기억 → 기계.

**⒝ 별 워크플로(기각).** 같은 단언을 얻는 데 워크플로 1개 + 트리거 배선이 는다. 그리고
`deployment-url` 은 **그 job 안에서만** 스텝 출력으로 읽힌다 — 밖에서 하려면 그 URL 을 아티팩트나
API 로 나르는 배관이 또 필요하다. ⒜ 가 하는 일을 더 비싸게 한다.

**⒞ `continue-on-error`(기각) — «폭발반경을 재보니» 살 이유가 없었다.**

```
이 스텝의 if:  github.event_name == 'push' && HAS_CF_TOKEN == 'true'
  ⇒ PR run 은 «건너뛴다»            ⇒ 열린 PR 차단 0건
  ⇒ 게이트③ 은 «PR 의» 검사를 읽는다 ⇒ 게이트③ 차단 0건
```

★**2026-09-07 오전의 `check-worklog-coverage` red 와 «형태가 다르다»** — 그것은
`origin/main` 을 `fetch-depth: 0` 으로 읽어 **열린 PR 전건**을 함께 red 로 만들었다.
⇒ hard-fail 이 막는 것이 **없으므로** `continue-on-error` 가 사는 대가(= 초록 run 에 묻힌 주석,
「아무도 안 보면 없는 것과 같다」)를 지불할 이유가 없다.

**⒟ 넣지 않는다(기각) — 다만 «이미 적혀 있다»는 것은 사실이었다.** `AGENTS.md` 는
「Run it by hand against production after a deploy」를 **적어 두고 있었다**. 기각 사유는 둘이다:
⑴그 문장에 **주체도 시점도 없다** ⑵실제 집행은 이 저장소 «밖»(orchestrator 머지계약 4-C)에 있어
저장소 관점에서는 **보이지도 강제되지도 않는다**. ⇒ 제안이 겨눈 「사람 기억」이 정확히 그것이다.
★그리고 그 문장의 **논거가 낡았다**: 「가리킬 URL 이 없다」는 **PR 에만** 참인데 push 를 다루지
않아, 저 절이 push 경로를 **조용히 비껴갔다**.

### F3 — ⑶「실패하면 누가 무엇을」 · 회신 «밖»에 적었다

> **그 스텝이 red 면 «그 머지를 착지시킨 게이트③ 회차»가 받는다** — 그 회차는 머지계약 4-C 로
> 이미 같은 스크립트를 운영 주소에 대고 돌리고 있으므로, 실패한 스텝이 찍은 URL 로 다시 돌려
> 티켓을 내거나 「배포가 나쁘다」를 회신에 적는다.

위치 = **`AGENTS.md` §Web-surface commands** 의 `npm run verify` 항목(인용 블록).
★**소유자를 «그 자리에 이미 있는 역할»로 고른 것은 선례가 있다** — 같은 파일의
`check-worklog-coverage` 소유자 조항이 같은 논법이다(「크로싱 회차가 착지이고, 다음에 저장소를
만지는 것은 또 다른 게이트③이며, 그 red 가 막는 것도 그 역할이다」).

### 개악 대조 — 스텝을 지우면 무엇이 지는가

로컬 실행(외부 주소 무접촉):

| 대조 | 형상 | 결과 |
|---|---|---|
| **C1** | 대상이 응답하지 않음(`127.0.0.1:8799`) | ★**rc=1** (`TimeoutError`) |
| **C2** | 앱이 «아닌» 것이 배포됨(정적 HTML) | ★**rc=1** (`TimeoutError` — 셀렉터 부재) |
| **C3** | 실제 배포본(운영 URL · F1⑷) | **rc=0** |

★C2 가 `TimeoutError` 라는 것은 **브라우저가 실제로 떴고 페이지를 받았다**는 뜻이다 ⇒
번들 chromium 전환이 동작한다(실 Chrome 없이 돌았다). ★**둘 다 rc=1 이라 «대상 무응답»과
«잘못된 아티팩트»를 스크립트가 구별하지는 못한다** — 둘 다 red 이므로 판정에는 영향이 없다.
★번들 chromium 이 이 wasm 앱을 실제로 «완주»한다는 양성 증거는 별도로 있다 —
`contract-roundtrip.mjs` 가 CI 에서 매 엔진 PR 마다 같은 브라우저로 같은 앱을 부팅한다.

### CI 시간 증가분 — 수로

```
engine-contract.yml  "Contract check — browser boot round-trip"     27s · 28s   (2개 run)
   = npm ci --no-audit --no-fund  +  npx playwright install --with-deps chromium  +  스크립트
web.yml  job "build-web" 총                                          210s
   그중 Build frontend 150s · Deploy 9s · D1 migrations 9s
```

⇒ 새 스텝은 **같은 3종 + 10초 고정 대기**라 **≈30~40초** · job 총계 대비 **약 +15~19%**.
★**push 에서만** 붙는다(PR run 은 건너뛴다).

### 사용자 영향

배포될 때마다 「앱이 뜨고 파일을 받고 게임 바이트가 나가지 않는다」가 **사람 기억 없이** 확인된다.
★**나쁜 배포를 막지는 못한다** — 배포 «뒤»이고 그것이 설계다(막으려면 별 결정이다).

### 범위 밖 — 손대지 않은 것

`verify-browser.mjs` 의 **검사 내용 무접촉**(브라우저 선택과 머리주석만) · `web.yml` 의 **배포 스텝
로직 무접촉** · Scenario F·러너 블록 무접촉 · worklog 재측 기록 **0줄** · 실 Chrome 설치 **미도입**.
★**단 하나 예외적으로 배포 스텝에 `id: deploy` 를 붙였다** — 동작·게이트·입력 무변의 표식이고,
없으면 `deployment-url` 을 읽을 수 없어 **별칭 전환 지연을 추측**해야 한다(아무도 재지 않은 수다).
그 판단을 여기 적어 검수자가 판정할 수 있게 남긴다.
