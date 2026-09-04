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
- 2026-09-05: **«버린 fork» 커밋 로그 통독** (`wie-abandoned-fork-commit-log-harvest`
  · 채택 제안 `2026-09-04-unported-hardening-two-axes#p0`) — ★**가설은 맞았다**: §10-2 가 인용한 «두 줄»은
  전부가 아니었고 실제로는 **9커밋 · 타이틀 13종 · API 10종**이다(2 → 9). ★**그러나 후속 후보는 0건** —
  핀이 6, `hardening.rs` 가 나머지 3을 이미 덮는다 ⇒ 목록의 값은 «할 일»이 아니라 «지금이 옳다»는 독립 확인이다.
  ★**술어의 한계를 함께 적었다** — 문구 술어(`Trace-specified` 등)는 **8건**만 잡고 `9be0ea3`(박정석_영웅탄생)을
  놓친다 ⇒ 최종 9건은 사람이 12커밋을 통독해 나왔다. 정본 **§10-6**.
  ★범위: 브랜치 2 · 태그 0 · 커밋 1,103 · 고유 87 · PR 30 · 이슈 **비활성화**. `main` 의 75커밋엔 관측 **0건**
  ⇒ 「버린 fork」는 저장소가 아니라 «`wie-ktf-hardening` 브랜치»다. ★코드 변경 0 · 저장소 쓰기 0.

## 완료 (최근)
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
