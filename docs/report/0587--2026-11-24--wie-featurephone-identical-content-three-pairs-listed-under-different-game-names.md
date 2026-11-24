## [2026-11-24] 안쪽 파일이 전부 같은 3쌍이 지원 목록에 다른 게임 이름으로 두 줄 — 한 이름으로 맞춤 (wie-featurephone-identical-content-three-pairs-listed-under-different-game-names)

**무엇을**: `scripts/player-data.mjs` 의 `SAME_NAME_HAND` 에 4행, `SAME_GAME` 에 3묶음을 더하고 `normalize` 로 `docs/player-data/compat.json` 을 다시 만들었다. 지문(`contentKey`)이 같은데 `game` 이 달랐던 3쌍이 쌍마다 한 `game` 이 된다. 규칙과 검사기 코드는 그대로다.

**왜**: 운영자 채택(2026-11-24 · 제안 `2026-11-22-featurephone-content-fingerprint#p0`). 0567 이 지문을 매기면서 지문이 같은 21묶음 중 3묶음의 `game` 이 서로 다른 것을 봤다. 이름 정규화 회차(0555 · 0560 · 0585)의 판정 표에는 이 여섯 파일이 없다.

**사용자 영향**: 지원 게임 목록에서 같은 게임이 파일 이름 때문에 두 줄로 보이던 3건이 각각 한 줄(파일 2개)이 된다. 목록의 게임 수는 364 → 361 이다. 행은 지우지 않았다(429 → 429 · sha256 불변). 원 이름은 `fileTitle` 에 남아 검색에 걸린다.

### 0. 다시 잰 수

식: `compat.json` 의 `entries` 를 `contentKey` 로 묶고(값이 있는 행만), 행이 둘 이상인 묶음 가운데 `game` 값의 종류가 둘 이상인 묶음을 센다.

- `origin/main` `a3217eac0`: 지문을 나눠 갖는 묶음 21 · 그중 `game` 이 갈린 묶음 **3** — 티켓의 3쌍 그대로다. 통신사가 갈린 묶음 0.
- 이 브랜치: 21 · **0**.

### 1. 판정 — 메타데이터만(게임 바이트·리소스 내용은 인용하지 않음)

방법은 0555 · 0560 · 0585 와 같다: 설명 파일(`__adf__` · `.msd`)의 PID · Name · Ver · DisplaySize · AID, 꾸러미 항목의 이름과 내용 해시의 일치 수(해시 값은 비교만 했다). 코퍼스는 `game_lab/`(git-ignored)이다. 이름은 이 문서에 적지 않는다 — 값은 `compat.json` 의 그 행에 있다.

| 쌍(sha 앞 12자) | 통신사 | 판정 | 근거 | 대표 이름 | variant |
|---|---|---|---|---|---|
| `2f84b8cc870d` ↔ `9e2307e4fa88` | SKT | same-game | 항목 4 ↔ 4 가 이름과 내용까지 전부 같음 · `.msd` 가 바이트까지 같음(MIDlet-Name · MIDlet-Version 1.0.0 · MIDlet-Vendor 같음) | 서술자 MIDlet-Name. 정규화(`gameKey`)하면 `9e2307e4fa88` 의 기존 `game` 과 같아 그 표기를 쓴다 | null · null |
| `4120f27288ab` ↔ `739c7657c1f2` | KTF | same-game | 항목 6 ↔ 6 전부 같음 · `__adf__` 가 바이트까지 같음(PID PD006079 · Ver 01.00.01 · DisplaySize 240*320 · AID 같음) | 서술자 Name 그대로. 두 파일 이름 어느 쪽과도 다르다(한쪽은 끝 한 글자가 없고 한쪽은 시리즈 이름을 줄였다) | null · null |
| `8b01d4ae6af4` ↔ `d60c34ffc9f6` | KTF | same-game | 항목 6 ↔ 6 전부 같음 · `__adf__` 가 바이트까지 같음(PID PD004840 · Ver 01.00.02 · DisplaySize 240*320 · AID 같음) | `8b01d4ae6af4` 의 기존 `game`(아래) | null · null |

**셋째 쌍만 서술자 Name 을 그대로 쓰지 않았다.** 그 Name 은 시리즈 이름뿐이고 편 이름이 없다. 목록에는 같은 시리즈의 다른 편이 KTF 에 둘 더 있어(위 둘째 쌍 · `d4188f8ef8c4`/`ed6ad7318ac9`) 시리즈 이름만으로는 어느 편인지 가르지 못한다. 두 파일 이름은 같은 편을 적고, `8b01d4ae6af4` 의 기존 `game` 은 «Name + 그 편 이름»이다. 그 `game` 은 LGT 행 `870cd071b4dc` 와 이미 한 줄이고 그 LGT 파일의 서술자 Name 과 글자까지 같다. 0585 의 쌍 A(Name 에 부제가 없어 목록의 더 구체적인 표기를 쓴 것)와 같은 방향이다. KTF 파일과 그 LGT 파일이 같은 편이라는 메타데이터 근거는 따로 얻지 못했다(본체 안 같은 이름 항목 1 · 내용 같은 항목 0 — 리소스 묶는 방식이 다르다). 그 KTF ↔ LGT 묶음은 0560 의 이름 규칙이 만든 것이고 이 회차가 만들거나 바꾼 것이 아니다.

**`variant` 는 여섯 행 모두 `null`**: 쌍 안의 두 파일은 설명 파일과 항목이 전부 같아 한쪽에만 참인 라벨이 없다. 0560 의 «zip 포장만 다른» 쌍과 같은 처리다. 셸은 두 행을 해시 라벨(`파일 #…`)로 가른다.

### 2. 수

- 행 429 → 429 · sha256 집합 같음(한쪽에만 있는 값 0 · 0) · 최상위 칸 같음 · 이름 칸(`title` · `game` · `variant` · `sameTitle`) 밖의 변경 0(sha 별 대조) — `status` · 축 값 변경 0.
- distinct `game` **364 → 361** · (platform, game) 401 → 398.
- 바뀐 행 6 · 그중 `title` 이 바뀐 행 **4**: `2f84b8cc870d` · `4120f27288ab` · `739c7657c1f2` · `d60c34ffc9f6`(`title` · `game` 이 함께 바뀌고 `sameTitle` 신설). `8b01d4ae6af4` · `9e2307e4fa88` 은 `sameTitle` 신설만.
- `same-game` 55 → 61행 · `undetermined` 0 → 0 · `variant` 실린 행 16 → 16.
- 셸의 묶음 식(`support-counts-check.mjs` 의 열쇠)으로 센 값: 게임 수 364 → 361 · 통신사별 대표 수 401 → 398(KTF 244 → 242 · LGT 77 · SKT 80 → 79) · 파일이 둘 이상인 줄 58 → 59. 대표에서 빠지는 행 3: `4120f27288ab` · `d60c34ffc9f6` · `9e2307e4fa88`. 새로 대표가 되는 행 0.
- 둘째 쌍의 공개 산출물 대표는 `739c7657c1f2` 다(여섯 축 `ok` 수는 둘 다 6 · `changes` 2 ↔ 1). 그 행의 `progress` 는 `stuck`, 같은 내용인 `4120f27288ab` 는 `ok` 다. 대표 규칙(계약 §1)은 `progress` 를 보지 않고, 두 파일은 안쪽이 같으므로 이 차이는 측정의 흔들림이다(0567 §1). 값은 바꾸지 않았다.

### 3. 검사

- `node scripts/player-data.mjs` → `player-data OK — 429 titles · status {"playable":426,"not-yet":2,"limited":1} · platform {"KTF":269,"LGT":78,"SKT":82} · 187 updates · runtimeAdjustments on 49 rows (ledger 54 rows)` · `contentKey — 408 distinct on 429 rows · 21 shared by 42 rows · 0 shared with differing status` rc=0. 장부(`runtime-adjustments.tsv`)에 이 여섯 파일의 행은 없다(제목을 따라 고칠 장부 행 0).
- `node scripts/player-data.mjs --selftest` → `player-data selftest: 67 rules each reject their mutation` rc=0. 규칙·검사기 코드는 바꾸지 않았다(표 일곱 줄뿐). 이 PR 에 Rust 변경 0.
- 착지 사전 증거: 이 브랜치는 wie `origin/main` `a3217eac0` 에서 바로 갈라져 있어 합친 형상이 브랜치 그대로다. 그 트리에서 `build --out` → otterpebble `origin/main` `b88d9f44d` 스크래치 클론에 `node scripts/compat-import.mjs <built>/compat.json`(rc=0 · 429종) → `apps/featurephone` 에서 `vitest run`: `Test Files 71 passed (71)` · `Tests 973 passed (973)` rc=0 · 실패 0. 넣기 전 대조군도 71 · 973. 셸 `lib/compat.json` 에서 이름 칸이나 `status` 가 달라지는 행은 위 6행뿐이다. 셸에 필요한 변경은 없다.
- 셸 시각 회귀 기준(`apps/featurephone/e2e/visual.spec.ts` · 기준 이미지 24장)에 `title` 이 바뀌는 4행은 찍히지 않는다. 코드로 본 근거: 게임 상세 캡처의 대상은 `SUPPORT_ID = "6e5a1f88f0c0"` 하나이고 제목을 고정 문자열로 바꿔 찍는다. 그 쪽(`app/support/[id]/page.tsx`)은 자기 행만 그린다. 나머지 캡처는 `/` 와 `/?demo=…` 이고 `app/page.tsx` 가 `/` 에 그리는 목록 유래 값은 게임 수와 통신사별 대표 수뿐이다(제목 0). 그 수 블록은 「최근 업데이트」 카드(캡처 동안 가려진다) 아래라 뷰포트(1280×800 · 390×844) 밖이다 — 홈 기준 이미지 4장은 2026-10-22 에 한 번 찍힌 뒤 그대로이고, 홈·보관함 대화상자·게임 상세 기준 이미지를 열어 그 블록이 없음을 봤다. playwright 는 돌리지 않았다.
- 네 게이트(`fmt --check` · `clippy --all -D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all`) rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(PR 의 beta 다리는 빈 통과라 이 로컬 실행이 beta 의 유일한 근거다).
- `npm run audit` → `AUDIT PASSED`. `node scripts/check-engine-contract.mjs` 는 이 워크트리의 `web/src/wasm`(git-ignored · 2026-11-22 빌드)로는 rc=1 이다 — `set_wake_margin` · `wake_in_ms` · `wake_tick` 3건이 그 낡은 산출물에 없다. 이 PR 은 wasm 표면을 건드리지 않고 산출물을 다시 빌드하지 않았다. 판정은 CI `contract` 잡(산출물을 새로 빌드한다)의 것이다.
- `node scripts/check-compat-revert.mjs` → `compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 4` rc=0. 이 검사는 `title` · `status` 등을 보고 `game` · `sameTitle` 은 보지 않는다 — 4 는 `title` 이 바뀐 행이고, `sameTitle` 만 새로 실린 2행을 더하면 §2 의 6 이다.

- 게임 이름 유입(`corpus-name-inflow`): BOUNDED 350쌍(1089회) · SUFFIX-ATTACHED 18쌍(60회). 의도한 유입이다 — 공개 목록(`compat.json` 의 `title`·`game`)이 산출물이고 `SAME_NAME_HAND`·`SAME_GAME` 이 그 이름을 적는다. 이 문서에는 이름을 적지 않았다. 게임 바이트·리소스 내용·캡처는 0.

<!-- corpus-name-inflow v1 subjects=4 tree=0df5f7cc84491f95 B=1089/350 P=4/4 S=60/18 -->
