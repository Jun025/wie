## [2026-10-12] `org.kwis.msf.io.Network.connect()` 가 0 을 돌려준다 — 짝 전수 162종 퇴행 0 (wie-msf-network-connect-zero-census-probe)

제안 `2026-10-11-lgt-1a69522a7d43-nested-paint-host-panic#p0` 채택. 대상 = jar 에 `org/kwis/msf/io/Network` 가 있는 160종(KTF 143 · LGT 16 · SKT 1) + 가드 `49ade89578c5` `ddd885583b15` = 162. 핀 `1c06afa6`.

**무엇을**: `Network.connect()` 의 반환값을 -1 → 0. 네트워크 미지원은 그대로 소켓 열기(`URL.find` → `SchemeNotFoundException` «Network is not supported»)에서 실패한다.
**왜**: -1 을 «스트림 없이 계속»으로 읽는 타이틀(`1a69522a7d43`)은 null 스트림에 쓰다 통신 스레드를 잃고 안내 화면에서 멈췄다(0511 §3). 0 이면 실패가 게임이 잡는 예외로 오고, 게임 자신의 실패 안내가 뜬다.
**사용자 영향**: `1a69522a7d43` 의 온라인 등록 «예» → «연결에 실패하였습니다» → 확인 → 주 메뉴. 다른 통신 타이틀은 같은 안내에 같거나 더 빨리 닿는다(§3).

### 1. 짝 결과 — census 6축 probe(162종 · 두 빌드 각 1회)

전수 표 = `0514--…--pair.csv`(sha12 · 162행). `--only probe`(probe A 27키 + B 0키, 30초) · `scripts/census-drive.sh` · `--jobs 3`.

| | base(-1) | zero(0) |
|---|---|---|
| status | limited 162(longplay 미실행) | limited 162 |
| 축이 바뀐 타이틀 | — | 3 · 전부 나아짐(아래) · 나빠진 축 0 |
| census `netWall`(probe A `net_connects` ≥ 200) | 1 (`513d4b49c0be` 386) | **0** (`513d4b49c0be` 3) |

- `513d4b49c0be`: speed n/a → ok. 이 회차에서 실제로 바뀐 유일한 것(§3).
- `a10a1f02b41b`(sound silent → ok) · `fd3bf717db21`(speed n/a → ok): 30초 probe 안에서 `Network::connect` 호출 0 → 두 빌드가 같은 길 = 잡음.

### 2. 도달 sweep — connect 를 실제로 부르는 타이틀

30초 probe 에서 `Network::connect` 를 부른 것은 162종 중 1종뿐이다 ⇒ progress 정책 키(census 의 `PROGRESS_KEYS` 3회)로 75초씩 base 를 한 번 더 돌려(162종 · ≤3 동시 · phase 40종 = long 임대 1건) 부르는 타이틀을 찾았다: **12종**. 그 12종 + `1a69522a7d43` `dbd078113b97` 를 zero 로 짝:

- 판정·stop·input_steps 바뀐 것 **0/14**. `java_exceptions` 는 zero 에서 0 → 1~2 가 늘어난다 — 소켓 열기의 `SchemeNotFoundException` 이고 게임이 잡는다(uncaught 0).
- 마지막 프레임이 다른 6종: 두 종(`1a69522a7d43` `513d4b49c0be`)은 이 판에서 connect 0회(정책 키가 그 메뉴에 안 닿음 = 타이밍) · `517ed32c92d6`(같은 브리핑 화면의 다른 쪽) · `73f3a21e981c`(같은 «통화료» 대화상자 · 커서 차이) · `dbd078113b97`(같은 화면 · 시간대 배경) · `a10a1f02b41b`(§3).

### 3. 따로 본 타이틀

| sha12 | base(-1) | zero(0) |
|---|---|---|
| `1a69522a7d43` | 랭킹→＃→예→예: «접속 시 통화료…»에서 멈춤 | «연결에 실패하였습니다» → 확인 → 주 메뉴 (`docs/keys/lgt-1a69522a7d43-online-register.keys`) |
| `dbd078113b97`(0448) | «통신장애입니다 재시도해주세요» · connect 4~5 | 같은 안내 · connect 4~5 — -1 비교 길 대신 소켓 예외 길로 같은 곳 |
| `3185174d2121`(0470 ⒞) | 글자 없는 보라 상자 | 같음(프레임 md5 동일) — 이 병은 connect 반환값이 아니다 |
| `513d4b49c0be` | 메뉴 6번 → connect 431회 재시도 동안 메뉴 정지 → «네트워크 통신이 지연되고 있습니다» | connect 3회 뒤 같은 안내로 바로 |
| `a10a1f02b41b` | 부팅 «수신동의 등록 중 · 연결대기» connect 4회(5초 간격) | 1회 → 소켓 예외 → 바로 진행(75초 안에 메뉴까지) |

`513d4b49c0be` 의 안내는 두 빌드 모두 CLR·OK·1 에 닫히지 않는다 — 기존 동작, 이 회차 범위 밖.
실기 규약 근거: 같은 실패를 0448 KTF 는 -1 비교로, 0511 LGT 는 소켓 예외로 처리한다. 0 이면 두 길 모두 실패 안내에 닿고(위 표) -1 이면 LGT 쪽이 멈춘다 ⇒ 타이틀별 분기는 만들지 않았다.

### 4. 게이트

fmt OK · clippy stable/wasm32 `-D warnings` OK · beta clippy rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` 777 통과 0 실패 · 러너 블록 6줄 PASS(keydraw KTF paints 79 · LGT 55) · `npm run build:wasm` rc 0.
시험 `connect_succeeds_so_the_socket_reports_the_failure`(`network.rs`) — **되돌리면 red**(`left: -1, right: 0`).

### 5. 한계

- longplay(600초)·progress 짝은 돌리지 않았다(162×2 = 약 18시간). 그 대신 §2 도달 sweep 으로 «connect 를 부르는 타이틀»을 찾아 짝했다. 75초 정책 키가 닿지 않는 깊은 메뉴의 connect 는 이 판이 보지 못한다.
- compat 은 `1a69522a7d43` `knownIssues_ko` 한 줄만 바꿨다(그 길을 직접 쟀다). `513d4b49c0be` 의 netWall 소멸은 census 판정용 계수이고 compat 행에 NET_KO 가 없어 바꿀 것이 없다.

### 6. 장부

- compat `1a69522a7d43` `knownIssues_ko` · census `HAND_NOTE` 같은 줄.
- `docs/player-updates/2026-10-12-lgt-online-register-returns-to-menu.json`(fix).
- 0507 원장 csv `1a69522a7d43` 행 note · basis 0514.
- worklog `docs/worklog/2026-10-12-msf-network-connect-zero.json`: adopted `2026-10-11-lgt-1a69522a7d43-nested-paint-host-panic#p0` · 카드 0.

유입: BOUNDED 330쌍은 전부 `compat.json` 의 기존 행(이 회차가 바꾼 것은 `1a69522a7d43` 의 `knownIssues_ko` 한 줄 · 제목 0) · 그 파일을 뺀 8파일은 BOUNDED 0 · SUFFIX-ATTACHED 1(census 의 기존 «턴종료» 줄 · 이 회차 diff 에 0회).

<!-- corpus-name-inflow v1 subjects=9 tree=7f3613da047d0967 B=718/330 P=0/0 S=38/18 -->
