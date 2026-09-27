## [2026-09-27] LGT 정적 long/double 필드 — 게스트 저장 순서를 재고 호스트를 high-first 로 맞췄다 (wie-2026-09-26-lgt-static-wide-field-word-order-adopt-p0)

**무엇을** — `class_definition.rs` 의 `get_static_field`/`put_static_field` 가 J/D 를 word n = **상위**, n+1 = **하위**로 읽고 쓴다(종전 low-first).
업스트림(#1337)에서 온 `test_native_jvm_runtime` 의 정적 `long` 단언을 게스트 근거(아래 §1) 순서로 바꿨다.

**왜** — 제안 `2026-09-26-lgt-wide-field-host-word-order#p0`. 인스턴스 필드(`0269`)·`long[]`(#275)는 high-first 로 맞췄는데 정적 필드만 반대였고,
그 순서를 고정한 시험에는 게스트 근거가 없었다.

**사용자 영향** — 화면 변화 없음(§2: 그 경로를 지나는 호출부가 0). 잠재 결함 제거다.

## 1. 게스트 저장 순서 — 디스어셈블(임시 쓰기 감시 · 커밋 안 함)

`prepare_generated` 가 할당한 정적 저장소(`ptr_class_fields + 0x14`)에 32비트 쓰기 감시를 걸고, 인접 두 워드에 짝으로 쓰는 명령을 찾았다.
값 `0x1a0` 은 `currentTimeMillis` 상위 워드(2026-09 ≈ `0x1a0_xxxxxxxx`)다.

| 타이틀 | 모드 | 명령 | 쓴 값 | 정적 워드 |
|---|---|---|---|---|
| 놈3 | Thumb | `0x1cc6e bl` → `0x1cc7a str r1, [r2, #0x5c]` · `0x1cc7c str r0, [r2, #0x60]` | `0x1a0` · `0xe13163b8` | class `r` word 18·19 |
| 체스마스터 | Thumb | `0x1d68 bl` · `adds r7, r1, #0` · `adds r6, r0, #0` → `0x1d70 str r7, [r3, #0x74]` · `0x1d72 str r6, [r3, #0x78]` | `0x1a0` · `0xe131fea3` | class `a` word 24·25 |
| 배틀몬스터 | ARM | `0xd9df0 str r7, [r3, #0x90]` · `0xd9df4 str ip, [r3, #0x94]` | `0x1a0` · `0xe1315f3e` | class `o` word 31·32 |

AAPCS 로 `bl` 의 64비트 반환은 r0 = 하위 · r1 = 상위다. ⇒ **상위가 앞 워드, 하위가 다음 워드 — high-first.** 세 곳의 `r2`/`r3` 는 `[클래스 객체, #8]` =
`ptr_class_fields` 이고 오프셋 − `0x14` 가 정적 워드 색인과 맞는다(`0x60 − 0x14 = 4×19`).
반대 순서로 보인 후보는 스파이더맨3 `0x2e43c`/`0x2e446` 1건뿐이고, 디스어셈블하면 서로 다른 `int` 두 칸(`str r6, [r2, #0x14]` · `add` 뒤 `str r6, [r2, #0x18]`)이다 — 기각.

## 2. 호스트 호출부 — 0

- **게스트 클래스의 정적 필드는 호스트가 이름으로 부를 수 없다.** 같은 계측으로 LGT Java 18타이틀의 게스트 클래스 **200개**를 봤다: 정적 워드는 최대 96개까지 있지만
  필드 표(`ptr_fields`)에 STATIC 항목은 **0개**다. ⇒ `get_static_field`/`put_static_field` 는 게스트 정적 저장소에 닿지 않는다.
- **게스트가 호스트 클래스의 정적 J/D 를 import 하는 링크: 0**(같은 18타이틀 · `link_class_members` 정적 표).
- 호스트가 정적 J/D 를 쓰는 곳은 rustjava `Long.MIN/MAX_VALUE`·`Double` 5개(clinit) — 호스트 전용 저장소라 이 한 쌍으로만 오가 왕복이 대칭이다.
  wie 쪽 `MathFP.E/PI`(`wie-skvm`)는 SKT 라 LGT 무관.
- ⇒ **판정: `0269` 와 같다 — 순서만 맞추는 일이다.** 게스트 근거가 확정돼 있어 맞췄다.

## 3. 시험 — 되돌리면 red

`class_definition.rs` 만 `HEAD` 판본으로 되돌린 실행(원문):

```
thread 'runtime::java::jvm_support::tests::test_native_jvm_runtime' panicked at wie-lgt/src/runtime/java/jvm_support.rs:1223:13:
  left: 2596069104
 right: 305419896
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 60 filtered out
```

(`2596069104` = `0x9abcdef0` 하위 · `305419896` = `0x12345678` 상위.) 복원 → `ok. 1 passed`. 시험은 호스트 쓰기 → 게스트 칸 · 게스트 쓰기(상위, 하위) → 호스트 읽기 두 방향을 본다.

## 4. 게임 — 짝지은 전/후

base = 이 브랜치에서 `class_definition.rs` 만 `HEAD` 로 되돌린 release · fix = 이 브랜치 release. `--timeout 60`, base·fix 번갈아 2쌍(load1 58~129).

| 타이틀 | base | fix |
|---|---|---|
| 체스마스터 | PASS · paints 2/2 · 36색 · 17.6% · 예외 0 · PNG `c05498a8` ×2 | 동일 · PNG `c05498a8` ×2 |
| 놈3 | PASS · paints 1095/1021 · 3색 · 예외 1 | PASS · paints 1400/1145 · 3색 · 예외 1 |
| 배틀몬스터 | PASS · paints 282/286 · 19색 · 25.8% · 예외 1 · PNG `e989d5e8` ×2 | PASS · paints 661/394 · 동일 · PNG `e989d5e8` ×2 |

놈3 는 60초 2쌍에서 PNG 가 base `eccc9be2` ↔ fix `0345d70e` 로 갈렸다. 두 프레임은 «아무키나 누르세요!!» 한 줄의 유무만 다르고(깜빡임),
45·50·55초로 다시 재니 **양쪽 다 두 프레임이 모두 나왔다**(base `0345d70e`/`eccc9be2`/`0345d70e` · fix `eccc9be2`/`eccc9be2`/`eccc9be2`) — 캡처 위상이다.
paints 차이는 부하다. ⇒ **회귀 0**, §2 와 정합.

## 5. 게이트

fmt · `cargo clippy --all -- -D warnings` · wasm clippy · `cargo +beta clippy --all -- -D warnings` 전부 rc=0.
`RUST_MIN_STACK=4194304 cargo test --all` **496 passed · 0 failed**.

## 6. 한계

- KTF 쪽 정적 필드는 범위 밖.
