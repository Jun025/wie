# STATE

> 실측 기준일 **2026-08-27**(`wie-upstream-realign-verdict`).
> ★★**이 회차로 이 repo 의 «위치»가 바뀌었다 — 위 절들보다 이 줄을 먼저 읽어라.**
> `origin/main`(`250d7e4c`)은 `upstream/main`(`dlunch/wie` `73938944`)보다 **1,067커밋 뒤**이고
> **192커밋 앞**이며, 공통조상 `fa641a8a` 는 **2026-06-10 이후 한 번도 움직인 적이 없다**
> (우리 192커밋에 upstream 동기화 시도 **0건** — 실측).
> 판정 정본 = **`docs/upstream-realign-verdict.md`**. 채택 갈래 = ★**⒟ — fork 를 배포·제품
> 오버레이로 남기고 엔진은 upstream 을 쓴다.** 집행은 이 회차가 하지 않는다(P1~P4 초안은 `REPORT.md`).
> ★**직전 갱신 시점의 `## 진행중` 1건(PR #65)은 착지**했다(squash `250d7e4c`) — `## 완료` 로 옮겼다.
> ★**`## 다음` 이 낡으면 이 레인은 굶는다** — 착지할 때마다 갱신하라(`AGENTS.md` §Session Discipline).
> ★★**그리고 «새로 쓰는 항목»도 diff 를 열고 써라** — 초판의 ①②는 PR 제목·개설일만 보고 작성돼
> **둘 다 사실과 어긋났다**(게이트② 반려). 낡음을 지운 자리에 새 부정확을 심으면 병은 그대로다.
> ★★★**`## 진행중` 과 「열린 PR 0건」 실측이 어긋나면 «절이 낡은 것»이다 — 착지 즉시 `## 완료` 로 옮겨라.**
> 2026-08-19 게이트②가 잡은 형태가 그것이다: 이미 머지된 PR 이 `## 진행중` 에 남아 바로 아래의
> 「열린 PR 0건」 줄과 서로를 반증했고, 앞줄만 읽은 총괄은 **죽은 `-merge` 를 발권할 수 있었다.**

## fork 의 현재 위치 (2026-08-27 실측 · 정본 `docs/upstream-realign-verdict.md`)

| 축 | 값 |
|---|---|
| 공통조상 | `fa641a8a` **2026-06-10** (이후 **불변**) |
| behind / ahead | **1,067** / **192** (★2026-09-03 재측 **1,089** / **194** — `origin/main` `ec1b7027` ↔ `upstream/main` `6cafdb0e` · ★**이 수는 조회 시각과 함께 읽어라**) |
| 우리 고유 변경 | 220파일 **+25,690/−164** (문서·CI 제외 시 148파일 +17,890/−127) |
| ① 중복(upstream 이 자기 구현으로 착지) | Rust **3,437줄 = 우리 Rust 의 64.3%** |
| ② 엔진 고유·가치 | Rust 1,134줄 (21.2%) |
| ③ 로컬 스캐폴딩(웹 제품·문서·CI) | **20,291줄 = 전체의 79.0%** |
| LGT Java import `0x64` 구현 | ours **8** ↔ upstream **31** · ★**ours 에만 있는 것 0** |
| `Jun025/RustJava` `[patch]` fork | ★**upstream RustJava 의 진부분집합**(우리에만 있는 `.rs` **0**) |

★**핵심**: fork 가 private 이라서가 아니라 **전진하지 않아서** 문제다(`Jun025/wie` 는 이미 public).
★**그리고 진짜 사슬은 `Jun025/RustJava` `[patch]` 표다** — 재정렬과 **독립적으로 지금 끊을 수 있다**(P1).

## 진행중
- **형제 `-18` 두 자리에 시험** (PR 개설 · `wie-sibling-shortbuf-sites-have-no-tests`
  · 채택 제안 `2026-09-07-get-resource-shortbuf-abi#p0`) — `wie_wipi_c/.../kernel.rs` 시험 **2건**.
  ★**제품 코드 무접촉** — diff hunk **3건 전부 `mod test` 안** · 제품 구간 hunk **0** · `return Ok(-18)` 세 줄 그대로.
  ★**시험 0건을 내가 셌다**: `kernel.rs` 시험 5건 중 그 둘을 부르는 것은 성공 경로만 보는 1건뿐이고
  `get_program_name` 은 저장소 전체 시험 참조 **0**이었다.
  ★**개악 대조 — 두 자리가 «각각» red 이고 «독립»**: M1(`-18→-1`)·M2(`-18→-9`) 각각 **1 failed**이고
  ★**형제 시험은 그때마다 green** · 원복 **바이트 동일** · `cargo test --all` **161 → 163**(+2).
  ★기대값은 «수»로 적었다 — `M_E_SHORTBUF` 는 정의가 **0건**(주석뿐)이라 이름을 쓰면 두 번째 진실원이 된다.
- **`get_resource` 의 버퍼-부족 코드가 ABI 어휘 밖이었다** (PR 개설 · `wie-get-resource-shortbuf-code-is-not-in-the-abi`
  · 채택 제안 `2026-09-06-spi-resource-failure-branches#p0`) — ★**⒜(`-18` 통일)** · 비-주석 변경 **2줄**.
  ★**결정 근거는 «일관성»이 아니라 실측이다**: 핀 rev 에서 `WIPICError::from_raw`(=`transmute`)를 지나는 호스트
  함수는 `get_resource` 와 `graphics::create_image` **둘뿐**이고(래퍼 6 = ktf·lgt·simulation ×2) 후자는 `1` 만
  돌려준다 ⇒ ★**이 한 자리가 transmute 경로 위의 유일한 어휘 밖 값 = UB** 였다.
  ★**⒝를 버린 이유**: 실기가 `-1` 을 정의한다는 근거가 **0건**(이 저장소·핀 저장소 양쪽에 규격 없음 · `M_E_*` 정의 0).
  ★★**호환성은 «확인할 수 없다»** — 코퍼스 0(Constraint 9). "안전하다"고 적지 않았다: **측정 불가능한 위험 ↔ 확정된 UB** 의 교환이다.
  ★**개악 대조를 «줄 단위로 한 자리만»**: M1(이 자리) **FAILED** · M2·M3(형제 두 자리) ★**둘 다 통과 = 그쪽은 시험 0건**(제안 등재).
  ★F1⑶ 목록만: 어휘 밖 값이 `-1` 3곳 + `-23` 1곳 더 있다(전부 transmute 경로 «밖» ⇒ 계급이 다르다 · 고치지 않았다).
  ★착지 시 실배포 1건 예상(`web.yml` — 착지 diff 로 다시 셀 것).
- **`--expect-last-frame` 를 «켰다»** (PR 개설 · `wie-expect-last-frame-gate-has-zero-callers`
  · 채택 제안 `2026-09-06-validate-last-frame-gate#p0`) — `AGENTS.md` 러너 블록의 `keydraw_*` 줄에 플래그 한 토큰.
  ★**F2 ⒜ · 실행 줄 0 증가** — 브리프가 ⒜의 대가로 든 「블록이 길어진다」는 ★**이미 사라진 전제**다(#104 가 그 두 줄을 넣었다).
  ★**⒝ 는 «없었다»**: CI 가 부르는 스크립트 11개 중 `wie_validate` 를 «실행»하는 것 **0**(`smoke_gate.sh` 는 로컬 전용).
  ★**⒞ 는 버렸다**: 그 클래스를 매 PR 에 막는 것은 **Scenario F** 이고 CI 에 또 걸면 20초 로컬 되먹임을 잃는다.
  ★**양방향**: 2026-09-05 형상 개악 → LGT **rc=1**(★`result` 는 PASS 인 채) · 위양성 **0** ·
  ★`helloworld_*` 에 걸면 **rc=1** 임을 보여 «걸지 않은» 근거로 삼았다.
  ★**F3 예산**: `--timeout` 은 `--inject` 경로에서 **무력**(deadline 재계산 `2.5+0.3+27×0.6+1.0 = 20.0s`) ⇒ **그대로 둔다**.
  6회 실측 KTF **20.1~26.1s** · LGT **20.2~21.4s**(초과분은 틱 오버런).
  ★착지 시 배포 없음 예상(`.rs`·`Cargo.*` 무접촉 — 착지 diff 로 다시 셀 것).
- (그 밖: ★열린 형제 PR: **#98 · #99 · #100** — 각자 자기
  브랜치에서 진행 중이고, 전부 `REPORT.md`·`STATE.md` 를 만지므로 착지할 때마다 뒤엣것이 원장 2파일에서 충돌한다(정상))

## 완료 (최근)
- 2026-09-07: **파리티 락 축⑷ 별칭 판정** (PR **#118** 착지 · `wie-parity-lock-axis4-module-alias-decision`
  · 채택 제안 `2026-09-06-parity-lock-guard-axes#p0`) — ★**⒝(술어 조이기)를 골랐다**: 별칭을 하드코딩에서
  ★**시험의 `mod <이름>;` 에서 읽기**로 바꿨다. ★의존성 0 · 워크플로 0 · 다른 축 무접촉.
  ★**교환을 수로**: `dod` 개명이 **guard 1/cargo 0 → guard 0/cargo 0**(위양성 해소) · 진짜 깨짐은 여전히 guard 1 ·
  `mod` 선언 제거는 ★**fail-closed(guard 1)**.
  ★**제안의 «위음성» 해석을 정정했다** — 일관 개명은 배선을 안 깨므로 guard 0 이 «옳은 답»이고, 값하는 것은 위양성이었다.
  ★F2: `--list` 는 개명 전/후 출력이 **바이트 동일**이라 이 축을 못 덮는다 · `tree-sitter` 는 상시 구간에 `npm ci` 를 넣어야 해 문서 PR 도 문다(상시 7스텝 전부 Node 내장 · `npm ci` 는 필터 안에만).
- 2026-09-07: **no-leak dist 스캔 처분 — ★«안 한다»로 닫음** (PR **#115** 착지 · `wie-no-leak-dist-scan-never-runs-in-ci`
  · 채택 제안 `2026-09-06-wire-audit-no-leak-in-ci#p0`) — ★**저울을 다시 쟀고 원 판단이 옳았다.**
  F1(★**as of 2026-09-06T22:14Z** · 모집단은 계속 자란다): 그 스텝을 실은 run **19건**(API: **18 success + 1 skipped**) 중
  ★**성공 18건 전건 로그를 받아 18/18 이 「skipping dist scan」** · `contains no game files` **0/18** · ★**미취득 0건**.
  ★함정 둘: `gh run view --log` 가 스텝명을 `UNKNOWN STEP` 으로 접는다(스텝명 말고 **출력 문자열**로 grep) ·
  `gh api …/jobs/<id>/logs` 는 **`--allow-escape-sequences`** 없이는 아무것도 내지 않는다. ★**어느 쪽 0 이든 «부재»가 아니라 측정 artefact 다.**
  ★F2⑶ = **참**: `web/public/` 부재 · 내려받기 0 · 유일한 미추적 입력(`web/src/wasm/`)이 `.js/.wasm/.d.ts` · ★실제 빌드 결과 dist **4파일 · 게임류 0**.
  ⇒ 주석 한 블록으로 근거와 ★**뒤집힐 조건**을 커밋했다. **판정 술어 무접촉**(비-주석 변경 0) · 워크플로 무접촉.
  ★착지 시 배포 없음 예상(`.rs`·`Cargo.*` 무접촉 — 착지 diff 로 다시 셀 것).
- 2026-09-07: **카드 신원을 `class_definition().name()` 으로** (PR **#116** 착지 · `wie-clet-card-identity-use-class-definition-name`
  · 채택 제안 `2026-09-06-clet-card-identity-design#p0`) — `card_canvas.rs` 한 파일(**+44/−24**).
  `getClass()`→`getName()` **invoke_virtual 2회 + to_rust_string** → ★**`class_definition().name()` 한 줄** ·
  `is_clet_card` 의 **`replace('.', "/")` 제거** ⇒ ★**형식 불일치가 «방어 대상»이 아니라 «비존재»** 가 된다.
  ★**F1 양 경로 프로브**: LGT `net/wie/CletWrapperCard` · KTF `CletCard` — **둘 다 내부 형식**.
  ★**개악 2종**: M1(정규화 재도입) → 새 음성 시험 **FAILED** · M2(호출부 되돌림) → LGT
  `last_frame_content=false` · `paints 83`(2026-09-05 서명 재현) · `--expect-last-frame` **rc=1**.
  ★러너 5픽스처 PASS · `paints` **55/55** 기준선 그대로 · ★**하드코딩 두 이름 무접촉**(PR #95 기각 축).
  ★**핀 결합을 적었다**(고치지 않았다): `class_definition()` 은 `RustJava@5b84dd1` API ⇒ 핀 이동 시 재검증.
- 2026-09-07: **`STATE.md` 「완료」 분할 판단** (PR **#114** 착지 · `wie-state-md-completed-section-per-round-split`
  · 채택 제안 `2026-09-07-report-per-round-files#p0`) — ★**하지 않는다 · 구현 0.**
  ★**대조군 실측**: 「완료」 **543줄**을 들어내(681 − 543 = **138** · 자리표시 스텁 4줄을 남겨 파일은 **142줄** · 79% 축소)
  가짜 회차 둘의
  `merge-tree` 가 ★**여전히 rc=1** — 추가 지점이 「완료」가 아니라 ★**「진행중」의 맨 위**다.
  ★열린 PR 5건 중 STATE 접촉 4건이 **전부** 「진행중」을 만지고 ★**「완료」만 만지는 것 0건**.
  ★형제 판단(approve)의 근거 「살아 있는 절 0」이 wie 엔 **성립하지 않는다** — ★자는 **전체줄**이고
  서두 17 + fork 16 + 진행중 8 + 다음 97 = **138줄**(= 681 − 완료 543)이 「지금」을 말한다.
- 2026-09-07: **audit 「허용 경고」 목록 기계 대조** (PR **#119** 착지 · `wie-rust-audit-allowed-warning-list-machine-checked`
  · 채택 제안 `2026-09-06-rust-audit-warning-list-remeasure#p0`) — ⒜ 기대값 파일 + 검사기 + `rust-audit.yaml` **별도 스텝**.
  ★**`cargo audit` 스텝 무접촉**(`--ignore` 0 · rc·트리거 불변) ⇒ **Constraint 5** 와 충돌 없음.
  ★**기대값 ≠ suppression 을 기계로**: 사라진 경고 → **rc=1** · 새 경고 → 인쇄하되 **rc=0**.
  ★**M3(개수는 2로 같고 구성만 스왑) → rc=1** — 실제로 났던 결함 형태이고 개수로는 못 잡는다. 현 상태 rc=0(오늘 red 아님).
  ★대가: 일 1회 스케줄 잡에 `cargo audit --json` 1회(웜 0.89~1.09s) · PR 상시 구간 무접촉 · npm 의존성 0.
  ★착지 시 배포 없음 예상(`.rs`·`Cargo.*` 무접촉 — 착지 diff 로 다시 셀 것).
- 2026-09-07: **key-reach 시험에 `paints > 0` 은 «얹지 않는다»** (PR **#117** 착지 · `wie-key-reach-tests-assert-frame-composited`
  · 채택 제안 `2026-09-06-j2me-guest-boot-in-cargo-test#p0`) — ★**단언 0 · 시험 수 0 증가 · 실행 코드 0줄**(주석만).
  ★**F4 를 «수»로 먼저 답했다**: Scenario **E**(KTF)·**F**(LGT)가 같은 3키(`HASH·STAR·NUM5`)를 돌아
  ★**픽셀 단언 6건**으로 두 캐리어를 덮는다 ⇒ 브리프의 ⒞ 조건 충족.
  ★**더 강한 이유**: 정상 `paints=1` ↔ 2026-09-05 검은 화면 `paints=83` ⇒ ★**`paints > 0` 이 두 상태 다 참**이라
  인용된 사고를 **가르지 못한다**. ★내 「영영 지는 시험이 된다」 가설은 프로브 실측(`paints=1`)로 **반증**됐다.
- 2026-09-06: **createImage 실패 갈래 픽스처 잠금** (PR **#110** 착지 · `wie-createimage-failure-branches-fixture-lock`
  · 채택 제안 `2026-09-06-createimage-fixture#p0`) — 어셈블러에 ★**예외 테이블** + 갈래 2종
  (없는 이름 `java/io/IOException` · 깨진 이미지 `java/lang/IllegalArgumentException` — ★**호스트 소스에서 읽었다**).
  ★**catch 를 좁게** 걸어 «타입»을 잠갔고, 단언은 네이티브 시험 + 브라우저 **C-err** 2곳이다.
  ★**개악 4종 중 M1 이 «물지 않았고 그것이 옳았다»** — 서브클래스라 잡는 것이 맞다 ⇒ 잠금은 「그 타입 «이거나 서브클래스»」로 **한정해 적었다**.
  M1′·M2(비-서브타입) **rc=101 · 2 failed** · M3(안 던지게) **rc=101 · 1 failed**.
  ★**회귀 0**: 브라우저 기준선 **44/44 → 46/46 rc=0** · `cargo test --all` **159 → 160**. ★zip 재생성 **1회** · 타임스탬프 고정으로 **결정적**.
  ★착지 시 배포 있음(`.rs`·`test_data/**` 접촉).
- 2026-09-07: **원장 파일 목록에 `docs/report/**`** (PR **#112** 착지 · `wie-merge-contract-path-list-stale-after-report-split`) —
  `AGENTS.md` §Landing paperwork 한 블록. ★**`REPORT` 존치**(고정 안내로 실재) · 충돌 해소는 **합집합**.
  ★**관측(고치지 않았다)**: 그 승인 열거의 정본은 `~/orchestrator/templates/merge-ticket.tpl` = **이 저장소 밖**이고
  그 파일의 `docs/report` 히트는 **0** ⇒ 형제 `otterpebble`(2026-08-23 이관)도 같은 잠복이다. **총괄 소관.**
- 2026-09-06: **audit 「허용 경고」 목록 재측** (PR **#111** 착지 · `wie-rust-audit-header-comment-says-spin-but-actual-is-chacha20`
  · 채택 제안 `2026-09-06-rtrb-rustsec-2026-0274#p0`) — `rust-audit.yaml` 헤더 주석의 낡은
  「spin 0.12.0 yanked」를 ★**실측값**으로 교체(+측정 날짜): `ttf-parser 0.25.1` + ★**`chacha20 0.10.0 yanked`**.
  ★**낡음이 두 겹**이었다 — spin 은 0.12.2 로 올라갔고 chacha20 은 새로 들어왔는데, ★**총 2건이라 수만 보면 멀쩡했다.**
  ★**주석만 · 동작 diff 0**(비-주석 추가·삭제 **0**, 계수 증명) · **suppression 0**(Constraint 5).
  ★착지 시 배포 있음(`web.yml` 은 필터 없음 · `publish-artifact` 는 `.rs`·`Cargo.*` 없어 **미발화**).
- 2026-09-06: **CI 에서 «안 돌던» 검사 3건 처분** (PR **#109** 착지 · `wie-ci-wire-or-document-three-never-run-checks`
  · 채택 제안 `2026-09-06-deletable-checks-census#p0`) — ★**셋을 한 덩이로 묶지 않았다**:
  ⒜`scripts/audit-no-leak.sh` **배선**(`engine-contract.yml` 네 번째 상시 스텝 · 필터 «밖») ·
  ⒝`verify-browser.mjs`·⒞`smoke_gate.sh` **문서화**(`AGENTS.md` 「Which of these CI actually runs」 절 신설).
  ★**대전제 ⓐ 실측**: 워크플로 8파일 전수에서 세 스크립트 경로 히트 **전건 0**.
  ★**계약 2 이행**: ⒞의 「구조적 불가」를 반증하려다 실패 — `game_lab/` 은 `.gitignore:23` · 추적 0건이고 내용물이 Constraint 9 금지 대상이다.
  ★**양방향**: 기준선 rc=0 ↔ M1(raw `fetch(`) rc=1 ↔ M2'(`git add -f decoy.jar`) rc=1 · 원복 후 rc=0.
  ★**부수 관측**: M2 첫 시도를 `.gitignore` 가 먼저 막았다 ⇒ Constraint 9 의 두 층은 «순서»가 있다.
  ★대가 CI **+0.26~0.34s** · `.rs` 0 · 새 의존성 0. ★착지 시 배포 없음(`.rs`·`Cargo.*` 무접촉).
- 2026-09-07: **`REPORT.md` 회차 파일 이관** (PR **#108** 착지 · `wie-report-md-per-round-files-port-from-otterpebble`) —
  형제 저장소 착지분(`otterpebble` `636bb8e7`)의 **포팅**이다. 643줄/51회차 → `docs/report/` **51파일** ·
  `REPORT.md` 는 자라지 않는 **고정 안내** · `docs/report-migration-revert.md`(되돌림 + 열린 PR 해소 레시피).
  ★**이력 소실 0** — 연번 내림차순 연결이 원문과 **바이트 동일**(sha256 `4f85ec30…` · 213,785B · 절 51 → 51).
  ★**충돌 0 을 대조군으로 보였다** — 두 가짜 회차가 «동시에» 자기 파일을 추가: 이관 후 `merge-tree` **rc=0** ↔
  같은 두 회차를 이관 «전» 부모 커밋에 얹으면 **rc=1 `CONFLICT … REPORT.md`**.
  ★**소비자 전수**(모집단 추적 파일 485): 프로그램 소비자 **0**(`.github` 0 · `scripts` 0) · 쓰는 쪽 **1**(`AGENTS.md`) 이관.
  ★**`.gitattributes` 신설 0** · ★**`STATE.md` 는 가르지 않았다**(현재 상태 파일) ⇒ 원장 충돌은 **2파일 → 1파일**로 준다.
  ★★**게이트② 반려 승계(`-fix`) — 이관은 통과됐고 «문서가 정본이라 부른 명령 2개»만 고쳤다**:
  ①검산 sha 명령이 **이관 이후 회차까지 먹어** 틀린 답을 냈다 ⇒ `sed -n '/\/0051--/,$p'` 로 범위를 자르고
  **유효 범위 `0001~0051`** 을 명시(+ 이관 부모 커밋 독립 축). ②읽기 명령이 `-h` 라 **날짜로 정렬**됐다 ⇒ **`-H` 4곳 전건**
  (실측 54파일: `-h` **52줄 어긋남** ↔ `-H` ★**0줄**). ★이관 산출물 `0001~0051` **무접촉** · base 당김 1회
  (해소 레시피를 이 PR 방향 `mine=:3`·`--ours` 로 실행 — **추가 절 2건 전건 이동** · 검산 `+2 = 53−51` 일치).
  ★★**반려 승계 2회차(`-fix2`) — 「독립 축」이 «착지 뒤» 틀린 sha 를 냈다. 원인은 `--first-parent` 한 토큰.**
  이 저장소는 upstream-sync 라 **머지 커밋으로 착지**하므로 `main` 의 first-parent 경로에 **이관 커밋이 없고**,
  남는 매치인 **머지 커밋**을 `tail -1` 이 골라 `$MIG^` = **착지 직전 `main`** 이 된다. ★**브랜치 위에서는 돌아서
  직전 회차 실측이 통과했다** — 갈리는 자리는 착지 전/후다. ⇒ 토큰 제거(2곳) + **절수 51 확인 줄** + §1 minor 1줄.
  ★**착지 «시뮬레이션»으로 검증**(`git merge --no-ff` 후 실행): 옛 명령 → 머지 커밋 `56b4f8dd` · sha `06b2fc19…` ·
  절수 **54**(틀림) ↔ 고친 명령 → 이관 커밋 `01ef775b` · sha **`4f85ec30…`** · 절수 **51**(★일치).
- 2026-09-06: **리소스 실패 갈래 커버** (PR **#105** 착지 · `wie-spi-resource-failure-branches-uncovered`
  · 채택 제안 `2026-09-06-spi-resource-fixture#p0`) — ★**전제가 절반 거짓이었다.**
  드릴(분기마다 `panic!()`): `-12`(M_E_NOENT `kernel.rs:203`) → `test_missing_resource_clears_size` **FAILED**
  ⇒ ★**이미 커버** / `-1`(버퍼 초과 `:232-233`) → **rc=0 · 아무것도 안 죽는다** ⇒ **미커버(참)**.
  ★★**그리고 티켓의 처방(게스트가 코드를 stdout 에 찍는다)은 `-1` 에 «UB 없이는 불가능»하다** —
  핀의 `WIPICError` 변종이 {1,0,-9,-12,-18,-22,-25} 로 ★**`-1` 이 없고** `from_raw` 가 `transmute` 다.
  ⇒ ★**호스트 단위시험 1건**으로 덮었다(그 쌍둥이 `-12` 가 이미 쓰는 형태) · **zip 재생성 0** · 픽스처 무접촉.
  ★개악 3/3 물었다(반환값 · 분기 제거 · 경계 `>`→`>=`) — 경계 개악이 물린 것은 **성공 방향도 단언**했기 때문이다.
  ★회귀 0(158 passed · `res:9:602` 단언 유지 · 픽스처 PASS). ★**계약 4 범위를 벗어났고 그 이유를 회신에 적었다.**
- 2026-09-06: **노출 판정을 남길 «자리»** (PR **#106** 착지 · `wie-record-dependency-exposure-verdict-for-next-advisory`
  · 채택 제안 `2026-09-06-rtrb-rustsec-2026-0274#p2`) — `docs/project-kb/02_status.md` 공급망 대장에
  **`### C. 해소분 — 노출 판정 보존`** 신설(C-1 = RUSTSEC-2026-0274/`rtrb`) + `rust-audit.yaml` 주석 포인터 1곳.
  ★**2파일 · +46/−0**(순수 추가). ★**새 파일을 만들지 않았다** — 대전제 ⓐ 실측에서 그 대장이 스스로
  「이 절이 정본이다」라고 선언하고 도달성 판정·버전 축을 이미 갖고 있었다(새 파일 = 두 번째 진실원).
  ★**A 에 안 넣은 이유** = A 의 자기 불변식(「audit 경고 수 == 행 수」)을 설계상 깨기 때문 — C 는 그 밖이다.
  ★**유효기간 축을 칸으로**: C-1 은 `rodio@0.22.2` 기준이고 **⑵만 판본 의존**(⑴우리 코드·⑶타입은 무관).
  ★★**관측(미수정)**: 대장 A 가 이미 낡았다 — 경고 **2건**(`ttf-parser`·★`chacha20` yanked) ↔ 표 **3행** ·
  A-2·A-3 은 이미 상향돼 사라졌고 `chacha20` 은 표에 없다. 계약 3(소급 금지)으로 손대지 않았다.
- 2026-09-06: **J2ME 게스트 부팅을 `cargo test --all` 안으로** (PR **#102** 착지 · `wie-j2me-guest-boot-in-cargo-test-all`
  · 채택 제안 `2026-09-06-createimage-fixture#p1`) — `wie_j2me/tests/test_boot.rs` 1건 + `test_utils` 하니스 확장
  + 커밋 픽스처 `test_data/draw_j2me.zip`(1,020B). ★**브라우저 잡 무접촉**(`.github/`·`scripts/`·`web/` diff **0파일**).
  ★**막힌 것은 하니스가 아니라 픽스처였다** — `*.jar` 는 Constraint 9 로 추적 금지라, ★**zip 이 jar 를 담는 기존 관례**
  (`keydraw_ktf.zip` 안의 `00000000.jar`)를 그대로 썼다(`npm run audit` PASSED).
  ★**하니스 핵심 = redraw 응답 루프** — 종전 `TestScreen` 이 요청을 기록 안 해 **첫 판이 10,000틱 paints 0** 이었다.
  ★단언은 «프레임이 합성됐다»(「안 던졌다」는 페인트 전에 죽는 게스트도 통과시킨다 = 2026-09-04 형상).
  ★양방향 2종 red(메인 클래스 미존재 · `image.rs` 마이그레이션 되돌림) · `cargo test --all` **40/156 → 41/157**(회귀 0).
  ★**착지 시 배포 있음** — `Cargo.lock`·`Cargo.toml` 이 `publish-artifact.yml` paths 에 매치(Release + otterpebble dispatch).
- 2026-09-07: **재측 의무 멱등화** (PR **#107** 착지 · `wie-worklog-remeasure-obligation-duplicates-per-round`) —
  `check-worklog-coverage.mjs` 에 **`--record`**(가드 2: 기한 미도래 · 배열 전체 중복) + OVERDUE 문면 교체
  + `AGENTS.md` 정합화. ★**임계·비율 판정 무접촉** · ★**기존 `measurements` 3 → 3 불변(바이트 동일)** ·
  ★**CI 가 도는 «인자 없는» 경로 동작 무변**.
  ★★**판정식이 «max» 가 아니라 «`at(-1)`» 임을 코드에서 확인**했고, 합성으로 흔들림을 재현했다 —
  `[...,40,25]` **rc=1** ↔ `[...,25,40]` **rc=0**(같은 집합·순서만 반대) · 중복에 `pct` 가 다르면 `BELOW-UNANSWERED`.
  ★양방향: 가드 있음 → no-op(항목 3) ↔ 가드 제거 → 항목 **4** · 중복 **{35:2}**.
  ★**남는 구멍**: 두 브랜치가 둘 다 착지 전에 기록하면 여전히 중복(판정식 축은 F3 이 금지 ⇒ 제안).
- 2026-09-06: **러너 목록에 `keydraw_*`·`--inject`** (PR **#104** 착지 · `wie-agents-md-runner-list-missing-keydraw-and-inject`
  · 채택 제안 `2026-09-06-rtrb-rustsec-2026-0274#p1`) — `AGENTS.md` 한 곳(루프 2줄 + 산문 6줄) ·
  ★**삭제행 0**(순수 추가) · 픽스처·러너 코드 무접촉.
  ★**전제 실측**(모집단 332줄): `keydraw` **0건** · `--inject` **0건** — 그런데 그 사실은 `wie_validate.rs:34`
  **코드에는 이미 있었다** ⇒ «읽히는 자리»에만 없었다.
  ★**갈림을 실행으로**: `keydraw_{ktf,lgt}` 플래그 없음 **FAIL · content false · paints 1** ↔
  `--inject` **PASS · content true · paints 55**(두 캐리어 동일).
  ★파리티 락 무영향(마커 구간 «밖» · `dod_ci_parity` 11 passed).
- 2026-09-06: **WIPI 리소스 브라우저 단언** (PR **#101** 착지 · `wie-resource-axis-has-no-browser-scenario-decision`
  · 채택 제안 `2026-09-06-spi-resource-fixture#p1`) — `contract-roundtrip.mjs` 에 **Scenario E-res·F-res 2건**.
  ★**새 zip 0 · 계약 재핀 0 · 글루 변경 0 · 추가 부팅 0 · 키 픽셀 단언 무접촉.**
  ★**제안의 대가 산정이 거짓이었다** — stdout 훅은 `Platform::write_stdout` → `console.log_1` 로 **이미 있었고**
  이 파일이 **이미 수집**하고 있었다(실패 때만 버렸다). ★**wasm 특이성도 실재했다**: 한 줄이 콘솔에
  **다섯 메시지로 쪼개져** 온다(네이티브는 바이트 스트림이라 안 쪼개진다) — 초판 단언이 그 때문에 red 였다.
  ★**개악이 내 이해를 반증**했다: LGT 폴백만 죽여도 green ⇒ 두 캐리어가 **같은 해결자**(클래스로더)를 쓴다.
  양방향: 기준선 **42/42** → 변경 후 **44/44** → 클래스로더 개악 시 **E-res·F-res 둘 다 red · 키 단언 green**.
  ★남는 구멍 = `System::filesystem()` 폴백(호스트별 구현이 갈리는 유일한 자리) — 제안 등재.
- 2026-09-06: **파리티 가드 축⑶ «철자 → 경로»** (PR **#97** 착지 · 머지커밋 `90261612` · `wie-parity-lock-guard-string-axes-to-structure-decision`
  · 채택 제안 `2026-09-06-parity-lock-self-deletion-guard#p1`) — ★**워크플로 무접촉 · 새 의존성 0 · CI 시간 +0s.**
  ★**오탐이 실재했다**: 완전한 `include!` 리팩터에서 `cargo test` **rc=0 · 11 passed** 인데 가드 **rc=1**.
  ★그 축은 «성질»이 아니라 «철자»를 잡고 있었다 — `include!` 형태에서도 검사기만 지우면 cargo **rc=101** 이고,
  `#[path]` 줄만 지우면 ★**`cargo test --all` 이 rc=101**(6다리가 이미 문다).
  ★**축⑷ 는 반대라 무접촉** — 공허한 락은 cargo **rc=0 · 1 passed** 로 지나가고 **가드만** 잡는다.
  ★**세 선택지 비용 실측**: 파서 **+4패키지 + `npm ci` 41.5s/PR** · `--list` **콜드 672.7s / 웜 5.69s**(게다가 축⑷ 미커버) · 무조치 0원(오탐 잔존)
  ⇒ **넷째(경로 참조)** 를 골랐다. 양방향 8종 대조 통과(★P1 오탐 소멸 · ★P5 원 구멍은 `cargo test --all` rc=0 인데 가드 rc=1).
- 2026-09-06: **클렛 카드 식별 설계 결정 — ★기각** (PR **#95** 착지 · 머지커밋 `34bca716` · `wie-clet-card-identity-by-class-name-design-decision`
  · 채택 제안 `2026-09-06-lgt-black-screen-name-compare#p1`) — ★**코드 변경 0** · `card_canvas.rs` 무접촉.
  ★**실측**(임시 프로브 · 되돌림): LGT 카드 = `net/wie/CletWrapperCard`(★**우리 Rust 프로토** · `isInstance` **true**) ↔
  KTF 카드 = `CletCard`(★**게스트 ARM 메모리의 클래스 이름** · `isInstance` **false**). `isInstance(Card)` 는 둘 다 true 지만
  ★**모든 카드가 true** 라 쓰면 검은 화면을 전 경로에 재현한다. 부팅 플래그는 LGT 만 가능(KTF 부팅은 ADF `MClass` → `Main.main` 범용 경로).
  ⇒ ★**두 경로를 한 벌로 덮는 대체 술어가 없다.** 미지(실게임의 KTF 카드 이름)는 **양쪽 갈래가 같은 결론**이라 판정을 흔들지 않는다.
  ★남는 제안 1건 = `getClass().getName()` → `class_definition().name()`(구현 안 함 · #p1 이 원한 것을 주지 않는다).
- 2026-09-06: **`last_frame_content` 게이트화** (PR **#96** 착지 · `wie-validate-last-frame-gate-with-per-fixture-expectation`
  · 채택 제안 `2026-09-06-validate-last-frame-axis#p0` + 흡수 `2026-09-06-lgt-black-screen-name-compare#p0`) —
  축은 이미 있었고 **REPORT-ONLY** 였다. 게이트의 전제는 「빈 마지막 프레임이 정상인 픽스처」를 선언할 자리다.
  ⇒ ★**기대값 선언 자리를 «명령줄»로 골랐다**(`--expect-last-frame` · 기본 off). 이유는 실측이다:
  기대값의 키가 `픽스처`가 아니라 ★**`픽스처 × 모드`** 이고(`keydraw_lgt` 는 `--inject` 여부로 기대값이 뒤집힌다)
  그 모드 절반이 **이미 명령줄에만** 있다 — 사이드카·이름 표는 키의 나머지 절반을 다른 곳에 둬 «두 번째 진실원»이 된다.
  ★**계약 4 충족**: 플래그 없이 6행 전건 현행 판정 불변.
  ★★**개악 M1 = «실제로 일어났던» LGT 검은 화면 회귀**(`is_clet_card` 정규화를 PR #88 이전으로 되돌림) →
  플래그 없음 **PASS·content true·last false**(★그때 새어 나간 형상 그대로) ↔ 플래그 **FAIL** · KTF 대조군 PASS.
  ⇒ ★**헤드리스가 20초에 그것을 문다**(브라우저 축은 wasm 빌드 + `contract` 잡 3~4분).
  ★**남는 것**: 호출자 **0** — 선언 자리를 만들었을 뿐 아직 아무도 켜지 않았다(제안 등재).
  ★★**게이트② 발견 ①(major)를 함께 남긴다 — 이 항목의 「부하 시」 서술은 «반증됐다»**: 검수자가 load **102.77/10CPU**
  구간에서 같은 개악을 11회 돌려 **전건 `PASS · last=true`**(게이트가 회귀를 **놓친다**)를 관측했고 `content` FAIL 은 **0회**였다.
  ⇒ REPORT.md 의 「기존 `content` 축이 먼저 FAIL 하므로 새 불안정을 더하지 않는다」는 **사유가 반대로 적혀 있다**.
  ★**판별자는 `ms > --timeout` 과 `paints` 붕괴**(굶은 런 21.2~29.2s·paints 26~53 ↔ 정상 20.1s·83).
  ★계약 7 대로 이 회차가 **고치지 않았다** — CI 게이트로 올리는 후속이 «반드시 먼저» 처분해야 한다.
- 2026-09-06: **「감시를 지웠는데 green」 전수 계수** (PR **#94** 착지 · 머지커밋 `0f7bb2fc` · `wie-count-deletable-checks-that-stay-green-repo-wide`
  · 채택 제안 `2026-09-06-parity-lock-self-deletion-guard#p0`) — ★**세기만 하는 회차 · 가드 0 · 코드 0 · 워크플로 무접촉.**
  술어 「워크플로가 «경로로» 부르는 검사 파일 A」↔「`scripts/`·`*/tests/` 실재 파일 B」 ⇒ ★**A=9 · B=23 · 차집합 14**.
  ★**두 형태로 갈린다**: ⒜**「CI 에서 돌고 있는데 지워도 green」 8**(전건 rust 통합시험 — `cargo test --all`·
  `tarpaulin --workspace` 가 **glob 으로 줍는다**) ⒝★**「애초에 CI 에서 안 도는 검사」 3**(`audit-no-leak.sh`·
  `verify-browser.mjs`·`smoke_gate.sh` — ★**지울 필요도 없다**) ⒞검사 아님 2 ⒟★**술어 오탐 1**
  (`…/tests/support/dod_ci_parity.rs` — 직전 회차 가드가 물어 red ⇒ **가드가 작동을 증명했다**).
  ★**실측 축**: 격리 워크트리에서 ⒜의 1건을 실제로 지우고 `cargo test --all` → **rc=0 · 156→155 passed ·
  그 이름 출력 «0회»**(커밋 0). ★`codecov.yml` 0바이트라 **커버리지 게이트도 못 잡는다**.
  ★★**회차를 낳은 「이 저장소에서 다섯 번」은 «출처가 없다»** — 실제로 센 five times 는 `AGENTS.md` 의
  **셀프머지**(다른 형태)다. 세 자리에 상호참조 정정(원문 보존). ★**8건에 각각 가드를 다는 것은 권하지 않는다** —
  값하는 자리는 ⒝의 3건이다.
- 2026-09-06: **`Image.createImage(String)` 픽스처** (PR **#93** 착지 · `wie-system-class-loader-createimage-fixture`
  · 채택 제안 `2026-09-05-system-class-loader-preemptive-migration#p1`) — 제안이 미완으로 남긴 문장은
  하나였다: 「갈림 «자체»는 측정됐다 … **미측정인 것은 «넓어진 가시 범위가 실제로 무엇을 찾는가»** 하나다」.
  ⇒ jar 에 `wie-img.png`(16×8 · 74바이트)를 동봉하고 `DrawMIDlet.startApp()` 이 그것을 **이름으로** 열어
  `getWidth()`·`getHeight()` 를 저장하며 `paint()` 가 **그 치수 그대로** 사각을 채운다
  ⇒ ★**칠해진 픽셀 수 = 호스트가 찾아 디코드한 이미지의 픽셀 수**(왕복 Scenario C-img 가 `1024+128=1152` 를 등호로 단언).
  ★**커버 0 → 1**: 같은 `panic!()` 프로브가 **전**에는 `cargo test --all` rc=0·156 passed·5픽스처 전건 PASS 였고,
  **후**에는 `draw_j2me.jar` 를 부팅에서 죽인다.
  ★**개악 E1**(`get_system_class_loader` → `jvm.current_class_loader()`) → `IOException: Resource not found: /wie-img.png`
  ⇒ ★**넓어진 범위가 찾는 것 = 1건 · 종전 경로 = 0건.** 왕복 **42/42**.
  ★**게이트②가 «다른 자리»로 재확인했다** — `wie_backend::decode_image` 를 개악하니(회신이 만진 크레이트 «밖») 이 픽스처만
  FAIL 하고 대조군 3건은 green ⇒ 공허한 통과가 아니다. ★단 `wie_validate` 는 이 축의 오라클이 아니다(폭을 2배로 해도 PASS) —
  이 커버는 사실상 **`contract` 잡 단독 의존**이다.
- 2026-09-06: **`rtrb` 보안 자문 해소** (PR **#92** 착지 · `wie-rustsec-2026-0274-rtrb-double-free-audit-red`) —
  매일 도는 `Security audit`(schedule 전용 · PR 게이트 아님)이 3일 연속 `error: 1 vulnerability found!` 였다.
  `RUSTSEC-2026-0274` = `rtrb` 의 `ReadChunk::commit` 에서 **원소의 `Drop` 이 panic 할 때** double free/UAF.
  ★**전이 의존이다** — `wie_cli → rodio 0.22.2 → rtrb`. rodio 요구가 `^0.3.2` 라 ★**rodio 무접촉**으로
  `cargo update -p rtrb` 만에 패치판 `0.3.5` 로 간다 ⇒ `Cargo.lock` **2줄** · 다른 크레이트 이동 0.
  ★★**노출은 «안 탄다»** — 구조는 ★**「⑴이 전제 · 그 아래 ⑵⑶ 이 독립 2중화」**다(게이트② 정정: ⑵⑶ 은
  rodio 의 단일 인스턴스화만 재므로 ⑴에 기댄다 — 「셋 다 독립」이 아니다. ⑴을 PR head 트리에서 0건으로
  직접 확인해 전제가 참이므로 결론은 선다):
  ⑴우리 `.rs`/`.toml` 의 `rtrb|ReadChunk` **0건**(쓰는 rodio 표면은 재생 전용)
  ⑵rodio 는 `microphone.rs` 에서만 rtrb 를 쓰는데 그 파일의 `read_chunk|ReadChunk|.commit` **0건**(`pop()` 만 쓴다)
  ⑶원소 타입 `rodio::Sample = f32` = **`Drop` 없음** ⇒ 자문의 전제가 구조적으로 성립 불가.
  ⇒ ★**red 를 끄는 것이 아니라 「노출되는가」에 답한 뒤 안고 가지 않기로 한 것**이다(실보안 이득은 0에 가깝다).
  ★검증: `cargo audit` **rc=0** · schedule 전용 검사를 브랜치에서 `workflow_dispatch` 로 돌려 **success**(run 34019000701) ·
  게이트②가 lock 의 rtrb `version` 한 줄만 되돌리는 돌연변이로 **rc=1 red 재현**.
- 2026-09-06: **WIPI 리소스 픽스처** (PR **#91** 착지 · `wie-system-class-loader-spi-resource-fixture`
  · 채택 제안 `2026-09-05-system-class-loader-preemptive-migration#p0`) — 아카이버는 이미 리소스 디렉터리를
  jar 에 넣고 있었는데 ★**그 디렉터리가 «비어 있었다»** ⇒ 9바이트 `res.bin` 을 넣고 게스트가 부팅 때 읽어
  `res:9:602` 를 찍게 했다. ★**한 줄이 «두 홉»을 각각 증명한다**(size=`get_resource_size` · 합=`read_resource`).
  ★**자리별 드릴로 4자리 «각각» FAILED rc=101**(기준선 rc=0) — 종전에는 4자리에 `panic!()` 을 심어도 **150 passed**.
  ★회귀 0: key-reach 2건 통과 · 브라우저 왕복 **35/35** · `verdict` ⒡ 표를 「커버 1곳 → 5곳」으로 정정(남은 미커버 = **6번 하나**).
- 2026-09-06: **파리티 락 «자기 삭제» 가드** (PR **#90** 착지 · `wie-dod-ci-parity-self-deletion-guard`
  · 채택 제안 `2026-09-05-dod-ci-parity-checker#p0`) — 락은 두 파일이고 `#[path]` 결합 덕에 «한쪽만» 지우면
  컴파일 오류지만 ★**둘을 «함께» 지우면 `cargo test --all` 이 rc=0**(실측 · 출력에 `dod_ci_parity` **0회**).
  ⇒ 그 두 경로를 «바깥»에서 부르는 유일한 참조자 `scripts/check-parity-lock-wired.mjs` + `engine-contract.yml`
  **상시 스텝 1개**(필터 밖 · paths 목록 무접촉). ★존재만이 아니라 `#[path]` 결합과 「검사기를 부르는 `#[test]`」까지 단언한다.
  ★**개악 8종 전건 red**(M1 = 두 파일 함께 · M8 = 가드 자신) · 기준선 green · 비용 ~50ms.
- 2026-09-06: **`wie_validate` «마지막 프레임» 축** (PR **#89** 착지 · `wie-lgt-validate-last-frame-axis`
  · 채택 제안 `2026-09-05-lgt-browser-paint-localize#p1`) — `saw_content` 가 프레임 전체에 대한 **OR** 이라
  ★**마지막 프레임을 구조적으로 못 본다**(OR 은 단조 — 나중 프레임이 값을 되돌릴 수 없다) ⇒ 같은 술어를
  마지막 프레임에만 적용하는 **`last_frame_content`** 를 더했다. ★**보고 전용 · `passed` 분기 무접촉.**
  ★**양방향을 «살아 있는 결함»으로** 보였다(#p0 이 미착지라 main 에 검은 화면이 그대로 있다):
  당시 main LGT `content=true · ★last_frame_content=false` ↔ #88 을 얹으면 `true` · KTF 는 둘 다 불변.
  ★★**[2026-09-06 갱신] 그 «당시 main» 행은 이제 «재현되지 않는다»** — **#88 이 착지**해(머지커밋 `3c02ce61`)
  LGT 도 `last_frame_content=true` 다. ★회신이 그 사실을 **미리 적어 뒀다** — 모순이 아니라 «예고된 것»이다.
  ⇒ ★**이 축의 값은 그대로다**: 같은 형태의 «다음» 덮어쓰기는 여전히 이 필드에서만 보인다.
  ★**게이트로 안 올린 이유도 실측이다** — 지금 걸면 `helloworld_*` 2픽스처가 PASS → FAIL 로 뒤집힌다.
- 2026-09-06: **LGT 검은 화면 «근인» 수정** (PR **#88** 착지 · `wie-lgt-browser-paint-black-screen-name-compare`
  · 채택 제안 `2026-09-05-lgt-browser-paint-localize#p0`) — `Class.getName()` 은 **점**(핀이 `replace('/', ".")`)인데
  `card_canvas.rs` 가 «슬래시» 리터럴과 비교해 ★**`disablePaint()` 분기가 한 번도 돈 적이 없었다** ⇒ MIDP 가 빈
  `screenImage` 로 덮어 마지막 프레임이 검정.
  ★**처방은 ⒜(점 형식 추가)가 «아니라» ⒝(정규화)** — ⒜는 지금의 두 이름만 맞추고 형식 취약성을 남긴다
  (KTF 가 무사한 것은 그 게스트 클래스에 **패키지가 없어서**다).
  ★**「분기가 실제로 도는가」를 호출 계수로 보였다**: 수정 후 LGT **1회** ↔ 개악 ★**0회**(KTF 는 둘 다 1회).
  ★**브라우저 양방향**: 무개악 **41/41 rc=0** ↔ 개악 **rc=1 · F 세 키 0 px**. `contract-roundtrip.mjs` **Scenario F 신설**.
  ★그물이 «그것뿐»이다 — `wie_validate` 는 sticky any-frame 이라 검은 화면을 PASS 로 낸다(별건 제안).
- 2026-09-05: **`get_system_class_loader` 6곳 선이행** (PR **#83** 착지 · `wie-system-class-loader-preemptive-migration-six-sites`
  · 채택 제안 `2026-09-05-current-class-loader-replacement-design#p0`) — `+34` 에서 비공개가 되는 통로를
  **bump 없이** 지금 핀 위에서 갈아탔다(6줄 · `use` 변경 0). ★**값은 «6줄»이 아니라 «검증의 분리»다.**
  ★★**자리별 커버리지를 «심어서» 쟀더니 6곳 중 «1곳»만 커버된다** — ★**①** LGT 부팅만 red(123/1 · 픽스처 2건 FAIL) ·
  나머지 5곳(②③ LGT 리소스 · ④⑤ KTF 리소스 · ⑥ MIDP)은 `panic!()` 을 심어도 **139/0 · 5/5 무변화**.
  ⇒ ★설계 문서 §8-4⑶-b⒡ 의 「2~5번을 태운다」는 **거짓**이었고 표로 정정했다.
  미커버 5자리는 **단서 주석 + 제안 2건**으로 닫았다(계약 3⒝).
  ★★**[게이트② 반려 · `-fix`] 「두 API 가 같은 값」이 ⑥에서 «거짓»이었다** — 프로토가 도는 동안 «호출 클래스»는
  게스트가 아니라 **`Image` 자신**이고 그것은 `RustJarClassLoader` 소속이라 `current_class_loader` 가 **그 로더**를 준다.
  ★그 로더는 `findResource` 가 항상 null 이고 parent 가 `None` 이라 리소스를 **영원히 못 찾았다** ⇒
  이 치환은 «무변경»이 아니라 ★**가시 범위를 시스템 로더로 넓히는 «잠재 수정»**이다. **코드는 되돌리지 않았다.**
  ★그리고 ⒡ 표의 자리 번호를 **⒝ 표와 같게** 되돌렸다(같은 절 안에 번호가 둘이면 정정문이 뒤집혀 읽힌다).
  ★★**[2회차 `-fix2`] «측정 ↔ 미측정»을 갈랐다** — 측정된 것 = ⑥의 **갈림 자체**·①의 커버·나머지 미커버 ·
  미측정 = ⑥에서 **«넓어진 가시 범위»가 무엇을 찾는가**(그 자리를 부르는 시험 0건).
  ★그리고 「같은 값」 원문이 남아 있던 **두 곳**(설계 회차 워크로그 `limits[0]` · `REPORT.md` 설계 항목)에
  **인라인 정정 + 상호참조**를 붙였다 — 「남의 착지본」이라는 사유를 쓰지 않았다.
  ★★**[4회차 `-fix4`] «어휘»로 세면 안 잡히는 자리가 있었다** — `verification.public-replacement-exists` 의
  「갈리는 경우는 «호출한 **게스트** 클래스가 자기 로더를 가질 때»뿐」이 정정 없이 남아 있었다(자리=1곳은 참 · **기전 귀속**이 반증된 쪽).
  ★가장 넓은 「같은 값」류 술어로도 그 줄은 **히트 0** — 같은 주장을 «갈림·한정» 어휘로 말하기 때문이다.
  ⇒ ★술어를 «주장»(주체어 ∧ 단정어)으로 다시 세웠다.
  ★★**[정정 · 5회차 `-fix5`] 종전의 「그것만 그 줄을 잡는다」는 «지웠다» — 그 술어는 «대조군»을 놓친다.**
  같은 주장을 하는 형제 문장 `docs/upstream-realign-verdict.md:735`(짧은 산문)은 ★**주체어 히트 0 ⇒ C 히트 0** 이다.
  A2 를 잡은 것은 A2 가 «한 줄짜리 거대 JSON 값»이라 주체어가 우연히 같은 줄에 있었기 때문이고,
  ★**그 줄을 실제로 찾아낸 것은 C 가 아니라 «검수자 술어(`자기 로더를 (가|갖)`) 단독»이었다.**
- **★★[6회차 `-fix6`] 이월이 «여섯 곳» 있었다 — 어휘로 «먼저» 전수 훑고 고쳤다**(★`-fix7` 정정: 초판 「세 곳 더」)
  ★**정본 절 §8-4⑶-b ⒢ 가 같은 절 44줄 위의 ⒞ 와 어긋났다** — ⒢ 가 2·3·4·5 를 「«게스트 프레임이 없다» 갈래」로 되돌리고 있었다.
  ★★**가벼운 자리가 아니다**: 치환된 `.rs` **5곳**의 주석이 `§8-4(3)-b` 를 가리킨다(★**예외는 ①`wie_lgt/src/emulator.rs` — 그 자리는 치환이 «한 줄»이라 주석이 «아예 없다»**) ⇒ 코드에서 온 독자의 «첫 문장»이 반증된 전제였다.
  ★★**[정정 2026-09-05 · 게이트² 7회차 `-fix7`] 직전 회차가 여기에 쓴 「**6곳 전부**」는 «거짓»이었다** — 실측은 **5**다(@`3d69c938` 파일별: `wie_lgt/src/emulator.rs` **0** · `wie_lgt/…/wipi_c/context.rs` 2 · `wie_ktf/…/wipi_c/context.rs` 2 · `wie_midp/…/image.rs` 1). ★**그 수는 검수 회신의 문면을 «옮긴 것»이고 옮기면서 «다시 세지 않았다»** — `git grep -c` 한 번으로 갈렸다. ★**논지는 약해지지 않는다**: 5곳이어도 「코드에서 온 독자의 첫 문장」은 그대로 서고, ★**오히려 ①에는 앵커가 «없다»는 편이 더 정확하다**(그 자리는 문서로 가는 길이 아예 없다).
  함께: `REPORT.md` 의 맨 단정 1곳 · `migration.json` `limits[0]`·`premise-3-same-value` · `verdict` ⒡ 어휘 · 계수의 «측정 rev» 표기.
  ★**원문·기존 각주 전건 보존**(각주 얹기) · **코드 변경 0**.
- **★★[5회차 `-fix5`] 「6곳 중 1곳」을 «수 하나»로 적을 수 없다 — 재도출했다**
  판별식은 「호출 클래스가 «게스트»인가」가 아니라 **「스택에 Java 프레임이 있는가 · 그 클래스의 로더가 `None` 인가」**다.
  ①프레임 없음 / ②③프레임 있음·로더 `None` / ★**④⑤프레임 있음·로더 `net/wie/KtfClassLoader`** / ⑥`RustJarClassLoader`
  ⇒ ★**«다른 객체»면 3곳(④⑤⑥) · «리소스 해결이 갈린다»면 1곳(⑥)**. ★④⑤ 가 안전한 이유는 「프레임이 없다」가 아니라
  **부모(시스템 로더) 위임**이다(`findResource` 재정의 0 + 부모가 시스템 로더 + 핀 `get_resource` 가 parent 를 먼저 물음).
  ★같은 이월이 있던 **세 자리**(verdict ⒝표 1행 · ⒞:735 · 원장 `…six-sites.done.md:44-45`)를 함께 고쳤다. **코드 변경 0.**
  ★★**[3회차 `-fix3`] 그 전수가 «0 이 아니었다» — 근인은 술어다**: `git grep` 기본이 **BRE** 라
  회신에 옮겨 적은 `'…|…'` 이 **0건**을 냈다(실행본은 `\|` 라 맞았고, **옮길 때 백슬래시를 떨어뜨렸다**).
  ⇒ 설계 워크로그 `proposals[0]` 의 **plainSummary·tradeoff** 2건을 마저 정정했다.
  ★처분 후 전수 25건 판정 = 무관 6 · 정정문 8 · 정정 부착 4 · 여전히 참 6 · 이번 처분 1 ⇒ **바레 주장 0**.
- 2026-09-05: **DoD ↔ `rust.yml` 파리티 잠금** (PR **#87** 착지 · `wie-dod-ci-parity-checker-port-from-rustjava`
  · 채택 제안 `2026-09-05-rust-yml-header-beta-lint-line#p1`) — RustJava 의 `check-dod-ci-parity.py` 를 «이식»했다.
  ★대조 대상은 형제 회차 #86 착지 «위에서» 정했다 — 「머리 주석 ↔ steps」가 아니라 ★**「`AGENTS.md` COMMIT-GATES 마커 구간 ↔ steps」**.
  ★**정의**: 축 A = cargo 호출 «집합» · 축 B = toolchain 집합 · **두 축 독립**(교차곱 아님) · **순서는 파리티가 아니다**.
  ★**바꾼 가정 4개**: 정본 파일(`CLAUDE.md`→`AGENTS.md`) · 구간(첫 블록→마커 구간 전부) ·
  게이트 판별자(`if:` 없음→**cargo 를 부르는가**) · 블록 스칼라(접기→**env 평탄화**).
  ★**배선 = `cargo test --all` 이 줍는 `#[test]`** ⇒ ★**워크플로 잡·스텝 변경 0** · rust.yml **6다리 전부**에서 돈다.
  ★**양방향 3종 실측**: 어긋남 **rc=101** · 무개악 **149 passed/0** · ★검사기 삭제 **rc=101**(2파일 구성의 이유).
  ★**현 판정 green**(축 A 대칭차 0 · 축 B 일치) — 단 그것은 #86 이 손으로 맞춘 «뒤»의 값이고, 이 회차는 «유지 장치»다.
  ★★**초판은 windows 두 다리에서 red 였다**(`1902b602`) — 근인은 파서가 아니라 **개악 대조 앵커의 `\r\n`** 이고,
  터진 것은 파리티 판정이 아니라 **「앵커 표류」 가드**였다(조용히 통과하지 않았다). 읽기 지점에서 CRLF 를 접고 회귀 시험을 넣었다.
  ★**천장 9개를 검사기가 매 실행 출력**한다(OS 축 · 교차곱 · 타 워크플로 · 비-cargo 게이트 · 동의어 · 손파서 · ★검사 자신의 삭제 · 순서).
- 2026-09-05: **`rust.yml` 주석 목록 «한 벌»로** (PR **#86** 착지 · `wie-rust-yml-header-single-source-point-to-agents-md`
  · 채택 제안 `2026-09-05-rust-yml-header-beta-lint-line#p0`) — 주석의 명령 **6줄 삭제** →
  `AGENTS.md` 의 `<!-- COMMIT-GATES:BEGIN … -->` ~ `:END` **마커 구간**을 가리킨다.
  ★★**[게이트② 반려 `-fix`] 초판의 「두 `sh` 블록」은 «거짓»이었다** — 그 소절엔 블록이 **4개**다
  (③`wie_validate` 러너 ④`gh pr checks` 도 «커밋 전 명령»이라 오독이 자연스럽다) ⇒ **세는 식별자를 버리고 마커로**.
  ★주석 안 명령 목록 **6 → 0** · ★**CI 동작 변경 0**(비-주석 변경 0행 · 주석 제거 후 바이트 동일).
  ★**Constraint 1 정합 = 충족** — 그 표의 `Locked by` 는 «why 의 거처»를 뜻하고 why 두 블록은 그대로다.
  ★★**과장하지 않는다**: 없앤 것은 «산문 사본 2 → 1» 이고 `steps:` 라는 기계 사본은 남는다 —
  문서↔steps 축은 파리티 검사기(`#p2`) 몫이라고 주석에 적었다. ★잠금은 ⒝(안 한다 — 대상이 사라졌다).
  ★★**[2회차 `-fix2`] 「바레 주장 0」이 «거짓 실측 주장»이었다** — 워크로그 `limits[1]` 이 옛 식별자
  (「경로+절+«두 블록»까지」)를 그대로 주장하고 있었다. 마커 형태로 고치고 선행 `done` 의 「0」 선언도 맞췄다.
  ★근인: 술어를 좁게 잡고 **원장(`git grep` 대상 밖)을 따로 세지 않았다** ⇒ 재계수 repo 12 · 원장 13 = **25건**,
  바레 주장 **0**. ★그리고 **END 마커가 「the two `sh` blocks above」로 다시 세고 있어** 「enclosed region · do not
  count blocks」로 고쳤다(목적이 «세는 것을 없애기»인데 마커 자신이 셌다).
  ★★**[`-fix`] 잠금 재판단 = ⒝ 유지 · 사유 교체** — 「Both(=2)」는 대상이 4로 늘어도 **조용히** 틀렸고
  이름 붙은 마커는 없어지면 `grep` 이 0건으로 **즉시** 답한다(**silent → loud**). 그래도 기계는 없다 —
  `engine-contract.yml` 필터에 `AGENTS.md`·`rust.yml` 이 **없어** 이 드리프트를 내는 PR 에선 어떤 in-filter 검사도
  돌지 않고, 상시 스텝 추가는 `steps:` 변경이라 이 티켓이 금지한다. ★**«알고» 남긴다.**
- 2026-09-05: **LGT 브라우저 검은 화면 «규명»** (PR **#85** 착지 · `wie-lgt-browser-canvas-paint-not-reaching-localize`
  · 채택 제안 `2026-09-05-roundtrip-ktf-key-reach-scenario#p0`) — ★**선행 서술이 반증됐다**:
  프레임은 캔버스에 **닿는다**(`incoming_nonblack=424` · `draw_image ok`). ★**그 «직후» MIDP 가**
  **비어 있는 screenImage 로 덮는다**(`disable_paint=false` → `incoming_nonblack=0`) ⇒ 마지막 프레임이 검정.
  ★근인 = `card_canvas.rs` 가 `Class.getName()`(**점** `net.wie.CletWrapperCard`)을 **슬래시** 리터럴과 비교 ⇒
  `disablePaint()` 영원히 미호출. KTF 카드는 `CletCard`(패키지 없음)라 우연히 매치된다.
  ★반증 실험(원복함): 점 형식 1개 추가 → 브라우저 LGT **0 px → 424 px** · 네이티브 paints **83 → 55**(KTF 동일).
  ★**고치지 않았다** — 수정은 다음 회차(제안 2건). 제품 코드 변경 **0**.
  ★★**[게이트② 반려 `-fix`] KTF 의 «앞으로»를 적었다**: `CletCard` 는 **게스트에서 오는 이름**이라
  카드 클래스에 **패키지가 붙는 순간 KTF 도 같은 형태로 깨진다** ⇒ ★처방 ⒜(점 형식 «추가»)는
  그 **형식 취약성을 남긴다**. 제안 1 `tradeoff` + 왕복 검사 헤더 두 곳에 넣었다.
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재라 스쿼시 금지.
  ★형제 #84 착지로 `REPORT.md` 가 충돌했고 `-fix` 회차가 **머지로** 해소했다(코드 충돌 0) ·
  착지 시점에 형제 **#83·#86** 이 열려 있다.
- 2026-09-05: **`rust.yml` 머리 주석에 beta 린트 줄** (PR **#84** 착지 · `wie-rust-yml-header-comment-beta-lint-line`
  · 채택 제안 `2026-09-05-dod-four-gates-beta-axis#p0`) — 「로컬에서 돌려라」 목록이
  `rust.yml` **4** ↔ `AGENTS.md` **6** 으로 갈려 있었다(PR #81 이 문서 쪽에만 더했다).
  ⇒ 같은 2줄을 더해 ★**명령 6개·순서까지 기계 대조로 일치**. ★**CI 동작 변경 0**(주석 제거 후 바이트 동일).
  ★★**잠금은 «하지 않기로» 정했다(⒝)** — 배선 없는 검사기는 「잠긴 것처럼 보이는데 안 도는」 것이고,
  배선 없이 되는 길은 **파리티 검사기 이식(`#p2`)과 충돌**하며, 더 나은 처방은 **중복 제거**다(제안 2건).
  ★대신 «잠금이 없다»를 주석에 **리터럴로** 적었다.
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재라 스쿼시 금지.
  ★충돌 **0**(형제 착지가 그 사이 없었다) · 착지 시점에 형제 **#83·#85** 가 열려 있다(같은 원장 2파일).
- 2026-09-05: **브라우저 왕복 KTF 키 «도달» Scenario E** (PR **#82** 착지 · `wie-browser-roundtrip-ktf-key-reach-scenario`
  · 채택 제안 `2026-09-05-ktf-lgt-key-reach-fixtures#p0`) — 왕복 **29 → 35 pass** · 제품 코드 **0줄**.
  ★**D 가 못 보는 홉을 잰다**(KTF 는 `CardCanvas` 가 MIDP 코드를 WIPI 코드로 한 번 더 바꾼다) —
  개악 ⒝(`KEY_NUM5 => Self::NUM1`)에서 ★**E 만 red · D 전건 green** 으로 독립을 실행으로 보였다.
  ★상수는 픽스처 소스(`make-wipi-keydraw-fixture.sh`)에서 **파싱해 파생**하고 표류 시 fail-closed throw.
  ★★**LGT 는 부분 완료 — 막은 것이 «키»가 아니다**: 키는 도달하는데(콘솔 `key:42`·`key:53`) 캔버스가 **0 px**.
  순서·픽스처 배제(네이티브 `wie_validate --inject` 는 같은 zip 을 **PASS · paints 83**) ⇒ 축은
  **LGT paint → WebScreen → canvas**. red 를 착지시키지 않고 «왜 없는가»를 헤더에 남기고 제안 1건.
  ★**형제 #81 착지로 원장 2파일이 충돌했고** 이 게이트③이 **2-c⒜**(원장 한정 승인)로 합집합 해소했다(코드 충돌 0).
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재라 스쿼시 금지.
  ★착지 시점에 형제 **#83** 이 열려 있다(같은 원장 2파일 · 그쪽 게이트③이 해소한다).
- 2026-09-05: **DoD 에 beta 축 한 줄** (PR **#81** 착지 · `wie-dod-four-gates-add-beta-axis` · 채택 제안 `2026-09-04-parity-sibling-repo-survey#p0`)
  — `AGENTS.md` §Definition of Done 에 `cargo +beta clippy --all -- -D warnings` 블록 1개. ★**코드 0 · CI 워크플로 0.**
  ★근거는 내가 다시 센 수다: `rust.yml` run **전량 246건**(`ac4ce1aa` 2026-06-24 → `11a35252` 2026-09-04) 중 실패 **34**,
  그중 **9건이 beta 다리 단독**이고 ★**9/9 가 린트 게이트**(8 × `chunks_exact_to_as_chunks` · 1 × `double_must_use`) ⇒ 한 줄.
  ★비용 실측 = 회차당 **+7.6~7.7s**(최초 1회 36.3s) · **스래싱 없음**(stable 복귀 0.50s).
  ★한계: 그 9건의 stable 다리는 전부 `cancelled` 이었다(`fail-fast: false` 는 2026-08-27 **PR #65**(`250d7e4c`)) — 증명은 린트 정체가 진다.
  ★★**[게이트② 반려 · `-fix`] «배경 서술»의 수를 정정했다**(헤드라인 34/9/9 는 재현돼 무접촉):
  2026-07 클러스터는 `main` 을 **6 push · 약 36시간**(07-06T18:24 → 07-08T05:51 · 전건 `image.rs:310`) red 로 뒀고
  해소는 ★**`37e3e4f6`(PR #21)** 이다. ★**`e3cbaa08`(PR #33)은 «07-13 `wie_lgt` 의 다른 자리»**(피처 브랜치)라
  그 클러스터의 해소가 **아니다** — 초판이 두 사건을 한 커밋에 귀속시키고 기간을 «7일」로 부풀렸다.
  ★권장 ③④도 넣었다: 설치 줄 주석을 「beta 가 구를 때마다 다시 쳐라」로 · 「내 변경 탓이 아닌 beta red 는 **별 회차로 분리**하라」 한 문단.
  ★**형제 #80 착지로 실제로 `REPORT.md`·`STATE.md` 가 충돌했고**(예고가 정확했다) 이 `-fix` 회차가 **머지로** 해소했다(리베이스 0).
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재라 스쿼시 금지.
  ★착지 시점에 형제 **#82·#83** 이 열려 있다(같은 원장 2파일 · 각자 해소).
- 2026-09-05: **`current_class_loader` 대체 API 설계(`+34` 벽)** (PR **#80** 착지 · `wie-current-class-loader-replacement-api-design-for-plus34`
  · 채택 제안 `2026-09-04-upstream-realign-p1-pin-plus33#p1`) — ★**결론: 벽이 아니었다.**
  ⒜파열은 계단표의 `+46` 이 아니라 ★**`+34`(`7dc1b90`)에서 시작**한다(그 커밋은 `pub` 한 줄만 내렸다)
  ⒝호출부 **6곳이 전부 같은 두 줄** — 클래스 «로딩»에 쓰는 자리 **0곳**
  ⒞★**공개 대체가 있다** — `get_system_class_loader` 가 **핀·HEAD 둘 다 `pub`** 이고 ★`current_class_loader`
    자신이 떨어지는 **폴백**이다(우회가 아니다) ⒟★**핀에서도 공개라 «bump 전»에 이행 가능** ⇒ 그 칸 **6 → 0**.
  ★프로브로 확인(6곳 스왑 → `cargo test --all` **139/0** · `wie_validate` 5픽스처 PASS → **원복** · 코드 델타 0).
  ★남는 구멍: `Image.createImage(String)` 자리는 스위트가 «부르지 않는다»(논증으로만 선다). 정본 **§8-4⑶-b**.
  ★**형제 #79 착지로 원장 2파일이 충돌했고** 이 게이트③이 2-c⒜(원장 한정 승인)로 합집합 해소했다
  (코드 충돌 0 · `docs/upstream-realign-verdict.md` 는 **auto-merge** 되어 hunk 가 바이트 동일).
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재라 스쿼시 금지.
- 2026-09-05: **«버린 fork» 커밋 로그 통독** (PR #79 착지 · `wie-abandoned-fork-commit-log-harvest`
  · 채택 제안 `2026-09-04-unported-hardening-two-axes#p0`) — ★**가설은 맞았다**: §10-2 가 인용한 «두 줄»은
  전부가 아니었고 실제로는 **9커밋 · 타이틀 13종 · API 10종**이다(2 → 9). ★**«타이틀 지목» 축의 후보는 0건** —
  핀이 6, `hardening.rs` 가 나머지 3을 이미 덮는다 ⇒ 목록의 값은 «할 일»이 아니라 «지금이 옳다»는 독립 확인이다.
  ★★**[게이트② 반려 · `-fix`] 그 «0» 의 «경계»를 박았다** — 0 은 «9행 표»에서만 도출되고,
  ★**타이틀이 «없는» 3커밋까지 세면 후보 1건이 남는다**(`1f0e52e` 의 `String.<init>([B)/([C)` null NPE 가
  핀에도 `hardening.rs` 에도 없다 · 정본 **§10-6-b**). 가드 구현은 별 판단이다.
  ★그리고 술어 수치도 정정했다 — 「12중 10 · 75중 69」는 `LC_ALL=C` 의 «바이트 클래스» 오탐이었고
  옳은 값은 **12중 9 · 75중 61** 이다(그 9건은 표의 9건과 **같은 집합** ⇒ 그 브랜치에선 «완전 판별자»).
  ★**술어의 한계를 함께 적었다** — 문구 술어(`Trace-specified` 등)는 **8건**만 잡고 `9be0ea3`(박정석_영웅탄생)을
  놓친다 ⇒ 최종 9건은 사람이 12커밋을 통독해 나왔다. 정본 **§10-6**.
  ★범위: 브랜치 2 · 태그 0 · 커밋 1,103 · 고유 87 · PR 30 · 이슈 **비활성화**. `main` 의 75커밋엔 관측 **0건**
  ⇒ 「버린 fork」는 저장소가 아니라 «`wie-ktf-hardening` 브랜치»다. ★코드 변경 0 · 저장소 쓰기 0.
  ★**착지 형태 = merge commit(부모 2개)** — 등재 repo 라 스쿼시 금지. ★형제 **#80** 은 여전히 열려 있다.
- 2026-09-05: **커버리지 재측정 «기계화» 결정** (PR #78 착지 · `wie-coverage-remeasure-mechanize-decision`
  · 채택 제안 `2026-09-04-worklog-mandate-reopen-threshold#p0`) — ★**결정: 기계화한다. 단 «임계 판정»이
  아니라 «재측정 약속»을.** 두 축은 오탐 성질이 정반대다 — 비율은 «정당한 미달»이 구조적으로 들어 있어
  (후속 없는 회차·upstream 동기 머지) red 로 걸면 **지킨 회차를 물고**, 「기한이 지났다」는 판단이 아니라
  **사실**이라 오탐이 0이다. ⇒ `scripts/check-worklog-coverage.mjs`(측정 정본) + 기록 파일 +
  `engine-contract.yml` 상시 스텝. ★`AGENTS.md` 의 셸 3줄은 스크립트 호출로 «교체»했다(정본 두 벌 금지).
  ★**지금 한 번 쟀다 — 10/10 = 100%**(착지 회차 13 ⇒ 규칙이 정한 첫 기한이 실제로 찼다).
  개악 4칸: OVERDUE red · BELOW-UNANSWERED red · `reopened` 기록 시 해소 · 얕은 클론 **fail-closed**.
  ★**형제 #77 이 먼저 착지해 원장 2파일이 충돌했고**, 이 게이트③이 2-c⒜(원장 한정 승인)로 합집합 해소했다
  (코드 충돌 0 · 버린 것은 «플레이스홀더» 2줄뿐 — 항목과 공존할 수 없다).
  ★**착지 형태 = merge commit(부모 2개)** — 등재 repo 라 스쿼시 금지. ★형제 **#79** 는 여전히 열려 있다.
- 2026-09-05: **KTF·LGT 키 «도달» 단언 + 픽스처 레시피** (PR #77 착지 · `wie-ktf-lgt-drawing-fixtures-for-key-reach-assertions`
  · 채택 제안 `2026-09-04-featurephone-keypress-reaches-guest#p0`) — J2ME 만 덮던 도달 축을 **두 경로에 다** 붙였다.
  ★**근인은 픽스처가 아니라 «레시피 부재»였다** — `helloworld_{ktf,lgt}.zip` 은 `dlunch/wipi` 에서 빌드된 ARM
  게스트인데 만드는 법이 wie 어디에도 없었다 ⇒ 핀된 rev 로 클론·주입·빌드하는 스크립트를 세웠다(게스트 소스 포함).
  ★**도달의 증거 = 게스트 stdout**(픽셀 아님) — `TestScreen` 이 프레임버퍼를 안 갖고, stdout 은 «정확한 정수»를
  단언하게 하며, 헤드리스라 3 OS 전부 돈다. ★이 시험들이 §4b·§4c 정적 핀을 **행동으로 관통**한다.
  ★개악 3칸 전부 제품 실물 — 값 오배선(`key:56`)·전달 절단(`""`)·★**LGT 전용 절단(LGT red · KTF green)** 으로
  **두 시험의 독립**까지 보였다. `cargo test --all` **137 → 139**.
  ★**주의**: 새 픽스처는 `--inject` 없이 `wie_validate` 를 돌리면 FAIL 이 정상이다(키 전엔 검은 화면).
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재 repo 라 스쿼시가 금지다.
  ★**형제 PR #78·#79 가 원장 2파일을 함께 만진다** — 이 착지가 그쪽 base 를 움직인다(그쪽 게이트③이 해소한다).
- 2026-09-04: **게임 액션 표 둘 핀** (PR #76 착지 · `wie-key-contract-pin-game-action-tables`
  · 채택 제안 `2026-09-04-ktf-third-key-table-pin#p1`) — 게스트가 「방금 받은 키가 어느 방향인가」를
  되묻는 통로 둘이 **전달 경로 밖**이라 아무 검사도 안 보고 있었다(틀리면 «위를 눌렀는데 아래로 간다»).
  ★**공통 5행은 값이 전건 동일**(1·6·2·5·8 = MIDP 사양) ⇒ 계약에 **한 번만** 적고 두 표를 그 하나에 대조.
  ★**갈리는 자리는 정확히 둘이고 «의도»다** — WIPI 만 `CLEAR→99` · 미매칭 반환 `0`(MIDP) ↔ `key`(KTF).
  통일하면 그 플랫폼이 깨지므로 **«갈린다»는 사실을 계약에 못박았다**(문서화된 적이 없었다).
  ★개악 5칸 전부 제품 실물 — 두 표가 «각각» 울었고, ★**위치자 개명 개악이 fail-open 구멍을 하나 잡아냈다**
  (접두 일치 → 괄호 고정으로 수정).
  ★**정적 검사 «착지 기준» 94 → 107 pass**(이 PR 이 더하는 §4d 검사 **13**개 — 집합 차로 확인).
  ※그 회차 본문의 「94 → 101」은 부정확했다 — 그 브랜치의 base 는 **88**(PR #74 착지 시점)이라 «101» 은 88+13 이고,
  «94» 는 형제 #75 착지 후 main 값이다. 두 기준이 섞여 있었다.
  ★**형제 PR #75 와 같은 두 파일**을 만진다 — 나중에 착지하는 쪽에서 충돌 해소가 필요할 수 있다.
  ★**착지 형태 = merge commit(부모 2개)** — 등재 repo 라 스쿼시 금지.
  ★**형제 #75 착지로 한 번 CONFLICTING 이 났고**(원장 2파일) `…-conflict-resolve` 회차가 «해소만» 했다(판단 필요 hunk 0 · 계약값 8축 바이트 동일) ⇒ 총괄이 재검 생략을 판정했다.
- 2026-09-04: **«통화»·«종료» 두 키를 세 표 전부에서 잠갔다** (PR #75 착지 · `wie-key-contract-pin-call-and-hangup`
  · 채택 제안 `2026-09-04-ktf-third-key-table-pin#p0`) — 화면 버튼 2개가 **지금도 눌리는데**
  `CALL`·`HANGUP` 만 계약 어휘 밖이라 §4·§4b·§4c 어디에서도 안 잠겨 있었다.
  ⇒ 어휘 2행 + 세 표에 각 2행(`CALL` 10/-10 · `HANGUP` -1/-11). ★**검사기 0줄 · 제품 코드 0줄** —
  세 블록이 전부 계약 데이터를 순회한다는 것을 **루프 머리로 확인**했다(정적 **88 → 94 pass**).
  ★**개악 3건을 «표마다 하나씩»** 제품 실물에 심어 셋이 각각 울었다(무개악 94 pass / 0 violation).
  ★왕복 Scenario A 스윕이 자동으로 **20 → 22 codes**(29/29 유지).
  ★★**제안의 「otterpebble 과 같은 롤아웃 필요」는 «과했다»** — 실측상 그쪽은 이 계약 파일을 **읽지 않는다**
  (수신부는 sha256 검증 + 핀 범프뿐) ⇒ 교차 저장소 롤아웃 불요.
  ★**`VOLUME_*` 2행은 일부러 열어 뒀다** — 셸이 보내지 않아 «도달 불가»(24행 중 22행 잠김).
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재 repo 라 스쿼시가 금지다.
  ★**형제 PR #76**(게임 액션 표 · 게이트② 진행 중)이 «같은 두 파일»을 만진다 — 이 착지가 그쪽 base 를 움직인다.
- 2026-09-04: **KTF 의 «셋째 키 표» 핀** (PR #74 착지 · `wie-ktf-third-key-table-pin-wipi-from-midp-raw`
  · 채택 제안 `2026-09-04-keypress-remaining-17-keys#p0`) — KTF(WIPI) 게스트는 키 번호를 **한 번 더**
  바꿔서 받는다(`CardCanvas` 가 `Canvas.keyPressed` 를 **재정의** → `WIPIKeyCode::from_midp_raw` → `Card.keyNotify`).
  ★**그 표를 잡는 것이 정말 0이었다**(검사기·계약·시험 전수 실측) ⇒ §4b 와 **같은 방식**으로 **§4c** 신설.
  ★**착수 재확인에서 등재값이 틀렸다** — 「20행」이 아니라 ★**24행**이다(초과 4 = `CALL`·`HANGUP`·`VOLUME_*` ·
  계약 어휘 밖이라 미핀으로 남겼다). 핀 범위는 **어휘 20종**(§4b 와 같은 키 집합).
  ★★**[정정 · 게이트② 반려] 초판이 그 넷을 묶어 「셸이 보내지 않는다」로 적은 것은 «거짓»이다** — 상류 실측상
  `VOLUME_*` 만 참이고 ★**`CALL`·`HANGUP` 은 «오늘 보낸다»**(피처폰 화면의 «통화»·«종료» 버튼 2개).
  ⇒ ★**사용자가 실제로 누르는 2키가 세 표 전부에서 미핀**이다 — 어휘 확장은 §4b 접촉이라 **제안**으로 올렸다.
  ★**「5를 눌러 8이 입력된다」는 참**이다 — 개악 출력이 `as 56 (Self::NUM8), contract pins 53` 이고 **56 = ASCII '8'**.
  정적 **68 → 88 pass** · 개악 5칸(팔 스왑·계약 변조·행 삭제·위치자 개명 → fail-closed·무개악 green) 전부 제품 실물.
  ★**J2ME 결론과 충돌하지 않는다** — 그쪽은 MIDP 경로 범위였고 이 표는 그 밖이다(재개 조건 무접촉).
  ★**남는 구멍**: 게임 액션 표 **둘**(`Canvas::getGameAction` · WIPI `Display::getGameAction`) 미핀 — 제안으로 올렸다.
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf:23` 등재 repo 라 스쿼시가 금지다(부모가 접히면 계보가 소실된다 · 자매 repo `rustjava` 가 「스쿼시했으면 behind 0 → 12」를 «수»로 보였다).
- 2026-09-04: **남은 17종 키 사각 처분 — «닫았다»** (PR #73 착지 · `wie-featurephone-keypress-remaining-17-keys-close-or-declare`
  · 채택 제안 `2026-09-04-featurephone-keypress-reaches-guest#p1`) — ★**「17개의 구멍」이 아니었다.**
  전달 경로를 홉으로 갈라 재니 ⒜**키 무관** 부분(이벤트 큐 → `Canvas::handleKeyEvent` → `keyPressed`)은
  ★**키로 분기하는 `match` 가 0건**이라 Scenario D 의 3증인으로 **이미 전건 닫혀 있었고**,
  ⒝**키별** 부분은 ★**표가 «둘»**인데(`parse_key` · `MIDPKeyCode::from_key_code`) 소스 핀이 **첫 표만** 지켰다.
  ⇒ ★**실제로 열린 것은 «둘째 표의 17행»**이고 그 결함은 「7을 눌렀는데 8이 입력된다」로 나온다(JS 표면 불가시).
  ⇒ Scenario D 를 넓히지 않고(그건 ⒜를 17번 더 증명하는 일이다) **계약 파일 + 정적 핀 §4b** 로 닫았다 —
  정적 **48 → 68 pass** · 왕복 **29/29 유지** · 개악 2종(팔 스왑 · 판별식 변경) 모두 red.
  ★**재개 조건**: 방아쇠는 «수»가 아니라 ★**«열거 목록에 없는 새 히트»**다 — 전달 **5파일**의 `match` **10건**을
  전건 열거하고 포함/제외를 지목했다(표 3 · 경로 안·키 무관 5 · 경로 밖 1 · 독 주석 1).
  ★초판의 「2 이상이면 재개」는 태어난 날 이미 10이라 거짓이었다(게이트② 반려 · `-fix` 로 교체).
  ★**남는 구멍**: KTF/WIPI 의 **셋째 표** `WIPIKeyCode::from_midp_raw` 20행 미핀 — Non-goal 이라 제안으로 올렸다.
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf` 등재 repo 라 스쿼시가 금지다(부모가 접히면 behind 가 0 → N 으로 튄다 · 자매 repo `rustjava` 가 «수»로 보였다).
- 2026-09-04: **미이식 하드닝 2축 처분 — «둘 다 이식»** (PR #72 착지 · `wie-unported-hardening-two-axes-decide-with-a-corpus-probe`
  · 채택 제안 `2026-09-04-upstream-realign-p1-pin-plus33#p0`) — ★**결정의 근거가 바뀌었다.**
  상수풀 프로브(우리 아카이브 전수 · `.class` **226개**)는 게스트 호출부 **0/0** 이었으나 ★**그 표본으로는 답이 안 난다**
  (`test_data` 3건은 우리가 만든 것 · `AromaWIPI` 는 게임이 아니라 플랫폼 라이브러리다).
  ★★**두 번째 측정이 갈랐다** — ⒜플랫폼은 `StringBuffer.insert` **9종** · `Timer.schedule` **4종+cancel** 을 선언하는데
  핀은 **0종 / 2종**뿐이고, ⒝★**버린 fork 의 커밋 로그에 «실제 타이틀»이 박혀 있었다**
  (`Timer.schedule` → **소울카드마스터2** · `StringBuffer.insert` → **미니고치** · 둘 다 「trace-specified as method-not-found」).
  ⇒ 규칙 ⒜(호출이 있으면 이식)가 **둘 다**에 걸린다. ★**§9-2 의 「등급이 낮아 미룬다」 판단을 이것이 정정한다** —
  증거는 우리 의존성 이력 안에 «이미» 있었고 아무도 거기를 보지 않았다(§8-4⑸ 프로브와 같은 형태의 실수).
  이식은 `hardening.rs` 의 `add()`(감싸기가 아니라 «추가» · 핀이 나중에 같은 메서드를 가지면 덮지 않고 신고) ·
  시험 2건은 **게임 파일 0 · 시계 비의존**(Timer 는 `TimerThread` 가 읽는 두 필드로 단언). 정본 §10.
  ★**남은 부분 표면**(`insert` 9중 1 · `Timer` 5중 3 · `cancel` 없음)은 ★**재개 조건을 «수»로** 달았다 — 누락 오버로드를 지목한 게스트 실패 **1건**.
  ★★**[게이트② 반려 승계 · `…-fix`] 필수 3건 해소**: ⒜**`coverage` red** — 두 시험을 한 파일에 둔 것이 `cargo tarpaulin` 아래서
  segfault 했다. ★로그가 «바이너리»만 지목해 미측정이었고, **시험당 바이너리 1개로 갈라 CI 로 재측**하니 **둘 다 통과**
  ⇒ ★**어느 한 시험도 원인이 아니라 «동거»가 트리거다**(정본 **§10-5** · 시험 삭제 0 · `#[ignore]` 0).
  ⒝**CI 를 게이트 목록에 박았다**(`AGENTS.md` §DoD — 네 게이트 뒤 `gh pr checks`). ⒞**§9-2 인라인 정정** — 축 8·9 의
  「미이식」 3자리에 각각 박았다(§10 의 정정문만으로는 §9-2 에 착지한 독자가 낡은 표를 읽는다).
  ★**착지 형태 = merge commit(부모 2개)** — `wie` 는 `contracts/upstream-sync-repos.conf` 등재 repo 라
  스쿼시가 금지다(계보가 접힌다 · 자매 repo `rustjava` 에서 스쿼시 3회가 족보를 원점으로 되돌렸다).
- 2026-09-04: **P1 집행 (PR #71 `19955ba1`) — `Jun025/RustJava` 핀 이탈(`dlunch/RustJava@5b84dd1`, +33) + 하드닝 3축 이식**
  (`wie-upstream-realign-p1-execute-pin-plus33-and-cost-hardening-port`) — ★**총괄이 §8-6 권고를 채택했고
  이 회차가 집행했다.** `[patch]` 표 삭제 ⇒ ★**fork 의존 소멸**(`Cargo.lock` 의 `Jun025` **0건**).
  ★**API 파열은 예상 ≥7 ↔ 실제 «11개소 / 7파일»** — 예상 밖 둘은 `ClassInstance::{identity, shallow_clone}`(3 impl)과
  `ArrayClassInstance: ClassInstance` 승격(1 impl). ★`shallow_clone` 은 게스트 객체를 **새로 할당해 필드를 복사**한다
  (구조체 복제는 같은 주소를 가리켜 «복제본에 쓰면 원본이 바뀐다»).
  ★**하드닝 6축 전부 사라졌고 3축을 wie 안으로 이식**했다 — `wie_jvm_support/src/hardening.rs`(본문 103·시험 99·배선 19).
  ★**fork 없이 됐다**: `find_rustjar_class` 가 프로토를 JVM 에 넘기기 «전»에 wie 가 본문을 감쌀 수 있다.
  ★**기준은 줄 수가 아니라 «실패의 등급»** — 이식분은 null 이면 호스트가 패닉(개악 대조로 재현), 미이식 2축은
  메서드 부재라 Java 레벨에서 잡힌다.
  ★★**[2026-09-04 정정 · 게이트② 반려] 종전의 「축 5(pending GC 루트)는 fork 없이 불가·영구 미복구」는 «거짓»이다** —
  새 핀이 **같은 창을 다른 설계로 이미 닫아 놓았다**(`GlobalRef<Thread>` 를 `ThreadStartProxy` 가 들고,
  `determine_garbage` 가 `global_references` 를 루트로 돌고, `Drop` 이 콜백 종료 시 해제한다).
  ★**왜 틀렸나 — 프로브는 «`pending` 이라는 fork 의 식별자»를 세지 «보호»를 세지 않는다.**
  ⇒ ★**상실은 6축이 아니라 5축이고, 갚지 못한 값은 «미이식 2축»뿐이다.** 정본 §9-2.
  정본 = `docs/upstream-realign-verdict.md` **§9**. 4게이트 green · `cargo test --all` **133 passed**.
- 2026-09-04: **키 입력 «도달»을 행동으로 단언 — 왕복 검사 Scenario D 신설** (PR #69 `f4569f3f`,
  `wie-featurephone-keypress-reaches-guest-behavioral-axis` · 채택 제안
  `2026-07-22--featurephone-engine-contract-selftest#p0`) — 종전에 키 축을 보던 것은 둘뿐이었다:
  ⒜왕복 검사 Scenario A 의 「어휘 **20종**을 눌러도 **예외가 안 났다**」 ⒝`check-engine-contract.mjs`
  §4 의 **소스 핀**(`wie_web/src/lib.rs` 의 `fn parse_key` 본문에서 `"UP" => KeyCode::UP` **쌍**을 읽는다).
  ★**둘 다 «게스트에 도달했는가»는 보지 않는다.** ⇒ 픽스처의 `keyPressed()` 가 **받은 MIDP 코드만큼 넓은 막대**를
  그리게 해서 캔버스가 **어느 코드가 도달했는지**를 말하게 했다(대표 키 **3종** — 소프트/숫자/방향).
  ★**개악 대조**: `key_down` 이 이벤트를 **버리게** 하면 소스 핀은 **48 pass / 0 위반**, Scenario A 도
  **✓ 20 codes** — ★**둘 다 못 잡는다.** Scenario D 만 **3건 red**. 오탐 0(기존 26건 전건 통과 · **29/29**).
  ★**제품 코드 변경 0**(`wie_web/src/lib.rs` 무접촉 — 개악은 되돌렸다) · CI 워크플로 변경 0.
- 2026-09-03: **P2 — ⒟ go/no-go 측정 회차** (PR #68 `7fb11c34`, `wie-upstream-realign-p2-gate-measurement-before-p1` + 반려 승계 `-fix`) —
  ★**총괄 결정으로 P2 를 P1 «보다 먼저» 돌렸다.** 판정 = ★**「P2 는 이 머신에서 측정 불가」**이고
  ★**사유가 «둘»이다**: ⒜코퍼스 부재(구조적 · Constraint 9 · 종전부터 알던 축) ⒝★**러너 부재 —
  upstream 에는 `wie_cli`·`wie_validate`·`scripts/` 가 «없고» 크레이트가 `wie_ktf`→`wie-ktf` 로
  전면 개명됐다** ⇒ 「upstream 체크아웃 + 우리 `smoke_gate.sh`」라는 §6-P2 의 처방 자체가 성립하지 않는다.
  ★**차이표는 지어내지 않았다** — 잴 수 있는 범위(커밋된 픽스처 2건)만 재서 **2/2 ↔ 2/2**(신규 FAIL 0).
  ★**대신 P1 축에서 «측정»이 나왔다**: §6-P1 의 「하드닝 있고 파열 없는 중간 rev」 가설은 ★**반증**됐고
  (`ClassDefinition` 파열이 **+1·+2**에 있다 — `cargo check` rc=101 로 확인), 그 자리에서
  ★**비용 계단 2 → 5 → 11 → 220**이 드러났다. 정본 = `docs/upstream-realign-verdict.md` **§8**.
  ★**제품 코드 변경 0 · `Cargo.toml` 무접촉 · upstream 발신 0.**
- 2026-09-01: **워크로그 «회차 의무» 기각 + 2026-07-22 백필** (PR #67 `ec1b7027`,
  `wie-worklog-mandate-decision-and-2026-07-22-backfill`) —
  ★**결정: 의무화하지 «않는다».** 규약 착지(`92c25276`) 후 착지한 **3회차 전건**이
  워크로그를 썼고(**3/3 = 100%** · `-fix` 승계까지 세는 커밋 축으로는 4/4), 직전 19회차는 **0/19** 였다 ⇒ 무조건 의무를 얹어 얻을 커버리지가 남아 있지
  않고, 이미 있는 **조건부** 문장이 면제까지 포함한다. ★그래서 `AGENTS.md`·검사기 **무접촉**이다.
  백필은 `limits` 3줄 중 **1줄만** 승격했다(1줄은 `1853d49e` 가 이미 닫았고 1줄은 Constraint 9 의 영구 경계).
  ★**분모가 3회차뿐**이라 기각은 영구 판정이 아니다 — 재측정 시점·임계는 이 회차의 유일한 후속 제안이다.
  ★★**[2026-09-04 · `wie-worklog-mandate-rejection-needs-a-reopen-threshold`] 그 제안이 채택돼
  «되돌릴 조건»이 박혔다 — 정본은 `AGENTS.md` §Landing paperwork **한 곳**이다.**
  ★여기에 주기·임계·방법을 옮겨 적지 마라(두 곳에 있으면 갈린다). ★위 「3회차 전건」 근거는 **사료로 그대로 둔다** —
  결정 자체는 바뀌지 않았고, 붙은 것은 **언제 다시 재고 얼마면 뒤집는가**뿐이다.
- 2026-08-27: **upstream 재정렬 판정 착지** (PR #66 `0cb309b4`, `wie-upstream-realign-verdict`
  + 반려 승계 `-fix`) — `docs/upstream-realign-verdict.md` 신설(3분류 표 + 갈래 판정 + LGT 코드
  대조 + RustJava fork 실측). ★**제품 코드 변경 0 · upstream 발신 0.** 채택 갈래 ⒟, 집행은 후속
  P1~P4(위 `## 다음` ①). ★**이 줄은 2026-09-01 회차가 옮겼다** — 착지 후에도 `## 진행중` 에 남아
  있어 바로 위 머리글의 「착지 즉시 `## 완료` 로 옮겨라」와 어긋났다(그 시점 열린 PR **0건** 실측).
- 2026-08-27: **외부 apt 장애가 착지를 막던 경로 차단 + `fail-fast: false` 착지**
  (PR #65 `250d7e4c`, `wie-rust-ci-beta-leg-blocks-gate-on-external-outage`) — `rust.yml` 1파일.
  ubuntu 스텝이 `bash -e` 아래에서 `apt update` 의 100(우리가 쓰지 않는 `packages.microsoft.com` 403)에
  죽어 `apt install` 에 도달조차 못 하던 것을 「update 는 best-effort · install 은 fatal」로 갈랐다.
  ★게이트를 무르게 하지 않았다 — `libasound2-dev` 를 못 받으면 여전히 red 다. 제품 코드 변경 0.
- 2026-08-27: **화면을 그리는 초소형 픽스처 + 픽셀 계수 실단언 승격 착지** (PR #64 `1853d49e`,
  `wie-drawing-fixture-makes-pixel-count-a-real-assertion`) — `scripts/make-draw-fixture.mjs`
  (JDK 없이 class 파일 바이트를 직접 찍는다 · jar 는 **커밋하지 않고** 메모리에서 서빙 —
  `*.jar` 는 git-ignore + 유출 감사가 tracked jar 를 거부한다) + `contract-roundtrip.mjs`
  **Scenario C**(J2ME · `nonBlackPixels() > 0` 실단언). 26/26 green · 개악 시 25/26 red.
  ★**게이트③ 이 1회차는 `blocked`** 였다 — 외부 apt 장애로 `ci-presence` rc=1(그 red 가 위 `## 진행중`
  회차를 낳았다). 2회차가 `rerun` **1회**로 green(rc=0)을 받아 착지시켰고, 머지 후 main 4런 전건 green ·
  Pages prod 배포 + D1 원격 마이그레이션 실집행 · self-verify `https://wie-web.pages.dev` **200 · 콘솔 0에러**.
- 2026-08-26: **회차 워크로그 `.json` + `proposals` 규약 이식 착지** (PR #63 `92c25276`,
  `wie-worklog-json-proposals-convention`) — cockpit 「후속 작업 추천」이 wie 를 구조적으로 0건으로
  읽던 것을 풀었다(착수 실측 `/api/proposals` `derived.coverage`: wie `json:1 · md:0 · proposals:0`).
  `AGENTS.md` §Landing paperwork 에 워크로그 `.json` 의무 + 소비 키 표, `scripts/check-worklog-json.mjs`
  5축 잠금, `engine-contract.yml` 의 항상 도는 잡에 편입. ★**소급 변환 0** · 코드·의존성 변경 0.
- 2026-08-21: **문서의 맨손 원격 변이 wrangler 명령 + `web.yml` 판본 핀 단일화 착지**
  (PR #62 `4cd0f43e`, `wie-cf-setup-bare-d1-create-and-wrangler-pin-consolidate`) —
  ①`docs/CLOUDFLARE_SETUP.md` 의 `wrangler d1 create` 를 계정 핀 접두로 + `docs/COMPLIANCE.md` 의
  원격 변이 7건도 블록마다 핀 ②`web.yml` 판본 리터럴 **5곳 → job env `WRANGLER_VERSION` 1곳**
  ③`CLOUDFLARE_ACCOUNT_ID` 가 비면 배포 전에 시끄럽게 죽는 가드 스텝. 코드·의존성·판본 변경 0.
- 2026-08-20: **Pages 가 거부하는 `account_id` — 핀을 env 로 이전 착지** (PR #61 `eef08184`,
  `wie-wrangler-pages-account-id-breaks-prod-deploy-fix`) — `wrangler.toml` 의 `account_id` 제거 +
  `package.json` 의 `deploy`·`db:migrate:remote` 에 `CLOUDFLARE_ACCOUNT_ID` 가드 + `web.yml` 에
  PR 에서도 도는 Pages 설정 검증 스텝. ★**「고쳤다고 본다」가 「고쳤다」로 확정됐다** — 머지 후
  `web.yml` on main run **32375925645 success**(2026-08-20T13:43Z), 직전 red run 32364381443 대비.
- 2026-08-20: **wrangler `account_id` 고정 착지** (PR #59, `wie-wrangler-account-id-pin`) —
  `wrangler.toml` 최상단에 otterpebble 계정을 박아 «어느 계정으로 배포되는지»를 파일이
  선언하게 했다. 게이트②는 08-20 에 approve 였으나 아래 beta clippy red 가 게이트③ `ci-presence` 를
  막고 있었고(1회차 `-merge` 는 그 rc=1 로 **정확히 거부**했다), 그 red 착지 후 base 를 당겨 착지했다.
  ★★**그리고 이 착지가 prod Pages 배포를 red 로 만들었다** — Pages 는 `account_id` 키를 문법으로
  거부한다(`web.yml` run 32364381443). ⇒ 위 「파일이 선언한다」는 **더 이상 참이 아니다**(env 핀으로
  이전 · 위 `## 진행중`). 사료로 남긴다.
- 2026-08-20: **beta clippy `double_must_use` repo 전역 CI red 해소 착지** (PR #60 `7a49aff0`,
  `wie-rust-ci-beta-clippy-double-must-use-red`) — `async-trait` 0.1.89 → 0.1.92(`Cargo.lock`) +
  `wie_ktf` `find_java_method` 국소 `allow` 1줄. 코드 동작 변경 0.
- 2026-08-18: **`STATE.md` 착지 잔재 정리 + `misc_unk9` 에러 문면 잠금 착지** (PR #58 `6f9dbae7`,
  `wie-state-landed-pr56-residue-and-misc-unk9-error-lock`).
- 2026-08-18: **LGT SVC 0x581 등재 착지** (PR #57 `bccf11f1`, `wie-lgt-svc-1409-unknown-and-state-stale-next`) —
  `WIPICSvcId::MiscUnk9 = 0x581` + `WieError::Unimplemented`(모듈·인덱스·인자 4개). 영웅서기5 LGT 는 여전히
  이 지점에서 멈추지만 로그가 「알 수 없는 SVC」에서 「misc index 9 미구현」으로 바뀌었다.
- 2026-08-15: **`STATE.md` 재판정 착지** (PR #56 `225392e8`, `wie-lane-restart-upstream-carryover-and-main-divergence`) —
  `## 진행중` 5행을 `## 완료` 로 이관 + `## 다음` 4항 재판정. 이 레인의 25시간 공백을 끝낸 회차다.
- 2026-08-16: `AGENTS.md` **재구조화 착지** (PR #54 `41721671`, `wie-agents-md-declarative-restructure`) —
  Goal/Constraints/DoD/사건 대장 골격 + Hard Req 12항 → 잠금 테이블. **16,888 → 13,099 바이트**(−22.4%).
  지도 4종은 `docs/architecture.md` 로 **이관**(삭제 아님). ★부수 실측 2건: ①`**/Cargo.toml` 은
  `publish-artifact.yml` 발행 경로라 **주석만 고쳐도 릴리스+dispatch 발화** ②**MCP 등록 0개**.
- 2026-08-17: KB 실행 경로 정정 착지 (PR #46 `7514d552`) — `~/Documents/dev/wie` → `~/work/otterpebble/wie`.
- 2026-08-08: 공급망 대장 **A-2·A-3 착지** (PR #55, `wie-supply-chain-cargo-updates-a2-a3`) —
  `event-listener 5.4.1→5.4.2` · `spin 0.12.0→0.12.2`. `Cargo.lock` 만 변경 · 코드 변경 0.
- 2026-08-03: **완주의 정의를 «PR 을 열어 둔 상태»로 개정** (PR #53, `wie-agents-md-gate2-contradiction-fix`) —
  헌장이 워커에게 머지를 «지시» 하던 문장 5곳(`AGENTS.md` 3 + `CONTRIBUTING.md` 2) 개정 + `CLAUDE.md` «완주 규율» 신설.
- 2026-08-03: **공급망 추적 대장 등재** (PR #52, `wie-rustsec-advisory-sweep-batch2`) —
  `docs/project-kb/02_status.md` 에 권고·공급망 3건 + upstream 이슈 9건 표. `#1292` 종결로 실질 개발 후보 3→2건.
- 2026-08-01: `.direnv/` gitignore (PR #51).
- 2026-07-31: **PR #45 잔재 착지** (PR #50, `wie-pr45-orphan-close-and-remnant-land-r2`) —
  `.dev.vars` read-deny + session discipline. ★**PR #45 본체는 미머지 종결**(2026-07-31 CLOSED);
  살릴 값만 골라 옮긴 것이 #50 이다. 이 경위가 아래 «해소된 항목» ③의 근거다.
- 2026-07-22: featurephone 소비 계약 드리프트 가드 (PR #36·#39) · main 브랜치 보호 코드 준비 (PR #43) ·
  security audit schedule red 정정 (PR #42). 상세는 `REPORT.md` 및
  `docs/worklog/2026-07-22--featurephone-engine-contract-selftest.json`.
  ★human-step 잔여: 운영자가 branch-protection ruleset 1회 적용 —
  `~/orchestrator/reports/wie-main-branch-protection.done.md` C항.

## 다음

**★① upstream 재정렬 집행 — 이 회차의 판정을 잇는 축**(정본 `docs/upstream-realign-verdict.md`).
채택 갈래 **⒟**. ★★★**[2026-09-04 갱신 · P1 집행 회차] P1 은 «끝났다» — 총괄 결정 대기 «아니다».**
총괄이 §8-6 권고를 **채택**했고(갈래 ⒝ · 핀 `5b84dd1`) 이 레인이 **집행**했다 — 집행 기록은
`docs/upstream-realign-verdict.md` ★**§9**. ★**「총괄 결정 대기」 문구를 되살리지 마라.**
★★★**[2026-09-03 갱신 · P2 회차] 순서 결정은 «내려졌다»** — 총괄이 **P2 를 P1 보다
먼저** 돌렸고 그 결과가 아래를 다시 썼다. 정본 = `docs/upstream-realign-verdict.md` ★**§8**.
- **P2**(측정전용) ★★**돌았다 — 판정 = 「이 머신에서 측정 불가」이고 사유가 «둘»이다**(§8-1).
  ⒜코퍼스 부재(구조적 · Constraint 9 · `find ~ -maxdepth 4 -name game_lab` **0건**)
  ⒝★**러너 부재 — 이 회차가 새로 찾은 축**: `upstream/main` 에는 `wie_cli`·`wie_validate`·`scripts/` 가
  **없고** 크레이트가 `wie_ktf` → `wie-ktf` 로 **전면 개명**됐으며 그쪽 바이너리는 창을 띄우는 앱이다
  (`--timeout` 없음 · JSON 판정 출력 없음). `smoke_gate.sh` 는 `cargo build -p wie_cli --bin wie_validate`
  에 하드코딩돼 있어 ★**빌드 대상이 해결되지 않는다.**
  ⇒ ★★**P2 를 살리려면 «코퍼스 있는 머신» + «`wie_validate` 772줄을 upstream 크레이트 위로 이식» 이
  «둘 다» 필요하다.** 종전 `size: M` 은 그 몫을 세지 않았다. ★**차이표는 지어내지 않았다.**
  ★잰 것: 커밋된 픽스처 2건이 양쪽에서 **2/2 ↔ 2/2**(신규 FAIL 0) — ★**코퍼스가 아니다. 부풀리지 마라.**
- **P1**(★★**집행 완료 · 2026-09-04**) `Jun025/RustJava` **핀 이탈** — ★**끝났다.**
  ★**핀 = `dlunch/RustJava@5b84dd1`(+33)** · `[patch]` 표 **삭제** · `Cargo.lock` 의 `Jun025` **0건** ·
  `cargo tree` 상 `java_class_proto`·`java_constants`·`java_runtime`·`jvm`·`jvm_rust` **전건 dlunch@5b84dd1**.
  ★**API 파열은 예상 ≥7 ↔ 실제 «11개소 / 7파일»**(§9-1) — 예상에 없던 것 둘:
  `ClassInstance::{identity, shallow_clone}`(3 impl · `shallow_clone` 은 **게스트 객체를 새로 할당해 필드를
  복사**해야 했다) · `ArrayClassInstance: ClassInstance` 승격(1 impl 재구조화).
  ★**하드닝 6축은 전부 사라졌고**(프로브 재실행) ★**그중 3축을 wie 안으로 «이식»했다** —
  `wie_jvm_support/src/hardening.rs`(본문 103 · 시험 99 · 배선 19). ★**fork 없이 됐다**:
  `find_rustjar_class` 가 `get_runtime_class_proto` 의 프로토를 JVM 에 넘기기 «전»에 wie 가 본문을 감쌀 수 있다.
  ★**고른 기준은 줄 수가 아니라 «실패의 등급»이다** — 이식한 3축은 null 이면 ★**호스트가 패닉**하고
  (개악 대조로 `jvm/src/class_instance.rs:108` `Option::unwrap()` 재현), 미이식 2축(8·9)은 **메서드 부재**라
  Java 레벨에서 시끄럽게 잡힌다. ★**축 5(pending-thread GC 루트)는 «불가»** — 34줄 중 25줄이 `jvm` 크레이트
  내부라 wie 가 닿을 이음매가 없다(★fork 없이는 영구 미복구 · 이 회차가 갚지 못한 유일한 값).
  ★**4게이트 green · `cargo test --all` 133 passed** · ktf·lgt helloworld ok — ★**단 green 을 «하드닝 보존»의
  증거로 읽지 마라**(§9-5). 보존의 증거는 프로브와 개악 대조뿐이다.
  ★**다음 칸**: 계단의 종점은 갈래 ⒜(`ba5797b`(+47) · **≥222곳** + crates.io 개명)이고 ★**이 회차는 거기까지
  가지 않았다.** `+34`(`current_class_loader` 비공개화 6곳 · 공개 대체 없음)가 그 앞의 벽이다.
- **P3**(L·med·★**선행 = P2 아님**. P2 가 답을 못 내므로 P1 결정 뒤로 붙인다) `wie_web` → `wie_featurephone` **개명**(upstream 이 같은 이름을 자기 용도로 쓴다)
  후 upstream 을 base 로 ③ 오버레이 재적용 + `compile_model.rs` **122줄 이식** + ★**엔트리포인트 규약 정합**
  (upstream `LgtEmulator` 는 `application.jar` 를 찾고 우리는 `00000000.jar` 를 넘긴다 — ★«부수 발견»이
  아니라 **작업목록 리터럴 항목**이다. 조용히 깨지는 것은 목록에 없으면 잊힌다). ★여러 회차로 쪼개라.
- **P4**(M·low·P3 와 병행) ② 를 upstream PR 로. ★**IP 방침 선 안쪽만**(#1239 2026-06-29
  「공개 문서 기반으로만 구현 · 펌웨어 리버스 계획 없음」) — `wipi_java` 공개 API 스텁 10종 +
  `canvas.rs` 단위테스트 9개는 **보낼 수 있고**, `docs/lgt_abi.md`·`docs/reference/` 는 **보내지 마라**.
★★**P3 의 DoD 에 리터럴로 박아라**: 머지 후 `git merge-base origin/main upstream/main` 이
`fa641a8a` 가 **아니어야** 한다. 그대로면 그 회차는 **실패**다 — 게이트③ `--squash` 가 upstream 계보를
평평하게 만들어 다음 회차를 **또 1,000커밋 넘게 뒤**에서 시작시킨다(동시 발권
`rustjava-upstream-sync-squash-defeats-convergence` 가 RustJava 에서 실측한 바로 그 형태).
★★**[2026-09-04 정정 · 게이트② minor] 그 `--squash` 금지는 «P3 한정»이 아니라 «이 repo 전체»다** —
`~/orchestrator/contracts/upstream-sync-repos.conf` 가 repo `wie` 를 등재하고 `bin/queue-lint` 검사 22 가
그 repo 의 **모든** `*-merge` 티켓에 `merge_strategy:` 선언을 요구한다. 「P3 한정」으로 읽히면
★**P1 착지 회차에서 그 규율이 빠진다.**

**② upstream #1260 후속 — ★«재발권 금지» 축이다(해제 조건 있음)**. 영웅서기5 LGT `Unknown SVC id 1409`.
1차 착수분(표 등재 + 미지원 예외)은 **PR #57 `bccf11f1` 로 착지**했다. 남은 것은 **misc index 9 의 정체 규명**이고,
★**이 repo 안의 근거로는 닫혀 있다** — 2026-08-19 게이트② 검수가 총괄 질의에 «동의한다 — 재발권 금지에 찬성»으로
답하며 근거 3개를 실측으로 댔다:
- repo 안에 **LGT misc 표가 없다**. KTF 쪽에는 `WIPICMiscMethodId` enum **자체가 없고**,
  유일한 misc 자료 `wie_ktf/.../method_table.rs` 의 `get_misc_method_table()` 은 **index 4 에서 끝난다**.
- ★**유추 경로도 닫혀 있다** — 두 구현의 인덱스 오프셋이 **모듈마다 다르다**(graphics **+1** / kernel **−3**).
  ⇒ KTF 인덱스 산술로 LGT index 9 를 옮겨 적으면 «근거 없이 맞아 보이는 문장»이 된다.
★**해제 조건 — 아래 셋 중 하나가 «새로» 생기기 전에는 이 축을 다시 열지 마라**:
⑴실기 덤프 ⑵다른 구현체의 LGT misc 표 ⑶게임 바이너리 호출부 디스어셈.
★조건 없이 발권하면 다음 사람이 **같은 벽에 다시 부딪힌다** — 그 왕복을 막으려고 여기 적어 둔다.

**③ upstream #1122 발권 판단** (대장 B · 실질 개발 후보) — 컴투스 삼국지 촉, 스테이지 5 부근 정지.
2026-08-15 실측: upstream **OPEN** 유지(2026-05-10 이후 정체), upstream 오너도 «에뮬레이터 버그로
추정 · 디버깅 난해»로만 답했다. ★**착수 전에 재현 가능성부터 판정하라** — 현 회귀 게이트는
**부팅+렌더까지만** 판정하므로(`scripts/smoke_gate_baseline.tsv` 의 `ktf/컴삼촉.zip PASS` 도 그 의미다)
스테이지 5 심도는 **기존 자동화로 도달하지 못한다.** ①보다 난도가 한 단계 높다.

**④ ~~(선택) 화면을 실제로 그리는 초소형 픽스처~~** — **해소**(2026-08-27,
`wie-drawing-fixture-makes-pixel-count-a-real-assertion` · 아래 `## 진행중`).
`scripts/make-draw-fixture.mjs` 가 사각형 하나를 칠하는 J2ME MIDlet jar 를 만들고,
왕복 검사 **Scenario C** 가 그 픽스처에 대해 `nonBlackPixels() > 0` 을 **실단언**한다
(실측 **1024 px / 2 frames**). ⇒ 왕복 검사가 **blit 회귀까지** 커버한다 — 코어가 프레임을
합성하지 못하거나 `WebScreen::paint` 가 캔버스에 닿지 못하면 CI 가 red 다.
★**남는 것 3가지(사실만)**: ⑴`helloworld_*.zip` 은 **그대로**이고 그쪽 픽셀 수는 여전히
**info-only** 다(전역 승격이 아니라 픽스처별 승격이다) ⑵새 픽스처는 **J2ME 경로**를 지난다 —
KTF·LGT 의 그리기 경로는 여전히 왕복 검사의 단언 밖이다 ⑶픽스처가 **exit 하지 않는다**
(Scenario C 는 첫 페인트에서 멈춘다) — 그리기 + 정상 종료를 한 픽스처로 함께 보진 않는다.

### «해소»로 내린 항목 — 다시 발권하지 마라 (판정일은 항목마다 표기)
> ★절 제목에 날짜를 박지 않는다. 회차마다 항목이 붙는데 제목 날짜는 안 따라와서
> 「언제 판정됐나」가 어긋났다(2026-08-19 게이트② 지적). 판정일은 **각 항목의 접두**로 읽어라.
- **[판정 2026-08-19]** ~~① PR #54 «게이트② 검수 상신 대기»~~ — **해소.** `feat/wie-agents-md-declarative-restructure` 는
  **MERGED 2026-08-16T14:13:11Z** · 머지커밋 `41721671f1b906a76e9298d08f2737c10cba7416`.
- **[판정 2026-08-19]** ~~② PR #46 «검수 상신»~~ — **해소.** `kb-path-update-2026-07-25` 는
  **MERGED 2026-08-17T04:14:07Z** · 머지커밋 `7514d5527263ba539a8f30e109bd8a2dbdbed0a8`.
  ⇒ 2026-08-19 실측 `gh pr list -R Jun025/wie --state open` = `[]`(**열린 PR 0**).
  ★**`-R Jun025/wie` 를 반드시 붙여라** — 이 워킹트리에서 `gh pr view 54` 는 fork 부모(`dlunch/wie`)로
  해석돼 2023년 dependabot PR 을 돌려준다. repo 를 못박지 않은 조회는 **다른 repo 를 잰 값**이다.
- **[판정 2026-08-15]** ~~게이트② approve 후 `-merge` 티켓~~ — **해소.** 당시 대기하던 PR #52·#53·#55 전건 머지 완료.
  `feat/wie-featurephone-engine-contract-selftest` 는 브랜치조차 없다 — 내용은 PR #36·#39(07-22)로 착지했다.
- **[판정 2026-08-15]** ~~로컬 main 분기(`0f13ab87`, ahead 1 / behind 12)~~ — **해소·무효.** 2026-08-15 실측 **0 / 0**.
  ★그 커밋을 «PR 로 착지시킬» 필요는 **없다**: PR #45 로 올라갔다가 미머지 종결됐고, 세 헝크 중
  `.claude/settings.json` deny 와 `.gitignore` 2줄은 **이미 origin/main 에 있다**(PR #50·#51 경유).
  남은 `CLAUDE.md` «자율운영 SOP» 블록은 **되살리면 안 된다** — 그 4개 조항은 전부
  `AGENTS.md` §Session Discipline 에 있고, 첫 조항 「확인 없이 이어서 완료한다」는 현행
  `CLAUDE.md` §착수 규율(**티켓 없는 착수 금지**)과 **정면으로 충돌**한다.
