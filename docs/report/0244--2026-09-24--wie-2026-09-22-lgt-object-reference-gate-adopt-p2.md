## [2026-09-24] 레전드오브마스터의 «헤더 없는 참조»는 해제도 미초기화도 아니었다 — `long` 필드 둘째 워드가 링크되지 않아 `Card.canvas` 를 덮었다 (wie-2026-09-22-lgt-object-reference-gate-adopt-p2)

채택 제안 `2026-09-22-lgt-object-reference-gate#p2` 의 판정 회차다.
★**판정 = 후보 (a)(해제 후 잔존)도 (b)(헤더 채우기 전 저장)도 아니다. 원인은 wie 링커다.** 그래서 2단계까지 고쳤다.

### ⒜ 전제 — 벽은 현 main 에도 있다

`origin/main` `3d7ac24c` 에서 게이트 warn 은 2회 실행에 **68 / 80건**이었다. raw 는 전부 `0x2`·`0x3` 이다(`Invalid memory access; address: 0`).
★**raw 가 힙 주소가 아니다.** `0x2` 의 «헤더»를 읽으면 저주소 0 이 나오고, 그것이 «`ptr_dispatch_table` = 0»의 정체였다.
⇒ 해제·재사용(a)이나 할당 직후(b)는 **힙 주소**여야 성립하므로, 두 후보는 이 값에서 이미 탈락한다.

### 근거 1 — 그 워드는 어느 필드인가(임시 계측 · 커밋 안 함)

`get_field` 에 참조형인데 `0 < raw < 0x10000` 인 경우를 찍었더니 **전건이 한 자리**였다:
class `f`(extends `org/kwis/msp/lcdui/Card`) · 필드 `canvas : Ljavax/microedition/lcdui/Canvas;` · **word 0**.
저장소 앞 7워드는 `[3, 0x488450d0, 0, 0, 0xf0, 0x140, 0]` 로 wie `Card` 선언(`canvas, display, x, y, w, h, transparent`)과 맞는다(`w=240 · h=320`). **word 0 만 틀렸다.**

### 근거 2 — 누가 썼나(쓰기 감시 · 임시)

`f` 인스턴스의 word 0 에 호스트·게스트 쓰기 감시를 걸었다:

```
PROBE_HOST_WRITE  value=0x488454c0            ← wie Card.<init> 이 진짜 Canvas 를 넣음(유일한 호스트 쓰기)
PROBE_GUEST_WRITE pc=0x91f34  0x488454c0 -> 0xd1c05277
PROBE_GUEST_WRITE pc=0x93834  0xd1c05277 -> 0x3
PROBE_GUEST_WRITE pc=0x91f34  0x3        -> 0xd1c0528d
PROBE_GUEST_WRITE pc=0x93834  0xd1c0528d -> 0x2
```

해제·재할당은 **0회**다. **게임 자신의 ARM 코드**가 덮었다.

### 근거 3 — 디스어셈블(`binary.mod` `.text` vaddr 0x1000)

```
0x91f24: ldrsh r2, [r5, #0x74]      ; 인스턴스 필드 워드 색인표의 두 칸
0x91f28: ldrsh ip, [r5, #0x76]
0x91f30: str r3(=r1 상위), [lr, r2, lsl #2]
0x91f34: str r0(하위),     [lr, ip, lsl #2]     ← 색인 0 ⇒ word 0
...
0x93824: ldrsh r2, [ip, #0x6a]      ; 64비트 뺄셈 결과(경과 ms)의 하위
0x93834: str r7, [lr, r2, lsl #2]               ← 색인 0 ⇒ word 0
```

`long` 하나가 색인표 **두 칸**(`+0x74/+0x76`, `+0x68/+0x6a`)을 쓴다. 값은 `currentTimeMillis` 하위 워드와 경과 ms 다.
둘째 칸이 import 표의 **이름·서술자 둘 다 null 인 «구멍»** 이다. `link_class_members` 가 그 칸을 `continue` 로 건너뛰어 u16 이 **0** 으로 남았고,
게스트는 `long` 의 하위 워드를 **word 0 = 상속된 `Card.canvas`** 에 썼다. GC 가 그것을 객체로 따라갔다.
★상위 워드가 먼저(이름 붙은 칸), 하위가 둘째(구멍) 순서다. 이는 #275 의 `long[]` 저장 순서(high first)와 같다.

### 근거 4 — 구멍은 «항상» 넓은 필드 바로 뒤다

LGT 91타이틀(broken 46 + working 45)을 전수로 봤다. 구멍 **36개 / 9타이틀**이 있고, **36/36 이 `J`·`D` 필드 바로 다음 칸**이다(예외 0).
레전드오브마스터 class `f` 의 구멍은 index 51·53·59 이고, 각각 `ew`@275·`ev`@273·`eu`@271(이 클래스의 `long` 전부) 뒤다.
종전 `read_member_name_and_descriptor` 주석은 5타이틀의 구멍을 «건너뛰면 된다»로 적었다. **그 구멍들이 전부 이것이었다.**

### 고침

`link_class_members` 에서 인스턴스 필드 구멍이 `J`/`D` 바로 뒤이면 `word_index + 1` 을 쓴다. 그 밖의 구멍은 종전대로 무접촉이다. 정적 필드 표는 구멍이 관측되지 않아 건드리지 않았다.
테스트 `hole_after_wide_field_links_to_its_second_word`: 표 `a:J · 구멍 · b:I · 구멍` 을 링크하면 `[a, a+1, b, 무접촉]` 이 나와야 한다.
변이 M1(채우기 제거) → `[0, 0xeeee, 2, 0xeeee]` 로 FAILED · M2(모든 구멍 채우기) → `[0, 1, 2, 3]` 으로 FAILED.

### before / after (같은 두 바이너리 · base `3d7ac24c` ↔ fix)

| | warn | result · stop | paints |
|---|---|---|---|
| 레전드오브마스터 base ×2 | 68 / 80 | PASS · max-ticks | 2 / 2 |
| 레전드오브마스터 fix ×2 | **0 / 0** | PASS · max-ticks | **85 / 83** |

LGT 91타이틀 스윕(`--timeout 20`)에서 판정은 PASS 55 · FAIL 35 로 **양쪽 동일**하고 PASS→FAIL 은 0이다. NO-JSON 은 크로이센 1건이며 양쪽 다 stack overflow(선재).
판정·벽이 바뀐 것은 **스파이더맨3 1건**이다. base 는 같은 병(게이트 warn 2 · raw `0xd1fe…` = ms 하위 워드)으로 흑화면에서 끝났다.
fix 는 warn 0 · Font 초기화 7→11 로 더 가서 **새 벽** `java/io/DataInputStream vtable index 27` 에 닿는다(전진).
나머지 stop(max-ticks↔deadline) 변화와 paints 요동은 부하다. 구멍 없는 타이틀 3종을 번갈아 짝지어 재니 차이가 사라졌다(테라 base 64/55 ↔ fix 74/74 · 블레이드마스터3 398/388 ↔ 406/363 · 하얀섬 164/188 ↔ 190/171).

### 게이트

fmt · clippy `--all -D warnings` · wasm · beta clippy 전부 OK. `RUST_MIN_STACK=4194304 cargo test --all` **431 passed / 0 failed**.
`cargo clippy --workspace --all-targets` 경고는 base 16 ↔ fix 16 이다. 러너 블록은 전건 PASS(keydraw ktf/lgt paints 55 · rc=0).
