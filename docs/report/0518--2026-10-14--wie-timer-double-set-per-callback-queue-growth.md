## [2026-10-14] WIPI 타이머 — 대기 중인 타이머에 다시 건 `MC_knlSetTimer` 는 실패한다 (wie-timer-double-set-per-callback-queue-growth)

**무엇을**: `MC_knlSetTimer` 가 아직 대기 중인(걸렸지만 아직 발화도 Unset 도 되지 않은) 타이머에 다시 불리면 **-1 을 돌려주고 기한을 바꾸지 않게** 했다. 바뀌는 것은 param 하나다. 그 Set 의 param 은 대기 중인 콜백에 전달된다. 종전에는 Set 두 번이 «콜백 두 개»가 됐다(`event_queue.rs` `arm_timer` 주석: 「two Sets give two callbacks, as they always did」). 그래서 콜백마다 같은 타이머를 두 번 거는 게임 3종(#549·0516 의 군집 D)은 대기 중인 콜백이 콜백마다 1개씩 늘었다.

**왜**: 운영자 Tower 채택 `2026-10-13-playable-speed-unknown-42-remeasure#p0`. 티켓 Acceptance 1(근거 먼저)은 아래 §1 의 참조 구현이 답했다. 그 답은 티켓이 가정한 «교체/추가» 둘 중 어느 쪽도 아닌 **«거절»** 이다.

**사용자 영향**: 3종(`cdd3eb5f9142` `4892a1abc0f8` `1cd151222bde`)이 명목 속도로 돈다. 종전에는 대기열이 쌓여 명목보다 빨랐다. 지원 현황의 속도 칸이 «알 수 없음» → «정상»이 된다. 다른 타이머 게임 107종은 실행 경로가 같다(§3).

### 1. 근거 — 참조 에뮬레이터의 `MC_knlSetTimer`
`game_lab/vendor_sdk/ezi/AromaWIPI-Emulator1[1].1.1.80/` 의 설치본(2004-05-13 · ReleaseNote 「WIPI spec 1.1.1 준수」)을 7-Zip 으로 풀어 `Emulator/WIPIEmul.exe`(PE32 x86)를 capstone 으로 읽었다.
- **함수 찾기**: `.data` 에서 `.text` 를 가리키는 포인터 236개가 이어진 표는 `0x519b64` 하나뿐이다. 앞 칸 `0x519b60` 은 데이터 포인터(`Java_sp`)라서 이 표를 `MG_NativeEnv`(`WIPIHeader.h` — 설치본 `CAppDemo/include/WIPIHeader.h` 와 byte 동일)로 읽었다. 헤더 순서대로면 Def/Set/Unset 은 26/27/28번 칸이다. 그러나 27·28번 칸은 `org/kwis/msf/core/Shared;+shBuf` 를 참조한다(SharedBuf). 그래서 이 판본의 표는 헤더보다 두 칸 앞이다. 24번 칸은 `cb` 를 `+0x1c`·`prg` 를 `+0x18` 에 쓰고 `type=1` 로 둔다(Def). 25번 칸은 `parm` 을 `+0x14` 에 쓰고 `0x44781e(tm, lo, hi, wakeup)` 를 부른다(Set). 26번 칸은 `0x4478f6` 으로 목록에서 떼어 낸다(Unset). `+0x14/+0x18/+0x1c` 는 헤더 `MCTimer` 의 `parm/prg/cb` 오프셋과 같다.
- **`0x44781e`(알람 걸기)의 첫 분기**: `cmp [tm+8], 0 ; je … ; or eax, -1 ; jmp ret`. `tm+8` 은 `MTimer.alarmWakupFunc` 이고, 걸린 동안에만 0이 아니다. ⇒ **이미 걸린 타이머에 다시 걸면 -1 을 돌려주고 아무것도 바꾸지 않는다.** 기한 `+0xc` 도, 목록 위치도 그대로다. 단 `parm` 은 호출부(25번 칸)가 이 분기보다 **먼저** `+0x14` 에 써 둔다. 그래서 다음 발화는 마지막 Set 의 param 으로 간다.
- **발화(`0x447958`)**: 머리 노드를 목록에서 떼고 `tm+8` 을 0 으로 지운 **뒤에** wakeup(`0x4484ec` = `MV_TIMER_EVENT 0x1001` 을 `(tm, parm=[tm+0x14])` 로 게시)을 부른다. ⇒ 콜백 안에서 타이머는 «걸리지 않은» 상태다. 첫 Set 은 걸리고, 같은 콜백의 둘째 Set 은 거절된다.
- **구조**: `MCTimer` 는 `MTimer{type, alarmNext, alarmWakupFunc, alarmTime}` 를 품은 **침입형 단일 연결 목록 노드**다(`WIPIHeader.h:339-353`). 구조체 하나는 목록에 한 번밖에 들어갈 수 없다. «두 Set = 두 콜백»은 이 구조로는 표현할 수 없는 의미였다.
- 한계: 이것은 참조 **에뮬레이터**다. KTF·LGT 실기 펌웨어 바이너리는 이 저장소에 없다. 그래도 «추가» 의미의 근거는 어디에도 없었다(0384 도 「Set 교체를 요구하는 타이틀은 없었다」고 적었다). 반대로 이 참조 구현과 헤더 구조는 둘 다 «한 타이머 = 한 대기»를 가리킨다.

### 2. 바꾼 것
- `wie-backend` `EventQueue::arm_timer(timer, param) -> Option<u64>`: 대기 중이면 `None` 을 돌려준다(param 은 갱신). `cancel_timer` 는 대기를 지운다. `fire_timer(timer, arming, tick) -> Option<u32>` 는 그 arming 이 아직 대기 중일 때만 대기를 풀고 param 을 돌려준다. 이 함수가 종전의 `is_timer_armed` + `timer_fired` 를 대신한다. `armings/cancelled_through` 는 `pending: Option<u64>` 하나로 줄였다.
- `wie-wipi-c` `set_timer` 는 `Result<i32>` 를 돌려준다(0 / -1 — 참조 구현의 `xor eax,eax` / `or eax,-1`). 종전에는 반환값이 없어 r0 이 미정이었다. 거절은 `debug` 한 줄로 남긴다.
- 시험 `kernel::test::test_set_timer_on_a_pending_timer_fails`: Set·Set → 콜백 1개 / 둘째 Set = -1 / 대기 1개 / param 은 둘째 것 / 발화 뒤 재Set = 0. ★**되돌리면 red**: `arm_timer` 의 대기 중 분기를 지우자 이 시험만 FAILED 가 났다(69 pass · 1 fail). 기존 `test_when_a_timer_may_keep_a_tick_alive` 는 대기 중인 타이머에 Set 을 겹쳐 «다른 기한의 새 대기»를 기대했다. 그래서 그 두 곳에 `unset_timer` 를 넣었다(검사하는 페이스 판정은 그대로다).

### 3. 퇴행 전수 — 타이머 게임 111종 전·후 짝
- 모집단: 6차 census(`4ac38566` · `~/scratch/w6census/out`)에서 probe 의 `pacing.timers > 0` 인 111종. 2d5cada03004·3cc7a9b4cb15·601556233e71·79d9bc10be82(0384 의 Unset 4종) 포함.
- 바이너리: before = `origin/main` `435375c0` · after = 이 브랜치. 둘 다 release `wie_validate` 다. `playability-census.mjs run --only probe` → `--only speed` 를 `census-drive.sh` 로 돌렸다(before 23:57–01:16 · after 01:16–01:54 · load1 9~19).
- **거절이 실제로 일어나는 곳**(after · `--inject --timeout 30` · `RUST_LOG=wie_wipi_c::api::kernel=debug` 로 Set·거절 수를 셈): **4종뿐**이다. `cdd3eb5f9142` 1075 Set 중 537 · `4892a1abc0f8` 1065 중 532 · `1cd151222bde` 109 중 54 · `3cc7a9b4cb15` 176 중 1. 나머지 107종은 거절 0 이다. ⇒ 그 107종은 before·after 의 실행 경로가 같다. 그 행의 수치 차이는 순차 실행의 부하 차이다.
- 판정 변화(status·boot·render·input·sound·speed): **5행, 전부 speed `n/a → ok`**. 나빠진 행은 0이다.

| sha12 | 거절/Set(30s) | speed | 비율 | 타이머 발화(A) | late p50 ms |
|---|---|---|---|---|---|
| `cdd3eb5f9142` | 537/1075 | n/a → **ok** | −407.682 → 0.997 | 774 → 489 | 10766 → 0 |
| `4892a1abc0f8` | 532/1065 | n/a → **ok** | −390.844 → 1.000 | 757 → 527 | 11043 → 0 |
| `1cd151222bde` | 54/109 | n/a → **ok** | −397.990 → 1.000 | 1537 → 341 | 4167 → 0 |
| `601556233e71` | 0 | n/a → ok | 0.765 → 1.000 | 426 → 299 | 15 → 0 |
| `b44b5fbe29e6` | 0 | n/a → ok | 0.342 → 0.979 | 921 → 527 | 0 → 0 |

- 위 표의 마지막 두 행(거절 0)은 이 변경의 효과가 **아니다**. before 런의 load1 이 19 였고, compat 은 이미 둘 다 `ok` 다. 그래서 compat 에 반영할 것이 없다.
- 3종: late p50 이 4~11 **초**에서 0 으로 떨어졌다(쌓이던 대기가 사라졌다). 발화 수는 명목에 가까워졌다. 0516 의 debug 런은 cdd3 의 콜백을 57~64/s 로 셌다(명목 47.6/s).
- `3cc7a9b4cb15`(거절 1회): speed ok → ok(0.992 → 0.999) · sound ok → ok.
- 전 111행: `docs/report/0518--2026-10-14--wie-timer-double-set-per-callback-queue-growth--pair.csv`(sha12 만).
- 위 CSV 의 `status_*` 는 probe 만 돈 상태의 값이다(longplay 미측 ⇒ `limited`). 전·후 차이는 0이다.
- **progress**(거절이 일어나는 4종 · `--only long --long 120` → `--only progress --progress 300` · before 02:49–03:08 · after 03:08–03:27): 4종 모두 longplay `ok → ok` · progress `ok → ok` · status `playable → playable`. 나빠진 행은 0이다.

| sha12 | speed | progress | p_stall(B→A) | p_distinct(B→A) |
|---|---|---|---|---|
| `cdd3eb5f9142` | n/a → ok | ok → ok | 0 → 20 | 12 → 13 |
| `4892a1abc0f8` | n/a → ok | ok → ok | 0 → 20 | 11 → 13 |
| `1cd151222bde` | n/a → ok | ok → ok | 0 → 70 | 6 → 5 |
| `3cc7a9b4cb15` | ok → ok | ok → ok | 30 → 0 | 21 → 26 |

- 0384 의 Unset 4종 중 `79d9bc10be82` 는 6차 probe 의 창 안 타이머 발화가 0이라 111종에 들지 않았다. 그런데 30초 런에서 거절이 3회(Set 78) 나와 따로 짝을 냈다(probe+speed): status·boot·render·input·sound·speed 전부 같다(speed ok 0.998 → 1.000). 다만 paints 522 → 371 · 타이머 발화 162 → 102 로 줄었다. 런 1쌍이라 이것이 «종전에 더 돌던 사슬이 사라진 몫»인지 변동인지 가르지 못했다(debug 런 2회의 Set 수도 서로 13 vs 5 로 흔들렸다).

### 4. compat · 게이트
- `docs/player-data/compat.json` 3행의 `axes.speed` 만 `unknown → ok` 로 바꿨다(cdd3 · 4892 · 1cd1). 다른 필드·행은 0이다.
- 이용자 소식 `docs/player-updates/2026-10-14-timer-set-twice-runs-at-its-own-speed.json`(kind `speed`).
- fmt · clippy(`-D warnings`) · wasm clippy · `cargo +beta clippy` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 54 스위트 0 실패. 러너 블록(debug `cargo run`): draw_j2me·helloworld_ktf·helloworld_lgt PASS · keydraw_ktf/lgt `--inject --expect-last-frame` PASS rc=0 · text_j2me PASS. `check-engine-contract` 137 pass · 0 violation.
- release 바이너리로 keydraw 를 돌리면 before·after 둘 다 `UNMEASURED`(stop `max-ticks`)다. 이 변경과는 무관하다. 문서의 러너 줄은 debug 다.

### 5. 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs`: BOUNDED 331쌍 + SUFFIX-ATTACHED 16쌍. 전부 이 PR 이 건드린 `compat.json` 의 기존 제목 값이다. 이 회차는 compat 의 speed 값 3개만 바꿨다(제목 문자열 차이 0). 타이틀은 sha12 로만 적었다.


