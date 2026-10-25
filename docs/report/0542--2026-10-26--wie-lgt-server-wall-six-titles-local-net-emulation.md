## [2026-10-26] LGT 서버 벽 6종 — 엔진 안 가짜 서버로 2종 통과, 4종은 막힌 지점 기록 (wie-lgt-server-wall-six-titles-local-net-emulation)

**무엇을**: 문 닫은 게임사 서버를 엔진 프로세스 안에서 대신 답하는 층(`wie_backend::fake_server`)을 만들고, LGT WIPI-C 소켓 호출(`MC_netConnect`·`Socket`·`SocketConnect`·`SocketWrite`·`SocketRead`·`SetReadCB`·`SetWriteCB`·`SocketClose`)을 그 층에 이었다. 표에 오른 타이틀(AID)과 그 타이틀이 거는 호스트:포트에만 답한다. 표 밖 타이틀은 종전과 바이트 단위로 같은 길(접속 실패 콜백 · 소켓 -1)을 탄다. 함께 `sprintf` 가 `-`(왼쪽 정렬)·`.n`(정밀도)을 못 읽어 서식 문자열을 그대로 내던 결함을 고쳤다.

**왜**: featurephone LGT 탭의 `limited` 6종은 엔진 결함이 아니라 서버 벽이었다(0445·0461·0469·0507). 이번 회차는 판마다 ⒜ 오프라인 경로 → ⒝ 최소 성공 응답 합성 → ⒞ 남김 순서로 가장 싼 길을 골랐다.

**사용자 영향**: `caf9d76ffd13`·`b9bfcaf42722` 가 첫 실행 인증을 지나 본 게임에 들어간다(`limited` → `playable`). LGT 집계는 playable 71 → 73, limited 6 → 4.

### 1. 판별 결과

| sha12 | 결과 | 근거 한 줄 |
|---|---|---|
| `caf9d76ffd13` | 고침(⒝) | 로그인 `0x0102` 에 `0x0103`(결과 1 · 40바이트 계정 기록 · 끝 바이트 1)으로 답하면 «인증이 완료되었습니다» → 타이틀 → 메뉴 |
| `b9bfcaf42722` | 고침(⒝) | 인사(유형 0) → 인증서(유형 0x14, 16바이트) 두 번 답하면 `cert.c2s` 를 쓰고 이야기 화면으로 간다. 다시 켜도 묻지 않는다 |
| `0093012b8c36` | 남김(⒞) | «인증서가 없습니다» 안내의 두 버튼 모두 접속 호출 0회로 게임이 꺼진다. 통신 모듈(121.254.169.67 · `0x6e0f0` «서버에 접속 중 입니다») 까지 가지 않는다 |
| `3ff5948e235e` | 남김(⒞) | 접속·요청까지는 간다(211.239.165.14:7205 · `@A<길이8><10자>A_LGT_…`). 응답 판정(`0x3d668`→`0x51728`)의 성공 조건을 이번 회차에 다 읽지 못했다 |
| `0fdf45ec169f` | 남김(⒞) | `MC_netConnect`(cb `0x1284fd`) 실패 뒤 소켓 없이 fd 0 에 1바이트 쓰기를 되풀이한다(200초에 354회). 소켓 생성 경로를 타지 않아 가짜 서버가 끼어들 자리가 없다 |
| `73f3a21e981c` | 남김(⒞) | aot-java. `org.kwis.msf.io.URL.find` 가 `SchemeNotFoundException` 을 던지고(30초 3회) 게임은 메뉴로 돌아온다. Java 소켓(`Socket`·스트림) 구현과 이름 등록 응답 해독이 함께 필요하다 |

### 2. 고친 두 판 — 응답이 만족시키는 것

**`caf9d76ffd13`** (binary.mod, Thumb). 패킷 = `[전체 길이 u16 LE][명령 u16 LE][본문]`(헤더 4바이트 포함 · `0x19c3c`). 받은 패킷은 `0x17c40` 이 본문 첫 바이트를 결과(`this+0x1760`)로 두고 명령으로 분기한다(`0x168b8`).
- `0x0103` 처리(`0x17a88`)는 결과가 1일 때만 진행한다. 40바이트 기록을 `audio.adt` 에 전화번호와 함께 저장하고(`0x164cc`), 다음 실행의 적재(`0x16674`)는 전화번호만 맞춰 본다.
- 끝 바이트 0 은 «SMS 수신 동의» 질문과 두 번째 접속으로, 1 은 «인증이 완료되었습니다» 로 간다(두 값 실측).
- 서버 접속 30초 시한(`0x7530`)은 실패 콜백이 와도 «서버 접속중» 표시 플래그(`this+0x1759`)를 내리지 않는다 — 실패를 돌려주는 ⒜ 길은 없다.

**`b9bfcaf42722`** (binary.mod, Thumb · Com2uS). 패킷 = `[전체 길이 u16 BE][0][유형][본문]`(`0x498ce`). 첫 실행 인증서 모듈의 처리기는 `0x1e4a8`(설치 `0x1e3b0`)이다. 실측으로 확인했다 — 같은 객체의 다른 처리기 `0x1ebe0`(인증서 화면 진입 시 설치)을 먼저 의심해 유형 4·5 를 보냈으나 반응이 없었고, `[0x150770c+0x44c]` 를 찍어 실제 처리기를 확인했다.
- 연결 → 유형 0 요청 → 유형 0 응답 → 모드 3 이 유형 0x14 요청 → 유형 0x14 응답의 `+6` 길이·`+8` 자료를 받아 `cert.c2s` 를 연다(`0x1e19c`). 실패 시 오류번호 4000(`0xfa0`)이 «인증오류»로 뜨던 그 경로다.

### 3. 개악 대조 (표에서 두 행을 지운 빌드)

같은 키 · 60초 · 두 빌드를 짝으로 돌렸다. 표의 행을 지운 빌드(`SERVERS = []`)는 두 판 모두 접속이 실패하고 종전 벽에서 멈춘다 — `caf9d76ffd13` 은 «서버 접속중», `b9bfcaf42722` 는 타이틀 위 인증서 질문. 붙인 빌드는 둘 다 벽 다음 화면(메뉴·이야기)이다.

캡처(로컬 · `~/orchestrator/reports/evidence/wie-lgt-server-wall-six-titles-local-net-emulation/`):
`fin-new-<sha12>-sheet.png`·`fin-new-<sha12>-t030.png`(붙인 빌드) · `fin-norows-<sha12>-sheet.png`·`fin-norows-<sha12>-t030.png`(지운 빌드).

### 4. 회귀 대조 (LGT playable 표본 14종 + 고친 2종)

census 프로브 A(정책 키 27개 · `--pacing 8`)·B(키 0)를 `origin/main` `3f4c8f7a` 빌드와 이 브랜치 빌드로 같은 묶음 안에서 쟀다. 표본 = clet 10종(sha 순 4칸 간격) + aot-java 4종. 결과 파일: 위 evidence 의 `probes-ab.tsv`.
- 14종 모두 A·B `PASS` · 입력 축 판정이 양쪽에서 같다(13종 `input_ok`, `287af341dac8` 은 양쪽 모두 `input_none` — 이 타이틀은 census 가 `--restart-at 4` 로 재는데 이번 짝에는 그 플래그를 넣지 않았다).
- 고친 2종: A 의 색 수 `caf9d76ffd13` 20 → 512, 그림 수 `b9bfcaf42722` 859 → 1575.
- load1 30~40 대에서 쟀다(다른 레인 census 동시 실행). 표본의 그림 수는 양쪽에서 -45% ~ +49% 흔들렸다 — 그림 수는 판정이 아니고(AGENTS.md «count as a floor»), 판정(PASS·입력 축)은 14종 모두 같았다.

### 5. compat.json

- `caf9d76ffd13`·`b9bfcaf42722`: `status` `limited` → `playable`, `knownIssues_ko` 비움, `axes.progress` 키를 뺐다. 그 `stuck` 은 벽에서 잰 값이고 이번 회차에 진도 정책(1800초)을 다시 재지 않았다 — 계약 §1 의 «키 없음 = 재지 않았다».
- `0093012b8c36`: 안내 문장을 실제 벽(버튼 둘 다 종료)에 맞췄다. census `HAND_WALL` 도 같은 문장이다.
- 다른 4판의 `HAND_WALL` 문장은 실제 벽과 맞아 그대로 두었다.

### 6. 함께 고친 것 — `sprintf` 의 `-`·`.n`

`3ff5948e235e` 의 헤더 `"@A%-8d%10.10s"` 가 서식 그대로 나갔다(`-` 와 `.` 를 «모르는 지정자»로 통째 출력). C 의 뜻대로 왼쪽 정렬과 `%s` 정밀도를 읽게 했다. 이 판의 요청은 이제 `@A51      1911112222A_LGT_…` 이다. `-`·`.n` 을 쓰는 다른 타이틀이 몇이나 되는지는 세지 않았다.

### 7. 한계

- 응답 바이트는 실기 서버의 것이 아니라 «파서가 성공으로 읽는 최소»다. 계정 기록 40바이트·인증서 16바이트는 0이다. 게임이 이후 그 값을 랭킹 등에 보이면 빈 칸으로 보일 수 있다.
- 두 판의 랭킹·추가 통신(표에 없는 명령)은 답하지 않는다 — 그 기능은 종전처럼 시한 뒤 실패한다.
- 진도(progress) 축은 다시 재지 않았다(§5).
- `caf9d76ffd13` 은 `wie_validate` 의 `--relaunch` 재부팅에서 인증 질문을 다시 냈다(예를 누르면 곧바로 다시 완료된다). 저장 파일이 남는 브라우저에서 두 번째 실행이 묻지 않는지는 재지 않았다.

### 8. featurephone 셸 동기

otterpebble `apps/featurephone/lib/compat.json` 은 다음 wie 릴리스 가져오기로 받는다. 이 회차에서 otterpebble 은 건드리지 않았다.

### 9. 게이트

- 네 게이트: fmt rc=0 · clippy `-D warnings` rc=0 · wasm clippy rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(822 통과 · 0 실패). beta clippy rc=0.
- 엔진 러너 블록(릴리스 빌드): `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me` PASS. `keydraw_ktf`·`keydraw_lgt` 는 기본 `--max-ticks` 에서 `UNMEASURED`(`stop` = `max-ticks`, 27키 중 8키) — 릴리스 빌드가 키 일정 전에 틱 상한을 먹은 것이다(AGENTS.md: max-ticks 면 상한을 올린다). `--max-ticks 2000000000` 으로 이 브랜치·`origin/main` 모두 PASS · rc=0 · 27/27 · paints 79/55(main 80/55).

### 10. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(기본 대상 = 이 브랜치의 추가·수정 파일 전체 본문): 유입 339건(BOUNDED) · 판단 필요 18건(SUFFIX-ATTACHED). 거의 전부가 이 회차가 세 행만 고친 `docs/player-data/compat.json` 의 기존 `title`·`fileTitle` 칸이다. 그 파일을 뺀 대상으로는 9건 · 2건이고, 전부 이 회차가 손댄 파일(`system.rs`·`svc_ids.rs`·`wipi_c.rs`)에 원래 있던 주석이다. 이 회차가 더한 줄에는 게임 이름이 없다(추가 줄 대조). 보고서·worklog·코드는 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=12 tree=9f465c735e13f57e B=729/339 P=0/0 S=38/18 -->
