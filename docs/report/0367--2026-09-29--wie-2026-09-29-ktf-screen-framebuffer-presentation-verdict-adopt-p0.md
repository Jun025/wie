## [2026-09-29] KTF Java 모드 — 네이티브 화면 프레임버퍼를 Java paint 주기에 화면으로 올린다 (wie-2026-09-29-ktf-screen-framebuffer-presentation-verdict-adopt-p0)

**무엇을**: midp `Display` 가 Java 화면 이미지를 올리기 직전에 KTF 가 건 갈고리(`System::set_screen_compositor`)를
부른다. 갈고리(`wie-wipi-c` `ScreenFramebufferSync`)는 `MC_grpGetScreenFrameBuffer` 버퍼와 Java 화면 이미지를
**양방향**으로 맞춘다 — 지난 동기 이후 네이티브에서 바뀐 화소는 Java 이미지로, Java 에서 바뀐 화소는 네이티브 버퍼로.

**왜**: 0361 — 대상 2종이 그림을 전부 화면 프레임버퍼에 그리고 `FlushLcd` 없이 `Card.repaint` 로 화면 갱신을 맡긴다
(실기에서는 한 메모리). wie 는 Java 이미지만 올려 흰 화면이었다.

**사용자 영향**: 대상 2종(`4120f27288ab` `739c7657c1f2`)이 흰 화면 대신 인트로(그림 + 한글 자막 + `#SKIP`)를 보여 준다.
다른 KTF 타이틀은 판정 변화 0(아래 전수).

### 재현(0361)
- 기준(`origin/main` 57506d11 · 비-LTO release) 대상 `739c7657c1f2` 20초: `MC_grpFillRect` 10,049 · `DrawString` 1,751 ·
  `PutPixel` 342 가 전부 핸들 `0x49052ce0`(= `MC_grpGetScreenFrameBuffer(0)`), `MC_grpFlushLcd` 0. 대상 2종 `FAIL` · `distinct_colors` 1.

### 설계 — 왜 «차이» 이고 왜 «양방향» 인가
- 통째 교체는 Java 로 그리면서 버퍼만 받아 두는 타이틀을 가린다 → 네이티브 쪽 **바뀐 화소만** 올린다. 첫 동기는 전부 올린다(검정 포함).
- ★단방향 차이로는 모자랐다(측정): 게임이 **Java 로 한 번** 화면을 흰색 `fillRect(0,0,240,320)` 한 뒤 네이티브로 같은 검정을
  다시 칠하면 값이 안 바뀌어 차이에 안 잡힌다 → 하단 약 25줄이 흰 띠로 남았다(띠 영역 흰 화소 6,000 → 2,696 로 남음).
  실기라면 한 메모리라 검정이 이긴다. ⇒ Java 에서 바뀐 화소를 네이티브에 되써 «한 메모리»를 paint 단위로 흉내 낸다.
  그 뒤 네이티브의 검정 재칠은 «변화»가 되어 올라간다(띠 소멸 · 스냅숏 확인).
- 천장(`ponytail:` 주석): 두 paint 사이에 **양쪽이 같은 화소를 바꾸면 네이티브가 이긴다**(실제 순서 무관). 그리기 단위 기록이 해법.
- Clet 모드는 midp paint 가 꺼져 갈고리가 불리지 않는다(`disable_midp_paint`) — 스스로 `FlushLcd` 하는 2종도 첫 paint 뒤 꺼진다(실측: 각 `disablePaint` 1 · `handlePaintEvent` 1).
- 곁 수정: `JavaImageBuffer::put_pixels` 가 줄 끝을 넘는 구간을 다음 줄로 흘리고(마지막 줄이면 panic) `x == width` 를 통과시켰다 → 경계 자름.

### 비용(240x320 · 대상 `739c7657c1f2` 20초 · 임시 계측 · 커밋 0)
| 단계 | 평균 µs/paint (n=200 · load1 ~130) |
|---|---|
| 네이티브 버퍼 읽기 | 12 |
| Java 이미지 읽기 | 16 |
| 비교 + 구간 쓰기 | 957 |
| 되쓰기 + 재읽기 | 17 |
- 합 약 1.0 ms/paint. load1 ~210 에서 잰 갈고리 전체는 3.48 ms/paint(n=400). 비교 루프가 거의 전부다(바뀐 화소마다 색 변환 + 구간 쓰기).
- 버퍼를 받지 않은 타이틀은 `0x7fff1000` 한 칸 읽고 돌아간다(따로 재지 않음).

### KTF 전수 짝(기준 ↔ 이번 · 같은 시각 동시 실행 · 비-LTO release · 12초 · 키 없음 · `__adf__` 보유 269종 · load1 170~840)
| 기준 → 이번 | 수 |
|---|---|
| PASS → PASS | 244 |
| FAIL → FAIL | 22 |
| FAIL → PASS | **2 (대상)** |
| PASS → FAIL | 1 → 재측 0 |
- PASS→FAIL 1건(`23919eb33365`, paints 3 → 0)은 굶주림이었다: 20초 짝 3회 전부 PASS/PASS(paints 70/69 · 47/50 · 71/76).
- PASS/PASS 244종에서 마지막 프레임 «빈 화면(색 ≤2)» 여부가 뒤집힌 타이틀 0.
- 1순위 확인 2종(`93d5b6b8ceb5` `974e0df9ab1e` · 스스로 `FlushLcd`) 20초: 전 PASS(29색 · 2색) → 후 PASS(435색 · 2색). 첫 paint 뒤 midp paint 가 꺼지므로 갈고리 영향 밖.

### 변이
- 갈고리 등록(`set_screen_compositor`) 제거 빌드: 대상 2종 20초 `FAIL` · `distinct_colors` 1 · content false(= 흰 화면 복귀, red).
- 단위 시험 `screen_sync_is_two_way`: 첫 동기 전부 · 무변화 0 · 네이티브 변화 구간 · Java 흰 화소 → 네이티브 되씀 · 같은 검정 재칠 = 변화.

### 다음 벽
- 대상 2종은 인트로까지 확인(12·16·20·25초 스냅숏). 키 입력 이후는 이 회차가 재지 않았다 — 제안 1.

### 검증
- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable·beta) · wasm32 clippy · `RUST_MIN_STACK=4194304 cargo test --all`(588 passed · 0 failed) rc=0.
- 러너 블록: draw_j2me · helloworld_ktf · helloworld_lgt · keydraw_ktf(rc=0) · text_j2me PASS. keydraw_lgt 첫 판 `UNMEASURED`(load1 ~370 굶주림) → 재측 2회 `PASS` · stop deadline · 27/27 키 · rc=0.
- `node scripts/corpus-name-inflow.mjs`: BOUNDED **5쌍(6회)** + SUFFIX-ATTACHED **0쌍**. 5쌍 전부 이 회차가 고친 파일의 **기존 줄**이다
  (`system.rs`·`display.rs` 의 페이싱 주석 · `graphics.rs` 의 null-guard 주석) — `git diff origin/main...HEAD` 의 추가·삭제 줄에서 그 이름 0회.
  이 회차가 새로 들인 타이틀 표기는 sha 앞 12자뿐이다.

<!-- corpus-name-inflow v1 subjects=8 tree=b4a776c0f995f04e B=6/5 P=0/0 S=0/0 -->
