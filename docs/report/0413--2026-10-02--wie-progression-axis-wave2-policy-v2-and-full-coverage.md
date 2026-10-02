## [2026-10-02] 진도 정책 v2 · 세이브 이어하기 계측 · 1000초 넘는 실행의 정렬 결함 · 대표 60분 (wie-progression-axis-wave2-policy-v2-and-full-coverage)

**무엇을**: 진도 축(0393) 2차다. ① `wie_validate` 에 정체 뒤에만 탈출 키를 섞는 `--stall-secs`/`--stall-keys` 를 넣었다. 저장을 유지한 채 다시 켜는 `--restart-at` 와 DB 계수(`db`)도 넣었다. ② census `--only progress` 를 그 정책(v2)으로 바꿨다. ③ 1차 ⒜ 35종, ⒝?·미조사 16종, 대표 8종(60분)을 v2 로 다시 쟀다. ④ 곡선 정렬 결함을 고쳤다. 이 결함 때문에 1차의 30분 «ok» 다섯 건이 거짓이었다.
**왜**: 운영자 지시(2026-09-30 · 10-02) — 「플레이 가능한 게임을 수십 분 이상 해 보고 계속 진도가 나가는지 검토하라」. 1차 확정 stuck 82 가운데 최대 군집이 ⒜ 정책 35였다. 대부분 «뒤로 가기 없음»이었다.
**사용자 영향**: 엔진 동작 변경 0(도구만). 지원 현황의 «오래 해 봤어요» 칸이 18종 `stuck → ok` 으로 바뀌었다. 거짓 `ok` 1종(`4ece6eeeaa04`)은 칸을 내렸다(아래 §3).

증적: 프레임·전사는 저장소 밖(`~/scratch/wie-progress-2026-10-02/`)에만 있다. 여기에는 sha12 만 적는다.

### 1. 정책 v2 — 무엇이 달라졌나

| 항목 | v1(0393) | v2 |
|---|---|---|
| 기본 키 | 확인·방향·5 연타 ~30초 주기 | 같다 |
| 탈출 | 없음(CLR·오른쪽 소프트키 금지) | **새 화면이 60초 없을 때만** 다음 중 하나를 돌아가며 끼운다: `CLR` · `RSOFT` · `DOWN OK` · `CLR CLR` · `DOWN DOWN OK` · `UP OK` · `NUM0 HASH STAR`. 끼우는 동안 나머지 키는 뒤로 민다. 키가 눌린 동안은 끼우지 않는다 |
| 재실행 | 꺼지면 3회 | 8회(탈출 키로 꺼지는 제목) |
| 세이브 | 안 잼 | 진도 창(600/3600초)이 끝나면 **저장을 유지한 채 다시 켜고** 120초 더 돈다. `db.resumed_reads` 는 «첫 키 이후 쓴 기록을 다음 부팅이 읽은 횟수»다 |
| 판정 | 마지막 1/3 동안 새 화면 0 → stuck | 같다. 재시작 뒤 120초는 곡선에 넣지 않는다 |

«새 화면»은 census 의 16×16 휘도 지문 규칙을 그대로 쓴다. `wie_validate` 가 같은 규칙(`novel`)으로 정체를 잰다. 세이브 칸은 `resume`(다시 켠 뒤 읽음) · `saved`(쓰기만) · `none`(쓰기 0) 셋이다.
**한계**(코드 `ponytail:`): 플레이 중 쓴 최고 점수·설정도 `resume` 으로 센다. 파일 시스템에 저장하는 제목은 `none` 으로 읽힌다. DB 경로(RMS · KTF `DataBase` · WIPI-C DB)만 센다.

### 2. 결과 — 55종 중 47종(남은 8종은 회차 시간으로 끊었다)

호스트: host-load-guard rc=0 을 기다려 13:2x 에 시작했다. `--jobs 3`, census 잠금을 지켰다. 실행 중 load1 은 9~218 이었다(다른 레인). `stuck` 은 **짝 재측(P2)을 못 돌렸다** — 그래서 v2 `stuck` 은 확정이 아니고 compat 에 새로 싣지 않았다. `ok` 는 짝이 필요 없다(0393 규칙).

| 무리 | 잰 수 | v1(1차) | v2 |
|---|---|---|---|
| ⒜ 정책 35 — 정책만 | 28 | stuck 28(확정) | **ok 15** · stuck 13(미확정) |
| ⒜ — 레시피 + 정책 60분(`a30bbe008b5e` `739c7657c1f2`) | 2 | 30분 F «ok»(거짓 · §3) | stuck 2(레시피 뒤 정책이 상점·능력치 메뉴를 맴돈다) |
| ⒝?·미조사 16 | 13 | stuck 13 | ok 4 · stuck 9 → 분류 §5 |
| 대표 8(60분) | 8 | ok 3 · stuck 5(30분) | ok 3 · stuck 2 · error 1 · 레시피 stuck 2(위 행과 겹침) |
| 못 잰 것 | 8 | — | `d607a2622126` `d647131cdc8d` `f5bd7a91a107` `fb7a86ce425b` `fba094100d13`(⒜) · `d9afc4db742c` `e68b1c8aef85` `ea35907b22a4`(미조사) |

v2 `ok` 가운데 프레임을 직접 본 것은 다섯이다. `4df05a4dc452` 는 도움말에서 CLR 로 빠져 본편에 들어갔다. `41466fc7f709` 는 빈 불러오기에서 새로하기로 갔다. `08aa799c11b0` 은 통신 메뉴를 지나 본편 화면에 닿았다. `2fc792485d91` 은 지도 선택 화면이다. `13d7e3c21856`(라이브 LGT)은 쿠폰 메뉴 뒤 3430초에 필드에 들어갔다. **곡선이 `ok` 인데 실제로는 통신 오류 화면을 왕복한 1종(`77c2d0bcd435`)은 compat 에 싣지 않았다**(⒞).

대표 60분 도달 지점(정책만, 레시피 표시 R):

| sha12 | v2 | 도달 지점 | 세이브 |
|---|---|---|---|
| `49ade89578c5`(가드) | ok · 새 화면 59 | 계속 새 장면 | resume(쓰기 42 · 이어 읽기 5) |
| `ddd885583b15`(가드) | ok · 51 | 계속 새 장면 | resume |
| `13d7e3c21856`(라이브 LGT) | ok · 11 | 쿠폰 메뉴 ~57분 → 필드 진입 | none |
| `1b107b96bf4e`(라이브 LGT) | **error 2564초** | 호스트 panic(§4) | resume(21 · 3) |
| `b475b6399684`(라이브 LGT) | stuck · 2 | «게임시작» 뒤 설명서 마지막 쪽(§5) | none |
| `4ece6eeeaa04`(라이브 LGT) | stuck · 7 | 체스판 — 커서만 움직인다(0393 의 알려진 한계) | saved |
| `a30bbe008b5e` R | stuck · 29 | 몬스터 능력치 배분 화면 순환 | none |
| `739c7657c1f2` R | stuck · 26 | 마을 상점·퀘스트 메뉴(«잔액 부족») 순환 | resume |

**스테이지 2 도달은 판정하지 못했다.** 지문은 스테이지를 모른다. 위 표의 «도달 지점»은 프레임을 직접 보고 적었다.

### 3. 정렬 결함 — 1000초 넘는 실행은 뒤섞여 판정됐다

`--shot-every` 프레임 이름은 `t990.0` · `t1000.0` 이다. census 는 이것을 **문자열로 정렬**했다. 그래서 `t1000.0` 이 `t110.0` 보다 앞에 왔다. 곡선은 «i 번째 = (i+1)×10초»로 읽으므로 1000초가 넘는 실행은 순서가 뒤섞였다. 1차 30분 실행(0393 대표 8종 + 레시피 2종)을 바른 순서로 다시 판정한 결과:

| sha12 | 1차 판정 | 바른 순서 |
|---|---|---|
| `49ade89578c5` P | ok stall 500 | stuck stall 710 |
| `4ece6eeeaa04` P | ok 470 | stuck 1270 |
| `ddd885583b15` P | ok 320 | stuck 1110 |
| `a30bbe008b5e` F(레시피) | ok 290 | stuck 980 |
| `739c7657c1f2` F(레시피) | ok 560 | stuck 1280 |

⇒ 1차가 «30분 ok» 라고 한 다섯이 모두 거짓이었다. 이번 회차 60분 v2 로 다시 쟀다. `49ade`·`ddd8` 은 `ok`, `4ece` 는 `stuck`(미확정)이다. 그래서 `4ece` 의 compat `progress` 는 내렸다(키 없음). 600초 실행은 이름이 모두 세 자리라 영향이 없었다.
수정: 시각을 이름에서 읽어 정렬한다(`shotTime`). selftest «frames are ordered by their time, not their name» 를 넣었다. 이 수정을 되돌리면 그 항목이 FAIL 이다(실측 40/41).

### 4. ⒝ — 라이브 LGT `1b107b96bf4e` 40분 뒤 호스트 panic

2564초에 `LGT host error unbuildable … (guest heap exhausted)` 가 3번 나왔다. 그 뒤 `class_instance.rs:109`(`class_definition` 의 `self.class().unwrap()`)이 `InvalidMemoryAccess(0)` 로 panic 했다. `host_error` 는 힙이 바닥나 예외를 못 만들면 **주소 0 인스턴스**를 대신 던진다(`jvm_support.rs` 의 `from_raw(0, …)`). 누군가 그 클래스를 읽는 경로가 남아 있다. 0403 이 막은 힙 고갈 panic 의 다른 출구다. 재현에 40분이 들어 이 회차에서는 고치지 않았다 → 후속 표.

### 5. ⒝?·미조사 16 분류(13 측정)

| sha12 | 1차 | 지금 | 근거 |
|---|---|---|---|
| `1cf2e6076079` | ⒝? Data 인스톨 실패 | **ok** | 0401(KTF DB 목록)이 고쳤다. 인스톨 통과 → 본편 |
| `5028b8a5d19f` | ⒝? 같은 경로 | ⒜ | 인스톨 통과 → 팀 강화 «포인트 부족» 대화상자 순환 |
| `01e2715ba07a` | ⒝? AIOOBE | ⒜ | 필드·능력치 화면까지 간다 — 능력치 표 순환 |
| `0865be217bde` | ⒝? 가게 이름 입력 | **ok** | 입력기 수정 뒤 진행(이어하기 resume) |
| `77c2d0bcd435` | ⒝? 메모리 접근 | ⒞ | «네트워크 문제로 서버에 접속하지 못하였습니다» 화면 왕복 |
| `9789fec50f39` | ⒝? NPE | ⒝ | 게임 스레드가 `NullPointerException: image is null` 로 죽는다 → 메뉴 고정 |
| `65bace1623a7` | ? 거의 빈 화면 | **ok** | 진행 |
| `38277d63b0ba` | ? 흰 화면 | ⒝ | `img is null` NPE · 흰 화면 |
| `bf54c05e58a9` | ? 검은 화면 | ⒝(화면 크기) | 패키지에 `title/main_logo_120/176.png` 만 있다. 게임은 240폭이라 `_240` 을 찾고 실패한다 — 다른 화면 크기 단말용 패키지 |
| `a540945188ca` | ? 갈색 칸 | ⒝? | 폼 위젯 칸만 그려지고 글자가 없다 |
| `d1e0badfce82` | ? 그라데이션 | ⒝? | 메뉴 뒤 배경만 그린다 |
| `44b6356d13f8` | ⒞? 시작 직후 종료 | ⒝? | `FileNotFoundException: File not found` 뒤 정상 종료 ×8 |
| `1793f87924d4` | ⒞? 다운로드 1/11 | ⒞ | «데이터파일을 다운받습니다 … 접속하시겠습니까?» |

`b475b6399684`(라이브 LGT, 1차 ⒜ «CLR 필요»)는 **1차 분류가 틀렸다.** 손 키 시험 결과, 설명서 앞 두 쪽은 확인으로 넘어간다. 마지막 쪽은 OK·CLR·소프트키·숫자·방향 모두 무반응이다. 60분 v2 의 탈출 58회도 이 쪽을 못 넘겼다 → ⒝?

### 6. 키 레시피 — 한 형식·한 위치

wave4(`wie-census-wave4-remaining-walls-and-locked-titles`, 먼저 착지)의 형식을 따른다. `--titles` 한 줄은 `<sha12> [secs] [recipe keys file]` 이다. 레시피는 `wie_validate --keys` 문법이다. 위치는 `game_lab/recipes-sound/<sha12>.keys`(비커밋)다. 이 회차의 레시피 2개도 그 디렉터리로 옮겼다. 도구가 레시피를 읽는 경로는 원래 커밋돼 있으므로 새로 추가한 것은 없다.

### 7. compat.json `axes.progress` 갱신 방법

`docs/player-data/compat.json` 에서 **잰 행만** 손으로 바꿨다(census `report` 는 전 행의 6축을 다시 쓰므로 쓰지 않았다). 규칙: v2 정책(레시피 아님) `ok` → `ok` · `error` → `stuck`. 거짓 `ok` 였던 `4ece6eeeaa04` 는 키를 지웠다. v2 `stuck`(미확정)은 기존 값을 둔다. `77c2d0bcd435` 는 제외했다(§2). 결과: `progress` ok 20 → 37 · stuck 82 → 64 · 키 없음 327 → 328. `node scripts/player-data.mjs` OK.

### 8. 진척 — 전수 확대

남은 playable(compat 에 `progress` 키 없는 것)은 이 회차 뒤 약 **270종**이다. 이번에 잰 것은 1차 stuck 재측뿐이고 새 제목은 0이다. 10분 v2(+재시작 2분) × 270 ÷ 3 jobs ≈ **18시간**이다. 다음 회차에 이어서 한다. 남은 일은 셋이다. ⑴ v2 `stuck` 23종의 짝 재측 ⑵ 못 잰 8종 ⑶ 새 제목.

### 9. 게이트

`cargo fmt --check` · `cargo clippy --all -D warnings` · wasm32 clippy · `+beta` clippy · `RUST_MIN_STACK=4194304 cargo test --all`(651 pass · 0 fail) · `npm run build:wasm` 은 모두 rc=0 이다. census selftest 48/48. 러너 블록 draw·helloworld×2·keydraw×2(`--expect-last-frame`)·text 는 모두 PASS 다. 되돌리면 red: `stall_escape_splices_keys_and_shifts_the_rest` · `novel_is_the_census_new_screen_rule` · `db_stats_count_a_resumed_read_only_across_boots_after_play` · `restart_at_reboots_once_and_reports_the_database` 가 새 함수와 동작을 직접 잡는다. census 쪽은 «the resume tail is outside the curve» 와 «frames are ordered by their time» 이 잡는다(각각 되돌려 FAIL 실측).

유입(corpus-name-inflow): BOUNDED 333쌍 · SUFFIX-ATTACHED 15쌍 — 전부 이 회차가 고친 파일에 **이미 있던** 문자열(compat.json 의 공개 `title` 필드 · wie_validate.rs 의 기존 주석)이다. 이 회차가 더한 줄의 게임명 0(`git diff origin/main` 의 `+` 줄 대조).
