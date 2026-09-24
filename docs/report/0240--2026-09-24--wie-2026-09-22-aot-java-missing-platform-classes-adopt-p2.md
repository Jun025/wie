## [2026-09-24] 슈퍼액션히어로의 «균일색 빈 프레임»은 화면 층이 아니다 — 게임 스레드가 import 0xfd 에서 첫 줄에 죽는다 · 0xfd = long[] 원소 저장으로 닫았다 (wie-2026-09-22-aot-java-missing-platform-classes-adopt-p2)

**무엇을**: LGT Java 시스템 import `0xfd` 를 `StoreLongArray`(checked `lastore`)로 등재했다. 처리기 1개 + SVC id 35 +
단위 테스트 2개. 채택 제안 `2026-09-22-aot-java-missing-platform-classes#p2`(「그리기 호출이 없는지, 그려도 화면에 안 붙는지 가려라」).

**왜**: 답은 «그리기 호출이 없다»였다. 그리고 호출이 없는 이유가 화면도 그리기 경로도 아닌 **런타임 import 하나**였다.

**사용자 영향**: 아직 없다 — 이 타이틀은 여전히 균일색이다. 벽이 import 0xfd 에서 `DataInputStream` vtable index 32 로 옮겨 갔다.

### ⒜ 전제 재측 — `origin/main` `718a9402` release `wie_validate --inject`, 3회
| 회 | result | stop | input | ticks | paints | distinct_colors |
|---|---|---|---|---|---|---|
| 1 | FAIL | deadline | 27/27 | 21,412,511 | 3 | 1 |
| 2 | FAIL | deadline | 27/27 | 20,783,955 | 3 | 1 |
| 3 | FAIL | deadline | 27/27 | 25,839,918 | 2 | 1 |

전제가 선다(형제 배틀몬스터 회차로 풀리지 않았다).

### ⒝ 1단계 — 그리기 호출 계수 대조 (`RUST_LOG=debug`, `--inject`, 한 판)
| 호출 | 놈3 (LGT · PASS · paints 163 · colors 99) | 슈퍼액션히어로 |
|---|---|---|
| `Graphics::drawImage` | 15,532 | **0** |
| `Graphics::drawLine` | 13,328 | **0** |
| `Graphics::fillRect` | 8,434 | **0** |
| `Graphics::setColor` | 21,762 | **0** |
| `Graphics::reset` / `translate` / `setClip` | 1,151 / 495 / 16,105 | 22 / 9 / 3 |
| `Image::createImage` | 48 | 1 |

슈퍼액션히어로에 남은 호출은 **호스트 자신의 paint 경로**(`CardCanvas::paint` → `Graphics::reset/translate/setClip`)뿐이다.
⇒ **판정: «그려도 화면에 안 붙는다»가 아니라 «그리기 호출이 없다».** 화면·blit·canvas 층이 아니다.

**그리기 호출이 없는 이유**: 게임의 `Thread.run()`(스레드 2)이 **첫 동작으로** import `(0x64, 0xfd)` 를 풀다가
`net.wie.WieError: Fatal error: Unknown lgt java import: 0xfd` 로 죽는다(`Uncaught exception` 1건, 3/3).
그리기 루프가 그 스레드에 있으므로 남는 것은 호스트가 그리는 빈 Card 뿐이다. upstream `dlunch/wie` 도 0xfd 를 모른다.

**코퍼스 도달**: `broken/lgt` 37파일 전수(무주입 10 s) — 0xfd 를 만나는 파일은 **이것 하나**.

### ⒞ 0xfd 의 정체 — 게임 binary.mod(ELF · 기호 없음) 디스어셈블
- 모든 import 는 `.data` 의 16바이트 ARM 트램폴린 `{push {lr}; bl resolver; table; index}` 이고, 리졸버(`0x4a9d4`)가
  `get_import_function(table, index)` 를 부른다. 0xfd 트램폴린 = `0x1401994`.
- `.text` 에서 그 주소를 가진 리터럴 풀 6곳 → 호출부 6곳 전부 같은 모양이다:
  `r0` = 클래스의 정적 필드(배열) · `r1` = 인덱스(0, 1, 또는 `0x97f0` 에서 증가하는 루프 카운터) · `r2:r3` = 64비트 값.
- ★**워드 순서는 high 먼저다.** `0x7b40` 이 `r5 = asr r4, #31` → `r2 = r5`, `r3 = r4` 로 int 를 long 으로 넓힌다 —
  부호 워드는 정의상 상위 워드다. Java 메서드 인자 코덱(`decode_method_arguments`)의 low-먼저와 **반대**다.
- 값: `(long)x`, 상수 `-1L`(`r2 = r3 = -1`), 네이티브 호출의 64비트 반환값 ⇒ long[] 로 판정. ★원소 타입을 클래스
  메타데이터에서 읽지는 않았다.
- ★**검사는 처리기 몫이다.** 0xfd 앞에서 어느 호출부도 null·범위 검사를 하지 않는데, 같은 함수가 바로 뒤의 byte[] 에는
  인라인으로 `0x22`(NPE)·`0x23`(AIOOBE)을 부른다. ⇒ null → NPE, 범위 밖 → `jvm.store_array` 의 AIOOBE.

### ⒟ 2단계 — 고쳤다: before / after (release · `--inject` · 3회씩)
| | stop | input | paints | distinct_colors | 게임 스레드의 죽음 |
|---|---|---|---|---|---|
| before | deadline ×3 | 27/27 | 3·3·2 | 1·1·1 | `Unknown lgt java import: 0xfd` 3/3 |
| after | **max-ticks** ×3 | **9·7·11 / 27** | 3·3·3 | 1·1·1 | `Unimplemented: java/io/DataInputStream vtable index 32` 3/3 |

스크린샷: `game_lab/reports-2026-09-24-superaction-import-0xfd/{before,after}_{1,2,3}.png`(git 무시 경로 — Constraint 9).
after 에서 0xfd 는 1회 풀리고, 스레드는 `ByteArrayInputStream` → `DataInputStream` 을 만든 뒤 import `0x55` 직후 vtable 32 에서 죽는다.
★**after 에서 입력이 27개 중 7~11개만 들어갔다** — 스레드가 죽은 뒤 실행기가 공회전으로 tick 상한(5천만)에 먼저 닿는다.
결과는 `FAIL`(균일색)이라 UNMEASURED 로 바뀌지 않지만, «27키를 견뎠다»는 주장은 after 에는 **없다**.

### 변이 검사 (양방향)
| 변이 | 결과 |
|---|---|
| 처리기 high/low 뒤바꿈 | `store_long_array_…` FAIL (`left: -7296712173568108936` ≠ `0x1234_5678_9abc_def0`) |
| null 가드 제거 | 같은 테스트 FAIL (`0x0[0] must throw into the guest`) |
| `try_from` 의 `35 => StoreLongArray` 행 제거 | `java_system_svc_ids_round_trip…` FAIL (`id 35 has no try_from row`) |
| 원상 | 셋 다 PASS |

★세 번째 테스트는 실제로 밟은 함정에서 나왔다: 처리기 단위 테스트는 green 인데 게스트에서는 `Unknown LGT Java system SVC id 35` 가 났다
(`try_from` 표가 enum 과 따로 손으로 적혀 있다).

### 회귀
- 4게이트: `fmt` rc0 · `clippy --all -D warnings` rc0 · wasm clippy rc0 · `cargo test --all` **424 passed / 0 failed** · `cargo +beta clippy` rc0.
- `cargo clippy --workspace --all-targets` 경고: base 와 **동일**(16줄, 전부 기존분 — `git stash` 로 짝지어 측정).
- runner 블록: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt --inject --expect-last-frame` PASS (paints 55·55, rc 0) · `text_j2me` PASS.
- `broken/lgt` 37파일 전후 대조(무주입 10 s): 판정이 바뀐 것은 `영웅서기4`(PASS→FAIL) 하나. 이 타이틀은 0xfd 를 **풀지 않는다**
  (debug 3회 0건)라 새 코드가 실행될 수 없고, 같은 바이너리로 다시 재면 **PASS 4/4**(10 s ×2 · 20 s ×2) ⇒ 부하 잡음.
  `크로이센` 은 전후 모두 JSON 출력 없음(기존). 부하 loadavg 13~38.

### 하지 않은 것
- `DataInputStream` vtable index 32 — 어떤 메서드인지 **재지 않았다.** LGT vtable ABI 계열(0220 의 index 25 와 같은 축)의 일이라 이 회차 범위 밖이다.
- 0xfd 를 double[] 에 쓰는 경우 — 관측 0건이라 다루지 않았다(`[D` 배열이 오면 `Long` 값이 들어간다).
