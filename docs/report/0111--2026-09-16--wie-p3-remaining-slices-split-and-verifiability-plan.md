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

**충돌 목록에 «없는» 파열 2건**(그래서 조각 D 의 검증식이 4게이트여야 한다):
⑴upstream 이 `wie_cli` 크레이트를 지웠다 — `UD` 는 `Cargo.toml` 1건뿐인데 그 안에 `wie_validate`
1,147줄이 있다. ⑵`data/binary_patches.toml`·`fonts/neodgm.ttf` 가 머지 후 사라지는데
`include_str!`/`include_bytes!` 경로는 그대로 남는다(둘 다 머지 트리에서 **GONE** 실측).

**사용자 영향** — 없음(문서 전용). 다음 회차가 「무엇을 어떤 순서로, 무엇으로 검증하며」 하는지
읽을 곳이 생겼다. 조각 **A**(LGT 회귀 규명)가 **D**(base swap)의 게이트다.

**이 회차가 못 잰 것** — 292 타이틀 코퍼스(부재) · 브라우저 왕복(툴체인) · 네이티브 창 호스트
(`wie_cli`의 `WindowHandle` — 두 그물 어디에도 안 덮인다). 각각 「무엇이 있으면 잴 수 있는가」는
계획 문서의 조각별 칸에 적었다.
