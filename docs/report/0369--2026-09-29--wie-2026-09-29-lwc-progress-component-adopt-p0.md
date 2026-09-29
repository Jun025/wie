## [2026-09-29] lwc 다음 벽 — ContainerComponent.removeAllComponents() (wie-2026-09-29-lwc-progress-component-adopt-p0)

**무엇을**: `docs/report/0366` 의 «후» 벽. `org/kwis/msp/lwc/ContainerComponent` 에 `removeAllComponents()V` 를 넣었다 — 자식 목록(`removeComponent` 와 같은 `children` Vector)을 `removeAllElements()` 로 비운다. `ShellComponent` 는 부모가 `ContainerComponent` 라 상속으로 받는다. 타이틀은 sha 앞 12자로만 적는다.
**왜**: `ca7fa8ade8ad` 의 딴 스레드가 `Fatal error: Method removeAllComponents()V@93 not found from org/kwis/msp/lwc/ShellComponent` 로 죽었다(0366 «후» 3/3). 정본 javadoc(`docs/reference/AromaWIPI_javadoc.zip` `ContainerComponent`) 「모든 컴포넌트를 삭제합니다.」
**사용자 영향**: **검은 화면 → 첫 장면이 나온다.** `wie_validate` 판정 FAIL → **PASS**(3/3). 단 **뒤 화면은 깨진다** — 키 4~5 번째부터 스프라이트 시트 전체가 화면 위에 겹쳐 그려진다(아래 «한계»). 다음 벽은 worklog 제안.

### 전/후 — 같은 target 에서 `origin/main`(`b4020093`) 을 먼저 빌드해 떼어 두고, 이 변경을 증분 빌드
- 명령: release `wie_validate --inject --keep-timeout --timeout 30 --pacing 8`(0366 그대로) · 3판 병렬 · load1 **196~271**.
- 1차(기본 `--max-ticks`): «후» 3/3 이 `UNMEASURED` rc=2 · stop `max-ticks` · 키 11~12/27 — 치명 벽이 사라져 판이 계속 돌면서 틱 상한에 먼저 닿았다(AGENTS §러너 블록 「on max-ticks raise `--max-ticks`」). 그래서 `--max-ticks 1000000000` 을 더해 다시 쟀다(아래 표).

| sha12 | 전 (n) | 후 (n) |
|---|---|---|
| `ca7fa8ade8ad` | **FAIL** · stop `deadline` · 키 27/27 · `content false` · 마지막 프레임 색 1 (3/3) · 딴 스레드 `removeAllComponents()V … not found` (3/3) · rc=1 | **PASS** · stop `deadline` · 키 27/27 · `content true` · 마지막 프레임 색 256~342 (3/3) · 치명 오류 **0** · rc=0 |

「전」 이 곧 변이(메서드 없음)다 — `origin/main` 에 `removeAllComponents` 가 없다. 양쪽 다 `SchemeNotFoundException: Network is not supported` 1건이 있다(이 변경과 무관 · 전에도 있었다).

### 한계 — 눈으로 본 것
`--shotdir --shot-every 5` 로 한 판을 찍어 봤다(화면 캡처는 저장소에 넣지 않는다 — Constraint 9). 키 3(`LSOFT`) 까지는 인트로 장면이 제대로 나온다. 키 5(`DOWN`) 부터 **스프라이트 시트·글꼴 시트가 통째로** 장면 위에 겹쳐 그려지고, 키 7 이후엔 화면 대부분을 덮는다. 원본 이미지의 일부만 잘라 그려야 할 자리에서 잘라내기(클립 또는 원본 영역)가 적용되지 않는 모양이다 — 판정기는 «색이 있다» 만 보므로 PASS 로 센다. 원인은 이 회차에서 좇지 않았다(worklog 제안).

### 검증
- 단위 시험: 기존 `added_children_come_back_by_index_and_leave_on_remove`(ShellComponent 위) 끝에 `removeAllComponents()` → 개수 0 · `getComponent(0)` null 을 더했다.
- 변이 2개 전건 red · 원상 green(저장 안 함): ⑴ proto 등록 줄 제거 ⑵ 본문을 `Ok(())` 로(비우지 않음).
- 게이트·러너·CI 결과는 PR 본문과 회신에 있다.

### 게임 이름 유입
`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` — BOUNDED **0** · SUFFIX-ATTACHED **0**.

<!-- corpus-name-inflow v1 subjects=4 tree=8622a4560d09c356 B=0/0 P=0/0 S=0/0 -->
