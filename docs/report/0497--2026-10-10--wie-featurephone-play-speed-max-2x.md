## [2026-10-10] 게임 배속 상한 3.0 → 2.0 (wie-featurephone-play-speed-max-2x)

**무엇을**: `SpeedClock::MAX_SPEED` 3.0 → 2.0. `set_speed` 범위가 `[1.0, 2.0]` 이 된다(NaN·무한 → 1.0, 1 미만 → 1.0 그대로).
문서 주석(`wie_featurephone/src/lib.rs` · `wie_validate --speed`) · 계약 `setSpeedNote` · 업데이트 소식 문구를 같이 고쳤다.
계약 `methods` 는 그대로라 export 표면은 바뀌지 않는다.

**왜**: 운영자 지시(2026-10-10) 「1~3배가 아닌 1~2배로」. 선행 #527 이 `[1.0, 3.0]` 으로 착지했다.

**사용자 영향**: 2배를 넘는 요청은 2배로 적용된다(반환값이 실제 적용값). 셸 슬라이더 상한은 otterpebble 쪽 티켓이 진다.

**업데이트 소식**: 새 항목을 내지 않고 `2026-10-09-play-speed.json` 의 `summary_ko` 를 정정했다 — #527 과 이 변경이 같은 릴리스 창 안이라
「3배까지 된다」 항목이 남으면 사실이 아닌 소식이 피드에 남는다(소식 계약 §2: 한 항목 = 한 변경, 원천은 하나).

**검증**: 4게이트(fmt · clippy stable/wasm · beta clippy · `cargo test --all` rc=0) · `speed_clock` 단위 시험 5종
(2.7→2.0 · 3.0→2.0 · 2.0 그대로 · 3x 요청 sleep(100) 이 2x 와 같이 49~52ms) · 러너 고정물 6종 PASS · `player-data.mjs` OK.
`cargo test` 첫 2회는 incremental 캐시 파일이 빌드 도중 사라지는 rustc ICE(공유 target 경합)로 죽었고 `CARGO_INCREMENTAL=0` 으로 통과했다 — 코드 무관.
