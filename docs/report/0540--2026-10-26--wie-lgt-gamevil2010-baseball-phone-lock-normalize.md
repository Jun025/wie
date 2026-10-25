## [2026-10-26] LGT `01f05f8231f4` 구매 휴대폰 잠금 해제 — 이 판 하나만, 해시 키 바이트 패치 2건 · stdlib `0x40b` = strchr (wie-lgt-gamevil2010-baseball-phone-lock-normalize)

**무엇을**: 0463 §4 가 «구매 휴대폰 잠금 · 풀지 않음»으로 둔 LGT `01f05f8231f4` 를 켰다. 바꾼 것은 세 가지다.
- 두 소유자 확인의 분기를 «일치» 쪽으로 돌리는 바이트 패치 2건을 `wie-core-arm/data/binary_patches.toml` 에 넣었다. 이 판의 `binary.mod` MD5 에만 걸린다.
- LGT 로더가 그 표의 **해시 키 항목만** 설치한다(`install_title_binary_patches`).
- 잠금 다음에 나온 벽인 LGT stdlib `0x40b` 를 strchr 로 구현했다.

`LOCK_HAND` 를 지웠다(남은 행이 없었다). `compat.json` 1행을 고치고 이용자 소식 1건을 더했다.

**왜**: 운영자 지시 2026-10-26 「(`01f05f8231f4`) 기기 잠금도 정상화」(원문은 타이틀 이름). 이 지시는 **이 sha 한 판에만** 2026-10-02 정책(인증 우회 금지)을 푼다. 다른 잠금 판(SKT `SecureUtil` · `install.dat` · KTF 슬롯 13)의 정책은 그대로다.

**사용자 영향**: 이 게임이 «지원 안 됨»에서 «플레이 가능»이 된다. 타이틀 화면, 메뉴, 팀 선택을 지나 경기(타석)까지 들어간다. 소리는 아직 나지 않는다(§5).

### 1. 두 벽과 패치 — 번호를 몰라도 되는 자리

두 곳 모두 12바이트를 `MC_knlGetSystemProperty("PHONENUMBER")` 와 비교한다. 패치는 그 비교를 받는 분기(`beq`)를 무조건 분기(`b`)로 바꾼다. 같은 오프셋이라 1바이트(`d0` → `e0`)만 다르다. 번호 값은 읽지도 쓰지도 않는다. 에뮬레이터의 `PHONENUMBER` 답(`01000000000`)도 그대로다.

| 자리 | 함수 | 원래 | 패치 뒤 |
|---|---|---|---|
| `0x69d72` | 저장 로더 `0x69ccc` — `if (소유자 확인) if (memcmp(저장+12, 번호, 12)) 오류 −1001` | 동봉 `game_*.sav` 7개가 전부 어긋난다 → «문의전화» 오류 상자 → 장면 멈춤(검은 화면) | 비교를 건너뛴다. 저장 자체의 체크섬 검사(`0x69dc2`)는 그대로 돈다 |
| `0x682cc` | 라이선스 `0x68280` — 복호화한 `P/audio.adt` +0x28 과 번호 비교. 같으면 1 | 어긋난다 → «인증에 실패하였습니다» 뒤 스스로 종료 | 같을 때의 `return 1` 로 간다. `audio.adt` 가 없을 때의 `return 0` 은 그대로다 |

- 두 패치에 `expect`(원래 바이트)를 달았다. 바이너리가 다르면 설치가 바로 실패한다.
- 범위: MD5 `8629d3e8…` 가 맞는 LGT 바이너리는 코퍼스 LGT `binary.mod` 100개 중 이 판 1개다(전수 실측).
- LGT 는 종전에 이 표를 설치하지 않았다. 표 전체(`install_binary_patches`)를 설치하면 해시 없는 일반 항목(KTF 복사 훅)도 모든 LGT 타이틀에 걸린다. 그래서 해시 키 항목만 설치하는 함수를 따로 두었다. 해시가 맞지 않으면 0건이다.

### 2. 다음 벽 — stdlib `0x40b` = strchr

- 잠금을 지나자 첫 투구에서 `Unknown lgt stdlib import: 0x40b` 로 죽었다(키 16 · `LSOFT`). 스크래치 빌드로 호출 인자를 읽었다. 호출 자리(`0x9cf22`)는 `f("ui/slt_pitch.raw", '.')` 이고, 돌려받은 포인터에서 `.raw` 를 읽는다.
- 자리도 맞는다. `0x40b..0x410` 은 C `<string.h>` 순서 strchr · strcspn · strpbrk · strrchr · strspn · strstr 과 정확히 6칸이다. `0x410` 은 이미 strstr 이다.
- 시험: `stdlib_import_0x40b_is_strchr_through_the_svc_table` 이 SVC 표를 거쳐 찾음 · 못 찾음 · 종단 문자를 확인한다.

### 3. 측정

빌드는 release `wie_validate` 다. base = `origin/main` `3f4c8f7ad`, head = 이 브랜치다. load1 은 측정 내내 69~240 이었다.

| 실행 | 결과 |
|---|---|
| head · census 형식 A(30초) | PASS · 키 27/27 · 그림 517 · 경기 화면 도달 |
| head · census `run`(A·B·L 600초·S) → `report` | **playable** · boot/render/input/longplay/speed ok · sound silent · L 은 키 854/900 · deadline |
| base · 같은 A | FAIL · 그림 0(0463 그대로) |
| 개악 ⑴ 해제 항목 전체 제거 | FAIL · 그림 0 · deadline — 검은 화면으로 돌아간다 |
| 개악 ⑵ 라이선스 행만 제거 | 그림 334 뒤 «인증에 실패하였습니다» 상자 · 키 20/27 에서 clean exit |
| 범위 대조 · SKT `SecureUtil` `c33090c12755` | 전·후 모두 그림 0 · clean exit · `report` 판정 not-yet · 잠금 안내 그대로 |

LGT clet 표본 10종(`1cd151222bde` `236c7da689f6` `2a8a3dcd07eb` `2dbde9acca99` `4fdbd64c9fbd` `8f7758fa43b6` `acc4215b7ec0` `b7699c10dfd1` `c9b287e3edcf` `ddd885583b15`)의 census probe A·B 전·후:
- 판정 20쌍이 전부 같다(PASS · deadline · 키 27/27).
- `report` 축에서 다른 것은 `b7699c10dfd1` 소리 하나다(silent → ok). A 의 재생 0 → 1회 차이라 흔들림으로 읽는다.
- 그림 수 차는 load1 190~240 구간의 차다.

### 4. 번호 유출 0

- 번호 값은 어느 단계에서도 출력하지 않았다. 패치는 비교 결과만 바꾸므로 값을 알 필요가 없었다.
- 대조 방법: 복호화한 `audio.adt` +0x28 의 비교 대상 값(전체 · 뒤 8자리 · 앞 0을 뺀 것, 3줄)을 스크래치 패턴 파일(600 권한)에 바로 썼다. 그 파일로 변경 diff, repo 의 추적·신규 파일 전부, 회신, 측정 로그, 캡처를 `grep -c` 했다. 결과는 전부 0건이다. 양성 대조(복호화한 `audio.adt`)는 1건이다. 그 뒤 디렉터리째 지웠다. 수치는 회신 파일에 있다.
- 캡처는 번호를 그리지 않는다. 오류 상자에 나오는 것은 고객센터 대표 번호다.

### 5. 남은 것 — 소리

- 동봉 저장에 원 주인의 «소리 끔» 설정이 들어 있다. 스크래치 사본에서 `game_*.sav` 를 빼고 돌리면 재생 2 · MIDI 이벤트 1,496 이 나온다(커밋 0 · 사본은 `/tmp` 에서 지웠다).
- 처분은 0537 의 동봉 소리 끔 장치(`wie-backend::owner_files`)로 한다. 다만 LGT 마운트 경로에는 그 장치가 없고, 이 저장에는 진도가 함께 들어 있다(바이트 방식 필요). 그래서 제안 1건으로 남겼다(worklog).

### 6. 겹침

- 같은 레인 앞 티켓 `wie-lgt-server-wall-six-titles-local-net-emulation` 은 이 회차 시작 시점에 PR 이 없었다(작업트리에서 진행 중). `compat.json` 은 이 판 1행만 바꿨다. `playability-census.mjs` 는 `LOCK_HAND` 블록만 지웠다. 그 PR 이 열리면 그 두 자리가 겹칠 수 있다.

### 7. 게임 파일명 유입

- 도구 표기 BOUNDED 338쌍 · SUFFIX-ATTACHED 17쌍이다. 전부 계약상 제목 목록인 `compat.json` 과, 이 회차가 고친 파일(`init.rs` · `stdlib.rs` · `svc_ids.rs`)에 **이미 있던** 주석이다.
- 이 회차가 더한 줄의 게임 이름은 0이다(타이틀은 sha12 로만 적었다).
