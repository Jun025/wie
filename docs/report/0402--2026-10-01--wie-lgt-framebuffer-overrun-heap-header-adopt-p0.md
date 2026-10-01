## [2026-10-01] LGT 화면 버퍼 아래 소프트키 줄 — 게임은 «알려준 높이 + 24줄»에 그린다 (wie-lgt-framebuffer-overrun-heap-header-adopt-p0)

**무엇을**: LGT 의 `MC_grpGetScreenFrameBuffer` 가 픽셀 버퍼를 **알려주는 높이보다 소프트키 줄만큼 더** 잡는다(240·320 폭 24줄 · 176 폭 20줄 · 120·128 폭 14줄). 알려주는 높이·폭·bpl 은 그대로다. 공유 함수에 `spare_rows` 인자를 둔 `screen_framebuffer` 를 더했고, KTF 는 0 을 넘긴다(바이트 단위로 이전과 같다).
**왜**: 0392 §4 의 벽 — `1eaa92092bee` · `fe76e641bb3d` 가 화면 버퍼 바로 뒤의 힙 목록 헤더를 덮어 `Allocation failure` 로 끝났다.
**사용자 영향**: `1eaa92092bee` 가 30초 조작 FAIL 3/3 → **PASS 3/3** · 지원 현황 limited → **playable**(playable 368 → 369 · limited 41 → 40). `fe76e641bb3d` 는 할당 실패를 지나 다음 벽(LGT WIPIC SVC 603 미구현)에서 멈춘다. 가드 6종(LGT 5 · KTF 1)은 전·후 같다.

증적: `~/orchestrator/reports/evidence/wie-lgt-framebuffer-overrun-heap-header-adopt-p0/`. sha12 만 쓴다.

### 1. 재현 · 쓰기 지점

탐침(스크래치 · 미커밋 · 증적 `probe.diff`): LGT `alloc_indirect` 가 처음 `0x25800`(240×320×2) 을 잡을 때 그 끝 주소를 감시하고, 엔진이 명령 하나를 실행할 때마다 그 8바이트가 바뀌면 pc·레지스터를 찍는다.

- `1eaa`: 첫 쓰기 `pc=0x1232e`(Thumb `strh r0,[r3,r1]`) · `r1=0x40346100` = 버퍼 시작 + **301줄** · `r3=0x23a0` = 19줄. 그 함수(`0x122d4`~)는 투명색 키(`cmp r0,[g+0xac]`)를 건너뛰는 이미지 복사이고, 목적지 줄 = `(y − y0) × stride` 다(증적 `1eaa-0x122d4.txt`). **클리핑이 없다** — 호출자가 준 범위를 그대로 쓴다.
- 같은 pc 에서 «버퍼 끝을 넘은 쓰기의 최대 줄»을 쟀다: **343**(증적 `1eaa-maxrow.txt`). 즉 이 게임의 화면은 **344줄**이다.
- `fe76`: 부팅 직후 `GetScreenFrameBuffer → GET_FRAME_BUFFER_WIDTH → GET_FRAME_BUFFER_HEIGHT → CreateOffScreenFrameBuffer(240, 344)`(증적 `fe76-offscreen-svc.txt`). 화면을 344줄로 보고 그 크기의 오프스크린을 만든다.

### 2. 344 는 어디서 오나 — `fe76` 의 호출부(binary.mod `0x289c4`)

```
fb = MC_grpGetScreenFrameBuffer(0)
W  = MC_GRP_GET_FRAME_BUFFER_WIDTH(fb)
H  = MC_GRP_GET_FRAME_BUFFER_HEIGHT(fb) - [0x1513ad4]   ; 여기서는 0
switch (W) { 176: bar = 20;  240, 320: bar = 24;  120, 128: bar = 14 }
MC_grpCreateOffScreenFrameBuffer(W, H + bar)                ; 240, 320 + 24
```

(증적 `fe76-0x289c4.txt`). 게임은 **알려준 높이 아래에 폭별 소프트키 줄이 더 있다**고 보고 그 줄까지 그린다. `1eaa` 는 다른 회사·다른 코드인데 같은 +24 를 쓴다(§1 최대 줄 343). 이 폭 표를 바이트 패턴으로 LGT `binary.mod` 54종에서 찾으면 `fe76` 1종만 걸린다(증적 `scan.py`) — 표는 그 게임의 코드이고, «+24 를 그린다»는 두 게임이 따로 보여 준 사실이다.

### 3. 0392 §4 의 «한 줄 여유를 주면 주소가 같이 밀린다»는 다르게 읽힌다

0392 는 그것을 «게임이 할당 크기에서 주소를 얻는다»로 읽었다. 그런데 감시하던 것은 **다음 블록의 헤더**였다. 버퍼를 한 줄 늘리면 헤더 자체가 한 줄 뒤로 가고, 게임은 여전히 24줄을 넘어 쓰므로 «덮인 주소가 한 줄 밀린» 것으로 보인다. 게임이 크기 단어를 읽는 호출은 두 게임 어디에서도 나오지 않았다(§1 의 목적지는 `r1` = 줄 시작 + 행 오프셋이고, `fe76` 의 크기는 `H + bar` 이다). 여유가 소용없었던 이유는 **1줄이라서**다. 필요한 것은 24줄이다.

### 4. 왜 «높이를 296 으로 알려준다»가 아니라 «24줄을 더 잡는다»인가

실기에서 `GET_FRAME_BUFFER_HEIGHT` 가 «LCD − 소프트키»(320 → 296)를 돌려줬다면 `fe76` 의 캔버스는 정확히 320 이 되어 화면에 다 보인다 — 그쪽이 더 그럴듯하다. 그러나 그 값은 **LGT 클렛 타이틀 전부의 배치**를 바꾼다(높이로 화면을 짜는 타이틀이 아래 24줄을 잃는다). 이 회차는 그것을 잴 수단이 없어 택하지 않았다. 지금 처방은 «그려도 되는 메모리»만 실기에 맞춘다: 보이는 화면은 이전과 같고(아래 24줄은 전에도 화면 밖이었다), 다른 타이틀에는 힙 배치가 24줄만큼 밀리는 것 말고 바뀌는 것이 없다 — 가드로 쟀다(§5).

### 5. 짝 재측

전 = `origin/main` `c387adef` · 후 = 이 PR · release · `--inject --pacing 8 --relaunch 1` · 3판씩 번갈아 · 시작 시 `host-load-guard --status --recovered` rc=0 · 순차(jobs 1) · load1 12~62.

| sha12 | 플랫폼 | 전 ×3 | 후 ×3 |
|---|---|---|---|
| `1eaa92092bee` | LGT | FAIL ×3 · 9~12번째 키 · Java 예외(할당 실패) | **PASS ×3** · 27/27 |
| `fe76e641bb3d` | LGT | FAIL ×3 · 2번째 키 · Java 예외(할당 실패) · paints 37 | FAIL ×3 · 20번째 키 · **SVC 603 미구현** · paints 154~161 |
| `2a57e33133b5` | LGT | PASS ×3 | PASS ×3 |
| `517ed32c92d6` | LGT | PASS ×3 | PASS ×3 |
| `ddd885583b15` | LGT | PASS ×3 | PASS ×3 |
| `863b8ab6a21d` | LGT | PASS ×3 | PASS ×3 |
| `40b9537968de` | LGT | PASS ×3 | PASS ×3 |
| `49ade89578c5` | KTF | PASS ×3 | PASS ×3 |

**부하 축을 숨기지 않는다.** 첫 묶음 도중 다른 레인의 부하로 `host-load-guard` 보류가 섰다(since `1790837634` — `863b`·`40b9` 일부와 `--max-ticks` 재측 전부가 그 안이었다). 그래서 보류가 풀린 뒤(`--status --recovered` rc=0 · 끝에서도 rc=0 · 증적 `hl-r3.txt`) 그 4종(`1eaa`·`517ed`·`863b`·`40b9`)을 `--max-ticks 2000000000` 으로 3판씩 다시 쟀다(증적 `pair3.log`). 결론은 같다: `1eaa` 전 FAIL ×3(11~12번째 키) · 후 PASS ×3 · 가드 3종 전·후 PASS(`40b9` 전 3판째 1회는 15/27 키에서 `clean exit` UNMEASURED — 전 빌드 쪽이다). 위 표는 이 재측까지 합친 값이다.

`1eaa`·`517ed` 는 첫 묶음에서 release 가 기본 `--max-ticks` 에 먼저 닿아 UNMEASURED(전·후 모두 `517ed` · 후 `1eaa`)였다 — FAIL 이 아니다. 그 둘만 `--max-ticks 2000000000` 으로 다시 3판씩 쟀고 위 표가 그 값이다(증적 `pair.log`·`pair2.log`).

### 6. 지원 현황(`compat.json`)

`playability-census.mjs run --jobs 2 --long 600`(후 빌드 release) · 두 대상만 담은 스크래치 corpus(심링크). 시작 시 `--recovered` rc=0, **끝날 때는 다른 레인 부하로 보류**(load1 60 · rc=1) — 장시간 축은 «오류 없이 버텼나»이므로 부하는 덜 진행시킬 뿐 거짓 ok 를 만들지 않는다.

| sha12 | A(30초) | L(600초) | census | 이 PR 의 행 |
|---|---|---|---|---|
| `1eaa92092bee` | 19/27 키 · `max-ticks`(UNMEASURED) | 854/900 키 · deadline · 오류 0 | playable · longplay ok | **limited → playable** · longplay no → ok · 알려진 문제 «멈추거나 꺼짐» 삭제 |
| `fe76e641bb3d` | FAIL · 20번째 키 · SVC 603 | 돌지 않음(A 가 그린 뒤 실패) | limited · longplay error | 그대로(sound 는 census 가 ok 를 냈지만 이 회차가 잰 축이 아니다) |

`status`·`longplay`·`knownIssues_ko` 만 바꿨다(0398 과 같은 방식) · 증적 `census/`.

### 7. 되돌리면 red

`get_screen_framebuffer` 가 `spare_rows` 0 을 넘기게 하면 `wipic_screen_framebuffer_has_softkey_rows_below_its_height` red(1 failed). 이 시험은 «알려주는 높이는 그대로»도 함께 잡는다.

### 8. 게이트

- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable · beta) · wasm32 clippy · `RUST_MIN_STACK=4194304 cargo test --all`(51 묶음 · 실패 0) — 전부 rc=0. 이 worktree 의 target · build-slot 경유.
- 러너 블록(debug): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me`(`--timeout 5`) PASS · `keydraw_ktf`·`keydraw_lgt` `--inject --expect-last-frame` PASS · `last_frame_content true` · rc=0.
- `node scripts/player-data.mjs` OK(429 · 369/40/20) · `node scripts/check-worklog-json.mjs` OK · `node scripts/check-docs-report-serial.mjs` OK.
- 유입(`node scripts/corpus-name-inflow.mjs`): BOUNDED 337쌍 · SUFFIX-ATTACHED 15쌍 — 전부 이 PR 이 손댄 파일에 이미 있던 줄이다(`compat.json` 의 `title`·`fileTitle` · `wie-lgt/src/runtime/wipi_c.rs`·`wie-wipi-c/src/api/graphics.rs` 의 기존 주석 — 타이틀별 출현 수가 `origin/main` 과 같다). 이 회차가 더한 줄(코드 · 이 문서 · worklog · player-updates · `compat.json` 의 `status`/`longplay`/`knownIssues_ko`)의 게임명은 0 — 타이틀은 sha12 로만 적었다.
