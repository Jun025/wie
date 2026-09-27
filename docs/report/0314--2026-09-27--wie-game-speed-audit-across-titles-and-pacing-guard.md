## [2026-09-27] 게임 속도 전수 점검 29종 · GC 비용 비례 간격 · 페이싱 계측과 퇴행 가드 (wie-game-speed-audit-across-titles-and-pacing-guard)

**무엇을**: 두 번의 «조용한» 속도 퇴행(#291 KTF clet 35→17fps · #338 LGT 20→12fps) 뒤에, 다른 게임에도 같은 일이 있는지 브라우저로 29종을 쟀다.
그리고 엔진이 스스로 «게스트가 요청한 것과 받은 것 사이의 지연»을 세게 했다. 같은 퇴행이 다시 들어오면 `cargo test --all` 이 빨개지게 했다.
**왜**: 두 번 모두 운영자가 체감으로 먼저 찾았다. #291 은 upstream base 교체와 함께 들어왔다. 각 수정 옆의 단위 시험은 함수 하나씩만 잠근다.
**사용자 영향**: SKT 무거운 힙 타이틀 1종이 10.85 → 11.85fps가 됐다. 1초마다 165ms씩 멈추던 GC 는 3.3초마다 1번으로 줄었다. 나머지 타이틀은 코드 경로가 같다(교대 2짝 · 퇴행 0).

증적: `~/orchestrator/reports/evidence/wie-game-speed-audit-across-titles-and-pacing-guard/`.
하네스 · 원자료 `*.jsonl` · 표 생성기 · 변이 로그 `mut.log` · 퇴행 실증 `regress-demo.log` · 게이트 로그가 있다. 게임 바이트 0 · 제목 0 · sha↔플랫폼 표만 있다.
핀: `main` = 착수 base(`156c5c56`) + 계측(동작 무변경). `fix` = 이 PR 코드 + wasm 스크래치 export `take_pacing`(PR 에 없음).

### A. 전수 표 — 29종(부팅 실패 2종 별도)
측정: 브라우저(CDP · headless Chrome for Testing · `caffeinate -u -d`)에서 8초 워밍 뒤 15~20초 창을 잰다. OK 키는 2.5초마다 누른다. 계측 핀(엔진 무변경 + 계측) 기준이다.
- fps: 3ms 안에 합쳐진 blit 을 1프레임으로 센다.
- 엔진 ms/frame: 게임 스레드 경로에서 엔진이 더한 지연을 프레임당으로 나눈 값이다(`wie_backend::Pacing` 합계 ÷ 프레임).
  = sleep 늦깸 + WIPI 타이머 늦음 + GC(모든 task 가 멈춘다) + (yield 스핀일 때만) repaint→paint 지연.
- 비율 = 1 − 엔진 ms/frame ÷ 실제 프레임 주기 = 의도 주기 ÷ 실제 주기의 하한이다. 두 번 잰 것 중 작은 값을 적었다.
  마감 루프는 늦깸을 다음 sleep 에서 흡수하므로 이 값은 보수적이다.

선정: 라이브 LGT aot-java 등재 5종 + KTF clet 1종(#291 대상), 그리고 `game_lab/working/{ktf,skt}` 를 sha 순으로 정렬해 KTF 14번째마다 13종, SKT 4번째마다 12종을 골랐다(편향 없는 표본).
루프 방식은 계측값에서 읽었다: WIPI 타이머 · sleep 인자 분포(단일값 = 고정 sleep · 분포 = 마감 sleep) · 프레임당 yield > 20 = 스핀 · repaint 호출.

| sha12 | 플랫폼 | 구분 | 루프 | fps (회차) | 간격 p50 ms | 비율 | 엔진 ms/frame | 주원인 | ms/GC |
|---|---|---|---|---|---|---|---|---|---|
| 49ade89578c5 | ktf | live | clet timer | 35.9/35.5 | 16.9/17.5 | **0.678** | timer=8.9 | R | 0 |
| 13d7e3c21856 | lgt | live | deadline sleep(60..80) + repaint | 13.5/13.6 | 70.1/68.5 | 0.927 | sleep=5.1 gc=0.2 | | 3 |
| 1b107b96bf4e | lgt | live | fixed sleep(100) + repaint | 2.6/2.7 | 115.8/113.8 | 0.973 | sleep=7.0 gc=0.3 | | 4 |
| 4ece6eeeaa04 | lgt | live | fixed sleep(150) + repaint | 0.8/0.8 | 7.7/8.5 | 0.967 | sleep=30.2 gc=1.9 | | 4 |
| a30bbe008b5e | lgt | live | deadline sleep(10..95) + yield spin + repaint | 16.2/16.2 | 50.1/50.1 | **0.838** | sleep=6.5 redraw=1.3 gc=0.4 | R(흡수) | 6 |
| b475b6399684 | lgt | live | deadline sleep(59..100) + repaint | 9.6/9.9 | 58.6/58 | 0.912 | sleep=8.5 gc=0.2 | | 3 |
| 01e2715ba07a | ktf | pool | deadline sleep(155..163) + repaint | 1.6/1.6 | 166.3/166.7 | 0.981 | sleep=8.9 gc=0.9 | | 6 |
| 135d1291501f | ktf | pool | deadline sleep(30..50) + repaint | 24.3/25.4 | 19.6/15.5 | 0.920 | sleep=2.2 gc=0.2 | | 6 |
| 2c2ba3b84b98 | ktf | pool | deadline sleep(10..100) + repaint | 7.5/8.1 | 91.8/84.5 | 0.954 | sleep=3.7 gc=2.3 | | 22 |
| 3be2a19f5d3f | ktf | pool | fixed sleep(70) + repaint | 11.9/11.9 | 83.4/83.5 | **0.840** | sleep=12.7 gc=0.7 | R | 9 |
| 4fcd4b74020e | ktf | pool | clet timer | 14.3/14.3 | 67.3/67 | **0.899** | timer=7.0 | R | 0 |
| 60704ad61a47 | ktf | pool | deadline sleep(37..98) + repaint | 15.2/15.1 | 69.9/69.9 | 0.988 | sleep=0.3 gc=0.5 | | 8 |
| 7218e8720f8c | ktf | pool | deadline sleep(20..80) + repaint | 11.2/11.2 | 82.8/82.6 | **0.833** | sleep=13.4 gc=0.8 | R | 9 |
| 888702965551 | ktf | pool | deadline sleep(44..55) + repaint | 24.0/23.9 | 35.2/38.4 | 0.902 | sleep=3.7 gc=0.3 | | 6 |
| 9c1c446a36e2 | ktf | pool | deadline sleep(10..100) + repaint | 1.6/1.5 | 28.2/109.8 | 0.984 | sleep=3.7 gc=6.4 | | 22 |
| c654601e389f | ktf | pool | deadline sleep(88..96) + repaint | 17.6/18.6 | 86.9/82.8 | 0.988 | sleep=0.2 gc=0.3 | | 5 |
| e70c0c1c282c | ktf | pool | clet timer | 61.9/61.9 | 16.6/16.7 | **0.644** | timer=5.7 | R | 0 |
| fe184f834bc7 | ktf | pool | fixed sleep(100) + repaint | 9.5/9.2 | 100.2/113 | 0.917 | sleep=6.6 gc=0.3 | | 3 |
| 02bc66e85cf4 | skt | pool | fixed sleep(2000) + yield spin | 16.5/16.6 | 16.7/16.7 | 0.998 | gc=0.1 | | 2 |
| 1f0d7e81336a | skt | pool | deadline sleep(49..50) + repaint | 19.9/19.9 | 50/50 | 0.994 | sleep=0.2 gc=0.1 | | 1 |
| 31c90441f639 | skt | pool | yield spin | 11.2/11.1 | 83.1/83.3 | **0.834** | gc=14.3 | **G** | **165** |
| 3fa46bfa50d7 | skt | pool | fixed sleep(100) + repaint | 9.5/9.3 | 100.1/102 | 0.943 | sleep=5.2 gc=0.1 | | 1 |
| 4f99be2762e2 | skt | pool | fixed sleep(70) + yield spin + repaint | 2.0/1.9 | 66.7/67.9 | 0.978 | sleep=0.1 redraw=10.4 | | 0 |
| 69c4a48cc7f0 | skt | pool | deadline sleep(28..38) + repaint | 19.9/19.9 | 50/50.1 | 0.993 | sleep=0.2 | | 1 |
| 97506b6246e2 | skt | pool | deadline sleep(25..100) + repaint | 10.0/10.0 | 100/100 | 0.967 | sleep=2.4 gc=0.1 | | 1 |
| a0436eb1ddbb | skt | pool | deadline sleep(10..50) + repaint | 19.7/19.7 | 57.7/58.2 | 0.990 | sleep=0.4 gc=0.1 | | 1 |
| cf5249e75df3 | skt | pool | fixed sleep(450) + yield spin + repaint | 2.2/2.2 | 450/450 | 0.927 | sleep=0.2 redraw=1.3 gc=31.0 | G | 91 |
| d6abd6258d08 | skt | pool | fixed sleep(33) + yield spin | 1.0/1.0 | 1016.9/1029.9 | 0.989 | sleep=9.6 gc=1.0 | | 1 |
| ec2f8f2e02a2 | skt | pool | fixed sleep(100) + yield spin + repaint | 0.4/0.4 | 2500.3/2500.3 | 0.953 | sleep=63.2 redraw=15.8 gc=0.3 | | 0 |

부팅 실패(브라우저에서 blit 0 · 두 번 다): `d9afc4db742c`(ktf) · `bf54c05e58a9`(skt). 속도 표에서 뺐다.

**비율 < 0.9: 7종**(29종 중).
- **R(rAF 격자) — 6종 · 이미 알려진 계급 · `wie-2026-09-27-sleep-wake-exact-within-tick-budget-adopt-p0`(wie-3) 소관**.
  모든 task 가 잠들면 tick 이 끝나서, 깸과 타이머 발화가 다음 호스트 프레임(16.7ms 격자)까지 밀린다.
  - 고정 sleep(70) → 간격 83.4 = 5프레임(`3be2a19f5d3f`).
  - 마감 sleep → 늦깸 13.4ms/frame(`7218e8720f8c`). 이 둘은 실제로 느리다.
  - `a30bbe008b5e` 는 늦깸을 다음 sleep 이 흡수해 간격 p50 50.1 = 그 장면 격자 50이다. 느리지 않다.
  - clet timer 3종(`49ade89578c5` · `e70c0c1c282c` · `4fcd4b74020e`)은 타이머가 5.7~8.9ms 늦는다.
    그중 둘은 이미 호스트 프레임마다 1프레임(16.7ms)으로 돈다. 격자를 없애면 **원래 폰보다 빨라질 수 있다** — wie-3 회차에 이 점을 알린다.
- **G(GC 비용) — 새 계급 · 이 회차에서 고쳤다**: `31c90441f639` 는 GC 1번에 165ms 가 든다.
  #338 의 «1초에 1번»에서는 벽시계의 약 15%가 GC 로 멈춘다(모든 task 가 선다). `cf5249e75df3` 도 91ms/GC 다(비율 0.927 · 느린 루프라 덜 보인다).

관찰(고치지 않음 · 속도 아님): repaint 뒤 한 번만 yield 하고 sleep 하는 루프는 페인트가 다음 tick 에서 시작된다.
예: `13d7e3c21856` 은 repaint→paint 17ms · 271/271 이 tick 을 넘는다. 게임 스레드는 그동안 자고 있어 주기는 그대로다. 화면이 1프레임 늦게 보일 뿐이다.
#338 이 일부러 그렇게 둔 것이다(첫 yield 전달 → 퇴행 실측).

### G 수정 — GC 간격을 직전 수집 비용에 비례시킨다
`Display.handlePaintEvent`: 다음 GC 는 `max(1000ms, 직전 수집 비용 × 20)` 뒤에 돈다. 즉 GC 가 벽시계의 5%를 넘지 않는다.
비용이 약 50ms 이하면 1000ms 로 종전과 같다. 라이브 LGT 5종의 GC 는 3~7ms 라 **동작이 같다**.

교대 짝(`main` = 계측만 · `fix` = 이 PR 머리 · 20초 창):
| sha12 | 짝 | main fps (간격 p50 · GC 횟수@ms) | fix fps (간격 p50 · GC 횟수@ms) |
|---|---|---|---|
| 31c90441f639 (skt) | 1 | 10.85 (82.9 · 19@190) | **11.85** (82.8 · 6@160) |
| 31c90441f639 (skt) | 2 | 10.80 (82.7 · 19@194) | **11.85** (82.9 · 7@153) |
| cf5249e75df3 (skt) | 1 | 2.20 (450 · 15@115) | 2.20 (450 · 10@92) |
| cf5249e75df3 (skt) | 2 | 2.20 (450 · 14@100) | 2.20 (450 · 10@92) |

같은 변경의 후보 핀에서도 2짝을 쟀다: 10.80 → 11.85 · 10.85 → 11.90(`gcpairs.jsonl`).
주기 p50 은 83ms(5 호스트 프레임)로 그대로다 — 늘어난 fps 는 GC 로 멈춘 프레임이 줄어든 몫이다.
`cf5249e75df3` 은 450ms 고정 sleep 안에 GC 가 들어가 fps 가 변하지 않는다.

### 회귀 — 라이브 6종(교대 2짝 · fps · 간격 p50)
| sha12 | 짝 1 main → fix | 짝 2 main → fix |
|---|---|---|
| a30bbe008b5e (lgt) | 16.25 → 16.30 | 16.25 → 16.30 |
| 13d7e3c21856 (lgt) | 13.55 → 13.50 | 13.35 → 13.55 |
| 1b107b96bf4e (lgt) | 2.55 → 2.40 | 2.30 → 2.55 |
| 4ece6eeeaa04 (lgt) | 0.80 → 0.80 | 0.80 → 0.80 |
| b475b6399684 (lgt) | 9.80 → 9.80 | 7.00 → 6.90 |
| 49ade89578c5 (ktf clet) | 35.6 → 34.7 | 34.7 → 35.0 |

퇴행 0. 차이는 짝마다 방향이 바뀌는 잡음이다. 이 6종에서는 GC 비용이 50ms 미만이라 수정 경로가 main 과 같다. KTF clet 은 이 GC 를 거치지 않는다.
주입(`wie_validate --inject --boot-secs 15 --action-secs 1.2 --max-ticks 100000000000` · release · 2회): **12/12 PASS 27/27**.

### B-1. 엔진 내부 계측 — `wie_backend::Pacing`
엔진 시계(`Platform::now`)로 잰다. 부하가 흔드는 fps 절대값이 아니라 «게스트가 요청한 것과 받은 것 사이»를 잰다.
- guest `Thread.sleep(n)`: 인자 분포(p05/p50/p95) · 늦깸(p50/p95/합).
- WIPI 타이머: 발화 늦음. `Thread.yield` 수.
- `Display.repaint` → `handlePaintEvent` 시작 지연과 그중 호스트 tick 을 넘은 수(`redraw_cross_tick` — 호스트 속도와 무관한 수).
- 페인트 · GC 횟수와 ms.

`Emulator::take_pacing()` 이 창 단위로 꺼낸다. `wie_validate --pacing [SECS]` 은 결과 줄에 `"pacing":{…}` 를 붙인다(플래그가 없으면 줄이 종전과 같다).
wasm 노출은 하지 않았다 — 계약 표면이 바뀐다(Constraint 3). 브라우저 측정은 스크래치 export 로 했다(증적 `scratch-take-pacing-export.patch`).

### B-2. 스케줄러 의미 — 시험과 변이
| 의미 | 시험 |
|---|---|
| sleep(N) 은 N+ε 에 깬다 | `executor::tests::test_sleep_wakes_on_the_first_step_at_or_after_its_deadline`(신설) |
| repaint 는 같은 task 의 두 번째 yield 에 전달 | `display::test::repaint_reaches_the_event_queue_when_the_guest_spins_on_yield`(#338) |
| 빈 큐 대기는 다음 타이머까지 | `event_queue::test::timer_due_mid_slice_fires_without_waiting_for_the_whole_slice`(#291) |
| GC 는 페인트마다 돌지 않는다 | `display::test::paint_collects_garbage_at_most_once_per_interval`(#338) · `an_expensive_collection_spaces_out_the_next_one`(신설) |
| 전 구간: 마감 루프가 요청한 주기에 돈다 | `wie-j2me/tests/test_pacing.rs`(신설 · `test_data/pace_j2me.zip`) |

`test_pacing` 은 #338 의 루프(repaint → yield 스핀 → 50ms 마감 sleep)를 가진 J2ME 픽스처다.
`TestClock::stepping(1)` 시계(읽을 때마다 1ms)로 10초 창을 돌린다 ⇒ 수치가 호스트 부하와 무관하다.
이 트리의 값: 페인트 200/200(주기 50.0ms) · tick 넘김 0 · repaint→paint p95 4 · 늦깸 p95 0 · GC 10/200.

변이(`-p wie-backend -p wie-midp -p wie-j2me` · `--no-fail-fast` · 무변이 green):
| 변이 | 빨개지는 시험 |
|---|---|
| #291 tick 예산 14→8 | `test_pacing` + executor/frame_pacer 7건 |
| #291 빈 큐 대기 16ms 고정 | `timer_due_mid_slice…`(J2ME 에는 WIPI 타이머가 없어 `test_pacing` 은 못 본다) |
| #338 yield 스핀에 repaint 미전달 | `test_pacing` + display 3건 |
| #338 repaint 를 호스트 `request_redraw` 로(종전·upstream 모양) | `test_pacing` + display 2건 |
| #338 GC 매 페인트 | `test_pacing` + `paint_collects_garbage…` |
| #338 빈 큐 대기가 큐를 안 봄 | `test_pacing` + `event_queued_mid_wait…` |
| sleep 깸을 16ms 격자로 | `test_pacing` + `test_sleep_wakes…` + event_queue 2건 |
| GC 간격이 비용을 무시 | `an_expensive_collection…` |
| GC 비용 미기록 | `an_expensive_collection…` |
상수를 자기와 비교하는 시험은 없다.

### B-3. upstream 동기 경로
`test_pacing` 은 `cargo test --all` 안에서 돈다 ⇒ `rust.yml` 6다리가 **모든 PR 에서** 돌린다. upstream base 교체·병합 PR 도 여기에 들어간다. 새 워크플로·새 스텝은 없다(과금 0).
- 기준선은 시험 안의 상수다: 주기 비율 ≥ 0.9 · tick 넘김 ≤ 10% · repaint→paint p95 ≤ 8 · 늦깸 p95 ≤ 5 · GC ≤ 페인트의 10%. 게임 바이트는 0이다.
- 벗어나면 CI 는 그 줄의 문장과 JSON 전체를 낸다. 예: `{cross} of {redraws} paints started a host tick after their repaint — the engine is holding repaints to the end of a tick again (#338)`.
- 실증: 로컬 브랜치에 `Display.repaint` 를 호스트 `request_redraw` 로 되돌린 커밋(푸시 안 함 · 삭제)을 만들었다. 거기서 `RUST_MIN_STACK=4194304 cargo test --all` 이 `test_pacing` 에서 rc=101 이었다(증적 `regress-demo.log`).
- otterpebble 핀 범프: 엔진 퇴행은 wie PR 에서 이 시험에 먼저 걸리고, 그 다음에 산출물이 나간다 ⇒ otterpebble 쪽 별도 신호는 필요 없다고 판단했다(후속 제안 없음).

### 한계
- 비율은 하한이다. 마감 루프가 흡수한 늦깸도 뺐다.
- 부하 load1 200~370 에서 쟀다. fps 절대값은 흔들리고, 짝 비교와 엔진 내부 계측이 판정 근거다.
- GC 가 드물어진 만큼 쓰레기가 더 오래 산다(165ms 타이틀은 최대 3.3초). 할당 실패 시 재시도가 없는 것은 #338 과 같다.
- `test_pacing` 은 J2ME 경로다. KTF clet 의 타이머 경로는 wie-midp 단위 시험이 잠근다.

### 게이트(rebase 뒤 PR 머리 · 증적 `gates-final.log` · `runner.log`)
- fmt · clippy `-D warnings`(stable · wasm32 · beta) rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` **501 passed / 0 failed**.
- runner 블록 전부 PASS: draw_j2me · helloworld 2종 · text_j2me, keydraw_ktf/lgt `--inject --expect-last-frame` 27/27 rc=0.
- `npm run build:wasm` · `check-engine-contract`(109 pass · 위반 0) · `npm run audit` · `check-engine-runner-fixtures` rc=0.

### 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 결과는 BOUNDED 10쌍 · SUFFIX-ATTACHED 0 이다.
10쌍 전부 이 브랜치가 건드린 파일(executor · system · display · event_queue · 4 emulator · wie_validate)에 **이미 있던** 주석이다. 이 회차가 새로 쓴 이름은 0 이다 — 표는 sha12 로만 적었다. 게임 바이트는 0 이다.


<!-- corpus-name-inflow v1 subjects=19 tree=ef4848c60191f62b B=11/10 P=0/0 S=0/0 -->
