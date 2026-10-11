## [2026-10-11] upstream #1122(`4fcd4b74020e` 스테이지 5 정지) — main 포함 여부 판정 (wie-upstream-1122-stage5-stall-repro-verdict)

**무엇을**: STATE.md «다음» ③ 을 실측으로 닫았다. 엔진 코드 변경 0.
**왜**: ③ 은 2026-08-15 기준 「upstream OPEN」이었는데 upstream 이 2026-09-07 에 닫았다.
**사용자 영향**: 없음(판정·문서). 스테이지 5 정지가 우리 빌드에서 사라졌다고 «측정»한 것은 아니다.

### ⑴ 해결 커밋 — 특정 불가, 그러나 전부 포함

- `gh issue view 1122 -R dlunch/wie`: CLOSED **2026-09-07T11:16:40Z**, 오너 댓글 「이슈 해결되었습니다」. 연결 PR 0,
  타임라인의 커밋 참조 2건(`78f40a6f`·`225392e8`)은 우리 포크의 문서 커밋이다(우리 #52·#56).
- 닫기 직전 upstream 엔진 커밋: `45d4791e` Preserve ARM instruction budget across SVC calls (#1398, 15:54 KST) ·
  `12e451cc` Fix input backlog from recurring display callbacks (#1405, 19:32 KST — 닫기 44분 전) ·
  `413009a4`·`b37455d0`·`28f85c14`(KTF 클래스 등록·vtable·AOT). 어느 것이 고쳤는지 upstream 은 말하지 않았다.
- 그래서 «어느 것»이 아니라 «전부»를 쟀다:
  `git rev-list upstream/main --before=2026-09-07T11:16:40Z | git rev-list --stdin --not origin/main | wc -l` → **0**.
  위 후보 5개 각각 `git merge-base --is-ancestor <c> origin/main` → 전부 ANCESTOR (merge-base `44fbf265`, 2026-09-15).
- 조상이라도 오버레이가 덮었을 수 있어 두 주후보의 산출물을 main 트리에서 확인했다: `instructions_executed`
  (#1398) `wie-core-arm/src/{core,engine,engine/arm32_cpu}.rs` 에 있음 · `wie-midp/src/classes/net/wie/command_event.rs`(#1405) 있음.

### ⑵ 재현 수단 — 없음

- 픽스처·`docs/keys/` 에 이 타이틀(`4fcd4b74020e`) 없음. 로컬 진도 레시피 `game_lab/recipes-progress/4fcd4b74020e.keys` 는
  1화 전투 지도에서 막힌다(`docs/report/0507` 원장: «move is never confirmed on the battle map» · 0478 600초 P/P2 stuck).
- 동봉 세이브는 진도를 열지 않는다(`docs/report/0419`: 이어하기 → 1화).
- ⇒ 스테이지 5 는 자동으로 도달하지 못한다. 장시간 실측은 경계상 하지 않았다.

### ⑶ 결론 — ⒜(재현은 미측정)

upstream 이 «해결»이라 한 시점의 코드는 전부 main 에 있다 → ③ 종결. 우리 빌드에서의 재현 0 은 **측정하지 못했다**
(수단 부재). 재개 조건: 우리 빌드에서 스테이지 5 정지 보고가 새로 들어올 때. upstream 발신 0.
