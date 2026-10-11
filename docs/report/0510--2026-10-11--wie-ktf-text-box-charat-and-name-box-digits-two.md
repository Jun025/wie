## [2026-10-11] KTF 진도 엔진 벽 2 — 숨은 글상자 `charAt(4)` · 이름 상자 숫자 · 재측 2 (wie-ktf-text-box-charat-and-name-box-digits-two)

**무엇을**: 0507 원장의 `b-engine` 4행을 이 핀(`origin/main` `6ea6a6bd`)에서 다시 쟀다. `8c71be3ad26d` 의 `charAt(4)` 와 `d448aee68157` 의 «이름 상자가 숫자를 안 받는다»는 둘 다 이 핀에서 진도를 막지 않는다. 정책 키만으로 이름 상자를 지나 본편에 들어간다. 엔진은 고치지 않았다. `14a62a8521a0` 은 정책 키 600초 짝이 둘 다 `ok` 라 `progress` 를 `ok` 로 바꿨다. `3ccc6cf147d2` 는 짝 둘 다 `stuck` 이라 그대로다.

**왜**: 운영자 지시(2026-09-30 · 10-11 «이어서 필요한 후속 작업들을 완전자율주행으로»)다. 0507 §5 가 이 넷을 «엔진 몫»으로 남겼다.

**사용자 영향**:
- 댄스 게임 1개의 진도 배지가 «자동으론 막힘»에서 «계속 진행»으로 바뀐다.
- 나머지 셋은 바뀌지 않는다. 엔진 동작도 바뀌지 않는다.

### 1. 측정 조건

- 엔진: `origin/main` `6ea6a6bd` 의 release `wie_validate`.
- 진도: `bash scripts/census-drive.sh … --only progress --progress 600 --jobs 3 --as P` → `--as P2`(정책 v2). 시작 때마다 `host-load-guard --status --recovered` rc=0 이었다(idle 7~73%).
- A·B·L 은 이전 out 에서 옮겼다: `8c71be3ad26d`·`3ccc6cf147d2` 는 0470 회차, `14a62a8521a0` 은 2026-10-07 회차, `d448aee68157` 은 0507 회차.
- 짝마다 t400~t600 화면을 눈으로 봤다.
- 증적(화면 · census 표 · 기각한 모형의 소스)은 저장소 밖 `~/orchestrator/reports/evidence/wie-ktf-text-box-charat-and-name-box-digits-two/` 에 있다.

### 2. 전/후 표(전 = 원장 값 · 후 = 이 핀)

| sha12 | 원장 핀 · progress | 이 핀 P / P2 정체(초) | 마지막 3분의 1(눈) | progress |
|---|---|---|---|---|
| `8c71be3ad26d` | `f5c02609` · stuck | 430 / 480 | 농장 집 안 · «1일»·«봄» 띠 · 메뉴 | stuck 그대로 |
| `d448aee68157` | `26f79198` · stuck | 250 / 310 | 펫 방 · 시계와 배고픔 막대가 간다 | stuck 그대로 |
| `14a62a8521a0` | `a3da330d` · stuck | 0 / 0 | 무대·농구장에서 춤 · 점수가 오른다 | stuck → **ok** |
| `3ccc6cf147d2` | `26f79198` · stuck | 340 / 340 | «환경설정» 메뉴 | stuck 그대로 |

- 여섯 축은 재지 않았다(`status` 그대로).
- `14a62a8521a0` 과 `3ccc6cf147d2` 는 레시피 판도 쟀다. 레시피 판 정체는 `14a62a8521a0` 90/30, `3ccc6cf147d2` 540/540 이다.
  - 계약(`docs/contracts/featurephone-public-data.md` §progress)은 레시피 결과를 싣지 않는다.
  - 그래서 위 표와 `compat.json` 은 정책 키 판(레시피 없음)만 쓴다.
- `8c71be3ad26d` 와 `d448aee68157` 은 짝 둘 다 본편 화면이다. 그래도 새 화면 수로는 막힘이라 `stuck` 을 그대로 둔다(0507 의 `6d63b4025bef` 와 같은 경우).
- 예외는 모두 기존 것이다. `8c71be3ad26d` 의 `StringIndexOutOfBoundsException` 3회(P), 두 SKT/KTF 타이틀의 `RecordStoreNotFoundException`(첫 실행)이다. `uncaught_threads` 는 넷 모두 0 이다.

### 3. `8c71be3ad26d` — `charAt(4)`

- 이름 상자는 `TextBoxComponent(null, 0)` · `setMaxLength(5)` 다. 게임은 키마다 `getString().charAt(4)` 를 읽는다. 글자가 5보다 짧으면 예외가 난다.
- 이 핀에서는 예외가 나도 게임이 계속 간다. 글자 하나를 넣고 OK 를 누르면 상자가 닫힌다. 그 뒤 할아버지 대사를 지나 본편에 들어간다(P t520 · P2 t550).
- 0470 의 «타이틀에서 정지»는 `f5c02609` 핀의 관측이다. 그 사이 무엇이 바꿨는지는 이분 탐색하지 않았다.

**고치지 않은 이유 — 시험한 모형을 기각했다.**
- 실기 `TextComponent.getString` 은 `AromaWIPI_classes.zip` 에서 본문 12바이트다. 이 길이는 `new String(m_td)` 와 맞고 `new String(m_td, 0, charCount)`(17바이트)와는 맞지 않는다. 그리고 실기에는 `charCount` 필드가 따로 있다.
- 그래서 «`setMaxLength` 가 `m_td` 를 그 길이의 버퍼로 만들고 `getString` 이 `'\0'` 까지 돌려준다»는 모형을 만들어 쟀다(`charCount` 추가 · `setString` 은 정확한 길이).
  - `8c71be3ad26d`: 예외 14 → 0 이다. 100초 판이 본편 띠까지 간다.
  - `9789fec50f39` · `0c67145b11df` · `b22a7fcfb406`: 이름 경로 짝이 같은 화면에서 끝난다.
- 그런데 빈 상자에서 OK 를 누르면 두 게임이 다르게 군다.
  - `8c71be3ad26d`: 이 핀은 빈 이름을 받지 않는다. 모형은 빈 이름(`'\0'` 5개)을 받아 상자를 닫는다.
  - `d448aee68157`: 이 핀은 «내용을 입력해 주세요» 창을 띄운다. 모형은 창도 띄우지 않고 넘어가지도 않는다.
- 두 게임이 따로 만든 «빈 이름 거절»을 모형이 둘 다 깬다. javadoc 도 `getString` 이 빈 문자열을 돌려준다고 적는다.
- ⇒ 모형을 버렸다. 실기의 `charAt(4)` 가 어떻게 지나가는지(예외를 삼키는지, 키가 이 카드까지 오지 않는지)는 ARM 코드를 풀어야 안다. 진도를 막지 않으므로 이 회차에서는 더 파지 않았다.

### 4. `d448aee68157` — «이름 상자가 숫자를 안 받는다»

- 낮 인트로(«러비이름 지어줘!»)에서 쟀다. 시계를 12시간 옮긴 비커밋 탐침 바이너리를 썼다(밤에는 다른 인트로가 나온다).
- 키는 상자에 들어간다. `*` 로 숫자 모드로 바꾸고 `2 2 6 3` 을 넣은 뒤 OK 를 누르면 «너는 누구야?» 프로필 창으로 넘어간다. 한글 모드도 같다.
- 0507 탐색이 본 «아무것도 안 써진다»는 원인이 따로 있다. **lwc 글상자가 글자를 그리지 않는다**(`Component::paint` 가 스텁). 입력은 되는데 화면에 안 보인다.
- 빈 상자에서 OK 를 누르면 «내용을 입력해 주세요» 창이 뜬다. 그래서 글자가 안 보이는 상자에 OK 만 누르면 막힌 것처럼 보인다.
- 고치지 않았다. 글상자를 그리려면 lwc 배치가 먼저다. 이 게임은 `FormComponent` 안에 상자 둘을 넣고 위치를 주지 않는다. 지금 lwc 층은 아무것도 배치하지 않는다. 크기는 M 이다(Form 세로 쌓기 + `TextComponent` 그리기). worklog 제안 1건으로 남긴다.

### 5. 원장 · compat · 소식

- 0507 원장 csv 4행:
  - `8c71be3ad26d` · `d448aee68157` → `a-below`(정책 키가 본편에 닿는다 · 엔진 벽 아님).
  - `3ccc6cf147d2` → `q-skill`(정책·레시피 둘 다 «환경설정»에서 끝난다).
  - `14a62a8521a0` → `ok-policy` · `progress_after` `ok`.
  - `pin` 은 `6ea6a6bd`, `basis` 에 `0510` 을 더했다.
- `compat.json`: `14a62a8521a0` 의 `axes.progress` `stuck` → `ok`, 이 한 필드뿐이다.
- 엔진 동작이 바뀌지 않아 `docs/player-updates/` 소식은 더하지 않았다(0478 §4 와 같다).
- 남은 stuck(playable)은 93 → 92 다.

### 6. 자원 · 자인

- 단발 확인과 짝은 `build-slot run --` 로 돌렸다. phase 짝 2묶음은 `build-slot run --long` 임대 안에서 2개씩, 진도는 `census-drive.sh` 로 돌렸다.
- ★자인 1: 첫 진도 판을 `--progress` 없이 띄웠다(기본 1800초). 10분 뒤 알아채 거두고 600초로 다시 돌렸다.
- ★자인 2: 정책 키 P 를 16:05 에 셸 `&` 로 띄웠다(CLAUDE.md ⒝ 위반). 같은 분에 드라이버·census·에뮬레이터를 거두고 도구 백그라운드로 다시 띄웠다. 겹친 시간은 1분 안이다.
- 끝난 뒤 내 `wv_*` 프로세스는 0 이다. `nohup` 은 0 이다.

### 7. 게이트

엔진 코드 변경은 0 이다. 바뀐 것은 compat 1필드 · 원장 csv 4행 · 문서뿐이다. 그래서 «되돌리면 red» 대조와 라이브 등재분·가드(`49ade89578c5` `ddd885583b15`) 퇴행 짝은 해당 없다.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · wasm32 clippy rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(기존 unused-dependency 경고 9줄).
- `RUST_MIN_STACK=4194304 cargo test --all` **775 passed / 0 failed**(build-slot).
- wasm 빌드: Rust 변경이 없어 돌리지 않았다.
- `node scripts/player-data.mjs` OK(429 · 156 updates) · `check-compat-revert` OK(착지 기준 바뀐 행 1) · `check-worklog-json` OK · `npm run audit` PASSED · 연번 검사 OK.
- 유입: 이 회차가 더한 문서(0510 · worklog)는 BOUNDED 0 · SUFFIX-ATTACHED 0 이다. 원장 csv 4행은 sha12 와 영어 메모뿐이다. 브랜치 전체 B=718/330 · S=36/16 은 기존 `compat.json` title 이다(이 회차는 title 을 바꾸지 않았다).


<!-- corpus-name-inflow v1 subjects=4 tree=10b59ba11e1b0247 B=718/330 P=0/0 S=36/16 -->
