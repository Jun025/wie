## [2026-09-11] P3 첫 조각 — `wie_web` → `wie_featurephone` 크레이트 개명 (wie-p3-rename-wie-web-to-featurephone)

### 무엇을
- 크레이트/디렉터리 `wie_web` → `wie_featurephone` 개명. **순수 `git mv` 커밋**(7파일, 100% rename)과
  **참조 갱신 커밋**을 분리 — 이후 upstream 병합의 rename detection 을 살리기 위해서다.
- 참조 갱신: `Cargo.toml`(members) · `Cargo.lock`(19+/19−, 순수 치환) · `wie_featurephone/Cargo.toml`(name) ·
  `scripts/build-wasm.sh`(`-p`, `WASM_IN`) · `scripts/check-engine-contract.mjs`(lib.rs 경로 3곳) ·
  `scripts/contract-roundtrip.mjs`(크레이트명 주석) · 워크플로 3종(주석/스텝명) · `web/src/index.css` ·
  `wie_cli/src/filesystem.rs` · `wie_jvm_support/src/runtime.rs`(doc 주석) · `AGENTS.md` · `docs/architecture.md` ·
  `docs/web.md` · `docs/project-kb/01_brief.md`·`02_status.md` · `docs/ARTIFACT-PUBLISH-PLAN.md` · `STATE.md ## 다음`.

### 왜
upstream `dlunch/wie` 가 `wie-web`(lib 타깃 `wie_web`)를 자기 브라우저 앱에 쓴다 — upstream 을 base 로
삼는 순간 이름이 정면 충돌한다(`docs/upstream-realign-verdict.md` §8-4⑴). `STATE.md ## 다음` P3 가
「여러 회차로 쪼개라」고 못박아 이 회차는 **개명까지만** 한다(오버레이 재적용·`compile_model.rs` 이식·
엔트리포인트 규약 정합은 후속).

### 핵심 결정 — 산출물 쌍 이름은 «일부러» 유지
`wie_web.js`/`wie_web_bg.wasm` 은 upstream 과 충돌하는 이름이 아니라 **otterpebble 소비자 계약**이다
(`docs/contracts/featurephone-engine-contract.json` `files`·`glueFetchesWasmByName`; 리시버
`wie-artifact-receive.yml` 이 릴리스 자산을 **이름으로** curl). `build-wasm.sh` 의 `--out-name wie_web` 이
크레이트명과 산출물명을 분리한다(스크립트에 why-주석). 산출물 개명은 wie+otterpebble 교차 repo 조율
회차다 — 워크로그 제안으로 남겼다.

### 실측
- 잔존 `git grep -F 'wie_web'`: **120건** — 전건 의도 잔존 3부류: ⑴산출물 이름(계약·워크플로·스크립트·
  `emulator.ts`) ⑵사료(`docs/report/**`·`docs/worklog/**`·`docs/upstream-realign-verdict.md`·`STATE.md ## 완료`·
  `02_status.md:245`) ⑶「개명했다」는 기록 자체.
- 4게이트 + beta: fmt OK · clippy stable/wasm/beta 전건 rc=0 · `RUST_MIN_STACK=4194304 cargo test --all`
  42스위트 전건 ok(실패 0).
- 러너 블록: draw_j2me/helloworld_ktf/helloworld_lgt **PASS** · keydraw_ktf/lgt `--inject --expect-last-frame`
  **PASS·rc=0** (5/5).
- `bash scripts/build-wasm.sh` 실빌드: `Compiling wie_featurephone` → 산출물 `wie_web.js`+`wie_web_bg.wasm`
  그대로 → `check-engine-contract.mjs` **107 pass / 0 violation**. `npm run audit` PASS.
- 대조쌍: members 를 `"wie_web"` 로 되돌리면 `cargo metadata` **rc=101**(manifest 로드 실패) — 복원 확인.

### 사용자 영향
없음(동작·산출물·배포 경로 불변). featurephone 셸이 받는 아티팩트 이름·표면 무변경.

### 계보 DoD 인계
`git merge-base origin/main upstream/main` = `fa641a8a` — 이 회차는 upstream 커밋을 머지하지 않으므로
움직일 수 없는 값이다. **이 repo 의 게이트③은 `--merge` 강제**(`upstream-sync-repos.conf` 등재 ·
queue-lint 검사22). merge-base ≠ `fa641a8a` DoD 는 «upstream 을 base 로 얹는 후속 조각»의 게이트③에
실어야 한다.
