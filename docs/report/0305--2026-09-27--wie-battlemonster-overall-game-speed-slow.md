## [2026-09-27] 배틀몬스터 게임 속도 — 숲 장면 14 → 20fps(목표 20) · repaint 전달 · 이벤트 대기 · GC 주기 (wie-battlemonster-overall-game-speed-slow)

**무엇을**: 운영자 실플레이 보고 「그림만이 아니라 전체 구동이 느리다」(배틀몬스터 · LGT aot-java)를 재고 고쳤다.
**왜**: 게임 루프가 목표 주기(50ms)를 못 지켜 게임 시간 전체가 느려진다. 게임은 프레임 수로 진행하므로 fps 가 곧 게임 속도다.
**사용자 영향**: 숲·마을 대화 장면이 14fps → 20fps(게임이 의도한 속도). 전투는 16.3 → 16.65fps(그 장면의 목표 ≈16.7).

### 게임이 원하는 속도 — 장면마다 다르다(측정)
게임 스레드 루프(네이티브 trace · `Thread.sleep`/`Card.repaint` 기록):
`repaint → Thread.yield() 스핀(페인트가 끝날 때까지, 프레임당 약 2,600~4,000회) → currentTimeMillis → sleep(마감 − now)`.
- **마감(deadline) 방식**이다: 타이틀에서 repaint 가 정확히 50.0ms 격자에 놓인다. 일이 주기를 넘으면 sleep 이 바닥값(10ms)에 붙는다.
- 장면별 주기(브라우저 · 수정본에서 sleep 이 바닥에 안 붙는 값): 타이틀 50ms · 메뉴 100ms · **숲/대화 50ms(20fps)** · 전투 약 60ms(16.7fps).
- ⇒ 「느린 장면」 판정은 **sleep 이 바닥에 붙는가**로 한다. main 은 숲·전투에서 바닥(10)에 붙고, 메뉴는 여유(70)가 있다.

### 시간이 어디로 가나 — 숲 장면 1프레임(브라우저 · main · p50)
| 구간 | ms |
|---|---|
| sleep(10) → 실제 깸 | 10.8 |
| repaint → 페인트 시작 | **16.6** — 호스트가 Redraw 를 tick 끝에만 넘긴다(한 프레임 늦음) |
| 페인트(게스트 paint + blit) | 30.0 |
| 페인트 뒤 전체 GC | **9.8** — `Display::handlePaintEvent` 가 매 페인트 mark-and-sweep(업스트림 `95555afe`) |
| **주기** | **67.3**(목표 50) |

게임 자체 몫(sleep 바닥 10 + 페인트 30 = 40)이 남기는 여유는 10ms 뿐인데, 호스트 지연(16.6)과 GC(9.8)가 각각 그보다 크다.
wasm CPU 프로파일(전투 · 10s): GC 가 벽시계의 20.9%(바쁜 시간의 약 30%) · ARM 해석은 약 6~7% — «인터프리터가 느리다»는 원인이 아니다.

### 변형별 분해 (숲 · 같은 창 · 1회씩)
| 빌드 | fps | 주기 p50 | repaint→paint | GC |
|---|---|---|---|---|
| main | 13.9 | 67.3 | 16.6 | 9.8 |
| GC 1초 1회만 | 14.9 | 66.7 | 16.5 | 0 |
| repaint 를 즉시 큐에(GC 매번) | 19.15 | 50.2 | 0.1 | 9.8 — sleep 이 바닥(여유 0) |
| repaint 즉시 + GC 1초 | 19.93 | 50.0 | 0.1 | 0 — sleep 17(여유) |

### 수정 — 3개 (`wie-backend` · `wie-jvm-support` · `wie-midp`)
1. **repaint 전달**: `Display.repaint` 는 호스트 `request_redraw` 대신 `System::request_redraw` 로 표시만 한다.
   Redraw 는 ⒜ 같은 스레드가 sleep 없이 **두 번째** `Thread.yield` 를 하면(= 페인트를 기다리는 스핀) 즉시 큐에,
   ⒝ 아니면 tick 끝에(종전 호스트 전달 시점) 들어간다. 스트릭은 실행기 task 별이다.
2. **이벤트 대기**: `getNextEvent` 의 빈 큐 대기(최대 16ms)가 1ms 마다 백엔드 큐를 다시 본다. 콜백·타이머는 종전대로 대기당 1회.
3. **GC 주기**: `handlePaintEvent` 의 `collect_garbage` 를 1000ms 에 한 번으로(`GC_INTERVAL_MS`).

#### 왜 «repaint 즉시 큐에»가 아닌가 — 두 번 시도해 두 번 퇴행했다
- **즉시 push**: 메이플스토리2007 13.65 → 12.6(-8%), 놈3 스플래시에서 정지 3/3.
  메이플은 `repaint → yield → sleep(60)` 인데 즉시 시작된 페인트가 ARM 선점으로 sleep 호출 앞에 끼어 sleep 이 약 5ms 늦고
  rAF 한 칸을 넘긴다(4 → 5프레임). 놈3 은 `serviceRepaints` 동기 페인트와 이벤트 스레드 페인트가 0.5ms 차로 같이 들어가
  둘 다 끝나지 않는다(0296 의 게스트 모니터 교착과 같은 모양 · main 에서는 12~35ms 떨어져 들어간다).
- **첫 yield 에 전달**: 메이플이 다시 12.75 — 그 한 번의 yield 가 바로 메이플의 «페인트 양보»다. ⇒ 스핀(두 번째 yield)만 본다.
- **스트릭을 전역으로**: 배틀몬스터가 다시 14.9 — 다른 스레드의 sleep 이 게임 스레드 yield 사이에 끼어 매번 리셋. ⇒ task 별.
- **이벤트 대기 1ms 확인이 왜 필요한가**: 전달 뒤에도 이벤트 스레드가 16ms 대기 중이면 16.4ms 뒤에 줍는다(측정).
  첫 yield 전달(fix2)은 우연히 한 step 앞서 떨어져 이 대기를 피했을 뿐이다.

### 목표 · 전 · 후 — 숲 장면(교대 3짝 · 40초 창 · 브라우저 CDP · `Chrome for Testing` headless)
| 짝 | main fps | 수정 fps | main 간격 p50/p95 | 수정 간격 p50/p95 | load1 (main/수정) |
|---|---|---|---|---|---|
| 1 | 14.25 | **19.98** | 67.1 / 83.1 | 50.1 / 50.9 | 16.8 / 8.6 |
| 2 | 13.60 | **19.98** | 67.8 / 83.5 | 50.1 / 51.0 | 8.5 / 8.7 |
| 3 | 14.22 | **19.95** | 67.1 / 83.2 | 50.1 / 53.3 | 8.1 / 9.3 |
목표 20fps(50ms) 대비 **99.8~99.9%**.

### 전투 · 마을(교대 짝 · 같은 측정법)
| 장면 | 짝 | main fps (간격 p50/p95) | 수정 fps (간격 p50/p95) | load1 |
|---|---|---|---|---|
| 전투(포획 튜토리얼 · 20초) | 1 | 16.30 (66.5 / 66.9) | 16.65 (66.4 / 67.6) | 8.3 / 8.0 |
| 전투 | 2 | 15.60 (66.6 / 66.9) | 16.70 (65.6 / 67.7) | 8.6 / 8.0 |
| **마을 자유 이동**(#331 경로 97단계 뒤 좌우 걷기 20초) | 1 | **11.95** (83.3 / 85.5) | **20.00** (50.1 / 52.2) | 11.6 / 6.6 |
- 전투의 목표는 약 16.7fps(수정본에서 sleep 이 33ms 로 여유 — 마감 방식이라 주기 = 장면 주기). main 은 sleep 이 바닥에 붙어 15.6~16.3.
- 마을: 두 판 모두 마을 화면에서 끝났다(끝 화면 캡처). 수정본은 같은 키 시간에 더 걸어 «아놀드» 표지 자리까지 갔다(게임이 빨라진 결과).

### 회귀 — 라이브 LGT 4종 + KTF 영웅서기4(교대 2짝 · 20초 · OK 2.5초마다)
페인트 수 = blit/s ÷ 2(WebScreen 은 페인트마다 putImageData + drawImage). fps 칸은 3ms 병합 프레임 수.
| 타이틀 | main 페인트/s (fps) | 수정 페인트/s (fps) | 판정 |
|---|---|---|---|
| 영웅서기4 (KTF) | 39.2 · 37.5 (37.35 · 36.4) | 37.4 · 37.5 (36.35 · 36.6) | 같음 — ≈35fps 유지 |
| 메이플스토리2007 | 13.65 · 13.65 (13.65 · 13.65) | 13.65 · 13.75 (13.55 · 13.65) | 같음 |
| 현영맞고2006 | 3.8 · 3.8 (3.8 · 3.8) | 3.75 · 3.8 (3.3 · 3.15) | 같음 — fps 칸은 병합 착시(한계 절) |
| 체스마스터 | 0.8 · 0.8 | 0.8 · 0.8 | 같음(정지 화면) |
| 놈3 | 6.8 · 6.75 | 9.9 · 7.0 | 같거나 빠름 |
- 끝 화면 비흑 화소는 짝마다 같다. 메이플만 판마다 41,483 또는 34,560 인데, 앞선 귀속 측정에서 main 도 34,560 이 나왔다(화면 전환 시점).

### 주입(네이티브 release `wie_validate --inject --boot-secs 15 --action-secs 1.2 --max-ticks 100000000000` · 교대 2회)
| 타이틀 | main | 수정 |
|---|---|---|
| 메이플스토리2007 · 체스마스터 · 놈3 · 영웅서기4 | PASS 27/27 ×2 | PASS 27/27 ×2 |
| 현영맞고2006 | PASS · UNMEASURED(clean exit 10/27 — main 쪽) | PASS 27/27 ×2 |

runner 블록(release 바이너리): draw_j2me · helloworld_ktf/lgt · text_j2me PASS. keydraw_ktf/lgt 는 기본 max-ticks 에서
main·수정 **둘 다** UNMEASURED(stop=max-ticks · 4/27 · 약 5초 · load1 6~8 — release 가 debug `cargo run` 보다 tick 을 빨리 쓴다) →
`--max-ticks 400000000` 에서 main·수정 모두 **PASS 27/27 · rc=0 · last true**(각 2회).

### 게이트 · 변이
- fmt · clippy `-D warnings`(stable · wasm32 · beta) · `RUST_MIN_STACK=4194304 cargo test --all`(495 passed / 0 failed) · `npm run build:wasm` 전부 rc=0.
  `check-engine-contract` 109 pass · `npm run audit` PASSED.
- 새 시험 6개. 변이 10종 전부 red(되돌림 → 해당 시험 FAILED · 무변이 49/49 green):
  repaint 를 호스트 플래그로 되돌림 · repaint 즉시 push · yield 가 전달 안 함 · 첫 yield 에 전달 · 스트릭 전역 · sleep 이 리셋 안 함 ·
  tick 끝 전달 없음 · 대기가 큐를 안 봄 · GC 매 페인트(0) · GC 주기 1,000,000.

### 측정 방법
- 하네스: `reports/evidence/…-speed-regression-bisect` 의 harness 를 확장(키 스크립트 · 창 시작 · 장면별 타임라인).
  fps = 창 안 blit 프레임 수(3ms 안의 putImageData+drawImage 는 1프레임). 키 = `docs/keys/battlemonster-village.keys` 앞부분(숲/전투는 시각 기준).
- 브라우저는 `open -na "Google Chrome for Testing.app" --args --headless=new --remote-debugging-port=9333`.
  ★**디스플레이가 잠들면 rAF 가 0 이 된다**(측정 중 1회 겪음 · 그 판은 버렸다) — `caffeinate -u -d` 로 깨워 두고 쟀다.
- 시간 분해는 스크래치 마커 빌드(커밋 안 함): repaint · 페인트 시작/끝 · GC 끝 · sleep 인자 · yield 수 · tick 시작/끝을
  `performance.now()` 로 적어 하네스가 꺼냈다.
- 빌드: CPU 클램프 없음(재부팅 뒤) — 네이티브 release 빌드 3분 22초 · 97% CPU, wasm release 35초~1분, load1 5~40.

### 한계
- **GC 가 1초에 한 번**이라 쓰레기가 최대 1초 산다. 힙이 빡빡한 타이틀은 고갈이 앞당겨질 수 있다
  (upstream `95555afe` 전에는 게스트 `gc()` 호출 때만 돌았다). 할당 실패 시 GC 후 재시도는 넣지 않았다 — 후속 제안.
- 마을은 1짝만 쟀다(한 판 12분 — 키 경로 707초). 숲 3짝 · 전투 2짝과 같은 형태(sleep 바닥 → 여유)라 추가 짝을 생략했다.
- 스핀 판정은 «같은 task 의 sleep 없는 연속 yield 2회»다. 페인트가 아닌 다른 것을 yield 로 기다리는 스레드도 걸린다 —
  그때도 결과는 «대기 중인 repaint 가 tick 끝보다 먼저 전달»뿐이다.
- 현영맞고2006 의 fps 칸은 3ms 병합 때문에 낮게 보인다: 마커 계수로 페인트 3.7/s · 게임 루프 sleep 9.6/s 가 main 과 같다
  (GC 가 빠져 두 페인트(serviceRepaints + 이벤트)가 3ms 안에 붙어 1프레임으로 세어진다). blit/s ÷ 2 칸으로 읽어라.

- 게임 파일명 유입(`scripts/corpus-name-inflow.mjs --corpus <로컬 코퍼스>`): BOUNDED 16건(원장에 이미 쓰인 제목들) + SUFFIX-ATTACHED 1건(`배틀몬스터가` — 조사가 붙은 진짜 언급, 다른 제목 아님).

<!-- corpus-name-inflow v1 subjects=7 tree=86f99020e931b345 B=29/16 P=0/0 S=2/1 -->
