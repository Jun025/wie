## [2026-10-10] executor `halted` 를 잠금 밖 `AtomicBool` 로 — 잠금은 빠졌지만 네이티브 CPU 는 #476 전으로 돌아오지 않았다(코드 배치) · 브라우저 변화 없음 (wie-executor-halted-flag-atomicbool-adopt-p0)

### 무엇을 · 왜
- 0500 §근거 ⑵: #476 이 `Executor::tick_for` 반복마다, `step` 태스크마다 `inner.lock().halted` 를 한 번 더 잡는다. 그 두 줄만 뺀 판이 #476 전과 같은 CPU 였다.
  운영자가 그 제안(`2026-10-10-play-speed-tick-cost-commit-bisect#p0`)을 채택했다.
- `halted` 를 `ExecutorInner` 에서 빼서 `Executor` 의 `Arc<AtomicBool>` 로 옮겼다. `halt()` 는 `store(true, Relaxed)`, 두 읽기는 `load(Relaxed)` 다.
  값은 한 번 `true` 가 되면 다시 `false` 가 되지 않는다. 그래서 잠금으로 지킬 불변식이 없다. 변경은 `wie-backend/src/executor.rs` 한 파일이다.
- 동작은 `test_halt_stops_every_task_including_later_ones_in_the_same_step` 가 잠근다. 통과한다.

### `Relaxed` 로 반복이 1회 더 도는가
- **현재 호출 경로에서는 1회도 더 돌지 않는다.** `halt()` 를 부르는 곳은 `System::exit` 하나다. `exit` 는 폴링 중인 게스트 태스크 안에서 불리거나, `tick` 밖에서 호스트가 부른다.
  두 경우 모두 `tick_for`/`step` 과 **같은 스레드**다(`tick_for` 는 `&mut self`).
  같은 스레드에서는 `Relaxed` 라도 앞선 store 를 다음 load 가 본다(원자 변수 하나의 수정 순서는 프로그램 순서를 따른다).
  그래서 `halt()` 다음 `step` 의 태스크 검사에서 바로 멈춘다. 잠금 판과 같다.
- 다른 스레드가 `halt()` 를 부르면 다음 검사 1회를 놓칠 수 있다. 그런 호출자는 지금 없다.
  잠금 판도 같은 성질이다. 검사 직후에 들어온 `halt` 는 그 반복이 끝날 때까지 보이지 않는다.
- step 안의 순서도 그대로다. `halt` 를 부른 태스크는 그 폴링을 끝까지 돈다(`System::exit_from_guest` 몫). 같은 step 에서 먼저 폴링된 태스크는 이미 돌았다.
  그 뒤 태스크는 폴링되지 않는다.

### 전후 tick 당 CPU — 네이티브(`wie_validate` 릴리스 · 50M tick · CPU ms(user+sys) · n=5)
- 3판을 섞어 한 번에 3개씩 돌렸다. 회차는 1~5 이고, 판×게임마다 회차당 1번이다. 전 실행이 `stop=max-ticks`(50M)에 닿았다. load1 15.4~40.1 이었고, `host-load-guard --status --recovered` 는 rc=0 이었다.

| 판 | `System::tick_for` 바이트 | `1065985081fc` 5회 · 중앙값 · 대비 | `0606d43702e7` 5회 · 중앙값 · 대비 |
|---|---|---|---|
| `main` `a3da330d`(잠금) | 4096 | 7950 7970 8060 8080 8100 · 8060 · 0 | 10160 10170 10210 10240 10290 · 10210 · 0 |
| **이 PR**(`AtomicBool`) | 3364 | 8340 8340 8390 8420 8460 · 8390 · **+4.1%** | 10300 10430 10450 10590 10600 · 10450 · **+2.4%** |
| 대조: `main` 에서 검사 두 줄 삭제(halt 무동작 · 시험 깨짐) | 3864 | 7880 7900 7940 7980 8020 · 7940 · −1.5% | 8660 9940 9990 10150 10330 · 9990 · −2.2% |

- 같은 두 판을 1차로 따로 섞어 돌렸을 때도 이 PR 이 10/10 쌍에서 느렸다(+2.4~3.7%).
- **판정: #476 전 수준으로 돌아오지 않았다. 네이티브에서는 오히려 2.4~4.1% 늘었다.**
  잠금을 빼는 것 자체의 몫은 대조 판으로 보면 −1.5~−2.2% 다(0500 의 +3~4% 보다 작다).
- **원인은 코드 배치다.** 이 PR 판에서 LLVM 이 `System::tick_for` 안의 `BTreeMap` `Values::next` 를 인라인하지 않았다.
  바깥 호출이 1곳 → 4곳이 됐다(`otool -tV`). 0500 이 #517 의 +4~5% 에서 본 것(1 → 3곳)과 같은 모양이다.
  원자 load 하나가 spin 잠금(CAS + 해제 store)보다 비쌀 수는 없다. 그래서 이 증가는 플래그 접근이 아니라 주변 코드의 인라인 결정 몫으로 본다.
  0500 의 권고대로 인라인 운을 좇아 고치지는 않았다. 다음 무관한 변경에서 다시 뒤집힌다.

### 전후 — 브라우저(헤드리스 Chromium · `audio-probe.mjs` · `3cc7a9b4cb15` · 30 s · 두 판 동시 · 4회)
- 0500 에서 #476 이 사용자에게 닿은 축이다(paints −1.5~−5.3%, 3/3). wasm 은 `a28f27cff687`(main) · `8bd4fc7595ca`(이 PR) 다.

| 회 | load1 | main → 이 PR paints | tick p99 ms |
|---|---|---|---|
| 1 | 7.4 | 391 → 397 (+1.5%) | 18 / 18 |
| 2 | 9.3 | 384 → 376 (−2.1%) | 18 / 19 |
| 3 | 11.2 | 397 → 399 (+0.5%) | 19 / 19 |
| 4 | 10.4 | 383 → 395 (+3.1%) | 19 / 19 |

- 중앙값은 387.5 → 396(+2.2%)이고 4회 중 3회가 올랐다. 0500 의 #476 하락폭(−1.5~−5.3%)만큼 되돌아온 방향이다.
  다만 −2.1% 인 회가 있다. 4회로는 잡음과 가를 수 없다. **판정: 브라우저에서 나빠지지 않았다. 회복은 «방향만» 보인다.**

### 사용자 영향
- 동작 변화 없음(halt 의미 동일 · 시험 통과). 브라우저 체감 속도는 4회 측정 범위에서 같거나 조금 낫다. 네이티브 데스크톱 호스트는 tick 당 CPU 가 2~4% 늘었다(코드 배치).

### 미측정
- 브라우저 n 을 늘린 측정(이번 4회). 발행 아티팩트·모바일 Safari·실기.
- `--inject` 경로(0500 과 같은 이유로 키 없는 부팅+대기 경로만 쟀다).

### 측정 명령 원문
```
# 판 — 판마다 자기 target
git archive a3da330d | tar -x -C <s>/before                       # main
git archive HEAD     | tar -x -C <s>/after-tree                   # 이 PR
# 대조: main 의 executor.rs 에서 `if self.inner.lock().halted {…}` 두 블록만 삭제
CARGO_TARGET_DIR=<자기 target> ~/orchestrator-live/bin/build-slot run -- cargo build -q --release -p wie_cli --bin wie_validate
# 네이티브 — 임대 1건(build-slot run --long) · 판×게임×회차 30건 · xargs -P 3
/usr/bin/time -p -o <f> wv_<판> --timeout 90 --pacing 5 <game>     # stop=max-ticks 만 · (user+sys)×50M/ticks
# 기계어 — nm -n 다음 심볼과의 차 · otool -tV -p <System::tick_for> 의 bl 대상
# 브라우저 — 각 판에서 bash scripts/build-wasm.sh 후
node scripts/audio-probe.mjs --wasm <s>/before/web/src/wasm --wasm <s>/after-tree/web/src/wasm --secs 30 --jobs 2 --json <game>
```
- 네이티브 sha256 앞 12자: main `98e4083b9ecf` · 이 PR `ead6616bc3e6` · 대조 `64505a5112c7`.
