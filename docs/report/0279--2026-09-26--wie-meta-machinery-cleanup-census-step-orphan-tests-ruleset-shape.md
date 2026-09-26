## [2026-09-26] 판정 없는 기계 3개 + 컴파일 안 되는 테스트 3개 철거 (wie-meta-machinery-cleanup-census-step-orphan-tests-ruleset-shape)

**무엇을**
- `scripts/checker-census.mjs`(530줄) + `engine-contract.yml` 의 census 단계·주석 블록 삭제. 수동 도구로 남기지 않았다 — 찾은 «호출자 0» 목록이 처분된 적이 없어 남겨도 읽는 사람이 없다(`git grep` 한 줄이 대체).
- `wie_jvm_support/tests/absent_{string_buffer_insert,timer_schedule}.rs` · `wie_midp/tests/create_image_missing_name_message.rs` 삭제 — 두 디렉터리에 `Cargo.toml` 이 없어 cargo 가 빌드하지 않았다(#193 실측: 되살리면 `E0061`/`E0432`). 디렉터리째 사라진다.
- `check-branch-protection-claim.mjs` 325→약 150줄: ruleset 모양 지문(`.github/branch-protection-expected.json` · `--print-current` · `normalizeRuleset`) 제거, 필수 체크 5개 대조만 남김.
- 참조 정리: `AGENTS.md`(census 문단 · 지문 문단), `cargo-metadata.mjs`·`game-lab-census-map.mjs`·`ktf-image-sweep.py` 주석.

**왜** — 셋 다 실패 상태가 없거나(census), 빌드되지 않거나(테스트), 경보를 받을 사람이 곧 ruleset 을 바꿀 사람이라 재시드 부담만 만들었다(지문).

**측정**
- 수동 실행 `node scripts/check-branch-protection-claim.mjs` rc=0(5=5). 개악(`AGENTS.md` REQUIRED-CHECKS 블록에서 `build-web` 삭제) → rc=1, `build-web is REQUIRED on main but … does not list it`.
- doc-liveness 09-23 red(run 35887190550) 원인: 「Landing-paperwork commands」 안의 `check-worklog-coverage.mjs` 가 **OVERDUE**(landed 195 · 마지막 기록 185)로 rc=1. 명령 부패가 아니다. `origin/main`(`5434ab0a`) 도 지금 OVERDUE(235 vs 225) — 소유자 규칙상 gate③ 가 `--record`.

**사용자 영향** — 없음(엔진·게임 동작 변경 0).
