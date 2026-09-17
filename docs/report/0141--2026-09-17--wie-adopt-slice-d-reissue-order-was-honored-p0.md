## [2026-09-17] #159 의 개악 대조가 «존재하지만 돌지 않았다» — 살렸다 (wie-adopt-slice-d-reissue-order-was-honored-p0)

**무엇을**: 고아가 된 가드 `wie_wipi_java/tests/preload_classes_come_from_the_runtime.rs` 를
**`wie-wipi-java/tests/`(= 실제 workspace member)로 옮기고** 새 base API 에 맞춰 **한 줄** 고쳤다.
★**제품 코드 0줄**(개악 대조 3회는 전부 복원) · 빈 `wie_wipi_java/` 디렉터리 제거.

**왜**: 채택 제안 `2026-09-16-slice-d-reissue-order-was-honored#p0` — 「#159 가 세운 개악 대조가 고아가 되어
돌지 않는다 — **살리거나, 죽었다고 선언하거나**」.

---

## ⓐ 제안이 «지금도» 참인가 — ★**전건 참**(직접 재측)
| 주장 | 실측 | 판정 |
|---|---|---|
| 파일이 3,136B 로 남아 있다 | `wie_wipi_java/tests/…rs` **3,136B** 실재 | 참 |
| workspace member 가 아니다 | `Cargo.toml` members = **`wie-wipi-java`**(하이픈) · `wie_wipi_java` **0건** | 참 |
| `cargo test --all` 에 그 이름 **0회** | 이 base(`eab1d567`) 기준 baseline **42타깃 / 385 passed** · 그 이름 **0회** | 참 |
| 컴파일되지 않는다 | 옮겨 빌드 → `E0061` **1건**(`invoke_virtual` 인자 5개인데 4개) | 참 |

★★**그리고 제안이 «말하지 않은» 한 가지를 더 쟀다 — 이것이 이 회차를 «살리는» 쪽으로 기울였다**:
`wie-wipi-java/src/lib.rs` 의 주석이 ★**「Locked by `tests/preload_classes_come_from_the_runtime.rs`」**라고
**단언**하고 있었다. ⇒ 소스가 «잠겨 있다»고 적은 잠금이 **한 번도 돌지 않았다** — 이 저장소가 반복해 이름 붙인
「검사가 있다 ≠ 검사가 돈다」의 교과서적 형태이고, ★**옮기는 순간 그 문장이 «참»이 된다.**

## ⓑ ⒜(살린다) ↔ ⒝(죽었다고 선언한다) — ★**판단 근거는 제안이 정한 그 축이다**
제안: 「판단 근거는 **그 preload 경로가 오늘도 회귀 가능한가**」.
★★**오늘이 «더» 회귀 가능하다** — 그 축이 결정적이었다: #159 당시 런타임은 **git `rev` 핀**(`dlunch/RustJava@5b84dd1`)이었는데
지금 `Cargo.toml` 은 ★**`rustjava-runtime = { version = "^0.1.1" }`**(crates.io 세버 범위)다.
⇒ ★**semver 호환 범프가 런타임이 등록하는 클래스를 바꿔도 이 저장소의 다른 어떤 축도 그것을 보지 않는다.**
⇒ **⒜ 살린다.** ⒝(삭제)는 「가드가 산 보증을 잃는다」는 대가를 **핀이 느슨해진 시점에** 치르는 것이라 역방향이다.

## ⓒ 되살리는 비용 — **실측 «한 줄»**(제안이 「미측정」이라 적은 그 수)
`invoke_virtual(&instance, "printStackTrace", "()V", ())` →
`invoke_virtual(&instance, **"java/lang/Throwable"**, "printStackTrace", "()V", ())`.
★**아무 문자열이나 채운 것이 아니다** — `jvm-0.1.1` 의 그 인자는 **해석(resolution) 클래스**이고(`resolve_class(class_name)`
→ 인스턴스 계층에서 override 선택), `java/lang/Throwable` 로 해석하는 것이 **원 주석이 적은 단언 그대로**다
(「같은 이름의 비-throwable 이면 안 된다」). 그 밖 수정 **0줄**.

## ⓓ ★개악 대조 — **제품 호출부 2곳이 물었고, 안 문 곳 1곳도 적는다**
| # | 개악(제품 파일) | 결과 |
|---|---|---|
| **MUT2** | `wie-jvm-support/src/runtime.rs` — 런타임이 `java/io/InterruptedIOException` 을 **안 준다** | ★**FAILED** · `JavaException(java/lang/NoClassDefFoundError)` |
| **MUT3** | 같은 파일 — **판별자** `java/lang/VirtualMachineError` 를 **안 준다** | ★**FAILED**(예외 클래스 단언이 깨진다) |
| 복원 | — | **ok · 1 passed** |
| ★**MUT1(안 물었다)** | `wie-jvm-support/src/lib.rs` — `java.class.path` 에서 **`RT_RUSTJAR` 제거** | ★**여전히 ok** |

★★**MUT1 의 «음성»을 숨기지 않는다 — 그것이 다음 사람에게 필요한 정보다**: 이 가드가 무는 자리는
**`java.class.path` 문자열이 아니라 `find_rustjar_class` 의 RT 분기**다. 클래스패스에서 런타임 항목을 빼도
그 넷은 여전히 해석됐다 ⇒ ★**「클래스패스 순서」로 이 축을 시험하려는 다음 회차는 헛다리를 짚는다.**
(왜 그런지는 이 회차의 범위 밖이라 **판정하지 않았다** — 관측만 남긴다.)

## ⓔ 잃는 것 / 안 하면
- **잃는 것**: `cargo test --all` 에 타깃 **1개 · 테스트 1개**가 는다(실측 **42/385 → 43/386**). 실행 시간 **0.07~0.19s**.
  그리고 ★**런타임이 그 넷 중 하나를 안 주는 날 CI 가 빨개진다** — 그것이 목적이지 비용이 아니다.
  ★**진짜 대가는 따로 있다**: 이 가드는 `rustjava-runtime` 의 «현재 동작»에 묶여 있어, 상류가 의도적으로 바꾸면
  **우리 PR 이 red 로 그것을 알게 된다**(원인은 우리 diff 가 아니다). 그때 할 일은 «이 파일을 지우는 것»이 아니라 **핀·기대를 갱신**하는 것이다.
- **안 하면**: 지금 그대로 — 소스가 「Locked by」라고 적은 가드가 **영원히 안 돈다**. ★게다가 그 문장 때문에
  다음 사람은 **덮여 있다고 믿는다**(그것이 고아 가드가 단순 삭제보다 나쁜 이유다).

## ⓕ 검증
`fmt`·`clippy --all -D warnings`·wasm·**`+beta`** 전건 **rc=0** ·
`RUST_MIN_STACK=4194304 cargo test --all` **rc=0** — ★**43타깃 · 합산 386 passed · 0 failed**
(같은 base 의 baseline = **42 / 385** ⇒ ★**정확히 +1 타깃 · +1 테스트**, 그 밖 변동 0) ·
검사기 8종 rc=0 · `npm run audit` PASSED.

**사용자 영향**: KTF 게스트 부팅 시 `MExe_init` 이 미리 올리는 네 클래스가 **런타임에서** 해석되는지를
CI 가 다시 본다. 그 경로가 깨지면 게스트는 **부팅 자체가 중단**되는데, 지금까지는 그 가드가 «파일로만» 있었다.
