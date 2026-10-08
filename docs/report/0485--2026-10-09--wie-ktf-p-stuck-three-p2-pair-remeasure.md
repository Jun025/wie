## [2026-10-09] KTF P stuck 3종 P2 짝 재측 — ok 1 · stuck 2 (wie-ktf-p-stuck-three-p2-pair-remeasure)

### 무엇을 · 왜
- 0479 §3 에서 600초 P 가 stuck 인 3종(`3c658a46bbfb` `55aadf368b8e` `b907b0faf483`)은 P2 를 재지 못했다. census 잠금 대기 때문이다. census 규칙(`progressAxis`)은 «P2 없는 stuck»을 `n/a` 로 본다. 그래서 compat 에 `progress` 가 없었다.
- 이 회차는 P2 만 쟀다. 판정을 확정하고 compat `axes.progress` 를 규칙대로 넣었다.
- 운영자 채택 제안: `2026-10-08-progression-wave5-ktf-suspects-and-unmeasured#p0`.

### 측정
- 엔진: 0479 의 release `wie_validate`(`11a6fac3`). `git diff 11a6fac3 origin/main`(`5137d54c`)는 문서 6파일만 바꾼다. 엔진 크레이트·`Cargo.*`·`patches/` 변경은 0이다. ⇒ 현 main 핀과 같은 엔진이다.
- 도구: `scripts/playability-census.mjs`(현 main · 0479 사본과 바이트 동일). `--only progress --progress 600 --as P2` · 정책 v2.
- A·B·L·P 는 0479 출력을 그대로 썼다(같은 엔진·같은 도구). 그래서 P 와 P2 가 같은 핀의 짝이다.

| sha12 | P 정체(초) | P2 정체(초) | 판정 | 근거 |
|---|---|---|---|---|
| `3c658a46bbfb` | 500 | 50 | **ok** | P2 는 약 550초에 새 화면(지도 같은 게임 안 화면)에 닿았다. P 는 방 ↔ 메뉴 ↔ 저장 슬롯만 오갔다. 둘 다 예외 0 |
| `55aadf368b8e` | 290 | 290 | **stuck** | 정체 초가 P 와 같다. 튜토리얼 되풀이 · 예외 0 |
| `b907b0faf483` | 560 | 560 | **stuck** | 정체 초가 P 와 같다. 타이틀 메뉴 LOAD ↔ OPTION · 예외 0 |

- P2 세 런 모두 `PASS` · stop `deadline` · stderr 의 panic/error 0줄.
- stuck 2종은 정체 초가 P 와 초 단위로 같다. 부하 탓 지연이 아니라 같은 자리에서 막힌다는 뜻이다.
- `3c658a46bbfb` 가 ok 로 바뀐 것은 정책 v2 의 섞기(새 화면이 60초 없으면 뒤로·소프트키·다음 항목)가 런마다 다른 길로 가기 때문이다. 규칙상 «움직인 짝»이 정체를 뒤집는다(`progressAxis`).

### 측정 자원
- long 임대를 phase 단위로 1건 잡았다(`build-slot run --long -- <P2 phase>`).
- 임대 요청 00:05:12 → 획득 00:22:37. long 2칸이 형제 census 로 만석이라 **17분** 기다렸다.
- 임대 안에서 census 호스트 잠금(pid 4351 · 형제 레인)을 00:22:37 → 00:57:43 **35분** 기다렸다.
- 실행 00:57:43 → 01:21:44(24분 · jobs 2 · 600초 × 3종). 임대 반납 01:21:44. load1 은 획득 때 85, 반납 때 27.
- `nohup` 0. 끝난 뒤 내 프로세스 0.

### compat.json
- 바뀐 행 3: `axes.progress` 없음 → `ok` 1(`3c658a46bbfb`) · `stuck` 2(`55aadf368b8e` `b907b0faf483`).
- `knownIssues_ko` 는 손대지 않았다. 엔진 코드 0. 진행 방식이 바뀐 게 아니라 측정만 새로 생겼다. 그래서 `docs/player-updates/` 항목은 없다.

### 사용자 영향
- 세 게임의 진도 칸이 빈칸에서 «진행됨» 1 · «막힘» 2 로 바뀐다.
