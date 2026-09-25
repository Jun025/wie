## [2026-09-26] LGT long/double 인스턴스 필드 — 호스트 읽기·쓰기를 게스트와 같은 high-first 로 (wie-2026-09-24-lgt-wide-field-second-word-link-adopt-p0)

**무엇을** — `class_instance.rs` 의 `get_field`/`put_field` 가 J/D 필드를 word n = **상위**, n+1 = **하위**로 읽고 쓴다(종전은 반대).
테스트 `wide_instance_field_is_stored_high_word_first` 1건.

**왜** — 제안 `2026-09-24-lgt-wide-field-second-word-link#p0`. 게스트(레전드오브마스터 `0x91f30`/`0x91f34`)는 `currentTimeMillis` 의
r1(상위)을 이름 붙은 칸에, r0(하위)을 다음 칸에 쓴다(`0244`). `long[]` 도 high-first 다(#275). 호스트만 반대였다.

**사용자 영향** — 지금 화면으로 보이는 변화는 없다(아래 ⒜: 그 경로를 지나는 호출부가 0 이다). 잠재 결함 제거다.

## 1. 반증부터

- ⒜ **호스트가 AOT 클래스의 J/D 인스턴스 필드를 get/put 하는 호출부: 0.** wie 전 크레이트에서 `get_field`/`put_field` 를 `"J"`/`"D"` 로 부르는 곳은
  `net/wie/EventQueue.lastRun`(Rust 클래스 · private) 2곳과 정적 필드(Player·MathFP — LGT 무관)뿐이다. `rustjava-runtime 0.1.1` 의 J/D 인스턴스 필드
  (Random·Date·Thread·Long·Double·TimerTask·ZipEntry·Calendar·LogRecord)도 전부 Rust 클래스이고 호스트 코드가 자기 필드를 만진다.
  ⇒ **판정: 순서만 맞추는 일이다.** 전제(«호스트가 AOT long 필드를 읽는 경로가 실재한다»)는 서지 않지만, 순서 자체는 게스트 근거로 확정돼 있어 맞췄다.
- ⒝ **Rust 클래스 필드의 현 워드 순서: low-first**(같은 `get_field`/`put_field`). 그 필드는 이 한 쌍으로만 오가므로 뒤집어도 왕복이 대칭이다.
  게스트가 볼 수 있는 Rust J/D 인스턴스 필드는 `Calendar.time`(protected) 하나이고, 게스트가 그것을 읽는다면 뒤집은 쪽이 맞다.
  ⇒ 티켓이 걱정한 «한쪽만 뒤집으면 다른 쪽이 깨진다»는 성립하지 않는다 — **두 계급을 가르지 않고 한 순서로** 뒀다(근거는 코드 주석 1줄).

## 2. 시험 — 되돌리면 red

순서만 `origin/main` 판본으로 되돌린 실행(원문):

```
thread 'runtime::java::jvm_support::tests::wide_instance_field_is_stored_high_word_first' panicked at wie-lgt/src/runtime/java/jvm_support.rs:1216:13:
  left: 2309737967
 right: 19088743
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 37 filtered out
```

(`2309737967` = `0x89abcdef` 하위 · `19088743` = `0x01234567` 상위.) 복원 → `test result: ok. 1 passed`.
시험은 세 방향을 본다: 호스트 쓰기 → 게스트 칸(J·D) · 게스트 쓰기(상위, 하위) → 호스트 읽기 · 호스트 왕복(D).

## 3. 레전드오브마스터 — 짝지은 전/후

base = `origin/main` `289c4290` release · fix = 이 브랜치 `6e0fb797` release. `--timeout 120`, base·fix 번갈아.
(`--timeout 20` 은 load 190 에서 fix 도 ticks 31 · paints 0 으로 부팅 전에 끝나 판정축에 못 닿았다 — 버린다.)

| 실행 | result · stop | paints | distinct / nondominant | WARN | load1 |
|---|---|---|---|---|---|
| base1 | PASS · deadline | 349 | 33 / 8.2 | 0 | 147 |
| fix1 | PASS · deadline | 273 | 33 / 8.2 | 0 | 153 |
| base2 | PASS · deadline | 485 | 33 / 8.2 | 0 | 111 |
| fix2 | PASS · deadline | 545 | 33 / 8.2 | 0 | 90 |
| base3 | PASS · deadline | 350 | 33 / 8.2 | — | — |
| fix3 | PASS · max-ticks | 148 | 33 / 8.2 | — | — |

마지막 프레임 PNG(`--screenshot`)는 base3·fix3 **바이트 동일**(md5 `74f504e4176fdc5fa178f79a0fe6cfc6`) — 문면은 «<이용안내> … 아무키나 누르세요.».
paints·ticks 는 부하로 흔들리고 방향도 섞였다(ticks 는 처리량이 아니다 — AGENTS.md). ⇒ **회귀 0**, ⒜ 와 정합.

## 4. 게이트

`RUST_MIN_STACK=4194304 cargo test --all` **448 passed · 0 failed** · `cargo clippy --all -- -D warnings` rc=0 · wasm clippy rc=0 ·
`cargo +beta clippy --all -- -D warnings` rc=0 · `cargo clippy --workspace --all-targets` 경고 16건 전부 이 diff 밖 파일(`wie-backend/src/canvas.rs`·
`wie-jvm-support/src/hardening.rs`·`wie_cli/tests/support/dod_ci_parity.rs`) — `wie-lgt` 0건.

## 5. 한계

- 정적 필드(`get_static_field`/`put_static_field`)는 무접촉. 게스트의 정적 long 저장 순서를 재지 않았고, 그 순서를 고정하는 `jvm_support.rs` 시험은
  업스트림(#1337)에서 왔다 — 게스트 근거가 없다. 후속 제안 `2026-09-26-lgt-wide-field-host-word-order#p0`.
- KTF `class_instance.rs` 는 범위 밖.

## 6. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus <코퍼스>` 결과는 BOUNDED **20회/12쌍** · SUFFIX-ATTACHED **2회/2쌍**이다.
이번 diff 가 새로 넣은 것은 `레전드오브마스터`(이 문서·worklog·시험 주석)뿐이고, 그 이름은 이미 `0244` 에 있었다.
나머지는 `jvm_support.rs` 에 원래 있던 줄이다(이 도구는 바뀐 파일의 «본문 전체»를 센다). SUFFIX 2건(`간호사타이쿤2`·`서든어택포켓`)도
원래 있던 줄에서 나왔고, 둘 다 조사가 아니라 «더 긴 다른 제목»이다.

