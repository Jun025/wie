## [2026-09-26] 귀신사냥2007(KTF) SIOOBE — 단말 문자열이 아니라 KTF `call_native` 가 `long` 상위 워드를 버린 것 (wie-2026-09-24-wec-systheme-oemdevice-no-branch-minimal-stub-adopt-p1)

채택 제안 `2026-09-24-wec-systheme-oemdevice-no-branch-minimal-stub#p1` 의 회차다.

**무엇을** — KTF JB 인터페이스 슬롯 `0x30`(`call_native`)이 네이티브 반환의 **r1 을 버리고 결과 칸 `[ptr+4]` 에 0 을 쓰던 것**을 r0:r1 그대로 쓰게 했다(`wie-ktf/src/runtime/java/interface.rs` · upstream 도 같은 줄이다).
`wie-core-arm` 에 `RunFunctionResult<(u32, u32)>` 한 impl 을 더했다.

**왜** — 제안의 전제 「단말 정보 문자열이 실기보다 한 글자 짧다」는 **반증**됐다. 길이 10 문자열은 단말 문자열이 아니라 `"" + System.currentTimeMillis()` 였다.

**사용자 영향** — 그 타이틀이 부팅 벽을 넘어 화면을 그린다(아래 표). `long`/`double` 을 이 슬롯으로 돌려받는 KTF 호출 전부가 상위 워드를 받는다.

## 1단계 — 출처 API 특정(반증)

`RUST_LOG=jvm=trace,rustjava_runtime=debug`(pre-fix release · 브랜치 `db3d29f2` — `origin/main` 과의 KTF/코어 차이 0 · `wie-backend/src/executor.rs` 만 다르다):

```
System::currentTimeMillis()
StringBuffer::append(0x484375b0, 3636375554)      // append(J)
StringBuffer::toString → String(count=10)
String::substring(…, 9, 10) → parseInt
String::substring(…, 10, 11) → SIOOBE begin 10, end 11, length 10
```

- 호출은 이 한 번뿐이다(`currentTimeMillis()` 계수 1). 게임은 그 문자열의 자리마다 `substring`→`parseInt` 를 돈다.
- `3636375554` = 당시 epoch ms `1790342770692` 의 **하위 32비트**(`1790342770692 mod 2^32 = 3636375556`, 2ms 차). 상위 워드 **416** 이 사라졌다.
  음수가 아니므로 `int` 캐스트가 아니라 **상위 워드가 0** 인 `long` 이다.
- 플랫폼 시계는 옳다 — `wie_validate`·`wie_featurephone`·`wie_cli` 전부 `from_epoch_millis(<13자리>)`.
- 호출 경로(Thumb, IMAGE_BASE `0x100000`): `0x1392cc bl 0x1432a8` → `0x143054`: 인터페이스 표 `[+0x30]`(= `call_native`) 호출 →
  **`0x143060 ldr r1,[r0,#4]; ldr r0,[r0]`**. 단어 반환 짝(`0x143040`)은 `ldr r0,[r0]` 만 읽는다.
  ⇒ 이 슬롯의 반환 규약은 «결과 칸 포인터»이고 `J` 는 그 칸의 두 워드다. `call_native` 는 `[ptr+4] = 0` 을 쓰고 있었다.

⒝ 규격: `System.currentTimeMillis()` 는 CLDC 규정상 1970-01-01 UTC 기준 ms 다 — **값을 새로 정하지 않았다.** 플랫폼이 이미 주는 값을 자르지 않게 했을 뿐이다.
단말 정보 API(번호·MIN 류)는 이 벽과 **무관**하다. OEMDevice 스텁(#273)과의 인과: 스텁이 이 지점까지 실행을 데려왔을 뿐 원인이 아니다.

## 2단계 — 수정 · before/after

`wie_validate --timeout 120`(debug · 같은 소스에서 한 줄만 되돌린 짝) · loadavg 160~255:

| 바이너리 | 결과 | stop | ticks | paints | 벽 |
|---|---|---|---|---|---|
| 수정 ×3 | **PASS** booted + rendered | deadline | 86 · 4626 · 7525 | **16 · 26 · 39** | — (1회차만 잡힌 `NullPointerException: image is null` 1건) |
| 수정 되돌림(`[ptr+4]=0`) ×3 | FAIL | error | 15 · 16 · 17 | 0 · 0 · 0 | `StringIndexOutOfBoundsException: begin 10, end 11, length 10` |
| pre-fix release(참고) | FAIL | error | 12 | 0 | 동일 |

**양방향 변이(단위)** — `test_native_method_entry_points` 에 `call_native` 경로를 더했다(기존 `TestClock 0x12345678_9abcdef0`).
되돌리면:

```
assertion `left == right` failed: currentTimeMillis, call_native
  left: [2596069104, 0]
 right: [2596069104, 305419896]
```

복원하면 통과.

★1회차의 `image is null` NPE 는 잡힌 예외(실행 계속 · 렌더됨)이고 2·3회차엔 없다 — 이 회차는 재지 않았다(부하 의존 가능).

## 게이트

- `cargo fmt --check` rc=0 · `clippy --all -D warnings` rc=0 · wasm clippy rc=0 · `+beta clippy` rc=0
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 — 46 스위트 · 447 passed · 0 failed
- `clippy --workspace --all-targets`(티켓 축): 경고 13 — 전부 이 diff 밖(`wie-backend/src/canvas.rs` 11 · `hardening.rs` 1 · `dod_ci_parity.rs` 1) ⇒ 증가 0
- runner block: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` **FAIL**(paints 7·9) ·
  `text_j2me` FAIL — loadavg 163~187. AGENTS 4단계대로 **수정 되돌림 바이너리와 교대 2회**: keydraw 는 양쪽 다 FAIL(paints 6~14 · 유휴 48~55 대비 기아),
  `text_j2me` 는 양쪽이 1회씩 PASS/FAIL 로 뒤집혔다 ⇒ 이 diff 가 아니라 부하다. LGT(이 diff 무관 경로)도 같은 모양이다.

## 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab`: ★**BOUNDED 4회 / 4쌍** · PREFIX-EMBEDDED 0 · ★**SUFFIX-ATTACHED 0**.
들어온 이름은 티켓이 부르는 타이틀 하나(커밋 헤드라인·이 문서·worklog)다 — 이름 없이는 어느 벽인지가 서지 않는다. 게임 바이트 유입 0 · 경로 유입 0.

<!-- corpus-name-inflow v1 subjects=5 tree=fb5be02458561c64 B=4/4 P=0/0 S=0/0 -->
