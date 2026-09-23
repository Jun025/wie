## [2026-09-23] 추천 후속작업 3차 정리 — adopted 3 · declined 10 · ★게임 호환 23건은 남겼다 (wie-prune-and-adopt-followup-proposals-2026-09-23)

**무엇을**: `docs/worklog/2026-09-23-followup-backlog-third-pass-adopt-three-decline-ten.json` 한 장을 새로 쓰고,
거기에 `adoptedProposals` 3 + `declinedProposals` 10 + ref별 `dispositionNotes` 를 선언했다.
★**제안 본문도, `proposals` 배열의 원소도 하나 고치지 않았다** — 기존 워크로그 197파일 diff 0.

**왜**: 2026-09-21~23 운영자 지시 「주요도가 낮은 작업들은 추천 작업 목록에서 삭제해 정리 · 배틀몬스터에 집중」의 3차 이행.
2차 정리(머지됨)가 34 → 10 으로 줄인 뒤 다시 36 으로 불었다.
★**이번엔 성격이 다르다** — 늘어난 것의 대부분이 **실제 게임 호환 작업**(배틀몬스터 리니지의 후속)이고 그것은 옳은 내용이다.
⇒ 이 회차가 닫은 것은 «이미 착지한 것» + «메타 도구» + «다른 플랫폼(KTF)»뿐이다.

**사용자 영향**: cockpit 「후속 작업 추천」 패널의 wie 열린 제안이 **36 → 23**.
배틀몬스터(LGT)를 실제로 돌게 만드는 축만 화면에 남는다. ★**제품 코드 0줄** — 에뮬레이터가 더 돌게 되지는 않는다.

### 기준선 (origin/main · tower `scanRepoSimple` + `collectProposals.settle` 재현)

| 축 | 값 |
|---|---|
| 전체 proposals | 313 |
| adopted / declined | 177 / 53 |
| injected(pending) / dismissed | 52 / 0 |
| ★**열린 제안** | ★**36** — 티켓이 인용한 36 과 정확히 일치 |

`injected.json`·`dismissed.json` 은 `~/tower/data/` 의 실파일을 읽었다(모형이 아니다).

### 집행 — 13건

**adopted 3** (★이미 **착지**했다 · 열어 두면 중복 발권된다):

| ref | 닫은 것 |
|---|---|
| `2026-09-22-lgt-vtable-index-family-runtime-thread-string#p0` | PR **#255** (`state=MERGED` 실측) |
| `2026-09-22-aot-java-address-zero-cluster#p3` | 같은 벽 ⇒ 같은 PR #255 |
| `2026-09-22-aot-java-address-zero-cluster#p2` | `wie-lgt-jvm-trait-has-no-error-channel…` (머지) |

★2차 정리의 adopted 5건은 «발권됐다»는 뜻이었고 이번 3건은 «착지했다»는 뜻이다 — 같은 열로 읽지 마라.

**declined 10** — ⒜메타 도구 4(`validate-inject-zero-step-unmeasured#p1`·`#p2` ·
`lgt-corpus-decides-graphics-wiring#p3`·`#p0`) · ⒝KTF slot 8 계열 6(`ktf-slot8-third-image-search#p0` ·
`ktf-slot8-h2-noop-arm-reach-gate#p0`·`#p1`·`#p2` · `svc-selector5-function0-is-a-named-sized-open#p0` ·
`knl-reserved-is-ktf-extension-space#p0`).

★★**⒝는 «1차 정리의 판단을 뒤집은 것»이다** — 09-21 회차가 그 6건을 «에뮬레이터 실행 능력»이라며 남겼다.
운영자의 불만은 「배틀몬스터에 몰입하지 않고 다른 일에 퍼져 있다」였고 그 시점에 실제로 돌던 것이 KTF slot 8 이었다.
★**배틀몬스터는 LGT 다** ⇒ KTF 를 «능력»으로 남긴 것은 지시보다 넓게 읽은 것이었다. 그 6건은 09-19~21 이후 아무도 손대지 않았다.

### 검증

- ★**선언 «전»** 13 ref 가 아직 열려 있는지 전건 확인 — **13/13 열려 있었다**(형제 티켓이 먼저 닫은 것 0 · 건너뛴 ref 0)
- ★세 목록이 실측 집합을 **정확히 분할**: adopted 3 ∪ declined 10 ∪ 남길 23 == 열린 36 (**True**) ·
  목록 간 겹침 0 · 목록 내 중복 0 · 실재하지 않는 ref 0 · 열려 있는데 어느 목록에도 없는 것 0
- ★집행 후 재측 — **36 → 23**. adopted 3 중 open 잔존 **0** · declined 10 중 open 잔존 **0**
- ★**남긴 23건이 그대로 열려 있는지도 셌다(23/23)** — 닫는 쪽만 세면 과잉 차단이 안 보인다
- ★형제 티켓과 **중복 선언 0**: `wie-nom3-live-title-inject-panic-and-partial-inject-pass` 가 채택하는
  `validate-inject-zero-step-unmeasured#p0` · `validate-max-ticks-backstop-inject-axis-already-closed#p0` 은
  이 파일이 **선언하지 않는다**. 그 티켓 자신이 같은 워크로그의 `#p1`·`#p2` 를 「이번 정리에서 기각 — 하지 마라」로
  넘겨 두어 두 회차의 경계가 일치한다
- ★「이미 착지했다」를 문서가 아니라 GitHub 에 물었다 — `gh pr view 255` → **MERGED**
- ★`ref` 인덱스 불변 — `proposals` 원소를 하나도 지우지 않았으므로 기존 `#pN` 이 움직이지 않는다(기존 197파일 diff 0)

### 남은 것 · 한계

- ★**`lgt-corpus-decides-graphics-wiring#p0` 은 «메타 도구»로 분류됐지만 내용은 «실측된 은폐 결함»이다** —
  `smoke_gate` 의 baseline 대조가 유니코드 정규형 불일치로 **0건을 검사하고 OK** 를 찍었고, 그 아래에
  baseline PASS **12건의 하드 실패**가 숨어 있다(원문 겹침 0 ↔ NFC 후 52/52).
  ★기각의 뜻은 «틀렸다»가 아니라 «지금 배틀몬스터보다 앞이 아니다»이고, **그 12건이 무엇인지는 이 회차가 열어 보지 않았다.**
  ⇒ 코퍼스 회귀 판정을 다시 믿어야 하는 회차가 오면 **이 한 줄을 먼저 되돌려라.**
- ★**«닫힘»은 착지해야 보인다** — tower 는 `origin/main` 아카이브 캐시를 읽는다(`cockpitd.js` 의 `worklogDirOf`).
  이 PR 이 열려 있는 동안 화면의 36 은 그대로다.
- ★**되살리는 법**: 그 워크로그의 `adoptedProposals`·`declinedProposals` 에서 해당 ref 한 줄을 빼면
  다음 `/api/proposals` 파생에서 다시 열린다(open = 전체 − adopted − declined − injected − dismissed).
- ★이 회차는 «판단을 기록»했을 뿐 13건의 내용을 재검토하지 않았다 — 선별의 책임은 총괄에 있다.
  확인한 것은 «그 ref 가 열려 있는가»와 «착지했다는 주장이 참인가»뿐이다.
- ★다른 repo 의 메타 제안(qts·sns·otterpebble·RustJava)은 **범위 밖이라 무접촉**이다 — 티켓이 「이번엔 두었다」로
  명시했고 이 회차가 그 판단을 넓히지 않았다.
