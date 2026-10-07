## [2026-10-08] LGT 입력기 글자 조합 · SKT Graphics2D DRAW_AND/OR · captureLCD (wie-skt-lgt-ime-composition-drawimage-and-or-capturelcd)

**무엇을**: 0469 §6 의 후속 카드 두 장(`2026-10-07-progression-wave4-skt-lgt-followups#p0` · `#p1`)을 함께 진다. ① LGT `MC_imHandleInput`(0x130)이 숫자만 확정하던 것을 글자 조합 입력기로 바꿨다. `buf1` 은 이 키로 끝난 글, `buf2` 는 아직 조합 중인 글이다. 0x12e·0x12f 를 `MC_imSetCurrentMode`·`MC_imGetCurrentMode` 로 구현했다(종전 stub `unk7`·`unk6`). ② SKT `Graphics2D.drawImage` 의 `DRAW_AND`(1)·`DRAW_OR`(2)가 그리지 않고 돌아오던 것을 채널별 AND·OR 로 그린다. ③ SKT `Graphics2D.captureLCD` 가 빈 이미지를 주던 것을 화면 이미지의 사본으로 바꿨다. ④ 저부하 재측 3종의 진도를 다시 쟀다.
**왜**: 운영자 지시(미지원·부분지원 게임 완성도) · 채택 제안 두 건.
**사용자 영향**: `1cd151222bde` 의 이름 칸에 한글·영문 이름이 들어간다(종전 숫자만 · «가니» 로 확인 대화상자까지 확인). `d1dce4a36141` 의 시작 회사 로고 두 장이 검은 상자 대신 그려진다. 나머지는 §3 표.

타이틀은 sha12 로만 적는다. 프레임 PNG·키 레시피는 커밋하지 않았다(증적 `~/orchestrator/reports/evidence/wie-skt-lgt-ime-composition-drawimage-and-or-capturelcd/`).

### 1. LGT 입력기 — 두 버퍼의 뜻은 호출부에서 읽었다

`1cd151222bde` 이름 칸에서 0x130 의 귀환 주소(`lr = 0x179f`)를 진단 빌드로 잡아 호출부를 역어셈했다(Thumb · 바이트는 레포 밖).

- 호출: `MC_imHandleInput(key, 0x1f6, buf1 = sp+0x52c, &size1 = sp+0x20, buf2 = sp+0x524, &size2 = sp+0x1c)`. 호출 전 두 버퍼를 5바이트씩(워드 + 바이트) 0 으로 채우고 두 크기를 0 으로 둔다. 숫자 키만 이 호출에 온다(이 회차 진단 3회 · 0469 진단 5회 모두 `0x32`~`0x36`).
- 호출 뒤:
  - `size1 > 0` → `memcpy(칸 + 길이, buf1, size1)` · 길이 += size1 · 꼬리 0 · 전역 «보류 중» 플래그 = 0.
  - 플래그가 서 있고 · 이번 키가 보류된 키와 다르고 · `size2 > 0` 이면 → 보류해 둔 글을 게임이 **스스로** 칸에 붙인다.
  - `size2 > 0` → `buf2` 를 칸 길이 **뒤에** 복사한다(길이는 늘리지 않는다 — 보이기만 한다). 그리고 «보류 중» = 1 · 보류 키 = 이번 키 · 보류 글 = buf2 를 전역에 둔다.
- ⇒ `buf1` = 끝난 글(붙인다), `buf2` = 조합 중인 글(보여 주고 다음 호출이 다시 쓴다). 게임이 스스로 붙이는 경로는 `size1 == 0` 일 때만 탄다(`size1 > 0` 이 플래그를 먼저 내린다). 그래서 «키가 바뀌면 지난 글을 buf1 로 돌려준다» 로 구현하면 이중으로 붙지 않는다.
- 모드: 이 타이틀은 화면에 들어설 때마다 `0x12e(2)` 를 부른다. 2 는 우리 0x12d 표(`EN/S · EN/L · KO · N123`)의 **KO** 다. 화면 오른쪽의 모드 표시도 «가» 다. 종전에는 0x12e 를 무시하고 숫자만 확정했다.

구현(`wie-lgt/src/runtime/wipi_c.rs`):

| 항목 | 고른 값 | 근거 |
|---|---|---|
| buf1 / buf2 | 끝난 글 / 조합 중인 글 · EUC-KR | 위 호출부(**실측**) |
| 자판 | `wie_util::keypad`(영문 E.161 멀티탭 · 천지인 한글) | lwc·lcdui·SKVM 입력기와 같은 표(0428) |
| 다시 누름 창 | 1초 | SKVM `TextComponentHandler`·lwc 와 같은 값 |
| 모드 번호 → 자판 | `EN/S` 소문자 · `EN/L` 대문자 · `KO` 한글 · `N123` 숫자 | 이름은 0x12d 표 그대로. 번호 순서는 종전과 같은 **가정**(0469) |
| N123 | 숫자를 바로 끝낸다(buf2 비움) | 종전 동작 그대로 |
| 0x12e | 모드를 바꾸고 조합을 끝낸다 | 이 타이틀은 화면마다 부른다 — 새 칸 = 새 조합 |
| 상태 | 게스트 루트 페이지 `0x7fff1020`(`TIME_VALUE_PTR` 와 같은 페이지) | 한 프로세스의 두 에뮬레이터가 따로 갖는다 · 0 = «EN/S · 조합 없음» = 종전 0x12f 응답 |
| `ᆢ`(ㆍ 두 번) | `‥` 로 보낸다 | EUC-KR 에 코드가 없다(**가정** — 실기 표시는 모른다) |
| CLR · `*` | 다루지 않는다 | 이 호출부에 오지 않는다(숫자 키만) |

- 0x12e 를 부르지 않는 타이틀은 0 = `EN/S`(소문자)로 시작한다. 종전에는 숫자만 들어갔다. 이 회차 프로브 A 30종(§4) 중 판정이 바뀐 것은 없다.

### 2. 측정 조건

| 엔진 | sha | 쓴 곳 |
|---|---|---|
| main | `0bacaa19` (`wv_base`) | 전/후의 «전» |
| 이 브랜치 | `c71ff066` 과 같은 코드(`wv_fix`) | 전/후의 «후» |

- 임대: 측정 스윕은 `build-slot run --long` 임대 네 번(§7) · `--jobs 2` · census 는 **사본 + 개인 잠금 경로**(`WIE_CENSUS_LOCK`). 기본 잠금은 다른 레인의 census 가 쥐고 있었다(0469 와 같은 처지). 사본은 진도 단계에서 «longplay 판정이 없어도 목록의 타이틀은 잰다» 한 줄만 바꿨다(이 회차는 long 축을 다시 재지 않았다).
- 단발 진단(15~85초 · 프레임 0.25~0.5초 간격)은 short 임대다.
- 프로브 A·B 는 전·후를 **같은 시각에 돌리지 않았다**(전 01:17~01:32 · 후 01:32~01:47 · load1 18~32). 그래서 paints 차는 판정에 쓰지 않는다.

### 3. 대상별 전/후

진도 P(census `--only progress --progress 600` · 정책 v2 · 재기동 120초 · 판정 = 마지막 새 화면 뒤 정체가 200초 이상이면 막힘). «레시피» 열은 정책 앞에 붙인 접두다(`game_lab/recipes-progress/` · 비커밋). 걷기 레시피 3종은 0469 의 «진행 요령» 측정과 같은 키다 — compat 의 진도 축은 정책 판정이라 이 값으로 바꾸지 않는다. 6축(boot·render·input·longplay·sound·speed)은 다시 재지 않았다(프로브 A·B 는 §4 · 바뀐 것 0).

| sha12 | 대상 | 레시피 | 진도 P 전 | 진도 P 후 | 무엇이 보였나 |
|---|---|---|---|---|---|
| `1cd151222bde` | 입력기 | 이름 칸까지 OK 13회 | stuck(정체 440초) | stuck(정체 230초) | 둘 다 이름 칸을 지나 종목 선택 → «레전드 선수 생성» 에서 정책이 맴돈다. 이름이 전 «5» · 후 «ㄴ»(정책 키 `NUM5` · KO 모드). 막힌 자리는 이름 칸이 아니다 — ⒜ 정책 한계 |
| `c107462e5f8a` | AND | 걷기 | ok(170) | ok(170) | 같음 |
| `d1dce4a36141` | OR·AND | 걷기 | ok(40) | ok(40) | 같음 |
| `f12984cd0d37` | AND·OR(0469 stub 집계) | — | (재지 않음 · compat stuck) | stuck(470) | 본편 첫 대화 뒤 장비·능력치 화면에서 정책이 맴돈다 |
| `ec2f8f2e02a2` | 캡처 | — | (재지 않음 · compat stuck) | stuck(350) | 시작 메뉴 → «환경설정» 화면에서 정책이 맴돈다 |
| `090877d7a3e0` | 캡처 | — | (재지 않음 · compat stuck) | stuck(520) | «이어하기» 빈 저장 칸 목록에서 정책이 맴돈다 |
| `7089dec0e8df` | 캡처 | 걷기 | stuck(220) · 예외 1 · 재기동 1 | ok(100) · 예외 0 | 전 P 는 420초쯤 `NullPointerException` 1회 뒤 검은 화면. **짝 P2 는 전 ok(140) · 후 ok(70)** — 전의 막힘은 재현되지 않았다. 이 수정의 효과로 보지 않는다 |

저부하 재측 3종(0469 에서 load1 168~173 에 잰 값 · 이 회차 load1 9~14 · `host-load-guard --status --recovered` rc=0 뒤 · 이 브랜치 엔진 · 레시피 없음 · P 가 stuck 이면 P2 짝):

| sha12 | P | P2 | 판정(census 규칙: P stuck 은 P2 가 정한다) | compat |
|---|---|---|---|---|
| `be08d047cbae` | stuck(260) | **ok(30)** | ok | progress `stuck → ok` |
| `af7d82e5e239` | stuck(380) | stuck(370) | stuck | 그대로 |
| `bf54c05e58a9` | stuck(320) | stuck(320) | stuck | 그대로 |

- 전(P)은 «후 P 가 ok 이거나 입력기 대상» 인 4종만 같은 키로 쟀다. 나머지 3종은 후가 compat 의 기존 값과 같아 전을 재지 않았다.

그림 전/후(180초 · 5초 간격 프레임 · 전·후 같은 키 · 같은 시각에 나란히):

| sha12 | 호출(전 · 180초) | 프레임 비교 | 진도와의 관계 |
|---|---|---|---|
| `c107462e5f8a` | `DRAW_AND` 71회(15~86초) | t005~t070 **0 px** 차. t075 부터는 걷는 길이 갈라진다(전·후 모두 주인공 잔상 — 이 수정과 무관) | 보이는 변화 없음 |
| `d1dce4a36141` | `DRAW_OR` 99회(시작 0.1초) · `DRAW_AND` 5회(13초) · `captureLCD` 1회 | 시작 회사 로고 두 장: 전 = **검은 상자** · 후 = 로고(0.5초 간격 16초 · t0.5~t3.0 에 3,563~5,040 px 차) | 시작 화면뿐 |
| `f12984cd0d37` | 이 창에서 0회 | 커서 깜빡임 위상 차뿐 | 관측 못 함(정책 키 180초 안에서 호출 0) |
| `ec2f8f2e02a2` | `captureLCD(0,0,240,336)` 2회(1.5·2.8초) | 메뉴 진입(3초)부터 메뉴 뒤 어두운 배경의 로고 자리 1,122 px 가 다르다(전 = 로고·장식, 후 = 없음) | 같은 메뉴 · 같은 키 반응. 어느 쪽이 실기와 같은지는 근거가 없다 |
| `090877d7a3e0` | `captureLCD` 12×13 영역 78회 | 36장(5초 간격) · 60장(0.25초 간격 15초) **전부 0 px** | 보이는 변화 없음 |
| `7089dec0e8df` | `captureLCD` 12×13 영역 130회 | 36장 **전부 0 px** | 보이는 변화 없음 |

- 12×13 영역 캡처 두 타이틀은 커서 크기의 배경을 떠 두는 쓰임으로 보인다. 빈 이미지여도 그 자리가 곧 다시 그려져 프레임에 남지 않는다.

### 4. 퇴행 — 프로브 A 30종(census 인자 30초)

라이브 LGT 5(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) · 가드 2(`49ade89578c5` `ddd885583b15`) · 대상 10 · LGT 입력기 모드 이름을 담은 타이틀 13(`binary.mod` 에 `N123`/`EN/S`/`EN/L` 문자열 · `5dc5d7091c01` `580a66c32fff` `0266ca417880` `236c7da689f6` `9acb5e0387eb` `595a443e1c16` `0093012b8c36` `fd8f52f3abf7` `63332c51d514` `3ff5948e235e` `b7699c10dfd1` `2d5cada03004` `87b04639cdfe`).

- **A: PASS · content · 키 27/27 · 예외 수 — 30/30 같음.** B(키 없음): 30/30 PASS → 30/30 PASS.
- paints 는 전·후가 다른 시각이라 비교하지 않는다(§2). 가장 크게 움직인 것: `0266ca417880` 184 → 758 · `49ade89578c5`(KTF · 이 회차 코드가 닿지 않는다) 1,103 → 1,739.

### 5. 되돌리면 red

- `wipic_im_handle_input_commits_finished_text_and_composes_the_rest` — 0x130 본문만 종전(숫자만)으로 되돌린 트리에서 FAILED(EN/L 첫 단언).
- `graphics_2d_and_or_modes_combine_with_the_target` · `graphics_2d_capture_lcd_copies_the_screen` — AND/OR 조기 반환·캡처 생략만 되돌린 트리에서 둘 다 FAILED(`--no-fail-fast`).

### 6. compat · 소식

- `docs/player-data/compat.json` 두 행만 바꿨다(`node scripts/check-compat-revert.mjs --base origin/main --head HEAD` → 아래 §7).
  - `1cd151222bde`: 안내 «이름 칸에는 아직 숫자만 들어가요.» 삭제(census `HAND_NOTE` 에서도 삭제). 진도 축은 stuck 그대로(§3).
  - `be08d047cbae`: progress `stuck → ok`(저부하 P·P2 짝 · §3). 이 회차 코드와 무관한 재측이다.
- SKT 6종은 compat 를 바꾸지 않았다 — 진도가 그대로다. 그림이 바뀐 `d1dce4a36141` 도 시작 로고뿐이라 안내 줄을 달지 않았다.
- `docs/player-updates/2026-10-08-lgt-name-letters.json`(kind `fix` · `1cd151222bde`) 1개.

### 7. 게이트

- `cargo fmt --check` · `cargo clippy --all -D warnings` · wasm32 · `+beta` 전부 rc=0. `RUST_MIN_STACK=4194304 cargo test --all` **736 passed / 0 failed**. `npm run build:wasm` rc=0 · `check-engine-contract` 113 pass / 0 · `npm run audit` PASSED · census selftest 62/62 · `player-data` OK · `check-worklog-json` OK.
- 러너 블록: draw · helloworld ×2 · keydraw ×2(`--inject --expect-last-frame` · rc=0 · paints 79 / 55) · text 전부 PASS.
- 회차가 띄운 프로세스: 회차 끝에 `wv_base`·`wv_fix`·census 0(`pgrep` 03:58).
- 측정 임대: `build-slot run --long` 4회(00:59~01:47 · 01:49~03:20 · 03:20~03:44 · 03:44~03:56) · 각 `--jobs ≤ 2` · 착수 전 `host-load-guard --status --recovered` rc=0(두 번째 임대는 rc=0 이 될 때까지 7분 대기) · `nohup` 0.
