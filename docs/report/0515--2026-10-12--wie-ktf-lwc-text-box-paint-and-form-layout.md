## [2026-10-12] KTF lwc 글상자가 친 글자를 그린다 — 폼 배치는 게임 자신의 layout() 이 한다 (wie-ktf-lwc-text-box-paint-and-form-layout)

**무엇을**: lwc 층이 컴포넌트 위치·크기를 보관하고(`configure` · `getX/getY/getWidth/getHeight`), 그리기 전에 `validate` → `layout` 을 부르고(게임 서브클래스의 재정의가 돈다), 컨테이너가 자식을 제 위치로 옮겨 그리고, `TextComponent` 가 배경과 글자를 그린다. 글상자는 한 줄 높이로 시작하고, `TextBoxComponent(String,int,int)` 는 javadoc 순서(data, constraints, h)로 고쳤다.
**왜**: 운영자 채택 제안 `2026-10-11-ktf-text-box-charat-name-box-remeasure#p0`. `d448aee68157` 의 이름 상자는 키를 받는데(0510) 화면에 아무것도 안 보여서 «입력이 안 된다»로 읽혔다.
**사용자 영향**: 이 게임의 «러비이름 지어줘!» 칸에 친 글자가 «이름» 옆 분홍 칸에 보인다.

엔진: `origin/main` `1c06afa6`(전) · 이 브랜치(후), 둘 다 release `wie_validate`. 증적(화면)은 저장소 밖 `~/orchestrator/reports/evidence/wie-ktf-lwc-text-box-paint-and-form-layout/`.

### 1. 배치 규칙 — 관측으로 정했다

티켓은 «Form 세로 쌓기 먼저»를 제안했지만, 실측은 다른 답을 냈다.

- 이 게임의 폼은 `FormComponent` 가 아니라 **게임 서브클래스 `b`** 다(클래스 이름을 찍은 비커밋 탐침). `b.paint` 가 대화상자를 그린 뒤 `super.paint` 를 부르고, `b.layout()` 이 이름 상자를 직접 놓는다.
- `getX/getY/getWidth/getHeight` 를 0 이 아닌 값으로 돌려주게 한 탐침 4판(입력 `(11,22,33,44)` `(0,0,33,44)` `(11,22,0,0)` `(0,0,0,44)`)에서 `b.layout()` 은 모두 `configure(box, 56, 93, 92, 16, 3)` 을 불렀다. 넷 다 0 일 때만 `(0,0,0,0)` 이었다. 즉 **상자에 크기가 있으면 게임이 자리를 정한다.**
- `(56, 93)` 은 게임이 그린 «이름»(`drawString` `(28, 95)`) 바로 오른쪽이고, 흰 칸 `(25..151, 71..131)` 안이다.
- 그런데 이 층은 `layout()` 을 한 번도 부르지 않았다(`validate` 가 `layout` 스텁에 묶여 있었다). javadoc: `validate` 는 `paintContent`/`showNotify` 에서 불리고 `layout` 을 부른다 — 그래서 `validate` 가 `layout` 을 **가상 호출**하고, `ContainerComponent.paint` 가 그리기 전에 `validate` 를 부른다.
- javadoc 은 `(String,int,int)` 생성자의 h 를 「초기 컴포넌트의 높이값」이라 부른다 — 글상자에는 처음부터 높이가 있다. `(String,int)` 도 기본 글꼴 한 줄 + 위아래 1px 로 시작한다. 폭은 무엇이 정할 때까지 0.
- **기본 세로 쌓기(`layoutChildVertical`)는 만들지 않았다.** 이 게임은 쓰지 않고, 측정 없이 만들면 지금 아무것도 그리지 않는 다른 게임의 폼에 상자를 그리게 된다. `gab` 기본값도 API 클래스 파일에 없다.
- 상수는 `docs/reference/AromaWIPI_classes.zip` 의 상수 풀에서 읽었다: `POS_MASK` 1 · `SIZE_MASK` 2 · `CONSTRAINT_PASSWORD` 2. 이 게임 폼의 첫 상자(제약 2 = 비밀번호)는 게임이 놓지 않아 폭 0 이라 그려지지 않는다.

### 2. `d448aee68157` 전/후

축 boot·render·input·longplay·sound·speed·progress. 진도는 600초 정책 v2 P/P2.

| | status | 6축 | progress | P / P2 정체 |
|---|---|---|---|---|
| 전 | playable | ok ok ok ok ok ok | ok | 200 / 190 |
| 후 | playable | ok ok ok ok ok ok | stuck | 220 / 310 |

- 600초 키 루프(L) 74번째 키 화면(`__74_NUM6`): 전은 «이름» 옆이 비어 있고, 후는 분홍 칸에 «ㅁㅅㄱㄷ» 가 보인다(`d448-L74-before.png` / `d448-L74-after.png`). 밤 인트로 손 실행에서도 같다(`d448-night-form-before.png` / `d448-night-typed-after.png`).
- 진도: 전·후 모두 t600 에 펫 방 본편(배고픔 막대·시계)이다. 판정은 문턱 근처에서 갈린다 — 0510 은 이 화면에서 250/310 stuck 이었다. 이름 창 이후에는 lwc 위젯이 없어 이 회차 변경이 닿는 화면이 아니다. compat 의 `progress` 는 `stuck` 그대로 둔다(«후» 와 같다).

### 3. 짝 재측 — 문자열 `setMaxLength` 를 가진 KTF 36종 + 라이브 LGT 5 + 가드 2

정적 검사(compat KTF 269행 중 파일에 `setMaxLength` 가 있는 것): **36종**. 전 = `1c06afa6`, 후 = 이 브랜치.

- **30초 probe(43종 전부)**: boot·render·input·speed 축 43/43 같다. sound 는 `1b3b4868d46e` 하나가 전 ok → 후 silent 였는데, 같은 판을 다시 돌리면 전도 2/2 silent · 후 2/2 silent 다(이 타이틀의 소리는 긴 판에서 난다) — 측정 흔들림. 예외 수는 `e56253df12f3` 6 → 8(둘 다 `RecordStoreNotFoundException: sysdata` · 이 판은 lwc 호출 0)만 다르다.
- probe 는 lwc 화면에 거의 닿지 않는다: 전 판 stderr 에서 lwc 자식 paint 에 닿은 것은 `09a6a300994d` `65ef7052f528` 둘뿐이고, 두 판의 다른 화면은 눈으로 봐서 타이밍(글자 흘림 한 칸) 차이다.
- **600초 L 짝(lwc 를 쓰는 것으로 기록된 7종)**: `0c67145b11df` `09a6a300994d` `65ef7052f528` `9789fec50f39` `33f3e7669599` `8c71be3ad26d` `85e94babc247` — 7/7 이 전·후 playable · 6축 같다 · 키 854/900 · FAIL 0 · 예외 수 같음(`85e94babc247` 13 → 12). 화면이 다른 판은 키 루프가 다른 길로 간 정상 플레이 화면이다(눈으로 4종 확인).
- **라이브 LGT 5**(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) · **가드 2**(`49ade89578c5` `ddd885583b15`): probe 축 전·후 같다.
- 한계: 나머지 29종은 30초 probe 만 쟀다(600초 × 36 × 2 는 이 호스트에서 약 4시간). 이 29종이 그 뒤에 lwc 폼을 띄우면 이 회차 측정이 못 본다.

### 4. «되돌리면 red»

`a_form_lays_out_through_its_own_layout_and_paints_its_text_boxes`(이 게임의 `layout()` 형태를 흉내 낸 테스트 서브클래스) · `configure_keeps_position_and_size_by_mask` · `get_max_length_answers_what_set_max_length_kept`(확장). 하나씩 되돌린 7판 모두 red: 컨테이너 paint · `validate`→`layout` · 글상자 초기 높이 · `TextComponent.paint` · `configure` 크기 보관 · 비밀번호 별표 · `(String,int,int)` 순서.

### 5. 게이트

- fmt OK · `clippy --all -D warnings` rc=0 · wasm32 clippy rc=0 · `+beta clippy` rc=0(기존 unused-dependency 경고) · `RUST_MIN_STACK=4194304 cargo test --all` 778 passed / 0 failed.
- `npm run build:wasm` rc=0 · `check-engine-contract` 137 pass / 0 violation · `npm run audit` PASSED.
- 러너 블록: draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS(release 바이너리) · keydraw_ktf/lgt `--inject --expect-last-frame` PASS(27/27 · 문서대로 `cargo run` 디버그 — release 는 전·후 둘 다 max-ticks 로 UNMEASURED).

### 6. 원장 · 소식

- 0507 원장 csv `d448aee68157` 행: basis `0515` · pin `1c06afa6` · note 갱신.
- `docs/player-updates/2026-10-12-ktf-name-box-shows-typed-text.json` 1건.
- compat 행 변경 0(§2).

### 7. 게임 파일명 유입

- 도구 기본(이 브랜치가 바꾼 9파일): BOUNDED 4회/4쌍 · SUFFIX-ATTACHED 1회/1쌍. 전부 `component.rs`·`text_component.rs` 에 **이 회차 전부터 있던** 주석 줄이다 — 이 회차 diff 의 추가 줄(`+`)에서 그 이름은 0회다. SUFFIX 1건은 BOUNDED 의 더 긴 제목 안의 접두다.
- 이 회차가 새로 쓴 문장은 타이틀을 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=9 tree=a21f7f39253ab80a B=4/4 P=0/0 S=1/1 -->
