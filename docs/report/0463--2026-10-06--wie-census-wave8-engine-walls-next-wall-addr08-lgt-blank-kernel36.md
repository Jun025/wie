## [2026-10-06] 8차 엔진 벽 — KTF paint 스레드 경합 · KTF 그래픽 컨텍스트 = 워드 64B · 커널 36 = 이름으로 표 찾기 · LGT 구매 휴대폰 잠금 (wie-census-wave8-engine-walls-next-wall-addr08-lgt-blank-kernel36)

**무엇을**: `docs/report/0438` §000-4 후속 표의 주인 없는 엔진 벽 4행(5종)을 보았다. 고친 것 셋, 판정 하나.
- KTF `paint()` 가 도는 동안 다른 게스트 스레드를 붙든다(0430 의 키 처리 장치를 paint 에도 · KTF 에도).
- KTF SDK 이미지의 그래픽 컨텍스트를 «모든 필드가 워드인 64바이트»로 읽고 쓴다(종전 52바이트 packed).
- KTF 커널 36 = 이름으로 메서드 표를 돌려준다. `"MXUserMemInterf"` 표(영역 할당기)를 넣었다.
- LGT `01f05f8231f4` = 구매 휴대폰 잠금(인증 우회 금지 · census 판정 표에 손으로 등재).

**왜**: 운영자 지시(2026-09-30 · 10-06 「이어서 필요한 후속 작업들을 완전자율주행으로」). 0438 §000-4 의 «1 · 2 · 1 · 1» 행에 주인 티켓이 없었다.

**사용자 영향**: §6 표. 타이틀은 sha12 로만 적는다.

### 1. `5267badf20b3`(KTF) — 그림 1~2장에서 정지 = paint 와 게임 스레드의 경합

- 증상: 부팅 뒤 그림 1~2장 · 무음 · 게스트 스레드 하나가 NPE 로 죽는다.
- 원인(스크래치 계측 — 분기 PC 추적 · 필드 쓰기 감시 · 커밋 0):
  1. 게임 스레드(`run()` · `0x104426`)는 정적 필드 A 가 −1 이 될 때까지 `Thread.yield` 로 돈다. 그다음 `this.B.method(…)` 를 부른다.
  2. A 와 B 는 `paint()`(`0x10a1a6` · `0x10a1ba`)가 이 순서로 쓴다: A = −1, B = 객체.
  3. wie 는 게스트 스레드를 1만 명령마다 자른다. paint 스레드가 **두 쓰기 사이**(B 의 필드 참조를 푸는 `0x1389c0`)에서 예산으로 잘렸다(`svc=false` · 매 실행 같은 자리).
  4. 그 틈에 게임 스레드가 A = −1 을 보고 B(아직 0)를 불러 `java/lang/NullPointerException` · `run()` 의 catch 로 스레드가 끝난다.
- 확인: 예산을 1만 → 10,007 로만 바꾼 스크래치 빌드에서 같은 타이틀이 그림 2,507 · 소리 9회 재생 · 예외 0 으로 끝까지 돈다.
- 고친 것: 0430 이 LGT 키 처리에 넣은 «붙들기»(`hold_others`)를 `Display` 의 paint 호출에도 쓴다. paint 의 게스트 코드는 다음 호스트 호출(sleep·wait·I/O)까지 한 덩어리로 돈다. 상한 `HOLD_ROUNDS` 는 같다. KTF `TaskRunner` 가 그 둘(`others_preempted`·`hold_others`)을 구현한다.
  - 대가: KTF 의 키 처리에도 0430 의 ⑴⑵가 켜진다(종전 LGT 만).
  - `hold_others` 는 «이미 붙들고 있었나»를 돌려준다. 키 처리 안에서 불린 paint 가 바깥 붙들기를 풀지 않게 하려는 것이다.
- 되돌리면 red: `paint_runs_with_the_other_guest_threads_held`(paint 붙들기 제거 → FAILED) · `a_nested_hold_reports_the_outer_one`.

### 2. KTF 그래픽 컨텍스트 — 게임이 직접 쓴 자리와 우리가 읽는 자리가 달랐다

- `3151fdc167b6` 의 SetContext 래퍼(`0x103818`)는 인덱스 3·4·5·10 을 **직접** 쓰고 나머지만 `MC_grpSetContext` 로 부른다.
  - TRANS → +0x1c · ALPHA → +0x20(과 +0x30) · PIXELOP → +0x2c · OFFSET → +0x24/+0x28.
  - 이것은 «clip·offset 까지 모든 필드가 워드»인 64바이트 배치다. 종전 wie 는 `wipi_types` 의 52바이트 packed(clip·offset 이 u16)로 읽었다.
- 그 결과:
  - 게임의 ALPHA 쓰기가 우리 쪽 `pixel_op_func_ptr`(+0x20)에 앉는다. 0447 이 KTF 블릿에서 그 포인터를 부르게 된 뒤로 블릿이 `0x7d00f81f` 로 점프했다(`3151` · 다음 벽).
  - 같은 회사(Com2uS)의 `7da00ecd4804` 는 `0x3000f81f` 로 점프한다. 이 회차 main 빌드 프로브 A **12/12 FAIL**(키 6~8 · 같은 주소). 0438 의 «주소 0/8» 과 다른 서명이다.
  - TRANS 를 +0x14 에서 읽으니 게임의 키 색(0xf81f)이 보이지 않는다.
- 고친 것:
  - `wie-wipi-c` 의 컨텍스트 읽기·쓰기 11곳을 `read_grp_ctx`/`write_grp_ctx` 로 모았다. `WIPICContext::wide_graphics_context()` 가 참이면 64바이트 워드 배치로 옮긴다(기본 거짓 · LGT·시험 그대로).
  - KTF 는 이미지가 KTF SDK 런타임일 때만 참이다. 판별 = 이미지에 `WIPICX_incMemInterface` 문자열이 있는가(SDK 런타임이 부팅 때 이 이름으로 인터페이스를 묻는다).
  - 저장소의 `keydraw_ktf` 픽스처(`wipi` 크레이트 빌드)는 packed 를 쓰고 그 문자열이 없다. 처음 판(KTF 전부 워드)은 그 픽스처를 깼다(`test_key_reach` — 64바이트 InitContext 가 52바이트 스택 구조를 넘어 썼다). 그래서 판별을 넣었다.
  - 코퍼스 KTF 266종 중 문자열 있음 239 · 없음 27(재배치 형식·DRM·옛 SDK). 컨텍스트를 쓰는 71종 중 문자열 없는 것 3종(`2b1ed0c8d061` `4166acd8fc62` `444821c513ac`)은 종전 배치 그대로다.
- 되돌리면 red: `a_wide_context_is_read_where_the_title_wrote_it`(넓은 배치 무시 → FAILED) · 판별 제거 → `test_key_reach`·`test_resource_reach`(KTF) 실패.

### 3. `3151fdc167b6`(KTF) — 커널 36

- 0389·0178 실측 그대로: `f("MXUserMemInterf", -1, -1, 0, 0)` → 표 포인터 → 곧바로 slot 0.
- 새 근거:
  - SDK 런타임이 `InitParam4.fn_get_interface` 로 자기 인터페이스를 같은 모양 `(이름, -1, -1, 0, 0)` 으로 묻는다(`5a59f62d1f1a` `0x10dc8e`) ⇒ 슬롯 36 = 이름으로 표를 찾는 서비스.
  - 표의 쓰임(래퍼 `0x1034a8`~`0x1035dc` · 호출 22+13곳): slot 0 `(영역, 크기)` — 정적 버퍼 0x14400 · 0x28800 바이트. slot 1 `(영역, n)` → 포인터(호출부가 n 바이트를 곧바로 복사 · null 검사 없음). 거의 모든 slot 1 앞에 같은 영역의 slot 0 이 다시 온다. slot 3 `(영역, p)` 래퍼는 있으나 부르는 곳이 없다. slot 2 는 읽는 곳이 없다.
- 고친 것: 슬롯 36 은 이름을 보고 표를 준다. `"MXUserMemInterf"` = 영역 위의 bump 할당기(slot 0·1). slot 2·3 과 다른 이름은 이름 붙은 `Unimplemented` 로 멈춘다(0 을 주면 null 호출이다).
  - 커서를 영역 앞 8바이트에 둔다(slot 1 은 영역만 받는다). 실기가 어디에 두었는지는 모른다.
- 되돌리면 red: `user_mem_arena_bumps_inside_its_region_and_starts_over_on_init`. 타이틀로는 슬롯 36 스텁 복원 → 부팅 `Unimplemented: 36`.

### 4. `01f05f8231f4`(LGT) 그림 0 — «스텁 대기»가 아니라 구매 휴대폰 잠금

- 0438 의 «wipi_c unk1 · `MC_imHandleInput` 스텁 대기»는 틀렸다. unk1(0xa600)은 paint 가 자기에게 보내는 이벤트이고, 받는 쪽은 아무것도 하지 않는다.
- 원인:
  1. 장면 관리자의 «멈춤» 바이트(+0x2b)가 서 있어 타이머가 시작되지 않는다.
  2. 그 바이트를 세우는 함수(`0x39e8`)는 오류 메시지 상자다. 오류 −1001 · 문구 「문의전화: 1588-4263」.
  3. 부른 곳은 저장 로더(`0x69ccc`)다. 그 로더의 유일한 호출자가 «소유자 확인 = 1»을 넘긴다. 로더는 저장 파일의 12바이트를 `PHONENUMBER` 와 비교한다(`game_*.sav` 7개 전부 이 로더를 지난다).
  4. 동봉 저장을 빼고 실행하면 부팅·그림 138~144·소리가 난다. 그러나 곧 「인증에 실패하였습니다 … 고객센터로 문의해 주십시오」를 그리고 스스로 끝난다(키 12/27 · clean exit). 이번에는 `P/audio.adt`(60바이트 · 복호화된 12바이트)를 같은 `PHONENUMBER` 와 비교한다.
- 판정: 원래 구매자 번호에 묶인 라이선스다. 운영자 정책 2026-10-02(번호 복원·검사 우회 금지)대로 **풀지 않는다**. 저장을 빼는 스크래치 실험은 커밋하지 않았다(인증 화면으로 바뀔 뿐 플레이는 안 된다).
- 처분: census 의 `phone` 잠금 판정(SKT `SecureUtil` 정적 규칙)이 이 LGT 네이티브 검사를 못 본다. `LOCK_HAND` 에 sha12 로 손 등재하고, compat 행 안내를 「구매한 휴대폰에서만 켜지도록 잠긴 파일…」로 바꿨다(§7).

### 5. 장시간 «주소 0/8»(`7da00ecd4804` `5a59f62d1f1a`) — 재현 조건

- 과거 census 기록(디스크의 `reports-*` · 증적 없이 JSON 만 읽었다):
  - `7da00ecd4804`: 프로브 키 6·11(2회)·14·20 에서 주소 0/8.
  - `5a59f62d1f1a`: 프로브 키 7·18(3회)·19, 장시간 61·73·107 에서 주소 0.
  - 키 위치가 흩어져 있다 ⇒ 키 시점 의존이다.
- 이 핀 main(`544a0643`)에서 `7da00ecd4804` 의 첫 벽은 §2 의 점프다(프로브 A 12/12 · 키 6~8). 그 벽이 «주소 0/8» 을 가리고 있었다.
- §2 를 고친 head 에서:
  - `7da00ecd4804` 장시간 L **2/3 FAIL**(키 24·37 `NUM5` · 주소 8) · 1/3 은 600초 생존.
    - 스크래치 진단 빌드로 같은 키를 다시 돌려 한 번 잡았다: `pc 0x13a544` · `str r3,[r2]` · r2 = 8. 직전에 읽은 리소스는 `0x20.fid`(jar 에 있다 · 45KB).
    - 그 함수(`0x13a080`)는 코스 로더다. `**(표[i]) + 8` 에 물체 좌표를 쓴다. 표[0] 의 핸들이 0 이라 0 → 읽기 0(널 읽기 정책) → +8 쓰기다.
    - 같은 키로 한 번 더 돌린 진단 실행은 키 168개·120초 동안 나지 않았다.
  - `5a59f62d1f1a` 장시간 L **0/3 FAIL**(600초 3회 생존 · 마젠타 FAIL 도 없어졌다). 이 회차는 주소 0 을 재현하지 못했다.
- 판정:
  - 재현 조건 = 장시간 키 루프 · 코스를 여는 경로(`0x20.fid`) · 키 시점.
  - 원인 = 핸들 표 0번 칸이 비는 것까지는 쟀다. 왜 비는지(할당 실패·해제 뒤 사용·우리 할당기 차이)는 못 쟀다. 쓰기를 삼키지 않았다.
  - 크기 M — 후속 표 1행(§9).

### 6. 측정 — census 형식 직접 실행

- 빌드: base = main `544a0643` ↔ head = 이 브랜치(§1~4 · 병합 전). release `wie_validate`.
- 인자는 census `run` 과 같다(A·B 30초 · L 600초 · `--max-ticks 100000000000` · `--relaunch 1`). 결과는 census 형식 디렉터리로 쓰고 `playability-census.mjs report` 가 판정했다.
- ★이탈 1건: census 호스트 잠금을 다른 레인의 600초 진도 census 가 80분 넘게 쥐었고, 둘이 더 줄 서 있었다. 그래서 도구 대신 같은 인자로 직접 돌렸다(0438 §5 선례).
  - 스윕 전체 = `build-slot run --long` 한 임대 · 동시 ≤3 · 배치마다 `host-load-guard --status --recovered` rc=0 대기 · 회차 밖 프로세스 0.
  - 시각·load1: main 집합 base 16:58→17:42(19→17) · head 18:03→19:19(12→14) · 컨텍스트 집합 19:21→20:26(10→11).

#### 6-1. 대상·라이브·가드 12종(A/B/L · `report` 판정)

| sha12 | base | head | 상태 |
|---|---|---|---|
| `5267badf20b3` | A 그림 1 · 소리 0 · L 그림 1 | A 그림 4,065 · 소리 9 · L 600초 키 814 뒤 스스로 종료 · 그림 21,231 · 소리 616 | limited → **playable** |
| `3151fdc167b6` | A·B·L 부팅 `Unimplemented: 36` | A 27/27 그림 961 · L 600초 생존 · 소리 338 | not-yet → **playable** |
| `5a59f62d1f1a` | L FAIL «magenta color-key not applied (16%)» | L 600초 생존 3/3 · 색 253 → 316 | limited → **playable**(소리 없음 그대로) |
| `7da00ecd4804` | A FAIL 키 6 · B FAIL 재기동 · L FAIL 키 5(§2 점프) | A·B PASS · L 2/3 FAIL 주소 8(§5) | limited 그대로 |
| `01f05f8231f4` | 그림 0 | 그림 0 | not-yet 그대로 · 안내 문구만(§4) |
| 라이브 LGT `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684` · 가드 `49ade89578c5` `ddd885583b15` | 7종 playable | 7종 playable · 축 동일 | 퇴행 0 |

- 라이브·가드의 그림 수 차는 ±7% 이내다(`49ade89578c5` A 1,313 → 1,981 만 크다 · 장시간은 21,150 ↔ 19,677).

#### 6-2. 그래픽 컨텍스트를 쓰는 KTF 71종(A/B)

- 모집단: KTF 266종을 head 로 15초씩 돌려 `MC_grpInitContext`/`SetContext`/`GetContext` 로그가 나온 것(로그는 세기만 하고 저장하지 않았다).
- base ↔ head 의 (result·stop·입력 단계·content) 또는 boot/render/input/sound 축이 다른 것 6종:
  - 좋아짐 2: `3151fdc167b6` `7da00ecd4804`(6-1).
  - `974e0df9ab1e`: 양쪽 FAIL `Unimplemented`(키 27 ↔ 17) — 같은 벽이다.
  - `f2280c6699a0`: head 1회 FAIL(키 10 · 쓰레기 주소). 짝 재측: base 4/6 FAIL · head(진단 빌드) 4/6 FAIL · 그 전에 3+3 양쪽 PASS ⇒ main 에도 있는 간헐 벽이다.
  - 소리 축만 갈린 3종(`a20c2044305c` ok→silent · `5028b8a5d19f` silent→ok · `7da00ecd4804` ok→silent): 각각 재생 0↔1 회의 차다. `a20c2044305c` 짝 3+3 은 양쪽 모두 0회 ⇒ 흔들림.
- 컨텍스트를 쓰지만 SDK 문자열이 없는 3종(`2b1ed0c8d061` `4166acd8fc62` `444821c513ac`)은 이 측정의 head 빌드(판별 전 · KTF 전부 워드)로 쟀다. 착지하는 코드는 이 셋을 종전 배치로 둔다.

#### 6-3. 병합 뒤(origin/main `9c8d9729` 병합 · 21:05→21:36 · load1 38→32)

| sha12 | 2회 | 판정 |
|---|---|---|
| `5a59f62d1f1a` | A PASS 2/2 · L 600초 생존 2/2 | 장시간 생존 누계 5/5 |
| `7da00ecd4804` | A 1/2 FAIL(키 15 `NUM5` · 주소 8) · L 600초 생존 2/2 | §5 의 벽이 프로브에서도 난다 · 누계 L 2/5 FAIL + A 1/2 FAIL |

### 7. compat · 소식

- compat 4행만: `5267badf20b3` `3151fdc167b6` `5a59f62d1f1a` = `report` 판정을 `scripts/player-data.mjs` 의 `fromCensus` 로 옮겨 `status`·`axes`·`knownIssues_ko` 만 덮었다(다른 필드와 `progress` 는 그대로). `01f05f8231f4` = `knownIssues_ko` 1줄만(§4 · `LOCK_HAND`).
- `7da00ecd4804` 는 바꾸지 않았다(상태 같음 · 소리 축은 재생 0↔1 회의 흔들림).
- 지원 현황 401/14/14 → 403/14/12 · 위 3종 · `fb80e97cbc57` 등 병합분은 main 의 값 그대로다.
- 소식 3: `2026-10-06-paint-thread-order.json`(`5267`) · `2026-10-06-ktf-drawing-settings.json`(`7da0` `5a59`) · `2026-10-06-ktf-memory-interface.json`(`3151`).

### 8. 게이트(build-slot · 병합 뒤 트리)

- fmt rc=0 · clippy `-D warnings` stable/wasm32/beta rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` **721/0**(`CARGO_INCREMENTAL=0`).
- 러너 픽스처: draw_j2me · helloworld 2 · text_j2me PASS · keydraw 2 PASS(`--max-ticks 100000000000` — release 빌드는 5천만 틱을 키 5개 안에 다 쓴다 · base 도 같다).
- `npm run build:wasm` rc=0 · `check-engine-contract` 113/0 · `npm run audit` PASS · census selftest 62/62 · worklog OK · 연번 · 유입 표식.

### 9. 후속 표

| 수 | 벽 | 계급 | 크기 |
|---|---|---|---|
| 1 | `7da00ecd4804` 코스 로드 «주소 8» — 핸들 표 0번 칸이 비는 경로(§5) | 엔진 | M |
| 1 | `5a59f62d1f1a` 소리 없음 | 엔진 | 미측정 |
| 3 | 컨텍스트를 쓰는데 SDK 문자열이 없는 KTF 3종의 배치 확인(§6-2) | 측정 | S |
| 1 | `01f05f8231f4` | 정책(풀지 않음) | — |
