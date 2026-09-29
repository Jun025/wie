## [2026-09-29] 「바로 해보기」를 게임 하나로 — 몽글셋을 동물 얼굴 3매치 「동물줄맞춤」으로 개편 (wie-featurephone-animal-match-remake)

**무엇을**: `demo/` 에 **동물줄맞춤**(`demo/dongmul-julmatchum/` · 옛 `monggeul-set` 을 `git mv`)과 `arcade-common` 만 남겼다.
삭제 8종: `pebble-snake` · `ssokssok-mole` · `gopjeol-tile` · `bamsongi-field` · `chagok-stack` · `bbokbbok-balloon` ·
`tongtong-pebble` · `jureong-berry`. 틀(`Arcade.java`)은 전용 홈 · 설정 · 그림 도움말 · 최고 점수 · 기록 초기화 · CLR 뒤로 ·
시간 막대를 갖게 됐고, 동물 얼굴은 빌드 때 `art/Faces.java` 가 그린 PNG 다. Rust 변경 0.

**왜**: 운영자 지시(2026-09-29) — 「몽글셋 제외하고는 모두 삭제 · 동물얼굴로 · 화질 개선 · "뒤로가기"로 갈 수 있는 메인화면 ·
첫 진입 시 전용 메인 · 설정 및 도움말 · 시간초가 아니라 막대 · 이름 변경」.

**사용자 영향**: 이 PR 만으로는 없다 — 셸에서 나머지 데모를 걷고 이 게임 하나만 싣는 것은
`otterpebble-featurephone-demo-single-animal-match`(이 PR 머지 뒤) 몫. 기존 타이틀 동작 변경 0 이라 `docs/player-updates/` 항목 없음.

### 규칙(유지한 것 · 바꾼 것)
7x7 · 옆 칸과 교환 · 가로/세로 셋 이상 → 터짐 · 떨어져서 또 맞으면 연쇄(점수 = 10 x 개수 x 연쇄 단계) — **그대로**.
바꾼 것: 난이도 두 칸 — 쉬움 = 동물 다섯 · 90초, 보통 = 동물 여섯 · 60초(옛 몽글셋은 다섯 · 60초 하나). 여섯째 동물이 생겨
보통의 판이 옛날보다 어렵기 때문에, 옛 난이도에 가까운 것을 「쉬움」으로 남기고 시간을 늘렸다. 최고 점수는 난이도별.

### 화질 — 무엇이 제한하는가(측정) → 고른 수단
| 요인 | 측정 | 게임이 바꿀 수 있나 |
|---|---|---|
| 화면 해상도 | 240x320 고정 — 셸 `apps/featurephone/lib/engine.ts` `SCREEN_W/H` · `wie_validate` `SCREEN_W = 240` | 없음(J2ME 에 크기 지정 경로 없음) |
| 도형 API | `Graphics.fillArc` 등 안티앨리어싱 없음 — 옛 판 보드 영역 **8색**, 원 둘레 4곳에 1px 돌기 | 없음(엔진) |
| 셸 확대 | `player.tsx` `useCrispMirror` — 정수배(ceil) 최근접 → 쌍선형(low) 축소 | 없음(셸) |
| 이미지 | `Image.createImage(String)`·PNG 디코드(`image` crate png)·`drawImage` **알파 섞기**(`blend_pixel`) 실재 | **있음** |

⇒ ⒝ 채택: 얼굴을 빌드 때 Java2D 안티앨리어싱으로 그린 **알파 가장자리 PNG**(30px 보드용 · 56px 홈/도움말용, 동물 6종 x 2 = 12장 ·
합 17,654B). 같은 장면 보드 영역 색 수 **8 → 933**. ⒜는 여지가 없고(이미 최대), ⒞(이중선)는 ⒝가 덮는다.

### CLR 키 코드(측정)
탐침 MIDlet(`keyPressed` 값과 `getGameAction` 을 화면에 찍기)을 release `wie_validate --keys "CLR"` 로 돌려
**`keyPressed(8)` · `getGameAction(8) = 0`**(wie-midp `MIDPKeyCode::CLEAR = 8`). 같은 탐침에서 5 는 `53` · 게임 액션 0 이라
숫자키를 직접 접는 틀의 코드가 필요하다. 틀은 `8` 과 `-8`(다른 단말 관례)을 둘 다 뒤로로 받는다.

### 이름 — 동물줄맞춤
KIPRIS 상표(9·28·41류 필터 · 2026-09-29): **동물줄맞춤 0건**(구성어 「줄맞춤」 6건 중 해당 류는 무관한 GUARDIAN 1건).
탈락: 몽글동물(「몽글」 09류 등록 2020 · 28류 등록 EBS) · 냥멍셋(「냥냥멍멍」 28류 등록 · 「멍냥」 09류 등록) ·
나란얼굴(「나란」 09·28류 출원 2026-08-27) · 동물모아(동일 28류 등록) · 셋셋동물(「셋셋셋」 28류 등록) ·
동물잇기(0건이나 사천성/「Connect Animal」 장르 연상).

### 결과(release `wie_validate` @ 이 브랜치 · openjdk 17.0.20.1)
| 바이트 | sha256(2회 동일) | `--timeout 1` | `playthrough.keys --inject` | fps |
|---|---|---|---|---|
| 34,956 | `08374aa901132604fb7f4ff5c6490957eb84de89d8f221d4b90f8cffe911c56e` | PASS · 첫 화면 = 홈 · 예외 0 | PASS 102/102 · 예외 0 · 소리 40 · 홈→설정→도움말→시작→CLR→예→홈→시작→시간 끝→CLR→홈 | 플레이 ≈8.7(설계 10 · load 369) · 홈 ≈5(설계 5) |

- 실브라우저 `WIE_BASE=https://wie-web.pages.dev node scripts/verify-browser.mjs demo/dongmul-julmatchum/out/dongmul-julmatchum.jar` →
  rc 0 · 콘솔 에러 0 · off-origin 0 · 게임 헤더 바이트 요청 0 · 홈 화면 렌더(PNG 얼굴 포함).
- 초 숫자: 소스의 화면 문자열에 초 0(「초」는 「기록 초기화」의 「초」뿐) · 남은 시간은 `leftMs` 로 막대 폭만 정한다.
- 진동: 엔진 `Display.vibrate` 실재(브라우저 `navigator.vibrate`, 없으면 조용히 무시) ⇒ 설정에 넣었다. `wie_validate` 는 no-op 라
  진동 자체는 재지 않았다.
