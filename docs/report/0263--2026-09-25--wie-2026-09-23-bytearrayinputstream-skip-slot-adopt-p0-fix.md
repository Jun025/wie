## [2026-09-25] LGT unwind — catch 에서 pop 하는 관례(Thumb)와 안 하는 관례(ARM)를 둘 다 (wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0-fix)

**무엇을**: `wie-lgt/src/runtime/java/exception.rs` — `unwind` 는 여전히 들어가는 프레임을 체인에서 떼고 해제하되,
그 프레임의 sp 를 스레드 상태 `consumed_frame_sp` 에 남긴다. push 없이 **같은 sp** 에서 오는 다음 `pop` 은
catch 가 자기 프레임을 치우는 것이므로 no-op(pending 만 지움). 다른 sp 의 pop 이나 push 는 그 기억을 지운다.
#287(스레드별 예외 상태 · 착지)을 병합하고 그 구조 위에 얹었다.

**왜**: 게이트②(`0256` 반려)가 release 계측으로 반례를 냈다 — 현영맞고2006·놈3(Thumb · LR 홀수)의 catch 는 자기
프레임을 **pop 한다**. `0256` 의 무조건 소비 아래서 그 pop 은 바깥 try 의 프레임을 해제했다. `0256` 의 «51/51 pop 없음»은
**배틀몬스터(ARM) 한정**이었다(`0256` 본문 정정).

**판별자를 sp 로 고른 근거**(계측 원문 `~/orchestrator/reports/evidence/wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0-fix/`):
- 세 타이틀 모두 pop 시점의 sp 가 그 프레임을 push 할 때의 sp 와 같다(배틀몬스터 `frame_sp == cur_sp` 전건 · Thumb 의 catch pop 도 unwind 가 복원한 sp).
- catch 뒤에 오는 «남의» pop 은 반드시 더 바깥 활성 레코드(더 큰 sp)거나 사이에 push 가 끼어 기억이 지워진다.
- ★남는 한계(주석 `ponytail:`): 한 ARM 함수가 **같은 sp 에 try 를 중첩**하고 안쪽 catch 가 바깥 try 로 돌아오면, 바깥 pop 이
  no-op 이 된다 — main 도 그 모양에서는 같은 프레임을 남겼으므로 퇴보는 아니다. 관측 0.

**계측(release · 스크래치 트리 · repo 무접촉)** — 수정본, 검수자와 같은 명령:

| 타이틀 | UNWIND | 직후 | 다음 PUSH 의 prev |
|---|---|---|---|
| 현영맞고2006 | 3 (resume 0x7ccb·0x82b3·0x8a07) | **POP-NOOP** 3/3 (lr 0x7cfb·0x82e5·0x8a39) | 바깥 프레임 그대로(0x4a85ca00 등) |
| 놈3 | 1 | **POP-NOOP** 1/1 (lr 0x1c96f) | 바깥 프레임 그대로 |
| 배틀몬스터(티켓 ⒝ 명령) | 1 (resume 0x7acb4) | pop 없음(ARM) | **0x0** — 죽은 프레임 없음 |

no-op 이 «다른 sp» 로 기억을 지운 경우(POP-SETTLE) 0회. 배틀몬스터의 0x7acb4 는 `0256` 의 main trace 에서 93초 뒤
죽은 프레임이 된 바로 그 자리다(main 은 다음 PUSH 의 prev 가 그 프레임).

**시험**: 신설 `catch_that_pops_leaves_the_enclosing_frame_on_the_chain`(Thumb 시퀀스) ·
`catch_that_returns_without_pop_leaves_no_dead_frame`(ARM 시퀀스 + 바깥 함수의 제 pop) · `rethrow_from_catch_reaches_the_enclosing_frame`
(#287 구조로 이식). `missing_database_record_is_translated_before_guest_unwind` 의 pop 은 wrapper 반환 **뒤**(호출자 sp)라 catch 의 pop 이
아니다 ⇒ 빈 체인 Err 유지. **변이**(`exception` 시험 5건):
M1 no-op 제거(=`0256` 원판) → 3 red · M2 sp 판별 제거(무조건 no-op) → 1 red · M3 소비 제거(=main 원판) → 2 red · 원상 5/5.

**게이트**: fmt · clippy `--all -D warnings` · wasm clippy · `+beta` clippy 전부 rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **447 passed / 0 failed**(main 재병합 뒤 · 병합 전 445/0).

**티켓 ⒝ 독립 재현 — 키 스크립트 없이, 티켓 명령 그대로**(`--inject --boot-secs 6 --action-secs 2 --max-ticks 2000000000 --timeout 300`,
release 계측 빌드 · loadavg 36~60):

| 바이너리 | 실행 | 0x7acb4 unwind | 다음 PUSH 의 prev | `address: 0` |
|---|---|---|---|---|
| main `2efea821`(#287 착지 뒤) | 3 | 2/3 | 그 프레임(0x4a85d400) — 이후 `cur_sp=0x400ffeac` 로 **같은 죽은 프레임에 재unwind** | **2/3**(unwind 난 2회 전부) |
| 수정본 | 3 | 3/3 | **0x0** | **0/3** |

덤프는 원 회차와 같다(`SP 0x40100004 · PC 0x7100026a`). 검수자의 0회는 unwind 자체가 나지 않은 실행이다(main 1/3 도 그랬다).
★`address: 0` 이 난 main 실행도 `"result":"PASS"` 였다 — 미처리 스레드 예외는 판정에 들어가지 않는다(형제 #296 의 축).
`0256` 회차의 임시 키 스크립트는 보존되지 않았다(복원 불가) — 위 명령이 그 대체다.
명령·계측 패치 = 증적 디렉터리 `repro.sh` · `scratch_trace_instrumentation.patch`.

**사용자 영향**: catch 에서 예외 프레임을 치우는 LGT 게임(현영맞고2006·놈3 등)에서, 잡힌 예외 뒤 바깥 try 가 체인에서
조용히 빠지던 잠복 결함이 막혔다 — 배틀몬스터의 `address: 0` 수리는 그대로 유지된다.

**게임 파일명 유입**(`corpus-name-inflow --corpus` · 브랜치 전체 7파일): BOUNDED 31 · SUFFIX-ATTACHED 7(재병합 뒤) — 전부 주석·문서 속 타이틀명(계측 근거 인용) · 게임 바이트 0.
