## [2026-10-11] 조각 E — PR #162 는 살리지 않고 닫는다: 그것이 잰 결함은 #161 사슬이 이미 고쳤다 (wie-p3-slice-e-pr162-rebase-or-close-after-161)

**무엇을** — 처분 전용(제품 코드 0줄). PR #162(head `3bf1bfbd` · 2026-09-16 · 게이트② approve)를 **닫았다**.
STATE.md «다음» 의 조각 E 줄과 `docs/upstream-realign-p3-slices.md` §6 E 배너를 그 결과로 고쳤다.

**왜** — #162 는 코드 PR 이 아니라 **#161 «착지 전» head `f533ba54` 를 잰 조사 기록**이다(리포트 0116 · 4파일 전부 문서).
그 피검체는 착지하지 않았고, #161 은 `-fix2`/`-fix3` 를 거쳐 다른 head 로 착지했다(`37734e74`).
#162 head 는 #161 을 포함하지 않고 `origin/main` 보다 **3,261 커밋** 뒤다.
`git merge-tree --write-tree origin/main 3bf1bfbd` → **CONFLICT 2**(`STATE.md` · `docs/upstream-realign-p3-slices.md`) —
두 파일 다 그 뒤 회차들이 같은 줄을 다시 썼다. 재적용하면 «미착지 트리의 결함 목록»을 현재형으로 들여오게 된다.

### 결함별 대조 — `origin/main` `c1572187` 에서 실측

| #162 결함 | 지금 main | 근거 |
|---|---|---|
| F1 `wie_featurephone` 컴파일 불가(어댑터 4축) | ★**해소** | `platform.rs:76 fn font` · `screen.rs:68 fn resize` · `audio.rs:167 fn send` · `database.rs:60 fn usage` 실재 |
| F4 `check-engine-contract.mjs` 옛 로케이터 | ★**해소** | `:157` `wie-midp/…/event_queue.rs` · `:192` `wie-wipi-java/…/card_canvas.rs` (리포트 0119 · 107 pass) |
| F5 고아 시험 5건 | ★**처분됨** | 리포트 0153 이 전수(4건 술어)로 셌다 · `wie-j2me/tests/test_boot.rs` · `wie-wipi-java/tests/preload_…` 되살림 |
| upstream `release.yaml` 무장 위험 | ★**해소** | 파일 머리 `PARKED` + `DECIDED 2026-09-16: NOT ADOPTED` |
| F3 #161 CONFLICTING 이라 PR 워크플로 미발화 | ★**소멸** | #161 착지 — 그 형상이 더는 없다 |
| F2 wasm clippy 레그가 `wie_featurephone` 을 안 본다 | ★**남았다(부분)** | `rust.yml:104` 는 여전히 `--all` 없이 `default-members = ["."]` 만 린트. ★단 **컴파일**은 required `build-web`(`web.yml`, `paths:` 없음)이 매 PR 잡는다 ⇒ 남은 사각은 «wasm 타깃 경고»뿐 |

★**산출물 이름 `wie_web.js`/`wie_web_bg.wasm` 불변** — 이 회차는 `scripts/`·`docs/contracts/` 를 건드리지 않았다.

**사용자 영향** — 없다. 문서 2곳이 «PR #162 OPEN» 대신 «닫힘»을 말한다.
