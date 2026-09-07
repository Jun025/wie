## [2026-09-06] richness 3축에 «마지막 프레임» 짝을 붙였다 — 보고까지, 임계 0 (wie-validate-richness-three-axes-lack-last-frame-pair)
- **무엇을**: `wie_cli/src/bin/wie_validate.rs` 에 `last_frame_distinct_colors`·`last_frame_nondominant_pct`·`last_frame_center_nonuniform_pct` 3필드. ★**`frame_richness` 를 마지막 프레임에 run 당 «1회»** 더 돌린다(프레임당이 아니다 — 그 프레임은 이미 메모리에 있다). ★**보고 전용 · 임계 0 · `passed` 무접촉.**
- **왜**: 운영자 채택 제안 `2026-09-06-validate-last-frame-axis#p1`. 세 축이 전부 `fetch_max` 라 **단조**이고 ⇒ 나중 프레임이 값을 되돌릴 수 없다 = `saw_content` 와 **구조적으로 같은 사각**. 직전 회차는 가장 거친 축(2색 이상)에만 짝을 만들었다.
- **★⑴ 대전제를 먼저 재측했다**(반증 실패 = 진행): `git show origin/main:… | grep -n 'fetch_max'` → `:185 max_magenta_px` · `:188 distinct` · `:189 nondominant` · `:190 center` ⇒ 세 축 전부 `fetch_max` 맞다.
- **★★⑵ 짝이 «개악 없이도» 이미 갈린다**: 무개악 `keydraw_{ktf,lgt} --inject` 에서 ★**max nd=2.1 ↔ last nd=1.1**. 근인은 키를 **오름차순**으로 눌러 「가장 넓은 막대」가 MAX 에 남지만 **마지막 프레임에는 마지막 키의 막대만** 있다는 것 ⇒ ★**가상의 위험이 아니라 현행 픽스처에서 이미 값이 다르다.**
- **★★⑶ Acceptance — max 는 높은데 last 만 무너진다**(개악 = 실제로 일어났던 LGT 검은 화면 회귀):

  | 형상 | max c/nd | last c/nd | result |
  |---|---|---|---|
  | E0 무개악 `keydraw_lgt --inject` | 2 / **2.1** | 2 / **1.1** | PASS |
  | **M1 개악** 같은 픽스처 | 2 / **2.1** ★불변 | ★**1 / 0.0** | PASS |
  | 대조군 `keydraw_ktf --inject`(M1 하) | 2 / 2.1 | 2 / 1.1 ★불변 | PASS |

  ★**`result` 가 두 형상 다 PASS 인 것이 «설계»다** — 계약 3 대로 **보고까지**이고 임계를 두지 않았다(막는 것은 형제 티켓 몫).
- **★⑷ 소비자를 «세고» 회귀를 실행으로 확인했다**(계약 2): JSON 을 실제로 파싱하는 소비자는 ★**1개**(`scripts/lgt_render_probe.sh:48`). 그 `field()` 를 그대로 돌려 ★**`field distinct_colors` = 2**(= MAX · 신규 필드 오염 **없음**) 확인 — 지켜 준 것은 **패턴의 여는 따옴표**다. `smoke_gate.sh` 는 `result` 만 읽어 소비자가 아니다.
- **★⑸ 회귀 0**: 5픽스처 판정·기존 `max_*` 값 **전건 불변** · `cargo test --all` **156 passed / 0 failed** · beta clippy 0.
- **사용자 영향**: 없음(보고 필드). 대신 「그렸다가 거의 빈 화면으로 덮였다」가 **수로** 보인다 — 지금까지 그 자리를 보는 것은 `last_frame_content`(2색 이상)라는 가장 거친 술어뿐이었다.
- **★남는 구멍**: ⒜★**막는 회귀는 0건**이다(임계가 없으므로 M1 에서도 PASS) — 값은 「다음 회차가 임계를 «데이터로» 고를 수 있다」에 있다(제안 등재) ⒝★`center_nonuniform` 은 현행 픽스처 전건 **0.0** 이라 ★**짝의 값이 아직 증명되지 않았다**(중앙에 그리는 픽스처가 없다) ⒞`max_magenta_px` 는 짝을 **일부러 안 만들었다** — 그 축은 「어느 프레임에서든 샜나」라 **MAX 가 옳은 술어**다 ⒟신규 3필드에 단위 시험 없음(새 로직이 아니라 기존 `frame_richness` 를 한 번 더 부르는 배선이다).

## [2026-09-06] `Image.createImage(String)` 픽스처 — «넓어진 가시 범위가 무엇을 찾는가»를 수로 냈다 (wie-system-class-loader-createimage-fixture)
- **무엇을**: `scripts/make-draw-fixture.mjs` 가 jar 에 `wie-img.png`(16×8 RGB · 74바이트 · 스크립트가 바이트로 조립)를 동봉하고, `DrawMIDlet.startApp()` 이 **`Image.createImage("/wie-img.png")`** 로 그것을 **이름으로** 연 뒤 `getWidth()`·`getHeight()` 를 정적 필드에 저장하며, `DrawCanvas.paint()` 가 **그 치수 그대로** 사각을 채운다. `scripts/contract-roundtrip.mjs` 에 Scenario C-img 1건. `.rs` 변경은 **주석뿐**.
- **왜**: 운영자 채택 제안 `2026-09-05-system-class-loader-preemptive-migration#p1`. 그 원문이 미완으로 남긴 문장이 이 회차의 Acceptance 다 — 「갈림 «자체»는 측정됐다 … ★**미측정인 것은 «넓어진 가시 범위가 실제로 무엇을 찾는가»** 하나다」.
- **★★⑴ 산출이 «수»다**(계약 5): 칠해진 픽셀 수가 곧 **호스트가 찾아 디코드한 이미지의 픽셀 수**라, 왕복이 `1024 + 128 = **1152**` 를 ★**등호로** 단언한다(「돌아간다」가 아니다). 상수는 픽스처가 단일 출처로 소유하고 왕복은 `page.evaluate` 인자로 받아 **재진술 0**.
- **★★⑵ 커버 전/후를 «같은 프로브»로 쟀다**(`image.rs` 그 줄에 `panic!()`): ★**전 — `cargo test --all` rc=0 · 156 passed / 0 failed · 5픽스처 전건 PASS**(`draw_j2me` nondom **1.3%**) ⇒ 아무것도 그 자리를 지나지 않았다(제안의 주장을 독립 재현) ↔ ★**후 — `draw_j2me.jar` FAIL · `panic during 'boot'` · ticks 0**. ⇒ ★**0 → 1.**
- **★★⑶ 양방향 개악**(계약 4 · 매번 원본 복원 후 실행): **E0** 무개악 → 왕복 ★**42/42 rc=0** · `wie_validate` PASS(nondom **1.5%**) / **E1** `get_system_class_loader` → `jvm.current_class_loader()`(이행 전 형태) → ★**FAIL · `java.io.IOException: Resource not found: /wie-img.png`**(스택 `createImage(String)` ← `startApp`) ⇒ ★**넓어진 범위가 찾는 것 1건 ↔ 종전 경로 0건** / **E2-b** 전달 바이트 절반 절단 → ★**`IllegalArgumentException: Failed to decode image`** ⇒ 바이트도 하중을 받는다.
- **★⑷ 음성 결과를 숨기지 않는다**: **E2**(바이트 «1개» 절단)는 ★**물지 않았다** — PNG 디코더가 꼬리 1바이트 결손을 견딘다. ⇒ 이 픽스처가 잠그는 것은 «전 바이트 무결»이 아니라 **「이름이 풀렸고 그 결과가 16×8 로 디코드된다」**이다.
- **★⑸ 거짓이 된 서술을 정정했다**(그 두 곳만): `image.rs` 의 「NOT covered by any fixture」 · `docs/upstream-realign-verdict.md` §8-4⑶-b 의 「남은 미커버는 6번 하나」(→ **0곳**). ★**형제 회차와 같은 형태로 «결론 줄을 다시 쓰지 않고» 정정 블록을 덧댔다**(이력 보존).
- **사용자 영향**: 없음(시험). 대신 「이미지를 이름으로 못 불러온다」류 회귀가 **커밋 전에** 잡힌다 — 6곳 중 **유일하게 동작이 달라진 칸**인데 그물이 0이었다.
- **★남는 구멍**: ⒜**실패 갈래 미커버** — 없는 이름·깨진 이미지 경로는 **개악으로만** 지나갔다. 픽스처 어셈블러가 **예외 테이블을 내지 않아** 게스트에 try/catch 를 쓸 수 없는 것이 실제 비용이다(제안 등재) ⒝★**증거 축이 브라우저 왕복과 `wie_validate` 뿐**이다 — `cargo test --all` 은 여전히 J2ME 게스트를 부팅하지 않으므로(AGENTS.md 가 적은 그 사각) 이 커버는 PR 의 `contract` 잡에 의존한다(제안 등재) ⒞커버 = «실행된다»이지 «규격에 맞다»가 아니다 ⒟상용 코퍼스 0(Constraint 9).
## [2026-09-06] `Security audit` 3일 red 를 껐다 — `rtrb` 0.3.3 → 0.3.5 (wie-rustsec-2026-0274-rtrb-double-free-audit-red)
- **무엇을**: `Cargo.lock` **2줄**(`rtrb` version + checksum). `cargo update -p rtrb` 한 번. ★그 밖의 크레이트 이동 **0** · `Cargo.toml` 무접촉.
- **왜**: 매일 도는 `Security audit`(★`schedule` 전용 — PR 게이트가 아니라 머지를 막은 적은 없다)이 3일 연속 `error: 1 vulnerability found!`. 자문 = `RUSTSEC-2026-0274`(`ReadChunk::commit` 에서 **원소의 `Drop` 이 panic** 하면 double free / UAF).
- **★★⑴ 「목록에 있다」와 「그 경로를 탄다」를 갈랐다 — 세 축을 «수»로**: ⒜우리 `.rs`/`.toml` 의 `rtrb|ReadChunk` ★**0건**(`wie_cli/src/main.rs:26` 이 쓰는 rodio 표면은 `DeviceSinkBuilder`·`Player`·`SamplesBuffer`·`SampleTypeConverter` = **재생 전용**) ⒝rodio 가 rtrb 를 쓰는 곳은 `src/microphone.rs` **한 파일뿐**인데 그 파일의 `read_chunk|ReadChunk|.commit` ★**0건**(실사용은 `pop()`·`slots()`) ⇒ ★**취약 함수가 애초에 호출되지 않는다** ⒞원소 타입 `rodio::Sample = Float = f32`(`common.rs:31,43`) = ★**`Drop` 구현 없음** ⇒ 자문의 전제가 **구조적으로 성립 불가**.
- **★⑵ 처방 = ⒜(버전 상향)**: ⒞(`cargo audit` 예외)는 ★**선택지가 아니었다** — Constraint 5 가 「no ignores」를 잠근다. ⒝(rodio bump)는 **불필요** — rodio 요구가 `^0.3.2` 라 0.3.5 가 이미 in-range. ★이 형태는 `rust-audit.yaml` 주석이 기록한 **2026-07-31 `cargo update -p wayland-scanner`** 선례 그대로이고, 그때처럼 **suppression 을 남기지 않는다**.
- **★⑶ 검증**: `cargo audit` ★**rc=0**(남은 2건은 비-게이트 allowed warning) · `cargo build -p wie_cli` 0 · 네 게이트 전건 0(**156 passed / 0 failed**) · ★`cargo +beta clippy --all -- -D warnings` **0** · `wie_validate` `draw_j2me`·`helloworld_{ktf,lgt}` ★**전건 PASS**(+ `keydraw_* --inject` PASS · `last_frame_content=true`).
- **사용자 영향**: 없음. 에뮬레이터 동작은 한 비트도 바뀌지 않는다(오디오 재생 경로는 rtrb 를 지나지 않는다).
- **★남는 구멍**: ⒜★**실보안 이득은 0에 가깝다** — 위 세 축이 전부 「안 탄다」다. 값은 daily red 를 끈 것과, rodio 가 나중에 `read_chunk` 를 쓰더라도 이미 패치판이라는 것뿐이다 ⒝⑴⒝ 의 근거는 **`rodio 0.22.2` 라는 «지금 그 판본»의 소스 실측**이라 rodio 를 올리면 다시 재야 한다 ⒞`cargo audit` 는 `Cargo.lock` 만 본다 — 「그 코드를 실행하는가」는 이 회차가 **손으로** 답했고 기계가 잠그지 않는다 ⒟`rust-audit.yaml` 주석의 「spin 0.12.0 yanked」는 실측(`chacha20 0.10.0`)과 다르지만 **워크플로 변경 0** 이라 고치지 않았다(제안 등재).
## [2026-09-06] 「감시를 지웠는데 green」을 저장소 전체에서 세었다 — 「다섯」은 출처가 없고 실측은 8+3 이다 (wie-count-deletable-checks-that-stay-green-repo-wide)
- **무엇을**: ★**세기만 했다 — 가드 0 · 코드 0 · 워크플로 무접촉.** 산출물은 `docs/worklog/2026-09-06-deletable-checks-census.json` 과 이월된 「다섯 번」 3자리의 인라인 정정뿐이다.
- **왜**: 운영자 채택 제안 `2026-09-06-parity-lock-self-deletion-guard#p0`.
- **★⑴ 술어와 수**: 「워크플로 스텝이 «경로로» 부르는 검사 파일 집합 **A**」 ↔ 「`scripts/`·`*/tests/` 에 실재하는 파일 집합 **B**」의 차집합. ★**A=9 · B=23 · B\A=14.** 재현은 `git grep` 두 줄이고 워크로그 `sets.reproduce` 에 그대로 실었다.
- **★★⑵ 14를 전건 분류했다 — 「지워도 green」은 «한 형태»가 아니라 «두 형태»였다**:
  ⒜★**「CI 에서 돌고 있는데 지워도 green」 8건** — 전건 rust 통합시험. `cargo test --all`·`tarpaulin --workspace` 가 **glob 으로 줍고** 워크플로가 이름을 부르는 자리가 **0** 이다.
  ⒝★**「애초에 CI 에서 안 도는 검사」 3건** — `audit-no-leak.sh`·`verify-browser.mjs`·`smoke_gate.sh`. ★**지울 필요도 없다. 이미 안 돈다.** 특히 `audit-no-leak.sh` 는 Constraint 9·10 의 «기계 절반»인데 ★**어느 워크플로도 npm 스크립트도 부르지 않는다**(전수 0건).
  ⒞검사 아님 2건(`lgt_render_probe.sh` 측정 하네스 · `smoke_gate_baseline.tsv` 데이터) ⒟★**술어의 오탐 1건** = `wie_cli/tests/support/dod_ci_parity.rs` — 직전 회차가 만든 `check-parity-lock-wired.mjs`(집합 A 원소)가 이 경로를 물어 red 가 된다 ⇒ ★**그 가드가 여기서 작동을 증명했다.**
- **★★⑶ 분류를 «주장»이 아니라 «실행»으로 냈다**: 격리 워크트리에서 ⒜의 하나(`wie_jvm_support/tests/absent_timer_schedule.rs`)를 **실제로 지우고** `RUST_MIN_STACK=4194304 cargo test --all` → ★**기준선 rc=0 · 40줄 · 156 passed · 이름 1회** ↔ ★**삭제 후 rc=0 · 39줄 · 155 passed · 이름 «0회»**. ★**커밋 0 · 워크트리 제거.** 같은 트리에서 `cargo fmt` rc=0 · 두 node 가드 rc=0.
  ★**커버리지 게이트도 못 잡는다**: `codecov.yml` 이 **0바이트**(Constraint 2 가 «일부러 비워 둔다»고 적은 그것)라 임계가 없다 — `fail_ci_if_error: true` 는 업로드 오류용이다.
- **★★⑷ 회차를 낳은 수가 «출처 없음»이었다**: 「이 저장소에서 다섯 번 났다」는 `2026-09-05-dod-ci-parity-checker.json` 에서 처음 나와 두 곳으로 인용됐을 뿐 ★**다섯 자리를 열거한 곳이 없다.** 저장소가 실제로 센 「five times」는 `AGENTS.md:234,255` 의 **셀프머지 5건**(다른 형태)이다 ⇒ ★**다른 대장 항목에서 빌려 온 수로 보인다.** 세 자리에 상호참조 정정을 붙였고 **원문은 사료로 보존**했다.
- **사용자 영향**: 없음(조사·문서). 대신 총괄이 발권 계획을 세울 «수»가 생겼다.
- **★남는 구멍**: ⒜★**술어가 «파일»을 보지 «모듈»을 안 본다** — 인라인 `#[cfg(test)]` 를 가진 `.rs` 가 **31개** 더 있어 진짜 모집단은 8보다 크다(다만 `mod tests` 삭제는 소스 diff 라 더 눈에 띈다) ⒝스텝«까지» 지우면 A 원소도 사라진다(무한 후퇴 — 직전 회차가 이미 적었다) ⒞`web/`·`functions/` 는 술어 밖이다(현재 시험 파일 **0건** ⇒ 손실 없음).
- **★권하지 않는 것**: ⒜의 8건에 **각각 가드를 다는 것**. 직전 회차의 가드가 정당했던 이유는 락이 **2파일 구성**이고 **CI 자신을 감시**하기 때문이며, 보통의 시험 1건 삭제는 **PR diff 에 그대로 보인다**. ⇒ ★**값하는 자리는 ⒝의 3건**이다.

### [2026-09-07] 착지 충돌을 «저작»으로 해소했다 (wie-validate-richness-three-axes-lack-last-frame-pair-fix)

- **무엇을**: `origin/main`(behind **116**)을 당겨 `wie_cli/src/bin/wie_validate.rs` 의 충돌 **2구간**을 판단으로 해소하고,
  순서를 잠그는 시험 **1건**을 넣었다. 원장 2파일은 합집합 · ★이 회차 기록은 `REPORT.md` 가 아니라 **이 파일**로 옮겼다
  (2026-09-07 이관 규약 — 그 착지 시점에 이 PR 은 이미 열려 있었다).
- **왜 «기계적 합집합»이면 안 됐나**: 두 구간이 **성격이 달랐다**.
  ⒜**모듈 헤더 doc** — 양쪽이 «같은 문장»을 서로 다르게 고쳤다. `origin/main`(#96)이
  `last_frame_content is REPORT-ONLY and deliberately not a gate` → `REPORT-ONLY BY DEFAULT …` 로 **일부러** 고쳤으므로
  ★**옛 문장을 되살리지 않았다**. 합집합이었다면 착지본이 「deliberately **not** a gate」와 「`--expect-last-frame` 이면 gate」를
  **동시에** 주장했을 것이다. 검산: 착지본에서 `deliberately not a gate` ★**0건**.
  ⒝**`run()` 꼬리** — 삽입 «순서»를 정해야 했다.
- **★순서의 근거는 «컴파일»이 아니라 «읽는 값이 채워져 있는가»다**(코드로 짚었다):
  게이트 `last_frame_gate_fails(expect_last_frame, passed, last_frame_content)` 는 **bool 셋만** 받고 richness 를 **읽지 않는다**.
  `passed` 는 `pass()`/`fail()` 이, `last_frame_content` 는 바로 윗줄이 채운다 ⇒ ★**오늘은 두 순서가 «교환 가능»하다**.
  ⇒ 그래서 **기록(richness 3필드) → 판정(gate)** 으로 뒀다: 사실을 다 채운 뒤에 판단이 오게 하면,
  ★**게이트를 넓혀 richness 를 읽게 만드는 다음 편집이 «0 을 읽는» 갈래로 빠지지 않는다**(그 편집은 컴파일도 실행도 된다).
- **★시험으로 잠갔다** — `richness_is_recorded_before_the_gate_judges_test`.
  ★**소스 순서를 읽는 시험이고 그것이 «의도»다**: 두 블록이 오늘 교환 가능하므로 ★**어떤 행위 시험도 두 순서를 구별하지 못한다**
  (구별하는 척하는 시험이 더 나쁘다). 바늘은 `concat!` 로 갈라 **자기 자신을 매치하지 않게** 했고,
  유일성을 `assert_eq!(count, 1)` 로 먼저 세어 «엉뚱한 위치 비교»를 배제한다.
  ★**양방향 실증**: 순서를 되돌리면 **FAILED**(「the gate is consulted before the richness fields are filled」) · 원복하면 **ok**.
- **사용자 영향**: 없다(보고 필드 · 임계 0 · `result` 판정 불변). 이 회차는 착지 가능하게 만든 것이 전부다.
