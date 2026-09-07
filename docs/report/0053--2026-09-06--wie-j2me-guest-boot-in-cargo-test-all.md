## [2026-09-06] J2ME 게스트를 `cargo test --all` 안에서 부팅한다 — 그물이 브라우저 잡 «하나»에 매달려 있었다 (wie-j2me-guest-boot-in-cargo-test-all)
- **무엇을**: `wie_j2me/tests/test_boot.rs` **1건** 신설 + `test_utils` 하니스 확장(페인트 계수기 · redraw 플래그) + 커밋 픽스처 `test_data/draw_j2me.zip`(**1,020바이트**). ★**브라우저 잡 무접촉**(`.github/`·`scripts/`·`web/` diff **0파일**) · 게스트 시나리오 추가 **0**.
- **왜**: 운영자 채택 제안 `2026-09-06-createimage-fixture#p1`.
- **★⑴ 전제를 먼저 쟀다**(모집단 명시): base `34bca716` 에서 `git ls-files '*/tests/*.rs'` = **10파일** · 그중 `wie_j2me` 를 부르는 것 ★**0건** · `wie_j2me/tests/` **부재** ⇒ 전제 성립.
- **★★⑵ 막힌 것은 «하니스»가 아니라 «픽스처»였다**: `test_data/draw_j2me.jar` 는 ★**일부러 커밋하지 않는다**(`.gitignore:24` 가 `*.jar` 무시 · `audit-no-leak.sh:128` 이 **추적된 `*.jar`** 거부 — Constraint 9. 생성기 머리주석의 명문). ⇒ ★**zip 이 jar 를 담는 «기존 관례»를 그대로 썼다** — 이미 커밋된 `keydraw_ktf.zip` 안에 `00000000.jar` 가 들어 있는 그 형태다. ★**blocklist 를 느슨하게 하지 않았고 확장자를 바꿔 규칙을 피하지도 않았다**(`npm run audit` **PASSED · 「no game binaries tracked in git」**).
- **★★⑶ 하니스의 핵심은 «redraw 응답»이고, 실측이 가르쳐 줬다**: 페인트는 **요청/응답** 루프다 — 코어가 `request_redraw()` 를 부르고 ★**호스트가 `Event::Redraw` 를 돌려줘야** `Screen::paint` 가 돈다. 종전 `TestScreen` 은 그 요청을 **기록하지 않아** ★**첫 판이 10,000틱에 paints 0 으로 실패**했다(추측이 아니라 그 실패를 보고 알았다). ★`wie_validate` 가 **정확히 이 루프**를 돈다 ⇒ 모양을 **발명하지 않고 가져왔다**.
- **★★⑷ 단언은 「아무것도 안 던졌다」가 아니라 «프레임이 합성됐다»**: 전자는 ★**페인트 전에 죽는 게스트도 통과**시키고 그것이 2026-09-04 형상이다(네 게이트 green · `NoClassDefFoundError` · paints 0). 픽셀은 여전히 브라우저 몫이다.
- **★⑸ 양방향 2종 — 둘 다 red**: **M-A** 게스트 메인 클래스를 못 찾게(=그 사고 형상) → FAILED · **M-B** `image.rs` 를 마이그레이션 «이전» `current_class_loader` 로 → FAILED. 원복 후 green.
- **★⑹ 대가를 수로**: 시험 실행 **0.04s** · 한계 링크+실행 **1.50s** · 워크스페이스 `cargo test --all` **62.1s**(**41줄 / 157 passed / 0 failed** = 종전 40/156 에서 **정확히 +1**, 회귀 0) · 커밋 파일 +1(1,020B) · `Cargo.lock` **+1줄** · 새 의존성 **0**.
  ★**티켓이 인용한 「CI p95 2,400~2,700s」는 이 저장소의 수가 아니다**(그것은 otterpebble self-hosted). `Jun025/wie` 실측(완료 run 496건): **p50 215s · p90 356s · p95 419s · max 958s** ⇒ 이 축의 대가는 **p95 의 0.4% 미만**.
- **사용자 영향**: 없음(CI). 대신 J2ME 부팅 축이 **브라우저 잡 하나에 매달린 상태를 벗어난다**.
- **★남는 구멍**: ⒜**옮기지 않았다** — 브라우저 Scenario C 는 그대로이고 이 축은 «두 번째 그물»이다(대체재 아님: 브라우저는 실제 픽셀, 여기는 합성 여부) ⒝커밋 픽스처는 **재생성 가드 없는 스냅샷**(기존 4개와 같은 성질) ⒞`paints > 0` 은 **빈 프레임도 센다** ⒟M-A 의 red 는 **패닉**이지 `tick()` 의 `Err` 가 아니다 — 「예외가 `Err` 로 전파된다」고 적지 않는다(재지 않았다).
- **★★착지 시 배포가 «있다»**: 착지 diff 에 `Cargo.lock`·`wie_j2me/Cargo.toml` 이 들어가고 그 둘이 `publish-artifact.yml` 의 `on.push.paths` 에 **매치** ⇒ ★**Release 컷 + otterpebble `repository_dispatch` 발화**(문서 전용 착지들과 다르다). 머지 회차는 3-a 예측에 그대로 적고 self-verify 하라.
