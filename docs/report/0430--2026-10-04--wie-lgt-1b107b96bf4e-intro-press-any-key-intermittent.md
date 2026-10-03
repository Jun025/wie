## [2026-10-04] LGT `1b107b96bf4e` 대국 소개 간헐 정체 — 키가 다른 스레드의 그리던 프레임 한가운데로 들어갔다 (wie-lgt-1b107b96bf4e-intro-press-any-key-intermittent)

**무엇을**: 0421 §5 가 원인을 재지 않고 남긴 «대국 소개(Loading · Press Any Key) 정체»를 쟀고 고쳤다. LGT 에서는 키 처리와 다른 게스트 스레드의 코드가 서로의 한가운데로 끼어들지 않는다. ⑴ 다른 스레드가 «게스트 코드 한가운데서» 잘려 있으면 키를 넘기기 전에 그 스레드가 호스트 호출(sleep·wait·I/O)에 닿기를 기다린다. ⑵ 키 처리가 도는 동안 그 스레드가 명령 예산으로 잘리면, 다른 스레드는 게스트 코드로 돌아가지 않고 기다린다. 둘 다 상한이 있다.
**판정**: 정체는 게임 탓도, GC 탓도 아니다. wie 는 게스트 스레드를 1만 명령마다 자른다. 키 처리 스레드가 메뉴 이미지 필드를 null 로 비우는 순간, 게임 스레드가 그 메뉴를 그리던 중에 잘려 있으면 이어 그린 프레임이 그 null 을 읽는다. 게임 스레드는 NPE 로 죽고 화면은 거기서 멈춘다. 정체 실행은 모두 이 모양이었고 통과 실행은 모두 아니었다(§2).
**사용자 영향**: `1b107b96bf4e` 에서 대국을 고르고 키를 누르면 매번 대국이 시작된다(같은 키 20회: main 정체 5 ↔ 0 · §5). 다른 LGT 게임은 키가 그 순간에 걸렸을 때만 몇 ms 늦게 들어간다(§4). KTF·SKT·J2ME 는 바뀌지 않는다.

증적: `~/orchestrator/reports/evidence/wie-lgt-1b107b96bf4e-intro-press-any-key-intermittent/`. 타이틀은 sha12 로만 적는다.

### 1. 재현율 — main(`d1ba7687`) release `wie_validate`

키는 `docs/keys/lgt-1b107b96bf4e-match.keys`. 정체는 결과 JSON 의 `java_exceptions` 7(통과는 6)과 `paints` 급감(약 1,950 → 720)으로 판정했다. 화면으로도 대조했다(증적 `stall-intro.png` · `pass-intro.png`).

| 키 | 실행 | 정체 |
|---|---|---|
| 문서 키 전체 + 대국 6회 · 120초 · 3개 동시 | 12 | **1** |
| 문서 키 1행 + OK 2 · 75초 · 계측 빌드 | 16 | 1 |
| 소개 직전 간격 2초 → 0.3초 | 8 | 3 |
| 소개 직전 간격 2초 → 8초 | 8 | 2 |

- 7번째 예외는 늘 같다: 게스트가 직접 낸 `java/lang/NullPointerException`(메시지 없음 = LGT AOT 의 null 검사 SVC `0x22`).
- 그 예외는 «Press Any Key» 를 누른 그 키에서 난다. 소개 화면을 8초 기다린 뒤 눌러도 난다(2/8). ⇒ «로딩이 덜 끝났는데 눌렀다»가 **아니다.**

### 2. 원인 — 스크래치 계측(커밋 안 함 · 증적 `scratch-probes.patch`)

1. **어디서**: `java_raise_null_pointer_exception` 에 레지스터·호출 스택 덤프를 붙였다. 게스트 LR `0x543d3`. 그 바이너리의 메서드 표(`.data`)로 함수 시작 `0x53f00` 의 이름을 찾았다(증적 `lgtsym.py`). 게임 클래스의 메뉴 그리기 메서드다. 7×N 칸을 도는 그리기 루프가 `this` 의 이미지 배열 필드(필드 표 칸 `0x16a`)를 읽는다. 그 필드가 0 이다.
2. **누가 null 로**: 그 객체(게임 `Card`, 매 실행 `0x488458c0`)의 필드 블록 쓰기를 엔진의 `w32` 에서 감시했다. 이 필드는 **모든 실행에서** 두 번 쓰인다. 부팅 때 이미지 생성 메서드가 채우고, «Press Any Key» 때 이미지 해제 함수(`pc 0xa696`)가 0 으로 비운다. 그 뒤 게임은 `System.gc()` 를 네 번 부른다. ⇒ 필드가 null 이 되는 것은 정상이다. 문제는 그 뒤에도 메뉴 그리기가 한 번 더 돈 것이다.
3. **어느 스레드가**: 해제는 **스레드 2**(이벤트 루프 — 키 처리)에서, NPE 는 **스레드 3**(게임 `run()` 루프)에서 났다. 간격은 약 2ms 다. 객체는 살아 있다: 수집기에 등록돼 있고 머리도 멀쩡하다(증적 `holder.txt`). ⇒ 과수거(GC)가 **아니다.**
4. **그 순간 스레드 3 은 어디 있었나**: 해제 시점에 스레드 3 이 마지막으로 멈춘 PC 를 기록했다(증적 `thread-trace.txt`).

| 실행 | 스레드 3 이 멈춘 자리 |
|---|---|
| 통과 10/10 | 호스트 호출 안(`run()` 의 sleep · LR `0xae0d`) |
| 정체 2/2 | **메뉴 그리기 루프 한가운데**(PC `0x54490` · `0x543fa`) — 명령 예산이 다해 잘렸다 |

⇒ 게임 스레드가 메뉴 프레임을 그리던 중 1만 명령 예산으로 잘린다. 그 사이 키 처리가 이미지를 비운다. 프레임이 이어 그리다 null 을 읽는다. 단말에서는 키 처리와 게임 스레드의 프레임이 이렇게 겹치지 않는다. 그래서 이 게임은 비우는 쪽에 잠금이 없다.

**기각한 가설**:
- 첫 수정(«다른 스레드의 MIDP `paint()` 중에는 키를 넘기지 않는다»)은 정체 2/12 로 반증됐다. 이 메뉴는 `paint()` 가 아니라 게임 `run()` 안에서 그린다.
- GC 독 채우기 모드(`--gc-stress 2000000000`, 해제 블록 `0xdeaddead`)는 정체 0/10 · 독 접근 0 · dangling 0 이었다. 그러나 이 모드는 실행이 약 20% 느려 시간 축이 바뀐다. 위 3번(객체가 살아 있다)과 함께 GC 설을 버렸다.

### 3. 처방

| 자리 | 바꾼 것 |
|---|---|
| `wie-core-arm` `ArmCore::run_function` | 명령 예산으로 멈출 때(`EngineStopReason::Yield`, SVC 가 아님) 그 스레드에 `preempted` 를 세우고 다시 돌 때 내린다. `others_preempted()` 는 «지금 스레드 말고 잘린 스레드가 있나»를 답한다 |
| 〃 `hold_others(on)` · `wait_for_holder` | `on` 인 동안 지금 스레드가 «붙든 스레드»다. 그 스레드가 잘려 있는 동안 다른 스레드는 루프 머리(게스트 코드로 들어가기 직전)에서 양보만 한다. 붙든 스레드가 호스트 호출 안에서 기다리면 막지 않는다. 한 번 기다릴 때 최대 200 회(`HOLD_ROUNDS`) |
| `wie-backend` `TaskRunner::others_preempted` · `hold_others` | 기본은 `false` · 아무것도 안 함. `System::guest_others_preempted` · `guest_hold_others` 가 부른다 |
| `wie-lgt` `LgtTaskRunner` | 둘 다 `ArmCore` 로 구현 — **LGT 만** |
| `wie-midp` `Display::handleKeyEvent` | ⑴ 잘린 스레드가 있으면 1ms 씩 잔다. 최대 250ms(`KEY_WAITS_FOR_GUEST_MS`) 뒤에는 종전처럼 넘긴다. 기다리면 debug 한 줄을 남긴다. ⑵ `routeKeyEvent` 동안 `guest_hold_others(true)` |

- **왜 ⑵ 가 필요했나**: ⑴ 만 넣은 빌드는 같은 키 20회에서 **1회 정체**했다(main 2회). 키 처리 자체가 게스트 코드다. 키가 들어와서 필드를 비우기까지 키 처리 스레드는 **25~45회** 잘렸다(계측 16회 전부). 그 틈마다 게임 스레드가 sleep 에서 깨어 새 프레임을 시작할 수 있다. ⑴ 은 «키가 들어오는 순간»만 지킨다.
- **왜 키 쪽에서 막나**: 선점 자체를 없애면(LGT 를 협조식으로) 호스트 호출 없이 도는 게임이 다른 스레드를 굶긴다. 바뀌는 범위도 전 LGT 다. 키는 한 지점이다. 게임 스레드의 실행 순서는 바뀌지 않고, 키 처리의 게스트 코드만 «다음 호스트 호출까지 한 덩어리»로 돈다.
- **왜 상한이 있나**: 호스트 호출 없이 계속 도는 스레드가 있으면 그 스레드는 늘 «잘린» 상태다. 상한이 없으면 ⑴ 은 키가 영영 안 들어가고, ⑵ 는 다른 스레드가 굶는다. 상한에 닿으면 종전과 같다.
- **KTF·SKT·J2ME**: `TaskRunner` 기본값이라 ⑴ 은 첫 검사에서 빠지고 ⑵ 는 아무것도 하지 않는다. `wait_for_holder` 는 붙든 스레드가 없으면 원자 읽기 한 번으로 돌아온다.

### 4. 퇴행 — LGT 78종 짝(main ↔ 이 브랜치 · 기본 27키 `--inject` · 교대 · 3개 동시 · 게이트 rc=0 후 시작 · load1 18~20)

증적 `pair-78.tsv`(⑴ 만 넣은 빌드의 앞선 짝은 `pair-78-step1-only.tsv`).
- 결과: main PASS 35 · UNMEASURED 42 · FAIL 1 ↔ 이 브랜치 PASS 33 · UNMEASURED 44 · FAIL 1.
- (결과·stop·content·예외 수)가 다른 4종 — 셋 다 `--max-ticks 100000000000` 로 3회씩 짝 재측하면 같다:
  - `0093012b8c36` · `af7d82e5e239` — PASS → UNMEASURED. 두 실행 모두 `max-ticks` 백스톱이 키 25/27 · 20/27 에서 끊었다. 이 실행에서 키 대기는 0회다. 재측: 양쪽 모두 PASS · deadline · 27/27 · paints 같은 범위(289~303 · 286~292).
  - `6b515884dbc1` — PASS 그대로 · `stop` 만 `deadline` ↔ `max-ticks`.
  - `73f3a21e981c` — PASS 그대로 · 이 브랜치 쪽에 게스트 NPE 1회. 재측에서는 다시 나지 않았다: 이 브랜치 0/33 · main 0/17(증적 `73f3a21e981c-reruns.tsv`). 짝 실행 한 번에서만 났다.
- **키가 기다린 게임 7종**: 키당 최대 **41ms**(`b475b6399684`), 나머지는 11ms 이하다. 250ms 상한에 닿은 키는 0 이다. main 쪽 대기는 0(당연).
- 0421 의 교착 가드 `b475b6399684`(paint 가 다른 스레드를 기다리는 게임)는 16회 기다렸다(합 152ms). 결과·paints(519 ↔ 523)가 같다.
- ⑴ 만 넣은 앞선 짝(같은 78종)도 같은 모양이었다: 판정 변화 4종 중 셋은 `stop` 흔들림이고, `320a5360a0f3` 은 한 실행이 더 멀리 가서 기존 벽 `Unknown LGT WIPIC SVC id 240` 에 닿았다(재측 4/4 양쪽 같음 · `ledger-grep` 0건).

### 5. 같은 N 재측 — 정체 0

같은 키(문서 키 1행 + OK 2 · 75초) · 빌드당 20회 · main 과 교대 · 3개 동시 · 게이트 rc=0 후 시작(증적 `remeasure.tsv`):

| 빌드 | load1 | 정체 / 실행 |
|---|---|---|
| main `d1ba7687` | 16~21 | **5 / 20** |
| ⑴ 만 | 7~14 | 1 / 20(같은 회차 main 2 / 20) |
| ⑴ + ⑵ (이 PR) | 16~21 | **0 / 20** |

- 이 PR 빌드의 paints 는 1,226~1,296 이다. main 의 통과 실행(1,190~1,299)과 같은 범위다.
- 정체가 잘 나는 키(소개 직전 간격 0.3초)에서도 이 PR 직전 빌드들은 0/12 · 0/16 이었다(main 3/8).

### 6. 게이트

- `cargo fmt --check` · `cargo clippy --all -D warnings`(stable · beta) · `cargo clippy --target wasm32-unknown-unknown -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` **680 pass / 0 fail**.
- 새 시험 2(`wie-core-arm`):
  - `only_a_thread_sliced_out_mid_code_counts_as_preempted` — 예산으로 잘린 스레드는 «잘림», 끝난 스레드와 호스트 호출 안에서 기다리는 스레드는 «아님». `set_preempted(…, true)` 를 끄면 red(«out of budget between two guest instructions»).
  - `a_holder_sliced_out_keeps_the_others_off_guest_code` — 붙든 스레드가 잘려 있는 동안 다른 스레드는 게스트 코드를 돌지 않고, 놓으면 돈다. `wait_for_holder` 를 끄면 red.
- 러너 블록(release · 이 브랜치): draw · helloworld ×2 · text PASS. keydraw ×2 `--inject --expect-last-frame` 는 이 부하에서 main 도 이 브랜치도 `max-ticks` 로 UNMEASURED 였다. `--max-ticks 100000000000` 로 PASS · rc=0(paints 79 · 55 — 0421 과 같음).
- 기준선: 측정 바이너리는 `d1ba7687`(#461) 위다. 그 뒤 #462(LGT ABI 2행)가 먼저 착지해 그 위로 rebase 했다(충돌 0). 시험·clippy 는 rebase 한 트리에서 다시 돌렸다(680/0).
- 측정 규율: emulator 실행은 전부 `build-slot run` 으로 감쌌고 3개 이하 동시 · `nohup &` 없음. 짝·재측은 `host-load-guard --status --recovered` rc=0 을 기다린 뒤 시작했다. census 락은 쓰지 않았다(전수 아님 · wie 레인 6차 전수가 그 락을 쥐고 있었다).

### 7. 후속

| 군집 | 수 | 계급 | 크기 |
|---|---|---|---|
| 0418 의 60분 «후» 실행(누수 장면) 재측 — 이 수정으로 소개를 넘기므로 이제 잴 수 있다 | 1 | 측정 | M |
| `320a5360a0f3` 24단계 `Unknown LGT WIPIC SVC id 240`(한 번 멀리 간 실행에서만) | 1 | ⒝ | S |
| 같은 모양의 경합이 KTF 에도 있는지 — KTF `TaskRunner` 는 이 두 훅을 구현하지 않았다 | — | 측정 | M |

### 8. 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 8회 / 7쌍 · SUFFIX-ATTACHED 0회 / 0쌍. 8회 모두 이 회차가 고친 파일(`system.rs` · `emulator.rs` · `display.rs`)에 **이미 있던** 주석·시험 행이다. 이 회차가 더한 줄의 게임 이름은 0이다(`git diff origin/main...HEAD` 의 `+` 줄 대조 · 타이틀은 sha12 로만 적었다).

<!-- corpus-name-inflow v1 subjects=9 tree=f4c9e1ac41ed111b B=8/7 P=0/0 S=0/0 -->
