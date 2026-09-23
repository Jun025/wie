## [2026-09-24] broken/lgt 도 ⒜/⒝ 차이 0 — 조각 D 의 benefit 절반을 쟀고 «0» 이다 (wie-2026-09-23-lgt-corpus-decides-graphics-wiring-adopt-p2)

**무엇을** — 측정 전용(**제품 코드 0줄**). `0222` 가 남긴 「남은 절반」 — `game_lab/broken/lgt` 를 두 graphics 배선
(⒜ upstream LGT 전용 · ⒝ 공용 = 현행)으로 돌려 「⒜ 가 우리가 못 여는 타이틀을 여나」를 쟀다.

**왜** — `docs/upstream-realign-p3-slices.md` §D 의 ⒜/⒝ 저울은 cost(`0222`: working/lgt 54건 차이 0)만 닫혀 있었고
benefit 은 「있다」도 「없다」도 측정되지 않았다.

**사용자 영향** — 없음(코드 0줄). §D 의 권고 ⒝ 가 benefit 쪽 증거로도 선다.

## 0. 전제 반증 — 먼저 쟀다

- ⒜ §D 결정은 **되돌릴 수 있는 상태다**: `origin/main` `fafcd213` 의 `wie-lgt/src/runtime/wipi_c.rs` 에
  `=> wie_wipi_c::api::graphics::` **27** 팔 · `upstream/main` 의 같은 파일 `=> graphics::` **27** ⇒ 전환 비용 27줄 그대로.
  ⇒ 측정은 기록용이 아니라 **결정 입력**이다.
- ★**모집단이 46 이 아니라 37 이다 — 그러나 같은 46 이다**: 형제 회차(`wie-2026-09-21-aot-java-render-rebaseline-adopt-p4`)가
  2026-09-24 07:12 에 **sha256 동일 사본 9건**을 `game_lab/_dup/broken/lgt/` 로 옮겼다(`_dup/README.md`).
  37 + 9 = 46 이고 옮긴 9건은 남은 37건 중 하나와 **바이트가 같다** ⇒ 37건을 재면 46건의 서로 다른 입력을 전부 잰 것이다.

## 1. 한 일

`0222` §2 와 같은 방법 — 격리 바이너리 둘:

| 바이너리 | 만든 법 |
|---|---|
| `wv_b` ⒝ | `origin/main` `fafcd213` 무수정 `cargo build -p wie_cli --bin wie_validate` |
| `wv_a` ⒜ | 위 + `sed 's/=> wie_wipi_c::api::graphics::/=> graphics::/g'` → `--numstat` **27 27 · 1파일** → 빌드 → `git checkout --` (`status --porcelain` 빈 출력) |

개악 대조쌍 `keydraw_lgt --inject --expect-last-frame`: ⒝ **PASS** ↔ ⒜ **FAIL**(`Undefined instruction` in `CletWrapperCard.paint`) —
`0222` §2-3 서명 그대로 ⇒ 두 바이너리는 다르다.

★**짝지은 측정**(`0230` 검수 권고): 타이틀마다 두 배선을 **연달아**, 순서를 타이틀마다 **번갈아** 돌렸다.
판정 = `scripts/smoke_gate.sh` `run_one` 의미론 그대로(`--timeout 15` · 50s kill 워치독 · `"result":"PASS"` 만 PASS).
RETRY=0 — 뒤집힌 타이틀만 재측(티켓 Contract 2).
시각 07:46:59 → 08:02:00(**15m01s**) · load1 **26.19 → 44.13** · idle(`top` 매 타이틀) 중앙 **31%**.

## 2. 결과 — 37건 × 2배선

| ⒝ \ ⒜ | PASS | FAIL |
|---|---|---|
| **PASS** | 8 | 0 |
| **FAIL** | **1** | 28 |

FAIL 사유 분포(⒝): 부팅 중 오류(`tick error`·`panic`) **19** · `no frame rendered` **7** · 빈/단색 프레임 **2** · JSON 없음 **1**
(호스트 `stack overflow` abort — 두 배선 동일). ⒜ 는 한 건이 `tick error` → `no frame` 으로 문면만 바뀌었다(판정 FAIL 불변).

★**뒤집힌 1건 = `당신은야구감독` — 재측에서 «잡음»으로 판정됐다**:

| 재측 | ⒜ | ⒝ |
|---|---|---|
| `--timeout 15` × 3(번갈아 · load1 57~65) | **FAIL 0/3** | **FAIL 0/3** |
| `--timeout 40` × 2(번갈아 · load1 30~47) | **PASS 2/2**(paints 49·175) | **PASS 2/2**(paints 7·171) |

⇒ 15초 데드라인에 걸친 타이틀이고(`0222` §2-1 의 데드라인 잡음 두 타이틀과 같은 계급), 시간을 주면 **⒝ 도 그린다.**
⇒ ★**판정: `broken/lgt` 에서 ⒜ 가 새로 여는 타이틀은 «0» 이다.**

## 3. §D 에 대한 뜻

cost 쪽(`0222`)도 benefit 쪽(이 회차)도 코퍼스 전체(working 54 + broken 46)에서 **0** 이다 ⇒ ⒜ 로 갈 측정된 이유가 없고
⒜ 는 `keydraw_lgt`(required check `contract` 의 Scenario F)를 깬다 ⇒ **권고 ⒝ 유지**. 결정 변경 제안 없음.

## 한계 — 숨기지 않는다

- ★**부팅 중 죽는 19건은 graphics SVC 에 닿기 «전»일 수 있다** — 그 경우 배선은 원리상 차이를 못 낸다.
  `0230` 이 main 의 LGT 부팅 실패 근인을 «#161 이 떨군 LGT 행들»로 규명했으므로, 그 행들이 복원되면 이 19건 일부가
  graphics 까지 닿고 **그때 benefit 을 다시 물을 수 있다.** 이 회차는 프로브(`0222` §2-2)를 다시 박지 않았다 —
  「지나는가」는 이 모집단에서 미측정이다.
- 판정 축은 boot+render 뿐(픽셀 대조 없음).
- 부하가 한 패스 안에서 26 → 44 로 올랐다. 짝지은 순서라 두 배선은 같은 부하 창을 본다.
- 부수: 이 회차의 8 PASS 는 형제 회차가 오늘 재생성한 `game_lab/census-map.tsv`(broken/lgt `PASS` 7 · `NO-REPORT` 1)와 같은 방향이다 —
  `broken/` 분류가 낡은 것은 그 리니지(`…-aot-java-render-rebaseline-*`)의 몫이라 여기서 제안하지 않았다.

## 검증 — 실행한 것

- 두 배선 짝지은 패스 37건(74회) · 뒤집힘 1건 재측 `--timeout 15` 3+3 · `--timeout 40` 2+2
- 개악 대조쌍 `keydraw_lgt` 두 배선
- 소스 복원 `git checkout --` → `git status --porcelain` 빈 출력

## 게임 파일명 유입 — 도구를 «실행해서» 적는다

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab`(코퍼스는 이 워크트리 밖이다):
★**BOUNDED 3회 / 2쌍** · PREFIX-EMBEDDED 0 · ★**SUFFIX-ATTACHED 0**.
새로 들어온 이름은 **1개**(2회) — 뒤집힌 타이틀(결과가 곧 그 타이틀이라 이름 없이 적으면 다음 회차가 다시 재야 한다).
나머지 1쌍은 §D 문서가 **이미 담고 있던** 이름이다(도구는 수정된 파일 전체를 읽는다). 모집단 37건의 목록은 repo 밖 회신에만 적었다.
게임 바이트 유입 0 · 경로 유입 0.
