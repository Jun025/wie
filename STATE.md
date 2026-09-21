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

> ★★**이 절은 «회차마다 고쳐 쓰지 않는다» — 정본은 열린 PR 목록이다**(2026-09-08 이관 · 티켓
> `wie-state-in-progress-top-insert-conflicts-every-landing` · 채택 제안
> `2026-09-07-state-completed-split-decision#p0`).
>
> ```sh
> gh pr list -R Jun025/wie --state open        # ← 「지금 무엇이 떠 있나」의 정본
> ```
>
> ★**왜 «고정 안내»인가**: 회차마다 이 절 **맨 위**에 자기 항목을 끼워 넣으면 형제 회차가 전부
> ★**«같은 삽입 지점»** 을 다투므로 착지 1회가 **열린 PR 전건의 충돌**로 증폭된다.
> `REPORT.md` 가 2026-09-07 에 같은 병으로 고정 안내가 됐고, **남은 절반이 정확히 이 절이었다.**
>
> ★★**«맨 아래 추가»(append)로는 안 고쳐진다 — 실측이다.** git 의 3-way 병합은 두 삽입 지점 사이에
> ★**단 1줄**만 있으면 깨끗이 붙고, **같은 지점이면 위든 아래든 충돌**한다(0줄 → CONFLICT · 1줄 → clean).
> 「맨 위」가 문제가 아니라 ★**«지점이 하나»** 인 것이 문제다. ⇒ 지점을 **없앴다**.
>
> ★**정렬키를 PR 번호로 두는 안도 이 저장소에선 안 통한다** — 형제 회차의 PR 번호가 **연속**이라
> (#120~#133 실측) 삽입 지점이 다시 서로 인접한다.
>
> ★**대가(숨기지 않는다)**: ⑴**오프라인에서 못 읽는다** — `gh` 없이는 「지금 뜬 것」을 알 수 없다.
> 그 대신 **착지한 것**은 아래 「완료」와 `docs/report/` 에 그대로 남으므로 **기록은 잃지 않는다.**
> ⑵**진행 중 회차의 «서술»이 이 파일에 남지 않는다** — 그 서술의 정본은 PR 본문과
> `~/orchestrator/tasks/<티켓>.md` 이고, 착지 시점에 「완료」로 들어온다.
>
> ★**되돌리는 법**: 이 인용 블록을 지우고 회차 항목을 다시 손으로 적으면 된다(코드·검사기 0).

## 완료 (최근)

> ★★**이 절은 «회차마다 고쳐 쓰지 않는다» — 착지 기록의 정본은 `docs/report/` 의 회차별 파일이다**
> (2026-09-18 이관 · 티켓 `wie-remove-state-md-completed-insertion-point` · 선행 =
> `wie-state-md-insertion-point-conflicts-every-open-pr`(PR #199)가 백필로 **기각 사유를 없앴다**).
>
> ```sh
> grep -H '^## \[' docs/report/*.md | sort -r     # ← 「무엇이 착지했나」의 정본(연번 내림차순)
> ```
>
> ★**왜 «고정 안내»인가 — §진행중 과 «같은 병»이다.** 회차마다 이 절 맨 위에 자기 항목을 끼워 넣으면
> 형제 회차가 전부 ★**«같은 삽입 지점»** 을 다투므로 **착지 1회가 열린 PR 전건의 충돌로 증폭**된다.
> ★**«맨 아래 추가»로는 안 고쳐진다** — 위 §진행중 절의 실측 그대로 **지점이 «하나»인 것**이 문제다.
> ⇒ 지점을 **없앴다.** `docs/report/NNNN--…` 는 ★**회차마다 «새 파일»** 이라 공유 지점이 아예 없다.
>
> ★**언제 가능해졌나**: 「이 절에만 있는 내용이 소실된다」가 **유일한 기각 사유**였고
> (`AGENTS.md` §Landing paperwork 의 SUPERSEDED 배너가 그 결정의 측정 기록이다),
> 2026-09-18 재측에서 ★**149 항목 «전건»이 `docs/report/` 사본을 갖는다**(사본 없음 **0**).
> ※티켓 id 가 없는 옛 3줄도 각각 `0092`·`0094`·`0159` 가 **원문을 그대로 인용**해 두었다.
>
> ★**대가(숨기지 않는다)**: ⑴★**한 홉 멀어진다** — 이 파일만 열면 보이던 «최근 착지 요약»이 이제
> `docs/report/` 를 읽어야 보인다(위 한 줄이 그 인덱스다). 오프라인에서도 읽히지만 **한 번에 한 파일**이다.
> ⑵**정렬이 «날짜»가 아니라 «연번»** 이 된다 — 대체로 일치하지만 **보장이 아니다**(그래서 `-H` 로 경로를
> 붙여 연번으로 정렬한다 · `AGENTS.md` 가 그 이유를 적는다). ⑶★**이 회차 자신이 열린 PR 전건을
> 한 번 더 무효화했다** — 그 비용은 «지금 한 번»이고, **안 하면 «착지마다 계속»** 이다.
>
> ★**이 절을 읽는 기계는 0 이다**(2026-09-18 재측 · 추적 코드·설정 전수): `## 완료` 파서 **0** ·
> `STATE.md` 언급 **1**(`wie_midp/tests/create_image_missing_name_message.rs` 의 doc 주석 —
> 그 줄번호 인용을 이 회차가 `docs/report/0047--…` 로 **함께 고쳤다**).
>
> ★**되돌리는 법**: 이 인용 블록을 지우고 `docs/report/` 항목을 다시 손으로 옮겨 적으면 된다(코드·검사기 0).
> ★**되돌릴 때 알아야 할 것**: 되돌리는 순간 위 ⑶의 비용이 **착지마다** 다시 붙는다.

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
  ★★**[반증됨 2026-09-16 — 아래 「[P2 재측]」 불릿을 읽어라. 이 줄의 두 축이 «둘 다» 과대평가였다]**
  ⒝이식은 **772줄이 아니라 «21줄»**(크레이트 개명 비용 **0** · 진입 API 바이트 동일) ·
  ⒜코퍼스는 **「다른 머신」이 아니라 «이 머신의 git-ignored `game_lab/`»** 으로 풀린다(구조적 불가가 아니다).
  ★**이 줄은 2026-09-03 P2 회차의 기록이라 «사료로» 남긴다 — 그러나 비용 산정에 인용하지 마라.**
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
- **P3**(L·med·★**선행 = P2 아님**. P2 가 답을 못 내므로 P1 결정 뒤로 붙인다) — **여러 회차로 쪼갠다. 첫 조각(개명)은 끝났다.**
  - ★**[첫 조각 완료 2026-09-11 · `wie-p3-rename-wie-web-to-featurephone`]** `wie_web` → `wie_featurephone` **개명**
    (upstream 이 `wie-web` 를 자기 브라우저 앱에 쓴다 — lib 타깃 `wie_web` 충돌). 순수 `git mv` 커밋과 참조 커밋 분리
    (rename detection 보존). ★**산출물 쌍 `wie_web.js`/`wie_web_bg.wasm` 이름은 «일부러» 안 바꿨다** —
    그것은 upstream 충돌이 아니라 **otterpebble 소비자 계약**이다(`featurephone-engine-contract.json` `files`·
    `glueFetchesWasmByName` · 리시버가 릴리스 자산을 **이름으로** curl). `build-wasm.sh` 의 `--out-name wie_web` 이
    그 분리를 만든다. 산출물 개명은 wie+otterpebble **교차 repo 조율 회차**다(선행: otterpebble 리시버·셸 합의).
  - ★**[둘째 조각 완료 2026-09-13 · `wie-lgt-entrypoint-jar-name-contract-align-with-upstream`]**
    **엔트리포인트 규약 정합** — ★**종전 문안 「upstream `LgtEmulator` 는 `application.jar` 를 찾는다」는
    부정확했다**: upstream 은 어떤 이름도 하드코딩하지 «않고» **내용으로 찾는다**
    (`*.jar` 중 zip 안에 `binary.mod` 가 있는 것 · `upstream/main:wie-lgt/src/emulator.rs:50-53` ·
    2026-08-23 PR #1368 이 `format!("{aid}.jar")` 를 그 형태로 바꿨다). upstream 테스트가 픽스처의
    `00000000.jar` 를 `application.jar` 로 개명해 넘기는 것은 **그 탐색을 증명하려는 장치**다.
    ⇒ 우리도 그 3줄을 **그대로** 채택했다(`wie_lgt/src/emulator.rs`) — 이름 규약이 «둘»로 남지 않고
    기존 `00000000.jar` 픽스처도 그대로 산다. 정본 = `docs/report/0104--….md`.
  - ★★★**[셋째 조각 완료 2026-09-16 · `wie-p3-remaining-slices-split-and-verifiability-plan`] 남은 셋(⑵⑶⑷)의
    «회차 분할 + 검증식»이 정해졌다 — 정본 = `docs/upstream-realign-p3-slices.md`.**
    ★**종전 문안 「upstream 을 base 로 ③ 오버레이 재적용 + `compile_model.rs` 122줄 이식」은 «둘 다 틀렸다»**:
    ⑴★**③ 은 재적용할 것이 없다** — 머지 예행에서 `web/` 32 · `functions/` 24 · `migrations/` 8 · `scripts/` 17 ·
    `wie_featurephone/` 7 **전건 온존**. 비용은 ③이 아니라 **②(51건) + ①(12) + 설정(11)** 에 있다.
    ⑵★**`compile_model.rs` 는 «이식»이 아니다** — 머지가 `AU wie-lgt/src/compile_model.rs` 로 **혼자 데려온다**
    (upstream 이 2026-08-30 `49db5171` 로 크레이트를 `wie_x`→`wie-x` 개명했는데 **git rename 탐지가 따라간다** ·
    `R` **146건** 실측) ⇒ ★**「우리 크레이트 하이픈 개명」 조각도 «만들지 마라».**
  - ★★★**[go/no-go 신호 1건 — 코퍼스 없이 나왔다] `keydraw_lgt` 가 upstream base 에서 «FAIL» 이다.**
    ours **PASS**(paints 55 · rc 0) ↔ upstream `44fbf265` ★**FAIL**(paints 0) ·
    `Undefined instruction` at `net/wie/CletWrapperCard.paint` · **2/2 결정적**.
    ★**verdict §3-5·§8-2 의 「신규 FAIL 0」은 «아무것도 그리지 않는» helloworld 2건으로 얻은 값**이고,
    그 뒤(2026-09-06) 커밋된 **그리는 픽스처**(`keydraw_*`)는 upstream 에 대고 돌려진 적이 없었다.
    ⇒ ★★**「upstream LGT 는 우리의 진부분집합」(§3-1)은 «구현 개수» 축에서 참이고 «동작» 축에서 거짓이다.**
    ★**base swap 전에 이것부터 규명하라** — 조각 **A** 가 조각 **D** 의 게이트다.
  - ★★★**[조각 A 돌았다 2026-09-16 · `wie-p3-slice-a-keydraw-lgt-breaks-on-upstream-base` · 정본 `docs/report/0115--….md`]
    원인이 «이름»으로 나왔다 — 「LGT 가 왜인지 깨진다」가 «27줄 배선»으로 좁혀졌다.**
    ★**원인**: upstream 이 graphics SVC **27개**를 LGT 전용 구현(`wie-lgt/src/runtime/wipi_c/graphics.rs` · **1,095줄**)으로
    보내고 그 구현이 게스트에게 **다른 레코드 ABI** 를 준다 — `LgtFramebuffer` **16B**(★`buf` 필드 **없음**) ↔
    공용 `WIPICFramebuffer` **20B**(픽셀 포인터 **+16**). 게스트 SDK(`dlunch/wipi`)가 ★**스스로 패닉**한다
    (게스트 printk: `panicked at wipi/src/framebuffer.rs : 149 : 18 :
    null reference produced`) → 주소 0 분기 → `Undefined instruction`(PC=0x0).
    ⇒ ★**`CletWrapperCard.paint` 스택은 «증상»이지 원인이 아니다.**
    ★★**[정정 2026-09-16 게이트②] 그 패닉 자리는 «핸들 자신»의 역참조(`:149` `read_fb` · 파일 전체에서 `&*` 는
    그 한 자리뿐)이고 `fb.buf`(+16) 읽기가 «아니다»** — +16 을 읽는 `:153` `buffer_ptr` 는 **raw 포인터를 돌려주므로**
    이 문구를 낼 수 없다. ⇒ ★**「+16 을 읽어 끝 너머에서 0 을 얻는다」는 «추론»이고 측정되지 않았다** —
    측정된 것은 ⒜레이아웃 상이 ⒝`149:18` 널 참조 패닉 ⒞27줄 치환으로 PASS, 셋뿐이고 D 의 결정은 그 셋으로 선다.
    ★**「레코드만 20B 로 맞춘다」는 선택지가 아니다**(202 단일 치환 FAIL 불변 = 계열 전체 · 검수자가 진짜 20B
    백킹을 줘도 같은 자리 FAIL — ※그 실험은 결정적이지 않아 **미지지**이지 반증이 아니다).
    ★**도입 커밋 = `9a88423b`(2026-08-23) `Fix LGT graphics and runtime compatibility (#1368)`** —
    ★**⑸ 엔트리포인트를 바꾼 그 PR 과 «같다».**
    ★**무는 것**: 그 27줄을 `wie_wipi_c::api::graphics::*` 로 치환하면 ★**PASS · paints 55**(2/2). 원본 FAIL · paints 0(3/3).
    ★**배제 4종**(전부 실행): 포인터 등록(양쪽 `0x25619`) · `init_process_state`/`set_use_annunciator` 2줄 ·
    **202 단일 치환** · 공용 구현 자체(diff 0). ★**범위 = LGT «그리기» 한정**(`keydraw_ktf`·`helloworld_lgt` upstream PASS).
    ★★**D 로 넘어간 결정 항목**: 그 27줄을 ⒜upstream LGT 전용 유지(**SDK 기반 게스트가 깨진다**) ↔
    ⒝공용 복귀(**upstream LGT 리버스 1,095줄을 버린다**) 중 어디로 둘 것인가.
    ★**코퍼스 없이 안전하게 못 정한다** — 단 축이 «좁혀졌다»: `docs/lgt_abi.md:930` 이 실제 clet 타이틀(놈ZERO)에서
    「`GetScreenFrameBuffer` 가 준 포인터에 **직접** 픽셀을 쓴다」를 관측했다(= 우리 픽스처와 **같은 모양**) ⇒
    ★**위험이 «픽스처 편향»에 한정될 가능성은 낮아졌다.** ★**그래도 확정은 아니다**(실제 clet 이 upstream 의
    간접 체인을 걸을 수 있다) ⇒ ★**코퍼스가 생기면 «LGT 52건»을 먼저 돌려라.**
    ★★**[정정 2026-09-16 게이트②] 그 «한 겹 아래»의 주소가 틀렸다** — 화면 FB 는 ★**`ptr_image: 0`**
    (`graphics.rs:374-381`)이고, 오프스크린의 `ptr_image` 가 가리키는 것도 `WIPICFramebuffer`(`create_backing`)이지
    `LgtImage`(8B · 이미지 디코드 전용)가 아니다. ★**실제 간접은 «두 겹»**: `핸들 → LgtFramebuffer.ptr_graphics →
    LgtGraphicsView.ptr_backing → WIPICFramebuffer.buf → 픽셀`. ⇒ ★**코퍼스가 생기면 물을 질문은
    「실제 clet 이 `GetScreenFrameBuffer` 반환값을 `WIPICFramebuffer` 로 «직접» 읽나, 아니면
    `ptr_graphics → view.ptr_backing` 을 걷나」**이다(「게임이 `ptr_image` 를 보나」는 **틀린 질문**이다).
    ★**부수 1건(원인 아님)**: 공용 `FrameBuffer::new` 의 `bpl` 이 upstream 에서 `width*bpp` → `buffer_size()` 로 바뀌었다.
    KTF 가 그 판본으로 PASS 하므로 이번 원인은 아니고 ★**조각 B 의 분류 축**이다.
  - ★★★**[조각 D 시도 → «멈췄다» 2026-09-16 · `wie-p3-slice-d-merge-upstream-main-as-base` · 정본 `docs/report/0114--….md`]
    머지 «0». ★막은 것은 «크기»가 아니라 «결정»이다.**
    ★**⑴ 중심 결정이 양쪽 다 검증 불가**(graphics **27줄 배선**): ⒜upstream LGT 유지 ⇒ headline benefit
    (LGT 그래픽 0→1,095줄)을 얻으나 ★`keydraw_lgt` **FAIL** · ⒝공용 복귀 ⇒ PASS 이나 ★**그 1,095줄을 버려
    benefit 을 스스로 취소**한다. ★**코퍼스 없이 어느 쪽도 «옳다»를 증명 못 한다.**
    ★★**[정정 2026-09-21 · 실측] 바로 위 「⒝ 는 그 1,095줄을 «버려» benefit 을 스스로 취소한다」는 «과장이다» —
    ⒝ 에서 그 파일은 트리에 «남는다».** 27줄은 `match` 팔이고, 그 밖에 **진입점 3개가 계속 살아 있다**
    (`wipi_c.rs:199-200` 의 `init_process_state`·`set_use_annunciator` · `init.rs:32` 의 `set_display_property`)
    ⇒ 모듈이 통째로 고아가 되지 않는다. ★**단 공짜도 아니다**: `graphics.rs` 의 `pub fn` **30개 중 27개**가
    미참조가 되고 `mod runtime` 이 크레이트 루트에서 **비공개**라 그 27개는 `dead_code` 다 ⇒ `-D warnings` 에서
    ★**`#![allow(dead_code)]` 1줄이거나 그 27개 삭제**다(최소재현으로 확인 — 사설 모듈 사슬의 미참조 `pub fn` 은 실제로 에러가 된다).
    ⇒ ★**정확한 문장은 「버린다」가 아니라 「«주차»한다」이고, ⒜ 로 되돌리는 값은 그 27줄이다.** 근거 수 = `docs/report/0206--….md`.
    ★★**새 실측 — ⒜ 를 고르면 그 테스트를 «다시 만들 수도 없다»**: 게스트 SDK 에 `lgt` feature 와
    `wipic-sys/src/lgt/graphics.rs` 가 **있는데** ★**`wipi/src/framebuffer.rs` 의 `lgt` 분기가 «0건»**
    (`ledger-grep -c -i lgt` → 0) — 고수준 `Framebuffer` 가 `width/height/bpl/bpp/buf` 를 **feature 무관하게**
    공용 배치로 직접 읽는다 ⇒ `--features lgt` 재빌드로도 `LgtFramebuffer` 와 안 맞는다.
    ★**조각 A 의 미판정 1건은 «닫혔다»**: upstream `get_framebuffer_pointer` 가 픽셀 포인터를 **실제로 준다**
    ⇒ upstream ABI 는 **자기완결적**. ★**그래도 결정은 안 풀린다**(우리 SDK 게스트가 그 접근자를 **안 부른다**).
    ★**⑵ 선행 둘이 done 인데 «미착지»**(A **#160** · C **#159**) — 오늘 `main`(`28fb4364`) 미해결 **74** 중
    ★**8건이 정확히 #159 가 지우는 파일**이다 ⇒ 지금 머지하면 조각 C 의 결정을 **다시 내리고** 그 PR 과 충돌한다.
    ★**이 항은 «중복·충돌» 블로커이지 «정확성»이 아니다** — ⑴과 계급이 다르다.
    ★**규모(참고)**: `UU` 35파일 **헝크 75** + `UD`/`AU`/`AA` **39파일** = **114 결정**. ★크기는 사유로 쓰지 않았다.
    ★★**재개 조건**: ⑴**292 코퍼스(LGT 52건)** ⑵**운영자·총괄이 ⒜/⒝ 를 명시로 고른다**(⒜면 `keydraw_lgt` 를
    **기대 실패로 재분류**, ⒝면 benefit 한 줄 **철회**) ⑶**upstream SDK 가 `framebuffer.rs` 에 LGT 분기를 넣는다**.
    ★**순서는 셋 중 무엇이든 #159 착지 «뒤»다.**
    ★★**[순서 선행 충족 2026-09-16 · 재확인 2026-09-21] 그 「#159 착지 뒤」는 «열렸다»** — #159 **MERGED
    `2026-09-16T01:20:39Z`**(머지 `3744f84e`) · 형제 #160 도 `02:04:45Z`. ⇒ ★**남은 재개 조건은 ⑴⑵⑶ 자신뿐이고,
    그중 ⑴코퍼스·⑶upstream SDK 는 이 레인 밖이다 — 이 레인이 움직일 수 있는 것은 ⑵ 하나다.**
    ★**그리고 ⑵ 는 «측정»이 아니라 «결정»이다** — 코퍼스 없이 어느 쪽도 «옳다»를 증명 못 한다는 위 판정은 **유효하다**.
    ⇒ 그래서 2026-09-21 회차가 **정확성이 아니라 «가역성»** 으로 ⒜/⒝ 를 재고 권고를 냈다:
    정본 `docs/report/0206--2026-09-21--wie-p3-slice-d-resume-gate-is-a-decision….md` ·
    ★**권고 = ⒝**(되돌림 27줄 대칭인데 **게이트 비용이 일방적**이다 · 그 근거 수는 그 문서에 있다).
    ★**권고이지 집행이 아니다 — ⒜/⒝ 선택은 여전히 운영자·총괄 몫이다.**
  - ★**부수 발견 — ★★[해소됨 · 총괄 판단 대기 «아니다» · 그 문구를 되살리지 마라]**: 연번 **`0113` 을 열린 PR 둘이
    함께 claim** 했다(`--next-serial` → `열린 PR claim [0113(#159) 0113(#160)]`). 당시 검사기는 `OK` 였으나
    **둘 다 착지하면 `main` 이 red** 였고, `AGENTS.md` 규율 「착지 안 한 쪽을 옮겨라」는 ★**둘 다 미착지**라 바로 적용되지 않았다.
    그래서 이 회차는 남의 PR 을 고치지 않고 「옮기는 쪽은 **0115** 를 잡아야 한다(0114 는 이 회차가 썼다)」로 판단을 올렸다.
    ★★**처분 — «예측이 그대로 맞았다»**(이 줄을 지우지 않고 남기는 이유가 이것이다): **#159**(조각 C)가 `0113` 을 갖고
    **먼저 착지**(`2026-09-16T01:20:39Z`)하자 **#160**(조각 A)의 `-fix` 회차가 커밋 `39ffb2ef` 로 자기 번호를
    ★**0113 → 0115** 로 옮겼다(착지 `2026-09-16T02:04:45Z`). ⇒ `origin/main` 실재:
    `docs/report/0113--2026-09-16--wie-p3-slice-c-drop-hunks-already-upstream.md` ·
    `0114--2026-09-16--wie-p3-slice-d-merge-upstream-main-as-base.md` ·
    `0115--2026-09-16--wie-p3-slice-a-keydraw-lgt-breaks-on-upstream-base.md` · 처분 기록
    `docs/worklog/2026-09-17-serial-0113-collision-already-resolved.json`.
    ★**red 는 나지 않았다 — «무증상»으로 닫혔다**(`check-docs-report-serial` rc=0 · 중복 연번 0 · 재확인 2026-09-21).
  - ★★**[P2 재측] 두 블로커는 «비대칭»이고 둘 다 verdict 가 적은 것보다 작다.**
    ⒝러너 = ★**21줄**(772/1,147 아님 — upstream 격리 워크트리에서 `cargo check` rc=0 · 빌드 · 5픽스처 실행까지 했다).
    크레이트 개명은 **비용 0**(cargo 가 `wie-ktf` 를 `wie_ktf` 로 노출하므로 `use` 가 안 바뀐다) ·
    진입 API(`from_archive`/`from_jar`/`loadable_*`)는 ★**바이트 동일** · ★**12개 오류**
    (`E0407`×5 · `E0046`×4 · `E0050`×3) **전부** 호스트 어댑터(`Screen::resize` · `AudioSink` 5→1 ·
    `DatabaseRepository` `&System` 제거+`usage` · `Platform::font`) 델타.
    ★**「15」로 되돌리지 마라** — 그 수는 `png` 의존을 **선언하기 전**에 잰 값이고 늘어난 3건은 전부
    `E0433 cannot find crate png` 다. ★**`E0433` 은 어댑터 델타가 «아니라» 매니페스트 의존**이고
    `png` 는 `Cargo.toml` **4줄에 이미 세어져 있다** ⇒ ★**15 로 쓰면 3건을 «두 번» 세어 21줄 산정을 부풀린다.**
    ⒜코퍼스 = ★**「이 머신에서 구조적 불가」가 아니다** — `.gitignore:23` 이 `/game_lab/` 를 무시하고
    `AGENTS.md` 가 그 배치를 스스로 「local only, and structurally so」로 설계라고 적는다.
    못 재는 이유는 구조가 아니라 ★**그냥 코퍼스가 여기 없다**는 것이다(`find ~ -maxdepth 4 -name game_lab` **0건** 재확인)
    ⇒ ★**human-step 후보**(`game_lab/working/{ktf,lgt,skt}` 292타이틀 배치 · 카드 발권은 총괄 몫).
  - **조각 A~E**(상세·검증식은 정본 문서. ★**표시는 위 「조각 A 돌았다」·「조각 D 시도 → 멈췄다」 항목과 정합시킨
    것이다** — 종전에는 B·C 에만 취소선이 그어져 ★**이미 돌아간 A 와 이미 시도된 D 가 «미착수»로 읽혔다**):
    ★~~**A** LGT keydraw 회귀 규명(M·low·선행없음 · ★D 의 게이트)~~ **돌았다(2026-09-16
    `wie-p3-slice-a-keydraw-lgt-breaks-on-upstream-base`)** — 정본 `docs/report/0115--….md` ·
    원인 = upstream LGT 전용 graphics **27줄 배선**(상세는 위 「조각 A 돌았다」 항목) →
    ★~~**B** ② 51건 hunk 분류(M·low · 제품코드 0)~~ **끝났다(2026-09-16 `wie-p3-slice-b-classify-51-engine-overlays` · PR #158)** —
    정본 `docs/upstream-realign-p3-slice-b-triage.md` · **폐기 21 · 재적용 24 · 발신 6 = 51 · 미분류 0** ·
    ★「51」의 구성은 **② 50 + ③f 1**(아래 §3 표에 `③f` 칸이 없어 접혔다) →
    ★~~**C** 이미 upstream 에 있는 것 삭제(M·med · 선행 B)~~ **부분 착지(2026-09-16 `wie-p3-slice-c-drop-hunks-already-upstream`)** —
    정본 `docs/upstream-realign-p3-slice-c-deletability.md` · ★**폐기 21행이 «세 계급»으로 갈렸다**:
    **A 7행 지웠다**(머지 예행 **74 → 67**) · ★**B 5행 «구조적 불가»**(되돌리면 `cargo check` 오류 10건 ·
    beta clippy red · `draw_j2me` FAIL) · ★**C 9행 보류**(검증이 공허 — 운영자 판정).
    ⇒ ★**그룹 B·C 는 조각 D 안에서 처분된다 — 조각 C 를 다시 발권하지 마라.**
    ★**C 의 착지 «전» 기대값 「`53 ≤ N < 74`」**(폐기 21 중 `UU` 5행은 파일이 남고 델타만 사라지므로 74→53 이 «아니다»)
    **는 «실측이 대체했다» — 실제 74 → 67** · ★**우리 base 위라 5게이트가 전부 산다** →
    ★~~**D** base swap 머지(L·★**high** · 선행 A·C)~~ **시도했고 «멈췄다»(2026-09-16 `wie-p3-slice-d-merge-upstream-main-as-base`)** —
    머지 **0** · ★막은 것은 «크기»가 아니라 «결정»이다(상세와 **재개 조건 3종**은 위 「조각 D 시도 → 멈췄다」 항목) →
    **E** 웹 계약·아티팩트(M·med · 선행 D) — ★**미착수**(선행 D 가 멈춰 있다).
    ★★**D 의 ⒟ 는 «충돌 0» 이 아니라 «빌드 게이트»다 — 근거는 「★«미해결 0» 이 «컴파일된다»를 뜻하지 않는다」이고,
    ★그 근거는 «참»이다.** 아래 둘은 **충돌 74 «안»에 `UD` 로 있고**, 그 항목을 「한쪽 고르고 끝」으로 풀면 빌드가 깨진다:
    ⑴★**`UD wie_cli/Cargo.toml`**(1건) — upstream 이 네이티브 호스트를 루트 패키지로 옮겼다
    (`R` **4건** `src/{database,filesystem,window}.rs` · `src/main.rs`→`src/lib.rs` + `D` **1건** `src/audio_sink.rs`).
    ★★**`wie_validate.rs` 는 머지가 «손대지 않는다»**(상태 히트 **0건** · 머지 트리 실재 ·
    `git diff origin/main -- wie_cli/src/bin/wie_validate.rs` **0줄**) ⇒ ★**위험의 이름은 «소스 소실»이 아니라
    «매니페스트·bin 타깃 미화해 시 러너 빌드 불가»다.**
    ⑵★**`UD wie_backend/src/canvas.rs`** — 그 `UD` 를 풀 때 `include_bytes!("../../fonts/neodgm.ttf")` 를
    `assets/` 로 고쳐야 한다(upstream 은 폰트를 `Platform::font()` 로 옮겼다 · `assets/neodgm.ttf` 실재).
    ★★**[정정 2026-09-16 `-fix`] 종전 문안은 「이 둘은 충돌 74건 «밖»이다」였고 «거짓»이었다 — 되살리지 마라.**
    근인 = `git status --porcelain -- wie_cli` 라는 **pathspec 제한 조회** ⇒ ★**경로를 제한하면 git 이 rename 짝을
    깨고 `R` 을 `D` 로 보여 준다.** ★**머지 상태는 «경로로 좁히지 말고» 전체를 받아 `grep` 하라.**
    ★**그리고 `binary_patches` 축은 «파열이 아니다»** — upstream 도 그 코드를 갖고 자산과 **함께** 이사시켜
    머지 트리의 `include_str!("../../data/…")` 가 **정확히 맞는다**. 종전 ⑵의 그 절반은 **일이 아니다.**
  - ★**오늘의 수**(2026-09-16 · `d30cb903` ↔ `44fbf265`): behind **1,140**(verdict 1,067) · ahead **535**(192) ·
    ② 엔진 **66f +2,629/−96**(45f +1,134) · 머지 미해결 **74**(정적 충돌면 43 → 70).
    ★**verdict 의 2026-08-27 수를 상수로 인용하지 마라 — 조회 시각과 함께 읽어라.**
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
