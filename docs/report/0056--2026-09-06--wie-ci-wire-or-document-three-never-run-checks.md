## [2026-09-06] 「한 번도 돈 적 없는」 검사 3건 — 하나는 배선하고 둘은 «로컬 전용»이라고 적었다 (wie-ci-wire-or-document-three-never-run-checks)
- **무엇을**: `engine-contract.yml` 에 ★**네 번째 상시 스텝**(`bash scripts/audit-no-leak.sh`) + `AGENTS.md` 에 ★**「Which of these CI actually runs」 절 신설**(`verify-browser.mjs`·`smoke_gate.sh` 가 왜 로컬 전용인지) + Constraint 9 행의 「Locked by」를 실제 집행자로 정정. ★**`.rs` 0 · 새 의존성 0.**
- **왜**: 운영자 채택 제안 `2026-09-06-deletable-checks-census#p0`.
- **★★대전제 ⓐ 를 «먼저» 쟀다**(모집단 = 워크플로 **8파일** 전수 · `/usr/bin/grep` 절대경로): 세 스크립트의 경로 히트 **전건 0** · 워크플로 안의 `npm run` 은 **`npm run build` 1건뿐**(`web/` 자신의 빌드) ⇒ **우회 호출 경로도 없다.** 「이미 되어 있다」가 아니었다.
- **★★⑴ 셋을 한 덩이로 묶지 않았다**(계약 1). **⒜ 배선**: `audit-no-leak.sh` 는 ★**문서가 집행을 약속하는데 집행이 없던** 자리다 — `AGENTS.md` Constraint 9 행이 「Locked by」로 적는데 어느 워크플로도 부르지 않았다. 오프라인·의존성 0 이라 **스텝 1줄**로 끝난다.
- **★⑵ 필터 «밖»에 둔 이유**: 지키는 것이 **repo 전역 성질**(추적 게임 바이너리 0 · 네트워크 프리미티브가 `lib/api.ts` 에만 · `rom_hash` 유일성이 소유자별)이라 ★**그것을 어기는 diff 가 엔진 경로를 하나도 안 건드릴 수 있다.** 필터 안에 두면 정확히 그 PR 에서 건너뛴다.
- **★★⑶ ⒝⒞ 는 «적었다» — 그리고 그 근거를 소스에서 다시 뽑았다**: `verify-browser.mjs` 는 ★**게임 파일이 필요 없다**(기본 인자가 커밋 픽스처 `helloworld_ktf.zip` — 원 제안의 「게임 파일 필요」는 **부정확**) · 진짜 비용은 **실 Chrome(`channel: "chrome"`)** 과 **`functions/` 까지 서빙하는 Pages dev 스택**이고, ★**PR 에는 가리킬 URL 이 «없다»**(`web.yml` 배포 스텝이 전부 `event_name == 'push'` 게이트). `smoke_gate.sh` 는 ★**선행 판단 「구조적 불가」를 반증하려다 실패**했다(계약 2): `WORKING_DIR` 기본값 `game_lab/`이 **`.gitignore:23` · 추적 0건 · 부재**이고 그 내용물이 Constraint 9 가 금지한 실게임 바이트다.
- **★⑷ 양방향 — 격리 워크트리에서 개악 2종이 «물었다»**: 기준선 **rc=0** ↔ **M1**(device-local `library.ts` 에 raw `fetch(`) **rc=1 · ❌ 2줄** ↔ **M2'**(0바이트 `decoy.jar` 를 `git add -f` 로 추적 편입) **rc=1 · ❌ `game binaries tracked in git`**. 원복 후 rc=0 · 트리 clean · **커밋 0**.
- **★★⑸ 계획에 없던 관측 — 개악이 «막혀서» 배운 것**: M2 의 첫 시도는 맨 `git add` 였고 ★**`.gitignore` 가 거부**했다. ⇒ Constraint 9 의 두 층에는 **순서가 있다** — 블록리스트가 먼저 막고, 스크립트의 `git ls-files` 검사는 ★**`-f` 로 그것을 뚫었을 때만** 발화하는 **둘째 선**이다.
- **대가**: CI **+0.26~0.34s**(로컬 3회 실측 · `node_modules` 없이 돎 · 네트워크 0). 같은 잡의 기존 상시 스텝 3개와 같은 급이다.
- **사용자 영향**: 없음(CI). 대신 ★**「유출 방지 검사가 있다」가 「돈다」와 같아진다.**
- **★남는 구멍**: ⒜`web/dist` 스캔은 CI 에서 **자기 스킵**된다(이 스텝이 프런트 빌드 전/없이 돌기 때문) — 덮으려면 `web.yml` 빌드 뒤 한 줄이고, ★**한계 효용이 낮다고 봐서 넣지 않았다**(제안 등재) ⒝`npm run verify` 는 여전히 **아무도 안 돌린다** — 「왜 안 도는가」를 고정했을 뿐이다(제안 등재).

