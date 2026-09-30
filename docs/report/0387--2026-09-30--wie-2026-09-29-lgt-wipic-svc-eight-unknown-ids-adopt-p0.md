## [2026-09-30] LGT 미식별 WIPI-C SVC — 오류에 r0~r3·lr 싣기 · 호출부로 8칸 식별 (wie-2026-09-29-lgt-wipic-svc-eight-unknown-ids-adopt-p0)

**무엇을**
- `Unknown LGT WIPIC SVC id N` 오류 문구 뒤에 `(r0=… r1=… r2=… r3=… lr=…)` 를 붙인다(`wie-lgt` `handle_wipic_svc`). 접두는 그대로라 전수 군집 키가 바뀌지 않는다.
- WIPIC 표에 8칸: 602·2000(소켓 → 공유 `net::socket` = -1 «네트워크 없음») · 809·811·833(UIC Configure·SetEnable·SetMaxTextSize → 수락 스텁 0) ·
  1211·1212(미디어 Pause·Resume → 공유 스텁) · 1100(전화기 데이터 저장소 이름 목록 → 빈 목록).
- 시험 1개 `wipic_svc_ids_named_from_call_sites_answer`(8칸 응답 + 미등재 id 문구의 r0~r3·lr).

**왜**: 채택 제안 `2026-09-29-census-wave2-recensus#p0` — `3c34efee` 전수에서 LGT 8종의 첫 벽이 이 오류였고 번호가 제각각이었다.

**사용자 영향**: 순위 등록 같은 온라인 기능·이름 입력창·음악 일시정지를 부르던 순간 꺼지던 LGT 게임이 «연결 안 됨»으로 답을 받고 계속된다. 3종은 600초 장시간에서 오류 없이 끝까지 갔다(아래).

### 1. 먼저 — 오류에 레지스터를 싣는다

`WieError` 가 잡히는 시점의 레지스터 덤프(`R0: 0x0 …`)는 JVM 경계로 되감긴 뒤라 **전부 0** 이다(전 회차 `A.json` 실측). 그래서 칸마다 게임을 계측 빌드로 다시 돌려야 했다.
이제 SVC 진입 시점 값이 문구에 남는다. 첫 실측(같은 8종 · 600초 장시간 키 · `--relaunch 1`)이 바로 이것으로 나왔다:

| sha12 | 첫 벽(이 회차 재측) |
|---|---|
| `1cd151222bde` | `809 (r0=0x4843109c r1=0x5a r2=0xf8 r3=0x3c lr=0x13ad)` |
| `8f7758fa43b6` | `809 (r0=0x48430a44 r1=0x5a r2=0xf8 r3=0x3c lr=0x789d)` |
| `3ff5948e235e` | `2000 (r0=0x2 r1=0x1 r2=0x0 r3=0x140066c lr=0xed38)` |
| `5dc5d7091c01` | `2000 (r0=0x2 r1=0x1 … lr=0x3554c)` |
| `863b8ab6a21d` | `2000 (r0=0x2 r1=0x1 r2=0x2 r3=0x14008bc lr=0x1bf2b)` |
| `87b04639cdfe` | `1100 (r0=0x400fee98 r1=0xffe r2=0x1000 … lr=0x40821)` — 부팅 |
| `b7699c10dfd1` | `1212 (r0=0x4b85a200 r1=0x40447aa0 r2=0x1 r3=0x1408c38 lr=0x11f1af)` |
| `619d98bc8f64` | 재현 안 됨 — 두 회차 모두 854/900키 · 600초 무오류(키 경로에 따라 닿는다. 정적 import 에 809·2000 이 있다) |

제안 문구의 103·239·240 은 이번 경로에서 첫 벽으로 나오지 않았다. 정적 import 로는 103 은 `b7699c10dfd1`, 239·240 은 `87b04639cdfe` 가 가지고 있다 — 다음 벽 후보다(맨 아래).

### 2. 칸마다 근거 — 호출부 디스어셈블

LGT import 썽크는 `.data` 에 16바이트씩 `push {lr}; bl <resolver>; .word 0x1fb; .word <id>` 로 놓인다. 호출부는 썽크 주소를 r3(또는 r4)에 싣고 `bx` 하므로, 호출부의 리터럴을 썽크의 뒤 두 워드로 풀면 id 가 나온다(오류의 r3 가 바로 그 썽크 주소다 — `0x1408c38` → `0x4bc`).

- **809·811·833 = UIC**(`1cd151222bde` `0x1350`–`0x13d4`, `8f7758fa43b6` `0x7856`–`0x78bc` 같은 모양): `803(this[4])` → `800()` → `801("TextComponent")` → `802(ctx, cls)` → **`809(comp, 90, 248, 60, h, 3)`** → **`833(comp, n)`** → 지역 함수 → **`811(comp, 1)`**.
  클래스 이름 문자열과 순서가 KTF `MC_uic*` 표(0 CreateApplicationContext · 1 GetClass · 2 Create · 3 Destroy · 9 Configure · 11 SetEnable · 33 SetMaxTextSize)와 칸마다 맞고 인자 개수도 맞는다(Configure 6개 = comp·x·y·w·h·flag).
  ⇒ `0x320` 구간은 UIC 다. 800–803 의 기존 이름(TimeNow…)과 몸통은 **그대로 두었다** — 핸들만 돌려주고 아무도 읽지 않아 이 회차 판정과 무관하다.
  답: 이 에뮬레이터의 컴포넌트는 실체가 없으므로 배치·활성·길이 설정은 수락(0).
- **602·2000 = 소켓 생성**: `863b8ab6a21d` `0x1bf18`–`0x1bf26` 이 플래그 하나로 `2000(2, 1)` 과 `602(2, 1)` 중 하나를 부르고, 결과를 -16·-14·-13·-1 과 비교한다. `3ff5948e235e` 는 `inet_addr`(904) → `htons`(901) 직후 `2000(2, 1)` 을 부르고 `< 0` 으로 가른다.
  (2, 1) = AF_INET·SOCK_STREAM, 602 는 KTF net 2 번 `MC_netSocket` 자리. ⇒ 둘 다 공유 `net::socket`(-1). 2000 이 어떤 변형인지는 네트워크가 없는 한 답이 같다.
- **1211·1212 = Pause·Resume**: `b7699c10dfd1` `0x11f180` 한 함수가 같은 클립(`this+4`)으로 불리언 인자에 따라 1211 또는 1212 를 부르고 `== 1` 을 본다. Play 1210 · Stop 1213 사이 — KTF 순서(Play·Pause·Resume·Stop)와 같다. ⇒ 공유 `media::pause`/`resume`(KTF 와 같은 스텁).
- **1100 = 데이터 저장소 이름 목록 → 빈 목록**: `87b04639cdfe` `0x40800` — `memset(buf, 0, 0x1000)` → `1100(buf, 0xffe)`(반환값 버림) → `buf` 를 NUL 로 나눠 세고, 각 이름을 `SMSDATA`·`MMSDATA`·`CALLHISTORY`·`SCHEDULE`·`NOTICE`·`PHONEBOOK`·`PHOTO`·`ALARM`·`MORNINGCALL` 과 `strcmp` 해 **걸러낸** 나머지만 1103–1105 로 읽는다.
  전 회차(0325)는 «무엇의 목록인지 근거가 없다»로 보류했다. 이번에 거르는 이름표가 보여 «전화기 데이터 저장소 이름»까지는 좁혔다. **어떤 저장소가 있는지는 여전히 모른다** — 그래서 답은 빈 목록이고, 개수 0 이면 호출부가 1101–1105 를 건너뛴다(`0x4087e` `bgt`).
  천장: 실기에서 이 목록에 사용자 데이터가 있어야 열리는 기능은 열리지 않는다.

### 3. 전/후 — 같은 시각 짝

측정 2026-09-30 01:0x KST · load1 ~180 · 600초 · 장시간 키 900스텝(`LONG_KEYS` ×60) · `--relaunch 1`. 전 = 이 브랜치의 §1 만 있는 빌드(표 불변 = `origin/main` 과 동작 동일) · 후 = 이 브랜치.

| sha12 | 전 | 후 |
|---|---|---|
| `3ff5948e235e` | FAIL 24키 · 2000 | 854/900키 · 600초 무오류(UNMEASURED = 시간 끝) |
| `5dc5d7091c01` | FAIL 70키 · 2000 | 854/900키 · 600초 무오류 |
| `b7699c10dfd1` | FAIL 94키 · 1212 | 854/900키 · 600초 무오류 |
| `863b8ab6a21d` | 854키 무오류(이번 짝은 2000 경로에 안 닿음 · §1 회차는 157키에 2000) | 854키 무오류 |
| `1cd151222bde` | FAIL 40키 · 809 | FAIL 100키 · **다음 벽 830** `(r0=comp r1=0 r2=ptr r3=0)` |
| `8f7758fa43b6` | FAIL 39키 · 809 | FAIL 67키 · **다음 벽** `Undefined instruction at pc=0x3f8c (arm)` · lr=0x2455(206 SetContext 복귀) |
| `87b04639cdfe` | FAIL 부팅 · 1100 | 부팅 통과 · FAIL 7키 · **다음 벽 410** `(r0=0x5b538 r1=ptr r2=0x3ff r3=1)` |
| `619d98bc8f64` | 854키 무오류 | 854키 무오류 |

- 900스텝이 600초에 다 안 들어가 «무오류» 줄은 `UNMEASURED`(시간 끝)다 — 판정이 아니라 «그 시간 안에 오류가 없었다»로 읽는다.
- `8f7758fa43b6` 의 새 벽: pc `0x3f8c` 는 Thumb 함수 한가운데(`str r1,[sp,#0xc]`)인데 ARM 으로 풀렸다 — 복귀 경로에서 T 비트를 잃은 모양이고, UIC 호출(키 39 무렵)보다 28키 뒤다. 이 회차 스텁의 답과 잇는 근거는 찾지 못했다. 별 벽으로 둔다.

### 4. 되돌리면 red · 퇴행

- 변이(단위 시험 `wipic_svc_ids_named_from_call_sites_answer` · 각각 rc=101 FAILED):
  ⒜ `0x7d0 => Self::SocketAlt` 행 삭제 → `Unknown LGT WIPIC SVC id 2000` ⒝ `0x44c => Self::ListDataStores` 행 삭제 → `… id 1100`
  ⒞ 1100 몸통의 NUL 기록 삭제 → `1100 must hand back an empty list` ⒟ 오류 문구에서 r0~r3 제거 → 문구 단언(`… id 2001`) 실패.
- 게임 층 변이 = §3 의 «전» 열(같은 시각 · 표만 다른 빌드).
- 퇴행 범위: 이 회차는 **전에 fatal 이던 id 에 행을 더한 것**과 오류 문구뿐이다. 그 id 를 부르지 않는 타이틀은 경로가 바뀌지 않는다(구성상). LGT 78 sha 중 새 8칸 가운데 하나라도 import 하는 것은 60 이다(정적 스캔) — 그 60 은 전에 그 칸에서 죽었거나 아직 닿지 않았다.

### 5. 게이트

스크래치 target(`/tmp/wie-lgtsvc/target*`) · `build-slot` 경유 · load1 170–480.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0(51 묶음 · 실패 0)
- 러너 블록(이 트리 release `wie_validate`): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt` `--inject --expect-last-frame` PASS rc=0 · `text_j2me --timeout 5` PASS

### 다음 벽(정적 import · LGT 78 sha 중 가진 수 · 표 등재 전 기준)

603(60) · 614(53) · 613(51) · 410(12) · 611(10) · 1700(10) · 9500(10) · 240(9) · 239(8) · 830(8) · 1300(8). 603 은 KTF net 3 = `MC_netSocketConnect` 자리 — 소켓이 -1 이면 대개 부르지 않지만, 순서에 따라 닿을 수 있다.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 15회/10쌍 · SUFFIX-ATTACHED 0회/0쌍. BOUNDED 15회는 전부 이 회차가 손댄 파일 중 `svc_ids.rs`·`wipi_c.rs`의 **기존 줄**이다 — 이 회차가 더한 줄(`git diff origin/main -- wie-lgt` 의 `+` 줄)의 한글은 0자이고, 타이틀은 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=5 tree=3a86ec9fd212e6c6 B=15/10 P=0/0 S=0/0 -->
