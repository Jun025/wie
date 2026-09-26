## [2026-09-26] 놈3 스택 넘침 — Rust 가 부른 게스트 호출이 남긴 예외 프레임을 해제 · String 26 = indexOf(S,I) (wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0-fix3)

**무엇을**: ⑴`wie-lgt/src/runtime/java/exception.rs` — `mark`/`release_to` 추가. `JavaMethod::run`(`jvm_support/method.rs`)이
`run_function` 전에 이 스레드의 체인·소비 기억을 기록하고, 돌아온 뒤(정상 · 예외 · 호스트 오류 모두) 그 뒤에 push 된 프레임을
해제하고 기억을 되돌린다. 기록한 프레임이 체인에 없으면(불균형 pop) 아무것도 해제하지 않는다.
⑵`wie-lgt/data/lgt_java_abi.toml` — `java/lang/String` 26 = `indexOf(Ljava/lang/String;I)I` + `abi_rows_cover_…` 고정 시험 행.
⑶검수 minor: 생존 변이 B·D·F 를 잡는 시험 2건 · 재시도 고리 누적의 최악 결과를 주석에 명시.

**왜**: 게이트② 3차 반려 critical — `--inject --boot-secs 15 --action-secs 1.2 --max-ticks 100000000000` 에서 #292 가 놈3 을
호스트 main 스레드 스택 넘침으로 죽인다(#306 보고). 검수자 재측은 부하로 대조군도 재현하지 못해 «판정 불가»였다.

**원인**(계측 빌드 · 원문 `~/orchestrator/reports/evidence/wie-2026-09-23-bytearrayinputstream-skip-slot-adopt-p0-fix3/`):
1. 게스트 `paint`(0x31405)를 Rust 가 `JavaMethod::run` 으로 부른다. 그 안 세 단(sp 0x400ffd84·ffd44·ffd0c)이 try 를 push 한 뒤,
   맨 안쪽(0x1f84c · split 루프)이 `Unimplemented: java/lang/String vtable index 26` 으로 **호스트 오류** — `run_function` 이 중단되고
   세 프레임이 체인에 남는다(`NESTEDLEAK … entry_top=0x4a85f000 exit_top=0x4a85fd00` · paint → handlePaintEvent → serviceRepaints 5회 모두 같은 두 값).
2. 그 오류가 `net/wie/WieError` 로 바뀌어 paint 의 호출자(sp 0x400fff14, 던진 곳 lr 0x31691)에서 던져진다 — **자기보다 깊은 sp 의 죽은 프레임**으로 unwind.
3. main: 첫 죽은 catch(0x1f867)가 pop 없이 재던지고(0x1f91b) 프레임을 소비하지 않으니 같은 프레임으로 **3,000,964회** 되돌아온다 — 스레드 하나가
   영원히 도는 livelock 이지만 할당이 없어 판정은 PASS(paints 599).
4. #292: 프레임을 소비하므로 죽은 catch 셋(0x1f867·0x20b87·0x21a07)을 **죽은 스택 위에서 실행**한 뒤 스레드 최상위 재시도 고리
   (push lr 0x1c7d9 → 던짐 lr 0x1cc49)에 들어가 매 회 **새 예외 객체**를 만든다(39,992회). 게스트 힙이 바닥나 `ARRAYERR len=47 Allocation failure`
   6,036회 → `JavaArrayClassDefinition::instantiate_array` 의 오류 경로가 `jvm.exception` 을 부르고 그것이 다시 배열을 할당해 **호스트 재귀** → 넘침
   (충돌 보고 상위 프레임 = `Jvm::exception` ↔ `instantiate_array` 교대).

**String 26 근거**(호출부 .text 0x1f8ce): r0 = s · r1 = delim · r2 = from, 결과를 `cmp r0,#0 ; blt` 로 검사하고 `substring(from, i)`(28)의
끝 인덱스로 쓰며 `from = i + delim.length()` — length() 가 r1 에 String 10번으로 디스패치되므로 r1 은 String, r2 는 int. 규칙상 이웃
(25 indexOf(S) · 27 substring(I))과도 맞는다. rustjava `5b84dd1` 에 구현 존재.

**놈3 4종 일정**(스크래치 launchd `ProcessType=Interactive` · non-LTO release · main `5434ab0a` · head `eb025ffa` · fix = 이 커밋의 워킹트리):

| 짝 | main | head | fix |
|---|---|---|---|
| 1 | PASS · ticks 57.4M · paints 601 | **스택 넘침 rc 134** | PASS · ticks 280.5M · paints 693 |
| 2 | PASS · 54.8M · 599 | **스택 넘침 rc 134** | PASS · 296.8M · 695 |
| 3 | PASS · 66.3M · 599 | **스택 넘침 rc 134** | PASS · 297.6M · 695 |

별도 head 단독 2/2 넘침. 진단 변형 «release_to 만(26 행 없음)»: PASS 2/2 · paints 599(= main) — **넘침은 프레임 해제만으로 사라지고**,
26 행은 paint 자체를 살려 게임 스레드가 실제로 돈다(ticks 약 5배 · paints 599 → 693~695).
레인 셸(CPU 클램프)에서 먼저 잰 head 2회는 넘치지 않았다(ticks 566,070 · 994) — 검수자 재측과 같은 «굶주림 = 판정 불가» 형태.

**다른 LGT 4종**(같은 일정 · head/fix 교대 2회): 메이플스토리2007 · 현영맞고2006 · 체스마스터 · 배틀몬스터 **16/16 PASS**.

**시험**: 새 4건 — `cx_thumb_catch_throws_after_nested_try_before_own_pop`(F) · `enclosing_frame_discards_entries_of_catches_that_never_pop`(B·D) ·
`release_to_drops_frames_a_rust_invoked_guest_call_left_behind` · `release_to_frees_nothing_when_the_mark_left_the_chain`. 반례 3건 무수정.
**변이**: B·D·F → 각 1 red · release_to 가 소비 기억 미복원 / 체인 미복원 / 도달 확인 생략 → 각 1 red · 원상 12/12.

**재시도 고리 누적(minor 4) 판단**: pop 없는 catch 가 같은 try 를 다시 push 하면 회당 1항목 — 상한 16 에서 멈추고, 바깥 프레임이 없으면
뒤이은 Thumb catch 의 자기 pop 이 `FatalError` 가 될 수 있다. 정리 로직은 넣지 않았다: 유일한 실측 사례(위 39,992회)는 증상이었고 이번 수정으로 사라졌다.

**게이트**: fmt · clippy `--all -D warnings` · wasm clippy · `+beta` clippy 전부 rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **464 passed / 0 failed**(460 + 4).
러너 블록(fix 빌드): draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS rc0 · keydraw_ktf/lgt(레인 셸) PASS rc0 · paints 55 · last_frame_content true(main 과 같음).
※Interactive 잡 안에서는 keydraw 가 main·head·fix **모두** `UNMEASURED`(stop max-ticks · input_steps 4~7)였다 — release 빌드가 키 일정 전에 `--max-ticks` 를 태우는 환경 산물이다.

**CI**: 직전 head red 원인 = worklog coverage 재측 기한(235) → `--record` 커밋 `824f07f3`.
연번: 이 PR 의 `0273` 과 #308 의 `0273` 이 겹친다. #292 가 먼저 claim(2026-09-25T21:26Z · #308 은 21:54Z) — 규칙상 #308 이 옮긴다.

**남은 것**: ⑴게스트 힙이 바닥나면 `instantiate_array` 오류 경로가 무한 재귀해 «깨끗한 실패» 대신 호스트가 abort 한다(이번 수정은 그 경로에 닿는
원인을 막았을 뿐) ⑵`release_to` 는 실측 경로인 `JavaMethod::run` 에만 걸었다 — `run_function` 을 부르는 다른 Rust 호출부(콜백·클래스 초기화 등)는 그대로다.

**사용자 영향**: 놈3 이 이 일정에서 죽지 않고, 그리던 화면의 split 루프가 실제로 돌아 paint 가 성공한다(main 은 해당 스레드가 멈춰 있었다).

**게임 파일명 유입**(`corpus-name-inflow --corpus` · 브랜치 전체 11파일): BOUNDED 44 · SUFFIX-ATTACHED 8(`0273` 시점 37 · 8 → 이 회차 주석·문서의 타이틀명 +7) — 전부 주석·문서 속 타이틀명(계측 근거 인용) · 게임 바이트 0. `0273` 의 표식은 이 회차가 브랜치에 파일을 더해 낡아 떼고 이 파일로 옮겼다.

<!-- corpus-name-inflow v1 subjects=11 tree=c3d388466b25bc7a B=99/44 P=1/1 S=15/8 -->
