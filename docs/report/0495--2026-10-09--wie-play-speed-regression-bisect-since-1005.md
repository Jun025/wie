## [2026-10-09] 플레이 속도 «갑자기 느려짐» — 엔진 회귀 없음 (10-05 `bd770b73` ↔ 현행 `7ca47380`) (wie-play-speed-regression-bisect-since-1005)

### 무엇을 · 왜
- 운영자 신고(2026-10-09): featurephone 게임 플레이가 갑자기 느려졌다. 어느 게임인지는 특정되지 않았다.
- 그래서 플랫폼마다 대표 표본 2종을 골라 쟀다. 기준 핀 `bd770b73`(10-05)과 현행 `origin/main` `7ca47380` 을 비교했다. 1순위 용의자 #521 은 앞뒤(`82959a5b` · `68db98e0`)도 쟀다.
- **판정: 엔진 회귀는 없다.** 세 자 모두에서 현행이 기준보다 느린 표본이 없다. 브라우저(wasm), 네이티브 고속 코어, 네이티브 저속 코어가 그 세 자다. J2ME 는 오히려 빨라졌다(#521).
  ⇒ 신고된 느려짐은 엔진 밖에서 찾아야 한다. 앱 쪽은 `otterpebble-featurephone-play-speed-app-side-regression-check` 가 진다.
- 코드는 바꾸지 않았다. 아래 §#521 비용을 보라.

### 재본 범위
- 엔진 착지 10-05 → 10-09 의 `wie*/src` 변경 커밋 41개(first-parent).
- 다음은 이 구간에 변경 0 이다(`git log bd770b73..origin/main` 실측): wasm 빌드(`scripts/build-wasm.sh`), 발행(`publish-artifact.yml`), 브라우저 호스트(`wie_featurephone/`), `Cargo.toml` 프로필, `Cargo.lock`. 스케줄러 `wie-backend/src/{executor,pacing}.rs` 도 그대로다(`TICK_BUDGET_MS=14`).
- 셸 핀(`otterpebble` `apps/featurephone/public/engine/manifest.json`)은 엔진 착지마다 거의 하나씩 따라 움직였다(10-05~10-09 사이 41회). 그래서 핀 이분은 엔진 커밋 이분과 같다.
- 표본(게임 이름 대신 sha256 앞 12자):

| 플랫폼 | 표본 |
|---|---|
| J2ME | stickfight · mobapp-game (재배포 허락 코퍼스, `docs/oss-corpus/`) |
| SKT | `976141a9525b` · `d1234e0ddb75` |
| KTF | `1065985081fc` · `db8ef04a6504` |
| LGT | `870cd071b4dc` · `0606d43702e7` |

- **아무도 재지 않은 것**: 실제 저사양 폰. 저속 코어(아래 ⒞)가 그것을 가장 가깝게 흉내 낸 것이다.

### 측정 ⒜ 브라우저 — 셸과 같은 경로(wasm · 헤드리스 Chromium)
- 명령: `node scripts/audio-probe.mjs --wasm <bd770b73 빌드> --wasm <7ca47380 빌드> --secs 30 --jobs 2 <게임>`. 두 빌드는 `bash scripts/build-wasm.sh` 로 만들었다.
- 두 빌드는 동시에 돌렸고 2회 반복했다. load1 은 13~21 이었다.
- `tick` 은 `emu.tick()` 1회의 벽시계 시간이다. 페이지는 두 tick 사이에서만 그리고 입력을 받는다.

| 표본 | 빌드 | paints (1회 / 2회) | tick max ms | tick p99 ms | > 50 ms |
|---|---|---|---|---|---|
| stickfight | bd770b73 | 103 / 103 | 608 / 606 | 602 / 598 | 57 / 57 |
| stickfight | 7ca47380 | **1305 / 1255** | 140 / 128 | **28 / 29** | 2 / 1 |
| SKT `976141a9525b` | bd770b73 | 608 / 608 | 203 / 200 | 16 / 15 | 2 / 3 |
| SKT `976141a9525b` | 7ca47380 | 610 / 607 | 207 / 208 | 16 / 15 | 2 / 2 |
| KTF `db8ef04a6504` | bd770b73 | 205 / 202 | 143 / 140 | 136 / 136 | 17 / 17 |
| KTF `db8ef04a6504` | 7ca47380 | 186 / 200 | 139 / 140 | 133 / 136 | 16 / 18 |
| LGT `870cd071b4dc` | bd770b73 | 225 / 221 | 150 / 143 | 27 / 20 | 2 / 2 |
| LGT `870cd071b4dc` | 7ca47380 | 223 / 224 | 139 / 145 | 23 / 22 | 2 / 2 |

- KTF 1회차는 205 → 186 이지만 2회차는 202 → 200 이다. tick 분포는 두 회차 모두 같다. ⇒ 편차로 읽는다.
- KTF `db8ef04a6504` 과 LGT 표본은 두 빌드 모두 30 s 안에 «guest exited» 로 끝났다. 끝난 시점은 두 빌드가 같다.
- KTF 의 tick p99 136 ms 는 기준 핀에서도 같다. 그러니 이번 회귀가 아니다.

### 측정 ⒝ 네이티브 · 고속 코어
- 명령: `wie_validate --inject --keep-timeout --timeout 20 --pacing 5 <게임>`(release). 두 빌드는 동시에 돌렸고 2회 반복했다. load1 은 14~41 이었다.
- `wie_validate` 루프는 잠들지 않는다. 그래서 `ticks` 는 처리량이 아니다(AGENTS.md). 축은 `paints` 다.

| 표본 | bd770b73 paints | 7ca47380 paints |
|---|---|---|
| stickfight | 123 / 130 | **771 / 699** |
| mobapp-game | 0 / 0 (FAIL — 0485 §4 의 양보 없는 부팅) | **720 / 759** |
| SKT `976141a9525b` | 189 / 186 | 186 / 194 |
| SKT `d1234e0ddb75` | 395 / 393 | 395 / 392 |
| KTF `1065985081fc` | 37 / 36 | 40 / 39 |
| KTF `db8ef04a6504` | 182 / 185 | 188 / 187 |
| LGT `870cd071b4dc` | 238 / 224 | 234 / 230 |
| LGT `0606d43702e7` | 183 / 178 | 197 / 189 |

### 측정 ⒞ 네이티브 · 저속 코어(저사양 기기 흉내)
- 명령은 ⒝ 와 같고 앞에 `taskpolicy -b` 를 붙였다. 프로세스가 M1 Max 의 효율 코어 2개에만 붙는다. 이 자에서는 게스트가 CPU 에 묶인다. 그래서 «틱당 비용»이 paints 에 드러난다. 고속 코어에서는 여유가 커서 드러나지 않는다.

| 표본 | bd770b73 paints | 7ca47380 paints | 비고 |
|---|---|---|---|
| stickfight | 97 / 95 | **160 / 154** | `sleep_late_p95` 7547 / 4003 ms → 376 / 351 ms |
| mobapp-game | 0 / 0 (FAIL) | **386 / 363** | |
| SKT `976141a9525b` | 148 / 132 | 143 / 122 | 아래 SKT 이분 참조 |
| SKT `d1234e0ddb75` | 385 / 385 | 385 / 384 | |
| KTF `1065985081fc` | 68 / 70 | 68 / 70 | |
| KTF `db8ef04a6504` | 192 / 203 | 203 / 211 | |
| LGT `870cd071b4dc` | 159 / 175 | 159 / 175 | |
| LGT `0606d43702e7` | 223 | 223 | 2회차는 동시 기록이 한 줄로 섞여 버렸다 |

- **SKT 이분**(저속 코어 · #521 앞뒤 포함 · 세 빌드 동시 · 2회):

| 묶음 | 82959a5b (#521 전) | 68db98e0 (#521) | 7ca47380 | bd770b73 |
|---|---|---|---|---|
| `976141a9525b` 1 | 126 | 401 | – | – |
| `976141a9525b` 2 | 104 | 103 | – | – |
| `976141a9525b` 3 | – | – | 130 | 136 |
| `976141a9525b` 4 | – | – | 131 | 133 |
| `d1234e0ddb75` 1 | 372 | 370 | – | – |
| `d1234e0ddb75` 2 | 330 | 328 | – | – |
| `d1234e0ddb75` 3 | – | – | 352 | 354 |
| `d1234e0ddb75` 4 | – | – | 318 | 319 |

- 같은 묶음 안에서는 빌드끼리 같다. 크게 갈리는 것은 묶음끼리다(103 ↔ 401). ⇒ `976141a9525b` 의 흔들림은 실행마다 갈리는 게임 경로 탓이다. 핀 탓이 아니다.
- **새로 보인 것 하나(속도와 무관)**: SKT 표본의 `pacing.sleeps` 가 3 → 230~430, 7 → 80~160 으로 늘었다. 이분상 `82959a5b` 에서 이미 그렇다. 시기로 보면 `33720b84`(SKT `AudioClip.play` 가 소리 스레드에서 블록)일 가능성이 크다. 이분으로 확정하지는 않았다. paints 는 같다.

### #521 비용 — 호출당 잰 값
- 총괄 사전 조사의 1순위 용의자였다. 게스트 메서드를 찾을 때마다 `Box<Preemptible>` 이 하나 생기고, 호출마다 `async_trait` 미래가 한 겹 더 박싱된다.
- 마이크로벤치: 같은 트리에서 `Spin.run(300000)` 을 쟀다. `spin_class()` 는 #521 시험의 것이다. 세 판을 6회 × 5번 번갈아 돌렸다(release · load1 17~25). 세 판은 다음과 같다.
  - off: `define_class` 가 `guest_calls: None` = #521 전 동작.
  - on: 현행.
  - unboxed: `Preemptible::run` 을 손으로 써서 999/1000 호출이 안쪽 미래를 그대로 돌려주게 한 판.

| 판 | 중앙값 | 최소 | 최대 |
|---|---|---|---|
| off | 246.1 ms | 239.6 | 284.1 |
| on (현행) | 262.6 ms (**+6.7%**) | 252.8 | 325.4 |
| unboxed | 259.5 ms (+5.4%) | 246.3 | 269.7 |

- 해석:
  - 빈 메서드 호출만 하는 최악의 경우에 호출 비용은 약 +7% 다(호출 1회 ≈ 0.82 → 0.88 µs). 실제 게임에서는 호출 외 작업이 섞여 그보다 작다. 위 세 자 어디에서도 paints 하락으로 보이지 않았다.
  - 추가 박싱을 없애도(unboxed) 1.3%p 밖에 돌아오지 않는다. 남는 비용은 `Box<Preemptible>` 할당과 `Arc` 복제인데, `ClassDefinition::method` 가 `Box<dyn Method>` 를 돌려주는 한 피할 수 없다.
  - ⇒ 이득이 시험·복잡도에 못 미친다. 그래서 고치지 않았다.
- **다시 열 조건**: 실기나 저속 코어에서 J2ME/SKT 의 paints 가 #521 전보다 낮게 재현될 때. 그 경우 다음 수는 「`define_class` 시점에 감싼 메서드 캐시」다. 지금 본 수치로는 정당화되지 않는다.

### 사용자 영향
- 없음. 엔진 동작 변경 0 이다. 문서 1개만 더했다.

### 측정 명령 원문
```
# 빌드 (rev 마다 · 공유 target)
CARGO_TARGET_DIR=<bins>/target cargo build --release -p wie_cli --bin wie_validate
bash scripts/build-wasm.sh                     # web/src/wasm → rev 별로 복사
# ⒝ / ⒞
[taskpolicy -b] wv_<rev> --inject --keep-timeout --timeout 20 --pacing 5 <game>
# ⒜
node scripts/audio-probe.mjs --wasm <dir bd770b73> --wasm <dir 7ca47380> --secs 30 --jobs 2 <game>
# 마이크로벤치 (임시 #[ignore] 시험 · 커밋 안 함)
cargo test --release -p wie-jvm-support --lib spin_bench -- --ignored --nocapture
```
