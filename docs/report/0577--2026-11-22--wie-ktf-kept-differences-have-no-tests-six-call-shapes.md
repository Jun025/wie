## [2026-11-22] KTF 가짜 서버의 «일부러 남긴 차이» 여섯 줄을 시험으로 잠갔다 — 닫기는 -1 그대로 (wie-ktf-kept-differences-have-no-tests-six-call-shapes)

**무엇을**
- `wie-ktf/src/runtime/wipi_c/local_net.rs` 에 시험 2개를 더했다. 0569 §1 표의 여섯 줄(슬롯 30~32 · 1ms 콜백 · 표 밖 fd 치명 · `MC_netSetReadCB` 0 · fd `0x4c4e` · 닫기 -1)을 게임이 부르는 길 그대로 — 메서드 표의 슬롯 번호로 — 부르고 반환값과 콜백을 본다.
- 닫기의 값은 바꾸지 않았다. 0 과 -1 을 실제 판에서 나란히 쟀고(§3) 차이가 없었다. 이 판은 닫기의 반환값을 읽지 않는다.
- 0569 §2 에 통신사 교차 AID 가장자리 한 줄을 덧붙였다(게이트② F2 · §4).
- 제품 코드의 동작은 한 줄도 바뀌지 않았다. 바뀐 것은 시험 · 시험 틀의 공개 범위(`context.rs` 의 `mod test` 를 같은 디렉터리에서 쓰게 열고 `run_on(platform, aid, body)` 를 뽑았다) · 주석 한 문단이다.

**왜** — 게이트② 회신(`wie-ktf-local-net-fold-into-fake-server.review.md` F1)이 KTF 호출 모양을 겨냥한 개악 6종을 걸었는데 전부 green 으로 살아남았다. 그 여섯 줄은 주석과 수동 게임 측정으로만 지켜지고 있었고, 특히 닫기 -1 은 #601 이 새로 명시한 분기였다.

**사용자 영향** — 없다. `compat.json` 변화 0.

### 1. 시험 2개가 보는 것

둘 다 실제 `KtfWIPICContext`(ARM 코어 · JVM · `System`)를 띄우고 `get_method_body(WIPICTableId::Net, 슬롯)` 으로 부른다. 콜백은 게스트 메모리에 쓴 Thumb 3명령(`cb(fd, result, param)` 이 `fd` 와 `result` 를 `param` 자리에 적는다)으로 받는다. 시계는 읽을 때마다 1µs 가는 `TestClock` 이라 지연은 호스트 부하가 아니라 실행기의 순번으로 센다.

| 0569 §1 의 줄 | 시험 | 단언 |
|---|---|---|
| 호출 자리(슬롯 30 · 31 · 32) | `listed_title_is_answered_in_the_shape_it_was_measured_against` | 슬롯 30 인자 6개 → 0 · 슬롯 31 158바이트 → 158 · 슬롯 32 → 5바이트 `[3, 0, 0xf9, 0x2a, 0]` 뒤 -19 |
| 〃 (LGT 자리 3 · 4 · 5 는 여전히 치명) | `a_descriptor_this_file_did_not_hand_out_is_fatal` | 슬롯 3 · 4 · 5 → `Unimplemented` |
| 연결 콜백 1ms 뒤 | 첫째 | 콜백 `(0x4c4e, 0)` 이 온 시각 − 부른 시각 = 정확히 1ms · 다른 포트는 `(0x4c4e, -1)` |
| 표 밖 fd 는 치명 | 둘째 | 표에 있는 판의 fd 0 · 표에 없는 판의 fd -1 로 슬롯 30 · 31 · 32 · 13 → `Unimplemented` |
| `MC_netSetReadCB` 0 · 콜백 없음 | 첫째 | 응답이 쌓인 상태에서 슬롯 13 → 0 · 5ms 동안 콜백 없음 |
| fd 는 항상 `0x4c4e` | 첫째 | 슬롯 2 를 두 번 불러 둘 다 `0x4c4e` |
| 닫기(슬롯 6 · 33) -1 · 소켓 남음 | 첫째 | 두 슬롯 다 -1 · 닫은 뒤에도 `fake_net().contains(0x4c4e)` |

표에 없는 판(둘째 시험의 앞 절반)은 `MC_netSocket` -1 · 닫기 -1 이다. 시험의 fd 는 `LOCAL_FD` 상수를 읽지 않고 `0x4c4e` 를 글자로 적었다 — 상수를 읽으면 상수가 바뀔 때 시험이 따라간다.

### 2. 검수의 개악 6종 — 전부 red

검수 증적 `gate2/mut.sh` 의 치환식 여섯 줄을 글자 그대로 이 브랜치에 하나씩 걸고 검수와 같은 명령(`cargo test -p wie-backend -p wie-ktf -p test-utils -p wie-wipi-c`)으로 돌렸다. 기준은 285 통과다(검수 때 283 + 새 시험 2). 판마다 원복했고 끝에 두 파일이 원본과 같은지 `cmp` 로 확인했다.

| 개악 | #601 때 | 지금 | red 가 된 시험 · 단언 |
|---|---|---|---|
| ⒜ 닫기 -1 → 0 | green | red | 첫째 · `slot 6` 좌 0 우 -1 |
| ⒝ `LOCAL_FD` `0x4c4e` → 0 | green | red | 첫째 · 둘째 · 슬롯 2 좌 0 우 19534 |
| ⒞ 연결 콜백 1ms 지연 제거 | green | red | 첫째 · `the connect callback's delay` 좌 0 우 1 |
| 닫기(슬롯 6 · 33)를 LGT `net::socket_close` 로 | green | red | 첫째 · `slot 6` 좌 0 우 -1 |
| `MC_netSetReadCB` 0 → -1 | green | red | 첫째 · 슬롯 13 좌 -1 우 0 |
| 슬롯 31 표 밖 fd 치명 → -1 | green | red | 둘째 · `slot 31` |

### 3. 닫기 0 대 -1 — 재 보니 같다. -1 로 잠근다

**정적 근거.** `scripts/ktf-image-sweep.py slots` 로 이 판의 이미지를 훑으면 통신 표(전역 `sl+0x1b8`)를 거치는 슬롯 6 호출은 한 자리(이미지 `0x103dee`)다. `window` 로 그 함수(`0x103dd0`~`0x103e08`)를 읽으면 닫기 호출 바로 다음 두 명령이 표의 슬롯 1 을 r3 에 싣고 다시 부른다(`MC_netClose`). 그 사이에 r0 을 읽는 명령이 없고, 함수는 연결 표시 바이트를 0 으로 쓰고 돌아간다. 닫기의 반환값은 어디에도 쓰이지 않는다.

**실측.** 릴리스 바이너리 둘 — 닫기 -1(`origin/main` `f9799f1e6` 그대로) · 닫기 0(그 위에 개악 ⒜ 한 줄) — 과 대조(-1 바이너리를 한 번 더)를 판마다 셋 동시에 띄웠다. 인자는 0569 의 레시피 L 그대로다(`--inject --keys game_lab/recipes/f44271803135-newgame.keys --keep-timeout --timeout 180 --max-ticks 100000000000 --shot-every 20 --relaunch 1`). 코퍼스 파일은 읽기만 했다. long 임대 1건(3판 · 약 10분).

| 판 | 실행 | 판정 | 키 | 다시 켬 | 닫기 호출 수 | 통신 줄 | DB(쓰기 · 읽기) | 색 수 | 마지막 프레임 | `paints` | load1 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 닫기 -1 | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 9 · 63 | 228 | `2111a16889ea` | 7312 | 20.9 |
| 1 | 닫기 0 | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 9 · 63 | 228 | `2111a16889ea` | 7076 | 〃 |
| 1 | 닫기 -1(대조) | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 9 · 63 | 228 | `2111a16889ea` | 7452 | 〃 |
| 2 | 닫기 -1 | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 10 · 65 | 512 | `043aba81976b` | 9905 | 37.4 |
| 2 | 닫기 0 | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 10 · 65 | 512 | `043aba81976b` | 9750 | 〃 |
| 2 | 닫기 -1(대조) | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 10 · 65 | 512 | `043aba81976b` | 9795 | 〃 |
| 3 | 닫기 -1 | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 10 · 65 | 512 | `b5c73a5abadb` | 10359 | 29.6 |
| 3 | 닫기 0 | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 10 · 65 | 512 | `b5c73a5abadb` | 10310 | 〃 |
| 3 | 닫기 -1(대조) | PASS | 11/11 | 1 | 1 | 5 · `7ef9744458ae` | 10 · 65 | 512 | `b5c73a5abadb` | 11144 | 〃 |

9개 실행 전부 rc=0 · `stop` 은 `deadline` · 프레임 23장. 닫기 -1 대 닫기 0: 판정 3/3 · 통신 줄 3/3 · DB 3/3 · 마지막 프레임 3/3 같음. 닫기 -1 대 대조: 같은 3/3 씩.

- 판 안에서는 세 실행이 판정 · 키 · 닫기 호출 수 · 통신 5줄 · DB 통계 · 마지막 프레임 해시까지 같다. 그리는 횟수(`paints`)만 실행마다 다르고 그 폭은 대조 쪽에서도 같다.
- 판 사이에는 색 수와 마지막 프레임이 다르다(부하에 따라 180초 안에 간 거리가 다르다). 그래서 비교는 판 안에서만 했다.
- 통신 5줄은 #601 때와 같다: `MC_netConnect` 성공 → 슬롯 30 `(0xfd126ed3, 8503)` → 슬롯 31 158바이트 응답 → `MC_netSocketClose(19534)` → `MC_netClose()`. 닫기는 실행마다 1번이고 그 뒤에 통신 호출은 없다.

**결정.** 값을 바꿀 근거가 없다 — 이 판은 그 값을 읽지 않고, 재어 봐도 같다. 지금 값 -1 을 시험으로 잠갔다(§1 · §2 의 ⒜). 0 으로 가면 «남긴 차이»가 하나 줄지만(닫기를 LGT 와 같은 함수로 보낼 수 있다), 그것은 게임이 보는 값을 근거 없이 바꾸는 일이라 하지 않았다. `socket_close` 의 주석에 이 근거를 한 문장 적었다.

### 4. 0569 §2 에 덧붙인 줄(게이트② F2)

표가 하나가 되면서 `online()` 은 AID 만 본다. AID `0103A2B4` 를 가진 LGT 파일은 이제 `MC_netConnect` 성공 콜백과 `MC_netSocket` fd 를 받고(전: 실패), 반대 방향은 전에도 열려 있었으며 fd 만 0 에서 `0x4c4e` 로 바뀐다. 코퍼스 실례는 0건(검수 실측)이다. 이 회차는 그 수를 다시 세지 않고 검수의 값을 옮겨 적었다.

### 5. 게이트

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | rc=0 |
| `cargo clippy --all -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=0. 이 줄은 루트 패키지만 본다 — `-p wie_featurephone` 을 붙여 따로 돌려 `wie-ktf` 를 지나게 했다: rc=0 |
| `RUST_MIN_STACK=4194304 cargo test --all` | rc=0 · 57 스위트 · 872 통과 · 0 실패(새 시험 2개 포함) |
| `cargo +beta clippy --all -- -D warnings` | rc=0(beta 1.101.0-beta.2). 매니페스트 경고 9줄(`wie_cli` 7 · `test-utils` 1 · `wie-util` 1)은 이 PR 이전부터 있고 rc 를 바꾸지 않는다. PR 의 beta 다리는 빈 통과라 beta 는 이 로컬 실행이 유일한 근거다 |
| 러너 블록 | `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS rc=0 · `keydraw_ktf.zip` · `keydraw_lgt.zip` `--inject --expect-last-frame` PASS rc=0(키 27/27 · paints 79 · 55) · `text_j2me.jar` `--timeout 5` PASS rc=0 · load1 21. 픽스처 재생성 뒤 `git status` 변화 0 |
| 그 밖 | `node scripts/player-data.mjs` · `check-worklog-json` · `check-doc-liveness-parity` · `npm run audit` rc=0 · `git diff origin/main -- docs/player-data/compat.json` 0줄 |

`cargo test` 는 `wie-ktf` 시험 코드에 이 PR 이전부터 있던 경고 1줄(`jvm_support.rs:768` `unused_mut`)을 찍는다. 고치지 않았다(범위 밖 · rc 무관).

### 6. 한계

- 잰 «닫기 0» 은 개악 ⒜(0 을 답하고 소켓은 남긴다)다. LGT 경로(0 을 답하고 소켓을 없앤다)는 게임으로 재지 않았다. 이 판은 닫기 뒤에 그 fd 로 아무것도 부르지 않으므로(통신 5줄의 끝) 둘이 갈릴 자리는 측정 경로에 없다.
- 정적 근거는 `slots` 가 찾은 호출 자리에 기댄다. 그 도구는 최선 노력이고(머리말), 슬롯 33(오류 정리 쪽 닫기)을 부르는 자리는 이 표 전역으로 잡히지 않았다. 실행에서 본 닫기는 판마다 1번, 슬롯 6 이다. 슬롯 33 의 -1 은 시험이 잠그지만 이 판이 그 값을 읽는지는 확인하지 못했다.
- 시험의 «1ms» 는 엔진 시계로 잰 값이다(1µs 계단 시계에서 정확히 1). 실제 기기의 콜백 지연과 같다는 주장이 아니다 — 이 판이 재어진 값을 지키는 것이다.
- 측정은 `wie_validate`(네이티브)다. 브라우저 호스트에서는 재지 않았다. 이 PR 은 두 호스트가 같이 쓰는 제품 코드를 바꾸지 않는다.
- 3판 · 한 판(`f44271803135`)이다. 표에 있는 KTF 판이 이것 하나다.

### 7. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치가 `origin/main` 대비 바꾼 파일 4개): 유입 1회 / 1쌍(BOUNDED) · 판단 필요 0건(SUFFIX-ATTACHED). 그 1회는 `wie-ktf/src/runtime/wipi_c/context.rs` 의 `ResourceCache` 주석에 원래 있던 이름이다 — 이 PR 은 그 파일의 시험 모듈만 고쳤고 그 주석은 건드리지 않았다. 이 문서와 새 시험 · 0569 에 덧붙인 줄에서는 0 이다(판을 sha 앞 12자로만 부른다).
