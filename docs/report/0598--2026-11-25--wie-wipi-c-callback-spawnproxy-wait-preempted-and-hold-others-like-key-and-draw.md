## [2026-11-25] WIPI C 콜백(클립 상태 · 소켓)을 다른 게스트 스레드의 코드 한가운데로 넣지 않는다 — 키 · 그리기와 같은 «기다림 + 붙듦» (wie-wipi-c-callback-spawnproxy-wait-preempted-and-hold-others-like-key-and-draw)

**무엇을** — `0591` 의 제안(`2026-11-24-8b01-stack-overflow-is-clip-callback-thread-inside-event-pump#p0`)을 집행했다. `wie-ktf` · `wie-lgt` 의 `SpawnProxy` 가 콜백을 새 함수 `wie_wipi_c::call_spawned` 로 부른다. 그 함수는 키 전달(`0430`)과 그리기(`0506`)가 이미 쓰는 두 단계를 그대로 쓴다:

1. 다른 게스트 스레드가 «게스트 코드 도중에 명령 예산으로 잘려» 있으면(`System::guest_others_preempted`) 1ms 씩 쉬며 기다린다. **상한 250ms** — 키의 `KEY_WAITS_FOR_GUEST_MS` 와 같은 값이다. 상한을 넘기면 지금처럼 들어간다(창이 닫히지 않는다) — 그때는 2 의 붙듦도 걸지 않는다(§8 F1).
2. 콜백이 도는 동안 다른 스레드를 붙든다(`System::guest_hold_others`) — 콜백이 잘려도 다른 스레드가 게스트 코드를 잇지 않는다(붙듦 자체의 상한은 `wie-core-arm` 의 `HOLD_ROUNDS` 200 그대로).

새 동기화 기구는 없다. 키 · 그리기 길(`wie-midp` `display.rs`)과 `wie-core-arm` 은 바꾸지 않았다(diff 에 없다).

**왜** — `0591` §5 · §6: `8b01d4ae6af4` 의 이벤트 펌프는 «바쁨 깃발을 본다 → 큐에서 꺼낸다» 사이를 잠그지 않는다. 주 스레드가 그 틈에서 잘린 사이 클립 상태 콜백 스레드가 들어와 주 스레드의 레코드를 꺼내 돌리고, 주 스레드는 돌아와 콜백의 레코드로 같은 스크립트 실행기를 겹쳐 돈다 → 스크립트 오류 → 오류 처리기 되부름 → 게스트 스택 넘침.

**사용자 영향** — 그 KTF 판이 시작 화면에서 소리가 바뀐 직후 드물게 멈추던 길이 닫혔다(강제 재현에서 겹침 · 오류 · 넘침 0). 클립 상태 · 소켓 콜백을 쓰는 KTF · LGT 판 전부가 같은 길을 탄다. 비용은 콜백 지연이다: 대상 세 판에서 최대 79ms · 상한 도달 0(§2 · §3), KTF 60판 표본에서 콜백 90개 중 상한(250ms) 도달 2 — 두 판에서 하나씩이고, 그 두 판은 매 런 그렇다(§4 · §8 F5). 붙든 동안 다른 스레드가 밀린 차례는 어디서도 0 이었다.

증적(전부 저장소 밖 · sha12 만): `~/orchestrator/reports/evidence/wie-wipi-c-callback-spawnproxy-wait-preempted-and-hold-others-like-key-and-draw/` — 런별 결과 줄(`runs/<묶음>/*.json`) · 드라이버(`run.sh` · `phase.sh` · `chunks.sh` · `smoke-parts.sh`) · 집계(`tally.py`). 계측 패치는 같은 곳의 `scratch-instrument.diff` 다 — `8e0df626` 트리에 `patch -p1` 로 얹는다(새 파일 포함 · §8 F7). 판 파일은 증적에 두지 않는다: `mklinks.sh` 가 코퍼스에서 `<sha12>.zip` 링크를 다시 만든다. 수정 회차(§8)의 것은 그 아래 `fix/` 다.

### 1. 측정 방법

스크래치 빌드 = 이 브랜치(`c14509e7d` — 엔진 트리는 `8e0df626` 과 같다)에 계측과 손잡이만 얹은 release `wie_validate`. 저장소에는 들어가지 않는다.

- **강제**(`WIE_SCRATCH_FORCE=2`): `0591` §5 와 같은 손잡이 — 주 스레드(스레드 id ≤ 2)가 `0x100efa`(펌프가 깃발이 꺼진 것을 본 직후 · 큐에서 꺼내기 전)에 닿으면, 다른 게스트 스레드가 살아 있을 때 거기서 슬라이스를 끝낸다.
- **전/후**(`WIE_SCRATCH_NOFIX=1`): `call_spawned` 의 기다림 · 붙듦을 건너뛰고 콜백만 부른다 — `main` 의 `SpawnProxy` 가 하던 호출 하나와 같다. 같은 바이너리로 전후를 갈랐다(빌드 차이 0).
- **탐침**(8b01 이미지 주소 · `WIE_SCRATCH_PROBE=1` 일 때만): 스크립트 실행기 진입 `0x106ee0` · 나감 `0x106f7c` · 오류 처리기 `0x1013d4`.
- 셈: «겹침» = 한 스레드가 실행기 안에 있는 동안 다른 스레드가 0 이 아닌 진입점으로 실행기에 들어간 런. «오류» = 오류 처리기에 닿은 런. «넘침» = `0x3FFFFFxx` 접근 위반으로 끝났거나 처리기 진입이 1,000회를 넘은 런.
- 실행: `--inject --keep-timeout --timeout 14 --max-ticks 100000000000 --pacing 8 --relaunch 1`(`0591` 과 같다) · 굶김 `d5` = 검증기를 `SIGSTOP`/`SIGCONT` 로 벽시계의 1/5 만 돌린다(`0591` 의 방법). 에뮬레이터는 늘 동시 3 이하.
- **임대**: long 풀(1칸)을 다른 레인의 `tests/no-llm-suite.sh` 가 이 회차 내내(관측 2시간 9분 이상) 잡고 있었다. `0591` §8 의 선례대로 측정은 **short 임대 1건당 6런(약 35 ~ 50초)** 으로 쪼개 돌렸다(`chunks.sh`). long 임대를 한 번 기다렸다가(20분) 내려놓았다.

### 2. 전후 — 겹침 · 오류 · 넘침 (할 일 2)

칸은 «겹친 런 · 오류 난 런 · 넘친 런 / 회».

| 조건 | 모드 | 전(`NOFIX`) | 후 | load1 |
|---|---|---|---|---|
| 강제 · 굶기지 않음 | 그림자 | 9 · 0 · 0 / 12 | **0 · 0 · 0** / 12 | 전 6.1 ~ 23.3 · 후 13.0 ~ 21.3 |
| 강제 · 굶기지 않음 | 훅 | 11 · 0 · 0 / 12 | **0 · 0 · 0** / 12 | 〃 |
| 강제 · 굶기지 않음 | 훅 없음 | 11 · 0 · 0 / 12 | **0 · 0 · 0** / 12 | 〃 |
| **강제 · 굶기지 않음 합** | | **31 · 0 · 0 / 36** | **0 · 0 · 0 / 36** | |
| 강제 · 굶김 1/5 | 훅 | **8 · 2 · 2 / 8** | **0 · 0 · 0 / 8** | 전 11.1 ~ 14.7 · 후 13.9 ~ 16.9 |

- 굶기지 않은 묶음은 오류가 전에도 0 이다. `0591` §5 도 오류 4건이 전부 load1 173 ~ 187 묶음에서 났고 139 ~ 157 묶음은 0/24 였다 — 이 회차의 호스트는 조용했다(load1 6 ~ 23). 그래서 굶긴 묶음을 더했고, 거기서 전 2/8 이 오류 → 스택 넘침(FAIL)으로 끝났다. 후는 0/8.
- 강제 손잡이가 실제로 슬라이스를 끊은 런: 전 32/36 · 후 35/36 · 굶김 전 8/8 · 후 8/8. 후의 0 은 «틈이 오지 않아서»가 아니다.
- 후에서 콜백 60개 중 48개(굶기지 않음 · 모드별 같은 꼴)가 기다렸다 — 최대 28ms. 굶김에서는 40개 중 39개 · 최대 79ms. 상한(250ms) 도달 0.
- 콜백 한 번의 길이(플랫폼 ms): 전 최대 19ms(굶김 163ms — 콜백이 펌프에서 남의 스크립트를 돌렸기 때문) → 후 최대 1ms.

### 3. 비용 (할 일 3)

강제 없음 · 자연 실행. 칸의 «기다림»은 콜백이 게스트 코드에 들어가기 전에 쉰 ms(`call_spawned` 의 1ms 단위)다. «붙듦 대기»는 콜백 스레드가 붙든 동안 다른 스레드가 게스트 코드를 잇지 못하고 양보한 차례 수다(`wait_for_holder` 의 돈 횟수).

| 판 | 굶김 | 런 | 콜백 | 기다린 콜백 | 기다림 최대 | 분포(0 · 1-4 · 5-19 · 20-99 · 100-249 · 상한) | 붙듦 대기 | 그리기 수 전 → 후 |
|---|---|---|---|---|---|---|---|---|
| `8b01d4ae6af4`(KTF) | 없음 | 6+6 | 30 | 24 | 24ms | 6 · 2 · 10 · 12 · 0 · 0 | 0 | 135 ~ 138 → 132 ~ 137 |
| `8b01d4ae6af4` | 1/5 | 6+6 | 30 | 30 | 29ms | 0 · 0 · 12 · 18 · 0 · 0 | 0 | 76 ~ 78 → 76 ~ 77 |
| `87b04639cdfe`(LGT) | 없음 | 6+6 | 24 | 18 | 28ms | 6 · 0 · 6 · 12 · 0 · 0 | 0 | 173 ~ 181 → 177 ~ 181 |
| `87b04639cdfe` | 1/5 | 6+6 | 12 | 7 | 6ms | 5 · 1 · 6 · 0 · 0 · 0 | 0 | 54 ~ 59 → 53 ~ 59 |
| `863b8ab6a21d`(LGT) | 없음 | 4+4 | 8 | 8 | 32ms | 0 · 4 · 0 · 4 · 0 · 0 | 0 | 366 ~ 376 → 363 ~ 371 |

**⑴ 콜백 지연** — 상한 도달 0. 최대 32ms(굶기지 않음) · 79ms(굶김 + 강제, §2). 전 판 조사는 §4.

**⑵ `87b04639cdfe` 의 박자** — 이 판은 «시작됨» 콜백에서 곡의 시작 시각을 잡는다(`media.rs` `play` 의 주석). 늦어지는 만큼이 곧 박자 밀림의 상한이다: **최대 28ms · 굶김에서 6ms**(콜백 18/24 · 7/12 가 기다렸다). 곡의 음표 간격과 견줄 수는 있지만 귀로 대조하지는 않았다 — 이 회차는 «기다린 ms»만 쟀다. `0533` 이 `863b8ab6a21d` 에서 본 결정선(«끝»을 250ms 늦게 줘도 2/2 가 같은 화면에 머물렀다)보다 한 자리 작다.

**⑶ 프레임 밀림** — 붙듦 대기 0회(전 판 조사 포함, §4). 콜백 길이가 후 최대 2ms 라 콜백이 슬라이스(1만 명령) 경계에서 잘린 일이 이 측정에는 없다. 그리기 수는 전후 범위가 겹친다.

### 4. 전 판 조사 — 콜백을 쓰는 판에서의 지연 · 붙듦 (후만)

`game_lab/working` 의 KTF · LGT 고유 판 215개를 후 빌드로 한 번씩(`--inject --keep-timeout --timeout 12` · 굶기지 않음 · 동시 2) 돌리려 했으나, short 풀이 다른 레인으로 차서(대기 8) **60판(KTF · sha 순 앞 60)에서 멈췄다** — 게임 랩 게이트(§5)에 임대를 넘기려고. 이 수는 표본이다(load1 9.4 ~ 35.4 · 12초 런은 키가 닿기 전에 끝나 판정은 전부 `UNMEASURED` 다 — 이 조사는 판정을 보지 않는다).

| 판 | 콜백을 띄운 판 | 콜백 | 기다린 콜백 | 분포(0 · 1-4 · 5-19 · 20-99 · 100-249 · 상한) | 붙듦 대기 | 콜백 길이 최대 |
|---|---|---|---|---|---|---|
| 60 | 12 | 90 | 58 | 32 · 11 · 31 · 14 · 0 · **2** | 0 | 1ms |

- **상한(250ms) 도달 2회 · 2판**(`503d5d2f196b` · `155972cac664` · 각 1회). 다른 스레드가 250ms 내내 게스트 코드 도중이었다 — 그 콜백은 250ms 늦게, 지금처럼 끼어든 채 들어갔다. **드문 일이 아니라 그 판의 매 런이다**: 검수 실측으로 `503d5d2f196b` 16/16 런 · `155972cac664` 11/11 런에서 콜백 하나가 상한을 채웠고(전 0), 이 수정 회차도 같다(§8 F5). 전후 짝에서 그리기 수는 갈리지 않았다(검수 4+4: 111 ~ 116 ↔ 111 ~ 116 · 276 ~ 283 ↔ 276 ~ 282 · §8 의 짝도 같다).
- 그 밖의 최대: 95ms(`1fe6c1c897d7`) · 65ms · 56ms.
- 붙듦 대기 0 · 콜백 길이 최대 1ms — 이 표본에는 «콜백이 길어 다른 스레드가 밀리는 판»이 없다. 나머지 155판(LGT 45 포함)은 보지 않았다.

### 5. 회귀 (할 일 4)

- 네 게이트(`cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all`) + `cargo +beta clippy --all -- -D warnings`: 전부 통과(`cargo test --all` 실패 0).
- 새 시험 `wie-wipi-c` `a_spawned_callback_waits_for_a_sliced_thread_and_runs_held`: 기다림 루프를 빼면 `left: 3 · right: 0` 으로 빨갛게 된다(확인함).
- 엔진 러너(§The four gates 의 ENGINE-RUNNER 블록): `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` · `text_j2me.jar` PASS · `keydraw_ktf.zip` PASS · paints 79 · rc 0 · `keydraw_lgt.zip` PASS · paints 55 · rc 0(디버그 빌드 · short 임대 1건).
- 게임 랩 게이트 `scripts/smoke_gate.sh`(ktf + lgt · 기준선 242행): 16부 전부 `OK: no regressions vs baseline.` — 기준선 242행 확인 · 부재 0 · 회귀 0 · 돌린 판 244 PASS / 0 FAIL(기준선 밖 LGT 2판 포함). 이 브랜치의 디버그 `wie_validate` · 기본 `TIMEOUT 15`. long 풀이 막혀 있어 `WORKING_DIR` 을 16판씩 나눈 심링크 디렉터리 + 그 판들만 뽑은 `BASELINE` 으로 16부를 short 임대 하나씩 차례로 돌렸다(`smoke-parts.sh` · 에뮬레이터 1).
- `0430` · `0506` 경로 무변경: `git diff origin/main` 에 `wie-midp` · `wie-core-arm` 파일 0.

### 6. 한계

- 자연 런의 FAIL 율(`0591`: 굶긴 런 1/180)은 다시 재지 않았다 — 같은 틈을 매번 오게 하는 강제 재현으로 대신했다.
- 같은 처리기를 가진 나머지 10판(`0580` §4)은 강제 손잡이로 돌리지 않았다(손잡이 주소가 이 이미지의 것이다). §4 의 조사가 그 판들의 비용 쪽을 본다.
- `set_timer` 로 오는 콜백(클립의 END_OF_DATA · `MC_knlSetTimer`)은 이 길이 아니다 — 범위 밖이고, 그 길의 끼어듦은 관측하지 않았다.
- 원 단말이 콜백을 어느 문맥에서 부르는지는 여전히 재지 못했다(실기 없음).
- 붙듦은 «가장 최근에 붙든 스레드 하나»만 기억한다(`hold_others` 의 기존 성질 · 칸이 하나). 상한을 넘겨 들어오는 콜백은 그 칸을 건드리지 않는다(§8 F1 — 처음 판본은 가져갔다). **남은 길은 하나다**: 붙든 처리기(키 · 그리기)가 잘린 것이 아니라 «막는 호스트 호출»(잠 · 대기 · 입출력)에 서 있을 때는 `others_preempted` 가 거짓이라 콜백이 기다림 없이 들어가 칸을 가져가고, 끝나며 지운다 — 그 처리기는 호스트 호출에서 돌아온 뒤 남은 구간을 붙듦 없이 돈다. 그리기가 다른 스레드의 키 처리기에 대해 갖는 것과 같은 성질이다. 관측: 검수 0/106 콜백 · 이 회차 §8 의 표.

### 7. 게임 이름 유입

`corpus-name-inflow`: BOUNDED 3회 / 2쌍 · SUFFIX-ATTACHED 0. 셋 다 이 회차가 한 줄씩 고친 `wie-ktf` · `wie-lgt` 의 `runtime/wipi_c/context.rs` 에 **이미 있던 주석**이다(파일을 고쳐 파일 전체가 대상이 됐다) — 이 회차가 더한 이름은 0. 수정 회차(§8)가 같은 두 파일에 시험을 더하고 `wie-wipi-c/src/api/net.rs` 를 고친 뒤에도 같다(대상 8파일 · BOUNDED 3회 / 2쌍 · SUFFIX-ATTACHED 0). 이 문서 · worklog · 소식은 판을 sha12 로만 적는다. 게임 바이트 · 캡처는 0.

### 8. 게이트② 수정 회차 (`…-fix` · 같은 PR)

검수(게이트②)가 반려한 셋(F1 · F2 · F3)을 고치고 권고(F4 ~ F9)를 닫았다. 엔진 커밋 `14e8cacaa` — `wie-wipi-c`(`context.rs` · `api/net.rs`)와 `wie-ktf` · `wie-lgt` 의 시험. `wie-midp` · `wie-core-arm` 은 여전히 diff 에 없다.

**F1 — 상한(250ms)에 닿은 콜백은 붙듦을 걸지 않는다.** 처음 판본(`8e0df626`)의 `call_spawned` 는 상한에 닿아도 `guest_hold_others(true)` 를 걸고 끝에 지웠다. 붙듦은 칸이 하나(`ArmCore::hold_others` = `holder.swap`)라, 250ms 내내 잘려 있던 스레드가 붙든 키 · 그리기 처리기였으면 그 처리기는 남은 구간을 붙듦 없이 돌았다 — `0430` · `0506` 의 보호가 벗겨진다. 이제 상한에 닿으면 `main` 과 같은 호출 하나로 들어간다.

측정은 새 head + 계측(`fix/scratch-instrument.diff`) release 빌드 하나에서 세 팔을 손잡이로 가르고, rep 마다 세 팔을 같은 시각에 돌렸다(에뮬레이터 3 · short 임대 1건당 6런 · `fix/measure.sh`): «main» = `WIE_SCRATCH_NOFIX=1`(§1), «처음 판본» = `WIE_SCRATCH_OLDCAP=1`(상한에서도 붙든다 = `8e0df626`), «지금» = 손잡이 없음. «가져감» = 콜백의 `hold_others(true)` 가 다른 스레드의 붙듦을 덮은 런 — 검수가 얹은 계수기 그대로다. 실행 인자는 §1 과 같다(강제 · 탐침 끔).

| 판 | 팔 | 런 | 상한에 닿은 런 | 가져간 런 | 그리기 수 |
|---|---|---|---|---|---|
| `503d5d2f196b` | main | 8 | 0 | 0 | 145 ~ 153 |
| | 처음 판본 | 8 | 8 | **8** | 91 · 146 ~ 153 |
| | 지금 | 8 | 8 | **0** | 143 ~ 153 |
| `155972cac664` | main | 4 | 0 | 0 | 312 ~ 330 |
| | 처음 판본 | 4 | 4 | 0 | 311 ~ 330 |
| | 지금 | 4 | 4 | 0 | 314 ~ 330 |

- 검수의 수(처음 판본 8/8 런에서 1회씩 · main 0/7)가 그대로 재현되고, 지금은 0/8 이다. load1 10.8 ~ 15.8.
- `155972cac664` 는 처음 판본에서도 0/4 다(검수 0/4 와 같다 — 잘려 있던 쪽이 붙든 스레드가 아니다).
- 콜백을 띄우는 15판(검수의 목록 `review/sel.txt`) × 1런 · 지금: 콜백 105 · 가져감 0 · 남의 붙듦을 지운 `hold_others(false)` 0 · 붙듦 대기 0 · 상한 도달 2(위 두 판 · 각 1). §6 에 남긴 길(붙든 처리기가 막는 호스트 호출에 서 있을 때)도 이 105 콜백에서 0 이다. load1 15.4 ~ 29.4.
- 처음 판본 팔의 한 런이 그리기 91 이다(나머지 146 ~ 153). 15판 묶음에서 «지금»으로 혼자 돈 같은 판도 79 였다(load1 22.0 ~ 27.5) — 이 판의 그리기 수는 호스트 부하에 크게 움직이고, 91 을 가져감의 증상으로 가리지는 못했다.

**강제 재현 한 조건**(강제 · 굶기지 않음 · 훅 없음 · 6런씩 · 세 팔 동시 · §1 의 손잡이와 셈):

| 팔 | 겹침 · 오류 · 넘침 / 런 | 강제가 슬라이스를 끊은 런 | 기다린 콜백 | 기다림 최대 |
|---|---|---|---|---|
| main | **5 · 2 · 2 / 6**(FAIL 2) | 6 | — | — |
| 처음 판본 | 0 · 0 · 0 / 6 | 6 | 24 / 30 | 28ms |
| 지금 | **0 · 0 · 0 / 6** | 6 | 24 / 30 | 37ms |

겹침 0 이 유지된다(load1 11.3 ~ 19.9). 상한 도달 0 · 붙듦 대기 0.

**F2 — 상한 길의 시험.** `wie-wipi-c` `a_spawned_callback_runs_at_the_cap_and_leaves_the_hold_alone`: 다른 스레드가 끝내 안 풀리고 누군가 이미 붙들고 있는 모형에서 ⑴콜백이 돈다 ⑵`others_preempted` 를 정확히 250번 묻는다(상한 고정 = 키의 값) ⑶붙듦 칸이 그대로다.

**F3 — `MC_netConnect` 콜백도 기다림을 받는다(고쳤다).** 그 콜백(`net.rs` `Callback { delay: true }`)은 본문 «안»에서 1ms 를 잔 뒤 게스트 함수를 불렀다 — `call_spawned` 의 기다림은 그 잠 «앞»에서 끝나므로, 자는 동안 다른 스레드가 다시 게스트 코드 도중에 잘려 있어도 그대로 들어갔다(검수의 탐침에서 `entered_mid_code = true` · `main` 대비 회귀는 아니다). 이제 잠은 제 스레드에서 자고, 그 스레드는 게스트를 부르지 않은 채 «지연 없는» 같은 콜백을 새로 spawn 한다 — 호출은 다른 콜백과 똑같이 기다림 + 붙듦을 받는다. 1ms 는 그대로다(«늘 그랬듯»). 바뀌는 것: 그 콜백이 기다림을 한 번 더 거친다(자는 스레드가 들어가기 전 + 호출 전 · 각각 상한 250ms). 시험 `connect_callback_is_spawned_after_its_sleep_test`. 이 콜백을 실제로 받는 판에서의 전후는 재지 않았다 — 위 15판의 계수(`spawns`)는 콜백 종류를 가르지 않는다.

**F4 — 배선 두 줄의 시험.** `wie-ktf` · `wie-lgt` 에 `a_spawned_callback_runs_held` 하나씩: 실제 `KtfWIPICContext` / `LgtWIPICContext` 의 `spawn` 으로 띄운 콜백이 붙든 채 도는지 본다(호출 자리만 잠근다 — 기다림과 상한은 `wie-wipi-c` 의 시험).

**개악 → red**(각 개악을 얹고 그 크레이트의 시험을 돌렸다 · `fix/mut.py` · `fix/mut.out`):

| 개악 | 결과 |
|---|---|
| 없음 | `wie-wipi-c` 95 passed / 0 failed |
| M1 기다림 루프 제거 | red `left: 3 · right: 0` · 상한 시험도 `left: 0 · right: 250` |
| M2 `hold_others(true)` 제거 | red «ran holding the others» |
| M3 `hold_others(false)` 제거 | red «and let go after» |
| M5 상한 조건 제거(끝없는 기다림) | red «the wait is capped»(검수에서는 green 이었다) |
| M6 상한에서도 붙듦(= `8e0df626`) | red «the holder's hold is still there» |
| M7 상한 250 → 500 | red `left: 500 · right: 250` |
| M8 `MC_netConnect` 가 잠 뒤 바로 게스트 호출(= `8e0df626`) | red «no guest call from the thread that slept» |
| M4 `wie-ktf` `SpawnProxy` 를 옛 호출로 | red `a_spawned_callback_runs_held`(검수에서는 `cargo test --all` green 이었다) |
| M4 `wie-lgt` `SpawnProxy` 를 옛 호출로 | red `a_spawned_callback_runs_held` |

**나머지 발견의 처분**

- F5 — §4 를 고쳐 적었다: 상한 도달은 드문 일이 아니라 그 두 판의 매 런이다(이 회차 «지금» 팔도 8/8 · 4/4).
- F6(기다림 루프와 250 이 `wie-midp` 와 두 벌) — 두지 않고 남긴다. `System` 으로 올려 한 벌로 만들려면 `wie-midp` 의 키 · 그리기 길(`display.rs`)을 고쳐야 하고, 그 길의 무변경이 원 티켓의 조건이다. 값이 갈리는 것은 M7 이 잡는다(시험이 250 을 못박는다).
- F7(증적 재현성) — ⑴`scratch-instrument.diff` 를 다시 떴다: `8e0df626` 트리에 `patch -p1` rc 0 으로 얹히고 `wie-util/src/scratch.rs` 를 만든다. 그렇게 만든 트리가 §1 ~ §4 를 잰 트리(`main` + 옛 패치 + 손으로 뗀 파일)와 파일 단위로 같음을 확인했다 ⑵기준이 이 브랜치다(처음 것은 `main` 기준이라 제품 변경이 섞여 있었다). 수정 회차의 패치 `fix/scratch-instrument.diff` 는 새 head 기준이고 `git apply --check` 로 확인했다 ⑶`/tmp` 심링크를 지웠다 — `mklinks.sh` 가 코퍼스에서 `<sha12>.zip` 링크를 다시 만든다(이름을 증적에 두지 않는다).
- F8 — §6 의 마지막 항을 고쳐 적었다.
- F9(`SpawnProxy` 는 콜백이 `Err` 면 `?` 로 나가 `detach_thread()` 를 건너뛴다 · 이 PR 이전부터) — 고치지 않았다. 그 `Err` 는 태스크의 결과로 `Executor` 의 tick 이 그대로 돌려줘 런이 거기서 끝난다 — 건너뛴 detach 를 볼 다음 차례가 없다. 이 PR 이 바꾼 줄의 동작도 전과 같다.

**회귀(새 head)**

- 네 게이트 + `cargo +beta clippy --all -- -D warnings`: rc 0 · 0 · 0 · 0 · 0. `cargo test --all` 67 바이너리 · 990 passed / 0 failed(처음 판본 986 + 새 시험 4). beta 는 `wie_cli` 의 매니페스트 경고 7줄(안 쓰는 의존성)을 찍는다 — lint 가 아니고 rc 0 이며 이 PR 이 건드린 것이 아니다.
- 엔진 러너 6종(디버그 · short 임대 1건): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me` PASS · `keydraw_ktf` PASS · paints 79 · rc 0 · `keydraw_lgt` PASS · paints 55 · rc 0.
- 게임 랩 게이트 `scripts/smoke_gate.sh`(ktf + lgt · 기준선 242행 · §5 와 같은 16부 · 이 head 의 디버그 `wie_validate`): 16부 전부 `OK: no regressions vs baseline.` — 확인 242 · 부재 0 · 회귀 0 · 돌린 판 244 PASS / 0 FAIL. long 풀(1칸)을 다른 레인의 스윕이 25분 넘게 잡고 있어, 임대 1건당 3부(약 15분)로 짠 드라이버(`fix/smoke-phases.sh`)를 «short 임대 1건당 1부(약 4분)»로 돌렸다 — §5 와 같은 꼴 · 에뮬레이터 1 · load1 3.4 ~ 17.9.
- wasm 계약 3종(로컬 `scripts/build-wasm.sh` rc 0 뒤): `check-engine-contract.mjs` rc 0(`OK — engine surface matches the pinned featurephone contract.`) · `check-audio-worklet.mjs --require-soundfont` rc 0 · `contract-roundtrip.mjs` rc 0(`OK — boot round-trip matches the pinned featurephone contract.`). 내보내는 표면은 바뀌지 않았다.

<!-- corpus-name-inflow v1 subjects=7 tree=94056e951fca4992 B=3/2 P=0/0 S=0/0 -->
