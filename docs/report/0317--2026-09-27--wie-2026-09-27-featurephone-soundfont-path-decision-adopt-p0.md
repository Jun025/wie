## [2026-09-27] featurephone 음질 — 사운드폰트 경로(spessasynth + GeneralUser GS) 도입 판정 (wie-2026-09-27-featurephone-soundfont-path-decision-adopt-p0)

**판정: 보류** — 라이선스·CPU 는 막지 않는다. 남은 결정축은 «실제로 더 낫게(원래 폰에 가깝게) 들리는가»이고 그것은 청취 판단이다. 그 답이 «예»일 때만 대가(첫 실행 +10.6MB · wie+otterpebble 계약 동시 변경)를 치를 값이 있다. 아래 녹음 3쌍이 그 판단 자료다.

**무엇을**: 판정 자료(라이선스 원문 · 같은 곡 녹음 대조 · 지연 로드 설계 · 셸 변경 목록). 코드 변경은 `wie_validate` 의 `WIE_AUDIO_DUMP=<file>`(엔진이 보낸 AudioCommand 를 JSON 줄로 남김) 1건 — 녹음 대조를 다시 만들 수 있게.

**왜**: worklog `2026-09-27-featurephone-audio-midi-bgm#p0` 채택.

**사용자 영향**: 없음(소리 경로 무변경).

### 1. 라이선스 — 원문

| 대상 | 판본 | 라이선스 | 근거 |
|---|---|---|---|
| spessasynth_lib | 4.3.14 (upstream `package-lock.json` 핀) | **Apache-2.0** | npm tarball `package/LICENSE` 전문 · `package.json` `"license": "Apache-2.0"` |
| spessasynth_core (의존) | 4.3.18 | **Apache-2.0** | 같은 방식 |
| stb-vorbis (core 의존 · sf3 복호) | 0.0.6 | **Apache-2.0** | 같은 방식 |
| GeneralUser GS | 2.0.1 (`wie-web/public/GeneralUser.sf3` · 10,580,436 B · upstream 과 동일 blob) | 자체 License v2.0 | sf3 INFO `ICMT`(아래) · `ICOP` 「1997-2024 by S. Christian Collins」 |

- Apache-2.0 §2 「reproduce … publicly display, publicly perform, sublicense, and distribute the Work」 · §4(a) 「You must give any other recipients of the Work … a copy of this License」 ⇒ 재배포·상업 이용 가능, **라이선스 사본 동봉 의무**. 세 패키지 모두 NOTICE 파일 없음(tarball 실측).
- GeneralUser GS `ICMT` 원문: 「You may use GeneralUser GS without restriction for your own music creation, private or commercial. … Please feel free to use it in your software projects, and to modify the SoundFont bank or its packaging to suit your needs.」 · 「If you plan to feature GeneralUser GS on your own website, please do not link directly to my download files. Either link to my website, or provide your own local copy instead.」 ⇒ **자체 호스팅 사본이면 허용.**
- ★단서(같은 원문): 「I cannot be 100% sure where all of the samples originated … This uncertainty may concern you if you intend to use GeneralUser GS in a commercial software product.」 — 막는 조항은 아니고 표본 출처 보증이 없다는 고지다.

### 2. 녹음 대조 — 같은 이벤트, 두 합성기

엔진이 실제로 보낸 명령을 `WIE_AUDIO_DUMP` 로 받고(릴리스 빌드 `wie_validate --inject`), 같은 MIDI 이벤트를 ⒜ `audio_worklet.js`(현행 · node 에서 check-audio-worklet 과 같은 스텁 전역) ⒝ `spessasynth_core` 4.3.18 `SpessaSynthProcessor` + `GeneralUser.sf3` 로 오프라인 렌더했다. 48 kHz 스테레오. 파일은 게임 음악이라 저장소 밖에 둔다(Constraint 9):

`~/orchestrator/reports/evidence/wie-2026-09-27-featurephone-soundfont-path-decision-adopt-p0/`

| 곡 | 이벤트 | 길이 | A 현행 FM | B 사운드폰트 | B 음량 맞춤 |
|---|---|---|---|---|---|
| 배틀몬스터(LGT) BGM h1 · `docs/keys/battlemonster-village.keys` | MIDI 768 · 루프 13,268 ms | 27 s(2회) | `battlemonster-bgm-h1-A-wie-fm.wav` | `…-B-soundfont.wav` | `…-B-soundfont-loudness-matched.wav` |
| 배틀몬스터 BGM h6(마을 후반) | MIDI 458 · 루프 6,400 ms | 13 s | `battlemonster-bgm-h6-A-wie-fm.wav` | `…-B-soundfont.wav` | `…-loudness-matched.wav` |
| SKT 더팜1 h0 | MIDI 782 · 13,332 ms | 14 s | `skt-thefarm1-bgm-h0-A-wie-fm.wav` | `…-B-soundfont.wav` | `…-loudness-matched.wav` |

- A·B 원본 wav 는 둘 다 피크 정규화(`audioToWav` 기본값). 정규화 전 RMS 는 FM 이 2.0–2.9배 컸다(0.1186 vs 0.0416 등) ⇒ 청취 비교는 **RMS 를 A 에 맞춘 `loudness-matched`** 로 하라(이득 1.31–1.49). 렌더 스크립트 `ab.mjs`·`match.mjs` 도 같은 디렉터리.
- ★**SMAF 원음과의 거리 — 관측 1건**: 배틀몬스터 h0(효과음 징글)은 11채널이 **전부 program 36**(GM Slap Bass 1)이다. 악기 선택은 두 합성기 모두 `smaf_player` 의 SMAF 음색 → GM 매핑이 정한다. 샘플 음원은 그 매핑의 오차를 더 «그럴듯한 다른 악기»로 들려준다 — 사운드폰트를 넣으면 **매핑 품질이 음질의 병목**이 된다.
- CPU(같은 조건 교대 3회 · 13 s 곡 · loadavg 273–281 · idle 0%): FM 10.2–12.9 s ↔ 사운드폰트 4.3–5.2 s ⇒ **사운드폰트가 2.4–2.9배 가볍다.** 절대값은 부하 때문에 의미 없고 비만 읽어라. CPU 는 반대 논거가 되지 못한다.

### 3. 도입한다면 — 지연 로드 설계(구현 안 함)

- **첫 소리 지연 0**: 시작은 지금 FM 워클릿. 첫 `Play` 후(=사용자 제스처로 `AudioContext` 가 풀린 뒤) sf3 를 백그라운드로 받고, 준비되면 **다음 새 핸들의 `Play` 부터** 사운드폰트로(곡 중간 음색 전환 금지). 실패·미제공이면 FM 유지 = 현행과 동일(기본 끔과 같은 효과).
- 사운드폰트 음색이 들리기까지: 10,580,436 B ⇒ 10 Mbps **8.5 s** · 50 Mbps **1.7 s**(gzip 서빙 시 6,939,797 B = 5.6 s · 1.1 s — sf3 는 이미 vorbis 압축이라 34%만 준다) + 파싱 0.18–1.0 s(node · 고부하 실측). 브라우저 실측 아님.
- **캐시**: 판본 박은 파일명 + `Cache-Control: public, max-age=31536000, immutable` ⇒ 두 번째 실행부터 다운로드 0.
- **코드 형태**: `spessasynth_lib` 의 `index.js` 는 bare specifier `from "spessasynth_core"` 를 import 한다(실측) ⇒ 번들러 없는 셸은 그대로 못 쓴다. `spessasynth_processor.min.js`(402,596 B · gzip 135,312 B)는 import 0 이지만 lib 내부 메시지 규약에 묶인다. ⇒ **spessasynth_core 를 우리 워클릿에 번들**(esbuild)해 지금처럼 wasm 안 문자열로 싣는 것이 계약 변경이 가장 작다: 코드 파일은 2개 그대로, 새 자산은 **sf3 1개**. wasm 약 +0.4 MB(상한 20,000,000 · 현재 약 15.56 MB).

### 4. 셸(otterpebble featurephone) 변경 목록 — 도입 시(별 티켓)

1. `GeneralUser-2.0.1.sf3` 정적 호스팅(판본 박은 이름 · immutable 캐시 헤더). 원 배포처 직링크 금지(라이선스 원문).
2. 엔진에 sf3 URL 전달 — 새 선택 인자 1개(없으면 FM). 계약 JSON 의 `constructorShape`/`methods` 와 동시 변경.
3. 라이선스 고지 화면: GeneralUser GS License v2.0 원문 + Apache-2.0 사본(spessasynth_core · stb-vorbis) — Apache §4(a) 의무.
4. 첫 실행 예산 문서화(셸 쪽 다운로드 예산에 +10.6 MB, 지연 로드).
5. wie 핀 범프 + `apps/featurephone/CLAUDE.md` 소리 절 갱신.

엔진(wie) 쪽: `docs/contracts/featurephone-engine-contract.json` 에 선택 자산 1종 · `scripts/build-wasm.sh` 에 워클릿 번들 단계(node devDependency `spessasynth_core`·`esbuild` 추가) · `check-audio-worklet.mjs` 에 사운드폰트 경로 케이스.

### 5. 도입으로 넘길 조건

운영자가 `loudness-matched` 3쌍을 듣고 «B 가 낫다»고 하면 → 위 3·4 로 wie+otterpebble 티켓 2건. «비슷하거나 A 가 폰에 가깝다»면 → 거부로 닫고, 음질 투자는 FM 음색 개선(SMAF 음색 파라미터 해석)으로.

### 회귀

`WIE_AUDIO_DUMP` 미설정이면 `dump_audio` 는 env 조회 1회 후 반환 — 판정·출력 무변경. 시험 `audio_dump_line_keeps_every_event_in_worklet_shape`(형식이 바뀌면 red). `cargo fmt --check` · `clippy --all -D warnings`(stable · wasm32 · beta) rc 0 · `RUST_MIN_STACK=4194304 cargo test --all` **508 passed / 0 failed** · `npm run build:wasm` rc 0(글루 35,151 B · wasm 15,584,819 B) · `check-engine-contract` **109 pass / 0 violation**.

게임 파일명 유입(`corpus-name-inflow --corpus ~/work/otterpebble/wie/game_lab`): BOUNDED 10회/6쌍 · SUFFIX-ATTACHED 0 — 이 회차 문서의 타이틀명 8(배틀몬스터·더팜1, 0315 에 이미 있는 이름) + `wie_validate.rs` 기존 줄 2(수정 파일이라 대상에 든 것 · 이 회차가 쓴 줄 아님).

<!-- corpus-name-inflow v1 subjects=3 tree=a6a10563e4276783 B=10/6 P=0/0 S=0/0 -->
