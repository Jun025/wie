## [2026-09-23] 코퍼스가 조각 D 의 미결 결정을 쟀다 — 실제 LGT 타이틀 54건에 ⒜/⒝ 차이가 «없다» (wie-p3-next-slice-after-slice-a-abi-finding)

**무엇을** — 측정 전용(**제품 코드 0줄** · 게이트·워크플로 무접촉). `docs/upstream-realign-p3-slices.md` §D 와
`docs/report/0206` 이 「코퍼스가 생기면 **LGT 52건을 먼저 돌려라** — 그 답이 이 권고를 뒤집을 수 있고,
뒤집어야 한다」로 남긴 재개 조건 ⑴ 을 **그대로 집행**했다. 코퍼스는 이제 이 머신에 있다.

**왜** — 그 축은 두 회차(0114·0206)가 「코퍼스 없이 어느 쪽도 «옳다»를 증명 못 한다」로 닫아 둔 자리이고,
막고 있던 것은 판단이 아니라 **부재하던 입력** 하나였다.

**사용자 영향** — 없음(코드 0줄). 파이프라인 영향: 조각 D 의 재개 조건 ⑴ 이 «해소»된다 — 그리고
그 답은 **결정을 뒤집지 않는다**. 부수로 **로컬 회귀 게이트가 «0건을 검사하고 OK 를 찍고 있었다»**는 사실이 나왔다.

---

## 0. 고른 조각과 «왜» (Contract 1)

★**고른 것 = 조각 D 의 재개 조건 ⑴ — 「292 코퍼스(LGT 52건)」를 실제로 돌린다.**
★**A~E 중에 «미착수 조각»은 없다**(아래 실측). 문서가 남긴 «열린 축»은 하나뿐이고 그것이 이것이다.

| 조각 | 문서가 적은 상태 | ★**2026-09-23 실측** |
|---|---|---|
| A | 돌았다(`0115`) | 일치 |
| B | 끝났다(`0158`) | 일치 |
| C | 부분 착지 | 일치 |
| D | ★**「시도했고 멈췄다 · 머지 0」** | ★**착지했다** — PR **#161** MERGED `2026-09-16T08:35:30Z`(머지 `37734e74`) · `git merge-base origin/main upstream/main` = **`44fbf265`**(≠ `fa641a8a`) · behind **1,140 → 15** |
| E | ★**「미착수(선행 D 가 멈춰 있다)」** | ★**돌았다 · «미착지»** — PR **#162 OPEN**(개설 `2026-09-16T03:27:17Z`) · 회신 `reports/wie-p3-slice-e-verify-web-contract-on-new-base-fix.done.md` |

⇒ ★**선행 관계 확인(Contract 1⒝)**: 내가 집은 축의 선행은 **A(게이트)와 D** 이고 **둘 다 이미 돌았다.**
조각 E 는 선행이 아니라 **형제**이고 지금 검수 대기 중이라 이 회차가 건드리지 않는다.

★★**문서 ↔ `STATE.md` 어긋남(Contract 1⒞) — «있다». 그리고 어긋난 쪽은 «둘 다»다.**
`STATE.md` 뿐 아니라 정본 `docs/upstream-realign-p3-slices.md` 의 **§E 절에도 착지 배너가 없다**
(§D 는 `-fix3` 배너까지 갖고 있다). ⇒ ★**두 문서 다 갱신했다** — 정본을 따르되, 정본도 낡은 칸은 «측정으로» 고쳤다.
★**그리고 그 낡음 셋이 바로 이 레인이 18시간 굶은 이유다**: 「D 가 멈췄다 ⇒ E 미착수 ⇒ 코퍼스 없음」을
그대로 읽으면 **집을 수 있는 조각이 «하나도 없다».**

★**전제 재측(Contract 2⒞ — 「불가를 옛 판정 인용으로 쓰지 마라」)**: `STATE.md` 가 「코퍼스 부재
(`find ~ -maxdepth 4 -name game_lab` **0건**)」로 적은 그 술어를 **그대로 다시 쳤다**:

```
$ find ~ -maxdepth 4 -name game_lab
/Users/jun0m1/work/otterpebble/wie/game_lab
$ for d in working/*/; do echo "$d $(ls "$d"|wc -l)"; done    # (game_lab 안)
ktf 190 · lgt 54 · skt 50 · j2me 0
```
⇒ ★**부재는 «낡은 사실»이다.** 이 회차의 전제는 그 재측 위에 선다.

## 1. 크기 판정 (Contract 2)

★**이 `timeout_min`(150) 안에 끝난다 — 끝냈다.** 실측 소요: 두 패스 **13m00s + 12m33s** · 프로브 2회
**약 12분** · 재측·빌드 **약 10분**. ⒜ 전환은 `sed` 치환 **27줄 / 1파일**이라 빌드가 **9~34초**다.
★**쪼개지 않았다** — 대신 아래 §6 에 **후속 4건의 분할안**을 남긴다(코퍼스 축이 열리며 «새로» 생긴 일들이다).

## 2. 한 일 — 전/후

**제품 코드 0줄.** 측정은 전부 **격리된 바이너리 4개**로 했고 소스는 매번 `git checkout --` 로 되돌려
`git status --porcelain` **빈 출력**을 확인했다.

| 바이너리 | 무엇 | 만든 법 |
|---|---|---|
| `wv_b` | ★**⒝ = `origin/main` 형상**(27 SVC → 공용 `wie_wipi_c::api::graphics`) | 무수정 빌드 |
| `wv_a` | ★**⒜ = upstream LGT 전용**(27 SVC → `graphics::`) | `sed 's/=> wie_wipi_c::api::graphics::/=> graphics::/g'` → `--numstat` **27 27 · 1파일** |
| `wv_aprobe` | ⒜ + `graphics::get_screen_framebuffer` 첫 줄 `panic!("PROBEHIT …")` | 위 + 1줄 |
| `wv_aprobe2` | ⒜ + `graphics::get_framebuffer_pointer` 첫 줄 `panic!("PROBEHIT …")` | 위 + 1줄 |

★**⒜ 치환이 upstream 형상과 «같다»를 확인하고 썼다**: `git show upstream/main:wie-lgt/src/runtime/wipi_c.rs`
의 `=> graphics::` **27** ↔ 치환 후 우리 파일 **27**(그리고 `shared_graphics::` 8 팔은 **양쪽 동일**).

### 2-1. ★본 측정 — `game_lab/working/lgt` **54건**을 두 배선에 같은 명령으로 먹였다

명령(양쪽 동일): `WORKING_DIR=<corpus>/working PLATFORM_FILTER=lgt scripts/smoke_gate.sh`
(판정 = **boot+render** · `TIMEOUT=15` · `RETRY=2` · 워치독 `KILL=50`).

| 배선 | 시각 | 결과 |
|---|---|---|
| ⒝ | 09:55:34 → 10:08:33 (**13m00s**) | ★**40 PASS / 14 FAIL** |
| ⒜ | 10:08:34 → 10:21:07 (**12m33s**) | ★**40 PASS / 14 FAIL** |

★★**타이틀 단위로 붙이면 54건 중 «52건이 동일»하고, 다른 2건은 «같은 파일»이다.**

```
PASS -> FAIL   lgt/(LGT)나는마왕이다2.zip
FAIL -> PASS   lgt/나는마왕이다2.zip
```

★**그 둘은 md5 가 같다**(`e17b76dcb581949b691d536fc4b4efe5` · 2,905,669B **양쪽 동일**) ⇒
★**같은 바이트가 «같은 패스 안에서» 한 번은 PASS, 한 번은 FAIL 했다.** 배선 효과라면 두 사본이
**같은 방향**으로 움직여야 한다 — **반대로 움직였다.**
재측(같은 파일 · 각 3회): ⒝ **PASS/PASS/PASS**(paints 6·14·14) · ⒜ **PASS/PASS/PASS**(16·28·34).
그 뒤 10:3x 에 같은 파일이 ⒝ 에서 **2/2 FAIL**(`no frame rendered`) — ★**렌더 데드라인에 걸친 타이틀**이다.
⇒ ★**그 2건은 «배선 차이»가 아니라 «잡음»이고, 코퍼스가 그 잡음의 계기를 «스스로» 제공했다**(중복 사본).

⇒ ★★**판정: `working/lgt` 54건에서 ⒜ 와 ⒝ 의 boot+render 판정 차이는 «0» 이다.**

### 2-2. ★★프로브 — 「그 54건이 문제의 함수를 실제로 지나는가」를 «묻고 쟀다»

★**이것이 없으면 §2-1 은 공허하다**(「green 을 보존의 증거로 읽지 마라」 · verdict §9-5).
⒜ 형상에서 **레코드 ABI 가 갈리는 바로 그 함수**에 `panic!` 을 넣고, ⒜ 에서 PASS 한 40건에 먹였다.

| 프로브 | 대상 | 결과 |
|---|---|---|
| ①`graphics::get_screen_framebuffer` | ⒜-PASS **40건** | ★**HIT 39 · NOHIT 1** |
| ②`graphics::get_framebuffer_pointer` | ①의 HIT **39건** | ★**HIT 27 · NOHIT 12**(그중 **PASS 4 · FAIL 8**) |

★**NOHIT 1 은 «다른 경로»가 아니라 «못 쟀다»**: `lgt/아니마.zip` 은 그 자리에서 재측하면
**⒜ 원본과 프로브판 둘 다 FAIL**(`no frame rendered` · paints 0) — §2-1 의 그 데드라인 잡음이다.
★**②의 NOHIT 12 도 «둘로 갈라 적는다»**: **4건은 PASS** ⇒ 픽셀 포인터를 **한 번도 요구하지 않고 그렸다**(드로잉 SVC 만 썼다) ·
**8건은 FAIL** ⇒ ★**못 쟀다**(도달 전에 죽었다). ★그 8건을 「안 부른다」로 세지 마라.

⇒ ★★**39/40 이 upstream 의 `LgtFramebuffer`(16B) 핸들을 «실제로 받아 가고» 그대로 렌더한다.**

### 2-3. ★개악 대조쌍 (Contract 4⒜) — 출력 원문

```
=== wiring b (keydraw_lgt --inject --expect-last-frame):
{"file":"test_data/keydraw_lgt.zip","platform":"lgt","result":"PASS","reason":"booted + rendered + survived input sequence (visual correctness NOT checked)","stop":"deadline","input_steps":27,"input_steps_total":27,"ticks":31233017,"paints":55,"content":true,"last_frame_content":true,"distinct_color
rc=0
=== wiring a (keydraw_lgt --inject --expect-last-frame):
{"file":"test_data/keydraw_lgt.zip","platform":"lgt","result":"FAIL","reason":"tick error during 'boot': Fatal error:
net.wie.WieError: Fatal error: Undefined instruction
	at net/wie/CletWrapperCard.paint(Lorg/kwis/msp/lcdui/Graphics;)V
	at net/wie/CardCanvas.paint(Ljavax/microedition/lcdui/Graphic
rc=1
```
⇒ ★**두 바이너리가 «다른 것»임이 확정된다** — 조각 A·`0130` 이 기록한 그 서명 그대로다(PASS 55 ↔ FAIL 0).
★**그러므로 §2-1 의 «차이 0» 은 「같은 바이너리를 두 번 돌렸다」로 설명되지 않는다.**

## 3. ★★그래서 무엇이 답해졌나 — 문서가 «물으라»고 적어 둔 질문에 그대로 답한다

`docs/upstream-realign-p3-slices.md` §A 와 `STATE.md` 가 **리터럴로** 남긴 질문:

> 「실제 clet 이 `GetScreenFrameBuffer` 반환값을 `WIPICFramebuffer` 로 «직접» 읽나, 아니면
> `ptr_graphics → view.ptr_backing` 을 걷나」

★**측정된 답**: `working/lgt` 의 clet 들은 ★**그 레코드의 «모양에 의존하지 않는다».** 39/39(측정 가능분)이
그 SVC 를 지나고 27건은 픽셀 포인터마저 **접근자 SVC 로** 받아 간다 — 그리고 **16B(⒜)든 20B(⒝)든
같은 판정**이 나온다. ★**레이아웃을 게스트 코드에서 직접 역참조하는 소비자는 이 코퍼스에 «없다».**

★**깨지는 것은 «우리가 만든 픽스처» 하나뿐이고, 그것이 깨지는 이유는 이제 이름이 있다**:
`keydraw_{ktf,lgt}` 는 `dlunch/wipi` **SDK 로 빌드한 게스트**이고 그 SDK 의 고수준 `Framebuffer` 가
공용 배치를 **게스트 Rust 에서 직접 읽는다**(조각 D 회차가 `lgt` 분기 **0건**으로 이미 쟀다).
⇒ ★**「⒜ 는 실제 게임을 깨뜨린다」는 «이 코퍼스에서는 참이 아니다» — 깨지는 것은 SDK 게스트다.**

★★**그러나 이것이 ⒜ 로 가라는 뜻은 «아니다» — 오히려 결정을 더 싸게 만든다.**

| 축 | ⒜ upstream LGT 전용 | ⒝ 공용(현행) |
|---|---|---|
| **실제 LGT 타이틀 54건** | 40 PASS | 40 PASS ⇒ ★**비김(차이 0)** |
| `keydraw_lgt` 픽스처 + Scenario F | ★**깨진다**(required check `contract`) | 온전 |
| 되돌리는 값 | 27줄 | 27줄 ⇒ 비김 |
| upstream 1,095줄 | 배선됨 | 트리에 **주차**(`allow(dead_code)` 1줄 — 이미 지불됨) |

⇒ ★★**`0206` 의 권고 ⒝ 는 «유지»되고, 근거가 «가역성»에서 «가역성 + 코퍼스 증거»로 올라간다.**
★**바뀐 것은 «대가의 크기»다**: ⒜ 가 사겠다던 benefit 은 이 코퍼스에서 **측정되지 않고**(PASS 수 동일),
⒝ 가 치른다던 비용(「1,095줄을 버린다」)도 **측정되지 않는다**(주차이고, 실제 타이틀은 그 구현을 필요로 하지 않는다).
⇒ ★**재개 조건 ⑴ 은 «해소»다 — 그리고 그 해소는 결정을 뒤집지 «않는다».**

★**이 문단이 주장하지 «않는» 것**: ⑴판정 축은 **boot+render 뿐**이다 — 시각적 정확성·게임 진행 심도는
재지 않았다(두 배선의 화면이 **같다**고 말하지 않았다). ⑵`broken/lgt` **46건은 안 돌렸다**(§6 ⑶).
⑶`ktf`·`skt` 는 이 질문의 대상이 아니다(LGT 전용 배선).

## 4. ★★부수 발견 — 로컬 회귀 게이트가 «0건을 검사하고 OK 를 찍는다»

두 패스가 **똑같이** 이렇게 끝났다(출력 원문):

```
== ran 54 titles: 40 PASS / 14 FAIL ==

== smoke_gate: checked 0 baseline titles, 292 absent, 0 regressions ==
OK: no regressions vs baseline.
```

★**「checked **0**」과 「OK: no regressions」가 같은 화면에 있다.** 근인은 **유니코드 정규화**다:
커밋된 `scripts/smoke_gate_baseline.tsv` 의 한글은 **NFC**(`리` = `EA B0 80` 류 3바이트 완성형)이고
APFS 위 코퍼스 파일명은 **NFD**(`ᄅ`+`ᅵ` 분해형)다 — 게이트의 대조는 `awk '$1==t'` **바이트 일치**라
★**한 건도 붙지 않는다.** 실측: 원문 대조 **겹침 0** ↔ NFC 정규화 후 **겹침 52/52**.

★★**그 대조를 복원하면 «조용한 빨강»이 드러난다** — 정규화 후 재계산:

| 축 | 값 |
|---|---|
| baseline `lgt/` PASS 행 | **52** |
| 코퍼스에 있는데 baseline 에 없는 것 | **2**(`놈ZERO.zip`·`하이브리드.zip`) |
| ★**baseline PASS 인데 지금 FAIL** | ★**12**(⒝ 기준 · ⒜ 도 **같은 12**, 차이는 §2-1 의 잡음 1건뿐) |

★**그 12 + 2 는 «잡음이 아니다» — 각각 2회 재측해 «2/2 FAIL»이고 사유가 하드 에러다**(paints 전건 0):
`panic during 'boot': attempt to subtract with overflow` **2건**(같은 게임의 사본 2개) ·
`tick error during 'boot': net.wie.WieError: …` **11건** · `no frame rendered (hang/black screen)` **1건**
(마지막 1건 = §2-1 의 데드라인 잡음 타이틀 — ★**이 1건만 «못 쟀다»로 세라**).
★**목록**(이미 `smoke_gate_baseline.tsv` 에 식별자로 커밋돼 있는 이름들이다 — 새 유입이 아니다):
`(LGT) 메탈슬러그 서바이벌`·`메탈슬러그 서바이벌`·`(LGT)리듬페스티발`·`리듬페스티발`·`(LGT)알바타이쿤2`·
`게임빌2010슈퍼사커`·`데몬헌터`·`바이오크로니클`·`제노니아1`·`제노니아2`·`하이브리드2`·`하이브리드`·`놈ZERO`·`나는마왕이다2`.

★★**그중 하나는 이 repo 의 문서가 «PASS 로 기록해 둔» 타이틀이다**: `docs/lgt_abi.md` cp47 이
「**Current headless 놈ZERO = PASS**: boots, 153 paints」라고 적는다 — 지금은 **FAIL**(2/2 · paints 0).
⇒ ★**문서와 제품이 어긋났고, 그것을 말해 줄 게이트가 «자기가 0건을 재고 있다»는 것을 몰랐다.**

★**여기서 «고치지 않았다» — 이유를 적는다.** ⑴티켓이 **한 조각**을 요구했고(Contract 2) 이것은 그 조각이 아니다
⑵대조를 복원하면 그 로컬 게이트는 **즉시 빨개진다**(12건) — 그 빨강을 「베이스라인을 갱신한다」로 닫을지
「12건을 고친다」로 닫을지는 **이 회차가 정할 일이 아니다**(그리고 §6⑵가 그 결정을 지고 간다)
⑶`smoke_gate.sh` 는 **로컬 전용**이라 CI 를 빨갛게 만들지 않는다 ⇒ **지금 당장의 파이프라인 위험은 0** 이다.
★**대신 이 절이 그 «침묵»의 정본 기록이다.**

## 5. `STATE.md` `## 다음` 전/후 (Contract 6)

★**이 절이 이 회차의 «두 번째 산출»이다** — 이 레인이 굶은 이유가 그 절의 유지이기 때문이다(그 파일 머리의 경고).
★**diff 를 열고 썼다**(같은 파일이 경고하는 「PR 제목만 보고 쓰지 마라」) — 위 §0 표의 5축이 그 실측이다.

| 항목 | 전 | 후 |
|---|---|---|
| 조각 D | 「시도했고 «멈췄다» · 머지 0」 | ★**「착지했다」** + 머지 sha·시각·`merge-base`·behind 15 |
| 조각 E | 「**미착수**(선행 D 가 멈춰 있다)」 | ★**「돌았다 · PR #162 «미착지»」** |
| 코퍼스 | 「부재(`find` 0건) ⇒ human-step 후보」 | ★**「있다」** + 갈래별 수 |
| 재개 조건 ⑴ | 「코퍼스가 생기면 LGT 52건을 먼저 돌려라」 | ★**「돌렸다 — 차이 0 · ⒝ 유지」** + 이 리포트 연번 |
| 다음 후보 | (없음 — 그래서 굶었다) | ★**§6 의 4건**을 이름으로 적었다 |

## 6. ★분할안 — 이 회차가 «하지 않은» 일과 그 이유 (Contract 2⒝)

| # | 무엇 | 왜 별 회차인가 | size |
|---|---|---|---|
| ⑴ | ★**`smoke_gate.sh` 의 NFC/NFD 대조 복원** — 게이트가 0건을 재고 OK 를 찍는다(§4) | 고치면 **즉시 12건 빨강**이고, 그 빨강의 처분(베이스라인 갱신 ↔ 12건 수리)이 **다른 결정**이다 | **S** |
| ⑵ | ★**LGT 하드 실패 12+2건 분류** — `panic: subtract with overflow` · `WieError` · `놈ZERO` 문서 대비 회귀 | 근인이 배선이 **아니고**(⒜·⒝ 동일) 현행 AOT-java 계열과 같은 계급이다 ⇒ 그 리니지의 일 | **M** |
| ⑶ | ★**`broken/lgt` 46건을 ⒜ 로** — 「upstream 구현이 «우리가 못 여는 것»을 여나」 | 결정의 **반대쪽 절반**이다. 이 회차는 「⒜ 가 깨뜨리나」만 쟀다 | **M** |
| ⑷ | `ktf` 190 · `skt` 50 을 baseline 과 대조 | ⑴이 착지해야 의미가 있다(지금은 그 대조가 **무효**다) | **M** |

★**⑶ 을 이 회차에 붙이지 않은 이유**: `broken/` 은 전건 FAIL 이라 `RETRY=2` 가 매 타이틀 3회로 불어
**패스당 약 35~40분**이고, 두 배선이면 이 `timeout_min` 을 넘긴다. ★**크기를 «먼저» 판정해서 뺐다.**

## 검증 — 무엇을 «실행»했나

- 두 배선 풀패스 **각 54건**(같은 명령·같은 코퍼스·연속 실행) · 시각 기록 · per-title 대조 `join`
- ★중복 사본 md5 대조(잡음 계기) · 잡음 타이틀 **각 배선 3회 재측**
- ★프로브 2종(`panic!` 1줄) × 40/39건 — 「지나는가」를 **추론이 아니라 실행**으로
- ★개악 대조쌍 `keydraw_lgt`(§2-3 출력 원문 · rc 포함)
- baseline 대조를 **NFC 정규화로 재계산**(`python3 unicodedata`) · 실패 14건 **각 2회 재측**
- 소스는 매 측정 뒤 `git checkout --` → `git status --porcelain` **빈 출력** 확인(제품 코드 0줄)

## 한계 — 숨기지 않는다

- ★**판정 축은 boot+render 뿐이다.** 「두 배선의 화면이 같다」를 주장하지 않았다 — 픽셀 대조는 안 했다.
- ★**`broken/lgt` 46건 미측정**(§6⑶) · `ktf`·`skt` 는 이 배선의 대상이 아님.
- ★부하가 6~20 사이에서 움직였다 — **데드라인에 걸친 타이틀 2종**(`나는마왕이다2`·`아니마`)이 그 영향을 받았고,
  그래서 그 둘은 **판정 근거에서 빼고 «잡음»으로 따로 적었다**.
- ★프로브 ② 의 NOHIT 8건은 ★**「안 부른다」가 아니라 「못 쟀다」**이다.
- ★§4 의 12건이 **언제부터** 실패했는지는 **재지 않았다**(이분 필요 — §6⑵).

## 게임 파일명 유입 — 도구를 «실행해서» 적는다

`node scripts/corpus-name-inflow.mjs --corpus <코퍼스>`(★기본 경로는 repo 안 `game_lab` 인데 이 워크트리에는
없다 — 코퍼스는 `~/work/otterpebble/wie/game_lab` 이라 `--corpus` 로 가리켰다):
★**BOUNDED 33회 / 20쌍** · **PREFIX-EMBEDDED 0** · ★**SUFFIX-ATTACHED 2회 / 1쌍**.
★**첫 측정은 31회/1회였다 — 이 문단을 쓰면서 «회» 가 2씩 늘었다**(쌍 수는 **20/1 불변**).
★그래서 아래 표식은 **이 문단까지 쓴 뒤에** 다시 재서 붙인 것이다(`0187` 이 같은 자리에서 17/10 → 20/11 로 움직였다).

★**SUFFIX-ATTACHED 1건은 내가 가른다**(도구가 못 가르는 바구니다): `하이브리드` ⊂ `하이브리드2` —
★**둘 다 코퍼스의 «서로 다른 실제 타이틀»**이고 조사가 붙은 언급이 아니다. ⇒ 판단 필요분 **실질 0**.
★**유입된 이름은 전건 «이미 커밋된 식별자»다** — `scripts/smoke_gate_baseline.tsv` 가 292행으로 갖고 있는
그 이름들이고, §4 의 판정을 이름 없이 적으면 후속 회차가 **어느 타이틀인지 다시 재야** 한다.
게임 바이트 유입 **0** · 경로 유입 **0**.

<!-- corpus-name-inflow v1 subjects=4 tree=18be09fcff4e5ea5 B=33/20 P=0/0 S=2/1 -->
