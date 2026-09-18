## [2026-07-22] 게이트 사각 둘을 닫았다 — 컴파일 자산 경로, 키매핑 «우변» (wie-contract-gate-paths-key-mapping)

★**백필 문서다**(2026-09-18 · `wie-state-md-insertion-point-conflicts-every-open-pr`). 이 회차는 `docs/report/` 규약이
생기기 전에 착지해 회차 파일이 없었고, `STATE.md §완료` 의 **2026-07-22 항목이 네 회차를 한 줄로 묶고 있어** 가려져 있었다.
★**지어낸 칸 0** — 아래는 전부 착지 커밋 `540dae4f`(merged 2026-07-22T09:29:36Z · PR **#39**)에서 읽은 값이다.

## 무엇을
**⑴ paths 필터** — `data/**`(`binary_patches.toml` → `wie_core_arm` 의 `include_str!`)와
`fonts/**`(`neodgm.ttf` → `wie_backend` 의 `include_bytes!`)는 **wasm 산출물로 컴파일되는데**
`engine-contract.yml`(두 트리거 전부)과 `publish-artifact.yml` 의 필터에 **없었다**
⇒ ★자산만 바꾼 변경이 **PR 에서 조용히 미검증**이고 **착지 후 조용히 미발행**이었다.

**⑵ 키매핑** — 검사기가 `"KEY" =>` **좌변만** 봤다 ⇒ 오배선 `"UP" => KeyCode::DOWN` 이 **통과**했다.
이제 각 어휘 키의 match arm 을 `parse_key` 함수 본문으로 **범위 한정**하고 **우변이 `KeyCode::<KEY>` 와 같아야** 한다.

## 왜
`include_str!`/`include_bytes!` **전수 감사**(+`build.rs`/`include!` 스윕)로 그 둘이 **빠진 유일한 빌드 입력**임을 확인했다 —
나머지 두 자리는 `test_data/**` 가 이미 덮는 `tests/` 픽스처다.

## 판정·실측
- 검사 수 **불변**(48 pass · 클린 트리 위반 **0**)
- 로컬 개악 대조: `"UP" => KeyCode::DOWN` → **위반 1**(종전: pass) · `parse_key` 개명 → **명시적 unverifiable 위반**
- **fail-closed**: 함수를 못 찾거나 arm 이 모호·파싱 불가면 **명시 위반**이지 조용한 통과가 아니다
- 착지 diff: `.github/workflows/engine-contract.yml` +4 · `.github/workflows/publish-artifact.yml` +2 ·
  `scripts/check-engine-contract.mjs` +19 −3

## 사용자 영향
**기록 없음**(CI 게이트 변경 · 제품 표면 무접촉).
