## [2026-09-27] 소리 silent 104 — «시도 없음 vs 경로 결손» 분류 · 결손 3종 수정 (wie-census-silent-104-classify-and-fix)

**무엇을**: 전수 점검(0321)의 «소리 silent» 104종을 다시 재서 가르고, 경로 결손 3종을 고쳤다. ① WIPI-C `MC_knlGetSystemProperty("MEDIADEVICES")` = `"Yamaha_MA3"` ② KTF `org.kwis.msp.media.Player` 의 `BaseClip` 오버로드(play/stop/resume/pause)가 `Clip` 인스턴스면 Clip 경로로 ③ `BaseClip.setBuffer` 가 배열을 붙잡고 `Player.play` 가 게임이 다시 쓴 배열을 다시 읽는다.
**왜**: 후속 군집 표 1행(0321) — 104종이 한 덩어리로 «silent» 였고, 측정 창 문제와 엔진 결손이 섞여 있었다.
**사용자 영향**: 6종이 소리를 낸다(KTF 3 · LGT 3). 나머지는 이 회차에서 바뀌지 않는다.

### 1. 측정 — 무엇을 어떻게 쟀나

- 대상: 0321 `clusters.md` 의 `sound:silent` 104종(KTF 59 · SKT 20 · LGT 25). 모집단·경로는 0321 의 비공개 `census.tsv` 그대로.
- 한 판 = `wie_validate --inject --keys <0321 장시간 키 루프> --keep-timeout --timeout 90 --max-ticks 1e11`, release 빌드.
  - 0321 의 30초 창은 `--max-ticks` 기본값(5천만)에 먼저 걸려 24~28초에 끝난 판이 있었다(예 `01e2715ba07a` `stop: max-ticks` 24.3초). 이번 판은 상한을 풀었다.
- 소리 API 호출은 `RUST_LOG` 로 미디어 모듈만 debug 로 올려 이름별로 셌다(`wie_wipi_c::api::media` · `wie_wipi_java::…::media` · `wie_skvm::…::m`·`net::wie` · `wie_midp::…::media`·`net::wie` · `wie_backend::system::audio`) + 모든 `stub` 경고 + `unknown system property`.
  - 도구는 스크래치(레포 밖)에 두었다. 이유: 결과 줄 JSON 에 새 키를 더하지 않고 기존 로그만 읽으면 됐다.
- 판정 축은 0321 과 같다: 싱크에 닿은 `Play` 중 이벤트가 있는 것(`plays - empty_plays`).
- 호스트: load1 **38 → 167**(최종 짝 재측 22:27→22:46 · 10코어) · 같은 시각에 전/후 빌드를 6판씩 나란히 돌렸다. 부하가 판의 진행 속도를 바꾸므로 **전/후는 같은 시각 짝으로만** 비교한다.

### 2. 분류(104)

| 분류 | KTF | LGT | SKT | 계 | 뜻 |
|---|---|---|---|---|---|
| ⒜ 창 — 재생 확인 | 5 | 4 | 0 | 9 | 90초 또는 0321 의 600초 판에서 소리가 났다. 30초 창이 짧았다 |
| ⒜ 창 — 준비만 | 14 | 7 | 10 | 31 | 볼륨·클립 준비 API 는 불렀으나 90초 안에 play 호출이 없다 |
| ⒜ 창 — 호출 없음 | 21 | 7 | 6 | 34 | 90초 안에 소리 API 호출 0 |
| ⒞ 다른 벽이 먼저 | 11 | 4 | 4 | 19 | 90초 판이 30초 안에 오류·자체 종료로 끝났다(무키 판도 같음) |
| ⒝ 결손 — #350 소관 | 5 | 0 | 0 | 5 | `Player.stop(null)` 호스트 panic 으로 판이 죽는다. #350 의 null 클립 수정이 진다 |
| ⒝ 결손 — 이번 수정 | 3 | 3 | 0 | 6 | 아래 §3 |

- ⒜ «준비만»·«호출 없음» 은 **엔진 결손의 증거가 없다** — 호출된 API 는 전부 구현돼 있고, 미구현 호출·Java 예외가 없다(15종 `java_exceptions` 직접 확인 · 0 또는 소리 무관).
  - 표본 확인: `6c9f969f089f` 의 0321 무키 판 10초 샷은 **로딩 막대**다. 포화 호스트에서 90초는 메뉴에 닿기에도 모자란 판이 있다.
  - ★그중 LGT 11종은 첫 벽이 «첫 실행 안내 · WIPIC `unk13`» 군집(0321 후속 표 3행)이다 — 그 화면을 못 넘기니 소리 차례가 오지 않는다.
- ⒞ 는 판정 근거가 «최종 짝의 전 빌드 판»이라 흔들린다: `888702965551` `1793f87924d4` 는 다른 판에서 90초를 버텼다. 소리 결손으로 세지 않았다.
- ⒝ #350 소관 5종: 스택 `jvm class_instance.rs:108`(null 역참조) 직후 `Player::stop` — #350 이 `Clip::player` 의 null 을 막았고 그 PR 은 아직 열려 있다. 이 브랜치에 합치지 않았다(겹쳐 고치지 않는다).

행 단위 표(104행 · sha12 전용)는 §6.

### 3. 고친 결손 3종

| # | 결손 | 신호 | 걸린 수 | 되돌리면 red |
|---|---|---|---|---|
| 1 | `MEDIADEVICES` 미등재 → `M_E_INVALID` | `unknown system property id: MEDIADEVICES` 직후 `MC_mdaClipCreate` 0회, `ClipPutData(0x0, …)`(클립 포인터 0) | LGT 3 | `test_get_system_property_media_devices` — 줄 삭제 시 panicked(실측) |
| 2 | `Player.play/stop(BaseClip…)` 스텁 | `stub org.kwis.msp.media.Player::play(…)` · 호출 인자는 `Clip` | KTF 2 | `test_clip_through_base_clip_overloads_plays` — `is_instance` 를 `false` 로 바꾸면 panicked(실측) |
| 3 | `setBuffer` 가 배열을 복사만 함 | `setBuffer` 1회 · 그 뒤 `availableDataSize`→`play` 123회 · 재생 전부 빈 시퀀스 | KTF 1 | `test_play_reloads_a_rewritten_set_buffer_array` — `refresh` 호출 삭제 시 panicked(실측) |

- 1: 게임 바이너리의 문자열 표에서 `MEDIADEVICES` 바로 뒤에 `Yamaha_MA3`(LGT 3종 · KTF 1종)와 `audio/MP3`·`audio/MIDI` 가 온다. 값에서 칩 이름을 찾아 소리 경로를 고르는 형태다. 엔진이 재생하는 형식이 SMAF 뿐이라 `Yamaha_MA3` 하나만 답한다.
  - `VOLUMELEVEL="3"`·`VIBRATORLEVEL="0"`(Java `HandsetProperty` 와 같은 값)도 같이 넣어 짝 재측했으나 해당 4종(`f44271803135` `1cd151222bde` `40b9537968de` `b44b5fbe29e6`)의 소리·호출이 **바뀌지 않아 뺐다**(효과가 재지지 않는 변경은 싣지 않는다).
- 2: `BaseClip` 에 있는 것은 `player` 필드뿐이고 볼륨은 `Clip` 에 있다 ⇒ `Clip` 이면 Clip 오버로드에 위임, 맨 `BaseClip` 은 종전대로 `false`(기존 시험 `test_base_clip_overloads_return_false` 유지). MIDP `stop` 이 재생 위치를 유지하므로 `pause` 는 `stop` 으로 보냈다.
- 3: 게임이 `setBuffer` 에 준 배열의 **첫 내용이 PNG**(14,636바이트 · 머리 `89 50 4E 47`)였다 — 스크래치 배열을 등록해 두고 소리마다 그 배열에 복사해 쓰는 형태다. 게임 자신은 `putData` 를 한 번도 부르지 않는다(로그의 `putData` 1회는 우리 `setBuffer` 가 내부에서 부른 것).
  - 처방: `BaseClip` 이 배열·크기·FNV-1a 해시를 들고, `Player.play`(Clip) 때 해시가 바뀌었으면 옛 플레이어를 닫고 다시 적재한다. 안 바뀌었으면 아무것도 안 한다(같은 핸들 — 시험이 잠근다).
  - `resume` 에는 걸지 않았다 — 매 `play` 직후 `resume` 을 부르는 타이틀이 있어, 거기서 해시를 매번 계산할 이유가 없다.
  - `clearData` 는 배열도 놓는다(지운 뒤 play 가 옛 배열을 되살리지 않게).
- KTF `MC_mdaUnk17/18(3)`(9종)은 반환값 변이(0 → 7)로 짝 재측했으나 4종 모두 호출·소리·종료 시점이 같았다 — 이 회차의 결손으로 세지 않았다.

### 4. 전/후 — 같은 시각 짝 재측(104종 전부 · 90초)

전 = 이 브랜치 부모 `bc358feb`(볼륨 PR #356 머리) · 후 = 이 브랜치.

| sha12 | 계열 | 수정 | 싱크에 닿은 소리(전 → 후) |
|---|---|---|---|
| 21ffd1c61ebd | lgt | 1 | 0 → 3 |
| 5814101b8010 | lgt | 1 | 0 → 2 |
| c1718bcdf8a9 | lgt | 1 | 0 → 8 |
| 232122cdfb92 | ktf | 2 | 0 → 43 |
| 3412d851f78c | ktf | 2 | 0 → 6 |
| 4451036a7fb6 | ktf | 3 | 0 → 8 (재생 120 → 9 · 빈 재생 120 → 1) |

- **나빠진 타이틀 0** · 같음 98. 수정 1·2 만 든 중간 빌드의 짝(22:06~22:26)에서도 1·2 의 5종이 0 → 3~20 으로 같은 방향이었다.
- `4451036a7fb6` 의 재생 수가 준 것은 소리가 이제 길이를 가져서다(MIDI 이벤트 0 → 1,457). 속도 저하가 아니다 — paints 1,151 → 1,174 · ticks 1.44억 → 1.52억.
- 판정이 바뀐 4종(`30c7bd6fb01b` `c7f543c73b91` `1793f87924d4` `6b515884dbc1`)은 전부 «오류/자체 종료 → 90초 생존» 방향이지만 **고쳤다고 세지 않는다** — 같은 전 빌드의 다른 판에서 이미 90초를 버틴 기록이 있다(흔들림).

### 5. 퇴행 가드

- 라이브 LGT 5종 + 퇴행 가드 2종: 전/후 짝(30초 · load1 ~203) **7/7 부팅·화면 ok**(`content: true`).
  - `1b107b96bf4e` 후 판 1회가 22초에 «자체 종료» — 짝 재측 2회 더(각 전/후 동시)에서 전후 모두 30초 생존·39키 ⇒ 재현 안 됨(부하 속 키가 다른 화면에 떨어진 흔들림).
- 러너 블록(AGENTS §The four gates): draw/helloworld 3종 PASS · keydraw KTF/LGT `--inject --expect-last-frame` PASS·rc 0 · text_j2me PASS(release 빌드로 돌렸다).

### 6. 행 단위 분류표

| sha12 | 계열 | 분류 | 근거 | 후 90초 전→후 | 신호 |
|---|---|---|---|---|---|
| 1b3b4868d46e | ktf | ⒜ 창 | 90초/600초 창에서 재생 2 | 0→0 |  |
| 3d2381657960 | ktf | ⒜ 창 | 90초/600초 창에서 재생 147 | 12→12 |  |
| 5bfe10a82849 | ktf | ⒜ 창 | 90초/600초 창에서 재생 32 | 32→32 |  |
| 6103e87874c6 | ktf | ⒜ 창 | 90초/600초 창에서 재생 1 | 0→0 |  |
| a540945188ca | ktf | ⒜ 창 | 90초/600초 창에서 재생 193 | 0→0 |  |
| 580a66c32fff | lgt | ⒜ 창 | 90초/600초 창에서 재생 28 | 4→4 | stub:MC_mdaClipAllocPlayer |
| 601556233e71 | lgt | ⒜ 창 | 90초/600초 창에서 재생 1 | 1→1 | stub:MC_mdaClipAllocPlayer |
| 6b515884dbc1 | lgt | ⒜ 창 | 90초/600초 창에서 재생 5 | 5→5 | stub:MC_mdaClipAllocPlayer,stub:unk15 |
| 863b8ab6a21d | lgt | ⒜ 창 | 90초/600초 창에서 재생 7 | 2→2 | stub:unk5,stub:MC_mdaClipAllocPlayer,stub:unk15 |
| 04159045a7ea | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 0e72b6bc12bb | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 36b82cb67723 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 4892a1abc0f8 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:MC_mdaUnk17 |
| 4b8c8f5ff7d6 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:MC_mdaUnk17 prop:ESN |
| 4decaeed58b1 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:MC_mdaUnk17 |
| 6c9f969f089f | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:org.kwis.msp.media.Clip::setListener |
| 7218e8720f8c | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:org.kwis.msp.media.Clip::setListener |
| 75f002ba70e3 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 974e0df9ab1e | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:MC_mdaUnk18,stub:MC_mdaUnk17 |
| b1ec149b354c | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| cdd3eb5f9142 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:MC_mdaUnk17 |
| f44271803135 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:MC_mdaUnk17 prop:VOLUMELEVEL |
| f7752f9124e8 | ktf | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 1045007289d8 | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 287af341dac8 | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| 34c48bbae783 | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| 619d98bc8f64 | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 · LGT unk13(첫 실행 안내 벽) | 0→0 | stub:MC_mdaClipAllocPlayer,stub:unk5 |
| 63332c51d514 | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| 870cd071b4dc | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| acf6863fc84c | lgt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| 38277d63b0ba | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| 3bafa1f1eed2 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| 3de4bcb4f5e2 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| 6e93f26fa2f5 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| c107462e5f8a | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| c361632541a7 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| ca1132f2e7bc | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 |  |
| d2957348ddf4 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| f483ba078c14 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled,stub:com.skt.m.Device::isKeyToneEnabled |
| fba094100d13 | skt | ⒜ 창 — 준비만 | 클립 준비 후 90초 안에 play 호출 없음 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| 01e2715ba07a | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 0865be217bde | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 1d18b7cc06d1 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 229291e20b13 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 23919eb33365 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 2cbd63e4427a | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 34ab350dc98a | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 5028b8a5d19f | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 51011b242bb5 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 5aa438fbdc87 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 689c491586b9 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 704839f1a6c8 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 74cb49013d64 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 8b899f410f5d | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 8bfd08fe4370 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 9e16cc54d0ab | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| a5651a180b7a | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| a7d1f26af3e1 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| c36cbe78011f | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| d9afc4db742c | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| eb1614abed70 | ktf | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 1352b27a7898 | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| 1cd151222bde | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 · LGT unk13(첫 실행 안내 벽) | 0→0 | prop:VIBRATORLEVEL |
| 2520654be6de | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| 40b9537968de | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 · LGT unk13(첫 실행 안내 벽) | 0→0 | prop:VIBRATORLEVEL |
| 73f3a21e981c | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| b44b5fbe29e6 | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 · LGT unk13(첫 실행 안내 벽) | 0→0 | prop:VIBRATORLEVEL |
| be08d047cbae | lgt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 14a62a8521a0 | skt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| 3eb73c20bbae | skt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| 640428a9cf9e | skt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| abc0c5789259 | skt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 |  |
| ccb45e6b8d80 | skt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| f12984cd0d37 | skt | ⒜ 창 — 호출 없음 | 90초 안에 소리 API 0 | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| 182fa44210dc | ktf | ⒝ 결손 — #350 소관 | Player.stop(null) 호스트 panic | 0→0 | thread |
| 41466fc7f709 | ktf | ⒝ 결손 — #350 소관 | Player.stop(null) 호스트 panic | 0→0 | thread |
| 52f1f32e3f72 | ktf | ⒝ 결손 — #350 소관 | Player.stop(null) 호스트 panic | 0→0 | thread |
| 91c02913e1ea | ktf | ⒝ 결손 — #350 소관 | Player.stop(null) 호스트 panic | 0→0 | thread |
| d1fba7ad5d1f | ktf | ⒝ 결손 — #350 소관 | Player.stop(null) 호스트 panic | 0→0 | thread |
| 232122cdfb92 | ktf | ⒝ 결손 — 이번 수정 |  | 0→43 |  |
| 3412d851f78c | ktf | ⒝ 결손 — 이번 수정 |  | 0→6 |  |
| 4451036a7fb6 | ktf | ⒝ 결손 — 이번 수정 |  | 0→8 | stub:org.kwis.msp.media.BaseClip::availableDataSize |
| 21ffd1c61ebd | lgt | ⒝ 결손 — 이번 수정 |  | 0→3 | prop:MEDIADEVICES |
| 5814101b8010 | lgt | ⒝ 결손 — 이번 수정 |  | 0→2 | stub:unk5 prop:MEDIADEVICES |
| c1718bcdf8a9 | lgt | ⒝ 결손 — 이번 수정 |  | 0→8 | stub:unk15,stub:MC_mdaClipAllocPlayer prop:MEDIADEVICES |
| 155972cac664 | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 5s | 0→0 |  |
| 1793f87924d4 | ktf | ⒞ 다른 벽이 먼저 | 자체 종료 7s | 0→0 |  |
| 30c7bd6fb01b | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 12s | 0→0 | stub:MC_mdaUnk18,stub:MC_mdaUnk17 prop:MEDIADEVICES |
| 4166acd8fc62 | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 0s | 0→0 |  |
| 4a4d2ac046f7 | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 0s | 0→0 | stub:MC_mdaUnk18,stub:MC_mdaUnk17 prop:TIMEZONE |
| 888702965551 | ktf | ⒞ 다른 벽이 먼저 | 자체 종료 15s | 0→0 |  |
| 89c214dbd15d | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 2s | 0→0 |  |
| aa4542c94ce4 | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 2s | 0→0 |  |
| c7f543c73b91 | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 10s | 0→0 | stub:org.kwis.msp.media.Clip::setListener,thread |
| ca7fa8ade8ad | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 1s | 0→0 |  |
| f981d228b757 | ktf | ⒞ 다른 벽이 먼저 | 실행 오류 1s | 0→0 | stub:MC_mdaUnk18,stub:MC_mdaUnk17 prop:TIMEZONE |
| 3ff5948e235e | lgt | ⒞ 다른 벽이 먼저 | 실행 오류 9s · LGT unk13(첫 실행 안내 벽) | 0→0 |  |
| 517ed32c92d6 | lgt | ⒞ 다른 벽이 먼저 | 실행 오류 14s | 0→0 | thread |
| a16f08d025eb | lgt | ⒞ 다른 벽이 먼저 | 실행 오류 1s | 0→0 | thread |
| fe76e641bb3d | lgt | ⒞ 다른 벽이 먼저 | 실행 오류 4s | 0→0 | thread,stub:MC_mdaClipAllocPlayer |
| 0ed66634d3dc | skt | ⒞ 다른 벽이 먼저 | 자체 종료 2s | 0→0 |  |
| a42f77f44955 | skt | ⒞ 다른 벽이 먼저 | 자체 종료 6s | 0→0 | stub:com.skt.m.Device::setKeyToneEnabled |
| c33090c12755 | skt | ⒞ 다른 벽이 먼저 | 자체 종료 2s | 0→0 |  |
| eefc947d8337 | skt | ⒞ 다른 벽이 먼저 | 자체 종료 2s | 0→0 |  |

(«후 90초 전→후» = 최종 짝 재측의 싱크 도달 소리 수. «신호» = 그 타이틀에서 관측한 소리 관련 스텁·미등재 속성.)

### 7. 남은 묶음(총괄 발권용)

| 걸린 수 | 첫 벽 | 크기 |
|---|---|---|
| 65 | ⒜ 준비만 31 + 호출 없음 34 — 포화 호스트(load 38~200)의 90초로는 소리 차례에 닿지 않는다. 조용한 호스트에서 창을 늘리거나 타이틀별 `--keys` 경로로 재야 «결손 없음»이 확정된다 | M(측정) |
| 11 | 위 65 중 LGT — 첫 벽이 «첫 실행 안내 · WIPIC `unk13`»(0321 후속 3행) | 그 군집에 합침 |
| 19 | ⒞ 소리 이전에 오류·자체 종료. SKT 4종은 무키로도 2~6초에 스스로 끝난다(`0ed66634d3dc` `a42f77f44955` `c33090c12755` `eefc947d8337`) | 각 부팅/장시간 군집 |
| 5 | #350 착지 후 재측(`182fa44210dc` `41466fc7f709` `52f1f32e3f72` `91c02913e1ea` `d1fba7ad5d1f`) | S |
| 9 | KTF `MC_mdaUnk17/18(3)` 미식별 — 반환값 변이 무반응. 소리 무음과의 관계 미확인 | S(식별) |
| 1 | `f44271803135` — `MC_mdaClipSetVolume(0x0, 0)`(클립 없음) · `VOLUMELEVEL` 미등재 · KTF `unk12-1` 4,407회 루프. `VOLUMELEVEL="3"` 은 무반응 | S |

게임 파일명 유입(`corpus-name-inflow --corpus ~/work/otterpebble/wie/game_lab`): BOUNDED 4회/2쌍 · SUFFIX-ATTACHED 0회/0쌍. BOUNDED 4회는 전부 수정 파일의 기존 줄이다(`kernel.rs` 1 · `player.rs` 3 — 이 회차의 추가 줄 중 일치 0).

<!-- corpus-name-inflow v1 subjects=7 tree=f2ec1d2a00bad1b0 B=4/2 P=1/1 S=0/0 -->
