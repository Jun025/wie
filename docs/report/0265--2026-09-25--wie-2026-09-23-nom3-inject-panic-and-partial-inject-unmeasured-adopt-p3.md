## [2026-09-25] 놈3 «한 프레임도 안 그리는» 판 — 저부하에서도 남는다 · 멈춤 자리 두 갈래 (wie-2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured-adopt-p3)

**무엇을**: 제안 `2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured#p3` — #264 가 대조군으로 «원래 있던 결함»이라 판정한
`no frame rendered`(paints 0 · input 27/27)를 부하와 갈랐다. **코드 변경 0.** 측정 + 멈춤 자리 판정 + 후속 제안.

**왜**: 그 제안의 tradeoff 가 「부하 의존이면 고칠 것은 엔진이 아니라 측정 조건 — 저부하에서 안 남으면 닫아라」였다.

**사용자 영향**: 없음(측정 회차). 판정은 «닫지 않는다» — 저부하에서도 나고, 부하와 무관한 멈춤 모양이 하나 잡혔다.

### 측정 조건

- 바이너리: `origin/main` `c496fe6c` release `wie_validate`(빌드 85분 · 고부하) — 측정 중 덮이지 않게 사본으로 고정.
- 타이틀: `game_lab/broken/lgt/놈3.zip` — sha256 `b475b639…`(라이브 배포 번들과 같은 값 · #264 인용값과 일치).
- 명령(한 번에 하나씩 순차 · 병렬 0): `wie_validate --inject <놈3.zip>` — RUST_LOG 없음.
- load1: 실행 **직전과 직후**를 재고 **큰 값**으로 분류했다(티켓의 「≥40 무효」를 보수적으로 적용).

### ⑴ 부하 구간별 재현율

| 구간(max load1) | n | no-frame | 비율 | 비고 |
|---|---|---|---|---|
| **< 40** | **40** | **1** | 2.5% | 그 1건은 load1 **26.0 → 29.0** 에서 났다 |
| 40–80 | 37 | 4 | 10.8% | |
| ≥ 80 | 30 | 1 | 3.3% | load1 최대 138 |
| 합 | 107 | 6 | 5.6% | FAIL 6건 전부 `paints 0` · `input_steps 27/27` · `stop deadline` |

부트 축만 따로(RUST_LOG=`wie_core_arm::core=info` — 스레드 생성·종료 줄만) **48회 · load1 17–52 · 0/48**
(그중 load1 < 40 이 **43**). ⇒ 저부하 합산 ★**1/83 = 1.2%**, 40 이상 **5/72 = 6.9%**.

- ★**부하와 무관하다고도, 부하 탓이라고도 말하지 않는다.** 한쪽 Fisher p = **0.075** — 기울기는 부하 쪽이지만 이 n 으로는 가르지 못한다.
  단조도 아니다(≥80 이 40–80 보다 낮다).
- ★**그러나 «저부하에서 안 남는다»는 반증됐다** — load1 26–29 에서 1건 났다. 제안의 닫는 조건이 서지 않으므로 **닫지 않았다.**

### ⑵ 멈춤 자리 — FAIL 2건을 debug 로그로 잡았다(9회 중 2건)

`RUST_LOG=debug` 로 돌려 FAIL 둘을 얻었다. ★**모양이 서로 다르다.**

| | FAIL A (debug 실행 #1) | FAIL B (debug 실행 #9) | PASS 대조(#2 · #3) |
|---|---|---|---|
| load1 | 72 | 53 | 46–72 |
| thread 1(`start`) 종료 | ★**끝나지 못함** — 21.7s 까지 부트 중(14,001/14,877 줄) | 3.12s | 8.59s · 11.45s |
| thread 3·4 생성 | 없음 | 3.12s | 있음 |
| ticks | 48 | 973 | 19,714 · 12,586,098 |
| 마지막 활동 | thread 1 이 `JarEntry::<init>` 에서 **여전히 일하는 중** | thread 4 = 3.13s · thread 3 = 3.54s 부터 **무음** · thread 2 의 `Thread.sleep(100)` 만 20.0s 까지 | 전 스레드 끝까지 |

- **A = 부트가 마감 안에 끝나지 않았다.** 부트는 매번 **같은 일**(14,877 줄)인데 끝나는 시각이 3.1s·8.6s·11.5s·>21.7s 로 흩어진다.
  ★**그러나 그 흩어짐의 상당 부분은 debug 로깅 자신이다** — 로그를 스레드 사건만으로 줄인 48회에서는 부트 종료가
  **0.24–3.20s** 이고 FAIL 0 이다. ⇒ A 는 «측정 조건»(마감 대 부트 시간) 쪽 모양이고 이 표본의 A 는 내 계측이 키웠다.
- ★**B = 부트 뒤 guest `paint` 안에서 멈춘다** — 마감이 16.5s 남았는데 두 그리기 스레드가 네이티브 호출을 한 번도 하지 않는다.
  순서(debug #9):
  1. 3.12s thread 1 종료 → thread 3·4 생성.
  2. **3.13s thread 4**(이벤트 그리기)가 `Display::handlePaintEvent` → `Canvas::handlePaintEvent` → `net.wie.CardCanvas::paint`
     → `Graphics::reset` → `Graphics::translate(0,0)` 까지 네이티브를 거친 뒤 **guest(AOT ARM) `Card.paint` 로 들어가 돌아오지 않는다.**
  3. 그동안 thread 3 은 0.42s 동안 자기 일을 하고(NPE `byte array is null` 1회 — PASS 판에도 똑같이 있다) **3.54s** 에
     `serviceRepaints` → 같은 `handlePaintEvent` 로 **같은 `javax Graphics` `0x488455e0`** 에 들어가 똑같이 `translate` 뒤 무음이 된다.
  4. thread 2 는 끝까지 100ms 마다 깬다 ⇒ **실행기(executor)는 살아 있다** — «부하로 굶었다»가 아니다.
- ★**B 에서 thread 4 가 먼저, 혼자 멈췄다** — thread 3 이 들어오기 0.42s 전이다. ⇒ «두 스레드가 동시에 그려서»는
  멈춤의 **방아쇠가 아니다**(동시 진입은 그 뒤 일이다). 다만 `Display::handlePaintEvent`(`wie-midp/.../display.rs:802`)에
  상호배제가 없어 두 스레드가 한 `screenGraphics` 를 함께 쓰는 것은 사실이다 — 원인으로 적지 않고 사실로만 남긴다.
- ★**B 의 guest PC 는 못 쟀다.** `wie_validate` 는 `profile: None` 을 박아 두었고(`wie_validate.rs:1129`) 프로파일러
  (`ArmCore::sample_profile`)는 GUI `wie` 의 `--profile-out` 으로만 열린다. 여는 것은 코드 변경 + 릴리스 재빌드(오늘 85분)라
  이 회차의 계약(「멈춤 자리 1회 판정까지」)을 넘는다 — 후속 제안으로 넘겼다.

### ⑶ 판정

- ★**엔진 결함(B)이 있다 — 닫지 않는다.** 저부하 1/83 이 남고, B 는 마감이 넉넉한데 멈추는 모양이라 부하·예산으로 설명되지 않는다.
- A(부트 초과)는 측정 조건 쪽 모양이다 — 단 계측 없는 저부하 FAIL 1건이 A 인지 B 인지는 **모른다**(로그가 없는 판이었다).
- ★`ticks` 는 판정에 쓰지 않았다(PASS 끼리 727 ~ 38,893,001). 표에는 모양 설명용으로만 적었다.

### 한계

- 저부하 FAIL 은 1건뿐이고 그 판의 모양은 미상이다. B 는 load1 53 에서 잡혔다.
- 저부하에서 FAIL 을 debug 로 잡으려면 ~1% 재현율이라 수십~백 회가 든다 — 하지 않았다.
- 이 머신의 load1 은 같은 시간 안에 17 ↔ 139 로 흔들린다. 구간 표는 «그 판의 앞뒤 1분 평균»이지 판 내내의 값이 아니다.

### 검증

- 회귀: `RUST_MIN_STACK=4194304 cargo test --all` · `cargo clippy --workspace --all-targets` — 결과는 회신에 적는다(코드 변경 0).
- 증거(저장소 밖 · 게임 바이트 0): `~/orchestrator/reports/evidence/wie-2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured-adopt-p3/`
  — `runs.tsv`(107회 원자료) · `cls-48.txt` · `debug-thread-timeline.txt` · FAIL A/B·PASS debug 로그(gz).

게임 이름 유입(`scripts/corpus-name-inflow.mjs`): BOUNDED 7회/2쌍 — 전부 이 회차의 대상 타이틀 `놈3`(리니지 0229 가 이미 이름을 적은 라이브 타이틀) · SUFFIX-ATTACHED 0.
