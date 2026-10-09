## [2026-10-10] 계약 게이트 S5 «held at first look» 레이스 — 누적 `holds` 로 결정적으로 (wie-contract-gate-s5-heldfirst-race-flake)

### 무엇을
- 워클릿 `stats` 에 `holds`(지금까지 붙잡힌 Play 수 · 누적)를 더했다(`wie_featurephone/src/audio_worklet.js` — `play()` 가 `held.set` 할 때 +1).
- `scripts/contract-roundtrip.mjs` S5 의 «붙잡혔다» 항을 `heldFirst === 1`(첫 폴링 시점의 `held`) → `holds` 증가분 `=== 1` 로 바꿨다. 나머지 항(FM 0 · synths ≥ 1 · rms)은 그대로. `held at first look` 은 상세 문구에만 남겼다.

### 왜
- publish run 37948035765(`d92edc49`)이 S5 1건으로 red: `held at first look 0 · FM voices seen 0 · rms 0.0473` — 엔진은 옳았다(같은 커밋 engine-contract run 37948035686 은 `held 1` 로 통과).
- 근인: 아무것도 울리지 않을 때 워클릿은 해독 작업을 쉬지 않고 이어 돌린다(SILENT_WORK_MS, `ae5fafc2`). 그래서 해독이 첫 `stats` 왕복보다 먼저 끝날 수 있고, 그때 `held` 는 0 이다 — 관측 타이밍을 단언한 것이지 엔진을 단언한 것이 아니다.
- `heldFirst` 를 그냥 빼면 안 된다(돌연변이 아래): 해독을 기다리지 않는 회귀는 FM 0 · 사운드폰트 소리 · rms 정상으로 지나간다 — 붙잡힘 항만 그것을 잡는다. 그래서 «빼기»가 아니라 «타이밍 무관한 관측»을 골랐다.

### 실측 (로컬 · load1 ≈ 24)
| | 결과 |
|---|---|
| 수정 전, 같은 커밋 20회 | 20/20 통과(재현 0 — 로컬 오디오 스레드가 CI 러너보다 늦게 해독을 끝낸다) |
| 수정 전, play 직후 500 ms 대기 주입(강제 레이스) | 5/5 실패 · `held at first look 0` — CI 서명과 동일 |
| 수정 후, 같은 커밋 20회 | 20/20 통과 (67/67) |
| 수정 후, 500 ms 대기 주입 | 15/15 통과 · `held 1 (at first look 0)` |
| 돌연변이: `mustWait` 가 해독 여부를 무시(`return !this.synthAvailable()`) | 4/4 실패 · `held 0` · FM 0 · rms 0.04 — 붙잡힘 항만이 잡는다 |

`check-audio-worklet.mjs --require-soundfont` OK · `check-engine-contract.mjs` OK · 4게이트 + beta clippy rc0 (`cargo test --all` 54 suites 769 passed).

### 사용자 영향
없음 — `stats` 는 측정용 표면이다(엔진 안에서 묻는 곳 없음). 계약 게이트의 거짓 red 로 엔진 artifact·Release 가 막히는 일이 줄어든다.

### 인접
`wie-featurephone-bgm-first-play-fm-vs-repeat-soundfont-inconsistent`(S5 불변식 개정)는 이미 착지(`e8532c55`·`ae5fafc2`)했고 열린 PR 이 없다 — 이 회차는 그 불변식의 본뜻을 바꾸지 않고 관측 축만 바꿨다.
