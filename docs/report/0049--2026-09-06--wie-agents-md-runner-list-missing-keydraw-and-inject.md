## [2026-09-06] 러너 목록에 `keydraw_*` 와 `--inject` 를 적었다 — 「FAIL 이 정상」이 문서에 없어 회차가 멈췄다 (wie-agents-md-runner-list-missing-keydraw-and-inject)
- **무엇을**: `AGENTS.md` 러너 `sh` 블록에 **루프 1개(2줄)** + 「`--inject` 없이는 FAIL 이고 그것이 정답」 산문 **6줄**. ★**삭제행 0**(순수 추가 · 기존 러너 항목 무변경) · 픽스처·러너 코드 **무접촉**.
- **왜**: 운영자 채택 제안 `2026-09-06-rtrb-rustsec-2026-0274#p1`.
- **★⑴ 전제를 먼저 쟀다**(모집단 = `AGENTS.md` 332줄 · `/usr/bin/grep` 절대경로): `keydraw` ★**0건** · `--inject` ★**0건** · `wie_validate` 4건 ⇒ 되어 있지 않다. ★역설적으로 그 사실은 **코드에는 이미 있었다** — `wie_validate.rs:34` 가 「without `--inject` a key-driven fixture is black until a key arrives」라고 적는다. ⇒ ★**아는 사실이 «읽히는 자리»에만 없었다.**
- **★★⑵ 갈림을 «실행»으로 냈다**(두 캐리어 × 플래그 유무 4회):
  `keydraw_ktf` 없음 → ★**FAIL · content false · paints 1** ↔ `--inject` → ★**PASS · content true · paints 55**
  `keydraw_lgt` 없음 → **FAIL · false · 1** ↔ `--inject` → **PASS · true · 55**
  ⇒ ★**플래그 하나가 FAIL↔PASS 를 뒤집고 `paints` 가 1 → 55.** 그 FAIL 은 «입력이 없어 화면이 검은 것»이고 검사기는 옳게 말한 것이다.
- **★⑶ 오독이 실제로 났다는 사실을 함께 남겼다** — 어느 회차가 그 FAIL 을 «자기 회귀»로 읽고 멈췄다가 무접촉 트리에서 같은 FAIL 을 재현해 풀었다. ★규칙만 적으면 다음 사람이 지운다.
- **★⑷ 파리티 락 무영향**: 고친 자리가 `COMMIT-GATES` **마커 구간 «밖»**이다(그 END 마커가 스스로 「아래 `sh` 블록들은 conditional extras」라고 적는다). 실행 확인 — `cargo test -p wie_cli --test dod_ci_parity` **11 passed / 0 failed**. 문서에 적은 명령을 그대로 돌려 **둘 다 `result=PASS`** 확인.
- **사용자 영향**: 없음(문서). 대신 「내가 깨뜨렸나」로 멈추는 자리가 하나 닫힌다.
- **★남는 구멍**: ⒜러너 목록은 **여전히 사람이 손으로 유지**한다 — 파리티 락은 마커 «안»만 물고 이 블록은 밖이다(설계대로) ⇒ 드리프트를 무는 기계가 **없다**(제안 등재) ⒝`paints 55` 류 절대값은 **이 판본의 값**이다(논지는 «갈리는 방향»이지 수가 아니다).
- **★인접 — 고치지 않았다**: 바로 아래 「`cargo test --all` … nothing in it boots a J2ME guest」는 열린 **PR #102** 가 반증한다. ★그러나 **#102 가 미착지라 지금 `main` 에서는 그 문장이 «참»**이고, 먼저 고치면 거짓을 심는다 ⇒ **그 문장의 소유자는 #102 착지 회차**다.

