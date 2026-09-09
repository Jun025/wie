## [2026-09-09] 「문서가 이름 대는 명령이 도는가」의 크기를 쟀다 — ★구현 0 · 빈틈은 «20개»가 아니라 «한 블록»이다 (wie-doc-named-commands-liveness-size-measure-first)
- **무엇을**: ★**재는 회차 · 구현 0 · 훅/CI/잡 신설 0.** 산출은 이 기록과 worklog 의 배치안뿐이다.
- **왜**: 채택 제안 `2026-09-06-parity-outside-gate-decision#p1`(「배치 결정이 선행 · 구현만 M」).
- **★F1⑴ 모집단 정의 + 전수**: 제안이 「**커밋 전** 명령」을 겨누므로 모집단을 **`AGENTS.md`** 로 좁혔다.
  펜스 **7블록 · 실행 가능 줄 25** → 셸 골격 4(`for`·`done`) + 자리표시 1(`$EDITOR`) 제외 ⇒ ★**대상 명령 20개.**
  ★★**[게이트② 정정 · R2] 초판의 「명령은 «한 파일»에만 있다」는 «거짓»이었다** — `sh|bash` 펜스를 가진 추적 `*.md` 는 **8개**
  (`AGENTS.md`·`REPORT.md`·`docs/CLOUDFLARE_SETUP.md`·`docs/COMPLIANCE.md`·`docs/lgt_native_classes.md`·
  `docs/report-migration-revert.md`·`docs/upstream-realign-verdict.md`·`docs/web.md`)이고 ★**그중 둘이 이 20개와 겹친다**
  (`REPORT.md` 의 `grep -H '^## \[' … | sort -r` · `docs/web.md` 의 `npm run audit`).
  ⇒ ★**좁힘 «자체»는 정당하고, 틀린 것은 «전칭 주장»이다.** 그 문장을 거둔다.
  ★★**계수기 자체가 처음에 틀렸다** — `grep '^```'` 은 «들여쓴 펜스»를 못 봐 **2블록 5줄을 놓쳤다**(10 ↔ 실제 14).
  대조군(`^ *```` `)으로 잡아 바로잡았다. ⇒ ★**「전수」를 주장하려면 계수기부터 대조해야 한다.**
- **★★F1⑵ 의 핵심 발견 — 「어디에도 «명령으로» 안 도는 것」은 «러너 블록 전부»다**(★게이트② R1 로 정정):
  ★**초판은 「`make-draw-fixture` 조차 CI 에서 돈다」고 적었고 그것은 «거짓»이다.** 내가 다시 쟀다 —
  `engine-contract.yml:191` 의 `- 'scripts/make-draw-fixture.mjs'` 는 ★**`dorny/paths-filter` 의 «경로 패턴»**(:187–191 이 전부 인용 경로)이고,
  `(node|bash|sh) scripts/make-draw-fixture` 호출은 `.github/`·`scripts/`·`package.json` 전수에서 ★**0건**이다.
  CI 가 쓰는 것은 `scripts/contract-roundtrip.mjs:169` 이 **import 한 `drawFixtureJar()`**(메모리)이고,
  ★**writer 는 `:509` main-guard(`process.argv[1] === import.meta.url`) 안의 `:511 writeFileSync` — 정확히 «안 도는 부분»이다.**
  ★★**저장소 정본이 이미 옳게 분류하고 있었다** — `wie_cli/tests/support/dod_ci_parity.rs:160` 「**메모리로 만들어 쓰는** make-draw-fixture」.
  ⇒ ★**초판이 그 분류에서 «퇴행»했다.** 근인은 계수기가 아니라 ★**«분류기»를 대조하지 않은 것**이다(`grep -rl` 히트를 «실행»으로 읽었다).
  ⇒ 미커버 = ★**러너 블록 3명령 전부**(`make-draw-fixture` · `wie_validate` ×3 · `--inject` ×2).
  `smoke_gate.sh` 0건은 ★**제약 9 로 «구조적 불가»**(기결정).
  ⇒ ★**헤드라인 「빈틈은 한 블록」은 «오히려 강해진다»** — 단 ⒟의 «가격»이 달라진다(아래).
- **★그러나 «두 번째» 빈틈이 남는다(더 미묘하다) — ★크기는 «최소 6건»이다**: 문서는 **호출 별칭**을, CI 는 **실체**를 돈다
  ⇒ ★**개명해도 CI 는 green 이고 «문서만» 죽는다.** 천장 ③ 이 「문서−CI · 이름만 다름」으로 잰 그 항목이고 ★**실행만이 가른다**.
  ⒜**npm 별칭 4**: `npm run audit`↔`bash scripts/audit-no-leak.sh` · `build:wasm`↔`bash scripts/build-wasm.sh` ·
  `frontend`↔`npm ci`+`npm run build` · `verify`↔`node scripts/verify-browser.mjs`.
  ⒝★★**[게이트② 추가 · R4] beta 축 2**: `rustup toolchain install beta --component clippy`·`cargo +beta clippy` 는
  워크플로 리터럴 ★**0건**이고, CI 는 `rust.yml` 의 `matrix.rust: [stable, beta]`(+`toolchain: ${{ matrix.rust }}`)로 **다른 표현으로 같은 일**을 한다.
  ⇒ ★**매트릭스에서 `beta` 를 빼면 CI 는 green 이고 «문서만» 죽는다 — 별칭 층과 «동형»이다.**
  ★**초판은 이 둘을 「나머지 18종은 이미 CI 에서 돈다」로 접었다.** ⇒ 층의 크기 = **npm 4 + beta 2 = 최소 6**,
  ★**그리고 그 크기가 곧 ⒝의 대가(「워크플로가 실체가 아니라 별칭을 불러야 한다」)의 크기다.**
- **★F1⑶ 표본 실행(무해·저비용만 · 12종 전건 rc=0)**: fmt 25s · clippy 28s · clippy(wasm) 14s · `cargo test --all` **219s** ·
  beta clippy 7s · audit 27s · grep 3s · coverage 8s · serial 3s · engine-contract 4s ·
  `wie_validate helloworld_ktf` **9s** · `--inject --expect-last-frame keydraw_ktf` **24s**.
  ★★**[게이트② 정정 · R3] 합 라벨**: **371 = «무해 12종 전부»의 합** · **4게이트 = 286**(초판 표가 371 을 4게이트로 라벨했다).
  ⇒ ⒜의 `~450s` 는 ★**러너 표본 33s(9+24)를 이중 계상**했다 — 371 이 이미 품는다.
  ★**재산: 371 − 33 + 러너 블록 75 = «413s» + writer**(`make-draw-fixture` 는 미측정 · 미실행).
- **★★부작용 있는 8종은 «돌리지 않았다»**(목록으로만): `rustup toolchain install beta` · `make-draw-fixture`(jar 쓰기) ·
  `build:wasm`(web/src/wasm 쓰기) · `frontend`(npm install·dist 쓰기) · `contract-roundtrip`(브라우저 내려받기) ·
  `verify`(실 Chrome + 배포 URL) · `check-worklog-coverage --record`(원장 append) · `--next-serial`(gh api).
  ⇒ 그 비용은 ★**CI 이력에서 «실행하지 않고» 쟀다**: `web.yml` 최근 5회 **217·230·231·240·407s** ·
  `engine-contract.yml` ★**필터가 접으면 11~16s · 실제로 돌면 213~221s**(같은 워크플로가 조건에 따라 **20배**).
- **★부수 실측**: `test_data/draw_j2me.jar` 는 ★**미추적**(`.gitignore:24 *.jar`) ⇒ 신선한 클론엔 **없다**.
  러너 블록의 첫 줄은 «선택»이 아니라 **선행 writer** 다 — 어떤 배치든 그것을 돌리거나 그 픽스처를 건너뛰어야 한다.
- **사용자 영향**: 없음(문서). 대신 ★**배치를 «고를 수 있게» 됐다** — 그것이 제안이 요구한 산출이다.
- **★★⒟ 의 가격표가 바뀌었다(R1 의 파생)**: ⒟는 「신설 0 · 러너 블록만 게이트③ 손 실행」인데,
  러너 첫 줄이 **선행 writer** 이므로 실제로는 ★**착지마다 «작업 트리에 쓰는» 명령을 손으로 도는 안**이다.
  ⇒ 그것은 ⒜를 탈락시킨 사유(「부작용 명령은 작업 트리를 덮는다」)와 ★**같은 반대**다. worklog `proposals[3].tradeoff` 에 명시했다.
- **★고르지 않았다**: 배치 4안과 각 대가는 worklog `proposals` 에 있다. ★**총괄·운영자가 고른다.**
