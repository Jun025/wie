## [2026-10-05] 게임 스레드 사망 재판정 — 지원 현황 반영 · 걸린 벽 수정 (wie-compat-apply-guest-thread-death-rejudge-and-five-walls)

**무엇을**: #467 의 새 판정기(게임 스레드 uncaught 사망 = longplay `error`)로 «거짓 생존» 5종 + `8d8c24b7c198` 을 현행 `main` 에서 600초 짝으로 다시 쟀다. 여전히 죽는 3종의 벽을 고쳤다. 엔진 변경 셋: 게스트가 종료하면 그 뒤로 어떤 게스트 스레드도 돌지 않는다(`System::exit` · `exit_from_guest`), KTF 배열 클래스가 Object 의 vtable 을 갖는다, LGT ABI 행 4개.
**판정**: 6종 모두 최종 빌드에서 600초 사망 0. `compat.json` 은 `8d8c24b7c198` 한 행만 바뀐다(나머지 5행은 이미 `longplay: ok` 이고, 이 PR 이 그 표기를 참으로 만든다).
**사용자 영향**: 날짜를 쓰는 LGT 게임 하나가 날짜를 읽는 순간 멈추지 않는다. 배열에 `equals` 를 부르는 KTF 게임이 그 자리에서 멈추지 않는다. 메뉴에서 «종료»를 고른 SKT 게임이 종료 도중 오류를 남기지 않는다. 오래 하면 멈추던 KTF 게임 하나가 지원 현황에서 «플레이 가능»으로 돌아온다(#469 가 고친 것을 현황에 반영).

타이틀은 sha12 로만 적는다. 측정 바이너리·로그: `~/scratch/gtd/`(레포 밖).

### 1. 재판정 재확인 — 6차 기록(`~/scratch/w6census/out`)을 현행 판정기로 `report`

| | longplay ok | error | n/a | playable | limited | not-yet |
|---|---|---|---|---|---|---|
| #467 회신(후) | 383 | 14 | — | 382 | 30 | — |
| 이 회차(재실행 · 판정기 `9b4eb62e`) | 383 | 14 | 32 | 382 | 30 | 17 |

**재확인 동일**: `error` 14건 중 사망줄 신규 5건이 같은 5건(`6c9f969f089f` `6eb93824daf8` `73f3a21e981c` `9789fec50f39` `d3e3b16cefd0`)이고, 나머지 9건은 원래 `error` 였다. 추가 변화 0.

### 2. 벽과 처방

| sha12 | 벽 (현행 `main`) | 원인 | 처방 |
|---|---|---|---|
| `6c9f969f089f` KTF | 주소 0 으로 분기(`lr=0x121c7f`) | 게임이 `int[]` 에 `equals(Object)` 를 부른다. 배열 클래스는 `ptr_vtable = 0` 이라 가상 디스패치가 null 표를 읽고 0 으로 뛴다. 게다가 null 포인터는 `get_vtable_index` 의 표에서 아무것과도 맞지 않아, 배열 클래스마다 «다음 클래스가 쓸» 자리를 받았다 | `JavaArrayClassDefinition::new` 가 일반 클래스처럼 `JavaVtable::new` 로 자기 vtable(Object 의 메서드들)을 만든다 |
| `73f3a21e981c` LGT | `GregorianCalendar vtable index 28` → (28 을 채우면) 22 → `Date` 10 → `StringBuffer` 30 | 날짜를 만들어 읽는 한 함수(0x5b9c0~)의 슬롯들 | 아래 §3 의 ABI 행 4개 |
| `d3e3b16cefd0` SKT | `NullPointerException` in the game thread's `run()` | 게임 루프가 `destroyApp`(필드를 null 로) → `notifyDestroyed` 를 부르고, 호출이 **돌아오자** 루프가 그 필드를 읽는다. 사망은 `notifyDestroyed` 2 ms 뒤(디버그 로그로 순서 확인) | 종료는 그 뒤로 어떤 게스트 스레드도 돌리지 않는다: `Executor::halt`(다른 태스크) + 부른 스레드는 돌아가지 않는다(`System::exit_from_guest`) |
| `6eb93824daf8` KTF | 현행 `main` 에서 재현 안 됨(2회 600초 생존) | 6차 핀 이후 착지분이 고쳤다. 6차의 벽(주소 0 분기 · null 읽기 0x40)은 `6c9f969f089f` 와 같은 모양이다 | — |
| `9789fec50f39` KTF | 현행 `main` 에서 재현 안 됨(2회 생존) | r2(#458) 이후 남지 않는다 | — |
| `8d8c24b7c198` KTF | 현행 `main` 에서 2회 생존 | #469(KTF 게스트 GC 루트)가 고쳤다 | compat 행만 갱신 |

**종료 처방의 범위**: 게스트 종료 경로 넷(`MIDlet.notifyDestroyed` · WIPI-C `MC_knlExit` · LGT `terminate_program` · RustJava `Runtime.exit`)이 모두 `System::exit` 를 거친다. 앞의 셋은 `exit_from_guest` 로 부른 스레드도 멈춘다. `Runtime.exit` 은 동기 트레잇 메서드라 다른 스레드만 멈춘다. 태스크는 버리지 않고 남긴다 — 해제는 지금처럼 호스트의 `teardown` 몫이다. `midlet.rs` 의 기존 주석(종료가 스텁이던 시절, 종료하는 타이틀이 다시 불러 스택이 넘쳤다)도 같은 방향이다.

**진단 한 줄**: `arm32_cpu` 가 null 분기 직전의 `lr` 을 `error` 로 남긴다(`guest branched to 0x0, lr=…`). 호출자의 레지스터 덤프는 문맥 복원 뒤라 그 자리를 잃는다 — 바로 아래 undefined-instruction 메시지와 같은 이유. `6c9f969f089f` 의 호출 지점은 이 한 줄로 찾았다.

### 3. LGT ABI 행 (`wie-lgt/data/lgt_java_abi.toml`) — 호출 지점 역어셈블(`objdump --triple=armv5te`)

| 클래스 | 인덱스 | 메서드 | 근거 |
|---|---|---|---|
| `java/util/Calendar` | 28 | `set(II)V` | 워드 0x74 를 `(1,y)(2,m-1)(5,d)(10,0)` 로 네 번 — YEAR MONTH DAY_OF_MONTH HOUR. 두 int 를 받는 메서드는 이것뿐 |
| `java/util/Calendar` | 22 | `getTime()Ljava/util/Date;` | 바로 다음 워드 0x5c · 인자 없음 · 결과를 null 검사 후 보관 |
| `java/util/Date` | 10 | `getTime()J` | 그 결과에 워드 0x2c · 인자 없음 · r0·r1 둘 다 쓴다(long). CLDC Date 의 첫 가상 메서드 = 배치 규칙과 일치 |
| `java/lang/StringBuffer` | 30 | `insert(ILjava/lang/String;)…` | r0=227(문자열 풀 인덱스) 헬퍼 결과를 r2, r1=0 — 기존 34(`insert(II)`)와 같은 숫자 루프의 `insert(0, ",")`. CLDC 순서에서 `insert(I,Object)` 다음 |

### 4. 전/후 — 600초 L 짝(센서스 L 과 같은 인자: `long.keys` · `--keep-timeout --timeout 600 --max-ticks 1e11 --shot-every 20 --relaunch 1`)

전 = `origin/main` `9b4eb62e` + `lr` 로그 한 줄 + Calendar 28 행(LGT 전용이라 KTF·SKT 다섯 종에는 무관). 그래서 `73f3a21e981c` 의 «전»만은 순수 `9b4eb62e` 로 따로 쟀다. 후 = 이 브랜치 최종(`origin/main` `e8b2b1e7` 위). 사망이 나오면 그 자리에서 멈췄다(판정이 이미 `error`).

| sha12 | 전 1 | 전 2 | 후 1 | 후 2 |
|---|---|---|---|---|
| `6c9f969f089f` | 사망 · 주소 0 | 사망 · 주소 0 | 600초 · 사망 0 | 600초 · 사망 0 |
| `6eb93824daf8` | 600초 · 사망 0 | 600초 · 사망 0 | 600초 · 사망 0 | 600초 · 사망 0 |
| `73f3a21e981c` | 사망 · Calendar 28 | 사망 · Calendar 28 | 600초 · 사망 0 | 600초 · 사망 0 |
| `9789fec50f39` | 600초 · 사망 0 | 600초 · 사망 0 | 600초 · 사망 0 | 600초 · 사망 0 |
| `d3e3b16cefd0` | 사망 · NPE | 사망 · NPE | 600초 · 사망 0 | 600초 · 사망 0 |
| `8d8c24b7c198` | 600초 · 사망 0 | 600초 · 사망 0 | 600초 · 사망 0 | 600초 · 사망 0 |

생존 실행은 모두 마감까지 갔고 키 854/900 단계를 받았다(6차 L 과 같은 수). 화면 수(후 1·2): `6c9f969f089f` 4,122·3,750 · `6eb93824daf8` 26,612·19,848 · `73f3a21e981c` 13,133·12,515 · `9789fec50f39` 4,555·4,600 · `d3e3b16cefd0` 8,040·8,004 · `8d8c24b7c198` 76,255·65,195.

여섯 축(compat, 후): 여섯 타이틀 모두 boot·render·input·longplay `ok`. sound·speed 는 이 회차가 다시 재지 않았다(6차 값 그대로 — `6c9f969f089f` sound `no`). `wie_validate` 의 `result` 는 이 키 경로에서 `UNMEASURED` 로 찍힌다(6차 L 기록도 같다). 판정기는 `FAIL` 이 아니면 사망 수만 본다.

이 측정은 센서스 대신 `wie_validate` 를 직접 돌렸다: 다른 레인의 진도 센서스가 호스트 잠금을 쥐고 있었다. 한 번에 2개 · `build-slot` · `host-load-guard` 가 rc≠0(«recovering»)이라 폭 ≤ 2. 인자는 센서스 L 과 같다. 환경 변수 `RUST_LOG`·`RUST_MIN_STACK` 는 붙이지 않았다(사망줄은 `error` 라 그대로 보인다).

### 5. 되돌리면 red

| 처방 | 시험 | 되돌렸을 때 |
|---|---|---|
| `Executor::halt` | `test_halt_stops_every_task_including_later_ones_in_the_same_step` | `left: 28, right: 0` |
| `exit_from_guest` (`notifyDestroyed`) | `notify_destroyed_exits_and_never_returns_to_the_guest` | `notifyDestroyed returned to the guest` |
| KTF 배열 vtable | `test_array_classes_carry_object_vtable` | `[I has no vtable` |
| ABI 행 | `abi_rows_cover_the_indexes_titles_actually_dispatch_on` (4행 추가) | `Calendar` 22 행 제거 시 패닉 |

종료가 돌아오지 않게 되면서 «종료 호출이 끝나기를 기다리는» 기존 시험 둘(`wipic_0x68_terminates_the_program` · Jlet `notify_destroyed_exits_without_destroy_app`)이 멈춘 채 돌았다. 둘 다 «종료했다 + 돌아오지 않았다»를 확인하도록 바꿨다. JVM 쪽 둘(MIDlet·Jlet)은 `test_utils::run_jvm_test_until_exit` 를 쓴다(종료까지 tick, 그 뒤 10 tick, `func` 이 돌아왔는지 반환). Jlet 시험의 `destroyApp` 은 정적 필드에 세던 것을 패닉으로 바꿨다 — 호출이 돌아오지 않으니 센 값을 읽을 자리가 없다.

게이트(최종 head): fmt OK · clippy stable·wasm32·beta `-D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 691 passed · 0 failed · `npm run build:wasm` rc=0 · `check-engine-contract` 113 pass · `npm run audit` 통과.

### 6. 퇴행 — 프로브 A 30초 짝(센서스 A 와 같은 인자) · 전 `9b4eb62e` · 후 최종

라이브 LGT 5(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) + 가드 2(`49ade89578c5` `ddd885583b15`) + KTF 8 · SKT 4(compat `playable` 에서 등간격): 19쌍 모두 `result`·`stop`·사망 수 같음(사망 0 → 0), 키 도달 ±1, 화면 수 ±16. `max-ticks` 로 끝나는 9쌍은 양쪽이 똑같이 그렇다. 커밋된 runner 블록 6줄 PASS(keydraw 2줄 rc=0).

### 7. compat 행

`8d8c24b7c198`: `longplay` `no` → `ok`, `status` `limited` → `playable`(규칙대로), `knownIssues_ko` 에서 «플레이 도중에 게임이 멈추거나 꺼질 수 있어요.» 제거. 나머지 다섯 행은 바꾸지 않는다 — 이미 `longplay: ok` 이고, 이 PR 이 착지하면 그 표기가 참이다. 업데이트 소식은 이 PR 이 고친 세 타이틀에 한 줄(`docs/player-updates/2026-10-05-game-thread-walls-three.json`). `8d8c24b7c198` 은 #469 의 소식이 이미 있다.

### 8. 한계

- 종료 뒤 태스크를 남겨 두므로, 종료를 무시하는 호스트(종료가 no-op 인 `wie-web`)에서는 게임이 그 자리에서 멈춘 채 남는다. 지금 셸·`wie_validate`·데스크톱은 모두 종료로 끝내거나 다시 띄운다.
- 30초 프로브 A 에서의 사망줄(#467 회신 한계 3)은 범위 밖이다.
- 6차 핀 이후 착지분이 고친 두 타이틀(`6eb93824daf8` `9789fec50f39`)은 어느 PR 이 고쳤는지 이분 탐색하지 않았다.

### 9. 유입

`node scripts/corpus-name-inflow.mjs`: 유입 371쌍(BOUNDED) · 판단 필요 20쌍(SUFFIX-ATTACHED). 전부 이 PR 이 손댄 파일에 **원래 있던** 이름이다 — `compat.json`(공개 제목 표, 계약상 제목을 싣는다)과 `lgt_java_abi.toml`·`wie-lgt` 시험·`jlet.rs` 시험의 기존 주석. 이 PR 이 더한 줄에는 게임명이 없다(주석은 sha12 로 적었고, 처음 쓴 이름 셋은 커밋 전에 sha12 로 바꿨다).
