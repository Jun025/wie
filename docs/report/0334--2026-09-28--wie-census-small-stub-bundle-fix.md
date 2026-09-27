## [2026-09-28] #361 반려 승계 — RecordEnumeration 을 MIDP 2.0 현재 위치 모델로 · strncat 은 n 바이트까지만 (wie-census-small-stub-bundle-fix)

**무엇을**: 게이트② 반려(F1·F2)와 권장(F3)을 같은 PR 에 얹었다. `origin/main`(`283e2332` · #350 `bb4d8931` 포함)을 병합했다(충돌 0).
**왜**: 0329 가 넣은 `net/wie/RecordEnumerationImpl` 은 ListIterator 식 «사이 커서»였다. JSR118 `RecordEnumeration` 은 생성·`reset()` 직후 첫 `previousRecord()` 가 **마지막 레코드**를 주고, 어느 지점에서든 previous 는 «바로 앞» 레코드다. 시험이 그 위반(`reset → previousRecordId` = 예외, 끝에서 previous = 같은 id)을 정답으로 잠그고 있었다.
**사용자 영향**: `while (e.hasPreviousElement()) e.previousRecord()` 로 역순 순회하는 게임(점수표·저장 목록)이 0건 대신 전 레코드를 본다.

### 바꾼 것
- `record_enumeration.rs`: `index` = 마지막으로 돌려준 원소, 생성·`reset` 직후 −1. next: −1→0 또는 +1 · previous: −1→len−1 또는 −1 · `hasNextElement` = `index != len−1` · `hasPreviousElement` = `len>0 && index != 0`.
- `record_store.rs` 시험: 생성 직후 `hasPreviousElement`=true(F2) · 끝(id 3)에서 previous = **2** · `reset` 뒤 previous 순회 = **[3,2,1]** · 그다음 previous = 예외.
- `stdlib.rs` `strncat`(F3): `src` 를 NUL 까지가 아니라 **최대 n 바이트**만 읽는다. 시험: 64 KiB 페이지 끝 2바이트 `"gh"`(다음 페이지 미매핑), n=2 → `"abcdgh"`.
  ※첫 판은 0x1000 크기로 매핑해 경계가 페이지 안이었다 — `EmulatedMemory` 페이지가 **64 KiB** 라 구 구현도 green 이었다. 끝을 `0x6000_fffe` 로 옮겨 구 구현이 `InvalidMemoryAccess(0x60010000)` 로 red 가 되는 것을 확인했다.

### 되돌리면 red
| 변이 | 결과 |
|---|---|
| `hasPreviousElement` `index != 0` → `index > 0` (검수 M5) | red |
| previous 시작 `count − 1` → `−1` | red |
| previous 걸음 `index − 1` → `index` (구 «사이 커서» 모양) | red |
| init `index = −1` → `0` | red |
| `strncat` 을 0329 구현으로 되돌림 | red(`InvalidMemoryAccess`) |

### I1 — #350 위에서 재측(검수자 미측정 2건)
- 전 = `origin/main` `283e2332`(#350 포함) · 후 = 이 브랜치 head. release · `wie_validate --inject --keep-timeout --timeout 30 --pacing 8` · 4병렬 · 벽시계 200초 상한 · load1 **~200**.

| sha12 | 전 | 후 |
|---|---|---|
| `66959afab216` | FAIL boot · `NoSuchMethodError RecordStore.enumerateRecords(…)` | **PASS** · paints 521 · 27/27 |
| `f6fe2adc8cce` | 200초 안에 끝나지 않음(SIGALRM) | **PASS** · paints 124 · 27/27 |

⇒ 0329 의 두 PASS 주장은 #350 착지 뒤 main 에서도 선다.

### 선택 항목(F4~F6)
손대지 않았다 — 검수 판정대로 기록만(F4 `free` 언더플로는 게스트 값으로 불가능에 가깝다 · F5·F6 은 한계가 주석에 있다).

### 게이트
- `cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all`(passed 546 · failed 0) rc=0.
- `npm run build:wasm` · `check-engine-contract` · `npm run audit` rc=0.
- 러너 줄(엔진 변경): `draw_j2me` · `helloworld_ktf/lgt` · `keydraw_ktf/lgt --inject --expect-last-frame` · `text_j2me --timeout 5` 전부 PASS · rc=0.
- 게임 파일명 유입: 이 회차가 **더한 줄**에는 0건이다. 도구 표기 BOUNDED 20회/13쌍은 0329 와 같은 수 — 고친 파일에 **이미 있던** 주석이다(SUFFIX-ATTACHED 0).

<!-- corpus-name-inflow v1 subjects=29 tree=b03fe5e830e2af14 B=20/13 P=0/0 S=0/0 -->
