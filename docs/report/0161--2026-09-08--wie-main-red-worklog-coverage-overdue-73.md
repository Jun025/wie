## [2026-09-08] `main` red 해소 — 재측 기록 **73** 을 `--record` 로 남겼다 (wie-main-red-worklog-coverage-overdue-73)

★**백필 문서다**(2026-09-18 · `wie-state-md-insertion-point-conflicts-every-open-pr`). 착지 커밋 `a0aae388` · PR **#139**.
★**지어낸 칸 0** — 본문은 `STATE.md §완료` 항목 **인용 그대로**다.

## 무엇을 · 왜 (STATE.md §완료 인용)
> **`main` red 해소 — 재측 기록 **73** 을 `--record` 로 남겼다** (PR **#139** 착지 ·
> `wie-main-red-worklog-coverage-overdue-73`) — ★**검사기·임계·워크플로 스텝 무접촉 · 코드 0.**
> ★**진 것은 «비율»이 아니라 «기록 약속»이다**: `10/10 = 100%` 인데 착지 **73** 이 `63+10` 을 채워 `OVERDUE`(rc=1).
> ★**손으로 붙이지 않았다** — `--record` 는 멱등이고, 손 append 는 2026-09-06 에 동일 행 **3건**을 만들었다.
> ★★**구조를 적어 둔다**: 이 red 는 «머지 직후»에만 보인다(검사기가 `origin/main` 기준) — 그래서 두 번 났다.
> ⇒ ★**`landed + 1 − lastRecorded ≥ 10` 은 머지 «전»에 계산된다** ⇒ 넘길 회차가 2-b 동봉에 함께 실으면 red 가 아예 안 난다.
> (그 계약 변경은 `~/orchestrator/templates/merge-ticket.tpl` 소관이라 이 repo 밖이다 — 후속으로 올렸다.)

## ★이후 관측 — 그 구조 서술이 «맞았다»
`check-worklog-coverage` 가 **착지 수를 `origin/main` 에서 세고 «기록»은 워킹트리에서 읽는다**는 성질은
2026-09-18 게이트③ 회차(`wie-adopt-slice-d-base-swap-fix3-p1-p0-p0-merge`)에서 다시 관측됐다 —
★**낡은 브랜치에서는 «항상» OVERDUE 로 보인다**(그 회차는 그것을 「main red」로 오독했다가 재서 정정했다).
⇒ 이 회차가 적은 「머지 직후에만 보인다」의 **다른 얼굴**이고, 같은 한 가지 사실이다.

## 사용자 영향
**기록 없음**(원장 기록 · 코드 0).
