## [2026-10-09] 로고 뒤 흰 화면 — 덮인 카드를 그리다 예외 (wie-ktf-aa3fcba4598b-logo-card-null-array-root-cause)

**무엇을**: `net.wie.CardCanvas.paint` 가 불투명 카드에 완전히 덮인 아래 카드를 더는 그리지 않는다.
**왜**: `aa3fcba4598b`(KTF) 가 로고 뒤 흰 화면에서 멈춰 «부분 지원»이었다(0484 §7 — 원인 지점까지만 쟀다). 운영자 채택 제안 `2026-10-09-non-playable-engine-residue#p0`.
**사용자 영향**: 그 게임이 로고 → 타이틀 메뉴 → 게임 모드 선택 → 수조 화면까지 간다. 화면 위에 다른 카드가 겹치는 KTF·LGT 게임 8종은 판정·화면이 그대로다(§4).

타이틀은 sha12 로만 적는다. 증적(캡처)은 저장소 밖 `~/orchestrator/reports/evidence/wie-ktf-aa3fcba4598b-logo-card-null-array-root-cause/` 에 둔다.

### 1. 규명 사슬

0484 의 가설은 «필드를 채우는 코드가 안 돈다»였다. 실제는 반대다 — **채웠다가 게임이 스스로 비운 뒤, 엔진이 그 카드를 계속 그렸다.**

임시 계측(커밋 안 함): `KtfClassLoader::findClass` 에서 클래스마다 메서드 본체 주소·필드 오프셋을 찍고, `java_throw` 에서 레지스터·`this` 의 필드 워드를 찍었다. 이미지는 `wie-ktf-dump` 로 떠서 capstone 으로 읽었다.

| 단계 | 주소(이미지 base 0x100000) | 읽은 것 |
|---|---|---|
| 로고 카드 | 클래스 `GNCLogo`(부모 `org/kwis/msp/lcdui/Card`) — `<init>` `0x131c2c` · `run` `0x131ea4` · `paint` `0x131f9c` | 필드 `im [Image` off `0x1c` · `display B` · `ani B` · `ani2 B` · `logoE Thread` … |
| 쓰기 지점 | `<init>` `0x131c9e` `anewarray(8)` → `0x131cac` putfield `im` · 루프 `0x131d10`~ 가 `im[0..7]` 에 `createImage` | **돈다** — 첫 paint 의 `drawImage` 가 성공한다(로그) |
| 비우는 지점 | `paint` `0x1320c4`: `ani > 11 && ani2 > 9` 이면 정적 플래그 1 · `im`/`logoE`/… 에 null · `0x13573e` 로 정적 호출 | 그 뒤 로그: `Runtime.getRuntime` → 게임 카드(`Context`)의 클래스 적재·생성 — `GOGOGO()`(`0x12fbf4`)가 `new Context` 뒤 `display.pushCard(canvas)` |
| 예외 지점 | `paint` `0x132536` → `0x134cc8`(aaload 도우미 · 배열 null 이면 `0x134cfc` 에서 NPE) | `R5`(배열)=0 · `R4`(색인)=6 · `this` 필드 워드: `im`=0 · `logoE`=0 · `ani`=12 · `ani2`=10 |

남은 질문은 «게임 카드를 쌓았는데 왜 로고 카드가 계속 그려지나»였고, 답은 엔진에 있었다: `CardCanvas.paint` 는 스택의 **모든** 카드를 아래부터 그리고, 카드 하나의 paint 가 예외를 내면 `?` 로 루프를 끝낸다. 로고 카드(아래)가 매 프레임 NPE ⇒ 위의 게임 카드는 한 번도 그려지지 않는다 ⇒ 로고 카드가 첫 칸에 칠한 흰 바탕만 남는다. 게임 스레드는 정상이라 `Uncaught` 0 · 예외 계수 0 이었다.

서버 연결과는 무관하다(접속 시도 0 · 이 경로에 네트워크 호출 없음).

### 2. 처방

- `CardCanvas.paint`: 먼저 스택 전체의 경계(`getX`·`getY`·`getWidth`·`getHeight`)와 `Card.transparent`(Card 자신의 필드 — `get_declared_field`, 하위 클래스가 같은 이름을 가져도)를 읽고, **위에 있는 불투명 카드 하나가 경계를 완전히 덮는 카드는 건너뛴다.** 덮이지 않거나 위가 투명이면 종전처럼 그린다.
- 단말 WIPI 가 «불투명 카드 아래는 안 보인다»를 어떻게 구현했는지는 문서로 확인하지 못했다. 이 규칙은 «안 보이는 것만 안 그린다» 쪽으로 좁혔다 — 덮인 카드가 자기 경계 밖에 그리는 픽셀(카드는 자기 경계로 클립되지 않는다 · upstream 7304facd)은 이제 안 보인다. 진단 스캔에서 그런 타이틀은 못 봤다(§4).
- 시험 `an_opaque_card_hides_the_cards_it_covers`: 아래부터 [넓은 불투명 · 덮이는 불투명 · 투명 · 불투명] 네 장 → paint 횟수 `[1, 0, 1, 1]`. **되돌리면** `[1, 1, 1, 1]` 로 red(확인).

### 3. 전/후 — `aa3fcba4598b`

- 짝 프로브(`--inject` 27키 · 같은 인자): base `distinct_colors 9`, 27 단계 전부 흰 화면 ↔ head `512`, glu 로고 → 타이틀 메뉴(게임시작·이어하기·정보이용료·게임종료) → 모드 선택(어드벤처·시간제한) → 수조·안내문. 캡처 = 증적 디렉터리의 `base-vs-head.png` 와 단계별 `base-*`/`head-*`.
- head census: §5.

### 4. 퇴행 — 변경이 닿는 타이틀

- **진단 스캔**(임시 빌드 · 커밋 안 함 · 건너뛸 때 한 줄): KTF·LGT 고유 **344종**을 `--inject` 로 돌렸다 ⇒ 건너뛰기가 실제로 일어난 것은 **9종**(대상 + `09a6a300994d` `0c67145b11df` `17ed9c29ae4d` `2b1ed0c8d061` `4166acd8fc62` `444821c513ac` `ca7fa8ade8ad` `d234152636c8`). 나머지 335종은 건너뛸 카드가 없어 그리는 순서·횟수가 종전과 같다.
- **짝 프로브 9종**(base = `origin/main` `82959a5b` · head = 이 브랜치 · release `wie_validate` · 3폭 · long 임대 1): result·stop·content·last_frame_content·input_steps 가 바뀐 것은 0 — 대상의 `distinct_colors` 만 9 → 512.
  - 프레임 해시가 다른 단계(4~28/28)를 8종 전부 나란히 봤다: 장면 진행 시점 차이(같은 장면이 한두 단계 앞뒤)이고 깨짐·빈 화면·누락 0.
  - 하나만 의심스러웠다 — `09a6a300994d` 닉네임 칸 글자가 base 26·27 단계에 없고 head 엔 있다. `--shot-every 0.3` 시계열로 다시 재니 두 판의 마지막 40프레임 글자 픽셀 수열이 **완전히 같았다**(깜빡임이 샘플 시점에 걸린 것).
- 통신사 코퍼스 smoke gate(`scripts/smoke_gate.sh` · `BIN`=head release · `WORKING_DIR=game_lab/working`): KTF **190 checked · 0 absent · 0 regressions**, LGT **52 · 0 · 0**. SKT 는 이 경로(`org.kwis.msp.lcdui` 카드 스택)를 쓰지 않아 돌리지 않았다.

### 5. head census — `aa3fcba4598b`

`scripts/census-drive.sh`(phase 마다 long 임대 1) · head release · 이 타이틀 하나: boot·render·input·longplay(600초)·sound·speed **전부 ok ⇒ playable**(전 `limited` `oonuoo-`). speed 비율 0.995.

- progress **stuck**(P·P2 1800초 둘 다): 진도 정책 키로는 수족관 1-1 을 넘지 못한다 — 물고기·돈·커서가 끝까지 움직이고(마지막 새 프레임 1,860초 · 고유 프레임 147/146) 화면은 그 판에 머문다. progress 는 `status` 밖이고, 같은 «playable · stuck» 109행 중 76행처럼 이유 문장은 두지 않았다(census `report` 가 만든 행 그대로 · 이 행만 손으로 옮겼다 — `report --compat` 를 저장소 경로에 걸면 그 파일을 1행으로 덮어쓴다, 한 번 덮어 되돌렸다).
- 이용자 문장은 «키를 눌러도 화면이 바뀌지 않을 수 있어요» → 없음.

### 6. 검증

- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` · `cargo +beta clippy --all -- -D warnings`(rc=0 · 0484 §9 와 같은 «unused dependency» 9건 — 손대지 않은 Cargo.toml) · `RUST_MIN_STACK=4194304 cargo test --all`(FAILED 0) 통과.
- 러너 줄(AGENTS · head release): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS · `keydraw_ktf/lgt --inject --expect-last-frame --max-ticks 500000000` PASS · rc=0 · 27/27 · paints 79 / 55.
- 되돌리면 red: `CardCanvas.paint` 를 `origin/main` 판으로 되돌리면 `an_opaque_card_hides_the_cards_it_covers` FAILED(`[1, 1, 1, 1]`) · 원상 green.
- `node scripts/player-data.mjs` · `--selftest` · `node scripts/check-compat-revert.mjs` · `node scripts/check-docs-report-serial.mjs` · `node scripts/check-worklog-json.mjs` · `npm run audit` 통과.
- 자원: 측정 실행마다 long 임대 1 · 그 안 3폭 이하 · `nohup` 0 · 끝난 뒤 자기 `wie_validate` 0. load1 13~19.

### 7. 게임 파일명 유입

도구 기본 실행(`origin/main...HEAD` · 6파일): BOUNDED 330쌍 · SUFFIX-ATTACHED 15쌍 — 둘 다 전부 `docs/player-data/compat.json` 의 기존 `title`·`fileTitle` 값이다(0484 §10 과 같은 모양). 이 회차는 그 파일의 1행 `status`·`axes`·`knownIssues_ko` 만 바꿨다. 그 밖의 파일은 0건 — 이 문서·worklog·업데이트 소식은 타이틀을 sha12 로만 적었다.

