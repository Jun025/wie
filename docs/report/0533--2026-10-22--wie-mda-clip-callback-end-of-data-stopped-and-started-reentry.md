## [2026-10-22] 클립 콜백 남은 두 축 — END_OF_DATA·STOPPED 를 보내고, STARTED 안의 재생 재진입을 막았다 (wie-mda-clip-callback-end-of-data-stopped-and-started-reentry)

**무엇을**: 0530(#563)은 WIPI-C `MC_mdaPlay` 뒤 클립 콜백에 STARTED(2)만 보냈다. 이번 회차는 남은 두 축을 닫았다.
- ⑴ **END_OF_DATA(1)·STOPPED(3)** 를 보낸다. 한 번 재생(repeat 0)은 소리 길이 + 500ms 뒤 END_OF_DATA. `MC_mdaStop` 은 소리가 나던 클립일 때만 STOPPED. 반복 재생은 고리 경계에서 아무것도 보내지 않는다.
- ⑵ **STARTED 재진입 가드**: 클립의 STARTED 가 아직 전달 중(대기 포함)이고 소리가 나는 동안 같은 클립에 `MC_mdaPlay` 가 오면 무시한다.
- 덤으로 **SMAF 길이**(`convert_smaf_events` 의 `duration`)가 마지막 이벤트가 아니라 가장 긴 PCM 샘플 끝까지 가게 고쳤다. 이게 없으면 END_OF_DATA 가 소리 도중에 온다(§4).

**왜**: 0530 검수 m2(STARTED 재진입 가드 없음)·m3(END/STOPPED 미발신의 잔여 위험). 발권 `wie-mda-clip-callback-end-of-data-stopped-and-started-reentry`.

**사용자 영향**: 곡이 끝나면 다시 트는 게임(LGT `5814101b8010`)은 전에는 곡이 한 번 끝나면 조용했다. 이제 곡이 이어진다. 멈춤 알림을 받아야 다음 소리를 싣는 게임(KTF `4a4d2ac046f7`·`f981d228b757` · LGT `e68b1c8aef85`)은 전에는 첫 소리 뒤로 소리가 더 나오지 않았다. 이제 다음 음악·효과음이 나온다. 화면 진행은 바뀌지 않는다(§5 · 판정 69/69 동일).

### 1. 발신 시점의 근거 — WIPI 명세

`docs/reference/AromaWIPI_javadoc.zip` 의 `org.kwis.msp.media.Player`(WIPI-C `MC_mda*` 의 Java 쪽 짝. 상수 값이 `WIPIHeader.h:1075-1130` 과 같다):
- `play`: «매체처리를 시작하면 … START 상태가 전달된다» · «재생중인 클립으로 다시 재생할려고 하면 이 함수는 아무런 역할도 하지 않는다» · «클립데이타가 소진되면 … END_OF_DATA 상태가 전달된다».
- `stop`: «매체처리를 종료하면 … STOP 상태가 전달된다. 정지되어 있는 처리기에 대해서 이 함수를 다시 부르면 … 아무런 역활을 하지 않는다».

여기서 정한 규칙(`wie-wipi-c/src/api/media.rs` `play` 머리 주석):

| 경우 | 보내는 것 |
|---|---|
| 한 번 재생이 끝까지 감 | END_OF_DATA 한 번(길이 + 500ms 뒤 · §3). 그 뒤 클립은 멈춘 상태다 |
| `MC_mdaStop`(소리가 나던 클립) | STOPPED 한 번. 그 재생의 END_OF_DATA 는 취소된다 |
| `MC_mdaStop`(이미 끝났거나 멈춘 클립) | 없음 |
| 반복 재생의 고리 경계 | 없음. 데이터가 다 쓰이지 않고 처음부터 다시 돈다. MIDP 의 `setLoopCount` 처럼 매 회 알리는 명세는 WIPI 에 없다 |
| 재생 중 다시 `MC_mdaPlay`(STARTED 전달 뒤) | 종전처럼 소리를 다시 시작하고 STARTED 를 다시 보낸다. 앞 재생의 END_OF_DATA 는 취소된다 |
| 해제된 클립(`MC_mdaClipFree`) | 대기 중인 콜백 전부 버린다(`clip_id` 를 0 으로 지우고 콜백이 대조한다). 0530 검수 m4(해제된 포인터로 콜백)도 같이 닫힌다 |

모든 이벤트는 그 원인이 된 호출이 돌아온 **뒤에** 간다(STARTED·STOPPED = `spawn`, END_OF_DATA = 타이머). 0530 의 STARTED 와 같은 길이다.

### 2. STARTED 재진입 가드

명세 근거: «재생중인 클립을 다시 재생하면 아무것도 하지 않는다». 이 규칙을 **STARTED 가 아직 전달 중인 동안에만** 적용한다. 그 밖에서의 재재생은 종전처럼 소리를 다시 시작한다. 재생 중 재트리거로 효과음을 끊고 다시 내는 게임이 있을 수 있어서 그 동작은 바꾸지 않았다.
- 콜백이 STARTED 에 `MC_mdaPlay` 로 답하면, 종전에는 재시작 → STARTED spawn → 재시작이 끝없이 돌 수 있었다. 이제 그 재생은 무시된다.
- 콜백 안에서 `MC_mdaStop` 뒤 `MC_mdaPlay` 하면 «재생 중»이 아니므로 다시 시작한다. 가드의 조건은 «이미 재생 중»이지 «콜백 안»이 아니다.
- 실측: 96종 60초 스캔에서 가드가 걸린 게임 **0**(로그 `STARTED not yet delivered, ignored` 0건). 0530 검수 때처럼 관찰은 0 이다. 가드는 시험으로만 잠근다.

### 3. END_OF_DATA 의 500ms — `863b8ab6a21d` 가 잰 하한

처음 구현은 길이가 지난 그 시각에 END_OF_DATA 를 보냈다. 회귀 짝(§5)에서 LGT 리듬 게임 `863b8ab6a21d` 가 걸렸다. 자기 진도 레시피(`game_lab/recipes-progress/863b8ab6a21d.keys`)로 150초를 돌리면 base 는 «자동싱크조정» 화면을 지나 연주 화면까지 간다. head 는 150초 내내 그 화면에 멈췄다.

원인(binary.mod 디스어셈블 · `objdump --triple=thumbv5te`):
- 클립 콜백 `0x1c730` 은 받은 status 를 무조건 전역 `0x150174c` 에 적는다. STARTED 면 시작 시각(`0x15019f8` = 지금 + 200·X + Y)을, 3·9 면 경과도 적는다.
- 게임 자신의 재생 루틴(`0x1c6a0`)도 `MC_mdaPlay` **앞에서** status=2 와 시작 시각을 스스로 적는다. 콜백 없이도 돌게 짜였다.
- 싱크 단계(`0xeee0`): `state == 1 && status == 2 && 지금 − 시작 > 3000(0xbb8)` 일 때만 끝낸다.
- 싱크 클립 `synchTest0.mmf` 는 PCM 하나(12kHz 모노 32400 샘플 = **2700ms**)다. 정시 END_OF_DATA 는 2700ms 에 status 를 1 로 바꾼다. 그러면 3000ms 조건이 영영 서지 않는다.

X 는 사용자 싱크 설정(저장에서 읽고 ±1)이고, 새 저장에서 0 이다. 그렇다면 데이터 끝 정시에 END_OF_DATA 를 주는 단말에서는 이 싱크가 끝날 수 없다. 이 게임이 돌던 단말은 끝을 **적어도 300ms 늦게** 알렸다는 뜻이다. 지연을 바꿔 가며 같은 레시피로 40초씩 쟀다:

| END_OF_DATA 지연 | 싱크 통과 |
|---|---|
| 0ms | 0/1(150초) |
| 250ms | 0/2 |
| 350ms | 2/2 |
| 500ms | 1/1(150초 · 연주 화면 · «GAME OVER» 까지 base 와 같은 흐름) |

경계는 300ms(= 3000 − 2700)로 모형과 맞는다. 게임의 프레임 폴링 여유로 200ms 를 더해 **500ms** 로 했다(`END_OF_DATA_LAG_MS`). `ponytail:` 주석에 상한을 적었다 — 한 게임의 하한에서 온 한 값이다. 더 짧은 값이 필요한 게임이 나오면 그때 나눈다. 곡이 끝나면 다시 트는 `5814101b8010` 은 500ms 늦어도 곡이 이어진다(§5).
- 측정 주의: 250/350 짝 4판은 40초씩 **동시 4개**로 돌았다(상한 ≤ 3 을 1 넘김). 판정이 2/2·2/2 로 갈렸고 부하 굶주림이면 350 쪽이 실패해야 하므로 결론에는 쓰되 기록해 둔다.

### 4. SMAF 길이 — PCM 꼬리까지

`Audio::duration` 은 SMAF 에서 **마지막 이벤트 시각**이었다(`convert_smaf_events`). WAV(`parse_wav`)는 샘플 길이를 넣고, 브라우저 워클릿(`audio_worklet.js` `load`)과 MIDP `SmafPlayer`(`sequence_length_ms`)는 «마지막 이벤트와 가장 긴 PCM 끝 중 늦은 것»을 쓴다. SMAF 만 짧았다. 같은 식으로 고쳤다(시험 `smaf_duration_covers_the_longest_pcm_tail`).
- 실측 근거: LGT `e68b1c8aef85` 는 PCM 효과음을 틀고 `MC_mdaStop` 한 뒤 STOPPED 를 받아야 클립을 풀고 다음 소리를 싣는다. 길이 수정 전 빌드에서는 END_OF_DATA 가 샘플 도중(마지막 이벤트 시각)에 와 클립이 «끝남»이 됐다. 그래서 뒤이은 stop 에 STOPPED 가 없었고, base 와 똑같이 `MC_mdaStop` 만 되풀이했다(60초 재생 1회). 수정 뒤에는 STOPPED 5회 · 재생 6회다(2/2 재현).
- 이 값을 읽는 다른 곳: 네이티브 싱크(`src/audio_sink.rs` — 한 번 재생의 정리·반복 재생의 고리 주기)는 이제 브라우저와 같은 길이로 돈다. SKVM `WieAudioClip` 의 `soundingUntil` 은 PCM 꼬리까지 «재생 중»으로 본다. MIDP 는 원래 둘의 최댓값이라 바뀌지 않는다. MIDP 의 `sequence_length_ms` 는 이제 중복이지만 이번 범위가 아니라 두었다.

### 5. 회귀 — 콜백 클립 96종

표본: 0530 이 찾은 «콜백이 0 이 아닌 클립을 만드는» KTF·LGT 96종(KTF 61 · LGT 35) 전부. release `wie_validate` · `--inject` 기본 27키 · 60초 · `RUST_LOG=wie_wipi_c::api::media=debug` · 동시 ≤ 3 · phase 마다 `build-slot run --long`. base = `origin/main` `54d137de5`. head = 이 브랜치 최종(`337723894` 와 같은 엔진).

**최종 head 96종**: PASS 90 · UNMEASURED 5 · FAIL 1(`f770b15f8876` — 0530 에서 base 도 3/3 FAIL 이던 기존 것). END_OF_DATA 를 받은 게임 56 · STOPPED 29 · 재진입 가드 0.

**짝**: 첫 head(`head60.tsv`)에서 새 이벤트(END 또는 STOPPED)를 받은 게임 69종 전부를 base 로 다시 돌렸다. 최종 head 에서 새 이벤트를 받은 게임은 67종이고 모두 이 69 안에 있다(남은 2종 `1eaa92092bee`·`f770b15f8876` 은 길이 수정 전의 이른 END 만 받았다). **판정 69/69 동일**(stop 사유도 69/69 같다). paints 가 10% 넘게 다른 것은 `f2280c6699a0` 하나(base 305 · head 147)이고, base 재측이 157 이라 흔들림 범위 안이다(`rb.tsv`). 재생 횟수가 다른 8종은 base·head 를 2~3판씩 더 돌렸다:

| sha12 | base 재생 수 | head 재생 수 | 판단 |
|---|---|---|---|
| `5814101b8010` (LGT) | 2 · 2 · 2 | 9 · 9 · 9(최종 8) | **곡 끝 대기 형상.** END_OF_DATA 를 받으면 같은 곡을 다시 싣고 튼다. base 는 곡이 한 번 끝나면 조용했다 |
| `4a4d2ac046f7` · `f981d228b757` (KTF) | 1 · 1 · 1 | 2 · 2 · 2 | 인트로를 `MC_mdaStop` → STOPPED 를 받아야 클립을 풀고 배경음(반복)을 튼다. base 는 stop 만 세 번 하고 배경음이 없었다. paints 2392 로 화면은 같다 |
| `e68b1c8aef85` (LGT) | 1 · 1 · 1 | 6 · 6 · 6 | 같은 형상(§4). base 는 `MC_mdaStop` 을 끝없이 되풀이했다 |
| `863b8ab6a21d` (LGT) | 4 · 4 · 4 | 2 · 2 · 2(첫 구현) → 최종 차이 없음 | §3 의 회귀. 500ms 지연으로 닫았다 |
| `2b1ed0c8d061` `2bafbffeb742` `7e2247bdf565` `f2280c6699a0` | 21~24 · 9~33 · 18~21 · 4~6 | 14~25 · 14~27 · 21~24 · 4~8 | 범위가 겹친다 — 실행 간 흔들림. `f2280c6699a0` 는 화면 11장(5초 간격)을 base·head 로 나란히 봤다: 같은 화투 판 진행 |

**곡 끝을 기다리는 형상**(발권 Acceptance): `5814101b8010` 을 찾았다. 전후 비교는 위 표 — base 재생 2회(곡이 한 번 끝나면 무음) → head 8~9회(END_OF_DATA 마다 다시 시작). 탐색 범위는 콜백 클립 96종 × 60초다. 60초 안에 소리를 내지 않는 게임과 3분 넘는 곡을 끝까지 기다리는 경우는 이 범위 밖이다.

증적(비커밋 · 게임 바이트 없음 · sha12 와 수치만): `~/scratch/mda-cb/`(`head60.tsv` · `base60.tsv` · `final60.tsv` · 재측 `rb/rh.tsv` · 호출 순서 `seq-*`/`q-*` · 화면 묶음 `sheet-*.png`).

### 6. 시험

`wie-wipi-c/src/api/media.rs` `tests`(0530 의 1개를 포함해 7개):
- `play_tells_the_clip_callback_it_started_test` — 0530 그대로(콜백 없는 클립은 spawn·타이머 0).
- `one_shot_ends_with_end_of_data_test` — 길이 500ms 클립: 타이머 1개, 기한 = 지금 + 1000~1500ms, 돌리면 `[2, 1]`. 그 뒤 stop 은 아무것도 보내지 않는다.
- `stop_sends_stopped_and_voids_end_of_data_test` — STOPPED 는 `MC_mdaStop` 이 돌아온 뒤 간다. 앞 재생의 END 는 취소되고, 두 번째 stop 은 무음이다.
- `repeat_play_has_no_end_of_data_and_replay_voids_the_old_one_test` — 반복 재생은 타이머 0, stop 에 STOPPED. 한 번 재생 두 번이면 END 는 뒤 것 하나.
- `play_inside_its_own_started_is_ignored_test` — STARTED 전달 전의 재생은 spawn·타이머를 늘리지 않는다. 전달 뒤 재생은 다시 시작한다. 가드를 끈 변이로 이 시험이 실패하는 것을 확인했다.
- `play_after_stop_inside_started_starts_again_test` — 콜백 안 stop → play 는 다시 시작한다.
- `freed_clip_gets_no_callback_test` — play → free 뒤 STARTED·END_OF_DATA 0.

`wie-backend` `smaf_duration_covers_the_longest_pcm_tail` — 100ms 에 시작하는 1000ms 스테레오 PCM: duration 1100.

### 7. 게이트

- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(경고는 `wie_cli` 매니페스트 «unused dependency» 9줄 · 기존).
- `RUST_MIN_STACK=4194304 cargo test --all` **809 passed / 0 failed**.
- 러너 블록: `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS · `keydraw_ktf.zip` PASS rc=0(paints 79) · `keydraw_lgt.zip` PASS rc=0(paints 55) · `text_j2me.jar` PASS.

<!-- corpus-name-inflow v1 subjects=5 tree=8aba4ce92f1e5098 B=0/0 P=0/0 S=0/0 -->
