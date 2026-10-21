## [2026-10-21] 소리 없음 16 · 소리 미상 3 · 속도 미상 26 — 36행 전수 표 · KTF 동봉 소리 끔 2종 바이트 고침 · LGT 1종 속도 «느림» 확정 (wie-sound-no-and-speed-unknown-remeasure)

**무엇을**: `compat.json`(origin/main `a90f1472`)에서 소리 `no` 16 · `unknown` 3 · 속도 `unknown` 26 을 행마다 갈랐다. 겹치는 행을 빼면 **36행**이다(티켓의 «42»는 중복 제거 전 어림). 표는 [`0525--…--table.csv`](0525--2026-10-21--wie-sound-no-and-speed-unknown-remeasure--table.csv)다.
- 엔진: `wie-ktf` `OWNER_SOUND_OFF_BYTES` 를 새로 두었다. 원 주인이 소리를 꺼 둔 동봉 설정 파일을 빼면 게임이 멈추는 2종에서, 그 설정 바이트만 «켬» 값으로 바꿔 올린다.
- `compat.json` 3행을 고쳤다. 소리 `no → ok` 2행(위 2종 · 안내 문장 제거)과 `9acb5e0387eb` 속도 `unknown → no` 1행이다. 속도 행에는 느린 이유 문장을 하나 더했다.
- 이용자 소식 1건.

**왜**: 운영자 지시(2026-10-21) — 완성도 향상 후속.

**사용자 영향**: 2종이 처음부터 소리가 난다. 1종은 «속도 모름» 대신 «키를 누른 뒤 절반 속도»라고 정직하게 보인다. 나머지 33행은 표에 이유를 적고 값은 그대로 뒀다.

### 1. 전/후 수

| 축 | 전 | 후 |
|---|---|---|
| sound `no` | 16 | **14** |
| sound `unknown` | 3 | 3 |
| speed `unknown` | 26 | **25** |
| speed `no` | 0 | 1 |
| 합집합(행) | 36 | 34 |

status 수는 그대로다(411 / 10 / 8).

### 2. 분류(36행)

| 계급 | 수 | 처분 |
|---|---|---|
| 기기 설정(동봉 «소리 끔») | 9 | KTF 2 → **이 PR 에서 고침**. SKT 7 → 안내 유지. 그중 4종(`c361632541a7` `abc0c5789259` `7089dec0e8df` `6e93f26fa2f5`)은 파일을 빼면 소리가 난다(0440). SKT 에는 동봉 파일을 처분하는 장치가 없어 후속으로 넘겼다(worklog p0) |
| 원래 소리 없음(닿을 수 있는 화면) | 3 | 서버 벽 2(`f44271803135` `640428a9cf9e`) · 서버 접속 실패 뒤 종료 1(`689c491586b9`). 닿을 수 있는 화면에서 재생 0 이다 |
| 미측정 — 잠긴/암호 파일 | 8 | 정책 행이다. 소리 `no` 4행(SKT 잠금)은 그림 0장 · 스스로 끝나는 거부 경로를 잰 값이다(`regression-diff.mjs` 정책 행 규칙). 안내 문장(«잠긴 파일»)이 참이라 그대로 뒀다 |
| 미측정 — 속도 · 대기 없음 | 14 | 창 안 sleep·타이머가 0 이다. 비율의 분모가 없어 이 도구로는 원리상 못 잰다(0516 N). 표에는 13행으로 보인다 — `7089dec0e8df` 는 소리 계급으로 셌다 |
| 미측정 — 속도 · 그 밖 | 3 | `d5e996a53118`(헤드리스뿐 · 후속 p1) · `71d1d8235bd1`(sleep 이 프레임 대기가 아니다 · 0516 W) · `689c491586b9`(창 미도달 · 위 소리 계급과 같은 행) |
| 속도 «느림» 확정 | 1 | `9acb5e0387eb` → `no`(§4) |

### 3. 엔진 고침 — `OWNER_SOUND_OFF_BYTES`

- 0519 의 `OWNER_SOUND_OFF_SETTINGS` 는 파일을 통째로 뺀다. 이 2종은 그게 안 된다. `5aa438fbdc87` 은 파일을 빼면 스스로 종료하고(0440), `6c9f969f089f` 는 화면이 멈춘다(0440 · 0517).
- 그래서 jar MD5 · 파일 · 오프셋 · 바이트 목록으로, 잰 바이트만 바꾼다. 같은 MD5 키 · 같은 `P/`·`p/` 철자 처리를 쓴다. 다른 타이틀과 그 파일의 다른 바이트는 그대로 올라간다.
  - `5aa438fbdc87` `op_save.db` +0: 소리 크기, 원 주인 0 → 3.
  - `6c9f969f089f` `personal_data` +4(빅엔디언 int): 소리 끔 플래그, 원 주인 1 → 0.
- 짝(release `wie_validate` · census probe A 인자 · 90초 · load1 60~190):

| sha12 | 원본 · 전 빌드 | 바이트 바꾼 사본 · 전 빌드 | 원본 · 후 빌드 |
|---|---|---|---|
| `5aa438fbdc87` | 재생 0 · 그림 141 | 재생 5 · MIDI 2,701 · 그림 139 | 재생 5 · MIDI 2,701 · 그림 161 |
| `6c9f969f089f` | 재생 0 · 그림 46 | 재생 3 · MIDI 117 · 그림 49 | 재생 2 · MIDI 46 · 그림 130 |

  두 행 모두 결과 PASS · `stop` deadline 이다. 후 빌드 로그에 `Turning the original owner's sound on in P/…` 가 1회씩 찍혔다.
- 되돌리면 red: `emulator::tests::owner_sound_off_bytes_rewrite_only_the_listed_bytes`. 목록 행(MD5 는 `md5 -q` 출력 그대로)과, 다른 jar 의 파일은 바이트 하나도 안 바뀌는 것을 함께 못박는다.
- 퇴행 범위: 목록의 MD5 둘 말고는 이 함수가 아무것도 하지 않는다. 시험의 `b"other"` 경우가 그 단언이다.

### 4. `9acb5e0387eb` 속도 `unknown → no`

- 계약 술어(`featurephone-public-data.md` §등급 술어)는 `no` 를 «브라우저 2회가 서로 10% 안이고 < 0.9»로 정한다. 0518 §1 의 브라우저 짝이 그 조건을 채운다. 키를 누른 실행 2회가 명목 대비 **0.50 · 0.50** 이었다. 키 없는 실행은 1.00 이었다.
- ★측정량 차이: 0518 은 census 의 «늦음 비율»이 아니라 «게스트 10~40초 창의 그리기 수 ÷ 명목 25/s»를 쟀다. 40 ms 타이머 한 번에 그리기 한 번인 게임이라 같은 것을 더 직접 본다. 원인은 0518 §2 에 있다(게임 자체 그리기 × ARM 해석 처리량 · 엔진 결함 아님).
- 추가 문장: «키를 눌러 게임이 시작된 뒤에는 원래 속도의 절반쯤으로 느리게 움직여요.»

### 5. 속도 재측 — 조용한 창은 없었다

- `origin/main` `a90f1472` release 로 36행 probe(`census-drive.sh --only probe`, jobs 3)를 돌렸다. 이어 playable 속도 미상 18행을 `--only speed --jobs 1` 로 쟀다.
- 시작 시 `host-load-guard --status --recovered` rc=0(load1 37)이었다. probe 도중 포화로 바뀌었다(23:22 rc=1 · load1 157 · idle 0.1%). 측정 행의 load1 은 84~187 이다(표 `load1` 열).
- 결과: **새 `ok` 0**. 대기 없는 14행은 비율이 여전히 없다(부하 무관). 비율이 있는 3행은 `d5e996a53118` 0.347(0516 0.78) · `71d1d8235bd1` 0.794 · `9acb5e0387eb` 0.854 로, 전부 부하 속 하한이다. 헤드리스로는 판정을 바꾸지 않았다(`9acb5e0387eb` 는 §4 의 브라우저 짝으로 바꿨다).
- 0520·0521(타이머 고침)이 0516 뒤에 들어왔지만, 대기 없는 행에 새 대기 경로가 생기지는 않았다(비율 `null` 그대로).
- 조용한 창에서 다시 재더라도 바뀔 수 있는 것은 `d5e996a53118` 하나다. 나머지는 구조상 같은 답이다. 그래서 창을 기다리지 않았다.

### 6. 검증

- 4 게이트: `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 · wasm32 rc 0 · `+beta` rc 0(기존 unused-dependency 경고 9줄) · `RUST_MIN_STACK=4194304 cargo test --all` **785 passed / 0 failed**.
- 러너 블록: draw · helloworld ×2 · text PASS · keydraw ×2 `--inject --expect-last-frame` PASS · rc 0.
- `node scripts/player-data.mjs` OK(429 · 411/10/8) · `node scripts/check-compat-revert.mjs` OK(착지 기준 바뀐 행 3).
- 측정 규율: 에뮬레이터는 전부 `build-slot run` 안에서 돌렸다. 스윕은 `census-drive.sh` phase 임대로 돌렸고, 동시 실행은 최대 3이다. `nohup` 0 · 회차 끝에 내가 띄운 에뮬레이터 0.
- 바이트 바꾼 사본은 `/tmp` 에만 있다(비커밋 · Constraint 9).

### 7. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 718회 / 330쌍 + SUFFIX-ATTACHED 36회 / 16쌍. 전부 `docs/player-data/compat.json` 의 기존 `title`·`fileTitle` 값이다. 이 회차는 그 파일에서 3행의 소리·속도 축과 안내 줄만 바꿨다. 그 밖 파일의 걸림은 0 이다. 코드 주석·회차 문서·표·worklog·소식은 타이틀을 sha12/sha256 으로만 적었다.
