## [2026-09-27] LGT 되던지기 무한 루프(hang 5종) — #292 가 이미 닫았다 · 코드 변경 0 (wie-lgt-unwind-handler-frame-reentry-rethrow-loop)

**무엇을**: 제안 `2026-09-25-surdenattack-pocket-wall-is-dos14-not-pre-save#p0`(catch 처리기가 되던지면 `exception::unwind` 가
그 처리기 프레임으로 되돌아가 무한 반복 — 간호사타이쿤2·서든어택포켓·턴·훼밀리마트타이쿤·학교가는길)을 채택해 대전제부터 다시 쟀다.
★**전제가 반증됐다** — 그 루프는 #292(`7bac9acd`)가 착지하며 사라졌다. 이 회차는 측정과 처분 기록만 한다.

**왜**: 티켓이 「#292 착지 뒤 그 위에서」를 조건으로 걸었고, #292 는 unwind 가 catch 진입 시 **프레임을 소비**(체인에서 떼고 해제)하도록 바꿨다 —
제안이 요구한 «처리기에 진입한 프레임을 다음 unwind 가 건너뛴다»와 같은 효과다. 이중 pop 걱정(Thumb catch 의 자기 pop)은
#292 의 consumed-sp 장부가 이미 무해화한다.

**사용자 영향**: 없음(측정 회차).

### 측정 — 같은 스크래치 계측(`unwind` 의 복귀 직전 1줄 로그 · 미커밋) · `wie_validate --timeout 60` · release

| 타이틀 | #292 직전 `81818e84` | 현 main `4aee5055` |
|---|---|---|
| 간호사타이쿤2 | hang · unwind **7,388,173** (한 복귀 주소 `0x29650` 이 7,388,172) | hang · unwind **2** — 벽은 `Unimplemented: java/lang/Object vtable index 9` |
| 서든어택포켓 | hang · **7,310,439** (`0x21e0`) | 0.29s 하드 실패 · unwind 1 — `DataOutputStream 14`(#328 이 다룬다) |
| 턴 | hang · **4,950,846** (`0xc130`) | 0.15s · unwind 0 — `DataOutputStream 14`(#328) |
| 훼밀리마트타이쿤 | hang · **5,092,554** (`0xc5808`) | 0.28s · unwind 0 — `DataOutputStream 14`(#328) |
| 학교가는길 | hang · **6,832,517** (`0xa4c04`) | 0.18s · unwind 2 — `Timer 12`(#329) |

- 루프가 가리던 벽이 이제 이름으로 드러난다 — 5종 모두 «되던지기 루프»가 아니라 **각자의 미구현 칸**에서 멈춘다.
- 간호사타이쿤2 의 `Object 9`(= `wait()`)는 #310 의 열린 제안(`간호사타이쿤2(LGT) — java/lang/Object vtable index 9 빈 칸`)이 이미 지고 있다 — 새 제안을 만들지 않았다.

### 시험 — 재진입 루프 가드는 이미 있다

#292 의 `rethrow_from_catch_reaches_the_enclosing_frame` 가 이 루프의 단위 시험이다. 개악(unwind 가 프레임을 떼지도 해제하지도 않게 두 줄 제거)으로
`cargo test -p wie-lgt --lib exception` 을 돌리면 **12 중 11 red**(그 시험 포함) — 가드가 살아 있다. 새 시험은 더하지 않았다(중복).

### 검증

- 네 게이트 + beta clippy — 결과는 회신에 적는다(코드 변경 0 · 이 문서와 worklog 만).
- 증거(저장소 밖 · 게임 바이트 0): `~/orchestrator/reports/evidence/wie-lgt-unwind-handler-frame-reentry-rethrow-loop/` — 두 트리의 unwind 계수 표.
