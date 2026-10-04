## [2026-10-04] census longplay — 게스트 Java 스레드 uncaught 사망을 «생존»으로 읽지 않는다 (wie-census-longplay-guest-thread-death-reads-as-ok)

### 무엇을
- `scripts/playability-census.mjs` `validate()` 가 stderr **전체**에서 rustjava 의 `Uncaught exception in thread N:`(및 `failed to format uncaught exception in thread`) 줄을 세어 결과 JSON 에 `uncaught_threads` 로 남긴다(꼬리 200줄과 무관 — `net_connects` 와 같은 방식).
- longplay 판정을 `longplayVerdict()` 로 뺐다: `FAIL` **또는** 게스트 스레드 사망이면 `error`. 그 필드가 없는 옛 기록은 `L.stderr` 꼬리에서 같은 줄을 찾는다(하한).
- clusters 의 벽 문구: 사망 건은 `guest thread died: <예외 블록>`(앱 클래스 이름은 기존 `wallOf` 가 지운다).

### 왜 — 신호 선택(실측 · wave6 전수 `~/scratch/w6census/out` · 재측 0)
같은 힙 고갈이 메인 tick 에 떨어지면 `FAIL`, 게임 스레드에 떨어지면 실행만 마감까지 헛돌아 `UNMEASURED` 로 끝나 «생존»이 됐다(`docs/report` 밖 근거: 회신 `wie-longplay-regression-8d8c24b7c198-bisect` — 6차 «ok → error» 퇴행 오탐).

| 후보 신호 | longplay 비-FAIL 392건 중 걸리는 수 | 판정 |
|---|---|---|
| `Uncaught exception in thread` | **5** — 4건 꼬리 동결(타이머 샷 29장 중 21~29 동일 · `frozen_tail_steps` 603~850) · 1건 사망 직후 clean exit(49/900 스텝) | **채택** |
| 꼬리 동결만(동일 샷 ≥ 21/29) | 58(그중 사망줄 없는 것 54) | 기각 — 대부분 키 루프가 못 빠져나오는 메뉴(기존 `stall` 기각 근거와 같다) |
| 꼬리 동결만(≥ 7/29) | 69(사망줄 없는 것 65) | 기각 |

- 사망줄이 있는데 화면이 계속 움직인 실행은 디스크 전수(wave6 + 짝 `pair/` 4회)에서 **0건** ⇒ 임계 없이 «1회 이상 = 생존 아님».
- 6차 짝 `pair/p_bd2337ff`·`q_bd2337ff` 의 `8d8c24b7c198`(전 빌드 «생존») 도 사망줄 보유 ⇒ 새 판정으로는 전/후 모두 `error` — 퇴행 오탐이 사라진다.

### 전/후 — wave6 기록 그대로 `report` 재실행(compat 은 스크래치에만 · repo compat 재생성 0)

| | longplay ok | longplay error | playable | limited |
|---|---|---|---|---|
| 전 (`origin/main` 판정기) | 388 | 9 | 387 | 25 |
| 후 | **383** | **14** | **382** | **30** |

빠진 5건(sha 앞 12): `6c9f969f089f`(ktf · Invalid memory access) · `6eb93824daf8`(ktf · 같은 벽) · `73f3a21e981c`(lgt · GregorianCalendar vtable 28 미구현) · `9789fec50f39`(ktf · NPE) · `d3e3b16cefd0`(skt · NPE 후 clean exit). 반대 방향 변화 0.

### 한계
- 어느 스레드든 센다 — 보조 스레드만 죽고 게임은 계속 도는 타이틀이 있으면 `error` 로 읽힌다(디스크 0건 · 코드에 `ponytail:` 표기).
- 옛 기록의 꼬리 판독은 하한이다(사망이 꼬리 200줄 밖이면 놓친다). 다음 전수부터 `uncaught_threads` 가 전체 수를 남긴다.

### 사용자 영향
- 다음 전수의 compat 에서 위 5종이 playable → limited 로 내려간다(이번 PR 은 compat 을 바꾸지 않는다).

유입 0건(BOUNDED) · 판단 필요 0건(SUFFIX-ATTACHED).
