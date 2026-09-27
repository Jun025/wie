## [2026-09-27] Canvas.serviceRepaints → Display.serviceRepaints — 아포칼립스 호스트 abort 제거 (wie-2026-09-27-canvas-service-repaints-via-display-apocalypse-adopt-p0)

**무엇을**: `wie-midp` `Canvas::service_repaints` 가 `Display.handlePaintEvent` 를 직접 부르던 것을 `Display.serviceRepaints` 로 넘긴다.
그쪽은 `repaintPending` 일 때만 `handlePaintEvent` 를 부른다(MIDP 정의 — 대기 중 repaint 가 없으면 즉시 돌아온다).
**왜**: 채택 제안 `2026-09-27-wie-display-handle-paint-event-concurrent-graphics-judge#p0`. 아포칼립스는 `CardCanvas.paint` 안에서
`Card.serviceRepaints` → `Canvas.serviceRepaints` 를 부르고, 그것이 무조건 다시 그려 paint 안 paint 가 끝없이 반복됐다(스택 넘침 · 호스트째 abort).
**사용자 영향**: 아포칼립스가 시작하자마자 에뮬레이터째 죽지 않는다. 그 밖의 타이틀은 화면 유무가 바뀌지 않았다(아래 스윕).

### 아포칼립스 (네이티브 release `wie_validate --timeout 10` · 교대 3짝)
| 짝 | 기준 origin/main dca280ec | 수정 |
|---|---|---|
| 1 | rc=134 · JSON 없음(stack overflow) | rc=0 · PASS · paints 1,606 · last true |
| 2 | rc=134 · 같음 | rc=0 · PASS · paints 368 · last true |
| 3 | rc=134 · 같음 | rc=0 · PASS · paints 2,149 · last true |

### 코퍼스 짝 — `serviceRepaints` 를 참조하는 타이틀 전건
대상 = 로컬 코퍼스 `working/`·`broken/` 에서 클래스(중첩 jar 포함) 바이트에 `serviceRepaints` 가 있는 **323 타이틀**
(KTF 284 · SKT 25 · LGT 11 · 미분류 3). 타이틀마다 기준/수정 순서를 번갈아 1회씩 · `--timeout 10` · 6병렬 · load1 150~370.
«화면 나옴» = `PASS` 그리고 `last_frame_content true`.
| 결과(기준, 수정) | 타이틀 |
|---|---|
| PASS, PASS | 255 |
| FAIL, FAIL | 62 |
| PASS → FAIL | 3 |
| FAIL → PASS | 2 |
| 없음(abort) → PASS | 1 (아포칼립스) |
화면 나옴 합계: 기준 253 · 수정 252.

**어긋난 10 타이틀 재측(교대 3짝)** — 기준에만 화면 6 · 수정에만 화면 4(아포칼립스 제외):
10 타이틀 × 3판에서 화면 나옴 기준 **21** · 수정 **22**. 전부 **기준도** 판마다 뒤집힌다(기준 FAIL/0 paint 가 5 타이틀에서 나옴) ⇒ 부하 굶주림.
가장 약했던 1건(스윕 기준만 화면 · 재측 기준 2/3 · 수정 1/3)은 `--timeout 20` 교대 5짝에서 **기준 5/5 · 수정 5/5** PASS·last true.
⇒ **수정 뒤 화면이 안 나오게 된 타이틀 0.**

### 지정 회귀 타이틀(스윕 행 · 같은 조건)
| 타이틀 | 기준 | 수정 |
|---|---|---|
| 영웅서기4 (KTF) | PASS 49 · true | PASS 34 · true |
| 현영맞고2006 (LGT) | PASS 96 · true | PASS 84 · true |
| 체스마스터 (LGT · KTF) | PASS 2 · true · PASS 2 · true | PASS 2 · true · PASS 1 · true |
| 놈3 (LGT · KTF) | PASS 64 · true · PASS 156 · true | PASS 60 · true · PASS 169 · true |
| 메이플스토리2007 (LGT · 교대 2짝 · `--timeout 20`) | PASS 122 · 212 · true | PASS 257 · 233 · true |
- LGT 메이플스토리2007 은 `serviceRepaints` 를 참조하지 않아 스윕 대상이 아니다(변경 경로 밖) — 지정 타이틀이라 따로 쟀다.
- paints 는 10초 창 · 부하 150~370 의 한 판 값이다. 짝끼리의 차는 재측에서 양쪽으로 뒤집힌다(위) — 판정은 화면 유무로 한다.

### 동작 변화(제안의 tradeoff 그대로)
guest paint 밖에서 `repaint()` 없이 `serviceRepaints()` 만 부르면 종전에는 매번 그렸고 이제 그리지 않는다. 위 스윕에서 그 때문에
화면을 잃은 타이틀은 없었다. 새 시험이 이 동작을 잠근다(«대기 없음 → 0회 · repaint 뒤 → 1회»).

### 게이트 · 변이
- `RUST_MIN_STACK=4194304 cargo test --all` **497 passed / 0 failed** · fmt · clippy `-D warnings`(stable · wasm32 · beta) rc=0.
- runner 블록: draw_j2me · helloworld_ktf/lgt · text_j2me PASS · keydraw_ktf/lgt PASS 27/27 · rc=0 · last true.
- 새 시험 `canvas::test::service_repaints_paints_only_a_pending_repaint`(paint 안에서 serviceRepaints 를 부르는 캔버스 · 재귀는 8회에서 끊는다).
  변이(`handlePaintEvent` 직접 호출로 되돌림) → **FAILED** `left: 9 right: 1`(«serviceRepaints inside paint re-entered paint») · 무변이 green.

### 한계
- paint 안에서 `repaint()` 를 부르고 이어서 `serviceRepaints()` 를 부르는 타이틀은 여전히 재귀한다(대기 중 repaint 가 매번 생긴다).
  아포칼립스는 그 형태가 아니다(원 제안 계수: repaint 3회). 교차 진입(다른 task)의 Graphics 상태 덮어쓰기는 이 회차 범위 밖이다.
- 스윕은 타이틀당 1짝이다. 어긋난 것만 재측했다.

- 게임 파일명 유입(`scripts/corpus-name-inflow.mjs --corpus <로컬 코퍼스>`): BOUNDED 5건(원장에 이미 쓰인 제목들) · SUFFIX-ATTACHED 0건.
