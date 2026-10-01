## [2026-10-01] LGT 다음 벽 — UIC 830 · 410 디렉터리 목록 · «Thumb 복귀 실패»의 정체는 sprintf 넘침 (wie-lgt-next-wall-uic830-db410-thumb-return-adopt-p0)

**무엇을**
- `wie-lgt` WIPIC 표 2칸: **830 = UIC InsertText**(수락 0) · **410 = 디렉터리 이름 목록**(빈 목록 · 0).
- `wie-wipi-c` `kernel::sprintf`: `%s`·서식 바이트를 **그대로** 옮긴다(EUC-KR 디코드→재인코드 제거). `MC_knlSprintk`·`MC_knlPrintk`·LGT stdlib `sprintf`/`vsprintf` 가 이 함수 하나를 지난다.
- 시험: `wipic_svc_ids_named_from_call_sites_answer` 에 830·410 · `sprintf_passes_non_euc_kr_bytes_through` 신설.

**왜**: 채택 제안 `2026-09-30-lgt-wipic-svc-eight-unknown-ids#p0` — 0387 §3 의 후 열 세 벽.

**사용자 영향**: 세 LGT 게임이 이름 입력창(830) · 음악 목록 읽기(410) · 종목 설명 글자 만들기(sprintf)에서 꺼지던 것이 600초 장시간에서 끝까지 간다(§4).

### 1. 830 — UIC InsertText

`1cd151222bde` `0x1b38`(Thumb): `memset(this+8, 0, 256)` → `strcpy(this+8, s)` → `n = strlen(this+8)` → **`830(this[4], 0, this+8, n)`** (썽크 `0x1405320` = `.word 0x1fb, 0x33e`).
`this[4]` 는 809·811·833 이 받는 그 컴포넌트 핸들이고, 인자 모양(comp · 위치 · 글 · 길이)이 KTF `MC_uic*` 30 = InsertText 자리와 맞는다 — 0387 의 «0x320 구간 = KTF UIC 순서» 그대로다.
답: 컴포넌트가 실체가 없으니 수락(0). **831·834·835 는 함께 넣지 않았다** — 이 타이틀의 import(`.data` 썽크 전수)에 UIC 는 800–803·809·811·830·833 뿐이라 읽어 갈 칸이 없다(근거 없는 칸은 넣지 않는다).

### 2. 410 — 디렉터리 이름 목록

`87b04639cdfe` `0x15a88`: `memset(buf, 0, 0x400)` → **`410("tbl/B", buf, 0x3ff, 1)`** → `== 0` 이면 `buf` 를 NUL 로 나눈 이름 목록(빈 이름으로 끝)으로 걷는다. 각 이름이 **세 자리 숫자이고 11 보다 클 때만** 처리하고(`0x15ac0`–`0x15af8`), 이어 `0x15da4` 에서 `410("tbl/O", …)` 로 `NNN.dat` 꼴을 센다.
jar 에는 `tbl/B/001`–`011` 만 있고 `tbl/O` 는 없다 ⇒ **기본 곡은 1–11, 목록에서 찾는 것은 그 밖의 추가 곡**이다. WIPI `MC_fsList(name, buf, len, mode)` 모양이다.
답: 빈 목록(0). 이 호출부가 실제로 쓰는 것(11 초과 이름 · `.dat` 개수)은 실기에서도 추가 콘텐츠가 없으면 0 이라 결과가 같다.
천장(`ponytail:` 주석): 백엔드에 디렉터리 열거가 없어 진짜 항목을 돌려주지 않는다 — 자기 저장 파일을 목록으로 찾는 타이틀은 하나도 못 본다. 그런 호출부가 나오면 jar + DB 저장소 열거로 넓힌다.

### 3. «Thumb 복귀 실패» — 엔진 복귀 경로가 아니다. sprintf 가 스택을 넘쳤다

0387 의 `Undefined instruction at pc=0x3f8c (arm) · lr=0x2455` 를 따라갔다. 결론부터: **interworking 비트도 스레드 문맥 전환도 아니다.** 게스트가 스택에서 꺼낸 복귀 주소가 이미 망가져 있었고, 망가뜨린 것은 우리 `sprintf` 다.

측정(계측 빌드 — 커밋하지 않음):
1. **문맥 전환 없음**: 마지막 SVC 206(SetContext, `0x2450`)부터 오류까지 같은 스레드(2), 진입·복귀 사이 `pc`/`cpsr`/`lr` 기록이 끊김 없이 이어진다. 206 은 `lr=0x2455` 로 정상 복귀하고 T 비트도 그대로다.
2. **206 은 무죄**: 206 진입·복귀 때 `[sp]` 4워드가 같다(`0x1d415` …). 복귀 후 `pop {r0}; bx r0` 은 호출자 `0x1d414` 로 정확히 간다.
3. **호출자 `0x1d3xx` 의 프레임**: `sub sp, #0x104` · 글 버퍼 `sp+0x6c` — 저장된 `r4–r7, lr` 앞까지 **152바이트**. 이 함수는 끝에서 `add sp, #0x104; pop {r4–r7}; pop {r0}; bx r0` 로 돌아간다.
4. **쓴 자 = SVC 0x65(Sprintk)**: 저장 `lr` 칸(`0x400fff74`)을 SVC 경계마다 감시하니, 값이 쓰레기(`0x2ea48b`)로 바뀐 마지막 경계가 `0x1c02c` 의 **`sprintk(sp+0x6c, "%s%s%s", a, "|", b)`** 이고 반환값이 **171**(=쓴 바이트)이었다.
5. **왜 171**: 두 인자는 이 타이틀의 문자열 표(`0x5364` = `tbl->strings[i]`)에서 오고, 그 글은 **UTF-8** 이다(머리 `ec a2 85 …` · 원천은 jar 의 자원 파일 하나). 입력 19+1+50 = **88바이트**. 우리 `sprintf` 는 인자를 EUC-KR 로 풀고 결과를 EUC-KR 로 다시 감쌌다 — 유효한 EUC-KR 에는 항등이지만 UTF-8 바이트는 U+FFFD 가 되고, 그것이 재인코드에서 `&#65533;` 로 나와 **171바이트**가 됐다. 152 를 넘어 저장 `lr` 을 덮었다.
6. **증상이 셋인 이유**: 덮인 값이 짝수면 ARM 으로 풀려 `Undefined instruction`(0387), 홀수면 매핑 밖 `Invalid memory access 0x2ea48a`, 그 사이 다른 칸이 먼저 깨지면 키 처리 중 `JavaException`(외부 `jvm` 크레이트 `jvm.rs:942` unwrap). 키가 벽시계로 주입돼 실행 속도에 따라 어느 것이 먼저 나는지가 바뀐다 — 계측 빌드마다 증상이 달랐던 이유다. **세 증상 모두 같은 키(54 · NUM5)** 다.

C `sprintf` 는 `%s` 바이트를 그대로 복사한다 ⇒ 88바이트, 버퍼 안. 처방은 그것과 같게 만드는 것: 바이트를 같은 값의 char 로 실어 서식을 돌리고 그대로 되돌린다. 유효한 EUC-KR 입력에서는 이전과 바이트 단위로 같다(구성상). 바뀌는 것은 EUC-KR 이 아닌 바이트와 0x80 이상의 `%c` 뿐이고, 둘 다 이전 출력은 C 와 달랐다(`&#…;` 삽입 · `%c 0xb7` → 2바이트 `a1 a4`).

### 4. 전/후 — 같은 조건 짝

600초 · 장시간 키 900스텝(`LONG_KEYS` ×60) · `--relaunch 1` · `--jobs 3` · 조용한 호스트(`host-load-guard --status --recovered` rc=0, load 9–15). 전 = `origin/main` `2bd39b27` · 후 = 이 브랜치.

| sha12 | 전 | 후 |
|---|---|---|
| `1cd151222bde` | FAIL 25키 · `830 (r0=comp r1=0 r2=ptr r3=0 lr=0x1b79)` | 854/900키 · 600초 무오류(UNMEASURED = 시간 끝) |
| `87b04639cdfe` | FAIL 7키 · `410 (r0=0x5b538 r1=ptr r2=0x3ff r3=1 lr=0x15aa5)` | 854/900키 · 600초 무오류 |
| `8f7758fa43b6` | FAIL 54키 · `Invalid memory access 0x2ea48a` | 854/900키 · 600초 무오류 |

- `8f7758fa43b6` 의 «전»은 회차마다 흔들린다(§3-6): 300초 1회는 425키에서 시간 끝, 600초 3회 중 2회 FAIL 54 · 1회 무오류. 계측 빌드(같은 결함)는 24회 전부 54키 FAIL. «후»는 600초 무오류.
- 900스텝이 600초에 다 안 들어가 «무오류»는 `UNMEASURED`(시간 끝)다 — «그 시간 안에 오류가 없었다»로 읽는다.

### 5. 대상 밖 퇴행 — sprintf 는 모든 KTF·LGT 타이틀이 지난다

`sprintf` 는 KTF·LGT 전 타이틀이 지나므로, 바뀌는 바이트가 실제로 나오는 곳이 있는지 재었다.
계측 빌드(커밋하지 않음): 같은 호출에서 **이전 방식(EUC-KR 왕복)으로도** 결과를 만들어 새 결과와 바이트 비교하고, 다를 때만 한 줄 남긴다.

- **양성 대조**: `8f7758fa43b6` 120초 → 차이 **231회**(`old=171 new=88` 포함 — §3 의 그 값).
- **작동 목록 전수**(`game_lab/working/{ktf,lgt}` · sha 중복 제외 **215종** = KTF 170 · LGT 45): 40초 · 장시간 키 · `--relaunch 1` · 틱 상한 해제 · `--jobs 3` · 전달 키 합 **11,015** → 차이 **0종 · 0회**.
  ⇒ 지금 작동하는 타이틀의 이 구간에서 `sprintf` 출력은 한 바이트도 바뀌지 않는다.
  천장: 40초 안에 닿는 경로만 본다 — 더 깊은 곳에서 EUC-KR 이 아닌 바이트를 넘기는 타이틀은 이 측정 밖이다(그 경우 이전 출력이 `&#…;` 였으니 바뀌는 쪽이 C 와 같다).
- 830·410 행은 전에 `Unknown LGT WIPIC SVC id` 로 끝나던 자리에만 닿는다(구성상) — 그 id 를 부르지 않는 타이틀은 경로가 같다.
- 첫 회 전수(키 없이 `--inject` 기본)는 106종이 기본 틱 상한에, 20종이 첫 실행 안내 종료에 걸려 얕았다 — 버리고 위 조건으로 다시 쟀다. 그 회도 차이 0.
- 전수 중 FAIL 7종은 관찰만 했다: 차이 0 이므로 이 회차의 `sprintf` 와 무관하고, 830·410 오류도 아니다.

### 6. 되돌리면 red

- ⒜ `0x33e => Self::UicInsertText` 행 삭제 → `wipic_svc_ids_named_from_call_sites_answer` `Unknown LGT WIPIC SVC id 830`
- ⒝ `0x19a => Self::ListDirectory` 행 삭제 → 같은 시험 `… id 410`
- ⒞ `sprintf` 를 EUC-KR 왕복으로 되돌림 → `sprintf_passes_non_euc_kr_bytes_through` 실패
- 게임 층 변이 = §4 의 «전» 열.

### 7. 게이트

스크래치 target(`/tmp/wie-nw/target`) · `build-slot` 경유.
- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0(51 묶음 · 실패 0)
- 러너 블록(`cargo run` 그대로): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt` `--inject --expect-last-frame` PASS rc=0(paints 80 / 55) · `text_j2me --timeout 5` PASS.
  ※ 같은 줄을 **release** 빌드로 돌리면 `keydraw_*` 가 `UNMEASURED · max-ticks · 5/27` 이다 — `origin/main` release 빌드도 똑같이 그래서(17/11 paints) 이 회차와 무관하다. release 는 기본 틱 상한을 키 전에 다 쓴다.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus <game_lab>`: BOUNDED 15회/10쌍 · SUFFIX-ATTACHED 0회/0쌍. BOUNDED 15회는 전부 이 회차가 손댄 `wie-lgt` `svc_ids.rs`·`wipi_c.rs` 의 **기존 줄**이다 — 이 회차가 더한 줄(`git diff origin/main -- wie-lgt wie-wipi-c` 의 `+` 줄)의 한글은 시험 문자열 `"종목|"` 하나이고 그것은 걸린 이름이 아니다. 타이틀은 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=6 tree=a8263be6be040da6 B=15/10 P=0/0 S=0/0 -->
