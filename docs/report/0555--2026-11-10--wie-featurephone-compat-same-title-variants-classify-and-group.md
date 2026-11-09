## [2026-11-10] 공개 게임 목록의 «같은 게임 두 줄»을 데이터 층에서 정리 — ` (2)` 순번 대신 `game`·`variant`·`sameTitle` (wie-featurephone-compat-same-title-variants-classify-and-group)

**무엇을**: `scripts/player-data.mjs` 의 `retitle` 이 모든 행에 `game`(통신사와 무관한 묶음 키)과 `variant`(판 라벨 또는 null)를 싣는다. 한 통신사 안에서 이름이 겹치는 행에는 `sameTitle`(`same-game`|`undetermined`)도 싣는다. `title` 에서는 순번 ` (2)` 와 판 라벨 ` (작은화면)` 을 뺐다. 묶음 안 순서는 «대표 행 먼저»다(status → 여섯 축 ok 수 → changes 수 → sha). `build` 는 changes 를 붙인 뒤 다시 정렬한다. 검사기는 순번 잔존·`game` 누락·대표 순서 어김·판정 없는 겹침을 거부한다. `normalize` 명령을 더해 커밋된 compat.json 을 다시 맞출 수 있게 했다. 계약 = `docs/contracts/featurephone-public-data.md` §1.

**왜**: 운영자 지시(2026-11-10). 라이브 `/support` 에 `거상 (2)` 처럼 순번이 붙은 이름이 17개 보였다. 이 순번은 PR #357 이 «겹치면 순번»으로 정한 규칙이고, 어느 쪽이 (2) 인지는 sha 순이라 뜻이 없었다. 행은 지우지 않았다(429 → 429). 매칭 키가 파일 내용 sha256 이라, 행을 지우면 그 판을 가진 이용자의 파일이 «미확인»이 된다.

**사용자 영향**: 이 PR 만으로는 화면이 «한 줄»이 되지 않는다. 묶어 그리는 일은 셸 후속 티켓(`otterpebble-featurephone-support-list-one-row-per-game`)이 맡는다. 다만 이 PR 이 실리면 ` (2)`·` (작은화면)` 이 이름에서 사라진다. 같은 이름의 두 줄은 셸이 `variant` 를 그리기 전까지 이름만으로는 구별되지 않는다.

### 판정 — 메타데이터만(게임 바이트·리소스 내용은 인용하지 않음)

근거: `__adf__`/`app_info`/`.msd` 의 PID·Name·Ver·DisplaySize, 아카이브 항목 이름 일치율, 본체 jar 해시 일치 여부(해시 값은 비교만 했다). 코퍼스는 `game_lab/`(git-ignored)이다.

| 묶음 | sha 앞 10자 | 판정 | 근거 | 결과 title / variant |
|---|---|---|---|---|
| KTF 거상 | 070daa5b55 · 0e6cd18872 | same-game | PID PD004036·Name·Ver 01.00.05 같음 · jar 같음 · `__adf__`/`__class__` 만 다름(AID 설치 칸) | 거상 / null · null |
| KTF 던파거너편 | d4188f8ef8 · ed6ad7318a | same-game | 항목 6/6 내용까지 같음(zip 포장만 다름) | 던파거너편 / null · null |
| KTF 리얼비트 | 3be2a19f5d · c01e041fa7 | same-game | PID·Ver 같음 · jar 같음 · AID 만 다름 | 리얼비트 / null · null |
| KTF 리얼싸커2009 | 1cf2e60760 · 5028b8a5d1 | same-game | PID PD007392·Ver 01.00.02 같음 · jar 같음 · 한쪽은 `W/apps/` 설치 트리 포장 | 리얼싸커2009 / null · null |
| KTF 마린블루스 | 4b1c194795 · eda7ecb85a | same-game | PID·Ver 같음 · jar 같음 · AID 만 다름 · 한쪽에 항목 1개 더 | 마린블루스 / null · null |
| KTF 마이프린세스2 | 41466fc7f7 · 52f1f32e3f | same-game | PID·Ver 같음 · 45항목 중 44 같음(`__adf__` AID) | 마이프린세스2 / null · null |
| KTF 메이플스토리 | 1cdea19859 · 3b5b98afa8 | **different-game** | PID 다름(PD001295 «메이플-마법사편» 01.00.06 176*220 ↔ PD002198 «메이플스토리» 01.00.05 240*320) · jar·아이콘 전부 다름 | 메이플스토리 마법사편 · 메이플스토리 전사편 / null(이름 출처: 앞은 `__adf__` Name, 뒤는 코퍼스 파일 이름 «(전사편)») |
| KTF 영웅전설4 주홍물방울 | 182fa44210 · 91c02913e1 | same-game | 항목 6/6 내용까지 같음 | 영웅전설4 주홍물방울 / null · null |
| KTF 이노티아1 | 93d5b6b8ce · 94a34abdea | same-game | PID PD005362·Ver 01.00.05 같음 · jar 같음 · 한쪽에 설치 안내 그림 1개 더 · AID 만 다름 | 이노티아1 / null · null |
| KTF 이노티아연대기2 | 974e0df9ab · f770b15f88 | same-game | PID PD007974·Ver 01.00.01 같음 · jar 같음 · 위와 같은 형태 | 이노티아연대기2 / null · null |
| KTF 지크 | 704839f1a6 · 8bfd08fe43 | same-game | PID PD005778·Ver 01.00.04 같음 · jar 같음 · 소리 리소스 5개 내용 다름 + 3개 더(에디트·재포장 변형) | 지크 / null · null |
| KTF 질주쾌감 스케쳐 | 306dcdb038 · e159b9727d | same-game | PID PD004547 같음 · Ver 01.00.05 ↔ 01.00.01 · DisplaySize 240*320 ↔ 176*220 · jar 다름(판올림·화면 판) | 질주쾌감 스케쳐 / 큰화면 · 작은화면 |
| KTF 추억의달고나 | 3d38a46bec · 44a4228cca | same-game | 항목 5/5 내용까지 같음 | 추억의달고나 / null · null |
| KTF 크아bnb2007 | 3ccc6cf147 · 4288d8c1c6 | same-game | PID PD004541·Ver 01.00.00 같음 · DisplaySize 176*220 ↔ 240*320 · jar 다름(화면 판) | 크아bnb2007 / 작은화면 · 큰화면 |
| KTF 투스워즈 | 1d18b7cc06 · a5651a180b | same-game | PID·Ver 같음 · 153항목 중 152 같음(`__adf__` AID) | 투스워즈 / null · null |
| LGT 영웅서기3 | 0bc9cc28c0 · 6f170f6925 | same-game | PID PD119037·Name 같음 · Ver 01.00.02 ↔ 01.00.04 · jar 다름(판올림) | 영웅서기3 / 01.00.02 · 01.00.04 |
| SKT 웰루시아 | d1b55085bd · d6abd6258d | same-game | MIDlet-Name·Version 1.7.0·Vendor 같음 · 항목 이름 6/6 같음 · jar·`.mod` 내용 다름(같은 크기 — 패치 변형) | 웰루시아 / null · null |

`undetermined` 는 0건이다. 이미 라벨로 갈려 있던 6묶음(KTF 마스터오브소드2·붕어빵타이쿤3·영어뇌습격·절묘한타이밍·프린세스메이커4, SKT 프린세스메이커4)도 같은 방법으로 쟀다. 여섯 모두 PID 가 같아 `same-game` 이고, 라벨은 `title` 에서 `variant` 로 옮겼다.

### 수치

- 행 429 → 429. title 이 아닌 필드의 변경은 0이다(대조 스크립트로 sha 별로 확인). 이름이 바뀐 행은 28이다: 순번 제거 17(그중 «메이플스토리 (2)»는 «메이플스토리 전사편»으로 개명), 순번이 없던 different-game 쪽 개명 1(«메이플스토리» → «메이플스토리 마법사편»), 라벨 ` (…)` 이 `variant` 로 옮겨 간 행 10.
- `game` 묶음 수는 **378**(통신사 무관)이고, (platform, game) 묶음은 406 이다. 티켓의 «≈384 = 429 − 45»는 라벨로 갈린 6묶음이 겹침 수에 빠진 근사치였다. 실측은 `node -e` 로 셌다.
- `sameTitle: same-game` 45행이다. `variant` 가 실린 행은 16이다(파일 이름 라벨 10 + 메타데이터에서 손으로 단 6).

### 검사

- `node scripts/player-data.mjs --selftest` → 42 rules. 개악 ⒜ `title: "거상 (2)"` → `title carries an ordinal or variant label`. ⒝ `game` 삭제 → `empty game`. ⒞ 대표 순서 뒤집기 → `comes after a worse row of its group "거상"`. 셋 다 red 다.
- `build` 산출물을 otterpebble `origin/main` 의 `compat-import.mjs` `validateCompat` 로 읽기 전용 실행했다. 위반 0 이다. 셸의 `normalizeCompat` 정렬 뒤에도 묶음의 첫 행은 하나도 바뀌지 않았다(안정 정렬).
- 게임 이름 유입: `corpus-name-inflow` BOUNDED 362쌍(1112회) · SUFFIX-ATTACHED 16쌍(52회). 의도한 유입이다 — 공개 목록(`compat.json` 의 `title`·`game`·`fileTitle`)이 산출물이고, 이 문서의 판정 표가 묶음 이름을 적는다. 게임 바이트·리소스 내용·캡처는 0.

