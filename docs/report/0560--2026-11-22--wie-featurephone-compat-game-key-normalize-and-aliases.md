## [2026-11-22] 공개 게임 목록의 `game` 키 — 한 게임에 표기 하나 · 이름이 다른 같은 게임은 메타데이터로 맞춤 (wie-featurephone-compat-game-key-normalize-and-aliases)

**무엇을**: `scripts/player-data.mjs` 의 `retitle` 이 «정규화(`gameKey` = NFKC · 소문자 · 공백과 하이픈 제거)하면 같은 이름»을 통신사와 무관하게 같은 `game`·`title` 로 내보낸다. 묶음의 표기는 대표 표기 규칙으로 고른다(①대문자가 많은 것 ②띄어 쓴 곳이 많은 것 ③하이픈이 적은 것 ④문자 코드 순). `validateCompat` 은 정규화하면 같은데 `game` 이 갈린 묶음을 거부한다. 이름이 아예 달라 정규화로 못 만나는 같은 게임 3건과 라벨 없는 판 1건은 메타데이터로 판정해 `SAME_NAME_HAND` 에 적었다. 계약 = `docs/contracts/featurephone-public-data.md` §1.

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
| KTF 붕어빵타이쿤3 라벨 없는 판 | ab64a56b2b (↔ 1742377585 작은화면 · 5bfe10a828 큰화면) | same-game · 큰화면 판과 같은 본체 | PID PD004489 같음 · 240*320·Ver 01.00.01 = 큰화면 판(작은화면 판은 176*220·01.00.02) · 본체 jar 가 큰화면 판과 같음(358항목 중 357 같음 · AID 만 다름) | 붕어빵타이쿤3 / 붕어빵타이쿤3 / 큰화면 |

`undetermined` 는 0건이다.

**티켓 쌍 밖 2건을 함께 적은 이유**: 근접 이름 쌍을 재던 프로브가 두 행의 서술자를 그대로 보여 줬다 — «물가에 돌튕기기»는 자기 `__adf__` 가 3편이라고 말하고, SKT «영웅서기»는 KTF «영웅서기 1» 과 같은 부제·같은 리소스다. 재 놓고 틀린 이름을 남기는 쪽이 더 나쁘다고 봤다. 근접 이름 전체를 훑은 것은 아니다(티켓의 4쌍과 그 쌍이 닿은 행뿐).

**붕어빵타이쿤3 — 같은 라벨 둘**: 티켓은 «같은 크기의 다른 판이면 판 번호»를 예상했지만, 재 보니 크기도 판 번호도 본체도 큰화면 판과 같다(설치 칸 AID 만 다른 재포장). 참인 라벨은 «큰화면» 하나뿐이라 두 행이 그 라벨을 나눠 가진다. 그래서 «(platform, title, variant) 유일» 규칙에 예외 하나를 뒀다: `same-game` 묶음 안의 손 라벨만 겹칠 수 있고, 그 밖의 겹침은 여전히 거부한다(selftest 양쪽). 셸의 행 열쇠는 sha 라 겹쳐도 안전하다(`lib/games.ts` `fileLabel`).

### 수치

- 행 429 → 429 · sha 집합 같음 · 이름 필드(`title`·`game`·`variant`·`sameTitle`) 밖의 변경 0(sha 별 대조).
- distinct `game` **378 → 365**(정규화 9묶음에서 −10 · 별칭 −3) · (platform, game) 406 → 402 · 정규화 충돌 묶음 **9 → 0**.
- `game` 이 바뀐 행 13 · `sameTitle` 만 새로 실린 행 3 · `variant` 만 바뀐 행 1. `same-game` 45 → 53행 · `variant` 실린 행 16 → 17.
- 대표 표기로 고른 9묶음: 2006 독일축구 · 던전앤파이터 귀검사 · 드래곤나이트EX · 메이플스토리 궁수편 · 메이플스토리 전사편 · 일지매 영웅전기 · 크로노스 윙 · 현영맞고 2006 · KBO프로야구2009.

### 검사

- `node scripts/player-data.mjs --selftest` → 45 rules. 개악 «one game spelled two ways»(한 게임을 `t` / `T` 로) — 규칙을 끄면 `selftest: NOT rejected — one game spelled two ways` rc=1, 켜면 rc=0.
- 실데이터 개악(한 행을 옛 표기로 되돌림) → `compat: game "크로노스윙" is spelled "크로노스윙" / "크로노스 윙" — one game, one key`.
- `build --out` 산출물을 otterpebble `origin/main` `a286be3e` 의 `compat-import.mjs` `validateCompat` 로 읽기 전용 실행 → 위반 0 · 429행. 셸 `normalizeGameKey` 로 묶은 수 = `game` 그대로 묶은 수 = 365(셸 정규화가 더 합치는 묶음 0). 셸 정렬 뒤 대표 행이 바뀐 묶음 0/402.
- 정규화로만 합쳐진 통신사 사이 6묶음도 서술자를 봤다. Name 이 서로 같거나, 한쪽이 부제를 빼거나 연도 자리만 달리 쓴 정도이고 다른 게임을 가리키는 묶음은 없다. 이 6묶음은 이름 규칙으로 합친 것이라 판정 표의 대상은 아니다.
- 네 게이트(`fmt --check` · `clippy --all -D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all`) rc=0. ★`cargo +beta clippy --all -- -D warnings` 는 **rc=101** 이다 — `src/window.rs:217` 의 `unnecessary_as_slice`(beta 1.101 새 린트). 이 회차가 건드리지 않은 코드이고(이 PR 에 Rust 변경 0), 열린 PR #591(`wie-beta-clippy-red-window-as-mut-slice`)이 고친다. PR 의 beta 다리는 빈 통과라 PR green 은 이것을 덮지 않는다.
- 착수 뒤 main 에 #585(0552)가 착지해 `compat.json` 과 계약을 건드렸다. 푸시 전이라 그 main 위로 옮겨 다시 생성했다(수치 같음 · `check-compat-revert` 로 확인).
- 게임 이름 유입(`corpus-name-inflow`): BOUNDED 359쌍(1114회) · SUFFIX-ATTACHED 18쌍(56회). 의도한 유입이다 — 공개 목록(`compat.json` 의 `title`·`game`·`fileTitle`)이 산출물이고, 이 문서의 판정 표와 `SAME_NAME_HAND` 주석이 묶음 이름을 적는다. 게임 바이트·리소스 내용·캡처는 0.


