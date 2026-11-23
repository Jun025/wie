## [2026-11-24] 공개 게임 목록에 남은 «이름이 거의 같은» 2쌍 판정 — 한 쌍은 같은 게임이라 한 줄로, 한 쌍은 판정 보류 (wie-featurephone-compat-residual-near-name-pairs-classify)

**무엇을**: `scripts/player-data.mjs` 의 `SAME_NAME_HAND` 에 한 행(`d1fba7ad5d1f` → «영웅전설3 하얀마녀»), `SAME_GAME` 에 그 묶음 하나를 더하고 `normalize` 로 `docs/player-data/compat.json` 을 다시 만들었다. KTF «영웅전설3» 과 «영웅전설3 하얀마녀» 가 한 `game` 이 된다. 다른 한 쌍(KTF «삼국쟁패2» ↔ SKT «삼국쟁패2열왕전기»)은 `undetermined` 로 두고 데이터를 바꾸지 않았다.

**왜**: 운영자 지시(2026-11-10 「동일한 게임 중복 표시 정리」). 0560 은 티켓의 쌍과 그 쌍이 닿은 행만 쟀고 근접 이름 전체는 훑지 않았다. 총괄이 이름 365개를 전부 쌍으로 대조해 이름만으로 가를 수 없는 2쌍을 남겼고, 이 회차가 그 2쌍과 나머지 55쌍의 서술자 이름을 쟀다.

**사용자 영향**: 지원 게임 목록에서 KTF «영웅전설3» 과 «영웅전설3 하얀마녀» 두 줄이 «영웅전설3 하얀마녀» 한 줄(파일 2개)이 된다. 행은 지우지 않았다(429 → 429 · sha256 불변). 원 이름은 `fileTitle` 에 남아 검색에 걸린다.

### 판정 — 메타데이터만(게임 바이트·리소스 내용은 인용하지 않음)

방법은 0555·0560 과 같다: `__adf__`/`.msd` 의 PID·Name·Ver·DisplaySize, 아카이브 항목 이름과 내용 해시의 일치 수(해시 값은 비교만 했다), 본체 jar 해시 일치 여부. 코퍼스는 `game_lab/`(git-ignored)이다.

| 쌍 | sha 앞 12자 | 판정 | 근거 | 결과 title / game / variant |
|---|---|---|---|---|
| A · KTF 영웅전설3 ↔ KTF 영웅전설3 하얀마녀 | d1fba7ad5d1f · b907b0faf483 | same-game | PID PD004460 · Name «영웅전설3» · Ver 01.00.02 · 실행 클래스 같음 · 본체 jar 항목 244 ↔ 244 중 같은 이름 243 이 전부 내용까지 같음 · 다른 한 항목은 클라이언트 이미지(이름과 내용이 다르다 — 그래서 jar 해시는 다르다) · AID 다름 | 양쪽 영웅전설3 하얀마녀 / 영웅전설3 하얀마녀 / null · null |
| B · KTF 삼국쟁패2 ↔ SKT 삼국쟁패2열왕전기 | 30c7bd6fb01b · 2d66945008c1 | undetermined | KTF `__adf__` Name «삼국쟁패2-전사편» PD005126 176*220 ↔ SKT `.msd` MIDlet-Name «삼국쟁패2 열왕전기» · 실행 클래스 다름 · 본체 jar 의 같은 이름 항목 1(내용 같음 0) · 소리 파일은 내용 34종 ↔ 34종 중 34 가 같음 · 소리 밖에서 내용이 같은 항목 0 | 그대로 |

**쌍 A 의 이름**: 두 파일의 서술자 Name 은 «영웅전설3» 이고 부제는 두 파일의 서술자 설명(`Desc`)에 있다. 목록은 이미 같은 시리즈를 «영웅전설4 주홍물방울» 로 적고 있어 부제를 단 표기로 맞췄다. 0560 의 «영웅서기» → «영웅서기 1» 과 같은 방향이다(더 구체적인 목록 표기).

**쌍 A 의 `variant` 는 둘 다 `null`**: 판 번호가 같다(01.00.02). 화면 크기는 `d1fba7ad5d1f` 만 `DisplaySize` 240*320 을 적고 `b907b0faf483` 은 그 줄이 없다. 없는 쪽을 «작은화면» 이라고 할 근거가 없고(엔진은 그 줄이 없으면 화면을 바꾸지 않는다 — `wie-ktf/src/emulator.rs` 의 `adf.display_size`), jar 의 리소스 243항목이 두 파일에서 같다. 한쪽에만 «큰화면» 을 달면 다른 쪽이 작은 화면이라는 뜻이 되므로 달지 않았다. 셸은 두 행을 해시 라벨(`파일 #…`)로 가른다. 두 파일의 차이는 클라이언트 이미지와 서술자의 배포 칸(한쪽에만 `ResizeType`·`OffRun`·`IdleType`, 다른 쪽에만 `isOnRom`·`FeeInfo`·`DcRate`)이다.

**쌍 B 를 보류한 이유**: 같은 작품이라는 근거는 있다 — 소리 34종이 전부 같고, 같은 제작사의 전작(SKT «삼국쟁패 패왕전기» `2f5246006bd8`)과는 34종 중 1종만 같다. 그러나 KTF 파일은 자기를 «전사편» 이라고 부르고, 그 서술자 설명은 이 작품이 캐릭터 네 가지로 나뉘어 있다고 적는다. SKT 파일의 이름에는 편 표기가 없다. KTF 파일이 넷으로 나눈 판 가운데 하나라면 목록의 선례(메이플스토리 전사편·마법사편·궁수편은 편마다 따로 싣고, 통신사가 달라도 같은 편끼리만 묶었다 — 0555·0560)로는 SKT 전체판과 한 줄이 아니다. SKT 파일이 네 캐릭터를 다 담았는지는 메타데이터로 알 수 없다(리소스 형식이 달라 이름도 내용도 겹치지 않는다). «나눈 판을 전체판과 한 줄로 묶을지» 는 목록 정책이라 운영자 결정으로 남긴다.

### 57쌍 재추출과 나머지 55쌍

식: `game` 365개의 모든 쌍에 대해 `n(s)` = NFKC · 소문자 뒤 글자와 숫자만 남긴 것(유니코드 범주 L·N)으로 두고, `difflib.SequenceMatcher(None, n(a), n(b)).ratio() ≥ 0.8` 이거나 한쪽이 다른 쪽에 포함되면 뽑는다. wie `origin/main` 의 `compat.json` 과 otterpebble `63ee481a9` 의 `apps/featurephone/lib/compat.json` 양쪽에서 **57쌍**(비율로만 34 · 포함 23)이 나왔다(두 파일의 (sha256, game, title, variant) 는 429행 전부 같다). 숫자를 빼고 글자만 남기면 80쌍이라 총괄의 수와 맞지 않는다.

57쌍마다 양쪽 `game` 의 모든 행을 서로 대어 서술자 Name(`Name` 또는 `MIDlet-Name`)이 그대로 같거나 `n()` 으로 같은지 봤다. **티켓 2쌍 밖에서 Name 이 같은 쌍은 0쌍**이다. PID 가 같은 쌍도 쌍 A 뿐이고 본체 jar 가 같은 쌍은 없다. 나머지 55쌍에서 같은 이름 항목 중 내용까지 같은 수가 가장 큰 것은 메이플스토리 마법사편 ↔ 전사편의 73(같은 이름 219)이고, 0555 가 이미 different-game 으로 판정한 쌍이다.

### 수치

- 행 429 → 429 · sha256 집합 같음 · 이름 칸(`title`·`game`·`variant`·`sameTitle`) 밖의 변경 0(sha 별 대조) · 최상위 칸 같음.
- distinct `game` **365 → 364** · (platform, game) 402 → 401.
- 바뀐 행 2: `d1fba7ad5d1f` 는 `title`·`game` «영웅전설3» → «영웅전설3 하얀마녀» 와 `sameTitle` 신설(`same-game`), `b907b0faf483` 은 `sameTitle` 신설만. `title` 이 바뀐 행은 1. `same-game` 53 → 55행 · `variant` 실린 행 16 → 16.
- 묶음의 대표 행은 `b907b0faf483` 이다(여섯 축 `ok` 수는 둘 다 6 · 공개 산출물에서는 `changes` 2 ↔ 1 로, 커밋 파일에서는 sha 순으로 앞). 그 행의 `progress` 는 `stuck`, 묶음에 들어간 `d1fba7ad5d1f` 는 `ok` 다 — 대표 규칙(계약 §1)은 `progress` 를 보지 않는다. 규칙은 이 회차 범위 밖이라 그대로 뒀다.

### 검사

- `node scripts/player-data.mjs` → `player-data OK — 429 titles · status {"playable":426,"not-yet":2,"limited":1} · platform {"KTF":269,"LGT":78,"SKT":82} · 187 updates` rc=0.
- `node scripts/player-data.mjs --selftest` → `player-data selftest: 67 rules each reject their mutation` rc=0.
- 착지 사전 증거: 이 브랜치의 데이터로 `build --out` → otterpebble `origin/main` `63ee481a9` 스크래치 클론에 `node scripts/compat-import.mjs <built>/compat.json`(rc=0 · 429종) → `apps/featurephone` 에서 `vitest run`: `Test Files 71 passed (71)` · `Tests 955 passed (955)` rc=0. 넣기 전 대조군도 71 · 955. 셸 `lib/compat.json` 에서 바뀌는 행은 위 2행뿐이다. 셸의 시각 회귀(playwright)는 돌리지 않았다 — 그 기준의 게임 상세 대상은 `6e5a1f88f0c0` 으로 이 회차가 바꾼 행이 아니다.
- 규칙·검사기 코드는 바꾸지 않았다(표 두 줄뿐). 이 PR 에 Rust 변경 0.
- 네 게이트(`fmt --check` · `clippy --all -D warnings` · wasm clippy · `RUST_MIN_STACK=4194304 cargo test --all`) rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(PR 의 beta 다리는 빈 통과라 이 로컬 실행이 beta 의 유일한 근거다).

- 게임 이름 유입(`corpus-name-inflow`): BOUNDED 351쌍(1093회) · SUFFIX-ATTACHED 17쌍(54회). 의도한 유입이다 — 공개 목록(`compat.json` 의 `title`·`game`)이 산출물이고, 이 문서의 판정 표와 `SAME_NAME_HAND` 주석이 묶음 이름을 적는다. 게임 바이트·리소스 내용·캡처는 0.


