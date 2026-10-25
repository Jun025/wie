## [2026-10-26] SKT 서버 벽 `640428a9cf9e` — 받던 것이 게임 자원이라 남김, 안내 문장만 교정 (wie-skt-server-wall-princessmaker4-small-required-data)

**무엇을**: SKT `limited` 1종 `640428a9cf9e`(서버 벽)가 서버에서 무엇을 받는지 javap 와 실행 로그로 쟀다. 판정은 ⒝(jar 밖 게임 자원)다. 엔진 코드는 바꾸지 않았다. `HAND_WALL` 과 `compat.json` 의 안내 문장만 실제 벽에 맞게 고쳤다.

**왜**: 운영자 지시(2026-10-26)는 SKT 미플레이 판을 플레이 가능으로 만드는 것이다. `not-yet` 5종은 구매 단말 잠금이라 정책상 범위 밖이고, 남은 1종이 이 판이다. 0542 의 `wie_backend::fake_server` 에 SKT 행을 더할 수 있는지 먼저 판별했다.

**사용자 영향**: 상태는 `limited` 그대로다. 안내가 «필수 데이터 파일을 서버에서 받아야 한다»에서 «그림과 게임 데이터가 이 파일에 없다 · 큰화면 판은 된다»로 바뀐다. 이용자 소식은 내지 않는다(켜진 것이 없다).

### 1. 판정

| sha12 | 판정 | 근거 한 줄 |
|---|---|---|
| `640428a9cf9e` | ⒝ 남김 | «예»를 고르면 `socket://211.239.157.163:5252` 로 파일 28개(그림 27 · `data.jar`)를 받아 그 이름 그대로 저장한다. 이 판의 jar 에는 28개 중 하나도 없다 |

### 2. 무엇을 받는가 (javap · openjdk@17)

- **접속**: `e.g()` 상태 42 → `XFile.fsavail() > 328668` 이면 `Connector.open("socket://211.239.157.163:5252")` 에서 `DataInput/OutputStream` 을 연다. 실패하면 상태 45(«연결상태가 좋지 않습니다» · 0.돌아가기)로 간다.
- **프로토콜**(상태 43, 파일마다 한 번): `writeInt(번호)` → 첫 회만 `readUTF()` → `readInt(길이)` → 그 길이만큼 바이트 → `c.a([BI)V` 가 `XFile(이름표[번호], 쓰기)` 로 저장 → `writeUTF("받았다")`. 마지막 번호에서 상태 44(닫기)로 간다.
- **이름표**(`c.<init>`): `/baca.png` `/back1.png` `/back2.png` `/bun.png` `/data.jar` `/friend1~7.png` `/l24~33.png`(9개) `/m24~32.png`(7개). 모두 28개이고 jar(`image/`·`sound/` 아래 721 png)에는 하나도 없다.
- **쓰임**: `c.a(int)` 가 `XFile("data.jar", "data<N>.txt")` 로 항목을 읽어 게임 진행(`e` 의 `f[28]` 번호)에 넘긴다. 받는 것은 인증·버전이 아니라 게임이 그리고 읽는 자원이다.
- **같은 게임 큰화면 판(`ccb45e6b8d80`)과 대조**(이름만 · 바이트는 옮기지 않았다): 그 패키지는 같은 28개 이름을 jar 옆에 함께 싣고 있고, `data.jar` 는 73항목(`dataN.txt`)이다. 작은화면 판은 이것을 빼고 첫 실행 때 서버에서 받게 만든 판이다. 그래서 큰화면 판은 `playable` 이다.

⒜(최소 성공 응답)가 안 되는 이유: 응답이 곧 그림과 게임 데이터 바이트다. 만들 수 없고, 큰화면 판에서 옮기는 것은 티켓이 금한다. ⒞(구매 인증)도 아니다.

### 3. 실행 로그

`wie_validate --inject --boot-secs 8 --keys "NUM1:6 NUM0:6 NUM1:6 NUM0:6"`(release · 이 브랜치):
- `00_boot`: «필수 데이타 파일을 전송받습니까? 1.예 2.아니오»
- `01_NUM1`: «연결상태가 좋지 않습니다.잠시후 다시 접속하세요 0.돌아가기»
- `java_exceptions`: `java/io/IOException: no network` 1회. 위 `Connector.open` 이 낸 것이다. 그 뒤 `clean exit`(rc=2 UNMEASURED · 키 2/4 · 판정 대상 아님)

증거 폴더(로컬): `~/orchestrator/reports/evidence/wie-skt-server-wall-princessmaker4-small-required-data/` — `javap-excerpt.txt` · `download-targets.txt` · `validate.json` · `shots/`.

### 4. 바꾼 것

- `scripts/playability-census.mjs` `HAND_WALL['640428a9cf9e']`: 문장 교정과 이 보고서를 가리키는 주석.
- `docs/player-data/compat.json` 같은 행 `knownIssues_ko` 1줄. 상태·축은 그대로다.
- 엔진 코드 0행. `fake_server` 의 `SERVERS` 에 SKT 행을 넣지 않았다.

### 5. 남은 SKT 미플레이

`not-yet` 5종(구매 단말 잠금)과 이 1종(⒝)이다. 이 판을 켜려면 원래 서버가 주던 28개 파일이 있어야 한다. 엔진 쪽에서 할 수 있는 일은 없다.

### 6. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치 대 `origin/main`): BOUNDED 718회/330쌍 · SUFFIX-ATTACHED 37회/17쌍이다. BOUNDED 와 SUFFIX-ATTACHED 는 모두 바뀐 파일 «전체»를 훑은 수다. 이 회차가 더한 줄에서 나온 것은 없다. `compat.json` 의 기존 `title`·`fileTitle` 칸과 `playability-census.mjs` 의 기존 안내 문장 한 줄(SUFFIX-ATTACHED · 조사가 붙은 일반 낱말)이 잡혔다. 이 보고서 본문에는 제목을 쓰지 않았다(sha12 로만 가리킨다).


