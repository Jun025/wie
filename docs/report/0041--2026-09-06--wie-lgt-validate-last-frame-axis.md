## [2026-09-06] `wie_validate` 에 «마지막 프레임» 축을 더했다 — any-frame 술어는 «덮어쓰기»를 구조적으로 못 본다 (wie-lgt-validate-last-frame-axis)
- **무엇을**: `wie_cli/src/bin/wie_validate.rs` — `fn has_content` 추출 + `Outcome.last_frame_content`(JSON 동명 키) 신설 + 모듈 헤더에 「두 축」 절 + 단위시험 2건. ★**보고 전용 · 게이트 아님** · `passed` 분기 무접촉.
- **왜**: 운영자 채택 제안 `2026-09-05-lgt-browser-paint-localize#p1`.
- **★⑴ 사각을 한 문장으로**(계약 5): ★**보는 것 = 「도는 동안 «한 번이라도» 2색 이상인 프레임이 있었나」 · 못 보는 것 = 「그 뒤에 덮였나」** — `saw_content` 가 프레임 전체에 대한 **OR** 이고 OR 은 **단조**라 나중 프레임이 값을 되돌릴 수 없다. ★**튜닝으로 못 고치는 «구조»다.**
- **★★⑵ 같은 술어, 다른 범위 — 기계로 만들었다**: `has_content` 를 추출해 `paint()`(ANY-frame OR)와 최종 계산(LAST-frame)이 **문자 그대로 같은 술어**를 쓰게 했다. ★술어까지 다른 두 축을 비교하면 아무것도 증명하지 못한다.
- **★★⑶ 양방향을 «합성»이 아니라 «살아 있는 결함»으로 보였다**: `#p0`(근인 수정)은 **PR #88 로 미착지**라 현 `main` 에 검은 화면이 **그대로 있다** ⇒
  ⒜**현 main**: `keydraw_lgt --inject` → `PASS · content=true · ★last_frame_content=false` / `keydraw_ktf` → `true · true`(대조군)
  ⒝**#88 을 «임시로» 얹으면**: LGT → `★last_frame_content=true` · KTF 불변.
  ★★**`result` 와 `content` 는 두 형상에서 «한 번도» 움직이지 않았다 — 움직인 것은 새 축 하나뿐이다.**
- **★⑷ 제안 ⑴ 의 경고를 «가정하지 않고 쟀다»**: `helloworld_ktf`·`helloworld_lgt` 는 `paints=0 · clean exit` 이라 ★**`last_frame_content=false`** 다 ⇒ ★**게이트를 걸었으면 지금 PASS 인 2픽스처가 FAIL 로 뒤집힌다.** 보고 전용은 «판단»이 아니라 **측정**에 근거한다.
- **사용자 영향**: 없음(검사기 필드). 대신 「검사는 통과하는데 화면은 검다」가 ★**PC 검사기 출력에서도 보인다** — 지금까지 그 그물은 브라우저 왕복뿐이었다.
- **★남는 구멍**: ⒜★**richness 3축도 `fetch_max` = MAX over frames** 라 «똑같은» 사각을 갖는다(마지막-프레임 짝 없음 · **고치지 않았다** · 제안 등재) ⒝게이트 승격은 픽스처별 기대값이 선행(제안 등재) ⒞2색 이상 술어는 거칠어 「단색이 아닌 쓰레기로 덮이면」 두 축 다 true 다.
