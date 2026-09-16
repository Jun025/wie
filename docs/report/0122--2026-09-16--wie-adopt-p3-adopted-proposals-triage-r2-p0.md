## [2026-09-16] 코퍼스 카드에 «첫 질문»을 넣었다 — 그리고 그 자리가 본문이면 안 되는 이유 (wie-adopt-p3-adopted-proposals-triage-r2-p0)

### 무엇을

채택 제안 `2026-09-16-p3-adopted-proposals-triage-r2#p0`(원문 =
`docs/worklog/2026-09-16-p3-adopted-proposals-triage-r2.json` `proposals[0]`)을 집행했다.
고친 파일은 **하나**다 — `~/orchestrator/humansteps/wie-p2-corpus-placement.md`:

- `what:` 에 한 절 — 「★**전부가 아니어도 됩니다 — `lgt/` 52건이 먼저이고, 그것만으로 지금 막힌 질문 하나가 닫힙니다**」
- `how:` 에 여덟 줄 — 「제일 먼저 확인할 것은 **첫 질문 하나**」 + 그 질문의 문면 + **답을 내는 명령**
  (`WORKING_DIR=game_lab/working PLATFORM_FILTER=lgt scripts/smoke_gate.sh`) + 통과/실패가 각각 무엇을 뜻하는지
  + 「ktf·skt 는 이 질문에 답하지 않는다」.

**wie 제품 코드 0줄.** 이 저장소의 변경은 원장 3파일(이 리포트 · worklog · `STATE.md`)뿐이다.

### 왜 — 그리고 «어디에» 가 이 회차의 실측이다

제안은 「카드에 한 줄을 덧붙여라」였고 **어느 필드인지는 말하지 않았다**. 그 선택이 실효를 가른다:

★**카드의 markdown 본문은 운영자 화면에 «닿지 않는다».** `bin/humanstep-scan` 은 frontmatter 만
파싱하고 본문을 **아예 읽지 않는다**(실측: 그 파일에 문자열 `body` **0건**). `--json` 이 내는 키는
15종 — `id title why how blocks origin status cost since whereLabel what readiness blocked_by age_days verify` —
이고 **본문이 없다**. 그 payload 를 `bin/pipeline-feed` 가 `human_steps` 로 그대로 싣는다.

⇒ 제안이 말한 「한 줄」을 본문에 적었으면 **집행은 명목상 끝나고 실효는 0**이었다. 개악 대조 ①이
정확히 그 형태이고 **RED 로 재현된다**(아래).

이 사각은 이 카드 한 장의 문제가 아니다 — 카드 **99장 중 57장**이 본문 3줄 이상을 갖고 있고 합 **2,254줄**이다.
그 축은 이 회차가 **세기만 했고 고치지 않았다**(후속 제안 `#p0` · `target: orchestrator`).

### 대전제 재고 — 제안이 «지금도» 참인가

제안 원문은 base swap(PR #161) **이전** 관측 위에 서 있다. 세 축을 다시 쟀다.

| 제안이 말한 것 | 재측(2026-09-16 · `origin/main` `3a1723bf`) | 판정 |
|---|---|---|
| 카드가 이미 `status: open` 인데 «무엇을 물을지»가 없다 | 카드 실재 · `status: open` · `blocked_by: decision` · `첫 질문` 문자열 **0건** | ★그대로 참 |
| 코퍼스 분할 ktf 190 · lgt 52 · skt 50 = 292 | `scripts/smoke_gate_baseline.tsv` → **190 / 52 / 50 = 292** | ★그대로 참 |
| 그 답이 「조각 D 의 graphics 27줄 결정」을 정한다 | ★**낡았다** — 결정은 이미 났다(⒝ 집행 · PR #161) | ★**문면을 갈아탔다** |

세 번째가 이 회차가 제안을 그대로 베끼지 않은 자리다. `wie-lgt/src/runtime/wipi_c.rs` 는
`=> wie_wipi_c::api::graphics::` **27** · `=> graphics::` **0** ⇒ ⒝(공용 구현 복귀)가 **이미 집행됐다**.
그러나 질문은 **죽지 않았다** — 조각 D 자신이 「⒜/⒝ 중 무엇이 실게임에 옳은지는 이 회차도 못 잰다」를
`limits` 에 남겼고, upstream 의 LGT 전용 `graphics.rs` **1,095줄**은 배선만 끊긴 채 **트리에 남아 있다**
(「연기이지 취소가 아니다」). ⇒ 질문의 형태가 **「무엇을 고를까」에서 「고른 것이 맞았나」로** 바뀌었을 뿐이고,
답을 내는 코퍼스는 여전히 **lgt 52건**이다. 카드 문안은 그 오늘자 형태로 썼다.

### 무엇을 잃는가 / 안 하면 무엇이 나쁜가

- **잃는 것 ⑴ 카드가 길어진다.** `how:` 가 5줄 → 13줄이다. 운영자가 읽어야 하는 분량이 늘고,
  ★**「차단 아님」이라는 이 카드의 계급이 흐려질 위험**이 있다(길면 무겁게 읽힌다). 그래서 추가분은
  ③ **아래**에 두어 「전부 없어도 된다」를 먼저 읽게 했고, `blocks: []` 와 `status` 는 건드리지 않았다.
- **잃는 것 ⑵ 이 문안은 썩는다.** 「⒝ 를 집행했다(PR #161)」와 「1,095줄이 트리에 남아 있다」는 **오늘의 사실**이다.
  둘 중 하나가 바뀌면 카드가 옛 답을 띄운다. `HUMANSTEP_ANSWER_STALE` 은 **날짜**로만 잡고 **내용**은 못 본다.
- **안 하면**: 292건이 도착해도 «무엇부터»가 없어 회차가 헤맨다. 그리고 요청량이 292 로 남아 운영자가
  응할 문턱이 그대로다 — ★제안이 든 실익의 절반은 「**52면 된다**」를 카드가 말하는 것이다.
- ★**「잃는 것이 없다」가 아니다**: 위 ⑵는 실재하는 부채이고, 이 회차는 그것을 **갚지 않고 기록만** 했다.

### 검증 — 양방향 개악 대조

술어는 **소비자를 통과한다**: `bin/humanstep-scan --json` 의 이 카드 `how` 필드에 마커 2종
(`PLATFORM_FILTER=lgt` · `첫 질문`)이 있는가. ★그 도구가 tower `human_steps` payload 의 생산자다
(`bin/pipeline-feed` 가 `--argjson hsteps` 로 싣는다) — **상수 대 상수가 아니다.**

| 회차 | 술어 결과 |
|---|---|
| 수정 «전»(카드 원본) | `markers-in-how=0` → **RED** |
| 수정 후 | `markers-in-how=2` → **GREEN** |
| ★**개악①** — 추가 블록을 `how:` 에서 **markdown 본문으로 이동** | `markers-in-how=0` → **RED** |
| 원상 | `markers-in-how=2` → **GREEN** |

개악은 `ORCH_OPS=$S`(격리 사본)에서 돌렸고 **라이브 카드는 무접촉**이다.
스키마 무회귀: `humanstep-scan` rc=0 · 이 카드에 대한 WARN **0**(유일한 WARN 은 무관한
`macos-27-major-upgrade-decision` 의 `readiness` 결손 — 선재).

회귀 0(★`tail` 아님 — `test result:` **42줄 전부**를 `awk` 로 합산):

```
cargo fmt --all -- --check                                    OK
cargo clippy --all -- -D warnings                             OK
cargo clippy --target wasm32-unknown-unknown -- -D warnings   OK
cargo +beta clippy --all -- -D warnings                       OK
RUST_MIN_STACK=4194304 cargo test --all   42 스위트 · 384 passed · 0 failed · 0 ignored
```

엔진 코드 무접촉이라 `wie_validate` 러너 블록은 돌리지 않았다(AGENTS.md 는 그 블록을
「Touching engine code?」에 건다).

### 범위 밖 — 일부러 안 한 것

- 형제 `#p*` 없음(이 worklog 의 `proposals` 는 1건뿐이다).
- 카드 `status` 를 `done` 으로 바꾸지 않았다 — **질문을 준비했을 뿐 답이 오지 않았다.**
- 본문 2,254줄(57장)을 옮기지 않았다 — 제안을 넓히는 일이고, 후속 제안으로 뺐다.
- ★**카드는 orchestrator-ops 에 untracked 다** — PR·머지 대상이 아니고, 이 wie PR 을 되돌려도
  카드는 되돌아가지 않는다(되돌리기는 그 파일의 역편집이다).
