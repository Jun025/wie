## [2026-09-18] 글자를 그리는 픽스처를 만들어 러너에 얹었다 — 「고쳤다」를 증명할 자가 생겼다 (wie-game-lab-add-a-text-drawing-fixture-and-wire-it-to-ci)

**무엇을**: `test_data/text_j2me.zip`(신설 · 생성물은 `text_j2me.jar`) + `scripts/make-draw-fixture.mjs` 파라미터화 +
AGENTS.md 러너 블록 **1줄** + `doc-liveness.yml` DOC-COPY **1줄**.
★**제품 Rust 0줄** · ★**`wie_validate.rs` 무접촉**(되돌림은 «일시 개악»이고 커밋 0 · 바이트 동일 확인) ·
★**새 워크플로 0** · ★**`game_lab/` 무접촉** · ★**`STATE.md` 무접촉**(아래 사유).

**왜**: `game_lab/broken/` 187 서명 표가 착지했고(머지 `01e3ff1a`) 다음은 수리 캠페인인데,
★**「고쳤다」를 증명할 자가 없었다.** `wie_validate` 의 `HeadlessPlatform::font()` 는 두 달간
`unimplemented!()` 였고 — 게스트가 문자열을 그리면 **게임이 아니라 검증기가 패닉**했다 —
그동안 러너 블록은 **내내 green** 이었다. 잡은 것은 `.gitignore` 된 `game_lab/` 코퍼스뿐이고 그것은
**Constraint 9 때문에 CI 에 영원히 못 들어간다.**

**사용자 영향**: 없다(제품 0줄). 러너 블록이 **약 5.1초** 길어진다.

### ⓐ 전제 재현 — ★**결손은 실재한다**

`font()` 를 `192cd17a^` 로 되돌린 트리에서 **기존 러너 5종이 전건 PASS·rc=0** 이다
(`draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`keydraw_ktf`·`keydraw_lgt`).
⇒ ★**이 저장소에는 그 버그를 무는 것이 하나도 없었다.** 근인도 수로 확인했다 —
`drawString` 이 **커밋된 픽스처 6개 중 0개 · 생성기 3종 중 0개**에 있다.

### ⑴ 픽스처가 «무는가» — ★**red/green 쌍**

| 트리 | 기존 5종 | ★`text_j2me.jar` |
|---|---|---|
| `font()` 되돌림 | **전건 PASS · rc=0** | ★**FAIL · rc=1 · `content=false`**(`wie_validate.rs:359` 에서 패닉) |
| 정상 | 전건 PASS · rc=0 | ★**PASS · rc=0 · `content=true`** |

⇒ ★**새 픽스처가 «유일하게» 그 결손을 문다** — 그리고 ★**남의 축을 깨지 않았다**(기존 5종이 양쪽에서 불변).
★개악 트리는 `trap` 으로 자동 원복하게 만들었고, 끝난 뒤 `git diff --exit-code` 로 **바이트 동일**을 확인했다.

### ⑵ 배선 — ★**새 워크플로 0 · 있는 자리에 얹었다**

AGENTS.md 러너 블록에 **한 줄**, `doc-liveness.yml` 의 `DOC-COPY` 에 **같은 한 줄**
(`check-doc-liveness-parity` 가 두 쪽을 집합 대조한다 — 실측 `25 → 26 documented line(s)` · rc=0).
`check-engine-runner-fixtures` 도 rc=0(★그 검사기가 먼저 **red 로 막았다** — 픽스처를 추적하기 «전»에
「named but not committed」라고 정확히 말했다).

★★**그러나 «어디서 도는가»를 정확히 적는다 — 「CI 에 배선했다」를 «PR 마다 돈다»로 읽지 마라**:
| 경로 | 이 픽스처를 도는가 |
|---|---|
| **로컬 러너**(AGENTS.md — 엔진 코드를 만진 회차가 반드시 돌린다) | ★**돈다** |
| `doc-liveness.yml`(**주간** + 이 PR 의 self-test) | ★**돈다** |
| **PR 마다**(`engine-contract`·`rust.yml`·`coverage`) | ★**안 돈다** |

★**왜 PR 마다가 아닌가**: `.github/` 의 **어느 워크플로도 `wie_validate` 를 부르지 않는다**(실측 0건).
per-PR 로 올리려면 node 전용 `contract` 잡에 **cargo 빌드**를 새로 넣거나 `rust.yml` 6다리에
**중복 6회**를 태워야 하고, ★**AGENTS.md 가 2026-09-07 에 정확히 그 비용으로 `--expect-last-frame` 승격을 기각했다.**
⇒ 이 회차는 그 결정을 **뒤집지 않았다**. 탐지 지연은 **로컬 러너(즉시) ~ 주간**이다.
※per-PR 층이 필요하다면 더 싼 형태가 따로 있다 — `wie_validate.rs` 안 `#[cfg(test)]` 로 `HeadlessPlatform::font()` 를
한 번 부르면 `cargo test --all` 이 6다리에서 잡는다. ★**그것은 «그 한 줄»만 지키고 «텍스트 경로 전체»는 못 지킨다**
⇒ 대체가 아니라 층이고, 이 티켓의 범위 밖이다.

### ⑶ 수행 시간 — ★**5.1초**(기본값이면 21초였다)

| `--timeout` | 결과 | 벽시계 |
|---|---|---|
| 1 | ★**1/3 FAIL**(flaky) | — |
| 2 | 3/3 PASS | — |
| 3 | 3/3 PASS | 3.1s |
| ★**5(채택)** | **3/3 PASS** | ★**5.04~5.19s** |
| 기본(20) | 3/3 PASS | **20.8~21.7s** |

⇒ ★**예산이지 추측이 아니다** — paint 는 **약 1초**에 앉고(1초에서 깨진다), 채택값 5는 그 **약 5배**다.
★**전부 loadavg 123~125 에서 쟀다**(한산한 기계가 아니다). ⇒ **4배 절약 · 여유 5배.**

### ★설계 판정 — 왜 «둘째 jar» 인가(측정이 정했다)

처음엔 `draw_j2me` 의 `paint()` 에 `drawString` 을 얹었다. ★**그 순간 픽스처의 픽셀 통계가 움직였다** —
`distinct_colors` **2 → 3** · `nondominant_pct` **1.5 → 1.7**. 배경색으로 그려도 잉크가 붙는다.
그런데 `contract-roundtrip.mjs` 는 이 생성기의 **export 에서 유도한 «정확한» 비-검정 픽셀 수**를 단언한다
(`BASE_RECT_PX`·`IMG_RECT_PX`·`keyBarPixels`) — 글자가 더하는 양은 **글리프 래스터화에 달려 있어 생성기가 알 수도,
export 할 수도 없다.** ⇒ ★**둘째 jar 로 갈랐고, `drawFixtureJar()` 가 «바이트 동일»임을 md5 로 확인했다**
(`0bc2f484…` ↔ HEAD `0bc2f484…`). ★상수풀 항목도 `withText` 일 때만 넣는다 — 무조건 넣었더니
md5 가 움직였다(`afdc1f22…`). ⇒ **라운드트립 계약 무접촉.**
★**그리고 글자가 «진짜로» 그려진다**: `text_j2me` 의 `distinct_colors` = **24**(안티에일리어싱된 글리프 가장자리) ↔
`draw_j2me` = **2**. 「호출만 하고 아무것도 안 그린다」가 아니다.

### 한계 — 숨기지 않는다

- ★**단언은 «텍스트 경로를 지나 호스트가 살아남았다 + 뭔가 칠해졌다»(`content: true`)이지 «글자가 읽힌다»가 아니다.**
  글리프 모양을 재려면 폰트 래스터화에 CI 를 묶어야 하고, 그것은 6다리 매트릭스에서 흔들린다.
- ★**per-PR 이 아니다**(위 표) — 탐지 지연 최대 1주. 즉시 잡는 것은 **로컬 러너**이고, 엔진을 만지는 회차는
  AGENTS.md 가 그것을 **반드시 돌리라**고 이미 요구한다.
- ★**가드가 조용히 무장해제될 수 있는 경로 하나**: 렌더러가 «같은 색 draw» 를 건너뛰게 바뀌면 이 픽스처는
  통과하면서 아무것도 안 잡는다. ⇒ ★**렌더러가 바뀌면 위 red/green 쌍을 다시 돌려라** — 그것만이 무장 여부를 말한다.
- ★**`STATE.md` 를 만지지 않았다**(티켓 계약 2). 지금 `wie` 는 열린 PR 대부분이 `STATE.md` 에서 CONFLICTING 이고
  (소관 `wie-state-md-insertion-point-conflicts-every-open-pr`), 만지면 이 착지가 형제들을 다시 무효화한다.
  ⇒ ★**관례상 §완료 항목이 빠진다** — 그 대가를 알고 뺐고, 이 파일과 worklog 가 기록을 진다.
- ★**수리 캠페인은 시작하지 않았다**(계약 3) — 187건 중 **0건**을 고쳤다. 이 회차는 «증명할 자»를 만드는 것까지다.
