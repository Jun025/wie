## [2026-11-22] 공개 게임 목록의 `game` 키 — 한 게임에 표기 하나 · 이름이 다른 같은 게임은 메타데이터로 맞춤 (wie-featurephone-compat-game-key-normalize-and-aliases)

**무엇을**: `scripts/player-data.mjs` 의 `retitle` 이 «정규화(`gameKey` = NFKC · 소문자 · 공백과 하이픈 제거)하면 같은 이름»을 통신사와 무관하게 같은 `game`·`title` 로 내보낸다. 묶음의 표기는 대표 표기 규칙으로 고른다(①대문자가 많은 것 ②띄어 쓴 곳이 많은 것 ③하이픈이 적은 것 ④문자 코드 순). `validateCompat` 은 정규화하면 같은데 `game` 이 갈린 묶음을 거부한다. 이름이 아예 달라 정규화로 못 만나는 같은 게임 3건은 메타데이터로 판정해 `SAME_NAME_HAND` 에 적었다. 라벨 없는 판 1건(붕어빵타이쿤3)은 판정만 하고 라벨은 달지 않았다(§검수 뒤 고침). 계약 = `docs/contracts/featurephone-public-data.md` §1.

**왜**: 운영자 지시(2026-11-16 「동일 게임으로 추정되는 것들도 검토해 깔끔히」). #588 데이터에는 띄어쓰기·하이픈·대소문자만 다른 `game` 이 9묶음 남아 있었고, 셸(otterpebble #1435)이 그 차이를 «표시할 때» 정규화해 합치고 있었다. 데이터가 스스로 맞으면 셸의 그 정규화는 아무 일도 하지 않는다.

**사용자 영향**: 지원 게임 목록에서 같은 게임이 통신사마다 다른 표기로 보이던 9건이 한 표기가 된다. 따로 줄이 나뉘어 있던 3건(통신사별 약칭 · 틀린 파일 이름)은 한 줄로 모인다. 행은 지우지 않았다(429 → 429 · 매칭 키 sha256 불변).

### 판정 — 메타데이터만(게임 바이트·리소스 내용은 인용하지 않음)

방법은 0555 와 같다: `__adf__`/`app_info`/`.msd` 의 PID·Name·Ver·DisplaySize, 아카이브 항목 이름과 내용 해시의 일치 수(해시 값은 비교만 했다), 본체 jar 해시 일치 여부. 코퍼스는 `game_lab/`(git-ignored)이다.

| 묶음 | sha 앞 10자 | 판정 | 근거 | 결과 title / game / variant |
|---|---|---|---|---|
| KTF 크로노스 윙 ↔ 크로노스윙 | d9384b388e · 1fe6c1c897 | same-game | PID PD007589·Name·Ver 01.00.06·240*320 같음 · 본체 jar 같음 · `__adf__` AID 만 다름 | 크로노스 윙 / 크로노스 윙 / null · null |
| KTF kbo 프로야구 2009 ↔ kbo프로야구 2009 | cdd3eb5f91 · 4892a1abc0 | same-game | PID PD007654·Ver 01.00.02·176*220·AID 같음 · 항목 89/89 내용까지 같음(zip 포장만 다름) | KBO프로야구2009 / KBO프로야구2009 / null · null |
| KTF 드래곤나이트 ex ↔ 드래곤나이트EX | c94d647779 · e9fac881e6 | same-game | PID PD003066·Name·Ver 01.00.10·240*320 같음 · 본체 jar 같음 · AID 만 다름 | 드래곤나이트EX / 드래곤나이트EX / null · null |
| KTF 메이플스토리 마법사편 ↔ SKT 메이플스토리-법사편 | 1cdea19859 · 52edc54e2b | same-game(이식판) | KTF `__adf__` Name «메이플-마법사편» ↔ SKT `.msd` MIDlet-Name «메이플스토리마법사편» · 본체 안 같은 이름 항목 138 중 135 가 내용까지 같음(같은 SKT 파일을 KTF 전사편·궁수편과 대면 20·26 뿐) | 양쪽 메이플스토리 마법사편 / 메이플스토리 마법사편 / null |
| KTF 물가에 돌튕기기 ↔ SKT 물가에돌튕기기IQ | f981d228b7 · 4b44b31e81 | different-game | KTF `__adf__` Name «물가에돌튕기기3» PD005691 ↔ SKT MIDlet-Name «물가에돌튕기기IQ» · 같은 이름 항목 0 | SKT 그대로 · KTF 행은 아래 줄 |
| KTF 물가에 돌튕기기 ↔ KTF 물가에돌튕기기3 (티켓 쌍 밖 · 같은 프로브에서 드러남) | f981d228b7 · 4a4d2ac046 | same-game(파일 이름이 틀렸다) | PID PD005691·Name·Ver 01.00.04·240*320 같음 · 본체 jar 같음 · AID 만 다름 | 물가에돌튕기기3 / 물가에돌튕기기3 / null · null (이름 출처: `__adf__` Name) |
| KTF 서든어택 ↔ LGT 서든어택포켓 | 0cc4ef7ede · 517ed32c92 | different-game | Name «서든어택» Ver 01.00.03 ↔ «서든어택포켓» Ver 01.00.07 · 실행 클래스 이름 다름 · 항목 397 ↔ 690 에 같은 이름 0 | 그대로 |
| LGT 슈퍼액션히어로3 ↔ KTF 액션히어로3D | 2d5cada030 · 3151fdc167 | different-game | Name «슈퍼액션히어로3» ↔ «액션히어로3D: WILD DOG» · 항목 20 ↔ 356 에 같은 이름 1 | 그대로 |
| SKT 영웅서기 ↔ KTF 영웅서기 제로 | 1f0d7e8133 · 60704ad61a | different-game | MIDlet-Name «영웅서기_솔티아의바람» ↔ Name «영웅서기 Zero(진홍의 사도)» · 같은 이름 항목 26 중 내용 같은 것 3 | 제로 그대로 · SKT 행은 아래 줄 |
| SKT 영웅서기 ↔ KTF 영웅서기 1 (티켓 쌍 밖 · 같은 프로브에서 드러남) | 1f0d7e8133 · ead25bc6ff | same-game(이식판) | 양쪽 서술자 Name 이 같은 부제(«영웅서기_솔티아의바람») · 같은 이름 항목 350 중 345 가 내용까지 같음 | 양쪽 영웅서기 1 / 영웅서기 1 / null |
| KTF 붕어빵타이쿤3 라벨 없는 판 | ab64a56b2b (↔ 1742377585 작은화면 · 5bfe10a828 큰화면) | same-game · 큰화면 판과 같은 본체 | PID PD004489 같음 · 240*320·Ver 01.00.01 = 큰화면 판(작은화면 판은 176*220·01.00.02) · 본체 jar 가 큰화면 판과 같음(358항목 중 357 같음 · AID 만 다름) | 붕어빵타이쿤3 / 붕어빵타이쿤3 / null (라벨은 큰화면 판 한 행만 가진다) |

`undetermined` 는 0건이다.

**티켓 쌍 밖 2건을 함께 적은 이유**: 근접 이름 쌍을 재던 프로브가 두 행의 서술자를 그대로 보여 줬다 — «물가에 돌튕기기»는 자기 `__adf__` 가 3편이라고 말하고, SKT «영웅서기»는 KTF «영웅서기 1» 과 같은 부제·같은 리소스다. 재 놓고 틀린 이름을 남기는 쪽이 더 나쁘다고 봤다. 근접 이름 전체를 훑은 것은 아니다(티켓의 4쌍과 그 쌍이 닿은 행뿐).

**붕어빵타이쿤3 — 라벨 없는 재포장 판은 `variant: null`**: 티켓은 «같은 크기의 다른 판이면 판 번호»를 예상했지만, 재 보니 크기도 판 번호도 본체도 큰화면 판과 같다(설치 칸 AID 만 다른 재포장). 참인 라벨은 «큰화면» 하나뿐인데 그 라벨은 이미 `5bfe10a828` 이 갖고 있다. 그래서 이 행에는 라벨을 달지 않는다 — (platform, title, variant) 유일 규칙(#588)은 예외 없이 그대로다. 셸은 이 행을 `파일 #ab64a5` 로 보인다(착지 전과 같다). 이 회차의 첫 판은 두 행이 «큰화면» 을 나눠 갖게 하고 `same-game` 묶음 안의 겹침을 허용했는데, 검수가 그 예외를 반려했다(§검수 뒤 고침).

### 수치

- 행 429 → 429 · sha 집합 같음 · 이름 필드(`title`·`game`·`variant`·`sameTitle`) 밖의 변경 0(sha 별 대조).
- distinct `game` **378 → 365**(정규화 9묶음에서 −10 · 별칭 −3) · (platform, game) 406 → 402 · 정규화 충돌 묶음 **9 → 0**.
- `game` 이 바뀐 행 13 · `sameTitle` 만 새로 실린 행 3 · `variant` 가 바뀐 행 0. `same-game` 45 → 53행 · `variant` 실린 행 16 → 16.
- 대표 표기로 고른 9묶음: 2006 독일축구 · 던전앤파이터 귀검사 · 드래곤나이트EX · 메이플스토리 궁수편 · 메이플스토리 전사편 · 일지매 영웅전기 · 크로노스 윙 · 현영맞고 2006 · KBO프로야구2009.

### 검사

- `node scripts/player-data.mjs --selftest` → 47 rules(첫 판 45 · §검수 뒤 고침에서 +2). 개악 «one game spelled two ways»(한 게임을 `t` / `T` 로) — 규칙을 끄면 `selftest: NOT rejected — one game spelled two ways` rc=1, 켜면 rc=0.
- 실데이터 개악(한 행을 옛 표기로 되돌림) → `compat: game "크로노스윙" is spelled "크로노스윙" / "크로노스 윙" — one game, one key`.
- `build --out` 산출물을 otterpebble `origin/main` `a286be3e` 의 `compat-import.mjs` `validateCompat` 로 읽기 전용 실행 → 위반 0 · 429행. 셸 `normalizeGameKey` 로 묶은 수 = `game` 그대로 묶은 수 = 365(셸 정규화가 더 합치는 묶음 0). 셸 정렬 뒤 대표 행이 바뀐 묶음 0/402(wie 순서 ↔ 셸 순서 비교다 · 착지 전 ↔ 후로는 대표가 2곳 바뀐다 — §검수 뒤 고침의 예측).
- 정규화로만 합쳐진 통신사 사이 6묶음도 서술자를 봤다. Name 이 서로 같거나, 한쪽이 부제를 빼거나 연도 자리만 달리 쓴 정도이고 다른 게임을 가리키는 묶음은 없다. 이 6묶음은 이름 규칙으로 합친 것이라 판정 표의 대상은 아니다.
- 네 게이트(`fmt --check` · `clippy --all -D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all`) rc=0. ★`cargo +beta clippy --all -- -D warnings` 는 **rc=101** 이다 — `src/window.rs:217` 의 `unnecessary_as_slice`(beta 1.101 새 린트). 이 회차가 건드리지 않은 코드이고(이 PR 에 Rust 변경 0), 열린 PR #591(`wie-beta-clippy-red-window-as-mut-slice`)이 고친다. PR 의 beta 다리는 빈 통과라 PR green 은 이것을 덮지 않는다.
- 착수 뒤 main 에 #585(0552)가 착지해 `compat.json` 과 계약을 건드렸다. 푸시 전이라 그 main 위로 옮겨 다시 생성했다(수치 같음 · `check-compat-revert` 로 확인).

### 검수 뒤 고침 (2026-11-22 · wie-featurephone-compat-game-key-normalize-and-aliases-fix)

게이트② 반려(핀 `409008a5`)의 R2 를 총괄 결정 ⒜안으로 고쳤다. R1(셸 `lib/games.test.ts`)은 otterpebble 티켓 `otterpebble-featurephone-games-test-accepts-normalized-game-keys` 의 몫이고, 그 착지가 이 PR 착지의 선행 조건이다.

**무엇이 틀렸나**: 첫 판은 `same-game` 묶음 안에서 같은 `variant` 를 허용했다. 그런데 `variant` 는 다행 묶음에만 실리고 현 데이터의 다행 묶음 26개는 전부 `same-game` 이라, 예외의 범위가 규칙의 범위 전체였다. 묶음의 모든 행을 한 라벨로 바꿔도 26/26 통과했고, 작은화면 판(다른 본체)을 «큰화면» 으로 바꿔도 통과했다. 위 문단에 적었던 «셸의 행 열쇠는 sha 라 겹쳐도 안전하다»도 틀렸다 — 셸 테스트(`lib/games.test.ts:147`)는 이 묶음에서 화면 라벨 행 2 · 해시 라벨 행 1 을 기대한다.

**고친 것**
- `ab64a56b2b44` 의 손 라벨을 지워 `variant: null` 로 두고, 검사기의 예외와 `retitle` 의 `handLabel` 배관을 지웠다. `validateCompat` 의 그 줄은 #588(main)과 같은 문장이다. 데이터 변화는 그 한 행의 `variant` 뿐이다(`큰화면` → `null` · main 과 같은 값).
- selftest 에 개악 «one label on two files of a same-game group»(same-game 쌍의 두 행에 `큰화면`)을 넣었다 — 거부돼야 한다.
- check(`node scripts/player-data.mjs`)가 커밋된 행을 `retitle()` 결과와 행 단위로 대조한다(고정점). 이름 규칙을 바꾸고 `normalize` 를 안 돌린 경우와 손 편집이 여기서 걸린다.
- 기존 결함 하나(검수 F6): 스크립트를 심링크 경로(`/var/…` ↔ `/private/var/…`)로 부르면 주 진입 가드의 두 표기가 어긋나 **출력 없이 rc=0** 으로 끝났다. 양쪽을 실경로로 바꿔 비교한다. selftest 가 심링크로 자기를 불러 확인한다(없는 명령 → rc=2).
- 하이픈 규칙(③)은 실데이터에 걸리는 묶음이 없어 고정점 검사로는 안 잠긴다. selftest 단언 하나(`a-b` ↔ `ab`)를 넣었다.

**개악 표** — 스크립트 사본의 호출부 한 줄씩 · 각 1회 · 실데이터

| 개악 | selftest | check | 비고 |
|---|---|---|---|
| m4 `retitle` 이 표기를 안 맞춤 | rc=1 | **rc=1** `compat.entries[0]: differs from what retitle() derives` | |
| m5 대표 표기 ①(대문자) 제거 | 0 | **rc=1** `entries[51]` | 첫 판에서는 둘 다 0 |
| m6 ②를 뒤집음(덜 띄어 쓴 것) | 0 | **rc=1** `entries[0]` | 첫 판에서는 둘 다 0 |
| m7 ③을 뒤집음(하이픈 많은 것) | **rc=1** `with capitals and spaces equal the spelling with fewer hyphens wins` | 0 | 실데이터를 안 바꾸는 개악이라 selftest 가 잠근다 |
| m8 `SAME_GAME` 조회를 정규화 안 함 | 0 | **rc=1** `entries[8]` | 첫 판에서는 둘 다 0 |
| m9 손 라벨 생존 규칙을 옛 식으로 | — | — | 해당 없음: ⒜안에서 그 «옛 식»이 곧 현재 코드다(손 라벨 생존 규칙 자체를 지웠다) |
| m9″ `ab64a56b2b44` 손 라벨을 되살림 | 0 | **rc=1** `entries[119]` | 첫 판의 데이터로 돌아가려는 편집 |
| m9′ 라벨 유일 조건 제거(겹쳐도 싣기) | 0 | 0 | ★안 잠긴다: 현 데이터에 파일 이름 라벨이 겹치는 묶음이 없다. 겹치는 데이터가 생기면 `validateCompat` 의 유일 규칙이 거부한다 |
| w1 유일 규칙 끔 | **rc=1** `NOT rejected — same title and variant twice` | 0 | |
| w1 `same-game` 예외를 되살림 | **rc=1** `NOT rejected — one label on two files of a same-game group` | 0 | 첫 판의 예외 |
| 고정점 검사를 끄고 m4 | rc=1 | 0 | 새 한 줄이 m4 를 check 에서 잡는 그 줄이라는 대조 |

실데이터 개악(head `validateCompat`): 작은화면 판 `174237758542` 의 `variant` 를 `큰화면` 으로 → `compat.entries[119]: (platform, title, variant) "KTF\t붕어빵타이쿤3\t큰화면" repeats` **REJECT**. 폭 탐침(다행 묶음의 모든 행을 한 라벨로): 26묶음 중 **REJECT 26**(첫 판 0).

**착지 사전 증거** — 고친 head `03ba112c` 를 wie `origin/main` `8daeedaf` 에 merge 한 스크래치 트리(`2978b542` = `git merge-tree --write-tree` 결과와 같음)에서 `build --out` → otterpebble `origin/main` `43adcdef` 스크래치 클론에 `node scripts/compat-import.mjs <built>/compat.json`(rc=0 · 429종) → `build-slot run -- vitest run`. 그 뒤 커밋은 문서뿐이다.
- otterpebble 현 main 기준: `Test Files 1 failed | 67 passed (68)` · `Tests 2 failed | 868 passed (870)`. red 는 `lib/games.test.ts:66`(`expected 365 to be less than 365`)과 `:137`(옛 표기 행을 실데이터에서 찾는다) 2건이고 둘 다 그 otterpebble 티켓의 몫이다. **`:147` 은 green**(첫 판에서는 `expected 3 to be 2`). 대조군: import 전 같은 스크래치에서 `lib/games.test.ts` 10/10.
- 그 otterpebble 티켓의 PR head 기준: 잴 대상이 없다 — 측정 시각에 열린 PR 0 · 원격 브랜치 0. 그 PR 이 열리면 같은 경로로 다시 재야 한다.

**착지 뒤 `/support` 예측**(셸 `support-counts-check.mjs` 의 묶음 식으로 전 = otterpebble main `lib/compat.json` · 후 = 위 빌드)
- 게임 수 368 → **365** · 통신사별 대표 수 403 → **402**(KTF 246 → 245 · LGT 77 · SKT 80) · 사이트맵 `/support/<id>` 403 → 402.
- 파일이 둘 이상인 줄 54 → 57 · 통신사를 넘는 줄 35 → 37 · `variant` 실린 행 16 → 16.
- 보이는 제목이 바뀌는 행 13(검수 표와 같은 13행 — 첫 판과 같다). `status` 변경 0 · 행 429 → 429.
- 대표가 바뀌는 곳 2: KTF KBO프로야구2009 의 대표 `cdd3eb5f91` → `4892a1abc0` · KTF `f981d228b7` 는 대표에서 빠져 `4a4d2ac046` 묶음으로 들어간다. 그 두 행은 묶음이 되면서 파일 라벨 `파일 #4a4d2a` · `파일 #f981d2` 가 새로 보인다.
- 붕어빵타이쿤3 KTF 세 행의 파일 라벨: `작은화면` · `큰화면` · `파일 #ab64a5` — 착지 전과 같다.

**게이트**: 네 게이트 rc=0(이 PR 에 Rust 변경 0). `cargo +beta clippy --all -- -D warnings` 는 이 브랜치에서 **rc=101** — `src/window.rs:217` 한 건이고, main 에 착지한 #591(`8daeedaf`)이 고친 자리다. 이 브랜치는 그 main 을 받지 않았다(merge 결과의 beta 는 따로 재지 않았다).

- 게임 이름 유입(`corpus-name-inflow`): BOUNDED INFLOW_B · SUFFIX-ATTACHED INFLOW_S. 의도한 유입이다 — 공개 목록(`compat.json` 의 `title`·`game`·`fileTitle`)이 산출물이고, 이 문서의 판정 표와 `SAME_NAME_HAND` 주석이 묶음 이름을 적는다. 게임 바이트·리소스 내용·캡처는 0.
