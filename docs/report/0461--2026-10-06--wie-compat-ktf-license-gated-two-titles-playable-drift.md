## [2026-10-06] compat 정정 — KTF 2판(974e0df9ab1e · f770b15f8876) playable → limited · 설치 인증 벽 안내 (wie-compat-ktf-license-gated-two-titles-playable-drift)

**무엇을**: 0457 이 «영구»로 판정한 슬롯 13(설치 라이선스 대조) 벽에 걸리는 두 판을 census 로 다시 재고, `compat.json` 두 행을 실측값으로 고쳤다. census 의 손 지정 벽 표(`NET_HAND` → `HAND_WALL` 로 이름만 넓힘)에 두 sha 를 넣어, 다음 `report` 재생성도 같은 안내 1줄과 «최대 limited» 를 낸다.

**왜**: 게이트② 지적(`wie-ktf-knl-get-access-level-slot13` 검수 F1) — `974e0df9ab1e` 가 `playable` 로 남아 있었다. 0449 §5 가 census 재측정으로 미룬 drift 다.

**사용자 영향**: 지원 현황 페이지에서 두 판이 «플레이 가능» 대신 «일부 가능»으로 보이고, 이유를 «설치 인증(구매 기록)을 확인하는 단계에서 멈춰서, 여기서는 그 뒤로 진행할 수 없어요.» 로 안내한다. 게임 동작은 바뀌지 않는다(이용자 소식 없음 — 바뀐 것은 표기뿐이다).

### 측정
- 바이너리: head 의 `wie_validate`(release). `scripts/playability-census.mjs run` — 이 2판만 담은 임시 코퍼스 · probe A/B 30초 · `build-slot run --long`(long 풀 만석으로 대기 후 실행) · load1 8.7~9.1.
- 결과(두 판 동일): A = `FAIL` · `stop: error` · 16/27 키 · paints 161 · `Unimplemented: 13: MC_knlGetAccessLevel`(0457 base 와 같은 자리). B(무입력) = `PASS` · deadline · paints 433.
- 판정: boot ok · render ok · input ok · longplay error(30초 probe 가 그린 뒤 실패 → 장주행 없이 error) · sound ok · speed ok · status **limited**.
- 증적(repo 밖): `~/orchestrator/reports/evidence/wie-compat-ktf-license-gated-two-titles-playable-drift/`(A/B JSON · census.tsv · 생성 compat.json).

### compat.json 반영 (계약 어휘로)
| sha12 | status | input | longplay | sound | speed |
|---|---|---|---|---|---|
| 974e0df9ab1e | playable → **limited** | ok | ok → **no** | no → **ok** | unknown → **ok** |
| f770b15f8876 | limited | no → **ok** | unknown → **no** | no → **ok** | unknown → **ok** |

knownIssues_ko 는 두 행 모두 위 안내 1줄로 바꿨다(벽 앞의 다른 줄은 «아직 고치는 중»으로 읽히므로 서버 벽과 같은 처리). progress 는 재지 않아 키를 넣지 않았다. `node scripts/check-compat-revert.mjs` → 착지 기준 바뀐 행 **2**.

### 검증
4게이트(fmt · clippy · wasm clippy · `RUST_MIN_STACK` test) green · `playability-census.mjs selftest` 62/62 · `player-data.mjs` OK · `check-worklog-json` OK.
게임명 유입: 이 문서·스크립트만 재면 BOUNDED 0 · SUFFIX-ATTACHED 0. 아래 표식의 B=718/330 · S=35/15 는 `compat.json` 전체 본문(기존 제목 필드)이고 0459 표식과 같은 값이다 — 이 회차가 더한 제목은 없다.

