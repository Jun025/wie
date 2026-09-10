## [2026-09-10] doc-liveness 주간 잡 구현 — 문서 «문면 그대로»가 주 1회 돈다 (wie-doc-liveness-weekly-job-implementation)

2026-09-10 배치 결정(⒞ · `docs/report/0101`)의 구현. 채택 ref
`2026-09-10-doc-named-commands-liveness-placement#p0`.

### 무엇이 생겼나

- **`.github/workflows/doc-liveness.yml`** — schedule(**토 20:35 UTC = 일 05:35 KST · 주 1회**) +
  `workflow_dispatch`. AGENTS.md 의 fenced `sh` 블록 실행 가능 **25줄**(0100 census 와 동수)을
  `DOC-COPY` 영역 5곳에 **별칭 그대로** 복사해 돈다 — `npm run audit`(`bash scripts/…` 아님) ·
  `cargo +beta clippy`(matrix 표현 아님). 의도적 비실행 **2건**은 워크플로 안
  `# NOT-RUN: <줄> — <이유>` 로 선언: `gh pr checks <n>`(자리표시 인자) · `$EDITOR …`(대화형).
- **`scripts/check-doc-liveness-parity.mjs`** — 갈림 ⒜(복사+대조)의 드리프트 축.
  문서 펜스 ↔ 워크플로 사본+NOT-RUN 을 **다중집합**으로 대조(rc 0/1/2 · 2 = 측정불가이지 통과 아님).
  `engine-contract.yml` 상시 스텝으로 **매 PR** 돈다.
- AGENTS.md 결정 블록 갱신: 「잡 미실재」 문장 제거 · 워크플로명 채움 · ★**새 fenced `sh` 블록은
  이제 파리티 대상**(같은 PR 에서 사본 또는 NOT-RUN 을 함께 — 아니면 다음 PR 이 red) 고지.

### 갈림의 처분

- **⒜ 복사+대조 채택 / ⒝ 추출 실행 기각** — 추출 «실행»기는 markdown 파서가 새 기계이고 실패
  양식이 «쓰레기를 실행하거나 조용히 건너뜀»이다. 대조의 실패 양식은 red 체크다. 파서 성질의
  코드는 검사기에도 있지만 실패가 안전한 쪽(rc=2 측정불가)으로 떨어진다. `dod_ci_parity.rs`
  marked-region 선례를 따랐다.
- **`npm run verify` — 넣었다, 단 schedule 이벤트 전용**: 결정(0101)이 별칭 6건을 «⒞가 덮는다»로
  약속했으므로 빼지 않았다. 기본값(localhost:8788 wrangler dev — CI 에 세울 수 없음) 대신 문서
  자신의 프로덕션 형태(`WIE_BASE=https://wie-web.pages.dev`)를 스텝 env 로 준다(별칭 문면 불변).
  **외부 접촉 = 주 1회 상한** — dispatch 런은 이 스텝을 건너뛴다(티켓 범위 밖 조항 준수).

### 실측

- 파리티 green: 「OK — 25 documented line(s) … nothing extra」 rc=0.
- **양방향 변이**: 문서에 가짜 명령 추가 → rc=1(`DOC ONLY: cargo fake-doc-mutation --probe`) ·
  워크플로에서 `npm run audit` 제거 → rc=1(`DOC ONLY: npm run audit`). 원복 후 rc=0.
- `workflow_dispatch` 1회 실행 — 결과·소요는 done 회신에 인용(verify 스텝은 설계대로 skipped).

### 알고 남긴 것

- **차단력 0** — 이 잡은 착지 «뒤»에 운다(잡 머리주석·결정 블록 명시). 소유자 = 빨간 런 뒤 첫
  게이트③ 회차(`gh run list --workflow=doc-liveness.yml -L1` — AGENTS.md 에 활성 기재).
- verify 생존성의 실측은 주간 런에서만 나온다(그 명령은 배포마다 web.yml + 게이트③ 수동 대조가
  이미 돌리므로 사각은 «주간 문면 확인»의 지연뿐).
- 검사기의 메타 락은 없다 — 스텝+파일을 «함께» 지우면 조용하다(2차 락은 과기계로 판단, worklog
  limits 에 기재).
