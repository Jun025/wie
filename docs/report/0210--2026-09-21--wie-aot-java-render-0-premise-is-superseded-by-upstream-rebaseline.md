## [2026-09-21] AOT-Java 「렌더 0」 재기준선 — 3종이 렌더하고, 배틀몬스터는 §7 «앞»에서 죽는다 (wie-aot-java-render-0-premise-is-superseded-by-upstream-rebaseline)

### 무엇을

`game_lab/broken/lgt` 의 **AOT-Java 24파일(고유 18종)** 을 `--inject` 로 전수 재실행하고, 그 결과로
「LGT AOT-Java 24종은 렌더 0」이라는 낡은 결론을 문서 6곳에서 정정했다. **엔진 동작 변경 0** —
바꾼 것은 산문과 doc 주석뿐이다.

### 왜 — 그 한 줄이 «제품이 게임을 막는 근거»였다

`apps/featurephone` 이 `lgt_compile_model()==="aot-java"` 로 24종을 일괄 차단하는데, 그 차단의 근거가
2026-07 에 닫힌 cp43/45/48 의 「0 render」다. 그 뒤 **upstream `cc652b1d`(2026-08-04
「Implement LGT Java AOT runtime (#1337)」)** 가 AOT 실행 모델을 갈아치웠고, **아무도 재측정하지
않았다.** 약 7주 동안 그 한 줄이 현재형으로 남았다.

### 측정 — 전수표 (고유 18종 · release `wie_validate` · `--inject` 기본 예산 20.0s)

바이너리 = 이 회차가 `394fde8b`(base merge 전 `origin/main`)에서 직접 빌드한 `target/release/wie_validate`.
★**debug 로 시작했다가 갈아탔다** — loadavg 100 대의 debug 는 `놈3` 을 20초에 **38~69 ticks** 밖에
못 돌려 `paints 0` 으로 떨어뜨렸다(= 거짓 음성). release 의 같은 타이틀은 **1,834,704 ticks**다.

| 타이틀 | sha256(앞8) | PASS/실행 | paints(최대) | distinct_colors(최대) | 벽 문면(FAIL 시) |
|---|---|---|---|---|---|
| **현영맞고2006** | `1b107b96` | ★**6/6** | 154 | 512 (nondominant 90.3%) | — |
| **메이플스토리2007** | `13d7e3c2` | ★**5/6**※ | 121 ★(PASS 회차 최대는 **120**) | 512 (87.1%) | `net.wie.WieError: Unimplemented: java/lang/Thread vtable index 13` @`24_HASH` (1회) |
| **놈3** | `b475b639` | ★**5/7** | 69 | 93 (10.3%) | `no frame rendered (hang/black screen)`, `paints 0` (2회 = 부하 기아) |
| (LGT)턴 / 턴 | `a16f08d0` | 0/4 | 0 | 0 | `net.wie.WieError: Invalid memory access; address: 0` @boot (ticks 1) |
| LGT 월드장기체스 / 월드장기체스 | `70d709c4` | 0/4 | 0 | 0 | `java.lang.NoClassDefFoundError: wec/SYSTheme` @boot (ticks 1) |
| lgt 레전드 오브 마스터 / 레전드오브마스터 | `735a579d` | 0/4 | 0 | 0 | `Invalid memory access; address: 0` (ticks 2) |
| **lgt 배틀몬스터 / 배틀몬스터** | `a30bbe00` | 0/4 | 0 | 0 | ★`net.wie.WieError: Invalid memory access; address: 0` @boot (ticks **2**) |
| lgt 서든어택 포켓 / 서든어택포켓 | `517ed32c` | 0/4 | 0 | 0 | `Invalid memory access; address: 0` @boot, `at net/wie/LgtClassLoader.findClass` (ticks 1) |
| 당신은골프왕 LGT / 당신은골프왕 | `b2da04c5` | 0/4 | 0 | 0 | `NoClassDefFoundError: org/kwis/msp/lwc/DialogComponent` @boot (ticks 1) |
| SD한국전쟁 | `61ed6952` | 0/3 | 0 | 0 | `Invalid memory access; address: 0` @boot (ticks **0**) |
| 간호사타이쿤2 | `5d0ba949` | 0/3 | 0 | 0 | `NoClassDefFoundError: org/kwis/msf/io/Message` @boot (ticks 2) |
| 붕어빵타이쿤3 | `1a69522a` | 0/3 | 0 | 0 | `NoClassDefFoundError: org/kwis/msp/lwc/DialogComponent` @boot (ticks 1) |
| 슈퍼액션히어로 | `c046d43d` | 0/3 | 0 | 0 | `NoClassDefFoundError: org/kwis/msp/lwc/DialogComponent` @boot (ticks 1) |
| 스파이더맨3 | `d01275d6` | 0/3 | 0 | 0 | `Invalid memory access; address: 0`, `at net/wie/LgtClassLoader.findClass` (ticks 1) |
| 체스마스터 | `4ece6eee` | 0/3 | 0 | 0 | `Invalid memory access; address: 0` @boot (ticks 1) |
| 학교가는길 | `be08d047` | 0/3 | 0 | 0 | `Invalid memory access; address: 0` (ticks 3) |
| 훼밀리마트타이쿤 | `73f3a21e` | 0/3 | 0 | 0 | `Invalid memory access; address: 0` (ticks 2) |
| 일지매영웅전기 | `2a57e331` | 0/3 (+0/4 큰예산) | 0 | 0 | ★`no frame rendered (hang/black screen)` — 다른 계급. 아래 별 절 |

※**메이플스토리2007 의 7회차 중 1회는 `--action-secs 0.3`(추적 실행)이고 나머지 6회는 기본값**이다.
AGENTS.md 가 그 손잡이를 「같은 축의 확률을 옮길 뿐」이라고 못박았으므로 분모에 섞은 것을 밝혀 둔다 —
기본값만 세면 **5/6** 이고, FAIL 1건은 그 6회 안에 있다. 다른 두 타이틀은 전 회차 기본값이다.

★**분류는 엔진 규약을 독립 구현해 재현했다**(`compile_model.rs` 의 `BL`+`.word 0x64`): **aot-java 24 ·
clet 22**, 그 파일 머리주석과 정확히 일치. aot-java 쪽 `0x1fb` 계수는 **전건 0**, clet 쪽 `0x64` 계수도
**전건 0** — 겹치는 타이틀이 없다. 고유 sha256 **18**개 · 중복 **6쌍**(표의 «A / B» 행).
★**중복 6쌍은 각각 두 이름을 다 돌렸고 판정·벽이 일치했다**(형제 파일이 아니라 같은 바이트다).

★★**「벽 문면」의 «스텝 라벨»은 잡음이다 — 벽 자체가 아니다. 이걸 모르면 진척을 오독한다.**
`학교가는길` 은 3회차가 각각 `boot` · `15_NUM5` · `04_NUM5` 로 달랐는데 **ticks 는 3으로 고정**이었다.
★**배틀몬스터가 더 위험한 사례다** — debug 3회차가 `11_LEFT` · `07_DOWN` · `12_OK` 로 나왔다.
「열한 번째 입력까지 살아남았다」로 읽고 싶어지지만 ★**세 회차 전건 `ticks 2`** 이고 release 4회차의
`boot` 와 **같은 자리**다. 죽는 시점은 부팅 tick 2 로 고정이고, 그때 주입 스케줄의 벽시계가 어느
칸에 있었느냐가 라벨을 정할 뿐이다. ⇒ **라벨로 진척을 읽지 마라. `ticks` 로 읽어라.**

### ★PASS 3종 — 스크린샷을 «열어서» 봤다

`game_lab/` 은 git-ignored 라 커밋하지 않는다. 경로(그 트리 안):
`game_lab/reports-2026-09-21-aot-java-render-evidence/rebaseline-0210/`

| 타이틀 | 스텝 PNG | 서로 다른 PNG | 사람이 읽는 화면인가 |
|---|---|---|---|
| **메이플스토리2007** | `메이플스토리2007-steps/` 28장 | ★**19** | ★**그렇다.** `__22_OK.png` = 로고·캐릭터 스프라이트·「PRESS ANY KEY」·「NEXON MOBILE Corporation」이 든 **타이틀 화면**. `__27_OK.png` = 성·구름 배경 일러스트 위에 한글 스토리 텍스트가 얹힌 **인게임 인트로 장면** |
| **현영맞고2006** | `현영맞고2006-steps/` 27장 | 9 | ★**그렇다.** `__12_OK.png` = 하늘 배경·꽃·인물 사진·「2006 현영맞고」 로고·회전 스피너가 든 타이틀 화면 |
| **놈3** | `놈3-steps/` 28장 | 6 | ★**그렇다(단 얕다).** 「시보구게임연구실」 로고 + 초록 스프라이트의 **스플래시**에 머문다. 20초 예산 안에서 메뉴로 넘어가지 않는다 |

★**`놈3` 의 `distinct_colors=2` 를 「검은 화면」으로 읽지 마라** — 이 회차의 release 실행에서는
**93색**이다. 총괄 실측이 본 `dc=2` 와 report 0208 의 `dc=2` 는 **기아된 실행**의 값이고, 큰 예산
(113s)으로 돌리면 `352,857 ticks · 48 paints`로 **PASS** 한다. ★**같은 타이틀·같은 바이너리에서
`dc` 가 2↔93 으로 움직인다 — 그 수를 절대값으로 인용하지 마라.**

### ★필수② — 「§7 은 아직 벽인가, 배틀몬스터는 그 뒤에 있는가」

**⑴ §7 이 서술하는 «기전»은 현 엔진을 서술하지 않는다 — 문서 신뢰가 아니라 실행으로 잰 것이다.**
`docs/lgt.md` §7 의 모델은 「ez-i 는 `TIMER_EVENT(21)` 을 쏘고 앱이 스스로 디스패치하는데, **wie 가
21 을 안 쏴서 `getNextEvent` 루프가 영구 블록**된다」이다. `메이플스토리2007` 을 `RUST_LOG=debug` 로
추적한 값:

| 무엇 | 횟수 | 의미 |
|---|---|---|
| `net.wie.EventQueue::getNextEvent` | **73** | ★§7 이 「영구 블록」이라 한 그 루프가 **돈다** |
| `net.wie.EventQueue::dispatchEvent` | **73** | 디스패치도 돈다 |
| `net.wie.CardCanvas::paint` | **21** | per-frame 카드 페인트 |
| `org.kwis.msp.lcdui.Graphics::drawImage` | **193** | 실제 그리기 |
| `org.kwis.msp.lcdui.Image::createImage` | **44** | 이미지 로드 |
| `org.kwis.msp.lcdui.Display::pushCard` | **1** | ★cp48 의 「never `Display.pushCard`, 카드 벡터가 빈 채로 남는다」를 **직접 반증** |
| `org.kwis.msp.lcdui.EventQueue::getNextEvent` | **0** | 앱은 kwis 래퍼를 안 거친다 |

트리 실측: `TIMER_EVENT` 문자열 **0건**. 구동축은 `RepaintEvent = 41`(`wie-midp/.../net/wie/event_queue.rs`)다.
⇒ ★**「우회됐나 / 다르게 구현됐나」의 답은 «다르게 구현됐다»** — upstream 이 `native_jvm.rs` 브리지를
`net/wie/LgtClassLoader`(진짜 `java.lang.ClassLoader` 서브클래스) + `runtime/java/jvm_support/` 로
갈아치워, AOT 클래스가 **평범한 JVM 클래스**가 됐다. §7 의 벽 A(「서브클래스 정체가 컴파일돼 사라져
`invoke_virtual` 불가」)는 그 모델과 함께 사라진 것으로 보인다. ★**그 인과는 추정이다 — 잰 것은
「루프가 돌고 그림이 나온다」이지 「어느 커밋의 어느 줄이 그것을 고쳤다」가 아니다.**

**⑵ 「§7 이 풀렸다」고는 «말하지 않는다» — 측정되지 않았다.** 나머지 15종은 전부 **ticks 0~3, 부팅에서**
죽는다. §7 은 「부팅 후 per-frame」 벽이므로 ★**그들은 §7 에 도달조차 하지 않는다.** 도달하는 표본이
없으니 §7 이 아직 그들을 막는지는 **알 수 없다.** cp43 의 「전 클러스터가 §7 «앞»에 있다」는 **형태는
그대로 참**이다 — 바뀐 것은 클러스터의 «내용»(오늘의 벽은 `NoClassDefFoundError`·`Invalid memory
access` 이지 `AbstractMethodError paint`·`NoSuchMethodError` 가 아니다)과 「그러므로 0 render」라는 결론이다.

**⑶ 배틀몬스터는 §7 «앞»에 있다 — 그 뒤에 §7 이 있는지는 ★모른다.**
현 벽 = **부팅 tick 2** 의 `net.wie.WieError: Invalid memory access; address: 0`(4회 실행 전건 동일 ·
두 파일명 동일). ★**티켓이 적은 「현 벽 = `Player.resume(Clip)Z` (tick 1)」은 이미 낡았다** —
형제 회차 `wie-battlemonster-aot-java-first-wall-player-resume-clip`(착지 · `docs/report/0208`)이 그
스텁을 닫았고, 벽이 tick 1 → **tick 2 의 메모리 접근 위반**으로 옮겨 갔다. 이 회차의 실측은 그 후속
상태를 확인한 것이다. ⇒ ★**그 메모리 접근 위반을 닫기 «전»에는 §7 이 이 타이틀을 막는지 알 수 없다.
추정으로 채우지 않는다.**

### ★정정한 문면 (6곳)

| 파일 | 무엇을 |
|---|---|
| `wie_featurephone/src/lib.rs` | ★`lgt_compile_model()` doc — 「boots but does not yet render (the §7 wall)」 + 「block `"aot-java"` at upload」 **권고 철회**. 이 getter 가 **otterpebble 차단의 직접 근거**라 가장 먼저 고쳤다 |
| `wie-lgt/src/compile_model.rs` | 머리주석 「(§7 render stays frozen)」 삭제 + `§Corpus` 절 신설(3종 렌더 · 15종 부팅사) · `AotJava` variant doc 의 같은 문장 |
| `docs/lgt.md` | §7 제목에 **SUPERSEDED 배너** + 「배틀몬스터, the one title reaching this wall」에 현행 주석. 별도로 §How We Emulate This 의 **`runtime/java/native_jvm.rs` 서술이 실재하지 않는 파일**이라 현행 구조로 교체 |
| `docs/lgt_abi.md` | §7b(cp43 스윕)에 **SUPERSEDED 배너** · §8 표머리에 「cp 행은 날짜 붙은 기록이다」 주의 · `AOT-Java title sweep … 0 render` 행 취소선 + **재기준선 행 신설** · `per-frame render driver` 행에 추적 실측 |
| `docs/project-kb/02_status.md` | 트랙② 전문 교체 · L25 getter 설명의 「부팅되나 §7 벽으로 미렌더」 · **권고 셸 패턴 철회** · L290 「AOT 24종은 §7 동결」 |
| `docs/project-kb/10_deep-assets.md` | §7 절에 **정정 블록** + 「화면을 못 그리는 단 하나의 벽」 취소선 |

★**cp 기록 자체는 한 줄도 지우지 않았다** — 사료이고 그 시점엔 옳았다. 전부 «배너 + 현행 실측»을
얹는 방식이다.

### ★정정하지 «않고» 남긴 것 — 그리고 그 이유

- **「유일 블로커 = `0x64` ordinal→native 등록표 / 실기 트레이스 필요」**(02_status·10_deep-assets·lgt_abi).
  ★**이 회차가 재지 않았다.** 렌더가 나온 이상 「유일」은 최소 3종에 대해 참일 수 없지만, 그 표가
  **다른 용도로** 여전히 필요한지는 미측정이다. ⇒ 「측정하지 않은 것을 정정하지 마라」에 따라 손대지 않고,
  각 자리에 「이 회차 미측정」이라고 **명시**했다.
- **`docs/contracts/featurephone-engine-contract.md` 의 「셸은 "aot-java" denylist」**. 그것은 wie 의
  사실 주장이 아니라 **셸이 무엇을 하는지에 대한 계약 서술**이다. 바꾸는 주체는 otterpebble 이고,
  이쪽이 먼저 고치면 계약이 실물과 어긋난다. ⇒ 그대로 뒀다.
- **cp43 의 버킷표(X-paint / X-vtable / X-class 멤버십)**. 오늘 그 문면대로 실패하는 타이틀은 **0**이지만,
  개별 타이틀이 «왜» 그 버킷을 벗어났는지는 안 팠다(이 회차는 실행과 계수만 한다). 배너로 「멤버십은
  더 이상 유효하지 않다」만 적고 표는 보존했다.
- **`docs/reference/ezi_dispatch_reference.md`·`lgt_0x64_ordinal_table.md`**. 외부 자료 정리물이고
  이 회차의 측정 대상이 아니다.

### ★측정 중에 발견한 결함 — 고치지 «않았다»(후속 몫)

1. ★**`메이플스토리2007` 은 24번째 주입키(`#`)에서 죽는다** — `net.wie.WieError: Unimplemented:
   java/lang/Thread vtable index 13`. 기본 예산 6회 중 **1회**만 거기까지 갔다(나머지는 예산 안에 그
   스텝에 도달 못 했다). ★**그 1회가 전 회차 중 paints 가 가장 많다(121 > PASS 최대 120)** — 즉
   «퇴행해서 실패한» 것이 아니라 **가장 멀리 가서 만난 벽**이다. **렌더가 된 뒤의 벽**이라 계급이 다르고,
   report 0208 의 `Player.resume` 과 같은 형태(게임이 실제로 부른 미구현 슬롯)로 보인다.
2. ★**`일지매영웅전기` 는 다른 계급이다** — 벽 문면이 없고 `no frame rendered`. 예산을 **50초**로 3회
   (`344~427 ticks`), **113초**로 1회(`1,292 ticks`) 늘려도 **paints 0**. 같은 113초 예산에서 `놈3` 은
   `352,857 ticks · 48 paints` 로 PASS 하므로
   ★**단순 기아가 아니다.** 「매우 느린가 / 멈췄나」는 이 회차가 가르지 못했다 — 그대로 적어 둔다.
3. ★**`SD한국전쟁` 은 `ticks 0` 에서 죽는다**(다른 타이틀은 1~3). 부팅 «전» 단계의 벽일 수 있다.
4. ★**중복 6쌍(12파일)이 코퍼스를 부풀리고 있다.** 0208 이 이미 지적했고 이 회차가 sha256 으로
   전수 확인했다. 파일은 지우지 않았다(이 회차 소관 아님).

### ★otterpebble 이 인용할 표 — 「어느 sha256 이 렌더되는가」

| 타이틀(파일명) | sha256 | 판정 | 근거 |
|---|---|---|---|
| `메이플스토리2007.zip` | `13d7e3c218560814…` | ★**렌더 + 인게임 진입** | PASS 5/6 · 최대 121 paints · 512색 · 타이틀 화면 + 인트로 장면 |
| `현영맞고2006.zip` | `1b107b96bf4ed7d2…` | ★**렌더(타이틀 화면)** | PASS 6/6 · 154 paints · 512색 |
| `놈3.zip` | `b475b63996844c2b…` | ★**렌더(스플래시)** | PASS 5/7 · 69 paints · 93색 |

★**그리고 «반대 방향»도 함께 인용하라 — 이것이 차단 규칙의 핵심이다**: `lgt_compile_model()` 은
**렌더 술어가 아니다.** 같은 `broken/lgt` 의 **clet 22종**도 렌더하지 못한다. 즉 `"clet"` 통과 /
`"aot-java"` 차단은 **양방향으로 틀린다.** 차단이 필요하면 **타이틀(sha256) 단위**여야 한다.
★그 축은 별 repo·별 티켓 몫이고 이 회차는 `apps/featurephone` 을 만지지 않았다.

### 게이트

`cargo fmt --all -- --check` **OK** · `cargo clippy --all -- -D warnings` **OK**(7m17s) ·
`cargo clippy --target wasm32-unknown-unknown -- -D warnings` **OK** ·
`RUST_MIN_STACK=4194304 cargo test --all` ★**0 failed**(`test result: ok` 40줄) ·
`cargo +beta clippy --all -- -D warnings` **OK**(14m17s).

★**wasm 게이트를 문서대로만 돌리면 이 변경을 «보지 못한다» — report 0208 이 적은 그 사각이 여기서도 났다.**
맨 명령은 기본 멤버만 봐서 **3.9초**에 끝났고, 내가 고친 `wie_featurephone` 은 `#![cfg(target_arch =
"wasm32")]` 라 **네이티브 게이트에서도 통째로 컴파일되지 않는다** ⇒ 두 게이트 다 그 파일을 안 본다.
그래서 **상위집합**으로 따로 돌렸다: `cargo clippy --target wasm32-unknown-unknown -p wie_featurephone
-p wie-lgt -- -D warnings` → **OK**(3m28s · `wie_featurephone` 체크됨을 출력으로 확인).

저장소 검사기: `check-worklog-json` OK(182개) · `check-docs-report-serial` OK(중복 0 · 열린 PR 충돌 0) ·
`check-doc-liveness-parity` OK(26줄).

### ★PR CI — 전건 green (10/10 · `contract`·`coverage`·`build-web`·`rust_ci` 6레그 전부)

★★**그리고 여기서 내가 한 예측이 «틀렸다» — 적어 둔다.**
작업 중 `check-worklog-coverage` 가 로컬에서 **OVERDUE(rc=1)** 였다(`landedRounds 175` ↔
`last recorded 165`). 비율 자체는 **10/10 = 100%** 였고 밀린 것은 «기록»뿐이며, 내 수정분을 stash 한
깨끗한 base 에서도 같은 OVERDUE 가 나 **선재 상태**임을 확인했다. AGENTS.md §Landing paperwork 가
소유자를 **gate③ 회차**로 못박았으므로 이 회차는 기록하지 않았고 — ★**그러면서 「이 PR 의 CI 는 그
이유로 red 로 뜬다」고 적었다. 실제로는 green 이다.**

★**왜 틀렸나(이게 쓸모 있는 부분이다)**: 그 검사기는 **`origin/main` 을 기준으로** 기한을 잰다
(「The obligation is keyed to `origin/main`」). CI 는 매 실행마다 `git fetch --no-tags origin main` 을
먼저 돌리므로 **그때의** origin/main 을 본다. 내가 로컬에서 잰 뒤 **다른 회차가 `--record` 를 착지**시켰고
(`last recorded` 165 → **175**), CI 는 이미 해소된 상태를 봤다. ⇒ ★**내 워킹트리의 기준선이 낡았던 것**이고,
그것은 report 0208 이 연번에서 밟은 것과 **같은 형태**다(「도구는 내 트리가 낡은 것을 모른다」).
⇒ ★**교훈: 이 검사기의 OVERDUE 를 보면 «fetch 후 다시 재라». 그 rc 는 내 트리가 아니라 origin 의 상태다.**
⇒ 결과적으로 **머지 회차가 `--record` 를 번들할 필요는 없다**(그 시점에 다시 재는 것은 여전히 옳다).

### 한계 — 숨기지 않는다

- ★**전수 3회가 기준이고 PASS 후보만 6~7회다.** 15종의 FAIL 은 전부 **명시적 fatal error**(기아 신호인
  `no frame rendered` 가 아니다)라 3회로 충분하다고 봤다. `일지매영웅전기` 만 그 예외라 별도로 4회 돌렸다
  (50초 3회 + 113초 1회 = 위 표의 `+0/4 큰예산`).
  ※★**세 수는 2026-09-21 에 정정됐다** — 이 절이 「7회」·「5회」·「loadavg 72~130」으로 적어 **회신
  (`reports/wie-aot-java-render-0-premise-is-superseded-by-upstream-rebaseline.done.md`)과 어긋났고**,
  앞의 둘은 **이 리포트 자신의 전수표와도** 어긋났다(현영맞고 6/6 · 메이플 6기본+1추적 · 놈3 7 · 일지매 `+0/4`).
  판정·표·결론은 건드리지 않았다 — 고친 것은 이 세 수의 «전사»뿐이다(티켓
  `wie-lgt-docs-cite-load-dependent-trace-counts-as-absolutes` ⑷).
- ★**loadavg 47~130 에서 쟀다.** AGENTS.md 의 4단계를 따라 `놈3` 의 FAIL 2건을 「기아」로 판정했고,
  큰 예산 재실행으로 교차 확인했다. ★**그 판정이 틀렸다면 「놈3 은 렌더하지 않는다」가 되지만,
  113초 PASS 가 그것을 배제한다.**
- ★**`--inject` 의 27키 시퀀스는 «메뉴를 타는» 스크립트가 아니다.** 3종이 «어디까지» 갈 수 있는지의
  상한은 이 회차가 재지 않았다 — 잰 것은 「거기까지는 간다」이다.
- ★**엔진 코드 변경 0.** 바꾼 `.rs` 2개는 **doc 주석뿐**이다(`git diff` 로 확인 가능).
