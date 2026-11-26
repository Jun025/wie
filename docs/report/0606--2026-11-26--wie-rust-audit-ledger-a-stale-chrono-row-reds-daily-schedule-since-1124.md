## [2026-11-26] 일일 감사 사흘 red — 대장 §A 의 낡은 `chrono` 행(A-5)을 C-17 로 내리고 기대 목록에서 지웠다 (wie-rust-audit-ledger-a-stale-chrono-row-reds-daily-schedule-since-1124)

**무엇을**: 문서 2파일만 고쳤다(코드 · `Cargo.toml` · `Cargo.lock` · 워크플로 0줄).

| 파일 | 바꾼 것 |
|---|---|
| `.github/rust-audit-expected-warnings.json` | `expected` 에서 `chrono 0.4.45`(RUSTSEC-2026-0543) 1줄 삭제 · `measured` 를 `2026-11-26T07:16Z` 로 |
| `docs/project-kb/02_status.md` | §A 머리 «2건» → «1건» · A-5 행을 §C 의 **C-17(구 A-5)** 로 옮김(지우지 않았다 · 번호 재사용 0) · §A 머리에 2026-11-26 재측 줄 1개 |

**왜**: 일일 Security audit(schedule)가 2026-11-24 · 25 · 26 사흘 연속 failure 였다(run `45353827542` · `45563278336` · `45769680060`). 새 취약점이 아니다 — 세 run 모두 `cargo audit` 단계는 통과하고 그 다음 `Expected-warning list and ledger §A match reality` 단계에서 섰다. `45769680060` 의 실패 로그:

```
❌ 기대 목록에 있는데 cargo audit 이 «더는 내지 않는» 경고 1건 — 목록이 낡았다:
  - chrono@0.4.45 unmaintained (RUSTSEC-2026-0543)
§A 행 2건 · 경고 1건 · 절 머리 선언 2건
  - §A 에만 있다(경고가 없다 = 해소됐다): A-5 chrono@0.4.45 RUSTSEC-2026-0543
```

`chrono` 는 PR #616(조각 S1b · `docs/report/0584` · 2026-11-23 11:18Z 착지)이 lockfile 에서 뺐다 — RustJava 0.5.1 과 `wie-lgt` 가 날짜 계산을 `jiff` 로 옮겼다. 그날 schedule run(05:31Z · `45118309471`)은 착지 «전»이라 green 이었고 다음 날부터 red 다. 0584 는 「경고 전 2 → 후 1」을 적었지만 기대 목록과 §A 는 그 PR 에서 내리지 않았고, 이 대조는 일일 잡에서만 돌아 PR 에서 걸리지 않았다.

**C-17 에 남긴 판정**: 결함 경로가 있는 자문이 아니라 «유지보수 종료» 신호였다(구 A-5 그대로). 해소 전에는 배포 WASM 에 실려 있었고 지금은 없다(`cargo tree -p wie_featurephone --target wasm32-unknown-unknown -i chrono` → `did not match any packages`). 판본을 올려 닫은 것이 아니라 의존을 걷어낸 것이라, `chrono` 가 lock 에 돌아오면 경고도 돌아온다 — 돌아오는 길 셋(RustJava 핀 하향 · 우리 크레이트의 재의존 · lockfile 통째 교체)을 유효 범위 칸에 적었다.

**§A 머리에 적은 한 줄**(초안 할 일 3): lock 이 바뀌어 «경고가 사라지는» 경우는 PR 단계에서 안 걸린다. `pr-audit.yml` 은 `cargo audit` 만 돌리고 이 대조를 PR 에 걸지 않는 것은 기록된 결정이다(그 파일 머리 주석). 배선은 이 회차에서 하지 않았고, 대신 «크레이트를 lock 에서 빼거나 올리는 PR 은 그 PR 에서 `node scripts/check-audit-warnings.mjs` 를 손으로 돌려라» 를 적었다.

**검증**

| 무엇 | 결과 |
|---|---|
| `node scripts/check-audit-warnings.mjs --selftest` | 4케이스 전건 일치 · rc 0 |
| `node scripts/check-audit-warnings.mjs`(cargo-audit 0.22.2) | 고치기 전 rc 1(위 문면) → 고친 뒤 **rc 0** — 기대 1건 · 실제 1건 · 취약점 0건 · §A 행 1건 · 절 머리 1건 · 억제 0건 |
| `node scripts/check-ledger-c-validity-scope.mjs` | C 행 17건 전부 6칸 · 유효 범위 칸 채움 · rc 0 |
| 브랜치에서 `rust-audit.yaml` 손 실행 | run id 와 결론은 PR 본문에 적는다(이 파일을 커밋한 뒤에 돈다) |

**사용자 영향**: 없다. 게임이 도는 방식 · 배포 아티팩트 · 지원 목록은 그대로다. 매일 도는 보안 감사가 다시 green 이 되어, 다음에 진짜 취약점이 떴을 때 red 가 묻히지 않는다.

**게임 파일명 유입**(`node scripts/corpus-name-inflow.mjs` · 대상 3파일): BOUNDED 18회/12쌍 · SUFFIX-ATTACHED 2회/2쌍 — 전부 `docs/project-kb/02_status.md` 에 이 회차 전부터 있던 언급이다(도구는 고친 파일의 «본문 전체»를 읽는다). 이 브랜치가 «더한 줄»에는 0건이다(`git diff origin/main...HEAD` 의 `+` 줄을 그 제목들로 대조).

**남은 것**: 착지 뒤 첫 schedule run 이 green 인지는 `-merge` 회차가 본다. 남은 경고는 `ttf-parser 0.25.1`(A-1) 하나이고 이 회차는 건드리지 않았다.
