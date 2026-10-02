## [2026-10-02] LGT 위 24줄을 비우는 4종 — 내보내는 창을 24줄 내린다 (wie-lgt-top-blank-four-titles-shift-export-window-adopt-p0)

**무엇을**: LGT 의 `MC_grpFlushLcd` 를 LGT 전용으로 감쌌다. 어넌시에이터를 켠 타이틀이 화면 프레임버퍼의 **마지막 여유 줄(343)을 쓰고 위 24줄(0~23)을 비운 채** flush 하면, 그 순간부터 **24~343줄**을 내보낸다(한 번 걸리면 고정). 그 밖의 경우는 종전 공유 경로 그대로다.
**왜**: 0405 §2 — 어넌시에이터 켠 14종 중 4종(`1eaa92092bee` `8f7758fa43b6` `acc4215b7ec0` `fe76e641bb3d`)은 «위 24줄(어넌시에이터) + 알려준 높이»로 그린다. 0~319줄을 내보내면 위에 검은 띠가 생기고 맨 아래 24줄(상태줄·소프트키 안내)이 잘린다. 0405 §3 대로 보고 높이(296)는 건드리지 않았다.
**사용자 영향**: 그 4종에서 잘려 있던 하단 줄이 보이고 위 검은 띠가 사라진다. 다른 LGT 타이틀은 그대로다(§2).

증적: `~/orchestrator/reports/evidence/wie-lgt-top-blank-four-titles-shift-export-window-adopt-p0/`. 타이틀은 sha12 로만 적는다.

### 1. 고정(래치) vs 프레임마다 — 짝 재측

탐침 빌드(`probe-variant.txt` · 같은 판별식을 매 화면 flush 마다 계산해 기록 · 래치 동작은 그대로) · 어넌시에이터 켠 14종 · `--inject --keep-timeout --timeout 60 --pacing 8 --relaunch 1 --max-ticks 1e11` · jobs 3 · 시작 guard rc=0(`hl-start.txt` · `probe.jsonl`).

| sha12 | flush | 걸림 | 첫 걸림 | 첫 걸림 뒤 안 걸린 flush | 연속 구간(앞부분) |
|---|---|---|---|---|---|
| `8f7758fa43b6` | 501 | 501 | 0 | 0 | 걸림 501 |
| `acc4215b7ec0` | 466 | 426 | 25 | 15 | 0×25 · 1×147 · 0×5 · 1×12 · 0×5 · 1×14 · 0×5 · 1×253 |
| `1eaa92092bee` | 1038 | 788 | 196 | 54 | 0×196 · 1×34 · 0×2 · 1×34 · 0×52 · 1×720 |
| `fe76e641bb3d` | 149 | 37 | 0 | 112 | 1×37 · 0×112(20번째 키 SVC 603 FAIL 까지) |
| 나머지 10종 | 26~4,239 | **0** | — | — | 전부 0 |

- **프레임마다 판단하면 4종 중 3종의 화면이 24줄씩 오르내린다** — 첫 걸림 뒤에도 5~112 flush 씩 판별식이 빗나가는 구간이 있다(위가 칠해지거나 마지막 여유 줄이 비는 화면). 그래서 **고정**을 택했다.
- **고정의 오판 위험**: 나머지 10종은 한 번도 걸리지 않았다 ⇒ 이 10종에서는 래치가 서지 않고 코드 경로가 main 과 같다. 어넌시에이터를 끈 64종은 판별 자체를 하지 않는다. 판별식이 «마지막 여유 줄 기록»을 요구하므로, 검은 전환 화면만으로는(여유 줄을 쓰지 않으니) 걸리지 않는다.
- **고정의 대가**(숨기지 않는다): `1eaa92092bee` 는 첫 걸림 뒤 공지 화면에서 0~23줄에도 그린다(52 flush 구간). 고정된 창은 그 위 띠를 감추고 아래 24줄을 보인다(`side/1eaa92092bee-strip.png` 첫 칸). 프레임마다였다면 그 구간과 앞뒤가 서로 다른 위치로 번갈아 보인다.

### 2. main ↔ 수정 짝 재측 — 판정과 화면

같은 14종 · main(`origin/main` `5cbd635e`) ↔ 수정(`fbfdee4b`) · 같은 시각 동시 · 2판 · jobs 3 · 5초 간격 화면(`pair.jsonl`).
- **판정 56/56 이 main = 수정이다.** PASS 27/27 12종 ×2 · `fe76e641bb3d` FAIL 20번째 키 SVC 603 ×2 · `63332c51d514` UNMEASURED 2/27 clean exit ×2 — 0405 §1 과 같은 기존 판정이다.
- **대상 4종 화면**(`side/<sha12>.png` · 왼쪽 main · 오른쪽 수정 · 같은 55초 시점 · `fe76e641bb3d` 는 10초): main 의 위 24줄 비검정 화소는 7/8 판에서 **0**(나머지 1판 792)이다. 수정본은 그 띠가 없고 맨 아래 줄이 보인다 — `acc4215b7ec0` 하단 안내 줄 · `8f7758fa43b6` 하단 소프트키 안내 · `1eaa92092bee` 하단 HUD(`-strip.png` 10~40초) · `fe76e641bb3d` 위 검은 띠 소거.
- **비대상 10종**: 판정이 같다. 같은 시점 화면은 타이밍 차로 다를 수 있어 화면 비교 대신 §1 의 «걸림 0» 으로 무변화를 판정한다(래치가 서지 않으면 수정 코드는 종전 `flush_lcd` 를 그대로 부른다).
- `1eaa92092bee` 의 장면 자체(마젠타·격자 무늬)는 main 과 수정 양쪽에 같다 — 이 회차와 무관한 기존 그리기 결함이다.

**호스트 부하**: 시작 guard rc=0 · **끝 guard rc=1**(10:01 재포화 · load 242 · `hl-end.txt`). §1 탐침은 해제 구간 안에 끝났다. §2 는 끝 무렵 포화와 겹쳤을 수 있으나 짝이 같은 시각에 돌아 부하가 대칭이고, 키 전달 수가 56판 전부 main 과 같다.

### 3. 게이트

`origin/main` `5cbd635e` 위 · 이 worktree 의 target · build-slot 경유.
- `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings`(stable · beta) · wasm32 clippy — 전부 rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 51 묶음 · 639 통과 · 실패 0. 새 시험 `flush_lcd_latches_window_below_blank_annunciator_rows`(FlushLcd SVC 배선 · 어넌시에이터 끔 · 위 줄 칠함 · 래치 후 유지).
- 러너 블록(수정 트리 release `wie_validate`): `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me` PASS. `keydraw_ktf/lgt` 는 기본 `--max-ticks` 에서 UNMEASURED(stop=max-ticks) — **손대지 않은 main 도 같다**. `--max-ticks 1e11` 로 둘 다 PASS · rc=0(LGT paints 55 = main 55).
- 유입(`node scripts/corpus-name-inflow.mjs`): **BOUNDED 12회/7쌍 · SUFFIX-ATTACHED 0회/0쌍.** BOUNDED 12회는 전부 이 회차가 손댄 `wie-lgt/src/runtime/wipi_c.rs`·`wie-wipi-c/src/api/graphics.rs` 의 **기존 주석**이다. 이 회차가 더한 줄(`git diff origin/main...HEAD` 의 `+` 줄)의 게임명은 0 — 타이틀은 sha12 · 업데이트 소식은 sha256 으로만 적었다.


