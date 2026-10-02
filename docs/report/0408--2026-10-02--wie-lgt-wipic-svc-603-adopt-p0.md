## [2026-10-02] LGT WIPIC SVC 603 = `MC_netSocketConnect` — 네트워크 없음을 콜백으로 알린다 (wie-lgt-wipic-svc-603-adopt-p0)

**무엇을**: LGT WIPIC 603(`0x25b`)을 `MC_netSocketConnect(fd, addr, port, cb, param)` 로 표에 올렸다. 네트워크가 없으므로 0 을 돌려주고 `cb(fd, M_E_ERROR, param)` 를 나중에 부른다 — 같은 파일의 `MC_netConnect` 스텁과 같은 모양이다. 공유 `wie_wipi_c::api::net::socket_connect` 로 두었고 **LGT 만 배선**했다(KTF 표의 index 3 은 `gen_stub` 그대로).
**왜**: 0402 §5 의 다음 벽 — `fe76e641bb3d` 가 20번째 키에서 `Unknown LGT WIPIC SVC id 603` 으로 끝났다.
**사용자 영향**: `fe76e641bb3d` 30초 조작 FAIL 3/3 → **PASS 3/3**(27/27 키). 가드 6종(LGT 5 · KTF 1)은 전·후 같다. 지원 현황은 §5.

증적: `~/orchestrator/reports/evidence/wie-lgt-wipic-svc-603-adopt-p0/`. sha12 만 쓴다.

### 1. 호출부 — `fe76` binary.mod `0x24428`(Thumb)

```
r5 = 904("222.…")                 ; inet_addr(리터럴 0xc0534)
r4 = 901(port) << 16 >> 16        ; htons
fd = 602(2, 1) → [0x1512a38]      ; MC_netSocket(AF_INET, SOCK_STREAM)
603(fd, r5, r4, 0x24501, [sp]=0)  ; ← 여기서 멈췄다 (lr 0x24467)
```

(증적 `fe76-0x24428.txt`·`fe76-literals.txt`). 오류의 `r1=0xaf4eedde` 는 그 주소를 network order 로 담은 값이고 `r2=0x3a9d` 는 `htons` 한 포트다 — 인자 셋이 오류 줄과 바이트 단위로 맞는다. `r0=0xffffffff` 는 602 가 돌려준 «소켓 없음»이다.

- 이 함수는 `MC_netConnect` 의 콜백으로 불린다(`0x2448c` 가 `0x24429` 를 cb 로 넘긴다 · 0402 census stderr 의 `stub MC_netConnect(0x24429, …)`). 그 콜백은 결과(r0)를 보지 않고 곧장 소켓을 연다.
- 603 의 반환값은 **쓰지 않는다**(`bl` 뒤 곧장 `pop`).
- 콜백 `0x24500` 은 **r1** 만 본다: 0 이면 `[0x1513250] = 2`, 아니면 `[0x151324c] = 1`. 호출 전에 `0x2448c` 가 `[0x151324c] = 0` · `[0x1513250] = 1` 로 둔다 ⇒ `1513250` 은 «연결 중 → 연결됨», `151324c` 는 «실패» 표지다.

⇒ 인자 순서는 KTF 표의 net index 3 `MC_netSocketConnect` · WIPI `MC_NetSocketConnectCB(fd, result, param)` 와 같다. 콜백이 «실패»를 받아야 게임이 «연결 중»에서 빠진다 — 그래서 반환값 -1 만 주는 동기 실패(콜백 없음)가 아니라 콜백 실패를 택했다.

### 2. 같은 SVC 를 쓰는 다른 타이틀

- `game_lab` 전체(리포트·stderr 포함)에서 `SVC id 603` 문자열: **0 파일**. 이 번호에서 멈춘 기록은 0402 증적의 `fe76` 뿐이다.
- 최신 census(`bd2337ff`) stderr 로 `MC_netConnect` 를 부른 LGT 타이틀은 20종, 그중 `MC_netSocket` 까지 간 것은 9종이다. 그 9종 중 603 에서 멈춘 것은 0 — 603 에 닿지 않거나(소켓 -1 을 보고 멈춘다) 닿아도 기록이 없다. 짝 재측에는 그 9종 중 4종을 가드로 넣었다(§3).

### 3. 짝 재측

전 = 이 브랜치에서 `0x25b` 행만 뺀 빌드(= `origin/main` `5cbd635e` 와 LGT 동작이 같다 — 나머지 차이는 쓰이지 않는 공유 함수와 시험 코드뿐) · 후 = 이 PR · release · `--inject --pacing 8 --relaunch 1 --max-ticks 2000000000` · 3판씩 번갈아 · 순차(jobs 1).

| sha12 | 플랫폼 | 전 ×3 | 후 ×3 |
|---|---|---|---|
| `fe76e641bb3d` | LGT | FAIL ×3 · 20번째 키 · SVC 603 · paints 150~158 | **PASS ×3** · 27/27 · paints 186~204 |
| `2a8a3dcd07eb` | LGT(net) | PASS ×3 | PASS ×3 |
| `320a5360a0f3` | LGT(net) | PASS ×3 | PASS ×3 |
| `b9bfcaf42722` | LGT(net) | PASS ×3 | PASS ×3 |
| `caf9d76ffd13` | LGT(net) | PASS ×3 | PASS ×3 |
| `2a57e33133b5` | LGT | PASS ×3 | PASS ×3 |
| `49ade89578c5` | KTF | PASS ×3 | PASS ×3 |

(증적 `pair.log`). **부하를 숨기지 않는다**: 시작 시 `host-load-guard --status --recovered` rc=0(09:41 · 보류가 풀리기를 약 55분 기다렸다 · `hl-start.txt`), 끝에서도 rc=0(`hl-end.txt`). 그러나 도중에 다른 레인 부하로 load1 이 25 → **280** 까지 올랐다(각 줄의 `load=`). 판정은 전·후 전부 같았고, `fe76` 의 전·후 갈림은 부하가 낮은 앞쪽(load 25~40)에서 났다.

### 4. 되돌리면 red

- `0x25b => Self::SocketConnect` 행을 빼면 `wipic_svc_603_socket_connect_is_in_the_table` red(1 failed).
- 콜백 결과를 `M_E_ERROR` 대신 0(성공)으로 바꾸면 `socket_connect_fails_through_callback_test` red(1 failed). 이 시험은 «반환 0 · 콜백 하나 · 인자 `(fd, -1, param)`»를 함께 잡는다. 그러려고 `TestContext` 가 `spawn` 을 `todo!()` 대신 쌓아 두고(`spawned`) `call_function` 의 인자를 남긴다(`call_args`).

### 5. 지원 현황(`compat.json`)

`playability-census.mjs run --jobs 1 --long 600`(후 빌드 release) · `fe76` 하나만 담은 스크래치 corpus(심링크). 시작 시 `--recovered` rc=0, **끝날 때는 다른 레인 부하로 보류**(load1 208 · rc=1 · 증적 `hl-census-end.txt`) — 장시간 축은 «오류 없이 버텼나»이므로 부하는 덜 진행시킬 뿐 거짓 ok 를 만들지 않는다(0402 §6 과 같은 읽기).

| sha12 | A(30초) | L(600초) | census | 이 PR 의 행 |
|---|---|---|---|---|
| `fe76e641bb3d` | **PASS** · 27/27 키 · paints 291 | 854/900 키 · deadline · 오류 0 | playable · longplay ok | **limited → playable** · longplay no → ok · 알려진 문제 «멈추거나 꺼짐» 삭제 |

`status`·`longplay`·`knownIssues_ko` 만 바꿨다(0402 와 같은 방식). census 는 sound·speed 도 ok 를 냈지만 이 회차가 잰 축이 아니라 그대로 두었다(«소리가 나지 않을 수 있어요»도 남는다). 증적 `census/`.

### 6. 게이트

- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable · beta) · wasm32 clippy · `RUST_MIN_STACK=4194304 cargo test --all`(실패 0) — 전부 rc=0. 이 worktree 의 target · build-slot 경유.
- 러너 블록(debug): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me`(`--timeout 5`) PASS · `keydraw_ktf`·`keydraw_lgt` `--inject --expect-last-frame` PASS · `content true` · rc=0(증적 `runner.log` — rc 열은 따로 다시 쟀다).
- `node scripts/player-data.mjs` OK · `node scripts/check-worklog-json.mjs` OK · `node scripts/check-docs-report-serial.mjs` OK.
- 유입(`node scripts/corpus-name-inflow.mjs --corpus <main checkout>/game_lab` — 이 worktree 에는 corpus 가 없다): BOUNDED 340쌍 · SUFFIX-ATTACHED 15쌍 — 전부 이 PR 이 손댄 파일에 이미 있던 줄이다(`compat.json` 의 `title`·`fileTitle` · `svc_ids.rs`·`wipi_c.rs` 의 기존 주석). 이 회차가 더한 줄(diff `+` 155줄)과 그 331개 이름을 대조하면 **0** — 타이틀은 sha12 로만 적었다.
