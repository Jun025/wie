## [2026-10-09] 양보하지 않는 MIDlet 을 1 000 호출마다 끊는다 — 바이트코드 JVM 협력적 양보 (wie-jvm-cooperative-yield-budget-for-non-yielding-midlet)

### 무엇을 · 왜
- 0485 §4: loveme(MIT-0 · LuaJ 로 Lua 를 해석)가 `startApp` 에서 한 번도 양보하지 않는다. `wie_validate --timeout 12` 가 돌아오지 않았다. 브라우저도 같은 틱 구조라 같은 시간 동안 멈춘다.
- 원인: crates.io `jvm-bytecode` 해석기는 스스로 `Pending` 을 내지 않는다. 게스트가 `sleep`·`yield`·`wait` 없이 계산하면 한 번의 poll 이 계산 전체를 쥔다. ARM 코어는 `INSTRUCTIONS_PER_YIELD`(10 000)마다 양보하는데, 바이트코드 JVM 에는 그 장치가 없었다.
- 고친 것: `wie-jvm-support` 가 게스트 바이트코드 클래스(`Runtime::define_class` 경로)의 메서드를 `Preemptible` 로 감싼다. 게스트 메서드 진입 `GUEST_CALLS_PER_YIELD`(1 000)회마다 `YieldFuture` 를 한 번 낸다. 카운터는 런타임 하나당 하나다.
  - 왜 메서드 진입인가: 해석기 루프는 crates.io 크레이트 안에 있다. wie 가 코드를 건네는 곳은 클래스 정의(`InheritedMethods`) 하나뿐이다. 호출 없는 순수 루프는 못 끊는다. LuaJ 를 포함해 지금까지 본 게스트는 모두 호출한다.
  - 왜 게스트 클래스만인가: 게스트 메서드는 원래도 `sleep`·`wait`·모니터 경합으로 멈출 수 있다. 그래서 그 호출자는 이미 «한 poll 에 안 끝남»을 견딘다. Rust 런타임 클래스(`define_class_rust`)는 감싸지 않는다 — 새 종류의 정지점을 만들지 않는다.
  - KTF·LGT 는 게스트 코드가 ARM 이라 이 경로를 지나지 않는다. 아래 smoke gate 로 확인했다.
- 운영자 채택 제안: `2026-10-09-license-clean-midp-corpus#p1`.

### 측정 — loveme
release `wie_validate`. base = `origin/main` `82959a5b`. 후보는 같은 트리에서 상수만 바꿨다.

| 빌드 | `--timeout 12` | 소요 | 결과 | ticks | paints | ms/tick |
|---|---|---|---|---|---|---|
| base | 돌아오지 않음 — 90 s 에 바깥 감시가 `kill -9`(rc 137) | 90 s+ | 출력 없음 | – | – | – |
| N = 10 000 | rc 0 | 13 s | PASS · stop `deadline` | 96 | 60 | 126.1 |
| N = 2 500 | rc 0 | 13 s | PASS | 368 | 58 | 32.7 |
| **N = 1 000** | **rc 0** | **13 s** | **PASS** · 예외 0 | 523 | 58 | **23.0** |

- 세 후보는 동시에 돌렸다(load1 ≈ 27). paints 는 N 에 상관없이 같다. 게스트의 처리량은 그대로이고, 한 틱이 쥐는 시간만 줄어든다.
- 1 000 을 고른 이유: 예산 14 ms 에 가장 가깝다. 양보 1회 비용은 executor step 1회다. 해석된 호출 1 000 회(이 부하에서 약 9 ms)에 비하면 작다.

### 측정 — 브라우저 응답
- 방법: `scripts/audio-probe.mjs`(헤드리스 Chromium · 셸과 같은 경로)에 두 가지를 더했다. ① `emu.tick()` 한 번의 벽시계 시간 — max · p99 · 50 ms 초과 수. 페이지는 두 tick 사이에서만 그리고 입력을 받는다. ② `putImageData` 호출 수(= 화면에 낸 게스트 프레임). 끝나지 않는 tick 은 `--secs`+60 s 뒤 «a tick never returned» 로 보고한다.
- 빌드 2종: `wasm-y1k`(이 PR), `wasm-noyield`(같은 트리에서 `YieldFuture` 한 줄만 뺀 것 = 동작상 main). `--secs 20~30` · 키 700 ms 순환 · 두 빌드 동시 실행 · load1 14~52.

| 게임 | 빌드 | tick max | p99 | > 50 ms | paints |
|---|---|---|---|---|---|
| loveme | noyield | **80 s 넘게 돌아오지 않음** | – | – | – |
| loveme | y1k | 138 ms | 26 ms | 3 | – |
| mobapp-game | noyield | 188 ms | 56 ms | 30 | 1054 |
| mobapp-game | y1k | 156 ms | **24 ms** | 3 | **1309** |
| stickfight | noyield | 643 / 646 ms | 64 / 126 ms | 14 / 14 | 1173 / 1105 |
| stickfight | y1k | **148 / 125 ms** | 28 / 28 ms | 1 / 1 | **1326 / 1264** |
| cave-escape | noyield → y1k | 107 → 111 ms | 20 → 19 ms | 1 → 1 | 1316 → 1296 |
| bubble-shot | noyield → y1k | 103 → 91 ms | 21 → 20 ms | 2 → 2 | 1325 → 1292 |
| sperm-race | noyield → y1k | 104 → 104 ms | 17 → 17 ms | 1 → 1 | – |

- 남은 tick max 약 100~150 ms 는 부팅 쪽 한 번이다. 양쪽 빌드에 다 있다(cave/bubble/sperm-race). 이 PR 의 범위가 아니다.
- mobapp-game 2회차의 noyield 쪽은 30 s 안에 «guest exited» 로 끝나 비교에서 뺐다.

### 측정 — 퇴행
**① 통신사 코퍼스 smoke gate**(`scripts/smoke_gate.sh` · boot+render · BIN = 이 PR 의 release `wie_validate`, 시간 때문에 release 를 썼다):

| 묶음 | 결과 |
|---|---|
| LGT + SKT (`PLATFORM_FILTER="lgt skt"`) | ran 104: **104 PASS / 0 FAIL** · baseline 102 확인 · 결측 0 · **퇴행 0** |
| KTF 앞 95 | ran 95: **95 PASS / 0 FAIL** · baseline 95 확인 · 결측 0 · **퇴행 0** |
| KTF 뒤 95 | ran 95: **95 PASS / 0 FAIL** · baseline 95 확인 · 결측 0 · **퇴행 0** |

KTF 는 190개라 한 임대(30분 목표)에 안 들어간다. 그래서 둘로 나눴다. 각 절반은 심링크 디렉터리와 그 절반만 담은 baseline(`BASELINE=`)으로 돌렸다. 「결측 > 절반 = UNMEASURED」 규칙에 걸리지 않게 하려는 것이다.

**② 속도·boot 표본**(무입력 `--timeout 12` · base / N=1 000 / N=10 000 을 같은 순간에 · load1 10.5~19.9 · 86개 = SKT 50 + OSS 16 + agneay 20):

| 묶음 | PASS base → 1 000 → 10 000 | paints 비(후보/base, base ≥ 5) 중앙 · 범위 | 예외 수 달라진 게임 |
|---|---|---|---|
| SKT 50 | 50 → 50 → 50 | 1 000: 1.000 · 0.98~1.03 / 10 000: 1.000 · 0.99~1.03 | 0 |
| OSS 16 | 13 → 13 → 13 | 1.000 · 1.00~1.00 | 0 |
| agneay 20 | 20 → 20 → 20 | 1 000: 0.994 · 0.98~1.00 | 0 |

OSS 의 base FAIL 3개는 0485 §4 의 알려진 벽이다(bubblet-asha · finifactory · kurve).

**③ 입력·진도 표본**(`--inject` · base / 1 000 동시 · 36개 = OSS 16 + agneay 20):
- 판정 일치 **36/36**(PASS 10 · UNMEASURED 23 · FAIL 3 — 셋 다 같다).
- 닿은 입력 단계 합 641 → 640. ±1 차이는 3개다(box-claim · brick-buster · checkpoint-rally). 벽시계 키 타이밍의 흔들림이다.
- paints 비 중앙 0.995(n = 30).
- ★예외 하나: **mobapp-game 0.80**. 같은 짝을 더 돌렸다(native · 동시 짝).

| | base | N=1 000 | N=10 000 |
|---|---|---|---|
| 실행 수 | 11 | 11 | 4 |
| paints 범위 · 중앙 | 551~907 · 798 | 455~817 · 613 | 661~870 · 825 |

- 판정은 전부 PASS · 27/27 키 · 예외 3(한 번 4)이다.
- 이 게임은 표본에서 유일하게 **매 틱 예산을 넘는다**(base 30 s 에 ticks 622~907 · 다른 게임은 수백만 틱 = 대기). N=1 000 은 그 프레임 계산을 틱 경계에서 자른다.
- `wie_validate` 는 틱 사이에 쉬지 않는다. 그래서 base 에서는 넘친 틱이 그 CPU 를 그대로 화면 갱신에 썼다. 자르면 그 몫이 줄어든다.
- **브라우저(rAF 마다 tick)에서는 반대다** — 1054 → 1309 paints · p99 56 → 24 ms. 넘친 tick 이 다음 rAF 를 먹던 것이 사라졌다.
- ⇒ 이 −20% 는 native 검증기 루프에서만 보이는 값이다. 사용자가 보는 호스트에서는 개선이다. 다만 census 속도 축은 `wie_validate` 로 재므로, «매 틱 예산을 넘는 J2ME/SKT 게임»은 그 축이 내려갈 수 있다. 이 표본(SKT 50 포함)에서 그런 게임은 mobapp-game 하나였다.

**④ 저장소 게이트**
- `cargo fmt --check` · `clippy --all -D warnings` · wasm clippy · `+beta clippy`: 모두 rc 0.
- `RUST_MIN_STACK=4194304 cargo test --all`: rc 0 · 실패 0. `wie-j2me/tests/test_pacing.rs` 도 통과했다. 그 픽스처의 게임 루프는 1 000 호출에 닿지 않는다.
- 코퍼스 이름 유입(`node scripts/corpus-name-inflow.mjs` · 이 브랜치의 바뀐 파일 전체): BOUNDED 0 · SUFFIX-ATTACHED 0.
- runner block: draw_j2me · helloworld_ktf/lgt PASS · keydraw_ktf 79 paints 27/27 · keydraw_lgt 55 paints 27/27(둘 다 rc 0) · text_j2me PASS.

### «되돌리면 red»
`a_guest_that_never_yields_is_preempted_between_method_calls`(`wie-jvm-support/src/jvm_implementation.rs`):
- 손으로 짠 클래스 `Spin` 을 `ClassLoader.defineClass` 로 정의한다. `run(n)` 은 `nop()` 을 n 번 부를 뿐 양보하지 않는다.
- `3 × GUEST_CALLS_PER_YIELD` 만큼 돌리는 동안, 다른 태스크가 2번 이상 poll 됐는지 본다.
- `YieldFuture` 줄을 빼면 «the other task ran 0 times during the loop» 로 실패한다(실행 확인).
- 시계를 읽지 않아 부하와 무관하다.

### 사용자 영향
- 무거운 일반 자바 게임을 열어도 탭이 얼지 않는다. loveme 는 이제 화면을 낸다(검증기 PASS · 58 paints).
- 매 프레임 예산을 넘는 게임은 브라우저에서 더 많은 프레임을 낸다.

### 하지 않은 것
- 호출 없는 순수 루프(산술만 도는 `for`)는 여전히 한 poll 을 쥔다. 해석기 루프 안에 정지점을 두려면 crates.io 크레이트를 갈라야 한다. 그런 게스트는 아직 관측되지 않았다.
- 시계 기반 예산(«틱 끝을 넘으면 양보»)은 쓰지 않았다. 그러려면 호출 경로에서 시계를 읽어야 한다. 그런데 `test_pacing.rs` 는 시계 읽기 1회 = 1 ms 로 결정성을 잡는다. 호출 수 기반은 결정적이고, ARM 의 `INSTRUCTIONS_PER_YIELD` 와 같은 모양이다.
