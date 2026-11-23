## [2026-11-23] 조각 S3 — upstream/main 병합 커밋 (behind 252 · 충돌 147 · 동작이 갈리는 자리는 우리 값) (wie-upstream-sync2-s3-merge-upstream-main)

**무엇을**: `docs/report/0551` §7 의 조각 S3 이다. `git merge upstream/main`(dlunch/wie `d5242bbff` · 2026-11-23)을 부모 2개짜리 병합 커밋 `23a94715f` 로 만들었다(squash · rebase 아님). 충돌 **147파일**을 0551 §2 술어로 다시 분류해 풀었고(미분류 0 · 부록 A), git 이 충돌 없이 합친 61파일을 하나씩 읽어 틀리게 합쳐진 곳을 되돌렸다(§7). merge-base 는 `44fbf265`(2026-09-16)에서 `d5242bbff` 로 옮겨 간다.

**왜**: 티켓 `wie-upstream-sync2-s3-merge-upstream-main`. 앞 조각 S1(0572) · S1b(0584) · S2(#603 계열)가 다 착지해 병합이 질 몫이 «남은 충돌»뿐이 됐다. 기다린 값은 계속 올랐다 — behind 173(11-06) → 252, 충돌 93 → 147.

**사용자 영향**: 지원 목록의 판정은 전수 429행에서 **달라진 행이 0**이다(부팅 · 그림 · 입력 축 · 퇴행 의심 0 · §8-7). ★이용자에게 보이는 변화가 하나 있다 — **KTF · LGT 게임이 엔진의 글자 그리기(`MC_grpDrawString`)로 쓰는 글자가 10 px 위로 올라간다.** 전/후 화면을 같은 순간에 찍어 보면, 대화창 아래로 잘리던 둘째 줄이 창 안에 들어온다(§6-1 · KTF `6af589a88cf9`). 이 그리기를 쓰는 판은 표본 41행 가운데 4행이다. upstream #1517 의 수정이고 우리 쪽은 그 줄을 고친 적이 없다. 그래서 `docs/player-updates/2026-11-23-ktf-lgt-native-text-sits-in-its-box.json`(`kind: fix`)을 더했다. 속도는 내려가지 않았다(§8-5 · 표본 7개 모두 50M 틱당 CPU 가 9 ~ 16% 줄었다). `compat.json` 은 고치지 않았다. `Cargo.lock` · 엔진 `*.rs` 가 바뀌므로 착지하면 otterpebble 핀 범프가 뜬다. wasm 아티팩트는 18,019,325 → **18,376,929** 바이트(+2.0%)다.

### 0. 먼저 읽을 것

1. **동작이 갈리는 자리는 전부 우리 값이다**(총괄 판정 2026-11-23). 0551 §8 의 6자리는 시험으로 고정돼 있다(§4). upstream 이 11-06 뒤 더 만든 같은 급의 갈림은 크레이트마다 표로 적었다(§5 · 부록 B) — 합쳐서 약 150행이고 전부 우리 값을 지켰다.
2. **우리 두 호스트는 병합 뒤에도 해석기로 돈다.** upstream 의 ARM AOT(#1415)는 트리에 들어왔지만 `wie_cli` · `wie_featurephone` 은 `Options.aot = None` 을 넘긴다. AOT 를 켜는 것은 자리별 전/후를 재는 별 조각이다.
3. **설계가 둘인 자리는 우리 것 하나로 뒀다**: GC 루트(`guest_roots`) · LGT 예외 프레임 · WIPI C 타이머 · DB 슬롯(`api/database.rs`) · `run_function` 의 문맥 보관. upstream 의 짝(`ExternalRoots` · 태스크 소유 예외 상태 · 게스트 메모리 타이머 목록 · `fs.rs` · 게스트 스택에 문맥 밀기)은 트리에 있어도 우리 표에 배선하지 않았다.
4. **upstream 만 고친 자리는 병합의 뜻대로 들어왔다.** 그 가운데 게임이 보는 값이 바뀌는 것을 §6 에 모았다. 눈에 보이는 것은 글자 위치 하나를 화면으로 확인했다.
5. **0551 의 판정이 바뀐 행은 셋이다**(폐기 → 재적용 2 · 폐기 → 발신 1 · §2). 셋 다 «이름은 같은데 동작이 다르다»가 이번에 읽혀서다.
6. **못 잰 것은 §9 에 있다.** 브라우저의 속도 축 · J2ME 코퍼스 · AOT 를 켠 실행은 재지 않았다.

### 1. 재측 — 그날의 `upstream/main`

2026-11-23 19:2x KST, `git fetch origin && git fetch upstream` 직후.

| 잰 것 | 0551(11-06) | 0584(11-23 오전) | 이 회차 |
|---|---|---|---|
| `origin/main` ↔ `upstream/main` | `2e66f9d68` ↔ `10c12ec58` | `3c3986da9` ↔ `ea5640ec1` | **`642110d1a` ↔ `d5242bbff`** |
| merge-base | `44fbf265` | `44fbf265` | `44fbf265`(불변) |
| behind / ahead | 173 / 2,970 | 248 / — | **252 / 3,196**(병합 제외 251) |
| upstream 델타(merge-base 부터) | — | — | 388파일 +42,250 / −11,800 |
| 충돌 파일 | 93 | 147 | **147**(UU 115 · AA 28 · UD 4) |
| 충돌 덩어리(`<<<<<<<`) | — | 663 | 663 |
| 충돌 없이 스테이징된 것 | M 108 · A 96 · R 5 · D 4 | — | M 86 · A 112 · R 5 · D 3 |
| 양쪽이 다 고쳤는데 충돌이 없는 파일(§7) | 35(이름바꿈 너머 36) | — | **61**(이름바꿈 너머 63) |

```sh
git merge-base origin/main upstream/main             # 44fbf2652c856af11c7140f838ad223cdebab641
git rev-list --left-right --count origin/main...upstream/main   # 3196	252
git merge-tree --write-tree --name-only origin/main upstream/main   # rc=1 · 충돌 147
git -c merge.conflictStyle=zdiff3 merge --no-commit --no-ff upstream/main   # rc=1
```

**갈래 × 영역**(147행 · 미분류 0 · 표 전문은 부록 A)

| 영역 | 파일 | 폐기 | 재적용 | 발신 |
|---|---:|---:|---:|---:|
| 스캐폴딩 · 루트 · backend · test-utils · core-arm · j2me · jvm-support · skt · web | 38 | 3 | 32 | 3 |
| `wie-ktf` · `wie-lgt` | 32 | 1 | 30 | 1 |
| `wie-midp` · `wie-skvm` · `wie-wipi-c` | 48 | 10 | 7 | 31 |
| `wie-wipi-java` | 29 | 6 | 4 | 19 |
| **합계** | **147** | **20** | **73** | **54** |

해소 방식: ours **57** · 3way-edit **55** · theirs 17 · union 16 · regen 2 = 147. 0551 은 3way-edit 67 · ours 8 이었다. ours 가 는 것은 S1 · S1b · S2 가 upstream 의 API 이동(필드 owner 인자 · sleep 마감)을 우리 쪽에서 먼저 했기 때문이다 — 그 뒤 upstream 쪽 델타가 «우리가 이미 한 것»만 남은 파일은 우리 판이 그대로 답이다.

### 2. 0551 표와의 차이 행

**새로 충돌한 54행**(0551 에 없던 경로). 영역별: 스캐폴딩 3(`coverage.yml` · `rust-audit.yaml` · `docs/lgt.md`) · backend 3(`system.rs` · `task.rs` · `tests/system.rs` — 0551 §6 의 «조용한 병합»이 S2 뒤 충돌로 올라왔다) · core-arm 1(`binary_patches/hook.rs` — 그 사이 우리가 고쳐 UD 가 됐다) · `wie-skt/Cargo.toml` 1 · j2me 1 · ktf 2 · lgt 4 · midp 24 · skvm 6 · wipi-java 9. midp 의 24 는 upstream 이 11-06 뒤 `lcdui/game/*` · `rms/*` · `List` · `TextField` 를 구현하면서 우리가 먼저 넣은 같은 클래스와 add/add 로 만난 것이다.

**갈래가 바뀐 3행**

| 0551 # | 경로 | 0551 | 이 회차 | 왜 |
|---:|---|---|---|---|
| 14 | `wie-backend/src/canvas/lbmp.rs` | 폐기 · theirs | **재적용 · ours** | upstream 판은 «대체판»이지만 동작이 다르다: 모든 형을 `ArgbPixel` 로 돌려주고, 형 8 · 16 의 마스크를 투명으로 풀고, 크기 불일치와 `mask > 1` 을 거부한다. 코퍼스의 LBMP 378장(작업 목록 372 + 그 밖 6)에 크기 불일치 · `mask > 1` · 잘린 파일은 0 이고, 마스크가 선 형 8 · 16 이 361장(5판)이다 — 그 361장은 우리 판에서 불투명하게, upstream 판에서 투명하게 그려진다. 옮기는 것은 전/후를 재는 일이라 제안 1건으로 올렸다(worklog) |
| 51 | `wie-lgt/src/runtime/stdlib.rs` | 폐기 · theirs | **재적용 · 3way-edit** | 0551 의 판정은 «이름 수준»이었다. 본문을 읽으면 `malloc`(실패 시 우리 0 ↔ upstream 치명 오류) · `strncmp` · `strtol` 이 다르다. 우리 본문을 두고 upstream 만 가진 슬롯(strtok · memcmp · memchr · abort · realloc · `0x450`)을 더했다 |
| 72 | `wie-wipi-c/src/api/util.rs` | 폐기 · theirs | **발신 · union** | `inet_addr("010.001.002.003")` 이 우리는 풀리고 upstream 은 `INADDR_NONE` 이다 |

**해소 방식만 바뀐 행**은 21행이다(부록 A 의 «0551» 칸에 «해소 다름»으로 적힌 것 — LGT 6 · wipi-java 10 · 그 밖 5). 스무 행이 3way-edit → ours 이고 사유는 §1 끝에 적은 것 하나다. 나머지 하나 #63 `x_display.rs` 는 upstream 이 그 사이 `drawImageEx` · `refresh` 를 구현해 union 이 아니라 3way-edit 로 풀었다. 그 밖에 #61 `graphics_2d.rs` 는 행의 해소(3way-edit)는 같고 그 안의 `captureLCD` 한 함수만 upstream 판을 받았다.

**0551 의 upstream 칸이 낡은 곳**: #65 `api/database.rs` 는 UD(upstream 삭제)가 아니라 UU 다 — upstream 이 같은 경로에 `MC_db*` 레코드 API 를 350줄로 다시 썼다. #52 의 `0x68` 은 upstream 에서 더는 `Unk13` 스텁이 아니고 `Exit` 다(우리 `terminate_program` 과 같은 뜻).

### 3. 몫 ⑴ ~ ⑪ — 처분과 근거

| 몫 | 처분 | 근거(파일 · 시험) |
|---|---|---|
| ⑴ `binary_patches` 살리기 | **살렸다.** UD 4파일(`binary_patches.toml` · `binary_patches.rs` · `hook.rs` · `stdlib.rs`)은 우리 판, 조용히 지워지던 `parser.rs` · `patch.rs` 를 복원, `lib.rs` 의 `mod binary_patches;` · `pub mod stdlib;` 와 `Cargo.toml` 의 `md5` · `serde` · `toml` 을 되살렸다. 패치는 AOT 스냅숏 «전에» 들어간다(KTF `runtime/init.rs` 의 `install_binary_patches` → `prepare_execution` · LGT 도 같은 순서 · 재배치 이미지의 `relocated::load` 도 이제 `prepare_execution` 으로 끝난다). 설치 «뒤»의 코드 쓰기(훅이 자리를 돌려주는 것 · shadow 모드)는 upstream 계약으로는 컴파일된 코드를 무효화하지 않으므로 `ArmCore::write_code` 를 더해 다섯 쓰기 자리를 그리로 모았다 | 장부 가드 `the_ledger_lists_exactly_*` 통과 · `host_code_writes_reach_the_aot_backend_in_every_preparation_state`(새 시험) · 0540 · 0547 세 행 전/후 PASS(§8-4). ★AOT 를 켠 실행은 재지 않았다(§9) |
| ⑵ KTF `InitSvcId` 7 · GC 루트 설계 | **GC 루트는 우리 것 하나.** `InitSvcId` 는 호스트 안쪽 번호다(게스트는 그 수를 보지 않는다) — 7 은 `JavaThrowInstance` 로 남고 upstream 의 `VisitGcRoot` 는 받지 않았다. `InitParam4.fn_visit_gc_root` 는 게스트가 읽는 구조체의 칸이지만 호스트가 이미지의 방문자를 돌릴 때만 닿는다 — 우리 설계에서는 0(merge-base 와 같다). upstream 의 `ExternalRoots` 구현 · `ArmCore::stack_words` 는 뺐다 | `test_register_static_classes_keeps_live_guest_roots_without_initializing` · `test_a_guest_word_into_an_object_keeps_it_alive` · `init_param_4_wires_throw_instance` 등 통과. ★자동 병합이 덩어리 밖에서 `register_static_classes` 호출과 로더의 `nativeStrings` 필드를 지웠다 — 둘 다 컴파일은 됐다. 되돌렸다 |
| ⑶ `validate_client_bin` · `validate_jar` | **우리 둘은 통과시키고 나머지는 upstream 의 검사를 둔다.** `validate_client_bin_entry` = «재배치 표 머리이면 통과, 아니면 upstream 검사». `validate_archive` 는 `from_archive` 가 실제로 돌리는 jar(`<AID>.jar`, 없으면 하나뿐인 jar)를 본다 | `validation_accepts_the_stub_and_the_relocation_prefixed_image_only` · `validate_archive_finds_the_jar_from_archive_runs`(새 시험 2) · `jar_filename_falls_back_to_the_only_jar` 유지. 코퍼스 client.bin 287개 = 표준 머리 284 + 재배치 3, 거부되는 것 0. 재배치 3행 전/후 paints 같음(§8-4) |
| ⑷ LGT 예외 프레임 | **우리 것(스레드별 프레임 장부).** 자동 병합이 덩어리 밖에서 upstream 의 태스크 소유 설계를 `LgtTaskRunner::run` 에 들이고 우리 `exception::init` 을 지웠다 — 되돌렸다 | `wie-lgt` 시험 101개(전 95) 통과 · 우리 예외 시험 12개 그대로 |
| ⑸ `wie-wipi-c/src/api/database.rs` | **남겼다 — `origin/main` 과 바이트가 같다.** upstream 의 `fs.rs` 는 다른 저장소(`System::filesystem()`)와 다른 핸들을 쓴다. 표 하나를 반씩 섞을 수 없고, 통째로 옮기면 저장이 다른 곳으로 간다. `fs.rs` 는 트리에 있으나 우리 표에 배선하지 않았다. KTF 표 이름만 upstream 을 따랐다(`Database` = 5 · `Fs` = 7) | `database.rs` 시험 20개 · `database_slot_11_answers_free_storage`. ★같은 경로를 양쪽이 가져 다음 병합마다 충돌한다(S4 후보: 우리 모듈 이름 바꾸기) |
| ⑹ 타이머 | **우리 것(호스트 이벤트 큐 + pacing).** 걸려 있는 타이머에 `SetTimer` 는 −1(upstream −5). `set_timer(due, pace_from, callback)` 3인자. upstream 의 새 스텁(`execute` · `get_exec_names` · `get_access_level` 등)은 받았다 | `test_set_timer_on_a_pending_timer_fails` · `test_unset_timer_cancels_a_pending_timer` · `test_when_a_timer_may_keep_a_tick_alive` |
| ⑺ `Cargo.toml` | members 합집합(우리 `wie_cli` · `wie_featurephone` · `wie_identify` + upstream 의 AOT 3크레이트) · `exclude = ["wie-app"]` 유지 · RustJava `=0.5.1` 유지 · `md5` 복원. upstream 은 CLI 의존을 `cli` 기능 뒤 optional 로 옮겼다 — 우리는 그 optional 들을 종전처럼 `cfg(not(target_arch = "wasm32"))` 블록 안에 둔다(`src/main.rs` 에 짝 `cfg`). wasm lint 게이트가 `-p` 없이 기본 기능으로 이 패키지를 빌드하기 때문이다 | `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0. ★이제 그 게이트가 엔진 크레이트 12개(`wie-backend` ~ `wie-lgt`)를 wasm32 로 lint 한다(종전에는 루트 크레이트가 wasm 에서 빈 라이브러리였다) |
| ⑻ `Cargo.lock` · `wasm-bindgen` | 우리 lock 에서 출발해 cargo 가 필요한 것만 더했다(40 패키지 추가 · 567 → 602). **`wasm-bindgen` 0.2.108 · `js-sys` · `web-sys` 0.3.85 그대로** | `grep -A1 '^name = "wasm-bindgen"$' Cargo.lock` → `version = "0.2.108"` · `npm run build:wasm` rc=0 · 계약 143 pass |
| ⑼ `rust.yml` | ours — `origin/main` 과 diff 0줄. required check 이름 `rust_ci (macos-latest, stable)` 그대로 | `git diff origin/main -- .github/workflows/rust.yml` 0줄 · PR 의 check 목록(§8-6) |
| ⑽ `get_protos()` 중복 | 67항목 · 중복 0. 우리 목록 + upstream 의 새 `ProxyCard`, `origin/main` 에 이미 있던 `ImageObserver` 중복 1건 제거 | preload 순서 시험 통과 |
| ⑪ `GeneralUser.sf3` | 우리 판 복원(upstream 은 삭제) | `contract-roundtrip` 시나리오 S 통과(§8-3) · `check-audio-worklet.mjs` rc=0 |

### 4. §8 의 6자리 — 병합 뒤에도 우리 값

| 자리 | 값 | 병합 뒤 그 값을 단언하는 시험(전부 `cargo test --all` 안에서 통과) |
|---|---|---|
| `HandsetProperty` `VOLUMELEVEL` | `"3"` | `handset_property::test::test_volume_level_is_a_number`(이 회차가 `"3"` 단언으로 조였다) |
| WIPI `Player.stop(null)` | 조용히 `false` | `media::player::test::test_null_clip_reports_failure` |
| WIPI `Player.stop` 반환 | STARTED 였을 때만 `true` | `media::player::test::test_stopping_an_idle_clip_reports_false` |
| MIDP `Canvas` GAME_A..D | 7 · 9 · * · # | `canvas::test::canvas_and_game_canvas_fullscreen_preserve_paint_and_keys`(upstream 의 시험 · 기대값을 우리 것으로 고치고 `// fork:` 주석) |
| LGT SVC `0xf0` / `0x19c` / `0x44c` / `0x68` | 외곽선 / `ListDatabases` / `ListDataStores` / 종료 | `svc_ids::tests::wipic_svcs_0x68_0xf0_0x19c_0x44c_keep_the_fork_meaning`(새 시험 · id → 변형). `0x19c` → `list_databases` 는 match 가지 하나이고 실행 시험은 없다 |
| 파일시스템 이름공간 | aid(저장 키 불변 · 이용자 저장 이전 0) | `wie-backend/tests/system.rs` `files_are_scoped_by_the_aid_not_the_pid`(새 시험). `System::new` 가 `FilesystemOverlay` 에 aid 를 넘긴다. J2ME 의 저장 id 도 종전대로다(upstream 은 `MIDlet-1` 제목으로 옮겼다) |

같은 급의 자리 하나를 더 고정했다: **`DatabaseRepository::open` 은 계속 «없으면 만든다».** upstream #1575 는 `create` 를 따로 두고 `open` 을 만들지 않게 바꿨다. 우리 `wie-wipi-c` DB 슬롯은 «열어서 만든다»(`if !exists { open }`)에 기대므로 호스트 둘과 `test-utils` 의 `open` 은 종전대로 만들고, `create` 는 더했다. `usage`(upstream 이 지움)도 남겼다.

### 5. upstream 이 11-06 뒤 더 만든 갈림 — 전부 우리 값

크레이트별 표 전문은 부록 B 에 그대로 실었다. 수와, 닿는 범위가 넓은 것만 여기 적는다.

| 크레이트 | 행 | 닿는 범위가 넓은 것 |
|---|---:|---|
| `wie-core-arm` | 10 | `run_function` 이 게스트 스택을 건드리지 않는다 · ★게스트 힙 256 MB 를 처음에 다 매핑(upstream 은 닿을 때) · ★스레드 스택 1 MB(upstream 64 KB) · null 페이지 읽기 0 |
| `wie-wipi-c` | 14 | `SetTimer` −1 · `knlAlloc(0)` NULL · 메모리 2 MiB · `PHONENUMBER` · sprintf 바이트 그대로 · `grpRepaint` = `request_redraw` · 클립 사각형 · DB 슬롯 |
| `wie-ktf` | 13 | 클래스 로더 parent-first · 예외 처리기 사슬 걷기 · 재배치 이미지 · jar 폴백 · `call_native` 상위 워드 |
| `wie-lgt` | 약 30 | 예외 프레임 · Java 가져오기 35/36 번호 · stdlib `malloc` · 그래픽(우리 모듈 통째) · 필드 배치(#1580) · 배열 형 토큰(#1606) |
| `wie-midp` | 29 | `createImage(w,h)` 투명 검정(upstream 흰색) · 그리기 이벤트가 `handlePaintEvent` · SKVM 명령 띠 18 · `flushGraphics` 는 다시 그리기 요청만 |
| `wie-skvm` · `wie-skt` · `wie-j2me` | 25 | SKT `m.*` 속성 · `XFile.fsavail` · 멀티탭 · `WieAudioClip` · J2ME 저장 id |
| `wie-wipi-java` | 약 25 | Card 위쪽 여백 · 입력기 · lwc `Component` 본문 · Clip 음량 · `Image.drawImage` 변환 · `setRGBPixels` 의 `bpl` 단위 |

★위 표에서 별표 둘(`wie-core-arm` 의 힙 매핑 · 스택 크기)은 «양쪽이 고친 자리»가 아니다. upstream 만 고쳤고 충돌도 없었다 — 그대로 두면 upstream 값이 들어온다. 게스트가 보는 것(한 번도 할당하지 않은 힙을 읽으면 0 ↔ 폴트 · 깊은 재귀의 여유)이 바뀌므로 우리 값으로 되돌렸다. 각각 한 줄이다.

★`wie-wipi-java` 의 두 행(`Image.drawImage` 변환 사상 · `bpl` 단위)은 «upstream 이 우리의 숨은 결함을 고친 것»으로 읽힌다(부록 B). 규칙대로 우리 값을 뒀다 — 옮기는 것은 자리별 전/후를 재는 일이다.

### 6. upstream 만 고친 자리 — 병합으로 들어온 동작 변화

우리가 그 줄을 고친 적이 없어 병합의 뜻대로 upstream 값이 됐다. 게임이 보는 것이 바뀌는 것만 적는다.

#### 6-1. 화면으로 확인한 것 — `MC_grpDrawString` 의 y 는 기준선이다(#1517)

- `wie-wipi-c/src/api/graphics/primitives.rs` `draw_text` 가 y 에서 글꼴 높이(10)를 뺀다. 글자가 종전보다 10 px 위에 그려진다.
- **전/후 화면**: KTF `6af589a88cf9` 의 같은 키 걸음(6번째 키 뒤)을 전 · 후 바이너리로 같은 순간에 찍었다. 전: 대화창의 둘째 줄이 창 아래로 잘린다. 후: 두 줄이 창 안에 들어온다. 캡처는 증적 디렉터리에 있다(저장소 밖).
- **얼마나 닿나**: KTF · LGT 코퍼스 377파일을 sha 순으로 9번째마다 뽑은 41행을 10초씩 돌려 `MC_grpDrawString` 호출을 셌다 — 4행(KTF 2 · LGT 2)이 부른다. 나머지는 자기 글꼴을 점으로 찍는다.
- 되돌리는 법: 그 두 줄과 시험 `strings_are_positioned_at_the_requested_baseline` 을 지운다.

#### 6-2. 읽기만 한 것(자리별로 재지 않았다 — 전수 census 가 판정만 본다)

| 자리 | 전(우리 = merge-base) | 후(upstream) |
|---|---|---|
| KTF 네이티브 클래스의 `prepare` | — | 상속한 vtable 칸을 재정의로 고쳐 쓴다(#1450). 재배치 이미지 3행은 전/후 paints 가 같다(§8-4) |
| KTF · LGT WIPIC SVC 한 번마다 네이티브 프레임 | 그 호출이 만든 Java 객체가 바깥 프레임에 쌓였다 | SVC 가 돌아오면 놓는다(#1542). 게스트가 쥔 것은 우리 스캔이 계속 붙든다 |
| KTF UIC 표 | 슬롯 12 부터 번호가 다르고 대부분 스텁 | 두 칸이 끼고 `paint` · `handle_event` 등 구현(#1596) |
| KTF · LGT 입력기 `im::*` · `GetAccessLevel`(`wie.slvl`) · `htonl` | 스텁 / 미구현 | 구현 |
| `KtfWIPICContext::alloc` | 그대로 | 0 으로 채운다(#1572) |
| KTF ADF 에 `MClass` 가 없는 꾸러미 | 클래스 `""` 를 찾다 실패 | «Main class not found» |
| LGT stdlib `0x424` | 등록하는 순간 콜백을 불렀다 | `atexit` 스텁 — 부르지 않는다 |
| LGT `vsprintf` 인자 | 미리 다 읽었다 | 쓸 때 읽는다 — 매핑 안 된 워드를 쓰면 오류 |
| LGT WIPIC id 21개 · stdlib 6칸 · ABI 16행 | «Unknown …» 로 치명 오류 | 처리한다 |
| `MC_grpDestroyImage` | 픽셀 · 마스크 버퍼를 놓지 않았다 | 놓는다(우리가 NULL 가드를 더했다) |
| SKT `loadable_jar` | `\x20\0…` 머리의 jar 만 SKT | 클래스가 `com/skt/m/` · `com/xce/` 를 가리키는 plain jar 도 SKT. `content-key.mjs check` 429/429 같음(§8-3) |
| SKT `from_jar` | 주 클래스를 인자로 받았다(호스트는 `None`) | 매니페스트의 `MIDlet-1` 에서 읽는다 |
| SKT `AudioSystem.getAudioClip(format)` | 형식 무관 | `"mmf"` 가 아니면 `UnsupportedFormatException` |
| SKT `Device.invokeWAPBrowser` | 스텁 | 앱을 끝낸다 |
| J2ME 매니페스트 | 정확한 이름 · 부팅 뒤 | 대소문자 무시 · 로드 때 읽고 없으면 오류 |
| MIDP `RecordEnumeration` · GameCanvas 키 상태 · `getRGB` 인자 검사 · `drawRGB` scanlength | 우리 스텁 / merge-base | upstream 구현(우리 것이 스텁이거나 실패하던 자리 — 부록 B `wie-midp` 1-B) |
| `Display.setCurrent` 를 `hideNotify` 안에서 부르기 | 호스트 스택이 넘칠 때까지 재귀 | 작은 가드로 한 번만 알린다(이 회차가 더함) |
| **smaf 0.1.2 → 0.1.4** | `MMMD` 머리의 길이를 넘겨 끝까지 읽었다 | 머리가 말한 길이까지만 읽는다. 코퍼스 SMAF 6,692개 가운데 길이를 줄여 적은 것 172개 · 0 으로 적은 것 0개 — 줄여 적은 172개는 종전에도 우리 `parse_smaf_in` 이 그 길이에서 잘라 읽었다. 0 으로 적은 것은 우리 픽스처 둘뿐이었고 고쳤다(§8-3) |

### 7. 충돌 밖 — git 이 조용히 합친 61파일

`git diff --name-only 44fbf265 origin/main` 과 `… upstream/main` 의 교집합 208 에서 충돌 147 을 뺀 **61**(이름바꿈 너머로는 `src/filesystem.rs` · `src/window.rs` 가 더해져 63)을 크레이트 담당이 하나씩 읽었다. 영역별: `.github` 1 · backend 3 · core-arm 4 · jvm-support 2 · ktf 5 · lgt 8 · midp 14 · skvm 9 · util 1 · wipi-c 1 · wipi-java 13.

**틀리게 합쳐져 손으로 고친 곳**(전부 컴파일은 되던 것이다 — 덩어리 밖이라 표식도 없었다)

| 파일 | 무엇이 잘못 합쳐졌나 | 처분 |
|---|---|---|
| `wie-backend/src/lib.rs` | `extract_zip` 의 이름 읽기 블록이 두 번(양쪽이 같은 수정을 했다) | 하나 지움 |
| `wie-ktf/src/emulator.rs` | `register_static_classes` 호출이 사라짐 | 복원 |
| `wie-ktf/.../ktf_class_loader.rs` | `nativeStrings` 필드 proto 가 사라짐(우리 `init` 은 그 필드에 쓴다) | 복원 |
| `wie-lgt/src/emulator.rs` · `jvm_implementation.rs` | upstream 의 태스크 소유 예외 설계가 들어오고 우리 `exception::init` 이 사라짐 | 우리 것으로 |
| `wie-midp` `Graphics.drawRegion` 등 8곳 | 양쪽의 변환이 둘 다 적용됨(영역이 두 번 변환된다) 외 | 고침(부록 B) |
| `wie-midp-vendor` `FullCanvas` | upstream 의 새 클래스 + 우리 `standardKeyCodes` → J2ME 에서 UP 이 −11 | 고치고 upstream 시험에 단언 추가 |
| `wie-skvm` `x_display.rs` · `x_file.rs` | `drawImageEx` 가 두 번 · 우리 `refresh` 가 upstream 것으로 · 시험 모듈이 뒤섞임 | 다시 세움 |
| `wie-wipi-java` `text_box_component.rs` · `annunciator_component.rs` | `init_with_height` 가 두 번 · upstream 의 `Component.h = 0` 쓰기가 들어옴 | 우리 판으로 |
| `wie-wipi-c/src/api/graphics/image.rs`(upstream 만 고침) | 565 + 마스크 재작성이 우리 `decode_image_framebuffer` 를 지움 | 우리 판으로 |
| `wie-core-arm` `allocator.rs` · `thread.rs`(upstream 만 고침) | 힙 지연 매핑 · 스택 64 KB | 우리 값으로(§5) |

**자동 병합이 떨어뜨린 것**(0551 §6): `binary_patches/{parser,patch}.rs` · `lib.rs` 의 `mod` 두 줄 · `wie-core-arm/Cargo.toml` 의 세 의존 · `GeneralUser.sf3` — 전부 복원(§3 ⑴ · ⑪). `get_protos()` 의 덩어리 밖 중복은 이번 트리에서는 `ImageObserver` 1건이었다.

**upstream 이 지워 같이 빠진 시험**: `wie-wipi-java` 의 merge-base 스텁 시험 4개(`backlight` · `event_queue` · `vibrator` · `volume` — upstream #1441 정책) + 우리 LED 시험 1개(0551 #76 의 처분) · `wie-util` 의 `read_generic_reads_into_initialized_storage`. upstream 시험 가운데 «upstream 쪽 동작»을 단언해 싣지 않은 것은 크레이트별 노트에 이름으로 있다(부록 B). 그래서 `coverage.yml` 을 이 브랜치에서 손으로 돌렸다(§8-6).

**포크 규칙에 걸려 고친 upstream 파일 하나**: `wie-skvm/tests/data/resource.jar`(116바이트 · 2바이트짜리 `data.bin` 하나) → `resource.zip`. `audit-no-leak.sh` 는 추적되는 `.jar` 를 거부한다(Constraint 9). 같은 바이트다.

### 8. 검증식 — 실제 출력

머리 `c7dfac8a7`(병합 커밋 `23a94715f` + 손본 커밋 하나) · 2026-11-23 · load1 5.6 ~ 14.8.

#### 8-1. V0 네 게이트 + beta

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | rc=0 |
| `cargo clippy --all -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | rc=0 · **67 스위트 · 982 passed · 0 failed**(병합 전 58 스위트 · 880). 그 뒤 §4 의 aid 시험 하나를 더하고 다섯 가지를 다시 돌렸다 — 전부 rc=0 · **983 passed** |
| `cargo +beta clippy --all -- -D warnings` | rc=0. `unused dependency` 매니페스트 경고가 `wie_cli` 에 7줄 찍힌다(이 병합이 건드리지 않은 매니페스트 · rc 0) |

#### 8-2. 러너 블록(debug 빌드 · load1 7.6)

| 픽스처 | 명령 | 결과 |
|---|---|---|
| `test_data/draw_j2me.jar` | `wie_validate <f>` | `PASS` · rc=0 · paints 1 |
| `test_data/helloworld_ktf.zip` | 〃 | `PASS` · rc=0 · clean exit |
| `test_data/helloworld_lgt.zip` | 〃 | `PASS` · rc=0 · clean exit |
| `test_data/keydraw_ktf.zip` | `--inject --expect-last-frame` | `PASS` · rc=0 · 27/27 · paints **79** · `content` true · `last_frame_content` true |
| `test_data/keydraw_lgt.zip` | 〃 | `PASS` · rc=0 · 27/27 · paints **55** · true · true |
| `test_data/text_j2me.jar` | `--timeout 5` | `PASS` · rc=0 · paints 1 |

`keydraw_*` 의 79 · 55 는 병합 전(0584 §5)과 같은 값이다.

#### 8-3. wasm · 계약 · 감사

| 검증식 | 결과 |
|---|---|
| `npm run build:wasm` | rc=0 · `wie_web_bg.wasm` 18,376,929 바이트(전 18,019,325 · +357,604 · +2.0%) |
| `node scripts/check-engine-contract.mjs` | `143 pass, 0 violation(s)` · rc=0 — wasm 표면은 바뀌지 않았다(계약 JSON 수정 0) |
| `node scripts/contract-roundtrip.mjs` | 1회째 **67/73 · rc=1** — 시나리오 S 의 «소리가 난다» 6개가 `voices 0 · rms 0.0000`. 원인은 우리 픽스처(`make-sound-fixture.mjs`)의 SMAF 가 `MMMD` 길이를 0 으로 적은 것이다(smaf 0.1.4 는 그 길이까지만 읽는다 — §6-2). 픽스처의 길이를 실제 값으로 고친 뒤 **73/73 · rc=0** |
| `npm run audit`(`audit-no-leak.sh`) | 1회째 rc=1(`wie-skvm/tests/data/resource.jar` 추적 — §7 끝) → 이름을 바꾼 뒤 `AUDIT PASSED` · rc=0 |
| `node scripts/content-key.mjs check` | rc=0 · 429행 전부 디스크에 있음 · 커밋된 `contentKey` == native 429/429 · native == wasm `identify()` 429/429 · 행의 platform/model == `identify()` 429/429 |
| `cargo audit` | 취약점 0 · 경고 1(`ttf-parser` unmaintained `RUSTSEC-2026-0192` — 병합 전과 같다) |
| `contract` 잡의 오프라인 검사기 12종(`check-worklog-json` · `check-doc-liveness-parity` · `check-docs-report-serial` · `check-parked-workflows` · `check-engine-runner-fixtures` · `player-data` · `check-compat-revert` · `check-audio-worklet` 등) | 전부 rc=0 |

#### 8-4. `smoke_gate.sh` 전/후 · 0540 · 0547 세 행 · 재배치 세 행

`WORKING_DIR=game_lab/working PLATFORM_FILTER=<p> bash scripts/smoke_gate.sh` · 플랫폼마다 `build-slot run --long` 임대 1건 · debug `wie_validate`. 전 = `origin/main` `642110d1a`(스크래치 트리 · 자기 `target/`), 후 = 병합 커밋 `23a94715f`.

| 플랫폼 | 전: 실행 | 전: baseline 대조 | rc | 후: 실행 | 후: baseline 대조 | rc |
|---|---|---|---|---|---|---|
| lgt | 54 PASS / 0 FAIL | 52 대조 · 0 absent · 회귀 0 | 0 | 54 PASS / 0 FAIL | 52 대조 · 0 absent · **회귀 0** | 0 |
| skt | 50 PASS / 0 FAIL | 50 대조 · 0 absent · 회귀 0 | 0 | 50 PASS / 0 FAIL | 50 대조 · 0 absent · **회귀 0** | 0 |
| ktf | 190 PASS / 0 FAIL | 190 대조 · 0 absent · 회귀 0 | 0 | 190 PASS / 0 FAIL | 190 대조 · 0 absent · **회귀 0** | 0 |

걸린 시간 · load1: 전 lgt 20:44 ~ 20:58(8.2 → 7.5) · skt ~ 21:11 · ktf ~ 21:59 / 후 lgt 21:23 ~ 21:38(8.0 → 13.9) · skt ~ 21:51 · ktf ~ 22:40(6.9 → 5.8). ktf 임대는 한 건이 49분이다(스크립트에 조각 내기가 없다 · 0551 과 같다).

**0540 · 0547 세 행과 재배치 이미지 세 행**(release · `--inject --keep-timeout --timeout 30 --relaunch 1 --pacing 8` · 키 27개 · 패닉 0):

| 행 | 무엇 | 전 | 후 |
|---|---|---|---|
| `01f05f8231f4` | LGT 잠금 해제(0540) | PASS · 27/27 · paints 711 · `last_frame_content` true | PASS · 27/27 · paints 574 · true |
| `974e0df9ab1e` | KTF 설치 인증(0547) | PASS · 27/27 · paints 389 | PASS · 27/27 · paints 379 |
| `f770b15f8876` | 〃 | PASS · 27/27 · paints 387 | PASS · 27/27 · paints 376 |
| `b907b0faf483` | KTF 재배치 이미지 | PASS · 27/27 · paints 298 | PASS · 27/27 · paints 299 |
| `1d5831e42a8a` | 〃 | PASS · 27/27 · paints 251 | PASS · 27/27 · paints 250 |
| `83fc429f9cbe` | 〃 | PASS · 27/27 · paints 80 | PASS · 27/27 · paints 80 |

`01f05f8231f4` 의 711 → 574 는 전/후 차이가 아니다. 전 · 후를 같은 순간에 두 번 더 돌리면 584 ↔ 573, 712 ↔ 702 다 — 이 판은 «전» 혼자서도 584 와 712 를 오간다(같은 키가 다른 장면에 닿는 두 갈래).

★잠금 «너머»는 `last_frame_content` 와 paints 수로만 봤다. 0540 · 0547 이 쓴 긴 키 경로(60키 · 전투 화면)는 다시 밟지 않았다.

#### 8-5. 속도 축 — `perf-watch.sh compare`(native)

`WIE_PERF_AXES=native bash scripts/perf-watch.sh compare <origin/main 빌드> <병합 빌드> <out>` · 표본 7 · 3회 · A · B 를 같이 돌린다 · load1 6.6 ~ 10.2.

| 표본 | 회별 «B 가 A 보다 느린 %» | 중앙값 | 판정 |
|---|---|---:|---|
| `0606d43702e7` | −11.3 · −11.3 · −11.0 | −11.3 | ok |
| `1065985081fc` | −15.8 · −15.6 · −16.5 | −15.8 | ok |
| `155972cac664` | −9.7 · −9.6 · −10.2 | −9.7 | ok |
| `3cc7a9b4cb15` | −10.2 · −10.3 · −10.2 | −10.2 | ok |
| `8238f5449347` | −9.4 · −7.8 · −9.6 | −9.4 | ok |
| `835acd535a5f` | −14.1 · −14.4 · −14.2 | −14.2 | ok |
| `d1234e0ddb75` | −13.2 · −15.4 · −14.5 | −14.5 | ok |

**퇴행 0 · 1차 경보 0 · 미측정 0/7.** 값은 50M 틱당 CPU ms 이고 음수는 «덜 든다»다 — 일곱 표본 모두 9 ~ 16% 줄었다. 어디서 줄었는지는 가르지 않았다(upstream 의 해석기 쪽 변경이 여럿 같이 들어왔다). ★브라우저 축(30초 paints)은 재지 않았다(§9).

#### 8-6. PR 의 CI

`gh pr checks 617 -R Jun025/wie`(머리 `c7dfac8a7` — 병합 커밋 + 손본 커밋):

| check | 결과 |
|---|---|
| `build-web` | pass · 4m4s |
| `contract` | pass · 9m59s(실제 브라우저 왕복 포함) |
| `pr_audit` | pass · 11s(`Cargo.lock` 이 움직여 돈다) |
| `rust_ci (ubuntu-latest, stable)` | pass · 7m35s |
| `rust_ci` 나머지 다섯 레그 | pass · 2 ~ 5초 — 빈 통과다(2026-11-07 부터 PR 에서는 일을 하지 않는다 · 플랫폼 보장이 아니다) |
| `dependabot` | skipping |

이 기록을 실은 문서 커밋의 값은 회신에 적는다. `coverage` 는 PR 에서 돌지 않는다.

**플랫폼 · 커버리지는 손으로 돌렸다**(이 병합은 플랫폼에 닿는 upstream 코드를 들이고 시험을 지우고 옮긴다 — `AGENTS.md` «That output is not a platform guarantee»):

| 실행 | 결과 |
|---|---|
| `gh workflow run rust.yml --ref <브랜치>` → run `45186257017` | **여섯 레그 전부 success**(ubuntu · macOS · windows × stable · beta). ★`rust_ci (macos-latest, beta)` 는 세 번째 시도에서 통과했다 — 첫 두 시도는 러너를 배정받지 못한 채 15분 뒤 `cancelled`(단계 0개 · `runner_name` 빈 값)였다. 코드의 실패가 아니다 |
| `gh workflow run coverage.yml --ref <브랜치>` → run `45186262397` | success |

**주 1회 그물의 상태**(읽기만 — 소유는 착지 회차다): `gh run list --workflow=rust.yml --event schedule -L1` 과 `--workflow=coverage.yml --event schedule -L1` 은 둘 다 **빈 목록**이다. `main` 의 손 실행(`--branch main --event workflow_dispatch`)도 빈 목록이다. `doc-liveness` 의 마지막 schedule run 은 2026-10-31(success)이다. schedule 이 서 있는 것은 이미 발권된 `wie-schedule-events-stopped-since-1104-reregister` 의 몫이다(`STATE.md`).

#### 8-7. 전수 census → `report` → 대조

주간 퇴행 감시(`scripts/regression-census.sh`)가 `origin/main` 에 하는 것을 이 브랜치의 머리에 손으로 했다 — 같은 저장소 스크립트 · 같은 순서다(그 스크립트를 그대로 부르지 않은 것은 첫 걸음이 주간 속도 원장에 줄을 쓰기 때문이다).

```sh
# 바이너리 = 머리 c7dfac8a7 의 release wie_validate · 기준 = origin/main 의 docs/player-data/compat.json(429행)
bash scripts/census-drive.sh --bin $R/wie_validate --out $R/out --only probe game_lab/working game_lab/broken   # 임대 10건 · 22:40 ~ 01:04
node scripts/playability-census.mjs report --out $R/out --pin c7dfac8a7…
node scripts/regression-diff.mjs changed --base $R/base-compat.json --new $R/out/compat.json                 # 0줄
bash scripts/census-drive.sh --bin $R/wie_validate --out $R/out --only long --titles $R/long.titles …           # 가드 7행 × 600초 · 01:04 ~ 01:24
node scripts/regression-diff.mjs candidates …                                                                  # 0
node scripts/regression-diff.mjs report …                                                                      # 퇴행 의심 0
node scripts/check-compat-revert.mjs
```

| 잰 것 | 값 |
|---|---|
| 모집단 | 464파일 → 고유 **429행**(`compat.json` 의 429행 전부) · `--jobs` 3(기본) · load1 6 ~ 74 |
| 프로브(30초 · 키 27개 A + 키 0개 B) 축 — 후 | boot ok 427 · no 2 / render ok 427 · no 2 / input ok 427 · unknown 2 |
| `regression-diff changed`(boot · render · input 이 기준과 다른 행) | **0행** |
| 소리(프로브에서 아무것도 못 들은 행) | 11행. `main` 을 잰 지난 세 주간 회차(11-01 · 11-11 · 11-15)도 11행이고 집합은 11-01 과 같다(11-15 와는 한 행씩 드나든다) — 프로브 밖 소리는 규칙상 내림으로 세지 않는다 |
| 오래 하기(600초) — 가드 7행 | 6행 `ok`(`49ade89578c5` · `ddd885583b15` · `13d7e3c21856` · `1b107b96bf4e` · `a30bbe008b5e` · `b475b6399684` — 판정 `playable` 그대로 · 소리 `ok`). `4ece6eeeaa04` 는 `short`(요청 시간 전에 스스로 끝난다 — `STATE.md` «다음» ⑵ 가 이미 적은 행이고 규칙상 «이번 회차 장주행 없음»으로 읽는다) |
| `regression-diff candidates` | **0** — 짝 재측 대상이 없다 |
| `regression-diff report` | **퇴행 의심 0행 · 내림 후보 0행 · 올림 0 · 정책 행 축 변화 0 · 미측 0** · 측정 행 429 |
| `node scripts/check-compat-revert.mjs` | `compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 0` |

- `report` 가 낸 `compat.json` 은 저장소에 넣지 않았다(행 수정 금지 · 프로브만 잰 행은 그 표에서 `limited` 로 읽힌다 — 장주행이 없어서이고 판정이 내린 것이 아니다).
- ★이 표가 못 보는 것: 422행의 600초 구간 · 진도 축 · 그림의 «옳음»(글자 위치 같은 것 — §6-1 은 그래서 화면으로 봤다) · 브라우저.
- 판정 문서는 증적 디렉터리의 `census-regression.md` 다(sha12 만).

### 9. 못 잰 것

| 못 잰 것 | 왜 · 무엇이 대신하나 |
|---|---|
| AOT 를 켠 실행(`Options.aot = Some`) | 우리 두 호스트는 `None` 이다. 켰을 때의 패치 × AOT 는 단위 시험 하나가 상태 셋을 본다(§3 ⑴). ★null 페이지 읽기 규칙은 AOT 로 컴파일된 코드에서는 성립하지 않는다 — 그 길의 폴트는 종단이고 되돌려 다시 해석할 수 없다(upstream 계약). AOT 를 채택하는 조각이 져야 한다 |
| 브라우저의 속도 축 · 실제 브라우저에서의 게임 | `perf-watch` 는 native 축만 돌렸다. wasm 은 빌드했고 `contract-roundtrip` 73/73 이 커밋된 픽스처의 부팅 · 키 · 소리를 본다 |
| §5 의 약 150행 각각의 전/후 | 전부 우리 값이라 «전»과 같다는 것이 주장이고, 그 주장은 크레이트 시험 982개와 census 가 진다. 행마다 따로 돌린 것은 없다 |
| §6-2 의 upstream 유입 각각 | 읽기만 했다. 화면으로 본 것은 글자 위치 하나(§6-1)다 |
| 0540 · 0547 의 긴 키 경로(전투 화면 도달) | 30초 · 키 27개까지만(§8-4) |
| J2ME 코퍼스 | `compat.json` 에 J2ME 행이 없다. 러너 블록의 두 jar 와 `test_pacing` 이 신호다 |
| upstream 의 호스트(`wie`(루트 CLI) · `wie-web` · `wie-app`) | 루트 CLI 와 `wie-web` 은 컴파일과 clippy 까지(`cargo clippy --all`). 실행하지 않았다. `wie-app` 은 워크스페이스에서 빠져 있다 |
| 주 1회 CI(schedule) | `rust.yml` · `coverage.yml` 의 schedule run 은 여전히 0건이다(§8-6). 그래서 이 브랜치에서 둘을 손으로 돌렸다 |

### 10. 남긴 것

- **S4(병합 뒤 정리)는 하지 않았다.** `tag-release.yaml` 의 PARKED 머리에서 거짓이 된 한 문장만 고쳤다(검사기 통과). upstream `AGENTS.md` 의 Attribution 조항 · `docs/lgt.md` 의 upstream 판(우리 판을 뒀다 — upstream 의 +310줄 재작성은 싣지 않았다) · `api/database.rs` 의 경로 충돌 · `wie-ktf/Cargo.toml` 의 안 쓰는 `anyhow` 는 그 조각 몫이다.
- **S5(발신)는 하지 않았다.** 발신 급은 54파일이다(0551 은 29).
- **`compat.json` 의 행은 고치지 않았다.**
- 제안 1건(worklog): SKT LBMP 의 마스크 투명(§2 의 #14).

### 부록 A. 분류표 — 147행 · 미분류 0

열: 상태 · 경로 · 판정 · 해소 · 0551 의 행과 판정(없으면 `new`) · 우리 델타 · upstream 근거. 칸은 담당이 적은 그대로다(영문 섞임).

| # | 상태 | 경로 | 판정 | 해소 | 0551 | 우리 델타 | upstream 근거 · 비고 |
|---:|---|---|---|---|---|---|---|
| 1 | UU | `.github/workflows/coverage.yml` | **재적용** | ours | new | fork CI: weekly+dispatch trigger instead of every push (0553) | theirs: permissions + concurrency block on `on: [push]` |
| 2 | UU | `.github/workflows/rust-audit.yaml` | **재적용** | ours | new | fork CI: read-only permissions, cargo audit by job status (Issues disabled on fork) | theirs: `checks: write`/`issues: write` permissions for rustsec/audit-check |
| 3 | UU | `.github/workflows/rust.yml` | **재적용** | ours | #1 재적용 · same | fork CI (required check name `rust_ci (macos-latest, stable)`) | theirs: xcode-27 leg, iOS lint, concurrency |
| 4 | AA | `.github/workflows/tag-release.yaml` | **재적용** | 3way-edit | #2 재적용 · same | PARKED header (one sentence reworded: scripts now in tree) | theirs body: ADMIN_GH_TOKEN checkout, taiki-e cargo-edit |
| 5 | UU | `.github/workflows/web.yaml` | **재적용** | ours | #3 재적용 · same | parked to workflow_dispatch | theirs: PR preview deploy |
| 6 | UU | `AGENTS.md` | **재적용** | ours | #4 재적용 · same | fork charter | theirs: Attribution + engineering principles (S4 decision) |
| 7 | UU | `Cargo.lock` | **폐기** | regen | #5 폐기 · same (regen seeded from OUR lock: cargo added 40 packages, wasm-bindgen stays 0.2.108) | our lock | upstream lock: wasm-bindgen 0.2.129 |
| 8 | UU | `Cargo.toml` | **재적용** | 3way-edit | #6 재적용 · same | members + exclude wie-app + `=0.5.1` pins + md5; cli deps stay behind the not-wasm32 target gate | theirs: 0.2.32, AOT crates, `cli` feature, zip ^9, wie-midp-vendor |
| 9 | UU | `README.md` | **재적용** | ours | #7 재적용 · same | fork web-service section | theirs: wording, preview paragraph |
| 10 | UU | `docs/lgt.md` | **재적용** | ours | new | fork LGT notes (points at docs/lgt_abi.md) | theirs: full rewrite of the same doc (+310) — not carried |
| 11 | UU | `package-lock.json` | **폐기** | regen | #8 폐기 · same (= ours; package.json is ours) | fork ops lock | upstream webpack frontend lock |
| 12 | UU | `package.json` | **재적용** | ours | #9 재적용 · same | fork ops package | theirs: sentry/tauri/webpack bumps |
| 13 | UU | `test-utils/src/lib.rs` | **재적용** | union | #10 재적용 · same | ledger + run_jvm_test_until_exit/_with_runner + TestScreen exports | theirs: native_executor::TestNativeExecutor |
| 14 | UU | `test-utils/src/platform.rs` | **재적용** | 3way-edit | #11 재적용 · same | audio_log, paint counter, redraw flag, stepping clock; `open` creates + `usage` kept | theirs: audio_handler/paint_handler, `create` |
| 15 | UU | `wie-backend/Cargo.toml` | **재적용** | union | #12 재적용 · same | md5 | theirs: serde, zip workspace ^9, smaf 0.1.4, wie-core-arm-aot |
| 16 | UU | `wie-backend/src/canvas.rs` | **재적용** | 3way-edit | #13 재적용 · same | our tests; code = theirs (encode_bmp(image), starts_with) | theirs: BmpDecoder indexed transparency, encode_bmp(image) |
| 17 | UU | `wie-backend/src/canvas/lbmp.rs` | **재적용** | ours | #14 폐기 → 재적용 · DIFFERS | our decoder kept: native Rgb332/Rgb565 buffers, mask != 0, lenient size | theirs: every type → ArgbPixel, mask on type 8/16, rejects size mismatch and mask > 1 (guest-observable) |
| 18 | UU | `wie-backend/src/executor.rs` | **재적용** | ours | #15 재적용 · 해소 differs (3way-edit → ours: S2 #603 already ported #1429/#1491 under our pacing API; both upstream tests are in ours) | BTreeMap order, tick_for, sleep_toward, halt/clear, wants_cpu/next_due | theirs: TaskWake, sleep(until) |
| 19 | UU | `wie-backend/src/lib.rs` | **재적용** | union | #16 재적용 · same | speed_clock/pacing exports, tick_for/take_pacing | theirs: storage_request, is_preparing, Options.aot, normalize_guest_path |
| 20 | UU | `wie-backend/src/system.rs` | **재적용** | ours | new (0551 §6 silent → conflict after S2) | sleep_from/sleep_toward/poll_until/poll; filesystem scope stays aid | theirs: SleepFuture::new(now + timeout); FilesystemOverlay::new(.., pid) |
| 21 | UU | `wie-backend/src/system/file_system.rs` | **발신** | union | #17 발신 · same | virtual_file() + tests | theirs: aid→pid rename, remove(), usage(), pub normalize_guest_path |
| 22 | UU | `wie-backend/src/task.rs` | **재적용** | ours | new (0551 §6 silent → conflict after S2) | toward/idle_toward | theirs: SleepFuture::new(until) |
| 23 | AA | `wie-backend/tests/system.rs` | **재적용** | ours | new | upstream's test verbatim + our poll-from-step test | theirs: sleep_waits_from_the_call_time_after_task_work only |
| 24 | UD | `wie-core-arm/data/binary_patches.toml` | **재적용** | ours | #18 재적용 · same | 3 hash-keyed entries (LGT 01f05f8231f4, KTF 974e0df9ab1e / f770b15f8876) + generic KTF hooks | deleted by 223782022 (#1415 ARM AOT); no equivalent table upstream |
| 25 | UU | `wie-core-arm/src/allocator/bucket.rs` | **발신** | 3way-edit | #19 발신 · same | address-based free (bucket resolved from the address), size-mismatch warn, usage() report, test — all kept (auto-merged); the one hunk (single-byte header read) is converged → upstream spelling | upstream: free reads one header byte (same as ours), alloc scans the header in 256B chunks + maps the slot page (lazy commit), tests on ArmCore::new(Options) |
| 26 | UD | `wie-core-arm/src/binary_patches.rs` | **재적용** | ours | #20 재적용 · same (tests ported to ArmCore::new(Options)) | generic + title entries stacked in one install, install_title_binary_patches, tests | deleted by 223782022; upstream lib.rs has no binary_patches export |
| 27 | UD | `wie-core-arm/src/binary_patches/hook.rs` | **재적용** | ours | new as a conflict (0551 had it as a silent staged delete — we changed it since: +567 lines, 0565/0566 generic hooks + shadow) · 재적용 | hook kinds, decline/unhook, shadow mode; 2 code-write sites moved to ArmCore::write_code, tests ported to ArmCore::new(Options) | deleted by 223782022 |
| 28 | UU | `wie-core-arm/src/core.rs` | **재적용** | 3way-edit | #21 재적용 · same (0551 «keep both or fold» → decided: ours only, upstream stack_words dropped) | guest_root_words incl. contexts set aside by nested calls (saved_contexts), run_function restoring the caller on Ok AND Err, hold_others/preempted, teardown hooks, live_cores, SVC stub high-water, shared stubs, thread-id tracking, tests; new in this merge: write_code (AOT is told when the host rewrites guest code) | upstream core.rs: new(Options), prepare_execution/is_preparing, run_function_sync, record_image, AOT; NOT taken: stack_words() and run_function pushing the suspended ArmCoreContext onto the guest stack (#1545), fault registers left after Err (#1534) |
| 29 | UU | `wie-core-arm/src/engine/arm32_cpu.rs` | **발신** | 3way-edit | #22 발신 · same | guest null-page READ (<0x1000) reads 0, logged once; undefined-insn message with insn bytes + lr; branch-to-null lr log; tests; per-instruction loop reads nothing but pc when no AOT backend (0559); new in this merge: code_written (invalidate/drop AOT on host code writes), null-page warn on an AOT GuestFault | upstream arm32_cpu.rs: AOT dispatch in run(), MemoryPage directory, CP15 icache invalidation, MemoryAccess; its message «Undefined instruction at {pc} (CPSR)» not taken. ★null-read rule does NOT hold inside AOT-compiled code (GuestFault is terminal — wie-core-arm-aot/src/lib.rs:186) |
| 30 | UU | `wie-core-arm/src/lib.rs` | **재적용** | 3way-edit | #23 재적용 · same | exports binary_patches::{install_binary_patches, install_title_binary_patches}, pub mod stdlib, SVC_STUB_CAPACITY/live_cores/svc_stub_high_water; re-added `mod binary_patches;` + `pub mod stdlib;` that the auto-merge dropped OUTSIDE the hunk | upstream lib.rs: +mod aot, no binary_patches/stdlib (223782022 #1415) |
| 31 | UD | `wie-core-arm/src/stdlib.rs` | **재적용** | ours | #24 재적용 · same | memcpy/memset/memmove/strcpy/strlen used by binary_patches/hook.rs (and shadow.rs) | deleted upstream; its LGT copies live in wie-lgt/src/runtime/stdlib.rs |
| 32 | UU | `wie-j2me/Cargo.toml` | **폐기** | theirs | #25 폐기 · same | test-utils dev-dep | theirs: same + zip dev-dep |
| 33 | UU | `wie-j2me/src/emulator.rs` | **재적용** | 3way-edit | new (0551 §6 silent-merge list)  | 6 standard MIDP system properties (JSR-118), Canvas.standardKeyCodes=true, JvmSupport::finish_launch, Drop→system.teardown, pacing tick_for/take_pacing; storage id (jar file name / MIDlet-Name) kept | upstream: descriptor read from the manifest up front, is_preparing, microedition.platform only, wie_midp_vendor protos, storage id = MIDlet-1 title (changes the save key — NOT taken) |
| 34 | UU | `wie-jvm-support/src/jvm_implementation.rs` | **재적용** | 3way-edit | #26 재적용 · same | InheritedMethods, Preemptible, ForcedBool | theirs: external_roots() default |
| 35 | UU | `wie-jvm-support/src/lib.rs` | **재적용** | 3way-edit | #27 재적용 · same | fallible trace, SeverOnDrop, finish_launch | theirs: external_roots → Jvm::new, push_native_frame in to_wie_err |
| 36 | UU | `wie-ktf/src/adf.rs` | **재적용** | union | #30 재적용/union → 같음 | is_relocation_prefixed (relocation-table-prefixed client.bin → runtime::relocated) + test | #1497 25dd41efc validate_client_bin (12-byte stub+header check). Kept both; added validate_client_bin_entry(filename, data) so a relocation-prefixed image passes; +1 test |
| 37 | UU | `wie-ktf/src/emulator.rs` | **재적용** | 3way-edit | #31 재적용/3way-edit → 같음 | per-title owner-save drop lists (jar MD5), sole-.jar fallback, private dir mount P/ p/, owner sound, tick_for/take_pacing + reg dump; register_static_classes call (silently dropped by auto-merge → restored) | #1497 validate_archive/validate_jar (now use our jar_filename fallback + P/ lookup; load() takes only the header verdict), #1415 ArmCore::new(Options)/is_preparing, #1597 KtfAdf.slvl → wie.slvl property, #1534 tick without reg dump (ours kept) |
| 38 | UU | `wie-ktf/src/runtime/init.rs` | **재적용** | 3way-edit | #32 재적용/3way-edit → 같음 | JavaThrowInstance SVC (InitParam4.unk1), native frames + stress_collect, guest_roots::add_region, relocated::load + MNInterface, sdk_runtime flag, init_param_4() + test, install_binary_patches | #1415 core.prepare_execution() (kept, placed after install_binary_patches); #1542/#1545 VisitGcRoot stub + ptr_visit_gc_root write (not taken — one GC-root mechanism = ours) |
| 39 | UU | `wie-ktf/src/runtime/java/interface.rs` | **재적용** | 3way-edit | #33 재적용/3way-edit → 같음 | call_native J/D high word + thread-context result tag, java_throw_instance, null-jump diagnostics, per-call native frame, nativeStrings keep-alive | #1534 4011a8566 jump/call_native results returned through the SVC holder + map_exception_unwind(caller ctx) (taken); #1542 conditional frames on ptr_visit_gc_root (not taken), unconditional frames in java_class_load/register_class (taken); nativeStrings Vector dropped upstream (ours kept) |
| 40 | UU | `wie-ktf/src/runtime/java/jvm_support.rs` | **재적용** | 3way-edit | #34 재적용/3way-edit → 같음 | relocated ABI, class-record region allocator, exception scopes/handler offset, instantiation_error/wie_error, register_static_classes, guest_roots::install, 11 tests of ours | non-test delta of upstream is small: init(.., properties) (taken), KtfJvmSupportContext.ptr_visit_gc_root + removal of register_static_classes (not taken). Tests: upstream's test_native_virtual_overrides + additions to shared tests taken (3 adapted to our core); test_native_gc_visitor_keeps_guest_references not carried (asserts the visitor); test_class_loader_before_native_initialization not carried (= head of our register_static test) |
| 41 | UU | `wie-ktf/src/runtime/java/jvm_support/class_definition.rs` | **재적용** | union | #35 재적용/union → 같음 | class record from alloc_class_record; instantiate error via instantiation_error | method()/field() walk raw descriptors via JavaFullName::matches; prepare() → JavaVtable::resolve_overrides (#1450 e24e8ad44) — both arrived outside the hunks; hunks = import lists (union) |
| 42 | UU | `wie-ktf/src/runtime/java/jvm_support/class_instance.rs` | **재적용** | union | new (0584 §8 named it: both sides moved the JavaType import) | guest_roots use (ours) + S1b JavaType import move | #1429 d63d6022d get_field reads the raw field (reference fast path) — arrived outside the hunk; adapted to our Option-returning object_from_raw. Hunk = import line (theirs ⊇ ours). +upstream test field_reads_observe_guest_types_offsets_and_values |
| 43 | UU | `wie-ktf/src/runtime/java/jvm_support/classes/net/wie/ktf_class_loader.rs` | **재적용** | 3way-edit | new | outOfMemoryError reserve, nativeStrings Vector, loadClass = parent-first except names the jar ships as bytecode (jar_classes) | #1584 6200d7e93 loadClass = native-first for every non-array class (not taken — ours kept); #1542 dropped nativeStrings field + init (field proto was removed OUTSIDE the hunk → restored) |
| 44 | UU | `wie-ktf/src/runtime/java/jvm_support/jvm_implementation.rs` | **재적용** | ours | #36 재적용/3way-edit → 판정 같음 · 해소 다름(ours 통째) | on_teardown empties java_functions; define_class/define_array_class return a guest exception instead of panicking | #1542/#1545 ExternalRoots impl (run_function_sync visitor + ArmCore::stack_words, which core-arm removed) and external_roots() override — not taken; KTF keeps the default None |
| 45 | UU | `wie-ktf/src/runtime/java/jvm_support/method.rs` | **재적용** | 3way-edit | #37 재적용/3way-edit → 같음 | relocated enter/leave, handler chain walk (4096 cap) + e slot + target pc rewrite, relocated name-table class match, per-call frame | #1534 run_with_unwind restructure + with_arm_context (taken, equivalent under our run_function); #1542 innermost-only handler with empty-table early return (not taken; its «header-only record» fact folded into our walk: nothing past the header is read until the method has a catch); conditional arg frame (not taken) |
| 46 | UU | `wie-ktf/src/runtime/svc_ids.rs` | **재적용** | ours | #38 재적용/3way-edit → 판정 같음 · 해소 ours(두 덩어리) | InitSvcId::JavaThrowInstance = 7, UserMem table id, ktf_kernel_extension_message + tests | #1545 InitSvcId::VisitGcRoot = 7 (host-internal id; dropped, ours keeps 7). #1520 renames arrived outside the hunks and are kept: WIPICTableId Interface4→Database(5), Database→Fs(7), WIPICDatabaseMethodId→WIPICFsMethodId |
| 47 | UU | `wie-ktf/src/runtime/wipi_c.rs` | **재적용** | union | #39 재적용/3way-edit → 판정 같음 · 해소 union | local_net, ResourceCache, wide graphics ctx flag, Reserved4 → extension interface, ScreenFramebufferSync | #1542 `let _frame = jvm.push_native_frame(&[])` per WIPIC SVC (taken) + 2 upstream tests (ported to our 4-arg register_wipic_svc_handler) |
| 48 | UU | `wie-ktf/src/runtime/wipi_c/context.rs` | **재적용** | 3way-edit | #40 재적용/3way-edit → 같음 | ResourceCache-backed get_resource_size/read_resource, wide graphics ctx, set_timer(due, pace_from, cb), free(NULL) no-op, tests (interface4 slot 0, fs slot 11, throw_instance) | #1572 alloc zero-fill, #1534 call_function with_arm_context, #1597 java_property (all taken, outside hunks); resource read via class loader + GC (not taken); tests: indirect_allocations_clear_reused_payloads taken, timer_fault_diagnostics… kept for the undefined-instruction half only |
| 49 | UU | `wie-ktf/src/runtime/wipi_c/method_table.rs` | **재적용** | 3way-edit | #41 재적용/3way-edit → 같음 | MXUserMem table, KTF net slots on local_net, record DB on table 5 slots 0..=10, file slots on table 7, unnamed-table/kernel-extension stubs + tests | #1520 Fs/Database rename + fs::* / new database::* wiring (names taken, bodies NOT: every slot still runs our api::database function; get_database_method_table removed, table 5 stays 64 slots); #1517 im table (Interface3 + graphics 37..41), #1514 unk12[1] screen flush, #1596 uic slot renumbering, kernel execute/get_exec_names/get_access_level, util::htonl, fill_polygon stub (all taken — ours had only stubs there); net::socket* (not taken) |
| 50 | UU | `wie-lgt/data/lgt_java_abi.toml` | **발신** | union | #42 발신/union — same | ours 105 vtable+field rows with derivation comments, host_field tables (Thread, TimerTask) | upstream 110 rows (#1607 #1624 …): 91 identical to ours, 16 upstream-only added (InputStream 16-18, OutputStream 10, DataInputStream 21/26/29/30/31, StringBuffer 19/20/21/24/25/26, Thread.run 11); 3 index disagreements → ours kept (Class.nameBytes/classLoader word 3/1 vs 1/3; Runnable.run 0 vs 10); upstream scalar instance_field_word_count (Thread 4, TimerTask 6) not taken — ours models those with host_field |
| 51 | UU | `wie-lgt/src/emulator.rs` | **재적용** | 3way-edit | #43 재적용/3way-edit — same | ORPHAN_ENTRIES + ledger test, owner files/sound, tick_for/take_pacing with register dump, jar-name P/ fix | upstream: load(app_info) + SLvl → `wie.slvl` property, ArmCore::new(Options), is_preparing — taken. Upstream LgtTaskRunner::run (task-owned exception state #1532) had auto-merged OUTSIDE the hunks → put back to ours; its test exception_frames_are_restored_per_thread dropped (ours exception_frames_are_per_thread covers the property) |
| 52 | UU | `wie-lgt/src/runtime/init.rs` | **재적용** | ours | new | tests module (entrypoint_follows_raptor_header, compiler_array_helpers_build_guest_arrays), choose_entrypoint/.raptor, install_title_binary_patches, guest-root regions | upstream deleted the tests module (#1441 policy) — conflict hunk; its non-conflicting adds (prepare_execution after load_executable, with_arm_context on the two run_function calls) are in |
| 53 | UU | `wie-lgt/src/runtime/java/classes/net/wie/clet_wrapper_card.rs` | **폐기** | theirs | new | owner-arg get/put_field only (S1) | 228e3ff48 #1444 same owner args; 4011a8566 #1534 CletWrapperContext::call_function |
| 54 | UU | `wie-lgt/src/runtime/java/classes/net/wie/lgt_class_loader.rs` | **재적용** | 3way-edit | #44 재적용/3way-edit — same | outOfMemoryError static, find_generated_class, unreadable exception ptr → WieError not panic | upstream with_arm_context on the generic error arm — taken (identity here, see notes) |
| 55 | UU | `wie-lgt/src/runtime/java/exception.rs` | **재적용** | ours | #45 재적용/3way-edit — 해소 differs (ours whole: decision ⑷) | per-thread exception-frame ledger, consumed-sp convention, mark/release_to, host-error state, root_words, 12 tests | 031cb8f8d #1532 task-owned JavaSupportContext (init returns ptr, destroy) — not taken; only the shared SUPPORT_CONTEXT_BASE const from upstream globals.rs is used |
| 56 | UU | `wie-lgt/src/runtime/java/interface.rs` | **재적용** | ours | #46 재적용/3way-edit — 해소 differs (ours whole) | imports 0x26/0x64, long-array handlers, wide-field links, array class token = java/lang/Class object, per-call native frame, GC tests | 7f788d802 #1606 array types as class-name pointers (get_*_array_class, instantiate_array/multi_array args), 0x0a→0x64 GetInterfaceDispatchTable, restore caller context before unwind, jvm.instantiate_class — all not taken (ABI choices differ, see divergence table) |
| 57 | UU | `wie-lgt/src/runtime/java/jvm_support.rs` | **재적용** | 3way-edit | #47 재적용/3way-edit — same | ABI vtable linking, reserved OOM, generated class walk, guest task/thread layout, guest_roots install, tests module (init_jvm helper) | upstream: init(.., properties) — taken; virtual_method_index → i16 and interface_dispatch_table(core, jvm, ptr_instance, name) — not taken (go with upstream's class_definition rewrite); upstream deleted the tests module |
| 58 | UU | `wie-lgt/src/runtime/java/jvm_support/class_definition.rs` | **재적용** | ours | #48 재적용/3way-edit — 해소 differs (ours whole) | try_name, core accessor, host fields, vtable placement with reserved slots, tests | 3f7fdb572 #1580 / 01d4425c2 #1607: runtime_field_word_count, negative-tag runtime fields, method_hierarchy/vtable_entries, per-class interface tables — not taken |
| 59 | UU | `wie-lgt/src/runtime/java/jvm_support/class_instance.rs` | **재적용** | ours | new | wide (J/D) field word order high-first for both AOT and Rust-declared fields (report 0244) | upstream binds runtime-only field tags to the receiver's tail (negative index) — goes with its class_definition rewrite, not taken |
| 60 | UU | `wie-lgt/src/runtime/java/jvm_support/field.rs` | **재적용** | ours | #49 재적용/3way-edit — 해소 differs (ours whole) | JavaHostField (host-side storage keyed by core id + instance), forget_core/forget_instance, JavaValue::default_of | upstream Cow name/descriptor (already equivalent on 0.5.1 in ours), negative word index for runtime fields — not taken |
| 61 | UU | `wie-lgt/src/runtime/java/jvm_support/method.rs` | **재적용** | ours | #50 재적용/3way-edit — 해소 differs (ours whole) | per-method native frame, exception mark/release_to around guest calls, bad exception ptr report, tests | 9fdd57701 #1575 / 7f788d802 #1606: raw(), thumb-stub tests, array_type_passes_array_creation_and_cast_check — tied to upstream's exception + array-type design, not taken |
| 62 | UU | `wie-lgt/src/runtime/java/jvm_support/vtable.rs` | **재적용** | ours | new | target index built once (boot-time cost fix, report 0275), shared missing-entry stubs, inherit parent's appended slots, reserve unplaced slots | upstream: child's fixed slots placed first, inherited runtime-only methods appended again — different placement rule, not taken |
| 63 | UU | `wie-lgt/src/runtime/stdlib.rs` | **재적용** | 3way-edit | #51 폐기/theirs — DIFFERS: 재적용 | 8 slots (vsprintf strtol strncat strncmp strchr memmove malloc free) with call-site derivations, malloc NULL-on-failure + 4-byte header, localtime jiff test, svc-table tests | upstream covers the 8 slots by name but differs in behaviour (malloc Err + 8-aligned 12-byte header, strncmp byte difference, strtol clamp) → ours bodies kept; upstream-only slots taken: strtok 0x413, memcmp 0x416, memchr 0x417, atexit 0x424 (replaces base `unk3`), abort 0x425, realloc 0x427 (ported onto our header), unk450 0x450; sprintf/vsprintf use upstream's lazy Result-iterator call form |
| 64 | UU | `wie-lgt/src/runtime/svc_ids.rs` | **재적용** | 3way-edit | #52 재적용/3way-edit — same | Java ids 35..38, WIPIC ids named from call sites (0x68 terminate, 0xee, 0xf0, Im*, Uic*, byte order incl. 0x386, 0x19a/0x19c/0x1a0, net, 0x44c, 0x7d0), 6 table tests | upstream renamed/re-meant 0x68 0x6a 0xeb 0xf0 0x190..0x1a0 0x320..0x323 0x338 0x44c 0x4b6 0x4c0 0x4ce → ours kept; 21 ids only upstream maps added (0x67 0x6e 0xed 0x2bc/2bd/2c0/2c1 0x325 0x328 0x330 0x331 0x334 0x33f 0x342 0x343 0x44d..0x451 0x4bf); 0x19f FsTell left out; Java 35/36/37 numbering is host-internal (import index → id map lives in interface.rs) so ours' numbers stay |
| 65 | UU | `wie-lgt/src/runtime/wipi_c.rs` | **재적용** | 3way-edit | #53 재적용/3way-edit — same | shared-graphics wiring (slice D ⒝), terminate/Im*/Uic*/byte-order/list_directory/list_data_stores handlers, softkey rows, flush_lcd latch, RGB565+mask images, 4MiB memory answer, 15 tests | upstream routes everything to wie_wipi_c::api (fs/im/uic/term_res/util) and wires its own LGT graphics — taken only for the 21 ids ours did not map; `_frame = jvm.push_native_frame(&[])` in handle_wipic_svc + test resource_calls_release_java_temporaries taken |
| 66 | UU | `wie-lgt/src/runtime/wipi_c/context.rs` | **재적용** | 3way-edit | #54 재적용/3way-edit — same | timer-driven collect_garbage_if_due + GcClock, alloc/free_indirect with free(NULL) guard, set_timer(due, pace_from, cb), 3 tests | upstream java_property (new trait method), with_arm_context on call_function, collect_garbage error → FatalError — taken; test callback_fault_diagnostics_survive_java_exception_restoration dropped (asserts fault registers after a failed run_function — core-arm keeps ours, which restores the caller's) |
| 67 | UU | `wie-lgt/src/runtime/wipi_c/graphics.rs` | **재적용** | ours | #55 재적용/3way-edit — 해소 differs (ours whole) | screen_present_row latch, init/set_shared_context (native fg/bg/alpha overlay), native pixel-op blits, get_framebuffer_bpp, display-property svc test; the rest of the module is unwired (slice D ⒝) | upstream rewrote the LGT-native graphics (LgtGraphicsState 4 fields, get_context, destroy_image, application_y, create_wipi_image/image_buffer) — not taken: the module is not wired here and the rewrite needs wie-wipi-c's upstream image.rs API, which that slice kept ours |
| 68 | AA | `wie-midp/src/classes/javax/microedition/io.rs` | **폐기** | theirs | new | mod connector + re-export only | upstream io.rs: same plus Connection, ConnectionNotFoundException (c40c9c53e #1599) |
| 69 | AA | `wie-midp/src/classes/javax/microedition/io/connector.rs` | **발신** | 3way-edit | new | Connector.open x3 throwing IOException("no network") + test open_throws_io_exception | upstream #1599: same 3 open overloads throwing ConnectionNotFoundException + openInputStream/openDataInputStream/openOutputStream/openDataOutputStream; took theirs surface, kept our exception for the 3 opens |
| 70 | UU | `wie-midp/src/classes/javax/microedition/lcdui/canvas.rs` | **발신** | 3way-edit | 0551 #56 발신 · 3way-edit — same | getKeyCode GAME_A..D=7,9,*,#; getGameAction w/o number keys + standard key codes (J2ME -1..-11); getKeyName; game_key_bit (moved here from game_canvas.rs); repaint keeps non-positive sizes; 4 tests | upstream #1569/#1599/#1601: getKeyCode/getGameAction 1,3,7,9; pointer/hasRepeat stubs (taken, same values); handleKeyEvent PROTECTED (taken); held/pressedGameKeys tracking in Displayable (taken, with our key table) |
| 71 | UU | `wie-midp/src/classes/javax/microedition/lcdui/display.rs` | **재적용** | 3way-edit | 0551 #57 재적용 · 3way-edit — same | paint serialization, dirty-rect clip union, pacing hooks, screen compositor, GC interval/cost spacing, show/hideNotify, 13 tests — file rebuilt from ours | upstream b5320a916 #1582 static screenImage/screenGraphics + <clinit> (taken); c40c9c53e #1599 9 MIDP2 stubs (taken); is_current helper (taken, used by our new hideNotify re-entrancy guard); 4c3cdcfbd #1581 SKVM softkey 16 (NOT taken); repaintLeft../painting flag/paint_current (NOT taken) |
| 72 | UU | `wie-midp/src/classes/javax/microedition/lcdui/displayable.rs` | **폐기** | theirs | new | owner-arg field access only (S1) | upstream displayable.rs: same owner args + held/pressedGameKeys fields and resets (#1601); result is byte-identical to upstream |
| 73 | UU | `wie-midp/src/classes/javax/microedition/lcdui/font.rs` | **발신** | 3way-edit | 0551 #58 발신 · 3way-edit — same | getFace/getStyle/getSize/isPlain/isBold/isItalic/isUnderlined backed by face/style/size fields; getBaselinePosition = height*4/5; 1 test | upstream #1569/#1599: same methods as constant stubs, baseline 10, + getFont(I) (taken) |
| 74 | UU | `wie-midp/src/classes/javax/microedition/lcdui/game.rs` | **발신** | union | new | game package test sprite_tiled_layer_and_layer_manager_paint | upstream: module list only (c79ccccce #1601); its sprite.rs test is carried here as mod upstream_test and passes on our implementation |
| 75 | UU | `wie-midp/src/classes/javax/microedition/lcdui/game/game_canvas.rs` | **발신** | 3way-edit | new | flushGraphics()/(IIII) request a repaint only; (our keysHeld/keysLatched + handleKeyEvent override dropped for upstream structure) | upstream #1601/#1581: getKeyStates from Displayable.held/pressedGameKeys, 0 while not shown; full-screen buffer; flush + serviceRepaints (NOT taken) |
| 76 | AA | `wie-midp/src/classes/javax/microedition/lcdui/game/layer.rs` | **발신** | ours | new | Layer (<init>(II)V, package-private fields) | upstream #1601: same public methods, <init>()V, private fields — internal coupling with Sprite/TiledLayer, so the four files stay one set |
| 77 | AA | `wie-midp/src/classes/javax/microedition/lcdui/game/layer_manager.rs` | **발신** | ours | new | LayerManager over a Layer[] field | upstream #1601: same 8 methods over java.util.Vector |
| 78 | AA | `wie-midp/src/classes/javax/microedition/lcdui/game/sprite.rs` | **발신** | ours | new | Sprite: 21 methods, collision, transforms + 2 tests | upstream #1601: same 21 methods, own field names, uses Image::transform_source; its test passes on ours |
| 79 | AA | `wie-midp/src/classes/javax/microedition/lcdui/game/tiled_layer.rs` | **발신** | ours | new | TiledLayer: 13 methods, animated tiles | upstream #1601: same 13 methods |
| 80 | UU | `wie-midp/src/classes/javax/microedition/lcdui/graphics.rs` | **발신** | 3way-edit | new | fillTriangle (real), copyArea, get*Component, getGrayScale (luma), set/getStrokeStyle (stored), drawRegion via draw_transformed/transformed_region + tests | upstream #1599: same 9 methods (fillTriangle/stroke stubs, gray = average), drawRegion via Image::transform_region; drawRGB scanlength + bounds (taken, ours==base) |
| 81 | UU | `wie-midp/src/classes/javax/microedition/lcdui/image.rs` | **발신** | 3way-edit | new | new Image() = 1x1; createImage(InputStream)/(Image,IIIII); JavaImageBuffer write guard; image_buffer(); 3 tests | upstream #1599: same 5 methods; getRGB/createRGBImage validation (taken); white fill for createImage(II), getGraphics ISE on immutable, IOException on undecodable name/stream (NOT taken); transform_region/transform_source (dropped, unused here) |
| 82 | AA | `wie-midp/src/classes/javax/microedition/lcdui/list.rs` | **발신** | ours | new | full List (Form-backed Choice: 22 methods, listType) | upstream #1569: two constructor stubs only |
| 83 | AA | `wie-midp/src/classes/javax/microedition/lcdui/text_field.rs` | **발신** | ours | new | TextField over StringItem: get/setString, constraints, maxSize, size | upstream #1569: one constructor stub |
| 84 | UU | `wie-midp/src/classes/javax/microedition/lcdui/ticker.rs` | **폐기** | theirs | new | owner-arg + NativeFrame guard in the test (S1) | upstream ticker.rs: same, reading the static Display.screenImage; result byte-identical to upstream |
| 85 | UU | `wie-midp/src/classes/javax/microedition/media.rs` | **발신** | 3way-edit | new | ToneControl interface | upstream layout: pub mod control (control/volume_control.rs) + control_interface.rs; our ToneControl moved to control/tone_control.rs, control_interfaces.rs removed |
| 86 | UU | `wie-midp/src/classes/javax/microedition/media/manager.rs` | **발신** | 3way-edit | new | playTone (real), createPlayer("device://tone"), toneHandle field | upstream #1599: playTone stub, createPlayer(String) always MediaException, + getSupportedProtocols/getSupportedContentTypes (taken) + test |
| 87 | UU | `wie-midp/src/classes/javax/microedition/midlet/midlet.rs` | **발신** | 3way-edit | 0551 #59 발신 · 3way-edit — same 판정; differs from its note: the wieDisplay field name is kept (origin/main still has it after S1/S1b) | notifyDestroyed -> exit_from_guest; wieDisplay field; 2 tests | upstream: owner-arg display field; checkPermission/platformRequest/notifyPaused/resumeRequest + MIDletStateChangeException (auto-merged) |
| 88 | UU | `wie-midp/src/classes/javax/microedition/rms.rs` | **폐기** | theirs | new | exports incl. RecordEnumerationImpl | upstream rms.rs exports the same set without the Impl (it lives at net/wie/RecordEnumeration) |
| 89 | AA | `wie-midp/src/classes/javax/microedition/rms/record_comparator.rs` | **폐기** | theirs | new | interface with compare only | upstream: same + PRECEDES/EQUIVALENT/FOLLOWS static fields |
| 90 | AA | `wie-midp/src/classes/javax/microedition/rms/record_enumeration.rs` | **폐기** | theirs | new | interface + net.wie.RecordEnumerationImpl (snapshot ids, keepUpdated/rebuild/destroy stubs) | upstream: identical interface; implementation is net/wie/record_enumeration.rs (keepUpdated, rebuild, destroy). Our one behaviour it lacked — ascending id order without a comparator — is carried into that file (3 lines) |
| 91 | AA | `wie-midp/src/classes/javax/microedition/rms/record_filter.rs` | **폐기** | theirs | new | interface matches([B)Z | upstream: identical |
| 92 | UU | `wie-midp/src/classes/javax/microedition/rms/record_store.rs` | **발신** | 3way-edit | new | RecordStoreNotFoundException message = store name; empty record read as empty array; 2 tests (first-run open, enumeration walk) | upstream #1599: enumerateRecords -> net.wie.RecordEnumeration, repository.create, null data with length 0, getRecord null for empty (NOT taken), tests |
| 93 | UU | `wie-midp/src/classes/net/wie.rs` | **발신** | union | new | PlayerControl | upstream: RecordEnumeration |
| 94 | UU | `wie-midp/src/classes/net/wie/event_queue.rs` | **재적용** | 3way-edit | new | pacing/slice/wake-tick machinery, held timers, ~900 lines + tests | upstream: Event::Repaint -> RepaintEvent(1) and NotifyEvent dispatch (taken); RepaintEvent via Display.serviceRepaints (NOT taken — stays handlePaintEvent) |
| 95 | UU | `wie-midp/src/classes/net/wie/smaf_player.rs` | **발신** | ours | 0551 #60 발신 · 3way-edit — 판정 same, 해소 differs: upstream delta is owner-arg only, already ours since S1 | MIDP Player model (END_OF_MEDIA, repeat, generation, PlayerControl, tone locator) + 5 tests | upstream: owner-arg rewrite only (#1444) |
| 96 | UU | `wie-midp/src/lib.rs` | **발신** | union | new | get_protos: Reference/WeakReference, ToneControl, PlayerControl | upstream: Connection, ConnectionNotFoundException, net.wie.RecordEnumeration; merged array = 60 entries, no duplicate |
| 97 | UU | `wie-skt/Cargo.toml` | **재적용** | union | new | test-utils dev-dep | theirs: zip dev-dep |
| 98 | UU | `wie-skt/src/emulator.rs` | **재적용** | 3way-edit | #28 재적용/3way-edit — same | SYSTEM_PROPERTIES (m.MODEL wie, m.EXT_SW 0, m.MIN 01000000000), package display size + md5-keyed DISPLAY_OVERRIDES row, owner sound, archive_entries/archive_jar/title/id, pacing, teardown, ledger test | upstream: m.MIN 00000000, m.MODEL "0", from_jar reads the manifest (signature lost main_class_name), loadable_jar also accepts plain jars that reference com/skt/m or com/xce, is_preparing |
| 99 | UU | `wie-skvm/src/classes/com/skt/m/graphics_2d.rs` | **발신** | 3way-edit | #61 발신/3way-edit — same 판정; resolution moved (captureLCD now theirs) | DRAW_AND/DRAW_OR blending in Graphics2D.drawImage + tests | upstream now blends AND/OR too but inside XDisplay.drawImageEx (Graphics2D.drawImage forwards there — the reverse of ours); captureLCD reads static Display.screenImage (taken: wie-midp's Display went static) |
| 100 | AA | `wie-skvm/src/classes/com/xce/io/byte_to_char_converter.rs` | **폐기** | theirs | new | abstract class: <init>, abstract convert/flush | upstream superset: buf/blen fields, concrete flush, abstract convert + getNumberOfChars, stateful decode() that keeps a split EUC-KR lead byte |
| 101 | AA | `wie-skvm/src/classes/com/xce/io/byte_to_char_euc_kr.rs` | **폐기** | theirs | new | stateless EUC-KR convert with length arguments, flush=0, 1 test | upstream: same length-argument contract, bounds/null checks, pending-byte carry, getNumberOfChars; our test passes on it unchanged in meaning |
| 102 | UU | `wie-skvm/src/classes/com/xce/io/x_file.rs` | **발신** | 3way-edit | #62 발신/3way-edit — same | XFile(jar,name) entry selection via filesystem().virtual_file + zip_entry, buf/offset exposure (open_jar_entry), fsavail 1000000, 3 tests | upstream: XFile(jar,name) still IOException; READ_RESOURCE via system class loader (taken), fsused = filesystem usage (taken), fsavail = 1 MiB − usage (NOT taken) |
| 103 | AA | `wie-skvm/src/classes/com/xce/lcdui/text_component.rs` | **폐기** | theirs | new | interface with 13 abstract methods | upstream: the same 13 names and descriptors, same flags |
| 104 | AA | `wie-skvm/src/classes/com/xce/lcdui/text_component_handler.rs` | **발신** | ours | new | multitap input method over TextComponent (wie_util::keypad), isLoaded=false, getInputMode, keyRepeated, focused-XTextField fallback, 4 tests | upstream: abstract class + net/wie/WieTextComponentHandler instance; digits/CLEAR only, honours UNEDITABLE (0x20000); no isLoaded/getInputMode/keyRepeated — every upstream method name exists in ours |
| 105 | UU | `wie-skvm/src/classes/com/xce/lcdui/x_display.rs` | **발신** | 3way-edit | #63 발신/union — resolution differs (upstream implemented drawImageEx/refresh since) | XDisplay.clear stub, drawImageEx forwarding to Graphics2D.drawImage, copyLCD via captureLCD, 2 tests | upstream: drawImageEx does the blit itself and draws into the second image when given, refresh paints the screen image directly, copyLCD still a stub, IMG_COPY/AND/OR/XOR constants (taken) |
| 106 | UU | `wie-skvm/src/classes/com/xce/lcdui/x_text_field.rs` | **발신** | 3way-edit | #64 발신/3way-edit — same | numeric multitap, repaint on change, static focused field, take_key, 2 tests | upstream head is the owner-arg rewrite only (already ours since S1); its new test flow (TrackingComponent through the handler) merged into ours |
| 107 | UU | `wie-skvm/src/classes/net/wie/wie_audio_clip.rs` | **발신** | ours | new | SMAF clip on Audio::load_smaf: close does not stop, orphan loops, loop idempotent, play blocks only on a sound thread, pause/resume, 8 tests | upstream implemented the same stubs differently: play always blocks until media end, close while playing = IOException, non-SMAF = UnsupportedFormatException, UserStopException on stop |
| 108 | UU | `wie-skvm/src/lib.rs` | **발신** | union | new | protos 26→33: ByteToChar*, TextComponent*, m/A3·V3·XOWorld | upstream 26→31: ByteToChar*, TextComponent*, net/wie/WieTextComponentHandler — union = 34, no duplicate |
| 109 | UU | `wie-web/src/rust/lib.rs` | **재적용** | 3way-edit | #29 재적용 · same | FramePacer wiring (3 lines) | theirs: rewritten host (WasmHost, aot, is_preparing) |
| 110 | UU | `wie-wipi-c/src/api/database.rs` | **재적용** | ours | #65 재적용/3way-edit(UD) → 판정 같음 · 상태·해소 다름(지금은 UU: upstream 이 같은 경로에 MC_db* 레코드 API 를 새로 썼다 · ours 통째) | KTF/LGT DB 슬롯 전부(slot4 lseek · slot7 list/rename · slot8 mkdir · slot15 size · exists · 16MiB · LGT mode8 · Interface4 record DB · 0547 packaged p-file/mode-4 seed) + 시험 20 | upstream database.rs = MC_db* 재작성(open/close/delete/insert/select/update/delete_record/list_records/get_access_mode/get_number_of_records/get_record_size · 핸들 매직 0x44424844) · 파일 슬롯은 api/fs.rs 로 감 — 둘 다 우리 저장소(database_repository · 매직 0x4D434442)와 안 섞인다 |
| 111 | UU | `wie-wipi-c/src/api/graphics.rs` | **재적용** | 3way-edit | #66 재적용/3way-edit → 같음(본은 ours · upstream 조각만 얹음) | pixel op · color key · LGT alpha mask · wide KTF ctx · screen fb 양방향 sync · NULL 이미지 · context clip(x1,y1,x2,y2 포함 · 교집합) · encodeImage(영역 clamp) · draw_polygon · 누수 수정 | 받음: fill_polygon 스텁 · destroy_image 가 img/mask 버퍼 해제(널 가드 더함) · FONT_SIZE · globals::SCREEN_FRAMEBUFFER_PTR · encode_bmp(image) 서명 · 시험 2(baseline · mask bit). 안 받음: repaint→Event::Repaint · create_wipi_image RGB565+mask · draw_image 의 upstream clip 의미 · upstream 시험 3 |
| 112 | UU | `wie-wipi-c/src/api/graphics/primitives.rs` | **발신** | 3way-edit | #67 발신/3way-edit → 같음 | write_canvas 가 건드린 영역만 읽고 쓴다(+시험) | #1517 draw_text 기준선 y -= FONT_SIZE 를 우리 write_canvas 서명 위에 얹음(다른 곳에 기준선 보정 없음 확인) |
| 113 | UU | `wie-wipi-c/src/api/kernel.rs` | **재적용** | 3way-edit | #68 재적용/3way-edit → 같음(본은 ours) | 타이머 event_queue arm/cancel + pacing 몫 · -1 · DefTimer 가 대기 해제 · PHONENUMBER 비지 않음 · alloc 0 채움/alloc_or_null · alloc(0)=NULL · 메모리 2MiB · MEDIADEVICES/VOLUMELEVEL · 시험 19 | 받음: execute · get_exec_names · get_access_level 스텁 · sprintf 반복자 인자. 안 받음: #1536 게스트 메모리 PendingTimer 목록(-5) · alloc(0) 실핸들 · 메모리 0x10000000 · upstream 타이머 시험 1 |
| 114 | UU | `wie-wipi-c/src/api/kernel/sprintf.rs` | **발신** | 3way-edit | #69 발신/3way-edit → 같음 | 바이트 그대로 통과(EUC-KR 왕복 없음) · %s 정밀도/왼쪽 정렬 · 정수 정밀도/0 채움 · %i · 시험 4 | #1481 Result 반복자 인자 · long long(next_arg64) · `%.*s` 를 우리 포맷터에 얹음 · upstream 시험(EUC-KR) 통과 |
| 115 | UU | `wie-wipi-c/src/api/media.rs` | **발신** | 3way-edit | #70 발신/3way-edit → 같음(본은 ours) | clip/master 볼륨 → AudioSink gain · 클립 콜백 STARTED/END_OF_DATA/STOPPED + generation 가드 · 0 아닌 clip id · 시험 12 | upstream 볼륨 4종은 여전히 warn 스텁 · 새 스텁 get_default_volume · clip_clear_data 를 더함 |
| 116 | UU | `wie-wipi-c/src/api/net.rs` | **재적용** | ours | #71 재적용/3way-edit → 판정 같음 · 해소 ours | 가짜 서버(fake_net) socket/connect/read/write/콜백 · 시험 4 | upstream 은 -1 스텁 6개(socket · socket_connect · socket_write · socket_read · set_read_callback · set_write_callback)뿐 — 이름만 다르고(우리 set_read_cb/set_write_cb) 우리 것이 포함한다 |
| 117 | UU | `wie-wipi-c/src/api/util.rs` | **발신** | union | #72 폐기/theirs → ★다름: 발신/union | inet_addr 가 앞자리 0 옥텟을 십진으로 받는다(parse_dotted_quad) | upstream inet_addr = Ipv4Addr::from_str(앞자리 0 거부 → INADDR_NONE) · htonl · ntohs · 시험 1 — htonl/ntohs/시험은 받고 inet_addr 본문은 우리 것(시험에 앞자리 0 행 1개 더함) |
| 118 | UU | `wie-wipi-c/src/context.rs` | **재적용** | 3way-edit | #73 재적용/3way-edit → 같음 | WIPICContext::set_timer(due, pace_from, cb) · wide_graphics_context() · TestContext calls/call_args/spawned/timers · 범위 검사 메모리 | #1509 TestContext functions + global_data(0x7fff0000) 분리 · #1536 timers VecDeque — functions/global_data 는 합침(범위 밖은 오류), timers 는 우리 꼴(Vec<(due, pace_from, cb)>), function_calls 는 우리 call_args 와 같은 것이라 안 둠 |
| 119 | UU | `wie-wipi-java/src/classes/net/wie/card_canvas.rs` | **발신** | ours | #74 발신·3way-edit → 해소만 다름(ours) | paint: Display 의 repaint clip 을 카드마다 유지(#400) + 불투명 카드에 가려진 카드는 그리지 않음(#524) | upstream 이 그 뒤 자기 판을 넣었다(translate+setClip+카드 경계 clipRect · 모든 카드 그림) — 동작이 갈려 받지 않음(notes ⑴). owner 인자는 양쪽 수렴 |
| 120 | UU | `wie-wipi-java/src/classes/net/wie/wipi_midlet.rs` | **폐기** | ours | new | 우리 델타는 owner 인자뿐(전부 upstream) | upstream 추가분 = `destroyed` 필드 + `Jlet::destroy` 재진입 가드. jlet.rs 의 우리 notifyDestroyed(= destroyApp 을 부르지 않는다)와 짝이라 받지 않음 → 파일은 origin/main 과 바이트 동일 |
| 121 | AA | `wie-wipi-java/src/classes/org/kwis/msf/io/message.rs` | **발신** | 3way-edit | new | Message(String,[B)·getData + `addr` 필드(생성자 인자를 버리지 않는다) + 시험 | upstream 판은 `data` 만 든다(부분집합). 우리 본문 + upstream 시험 `message_retains_the_supplied_buffer` 추가 |
| 122 | UU | `wie-wipi-java/src/classes/org/kwis/msp/handset/handset_property.rs` | **발신** | 3way-edit | #75 발신·3way-edit · 같음 | VOLUMELEVEL "3" + 시험 3개 | upstream #1507 VOLUMELEVEL "100"(갈림 · 우리 유지) · PHONENUMBER "01000000000" 은 그 뒤 upstream 에도 들어옴(수렴) · upstream 은 시험 모듈 삭제(#1441) |
| 123 | AA | `wie-wipi-java/src/classes/org/kwis/msp/handset/led.rs` | **폐기** | theirs | #76 폐기·theirs · 같음 | LED.set(I)V 스텁 + 호출 가능 시험 | upstream led.rs 동일 proto · 같은 no-op(#1519). 우리 시험 1개는 버림(0551 대로) |
| 124 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/card.rs` | **발신** | ours | #77 발신·3way-edit → 해소만 다름(ours) | Card()/Card(Z) 가 y=AnnunciatorComponent::shown_height 에서 시작(#464) · bound/set_bound(= owner 인자와 같은 동작) · 시험(ShadowingCard · 불투명 덮기 · clip) | upstream 본문 델타는 owner 인자(수렴)뿐이고 나머지는 card_canvas 의 새 clip 동작에 맞춘 시험 수정 — card_canvas 를 우리 것으로 뒀으므로 받지 않음 |
| 125 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/display.rs` | **발신** | ours | new | callSerially(Runnable,I) 실구현(별 태스크에서 기다린 뒤 무시간 형으로 넘김 · GlobalRef) · card_canvas() 접근자 · 시험 | upstream 도 같은 메서드를 구현(Event::timer 로 시스템 이벤트 큐에) — 방식이 갈려 우리 본문 유지. upstream 시험 `delayed_callbacks_follow_deadlines_and_survive_gc` 는 upstream 방식(시계를 `set` 으로 옮기고 입력 이벤트와의 순서를 단언)에 묶여 있어 받지 않음 — 같은 것을 우리 `timed_call_serially_queues_the_runnable_after_the_timeout` 이 잰다. 파일은 origin/main 과 바이트 동일 |
| 126 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/graphics.rs` | **발신** | 3way-edit | #78 발신·3way-edit · 같음 | setRGBPixels 의 processAlpha=false(#455 — upstream 도 그 뒤 false 로 수렴) · copyArea 자체 구현 · setFont(null) · 시험 | upstream 신규 getRGBPixels 실구현을 받음(우리는 스텁이었다) — 단 bpl 단위는 우리 setRGBPixels 와 맞춤(int 개수 · upstream 은 bpl/4). copyArea 의 MIDP 위임 · setRGBPixels 의 bpl/4 는 받지 않음(notes ⑴) |
| 127 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/image.rs` | **발신** | ours | new | `mutable` 필드로 isMutable(createImage(II) 만 true) · 그 밖 owner 인자 | upstream: createImage(Image) 가 흰 바탕에 눌러 그린 mutable 사본 · 불변 이미지의 getGraphics 는 null · isMutable 을 MIDP 에 위임 · drawImage 의 transform 을 WIPI 각도값에서 MIDP 값으로 사상 — 넷 다 게스트가 보는 동작이라 받지 않음(notes ⑴) |
| 128 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/input_method_handler.rs` | **발신** | ours | #79 발신·3way-edit → 해소만 다름(ours) | 입력기 전체: 모드 get/set/next · notifyKeyInput 멀티탭·한글 조합(wie_util::keypad) · 리스너 통지 | upstream 은 그 뒤 스텁에서 «모드 색인 4개 + allowedModes + 숫자 키면 "temp" 통지»로 감 — 우리 구현과 모드 체계가 달라 받지 않음. upstream 전용 멤버는 필드 2개(currentMode·allowedModes)뿐 |
| 129 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/input_method_listener.rs` | **폐기** | theirs | #80 폐기·theirs · 같음 | 빈 인터페이스 proto | upstream: 같은 이름 + notifyTextChanged([CII)V abstract(#1519) = 상위집합 |
| 130 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lcdui/jlet.rs` | **발신** | ours | #81 발신·3way-edit → 해소만 다름(ours) | notifyDestroyed 가 destroyApp 을 부르지 않음(#350) · getCurrentJlet · dis owner 인자 · 시험 3개 | upstream 도 그 뒤 getCurrentJlet 을 넣었고(수렴) 재귀는 `destroyed` 가드로 막되 destroyApp 을 1회 부른다(갈림 · 우리 유지). 자동 병합은 getCurrentJlet 을 두 번 등록했다 → origin/main 판으로 둠(시험 모듈의 안 쓰는 import 2줄만 지움 — origin/main 에 원래 있던 `--all-targets` 경고). upstream 시험 `destroy_app_runs_once_when_it_notifies_destroyed` 는 그 갈린 동작을 단언해 받지 않음 |
| 131 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc.rs` | **발신** | union | #82 발신·union · 같음 | mod/재수출: ActionListener·ButtonComponent·ProgressComponent·KEY_NOTIFY 등 | upstream 의 새 `proxy_card::ProxyCard` 를 더함(우리 목록 + 1) |
| 132 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/component.rs` | **발신** | 3way-edit | #83 발신·3way-edit · 같음 | 상태 있는 Component(x/y/w/h/bg/fg · 포커스 · validate→layout · repaint · paint · setEventListener · 키 전달) + 시험 | upstream 은 같은 이름을 스텁으로 둠 → 우리 본문 유지. upstream 전용 서명 13개(getXOnScreen·getYOnScreen·canHandleInput·isValid·calcPreferredSize·getPreferredWidth·getPreferredHeight()·(I)·paintContent·invalidate·pointerNotify·processEvent·getCard)를 목록 «끝»에 스텁으로 더함. upstream 의 `focused` 인스턴스 필드는 받지 않음(lwc 인스턴스 필드는 LGT AOT 하위 클래스의 오프셋을 민다) |
| 133 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/container_component.rs` | **발신** | ours | new | 자식 목록(add/get/remove/removeAll/getNumberOfComponent) · paint 가 자식을 그림 · 시험 | upstream 델타 = paint 스텁 1개(PROTECTED) — 우리 paint 가 실구현이라 덩어리를 ours 로 풀면 origin/main 과 같다 |
| 134 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/dialog_component.rs` | **발신** | ours | new | type 필드 · 생성자가 cmp 를 첫 자식으로 · doModal = TYPE_NONE 이면 DLG_TIMEOUT 아니면 DLG_OK · 시험 2개 | upstream 판은 스텁: doModal 이 항상 12(DLG_CANCEL) · cmp 를 더하지 않음(갈림 · 우리 유지) |
| 135 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/form_component.rs` | **발신** | ours | #84 발신·3way-edit → 해소만 다름(ours) | `vertical` 필드 유지 · setFocus(Component) · 시험 | upstream 판은 생성자 둘뿐(부분집합 · flag 를 버린다) |
| 136 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/grab_key_listener.rs` | **폐기** | theirs | #85 폐기·theirs · 같음 | grabKeyNotify(IILjava/lang/Object;)Z 인터페이스 | upstream 바이트 동일 proto(#1532) — 주석만 다름 |
| 137 | AA | `wie-wipi-java/src/classes/org/kwis/msp/lwc/label_component.rs` | **폐기** | theirs | #86 폐기·theirs · 같음 | <init>()V · <init>(String)V 스텁 | upstream: 같은 둘 + (String,Image) + (String,String)(#1569) = 상위집합 |
| 138 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/shell_component.rs` | **발신** | ours | #87 발신·3way-edit → 해소만 다름(ours) | getWidth/getHeight · 제목 · keyNotify 를 포커스 잎으로 · paint · grabKey/ungrabKey · setWorkComponent · show/hide(net.wie.ShellCard) + 시험 | upstream 은 그 뒤 ProxyCard 위에 자기 show/hide/isShown/getCard/repaint/serviceRepaints 를 올렸다(`cd` 인스턴스 필드) — 같은 일을 다른 설계로. 우리 것 유지(`cd` 필드는 LGT AOT 오프셋 때문에 받지 않음). ShellComponent.getCard 는 Component 의 스텁(null)을 탄다 |
| 139 | UU | `wie-wipi-java/src/classes/org/kwis/msp/lwc/text_component.rs` | **발신** | ours | #88 발신·3way-edit → 해소만 다름(ours) | getMaxLength · keyNotify(입력기) · paint · 비밀번호 '*' · m_td/iMode/maxLength 필드 | upstream 도 m_td·maxLength·iMode 를 넣음(수렴) — 덩어리를 ours 로 풀어도 `iMode` 필드가 두 번 선언돼 남는다 → origin/main 판. upstream 전용 멤버 0 |
| 140 | UU | `wie-wipi-java/src/classes/org/kwis/msp/media/base_clip.rs` | **발신** | ours | #89 발신·3way-edit → 해소만 다름(ours) | setBuffer 가 게임의 배열을 붙들고 FNV 해시 · refresh() 가 고쳐 쓴 배열을 다시 읽음 · putData 가 등록 배열을 놓음(#359) | upstream #1490: `stream` 필드 + 첫 재생 때 플레이어 생성(Clip.getOrCreatePlayer). 0551 은 «그 위에 refresh 를 다시 표현»을 권했으나 플레이어 생성 시점이 바뀌는 재설계라 하지 않음 — 우리 판 유지(notes ⑴) |
| 141 | UU | `wie-wipi-java/src/classes/org/kwis/msp/media/clip.rs` | **발신** | ours | #90 발신·3way-edit → 해소만 다름(ours) | 기본 음량 100 · Clip(type,null) 은 NPE · setVolume 이 오디오 핸들에 적용(#356) · player(null)=None | upstream: 기본 음량 0 · null 검사 없음 · setVolume 은 필드만 · getOrCreatePlayer(내부용 · WIPI API 아님) — 받지 않음 |
| 142 | AA | `wie-wipi-java/src/classes/org/kwis/msp/media/media_unsupported_exception.rs` | **폐기** | theirs | #91 폐기·theirs · 같음 | 생성자 둘 | upstream 같은 둘 + PUBLIC 플래그(#1569) |
| 143 | UU | `wie-wipi-java/src/classes/org/kwis/msp/media/player.rs` | **발신** | ours | #92 발신·3way-edit → 해소만 다름(ours) | BaseClip 오버로드가 Clip 길로 · resume(Clip) 실구현 · stop 은 STARTED 였을 때만 true(#550) · stop(null) 은 조용히 false · play 가 버퍼 갱신 + 음량 적용 + 시험 12개 | upstream: stop(null) = NullPointerException(#1501) · 플레이어가 있으면 true · BaseClip 오버로드·resume 은 스텁 false — §8 의 두 자리 · 우리 값 유지 |
| 144 | AA | `wie-wipi-java/src/classes/wec.rs` | **재적용** | ours | new | mod dm_info/oem_device/sys_theme + 재수출 | upstream 판은 oem_device·sys_theme 둘(부분집합). `wec/dm_info.rs` 는 우리만 있다 |
| 145 | AA | `wie-wipi-java/src/classes/wec/oem_device.rs` | **재적용** | ours | new | getSYSTheme 이 null 이 아닌 SYSTheme 을 준다(호출부가 null 이면 NPE 길로 간다 — 코퍼스 디스어셈으로 잰 것) | upstream 판은 null 을 준다(갈림 · 우리 유지). 공개 WIPI 가 아닌 단말 제조사 확장이고 근거가 타이틀 역공학이라 발신이 아니라 재적용 |
| 146 | AA | `wie-wipi-java/src/classes/wec/sys_theme.rs` | **재적용** | ours | new | private <init> · 정적 둘은 JVM 기본값 0 · saveItem 은 0 · 시험 | upstream 판: <clinit> 이 ITEM_GROUP_SOUND_DEFAULT=1 · <init> 없음 · saveItem 은 -1(갈림 · 우리 유지 — OEMDevice 가 `new` 로 만든다). 같은 이유로 재적용 |
| 147 | UU | `wie-wipi-java/src/lib.rs` | **재적용** | 3way-edit | #93 재적용·3way-edit · 같음 | get_protos 42→67: 맨 위 preload 순서 블록 · LED · Button/Dialog/Progress · ShellCard · wec/* · com/ktf/kfc/* · 주석(시험이 잠금) | upstream 42→53. 우리 목록 + upstream 전용 ProxyCard 1개. 덩어리 밖 중복을 지움(자동 병합 6건 + origin/main 에 원래 있던 ImageObserver 1건) → 67개 · 중복 0 |
**계수**: 폐기 **20** + 재적용 **73** + 발신 **54** = **147** · 미분류 0.

### 부록 B. 크레이트별 노트(담당이 적은 원문)

병합은 크레이트 일곱 묶음으로 나눠 풀었다. 아래는 묶음마다의 노트 원문이다 — 결정의 근거 · 갈림 표 · 조용한 병합 확인 · 그 시점의 명령 출력. «내 몫 밖에서 바꿔야 하는 것» 절의 요청은 전부 이 병합에서 처리됐다(`DatabaseRepository::usage` · `SktEmulator::from_jar` 호출부 · `aot: None` · 스테이징).

#### 부록 B — `wie-core-arm` 묶음

##### wie-core-arm — S3 merge notes (fork s3-core-arm)

###### ⑴ Post-merge public API — what wie-ktf / wie-lgt / hosts code against

State: `cargo check -p wie-core-arm --all-targets` clean (this section written at that point; test/clippy results in ⑹).

`wie-core-arm/src/lib.rs` exports (ours + upstream, nothing of ours dropped):

```rust
pub mod stdlib;                                   // ours, kept (hook.rs uses memcpy/memset/strcpy/strlen)
pub type ThreadId = usize;
#[cfg(not(target_arch = "wasm32"))] pub use binary_patches::hook_shadow;
pub use {
    allocator::Allocator,
    binary_patches::{install_binary_patches, install_title_binary_patches},
    context::ArmCoreContext,
    core::{ArmCore, RUN_FUNCTION_LR, RunFunctionResult, SVC_STUB_CAPACITY, live_cores, svc_stub_high_water},
    function::{EmulatedFunction, EmulatedFunctionParam, JumpTo, RegisteredFunction, RegisteredFunctionHolder, ResultWriter, SvcId},
};
```

| item | post-merge | note |
|---|---|---|
| `ArmCore::new` | **`ArmCore::new(options: wie_backend::Options) -> Result<Self>`** (upstream) | old `ArmCore::new(false, None)` → `ArmCore::new(wie_backend::Options { enable_gdbserver: false, aot: None, profile: None })`. `enable_gdbserver: true` ignores `aot` (debug engine is interpreter-only) |
| `ArmCore::load(data, address, map_size)` | same signature | upstream: also records the range as an AOT image (`record_image`) |
| `ArmCore::prepare_execution(&mut self).await -> Result<()>` | new (upstream) | snapshots every `load`ed range and compiles it. No-op with `aot: None`. ★Call it AFTER `install_*binary_patches` and after the last `load` |
| `ArmCore::is_preparing(&self) -> bool` | new (upstream) | `false` with `aot: None`. ★With `aot: Some`, it is `true` from construction until `prepare_execution` has finished — a load path that never calls `prepare_execution` (e.g. KTF `relocated::load`, which returns before the call in `runtime/init.rs`) leaves the emulator «preparing» forever. Every load path must end in `prepare_execution` |
| `ArmCore::run_function::<R>(address, params).await -> Result<R>` | **OURS** | caller context is put aside in `saved_contexts` (visible to `guest_root_words`) and restored on **Ok and Err** alike. NOT upstream's shape: nothing is pushed on the guest stack, SP is untouched, and on `Err` the registers are the caller's, not the fault's (upstream #1534 leaves the fault registers for its `with_arm_context` diagnostics — a dump taken after a failed `run_function` shows the caller's registers here; the fault's pc/lr are in the error text / log instead, as before the merge) |
| `ArmCore::run_function_sync::<R>(address, params, svc: impl FnMut(&mut ArmCore, u32) -> Result<()>) -> Result<R>` | new (upstream) | no yield, no registered async SVC handlers; the closure gets the SVC category. Restores context on Ok and Err |
| `ArmCore::guest_root_words(&self) -> Result<Vec<u32>>` | OURS, the one GC-root word source | live registers + every `saved_contexts` entry + every thread's saved context + each thread stack from its lowest live SP |
| `ArmCore::stack_words()` | **REMOVED** (upstream's) | one mechanism. Upstream caller that must go/port: `wie-ktf/src/runtime/java/jvm_support/jvm_implementation.rs:125` (`ExternalRoots` impl) → `guest_root_words()` or drop the impl |
| `ArmCoreContext` | upstream: `#[repr(C)] #[derive(Clone, Copy, Pod, Zeroable)]`; ours: `words() -> [u32; 17]` kept | ★`Copy` now — our `context.clone()` call sites trip `clippy::clone_on_copy` under `-D warnings` (drop the `.clone()` / use `.copied()`); `read_thread_context` returns it by value |
| `ArmCore::{id, teardown, on_teardown, current_thread_id, current_stack_base, others_preempted, hold_others, make_shared_svc_stub, make_svc_stub, register_svc_handler, enter_thread_context, read_thread_context, write_thread_context, delete_thread_context, run_in_thread, get_thread_ids, map, dump_reg_stack, save_context, restore_context, read_pc_lr, read_param, write_return_value, set_next_pc}` | unchanged from ours | |
| `ResultWriter` | + upstream impls for `i64`, `f64` (r0/r1 pair) | additive |
| `Allocator::{init, alloc, free, is_allocated}` | signatures unchanged | see ⑶: whole heap stays mapped (ours) |
| `install_binary_patches(core, data, scan_ranges) -> Result<usize>` / `install_title_binary_patches(..)` | unchanged (ours) | call before `prepare_execution`; a later call is still correct (⑵) |
| `wie_core_arm::stdlib::{memcpy, memset, memmove, strcpy, strlen, ..}` | unchanged (ours); upstream deleted the module | upstream LGT has its own copies in `wie-lgt/src/runtime/stdlib.rs` — both can exist |
| `EngineRunResult.instructions_executed` → `budget_consumed` | crate-internal rename (upstream) | |

Renames old → new that callers will hit: `ArmCore::new(bool, Option<ProfileCallback>)` → `ArmCore::new(Options)`; nothing else public was renamed in this crate.

###### ⑵ AOT × binary patches — answer with code

**Are the patches installed before the AOT snapshot? Yes on both live paths; one path never prepares at all.**
- The snapshot is taken in `Aot::begin` — `wie-core-arm/src/aot.rs:59-66`: `compile_request(memory, take(&mut self.ranges), None)` reads the image bytes out of guest memory **at that call** (`code_image`, `aot.rs:170-195`). `ArmCore::load` only records ranges (`record_image`, `aot.rs:53`).
- KTF: `wie-ktf/src/runtime/init.rs:91` `install_binary_patches(..)` → `:93` `core.prepare_execution().await?` (as the ktf fork resolved the hunk: ours first, upstream's line after). Patched bytes and hook SVCs are in the snapshot.
- LGT: `wie-lgt/src/runtime/init.rs:196` `install_title_binary_patches` runs inside `load_executable`, which returns before `:46` `core.prepare_execution().await?`. Same.
- ★KTF relocated images: `init.rs:71` `return relocated::load(..)` leaves before `:93`. With `aot: Some` nothing is ever compiled **and `is_preparing()` stays true forever** (state `Loading`). Harmless with `aot: None`. → outside-slice request in ⑸.

**Does a later guest-memory write invalidate compiled regions? No — by upstream's contract.**
- `wie-core-arm-aot/src/lib.rs:167,173`: «guest stores do not invalidate compiled code». Only the guest's own CP15 cache-maintenance instruction does (`arm32_cpu.rs` `instruction_cache_invalidation` → `aot.invalidate`). Upstream's own test pins it: `invalidated_regions_recompile_on_entry_without_blocking_other_code` writes `0x1000` through `mem_write` and the region stays installed.
- Our host-side code writes that happen AFTER install: `hook::unhook` (a hook that declines gives the site back — `binary_patches/hook.rs:405`), shadow mode's swap of the site (`hook/shadow.rs:166,174`), and any `install_*` called after `prepare_execution`.

**What I did (smallest correct place): the engine is told.** `ArmCore::write_code` (`core.rs:655`, `pub(crate)`) = `mem_write` + `ArmEngine::code_written` (`engine.rs:38`, impl `arm32_cpu.rs:317`):
- `Loading` → nothing (the snapshot is yet to be taken, from memory);
- `Ready` → `aot.invalidate(range)` — upstream's own mechanism: regions over those bytes are released and recompiled from memory on their next entry;
- `Preparing` → the snapshot in flight is stale and cannot be recalled → `tracing::warn!` + drop the backend for this core (interpreter from here).
All five code-write sites of `binary_patches` go through it (`hook.rs:249,405` · `patch.rs:103` · `shadow.rs:166,174`). Data writes (frame slots, copies) stay `write_bytes`.
Test: `engine::arm32_cpu::tests::host_code_writes_reach_the_aot_backend_in_every_preparation_state` — the write before preparation is in the snapshot (immediate 2), a write when Ready removes the region's entry and releases its handle, a write while Preparing leaves `aot == None` and `is_preparing() == false`.

Why hooks are otherwise compatible: a hook site is a Thumb `svc #0x80`; the AOT decoder does not decode SVC (`aot/analysis.rs:365` — 0xdf is «undefined/SWI»), so the site is a hole no region owns, the interpreter executes it from live memory, and the registered SVC handler runs as before. Not measured on a title — both hosts pass `aot: None`.

**★Not held under AOT: the null-page read rule (0551 row #22).** Interpreter: a guest READ below 0x1000 of unmapped memory reads 0 (`Arm32CpuMemory::read_page`, test `guest_reads_through_null_are_zero_but_writes_and_higher_reads_still_fault` passes). Compiled code reads the page directory directly and exits `GuestFault`, which upstream defines as terminal «without reconstructing flags or partial-instruction effects» (`wie-core-arm-aot/src/lib.rs:186`) — the instruction cannot be handed back to the interpreter for a retry, and making it resumable means changing both code generators (`wie-core-arm-native`, `wie-core-arm-wasm`), which I did not touch. So under AOT a title that reads through null (2b1ed0c8d061, `docs/report/0391`) stops with `InvalidMemoryAccess(addr)` — loud, not silent; the engine adds a `warn` naming the reason (`arm32_cpu.rs` run loop, GuestFault arm). This belongs on the list of the ticket that adopts AOT. No test: there is nothing to hold.

###### ⑶ Behaviour divergences — ours kept

| 자리 | 우리(유지) | upstream |
|---|---|---|
| `ArmCore::run_function` — where the caller's registers wait during a host→guest call | host side, `saved_contexts` map; guest stack and SP untouched | pushed on the guest stack: `sp = (sp − 68) & !7` for every nested call (#1545) |
| `ArmCore::run_function` on `Err` | caller's registers and SP restored (test `failed_arm_calls_restore_caller_registers_and_stack`, put back; upstream's `…preserve_fault_registers…` removed) | fault registers left in place for diagnostics (#1534) |
| GC root words | `guest_root_words()` — live regs + nested saved contexts + thread contexts + stacks from the lowest live SP | `stack_words()` — removed here |
| Guest heap mapping (`Allocator::init`) — ★upstream-only change that arrived WITHOUT a conflict | whole 256 MB heap mapped up front, as before the merge: heap the guest never allocated reads 0 and takes writes | pages committed as allocations reach them (#1415 lineage): touching an untouched 64 KB heap page is `InvalidMemoryAccess`. Upstream's `map` calls in both allocators stay (no-ops here); its test assertion «untouched heap stays uncommitted» is flipped |
| Guest thread stack (`thread.rs` `STACK_SIZE`) — ★also arrived without a conflict | 1 MB, as before the merge | 64 KB. The stack is a heap block: recursing past it writes into neighbours, no fault |
| Guest READ through null (<0x1000, page unmapped) | reads 0, logged once (interpreter). Under AOT-compiled code: fault (⑵) | fault |
| Undefined-instruction error text | `Undefined instruction at pc=0x… (thumb|arm) insn=0x… lr=0x…` (test with the exact string passes) | `Undefined instruction at 0x… (CPSR: 0x…)` |
| Branch to the null page | error + `tracing::error!("guest branched to {pc}, lr={lr}")` | error only |
| `BucketAllocator::free` | bucket resolved from the address; the caller's size is only checked (warn) | (0551 #19) by size |
| Interpreter loop with no AOT backend | reads only pc per instruction (no cpsr read, no CP15 decode, no post-step reads) — speed only, same results | reads cpsr + decodes for CP15 every instruction even with no backend |

The two ★ rows are the ones to rule on: nobody chose them — they are upstream-only edits to lines we never touched, so the plain merge would have taken upstream's. I kept ours under the «guest-observable → ours» rule (heap: `allocator.rs` one `core.map` line + one flipped assertion; stack: one constant). Reverting either to upstream's is a one-line change.

###### ⑷ Silently merged files (both sides touched, no conflict) — read

| file | merged result | finding |
|---|---|---|
| `src/allocator.rs` | ours (failure report with list/bucket usage, address-based free) + upstream (lazy commit, `is_allocated` test) | upstream dropped `core.map(HEAP_BASE, HEAP_SIZE)` → **put back** (⑶) |
| `src/allocator/list.rs` | ours (`usage`) + upstream (`map` on alloc, `is_allocated` tolerates an unmapped header) | fine; tests pass |
| `src/context.rs` | upstream `Copy + Pod + repr(C)` + ours `words()` | fine; our `.clone()` calls became `clippy::clone_on_copy` (fixed here; same will hit ktf/lgt) |
| `src/thread.rs` | ours `preempted` + upstream `STACK_SIZE` 64 KB | **constant put back to 1 MB** (⑶) |
| `src/binary_patches/{parser,patch}.rs` | upstream deleted, parent restored from origin/main | `parser.rs` byte-identical to ours; `patch.rs` = ours + `write_code` + test port. ★Both are **untracked on disk** (`?? …` beside a staged `D`) — they need `git add` |
| `src/binary_patches/hook/shadow.rs` | ours-only | 2 lines → `write_code` |
| `Cargo.toml` | upstream (`arm32_cpu ^0.1.2`, `hashbrown`, `wie-core-arm-aot`, dev `wie-core-arm-wasm`) + parent's `md5`/`serde`/`toml` | fine |
| `src/engine.rs`, `engine/debugged_arm32_cpu.rs`, `function.rs`, `gdb/*`, `aot*` | upstream-only | + `code_written` on the trait and the debug engine (delegates) |

Lost-symbol check: every `fn` name of origin/main and of upstream/main in `core.rs`, `arm32_cpu.rs`, `allocator*.rs` is in the merged files except (a) upstream's `stack_words` (dropped on purpose), (b) upstream's test `failed_arm_calls_preserve_fault_registers_and_stack` (replaced by ours), (c) our base-era test name `run_reports_executed_instructions_at_budget_and_return_boundaries` (upstream renamed it `run_reports_consumed_budget_at_yield_and_return_boundaries`; same body). Upstream's test `nested_arm_calls_share_the_budget_and_keep_suspended_roots_visible` is kept and now asserts through `guest_root_words()` — it passes on our mechanism.
`page_table_is_heap_allocated` adapted: upstream's directory is `Box<[MemoryPage; 0x10000]>`, so the assertion is «one pointer», not the old fat-pointer size.

###### ⑸ Outside-slice change requests

- **parent**: `git add wie-core-arm/src/binary_patches/parser.rs wie-core-arm/src/binary_patches/patch.rs` (untracked, see ⑷) along with the rest of the crate.
- **parent / hosts** (`wie_cli`, `wie_featurephone`, `wie-web`, root `src/`): `ArmCore`/emulator construction takes `Options { enable_gdbserver, aot, profile }`; ours pass `aot: None`.
- **wie-ktf**: `runtime/java/jvm_support/jvm_implementation.rs:125` calls the removed `core.stack_words()` (upstream's `ExternalRoots`). `relocated::load` must end in `core.prepare_execution().await?` too (⑵). Upstream's `with_arm_context` diagnostics (#1534) assume the fault registers survive a failed `run_function`; here they are the caller's.
- **wie-ktf / wie-lgt**: `ArmCoreContext` is `Copy` — `.clone()` on it fails `clippy -D warnings`.
- **perf check (parent's V list)**: the interpreter loop is ours when `aot` is `None`, but `Allocator::alloc` now calls `core.map` once per allocation (upstream; a page-directory check, one lock) — `perf-watch.sh compare` is the measurement.

###### ⑹ Final command outputs (2026-11-23 20:5x KST, load1 ≈ 11)

| command | result |
|---|---|
| `cargo check -p wie-core-arm --all-targets` | rc=0, 0 warnings |
| `RUST_MIN_STACK=4194304 cargo test -p wie-core-arm` | rc=0 — **172 passed, 0 failed** (lib) · doc-tests 0 |
| `cargo clippy -p wie-core-arm --all-targets -- -D warnings` | rc=0 |
| `cargo clippy -p wie-core-arm --target wasm32-unknown-unknown -- -D warnings` | rc=0 |
| `cargo fmt -p wie-core-arm` then `-- --check` | rc=0 |
| `cargo check -p wie-core-arm-aot -p wie-core-arm-native -p wie-core-arm-wasm --all-targets` | rc=0 — the three crates are untouched (no worktree change) |
| `/usr/bin/grep -rn -E '^(<<<<<<<|\|\|\|\|\|\|\||=======|>>>>>>>)' wie-core-arm` | no output |

Not run: any title (`wie_validate`) — the workspace above this crate does not compile yet; AOT with a real backend on a real image.


#### 부록 B — `wie-wipi-c` 묶음

##### wie-wipi-c — S3 merge slice notes (2026-11-23)

9 unmerged files resolved, 0 markers left. Files I touched beyond the 9: `src/globals.rs` (+`#[allow(dead_code)]` on
`PENDING_TIMERS_HEAD`), `src/api/graphics/image.rs` (restored to ours = base — see ⑵), `src/api/fs.rs` (two test
assertions: aid namespace). Nothing outside `wie-wipi-c/` was edited. Nothing is staged (`git add` is the parent's).

###### ⑴ Decisions

####### ⑸ `api/database.rs` — ours, whole. `api/fs.rs` stays in the tree but must NOT be wired where ours serves.
- ★0551 row #65 is stale: upstream did not leave the path deleted. Today `upstream/main:wie-wipi-c/src/api/database.rs`
  is a 350-line rewrite as the real `MC_db*` record API (handle magic `0x44424844`), and the file slots moved to
  `api/fs.rs` (magic `0x46534648`, store = `System::filesystem()`). So the status is UU, not UD.
- Both upstream modules keep their own handle struct and (fs.rs) a different store. Ours keeps every slot in
  `platform().database_repository()` under `system.pid()` with magic `0x4D434442`. A handle or a name made by one is not
  seen by the other ⇒ per table it is all-ours or all-upstream, and all-upstream moves saves to another store
  (forbidden: save key unchanged, no user-save migration). ⇒ `database.rs` = `origin/main` byte-for-byte
  (`cmp` against `git show origin/main:…` = identical), 20 tests kept.
- Upstream's rewritten `database.rs` is therefore NOT in the tree (same path). Lost: `database::get_access_mode`,
  `database::get_record_size` (no ours equivalent; KTF slot 11 `GetRecordSize` is served by ours `list_databases`,
  measured — callers pass no argument). Recurring cost: every later upstream edit to `database.rs` conflicts and
  resolves to ours. S4 candidate (not done): move ours to a fork-named module so upstream's two files merge clean.
- `fs.rs` (upstream-added, 2 tests) compiles and passes against the merged backend; it is dead for our tables.

Slot → function map the KTF/LGT resolvers keep (= `origin/main` wiring, all `wie_wipi_c::api::database::*`):

| consumer | slot | ours function |
|---|---|---|
| KTF Interface4 record DB 0..7, else | open/close/delete/insert/select/update/delete_record/list, count | `open_record_database` · `close_database` · `delete_database` · `insert_record` · `select_record` · `update_record` · `delete_record` · `list_record` · `get_number_of_records` |
| KTF table upstream now calls `Fs` (ours `WIPICDatabaseMethodId`) | OpenDatabase · StreamRead · StreamWrite · CloseDatabase | `open_database` · `stream_read` · `stream_write` · `close_database` |
| 〃 | SelectRecord(slot4 lseek) · UpdateRecord(stat) · DeleteRecord · ListRecord(slot7 list/rename) · SortRecords(slot8 mkdir) | `select_record_ktf` · `stat_by_name_ktf` · `delete_record_ktf` · `list_record_or_rename_ktf` · `sort_records` |
| 〃 | GetNumberOfRecords · GetRecordSize · ListDatabases · Unk15(size) · Exists | `get_number_of_records` · `list_databases` · `list_databases` · `file_size_ktf` · `exists_database_ktf` |
| LGT SVC | OpenDatabase · ReadRecordSingle · WriteRecordSingle · CloseDatabase · Unk12(seek) · Unk9 · DeleteRecord · ListRecord · UpdateRecord · SelectRecord · ListDatabases(`0x19c`) · Unk8 | `open_database_lgt` · `stream_read` · `stream_write` · `close_database` · `seek_record_single` · `list_record_info` · `delete_database` · `list_record` · `update_record` · `select_record` · `list_databases` · `exists_database` |

Upstream wiring that must not be copied into those tables: `database::{open, close, delete, list_records, get_access_mode,
get_record_size}` (names do not exist in ours) and every `fs::*` (`open/read/write/close/seek/tell/file_attribute/remove/
rename/make_directory/remove_directory/list/total_space/available/set_mode/get_counts/exists`). `fs::available` is what
upstream maps LGT `0x19c` to — ours stays `database::list_databases`.

####### ⑹ Timers — ours (host event queue + pacing share), return codes ours.
- `kernel.rs` = `origin/main` + upstream's three stubs (`execute`, `get_exec_names`, `get_access_level`) + the
  Result-iterator `sprintf` call form. `set_timer` on a pending timer returns **-1** (not upstream's -5), `def_timer`
  resets a pending timer, `unset_timer` cancels through `event_queue().cancel_timer`. Tests kept: `test_unset_timer_cancels_a_pending_timer`,
  `test_set_timer_on_a_pending_timer_fails`, `test_when_a_timer_may_keep_a_tick_alive`, `test_a_timer_may_fire_speed_times_per_tick`.
- Not taken: #1536's guest-memory `PendingTimer` list (`globals::PENDING_TIMERS_HEAD` stays defined, unused) and its
  test `timers_cancel_and_rearm_without_stale_callbacks` (it asserts -5 and pops upstream's `VecDeque<(Instant, body)>`;
  the cancel / re-arm / Def-reset cases it covers are in the four tests above).
- `WIPICContext::set_timer(&mut self, due: Instant, pace_from: u64, callback: WIPICMethodBody)` — ours, 3 args.
  Upstream's KTF/LGT impls have the 2-arg form: keep ours.

####### Trait `WIPICContext` after the merge (= ours; upstream changed nothing but `set_timer`'s test double)
`alloc_raw` · `alloc` · `free` · `free_raw` · `data_ptr` · `call_function` · `system` · `spawn` · `java_property` ·
`get_resource_size` · `read_resource` · `set_timer(due, pace_from, callback)` · `wide_graphics_context() -> bool` (default false).
`TestContext` (cfg(test)): ours' `timers: Vec<(Instant, u64, body)>`, `calls`, `call_args`, `spawned`, `wide_graphics_context`
+ upstream's `functions` (a registered address is called, any other answers 0) + upstream's `global_data` region at
`0x7fff0000` (out of range = `InvalidMemoryAccess`, not a panic). Upstream's `function_calls` = our `call_args`, not duplicated.

####### Names: upstream → what exists here
| upstream | here |
|---|---|
| `net::set_read_callback` · `net::set_write_callback` | `net::set_read_cb` · `net::set_write_cb` (ours; real on a fake server's fd, -1 otherwise) |
| `net::socket/socket_connect/socket_write/socket_read` (stubs -1) | same names, ours bodies (fake server) |
| `graphics::{create_wipi_image, image_buffer}` (re-exports) | not exported. Ours: `graphics::decode_image_framebuffer`, `graphics::image_alpha`, `graphics::WIPICGraphicsContextIdx`, `PixelOpArgs`, `blit_with_pixel_op`, `screen_framebuffer(_handle)`, `ScreenFramebufferSync`, `draw_image_with_context_pixel_op`, `copy_frame_buffer_with_context_pixel_op`, `draw_polygon` |
| `graphics::fill_polygon` (stub) | present beside ours `draw_polygon`; LGT `0xf0` stays on `draw_polygon` |
| `canvas::encode_bmp(image)` | used as upstream has it; `graphics::encode_image` crops the region itself |
| `kernel::sprintf(ctx, fmt, impl Iterator<Item = Result<u32>>)` | upstream's signature (callers: `args.into_iter().map(Ok)`) |
| `kernel::{execute, get_exec_names, get_access_level}`, `media::{get_default_volume, clip_clear_data}`, `util::{htonl, ntohs}`, `im::*`, `serial::*`, `term_res::*`, `uic::*`, `misc::back_light` | present, upstream's |
| `util::inet_addr` | ours body, same `(ctx, u32) -> u32` signature |

Guest globals upstream's modules read/write: `0x7fff1000` screen framebuffer ptr (ours too), `0x7fff1020` input-mode table
(`im.rs`). The KTF/LGT cores must keep that page mapped if `im::*` is wired.

###### ⑵ Behaviour divergences (ours kept unless marked)

| 자리 | 우리(유지) | upstream |
|---|---|---|
| `MC_knlSetTimer` on a pending timer | refused, **-1**, due time unchanged, param replaced; host event-queue arming + pacing share | **-5** (`M_E_EXIST`), guest-memory pending list (#1536) |
| `MC_knlAlloc(0)` / `MC_knlCalloc(0)` | NULL handle (0); fresh blocks zeroed; OOM → NULL | a real handle; `alloc` not zeroed; OOM is fatal |
| `MC_knlGetTotalMemory` / `GetFreeMemory` | `0x200000` (2 MiB, swept over KTF callers — report 0456) | `0x10000000` |
| `MC_knlGetSystemProperty("PHONENUMBER")` | `"01000000000"` | `""` |
| 〃 `MEDIADEVICES` · `VOLUMELEVEL` | `"Yamaha_MA3"` · `"3"` | unknown id → -9 |
| `sprintf` / `MC_knlSprintk` bytes | format and `%s` bytes pass verbatim; `%-`, `%i`, integer precision, `%s` precision + padding | EUC-KR decode → re-encode; no `-`, `%i`, integer precision |
| `MC_utilInetAddr("010.001.002.003")` | parsed (leading zeros are decimal) | `INADDR_NONE` (`Ipv4Addr::from_str`) |
| `MC_grpRepaint` | `Screen::request_redraw()` — the host hands an `Event::Redraw` back at its boundary (wie-midp `event_queue.rs` tests rely on it) | pushes `Event::Repaint` on the queue (#1516) |
| `MC_grpCreateImage` backing (shared path = KTF) | pixels in the decoder's format (ARGB 32bpp for alpha images), no mask; LGT converts to RGB565 + 8bpp mask in `wie-lgt` | RGB565 + 8bpp alpha mask for every carrier (`image.rs` #1580) |
| `MC_grpSetContext(CLIP)` rectangle | x1,y1,x2,y2 **inclusive**, intersected with the draw's own rectangle; text is not clipped by it; a null clip also resets the rectangle | left,top,right,bottom exclusive, **replaces** the draw rectangle (draw_image only) |
| `MC_grpEncodeImage` on a region partly/fully outside | clamped to the image; empty → 0 with `*len` untouched | any overhang → 0 and `*len = 0` |
| `MC_netSocket*` · `SetReadCB` · `SetWriteCB` | fake server for listed titles; connect failure through the callback | -1 stubs |
| `MC_mdaGetVolume/SetVolume/ClipGet/SetVolume`, clip callbacks | real (AudioSink gain, STARTED/END_OF_DATA/STOPPED) | warn stubs |
| DB/file slots | `database_repository` store, ours semantics (table in ⑴) | `fs.rs` on `System::filesystem()` + `MC_db*` rewrite |
| ★taken from upstream — `MC_grpDrawString` y | — | y is a **baseline**: text is drawn `FONT_SIZE` (10 px) higher than on `origin/main` (#1517; 0551 row #67 said to carry it). User-visible in every KTF/LGT native text screen; no value a guest reads changes. To put ours back: delete the two lines at `graphics/primitives.rs:197-198` and the test `strings_are_positioned_at_the_requested_baseline`. |
| ★taken from upstream — `MC_grpDestroyImage` | — | also frees the image's pixel and mask buffers (ours leaked them); I added NULL guards (handle 0, buf 0) |

Upstream tests not carried because they assert the upstream column: `kernel::timers_cancel_and_rearm_without_stale_callbacks`,
`kernel::test_zero_size_memory` (ours `test_zero_size_memory_returns_null` kept), `graphics::encoded_framebuffer_region_can_be_loaded_as_an_image`,
`graphics::indexed_images_preserve_transparency_and_context_clipping`, `graphics::png_images_blend_partial_alpha_with_native_pixels`.
Carried: `graphics::strings_are_positioned_at_the_requested_baseline`, the mask-bit lines in `context_values_are_written_to_the_output_pointer`,
`sprintf::sprintf_formats_euc_kr_guest_strings_and_values` (passes on ours), `util::ipv4_conversion_returns_network_order` (+1 leading-zero row),
both `fs` tests (aid namespace), `im`'s test.

###### ⑶ Silently merged files checked
- Both sides changed, no conflict: only `src/lib.rs` — result = upstream's `mod globals;` + our `wipi_vocabulary_lock` test.
  That test compiles and passes on `wipi-types 0.0.5` (the `WIPICError` vocabulary did not move).
- Upstream-only files that came in: `api.rs` (mods fs/im/serial/term_res), `fs.rs`, `im.rs`, `serial.rs`, `term_res.rs`,
  `uic.rs` (+122), `globals.rs`, `graphics/image.rs`. Overlap with ours: `image.rs` (upstream's 565+mask rewrite deletes
  `decode_image_framebuffer`, which ours `graphics.rs` re-exports and `wie-lgt` calls) → restored to ours; `globals.rs`
  → allow on the unused timer slot; the rest untouched. `uic.rs`/`im.rs` duplicate what `wie-lgt` does locally in ours
  (Im*, Uic* handlers) — which one LGT wires is the LGT resolver's call (ours = local handlers).

###### ⑷ Needed outside the slice
1. ★**`wie-backend/src/database.rs` — blocks this crate's build right now.** `DatabaseRepository::usage` was given a
   default body; under `#[async_trait]` a default-bodied `&self` method gets `where Self: Sync`, so it cannot be called
   on `dyn DatabaseRepository`: `error[E0277] … wie-wipi-c/src/api/database.rs:363` (`list_databases`, KTF free-space slot).
   Fix in the backend, either: make it a required method again (as on `origin/main`), or keep the default but write it
   without capturing `self`:
   `fn usage<'a, 'b, 'c>(&'a self, _app_id: &'b str) -> Pin<Box<dyn Future<Output = u64> + Send + 'c>> where 'a: 'c, 'b: 'c, Self: 'c { Box::pin(async { 0 }) }`.
   `test-utils/src/platform.rs:326` already implements `usage`, and `database::tests::ktf_available_database_storage_tracks_app_usage` needs it real.
2. wie-ktf / wie-lgt: keep `set_timer(due, pace_from, callback)` impls; keep ours DB/net wiring (tables in ⑴); new
   upstream stubs are available to wire for slots ours left unimplemented.
3. `docs/contracts/runtime-adjustments.tsv:50` cites `wie-wipi-c/src/api/database.rs` — still true (path kept).
4. Pre-existing, not from the merge: `cargo clippy -p wie-wipi-c --all-targets` reports `manual_is_multiple_of` twice in
   ours test code (`graphics/primitives.rs:436,483` — same lines on `origin/main`); the gate form (no `--all-targets`) is clean.

###### ⑸ Commands
- `/usr/bin/grep -rn -E '^(<<<<<<<|\|\|\|\|\|\|\||=======|>>>>>>>)' wie-wipi-c` → 0 lines.
- With the tree as left: `cargo check -p wie-wipi-c --all-targets` → **rc≠0, 1 error** — the backend one in ⑷-1, nothing else.
- With that single call stubbed in a scratch edit (since reverted; `database.rs` is `cmp`-identical to `origin/main`):
  `cargo check -p wie-wipi-c --all-targets` rc=0, 0 warnings · `RUST_MIN_STACK=4194304 cargo test -p wie-wipi-c` →
  **90 passed, 1 failed** (the one failure = `ktf_available_database_storage_tracks_app_usage`, which is the stubbed call itself) ·
  `cargo clippy -p wie-wipi-c -- -D warnings` rc=0 · `cargo clippy -p wie-wipi-c --all-targets -- -D warnings -A clippy::manual_is_multiple_of` rc=0 ·
  `cargo fmt -p wie-wipi-c -- --check` rc=0 (on the restored tree).
- Not run: anything in wie-ktf / wie-lgt (not resolved yet), wasm clippy.


#### 부록 B — `wie-ktf` 묶음

##### wie-ktf — S3 merge slice notes (fork s3-ktf · 2026-11-23)

14 unmerged files resolved, 0 markers left under `wie-ktf/` and `wie-ktf-dump/`. Also edited inside the slice (not in
the unmerged list): `src/runtime/wipi_c/interface.rs` (table 5 back to 64 slots), `src/runtime/relocated.rs`
(`prepare_execution`, `ArmCore::new(Options)`, `clone_on_copy`), `tests/test_{key_reach,resource_reach,emulator_freed}.rs`
(`aot: None`). Nothing outside `wie-ktf/` was edited. Nothing is staged.

###### ⑴ Decisions

####### 몫 ⑵ — GC roots: one mechanism, ours. `InitSvcId` 7 stays `JavaThrowInstance`; upstream's `VisitGcRoot` is dropped.
- **What the two things are** (code, both sides):
  - `InitParam4.fn_visit_gc_root` is a **slot in a struct the guest reads** (`wipi_types::ktf::InitParam4`, handed to the
    image's `fn_init` as its 5th argument) — guest ABI by position. Its **value** is whatever the host puts there.
  - `InitSvcId::*` is **host-internal numbering**: the id baked into the SVC stub the host plants
    (`core.make_svc_stub(SVC_CATEGORY_INIT, id)`); the guest never sees the number. So 7 is free to be ours.
- **Who calls the slot**: only the image's own `WipiExe.fn_visit_gc_roots`, and only the host starts that function —
  upstream's `ExternalRoots::roots` does (`core.run_function_sync(context.ptr_visit_gc_root, …)`,
  `upstream/main:wie-ktf/src/runtime/java/jvm_support/jvm_implementation.rs`). Under our design nothing calls
  `fn_visit_gc_roots`, so nothing reaches the slot: it stays `0`, exactly as on `origin/main` (and in the merge base —
  `44fbf265:wie-ktf/src/runtime/init.rs:103` already had `fn_visit_gc_root: 0`).
- **Not taken** (all one upstream design, #1542/#1545): `InitSvcId::VisitGcRoot`, the `fn_visit_gc_root` stub,
  `KtfJvmSupportContext.ptr_visit_gc_root` + its write at the end of `load_native`, `impl ExternalRoots for
  KtfJvmImplementation` (needs `ArmCore::stack_words`, which core-arm removed, and runs guest code during a collection —
  not «purely additive»), the four `(support.ptr_visit_gc_root != 0).then(push_native_frame)` frames, the removal of
  `register_static_classes`, the removal of the loader's `nativeStrings` Vector.
  `KtfJvmImplementation::external_roots()` = trait default `None`. `jvm_implementation.rs` = `origin/main` byte for byte.
- ★**Two silent drops the auto-merge made OUTSIDE conflict hunks, restored by hand** (both would have compiled):
  1. `emulator.rs` `start`: the call `KtfJvmSupport::register_static_classes(core, &jvm, class_loader, &main_class_name)`
     was gone (upstream deleted it; we had not touched that line). Ours roots native classes' statics through it.
  2. `ktf_class_loader.rs`: the field proto `nativeStrings` was gone while our `init` still `put_field`s it.
- Upstream frames taken (they scope lifetimes, they do not replace a root source): `handle_wipic_svc` (one per WIPIC SVC),
  `java_class_load`, `register_class`. ★The WIPIC one is a behaviour change worth a line in the report: objects a
  WIPIC call creates used to land in the frame of the host call the guest runs under (never released for a game loop);
  now they are released when the SVC returns. What the guest keeps is still rooted by our guest scan.
- Evidence: our GC tests pass on the merged tree — `test_register_static_classes_keeps_live_guest_roots_without_initializing`,
  `test_what_a_host_method_made_for_the_guest_is_collected_once_dropped`, `test_a_guest_new_the_guest_dropped_is_collected`,
  `test_a_guest_word_into_an_object_keeps_it_alive`, `init_param_4_wires_throw_instance`,
  `throw_instance_throws_the_instance_it_is_given`.

####### 몫 ⑶ — validation: both of ours pass, upstream's check kept for everything else.
- `adf.rs`: `is_relocation_prefixed` (ours) + `validate_client_bin` (upstream, unchanged: 12-byte stub + format header) +
  new `validate_client_bin_entry(filename, data)` = «relocation-prefixed for this filename's bss size, or
  `validate_client_bin`». Test `validation_accepts_the_stub_and_the_relocation_prefixed_image_only`.
- `emulator.rs`:
  - `validate_jar` → `validate_client_bin_entry`.
  - `validate_archive` → the jar `from_archive` runs: `archive_jar_entry` (`<AID>.jar`, else the archive's only jar),
    else the same name under the private dir (`P/` or `p/`). Test `validate_archive_finds_the_jar_from_archive_runs`;
    ours `jar_filename_falls_back_to_the_only_jar` kept.
  - `load()`: upstream calls `validate_jar` on the jar entry and fails construction on ANY error. Ours takes only the
    **header verdict** there (`find_client_bin` ok → `validate_client_bin_entry?`); «not a zip» / «no client.bin» still
    report from the start task as before. Reason: our `from_archive_turns_the_owner_sound_on` constructs an emulator
    around a jar that is not a zip (`WIRING_PROBE_JAR`) and must keep constructing. Upstream's
    `tests/test_helloworld.rs::test_unsupported_client_header` passes unchanged (3 bad headers × from_jar/from_archive/dump).
- ★Measured, not assumed — does the 12-byte check reject any title we have? Local corpus, counts only
  (`/tmp/wie-s3/ktf-scripts/hdr.py`): `game_lab/working/ktf` 190 client.bin = **190** standard 12-byte header;
  `game_lab/broken` 97 client.bin = **94** standard + **3** whose first word is the filename's bss size (the relocated
  ones: `b907b0faf483` · `1d5831e42a8a` · `83fc429f9cbe`). 0 of 287 is «stub in the first 4 bytes but a different
  header» and 0 is anything else. So with the relocation exemption the check rejects none of them.
- `dump.rs` (upstream-only file) still calls `validate_client_bin` directly: a relocated image is not dumpable by that
  tool (it runs the stub entry). Left as upstream has it.

####### 몫 ⑴ — binary patches: call stays, before `prepare_execution`.
`runtime/init.rs::load_native`: `core.load` → `guest_roots::add_region` → `wie_core_arm::install_binary_patches` →
`core.prepare_execution().await?` → SVC handlers → entry. `runtime/relocated.rs::load` also ends its load in
`prepare_execution` now (core-arm notes ⑴: with `aot: Some`, a load path that never calls it leaves `is_preparing()`
true forever; with `aot: None` it is a no-op). The relocated path has no `install_binary_patches` call on `origin/main`
either — unchanged.

####### `#1520` table rename — names taken, bodies ours (follows wie-wipi-c notes ⑴)
| guest table (`id >> 16`) | name before → after | served by |
|---|---|---|
| 4 | `Interface3` (5 unknown stubs) → `Interface3` = `get_im_method_table()` | upstream `im::*` (ours had only stubs) |
| 5 | `Interface4` → `Database` | **ours**: slots 0..=7, 10 = `database::{open_record_database, close_database, delete_database, insert_record, select_record, update_record, delete_record, list_record, get_number_of_records}`; other ids < 64 = `gen_unnamed_table_stub(5, "Interface4", id)`. Table written with **64** slots as before (upstream writes 13 from `get_database_method_table`, which is removed here — its `database::open/close/delete/list_records/get_access_mode/get_record_size` do not exist in our `database.rs`) |
| 7 | `Database` → `Fs` (`WIPICDatabaseMethodId` → `WIPICFsMethodId`, same numbers) | **ours**, id for id: Open=`open_database` · Read=`stream_read` · Write=`stream_write` · Close=`close_database` · Seek=`select_record_ktf` · FileAttribute=`stat_by_name_ktf` · Remove=`delete_record_ktf` · Rename=`list_record_or_rename_ktf` · MkDir=`sort_records` · RmDir=stub 9 · List=`get_number_of_records` · TotalSpace=`list_databases` · Available=`list_databases` · SetMode=stub 13 · GetCounts=stub 14 · Tell=`file_size_ktf` · Exists=`exists_database_ktf`. No `fs::*` is wired (other store → saves would move) |
★Trap met: our test `database_slot_11_answers_free_storage` said `WIPICTableId::Database` meaning table 7. After the
rename that name is table 5 and the test still compiled. Now `WIPICTableId::Fs`; `interface4_slot_0_…` uses `Database`.
Save namespace: untouched here (ours `database.rs`, `system.pid()` store).

###### ⑵ Behaviour divergences — ours kept

| 자리 | 우리(유지) | upstream |
|---|---|---|
| GC roots (KTF) | conservative guest scan (`guest_roots`, image region + `ArmCore::guest_root_words`) + `register_static_classes` + loader `nativeStrings`; `InitParam4.fn_visit_gc_root` = 0 | image's `fn_visit_gc_roots` run during GC + stack-word scan (`ExternalRoots`, #1542/#1545); slot = SVC stub |
| `InitParam4.unk1` | SVC stub `JavaThrowInstance` (`throw e` of an instance the guest built; `throw null` → NPE) | 0 |
| `KtfClassLoader.loadClass` | parent-first; native-first only for names the jar also ships as `.class` | native-first for every non-array class (#1584) |
| throw → catch lookup | walks the handler chain to a caller's catch (cap 4096), writes `e` slot and moves the record's pc to the handler target, drops the dead frames' records | innermost record only; writes `e` slot; a method without catches → uncaught at once |
| WIPIC table 7 (`Fs`) / table 5 (`Database`) | `api::database` bodies, 64-slot table 5 (table in ⑴) | `api::fs` on the filesystem store / new `MC_db*` module, 13-slot table |
| WIPIC net slots | 2·6·13 = `local_net` (in-engine server); 3·4·5·14 report `Unimplemented` | `net::socket*`/`set_*_callback` = -1 stubs |
| resource size / read (`MC_knlGetResourceID` path) | `ResourceCache` | class-loader stream each call + `collect_garbage` |
| `call_native` result | both words written back (J/D high word), result published in the thread context honoured | r0 only |
| jar that cannot load | bad client.bin header → error at construction (upstream's check, relocation-prefixed images exempt); not-a-zip / no client.bin → error from the start task | every case at construction |
| jar name | `<AID>.jar`, else the archive's only `.jar` | `<AID>.jar` only |
| relocation-table-prefixed client.bin (3 images) | loaded (`runtime::relocated`) | «Unsupported KTF client.bin format» |
| fatal error text from a tick | message + register/stack dump appended in `tick_for` (as before) | dump only where the fault is mapped (`with_arm_context`), none in `tick` |
| registers in a fault dump | the caller's (our `run_function` restores on Err too — core-arm) | the fault's (#1534) |

####### Upstream-only behaviour that arrived (ours had not touched the place — listed so the census knows what moved)
- `JavaClassDefinition::prepare` now rewrites inherited vtable slots to their overrides (`JavaVtable::resolve_overrides`,
  #1450). Runs for every native class of every KTF image, relocated ones included. ★Worth one `wie_validate` each on the
  three relocated rows above — I could not run them (wie_cli does not build yet, ⑷).
- Table 12 slot 1 = `unk_screen_flush` (flushes the screen framebuffer; was an unknown stub) — #1514.
- UIC table: slots renumbered from 12 on (two unknown slots inserted) and `paint/handle_event/configure/set_enable/
  set_event_handler/get_time/insert_text/delete_text/set_max_text_size/get_text_size/get_text` implemented; old slot 26
  `get_menu_item` is now `get_time` and menu-item is a stub at 28 — #1596 and earlier.
- Input method: table 4 and graphics ids 37..=41 = `im::*` (#1517). Kernel `GetExecNames`/`Execute`/`GetAccessLevel`
  implemented (`GetAccessLevel` reads the ADF `SLvl` through the new JVM property `wie.slvl`, #1597). `util` slot 0 =
  `htonl`. Graphics id 42 = `fill_polygon` stub (was `Unimplemented`).
- `KtfWIPICContext::alloc` zero-fills the payload (#1572); `call_function` errors carry a register dump (#1534).
- SVC results of `java_jump_1/2/3` and `call_native` are returned through the handler (#1534). Read off the code, not
  reproduced: before, the handler wrote
  the result and then the holder's `()` write set pc back to lr — a resume-at-handler pc from `map_exception_unwind` was
  overwritten. Now it is written once.
- An archive whose ADF has no `MClass` fails with «Main class not found» (was: tried to load class "").

###### ⑶ Silently merged files (both sides touched, no conflict) — read
| file | merged result |
|---|---|
| `src/runtime.rs` | ours + upstream `with_arm_context` (same text our `method.rs` built by hand). OK |
| `src/runtime/wipi_c/interface.rs` | upstream's `get_fs_interface` / `database_interface` field names (wipi-types 0.0.5). ★`get_database_method_table()` (13 upstream fns that do not exist here) → `get_stub_method_table(4)` (64 slots, ours) |
| `jvm_support/array_class_definition.rs`, `array_class_instance.rs`, `field.rs` | identical to `origin/main` (both sides made the same `JavaType` import move) |
| `src/emulator.rs` (non-hunk part) | ★`register_static_classes` call dropped → restored (⑴) |
| `jvm_support/classes/net/wie/ktf_class_loader.rs` (non-hunk part) | ★`nativeStrings` field proto dropped → restored (⑴) |
| `src/runtime/svc_ids.rs` (non-hunk part) | #1520 renames — kept, callers re-laid (⑴) |
| `jvm_support/method.rs` (non-hunk part) | upstream `run_with_unwind` shape — correct under our always-restoring `run_function`; the conditional arg frame removed |
| `java/interface.rs` (non-hunk part) | upstream return-through-holder — kept, our frame wrapper retyped to `Result<JavaMethodResult>` |
Upstream-only files that came in and compile: `Cargo.toml` (+`anyhow`, +dev `zip`), `dump.rs`, `jvm_support/name.rs`,
`jvm_support/vtable.rs`, `tests/test_helloworld.rs`, `wie-ktf-dump/src/main.rs`. `anyhow` is now unused in this crate
(it was for the `ExternalRoots` impl) — left in the manifest to keep it equal to upstream's; beta may list it under
`unused dependency` (rc 0).

###### ⑷ Outside the slice
1. Nothing in another crate has to change for wie-ktf to build and pass.
2. `wie_cli` does not build yet (`wie_validate.rs:1989` — `SktEmulator::from_jar` arity, not KTF), so I ran no real
   title. Owed by whoever closes the merge: runner block `helloworld_ktf` / `keydraw_ktf --inject --expect-last-frame`,
   the three relocated rows, and 0540/0547's two KTF rows (`974e0df9ab1e` · `f770b15f8876`).
3. Hosts that build `Options` by hand need `aot: None` (done for this crate's tests).
4. `cargo clippy -p wie-ktf --all-targets` reports `type_complexity` once in our test helper
   (`wipi_c/context.rs` `fn run(body: fn(KtfWIPICContext) -> Pin<Box<…>>)`) — the same line is on `origin/main`; the gate
   form (no `--all-targets`) is clean.

###### ⑸ Commands (merged tree as left)
- `/usr/bin/grep -rn -E '^(<<<<<<<|\|\|\|\|\|\|\||=======|>>>>>>>)' wie-ktf wie-ktf-dump` → 0 lines.
- `cargo check -p wie-ktf -p wie-ktf-dump --all-targets` → rc=0, 0 warnings.
- `RUST_MIN_STACK=4194304 cargo test -p wie-ktf -p wie-ktf-dump` → rc=0 · **69 passed, 0 failed** (lib 64 ·
  `test_emulator_freed` 1 · `test_helloworld` 2 · `test_key_reach` 1 · `test_resource_reach` 1).
  Every test name on `origin/main` is still present (name-set diff: ours-not-now = 0). New here: 2 (⑶ of 몫 ⑶).
  Upstream tests not carried: `test_native_gc_visitor_keeps_guest_references` (asserts the visitor design),
  `test_class_loader_before_native_initialization` (the head of our register-static test). Upstream tests adapted to
  our core, each with a `// fork:` line saying why: `test_native_virtual_overrides` (class records from
  `alloc_class_record`), `test_native_method_entry_points` (no `R4: 0x55` — fault dump shows the caller),
  `test_exception_class_matches_raw_class_and_vtable` (handler sp = caller sp; record pc reset before each nested throw),
  `timer_fault_diagnostics_survive_java_exception_restoration` (undefined-instruction half only — ours reads the null
  page as 0), `native_*` in `wipi_c.rs` (4-arg `register_wipic_svc_handler`).
- `cargo clippy -p wie-ktf -p wie-ktf-dump --no-deps -- -D warnings` → rc=0 (gate form; `--no-deps` because wie-midp was
  mid-resolution and red in clippy at the time). `… --all-targets … -A clippy::type_complexity` → rc=0.
- `cargo clippy --target wasm32-unknown-unknown -p wie-ktf --no-deps -- -D warnings` → rc=0.
- `cargo fmt -p wie-ktf -p wie-ktf-dump -- --check` → rc=0.


#### 부록 B — `wie-lgt` 묶음

##### wie-lgt — S3 merge slice notes (fork s3-lgt, 2026-11-23)

18 unmerged files resolved, 0 markers left under `wie-lgt/`. Nothing outside `wie-lgt/` was edited; nothing is staged.
Shape of the result: **the crate is `origin/main` plus a short, listed set of upstream additions.** Where upstream
re-derived the same guest ABI differently (exception frames, array-class tokens, vtable placement, the file/DB SVC
block, the LGT-native graphics module) ours stays whole — per-function mixing of two reverse-engineered ABIs was the
one thing I would not do unmeasured.

Files touched beyond the 18 (all inside the crate): `runtime.rs` (`with_arm_context` → identity, re-exports back to
ours), `runtime/globals.rs` (comment + `TIME_VALUE_PTR`), `runtime/java.rs` (`mod exception` private again),
`runtime/java/abi.rs` (= ours; upstream's unread `instance_field_word_count` field dropped),
`jvm_support/jvm_implementation.rs` (`exception::init` put back), `jvm_support/array_class_instance.rs` (base test
`storage_layout_matches_lgt_abi` restored — upstream's test purge had removed it silently), `tests/*.rs` ×4
(`aot: None` in `Options`).

###### ⑴ Decisions

####### 몫 ⑷ exception frames — ours, whole
- `runtime/java/exception.rs` = `origin/main` (only change: `SUPPORT_CONTEXT_BASE` now comes from upstream's
  `runtime/globals.rs`; same address `0x7fff0000`, comment there says 8 bytes = our `JavaSupportContext`).
- ★The auto-merge had already taken upstream's half OUTSIDE the conflict hunks, in two places: `emulator.rs`
  `LgtTaskRunner::run` (per-task `exception::init`/`destroy` + a `SUPPORT_CONTEXT_BASE` write on every poll) and
  `jvm_implementation.rs` (our `exception::init(core)?` deleted). Both are back to ours. Left as merged, the crate
  would not have compiled (`init` returns `()` here) — but it is the pattern 0551 §6 warned about.
- No upstream ABI detail was lost: its «fixed ABI cell» is the same word ours already uses; `unwind`'s tail
  (`restore_context` + `set_next_pc`) is identical on both sides; «catch consumes its frame» is in ours too.
- Tests: all 12 of ours in `exception.rs` pass (`exception_frames_are_per_thread`, `catch_that_pops_…`,
  `cx_thumb_catch_*` ×4, `release_to_*` ×2 …). Upstream's `emulator::tests::exception_frames_are_restored_per_thread`
  is NOT carried: it needs `init() -> ptr` and asserts upstream's pop convention (a pop after a catch always pops the
  enclosing frame — ours makes that pop a no-op when the catch's own frame was consumed at the same sp, which is the
  measured Thumb convention). The property it names is `exception_frames_are_per_thread`.

####### GC roots — ours
`guest_roots::install` in `LgtJvmSupport::init`, per-method native frames, `ArmCore::guest_root_words`,
`exception::root_words`. `external_roots()` stays the default `None`. Tests: `interface::tests::a_guest_word_into_an_object_keeps_it_alive`
and the `guest_roots` tests pass.

####### The four fixed SVC meanings (+ evidence after the merge)
| id | meaning kept | id → variant | variant → handler |
|---|---|---|---|
| `0x68` | terminate program (`System::exit_from_guest`) | `svc_ids::tests::wipic_svcs_0x68_0xf0_0x19c_0x44c_keep_the_fork_meaning` (new) | `wipi_c::tests::wipic_0x68_terminates_the_program` (runs the SVC stub) |
| `0xf0` | polygon outline (`graphics::draw_polygon`) | same new test | `wipi_c::tests::wipic_0xf0_draws_the_polygon_outline` (runs the stub, checks pixels) |
| `0x19c` | `database::list_databases` | same new test + `wipic_svc_412_list_databases_is_in_the_table` | `wipi_c.rs:233` match arm (compile-time; no test runs this stub) |
| `0x44c` | `list_data_stores` (empty list) | same new test | `wipi_c::tests::wipic_svc_ids_named_from_call_sites_answer` (runs the stub) |

★0551's upstream column is stale for `0x68`: upstream today maps it to `Exit` → `kernel::exit` (not an `Unk13`
stub) and has no `0x6b`. Ours: `0x68` terminate, `0x6b` `kernel::exit`.

Java-system ids 35/36/37/38: host-internal numbers only — the guest's import index → id map is `interface.rs`
(`0xfd` → StoreLongArray, `0x5b` → LoadLongArray, `0x26`, `0x64`), so upstream's «swap» is a renumbering, not an ABI
difference. Ours' numbers stay (`java_system_svc_ids_round_trip_and_table_stays_fail_closed` passes).

####### `runtime/stdlib.rs` — re-judged 폐기 → 재적용 (0551 row #51 differs)
Base = ours. The 8 slots 0551 called «covered upstream» differ in what a guest sees, so ours' bodies stay:
`malloc`/`free` (NULL on failure, 4-byte header — upstream: `Err` = run ends, 12-byte header, 8-aligned),
`strncmp` (−1/0/1 — upstream: byte difference), `strtol` (ours' `parse_long`), `strncat`, `strchr`, `vsprintf`
(0x3f9), `memmove` (via `wie_core_arm::stdlib`). Ours' svc-table tests (`stdlib_import_0x40a_…`, `…0x40b_…`,
`…0x415_…`, `…0x3ff_…`, `…0x3f9_0x408_0x426_0x428_…`, `localtime_fills_tm_in_kst`, `parse_long_follows_c_strtol`) all pass.
Taken from upstream (slots ours did not have): `strtok` 0x413, `memcmp` 0x416, `memchr` 0x417, `abort` 0x425,
`realloc` 0x427 (re-written onto our header; a failed allocation answers NULL and leaves the old block), `unk450` 0x450,
and upstream's tests `tokenization_resumes_…`, `variadic_formatting_reads_only_consumed_guest_arguments`, the
memcmp/memchr half of `memory_and_string_imports_…` (as `memcmp_and_memchr_imports_scan_guest_bytes_across_chunks`), plus
one new `realloc_atexit_and_abort_imports_answer`.
★Two places where upstream's behaviour was TAKEN — flag for the parent's table:
1. `0x424`: ours was unchanged base code (`unk3`) that **ran the callback at once**; upstream made it `atexit`
   (registers, returns 0, never runs it — #1502). Taken, as the brief listed atexit among the slots to take. To put
   ours back: `stdlib.rs` `atexit` body → `core.run_function(callback, &[])`, and drop the 0x424 line of the new test.
2. `sprintf`/`vsprintf` argument reads: upstream's lazy Result-iterator form (wie-wipi-c took that signature). `vsprintf`
   now reads a word only when the format consumes it, and a consumed word that is unmapped is an error (ours read up
   to 2 words per `%` up front and stopped silently at the first unmapped one). Same output for every well-formed call.

####### `wipi_c.rs` / `svc_ids.rs` — ours' table + 21 ids only upstream maps
Every id both sides map keeps ours' handler (database slots from `api::database`, shared graphics per slice D ⒝, local
Im*/Uic*/time/byte-order handlers, `net::set_read_cb/set_write_cb`). Added, with upstream's shared bodies:
`0x67` execute · `0x6e` get_access_level (reads `wie.slvl`, which `emulator.rs` now passes from `app_info` `SLvl:`) ·
`0xed` encode_image · `0x2bc/0x2bd/0x2c0/0x2c1` serial · `0x325 0x328 0x330 0x331 0x334 0x33f 0x342 0x343` uic (all
stateless stubs upstream) · `0x44d..0x451` term_res · `0x4bf` get_volume. Before the merge each of these was a fatal
«Unknown LGT WIPIC SVC id». NOT added: `0x19f` (`fs::tell` takes a handle of upstream's file table; ours serves `0x190..`
from `api::database`).

####### `wipi_c/graphics.rs` — ours, whole
Upstream rewrote the LGT-native graphics module; here that module is deliberately not wired (26 of 27 graphics SVCs go
to shared `wie_wipi_c::api::graphics`, slice D decision ⒝), and the rewrite calls `create_wipi_image`/`image_buffer`
from wie-wipi-c's upstream `image.rs`, which that slice restored to ours. So the merged text could not compile and
would have been dead code anyway. Only change: its two root constants now come from `runtime/globals.rs`.

####### `with_arm_context` (upstream #1534) → identity
Upstream appends the register dump at the failing call (5 call sites merged in: `init.rs` ×2, `wipi_c/context.rs`,
`net/wie.rs`, `lgt_class_loader.rs`) and removed it from `tick`. Ours dumps once in `LgtEmulator::tick_for` for every
error, and core-arm's `run_function` (ours) hands back the caller's registers on `Err`, so the upstream dump would be
a second copy of the same registers in the message. The call sites stay upstream-shaped; the function returns the
error unchanged. Upstream's test `callback_fault_diagnostics_survive_java_exception_restoration` is not carried (it
asserts the fault's `R4` in the message, i.e. upstream's `run_function`).

###### ⑵ Behaviour divergences (우리 유지 unless marked ★)

| 자리 | 우리(유지) | upstream |
|---|---|---|
| WIPIC `0x68` | terminate program → `exit_from_guest` | `Exit` → `kernel::exit` |
| WIPIC `0x6a` | `unk1` stub, returns 0 | `GetCurProgramId` |
| WIPIC `0x6b` | `kernel::exit` | unmapped |
| WIPIC `0xeb` | `unk0` stub, returns 0 | `DestroyImage` (frees the image) |
| WIPIC `0xf0` | polygon outline | `fill_polygon` (stub) |
| WIPIC `0x12c..0x130` (input method) | local: 4 modes EN/S·EN/L·KO·N123, multitap + Hangul composition per emulator | shared `im::*` stubs |
| WIPIC `0x190..0x1a0` | `api::database` slots on `database_repository` (open_database_lgt, stream_read/write, seek_record_single, list_record_info, delete_database, list_record, update_record, select_record, list_directory, list_databases, exists_database) | `api::fs::*` on `System::filesystem()` |
| WIPIC `0x19c` | `ListDatabases` | `FsAvailable` |
| WIPIC `0x19f` | unmapped (fatal, with r0–r3/lr in the message) | `fs::tell` |
| WIPIC `0x320..0x323`, `0x338` | time handle calls (`time_now/component/convert/to_tm`) | `uic::create_application_context/get_class/create/destroy`, `uic::get_time` |
| WIPIC `0x329 0x32b 0x33e 0x341` | local stubs, return 0 | `uic::configure/set_enable/insert_text/set_max_text_size` |
| WIPIC `0x384..0x388` | byte swaps incl. `0x386` ntohl; `inet_addr` | `util::htonl/htons/ntohs`; no `0x386` |
| WIPIC `0x44c` | `ListDataStores` (empty list) | `TermResGetSupportedGroups` |
| WIPIC `0x4b6` · `0x4c0` · `0x4ce` | stubs returning 0 (`unk15` · `unk5` · `unk10`) | `media::clip_clear_data` · `set_volume` · `get_default_volume` |
| WIPIC `0x7d0` | same as `Socket` (`net::socket`, fake server aware) | `unk_socket` stub −1 |
| `MC_knlGetTotalMemory/GetFreeMemory` | fixed 4 MiB (LGT-local) | shared kernel answer |
| graphics SVCs | shared `wie_wipi_c::api::graphics` records + local wrappers (soft-key rows, flush latch, RGB565 + 8bpp mask images, native pixel-op, native fg/bg/alpha overlay) | LGT-native module (`LgtFramebuffer`/`LgtGraphicsContext` records) |
| exception frames | per-thread ledger, catch consumes frame, pop at the consumed sp is a no-op, mark/release, host-error state | per-task state, pop always pops |
| array class token handed to guest (`get_string_array_class`, `get_array_class`) and taken by `instantiate_array`/`multi_array` | `java/lang/Class` object pointer | native class-name pointer (#1606) |
| Java import `0x0a` / `0x64` | `0x0a` → GetInterfaceDispatchTable, `0x64` → GetInterfaceMethodTable | `0x64` → GetInterfaceDispatchTable, `0x0a` unmapped |
| JavaException out of a Java-system SVC | unwind from the registers as they are | caller context restored first |
| instance layout of Thread / TimerTask | `host_field` storage outside the instance | `instance_field_word_count` 4 / 6 + negative-tag runtime fields |
| vtable placement | parent's appended slots inherited, unplaced methods reserve a slot | fixed slots first, inherited runtime-only methods re-appended |
| `lgt_java_abi.toml` `java/lang/Class` fields | `classLoader` word 1, `nameBytes` word 3 | `nameBytes` 1, `classLoader` 3 |
| `lgt_java_abi.toml` `java/lang/Runnable.run()V` | index 0 | index 10 |
| stdlib `malloc` 0x426 | NULL on failure; 4-byte header | `Err` (run ends); 8-aligned |
| stdlib `strncmp` 0x40a | −1 / 0 / 1 | byte difference |
| stdlib `strtol` 0x3ff | `parse_long` (saturates to i32) | own clamp, `*end` written first |
| error text | one register dump, at the tick | dump appended at the failing call |
| ★taken from upstream — stdlib `0x424` | — | `atexit` stub: callback not run (ours/base ran it at registration) |
| ★taken from upstream — `vsprintf` arg reads | — | lazy; an unmapped consumed word is an error |
| ★taken from upstream — 21 WIPIC ids, 6 stdlib slots, 16 ABI rows | were fatal «Unknown …» / missing-vtable-entry | handled |
| ★taken from upstream — `handle_wipic_svc` native frame | — | Java temporaries of a WIPIC call are released when it returns (#1542) |

###### ⑶ Silently-merged files checked (both sides changed, no conflict)
| file | merged result | action |
|---|---|---|
| `Cargo.toml` | identical to ours (both moved chrono → jiff) | none |
| `runtime/java.rs` | upstream `pub mod exception` | back to `mod exception` (nothing outside uses it now) |
| `runtime/java/abi.rs` | + `instance_field_word_count` field, read by nothing here | = ours |
| `classes/net/wie/clet_wrapper.rs` | upstream `context.call_function(jvm, …)` | kept (same error mapping; dump is identity) |
| `jvm_support/array_class_definition.rs` | identical to ours | none |
| `jvm_support/array_class_instance.rs` | base test `storage_layout_matches_lgt_abi` removed by upstream's purge | restored |
| `jvm_support/jvm_implementation.rs` | our `exception::init(core)?` removed | restored |
| `tests/test_helloworld.rs` | upstream's loop over `aot: None` and `TestNativeExecutor` + our `…_jar_under_p_prefix` | kept; passes both ways (the AOT leg runs after our `install_title_binary_patches`) |
Upstream-only files that came in: `runtime.rs` (see ⑴), `runtime/globals.rs` (kept; + our `TIME_VALUE_PTR 0x7fff1004`,
disjoint from wie-wipi-c's `0x7fff1000/1020/1024`), `classes/net/wie.rs` (`CletWrapperContext::call_function`, kept),
`tests/data/helloworld_lgt.zip` (upstream's rebuilt fixture, 2187 → 2157 bytes; `test_helloworld` ×3 pass on it; the
runner's `test_data/helloworld_lgt.zip` is a different file and untouched).
`runtime/init.rs`: `core.prepare_execution().await?` (upstream) sits right after `load_executable`, which is where our
`install_title_binary_patches` runs — patches first, then AOT preparation, as the core-arm notes require.

###### ⑷ Requests outside the slice
None blocking. For the parent's records:
1. `LgtEmulator`'s public API is unchanged (`from_archive`, `from_jar(platform, jar_filename, jar, pid, aid, main_class, options)`,
   `archive_*`, `loadable_*`); hosts only need `Options { aot, .. }` and `Emulator::is_preparing`.
2. `svc_ids.rs` cites «the round record of ticket `wie-upstream-sync2-s3-merge-upstream-main` (`docs/report/`)» for the
   id divergence list — the report should carry the ⑵ table (or edit that comment to the serial).
3. `cargo clippy -p wie-lgt` had to run with `--no-deps`: at that moment `wie-midp` (another slice) failed clippy
   (`too_many_arguments` in `lcdui/image.rs:292`).
4. S4/S5 material, not done: upstream's LGT-native graphics rewrite, `#1606` array-type tokens, `#1580` runtime-field
   layout and `api::fs` are each a measured switch (before/after on the 54+ LGT rows), not a merge resolution.

###### ⑸ Commands (final tree)
- `/usr/bin/grep -rn -E '^(<<<<<<<|\|\|\|\|\|\|\||=======|>>>>>>>)' wie-lgt` → 0 lines.
- `build-slot run -- cargo check -p wie-lgt --all-targets` → rc=0, 0 warnings.
- `RUST_MIN_STACK=4194304 build-slot run -- cargo test -p wie-lgt` → rc=0: lib **101 passed** (origin/main had 95;
  +6 = svc table 1, stdlib 4, wipi_c 1) · `test_emulator_freed` 1 · `test_helloworld` 3 · `test_key_reach` 1 ·
  `test_resource_reach` 1 · 0 failed. load1 ≈ 10.
- `build-slot run -- cargo clippy -p wie-lgt --all-targets --no-deps -- -D warnings` → rc=0 (fixed: 2 `clone_on_copy`
  on the now-`Copy` `ArmCoreContext` in `method.rs` tests, 1 unused `mut`).
- `cargo fmt -p wie-lgt -- --check` → rc=0.
- Not run: wasm clippy, `wie_validate` on the LGT fixtures/corpus (needs the whole workspace).
- Upstream tests not carried (11 of its 22 lib tests; each asserts the upstream column of ⑵):
  `exception_frames_are_restored_per_thread`, `callback_fault_diagnostics_survive_java_exception_restoration`,
  `array_type_passes_array_creation_and_cast_check`, `allocation_imports_resize_…`, `bounded_string_imports_…`,
  `strtol_import_…`, `memory_and_string_imports_…` (its memcmp/memchr half is carried),
  `context_queries_write_values_and_exclusive_clip_bounds`, `destroying_an_image_releases_its_guest_allocations`,
  `screen_geometry_is_stable_before_and_after_raw_pointer_access` (the last three belong to the unwired graphics rewrite);
  `resource_calls_release_java_temporaries` IS carried and passes.


#### 부록 B — `wie-midp` 묶음

##### wie-midp slice — notes (fork s3-midp · 2026-11-23)

Scope: 29 unmerged files under `wie-midp/` + fallout in `wie-midp/` and `wie-midp-vendor/`. Nothing outside was edited. No git writes.
Classification: `/tmp/wie-s3/class/wie-midp.tsv` — 발신 20 · 폐기 7 · 재적용 2 = 29 (해소: 3way-edit 12 · theirs 7 · ours 7 · union 3).
Rule applied: where a guest can observe a difference, ours stays — including places where ours is the merge-base behaviour and only upstream moved (§1-A).
Upstream was taken where it adds surface, where ours was a marked stub or a crash, and for invalid-argument error paths (§1-B).

###### ⑴ Behaviour divergences

####### 1-A. Kept ours (자리 | 우리(유지) | upstream)

| 자리 | 우리(유지) | upstream |
|---|---|---|
| `Canvas.getKeyCode(GAME_A..D)` (0551 §8) | 7 · 9 · * · # | 1 · 3 · 7 · 9 |
| `Canvas.getGameAction(number key)` | 0 — only d-pad and FIRE map (as merge base) | KEY_NUM1/3/7/9 → GAME_A..D |
| `GameCanvas.getKeyStates` key → bit | own table: 2/4/6/8 steer, 5 = FIRE, GAME_A..D = 7 9 * #; no guest call | `1 << getGameAction(code)` (virtual call into the guest; 1/3/7/9; number keys do not steer) |
| `Canvas.getKeyName` | "Up" "Down" "Left" "Right" "Select" "*" "#" digits; anything else IllegalArgumentException; aware of J2ME standard codes | "UP" … "SELECT" "SOFT1" "SOFT2" "CLEAR" "SEND" "END" "VOLUME UP/DOWN"; other keys = their ASCII char |
| `Canvas.repaint(x,y,w,h)` with w ≤ 0 or h ≤ 0 | request kept — Display paints the whole content area (merge base) | dropped, nothing painted |
| `Display` command band on an SKVM Canvas | 18 px (Canvas with commands reports 206 high at 240) | 16 px (#1581 · 208) |
| `Display.setCurrent` callback order | old.hideNotify (old still current and attached) → detach → switch → attach → new.showNotify → sizeChanged; an exception from hideNotify aborts the switch | switch first, then hideNotify; sizeChanged before showNotify; callback errors deferred until the switch completed |
| `Display.repaint` dirty area · `serviceRepaints` inside paint · paint serialization · GC spacing | ours (0551 #57): dirtyLeft..Bottom union, paintingThread, hold of other guest threads, 1 s / 20× cost GC spacing | repaintLeft..Bottom clamped to the viewport, `painting` flag, GC every paint |
| `EventQueue` RepaintEvent dispatch | `Display.handlePaintEvent` — every delivered `Event::Redraw` paints (merge base; hosts answer `request_redraw` with it, Clet screens are presented from it) | `Display.serviceRepaints` — paints only when a repaint is pending |
| `GameCanvas.flushGraphics()` · `(IIII)` | repaint request only (merge base for `()`) | repaint + `serviceRepaints` (synchronous) |
| `Image.createImage(w,h)` initial pixels | transparent black (0) (merge base) | opaque white (255) ★ upstream's tests assume it — see ⑷ «upstream tests changed» |
| `new Image()` | 1×1 image | object without pixel data (0×0) |
| `Image.getGraphics()` on an immutable image | returns a Graphics (merge base) | IllegalStateException |
| `Image.createImage(String)` · `(InputStream)` with undecodable data | IllegalArgumentException (through the byte-array overload; merge base for `(String)`) | IOException |
| `Font.getBaselinePosition` | height × 4 / 5 (9 at 12) | 10 |
| `Font.getFace/getStyle/getSize/isPlain/isBold/isItalic/isUnderlined` | what `getFont(face, style, size)` was asked for | constants (0, 0, 0, true, false, false, false) |
| `Graphics.getGrayScale` | luma (76R + 150G + 29B) >> 8 | (R + G + B) / 3 |
| `Graphics.setStrokeStyle` / `getStrokeStyle` | stored; IllegalArgumentException on an invalid style | stub — ignored, always SOLID |
| `Graphics.fillTriangle` | fills | stub — draws nothing |
| `Graphics.drawRegion` · `Image.createImage(Image,x,y,w,h,transform)` | our `draw_transformed` / `transformed_region` | `Image::transform_region` copy (same pixels wherever upstream's test looks) |
| `Connector.open` (3 overloads) | IOException "no network" | ConnectionNotFoundException "Unsupported protocol" |
| `Manager.createPlayer(String)` | "device://tone" → a player; null → IllegalArgumentException; else MediaException | always MediaException |
| `Manager.playTone` | plays; IllegalArgumentException on a bad note/duration | stub |
| `RecordStore.getRecord(id)` of an empty record | empty byte[] (merge base) | null |
| `RecordStore.openRecordStore(name, false)` missing store | RecordStoreNotFoundException, message = the store name | message "Record store not found" |
| `RecordEnumeration` order without a comparator | ascending record id | repository order |
| `List` · `TextField` | full implementations (22 and 9 methods) | constructor stubs |
| lcdui.game `Layer` · `LayerManager` · `Sprite` · `TiledLayer` | our implementation (kept as one set — `Layer.<init>` signatures differ) | upstream's implementation of the same public methods |

####### 1-B. Upstream taken although ours had something there (exceptions to «keep ours», each with its reason)

| 자리 | 우리(버림) | upstream(채택) | 왜 |
|---|---|---|---|
| `RecordEnumeration` implementation | `net/wie/RecordEnumerationImpl`: id snapshot; `keepUpdated` ignored, `rebuild`/`destroy` no-ops, `isKeptUpdated` false | `net/wie/RecordEnumeration`: keepUpdated tracking, rebuild, destroy (then IllegalStateException) | ours were marked stubs (`ponytail:` comment naming this upgrade); traversal model is the same — our test `enumerate_records_filters_orders_and_walks_both_ways` passes unchanged |
| GameCanvas key state bookkeeping | bits in GameCanvas (`keysHeld`/`keysLatched`), never reset | bits in Displayable (`heldGameKeys`/`pressedGameKeys`), reset when the canvas is shown again and when the command menu opens; `getKeyStates` = 0 while not shown | ours left a key «held» forever when its release went to another screen; the key → bit table is still ours (1-A) |
| GameCanvas off-screen buffer size | `Displayable.getWidth/Height` (merge base) | full screen | upstream fix for drawing cut off when the reported area is smaller |
| `Image.getRGB` · `createRGBImage` argument checks | IllegalArgumentException for a too-small array; no scanlength check; NPE before size check | spec: `|scanlength| < width` → IAE, too-small array → ArrayIndexOutOfBoundsException, null array allowed when nothing is read, negative scanlength works | invalid-argument paths only; results for valid calls are the same |
| `Graphics.drawRGB` | merge base (`TODO proper scanlength support`) | scanlength honoured, bounds checked | ours untouched; upstream completes it |
| `RecordStore.addRecord/setRecord(null, 0, 0)` | null array dereferenced | empty record | ours failed |
| `Display.setCurrent` re-entrancy | hideNotify calling setCurrent recursed until the host stack overflowed (reproduced by upstream's test) | — | ★ours fixed with a small guard (`hidingDisplayable` field + upstream's `is_current`): the callback is told once and a switch it makes supersedes the outer one. Ordering stays ours (1-A) |

New surface from upstream (no ours counterpart): `Connection`, `ConnectionNotFoundException`, `Connector.open*Stream` ×4, `Manager.getSupportedProtocols/getSupportedContentTypes`, `Font.getFont(I)`, `Display.isColor/flashBacklight/numAlphaLevels/numColors/getColor/getBorderStyle/getBestImageWidth/getBestImageHeight/setCurrentItem`, `RecordComparator.PRECEDES/EQUIVALENT/FOLLOWS`, `Event::Repaint` → RepaintEvent(1), NotifyEvent dispatch, `Canvas.handleKeyEvent` now PROTECTED, static `Display.screenImage/screenGraphics` + `<clinit>` (#1582 · internal).

###### ⑵ Silent merges

**14 non-conflicted files changed on both sides** (`javax/microedition.rs`, `lcdui.rs`, `lcdui/{alert,choice_group,command,form,gauge,image_item,item,string_item}.rs`, `rms/record_store_not_found_exception.rs`, `net/wie/{choice_element,command_event,item_state_event}.rs`): compared function by function against both sides (`/tmp/wie-s3/midp/fncmp.py`). Every function equals upstream's, and ours differed from the base only by the owner-arg rewrite. No duplicate `mod`/`pub use` line. Nothing to fix.

**Auto-merged regions INSIDE conflicted files were wrong in 8 places** — none of these was inside a conflict hunk, all would have compiled or nearly so:

| 파일 | git 이 조용히 만든 것 | 처분 |
|---|---|---|
| `lcdui/graphics.rs` `draw_region` | ★upstream's `Image::transform_region` copy AND our `draw_transformed` both applied — a region transformed twice. Compiles. | function replaced by ours |
| `lcdui/canvas.rs` `get_game_action` | our standard-key translation + upstream's KEY_NUM1/3/7/9 → GAME_A..D rows | rows removed (ours) |
| `lcdui/canvas.rs` | upstream's `get_key_code` beside ours (duplicate fn + duplicate proto), `showNotify`/`hideNotify` registered twice | upstream copies removed |
| `lcdui/display.rs` | duplicated `as_proto`/`init`/`handle_paint_event` fragments, both dirty-rect designs interleaved | file rebuilt from ours + the upstream pieces listed in the tsv |
| `lcdui/image.rs` | `createImage(w,h)` white fill, `getGraphics` ISE, `createImage(String)` → stream overload, second `init_empty` + proto | reverted to ours (1-A) / duplicate removed |
| `lcdui/game/game_canvas.rs` | two `get_key_states`, two `flushGraphics(IIII)` protos | upstream file + our flush behaviour |
| `lcdui/font.rs` | every `getFace…isUnderlined` twice | rebuilt from ours + `getFont(I)` |
| `rms/record_store.rs` | two `enumerate_records` + the proto twice | rebuilt from upstream + our deltas |

**One semantic break with no git conflict at all — fixed in `wie-midp-vendor`**: upstream's new `com.nokia.mid.ui.FullCanvas` translates key codes to Nokia's -1..-11, and wie-j2me (which now registers that crate) also turns on our `Canvas.standardKeyCodes`, which hands out the same codes. Both applied: UP became -1, was then read as the shared HANGUP code and delivered as -11; `getKeyCode(UP)` returned -11 too. `full_canvas.rs` now skips its table when `standardKeyCodes` is on (+ assertions in upstream's test: DOWN → -2, UP → -1, `getKeyCode(1)` → -1).

`lib.rs` `get_protos()`: 60 entries, 0 duplicates (checked by counting `::as_proto()` paths).

###### ⑶ Needed outside this slice (not done here)

1. **`wie-backend/src/canvas/lbmp.rs` is currently ours.** Upstream's midp test `lbmp_images_preserve_grayscale_and_transparency_when_drawn` also checks a mask plane on LBMP types 8 and 16, which our decoder does not read (type 2 only). I made the test skip `masked != 0 && kind != 2` with a `// fork:` comment. If `lbmp.rs` goes to upstream's (0551 #14 said 폐기 · theirs), delete those 3 lines in `wie-midp/src/classes/javax/microedition/lcdui/image.rs`.
2. **`Display.screenImage` / `screenGraphics` are static now, `getScreenGraphics` is a static method.** Callers must use `invoke_static` / `get_static_field`. `wie-skvm` already does (checked by grep at 21:00). wie-wipi-java · wie-ktf · wie-lgt had no caller at that time.
3. `RecordStore` calls `DatabaseRepository::create` (upstream's trait, already in the auto-merged `wie-backend/src/database.rs`) — hosts need it (`wie_cli`, `wie_featurephone`, `test-utils` compile today as far as `wie-midp`'s tests need).
4. `wie-midp/src/classes/javax/microedition/media/control_interfaces.rs` is deleted and `media/control/tone_control.rs` is new (untracked) — stage both. `wie-midp-vendor/.../full_canvas.rs` and `wie-midp/src/classes/net/wie/record_enumeration.rs` are upstream-added files I edited.
5. `Display` is a class with `<clinit>` now: the screen image is created at class initialization from `platform.screen()`, not per `new Display()`. A host that resizes the screen after the class initialized keeps the old image size (ours created it per instance). Not exercised by `wie-midp` tests; Scenario G/G2 of `contract-roundtrip.mjs` is the check.
6. Convergence candidates for a measured follow-up (each is a 1-A row whose only cost is a permanent conflict): the lcdui.game package + `drawRegion` (upstream's test passes on ours; ours were not run on upstream's), `createImage(w,h)` white fill (upstream tests keep assuming it), `EventQueue` → `serviceRepaints`, `flushGraphics` + `serviceRepaints`, SKVM softkey 16.

###### ⑷ Final command output (2026-11-23 21:03 · load1 7.5)

| 명령 | 결과 |
|---|---|
| `/usr/bin/grep -rn -E '^(<<<<<<<\|\|\|\|\|\|\|\|\|=======\|>>>>>>>)' wie-midp wie-midp-vendor` | 0 lines |
| `cargo check -p wie-midp -p wie-midp-vendor --all-targets` | rc=0 · lib 0 warnings · lib test 6 warnings (below) |
| `RUST_MIN_STACK=4194304 cargo test -p wie-midp -p wie-midp-vendor` | rc=0 · wie-midp **96 passed / 0 failed** · wie-midp-vendor **1 passed / 0 failed** |
| `cargo clippy -p wie-midp -p wie-midp-vendor -- -D warnings` (what the gate lints) | rc=0 |
| `cargo clippy -p wie-midp -p wie-midp-vendor --target wasm32-unknown-unknown -- -D warnings` | rc=0 |
| `cargo clippy -p wie-midp -p wie-midp-vendor --all-targets -- -D warnings` | **rc=101** — 6 lints, all in OUR test code and identical on `origin/main` (grep-confirmed): `midlet.rs` unused `Arc`, `AtomicBool`/`Ordering`; `display.rs` deprecated `fetch_update`; `image.rs` unneeded `mut`; `smaf_player.rs` 2× unreachable `else`. Left alone (not the merge's). |
| `cargo fmt -p wie-midp -p wie-midp-vendor -- --check` | rc=0 |

Tests: origin/main 90 · upstream 47 · merged 96 (+1 vendor). Lost from ours: 0 (`delete_record_removes_record_and_rejects_unknown_id` is the base test upstream renamed to `empty_records_can_be_updated_and_deleted`). Lost from upstream: 1, deliberately — `image_can_be_constructed_without_pixel_data` asserts 0×0 for `new Image()`; ours `no_arg_image_is_one_pixel` asserts the kept 1×1.

**Upstream tests changed to match kept-ours behaviour** (each edit carries a `// fork:` comment):
- `canvas.rs` `canvas_and_game_canvas_fullscreen_preserve_paint_and_keys`: getKeyCode table 7/9/*/# and `getGameAction` 0 for them; command band 18 on SKVM; paint count +1 after non-positive repaints; GAME_A/B pressed as KEY_NUM7/KEY_NUM9. Its dirty-clip expectations (`[10,5,55,55]`, `[0,0,5,6]`, `[4,5,6,7]`) and both nested-`setCurrent` cases pass on our Display unchanged.
- `graphics.rs`: 6 tests are ours (= merge base) versions, not upstream's white-background rewrites; `test_draw_region_applies_transform_and_anchor` background black.
- `image.rs` `rgb_images_copy_pixels_and_validate_reads`: blank image `[0,0]`, the getGraphics-ISE block removed, undecodable stream = IllegalArgumentException; LBMP test see ⑶-1.
- `rms/record_store.rs` `empty_records_can_be_updated_and_deleted`: empty record = empty array.
- `net/wie/event_queue.rs`: test module is ours (upstream's version of `input_precedes_recurring_callbacks_and_pending_paint` expects the serviceRepaints dispatch).
- upstream `sprite.rs` test `sprites_and_tiled_layers_paint_and_collide` → `lcdui/game.rs` `mod upstream_test`, background constant black. Passes on our game package.


#### 부록 B — `wie-skvm` 묶음

##### S3 slice notes — wie-skvm · wie-skt · wie-j2me

Slice: 14 unmerged paths (wie-skvm 10 · wie-skt 2 · wie-j2me 2; the two `Cargo.toml` were resolved by the parent).
갈래: 폐기 4 · 재적용 3 · 발신 7. 해소: theirs 4 · ours 2 · union 2 · 3way-edit 6. Rows: `/tmp/wie-s3/class/wie-skvm.tsv`.

###### ⑴ 동작이 갈리는 자리

####### ⑴-A 양쪽이 다 고친 자리 — 우리 값 유지

| 자리 | 우리(유지) | upstream |
|---|---|---|
| SKT 시스템 속성 `m.MIN` | `01000000000` | `00000000` |
| SKT 시스템 속성 `m.MODEL` | `wie` | `0` |
| SKT 시스템 속성 `m.EXT_SW` | `0` | 없음(null) |
| J2ME 시스템 속성 | `microedition.platform` + configuration · profiles · locale · encoding · `file.separator` | `microedition.platform` 하나 |
| J2ME `Canvas.standardKeyCodes` | J2ME 에뮬레이터가 `true` 로 켠다(-1..-7) | 그 정적 필드 없음 |
| `XFile.fsavail()` | 상수 1,000,000 | 1 MiB − 파일시스템 사용량 |
| `XFile(jarfile, name)` | 꾸러미 옆 jar 의 엔트리를 연다 | `IOException`(미지원) |
| `XFile` READ_RESOURCE 의 `buf`/`offset` | 엔트리를 `buf` 에 실어 보인다(게임 쪽 `XResource` 가 직접 읽는다) | `is` 스트림만 |
| `Graphics2D.drawImage` / `XDisplay.drawImageEx` 의 일하는 쪽 | Graphics2D 가 그리고 XDisplay 가 넘긴다 | XDisplay 가 그리고 Graphics2D 가 넘긴다 |
| `XDisplay.drawImageEx` 의 둘째 `Image` | 무시(그래픽스의 대상에 그린다) | null 이 아니면 그 이미지에 그린다 |
| `Graphics2D.drawImage` DRAW_AND / DRAW_OR 를 «자기 자신 위에» 겹쳐 그릴 때 | 앞에서 뒤로 섞는다(겹친 칸이 번진다) | 겹침 방향을 피해 훑는다(번지지 않는다) |
| `XDisplay.drawImageEx` 의 폭·높이 ≤ 0 | 음수면 `IllegalArgumentException`, 0 이면 무동작 | 둘 다 조용히 무동작 |
| `XDisplay.copyLCD` | `captureLCD` 결과를 이미지 (0,0) 에 그린다 | 스텁(무동작) |
| `XDisplay.clear` | 있다(기록만 하는 무동작) | 없다(`NoSuchMethodError`) |
| `TextComponentHandler.keyPressed` | 멀티탭 입력기(영문 대문자·숫자) · 등록된 컴포넌트가 없으면 포커스된 `XTextField` 로 | 숫자·CLEAR 만 그대로 넣는다 · 등록이 없으면 `false` |
| `TextComponentHandler` UNEDITABLE(0x20000) 필드 | 입력한다 | 삼킨다(`true` · 글자 안 들어감) |
| `TextComponentHandler.isLoaded` / `getInputMode` / `keyRepeated` | 있다(`false` / 0 / 키 처리) | 없다 |
| `TextComponentHandler` 인스턴스 | 그 클래스 자신(구체 클래스) | 추상 클래스 + `net/wie/WieTextComponentHandler` |
| `WieAudioClip.open` 의 SMAF 아닌 데이터 | 오류 로그만 · 예외 없음 | `UnsupportedFormatException` |
| `WieAudioClip.open` 의 로드 실패 | 오류 로그만 | `ResourceAllocException` |
| `WieAudioClip.play` | 소리 스레드에서만 끝날 때까지 기다린다 · 울리는 중이면 다시 시작하지 않는다 | 항상 끝날 때까지 막는다 |
| `WieAudioClip.loop` | 이미 도는 중이면 무동작 · 닫힌 클립이 남긴 루프를 멈춘다 | 매번 stop 뒤 다시 시작 · 호출이 돌아오지 않는다(wait 0) |
| `WieAudioClip.close` | 소리를 멈추지 않는다(클립만 은퇴) | 재생 중이면 `IOException` · 아니면 핸들 해제 |
| `WieAudioClip.stop` / `pause` (닫힌 클립) | 무시 | stop 은 재생 중일 때만 동작 · pause/resume 은 스텁 |
| J2ME 저장 id(`System` aid·pid) — ★저장 키 | bare jar = jar 파일 이름 · JAD+jar = `MIDlet-Name` (병합 전 그대로) | 둘 다 `MIDlet-1` 의 첫 칸(앱 제목) |

마지막 행은 우리 쪽이 고친 줄이 아니다(base = ours). upstream 이 저장 키를 옮긴 것이라 «파일시스템 이름공간 — 저장 키 불변» 판정(0551 §8 여섯째)과 같은 규칙으로 우리 값을 지켰다. `load()` 에 `id` 인자를 되살렸고 upstream 시험 `jar_and_jad_use_midlet_name_for_storage` 는 `jar_and_jad_keep_their_storage_ids` 로 단언을 우리 값에 맞췄다.

####### ⑴-B upstream 만 고친 자리(우리 = base) — 병합으로 그대로 들어온다. 총괄이 볼 것

| 자리 | 병합 전(우리 = base) | 병합 뒤(upstream) | 처분 |
|---|---|---|---|
| `SktEmulator::loadable_jar` | `\x20\0…` 머리의 jar 만 SKT | + `.class` 가 `com/skt/m/` · `com/xce/` 를 참조하는 plain jar 도 SKT | 받았다. `wie_identify` 의 통신사 판정이 이것을 부른다 — `scripts/content-key.mjs check` 로 확인할 것 |
| `SktEmulator::from_jar` | `main_class_name` 인자 · 호스트가 `None` 을 넘겨 늘 «Main class not found» | 매니페스트(`MIDlet-1`)에서 읽는다 · 인자 4개 | 받았다(호출부 2곳 — ⑶) |
| `AudioSystem.getAudioClip(format)` | 형식 무관하게 클립 | `"mmf"` 가 아니면 `UnsupportedFormatException` | 받았다(우리 실측 5판은 전부 `"mmf"`) |
| `AudioSystem.getClipFormats()` | 빈 배열 | `["mmf"]` | 받았다 |
| `Device.invokeWAPBrowser(url)` | 스텁 | `destroyApp(true)` 뒤 `platform().exit()` — 앱이 끝난다 | 받았다(파일은 upstream 것 그대로) — 부르는 판이 있으면 종료한다 |
| `XFile.fsused()` | 0 | 파일시스템 사용량 | 받았다 |
| `XFile` READ_RESOURCE 조회 | `getClass().getResourceAsStream` | 시스템 클래스 로더(같은 로더에 위임 — 결과 같음) | 받았다 |
| `Toolkit.<clinit>` 의 `graphics` | 현재 MIDlet 이 있을 때만 | 항상(정적 `Display.getScreenGraphics()`) | 받았다(wie-midp 의 Display 가 정적 화면으로 갔다) |
| `XDisplay.refresh` | `screen.request_redraw()` | `Display.screenImage` 를 그 자리에서 `screen.paint` | ★**받지 않았다** — 우리 Display 의 그리기 길(pacing `paint_started` · 합성기)을 건너뛴다. 자리별 전/후를 재는 별 티켓 감 |
| J2ME `jar_metadata` 의 제목 · `J2MEDescriptor.name` | `MIDlet-Name` | `MIDlet-1` 의 첫 칸 | 받았다(표시용 제목 · 저장 id 는 ⑴-A) |
| J2ME 매니페스트 조회 | 정확히 `META-INF/MANIFEST.MF` · 부팅 뒤 클래스 로더로 | 대소문자 무시 · 로드 때 읽고 없으면 오류 | 받았다 |

###### ⑵ 조용히 합쳐진 파일

`comm -12` of `git diff --name-only 44fbf265 origin/main -- <crate>` and `… upstream/main` minus the unmerged list: wie-skvm 9 · wie-skt 0 · wie-j2me 0.

| 파일 | 본 것 |
|---|---|
| `wie-skvm/Cargo.toml` | 병합 결과 = upstream + 우리 `wie-util`. `bytemuck` · `encoding_rs` 가 있어 upstream `ByteToChar*` 가 선다 |
| `com/skt/m/audio_system.rs` | 양쪽 변화가 다른 줄 — 우리 `MAX_VOLUME`·master volume + upstream `"mmf"` 검사·`getClipFormats`(⑴-B). import 중복 없음 |
| `com/skt/m/progress_bar.rs` · `sms_message.rs` · `com/skt/m3d/object_3d.rs` · `com/xce/io/file_input_stream.rs` · `file_output_stream.rs` | 병합 결과가 ours 와도 theirs 와도 바이트 동일(양쪽이 같은 owner 인자 수정) |
| `com/xce/io.rs` · `com/xce/lcdui.rs` | 양쪽이 같은 `mod`/`pub use` 를 더했다 — 중복 없음 |
| (충돌 파일 안) `com/xce/lcdui/x_display.rs` 의 덩어리 «밖» | ★자동 병합이 `drawImageEx` proto 와 `draw_image_ex` 함수를 **둘씩** 남겼고(컴파일 오류) `refresh` 를 upstream 것으로 바꿔 놓았다. 머리 전체를 우리 판 + upstream 상수 4개로 다시 짰다 |
| (충돌 파일 안) `com/xce/io/x_file.rs` 의 시험 모듈 | 자동 병합이 두 시험을 섞어 놓았다(닫히지 않는 블록). upstream 시험 모듈 + 우리 시험 3개로 다시 짰다 |
| upstream 만 고친 것: `com/skt/m/device.rs` · `com/xce/lcdui/toolkit.rs` · `net/wie.rs` · 새 `net/wie/wie_text_component_handler.rs` · `tests/data/resource.jar` | ⑴-B. `WieTextComponentHandler` 는 등록만 되고 우리 `getTextComponentHandler` 는 쓰지 않는다 |

###### ⑶ 내 몫 밖에서 바꿔야 하는 것

1. `SktEmulator::from_jar` 가 인자 4개가 됐다(upstream). 다섯째 `None` 을 지울 곳: `wie_featurephone/src/lib.rs:642` · `wie_cli/src/bin/wie_validate.rs:1984`.
2. `wie-skvm` 의 upstream 코드가 기대는 다른 크레이트 API: `wie_backend` `Filesystem::usage()`(fsused) · `Platform::exit()`(Device) · `test_utils::TestPlatform::with_event_handler` + `TestPlatformEvent`(device.rs 시험). `TestPlatform::with_paint_handler` 는 쓰지 않게 고쳤다(x_display 시험).
3. `wie_identify` · `wie_featurephone` · `wie_validate` 의 통신사 판정이 넓어진 `loadable_jar` 를 탄다(⑴-B 첫 행).
4. ★`smaf` 0.1.2 → 0.1.4(`wie-backend/Cargo.toml` · upstream 값)는 **`MMMD` 머리의 선언 길이를 지킨다**(0.1.2 는 무시했다). 스크래치 탐침으로 확인: 길이 칸이 0 인 우리 시험용 SMAF 가 0.1.2 에서는 사건 5개, 0.1.4 에서는 `ParseError(Eof)` · 사건 0. 길이 칸을 맞추면 두 판 결과가 같다. `wie_audio_clip.rs` 시험 픽스처의 길이 칸을 고쳤다(그 형태를 쓰는 곳은 그 파일 하나). `wie-backend/src/system/audio.rs` 의 `parse_smaf_in` 주석 «a header that misstates its length (0 in a committed test) still parses whole» 은 이제 사실이 아니다 — 머리가 길이를 «짧게» 적은 실제 파일은 0.1.4 에서 소리가 안 난다(코퍼스에서 세지 않았다 · census 소리 축이 본다).
5. `wie-midp` 의 `Image.createImage(w, h)` 바탕색이 우리(검정)와 upstream(흰색)에서 다르다. 병합 트리는 검정이다(midp 몫). skvm 의 upstream 시험 한 개가 흰색을 단언해서 검정으로 맞췄다(`graphics_2d_raster_modes_…`).
6. S4 후보(이 조각에서 하지 않았다): `Graphics2D.drawImage` ↔ `XDisplay.drawImageEx` 를 upstream 방향으로 뒤집으면 두 파일이 거의 upstream 과 같아진다 — 남는 차이는 «둘째 이미지»와 폭·높이 ≤ 0 두 가지다.

###### ⑷ 명령 출력

병합 트리(작업 트리 · 커밋 전) · 2026-11-23 · 무거운 명령은 `build-slot run --` 경유.

| 명령 | 결과 |
|---|---|
| `/usr/bin/grep -rn -E '^(<<<<<<<\|\|\|\|\|\|\|\||=======\|>>>>>>>)' wie-skvm wie-skt wie-j2me` | 0줄(rc=1) |
| `cargo check -p wie-skvm -p wie-skt -p wie-j2me --all-targets` | rc=0 · 경고 0 |
| `RUST_MIN_STACK=4194304 cargo test -p wie-skvm -p wie-skt -p wie-j2me` | rc=0 · wie-skvm **58** passed · wie-skt **10** passed · wie-j2me lib 4 + `test_boot` 2 + `test_emulator_freed` 1 + `test_pacing` 2 = **9** passed · 실패 0 |
| `wie-j2me/tests/test_pacing.rs` | 2 passed — 수치 단언은 그 시험 안에 있고 고치지 않았다 |
| `cargo clippy -p wie-skvm -p wie-skt -p wie-j2me --all-targets -- -D warnings` | rc=0 |
| `cargo fmt -p wie-skvm -p wie-skt -p wie-j2me -- --check` | rc=0 |

- `git add` 는 하지 않았다(전부 작업 트리에만 있다). 내가 쓴 파일: `wie-j2me/src/emulator.rs` · `wie-skt/src/emulator.rs` · `wie-skvm/src/lib.rs` · `wie-skvm/src/classes/com/skt/m/graphics_2d.rs` · `com/xce/io/{byte_to_char_converter,byte_to_char_euc_kr,x_file}.rs` · `com/xce/lcdui/{text_component,text_component_handler,x_display,x_text_field}.rs` · `net/wie/wie_audio_clip.rs`.
- 시험을 고친 곳(동작은 안 고쳤다):
  - upstream 시험을 우리 값에 맞춘 것 5: j2me `jar_and_jad_keep_their_storage_ids`(저장 id · 합성 jar 에 엔트리 하나 추가 — 우리 `extract_zip` 이 모든 엔트리를 감싼 디렉터리를 벗긴다) · x_display `image_regions_…`(둘째 이미지 경우와 자기 겹침 AND/OR 를 뺐다) · x_display `toolkit_graphics_…`(refresh 의 «그 자리에서 그림» 단언을 뺐다) · x_text_field 흐름 시험(UNEDITABLE 단언 `"1"` → `"1W"` · 마지막 `repaintCount == 4` 를 뺐다) · graphics_2d `…raster_modes…`(바탕 흰색 → 검정).
  - 우리 시험을 병합된 API 에 맞춘 것 2: `wie_audio_clip` 픽스처의 SMAF 길이 칸 · `getClipFormats` 길이 0 → 1 과 형식 이름 `"audio/mpeg"` → `"mmf"`(⑴-B) · graphics_2d `capture_lcd` 시험의 `getScreenGraphics` 를 정적 호출로.
  - 가져오지 않은 upstream 시험 1: `wie_audio_clip` 의 `audio_clip_blocks_until_media_end_and_preserves_stopped_playback_across_replay`(upstream 의 «항상 막는 play» 를 단언한다 — 우리 동작과 반대).
- 못 한 것: wasm32 타깃 clippy · `wie_cli`/`wie_featurephone` 컴파일(⑶-1 의 호출부 2곳이 아직 인자 5개다) · 코퍼스 실행(SKT 82행) — 전부 내 몫 밖이다.


#### 부록 B — `wie-wipi-java` 묶음

##### wie-wipi-java — S3 병합 조각 노트 (fork `s3-wipi-java`)

범위: `wie-wipi-java/` 의 미해결 29파일(UU 18 · AA 11) + 같은 크레이트의 조용한 병합. 분류표 = `/tmp/wie-s3/class/wie-wipi-java.tsv`.

###### 0. 수

| 갈래 | 수 | 해소 |
|---|---:|---|
| 폐기 | 6 | theirs 5 · ours 1(`wipi_midlet.rs` — 아래) |
| 재적용 | 4 | ours 3(`wec.rs` · `wec/oem_device.rs` · `wec/sys_theme.rs`) · 3way-edit 1(`lib.rs`) |
| 발신 | 19 | ours 14 · 3way-edit 4 · union 1 |
| **합** | **29** | ours 18 · theirs 5 · 3way-edit 5 · union 1 |

- «해소 ours» 18파일 가운데 17은 `git hash-object` 가 `origin/main` 의 blob 과 같다(바이트 동일). 나머지 하나 `lcdui/jlet.rs` 는 origin/main 판에서 시험 모듈의 안 쓰는 import 2줄만 지웠다(§4). «theirs» 5파일은 `upstream/main` 과 같다. 내용이 섞인 것은 6파일이다: `lib.rs` · `lwc.rs` · `lwc/component.rs` · `lcdui/graphics.rs` · `handset/handset_property.rs` · `msf/io/message.rs`.
- ★0551 과의 차이: 0551 의 20행(#74~#93) 가운데 **갈래가 바뀐 행은 0**이다. 해소 방식만 **10행**이 `3way-edit → ours` 로 바뀌었다(#74 · #77 · #79 · #81 · #84 · #87 · #88 · #89 · #90 · #92 — 표의 «해소만 다름» 칸). 까닭은 둘이다: ⑴ S1 · S1b 가 owner 인자를 이미 옮겨서 그 몫의 손 수정이 없어졌다. ⑵ upstream 이 11-06 뒤 같은 자리를 «자기 방식으로» 구현했는데(card_canvas clip · lwc ShellComponent/ProxyCard · InputMethodHandler 모드 · Jlet 재진입 가드 · Clip 의 지연 생성) 그 동작이 우리 것과 갈려 «우리 값 유지» 규칙으로 통째 우리 판이 됐다. 반대로 수렴이 더 나아간 곳: `PHONENUMBER`(#75) · `getCurrentJlet`(#81) · `setRGBPixels` 의 processAlpha=false(#78) · TextComponent 의 필드 셋(#88).
- 새 행 9: `net/wie/wipi_midlet.rs` · `msf/io/message.rs` · `lcdui/display.rs` · `lcdui/image.rs` · `lwc/container_component.rs` · `lwc/dialog_component.rs` · `wec.rs` · `wec/oem_device.rs` · `wec/sys_theme.rs`.
- ★`wipi_midlet.rs` 의 «폐기 · ours»: 술어로는 폐기다(우리 델타 = owner 인자뿐 · 전부 upstream). 그런데 upstream 이 더한 `destroyed` 가드는 `Jlet::destroy`(jlet.rs)와 짝이고, jlet.rs 는 우리 동작(notifyDestroyed 가 destroyApp 을 부르지 않는다)을 지키므로 그 가드를 받을 자리가 없다. 그래서 내용은 origin/main 그대로다.

###### 1. 동작이 갈리는 자리 — 전부 우리 값 유지

§8 의 세 자리(★)와 이번에 새로 만난 자리. «upstream» 칸은 `upstream/main` `d5242bbff` 의 값이다.

| 자리 | 우리(유지) | upstream |
|---|---|---|
| ★`HandsetProperty.getSystemProperty("VOLUMELEVEL")` | `"3"` | `"100"`(#1507) |
| ★`Player.stop(null)`(·`resume(null)`·`play(null)`) | 조용히 `false` | `stop` 은 `NullPointerException`(#1501) |
| ★`Player.stop(Clip)` 의 반환 | STARTED 였을 때만 `true` | 플레이어가 있으면 `true` |
| `Player.pause/stop/resume/play(BaseClip…)` | `Clip` 이면 Clip 길로(소리가 난다) | 스텁 `false` |
| `Player.resume(Clip)` | 실구현(재생 중이면 그대로 · 끝났으면 다시) | 스텁 `false` |
| `Clip` 기본 음량 · `setVolume` | 100 · 오디오 핸들에 바로 적용 | 0 · 필드에만 적는다 |
| `Clip(type, null)` | `NullPointerException` | null 검사 없음 |
| `BaseClip.setBuffer` · 플레이어 생성 시점 | 등록할 때 바로 만들고, 배열이 고쳐 쓰였으면(FNV 해시) 재생 때 다시 읽는다 | `stream` 만 붙들고 첫 재생 때 만든다(`Clip.getOrCreatePlayer`) · 같은 배열을 고쳐 써도 다시 읽지 않는다 |
| `BaseClip.putData` | 그 바이트로 바로 플레이어를 만든다 · 등록 배열을 놓는다 | 사본을 떠 `setBuffer` 로 넘긴다 |
| `Jlet.notifyDestroyed` | `destroyApp` 을 부르지 않고 MIDlet 종료로 간다(돌아오지 않는다) | `destroyed` 가드 뒤 `destroyApp(false)` 를 1회 부른다 |
| `WIPIMIDlet.destroyApp` | `Jlet.destroyApp` 을 그대로 부른다 | `Jlet::destroy`(가드 — 두 번째 호출은 무시) |
| `Display.callSerially(Runnable, int)` | 별 태스크가 `timeout` 만큼 잔 뒤 무시간 `callSerially` 로 넘긴다 | `Event::timer` 를 시스템 이벤트 큐에 넣는다(입력 이벤트가 먼저) |
| `CardCanvas.paint` | repaint clip 을 카드마다 유지 · 불투명 카드에 완전히 가려진 카드는 안 그린다 · 카드 경계로는 자르지 않는다 | `g` 의 translate 를 더하고 `setClip` + 카드 경계 `clipRect` · 모든 카드를 그린다 |
| `Card()` · `Card(Z)` 의 y | `AnnunciatorComponent` 를 보였으면 그 높이(240→24 · 176→18)에서 시작 | 0 |
| `AnnunciatorComponent.<init>` | `Component.h` 필드를 따로 적지 않는다(getWidth · getHeight 는 클래스가 덮어쓴다) | `Component.h = 0` 을 적는다 |
| `Graphics.setRGBPixels` 의 `bpl` | scanlength 그대로(int 개수) | `bpl / 4`(바이트로 읽는다) |
| `Graphics.getRGBPixels` | ★upstream 구현을 받았다(우리는 스텁이었다). 단 `bpl` 은 위와 같은 단위(int 개수)로 | `bpl / 4` |
| `Graphics.copyArea` | 우리 구현(`canvas.copy_area` · w/h ≤ 0 이면 무시) | MIDP `Graphics.copyArea` 에 위임 |
| `Image.createImage(Image)` | MIDP 사본(불변 · 투명 유지) | 흰 바탕에 눌러 그린 mutable 사본 |
| `Image.getGraphics` (불변 이미지) | Graphics 를 준다 | `null` |
| `Image.isMutable` | `createImage(II)` 로 만든 것만 `true`(자기 필드) | MIDP `isMutable` 에 위임 |
| `Image.drawImage(…, transform, …)` | transform 값을 MIDP `drawRegion` 에 그대로 넘긴다 | WIPI 각도값(0 · 90 · 180 · 270 · −99 · −90 · −180 · −270)을 MIDP 값으로 사상 · 그 밖은 `IllegalArgumentException` |
| `InputMethodHandler` | 우리 모드 체계 + 멀티탭 · 한글 조합 · 리스너 통지 | 모드 색인 0..3 + `allowedModes`(제약별) · 범위 밖은 `IllegalArgumentException` · 숫자 키면 `"temp"` 통지 |
| lwc `Component` 의 getX…/validate/layout/repaint/hasFocus/isShown/set·getBackground/Foreground/setEventListener | 상태 있는 실구현 | 스텁(0 · no-op) + `focused` 인스턴스 필드 |
| lwc `ShellComponent` show/hide | `net.wie.ShellCard`(lwc 클래스에 필드를 두지 않는다 — LGT AOT 하위 클래스 오프셋) | `ProxyCard` + `cd` 인스턴스 필드 · `getCard` 가 그 카드를 준다 |
| `DialogComponent.doModal` | TYPE_NONE → 10(DLG_TIMEOUT) · 그 밖 → 11(DLG_OK) · 생성자가 cmp 를 첫 자식으로 | 항상 12(DLG_CANCEL) · cmp 를 버린다 |
| `FormComponent` | `vertical` 을 붙들고 `setFocus(Component)` 가 있다 | 생성자 둘뿐 |
| `wec.OEMDevice.getSYSTheme` | null 이 아닌 `SYSTheme` | `null` |
| `wec.SYSTheme` | 정적 둘 = 0 · `saveItem` = 0 · private `<init>` | `ITEM_GROUP_SOUND_DEFAULT` = 1 · `saveItem` = −1 · `<init>` 없음 |
| `org.kwis.msf.io.Message` | `addr` 도 붙든다 | `data` 만 |

★이 가운데 «upstream 이 우리 쪽 잠재 버그를 고친 것으로 읽히는» 자리가 둘 있다 — 규칙대로 우리 값을 뒀고, 옮기려면 자리마다 전/후를 재야 한다: `Image.drawImage` 의 transform 사상(우리는 90 같은 WIPI 각도값을 MIDP 에 그대로 넘긴다 · 반대로 upstream 은 1..7 을 거부한다) · `setRGBPixels`/`getRGBPixels` 의 `bpl` 단위(우리 값은 `a540945188ca` 로 잰 것이다 — 0420).

###### 2. 조용히 합쳐진 파일 — 하나씩 확인

`git diff --name-only 44fbf265 origin/main` ∩ `… upstream/main` 에서 충돌 29를 뺀 13파일 + upstream 만 고친 3파일.

| 파일 | 병합 결과 | 확인 |
|---|---|---|
| `src/classes.rs` | = ours | 우리 `com` · `java` 모듈이 upstream 목록의 상위집합 |
| `net/wie/wipi_file_output_stream.rs` · `msf/io.rs` · `msp/handset.rs` · `msp/lcdui.rs` · `msp/lcdui/font.rs` · `msp/media.rs` | ours = theirs | 양쪽이 같은 내용으로 수렴 |
| `msp/io/file.rs` | = ours | upstream 델타가 우리 것에 들어 있다 |
| `msp/db/data_base.rs` | ours + upstream 2줄 | 빈 줄 1 · `openDataBase` 로그를 warn → debug. 동작 무관 |
| `msp/lcdui/event_queue.rs` · `msp/media/volume.rs` | ours − 시험 1개씩 | merge-base 에 있던 스텁 호출 시험을 upstream 이 지웠다(#1441). 우리가 더한 시험이 아니다 |
| ★`msp/lwc/text_box_component.rs` | **자동 병합이 깨뜨렸다** → origin/main 판으로 | 양쪽이 `<init>(Ljava/lang/String;II)V` 를 따로 더해 `init_with_height` 가 두 번 정의됐다(컴파일 오류). 우리 것은 높이를 실제로 적는다 · upstream 은 스텁 |
| ★`msp/lwc/annunciator_component.rs` | **동작이 바뀌어 들어왔다** → origin/main 판으로 | upstream 의 `Component.h = 0` 기입이 조용히 들어왔다. 우리 판은 getWidth · getHeight 를 클래스에서 덮어쓰고 `h` 필드는 따로 적지 않는다(§1 표) |
| `msp/handset/backlight.rs` · `msp/media/vibrator.rs` (upstream 만 수정) | = theirs | merge-base 의 스텁 시험 삭제(#1441)뿐 |
| `msp/lwc/proxy_card.rs` (upstream 신규) | = theirs | `lwc.rs` · `lib.rs` 에 등록했다. 시험 `tests/proxy_card.rs` 를 더했다 |

충돌 파일 «안»에서 덩어리 밖으로 조용히 들어온 것(덩어리만 풀면 남는다):
- `jlet.rs` — `getCurrentJlet` 이 두 번 등록됐다(우리 별칭 + upstream 판).
- `card_canvas.rs` — 덩어리를 전부 ours 로 풀어도 upstream 의 `translate_x + x` · `setClip` · 카드 경계 `clipRect` 가 우리 루프 안에 남고, 그 변수(`translate_x` · `width`)는 정의가 없다.
- `component.rs` — `set_event_listener` · `layout` · `validate` 가 두 번 정의된다(컴파일 오류). `text_component.rs` — `iMode` 필드가 두 번 선언된다. `input_method_handler.rs` — `getCurrentMode()I` 가 두 번 등록된다. `lib.rs` — 중복 등록 6건(0551 §6 그대로).
- (`git merge-file --ours` 로 «덩어리를 전부 ours 로 푼» 결과를 다시 만들어 센 것이다.)
- ⇒ 그래서 이 조각은 덩어리를 푸는 대신 파일마다 `git diff origin/main upstream/main -- <p>` 로 «upstream 에만 있는 것»을 뽑아 우리 판 위에 올렸다.

###### 3. 내 범위 밖에서 필요한 것

- **없다** — §4 가 내 범위 안의 수정만으로 통과했다. `wie-wipi-java/Cargo.toml` 도 고치지 않았다.
- 참고(요청이 아니다): `ProxyCard` 는 `Card` 의 `<init>(Z)V` · `(IIII)V` · `(Lorg/kwis/msp/lcdui/Display;IIIIZ)V` 와 `Display.getDefaultDisplay` 만 쓴다 — 전부 이 크레이트 안이다.
- 참고: `Graphics.getRGBPixels` 는 `wie-midp` 의 `Image.getRGB([IIIIIII)V` 와 `Graphics::image` 헬퍼를 쓴다(양쪽에 다 있다).

###### 4. 명령 출력 (2026-11-23 20:5x · load1 12.6 · check · test · clippy 는 `build-slot run --` 경유 · 한 번에 하나)

| 명령 | 결과 |
|---|---|
| `cargo check -p wie-wipi-java --all-targets` | rc=0 |
| `RUST_MIN_STACK=4194304 cargo test -p wie-wipi-java` | rc=0 — unit **90 passed · 0 failed** · `tests/preload_classes_come_from_the_runtime.rs` 1 passed · `tests/proxy_card.rs` 1 passed · doc 0 |
| `cargo clippy -p wie-wipi-java --all-targets -- -D warnings` | rc=0 (아래 두 줄을 고친 뒤 · 그 전에는 rc=101) |
| `cargo fmt -p wie-wipi-java -- --check` | rc=0 |
| `/usr/bin/grep -rn -E '^(<<<<<<<\|…\|>>>>>>>)' wie-wipi-java` | 0줄 |

- ★`--all-targets` clippy 는 저장소 게이트(`cargo clippy --all -- -D warnings` · lib 만)보다 넓다. origin/main 의 시험 코드에 원래 있던 경고 둘이 여기서 걸려 고쳤다 — 충돌과 무관하고 둘 다 시험 모듈 안이다: `lcdui/jlet.rs` 의 안 쓰는 import 2줄(`Arc` · `AtomicBool` · `Ordering` · `TestPlatform` · `run_jvm_test_with_system`) · `net/wie/shell_card.rs:849` 의 `clippy::needless_question_mark` 1줄(충돌 목록 밖 파일 · 같은 크레이트).
- 시험 수: `src` 의 `#[test]` 는 origin/main 93 → **90**(upstream 은 27).
  - −5: `handset/led.rs`(우리 시험 · 0551 #76 대로 버림) · `handset/backlight.rs` · `lcdui/event_queue.rs` · `media/vibrator.rs` · `media/volume.rs`(이 넷은 merge-base 에 있던 스텁 호출 시험을 upstream #1441 이 지운 것이 자동 병합으로 들어왔다 — 우리가 더한 시험이 아니다).
  - +2: `msf/io/message.rs` 의 upstream 시험 `message_retains_the_supplied_buffer` · `lcdui/graphics.rs` 의 `test_get_rgb_pixels_round_trips_through_set_rgb_pixels`(새 getRGBPixels).
  - 통합 시험 +1: `tests/proxy_card.rs`(`every_proxy_card_constructor_yields_a_usable_card`).
  - ★병합이 시험 5개를 지운다는 뜻이다(`AGENTS.md` «one that deletes or moves tests runs `coverage.yml`»). 넷을 되살리려면 그 네 파일을 `git show origin/main:<p> > <p>` 로 두면 된다(본문은 upstream 과 같고 시험 모듈만 다르다) — 부모 판단에 맡긴다.
- 받지 않은 upstream 시험 4개(전부 «갈린 동작»을 단언한다 — §1 표의 해당 행): `lcdui/display.rs::delayed_callbacks_follow_deadlines_and_survive_gc` · `lcdui/jlet.rs::destroy_app_runs_once_when_it_notifies_destroyed` · `lcdui/input_method_handler.rs::input_modes_respect_constructor_constraints` · `lwc/shell_component.rs::shell_component_paints_and_receives_keys_through_its_card`(이 가운데 ProxyCard 생성자 부분만 `tests/proxy_card.rs` 로 옮겼다). 그 밖에 upstream 이 고쳐 쓴 기존 시험(card.rs 의 clip 픽셀 · graphics.rs 의 `test_copy_area` 확장 · image.rs · clip.rs 의 기본 음량 0 · player.rs 의 `test_clip_playback_reuses_player_until_buffer_replacement`)도 같은 이유로 우리 판이다.
- 건드린 파일(크레이트 안): 충돌 29 + `lwc/annunciator_component.rs` · `lwc/text_box_component.rs`(조용한 병합을 origin/main 판으로) + `net/wie/shell_card.rs`(clippy 1줄) + 새 파일 `tests/proxy_card.rs`. `lwc/proxy_card.rs` 는 병합이 이미 넣어 둔 upstream 판 그대로다. git 쓰기 명령은 쓰지 않았다(스테이징은 부모 몫).

###### 5. §8 세 값이 병합 뒤에도 우리 값임을 단언하는 시험

| 값 | 시험 |
|---|---|
| `VOLUMELEVEL` = `"3"` | `classes::org::kwis::msp::handset::handset_property::test::test_volume_level_is_a_number` — 종전에는 «숫자다»만 단언했다. 이 조각이 `assert_eq!(…, "3")` 으로 조였다(upstream 값 `"100"` 이면 red) |
| `Player.stop(null)` = 조용히 `false` | `classes::org::kwis::msp::media::player::test::test_null_clip_reports_failure` — `stop`·`resume`·`play` 에 null clip → 셋 다 `false`(예외면 `?` 에서 red) |
| `Player.stop` 은 STARTED 였을 때만 `true` | `classes::org::kwis::msp::media::player::test::test_stopping_an_idle_clip_reports_false` — 재생 전 `false` → 재생 중 `true` → 다시 `false` |

### 끝. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치가 main 대비 더하거나 고친 318파일): **유입 88건(BOUNDED)** + **판단 필요 9건(SUFFIX-ATTACHED)**. 전부 `origin/main` 에 이미 있던 `.rs` · `.toml` 의 주석이다 — 병합이 그 파일들을 다시 건드려 대상에 들었을 뿐이다(가장 많은 곳: `wie-lgt/data/lgt_java_abi.toml` · `wie-lgt/src/runtime/java/jvm_support.rs` · `wie-skvm/.../wie_audio_clip.rs`). 이 브랜치가 `origin/main` 대비 코드(`*.rs` · `*.toml` · `*.mjs` · `*.sh`)에 더한 한글 줄은 2줄이고 둘 다 upstream 의 줄이다 — 이 회차가 쓴 것은 0줄이다. 이 회차가 쓴 문서(이 기록 · worklog · 고지 · `STATE.md`)만 넣어 돌리면 BOUNDED 0 · SUFFIX-ATTACHED 0 이다.
