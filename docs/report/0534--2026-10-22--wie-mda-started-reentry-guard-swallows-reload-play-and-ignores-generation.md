## [2026-10-22] STARTED 재진입 가드 좁히기 — 같은 재생만 무시하고, 옛 STARTED 꼬리는 세대를 본다 (wie-mda-started-reentry-guard-swallows-reload-play-and-ignores-generation)

**무엇을**: 0533(#566)의 STARTED 재진입 가드에서 검수가 짚은 두 결함(검수 minor 2·3)을 닫았다. 대상은 `wie-wipi-c/src/api/media.rs`.
- ⑵ 가드는 이제 **같은 재생**만 무시한다. 같은 재생이란 같은 데이터에 같은 반복 설정인 재생이다. `MC_mdaClipPutData` 는 `starting` 을 지운다. `play` 는 직전 재생의 `repeat != 0` 을 새 필드 `MdaClip::repeat` 에 적어 두고, 가드는 이 값까지 맞을 때만 건다.
- ⑶ STARTED 꼬리의 `starting = 0` 에 `clip.generation == self.generation` 조건을 붙였다. END_OF_DATA 쪽과 같은 대조다.

**왜**: 검수 `wie-mda-clip-callback-end-of-data-stopped-and-started-reentry.review.md` minor 2·3.
- ⑵: 가드가 STARTED 대기 창 안이면 콜백 밖에서 온 호출까지 삼켰다. `play(c,0)` → `MC_mdaClipPutData(c, 새 데이터)` → `play(c,…)` 순서면 새 소리가 나지 않았다. `play(c,0)` → `play(c,1)` 순서면 반복 요청이 사라졌다.
- ⑶: 콜백이 STARTED 안에서 stop → play 를 하면 새 STARTED 가 대기열에 선다. 그런데 옛 꼬리가 `starting` 을 지워서 새 STARTED 창에서 가드가 꺼졌다.

**사용자 영향**: 없음(측정 범위 안). 콜백 클립 표본 16종을 60초씩 돌렸을 때 가드가 걸린 횟수는 base·head 모두 0이다. 판정은 16/16 같다(§3).

### 1. 규칙 — «재생 중인 클립을 다시 재생하면 아무것도 하지 않는다»의 «다시»

근거는 0533 §2 와 같은 명세 문장이다(AromaWIPI `org.kwis.msp.media.Player.play`). 이 문장은 **같은 재생을 한 번 더** 부르는 경우를 말한다.
- 새 데이터를 실은 클립을 트는 것은 다른 소리다. 종전 가드는 이때 옛 handle 을 계속 울렸고, 새 데이터는 STARTED 가 전달되기 전까지 틀 수 없었다.
- 반복 설정을 바꿔 다시 트는 것도 다른 재생이다. 0533 이전에는 둘 다 다시 시작했다. 이번 변경으로 그 동작이 돌아온다.

가드를 «콜백 안» 판정으로 좁히는 방법은 쓰지 않았다. 0533 §2 의 «가드의 조건은 콜백 안이 아니라 이미 재생 중»을 유지하기 위해서다. 종전처럼 STARTED 대기 창 안의 같은 재생은 콜백 밖에서 와도 무시한다.

`clip_put_data` 는 `generation` 을 올리지 않는다. 올리면 ⑶ 조건 때문에 옛 꼬리가 `starting` 을 영영 지우지 못한다. 그러면 put 만 하고 재생하지 않은 클립에 가드가 남는다. 옛 재생의 END_OF_DATA 를 취소할지도 이 회차에서 정할 일이 아니어서 손대지 않았다.

### 2. 시험 (RED → GREEN)

수정 전 트리에서 셋 다 실패하는 것을 확인했고(`10 passed; 3 failed`), 수정 뒤 13/13 통과했다.
- `play_after_new_data_while_started_pending_starts_test`: play → putData → play 이면 STARTED 가 2개 spawn 된다.
- `play_with_other_repeat_while_started_pending_starts_test`: play(0) → play(1) 이면 상태가 `[2, 2]` 다. 반복 재생이 앞 재생의 END_OF_DATA 를 취소한다.
- `old_started_return_keeps_the_new_plays_guard_test`: play → stop → play 뒤 옛 STARTED 를 실행해도 다음 같은 재생은 무시된다. spawn 은 STOPPED·새 STARTED 2개 그대로다.

0533 의 7개 시험은 그대로 통과한다(`play_inside_its_own_started_is_ignored_test` · `play_after_stop_inside_started_starts_again_test` 포함).

### 3. 회귀 — base `12acc67a` ↔ head 60초 짝

측정 방식은 0533 §5 와 같다: release `wie_validate` · `--inject` 기본 27키 · 60초 · 동시 ≤ 3 · `build-slot run --long` 임대 1건. 0533 의 `probe.sh`/`run.sh` 를 썼다. 측정 시점 load1 은 6.5~8.4 였다.

표본 16종은 0533 에서 재생 횟수가 달랐던 8종에 END·STOPPED 를 받은 게임 중 재생이 많은 8종을 더했다.
- **판정 16/16 같고, stop 사유도 16/16 같다.** 가드가 걸린 횟수는 base 0 · head 0 이다.
- 재생 횟수가 다른 게임은 5종이다. 그중 4종(`2b1ed0c8d061` `2bafbffeb742` `7e2247bdf565` `f2280c6699a0`)은 0533 §5 에서 이미 흔들림으로 판정한 게임이다. 남은 `0baceb1dfe9c` 는 처음에 base 12 · head 15 였다. 재측하니 base 15·15 · head 15·15 로 흔들림이었다.
- 가드가 base 에서 한 번도 걸리지 않았으므로 두 빌드는 `play` 에서 같은 갈래를 탄다. 이 결과는 그 사실과 맞는다.

증적(비커밋 · sha12 와 수치만): `~/scratch/mda-gen/`(`base.tsv` · `head.tsv` · `rb.tsv` · `rh.tsv`).

### 4. 범위 밖 — 검수 minor 4(`live()` 재할당 오인)

이번에는 손대지 않았다. ⑶ 이후 STARTED 꼬리가 메모리에 쓰려면 `clip_id` 와 `generation` 두 word 가 모두 맞아야 한다. END 꼬리는 원래 그랬다. 해제 뒤 같은 주소가 재할당되고, 첫 word 와 `generation` 오프셋까지 우연히 맞아야 하므로 확률은 종전보다 더 낮다.

### 5. 게이트

- `cargo fmt --all -- --check` OK · `cargo clippy --all -- -D warnings` rc=0 · wasm clippy rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` **812 passed / 0 failed**.
- 러너 블록: `draw_j2me.jar` · `helloworld_ktf.zip` · `helloworld_lgt.zip` PASS · `keydraw_ktf.zip` PASS rc=0(paints 79) · `keydraw_lgt.zip` PASS rc=0(paints 55) · `text_j2me.jar` PASS.

<!-- corpus-name-inflow v1 subjects=2 tree=454471c08986cca0 B=0/0 P=0/0 S=0/0 -->
