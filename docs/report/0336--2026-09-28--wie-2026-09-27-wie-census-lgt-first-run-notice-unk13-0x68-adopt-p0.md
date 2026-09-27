## [2026-09-28] 웹 셸: 게임이 스스로 끝나면 저장 후 «다시 실행» 패널 (wie-2026-09-27-wie-census-lgt-first-run-notice-unk13-0x68-adopt-p0)

**무엇을**: `web/` 셸(`wie-web`)이 매 tick 뒤 `has_exited()` 를 본다. 참이면 루프를 멈추고 → 최종 세이브를 persist 하고 → **그 다음에** 종료 패널(«다시 실행» / «나가기»)을 띄운다.
«다시 실행» 은 기존 재부팅 경로(`bootNonce`)이고, 부팅 때 같은 ROM 해시의 세이브를 불러온다.
- `web/src/lib/emulator.ts` — `onExit` 콜백 · 루프의 종료 분기(stop → persist → onExit)
- `web/src/components/Player.tsx` — `status: "exited"` · 종료 패널
- `scripts/contract-roundtrip.mjs` — Scenario B 에 DB 레코드 1개를 심고, 종료 뒤 export 가 그대로인지 · 새 인스턴스에 import 해 끝까지 돌려도 같은지(B-relaunch) 단언

**왜**: LGT 일부 타이틀은 첫 실행에서 표식 DB 를 쓰고 스스로 종료한다(docs/report/0325 · WIPIC 0x68). 두 번째 실행부터 본 게임이 나온다.
이 셸은 종료 신호를 읽지 않았다. 그래서 종료된 화면이 그대로 멈춰 보였고, 빠져나갈 길은 «나가기» 뒤 재실행뿐이었다.
★**자동 재실행은 하지 않는다** — 메뉴 «종료» 와 첫 실행 종료가 같은 호출이어서, 자동으로 다시 켜면 사용자가 고른 종료까지 되살린다.

**사용자 영향**: 첫 실행 안내 뒤 꺼지는 게임에서 버튼 한 번으로 본 게임에 들어간다. 메뉴에서 «종료» 를 고르면 같은 패널이 뜨고, 사용자가 «나가기» 를 고를 수 있다.

### 검증
- `node scripts/contract-roundtrip.mjs` rc=0 — `B: DB survives the clean exit (export == seed) — 52 vs 52 bytes` · `B-relaunch: relaunched instance runs to exit with the same DB — 52 save bytes`.
- 실앱(로컬 vite · 헤드리스 Chromium) · LGT `8f7758fa43b6`:
  첫 실행 안내 → Enter **1회** → 종료 패널 → «다시 실행» → 키 설정 안내 → Enter 3회 → 타이틀 화면. 종료 패널 재등장 없음 · 페이지 오류 0.
  캡처는 게임 화면이라 저장소 밖에 둔다: `~/orchestrator/reports/evidence/wie-2026-09-27-wie-census-lgt-first-run-notice-unk13-0x68-adopt-p0/{1-first-run-notice,2-exit-panel,3-after-rerun,4-after-rerun-keys}.png`.

### 범위 밖
- otterpebble featurephone 셸(worklog 의 `target`)에는 같은 흐름이 **이미 있다** — `apps/featurephone/app/page.tsx` `handleCleanExit`(persist → free → 종료 패널)와 `replay`, 착지 `9084141c7`(2026-07-13). 그래서 이 회차는 wie 의 `web/` 셸만 고친다.
- `docs/player-updates` 는 넣지 않았다. 그 목록은 featurephone 사이트용이고, 이 변경은 `wie-web` 셸에만 있다.

- 게임 파일명 유입: BOUNDED 0 · SUFFIX-ATTACHED 0 (`node scripts/corpus-name-inflow.mjs`).

<!-- corpus-name-inflow v1 subjects=5 tree=f4cb8c1b73ba9115 B=0/0 P=1/1 S=0/0 -->
