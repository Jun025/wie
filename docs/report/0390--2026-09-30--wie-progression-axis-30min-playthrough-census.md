## [2026-09-30] 진도 축 신설 — «죽지 않음»이 아니라 «계속 앞으로 가나»를 잰다 · 막힘 원인 엔진 수정 4건 (wie-progression-axis-30min-playthrough-census)

**무엇을**: `playability-census.mjs` 에 `--only progress` 단계(진도 축)를 넣고, 플레이 가능 판정 350종 가운데 102종을 진행 정책 키로 10~30분 돌렸다. 막힌 화면을 직접 보고 ⒜정책 부족 ⒝엔진 탓 ⒞게임 한계(+⒟측정 한계)로 갈랐고, ⒝ 중 넷을 고쳤다.
**왜**: 운영자 지시(2026-09-30) — 「플레이 가능한 게임도 수십 분 직접 플레이해 계속 진도를 나갈 수 있는지 검토하고, 부족하면 조치하라」. 기존 `longplay` 축은 «10분 키 주입 동안 죽지 않았다»만 잰다. 메뉴를 맴돌거나 조용히 멈춰도 `ok` 였다.
**사용자 영향**: 네 가지가 고쳐졌다. ① 텍스트 파일을 한 번에 읽는 게임이 메뉴에서 «게임 시작»을 눌러도 시작되지 않던 문제(라이브 등재 LGT `1b107b96bf4e` 포함). ② 이름 입력칸에 글자가 들어가지 않아 입력 화면에서 영원히 멈추던 문제. ③ 타이틀 화면에서 더 넘어가지 않던 문제. ④ LGT 두 종의 게임 스레드가 죽던 첫 벽. 셸이 보여 주는 등급(`status`)은 바꾸지 않았다(§6).

증적: 프레임 PNG·전사는 저장소 밖(`~/scratch/wie-progress-2026-09-30/`)에만 있다. 이 문서는 sha12 만 적는다. 게임 이름·바이트는 0 이다.

### 1. 진도 축 — 무엇을 재나

| 항목 | 값 |
|---|---|
| 입력 | 진행 정책 키 ~30초 한 주기: 확인(OK·5·1·왼쪽 소프트키 1회)으로 안내·메뉴를 넘기고, 방향키 누르기·길게 누르기, 5 연타. **CLR·오른쪽 소프트키는 쓰지 않는다**(대부분 «뒤로/종료») · 종료하면 3회까지 다시 켠다(DB 유지) |
| 표본 | 10초마다 한 장(`--shot-every 10`) |
| 새 화면 | 16×16 칸 평균 휘도 지문. 지금까지 본 **모든** 화면과 8칸 이상이 32단계 넘게 다르면 새 화면 |
| 판정 | 마지막 새 화면 이후 남은 시간(`stall`)이 실행 시간의 1/3 이상(하한 180초)이면 `stuck` · FAIL 줄이면 `error` · 그 밖 `ok` |
| 확정 | `stuck` 은 **같은 바이너리로 짝 재측(`--as P2`)** 해 둘 다 `stuck` 일 때만 확정. P2 가 없으면 `n/a` — 추측하지 않는다 |
| 레시피 | 제목별 키 파일을 **머리말**로 붙이고 그 뒤에 정책을 잇는다(`--titles` 셋째 열) |

**왜 PNG 해시가 아니라 휘도 지문인가 — 첫 판이 틀렸다.** 첫 판은 PNG sha256 으로 «새 화면»을 셌다. 30분 대표 실행에서 두 종이 거짓 `ok` 로 나왔다. 빈 슬롯 선택창의 커서 빛(`739c7657c1f2`, 해시 27종)과 쿠폰 입력칸의 글자 순환(`13d7e3c21856`, 해시 76종)이다. 둘 다 30분 내내 같은 화면이었다. 지문은 문턱을 두 벌 대조해 골랐다. 24/3 은 슬롯 선택창을 여전히 통과시켰다. 32/8 은 두 종과 얼어붙은 메뉴(`1b107b96bf4e`)를 `stuck` 으로 잡았다. 실제로 진행하는 두 종(1300초·430초까지 새 화면)은 `ok` 로 남겼다.
**알고 남긴 한계**(코드 `ponytail:` 주석): 커서만 판 위를 도는 보드 게임은 `ok` 로 읽힌다(체스 한 종 관측). 반대로 한 화면 안에서 계속 플레이하는 게임(횡스크롤 한 구간, 테니스 연습)은 `stuck` 으로 읽힐 수 있다 — §3 의 ⒟.

**가상 시간이 아니다.** `wie_validate` 는 벽시계로 돈다. 10분 = 실시간 10분이고, 부하가 크면 게임이 느려져 «정체»처럼 보인다. 그래서 짝 재측을 판정 조건으로 넣었다. 호스트 부하(load1)는 P 9~190, P2 {{P2LOAD}} 이었다.

### 2. 표본과 결과

전수(350종 × 30분 = 5 jobs 로 약 35시간)는 이 회차 시간(8시간) 안에 들어가지 않는다. 그래서 1차를 이렇게 끊었다.
- **대표 8종 × 30분**: 라이브 LGT 5 `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e`(배틀몬스터) `b475b6399684` · 가드 2 `49ade89578c5` `ddd885583b15` · KTF `739c7657c1f2`.
- **정체 후보 84종 × 10분**: 기존 전수(`bd2337ff`)의 longplay 스크린샷(20초 간격 30장)에서 끝 5장 이상이 같았거나 도중 종료한 것.
- **대조군 10종 × 10분**: 나머지 playable 258종에서 정렬 순서로 26번째마다.

{{RESULT}}

**대조군이 말하는 것**: 대조군 10종 중 7종이 `stuck` 이었다. longplay 선별이 정체를 잘 가려내지 못했고, 범용 정책으로 10분 돌리면 **대부분 어딘가에서 멈춘다**. 막힌 곳은 대개 ⒜ — 이 표는 «엔진이 막는다»가 아니라 «범용 키로는 여기까지 간다»로 읽어야 한다. 전수의 진도 `stuck` 비율을 이 102종으로 외삽하지 마라(선별 편향).

### 3. 분류 — 막힌 프레임을 직접 봤다

{{CLASSES}}

- **⒜ 정책 부족 — 가장 큰 군집은 «뒤로 가기 없음»**: 정보·도움말·빈 «이어하기» 슬롯·«No Data» 불러오기 화면에 들어가면 CLR 없이는 나올 수 없다(`a30bbe008b5e` `b475b6399684` `739c7657c1f2` `41466fc7f709` `4df05a4dc452` `fba094100d13` `c6cadf75c454` …). 둘째는 숫자키가 메뉴 항목을 직접 고르는 게임이다. `1b107b96bf4e` 는 6이 «게임종료»였다. 셋째는 메뉴·대화상자 순환이다.
- **⒞ 게임 한계 — 통신**: «추가 다운로드» 안내 뒤 종료(`0e72b6bc12bb` `5aa438fbdc87` `75f002ba70e3` `f7752f9124e8`), «통신 에러»·«CONNECTING»·«네트워크 접속에 실패».
- **⒟ 측정 한계**: 한 장면 안에서 계속 플레이한다(횡스크롤 한 구간, 러너, 퍼즐, 테니스 연습). 16×16 지문으로는 새 화면이 아니다.
- **?(미조사)**: 흰/검은 화면에서 그리기를 멈춘 것들. 전사에 오류가 남지 않았다.

### 4. ⒝ 고친 것 — 전/후

| 원인 | 고친 곳 | 걸린 타이틀(관측) | 전 → 후 |
|---|---|---|---|
| `InputStreamReader.read(char[],0,759)` 가 **6자**를 돌려준다(10바이트 조각 하나 디코드 후 `break`). EUC-KR 글자가 조각 경계에서 갈리면 U+FFFD 로 깨진다 | `wie-jvm-support/src/hardening.rs` — 핀의 메서드 본문을 바꿔 끼운다(JDK `StreamDecoder` 규칙: 뭔가 읽었고 더 읽으면 막히면 멈춘다 · 진짜 미완성 글자만 남긴다) | `1b107b96bf4e`(LGT) `f12d97040c33`(KTF) `9a2cf5ffc9d3`(SKT) | 메뉴 «게임시작»에서 키마다 `StringIndexOutOfBounds` → 튜토리얼·게임 진입 · 고스톱 판 진행 · 이야기·게임 화면 진행(게임 스레드 NPE 사라짐) |
| `InputMethodHandler.notifyKeyInput` 가 키를 받지 않고 리스너도 부르지 않는 stub | `wie-wipi-java/.../lcdui/input_method_handler.rs` — 멀티탭(ITU E.161)·CLR 삭제·숫자 제한자, `notifyTextChanged(chars,len,pMode)`(javadoc) · `getCurrentInputMode` · `changeCurrentModeToNext` | `1e43e2e0055f` `d448aee68157`(KTF) · `09a6a300994d` `0865be217bde` 같은 형태 | 매장 이름 입력 «최소 1자» 무한 반복 → 게임 본편(2007년 1월 지도) · `d448` 은 벽 두 개를 넘어 다음 벽 `lwc.Component` 필드 `x` |
| `org.kwis.msp.lcdui.Display.callSerially(Runnable,int)` 가 Runnable 을 버리는 stub | `wie-wipi-java/.../lcdui/display.rs` — `timeout` ms 뒤 이벤트 큐에 넣는다(한정 없는 판으로 넘김 → `run()` 은 이벤트 스레드) | `5d3ba49eccf7`(KTF) | 10분 동안 타이틀 3장 → 메뉴·이야기·로딩·던전 플레이 |
| LGT Java ABI 에 `Stack` 35 · `Timer` 16 이 없다 → `Unimplemented … vtable index` 로 게임 스레드 사망 | `wie-lgt/data/lgt_java_abi.toml` — CLDC 1.1 선언 순서로 `empty()Z`=35 · `cancel()V`=16(**호출부 역어셈 안 함 · 순서 유도**) | `73f3a21e981c` `be08d047cbae`(LGT) | 화면상 진전 없음. 다음 벽으로 이동: `PrintStream` 27 · 게스트 `TimerTask` 하위 11 |

{{AFTER}}

**넣었다가 뺀 것 하나**: `TimerTask.cancel()Z`=11 행을 넣으면 `be08d047cbae` 가 둘째 키에서 **호스트 panic**(`Expected object, got Int`)으로 바뀌었다. 행을 빼고 그 벽은 남겼다. 순서 유도만으로 넣지 않을 행의 실례다.

**«되돌리면 red»**: 네 수정 모두 단위 시험이 있다. 수정을 빼면 각각 실패한다(실측).
- `reader_read_tests::one_read_fills_the_buffer_from_a_resource_stream` — 핀 본문이면 540 대신 8을 돌려준다.
- `input_method_handler::tests::keys_reach_the_listener_as_text`
- `display::tests::timed_call_serially_queues_the_runnable_after_the_timeout` — stub 이면 큐가 늘지 않는다.
- ABI 두 행: `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 목록에 추가했다(빈 슬롯이면 panic).

### 5. 대표 타이틀 심층 플레이

{{REPS}}

**60분은 채우지 못했다.** 대표 8종은 정책으로 30분(P), 확정 재측으로 30분(P2)을 돌렸다. 레시피가 있는 두 종(배틀몬스터 `docs/keys/battlemonster-village.keys` · `739c7657c1f2` `docs/keys/ktf-739c7657c1f2-town.keys`)은 레시피 + 정책으로 30분을 수정 바이너리로 돌렸다. 스테이지 2 도달과 «세이브 후 이어하기»는 **이 도구로 판정하지 않았다**. 화면 지문은 «어느 스테이지인가»를 모른다. 세이브 쓰기 횟수도 아직 `wie_validate` 가 내지 않는다(§8).

### 6. compat.json · 셸

- `axes.progress` 는 **선택 축**으로 넣었다(`scripts/player-data.mjs` 의 `EXTRA_AXES`). 잰 행에만 싣는다. 값은 `stuck→no`, `ok→ok`, `n/a→unknown` 이다. 없는 키는 «재지 않음»이다.
- 셸(otterpebble `apps/featurephone`)은 **바꿀 것이 없다**(origin/main 실측). 가져오기(`scripts/compat-import.mjs`)는 여섯 축만 검사하고 나머지 키는 `...x` 로 그대로 넘긴다. 화면(`app/compat.tsx`)은 `AXIS_LABEL` 고정 목록만 그린다. `Axis` 타입(`lib/compat.ts`)도 여섯이라 `progress` 는 보이지 않는다.
- `status` 규칙은 **바꾸지 않았다**. 제안(운영자 문안 결정): 확정 `stuck` 을 `limited` 로 내리지 **말 것**. 확정 `stuck` 의 다수가 ⒜(범용 정책의 한계)와 ⒟(측정 한계)다. 게임이 멈춘 것이 아니다. 내린다면 ⒝·⒞ 로 분류된 것만, 그리고 `knownIssues_ko` 문장과 함께 내려야 한다.
- `docs/player-data/compat.json` 갱신 방법: 이 회차에 잰 {{COMPATN}}행에만 `axes.progress` 를 넣었다. 값은 이 브랜치 머리(수정 포함)로 잰 F 가 있으면 F, 없으면 P/P2(`f44c6bcd` 기준 · 수정 전)다. 나머지 행·`enginePin`·`status` 는 그대로다.

### 7. 퇴행 확인

- 라이브 LGT 5 + 가드 2 + `739c7657c1f2` 를 30초 A 프로브(전수와 같은 인자)로 다시 돌렸다. `wv-fix`·최종 바이너리 모두 **8/8 PASS → PASS**, `content` 참이다. 그림 수는 load1 ~70~130 에서 오르내린다.
- 러너 블록(AGENTS.md): `draw_j2me` `helloworld_ktf` `helloworld_lgt` `text_j2me` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 릴리스 바이너리에서 `UNMEASURED · stop max-ticks` 였다. 판정 절차 ⑴~⑶을 따랐다. `--max-ticks 1000000000` 이면 **수정 전(`wv-main`)·후 모두** PASS, 27/27 키, 55장, 마지막 화면 내용 있음이다. 기본 5천만 틱을 빠른 릴리스 빌드가 먼저 태운 것이고, 이 diff 와 무관하다.
- 네 관문: `cargo fmt --check` · `clippy --all -D warnings` · `clippy --target wasm32-unknown-unknown -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` · `cargo +beta clippy --all -D warnings` 모두 rc=0 이다.

### 8. 후속(남긴 군집)

| 군집 | 수 | 계급 | 크기 | 근거 |
|---|---|---|---|---|
| «Data 인스톨에 실패» 뒤 종료 — KTF DB `stream_write` 17회(1.7MB) → `close` → `MC_dbListRecords(fd, buf, 1)` 에서 실패로 판정 | 2 (`1cf2e6076079` `5028b8a5d19f`) | ⒝ 추정 | M | 두 종 모두 census 에서 `playable` 이지만 실제로는 매 부팅 3초 안에 종료한다 — **등급 거짓 양성** |
| KTF `java_check_type(…, unk≠0)` 가 항상 참 → 게임 스레드 AIOOBE/NPE 의심 | 2 (`01e2715ba07a` `9789fec50f39`) | ⒝ 의심 | L(역어셈 필요) | `interface.rs` 의 `// TODO is it correct?` |
| SKVM `com/xce/lcdui/TextComponent` 없음 | 1 (`85f03ca7389e`) | ⒝ | M | `NoClassDefFoundError` · 게임 스레드 사망 |
| LGT ABI `PrintStream` 27 · 게스트 `TimerTask` 11 | 2 (`73f3a21e981c` `be08d047cbae`) | ⒝ | S~M | 호출부 역어셈으로 행 확정 필요(순서 유도만으로 넣은 11 은 panic) |
| 입력기 다음 벽 `lwc.Component` 필드 `x` | 1 (`d448aee68157`) | ⒝ | S | `Field xI@182 not found` |
| 진행 정책 v2 — 정체 뒤에만 CLR 1회 | ⒜ 36 중 다수 | 도구 | M | 고정 키 스크립트로는 «막혔을 때만 뒤로»를 못 쓴다 |
| 세이브 쓰기 횟수·RMS 를 진도 축에 | – | 도구 | S | 과제 문안의 측정 항목 중 이번에 안 낸 것 |
