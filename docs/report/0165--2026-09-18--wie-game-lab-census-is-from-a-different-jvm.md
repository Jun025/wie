## [2026-09-18] `game_lab/broken` census 를 **오늘 트리로 다시 떴다** — 서명 분포가 갈렸고, 최대 군집이 **바뀌었다** (wie-game-lab-census-is-from-a-different-jvm)

**무엇을**: 채택 제안 `2026-09-17-game-lab-broken-187-signature-triage#p0`. `broken/**/*.zip` **187건 전수**를 오늘 트리로
재실행하고 **7월 표와 나란히** 실었다. ★**분류 규칙은 바꾸지 않았다 — 복원해서 «증명»한 뒤 그대로 썼다**(아래 ⓓ).
★수리 **0건**(별 축) · `classify.sh` **무접촉** · 코퍼스 바이트·경로 **repo 유입 0**.

## ⓒ 모집단 — 「1,312」은 **오늘 트리에서 재현되지 않는다**
| 센 것 | 값 |
|---|---|
| `broken/**/*.zip` | **187**(★distinct stem **184** — 중복 stem 3) |
| `working/**/*.zip` | 294 |
| `reports/` | `.json` **452** · `.log` 470 · `.md` 66 · `.tsv` 3 · `.png` 6,389 |
| `reports/` mtime | **2026-06 181 · 2026-07 271** — ★**2026-08 이후 0** |

⇒ ★**`1,312` 은 어떤 조합으로도 나오지 않는다**(json+log **922** · json+log+md+tsv **991** · zip 전체 **483**).
★**이 티켓의 모집단은 `broken` 187** 이고, 그중 **186** 이 7월 리포트를 갖는다(**1건 결손**) — 공표표의 `합 186` 이 바로 이 수다.
★「낡았다」는 **참이다**: 리포트 mtime 이 **전건 6~7월**이다.

## ⓓ ★분류 규칙을 «복원»했다 — 인용하지 않고 재현으로 증명
★**규칙이 어디에도 기록돼 있지 않았다**(선행 리포트·worklog·`classify.sh` 전부에 없음). 그래서 **7월 공표표를 재현할 때까지 역산**했다:

| 서명 | 복원한 술어 |
|---|---|
| `NoSuchMethod` | `NoSuchMethodError` **또는** `Method .*? not found` |
| `panic-unwrap` | `unwrap\(\)` |
| `unimpl-stub` | `WieError: Unimplemented` |
| `unknown-stdlib-import` | `Unknown lgt stdlib import` |
| `panic-unreachable` | `unreachable code` |
| `UNCLASSIFIED` | 그 밖 |

★**검산 — 7월 데이터에 적용하니 공표표와 «전건 일치»**: `37 · 37 · 15 · 6 · 4 · 87 = 186` **6/6 ✓**.
★**이것이 「같은 규칙, 다른 입력」의 증거다** — 추측이 아니다.
※첫 시도(`not implemented`·`NoClassDefFoundError` 같은 상식적 술어)는 **21/37/7/3/4/114** 로 어긋났다.
  급소는 **`Method …@N not found from …`(16건)** 와 **`Unknown lgt stdlib import`(6건)** 가 `WieError: Fatal error` 안에 숨어 있던 것이다.

## ★★두 표 — 같은 규칙 · 다른 입력(FAIL 만: census 의 목적이 «실패 서명»이다)
| 서명 | 7월 | **오늘** | 차 |
|---|---:|---:|---:|
| `NoSuchMethod` | 37 | **8** | **−29** |
| `panic-unwrap` | 37 | **15** | **−22** |
| `unimpl-stub` | 15 | **6** | **−9** |
| `unknown-stdlib-import` | 6 | **2** | **−4** |
| `panic-unreachable` | 4 | **0** | **−4** |
| `UNCLASSIFIED` | 80 | **99** | +19 |
| **FAIL 합** | **179** | **130** | −49 |
| **PASS** | **7** | ★**57** | **+50** |

★**짝지은 186건 중 75건(40%)이 서명 버킷을 바꿨다** — `NoSuchMethod→UNCLASSIFIED` 29 · `panic-unwrap→` 27 ·
`unimpl-stub→` 9 · `unknown-stdlib→` 4 · `panic-unreachable→panic-unwrap` 3 · `UNCLASSIFIED→panic-unwrap` 2 · 불변 111.
⇒ ★**제안의 전제(「낡았다」)는 참이고, 표를 다시 뜨는 것이 값을 했다.**

## ★★오늘의 최대 군집은 **새것**이다 — `no frame rendered` **32**(7월 **1**)
오늘 FAIL 130 의 `UNCLASSIFIED` 99 내부:
```
32  no frame rendered (hang/black screen)      ← ★7월에는 1건
12  WieError: Invalid memory access
 9  Fatal error: U…            8  WieError: Fatal error
 7  java.lang.NullPointerException              6  java.lang.NumberFormatException
 6  java.lang.NoClassDefFoundError              5  only blank/uniform
 3  java.lang.Error   3  load error   2  hang: SIGKILL   1  attempt to add with overflow
```
★★**부하 인공물이 아니다 — 갈랐다**(이 맥은 측정 중 **CPU idle 0.0%** 였다):
그 32건은 **20초 예산을 전부 태우고**(`ms` 중앙 **21,232**) `paints` **최대 0** 이며,
3건 재실행에서 틱이 **48,751,357 / 29,017,341** 이었다 — ★**수천만 틱을 돌리고도 한 프레임도 안 그린다.**
기아(starvation)면 `ms`·`ticks` 가 **작아야** 한다. ⇒ ★**진짜 「돌지만 안 그린다」 군집**이고 **다음 수리 캠페인의 1순위**다.
※동질적이지는 않다 — 같은 3건 중 하나는 `ticks=20` 이라 **단일 틱 행**에 가깝다(그 안에서 다시 갈라야 한다).

## ★혼입 변인을 숨기지 않는다 — 「다른 JVM」만이 원인이 아니다
`PASS 7 → 57` 의 상당 부분은 ★**같은 날 착지한 폰트 수정**(`wie-game-lab-broken-187-failure-signature-triage` · `HeadlessPlatform::font()`)이다.
그 회차가 **A/B 로 `panic … not implemented` 8/9 → 0/9** 를 이미 쟀다. ⇒ ★**오늘 표의 «7월 대비 차»는 «JVM base swap + 폰트 수정»의 합**이고,
★**둘을 가르지 않았다**(가르려면 두 트리에서 각각 전수를 떠야 한다 = 비용 2배). **제안의 문면(「다른 JVM 의 기록」)은 절반만 맞다.**

## ⓑ 비용 — 실측(ⓑ 가 요구한 칸)
- 표본 8건 직렬 **111초** ⇒ 187건 추정 **43분** · **실제 전수 45분**(04:55 → 05:39 · `nice -n 15`)
- ★**이 맥은 self-hosted CI 러너를 겸한다** — 측정 중 load **98~135**, ★**CPU idle 0.0%**, `vm_pressure` 1(normal).
  ★**그 45분 동안 열린 PR 5건이 그 러너를 기다렸다** — 이것이 이 회차가 치른 값이다.
- ★**병렬화하지 않았다**: 이 저장소가 이미 기록한 「부하가 verdict 를 뒤집는다」(`AGENTS.md` §Definition of Done) 때문이다.
  ⇒ 대신 **`nice -n 15`** 로 양보했고, 판정 축을 **PASS/FAIL 이 아니라 «서명»**으로 유지했다.

## ⓐ 왜 전수를 돌렸나 — 표본이 «갈린다»고 답했다
시드 고정 표본 8건(그중 7건 비교 가능): ★**4건이 갈렸다**. ★그리고 **부하 flake 가 아니다** — 그 4건을 **3런씩** 재실행해 **12/12 PASS 안정**이었고,
그중 둘은 7월에 **`NoSuchMethod` 서명**이었다(서명이 사라지는 것은 부하로 설명되지 않는다).

## ★7월 표를 인용한 문서 — **고치지 않고 «기준일»을 명시**한다(판정)
전수 조회: 7월 수치를 인용하는 파일은 ★**`STATE.md` 하나**다(`docs/report/0151`·worklog 는 그 표를 **수로 싣지 않는다**).
⇒ ★**`docs/report/0151` 을 고치지 않는다** — 그것은 **날짜 박힌 회차 기록**이고, 원장 기록을 사후에 고쳐 쓰는 것은 이 저장소가 금지하는 형태다.
대신 **`STATE.md` 의 그 항목에 「7월 기준」과 이 회차 포인터를 붙였다**(항목 «안» 편집 — 새 삽입점 **0**).

## 대가 — 「잃는 것이 없다」가 아니다
⑴**러너 45분**(위) ⑵★**두 변인이 섞였다**(JVM + 폰트) — 가르려면 비용 2배 ⑶★**이 표도 «오늘» 에 묶인다**:
`reports/` 가 6~7월에 멈춰 두 달 만에 낡았듯, **base 가 다시 움직이면 이 표도 낡는다**
⇒ ★그래서 **복원한 규칙을 이 문서에 박아** 다음 census 가 **재역산 없이** 같은 축으로 뜰 수 있게 했다.
⑷`STATE.md` 항목 1건 추가 ⇒ 착지 시 `behind ≥ 1` 열린 PR 을 무효화한다(소관 = `wie-remove-state-md-completed-insertion-point`).

## 범위
187건 중 **수리 0** · `classify.sh` **0줄** · 분류 규칙 **무접촉**(복원만) · 새 검사기·새 잡 **0** · 제품 코드 **0줄** ·
★코퍼스는 **읽기 전용**으로만 썼다(다른 레인이 그 체크아웃에서 작업 중 — 쓰기 **0**) · ★**게임 바이트·파일명 repo 유입 0**(집계만).
