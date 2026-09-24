## [2026-09-25] LGT 예외 프레임 스택을 스레드별로 — 교차는 실재했다(2/91 타이틀) (wie-2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured-adopt-p1)

**무엇을**: `wie-lgt/src/runtime/java/exception.rs` 의 예외 프레임 연결 리스트가 모든 게스트 스레드의 공유 칸 하나(`0x7fff0000`)였다. 이제 그 칸은 **스레드별 기록**(thread_key · 현재 프레임 · pending · next)의 목록 머리이고, push/pop/pending/unwind 는 모두 «지금 들어가 있는 스레드»의 기록만 본다. 스레드 판별을 위해 `ArmCore::current_thread_id()` 를 추가했다(`ThreadContextGuard` 가 진입 시 세우고 나갈 때 이전 값으로 되돌린다 — 중첩 안전).

**왜**: 제안(`2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured#p1`)은 «관측 피해 0 — 먼저 교차가 실제로 나는 타이틀을 찾아라»였다. 그래서 먼저 쟀다.

**1단계 — 관측(프로브, 커밋 안 함)**: push/pop/unwind 마다 «현재 스레드 id» 와 «리스트 머리 프레임을 push 한 스레드 id» 를 stderr 로 찍는 스크래치 패치(`ArmCoreThreadWrapper::poll` 에 전역 tid · exception.rs 에 프레임→소유 tid 맵). LGT 코퍼스 `game_lab/{working,broken}/lgt` 91개를 `wie_validate <zip> --inject` 로 1회씩.

| 타이틀 | push | pop | unwind | 교차 push | 교차 pop | 교차 unwind | 스레드 |
|---|---|---|---|---|---|---|---|
| 놈3 | 213 | 210 | 0 | **49** | **6** | 0 | 1,2 |
| 현영맞고2006 | 20 | 16 | 3 | **2** | 0 | 0 | 1,2 |
| (프레임을 쓰는 나머지 11) | — | — | — | 0 | 0 | 0 | 스파이더맨3·메이플스토리2007 은 2스레드 |

- 프레임을 쓰는 타이틀 13 / 91 · **교차 타이틀 2** · unwind 교차 0 · 크로이센 1건은 디버그 빌드 main 스레드 스택 오버플로(rc=134, 프로브 무관).
- **교차 pop 은 실제 손상이다**: 놈3 원문 — `PUSH cur=2 head=…be80 owner=1` 다음 `POP cur=1 head=…ca80 owner=2` ⇒ 스레드 1 이 스레드 2 의 프레임을 풀어 해제했고, 스레드 2 의 체인 머리는 스레드 1 의 프레임이 됐다. 그 뒤 스레드 2 에서 예외가 나면 스레드 1 의 레지스터로 catch 가 복원된다.

**2단계 — 이전**: 위 표가 1건 이상이라 계약 2단계를 집행. 게스트는 이 칸을 읽지 않는다(값은 SVC 로만 넘어간다 · `git grep 0x7fff0000` 에서 LGT 쓰임은 이 파일뿐) ⇒ 게스트 ABI 무접촉.

**시험**: `exception_frames_are_per_thread`(놈3 의 교차를 그대로 재연 — 스레드 2 push → 스레드 1 pop → 스레드 2 unwind 가 자기 r4/lr 로 복원되는가) · `current_thread_id_follows_the_entered_context`(중첩 guard 복원). 변이: 스레드 키를 상수 0(= 옛 공유 체인)으로 → **red**(`left: Some(4097) right: None` — 스레드 1 이 남의 프레임을 pop 하고 자기 프레임으로 unwind) · 원상 → green.

**before/after — 판정은 «차이 없음»이고, 그게 정직한 답이다**: 프레임을 쓰는 13 타이틀을 before(프로브)/after(수정) 바이너리 **교대로 같은 루프에서 5회**: PASS 합계 **5 ↔ 5**, 타이틀별 PASS 수도 전건 동일(놈3 0/5 ↔ 0/5 · 현영맞고2006 1/5 ↔ 1/5). loadavg 150~167 이라 paints 가 대부분 0~11(유휴 ~50) — 판정은 부하로 굶었다. 비교대(非交代) 전체 91 재측은 PASS 19 → 10 으로 요동했지만 뒤집힌 17 타이틀 중 12 는 프레임을 **한 번도** push 하지 않아 바뀐 코드에 닿지 않는다 ⇒ 부하 잡음이지 회귀가 아니다(AGENTS.md §The four gates 의 굶김 절 그대로).
⇒ 이 회차가 산 것은 «보이는 개선»이 아니라 «놈3·현영맞고2006 이 다음 벽을 넘었을 때 엉뚱한 catch 로 뛰지 않는 것»이다. 두 타이틀의 현재 벽은 둘 다 `no frame rendered` 로 이 결함 밖이다.

**사용자 영향**: 지금 당장 달라지는 게임 화면은 없다. 여러 흐름이 동시에 try/catch 를 쓰는 LGT 게임(놈3·현영맞고2006)에서 예외 처리 상태가 섞이지 않게 됐다.

**한계**: 스레드가 try 안에서 끝나면 그 스레드의 기록·프레임(20~88바이트)이 남는다(스레드 id 는 재사용되지 않으므로 오동작은 없다 · 이전 구현도 같은 프레임을 남겼다). KTF 는 무접촉.

**게이트**(2026-09-25 01:54~02:33): fmt rc0 · clippy `--all -D warnings` rc0 · wasm clippy rc0 · `cargo +beta clippy` rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **434 passed / 0 failed** · `clippy --workspace --all-targets` 경고 16 = 16(접촉 파일 0). 러너 블록: draw_j2me·helloworld_ktf·helloworld_lgt·text_j2me rc0 · **keydraw_ktf/lgt rc1**(paints 10·8). §The four gates 의 ⑴~⑷를 그대로 밟았다: 수정본/변경 전 바이너리를 같은 루프에서 교대 4회 ⇒ **양쪽 모두 8/8 FAIL · paints 3~13**(유휴 48~55) · loadavg 217~236 ⇒ 굶김이지 회귀가 아니다(같은 변경 전 바이너리가 저부하 때 keydraw_lgt PASS · paints 38).

**게임 파일명 유입**: `node scripts/corpus-name-inflow.mjs --corpus <코퍼스>` ⇒ BOUNDED 24회/8쌍 · SUFFIX-ATTACHED 0 — 전부 관측 근거로 적은 타이틀명(놈3·현영맞고2006 등)이고 게임 바이트는 0.

<!-- corpus-name-inflow v1 subjects=4 tree=71c6dfa1df7efc1c B=24/8 P=0/0 S=0/0 -->
