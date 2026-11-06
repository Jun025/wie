## [2026-11-06] 진도 — 0525 ⒟ «확인 키만으로 들어감» 15행 현행 핀 재측 · 15행 모두 «넘는 키 미발견» · compat 변경 0 (wie-progression-stuck-confirm-only-15-remeasure)

**무엇을**: 0525 원장(`0525--…--ledger.csv`)에서 `cluster = confirm-only-entry` 인 15행을 현행 `origin/main` `2e66f9d6` 에서 다시 쟀다. 측정은 `--only progress` 600초 · 정책 v2 · P 와 P2 짝이다. 끝 화면과 stderr 서명으로 막힘의 원인을 «넘는 키 미발견 / 엔진 벽 / 게임 설계» 셋으로 나눴다. **15행 모두 «넘는 키 미발견»이다.** 엔진 벽은 0이다(단일 행 예외 서명 1건은 §3). 게임 설계도 0이다. `ok` 로 뒤집힌 행이 없어서 compat 과 이용자 소식은 바꾸지 않았다.

**왜**: 이 15행의 마지막 측정은 0502(`7ca47380`)와 0507(`26f79198`) 핀이었다. 0525 §2⑶ 에서는 stuck 의 예외 대부분이 «지난 핀 것»이었다. 그래서 이 행들의 막힘도 이미 풀렸는지 현행 핀에서 확인해야 했다(티켓 근거: `wie-state-next-refresh-and-next-round-proposal` 후보 2).

**사용자 영향**: 없다. 지원 현황 · 진도 배지 · 요령 문장은 그대로다.

### 1. 측정 조건

- 엔진: `origin/main` `2e66f9d6` 의 release `wie_validate`. 사본을 저장소 밖 `~/scratch/c15/` 에 두었다.
- 명령: `bash scripts/census-drive.sh --bin … --out … --only progress --progress 600 --as P|P2 --jobs 3 --titles <15행> <15행 코퍼스>`. 15행에 레시피는 쓰지 않았다(«확인 키만» 행이라 정책 자체가 들어가는 길이다).
  - `--only progress` 의 대상은 A·B·L 기록이 있는 행으로 거른다. A·B 는 주간 퇴행 감시 2026-11-01 out 에서, L 은 가장 최근 회차 out(10-06~10-11)에서 옮겼다.
  - `fb80e97cbc57` 은 L 기록이 어디에도 없었다. 그래서 이 핀에서 `--only long --long 600` 을 먼저 쟀다. 결과는 `UNMEASURED`(854/900 키 · deadline)라 longplay 는 `ok` 다.
- 임대: long 1회 + P 3회 + P2 3회. 모두 `census-drive.sh` 의 `run --budget 900` 단위로 잡았다. 스윕 전체를 임대 1건으로 감싸지 않았다.
- 부하: load1 은 9~99 였다(행별 값은 증적 `judge.txt`).
- 증적(화면 모음 · 행별 정체 값)은 저장소 밖 `~/orchestrator/reports/evidence/wie-progression-stuck-confirm-only-15-remeasure/` 에 있다.

### 2. 판정표(15행)

`P / P2` 는 census 의 행별 판정이다(정체 ≥ 200초면 `stuck`). «눈»은 t380~t600 화면 모음이다. 0507 §3 계약(짝 둘 다 `ok` + 눈 확인)을 넘은 행만 `ok` 로 바꾼다.

| sha12 | 플랫폼 | P / P2 | 정체(초) P / P2 | 마지막 3분의 1(눈) | 판정 |
|---|---|---|---|---|---|
| `0cc4ef7ede37` | KTF | stuck / stuck | 540 / 540 | 사격 연습장. 점수가 100 에서 멈췄고 과녁이 남아 있다 | 넘는 키 미발견(조준) |
| `1cdea1985955` | KTF | stuck / stuck | 310 / 220 | 마을 끝에서 «출입이 불가능합니다» 대사 | 넘는 키 미발견(다른 길) |
| `3151fdc167b6` | KTF | stuck / stuck | 340 / 340 | 능력치 화면(«레벨업 포인트 없음»)과 거점 메뉴를 오간다 | 넘는 키 미발견(거점 메뉴의 «스테이지 이동») |
| `44c292be4e4c` | KTF | stuck / stuck | 260 / 260 | 방 하나에서 상태창과 지도를 오간다 · 퀘스트 «…진입» | 넘는 키 미발견(출구) |
| `6af589a88cf9` | KTF | stuck / stuck | 300 / 300 | 저장 칸 화면에서 덮어쓰기를 되풀이한다(db 쓰기 236) | 넘는 키 미발견 |
| `a20c2044305c` | KTF | stuck / stuck | 320 / 320 | 들판 위 능력치 배분 창(«GP가 없습니다») | 넘는 키 미발견 |
| `fe76e641bb3d` | LGT | **ok** / stuck | 40 / 430 | P 는 방을 나와 거리로 가서 돈이 오른다. P2 는 내내 방 안이다. 둘 다 시계는 간다 | 넘는 키 미발견(방 문) — 짝 불일치라 stuck 그대로 |
| `ecb1bf8d2079` | LGT | stuck / stuck | 440 / 440 | 메뉴의 랭킹 → «네트워크 이용시 통화료가 부과됩니다» 확인 창 반복 | 넘는 키 미발견(메뉴 항목) |
| `a0436eb1ddbb` | SKT | **ok** / **ok** | 40 / 80 | 놀이기구 상점에서 «구입비용이 부족합니다»를 되풀이하고, 날짜 표시가 그대로다 | 넘는 키 미발견 — 눈 확인 불합격이라 stuck 그대로 · §3 |
| `fb80e97cbc57` | SKT | stuck / stuck | 440 / 440 | 지도 화면을 되풀이해 연다 | 넘는 키 미발견 |
| `66959afab216` | SKT | stuck / stuck | 360 / 280 | 들판과 상태창을 오간다 | 넘는 키 미발견 |
| `bf54c05e58a9` | SKT | **ok** / **ok** | 160 / 160 | 같은 방에서 같은 대사 두 줄이 되풀이된다 | 넘는 키 미발견 — 눈 확인 불합격이라 stuck 그대로 |
| `52edc54e2b23` | SKT | **ok** / **ok** | 10 / 30 | 마을 오른쪽 끝에 선 채 지도 창만 열린다 · `1cdea1985955` 와 같은 자리 | 넘는 키 미발견 — 눈 확인 불합격이라 stuck 그대로 |
| `6e93f26fa2f5` | SKT | **ok** / stuck | 120 / 440 | 장비 · 코인 메뉴 | 넘는 키 미발견 — 짝 불일치라 stuck 그대로 |
| `e3276ce8557c` | SKT | stuck / stuck | 300 / 300 | 육상 트랙 경기 중 일시정지 메뉴(«1.계속하기») | 넘는 키 미발견(종목 조작) |

- 15 = 넘는 키 미발견 15 · 엔진 벽 0 · 게임 설계 0. 미측정 0.
- 짝 둘 다 `ok` 는 3행(`a0436eb1ddbb` · `bf54c05e58a9` · `52edc54e2b23`)이다. 셋 다 눈 확인에서 떨어졌다. `3b5b98afa890`(0507 §1)과 같은 꼴로, 커서 · 창 · 배경이 움직여서 새 화면으로 세졌을 뿐이다.
- ★15행 중 9행은 P 와 P2 의 정체 값이 **같다**(새 화면 수도 같거나 1 차이). 정책 v2 가 결정적이어서 짝은 같은 판을 다시 돌린 셈이다. 그러니 이 행들에서 «P2 가 동의했다»는 독립 증거가 아니다. 판정을 받치는 것은 눈 확인이다.
- `52edc54e2b23` 들판 배경의 넓은 마젠타 면은 0460 §관찰 그대로다(10-05 out 에도 있다). 진도와는 따로 있는 렌더 문제이고 이 회차는 재지 않았다.

### 3. stderr · 예외 서명 — 2행 이상 겹치는 엔진 서명 없음

- 15행 모두 P · P2 의 `uncaught_threads` 0 이고, stderr 꼬리(200줄)의 `Exception|panic` 도 0 이다. `ERROR` 는 P2 에서 `ecb1bf8d2079` · `fe76e641bb3d` 의 `MC_netConnect … M_E_ERROR (no network)` 뿐이다. `ecb1bf8d2079` 은 위 랭킹 창의 접속이고, 둘 다 `net_connects` 1~2 다. 스텁 상위는 `Font::getDefaultFont`/`Font::<init>` · `MC_grpGetFont` 로, 0525 §2⑴ 이 모집단과 비율이 같다고 잰 서명들이다.
- 런타임이 낸 Java 예외(`java_exceptions`)가 있는 행은 셋이다.
  - `a0436eb1ddbb`: `ArrayIndexOutOfBoundsException: 13 > 12` 가 P 755 · P2 2514 번 나왔다. 0525 의 150초 프로브는 0 이었으므로 놀이 중 더 뒤에서 시작된다. **이 행 하나뿐이라** 할 일 4(«같은 서명 2행 이상이면 고친다»)에 해당하지 않는다. 던지는 자리와, 날짜가 안 가는 것과 관계가 있는지는 재지 않았다(worklog 카드).
  - `fb80e97cbc57`: `NoClassDefFoundError: i` 6번. 0525 §2⑶ 에서 이미 «jar 에 없는 클래스를 게임이 있는지 묻는 것»으로 판정했다.
  - `52edc54e2b23`: `FileNotFoundException` 2번. 게임이 처리한다(이어서 정상 진행).
- ⇒ 할 일 4: 해당 없음. 엔진은 고치지 않았다.

### 4. 전/후

- `progress=stuck` **91 → 91**. compat 변경 0행 · 이용자 소식 0건.
- 0525 ⒟ `confirm-only-entry` 15행의 원인 칸은 «미판정» → «넘는 키 미발견» 15 로 바뀐다. 이 표가 그 기록이다(0525 원장은 착지한 파일이라 고치지 않았다).
- 다음에 줄일 길: 행마다 레시피를 찾는 일이다(0507 방식). 끝 화면이 «어느 메뉴/어느 길»인지는 위 표가 행마다 적는다.

### 5. 게이트 · 자원

- 바뀐 것은 이 문서와 worklog 1개다. 엔진 코드 변경은 0이다.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` · `cargo +beta clippy --all -- -D warnings` 모두 오류 없이 끝났다(build-slot).
- `RUST_MIN_STACK=4194304 cargo test --all` **837 passed / 0 failed**(build-slot).
- `check-worklog-json` · `check-docs-report-serial`(0551 을 #584 가 먼저 잡아 0552 로 옮김) · `check-doc-liveness-parity` · `player-data` OK.
- 에뮬레이터는 census 의 `--jobs 3` 으로만 돌렸다. 단발 프로브는 0회다. `nohup` 은 0이고, 회차 끝에 내 `wv_2e66f9d6` 프로세스는 0이다.
- 유입(`node scripts/corpus-name-inflow.mjs`): BOUNDED 0 · SUFFIX-ATTACHED 0.
