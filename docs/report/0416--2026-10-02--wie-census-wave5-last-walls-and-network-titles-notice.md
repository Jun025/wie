## [2026-10-02] 5차 — «통신망» 3종 중 2종은 엔진 벽 · KTF `p/` 탑재 · 동봉 DB 설치 · MIDP `showNotify` · 통신망 안내 규칙 (wie-census-wave5-last-walls-and-network-titles-notice)

**무엇을**
- `wie-ktf`: 아카이브의 개인 폴더를 `P/` 뿐 아니라 `p/` 로 써도 루트에 탑재한다(`private_path`).
- `wie-wipi-java` `org.kwis.msp.db.DataBase.openDataBase`: 아카이브가 단말 형식(`<name>.idx` `qtpdb` 머리 + `<name>.db` 레코드)으로 채워 둔 DB 를, 저장된 것이 없을 때 처음 열면 설치한다.
- `wie-midp`: `Canvas.showNotify()`·`hideNotify()`(빈 기본 구현)를 선언하고, `Display.setCurrent` 가 Canvas 가 바뀔 때 부른다.
- 전수 도구: **통신망 벽** 판별(`netWall`) — 한 실행에서 접속을 20회 이상 되풀이하면 `knownIssues_ko` 를 안내 한 줄로 바꾸고 `status` 는 최대 `limited`. 계약 문서에 그 예외 한 줄.

**왜**: 4차 회신 §후속 · 총괄 정책 판정(2026-10-02 «통신망 다운로드가 필요한 타이틀은 정직하게 안내»). wave4 가 «네트워크로 데이터를 받아야 진행»으로 분류한 3종을 안내하기 전에 재 보니, 2종은 데이터를 이미 싣고 있었다.

**사용자 영향**: 지원 현황 playable 380 → **381** · limited 30 → **29** · not-yet 19(SKT 2종 limited → playable · `f44271803135` playable → limited). 행별은 §5 표. `f44271803135` 1종만 통신망 안내 대상이다. 두 KTF 종은 메뉴·게임 화면으로 넘어가고, SKT 2종은 막힌 첫 화면을 넘는다. ★대가: 동봉 설정이 이제 적용되어 KTF 6종이 30초 프로브에서 소리가 나지 않는다(§2).

### 1. «통신망» 3종 — 재 보니 2종은 엔진 벽

| sha12 | wave4 화면 | 실측(진단 빌드 · 커밋 0) | 처분 |
|---|---|---|---|
| `1793f87924d4` | «다운로드중 1/11» 영구 | `isFile("imgCEff_.ed") → false`. 그 파일은 아카이브 `p/` 에 있다(11쌍 `.dat`+`.ed`). 로더가 `P/` 만 벗겨 `p/…` 경로로 탑재했다 | **고침** — `p/` 도 루트에. 후: 22개 파일을 읽고 메뉴 → 이야기 인트로 |
| `74cb49013d64` | «추가다운로드를 받아야만… 1번 파일을 다운로드 하시겠습니까?» | `openDataBase("/D/FConfig" … "/D/FStageInit")` 가 빈 DB 를 새로 만든다. 아카이브에 `P/D/*.idx`+`.db` 4쌍과 내려받을 그림 `P/I/*.png` 가 이미 있다 | **고침** — 동봉 DB 설치. 후: 스테이지 선택 → 파티 선택 화면, `/I/*.png` 를 읽는다 |
| `f44271803135` | «추가 데이터 0K… CONNECTING» | 동봉 8파일(합 892,964B = 872KB)은 전부 읽는다. 그 뒤 `MC_netConnect` 를 30초에 99회(실패 콜백 M_E_ERROR 를 받고 되풀이). 게임이 이름 짓는 `/res/%d.dat` 중 `1.dat`·`4.dat` 이 jar 에 없다 | **안내** — 진행 불가 |

- `f44271803135` 오프라인 길: CONNECTING 화면에서 CLR·RSOFT·LSOFT·#·*·0·OK 를 각 4초 눌러 보았다 — 화면 9장 동일. 안내 문장에 담을 키가 없다.
- 안내 문장(정책 예문 그대로): 「게임을 시작하려면 옛 통신사 서버에서 데이터를 받아야 하는데, 그 서버가 지금은 없어 여기서는 진행할 수 없어요.」

### 2. 동봉 데이터 — 형식과 영향 범위

- **`qtpdb` 형식**(코퍼스 18 아카이브 · 40개 `.idx` 실측): `.idx` 45바이트, `qtpdb` + 레코드 크기(5, BE) + 레코드 수(9, BE) + 시각. `.db` = 레코드 크기 × 수(전 40개에서 정확히 일치). 머리가 깨진 1개(`qtpd\xff\xff`)는 설치하지 않는다. 레코드는 앞에서부터 MIDP id 1.. 로 들어가므로 WIPI id 0.. 와 맞는다(`to_midp_record_id`).
- **영향 범위 = 36종**(소문자 `p/` 를 싣는 KTF 25 ∪ `qtpdb` 18). 전수 도구 프로브(A·B 각 30초)를 전(origin/main)·후 바이너리로 같은 코퍼스에 돌렸다(`--jobs 2` · load1 150~305):
  - boot·render: 변화 0. input: 2건 뒤집힘(`974e0df9ab1e` ok→none · `f770b15f8876` none→ok — 같은 게임 두 판, 둘 다 A 가 `clean exit` 로 끝나 키 축이 원래 흔들린다).
  - **소리 생김 5**: `0e72b6bc12bb` `75f002ba70e3` `f7752f9124e8`(전 A·B 5장/4장 · max-ticks → 후 197·38·432장, 재생 3·2·6) · `1793f87924d4` · `74cb49013d64`.
  - **★소리 잃음 6**: `2fc792485d91` `3185174d2121` `5e53e490c6f1` `6a885f89343c` `bfa8ec352451` `de00506611a5` — 전 A·B 재생 1~6 → 후 A·B **둘 다 0**. 여섯 다 설정으로 보이는 파일을 싣는다(`opt.txt`·`setup.dat`·`sky*`·`kill`·`haga2` DB). 원 주인의 «소리 끔» 설정이 이제 적용되는 것으로 **추정**한다(게임 안 설정 화면에서 켜지는지는 재지 않았다). 대문자 `P/` 아카이브는 원래 이렇게 동작한다 — 이 PR 은 그 규칙을 소문자와 Java DB 로 넓혔다.
  - speed: 양쪽으로 뒤집힘 — 부하 소음(헤드리스 속도는 하한).

### 3. SKT 조작 멈춤 2종 — `showNotify` 를 부르지 않았다

- `38277d63b0ba`: 바이트코드(로컬 임시 역어셈) — `showNotify()` 가 그림 상태 카운터를 `-2` 로 놓고, `paint` 가 매번 +1 한 뒤 `-1` 일 때만 로딩 그림을 만든다. 엔진이 `showNotify` 를 부르지 않아 둘째 paint 가 `drawImage(null)` NPE 로 끝나고, 그 뒤의 `notify()` 에 닿지 못해 게임 스레드가 `wait()` 에서 영원히 기다렸다(화면 2장).
- `wie-midp` 에 `showNotify`/`hideNotify` 가 아예 없었다. MIDP 규격대로 Canvas 가 바뀔 때만 부른다(같은 것을 다시 `setCurrent` 하면 부르지 않는다).
- **영향 범위 = `showNotify` 문자열을 가진 SKT 28종**(코퍼스 SKT 82 중). 전·후 프로브: input 26 ok/2 none → **28 ok** · sound 24 → 25 ok · boot·render 변화 0 · 그 밖 변화는 speed `n/a→ok` 4건(부하).
- `6e9991f08650`: 같은 처방으로 로고 → 시나리오 선택. 1장 → 251·144장.

### 4. 4차 제안 p0·p1 · KTF AOT 소리 3종

- **p1 `1045007289d8`(숫자 칸 첫 글자) — 엔진 결함이 아니었다.** 생일 화면이 뜬 뒤 숫자를 보내면 0.7초 간격·load1 327 에서도 8자리가 다 들어가고(「1978年 04月 04日」) 메뉴 → 게임 시작 → 대화 장면까지 간다. wave4 레시피는 화면이 뜨기 전에 키를 보냈다(`OK:2` 뒤 25초 로딩). 레시피를 고쳐(`game_lab/` · 비커밋) 재생 10 · MIDI 1,738 ⇒ 소리 축 no → ok(측정 정정).
- **p0 `a16f08d025eb`(LGT 인터페이스 메서드 표) — 못 고침 · 이번 회차 미착수.** wave4 진단(표 21칸 중 11칸 이름 없음 · 서술자 메서드 0개) 그대로다. 크기 M~L.
- **KTF AOT 소리 3종(`1b3b4868d46e` `7218e8720f8c` `8bfd08fe4370`) — 못 고침 · 이번 회차 미착수.** ARM 추적이 필요하다(M). 위 세 고침과 그 영향 측정에 회차를 썼다.

### 5. `compat.json` — 이 회차가 잰 행만

| sha12 | 바뀐 것 | 근거 |
|---|---|---|
| `f44271803135` | status playable → **limited** · `knownIssues_ko` = 안내 한 줄 | §1 · 통신망 벽 |
| `1793f87924d4` `74cb49013d64` `0e72b6bc12bb` `75f002ba70e3` `f7752f9124e8` | sound no → ok | §2 프로브 A·B 둘 다 재생 |
| `1045007289d8` | sound no → ok | §4 레시피 실행 |
| `2fc792485d91` `3185174d2121` `5e53e490c6f1` `6a885f89343c` `bfa8ec352451` `de00506611a5` | sound ok → **no** | §2 프로브 A·B 둘 다 0 |
| `38277d63b0ba` `6e9991f08650` | status limited → **playable** · input no → ok · sound → ok · `6e9991f08650` longplay unknown → ok | §3 프로브 + 전수 도구 `--only long` 600초: 실패 줄 0 · 재생 230·15 |
| `38277d63b0ba` `1793f87924d4` | `progress` 키 제거(옛 엔진에서 잰 `stuck` — 그 벽을 넘었고 진도 축은 다시 재지 않았다) | §1·§3 |

`0e72b6bc12bb` `75f002ba70e3` `f7752f9124e8` `2fc792485d91` 의 `progress: stuck` 은 옛 엔진 값 그대로 두었다(이 회차가 진도 축을 재지 않았다).

방법: 전수 도구 `run --only probe`(대상 코퍼스 심링크 디렉터리 · 전/후 바이너리) → `report` → 바뀐 축만 손으로 옮겼다. 최상위 `enginePin` 유지.

### 6. 통신망 판별 — 규칙과 전수 재스캔

- 규칙(`netWall`): 프로브 A 또는 장시간 L 의 stderr 에서 `MC_netConnect`·`MC_netSocketConnect`·`MC_netHttpConnect`·`Network::connect(` 가 **20회 이상**.
- ★호출만으로는 틀린다: bd2337ff 전수(429종)에서 접속을 부른 타이틀 **30종**, 그중 29종은 1~5회 부르고 playable/limited 로 진행한다. 20회 이상 = **1종**(`f44271803135` 99회).
- 정책 판정 3종과 비교: 규칙이 고른 1 = 실제 진행 불가 1. 나머지 2는 위 §1 대로 엔진을 고쳐 대상에서 빠졌다. **추가분 0**.

### 7. 되돌리면 red(전부 실측)

| 고침 | 시험 | 되돌림 |
|---|---|---|
| 동봉 DB 설치 | `wie-wipi-java` `a_shipped_database_opens_with_its_records` | `database.add` 제거 → FAILED `left: 0` |
| `showNotify` 호출 | `wie-midp` `set_current_notifies_the_canvas_shown_and_the_one_hidden` | 호출 제거 → FAILED `left: 0` |
| `p/` 탑재 | `wie-ktf` `private_directory_mounts_at_the_root_in_either_case` | `P/` 만 → FAILED `left: "p/imgChar_.dat"` |
| 통신망 판별 | 전수 selftest 47/47 | 문턱 20→1 → 46/47 · `Network::connect` 제거 → 46/47 |

### 8. 게이트(빌드 전부 `build-slot`)

`cargo fmt --check` rc=0(첫 실행에서 한 줄 길이로 실패 → `cargo fmt`) · `cargo clippy --all -D warnings` rc=0 · wasm32 rc=0 · `cargo +beta clippy --all -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 51 스위트 **654/0** · `npm run build:wasm` rc=0. 러너 블록: draw · helloworld ×2 · text PASS · keydraw ×2 `--max-ticks 2e9` PASS.

### 9. 측정 조건

- load1 150~365(idle 0~1%) 동안 쟀다. `host-load-guard` 가 rc=1 이라 폭을 2로 줄였다(CLAUDE.md ⒞). 공용 전수 락은 전수 도구가 잡았다(전·후 차례로).
- 진단(`isFile`·`File`·`openDataBase`·`exists` 이름 로그)은 작업 트리에서 넣고 커밋 전에 되돌렸다.

### 10. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 333쌍 + SUFFIX-ATTACHED 15쌍. 전부 이 PR 이전부터 있던 값이다 — `compat.json` 기존 제목 값, 그리고 `wie-midp` `display.rs` 의 기존 주석 4곳(이 PR 이 같은 파일을 고쳐 대상이 됐다). 이 PR 이 더한 줄에서는 0이다. 새로 적은 타이틀은 전부 sha12 다.


<!-- corpus-name-inflow v1 subjects=12 tree=ece06e1509dc372e B=722/333 P=0/0 S=35/15 -->
