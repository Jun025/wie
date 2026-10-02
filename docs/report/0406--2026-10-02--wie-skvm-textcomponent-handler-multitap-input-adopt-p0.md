## [2026-10-02] SKVM TextComponentHandler 최소 멀티탭 입력기 — 등록된 입력칸에만 숫자 키 (wie-skvm-textcomponent-handler-multitap-input-adopt-p0)

**무엇을**: `com/xce/lcdui/TextComponentHandler`(`wie-skvm`)를 최소 입력기로 만들었다. 입력칸이 `setTextComponent` 로 등록돼 있을 때만 키를 먹는다. 숫자 키는 멀티탭(ITU-T E.161, 영문 대문자)으로 글자를 넣고, 같은 키를 1초 안에 다시 누르면 그 글자를 돌린다. `NUMERIC`·`PHONENUMBER`·`DECIMAL` 입력칸에는 숫자를 그대로 넣는다. `CLEAR` 는 지우고, 좌우는 입력칸의 `moveCursor` 로 넘긴다.
**왜**: `85f03ca7389e`(SKT)는 0400 이후 이름 입력 폼까지 열린다. 그러나 입력칸 `keyPressed(I)V` 의 바이트코드는 `getstatic 핸들러; iload_1; invokevirtual keyPressed(I)Z; pop; return` 뿐이다. 글자는 핸들러가 `TextComponent.insert(C)` 로 넣을 때만 들어간다. 핸들러가 키를 받지 않으니 이름 칸이 끝까지 비었다. 폼은 두 이름이 `trim()` 후 비어 있으면 넘어가지 않는다.
**사용자 영향**: 이름 입력 화면에서 숫자 키로 영문 이름을 넣고 실제 게임(키우기 방)으로 들어간다. 한글 이름은 아직 넣을 수 없다.

### 1. 모양 — 전부 게임 바이트코드 실측(0400 과 같은 스크래치 파서)
| 호출부 | 핸들러가 할 일 |
|---|---|
| 입력칸 포커스 설정 `a(Z)V`: 켜면 `setTextComponent(자기 InputMethodImpl)`, 끄면 `setTextComponent(null)` | 마지막으로 등록된 입력칸만 붙잡는다(null 이면 놓는다) |
| `InputMethodImpl.insert(C)` → 커서 자리에 끼움 · `replace(C)` → `b[caret-1]` 덮어씀 · `delete()` | 새 글자는 `insert`, 순환은 `replace` |
| `moveCursor(I)`: 키 142(좌)·145(우)만 처리하고, 끝에서 우는 공백을 넣는다. 마지막에 `handler.clear()` + `repaint()` | 좌우는 그대로 넘긴다 |
| 입력칸이 글자를 다시 채울 때(`a(String)`)와 `moveCursor` 끝에서 `handler.clear()` | 멀티탭 순환을 끝낸다 |
| 폼 생성 `K()`: 이름 두 칸 `XTextField("", 8, 0)` — 제약 `ANY` · 다른 화면 `(…, 11, 2)` — `NUMERIC` | 제약으로 멀티탭/숫자를 가른다 |

- 키를 «먹었다»(`true`)는 결과는 이 게임에서 버려진다(`pop`). 먹는 것은 등록된 입력칸이 있을 때로 한정했다. 등록이 없으면 종전과 같이 `false` 다.
- `CLEAR` 가 순환을 끝내지 않으면 `5 → J`, `CLEAR`, `5` 가 빈 칸에 `replace` 를 불러 `b[-1]` 이 된다(게임 쪽 배열 예외). 그래서 `CLEAR` 도 순환을 끝낸다. 꽉 찬 칸에서는 `insert` 가 무시되므로, 그 키는 순환을 시작하지 않는다(마지막 글자를 덮어쓰지 않는다).
- **이 핸들러를 쓰는 다른 SKT 두 종은 구조상 영향이 없다**(바이트코드 실측 · 메서드 참조 수):
  - `14a62a8521a0`: `getTextComponentHandler` 1 · `getInputMode` 4 · `keyPressed` 4 · `keyReleased` 5 · ★`setTextComponent` **0**. 등록이 없으니 `keyPressed` 는 늘 `false` 다.
  - `9a2cf5ffc9d3`: `isLoaded` 1 · `getTextComponentHandler` 1 · `getInputMode` 1 · `getTextComponent` 1 · `setTextComponent` **0** · 키 메서드 0.
- `isLoaded()` 는 `false` 그대로 둔다(9a2cf5ffc9d3 의 입력기 표시를 켜지 않는다). `keyReleased`·`keyRepeated` 도 `false` 그대로다. 측정된 호출부는 모두 결과를 버리거나, 등록이 없다.

### 2. 전/후 — `wie_validate`(release · 같은 시각 짝 · 2회씩)
`host-load-guard --status --recovered` rc=0 에서 시작했다. 그러나 실행 중 load1 이 **234~281** 로 올랐다(가드는 idle 표본으로 판정한다). 수치는 그 부하 아래의 값이다.

| 실행 | 전 | 후 |
|---|---|---|
| `85f03ca7389e` `--keys` 이름 경로¹ — 마지막 화면 색 수 | 305 · 307(빈 이름 폼) | **139 · 139(게임 안 방)** |
| 〃 paints · 예외 | 376 · 375 / 0 · 0 | 371 · 373 / 0 · 0 |
| `85f03ca7389e` `--inject` 27키 — paints · 마지막 색 · 예외 | 219 · 233 / 307 · 307 / 0 | 240 · 241 / **317** · **317** / 0 |
| `14a62a8521a0` `--inject` — 결과 · 키 · 예외 | PASS 27 · PASS 27 / 8 · 8 | UNMEASURED(clean exit, 12) · 〃 / 7 · 7 |
| `9a2cf5ffc9d3` `--inject` — paints · 마지막 색 · 예외 | 31 · 31 / 109 / 0 | 29 · 30 / 109 / 0 |

¹ `OK:2 OK:2 LSOFT:3 NUM5:1.5 NUM2:1.5 UP:1 NUM6:0.3 NUM6:1.5 DOWN:1 DOWN:1 DOWN:1 OK:3 OK:3 OK:3 OK:3`. 후 화면에서 아래 칸에 `JA`(5 → J, 1.5초 뒤 2 → A)가, 위 칸에 `N`(6 → M, 0.3초 안에 6 → N)이 들어간다. OK 로 게임 방에 들어간다. 전에서는 두 칸이 비어 폼에 남는다.
- `85f03ca7389e` 기본 27키에서도 마지막 화면 색이 307 → 317 로 바뀐다. 기본 스크립트의 `NUM5` 가 이름 칸에 `J` 를 넣기 때문이다(화면 확인). 두 칸이 다 차지 않아 폼을 넘지는 않는다.
- **`14a62a8521a0` 의 «clean exit · 12키»는 이 변경 탓이 아니다.** 짝을 늘려 재측했다. **전 1/11 · 후 2/11** 이 같은 지점(12번째 키, paints 173~179, 예외 7)에서 끝났고, 나머지 20회는 둘 다 PASS 27/27 · 예외 7~8 이다. 전 바이너리에서도 같은 서명이 나왔다(로그: 전 1회차 `UNMEASURED clean exit 12 173 7`). 구조상으로도 등록이 없어 반환값이 종전과 같다(§1). 부하 아래 키 타이밍 경합이다.

### 3. 진도 축(0393 도구 · `--only progress` 600초 · P + P2 · `--jobs 1`)
probe → long 600 → P → P2 를 바이너리마다 직렬로 돌렸다(10:12 시작 시 load1 11).

| | probe A paints · 예외 | long paints · 예외 | P stall · 새 화면 | P2 stall | 축 | load1(A/L/P/P2) |
|---|---|---|---|---|---|---|
| 전 | 368 · 0 | 7,313 · 0 | 590 · 1 | 590 | stuck | 20 / 31 / 108 / 206 |
| 후 | 255 · 0 | 6,186 · 0 | **520 · 5** | **500** | stuck | 208 / 133 / 189 / 131 |

- **축 판정은 stuck → stuck 이다. 숨기지 않는다.** 서는 자리가 바뀌었다. 전은 10초 첫 표본부터 빈 이름 폼이다(새 화면 1). 후는 80~100초까지 새 화면이 나오고, 그 뒤 게임 안 «먹이» 화면(말풍선·음식 줄이 바뀐다)에서 정책 키를 돈다. 예외는 0 이다. 그 화면을 넘어가는 키는 진도 정책에 없다. 키우기 게임이 진도 정책과 맞지 않는 것인지, 다음 벽인지는 이 회차가 가르지 않았다.
- 후 쪽 load1 이 130~208 로 전보다 높다(호스트가 측정 중 다시 포화). 그래서 후의 paints 감소(probe 368 → 255)를 회귀로 읽지 않는다.

### 4. 시험
- `registered_field_takes_multitap_and_digits`(wie-skvm): 등록이 없으면 `false`. 등록하면 `5 5 2` → `KA` · `CLEAR` → `K` · 다시 `2` → `KA`(CLEAR 가 순환을 끝냄) · `clear()` 뒤 `2` → `KAA` · 꽉 찬 칸에서 `4 4` → 그대로. 좌 → `moveCursor(142)`. `#` → `false`. `NUMERIC` 칸에 `0 1 1` → `011`. null 등록 뒤 `false`.
- 개악 red 3/3(실측): `keyPressed` 를 종전 `false` 로 되돌림 → 실패 · `clear()` 를 no-op 으로 → 실패 · `CLEAR` 의 순환 종료 제거 → 실패.
- 러너 블록(AGENTS.md · release `wie_validate`): `draw_j2me` `helloworld_ktf` `helloworld_lgt` `text_j2me` PASS · `keydraw_ktf/lgt --inject --expect-last-frame --max-ticks 1000000000` PASS 27/27 · rc=0(paints 79 · 55).
- 네 관문: `cargo fmt --check` · `clippy --all -D warnings` · `clippy --target wasm32-unknown-unknown -D warnings` · `cargo +beta clippy --all -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` 모두 rc=0.

### 5. 범위 밖으로 둔 것
- 한글 조합 · 대소문자/입력 모드 전환(`#`·`*`) · `getInputMode` 실값 · `getTextComponent` — 측정된 호출부가 필요로 하지 않는다.
- 멀티탭 표와 1초 창은 고정값이다(코드 `ponytail:` 주석). 바꿀 근거가 생기면 그때 바꾼다.
