## [2026-09-07] 러너 목록 ↔ 픽스처 집합 드리프트 검사 (wie-engine-runner-list-vs-fixture-set-drift-check)

### 무엇을
「엔진을 만졌으면 이 러너를 돌려라」 목록(`AGENTS.md`)을 마커로 감싸고, `git ls-files test_data/` 와
**양방향으로** 대조하는 검사를 넣었다. 존재 대조(S)까지이고 **분류(M)는 하지 않았다**.

### 왜 — F1 실측

**⑴ 목록의 자리** = `AGENTS.md:117-125`(마커 삽입 전), 「Touching engine code?」 아래 `sh` 블록.
파리티 락은 이 블록을 **보지 않는다** — 그 `COMMIT-GATES:END` 마커가 스스로 「이 아래 `sh` 블록은
조건부 부록」이라고 적는다.

**⑵ 픽스처 집합** = `git ls-files test_data/` (추적 5건).

**⑶ ★현재 차 — 이 회차의 급소가 여기서 나왔다**

```
추적 픽스처(5): draw_j2me.zip helloworld_ktf.zip helloworld_lgt.zip keydraw_ktf.zip keydraw_lgt.zip
블록 호명 (5): draw_j2me.jar helloworld_ktf.zip helloworld_lgt.zip keydraw_ktf.zip keydraw_lgt.zip

★계수 5 : 5  ⇒ «같다»
경로 기준 — 블록에만: test_data/draw_j2me.jar     (1건)
경로 기준 — 픽스처에만: test_data/draw_j2me.zip    (1건)
stem 기준 — 양방향 0 / 0
```

★**티켓이 경고한 그 형태가 «이 회차의 첫 측정»에서 그대로 나왔다** — 계수는 같은데 집합이 다르다.
★**그리고 그 차는 «드리프트가 아니다»**: 블록의 **첫 줄**이 `scripts/make-draw-fixture.mjs` 로
`.jar` 를 만들고, `.gitignore:24 *.jar` 라 그 산출물은 커밋되지 않는다 — 같은 픽스처의 두 확장자다.

⇒ ★**술어를 «stem 집합 동등»으로 골랐다.** 부분문자열이 아니라 집합 비교라 `draw_j2me_v2.zip` 은
`draw_j2me` 와 **매치되지 않는다**. ⇒ ★**지금 차는 0 이고, 검사는 조용하다.** 해소할 것도 남길 것도 없었다.

### 두 번째 진실원을 만들지 않는 방법 — 면제는 «문서 안»에 쓴다

제안이 경고한 대가 ⑴은 「검사기가 분류를 알아야 하고 그것이 두 번째 진실원이 된다」였다.
⇒ 검사기는 분류를 **모른다**. 러너가 일부러 안 도는 픽스처는 마커 구간 안에
`NOT-RUN: test_data/<name> — <why>` 로 적는다. **오늘 해당 0건**이라 구문만 문서화했다.

### 양방향 실증 — 넷 다 돌렸다

| 심은 것 | 결과 |
|---|---|
| ⑴ 픽스처를 «늘림»(`keydraw_skt.zip`) | ★**rc=1** — 그 파일을 이름으로 지목 |
| ⑵ 목록에서 «뺌»(`keydraw_lgt.zip`) | ★**rc=1** — 그 파일을 이름으로 지목 |
| ⑶ `END` 마커 제거 | ★**rc=1** — 빈 구간을 `OK` 로 읽지 않는다 |
| ⑷ 늘린 뒤 `NOT-RUN` 면제 | ★**rc=0** · `excused` 로 분류(★`named` 에 섞이지 않는다) |

★**⑶ 을 넣은 이유**: 「검사기가 자기 입력을 못 찾으면 조용히 통과한다」가 이 계급의 가장 흔한
무증상 형태다. ★심은 것은 **전건 원복**했고 `git status --porcelain` 에는 계획된 변경만 남았다.

### F3 — 어디서 돌리는가, 그리고 ★**승격하지 않았다**

**`engine-contract.yml` 의 `contract` job** — 형제 문서 검사기 4종(`check-worklog-json`·
`check-worklog-coverage`·`check-docs-report-serial`·`npm run audit`)이 전부 거기 있고, node 1초짜리라
`cargo test` 에 넣으면 매트릭스 **6배**로 헛돈다(같은 논거를 `--expect-last-frame` 결정이 이미 썼다).

★★**그런데 그 job 은 `main` 의 required 검사다** — 그래서 hard-fail 로 넣으면 **배치만으로 승격**이 되고,
그것은 이 회차가 정하지 말라고 지시받은 바로 그것이다. ⇒ **`continue-on-error: true`** 로 붙였다.
★스텝은 여전히 `::error::` 를 찍고 run 에서 **실패로 보인다** — 다만 job 을 떨어뜨리지 않는다.
★**승격 = 그 한 줄을 지우는 것**이고(브랜치 보호 무접촉), 조건은 「실제 픽스처 변경에서 한 번이라도
울었고 그 메시지가 옳았다」이다. 워크플로 주석에 적었다.
★**대가를 숨기지 않는다**: 그때까지 이 검사는 **아무도 막지 않고**, 아무도 run 을 안 열면 안 보인다.

### 사용자 영향

「문서가 시키는 대로 돌렸는데 새 픽스처를 놓쳤다」가 **PR 시점에 말해진다**(지금은 주석으로).
`keydraw_*` 두 픽스처가 2026-09-06 에 커밋돼 있는데 목록에는 0건이던 것이 이 결함의 원형이다.

### 범위 밖 — 손대지 않은 것

러너 자체 **무접촉**(줄 하나 안 늘렸다) · 픽스처 **정리 0**(추가·삭제·이름변경 0) ·
required 승격 **안 함** · `.github/workflows` 다른 스텝 순서 **무변**(diff = **삽입 17줄 · 삭제 0줄**).
