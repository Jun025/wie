## [2026-11-06] 진도 — 0525 ⒟ «확인 키만으로 들어감» 15행 현행 핀 재측 · `fe76e641bb3d` 1행 `ok` · 4행은 «눈 확인 되풀이» 예외로 `stuck` · 계약 개정 (wie-progression-stuck-confirm-only-15-remeasure)

**무엇을**: 0525 원장(`0525--…--ledger.csv`)에서 `cluster = confirm-only-entry` 인 15행을 현행 `origin/main` `2e66f9d6` 에서 다시 쟀다. 측정은 `--only progress` 600초 · 정책 v2 · P 와 P2 짝이다. census 규칙(`progressAxis`)으로는 5행이 `ok` · 10행이 `stuck` 이다. 그 5행의 t380~t600 화면을 눈으로 봤다. `fe76e641bb3d` 는 실제로 진행해서 compat 을 `ok` 로 바꿨다. 나머지 4행은 같은 상태를 되풀이할 뿐이라 `stuck` 으로 남겼다. 이 4행을 받칠 규칙이 계약에 없었으므로 `docs/contracts/featurephone-public-data.md` 의 progress 정의에 «눈 확인 되풀이» 예외를 넣었다(§2-1). 막힘 원인은 15행 모두 «넘는 키 미발견»이고, 엔진 벽과 게임 설계는 0이다.

**왜**: 이 15행의 마지막 측정은 0502(`7ca47380`) 8행 · 0507(`26f79198`) 6행 · 0507/0525(`a90f1472`) 1행(`a0436eb1ddbb`)이다. 0525 §2⑶ 에서는 stuck 의 예외 대부분이 «지난 핀 것»이었다. 그래서 이 행들의 막힘도 이미 풀렸는지 현행 핀에서 확인해야 했다(티켓 근거: `wie-state-next-refresh-and-next-round-proposal` 후보 2). 계약 개정은 검수(request-changes, pin `c3890660`)가 짚은 대로다. 첫 판은 레시피 예외 전용인 «짝 둘 다 ok + 눈 확인»(0507 §3 · 계약의 0502 예외)을 정책 키 판에 갖다 썼는데, 그 인용은 틀렸다. 그렇다고 규칙값대로 5행을 모두 `ok` 로 실으면 상점의 «잔액 부족» 루프 같은 판을 «진행됨»으로 알리게 된다. 그래서 총괄이 계약을 좁게 고치는 쪽을 골랐다.

**사용자 영향**: 1개 게임(`fe76e641bb3d`)의 진도 막힘 표시가 사라진다. 이용자 소식 1건(`docs/player-updates/2026-11-06-lgt-part-time-progress-confirmed.json`)을 썼다. 나머지 14행의 표시는 그대로다.

### 1. 측정 조건

- 엔진: `origin/main` `2e66f9d6` 의 release `wie_validate`. 사본을 저장소 밖 `~/scratch/c15/` 에 두었다.
- 명령: `bash scripts/census-drive.sh --bin … --out … --only progress --progress 600 --as P|P2 --jobs 3 --titles <15행> <15행 코퍼스>`. 15행에 레시피는 쓰지 않았다(«확인 키만» 행이라 정책 자체가 들어가는 길이다).
  - `--only progress` 의 대상은 A·B·L 기록이 있는 행으로 거른다. A·B 는 주간 퇴행 감시 2026-11-01 out 에서, L 은 가장 최근 회차 out(10-06~10-11)에서 옮겼다.
  - `fb80e97cbc57` 은 L 기록이 어디에도 없었다. 그래서 이 핀에서 `--only long --long 600` 을 먼저 쟀다. 결과는 `UNMEASURED`(854/900 키 · deadline)라 longplay 는 `ok` 다.
- 임대: long 1회 + P 3회 + P2 3회. 모두 `census-drive.sh` 의 `run --budget 900` 단위로 잡았다. 스윕 전체를 임대 1건으로 감싸지 않았다.
- 부하: load1 은 9~99 였다(행별 값은 증적 `judge.txt`).
- 증적(화면 모음 · 행별 정체 값)은 저장소 밖 `~/orchestrator/reports/evidence/wie-progression-stuck-confirm-only-15-remeasure/` 에 있다.

### 2. 판정표(15행)

`P / P2` 는 census 의 행별 판정이다(정체 ≥ 200초면 `stuck`). «도구»는 `progressAxis` 값이다. 이 함수는 P 가 `ok` 면 P2 를 보지 않고 `ok` 를 낸다(`if (a !== 'stuck') return a;`). P 가 `stuck` 일 때만 P2 를 보고, P2 가 `ok` 면 `ok` · `stuck` 이면 `stuck` 이다. «눈»은 t380~t600 화면이다. 증적 경로의 앞부분은 `~/orchestrator/reports/evidence/wie-progression-stuck-confirm-only-15-remeasure/` 이다.

| sha12 | 플랫폼 | P / P2 | 정체(초) P / P2 | 도구 | 마지막 3분의 1(눈) | 최종 | 적용 규칙 · 증적 |
|---|---|---|---|---|---|---|---|
| `0cc4ef7ede37` | KTF | stuck / stuck | 540 / 540 | stuck | 사격 연습장. 점수가 100 에서 멈췄고 과녁이 남아 있다 | stuck | census 규칙 |
| `1cdea1985955` | KTF | stuck / stuck | 310 / 220 | stuck | 마을 끝에서 «출입이 불가능합니다» 대사 | stuck | census 규칙 |
| `3151fdc167b6` | KTF | stuck / stuck | 340 / 340 | stuck | 능력치 화면(«레벨업 포인트 없음»)과 거점 메뉴를 오간다 | stuck | census 규칙 |
| `44c292be4e4c` | KTF | stuck / stuck | 260 / 260 | stuck | 방 하나에서 상태창과 지도를 오간다 · 퀘스트 «…진입» | stuck | census 규칙 |
| `6af589a88cf9` | KTF | stuck / stuck | 300 / 300 | stuck | 저장 칸 화면에서 덮어쓰기를 되풀이한다(db 쓰기 236) | stuck | census 규칙 |
| `a20c2044305c` | KTF | stuck / stuck | 320 / 320 | stuck | 들판 위 능력치 배분 창(«GP가 없습니다») | stuck | census 규칙 |
| `fe76e641bb3d` | LGT | **ok** / stuck | 40 / 430 | **ok** | P 는 방을 나와 거리로 간다. 돈이 10,000 → 10,100 으로 오르고 날짜·시계가 간다 | **ok** (compat 반영) | census 규칙. 눈으로도 진행이라 예외 대상이 아니다 · `tail-fe76e641bb3d-P.png` |
| `ecb1bf8d2079` | LGT | stuck / stuck | 440 / 440 | stuck | 메뉴의 랭킹 → «네트워크 이용시 통화료가 부과됩니다» 확인 창 반복 | stuck | census 규칙 |
| `a0436eb1ddbb` | SKT | **ok** / **ok** | 40 / 80 | ok | 놀이기구 상점에서 «구입비용이 부족합니다»를 되풀이한다. 날짜 표시가 그대로다 | stuck | ⒝ 눈 확인 되풀이 · `tail-a0436eb1ddbb-P.png` · `tail-a0436eb1ddbb-P2.png` |
| `fb80e97cbc57` | SKT | stuck / stuck | 440 / 440 | stuck | 지도 화면을 되풀이해 연다 | stuck | census 규칙 |
| `66959afab216` | SKT | stuck / stuck | 360 / 280 | stuck | 들판과 상태창을 오간다 | stuck | census 규칙 |
| `bf54c05e58a9` | SKT | **ok** / **ok** | 160 / 160 | ok | 같은 방에서 같은 대사 두 줄이 되풀이된다 | stuck | ⒝ 눈 확인 되풀이 · `tail-bf54c05e58a9-P.png` · `tail-bf54c05e58a9-P2.png` |
| `52edc54e2b23` | SKT | **ok** / **ok** | 10 / 30 | ok | 마을 오른쪽 끝에 선 채 «출입이 불가능» 대사와 지도 창만 열린다 · `1cdea1985955` 와 같은 자리 | stuck | ⒝ 눈 확인 되풀이 · `tail-52edc54e2b23-P.png` · `tail-52edc54e2b23-P2.png` |
| `6e93f26fa2f5` | SKT | **ok** / stuck | 120 / 440 | ok | 장비/정보 → 코인 → «강철1호 EMPTY» → «정말 장착?» 을 오간다. 전적 «0승» 그대로이고 놀이 화면으로 돌아가지 않는다(P2 도 같은 창) | stuck | ⒝ 눈 확인 되풀이 · `tail-6e93f26fa2f5-P.png` · `tail-6e93f26fa2f5-P2.png` |
| `e3276ce8557c` | SKT | stuck / stuck | 300 / 300 | stuck | 육상 트랙 경기 중 일시정지 메뉴(«1.계속하기») | stuck | census 규칙 |

- 최종 = `ok` 1 · `stuck` 14. 적용 규칙 = census 규칙 11(`ok` 1 · `stuck` 10) · ⒝ 눈 확인 되풀이 4.
- 막힘 원인 = 넘는 키 미발견 14 · 엔진 벽 0 · 게임 설계 0. 미측정 0.
  - 끝 화면이 가리키는 다음 길: `0cc4ef7ede37` 조준 · `1cdea1985955` 다른 길 · `3151fdc167b6` 거점 메뉴의 «스테이지 이동» · `44c292be4e4c` 출구 · `ecb1bf8d2079` 메뉴 항목 · `e3276ce8557c` 종목 조작 · `6e93f26fa2f5` 메뉴 빠져나가기.
- 첫 판(커밋 `5881fcbe`)은 `fe76e641bb3d` · `6e93f26fa2f5` 를 «짝 불일치라 stuck 그대로»라고 적었다. 그 서술은 도구 코드와 반대라 지웠다. P 가 `ok` 면 짝은 필요 없다(selftest `'ok and error need no pair'`). 선례는 0486 의 `3c658a46bbfb` 다.
- ★15행 중 9행은 P 와 P2 의 정체 값이 **같다**(새 화면 수도 같거나 1 차이). 정책 v2 가 결정적이어서 짝은 같은 판을 다시 돌린 셈이다. 그러니 이 행들에서 «P2 가 동의했다»는 독립 증거가 아니다. 그중 8행은 둘 다 `stuck` 이어서 `stuck` 확정의 근거도 그만큼 약하다. 판정은 바꾸지 않았다.
- `52edc54e2b23` 들판 배경의 넓은 마젠타 면은 0460 §관찰 그대로다(10-05 out 에도 있다). 진도와는 따로 있는 렌더 문제이고 이 회차는 재지 않았다.

#### 2-1. 계약 개정 — «눈 확인 되풀이» 예외

`docs/contracts/featurephone-public-data.md` §progress 의 `stuck` 칸과 레시피 예외 다음 문단에 넣었다.

- 정책 키 판(레시피 0)에서 도구 값이 `ok` 여도, t380~t600 화면이 같은 상태의 되풀이면 `stuck` 을 싣는다. «같은 상태»는 같은 창·같은 대사·같은 위치를 오가면서 재화·날짜·시계 변화가 0인 것이다.
- 그 회차 문서에 행마다 증적 화면 경로를 적어야 한다. 위 표의 ⒝ 4행이 그 경로를 단다.
- 좁게 쓴다. `ok` → `stuck` 한 방향뿐이다. 눈으로 진행(재화·날짜·장소 변화)이 하나라도 보이면 도구 값 `ok` 를 뒤집지 않는다. 그래서 `fe76e641bb3d` 는 `ok` 다.

### 3. stderr · 예외 서명 — 2행 이상 겹치는 엔진 서명 없음

- 15행 모두 P · P2 의 `uncaught_threads` 0 이고, stderr 꼬리(200줄)의 `Exception|panic` 도 0 이다. `ERROR` 는 P2 에서 `ecb1bf8d2079` · `fe76e641bb3d` 의 `MC_netConnect … M_E_ERROR (no network)` 뿐이다. `ecb1bf8d2079` 은 위 랭킹 창의 접속이고, 둘 다 `net_connects` 1~2 다. 스텁 상위는 `Font::getDefaultFont`/`Font::<init>` · `MC_grpGetFont` 로, 0525 §2⑴ 이 모집단과 비율이 같다고 잰 서명들이다.
- 런타임이 낸 Java 예외(`java_exceptions`)가 있는 행은 셋이다.
  - `a0436eb1ddbb`: `ArrayIndexOutOfBoundsException: 13 > 12` 가 P 755 · P2 2514 번 나왔다. 0525 의 150초 프로브는 0 이었으므로 놀이 중 더 뒤에서 시작된다. 지난 핀(`a90f1472`)의 P 에서도 2430 번이었다(0525 원장 note). 같은 서명이 핀을 건너 이어진다. **이 행 하나뿐이라** 할 일 4(«같은 서명 2행 이상이면 고친다»)에 해당하지 않는다. 던지는 자리와, 날짜가 안 가는 것과 관계가 있는지는 재지 않았다(worklog 카드).
  - `fb80e97cbc57`: `NoClassDefFoundError: i` 6번. 0525 §2⑶ 에서 이미 «jar 에 없는 클래스를 게임이 있는지 묻는 것»으로 판정했다.
  - `52edc54e2b23`: `FileNotFoundException` 2번. 게임이 처리한다(이어서 정상 진행).
- ⇒ 할 일 4: 해당 없음. 엔진은 고치지 않았다.

### 4. 전/후

- `progress=stuck` **91 → 90**(compat 실측 `grep -c '"progress": "stuck"'`). compat 변경 1행(`fe76e641bb3d` `stuck` → `ok`) · 이용자 소식 1건.
- `node scripts/check-compat-revert.mjs`: `compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 1`. 표의 «compat 반영» 1행과 같다.
- 0525 ⒟ `confirm-only-entry` 15행의 원인 칸은 «미판정»에서 «넘는 키 미발견 14 + 해소 1»로 바뀐다. 이 표가 그 기록이다(0525 원장은 착지한 파일이라 고치지 않았다).
- 다음에 줄일 길: 행마다 레시피를 찾는 일이다(0507 방식). 끝 화면이 «어느 메뉴/어느 길»인지는 위 표가 행마다 적는다.

### 5. 게이트 · 자원

- 바뀐 것은 이 문서 · worklog 1개 · 계약 문서 · `compat.json` 1행 · 이용자 소식 1건이다. 엔진 코드 변경은 0이다. cargo 4종은 첫 판(코드 0)에서 돌린 결과이고, 수정 판도 코드 0이라 다시 돌리지 않았다.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` · `cargo +beta clippy --all -- -D warnings` 모두 오류 없이 끝났다(build-slot).
- `RUST_MIN_STACK=4194304 cargo test --all` **837 passed / 0 failed**(build-slot).
- `check-worklog-json` · `check-docs-report-serial`(0551 을 #584 가 먼저 잡아 0552 로 옮김) · `check-doc-liveness-parity` · `player-data` OK.
- 에뮬레이터는 census 의 `--jobs 3` 으로만 돌렸다. 단발 프로브는 0회다. `nohup` 은 0이고, 회차 끝에 내 `wv_2e66f9d6` 프로세스는 0이다.
- 유입(`node scripts/corpus-name-inflow.mjs`, 이 브랜치 대 `origin/main`): BOUNDED 718회/330쌍 · SUFFIX-ATTACHED 36회/16쌍이다. 두 바구니 모두 바뀐 파일 «전체»를 훑은 수이고, 걸린 것은 전부 `docs/player-data/compat.json` 의 기존 `title`·`fileTitle` 칸이다(0526 · 0543 과 같은 수). 이 회차가 그 파일에서 바꾼 것은 `progress` 값 1개다. 그 밖 파일의 걸림은 0이다. 첫 판의 «BOUNDED 0 · SUFFIX-ATTACHED 0»은 compat 을 건드리지 않았을 때의 수다. 회차 문서·worklog·소식은 타이틀을 sha12/sha256 으로만 적었다.

<!-- corpus-name-inflow v1 subjects=5 tree=9c2880b39532c9e6 B=718/330 P=0/0 S=36/16 -->
