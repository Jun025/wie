## [2026-09-29] 스프라이트 시트가 통째로 겹쳐 그려진다 — 동시 paint 직렬화 · Card 경계 필드 가림 (wie-2026-09-29-lwc-remove-all-components-adopt-p0)

**무엇을**: ⑴ `javax.microedition.lcdui.Display::handlePaintEvent` 가 **한 번에 하나씩** 돈다 — 다른 스레드의 paint 가 진행 중이면 기다리지 않고 `repaintPending` 으로 남겨 다음 틱에 그린다(같은 스레드의 재진입 — paint 안의 `serviceRepaints` — 은 그대로 들어간다). 첫 판은 모니터였고 게이트② 에서 LGT 교착으로 반려됐다(아래 «반려 승계»). ⑵ `org.kwis.msp.lcdui.Card` 의 `x/y/w/h` 를 이름이 아니라 **`Card` 클래스에서 찾은 필드**로 읽고 쓴다(`getX`·`getWidth`·`move`·`resize`·생성자 등 네이티브 전부). 타이틀은 sha 앞 12자로만 적는다.
**왜**: `docs/report/0369` 가 남긴 한계 — `ca7fa8ade8ad` 가 키 5 이후 스프라이트·글꼴 시트를 통째로 그린다.
**사용자 영향**: `ca7fa8ade8ad` 대화·메뉴 화면이 겹침 없이 나온다. `34ab350dc98a` 는 흰 화면(+조각) → **실제 게임 화면**(지도·배·퀘스트 창). `249e655147a1` 은 키 입력 중 panic(3/3) → PASS(3/3).

### 원인 ⑴ — 두 paint 가 한 `screenGraphics` 를 번갈아 쓴다 (lcdui · 공용 경로)
`RUST_LOG=wie_midp=debug,wie_wipi_java=debug` 로 한 판을 떠서 kwis `Graphics` 호출 ↔ 바로 아래 midp 호출을 짝지어 셌다.
- 게임 쪽 `setClip` 은 **전부 작은 사각형**이었다(시트를 그리는 `drawImage` 앞의 kwis 클립이 화면 전체였던 경우 0). 그런데 midp 층에서는 `drawImage` 16,824건 중 **96건**이 «다른 kwis `Graphics` 가 마지막으로 바꾼 클립» 위에서 그려졌고, 그중 9건은 직전 동작이 `reset()`(= 화면 전체 클립)이었다.
- 모든 kwis `Graphics` 가 midp `Graphics` **하나**(`Display.screenGraphics`)를 감싼다(`net.wie.CardCanvas::paint` 가 paint 마다 새로 감싼다). 로그에서 paint A(게임 스레드의 `serviceRepaints`)가 그리는 도중 `EventQueue::dispatchEvent → Display::handlePaintEvent` 로 paint B 가 시작되고, B 의 `reset()` 이 A 가 막 건 클립을 지운다 → A 의 다음 `drawImage(시트, -114, 151)` 가 **시트 전체**를 그린다.
- 즉 «원본 영역 잘라내기 누락» 이 아니라 **클립 상태 경합**이다. KTF 네이티브·lwc 경로는 관계없다.

### 원인 ⑵ — 게임의 `Card` 하위 클래스가 `x`·`w` 를 가린다 (⑴ 을 고치자 드러났다)
- KTF 전수 짝(아래) 1차에서 `34ab350dc98a` 가 PASS 3/3 → FAIL 3/3(흰 화면)으로 뒤집혔다. 로그: 매 paint `translate(240, 0)` · `clipRect(0, 0, 0, 320)` — 카드가 화면 밖, 폭 0. 전에도 똑같았다(전/후 로그 모두 `translate(240,0)` 670/509건). **전의 PASS 는 ⑴ 의 경합이 화면 밖 그림을 화면에 새게 한 것**이었다(전 캡처 = 흰 바탕에 조각 몇 개).
- `Card.move` 호출은 0건. `Card::getX` 에 임시 탐침을 넣어 런타임 클래스를 찍으니 게임의 `b`(난독화) 가 자기 필드 **`w:I`·`x:I`** 를 선언하고 있었다. `jvm.get_field(this, "x")` 는 런타임 클래스부터 찾으므로(`jvm-0.1.1` `find_field`) 게임의 x(240)·w(0) 를 돌려줬다.
- 정본 javadoc(`docs/reference/AromaWIPI_javadoc.zip` `Card.html`)에서 `x`·`y`·`w`·`h` 는 **protected** — 하위 클래스가 같은 이름을 선언해도 합법이고, Card 자신의 코드는 Card 의 필드를 뜻한다. 그래서 `Card` 에서 필드를 찾아 인스턴스에 읽고 쓴다.

### KTF 전수 짝 — `game_lab/{working,broken}/ktf` 286 파일(고유 266) · `--inject` · 전/후 동시 실행(같은 부하)
| | PASS | FAIL | UNMEASURED | JSON 없음 |
|---|---|---|---|---|
| 전 `origin/main` | 201 | 32 | 31 | 2 |
| 후 (⑴+⑵) | 207 | 30 | 27 | 2 |

load1 **287~446**. PASS↔FAIL 로 뒤집힌 것과 content 가 바뀐 것을 전/후 3판씩 다시 쟀다:
| sha12 | 전 | 후 | 판정 |
|---|---|---|---|
| `249e655147a1` | FAIL 3/3 (panic `Option::unwrap`) | PASS 3/3 | 개선 |
| `34ab350dc98a` | PASS(흰 화면+조각) | PASS 3/3 · 게임 화면 | 개선(⑵) |
| `60704ad61a47` | PASS · content false 2/3 | PASS · content true 3/3 | 개선 |
| `d552e095ddcf` | PASS 3/3 | PASS 3/3 | 1차 FAIL 은 부하 |
| `6103e87874c6` | 동시 10판 FAIL 5/10 | 동시 10판 FAIL 3/10 | 기존 경합 — 아래 |
| `2b1ed0c8d061` | FAIL 2/3 (ARM `address 0`) | FAIL 1/3 | 기존 · 무관 |

`6103e87874c6` 은 부하가 클 때만 `MasterCard.paint` 안에서 `Invalid memory access; address: 76` 로 죽는다. 이벤트 스레드의 paint 가 그리는 도중 게임 스레드가 키 처리(`Player.stop` → `repaint`)로 상태를 바꾸는 모양 — paint 끼리가 아니라 **게임 로직 ↔ paint** 경합이라 이 변경 범위 밖이다. 같은 부하에서 전·후 모두 난다(위 표).

### 판정기 — 이 겹침을 `wie_validate` 가 잡게 하는 안
넣지 않았다. «마지막 프레임이 알려진 시트 이미지와 대량 일치» 는 시트가 게임 바이트라 기준 이미지를 저장소에 둘 수 없고(Constraint 9), 실행 중 로드된 이미지 전부와 프레임을 대조하는 방식은 정상 화면(배경 한 장을 그대로 그리는 게임)도 같이 문다. 이번 원인은 **클립 경합**이었으므로 증상보다 원인을 세는 편이 싸다 — ⑴ 뒤로는 구조상 0 이라 새 계수의 값어치가 없다. 다시 필요해지면 «paint 중 다른 kwis `Graphics` 가 같은 midp `Graphics` 의 클립을 바꾼 횟수» 가 로그 짝짓기로 이미 잴 수 있는 신호다(이 회차가 쓴 방법).

### 검증
- 단위 시험 2건 + 변이(저장 안 함): `a_paint_from_another_thread_neither_waits_nor_cuts_into_the_one_in_progress`(wie-midp) — 가드 제거 시 `left: 320 right: 2`, 모니터로 되돌리면 `the other thread's paint returned` 로 red(아래). `card_bounds_are_cards_own_fields_not_a_subclasss`(wie-wipi-java) — `bound` 를 이름 조회로 되돌리면 `left: 240 right: 3` 로 red.
- `ca7fa8ade8ad` release `--inject --keep-timeout --timeout 40 --pacing 8 --max-ticks 1000000000 --shot-every 5`: PASS · 키 27/27 · 캡처에서 대화창·저장 메뉴가 겹침 없이 나온다(캡처는 저장소에 넣지 않는다).
- 4게이트 · beta clippy · 러너 블록 6줄 PASS(`keydraw_*` rc=0). 첫 판은 LGT·SKT 코퍼스 짝을 재지 않았고, 그게 아래 교착을 놓친 자리다.

### 반려 승계 — 모니터가 LGT `b475b6399684` 를 교착시켰다 (wie-2026-09-29-lwc-remove-all-components-adopt-p0-fix)
- 검수 실측: 이벤트 스레드가 paint 중(게임 `paint()` 안) → 게임 스레드의 `repaint → serviceRepaints → handlePaintEvent` 가 모니터에서 대기 → 게임 `paint()` 는 그 게임 스레드의 진행을 기다린다 = **락 순서 역전**. 첫 판 head 는 4/4 무프레임, base 는 4/4 렌더.
- 처방 ⒜ 채택: `Display.paintingThread` 에 그리는 스레드를 적고, 다른 스레드가 들어오면 **막지 않고** `repaintPending = true` + `request_redraw` 로 돌려보낸다. 더티 영역은 건드리지 않으므로 다음 paint 의 클립에 그대로 남는다. 같은 스레드 재진입은 이전 값을 복원하는 식으로 그대로 통과.
- ⒝(paint 마다 `screenGraphics` 클립·translate 저장·복원)를 버린 이유: 경합은 paint **도중**에 난다 — A 가 클립을 걸고 양보한 사이 B 의 `reset()` 이 들어오면, B 가 끝나며 복원해도 A 가 그 사이 그린 `drawImage` 는 이미 B 의 클립 위였다. 모든 양보 지점을 감쌀 수 없으니 ⒝ 는 원인을 닫지 못한다.
- 대가: MIDP `serviceRepaints` 는 «대기 중인 repaint 를 처리할 때까지 막는다» 인데, 다른 스레드가 그리는 중이면 이제 바로 돌아온다(그 프레임은 다음 틱). 모니터 도입 전에는 두 paint 가 동시에 돌았으므로 그보다 나빠지는 경우는 없다.
- 시험: 첫 paint 가 다른 스레드의 `handlePaintEvent` 반환을 (상한 50회 양보로) 기다리는 형태 — 모니터로 되돌리면 그 반환이 오지 않아 red, 가드를 빼면 클립이 5 로 바뀌어 red. 이어 `serviceRepaints` 한 번에 밀린 paint 가 그려진다.

전/후 동시 실행 `--inject --timeout 40` · 전 = `b9fd0fc0`(merge-base) · 후 = 이 head · load1 **172~252**. 판정이 바뀐 건 중 PASS↔FAIL 은 전/후 3판씩 다시 쟀다.

KTF 전수 짝 — `game_lab/{working,broken}/ktf` 286 파일(고유 266):
| | PASS | FAIL | UNMEASURED | JSON 없음 |
|---|---|---|---|---|
| 전 | 239 | 29 | 18 | 0 |
| 후 | 240 | 26 | 20 | 0 |

LGT·SKT 최소 짝 — `game_lab/broken/{lgt,skt}` 71 + KTF 표본 4(`34ab350dc98a` `60704ad61a47` `249e655147a1` `ca7fa8ade8ad`) = 75, 검수와 같은 집합:
| | PASS | FAIL | UNMEASURED | JSON 없음 |
|---|---|---|---|---|
| 전 | 57 | 11 | 6 | 1 |
| 후 | 56 | 11 | 7 | 1 |

1차 PASS→FAIL 2건 재측(3판·동시):
| sha12 | 전 | 후 | 판정 |
|---|---|---|---|
| `70d709c40e10` (LGT) | FAIL 3/3 | FAIL 3/3 | 기존 — `Unknown lgt java import: 0x64`(#405 가 다루는 벽) · 1차 전 PASS 는 키가 벽에 닿기 전에 끝난 것 |
| `2b1ed0c8d061` (KTF) | FAIL 2/3 | FAIL 1/3 | 기존 — ARM `Invalid memory access` · 첫 판 회차에도 같은 요동 |
| `b475b6399684` (LGT · 반려 표본) | PASS 3/3 | PASS 3/3 (paints 142/182/200) | 교착 해소 — 첫 판 head 는 0 paints 4/4 |

⇒ 재측 후 PASS→FAIL **0**. 나머지 뒤집힘은 PASS↔UNMEASURED(`clean exit` — 키 스크립트 전에 게임이 끝남)가 양방향(PASS→UNMEASURED 8 · UNMEASURED→PASS 5)이고, FAIL→PASS 4(`b22a7fcfb406` `c7f543c73b91` `6103e87874c6` `249e655147a1`) 중 뒤 둘은 첫 판과 같은 방향이다. `ca7fa8ade8ad` 는 양쪽 PASS(245/263 paints). 원 대상 겹침: 후 head release `--inject --keep-timeout --timeout 40 --pacing 8 --max-ticks 1000000000 --shot-every 5` PASS · 키 27/27 · 키 3~8 캡처(대화창 4장·성 앞·성 안)와 키 26·27 저장 메뉴에 시트 겹침 없음(캡처는 저장소에 넣지 않는다).

### 게임 이름 유입
`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` — BOUNDED **4회 / 3쌍** · SUFFIX-ATTACHED **0**. BOUNDED 4회는 전부 `display.rs` 에 **이미 `origin/main` 에 있던 주석**이다(도구는 수정 파일 본문 전체를 본다) — 이 변경의 추가 줄(`git diff origin/main...HEAD` 의 `+`)에서 세면 **0**.

<!-- corpus-name-inflow v1 subjects=5 tree=5773b623a82fd875 B=4/3 P=1/1 S=0/0 -->
