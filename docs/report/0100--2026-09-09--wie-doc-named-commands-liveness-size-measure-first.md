## [2026-09-09] 「문서가 이름 대는 명령이 도는가」의 크기를 쟀다 — ★구현 0 · 빈틈은 «20개»가 아니라 «한 블록»이다 (wie-doc-named-commands-liveness-size-measure-first)
- **무엇을**: ★**재는 회차 · 구현 0 · 훅/CI/잡 신설 0.** 산출은 이 기록과 worklog 의 배치안뿐이다.
- **왜**: 채택 제안 `2026-09-06-parity-outside-gate-decision#p1`(「배치 결정이 선행 · 구현만 M」).
- **★F1⑴ 전수**: 명령은 **`AGENTS.md` 한 파일**에만 있다(`CLAUDE.md`·`README.md` 코드펜스 **0**).
  펜스 **7블록 · 실행 가능 줄 25** → 셸 골격 4(`for`·`done`) + 자리표시 1(`$EDITOR`) 제외 ⇒ ★**대상 명령 20개.**
  ★★**계수기 자체가 처음에 틀렸다** — `grep '^```'` 은 «들여쓴 펜스»를 못 봐 **2블록 5줄을 놓쳤다**(10 ↔ 실제 14).
  대조군(`^ *```` `)으로 잡아 바로잡았다. ⇒ ★**「전수」를 주장하려면 계수기부터 대조해야 한다.**
- **★★F1⑵ 의 핵심 발견 — 「어디에도 안 도는 것」은 «하나»다**: 워크플로 전수 대조 결과
  ★**`wie_validate` 만 «어디에도 없다»**(러너 블록 5줄). `make-draw-fixture` 조차 `engine-contract.yml` 에서 돈다.
  `smoke_gate.sh` 도 0건이지만 그것은 ★**제약 9 때문에 «구조적으로 불가»**이고 이미 결정된 자리다.
  ⇒ ★**제안이 겨눈 빈틈은 «20개 명령»이 아니라 «러너 블록 하나»다** — 크기가 한 자리 줄어든다.
- **★그러나 «두 번째» 빈틈이 남는다(더 미묘하다)**: 문서는 **호출 별칭**(`npm run audit`·`build:wasm`·`frontend`·`verify`)을 이름 대고
  CI 는 **실체**(`bash scripts/audit-no-leak.sh` 등)를 돈다. ⇒ ★**`package.json` 이 별칭을 개명해도 CI 는 green 이고 문서만 죽는다.**
  ★이것이 천장 ③ 이 「문서−CI 6 · 이름만 다름」으로 잰 바로 그 항목이고, ★**실행만이 가른다**(존재 검사는 그 회차가 이미 기각했다).
- **★F1⑶ 표본 실행(무해·저비용만 · 12종 전건 rc=0)**: fmt 25s · clippy 28s · clippy(wasm) 14s · `cargo test --all` **219s** ·
  beta clippy 7s · audit 27s · grep 3s · coverage 8s · serial 3s · engine-contract 4s ·
  `wie_validate helloworld_ktf` **9s** · `--inject --expect-last-frame keydraw_ktf` **24s**.
- **★★부작용 있는 8종은 «돌리지 않았다»**(목록으로만): `rustup toolchain install beta` · `make-draw-fixture`(jar 쓰기) ·
  `build:wasm`(web/src/wasm 쓰기) · `frontend`(npm install·dist 쓰기) · `contract-roundtrip`(브라우저 내려받기) ·
  `verify`(실 Chrome + 배포 URL) · `check-worklog-coverage --record`(원장 append) · `--next-serial`(gh api).
  ⇒ 그 비용은 ★**CI 이력에서 «실행하지 않고» 쟀다**: `web.yml` 최근 5회 **217·230·231·240·407s** ·
  `engine-contract.yml` ★**필터가 접으면 11~16s · 실제로 돌면 213~221s**(같은 워크플로가 조건에 따라 **20배**).
- **★부수 실측**: `test_data/draw_j2me.jar` 는 ★**미추적**(`.gitignore:24 *.jar`) ⇒ 신선한 클론엔 **없다**.
  러너 블록의 첫 줄은 «선택»이 아니라 **선행 writer** 다 — 어떤 배치든 그것을 돌리거나 그 픽스처를 건너뛰어야 한다.
- **사용자 영향**: 없음(문서). 대신 ★**배치를 «고를 수 있게» 됐다** — 그것이 제안이 요구한 산출이다.
- **★고르지 않았다**: 배치 4안과 각 대가는 worklog `proposals` 에 있다. ★**총괄·운영자가 고른다.**
