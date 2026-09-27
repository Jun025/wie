## [2026-09-27] 게임이 정한 음량 반영 — Clip/Volume/MC_mda/AudioSystem 볼륨 (wie-2026-09-27-featurephone-game-volume-api-adopt-p1)

**무엇을**: 게임이 부르는 볼륨 API 가 소리 출력까지 닿는다. WIPI Java `Clip.setVolume`·`Volume.set/get`, WIPI-C `MC_mdaClipSetVolume/GetVolume`·`MC_mdaSetVolume/GetVolume`, SKT `AudioSystem.setVolume/getVolume/getMaxVolume` 가 백엔드 `Audio` 의 핸들별 볼륨 × 마스터 볼륨이 되고, 워클릿이 그 게인을 재생마다 곱한다. 셸의 마스터 GainNode(UI 슬라이더·음소거)는 워클릿 출력 뒤에 있으므로 곱셈 합성은 배선만으로 성립한다.
**왜**: 추천 `2026-09-27-featurephone-audio-midi-bgm#p1` — 전부 스텁이라 게임 설정의 음량·소리 끔이 먹지 않았다.
**사용자 영향**: 게임이 정한 음량으로 들린다. ★대부분의 타이틀이 **지금보다 작아진다** — 게임들이 기본값으로 40~60% 를 고른다(아래 실측). 게임 설정에서 소리를 끈 세이브는 무음이 된다.

### 1. 싼 경로 판정 — `AudioCommand` 는 건드리지 않았다

`AudioSink` 에 **기본 구현이 있는 메서드** `set_gain(handle, gain)` 하나를 더했다(`wie-backend`). 변형(variant)을 더하면 `AudioCommand` 를 완전 매칭하는 싱크(`wie-web`·`wie_validate`·`test-utils`)가 전부 깨지지만, 기본 no-op 메서드는 그 셋을 **한 줄도 바꾸지 않는다**. upstream 공유 크레이트 접촉은 `wie-backend` 의 **추가만**(시그니처 변경 0)이다.

- 게스트 API 크레이트(`wie-wipi-java`·`wie-wipi-c`·`wie-skvm`)와 싱크(`wie_featurephone`)가 공유하는 것은 `wie-backend` 뿐이라, 백엔드 무접촉 경로는 없다.
- 계약: `Audio` 가 **모든 Play 직전**에 그 핸들의 게인을 보내고, 재생 중 볼륨이 바뀌면 다시 보낸다 ⇒ 싱크는 게인을 «다음 Play 까지»만 들고 있으면 된다. 워클릿의 대기 게인 맵이 무한히 자라지 않는 이유가 이것이다(play 가 소비 · evict 가 지움).
- 워클릿 음성은 재생 객체(`pb`)를 참조로 쥔다 — 정지된 핸들의 릴리스 꼬리는 마지막 게인을 유지하고 1 로 튀지 않는다.
- 폴백(워클릿 불가) 경로는 게인을 무시한다 — 이미 MIDI 무음인 경로라 범위 밖.

### 2. 인자 범위 — 호출부 실측(로컬 전 코퍼스 294타이틀 · `wie_validate --inject --timeout 40`)

| API | 호출 타이틀 | 관측 값 | 채택 범위 |
|---|---|---|---|
| WIPI Java `Clip.setVolume` | KTF 85 | 0~100 (40·60·50 최다) | 0..100 |
| WIPI Java `Volume.set` | KTF 28 | 20~80 | 0..100 (마스터) |
| WIPI-C `MC_mdaClipSetVolume` | KTF 20 · LGT 30 | 0~100 (60·50·40 최다) | 0..100 |
| WIPI-C `MC_mdaSetVolume` | 0 | — | 0..100 (마스터 · `ClipSetVolume` 과 같은 척도) |
| SKT `AudioSystem.setVolume` | SKT 30 | 스텁 max=0 아래 0 ×82 | 0..5 (마스터) |

- SKT 최대값 5 는 KEmulator(`vendor_sdk` 의 `KEmulator-mmpp.jar`) `com/skt/m/AudioSystem.getMaxVolume` 바이트코드가 `iconst_5` 인 것에서 가져왔다. 스텁이 0 을 답하던 동안 **21/30 타이틀이 그것을 읽고 0 을 설정**했다 — max 를 5 로 고치지 않고 setVolume 만 살렸으면 SKT 가 거의 전부 무음이 됐다. 고친 뒤 21 중 **20** 이 1~5 를 고른다(영웅서기2 만 0 유지). 최대를 넘는 값(노리타이쿤 50 · 사고뭉치트윈스 15)은 최대로 본다.
- ★같은 함정이 WIPI-C 에도 있었다: **13 타이틀**(컴투스 11 + 검은방3 2)이 `MC_mdaClipGetVolume` 을 읽고 **그 값을 그대로** `MC_mdaClipSetVolume` 한다. 스텁 게터가 0 이라 전부 0 을 설정하고 있었다 → 게터가 저장값(기본 100)을 답하게 한 뒤 13/13 이 **100** 을 설정한다(전/후 스캔 대조).
- WIPI Java `Clip.getVolume` 의 기본값도 필드 기본 0 → **100** 으로 바꿨다(같은 형태 예방 · 기존 시험 `test_volume_range_round_trip` 의 기대값 0 → 100 갱신).
- 선형 매핑(`level / max` = 진폭 배수). JSR-135 VolumeControl 도 «선형 배수로의 매핑은 구현 의존»이라 적는다 — dB 곡선은 낮은 값을 더 작게 만들 뿐이라 택하지 않았다.
- `MdaClip` 끝에 내부 필드 `volume`·`loaded` 를 붙이고 `MC_mdaClipCreate` 가 구조체를 명시 초기화한다(할당기가 0 을 보장하지 않는다). `loaded` 는 데이터 없는 클립의 볼륨이 **핸들 0(처음 로드된 다른 클립)** 에 닿지 않게 한다.

### 3. 전/후 — 브라우저 출력 RMS

headless Chromium · 실제 `AudioContext → 마스터 GainNode → AnalyserNode` · 키 700ms 순환 · **기준 = #352 머지 기점 `2aafcf2b` wasm** / 이 브랜치 wasm 을 **동시 실행** · 40s. 스크래치 프로브(커밋 안 함 · 게임 파일은 저장소 밖).

| 타이틀 | 게임이 부른 것 | 워클릿 게인 | 기준 RMS | 이 브랜치 RMS | 배수 |
|---|---|---|---|---|---|
| SKT 미니동화TING | `setVolume(mmf, 3)` | 0.6 | 0.1269 | 0.0813 | ×0.64 |
| KTF 간호사타이쿤 | `Clip.setVolume(20)` | 0.2 | 0.0109 | 0.0024 | ×0.22 |
| LGT 배틀몬스터 | `Volume.set(60)` × `Clip.setVolume(60)` | 0.36 | 0.0939 | 0.0344 | ×0.37 |
| SKT 더팜1(회귀) | `setVolume(mmf, 5)` (전엔 0) | 1.0 | 0.1296 | 0.1292 | ×1.00 |
| LGT 검은방3(회귀) | GetVolume → Set 100 (전엔 0 뒤 100) | 1.0 | 0.0041 | 0.0038 | ≈1 |

**게임 안 설정 변경 → RMS** (LGT 데몬헌터 · 같은 빌드 · 세이브 `P/Config.dat` 첫 바이트 = 게임의 음량 단계). 그 바이트만 바꾼 스크래치 사본 4개를 동시 실행, BGM 최고 초 RMS:

| 설정 단계 | 게임이 부른 것 | RMS |
|---|---|---|
| 0 (동봉 세이브 그대로) | `MC_mdaClipSetVolume(clip, 0)` | **0** |
| 1 | `…, 20` | 0.030 |
| 3 | `…, 60` | 0.087 |
| 5 | `…, 100` | 0.153 (기준 빌드 0.151) |

- 단계 → 호출 값의 사상(0/20/60/100)은 `wie_validate` 로그에서 직접 읽었다. 1/5 : 3/5 : 5/5 = 0.20 : 0.57 : 1.
- ★데몬헌터는 **동봉 세이브 그대로면 무음**이 된다(기준 빌드에선 BGM 이 들렸다). 원인은 이 세이브의 전 소유자가 음량 0 으로 둔 것 — 게임 설정이 먹는다는 뜻이고 결함이 아니다. 같은 모양(게임이 스스로 0 을 설정)은 전 코퍼스에서 **8 타이틀**: KTF 점핑펭·페이블오브나이트(`Clip.setVolume(0)` — 전에도 0) · LGT 데몬헌터·리듬페스티발 2종·창세기전3ep1 · SKT 영웅서기2·삼국쟁패2열왕전기(주입 키로 설정 화면에서 바뀜). 데몬헌터 외에는 출처를 추적하지 않았다.

### 되돌리면 red

| 되돌림 | 결과 |
|---|---|
| `Audio::play` 가 Play 전 게인을 보내지 않음 | `volume_reaches_the_sink_before_every_play_and_while_playing` FAILED |
| `MC_mdaClipGetVolume` 이 다시 0 | `clip_and_master_volume_reach_the_audio_handle_test` FAILED |
| `loaded` 가드 제거(데이터 없는 클립이 핸들 0 에 닿음) | 같은 시험 FAILED |
| `Player.play` 에서 클립 볼륨 적용 제거 | `test_clip_and_handset_volume_reach_the_audio_handle` FAILED |
| `getMaxVolume` 이 다시 0 | `audio_system_volume_is_zero_to_five_and_sets_the_master` FAILED |
| 워클릿 MIDI 음성 게인 제거 · PCM 게인 제거 · 대기 게인을 play 가 소비하지 않음 | check-audio-worklet ⑩ FAIL (각각 midi ×1.000 · pcm ×1.000 · replay/full ×0.510) |

### 회귀

코퍼스 전/후 `wie_validate` 판정 차이 18건(294 중): PASS↔UNMEASURED 13 · FAIL→PASS 1 · UNMEASURED→FAIL 1 · PASS→FAIL 3. UNMEASURED 는 주입 키 0(부하 · loadavg 180~250). FAIL 4건은 다시 쟀다 — 미니게임의달인은 재실행 2/2 PASS, 던전앤히어로·무한의통통·삼국쟁패2 는 **기준 바이너리(`2aafcf2b`)에서도 FAIL**(AGENTS.md ⑶). 볼륨 경로는 렌더·입력에 닿지 않는다. 게이트 수치는 회신 정본(`~/orchestrator/reports/…done.md`)에 적는다.

### 한계

- `Volume.setMute`·`MC_mdaSetMuteState` 는 여전히 스텁 — 관측된 호출(리듬스타1 `(6, 1)` · 검은방3 `(0, 1)`)의 source 인자 의미를 모른 채 살리면 소리를 끌 수 있다.
- `wie_featurephone/src/audio.rs` 의 `set_gain` 은 네이티브 시험이 없다(wasm32 전용 크레이트 · Constraint 7) — 위 브라우저 표의 워클릿 게인 값(0.200/0.600/0.360/1.000 — 포트 메시지 가로채기로 읽음)이 판정한다.

게임 파일명 유입(`corpus-name-inflow --corpus ~/work/otterpebble/wie/game_lab`): BOUNDED 34회/27쌍 · SUFFIX-ATTACHED 3회/3쌍. BOUNDED 중 29회는 이 회차가 쓴 줄(이 문서 · worklog · `audio_system.rs` 주석 2)이고 5회는 수정 파일의 기존 줄(`player.rs` 3 · `audio.rs` 2 — 이 회차가 쓴 줄 아님). SUFFIX-ATTACHED 3회는 조사가 붙은 진짜 언급 2(이 문서) + 더 긴 다른 제목 속 부분 일치 1.

<!-- corpus-name-inflow v1 subjects=12 tree=026ebc4c5397ec49 B=34/27 P=0/0 S=3/3 -->
