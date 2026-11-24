## [2026-11-24] 조각 S4 — upstream 병합 뒤 정리 (다음 병합이 다시 충돌할 두 자리를 치움 · S3 검수의 문서 정정) (wie-upstream-sync2-s4-post-merge-cleanup)

**무엇을**: `docs/report/0551` §7 의 조각 S4 다. S3(병합 커밋 · PR #617 · `de096979`)이 착지해 merge-base 가 `d5242bbff` 로 옮겨 간 직후에, 같은 경로를 양쪽이 따로 가져서 upstream 이 그 파일을 고칠 때마다 충돌하던 두 자리를 치웠다 — `wie-wipi-c/src/api/database.rs` 와 `docs/lgt.md` 를 upstream 판 그대로 두고 우리 것은 `api/fork_database.rs` · `docs/lgt-fork-notes.md` 로 옮겼다. `tag-release.yaml` PARKED 머리에서 거짓이 된 문장 셋을 지금의 스크립트대로 고쳤고, S3 검수(게이트②)가 남긴 문서 정정 넷(F1 · F3 · F4 · F9)을 0586 과 worklog 에 넣었다.

**왜**: 티켓 `wie-upstream-sync2-s4-post-merge-cleanup`. merge-base 가 68일 만에 움직인 직후가 이 정리를 가장 싸게 하는 때다 — 지금은 `upstream/main` 이 merge-base 와 같아서(`d5242bbff` · 2026-11-24 조회) 우리 쪽만 고치면 된다.

**사용자 영향**: 없다. 게임이 도는 길은 한 줄도 바뀌지 않았다 — KTF · LGT 표가 부르는 함수는 이름 앞의 모듈만 `database::` → `fork_database::` 로 바뀌었고 본문은 옮긴 파일 그대로다(§3). `compat.json` 은 고치지 않았다. 엔진 `*.rs` 가 바뀌므로 착지하면 엔진 아티팩트 릴리스와 otterpebble 핀 범프가 뜬다.

### 0. 먼저 읽을 것

1. **`wie-wipi-c/src/api/database.rs` 와 `docs/lgt.md` 는 이제 upstream 의 파일이다(바이트 동일). 여기서 고치지 마라.** 고치면 그 줄부터 병합마다 다시 충돌한다. 우리 쪽 변경은 `api/fork_database.rs` · `docs/lgt-fork-notes.md` 에 한다.
2. **upstream 의 `api/database.rs`(`MC_db*` 레코드 API)는 컴파일되지만 우리 표에 배선하지 않았다** — `api/fs.rs` 와 같은 처지다. 저장소와 핸들이 달라 섞을 수 없다는 판단(0586 부록 B `wie-wipi-c` ⑸)은 그대로다.
3. **`wie-ktf/Cargo.toml` 의 안 쓰는 `anyhow` 는 지우지 않았다.** 그 파일은 upstream 과 바이트가 같고, 지우면 그 줄이 새 충돌 자리가 된다(§4 · 잰 것).
4. **upstream `AGENTS.md` 의 Attribution 조항은 정하지 않았다**(사람 결정). 원문 · 지금 있는 표기 · 선택지는 §5.

### 1. 먼저 잰 것 — S4 의 다섯 항목이 `de096979` 에서 아직 참인가

| 항목(`STATE.md` S4 절) | `origin/main` `de096979` 에서 | 처분 |
|---|---|---|
| `tag-release.yaml` PARKED 머리의 나머지 문구 | 참 — S3 이 한 문장을 고쳤고, 거짓인 문장이 셋 더 있다(§2) | 고쳤다 |
| `docs/lgt.md` 의 upstream 판을 받을지 | 참 — 우리 167줄 ↔ upstream 310줄 · merge-base 대비 우리 +152 −293 · upstream +3 −1 | 받았다(§3) |
| `api/database.rs` 가 병합마다 충돌 | 참 — 우리 1,325줄 ↔ upstream 350줄 · 같은 경로 | 이름을 바꿨다(§3) |
| `wie-ktf/Cargo.toml` 의 안 쓰는 `anyhow` | 참 — `wie-ktf/src` · `tests` 에 `anyhow` 0건 | 일부러 뒀다(§4) |
| upstream `AGENTS.md` Attribution | 참 — 우리 `AGENTS.md` 에 없다 | 사람 결정(§5) |

S3 이 이미 치운 항목은 없다.

### 2. `tag-release.yaml` PARKED 머리

머리가 적은 것과 지금 트리의 `.github/scripts/release/tag-version.sh`(S3 병합으로 들어온 upstream 판)를 줄마다 대조했다.

| 머리의 문장 | 지금의 스크립트 | 판정 |
|---|---|---|
| `cargo set-version` 으로 워크스페이스 전체 버전을 올린다 | `cargo set-version --workspace "$RELEASE_VERSION"` | 참 |
| `v*` 태그와 `release/v*` 브랜치를 밀고 버전 범프 PR 을 연다 | `git push --atomic origin HEAD:refs/heads/main "refs/tags/$tag"` — 범프 커밋을 `main` 에 곧바로 민다. 브랜치 · PR 없음(upstream `6503cf383` · 2026-09-26) | **거짓** |
| 그 뒤 `gh workflow run release.yaml` 을 친다 — 그래서 이 파일이 `release.yaml` 을 띄울 수 있는 유일한 길이다 | 그 단계가 없다. upstream 은 태그 push(`on.push.tags`)로 `release.yaml` 을 띄우는데, 우리 `release.yaml` 은 그 트리거를 뗀 채 주차돼 있다 | **거짓** |
| 다음 버전을 `select(.name == "wie-app")` 로 읽는다 | `tag-version.sh` 에 없다(그 줄은 `release.yaml` 이 쓰는 `resolve-context.sh:12` 에 있다 · upstream `f6b905d9f` 가 태거에서 뺐다) | **거짓** |
| 버전 범프가 `**/Cargo.toml` 을 건드리고 그것이 `publish-artifact.yml` 의 릴리스 트리거다 | `git add -- Cargo.toml Cargo.lock ':(glob)**/Cargo.toml'` · `publish-artifact.yml` `on.push.paths` 에 `**/Cargo.toml` | 참(«PR 이 착지하면»만 «그 push 가 받아들여지면»으로) |
| 본문은 2026-11-23 병합 때의 upstream 그대로 | 머리 22줄을 뺀 본문 == `git show upstream/main:.github/workflows/tag-release.yaml`(diff 0줄) | 참 |

거짓 셋을 지금의 동작으로 다시 적었다. 트리거는 건드리지 않았다. 짝인 `release.yaml` 의 머리도 같이 읽었다 — `npm run build:prod` 가 우리 `package.json` 에 없다는 것 · Pages 프로젝트 이름 · `wie-app` 제외는 지금도 참이라 고칠 문장이 없다.

### 3. 같은 경로를 양쪽이 가진 두 자리 — upstream 판을 제 경로에 두고 우리 것의 이름을 바꿨다

**`wie-wipi-c/src/api/database.rs`**
- `git mv api/database.rs api/fork_database.rs` 뒤 `api/database.rs` 에 `upstream/main` 의 것을 그대로 뒀다(`cmp` 동일). `api.rs` 에 `pub mod fork_database;` 한 줄.
- 부르는 쪽: `wie-ktf/src/runtime/wipi_c/method_table.rs` 23자리 · `wie-lgt/src/runtime/wipi_c.rs` 12자리의 `database::<함수>` 를 `fork_database::<함수>` 로. 함수 이름 · 순서 · 슬롯 번호는 그대로다.
- 옮긴 파일의 본문은 머리 주석 9줄을 더한 것 말고 같다(`git diff -M` 유사도는 §8). 시험 20개가 같이 갔다.
- upstream 의 `database.rs` 는 `pub mod database;` 로 컴파일되고 우리 표 어디에서도 부르지 않는다. 그 파일의 시험 `fixed_records_persist_size_and_support_crud` 가 다시 돈다(S3 이 싣지 못한 upstream 시험 가운데 하나 — 0586 §11 F4).
- 주석 · 장부의 경로 표기 넷을 같이 고쳤다(`wie-lgt` `svc_ids.rs` · `wie-wipi-c` `lib.rs` · KTF 표 주석 · `docs/contracts/runtime-adjustments.tsv` 의 설명 주석 줄 — 장부의 행은 그대로다).

**`docs/lgt.md`**
- upstream 판(310줄 · 꾸러미 · 네이티브 초기화 · AOT Java 자료 모델 · import 표 `0x64` 의 색인표)을 그대로 받았다(`cmp` 동일). 우리 판은 `docs/lgt-fork-notes.md` 로 옮기고 머리에 «플랫폼 참조는 `docs/lgt.md` · 여기는 fork 노트» 라는 안내를 달았다.
- 받은 까닭: upstream 판이 적는 것은 플랫폼 ABI 이고 그 ABI 를 구현한 코드(`wie-lgt/src/runtime/java/*`)는 이미 우리 트리에 있다 — 예: import `0x64` 는 우리 트리에서도 `JavaSystemSvcId::GetInterfaceMethodTable` 이다(`runtime/java/interface.rs:60`). 우리 판은 첫 문단이 «Java 앱은 아직 구현되지 않았다»로 낡아 있었다(같은 파일의 뒤 절이 AOT Java 세 판이 그려진다고 적는다) — 옮기면서 그 문장만 고쳤다.
- `docs/lgt.md` §7 을 가리키던 살아 있는 표기 셋을 새 경로로 고쳤다(`wie_featurephone/src/lib.rs` 의 문서 주석 · `docs/architecture.md` · `docs/project-kb/10_deep-assets.md`). 날짜가 박힌 옛 판정 문서(`docs/upstream-realign-verdict.md` 등)의 인용은 그대로 뒀다 — 새 파일 머리가 «옛 문서의 `docs/lgt.md` §7 은 이 파일»이라고 받는다.

**다음 병합이 정말 안 충돌하는가** — `upstream/main` 이 merge-base 와 같아 실제 병합으로는 잴 것이 없다(`git merge-tree --write-tree origin/main upstream/main` 은 전에도 rc=0 · 트리 == `origin/main`). 그래서 upstream 의 다음 커밋을 흉내 냈다: `upstream/main` 위에 `database.rs` 에 상수 한 줄 · `docs/lgt.md` 의 표 한 줄 · `wie-ktf/Cargo.toml` 에 의존 한 줄을 고친 커밋을 객체로만 만들어(ref 없음) 메모리에서 합쳤다. 결과는 §8.

### 4. `wie-ktf/Cargo.toml` 의 `anyhow` — 지우지 않았다

- 안 쓰는 것은 맞다(`wie-ktf/src` · `tests` 0건 · upstream 은 `jvm_implementation.rs:86` 의 `ExternalRoots` 구현에서 쓰고, 우리는 그 설계를 배선하지 않았다 — 0586 부록 B).
- 그런데 그 파일은 지금 upstream 과 **바이트가 같다**(`git diff upstream/main origin/main -- wie-ktf/Cargo.toml` 0줄). upstream 은 merge-base 가 서 있던 68일 동안 그 파일을 세 번 고쳤다.
- 지우면 생기는 것: 그 줄과 `Cargo.lock` 의 `wie-ktf` 의존 목록, 두 군데가 upstream 과 갈린다. §8 의 흉내 병합에서 upstream 이 바로 옆 줄에 의존을 더하면 — 지금은 깨끗하게 합쳐지고, `anyhow` 를 지운 트리에서는 충돌한다.
- 지워서 얻는 것: 없다. `anyhow` 는 `jvm 0.5.1` 이 끌어와 wasm 그래프에 어차피 있다(`cargo tree -p wie_featurephone --target wasm32-unknown-unknown -i anyhow`). 안 쓰는 의존을 잡는 게이트도 없다(`unused_crate_dependencies` 는 기본 allow · CI 에 machete/udeps 0건).
- S4 의 뜻이 «다음 병합이 다시 충돌할 자리를 치운다»이므로 여기서는 두는 쪽이 그 뜻에 맞다. 되돌리는 조건: upstream 이 그 줄을 지우거나, 안 쓰는 의존을 막는 게이트가 생길 때.

### 5. upstream `AGENTS.md` 의 Attribution 조항 — 사람이 정할 것

**원문**(upstream `334eb379e` · 2026-10-01 에 더해짐):
> - When creating a product derived from or using this project, you must include the following acknowledgement in its README, documentation, or credits: "This project is based on [wie](https://github.com/dlunch/wie), created by Inseok Lee."
> - The acknowledgement must also be visible within the product itself, for example on an About or Credits screen.
> - This is a mandatory agent instruction, not an additional license condition.

**지금 이 트리에 있는 표기**
- `LICENSE` 15행에 그 문장이 있다(upstream 의 `LICENSE` 와 바이트 동일 — 거기서는 «encouraged» 다).
- `web/src/components/ServiceInfo.tsx:72-77`(wie-web 의 서비스 정보 화면): «에뮬레이터 코어: MIT 라이선스 · © 2020 Inseok Lee» 와 `github.com/dlunch/wie` 링크. 조항의 문장 그대로는 아니다.
- `README.md` 에는 «based on» 문장이 없다(upstream 의 README 를 바탕으로 한 파일이다).
- otterpebble 의 피처폰 셸 화면은 이 회차가 보지 않았다(다른 저장소).

**선택지**
- ⒜ 조항을 통째로 받는다 — 우리 `AGENTS.md` 에 조항을 싣고, 문장을 `README.md` 와 제품 화면 둘(wie-web 서비스 정보 · otterpebble 셸)에 넣는다.
- ⒝ 문장만 넣는다 — `README.md` 와 `ServiceInfo.tsx` 에 문장을 그대로 넣고, 셸 화면은 otterpebble 티켓으로 낸다. `AGENTS.md` 에는 «그 문장을 지우지 마라» 한 줄만.
- ⒞ 받지 않는다 — `LICENSE` 의 문장과 지금의 저작권 줄로 둔다.

**권고는 ⒝ 다.** 이 저장소와 otterpebble 셸이 바로 조항이 말하는 «derived product» 이고, 저자가 요청한 표기의 비용은 문장 두 줄이다. 조항 전문을 우리 `AGENTS.md` 에 싣는 것은 얻는 것이 적다 — 그 파일은 양쪽이 통째로 다른 UU 자리라 조항을 실어도 다음 병합의 충돌은 그대로고, 우리 헌장은 «상시 규칙만 · 850줄 상한»이다. 제품 화면에 문구를 넣는 일은 이용자에게 보이는 변화라 사람이 정하는 것이 맞다.

### 6. S3 검수(게이트②)가 남긴 문서 정정

정정의 본문은 `docs/report/0586` §11 에 넣었다(그 기록을 읽는 사람이 거기서 만나야 한다). 여기에는 무엇을 고쳤는지만 적는다.

| | 고친 곳 | 내용 |
|---|---|---|
| F1 | 0586 §2 #14 · worklog `2026-11-23-upstream-sync2-s3-merge-upstream-main.json` 의 `proposals[0].why` · `tradeoff` · `effort` | LBMP 378장 · 361장 · 5판 → 6,202장 · 4,628장 · 35판(검수 실측 — 이 회차가 다시 세지는 않았다). `effort` 는 수를 고치면서 S → M 으로 적었다(전/후 캡처가 5판에서 35판으로 는다). 제안은 채택하지 않았다 |
| F3 | 0586 §11 | S3 회신의 빈 CI 칸에 들어갈 값(검수 시각 rc=5 · 착지 직전 rc=0). 회신 파일은 저장소 밖 원장이라 고치지 않았다 |
| F4 | 0586 §7 머리 · §11 | «빠진 시험 6개»는 이름 기준 24개다. 다시 셌다: `git grep` 으로 `a3217eac0` 871개 · `de096979` 991개 · 전자에만 있는 이름 **24** |
| F9 | 0586 §11 · `scripts/regression-diff.mjs:121` | 증적 머리의 «(origin/main · release)» 는 스크립트의 고정 문구였다. 브랜치에서 손으로 돌리면 틀린다 — 문구를 «주간 감시면 origin/main, 손으로 돌렸으면 넘긴 sha»로 고쳤다(`selftest` 22/22). 증적 파일은 그대로다 |

### 7. F11 — release 바이너리의 `keydraw_ktf.zip --inject --expect-last-frame`

**판정: 퇴행이 아니다. 원래 그렇다** — 병합 전 main(`a3217eac0`)의 release 도 같은 조건에서 같은 값을 낸다. 고친 것은 없다.

두 release `wie_validate`(병합 전 main `a3217eac0` · 착지본 `de096979`)를 각자 지어 같은 픽스처(두 커밋에서 blob 동일 `83d265aa`)를 **같은 순간에** 쌍으로 세 번씩 돌렸다.

| 조건 | 병합 전 `a3217eac0` | 착지본 `de096979` |
|---|---|---|
| 기본값(`--max-ticks` 50M) · load1 58 ~ 61 | 3/3 `PASS` · rc=0 · 입력 27/27 · `stop=deadline` · paints 79 · ticks 22.8M ~ 30.1M | 3/3 `PASS` · rc=0 · 입력 27/27 · `stop=deadline` · paints 79 ~ 80 · ticks 25.5M ~ 33.6M |
| `--max-ticks 20000000` · load1 78 ~ 85 | 3/3 `PASS` · 27/27 · ticks 13.4M ~ 14.6M(상한에 안 닿음) | 3/3 `PASS` · 27/27 · ticks 14.2M ~ 16.8M(상한에 안 닿음) |
| `--max-ticks 8000000` · load1 103 ~ 104 | 3/3 `UNMEASURED` · rc=2 · 입력 7 ~ 10/27 · `stop=max-ticks` | 3/3 `UNMEASURED` · rc=2 · 입력 7 ~ 10/27 · `stop=max-ticks` |

- 검수가 본 `UNMEASURED`(입력 17/27 · `max-ticks`)는 이 회차의 기본값 실행에서 재현되지 않았다. 그 판정의 원인은 틱 상한이다 — `--inject` 의 키 일정은 벽시계(약 20초)로 가고 `--max-ticks` 는 틱 수로 끊는다. release 는 debug 보다 틱이 빨라서, 호스트가 한가하면 20초가 되기 전에 50M 틱을 채운다. 상한을 8M 으로 내려 그 상태를 억지로 만들면 두 바이너리가 똑같이 `UNMEASURED` 가 된다(셋째 행).
- 착지본이 같은 20초에 틱을 5 ~ 15% 더 돈다(상한에 안 닿은 여섯 쌍 모두 · 0586 §8-5 의 «50M 틱당 CPU 9 ~ 16% 감소»와 같은 방향). 그래서 착지본이 상한에 먼저 닿는다 — 한가한 호스트에서 병합 전은 `PASS`, 착지본은 `UNMEASURED` 가 나오는 좁은 구간이 있을 수 있다. 그것은 빨라진 것이지 고장이 아니다.
- `AGENTS.md` 가 이미 적은 대로 읽으면 된다: `UNMEASURED` 는 `FAIL` 이 아니고, `stop` 이 `max-ticks` 면 `--max-ticks` 를 올린다. 러너 블록은 debug 기준이고 debug 는 27/27 `PASS` 다(§8).
- 재현: `cargo build --release -p wie_cli --bin wie_validate` 뒤 `target/release/wie_validate --inject --expect-last-frame --max-ticks 8000000 test_data/keydraw_ktf.zip`.

### 8. 검증식 — 실제 출력

이 Mac · 2026-11-24 05:29 ~ 07:31 KST · 게이트와 F11 을 돌린 순간의 load1 19 ~ 117(다른 레인의 스윕과 빌드가 같이 돌았다). 무거운 것은 `build-slot run`(short) 경유.

**네 게이트 + beta** — 커밋마다 그 커밋의 트리에서 돌렸다.

| 트리 | fmt | `clippy --all -D warnings` | wasm clippy | `RUST_MIN_STACK=4194304 cargo test --all` | `cargo +beta clippy --all -D warnings` |
|---|---|---|---|---|---|
| A — 이름만 바꾼 것(커밋 `1a53d7463`) | rc=0 | rc=0 | rc=0 | rc=0 · 67 스위트 · **983 passed** · 0 failed | — |
| B — upstream 두 파일을 제 경로에(커밋 `3da885340`) | rc=0 | rc=0 | rc=0 | rc=0 · 67 스위트 · **984 passed** · 0 failed | rc=0 |
| C — 문서 커밋(연번을 0592 로 옮기며 `fork_database.rs` 머리 주석의 숫자가 바뀜) | rc=0 | rc=0 | rc=0 | rc=0 · 67 스위트 · **984 passed** · 0 failed | rc=0 |

- 983 → 984 는 upstream `api/database.rs` 의 `fixed_records_persist_size_and_support_crud` 하나다. `api::fork_database::tests::` 20개는 A · B 양쪽에서 돈다. `database_slot_11_answers_free_storage` · `wipic_svcs_0x68_0xf0_0x19c_0x44c_keep_the_fork_meaning` · 장부 가드 `the_ledger_lists_exactly_*` 6개 ok.
- beta(`clippy 0.1.101 · f47eed4a8d 2026-11-22`)는 rc=0 이고 경고 7줄을 낸다 — 전부 `wie_cli` 매니페스트의 `unused dependency`(`directories` · `fast_image_resize` · `hqx` · `midir` · `rodio` · `softbuffer` · `winit` · cargo 의 `cargo::unused_dependencies` 가 beta 에서 기본 warn). 이 회차가 만든 것이 아니고(`wie_cli/Cargo.toml` 무수정) `-D warnings` 에 걸리지 않는다. ★`wie-ktf` 의 `anyhow` 는 그 목록에 없다.
- PR 의 green 은 beta 를 덮지 않는다(`AGENTS.md` — PR 의 beta 레그는 빈 통과). 위 줄이 이 회차의 beta 그물 전부다.

**러너 블록**(트리 B · debug · 06:20 ~ 06:23 · load1 117): `make-draw-fixture` rc=0 뒤 6/6 `PASS` · rc=0 — `draw_j2me.jar`(paints 1) · `helloworld_ktf.zip` · `helloworld_lgt.zip` · `keydraw_ktf.zip --inject --expect-last-frame`(입력 27/27 · paints **79** · 끝 화면 있음) · `keydraw_lgt.zip`(27/27 · paints **55**) · `text_j2me.jar --timeout 5`(paints 1). paints 79 · 55 는 S3 머리에서 검수가 잰 값과 같다. 트리 C 에서는 다시 돌리지 않았다(B 와의 차이가 주석의 숫자 하나다).

**S4 의 검증식**(0551 §7: V0 · `check-doc-liveness-parity` · `check-parked-workflows` — 셋째 명령은 덤이다)
```
$ node scripts/check-doc-liveness-parity.mjs
doc-liveness parity: OK — 26 documented line(s), workflow copy + NOT-RUN cover all of them, nothing extra
$ node scripts/check-parked-workflows.mjs
  ✓ .github/workflows/release.yaml: workflow_dispatch only
  ✓ .github/workflows/tag-release.yaml: workflow_dispatch only
  ✓ .github/workflows/web.yaml: workflow_dispatch only
parked-workflow guard — 3 parked, 0 violation(s)
OK — the parked upstream workflows are still dispatch-only.
$ node scripts/check-upstream-new-workflows.mjs
new-upstream-workflow guard — upstream 7, ours 12, declared-absent 0, new 0
```
V0 의 나머지는 위 표와 러너 블록이다.

**다음 병합 흉내**(§3 · upstream 의 다음 커밋을 흉내 낸 객체 `44866c89` = `upstream/main` + `database.rs` 1줄 · `docs/lgt.md` 1줄 · `wie-ktf/Cargo.toml` 1줄)

| 우리 쪽 | `git merge-tree --write-tree --name-only <우리> 44866c89` |
|---|---|
| 전 — `origin/main` `de096979` | rc=1 · `CONFLICT (content)` 둘: `docs/lgt.md` · `wie-wipi-c/src/api/database.rs` |
| 후 — 커밋 `3da885340` | **rc=0 · 충돌 0**. 합쳐진 트리에 upstream 의 세 줄이 다 들어가 있고 `api/fork_database.rs` 는 우리 것 그대로 |
| 후에서 `wie-ktf/Cargo.toml` 의 `anyhow` 줄만 지운 트리(§4) | rc=1 · `CONFLICT (content)`: `wie-ktf/Cargo.toml` |

티켓이 적은 문자 그대로의 명령 `git merge-tree --write-tree HEAD upstream/main` 은 rc=0 이고 결과 트리가 `HEAD` 와 같다 — `upstream/main` 이 merge-base 라서 전에도 후에도 그렇다. 그래서 위 흉내로 쟀다.

**옮긴 것이 «그대로»인가**: `git show -M --name-status 1a53d7463` → `R098 wie-wipi-c/src/api/database.rs → api/fork_database.rs`(+9줄 머리 주석) · `R093 docs/lgt.md → docs/lgt-fork-notes.md`. `cmp` 로 `HEAD` 의 `api/database.rs` · `docs/lgt.md` == `upstream/main` 의 것. `git diff upstream/main HEAD -- wie-ktf/Cargo.toml` 0줄.

**착지 기준**: `git merge-tree --write-tree origin/main HEAD` rc=0(main 은 그 사이 `117a4e901` 로 움직였다 — PR #621 · 겹치는 파일 0) · 그 트리의 `docs/player-data/compat.json` == main · `node scripts/check-compat-revert.mjs` → `compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 0`. `node scripts/player-data.mjs` → `player-data OK — 429 titles … runtimeAdjustments on 49 rows (ledger 54 rows)`(장부의 주석 줄만 고쳤다). `node scripts/check-worklog-json.mjs` OK.

**옮긴 경로를 만지는 열린 PR**: 0건(2026-11-24 조회 · `api/database.rs` · `api/fs.rs` · `docs/lgt.md` · KTF/LGT 표 · `tag-release.yaml`).

**연번**: 처음 집은 0590 이 그 사이 열린 PR #622 와 겹쳐(이쪽이 나중 · 미push) 0592 로 옮겼다.

### 9. 못 잰 것 · 남긴 것

- **wasm 빌드 · `check-engine-contract` · `contract-roundtrip` 을 로컬에서 돌리지 않았다.** PR 의 `contract` 잡이 돈다. `wie_featurephone` 에서 바뀐 것은 문서 주석 한 줄이고 `#[wasm_bindgen]` 항목은 0줄이다. upstream 의 `api/database.rs` 는 어디서도 부르지 않는 `pub` 함수라 아티팩트 크기가 조금 움직일 수는 있다 — 재지 않았다.
- **코퍼스(`smoke_gate` · census)를 돌리지 않았다.** 게임이 타는 코드는 모듈 이름 말고 같다는 것이 주장이고, 그 주장은 위 시험과 `R098` 이 진다.
- **F1 의 수(6,202 · 4,628 · 35)를 다시 세지 않았다.** 검수 실측을 옮겨 적었다.
- **F11 은 한가한 호스트에서 재지 못했다.** 회차 내내 load1 이 58 이상이라 기본값(50M)에서는 상한이 물리지 않았다. 상한을 내려 같은 상태를 만든 것으로 갈음했다(§7).
- **otterpebble 셸의 저작자 표기는 보지 않았다**(§5 · 다른 저장소).
- **주 1회 CI · 착지 뒤 windows 레그**(S3 `-merge` 회신이 남긴 것)는 이 조각이 다루지 않았다 — `AGENTS.md` 의 게이트③ 규칙 몫이다.
- worklog 주기: `node scripts/check-worklog-coverage.mjs`(기록 없이) → `landed rounds 549 · last recorded at 505 · 최근 10회 8/10 = 80.0%`(OVERDUE 경고 · 비차단). `--record` 는 치지 않았다(총괄 유보).
- 제안 0건. 이 회차의 worklog 파일은 없다.
