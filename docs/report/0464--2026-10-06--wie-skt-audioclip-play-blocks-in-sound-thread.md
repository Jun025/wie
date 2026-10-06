## [2026-10-06] SKT `AudioClip.play` — 소리 스레드에서 재생 동안 블록 · `stop`/`close` 가 `UserStopException` 으로 끊는다 (wie-skt-audioclip-play-blocks-in-sound-thread)

**무엇을**: SKT `AudioClip.play()` 를 «소리 스레드»에서 부르면 그 곡이 끝날 때까지 돌아오지 않게 했다. 그동안 다른 스레드가 같은 클립에 `stop()`·`close()` 를 부르면 `play()` 가 `com.skt.m.UserStopException` 으로 끝나고 소리도 멎는다. 소리 스레드가 아닌 곳의 `play()` 는 종전처럼 바로 돌아온다.

**왜**: 운영자 채택 제안 `2026-10-06-skt-3d-and-ktf-reloc-verdict#p0`(0455 §2-3). `71d1d8235bd1` 은 1스테이지 시작에서 멈췄다. 다른 SKT 소리 스레드는 곡을 100ms 마다 다시 틀었다.

**사용자 영향**: `71d1d8235bd1` 의 1스테이지가 시작되고 진행된다(limited → playable). SKT 소리 스레드 타이틀 일부는 배경음이 처음부터 다시 시작되는 일이 줄었다. 다른 SKT 타이틀의 부팅·화면·입력·소리 판정은 바뀌지 않았다(§3).

### 1. 블록 기준 — 어느 스레드를 블록하나 (정적 · `javap -c` 전수)

SKT 코퍼스 84파일(82종)을 다시 셌다. 0455 의 «74/84» 는 셈법이 달라 이 회차가 다시 잰 값으로 바꾼다.

| 무엇 | 수 |
|---|---|
| `AudioClip.play()` 호출 지점이 있는 타이틀 | **79** (호출 지점 94) |
| ⒜ 호출 메서드가 `run()` | 71종 |
| ⒝ 호출 메서드가 도우미이고, 그 도우미를 `run()` 이 부름 | 8종 |
| 호출 지점을 감싸는 `catch` | `Exception` 93 · `UserStopException`+`Exception` 1(`47fe675bfffd`) · 없음 0 |

- ⒜⒝ 의 `run()` 은 전부 소리만 다룬다. 호출하는 것은 `AudioClip`·`Thread`·`Object`·문자열·리소스 읽기와 자기 소리 도우미뿐이다. `repaint`·`serviceRepaints`·`Graphics` 호출은 0이다.
- 주 스레드 효과음 표본은 `1367261bc3ee` 다. 이 게임은 소리 스레드(`c.run → c.b → play`) 말고도 게임 코드(`w.c(int)`·`w.T()`·`ak()` → `c.b` → `play`)에서 효과음을 낸다. 이 경로에서 `play` 와 `run()` 사이에는 프레임이 셋 이상 있다.
- **기준**: Java 스택에서 `play` 의 호출자 또는 그 호출자가 `run()V` 이면 블록한다(`stack_trace()[1..=2]`). ⒜⒝ 79종을 모두 덮고, 위 주 스레드 경로는 덮지 않는다.
  상한(코드 주석 `ponytail:`): 게임 루프의 `run()` 이 소리 도우미를 직접 부르면 그 효과음도 블록한다. 코퍼스에는 그런 곳이 없다.
- **예외 종류**: `47fe675bfffd` 는 다른 스레드의 `clip.stop()` 으로 소리 스레드의 `play()` 를 끊고, 그 `play()` 를 `catch (UserStopException)` 으로 받는다. 실기 의미가 이름으로 남은 유일한 근거다. 나머지 93곳은 `Exception` 을 잡으므로 `UserStopException extends Exception` 은 94곳 모두에서 잡힌다.

### 2. 의미

- `play()`(소리 스레드): 종전대로 소리를 시작한다. 그다음 `soundingUntil`(곡 길이) 까지 `Thread.sleep` 으로 기다린다(20ms 단위).
  - `pause()` 중이면 계속 기다린다. 이렇게 안 하면 반복 재생 스레드가 일시정지를 바로 다시 틀어 버린다.
  - 다른 스레드의 `open()` 이면 정상으로 돌아온다.
  - 다른 스레드의 `stop()`·`close()` 면 `UserStopException` 을 던진다.
- `close()`: 블록 중인 `play()` 가 있을 때만 소리를 멈춘다. 더팜1 의 `open → play → close`(같은 스레드 · 블록 없음)는 종전대로 소리를 남긴다.
  선행 결론(`wie-featurephone-audio-loop-overlap-and-worklet-sequence-leak`: `close` 는 소리를 끊지 않는다 · 닫힌 클립 `stop` 무시 · 고아 루프 정지)은 그대로다. `close` 가 소리를 끊는 경우는 «블록 중인 `play()` 를 다른 스레드가 끝낼 때» 하나만 더해졌다.
- `loop()` 는 바꾸지 않았다(`47fe675bfffd` 는 `loop()` 도 같은 `try` 안에 있지만, 블록하는 `loop` 는 이 회차 근거 밖이다).
- `71d1d8235bd1` 의 교착이 풀리는 길(0455 §2-3): `stop()` 이 `clip.close()` 를 부르면 소리 스레드의 `play()` 가 `UserStopException` 으로 끝난다. 그 `catch` 가 `isRepeat=false` 로 만들고, 스레드는 `wait()` 로 모니터를 놓는다.

### 3. 측정

**3-1. `71d1d8235bd1`**: 같은 키 스크립트(`WAIT:5 OK:3 ×3 NUM2:4 OK:3 ×15` · 3초 간격 스냅샷 22장)로 비교했다.
`NUM2` 는 «New Game» 이다. 0455 의 키 스크립트처럼 `OK` 만 누르면 시작 메뉴에서 넘어가지 않는다.

| | 스냅샷(33~66 s) | 점수 | plays / 66 s |
|---|---|---|---|
| 전(`origin/main` `9c8d9729`) | **12장 동일** · 1스테이지 화면에서 정지 | 0 | 265 |
| 후 | 22장 모두 다름 · 춤 진행 | 10 → 180 | 22 |

후 빌드의 longplay 600 s(census `--only long` · 같은 메뉴 레시피 + 순환 키): `still` 1 · 마지막 스냅샷(580 s)도 1스테이지 진행 중이다.
Java 예외는 첫 실행의 `RecordStoreNotFoundException` 1건뿐이다. 이것은 전과 같다. census 판정은 longplay `ok` 이다.

**3-2. SKT 전수 짝**: `playability-census.mjs run --only probe`, `build-slot --long` 임대 1개 안에서 base → head 순서로 82종을 쟀다(`--jobs 3`). load1 은 base 10~33, head 13~38 이다.

| 축 | 82종 중 바뀐 것 |
|---|---|
| status · boot · render · input · sound | **0** |
| A/B `result` · `stop` · 첫 Java 예외 | **0** |
| A/B Java 예외 수 | **0** |
| plays(A·B 164 probe) | 감소 17 · 증가 3(모두 +1) · 같음 144 |

- 감소가 큰 것: `71d1d8235bd1` B 277→2 · A 95→10, `9e2307e4fa88` A 86→3 · B 80→2, `2f84b8cc870d` A 84→3 · B 79→2, `bf54c05e58a9` A 32→2 · B 26→2, `e9ea67541b01` A 46→12. 곡을 100ms 마다 다시 틀던 소리 스레드들이다.
- 주 스레드 효과음 표본 `1367261bc3ee` 는 A 4→4 · B 2→2 · 예외 0→0 이다(행 동일).
- paints 차이가 25% 를 넘은 3건은 base·head 를 **동시에** 2회씩 다시 쟀다. 결과는 `2b6d78567795` 5191/5191 · 5137/5131, `e9b262f8f297` 66/113(1회차) · 111/112, `f78334f270be` 1883/1888 · 1617/1617 이었다. 부하 차이로 본다.

**3-3. 되돌리면 red — 6 변이 전건**:
- 블록 제거 → 두 시험 FAILED.
- 항상 블록 → `play_on_a_sound_thread_blocks_until_the_clip_runs_out`(효과음 대기) FAILED.
- `run()` 직속만 블록(도우미 깊이 무시) → 같은 시험 FAILED.
- 예외 대신 정상 반환 → `stop_or_close_ends_a_sound_threads_play_with_user_stop_exception` FAILED.
- `close` 가 소리를 안 끊음 → 같은 시험(`["play","stop"]`) FAILED.
- `stop` 이 블록을 안 끊음 → 같은 시험 FAILED.
- 원상태에서는 7/7 통과한다.

### 4. compat

`71d1d8235bd1` 1행만 바꿨다: status `limited → playable` · longplay `unknown → ok` · knownIssue 「첫 스테이지가 시작되기 직전에 멈춰요.」 삭제. speed 는 재지 않아 `unknown` 그대로다.
다른 81종은 3-2 에서 축이 바뀌지 않았으므로 손대지 않았다.

### 검증

- `cargo fmt --check` · `clippy --all -D warnings`(stable · beta) · `clippy --target wasm32 -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all`(719 통과 · 0 실패) 모두 rc=0.
- 러너 줄(이 브랜치 release `wie_validate`):
  - `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` 는 PASS 다.
  - `keydraw_ktf/lgt --inject --expect-last-frame` 는 첫 실행이 `UNMEASURED · stop max-ticks` 였다(load 높음).
  - AGENTS 지시대로 `--max-ticks` 를 올리자 전·후 모두 PASS · rc=0 이었다. paints 는 79/80 · 55/55 다.

### 후속

없음. `71d1d8235bd1` 의 speed·progress 는 다음 census 회차가 잰다. 0455 후속 3 의 longplay 는 이 회차에서 쟀다.
