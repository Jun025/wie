## [2026-09-27] 전수 점검이 찾은 첫 벽 군집 7개 수정 — 부팅·종료·zip 경로 (wie-full-corpus-playability-census-and-fix)

**무엇을**: 보유 게임 전수 점검(`docs/report/0317`)에서 걸린 게임 수가 많은 «첫 벽» 군집 7개를 고쳤다. 한 군집마다 커밋 하나다.
**왜**: 7개 모두 엔진 쪽 결손이다. 게임은 정상인데 호스트가 죽거나, 없는 메서드·속성 때문에 부팅에서 멈췄다.
**사용자 영향**: 부팅조차 안 되던 게임 **16종**이 화면을 띄운다. 한 색 화면이던 **4종**이 그림을 그린다. 키를 누르면 죽던 **3종**이 버틴다(아래 표).
종료할 때 앱이 죽던 경로도 막았다. 2종은 첫 벽을 넘었지만 다음 벽에서 멈춘다(끝의 «넘었으나 다음 벽»).

증적: `~/orchestrator/reports/evidence/wie-full-corpus-playability-census-and-fix/`(sha↔플랫폼만. 제목·바이트 없음).
전/후 측정: 같은 명령(`scripts/playability-census.mjs run --only probe`)을 같은 표본에 두 번 돌렸다.
- 전: `origin/main`(`aeb09129`) + #347 + #348. 스크래치 `19effc06`.
- 후: 같은 핀 + 이 PR 의 수정. `178e8787` → `920e56fb`. 재루팅 한 건만 디버그 빌드로 쟀다.
- 호스트 부하: load1 **297~415**. 호스트 전체 idle 0%다.

### 군집과 전/후

| # | 군집(첫 벽) | 걸린 타이틀 | 고친 곳 | 전 → 후 |
|---|---|---|---|---|
| 1 | `Player.stop/play/resume(null)` 이 호스트 panic(`ClassInstanceRef` null deref) | KTF 7 | `Clip::player` 가 null 클립이면 null 플레이어를 돌려준다. 세 호출이 이미 «플레이어 없음 = false» 로 처리한다 | 부팅 fail 2 · 한 색 2 · 입력 중 panic 3 → 7종 모두 부팅·화면·조작 ok |
| 2 | `HandsetProperty.getSystemProperty("VOLUMELEVEL")` = `""` → startApp 의 `Integer.parseInt` 가 NumberFormatException | KTF 5 | `"3"` | 부팅 fail 3 · 한 색 2 → 5종 모두 부팅·화면 ok |
| 3 | SKT `new Image()` → NoSuchMethodError `Image.<init>()V` | SKT 5 | 1×1 이미지 생성자. 0×0 은 안 된다 — 읽는 쪽이 모두 `bpl / width` 로 픽셀 폭을 구한다 | 부팅 fail 5 → 5종 모두 부팅·화면·조작 ok |
| 4 | 게임의 `super.getHeight()` = invokespecial `Canvas.getHeight` → NoSuchMethodError | SKT 2 | `Canvas` 에 `getWidth/getHeight` 를 선언하고 `Displayable` 로 넘긴다. 핀의 JVM 이 invokespecial 을 이름 붙은 클래스에서만 찾기 때문이다(JVMS §5.4.3.3 는 상위까지 찾는다 · 후속) | 부팅 fail 2 → ok 1 · 다음 벽 1 |
| 5 | SKT `System.getProperty("m.MODEL"/"m.EXT_SW")` = null → `.equals`·`.length` NPE | SKT 3 | 두 속성에 실제 기기와 겹치지 않는 값. 코퍼스 84종 중 `m.MODEL` 59종 · `m.EXT_SW` 7종이 읽는다. null 을 검사하던 타이틀은 같은 기본 경로를 탄다 | 부팅 fail 3 → ok 2 · 다음 벽 1 |
| 6 | 게임이 끝날 때 호스트 스택 넘침(abort) | KTF 4 | `Jlet.notifyDestroyed` 가 `destroyApp` 를 되부르고 있었다. 게임의 `destroyApp` 는 `notifyDestroyed()` 로 끝나므로 무한 재귀가 된다. MIDP 에서 `destroyApp` 는 플랫폼이 부른다. 그래서 되부르기를 뺐다. `MIDlet.notifyDestroyed` 는 스텁이었는데, 이제 `MC_knlExit`·`System.exit` 과 같은 종료 경로를 탄다 | 전수 1회차 abort 4 · 재실행은 키 타이밍 따라 재현이 갈린다 — 판정은 단위 시험으로 |
| 7 | zip 안 게임 폴더가 2~3단 깊이, 또는 루트에 스크린샷이 같이 있음 → `unrecognized zip archive` | KTF 3 | `extract_zip` 이 표식(`__adf__`·`app_info`·`*.msd`)이 든 **유일한** 디렉터리로 재루팅한다. 루트에 표식이 있거나 후보가 둘이면 손대지 않는다 | 로드 실패 3 → 부팅 3(디버그) |

«후» 는 전수 429종을 수정 빌드(`920e56fb`, 재루팅 전)로 한 번 더 잰 값이다. 개선 26건 중 20건이 위 군집이다. 나머지 6건은 조작 축이 한 번 재서 뒤집힌 것이다(0317 §측정의 한계).

**퇴행 0**: 전수 비교에서 후가 나빠 보인 12건은 모두 조작 축이거나, 원래 흔들리던 1건(한 색 화면)이다. 12건 중 10건을 전·후 빌드로 같은 시각에 짝지어 다시 쟀다(load1 약 300). 후가 전보다 나쁜 축은 **0** 이었다. 나머지 2건(`5028b8a5d19f`·`6e93f26fa2f5`)도 전·후 빌드 짝 재측에서 같은 결과였다.
라이브 LGT 5종과 티켓이 지정한 퇴행 가드 2종(`49ade89578c5`·`ddd885583b15`)은 전·후 모두 부팅·화면 ok 다.

**동작이 바뀐 곳 하나(의도)**: 라이브 LGT `4ece6eeeaa04` 는 이제 27키 프로브의 8번째 키에서 끝난다(clean exit).
커서가 메뉴 «게임종료»에 있을 때 OK 가 눌리기 때문이다. 전에는 종료가 스텁이라 아무 일도 없었다. 전·후 빌드 각 2회 모두 같았다.
셸은 `has_exited()` 로 이 상태를 본다. 종료 후 화면 처리는 셸 소관이다.

### 넘었으나 다음 벽
- `f6fe2adc8cce`(skt): getHeight 는 넘었다. 그다음 한 tick 이 끝나지 않는다(census kill 150s).
- `66959afab216`(skt): 속성은 넘었다. 다음 벽은 `RecordStore.enumerateRecords(RecordFilter,RecordComparator,Z)` 부재다(MIDP RMS · 후속).

### 되돌리면 red
| 커밋 | 시험 | 변이 |
|---|---|---|
| null 클립 | `wie-wipi-java player::test_null_clip_reports_failure` | 가드 제거 → panic |
| VOLUMELEVEL | `handset_property::test_volume_level_is_a_number` | 행 제거 → FAILED |
| `new Image()` | `image::tests::no_arg_image_is_one_pixel` | 행 제거 → FAILED |
| Canvas getHeight | `canvas::canvas_super_size_resolves_on_canvas` | 행 제거 → FAILED |
| m.MODEL · m.EXT_SW | `wie-skt emulator model_and_ext_sw_properties_are_set` | 행 제거 → FAILED |
| notifyDestroyed | `wie-midp midlet::notify_destroyed_exits` · `wie-wipi-java jlet::notify_destroyed_exits_without_destroy_app` | exit 제거 / destroyApp 복원 → FAILED |
| 재루팅 | `wie-backend extract_zip_reroots_a_nested_game` | 호출 제거 → FAILED |

### 게이트 · 유입
- `cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` 전부 rc=0.
- `scripts/build-wasm.sh` rc=0 · `check-engine-contract.mjs` 109 pass / 0 violation · `npm run audit` PASSED.
- 게임 파일명 유입: 이 회차가 **더한 줄**에는 0건이다(`git diff origin/main` 의 `+` 줄 기준).
- 도구 표기는 BOUNDED 4회/2쌍 · SUFFIX-ATTACHED 1회/1쌍이다. 모두 이 회차가 손댄 파일에 **이미 있던** 주석 3곳이다(`jlet.rs` 1 · `player.rs` 3).
