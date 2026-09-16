# 조각 B — ② 엔진 오버레이 **51건** 분류표 (폐기 / 재적용 / upstream 발신)

> 회차 티켓 `wie-p3-slice-b-classify-51-engine-overlays` · 채택 제안
> `docs/worklog/2026-09-16-p3-remaining-slices-plan.json#p1`.
> ★**제품 코드 0줄 — 이 회차는 «분류»다. 지우지도 보내지도 않는다.** 집행은 조각 C(삭제)·D(머지)·P4(발신).
> 계획 정본 = `docs/upstream-realign-p3-slices.md` §3·§6-B. 판정 정본 = `docs/upstream-realign-verdict.md`.
>
> ★**측정 트리 명시**: `origin/main` **`c77998ad`** ↔ `upstream/main` **`44fbf265`** ↔
> `merge-base` **`fa641a8a`**, 2026-09-16 07:2x KST(`git fetch` 직후). 워킹트리(`@live`) 값은 **하나도** 쓰지 않았다.
> 머지 예행은 `git worktree add --detach /tmp/wie-p3b-probe origin/main` 격리 체크아웃에서 돌고
> `--abort` + `worktree remove --force` 로 **제거됐다**(`git worktree list` 로 확인) — 이 저장소 diff 에 남지 않는다.

## 0. 결론 — 세 줄

1. ★**21/51 이 「폐기」다.** 우리 오버레이의 **41%** 가 upstream(또는 upstream 도 쓰는 `rustjava-runtime`)에
   **이미 있다**. 조각 C 가 그만큼을 머지 «전»에 지울 수 있다.
2. ★★**그중 7건은 «upstream 이 가졌다»가 아니라 «우리 자신의 의존이 이미 가졌다»** — `wie_wipi_java/src/classes/java/**`
   는 **2026-09-04 핀 이동(`5b84dd1`) 이후 이미 죽은 코드**다. §3-A.
3. ★**「51」은 «② 50 + ③f 1» 이다** — 계획 §3 표와 §1 분류 술어가 서로 어긋난다. §1.

## 1. ★수를 다시 쟀다 — 「51」의 정확한 구성

```sh
git fetch origin && git fetch upstream
git rev-parse origin/main upstream/main && git merge-base origin/main upstream/main
#   c77998ad3cac8aba42aee72fff0177a14688f1d0
#   44fbf2652c856af11c7140f838ad223cdebab641
#   fa641a8a1805980c078206937c8501b21118fc0c
git rev-list --left-right --count origin/main...upstream/main    # 539  1140
git worktree add --detach /tmp/wie-p3b-probe origin/main && cd /tmp/wie-p3b-probe
git merge --no-commit --no-ff upstream/main                       # rc=1
git status --porcelain > /tmp/wie-p3b-status.txt                  # ★경로로 좁히지 않는다(§3-2 의 그 함정)
awk '{print substr($0,1,2)}' /tmp/wie-p3b-status.txt | sort | uniq -c | sort -rn
#   167 'A '  · 146 'R ' · 104 'D ' · 35 UU · 19 AU · 17 UD · 4 'M ' · 3 AA
```

미해결 합계 = **35+19+17+3 = 74** — 계획(2026-09-16 04:1x)과 **같다**. 그 사이 `origin/main` 이
`d30cb903 → c77998ad` 로 4커밋 전진했으나 `git diff --name-only d30cb903 c77998ad` 는
`STATE.md`·`docs/report/0111…`·`docs/upstream-realign-p3-slices.md`·`docs/worklog/…json` **4개 문서뿐**이라
엔진 축이 움직일 수 없었다. `upstream/main` 은 **불변**(`44fbf265`).

갈래 분류는 계획 §1 의 술어를 그대로 쓰되 ★**하이픈 경로를 함께 매칭**했다 — 머지 상태의 `AU`/`UU`/`AA`
행은 upstream 개명(`wie_x` → `wie-x`)을 git 이 따라가서 **하이픈으로 온다**:

```sh
LC_ALL=C /usr/bin/grep -aE '^(UU|AU|UD|AA|DU|UA|DD) ' /tmp/wie-p3b-status.txt | awk '{
  st=substr($0,1,2); p=substr($0,4);
  if (p ~ /^wie[_-]lgt\// || p ~ /^wie[_-]featurephone\//) b="1";
  else if (p ~ /^wie[_-]cli\//)                            b="3f";
  else if (p ~ /^wie[_-]/ || p ~ /^test[_-]utils\//)        b="2";
  else                                                     b="3";
  print b"\t"st"\t"p }' | sort
```

| 갈래 | 미해결 | 내역 |
|---|---:|---|
| ① `wie_lgt` | 12 | UU 5 · UD 4 · AU 3 |
| ★**② 엔진 고유** | ★**50** | UU 21 · AU 16 · **UD 12** · AA 1 |
| ③ 스캐폴딩·설정 | 11 | UU 9 · AA 2 |
| ★**③f `wie_cli`** | ★**1** | UD 1 (`wie_cli/Cargo.toml`) |
| 합계 | **74** | |

★★**「② 51 · UD 13」은 «③f 1건을 ② 안에 접은 값»이다 — 계획 문서가 «두 절에서 서로 다르게» 갈랐다.**
§1 은 `③f = wie_cli/` 를 **별 갈래로 선언**하는데 §3 의 갈래별 표는 `① 12 · ② 51 · ③ 11 = 74` 로
**③f 칸이 없다**. 그 1건이 갈 곳은 ② 뿐이므로 51 이 나온다.
⇒ ★**총계 74 는 양쪽이 같고, 틀린 것은 «분해»뿐이다.** 이 문서는 **51행 전건**을 분류한다(② 50 + ③f 1) —
어느 읽기를 택하든 미분류가 0 이 되도록.

※`docs/upstream-realign-p3-slices.md:142,343` 의 `UD 13` 은 이 회차가 **`UD 12 + ③f UD 1`** 로 정정했다.
전수 계수(§6)로 닫았다.

## 2. ★분류 술어 — 먼저 적는다(인상 배제)

세 갈래는 **배타**이고 51행이 정확히 한 칸씩 든다.

| 갈래 | 술어 | 집행자 |
|---|---|---|
| **폐기** | 우리 델타가 더한 **메서드/필드/심볼**을 upstream(또는 upstream 도 쓰는 의존)이 **이미 갖는다** ⇒ 머지에서 우리 쪽을 버려도 기능이 안 준다 | 조각 **C**(삭제) |
| **발신** | 우리에게만 있고 **verdict §6-P4 의 IP 선 안쪽**(공개 WIPI/MIDP 스펙 API 스텁) ⇒ 머지는 지키고 **P4 가 upstream 에 제안** | 조각 **D**(보존) + **P4** |
| **재적용** | 우리에게만 있고 위 둘이 아니다 ⇒ 머지가 **반드시 살려야** 한다 | 조각 **D** |

기계 술어 2단:

```sh
# ⑴ 심볼 델타 — 우리가 base 위에 «더한» 심볼이 upstream 판본에 있는가
syms(){ LC_ALL=C /usr/bin/grep -aoE '\b(fn|struct|enum|trait|const|static|impl)[[:space:]]+[A-Za-z_][A-Za-z0-9_]*' \
        | awk '{print $1":"$2}' | sort -u; }
git show "fa641a8a:$U"    | syms > b; git show "origin/main:$U" | syms > o
git show "upstream/main:$H" | syms > u
comm -13 b o > added; comm -12 added u | wc -l; comm -23 added u | wc -l    # inUp / notUp
# ⑵ ★심볼은 «거칠다» — 판정은 Java 가시 표면을 «열어서» 확인한다
git show upstream/main:<H> | LC_ALL=C /usr/bin/grep -na 'JavaMethodProto::new\|JavaFieldProto::new'
```

★★**⑴만으로 닫지 않은 이유 — 양방향으로 틀린다.** `wie-jvm-support/src/runtime.rs` 는 `inUp=1 notUp=0`
(= 폐기처럼 보인다)인데 **실제 델타는 `hardening::harden` 호출**이고 upstream 은 `harden` **0건**이다.
반대로 `wie-ktf/.../wipi_c/context.rs` 는 `added=0`(= 볼 것 없음처럼 보인다)인데 **행 전체가 폐기**다.
⇒ ★**`added=0` 행 17건과 `inUp>0` 행 전건을 «diff 와 upstream 파일을 열어» 따로 봤다.**

## 3. ★★51행 분류표 — 빈 칸 0

> 「근거」는 전부 **오늘 이 트리에서 연 것**이다. 「폐기」 행은 acceptance 요구대로
> `git show upstream/main:<path>` 로 **해당 심볼이 실재함을 눈으로** 확인했다.

### A. 폐기 — **21건** (41%)

| # | 상태 | 경로(머지 상태 기준) | 우리 델타 | upstream 실측 근거 |
|---:|---|---|---|---|
| 1 | AA | `wie-wipi-java/…/lcdui/image_observer.rs` | 빈 interface proto | upstream 판은 **진상위집합** — 추상 `notify (Lorg/kwis/msp/lcdui/Image;I)V` 선언 + `PUBLIC\|INTERFACE\|ABSTRACT` |
| 2 | AU | `wie-wipi-java/src/classes/java.rs` | `pub mod io; pub mod lang;` | #5~#8 의 배선일 뿐 — 그 넷이 폐기면 함께 죽는다 |
| 3 | AU | `wie-wipi-java/src/classes/java/io.rs` | 모듈 배선 2줄 | 〃 |
| 4 | AU | `wie-wipi-java/src/classes/java/lang.rs` | 모듈 배선 2줄 | 〃 |
| 5 | AU | `…/java/io/interrupted_io_exception.rs` | `java/io/InterruptedIOException` 등록 | ★`rustjava@5b84dd1 java_runtime/src/loader.rs:26` 이 **이미 등록**한다 — §3-A |
| 6 | AU | `…/java/io/unsupported_encoding_exception.rs` | 〃 | ★`loader.rs:36` |
| 7 | AU | `…/java/lang/out_of_memory_error.rs` | 〃 | ★`loader.rs:80` |
| 8 | AU | `…/java/lang/virtual_machine_error.rs` | 〃 | ★`loader.rs:96` |
| 9 | UD | `wie_wipi_java/…/db/data_base.rs` | `getSizeAvailable`·`getRecordSize`·`selectRecord(I[BI)V` + 필드 `recordSize` | upstream `:87 getRecordSize` · `:88 getSizeAvailable` · `:61 selectRecord (I[BI)V` · `:93 JavaFieldProto recordSize` — **4/4** |
| 10 | UD | `wie_wipi_java/…/io/file.rs` | `openOutputStream`·`openDataOutputStream` | upstream proto 블록에 **둘 다** + 구현 `:279 open_output_stream` · `:305 open_data_output_stream` |
| 11 | UD | `wie_wipi_java/…/io/file_system.rs` | `isFile(Ljava/lang/String;I)Z`·`isDirectory(…;)Z`·`mkdir(…;)V` | upstream 이 **세 시그니처 전부** 보유(`isFile` 2종 · `isDirectory` 2종 · `mkdir` 2종) |
| 12 | UD | `wie_wipi_java/…/lcdui/card.rs` | `<init>(Z)V`·`move(II)V`·`resize(II)V`·`getDisplay` | upstream proto 목록에 **4/4** |
| 13 | UD | `wie_wipi_java/…/lcdui/display.rs` | `popCard`·`getBitsPerPixel`·`callSerially(Ljava/lang/Runnable;I)V` | upstream 에 `popCard` · `getBitsPerPixel ()I` · `callSerially` **2종** |
| 14 | UD | `wie_wipi_java/…/lcdui/font.rs` | `getBaselinePosition ()I` | upstream `:24` 동일 proto |
| 15 | UD | `wie_wipi_java/…/media/clip.rs` | `getVolume ()I` + 필드 `volume` | upstream `:48 getVolume` · `:54 JavaFieldProto volume` |
| 16 | UD | `wie_wipi_java/…/media/base_clip.rs` | `setBuffer([BI)Z` | upstream `:22` 동일 proto |
| 17 | UU | `wie-ktf/src/runtime/java/interface.rs` | `#[allow(clippy::double_must_use)]` | upstream `:163` 에 **같은 allow**(주석 문안만 다르다) |
| 18 | UU | `wie-ktf/…/jvm_support/array_class_instance.rs` | +33 핀의 `ClassInstance`/`ArrayClassInstance` 분리 + `identity`/`shallow_clone` | upstream 이 `fn identity`·`fn shallow_clone` **둘 다** 보유 — 같은 핀 이동을 저쪽도 했다 |
| 19 | UU | `wie-ktf/…/jvm_support/class_instance.rs` | `identity`/`shallow_clone` | 〃 |
| 20 | UU | `wie-ktf/src/runtime/wipi_c/context.rs` | `attach_thread(None)` · `current_class_loader` → `get_system_class_loader` | upstream `:77 attach_thread(None)` · `:94`·`:125 get_system_class_loader` · `current_class_loader` **0건** ⇒ **완전 수렴** |
| 21 | UU | `wie-midp/…/lcdui/image.rs` | 같은 loader 교체 | upstream `:116 get_system_class_loader` |

### B. 재적용 — **24건** (머지가 «반드시» 살려야 한다)

| # | 상태 | 경로 | 우리 델타 | upstream 부재 근거 |
|---:|---|---|---|---|
| 22 | AU | `wie-jvm-support/src/hardening.rs` | 널 가드 3축(338줄) | ★**upstream `harden` 0건.** AGENTS.md 사건대장이 「가드가 조용히 안 붙는 것이 이 하드닝을 처음 잃은 방식」이라고 못박은 축이다 — ★**지우지 마라** |
| 23 | AU | `wie-ktf/tests/test_key_reach.rs` | 키 입력이 게스트에 닿는가 | upstream 에 그 경로 부재 |
| 24 | AU | `wie-ktf/tests/test_resource_reach.rs` | 번들 리소스 도달 | 〃 |
| 25 | UD | `wie_ktf/src/runtime/java/jvm_support.rs` | `KtfClassLoader.<init>` 실패를 `WieError` 로 전파(호스트 abort 방지) | upstream 은 **여전히 `.unwrap()`** |
| 26 | UD | `wie_wipi_java/…/lcdui.rs` | 모듈 목록 — 우리 `input_method_listener` ↔ upstream `jlet_wrapper` | ★**합집합이 필요**(어느 쪽을 통째로 고르면 다른 쪽이 조용히 사라진다) |
| 27 | UD | `wie_wipi_java/…/media/player.rs` | `play_clip`/`stop_clip` 의 `clip.is_null()` 조기 반환 | upstream 의 `:104`·`:118` 가드는 **`player` 에 걸린 다른 값**이다 ⇒ 3-way 화해이지 붙여넣기가 아니다 |
| 28 | UD | `wie_wipi_java/…/net/wie/card_canvas.rs` | clet 래퍼 판정을 **내부 이름**으로(바이너리 이름 왕복 제거) | ★upstream 에 `Clet` 문자열 **0건** — 그 특례 자체가 없다. ★**조각 A 의 `CletWrapperCard.paint` FAIL 과 같은 구역**이다 |
| 29 | UU | `test-utils/src/lib.rs` | `TestScreen` 재수출 | #30 을 따른다 |
| 30 | UU | `test-utils/src/platform.rs` | paint 카운터 + redraw 플래그 | ★양쪽 `TestScreen` 이 **독립 진화**했다(upstream 은 `width`/`height` + `resize`) ⇒ **둘 다 필요** |
| 31 | UU | `wie-backend/src/executor.rs` | `HashMap` → `BTreeMap`(스폰 순서 결정성) | upstream **`HashMap` 그대로**(`:8,19,20,72,73,143-145`) |
| 32 | UU | `wie-j2me/Cargo.toml` | `[dev-dependencies] test_utils` | upstream `test-utils` **0건** |
| 33 | UU | `wie-jvm-support/src/jvm_implementation.rs` | `ClassDefinitionError` → JLS 예외 매핑 | upstream `ClassDefinitionError` **0건** |
| 34 | UU | `wie-jvm-support/src/lib.rs` | `PATH_SEPARATOR` + 그 시험 | upstream **0건** |
| 35 | UU | `wie-jvm-support/src/runtime.rs` | `hardening::harden(&mut proto)` 훅 | ★`fn exit` **반쪽은 수렴**(upstream `:182` 이 같은 본문) — 남는 것은 harden 훅이고 upstream **0건** |
| 36 | UU | `wie-ktf/…/net/wie/ktf_class_loader.rs` | panic → `net/wie/WieError` 예외 | upstream `net/wie/WieError` **0건** |
| 37 | UU | `wie-midp/src/classes/net/wie/launcher.rs` | MIDlet 을 **시스템** 클래스로더로 적재 | upstream `:45` 는 **여전히 `jvm.new_class(&main_class, …)`** |
| 38 | UU | `wie-wipi-java/src/lib.rs` | `get_protos()` **46** ↔ upstream **42** | ★우리만 10 · upstream 만 6 ⇒ **합집합 화해**. ★#5~#8 이 폐기되면 46 → 42 로 준다(연동) |
| 39 | UD | `wie_backend/src/canvas.rs` | `decode_image` 의 `data.len() >= 4` 가드 + 단위시험 **8개** | upstream `data.len() >= 4` **0건** · 시험 이름 **교집합 0**(§4-2) |
| 40 | UU | `wie-wipi-c/src/api/graphics.rs` | `read_framebuffer_or_null` 널 가드(5곳) | upstream **0건**. ★단 `get_context`(MC_grpGetContext) 반쪽은 **수렴**했다(upstream `fn get_context` 1건) |
| 41 | UU | `wie-wipi-c/src/api/database.rs` | `get_number_of_records` 외 | upstream **0건** |
| 42 | UU | `wie-wipi-c/src/api/kernel.rs` | `alloc_or_null` 외 7심볼 | upstream **0건** |
| 43 | UU | `wie-midp/…/lcdui/font.rs` | `getBaselinePosition` | upstream **0건**(★WIPI 쪽 #14 와 혼동 금지 — 이쪽은 MIDP 다) |
| 44 | UU | `wie-wipi-java/…/lwc/component.rs` | `repaint` | upstream **0건** |
| 45 | UD | `wie_cli/Cargo.toml` (갈래 **③f**) | 우리 매니페스트 | ★upstream 에 `wie-cli` **부재** — 네이티브 호스트가 루트 패키지로 이사했다. ★**§3-2 ①의 그 행이다**: 「해소」만으로 부족하고 `[[bin]] wie_validate` 를 살리는 **코드 수정**이 든다 |

### C. upstream 발신(P4 후보) — **6건**

★**머지에서는 재적용과 같다**(전부 살린다). 다른 것은 「P4 가 추가로 upstream 에 제안한다」뿐이다.
★**이 회차는 목록까지다 — 발신하지 않는다.** 실제 발신은 **외부 저장소 발신이라 운영자 확인 대상**이다.

| # | 상태 | 경로 | 클래스 | IP 선 |
|---:|---|---|---|---|
| 46 | AU | `…/lcdui/input_method_listener.rs` | `org.kwis.msp.lcdui.InputMethodListener` | 선 안(공개 WIPI API 스텁) |
| 47 | AU | `…/lwc/action_listener.rs` | `org.kwis.msp.lwc.ActionListener` | 선 안 |
| 48 | AU | `…/lwc/form_component.rs` | `org.kwis.msp.lwc.FormComponent` | 선 안 |
| 49 | AU | `…/lwc/grab_key_listener.rs` | `org.kwis.msp.lwc.GrabKeyListener` | 선 안 |
| 50 | AU | `…/lwc/label_component.rs` | `org.kwis.msp.lwc.LabelComponent` | 선 안 |
| 51 | AU | `…/media/media_unsupported_exception.rs` | `org.kwis.msp.media.MediaUnsupportedException` | 선 안 |

**계수**: 폐기 **21** + 재적용 **24** + 발신 **6** = **51** · ★**미분류 0**.

### 3-A. ★★폐기 #5~#8 은 「upstream 이 가졌다」가 아니라 「우리 의존이 이미 가졌다」

이 넷은 upstream `wie-wipi-java` 에 **없다**(파일도, 등록도). 그런데도 폐기인 이유:

```sh
$ LC_ALL=C /usr/bin/grep -naE 'OutOfMemoryError|VirtualMachineError|InterruptedIOException|UnsupportedEncodingException' \
    ~/.cargo/git/checkouts/rustjava-b78e8f47b488d719/5b84dd1/java_runtime/src/loader.rs
26:        crate::classes::java::io::InterruptedIOException::as_proto(),
36:        crate::classes::java::io::UnsupportedEncodingException::as_proto(),
80:        crate::classes::java::lang::OutOfMemoryError::as_proto(),
96:        crate::classes::java::lang::VirtualMachineError::as_proto(),
```

그리고 `wie_jvm_support/src/lib.rs` 가 클래스패스를 **`RT_RUSTJAR : WIE_RUSTJAR : <jar>`** 순서로 세운다
(`java.class.path` · `Jvm::new(java_runtime::get_bootstrap_class_loader(…))`).
⇒ ★**java_runtime 의 정의가 «먼저» 걸리므로 우리 사본은 오늘 이미 그림자에 있다.**

★**왜 예전엔 옳았나 — 시간 순서가 답이다**(다음 사람이 「그럼 왜 넣었나」로 되밟지 않도록):

| 언제 | 무슨 일 |
|---|---|
| 2026-07-02 | `5603a7f9 feat(wipi-java): register KTF init-preload classes — recover 5 KTF titles`. 당시 핀은 **Jun025/RustJava 포크**(2026-07-07 동결본 계열) |
| 2026-09-04 | `1762a32c` 가 핀을 **`dlunch/RustJava@5b84dd1`(+33)** 로 옮겼다 |

⇒ ★**파일 머리주석의 「Not provided by the bundled java_runtime」는 2026-07-02 에는 참이었고 2026-09-04 부로 낡았다.**
★**이것은 「upstream 재정렬」과 무관하게 «지금 우리 base 에서» 참인 폐기**라, 조각 C 가 4게이트로 바로 검증할 수 있다.

★★**그래도 «자동 폐기»로 읽지 마라 — 조각 C 가 반드시 확인할 것**: 원 커밋이 주장한 **5 KTF 타이틀**은
**292 코퍼스가 이 머신에 없어 재현 불가**다(§5). C 는 ⑴이 7파일을 지운 뒤 **4게이트 + 5픽스처**가 살아 있는지
⑵`grep` 으로 **우리 안의 호출자 0건**(계획 §6-C 의 완화책)을 보이고, 둘 다 만족할 때만 착지시켜라.

### 3-B. ★verdict §6-P4 의 「선 안쪽 10종」을 오늘 값으로 재측 — **10 → 6**

verdict §6-P4 와 `docs/report/0016` 이 열거한 10종 중 ★**4종이 이 회차의 폐기로 빠진다**:

| 클래스 | 2026-08-27 verdict | ★오늘 |
|---|---|---|
| `InterruptedIOException`·`UnsupportedEncodingException`·`OutOfMemoryError`·`VirtualMachineError` | 발신 후보 | ★**폐기** — `rustjava@5b84dd1` 이 등록한다(§3-A) ⇒ **보내 봐야 중복이다** |
| `InputMethodListener`·`ActionListener`·`FormComponent`·`GrabKeyListener`·`LabelComponent`·`MediaUnsupportedException` | 발신 후보 | ★**발신 유지**(#46~#51) |

### 3-C. ★`canvas.rs` 「+149줄 · 전부 단위테스트 9개」도 재측 — **두 곳이 틀렸다**

```sh
$ git diff --numstat fa641a8a origin/main -- wie_backend/src/canvas.rs
149     6       wie_backend/src/canvas.rs
$ git show origin/main:wie_backend/src/canvas.rs   | grep -c '#\[test\]'   # 9
$ git show fa641a8a:wie_backend/src/canvas.rs      | grep -c '#\[test\]'   # 1
$ git show upstream/main:wie-backend/src/canvas.rs | grep -c '#\[test\]'   # 28
```

- ★**`+149` 는 오늘도 맞다.**
- ★**「9개」는 «우리 파일의 총 시험 수»이고 «더한 수»가 아니다** — base 가 이미 1개(`test_canvas`)를 갖고 있으므로
  **더한 것은 8개**다.
- ★★**「전부 단위테스트이고 구현이 아니다」는 «거짓»이다.** 구현 hunk 가 **둘** 있다:
  ⑴`Rgb332Pixel` 의 `u8` 오버플로 교정(`(color.r as u16 * 7 + 127) / 255`) — ★**upstream 에 이미 있다**(2건) ⇒ 폐기 몫
  ⑵`decode_image` 의 `data.len() >= 4` 경계 가드 — ★**upstream 0건** ⇒ 재적용 몫(#39)
- 시험 이름 교집합은 **`test_canvas` 1개(= base 것)뿐**이다: 우리 8개는 `decode_image`·픽셀 왕복 축 ·
  upstream 28개는 draw/clip/arc 축. ⇒ ★**양쪽이 겹치지 않는다** — 발신했을 때 값을 한다.

## 4. ★이 표가 조각 C·D 에 «수»로 무엇을 약속하는가

- 조각 **C** 는 **21행**을 지운다 ⇒ 미해결 **74 → 53** 이 기대값이다.
  ★**그것을 «세어서» 닫아라** — 계획 §6-C 대로 삭제 전후로 머지 예행을 다시 돌려 `74 → N` 을 적는다.
  줄지 않았으면 그 PR 은 목적을 달성하지 못한 것이다.
- ★**주의 — 21 이 그대로 21 만큼 줄지 않을 수 있다.** 삭제가 `UD`/`AU` 를 지우면 그 행은 사라지지만,
  `UU` 행(#17~#21)은 **파일이 남고 델타만 사라지므로** 머지가 그 파일을 자동 해소할 수도, 여전히 `UU` 로 설 수도 있다.
  ⇒ ★**기대값은 「53 ≤ N < 74」이지 「N = 53」이 아니다.** 등식으로 적어 두고 실패로 읽지 마라.
- 조각 **D** 는 **30행**(재적용 24 + 발신 6)을 **살려야** 한다. 그중 «해소 안에 코드 수정이 든» 것:
  **#45**(`wie_cli` 매니페스트·bin 타깃) · **#26**(모듈 목록 합집합) · **#38**(`get_protos` 합집합) ·
  **#30**(`TestScreen` 양측 독립 진화) · **#27**(널 가드 3-way).

## 5. ★못 재는 것 — 「무엇이 있으면 잴 수 있는가」와 함께

| 못 재는 것 | 있으면 잴 수 있는 것 |
|---|---|
| ★**분류가 «동작»을 보증하지 않는다.** 같은 이름의 메서드가 다르게 동작할 수 있다 — 특히 폐기 #9~#16 은 **시그니처 일치**까지만 봤고 **본문 의미**는 비교하지 않았다 | 292 코퍼스(`game_lab/working/`), 또는 그 심볼을 실제로 부르는 픽스처. ⇒ ★**그래서 이 표는 «폐기 후보»이고 집행은 조각 C 가 4게이트를 지고 한다** |
| ★**upstream 의 `rustjava-runtime ^0.1.1` 내용**(§3-A 의 폐기가 upstream 쪽에서도 성립하는가) — 그 크레이트는 **crates.io** 판이고 이 머신 레지스트리에 **없다**(`~/.cargo/registry/src/*/java_runtime-*` 0건) | `cargo fetch` 가 가능한 회차, 또는 조각 D 의 머지 트리에서 실제 빌드. ※§3-A 의 폐기 판정은 **우리 핀 `5b84dd1` 실측만으로도 성립**한다 — upstream 축은 «추가 확인»이지 «전제»가 아니다 |
| ★**폐기 #5~#8 이 주장하는 「5 KTF 타이틀」** 이 실제로 살아 있는가 | 코퍼스. 없으면 조각 C 의 4게이트 + 5픽스처 + 호출자 0건 grep 이 대체 증거다 |
| ★`skt` 축은 이 표에 **한 행도 없다** — ② 미해결에 `wie_skt/` 가 0건이다 | 그대로 사실이다(오류 아님). 단 커밋된 `skt` 픽스처가 **0** 이라 조각 D 에서 skt 는 **신호 0** 이다(계획 §5) |
| ★**① `wie_lgt` 12건과 ③ 11건은 이 표의 범위가 아니다** | ①은 조각 D 의 머지가 `D`/`UD` 로 진다(계획 §6-C 가 명시로 C 에서 뺐다) · ③은 설정 화해로 D 몫 |

## 6. ★정정 — 전수 계수로 닫았다

`UD 13` 리터럴(= ③f 1건을 ② 에 접은 분해):

```sh
$ git ls-files -z '*.md' | xargs -0 ~/orchestrator/bin/ledger-grep -nF 'UD 13'
docs/upstream-realign-p3-slices.md:142
docs/upstream-realign-p3-slices.md:343
```

⇒ **2곳 전건**을 `UD 12 + ③f UD 1` 로 정정했다. ★**총계 `② 51`·`74` 는 «건드리지 않았다»** — 그 수들은
「② + ③f」 읽기에서 참이고, 이 문서 §1 이 두 읽기를 모두 적어 둔다.
★**맨 `grep` 을 쓰지 않았다** — 셸 스냅샷의 `ugrep` 은 `-G`/`-I` 강제로 조용히 0건을 답한다(헌장).
