## [2026-10-11] LGT 온라인 등록 뒤 호스트 패닉 — paint 안 serviceRepaints 가 paint 를 끝없이 중첩했다 (wie-lgt-1a69522a7d43-online-register-guest-exception-host-panic)

제안 `2026-10-11-progression-stuck-101-ledger-round3#p1` 채택. 대상 `1a69522a7d43`(LGT · aot-java). 핀 `6ea6a6bd`.

**무엇을**: 같은 스레드의 paint() 안에서 부른 serviceRepaints 가 paint 를 한 겹 더 열지 않고, 그 repaint 를 다음 틱으로 미룬다(`Display::handle_paint_event` 한 줄 · 다른 스레드에서 온 paint 와 같은 처리).
**왜**: 그 중첩이 끝없이 깊어지며 게스트 힙을 채웠고, 마지막에 NPE 객체 하나를 만들지 못해 호스트가 패닉했다.
**사용자 영향**: 랭킹 → ＃ 온라인 등록 → «예»를 골라도 게임이 꺼지지 않는다. 등록은 여전히 안 되고 그 안내 화면에서 멈춘다(§3) — compat `knownIssues_ko` 한 줄 · 업데이트 소식 한 건.

### 1. 순서 — 힙 실패가 먼저이고, 그 원인은 paint 중첩이다

- `jvm.rs:943` 은 `Jvm::exception()` 안의 `new_class(...).unwrap()` 이다. 예외 객체를 **만드는** 할당이 실패해 패닉한다. 그 예외는 `org.kwis.msp.lcdui.Graphics::draw_image` 의 «image is null» NPE(게임이 null 이미지를 그린다 · 평소에도 난다)다.
- 그 직전 힙: 16바이트 칸 **524288/524288**(나머지 칸·목록 할당기는 여유). 할당 지점 표본(997번째마다 · 임시 탐침): 거의 전부 호스트 쪽 `JavaArrayClassInstance::new`·12바이트 인스턴스.
- «예» 직후 16바이트 칸이 3초에 약 15만 칸씩 늘었다(탐침: 114,432 → 264,344 → 414,278).
- 패닉 시점 Rust 백트레이스: `Display.handlePaintEvent → Canvas.handlePaintEvent → CardCanvas.paint → (게스트 paint) → Card.serviceRepaints → Canvas.serviceRepaints → Display.serviceRepaints → Display.handlePaintEvent → …` 가 수백 겹이다. 게임의 paint() 는 repaint() 뒤 serviceRepaints() 를 부르고, 우리 Display 는 같은 스레드면 그 자리에서 paint 를 다시 열었다. 겹마다 만든 객체가 열린 프레임에 묶여 GC 가 못 걷는다.
- ⇒ ★`0266ca417880`(타이머 콜백 GC 누락 · 0490·0492) 과는 다른 병이다. 거기는 수거 호출이 없었고, 여기는 수거가 돌아도 걷을 수 없었다.
- ⇒ 할 일 2의 «JavaException 을 게스트 예외로 돌려줄 자리»는 없다: 실패하는 것은 예외를 **만드는** 할당이고 그 unwrap 은 고정 핀(Constraint 8) 안이다. 그래서 할당이 차지 않게 했다. 예외를 삼키지 않았다 — NPE 는 그대로 게임에 간다.

### 2. 고친 것

`wie-midp/src/classes/javax/microedition/lcdui/display.rs` `handle_paint_event`: `!painting.is_null() && painting != current` → `!painting.is_null()`. paint 가 진행 중이면 어느 스레드든 `repaintPending` 을 세우고 `request_redraw()` 뒤 돌아간다. `paint_serialized` 의 기존 주석(«Repaints requested during painting belong to the next cycle»)과 일치한다.

- 시험: `canvas.rs` `service_repaints_paints_only_a_pending_repaint` 에 «paint 안에서 repaint() 후 serviceRepaints()» 단계를 더했다(1회만 그리고 `repaintPending` 이 남는다).
- 되돌리면 red: 고친 줄을 되돌린 빌드에서 `left: 9, right: 3`(«serviceRepaints inside paint nested a paint») 로 실패. 고친 빌드는 통과.

### 3. 남는 것 — 등록 안내에서 멈춘다(엔진이 아니라 게임의 통신 실패 길)

- 고친 빌드에서 «예» 뒤: 패닉 없음 · 화면은 «접속 시 통화료…» 그대로 · `Network::disconnect()` 수천 회 · CLR 무반응.
- 통신 스레드(스레드 4): `Network.connect()` = -1 → 게임 코드가 `adds r0,#1; bne`(0x6d9b2)로 -1 을 비교해 스트림을 **열지 않고** 1 을 돌려준다(0x6db6e). 이어 요청 76바이트를 쓰려다 스트림 필드가 null 이라 게임이 스스로 NPE 를 낸다(import 0x22 · lr 0x6df45). 그 try(0x6ded2)의 catch 는 형 검사 1회 뒤 다시 던지고, 바깥 catch 가 없어 스레드가 끝난다. 예외 프레임 걷기는 정상이다.
- 주 스레드는 paint() 안에서 그 스레드의 결과를 기다리며 돈다(탐침: 같은 스레드 미룸 315회 · 다른 스레드 미룸 3266회). 고치기 전에는 이 기다림 한 바퀴마다 paint 가 한 겹 중첩되어 패닉으로 끝났다.
- 네트워크 미지원 규칙(`connect()` = -1 · 소켓 열기 `SchemeNotFoundException`)은 **바꾸지 않았다**. 시험 빌드에서 `connect()` 만 0 으로 바꾸면 소켓 열기가 «Network is not supported» 를 받고, 게임이 그것을 잡아 «연결에 실패하였습니다. 잠시 후 이용해 주시기 바랍니다» → 확인 → 주 메뉴로 돌아온다(paints 1607 · 멈춤 없음). 그러나 같은 클래스를 싣는 타이틀이 160종(KTF 143 · LGT 16 · SKT 1)이고 0448 의 KTF `dbd078113b97` 은 -1 을 실패로 비교한다 ⇒ 전수 짝 프로브가 먼저라 worklog 카드 1장으로 넘긴다.

### 4. 측정

| 항목 | base(`6ea6a6bd`) | head |
|---|---|---|
| 대상 · 랭킹→＃→예→확인(`--keys` · 60초) | **FAIL** «panic during '12_OK'» · 16칸 524288/524288 | PASS · 패닉 0 · 마지막 화면 = 등록 안내 |
| 시험 `service_repaints_paints_only_a_pending_repaint` | FAIL(paints 9) | PASS |

짝 프로브(`--inject` · 30초 · `--pacing 8` · `--max-ticks 1e11` · 한 번에 1개 · build-slot):

| sha12 | base | head |
|---|---|---|
| `13d7e3c21856` | PASS/deadline · paints 298 · steps 27 | PASS/deadline · 298 · 27 |
| `1b107b96bf4e` | PASS · 349 · 27 · 예외 6 | PASS · 351 · 27 · 6 |
| `4ece6eeeaa04` | UNMEASURED/clean exit · 12 · 8 | 같음 |
| `a30bbe008b5e` | PASS · 293 · 27 · 1 | PASS · 292 · 27 · 1 |
| `b475b6399684` | PASS · 528 · 27 · 1 | PASS · 529 · 27 · 1 |
| `49ade89578c5`(가드) | PASS · 1180 / 1878 / 1860 | PASS · 1862 / 1168 / 1154 |
| `ddd885583b15`(가드) | PASS · 855 · 27 | PASS · 871 · 27 |
| `1a69522a7d43`(대상) | PASS · 886 · 27 | PASS · 903 · 27 |

- 판정·stop·content·steps 바뀐 것 0. `49ade89578c5` 의 paints 는 두 빌드 모두 약 1170 / 약 1870 두 갈래로 나뉜다(빌드 차이 아님).
- 넓은 짝 프로브(무작위 40종 + 위 8종 · census `--only probe`)는 다른 회차의 장시간 census 가 호스트 잠금(`/tmp/wie-playability-census.lock`)을 연달아 잡아 돌지 못했다. 기다리는 동안 내 long 임대가 놀아 그 판은 내렸다.
- 대상 6축 + `progress 600`: census 를 잠금 뒤에 걸어 두었다(아래 «6축» 절 · 결과가 오면 채운다).
- 네 게이트: fmt OK · clippy stable/wasm32 `-D warnings` OK · beta clippy rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` 775 통과 0 실패. 러너 블록 6줄 PASS(keydraw KTF paints 79 · LGT 55).

### 5. 6축

(대기 중)

### 6. 장부

- `compat.json` `1a69522a7d43` `knownIssues_ko` 1줄 · census `HAND_NOTE` 같은 줄(다음 `report` 가 같은 값을 다시 만든다).
- `docs/player-updates/2026-10-11-lgt-online-register-no-crash.json`(fix).
- 0507 원장 csv 72행 note · basis 0511.
- worklog `docs/worklog/2026-10-11-lgt-1a69522a7d43-nested-paint-host-panic.json`: adopted `…round3#p1` · 카드 1(`connect()` 반환값 전수).

유입: BOUNDED 333쌍은 전부 `compat.json` 의 기존 행(이 회차가 바꾼 것은 `1a69522a7d43` 의 `knownIssues_ko` 한 줄 · 제목 0) · 그 파일을 뺀 6파일은 BOUNDED 3쌍(`display.rs` 기존 주석 · 이 회차 diff 에 0회) · SUFFIX-ATTACHED 1(census 의 기존 «턴종료» 줄).

<!-- corpus-name-inflow v1 subjects=7 tree=500e503c8b1fcbd8 B=722/333 P=1/1 S=38/18 -->
