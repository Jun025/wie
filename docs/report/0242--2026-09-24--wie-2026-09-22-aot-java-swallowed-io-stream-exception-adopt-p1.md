## [2026-09-24] 검증기가 «런타임이 던진 Java 예외»의 계수·첫 문면을 JSON 에 싣는다 — 보고만, 게이트 0 (wie-2026-09-22-aot-java-swallowed-io-stream-exception-adopt-p1)

**무엇을** — `wie_validate` JSON 에 `java_exceptions: {count, first, first_truncated}` 를 더했다.
`first` 는 `"<클래스>: <문면>"`(escape · 256바이트 상한 · 잘리면 `first_truncated:true`).

**왜** — 2026-09-22 회차의 LGT-AOT 4타이틀은 모두 `no frame rendered` 로 보였지만, 실제로는
런타임이 던진 예외 1건을 게스트가 삼킨 것이었다. 그것을 찾으려고 `RUST_LOG` 를 켜고
`jvm::jvm: throwing java exception` 줄을 손으로 셌다(제안 `2026-09-22-aot-java-swallowed-io-stream-exception#p1`).

**반증 ⒜ — upstream 수정 없이 관측할 수 있는가: 된다.** `jvm 0.1.1`(crates.io)이 던지는 예외는
전부 `Jvm::exception` 을 거치고, 그 첫 문장이 `tracing::info!("throwing java exception: {} {message}")`
(target `jvm::jvm`)다. wie 쪽 `tracing` 레이어가 그 이벤트를 센다. stderr 로거 필터는 전역에서
**레이어 단위**로 바꿨다. 그래서 `RUST_LOG` 가 없어도 수는 세지고 stderr 는 이전처럼 0바이트다(실측).

**보이지 않는 것** — 게스트가 바이트코드로 직접 만든 예외(`new`+`athrow`)는 `Jvm::exception` 을 거치지
않으므로 세지 않는다. 이름을 `swallowed_exceptions` 로 두지 않은 이유는 이 줄이 «던질 때» 찍혀서
잡혔는지 모르기 때문이다.

**실측**(`--timeout 30` · load ≈ 28)

| 입력 | result | java_exceptions | 손 계수 |
|---|---|---|---|
| 훼밀리마트타이쿤(lgt) | FAIL no frame rendered | count 1 · `net/wie/WieError: Unimplemented: java/lang/StringBuffer vtable index 22…`(잘림) | 1 |
| 체스마스터(lgt · 대조군) | PASS | count 0 · null | 0 |
| 배틀몬스터(lgt) | PASS p3 | count 0 | 0 — 형제 회차 이후 더는 던지지 않는다 |
| 서든어택포켓(lgt) | FAIL no frame rendered | count 0 | 0 — 지금 막는 것은 런타임 예외가 아니다 |
| `draw_j2me.jar` · `text_j2me.jar` | PASS | count 2 · `java/io/FileNotFoundException: Resource not found: /wie-absent.png` | — |

마지막 줄은 계수로 판정하면 안 되는 이유를 보여 준다. 픽스처가 createImage 실패 갈래를 일부러
밟고 잡는데, 결과는 PASS 다.

**판정 불변** — runner block 7줄(before `13437469` ↔ after) 모두 result·rc 동일. `passed` 는 이 필드를 읽지 않는다.

**변이** — 레이어 `on_event` 기록 절단 → 새 테스트 red · 필터 INFO→WARN → red · 원상 green.
`main` 배선은 source-shape 테스트가 잠근다.

**사용자 영향** — 없음(검증기 출력에 키 1개 추가). 기존 파서는 `grep -o` 키 단위라 영향 없다.
`first` 는 escape 되므로 `"result":"` 리터럴이 그 안에 나타날 수 없다(테스트로 확인).
