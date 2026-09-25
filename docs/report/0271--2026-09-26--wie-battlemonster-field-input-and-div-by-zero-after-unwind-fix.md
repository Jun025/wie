## [2026-09-26] 배틀몬스터 필드 — 입력은 먹는다 · 멈춘 것은 장면 스레드였다(#292 가 푼다) · `/ by zero` 재현 0 (wie-battlemonster-field-input-and-div-by-zero-after-unwind-fix)

**무엇을**: 코드 변경 없음. #292(`4646c3d8` · unwind + StringBuffer 10/13/22 행) 위에서 티켓의 두 벽을 재현·판정했다.
증적(게임 바이트 0): `~/orchestrator/reports/evidence/wie-battlemonster-field-input-and-div-by-zero-after-unwind-fix/`.

**왜**: 셸 실플레이(엔진 `852143b7` = #292 이전)가 «필드에서 방향키 무반응»과, #292 중간 회차의 `ArithmeticException: / by zero` 를
다음 벽으로 남겼다.

**사용자 영향**: #292 가 착지하면 배틀몬스터를 새로하기 → 스토리 → 숲 → 포획 튜토리얼 전투 → 마을까지 걸어 다닐 수 있다
(headless 로 확인 · 셸 재확인은 착지 뒤). 단 아래 «놈3» 절의 회귀가 #292 에 있다.

### 벽 ⑵ 필드 입력 — 원인 계급 = ABI 행(`StringBuffer` vtable 22)

- 키는 필드에서도 게임에 닿는다: `net.wie.CardCanvas::keyPressed` → 유일한 카드의 `keyNotify` 경로가 메뉴와 **같다**(로그로 확인).
- 셸이 본 «무반응»은 키 경로가 아니라 **장면 스크립트 스레드 `a.run()` 의 사망**이다. 새로하기 경로(같은 키 스크립트)를 교대 2짝:

  | 바이너리 | 미처리 예외 | 숲 장면 |
  |---|---|---|
  | main `1ad81359` | **2/2** `Unimplemented: java/lang/StringBuffer vtable index 22`(PC `0x7100026a`) | 주인공 혼자 · 대화창 없음 · 정지 = 셸 스크린샷과 같다 |
  | #292 `4646c3d8` | **0/2** | 대화창 → 에이미 등장 → 대사 진행 |

  ⇒ 셸의 숲 장면은 스크립트가 죽어서 대사가 안 뜬 상태였고, 방향키가 안 먹은 것은 그 결과다. 수리 = #292 의 StringBuffer 22 행.
- #292 위 끝까지(키 스크립트 약 100단계 · 커밋 안 함): 숲 대화 → 「전투의 포획연습」 → 전투 튜토리얼(`*` 정보 · 취소 · `#` 포획 ·
  EZ 공격 → 포획 성공) → 파티 메뉴 튜토리얼 → 마을. 각 안내문이 요구한 키에만 반응했다.
- **입력 판정(대조군)** — 마을 자유 이동, 같은 실행의 4초 구간 비교(`field_pixel_diffs.txt`):

  | 구간 | 입력 | 변한 화소 |
  |---|---|---|
  | 91→92 | 없음(대조) | 891 (NPC 대기 애니메이션) |
  | 92→93 | UP 1초 | **49,967** (시점 이동 · 주인공 이동) |
  | 93→94 · 94→95 | 없음 | 709 · 1,015 |
  | 95→96 | UP 2초 | **76,800** (주인공이 집 안으로 들어감) |

  별도 실행에서 마을 출구로 DOWN → 「지금 밖으로 나가면 대회에 출전할 수 없어」(출구 차단 이벤트)도 떴다.

### 벽 ⑴ `/ by zero` — 재현 0 · 원인 미판정

- 티켓 명령(`--inject --boot-secs 6 --action-secs 2 --max-ticks 2000000000 --timeout 300`) 교대 4짝(load 89~124):
  main 0/4 · #292 0/4 미처리 예외. 둘 다 `FileNotFoundException` 1건만(아래).
- 산술 예외 SVC(`RaiseArithmeticException`)에 레지스터 덤프 훅을 건 #292 바이너리로 **27회**(티켓 명령 7 · 새로하기/이어하기/전투
  긴 경로 20) — 훅 호출 0 · `ArithmeticException` 0.
- #292 회차의 a1 은 «수정본(중간)» 바이너리였고 그 트리는 남아 있지 않다. 그래서 원인을 판정하지 않는다(지어내지 않는다).
- 관련 사실 하나: a1 이 멈춘 화면은 **「이어하기」 경로의 마을**이다. 게임 아카이브에는 10바이트 `mastercom.sav` 만 들어 있고
  슬롯 데이터는 없다 ⇒ 「저장된 데이터가 있습니다」는 맞는 동작이고, 이어하기는 슬롯 파일에서 `FileNotFoundException` 을 받은 뒤
  0 으로 채운 상태로 들어간다(주인공이 지도 원점 (0,0) 에 그려진다). 엔진 스텁 문제가 아니라 코퍼스 사본의 한계다.

### ★놈3 — #292 에 회귀가 있다(이 회차 소관 아님 · -fix 로 넘김)

`--inject --boot-secs 15 --action-secs 1.2 --max-ticks 100000000000`(otterpebble 등재 회차의 4종 명령)에서 놈3 이
**main 스레드 스택 넘침**(rc 134)으로 죽는다:

| 빌드 | 결과 |
|---|---|
| 깨끗한 `4646c3d8`(LTO) | overflow 2/2 |
| main `ed36be98` + `4646c3d8` 병합 | overflow 2/2 |
| main `1ad81359` | PASS 2/2 |

기본 일정(`--inject --expect-last-frame --timeout 300`)에서는 둘 다 PASS 2/2 — -fix 회차가 잰 것은 이 일정이다.
계측(-fix 회차의 trace 패치): 연쇄 unwind 뒤 프레임 `0x4a85e500`(sp `0x400fff44`)이 lr `0x1c7d9`(Thumb)로 다시 push 되고 곧바로
같은 프레임으로 unwind(resume `0x1c7d9`) — 이 짝이 40,078회 반복된 뒤 스택이 넘친다. 발췌 = `nom3_292_overflow_trace_excerpt.txt`.
나머지 3종(메이플스토리2007 · 현영맞고2006 · 체스마스터)은 같은 명령으로 main · #292 모두 PASS 2/2.

### 어디까지 갔나 (#292 기준 · headless)

| 단계 | main | #292 |
|---|---|---|
| 새로하기 → 스토리 → 숲 | ✓ (숲에서 장면 스레드 사망) | ✓ |
| 숲 대화 · 포획 튜토리얼 전투 | ✗ | ✓ |
| 마을 자유 이동(방향키) | ✗ | ✓ |
| 전투 진입(튜토리얼) | ✗ | ✓ |

**게이트**(이 트리 · 코드는 #292 head 그대로): fmt · clippy `--all -D warnings` · wasm clippy · `+beta` clippy rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **447 passed / 0 failed**.
