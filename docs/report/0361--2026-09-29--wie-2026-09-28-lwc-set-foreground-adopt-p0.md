## [2026-09-29] lwc 다음 벽 — FormComponent(boolean) 생성자 (wie-2026-09-28-lwc-set-foreground-adopt-p0)

**무엇을**: `docs/report/0357` 의 «후» 벽. `FormComponent` 에 `<init>(Z)V` 를 넣었다. `bVertical` 은 `vertical` 필드에 보관만 한다. 타이틀은 sha 앞 12자로만 적는다.
**왜**: `ca7fa8ade8ad` 의 부팅 스레드 밖 스레드가 `FormComponent.<init>(Z)V` 없음으로 죽었다(0357 «후» 3/3). 정본 javadoc(`docs/reference/AromaWIPI_javadoc.zip`) 에 `FormComponent(boolean bVertical)` 이 있다.
**사용자 영향**: **화면 변화 0** — 여전히 FAIL(검은 화면)이다. 그 스레드가 폼을 만들고 **한 걸음 더** 가서 다음 벽 `NoClassDefFoundError org/kwis/msp/lwc/ProgressComponent` 에서 죽는다(3/3). 이 층은 폼 배치를 하지 않으므로 `bVertical` 은 읽히지 않는다.

### 무엇을 바꿨나
| 고친 곳 | 근거 |
|---|---|
| `<init>(Z)V` — `ContainerComponent.<init>()` 으로 잇고 `vertical` 에 보관 | javadoc 「폼 컴포넌트를 생성합니다」 |
| `<init>()V` 는 그대로 — `vertical` 은 JVM 기본값(false) | javadoc 은 인자 없는 생성자의 방향을 적지 않는다. 지어내지 않았다 |

보관을 뺄 수도 있었다(벽은 생성자 하나). 넣은 이유: 폼 배치(`layoutChildVertical`/`Horizontal`)가 들어올 때 필요한 유일한 입력이고, 비용은 필드 하나다.

### 전/후 — 같은 target 에서 이 변경을 빌드해 떼어 두고, `origin/main`(`270e4770`) 의 이 파일로 되돌려 다시 빌드
- 명령: release `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(0357 의 프로브 A 그대로) · 벽시계 200초 상한 · 6판 병렬.
- 호스트 부하 load1 **238~244**.

| sha12 | 전 (n) | 후 (n) |
|---|---|---|
| `ca7fa8ade8ad` | FAIL · stop `max-ticks` · «only blank/uniform frames» (3/3) · paints 179~189 · 키 19~20/27 · 딴 스레드 `Method <init>(Z)V not found from org/kwis/msp/lwc/FormComponent` (3/3) | FAIL · stop `max-ticks` · «only blank/uniform frames» (3/3) · paints 177~187 · 키 19~20/27 · 딴 스레드 `java.lang.Error at d.run()V` (3/3) — 원인은 `NoClassDefFoundError org/kwis/msp/lwc/ProgressComponent` |

«후» 의 `java.lang.Error` 가 무엇인지는 `RUST_LOG=debug` 1판으로 확인했다: 같은 스레드가 `ContainerComponent.<init>` → `ShellComponent.<init>` 을 지난 뒤 `KtfClassLoader::findClass(org/kwis/msp/lwc/ProgressComponent)` 가 실패하고 `NoClassDefFoundError` 가 올라온다. javadoc 에 `ProgressComponent` 가 있다 — 다음 회차로 넘긴다(worklog 제안). paints·키 수 차이는 0357 이 말한 부하 잡음 범위 안이다.

### 검증
- 단위 시험 1개: `(Z)V` 로 true/false 각각 만들어 `vertical` 이 그대로 보관되고, 부모 생성자를 거쳐 `getBackground()` 가 -1 로 시작한다.
- 변이 2개 전건 red · 원상 green(저장 안 함): ⑴ `(Z)V` 등록 제거 ⑵ `vertical` 보관 제거.
- 이용자 소식(`docs/player-updates/`) 은 더하지 않았다 — 타이틀이 여전히 검은 화면이다.
- 게이트·러너·CI 결과는 PR 본문과 회신에 있다.
