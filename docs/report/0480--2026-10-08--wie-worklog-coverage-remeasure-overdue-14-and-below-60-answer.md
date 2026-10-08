## [2026-10-08] worklog 재측 OVERDUE 14 해소 · 430 기록 60% 응답 (wie-worklog-coverage-remeasure-overdue-14-and-below-60-answer)

**무엇을**: `docs/worklog-coverage-remeasures.json` 에 재측 1행(착지 444 · `089eb964..2dec4625` · 7/10 = 70%)을 `--record` 로 붙였고,
직전 기록(430 · 6/10 = 60%) 행에 `reopened: true` 와 응답을 적었다. 판정 규칙 코드 변경 0.

**왜**: `check-worklog-coverage.mjs` 가 OVERDUE 14(경고)와 BELOW-UNANSWERED(rc=1)를 냈다. 머지 회차 둘이 결정을 묻지 않으려고 일부러 동봉하지 않았다.

**응답 — 재개했고, 결론은 조건부 유지.** 430 창의 worklog 없는 4회차(#487 · #493 · #494 · #497)는 넷 다 done 회신에
«제안 0(문턱 규칙)» 을 적었다 — 2026-09-26 §Proposal threshold(0건이 정상값)를 지킨 회차이고, AGENTS.md «Why 70» 이
이미 «후속이 없는 회차도 miss 로 센다» 고 적어 둔 정당 miss 다. 미준수 신호 0. 의무화하면 제안 0 회차에 빈 worklog 를
강제해 문턱 규칙과 부딪힌다. 444 창은 70% 로 돌아왔다(miss 3 중 #504·#505 는 제안 0 명시 · #510 은 회신에 언급 없음).

**사용자 영향**: 없음(원장만).

**재개 조건**: 다음 기록이 70% 미만이고 miss 중 «제안 0 명시» 가 아닌 회차가 절반을 넘으면 — 그때는 문턱 탓이 아니라 기록 누락이다.
