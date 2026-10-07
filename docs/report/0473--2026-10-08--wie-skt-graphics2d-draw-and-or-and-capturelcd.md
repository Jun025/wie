## [2026-10-08] SKT Graphics2D 그림 벽 — DRAW_AND/OR · captureLCD · copyLCD (wie-skt-graphics2d-draw-and-or-and-capturelcd)

**무엇을**: `com/skt/m/Graphics2D.drawImage` 의 모드 1·2(`DRAW_AND`·`DRAW_OR`)가 경고만 남기고 그리지 않던 것을 픽셀 연산으로 그리게 했다. `Graphics2D.captureLCD` 가 빈 그림을 돌려주던 것을 화면 그림을 잘라 주게 했다. 같은 벽인 `com/xce/lcdui/XDisplay.copyLCD`(빈 no-op)도 구현했다.
**왜**: 0469 §6 후속 군집 2종(worklog `2026-10-07-progression-wave4-skt-lgt-followups#p1` 채택). 운영자 지시 «더 많은 게임 정상화».
**사용자 영향**: `d1dce4a36141` 은 시작 로고 자리가 검은 상자였는데 이제 로고가 나온다. `ec2f8f2e02a2` 는 메뉴 뒤에 지난 화면의 아이콘·글자 조각이 남아 있었는데 이제 깨끗한 배경이 나온다. `c107462e5f8a` 는 걸어 다닐 때 주인공 그림이 지워지지 않고 열댓 개씩 줄지어 남았는데 이제 하나만 남는다(`copyLCD`). 진도(`progress`)는 6종 모두 그대로 `stuck` 이다 — 이번 수정은 그림만 바꾼다.

타이틀은 sha12 로만 적는다. 스크린샷·키 레시피는 커밋하지 않았다(Constraint 9).

### 1. SKVM 의미 — 근거

원 단말 문서는 저장소에 없다. 근거는 0459 와 같은 KEmulator(SK-VM 에뮬레이터 `KEmulator-mmpp.jar`)의 `javap -c` 다.

- `Graphics2D.drawImage(tx, ty, src, sx, sy, sw, sh, mode)`: 원본 `getRGB` 와 대상(화면 Graphics 의 이미지) `getRGB` 를 같은 크기로 읽고, 모드 1 은 `dst & src`, 모드 2 는 `dst | src`, 모드 3 은 `dst ^ src` 를 픽셀마다 계산한다. 결과를 `createRGBImage(…, false)`(알파 무시 = 불투명)로 만들어 (tx, ty) 에 그린다.
- `Graphics2D.captureLCD(x, y, w, h)`: `IScreen.getBackBufferImage()` 를 `Image.createImage(back, x, y, w, h, 0)` 로 자른다.
- `XDisplay.copyLCD(gfx, image, x, y, w, h)`: `Graphics2D.captureLCD(x, y, w, h)` 를 `image.getGraphics().drawImage(…, 0, 0, TOP|LEFT)` 로 그린다. `gfx` 는 쓰지 않는다.

이 저장소에서의 대응:

| 축 | 선택 | 이유 |
|---|---|---|
| 픽셀 형식 | 8비트 채널(`Color`)에서 `&`/`|` | 저장소의 픽셀 형식은 8비트로 펼칠 때 비트를 복제한다(예: RGB332 의 r·36 = `r<<5 \| r<<2`). 그래서 펼친 값의 `&`/`|` 는 원래 비트의 `&`/`|` 와 같고 되돌릴 때도 정확하다. 형식마다 연산을 따로 더할 필요가 없었다(티켓의 «5종 각각»은 필요 없었다) |
| 알파 | 결과는 불투명 | KEmulator 가 `processAlpha=false` 로 만든다 |
| 투명 원본 픽셀 | 대상을 건드리지 않음 | KEmulator 와 다르다(그쪽은 0 과 연산). 이 저장소의 `DRAW_XOR` 가 이미 «투명 원본 = 무동작»이라 같은 규칙으로 맞췄다. 원 단말 근거는 없다 |
| 클립·translate | 대상 Graphics 의 것 | 기존 COPY/XOR 경로와 같다 |
| 화면 back buffer | 현재 MIDlet 의 `Display` 화면 이미지(마지막 paint 가 내보낸 것) | `getScreenGraphics` 로 얻는다. MIDlet 이 없으면 종전처럼 빈 그림 |
| `copyLCD` 의 null·빈 크기 | 무동작 | KEmulator 는 NPE·IAE 다. 종전 no-op 이던 호출을 새 예외로 깨지 않으려고 남겼다 |

### 2. 측정 조건

| 바이너리 | 소스 | 쓴 곳 |
|---|---|---|
| `wv_main` | `76423d05`(main) | 전 |
| `wv_fix` | AND/OR + captureLCD | 후(1차 짝) |
| `wv_fix2` | + copyLCD(= 이 PR 의 코드) | 후(c107 재짝 · 진도 P) |

- 전/후 짝: 같은 키(진도 레시피가 있으면 그것 · 없으면 census `PROGRESS_KEYS`) · `--keep-timeout --timeout 120`(c107 재짝 180) · 10초 간격 화면 · `--relaunch 1` · 틱 상한 해제. «전»과 «후»를 **임대 하나 안에서 동시에** 띄웠다(타이틀 하나씩 · 동시 2개). `RUST_LOG=…graphics_2d=debug` 로 호출 수를 셌다.
- 진도 P: census `--only progress --progress 600`(정책 v2 · `--jobs 3`)을 `wv_fix2` 로 돌렸다. census 의 진도 단계는 같은 `--out` 에 이전 판정(boot·render·longplay ok)이 있어야 대상을 고른다. 그래서 그 거름 한 줄만 뺀 **스크래치 사본**(저장소 밖)으로 돌렸다. 판정은 census 의 `progressCurve`·`fingerprint` 코드를 그대로 가져다 썼다(`stall ≥ max(180, secs/3)` = stuck). `--as P2` 짝은 돌리지 않았다 — 결과가 compat 의 기존 `stuck` 과 같아 바꿀 행이 없었다.
- ★임대 대기: long 풀 2칸을 다른 레인이 계속 쥐고 있었다(r4 레인 1개는 3.4시간째). 1차 짝 **약 28분** · c107 재짝 **약 19분** · 진도 P 첫 시도 **30분**(상한) · 진도 P 재시도 **약 30분**(상한). 첫 진도 P 는 위 거름 때문에 0종을 돌리고 끝났다 — 임대 30분을 그대로 버렸다.
- 호스트: 착수 시 `host-load-guard --status --recovered` rc=1(포화 · load 42) → 짝은 동시 2개. 진도 P 직전 rc=0(load 7.5) → `--jobs 3`.

### 3. 전/후 — 걸린 타이틀 전건 + 가드

호출 수 = 120초 동안 «전/후» 각각(captureLCD · AND · OR). 경고 = Graphics2D stub 경고 수. 화면 = 같은 이름 화면의 바이트 비교(같음/다름) — 애니메이션이 있는 타이틀은 시간 흔들림만으로도 «다름»이 나온다(가드 둘이 그 기준선이다).

| sha12 | 군집 | 호출(전→후) | 경고 전→후 | 화면 같음/다름 | 눈으로 본 차이 | 진도 P(후) |
|---|---|---|---|---|---|---|
| `d1dce4a36141` | AND/OR | AND 5 · OR 92 → 5 · 91 | 98 → 0 | 186 / 16 | ★부팅 첫 화면: «전» 흰 바탕 가운데 **검은 상자**(120×42) → «후» 제작사 로고. 첫 결함 지점 = 이 로고(마스크 AND 후 OR) | stuck(정체 240초) |
| `c107462e5f8a` | AND/OR + copyLCD | AND 72 · copyLCD 23 → AND 72 · capture 24 | 72+23 → 0 | 125 / 77(1차) · 27 / 280(재짝) | ★걷기 화면: «전» 주인공 그림이 지나간 자리마다 **줄지어 남음**(약 15개) → «후» 하나. 이건 `copyLCD`(1차 `wv_fix` 에서는 그대로 · `wv_fix2` 에서 사라짐). AND 만의 차이는 눈으로 가르지 못했다(1픽셀 높이 띠 · 0459) | stuck(440초) |
| `f12984cd0d37` | AND/OR | 0 → 0 | 0 → 0 | 12 / 26 | 120초 경로에서 AND/OR 에 **닿지 않았다** — 다름 26은 애니메이션 흔들림. 레시피 없음 | stuck(470초) |
| `ec2f8f2e02a2` | captureLCD | 2 → 2 | 2 → 0 | 1 / 37 | ★메뉴 화면: «전» 메뉴 뒤에 지난 화면의 아이콘·띠 조각이 남음(1,122 픽셀 · 예 `24da00`) → «후» 깨끗한 방 배경. 게임이 캡처해 둔 배경으로 지우는 구조 | stuck(350초) |
| `090877d7a3e0` | captureLCD | 10 → 10 | 10 → 0 | 38 / 0 | 120초 동안 화면 차이 0 — 캡처한 그림을 보이는 곳에 쓰는 장면에 닿지 않았다 | stuck(520초) |
| `7089dec0e8df` | captureLCD | 86 → 86 | 86 → 0 | 202 / 0 | 같음(화면 차이 0 · 레시피 경로) | stuck(420초) |
| `49ade89578c5` | 가드(KTF) | 0 | 0 | 28 / 10 | 결과 PASS → PASS · 색 378 → 378 — 애니메이션 흔들림만 | — |
| `ddd885583b15` | 가드(LGT) | 0 | 0 | 9 / 26 | 결과 PASS → PASS · 색 357 → 357 — 애니메이션 흔들림만 | — |

- 6종 전건: 결과·종료 사유 동일. Java 예외는 c107 만 흔들린다 — 1차 짝 8→9 · 재짝 10→9, 전부 세이브 슬롯 없음(`RecordStoreNotFoundException`) 종류이고 «후»가 늘지 않는다.
- 가드 퇴행 0: 두 타이틀은 SKT 코드를 지나지 않는다(Graphics2D 호출 0). 결과·색 수 동일.
- ★진도와의 관계: 6종 모두 `stuck` 그대로다. 0469 가 이 6종을 전부 ⒜(정책 키 한계)로 분류했고 이번 수정은 게임 상태가 아니라 그림만 바꾼다 — 화면 순서도 c107 은 키 단계 55 직전까지(1차 짝 · 숫자 순서) «전/후»가 바이트 동일했다. 따라서 compat 의 `progress` 는 고치지 않았다.

### 4. 못 한 것 · 다음 벽

- `090877d7a3e0`·`7089dec0e8df`: captureLCD 는 불리지만 그 그림이 화면에 쓰이는 장면까지 못 갔다. «고쳐졌다»고 말할 화면 증거가 없어 플레이어 소식에서 뺐다. 다음 확인은 그 장면(메뉴 뒤 대화·전환)으로 가는 레시피가 생길 때 — 크기 S.
- `f12984cd0d37`: 120초 경로에서 AND/OR 호출 0. 0469 의 stub 집계는 더 긴 실행에서 나온 것이다. 화면 증거가 없어 소식에서 뺐다.
- `XDisplay.clear`(0469 이전부터 stub)는 이번 6종의 경고 목록에 없었다. 손대지 않았다.
- 새 엔진 벽은 찾지 못했다(6종의 «후» stub 경고 목록에 Graphics2D·XDisplay 항목 0).

### 5. compat · 소식

- `compat.json`: 바꾼 행 0(진도 그대로 · 그림 관련 `knownIssues_ko` 없음). `check-compat-revert` rc=0.
- `docs/player-updates/2026-10-08-skt-graphics2d-mask-capture.json`: 화면 증거가 있는 3종(`c107462e5f8a` `d1dce4a36141` `ec2f8f2e02a2`)만.

### 6. 게이트

- `cargo fmt --check` · `cargo clippy --all -D warnings` · wasm32 · `+beta` 전부 rc=0. `RUST_MIN_STACK=4194304 cargo test --all` **735 passed / 0 failed**(새 테스트 2). `npm run build:wasm` rc=0 · `check-engine-contract` 113 pass / 0 · `npm run audit` PASSED · `player-data` OK · `check-worklog-json` OK.
- 되돌리면 red: AND/OR 분기를 막으면 `graphics_2d_and_or_modes_combine_target_and_source_inside_the_clip` · captureLCD 를 빈 그림으로 되돌리면 `graphics_2d_capture_lcd_copies_the_screen_image` · copyLCD 를 no-op 으로 되돌려도 같은 테스트가 red(각각 실제로 돌려 확인).
- 러너 블록: draw · helloworld ×2 · text PASS(`wv_fix2` 릴리스) · keydraw ×2 `--inject --expect-last-frame` PASS rc=0(paints 79 / 55 · 문서 그대로 `cargo run` 디버그). ★릴리스 바이너리로 같은 줄을 돌리면 keydraw 가 `UNMEASURED`·`stop=max-ticks`(키 5개)다 — main 바이너리도 똑같다. 릴리스가 빨라 5천만 틱 상한을 키 일정 전에 다 쓴다. 이 회차의 퇴행이 아니다.
- 회차가 띄운 프로세스: 끝낼 때 `wv_main`·`wv_fix*`·census 0(`pgrep` 확인).
