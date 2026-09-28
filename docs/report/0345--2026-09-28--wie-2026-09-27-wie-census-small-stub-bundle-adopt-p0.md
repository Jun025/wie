## [2026-09-28] KTF lwc 위젯 툴킷 3종 — 자식 저장·ButtonComponent·ProgramExitException (wie-2026-09-27-wie-census-small-stub-bundle-adopt-p0)

**무엇을**: `docs/report/0329` 짝 재측의 «후» 벽 3군을 고쳤다. `ContainerComponent` 가 자식을 실제로 들고 `getComponent(I)`·`getNumberOfComponent()` 에 답한다. `DialogComponent` 는 생성자에 받은 컴포넌트를 0번 자식으로 넣는다. `ButtonComponent`·`org/kwis/msf/core/ProgramExitException` 클래스가 생겼다. 타이틀은 sha 앞 12자로만 적는다.
**왜**: 세 타이틀이 lwc 를 «위젯 툴킷»으로 쓴다 — 넣은 자식을 번호로 다시 꺼내고, 버튼 위젯을 만든다. 종전 `addComponent` 는 자식을 버리고 0 을 돌려줬다.
**사용자 영향**: 2종이 이번 벽을 넘어 다음 벽으로 갔다(아래 표). PASS 로 바뀐 것은 없다. 1종(`a10a1f02b41b`)은 그대로다 — 그 벽은 이번 세 군과 무관하다고 쟀다.

### 무엇을 바꿨나 — 근거는 정본 클래스와 javadoc

`docs/reference/AromaWIPI_classes.zip` 의 클래스 파일은 메서드 본문이 0 으로 비어 있는 API 전용이다(계층·시그니처·필드 이름만 읽힌다). 동작은 같은 곳의 `AromaWIPI_javadoc.zip` 에서 읽었다.

| 고친 곳 | 근거 |
|---|---|
| `ContainerComponent.addComponent` 가 자식을 `children`(Vector)에 끝에 넣고 그 인덱스를 돌려준다 | javadoc 「맨 위에 자식 컴포넌트를 추가합니다」 · `getComponent` 는 「stack 순서」로 꺼낸다 |
| `getComponent(I)` — 범위 밖은 **null** | javadoc 「인덱스가 유효한 영역을 벗어나는 경우에는 null을 돌려줍니다」 |
| `getNumberOfComponent()I` | 정본 클래스의 이름이 이것이다. `getComponentCount` 는 정본에 없어 넣지 않았다 |
| `removeComponent(I)`·`(Component)` 가 실제로 뺀다 | 목록을 들고 있으니 빼지 않으면 `getComponent` 가 틀린다 |
| 목록은 첫 사용 때 만든다 | `ShellComponent` 생성자는 `ContainerComponent.<init>` 을 건너뛰고 `Component.<init>` 으로 간다 — `<init>` 에서 만들면 셸·대화상자에만 목록이 없다 |
| `DialogComponent(Component, String, int)` 가 `cmp` 를 자식으로 넣는다(null 이면 넣지 않음) | javadoc 「넓이와 높이는 DialogComponent에 추가된 컴포넌트의 넓이값과 높이값에 따라」 + `ca7fa8ade8ad` 가 생성 직후 `getComponent(0)` 을 부른다(추적 로그) |
| `ButtonComponent` — 부모 `Component` · `<init>()V` · `<init>(String, Image)V` · `setActionListener` · 필드 `str/img/l/o` | 정본 클래스 그대로. `33f3e7669599` 가 `(String, Image)` 생성자를 부른다(빈 클래스를 넣고 잰 다음 벽) |
| `ProgramExitException` — 부모 **`java/lang/RuntimeException`** · `<init>()V`·`(String)V` | 정본 클래스 그대로. 게스트 AOT 런타임이 스스로 던지는 예외 표(NPE·Arithmetic·ClassCast 옆)에 세 타이틀 모두 이 이름을 싣는다 |

**«lwc 는 배치하지 않는다»를 넘은 범위**: 자식 «목록»만 들었다. 배치·그리기는 여전히 하지 않는다 — `getX/getWidth` 는 0, `Component.paint` 는 no-op 그대로다. `ButtonComponent` 는 그려지지 않고, 키를 눌러도 `ActionListener.action` 을 부르지 않는다(`keyNotify` 는 `Component` 의 것). 라벨·이미지·리스너는 보관만 한다.

**확인하지 않은 것**: javadoc 의 `addComponent` 예외 두 가지(null → `NullPointerException`, 이미 부모가 있는 자식 → `IllegalArgumentException`)는 구현하지 않았다 — 부모를 추적하지 않는다. `addComponent` 의 반환값이 인덱스라는 것은 javadoc 이 명시하지 않는다(「stack 순서」와 맞춘 읽기다).

**처음에 틀렸던 것 하나**: `ProgramExitException` 의 부모를 정본 zip 을 보기 전에 `Error` 로 골랐다가(«종료 풀기가 `catch (Exception)` 에 삼켜지면 안 된다»는 추론) 정본을 읽고 `RuntimeException` 으로 고쳤다. 커밋된 것은 정본 쪽이다.

### 전/후 — 같은 스크래치 target 에서 main 을 먼저 빌드해 떼어 두고, 이 변경을 증분 빌드

- 전: `origin/main`(`1dd9f81c`) · 후: 그 위 이 PR. 둘 다 release.
- 명령: `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(프로브 A · `0329` 와 같다). 벽시계 200초 상한. 타이틀마다 전·후를 병렬로.
- 호스트 부하 load1 **218~290**(매우 높다 — 벽시계 축은 덜 진행된 상태로 쟀다).

| sha12 | 전 (n) | 후 (n) |
|---|---|---|
| `ca7fa8ade8ad` | FAIL boot · `ContainerComponent.getComponent(I)` 없음 (3/3) | FAIL boot · 다음 벽 **`Component.setBackground(I)V` 없음** (3/3) |
| `33f3e7669599` | FAIL `12_OK`·`14_OK` · `NoClassDefFoundError: ButtonComponent` → `q.paint` 에서 `java.lang.Error` (3/3) | FAIL `14_OK` · 다음 벽 **`ShellComponent.setTitle(String)V` 없음** (3/3) · paints 97~109 |
| `a10a1f02b41b` | FAIL · 잘못된 메모리 접근 `0xFFFFFFFC` (4/4 — `02_OK` 3 · `01_OK` 1) | 같은 벽 (4/4 — `02_OK` 3 · `04_NUM5` 1) |

`a10a1f02b41b` 는 `getComponent`·`ButtonComponent` 를 참조하지 않는다(client.bin 문자열 기준). 반복 측정에서 전·후가 같은 주소에서 멈추므로 이번 변경의 효과는 없다고 본다. 멈추는 키 단계는 판마다 조금 흔들린다. ※중간 빌드(ButtonComponent 가 빈 클래스이던 때) 1판은 `27_OK` 까지 가서 `AllocationFailure` panic 으로 끝났는데, 최종 빌드 4판에서는 재현되지 않았다. `0329` 가 적은 「한 판은 `ProgramExitException` 없음」은 이번 `a10a1f02b41b` 9판 어디에서도 나오지 않았다 — 클래스는 넣었지만 그 경로가 이제 무엇을 하는지는 관측하지 못했다.

### 검증
- 단위 시험 3개 파일 · 4개: 자식 추가·인덱스 조회·범위 밖 null·삭제(셸 위 — `ContainerComponent.<init>` 을 건너뛰는 경로) · 대화상자 0번 자식 · `ButtonComponent(String, Image)` + `setActionListener` 보관 · `ProgramExitException` 이 `RuntimeException`.
- 변이 6개 전건 red · 원상 green: ⑴ `addComponent` 가 저장 안 함 ⑵ 범위 밖에서 예외 ⑶ 대화상자가 자식을 안 넣음 ⑷ `(String, Image)` 생성자 제거 ⑸ 부모 `Error` ⑹ `getNumberOfComponent` 를 `getComponentCount` 로.
- 게이트·러너 결과는 PR 본문과 회신에 있다.
