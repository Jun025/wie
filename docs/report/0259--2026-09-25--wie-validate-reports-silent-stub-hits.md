## [2026-09-25] 검증기 JSON 에 `stub_hits` · `svc_stub_slots` — «스텁 위의 PASS» 를 보이게, 스텁 공간 고갈 abort 에도 결과 줄을 먼저 (wie-validate-reports-silent-stub-hits)

**무엇을**: `wie_validate` 의 JSON 한 줄에 두 필드를 더했다 — `stub_hits: {count, distinct, first:[{name,count}] 상위 5}` 와
`svc_stub_slots: {used, capacity}`. SVC 스텁 공간이 바닥나면 그 순간 결과 줄(FAIL)을 먼저 쓴다. 판정 로직(PASS/FAIL/UNMEASURED)은 무변경.

**왜**: 경고만 남기고 기본값을 돌려주는 스텁 위에서 지나간 PASS 가 «진짜 PASS» 와 구별되지 않았다. 스텁 공간 고갈은 LGT 에서 JSON 없이 죽는다.

**사용자 영향**: 없음(검증기 출력 필드 추가 · 엔진 동작 무변경). 트리아지 하는 쪽이 줄 하나로 «어느 스텁에 기대 지나갔나» 를 본다.

## 1. «스텁» 판별 = 이미 있는 규약

새 규약을 만들지 않았다: **WARN 레벨 · 메시지가 `stub ` 으로 시작**(target 무관). `warn!(` 343곳 중 278곳이 이 모양이다.
이름 = `stub ` 뒤에서 `(` 또는 공백 전까지.

**세지 않는 것**(의도): `debug!("stub …")` 8곳(lwc/lcdui 생성자 — 일부러 강등된 것) · `stub` 이라고 쓰지 않은 경고
(LGT `stdlib.rs` 의 `unk2/3/4` · skvm `unsupported com.xce.io.XFile::…` · KTF `Unknown {name}`). 편입하려면 문구를 `stub …` 로 바꾸면 된다.

**한계**: 이 규약은 «근사 구현» 도 `stub` 이라 부른다(`Font::stringWidth` 는 실제로 폭을 잰다). 그래서 `count` 는 거친 신호이고 읽을 것은 `first` 의 이름이다.

## 2. SVC 스텁 공간

`ArmCore::make_svc_stub` 이 16바이트짜리를 64 KiB 에서 잘라 쓴다 = **4,096칸**. `wie_core_arm::svc_stub_high_water()`(프로세스 전역 최고 수위) + `SVC_STUB_CAPACITY` 를 줄을 쓸 때 읽는다. ARM 코어가 없으면(J2ME) `capacity: null`.

**처음 설계(스텁 등록 trace 에 필드를 싣고 layer 가 TRACE 로 듣기)는 버렸다 — 측정이 기각했다.** 한 target 이라도 TRACE 를 켜면
`tracing` 의 전역 레벨 힌트가 올라가고, 그것만으로 `keydraw_lgt --inject` paints 가 **main 45/34/41 ↔ TRACE판 10/10/15**,
TRACE 지시만 뺀 판 **51/34/48** 이 됐다(같은 부하 · 교대 실행). 최종판(카운터)은 CPU/run **main 0.90/0.92/0.90 s ↔ 0.95/0.91/0.92 s**, paints 21/21 · 20/19 · 20/20 — 동급.

## 3. 고갈 경로 — «JSON 을 먼저» 

스크래치로 한도를 1,000칸으로 낮춰 두 캐리어를 돌렸다(커밋 안 함):

| | 변경 전 | 변경 후 |
|---|---|---|
| KTF `helloworld_ktf` | `unwrap()` 패닉 → `catch_unwind` → FAIL 줄 1개 · rc 1 | FAIL 줄 1개 · rc 1 |
| LGT `helloworld_lgt` | 재귀 → `stack overflow, aborting` · **rc 134 · 줄 0개** | **FAIL 줄 1개** · rc 134(abort 유지) |

방식: 고갈 시 `make_svc_stub` 이 INFO 이벤트 1개를 낸다(반환값·동작 무변경). layer 가 그걸 보면 **1회만** 결과 줄을 쓴다.
`main` 은 끝까지 살아남아도 자기 판정이 같은 FAIL 이면 두 번째 줄을 쓰지 않는다 — 트리 내 파서(`smoke_gate.sh`·`lgt_render_probe.sh`)는 첫 매치를 쓰고
매치 2건이면 경고하며, `game-lab-recensus.sh` 는 stdout 전체를 `json.loads` 한다. 판정이 다를 때만(관측 0) 두 줄을 내서 그 경고가 울리게 했다.
INFO 는 `jvm::jvm` 이 이미 켜는 레벨이라 §2 의 비용이 없다.

## 4. 소비 지점 시험 · 개악

`wie_cli/tests/validate_stub_hits.rs` — 바이너리를 실행해 JSON 줄을 읽는다(커밋된 픽스처만):
`helloworld_ktf.zip` → `stub_hits.count ≥ 1` · `Font::<init>` 이름 · `used ≥ 1` · `capacity 4096` / `draw_j2me.jar`(★-fix: `draw_j2me.zip` 에서 시험 안에 추출 — 트리에는 jar 가 없고, 개발 트리의 것은 `make-draw-fixture.mjs` 가 남긴 git-ignored 사본이라 초판 대조군은 CI·깨끗한 체크아웃에서 읽기 오류로 «아무것도 안 돌았다». 이제 J2ME 가 실제로 돌아 `stub_hits.count ≥ 1` 을 내고) → `capacity: null` / `draw_j2me.zip`(로드 실패) → 전부 0.

- 개악 ① `stub_hits.record` 끄기 → `a KTF boot hits the Font stubs: {"count":0,…}` **red**
- 개악 ② `SVC_STUBS_HIGH_WATER.fetch_max` 끄기 → `KTF binds SVC stubs: {"used":0,"capacity":null}` **red**

## 5. 코퍼스 1패스 (`game_lab/{working,broken}` 475개 · 디버그 빌드 · `--timeout 8` · 병렬 8 · 부하 110~190)

| | 1회차 | 2회차(최종 바이너리) |
|---|---|---|
| PASS | 57 | 50 |
| **PASS 중 `stub_hits.count > 0`** | **57 (100%)** | **50 (100%)** |
| `stub_hits.count > 0` 전체 | 172 | 180 |
| 줄 2개 이상 | 0 | 0 |
| 고갈 | 0 | 0 |
| JSON 없음 | 2 (KILL 1 · rc −6 1) | 2 (같은 둘) |
| `svc_stub_slots.used` 최대 / 중앙 | 2,443 / 948 | 3,071 / 928 |

- **«스텁 위의 PASS» 규모 = 전부다.** 스텁을 안 치고 PASS 한 타이틀은 0. 상위 이름: MIDP `Font::<init>`(153) · `Font::getDefaultFont`(143) · `Display::getDisplay`(86) · `BackLight::on`(47) · `MC_knlSetSystemProperty`(39).
- 스텁 0 이면서 부팅한(ticks>0) 실행은 284건 있다(전부 FAIL — 대부분 no frame). 두 필드가 갈리는 한 쌍(재베이스 후 main 위 실행):
  `saveItem` 경로 타이틀(LGT · `--inject`) `stub_hits.count 4 · distinct 4 · svc used 1716/4096` ↔ KTF no-frame 타이틀 `count 0 · used 710/4096` ↔ J2ME `capacity: null`.
- **`wec.SYSTheme::saveItem` 은 현 코퍼스 어느 실행에서도 적중하지 않았다** — 그 경로의 유일한 타이틀이 아직 앞 벽(`Unimplemented … vtable index`)에서 멈춘다. 적중하면 이 필드가 그 이름을 낸다.
- PASS 57→50 은 부하다: 뒤집힌 25건이 양방향(9 FAIL→PASS · 16 PASS→FAIL)이고, 16건을 main 바이너리로 재실행하면 main 도 12건 FAIL · 남은 불일치 1건은 재시도에서 재현 0.
- **JSON 없음 rc −6 1건은 이 회차 범위 밖이다**: 스텁 공간 고갈이 아닌 별도 stack overflow abort 이고, **origin/main 바이너리도 같다**(2/2 rc 134).

## 6. 게이트

fmt · clippy(`-D warnings` / wasm / beta) ✅ · `RUST_MIN_STACK=4194304 cargo test --all` **444 passed · 0 failed**(origin/main `802652db` 위로 재베이스 후 · #281 `make_shared_svc_stub` 도 `make_svc_stub` 을 거치므로 계수에 든다) · 러너 블록: draw/helloworld 3종 PASS · text PASS ·
`keydraw_*` 는 부하 120~190 에서 main·본 브랜치 둘 다 FAIL/PASS 가 섞인다(교대 측정 · 위 §2 CPU 동급).

게임 파일명 유입: `corpus-name-inflow.mjs`(이 회차 변경 파일 경로 인자 모드 · 표식 없음) **BOUNDED 0 · SUFFIX-ATTACHED 0**.
