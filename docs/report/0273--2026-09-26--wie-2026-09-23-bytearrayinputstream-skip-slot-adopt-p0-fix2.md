## [2026-09-26] LGT unwind — 소비 기억을 push/pop 짝으로 저장·복원 (wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0-fix2)

**무엇을**: `wie-lgt/src/runtime/java/exception.rs` — `0263` 의 스칼라 `consumed_frame_sp` 를 **스택**으로 바꿨다
(`consumed_sp[16]` · `consumed_len` · `consumed_base`). `unwind` 가 소비한 프레임의 sp 를 스택에 올리고, `push` 는
그 위로 **기준(base)을 올려 가리기만** 하며(종전: 지움) 가려진 기준을 새 프레임의 19번째 워드에 저장한다. 그 프레임의
정상 `pop`(또는 `unwind`)이 기준을 되돌린다. catch 의 자기 pop 은 «보이는 맨 위 항목의 sp == 현재 sp» 일 때 no-op.

**왜**: 게이트② 2차 반려(critical) — `push` 가 무조건 기억을 지워, Thumb catch 가 자기 pop «전에» try 를 가진 코드를
거치면(메서드 호출 · catch 안의 같은 sp try · 재귀) 자기 pop 이 **바깥 try 프레임을 해제**했다. 그 모양에서는 main 이 옳고
`0263` 이 회귀였다.

**판별자 근거**: 항목이 «아직 catch 안»인지는 push/pop 짝이 스택 규율로 안다 — catch 안에서 시작된 push 는 그 짝 pop 에서
끝나므로, 그 사이에 생긴 항목만 버리고 catch 의 항목은 되살린다. sp 는 «어느 pop 이 catch 의 자기 pop 인가»에만 쓴다(`0263` 과 같다).
남는 한계(`ponytail:` 주석 갱신): ⑴같은 sp 에 try 를 중첩한 ARM 함수에서 안쪽 catch 가 바깥 try 로 돌아오면 바깥 pop 이 no-op
(`0263` 과 같은 한계 · main 도 그 프레임을 남겼다) ⑵pop 하지 않는 catch(ARM)의 항목은 바깥 프레임의 pop/unwind 가 치우고,
바깥 프레임이 없으면 남는다 — 상한 16 을 넘는 새 항목은 기록하지 않으며 그때 그 catch 의 pop 은 main 처럼 바깥 프레임을 해제한다.

**시험**(검수자 반례 3건 · 기존 5건 무수정): `cx_thumb_catch_calls_function_with_try` · `cx_thumb_catch_with_nested_try_same_sp`
(안쪽 try 가 던지는 경우 포함 — 같은 sp 에 대기 중인 catch 2개) · `cx_thumb_catch_recurses`. `cargo test -p wie-lgt --lib exception` **8/8**.
**변이**: M4 push 가 보이는 항목을 지움(=`0263`) → **3 red**(반례 3건) · M5 정상 pop 이 기준을 되돌리지 않음 → **3 red** · 원상 8/8.
(pop 쪽 «반환된 항목 버리기» 헬퍼는 변이 M6 에서 어떤 시험도 붉히지 못해 **삭제**했다 — 정확성에 불요.)

**게이트**(loadavg 108~218): fmt · clippy `--all -D warnings` · wasm clippy · `+beta` clippy 전부 rc0 ·
`RUST_MIN_STACK=4194304 cargo test --all` **450 passed / 0 failed**(447 + 반례 3).

**실게임 재측**(release · 계측은 커밋하지 않은 임시 패치 · 원문 `~/orchestrator/reports/evidence/wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0-fix2/`):

| 타이틀 | 결과 |
|---|---|
| 현영맞고2006 | UNWIND 3 (resume 0x7ccb·0x82b3·0x8a07) → **POP-NOOP 3/3**(lr 0x7cfb·0x82e5·0x8a39) · 다음 PUSH prev = 바깥 프레임 · PASS rc0 |
| 놈3 | UNWIND 1 (0x1c937) → **POP-NOOP 1/1**(lr 0x1c96f) · PASS rc0 |
| 배틀몬스터(티켓 ⒝ 명령 · 유효 4회) | `address: 0` **0/4** · 0x7acb4 unwind 1/4(run6) → 다음 PUSH **prev=0x0** · PASS rc0 전건 |

배틀몬스터 run6 은 새 설계의 ARM 쪽을 실게임에서 밟았다: pop 하지 않는 catch 뒤 **같은 sp(0x400fff04)** 에 새 try 가 push 되고
(`consumed_len=1`) 그 pop 은 기준 1 로 항목이 가려져 **정상 pop**(no-op 아님)이다. unwind 0/3 실행은 그 경로에 닿지 않은 것이다(`0263`·검수자와 같은 관측).
★첫 3회(`fix2_trace.txt`)는 결과 줄(stdout)만 나오고 stderr 줄(계측·`address: 0` 모두)이 **0줄**이었다 — 원인 미규명. 같은 바이너리·같은 명령을 `tee` 로 다시 돌린 run7 은 1016줄이다. stderr 가 비어 있으면 `address: 0` 도 보일 수 없으므로 그 3회는 **수에서 뺐다**.
★run6 의 unwind 뒤 스레드에서 `ArithmeticException: / by zero` 미처리 — 형제 회차 `wie-battlemonster-field-input-and-div-by-zero-after-unwind-fix` 의 대상이며 이 회차 범위 밖.

**사용자 영향**: 현영맞고2006·놈3 류(Thumb) 게임에서 catch 가 메서드를 부르거나 try 를 중첩해도 바깥 try 가 살아 있어, 다음 예외가
미처리로 새거나 `FatalError` 로 죽지 않는다. 배틀몬스터의 `address: 0` 수정은 그대로 유지.

**게임 파일명 유입**(`corpus-name-inflow --corpus` · 브랜치 전체 8파일): BOUNDED 37 · SUFFIX-ATTACHED 8(main 재병합 뒤 · `jvm_support.rs` 주석 합집합 +2 · 아래 병합 절 자신의 언급 +1) — 전부 주석·문서 속 타이틀명(계측 근거 인용) · 게임 바이트 0. `0263` 의 표식은 이 회차의 코드 변경으로 낡아 떼고 이 파일로 옮겼다. (이 회차의 표식도 `0281` 이 브랜치에 파일을 더해 낡아 `0281` 로 옮겼다.)

**병합**: `origin/main 5434ab0a` 를 merge commit 으로 흡수(리베이스·force 0). 충돌 1곳 `jvm_support.rs` ABI 고정 시험 주석 — 합집합(StringBuffer 10/13/22 + 월드장기체스 String 16). 연번 `0272` 는 #307 이 먼저 착지해 `0273` 으로 옮겼다. 병합 뒤 게이트: fmt · clippy · wasm clippy · `+beta` clippy rc0 · `cargo test --all` **460 passed / 0 failed**.

