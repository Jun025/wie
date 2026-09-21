## [2026-09-21] 배틀몬스터의 첫 벽 `Player.resume(Clip)Z` — 그리고 같은 계급 10개를 닫았다 (wie-battlemonster-aot-java-first-wall-player-resume-clip)

### 무엇을

LGT AOT-Java 타이틀이 「미해결 메서드/필드」로 부팅 첫 tick 에서 죽던 벽 **10개**를 닫았다.
전부 «그 게임이 실제로 부르다 죽은» 서명이고, 각 서명은 게임 자신의 `binary.mod` 참조 풀에서
교차 확인했다. 파일 5개 · 본문 +182 / −10.

| 파일 | 추가 |
|---|---|
| `org/kwis/msp/media/player.rs` | `resume(Lorg/kwis/msp/media/Clip;)Z` + 시험 2 |
| `org/kwis/msp/lcdui/input_method_handler.rs` | `getCurrentMode()I` · `hideSymbolCard()V` · 필드 `mode I` |
| `org/kwis/msp/lwc/component.rs` | `getWidth()I` · `serviceRepaints()V` · `hasFocus()Z` · `repaint(IIII)V` |
| `org/kwis/msp/lwc/text_component.rs` | 필드 `iMode I` · `maxLength I` · `m_td [C` |
| `org/kwis/msp/lwc/text_box_component.rs` | `<init>(Ljava/lang/String;II)V` |

### 왜 — 서명을 «게임에게 물어서» 골랐다

`binary.mod` 의 NUL 구분 문자열 풀은 클래스별 멤버 이름을 그 클래스 이름 «앞»에 모아 둔다.
배틀몬스터의 `org/kwis/msp/media/Player` 블록 전문:

```
933656 (Lorg/kwis/msp/media/Clip;Z)Z | 933686 play | 933691 resume
933698 (Lorg/kwis/msp/media/Clip;)Z  | 933727 stop | 933732 org/kwis/msp/media/Player
```

이 풀 판독을 `broken/lgt` 46종 전건에 돌렸고, 분류 축(`compile_model.rs` 의 `BL`+`.word 0x64` 규약)을
**독립 구현으로 재현**해 **aot-java 24 · clet 22** — 그 파일 머리주석의 수와 정확히 일치했다.

### 측정 — 벽이 어디로 옮겨 갔나 (`--timeout 30~40` · 전건 실행)

| 타이틀 | 닫은 벽 (전) | 멈춘 자리 (후) | ticks 전→후 |
|---|---|---|---|
| **배틀몬스터** | `Player.resume(Lorg/kwis/msp/media/Clip;)Z` | `net.wie.WieError: Invalid memory access; address: 0` | 1 → **2** |
| 서든어택포켓 | `InputMethodHandler.getCurrentMode()I` → `TextComponent.maxLengthI` | 같은 메모리 접근 위반 (`net/wie/LgtClassLoader.findClass` 프레임) | 1 → 1 |
| 훼밀리마트타이쿤 | `InputMethodHandler.hideSymbolCard()V` → `TextComponent.m_td[C` | 같은 메모리 접근 위반 | 1 → **2** |
| 학교가는길 | `ShellComponent.serviceRepaints()V` → `ShellComponent.repaint(IIII)V` | 같은 메모리 접근 위반 | 2 → **3** |
| 레전드오브마스터 | `TextComponent.iModeI` | 같은 메모리 접근 위반 | 2 → 2 |
| 월드장기체스 | `Component.hasFocus()Z` | `java.lang.NoClassDefFoundError: wec/SYSTheme` | 1 → 1 |
| 붕어빵타이쿤3 | `TextBoxComponent.<init>(Ljava/lang/String;II)V` | `NoClassDefFoundError: org/kwis/msp/lwc/DialogComponent` | 1 → 1 |

★**7종이 전부 «다른 계급»으로 옮겨 갔다** — 남은 벽은 메모리 접근 위반 5 · 미정의 클래스 2 이고,
티켓이 범위 밖으로 명시한 바로 그 둘이다. ⇒ **여기서 멈췄다.**

★**`paints` 는 7종 전건 0 → 0.** 이 회차는 «렌더를 켜지 않았다». 부풀리지 않는다.

### 회귀 — 없다 (`broken/lgt` 46종 전건 before/after)

`PASS → non-PASS` **0건**. 판정+벽이 그대로인 것 33/46, 바뀐 13건은 전부 위 표의 대상(이름 중복 포함)
이거나 아래 잡음이다.

★**clet 2종이 판정을 흔들었는데 내 변경이 아니다 — 저장해 둔 «변경 전» 바이너리로 재현했다**:
`슈퍼액션히어로3` 은 변경 전 바이너리로도 after 쪽 문면(`no frame rendered`)이 **3/3**,
`붉은보석` 은 변경 전 바이너리로 **FAIL 3/3**(before 스윕에선 PASS 였다). loadavg 120 대의 변동이다.
※구조적으로 「clet 은 Java 프로토를 안 쓴다」고 쓰려다 **틀린 것을 확인했다** — `clet_wrapper.rs` 가
`wie_wipi_java` 를 참조하므로 프로토는 전 LGT 앱에 등록된다. 근거는 구조가 아니라 위 재현이다.

### 판단 — 넣지 «않은» 것과 그 근거

★**`Player.pause(Clip)Z` 를 넣지 않았다.** 배틀몬스터의 `pause` 토큰은 **1건**뿐이고 그것은
`pauseApp`(Jlet 생명주기 · 오프셋 935287)으로, `Player` 블록(933656~933732)과 다른 자리다.
`Player` 를 참조하는 **LGT 23종 전건**을 훑으면 `play`+`stop` 은 전건, `resume` 은 **배틀몬스터 단 1종**,
`pause` 는 **0종**이다. ⇒ 대칭 결손이지만 부르는 타이틀이 없다.

★**`Component.getWidth()I` 만은 «관측된 벽»이 아니다** — 배틀몬스터·학교가는길의 `Component` 참조
블록에 `getWidth` 가 실재해서 넣었고, 실제로 그것이 막고 있는 것을 보지는 못했다. 나머지 9개는
전부 런타임이 문면으로 지목한 것이다. 이 한 건만 근거의 등급이 다르다.

### 어떻게 구현했나 — 위임 대상을 실측으로 골랐다

`resume` 은 `javax/microedition/media/Player::start()V`(인터페이스)로 위임한다.
같은 파일의 두 선례가 **서로 다른 대상**을 쓰는 이유가 여기서 갈린다: `play_clip` 이
`net/wie/SmafPlayer::start(Z)V`(구상 클래스)를 쓰는 것은 **반복 인자를 받는 오버로드가 인터페이스에
없기** 때문이고(`wie-midp/.../media/player.rs` 실측), `stop_clip` 이 인터페이스를 쓰는 것은 거기
`stop()V` 가 선언돼 있기 때문이다. `resume` 은 반복 인자가 없으므로 후자 규칙에 걸린다.
의미론도 그쪽이다 — MIDP 에는 pause/resume 이 없고 `stop()` 이 media time 을 유지한 채 멈추며
`start()` 가 거기서 잇는다.

`TextBoxComponent.<init>(String;II)V` 의 두 int 순서는 MIDP `TextBox(title,text,maxSize,constraints)`
관례를 따라 (text, maxLength, constraint) 로 읽었다. ★**오늘은 관측 불가다** — 두 도착지
(`setMaxLength`·기존 2인자 `<init>`)가 모두 스텁이라 어느 쪽으로 읽어도 결과가 같다. 그대로 적어 둔다.

`Component.hasFocus()Z` 는 필드를 두지 «않고» `false` 를 낸다. `setFocus`/`focusNotify` 가 둘 다
상태를 남기지 않는 스텁이라, 플래그를 저장하면 이 계층이 주지도 않은 포커스를 주장하게 된다.

### 시험 — 「있다」가 아니라 「지킨다」를 확인했다

`player.rs` 에 시험 2개를 그 파일의 `mod test` 관용구로 추가했다(위임 성공 / 백킹 플레이어 없을 때 false).
★**변이 시험**: 프로토 등재 1줄을 지우면 새 시험 **2건이 FAILED**, 기존 2건은 **ok** —
즉 기존 시험만으로는 이 결손이 잡히지 않았다는 뜻이다.

### 게이트

`cargo fmt --all -- --check` OK · `cargo clippy --all -D warnings` OK ·
`cargo clippy --target wasm32-unknown-unknown -D warnings` OK ·
`RUST_MIN_STACK=4194304 cargo test --all` **0 failed**(결과 줄 22개) · `cargo +beta clippy --all -D warnings` OK.

★**wasm 게이트 한 가지를 적어 둔다**: 문서대로의 맨 명령은 기본 멤버만 봐서 이 변경에 대해
**0.6초 무동작**이었다(`wie-wipi-java/src/lib.rs` 를 touch 해도 그대로). 그래서 실제로 wasm 으로
빌드되는 크레이트에 **상위집합**으로 따로 돌렸다 — `-p wie-wipi-java -p wie-lgt -p wie-web`, 14m43s, 통과.

### 사용자 영향

배틀몬스터는 **아직 화면이 뜨지 않는다**(`paints` 0). 이 회차가 판 것은 그 앞의 벽 하나이고,
다음 벽은 계급이 달라 별 티켓 몫이다.

★**그러나 «지금 당장 플레이 가능한» aot-java 3종을 실측으로 확정했다** — 아래 별 절.

### ★다른 repo 가 인용할 사실 — aot-java 3종은 이미 렌더한다

`apps/featurephone` 셸이 `lgt_compile_model() === "aot-java"` 를 일괄 차단하는데,
**그 차단 안에 이미 돌아가는 타이틀이 셋 있다.** 단독 실행 실측(스크린샷 육안 확인):

| 타이틀 | 판정 | paints | distinct_colors | 화면 |
|---|---|---|---|---|
| `메이플스토리2007.zip` | PASS | 16 | 63 (nondominant 8.80%) | ★**균일색 아님** — 한글 이용안내 + 「아무키나 누르세요!!」 |
| `현영맞고2006.zip` | PASS | 50 | 62 (nondominant 5.80%) | ★**균일색 아님** — 같은 형태의 안내 화면 |
| `놈3.zip` | PASS **6/6** | 3~31 | 2 | ★**균일색 아님** — 흰 바탕 + 붉은 한글 텍스트 |

★**셋 다 «아무키나 누르세요» 프롬프트에서 입력을 기다리는 상태**다. 즉 렌더만 되는 것이 아니라
**대화형 진입점에 도달**해 있다.

★**`놈3` 의 `distinct_colors=2` 를 「검은 화면」으로 읽지 마라** — 흰 바탕/붉은 글씨라 2색인 것이고,
스크린샷에는 글이 또렷하다. ★그리고 `놈3` 은 loadavg 120 에서 **1회 FAIL(paints 0)** 이 나왔는데,
AGENTS.md 의 4단계대로 재실행하니 현 바이너리 **6/6 PASS** · 변경 전 바이너리 **4/4 PASS** 였다.
**기아된 실행이지 회귀가 아니다.**

### 코퍼스 — 같은 파일이 두 이름으로 두 번 세어지고 있다

`배틀몬스터.zip` 과 `lgt 배틀몬스터.zip` 은 **sha256 동일**(`a30bbe00…`). 같은 형태가 최소 6쌍
(배틀몬스터 · 서든어택포켓 · 월드장기체스 · 레전드오브마스터 · 당신은골프왕 · 턴 · 붉은보석 포함)이라
★**「aot-java 24종」의 고유 타이틀은 그보다 적다.** 파일은 지우지 않았다(이 회차 소관 아님).

### 낡은 리포트 2건 재생성 (`game_lab/` 은 git-ignored · 커밋 0)

같은 sha256 이 **서로 다른 실패**를 보고하고 있었다 —
`배틀몬스터.json` 은 트리에 **0건인 문자열**(`Unimplemented: LGT Java apps are not implemented yet`,
`7f9ab2f7` 에서 소멸)을, `lgt 배틀몬스터.json` 은 `ticks 50000000 / paints 1` 의 검은 화면을.
둘 다 현 코드로 재생성했고 이제 **동일 판정**(`ticks 2 · paints 0 · 메모리 접근 위반`)이다.
낡은 산출물 23개(json·log·png·shots 18)는 `game_lab/reports-2026-09-21-stale-backup/` 에 백업했고,
갱신되지 않는 낡은 스크린샷 `lgt 배틀몬스터.png` 는 새 판정과 어긋나므로 제거했다.
★`scripts/game-lab-recensus.sh` 는 쓰지 않았다 — 그 도구는 `--out game_lab/reports/` 를 exit 3 으로
거부하고(7월 기준선 보호), 이 회차가 만진 것은 그 1312개 중 **2개 타이틀분뿐**이다.

### 코퍼스 파일명 유입 — 0 이 아니다 (도구 실행값)

`node scripts/corpus-name-inflow.mjs` 실행값: ★**유입 BOUNDED 29쌍(52회)** ·
★**판단 필요 SUFFIX-ATTACHED 9쌍(16회)**. 두 수를 함께 적는다 — 「0건」이 아니다.
※이 절 자체가 제목을 한 번 언급하므로 회수는 51/15 → 52/16 으로 움직였다. 쌍 수는 29/9 로 불변이다.

★**의도된 유입이다**: 이 티켓은 «특정 타이틀이 화면에 뜨는 것»을 좇으므로 어느 타이틀의 어느 벽인지를
이름으로 적지 않으면 회신도 후속 티켓도 성립하지 않는다. 코드 쪽 유입은 주석 1건뿐이고
(`text_component.rs` — 어느 게임이 그 필드를 읽는지의 근거), **게임 바이트는 0**이다.
SUFFIX-ATTACHED 9쌍은 기계가 못 가르는 바구니라 눈으로 봤다 — 전부 «조사가 붙은 진짜 언급» 또는
«더 긴 실제 제목»(`슈퍼액션히어로3`)이고, 잘못 붙은 것은 없다.

### 한계

- 남은 두 벽(메모리 접근 위반 · 미정의 클래스)은 **손대지 않았다**. 원인 조사도 하지 않았다.
- `m_td [C` 는 필드만 만들었고 **null 이다**. 버퍼를 누가 소유해야 하는지는 정하지 않았다.
- `크로이센`(양쪽 이름)은 검증기 자체가 **스택 오버플로**로 죽어 JSON 을 못 낸다. 기존 상태이고 범위 밖이다.

### 연번을 0207 → 0208 로 옮겼다 (착지한 쪽은 건드리지 않는다)

처음 `--next-serial` 이 **0207** 을 줬는데, 그 답은 **내 작업 트리 기준**이었고 그 트리는
`e8ff2891` 에 머물러 있었다. 그 사이 `origin/main` 은 `c2ac05d2` 로 나아가 있었고 거기에
**같은 번호를 쓴 ktf 회차가 이미 착지**해 있었다. ⇒ CI 의 `contract`(필수 검사)가
`check-docs-report-serial --selftest` 의 「실 저장소가 지금 깨끗하다」 케이스로 **정확히 잡았다**
(27케이스 중 1건 불일치 · 16초).

처분은 AGENTS.md 규칙 그대로다 — ★**착지한 파일은 다시 번호 매기지 않고, 아직 착지하지 않은 쪽이 옮긴다.**
base 를 병합한 뒤 도구에 다시 물어 **0208** 로 이동했다.

★**교훈은 「도구를 안 썼다」가 아니라 「썼는데도 났다」이다** — `--next-serial` 은 열린 PR 은 조회하지만
**내 트리가 낡은 것은 모른다**. ⇒ 물어보기 «전»에 base 를 당겨라.

<!-- corpus-name-inflow v1 subjects=7 tree=a53c6315b78df8a3 B=52/29 P=0/0 S=16/9 -->
