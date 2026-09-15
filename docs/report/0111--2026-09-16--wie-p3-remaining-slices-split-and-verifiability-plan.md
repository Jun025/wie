## [2026-09-16] P3 남은 조각(⑵⑶⑷) 분할과 검증식 계획 (wie-p3-remaining-slices-split-and-verifiability-plan)

**무엇을** — `docs/upstream-realign-p3-slices.md` 신설. P3 의 남은 셋을 **5조각(A~E)** 으로 쪼개고
조각마다 ⒜범위 ⒝size/risk ⒞선행 ⒟**이 머신에서 돌릴 수 있는 명령 + 기대 출력**을 채웠다.
「못 재는 것」은 전부 **「무엇이 있으면 잴 수 있는가」와 짝지어** 적었다. 제품 코드 0줄.

**왜** — verdict 는 P3 의 선행을 P2 로 적고 P2 를 「이 머신에서 측정 불가」로 닫았다. 그대로 두면
P3 는 **검증 없는 대규모 교체**가 된다. 이 회차는 「P3 를 할까」가 아니라 **「P2 없이 무엇을 잴 수 있고
무엇은 끝내 못 재는가」**에 답한다. 계기는 `LANE_IDLE wie`(큐 54시간 무이동) — 할 일이 없어서가
아니라 **다음 칸이 정의되지 않아서** 굶었다.

**측정 (2026-09-16 04:1x~05:0x KST · `origin/main` `d30cb903` · `upstream/main` `44fbf265` · `merge-base` `fa641a8a`)**

- 계보: behind **1,067 → 1,140** · ahead **192 → 535**(verdict 의 수는 2026-08-27 값이다).
- 분류 재측(.rs): ① `wie_lgt` **16f +2,747/−54** · ① `wie_featurephone` 6f +1,011 ·
  ② 엔진 **66f +2,629/−96**(verdict 45f +1,134) · ③f `wie_cli` **4f +1,805**(`wie_validate` 단독 **1,147**) ·
  ③ 스캐폴딩 367f +32,913. `compile_model.rs` = **122줄**(불변).
- ★**머지 예행**(격리 워크트리 · `--abort` 후 제거): rc=1 · 미해결 **74**
  (UU 35 · AU 19 · UD 17 · AA 3) · 자동 A 167 / R 146 / D 104.
  갈래별 ② **51** · ① **12** · ③ 설정 **11**.

**바뀐 판정 3건**

1. ★**`keydraw_lgt` 가 upstream base 에서 FAIL 이다** — ours PASS(paints 55) ↔ upstream
   **FAIL(paints 0)** · `Undefined instruction` at `net/wie/CletWrapperCard.paint` · 2/2 결정적.
   verdict 의 「신규 FAIL 0」은 **아무것도 그리지 않는 helloworld 2건**으로 얻은 값이고,
   그 뒤(2026-09-06) 커밋된 **그리는 픽스처**는 upstream 에 대고 돌려진 적이 없었다.
   ⇒ **코퍼스 없이 no-go 신호 1건을 얻었다.**
2. ★**P2 러너 이식 = 21줄**(verdict 의 「772줄 이식」 아님). upstream 격리 워크트리에서 실제로
   `cargo check` rc=0 · 빌드 · 5픽스처 실행까지 했다. 크레이트 개명은 **비용 0**(cargo 가
   `wie-ktf` 를 `wie_ktf` 로 노출) · 진입 API(`from_archive`/`from_jar`/`loadable_*`)는 **바이트 동일** ·
   15개 오류 **전부** 호스트 어댑터(`Screen`/`AudioSink`/`DatabaseRepository`/`Platform`) 델타.
3. ★**코퍼스 부재는 「이 머신에서 구조적 불가」가 아니다** — `.gitignore:23` 이 `/game_lab/` 를
   무시하고 `AGENTS.md` 가 그 배치를 스스로 설계로 적는다. 못 재는 이유는 구조가 아니라
   **코퍼스가 여기 없다**는 것뿐이다 ⇒ **human-step 후보**(회신에 «후보»로만 적었다).

**조각이 둘 소멸했다** — 만들지 마라: ⑴「우리 크레이트를 하이픈 개명」(git rename 탐지가 이미 한다 ·
`R` 146건 실측) ⑵「`compile_model.rs` 122줄 이식」(머지가 `AU wie-lgt/src/compile_model.rs` 로 데려온다).
그리고 「③ 오버레이 재적용」도 **재적용할 것이 없다** — 머지된 트리에서 `web/` 32 · `functions/` 24 ·
`migrations/` 8 · `scripts/` 17 · `wie_featurephone/` 7 전건 온존.

**★«미해결 0» 은 «컴파일된다»가 아니다 — 그래서 조각 D 의 ⒟ 가 빌드 게이트다**(2건, 둘 다 `UD` 로
**충돌 74 «안»에 있다**): ⑴★**`UD wie_cli/Cargo.toml`** — upstream 이 네이티브 호스트를 루트 패키지로
옮겼다(`R` 4건 `src/{database,filesystem,window}.rs`·`main.rs→lib.rs` + `D` 1건 `audio_sink.rs`).
★**`wie_validate.rs` 는 머지가 손대지 않는다**(상태 히트 0 · `git diff origin/main` 0줄) — 위험은
«소스 소실»이 아니라 **«매니페스트·bin 타깃 미화해 시 러너 빌드 불가»** 다. ⑵★**`UD wie_backend/src/canvas.rs`**
— 그 `UD` 를 풀 때 `include_bytes!("../../fonts/neodgm.ttf")` 를 `assets/` 로 고쳐야 한다(upstream 은 폰트를
`Platform::font()` 로 옮겼다).

> ★★**[정정 2026-09-16 · 게이트② 반려 승계 `-fix`] 위 문단의 초판은 이 둘을 「충돌 목록 «밖»의 파열 2건」으로
> 적었고 ★그 주장은 «거짓»이었다 — 두 항목 다 `UD` 로 74 «안»에 있다.** 근인 = 초판이
> `git status --porcelain -- wie_cli` 라는 **pathspec 제한 조회**로 읽었고, ★**경로를 제한하면 git 이
> rename 짝을 깨고 `R` 을 `D` 로 보여 준다**(같은 트리에서 제한 없이 읽으면 `R` 4 + `UD` 1 + `D` 1).
> ★**그리고 `binary_patches` 축은 «파열이 아니다»** — upstream 도 그 코드를 갖고 자산과 **함께** 이사시켰다
> (머지 트리 `wie-core-arm/src/binary_patches/parser.rs:11` 은 `include_str!("../../data/…")` 이고 그 대상이
> **실재한다**). 초판은 **우리 트리의 include 줄**을 읽고 **머지 트리의 파일 위치**와 맞붙였다 — pathspec
> 오류와 **같은 계급**이다. ★**조각 D 의 빌드 게이트는 그대로 남는다** — 근거만 「목록 밖이라 안 보인다」에서
> **「목록 안에 있어도, 해소 ≠ 컴파일」**로 갈아 끼웠다. 상세 = `docs/upstream-realign-p3-slices.md` §3-2.
> ※같은 회차에서 함께 정정: **이식 델타 오류 수 = 15 → ★12**(초판은 `png` 의존을 선언하기 전에 쟀고,
> 늘어난 3건은 전부 `E0433 cannot find crate png` 였다. `png` 는 `Cargo.toml` 4줄에 이미 포함 ⇒ 두 번 세면
> 안 된다. ★**「21줄」 결론은 불변**) · **「커밋된 픽스처 5건」 → ★「커밋 4건 + 생성 1건」**
> (`draw_j2me.jar` 는 `.gitignore:24` `*.jar` 로 미추적이고 커밋된 생성기가 만든다. 커밋본 `draw_j2me.zip`
> 은 이 러너의 입력이 아니다 — 실측 `FAIL · unrecognized zip archive`).

**사용자 영향** — 없음(문서 전용). 다음 회차가 「무엇을 어떤 순서로, 무엇으로 검증하며」 하는지
읽을 곳이 생겼다. 조각 **A**(LGT 회귀 규명)가 **D**(base swap)의 게이트다.

**이 회차가 못 잰 것** — 292 타이틀 코퍼스(부재) · 브라우저 왕복(툴체인) · 네이티브 창 호스트
(`wie_cli`의 `WindowHandle` — 두 그물 어디에도 안 덮인다). 각각 「무엇이 있으면 잴 수 있는가」는
계획 문서의 조각별 칸에 적었다.
