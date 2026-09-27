## [2026-09-27] featurephone 「바로 해보기」 데모 — 외부 후보 9종 실측 · 직접 만든 MIT 데모 Pebble Snake 채택 (wie-featurephone-demo-game-license-clean-selection)

**무엇을**
- `demo/pebble-snake/` — 우리가 쓴 MIDP 게임(MIT). 소스 3파일 · 컴파일 전용 MIDP 스텁 · `build.sh`(JDK `--release 7` · preverify 없음 · `jar --date` 로 재현 가능) · `LICENSE` · `README.md`.
- `.gitignore` — `/demo/*/out/`(빌드 산출물. jar 는 원래 `*.jar` 로 무시된다).
- Rust 변경 0 · 엔진 변경 0.

**왜**: 운영자 지시(2026-09-27) 「저작권 문제 없는 공개 데모 게임도 넣어라」. 파일이 없는 첫 방문자가 셸에서 바로 눌러 볼 게임이 필요하다. 셸 번들은 후속 `otterpebble-featurephone-bundled-demo-game-try-now` 몫.

**사용자 영향**: 이 회차 자체로는 없다(셸이 아직 번들하지 않는다). 번들되면 첫 화면에서 한글 조작 안내가 붙은 게임 하나가 바로 돈다.

### 외부 후보 — 라이선스 원문이 있는 것만 · 현 `origin/main`(f12e16e8) 엔진으로 재실측

| 후보 | 저작자 · 라이선스 | 부팅 | `--inject` | 막힌 곳 / 판정 |
|---|---|---|---|---|
| TilePuzzle | Sun (WTK 2.5.x) · BSD-3 | PASS | PASS(16 paints) | 돈다. 영문·흑백 LCDUI 폼 위주라 첫 경험으로 약함 — **차선** |
| WormGame | Sun (WTK) · BSD-3 | PASS(34) | **FAIL** | `Manager.createPlayer(String)`(톤 플레이어) 없음 |
| PushPuzzle | Sun (WTK) · BSD-3 | FAIL | — | `lcdui/game/LayerManager` 없음 |
| Lines | Thinh Pham · GPL-3.0+ | FAIL | — | `Image.createImage(InputStream)` 없음 |
| Sliding Puzzle | Thinh Pham · GPL-3.0 | FAIL | — | 같은 메서드 · 그림 10장 출처 불명(탈락) |
| StickFight | 가명 「Dash animation v2」· MIT | FAIL | — | `RecordStore.enumerateRecords` 없음 · 저작자 실체 불명 |
| Shooter | Muhammad Rehan Saeed · MIT | FAIL | — | `lcdui/List` 없음 |
| wipi_game (WIPI) | Inseok Lee · MIT | FAIL | FAIL | KTF·LGT 모두 단색 프레임만(10 / 133 paints · 색 1) |
| Pixelus · Mummy Maze | Thinh Pham · GPL | — | — | **탈락** — PopCap 타이틀의 무단 이식(코드 라이선스와 별개로 IP 가 저작자 것이 아니다) |

라이선스 원문이 없어 **탈락**: camark/j2me-tetris · tvc97/Co-Caro · alvarogzp/heroe-del-futuro · wmok/midlet_game · isaacflaum/SlimeVolleyball(타사 게임 이식이기도) · tequilacat/TCatris. 그 밖 탈락: gravitydefied(안드로이드 이식 · 원저작권자 미허락) · freebitva(뼈대만). upstream 소개 사이트의 「바로 해보기」는 번들 데모가 없는 웹앱 링크다.
상용·출처 불명 바이트 커밋 **0**. 외부 후보 파일은 repo 밖(`/tmp`)에만 두었다.

### 직접 만든 데모 — Pebble Snake (채택)

| 항목 | 실측 |
|---|---|
| 크기 · 해시 | **7,688 B** · sha256 `5ed67caa7a8b473cacc0a139fcff2df1a51d17f83903c6581d1c2009d7d05c68`(openjdk 17.0.20.1 · 2회 빌드 동일) |
| 첫 페인트 | `--timeout 1` 3회 모두 PASS(paints 1·6·5) ⇒ **1초 안** |
| 조작 | 13키 스크립트(OK·방향·2/4/6/8·5·0) PASS · 예외 0 · 방향 전환·멈춤·소리 끔 표시가 캡처로 확인 |
| 소리 | `audio.plays` **7** · `midi_events` **31**(시작·방향·멈춤·게임끝) — 엔진이 효과음을 합성 단까지 보낸다 |
| 속도 | 10초 창 paints **41–52**(3회 · loadavg 180대) · `sleep_late_p50` 23–46ms. 설계 목표 6~8fps 에 조금 못 미치는 4~5fps(부하 탓 — 아래) |
| 실브라우저 | `WIE_BASE=https://wie-web.pages.dev node scripts/verify-browser.mjs <jar>` → rc 0 · 타이틀·한글 안내 렌더 · 콘솔 에러 0 · off-origin 0 · 게임 바이트 요청 0 |

측정 시 loadavg **178–245**(다른 워크트리 빌드 수십 개). 속도 수치는 그 부하 아래 값이다.

### 채택 이유
1. 라이선스 판단이 필요 없다(우리 저작 MIT · 외부 자산 0 · 글꼴은 호스트 neodgm).
2. 외부 후보 중 입력까지 통과한 것은 TilePuzzle 하나였고, 그것은 영문 폼 퍼즐이다. Pebble Snake 는 **한글 조작 안내**가 첫 화면이고 방향키·숫자키·확인·소리를 한 번씩 다 쓴다 — 셸의 조작 안내 역할을 겸한다.
3. 엔진이 가진 MIDP 표면만 쓴다 — `stubs/` 로 컴파일하므로 없는 메서드를 부르면 빌드가 실패한다.

### 만들다 밝힌 엔진 결함 1건 (고치지 않았다 — upstream 크레이트)
문자열 상수에 NUL 이 든 클래스(`"MTR\0"`)는 `ClassFormatError: Invalid class file` 로 로드되지 않는다. 클래스 파일은 NUL 을 modified UTF-8 `C0 80` 으로 적는데 `classfile 0.1.1` `parse_utf8` 이 `String::from_utf8` 로 읽어 거부한다(보조 평면 문자의 서로게이트 쌍 인코딩도 같은 이유로 거부될 것 — 미실측). 데모는 태그 바이트를 따로 써서 우회했다. 후속 제안으로 남긴다.
