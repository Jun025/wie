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

### 3. 채택분 6축 + 진도

CENSUS_TABLE

### 4. 남은 벽 — 고치지 않은 것과 이유

| 게임 | 벽 | 판정 |
|---|---|---|
| finifactory (MIT) | `javax.xml.parsers.SAXParserFactory` — JSR-172 | 범위 밖: no_std 엔진에 XML 파서를 새로 들여야 한다(`quick-xml` 은 std). 한 게임 |
| kurve (GPL) | 생성자에서 `javax.bluetooth.LocalDevice` | 범위 밖: 블루투스 2인 대전 전용 — 스텁을 넣어도 «블루투스를 켜라» 경고 뒤 종료가 원작 동작 |
| bubblet-asha (GPL) | `System.getProperty("com.nokia.keyboard.type").equals(…)` | 범위 밖: Nokia Asha 전용(+JSR-211). 우리가 Nokia 로 자처하면 다른 게임이 Nokia 전용 API 로 간다 |
| loveme (MIT-0) | 고친 뒤 startApp 이 Lua 를 해석하며 **한 틱 안에서 30 s 넘게** 양보하지 않는다 — `wie_validate --timeout 12` 가 반환하지 않고 7분 넘게 CPU 를 썼다(sample: 바이트코드 해석기 루프) | 제안(#p1): 브라우저 호스트도 같은 시간 동안 멈춘다 |
| wipi_game (MIT · KTF/LGT) | 로딩 화면이 검정 단색(KTF 11 · LGT 10 paints). SDK 가 `fgpxl` 에 ARGB32 를 그대로 넣는 것은 확인(16bpp 화면에서는 색이 틀어진다) — **단색 검정의 원인은 아니다**(흰 글자도 안 보인다) | 미규명 — 이 회차 시간 상한. WIPI-C 경로라 통신사 코퍼스와 같은 코드를 지난다 |

**입력 축의 공통 벽**(제안 #p0): 노키아·소니에릭슨용 게임은 방향키 -1~-4 · 확인 -5 · 소프트키 -6/-7 을
기다리는데 wie 는 SKT 값(위 141 · 소프트 6/7)을 보낸다. sperm-race 캐릭터 화면 실측 — RIGHT 두 번의 캡처는
직전 프레임과 같은 해시, 같은 자리의 NUM6 두 번은 새 화면 2장. J2ME 만 표준값으로 바꾸면 우리 데모
(`demo/arcade-common`)가 -1 을 종료키로 읽어 위쪽 키가 종료 확인을 띄우므로 **데모와 한 PR 로** 가야 한다.

### 5. 퇴행

- 4게이트: fmt ✅ · clippy `-D warnings` stable ✅ · wasm32 ✅ · beta ✅ · `RUST_MIN_STACK=4194304 cargo test --all` 실패 0 ✅
- 러너 블록(AGENTS §The four gates): draw_j2me · helloworld_ktf/lgt PASS · keydraw_ktf PASS · keydraw_lgt PASS(rc 0) — 첫 1회는
  UNMEASURED(load1 69) 였고 같은 부하에서 base·after 재실행이 둘 다 PASS 55 paints · text_j2me PASS
- 가드 두 제목(after vs base · `--inject` 27키): `49ade89578c5` PASS/PASS · `ddd885583b15` PASS/PASS(입력 27/27 둘 다)
- SMOKE_LINE
