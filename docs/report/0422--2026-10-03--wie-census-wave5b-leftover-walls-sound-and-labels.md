## [2026-10-03] 5b차 — WIPI `setRGBPixels` 불투명 · 거절 문구를 그리고 끝나는 구매 단말 검사 · 남은 벽 셋 실측 (wie-census-wave5b-leftover-walls-sound-and-labels)

**무엇을**: wave5·r3 가 남긴 벽 중 주인 없는 넷(티켓 4·5·6·7항)을 쟀다. 1·2항은 wave6(#453), 3항은 #449 가 맡아 이 회차는 손대지 않았다.
**판정**: 5항 `a540945188ca` «글자 없음»은 **고쳤다**. 원인은 WIPI `Graphics.setRGBPixels` 였고, 같은 원인을 가진 KTF 4종이 함께 고쳐졌다(§1). 6항(설치 토큰 잠김 표기)도 **고쳤다**(§2). 4항 `38277d63b0ba` 는 #448 이 **이미 고쳤다**(§3). 5항 `d1e0badfce82` AIOOBE 는 **원인을 쟀고 고치지 않았다**. KTF 화면 높이 모델을 바꿔야 하는 일이다(§4). 7항은 **쟀다**(§5).
**사용자 영향**: KTF 게임 5종에서 안 보이던 글자·메뉴·그림이 보인다. 한 종은 첫 화면부터 빈 판이었다. 구매 단말 검사에 막히는 SKT 게임 1종은 «플레이 가능»으로 잘못 나오던 것을 «잠긴 파일» 안내로 바꿨다.

증적: `~/orchestrator/reports/evidence/wie-census-wave5b-leftover-walls-sound-and-labels/`. 타이틀은 sha12 로만 적는다.

### 1. `a540945188ca` 글자 없음 → WIPI `setRGBPixels` 가 모든 화소를 투명하게 그렸다

- 실측(디버그 로그 30초): 그리기 호출의 대부분이 `org.kwis.msp.lcdui.Graphics.setRGBPixels` 였다(30초에 49만 회 · 1×1 화소 단위 글꼴). 게임은 로고·메뉴·글자를 전부 이것으로 찍는다.
- 원인: WIPI 의 화소는 알파 없는 `0x00RRGGBB` 다. 첫 구현(`840014d9`)은 이를 불투명 RGB 로 읽었다. upstream 의 MIDP 이관(`15ec4c09`)이 `drawRGB(…, processAlpha=true)` 로 넘기면서 알파 0 → **완전 투명**이 됐다. upstream main 도 같은 상태다.
- 처방: `processAlpha=false`(`wie-wipi-java/src/classes/org/kwis/msp/lcdui/graphics.rs`). 시험 `test_set_rgb_pixels_draws_alpha_less_pixels_opaque` — `0x00123456` 을 찍고 `getPixel` 이 그 값이어야 한다. **되돌리면 red**(실측: `left: 0 · right: 1193046`).
- 코퍼스에서 `setRGBPixels` 문자열을 가진 9종을 전·후 짝으로 쟀다(같은 27키 · 30초 · `wie_validate` 1개씩 · 증적 `setrgbpixels-pair.txt`).
  - **글자가 나타남 5종**: `a540945188ca`(색 3 → 181 · 로고·메뉴·도움말) · `2c2ba3b84b98`(메뉴 글자) · `65bace1623a7` · `9c1c446a36e2`(「EMPTY · Play Time」) · `1b3b4868d46e`(메뉴).
  - **변화 없음 4종**: `f80713702111` · `d9afc4db742c` · `5891a0c5d595` · `44c292be4e4c`(결과·색 수 같음).
  - `2c2ba3b84b98` 후 실행은 `clean exit` 였다. 고정 27키가 이제 보이는 메뉴의 «나가기»를 고른 것이다(증적 `2c2b-after.png`). 전에는 같은 키가 빈 화면에서 아무것도 고르지 못했다. 엔진 퇴행이 아니라 키 일정의 결과다.

### 2. 6항 — 거절 문구를 그리고 스스로 끝나는 구매 단말 검사도 «잠긴 파일»

- r3 §5 의 `44b6356d13f8` 은 **이미 정적 판별(XCE `SecureUtil` 3속성)에 걸린다**. 놓친 것은 실행 절반이었다. 종전 규칙은 «0 paints 로 끝남»이었는데, 이 게임은 검사가 실패하면 「인증 되지 않은 컨텐츠」 상자를 **그린 뒤** `System.exit` 한다(javap: `q` 에서 `install.dat` 이 없으면 `a.a(MIDlet)` → 그리기 → `System.exit`).
- 새 규칙(`scripts/playability-census.mjs` `quitOnItsOwn`): 모든 실행이 «스스로 끝났거나 그리지 않았다» + 하나 이상 `clean exit`. 키 없는 실행도 끝났다는 뜻이다.
- bd2337ff 전수에서 그 클래스를 가진 41종에 적용하면 **2종만 바뀐다**: `44b6356d13f8`(A·B 둘 다 `clean exit` · paints 7·8) · `eefc947d8337`(1·1 · 이미 잠김 안내가 있던 행). 플레이되는 36종은 전부 `deadline` 으로 끝나 걸리지 않는다. `44b6356d13f8` 의 거절 문구도 `eefc947d8337` 의 `SecureUtil` 문구와 같다(javap).
- 잠긴 파일의 `status` 는 이제 `not-yet` 을 넘지 않는다. 그리는 검사가 축을 전부 `ok` 로 만들어 «플레이 가능»으로 읽혔기 때문이다.
- selftest 2행: «계속 도는 게임은 잠금 아님» · «거절을 그리고 끝나면 잠금». **되돌리면 red**(종전 술어면 둘째 행이 진다). 52/52. 인증 우회는 없다 — 판별·표기만 했다.

### 3. 4항 `38277d63b0ba` — #448 로 이미 해소 · stand down

r3 는 `890ae6e1` 에서 쟀다. 그 커밋에는 #448(`ce469d81` · `setCurrent` 가 `showNotify` 를 부름 · 바로 이 타이틀을 주석에 적었다)이 들어 있지 않다. `origin/main` 빌드로 30초를 돌리면 `img is null` 이 0회이고, paints 2013 · 결정 화면 → 대화로 간다(증적 `38277-main-plays.png`). 진도 축 600초도 `ok` 다(§5).

### 4. `d1e0badfce82` AIOOBE — 원인은 «화면 높이 320», 고치려면 KTF 화면 높이 모델이 필요하다

- 실측(임시 계측 · 커밋 안 함): `java_throw` 의 첫 인자 `0x1453f8` 은 **예외 클래스 이름 문자열**이다. r3 가 적은 «pc» 가 아니다. 실제로 던진 자리는 스택에 남은 복귀 주소 `0x1253c1` → 배열 원소 읽기 도우미 `0x139ca0`(경계 검사)이다.
- 게임 코드(`BlueMarbleCard.paint`): 배경 띠 수를 `X = getHeight()/10 + 1` 로 정하고 고정 크기 표 `int[5][32]` 를 `for (i < X) A[B][i]` 로 읽는다. 높이 320 이면 X=33 → 인덱스 32 에서 AIOOBE → paint 가 끝나 판 위가 그려지지 않는다(매 paint).
  - 계측값: `X` 필드 = 33 · `getHeight()` 필드 = 320 · 배열 길이 32 · 인덱스 32.
- 화면 높이만 300 으로 바꾼 임시 빌드: AIOOBE **0회** · 판·말·정보창이 그려진다(증적 `d1e0-h300-board.png`). ⇒ 높이 ≤ 319 면 넘는다.
- **고치지 않은 이유**: ADF 가 `DisplaySize:240*320` 을 선언하고, 코퍼스 KTF 의 240폭 177종이 전부 같은 값이다. 실기가 상태 표시줄을 빼고 더 작은 Card 높이를 줬다는 것이 가장 그럴듯하지만, 그 값(LGT upstream 의 240폭 표시줄은 24)을 KTF 에서 잰 증거가 없다. 높이를 바꾸면 177종 전부의 배치가 바뀐다. 근거 없이 하지 않았다. 후속 p0(크기 M · 177종 짝 전수가 근거다).

### 5. 7항 — 진도 축 재측(600초 · 정책 v2 · census `--only progress` · `--jobs 1`)

| sha12 | 전 | 후 | 비고 |
|---|---|---|---|
| `b475b6399684` | stuck | **ok**(stall 30 · 새 화면 41) | 레시피(`game_lab/recipes-progress/` · 비커밋 · OK 12회로 설명서 → 스킨판 → 게임) |
| `78bd51675574` | stuck | **ok**(stall 150 · 11) | 120×160 임시 빌드(r3 화면 규칙이 고르는 크기 · `wie_validate` 화면만 바꿈) |
| `a540945188ca` | stuck | **ok**(stall 0 · 11) | 이 브랜치(§1) |
| `38277d63b0ba` | (없음) | **ok**(stall 70 · 10) | `origin/main` 빌드 |

`ok` 는 짝 재측이 필요 없다(`stuck` 만 P2 를 요구한다). 다만 1800초 기본이 아니라 **600초 창**이다(r3 와 같은 창).

### 6. `compat.json`(행 단위 · 손으로)

- `44b6356d13f8`: status playable → **not-yet** · 안내 = 잠긴 파일(§2 규칙이 다시 만들 값과 같다).
- `a540945188ca` · `b475b6399684` · `78bd51675574`: `progress` stuck → **ok**. `38277d63b0ba`: `progress` **ok** 추가.
- 다른 축은 손대지 않았다(글자가 생긴 4종은 원래 6축 `ok` 였다 — 화면 내용은 그 축이 못 본다). `docs/player-updates/2026-10-03-wipi-text-visible.json` 이 5종의 소식을 만든다.

### 7. 퇴행

- `RUST_MIN_STACK=4194304 cargo test --all` **660 pass / 0 fail** · fmt · clippy stable·wasm32·beta `-D warnings` · `npm run build:wasm` rc0 · `check-engine-contract` OK · `npm run audit` PASSED · census selftest 52/52 · `player-data` OK.
- 러너 블록: draw · helloworld ×2 · text PASS. keydraw ×2 는 기본 tick 상한에서 `UNMEASURED`(stop `max-ticks`)였다. **`origin/main` 빌드도 똑같았다**(5·6 키에서 멈춤). `--max-ticks 100000000000` 이면 둘 다 PASS · rc=0 · content true.
- 라이브 LGT 5 + 가드 2(27키 · 30초 · `origin/main` `71d22a8b` 빌드 ↔ 이 브랜치): 결과·`content`·예외 수 전부 같다. paints 만 부하에 따라 흔들렸다(`49ade89578c5` 2017 → 1356 · 이 타이틀은 `setRGBPixels` 를 갖지 않는다). `4ece6eeeaa04` 는 양쪽 다 `clean exit` 이다(기존). 증적 `lgt-guard-pair.txt`.
- 측정 규율: 에뮬레이터 실행은 전부 `build-slot run` 으로 1개씩 돌렸다. `host-load-guard` 는 rc=1(saturated · holding)이어서 폭을 1로 뒀다. census 는 호스트 락이 빈 뒤 `--jobs 1`. 내가 띄운 프로세스는 0 이다.

### 8. 후속

| 군집 | 수 | 계급 | 크기 |
|---|---|---|---|
| KTF Card 높이(상태 표시줄) 모델 — `d1e0badfce82` | 1(+240×320 KTF 177종의 배치) | ⒝ · worklog p0 | M(짝 전수가 근거) |

### 9. 유입

`node scripts/corpus-name-inflow.mjs --corpus <로컬 코퍼스>`: BOUNDED 718회 / 330쌍 · SUFFIX-ATTACHED 35회 / 15쌍. 전부 `docs/player-data/compat.json` 의 `title`·`fileTitle` 이다. 이 회차가 바꾼 5행에 원래 있던 표시 이름이 diff 맥락으로 잡힌 것이고, 이 파일은 설계상 이름을 갖는다. 회차 문서·worklog·소식·코드에 더한 게임 이름은 0이다(sha12 만).

<!-- corpus-name-inflow v1 subjects=6 tree=1eb1030d104c8e7c B=718/330 P=0/0 S=35/15 -->
