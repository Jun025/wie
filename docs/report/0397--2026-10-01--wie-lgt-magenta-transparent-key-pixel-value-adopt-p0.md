## [2026-10-01] LGT 마젠타 투명색 — 키 비교는 게임이 컨텍스트에 꽂은 pixel-op 프록 안에 있다 (wie-lgt-magenta-transparent-key-pixel-value-adopt-p0)

**무엇을**: LGT 의 `MC_grpDrawImage`·`MC_grpCopyFrameBuffer` 를 LGT 래퍼로 바꿨다. 그래픽 컨텍스트의 native `pixel_op`(+28)에 프록이 있으면 대상이 16bpp 일 때 화소마다 `proc(dst, src)` 를 게스트에서 부르고 결과를 쓴다. 디코드 이미지(32bpp ARGB) 화소는 RGB565 값으로 넘기고, 완전 투명(α 0) 화소는 건너뛴다. 한 blit 안에서 `(dst, src)` 결과는 메모한다. 프록이 없으면 종전 공유 경로 그대로다. `compat.json` 1행(`5814101b8010` limited → playable).

**왜**: 0391 §7 의 «MC_grpDrawImage 0회»는 키 없는 실행의 값이었다. 키를 넣으면 같은 타이틀이 `MC_grpDrawImage` 를 604회 부르고, 그 컨텍스트(`0x15000d8`) +28 에 `0x3839` 가 들어 있다.

### 1. 게임의 키 비교 — 호출부

| 무엇 | `5814101b8010` | `34c48bbae783` |
|---|---|---|
| 프록 | `0x3838`: `r2=r0; r0=r1; if (r1 == [g+0x74]) r0=r2; return r0` | `0x6ea0`: 같은 모양, 키는 `[g+0x68]` |
| 저장 | `0x29aa: ldr r3,=0x3839 ; str r3,[r1,#0x1c]` (r1 = 컨텍스트) — SetContext 아님 | 리터럴 `0x6ea1` @ `0x78a8` |
| 키 값 | 부팅 때 `MC_grpGetPixelFromRGB(0xff, 0, 0xff)` → `0xf81f`(우리 답도 `0xf81f`) | 같음 |
| 다른 프록 | `0x3850`(실루엣) · `0x3864` AND · `0x3878` OR · `0x388c` XOR · `0x38a0` 반투명 — 전부 «`src == 키` 면 `dst`» | `0x6cb1` `0x6eb9` `0x6f85` `0x703d` |

⇒ 인자 순서는 `(dst, src)` 이고 비교는 **16비트 화소값**이다. 세 번째 인자를 읽는 프록은 없다.

| 질문 | 답 |
|---|---|
| 우리 이미지 화소의 키 값은 게임 기대와 같은가 | **다르다.** BMP 디코더가 32bpp ARGB 로 올려 키 자리가 `0xffff00ff` 다. 게임은 RGB565 `0xf81f` 와 비교한다 ⇒ 블릿 경로에서 565 로 바꿔 넘긴다. 디코더 자체는 바꾸지 않았다(다른 SVC·KTF 가 같은 디코더를 쓴다) |
| 그럼 왜 마젠타가 찍혔나 | 공유 `draw_image`·`copy_frame_buffer` 가 컨텍스트를 아예 읽지 않는다. 프록이 불린 적이 없다 |
| 다른 LGT 타이틀이 바뀌나 | 전수(78종 · 키 27개 · 30초)에서 프록을 탄 것은 이 2종뿐이다. 나머지는 프록 0회 ⇒ 종전 경로 |
| +28 은 공유 `offset` 자리다 — 충돌은 | SetContext(Offset) 를 부르는 LGT 타이틀은 1종(`40b9537968de`)이고 그 타이틀은 Copy 0회(15초 전수). 관측 0 |

추적 방법(커밋 0): 화면 버퍼 범위에 쓰는 `0xf81f` 를 CPU 저장·호스트 쓰기 양쪽에서 잡는 진단 빌드. CPU 저장 0회 · 호스트 전면 쓰기 직전 SVC = `MC_grpDrawImage` 604 · `MC_grpFillRect`(이미 찍힌 마젠타를 다시 쓴 것).

### 2. 짝 재측

조용한 호스트(`host-load-guard --status --recovered` rc=0 · 시작 load1 20.06) · 전 = `origin/main 1becaae2` · 후 = 이 브랜치 · 같은 시각 나란히 · `--jobs` ≤ 3.

**장시간**(census 와 같은 인자 · 600초 · 900키 · `--relaunch 1`). 마젠타 프레임 = 20초 간격 스크린숏 중 `R>200·G<60·B>200` 화소가 있는 것.

| 타이틀 | 전 | 후(메모 전 빌드) |
|---|---|---|
| `5814101b8010` | **FAIL** magenta 20% · 878/883 프레임 · 최대 20.3% | 854/900키 · deadline · **0/883** |
| `34c48bbae783` | **FAIL** magenta 28% · 879/883 · 최대 28.0% | 854/900키 · deadline · 78/883 프레임에 ≤10px(<0.02%) |
| `6b515884dbc1` | 0/883 | 0/883 |

**메모의 값**(180초 · 세 빌드 나란히 · load1 13.7→19.6): paints

| 타이틀 | main | 메모 없음 | 메모 |
|---|---|---|---|
| `34c48bbae783` | 12,930 (FAIL) | 9,872 (−24%) | **13,763** |
| `5814101b8010` | 5,023 (FAIL) | 4,667 (−7%) | **4,840** |

화소당 게스트 호출은 600초 짝에서도 −14%·−23% 로 보였다 ⇒ 메모를 넣었다. 프록은 인자와 타이틀 전역만 읽고, 한 blit 동안 게스트는 돌지 않는다.

**프로브**(30초 · 27키 · 빌드마다 2회): 마젠타 프레임 / paints

| 타이틀 | 전 | 후 |
|---|---|---|
| `5814101b8010` | FAIL · 23/23 · 28/28 | PASS · 0/28 · 0/28 |
| `34c48bbae783` | PASS · 25/28 · 25/28 | PASS · 0/28 · 0/28 |
| `6b515884dbc1` | 1/28 · 1/28 (1·17px) | 1/28 · 0/28 (17px) |
| 라이브 LGT 5종 `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684` | 마젠타 0 · paints 141/132 · 110/113 · 12/12 · 87/88 · 244/242 | 마젠타 0 · 135/138 · 128/150 · 12/10 · 87/89 · 245/244 |
| 가드 `ddd885583b15` | PASS · 1246/1074 | PASS · 1132/1012 |

`b475b6399684` 는 전 2회 중 1회만 deadline PASS 이고 나머지 3회는 `max-ticks` UNMEASURED 다 — 프록을 타지 않는 타이틀이라(전수 0회) 부하 흔들림이다. `4ece6eeeaa04` 는 0329 와 같은 clean exit.

### 3. `6b515884dbc1` — 이 PR 의 메커니즘이 아니다

census L(load1 653)의 «23% 마젠타» 는 재현되지 않았다: 600초 두 빌드 모두 0/883, 프로브 최대 17px. 프록 0회 · `CopyFrameBuffer` 컨텍스트 +28 = 0. census 가 그 프레임을 지워 화면을 볼 수 없다. `compat.json` 행은 그대로 두었다.

### 4. 되돌리면 red
`wipic_blits_run_the_native_pixel_op`: ⑴ `DrawImage` 를 공유 경로로 되돌림 → red ⑵ ARGB→565 변환 제거 → red ⑶ (첫 커밋) `CopyFrameBuffer` 를 공유 경로로 → red.

### 5. 게이트
fmt · clippy stable/wasm32/beta `-D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 러너 블록 6줄 PASS(`keydraw_*` rc=0 · paints 79/55).

**사용자 영향**: 두 LGT 타이틀의 타이틀 화면·메뉴·글자 둘레 분홍색이 사라진다. 지원 현황 playable 367 → 368 · limited 42 → 41(`5814101b8010` — longplay 의 FAIL 이 바로 이 마젠타 판정이었다. speed `no` 는 그대로).

**남은 것**: `34c48bbae783` 의 ≤10px 잔여(78/883 프레임) — 원인 미조사. 600초 «후» 열은 메모 전 빌드다 — 메모 빌드는 180초 짝에서 두 타이틀 모두 FAIL 아님(마젠타 임계 미만).

**게임 파일명 유입**: BOUNDED 336쌍 · SUFFIX-ATTACHED 15쌍 — 전부 `compat.json` 파일 단위 스캔이 잡은 기존 `title`·`fileTitle` 값이다. 이 회차가 더한 줄(코드 · 이 문서 · worklog · player-updates · compat 의 status/longplay 2줄)의 게임명은 0.
