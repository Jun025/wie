## [2026-09-28] MIDP PlayerListener END_OF_MEDIA — 곡이 끝나면 리스너에 통지 (wie-2026-09-27-featurephone-midp-playerlistener-end-of-media-adopt-p1)

**무엇을**: `net/wie/SmafPlayer` 의 `addPlayerListener`·`removePlayerListener` 스텁을 구현했다(리스너는 `java/util/Vector` 필드에 둔다 · null 무시 · 중복 무시).
`start` 는 리스너가 **하나라도 있을 때만** `startedAt + lengthMs`(#352 가 만든 두 필드)에 타이머를 건다. 타이머가 울리면:
- 한 번 재생이면 `END_OF_MEDIA` 를 1회 통지하고 PREFETCHED 로 돌아간다.
- 반복 재생이면 한 바퀴마다 통지하고 다시 건다. MIDP `setLoopCount` 규약이 «반복마다 END_OF_MEDIA» 다.
- 이벤트 데이터는 `Long(lengthMs)` 다.

`start`·`stop`·`close` 는 `generation` 을 올리므로, 앞선 시작이 건 타이머는 무효가 된다. 멈추거나 닫은 뒤에는 통지하지 않는다.

**왜**: 곡이 끝나면 다음 곡을 트는 게임이 그 이벤트를 받지 못했다(worklog `2026-09-27-featurephone-audio-loop-overlap-and-worklet-sequence-leak#p1`).

**사용자 영향**: 지금 보유 게임에서는 **없다**. 로컬 코퍼스 전수(`game_lab/working`·`broken`, jar 안의 jar 포함)에서 문자열 `addPlayerListener` 가 들어 있는 파일은 **0개**다.
`PlayerListener` 가 걸린 3개(`5028b8a5d19f` `b7699c10dfd1` `1cf2e6076079`)는 네이티브 이미지의 클래스 이름표 `javax_microedition_media_PlayerListener` 뿐이고, 호출은 없다.
리스너가 없는 플레이어는 타이머를 걸지 않으므로 기존 동작은 바이트 단위로 같다. 효과는 단위 시험으로만 잠갔다(**관측 타이틀 0**).

### 시험 — `test_end_of_media_reaches_listeners`(TestClock)
- 리스너가 없으면 타이머 0
- 한 번 재생: 499ms 통지 0 → 500ms 통지 1(`endOfMedia`) → `getState` = 300 → 그 뒤 0
- 끝나기 전에 `stop`·`close`: 타이머는 울리지만 통지 0
- 반복 재생: 500ms마다 1회씩 3바퀴 → `stop` 뒤 0

**변이** — 셋 다 red:
- generation 검사 제거(멈춘 뒤 통지)
- 반복 재무장 제거
- 리스너 조건부 타이머 제거(통지 없음)

### 하지 않은 것
- `STARTED`·`STOPPED`·`CLOSED` 통지. 명세에는 있지만 부르는 타이틀이 0 이라 과구현이다.
- `getMediaTime` 은 계속 -1 이다.
