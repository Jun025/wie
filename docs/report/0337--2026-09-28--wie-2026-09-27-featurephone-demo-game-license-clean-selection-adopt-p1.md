## [2026-09-28] Manager.createPlayer(String) — 톤 장치 로케이터에 무음 플레이어 (wie-2026-09-27-featurephone-demo-game-license-clean-selection-adopt-p1)

**무엇을**: `javax/microedition/media/Manager` 에 `createPlayer(String locator)` 를 더했다.
- `"device://tone"`(`Manager.TONE_DEVICE_LOCATOR`) → 빈 데이터의 `net/wie/SmafPlayer`. `realize`·`prefetch`·`start`·`stop`·`close` 는 기존 SMAF 플레이어 그대로 성공하고, 소리는 없다.
- `null` → `IllegalArgumentException`, 그 밖의 로케이터 → `MediaException`(MIDP 명세의 두 실패).
- `getControl("ToneControl")` 은 `null` 이다. 톤 시퀀스 합성은 범위 밖(티켓 경계).

**왜**: 재배포 가능한 데모 후보 WormGame(Sun WTK · BSD-3)이 확인 키에서 `NoSuchMethodError` 로 죽었다(worklog `2026-09-27-featurephone-demo-game-license-clean-selection#p1`, `docs/report/0327` 표).

**사용자 영향**: 보유 코퍼스에는 **없다** — `game_lab/working`·`broken` 464개(jar 안의 jar 포함)에서 `device://tone`·`(Ljava/lang/String;)Ljavax/microedition/media/Player;` 문자열 **0개**. 그래서 `docs/player-updates/` 항목도 없다. 효과는 WormGame(repo 밖 `/tmp`)과 단위 시험으로만 잰다.

### WormGame `wie_validate --inject`(release · 기본 27키)
| | result | stop | 입력 | paints | rc |
|---|---|---|---|---|---|
| 전 | FAIL — `06_OK` 에서 `NoSuchMethodError … Manager.createPlayer:(Ljava/lang/String;)…` (`WormPit.createAudioPlayer` ← `WormMain.commandAction`) | error | 6/27 | 31 | 1 |
| 후 | PASS · `content true` · `last_frame_content true` | deadline | 27/27 | 39 | 0 |

WormGame 은 받은 플레이어에 `getControl("ToneControl").setSequence(…)` 를 부른다. 컨트롤이 `null` 이라 거기서 NPE 가 나지만, 자기 `catch (Exception)` 이 삼키고 `audioPlayer` 는 `null` 로 남는다. 그래서 `Manager.playTone`(엔진에 없음)은 `audioPlayer != null` 가드에 막혀 불리지 않는다 — 소스(`WormPit.java:369-381,595-609`)로 확인. 전 측정은 PR #366 브랜치의 release 바이너리(그 브랜치에도 이 오버로드가 없다), 후 측정은 이 브랜치.

### 시험 — `test_tone_device_locator_player`
- `device://tone`: `realize`·`prefetch`·`start`·`stop` 모두 성공, 매 단계 `getState` = 300(`get_state` 는 STARTED/PREFETCHED 만 모델링하고 매체 길이가 0 이다) · `ToneControl` = null · `close` 성공
- `null` → `IllegalArgumentException` · `http://…` → `MediaException`

**변이**: 받는 로케이터를 `device://nothing` 으로 바꾸면 red.

### 하지 않은 것
- `ToneControl`·톤 시퀀스 합성, `Manager.playTone` — 부르는 보유 타이틀 0.
- `device://midi`·`capture://`·`http://` 등 다른 로케이터.

### 게이트
fmt · clippy `-D warnings`(stable·beta·wasm32) · `RUST_MIN_STACK=4194304 cargo test --all` rc 0. 러너 블록: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me` PASS.
`keydraw_ktf`·`keydraw_lgt` 는 기본 예산에서 `UNMEASURED` · `stop=max-ticks`(release 빌드) — 손대지 않은 바이너리도 같았다. `--max-ticks 500000000` 로 올리면 두 바이너리 모두 PASS · 27/27 · paints 55 · rc 0.
