## [2026-10-05] KTF·LGT 시작 스레드도 떼기 · 347종 짝 · J2ME 표본 · #473 비차단 잔여 3건 (wie-startup-thread-ktf-lgt-pairs-and-sound-fix-residuals)

**무엇을**: #473(0440 §4-2)이 SKT·J2ME 에만 적용한 «시작 스레드 떼기»를 KTF·LGT 시작 함수에도 적용했다. 둘 다 시작이 끝나면 `JvmSupport::finish_launch` 를 부른다. 게이트② 비차단 지적 m1~m3 도 고쳤다.
- m1: `parse_smaf_in` 의 길이 덧셈을 u32 `saturating_add` 로 바꿨다.
- m2: 네 통신사 시작 함수가 `finish_launch` 로 끝나는지 잠그는 시험을 더했다.
- m3: MIDP `SmafPlayer` 의 `lengthMs` 가 소리와 같은 `parse_smaf_in` 을 쓴다.

**왜**: 0440 §8 의 후속(347 · M)과 게이트② 지적 m1~m4.

**사용자 영향**: 체감 변화는 없다(아래 §2 — 347종 결과 같음 · `activeCount` 를 부르는 KTF·LGT 타이틀 0). 클립 길이를 읽는 MIDP 게임은 여분 붙은 SMAF 에서 «바로 끝남» 대신 실제 길이를 받는다.

### 1. KTF·LGT 진단 — 같은 모양인가

- 코드: KTF `start`·LGT `do_start` 는 `JvmSupport::new_jvm`(→ rustjava `Jvm::new` 가 시작 스레드를 attach)을 부르는 전용 spawn 태스크에서 돌고, 끝날 때 떼지 않았다. SKT·J2ME 와 같은 모양이다.
- 진단 빌드(커밋 0): `jvm` 0.1.1 의 `attach_thread`·`detach_thread` 가 붙은 스레드 수를 찍게 했다. 표본 15종(KTF 7 · LGT 8 · playable 에서 12개 간격) × 20초 · 전/후 동시 2.
  - 플레이 중 마지막 값(= 그 시점 `Thread.activeCount()` 가 돌려줄 값): **15/15 에서 후가 전보다 정확히 1 작다**. 전 2~5 · 후 1~4. 예: KTF `8cff5587290d` 2 → 1 · `513d4b49c0be` 4 → 3 · LGT `a23f3c9fc2cb` 2 → 1.
  - 후 쪽은 모든 표본에 `detach` 1줄이 더 있다(시작 끝).
- ⇒ 같은 모양이다. 그래서 같은 처방을 적용했다.
- 단, **그 값을 읽는 타이틀은 없다.** `Thread.activeCount` 몸체를 찍게 한 진단 빌드로 347종 짝(694회 실행)을 돌렸고, 호출 **0회**였다. 문자열 스캔으로는 KTF 83종(전부 `client.bin`)·LGT 0종이 `activeCount` 를 갖는다. 0440 의 «import 이름표라 부른다는 증거는 아니다»가 실측으로 맞았다(30초 프로브 범위).

### 2. 347종 짝(KTF 269 · LGT 78)

- 방법: 전 = `e8b2b1e7`(main) · 후 = 이 브랜치. 둘 다 위 `activeCount` 진단을 넣은 빌드다(그 줄은 0회 실행 — 동작 차이 없음). 프로브 A(전수 도구와 같은 인자 · 30초)를 전/후 동시 2로 돌렸다. 전체를 `build-slot run --long` 한 임대 안에서 돌렸다. load1 10~156(중앙 41).
- 결과(PASS/FAIL/UNMEASURED + stop)가 같은 것: **345/347**.
  - 다른 2종은 재측 2짝에서 양쪽이 같았다.
    - `9c1c446a36e2`: 첫 짝 UNMEASURED/max-ticks → PASS/deadline. 재측 2회는 양쪽 UNMEASURED/max-ticks.
    - `49f2734f17d8`: 첫 짝 PASS/deadline → PASS/max-ticks. 재측 2회는 양쪽 같음.
- 재생 수가 같은 것: **314/347**(KTF 239/269 · LGT 75/78).
  - 다른 33종 중 28종은 ±1~2 다.
  - ±3 이상 5종은 재측 2짝을 했다. 같거나 방향이 뒤집혔다. 예: `2b1ed0c8d061` 26→14 · 18→13 · 14→22.
  - `7da00ecd4804` 0→6 은 0440 이 적은 기존 흔들림이다. 재측 2회는 양쪽 0이었다.
- 소리 없음(빈 재생 제외 재생 0) 타이틀: 47 → 45. 갈린 2종은 위 `7da00ecd4804` 와, 전 쪽이 굶은 `f12d97040c33`(paints 48 vs 176)이다.
- **나빠짐 0**(재측으로 확인되지 않은 차이는 전부 흔들림).

### 3. J2ME 표본 — #473 의 activeCount 변경

- 코퍼스에 J2ME 타이틀은 **0종**이다(compat.json 429 = KTF 269 · SKT 82 · LGT 78). 그래서 저장소의 J2ME 손님 4종(draw · text · pace · sound)과 이 회차 스크래치 손님 1종(`count` — `paint` 에서 `Thread.activeCount()` 를 출력 · 커밋 0)으로 쟀다.
- 전 = `9b4eb62e`(#473 base) · 후 = `e8b2b1e7`(main). 무키(`--timeout 10`) · `--inject` 두 방식.
  - 결과·stop·content: **10/10 짝 같다**. draw·text 무키 PASS · 키 UNMEASURED(max-ticks) · pace 무키 PASS · 키 UNMEASURED · sound·count 는 그리지 않는 손님이라 FAIL(빈 화면)이 양쪽 같다.
  - paints 는 ±1(키 단계 수 ±1과 같이 움직임) · pace 무키만 164 / 151(벽시계).
  - sound 키 재생 6 / 5 — 받은 키 수(6 / 5)와 같다.
- `count` 손님의 `activeCount`(진단 빌드): 전 **2**(6/6회) → 후 **1**(6/6회). #473 이 J2ME 에서도 의도대로 움직인다.
- `dae153ad28ba`(SKT) 3회 재측(프로브 A 30초 · 전/후 번갈아 1개씩): 재생 **1/1 · 1/1 · 1/1**, MIDI 1,781 모두 같다. 1→230 은 재현되지 않았다. 이 타이틀은 `activeCount` 를 갖지 않는다(문자열 스캔 SKT 1종 = `66959afab216`) ⇒ 그 흔들림은 #473 변경과 무관하다.

### 4. 잔여 m1~m3

| | 고침 | 시험 | 되돌림 |
|---|---|---|---|
| m1 | `parse_smaf_in` — `u32::from_be_bytes(..).saturating_add(8) as usize` | `smaf_declaring_a_length_near_u32_max_loads_empty`(머리 길이 `u32::MAX`) | `+ 8`(u32)로 되돌리면 `attempt to add with overflow` red(실측). 옛 식(usize)은 64비트에서 넘치지 않아 wasm32 에서만 red 였다 — 덧셈을 u32 로 옮겨 어느 호스트에서도 그 경로를 시험한다 |
| m2 | — | `every_carrier_launch_ends_through_finish_launch`(SKT·J2ME·KTF·LGT 시작 소스에 `finish_launch(&jvm, ` 1회) | SKT 를 옛 `if let Err … Ok(())` 로 되돌리면 `skt left: 0 right: 1` red(실측) |
| m3 | `parse_smaf_in` 을 `wie_backend` 에서 공개 · `SmafPlayer::init` 이 그것을 쓴다 | `test_a_padded_clip_still_has_a_length`(여분 8바이트 붙은 클립 → `createPlayer` → `lengthMs > 0`) | `parse_smaf` 로 되돌리면 `lengthMs 0` red(실측) |

- m2 는 소스 문자열 시험이다. 시작 함수는 손님 전체가 있어야 돌고, `cargo test` 에서 도는 손님은 J2ME 뿐이다(SKT 고정 손님 0). `finish_launch` 의 동작은 기존 단위 시험이 잠근다.

### 5. 검증

- 4 게이트: `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` · wasm32 · `+beta` 모두 rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` **692 pass / 0 fail**.
- 러너 블록: draw · helloworld ×2 · text PASS · keydraw ×2 `--inject --expect-last-frame` PASS · rc 0.
- `player-data OK — 429 · 388/24/17`. compat.json 은 바꾸지 않았다 — 등급 변화 0(§2: 결과 345/347 같음, 다른 2종은 재측에서 같음).
- 측정 규율: 에뮬레이터는 동시 3개를 넘지 않았다(짝 스윕 2 + 단건 1). `nohup` 0. 진단 빌드는 스크래치 target 에서만 했다.
  - 함정 1건: 두 체크아웃(이 브랜치 · `9b4eb62e`)이 같은 스크래치 target 을 쓰자 cargo 가 다른 체크아웃의 `wie-backend` 산출물을 최신으로 보고 빌드가 실패했다. 그 빈 빌드의 결과는 버리고, 새 target 에서 다시 쟀다(§1 수치는 다시 잰 것). §2 스윕 바이너리는 그 이전에 만든 것이라 영향이 없다.

### 6. 남은 것

- 없음. `activeCount` 를 부르는 KTF·LGT 타이틀이 나오면 이 고침이 그때 값을 한다(지금은 상태 정리뿐이다).

### 7. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 5회 / 5쌍 + SUFFIX-ATTACHED 0회 / 0쌍. 5회는 전부 이 회차가 고친 파일(`wie-backend/src/system.rs` · `wie-lgt/src/emulator.rs` · `wie-midp/.../smaf_player.rs`)의 **기존** 주석·시험 줄이다 — 이 회차가 더한 줄(`git diff origin/main` 의 `+` 줄)에는 0 이다. 이 회차는 타이틀을 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=9 tree=272bcc142a6aa3df B=5/5 P=0/0 S=0/0 -->
