## [2026-11-06] upstream/main 2차 동기화 예행·분류 — 충돌 93 분류 · 예행 트리 게이트 red · 조각 계획 (wie-upstream-sync2-dry-run-classify)

**무엇을**: 격리 워크트리에서 `git merge upstream/main` 을 예행했다. 충돌 파일 **93개**를 조각 B 형식(폐기·재적용·발신)으로 전수 분류했다(미분류 0). 예행 트리에서 4게이트를 돌렸고 결과는 **전부 red** 다(§5). 실제 병합을 조각으로 나눈 계획과 조각별 검증식을 적었다(§7). 예행 트리는 push 하지 않았고 지웠다. 이 PR 은 문서만 바꾼다.

**왜**: 티켓 `wie-upstream-sync2-dry-run-classify`(총괄 2026-11-06 채택 · 근거 `reports/wie-state-next-refresh-and-next-round-proposal.done.md` 후보 1). merge-base `44fbf265` 가 2026-09-16 이후 그대로다. behind 는 173 이다.

**사용자 영향**: 없다. 제품 코드 0줄이다. 이 문서는 다음 병합 회차가 «무엇을 버리고 무엇을 살릴지»를 미리 정해 둔다.

### 0. 결론 — go/no-go

1. ★**«한 번에 병합»은 no-go 다.** 비용을 정하는 것은 behind 커밋 수(173)가 아니다. upstream 이 갈아엎은 API 세 축이다.
   - ⑴ RustJava `^0.1.1 → ^0.4.1`(#1430 · #1444 · #1489 · #1542). 필드 접근에 owner 인자가 붙었다. Java 클래스 파일 충돌 대부분의 원인이 이것 하나다.
   - ⑵ `wie-backend` executor 재설계(#1429 · #1491). 예행 트리가 **맨 처음 깨지는 곳**이 여기다(§5).
   - ⑶ ARM AOT(#1415). 이 커밋이 `wie-core-arm` 의 `binary_patches` 를 **통째로 지웠다**. 우리 10월 잠금 해제(0540 · 0547)가 사는 곳이다.
2. ★**조각 계획은 go 다 — S1(RustJava 핀을 우리 쪽에서 먼저 0.4.1 로) 부터.** 우리 쪽만 바꾸는 조각이라 upstream 과 엮이지 않는다. 가장 큰 단일 원인을 병합 전에 치운다(§7).
3. **수**: 폐기 **11** · 재적용 **53** · 발신 **29** = **93**. 해소 방식은 3-way 코드 수정 **67** · ours 8 · union 7 · theirs 9 · 재생성 2.
4. **10월 벽·잠금 행(0539~0548)과 겹치는 파일은 19개**다(재적용 16 · 폐기 2 · 발신 1 · §3-B).
5. **예행 트리 게이트는 전부 red 다.** 충돌 표식 트리는 `Cargo.toml:160` 파싱에서, ours·theirs 탐침은 둘 다 `wie-backend` 컴파일에서 죽는다. 그래서 runner 블록과 `smoke_gate` «후»는 잴 수 없다(UNMEASURED). `smoke_gate` «전»(`origin/main`)은 lgt 52 · skt 50 · ktf 190 대조에서 회귀 1건이 났다. 그 1건은 부하에 굶은 것이다(§5-B).
6. ★**충돌 93 만 보면 모자란다.** 양쪽이 다 고쳤는데 git 이 조용히 합친 파일이 **35개**(upstream 이름바꿈 너머까지 세면 36) 더 있다. 그중 `wie-backend/src/system.rs`·`task.rs` 가 컴파일 오류를 낸다(§6).

### 1. 측정 트리

2026-11-06 11:5x~12:1x KST, `git fetch origin && git fetch upstream` 직후에 쟀다.

```sh
git rev-parse origin/main upstream/main     # 2e66f9d68… · 10c12ec58…
git merge-base origin/main upstream/main    # 44fbf2652c856af11c7140f838ad223cdebab641 (09-16 이후 불변)
git rev-list --count origin/main..upstream/main   # 173 (병합 제외 172)
git rev-list --count upstream/main..origin/main   # 2970
git worktree add --detach /tmp/wie-sync2-probe origin/main && cd /tmp/wie-sync2-probe
git merge --no-commit --no-ff upstream/main       # rc=1
git status --porcelain | awk '{print substr($0,1,2)}' | sort | uniq -c | sort -rn
#   108 'M ' · 96 'A ' · 82 UU · 7 AA · 5 'R ' · 4 UD · 4 'D '
```

미해결은 UU 82 + AA 7 + UD 4 = **93** 이다. 0550 이 `git merge-tree --write-tree --name-only` 로 잰 값과 같다.

'D ' 4 는 충돌 없이 사라지는 파일이다: `wie-core-arm/src/binary_patches/{hook,parser,patch}.rs` 3 + `wie-web/public/GeneralUser.sf3` 1(upstream `cfec8186a` #1460 이 지웠고 우리는 고치지 않았다). 넷 다 §6 «떨어뜨린 것»과 S3 ⑴·⑪ 에 있다. ('R ' 5 는 upstream 이름바꿈이다 — `src/{audio_sink,database,filesystem,window}.rs → src/cli/…` 와 `configure-android-signing.sh → configure-android-release.sh`.)

★**upstream 은 이 측정 뒤 2커밋 전진했다**(`10c12ec58` → `69ca6f07d`: `e169a90e0` #1570 RGB/RMS/media/IPv4/SKVM API · `69ca6f07d` 0.2.20). #1570 은 media·IPv4 를 건드려 발신·폐기 행(#72 `util.rs` · media 계열)과 겹칠 수 있다. **S3 착수 시 이 표 전체를 새 `upstream/main` 에서 다시 잰다.**

예행 트리 3개(`/tmp/wie-sync2-{probe,ours,theirs}`)는 회차 끝에 `git worktree remove --force` 로 지웠다. 이 저장소 diff 에 남지 않는다. push 는 0 이다.

### 2. 분류 술어 — 조각 B(`docs/upstream-realign-p3-slice-b-triage.md` §2)를 그대로 쓴다

| 갈래 | 술어 |
|---|---|
| **폐기** | 이 파일에서 우리가 더한 것이 **전부** upstream 에 이미 있다(같은 심볼·proto·동작, 또는 그것을 대체하는 upstream 판). 해소는 theirs 다. 잠금 파일(`Cargo.lock`·`package-lock.json`)도 폐기로 센다. 병합한 매니페스트에서 다시 만든다. |
| **발신** | upstream 에 없는 우리 몫이 **공개 WIPI/MIDP/SKVM API 스텁이나 게임에 묶이지 않은 엔진 수정뿐**이다. 타이틀별 패치·인증 우회·가짜 서버·역공학한 타이틀 자료는 들지 않는다. 병합에서는 살리고, 나중에 upstream 에 제안할 수 있다. |
| **재적용** | 위 둘이 아닌 우리 몫이 남는다(포크 CI·하드닝·시험·결정성·잠금/서버 벽·census 훅 등). 병합이 **반드시** 살려야 한다. |

파일 단위 규칙도 같다. 우리 델타가 «전부» upstream 에 있을 때만 폐기다. 남은 몫이 전부 발신 급이면 발신이다. 그 밖은 재적용이다. 일부만 수렴한 파일은 표의 델타 칸에 «반쯤 수렴»으로 적었다.

분류는 파일마다 `git diff 44fbf265 origin/main -- <p>` 와 `git diff 44fbf265 upstream/main -- <p>` 를 열어서 했다. 폐기 행의 근거 칸은 `git show upstream/main:<p>` 에서 그 심볼을 찾은 자리다. 93개를 영역별 4묶음(스캐폴딩·backend·core 29 / ktf·lgt 26 / midp·skvm·wipi-c 18 / wipi-java 20)으로 나눠 읽었다. 경로 집합은 충돌 목록과 바이트 대조로 같다(93 = 93).

### 3. 수

#### 3-A. 갈래 × 영역

| 영역 | 파일 | 폐기 | 재적용 | 발신 |
|---|---:|---:|---:|---:|
| 스캐폴딩·루트·backend·test-utils·core-arm·j2me·jvm-support·skt·web | 29 | 4 | 22 | 3 |
| `wie-ktf` · `wie-lgt` | 26 | 1 | 24 | 1 |
| `wie-midp` · `wie-skvm` · `wie-wipi-c` | 18 | 1 | 6 | 11 |
| `wie-wipi-java` | 20 | 5 | 1 | 14 |
| **합계** | **93** | **11** | **53** | **29** |

해소 방식: 3way-edit **67** · ours 8 · union 7 · theirs 9 · regen 2 = 93.

★**발신 29 의 거의 전부가 «3way-edit» 다.** 내용은 공개 API 급인데, upstream #1444 가 필드 접근 서명을 바꿔서 손을 대야 한다. S1 이 그 원인을 먼저 치운다(§7).

★**조각 B 의 발신 후보 6종은 이번에 1종만 남았다.** `InputMethodListener`·`GrabKeyListener`·`LabelComponent`·`MediaUnsupportedException` 은 upstream #1569(11-06)·#1519 판이 우리 것을 포함한다(폐기). `ActionListener` 는 이번 충돌 목록에 없다. `FormComponent` 만 우리 몫(`setFocus(Component)` · `vertical` 필드)이 남아 발신이다.

#### 3-B. 10월 벽·잠금 행(0539~0548)과 겹치는 파일 — 19

각 회차의 착지 커밋(first-parent, 제목의 `(05NN)`)에서 `git diff --name-only <c>^1 <c>` 로 파일을 모았다. 문서·`test_data/` 는 뺐다. 그 집합과 충돌 93 의 교집합이다.

| 갈래 | 수 | 파일(회차) |
|---|---:|---|
| 재적용 | 16 | `wie-backend/src/lib.rs`(0542) · `wie-core-arm/data/binary_patches.toml`(0540·0547) · `wie-core-arm/src/binary_patches.rs`(0540·0547) · `wie-core-arm/src/lib.rs`(0540) · `wie-jvm-support/src/jvm_implementation.rs`(0546) · `wie-jvm-support/src/lib.rs`(0546) · `wie-skt/src/emulator.rs`(0539·0545) · `wie-ktf/src/emulator.rs`(0539·0545) · `wie-ktf/src/runtime/wipi_c.rs`(0541) · `wie-ktf/src/runtime/wipi_c/method_table.rs`(0541) · `wie-lgt/src/emulator.rs`(0545) · `wie-lgt/src/runtime/svc_ids.rs`(0540·0542) · `wie-lgt/src/runtime/wipi_c.rs`(0542) · `wie-wipi-c/src/api/database.rs`(0547) · `wie-wipi-c/src/api/graphics.rs`(0541) · `wie-wipi-c/src/api/net.rs`(0542·0548) |
| 폐기 | 2 | `Cargo.lock`(0539·0546 · 재생성) · `wie-lgt/src/runtime/stdlib.rs`(0540·0542) |
| 발신 | 1 | `wie-wipi-c/src/api/kernel/sprintf.rs`(0542 · 정렬·정밀도) |

- ★**`wie-lgt/src/runtime/stdlib.rs` 는 10월 행인데 폐기다.** 0540 의 `0x40b strchr` 와 0542 의 `0x40a strncmp` 를 포함해 우리 슬롯 8개가 전부 upstream `stdlib.rs` L22-51 에 있다. 우리 `0x424 Unk3` 는 upstream 의 `Atexit` 이다. 판정은 **이름 수준**이다. upstream 에서 `strchr` 시험은 찾지 못했다. S3 가 그 슬롯을 시험으로 확인한다.
- ★**0543 · 0544 는 겹치지 않는다**(바꾼 파일이 `scripts/playability-census.mjs`·`wie-backend/src/fake_server.rs` 로, 둘 다 upstream 이 건드리지 않았다).
- ★**0546 의 `class_patches.rs`, 0541 의 `local_net.rs`, 0542 의 `fake_server.rs`·`system.rs` 는 충돌 목록에 없다.** 그런데 `system.rs` 는 양쪽이 다 고친 «조용한 병합» 35개 중 하나이고 컴파일 오류를 낸다(§6).

### 4. 분류표 — 93행 · 미분류 0

열: 상태(머지 상태 코드) · 판정 · 해소 · 10월(겹치는 회차) · 우리 델타 · upstream 근거와 비고. 경로는 머지 상태 기준(하이픈)이다. 칸은 짧게 줄였다. 근거는 전부 이 회차에 이 트리에서 연 것이다.

| # | 상태 | 경로 | 판정 | 해소 | 10월 | 우리 델타 | upstream 근거 · 비고 |
|---:|---|---|---|---|---|---|---|
| 1 | UU | `.github/workflows/rust.yml` | **재적용** | ours | — | fork CI: header pointer to AGENTS gates, fail-fast:false, RUST_MIN_STACK split steps, apt best-effort, wasm clippy w/o -p | theirs: concurrency group, os xcode-27 (was macos-latest), iOS lint step, wasm clippy -p wie-web, cache save split · Do NOT take xcode-27 rename: required check «rust_ci (macos-latest, stable)» would vanish -> merge deadlock. Concurrency block is safe to absorb. Recheck wasm clippy scope vs new native crates. |
| 2 | AA | `.github/workflows/tag-release.yaml` | **재적용** | 3way-edit | — | PARKED header (not adopted, DO NOT DISPATCH) over an older upstream body | upstream/main body: ADMIN_GH_TOKEN checkout, taiki-e/install-action cargo-edit, step renamed · Take theirs body + re-prepend our PARKED header; header line «tag-version.sh not in our tree» becomes false after this merge — reword. check-parked-workflows.mjs must stay green. |
| 3 | UU | `.github/workflows/web.yaml` | **재적용** | ours | — | triggers parked to workflow_dispatch + NOT ADOPTED header (dup of fork web.yml) | theirs: reopened type, permissions, concurrency, build:prod/dev, PR preview upload + deploy_preview job · Keep parked; upstream preview deploy needs CLOUDFLARE secrets and npm build:* scripts we do not have. |
| 4 | UU | `AGENTS.md` | **재적용** | ours | — | fork-owned agent charter (constraints table, DoD gates, ledger) ~800 lines | theirs: +Attribution section, +type-path/modules/visibility rules, +engineering principles (pid namespace, logging, invoke_* rule) · Fork file wins. Human decision: upstream Attribution clause is «mandatory agent instruction» — consider carrying a pointer. 850-line cap applies. |
| 5 | UU | `Cargo.lock` | **폐기** | regen | 0539,0546 | lock for jvm 0.1.1, md5, classfile 0.1.1 (patched), wasm-bindgen 0.2.108 | upstream lock: jvm 0.4.1, classfile 0.4.1, wasm-bindgen 0.2.129, sha2, zip 9 · Regen from merged manifests. Risk: wie_featurephone =0.2.108 pin forces single 0.2.x for whole graph (upstream wie-web deps may need newer); classfile patch 0.1.1 will be «not used» under 0.4.1. |
| 6 | UU | `Cargo.toml` | **재적용** | 3way-edit | — | members +wie_cli +wie_featurephone +patches/classfile; exclude wie-app (+rationale); [patch.crates-io] classfile; keeps md5 dep, jvm ^0.1.1 | theirs: +wie-core-arm-{aot,wasm,native}, version 0.2.19, jvm/rustjava ^0.4.1, md5 dropped, zip ^9 ws dep, cli feature on root bin, wie path dep · Union members (ours + 3 AOT crates), keep exclude wie-app, re-add md5 (backend, core-arm patches, jvm-support, lgt, skt). Re-vendor classfile patch at 0.4.1 or it silently stops applying. |
| 7 | UU | `README.md` | **재적용** | ours | — | fork «Web service (this fork)» section (web/, Pages Functions, D1) | theirs: «apps» wording, Xcode 27 note, PR preview paragraph (wie-dev Pages) · Keep ours; optionally absorb the one-word «games->apps» intro edit. Preview paragraph describes upstream infra, not ours. |
| 8 | UU | `package-lock.json` | **폐기** | regen | — | lock of fork ops package (wrangler, playwright, esbuild, spessasynth_core) | upstream lock of webpack frontend (@sentry 11, @tauri-apps/api) · Regenerate from OUR package.json (effectively ours); upstream lock belongs to a package.json we do not use. |
| 9 | UU | `package.json` | **재적용** | ours | — | replaced with private wie-web-ops (frontend/build:wasm/deploy/audit/verify scripts, wrangler/playwright devDeps) | theirs: bumps @sentry/browser 11, +@sentry/wasm, +@tauri-apps/api, webpack 5.112, +sentry plugin · Fork-owned. Upstream's wie-web/ webpack build stays unbuildable here (same as today; web.yaml parked). |
| 10 | UU | `test-utils/src/lib.rs` | **재적용** | union | — | exports run_jvm_test_until_exit, run_jvm_test_with_runner, TestScreen, guest_line_complete | theirs: +mod native_executor, export TestNativeExecutor (lib.rs:6,12) · Union both export lists. |
| 11 | UU | `test-utils/src/platform.rs` | **재적용** | 3way-edit | — | stepping TestClock (deterministic), speed, audio_log Vec, paint_counter/redraw_flag TestScreen, guest_line_complete | theirs: audio_handler callback + with_audio_handler, TestScreen paint_handler + with_paint_handler, removed DatabaseRepository::usage · Half converged (both made audio sink/screen observable, different shapes). Keep ours + add theirs with_* constructors; drop usage() per trait change. |
| 12 | UU | `wie-backend/Cargo.toml` | **재적용** | union | — | +md5 (owner_files.rs:279 hash lookup) | theirs: +serde, zip -> workspace ^9, smaf 0.1.4, +wie-core-arm-aot, +test-utils dev · Union; keep md5 (needs workspace md5 restored). zip 8->9: verify our zip_entry/extract_zip code compiles. |
| 13 | UU | `wie-backend/src/canvas.rs` | **재적용** | 3way-edit | — | encode_bmp(region), LBMP len guard, ~10 pixel/decoder tests (incl. LBMP type-2 test) | upstream canvas.rs:963 encode_bmp(image) + caller crops region (wie-wipi-c graphics.rs:295); decode_image starts_with(b"LBMP"); +indexed BMP transparency · Code converged -> take theirs. Only our tests remain; port them, but test_decode_lbmp_rgb332 asserts bpp=1 while upstream LBMP now returns ArgbPixel (bpp 4) — fix assertion. |
| 14 | UU | `wie-backend/src/canvas/lbmp.rs` | **폐기** | theirs | — | truncated-header guard, type 2 2-bit gray planes + mask decoder | upstream lbmp.rs decode_lbmp: data.get(..24) guard; kind 2 high=plane0 low=plane+size, gray=255-(h*2+l)*85, mask bit->alpha 0; truncated_lbmp test · Same plane order/levels/mask semantics as ours. Upstream also rejects mask>1 and size mismatch (stricter). |
| 15 | UU | `wie-backend/src/executor.rs` | **재적용** | 3way-edit | — | BTreeMap spawn-order determinism, TICK_BUDGET_MS 14/tick_for, paced sleep_toward, wait_until inside budget, halt/clear, wants_cpu/next_due, AtomicBool halted | theirs: TaskWake waker, loop exits when no wake, sleep(until: Instant) computed at call time (#1491), last_now removed, HashMap kept · Highest-risk merge: both replaced noop waker with Wake differently; upstream's «break if not woken» changes tick semantics our pacing/achieved_speed rely on; sleep API changed under all callers. |
| 16 | UU | `wie-backend/src/lib.rs` | **재적용** | 3way-edit | 0542 | mods fake_server/owner_files/frame_pacer/pacing/speed_clock; Emulator::tick_for/take_pacing; zip_entry, reroot_at_marker, strip_common_wrapper_dir | theirs: mod storage_request, Emulator::is_preparing, Options.aot (CompiledExecutor), normalize_guest_path export, zip name error handling · Union exports + trait methods (all emulators must impl is_preparing AND tick_for/take_pacing). Take theirs entry-name handling inside our extract_zip. |
| 17 | UU | `wie-backend/src/system/file_system.rs` | **발신** | union | — | FilesystemOverlay::virtual_file() (archive bytes ignoring guest writes) + 2 tests | theirs: aid->pid rename throughout, pub normalize_guest_path, +remove(), +usage() · Generic accessor; add to upstream's struct (use pid). Risk: aid->pid changes the persistent save namespace for our web/featurephone hosts — needs a migration decision. |
| 18 | UD | `wie-core-arm/data/binary_patches.toml` | **재적용** | ours | 0540,0547 | +3 hash-keyed entries: LGT 01f05f8231f4 owner check (2 patches), KTF 974e0df9ab1e/f770b15f8876 install licence (3 patches) — reports 0540/0547 | deleted by 223782022 (ARM AOT replaces binary patches); no equivalent table upstream · Keep. Also restore staged-D binary_patches/{hook,parser,patch}.rs, lib.rs mod line, md5/serde/toml in core-arm Cargo.toml (auto-merge DROPPED them); callers ktf init.rs:92, lgt init.rs:188. |
| 19 | UU | `wie-core-arm/src/allocator/bucket.rs` | **발신** | 3way-edit | — | free() resolves bucket from address (host panic fix), size mismatch warn, usage() report, test | theirs: free single-byte header read (same as ours), alloc scans header in 256B chunks, tests use ArmCore::new(Options) · Half converged (header read). Our address-based free is a generic engine fix. Port test to ArmCore::new(wie_backend::Options{..aot: None..}). |
| 20 | UD | `wie-core-arm/src/binary_patches.rs` | **재적용** | ours | 0540,0547 | generic+title entries stacked in one install (single SVC dispatcher), install_title_binary_patches (LGT, no generic), test | deleted by 223782022; upstream lib.rs has no binary_patches export · Keep. Install before prepare_execution/AOT begin (AOT compiles from guest memory) or rely on AOT write invalidation; hooks plant SVC - verify AOT exits. Tests: ArmCore::new(Options). |
| 21 | UU | `wie-core-arm/src/core.rs` | **재적용** | 3way-edit | — | guest_root_words (incl. nested saved contexts), hold_others/preempted, teardown hooks, live_cores, SVC stub high-water, shared stubs, thread-id tracking, tests | theirs core.rs:199 stack_words (regs+live stacks, no nested contexts), new(Options), prepare_execution/is_preparing, run_function_sync, AOT · Half converged on GC root scan; ours also covers contexts put aside by nested calls. Keep both or fold ours into stack_words. ArmCore::new signature change ripples to all tests. |
| 22 | UU | `wie-core-arm/src/engine/arm32_cpu.rs` | **발신** | 3way-edit | — | guest null-page READ (<0x1000) yields 0, log once; undefined-insn message with insn bytes + lr; branch-to-null logs lr; tests | theirs arm32_cpu.rs:169 «Undefined instruction at {pc} (CPSR)»; AOT dispatch, MemoryPage pages, icache invalidation · Diagnostics half converged. Null-read rule must be applied to the AOT MemoryAccess path too, else compiled code faults where interpreter reads 0. Memory struct rewritten upstream -> re-port read_page. |
| 23 | UU | `wie-core-arm/src/lib.rs` | **재적용** | 3way-edit | 0540 | exports binary_patches::{install_binary_patches, install_title_binary_patches}, pub mod stdlib, SVC_STUB_CAPACITY/live_cores/svc_stub_high_water | theirs lib.rs: mod aot, no binary_patches/stdlib · Auto-merged part ALREADY dropped «mod binary_patches; pub mod stdlib;» — re-add both by hand, not just the conflicted export line. |
| 24 | UD | `wie-core-arm/src/stdlib.rs` | **재적용** | ours | — | +memmove (overlap-safe) | deleted upstream; memmove/strchr moved to wie-lgt/src/runtime/stdlib.rs:162/:297 (returns u32) · Our memmove alone is covered upstream, but the file must survive: binary_patches/hook.rs:6,262-328 calls crate::stdlib memcpy/memset/strcpy/strlen. Coordinate with LGT stdlib resolution. |
| 25 | UU | `wie-j2me/Cargo.toml` | **폐기** | theirs | — | [dev-dependencies] test-utils | upstream wie-j2me/Cargo.toml: same test-utils dev-dep + zip · Fully converged. |
| 26 | UU | `wie-jvm-support/src/jvm_implementation.rs` | **재적용** | 3way-edit | 0546 | InheritedMethods (invokespecial walks superclasses), Preemptible yield between calls, ForcedBool class patch hook, tests | theirs: JvmImplementation::external_roots() default (RustJava 0.4 ExternalRoots) · Must be re-ported onto jvm 0.4.1 traits (ClassDefinition/Method signatures likely moved). Check whether 0.4 invoke_special already searches superclasses before keeping InheritedMethods. |
| 27 | UU | `wie-jvm-support/src/lib.rs` | **재적용** | 3way-edit | 0546 | mods class_patches/declared_field/guest_roots/hardening; RT_RUSTJAR on classpath; SeverOnDrop; finish_launch; fallible to_wie_err | theirs: external_roots passed to Jvm::new, push_native_frame GC-roots exception in to_wie_err, JavaValue import · Combine our fallible trace + their push_native_frame. Our irrefutable «let JavaError::JavaException» assumes 1 variant - verify on 0.4.1; also RT_RUSTJAR. guest_roots vs ExternalRoots half converged. |
| 28 | UU | `wie-skt/src/emulator.rs` | **재적용** | 3way-edit | 0539,0545 | SYSTEM_PROPERTIES (+m.MODEL wie, +m.EXT_SW), package display size from jar (incl. md5-keyed override row), owner sound, pacing/tick_for | theirs: m.MIN 00000000, m.MODEL "" (d1fabf563), Emulator::is_preparing false · Half converged on m.MODEL (value differs: ours non-empty to avoid .equals on null — empty also non-null). Decide m.MIN/m.MODEL values; add is_preparing. Per-title override stays fork-only. |
| 29 | UU | `wie-web/src/rust/lib.rs` | **재적용** | 3way-edit | — | FramePacer: tick_for(budget) + pacer.end per frame | theirs: rewritten host (WasmHost, aot, save_migration, is_preparing), tick at lib.rs:253 · wie-web is upstream's host, not our shipped artifact (wie_featurephone). Re-wire 3 pacer lines at the new tick, or deliberately drop — must still compile against our Emulator trait. |
| 30 | UU | `wie-ktf/src/adf.rs` | **재적용** | union | — | is_relocation_prefixed() detector for relocation-table-prefixed client.bin (feeds our runtime::relocated) + test | upstream adds validate_client_bin() (adf.rs ~L81) that REJECTS anything not starting 04e0c046...; no relocated support upstream · Keep both fns, but upstream validate_client_bin/validate_jar (emulator.rs) would reject our 3 relocated KTF images: must let is_relocation_prefixed pass validation |
| 31 | UU | `wie-ktf/src/emulator.rs` | **재적용** | 3way-edit | 0539,0545 | per-title owner-save drop lists (jar MD5 keyed), jar-name fallback to sole .jar, private dir mount, owner-sound-on, pacing/tick hooks | upstream: validate_archive/validate_jar, ArmCore::new(Options), is_preparing, dropped dump_reg_stack in tick; none of ours present · Game-specific lists; ArmCore::new signature changed; validate_jar uses strict <AID>.jar lookup that contradicts our jar fallback |
| 32 | UU | `wie-ktf/src/runtime/init.rs` | **재적용** | 3way-edit | — | JavaThrowInstance SVC, guest_roots native frames/stress_collect, relocated::load + MNInterface, sdk_runtime (wide gfx ctx) flag, init_param_4() + test, non-UTF8 name error | upstream #1545 wires fn_visit_gc_root + InitSvcId::VisitGcRoot=7 and push_native_frame; different GC-root design · Half converged (both root KTF native refs, different mechanisms). InitSvcId 7 collides (ours JavaThrowInstance vs theirs VisitGcRoot) - renumber ours |
| 33 | UU | `wie-ktf/src/runtime/java/interface.rs` | **재적용** | 3way-edit | — | call_native keeps J/D high word + thread-context result tag, throw null->NPE, java_throw_instance, null-jump diagnostics by lr, per-call native frame | upstream: push_native_frame gated on ptr_visit_gc_root, map_jump_result(&caller) restore, dropped nativeStrings Vector; call_native still writes 0 to ptr_data+4 (L320-321) · Half converged on frames/context restore. J/D high-word fix is generic and ours-only (발신 candidate in isolation) |
| 34 | UU | `wie-ktf/src/runtime/java/jvm_support.rs` | **재적용** | 3way-edit | — | relocated ABI, class-record region allocator, exception scopes/handler offset, instantiation_error/wie_error (no-panic OOM), guest-root GC tests, net slot 34 test | upstream #1542/#1545: RustJava 0.4 adapters, KtfJvmSupportContext.ptr_visit_gc_root, ExternalRoots; none of our fns present · Largest KTF conflict; RustJava ^0.4 API changes under our code; two GC-root designs must be reconciled |
| 35 | UU | `wie-ktf/src/runtime/java/jvm_support/class_definition.rs` | **재적용** | union | — | class record from KtfJvmSupport::alloc_class_record; instantiate error via instantiation_error | upstream rewrote method/field lookup to walk raw descriptors (JavaFullName::matches), Cow names · Different regions; depends on our jvm_support helpers |
| 36 | UU | `wie-ktf/src/runtime/java/jvm_support/jvm_implementation.rs` | **재적용** | 3way-edit | — | on_teardown empties java_functions (Arc cycle); define_class/define_array_class return guest exception instead of unwrap panic | upstream adds ExternalRoots impl (visit_gc_root + stack scan) and still .unwrap()s nothing of ours · Hardening; on_teardown is our ArmCore API (absent upstream) |
| 37 | UU | `wie-ktf/src/runtime/java/jvm_support/method.rs` | **재적용** | 3way-edit | — | relocated enter/leave around run; exception handler chain walk (4096 cap), writes e slot + target pc, relocated name-table class match | upstream writes thrown object to ptr_exception_class_instance (same as our unk3 write) and empty-table early return; no chain walk · Half converged (catch e-slot write). Chain walk + pc rewrite are generic fixes but entangled with relocated ABI |
| 38 | UU | `wie-ktf/src/runtime/svc_ids.rs` | **재적용** | 3way-edit | — | InitSvcId::JavaThrowInstance=7, UserMem=18 table id, ktf_kernel_extension_message + 2 tests | upstream InitSvcId::VisitGcRoot=7 (L13); Database table renamed Fs (WIPICFsMethodId, Fs=7) · Id 7 collision must be renumbered; UserMem/Reserved4 extension still ours-only |
| 39 | UU | `wie-ktf/src/runtime/wipi_c.rs` | **재적용** | 3way-edit | 0541 | mod local_net (fake server), ResourceCache, wide_graphics_context flag, Reserved4 -> get_extension_interface, ScreenFramebufferSync compositor | upstream +115 lines in same handler (with_arm_context, fs/im routing); set_screen_compositor/local_net/ResourceCache absent upstream (git grep 0 hits) · Fork infra + fake server |
| 40 | UU | `wie-ktf/src/runtime/wipi_c/context.rs` | **재적용** | 3way-edit | — | ResourceCache, wide graphics context, db slot 11 free storage, interface4 slot 0, free(NULL) no-op, throw_instance + tests | upstream: run_function maps errors with with_arm_context; collect_garbage error -> FatalError; new timer fault test · Partial overlap only in run/collect_garbage error mapping |
| 41 | UU | `wie-ktf/src/runtime/wipi_c/method_table.rs` | **재적용** | 3way-edit | 0541 | MXUserMem arena table, KTF net extension slots (net_open_by_host), unnamed-table/kernel-extension stubs with messages, tests | upstream renamed database->fs interface (get_fs_interface, WIPICFsInterface), added im, util::htonl body · Our rows must be re-laid onto renamed tables |
| 42 | UU | `wie-lgt/data/lgt_java_abi.toml` | **발신** | union | — | +73 vtable rows (Object/String/StringBuffer/Vector/Thread/Timer/DataInputStream...) with derivation comments | upstream +26 rows; 25 of our 73 identical upstream, 0 index disagreements; upstream-only row OutputStream write(I)V 10 · Half converged. 48 rows ours-only, all standard CLDC/MIDP class ABI (no title data) - but comments name titles, scrub before offering |
| 43 | UU | `wie-lgt/src/emulator.rs` | **재적용** | 3way-edit | 0545 | ORPHAN_ENTRIES per-title (jar MD5) save-flag drops, owner files/sound, pacing hooks, tests | upstream: run(), is_preparing, ArmCore Options, exception_frames_are_restored_per_thread test · Game-specific lists |
| 44 | UU | `wie-lgt/src/runtime/java/classes/net/wie/lgt_class_loader.rs` | **재적용** | 3way-edit | — | outOfMemoryError static field, find_generated_class helper, unreadable exception ptr -> WieError instead of panic | upstream: get/put_field with owner class (RustJava 0.4), with_arm_context on error · Must adopt owner-class field API |
| 45 | UU | `wie-lgt/src/runtime/java/exception.rs` | **재적용** | 3way-edit | — | per-thread exception-frame ledger, host-error mark/release, catch/rethrow semantics + ~12 tests | upstream #1532: task-owned exception state (init/destroy per task, fixed ABI cell points at active task) · Half converged: both isolate frames per thread, different designs - pick one, keep our tests |
| 46 | UU | `wie-lgt/src/runtime/java/interface.rs` | **재적용** | 3way-edit | — | imports 0x26 RaiseClassCast + 0x64 GetInterfaceMethodTable, long-array handlers, wide-field links, GC/clear_guest_registers tests | upstream maps 0x5b LoadLongArray + 0xfd StoreLongArray (L51,L58) with NPE checks, wide-field high/low slot linking, save/restore context on unwind · Half converged: long arrays + wide fields covered. Ours-only: 0x26, 0x64 imports |
| 47 | UU | `wie-lgt/src/runtime/java/jvm_support.rs` | **재적용** | 3way-edit | — | ABI vtable linking machinery, reserved OOM, generated class walk, guest task/thread layout, ~15 tests | upstream ~100 lines RustJava 0.4 adaptation; none of our fns present · Biggest LGT file (+1346); RustJava API move |
| 48 | UU | `wie-lgt/src/runtime/java/jvm_support/class_definition.rs` | **재적용** | 3way-edit | — | try_name (unreadable class no panic), core accessor + tests | upstream also adds patch_declared_instance_field_word_indices (same name in our delta) + Cow names · Partially converged on field word-index patching |
| 49 | UU | `wie-lgt/src/runtime/java/jvm_support/field.rs` | **재적용** | 3way-edit | — | JavaHostField (host-side field storage keyed by core id+instance), forget_core/forget_instance | upstream Field::name/descriptor now return Cow (RustJava 0.4) · Our JavaHostField impl must switch String -> Cow |
| 50 | UU | `wie-lgt/src/runtime/java/jvm_support/method.rs` | **재적용** | 3way-edit | — | per-method native frame, host-error release of guest frames, bad exception ptr report + tests | upstream: Cow name/descriptor, new thumb-stub tests · GC-root frame design differs from upstream's |
| 51 | UU | `wie-lgt/src/runtime/stdlib.rs` | **폐기** | theirs | 0540,0542 | impls for 0x3f9 vsprintf, 0x3ff strtol, 0x408 strncat, 0x40a strncmp, 0x40b strchr, 0x415 memmove, 0x426 malloc, 0x428 free + svc-table tests | upstream stdlib.rs L22-51 dispatches every one of our slots (plus strtok/memcmp/memchr/atexit/abort/realloc/unk450) with own tests (strtol_import..., bounded_string_imports...) · All 8 slots covered (ours 0x424 Unk3 = theirs Atexit). Drops our svc-table regression tests and use of wie_core_arm::stdlib; check strchr has an upstream test · Behaviour gap on allocation failure only: upstream malloc returns Err (fatal), ours returns 0 so the guest's operator new raises its own exception (b7699c10dfd1 comment) |
| 52 | UU | `wie-lgt/src/runtime/svc_ids.rs` | **재적용** | 3way-edit | 0540,0542 | Java 35..38 ids, WIPIC 30 ids (0x68,0xcf,0xee,0xf0,Im*,Uic*,net,byte order,fs list,0x44c,0x7d0), stdlib 8 ids, 6 table tests | upstream enum (L134-277, L441-471) has all but: 0x68 is Unk13 stub, 0x386 Ntohl missing, Java 37/38 missing; Java 35/36 numbered swapped (Load=35,Store=36) · Half converged. Semantics differ: 0xf0 FillPolygon vs our outline, 0x19c FsAvailable vs ListDatabases, 0x44c TermRes groups vs ListDataStores |
| 53 | UU | `wie-lgt/src/runtime/wipi_c.rs` | **재적용** | 3way-edit | 0542 | local handlers: terminate_program (0x68), Im*, Uic*, byte order incl. ntohl, list_directory/data_stores, softkey rows, flush_lcd latch, rgb565 alpha images, memory notice + tests | upstream routes most ids to shared wie_wipi_c::api (uic::*, net::*, util::*, fs::list, term_res::*) via into_body; no terminate/softkey/latch · Largely superseded at the id level; behaviour (0x68 exit, flush latch, softkey rows, local_net) ours-only |
| 54 | UU | `wie-lgt/src/runtime/wipi_c/context.rs` | **재적용** | 3way-edit | — | timer-driven collect_garbage_if_due, alloc/free_indirect with free(NULL) guard, resource_stream + tests | upstream: callback fault diagnostics test, with_arm_context · GC scheduling is ours |
| 55 | UU | `wie-lgt/src/runtime/wipi_c/graphics.rs` | **재적용** | 3way-edit | — | native pixel op for blits, shared graphics context, screen_present_row annunciator latch, display-property svc test | upstream rewrote with LgtGraphicsState, get_context, destroy_image, clip bounds, application_y · Heavy overlap in the same functions; reconcile per function |
| 56 | UU | `wie-midp/src/classes/javax/microedition/lcdui/canvas.rs` | **발신** | 3way-edit | — | pointer/notify stubs (hasPointerMotionEvents,hasRepeatEvents,pointer*,show/hideNotify), getKeyName, J2ME standard keycodes (-1..-7), serviceRepaints via Display, 3 tests | serviceRepaints->Display.serviceRepaints (canvas.rs service_repaints); hasPointerEvents L40; getKeyCode L34 (#1569) · half converged (serviceRepaints, hasPointerEvents, getKeyCode). RISK: GAME_A..D map differs (ours 7,9,*,# vs up 1,3,7,9) - pick one. field calls need RustJava owner arg |
| 57 | UU | `wie-midp/src/classes/javax/microedition/lcdui/display.rs` | **재적용** | 3way-edit | — | paint serialization (hold other guest threads), dirty-rect clip union, pacing hooks, screen compositor (Clet fb present), GC interval/cost spacing, set_current show/hideNotify, 13 tests | none for ours parts; up only switched put/get_field to explicit owner (#1444) and GC error->WieError (#1542) · 763+ lines on fork infra (pacing, has_screen_compositor, compose_screen absent upstream). rewrite every field access to owner form; reconcile with RustJava 0.4 sync GC roots |
| 58 | UU | `wie-midp/src/classes/javax/microedition/lcdui/font.rs` | **발신** | 3way-edit | — | getFace/getStyle/getSize/isPlain/isBold/isItalic/isUnderlined + face/style/size fields; getBaselinePosition = height*4/5; 1 test | getBaselinePosition L26 exists but stub returning 10 (L135, #1569) · half converged (method present). keep our baseline body over up stub; add owner arg to field access |
| 59 | UU | `wie-midp/src/classes/javax/microedition/midlet/midlet.rs` | **발신** | 3way-edit | — | wieDisplay rename + get/put_declared_field (subclass shadowing); notifyDestroyed -> exit_from_guest; 2 tests | explicit field owner get_field(this,"javax/microedition/midlet/MIDlet","display",..) (#1444) solves shadowing; notifyDestroyed still stub · half converged: drop wieDisplay/declared-field hack, take theirs; keep notifyDestroyed exit (needs fork System::exit_from_guest + test-utils run_jvm_test_until_exit) |
| 60 | UU | `wie-midp/src/classes/net/wie/smaf_player.rs` | **발신** | 3way-edit | — | MIDP Player model: END_OF_MEDIA to listeners, repeat/started/lengthMs/generation, PlayerControl, tone device locator, 5 tests | none (up only owner-arg rewrite #1444) · generic Player semantics; depends on fork audio().duration. every hunk is the owner-arg API change around our extra fields |
| 61 | UU | `wie-skvm/src/classes/com/skt/m/graphics_2d.rs` | **발신** | 3way-edit | — | DRAW_AND/DRAW_OR blending, captureLCD via screen Graphics with null midlet/display guards, copyLCD, 2 tests | captureLCD implemented L139 via Display.screenImage (#1482); DRAW_AND/OR still warn+return L222 · half converged (captureLCD). keep our AND/OR path replacing up early-return; pick one captureLCD (ours has null guards) |
| 62 | UU | `wie-skvm/src/classes/com/xce/io/x_file.rs` | **발신** | 3way-edit | — | XFile(jar,name) jar entry selection, buf exposure (open_jar_entry), fsavail 1000000, 3 tests | none (up still IOException "JAR file selection is not supported"; owner-arg #1444) · needs fork backend filesystem().virtual_file + zip_entry (absent upstream) |
| 63 | UU | `wie-skvm/src/classes/com/xce/lcdui/x_display.rs` | **발신** | 3way-edit | — | XDisplay.clear, drawImageEx (delegates Graphics2D.drawImage), 2 tests | up has only clinit+refresh (rewritten #1493); no clear/drawImageEx · conflict = test module (ours mod test vs up mod tests): union into one module; our new code needs owner-arg edit |
| 64 | UU | `wie-skvm/src/classes/com/xce/lcdui/x_text_field.rs` | **발신** | 3way-edit | — | numeric multitap (tapKey/tapChar/tapTime), repaint on change, static focused field so handler-only keys reach it, 2 tests | none (up keyPressed append unchanged except owner arg #1444) · uses fork wie_util::keypad; all 5 hunks are owner-arg rewrite vs our refactor (append/take_key/put_text) |
| 65 | UD | `wie-wipi-c/src/api/database.rs` | **재적용** | 3way-edit | 0547 | KTF/LGT DB slots: slot4 lseek, slot7 list/rename, slot8 mkdir, slot15 size, exists, 16MB space, LGT mode8 create, sorted listings, KTF Interface4 record DB, packaged p-file + mode-4 jar seed (0547) | deleted by #1520 9237ca53a; slots rerouted to wie-wipi-c/src/api/fs.rs (open/read/write/seek/exists/available); fs list/rename/mkdir are stubs -1 (fs.rs L161,L220,L225) · RISK: semantics moved to fs.rs + KTF/LGT method_table (other groups). decide: keep file + our wiring, or port slot behaviour into fs.rs. 641+ lines, title-specific seeding |
| 66 | UU | `wie-wipi-c/src/api/graphics.rs` | **재적용** | 3way-edit | 0541 | pixel ops+color key, LGT alpha mask, wide KTF ctx, screen fb 2-way sync, read_framebuffer_or_null, NULL image size 0 (0541), null clip, encodeImage BMP, draw_polygon, leak fix, LGT 24-line shift | null clip clears mask (#1514, set_context pv==0); encode_image BMP L~478 (#1506) using encode_bmp(&region); draw_image honours ctx clip (#1514); fill_polygon stub · half converged (null clip, encodeImage, ctx clip). encode_bmp signature differs (ours 5 args vs up 1) - wie-backend canvas must agree. ours draw_polygon vs up fill_polygon may be same slot |
| 67 | UU | `wie-wipi-c/src/api/graphics/primitives.rs` | **발신** | 3way-edit | — | write_canvas limited to touched region (no full framebuffer copy), 1 test | draw_text baseline shift y-=FONT_SIZE (#1517) - orthogonal · carry up baseline shift into our new write_canvas(ctx,fb,None,clip,..) signature; check no double baseline adjust elsewhere |
| 68 | UU | `wie-wipi-c/src/api/kernel.rs` | **재적용** | 3way-edit | — | timer arm/cancel via event_queue + pacing share per tick, DefTimer clears pending, PHONENUMBER non-empty, alloc zeroed/alloc_or_null, free mem 2MiB, sysprops, buffer-size rejects, 13 tests | #1536 timer cancel/rearm via guest-memory PendingTimer list (-5 on double set); #1569 execute/get_access_level/get_exec_names; PHONENUMBER "" kept · RISK: timer cancellation converged in intent, divergent mechanism + return code (ours -1, up -5); must pick one and keep our pacing hooks. union up's 3 new stubs |
| 69 | UU | `wie-wipi-c/src/api/kernel/sprintf.rs` | **발신** | 3way-edit | 0542 | bytes pass through verbatim (no EUC-KR roundtrip), %s precision/left-justify, integer precision/zero pad (pad_integer), %i, 4 tests | #1481: args now Iterator<Result<u32>>, long long (next_arg64), (null) for %s; still EUC-KR decode, no precision/%i · port our formatting onto up's Result-iterator args + keep up long-long support |
| 70 | UU | `wie-wipi-c/src/api/media.rs` | **발신** | 3way-edit | — | clip/master volume -> AudioSink gain, clip callbacks STARTED/END_OF_DATA/STOPPED with generation guard, nonzero clip ids, 11 tests | get_volume/set_volume/clip_get_volume/clip_set_volume are warn stubs (L167-186); new get_default_volume, clip_clear_data stubs (#1532,#1502) · keep our bodies over up stubs, add up get_default_volume + clip_clear_data. depends on fork AudioSink::set_gain |
| 71 | UU | `wie-wipi-c/src/api/net.rs` | **재적용** | 3way-edit | 0542,0548 | in-engine fake server (fake_net) for listed titles: socket write/read, read/write callbacks; connect fails through callback (0542,0548); 4 tests | up LGT wiring (#1508,#1502) only; no fake server · fake server = fork-only, title-keyed; keep ours and re-check up socket fn signatures |
| 72 | UU | `wie-wipi-c/src/api/util.rs` | **폐기** | theirs | — | inet_addr (dotted quad -> network order, INADDR_NONE) + parse_dotted_quad | inet_addr L27 (Ipv4Addr parse, map_or u32::MAX, from_le_bytes) + ntohs (#1508/#1519) · fully converged; minor diff: Ipv4Addr rejects leading-zero octets ours accepted. callers (KTF slot4, LGT 904) same u32 signature |
| 73 | UU | `wie-wipi-c/src/context.rs` | **재적용** | 3way-edit | — | WIPICContext::set_timer gains pace_from; wide_graphics_context(); TestContext records calls/spawned/timers, bounds-checked memory (range) | #1509 TestContext functions + global_data split (GLOBAL_DATA_BASE); #1536 timers VecDeque, call_function invokes registered fn · trait signature change ripples to KTF/LGT impls; test ctx must union both field sets; follows kernel.rs timer choice |
| 74 | UU | `wie-wipi-java/src/classes/net/wie/card_canvas.rs` | **발신** | 3way-edit | — | paint keeps Display's repaint clip per card (#400) + skips cards fully covered by an opaque card above (#524); reads Card.transparent via our get_declared_field | upstream only changed get/put_field to explicit owner (228e3ff48 #1444, e.g. get_field(&this,"net/wie/CardCanvas","cards",...)); no clip/cover logic · generic engine fix. Take ours logic, convert to owner-arg get_field; get_declared_field(...Card,transparent) -> jvm.get_field(&card,"org/kwis/msp/lcdui/Card",...) |
| 75 | UU | `wie-wipi-java/src/classes/org/kwis/msp/handset/handset_property.rs` | **발신** | 3way-edit | — | getSystemProperty VOLUMELEVEL="3", PHONENUMBER="01000000000" + 2 tests | 87c798299 #1507 adds "VOLUMELEVEL" => "100" (upstream line ~44); upstream deleted the test module (e96d36cca #1441) · half converged: VOLUMELEVEL key converged but VALUE differs (3 vs 100) - pick one deliberately (our wipi-c VOLUMELEVEL #550 too); PHONENUMBER ours-only |
| 76 | AA | `wie-wipi-java/src/classes/org/kwis/msp/handset/led.rs` | **폐기** | theirs | — | LED.set(I)V stub + led_set_is_callable test | upstream led.rs: identical proto LED.set (I)V PUBLIC\|STATIC, same no-op body (a655ed231 #1519) · only residue is a trivial callability test; upstream #1441 policy removes such tests - drop it |
| 77 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/card.rs` | **발신** | 3way-edit | — | Card()/Card(Z) start at y=AnnunciatorComponent::shown_height (KTF status bar, #464); x/y/w/h/display read via declared-field helpers (subclass shadowing #414); tests incl. ShadowingCard, opaque-cover | 228e3ff48 #1444: every get/put_field now passes owner "org/kwis/msp/lcdui/Card" (upstream card.rs:245-274) = same shadowing fix · half converged: shadowing fix superseded by owner API (drop bound/set_bound helpers). Remaining: status-bar top offset (needs our AnnunciatorComponent::shown_height) + tests. 10 hunks |
| 78 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/graphics.rs` | **발신** | 3way-edit | — | setFont(null)=default font; setRGBPixels passes processAlpha=false (0x00RRGGBB opaque, #455) + 2 tests | b1afb6ef8 "handle null fonts in WIPI graphics": upstream set_font line 267 `if font.is_null()`; upstream drawRGB still `true` (line 957) · half converged: null font = theirs. Keep ours: drawRGB arg true->false + test_set_rgb_pixels_draws_alpha_less_pixels_opaque |
| 79 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/input_method_handler.rs` | **발신** | 3way-edit | — | full input method: mode get/set/next, getCurrentInputMode, notifyKeyInput multitap/Hangul composition via wie_util::keypad, listener notifyTextChanged, fields (#244 #412 #425 #461 #508) | upstream still stubs only (<init>, setCurrentMode returns true, hideSymbolCard, setInputMethodListener no-op) after a655ed231/031cb8f8d · take ours whole, port to owner-arg get/put_field; depends on our wie_util::keypad. Upstream InputMethodListener now declares notifyTextChanged, so our listener-class lookup still works |
| 80 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/input_method_listener.rs` | **폐기** | theirs | — | empty interface proto (no methods, INTERFACE flag) | upstream input_method_listener.rs: same name + notifyTextChanged([CII)V abstract, PUBLIC\|INTERFACE\|ABSTRACT (a655ed231 #1519) = superset · prior-round 발신 candidate now upstream |
| 81 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/jlet.rs` | **발신** | 3way-edit | — | getCurrentJlet alias (#276); notifyDestroyed no longer calls destroyApp (recursion/stack overflow, #350); dis via get_declared_field (#414); 3 tests | 228e3ff48: display() uses get_field(this,"org/kwis/msp/lcdui/Jlet","dis",...) = shadowing fix; upstream notify_destroyed still calls destroyApp, no getCurrentJlet · half converged (dis). Keep alias + recursion fix + tests; notify test needs our test_utils run_jvm_test_until_exit. A test comment names a title in Korean - scrub before offering upstream |
| 82 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc.rs` | **발신** | union | — | mods+re-exports: ActionListener, ButtonComponent, DialogComponent, ProgressComponent, KEY_NOTIFY, Form/GrabKey/Label | 10c12ec58 #1569 adds form_component, grab_key_listener, label_component mods/re-exports · half converged; take ours list (superset of theirs) |
| 83 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/component.rs` | **발신** | 3way-edit | — | stateful lwc Component: x/y/w/h/bg/fg/evtListeners fields, getX..getHeight, focus, validate->virtual layout, repaint via ShellCard, paint, setEventListener, key dispatch + tests | 10c12ec58 #1569 adds same-name methods as STUBS (getX/getWidth return 0, validate/layout no-op) + extras (calcPreferredSize, canHandleInput, getCard, invalidate, pointerNotify, processEvent...) · keep ours bodies; add only upstream's extra signatures. Taking theirs for a shared name silently reverts behaviour (e.g. validate no-op breaks guest layout) |
| 84 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/form_component.rs` | **발신** | 3way-edit | — | FormComponent()/(Z) + `vertical` field kept + setFocus(Component) (#394 #470) + test | upstream form_component.rs: both <init> (init_with_orientation drops the flag), no setFocus, no field (a655ed231) · half converged: ctors upstream. Add setFocus(Lorg/kwis/msp/lwc/Component;)V + vertical field onto theirs |
| 85 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/grab_key_listener.rs` | **폐기** | theirs | — | interface grabKeyNotify(IILjava/lang/Object;)Z | upstream grab_key_listener.rs: byte-identical proto (031cb8f8d #1532); only comment differs · comment cites a sha only |
| 86 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/label_component.rs` | **폐기** | theirs | — | <init>()V, <init>(String)V stubs | upstream label_component.rs: same 2 ctors + (String,Image) + (String,String) ctors (10c12ec58 #1569) = superset |
| 87 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/shell_component.rs` | **발신** | 3way-edit | — | getWidth/getHeight, title get/set, keyNotify routing to focused leaf, paint, grabKey/ungrabKey + GrabKeyListener, setWorkComponent handling (#458 #470 #485 #503 #548) + tests | 10c12ec58 adds only ShellComponent.layout()V stub · keep ours + add theirs layout stub (harmless: Component.layout is no-op in ours too; guest overrides still win) |
| 88 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/text_component.rs` | **발신** | 3way-edit | — | m_td/iMode/maxLength fields, getMaxLength, keyNotify (input method, cap), paint, password '*' (#445 #458 #461 #503 #548) + tests | 031cb8f8d #1532 adds fields m_td [C and iMode I (PROTECTED) = converged with ours #458; 228e3ff48 owner-arg put_field · half converged (fields); keep ours methods, owner-arg API |
| 89 | UU | `wie-wipi-java/src/classes/org/kwis/msp/media/base_clip.rs` | **발신** | 3way-edit | — | setBuffer keeps the game's array + FNV hash; refresh() reloads on play if rewritten; putData releases registered array (#359) | 852a4cb01 #1490: setBuffer stores ByteArrayInputStream over the array, player created lazily at first play (Clip.getOrCreatePlayer), reused "until buffers are replaced" · half converged: first-play read now upstream. Rewrite-between-plays (same array) still ours-only - re-express refresh on top of stream/getOrCreatePlayer (clearData + new stream when hash changes) |
| 90 | UU | `wie-wipi-java/src/classes/org/kwis/msp/media/clip.rs` | **발신** | 3way-edit | — | default volume 100; Clip(type,null) throws NPE; setVolume applies to audio handle (apply_volume, #356); player(null)=None; setBuffer->BaseClip.setBuffer + tests | 852a4cb01: setBuffer -> BaseClip.setBuffer (converged), adds getOrCreatePlayer; no volume apply, default stays 0 · half converged; our Clip::player() helper is replaced by getOrCreatePlayer - port apply_volume/null-safe callers onto it |
| 91 | AA | `wie-wipi-java/src/classes/org/kwis/msp/media/media_unsupported_exception.rs` | **폐기** | theirs | — | 2 ctors ()V,(String)V calling Exception.<init> | upstream media_unsupported_exception.rs: same 2 ctors + PUBLIC flags (10c12ec58 #1569) |
| 92 | UU | `wie-wipi-java/src/classes/org/kwis/msp/media/player.rs` | **발신** | 3way-edit | — | BaseClip overloads route to Clip path; resume(Clip) real (no restart if STARTED); stop returns true only if was STARTED (#550); play refreshes buffer + applies volume + tests | upstream: resume(Clip) registered but -> stub false; BaseClip overloads stubs; stop_clip(null) throws NPE (29404b979 #1501), returns true for any player; play via getOrCreatePlayer (#1490) · SEMANTIC CONFLICT: null-clip stop = NPE (upstream) vs silent false (ours, census: titles stop(null) for sound-off); stop return value also differs. Decide before merge |
| 93 | UU | `wie-wipi-java/src/lib.rs` | **재적용** | 3way-edit | — | get_protos 42->67: preload-order block at top, LED, Button/Dialog/Progress, ShellCard, wec/*, com/ktf/kfc/*, preload comment (locked by a test) | upstream 42->48 adds LED, InputMethodListener, Form/GrabKey/Label, MediaUnsupportedException (10c12ec58 et al.) · auto-merge already DUPLICATED 6 entries outside the hunk (ImageObserver, InputMethodListener, Form/GrabKey/Label, MediaUnsupportedException): dedupe keeping our top order, recount array length |

**계수**: 폐기 **11** + 재적용 **53** + 발신 **29** = **93** · ★**미분류 0**. 이 값은 표의 판정 칸을 `grep -oE '\*\*(폐기|재적용|발신)\*\*' | sort | uniq -c` 로 세어 확인했다(93행).

### 5. 예행 트리 게이트 — red 를 그대로 적는다

예행 트리는 셋이다. 충돌 표식이 남은 트리는 매니페스트 파싱에서 죽기 때문에, 충돌을 기계적으로 한쪽으로 몬 트리 둘을 더 만들었다. 이 둘은 **해소안이 아니라 탐침**이다. 「충돌 밖에서 git 이 자동으로 합친 200여 파일이 어느 쪽 코드와 함께 서는가」를 본다.

| 트리 | 만든 법 |
|---|---|
| **P0 raw** | `git merge --no-commit --no-ff upstream/main` 그대로(표식 93 파일) |
| **P1 ours** | P0 에서 UU·AA 89개는 `git checkout --ours`, UD 4개는 우리 판 유지. 1차(L0)는 매니페스트 오류로 죽었다. L1 은 매니페스트만 고쳤다: `Cargo.toml`·`wie-backend/Cargo.toml`·`wie-j2me/Cargo.toml` 의 충돌 덩어리를 양쪽 합집합으로, 루트 `workspace.dependencies` 에 `md5` 를 되살렸다(upstream 이 지웠고 우리 5크레이트가 쓴다). |
| **P2 theirs** | UU·AA 는 `--theirs`, UD 는 삭제. L0 은 매니페스트 오류. L1 은 루트에 `md5` 를 되살리고 `wie-app` 을 members 에서 빼 `exclude` 로 두었다(우리 `exclude` 결정 · 0119). |

결과(rc 는 명령 그대로 · 무거운 명령은 `build-slot run --` 으로 감쌌다):

| 게이트 | P0 raw | P1 ours L0 | P1 ours L1 | P2 theirs L0 | P2 theirs L1 |
|---|---|---|---|---|---|
| `cargo fmt --all -- --check` | rc=1 · `cargo metadata`: `Cargo.toml:160` `<<<<<<< HEAD` | rc=1 | rc=1 · `failed to resolve mod hook: wie-core-arm/src/binary_patches/hook.rs does not exist` | rc=1 | **rc=0** |
| `cargo clippy --all -- -D warnings` | rc=101 · 같은 파싱 오류 | rc=101 · ``dependency.wie-core-arm-aot` was not found in `workspace.dependencies`` | rc=101 · `wie-backend` 오류 4 | rc=101 · ``dependency.md5` was not found`` | rc=101 · `wie-backend` 오류 8 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=101 · 같음 | rc=101 · 같음 | rc=101 · `getrandom 0.3.4`: wasm32 는 `wasm_js` 필요 | rc=101 · 같음 | rc=101 · `wie-backend` 오류 8 |
| `RUST_MIN_STACK=4194304 cargo test --all` | rc=101 · 같음 | rc=101 · 같음 | rc=101 · `wie-backend` 오류 4 | rc=101 · 같음 | rc=101 · `wie-backend` 오류 7 |
| `cargo +beta clippy --all -- -D warnings` | rc=101 · 같음 | rc=101 · 같음 | rc=101 · `wie-backend` 오류 4 | rc=101 · 같음 | rc=101 · `wie-backend` 오류 8 |

`wie-backend` 는 스택 맨 아래다. 그래서 `cargo check --workspace --all-targets --keep-going` 으로 다시 돌려도 다른 크레이트는 컴파일까지 가지 못했다(P1 오류 4 · P2 오류 7 · 둘 다 rc=101). 오류 원문(축약):

```
P1 ours L1
wie-backend/src/system.rs:31  E0603 function `normalize_guest_path` is private
wie-backend/src/task.rs:44    E0308 expected `u64`, found `Instant`          ← upstream #1491 sleep 마감
wie-backend/src/lib.rs:99     E0277 `Result<Cow<str>, ZipError>` doesn't implement Display   ← zip 8 → 9
wie-backend/src/lib.rs:102    E0599 `to_string` on `Result<Cow<str>, ZipError>`
P2 theirs L1
wie-backend/src/system.rs:16  E0432 unresolved `crate::fake_server`, `crate::pacing`, `executor::TICK_BUDGET_MS`
wie-backend/src/system.rs:89  E0599 no method `tick_for` on `Executor`     (+ wants_cpu · next_due · halt · clear · sleep_toward)
```

- P1 L1 의 wasm 오류는 탐침의 매니페스트 해소가 만든 것이다. upstream 은 루트 `[dependencies]` 의 `sentry` 등을 `cli` 기능 뒤 optional 로 옮겼다. 우리 판은 `cfg(not(target_arch = "wasm32"))` 블록으로 막았다. 합집합 해소는 그 cfg 를 잃어서 기본 기능 `cli` 가 wasm 에서 `sentry → rand → getrandom 0.3` 을 끌어온다. 실제 병합에서는 `Cargo.toml` 행(#6)의 3-way 몫이다.
- **예행 트리에서는 runner 블록(픽스처 5종 + `text_j2me`)을 돌리지 못했다.** `wie_validate` 는 `wie_cli` 의 바이너리이고 `wie_cli` 는 `wie-backend` 에 기댄다. 어느 예행 트리에서도 바이너리가 생기지 않는다. 이것은 «PASS 를 못 봤다»이지 «FAIL» 이 아니다.

### 6. 충돌 93 밖 — git 이 조용히 합친 35파일

`git diff --name-only 44fbf265 origin/main` 과 `… upstream/main` 의 교집합은 128 파일이다. 그중 93 은 충돌이고 **35는 자동으로 합쳐졌다**.
★이 35 는 경로 문자열 교집합이라 **upstream 이 이름을 바꾼 파일을 못 센다.** `--no-renames` 로 세면 129 · **36** 이다. 하나 더는 우리 `src/filesystem.rs`(+68 · `path_for` 경계 시험)가 upstream `src/cli/filesystem.rs`(R056)로 자동 병합되는 것이다. 병합 결과는 upstream `path_for`(aid→pid 이름만 바뀜) + 우리 `mod tests` 이고 단언 리터럴이 그대로 성립한다(검수 확인). 다만 aid→pid 사람 결정(§8 «파일시스템 이름공간»)이 이 시험의 이름도 정한다. 자동 병합이 옳다는 보장은 없다. §5 의 첫 오류 둘(`system.rs` · `task.rs`)이 바로 이 35 에서 나왔다.

```
.github/workflows/{coverage.yml,release.yaml}
wie-backend/src/{platform.rs,system.rs,system/event_queue.rs,task.rs}
wie-core-arm/src/{allocator.rs,allocator/list.rs,context.rs}
wie-j2me/src/emulator.rs
wie-ktf/src/runtime.rs · wie-ktf/src/runtime/wipi_c/interface.rs
wie-ktf/src/runtime/java/jvm_support/{array_class_definition.rs,class_instance.rs,classes/net/wie/ktf_class_loader.rs}
wie-lgt/src/runtime/{init.rs,java.rs} · wie-lgt/tests/test_helloworld.rs
wie-lgt/src/runtime/java/jvm_support/{array_class_definition.rs,jvm_implementation.rs,vtable.rs}
wie-midp/src/classes/javax/microedition/lcdui/{game/game_canvas.rs,graphics.rs,image.rs}
wie-midp/src/classes/javax/microedition/rms/record_store.rs · wie-midp/src/classes/net/wie/event_queue.rs
wie-util/src/lib.rs · wie-wipi-c/src/lib.rs
wie-wipi-java/src/classes/org/kwis/msp/{db/data_base.rs,handset.rs,io/file.rs,lcdui.rs,lcdui/display.rs,media.rs,media/volume.rs}
```

그 밖에 자동 병합이 **떨어뜨린 것**도 있다(ours 탐침에서 확인).
- `wie-core-arm/src/binary_patches/{hook,parser,patch}.rs` 는 우리가 고치지 않아 upstream 의 삭제가 그대로 들어왔다. 그런데 UD 로 남긴 `binary_patches.rs` 는 그 셋을 `mod` 로 부른다. P1 의 fmt 오류가 이것이다.
- `wie-core-arm/src/lib.rs` 의 `mod binary_patches;`·`pub mod stdlib;` 는 충돌 덩어리 «밖»에서 조용히 지워진다(덩어리에는 `use` 한 줄만 선다). `wie-core-arm/Cargo.toml` 의 `md5`·`serde`·`toml` 은 충돌 없이 빠진다.
- `wie-web/public/GeneralUser.sf3` 는 upstream `cfec8186a`(#1460)가 지웠고 우리가 고치지 않아 충돌 없이 사라진다. 우리 소비자: `scripts/contract-roundtrip.mjs:346`(시나리오 S 사운드폰트 — S3 의 Constraint 3 검증식 자체) · `scripts/check-audio-worklet.mjs:299` · `scripts/soundfont-stall-probe.mjs:50` · `wie-web/src/ts/midi.ts:38` · `.github/workflows/engine-contract.yml:333` paths. 0317·0349 가 채택한 사운드폰트 경로다.
- `wie-wipi-java/src/lib.rs` 의 `get_protos()` 에 충돌 덩어리 **밖**에서 중복 등록 6건이 생긴다(`ImageObserver` 외). 표식만 풀면 남는다.

⇒ 병합 조각의 검증은 «충돌 93 해소»로 닫으면 안 된다. 컴파일과 시험으로 닫아야 한다.

### 5-B. 코퍼스 `smoke_gate` — 전(`origin/main` `2e66f9d6`)은 쟀고, 후는 못 잰다

`WORKING_DIR=game_lab/working PLATFORM_FILTER=<p> bash scripts/smoke_gate.sh` 를 플랫폼마다 `build-slot run --long` 임대 1건으로 돌렸다(phase = 플랫폼). 바이너리는 `origin/main` 의 `target/debug/wie_validate` 다. «후»는 §5 대로 예행 트리에 바이너리가 없어 **UNMEASURED** 다.

| 플랫폼 | 전: 실행 | 전: baseline 대조 | rc | 걸린 시간 · load1(끝날 때) | 후 |
|---|---|---|---|---|---|
| lgt | 54 PASS / 0 FAIL | 52 대조 · 0 absent · 회귀 0 | 0 | 879 s · 54.6(도중 134 까지) | UNMEASURED |
| skt | 50 PASS / 0 FAIL | 50 대조 · 0 absent · 회귀 0 | 0 | 765 s · 22.9 | UNMEASURED |
| ktf | 189 PASS / **1 FAIL** | 190 대조 · 0 absent · **회귀 1** | **1** | 2,984 s · 31.3 | UNMEASURED |

★**ktf 의 FAIL 1건(sha256 `c36cbe78011f`)은 이 PR 과 무관하다. 손대지 않은 `origin/main` 에서 난 것이고, 부하에 굶은 것으로 판정한다.** smoke_gate 는 이미 재시도 2회를 했다. 끝난 뒤 따로 다시 돌렸다(load1 17~28).
- `--timeout 15`(gate 기본값): 3회 중 PASS 1. FAIL 2회는 `paints=0 · stop=deadline` 이다.
- `--timeout 30`: 2회 중 PASS 2. `paints` 는 130 · 129 다.

시간을 주면 그린다. 그래서 «마감 직전에 굶은 것»이다. `AGENTS.md` 의 ⑴(다시 돌리기)·⑷(부하 읽기) 순서로 판정했다. ⑶(손대지 않은 트리)은 이것이 이미 그 트리다. 이 회차가 고칠 것은 아니고, 제안 문턱(관찰 · 지금 실행 가능 · 중복 아님)도 넘지 않는다. 다음 «전/후» 대조에서 이 sha 가 또 FAIL 이면 `--timeout 30` 으로 다시 재서 가른다.

이 «전» 값은 S1·S2·S3 의 «전/후» 대조에서 «전»의 기준점이 된다. 그 회차들은 착지 직전 `origin/main` 에서 다시 재야 한다(이 값은 2026-11-06 의 것이다).

### 7. 조각 계획 — 순서 · 범위 · 검증식

조각 D 와 같은 원칙이다. «병합 전에 우리 쪽에서 치울 수 있는 것은 먼저 치운다. 병합은 남은 것만 진다.» 모든 조각은 브랜치 → PR 에서 멈춘다. 착지는 별도 `-merge` 티켓이 한다(`merge_strategy: merge` · squash 금지). 스윕은 `bash scripts/census-drive.sh <run 인자…>` 로 phase 단위 `--long` 임대를 쓴다.

**공통 검증식 V0**(모든 조각): `AGENTS.md` §The four gates 4종 + `cargo +beta clippy --all -- -D warnings` · runner 블록(`make-draw-fixture` → 픽스처 5종 + `text_j2me --timeout 5`, `keydraw_*` 는 `--inject --expect-last-frame` 으로 PASS·rc=0) · `gh pr checks <n> -R Jun025/wie` 인용.

| 조각 | 범위 | 조각만의 검증식 | 기대 |
|---|---|---|---|
| **S1** RustJava 핀을 우리 쪽에서 `^0.4.1` 로 (병합 전 · 우리만) | 루트 `jvm*`·`rustjava-runtime` 을 0.4.1 로 올린다. upstream 이 같은 일을 한 커밋을 본으로 쓴다: `5fb65ee82`(#1430 0.2.0) · `228e3ff48`(#1444 0.3.0 필드 owner) · `7951304c9`(#1489 0.3.2 JarFile) · `28c182434`·`b149acdc2`·`0c338e2b7`(0.3.3~0.3.5) · `9fd11d483`(#1542 0.4 동기 GC 루트) · `a6dca709d`(0.4.1). `patches/classfile` 은 지울 후보다. crates.io `classfile 0.4.1` 의 `parse_utf8` 이 `jvm_types::modified_utf8::decode` 를 쓴다(이 머신 레지스트리에서 열어 봄). 너무 크면 S1a(0.1→0.3, owner 인자)와 S1b(0.3→0.4, GC 루트)로 나눈다. | V0 · `cargo test -p wie-jvm-support` 의 `hardening` 시험(AGENTS.md 사건대장: 핀을 옮기면 다시 돌린다) · `patches/classfile` 을 지우면 그 README 의 시험 4종을 0.4.1 에 옮겨 통과 · `smoke_gate.sh` 를 ktf·lgt·skt 각각 전/후로 돌려 회귀 0 · `node scripts/check-engine-contract.mjs`(wasm 표면 불변) · 조각 뒤 예행을 다시 돌려 충돌 수 `93 → N` 을 적는다 | `N < 93`. 등식은 아니다. 양쪽이 같은 줄을 따로 고친 곳은 여전히 충돌한다. ★Java 클래스 파일 발신 29 의 3way-edit 대부분이 union·theirs 로 내려오는지가 이 조각의 성패다 |
| **S2** executor·pacing 을 upstream 의미로 (병합 전 · 우리만) | `wie-backend` `executor.rs`·`system.rs`·`task.rs`. upstream 의 sleep 마감 의미(`49078c6b6` #1491)와 협동 폴링(`d63d6022d` #1429)을 우리 pacing API(`tick_for`·`wants_cpu`·`next_due`·`halt`·`clear`·`sleep_toward`·`TICK_BUDGET_MS`) 아래로 옮긴다. 우리 쪽 결정성(`BTreeMap` 스폰 순서)은 지킨다. | V0 · `wie-j2me/tests/test_pacing.rs`(`pace_j2me`) 수치 불변 · `bash scripts/perf-watch.sh compare origin/main <S2 head> <out>` 로 속도 축 회귀 없음 · 예행 탐침 P1 에서 `wie-backend` 오류 0 | 예행 탐침의 첫 벽이 `wie-backend` 밖으로 옮겨 간다 |
| **S3** 병합 커밋 (`git merge upstream/main`) | 남은 충돌 해소 + §6 의 조용한 병합 35 확인. 몫이 큰 것: ⑴ `binary_patches` 살리기 — UD 3개 우리 판 유지, `binary_patches/{hook,parser,patch}.rs` 복원, `wie-core-arm/src/lib.rs` 의 `mod` 두 줄과 `Cargo.toml` 의 `md5`·`serde`·`toml` 복원, 패치가 AOT 컴파일 «전에» 들어가거나 AOT 가 쓰기 무효화를 하는지 확인. ⑵ KTF `InitSvcId` 7 충돌(우리 `JavaThrowInstance` ↔ upstream `VisitGcRoot`)과 GC 루트 설계 하나 고르기(#1545 ↔ 우리 `guest_roots`). ⑶ `adf.rs` 의 새 `validate_client_bin`·`validate_jar` 가 재배치 표 머리를 가진 KTF 이미지 3종과 «jar 하나뿐이면 그것» 폴백을 거부하지 않게. ⑷ LGT 예외 프레임 설계 하나 고르기 + 시험 이식. ⑸ `wie-wipi-c/src/api/database.rs`(upstream 삭제 · 0547 몫 포함)를 남길지 `fs.rs` 로 옮길지. ⑹ 타이머(#1536 ↔ 우리 `arm_timer`+pacing). ⑺ `Cargo.toml` members 합집합 + `exclude = ["wie-app"]` 유지 + wasm 에서 `cli` 기능 의존이 안 끌려오게. ⑻ `Cargo.lock` 재생성이 `wasm-bindgen =0.2.108` 고정(Constraint 8)과 맞는지 — upstream lock 은 0.2.129 다. ⑼ `rust.yml` 은 ours(upstream 의 `xcode-27` 레그 이름을 받으면 required check `rust_ci (macos-latest, stable)` 가 사라져 머지가 막힌다). ⑽ `wie-wipi-java/src/lib.rs` `get_protos()` 의 덩어리 밖 중복 6건 제거. ⑪ `wie-web/public/GeneralUser.sf3` 우리 판 복원(upstream 은 삭제 · 소비자는 §6). | V0 · `npm run build:wasm` → `node scripts/check-engine-contract.mjs` → `node scripts/contract-roundtrip.mjs`(Constraint 3 · 표면이 바뀌면 계약 JSON 을 같은 PR 에서) · `npm run audit` · `smoke_gate.sh` ktf·lgt·skt 전/후 회귀 0 · 0540·0547 행(`01f05f8231f4` · `974e0df9ab1e` · `f770b15f8876`)을 `wie_validate` 로 한 번씩(잠금 화면 너머까지 가는지) · `bash scripts/census-drive.sh run …` 전수 → `report` → `node scripts/check-compat-revert.mjs` 「착지 기준 바뀐 행」 인용 · `perf-watch.sh compare` · `git log --first-parent` 로 병합 커밋의 두 부모 확인 | 충돌 «해소 수»가 아니라 V0 와 회귀 0 으로 닫는다(§6) |
| **S4** 병합 뒤 정리 | `tag-release.yaml` PARKED 머리 문구(「release 스크립트가 우리 트리에 없다」가 거짓이 됨) · 문서 갱신 · upstream `AGENTS.md` 의 Attribution 조항을 받을지(사람 결정) | V0 · `node scripts/check-doc-liveness-parity.mjs` · `node scripts/check-parked-workflows.mjs` | — |
| **S5** 발신 (운영자 확인 뒤 · 외부 저장소) | §4 의 발신 29. 보내기 전에 주석·시험의 sha 접두와 타이틀 언급을 지운다(`jlet.rs` 시험 주석 1곳에 게임 이름이 있다 · 이 문서에는 옮기지 않았다). `x_file`·`midlet`·`smaf_player`·`media`·`x_text_field` 는 우리 backend 기반(`virtual_file`·`exit_from_guest`·`set_gain`·`wie_util::keypad`)이 같이 가야 한다. | 발신 PR 마다 upstream CI | — |

S1 과 S2 는 서로 독립이다. 둘 다 우리 쪽만 바꾸므로 병렬로 열 수 있다. S3 는 둘이 착지한 뒤에 연다.

### 8. 사람이 정할 것 — 동작이 갈리는 곳 (S3 전에)

표의 해당 행 비고에 원문이 있다. 어느 쪽을 골라도 «합칠 수는» 있다. 고르는 순간 게임이 보는 값이 바뀐다.

| 자리 | 우리 | upstream |
|---|---|---|
| `HandsetProperty` `VOLUMELEVEL` | `"3"` | `"100"`(#1507) |
| WIPI `Player.stop(null clip)` | 조용히 `false`(census 에서 소리 끄기로 `stop(null)` 을 부르는 타이틀이 있었다) | `NullPointerException`(#1501) |
| WIPI `Player.stop` 반환 | STARTED 였을 때만 `true`(#550) | 플레이어가 있으면 `true` |
| MIDP `Canvas` GAME_A..D | 7 · 9 · * · # | 1 · 3 · 7 · 9 |
| LGT SVC `0xf0` / `0x19c` / `0x44c` / `0x68` | 외곽선 / `ListDatabases` / `ListDataStores` / 종료 | 채움 / `FsAvailable` / 단말 자원 그룹 목록 / `Unk13` 스텁 |
| 파일시스템 이름공간 | aid | pid — ★저장 키가 바뀐다. 웹·featurephone 호스트의 기존 저장을 옮길지 정해야 한다. `src/cli/filesystem.rs` 에 자동 병합되는 우리 `path_for` 시험(§6 의 36번째)도 이 결정을 따른다 |

### 9. 못 잰 것

| 못 잰 것 | 왜 · 무엇이 있으면 잴 수 있나 |
|---|---|
| 예행 트리의 runner 블록 · `smoke_gate` «후» | 어느 탐침도 `wie-backend` 를 넘지 못해 `wie_validate` 가 없다. S2 뒤 탐침, 또는 S3 트리에서 잰다 |
| 폐기 판정의 «동작» 동일성 | 시그니처·심볼 일치까지 봤다. 본문 의미는 S3 의 시험·smoke 가 진다. 특히 #51(LGT stdlib 이름 수준) |
| RustJava 0.4.1 API 세부(`invoke_special` 이 상위 클래스를 찾는지, `RT_RUSTJAR` 존재 등) | 이 회차는 `cargo check` 를 `wie-backend` 너머로 못 갔다. S1 이 잰다 |
| `Cargo.lock` 재생성이 `wasm-bindgen =0.2.108` 로 풀리는지 | 탐침 L1 은 lock 을 갱신했지만 wasm 빌드가 `wie-backend` 에서 먼저 죽었다. S3 에서 `npm run build:wasm` 이 판정한다 |
| skt 축 신호 | 커밋된 skt 픽스처가 0 이다(조각 D 와 같다). `smoke_gate` skt 가 유일한 신호다 |

<!-- corpus-name-inflow v1 subjects=2 tree=bd91d28f1b94caa9 B=0/0 P=0/0 S=0/0 -->
