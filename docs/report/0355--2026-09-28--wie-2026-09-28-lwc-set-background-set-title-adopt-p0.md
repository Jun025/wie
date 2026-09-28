## [2026-09-28] lwc 다음 벽 — Component.setForeground(I) · getForeground() (wie-2026-09-28-lwc-set-background-set-title-adopt-p0)

**무엇을**: `docs/report/0352` 의 «후» 벽. `Component` 에 `setForeground(I)V`·`getForeground()I` 를 넣었다. 타이틀은 sha 앞 12자로만 적는다.
**왜**: `ca7fa8ade8ad` 가 부팅 중 `setBackground` 바로 다음 줄의 `setForeground(I)V` 가 없어 멈췄다(0352 «후» 3/3). 정본 javadoc 의 공개 메서드이고 `getForeground()I` 짝이 있다.
**사용자 영향**: `ca7fa8ade8ad` 가 **부팅 예외 없이 넘어간다**(3/3) — 그러나 여전히 FAIL(검은 화면)이다. 다음 벽은 다른 스레드의 `FormComponent.<init>(Z)V` 없음(3/3). **화면 변화 0** — 전경색도 보관만 한다(이 층은 lwc 위젯을 그리지 않는다, 0345·0352 범위 그대로).

### 무엇을 바꿨나 — 근거는 정본 javadoc(`docs/reference/AromaWIPI_javadoc.zip`, EUC-KR)

| 고친 곳 | 근거 |
|---|---|
| `setForeground(I)V` 가 `fg` 필드에 보관 · `getForeground()I` 가 돌려준다 | javadoc 「전경색을 지정합니다」 · `getForeground` 「전경생을 돌려줍니다」 · See Also 로 서로 짝 |
| `fg` 초기값 **-1** — `Component.<init>` 에서 넣는다 | 아래 |

**초기값 -1 의 근거 — 그리고 한계.** javadoc 은 전경색 기본값을 「각 컴포넌트에 따라 다르게 설정 됩니다」라고만 적고 값을 하나도 주지 않는다. 위젯별 값은 API 전용 클래스(`AromaWIPI_classes.zip`, 본문 없음)에도 없다. 그래서 색을 **지어내지 않고**, 이 API 가 «색 없음»으로 문서화한 유일한 값 **-1** 을 `bg` 와 같은 규칙으로 쓴다(`setBackground` 「지정을 해제 할경우 -1값」 · `paintContent` 「색상이 -1이면, 칠하진 않습니다」). **한계**: 실기의 위젯별 기본 전경색과는 다를 수 있다 — 지정 전 `getForeground()` 를 읽어 그리는 게임은 실기와 다른 값을 본다. 이 층은 lwc 를 그리지 않으므로 오늘 화면 영향은 없고, 그런 게임이 측정되면 그때 값을 정한다.

`getForeground` 를 뺄 수도 있었다(벽은 `setForeground` 하나). 넣은 이유: 빼면 보관이 죽은 코드가 되고, 짝을 부르는 다음 게임은 예외로 멈춘다 — -1 을 돌려주는 쪽이 덜 나쁘다.

### 전/후 — 같은 target 에서 `origin/main`(`766ce2a6`) 을 먼저 빌드해 떼어 두고, 이 변경을 증분 빌드

- 명령: release `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(0352 의 프로브 A 그대로) · 벽시계 200초 상한 · 6판 병렬.
- 호스트 부하 load1 **130~290**.

| sha12 | 전 (n) | 후 (n) |
|---|---|---|
| `ca7fa8ade8ad` | FAIL · stop `error` · boot 에서 `Component.setForeground(I)V` 없음 (3/3) · paints 3 · 키 0/27 | FAIL · stop `max-ticks` · **예외 없음** · «only blank/uniform frames» (3/3) · paints 89 · 키 7/27 |

«후» 의 stderr 에는 3/3 모두 `Uncaught exception in thread …: Method <init>(Z)V not found from org/kwis/msp/lwc/FormComponent` 가 한 줄 있다. 부팅 스레드가 아니라 결과가 `error` 로 끊기지 않지만, 화면을 그릴 폼이 만들어지지 않으니 검은 화면의 가장 가까운 원인이다. javadoc 에 `FormComponent(boolean bVertical)` 이 있다 — 다음 회차로 넘긴다(worklog 제안).

### 검증
- 단위 시험 1개: 셸 위에서 초기 -1 · `setForeground` 보관 · `bg` 는 -1 그대로(전경색이 배경색 칸을 덮지 않음).
- 변이 4개 전건 red · 원상 green(저장 안 함): ⑴ `setForeground` 가 저장 안 함 ⑵ 초기 -1 제거 ⑶ `setForeground` 가 `bg` 에 씀 ⑷ `setForeground` 등록 제거.
- 이용자 소식(`docs/player-updates/`) 은 더하지 않았다 — 타이틀이 여전히 검은 화면이라 이용자가 보는 변화가 없다.
- 게이트·러너·CI 결과는 PR 본문과 회신에 있다.

### 게임 이름 유입
`node scripts/corpus-name-inflow.mjs` — BOUNDED **1** · SUFFIX-ATTACHED **0**. 1건은 0352 가 적은 그 줄 — `component.rs` 의 기존 주석(`repaint(IIII)V` 위 · `origin/main` 에 이미 있다)이다. 도구가 «바뀐 파일의 본문 전체»를 보므로 그 파일을 고친 회차마다 잡힌다. 이 회차가 더한 줄에서는 **0건**이다.

