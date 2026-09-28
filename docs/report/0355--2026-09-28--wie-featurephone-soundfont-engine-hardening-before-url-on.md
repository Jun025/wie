## [2026-09-28] 사운드폰트 엔진 보강 — 셸이 URL 을 켜기 전 선행 조건 F1~F3·m1 (wie-featurephone-soundfont-engine-hardening-before-url-on)

근거: 게이트② `wie-featurephone-soundfont-lazy-load-engine.review.md` §3(#381 approve · URL 켜기 전 조건으로 넘김). URL 기본값은 여전히 **끔**(셸이 켠다) — 이 회차만 착지해도 플레이어가 듣는 것은 바뀌지 않는다.

**무엇을**
- **F1 첫 소리 지연** — `audio.rs`: 워클릿 모듈은 이제 URL 유무와 무관하게 **워클릿만** 싣는다. 프렐류드(spessasynth_core)는 사운드폰트 파일이 **도착한 뒤** 같은 `AudioWorklet` 에 **두 번째 `addModule`** 로 싣고(같은 전역 스코프 — `globalThis.wieSoundfont` 가 이미 떠 있는 프로세서에 보인다), 그것이 풀린 뒤에 파일을 넘긴다. 상태기계에 `Prelude` 한 칸이 늘었다: `NotRequested → Requested → (Arrived) → Prelude → Posted`. 프렐류드 적재 실패도 FM 유지 + `warn` 1줄.
- **F2 합성기 상한·재사용** — `audio_worklet.js`: 동시에 렌더되는 사운드폰트 합성기 **최대 3개**(`MAX_SF_SYNTHS`) + **유휴 풀**. 새 Play 는 ⑴유휴 풀 ⑵상한 미만이면 새로 ⑶상한이면 «꼬리만 울리는» 합성기를 빼앗는다 ⑷전부 살아 있는 재생이면 **FM**(소리는 난다). 풀로 돌아갈 때 `stopAllChannels(true)` + `reset()` — 앞 재생의 프로그램·CC 가 다음 재생에 새지 않는다. `Stop` 된 합성기는 페이드 뒤에도 **SF_TAIL_S(1 s) 동안 들리지 않게 계속 렌더**해 리버브를 비운 뒤 풀로 간다(렌더하지 않으면 리버브 지연선이 얼어 있다가 다음 재생 밑에서 옛 곡의 꼬리가 다시 난다). `stats` 에 `idle`·`built` 추가.
- **F3 상태기계 시험** — `scripts/make-sound-fixture.mjs`(새 픽스처: 키마다 SMAF 한 음을 새 `Player` 로 재생 · 메모리에서 생성 · 커밋 안 함) + `contract-roundtrip.mjs` **Scenario S1–S4**(아래). 엔진 필터에 픽스처 생성기와 `wie-web/public/GeneralUser.sf3` 를 더했다(시험 입력일 뿐이라 `publish-artifact.yml` paths 에는 넣지 않았다 — 아티팩트 입력이 아니다).
- **m1** — `check-audio-worklet.mjs` 케이스 12 에 `stats.voices === 0`(사운드폰트 Play 에서 FM 보이스 0). 브라우저 S2·S3 에도 같은 단언.
- 계약 `constructorShapeNote` 문안만 갱신(두 번째 모듈 · 상한 3 · 첫 모듈 바이트 동일). 표면·arity 불변.

**왜 이 모양인가**
- F1 을 «프렐류드를 첫 Play 뒤에 싣기»가 아니라 «파일이 온 뒤에 싣기»로 했다: 프렐류드 평가와 사운드폰트 파싱은 **둘 다 오디오 스레드**에서 돈다. 따로 두면 끊김이 두 번, 붙이면 한 번이다. 파일이 없으면(404) 프렐류드를 아예 싣지 않는다(S4 가 단언).
- F2 에서 이펙트를 끄지 않았다: 합성기 비용의 **~90% 가 이펙트**(아래 표)라 끄면 싸지지만, 운영자가 «B 가 낫다»고 판정한 소리가 이펙트 포함이다. 대신 **합성기 수**를 묶었다(비용이 음 수가 아니라 합성기 수에 비례하므로 그것이 맞는 축이다).
- F3 을 Rust 단위 시험이 아니라 브라우저 왕복으로 했다: `wie_featurephone` 은 wasm32 밖에서 빈 라이브러리(Constraint 7)이고, 상태기계는 `fetch`·`addModule`·`AudioWorkletNode` 의 비동기 **순서**에서 깨졌다(#381 에서 실제로 깨진 것도 `on_ready` 와 첫 Play 의 순서였다). 순서는 실브라우저에서만 재진다.

**실측**

*F1 — 첫 모듈(첫 Play 를 가두는 것) 적재 시간* · Chromium headless(Playwright) · 같은 소스를 Blob 으로 `addModule` 7회 · 호스트 load ≈200 · idle 0% (벽시계라 부풀어 있다 — 비교만 읽어라)

| | 첫 모듈 | 중앙값 | 표본 |
|---|---|---|---|
| 종전(#381 · URL 있음) | 프렐류드+워클릿 한 Blob | **34.1 ms** | 55 43 34 34 34 34 41 |
| 종전·이번(URL 없음) | 워클릿만 | **8.6 ms** | 25 4 9 10 4 6 12 |
| 이번(URL 있음) | 워클릿만 → (파일 도착 뒤) 프렐류드 | **7.5 ms** → 29.9 ms | — |

⇒ URL 유무별 첫 Play 지연 차이가 **구조적으로 0** 이다(첫 모듈이 바이트 동일 — S1·S3 가 `26943B prelude=false` 로 단언). 대가: 프렐류드 평가(~30 ms)가 **첫 소리 뒤** 오디오 스레드에서 한 번 돈다 — 그 순간 끊김 가능. 실기기 측정은 #381 worklog p1(실기기 끊김)이 이미 진다.

*F2 — 동시 합성기 N 개 렌더 비용* · node vm · `process.cpuUsage()`(CPU 시간 ÷ 실시간) · 오르간(지속음) 3 s · JIT 워밍 후 · 같은 부하

| 합성기 × 음 | 이펙트 켬 | 이펙트 끔 |
|---|---|---|
| 1 × 0 | 3.4% | 0.3% |
| 1 × 8 | 4.1% | 1.1% |
| 2 × 8 | 7.9% | 1.4% |
| 3 × 8 | **12.4%** | 2.0% |
| 4 × 4 | 16.9% | 2.1% |
| 4 × 8 | 16.8% | 3.3% |

- 합성기 생성: 중앙 **3.28 ms** · 최대 **66.5 ms**(30회) — 렌더 양자(128/48 kHz = 2.67 ms)를 중앙값만으로 넘는다 ⇒ 풀이 값을 한다(케이스 16: 8회 연속 재생에 `built` 불변).
- ★검수 §3 의 «4×8음 135%» 와 자리가 다르다: 그 수는 **벽시계**(load 224)였고 이 표는 **CPU 시간**이다. 같은 부하에서 벽시계는 스케줄러 대기를 함께 센다. 어느 쪽이든 결론(합성기 수에 비례 · 상한이 필요)은 같다.
- 재사용 합성기의 음색: 새 합성기 대비 차이 rms 6.3e-5(리버브 1 s 비운 뒤) ~ 9.9e-4(코러스 LFO 위상 — 시간 가변이라 샘플 동일은 불가) · 기준 rms 0.0296. 비우지 않으면 3.05e-3(옛 곡 리버브).

*시험* — `check-audio-worklet.mjs --require-soundfont` **19/19**(케이스 0–18 · 새 16 풀 재사용 · 17 MIDI 상태 초기화 · 18 상한/빼앗기/FM) · `contract-roundtrip.mjs` **65/65**(S1–S4 8건 포함).

| 시나리오 | 도는 화살표 | 단언 |
|---|---|---|
| S1 URL 없음 | `Off` | 모듈 1개·프렐류드 없음 · fetch 0 · FM 음 출력 rms > 1e-3 |
| S3 URL · 모듈이 첫 Play 보다 먼저 준비 | `NotRequested`(on_ready 생존) → `Requested` → `Prelude` → `Posted` | 첫 모듈 프렐류드 없음 · 첫 Play 전 fetch 0 · fetch ≥ 첫 play · 둘째 모듈 = 프렐류드 · 파일 도착 뒤 · 다음 Play 합성기 ≥1·**FM 보이스 0**·출력 있음 |
| S2 URL · 파일이 모듈보다 먼저 | `Arrived` → (on_ready) `Prelude` → `Posted` | 첫 모듈 해소를 파일 본문 도착까지 붙잡아 Play 를 큐에 둔다 · body < node · 프렐류드 ≥ node · 다음 Play 사운드폰트·FM 0 |
| S4 URL 404 | `Requested` → 실패 | `warn` 1줄 · 모듈 1개(프렐류드 안 실음) · FM · `console.error` 0 |

*변이(전부 red 여야 한다)*
- 워클릿(node): 이중 렌더(`processMessage` 뒤 `return` 삭제) → 케이스 12·18 FAIL · 상한 제거(`MAX_SF_SYNTHS=99`) → 18 · 풀 제거 → 16(`built` 6→14)·18 · `reset()` 제거 → 17 · 비우기 제거(페이드 0 에서 곧장 풀로) → 14.
- Rust·워클릿(브라우저 · 변이마다 `build:wasm` 재빌드 · 전부 rc=1):

| 변이 | 결과 | 빨개진 단언 |
|---|---|---|
| R1 `on_ready` 가드 제거(#381 에서 실제로 난 결함 재현 — `NotRequested` 가 첫 Play 전에 사라짐) | 62/65 | S3 fetch·프렐류드·게시 · S3 다음 Play · S4 경고 없음 |
| R2 프렐류드를 첫 모듈에 다시 합침(F1 되돌림) | 63/65 | S1 «모듈 1개 · 프렐류드 없음»(376,984 B prelude=true) · S3 «워클릿만» |
| R3 모듈 준비 전에 온 버퍼를 버림(`Arrived` 경로) | 63/65 | S2 게시 · S2 다음 Play(synths 0) |
| R4 이중 렌더(워클릿 · m1 의 브라우저 판) | 63/65 ×3 | S3·S2 «FM 보이스 0» |

**Definition of Done** — `cargo fmt --check` rc0 · `cargo clippy --all -D warnings` rc0 · wasm32 clippy rc0 · `cargo +beta clippy --all -D warnings` rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **580 passed / 0 failed** · `npm run build:wasm` rc0 — wasm **16,261,791 B** · 글루 **38,544 / 40,000 B**(여유 1,456 B = **3.6%** · #381 의 38,683 보다 139 B 작다 — 새 web-sys 바인딩 0) · `check-engine-contract` **113/0** · `check-audio-worklet`(기본·`--require-soundfont`) OK · `contract-roundtrip` **65/65**(연속 3회) · `npm run audit` PASSED.
- ★round-trip 이 두 번 흔들렸고 둘 다 시험 쪽 결함이었다 — 고친 판에서 연속 3회 65/65:
  ⑴**S1 64/65**(«voices 0 · rms 0» · 호스트 load 344): 픽스처 음이 2 s 라 `stats` 가 돌아오기 전에 끝났다 ⇒ 음 5 s + 한 번 재는 대신 «울릴 때까지» 폴링.
  ⑵그 폴링이 **R4 를 놓쳤다**(2회 중 1회 65/65): 사운드폰트 합성기는 Play 메시지가 닿을 때 할당되고 음(과 이중 렌더의 FM 음)은 **다음 양자**에 나온다 — `stats` 를 먼저 읽고 출력 레벨을 나중에 읽으면 그 사이에 음이 나가 «synths 1 · voices 0 · 소리 있음»이 된다. 순서를 «레벨 먼저 → stats»로 바꾼 뒤 R4 **3/3 red**(S2·S3 둘 다).
  R1–R3 은 ⑴⑵ 이전 판에서 쟀다 — 바뀐 것은 폴링·순서뿐이고 R1–R3 이 빨갛게 만드는 단언(fetch·모듈·게시)은 그 경로를 지나지 않는다.

**한계**
- 프렐류드 평가(~30 ms)·파싱이 첫 소리 **뒤** 오디오 스레드에서 한 번 돈다 — 첫 소리 지연을 «한 번의 끊김 가능성»과 바꾼 것이다. 실기기 크기는 모른다(#381 worklog p1).
- 상한에서 빼앗긴 합성기는 리버브를 다 비우지 못한 채 넘어간다(꼬리 잔향이 새 재생 밑에 잠깐 남을 수 있다). 빼앗기는 순서를 «이미 조용한 것 먼저 · 꼬리가 가장 오래된 것»으로 두어 줄였다.
- 네 번째 동시 MIDI 재생은 FM 이다 — 사운드폰트와 FM 음색이 섞일 수 있다. 50타이틀 코퍼스의 동시 MIDI 핸들 분포는 재지 않았다(#381 의 브라우저 실행 게임은 동시 1).
- S 시나리오의 소리 검사는 헤드리스 Chromium 의 가짜 오디오 출력 위 `AnalyserNode` rms 다 — «실제 스피커»가 아니라 «그래프에 샘플이 흘렀다»를 뜻한다.

게임 파일명 유입: BOUNDED 2회/2쌍 — 둘 다 `audio.rs` 의 기존 줄(이 회차가 쓴 것 아님) · SUFFIX-ATTACHED 0 · PREFIX-EMBEDDED 1(제외).
