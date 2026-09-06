## [2026-09-06] WIPI 리소스 픽스처 — «아무 시험도 지나지 않던» 4자리를 태웠다 (wie-system-class-loader-spi-resource-fixture)
- **무엇을**: `scripts/make-wipi-keydraw-fixture.sh` 에 리소스 1건(`res.bin` · `WIE-RES-1` · 9바이트) + 게스트 부팅 시 읽기 → `res:9:602` 출력. `test_data/keydraw_{ktf,lgt}.zip` 재생성. `wie_{ktf,lgt}/tests/test_resource_reach.rs` 신설.
- **왜**: 운영자 채택 제안 `2026-09-05-system-class-loader-preemptive-migration#p0`.
- **★★⑴ 제안 ⑵ 가 「먼저 확인하라」고 한 것을 «먼저» 봤다**: `wipi-archiver/src/lib.rs:96-99` 가 `resource_path` 를 재귀 복사한다 ⇒ ★**동봉 경로는 이미 열려 있었고 아카이버 축은 «추가되지 않았다»**. 스크립트가 그 디렉터리를 `mkdir -p` 하고 ★**아무것도 넣지 않은 것**이 커버리지 0 의 기전이다.
- **★★⑵ «지금 0» 을 먼저 실행으로 냈다**: 4자리에 `panic!()` 을 심고 `cargo test --all` → ★**rc=0 · 150 passed / 0 failed** · `wie_validate` **5픽스처 전건 PASS** ⇒ 그 네 줄은 «실행된 적이 없다».
- **★★⑶ 자리별 드릴로 «4자리 각각»을 보였다**(쌍이 아니라 낱개): ①`ktf get_resource_size` ②`ktf read_resource` ③`lgt get_resource_size` ④`lgt read_resource` — ★**전건 FAILED rc=101** · 무개악 기준선 **rc=0**.
- **★⑷ 한 줄이 «두 홉»을 각각 증명한다**: `size` 는 `MC_knlGetResourceID` ⇒ 호스트 `get_resource_size` · 합(602)은 `MC_knlGetResource` 가 **실제로 넘긴 바이트**로만 계산된다 ⇒ 호스트 `read_resource`. ★`res:err` 를 따로 단언한다 — 「호출이 실패했다」와 「호출이 없었다」는 다른 결함이다.
- **★⑸ 회귀 0**: 기존 `test_key_reach` 2건이 재생성 zip 으로도 통과 · 브라우저 왕복 ★**35/35 rc=0**(Scenario E 픽셀 280·336·424 불변) · `wie_validate` 3픽스처 PASS. ★게스트는 리소스를 **그리지 않는다** — 키 픽셀 단언을 지키려는 의도적 선택이다.
- **★⑹ 거짓이 된 서술을 정정했다**(범위 판단을 회신에 적었다): 4자리 `.rs` 주석의 「NOT covered by any fixture」 + `verdict` §8-4⑶-b ⒡ 표 4행·결론(「커버되는 자리는 1곳」 → **5곳** · 남은 미커버는 **6번 하나**). ★`image.rs` 의 같은 주석은 **무접촉**(이 픽스처가 그 자리를 덮지 않는다 — 여전히 참).
- **사용자 영향**: 없음(시험). 대신 리소스를 읽는 실제 게임에서만 터지던 회귀가 **커밋 전에** 잡힌다.
- **★남는 구멍**: ⒜**실패 갈래 미커버** — `kernel.rs:198-205`(없는 리소스 → -12) · `:232-234`(버퍼 초과 → -1) 분기가 실재하는데 시험이 없다(제안 등재) ⒝**브라우저 축 없음** — 증거가 전부 헤드리스 stdout 이다(제안 등재) ⒞`wie_wipi_c/…/database.rs:620·624` 의 같은 호출도 미커버(세기만 했다) ⒟커버 = «실행된다»이지 «옳다»가 아니다.

