## [2026-10-09] 재배포 허락된 MIDP 게임 코퍼스 · MIDP 2.0 일반 게임 표면 (wie-license-clean-game-corpus-expansion-open-source-j2me-wipi)

**무엇을** — 재배포가 **문서로** 허락된 피처폰 게임을 찾아(후보 78 · 채택 20 · 보류 16 · 제외 42),
소스에서 직접 빌드해 돌리고, 그 게임들이 밟은 엔진 벽을 고쳤다. 상용 게임 파일은 한 개도 받지 않았다
(총괄 판정 · featurephone 의 BYOF 법무 모델). 원장 = `docs/oss-corpus/`.

**왜** — 통신사 코퍼스(`game_lab/`)는 WIPI-C 와 통신사 자바가 대부분이라 일반 MIDP 2.0 표면
(`lcdui.game`, `List`, MIDI/WAV 플레이어)을 거의 밟지 않는다. 채택한 일반 J2ME 게임 117개 중
**44개가 부팅·첫 화면에서 죽었고**, 원인은 전부 «없는 클래스·메서드» 였다.

**사용자 영향** — 일반 J2ME 게임(노키아·소니에릭슨용으로 만든 것)이 스프라이트·타일 지도·목록 화면·
MIDI/WAV 소리를 쓰면 이제 열리고 소리가 난다. MIDP `Graphics.drawRegion` 은 이제 변환(좌우 반전·회전)을
그린다 — 그동안 변환 인자를 버리고 원본 방향으로 그렸다. 통신사 코퍼스(`game_lab/working` 294개)에는 그 메서드를
부르는 클래스가 **0개**라(중첩 jar 까지 바이트 검색) 이 변경이 닿지 않는다.

### 1. 코퍼스 — 무엇을 왜 받았나

| 판정 | 수 | 대표 사유 |
|---|---|---|
| 채택 | 20 | OSI 라이선스 파일 또는 파일별 헤더(GPL 9 · MIT 7 · BSD-3 2 · MIT-0 1 · CC0 1) |
| 보류 | 16 | 코드는 허락됐으나 자산 출처 불명 · 타사 IP(제목·캐릭터·음악) · NC 라이선스 · 독점 SDK 동봉 |
| 제외 | 42 | 라이선스 없음 21 · 상용작 디컴파일·역공학·이식·펌웨어 추출 12 · IP 클론·이미지 출처 불명 5 · 게임 또는 J2ME 가 아님 4 |

- 채택 20 = J2ME 18 항목(단일 17 + MIT 100종 묶음 1 → jar **117**) + WIPI 2(dlunch/wipi_game 빌드 · libwipi 는 툴체인 부재로 미빌드).
- 직전 회차(#360) 판정 둘을 **정정**: SlimeVolleyball 은 7/7 파일에 GPLv2+ 헤더가 있다(→ 채택) ·
  RehanSaeed/Shooter 는 Git LFS 이미지가 출처 표기 없는 스프라이트팩 계열이고 TV 테마곡 mp3 를 동봉(→ 보류).
- **j2me-lines** 의 `util/ImageHelper.java` 는 헤더가 저장소에 없는 Nokia `LICENSE.TXT` 를 가리킨다 —
  추정으로 넘기지 않고 15줄 clean-room 대체로 빌드했다(같은 시그니처 · 나머지와 같은 GPL-3.0).
- **jar 는 커밋하지 않는다** — Constraint 9 의 기전(이 repo 가 셸 배포용 wasm 을 낸다)은 허락된 jar 와
  아닌 jar 를 구별하지 못하고, GPL 바이너리를 실으면 소스 제공 의무가 산출물에 붙는다. 근거 전문·재현 방법 =
  `docs/oss-corpus/README.md`.

### 2. 고친 엔진 벽 — 전/후

boot 프로브(무입력 12 s · `--jobs 2` · 같은 jar 119개 · base = `origin/main` `5137d54c`, after = 이 PR):

| | base | after |
|---|---|---|
| PASS | **75** | **113** |
| 부팅/첫 화면에서 죽음 | 44 | 6 |

죽던 44개의 첫 예외(base) → after:

| 벽 | base 에서 죽은 수 | after |
|---|---|---|
| `Image.createRGBImage` 없음 | 18 | 18 PASS |
| `Graphics.fillTriangle` 없음 | 16 | 16 PASS |
| `lcdui.game.Sprite` 없음 | 1 | PASS(MIDI 1,212 이벤트 재생) |
| `Image.createImage(InputStream)` 없음 → 그 뒤 `Class.getResourceAsStream` null | 1 | PASS |
| `TextField.<init>` 없음 | 1 | PASS |
| `Canvas.hasPointerEvents` → `Font.getFace` 없음 | 1 | PASS |
| `java.lang.ref.WeakReference` 없음 → `file.separator` null | 1 | **멈춤**(§4) |
| 범위 밖 3 · WIPI 2 | 5 | 그대로(§4) |

고친 것(파일 = 그 벽의 정본 주석):

| 무엇 | 어디 | 잠그는 테스트 |
|---|---|---|
| `Layer`·`Sprite`·`TiledLayer`·`LayerManager` 신설 · `GameCanvas.getKeyStates`·`flushGraphics(IIII)` | `wie-midp/.../lcdui/game/` | `game::test::sprite_tiled_layer_and_layer_manager_paint` · `sprite::test::*` |
| `drawRegion` 이 `transform` 을 버리고 RIGHT 앵커에 `height` 를 쓰던 결함 | `lcdui/graphics.rs` `draw_transformed` | 위 테스트(반전 픽셀) · `rgb_images_round_trip_through_a_mirrored_region` |
| `Image` createImage(InputStream)/(Image,IIIII)·createRGBImage·getRGB·isMutable | `lcdui/image.rs` | `rgb_images_round_trip_through_a_mirrored_region` |
| `lcdui.List`(Form + ChoiceGroup 위임 · IMPLICIT 의 FIRE → SELECT_COMMAND) | `lcdui/list.rs` | `implicit_list_selects_the_focused_element` |
| `TextField` 생성자·읽고 쓰기(StringItem 위 · 키패드 편집은 없음) | `lcdui/text_field.rs` | `text_field_holds_what_the_game_writes` |
| `fillTriangle`·`copyArea`·색 성분·`getDisplayColor`·stroke | `lcdui/graphics.rs` | `triangle_spans_cover_rows_within_the_clip` |
| `Font` getFace/Style/Size·isBold… · `Canvas` 포인터 3종·getKeyCode/Name | `font.rs` · `canvas.rs` | (없음 — 상수 반환) |
| `java.lang.ref.Reference`/`WeakReference`(강참조) | `wie-midp/src/classes/java/lang/` | (없음) |
| MIDP 표준 시스템 속성(`microedition.*`·`file.separator`) | `wie-j2me/src/emulator.rs` | (없음) |
| `Manager.createPlayer` 가 SMAF 외 전부 `MediaException` → MIDI(SMF 0/1 · 템포 맵)·PCM WAV·`playTone`·`ToneControl`·`VolumeControl` | `wie-backend/src/system/audio_formats.rs` · `media/manager.rs` · `net/wie/player_control.rs` | `audio_formats::tests::*` · `test_unsupported_player_controls`(갱신) |
| `Class.getResourceAsStream` 이 플랫폼 클래스(부트스트랩 로더)에서 null → MIDlet jar 폴백 | `wie-jvm-support/src/hardening.rs` | `every_guard_is_actually_applied`(`java/lang/Class` 1) |
| `new InputStreamReader(in, "US-ASCII")` 가 `UnsupportedEncodingException` → UTF-8 로 읽음(ASCII 의 상위집합 · ISO-8859-1 은 그대로 예외) | 같은 파일 | `us_ascii_reader_opens_and_reads` · `every_guard…`(`InputStreamReader` 2) |

### 3. 채택분 6축 + 진도

`scripts/playability-census.mjs run --jobs 2 --secs 20 --long 30` + `--only progress --progress 300`(대상 = 이름 붙은 19개 중 boot·render·longplay ok 9개),
after 바이너리(엔진 코드 = `3b079c29` · §2 마지막 행의 US-ASCII 수정 전), load1 11~134. 축 순서 boot·render·input·longplay·sound·speed·progress
(`o` ok · `n` no/none/silent/error · `u` 측정 안 됨/n/a · `s` stuck). 원본 census.tsv = 회신 증거 폴더.

**전체 119: playable 108 · limited 5 · not-yet 6.**

| title | sha12 | status | boot · render · input · longplay · sound · speed · progress | note |
|---|---|---|---|---|
| bubblet-asha | `327cea349314` | not-yet | `nnuuuuu` | tick error during 'boot': Fatal error: java.lang.NullPointerException: |
| finifactory | `69913109bf0c` | not-yet | `nnuuuuu` | tick error during 'boot': Fatal error: java.lang.NoClassDefFoundError: |
| j2me-2048 | `8cc1b48394d5` | limited | `oonunuu` | --inject delivered 1/27 input steps (run ended: clean exit) — input su |
| j2me-lines | `cea17e90494e` | playable | `oooonou` | booted + rendered + survived input sequence (visual correctness NOT ch |
| j3de | `9fdfe561edf0` | limited | `oonunou` | booted + rendered + survived input sequence (visual correctness NOT ch |
| kurve | `d802d0331bc1` | not-yet | `nnuuuuu` | tick error during 'boot': Fatal error: java.lang.NoClassDefFoundError: |
| loveme | `6aefa71fd645` | not-yet | `nnuuuuu` | died on SIGKILL (census kill at 140s) |
| minitruco | `a72a2e40174c` | playable | `oooonoo` | booted + rendered + survived input sequence (visual correctness NOT ch |
| mobapp-game | `fee8af1b3a15` | limited | `ooonnou` | --inject delivered 39/45 input steps (run ended: deadline) — input sur |
| pipes | `a11c41405c4e` | playable | `oooonuu` | booted + rendered + survived input sequence (visual correctness NOT ch |
| redball | `8238f5449347` | limited | `oonunuu` | booted + rendered + survived input sequence (visual correctness NOT ch |
| slime-volleyball | `0ca8a98b8fa3` | playable | `oooonuu` | booted + rendered + survived input sequence (visual correctness NOT ch |
| sperm-race | `bf5b031018aa` | playable | `ooooooo` | booted + rendered + survived input sequence (visual correctness NOT ch |
| stickfight | `835acd535a5f` | playable | `ooooouo` | booted + rendered + survived input sequence (visual correctness NOT ch |
| sudoku-woodie | `4e86c821ccd2` | playable | `oooonuu` | booted + rendered + survived input sequence (visual correctness NOT ch |
| wipi_game_ktf | `c20b2f9318f4` | not-yet | `onuunuu` | only blank/uniform frames (black screen) |
| wipi_game_lgt | `2e4d71573ae6` | not-yet | `onunnuu` | SVC stub space exhausted (line written at exhaustion; see stderr) |
| wtk-tilepuzzle | `e75e4d830e0c` | playable | `oooonuu` | booted + rendered + survived input sequence (visual correctness NOT ch |
| wtk-wormgame | `f408cc6f846a` | playable | `ooooouu` | booted + rendered + survived input sequence (visual correctness NOT ch |

agneay-100(100종 · MIT · 전부 코드로 그림): **playable 99 · limited 1** — boot·render 100/100 · input ok 99(none 1 = `noughts-grid`, 첫 키가
마감 전에 1/27 만 닿음 · 4.3M ticks) · longplay ok 99 · sound ok 96(silent 4) · speed ok 87(n/a 13 = 0.9 미만 · 부하 측정).

- **진도**: 9개 중 ok 3(stickfight · minitruco · sperm-race), 나머지 6은 단발 정체(stall 200~290 s)인데 P2 짝을 돌리지 않아 census 규칙상 `n/a`.
  ★정체 6 중 다수는 아래 «입력 축의 공통 벽»(표준 키코드)과 겹친다 — 방향키가 먹지 않으면 새 화면이 안 나온다.
- **j2me-2048** `oonunuu`: census 는 US-ASCII 수정 전 바이너리였다 — 첫 키에 `InputStreamReader(in, "US-ASCII")` 가
  `UnsupportedEncodingException` 을 던져 게임이 스스로 `notifyDestroyed`. 수정 후 단발 `--inject`(틱 상한 해제): **PASS · 27/27 · 예외 0**,
  캡처에서 타일이 합쳐져 점수 20.
- **mobapp-game** longplay `n`: L 결과가 UNMEASURED(39/45 키 · 마감) — 예외 9건은 전부 첫 실행의 `RecordStoreNotFoundException`. 원인 미규명.
- **redball** input none: 공이 튀는 애니메이션 예제라 키를 받지 않는다(소스에 keyPressed 없음) — 엔진 문제 아님.
- 화면 확인(캡처 · 회신 증거 폴더): j2me-lines 보드·공(Sprite) · sperm-race 캐릭터 화면 · j2me-2048 진행 판.

### 4. 남은 벽 — 고치지 않은 것과 이유

| 게임 | 벽 | 판정 |
|---|---|---|
| finifactory (MIT) | `javax.xml.parsers.SAXParserFactory` — JSR-172 | 범위 밖: no_std 엔진에 XML 파서를 새로 들여야 한다(`quick-xml` 은 std). 한 게임 |
| kurve (GPL) | 생성자에서 `javax.bluetooth.LocalDevice` | 범위 밖: 블루투스 2인 대전 전용 — 스텁을 넣어도 «블루투스를 켜라» 경고 뒤 종료가 원작 동작 |
| bubblet-asha (GPL) | `System.getProperty("com.nokia.keyboard.type").equals(…)` | 범위 밖: Nokia Asha 전용(+JSR-211). 우리가 Nokia 로 자처하면 다른 게임이 Nokia 전용 API 로 간다 |
| loveme (MIT-0) | 고친 뒤 startApp 이 Lua 를 해석하며 **한 틱 안에서 30 s 넘게** 양보하지 않는다 — `wie_validate --timeout 12` 가 반환하지 않고 7분 넘게 CPU 를 썼다(sample: 바이트코드 해석기 루프) | 제안(#p1): 브라우저 호스트도 같은 시간 동안 멈춘다 |
| wipi_game (MIT · KTF/LGT) | 로딩 화면이 검정 단색(무입력 12 s: KTF 11 · LGT 10 paints · census 의 `--inject` 에서는 LGT 가 `SVC stub space exhausted`). SDK 가 `fgpxl` 에 ARGB32 를 그대로 넣는 것은 확인(16bpp 화면에서는 색이 틀어진다) — **단색 검정의 원인은 아니다**(흰 글자도 안 보인다) | 미규명 — 이 회차 시간 상한. WIPI-C 경로라 통신사 코퍼스와 같은 코드를 지난다 |

**입력 축의 공통 벽**(제안 #p0): 노키아·소니에릭슨용 게임은 방향키 -1~-4 · 확인 -5 · 소프트키 -6/-7 을
기다리는데 wie 는 SKT 값(위 141 · 소프트 6/7)을 보낸다. sperm-race 캐릭터 화면 실측 — RIGHT 두 번의 캡처는
직전 프레임과 같은 해시, 같은 자리의 NUM6 두 번은 새 화면 2장. J2ME 만 표준값으로 바꾸면 우리 데모
(`demo/arcade-common`)가 -1 을 종료키로 읽어 위쪽 키가 종료 확인을 띄우므로 **데모와 한 PR 로** 가야 한다.

### 5. 퇴행

- 4게이트: fmt ✅ · clippy `-D warnings` stable ✅ · wasm32 ✅ · beta ✅ · `RUST_MIN_STACK=4194304 cargo test --all` 실패 0 ✅
- 러너 블록(AGENTS §The four gates): draw_j2me · helloworld_ktf/lgt PASS · keydraw_ktf PASS · keydraw_lgt PASS(rc 0) — 첫 1회는
  UNMEASURED(load1 69) 였고 같은 부하에서 base·after 재실행이 둘 다 PASS 55 paints · text_j2me PASS
- 가드 두 제목(after vs base · `--inject` 27키): `49ade89578c5` PASS/PASS · `ddd885583b15` PASS/PASS(입력 27/27 둘 다)
- **통신사 코퍼스 smoke gate**(`WORKING_DIR=game_lab/working scripts/smoke_gate.sh` · after 바이너리 · boot+render): **294 PASS / 0 FAIL ·
  baseline 292 대비 퇴행 0 · rc 0**.
- 이 PR 의 동작 변경 중 기존 타이틀에 닿을 수 있는 것은 셋뿐이다 — `Class.getResourceAsStream` 폴백 · `createPlayer` 가 받는 형식 ·
  `getControl` 이 null 대신 컨트롤을 돌려줌. 나머지는 «없던 메서드·클래스» 추가라 그것을 부르던 타이틀은 전에는 죽었다.
- 코퍼스 이름 유입(`node scripts/corpus-name-inflow.mjs` · 이 브랜치의 바뀐 파일 전체): BOUNDED 6회/5쌍 · SUFFIX-ATTACHED 0 — 6회 전부
  이 브랜치 이전부터 있던 줄이다(`git diff origin/main...HEAD` 의 추가 줄에서 그 이름 0회). 이 회차가 들인 이름 = 0.

<!-- corpus-name-inflow v1 subjects=36 tree=2b90f7ab6e1d45ca B=6/5 P=0/0 S=0/0 -->
