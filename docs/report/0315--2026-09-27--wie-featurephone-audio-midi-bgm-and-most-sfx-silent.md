## [2026-09-27] featurephone 소리 — MIDI 합성(오디오 워클릿) · repeat 루프 · Stop · SKT 클립 배선 · 배틀몬스터 BGM 루프 (wie-featurephone-audio-midi-bgm-and-most-sfx-silent)

**무엇을**
- `wie_featurephone/src/audio.rs` + `audio_worklet.js`: 모든 `AudioCommand` 를 AudioWorklet(오디오 스레드)로 보낸다. 워클릿이 MIDI 를 합성하고(GM 패밀리별 2-op FM + 10번 채널 드럼) PCM 을 재생하며, 핸들별로 `repeat` 루프 · `Stop` 즉시 정지 · 핸들 간 믹스를 한다. 워클릿 소스는 wasm 안의 문자열 → Blob URL 로 싣는다(산출물 2파일 계약 불변). 워클릿을 못 쓰는 브라우저는 종전 PCM 경로로 폴백.
- `wie-skvm` `net.wie.WieAudioClip`: 전 메서드 스텁 → `Audio::load_smaf`/`play`/`stop` 배선. `close` 는 소리를 끊지 않는다(아래).
- `wie-midp` `SmafPlayer`: `start()` 가 루프 모드를 유지 · `setLoopCount` 반영. `wie-wipi-java` `Player.resume(clip)`: 재생 중인 클립은 다시 틀지 않는다.
- `wie_validate` 결과 줄에 `audio` 키(plays·repeat_plays·stops·wave_events·midi_events·empty_plays) — 보고만, 판정 무관.
- `scripts/check-audio-worklet.mjs`(node · 약 0.5초) — `engine-contract.yml` 상시 단계.

**왜**: 운영자 실플레이 보고 「일부 효과음만 나오고 대부분의 효과음과 배경음악은 안 나온다」.

**사용자 영향**: KTF·LGT·SKT 게임에서 배경음악과 MIDI 효과음이 나고, 배경음악이 반복되며, 장면 전환 때 끊긴다. 속도 변화 없음.

### 원인 — 셋이 겹쳐 있었다

| # | 자리 | 증상 | 측정 |
|---|---|---|---|
| ① | 호스트 `WebAudioSink` | `Wave` 만 재생 · MIDI·`repeat`·`Stop` 버림 | 배틀몬스터 이벤트 272 중 **MIDI 258(95%)** |
| ② | SKT `WieAudioClip` | 전 메서드 스텁 → 엔진이 아무것도 안 보냄 | SKT 7타이틀 `plays` **0** · 스텁 적중 open/play/loop/close |
| ③ | WIPI Java `Player.resume` | `play(clip,true)` 직후 `resume` → `start()` = `repeat=false` 재생 | 브라우저 타임라인: `play R` → `stop` → `play`(같은 순간) |

- SKT 클립 형식: 5타이틀 **41건 전부 `MMMD`(SMAF)** · `getAudioClip("mmf")` ⇒ 기존 `load_smaf` 로 충분.
- 더팜1 은 효과음마다 `open → play → close` 를 부른다. `close` 에서 멈추면 전부 무음(브라우저 RMS **0**, 10 play 모두 같은 순간 stop). 실기에서 소리가 나므로 `close` 는 «핸들 은퇴»만 하고, 다음 `open` 이 해제(정지)한다 — 클립당 핸들 1개로 누적 상한.

### 계측표 — 엔진이 보내는 것 (`wie_validate --inject` · 최종 브랜치)

| 플랫폼 | 타이틀 | plays | repeat | stops | Wave | MIDI |
|---|---|---|---|---|---|---|
| LGT(aot-java · `org/kwis/msp/media`) | 배틀몬스터 | 2 | 1 | 1 | 7 | 897 |
| KTF | 2007루노베이스볼 | 2 | 1 | 1 | 0 | 1444 |
| KTF | 마린블루스 | 2 | 0 | 1 | 1 | 1483 |
| KTF | 간호사타이쿤 | 1 | 0 | 0 | 0 | 58 |
| SKT | 더팜1 | 8 | 0 | 7 | 7 | 782 |
| SKT | 고래사냥2 | 4 | 1 | 3 | 0 | 1228 |
| SKT | 아슬아슬타워쿤 | 3 | 1 | 2 | 2 | 988 |
| SKT | 웰루시아 | 1 | 0 | 0 | 0 | 78 |

- 수정 전 SKT 는 같은 타이틀들이 전건 **0/0/0/0/0**. KTF 영웅서기4 는 헤드리스에서 FAIL(paints 0 · 별건) — 브라우저에서는 MIDI 9,298 전달.
- MIDP(J2ME): 로컬 코퍼스에 J2ME 타이틀 **0** — 타이틀 계측 불가. 경로(`SmafPlayer`)는 LGT/KTF-Java 와 공유하므로 ③의 수정이 같이 걸린다.
- LGT 소리 경로 판정: 배틀몬스터는 WIPI-C `MC_mda*` 가 아니라 **Java `org.kwis.msp.media.Clip/Player` → `net.wie.SmafPlayer` → `Audio`** 로 AudioSink 까지 닿는다(끊긴 곳 없음 · 잘못된 것은 ③).

### 설계 결정

- ⒜ **사운드폰트를 쓰지 않았다.** upstream 은 `spessasynth_lib` + `GeneralUser.sf3`(**10,580,436 B** · INFO `ICMT` 「License v2.0 … use it in your software projects」 = 재배포 가능). 그러나 featurephone 산출물은 **2파일 계약**(`wie_web.js`·`wie_web_bg.wasm`)이고 셸은 글루를 번들러 없이 ES 모듈로 불러 bare specifier(`spessasynth_lib`)를 풀 수 없다 ⇒ 셸 자산 3종(처리기 JS·라이브러리·sf3) 서빙 + 계약 변경이 필요하다. 이번 회차는 **엔진 산출물 안에서 끝나는 경로**를 택했다: 자체 합성기(외부 코드·자산 0 · 라이선스 문제 0 · 추가 다운로드 0). 대가 = 음색이 사운드폰트보다 단순하다(원래 SMAF 도 FM 칩 음원). 업그레이드 경로는 worklog 제안.
- ⒝ `repeat`: 시퀀스 길이(= max(duration, PCM 끝))마다 처음으로. `Stop`: 그 핸들의 음표·PCM 을 15ms 릴리스로 끊는다. 핸들마다 MIDI 채널 상태가 따로라 BGM·SFX 가 서로의 음색/볼륨을 건드리지 않는다. 동시 발음 48 상한.
- ⒞ 자동재생: 셸이 쥔 `AudioContext` resume 규칙 그대로(워클릿 모듈은 suspended 상태에서도 로드된다). 출력은 셸의 마스터 GainNode 로 → **볼륨·음소거가 MIDI 에도 적용**. WebKit(Playwright `webkit`)에서 워클릿 로드·합성 실측 확인(아래). iOS 실기는 미확인.
- ⒟ 동기: 시퀀스 내부 타이밍은 오디오 스레드 샘플 단위(128프레임 블록)로 진행 — 에뮬레이션 tick 속도와 무관. Play 메시지 도착 즉시 시작.
- 메인 스레드: 핸들당 첫 Play 에서만 이벤트 배열을 만든다(재생은 핸들 번호만). 합성 0.

### 전/후 — 브라우저 오디오 출력 (headless Chromium · AudioContext → 마스터 게인 → AnalyserNode · 초당 RMS)

| 타이틀 | main | 이 브랜치 |
|---|---|---|
| 배틀몬스터(키 9개 · 24s ×2 교대) | s6–10 PCM 만(0.29·0.14·0.30…) 이후 **0.000** | 후반 평균 RMS **0.060 / 0.060** · 워클릿 전달 MIDI 1,331 |
| 배틀몬스터 마을 경로(`docs/keys/battlemonster-village.keys` · 700s) | — | play 19 · stop 18 · repeat 9 · MIDI 6,385 · BGM 연속(6.4초 곡이 204초 루프 = 약 32회) · 장면 전환마다 Stop 후 다음 곡 |
| 간호사타이쿤(KTF · 30s) | 전 구간 **0.000** | 후반 평균 **0.067** |
| 2007루노베이스볼(KTF) | s22–24 PCM 외 0 | 후반 평균 **0.054** · 비반복 징글(6.2초) 끝나면 무음(정상) |
| 영웅서기4(KTF · 40s ×2) | **0.000** | 0.024–0.057(볼륨 상향 전) · MIDI 9,298 |
| 더팜1(SKT) | **0.000** | BGM 0.06 + 효과음 겹침 0.10–0.11 |
| 고래사냥2(SKT) | **0.000** | 0.015–0.069 |

- 루프: 배틀몬스터 BGM(13.3초 곡) — 수정 후 `play R` 1회뿐(③ 해소)이고 s12→s41 연속 = 곡 길이의 2배 이상.
- 겹침: 배틀몬스터는 매번 이전 핸들을 Stop 한 뒤 다음을 튼다(게임 자체가 한 채널) — 겹침은 더팜1 과 `check-audio-worklet` ④에서 확인.
- WebKit: 간호사타이쿤 15s — Chromium 과 타임라인·RMS 동일(0.076·0.109·0.116·0.123 … 게임이 4.14s 에 Stop).
- 볼륨: MIDI 음성 게인 0.16 → 0.28 로 올린 뒤 배틀몬스터 BGM RMS 0.10–0.12(PCM 효과음 0.2–0.3 보다 아래).

### 속도 퇴행 0 (같은 조건 교대 · 캔버스 변화 프레임/초 · rAF 구동)

| 타이틀 | 구간 | main fps | 이 브랜치 fps | ms/tick main → 이 브랜치 |
|---|---|---|---|---|
| 배틀몬스터 마을 경로 ×2 | s45–100 | 11.56 · 11.56 | 11.52 · 11.56 | 1.91 · 1.91 → 1.90 · 1.94 |
| 〃 | s100–165 | 12.43 · 12.31 | 12.00 · 12.75 | 9.65 · 9.67 → 9.45 · 10.04 |
| 영웅서기4 ×2 | 초당 | 52 45 · 59 60 45 | 52 46 · 60 60 45 | 10.82 · 10.56 → 10.58 · 10.65 |

- loadavg 55–230. 이 fps 는 프로브가 매 rAF 캔버스를 읽어 센 값이라 0305 의 «20fps»(blit 계수)와 자가 다르다 — 판정은 **같은 자 교대 비교**.

### 되돌리면 red

| 되돌림 | 결과 |
|---|---|
| 워클릿 MIDI note-on 무시 | check-audio-worklet 4 FAIL |
| `repeat` 무시 | 1 FAIL |
| `stop` 무시 | 2 FAIL |
| 재생 캐시 제거 | 1 FAIL |
| `SmafPlayer.start()` → repeat=false | `test_clip_resume_keeps_playing_clip_and_loop_mode` FAILED |
| `resume` 이 재생 중 클립을 다시 틈 | 같은 시험 FAILED |
| SKT `close` 가 소리를 끊음 | `audio_clip_drives_the_audio_sink` FAILED |

### 회귀

`cargo fmt --check` · `clippy --all -D warnings` · wasm32 clippy · `+beta clippy` rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` **499 passed / 0 failed** · runner 블록 전건 PASS(keydraw 2종 `--expect-last-frame` rc 0 · paints 55) · `npm run build:wasm` · `check-engine-contract` **109 pass / 0 violation** · 글루 27,215 → **35,151 B**(상한 40,000 · 여유 12%) · wasm 15,510,310 → 15,557,806 B.

### 한계

- 음색은 합성기 근사다(사운드폰트 아님). SMAF 의 FM 음색 파라미터(SysEx)는 해석하지 않는다.
- 볼륨 API(`Clip.setVolume`·`AudioSystem.setVolume`·`MC_mdaSetVolume`)는 여전히 스텁 — 게임이 정한 음량은 무시된다. `pause`/`resume` 은 위치를 모른다(처음부터).
- iOS Safari 실기·모바일 성능은 재지 않았다(WebKit 데스크톱 엔진만).
- 셸 문서(otterpebble `apps/featurephone/CLAUDE.md` 「MIDI 는 엔진 측 무음 스텁」)는 핀 범프 뒤 낡는다 — 코드 변경은 불요.
