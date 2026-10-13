## [2026-10-13] KTF 동봉 «소리 끔» 설정 2종을 `OWNER_SOUND_OFF_SETTINGS` 로 — 처음부터 소리가 난다 (wie-ktf-owner-sound-off-settings-two-titles)

**무엇을**: `wie-ktf` `OWNER_SOUND_OFF_SETTINGS` 에 2행을 더했다(0417·0420·0440 과 같은 목록·같은 함수 · jar MD5).
- `b907b0faf483` — `config.sav`(6바이트 · `P/` 아래 1개).
- `5a59f62d1f1a` — `config.dat`(4바이트 · 같은 내용이 맨 이름과 `P/` 아래 2벌 · 둘 다 빠진다).

`compat.json` 2행: `sound no → ok`, 안내 «게임 파일에 소리를 끈 설정이 들어 있어요. …» 제거. 다른 축은 손대지 않았다. 이용자 소식 1건.

**왜**: 운영자 Tower 채택 제안 `2026-10-13-playable-sound-no-eleven-unexplained#p0`. #550(0517)은 이 2종에 안내 문장만 달았다.

**사용자 영향**: 두 게임이 게임 안 설정을 찾아 들어가지 않아도 처음부터 소리가 난다. 대가: 그 파일의 다른 설정도 처음 설치한 값이 된다(아래 §2 · 판정 불변).

### 1. «파일째 빼기» 짝 (Acceptance 1)

- 방법: 원본 zip 과, 그 파일만 뺀 사본(`zip -d` · 비커밋)을 같은 `origin/main` `435375c0` release `wie_validate` 로 census `run` 한 번에(jobs 3 · 같은 시각) 쟀다.
- `5a59f62d1f1a` 는 0517 이 «첫 바이트 3» 만 쟀고 빼기 짝은 이번이 처음이다. **빼도 난다** ⇒ 편입했다.

| sha12 | 측정 | 원본 | 뺀 사본 |
|---|---|---|---|
| `5a59f62d1f1a` | probe A 120초 재생 · MIDI | 0 · 0 | 6 · 4,152 |
| | probe B(키 0) 120초 | 0 · 0 | 1 · 2,081 |
| | long 600초 | 0 · 0 | 453 · 43,947 |
| `b907b0faf483` | probe A 120초 | 0 · 0 | 1 · 130 |
| | probe B 120초 | 0 · 0 | 1 · 130 |
| | long 600초 | 0 · 0 | 278 · 130 |

### 2. 대가 — 판정 전/후 (Acceptance 2)

census `report` 판정(위 같은 실행 + `--only long --long 600` + `--only progress --progress 600`):

| sha12 | boot | render | input | longplay | sound | speed(ratio) | progress(p_stall · p_distinct) |
|---|---|---|---|---|---|---|---|
| `5a59f62d1f1a` 원본 | ok | ok | ok | ok | silent | ok(0.918) | ok(140 · 14) |
| `5a59f62d1f1a` 뺀 사본 | ok | ok | ok | ok | **ok** | ok(0.923) | ok(70 · 14) |
| `b907b0faf483` 원본 | ok | ok | ok | ok | silent | ok(0.930) | n/a(560 · 2) |
| `b907b0faf483` 뺀 사본 | ok | ok | ok | ok | **ok** | n/a(0.900) | ok(0 · 8) |

- boot·render·input·longplay·progress 퇴행 0. probe A 그림 수·색 수: 959 → 1,061 · 176 → 176 / 1,187 → 1,182 · 234 → 234.
- `b907b0faf483` speed `n/a` 는 문턱(0.9) 에 걸친 한 번 값이다(그 짝 load1 27 · 원본 16). 같은 시각 짝 3회로 다시 쟀다: 원본 아카이브를 전 = `435375c0` · 후 = 이 브랜치 빌드로 동시에, probe A 30초(census 인자 그대로): 전 0.994 · 0.994 · 0.992 / 후 0.995 · 0.995 · 0.994. ⇒ 부하이고 퇴행이 아니다.
- progress 는 `b907b0faf483` 쪽이 오히려 나아졌다(600초 정체 560 → 0). compat 의 `progress: stuck` 은 티켓 범위(소리 축만) 밖이라 그대로 뒀다.
- `5a59f62d1f1a` 뺀 사본은 db 쓰기 3(원본 2) — 파일이 없으면 게임이 자기 설정을 쓴다(0440 `db8ef04a6504` 과 같은 모양).

### 3. 후 빌드로 원본 아카이브 (끝단 확인)

- 이 브랜치 release `wie_validate` · 원본 zip · probe A 인자 · 120초 · `RUST_LOG=info`:
  - `5a59f62d1f1a`: `Not mounting the original owner's config.dat` · `… P/config.dat` · 재생 6 · MIDI 4,152(뺀 사본과 같다).
  - `b907b0faf483`: `… P/config.sav` · 재생 1 · MIDI 130(뺀 사본과 같다).
- ★첫 판은 `b907b0faf483` 의 jar MD5 를 바이트 열로 옮기다 3바이트를 틀려 **빠지지 않았다**(재생 0). 이 끝단 확인이 잡았다. 그래서 시험은 MD5 를 `md5 -q` 출력 문자열 그대로 받아 표의 바이트 열과 대조한다 — 그 오타를 다시 넣으면 시험이 red(`left: None` · 실측).

### 4. 검증

- 되돌리면 red: `emulator::tests::owner_sound_off_settings_cover_the_two_0519_titles`. `b907b0faf483` 행을 빼면 `left: None`(실측). 게임 jar 는 커밋할 수 없어(Constraint 9) 빠지는 동작이 아니라 행을 못박는다 — 동작은 `device_bound_saves_dropped_for_listed_jar_only` 가 이미 본다.
- 4 게이트: `cargo fmt --all -- --check` rc 0 · `cargo clippy --all -- -D warnings` rc 0 · wasm32 rc 0 · `+beta` rc 0(기존 unused-dependency 경고 9줄) · `RUST_MIN_STACK=4194304 cargo test --all` **783 passed / 0 failed**.
- 러너 블록: draw · helloworld ×2 · text PASS · keydraw ×2 `--inject --expect-last-frame` PASS · rc 0.
- `node scripts/player-data.mjs` · `node scripts/check-compat-revert.mjs` — 회신에 인용.
- 측정 규율: 에뮬레이터는 전부 `build-slot run` 안(스윕은 `census-drive.sh` phase 임대) · 동시 최대 3 · `nohup` 0 · 회차 끝 내가 띄운 에뮬레이터 0. 다른 레인 census 가 호스트 락을 잡고 있어 speed 재측은 census 대신 단발 2개 동시(`build-slot run`)로 했다.

### 5. 되돌리기

두 행을 빼면 §1 의 «원본» 열로 돌아간다. compat 2행은 `sound: no` + 안내 문장으로.

### 6. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 718회 / 330쌍 + SUFFIX-ATTACHED 36회 / 16쌍. 전부 `docs/player-data/compat.json` 의 기존 `title`·`fileTitle` 값이다(이 회차는 그 2행의 소리 축·안내 줄만 바꿨다). 그 밖 파일은 0 이다. 코드·회차 문서·worklog·소식은 타이틀을 sha12/sha256 으로만 적었다.
