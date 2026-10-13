## [2026-10-13] playable 인데 소리 없음 11종 — 엔진 3곳(상대 이름 자원 · 멈출 것 없는 `Player.stop` · WIPI-C `VOLUMELEVEL`) · 동봉 설정 «끔» 5 · 측정 정정 1 · 이유 1 (wie-playable-sound-no-eleven-unexplained)

**무엇을**: `compat.json` 에서 `playable` 인데 `sound: no` 이고 이유 없이 «소리가 나지 않을 수 있어요.» 만 적힌 11행을 행마다 갈랐다. 엔진은 세 군데를 고쳤다.
- `wie-jvm-support` `hardening.rs`(`Class.getResourceAsStream`): 이름이 `/` 로 시작하지 않으면 클래스의 패키지 기준으로도 찾는다. 원래 이름으로 못 찾았을 때만 한다.
- `wie-wipi-java` `Player.stop(Clip)`: 재생 중(`STARTED`)이던 클립을 멈췄을 때만 `true` 다. 멈출 것이 없으면 `false`.
- `wie-wipi-c` `MC_knlGetSystemProperty`: `VOLUMELEVEL` 에 `"3"` 을 답한다(Java `HandsetProperty` 와 같은 값).

`compat.json` 은 11행을 고쳤다. 소리 `no → ok` 5행, 동봉 설정 안내로 교체 5행, 이유 문장으로 교체 1행이다. 이용자 소식은 3건이다. 이제 `playable` 행 가운데 «소리가 나지 않을 수 있어요.» 가 남은 행은 0 이다.

**왜**: 운영자 지시(2026-09-30 · 10-13): 엔진 등 기능 완성도. 원인 미상 문장 0 이 목표다.

**사용자 영향**: 5종에서 소리가 난다(엔진 4 · 측정 정정 1). 5종은 «게임 파일에 소리를 끈 설정이 있어 게임 안에서 켜면 들린다»고 안내한다. 1종은 왜 소리가 안 나는지 정확한 이유를 적는다. 11종 밖에서도 `40b9537968de`(이미 `ok`)의 30초 재생이 3 → 10 으로 늘었다(4-3).

### 1. 방법

- 대상: 티켓의 11행(GUI 총괄 실측 · `origin/main` compat).
- 빌드: 전 = `origin/main` `d7059f1f` release `wie_validate` · 후 = 이 브랜치.
- 매체 호출 기록: 120초 · 키 = 타이틀 레시피(`game_lab/recipes-*` · 비커밋) + 전수 장시간 키 고리 12회. `RUST_LOG` 로 `MC_mda*` · `org.kwis.msp.media.*` · `com.skt.m.Audio*` · `net.wie.*` 를 셌다.
- 원인 추적(진단 빌드 · 커밋 0):
  - `Class.getResourceAsStream` 이 받은 이름과 결과를 찍는 빌드.
  - KTF 자바 메서드 프록시가 받은 guest `lr` 을 찍는 빌드(어느 코드가 `Player.stop` 을 불렀나).
  - `binary_patches.toml` 임시 항목(`IsSoundOn` → 1 · `PlayMMF` 진입 함정).
  - `wie-ktf-dump` 로 푼 이미지를 capstone 으로 읽었다(`uv run --with capstone`).
- 동봉 파일 한 바이트 바꾸기: 로컬 사본(비커밋)을 원본과 같은 빌드로 나란히(동시 2) 120초.
- 퇴행 짝(§5): census probe A(기본 27키 · 30초 · `--pacing 8`)를 전·후 빌드로.
- 측정 규율: 에뮬레이터는 전부 `build-slot run` 안 · 동시 최대 3. 스윕은 phase(80건 ≈ 14분)마다 `build-slot run --long` 임대 1건. 시작 시 `host-load-guard --status --recovered` rc=0. census `run`·호스트 락 미사용 · `nohup` 0 · 회차 끝 내가 띄운 에뮬레이터 0.

### 2. 11행 표

계급:
- ⒜ 동봉 설정 «끔» — 게임 파일에 원 주인의 소리 끔 설정이 있다. 그 바이트를 바꾸면 난다.
- ⒝ 재생 호출 0 — 들어갈 수 있는 화면에서 게임이 소리를 틀지 않는다.
- ⒞ 엔진 결손 — 고쳤다.
- ⒠ 측정 정정 — 이미 난다.

«전·후» = 재생 수 · MIDI 사건 수(같은 키 · 같은 시간 · 동시 짝).

| sha12 | 통신사 | 계급 | 근거 | 전 | 후 |
|---|---|---|---|---|---|
| `a7d1f26af3e1` | KTF | ⒞ 상대 이름 자원 | 게임의 소리 클래스(`ghostcity/Sound`)가 `"title.mmf"` 처럼 패키지 기준 이름으로 묻는다. 파일은 `ghostcity/*.mmf` 다. 엔진이 jar 루트에서 찾아 null 을 주니 `PlayMMF` 가 클립을 만들기 전에 돌아갔다 | 0 | 3 · 466(probe A) · 4 · 679(레시피 100초) |
| `51011b242bb5` | KTF | ⒞ 상대 이름 자원 | `sook/Utils/CSound` 가 `"sound/ting.mmf"` 로 묻는다. 파일은 `sook/Utils/sound/` 다 | 0 | 3 · 2,265(probe A) · 17 · 2,265(120초) |
| `888702965551` | KTF | ⒞ `Player.stop` | 효과음마다 먼저 `Player.stop(clip)` 을 부르고 그 값이 `false` 일 때만 재생한다(이미지 `0x10630e`~`0x106316`). 엔진은 플레이어만 있으면 `true` 였다 | 0 | 2 · 4,223(probe A) |
| `1cd151222bde` | LGT | ⒞ `VOLUMELEVEL` | 소리 전에 `MC_knlGetSystemProperty("VOLUMELEVEL")` 을 묻는다(30초에 5회). 엔진은 `M_E_INVALID` 였다 | 0 | 1 · 26(probe A) · 1 · 26(레시피 120초) |
| `c107462e5f8a` | SKT | ⒠ | 진도 레시피 120초에서 재생 8 · MIDI 5,776(현 main). 0440 의 «90초 안에 호출부에 닿지 않는다»는 그 회차 레시피 길이의 한계였다 | 0 | 8 · 5,776 |
| `fb80e97cbc57` | SKT | ⒜ | 동봉 `ds2.opt` 첫 바이트 = 소리 모드 `f.C`, 원 주인 0. 소리 요청 `h.a(IB)V` 가 `f.C == 0` 이면 바로 돌아간다(javap). 파일이 없으면 기본 1. 옵션 화면이 `f.C` 를 쓴다 | 0 | 0 · 바이트 1 이면 31 · 114,378 |
| `ccb45e6b8d80` | SKT | ⒜ | 동봉 `sc.txt` 의 `bp`(33번째 값) = 소리 켬, 원 주인 0. `e.a(II)V` 는 `bp == 1` 일 때만 클립을 연다(javap · 필드 추적에서 시작 0.1초 뒤 `bp` 1 → 0). 환경설정 `u(int)` 가 `bp` 를 바꾼다 | 0 | 0 · 값 1 이면 5 · 3,685 |
| `5a59f62d1f1a` | KTF | ⒜ | 동봉 `config.dat` 첫 바이트 = 소리 크기, 원 주인 0. 게임이 그 값으로 `MC_mdaSetVolume(0)` 을 부른다. 3 이면 재생. 처음 메뉴에 OPTION | 0 | 0 · 3 이면 68 · 8,321 |
| `b907b0faf483` | KTF | ⒜ | 동봉 `config.sav` 둘째 바이트 = 소리 켬, 원 주인 0. 1 이면 재생. 첫·넷째·셋째 바이트는 무관(0 그대로). 처음 메뉴에 OPTION · 이미지에 `VolumeUp`/`SetMute` | 0 | 0 · 1 이면 50 · 130 |
| `689c491586b9` | KTF | ⒝ | 시작하자마자 서버 접속 실패 안내 뒤 스스로 끝난다(5키 · `clean exit`). 그때까지 클립 생성조차 0 — 매체 호출 기록 0(전·후 같음). 이미지에는 `Clip`·`Player.play` 가 있다 | 0 | 0 |
| `6c9f969f089f` | KTF | ⒜ | 동봉 `P/personal_data` 의 둘째 int(오프셋 0x04) = 소리 끔 플래그, 원 주인 1. 0 이면 재생. 이미지에 `PD__SOUND_OFF` · `SetMute` · `IDS_SOUND` · `op_options`, 처음 메뉴에 «환경 설정» (§3) | 0 | 0 · 0 이면 44 · 1,012 |

**합**: ⒞ 4 · ⒠ 1 · ⒜ 5 · ⒝ 1 = 11.

### 3. `6c9f969f089f` — 클립은 7개 만들고 재생은 0

- 120초 기록: `Clip` 생성·`setBuffer` 7회 · `Player.play`·`Player.stop` 0회(`lr` 기록 빌드에서도 `stop`/`play` 진입 0).
- 이미지 이름표에 `PD__SOUND_OFF`(personal_data 의 칸 번호 상수) · `SetMute(Z)` · `Play(IZ)` · `MAX_VOLUME` · `IDS_SOUND` 가 있다.
- 0440 은 이 파일의 int 몇 개를 0·3 으로 바꿔 보고 재생 0 이었다. 이번에는 값이 1 인 칸 다섯(0x04 · 0x08 · 0x18 · 0x30 · 0x34)을 하나씩 0 으로 바꿔 120초씩 쟀다: **0x04 만 재생 44**, 나머지는 0(0x30 은 그림 35장으로 멈춤). `scratchpad` 를 빼도 0.
- ⇒ 원 주인이 끈 소리 설정이다. 파일을 통째로 빼면 화면이 멈추므로(0440) 목록으로 뺄 수 없고, 안내 문장으로 처분했다.

### 4. 고친 엔진 결손

#### 4-1. 상대 이름 자원(`wie-jvm-support` `ResourceFromMidletJar`)

- `Class.getResourceAsStream(name)` 은 `/` 로 시작하지 않는 이름을 클래스 패키지 기준으로 푼다(자바 규약). 핀(`dlunch/RustJava@5b84dd1`)의 `Class.getResourceAsStream` 은 이름을 그대로 로더에 넘긴다.
- 진단 빌드 기록: `a7d1f26af3e1` 50초에 `"story.mmf"` 181 · `"title.mmf"` 1 · `"menu_sel.mmf"` 1 요청, 전부 null(시스템 로더 대체도 null). `51011b242bb5` 60초에 `"sound/*.mmf"` 8종 · 44회, 전부 null.
- `PlayMMF`(`a7d1f26af3e1` · 이미지 `0x106630`): `this` 필드 검사 → 경로로 자원 열기 → null 이면 반환. 함정 패치로 첫 키에서 `PlayMMF` 가 불리는 것을 확인했다. `IsSoundOn` 을 1 로 고정한 판은 재생 0 그대로였다(소리 설정은 켜짐 · 환경설정 화면 «on»).
- 고침: 원래 이름 → 시스템 로더(종전 그대로) → 둘 다 null 이고 이름이 상대이면 `패키지/이름` 으로 같은 두 길을 한 번 더.
  - ★종전에 찾던 자원은 같은 길로 찾는다. 바뀌는 것은 «종전에 null 이던 요청»뿐이다.
- 되돌리면 red: `a_relative_resource_name_is_found_beside_its_class`. 상대 분기를 끄면 `left: None`(실측).

#### 4-2. 멈출 것 없는 `Player.stop`(`wie-wipi-java` `Player::stop_clip`)

- `888702965551` 은 이름이 지워진 AOT 이미지다. 진단 빌드가 `Player.stop` 의 guest `lr` 을 찍었다: `0x106313`(2회) · `0x1063b5`.
- `0x10630e` 의 `stop` 호출 뒤 `lsls r0, #24 · cmp r0, #0 · beq` — `false` 일 때만 클립 설정 쪽으로 간다. `true` 면 상태 `0xa4` 를 두고 빠진다.
- 이 게임은 단말에서 소리가 났다 ⇒ 그 단말에서 쉬는 클립의 `stop` 은 `false` 였다.
- 고침: `getState() == STARTED` 일 때만 `true`(`resume_clip` 과 같은 판정). `pause` 도 이 함수를 쓴다.
- 기존 시험 `test_clip_resume_delegates_to_backing_player` 는 한 번도 재생하지 않은 클립의 `stop` 이 `true` 라고 단언하고 있었다. 그 단언만 `false` 로 바꿨다(그 시험의 목적인 `resume` 위임은 그대로).
- 되돌리면 red: `test_stopping_an_idle_clip_reports_false`(`true ||` 로 바꾸면 «a clip that never played has nothing to stop» · 실측).

#### 4-3. WIPI-C `VOLUMELEVEL`(`wie-wipi-c` `get_system_property`)

- `1cd151222bde`(LGT 네이티브) 30초 로그: `unknown system property id: VOLUMELEVEL` 5회 · `VIBRATORLEVEL` 4회.
- Java `HandsetProperty` 는 이미 `"3"` 을 답한다(eceeb889). 같은 단말이니 같은 값을 쓴다. `VIBRATORLEVEL` 은 소리가 아니라 그대로 둔다.
- 되돌리면 red: `test_get_system_property_volume_level`(줄을 빼면 rc ≠ 0 · 실측).
- 전·후 짝: 문자열 `VOLUMELEVEL` 을 네이티브 바이너리에 가진 18종(정적 검사) × probe A. 16종 같음 · `1cd151222bde` 0 → 1 · `40b9537968de` 3 → 10(MIDI 742 → 2,500 · 이미 `ok`). 18종 모두 후 빌드에서 `unknown system property id: VOLUMELEVEL` 0회.

### 5. 퇴행

census probe A(30초) 짝. 전 = `origin/main` · 후 = 이 브랜치(4-3 전 판 · 4-3 은 위 18종 짝).

- **짝 207종**: 문자열 검사로 `org/kwis/msp/media/Player`+`stop`(클래스) 또는 `(…Clip;)Z+stop`(AOT 이미지)을 가진 195종(KTF 193 · SKT 2) ∪ 라이브 LGT 5 ∪ 가드 2 ∪ 11종.
  - 판정(result · stop · 소리 들림 여부)이 다른 것: **3 — 전부 고친 타이틀**(`a7d1f26af3e1` `51011b242bb5` `888702965551` 이 0 → 재생).
  - 재생·MIDI 수만 다른 것 18종. 7종을 전·후 2회씩 다시 쟀다: 전 빌드끼리도 같은 폭으로 흔들린다(`0a6f495f0b35` 전 19·14·14 / 후 15·14·14 · `1d18b7cc06d1` MIDI 전 10,224·6,199·6,199 / 후 6,199·6,199·10,224).
    - `1b107b96bf4e`(라이브 LGT) 전 16·14·13 / 후 12·11·11. 이 타이틀은 `Player.stop` 문자열이 없고, 상대 이름 분기 도달 0 · `VOLUMELEVEL` 0 — 바뀐 코드가 닿지 않는다. 0440 도 이 타이틀에서 ±1~2 를 봤다.
    - `704839f1a6c8` 전 2·1·2 / 후 1·1·1 — 매체 debug 짝에서 호출 순서(`stop`·`stop`·`play`)가 양쪽 같고 둘 다 재생 2. 둘째 재생이 30초 끝에 걸리는 시각 차다.
- **나머지 222종**: 후 빌드만 돌렸다(상대 이름 분기가 닿는지 보려고). PASS 212 · `clean exit` 5 · FAIL 5. FAIL 은 `MC_knlGet…` 미구현(`974e0df9ab1e` `f770b15f8876` · `limited` 설치 인증 벽) 등 이 회차 코드가 닿지 않는 것이다.
- **상대 이름 분기 도달**: 429종 후 빌드 로그에서 `also tries` 가 찍힌 것은 `a7d1f26af3e1` · `51011b242bb5` 둘뿐이다.
- 라이브 LGT 5 · 가드 2: 결과·재생 수 같음(`1b107b96bf4e` 은 위). 그림 수 `49ade89578c5` 2,141 → 2,059 · `ddd885583b15` 1,603 → 1,588 · 나머지 ±1 — 두 가드는 바뀐 코드가 닿지 않는다(`Player.stop` 문자열 없음 · 분기 도달 0 · `VOLUMELEVEL` 없음). 배틀몬스터 사운드폰트 경로(`wie-backend` 오디오 · 브라우저 워크렛)는 손대지 않았다.
- 4 게이트: `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 · wasm32 rc 0 · `+beta` rc 0(기존 unused-dependency 경고 9줄) · `RUST_MIN_STACK=4194304 cargo test --all` **782 passed / 0 failed**.
- `npm run build:wasm` rc 0 · `check-engine-contract` 137 pass / 0 violation.

### 6. 지원 현황(compat.json) — 방법

- `origin/main` 판 위에서 행 단위로 고쳤다(`JSON.stringify(_, null, 1)` + 줄바꿈 — 손대지 않은 행은 바이트 그대로). 다른 축은 손대지 않았다.
  - `sound no → ok` + «소리가 나지 않을 수 있어요.» 제거: `a7d1f26af3e1` `51011b242bb5` `888702965551` `1cd151222bde`(⒞) · `c107462e5f8a`(⒠).
  - 동봉 설정 안내로 교체: `fb80e97cbc57` `ccb45e6b8d80` `5a59f62d1f1a` `b907b0faf483` `6c9f969f089f` — «게임 파일에 소리를 끈 설정이 들어 있어요. 게임 안 설정에서 소리를 켜면 들려요.»(0440 C 와 같은 문장 · `sound` 는 `no` 그대로).
  - 이유 문장으로 교체: `689c491586b9` — «여기서 볼 수 있는 처음 화면과 안내 화면에서는 게임이 소리를 틀지 않아요. 그 뒤 화면은 아래 서버 문제로 들어갈 수 없어요.»(서버 벽 문장은 그대로 둔다).
- 이용자 소식: 엔진 고침 3건(4-1 · 4-2 · 4-3). 측정 정정(`c107462e5f8a`)은 소식을 내지 않았다(0440 선례).
- 갱신 방법: 위 행만 손으로 고쳤다(전수 재생성 아님). `node scripts/check-compat-revert.mjs` OK.
- KTF ⒜ 2종(`5a59f62d1f1a` · `b907b0faf483`)은 0440 B 처럼 `OWNER_SOUND_OFF_SETTINGS` 로 동봉 파일을 빼면 처음부터 소리가 난다(`b907b0faf483` 파일을 빼면 재생 50 · 실측). 티켓 ⒜ 처분(안내 문장)을 따랐고 목록 추가는 하지 않았다.
