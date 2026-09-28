## [2026-09-29] lwc 다음 벽 — ProgressComponent 클래스 (wie-2026-09-29-lwc-form-component-vertical-adopt-p0)

**무엇을**: `docs/report/0362` 의 «후» 벽. `org/kwis/msp/lwc/ProgressComponent`(부모 `Component`)를 새로 넣었다 — `<init>(ZI)V` · `setValue(I)I` · `getValue()I` · `getMaxValue()I` · `setMargin(II)V`. 타이틀은 sha 앞 12자로만 적는다.
**왜**: `ca7fa8ade8ad` 의 딴 스레드가 `NoClassDefFoundError org/kwis/msp/lwc/ProgressComponent` 로 죽었다(0362 «후» 3/3 · 표면은 `java.lang.Error at d.run()V`). 정본 javadoc(`docs/reference/AromaWIPI_javadoc.zip`) 에 그 클래스가 있다.
**사용자 영향**: **화면 변화 0** — 여전히 FAIL(검은 화면)이다. 그 스레드가 진행 막대를 만들고 **한 걸음 더** 가서 다음 벽 `removeAllComponents()V` 없음에서 죽는다(3/3). 이 층은 lwc 위젯을 그리지 않으므로 **막대는 보이지 않는다**.

### 무엇을 넣었나 — 게임이 부르는 것만
타이틀의 AOT 이미지(`client.bin`) 문자열 표에서 `(sig)+name` 참조를 걷었다. javadoc 의 `ProgressComponent` 메서드 중 표에 **서명까지 일치**하는 것은 `(ZI)V+<init>` · `(I)I+setValue` · `(II)V+setMargin` 셋뿐이다(`setStep`·`setMaxValue`·`getStep`·`getValue`·`setChangeListener` 는 참조 0).

| 넣은 것 | 근거 |
|---|---|
| `<init>(ZI)V` — `Component.<init>()` 으로 잇고 `interactive`·`max` 보관 | javadoc 「0을 최소값으로 하는 새로운 프로그래스 콤포넌트」 |
| `setValue(I)I` — 0 미만은 0, max 초과는 max 로 보관하고 그 값을 돌려준다 | javadoc 「value 가 0보다 작은경우 0 … MAX 보다 큰경우 MAX」 · 반환 「설정된 값」 |
| `getValue()I` · `getMaxValue()I` | 게임 참조는 없다. 빼면 `setValue` 보관이 죽은 코드가 된다 — 짝이 필드 읽기 하나라 넣었다 |
| `setMargin(II)V` — 받기만 한다 | 여백은 그려진 막대의 모양만 바꾼다. 이 층은 그리지 않는다 |

**한계**: step 은 javadoc 기본값 1 로 본다(`setStep` 미구현 → `value - value % step` 은 항등). `interactive` 모드의 키 증감(`keyNotify`)·`paintContent` 는 없다.

### 전/후 — 같은 target 에서 `origin/main`(`57506d11`) 을 먼저 빌드해 떼어 두고, 이 변경을 증분 빌드
- 명령: release `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(0362 의 프로브 A 그대로) · 벽시계 200초 상한 · 3판 병렬.
- 호스트 부하 load1 — 전 **136** · 후 **650~725**.

| sha12 | 전 (n) | 후 (n) |
|---|---|---|
| `ca7fa8ade8ad` | FAIL · stop `max-ticks` · «only blank/uniform frames» (3/3) · paints 170~174 · 키 27/27 · 딴 스레드 `java.lang.Error at d.run()V` (3/3) | FAIL · stop `max-ticks` · «only blank/uniform frames» (3/3) · paints 87 · 키 7/27 · 딴 스레드 `Fatal error: Method removeAllComponents()V@93 not found from org/kwis/msp/lwc/ShellComponent` (3/3) · `java.lang.Error` **0** |

「전」 이 곧 변이(클래스 없음)다 — `origin/main` 에 `ProgressComponent` 가 없다. «후» 의 새 벽은 `WieError`(치명) 라 그 뒤로 판이 일찍 끝난다(ms 6.4초 · 키 7/27) — paints·키 수가 준 것은 이 때문이지 퇴행이 아니다(«전» 도 같은 검은 화면). `removeAllComponents()` 는 javadoc 의 `ContainerComponent` 공개 메서드다 — 다음 회차로 넘긴다(worklog 제안).

### 검증
- 단위 시험 1개: `(ZI)V` 로 만들어 `getMaxValue` 100 · `setValue` 40/-5/250 → 40/0/100 이 반환·보관되고, 부모 생성자를 거쳐 `getBackground()` 가 -1.
- 변이 3개 전건 red · 원상 green(저장 안 함): ⑴ `get_protos` 에서 클래스 등록 제거 ⑵ 범위 자르기 제거 ⑶ `value` 보관 제거.
- 이용자 소식(`docs/player-updates/`) 은 더하지 않았다 — 타이틀이 여전히 검은 화면이다.
- 게이트·러너·CI 결과는 PR 본문과 회신에 있다.
