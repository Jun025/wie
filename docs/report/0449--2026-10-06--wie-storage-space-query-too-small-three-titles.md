## [2026-10-06] 저장·설치 공간 부족으로 끝나는 3종 — KTF 여유 공간 답 1 MB → 16 MB · LGT 1종은 공간 질의가 아니다 (wie-storage-space-query-too-small-three-titles)

제안 `2026-10-05-sound-silent-57#p1` 채택. 대상 `974e0df9ab1e` · `f770b15f8876`(KTF · 같은 앱 두 판 · «설치(또는 실행) 공간이 부족합니다. 2103KB의 저장공간이 필요합니다») · `63332c51d514`(LGT · «데이터 저장 실패로 게임을 종료합니다 … 공간을 확보하시거나»).

### 1. KTF 2종 — 원인 = 여유 공간 질의의 답이 작다(엔진 값)

부팅 직후 `MC_dbListDataBase()`(KTF DB 슬롯 12 = 남은 저장 공간 바이트 · `docs/report/0438` §2) 를 한 번 묻고 **1,048,576**(`KTF_DATABASE_STORAGE_LIMIT` = 1 MB) 을 받은 뒤 바로 안내를 띄운다. 그 사이 쓰기 0.

**문턱은 실측했다**(같은 빌드 · 상수만 바꿈 · `wie_validate --timeout 6`, `stream_write` 수):

| 답(바이트) | 결과 |
|---|---|
| 2,153,471 (2103 KB − 1) | 안내 · 쓰기 0 |
| 2,153,472 (2103 KB) | 안내 · 쓰기 0 |
| 2,201,600 (2150 KB) | 통과 · 쓰기 55(설치) |
| 2,252,800 · 3 MB · 16 MB | 통과 · 쓰기 55 |

1 MB 는 측정된 값이 아니었다(`3e084d7d` 가 «알려진 호출부는 0x100 · 0x1200 미만을 거절한다»를 넘기려고 고른 수). 이 값을 쓰는 호출부(슬롯 11·12)는 전부 **아래에서** 비교한다 — 저장 길이 · 0x176f · 이번 설치 문턱. ⇒ **16 MB** 로 올렸다(`wie-wipi-c/src/api/database.rs`). 사용량 차감(`usage`)은 그대로다.

### 2. 측정 — base(`9f8b0ae2`) ↔ head

조건: `wie_validate --inject --keep-timeout --timeout 60 --max-ticks 100000000000` · 두 빌드를 같은 분에 나란히 · short 풀 단발 짝(타이틀당 1분).

| sha12 | base | head |
|---|---|---|
| `974e0df9ab1e` | 안내 뒤 종료 · `clean exit` · 키 1/27 · paints 35 · 여유 1,048,576 · 쓰기 0 | 설치(쓰기 55) → 제작사 로고 · 키 16/27 · paints 160 → **다음 벽**(§4) |
| `f770b15f8876` | 같음(키 1/27 · paints 36) | 같음(키 16/27 · paints 158 · 쓰기 55) |
| `63332c51d514`(LGT · 대조) | `clean exit` 1/27 · paints 27 | 같음 — 이 수정과 무관(§3) |

부작용 표본 — 같은 슬롯을 쓰는 #471 의 2종(`3c658a46bbfb` · `4a4d2ac046f7`): base↔head PASS 27/27 · paints 1111↔1110 · 2362↔2360.

개악 red: `ktf_available_database_storage_covers_a_2103kb_install` — 1 MB 로 되돌리면 FAIL. 게이트 4종 + beta clippy rc=0. 러너 블록 draw/helloworld×2/text PASS · `keydraw_*` 는 base·head 같이 `UNMEASURED(max-ticks)`(0448 과 같은 기존 상태 · `--max-ticks` 올리면 PASS).

### 3. LGT `63332c51d514` — 공간 질의가 아니다. 남김

안내 문구는 «공간»을 말하지만 공간을 묻는 호출이 없다(`MC_knlGetFreeMemory` 는 4 MB 로 답하고 그 결과로 갈리지 않는다). 실제 경로(`binary.mod` · `.text` 0x1000):

- 저장 로더 `0xf248`: `MC_dbOpenDataBase(name, 8, 1)` → 결과가 −11/−13/−24/−3/−9/−1 이면 «저장 없음»(크기 전역 0) · 아니면 `MC_dbListRecordInfo(name, sp, 1)` 후 **반환값을 보지 않고** `[sp+8]`(항목의 크기 칸)을 크기 전역에 넣는다.
- 호출부 `0x3fd32`: 크기 0 이면 새 게임 · 아니면 `ReadRecordSingle(h, buf, 크기)` 가 크기와 다르면 **오류 깃발 = 1** → 이 안내.
- 우리: 없는 `SAV*` 를 모드 8 로 열면 핸들을 주고(모드 1 만 −12), `list_record_info` 는 −12 를 돌려주며 **아무것도 안 쓴다** ⇒ `[sp+8]` 은 스택 찌꺼기(22,067,244 · 1,216,618,080 등) ⇒ 읽기 −23 ≠ 크기 ⇒ 깃발.

실기는 «없는 DB» 를 열기 실패(위 6개 중 하나)로 알렸거나, 목록에 크기 0 항목을 썼다 — 둘 다 이 게임을 새 게임으로 보낸다. 어느 쪽인지 근거가 없고, 둘 다 다른 LGT 호출부의 계약을 바꾼다. ⇒ 고치지 않았다(제안 카드 1).

### 4. KTF 2종의 다음 벽 — `MC_knlGetAccessLevel`(커널 슬롯 13) · 남김

head 에서 로고 뒤 `Unimplemented: 13: MC_knlGetAccessLevel`. 호출부 `0x12af8c`: `if ((level & 0xbc) == 0xbc) { MC_knlGetSystemProperty("PHONENUMBER", buf, 0x14); 12자 이하 숫자면 1 }` 아니면 오류값 → 호출자(`0x176956`)는 1 이면 한 갈래, 아니면 다른 갈래(`0x1765d0`). 즉 전화번호 접근 권한 질의이고 게임은 두 답을 다 처리한다. 실기의 값은 모른다 — 이 티켓 범위 밖이라 남겼다(제안 카드 1). census 출력에서 이 슬롯에 닿은 타이틀은 지금까지 0 이었다(이 수정이 처음 열었다).

### 5. compat

수정하지 않았다. 두 KTF 행의 축은 이번 짝 측정이 아니라 census 6축으로 다시 재야 한다(`974e0df9ab1e` 는 지금 `playable` 인데 실제로는 첫 화면에서 끝났다 — census 축이 안내 화면을 «작동»으로 본 것). ⇒ `check-compat-revert` 인용 대상 없음(바뀐 행 0).
