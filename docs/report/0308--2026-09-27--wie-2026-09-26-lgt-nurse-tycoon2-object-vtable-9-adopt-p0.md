## [2026-09-27] 간호사타이쿤2 — java/lang/Object vtable 9 = wait() · 7 = wait(J) (wie-2026-09-26-lgt-nurse-tycoon2-object-vtable-9-adopt-p0)

**무엇을**: `wie-lgt/data/lgt_java_abi.toml` 의 `java/lang/Object` 에 `wait()V`=9 · `wait(J)V`=7 두 행을 넣었다. `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 두 행을 `// 간호사타이쿤2` 로 추가했다.
**왜**: 간호사타이쿤2 는 클래스 초기화 뒤 thread 3 에서 `Unimplemented: java/lang/Object vtable index 9` 로 멈췄다(paints 0).
**사용자 영향**: 간호사타이쿤2 가 처음으로 화면을 그린다. SD한국전쟁은 index 9 벽이 사라지고 키 입력에 반응한다.

### 호출부 (binary.mod, `objdump --triple=armv5te`)

| 주소 | 형태 | 판정 |
|---|---|---|
| 0x20ce8 | monitorenter(import 0x56) · r0 = 정적 필드 L | `synchronized (L)` |
| 0x20d98 | `ldr r12,[r3,#0x28]` = index 9 · r0 만 세팅 | 인자 없음 → `wait()` |
| 0x20e00 | `ldr r12,[r3,#0x20]` = index 7 · r1:r2 = 부호확장 long `30*(2-x)+10` · 실행 시 40:0 | long 1개 → `wait(J)` (`wait(JI)` 아님) |
| 0x21790 | 형제 함수 · 같은 풀 워드 0x015013da 의 L 에 index 5 | `synchronized (L) { L.notify(); }` |

기존 0x1f10/0x2200 핸드셰이크 앵커(다른 이미지)와 같은 모양이다. 이미지 전체에서 `[r3,#0x28]` 디스패치는 이것 1건이다.
7 은 9 를 채운 실행이 실제로 도달했다. 그래서 «도달 실행이 없으면 비워 둔다» 조건 밖이다. 8 · 2 · 6 은 계속 비워 둔다.

### 실측 (release, `wie_validate`)

| 제목 | 모드 | base (`156c5c56`) | fix |
|---|---|---|---|
| 간호사타이쿤2 | `--timeout 60` | FAIL · paints 0 · index 9 벽 | PASS · paints 722 · 벽 0 |
| 간호사타이쿤2 | `--inject` | FAIL · paints 0 | PASS · paints 91 · last frame 96색 · 27/27 키 |
| SD한국전쟁 | `--inject` | PASS · paints 3 · `o vtable index 9` 스레드 사망 | PASS · paints 29 · last frame 102색 |

hang 판정: 간호사타이쿤2 대기 스레드는 index 9 에서 약 25 ms 뒤 notify 로 깨었다. 60 초 동안 대기 루프가 약 1,080회 돌았다(탐침 6,486회 ÷ 회당 6). 탐침은 커밋하지 않았다.

### 회귀

- 코퍼스 짝 회귀: `game_lab/{working,broken}/lgt` 93건 · `--timeout 20` · 6병렬 · loadavg 96~135. 판정이나 벽이 바뀐 것은 위 2건뿐이다.
- 부작용: 배치되지 않은 메서드는 표 끝에 빈 칸으로 덧붙는다(`vtable.rs` `unplaced`). 그래서 호스트 클래스 표가 2칸 짧아진다(Object 17 → 15). 위 93건에서 새 벽은 0건이다.
- 변이: 두 행을 지우면 `abi_rows_cover…` 가 FAILED 한다.
- 4게이트: fmt · clippy stable/beta/wasm32 `-D warnings` rc0 · `cargo test --all` 495 passed / 0 failed. 러너 블록 6 fixture 전부 PASS.

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 결과는 아래 표식 줄의 B·S 값이다. BOUNDED 와 SUFFIX-ATTACHED 는 대부분 이 브랜치가 건드린 두 파일에 이미 있던 제목 주석이다. 이 회차가 새로 쓴 이름은 간호사타이쿤2 · SD한국전쟁 두 제목과 코퍼스 경로 서술뿐이다. 게임 바이트는 쓰지 않았다.

<!-- corpus-name-inflow v1 subjects=4 tree=213480240b1ff3d9 B=96/32 P=0/0 S=27/7 -->
