## [2026-10-26] KTF f44271803135 «CONNECTING» 벽 — ⒝ 절차였다: 다운로드 서버를 엔진 안에서 «받을 것 없음»으로 답함 · NULL 이미지 크기 0 (wie-ktf-dragoneyes2-connecting-wall-offline-path)

**무엇을**
- `wie-ktf` `runtime/wipi_c/local_net.rs`(새 파일): 타이틀(AID)로 고르는 «죽은 서버» 표. 표에 있는 타이틀만 `MC_netConnect` 가 성공하고 `MC_netSocket` 이 fd 를 받으며, Net 슬롯 30(연결)·31(쓰기)·32(읽기)·13(`MC_netSetReadCB`)이 그 표의 응답과만 말한다. 표 밖 타이틀은 전과 같은 실패(`wie_wipi_c::api::net`). 행은 1개 — f44271803135(`0103A2B4`) · `211.110.18.253:8503` · 목록 요청 `0x2af8` → 빈 목록 `0x2af9`.
- `wie-wipi-c` `MC_grpGetImageProperty`·`MC_grpDrawImage`(KTF 경로 `draw_image_with_context_pixel_op` 포함): NULL 이미지는 크기 0, 그리기 없음. 게스트 쪽 null 읽기 규칙(0391)의 호스트 쪽 짝.
- 전수 도구: 통신망 벽 주석(이 판이 예시)에 «0541 부터 로컬 서버가 답함» · 이 판의 `HAND_NOTE` 2줄 · `HAND_TIP` 1줄. `compat.json` 1행.

**왜**: 운영자 지시(2026-10-26) — KTF 미플레이 5종 중 이 1종. 0416 이 남긴 질문 ⒜⒝⒞ 에 답이 없었다.

**사용자 영향**: f44271803135 `limited` → **`playable`**(sound no → ok · `progress` 키 제거 — 옛 엔진의 `stuck` 이었고 진도 축은 다시 재지 않았다). 처음 실행은 게임 자신의 «재시작» 안내로 한 번 꺼지고, 다시 켜면 처음 메뉴 → «새로하기» → 이야기·첫 맵으로 간다. KTF playable 264 → 265.

### 1. 판정 — ⒝(절차) · 근거는 이미지 주소

이미지 `client.bin259716`(Thumb · 적재 0x100000 · sl = 0x17aa4c).

| 무엇 | 주소 | 읽은 것 |
|---|---|---|
| 부팅 분기 | 0x1052a4 | `bl 0x13cf18`(DB `option` 읽기) 뒤 «다운로드 완료» 바이트 `*(sl+0x108)` ≠ 0 이면 0x1052b4 정상 시작, 0 이면 다운로드 확인 0x105308 |
| «0K» 계산 | 0x103b5c | 8파일마다 `max(0, 기대 − 실제)` 합. ★반환값은 «필요 여부»가 아니라 **빈 공간 검사**(합 + 10240 ≤ `MC_fsAvailable`) — 그래서 0K 여도 접속한다 |
| 기대 크기 | 0x1721f8..0x172214 | 이미지 초기화 데이터에 **박혀 있는** 8개 값 = 동봉 `P/` 8파일 크기와 바이트 단위로 같다 |
| 접속 루프 | 0x104f08 · 0x104de8 | 상태 9(포트 8503)는 **타임아웃이 없다**: 「접속 중」 바이트가 0 이면 다시 `MC_netConnect`. 실패 콜백(`M_E_ERROR`)은 그 바이트만 0 으로 → 0416 의 99회 |
| 응답 파서 | 0x101160 → 0x1025b6 | `[len u16 LE][id u16 LE]…`. `0x2af9` = `[n u8]` 뒤 `[이름 길이 u8][이름][크기 u32 LE]` × n — 이름은 8개 `P/` 이름과만 대조(0xdc 표) |
| 완료 판정 | 0x1028a4 | 기대 크기가 모두 0 아님 + 실제 ≥ 기대 → `*(sl+0x108)=1`, `option` 저장(0x13d09c), 상태 3, 화면 종료값 `e4=3` |

- ⒜ 아님: 서버가 줄 수 있는 것은 그 8개 `P/` 파일뿐이고, 동봉본이 이미 기대 크기와 같다. 지어낸 것은 «목록에 0개» 1바이트이지 게임 자산 바이트가 아니다.
- ⒞ 아님: 진단 빌드(커밋 0)로 «PPP 성공 · 소켓 실패»를 주면 오류 18(0x12) 갈래로 가지만 화면은 «DOWNLOADING.» 에서 서고(27장 동일), 오류 키 처리기(0x10cc68 · 코드 18 → 0x10ccd2)는 재시도/취소뿐이다. 완료 바이트를 세우는 길은 서버 목록을 받은 뒤 하나뿐이다.
- 실기에서는: 다운로드가 끝난 단말은 `option` 에 완료 바이트가 있어 다시 접속하지 않는다. 이 덤프에는 그 기록이 없다(동봉은 `P/` 8파일 + `list`).

### 2. `1.dat`·`4.dat` — 다운로드와 무관한 글 묶음

- 0x107540(`bank`, `index`)가 `"/res/%d.dat"` 를 `MC_knlGetResourceID`/`MC_knlGetResource` 로 읽어 길이 접두 문자열 `index` 번째를 꺼낸다 — 대사·글 묶음. 다운로드 목록(위 0xdc 표)에 없다.
- 측정: 첫 맵 진입까지(아래 §3 의 키 경로) 자원 요청 64종 전부 jar 에 있었고 `.dat` 은 `/res/5.dat` 과 `saveData0..3` 뿐 — `1.dat`·`4.dat` 은 한 번도 요청되지 않았다. 이 회차가 간 길에서는 벽이 아니다. 그 뒤 장면이 요청하는지는 재지 않았다.
- 코퍼스: `game_lab` 의 500KB 넘는 zip 전부에서 `AID:0103A2B4` 를 찾음 — 이 덤프 1개뿐.

### 3. 고친 뒤 실측(전부 `wie_validate` 릴리스 · `--relaunch 1`)

1. 첫 실행: «추가 데이터 0K… OK» → OK → `MC_netConnect` 1회 → 슬롯 30 `211.110.18.253:8503` → 슬롯 31 요청 158바이트 → 응답 5바이트 → 게임 자신의 «안정성을 위해 단말기를 재시작 해야 됩니다» → OK → 정상 종료.
2. 다시 실행(DB 유지 — `option` 은 DB 다): 접속 0회, 등급 화면 → 처음 메뉴. 메뉴는 첫 키를 먹고, 왼쪽 1번 «새로하기»(2번이면 «게임종료»), 오른쪽은 «환경설정»·«도움말»·«네트워크창고».
3. «새로하기» → 덮어쓰기 확인 «예» → 이야기 → **첫 맵 대화 장면에서 `MC_grpGetImageProperty(NULL)` → `MC_grpDrawImage(NULL)` 로 호스트가 주소 0 읽기 실패**. 만들지 않은 이미지 슬롯(호출부 0x1058cd·0x105949 의 «기준점 그리기» 도우미). 단말은 주소 0 이 읽혔으므로 넘어갔다 — §무엇을 의 NULL 처리 뒤 첫 맵 대화가 이어진다.
4. 처음부터 들어 있는 저장(SLOT 1 · Lv 74)을 «이어하기» 하면 «세이브 데이터가 강제로 변경되어 게임을 종료합니다» 뒤 종료. 게임 자신의 세이브 검사(이미지에 `PHONENUMBER` 가 저장 파일 이름 옆에 있다)다. 고치지 않고 안내 줄로 남겼다.
5. «네트워크창고»: 게임 자신의 «통화료가 부과됩니다» 질문까지만 봤다. 새 접속 0회.

**전수 도구(`census-drive.sh` · 이 판 + 표본 15)**

| 측정 | 결과 |
|---|---|
| 프로브 A·B(전 = origin/main 3f4c8f7a · 후) | 이 판: A 접속 **1220 → 1**, A 정상 종료(첫 실행 재시작 안내 → 다시 실행 1회 → 처음 메뉴의 «이어하기»가 저장 검사로 종료), sound silent → **ok**, speed n/a → ok. load1 52·33 |
| 장시간 L 600 s(기본 키) | 16/900 키에서 정상 종료 — 기본 키가 «이어하기»를 고른다(4번). 실패 줄 0 이라 longplay ok |
| 장시간 L 600 s(레시피 `game_lab/recipes/f44271803135-newgame.keys`) | **마감까지**(stop deadline) · paints 45,405 · 20초 간격 29장 중 27장 서로 다름 · 300 s·580 s 둘 다 이야기 장면 안 |
| `report` | status **playable** · boot/render/input/longplay/sound/speed 전부 ok · load1 40 |

**표본 15종 회귀**(KTF playable: 접속 1~2회 3종 `79d9bc10be82` `1fe6c1c897d7` `d9384b388ea5` + 확장 Net 슬롯 쓰는 2종 `30c7bd6fb01b`(34) `04159045a7ea`(33) + 무작위 10): boot·render·input 변화 0. 첫 판에서 뒤집힌 2건은 같은 쌍을 두 바이너리로 두 번 다시 재서 귀속을 갈랐다 — `1cf2e6076079` sound 는 전·후 둘 다 2/2 ok, `1e43e2e0055f` speed 는 전·후 둘 다 2/2 n/a(속도 축은 부하 소음 · 0321). **회귀 0.**
⇒ 0416 §6 의 «접속 1~5회 30종»은 전 통신사(429종) 기준이다. w6 전수(`~/scratch/w6census` · KTF 269)에서 프로브 A 접속 1~5회인 KTF 는 위 3종뿐이었다.

### 4. 되돌리면(전부 실측)

| 되돌림 | 결과 |
|---|---|
| 표의 행 제거(AID 를 다른 값으로) | 다시 CONNECTING(872KB/872KB) · `MC_netConnect` **2,424회** · 종료 0 · 다시 실행 0 |
| NULL 이미지 처리 둘 다 제거 | 첫 맵 진입에서 `Invalid memory access; address: 0` FAIL(2/2) |
| `MC_grpDrawImage` 쪽만 제거(크기만 0) | 같은 FAIL — `MC_grpGetImageProperty(NULL,5)` 바로 뒤 |
| 단위 시험 | `wie-ktf` `the_list_request_is_answered_with_an_empty_list` · `wie-wipi-c` `a_null_image_has_no_size_and_draws_nothing` |

`wie_validate --relaunch` 를 «파일도 유지»로 바꿀 필요가 있다고 보고 고쳤다가 되돌렸다 — 파일을 매번 새로 만들어도 두 번째 실행이 처음 메뉴에 갔다(`option` 은 DB 이고 `--relaunch` 는 DB 를 이미 유지한다). 브라우저 호스트도 DB·파일 둘 다 다음 실행에 복원한다(`wie_featurephone/src/lib.rs` 저장 상태 내보내기).

### 5. LGT 회차와 합칠 자리

`wie-lgt-server-wall-six-titles-local-net-emulation` 은 이 회차 시작 시 착지 전·열린 PR 0 이었다(`gh pr list` 실측). 그래서 장치는 `wie-ktf` 안에만 두었다. 합칠 자리:
- **표 + 응답 함수**(`LocalServer { aid, addr, port, reply }`)는 통신사 무관이라 `wie-backend`(또는 `wie-wipi-c`) 로 올리고, KTF·LGT 각각의 슬롯 배선만 남긴다.
- **대기 응답 버퍼**는 지금 프로세스 하나(`spin::Mutex` 정적 · `ponytail:` 주석). 두 번째 타이틀이 동시에 소켓을 열 수 있게 되면 `System` 별 상태로 옮긴다.

### 6. 게이트(빌드 전부 `build-slot`)

`cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 (+ `-p wie_featurephone` wasm32 rc=0) · `cargo +beta clippy --all -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(818 통과 · 0 실패). 러너 블록: draw · helloworld ×2 · text PASS, keydraw ktf·lgt `--inject --expect-last-frame` PASS rc=0(paints 79·55). 전수 selftest 65/65.

### 7. 측정 조건

load1 7~294(빌드 대기 중 최고 294 · 전수 프로브 52·33·17~34 · 이 판 프로브 40 · 레시피 L 동안은 따로 재지 않았다 — 끝난 직후 7). 진단(접속 시점 상태 바이트 · SVC LR 기록 · «PPP 성공» 실험)은 작업 트리에서만 넣고 커밋 전에 되돌렸다. 프레임 캡처는 저장소 밖 `~/orchestrator/reports/evidence/wie-ktf-dragoneyes2-connecting-wall-offline-path/`.

### 8. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 331쌍 + SUFFIX-ATTACHED 17쌍. 전부 이 PR 이전부터 있던 값이다 — `compat.json` 기존 제목 값, `graphics.rs` 의 기존 주석 1곳, 전수 도구의 다른 판 요령 1곳. 이 PR 이 더한 줄에서는 0 이다(이 문서·`local_net.rs`·새 안내 3줄 모두 판을 sha 로만 부른다). 티켓 id 의 로마자 표기는 총괄이 정한 이름이라 그대로 두었다.
