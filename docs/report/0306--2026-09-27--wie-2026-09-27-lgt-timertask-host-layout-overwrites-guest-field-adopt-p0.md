## [2026-09-27] LGT TimerTask — lastScheduledExecutionTime 을 인스턴스 밖으로 · 폰 배치 6단어 · 학교가는길 게임 루프 생존 (wie-2026-09-27-lgt-timertask-host-layout-overwrites-guest-field-adopt-p0)

**무엇을**: LGT 호스트 `java/util/TimerTask` 를 폰 배치(6단어)로 맞췄다. `rustjava-runtime 0.1.1` 이 더한 `lastScheduledExecutionTime J` 는 인스턴스 워드를 받지 않고 호스트 쪽 표에 둔다.
**왜**: 호스트 TimerTask 가 8단어라 Timer 스레드가 매 틱 게스트 하위 클래스의 자기 필드(word 6)를 덮고 인스턴스 밖 1단어(word 7)에 썼다(0296 ⑴-c). 학교가는길의 게임 루프 `c.run()` 이 첫 틱에 죽었다.
**사용자 영향**: 학교가는길의 타이머 작업이 죽지 않고 100ms 마다 돈다. 아직 화면은 검다 — 다음 벽은 `org.kwis.msp.lwc.Component::repaint` 가 스텁이라는 것(아래).

### 설계 선택 — 호스트 쪽 표(채택) vs 6단어 축소
| | 호스트 쪽 표 | 6단어 축소(필드 삭제 + J2SE 식 재계산) |
|---|---|---|
| `scheduledExecutionTime()` | rustjava 값 그대로(마지막 실행의 예정 시각) | `period<0 ? next+period : next−period` — ★고정지연(`schedule(TJJ)`, 학교가는길이 쓰는 형태)에서 «재예약 시각»이 돼 늦은 만큼 어긋난다 |
| upstream 무접촉 | ○ | ✗ — Timer 스레드가 `put_field(lastScheduledExecutionTime)` 를 하므로 필드를 지우면 `NoSuchFieldError`. 메서드 교체도 hardening(KTF·J2ME 공용)에 들어가야 한다 |
| 범위 | LGT 만(배치 문제가 LGT ABI 의 것) | 공용 |

⇒ 호스트 쪽 표. **rustjava-runtime 수정 없음.**

- 선언: `data/lgt_java_abi.toml` `[[class]] java/util/TimerTask` 의 `host_field` 행(ABI 파일이 «폰에 칸이 없다»를 말하는 자리). 원시 타입만(수집기가 이 값을 보지 않는다).
- `JavaClassDefinition::new` 가 그 필드를 워드 배치·게스트 필드 표에서 뺀다 ⇒ TimerTask 인스턴스 워드 **8 → 6**.
- `ClassDefinition::field()` 가 게스트 표에서 못 찾으면 그 행을 보고 `JavaHostField` 를 돌려준다(미스 경로에서 이름 선검사 후에만 클래스 이름을 읽는다).
- 값은 `(ArmCore::id(), 인스턴스 포인터)` 키 전역 표. 인스턴스 생성·파괴 때 지운다(주소 재사용 시 이웃 값 유입 방지). `ArmCore::id()` 는 이 회차가 더한 3줄 — 한 프로세스의 두 에뮬레이터가 같은 게스트 주소를 쓰기 때문.

### 시험(되돌리면 red)
`timer_task_keeps_the_phone_layout_and_scheduled_execution_time`: TimerTask 6워드 · 하위 클래스 7워드 · `put_field(lastScheduledExecutionTime)` 전후 인스턴스+1워드(0..=7) 불변 · `scheduledExecutionTime()` = 넣은 값(1790431442004) · 새 인스턴스 0 · 실제 `Timer.schedule` → Timer 스레드가 `run()` 을 부르고 거기서 본 자기 필드 0 · 예정 시각 > 0.
`host_field` 행을 지우면 `left: 8 right: 6` 으로 red(실측).

### 학교가는길 실측(release `wie_validate --timeout 20`, load1 266~298)
| | origin/main `156c5c56` | 이 브랜치 |
|---|---|---|
| 판정 | FAIL · deadline · paints 1 · only blank — 3/3 | FAIL · deadline · paints 1 · only blank — 3/3 |
| ticks | 20.4M~25.9M(스레드가 죽어 실행기가 빈 채 돈다) | 988~1,232 |
| Timer 스레드 | `Uncaught exception … Invalid memory access; address: 0` | 예외 0 · `Component::repaint` **196회/20s**(≈100ms 주기) |

- 죽는 주소가 0296 의 `0xde07b454` 가 아니라 `0` 인 것은 그 사이 main 이 움직인 결과로 보인다(가르지 않았다 — 같은 덮어쓰기 칸).
- ★다음 벽: `org.kwis.msp.lwc.Component::repaint()` 가 스텁(`wie-wipi-java` `component.rs`)이라 틱마다 불려도 아무것도 그리지 않는다. 제안으로 남긴다.

### 회귀
- 코퍼스에서 `java/util/TimerTask` 를 참조하는 LGT 타이틀(중첩 jar 까지 풀어 문자열 검색): **100 중 1**(학교가는길). KTF·J2ME 코드 경로는 무접촉(공유 변경은 `ArmCore::id()` 추가뿐).
- LGT 100파일 스윕(`--timeout 10`, 병렬 6, main ↔ 이 브랜치): 양쪽 PASS 77 / FAIL 21 / JSON 없음 2 — 같음. 개별 뒤집힘 4건(제노니아2·테라 영원의 혼돈 ↑, 나는마왕이다2·아니마 ↓)은 전부 `no frame rendered` 이고 `--timeout 20` 순차 재실행 3/3 양쪽 PASS ⇒ 부하 굶주림(§The four gates ⑴).
- 네 게이트 + beta: fmt OK · clippy/wasm clippy/beta clippy rc0 · `RUST_MIN_STACK=4194304 cargo test --all` 48 스위트 **496 passed 0 failed**.

### 한계
- 러너 블록(`wie_validate` 고정 픽스처)은 엔진 LGT 경로 변경이라 CI 밖에서만 본다 — 스윕이 그 몫을 했다.
- `host_field` 에 참조 타입을 넣으면 수집기 루트가 되지 않는다(선언 주석에 적었다 · 검사기 없음).

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus <game_lab>`): BOUNDED 35 · SUFFIX-ATTACHED 6(아래 표식). 대상이 `lgt_java_abi.toml`·`jvm_support.rs` «파일 전체»라 기존 주석의 타이틀이 함께 세어진다. 이 회차가 새로 쓴 이름은 `학교가는길`(대상)과 회귀 스윕의 뒤집힘 4건(`제노니아2`·`테라 영원의 혼돈`·`나는마왕이다2`·`아니마`, 이 문서만)이다.

<!-- corpus-name-inflow v1 subjects=9 tree=53c2fb931aa0bcce B=91/35 P=9/2 S=16/6 -->
