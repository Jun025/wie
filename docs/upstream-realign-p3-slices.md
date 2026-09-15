# P3 남은 조각(⑵⑶⑷)의 회차 분할과 «이 머신에서의» 검증식

> 회차 티켓 `wie-p3-remaining-slices-split-and-verifiability-plan`. **계획 문서다 — 이 회차는 집행하지 않는다.**
> 제품 코드 0줄 · `Cargo.toml` 무접촉 · upstream 발신 0.
> 정본 판정은 `docs/upstream-realign-verdict.md`(2026-08-27, §8 이 2026-09-03 갱신).
> **이 문서는 그 판정을 대체하지 않고, P3 의 «남은 셋»을 회차로 쪼개고 각 조각의 검증식을 정한다.**
>
> ★**측정 트리 명시**: 아래 수치는 전부 2026-09-16 04:1x~05:0x KST 에 `git fetch` 한
> `origin/main` **`d30cb903`** ↔ `upstream/main` **`44fbf265`** 에서 쟀다. `merge-base` = `fa641a8a`.
> 워킹트리(`@live`) 기준 값은 하나도 쓰지 않았다. 프로브는 전부 `git worktree` 격리 체크아웃에서
> 돌고 **제거됐다** — 이 저장소 diff 에 남지 않는다.

## 0. 결론 먼저 — 세 가지가 «판정을 바꾼다»

1. ★★**`keydraw_lgt` 가 upstream base 에서 «깨진다».** 우리 트리 PASS(paints 55) ↔ upstream base
   **FAIL**(paints 0 · `Undefined instruction` at `net/wie/CletWrapperCard.paint`). 결정적 재현
   (2/2). ⇒ ★**「upstream LGT 는 우리의 진부분집합」(§3-1)은 «구현 개수» 축에서 참이고
   «동작» 축에서 «거짓»이다.** 코퍼스 없이도 신규 FAIL 1건을 찾았다 — §2.
2. ★★**P2 의 두 블로커는 «비대칭»이고, 둘 다 verdict 가 적은 것보다 작다.**
   ⒝러너 = ★**21줄**(772/1,147 아님 · 실제로 컴파일·실행했다) · ⒜코퍼스 = **여전히 부재**이나
   ★**«다른 머신»이 아니라 «이 머신의 git-ignored `game_lab/`»으로 풀린다** — §4.
3. ★★**⑶(`compile_model.rs` 122줄 이식)은 «이식 작업이 아니다».** 실제 머지에서 그 파일은
   `AU wie-lgt/src/compile_model.rs` 로 **혼자 도착한다**(git rename 탐지가 크레이트 개명을 따라간다).
   ⇒ 조각 하나가 **소멸**한다 — §3.

## 1. 오늘의 수 — verdict 의 2026-08-27 값을 상수로 인용하지 마라

```sh
git fetch origin && git fetch upstream
git rev-parse origin/main upstream/main && git merge-base origin/main upstream/main
git rev-list --left-right --count origin/main...upstream/main    # left=ahead right=behind
```

| 축 | verdict 2026-08-27 | §8 재측 2026-09-03 | ★**오늘 2026-09-16** |
|---|---:|---:|---:|
| `merge-base` | `fa641a8a` | `fa641a8a` | ★**`fa641a8a`**(한 번도 안 움직였다) |
| behind (upstream 만) | 1,067 | 1,089 | ★**1,140** |
| ahead (우리만) | 192 | 194 | ★**535** |

분류 재측(술어를 먼저 적는다 — 인상 배제):
`git diff --numstat fa641a8a origin/main` 을 경로로 갈랐다. ①=`wie_lgt/`·`wie_featurephone/` ·
③f=`wie_cli/` · ②=그 밖의 `wie_*`+`test_utils/` · ③=나머지 전부.

| 갈래 | verdict(전체축) | ★**오늘(전체축)** | ★**오늘(.rs 만)** |
|---|---|---|---|
| ① `wie_lgt` | 12f +2,426/−41 | **16f +2,747/−54** | 16f +2,747/−54 |
| ① `wie_web`→`wie_featurephone` | 7f +1,065/−0 | **7f +1,070/−0** | 6f +1,011/−0 |
| ② 엔진 고유 | 47f +1,908/−61 | **67f +2,632/−96** | **66f +2,629/−96** |
| ③ 로컬 스캐폴딩 | 154f +20,291/−62 | **367f +32,913/−69** | — |
| ③f fork 전용(`wie_cli`) | 1f +772/−0 | **5f +1,807/−0** | 4f +1,805/−0 |

★**`wie_validate.rs` 단독 = 772 → 오늘 `1,147`줄**(`git show origin/main:wie_cli/src/bin/wie_validate.rs | wc -l`).
★**`compile_model.rs` = 122줄 — 변하지 않았다**(`git show origin/main:wie_lgt/src/compile_model.rs | wc -l`).

### 1-1. ★verdict 가 모르는 구조 변화 — upstream 이 크레이트 디렉터리를 «전면 개명»했다

`git log --reverse --format='%h %ad %s' --date=short upstream/main -- wie-ktf | head -2`
→ ★**`49db5171` 2026-08-30 `Rename workspace crates`** · `691cf858` 2026-08-30 `Update workspace crate metadata`.
`git ls-tree --name-only upstream/main | grep -c 'wie_'` → ★**0**.

⇒ upstream 의 14개 크레이트가 `wie_x` → **`wie-x`** 로 갔다. ★**verdict 본문(2026-08-27)보다 뒤의 일**이고,
§8-1(2026-09-03)이 「전면 개명됐다」로 한 줄 적었으나 ★**조각 계획에는 반영되지 않았다.**

★★**그런데 이것은 «선행 조각»을 만들지 않는다 — git 이 스스로 따라간다.** §3 의 머지 예행에서
**146건이 `R`(rename)** 로 짝지어졌고 우리 수정분이 하이픈 경로에 그대로 착지했다.
⇒ ★**「우리 크레이트를 먼저 하이픈으로 개명한다」는 조각을 «만들지 마라». 측정으로 불필요함이 확인됐다.**

## 2. ★★이 회차가 «실제로 찾은» 회귀 — `keydraw_lgt` 는 upstream base 에서 FAIL 이다

verdict §3-5·§8-2 는 `helloworld_{ktf,lgt}` **2건**으로 「신규 FAIL 0」을 얻고 스스로
「부풀리지 마라 — helloworld 는 아무것도 그리지 않는다」고 적었다. ★**그 경고가 옳았다.**
그 뒤(2026-09-06) **그리는 픽스처**가 커밋됐다(`keydraw_ktf.zip`·`keydraw_lgt.zip`) —
★**verdict 는 그것들을 한 번도 upstream 에 대고 돌려 본 적이 없다.** 이 회차가 돌렸다.

**방법**: §4 의 21줄 이식본을 upstream `44fbf265` 격리 워크트리에 얹어 빌드하고, **우리 저장소의
픽스처 5건**을 양쪽 엔진에 같은 명령으로 먹였다. 게임 바이트 0 · 네트워크 0.
★**정확히는 «커밋 4건 + 생성 1건»이다**(초판의 「커밋된 픽스처 5건」은 부정확했다 · 정정 `-fix`):
`git ls-tree origin/main test_data/` → `draw_j2me.zip`·`helloworld_{ktf,lgt}.zip`·`keydraw_{ktf,lgt}.zip`
⇒ ★**`draw_j2me.jar` 는 커밋본이 아니다**(`.gitignore:24` `*.jar` · `git ls-files` **0**). 그것은
**커밋된 생성기** `scripts/make-draw-fixture.mjs` 가 만든다(`AGENTS.md` 러너 블록의 첫 줄이 그 writer 다).
★**그리고 커밋본 `draw_j2me.zip` 은 이 러너의 입력이 «아니다»** — 실측:
`wie_validate test_data/draw_j2me.zip` → `FAIL · unrecognized zip archive (no __adf__/app_info/.msd)`
(그 zip 은 브라우저 왕복 Scenario C 쪽 자산이다). ⇒ ★**러너 표는 «생성된 `.jar`» 로 읽어라.**

| 픽스처 | 명령 | ours `4cf79b4d` | ★**upstream base `44fbf265`** |
|---|---|---|---|
| `helloworld_ktf.zip` | `wie_validate <f>` | PASS · paints 0 | PASS · paints 0 |
| `helloworld_lgt.zip` | 〃 | PASS · paints 0 | PASS · paints 0 |
| `draw_j2me.jar` ★(생성본) | 〃 | PASS · paints 1 · content | PASS · paints 1 · content |
| `keydraw_ktf.zip` | `wie_validate --inject --expect-last-frame <f>` | PASS · paints 55 · rc 0 | PASS · paints 55 · rc 0 |
| ★**`keydraw_lgt.zip`** | 〃 | ★**PASS · paints 55 · rc 0** | ★★**FAIL · paints 0** |

★**신규 FAIL 1 · 신규 PASS 0.** FAIL 의 전문(앞부분):

```
tick error during 'boot': Fatal error:
net.wie.WieError: Fatal error: Undefined instruction
	at net/wie/CletWrapperCard.paint(Lorg/kwis/msp/lcdui/Graphics;)V
	at net/wie/CardCanvas.paint(Ljavax/microedition/lcdui/Graphics;)V
	at javax/microedition/lcdui/Canvas.handlePaintEvent(…)V
```

★**결정적이다** — `--inject` 2회 재실행에서 동일(FAIL · paints 0). 부하로 흔들리는 축이 아니다
(`AGENTS.md` 의 「paints 는 하한이지 등식이 아니다」는 **PASS 안에서의** 변동을 말한 것이고,
여기는 `result` 자체가 뒤집혔다).

★**양쪽 다 `net/wie/CletWrapperCard` 를 «갖고 있다»**(upstream `wie-lgt/src/runtime/java/classes/net/wie/clet_wrapper_card.rs` ·
ours `wie_lgt/src/runtime/java/classes/net/wie/clet_wrapper_card.rs`) ⇒ **클래스 부재가 아니라 ARM 실행 경로의 차이**다.
★**여기서 원인을 단정하지 않는다** — 그것이 조각 **A** 의 일이다.

★★**이 한 줄이 이 회차의 실제 산출이다**: verdict 는 「코퍼스가 없으면 go/no-go 를 못 잰다」고 적었는데,
★**코퍼스 없이, 커밋된 픽스처만으로, no-go 신호 1건이 나왔다.** ⇒ ★**「P2 없이 간다」를 «검증 없이 간다»로
읽어서는 안 된다 — 이 5건은 이미 무언가를 «말한다».** 동시에 ★**이 5건이 292건을 대체하지도 않는다**(§5).

## 3. ★머지 예행 — «조각을 어떻게 나눌 수 있는가»는 여기서 결정된다

```sh
git worktree add --detach /tmp/probe origin/main && cd /tmp/probe
git merge --no-commit --no-ff upstream/main        # rc=1
git status --porcelain | awk '{print substr($0,1,2)}' | sort | uniq -c | sort -rn
git merge --abort && cd - && git worktree remove --force /tmp/probe
```

| 상태 | 건수 | 뜻 |
|---|---:|---|
| `A ` | **167** | upstream 신규 — 자동 |
| `R ` | **146** | ★**개명 추적 성공**(§1-1) — 자동 |
| `D ` | **104** | upstream 삭제 — ★**전부 «자동»으로 두면 안 된다**(아래 3-2) |
| `UU` | **35** | 양쪽 수정 — **손으로 화해** |
| `AU` | **19** | 우리만 추가 — 대개 `git add` 또는 폐기 |
| `UD` | **17** | 우리 수정 ↔ upstream 삭제 — **재적용 대상** |
| `AA` | **3** | 양쪽 독립 추가 |
| **미해결 합계** | ★**74** | |

★**verdict 의 「충돌면 43파일」(2026-08-27 정적 교차)은 오늘 정적으로 70, 실제 머지로 74 다.**

갈래별:

| 갈래 | 미해결 | 내역 |
|---|---:|---|
| ① `wie_lgt` | **12** | UU 5 · UD 4 · AU 3 |
| ② 엔진 | **51** | UU 21 · AU 16 · UD 13 · AA 1 |
| ③ 스캐폴딩·설정 | **11** | UU 9(`.github/*`×3 · `AGENTS.md` · `Cargo.toml` · `Cargo.lock` · `.gitignore` · `README.md` · `docs/lgt.md`) · AA 2(`package.json`·`package-lock.json`) |

★★**③ 오버레이의 «본체»는 충돌하지 않는다** — 머지된 워크트리에서 실측:
`web/` 32파일 · `functions/` 24 · `migrations/` 8 · `scripts/` 17 · `docs/report/` 108 · `docs/worklog/` 93 ·
`wie_featurephone/` 7 **전건 온존**. ⇒ ★**⑵의 「③ 재적용」은 «재적용»이 아니다 — 그냥 남는다.**
비용은 ③이 아니라 **②(51) + ①(12) + 설정(11)** 에 있다.

### 3-1. ★⑶ 은 «소멸한다» — `compile_model.rs` 는 혼자 도착한다

머지 예행에서 그 파일의 상태는 ★**`AU wie-lgt/src/compile_model.rs`** 다.
⇒ 우리 122줄이 **하이픈 크레이트 경로로 따라와** 「우리만 추가」로 서 있다. 처분은 `git add` 한 줄이고,
검증은 `wie_featurephone` 이 `wie_lgt::detect_compile_model` 을 계속 import 할 수 있는가 뿐이다.
★**「122줄 이식」이라는 조각을 따로 세우지 마라** — 조각 **D** 안의 한 줄이다.

### 3-2. ★★«충돌 0» 이 «빌드 성공»을 뜻하지 않는 자리 2건

> ★★**[정정 2026-09-16 · 게이트② 반려 승계 `-fix`] 이 절의 초판은 「이 둘은 충돌 74건 «밖»이다」로
> 적었고 ★그 주장은 «거짓»이었다 — 두 항목 다 `UD` 로 74 «안»에 있다.**
> ★**근인**(다음 사람이 같은 자리를 밟지 않도록): 초판이 상태를 **`git status --porcelain -- wie_cli`**
> 라는 **pathspec 제한 조회**로 읽었다. ★**경로를 제한하면 git 이 rename 짝을 깨고 `R` 을 `D` 로 보여 준다** —
> 목적지가 pathspec 밖이기 때문이다. 실측 대조(같은 머지 트리, 같은 순간):
> ```
> $ git status --porcelain -- wie_cli          $ git status --porcelain | grep wie_cli
> UD wie_cli/Cargo.toml                        R  wie_cli/src/database.rs   -> src/database.rs
> D  wie_cli/src/audio_sink.rs                 R  wie_cli/src/filesystem.rs -> src/filesystem.rs
> D  wie_cli/src/database.rs                   R  wie_cli/src/main.rs       -> src/lib.rs
> D  wie_cli/src/filesystem.rs                 R  wie_cli/src/window.rs     -> src/window.rs
> D  wie_cli/src/main.rs                       UD wie_cli/Cargo.toml
> D  wie_cli/src/window.rs                     D  wie_cli/src/audio_sink.rs
> ```
> ⇒ ★**머지 상태를 읽을 때 «경로로 좁히지 마라». 전체를 받아 `grep` 하라.**

★**그래서 남은 참인 명제는 이것 하나다 — 그리고 이것으로 «충분»하다**:
★★**«미해결 0» 은 «컴파일된다»를 뜻하지 않는다.** 아래 둘은 충돌 목록에 **있지만**,
그 항목을 「충돌 해소」로만 처리하면(양쪽 중 하나를 고르고 끝내면) **빌드가 깨진다.**

| # | 무엇 | 실측 상태 | 왜 «해소»만으로 부족한가 |
|---|---|---|---|
| 1 | ★**`wie_cli` 매니페스트·bin 타깃 미화해** — upstream 이 네이티브 호스트를 루트 패키지 `wie`(`src/`)로 옮겼다 | ★**`UD wie_cli/Cargo.toml` 1건**(74 안) + ★**`R` 4건**(`src/{database,filesystem,window}.rs` → `src/…` · `src/main.rs` → `src/lib.rs`) + **`D` 1건**(`src/audio_sink.rs`) | ★★**`wie_validate.rs` 는 머지가 «손대지 않는다»** — 실측: 상태 목록 히트 **0건** · 머지 트리에 실재 · `git diff origin/main -- wie_cli/src/bin/wie_validate.rs` = **0줄**. ★**위험은 «소스 소실»이 아니라 «매니페스트·bin 타깃 미화해 시 러너 빌드 불가»** 다: `UD Cargo.toml` 을 upstream 쪽(삭제)으로 고르면 크레이트가 사라지고, 우리 쪽으로 고르면 그 매니페스트가 가리키는 `main.rs`·`window.rs`·`database.rs`·`filesystem.rs` 가 **루트로 이사한 뒤**다 |
| 2 | ★**`wie_backend` 폰트 include 경로** — upstream 은 폰트를 `assets/neodgm.ttf` 로 옮기고 `Platform::font()` 로 읽는다(실측: `test-utils/src/platform.rs`·`tests/font.rs`). 우리 `canvas.rs:16` 은 `include_bytes!("../../fonts/neodgm.ttf")` 로 **직접** 읽는다 | ★**`UD wie_backend/src/canvas.rs`**(74 안) · 머지 트리에 `fonts/neodgm.ttf` **GONE** · `assets/neodgm.ttf` **실재** | 그 `UD` 를 「우리 +149줄을 upstream 판본 위에 얹는다」로만 풀고 **include 경로를 안 고치면** 컴파일 타임에 없는 파일을 가리킨다. ★**충돌은 «보인다» — 보이지 않는 것은 «그 안에 경로 수정이 들어 있다»는 사실이다** |

★★**`binary_patches` 축은 «파열이 아니다» — 초판이 틀렸다.** upstream **도** 그 코드를 갖고 있고
자산과 **함께** 이사시켰다. 머지 트리 실측:
```
R  data/binary_patches.toml                 -> wie-core-arm/data/binary_patches.toml
R  wie_core_arm/src/binary_patches/parser.rs -> wie-core-arm/src/binary_patches/parser.rs
$ grep -n include_str wie-core-arm/src/binary_patches/parser.rs
11: const BINARY_PATCHES_TOML: &str = include_str!("../../data/binary_patches.toml");   # ★"../../" (우리는 "../../../")
```
`wie-core-arm/src/binary_patches/parser.rs` 에서 `../../data/…` = **`wie-core-arm/data/binary_patches.toml`** 이고
그 파일은 **실재한다**. ⇒ ★**깨지지 않는다.** ★초판의 오류는 **우리 트리의 include 줄**을 읽고
**머지 트리의 파일 위치**와 맞붙인 것 — ★**한쪽만 읽고 다른 쪽을 가정한, 위 pathspec 오류와 같은 계급**이다.

⇒ ★**조각 D 의 ⒟ 를 «충돌 0» 이 아니라 «빌드 게이트»로 두는 근거는 그대로 살아 있다** —
근거가 「목록 밖이라 안 보인다」에서 ★**「목록 안에 있어도, 해소 ≠ 컴파일」**로 바뀌었을 뿐이다.
★**게이트를 빼지 마라.**

### 3-3. ★★머지는 «쪼갤 수 없다» — 그래서 «머지 앞»에서 쪼갠다

머지 커밋은 하나다. 74건을 두 PR 로 나눌 방법이 없다. ⇒ ★**분할의 축은 «머지를 작게 만드는 선행 회차»** 다:
② 51건 중 **upstream 이 이미 가진 것**을 머지 «전»에 우리 쪽에서 지우면, 그만큼 충돌이 줄고
그 삭제는 **우리 현재 base 에서 4게이트로 검증된다**(머지 안에서 하면 검증이 섞인다).

## 4. ★★P2 를 되살리는 «최소 비용» — 재라고 해서, 쟀다

verdict §8-1 은 「코퍼스 있는 머신 **그리고** `wie_validate` 772줄을 upstream 개명 크레이트 위로 이식 —
**둘 다** 필요」라고 적었다. 두 축을 각각 쟀다.

### 4-1. ⒝ 러너 이식 = ★**21줄**(그리고 «개명»은 비용이 아니다)

**방법**: `git worktree add --detach /tmp/probe upstream/main` → `wie_cli/src/bin/wie_validate.rs` 를
그대로 복사 → 루트 패키지에 `[[bin]]` 2줄 + `test-utils`·`png` 의존 2줄 → `cargo check --bin wie_validate`.

| 회차 | 결과 |
|---|---|
| 원본 그대로 | ★**12 errors** — `E0407`×5(`AudioSink` 에 없는 메서드) · `E0046`×4(미구현 항목) · `E0050`×3(인자 수) |
| 어댑터 21줄 수정 후 | ★**rc=0** · `cargo build` · §2 의 픽스처 5건 실행 성공 |

> ★**[정정 `-fix`] 초판은 「15 errors」라고 적었다 — 그 수는 `png` 의존을 «아직 선언하기 전»에 잰 값**이고,
> 늘어난 3건은 전부 `E0433 cannot find crate png` 였다. `png` 선언은 위 `Cargo.toml` **4줄에 이미 포함**돼
> 있으므로 **소스 델타로 두 번 세면 안 된다.** ⇒ ★**이식 델타의 정확한 수는 «12» 다**(재측 `cargo check
> --bin wie_validate` → `due to 12 previous errors`). ★**아래 델타 표가 예측하는 수와 정확히 일치한다**
> (5 + 4 + 3 = 12). ★**「21줄」 결론은 그대로다** — 같은 21줄을 얹고 `rc=0` 을 다시 받았다.

★★**「개명 이식」은 **0줄**이다** — cargo 가 패키지명 `wie-ktf` 의 lib 를 **`wie_ktf`** 로 노출하므로
`use wie_ktf::KtfEmulator;` 가 **한 글자도 바뀌지 않는다**. verdict §8-1 의 「upstream 크레이트 이름 위로 이식」은
그 축에 대해 **비용을 0 이 아닌 것으로 읽히게** 한다.

★**진입 API 는 «바이트 동일»하다**(실측): `KtfEmulator::{from_archive,from_jar,loadable_archive,loadable_jar}` ·
`LgtEmulator`·`SktEmulator`·`J2MEEmulator` 전건. `from_jar` 7인자 시그니처는 양쪽이 **글자 그대로 같다**.
`test-utils` 의 `MemoryFilesystem` 도 양쪽에 있다.

★★**12건 전부가 «호스트 어댑터» 한 덩어리에 있다**(`wie_validate.rs:202~344` = `HeadlessScreen`·
`HeadlessAudioSink`·`MemDbRepository`·`HeadlessPlatform`). ★**에뮬레이터를 모는 본문 — 틱 루프 ·
주입 스케줄 27스텝 · JSON 판정 — 에서는 오류가 «0» 이다.**
★**여기에 「15」를 쓰면 위 정정 블록과 «자기모순»이다** — 15 의 여분 3건은 `E0433 cannot find crate png`
이고 그것은 ★**어댑터 델타가 아니라 매니페스트 의존**이다(그리고 `Cargo.toml` 4줄에 이미 세어져 있다).

| 트레이트 | 델타 | 줄 |
|---|---|---:|
| `Screen` | `resize(&self,u32,u32)->Result<()>` **추가** | +3 |
| `AudioSink` | `play_wave`+`midi_*` **5개 → `send(AudioCommand)` 1개** | −5/+1 |
| `DatabaseRepository` | `open`/`exists`/`delete` 의 `&System` 인자 **제거** · `usage(&str)->u64` **추가** | 3 수정 +3 |
| `Platform` | `font(&self)->&Font` **추가** | +3 |
| (의존) | `png` 1줄 · `[[bin]]` 2줄 · `test-utils` 1줄 — `Cargo.toml` | +4 |

⇒ ★**`+/-` 합계 21줄**(본문) **+ 4줄**(`Cargo.toml`). ★**772 도 1,147 도 아니다.**
★**단 「어댑터를 upstream 시맨틱에 맞게 채웠다」가 아니라 「컴파일되게 스텁했다」** — `Platform::font` 는
`unimplemented!()` 고 `AudioSink::send` 는 no-op 이다. **헤드리스 판정에는 충분하고**(5픽스처가 실제로 돌았다)
★**소리·폰트를 판정에 쓰는 순간 그 스텁은 거짓이 된다.** 조각 **A**·**D** 에서 쓸 때 이 한계를 안고 써라.

### 4-2. ⒜ 코퍼스 — ★**「다른 머신」이 아니라 「이 머신의 git-ignored 디렉터리」다**

verdict §5·§7-1·§8-1 은 「게임 바이트는 Constraint 9 로 repo 에 들어올 수 없으므로 **이 머신에서는 영구히**
못 잰다」고 적었다. ★**그 문장의 두 반쪽은 계급이 다르다**:

- 「repo 에 들어올 수 없다」 — **참이고 불변**이다(Constraint 9).
- 「이 머신에서 영구히 못 잰다」 — ★**참이 아니다.** `.gitignore:23` 이 **`/game_lab/`** 를 무시하고,
  `AGENTS.md` 가 `scripts/smoke_gate.sh` 를 스스로 「**local only, and structurally so** — reading titles
  from `WORKING_DIR`(default `game_lab/working`). **`game_lab/` is git-ignored and holds real game bytes**」
  라고 적는다. ⇒ ★**그 배치는 «금지된 것»이 아니라 «설계된 것»이다.** 못 재는 이유는 구조가 아니라
  ★**그냥 코퍼스가 여기 없다**는 것이다(`find ~ -maxdepth 4 -name game_lab` → **0건** · 오늘 재확인).

⇒ ★★**human-step 후보(이 회차는 «후보»까지만 적는다 — 카드 발권은 총괄 몫)**:

> `~/work/otterpebble/wie/game_lab/working/{ktf,lgt,skt}/` 에 `scripts/smoke_gate_baseline.tsv` 가
> 이름으로 열거한 **292 타이틀**의 아카이브를 놓는다. 디렉터리는 git-ignored 이고 `npm run audit`
> (`scripts/audit-no-leak.sh`)이 유출을 막는다. ★**그러면 P2 는 «이 머신에서» 돈다** — ⒝는 §4-1 로 이미 풀렸다.

★**그 human-step 이 불가하면 P2 는 영구 미측정으로 남는다** — 그 경우의 대가가 §5 다.
★**그리고 그때도 조각 A·B·C·D 는 전부 돌 수 있다**(검증 강도만 낮다). 막혔다고 멈출 축이 아니다.

## 5. ★대가 비교 — 두 갈래를 «나란히» 적는다. 이 회차는 고르지 않는다

| | **갑: P2 없이 간다**(커밋된 5픽스처만) | **을: P2 를 먼저 세운다**(코퍼스 human-step) |
|---|---|---|
| **잃는 것** | ★**292 타이틀이 전부 미검증**이다. 갈래별 = `ktf 190 · lgt 52 · skt 50`. ★**`skt` 는 커밋된 픽스처가 «하나도 없다»** ⇒ 50건이 **0 신호**다. 증상은 조용하다 — 4게이트 green 인 채로 특정 게임이 특정 지점에서 죽는다(verdict §6-P1 이 하드닝 상실에 대해 적은 것과 **같은 형태**) | ★**시간**. human-step 지연이 곧 P3 지연이고 그 길이는 이 레인이 정하지 못한다. 그리고 ★**292 기준선 자체가 «우리 엔진에서 잰 값»**이라 upstream 에서의 FAIL 이 「회귀」인지 「애초에 우리 것이 더 관대했나」인지는 **차이표만으로는 안 갈린다**(개별 추적이 필요하다) |
| **얻는 것** | 지금 시작한다. upstream **1,140커밋**(LGT import 8→31 · LGT 전용 그래픽 0→1,095줄 · canvas 709→1,647)을 탄다 | ★**FAIL 목록이 그대로 ⑷의 작업목록이 된다**(verdict §6-P2 의 설계). 조각 **B** 의 분류가 추측이 아니라 **실패한 타이틀**로 정렬된다 |
| **이미 아는 것** | ★★**갑은 «검증 0» 이 아니다** — 5픽스처가 이미 **신규 FAIL 1건**을 말했다(§2) | ★**을은 «갑을 대체»하지 않는다** — §2 의 FAIL 은 코퍼스가 있어도 그대로 있다 |

★★**그래서 이 회차의 판정은 「갑이냐 을이냐」가 아니다**: ★**조각 A 는 어느 쪽에서도 «먼저» 돈다.**
이미 손에 든 no-go 신호를 규명하지 않고 base 를 갈아타는 것은 어느 갈래에서도 정당화되지 않는다.
갈래가 실제로 가르는 것은 ★**조각 D 가 «5픽스처 증거»로 착지하느냐 «292 차이표 증거»로 착지하느냐** 하나다.

## 6. ★★조각 표 — ⒜범위 ⒝size/risk ⒞선행 ⒟이 머신에서의 검증식 · 그리고 «못 재는 것»

> ★**빈 칸을 남기지 않았다.** 「못 잰다」는 칸은 **「무엇이 있으면 잴 수 있는가」와 함께** 적었다.
> ★**전 조각 공통 DoD**: ⓐ`--squash` **금지**(`contracts/upstream-sync-repos.conf` 등재 repo ·
> `bin/queue-lint` 검사 22 가 **모든** `*-merge` 티켓에 `merge_strategy:` 선언을 요구 · 값은 `merge`) ·
> ⓑ**자기 PR 자기 머지 0**(`AGENTS.md` Constraint 12) · ⓒ`STATE.md ## 다음` 갱신 + `docs/report/NNNN--…` 1장.

### ★이미 끝난 것 — 다시 발권하지 마라 (P3 ⑴ · ⑸)

| P3 항 | 언제 | 상태 |
|---|---|---|
| ⑴ `wie_web` → `wie_featurephone` 개명 | 2026-09-11 `wie-p3-rename-wie-web-to-featurephone` | ★**끝났다.** 산출물 이름 `wie_web.js`/`wie_web_bg.wasm` 은 **일부러 안 바꿨다**(otterpebble 소비자 계약) |
| ⑸ 엔트리포인트 규약 정합 | 2026-09-13 `wie-lgt-entrypoint-jar-name-contract-align-with-upstream`(+ `…-under-p-prefix-never-opens`) | ★**끝났고, ⑸ 의 «전제»가 반증됐다** |

★★**⑸ 의 전제 반증을 여기 «리터럴로» 남긴다 — verdict §3-5 의 낡은 문안이 조각 브리프로 되살아나지 않도록**:
verdict §3-5 는 「upstream `LgtEmulator` 는 아카이브에서 **`application.jar`** 를 찾고 우리는
`00000000.jar` 를 그대로 넘긴다」라고 적었다. ★**그 문장은 부정확하다.**
★**upstream 은 어떤 이름도 하드코딩하지 «않는다»** — `*.jar` 중 **zip 안에 `binary.mod` 가 있는 것**을
**내용으로** 찾는다(`upstream/main:wie-lgt/src/emulator.rs` · 2026-08-23 PR #1368 이 `format!("{aid}.jar")` 를
그 형태로 바꿨다). upstream 테스트가 픽스처의 `00000000.jar` 를 `application.jar` 로 개명해 넘기는 것은
**그 탐색을 증명하려는 장치**였다. ⇒ 우리도 그 술어를 **그대로 채택**했으므로 ★**이름 규약이 «둘»로 남지
않았고 기존 `00000000.jar` 픽스처도 그대로 산다.** ★**조각 D 의 목록에 「엔트리포인트 이름 맞추기」를
넣지 마라 — 없는 일이다.**

### A — `wie-p3-lgt-keydraw-upstream-regression-triage`

> ★★★**[돌았다 2026-09-16 · `wie-p3-slice-a-keydraw-lgt-breaks-on-upstream-base` · 정본 `docs/report/0113--….md`]
> 원인이 «이름»으로 나왔다 — 이 칸의 ⒜~⒟ 는 그 회차가 실제로 따른 것이고, 아래가 그 답이다.**
>
> ★**원인**: upstream 이 graphics SVC **27개**를 LGT 전용 구현(`wie-lgt/src/runtime/wipi_c/graphics.rs` ·
> **1,095줄**)으로 보내고, 그 구현이 게스트에게 **다른 레코드 ABI** 를 준다 —
> `LgtFramebuffer{owned_image,ptr_graphics,ptr_image,screen_kind}` **16B**(★`buf` 필드 **없음**) ↔
> 공용 `WIPICFramebuffer{width,height,bpl,bpp,**buf**}` **20B**(픽셀 포인터 **+16**).
> 게스트 SDK(`dlunch/wipi` 의 `wipi/src/framebuffer.rs`)가 `fb.buf` = **+16** 을 읽어 **끝 너머**에서 0 을 얻고
> ★**스스로 패닉**한다(게스트 printk 전문: `panicked at wipi/src/framebuffer.rs : 149 : 18 : null reference produced`).
> 그 뒤 주소 0 으로 분기해 `Undefined instruction`(PC=0x0)이 난다 ⇒ ★**`CletWrapperCard.paint` 스택은 증상이다.**
> ★**도입 커밋 = `9a88423b`(2026-08-23) `Fix LGT graphics and runtime compatibility (#1368)`** —
> ★**⑸ 엔트리포인트를 바꾼 그 PR 과 «같다».**
>
> ★**무는 것(결정 실험)**: `wie-lgt/src/runtime/wipi_c.rs` 의 `=> graphics::` **27건**을
> `=> wie_wipi_c::api::graphics::` 로 치환 → ★**PASS · paints 55**(2/2). 원본은 FAIL · paints 0(3/3).
>
> ★**배제된 가설 4종**(전부 실행): 포인터 등록(양쪽 `0x25619` 동일 · `paint` 본체 바이트 동일) ·
> `init_process_state`/`set_use_annunciator` 2줄(꺼도 FAIL) · **202 단일 치환**(FAIL 불변) ·
> 공용 구현 자체(두 트리 diff **0**).
> ★**범위**: `keydraw_ktf` upstream **PASS**(upstream 도 공용 경로) · `helloworld_lgt` upstream **PASS**
> ⇒ ★**LGT «그리기» 경로 한정**이다.
>
> ★★**⇒ 조각 D 로 넘어간 «결정 항목»**: base swap 시 그 **27줄을 어느 쪽으로 두는가**.
> ⒜upstream LGT 전용 유지 ⇒ **SDK 기반 게스트(우리 픽스처)가 깨진다** ·
> ⒝공용으로 되돌림 ⇒ **upstream 의 LGT 리버스 1,095줄을 버린다**.
> ★**이 결정은 코퍼스 없이 «안전하게» 내릴 수 없다** — 아래 「못 재는 것 ⑵」가 그 이유이고,
> 그 축은 이 회차가 **좁혔지만 닫지 못했다**(`docs/lgt_abi.md:930` 이 실제 clet 타이틀 놈ZERO 에서
> 「`GetScreenFrameBuffer` 가 준 포인터에 **직접** 픽셀을 쓴다」를 관측했다 ⇒ 우리 픽스처와 **같은 모양**.
> ★그러나 upstream 이 픽셀을 `ptr_image`→`LgtImage` **한 겹 아래**로 내줄 가능성은 배제되지 않았다).

- **⒜ 범위**: 제품 코드 **0줄**. 격리 워크트리 조사만. 대상 = upstream `wie-lgt` 의 clet paint 경로
  (`net/wie/CletWrapperCard.paint` → ARM `Undefined instruction`) ↔ 우리 `wie_lgt/src/runtime/`.
  산출물 = `docs/report/` 1장 + (필요시) upstream 이슈 **초안**(★**발신하지 마라** — §7).
- **⒝** size **M** · risk **low**(읽기 전용)
- **⒞ 선행**: 없음. ★**이 조각이 D 의 게이트다.**
- **⒟ 검증식**:
  ```sh
  # 재현 (upstream base) — 3회 전부 FAIL 이어야 «결정적»이다
  git worktree add --detach /tmp/p upstream/main
  # …§4-1 의 21줄 이식…
  for i in 1 2 3; do /tmp/p/target/debug/wie_validate --inject \
      ~/work/otterpebble/wie/test_data/keydraw_lgt.zip | jq -r '.result,.paints'; done
  # 기대: FAIL 0 / FAIL 0 / FAIL 0
  # 대조군 (ours) — 같은 픽스처가 PASS 여야 한다
  cargo build -p wie_cli --bin wie_validate
  ./target/debug/wie_validate --inject --expect-last-frame test_data/keydraw_lgt.zip; echo rc=$?
  # 기대: "result":"PASS" · paints ≥ 28 · rc=0   (paints 는 하한 — AGENTS.md)
  # 이분: 1,140커밋 중 그 경로를 만진 커밋으로 좁힌다
  git log --oneline upstream/main -- wie-lgt/src/runtime/java/classes/net/wie/clet_wrapper_card.rs
  ```
- ★**못 재는 것**: ⑴이 FAIL 이 **실제 LGT 타이틀 52건 중 몇 건에 닿는가** — 픽스처 1건은 빈도를 모른다.
  ★**있으면 잴 수 있는 것** = `game_lab/working/lgt/` 코퍼스(§4-2). ⑵**`keydraw_lgt` 가 대표적인가**
  — 그 픽스처는 우리가 만든 것이라 우리 경로에 유리하게 편향됐을 수 있다. ★**교차 픽스처로는 못 푼다**:
  실측 `git ls-tree -r --name-only upstream/main | grep -i 'test.*data'` → upstream 이 가진 것은
  `wie-ktf/tests/data/helloworld_ktf.zip` · `wie-lgt/tests/data/helloworld_lgt.zip` ★**둘뿐이고 둘 다
  아무것도 그리지 않는다** ⇒ ★**upstream 쪽에는 이 축의 대조군이 «없다».**
  ★**있으면 잴 수 있는 것** = 코퍼스, 또는 upstream 이 나중에 그리는 픽스처를 커밋하는 것.

### B — `wie-p3-engine-overlay-hunk-triage`

- **⒜ 범위**: 제품 코드 **0줄**. ② 미해결 **51건**(UU 21 · AU 16 · UD 13 · AA 1)을 파일 단위로
  **{폐기 — upstream 이 이미 가졌다 / 재적용 — 우리만 있다 / upstream 으로 보낸다(P4)}** 로 분류한 표 1장.
  ★verdict §6-P4 의 「선 안쪽 10종 + `canvas.rs` 테스트 9개」를 **오늘 값으로 재측**해 그 표에 합친다.
- **⒝** size **M** · risk **low**
- **⒞ 선행**: 없음(A 와 병행 가능)
- **⒟ 검증식**: 분류의 «참/거짓»은 파일별로 기계로 확인된다.
  ```sh
  # 「upstream 이 이미 가졌다」의 술어 — 심볼 단위로 세라. 파일 존재로 세지 마라
  git show upstream/main:wie-wipi-java/src/classes/org/kwis/msp/lcdui/card.rs | grep -c 'fn '
  git show origin/main:wie_wipi_java/src/classes/org/kwis/msp/lcdui/card.rs   | grep -c 'fn '
  # 표의 모든 행이 위 두 수 + 「ours 에만 있는 심볼 목록」을 갖는다 (빈 칸 0)
  ```
  ★**Acceptance**: 51행 전건이 분류되고 ★**「ours 에만」열이 «비어 있다»와 «안 셌다»가 구별된다**.
- ★**못 재는 것**: **분류가 «동작»을 보증하지 않는다** — 같은 이름의 메서드가 다르게 동작할 수 있다.
  ★**있으면 잴 수 있는 것** = 코퍼스(§4-2), 또는 그 심볼을 실제로 부르는 픽스처. ★**그래서 이 표는
  «폐기 후보»를 만들 뿐 «폐기»를 집행하지 않는다** — 집행은 C 가 4게이트를 지고 한다.

### C — `wie-p3-drop-already-upstreamed-engine-hunks`

- **⒜ 범위**: B 가 «폐기» 로 분류한 것만 **우리 현재 base 에서** 삭제. ★**`wie_lgt/` ① 12건은 여기 넣지 마라**
  (그건 D 의 머지가 `D`/`UD` 로 처리한다). ★**여러 PR 로 쪼개라** — 크레이트 단위가 자연 경계다.
- **⒝** size **M**(PR 당) · risk **med** — ★삭제는 되돌리기 쉽지만 **조용히 기능을 지운다**.
- **⒞ 선행**: **B**
- **⒟ 검증식**: ★**이 조각은 전 조각 중 검증이 가장 강하다 — 우리 base 위라 모든 게이트가 산다.**
  ```sh
  cargo fmt --all -- --check
  cargo clippy --all -- -D warnings
  cargo +beta clippy --all -- -D warnings
  cargo clippy --target wasm32-unknown-unknown -- -D warnings
  RUST_MIN_STACK=4194304 cargo test --all
  node scripts/make-draw-fixture.mjs
  for f in test_data/draw_j2me.jar test_data/helloworld_ktf.zip test_data/helloworld_lgt.zip; do
    cargo run -q -p wie_cli --bin wie_validate -- "$f"; done          # 각 "result":"PASS"
  for f in test_data/keydraw_ktf.zip test_data/keydraw_lgt.zip; do
    cargo run -q -p wie_cli --bin wie_validate -- --inject --expect-last-frame "$f"; done  # PASS + rc=0
  ```
  ★**그리고 «줄었는지»를 세라**: 삭제 전후로 §3 의 머지 예행을 다시 돌려 ★**미해결 74 → N** 을 적어라.
  줄지 않았으면 그 PR 은 **목적을 달성하지 못한 것**이다.
- ★**못 재는 것**: 지운 심볼을 **게임만 부르는** 경우 — 5픽스처는 그것을 못 본다.
  ★**있으면 잴 수 있는 것** = 코퍼스. ★**완화책(코퍼스 없이도 가능)**: 지우기 전에
  `grep -rn '<심볼>' wie_* web/ functions/` 로 **우리 안의 호출자 0건**을 먼저 보이고, 0건이 아니면 지우지 마라.

### D — `wie-p3-base-swap-merge`

- **⒜ 범위**: ★**⑵ 그 자체.** `git merge upstream/main` 한 커밋 + 해소. ★**리터럴 작업 목록**:
  ⑴미해결 잔여 전건 해소 ⑵★**`wie_cli` 매니페스트·bin 타깃 화해**(`UD wie_cli/Cargo.toml` **1건** · §3-2 —
  ★**`wie_validate.rs` 자체는 머지가 손대지 않는다.** 위험은 «소스 소실»이 아니라 «러너 빌드 불가»다:
  그 매니페스트가 가리키던 `main.rs`·`window.rs`·`database.rs`·`filesystem.rs` 가 **`R` 로 루트 패키지에 이사**했고
  `audio_sink.rs` 는 **`D`** 다)
  ⑶★**`wie_backend/src/canvas.rs` 의 폰트 include 교정**(`UD` 를 풀 때 `fonts/` → `assets/neodgm.ttf` ·
  ★**`binary_patches` 축은 «깨지지 않는다»** — upstream 이 자산과 함께 이사시켰다 · §3-2)
  ⑷`compile_model.rs` 는 `git add`(§3-1) ⑸`wie_featurephone` 무접촉 ⑹`Cargo.toml` 워크스페이스 멤버 화해
  (upstream 은 `default-members=["."]` 루트 패키지 · 우리는 `wie_cli`).
  ★**⑵⑶ 을 「충돌 하나 지우면 끝」으로 읽지 마라** — 둘 다 **해소 안에 «코드 수정»이 들어 있다.**
- **⒝** size **L** · risk ★**high**(verdict 의 `med` 를 올린다 — 근거: 미해결 74 + ★**«해소 ≠ 컴파일» 2건**(§3-2) + §2 의 FAIL)
- **⒞ 선행**: ★**A**(규명 없이 갈아타지 않는다) · **C**(충돌을 줄인 뒤에 한다) · ★**을 갈래를 고르면 P2 도**
- **⒟ 검증식**: C 의 5게이트 + 5픽스처 **전부**, 그리고 ★**계보 DoD 를 «리터럴로»**:
  ```sh
  # ★머지 후 반드시 — 이 값이 fa641a8a 이면 그 회차는 «실패»다
  git merge-base origin/main upstream/main    # 기대: fa641a8a 가 «아니다»
  # ★«미해결 0» 은 «컴파일된다»가 아니다 — §3-2 의 둘을 «빌드»로 확인하라
  cargo build -p wie_cli --bin wie_validate   # ★러너가 살아 있는가 (⑵)
  cargo build -p wie_backend                  # ★폰트 include 가 해결되는가 (⑶)
  node scripts/check-engine-contract.mjs      # 웹 계약 표면
  npm run audit
  ```
  ★**게이트③ 에서 `merge_strategy: merge`** — `--squash` 는 계보를 평평하게 만들어
  `merge-base` 를 `fa641a8a` 에 못박는다(그 다음 회차가 **또 1,140커밋 뒤**에서 시작한다).
- ★**못 재는 것**: ⑴**292 타이틀**(갑 갈래를 고른 경우) — ★있으면 잴 수 있는 것 = 코퍼스.
  ⑵**브라우저 왕복**(`contract-roundtrip.mjs`) — 로컬에 playwright chromium 이 필요하고 CI 가 진다
  ⇒ 조각 **E**. ⑶★**네이티브 창 호스트**(`wie_cli` 의 `WindowHandle`) — `wie_validate` 는 `HeadlessScreen`
  을 지나므로 ★**두 그물 다 이 호스트를 덮지 않는다**(`AGENTS.md` 가 이미 그렇게 적었다).
  ★있으면 잴 수 있는 것 = 사람이 창을 띄워 한 번 보는 것 — 자동화 없음.

### E — `wie-p3-post-swap-web-contract-and-artifact`

- **⒜ 범위**: 새 base 에서 **웹 표면**이 계약을 지키는가. `web/`·`scripts/build-wasm.sh`·
  `docs/contracts/featurephone-engine-contract.json`. ★**계약이 바뀌면 같은 PR 에서 갱신**(Constraint 3).
  ★**산출물 이름 `wie_web.js`/`wie_web_bg.wasm` 은 «바꾸지 마라»** — 그것은 upstream 충돌이 아니라
  **otterpebble 소비자 계약**이다(2026-09-11 회차가 일부러 남긴 값).
- **⒝** size **M** · risk **med**
- **⒞ 선행**: **D**
- **⒟ 검증식**:
  ```sh
  npm run build:wasm && node scripts/check-engine-contract.mjs
  npx playwright install chromium && node scripts/contract-roundtrip.mjs   # Scenario A~F
  npm run frontend
  gh pr checks <n> -R Jun025/wie      # ★로컬 green 은 CI green 이 아니다 — 결과를 red 여도 그대로 적어라
  ```
- ★**못 재는 것**: **배포 후 실제 otterpebble 셸이 그 아티팩트를 먹는가** — 교차 repo다.
  ★있으면 잴 수 있는 것 = otterpebble 쪽 회차와의 조율(그쪽 리시버가 릴리스 자산을 **이름으로** curl 한다).
  ★**이 조각에서 아티팩트 이름을 바꾸면 그 소비자가 조용히 깨진다.**

### ★버려진 조각 — «만들지 마라»와 그 이유

| 후보 | 왜 안 만드나 |
|---|---|
| 「우리 크레이트를 하이픈으로 개명」 | git rename 탐지가 이미 한다(§1-1 · `R` **146건** 실측) |
| 「`compile_model.rs` 122줄 이식」 | 머지가 `AU` 로 데려온다(§3-1) — D 안의 `git add` 한 줄 |
| 「③ 오버레이 재적용」 | 재적용할 것이 없다 — `web/`·`functions/`·`migrations/`·`scripts/` 전건 온존(§3) |
| 「`wie_validate` 이식본을 repo 에 상주시킨다」 | 21줄이라 필요할 때 격리 워크트리에서 다시 만드는 편이 싸다(§4-1). ★상주시키면 **두 판본을 동기화**해야 한다 |

## 7. 범위 밖 — 이 계획이 «하지 않는» 것

- ★**upstream 에 PR 을 내지 않는다.** P4 는 별건이고 **외부 저장소 발신이라 운영자 확인 대상**이다.
  조각 B 가 「보낼 수 있다」로 분류해도 그것은 **목록**이지 발신 허가가 아니다.
- ★**실제 재정렬을 시작하지 않는다.** 이 회차의 머지 예행은 격리 워크트리에서 돌고 `--abort` + 제거됐다.
- ★**`Jun025/RustJava` 무접촉** — `[patch]` 핀 이탈은 2026-09-04 에 끝났다(verdict §9).
- ★**P1 의 남은 칸**(`+34` `current_class_loader` 비공개화 6곳 → 갈래 ⒜ `ba5797b`)은 **이 계획의 축이 아니다.**
  ★단 **조각 D 가 그것을 건드릴 수 있다** — upstream 은 이제 RustJava 를 **crates.io** 로 쓴다(verdict §8-5) ⇒
  머지에서 `Cargo.toml`/`Cargo.lock` 이 **UU** 로 서 있다(§3). ★**그 화해가 핀을 움직이면 하드닝 3축
  (`wie_jvm_support/src/hardening.rs`)이 조용히 떨어진다** — 그 모듈의 시험이 그것을 잡는다(Constraint · verdict §9-3).
  D 는 `RUST_MIN_STACK=4194304 cargo test --all` 로 그 시험이 **여전히 도는지**를 반드시 확인하라.
