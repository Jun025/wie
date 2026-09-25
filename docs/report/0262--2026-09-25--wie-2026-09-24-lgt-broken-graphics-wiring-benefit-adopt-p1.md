## [2026-09-25] 공용 graphics(⒝)가 LGT 이용안내 글자를 못 그리는 원인 — «빠진 SVC» 는 없다 · 컨텍스트 레코드 가설은 양방향 반증 (wie-2026-09-24-lgt-broken-graphics-wiring-benefit-adopt-p1)

**무엇을** — 측정 전용(**제품 코드 0줄**). `0236` §2-2 가 남긴 «⒜ PASS ↔ ⒝ 흰 화면» 1건의 원인을 1단계(SVC 판정)까지 쟀고,
가장 유력하던 수리(LGT 그래픽 컨텍스트 레코드만 ⒜ 형식으로)를 실제로 빌드해 돌려 **반증**했다.

**왜** — 제안 `2026-09-24-lgt-broken-graphics-wiring-benefit#p1` 의 전제는 「공용 쪽에서 빠지는 것(글꼴·텍스트 SVC 추정)을 메우면
⒜ 전면 교체 없이 연다」였다.

**사용자 영향** — 없음(코드 0줄). 그 타이틀은 여전히 흰 화면이다. 전제가 서지 않는다는 것이 이 회차의 산출물이다.

## 0. 측정 조건

- 기준 `origin/main` `7e97a7db`(측정 중 `main` 은 `2efea821`(#294 · LGT SVC 207 `GetContext` 복원)로 전진 — 이 타이틀은 어느 실행에서도
  미등재 SVC 오류를 내지 않았으므로 207 을 부르지 않는다 ⇒ 결론 무관).
- `wie_validate` **release** 빌드. debug 는 load 115~200 에서 40초·120초 모두 데이터베이스 적재 중(`paints 0`)이라 판정축에 못 닿았다.
- 바이너리 넷(`/tmp` 격리 · 소스는 매번 `git checkout --` 로 복원, 최종 `git status --porcelain` 빈 출력):
  ⒝ 무수정 · ⒜ `0236` 방법(27팔 `sed`) · E1 공용 `init_context` 에 `alpha: 255` · F 컨텍스트 어댑터(§3).
- 판정 `--timeout 60`. load1 은 실행마다 40~170 을 오갔다 — 아래 수치는 **짝지은 방향**만 인용하라.

## 1. 재현 — 전제 ⒜

| 바이너리 | 결과 | paints | distinct_colors | 비고 |
|---|---|---|---|---|
| ⒝ | FAIL «only blank/uniform» | 202 | **1** | 마지막 프레임 흰색 단색 |
| ⒜ | **PASS** | 132 | **2** | nondominant 2.8% |

⇒ `0236` §2-2 재현. 대조쌍 `keydraw_lgt --inject --expect-last-frame`: ⒝ **PASS**(paints 55 · last_frame_content true) ↔ ⒜ **FAIL**
(`Undefined instruction` in `CletWrapperCard.paint`) — `0222` §2-3 서명 그대로 ⇒ 두 바이너리는 다르다.

## 2. 프로브 — 전제 ⒝: 이 화면이 부르는 SVC

`RUST_LOG=wie_wipi_c=debug,wie_lgt=debug`, 타이머 콜백(= 매 프레임) 안의 SVC 를 셌다(데이터베이스 제외):

| SVC | ⒜ (132 프레임) | ⒝ (202 프레임) | 프레임당 |
|---|---|---|---|
| `MC_GRP_GET_FRAME_BUFFER_POINTER` | 1189 | 1824 | 9 · 9 |
| `MC_grpSetContext` | 532 | 812 | 4 · 4 |
| `MC_grpGetPixelFromRGB` | 532 | 812 | 4 · 4 |
| `MC_grpInitContext` | 266 | 406 | 2 · 2 |
| `MC_grpFlushLcd` | 132 | 202 | 1 · 1 |
| ★`MC_grpFillRect` | **0** | **203** | 0 · 1 |
| `MC_grpDrawString` | 0 | 0 | — |

★**판정: ⒝ 에서 «빠지는» SVC 는 없다.** ⒝ 는 ⒜ 의 **진부분집합이 아니라 상위집합**이다(`FillRect` 1개 더).
글자는 **어느 배선에서도 SVC 로 그려지지 않는다** — 게스트가 `GET_FRAME_BUFFER_POINTER` 로 받은 픽셀 버퍼에 **스스로** 쓴다
(프레임 순서: InitContext → SetContext(clip) → GetPixelFromRGB → SetContext(fg 흰색) → [⒝ 만 FillRect] → InitContext →
SetContext(clip) → SetContext(fg 빨강 `0xf800`) → 포인터 9회 → FlushLcd).
★그리고 ⒜ 에서는 흰 바탕도 SVC 가 아니다 — 게스트가 **같은 자기 경로로** 채운다. 즉 게스트는 «자기가 그릴지 · SVC 에 맡길지»를
**스스로 판단**하고, ⒝ 에서는 바탕을 SVC 에 맡기며 자기 경로의 글자는 보이지 않는다.
⇒ 차이는 SVC 목록이 아니라 **게스트가 메모리에서 직접 읽는 그래픽 레코드**에 있다(판단 근거는 게스트 코드 안 · 미관측).

## 3. 가장 유력한 레코드 — 컨텍스트 — 를 수리해 봤다: 양방향 반증

근거: LGT 게스트 컨텍스트는 56B(`LgtGraphicsContext` — clip i32×4 · `foreground`@+16 · `alpha`@+24 · `pixel_param`@+32 …),
공용은 48B(`WIPICGraphicsContext` — `mask`@0 · clip u16×4 · `fgpxl`@+12 …, `Default` = 전부 0). ⒝ 의 `InitContext` 뒤
LGT 오프셋으로 읽으면 `alpha`=0 · `pixel_param`=0 · `foreground`=공용 `bgpxl`(0) — «불투명 아님 ⇒ SVC 에 맡김 · 자기 글자는 투명»이
위 SVC 표와 정확히 맞는다. 그래서 두 단계로 쟀다.

| 시도 | 이 타이틀 | `keydraw_lgt` |
|---|---|---|
| E1 공용 `init_context` 가 `alpha: 255` | FAIL · paints 85 · distinct **1** | (미측정) |
| F 어댑터 — `InitContext`/`SetContext` 는 ⒜ 의 LGT 레코드, 컨텍스트를 읽는 공용 7종(`put_pixel`·`fill_rect`·`draw_rect`·`draw_line`·`draw_arc`·`fill_arc`·`draw_string`)은 번역 사본 | 짝지은 2쌍(F→⒝, ⒝→F): **F FAIL 41·187 / ⒝ FAIL 40·109 — 전건 distinct 1** | ★**FAIL** — `Undefined instruction` in `CletWrapperCard.paint`(⒜ 와 **같은 서명**) |

★**판정 — 컨텍스트 가설은 양쪽에서 진다**:
⑴ LGT 컨텍스트를 온전히 줘도 이 타이틀은 **안 열린다** ⇒ 컨텍스트는 «그것만으로» 원인이 아니다.
⑵ 그런데 그것만 바꿔도 **`keydraw_lgt` 가 ⒜ 와 똑같이 깨진다** ⇒ fixture 의 게스트 SDK 도 컨텍스트를 **공용 레이아웃으로 직접 읽는다.**
   ★부수 발견: `0222` §2-3 의 ⒜ `keydraw_lgt` 파손은 프레임버퍼 레코드만이 아니라 **컨텍스트 레코드도** 원인 축이다
   (`wipi_c.rs` 의 `mod graphics` 주석은 프레임버퍼 ABI 만 적는다).
⇒ 어댑터는 원복했다(제품 코드 0줄).

## 4. §D 에 대한 뜻 — «공용 쪽 최소 수리»는 이 축에 없다

남은 차이는 프레임버퍼·화면 레코드 계열(`GetScreenFramebuffer` 가 돌려주는 핸들의 기록 · `LgtFramebuffer` 16B ↔ `WIPICFramebuffer` 20B)이고,
★그것은 `keydraw_lgt` 의 SDK 가 `WIPICFramebuffer` +16 을 직접 읽는 바로 그 레코드다(`wipi_c.rs` 주석 · `0222`).
컨텍스트에서 방금 잰 것과 같은 형태 — **한 레코드를 두 게스트가 서로 다른 레이아웃으로 직접 읽는다** — 라면,
«공용 레코드 하나를 고쳐 둘 다 연다»는 수리는 **존재하지 않는다**. 선택지는 «게스트마다 레코드 ABI 를 고르는 것»이고, 그것은
최소 수리가 아니라 설계 결정이다(후속 제안). ★단 프레임버퍼 레코드가 원인이라는 것은 **추론이다 — 미측정**.

## 인접 — 배틀몬스터 «paints 3» 은 같은 계급이 아니다

⒜ · ⒝ 모두 PASS · paints **3** · distinct **2** · nondominant 2.8%(동일) · `stop: max-ticks`. 로그에 WIPI-C 그래픽 SVC **0건** —
Java(Jlet) 타이틀이라 lcdui 경로로 그린다 ⇒ 이 배선과 무관하다. 「paints 3」 은 틱 예산 축(별 리니지 소관)이다.

## 한계 — 숨기지 않는다

- ★«게스트가 무엇을 읽고 자기 경로/SVC 를 고르는가»는 **관측하지 않았다** — SVC 표와 두 반증으로 좁혔을 뿐이다.
  결정 증거는 그 분기의 디스어셈블리 또는 핸들·컨텍스트 주소 읽기 감시다(후속 제안).
- 표본은 이 타이틀 1건 · 짝지은 2쌍. release 빌드 1회 ≈ 1시간(load 40~200)이라 반복을 늘리지 않았다.
- working/lgt 54건 회귀는 **돌리지 않았다** — 착지할 수리가 없다(Contract 2 는 수리 조건부).

## 검증 — 실행한 것

- release 바이너리 넷 빌드 · 이 타이틀 ⒜ 1 · ⒝ 3 · E1 1 · F 2(짝지은) · `keydraw_lgt` ⒜/⒝/F · 배틀몬스터 ⒜/⒝ · SVC 계수 로그 ⒜/⒝
- 소스 복원: 매 실험 뒤 `git checkout --` · 신규 파일 삭제 → `git status --porcelain` 빈 출력

## 게임 파일명 유입 — 도구를 «실행해서» 적는다

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab`: ★**BOUNDED 5회 / 2쌍** · PREFIX-EMBEDDED 0 ·
★**SUFFIX-ATTACHED 0**. 들어온 이름은 **1개**(배틀몬스터 — 이 문서 1쌍 · worklog 1쌍)이고 티켓 자신이 그 이름으로 «같은 계급인지»를
물었기 때문에 이름 없이 적으면 답이 되지 않는다. `0236` §2-2 의 뒤집힘 타이틀은 이름 대신 «그 타이틀»로만 적었다.
게임 바이트 유입 0 · 경로 유입 0.
