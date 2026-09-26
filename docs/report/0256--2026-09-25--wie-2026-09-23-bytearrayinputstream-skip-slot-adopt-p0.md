## [2026-09-25] 배틀몬스터 마을 이후 `address: 0` = 죽은 catch 프레임으로의 unwind — unwind 가 프레임을 소비하게 · StringBuffer 10/13/22 (wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0)

**무엇을**: `wie-lgt/src/runtime/java/exception.rs` 의 `unwind` 가 이제 들어가는 예외 프레임을 체인에서 떼고
해제한다. 같은 스레드(`a.run`)가 부딪히던 `java/lang/StringBuffer` 누락 칸 셋을 호출부 실측으로 채웠다 —
`length()I = 10` · `setLength(I)V = 13` · `append(C) = 22`.

**왜 — 원인 계급은 «런타임 결함»이다(ABI 한 칸이 아니다)**:

1. ⒜ 반증 먼저: `2e356dfc`(당시 main)에서 같은 벽이 **재현됐다** — `Invalid memory access; address: 0` ·
   같은 스레드 id `2125953043` · 같은 쓰레기 덤프(`SP 0x40100004 · PC 0x7100026a`). 단 티켓 명령 그대로(기본
   27키 · `--action-secs 4`)는 loadavg 110~200 에서 마을에 닿지 못하고 도움말 화면으로 빠졌다 — 경로는 임시
   키 스크립트(커밋 안 함)로 고정했다.
2. 덤프의 PC 는 폴트 지점이 아니다: 폴트 직전 trace 의 마지막 호스트 호출은 `0x7100000a` 이고
   `0x7100026a` 가 아니다. 그 덤프는 `Thread.run()`(Rust 스텁)이 스레드 스택 꼭대기에서 들어간 순간의 문맥이다.
3. ⒝ 마지막 정상 PC — 엔진에 임시 링 버퍼(커밋 안 함)를 달아 잡았다:
   `0x7acf0` → `.data` 트램펄린 `0x140455c` → import `(0x64, 0x12)` = `IsClassAssignable` → 참 →
   `0x7cb1c: sub sp, fp, #0x24 ; ldm sp, {r4-r8, sl, fp, sp, lr} ; bx lr` → **pc = 0**.
   ldm 이 읽은 `[0x400fff18, 0x400fff40)` 은 `1400df4 48845aa0 400fff40 2a 4b85b800 48846cc0 0 0 0` —
   폴트 레지스터(`r4=0x1400df4 r5=0x48845aa0 r6=0x400fff40 r7=0x2a fp=sp=lr=0`)와 **전건 일치**한다.
4. 그 구간은 왜 남의 데이터인가 — 예외 프레임 프로브(커밋 안 함):
   ```
   push   frame=0x4a85d180 prev=0x0 sp=0x400fff04 fp=0x400fff3c lr=0x7acb4     ← 함수 0x7ac84 의 try
   unwind frame=0x4a85d180 frame_sp=0x400fff04 cur_sp=0x400fff04 resume=0x7acb4 ← 같은 함수 안에서 throw · catch 일치 → return
   push   frame=0x4a85f580 prev=0x4a85d180 …                                    ← F 가 체인에 그대로 남았다
   …(93초, 같은 스택 구간을 pc 0x190c·0xd85e8·0x69ce8 … 가 덮어씀)…
   unwind frame=0x4a85d180 frame_sp=0x400fff04 cur_sp=0x400ffeac resume=0x7acb4 ← 무관한 throw 가 죽은 프레임으로
   ```
5. 왜 프레임이 남았나 — **게스트는 catch 에서 자기 프레임을 pop 하지 않는다**. 함수 `0x7ac84` 는 7.8KB 본문에
   `PushExceptionFrame` 1 · `PopExceptionFrame` **0**. catch 경로는 일치하면 에필로그로 바로 가고, 불일치면
   `Throw(r5)` 로 다시 던진다 — 프레임이 남아 있으면 그 재던짐은 **같은 프레임으로 되돌아와 무한 루프**다.
   `binary.mod` 전수: push 호출부 57 중 파싱된 catch 블록 **51/51 에 pop 이 없다**(6곳은 패턴 밖).
   ⇒ 배틀몬스터의 게스트 ABI 는 «unwind 가 들어가는 프레임을 소비한다» 이다. ★**배틀몬스터(ARM) 한정** — 게이트② 계측에서
   현영맞고2006·놈3(Thumb)의 catch 는 자기 프레임을 pop 했다(정정·두 관례 설계는 `0263`). 우리 `unwind` 는 pending 만 세우고 프레임을 남겼다.
   upstream(`4a7a8882` #1411)의 시험이 «unwind → 호출자가 pop» 을 전제했고, 이 회차가 그 전제를 뒤집는다.

**StringBuffer 행 — 같은 스레드의 앞선 벽(짝지은 실행에서 번갈아 먼저 난다)**. 셋 다 CLDC 1.1 선언 순서로
도출되고 실측 앵커 17·23·27 사이에 여유 없이 맞는다. 호출부(ELF `.text` 파일 0x34 = 가상 0x1000):

| 칸 | LR | 호출부 | 판정 |
|---|---|---|---|
| 22 | `0xff9c` | `ldrh r1, [r3, #4]`(r1 = `0xc624`) → `ldr ip, [r3, #0x5c]` · 결과 버림 | `sb.append(chars[i]);` |
| 10 | `0xffe8` | r0 만 세움(r1 은 낡은 `0x21`) → `cmp r0, #0 ; ble` → index 4(toString) | `if (sb.length() > 0)` |
| 13 | `0x1003c` | `mov r1, #0`(리터럴) → `ldr ip, [r3, #0x38]` · r0 안 읽음 | `sb.setLength(0)` |

**before/after**(같은 키 경로 · 같은 로그 설정 · 동시 실행, loadavg 110~190):

| 실행 | 바이너리 | 끝 | 마지막 장 |
|---|---|---|---|
| b1 | main `2e356dfc` | `Invalid memory access; address: 0` | 마을 · 8장 동일(125,459B) |
| b2 | main | 다른 경로 · 미처리 예외 0 | 움직임 |
| a1 | 수정본 | **`java.lang.ArithmeticException: / by zero`** (at `a.run()V`) | 마을 · 8장 동일(125,443B) |
| a2 | 수정본 | 미처리 예외 0 | 마을 · 캐릭터(121,626~8B) |
| f1 · f2 | 최종 트리(리베이스 뒤 · 13 행 제자리) | **미처리 예외 0 · 누락 칸 0 · paints 409 / 411** | 숲 장면 → 대화창 「[앨런] 오늘은 꼭 잡고 말겠어!」 |

(a1·a2 바이너리는 setLength 행을 String 클래스에 잘못 넣은 상태였다 — 고정 시험이 잡았고 f1·f2 가 고친 트리다.
b·a 의 paints 는 173~184 였다. 대화창은 보였지만 그것이 입력에 반응한 결과인지는 판정하지 않았다.)

⇒ `address: 0` 은 사라졌고, **같은 순간 원래 나던 예외가 정직한 이름으로 드러났다** — 죽은 프레임이 삼키던
그 throw 다(프로브 실행들에서 게임은 직전에 import `(0x64, 0x25)` = `RaiseArithmeticException` 을 처음 해석했다).
0 인 나눗수의 출처가 다음 벽이다(worklog 제안).
스크린샷·trace 원문: `~/orchestrator/reports/evidence/wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0/`.

**시험**: `rethrow_from_catch_reaches_the_enclosing_frame`(신설 — 안쪽 catch 의 재던짐이 바깥 프레임으로,
그다음은 None) · `exception_frame_restores_guest_context`(갱신 — catch 뒤 무관한 throw 는 None) ·
`missing_database_record_is_translated_before_guest_unwind`(갱신 — catch 뒤 pop 은 빈 체인) ·
ABI 고정 시험에 StringBuffer 3행. **변이**: `unwind` 의 두 줄(체인 떼기·해제)을 되돌리면 시험 **3건 red** · 원상 green.

**게이트**: fmt rc0 · clippy `--all -D warnings` rc0 · wasm clippy rc0 · `cargo +beta clippy` rc0 ·
`RUST_MIN_STACK=4194304 cargo test --all` **434 passed / 0 failed** · `clippy --workspace --all-targets` 경고 16
(전부 비접촉 파일 — `canvas.rs` 11 · `hardening.rs` 1 · `dod_ci_parity.rs` 1 ⇒ 증가 0).
러너 블록(디버그 · loadavg 183~196): draw_j2me·helloworld_ktf·helloworld_lgt PASS rc0 · keydraw_ktf/lgt FAIL
(paints 9·6) · text_j2me FAIL(paints 0). §The four gates ⑴~⑶ 대로 릴리스 바이너리를 main(`2e356dfc`)과 같은
루프에서 교대: keydraw_ktf·keydraw_lgt **main 6/6 · 수정본 6/6 PASS**(paints 50~55) · text_j2me main 7/7 ·
수정본 6/7(FAIL 1건은 paints 0 · loadavg 215 — 이 diff 는 wie-lgt 만 건드리고 text_j2me 는 J2ME 다) ⇒ 굶김.

**#287 과의 관계**: #287(열림 · 스레드별 예외 체인)도 `exception.rs` 의 같은 함수를 고친다. 결함이 다르다 —
#287 은 «다른 스레드의 프레임과 섞임», 이쪽은 «같은 스레드에서 소비되지 않은 프레임»(이 회차의 관측은 전부
스레드 2 한 곳). #287 의 `unwind` 도 프레임을 떼지 않으므로 **둘 다 필요**하고, 늦게 착지하는 쪽이 상대의
변경을 옮겨 실어야 한다(그 구조에서는 `state.ptr_current_exception_frame = frame[0]` + 해제).

**사용자 영향**: 배틀몬스터가 마을 이후 «메모리 오류»로 멈추던 자리가 실제 원인 이름(0으로 나누기)으로 바뀌었고,
그 이름이 붙지 않은 경로에서는 마을에 남아 계속 돈다. catch 에서 pop 하지 않는 LGT 게임(배틀몬스터)에서, 잡힌
예외 뒤에 나는 무관한 예외가 이미 끝난 함수로 튀지 않는다(catch 에서 pop 하는 관례는 `0263` 이 함께 다룬다).

**한계**: 이 벽은 타이밍 의존이다(같은 바이너리 짝 b1/b2 에서 1/2) — 스레드가 어느 경로로 먼저 가느냐에 따라
앞선 벽이 달라진다. 입력 반응은 아직 판정하지 못했다. `/ by zero` 는 a1 한 번 관측됐고 f1·f2 에서는 나오지
않았다 — 사라진 것이 아니라 그 경로를 타지 않았을 수 있다(다음 회차 제안).
