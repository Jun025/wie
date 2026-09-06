## [2026-09-06] `last_frame_content` 를 게이트로 올렸다 — 기대값 선언 자리를 «명령줄»로 골랐다 (wie-validate-last-frame-gate-with-per-fixture-expectation)
- **무엇을**: `wie_cli/src/bin/wie_validate.rs` 에 **`--expect-last-frame`**(기본 **off**) + 순수 함수 `last_frame_gate_fails()` + 단위시험 1건. ★플래그가 있을 때만 `last_frame_content` 가 판정에 개입하고, ★**PASS → FAIL 한 방향뿐**(이미 실패한 런의 더 구체적인 사유를 덮지 않는다).
- **왜**: 운영자 채택 제안 `2026-09-06-validate-last-frame-axis#p0`(+ 흡수 `2026-09-06-lgt-black-screen-name-compare#p0` — 그 제안이 요구한 «축 추가»는 이미 착지했고 남은 것이 게이트화뿐이라 중복 발권을 피했다). 축은 있었으나 **REPORT-ONLY** 라 아무것도 막지 않았다.
- **★★⑴ 전제를 먼저 반증했다**: `git show origin/main:…/wie_validate.rs | grep -n last_frame_content` → **10곳** 실재 · `:30` 이 스스로 「REPORT-ONLY and deliberately not a gate」 ⇒ 여전히 게이트 아님(이미 게이트였다면 이 회차는 거기서 끝났다).
- **★★⑵ 선언 자리를 «골랐고 이유가 실측이다»**: 후보 셋 중 ⒞ 플래그. ★**기대값의 키가 `픽스처`가 아니라 `픽스처 × 모드`** 이고 — `keydraw_lgt` 는 `--inject` 유무로 기대값이 뒤집힌다(`PASS·last TRUE` ↔ `FAIL·last false`) — ★**그 모드 절반이 이미 명령줄에만 있다**. ⒜사이드카·⒝이름 표는 키의 나머지 절반을 다른 곳에 둬 **두 번째 진실원**이 된다(픽스처 개명·모드 추가·처음 보는 파일에서 드리프트).
- **★⑶ 계약 4 충족**: 플래그 없이 **6행 전건 현행 판정 불변**(helloworld_* PASS · keydraw_* `--inject` PASS · keydraw_* 모드 없음 FAIL). 기본값 off 라 구조적으로 그렇다. `cargo test --all` **157 passed**(직전 156 + 신규 1).
- **★★⑷ 개악 대조가 «합성»이 아니다 — 실제로 일어났던 회귀를 되돌렸다**: `is_clet_card` 정규화를 **PR #88 이전** 형태로 → `keydraw_lgt --inject` 플래그 없음 ★**PASS · content true · last_frame_content FALSE**(그때 실제로 새어 나간 형상 그대로) ↔ 같은 형상 + `--expect-last-frame` ★**FAIL**. KTF 대조군은 **PASS**(개악이 LGT 캐리어에만 닿는다).
- **★⑸ 양의 방향도 보였다**: `helloworld_{ktf,lgt} --expect-last-frame` → **FAIL** ⇒ 제안이 「지금 그대로 게이트를 걸면 뒤집힌다」고 실측한 그 형상이 **선언했을 때만** 일어난다.
- **사용자 영향**: 없음(기본 동작 불변). 대신 「검사는 통과하는데 화면은 검다」를 ★**브라우저 없이 약 20초에** 잡을 수 있게 됐다(그 그물은 지금까지 wasm 빌드 + `contract` 잡 3~4분뿐이었다).
- **★남는 구멍**: ⒜★**호출자가 «0»이다** — 선언 자리를 만들었을 뿐 아직 아무도 켜지 않았다. 어느 워크플로도 `wie_validate` 를 부르지 않고, 러너 블록 3종 중 `helloworld_*` 는 「비어야 정상」이라 플래그가 틀린다(제안 등재) ⒝opt-in 은 잊은 호출을 못 막는다(사이드카도 같다) ⒞`keydraw_*` 는 기본 `--timeout 20` 에 바짝 붙어 돈다(`ms 20028`) — 상시 게이트로 올리려면 예산을 먼저 재야 한다 ⒟richness 3축의 같은 사각은 **계약 3 대로 무접촉**(형제 티켓 몫).

