## [2026-09-28] 사운드폰트 전환 순간의 끊김 — Android 에뮬레이터 측정 (wie-2026-09-28-featurephone-soundfont-lazy-load-adopt-p1)

근거: worklog `2026-09-28-featurephone-soundfont-lazy-load` p1(0349 · 0355 한계 절 「실기기 크기는 모른다」). 코드 변경 0 — 측정 도구 1개(`scripts/soundfont-stall-probe.mjs`)와 판정.

**판정: 끊김이 크다.** 에뮬레이터에서 사운드폰트로 처음 쓰는 악기의 첫 소리는 FM 대조군보다 **약 200 ms 늦고**(중앙 258 ms ↔ 57 ms), 그 순간 렌더 1회가 **중앙 156 ms**(최대 605 ms) 오디오 스레드를 잡는다 — 출력 버퍼(`baseLatency` 90.8 ms)보다 길어 **소리가 끊긴다**. 티켓 기준(첫 소리 지연 > 100 ms 또는 들리는 끊김) 둘 다 넘는다. 처방 제안 1개를 worklog 에 올렸다.

**무엇을 쟀나**

`scripts/soundfont-stall-probe.mjs` 는 **출하 워클릿을 고치지 않고** 잰다: 그보다 먼저 싣는 모듈이 `registerProcessor` 를 가로채 `process`/`onMessage` 를 감싼다. 순서는 audio.rs 와 같다(워클릿 → FM 첫 Play → 프렐류드 두 번째 `addModule` → sf3 전달 → 새 Play 들). 단계마다:
- `gapMax` 렌더 양자 사이 벽시계 최대 간격 — 프렐류드 평가·파싱·합성기 생성·샘플 해독 무엇이든 오디오 스레드를 잡으면 여기 나온다
- `procMax` `process()` 1회 최대 시간 · `firstMs` Play 메시지 → 첫 비무음 샘플
- `--control` 은 같은 순서를 사운드폰트 없이(FM 만) 돌린다 — 부하 높은 호스트에서 **빼야 할 바닥값**이다

환경: AVD `galaxy-s24`(Android 15 · Chrome 124 · `bin/emu-run` 경유 · `adb reverse` 로 `http://localhost` — AudioWorklet 은 보안 컨텍스트가 필요하다) · 48 kHz · `baseLatency` 90.8 ms · `outputLatency` 104 ms. **호스트 load1 40–241**(10코어) — 에뮬레이터 수치는 이 부하를 그대로 먹는다. 그래서 짝(사운드폰트 ↔ 대조) 7쌍을 번갈아 돌렸다.

**실측 — 에뮬레이터(7쌍 · 중앙값 (범위))**

| 단계 | 사운드폰트 | FM 대조 |
|---|---|---|
| 프렐류드 적재 `gapMax` | 462 (189–3730) ms · 메인 스레드 측 814 (228–4459) ms | 169 (97–320) |
| 파싱(+첫 합성기) `gapMax` | 397 (155–3766) ms · 워클릿 자체 `ms` 382 (106–3207) | 113 (99–610) |
| 첫 사운드폰트 Play(피아노) `procMax` | **156 (42–605)** ms | 1 (1–85) |
| 〃 `firstMs` | **258 (44–601)** ms | 57 (12–150) |
| 새 악기(현악) `procMax` / `firstMs` | 55 (19–142) / 124 (73–187) | 1 / 70 |
| 같은 악기 재사용(피아노 again) `procMax` / `firstMs` | 20 (2–472) / 60 (9–508) | 1 / 46 |

- 비용은 **악기 첫 사용**에 몰린다: 같은 피아노를 다시 쓰면 `procMax` 가 156 → 20 으로 떨어진다(해독된 샘플이 남는다). 새 악기(현악)는 다시 오른다. spessasynth 가 sf3(Vorbis) 샘플을 **처음 쓸 때 오디오 스레드에서 해독**하는 설계 그대로다.
- 프렐류드·파싱은 세션당 1회지만 에뮬레이터에서 **0.1–3.7 s** 오디오 스레드를 잡는다 — 그동안 이미 울리던 FM 배경음도 멈춘다.
- `lost`(컨텍스트 시계 ↔ 벽시계 차이) 열은 에뮬레이터(`-no-audio`)에서 초 단위로 양쪽으로 흔들려(대조군도 −3111 ~ 23526) **판정에 쓰지 않았다**. 판정은 `procMax` > `baseLatency` 로 했다: 렌더 1회가 출력 버퍼 전체보다 길면 그 사이 채울 샘플이 없다.
- 첫 회차(load 118 · 부팅 직후)에 Chrome 본 프로세스가 한 번 죽었다(`Process com.android.chrome … has died: fg TOP` · 원인 줄 없음). 그 회차 3·4번은 버리고 1쌍씩 나눠 다시 돌렸다.

**대조 — 데스크톱 헤드리스 Chromium(3쌍)**: 첫 사운드폰트 피아노 `procMax` 9 ms(대조 1) · `firstMs` 14(대조 3) · 파싱 37 ms · 프렐류드 `gapMax` 25 ms. 에뮬레이터가 **약 15배** 느리다 — 같은 코드가 데스크톱에서는 기준 아래다.

**«미리 해독»의 메모리 비용**(node · `spessasynth_core` 4.3.18 · `GeneralUser.sf3`)
- 전 샘플 해독 = **61.1 MB**(Float32) · 해독 610 ms(호스트 부하 상태) — 사운드폰트 도착 순간에 하면 지금의 파싱 끊김보다 긴 끊김을 한 번 더 만든다.
- 악기별(그 프리셋의 전 샘플 · 상한): 피아노 4.14 MB · 현악 4.68 · 트럼펫 0.28 · 플루트 1.54.
- ★지금의 지연 해독도 **한 번 해독한 것은 남긴다** ⇒ «쓸 악기만 미리» 해독하면 메모리는 **지금과 같고** 시점만 바뀐다. 61 MB 는 «쓰지 않을 악기까지»의 값이다.

**처방 제안(worklog p0)**: 해독 안 된 악기가 필요한 새 Play 는 **FM 으로 내고**, 그 악기 샘플을 양자마다 조금씩(시간 예산 안에서) 해독해 **다음 Play 부터** 사운드폰트로 — 이미 있는 «재생 중에는 바꾸지 않는다» 규칙과 같은 모양이다. 메모리는 지금과 같다(쓰는 악기만).

**한계**
- 실기 미검증: Galaxy S24 실기(Chrome) · iOS Safari. 에뮬레이터는 Apple Silicon 호스트 위 arm64 가상화라 **실기의 CPU 가 아니다** — 호스트 부하(40–241)가 수치를 부풀린 쪽이다. 방향(첫 사용 해독이 끊김의 주원인)은 데스크톱에서도 같은 모양(9 ms ↔ 1 ms)으로 보인다.
- Date.now() 는 1 ms 해상도다. 짧은 값(≤ 3 ms)은 읽지 마라.
- 이 도구는 **로컬 전용**이다(에뮬레이터 · 헤드리스 둘 다 CI 밖). 게임 파일은 쓰지 않는다.

재현: `npm ci && node scripts/build-soundfont-prelude.mjs && node scripts/soundfont-stall-probe.mjs --runs 3 --control`(데스크톱) · 에뮬레이터는 도구 머리주석. 원 로그: `~/orchestrator/reports/evidence/wie-2026-09-28-featurephone-soundfont-lazy-load-adopt-p1/`.

게임 파일명 유입: BOUNDED 0 · SUFFIX-ATTACHED 0 (`scripts/corpus-name-inflow.mjs`).

<!-- corpus-name-inflow v1 subjects=3 tree=507f1a7c42c0da88 B=0/0 P=0/0 S=0/0 -->
