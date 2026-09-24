## [2026-09-24] SD한국전쟁 — LGT 진입점은 «실패»가 아니라 «다른 곳을 가리켰다» · `.raptor` 진입점을 따른다 (wie-2026-09-22-aot-java-address-zero-cluster-adopt-p4)

**무엇을**: LGT 로더 `load_executable`(`wie-lgt/src/runtime/init.rs`)이 진입점을 ELF `e_entry` 대신 `.raptor` 헤더의 진입점 오프셋(+ 첫 실행 섹션 주소)에서 고른다. 헤더가 없으면 종전대로 `e_entry` 를 쓴다.

**판정 — «조용한 실패»가 아니라 «다른 규약»이다.** 근거는 main `c3225fe3` 에서 잰 binary.mod 정적 해석이다.
- ⒜ 전제 재현: 현 main 에서 여전히 tick 0 · `LGT entrypoint returned without publishing an init struct (ptr_init_struct is null)` · PC 0 이다.
- `e_entry = 0x1000` 은 초기화 스텁이 아니다. `0x1318` 을 부르는데, 이것은 `[[param1+8]+8]` 을 2항 클래스 표(`0x3c29c` · `"SDWar"`·`"o"` 이름 포인터)와 대조해 일치할 때만 핸들러를 부르는 **클래스 레지스트리 디스패처**다. param1 이 0 으로 채워져 있어 일치 0 ⇒ 아무것도 쓰지 않고 정상 반환한다. 오류로 반환한 것이 아니다.
- 표준 LGT 초기화 스텁은 이 바이너리에도 **있다**. `0x3baec` 가 r0/r1/r2 를 전역에 저장하고 `param1+0x214`(= `ptr_init_struct`)를 채운다. 다른 타이틀의 `e_entry` 가 가리키는 코드와 같은 형태다. 지연 임포트 리졸버(`0x3ba5c`)는 그 스텁이 저장한 param2(`[0x1500270]`)에서 `fn_get_import_function` 을 읽는다.
- `.raptor` +12 워드 = `0x3aaed` ⇒ `.text`(0x1000) 기준 `0x3baed`(Thumb) = 그 스텁이다.
- 코퍼스 90 바이너리(working+broken LGT, 91 zip) 전수: `.raptor` 진입점 == `e_entry` 가 **89**, 다른 것은 **SD한국전쟁 1건**이다(와일드프론티어는 `.text` 대신 `ER_RO` 라서 «첫 실행 섹션»을 기준으로 삼았고, 그 기준으로 일치한다). 이 바이너리만 `.raptor` 헤더 길이 0x58, 라이브러리 목록 `kernel dlet cldc wipijava lgte`, 빌드 식별자 `0000DF53`(코퍼스 최저)를 가진다 ⇒ 구판 툴체인이다.

**전/후**(`wie_validate --timeout 20`, 부하 ~40):

| | result | ticks | paints | 문면 |
|---|---|---|---|---|
| before | FAIL | 0 | 0 | `…without publishing an init struct…` |
| after | **PASS** | 5,180,936 | 3 | `booted + rendered` · distinct_colors 64 |

**양방향 변이**: ⑴ `choose_entrypoint` 가 `e_entry` 를 돌려주게 바꾸면 → 시험 `entrypoint_follows_raptor_header` FAIL, SD한국전쟁은 원래 문면으로 FAIL. ⑵ 헤더가 없을 때의 폴백을 `0` 으로 바꾸면 → 같은 시험 FAIL. 두 변이 모두 되돌린 뒤 green 이다.

**LGT 전 타이틀 짝 회귀**(before/after 바이너리, 타이틀마다 연속 실행, `--timeout 10`, `-P 6`, loadavg 28–44): 짝 90(크로이센은 양쪽 모두 stack overflow rc=134 · 선재) · 판정·벽 유지 86 · 변경 4 · PASS 29 → 31. 변경 4건 중 SD한국전쟁을 뺀 3건(리듬스타1 F→P · 그랜드체이스 F→P · 판타지포에버3 P→F)은 방향이 섞여 있다. 이 셋은 진입점 값이 전과 후에 **같다**(위 89건에 속함). 4회 짝 재측에서는 16/16 짝이 같은 판정이었다(전부 기아 상태 FAIL, ticks 수십) ⇒ 부하 잡음이다.

**게이트**: fmt·clippy `-D warnings`·wasm clippy·beta clippy 전건 rc=0 · `cargo test --all` 423 passed / 0 failed(46 suites) · `clippy --workspace --all-targets` 경고 16줄, 전부 무접촉 크레이트(`wie-backend`·`wie-jvm-support`·`wie_cli` 테스트 타깃) ⇒ 증가 0. 러너 블록 6픽스처 PASS. keydraw 양쪽 before/after 모두 `PASS` · rc=0 · 27/27 · paints 55(loadavg 10). 부하 40대에서 한 차례 `UNMEASURED` 가 나왔지만 재측에서 해소됐다.

**사용자 영향**: SD한국전쟁이 부팅해서 화면을 그린다. 다른 LGT 타이틀은 진입점 값이 같으므로 실행이 바뀌지 않는다.

**유입**(`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab`): BOUNDED 와 SUFFIX-ATTACHED 를 아래 표식에 함께 적었다. 전부 이 회차가 측정하거나 근거로 든 타이틀(SD한국전쟁 · 대조군 검은방2 · 와일드프론티어 · 짝 회귀 4건)을 이름으로 적은 것이다. SUFFIX 는 모두 조사가 붙은 언급(«…은/을/는/이»)이고 다른 제목이 아니다. `init.rs` 주석의 타이틀명은 main 에 이미 있던 주석을 이은 것이다.


<!-- corpus-name-inflow v1 subjects=3 tree=4c4c986ba93362b9 B=14/9 P=0/0 S=6/4 -->
