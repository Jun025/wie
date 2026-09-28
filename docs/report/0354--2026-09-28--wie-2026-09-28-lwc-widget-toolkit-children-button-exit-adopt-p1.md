## [2026-09-28] a10a1f02b41b 의 0xFFFFFFFC 벽 — 원인은 KTF catch 블록의 `e` 가 0, #375 가 이미 걷었다 (wie-2026-09-28-lwc-widget-toolkit-children-button-exit-adopt-p1)

**무엇을**: `docs/report/0345` 가 남긴 KTF `a10a1f02b41b` 의 `0xFFFFFFFC` 잘못된 메모리 접근(PC `0x71008b7a` 네이티브 함수)을 추적했다. 코드 변경은 없다.
**왜**: 0345 는 «널 객체 헤더 읽기로 보이지만 확인하지 않았다»고 적었다.
**사용자 영향**: 이 타이틀은 현행 `main` 에서 이미 첫 화면 너머로 간다 — 벽을 걷은 것은 같은 날 착지한 #375(`84461452` · `wie-ktf-paint-npe-loop`)다. 0345 의 «후» 측정은 #375 이전(`1dd9f81c`)을 바탕으로 한 PR 브랜치에서 쟀기 때문에 그 효과가 안 보였다.

### 원인
- 게스트가 `wec/DMInfo` 를 로드하다 실패하면(`load_java_class(wec/DMInfo) failed`) 예외가 나고, 메서드 `0x189414` 의 catch 핸들러(`target 0x54`)가 받는다.
- #375 이전의 `JavaMethod::handle_exception` 은 핸들러의 `e` 칸(`RawJavaExceptionHandler.unk3`)을 채우지 않았다. 그래서 catch 블록 안의 `e` 가 0 이고, `e` 를 건드리는 순간 같은 try 범위 안에서 다시 예외가 나서 같은 핸들러로 돌아온다.
- 이 되돌이는 두 가지로 끝난다: 널(0)의 헤더를 읽다가 `0 - 4 = 0xFFFFFFFC` 에서 멈추거나, 되돌이마다 만든 예외 객체로 힙이 차서 `AllocationFailure` 로 멈춘다. 0345 가 중간 빌드에서 한 번 보고 재현하지 못한 `AllocationFailure` 가 이것이다.
- 즉 가설(«널 객체 헤더 읽기») 그대로다. KTF 런타임 ABI 를 새로 구현할 일은 없다.

### 측정 — release · `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(0345 의 프로브 A) · load1 200~290

| 빌드 | n | 결과 | 핸들러 `0x189414:0x54` 진입 수 |
|---|---|---|---|
| 0345 의 PR 머리 `76795f84`(#375 이전 바탕) · 로그 끔 | 3 | 3/3 PASS 이지만 `stop: deadline` · paints **3** — 멈춰 있다 | — |
| 같은 빌드 · `RUST_LOG=wie_ktf=debug` | 3 | 1/3 **FAIL `0xFFFFFFFC` · PC `0x71008b7a`**(0345 와 같은 벽) · 2/3 paints 3~4 정지 | 3 · 12,734 · 13,737 |
| 현행 `main` `081029a5` | 5+3 | **8/8 PASS · `stop: max-ticks` · 27/27 키 · paints 152~166** | **1** (로그 켠 2판) |
| 현행 `main` 에서 `unk3` 쓰기 5줄만 뺀 변이 | 6 | 6/6 `stop: deadline` · paints 4~8 — 멈춤으로 되돌아간다 | — |
| 같은 변이 · 로그 켬 | 4 | 4/4 FAIL `AllocationFailure` | **26,173** (4판 모두) |

PASS 판정 문구가 «멈춤»을 가리는 점에 주의: `76795f84` 와 변이의 PASS 는 30초 창에서 3~8 장만 그린 채 창이 닫힌 것이다(`stop: deadline`). 현행 `main` 은 창이 닫히기 전에 틱 상한(`max-ticks`)에 먼저 닿는다.

### 시험
#375 가 넣은 `test_catch_handler_receives_thrown_exception`(`wie-ktf`)이 이 쓰기를 지킨다. 위 변이(쓰기 5줄 제거)에서 그 시험은 **red**(`assertion left == right failed: the catch block reads e from this slot` · `jvm_support.rs:1014`)이고, 원상에서 green 이다. 새 시험은 넣지 않았다.

### 남은 것
- 현행 `main` 의 이 타이틀 마지막 장면은 한 색인 판이 있다(`last_frame_content: false` — 로그 끈 첫 판). 이번 범위 밖이고 따로 재지 않았다.
- `wec/DMInfo` 로드 실패 자체는 그대로다. 게스트가 잡아서 넘어가므로 지금은 벽이 아니다.

### 게임 이름 유입
`node scripts/corpus-name-inflow.mjs` — BOUNDED **0** · SUFFIX-ATTACHED **0**.


<!-- corpus-name-inflow v1 subjects=2 tree=2313f346f8eba033 B=0/0 P=0/0 S=0/0 -->
