## [2026-10-08] LGT 입력기 — 영문 멀티탭 · 천지인 한글 조합 · 모드 설정/조회 (wie-lgt-input-method-letter-composition-multitap-hangul)

**무엇을**: LGT WIPI-C 입력기 3개를 붙였다. `MC_imHandleInput`(0x130)은 이제 숫자만 확정하지 않는다. 같은 키 반복으로 영문 글자를 고르고(멀티탭), 천지인으로 한글을 조합한다. 결과는 «확정»(buf1)과 «조합 중»(buf2)으로 나눠 EUC-KR 로 돌려준다. 0x12e·0x12f 는 `Unk7`·`Unk6` 스텁이었다. 이제 `MC_imSetCurrentMode`·`MC_imGetCurrentMode` 로 모드를 기억한다. 조합 규칙은 이미 있던 `wie_util::keypad` 를 그대로 썼다. lwc·SKVM 입력기와 같은 규칙이다. 멀티탭 창(1000ms)은 세 곳에 따로 있던 상수를 `keypad::MULTITAP_MS` 하나로 모았다.
**왜**: #502 는 숫자 확정만 넣었다. 그래서 `1cd151222bde` 의 이름 칸에는 숫자만 들어갔다(안내 「이름 칸에는 아직 숫자만 들어가요」). worklog `2026-10-07-progression-wave4-skt-lgt-followups#p0` 채택.
**사용자 영향**: `1cd151222bde` 의 이름 칸에 한글(«가나»)과 영문(«BFWM»)이 들어간다. 게임이 그 이름을 받아 «이름 : 가나 · 국가 : GHA» 확인창으로 넘어간다. 위아래 키로 한글·영문·숫자 모드가 바뀐다(게임이 0x12e 를 부른다). 그 타이틀의 «숫자만» 안내는 지웠다. 진도 축은 `stuck` → `ok` 로 고쳤다. 단 이 값은 숫자 이름만으로도 이미 `ok` 였다(아래 §3). 이번 변경이 진도 축을 옮긴 것은 아니다.

타이틀은 sha12 로만 적는다. 키 레시피·화면은 커밋하지 않았다(`/tmp` 사본).

### 1. 측정 조건

| 엔진 | sha | 쓴 곳 |
|---|---|---|
| main | `76423d05` | 전/후의 «전» |
| 이 브랜치 | `a421b0d4`(위 base 위 · 그 뒤 `origin/main` 으로 rebase) | 전/후의 «후» · 입력기 호출 기록 |

- census(사본 아님 · 저장소 `scripts/playability-census.mjs`)를 쓰고 코퍼스는 `/tmp` 의 심볼릭 링크 디렉터리로 줬다. 호출 타이틀 8은 6축 전부(probe A·B · long 600초 · speed)를 쟀다. 라이브 LGT 5 + 가드 2 는 probe A·B 만 쟀다. 진도 P(600초 · 정책 v2)는 호출 타이틀 중 `playable` 6종만 쟀다. `01f05f8231f4`(잠긴 파일 · `not-yet`)와 `4fdbd64c9fbd`(`limited` · long `error`)는 뺐다.
- 전체를 `build-slot run --long` 임대 하나 안에서 «후 → 전» 순서로 돌렸다(`--jobs 2`). `host-load-guard --status --recovered` 는 착수 시점에 rc=1(`saturated` → `recovering`)이어서 폭을 2로 묶었다. load1 은 8~33 이다(아래 표 열).
- ★**기다린 시간(형제 레인과 나눠 썼다)**: long 임대 대기 1회차 8분(23:37 → 23:45) · 가드 대기 15분 · census 잠금 대기 7분(r4 레인 `probeB`). 그 회차는 내 실수(`--bin` 상대 경로 → `ENOENT`)로 결과 없이 끝났다. 2회차는 임대 37초 · 가드 2분 · 잠금 4분(r4 `longA` 종료 00:22:47)이다. 그 뒤 측정 본체가 2시간 34분 걸렸다(00:22 → 02:56). 입력기 호출 기록 스윕(아래 §2)은 long 임대를 23분 기다렸다(03:02 → 03:25 · 두 칸 모두 #507 레인이 쥐고 있었다).
- 단발 진단(레시피로 이름 칸까지 가서 글자 넣기 · 30~90초)은 short 임대에서 돌렸다.

### 2. 게스트 쪽 의미 — `1cd151222bde` 호출부에서 읽었다

진단 빌드(호출부 lr 과 그 앞 0x300 바이트를 찍는 1회용 패치 · 커밋 안 함)와 `binary.mod` ELF 정적 역어셈(capstone)으로 읽었다. 글 칸 객체는 `text = obj+8` · `len = obj+0x108` · 용량 0x100 · `mode = obj+0x10c` 다.

- 0x12e: `setMode(obj, m)` 가 `obj+0x10c = m` 을 쓰고 `0x12e(table[m])` 을 부른다. `table` 은 0x12d 이름표에서 찾은 인덱스다. 이 타이틀의 m 은 0 EN/L · 2 N123 · 3 KO 로 관측됐다(1 은 남는 EN/S 로 본다 · 관측 안 됨). 위아래 키(-1·-2)가 m 을 바꾼다. 첫 화면에서는 KO 다.
- 0x130 호출 셋. 셋 다 `(key, 0x1f6, buf1, &size1, buf2, &size2)` 이다. 두 버퍼는 8바이트 간격이고 각 5바이트(4 + NUL)가 0 으로 채워진다. 두 크기는 0 으로 들어오고 반환값은 버린다.
  - 숫자 등 일반 키: `size1 > 0` 이면 buf1 을 `text+len` 에 복사하고 `len += size1` 한다. 대기 중 표식을 지운다. 영문 모드(m ≤ 1)에서 대기 중인 글자가 있고, 키가 지난번 키와 다르고, `size1 == 0`·`size2 > 0` 이면 게스트가 대기 글자를 스스로 확정한다. `size2 > 0` 이면 buf2 를 `text+len` 에 쓰되 `len` 은 늘리지 않는다(보여 주기만 한다). 그 키와 buf2 를 «대기 중»으로 적는다.
  - 위·아래·오른쪽(0xff·0xfe·0xfc): 호출 뒤 대기 표식을 지우고 `len = strlen(text)` 로 다시 잰다. 보여 주던 조합 글자가 그대로 확정된다.
  - CLR(0xf0 = -16): 같은 확정 뒤 마지막 글자를 지운다(KS X 1001 2바이트를 한 글자로 본다).
  - 확인(-5)은 0x130 을 부르지 않고 바로 끝낸다. 이름은 C 문자열로 읽히므로 조합 중 글자도 포함된다.
- ⇒ **buf1 = 이번 키가 확정한 글자, buf2 = 아직 조합 중인 글자**. 둘 다 EUC-KR 이다. 게스트의 KS X 1001 검사(첫 바이트 0xb0..0xc8 · 0xa4 + 0xa1..0xd3)가 근거다. 타이머는 없다. 멀티탭 글자는 다음 키가 확정한다. 그래서 «같은 키인가 · 창 안인가»는 키를 받을 때 판정하면 된다.
- 다른 호출 타이틀(정책 키 300초 · `RUST_LOG` 디버그 · 이 브랜치): `236c7da689f6` `2a8a3dcd07eb` `4fdbd64c9fbd` `601556233e71` `87b04639cdfe` 는 같은 버퍼 배치(buf2 = buf1 − 8)로 부른다. 다섯 다 키 **0x9d** 를 한 번씩 보낸다. 숫자 입력 전에 오거나(빈 상태) 숫자 바로 뒤에 온다(`236c` 는 `'1'` → `0x9d`). «조합 끝»으로 읽었다. 비숫자 키는 모두 대기 글자를 buf1 로 확정하므로 따로 분기하지 않았다. `0266ca417880` 은 300초 안에 0x130 을 부르지 않았다(서버 선택 확인창). `4fdbd64c9fbd` 는 N123 에서 `'5'` 세 번을 불렀고 셋 다 바로 확정됐다.
- ★#507(중복 티켓)과 다른 점 하나: 0x130 에는 숫자가 아닌 키도 온다(위 0xff·0xfe·0xfc·0xf0·0x9d · 실측). 숫자만 온다고 가정하고 `key - '0'` 으로 표를 찾으면 그 키에서 범위를 넘는다.

### 3. 구현과 전/후

- 모드(0x12d 순서 EN/S·EN/L·KO·N123 → `keypad` Lower·Upper·Hangul·Digit)와 조합 토큰을 에뮬레이터별(core id) 상태로 둔다. `HOST_FIELD_VALUES` 와 같은 방식이다. 0x12e 는 모드를 바꾸고 조합을 비운다. 0x12f 는 현재 모드를 돌려준다. 0x12e 를 부르기 전의 모드는 인덱스 0(EN/S)이다. 스텁 0x12f 가 답하던 값이고, 기기에서 잰 값은 아니다(코드에 `ponytail:`).
- 숫자 키: `keypad::press` 를 부르고(같은 키 · 1000ms 안이면 순환) 렌더한다. 마지막 글자만 buf2(조합 중)로 내고 나머지는 buf1(확정)로 낸다. 토큰은 마지막 글자를 만드는 꼬리만 남긴다. 천지인은 마지막 음절만 다시 쓴다(각 + ㅏ → 가가). 그래서 확정한 글자가 되돌아오지 않는다. N123 은 바로 확정한다. 비숫자 키는 보여 주던 글자를 buf1 로 확정하고 상태를 비운다. `ᆢ` 는 EUC-KR 에 없어 `ㆍㆍ` 로 보낸다.
- 시험 `wipic_im_handle_input_commits_and_composes`: N123 확정 · EN/L 멀티탭(A→B · 다른 키 B 확정 · 창 넘은 같은 키 · RIGHT 확정) · KO 천지인(ㄱ → 기 → 가 → 각 → 가 + 기 · 바이트 값) · 0x12f 조회. `im_settle` 를 «전부 확정»으로 바꾼 트리에서 FAILED(`([65], []) != ([], [65])`) · 원복 후 ok.

**단발 확인(레시피 · 이 브랜치 · 화면 직접 확인)**

| 입력 | 이름 칸 | 그다음 |
|---|---|---|
| KO 기본 · 4 1 2 5 1 2 | ㄱ → 기 → 가 → 간 → 가니 → 가나 | 확인창 «이름 : 가나 · 국가 : GHA» |
| 위 키(EN/L) · 2 2 · 3 3 3 · 9 · 6 (1.5초) 6 · CLR | A → B → BD → BE → BF → BFW → BFWM → BFWMM → BFWM | 확인창 «이름 : BFWM» |

**전/후 — 6축 + progress(census · 같은 임대 · 전 = main `76423d05` · 후 = 이 브랜치)**

| sha12 | 축(boot·render·input·longplay·sound·speed) | progress (P 정체초 · 새 화면 수) | load1 전/후 |
|---|---|---|---|
| `1cd151222bde` | 같음(ok·ok·ok·ok·silent·n/a) | ok → ok (10 · 10 → 10 · 10) — 이름이 «64285555» 같은 숫자에서 «ㄷㅅ» 같은 자모로 바뀌고 둘 다 종목 선택·선수 생성·경기 모드까지 간다 | 14/14 |
| `0266ca417880` | 같음(전부 ok) | n/a → n/a (560 · 3) — 서버 선택 확인창(⒜) · 0x130 미도달 | 12/12 |
| `87b04639cdfe` | 같음(전부 ok) | n/a → n/a (420 · 5 → 530 · 4) — 두 키 동시 누르기(⒜) · 0x130 은 0x9d 1회 | 18/13 |
| `236c7da689f6` | 같음(전부 ok) | ok → ok (0 → 70) | 14/14 |
| `2a8a3dcd07eb` | 같음(전부 ok) | n/a → ok (250 → 120) — P2 없음 · 부하 차로 읽는다 | 20/16 |
| `601556233e71` | 같음(speed n/a 외 ok) | ok → ok (120 → 10) | 18/13 |
| `4fdbd64c9fbd` | 같음(longplay error 외 ok) — 힙 소진, 종전 그대로 | 안 잼 | 20/16 |
| `01f05f8231f4` | 같음(boot fail · 잠긴 파일) | 안 잼 | 12/12 |

- 첫 결함 지점·다음 벽: `1cd151222bde` 의 이름 입력은 이제 막히지 않는다. 정책 키 600초는 이름 확인 → 종목 선택 → 선수 생성 → 경기 모드까지 간다(전·후 같음). 나머지 7종은 정책 키로 글 입력 화면에 닿지 않는다. 그래서 이 변경으로 진도가 바뀌는 타이틀이 없다.
- compat: `1cd151222bde` progress `stuck` → `ok`(이 회차 P · main 도 ok — #502 뒤 다시 재지 않은 행이었다) · `knownIssues_ko` 의 «숫자만» 줄과 census `HAND_NOTE` 항목 삭제. `check-compat-revert` OK(착지 기준 바뀐 행 1). 소식 `docs/player-updates/2026-10-08-lgt-name-letters.json`.

**퇴행(probe A·B 30초 · census 인자 · 전/후)**: 라이브 LGT 5(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) + 가드 2(`49ade89578c5` `ddd885583b15`) + 호출 타이틀 8 = 15종. **판정 15 → 15 같음**(PASS 14 · FAIL 1 = `01f05f8231f4` 종전 그대로) · 키 27/27 · 소리 재생 수 같음(`1b107b96bf4e` A 12 → 11 · `236c7da689f6` long 43 → 44 외). paints 차는 −3% ~ +15% 다. 예외는 `1cd151222bde` B 하나로 +76% 다. 그 실행은 키를 넣지 않아 0x130 을 부르지 않는다. 두 실행이 40분 간격(load1 차 최대 26 → 8)으로 돌아서 그 차로 읽는다.

### 4. 셸 키패드(otterpebble `apps/featurephone`)

`app/player.tsx` 의 `GameKey` 는 누를 때 `press(code)` → `emu.key_down` 을, 뗄 때 `key_up` 을 보낸다. 쥔 키는 반복하지 않는다(`lib/engine.ts` 주석 「쥔 키에 key_repeat 은 보내지 않는다」). 그래서 탭 한 번이 0x130 한 번이 된다. 엔진은 게스트가 키를 처리하는 순간의 `platform.now()` 로 «1초 안 같은 키»를 가린다. 그러니 셸 쪽 키 반복 타이밍과는 무관하다. 1초 안에 같은 키를 다시 탭하면 순환하고, 늦으면 새 글자다. ⇒ otterpebble 후속 없음. 모드 표시(가·A)는 게임이 스스로 그린다.

### 5. 게이트

- (rebase 뒤 트리) `cargo fmt --check` · `cargo clippy --all -D warnings` · wasm32 · `+beta` 전부 rc=0. `RUST_MIN_STACK=4194304 cargo test --all` **737 passed / 0 failed**. `npm run build:wasm` rc=0 · `check-engine-contract` 113 pass / 0 · `npm run audit` PASSED · census selftest 62/62 · `player-data` OK.
- 러너 블록: draw · helloworld ×2 · keydraw ×2(`--inject --expect-last-frame` · rc=0 · paints 79 / 55) · text 전부 PASS.
- 회차가 띄운 프로세스: 끝에 `/tmp/imdbg/wv_*` 0(`pgrep`).
- 폭 하나를 어겼다: 입력기 호출 기록 스윕은 가드 rc=1 을 보고도 `-P 3` 으로 걸어 두었다. 임대를 기다리는 동안 `-P 2` 로 고쳤고, 실제로는 2로 돌았다.
- 게임 파일명 유입(`corpus-name-inflow` · 이 브랜치 대 `origin/main`): BOUNDED 338쌍 · SUFFIX-ATTACHED 15쌍. 전부 «건드린 파일 전체»의 기존 언급이다. `compat.json` 의 `title`·`fileTitle`(공개 계약 필드)과 `svc_ids.rs`·`wipi_c.rs` 의 기존 주석이다. 이 회차가 «추가한» 줄의 게임명은 0 이다(`git diff` 추가 줄 대조 · 종전 주석의 한 이름은 sha12 로 바꿨다).

<!-- corpus-name-inflow v1 subjects=10 tree=87a9e1ee3254fe6a B=728/338 P=0/0 S=35/15 -->
