## [2026-11-22] 엔진에 컴파일돼 들어가는 표만 바꾼 착지도 릴리스가 서게 — 발행 경로 필터의 죽은 `data/**` 를 `wie*/data/**` 로 (wie-publish-artifact-path-filter-misses-wie-core-arm-data-binary-patches)

**무엇을** — `publish-artifact.yml` 의 `on.push.paths` 와 `engine-contract.yml` 의 job 안 paths-filter 에서 `data/**` 를 `wie*/data/**` 로 바꿨다.
`engine-contract.yml` 필터에 `scripts/make-resize-fixture.mjs` 한 줄을 더했다(§3). 엔진 코드 변경 0.

**왜** — `data/**` 는 저장소 루트 기준인데, 루트 `data/` 는 2026-09-16 상류 base 교체(`37734e742`) 때 크레이트 안으로 옮겨져 그 뒤로 0개 파일에 걸렸다.
엔진이 `include_str!` 로 읽는 표 둘(`wie-core-arm/data/binary_patches.toml` · `wie-lgt/data/lgt_java_abi.toml`)이 어느 줄에도 안 걸려, 표만 바꾼 착지는 릴리스가 서지 않고 otterpebble 핀도 오르지 않았다.
같은 줄이 `contract` 의 필터에도 있어, 표만 바꾼 PR 은 계약 검사(빌드 + 브라우저 왕복)를 건너뛰고 통과했다.

**사용자 영향** — 이 PR 자체로는 없다. 이 PR 이 착지하면 워크플로 파일이 필터에 걸려 릴리스가 서고, 아직 운영에 안 실린 #608(`0b50fb735` · 발화 0 인 훅 삭제 — 결과 변화 없음)이 같이 실린다.
앞으로는 표 단독 변경(잠금 해제 행 등)이 착지와 함께 운영에 나간다.

### §1 대조 표 — 엔진 릴리스의 입력 × 발행 필터

`git grep -n -E 'include_(str|bytes)!|build\.rs'` 와 `build-wasm.sh` · 릴리스 스텝이 읽는 파일 전수.

| 입력 | 읽는 자리 | 고치기 전 | 고친 뒤 |
|---|---|---|---|
| Rust 소스 전부(`patches/classfile` · `build.rs` 포함) | cargo | ✓ `**/*.rs` | ✓ |
| `Cargo.toml` · `Cargo.lock`(RustJava `rev` 핀 포함) | cargo | ✓ | ✓ |
| `wie-core-arm/data/binary_patches.toml` | `wie-core-arm/src/binary_patches/parser.rs:11` `include_str!` | **✗** | ✓ `wie*/data/**` |
| `wie-lgt/data/lgt_java_abi.toml` | `wie-lgt/src/runtime/java/abi.rs:5` `include_str!` | **✗** | ✓ `wie*/data/**` |
| `assets/neodgm.ttf` | `wie_featurephone/src/platform.rs:65` 외 `include_bytes!` | ✓ `assets/**` | ✓ |
| `wie_featurephone/src/audio_worklet.js` | `audio.rs:17` `include_str!` | ✓ `wie_featurephone/src/*.js` | ✓ |
| `wie_featurephone/src/soundfont_prelude.mjs` → `OUT_DIR/soundfont_prelude.js` | `scripts/build-soundfont-prelude.mjs` → `wie_featurephone/build.rs` → `audio.rs:26` | ✓ `*.mjs` · 스크립트 · `package-lock.json` | ✓ |
| `scripts/build-wasm.sh` · wasm-bindgen/binaryen 핀(워크플로 안) | 빌드 | ✓ | ✓ |
| `licenses/**` | 릴리스 자산 | ✓ | ✓ |
| `scripts/player-data.mjs` · `docs/player-data/**` · `docs/player-updates/**` | 「Build public data」 | ✓ | ✓ |
| `docs/contracts/runtime-adjustments.tsv` | `player-data.mjs build` 가 **검증으로만** 읽는다 | ✗(의도) | ✗(의도) |
| `package.json` | `npm ci` | ✗(의도) | ✗(의도) |
| 릴리스 게이트의 입력(`check-engine-contract.mjs` · `contract-roundtrip.mjs` · 계약 JSON · `test_data/**` · fixture 생성기) | 게이트 두 스텝 | ✗(의도) | ✗(의도) |

- 「✗(의도)」 셋은 산출물의 바이트를 바꾸지 않는다. 장부 tsv: `build` 의 출력은 `compat.json` 에서 나오고, 장부와 어긋난 `compat.json` 은 red 라 출력이 달라지는 장부 변경은 `docs/player-data/**` 를 같이 건드린다. `package.json`: 의존성이 바뀌면 `package-lock.json` 이 같이 움직인다. 게이트 입력: `engine-contract.yml` 의 주석이 「test inputs only — not artifact inputs」 로 이미 갈라 둔 구분이다.
- 시험에서만 읽는 `include_*!`(`test_data/*.zip` · `wie-*/tests/data/*.zip` · `test-utils` 의 장부 tsv · `wie-jvm-support` 의 `emulator.rs` 4건)는 산출물에 안 들어간다 — `test-utils` 는 `wie-core-arm` · `wie-lgt` 의 `[dev-dependencies]` 다.
- `wie-app/build.rs` 는 tauri 호스트라 wasm 산출물 밖이다. 서브모듈은 없다(`.gitmodules` 없음).
- 죽은 줄은 `data/**` 하나였다 — 두 필터의 나머지 줄은 전부 추적 파일 1개 이상에 걸린다(실측).

`wie*/data/**` 를 고른 까닭: `*` 는 `/` 를 넘지 않아 `wie-ktf/tests/data` · `wie-lgt/tests/data` 의 fixture 는 안 걸리고, 다음에 크레이트가 `data/` 표를 더해도 걸린다. dorny/paths-filter 의 매처(picomatch · `dot:true`)로 확인했다:

```
MATCH wie-core-arm/data/binary_patches.toml
MATCH wie-lgt/data/lgt_java_abi.toml
miss  wie-lgt/tests/data/helloworld_lgt.zip
miss  wie-ktf/tests/data/helloworld_ktf.zip
miss  data/x.toml
```

`on.push.paths` 쪽은 GitHub 문서의 같은 규칙(`*` 는 `/` 를 뺀 문자열)에 기대며, 로컬에서 그 매처 자체를 돌릴 수는 없다 — 착지 뒤 표 단독 변경의 첫 착지에서 run 이 서는지가 실물 확인이다.

### §2 과거에 그 구멍에 빠진 착지 — 8건 · 운영 미반영 1건(#608)

`origin/main` first-parent 에서 `wie-core-arm/data` · `wie-lgt/data` 를 건드린 착지 46건을 `publish-artifact.yml` 의 실제 run 목록(334건 · headSha 대조)과 맞췄다. run 이 없는 착지가 8건이고, 8건 모두 그 밖의 필터 줄에 걸리는 파일이 0개다(표 + `docs/report` + `docs/worklog` 류).

| 착지 | 날짜 | 표 | 뒤 릴리스가 실어 간 시점 | 운영(`wieHead` `d65954359`)에 |
|---|---|---|---|---|
| `6e1945908` | 09-22 | `lgt_java_abi.toml` | `cb2e6dca3` · 11.4h 뒤 | 실림 |
| `de4fe883f` | 09-23 | 〃 | `24a920c46` · 3.0h 뒤 | 실림 |
| `e7ef1eda9` | 09-24 | 〃 | `4dea6c03f` · 2.8h 뒤 | 실림 |
| `2e356dfc2` | 09-24 | 〃 | `852143b72` · 4.2h 뒤 | 실림 |
| `81818e84f` | 09-26 | 〃 | `7bac9acd1` · 0.2h 뒤 | 실림 |
| `4d8459195` | 09-27 | 〃 | `96c9f243a` · 0.2h 뒤 | 실림 |
| `e98333c12` | 09-27 | 〃 | `96c9f243a` · 0.1h 뒤 | 실림 |
| `0b50fb735`(#608) | 11-22 | `binary_patches.toml` | 아직 없음 | **안 실림** |

- 운영 값은 `https://featurephone.otterpebble.com/engine/manifest.json` 의 `wieHead` = `d659543599f4…`(2026-11-22 실측)이고, 앞 7건은 그 조상이다(`git merge-base --is-ancestor`). 무해.
- #608 은 `origin/main` 의 머리이고 `d65954359` 의 조상이 아니다 — 티켓이 적은 «알려진 사실» 그대로다. 손으로 릴리스를 띄우지 않았다. 이 PR 의 착지가 싣는다.
- `binary_patches.toml` 단독 착지는 #608 이 처음이다. 앞 7건은 전부 LGT ABI 표의 행 추가였고, 가장 오래 기다린 것이 11.4시간이다.

### §3 다른 워크플로의 필터

| 워크플로 | 필터 | 판정 |
|---|---|---|
| `engine-contract.yml`(`contract` · job 안 paths-filter) | 같은 죽은 `data/**` | 고쳤다 — 표만 바꾼 PR 이 계약 검사를 건너뛰었다 |
| 〃 | `contract-roundtrip.mjs` 가 import 하는 생성기 셋 중 `make-resize-fixture.mjs` 만 없었다 | 한 줄 더했다(`make-draw-fixture` · `make-sound-fixture` 와 같은 까닭) |
| `pr-audit.yml` | `Cargo.toml` · `Cargo.lock` · `**/Cargo.toml` | `cargo audit` 의 입력은 `Cargo.lock` 뿐 — 누락 없음 |
| `doc-liveness.yml` | 자기 파일만(self-test) | 의도 — 누락 없음 |
| `web.yml`(`build-web`) · `rust.yml` | 트리거에도 job 안에도 경로 필터 없음 | 해당 없음 |
| `coverage.yml` · `rust-audit.yaml` · `web.yaml` · `release.yaml` · `tag-release.yaml` | schedule / dispatch 뿐 | 해당 없음 |

### §4 검증

- `actionlint`(1.7.12) 두 파일: rc=1 · 지적 2건은 `publish-artifact.yml:162` 의 shellcheck SC2086(info · `$LICENSES` 의 의도된 단어 분리)이고 `origin/main` 의 파일에서도 같은 2건이다. `-ignore SC2086` 로는 rc=0 — 이 diff 가 더한 지적 0.
- 워크플로를 읽는 repo 검사: `check-doc-liveness-parity` · `check-parked-workflows` · `check-parity-lock-wired` · `check-upstream-guard-wired` 전부 rc=0.
- `check-engine-contract.mjs` 가 `publish-artifact.yml` 에서 읽는 것은 dispatch 페이로드와 자산 이름이고 경로 목록이 아니다 — 두 필터의 «같이 맞춘다»(Constraint 4)를 기계로 잡는 검사는 없다.
