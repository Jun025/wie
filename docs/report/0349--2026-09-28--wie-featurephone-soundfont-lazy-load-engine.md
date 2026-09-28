## [2026-09-28] 피처폰 게임 음악 — 사운드폰트(GeneralUser GS) 지연 로드 · 엔진 쪽 (wie-featurephone-soundfont-lazy-load-engine)

**무엇을**
- 생성자 **8번째 선택 인자 `soundfontUrl`**(`wie_featurephone/src/lib.rs`). 없으면 종전 7인자와 같은 동작 — FM 만 · fetch 0 · 워클릿 모듈 바이트 동일(아래 실측).
- 주면: 첫 소리는 **FM 그대로**(지연 0) → 첫 `Play` 를 보낸 **뒤** 그 URL 을 백그라운드 GET(`audio.rs` `fetch_soundfont`) → `ArrayBuffer` 를 워클릿에 **이전(transfer)** → 워클릿이 파싱 → 그 뒤 **새로 시작하는 Play** 부터 그 Play 전용 `SpessaSynthProcessor` 로 MIDI 를 낸다. 이미 울리는 재생은 다음 Play 까지 FM(음 중간 교체 없음). PCM 은 항상 종전 경로.
- fetch 실패 · HTTP 오류 · 파싱 실패 · 프렐류드 없는 빌드 = **전부 FM 유지**(throw 없음 · 콘솔 `warn` 1줄 `[wie] soundfont … — playing FM`). 성공은 콘솔 `log` 1줄. ★`console.error` 는 쓰지 않는다(`verify-browser.mjs` 는 error 로 배포를 red 로 만든다 — 사운드폰트 부재는 «음질 저하»지 실패가 아니다).
- 합성기 = spessasynth_core 4.3.18 을 esbuild 로 IIFE 하나(350,043 B)로 묶은 **프렐류드**(`wie_featurephone/src/soundfont_prelude.mjs` → `scripts/build-soundfont-prelude.mjs`). `scripts/build-wasm.sh` 가 만들고(`node_modules` 없으면 `npm ci`) `WIE_SOUNDFONT_PRELUDE` 로 넘기며, `wie_featurephone/build.rs` 가 `include_str!` 할 파일을 `OUT_DIR` 에 쓴다. 워클릿과 **한 Blob 모듈**로 실려 계약의 «파일 2개»(`artifacts.files`)는 그대로다.
- 계약: `constructorArity` 7→8 · `constructorShape`·`constructorShapeNote` · 새 절 `soundfontPrelude`(marker · 라이선스 경로). `check-engine-contract.mjs` §3b 가 **wasm 바이트에서 marker 를 찾는다** — `build-wasm.sh` 를 안 거친 빌드(프렐류드 빈 문자열 → URL 을 받아도 FM 만)는 이제 위반이다.
- 라이선스(`licenses/`): `Apache-2.0.txt`(spessasynth_core · stb-vorbis — 두 LICENSE 바이트 동일) · `GeneralUser-GS-2.0.1-LICENSE.txt`(sf3 **안의 INFO/ICMT 를 그대로 추출** — 재타이핑 아님) · `THIRD_PARTY_AUDIO.md`. `publish-artifact.yml` 이 셋을 **엔진 릴리스 자산으로 함께** 올린다. 프렐류드 첫 줄에도 Apache 고지 1줄.
- CI: `engine-contract.yml` 엔진 필터와 `publish-artifact.yml` `on.push.paths` 에 `wie_featurephone/src/*.js`·`*.mjs`·`scripts/build-soundfont-prelude.mjs`·`licenses/**` 를 **양쪽 같게** 더했다(Constraint 4). publish 쪽엔 `package-lock.json` 도(프렐류드 바이트가 락파일에서 나온다). ★덤: 종전에는 `audio_worklet.js` **만** 바꾼 PR 이 main 에 가도 **새 아티팩트가 안 나갔다** — 그 파일이 `include_str!` 로 wasm 에 실리는데 필터·paths 어디에도 없었다. 같이 닫힌다.
- `check-audio-worklet.mjs` 케이스 10–15(사운드폰트) + `--require-soundfont` · `--source <파일>`(변이 시험용). 사운드폰트 케이스는 루트 devDeps 가 있어야 돌고, 없으면 **`SKIP … NOT MEASURED`** 라고 말한다. always-run 스텝은 종전 그대로(케이스 1–9 · node_modules 없음) · 엔진 필터 뒤(`build-wasm.sh` 가 `npm ci` 한 뒤)에 `--require-soundfont` 스텝을 더했다.

**왜**
- 운영자 청취 판정 「B 가 낫다」(`hs:main:featurephone-soundfont-listening-ab`). 설계 정본 = `docs/report/0317` 「지연 로드 설계」 — 그대로 구현했다. 셸 쪽(sf3 호스팅 · URL 전달 · 핀 범프)은 `otterpebble-featurephone-soundfont-host-and-pin-bump`.
- **Play 마다 합성기 1개**인 이유: 핸들마다 MIDI 16채널 상태가 따로여야 하고(BGM 과 효과음이 같은 채널 번호를 쓴다), `Stop`·게임 볼륨이 핸들 단위여야 한다 — FM 경로와 같은 의미를 지키는 가장 짧은 길. 파싱된 뱅크 객체 하나를 전 합성기가 **공유**한다(node 실측: 두 프로세서가 한 뱅크로 각자 소리).
- `Stop` 은 합성기 출력을 FM 과 같은 15 ms 로 **램프 다운** 후 버린다(라이브러리의 강제 정지는 즉시 절단 = 클릭). 끝난 비반복 재생은 자연 릴리스 + 이펙트 꼬리 1 s 후 버린다.
- ★**`SF_GAIN = 2.7` — 실측으로 정했다.** 같은 이벤트에서 사운드폰트 원출력 RMS 는 FM 의 1/2.00 · 1/2.73 · 1/2.85(0317 의 세 곡, 아래 표의 `A/B직접`)다. 청취 파일도 FM RMS 에 맞춰졌으므로 이것이 «판정이 내려진 음량»이다. 중앙값 2.73 → 2.7. ★처음엔 0317 의 «1.31–1.49» 를 옮겨 1.4 로 적었는데 그것은 **피크 정규화된 wav 사이의** 이득이라 원출력엔 맞지 않았다 — 재렌더에서 FM→사운드폰트 전환이 **1.45–2.05배 조용해지는** 것으로 드러나 고쳤다.

**실측**

*3곡 전/후 재렌더* (#351 `ab.mjs` 방법 — 같은 AudioCommand 덤프 · node 오프라인 48 kHz. 도구 `ab2.mjs` 와 wav 는 `~/orchestrator/reports/evidence/wie-featurephone-soundfont-lazy-load-engine/` — 게임 음악이라 저장소 밖, Constraint 9). 덤프는 이 회차에 다시 떴다: 배틀몬스터 h1 **768 MIDI · 13,268 ms** · h6 **458 · 6,400 ms** · 더팜1 h0 **782 · 13,332 ms** — 0317 과 같은 수. A(전) RMS 도 0317 과 같다(h1 0.1186).

| 곡 | A = 전(`origin/main` 워클릿) ↔ A2 = 후(프렐류드 실림 · 사운드폰트 미수신) | RMS A · B2(후 · 사운드폰트) · B직접(0317 의 B) | A/B2 | A/B직접 | 피크 B2 · >0.9 샘플 | 포락선 상관 B2↔B · A↔B |
|---|---|---|---|---|---|---|
| 배틀몬스터 h1(27 s) | **샘플 단위 동일** | 0.1186 · 0.1102 · 0.0416 | 1.076 | 2.850 | 0.609 · 0 | **0.9954** · 0.9392 |
| 배틀몬스터 h6(13 s) | **샘플 단위 동일** | 0.1310 · 0.1703 · 0.0653 | 0.769 | 2.004 | 0.790 · 0 | **0.9990** · 0.9792 |
| 더팜1 h0(14 s) | **샘플 단위 동일** | 0.1000 · 0.0963 · 0.0366 | 1.038 | 2.734 | 0.532 · 0 | **0.9979** · 0.9190 |

- 「URL 없음 = 무변경」의 두 겹: ⑴위 A↔A2 **샘플 동일**(프렐류드가 실려도 사운드폰트가 안 오면 한 비트도 안 바뀐다) ⑵URL 없으면 Rust 가 **프렐류드를 Blob 에 싣지도 않는다** — 브라우저 실측 Blob 파트 `[23794]`(워클릿만) ↔ URL 있음 `[350041, 23794]`.
- 후(B2)는 운영자가 들은 B 와 **같은 음악**이다(20 ms 포락선 상관 0.995–0.999). 파형 상관(0.07–0.52)은 위상 민감 지표라 쓰지 않았다 — 한 블록(2.7 ms) 오프셋·코러스 LFO 위상만으로 떨어진다.
- 음량: FM 대비 0.77–1.08배 · 클리핑 0(피크 ≤0.79 · 믹스 끝 `tanh` 앞).

*브라우저 끝-끝* (Playwright chromium · 로컬 서버 · 더팜1 15 s · `browser-e2e.mjs` · 릴리스 wasm):

| 실행 | sf3 요청 | 첫 Play → GET | 워클릿 회신 | 준비 뒤 새 Play | 종료 시 워클릿 상태 | 콘솔 |
|---|---|---|---|---|---|---|
| URL 없음 | **0** | — | — | 0 | `soundfont none · synths 0` | 0 |
| URL = sf3 | 1 · 200 | 1,050 → 1,291 ms(**Play 뒤**) | `ok · 62 ms` 파싱 | **4** | `soundfont ready · synths 1 · FM voices 0` | `log: [wie] soundfont ready …` |
| URL = 404 | 1 · 404 | 1,528 → 1,636 ms | 없음 | 0 | `soundfont none` · 재생 계속 | `warn: … HTTP 404 — playing FM` · error 1 = 크롬 자신의 `Failed to load resource … 404` |

- ★**이 실행이 결함 1건을 잡았다.** 첫 판은 URL 을 줘도 **요청 0**이었다: 워클릿 준비 콜백이 `mem::replace(&mut soundfont, Posted)` 를 **무조건** 해 `NotRequested` 를 첫 Play 전에 `Posted` 로 덮었다(디버그 빌드 로그 `send play=true sf=posted`). `Arrived` 일 때만 옮기도록 고쳤다. ★**node 케이스 15개 전부 green 인 채였다** — 그 코드(`audio.rs`)는 CI 어디서도 실행되지 않는다(아래 한계).
- 파싱 시간: 브라우저 43–62 ms ↔ node vm 510–1,616 ms(부하 300–450). 샘플 Vorbis 해독은 악기 첫 사용 때 오디오 스레드에서(라이브러리 설계 · spessasynth_lib 와 같다).

*변이(폴백·배선 제거)*: `--source` 로 워클릿 변이 4개를 돌렸다 — 전부 red.
| 변이 | 결과 |
|---|---|
| M2 뱅크 없이도 사운드폰트 경로(`if (this.bank)` → `if (globalThis.wieSoundfont)`) | 케이스 10 에서 예외 · rc=1 |
| M4 MIDI 를 사운드폰트로 안 보냄 | 케이스 12 FAIL(rms 가 FM 과 같다) |
| M5 `Stop` 이 합성기를 안 끔 | 케이스 14·15 FAIL |
| M6 게임 볼륨 무시 | 케이스 15 FAIL(×1.000) |

**회귀**: `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc 0 · `cargo +beta clippy --all -- -D warnings` rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` **575 passed / 0 failed**(51 바이너리) · `npm run build:wasm` rc 0 · `check-engine-contract` **113 pass / 0**(종전 109 + marker 1 + 라이선스 3) · `contract-roundtrip` rc 0(7인자 호출 그대로 부팅) · `check-audio-worklet --require-soundfont` **15/15** · `npm run audit` PASSED · always-run node 검사 7종 rc 0.

**크기**: wasm 15,584,819(0317 기준) → **16,213,169 B**(상한 20,000,000) · 글루 35,151 → **38,683 B**(상한 40,000 — ★여유 **3.3%**). 첫 다운로드에 sf3 10,580,436 B 는 들지 않는다(첫 Play 뒤 · 셸이 URL 을 줄 때만).

**사용자 영향**: 셸이 URL 을 주는 순간부터(otterpebble 티켓) 게임 음악이 운영자가 고른 B 로 난다 — 첫 소리는 여전히 즉시(FM), 몇 초 뒤 시작하는 곡부터 사운드폰트. 셸이 아무것도 안 바꾸면 **아무것도 안 바뀐다**(7인자 · 워클릿 바이트 동일).

**한계(숨기지 않는다)**
- `audio.rs` 의 fetch·전달 배선은 **CI 에서 실행되지 않는다** — 위 결함을 잡은 브라우저 실행은 로컬(게임 파일 사용 · Constraint 9)이다. 소리 나는 커밋 픽스처 + 왕복 시나리오가 없어서다 → 후속 제안 p0.
- 글루 예산 여유 **3.3%** — 다음 바인딩 추가가 계약 PR(상한 재산정)을 강제한다. 제안으로 올리지 않았다(worklog 당 2건 상한).
- 실기기(휴대폰) 파싱·첫 악기 해독의 오디오 끊김은 재지 않았다(데스크톱 크롬만) → 제안 p1.
- `build-wasm.sh` 가 이제 node 를 요구한다(프렐류드). 모든 CI 호출처(`engine-contract`·`publish-artifact`·`web.yml`·`doc-liveness`)는 이미 node 를 셋업한다(실측).

게임 파일명 유입(`corpus-name-inflow --corpus ~/work/otterpebble/wie/game_lab`): **BOUNDED 14회/8쌍** · **SUFFIX-ATTACHED 0**(판단 필요 0) · PREFIX-EMBEDDED 1. 이 문서의 8회(이 문장 자신의 2회 포함)는 배틀몬스터·더팜1(0315·0317 에 이미 있는 이름 — 같은 세 곡의 재렌더라 이름 없이는 대조를 적을 수 없다) · 나머지 6회는 이 회차가 고친 파일(`audio.rs`·`lib.rs`·`platform.rs`)의 **기존 줄**이다(이 회차가 쓴 줄 아님).
