## [2026-09-29] 네이티브 필드 이름 조회가 게임 하위 클래스 필드에 가려진다 — 선언 클래스에서 찾기 (wie-2026-09-29-paint-serialize-card-bounds-adopt-p0)

**무엇을**: 네이티브 코드가 `this` 의 필드를 이름으로 읽고 쓰는 자리 중, **코퍼스에서 게임 하위 클래스가 같은 이름·같은 타입의 필드를 실제로 선언한 것**만 «선언 클래스에서 찾은 필드» 로 바꿨다 — `Card.display`, `MIDlet.display`, `Jlet.dis`. 헬퍼는 `wie-jvm-support` 의 `get_declared_field`/`put_declared_field` 한 곳이고, 0374 가 `Card` 안에 둔 `x/y/w/h` 전용 헬퍼도 이것으로 옮겼다. `jvm` 크레이트는 무접촉.
**왜**: 제안 `2026-09-29-paint-serialize-card-bounds#p0` — `jvm-0.1.1` `find_field` 는 런타임 클래스부터 찾으므로, 게임 하위 클래스가 같은 이름을 선언하면 네이티브 코드가 게임의 값을 읽는다(0374 의 `Card.x/w`).
**사용자 영향**: 이 코퍼스에서는 **보이는 변화 없음**(아래 전/후). 게임이 자기 필드에 다른 값을 쓰는 순간 네이티브 쪽이 따라 바뀌던 잠재 결함을 닫는다.

### 전수 계수 — 무엇을 셌나
정적 파싱은 KTF·LGT 에서 불가능하다(클래스가 ARM 이미지 안의 구조체다). 그래서 **스크래치 빌드의 `jvm` 사본**(`[patch.crates-io]` · 저장소에 넣지 않음)의 `register_class_internal` 에 탐침을 넣어, 클래스가 등록될 때마다 «자기 인스턴스 필드(name, descriptor) ∩ 조상 체인의 인스턴스 필드» 를 기록했다 — `find_field` 가 쓰는 키 그대로다. 조상이 wie 프로토(`name: "…/…"` 179개) 또는 `java/*` 인 것만 남겼다(게임끼리의 가림은 네이티브와 무관).
- 대상: `game_lab/{working,broken}/*` 464 경로 · **고유 429**(KTF 266 · SKT 82 · LGT 78 · 미분류 3) · 각 `wie_validate --inject` 1판(load1 ~450~660).
- 한계: **그 판에서 등록된 클래스만** 본다(KTF 31,308 · SKT 8,685 · LGT 7,100 등록). 키 스크립트가 닿지 않는 화면에서만 로드되는 클래스는 빠진다.

| 네이티브 필드 | KTF | SKT | LGT | 처분 |
|---|---|---|---|---|
| `Card.display:Lorg/kwis/msp/lcdui/Display;` | 13 | 1 | 0 | **이번에 고침** |
| `Card.x/y/w/h:I` | 10/7/6/6 | 0/0/1/1 | 0 | 0374 에서 고침(헬퍼만 옮김) |
| `MIDlet.display:Ljavax/microedition/lcdui/Display;` | — | 5 | — | **이번에 고침**(단 아래 ⚠) |
| `Jlet.dis:Lorg/kwis/msp/lcdui/Display;` | 2 | 0 | 0 | **이번에 고침** |

그 밖의 네이티브 클래스(Canvas·Displayable·Graphics·lwc Component·Font 등)는 **0** — 손대지 않았다. 게임이 상속한 네이티브 클래스는 `Card`·`Jlet`(KTF 259 · LGT 17~18) · `Canvas`·`MIDlet`(SKT 79) · lwc Component 소수(≤4) 였다. 고친 세 클래스 안에서도 **겹친 필드만** 바꿨다(`Card.canvas`·`Jlet.eq` 등은 0 이라 그대로).

### ⚠ SKT 의 private 필드는 «가림»이 아니라 «공유»다 — upstream
`jvm-bytecode-0.1.1` `ClassInstanceImpl` 은 인스턴스 저장소를 `BTreeMap<FieldImpl, _>` 로 두고, `FieldImpl` 의 순서는 (name, descriptor, access_flags) 내용 비교다(선언 클래스가 키에 없다). 그래서 `MIDlet.display`(private)와 게임의 **private** `display` 는 **같은 칸**이다 — 선언 클래스에서 필드를 찾아도 같은 칸을 가리킨다. SKT 5종의 플래그(클래스 파일 직접 파싱): private 3 · package 1 · public 1 ⇒ 이 변경이 닿는 것은 2종이고, 나머지 3종은 **`jvm-bytecode` 저장소 키에 선언 클래스를 넣어야** 풀린다. 고치지 않았다(upstream 크레이트). KTF·LGT 는 ARM 오프셋 저장이라 해당 없다.
같은 이유로 단위 시험의 가리는 필드는 네이티브와 **다른 접근 플래그**로 선언했다(같으면 시험이 «공유»를 재게 된다).

### 전/후 — 가림이 걸린 32종(위 표 전부) · `--inject` · 전/후/탐침 3개 동시 실행
전 = `origin/main`(`fbeb045a`) + 등록 탐침만 · 후 = 이 head · 탐침 = 후 + «선언 조회 값 ≠ 이름 조회 값» 기록.
- 판정: 32종 중 전/후 판정이 갈린 것은 1차 3건(`9c1c446a36e2` `f5bd7a91a107` 전 UNMEASURED→후 PASS · `65bace1623a7` 양쪽 UNMEASURED). 3판씩 재측하니 **전·후 모두 PASS/UNMEASURED(`clean exit`) 가 섞인다** ⇒ 부하 요동이지 이 변경이 아니다. `java_exceptions` 개수가 바뀐 타이틀 0.
- 값이 실제로 갈린 자리(탐침): `Card.display` `1793f87924d4` 44회 · `Jlet.dis` `232122cdfb92` 1회 · `4d6f8e78cf92` 2회 · `MIDlet.display` `ae877a276f59` 1회 · `ec2f8f2e02a2` 1회 — **전부 «선언 필드 = Display, 게임 필드 = null»**. 즉 게임은 자기 필드를 한 번도 쓰지 않았고, 전에는 네이티브가 `<init>` 에서 **게임의 칸에** Display 를 써 넣고 같은 칸에서 읽었으므로 값이 일치했다. 화면: t=15s 캡처가 `1793f87924d4`·`232122cdfb92`·`ec2f8f2e02a2` 에서 전/후 **바이트 동일**. `ae877a276f59` 는 4판 중 1판만 동일했으나 전끼리도 4판 모두 다르다(타이밍).
- ⇒ **이 코퍼스에서 화면 차이는 없다.** 고친 것은 «게임이 자기 필드에 다른 값을 쓰면 네이티브가 따라 바뀐다» 는 잠재 결함이고, 실기기 의미(private 필드는 별개)와도 이제 맞다 — 게임이 자기 `display` 를 읽으면 이제 null 을 본다(실기기와 같다). 그로 인한 예외 증가는 0.

### 검증
- 단위 시험 3건 추가 — `display_is_the_midlets_own_field_not_a_subclasss`(wie-midp) · `display_is_the_jlets_own_field_not_a_subclasss` · `card_display_is_cards_own_field_not_a_subclasss`(wie-wipi-java). 변이(저장 안 함): 헬퍼 두 함수를 `jvm.get_field`/`put_field` 이름 조회로 되돌리면 새 3건 + 0374 의 `card_bounds_…`(`left: 240 right: 3`) **4/4 red**.
- 4게이트 · beta clippy · 러너 블록 — 회신 파일에 결과.

### 게임 이름 유입
`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` — BOUNDED **1회 / 1쌍** · SUFFIX-ATTACHED **1회 / 1쌍**. 둘 다 `jlet.rs` 의 **이미 `origin/main` 에 있던 주석** 한 줄이다(도구는 수정 파일 본문 전체를 본다) — 이 변경의 추가 줄(`git diff origin/main...HEAD` 의 `+`)에서 세면 **0**. 표·본문은 sha 앞 12자만 쓴다.
