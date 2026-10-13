## [2026-10-14] WIPI 타이머 — `MC_knlDefTimer` 는 대기 중인 타이머를 지운다 · 0520 보정 (wie-timer-reject-followup-def-reset-and-report-correction)

**무엇을**: `def_timer` 가 그 타이머의 대기(pending)를 지우게 했다. 그리고 0520 의 서술 두 곳(체감 속도 · r0)을 바로잡았다. 출처는 #553 게이트② 검수(`~/orchestrator/reports/wie-timer-double-set-per-callback-queue-growth.review.md` §3·§4 · pin `306054dd`)의 비차단 지적 3건이다.

**왜**: 0520 이 따른 참조 에뮬레이터는 Def 할 때 타이머를 먼저 비운다. wie 는 비우지 않았다. 그래서 대기 중에 Def → Set 하면 wie 는 -1 을 돌려줬고, 참조 구현은 0 을 돌려준다.

**사용자 영향**: 아래 §2 의 전수에서 이 코드 변경으로 실행이 바뀌는 타이틀은 **0종**이다. 문서 보정(§3)은 #553 이 이미 낸 체감 변화를 적는다. 2종(`3cc7a9b4cb15` `79d9bc10be82`)이 게임이 요청한 주기로 돌아와 전보다 느리다. 이용자 소식 1건을 더했다.

### 1. 근거와 바꾼 것
- 참조 `WIPIEmul.exe`(0520 §1 과 같은 설치본)의 Def(`MG_NativeEnv` 24번 칸 `0x448493`)는 `cb`·`prg` 를 쓰기 **전에** `0x4477eb` 를 부른다. 그 함수는 `type=2 · alarmNext=0 · alarmWakupFunc=0 · alarmTime=0` 으로 만든다(검수 §4-3 의 디스어셈블). `alarmWakupFunc` 는 0520 §1 이 «걸린 동안에만 0이 아니다»로 읽은 칸이다. ⇒ **재Def 는 대기를 지운다.** 0520 §1 의 Def 서술에서 이 호출이 빠져 있었다.
- `wie-wipi-c` `def_timer`: 구조체를 쓴 뒤 `event_queue().cancel_timer(ptr_timer)` 를 부른다(Unset 과 같은 함수). 지워진 arming 은 `fire_timer` 가 `None` 을 돌려줘 콜백하지 않는다.
- 한계: 참조 구현은 목록에 매달린 노드를 떼지 않고 지운다(검수가 «UB» 로 적었다). wie 는 그 노드를 목록에 따로 두지 않으므로 대응되는 동작이 없다. 옛 arming 은 콜백 없이 사라진다.
- 시험 `kernel::test::test_set_timer_on_a_pending_timer_fails` 끝에 다음을 더했다. 대기 중 Def(콜백 `0x5678`) → Set = 0 이고, 발화하는 것은 Def 뒤 arming 하나뿐이다(`(0x5678, [timer, 4])`). ★**되돌리면 red**: `cancel_timer` 줄을 지우면 이 시험만 FAILED 다(3 pass · 1 fail · `kernel.rs:572`). 원복하면 4 pass.

### 2. 퇴행 전수 — 대기 중 재Def 가 실제로 일어나는가
- 모집단: 0520 의 짝 CSV 111종 + `79d9bc10be82` = **112종**(sha12 ↔ 파일 112/112 대응).
- 계측: 이 브랜치에 커밋하지 않는 debug 줄 1개를 더한 스크래치 빌드다. 그 줄은 `def_timer` 진입 시 그 타이머가 대기 중일 때만 `DEFRESET` 을 찍는다. release `wie_validate` · `RUST_LOG=wie_wipi_c::api::kernel=debug` · census probe A 인자(`--inject --pacing 8 --timeout 30`) · `--jobs 3` · 38종 단위로 `build-slot run --long` 3회(04:4x–05:03 · load1 10~13).
- **결과: `DEFRESET` 0회 / 112종.** Def 호출은 112종 모두에 있다(합 5,146회). 같은 주소를 두 번 넘게 Def 하는 타이틀은 **16종**이다. 그중 5종은 Set 마다 Def 를 다시 부른다(Def 수 = Set 수). 16종 모두 Def 시점에 그 타이머는 대기 중이 아니었다(발화 뒤나 Unset 뒤).
- 16종 연장 재측(`--keep-timeout --max-ticks 100000000 --timeout 120`): Set 수가 30초 런의 약 1~6배로 늘었고, **`DEFRESET` 은 여전히 0회**다(stop: deadline 6 · max-ticks 8 · clean exit 2). ※처음 친 `--timeout 120` 은 `--inject` 의 일정 기반 마감에 덮여 30초 런과 같은 수를 냈다. `--keep-timeout` 을 붙여 다시 쟀다.
- ⇒ 이 코드 변경으로 실행이 바뀌는 타이틀은 **0종**이다. 그래서 전·후 짝을 내지 않았다. 내 봐야 같은 경로를 두 번 재는 부하 차이일 뿐이다. 이 변경은 참조 구현과의 의미 일치이고, 실제로 그 경로를 타는 타이틀은 아직 관측되지 않았다.
- 한계: 30초/120초 창과 고정 27키 입력이 닿는 화면까지만 봤다. 그 밖의 화면(스테이지 전환 등)에서 대기 중 재Def 가 있을 가능성은 남는다.

### 3. 0520 보정
**⑴ 체감 속도(검수 §3·§4-1)** — 0520 은 `3cc7a9b4cb15` 를 「거절 1회 · speed ok → ok」로만, `79d9bc10be82` 를 「런 1쌍으로 못 갈랐다」로 적었다. 검수의 직접 재측(before `b88b104c` / after `306054dd` · release · probe-A 인자)은 둘 다 갈랐다.

| sha12 | before: 발화 · paints | after: 발화 · paints | 출처 |
|---|---|---|---|
| `3cc7a9b4cb15` (2쌍) | 548/549 · 674/676 | 282/283 · 413 | 검수 §3 |
| `79d9bc10be82` (3쌍) | 162·162 · 518·522 (1회 이상값 459 · 1310) | 103·102·103 · 371×3 | 검수 §3 |

- after 3/3 이 같다(결정적). 변형 바이너리 분해(검수 §3 V1/V2)로 보면 차이는 전부 «거절» 몫이다.
- 그 의미: 콜백 안 재Set 의 요청 timeout 은 3cc7 중앙 54ms · 79d9 220ms 다. before 의 재Set 간격은 **33ms · 90ms** 였고 after 는 70ms · 240ms 다. before 는 같은 타이머 사슬이 2~2.4개 겹쳐 돌아, 게임이 스스로 요청한 주기보다 빨랐다. ⇒ **약 1.6~2배 느려진 것은 퇴행이 아니라 충실도 교정이다.** 다만 이용자는 체감한다.
- compat 의 speed 축(지연 비율)은 «요청 주기 대비 발화율»을 재지 않는다. 그래서 이 변화가 판정에 나타나지 않는다(두 행 다 `ok` 그대로 · compat 변경 0).
- 이용자 소식 `docs/player-updates/2026-10-14-timer-games-back-to-their-own-pace.json`(kind `speed` · 두 sha).

**⑵ r0(검수 §4-2)** — 0520 의 「다른 타이머 게임 107종은 실행 경로가 같다」는 엄밀히 참이 아니다. 종전 `set_timer` 는 `Result<()>` 였고 빈 결과는 r0 을 건드리지 않는다. 그래서 성공한 Set 뒤 r0 은 첫 인자 `ptr_timer`(0 아님)였다. 지금은 0 이다(참조 `xor eax,eax` 와 일치). 107종 모두 성공한 Set 마다 이 값이 바뀌었다. 검수의 V1/V2 분해에서 실측 효과는 0 이고(107행 판정 변화 0), 그 107종에서 바뀐 것은 이 반환값 하나뿐이다.

**⑶ 거절 횟수(검수 §4-4 · 참고)** — 0520 의 거절/Set 수는 런 인자에 따라 흔들린다(검수 debug 런: 3cc7 4/420 · cdd3 758/1517 · 79d9 5/177). 결론은 같다.

### 4. 게이트
- fmt · clippy `-D warnings`(stable · wasm32 · beta) rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 54 스위트 784 pass · 0 fail.
- 러너 블록(debug `cargo run`): draw_j2me · helloworld_ktf · helloworld_lgt PASS · keydraw_ktf/lgt `--inject --expect-last-frame` PASS rc=0 · text_j2me PASS.
- `npm run build:wasm` → `check-engine-contract` 137 pass · 0 violation. compat 변경 0.

### 5. 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs`: BOUNDED 1쌍 + SUFFIX-ATTACHED 0쌍. 그 1쌍은 `wie-wipi-c/src/api/kernel.rs` 에 `origin/main` 부터 있던 기존 주석이다. 이 회차가 그 파일을 고쳐서 대상에 들어왔을 뿐이고, 이 회차 diff 의 그 문자열은 0회다. 이 회차는 타이틀을 sha12/sha256 으로만 적었다.



<!-- corpus-name-inflow v1 subjects=4 tree=42282fec12c0aa1a B=1/1 P=0/0 S=0/0 -->
