## [2026-11-22] 사라진 게임사 서버의 답을 만드는 장치 둘을 하나로 — KTF 1행을 `fake_server` 표로 옮기고 KTF 쪽은 호출 모양만 남겼다 (wie-ktf-local-net-fold-into-fake-server)

**무엇을**
- `wie-backend/src/fake_server.rs` 의 `SERVERS` 가 이제 유일한 표다. 7행(LGT 5종)에 KTF `f44271803135`(AID `0103A2B4` · `211.110.18.253:8503`) 1행과 그 응답 함수 `download_list_nothing` 을 옮겨 8행 · AID 6개가 됐다. 응답 바이트는 그대로다(`[3, 0, 0xf9, 0x2a, 0]`).
- `wie-ktf/src/runtime/wipi_c/local_net.rs` 에는 표와 프로세스 전역 대기 버퍼(`static PENDING`)가 없어졌다. 남은 것은 KTF 가 그 표를 부르는 모양뿐이고, 소켓 상태는 LGT 와 같은 `System` 별 `FakeNet` 에 산다. `FakeNet` 에 더한 것은 `socket_at(fd)` 하나다(호출자가 고른 fd 자리에 새 소켓).
- 장부 `docs/contracts/runtime-adjustments.tsv`: KTF 행의 `device` 를 `ktf-local-servers` → `fake-servers` 로 옮겼다. 줄 수는 54 그대로이고 엔진 표는 11 → 10 이다. `test-utils/src/ledger.rs` 의 `DEVICES` 도 10개다. 그 행의 나머지 9칸은 그대로다.

**왜** — 0541 §5 가 «합칠 자리»로 적어 둔 일이다(표와 응답 함수는 `wie-backend` 로, 대기 버퍼는 `System` 별로). 운영자 지시 2026-10-26 의 발권 줄에 있던 가짜 응답 장치 단일화가 `origin/main` 에 두 장치로 남아 있었고, 전수 조사(2026-11-16)가 7번 · 8번 경로로 따로 세었다.

**사용자 영향** — 없다. 6종 모두 합치기 전과 같은 지점까지 간다(§3). `compat.json` 은 한 바이트도 바뀌지 않았다(`runtimeAdjustments` 의 `local-server` 6행 그대로).

### 1. 합친 것과 남긴 것

| | 합치기 전 | 합친 뒤 |
|---|---|---|
| 표 | `fake_server.rs` 7행 + `local_net.rs` 1행 | `fake_server.rs` 8행 |
| 응답 함수 | 두 파일 · 두 가지 시그니처 | `fake_server.rs` 한 곳 · `Respond` 하나 |
| 소켓 상태 | LGT `System` 별 `FakeNet` · KTF 프로세스 전역 정적 버퍼 | 둘 다 `System` 별 `FakeNet` |
| `MC_netConnect` | LGT `api::net::connect` · KTF `local_net::connect` | 둘 다 `api::net::connect` |
| 장부 `device` | 2종 | 1종 |

KTF 쪽에 남긴 것은 6개 함수(`socket` · `socket_open` · `socket_send` · `socket_recv` · `set_read_cb` · `socket_close`)다. `api::net` 의 `MC_netSocket*`(LGT 모양)로 보내면 이 판이 보는 값이 바뀌기 때문이다.

| 남긴 차이 | KTF(이 판이 재어진 값) | LGT 모양으로 보내면 |
|---|---|---|
| 호출 자리 · 인자 | 확장 슬롯 30(연결 · 인자 6개) · 31 · 32 | `MC_netSocketConnect`(인자 5개) · `Write` · `Read` — KTF 표에서 이 세 자리는 여전히 치명 stub 다 |
| 연결 콜백 | 1ms 뒤 | 바로 |
| 표 밖 fd | `Unimplemented`(치명) | -1 |
| `MC_netSetReadCB` | 0 을 돌려주고 콜백을 부르지 않는다 | 응답이 쌓이면 콜백을 부른다 |
| fd 값 | 항상 `0x4c4e` · `MC_netSocket` 을 다시 부르면 같은 자리에 새 소켓 | 0 부터 하나씩 늘어난다 |
| 닫기(슬롯 6 · 33) | -1(종전 stub 의 답) · 소켓은 다음 `MC_netSocket` 까지 남는다 | 0 · 소켓 제거 |

닫기의 -1 은 일부러 남겼다. 이 판은 응답을 받은 직후 닫기를 부른다(측정한 호출 순서의 4번째 줄). 종전에는 KTF 소켓이 `FakeNet` 밖에 있어서 공용 `socket_close` 가 -1 을 답했고, 0541 의 측정은 전부 그 값 위에서 이루어졌다.

### 2. 게임이 보는 값 — 호출별 대조

| 호출 | 전 | 후 |
|---|---|---|
| `MC_netConnect`(표에 있는 판) | 0 반환 · 1ms 뒤 `cb(0, param)` | 같다 |
| `MC_netSocket` | `0x4c4e` · 대기 버퍼 비움 | `0x4c4e` · 새 소켓 |
| 슬롯 30 — 제 호스트:포트 | 0 반환 · 1ms 뒤 `cb(fd, 0, param)` | 같다 |
| 슬롯 30 — 다른 호스트:포트 | 0 반환 · 1ms 뒤 `cb(fd, -1, param)` | 같다 |
| 슬롯 31 — 목록 요청 `0x2af8` | `len` 반환 · 5바이트 대기 | 같다 |
| 슬롯 31 — 다른 요청 · 4바이트 미만 | `len` 반환 · 응답 없음 · 다음 쓰기에 남기지 않음 | 같다 |
| 슬롯 32 | 대기 바이트 또는 -19 | 같다 |
| 슬롯 13 | 0 · 콜백 없음 | 같다 |
| 슬롯 6 · 33 | -1 | -1 |
| 표 밖 KTF 판의 위 호출 전부 | `MC_netConnect` 실패 콜백 · `MC_netSocket` -1 · 슬롯은 `Unimplemented` | 같다 |

달라진 곳은 측정 경로 밖 둘이다. ⑴ 연결에 성공하기 «전»이나 거절된 «뒤»의 쓰기: 전에는 목록 요청이면 응답을 쌓았고, 지금은 쌓지 않는다(`FakeNet` 은 연결된 소켓에만 답한다 — LGT 5종이 쓰는 규칙이다). 이 판은 성공 콜백 뒤에만 쓴다. ⑵ 로그 문구 3줄(`MC_netConnect` 의 모듈 경로와 «local server» → «callback M_E_SUCCESS (fake server)» · 슬롯 31 의 «reply Some(5)» → «reply queued» · 닫기 줄의 모듈 경로). 전수 도구의 접속 횟수 정규식(`MC_netConnect`)에 걸리는 줄 수는 같다.

### 3. 전후 측정 — 6종 · 같은 명령 · 같은 지표

두 릴리스 바이너리(전 = `origin/main` `524c400b1` · 후 = 그 위 이 변경 1커밋 `0738d2d16`)를 쌍마다 «동시에» 띄웠다(같은 부하 분). 인자는 전수 도구의 프로브 그대로다: A `--inject --keep-timeout --timeout 30 --max-ticks 100000000000 --relaunch 1 --pacing 8`(정책 키 27개) · B 는 `--inject-keys 0 --shot-every 1`(키 0). KTF 판은 레시피 L(`game_lab/recipes/f44271803135-newgame.keys` · 180초 · `--relaunch 1`)을 더 쟀다. 코퍼스 파일은 그대로 읽기만 했다. 임대는 판마다 `build-slot run --long` 1건이다(동시 실행 2).

| 판 | 프로브 | 쌍 | 판정 같음 | 통신 호출 순서 같음 | DB 통계 같음 | 전의 결과 | 후의 결과 |
|---|---|---|---|---|---|---|---|
| `f44271803135` (KTF) | A | 7 | 6/7 | 7/7 | 7/7 | PASS 2 · UNMEASURED(clean exit) 5 | PASS 3 · UNMEASURED(clean exit) 4 |
| 〃 | B | 4 | 4/4 | 4/4(접속 0) | 4/4 | PASS 4 | PASS 4 |
| 〃 | L | 3 | 3/3 | 3/3 | 3/3 | PASS 3 · 키 11/11 | PASS 3 · 키 11/11 |
| `caf9d76ffd13` (LGT) | A | 7 | 7/7 | 6/7 | 7/7 | PASS 7 | PASS 7 |
| 〃 | B | 4 | 4/4 | 4/4(접속 0) | 4/4 | PASS 4 | PASS 4 |
| `b9bfcaf42722` (LGT) | A · B | 4 · 4 | 8/8 | 8/8 | 8/8 | PASS 8 | PASS 8 |
| `0093012b8c36` (LGT) | A · B | 4 · 4 | 8/8 | 8/8 | 8/8 | PASS 8 | PASS 8 |
| `3ff5948e235e` (LGT) | A · B | 4 · 4 | 8/8 | 8/8 | 8/8 | PASS 8 | PASS 8 |
| `0fdf45ec169f` (LGT) | A · B | 4 · 4 | 8/8 | 8/8 | 8/8 | PASS 8 | PASS 8 |

합계 57쌍 · 판정 같음 56 · 통신 호출 순서 같음 56 · 실패(FAIL) 0 · load1 20~117.

- KTF 판의 통신 호출은 10쌍(A 7 · L 3) 전부 전후가 같은 5줄이다: `MC_netConnect(0x104de9, 0xa)` 성공 → 슬롯 30 `(0xfd126ed3, 8503)` 연결 → 슬롯 31 158바이트에 응답 → `MC_netSocketClose(19534)` → `MC_netClose()`. 접속 1회, 다시 켠 뒤 접속 0회.
- **갈린 쌍 1 — KTF A 한 쌍(전 UNMEASURED · 후 PASS).** 이 판의 프로브 A 는 한 빌드 안에서도 둘로 갈린다. 다시 켠 뒤 처음 메뉴에서 정책 키가 «이어하기»에 떨어지면 게임이 저장 검사로 스스로 끝나고(clean exit · 0541 §3-4), 아니면 마감까지 간다. 전 빌드만 세어도 PASS 2 · UNMEASURED 5 다. 같은 바이너리 둘을 짝지은 대조 판(아래)에서는 둘 다 UNMEASURED 였다. 키 경로가 고정된 L 은 3/3 이 같다.
- **갈린 쌍 2 — `caf9d76ffd13` A 첫 판(전 7줄 · 후 5줄).** 후 쪽이 같은 순서의 앞 5줄에서 30초가 끝났다(색 512 대 20 · load1 104). 다시 잰 6쌍(기동 순서를 바꾼 3쌍 포함)은 전부 7줄 대 7줄이고 색도 512 대 512 다. 이 판의 호출 경로(`api::net` · `login_0102`)는 이 PR 이 고치지 않았다.
- **대조 판(같은 전 빌드 둘 · A 6쌍):** 판정 6/6 · 통신 호출 6/6 같음, 마지막 프레임은 3/6 만 같았다. 마지막 프레임 일치는 움직이는 화면에서 같은 바이너리끼리도 절반이라 판정 근거로 쓰지 않았다(B 는 전후 17/18 일치, L 은 프레임을 남긴 2쌍 모두 일치).

원자료(쌍별 표 · 통신 호출 줄 · 판정 JSON · 프레임 해시): `~/orchestrator/reports/evidence/wie-ktf-local-net-fold-into-fake-server/`. 프레임 그림은 남기지 않았다.

### 4. 장부 · 가드 · 지원 목록

- 장부: 주석 밖 줄을 `device` 칸만 빼고 `origin/main` 과 `diff` → 차이 0. `device` 별 줄 수는 `fake-servers` 5 → 6 · `ktf-local-servers` 1 → 0 이고 나머지 9종은 그대로다.
- 가드: `fake_server::tests::the_ledger_lists_exactly_this_table` 이 8행의 AID 6개와 장부의 `fake-servers` 6줄을 양방향으로 본다. `ledger::tests::the_ledger_names_only_tables_the_guard_reads` 는 장부에 `DEVICES` 10종만 있는지 본다. 되돌려 보기: 장부 줄만 `ktf-local-servers` 로 되돌리면 두 시험이 red(§5).
- 새 시험 2개: `download_list_nothing_answers_each_write_alone`(종전 `local_net` 의 응답 시험을 `FakeNet` 을 거치는 길로 옮기고, 연결 전 쓰기 · 다른 포트 거절 · 쓰기마다 따로 판정 · `MC_netSocket` 재호출이 남은 응답을 버리는 것을 더했다) · `each_connect_has_one_row`(같은 (AID, 호스트, 포트)가 두 번 있으면 뒤 행이 영영 답하지 못한다).
- `compat.json`: `git diff origin/main -- docs/player-data/compat.json` 0줄. `node scripts/player-data.mjs` → `429 titles · … · runtimeAdjustments on 49 rows (ledger 54 rows)`. `local-server` 가 실린 행은 6개 그대로다.

### 5. 되돌려 보기(실측 · 둘 다 원복)

| 되돌림 | 결과 |
|---|---|
| 장부의 KTF 줄만 `ktf-local-servers` 로 | `the_ledger_lists_exactly_this_table`(wie-backend) red · `the_ledger_names_only_tables_the_guard_reads` red |
| `SERVERS` 에서 KTF 행 제거 | `the_ledger_lists_exactly_this_table` red(장부에만 있는 `0103A2B4`) · `download_list_nothing_answers_each_write_alone` red |

닫기의 -1 은 되돌려 보지 않았다 — 그 값을 보는 시험이 없다(한계 §7).

### 6. 게이트

`origin/main` 이 작업 중 `07c92f5ab`(#596 · #597 착지)로 나아가 머지로 당겼다(충돌 0). 아래는 전부 당긴 트리에서 돌렸다.

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | rc=0 |
| `cargo clippy --all -- -D warnings` | rc=0(`wie-backend` · `test-utils` · `wie-ktf` 를 다시 검사시켜 확인) |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=0 — 이 줄은 루트 패키지만 봐서 고친 두 crate 를 지나지 않는다. `-p wie_featurephone` 을 붙여 따로 돌렸다: `wie-backend` · `wie-ktf` · `wie_featurephone` 검사 · rc=0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | rc=0 · 856 통과 · 0 실패 |
| `cargo +beta clippy --all -- -D warnings` | rc=0. 매니페스트 경고 9줄(`unused dependency` · `wie_cli` 7 · `test-utils` 1 · `wie-util` 1)은 이 PR 이전부터 있고 rc 를 바꾸지 않는다. PR 의 beta 다리는 빈 통과이므로 beta 는 이 로컬 실행이 유일한 근거다 |
| 러너 블록 | `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS rc=0 · `keydraw_ktf.zip` · `keydraw_lgt.zip` `--inject --expect-last-frame` PASS rc=0(키 27/27 · paints 79 · 55) · `text_j2me.jar` `--timeout 5` PASS rc=0 · load1 47. 픽스처 재생성 뒤 `git status` 변화 0 |
| 그 밖 | `node scripts/player-data.mjs` OK · `--selftest` 59 · `check-compat-revert`(착지 기준 바뀐 행 0) · `check-docs-report-serial`(열린 PR claim 충돌 0) · `check-worklog-json` · `check-doc-liveness-parity` · `npm run audit` — 전부 rc=0 |

### 7. 한계

- 닫기의 -1 은 시험이 지키지 않는다. `wie-ktf` 에는 `WIPICContext` 시험 문맥이 없어(`wie-wipi-c` 의 `TestContext` 는 그 crate 의 `cfg(test)` 안에 있다) KTF 쪽 6개 함수는 합치기 전과 같이 단위 시험이 없고, 근거는 §3 의 10쌍이다.
- 닫기가 0 을 답해도 이 판이 같은 지점에 가는지는 재지 않았다. 0 으로 바꾸는 것은 동작 변경이라 이 회차의 범위 밖이다.
- 측정은 `wie_validate`(네이티브 · 벽시계)다. 브라우저 호스트에서는 재지 않았다. 고친 코드는 두 호스트가 같이 쓰는 crate 이고 `System` 별 상태로 옮긴 것이 호스트에 따라 달라질 자리는 없다.
- §3 의 load1 은 20~117 이었다. 같은 쌍을 동시에 띄워 부하를 나눴지만, 갈린 2쌍의 귀속은 재측 횟수(6쌍 · 대조 6쌍)에 기대고 있다.

### 8. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치가 `origin/main` 대비 바꾼 파일 6개): 유입 44회 / 37쌍(BOUNDED) · 판단 필요 0건(SUFFIX-ATTACHED). 전부 장부 `docs/contracts/runtime-adjustments.tsv` 의 `title` 칸에 원래 있던 값이다 — 이 PR 은 그 파일에서 주석 3줄과 한 줄의 `device` 칸만 고쳤다. 이 문서와 코드 4파일이 더한 줄에서는 0 이다(판을 sha 앞 12자로만 부른다).
