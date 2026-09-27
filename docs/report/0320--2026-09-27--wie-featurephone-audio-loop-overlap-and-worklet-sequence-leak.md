## [2026-09-27] 소리 복구(#348) 후속 — 겹쳐서 안 끝나는 BGM · 매 프레임 재시작 BGM · 워클릿 시퀀스 무한 누적 · 끝난 효과음 resume 무음 (wie-featurephone-audio-loop-overlap-and-worklet-sequence-leak)

**무엇을**
- `wie-skvm` `net.wie.WieAudioClip`: 루프를 시작하면 **닫힌(`close`) 클립이 남긴 루프**를 먼저 멈춘다(`orphanLoops` 정적 필드). 열린 클립의 루프는 건드리지 않는다. 이미 루프 중인 클립의 `loop()` 는 아무것도 하지 않는다.
- `wie_featurephone` `audio.rs` + `audio_worklet.js`: 워클릿이 보유하는 시퀀스를 **최근 재생 32개**로 제한(`RESIDENT_SEQUENCES`) — 넘치면 가장 오래 안 튼 핸들을 `evict`. 재생 중인 소리는 끊기지 않고(재생이 시퀀스 객체를 직접 쥔다), 다시 틀면 이벤트를 재전송한다. `registerProcessor` 중복 등록 가드(검수 minor 4). 계측용 `stats` 메시지.
- `wie-midp` `SmafPlayer.getState()`: 스텁(항상 300) → 재생 중 `STARTED(400)` / 아니면 `PREFETCHED(300)`. 1회 재생은 **클립 길이가 지나면** 끝난 것으로 본다. `wie-wipi-java` `Player.resume` 은 `started` 필드 대신 `getState()` 로 판정 → 끝난 효과음을 resume 하면 다시 난다.
- `wie_validate` `audio` 키에 `loop_overlaps`(다른 핸들의 루프가 살아 있는 채 시작된 루프 수)·`handles`(재생된 서로 다른 핸들 수) — 보고만, 판정 무관.
- `scripts/check-audio-worklet.mjs` 2케이스 추가(7 → 9).

**왜**: #348 게이트② 검수 minor 1·2·3·4(`reports/wie-featurephone-audio-midi-bgm-and-most-sfx-silent.review.md`).

**사용자 영향**: SKT 게임에서 곡이 바뀔 때 앞 곡이 겹쳐 계속 도는 현상(사고뭉치트윈스)과, 배경음악이 첫 음에서 매 프레임 다시 시작되는 현상(나이트메이커)이 사라진다. 효과음을 매번 새로 여는 게임(더팜1)을 오래 켜 두어도 브라우저 메모리가 계속 늘지 않는다. 다른 게임의 소리는 그대로다.

### 1. 겹치는 루프 — 규칙을 코퍼스가 골랐다

SKT 50타이틀 전수(`wie_validate --inject --timeout 60`, `RUST_LOG=wie_skvm=debug` 로 `WieAudioClip` 호출열 추출):

| 형태 | 타이틀 | 뜻 |
|---|---|---|
| 클립 1개를 `stop → open → loop/play → close` 로 재사용 | 아슬아슬타워쿤·생과일타이쿤2·노리타이쿤·드래곤나이트EX 등 **대다수** | BGM 이 `open → loop → close` 로 시작된다 |
| `play → close → stop` 이 **66 µs** 안에 | 드래곤나이트EX | 닫힌 클립의 `stop` 이 소리를 끊으면 효과음이 전부 죽는다 |
| 클립 A `open → loop → close` → A`.stop`(무시) → **새 클립 B** `open → loop` | **사고뭉치트윈스**(유일) | A 의 루프를 멈출 길이 없다 |
| 매 프레임 `loop()` | 나이트메이커 | 60초에 play **86,770** — 매번 첫 음으로 되감김 |

- 기각 ⒜ 검수자 안 「`close` 시 루프 핸들만 stop」: 위 1행의 BGM 이 **시작 즉시 무음**이 된다.
- 기각 ⒝ 「닫힌 클립의 `stop` 도 소리를 끊는다」: 위 2행(드래곤나이트EX)의 효과음이 66 µs 만에 끊긴다 — #348 이 더팜1 에서 반증한 «`close` 가 끊는다»와 같은 형태.
- 채택: **루프가 시작될 때, 닫힌 클립이 남긴 루프를 멈춘다.** 닫힌 클립의 루프는 게임이 더는 멈출 수 없는 유일한 소리이고(`stop`·`pause` 무시, 같은 객체의 `open` 만 해제), 열린 클립의 루프는 게임이 스스로 멈출 수 있으니 둔다. + 루프 중 `loop()` 는 무시.

전/후 (SKT 50타이틀 · 같은 명령 · loadavg 375–417):

| | 수정 전 | 수정 후 |
|---|---|---|
| `loop_overlaps` 합 | **1**(사고뭉치트윈스) | **0** |
| 사고뭉치트윈스 | plays 2 · stops **0** · overlaps 1 | plays 2 · stops **1** · overlaps 0 |
| 나이트메이커 | plays **86,770** · midi 68,201,342 | plays **2** · midi 1,694 |
| 소리가 난 타이틀 / PASS | 37 / 44 | 37 / 44 |

- 나머지 타이틀의 plays 차이(±1~8)는 부하 아래 60초 창에 든 구간 차이다(같은 호출 규칙은 바뀌지 않았다 — 위 규칙은 오직 «닫힌 클립 루프가 살아 있을 때 새 루프» 와 «루프 중 `loop()`» 에서만 발동하며, 전수에서 앞은 1회 · 뒤는 나이트메이커뿐).

### 2. 워클릿 시퀀스 누적 — Close 대신 상한

`AudioCommand` 에 Close 를 더하지 않았다: upstream 소유 타입이고, 다른 싱크 `wie-web/src/rust/audio_sink.rs` 가 **완전 매칭**이라 변형 하나로 그 크레이트가 깨진다(upstream 크레이트 무접촉 경계). 싱크 안의 LRU 는 다른 누구도 필요 없고, 재생 중 소리를 끊지 않으며(워클릿 playback 이 시퀀스 객체를 직접 쥔다), 되살릴 때는 Rust 가 이벤트를 다시 보내므로 **소리 손실 0**이다.

브라우저 10분 세션 — 더팜1(SKT) · headless Chromium · 실제 `AudioContext → 마스터 GainNode → AnalyserNode` · 키 700ms 순환 · main/이 브랜치 **같은 시간대 동시 실행**:

| 시점 | main 워클릿 보유 시퀀스 | 이 브랜치(워클릿 `stats` 응답) |
|---|---|---|
| 10s | 10 | 10 |
| 70s | 96 | **32** |
| 310s | 439 | 32 |
| 600s | **853**(선형 · 약 85/분) | **32** |
| 초당 RMS 평균 · 무음 초 | 0.1337 · 0/600 | 0.1338 · 0/600 |
| play / evict | 853 / 0 | 853 / **821** |

- main 은 `stats` 를 모르므로 «이벤트를 실어 보낸 서로 다른 핸들 수»(main 은 해제가 없어 = 워클릿 보유 수)로 셌다. 이 브랜치는 워클릿이 직접 답한 `sequences.size` 이고, Rust 가 보낸 값(ev − evict)과 전 구간 일치했다.
- 더팜1 은 853 play 가 **전부 새 핸들**이라 재전송은 0회(추가 비용 0).

### 3. 끝난 효과음 resume — 구현

`SmafPlayer` 가 `start` 시각과 클립 길이(`parse_smaf` 로 계산 — 마지막 이벤트와 가장 긴 PCM 끝 중 늦은 쪽, 워클릿 `load` 의 루프 길이와 같은 정의)를 기억하고, `getState()` 가 «루프면 멈출 때까지, 1회면 길이 동안» `STARTED` 를 답한다. `Player.resume` 은 `STARTED` 가 아니면 다시 튼다. 배틀몬스터의 `play → resume`(즉시)은 길이 안이라 그대로 무시된다(아래 회귀 표 plays 4 = 4).
- 비용: `SmafPlayer` 생성마다 SMAF 를 한 번 더 파싱한다(백엔드 `Audio` 가 길이를 내주지 않고, 그것을 여는 것은 upstream 크레이트 변경).
- MIDP 게임이 `getState()` 를 폴링하면 이제 300 고정 대신 실제 상태를 본다 — 로컬 J2ME 타이틀 0 이라 타이틀 계측 불가(#348 과 같은 한계).

### 회귀 — BGM (브라우저 · main/이 브랜치 동시 쌍)

| 타이틀 | main | 이 브랜치 |
|---|---|---|
| LGT 배틀몬스터 40s | plays 4 · 후반 RMS 0.1012 | plays 4 · **0.1012** |
| KTF 간호사타이쿤 40s | plays 7 · 0.0734 | plays 7 · 0.0814 |
| SKT 더팜1 600s | 0.1337 | 0.1338 |
| SKT 사고뭉치트윈스 60s | 동시 루프 **2** · 0.2229(두 곡 합) | 동시 루프 **1** · 0.1577 |
| SKT 나이트메이커 60s | play **3,694,995**(브라우저에선 tick 마다) · 0.1945 | play **2** · 0.1144 |

페이지 에러 0 · 워클릿 노드 1 · 전 실행 `threw null`.

### 되돌리면 red

| 되돌림 | 결과 |
|---|---|
| 루프 시작 시 고아 루프 정지 제거 | `audio_clip_loop_stops_loops_orphaned_by_close` FAILED |
| 루프 중 `loop()` 무시 제거 | 같은 시험 FAILED |
| `Player.resume` 이 `started` 필드를 다시 읽음 | `test_clip_resume_replays_a_finished_one_shot` FAILED |
| 워클릿 `evict` 무시 | check-audio-worklet ⑧ FAIL(sequences 11 → 11) |
| `registerProcessor` 가드 제거 | check-audio-worklet ⑨ FAIL(NotSupportedError) |
| `loop_overlaps`·`handles` 집계 | `audio_tally_counts_what_reached_the_sink` 가 두 값을 문자열로 고정(되돌림 실행은 안 함 — 보고 전용 키) |

### 회귀

`cargo fmt --check` · `clippy --all -D warnings` · wasm32 clippy(워크스페이스 + `-p wie_featurephone`) · `+beta clippy` rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` **510 passed / 0 failed** · runner 블록 전건 PASS(keydraw 2종 `--expect-last-frame` rc 0 · paints 55 · text_j2me PASS) · `npm run build:wasm` rc 0 · `check-engine-contract` **109 pass / 0 violation** · `check-audio-worklet` **9/9** · 글루 35,151 → **35,151 B**(불변) · wasm 15,557,806 → 15,583,237 B(+25,431 · 상한 안 · 원인 분해는 안 함).

### 한계

- Rust 쪽 LRU(`audio.rs`)는 네이티브 시험이 없다 — 크레이트가 wasm32 전용(Constraint 7). 판정은 위 브라우저 10분 표(워클릿 자가 보고 32 = Rust 계산 32)와 checker ⑧(워클릿 쪽)이 진다.
- 상한 32 는 «동시에 살아 있는 서로 다른 핸들» 최대치보다 크게 잡은 값이다(SKT 는 클립당 1 핸들). 33개 이상을 번갈아 트는 타이틀은 재생마다 이벤트 재전송 — 소리는 같고 메인 스레드 비용만 는다. 관측 사례 없음.
- 브라우저 프로브는 스크래치 도구다(커밋하지 않음) — 영구화는 worklog 제안.
