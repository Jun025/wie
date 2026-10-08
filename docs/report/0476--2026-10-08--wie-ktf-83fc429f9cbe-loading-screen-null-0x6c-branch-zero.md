## [2026-10-08] KTF 재배치 이미지 — 리졸브 헬퍼에서 들어온 호스트 진입이 바깥 Java 프레임을 덮었다 (wie-ktf-83fc429f9cbe-loading-screen-null-0x6c-branch-zero)

**무엇을**: `relocated::enter` 가 새 Java 프레임 자리를 고를 때, 리졸브 헬퍼가 네이티브 스택 맨 위(`top - 8`)에 넣어 둔 호출자 Java `sp` 도 본다(`wie-ktf/src/runtime/relocated.rs` `pushed_java_sp`).
**왜**: `83fc429f9cbe` 의 게임 스레드 `c.run` 이 불러오기 화면에서 죽었다. 원인은 `null+0x6c` 가 아니다. 호스트가 `c.run` 자신의 프레임을 덮어써서 `this` 가 엉뚱한 값이 됐다(§1).
**사용자 영향**: 그 타이틀이 불러오기 화면을 지나 이야기 장면과 지도 화면까지 간다(§2). 재배치 이미지만 이 경로를 탄다(3종). 나머지 KTF 는 `enter` 가 바로 `None` 을 돌려준다.

### 1. 결함 지점 — 실측으로 좁힌 순서

0468 §4 의 「null 포인터 `+0x6c` 를 읽고 주소 0 으로 분기」는 같은 자리를 다르게 읽은 것이다. 오늘 release 바이너리 `--inject`(기본 27키)로 재면 결과가 결정적이고 3/3 같다. `wie-core-arm` 의 메모리 오류 지점에 임시 로그를 걸어 쟀다(커밋하지 않았다).

| 칸 | 값 |
|---|---|
| 오류 | `Invalid memory access; address: 0x39001fc` · pc `0x100184` · lr `0x1489f9` |
| `r0` | `0x6c` = vtable 칸 `0x1b` × 4 (0468 이 「`+0x6c`」로 읽은 값) |
| `r1`(`this`) | `0x15db40` |
| `[0x15db40]` | `0x4b858700` = `MNInterface` 표. 그 다음 칸부터는 호스트 트램펄린 바이트(`0xbc0c 0xb408 …`)다 |

- `0x100184` 는 invoke-virtual 헬퍼다(`ktf-image-sweep.py window`): `r2 = [[this]] >> 5 + [fp+0x38]` → `[r2+0xc]` = vtable. `0x15db40` 은 Java 객체가 아니다. 이미지가 `init` 에서 받은 `MNInterface*` 를 두는 bss 마지막 칸이고(GOT `sl+0x618` 이 그곳을 가리킨다), 그래서 «클래스 단어»가 호스트 함수 주소 `0x71003e01` 이 되어 `0x39001fc` 를 읽는다.
- 호출 자리 `0x1489f4`(`bl 0x10013e`)의 `r1` 은 `[sp+0xb4]` 다. 함수 `0x1485ec` 는 `c.run` 이고, 프롤로그 `0x1485fa: str r1, [sp, #0xb4]` 가 `this` 를 넣는 칸이다.
- 그 칸(`0x400f7fe4`)에 쓰기 감시를 걸었다. `c.run` 프롤로그가 `this = 0x484347e8` 를 쓴 뒤, **호스트**가 같은 칸에 두 번 썼다.
  ```
  enter sp=0x400fbfe8 saved_sp=0 prev_native=0x400fc004 lr=0x15259f   ← 리졸브 헬퍼에서 MN_CLASS_LOAD(java/util/Random)
  host write 0x400f7fe4                                                ← run_function 인자
  w32 0x400f7fe4 <- 0x15db40  pc=0x71002c80                            ← KtfClassLoader.loadClass 의 호스트 함수
  ```
- 경로: `c.run` → invoke 헬퍼(`0x10013e`)가 아직 링크되지 않은 메서드 칸을 만난다 → 네이티브 스택(`[fp+0x34]`)으로 옮겨 클래스 리졸브(`0x152570`)를 부른다 → `MN_CLASS_LOAD` → JVM 이 클래스 로더 사슬(`String.<init>` · `KtfClassLoader.loadClass` …)을 KTF 메서드 실행기로 돌린다 → 그때마다 `enter` 를 거친다.
- `enter` 는 `+0x24` 에 파킹된 Java `sp` 가 없으면(`saved_sp = 0`) 새 프레임을 `sp - 0x4000` 에 둔다. 여기서 `sp` 는 네이티브 스택 맨 위 근처(`0x400fbfe8`)다. 그러면 `0x400f7fe8` 이 되고, 그 자리가 바로 이 진입의 가장 바깥 Java 프레임 `c.run`(`0x400f7f28..0x400f8004`) 위다.

**헬퍼가 Java `sp` 를 두는 곳 — 재배치 이미지 3종 전수.** `[fp,#0x34]` 를 읽어 `sp` 로 옮기는 자리는 세 이미지 모두 32곳이고 형태도 같다.

| 형태 | 곳 | Java `sp` 위치 | 하는 일 |
|---|---|---|---|
| `str r2,[fp,#0x24]` 뒤 `push {lr}` | 4 | `+0x24`(파킹) | `new` 계열 — Java 를 돌릴 수 있다 |
| `mov r2, sp` 뒤 `push {r2, lr}` / `push {r1, r2, lr}` · `mov ip, sp` 뒤 `push {r3, lr}` | 8 + 15 + 1 = 24 | **`[native top - 8]`**(복귀 주소 바로 아래) | 클래스·필드·메서드 리졸브(`0x152570` · `0x1525d0` · `0x15265c`) |
| 바로 `bl` | 4 | 남기지 않는다 | `MN_THROW` · `MN_THROW_INSTANCE` — 돌아오지 않는다 |

`enter` 는 첫째 줄만 알고 있었다. 이 회차는 둘째 줄을 더했다. `sp` 가 이번 진입의 네이티브 스택 위(`(top - 0x4000, top]`)에 있고 파킹이 없으면, `[top - 8]` 을 호출자의 Java `sp` 로 읽는다. 그 값은 이번 진입의 Java 영역(`≤ top - 0x4000` · 1 MB 안)에 있을 때만 쓴다. 그 범위 밖이면 종전대로 둔다.

**「상속하지 않은 메서드 = vtable 칸 없음 → 주소 0」 가설(GFormComponent 주석)은 해당하지 않는다.** vtable 을 읽은 대상이 객체가 아니었고, 덮어쓰기를 막자 같은 자리의 같은 칸 `0x6c` 호출이 정상으로 간다.

### 2. 부팅 단계 — base `60e9062c` ↔ head

`wie_validate --inject`(release · 기본 27키 · 2회씩):

| | base | head |
|---|---|---|
| 판정 | PASS(주 스레드만 본다) | PASS |
| 게임 스레드 | `Uncaught exception … Invalid memory access` 1 | 0 |
| `frozen_tail_steps` | 25 / 25 | **0 / 0** |
| paints | 33 / 32 | 80 / 81 |
| distinct_colors | 122 / 122 | 203 / 203 |
| 멈춘 곳 | 불러오기 화면 | 없음 — 이야기 장면 → 지도 화면(주인공·대화창) |

### 3. 퇴행 — A/B 고유 20종

`wie_validate --inject`(release · 기본 27키 · `--max-ticks 100000000000`)로 각 타이틀을 base·head 동시에 돌렸다(2개씩 · long 풀 임대 1회 · load1 16~20). census `--only probe` 를 쓰지 않은 이유: 그 호스트 잠금을 다른 레인의 전수 progress 실행이 쥐고 있었다.
대상(고유 20종): 재배치 이미지 3종 전부, `com/ktf/kfc` 를 쓰는 다른 3종(`0c67145b11df` `bfa8ec352451` `f07cbc782828`), 표준 KTF `game_lab/working/ktf` sha 순 앞 16개(그중 같은 sha 2쌍 → 14종)다. kfc 3종과 표준 KTF 는 0468 §3 과 같은 표본이다.

| 묶음 | 종 | 판정·stop·content·예외 수·게임 스레드 uncaught 가 바뀐 것 |
|---|---|---|
| `83fc429f9cbe` | 1 | §2 그대로(uncaught 1 → 0 · frozen_tail 25 → 0 · paints 32 → 80) |
| 재배치 나머지 `1d5831e42a8a` · `b907b0faf483` | 2 | 0 — paints 167/167 · 194/194, 예외 0/0 · 3/3 |
| kfc 3 · 표준 KTF 14 | 17 | 0 |

- paints 는 ±3 안에서 흔들린다(`0f9e1026724d` 339/342 등). 소리 재생 수가 다른 칸이 하나 있다(`0a6f495f0b35` 18/14). 표준 KTF 는 `enter` 가 `relocated_abi` 없음으로 바로 돌아가 이 변경을 타지 않는다. 그래서 시점 차로 본다.
- `f07cbc782828` FAIL(error) · `bfa8ec352451` UNMEASURED(clean exit)는 base 와 head 가 같다.

### 4. compat

`83fc429f9cbe` 행: `not-yet` → `limited`. boot·render·input `ok`(§2 head · 27키 내내 화면이 바뀐다) · sound `ok`(head 실행에서 MIDI 재생이 실제로 났다) · longplay·speed `unknown`. `knownIssues_ko` 의 «시작하는 도중에 멈춰요» 를 지웠다. longplay 를 재지 못한 이유: census 호스트 잠금을 다른 레인의 전수 progress 실행(600초 × 전 타이틀)이 쥐고 있었다. 그래서 `playable` 로 올리지 않았다.
<!-- COMPAT_REVERT -->

업데이트 소식: `docs/player-updates/2026-10-08-ktf-loading-screen-into-game.json`.

### 5. 남은 것

- **던지기 헬퍼 4곳은 Java `sp` 를 남기지 않는다.** `MN_THROW` 는 예외 객체를 만들 때 생성자를 같은 `enter` 로 돌리고, 그 프레임도 `sp - 0x4000` 에 놓인다. 잡는 프레임이 이번 진입의 가장 바깥 프레임이면 그 지역변수가 덮일 수 있다. 이 회차에서 그렇게 실패한 타이틀은 관측하지 못했다. 이 형태로 이어지는 근거도 없어 고치지 않았다.

### 6. 검증

- 4게이트 통과: fmt · clippy stable/beta/wasm32 `-D warnings` · `RUST_MIN_STACK=4194304 cargo test --all`. beta 의 `unused dependency softbuffer/winit` 경고는 원래 있던 것이고 rc 는 0 이다.
- 시험 `a_nested_entry_goes_below_the_java_sp_a_resolve_helper_pushed`: 네이티브 스택 맨 위에 `push {r2, lr}` 모양을 놓고 `enter` 뒤 `sp` 를 본다. 수정을 되돌리면 FAILED(`left 0x4000bfe8 · right 0x40007f2c`)였고, 원상에서 통과했다.
- 러너 줄(head release): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 기본 상한에서 `UNMEASURED · max-ticks` 였다(0473 §러너 · release 가 5천만 틱을 키 일정보다 먼저 쓴다). `--max-ticks 100000000000` 로 PASS · rc=0(paints 79 · 55)이었다.
