## [2026-10-22] 길 끝 «원인 미상 벽» 2행 재측 — LGT 는 엔진(클립 콜백 STARTED 미전달)이라 고쳤고, KTF 는 키 타이밍이었다 (wie-ktf-8c6821fab969-lgt-87b04639cdfe-path-end-wall-remeasure)

**무엇을**: 0525 판정표 ⒜ 군집 A2 의 두 행(KTF `8c6821fab969` · LGT `87b04639cdfe`)을 현 main `6bbbbf94` 에서 600초 짝으로 다시 쟀다. 두 벽 모두 남아 있었다. 원인을 실측으로 찾았다.
- `87b04639cdfe`: **엔진 벽**이다. WIPI-C `MC_mdaPlay` 가 `MC_mdaClipCreate` 로 받은 클립 콜백을 한 번도 부르지 않았다. 이 게임은 그 콜백이 `MC_MDA_STATUS_STARTED`(2)를 받을 때만 곡 시작 시각을 적는다. 그래서 음악은 나오는데 음표가 하나도 안 내려왔다. `MC_mdaPlay` 가 성공하면 콜백에 STARTED 를 알리도록 고쳤다(시험 1개).
- `8c6821fab969`: **엔진 벽이 아니다.** 옥상 조작 안내 두 개가 각각 정해진 키 타이밍을 요구한다. 레시피가 그 타이밍을 못 냈다. 레시피를 고치자 안내를 넘어 600초 끝까지 진행한다.

**왜**: 운영자 채택 지시(2026-10-22 08:35 · 제안 `2026-10-22-progression-stuck-triage-wave7#p1`). 두 행의 원인 칸이 비어 있었다. 판정은 10-08 핀이었고 그 뒤 엔진 커밋이 40개 넘게 들어왔다.

**사용자 영향**: 이 게임(LGT 리듬 게임)에서 곡을 고르면 음표가 내려오고 판정과 점수가 붙는다. 전에는 음악만 나오고 음표가 하나도 없었다. 두 행의 진도 배지가 «자동으론 막힘»에서 «계속 진행»으로 바뀌고, 지원 현황 상세에 진행 요령이 한 줄씩 생긴다. 같은 엔진 길(WIPI-C `MC_mdaPlay` + 클립 콜백)을 쓰는 다른 96종은 20초 짝에서 판정이 바뀌지 않았다(§5).

### 1. 재측 — 현 핀에서 둘 다 남아 있었다

엔진: `origin/main` `6bbbbf94` 의 release `wie_validate`. `census-drive.sh` 로 `--only probe` → `--only long --titles` → `--only progress --progress 600 --as P` → `--as P2`(정책 v2 · `--jobs 2` · 레시피 = 비커밋 `game_lab/recipes-progress/<sha12>.keys`). A·B·L 도 이 핀에서 새로 쟀다.

| sha12 | P 정체(초) · 새 화면 | P2 정체(초) | census | 화면(마지막 1/3) |
|---|---|---|---|---|
| `87b04639cdfe` | 210 · 13 | 220 | `stuck` | t110~t600 연주 화면. 키를 누르면 줄이 켜지지만 음표가 하나도 안 내려온다. 점수 0 |
| `8c6821fab969` | 30 · 31 | 140 | `ok` | t230~t400 옥상 안내 «점프 후, 상 키를 누르면 앞뒤 흔들기(swing)를 합니다» 그대로. 정책 탈출이 메뉴·스테이지 처음으로 되돌린다. t520 에 같은 안내로 돌아온다. P2 는 t400~t560 같은 안내 뒤 «게임 문의» 화면 |

- `8c6821fab969` 의 census `ok` 는 0502 와 같은 함정이다. 정책 탈출이 스테이지를 다시 시작해 새 화면이 생겼을 뿐 앞으로 가지 않았다.

### 2. `87b04639cdfe` — 클립 콜백이 STARTED 를 못 받았다(엔진)

**관찰**(`RUST_LOG=debug` 180초 · 이 핀): t130 에 `MC_mdaClipCreate(0x140044c, 0x5f46, 0x14335)` → `MC_mdaClipPutData` → `MC_mdaPlay` 로 곡이 시작된다(MIDI 이벤트 22210). 그 뒤 연주 화면의 프레임마다 부르는 것은 `MC_knlCurrentTime` 2회 · 그리기 · `MC_knlSetTimer` 뿐이다. 매체(`MC_mda*`) 호출은 없다.

**원인**(binary.mod 디스어셈블 · capstone Thumb): 세 번째 인자 `0x14335` 가 클립 콜백 `cb(clip, status)` 이다.
- `0x14340` `bl 0x1414c` — 클립으로 자기 곡 객체를 찾는다.
- `0x14348` `cmp r6, #2` → `bl 0x58634`(시각 읽기) → `str r0,[r5,#0x14]` · `str r1,[r5,#0x18]` — **status 2(STARTED)일 때만 곡 시작 시각을 적는다.**
- status 1(END_OF_DATA) · 3(STOPPED) · 9 이면 지금 − 시작으로 경과를 `+0x1c` 에 적는다.
- 끝에 `bl 0x142a4(obj, status)` 로 게임 쪽에 넘긴다.

엔진의 `clip_create` 는 `callback` 을 로그에만 찍고 버렸다(`wie-wipi-c/src/api/media.rs`). 그래서 시작 시각이 끝내 0 이었다. 음표는 그 시각을 기준으로 내려오므로 하나도 화면에 들어오지 않았다.

**고침**: `MdaClip` 끝의 내부 필드(SDK 밖 · 기존 `handle`/`volume`/`loaded` 옆)에 `callback` 을 둔다. `MC_mdaPlay` 가 `audio().play` 에 성공하면 `spawn` 으로 `cb(clip, 2)` 를 부른다. 이벤트는 `MC_mdaPlay` 가 돌아온 **뒤에** 간다. 같은 크레이트의 `MC_netConnect`/`MC_netSocketConnect` 콜백과 같은 길이다. 콜백이 0 인 클립은 아무것도 받지 않는다.
- 시험: `media::tests::play_tells_the_clip_callback_it_started_test` — 콜백 없는 클립은 spawn 0 · 있는 클립은 `MC_mdaPlay` 안에서는 호출 0 · spawn 을 돌리면 `(0x14335, [clip, 2])` 한 번.
- **하지 않은 것**: END_OF_DATA(1) · STOPPED(3)는 보내지 않는다. 곡 길이(`Audio::duration`)로 끝 시각을 셀 수는 있다. 하지만 이 게임은 음표표로 곡 끝을 정하고, 이번에 막은 벽은 STARTED 하나였다. 필요한 게임이 나오면 그때 붙인다.

**효과**(이 브랜치 `7c840765` · 같은 레시피):
- 240초 탐색: 음표가 내려오고 판정(«MISS»·«GOOD»)이 뜬다. 점수 11 · 최대 콤보 1 이다. 레시피 키가 박자에 안 맞아 «GAME OVER» 가 난다. 확인 키로 곡 선택으로 돌아가 다음 곡이 시작된다.
- 600초 짝: §4.

### 3. `8c6821fab969` — 옥상 안내 둘의 키 타이밍(레시피)

이 게임의 «도움말 → 조작법»(이 핀에서 화면으로 읽음): 이동 = 방향키·2/8/4/6 · 확인·공격 = OK/5 · «(상/2) 클릭 = 거미줄 공중이동» · «공중 (상/2) 더블클릭 = 거미줄 이동» · «(상/2) 더블클릭 = 수퍼점프» · 거미줄 발사 = OK/5.

탐색 420초(옛 레시피로 옥상까지 간 뒤 네 가지 UP 패턴을 6번씩 · 키마다 화면):

| 패턴(`이름:GAP:HOLD`) | 결과 |
|---|---|
| `UP:0.4:0.1 UP:2.5:0.1` (톡 · 0.4초 뒤 톡) | 6번 모두 안내로 돌아옴 |
| `UP:0.25:0.08 UP:0.25:0.08` (빠른 두 번) | 6번 모두 안내로 돌아옴 |
| `UP:0.7:0.1 UP:0.3:0.6` (톡 · 0.7초 뒤 **0.6초 누름**) | 2번째에 안내가 사라지고 다음 옥상으로 넘어감 |

- 화면 판정: 안내 상자(176×220 화면 y 145~185)가 검은 칸 60% 이상이면 «안내»로 셌다. 첫 UP(점프)마다 안내가 잠깐 사라졌다가 착지하면 돌아온다. 세 번째 패턴 뒤로는 돌아오지 않았다.
- 그다음 안내가 하나 더 있다: «점프 후, 상 키를 더블클릭 시, 천장으로 (주인공을) 당겨 올려 줍니다». 360초 탐색에서 흔들기만 넣은 레시피는 t140~t350 그 안내에 멈췄다. `UP:0.6:0.1 UP:0.2:0.08 UP:1.5:0.08`(점프 · 0.6초 뒤 0.2초 간격 두 번)을 순환에 넣자 넘었다.
- 새 레시피(비커밋 `game_lab/recipes-progress/8c6821fab969.keys` · 옛 것은 증적 폴더에 `8c6821fab969.0502.keys`): 첫 줄(«새로시작»·난이도·대화 OK)은 그대로다. 순환 줄은 오른쪽 이동·OK·NUM5 에 위 두 패턴을 더했다.
- 엔진을 의심할 근거는 없었다. 같은 핀에서 키 타이밍만 바꿔 넘었다. HOLD 0.6초 키는 `wie_validate` 가 Keydown 과 Keyup 사이를 비워 두는 것으로 충분했다(0525 §2⑵ 의 Keyrepeat 가설과 같은 결론).

### 4. 고친 뒤 600초 짝

빌드: 이 브랜치 `7c840765`(release). A·B·L 은 §1 의 `6bbbbf94` 측정을 옮겨 썼다(두 빌드의 차이는 `MC_mdaPlay` 뒤 콜백 하나다). 레시피: `87b04639cdfe` 는 §1 과 같다. `8c6821fab969` 는 §3 의 새 레시피다. 정책 v2 · `--jobs 2`.

| sha12 | 전 P · P2 정체(초) | 후 P · P2 정체(초) | 새 화면(P) | 정책 탈출(P) | census | 마지막 1/3(두 판 다 눈으로) |
|---|---|---|---|---|---|---|
| `87b04639cdfe` | 210 · 220 | **0 · 0** | 13 → 45 | 8 → 0 | `stuck` → `ok` | 곡 → 음표·판정 → «GAME OVER» → 곡 선택 → 다음 곡이 t600 재시작까지 되풀이된다. 레시피의 숫자 키가 박자에 안 맞아 곡마다 진다 |
| `8c6821fab969` | 30 · 140 | **0 · 10** | 31 → 41 | 2 → 0 | `ok` → `ok` | 총 든 적 · 특수 공격 안내 · «지지대에서 이탈하려면 하 키를 두번» 안내를 지나 t600 재시작까지 싸우며 나아간다 |

⇒ 공개 값 계약 §1 의 예외(레시피 짝 둘 다 `ok` + 두 판 마지막 1/3 을 사람이 놀이로 확인 + 같은 행의 `playTips_ko`)를 채웠다. `compat.json` 두 행의 `progress` 를 `stuck` → `ok` 로 바꾸고 요령을 한 줄씩 실었다. 같은 문장을 `playability-census.mjs` `HAND_TIP` 에도 넣었다(다음 `report` 가 지우지 않도록). `check-compat-revert`: 착지 기준 바뀐 행 2.
- `87b04639cdfe`: «두 키를 함께 누르는 시험에서 «미지원폰» 창이 나오면 오른쪽 방향키로 «아니오»를 고른 뒤 확인 키를 누르고, 곡을 고르면 내려오는 음표의 번호에 맞춰 숫자 키를 누르면 돼요.»
- `8c6821fab969`: ««새로시작»으로 들어가 옥상 안내가 나오면 위 방향키로 뛰어오른 뒤 공중에서 위 방향키를 길게 누르면 흔들기, 빠르게 두 번 누르면 천장으로 올라가며 다음으로 넘어가요.»
- `87b04639cdfe` 의 이용자 소식은 `docs/player-updates/2026-10-22-rhythm-notes-fall-with-the-song.json` 이다.

### 5. 콜백을 받게 된 다른 게임 — 회귀 확인

바뀐 길은 WIPI-C `MC_mdaPlay` 하나다(KTF `method_table.rs` · LGT `wipi_c.rs` 가 같은 함수를 쓴다). J2ME `SmafPlayer` 와 SKVM `WieAudioClip` 은 이 함수를 지나지 않는다.

**누가 콜백을 받게 됐나**: `compat.json` 에서 KTF·LGT 이고 `sound: ok` 인 340종을 이 브랜치 빌드로 20초씩 돌렸다(`--inject` 기본 27키 · `RUST_LOG=wie_wipi_c::api::media=debug` · 동시 ≤ 3). 콜백이 0 이 아닌 `MC_mdaClipCreate` 를 부른 행이 **96종**(KTF 61 · LGT 35)이다. 20초 안에 소리를 안 내는 게임은 이 표본에서 빠진다.

**짝**: 그 96종을 `6bbbbf94` 빌드로 같은 조건에서 다시 돌렸다.

| 전 → 후 | 수 |
|---|---|
| PASS → PASS | 87 |
| UNMEASURED → UNMEASURED | 5 |
| PASS → UNMEASURED(`clean exit`) | 2 (`0baceb1dfe9c` `e159b9727dbd`) |
| UNMEASURED(`clean exit`) → PASS | 1 (`d5e996a53118`) |
| FAIL → PASS | 1 (`f770b15f8876`) |

- 판정이 바뀐 4종을 두 빌드로 3번씩 다시 돌렸다(동시 3). **4종 모두 두 빌드가 3/3 같은 판정이다**: 앞 3종은 6판 모두 PASS(deadline)다. `f770b15f8876` 은 두 빌드 모두 3/3 FAIL(error)이다 — 이 회차 전부터 있던 것이다. 한 판짜리 뒤집힘은 재현되지 않았다.
- paints 는 한 판끼리 0.29~4.35배로 흔들렸다(평균 비 1.24). 두 빌드를 다른 시각에 돌렸고 그동안 load1 이 70~110 이었다. 이 수는 판정에 쓰지 않았다.
- ⇒ 콜백을 받게 된 96종에서 이 변경이 만든 판정 변화는 0 이다.

### 6. 게이트

- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(경고는 `wie_cli` 의 «unused dependency» 9줄 · 이 회차와 무관한 기존 것).
- `RUST_MIN_STACK=4194304 cargo test --all` **785 passed / 0 failed**(0525 의 784 + 이 회차 시험 1).
- 러너 블록(AGENTS.md §Definition of Done): `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS · `keydraw_ktf.zip` PASS rc=0(paints 79) · `keydraw_lgt.zip` PASS rc=0(paints 55) · `text_j2me.jar` PASS.
- 프로세스: 에뮬레이터는 동시 3 이하로 돌렸다. census 는 `census-drive.sh`(long 임대 · phase 마다)로 돌렸다. 340종 스캔은 85종 묶음마다 long 임대 1건이다. 단발 탐색은 `build-slot run`(short)이다. `nohup` 은 0 이다. 회차 끝에 내 `wv_*` 프로세스는 0 이다. 이 회차 동안 load1 은 14~110 이었다.
- 유입(`node scripts/corpus-name-inflow.mjs`): 유입 330건(BOUNDED 718회/330쌍) + 판단 필요 17건(SUFFIX-ATTACHED 37회/17쌍). 이 수는 파일 전체를 센 것이다. BOUNDED 는 모두 `compat.json` 의 기존 `title`/`fileTitle` 필드다(그 파일은 설계상 모든 제목을 담는다). SUFFIX-ATTACHED 는 그 파일의 «더 긴 다른 제목»(…2 · …3 · …1.04)과 `playability-census.mjs` 의 0507 줄 «턴종료» 다. 이 회차가 더한 줄(요령 2줄 · 회차 문서 · 이용자 소식 · worklog)의 BOUNDED 와 SUFFIX-ATTACHED 는 0 이다. 게임 화면의 안내 문구를 옮길 때 주인공 이름은 «(주인공)»으로 바꿨다.
- 증적(화면 · 키 스크립트 · out · 스캔 표)은 저장소 밖 `~/scratch/a2wall/` 에 있다. 레시피는 비커밋 `game_lab/recipes-progress/` 다.


