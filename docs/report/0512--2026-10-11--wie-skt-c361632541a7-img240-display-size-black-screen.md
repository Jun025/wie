## [2026-10-11] SKT `c361632541a7` — 캔버스 높이로 그림 묶음을 고르는데 `img_240/` 이 없어 화면이 검다 · 화면 176x186 을 jar MD5 로 (wie-skt-c361632541a7-img240-display-size-black-screen)

**무엇을**: `c361632541a7` 의 화면 크기를 176x186 으로 띄운다. `wie-skt` 에 jar MD5 로 찾는 덮어쓰기 표 한 줄을 더했다(`DISPLAY_OVERRIDES`).
**왜**: 게임이 «캔버스 높이»만 보고 그림 묶음 디렉터리를 고른다. 240x320 에서는 `/img_240/` 을 찾는데 jar 에는 `img_176/` 63개뿐이다. 그래서 그림마다 `FileNotFoundException` 이 나고 글상자 말고는 전부 검다.
**사용자 영향**: 시작 화면과 거리 배경, 가게 안·메뉴·요리사·손님 화면이 뜬다. 예외는 판마다 약 1,700 → 0 이다. 600초 진도 축은 아직 `stuck` 이다(§4).

증적은 스크래치에만 있다(게임 바이트·화면은 커밋하지 않는다). 타이틀은 sha12 로만 적는다.

### 1. 화면 크기를 어디서 읽나 — SKT 에는 선언이 없다

- msd 에 크기 항목이 없다(KTF `__adf__` 의 `DisplaySize:` 에 해당하는 줄이 없다). jar 매니페스트에도 없다.
- `CGame` 생성자(javap): `buffer = Image.createImage(getWidth()+1, getHeight()+16)` 다음에 `Height = buffer.getHeight()`. 그리고 `Height > 250` 또는 `> 212` 면 `/img_240/`, `> 166` 이면 `/img_176/`, 아니면 `/img_120/` 이다. ⇒ **캔버스 높이 151..196 일 때만 `img_176/`** 이다. 폭은 보지 않는다.
- 같은 판정이 배경에도 있다. `Height` 가 166..212 면 `img_176/mbg176.png` 를 `(88, 101)` 중심에 그린다. 그 그림은 **176x202** 다. ⇒ `Height` 202 = 캔버스 186. 이 단말의 화면은 220 판에서 상태줄 18 · 소프트키 줄 16 을 뺀 것으로 읽힌다.
- 0425 KTF 처방과의 관계: KTF 는 패키지가 크기를 **선언**하고 엔진이 그것을 따른다. SKT 는 선언이 없다. 그래서 기존 `package_display_size` 는 «jar 가 가진 그림 폭»으로 추정한다(2026-10-02 · `bf54c05e58a9` `78bd51675574`). 이번 타이틀도 같은 규칙으로 잡히게 고친 것이 첫 시도다(§2). 규칙은 같다: «패키지가 말하는 크기를 따른다». 다른 것은 «어디서 읽느냐»뿐이다.

### 2. 첫 시도 둘과 버린 이유

| 시도 | 결과 |
|---|---|
| ① 기존 규칙을 디렉터리 형 `<stem>_<width>/` 까지 넓힘 → 176x220 | 크기는 바뀌었으나 캔버스 220 → `Height` 236 > 212 → 여전히 `img_240/` · 검은 화면(예외 1,437) |
| ② ① + 176 칸 높이를 220 → 186 | 이 타이틀은 뜬다. 그러나 **`bf54c05e58a9` 의 아래 대화상자가 잘린다** — 그 타이틀은 220 높이에 대화상자를 놓는다(600초 판 t100 화면 대조). 같은 176 단말끼리도 캔버스 높이가 달랐다 |

②의 비용이 실측돼 «폭당 높이 하나»로는 두 타이틀을 다 맞출 수 없다. 패키지 안에서 높이를 일반적으로 읽어 낼 근거(API·선언·그림)를 찾지 못했다. 두 jar 모두 `Canvas`·`XDisplay` 를 쓰고 fullscreen API 는 없다. 그래서 이 타이틀 한 줄을 jar MD5 로 덮어쓴다. 같은 꼴의 선례는 `wie-core-arm/data/binary_patches.toml` 의 MD5 키다. ①의 디렉터리 형 규칙은 말뭉치에서 이 타이틀 하나만 잡는다. 덮어쓰기가 그 하나를 이미 덮으므로 **넣지 않았다**(말뭉치 스캔: 크기 규칙에 걸리는 SKT 는 기존 2 + 디렉터리 형 1).

### 3. 전/후

바이너리: 전 = `origin/main 6ea6a6bd` release, 후 = 이 브랜치 release(덮어쓰기판). census 를 `scripts/census-drive.sh` 로 돌렸다(phase 마다 `build-slot --long` 임대 · `--jobs` 기본).

**대상과 가드(6축 + progress 600초 · P/P2)**

| sha12 | 6축 전 → 후 | progress 전 → 후 | 예외(A/B/L) 전 → 후 | 비고 |
|---|---|---|---|---|
| `c361632541a7` SKT | 같다(sound `silent` 는 0440 의 게임 안 소리 끔 설정) | stuck(stall 230 · 새 화면 3) → **stuck**(stall 440/450 · 새 화면 **16**) | 94/1431/177 → **0/0/0** | B(무키) FAIL «black screen» → **PASS** · 마지막 프레임 색 2 → 123 |
| `bf54c05e58a9` SKT | 같다 | ok → ok | 0 → 0 | 176x220 그대로(덮어쓰기 대상 아님) |
| `78bd51675574` SKT | 같다 | ok → ok | 0 → 0 | 120x160 그대로 |
| `49ade89578c5` 가드 | 같다 | — | 0 → 0 | |
| `ddd885583b15` 가드 | 같다 | — | 0 → 0 | |

6축이 대상에서 안 바뀐 것은 축의 한계다. render 는 «2색 이상 프레임 하나»라 글상자만으로도 `ok` 였다. 바뀐 것은 위의 B 판정·예외·색 수다.

**라이브 SKT 등재분 77종 짝(census probe A/B · 30초)**: 전 A PASS 77 · B PASS 76/FAIL 1 → 후 A 77 · B 77. 결과·content·마지막 프레임 content·예외 수가 달라진 행은 **대상 1종 2행뿐**이다(A 예외 97 → 0 · B FAIL → PASS). 나머지 76종은 네 값 모두 같다.

### 4. 남은 벽 — «메뉴 개발 완료» 대화상자

후 P 판의 화면 순서: 가게 안 → 메뉴 목록 → 요리사·손님 주문 → 홀 → **160초부터 «개발 완료» 대화상자**에서 끝까지 같은 화면이다. P2 도 같다(stall 450). 예외 0 · uncaught 0 이다. 정책 키(OK·5·좌우)가 이 대화상자를 넘기지 못하는 이유는 재지 않았다. 정책 한계인지 엔진 벽인지 모른다. 그래서 카드로 올리지 않았다(worklog `notCarded`).

### 5. 게이트

- `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · wasm32 clippy rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 통과 776 · 실패 0 · `npm run build:wasm` rc=0. 모두 rebase 전(`6ea6a6bd` 위) 값이다.
- 러너 블록(release): draw · helloworld ×2 · text PASS. keydraw ×2 는 `--max-ticks 1e9` 에서 PASS 27/27 · rc=0 이다(기본 상한에서는 UNMEASURED `max-ticks` — 0425 와 같은 조건).

### 6. 되돌리면 red

| 수정 | 시험 | 되돌림 결과 |
|---|---|---|
| 덮어쓰기 행 (176, 186) | `wie-skt` `a_title_whose_code_needs_another_canvas_gets_it_by_jar_hash` | FAILED(`Some((176, 220))` ≠ `Some((176, 186))`) |
| 덮어쓰기 전체 | 전 바이너리 = 덮어쓰기 없음 | B FAIL «black screen» · 예외 1,437(§3) |

시험이 실제 jar 를 쓰지 않는 이유: 게임 바이트는 커밋할 수 없다(Constraint 9). 그래서 MD5 → 크기 조회만 단언한다.

### 7. 측정 조건

- `host-load-guard --status --recovered` 는 시작 때 rc=0 이었다(idle 19~50% · load 9). 회차 중 load1 은 6~20 이다. census 행의 `load1` 은 7~11 이다.
- 77종 짝의 «후» 판은 다른 레인의 census 락을 약 45분 기다렸다(census 가 기다림 · 실패 아님).
- 처음 «후» 6축 판은 시도 ②의 바이너리로 돌았다. 그 결과는 지우고 덮어쓰기판으로 다시 쟀다. 위 표는 모두 덮어쓰기판이다. 단 77종 짝의 «후»는 처음부터 덮어쓰기판이다.
- `nohup … &` 는 쓰지 않았다. 회차 끝에 자기 프로세스는 0 이다.

### 8. compat.json · 소식

`compat.json` 은 바꾸지 않았다. 6축과 `progress` 가 전·후 같다(§3). 소식 `docs/player-updates/2026-10-11-skt-small-screen-art.json` 한 줄을 넣었다. 원장 `0507 …--ledger.csv` 의 해당 행을 갱신했다.

### 9. 게임 이름 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치): 유입 0건(BOUNDED) · 판단 필요 0건(SUFFIX-ATTACHED). 새로 쓴 주석·문서·소식은 sha12 만 쓴다.
