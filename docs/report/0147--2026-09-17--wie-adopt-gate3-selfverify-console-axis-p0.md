## [2026-09-17] verify-browser 가 repo 루트에 스크린샷 2장을 쓴다 — 그 부작용을 문서가 말하게 했다 (wie-adopt-gate3-selfverify-console-axis-p0)

**무엇을**: 채택 제안 `2026-09-17-gate3-selfverify-console-axis#p0` 의 이행. `AGENTS.md` §Web-surface commands 의
`npm run verify` 불릿 끝에 **문단 하나**를 더했다 — 그 스크립트가 repo 루트에 남기는 두 파일의 **정확한 이름**,
**커밋하지 않는다**는 것과 그 근거(`.gitignore` 의 `/verify_*.png`), 보존용 스크린샷이 사는 곳(`docs/verification/`),
그리고 **왜 여기 적는가**(게이트② 검수자가 실제로 놀라 손으로 지우고 신고했다).

**왜**: 결함이 아니라 **문서 결손**이다 — `.gitignore:35` 가 이미 덮으므로 아무것도 새지 않는다.
새는 것은 **사람의 시간**이다: 손으로 돌린 회차가 매번 「이거 커밋해야 하나」를 다시 확인한다.
★**이 회차 자신이 그 증거다** — 오늘 게이트③ 회차가 `npm run verify` 를 돌리고 그 2장을 **손으로 지웠다**.

**사용자 영향**: 없다. **제품 코드 0줄 · 스크립트 0줄 · CI 0줄** — 바뀐 것은 `AGENTS.md` 한 문단뿐이다.

### 대전제 — 제안을 «먼저 반증하려» 했고, 실패했다
- ⒜**지금도 참인가**: 참. `origin/main edfe108e` 의 `scripts/verify-browser.mjs:60-61` 이 여전히
  `path.join(root, …)` 로 두 png 를 쓴다. 부작용 서술은 **0건**
  (`git grep -e 'verify_\*' -e 'verify_.*png' -e screenshot` 전수 = `.gitignore` 2 · 스크립트 2 · 무관 3 · 제안 자신 2).
- ⒝**이미 같은 축이 있나**: `.gitignore:35` 는 **무시**만 하지 **고지**하지 않는다.
  ★같은 불릿을 만지는 **열린 PR #184** 의 `AGENTS.md` 헝크(`@@ -292,14 +292,33 @@`)를 **전문 확인** — 부작용 서술 **0건** ⇒ 중복 아님.
- ⒞**제안이 틀렸나**: 아니다. 위 둘로 확인됐고, 제안 자신이 「결함이 아니라 문서 결손」이라고 **정확히** 분류했다.

### 무엇을 잃는가 / 안 하면 무엇이 나쁜가 (계약 2)
- **잃는 것**: `AGENTS.md` 가 **7줄** 길어진다. ★**이 파일에서 그것은 공짜가 아니다** — 이 문서는 이미
  길고, 모든 회차가 읽는다. 그래서 제안의 처방(「한 줄」)에 가깝게 **문단 하나**로 묶고,
  파일명·근거·계기를 **한 번씩만** 적었다(중복 0). 그리고 ★**펜스 블록은 건드리지 않았다** —
  `check-doc-liveness-parity.mjs` 가 그 25줄을 `doc-liveness.yml` 사본과 대조하므로 거기 손대면 parity 가 깨진다.
- **안 하면**: 같은 놀람이 **회차마다 재발**한다. 비용은 회차당 몇 분이고 영구적이다.
  ★그리고 더 나쁜 갈래가 있다 — 누군가 `.gitignore` 를 못 믿고 **`git add -A` 전에 손으로 지우는 습관**을
  들이거나, 반대로 **커밋해 버린다**(오늘 `.gitignore` 개악 실측이 보여 주듯 그 줄 하나가 사라지면 즉시 `??` 로 뜬다).

### ★개악 대조 — 양방향, 제품 «호출부»에서
문서만 고쳤으므로 「내 문장을 검사하는 테스트」는 만들지 않았다(그것은 증거가 아니다).
대신 ★**그 문단이 주장하는 두 사실을 각각 그 사실의 «주인 파일»에서 무너뜨렸다**.

| 축 | 개악 자리 | 정상(green) | 개악(red) | 복원 |
|---|---|---|---|---|
| **A** 「repo 루트에 이 두 이름으로 쓴다」 | `scripts/verify-browser.mjs:60-61` 의 `path.join(root, …)` → 임시 디렉터리 | repo 루트 **2장**(`verify_helloworld_ktf.zip_{screen,page}.png` · rc=0) | repo 루트 ★**0장**(파일은 임시 디렉터리로 갔다) ⇒ 문단이 **거짓**이 된다 | `git show HEAD:…` 와 **바이트 동일** |
| **B** 「`.gitignore` 가 덮어 `git status` 가 깨끗하다」 | `.gitignore:35` `/verify_*.png` → `#MUT …` | `git status --porcelain -uall` 의 `verify_` 줄 ★**0** · `git check-ignore -v` 가 **:35** 를 두 파일 모두에 지목 | ★**2줄** (`?? verify_…_page.png` · `?? verify_…_screen.png`) ⇒ 「깨끗하다」가 **거짓** | 다시 **0** |

★**두 축 모두 «상수 대 상수»가 아니다** — A 는 실제 브라우저 실행이 만든 파일을, B 는 git 자신의 판정을 읽는다.

### 회귀 0 — 전건 합산(★`tail` 미사용)
`cargo fmt --all -- --check` **rc=0** · `cargo clippy --all -D warnings` **rc=0** · wasm clippy **rc=0** ·
`cargo +beta clippy --all -D warnings` **rc=0** ·
`RUST_MIN_STACK=4194304 cargo test --all` **rc=0** — `^test result:` 행 **전수 합산** = ★**43타깃 · 386 passed · 0 failed · 0 ignored**.
node 검사기 **8/8 rc=0**(worklog 124파일 · doc-liveness parity 25줄 전건 커버 · 연번 139건 중복 0) · `npm run audit` **PASSED**.
