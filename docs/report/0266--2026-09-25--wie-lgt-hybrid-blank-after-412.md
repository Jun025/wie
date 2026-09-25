## [2026-09-25] 하이브리드 단색 벽 = `MC_GRP_GET_FRAME_BUFFER_BPP` 가 인자를 핸들로 읽는다 — LGT 접근자로 한 줄 옮겨 PASS (wie-lgt-hybrid-blank-after-412)

**무엇을** — `wie-lgt` 의 WIPIC `GetFramebufferBpp`(0x36) 한 줄을 공용 `wie_wipi_c::api::graphics` 에서 트리 안의 LGT 접근자로 옮겼다.
SVC 스텁을 거치는 시험 1개를 더했다.

**왜** — 채택 제안 `2026-09-25-lgt-three-dropped-rows-restored#p0`: 412 행 복원 뒤에도 `하이브리드` 는 «only blank/uniform frames» 였다.
원인 축을 먼저 재라는 제안이었다.

**사용자 영향** — `working/lgt` 에서 흰 단색이던 4 타이틀(`하이브리드`·`놈ZERO`·`제노니아1`·`제노니아2`)이 이용안내 화면(붉은/검은 글자)을 그린다. 짝지은 게이트에서 회귀는 0 이다(§3).

## 0. 대전제

- `origin/main` `1d562b34` 재측: FAIL «only blank/uniform frames» · paints 503 · `distinct_colors` **1** · `stop: max-ticks`(60초 · release).
- 같은 처방: `ledger-grep` tasks·queue·running 에서 이 티켓만 걸린다 · `gh pr list` 에 같은 SVC 를 고치는 PR 없음.
- 코퍼스의 working·broken 두 사본은 sha256 이 같다. 2026-06-30 `reports/` 기록도 이미 `content false`·1색이었다 ⇒ 원래 있던 벽이다(412 와 무관).

## 1. SVC 계수 — 원인 축 판정

`RUST_LOG=wie_wipi_c=debug,wie_lgt=debug` · release · 60초 · 503 프레임:

| SVC | 호출 | 프레임당 |
|---|---|---|
| `MC_GRP_GET_FRAME_BUFFER_POINTER` | 46,291 | ≈92 |
| `MC_GRP_GET_FRAME_BUFFER_WIDTH` | 45,792 | ≈91 |
| ★`MC_GRP_GET_FRAME_BUFFER_BPP` | 45,286 | ≈90 |
| `MC_grpFlushLcd` | 503 | 1 |
| 그리기 SVC(`FillRect`·`DrawString`·`DrawImage` …) | **0** | — |

- 게스트는 픽셀을 **스스로** 쓴다(형제 #298 이 다른 타이틀에서 본 형태와 같다). 첫 대기 지점은 없다 — 입력 대기도 리소스 결손도 아니다.
  리소스 16건은 전부 적재됐고, 미등재 SVC 오류는 0건이다. 타이머 루프가 매 프레임 그리고 flush 한다.
- ★**BPP 의 인자가 핸들이 아니다.** 핸들은 `0x4904d684` 인데 BPP 인자는 `0x4904dbe5` 40,252 · `0x4046cf5c` 1,509 · `0x4046cf6a` 1,006 … 이다.
  홀수 주소이거나 적재된 리소스 버퍼 안의 주소다. 공용 접근자는 그 주소를 `WIPICFramebuffer` 레코드로 읽어 +12 의 쓰레기를 bpp 로 돌려준다.
- 트리 안의 LGT 접근자(`wie-lgt/src/runtime/wipi_c/graphics.rs` `get_framebuffer_bpp`)는 이미 이렇게 적는다:
  「The native accessor ignores its argument … The application consequently passes a non-framebuffer value here」. 그 함수는 인자를 무시하고 16 을 돌려준다.
  upstream 은 이 줄을 LGT 쪽으로 배선한다(⒜ 27줄 중 1줄).

## 2. 변이 — 그 한 줄만 다른 두 바이너리

⒝ = `origin/main` · C = BPP 한 줄만 LGT 접근자. 둘 다 release · `--timeout 60` · 순서를 교대했다.

| 순서 | 바이너리 | 결과 | paints | distinct | nondominant | load1 |
|---|---|---|---|---|---|---|
| 1 | C | **PASS** | 182 | 2 | 3.7% | 98.7 |
| 2 | ⒝ | FAIL 단색 | 368 | 1 | 0 | 80.5 |
| 3 | ⒝ | FAIL 단색 | 424 | 1 | 0 | 59.6 |
| 4 | C | **PASS** | 146 | 2 | 3.7% | 41.7 |

`--shotdir` 45초 프레임: C 는 흰 바탕에 「◀ 이용안내 ▶ … 아무키나 누르세요!!」(붉은/검은 글자)이고, ⒝ 는 흰 단색이다.
⇒ **원인은 이 한 줄이다.** 크기가 S 라 같은 회차에 고쳤다.

**fixture 회귀 없음**: `keydraw_lgt --inject --expect-last-frame --max-ticks 5000000000` C·⒝ 모두 PASS · paints 55 · 27/27 키 · rc=0.
`helloworld_lgt` 는 둘 다 PASS 다. ★기본 `--max-ticks` 로는 release 가 첫 키 전에 백스톱을 태워 둘 다 `UNMEASURED` rc=2 였다(AGENTS.md 의 그 판정 · FAIL 아님).
⇒ BPP 는 fixture SDK 가 직접 읽는 레코드를 건드리지 않는다. `0222`·#298 의 ⒜ 파손(프레임버퍼·컨텍스트 레코드)과 다른 축이다.

## 3. `working/lgt` 54 짝지은 게이트

smoke_gate 판정(boot+render · `--timeout 15` · 최대 3회 · 워치독 50s)으로 쟀다. 타이틀마다 ⒝·C 를 번갈아 돌렸고 순서도 타이틀마다 교대했다.
두 러너가 1~27·28~54 를 나눠 돌렸다. load1 은 **12~99** 였다.

| | PASS | FAIL |
|---|---|---|
| ⒝ `origin/main` | 49 | 5 |
| C | **52** | 2 |

어긋난 5건은 `--timeout 40` 으로 순서를 바꿔 가며 2쌍씩 다시 쟀다(load1 28~63):

| 타이틀 | 15s | 40s 재측(⒝ · C) | 판정 |
|---|---|---|---|
| `하이브리드` | ⒝ FAIL · C PASS | §2 의 2쌍 | **FAIL → PASS** |
| `놈ZERO` | ⒝ FAIL · C PASS | ⒝ FAIL 1색 ×2 · C PASS 3색 ×2 | **FAIL → PASS** |
| `제노니아1` | ⒝ FAIL · C PASS | ⒝ FAIL 1색 ×2 · C PASS 2색 ×2 | **FAIL → PASS** |
| `제노니아2` | ⒝ FAIL · C PASS | 1쌍 양쪽 no frame(판정 불가) · 1쌍 ⒝ FAIL 1색 ↔ C PASS 2색 | **FAIL → PASS**(판정 가능 1쌍 · 15s 1쌍) |
| `(LGT)레이카르나` | ⒝ PASS · C FAIL | ⒝ PASS ×2 · C PASS ×2 | 15초 데드라인 잡음 ⇒ **회귀 0** |

C 의 마지막 프레임은 셋 모두 붉은/검은 글자의 이용안내 화면이다(`놈ZERO` 「<이용 안내> … 아무 키나 누르세요!!」 · 제노니아 둘 「◀ 이용안내 ▶ …」). ⒝ 는 셋 모두 흰 단색이다.
⇒ **이 한 줄이 `working/lgt` 에서 4건을 연다. 회귀는 0 이다.**
- 남은 C FAIL 2건: `레이카르나`(위 · 잡음)와 `하이브리드2`(양쪽 FAIL · 이 줄과 무관)다.
- ★`0230` §E 는 `놈ZERO`·`제노니아1/2`·`하이브리드2` 를 «교체 전 트리에서도 hang/blank»로 분류했다. 그중 셋이 이 축이었다.
- ★절대 수 49/54 는 이 부하·이 15초 판정의 값이다. `0258` 의 6/54(load 130~215)와 비교하지 마라.

## 4. 형제와의 관계 — #298 과 «합류», #296 과는 무관

- ★**#298 의 타이틀(`0236` §2-2 의 뒤집힌 1건)도 같은 첫 층이다.** 그 타이틀도 화면을 만들 때 BPP 를 **한 번** 핸들이 아닌 값(`0x403c7160`)으로 부른다.
  #298 은 타이머 콜백 안의 SVC 만 셌기 때문에 이 호출이 표에 없었다. C 에서 그 타이틀은 validator 기준 FAIL(1색) → **PASS(2색)** 로 바뀐다.
  ★**그러나 사람 눈에는 여전히 거의 흰 화면이다.** 글자 픽셀이 `#fffbff`(흰 바탕 `#ffffff` 위 2,125 px)이다. BPP 가 게스트를 «자기 글리프 경로»로 돌려놓지만,
  그 경로의 색은 게스트가 컨텍스트 레코드에서 직접 읽는다. #298 §3 이 잰 LGT 56B ↔ 공용 48B 불일치가 그 둘째 층이다.
  ⇒ #298 의 「빠진 SVC 는 없다」는 맞다. 다만 「컨텍스트만 고치면 안 열린다」의 이유가 이 BPP 층이었다. 그 타이틀은 **BPP + 컨텍스트 두 층**이 필요하다(후속 제안).
- #296(LGT hang 5종 = 미구현 칸의 되던지기 루프): 이 타이틀은 hang 이 아니라 매 프레임 flush 한다 ⇒ 다른 축이다.

## 검증

- `cargo fmt --check` rc=0 · `cargo clippy --all -D warnings` rc=0 · wasm clippy rc=0 · `cargo +beta clippy --all -D warnings` rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` rc=0 · 46 스위트 · **445 passed · 0 failed**.
- 개악(변이) red: `wipic_framebuffer_bpp_ignores_its_argument` — 배선을 공용으로 되돌리면 `left: 0 · right: 16` FAILED · 복원하면 ok.
- `allow(dead_code)` 를 떼고 센 경고 수는 45 → **42** 다(`get_framebuffer_bpp`·`state`·`FRAMEBUFFER_DEPTH` 가 살아남). 모듈 주석과 `docs/upstream-realign-p3-slices.md` §D(공용 배선 28 → 27줄)를 갱신했다.

## 한계

- 공용 접근자가 실제로 돌려준 bpp 값은 로그에 없다 — 인자가 핸들이 아니라는 것은 관측이고, «쓰레기를 돌려줬다»는 시험(0 이 나온다)과 변이가 근거다.
- 판정 축은 boot+render 다. 하이브리드의 이용안내 뒤 화면(키 입력 뒤)은 재지 않았다.

## 게임 파일명 유입 — 도구를 «실행해서» 적는다

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab` 의 대상은 이 브랜치가 바꾼 5파일이다. 수는 아래 표식이 최종이다.
BOUNDED 쌍은 이 회차가 잰 타이틀 이름(티켓 대상 · 게이트에서 뒤집힌 5건 · #298 의 타이틀)이고, 나머지는 `wipi_c.rs` 에 원래 있던 주석 속 이름이다.
SUFFIX-ATTACHED 는 손으로 갈랐다. `하이브리드2` 는 코퍼스의 **별개 실제 타이틀**이고(§3 게이트 표본), 나머지는 stem 에 조사(는·의)가 붙은 **진짜 언급**이다 ⇒ 판단이 더 필요한 것 0.
유입된 것은 이름뿐이다. 게임 바이트 유입 0 · 경로 유입 0.


<!-- corpus-name-inflow v1 subjects=5 tree=3ea99070c3ace41e B=36/18 P=0/0 S=6/3 -->
