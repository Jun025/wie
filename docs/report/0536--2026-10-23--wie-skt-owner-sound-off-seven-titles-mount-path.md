## [2026-10-23] SKT 동봉 «소리 끔» 설정 7종 — KTF 장치를 `wie-backend` 로 옮겨 SKT 마운트 경로에도 (wie-skt-owner-sound-off-seven-titles-mount-path)

**무엇을**: KTF 의 `OWNER_SOUND_OFF_SETTINGS`(파일 빼기)·`OWNER_SOUND_OFF_BYTES`(바이트만 «켬») 목록과 두 적용 함수를 `wie-backend::owner_files` 로 옮겼다. 이제 KTF·SKT 의 `from_archive` 가 같은 목록과 같은 함수(`turn_owner_sound_on`)를 부른다. 목록의 열쇠는 jar MD5 다. MD5 는 통신사와 관계없이 한 빌드를 가리키므로, 통신사 축은 목록 안의 `// KTF`·`// SKT` 머리 주석만으로 충분하다. SKT 7행을 더했다(파일 빼기 2 · 바이트 5). `compat.json` 7행은 소리 `no → ok` 로 바꾸고 안내 문장을 뺐다. 이용자 소식 1건.

**왜**: 제안 `2026-10-21-sound-no-and-speed-unknown-remeasure#p0`(운영자 채택 2026-10-23). KTF 에만 있던 장치가 SKT 마운트 경로에는 없었다(0526 §2).

**사용자 영향**: SKT 게임 7개가 게임 안 설정을 찾지 않아도 처음부터 소리가 난다. 원 주인의 진도가 소리 설정과 같은 파일에 든 1종(`6e93f26fa2f5`)은 그 바이트만 바꾸므로 진도가 그대로 남는다.

### 1. 7종 처분과 재생 전·후

짝: release `wie_validate`, `origin/main` `fb531f06c` 빌드(전)와 이 브랜치 빌드(후). census probe A 인자(`--inject --keep-timeout --timeout 90 --max-ticks 100000000000 --relaunch 1 --pacing 8`)로 90초씩 돌렸다. 동시 실행은 3, `build-slot run --long` 임대 1건이다. 측정 중 load1 은 25~42 였다. 14회 모두 PASS · `stop` deadline · 키 27/27 이다.

| sha12 | 파일 | 처분 | 근거(javap) | 재생 전 → 후 | MIDI 후 | 그림 전 → 후 |
|---|---|---|---|---|---|---|
| `c361632541a7` | `pnjrestop` 12B | 파일 빼기 | 연습·속도·소리 옵션뿐이다. 없으면 `CGame` 이 소리 켬 기본값을 써 넣는다 | 0 → **10** | 4,839 | 712 → 1,285 |
| `7089dec0e8df` | `slot3.sav` 4B | 파일 빼기 | 옵션 4바이트뿐이다. [0] = 소리, 원 주인 0. 없으면 1 | 0 → **2** | 3,075 | 1,603 → 1,592 |
| `abc0c5789259` | `Rms` 315B | 바이트 +30 → 0 | 게임 문맥 전체가 든 파일이다. `context[30] == 0` 이면 소리 켬, 원 주인 1 | 0 → **1** | 795 | 10,319 → 10,319 |
| `6e93f26fa2f5` | `data.sav` 581B | 바이트 +569 → 1 | `GameMgr.LoadConfig` 가 568 로 seek 해 «시나리오 클리어», «소리 켬»(+569) 순으로 읽는다. 원 주인 0. 그 앞은 진도다 | 0 → **1** | 698 | 1,796 → 1,797 |
| `fb80e97cbc57` | `ds2.opt` 295B | 바이트 +0 → 1 | [0] = 소리 모드 `f.C`, 원 주인 0. 없으면 1(0517) | 0 → **5** | 10,203 | 5,297 → 4,997 |
| `ccb45e6b8d80` | `sc.txt` 71B | 바이트 +64 → `'1'` | `/` 로 나눈 숫자 33번째 값(`bp`) = 소리 켬, 원 주인 `0`(0517) | 0 → **3** | 445 | 494 → 494 |
| `f483ba078c14` | `option` 2B | 바이트 +0 → 1 | `bb` 가 [0] 을 `p.c` 로 읽고, 소리 켬 = `p.c != 3` 이다. 원 주인 3. 없으면 1 | 0 → **12** | 2,883 | 4,213 → 4,215 |

- **`6e93f26fa2f5`(진도 섞임)**: 0440 은 파일을 빼서 재생 1을 쟀다. 그 방식은 원 주인 진도까지 지운다. 이 회차에서 javap 로 소리 바이트(+569)를 짚었다. 그래서 바이트 방식으로 처분했고 진도는 그대로 올라간다. 시험 `a_file_holding_progress_is_rewritten_never_dropped` 가 이 행이 빼기 목록에 들어가는 것을 막는다.
- **`abc0c5789259`**: 0440 은 `Rms` 를 빼서 쟀다(재생 2). `Rms` 는 게임 문맥 전체(315B · 0 아닌 바이트 9개)다. 그래서 소리 바이트만 바꾸는 쪽으로 바꿨고 재생 1을 쟀다.
- **`f483ba078c14`**: 재생 측정이 없던 행이다. 이 회차에서 0 → 12 를 채웠다.
- 파일 빼기 2종은 파일이 옵션만 담고 있다(javap). 0440 이 잰 방식 그대로다.

### 2. 기존 KTF 처분 회귀

- 같은 짝(전·후 빌드 · 90초)을 KTF 4종에 돌렸다. 파일 빼기 2종(`b907b0faf483` `5a59f62d1f1a`)과 바이트 2종(`5aa438fbdc87` `6c9f969f089f`)이다. 재생은 1 → 1 · 6 → 6 · 5 → 5 · 7 → 6 이다. MIDI 는 앞의 3종이 같고, `6c9f969f089f` 만 161 → 138 이다. 넷 다 PASS 다. `6c9f969f089f` 의 차는 그림 370 → 376 과 같은 흔들림 폭이다.
- 시험: 종전 `wie-ktf` 의 세 시험(빼기 · 바이트 고정 · 다른 jar 무변경)을 `wie-backend::owner_files::tests` 로 옮겼다. 거기에 SKT 7행 고정과 진도 행 시험을 더했다. KTF 의 `DEVICE_BOUND_SAVES`·`OWNER_PROGRESS_SAVES` 는 KTF 에 남았고, 옮긴 `drop_owner_files` 를 부른다.
- `wie-ktf` 는 `md5` 를 더 쓰지 않아 의존성에서 뺐다. `wie-backend` 에 `md5` 를 더했다(워크스페이스 고정 그대로 · `no_std`).

### 3. 검증

- 4 게이트 · beta · 러너 블록: 회신 파일에 rc 를 적었다.
- `node scripts/player-data.mjs` · `node scripts/check-compat-revert.mjs` · `node scripts/check-worklog-json.mjs`: 회신 파일에 적었다.
- 측정 규율: 에뮬레이터 실행은 모두 `build-slot run --long` 임대 2건 안에서 했고, 동시 실행은 최대 3이다. `nohup` 0. 회차 끝에 내가 띄운 에뮬레이터 0.
- 게임 파일과 꺼낸 jar 는 `/tmp` 에만 있다(비커밋 · Constraint 9). 코드·문서는 타이틀을 sha12 와 jar MD5 로만 적었다.
