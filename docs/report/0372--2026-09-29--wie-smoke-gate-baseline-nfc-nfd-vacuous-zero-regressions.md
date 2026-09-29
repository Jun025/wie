## [2026-09-29] smoke_gate — 제목 대조 전 NFC 정규화 · absent 과반이면 UNMEASURED (wie-smoke-gate-baseline-nfc-nfd-vacuous-zero-regressions)

**무엇을**: `scripts/smoke_gate.sh` 가 기준선 대조 전에 양쪽 제목(코퍼스 결과 · 기준선)을 한 헬퍼 `nfc()`
(`perl Unicode::Normalize`)로 NFC 정규화한다. 범위 안 기준선의 절반 넘게 코퍼스에 없으면 `OK` 대신
`UNMEASURED` · rc=2. `PLATFORM_FILTER` 밖 플랫폼 제목은 absent 로 세지 않는다(그래야 `lgt` 만 돌려도 UNMEASURED 가 안 난다).
자체검사 `scripts/test-smoke-gate.sh`(가짜 validator · 게임 바이트 0). 기준선 내용 변경 0.

**왜**: APFS 가 돌려주는 코퍼스 파일명은 NFD(ktf 190/190 실측), 기준선은 NFC(292 중 291 · 1줄만 NFD).
바이트 대조라 0~1건만 겹쳤고 게이트는 «못 잰 채 OK» 를 냈다(2026-09-23 · 2026-09-29 두 회차 독립 관측).

**사용자 영향**: 없음(로컬 전용 게이트). 개발자가 보는 «0 regressions» 가 이제 실제로 잰 값이다.

### 전/후 (실코퍼스 · `PLATFORM_FILTER=lgt` · debug `wie_validate` · RETRY=2)
- 전(origin/main 스크립트 · 후-회차와 **같은 판정**을 재생해 대조만 비교): `checked 0 baseline titles, 292 absent, 0 regressions` → `OK` rc=0
- 후: `checked 52 baseline titles, 0 absent, 5 regressions` → `FAIL` rc=1
- 그 5건을 곧바로 단건 재실행(load1 75~145): **5/5 × 2회 PASS** ⇒ 부하 기아. 본 회차 load1 은 190~500 이었다.
  즉 회귀 0 에 가깝고, 게이트가 이제 부하 기아를 «보이게» 됐다(전에는 아무것도 안 봤다).
- 전 플랫폼 실행은 load ~490 에서 2시간+ 예상이라 중단하고 LGT 로 좁혔다 — KTF·SKT 대조는 이 회차에 재지 않았다.

### 픽스처 (`bash scripts/test-smoke-gate.sh`)
```
ok   NFD corpus vs NFC baseline      checked 3, 0 absent, 0 regressions · rc=0
ok   planted regression              checked 3, 0 absent, 1 regressions · rc=1
ok   absent 2/3 → UNMEASURED         rc=2
ok   mutant (no NFC) is not OK       checked 0, 3 absent · UNMEASURED rc=2
```
