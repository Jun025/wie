## [2026-11-24] SKT `a0436eb1ddbb` — 저장 없는 «저장 있음» 표식(`option.dat`)을 올리지 않아 새 게임으로 시작한다 · 조정 1행 (wie-a0436eb1ddbb-skip-previous-owner-option-dat-so-it-starts-a-new-game)

**무엇을**: 한 판(SKT `a0436eb1ddbb` · 본체 jar MD5 `699700ed…c92c`)에서 꾸러미에 실린 전 주인의 `option.dat` 를 가상 파일로 올리지 않는다. 그 파일이 가리키는 저장 파일 두 칸이 꾸러미에 하나도 없을 때만이다. 장치는 이미 있던 «짝 없는 저장 표식» 표(`ORPHAN_ENTRIES` · 표식 `orphan-flag`)다. 그 표와 적용 함수를 `wie-lgt` 에서 `wie-backend::owner_files` 로 옮겨 LGT · SKT 의 `from_archive` 가 같이 쓰게 하고 SKT 행 1개를 더했다. 장부 1행 · `compat.json` 의 `runtimeAdjustments` 칸 1행 · 이용자 소식 1건. 디스크의 게임 파일은 한 바이트도 바꾸지 않는다.

**왜**: 제안 `2026-11-22-a0436eb1ddbb-aioobe-throw-site-measure#p0`(운영자 tower 채택 2026-11-24). 0570 이 쟀다 — 이 꾸러미의 `option.dat` 는 저장 칸 둘 다 «저장 있음»이라 말하는데 저장 파일은 꾸러미에 없다. 그래서 놀이가 «이어하기»로 들어가 빈 지도에서 시작하고, 방문객이 지도 밖으로 걸어 나가 예외가 난다.

**사용자 영향**: 이 판이 새 게임으로 시작한다. 여는 이야기와 안내(튜토리얼)가 나오고 공원 지도가 제대로 실린다. 전 주인의 설정(소리 등)도 같은 파일에 있어 함께 빠지고, 게임이 기본값으로 시작한다. 날짜가 다시 간다 — 진도 표기를 `stuck` → `ok` 로 실었다(선 가까이의 값이다 · §4). `status` 는 그대로다.

### 1. 무엇을 골랐나 — 소리 설정용 표가 아니라 «짝 없는 저장 표식» 표

- 고른 것: `ORPHAN_ENTRIES`. 뜻이 같다 — 계약 §1 의 `orphan-flag` 는 「짝이 되는 저장이 없는 «저장 있음» 표식을 올리지 않는다」이고, 이 판의 `option.dat` 가 바로 그 표식을 담은 파일이다(5 · 6번 바이트 · 0570 §3-B ⑵).
- 이 표에는 `unless_present` 가 있다. 이름 붙은 저장 파일이 꾸러미에 하나라도 있으면 표식을 그대로 올린다. 같은 빌드를 저장까지 함께 떠 온 이용자의 꾸러미는 온 그대로 이어하기가 된다. 소리 설정용 표(`OWNER_SOUND_OFF_SETTINGS` · 표식 `owner-file-skip`)에는 이 조건이 없고, 거기에 넣으면 공개 표식도 «소리 설정을 뺐다»로 틀리게 실린다.
- 표는 LGT 에만 있었다(`wie-lgt/src/emulator.rs`). SKT 에 같은 구조체와 함수를 한 벌 더 두는 대신 `wie-backend::owner_files` 로 옮겼다 — 소리 표 셋이 이미 거기 있고, 키가 본체 jar 의 MD5 라 통신사를 가리지 않는다. KTF 는 행이 없어 부르지 않는다.
- 바뀐 꼴 두 가지. ① `unless_present` 가 이름 하나에서 목록으로(이 판은 저장 칸이 둘이다 · 하나라도 있으면 둔다). ② 시험용 가짜 jar(`WIRING_PROBE_JAR`) 행 1개 — 소리 표 셋과 같은 관용이고, 통신사 쪽 시험이 «`from_archive` 가 이 표를 부른다»를 본다.
- LGT 의 동작은 그대로다. 조회 철자는 종전 LGT 함수와 같은 둘(맨 이름 · `P/`)이고, LGT 행(`a30bbe008b5e`)의 키 · 빼는 파일 · 조건이 같다. SKT 꾸러미는 맨 이름으로 싣는다.
- `wie-lgt` 의 `md5` 의존은 쓰는 곳이 없어져 뺐다(`Cargo.lock` 의 그 한 줄도).
- 저장 파일 이름은 0570 의 탐침 로그(`XFile.exists` 인자)와 디스어셈블리(`ag()Z` 의 문자열 상수 + 칸 번호)에서 왔다. 칸은 둘(0 · 1)이다.
- 이 꾸러미에 든 파일은 5개다 — jar · `.mod` · `.msd` · `.wmr` · `option.dat`(66바이트 · 최상위). 본체 jar 의 MD5 는 `unzip -p … | md5 -q` 로 다시 쟀고 `compat.json` 그 행의 `contentKey`(`zip:699700ed…`)와 같다. `option.dat` 의 내용은 이 회차에서 읽지 않았다.
- 범위: zip 꾸러미로 올릴 때만(`from_archive`). 같은 게임을 맨 `.jar` 로 올리면 올릴 설정 파일이 없어 걸리지 않는다.

### 2. «이 본체에만» 걸린다 — 시험과 개악 1건

- `wie-backend` `orphan_flag_dropped_for_listed_jar_only`: 다른 jar 로는 실제 표(`ORPHAN_ENTRIES`) 아래에서도 `option.dat` · `mastercom.sav` 가 그대로 남는다.
- `wie-backend` `orphan_flag_kept_when_any_of_its_data_is_present`: 저장 칸 파일이 하나라도(맨 이름 · `P/`) 있으면 표식이 남는다.
- `wie-backend` `orphan_rows_cover_the_measured_titles`: 두 행의 MD5 · 빼는 파일 · 조건 · jar 하나에 행 하나.
- `wie-skt` `from_archive_prepares_the_owner_files`: 가짜 jar 꾸러미에 `option.dat` 를 같이 실으면 그대로 올라가고(`Some([7])`) `probe.flag` 는 빠진다 — SKT 마운트 경로가 이 표를 부르고, 다른 MD5 에는 걸리지 않는다.
- `wie-lgt` `mount_path_prepares_the_owner_files`: LGT 경로도 같은 표를 부른다.
- `the_ledger_lists_exactly_these_tables`: 장부의 `orphan-entries` 행 집합과 표의 키 집합이 양방향으로 같다(시험용 행 제외).
- **개악 1건**: 적용 함수의 조건 `entry.jar_md5 == hash` 를 항상 참으로 바꾸면 `orphan_flag_dropped_for_listed_jar_only` 가 red 다 — `left: {"P/maplesave1.do", "app.jar"}` · `right: {…, "flag.sav", "mastercom.sav", "option.dat"}`(다른 jar 의 파일이 빠졌다) · `test result: FAILED. 2 passed; 1 failed`. 되돌린 뒤 파일 해시가 개악 전과 같음을 확인했다. cargo 가 첫 실패 묶음에서 멈춰 그 개악 아래의 `wie-skt` 시험은 돌지 않았다.

### 3. 전/후 측정 — 같은 순간 · census 진도 정책 600초

- 엔진: «전» = `origin/main` `de0969794` · «후» = 그 위에 이 PR 의 변경. 둘 다 release `wie_validate`. 꾸러미는 코퍼스의 원본 하나(sha12 `a0436eb1ddbb`)이고 읽기만 했다. 수정 사본은 만들지 않았다.
- 계측: crates.io `jvm-bytecode` 0.5.1 사본을 저장소 밖 스크래치에 두고 `eprintln!` 만 더했다(0570 의 0.1.1 용 계측을 지금 핀으로 옮긴 것). 20초마다 지도 `cq` 전체의 값 분포와 세 칸 · 지도 적재 `r(B)V` 호출마다 한 줄과 직후의 지도 · `h(II)B` 에서 나는 `ArrayIndexOutOfBoundsException` 마다 한 줄. 빌드는 `--config patch.crates-io.jvm-bytecode.path=<스크래치>` 로 스크래치 트리에서 했다. 저장소에 들어간 계측은 0 이다.
- 런: 판(P · P2)마다 셋을 **같은 임대 안에서 같은 순간에** 돌렸다 — 계측 «전» · 계측 «후» · 계측 없는 «후»(공개 값의 근거). 셋 다 `node scripts/playability-census.mjs run --only progress --progress 600 --as P|P2 --jobs 1` 이라 키 · 정책 · 판정이 도구 것 그대로다(창 600초 + 재개 120초). 계측 런 둘은 `--bin` 에 stderr 전체를 따로 받아 두는 두 줄짜리 래퍼를 줬다(census 는 마지막 200줄만 남긴다).
- ★census 의 호스트 잠금: 한 순간에 census 하나만 돌게 하는 잠금이라 그대로는 «같은 순간 짝»이 안 된다. 계측 없는 «후» 런이 진짜 잠금을 잡고, 계측 런 둘은 `WIE_CENSUS_LOCK` 을 스크래치 경로로 줬다. 동시 에뮬레이터는 3개(상한)이고 그동안 다른 census 는 진짜 잠금에 막혀 시작하지 못한다.
- census 는 진도 대상을 고를 때 그 판의 프로브(A · B · L)를 본다. 그 셋은 이전 전수 결과의 사본을 넣었다(대상 관문용 — 진도 값에는 들어가지 않는다).

**P** — 2026-11-24 04:57:11 ~ 05:09:18 · load1 181.5 → 171.7 → 196.0 · `build-slot` waiting 7 → 8

| | «전»(계측) | «후»(계측) | «후»(계측 없음) |
|---|---|---|---|
| `r(B)V`(지도 적재) 호출 | **0** | **2**(44 · 78초) | — |
| 지도 564칸(20초마다) | 40 ~ 581초 내내 -1 이 562칸 · 0 이 1 · 1 이 1 | 78 ~ 580초 내내 -1 이 251 · -2 가 78 · -3 이 230 · -4 가 5 — 0570 이 적은 자원 `data/1.dat` 분포와 같다 | — |
| `cq[13][0]` · `cq[19][3]` · `cq[35][11]` | -1 · -1 · -1 | -2 · -4 · -3 | — |
| `h(II)B` 예외(600초 창 · 재개 120초 포함) | 0 | 0 | 계측이 없어 세지 못한다(`uncaught_threads` 0) |
| 화면 윗줄 날짜 | 11/24 → 11/25(40초) 뒤 600초까지 그대로 | 11/24(100 ~ 350초 상점 창) → 11/28(380초) → 12/23(450초) → 1/27(540초) → 2/15(600초) | 11/24(100초) → 12/3(200초) → 12/29(300초) → 1/11(380초) → 1/15(420초) 뒤 600초까지 그대로(상점 창) |
| 날짜 띠가 바뀐 10초 프레임 수(380 ~ 600초) | 0 | 23 | 4 |
| 시작 | 여는 이야기 없음 · 소지금 2000만원 | 여는 이야기 → 안내 → 공원 · 소지금 0원 | 같다 |
| 도구 값 `p_stall` · 새 화면 수 | 550초 · 4 → `stuck`(짝 필요) | 210초 · 10 → `stuck`(짝 필요) | 180초 · 11 → `ok` |
| paints · 탈출 키 | 6556 · 8 | 11449 · 7 | 11029 · 7 |

- «전»의 예외 0 은 0570 과 같은 꼴이다. 거기서도 원본은 창 900초의 끝(898초)에야 예외가 났고 600초 창(런 P)은 0 이었다 — 상점 창이 열려 있는 동안은 날짜가 서고 방문객이 걷지 않는다. 그래서 이 표에서 조정의 효과를 가르는 줄은 예외 수가 아니라 **지도 내용과 `r(B)V` 호출**이다. 빈 지도(예외의 조건)가 «후»에는 한 번도 없다.
- «후»의 20 ~ 40초 · 620초 뒤(재개)의 지도는 -1 균일이다. 놀이에 들어가기 전 · 다시 띄운 뒤의 초기값이고 «전»도 같다. 60초의 한 번은 여는 이야기가 보여 주는 완성된 공원이다.

**P2** — 2026-11-24 05:16:19 ~ 05:28:23 · load1 139.2 → 193.5 → 132.5 · `build-slot` waiting 5 → 9

| | «전»(계측) | «후»(계측) | «후»(계측 없음) |
|---|---|---|---|
| `r(B)V`(지도 적재) 호출 | **0** | **2**(44 · 79초) | — |
| 지도 564칸(20초마다) | 40 ~ 600초 내내 -1 이 562칸 · 0 이 1 · 1 이 1 | 79 ~ 600초 내내 -1 이 251 · -2 가 78 · -3 이 230 · -4 가 5 | — |
| `cq[13][0]` · `cq[19][3]` · `cq[35][11]` | -1 · -1 · -1 | -2 · -4 · -3 | — |
| `h(II)B` 예외 | 0 | 0 | 계측이 없어 세지 못한다(`uncaught_threads` 0) |
| 화면 윗줄 날짜 | 11/25 에서 600초까지 그대로 | 11/24 → 12/3(200초) → 1/3(300초) → 2/1(370초) 뒤 600초까지 그대로(상점 창) | 11/24 → 12/3(200초) → 1/3(300초) → 2/3(380초) → 2/14(420초) → 2/23(460초) 뒤 600초까지 그대로(상점 창) |
| 날짜 띠가 바뀐 10초 프레임 수(380 ~ 600초) | 0 | 0 | 9 |
| 도구 값 `p_stall` | 550초 → `stuck` | 220초 → `stuck` | 100초 → `ok` |
| paints · 탈출 키 | 6774 · 9 | 11699 · 8 | 12286 · 6 |

- 두 판을 합친 도구 값(census `report`): «전»(계측) `stuck`(550 · 550) · «후»(계측) `stuck`(210 · 220) · «후»(계측 없음) `ok`(180 · 100). 가르는 선은 창의 1/3 인 200초다.
- 지도 쪽은 두 판이 같다 — «전»은 600초 내내 빈 지도이고 적재가 0번, «후»는 두 번 적재되어 자원 분포 그대로다.

### 4. 진도 표기 — `stuck` → `ok` 로 싣는다 · 다만 선 가까이에 있다

- **실은 값**: `compat.json` 그 행의 `axes.progress` 를 `stuck` → `ok`. `status` 와 여섯 축은 건드리지 않았다.
- 근거(계약 `docs/contracts/featurephone-public-data.md` §1 의 progress 줄): 공개 값은 그 빌드의 census 정책 키 판이 정한다. 계측 없는 «후» 빌드의 짝이 둘 다 `ok` 다(`p_stall` 180초 · 100초). «눈 확인 되풀이» 예외(0552)는 380 ~ 600초 화면에 재화 · 날짜 · 시계 변화가 0 일 때만 `ok` 를 `stuck` 으로 내린다. 두 판 다 그 창 안에서 날짜가 갔다(P 1/11 → 1/15 · P2 2/3 → 2/23). 예외에 해당하지 않는다.
- 0552 가 이 행을 `stuck` 으로 둔 까닭은 «상점에서 구입비용 부족을 되풀이하고 날짜가 그대로»였다. «전»은 지금도 그렇다(두 판 다 40초부터 600초까지 날짜 변화 0). «후»는 네 런 모두 600초 안에 날짜가 50일 넘게 간다(52 ~ 91일).
- ★**선 가까이다 — 숨기지 않는다.** «후» 네 런의 `p_stall` 은 180 · 100 · 210 · 220초이고 선은 200초다. 계측 빌드의 짝은 같은 게임 코드인데 도구 값이 `stuck` 이다(계측 빌드는 공개 값의 근거가 아니다 — 싣는 빌드가 아니다). 까닭은 같다: 정책 키가 놀이기구 상점을 열고, 새 게임의 소지금이 0원이라(§5) «구입비용이 부족합니다» 창이 되풀이되며, 그 창이 열려 있는 동안은 날짜가 선다. 그 구간이 언제 오는지가 런마다 다르다(시작 100 · 370 · 420 · 460초). 계측 P2 는 380 ~ 600초 전체가 그 창이었다.
- 또 하나: 도구의 «새 화면»(16×16 휘도 지문)은 날짜 숫자만 바뀌는 프레임을 새 화면으로 세지 않는다. 계측 P 는 380 ~ 600초 내내 10초마다 날짜가 갔는데도 `p_stall` 이 210초다. 반대로 상점 창의 품목이 바뀌면 새 화면으로 센다.
- 호스트가 내내 붐볐다(load1 132 ~ 196). 네 런 모두 같은 순간의 «전»과 짝이라 전/후 차이는 부하 탓이 아니지만, 선 가까이의 값이 조용한 호스트에서 어느 쪽으로 가는지는 재지 않았다.
- 증적 화면(저장소 밖): `sheets/out-before-P.png` · `out-before-P2.png` · `out-after-P.png` · `out-after-P2.png` · `out-after-i-P.png` · `out-after-i-P2.png`(각 100 ~ 600초 12장) · `out-after-P-early.png` · `out-before-P-early.png`(10 ~ 130초).

### 5. 새 게임의 소지금 0원 — 게임 코드가 넣는 값으로 읽힌다(읽은 것 · 재지 않았다)

- «후»의 화면은 소지금이 0원이다. «전»은 2000만원에서 시작해 놀이기구를 사고 190만원이 된다.
- 바이트코드에서 읽은 것(스크래치 디스어셈블리 · `GameCanvas`): 놀이 초기화 `c(Z)V` 는 정적 필드 `A` 가 1 이면 소지금 `cd` 에 20,000,000 을 넣고(«전»의 이어하기 길에서 본 2000만원과 맞는다), 아니면 공원별 표 `S[ca]` 의 값을 넣는다. `cd` 에 상수 0 을 넣는 자리는 큰 갈래 메서드 `a(S)V` 의 두 곳(오프셋 4355 · 5003)이고, 같은 메서드의 다른 갈래(4696)는 `l(2000000)` 으로 200만원을 더한다. 어느 자리에도 플랫폼 API 의 답이 들어가지 않는다.
- 그래서 0원은 새 게임 길에서 게임 코드가 넣은 값으로 읽힌다. ★어느 자리가 실제로 돌았는지는 재지 않았다. 안내(튜토리얼)를 지시대로 따라가면 돈이 나오는지도 재지 않았다 — 진도 정책 키는 안내의 지시를 따르지 않는다.

### 6. 검사

- 네 게이트: `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(981 passed · 0 failed). main 의 982 에서 1 준 것은 시험을 옮기며 합쳤기 때문이다 — `wie-lgt` 의 4개(장부 대조 1 · 표식 3)가 `wie-backend` 의 3개와 기존 장부 대조 시험의 한 줄이 됐고, `P/` 철자는 반복문의 한 경우로 들어갔다.
- `cargo +beta clippy --all -- -D warnings` rc=0. PR 의 beta 다리는 빈 통과라 이 로컬 실행이 beta 의 유일한 근거다.
- 러너 블록(엔진 코드를 건드렸다): `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS rc=0 · `keydraw_ktf.zip` · `keydraw_lgt.zip` `--inject --expect-last-frame` PASS rc=0(`input_steps` 27/27) · `text_j2me.jar --timeout 5` PASS rc=0. load1 101.6.
- `node scripts/player-data.mjs` → `player-data OK — 429 titles · status {"playable":426,"not-yet":2,"limited":1} · platform {"KTF":269,"LGT":78,"SKT":82} · 189 updates · runtimeAdjustments on 50 rows (ledger 55 rows)` rc=0. `--selftest` → `player-data selftest: 67 rules each reject their mutation` rc=0.
- `node scripts/check-compat-revert.mjs` → `compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 1`. 그 1행이 `a0436eb1ddbb` 다: `axes.progress` `stuck` → `ok` · `runtimeAdjustments: ["orphan-flag"]` 추가(칸은 `normalize` 가 장부에서 만들었다).
- `node scripts/check-worklog-json.mjs` OK · `npm run audit` → `AUDIT PASSED`.
- 시험을 옮긴 PR 이라 `coverage.yml` 을 이 브랜치로 손 실행한다(결과는 PR 과 회신에 적는다).
- WASM export 표면은 바뀌지 않았다(`wie_featurephone` 무접촉) — 엔진 계약 파일 변경 0.

### 7. 측정 규율 · 유출

- 임대: 측정은 판마다 `build-slot run --long` 1건(약 12분) — P · P2 두 건이고 스윕 전체를 한 임대로 감싸지 않았다. 동시 에뮬레이터는 3개 이하. 빌드 · 시험은 short 임대. 회차 끝에 내 프로세스는 0 이다.
- 스크래치(`~/scratch/wie-a0436-option`)는 이 티켓을 먼저 잡았던 세션이 만든 것을 이어받았다(계측 사본 · census 틀). 회차 끝에 디렉터리째 지웠다. 거기 있던 것 가운데 게임에서 온 것은 원본을 가리키는 심링크와 분석용으로 푼 클래스 파일 하나뿐이고 같이 지웠다.
- 증적(탐침 줄 · 판별 부하 · 날짜 띠 · 화면 모음 · 계측 패치 · 도구 출력)은 저장소 밖 `~/orchestrator/reports/evidence/wie-a0436eb1ddbb-skip-previous-owner-option-dat-so-it-starts-a-new-game/` 에 있다.
- 저장소 · PR 에 게임 바이트 · 자원 내용 · 캡처는 0 이다. `option.dat` 의 내용은 읽지도 옮기지도 않았다.

### 8. 게임 파일명 유입

- 게임 이름 유입(`corpus-name-inflow`): BOUNDED __B__ · SUFFIX-ATTACHED __S__. 이 회차가 더한 이름은 장부 한 줄의 `title` 칸 하나다(장부의 열이 `compat.json` 그 행의 `title` 을 그대로 요구한다). 나머지는 `compat.json` 을 건드려 파일 전체가 대상이 된 것이고 공개 목록의 `title` · `game` 값이다. 이 문서 · 소식 · worklog 는 그 판을 sha12 로만 적는다. 게임 바이트 · 자원 내용 · 캡처는 0.
