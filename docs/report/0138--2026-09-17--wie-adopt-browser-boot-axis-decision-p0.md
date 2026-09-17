## [2026-09-17] `Screen::resize` 가 «한 번도» 불린 적 없었다 — 크기를 요구하는 픽스처로 처음 덮었다 (wie-adopt-browser-boot-axis-decision-p0)

**무엇을**: `DisplaySize:` 를 선언하는 KTF 픽스처 `test_data/resize_ktf.zip`(1,627B)과 그 **레시피**
`scripts/make-resize-fixture.mjs` 를 만들고, 브라우저 라운드트립에 **Scenario G** 를 붙였다.
★**제품 동작 변경 0** — 제품 코드는 한 줄도 고치지 않았다(개악 대조용 2회는 복원했다).

**왜**: 채택 제안 `2026-09-16-browser-boot-axis-decision#p0`.
「엔진의 `Screen::resize` 산 호출부는 `adf.display_size` 가 `Some` 일 때만 도는데 **커밋된 두 KTF 픽스처
어디에도 `DisplaySize` 줄이 없다** ⇒ 그 호출부와 그 아래 모든 구현이 **0회** 실행됐다. 게다가 그 호출부는
`Err` 를 `tracing::warn!` 으로 삼키므로 실패해도 부팅이 계속된다」.

---

## ⓐ 제안이 «지금도» 참인가 — ★**전건 참이다**(직접 재측)

| 제안의 주장 | 이 회차 실측 | 판정 |
|---|---|---|
| 산 호출부는 `wie_ktf/src/emulator.rs` 하나 | ★**경로만 바뀌었다** — 상류 base swap 으로 `wie-ktf/src/emulator.rs:70`(하이픈) | **참** |
| 조건이 `adf.display_size == Some` | `if let Some((width, height)) = adf.display_size && let Err(error) = platform.screen().resize(..)` | **참** |
| 커밋된 두 KTF 픽스처에 `DisplaySize` 줄 **0** | 두 `__adf__` 전문이 `AID:00000000\nPID:PD000000\nMClass:Clet\n` — ★**38바이트, 동일** | **참** |
| `Err` 를 `warn!` 으로 삼킨다 | 같은 블록 `tracing::warn!("Ignoring unsupported display size …")` | **참** |
| LGT 쪽 호출부는 PR #161 이 끊었다 | `wie-lgt/.../graphics.rs` 의 `resize_presentation` 은 **내부 함수**이고 `Screen::resize` 를 안 부른다 | **참**(이 회차가 덮지 «않는다» — 아래 ⓔ) |

## ⓑ 같은 축이 있는가 — **없다. 그리고 «왜 안 잡혔는지»가 이 결함의 형태다**
`test-utils` 의 TestScreen 은 `resize` 를 구현해 width/height 를 저장하지만 **그것을 읽는 단언이 0** 이고,
`wie_validate` 의 `resize` 는 ★**`Ok(())` 반환뿐인 no-op** 이다. ⇒ ★**계측기는 있는데 입력이 없었다** —
「검사가 없다」가 아니라 «그 코드로 들어가는 입력이 저장소에 없었다».

## ⓒ 어디에 놓았는가 — **브라우저 시나리오**(제안의 자리 그대로)
⒜ 제안의 userBenefit 이 **브라우저**를 지목한다(「앞뒤 두 캔버스가 어긋나도 아무 게이트가 울지 않는다」) ·
⒝ `wie_validate` 의 resize 가 no-op 이라 ★**거기서 돌리면 아무것도 단언하지 못한다** ·
⒞ 실패가 사용자에게 보이는 자리가 캔버스다. ⇒ **Scenario G** 가 부팅 직후 캔버스 크기를 단언한다.
★**픽스처 수정이 아니라 «별 픽스처»** — Scenario E/F 가 현재 기하에 **정확 픽셀**을 걸고 있어 기존 것을 키우면 함께 깨진다(제안이 예고한 그대로).

## ⓓ 산출물이 «유지 비용»을 어떻게 낮췄나
- **레시피가 코드다**: `make-resize-fixture.mjs` 가 `helloworld_ktf.zip` 을 읽어 `__adf__` 에 **한 줄만 덧붙여** 다시 묶는다.
  ★**같은 게스트 jar** 이므로 이 픽스처가 재는 것은 «크기 경로» 하나다.
- ★**바이트 안정**: STORED + 타임스탬프 0(기존 `zip()` 헬퍼 재사용 · `export` 한 단어만 추가) ⇒ 재생성해도 **cmp 동일**.
  `make-wipi-keydraw-fixture.sh` 가 「출력이 재현되지 않는다」고 적은 그 문제가 여기엔 없다.
- ★**수를 restate 하지 않는다**: `RESIZE_W/RESIZE_H` 를 생성기가 **export** 하고 시나리오가 **import** 한다
  (`make-draw-fixture.mjs` 의 기존 관용구). 240x320 → **176x220** 으로 ★**두 축 모두** 다르게 골랐다 —
  한 축만 적용되는 반쪽 resize 가 pass 로 읽히지 않게.
- ★**AGENTS.md `NOT-RUN` 1줄**: `check-engine-runner-fixtures.mjs` 가 새 픽스처를 **실제로 잡아 red 를 냈고**(rc=1),
  그 설계대로 «분류를 문서에» 적어 해소했다(검사기는 분류를 모른다). ★**그 기구의 첫 사용자다**(종전 `excused` 0건).

## ⓔ ★개악 대조(양방향 · **제품 호출부 2곳** · 실브라우저)

| 형상 | 무엇을 망가뜨렸나 | 라운드트립 |
|---|---|---|
| 정상 | — | **rc=0 · 49/49** · `G: … 240x320 -> 176x220 — 176x220` |
| **MUT1** | 엔진 호출부 — `wie-ktf/src/emulator.rs` 의 `adf.display_size` 를 `None` 으로 | ★**rc=1 · 48/49** · `✗ G: … — 240x320` |
| **MUT2** | 브라우저 호스트 — `wie_featurephone/src/screen.rs` 의 `front.set_width/height` 제거 | ★**rc=1 · 48/49** · `✗ G: … — 240x320` |
| 복원 | — | **rc=0 · 49/49** |

★**두 층을 각각 물었다** — 엔진이 안 부르든 호스트가 안 적용하든 같은 red 다. 각 회차마다 **wasm 재빌드 후** 브라우저에서 쟀다.
★**상수 대 상수가 아니다** — 단언의 좌변은 실제 `HTMLCanvasElement.width/height` 이고, 우변은 픽스처 생성기가 쓴 ADF 줄에서 온다.

## ⓕ 잃는 것 — **적는다**
- ★**커밋 바이너리 1개(1,627B)와 유지 대상 1개(생성기)** 가 는다. 제안이 예고한 대가이고, 레시피가 있어 **재생성 가능**하다는 점으로 상쇄했다.
- ★**라운드트립이 부팅을 한 번 더 한다** — 실측 체크 46 → **49**, 로컬 실행에서 체감 증가는 초 단위다(픽스처가 즉시 종료하는 helloworld 게스트라 5초 데드라인 중 1프레임에 끝난다).
- ★★**덮지 «못하는» 것 — 숨기지 않는다**: `WebScreen::resize` 는 **뒤 버퍼도** 옮기는데 그 캔버스는 wasm 안에서 만들어져
  **JS 가 크기를 못 읽는다** ⇒ 뒤 버퍼만 옛 크기로 남는 회귀는 이 시나리오가 **못 본다**(clip 은 throw 를 내지 않는다).
  그리고 **LGT 호출부**(PR #161 이 끊은 27줄)와 `wie_validate`·네이티브 창은 여전히 **미덮음**이다.
- ★**안 하면**: 지금 상태 그대로 — resize 회귀가 **조용하다**. 호출부가 `Err` 를 삼키므로 ★**실패해도 부팅이 계속되고**, 네이티브 게이트는 전부 green 이다.

## ⓖ 검증
`fmt`·`clippy --all -D warnings`·**wasm**·**`+beta`** 전건 **rc=0** · `RUST_MIN_STACK=4194304 cargo test --all` **rc=0**
(`test result: ok` **42줄 · 합산 384 passed · 0 failed**) · `npm run audit` **PASSED**(새 픽스처는 게임 바이너리가 아니다) ·
검사기 `engine-runner-fixtures`(6 tracked · 1 excused)·`doc-liveness-parity`(25줄)·`engine-contract`(109 pass)·
`worklog-json`·`worklog-coverage`·`parked-workflows`·`docs-report-serial`·`parity-lock-wired`·`linux-system-deps` **전건 rc=0** ·
`wie_validate test_data/resize_ktf.zip` → **`"result":"PASS"`**(네이티브에서도 부팅은 정상 — 다만 그쪽 resize 는 no-op).

**사용자 영향**: 게임이 ADF 로 요구한 화면 크기가 브라우저에서 **실제로 적용되는지**가 처음으로 게이트가 된다.
지금까지는 엔진이 그 호출을 빠뜨려도, 호스트가 캔버스를 안 바꿔도 **모든 검사가 green** 이었다.
