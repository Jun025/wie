## [2026-11-22] upstream 의 실행기 시간 의미를 우리 pacing 아래로 — sleep 마감은 «부른 때»부터 · 깨워진 태스크만 틱 안에서 다시 돈다 (wie-upstream-sync2-s2-backend-executor-sleep-deadline-cooperative-poll)

**무엇을**: upstream 이 실행기에 넣은 두 의미를 `wie-backend` 의 `executor.rs` · `system.rs` · `task.rs` 에 옮겼다.
- **sleep 마감(upstream `49078c6b6` · #1491)**: `sleep(n)` 의 깨는 시각을 «부른 때 + n» 으로 센다. 그전에는 «그 step 이 시작한 때 + n» 이라, 자기 전에 한 일만큼 잠이 짧았다.
- **협동 폴링(upstream `d63d6022d` · #1429)**: 틱 안에서 다시 도는 것은 «다시 돌려 달라고 한» 태스크뿐이다. 모니터 `wait` 처럼 걸려 있는 태스크는 누가 깨우거나 다음 틱이 올 때 돈다. 그전에는 걸린 태스크가 하나라도 있으면 틱이 예산(14 ms) 끝까지 헛돌았다.

우리 pacing API(`tick_for` · `wants_cpu` · `next_due` · `halt` · `clear` · `sleep_toward` · `TICK_BUDGET_MS`)의 서명과 기본값은 그대로다. 스폰 순서(`BTreeMap`)도 그대로다. 더한 공개 함수는 `System::poll_until` 하나다(§3).

**왜**: 티켓 `wie-upstream-sync2-s2-backend-executor-sleep-deadline-cooperative-poll`(운영자 채택 제안 `2026-11-06-upstream-sync2-dry-run-classify#p1` · 계획 = 0551 §5·§7 의 S2 행). upstream 2차 병합 예행의 ours 탐침이 맨 처음 죽는 곳이 `wie-backend` 이고, 그 오류 4건 중 2건이 이 의미 차이다(§6).

**사용자 영향**: 게임 스레드의 `sleep(n)` 이 요청한 만큼 잔다(그전에는 자기 전에 한 일만큼 — 한 조각, 길어야 수 ms — 짧았다). 마감에 맞춰 자는 게임은 주기가 그대로이고(§3), 고정 길이로 자는 게임은 그 조각만큼 느려져 제 박자가 된다. 걸려 있는 스레드만 남은 판은 틱이 예산 끝까지 헛돌지 않는다. 속도 축 측정은 §4.

### 1. 옮긴 의미 — upstream 줄 ↔ 우리 줄

| upstream | 무엇 | 우리 줄(이 PR 뒤) |
|---|---|---|
| #1491 `executor.rs` — `last_now` 필드 삭제 · `step` 이 더는 적지 않음 | sleep 이 step 시작 시각에 기대지 않는다 | `executor.rs` `ExecutorInner` 에서 `last_now` 삭제 · `step` 의 대입 삭제 |
| #1491 `executor.rs` — `pub fn sleep(&self, until: Instant)` | 실행기는 «언제까지»만 받는다 | `Executor::sleep(until)`(같은 서명) · `Executor::sleep_toward(until, pace)` |
| #1491 `task.rs` — `SleepFuture::new(until: Instant, …)` | 같은 뜻 | `SleepFuture::new` 두 줄이 upstream 과 글자까지 같다 · `SleepFuture::toward(until, pace, …)` |
| #1491 `system.rs` — `SleepFuture::new(self.platform.now() + timeout, …)` | 마감을 부른 때의 시계로 만든다 | `System::sleep` → `sleep_from(now, timeout)` · `System::sleep_toward` · `System::guest_sleep`(시계를 한 번만 읽어 마감과 «늦음» 기준을 같게 둔다) |
| #1491 `wie-backend/tests/system.rs`(새 파일) | 회귀 시험 `sleep_waits_from_the_call_time_after_task_work` | upstream/main 의 파일을 그대로 실었다(바이트 동일) |
| #1491 `wie-backend/Cargo.toml` — dev 의존 `test-utils` | 위 시험용 | 이미 있었다(변경 0) |
| #1429 `executor.rs` — `TaskWake`(깃발 waker) | no-op waker 를 깃발로 | 0524 에서 이미 `Woken` 으로 들어와 있었다(`wants_cpu` 용) |
| #1429 `executor.rs` — `tick`: `step` 뒤 깃발이 안 섰으면 `break` | 깨워진 태스크가 없으면 그 틱은 더 돌지 않는다 | `tick_for` 의 `idle` — 아래 §2 ⑴ |
| #1429 `executor.rs` — `spawn` 이 깃발을 세운다 | 새 태스크는 스폰된 틱 안에서 돈다 | `Executor::spawn` 끝의 `self.woken.wake_by_ref()` |
| #1429 `executor.rs` — `sleep` 이 깃발을 세운다 | 0 ms sleep 이 같은 틱에서 이어진다 | 깃발은 세우지 않는다 — 아래 §2 ⑵. 같은 결과를 `idle` 경로의 «깰 때가 된 태스크가 있으면 step» 이 낸다 |
| #1429 시험 2개 · `YieldOnce` 가 waker 를 부름 | `pending_tasks_only_repeat_within_a_tick_when_woken` · `zero_duration_sleeps_and_newly_spawned_tasks_resume_in_the_same_tick` | 둘 다 그대로 옮겨 통과 · `YieldOnce` 도 같게 |
| #1429 `wie-ktf/…/class_instance.rs`(필드 읽기 최적화) | 실행기와 무관 | 옮기지 않았다 — 0551 §6 의 «조용한 병합» 목록에 있는 파일이고 S3 몫이다 |

### 2. 우리 pacing 을 지키려고 upstream 과 다르게 둔 두 곳

⑴ **깨워진 태스크가 없을 때 틱을 바로 끝내지 않는다.** upstream 은 `break` 한다. 우리는 «모두 잠든 틱»과 같은 길로 보낸다: 예산 안에 드는 paced 깨움이 있으면 그때까지 기다렸다가 step, 없으면 끝낸다. 걸린 태스크 옆에서 자는 게임 스레드의 `sleep(60)` 이 호스트 프레임 격자(66.7/83.3 ms)로 밀리지 않게 하는 우리 규칙(`test_tick_waits_for_a_wake_inside_its_budget`)이 걸린 태스크가 있을 때도 서야 하기 때문이다. 새 시험 `test_tick_waits_for_a_wake_inside_its_budget_beside_a_parked_task` 가 잠근다(걸린 태스크는 그 틱에 2번만 폴링된다 — 재우는 step 과 깨우는 step).

⑵ **`sleep` 은 깃발을 세우지 않는다.** 그 깃발은 `wants_cpu`(«예산을 다 쓰고도 더 달라는 게스트» → `starved_ticks` → `achieved_speed()`)가 읽는다. 잠드는 것은 CPU 를 달라는 것이 아니다. 세우면 모두 잠든 틱이 «굶었다»로 읽힌다.

그 밖에 자리만 옮긴 것: «1 ms 이하 sleep 은 폴링이라 틱을 붙잡지 않는다»(`POLL_SLEEP_MS`)는 판정이 `Executor` 에서 `System::sleep` 으로 갔다. 실행기가 더는 `timeout` 을 받지 않아서다. 값(1)과 뜻은 그대로다.

### 3. 측정 — `test_pacing`(전/후)

`wie-j2me/tests/test_pacing.rs` 의 픽스처(`pace_j2me` · 50 ms 마감 루프 · 읽을 때마다 1 ms 가는 시계). 워밍업 2 s 뒤 10 s 창의 `Pacing::summary_json`. «전» = `origin/main` `1853f3b6f` 를 따로 꺼낸 워크트리, «후» = 이 PR.

| 칸 | 전 | 후 | |
|---|---|---|---|
| `ticks` | 720 | 720 | 같음 |
| `paints` · `redraws` | 200 · 200 | 200 · 200 | 같음 — 프레임 주기 50.0 ms |
| `redraw_p50` · `p95` · `sum` | 4 · 4 · 800 | 4 · 4 · 800 | 같음 |
| `redraw_cross_tick` | 0 | 0 | 같음 |
| `yields` · `sleeps` | 400 · 200 | 400 · 200 | 같음 |
| `gcs` · `starved_ticks` | 10 · 0 | 10 · 0 | 같음 |
| `sleep_ms_p05` · `p50` · `p95` | 39 · 39 · 40 | 37 · 37 · 38 | **−2** |
| `sleep_late_p50` · `p95` · `sum` | 0 · 0 · 0 | 2 · 2 · 300 | **+2**(상한 5) |

틱 하나하나를 적어 본 것(스크래치 시험 · 커밋하지 않음 · 창 260 ms = 16틱)도 같다. 틱 길이의 분포(18 · 16 · 16 ms 가 프레임마다 반복 · 19 ms 1회)와 틱 끝 `due` 의 앞섬(34 · 18 · 2 ms · 35/19/3 각 1회)이 전/후 한 줄도 다르지 않다. 달라진 것은 무늬 전체가 2 ms 뒤로 간 것뿐이다(첫 틱 2000..2018 → 2002..2020).

바뀐 두 칸은 옮긴 의미 그 자체다. 이 게스트는 `sleep(다음 마감 − 지금)` 을 부른다. «전»은 그 잠을 step 시작 시각에서 세어 마감보다 시계 읽기 2번만큼 **일찍** 깼다 — 그래서 «늦음»이 0 으로 읽혔고, 일찍 깬 만큼 다음 잠이 길었다(39). «후»는 마감에 깬다. 읽기 한 번이 1 ms 인 이 시계에서 그 뒤 첫 읽기까지가 2 ms 다. 주기 · 그리기 수 · repaint→paint · 틱을 넘긴 그리기는 그대로다. 그래서 «박자는 그대로, 잠의 기준점만 옮겨졌다»로 판정했다. 시험 주석의 실측값 한 줄을 고쳤다(p95 `4 and 0` → `4 and 2`). 단언의 상한은 건드리지 않았다.

★**이 PR 의 첫 판은 이 시험이 red 였다 — S3 가 같은 자리를 밟는다.** upstream 의 꼴(`System::sleep` 이 부를 때마다 `platform.now()` 를 읽는다)을 그대로 옮기면 MIDP 이벤트 스레드의 1 ms 폴링이 한 바퀴에 시계를 두 번 읽는다(대기 조건 1 + sleep 1). 읽기가 곧 시간인 이 시계에서 그 판은 `redraw_p95` 4 → 9 · `redraw_cross_tick` 0 → 76/201 · `sleep_late_p95` 0 → 7 · `starved_ticks` 0 → 102 로 두 시험 모두 red 였다(`paints` 는 201 로 주기는 섰다). 그래서 대기 루프가 제 조건으로 읽은 시각을 폴링의 시작으로 그대로 쓰게 했다(`System::poll_until` · `wie-midp` `event_queue.rs`). 폴링당 읽기 1회 — 고치기 전과 같다. 실기기 시계에서는 읽기 한 번이 수십 ns 라 이 차이가 박자에 닿지 않는다. 이것은 자(읽기 수를 세는 시계)를 지키는 고침이다. S3 에서 `system.rs` 의 `sleep` 충돌을 upstream 쪽으로 풀면 이 red 가 다시 난다.

### 4. 측정 — `perf-watch compare` · 무거운 장면 표본

이 커밋 시점에는 두 엔진(`origin/main` `1853f3b6f` · 이 PR)의 release 빌드가 빌드 슬롯 대기 중이다(호스트 load1 130~290 · 대기 5~16건). 결과는 같은 PR 의 다음 커밋이 이 절에 채운다.

### 5. 게이트

모두 이 PR 의 코드 트리에서 돌렸다(`build-slot` 경유 · 2026-11-22 13:03~13:11 KST · load1 126~208).

| 게이트 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | rc=0 |
| `cargo clippy --all -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -- -D warnings` | rc=0 |
| `cargo clippy --target wasm32-unknown-unknown -p wie-core-arm -p wie_featurephone -- -D warnings` | rc=0 |
| `cargo +beta clippy --all -- -D warnings` | rc=0 · `1.101.0-beta.2`(`rustup check`: up to date) |
| `RUST_MIN_STACK=4194304 cargo test --all --no-fail-fast` | rc=0 · 시험 바이너리 55 · **869 passed · 0 failed** |
| 그중 `wie-backend` | 139 passed + `tests/system.rs` 1 passed |

러너 블록(문서대로 debug · load1 173):

| 픽스처 | 결과 |
|---|---|
| `draw_j2me.jar` | PASS · rc=0 · paints 1 |
| `helloworld_ktf.zip` · `helloworld_lgt.zip` | PASS · rc=0(둘 다 `clean exit`) |
| `keydraw_ktf.zip` `--inject --expect-last-frame` | PASS · rc=0 · `content true` · paints 79 · 27/27 |
| `keydraw_lgt.zip` `--inject --expect-last-frame` | PASS · rc=0 · `content true` · paints 55 · 27/27 |
| `text_j2me.jar` `--timeout 5` | PASS · rc=0 · paints 1 |

`test_pacing.rs` 의 주석 한 줄은 린트 묶음이 시작한 뒤(13:05)에 고쳤다. 그 뒤 `cargo fmt --all -- --check` 는 다시 돌려 rc=0 이다. clippy 는 주석만 다른 트리에서 돈 것이다.

### 6. upstream 병합 예행 탐침 — 전/후

0551 §5 의 P1(ours) 탐침을 오늘 트리로 다시 만들었다: `git merge --no-commit --no-ff upstream/main`(`24b670780` · behind 247) → UU·AA 는 `--ours` · UD 는 우리 판 · 매니페스트 4개(`Cargo.toml` · `wie-backend` · `wie-j2me` · `wie-skt`)는 양쪽 합집합 · 루트 `workspace.dependencies` 에 `md5` 복원. 그 뒤 `cargo check -p wie-backend --all-targets`.

| | 전(`origin/main` `1853f3b6f`) | 후(이 PR) |
|---|---|---|
| 충돌 경로 | 137(UU 106 · AA 27 · UD 4) | 139(UU 108 · AA 27 · UD 4) |
| `wie-backend` 오류 | **4** — `system.rs:31` E0603 · `task.rs:44` E0308 · `lib.rs:99` E0277 · `lib.rs:102` E0599 | **2** — `lib.rs:99` E0277 · `lib.rs:102` E0599 |

- 충돌이 2 는 것은 `task.rs` 와 `system.rs` 가 «조용히 (틀리게) 합쳐지는 파일»에서 «보이는 충돌»로 옮겨 온 것이다. 둘 다 덩어리 1개이고 답은 우리 쪽이다(`git merge-file` 로 따로 확인: `task.rs` 1 · `system.rs` 1 · `executor.rs` 10 — 전에도 10).
- ★`system.rs` 의 그 덩어리 **밖**에서는 upstream 의 두 줄이 여전히 조용히 들어온다: `pub use … normalize_guest_path` 와 `FilesystemOverlay::new(…, pid)`(aid → pid). 탐침은 파일을 통째로 `--ours` 로 풀어 못 본다. 0551 §8 의 «파일시스템 이름공간» 사람 결정이 그 줄이다 — S3 몫이다.
- `wie-midp` `event_queue.rs` 는 이 PR 뒤에도 충돌 없이 합쳐지고(같은 `merge-file` 확인 rc=0) 대기 루프가 그대로 남는다.
- **남은 2건은 zip 8 → 9 다**(`file.name()` 이 `Result` 를 돌려준다). 실행기와 무관하고, S1 레인(`wie-4` · 브랜치 `wie-upstream-sync2-s1-rustjava-pin-0-4-1` · 미푸시 WIP `4fbd7657d`)이 `wie-backend/Cargo.toml` 과 `lib.rs` 에서 이미 고치고 있다(읽기로 확인). 그래서 이 PR 에 넣지 않았다. 0551 §7 이 S2 에 적은 기대 «탐침 P1 에서 `wie-backend` 오류 0»은 S2 만으로는 서지 않고 S1 이 같이 착지해야 선다.
- 다음 벽: 스크래치 탐침에만 zip 고침(`lib.rs` 의 `file.name()` 두 곳 — S1 WIP 과 같은 꼴)을 얹고 `cargo check --workspace --all-targets --keep-going` 을 돌렸다. **`wie-backend` 오류 0 — 탐침이 `wie-backend` 를 지난다.** 다음에 죽는 곳은 셋이다(오류 38 · 그 아래 크레이트는 이 셋에 막혀 컴파일까지 못 간다).
  - `wie-jvm-support` 27 — RustJava 0.4.1 API(`name()` · `descriptor()` 가 `Cow<str>` · 인자 수). S1 축이다.
  - `wie-core-arm` 7 — `binary_patches/{parser,patch}.rs` 없음 · `md5` · AOT 엔진 트레이트(`record_image` 외) · `instructions_executed`. 0551 §7 S3 ⑴.
  - `wie-wipi-c` 4 — upstream 쪽 코드가 부르는 `decode_image_framebuffer` · `DatabaseRepository::usage` · `FilesystemOverlay::{remove,usage}` 가 `--ours` 로 푼 backend 파일에 없다. S3 의 3-way 몫이다.

### 7. S1 · 열린 PR 과의 겹침

- **S1**(RustJava 핀 · 위 WIP `4fbd7657d` · PR 아직 없음): 이 PR 과 같은 파일은 `wie-midp/src/classes/net/wie/event_queue.rs` 하나다. `git merge-tree --write-tree` rc=0(충돌 0). `wie-backend` 에서 S1 이 만지는 것은 `Cargo.toml` · `lib.rs` 이고 이 PR 은 `executor.rs` · `system.rs` · `task.rs` · `speed_clock.rs` · `tests/system.rs` 다. 착지 순서 제약 없음 — 단 S1 이 WIP 라 PR 이 열리면 다시 재야 한다.
- **PR #589**(`wake_tick` · 11-11 부터 열림): `wie-backend/src/executor.rs` 에서 충돌한다(`merge-tree` rc=1 · 그 파일 하나). #589 는 `tick_for` 에 `max_wait` 인자를 더하고 paced 판정 한 줄을 고친다. 나중에 착지하는 쪽이 그 한 줄과 `tick_for` 서명, 시험 도우미(`sleeper`)를 맞춰야 한다. `system.rs` · `speed_clock.rs` 는 깨끗이 합쳐진다.
- PR #599 · #601: `merge-tree` rc=0.

### 8. 한계

- 걸린 태스크만 있는 틱이 이제 바로 끝난다. `wie_validate` 처럼 틱을 쉬지 않고 부르는 호스트에서는 그런 판의 틱 수가 빨리 는다(잠만 자는 판이 이미 그렇듯). 기본 `--max-ticks`(5천만)로 키를 넣는 실행은 대본이 끝나기 전에 그 상한에 닿을 수 있다 — census 는 이미 상한을 풀고 돌린다(`NO_TICK_CAP`). 러너 블록의 `keydraw_*` 는 1,040만 · 1,657만 틱에 `deadline` 으로 끝났다(27/27).
- `Executor::sleep` 의 깃발(§2 ⑵)과 예산 안 기다림(§2 ⑴)은 upstream 과 다르다. S3 에서 `executor.rs` 는 우리 쪽으로 풀고 upstream 시험 2개가 남아 있는지 본다.
- 브라우저에서 «걸린 태스크가 있는 판의 틱이 실제로 짧아졌는지»(발열)는 따로 재지 않았다. PR #589 의 `tick_occupancy` 가 그 자다.
