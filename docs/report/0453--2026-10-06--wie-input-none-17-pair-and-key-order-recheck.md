## [2026-10-06] input none 17종 재측 — 9종은 30초 측정의 틱 상한 탓 · 8종은 키 이전 벽 (wie-input-none-17-pair-and-key-order-recheck)

**무엇을**: `scripts/playability-census.mjs` 의 30초 측정(A·B)과 speed 측정에도 `--max-ticks 100000000000` 를 붙였다(long·progress 는 이미 그랬다 — 네 곳이 상수 `NO_TICK_CAP` 하나를 쓴다). `compat.json` 9행: input no → ok · longplay → ok · limited → **playable**.

**왜**: 0438 §6 의 input none 17종(잠금 1 제외)을 짝 2회 × 키 순서 4종으로 다시 쟀다(제안 `2026-10-05-wave7-engine-walls#p0`). 17종 중 12종에서 30초 A 측정이 `wie_validate` 기본 `--max-ticks` 50M 에 닿아 **27키 중 2~17키만 보낸 채** 끝났다(`stop: max-ticks`). census 의 input 판정은 그림만 보고 `stop`·`input_steps` 를 보지 않으므로, 이것이 «키가 안 먹는다»로 기록됐다.

**사용자 영향**: 키가 안 먹는다고 안내되던 9종이 «잘 됨»으로 바뀐다(지원 현황(playable/limited/not-yet) 391/23/15 → **400/14/15**). 엔진은 바꾸지 않았다 — 게임 동작 변화는 없고 측정 정정이라 이용자 소식은 내지 않았다(0432 선례).

### 1. 측정
- 바이너리: `wie_validate` release @ `d6db09d6`(그때 origin/main). 측정 중 main 에 #483(LGT 이미지 · DB)이 들어왔다 — 아래 수는 `d6db09d6` 기준이다.
- 단계 1 — 17종 × 7회 = 119회(30초 · 동시 3 · build-slot `--long` 1 임대): B(키 0) 2회 · A 기본 순서 2회 · 역순 · 방향키 먼저 · 소프트키 먼저. load1 9~112.
- 단계 2 — 틱 상한에 닿은 12종 × {B, A} × 2회, `--max-ticks 100000000000`. load1 10~20.
- 단계 3 — 고친 census 그대로 `run`(17종 probe + longplay 600초 9종) → `report`. 05:08 종료 · load1 8~11.
- 판정 = census 규칙 그대로(A 의 그림 중 B 에 없던 것 ≥1 ⇒ input ok).

### 2. 결과 — 원인 계급
| 계급 | 수 | sha12 | 근거 |
|---|---|---|---|
| ★측정 — 틱 상한(키 2~17개만 감) | 7 | `0eb19d9bbe7a` `1cdea1985955` `3b5b98afa890` `4b44b31e8107` `89c214dbd15d` `8c71be3ad26d` `bc94ba53677b` | 단계 1 기본 순서: `stop: max-ticks` · 키 8~22 · 새 그림 0~21(흔들림). 단계 2(상한 해제): **짝 2/2 모두 27키 · 새 그림 8~27**. 단계 3 census: input ok · L 600초 생존 |
| 측정 — 흔들림(상한 무관) | 2 | `d552e095ddcf` `bf54c05e58a9` | 단계 1: 새 그림 23/23/23/23/23 · 22/22/7/22/22(5회 중 5·5). 단계 3 census: input ok · L 생존 |
| 키 이전 렌더 벽 | 3 | `7ec716a0cec9`(상태바 + 검은 화면) · `aa3fcba4598b`(흰 화면) · `5267badf20b3`(검은 화면 · 그림 2) | 27키 다 감 · 그림이 한 장(B 1~3장)에서 안 바뀐다 · 키 순서 4종 모두 0 |
| 키 이전 안내 — 저장 공간 | 2 | `63332c51d514`(LGT «데이터 저장 실패로 게임을 종료합니다») · `f770b15f8876`(KTF «2103KB 저장공간 필요 · OK: 종료») | OK 가 곧 종료 = 2키째 `clean exit`. 어느 API 의 값인지 미식별(f770 로그: `Unknown WIPICX_incMemInterface`) |
| 키 이전 안내 — 남은 메모리 | 1 | `4b8c8f5ff7d6`(KTF «메모리가 부족합니다. 팝업어플 등을 종료 후 재실행») | `MC_knlGetFreeMemory` 스텁 = 1MiB(§3) |
| 키 이전 안내 — 재실행 | 1 | `287af341dac8`(LGT «안전한 실행을 위해 완전히 종료후 다시 실행») | `--relaunch 1` 뒤에도 같은 안내 · 원인 미식별 |
| 엔진 — 없는 API | 1 | `fb80e97cbc57`(SKT · 로고에서 멈춤 · 그림 14) | `java.lang.NoSuchMethodError: com/xce/lcdui/XDisplay.drawImageEx:(…Graphics;…Image;II…Image;IIIII)V` |

포커스 계급은 0 이다 — 27키가 다 가고도 안 바뀌는 8종은 전부 «키를 받기 전» 화면에서 멈춘다.

재현(한 타이틀): `wie_validate --inject --keep-timeout --timeout 30 --shotdir <d> --relaunch 1 --pacing 8 <file>` → `stop`·`input_steps` 를 본다. `--max-ticks 100000000000` 를 붙이면 상한 계급 7종이 27키를 다 받는다. 화면·JSON: `~/scratch/w8inp/{inp,mt,census,log}/`(로컬 · 게임 바이트 포함 경로라 커밋 안 함).

### 3. `4b8c8f5ff7d6` — 0341 의 전제가 뒤집혔다(이 회차는 고치지 않았다)
0341 은 «KTF 는 1MiB 유지 — `4b8c8f5ff7d6` 이 `free − 100KiB` 로 풀을 잡고 1.2MB 이상이면 부팅 중 `Invalid memory access`»로 공유 스텁을 1MiB 에 묶었다.
오늘 main 에서 같은 타이틀(같은 sha)은 반대다(임시 빌드 · 커밋 안 함):
| 답한 값 | 30초 | 100초(OK·5·↓ 반복) |
|---|---|---|
| 1MiB(main) | «메모리 부족» 한 장 · 7/7회 | 다른 그림 **1** |
| 1,310,720 · 2MiB · 8MiB | 로고 진행(다른 그림 12 · 12 · 10) | 2MiB: 다른 그림 **44** · 이야기 화면까지 · PASS |
- KTF 전체에 닿는 값이라 0341 이 잰 KTF 호출 46종의 전/후 스윕 없이는 바꾸지 않는다 — 후속 제안 p0.

### 4. 고친 것의 근거 — 상한 해제 전/후(같은 바이너리 · 같은 인자)
§2 첫 행이 그것이다: 7종 모두 전(기본 상한) `max-ticks` · 짧은 키 / 후(해제) `deadline` · 27/27키 · 짝 2회 input ok. 남은 8종은 해제 전/후 판정이 같다(악화 0).
`node scripts/playability-census.mjs selftest` 62/62.

### 5. compat
`node scripts/check-compat-revert.mjs`: `OK — main 에서 받은 행을 받기 전 값으로 되돌린 필드 0 (fork d6db09d6 → mb 2e0ac37e) · 착지 기준 바뀐 행 9` — 이 회차가 잰 9종뿐.
바꾼 필드는 9행의 status · input · longplay · knownIssues_ko(입력·장시간 안내 2문장) 뿐이다. sound · speed · progress 는 main 값 그대로(`bf54c05e58a9` progress `stuck` 유지).
`bc94ba53677b` 의 longplay no → ok 는 L 1회(600초 생존 · 854/900키)다 — 0432 의 longplay no 는 그 측정의 오류 1회였다. 재측 1회라 흔들림일 수 있음을 적어 둔다.
`node scripts/player-data.mjs` OK(429 · playable 400 · limited 14 · not-yet 15).

### 6. 게이트
fmt · clippy `-D warnings`(stable · wasm32 · beta) rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(702 passed · 0 failed). 엔진 무변경이라 러너 블록은 돌리지 않았다.

### 7. 게임 파일명 유입
`node scripts/corpus-name-inflow.mjs`: BOUNDED 330쌍 + SUFFIX-ATTACHED 15쌍 — 전부 `compat.json` 의 기존 `title`·`fileTitle` 값이다(이 회차는 제목 문자열을 바꾸지 않았다). 그 밖 파일 0. 화면 안내 문구는 게임 안 문장이고 제목이 아니다. 타이틀은 sha12 로만 적었다.
