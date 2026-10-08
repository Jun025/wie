## [2026-10-08] KTF 현 핀 600초 P·P2 재측 — 막힘 38종 (wie-ktf-current-pin-600s-remeasure-38)

**무엇을**: `docs/report/0470` §3 에서 막힘(`progress: stuck`)으로 남은 KTF 38종(⒟ 판정 보류 35 + 진도 4차에서 고친 3)을 현 main 핀으로 정책 v2 600초 P·P2 로 다시 쟀다. 엔진은 고치지 않았다.

**왜**: 운영자가 채택한 후속 제안 `2026-10-07-progression-wave4-ktf-followups#p0`. ⒟ 35종은 짧은 재현만 있어 계급이 없었다. 고친 3종의 600초 판정도 없었다(0470 §1·§7).

**사용자 영향**:
- KTF 14종의 진도 배지가 «막힘»에서 «진행됨»으로 바뀐다.
- 진도 4차에서 고친 이름 칸·프로필 입력 2종이 여기에 든다. 셋째(두뇌 게임)는 아직 막힘이다.
- 나머지 24종은 현 핀에서도 막힘이다. 표시는 그대로다.

### 1. 측정 조건

- 엔진: `origin/main` `60e9062c` 의 release `wie_validate`.
- 도구: `scripts/playability-census.mjs` (같은 커밋) `run --only progress --progress 600 --titles <38종>`. 정책 v2(정체 60초 탈출 키 · `--restart-at 600` 뒤 이어하기 120초 · `--relaunch 8`).
- P: 38종 전부. P2(짝 재측): P 가 stuck 인 25종만. census 판정 규칙(`progressAxis`) 그대로다 — P ok 면 ok, P stuck 이면 P2 가 정한다.
- A·B·L(진도 후보 조건)은 0470 회차 out(`f5c02609` 핀)의 것을 옮겨 썼다. 38종 모두 boot·render·longplay ok 다.
- 자원: 전체를 `build-slot run --long` 임대 1개로 감쌌다. 그 안에서 census 호스트 잠금 1개 · `--jobs 3`. 단계 시작 전 `host-load-guard --status --recovered` rc=0(두 번 다 첫 확인에서 통과).
- 시간:
  - long 임대 요청 09:15:45 → 획득 09:15:46(대기 0초). census 잠금 대기 0.
  - P 09:15:47–11:51:57(2시간 36분) · P2 11:55:59–13:44:04(1시간 48분). 임대 해제 13:44:06.
  - ⇒ **다른 레인 스윕이 기다렸을 수 있는 시간 = long 슬롯·census 잠금 점유 4시간 28분**.
- 부하: 타이틀별 load1 중앙 10 · 최소 6 · 최대 123(P 의 한 묶음 `155586ece7f8` `174237758542` `182fa44210dc` 이 load1 123 에서 돌았다). 그 셋의 정체 길이는 옛 핀 값과 초 단위로 같아 부하 탓으로 보지 않는다. 그래도 다시 볼 때는 그 셋을 먼저 의심하라.
- `nohup` 0. 끝난 뒤 `wv_60e9062c` 프로세스 0 을 확인했다.

### 2. 전/후 판정

- «전» 판정은 `compat.json` 의 `axes.progress` 이고 38종 모두 `stuck` 이다.
- «구 P / P2 정체»는 로컬에 남은 가장 최근 옛 핀 측정의 정체 길이(초)다. 출처는 `w7prog`·`wie-progress-2026-09-30`·`-10-02`·`w6census` out 이고 정책은 v1 10 · v2 28 이다. compat 의 값이 아닌 것도 섞였다(`4df05a4dc452` `65bace1623a7` `6af589a88cf9` 는 옛 P 가 ok 였는데 compat 은 stuck · `4decaeed58b1` 은 옛 P 가 error).
- 정체 = 마지막 새 화면부터 600초 끝까지. stuck 기준은 `max(180, 600/3)` = 200초다.

| sha12 | 0470 계급 | 구 P / P2 정체 | 현 P / P2 정체 | 현 판정 | P load1 |
|---|---|---|---|---|---|
| `09a6a300994d` | ⒝ 고침 | 550 / 550 | 70 / — | **ok** | 14 |
| `0c67145b11df` | ⒝ 고침 | 590 / 590 | 120 / — | **ok** | 14 |
| `0e72b6bc12bb` | ⒟ | 540 / 540 | 10 / — | **ok** | 8 |
| `4decaeed58b1` | ⒟ | error / error | 0 / — | **ok** | 15 |
| `4df05a4dc452` | ⒟ | 60 / — | 60 / — | **ok** | 8 |
| `5a0fa312a793` | ⒟ | 250 / 260 | 100 / — | **ok** | 8 |
| `5d3ba49eccf7` | ⒟ | 590 / 590 | 30 / — | **ok** | 12 |
| `65bace1623a7` | ⒟ | 170 / — | 360 / 170 | **ok** | 12 |
| `75e6050fe272` | ⒟ | 360 / 310 | 80 / — | **ok** | 21 |
| `75f002ba70e3` | ⒟ | 590 / 590 | 110 / — | **ok** | 21 |
| `ab64a56b2b44` | ⒟ | 260 / 260 | 110 / — | **ok** | 60 |
| `e538cbb4e687` | ⒟ | 590 / 590 | 70 / — | **ok** | 20 |
| `f12d97040c33` | ⒟ | 590 / 590 | 0 / — | **ok** | 7 |
| `f7752f9124e8` | ⒟ | 540 / 540 | 20 / — | **ok** | 7 |
| `0cc4ef7ede37` | ⒟ | 540 / 540 | 540 / 540 | stuck | 14 |
| `135d1291501f` | ⒟ | 530 / 530 | 530 / 530 | stuck | 8 |
| `145f760b2f4a` | ⒟ | 280 / 280 | 280 / 280 | stuck | 8 |
| `155586ece7f8` | ⒟ | 410 / 410 | 410 / 410 | stuck | 123 |
| `174237758542` | ⒟ | 320 / 320 | 320 / 370 | stuck | 123 |
| `182fa44210dc` | ⒟ | 580 / 580 | 580 / 580 | stuck | 123 |
| `33801c1ba14f` | ⒟ | 470 / 470 | 470 / 470 | stuck | 7 |
| `33f3e7669599` | ⒝ 고침 | 300 / 300 | 240 / 240 | stuck | 7 |
| `36acdf213c33` | ⒟ | 210 / 350 | 350 / 210 | stuck | 7 |
| `44c292be4e4c` | ⒟ | 260 / 260 | 260 / 260 | stuck | 15 |
| `46b2238f87a6` | ⒟ | 420 / 420 | 420 / 420 | stuck | 15 |
| `4fcd4b74020e` | ⒟ | 290 / 290 | 290 / 290 | stuck | 8 |
| `6af589a88cf9` | ⒟ | 100 / — | 300 / 300 | stuck | 12 |
| `6d63b4025bef` | ⒟ | 220 / 220 | 220 / 220 | stuck | 21 |
| `859aa864797b` | ⒟ | 450 / 450 | 450 / 450 | stuck | 30 |
| `8a33aafc06a2` | ⒟ | 420 / 420 | 420 / 420 | stuck | 30 |
| `8d2828ab8a3f` | ⒟ | 420 / 420 | 420 / 420 | stuck | 30 |
| `965eee81e442` | ⒟ | 520 / 520 | 520 / 520 | stuck | 60 |
| `b1ec149b354c` | ⒟ | 440 / 540 | 550 / 540 | stuck | 60 |
| `c181cae84146` | ⒟ | 420 / 420 | 420 / 420 | stuck | 17 |
| `cb7c7f87f9e6` | ⒟ | 400 / 400 | 390 / 400 | stuck | 17 |
| `d448aee68157` | ⒟ | 550 / 540 | 340 / 200 | stuck | 17 |
| `d9384b388ea5` | ⒟ | 430 / 430 | 430 / 310 | stuck | 20 |
| `ed6ad7318ac9` | ⒟ | 290 / 450 | 280 / 310 | stuck | 20 |

- 합계: **ok 14 · stuck 24**. 전 38 stuck → 후 14 ok. ok 14 = ⒟ 12 + 고친 3종 중 2.
- P2 가 판정을 바꾼 것은 `65bace1623a7` 1종이다(P 360초 stuck → P2 170초 ok).
- `33f3e7669599`(고친 3종의 셋째)는 P 240 · P2 240 으로 stuck 이다. 0470 §4-3 의 `timeZone` NPE 는 0 이고(P 첫 예외는 `RecordStoreNotFoundException`), 새 화면 수는 구 6 → 현 23 이다. 더 가긴 하지만 마지막 3분의 1 에 새 화면이 없다.
- 판정이 같은 24종 가운데 17종은 P 정체가 옛 핀 값과 초 단위로 같다. 막힌 자리가 이 사이 엔진 변경과 무관하다는 뜻이다.
- ★ok 의 한계(census 머리 주석의 ponytail 줄 그대로): 화면 새로움은 «커서만 움직이는 판»을 진행과 가르지 못한다. 정체 0초인 `f12d97040c33`(맞고 · 새 화면 53)과 `4decaeed58b1` 은 그 모양일 수 있다. 마지막 화면은 이 회차에 보지 않았다.

### 3. stuck 24 — 엔진 벽 관측

- P stderr 를 봤다. 게임 스레드 사망 · `panicked` · `Invalid memory access` 는 0 이다.
- 첫 예외는 아래 셋이다. 모두 엔진 벽의 근거로 보지 않는다.
  - `RecordStoreNotFoundException` 4종: 첫 실행에 저장이 없다.
  - `FileNotFoundException` 3종: `135d1291501f` 의 `/bg_wa1.png` 는 jar 에 원래 없다(0470 §3).
  - `StringIndexOutOfBoundsException` 1종(`36acdf213c33`): 0470 에서도 1회였다. 진행은 계속된다.
- `Unknown WIPICX_incMemInterface` 1~2회가 5종에 있다(`174237758542` `4fcd4b74020e` `6af589a88cf9` `d9384b388ea5` `ed6ad7318ac9`). 같은 문자열이 ok 판에도 있는지는 대조하지 않았다.
- ⇒ 이 회차에 새 엔진 벽을 찾지 못했다. stuck 24 를 ⒜(키 루프 한계)·⒝ 로 다시 나누려면 마지막 화면을 봐야 한다. 이 회차 범위(측정만) 밖이라 제안 카드도 만들지 않았다(문턱: «관측»이 없다 — worklog `notCarded`).

### 4. compat.json

- 이 브랜치의 `compat.json`(= `origin/main` `60e9062c` 의 파일) 위에 위 14행의 `axes.progress` 만 `stuck` → `ok` 로 바꿨다. 다른 키·행은 그대로다. 직렬화는 원본과 같다(들여쓰기 1칸).
- 고친 3종은 엔진이 바뀐 회차(#503)가 이미 `docs/player-updates/2026-10-07-ktf-input-forms-pass.json` 을 냈다. 이 회차는 게임 동작을 바꾸지 않아 소식 파일을 더하지 않았다.
- `node scripts/check-compat-revert.mjs --head HEAD --base origin/main`: «compat-revert: OK — 브랜치가 main 을 받은 적이 없다 · 착지 기준 바뀐 행 14».

### 5. 게이트

- `cargo fmt --check` rc=0 · `cargo clippy --all -D warnings` rc=0 · wasm32 rc=0 · `+beta` rc=0.
  - `+beta` 는 `wie_cli` 에 «unused dependency» 경고를 낸다(`directories` `midir` `rodio` 등). rc 는 0 이다. 이 회차가 건드린 코드는 없다.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 739 passed / 0 failed.
- `node scripts/player-data.mjs` OK · `check-worklog-json.mjs` OK · `npm run audit` PASSED.
- 러너 블록은 돌리지 않았다. 엔진 코드 변경이 0 이기 때문이다.

### 6. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치 ↔ `origin/main`): BOUNDED 718회/330쌍 + SUFFIX-ATTACHED 35회/15쌍.
- 전부 `compat.json` 의 기존 `title` 값이다. 이 회차는 그 파일의 `axes.progress` 14값만 바꿨다.
- 이 문서와 worklog 는 타이틀을 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=3 tree=893d1a23368ca2ca B=718/330 P=0/0 S=35/15 -->
