## [2026-10-09] SKVM XTextField 키 입력 — 숫자 멀티탭 · CLEAR · 글자가 바뀌면 스스로 repaint (wie-skt-xtextfield-key-input-name-screen)

**무엇을**: `com/xce/lcdui/XTextField`(`wie-skvm`)의 `keyPressed` 가 숫자 키를 멀티탭 영문 대문자로 넣는다. 같은 키를 1초 안에 다시 누르면 마지막 글자를 돌린다(`wie_util::keypad` · SKVM `TextComponentHandler` 와 같은 표). `NUMERIC`·`PHONENUMBER`·`DECIMAL` 칸은 숫자를 그대로 넣는다. `CLEAR` 는 한 글자 지운다. 글자가 바뀌면 입력칸이 **스스로 `repaint()`** 한다. `keyRepeated` 는 숫자를 무시한다(누른 채로 두면 글자가 돌지 않게). `keyReleased` 스텁은 할 일이 없어 no-op 로 바꿨다. `setText` 가 순환을 끝낸다.
**왜**: `ec2f8f2e02a2` 이름 화면에서 키가 반영되지 않았다(0482 §3 · `keyReleased` 스텁 로그). 원인은 스텁이 아니라 **다시 그리기**다. 게임 바이트코드(스크래치 파서 · 0400 과 같은 방식) 실측:
- 캔버스 `keyPressed` 는 이름 상태에서 `key_name(I)` 만 부르고 `repaint` 없이 끝난다. `run()` 루프도 이 상태에서는 다시 그리지 않는다.
- `key_name`: `lookupswitch {131, 148(FIRE)}` — 확인 키는 `name != null && name.length() != 0` 일 때만 넘어간다. 그 밖의 키는 `tf` 가 켜져 있으면 `field.keyPressed(key)` 로 넘기기만 한다.
- `name` 은 `draw_name()`(paint) 안의 `name = field.getText()` 에서만 갱신된다.
- ⇒ 종전 `keyPressed` 는 `5` 를 글자 `'5'` 로 넣었지만 다시 그리지 않아 `name` 이 끝까지 `""` 였고, 확인 키가 거절됐다.
**사용자 영향**: 이 게임에서 숫자 키로 영문 이름을 넣고 확인 키로 프롤로그로 넘어간다. 같은 입력칸을 쓰는 다른 SKT 게임은 이름 글자가 숫자 대신 영문으로 들어간다(5 → J). 한글 이름은 아직 넣을 수 없다.

### 1. 근거 · 가정
| 항목 | 값 | 근거 |
|---|---|---|
| 스스로 repaint | 글자가 바뀌는 키마다 `canvas.repaint(bounds)` | 위 호출 형태(게임이 다시 그리지 않음) — 근거 있음 |
| 글자 상한 | 생성자 `maxSize` 그대로(꽉 차면 새 글자 무시 · 순환도 시작하지 않음) | 호출부 `XTextField("", 5, 0, this)` · `("", 8/9/11, 0/2, …)` |
| 입력 모드 | 제약 `& 0xFFFF` 가 2·3·5 면 숫자, 아니면 멀티탭 대문자 | 호출부 제약값 0 · 2 만 실측. **멀티탭 표·1초 창·대문자는 가정**(0428 §2 와 같은 표 — 문서 없음) |
| 한글 | 없음 | **가정 밖** — `2f5246006bd8` 화면 문구가 «한글 4자까지 지원» 이다. 실기 기본은 한글 입력기일 가능성이 높다 |
| `inputChar(C)` | 그대로(기존 동작) | `ec2f8f2e02a2` 캔버스가 `inputChar(C)` 를 입력칸으로 넘긴다 — 실기 입력기가 글자를 이 경로로 넣는 것으로 보이나 이 엔진은 그 콜백을 부르지 않는다 |
| `keyReleased` | 할 일 없음 | 순환은 시간·다른 키로 끝난다 |

### 2. 대상 3종 전/후 (release `wie_validate` · 같은 시각 짝 · `--inject --max-ticks 1e9 --keys` · 캡처는 로컬 스크래치)
| sha12 | 키 | 전 | 후 |
|---|---|---|---|
| `ec2f8f2e02a2` | `WAIT:15 OK OK RIGHT OK`(이름 화면) `NUM2:0.3 NUM2 NUM3 OK …` | 이름 칸 빈칸 · 확인 키 무반응 · 마지막 화면 = 이름 화면(색 5) | 칸에 `BD` · 확인 → **프롤로그** 화면(색 39) |
| `2f5246006bd8` | 진도 레시피 앞 2줄(이름 = `NUM5 UP`) | 이름 `5` → 머리말 · 같은 경로 | 이름 `J` → 머리말 · 같은 경로(차이 = 이름 글자 영역뿐) |
| `2d66945008c1` | 진도 레시피 앞 2줄 | 같은 경로 | 같은 경로 · 마지막 화면 동일 |

- ★**티켓 전제 정정**: `2d66945008c1` 은 `com/xce/lcdui/XTextField` 참조가 **0** 이다(jar 전수 · 쓰는 xce 클래스는 `XDisplay`·`Toolkit`·`XFile` 뿐). 이름 화면은 게임이 스스로 그린다. 0482 의 «같은 스텁 경유»는 이 게임에 맞지 않는다. 전/후 프레임 차이는 연출 타이밍뿐이다.
- `f2ae515201f2`(퇴행 짝 · 레시피 `NUM5 NUM5 OK`): 이름 `55` → `JJ`(3초 간격이라 순환하지 않음) · 같은 경로 · 마지막 화면 동일. 진행 요령 문구(«숫자 5 키를 두 번»)는 그대로 맞다.

### 3. 다른 SKT 이름 화면 — 짝 재측
SKT 코퍼스(`game_lab/{working,broken}/skt` 84 파일)에서 `XTextField` 를 부르는 jar 는 **12종**이다: 위 3종(`ec2f8f2e02a2` `2f5246006bd8` `f2ae515201f2`) + `640428a9cf9e` `b42e4242866b` `8fec741a782d` `9a2cf5ffc9d3` `47fe675bfffd` `ccb45e6b8d80` `14a62a8521a0` `ca1132f2e7bc` `fb80e97cbc57`. 전원 `keyPressed`·`keyReleased` 를 부른다.
9종 `--inject` 27키 전/후(동시 ≤ 3 · load1 8~19): 결과 9/9 PASS 27/27 양쪽 · 예외 수 동일 · 마지막 프레임 8/9 바이트 동일. `fb80e97cbc57` 은 마지막 색 154 → 152 인데 같은 화면이고 차이는 타자 연출·캐릭터 위치(첫 키부터 시간차)다.
- `14a62a8521a0`(0482 «이름 화면 무반응» 의심)은 27키로 `XTextField` 생성까지 가지 않는다(디버그 로그 0회). 이름 화면은 이 회차에서 재지 않았다.

### 4. 진도 축 — 판정 불변(compat 무변경)
`ec2f8f2e02a2` 후 바이너리 probe → long 600 → P → P2(`--progress 600` · 정책 v2 · 레시피 없음 · 한 파일 코퍼스).
- probe A PASS · paints 63 · 예외 0 · long 600 UNMEASURED(deadline · 854/900) · 예외 0.
- P stall **350** · P2 stall **350** ⇒ **stuck**(0475 의 350 과 같다). load1 P 34 · P2 10.
- 이유: 정책은 «새로 시작?» 창이 NO 에 있어 이름 화면까지 가지 않는다. 120초부터 «환경설정» 화면을 돈다(0475 와 같은 자리). ⇒ 이 고침은 정책 축이 볼 수 없는 자리다. compat 행 변경 0.

### 5. 시험 · 관문
- `digit_keys_type_multitap_and_repaint`(wie-skvm): `2 2 3` → `BD` · repaint 3회 · CLEAR 뒤 `3` → 새 글자 · 누른 채 `3`(keyRepeated) → 불변 · `setText` 뒤 꽉 찬 칸 `4 4` → 불변 · NUMERIC `0 1 1` → `011`. 기존 시험의 repaint 횟수 단언은 1 → 3(키 두 번이 스스로 그린다).
- 개악 red 2/2(실측): self-repaint 제거 → 실패 · 숫자 멀티탭 제거(종전 `'2'` 그대로) → `"223"` ≠ `"BD"` 실패.
- 네 관문: `cargo fmt --check` · `clippy --all -D warnings` · wasm32 clippy · `cargo +beta clippy --all -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` 760 passed / 0 failed.
- 러너 블록(release): `draw_j2me` `helloworld_ktf` `helloworld_lgt` `text_j2me` PASS · `keydraw_ktf/lgt --inject --expect-last-frame --max-ticks 1e9` PASS 27/27(paints 79 · 55).
- `smoke_gate.sh`(SKT · 후 바이너리): 50/50 PASS · 퇴행 0.

### 6. 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs`: 유입 0건(BOUNDED 0회/0쌍) · 판단 필요 0건(SUFFIX-ATTACHED 0회/0쌍). 새 주석·문서·소식은 sha12 만 쓴다.

<!-- corpus-name-inflow v1 subjects=4 tree=ed33014fe3486c52 B=0/0 P=0/0 S=0/0 -->
