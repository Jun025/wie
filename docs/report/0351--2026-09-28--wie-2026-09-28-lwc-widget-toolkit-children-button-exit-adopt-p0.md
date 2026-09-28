## [2026-09-28] lwc 다음 벽 2개 — Component.setBackground(I) · ShellComponent.setTitle(String) (wie-2026-09-28-lwc-widget-toolkit-children-button-exit-adopt-p0)

**무엇을**: `docs/report/0345` 짝 재측의 «후» 벽 두 개를 고쳤다. `Component` 에 `setBackground(I)V`·`getBackground()I` 를, `ShellComponent` 에 `setTitle(String)V`·`setTitle(Component)V`·`getTitle()` 를 넣었다. 타이틀은 sha 앞 12자로만 적는다.
**왜**: `ca7fa8ade8ad` 는 부팅 중 `setBackground(I)V`, `33f3e7669599` 는 14번째 키(`14_OK`)에서 `setTitle(String)V` 가 없어 멈췄다. 둘 다 정본 javadoc 의 공개 메서드다.
**사용자 영향**: `33f3e7669599` 가 **PASS** 로 바뀌었다(3/3 · 키 27/27 전부 전달). `ca7fa8ade8ad` 는 이 벽을 넘어 바로 옆 벽 `setForeground(I)V`(부팅) 에서 멈춘다. **화면 변화는 0 이다** — 이 층은 lwc 위젯을 그리지 않으므로 배경색·제목은 보관만 한다(아래).

### 무엇을 바꿨나 — 근거는 정본 클래스와 javadoc

`docs/reference/AromaWIPI_classes.zip` 는 본문 없는 API 전용 클래스라 이름·시그니처·필드 이름만 읽힌다. 동작은 같은 곳의 `AromaWIPI_javadoc.zip`(EUC-KR) 에서 읽었다.

| 고친 곳 | 근거 |
|---|---|
| `Component.setBackground(I)V` 가 값을 `bg` 필드에 보관 · `getBackground()I` 가 돌려준다 | javadoc 「배경색을 지정합니다 … 색은 0x00RRGGBB」 · `getBackground` 「배경색을 돌려 줍니다」 |
| `bg` 의 초기값 **-1** — `Component.<init>` 에서 넣는다 | javadoc 「지정을 해제 할경우 -1값으로 설정」. 위젯별 기본색은 API 전용 클래스에 없어 **지어내지 않았다** — -1 은 «지정 안 됨»이라는 javadoc 자신의 값이다 |
| `ShellComponent.setTitle(String)V` 가 `LabelComponent(String)` 을 만들어 `cmpTitle` 에 넣는다 | javadoc 의 `getTitle()` 은 「타이틀 **컴포넌트**」를 돌려준다 · 정본 `ShellComponent` 클래스가 `LabelComponent` 를 참조하고 필드 이름이 `cmpTitle` 이다 |
| `setTitle(Component)V` · `getTitle()Lorg/kwis/msp/lwc/Component;` | javadoc 「특정 레이블 컴포넌트로 타이틀을 출력」 · 「지정된 타이틀을 돌려줍니다」. 지정 전에는 null |

**화면 변화 0 — 보관만 한다.** `Component.paint` 는 no-op 그대로이고 lwc 위젯은 배치·그리기되지 않는다(`0345` 의 범위 그대로). 배경색은 `getBackground` 로만, 제목은 `getTitle` 로만 돌아온다. `setTitle(String)` 이 만든 `LabelComponent` 도 라벨 문자열을 보관하지 않는다(그 클래스의 기존 동작).

### 전/후 — 같은 target 에서 `origin/main` 을 먼저 빌드해 떼어 두고, 이 변경을 증분 빌드

- 전: `origin/main`(`b29d043e`) · 후: 그 위 이 PR. 둘 다 release `wie_validate`.
- 명령: `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(`0345`·`0329` 의 프로브 A 그대로) · 벽시계 200초 상한 · 6판씩 병렬.
- 호스트 부하 load1 **160~250**.

| sha12 | 전 (n) | 후 (n) |
|---|---|---|
| `ca7fa8ade8ad` | FAIL boot · `Component.setBackground(I)V` 없음 (3/3) · paints 2~3 | FAIL boot · 다음 벽 **`Component.setForeground(I)V` 없음** (3/3) · paints 2~3 |
| `33f3e7669599` | FAIL `14_OK` · `ShellComponent.setTitle(String)V` 없음 (3/3) · paints 116~121 | **PASS** (3/3) · rc 0 · 키 **27/27** · stop `max-ticks` · last_frame_content true · paints 276~317 |

`33f3e7669599` 의 PASS 는 `wie_validate` 자신의 말대로 «부팅 + 그림 + 입력 생존»이지 화면이 맞다는 뜻이 아니다(visual correctness NOT checked). `ca7fa8ade8ad` 의 다음 벽 `setForeground` 는 javadoc 에서 `setBackground` 바로 옆이지만 **기본값이 «컴포넌트마다 다르다»** 고 적혀 있어(-1 같은 «지정 안 됨» 값이 없다) 이번 회차에 끼우지 않고 후속으로 넘긴다.

### 검증
- 단위 시험 2개: `setBackground` 보관 + 초기 -1(셸 위 — `ContainerComponent.<init>` 을 건너뛰는 경로라 기본값이 `Component.<init>` 에서 와야 한다) · `setTitle(String)` 이 `LabelComponent` 로 보관되고 `setTitle(Component)` 가 바꾼다 · 지정 전 null.
- 변이 6개 전건 red · 원상 green: ⑴ `setBackground` 가 저장 안 함 ⑵ 초기 -1 제거 ⑶ `setTitle` 이 저장 안 함 ⑷ 문자열 제목이 `LabelComponent` 가 아님 ⑸ `setTitle(String)` 등록 제거 ⑹ `setBackground` 등록 제거.
- 게이트·러너 결과는 PR 본문과 회신에 있다.

### 게임 이름 유입
`node scripts/corpus-name-inflow.mjs` — BOUNDED **1** · SUFFIX-ATTACHED **0**. 1건은 `component.rs` 의 기존 주석 한 줄(`repaint(IIII)V` 위 · `origin/main` 에 이미 있다)이다. 도구는 «바뀐 파일의 본문 전체»를 보므로 그 파일을 고친 이 회차에 잡혔다. 이 회차가 더한 줄에서는 **0건**이다.

<!-- corpus-name-inflow v1 subjects=4 tree=c132bcf475cd67cb B=1/1 P=0/0 S=0/0 -->
