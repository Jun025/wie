## [2026-10-08] KTF 재배치 이미지 — `pushed_java_sp` 범위 가드 시험 · 스레드 스택 하한 (wie-ktf-relocated-pushed-java-sp-guard-test-and-lower-bound)

**무엇을**: PR #509 게이트② 권고 F1·F2. ⑴`[native top - 8]` 의 낱말이 이 진입의 Java `sp` 가 아니면 무시하는지 지키는 시험을 넣었다. ⑵그 낱말이 받아들여져도 새 프레임(`pushed - NATIVE_STACK_GAP`)이 현재 스레드 스택 바닥 아래로 가지 않게 하한 가드를 넣고 시험했다(`wie-ktf/src/runtime/relocated.rs` `pushed_java_sp` · `wie-core-arm` `ArmCore::current_stack_base`).

**왜**: F1 — 가드를 `Ok(Some(pushed))` 로 바꾼 개악이 전 스위트 green 이었다. F2 — 던지기 헬퍼 4곳은 `[top-8]` 에 아무것도 넣지 않으므로 이전 리졸브 호출의 낡은 Java `sp` 가 남는다. 종전 가드는 `java_top - pushed < 1MB` 만 보고 `- NATIVE_STACK_GAP` 여유를 보지 않았다.

**사용자 영향**: 없음(관측된 적 없는 경로를 막는다). 재배치 이미지 3종 결과 불변(아래).

### F2 — 가드를 택한 이유
«도달 불가» 를 역어셈으로 입증하려면 낡은 낱말을 남긴 리졸브 호출이 반드시 같은 깊이에서 호스트에 들어갔다는 것까지 보여야 한다. 그보다 하한 한 줄이 싸다. 하한은 그 스레드 스택의 실제 바닥(`ThreadState::stack_base`)이다 — 1MB 창을 `java_top` 에서 재는 근사는 스택 맨 위가 `native_stack` 보다 위일 때 바닥을 모른다. `run_in_thread` 밖(부팅·시험 대부분)에서는 바닥을 모르므로 종전 가드만 남는다.

### 시험 · 개악 대조
| | 결과 |
|---|---|
| `a_nested_entry_ignores_a_word_that_is_not_a_java_sp` — 코드 주소 · `java_top + 4` · `java_top - 1MB` | 종전 위치(`sp - NATIVE_STACK_GAP`) |
| `a_nested_entry_stays_inside_the_thread_stack` — 실제 스레드에서 `base + 0x100` 은 거절 · `base + GAP` 은 수락(새 sp = `base`) | 통과 |
| 개악 M2(가드 → `Ok(Some(pushed))`) | **두 시험 red** |
| 개악 M3(하한만 제거) | **`…stays_inside_the_thread_stack` red** |

### 퇴행 — release `wie_validate --inject --max-ticks 100000000000`(base `71b1cc6e` ↔ head 동시 · long 풀 임대 · load1 13~25)
| sha | 판정 | paints | colors | frozen_tail | java 예외 | 마지막(27키) 화면 |
|---|---|---|---|---|---|---|
| `83fc429f9cbe` | PASS/PASS | 80/80 | 203/203 | 0/0 | 0/0 | 바이트 동일(지도 화면) |
| `1d5831e42a8a` | PASS/PASS | 167/167 | 160/160 | 0/0 | 0/0 | 다름 — base 끼리도 다르다(아래) |
| `b907b0faf483` | PASS/PASS | 196/196 | 55/55 | 0/0 | 3/3 | 바이트 동일 |

`1d5831e42a8a` 의 키별 화면은 base↔head 28장 중 20장이 다른데, base 를 두 번 더 돌리면 base↔base 도 20장·4장이 다르다(마지막 화면 색 수 63·78·63, head 74). 시점 차이다.

compat 변경 0(엔진 결과 불변).

유입: BOUNDED 0 · SUFFIX-ATTACHED 0.
