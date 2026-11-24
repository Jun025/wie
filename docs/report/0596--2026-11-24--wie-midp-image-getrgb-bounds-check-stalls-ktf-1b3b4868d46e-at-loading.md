## [2026-11-24] KTF 게임 1개(`1b3b4868d46e`)가 로딩 화면에서 멈추던 것 — WIPI `getRGBPixels` · `setRGBPixels` 의 `bpl` 을 바이트로 읽는다 (wie-midp-image-getrgb-bounds-check-stalls-ktf-1b3b4868d46e-at-loading)

**무엇을** — `org.kwis.msp.lcdui.Graphics` 의 `getRGBPixels` 와 `setRGBPixels` 가 `bpl` 인자를 «한 줄의 바이트 수»로 읽는다(`bpl / 4` 가 한 줄의 int 수). upstream 과 같은 값이다. `wie-midp` 의 `Image::getRGB` 경계 검사는 고치지 않았다.

**왜** — 0595 §3 이 올린 제안(`2026-11-24-compat-longplay-reopen-filled-57-rows-remeasure#p0`). `1b3b4868d46e` 가 로고 다음 로딩 화면에서 600초 내내 멈춰 있었다. 티켓은 `Image::getRGB` 의 경계 검사를 급소로 적었는데, 재 보니 그 검사는 옳고 그 검사에 틀린 줄 간격을 넘기는 호출부가 원인이었다(§1).

**사용자 영향** — 그 게임이 로딩 화면을 넘어 메인 메뉴로 들어가고 소리가 난다. 지원 목록의 그 행에서 «소리가 나지 않을 수 있어요.» 가 빠진다(§4). `setRGBPixels` 를 두 줄 이상으로 부르는 다른 KTF 게임 1개(`d9afc4db742c`)는 국기 · 배경 그림이 빠져 보이던 것이 다시 보인다(§3 · 그 행의 `compat.json` 은 다시 재지 않았다).

### 1. 원인 — 예외는 `Image::getRGB` 에서 나지만 틀린 것은 호출부의 단위다

- 30초 탐침(고치기 전 빌드 `db18853d` · 디버그 로그): `ArrayIndexOutOfBoundsException: destination is too small` 254건. 그 254건 전부가 `Graphics.getRGBPixels(0, 0, 127, 64, pixels, 0, 508)` → `Image.getRGB(pixels, 0, 508, 0, 0, 127, 64)` 한 가지 호출이다. 배열 길이는 15000 이다(임시 로그 한 줄로 잼 · 커밋하지 않음).
- `508 = 127 × 4`. 게임은 `bpl` 을 바이트로 넘긴다. 우리 `getRGBPixels` 는 그 값을 int 개수로 읽어 줄 간격 508 로 `getRGB` 에 넘겼다. 그러면 마지막 줄이 `63 × 508 = 32004` 에서 시작해 길이 15000 을 넘는다. MIDP 사양대로 예외가 맞다.
- 줄 간격을 127 로 읽으면 `64 × 127 = 8128` 칸을 쓰고 15000 안에 든다.
- 왜 #617 뒤에 생겼나: 병합 전 우리 `getRGBPixels` 는 아무것도 하지 않는 스텁이었다(예외 0 · 화면 29/29 였던 9~10월 보관 기록이 그 상태다). S3 병합이 upstream 의 실구현을 받으면서 `bpl` 만 «우리 `setRGBPixels` 와 같은 단위(int 개수)»로 맞췄다(0586 부록 B). upstream 은 두 메서드 모두 `bpl / 4` 다.
- 0586 은 우리 단위를 «`a540945188ca` 로 잰 값»이라고 적었다. 그 게임은 `setRGBPixels` 를 높이 1 로만 부른다(§3). 높이 1 이면 줄 간격을 읽지 않으므로 그 측정으로는 단위를 가를 수 없었다.
- WIPI C 쪽 같은 함수(`MC_grpGetRGBPixels` · `MC_grpSetRGBPixels`)도 줄 간격을 바이트로 센다(`wie-wipi-c` `primitives.rs` 가 `row × bpl` 바이트 주소로 쓴다).

### 2. 처방과 시험

- `wie-wipi-java/src/classes/org/kwis/msp/lcdui/graphics.rs`: `getRGBPixels` 의 `scanlength = bpl / 4` · `setRGBPixels` 가 `drawRGB` 에 넘기는 줄 간격 `bpl / 4`. 두 자리 모두 upstream `main` 과 같은 식이다. 이 두 자리는 0586 부록 B 가 «upstream 이 우리 쪽 잠재 결함을 고친 것으로 읽힌다»고 적어 둔 자리였고, 이제 다음 upstream 동기화에서 갈리는 축이 아니다.
- `Image::getRGB`(`wie-midp`)는 그대로다. upstream 과 바이트가 같은 검사이고, 느슨하게 하면 배열 밖 쓰기를 조용히 넘기게 된다.
- 새 시험 `test_rgb_pixels_take_bpl_in_bytes`: 실측 인자 그대로(127×64 · 배열 15000 · `bpl` 508) 읽어 62·63째 줄이 `줄 × 127` 자리에 들어가는지 본다. `setRGBPixels` 는 게임이 쓰는 형태(폭 × 2줄 · `bpl = 4 × 폭`)로 둘째 줄이 첫 줄 바로 뒤 int 에서 오는지 본다. 배열이 한 칸 모자라면(8127) 여전히 `ArrayIndexOutOfBoundsException` 이고 배열에 쓴 것이 없어야 한다.
- 기존 시험 `test_get_rgb_pixels_round_trips_through_set_rgb_pixels` 의 `bpl` 은 3 → 12(upstream 시험과 같은 값).
- 되돌려 보기(증적 `mutations.txt`):

| 되돌린 자리 | 결과 |
|---|---|
| `getRGBPixels` 호출부 `bpl / 4` → `bpl` | 2 failed(새 시험 · 왕복 시험) |
| `setRGBPixels` 호출부 `bpl / 4` → `bpl` | 2 failed(같은 둘) |
| `Image::getRGB` 경계 줄 `> length` → `> length + 1` | `wie-wipi-java` 1 failed(새 시험 · 배열에 써졌다) · `wie-midp` 1 failed(기존 `rgb_images_copy_pixels_and_validate_reads`) |
| 원본 | `wie-wipi-java` 3 passed · `wie-midp` 7 passed |

- 처음 만든 셋째 개악(`if false && … || …`)은 연산자 순서 때문에 위쪽 경계 절반이 살아 있어 green 이었다. 잘못 만든 개악이었고 위 형태로 다시 했다(증적에 둘 다 남김).

### 3. 다른 게임의 인자 — 전부 바이트다

코퍼스에서 `setRGBPixels` 문자열을 가진 9종(0422 §1 의 목록) 가운데 디스크에 있는 8종을 고친 빌드로 하나씩 돌려 호출 인자만 셌다(`wie_validate --inject` · 27키 · 약 20초 · 증적 `other-titles-arg-shapes.txt`).

| sha12 | 호출 | `bpl` | 예외 |
|---|---|---|---|
| `1b3b4868d46e`(60초) | `setRGBPixels` 148 · 전부 높이 2 / `getRGBPixels` 2 · 높이 64 | 전부 `4 × 폭` | 0 |
| `d9afc4db742c` | `setRGBPixels` 12 · 높이 2 이상 | 전부 `4 × 폭` | 0 |
| `44c292be4e4c` | `getRGBPixels` 7168 · `setRGBPixels` 7168 · 높이 1 | 전부 `4 × 폭` | 0 |
| `a540945188ca` | `setRGBPixels` 945595 · 높이 1 | 전부 `4 × 폭` | 0 |
| `2c2ba3b84b98` · `65bace1623a7` · `9c1c446a36e2` | `setRGBPixels` 24만~26만 · 높이 1 | 전부 `4 × 폭` | 0 |
| `5891a0c5d595` | 이 20초 안에는 호출 없음 | — | 0 |

- `bpl` 을 폭(int 개수)으로 넘긴 호출은 0건이다.
- 높이 1 인 호출은 줄 간격을 읽지 않아 고치기 전후가 같다. 달라지는 것은 높이 2 이상으로 부르는 `1b3b4868d46e` 와 `d9afc4db742c` 다.
- `d9afc4db742c` 전후 짝(같은 27키 · 고치기 전 `db18853d` 릴리스 빌드 ↔ 고친 빌드 · 하나씩): 고치기 전은 `ArrayIndexOutOfBoundsException: source is too small` 12건(`setRGBPixels` → `drawRGB` 의 같은 단위 문제 · 고친 빌드에서 센 높이 2 이상 호출 수도 12다)이고 팀 고르기 화면의 국기 · 설정 화면의 배경 그림이 빠져 있다. 고친 빌드는 예외 0건이고 그 그림이 보인다. 키 단계 화면 28장 중 18장이 다르다(증적 `d9afc4db742c-old-left-new-right-steps-10-14-20-27.png`). 이 예외도 #617 병합 뒤에 생긴 것으로 읽힌다(`drawRGB` 의 경계 검사가 그 병합으로 들어왔다 · 병합 전 빌드로 재지는 않았다).
- 이 표는 인자 형태만 센 것이다. `UNMEASURED max-ticks` 로 끝난 실행이 넷 있는데 디버그 로그 수십만 줄을 쓰느라 느려진 탓이고, 그 판정은 읽지 않았다. 이 7종의 `compat.json` 행은 다시 재지 않았다(티켓 범위 밖).

### 4. `1b3b4868d46e` 600초 재측과 반영

- 바이너리: `abe853aa9` 트리의 `cargo build --release -p wie_cli --bin wie_validate`. 방법은 0595 와 같다: `bash scripts/census-drive.sh --bin … --out … --titles <file> <코퍼스 링크>` · 줄은 `1b3b4868d46e 600 <recipes-sound 키 순서>` · 탐침 A·B 와 600초 실행을 한 호출(긴 슬롯 임대 1건)로. 게임 파일은 코퍼스 원본을 링크로 읽었다.

| 실행 | 초 | 예외 | 소리 재생 | 화면 | load1 |
|---|---|---|---|---|---|
| 탐침 A(27키) | 30.0 | 0 | 0 | 로고 → 로딩 → 소리 묻기 → 메인 메뉴 → 메뉴 안 안내 화면 | 14.1 |
| 탐침 B(키 없음) | 30.0 | 0 | 0 | — | 13.6 |
| 600초(`recipes-sound` 키 순서 · 키 813/902) | 600.0 · `stop` `deadline` | **0** | **1**(MIDI 이벤트 2487) | 20초 간격 29장 중 서로 다른 것 **28** | 10.3 |

- 0595 의 같은 키 순서 실행은 예외 12924건 · 화면 1/29 · 소리 0 이었다.
- **600초 실행이 머문 자리**: 20초 화면은 로딩, 60초부터 끝까지는 제작진 소개 글이 위로 흘러가는 화면이다. 이 키 순서가 메뉴에서 그 항목으로 들어간다. 서로 다른 28장은 그 글이 흘러간 결과이고, 경기 화면은 이 실행에 없다. §3 의 60초 탐침에서 센 `setRGBPixels` 폭 × 2줄 호출 148회(폭 38 ~ 606)가 어느 화면의 것인지는 가려 보지 않았다.
- `L.json` 의 `result` 는 `UNMEASURED` 다. 키 902개 중 813개가 들어간 채 시한이 왔다는 뜻이고(0595 의 같은 실행도 813/902), census 의 `longplay` 는 `stop` 이 `deadline` 이면 채움으로 읽는다.
- 반영: `playability-census.mjs report`(1행) → `player-data.mjs import-rows` 실제 출력 `import-rows: 1 rows read · 1 changed (1b3b4868d46e=playable) -> docs/player-data/compat.json`. 손 기입 0.
- 바뀐 행 1: `1b3b4868d46e` — `axes.sound` `no` → `ok` · `knownIssues_ko` 한 줄 → `[]`. `status` 는 `playable` 그대로이고 `speed` 는 `unknown` 그대로다(이 실행들에서 읽지 않았다). 행 수 429 불변.
- `node scripts/check-compat-revert.mjs` 의 «착지 기준 바뀐 행»은 회신에 실제 출력으로 적는다.
- 소식: `docs/player-updates/2026-11-24-one-ktf-game-past-loading.json`(두 게임 · 0595 의 «원인을 찾고 있어요» 소식의 뒤를 잇는다).

### 5. 한계

- `d9afc4db742c` 는 27키 · 약 20초 전후 짝까지만 봤다. 600초 실행과 `compat.json` 행 재측은 하지 않았다(그 행은 지금 `playable` · 전 축 `ok` 라 census 로 다시 재도 바뀔 칸이 없다).
- `f80713702111` 은 디스크에 없어 인자를 세지 못했다.
- `1b3b4868d46e` 의 경기 화면은 census 실행으로 보지 못했다(600초 실행이 제작진 소개 화면에 머문다 · §4). 경기까지 가는 키 순서로 다시 재는 일은 하지 않았다.
- 그림이 원본 단말과 같은지는 대조할 기준 화면이 없다.
