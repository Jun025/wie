## [2026-10-01] SKVM com/xce/lcdui/TextComponent — 이름 입력칸 생성 벽 (wie-skvm-xce-lcdui-textcomponent-class-adopt-p1)

**무엇을**: `com/xce/lcdui/TextComponent` 인터페이스를 새로 넣었다. `TextComponentHandler` 에 `setTextComponent`·`clear` 를 더했다(`wie-skvm`). MIDP `javax/microedition/lcdui/TextField` 를 상수 전용 클래스로 넣었다(`wie-midp`).
**왜**: `85f03ca7389e`(SKT)는 시작 화면에서 키를 받으면 이름 입력 폼을 만든다. 폼의 글자 입력칸 클래스가 `com/xce/lcdui/TextComponent` 를 구현해서, 그 클래스를 읽는 순간 `NoClassDefFoundError` 가 났다. 게임 스레드가 죽고 화면이 시작 화면에 굳었다(0393 §8).
**사용자 영향**: 시작 화면에서 굳던 게임이 이름 입력 화면으로 넘어간다. 아직 이름을 넣지는 못한다(§4 — 다음 벽).

### 1. 넣은 모양 — 전부 호출부 실측
XCE 문서는 이 repo 에 없다. 게임 jar 의 클래스 파일을 직접 읽었다(상수 풀 + 바이트코드, 스크래치 파서). 호스트에 Java 런타임이 없어 `javap` 은 쓰지 못했다.

| 클래스 · 멤버 | 근거 |
|---|---|
| `TextComponent` = **인터페이스** | 게임의 두 입력칸 클래스의 내부 클래스(`…$InputMethodImpl`) 둘 다 `implements com/xce/lcdui/TextComponent`, 부모는 `java/lang/Object` |
| 추상 메서드 13개 — `getCaretPosition()I` `getConstraints()I` `getMaxSize()I` `size()I` `insert(C)V` `delete()V` `clear()V` `replace(C)V` `moveCursor(I)V` `setCaretPosition(I)V` `setCaretVisible(Z)V` `repaint()V` `repaintIM()V` | 두 내부 클래스가 구현한 공개 메서드. 이름·서술자가 **두 클래스에서 같다**. 그 밖의 메서드는 없다 |
| `TextComponentHandler.setTextComponent(Lcom/xce/lcdui/TextComponent;)V` | 입력칸의 포커스 설정 메서드가 부른다(두 클래스 모두) |
| `TextComponentHandler.clear()V` | 입력칸이 글자를 다시 채울 때와 커서 이동 끝에서 부른다 |
| `TextField.CONSTRAINT_MASK` · `TextField.PASSWORD` | `TextComponent` 다음 벽. 같은 생성자 경로(`XTextField.<init>` → `c(I)V`)가 `getstatic` 으로 읽는다. 생성은 하지 않는다 |

- `TextComponent` 만 넣으면 벽이 한 줄 뒤(`TextField`)로 옮겨질 뿐이었다(§2 표 «중간»). 같은 생성자 경로의 두 벽을 함께 넣었다.
- `TextField` 는 MIDP 2.0 상수 13개만 넣었다. `<init>` 이 없으니 생성하는 게임은 `NoSuchMethodError` 로 다음 할 일을 알린다.
- **선행 `TextComponentHandler`(wie-census-small-stub-bundle · 0391 계열)와의 관계**: 같은 핸들러를 그대로 쓴다. 이 게임도 `<clinit>` 에서 `getTextComponentHandler()` 로 한 개를 받고 키를 `keyPressed/Released/Repeated(I)Z` 로 넘긴다 — 이미 있던 메서드다. 새로 필요한 것은 위 두 메서드뿐이었다. 둘 다 아무것도 하지 않는다. 입력기가 돌지 않으니 지울 조합 상태도, 붙잡을 입력칸도 없다.

### 2. 전/후 — `wie_validate --inject` (release · 같은 시각 짝)
| 실행 | 전 | 중간(TextComponent 만) | 후 |
|---|---|---|---|
| java_exceptions | 27 · 27 · 27 | 27 · 27 | **0 · 0** |
| 첫 예외 | `NoClassDefFoundError: com/xce/lcdui/TextComponent` | `NoClassDefFoundError: javax/microedition/lcdui/TextField` | 없음 |
| frozen_tail_steps (27 중) | 26 · 26 · 26 | 26 · 26 | **0 · 0** |
| paints | 33 · 34 · 33 | 34 · 35 | **238 · 241** |
| 마지막 화면 색 수 | 129 · 129 · 128 | 129 · 129 | **307 · 307** |
| 마지막 화면 | 시작 화면 | 시작 화면 | 이름 입력 폼(입력칸 2 + 선택 1 + OK) |

- 호스트 load1 10~50(13:47~14:26). `host-load-guard --status --recovered` rc=0 에서 쟀다(14:0x~14:25 는 보류라 기다렸다).

### 3. 진도 축(0393 과 같은 도구 · `--only progress` 600초 · P + P2 짝)
`playability-census.mjs run` 을 probe → long 600 → progress P → P2 순으로 바이너리마다 돌렸다(`--jobs 1` · 호스트 잠금으로 직렬).

| | probe A paints · 예외 | long paints · 예외 | P | P2 | 축 |
|---|---|---|---|---|---|
| 전 | 33 · 27 | 32 · 854 | stuck(stall 590 · 화면 1) | stuck(590 · 1) | **stuck** |
| 후 | 368 · 0 | 7,304 · 0 | stuck(stall 590 · 화면 1) | stuck(590 · 1) | **stuck** |

- **진도 축은 움직이지 않았다 — 숨기지 않는다.** 서는 자리만 바뀌었다(시작 화면 → 이름 입력 폼). 정책 키(확인·방향·5)로는 폼을 넘지 못한다. 10초 첫 표본이 이미 폼이라 «새 화면 1»이다.
- load1: 전 P 13.2 · P2 14.9, 후 P 9.9 · P2 25.3.

### 4. 다음 벽 — 입력칸에 글자가 들어가지 않는다
게임의 입력칸 `keyPressed(I)V` 는 바이트코드 전체가 `getstatic 핸들러; iload_1; invokevirtual keyPressed(I)Z; pop; return` 이다. **글자는 핸들러(입력기)가 `TextComponent.insert(C)` 로 넣을 때만 들어간다.** 지금 핸들러는 키를 받지 않으므로(`false`) 이름 칸은 끝까지 비고, 폼을 넘지 못한다. 다음 회차는 핸들러가 숫자 키를 받아 `setTextComponent` 로 받은 입력칸에 `insert`/`delete` 하는 최소 입력기다(worklog 제안).

### 5. 시험
- `text_component_implementor_loads_and_registers`(wie-skvm): 게임 내부 클래스 모양(인터페이스 구현 + 메서드 1)을 만들고, 인터페이스로 호출하고, 핸들러에 등록(+null)·`clear` 한다.
- `text_field_constants_match_midp`(wie-midp): `CONSTRAINT_MASK`·`PASSWORD`·`NUMERIC` 값.
- 개악 red 3/3: `get_protos()` 에서 `TextComponent` 를 빼면 `NoClassDefFoundError` · 핸들러 `clear` 를 빼면 `NoSuchMethodError` · `TextField` 를 빼면 `NoClassDefFoundError`.
- 러너 블록(AGENTS.md): `draw_j2me` `helloworld_ktf` `helloworld_lgt` `text_j2me` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 는 릴리스 바이너리에서 `UNMEASURED · stop max-ticks`(0393 §7 과 같다). `--max-ticks 1000000000` 으로 **전·후 모두** PASS · 27/27 키 · rc=0 · 마지막 화면 내용 있음(paints 전 79·55, 후 80·55).
- 네 관문: `cargo fmt --check` · `clippy --all -D warnings` · `clippy --target wasm32-unknown-unknown -D warnings` · `cargo +beta clippy --all -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` 모두 rc=0.

### 6. 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs --corpus <game_lab>`: 유입 0건(BOUNDED 0회/0쌍) · 판단 필요 0건(SUFFIX-ATTACHED 0회/0쌍). 새 주석·문서·소식은 sha12 만 쓴다.

<!-- corpus-name-inflow v1 subjects=10 tree=13fabe97846e52ba B=0/0 P=0/0 S=0/0 -->
