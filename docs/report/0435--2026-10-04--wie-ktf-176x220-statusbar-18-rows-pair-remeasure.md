## [2026-10-04] KTF 176×220 상태 표시줄 18행 — 폭별 한 표로 확장 (wie-ktf-176x220-statusbar-18-rows-pair-remeasure)

**무엇을**: #464 의 표시줄 모델(`AnnunciatorComponent.show()` 뒤 Card 는 표시줄 아래 · `Display.getHeight()` 는 남은 높이)을 176폭에 18행으로 넓혔다. 상수는 `STRIP_HEIGHTS` 한 표(폭 → 행)다 — 240 → 24 · 176 → 18 · 그 밖 0.
**왜**: 176폭 전폭 그림의 최대 높이 최빈값이 202(= 220 − 18 · 7종)이고 220 은 4종이다(0431 §1). 240폭과 같은 형태다.
**사용자 영향**: 표시줄을 보이는 작은 화면 KTF 게임은 화면이 18행 아래로 내려앉는다. 검은 화면만 보이던 한 타이틀(`8b899f410f5d`)이 그림을 그린다. 짝 전수에서 나빠진 타이틀은 0 이다.

타이틀은 sha12 로만 적는다.

### 1. 짝 전수 — 나빠짐 0

`scripts/playability-census.mjs run --only probe`(probe A 27키 · B 키 없음 · 각 30초 · `--jobs` 기본 3 · census 락 · 각 쪽 착수 전 `host-load-guard --status --recovered` rc=0 대기 · `build-slot` 경유). base = `origin/main` `1d015ed0` 릴리스 빌드, 후 = 같은 커밋 + 모델. 집합 = `game_lab/{working,broken}/ktf` sha256 중복 제거 후 `__adf__` 의 `DisplaySize:176*220` **95종**(티켓의 93 이후 코퍼스가 늘었다). 그중 jar 안에 `AnnunciatorComponent` 참조가 있는 것은 71종이다.

| 축 | base | 후 |
|---|---|---|
| status | limited 93 · not-yet 2 | limited 94 · not-yet 1 |
| boot | ok 94 · fail 1 | 같음 |
| render | ok 93 · none 1 · uniform 1 | ok 94 · none 1 |
| input | ok 88 · none 5 · n/a 2 | ok 89 · none 5 · n/a 1 |
| sound | ok 80 · silent 14 · n/a 1 | 같음 |

측정: base 15:16~15:33 · 후 15:33~15:49 (load1 10~26).

- **나빠짐 0.**
- `8b899f410f5d`: status not-yet → limited, render uniform → ok. input n/a → none 은 나빠짐이 아니다 — render 가 uniform 이면 input 은 재지 않고(n/a), 그림이 생겨 처음 잰 값이다. 두 타이틀 재측(각 빌드 2회): base 2/2 uniform · 후 2/2 ok ⇒ 재현된다.
- `1cdea1985955`: input none → ok 는 잡음이다. 재측 4회(각 빌드 2회) 모두 none 이었다. 고침으로 주장하지 않는다.
- 재측 2타이틀은 host gate rc=1 상태에서 돌았다(동시 2 프로세스). 판정이 양쪽에서 같게 반복돼 부하 탓으로 읽지 않았다.

### 2. 구현

- `wie-wipi-java/.../lwc/annunciator_component.rs`: `STRIP_HEIGHT_240` → `STRIP_HEIGHTS: [(240, 24), (176, 18)]`. `show()` 는 그 표에서 폭을 찾고, 없으면 0.
- 시험: 240폭 시험을 `shown_strip_moves_new_cards_below_it(w, h, rows)` 로 묶고 176×220·18 을 더했다(show 전 `[0, 176, 220]` · 후 높이 202 · `[18, 176, 202]`).
- 240폭 동작 변경 0(같은 값 24).

### 3. 한계

- 18 은 최빈값이다(176폭 202 = 7종 · 220 = 4종). 단말별 실제 높이는 가릴 근거가 없다.
- 표시줄은 그리지 않는다 — 위 18행은 비어 있다. 전폭 220 그림을 쓰는 타이틀이 `show()` 를 부르면 아래 18행이 잘린다. 프로브 축은 그대로였다.
- longplay·progress 축은 재지 않았다(`--only probe`).

### 4. 유입

- `node scripts/corpus-name-inflow.mjs` 결과: BOUNDED 1건, SUFFIX-ATTACHED 0건.
- 그 1건은 `shell_card.rs` 에 **main 에 이미 있던** 한 줄이다(이 PR 의 diff 에 0회). 파일을 고친 탓에 대상에 잡혔다(0431 §6 과 같은 줄).
- ⇒ 이 회차가 들인 게임 파일명은 **0건**이다.

