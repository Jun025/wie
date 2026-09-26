## [2026-09-27] `Display::handlePaintEvent` 동시 진입 판정 — 놈3 no-frame 의 원인이 아니다 · 대신 같은 스레드 재진입이 아포칼립스를 호스트 abort 로 죽인다 (wie-display-handle-paint-event-concurrent-graphics-judge)

**무엇을**: 제안 `2026-09-25-nom3-no-frame-low-load-split#p1` 채택. 두 스레드가 한 `screenGraphics` 로 `handlePaintEvent` 에
같이 들어가는 일이 얼마나 흔한지, 그리고 해를 끼치는지 스크래치 프로브로 쟀다. 코드 변경은 없다(이 파일과 worklog 뿐).

**왜**: 0265 의 FAIL B 에서 thread 3(`serviceRepaints`)과 thread 4(이벤트)가 같은 Graphics 로 같은 guest paint 에 들어가 있었다.
선행 #312(`--profile-out`, 머지 `a8c552d2`)는 착지했다.

**사용자 영향**: 없음(측정 회차). 아래 ⑶의 크래시는 제안으로 넘긴다.

### 판정

| 질문 | 답 | 근거 |
|---|---|---|
| 놈3 «한 프레임도 안 그리는» 판의 원인인가 | ★**아니다** | 교차 진입이 놈3 `--inject` **6/6 판 전부**에서 판당 29–46회 난다 ⇒ 흔한 경로이고 FAIL 과 PASS 를 가르지 못한다. 원인은 0296 이 이미 쟀다: 게스트 모니터 교착(스레드 3 A→B · 스레드 4 B→A, FAIL 3/3 같은 모양) |
| 교차 진입이 해를 끼치나 | **상태는 바뀐다 · 화면 영향은 안 쟀다** | 뒤에 들어온 paint 의 `Graphics.reset()` 이 **guest paint 안에 있는 다른 paint** 의 색을 0 으로 되돌린다(⑵) |
| 같은 스레드 재진입 | ★**크래시다** | guest `paint` 안의 `serviceRepaints` 를 `Canvas::serviceRepaints` 가 `Display::handlePaintEvent` 로 **무조건** 넘겨 끝없이 재귀한다 ⇒ 호스트 스택 넘침 abort(⑶) |

### ⑴ 프로브와 측정 (커밋 안 함)

- 프로브(`probe.diff`, origin/main `a741291f` 위): `handle_paint_event` 진입·퇴장에 실행기 task id 와 진입 깊이를 적고,
  Graphics 를 바꾸는 네 자리(pre-reset · post-reset · chrome · null-displayable)에서 **다른 paint 가 guest
  `Displayable.handlePaintEvent` 안에 있으면** 그 순간의 공유 Graphics 상태(translate · clip · color)를 적는다.
- 스윕: `game_lab/working` 294타이틀 · `--timeout 10` · 병렬 8 · load1 30–89.

| 통신사 | 타이틀 | paint 진입 합 | 겹친 타이틀 |
|---|---|---|---|
| KTF | 190 | 15,908 | **20** |
| LGT | 54 | 237 | 0 |
| SKT | 50 | 5,073 | 0 |

- 겹친 20타이틀을 task id 프로브로 다시 돌렸다(겹침은 판마다 달라 15타이틀에서 재현): **교차 13** · 같은 task 1회 중첩 1(유계) ·
  ★**같은 task 무한 중첩 1**(⑶).
- 놈3(`game_lab/broken/lgt` · sha256 `b475b639…` · `--inject`) 6판: 교차 29/38/39/45/45/46 · 중첩 0.
  ★6판 전부 `UNMEASURED · max-ticks` 다 — PASS/FAIL 짝 비교가 아니다. «흔한 경로라 가르지 못한다»의 근거는 부모 wie#300 의 PASS 판 관측과 0296 의 교착 원인이다.

### ⑵ 교차 진입이 바꾸는 것

- 레이아웃은 관측한 전 판에서 `content=(0,0)` 전체 화면이다 ⇒ `reset()` 의 translate·clip 되돌리기는 **값이 같아 무해**하다.
- 바뀌는 것은 **색**이다: 뒤에 들어온 paint 의 첫 `reset()` 이 앞 paint 가 guest 안에서 설정한 색을 지운다.
  놈3 6판에서 교차 진입 242회 중 **235회**가 색이 기본값이 아닌 상태를 지웠다(예: `-16711681`, `16777215` → `0`).
  KTF 스윕 첫 판에서 10타이틀이 같은 모양이었다.
- `reset()` 은 font·xorMode 도 되돌리지만 이 프로브는 그 둘을 읽지 않았다.
- ★**화면에 보이는지는 재지 않았다** — 앞 paint 가 재개한 뒤 색을 다시 설정하면 흔적이 없다. 픽셀 대조 없이 «깨진 화면»이라 적지 않는다.

### ⑶ 같은 task 재진입 — `(KTF)아포칼립스` 호스트 abort

- 프로브 판: `depth_before` 가 0 → 424 까지 한 task 에서 오른 뒤 `thread 'main' has overflowed its stack` · JSON 줄 없음.
- ★**프로브 없는 origin/main 바이너리 3/3 같은 결과**: `rc=134` · `fatal runtime error: stack overflow, aborting` · JSON 0바이트.
- 경로(`RUST_LOG=wie_midp=debug,wie_wipi_java=debug`, 마지막 반복):
  `Display::handlePaintEvent` → `Canvas::handlePaintEvent` → `net.wie.CardCanvas::paint` → guest `Card.paint`
  → `org.kwis…Card::serviceRepaints` → `Canvas::serviceRepaints` → `Display::handlePaintEvent` → …
- 계수(프로브 없는 origin/main `bb13b293` release · 같은 명령 · rc=134): `Display::handlePaintEvent` **425** · `Canvas::handlePaintEvent` 425 ·
  `Canvas::serviceRepaints` **424** · `Card::serviceRepaints` 424 · ★`Display::serviceRepaints` **0** · repaint 계열 **3**.
- 메커니즘: `Canvas::serviceRepaints`(`wie-midp/.../lcdui/canvas.rs` `service_repaints`)가 `getDisplay()` 뒤
  **`repaintPending`·진행 중 paint 여부를 보지 않고** `Display::handlePaintEvent` 를 직접 부른다. `repaintPending` 을 보는
  `Display::serviceRepaints` 는 이 경로에서 한 번도 불리지 않는다 — 424회 재귀 동안 repaint 는 3회뿐이다.
  ※이 절의 초판은 경로에 `Display::serviceRepaints` 를 넣고 원인을 `repaintPending` 재설정으로 적었다 — 게이트② 실측으로 틀렸다.
- 스크래치 확인(커밋 안 함 · 제안 `#p0` 의 효과 근거): `canvas.rs` `service_repaints` 의 호출 대상을 `handlePaintEvent` → `Display::serviceRepaints`
  로 한 줄 바꾼 release 바이너리에서 아포칼립스 `--timeout 10` **3/3 rc=0 · `PASS` · `content true`**(paints 505/549/570 · stop=deadline).
  같은 바이너리로 러너 줄: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me` PASS · `keydraw_*` `--inject` 는 `UNMEASURED · max-ticks`
  — 가드 없는 main release 도 같은 결과(load1 ≈ 50)라 가드와 무관하다.

### 하지 않은 것

- **직렬화 실험**: 원인이 아니므로 계약대로 판정으로 닫았다. 게다가 0296 의 교착 모양(스레드 3 이 A 를 쥔 채 `serviceRepaints` · 스레드 4 의
  paint 가 A 를 기다림)에서 호스트 잠금으로 직렬화하면 guest 교착이 호스트 대기로 옮겨갈 뿐이다. 교차 진입의 색 되돌리기 수리는
  화면 영향을 재기 전에는 제안하지 않았다.
- **놈3 FAIL 재포착**: 조용한 기계(load1 6)에서 프로브 없는 main 도 `--inject` 3/3 `UNMEASURED · max-ticks`(ticks 50,000,000 · 입력 13/27)로
  끝난다 — 이 회차와 무관한 기존 동작이고 프로브 판도 같다.
- 스윕 FAIL 7건은 판정에 쓰지 않았다(겹침 여부와 무관한 기존 결과).

### 검증

- 증거(저장소 밖 · 게임 바이트 0): `~/orchestrator/reports/evidence/wie-display-handle-paint-event-concurrent-graphics-judge/` —
  `probe.diff` · `one.sh` · `classify.py` · `sweep.tsv`(294) · `rerun.tsv` · `n3.tsv` · 겹친 판의 `*.err.gz`.
- 네 게이트 + beta clippy 는 회신에 원문.

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus <game_lab>`): BOUNDED 3쌍 — `놈3`(이 회차의 대상 · 리니지 0265·0276·0296 이 이미 적은 타이틀) · `(KTF)아포칼립스`(⑶의 크래시 재현 대상 — 이름 없이는 재현 명령이 성립하지 않는다) · SUFFIX-ATTACHED 0.

<!-- corpus-name-inflow v1 subjects=2 tree=ca71daa193645462 B=15/3 P=0/0 S=0/0 -->
