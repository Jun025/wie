## [2026-09-27] lwc ShellComponent 를 Display 에 올린다 · Component.repaint 가 그 카드로 — 학교가는길 첫 화면 (wie-2026-09-27-lwc-component-repaint-stub-adopt-p0)

**무엇을**: `ShellComponent.show()` 가 wie 전용 카드 `net/wie/ShellCard` 를 기본 Display 에 올리고(`hide()` 는 내린다), `Component.repaint()`·`repaint(IIII)`·`serviceRepaints()` 가 «그 셸의 카드»가 있으면 Card 의 같은 메서드로 넘긴다. `ShellCard` 는 paint·keyNotify·showNotify 를 셸에 그대로 넘긴다. `Component.paint(Graphics)` 기본(빈) 메서드를 더했다.
**왜**: 0306 이 남긴 벽. 학교가는길은 `Card` 를 한 번도 쓰지 않는다(binary.mod 문자열에 `org/kwis/msp/lcdui/Card` **0**) — 화면이 `ShellComponent` 하위 클래스(`paint`·`keyNotify`·`showNotify`·`show`·`hide` 참조)이고, `show()` 가 스텁이라 Display 에 아무것도 없었다. repaint 가 스텁이었던 것은 그 결과다.
**사용자 영향**: 학교가는길이 첫 화면(흰 바탕 안내문 5줄)을 그린다. 키도 셸의 keyNotify 로 간다.

### 설계 판정 — ⒜ Display 위임(채택) vs ⒝ lwc 자체 페인트 큐
| | ⒜ ShellCard → Card → CardCanvas | ⒝ lwc 페인트 큐 |
|---|---|---|
| 누가 그래픽스를 주나 | 기존 CardCanvas.paint(카드 스택 순회) — Card 게임과 같은 한 경로 | lwc 가 따로 Graphics 를 만들어야 한다 — 두 번째 그리기 경로 |
| repaint → 그리기 | Card.repaint(IIII) → Canvas.repaint → request_redraw(#338 경로 그대로) | 새로 짜야 한다 |
| 키 | CardCanvas.keyPressed → ShellCard.keyNotify → 셸 | 새로 짜야 한다 |
| LGT 배치 | lwc 클래스에 필드 **0** 추가 — 셸↔카드 연결은 카드 쪽 필드와 카드 스택 탐색 | 셸 상태가 필요하면 lwc 필드 → 게스트 하위 클래스 배치가 움직인다(0306 의 TimerTask 와 같은 형태) |

⇒ ⒜. 실제 WIPI 에서도 ShellComponent 는 Display 위에 올라가는 최상위 컴포넌트다.

- 셸↔카드 연결은 `ShellCard::find` 가 기본 Display 의 카드 스택에서 `shell` 필드 identity 로 찾는다(선형 탐색 · 스택은 실측상 1~2장). `show()` 두 번이면 한 장.
- 보이지 않는 컴포넌트(셸이 아니거나 아직 show 전)의 repaint 는 이전처럼 아무것도 안 한다 — 자식 컴포넌트 배치(addComponent 등)는 여전히 스텁이라 그릴 자리가 없다.

### 시험(되돌리면 red)
`shown_shell_component_is_painted_through_the_display`(wie-wipi-java): 학교가는길 모양의 셸 하위 클래스 → show 전 repaint 무예외·카드 0 · show 두 번 → 카드 1·showNotify 1 · `repaint(1,2,3,4)` 가 캔버스에 `[1,2,3,4]` 로 닿음 · serviceRepaints → 셸 paint +1 · keyPressed → 셸 keyNotify 1 · hide → 카드 0.
세 곳을 하나씩 스텁으로 되돌려 실측: show → `left: 0 right: 1` · repaint(IIII) → `left: [0,0,0,0]` · serviceRepaints → `left: 0 right: 1`. 모두 red.

### 학교가는길 실측(release `wie_validate --timeout 20`, 교대 3/3, load1 160~380)
| | origin/main `e5a76be3` | 이 브랜치 |
|---|---|---|
| 판정 | FAIL · only blank · paints 1 · 색 1 — 3/3 | **PASS** · booted + rendered · paints 152/155/153 · 색 2 · nondominant 11.7% — 3/3 |
| java 예외 | 2(`FileNotFoundException`, 부팅 시 저장 파일 탐색 · 게스트가 잡는다) | 2(같음) |

### 회귀
- 라이브 LGT 4종 `--inject`(main ↔ 이 브랜치): 메이플스토리2007 PASS↔PASS · 현영맞고2006 PASS↔PASS · 체스마스터 PASS↔PASS · 놈3 PASS↔PASS. KTF 영웅서기4 `--inject` PASS↔PASS. (현영맞고 예외 6↔9 는 재실행 9↔9 — RMS «Record not found», 진행 정도 따라 는다. 이 타이틀은 lwc 를 참조하지 않는다.)
- lwc 를 참조하는 코퍼스 **228** 타이틀(중첩 jar 까지 문자열 검색; 그중 `ShellComponent` 35) `--timeout 10` 병렬 5 · 타이틀마다 main→이 브랜치 교대:
  - ShellComponent 35: PASS↔PASS 29 · FAIL↔FAIL 5 · **FAIL→PASS 1(학교가는길)**
  - lwc 만 193: PASS↔PASS 161 · FAIL↔FAIL 30 · PASS→FAIL 2 — 둘 다 `--timeout 20` 순차 3회 재실행:
    - 일지매_영웅전기: 양쪽 3/3 PASS ⇒ 부하 굶주림(당시 load1 377).
    - 북천항해기2: main 1/3 FAIL · 이 브랜치 1/3 FAIL, **같은 주소·같은 PC** 의 `Invalid memory access; address: 1782216` ⇒ main 에 이미 있는 간헐 충돌. ShellComponent 를 참조하지 않는다.
  ⇒ 퇴행 0.
- 네 게이트 + beta: fmt 0 · clippy 0 · wasm clippy 0 · beta clippy 0 · `RUST_MIN_STACK=4194304 cargo test --all` 48 스위트 **501 passed 0 failed**.

### 같은 날 표시 경로 변경과의 겹침
- #338(`156c5c56`, repaint → request_redraw): 파일 겹침 0. 이 회차의 repaint 는 Card.repaint(IIII) → Canvas.repaint 로 그 경로를 **그대로 탄다**.
- #345(대기 중, Canvas.serviceRepaints 가 대기 중 repaint 만 그린다): 파일 겹침 0(그쪽은 `wie-midp/…/canvas.rs` 만). 의미 겹침 1 — lwc serviceRepaints 가 Card → Canvas.serviceRepaints 로 가므로 #345 가 착지하면 그 규칙을 따른다. 시험의 스파이 캔버스가 repaint 를 super 로 넘기게 해 두었고, #345 의 canvas.rs 를 얹어 wie-wipi-java 전 시험(37) 통과를 실측했다.

### 한계
- 러너 블록(고정 픽스처)에는 lwc 셸 타이틀이 없다 — 이 경로는 위 단위 시험과 코퍼스 스윕만 본다.
- 셸의 `<init>(IIII)` 경계는 여전히 버린다(ShellCard 는 전체 화면 Card). 학교가는길은 전체 화면이라 영향 없음.
- 첫 화면은 안내문(색 2). 그 다음 화면까지 가는지는 이 회차가 재지 않았다.

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus <game_lab>`): BOUNDED 15회/11쌍 · SUFFIX-ATTACHED 3회/1쌍(학교가는길 + 조사 «은·이» — 진짜 언급). 전부 이 회차가 쓴 이름이다 — 대상 `학교가는길`(코드 주석 2곳 포함: `shell_card.rs` 시험 주석 · `component.rs` 의 기존 주석 줄) · 티켓이 지정한 회귀 5종 · 스윕 재실행 2건(`북천항해기2`·`일지매_영웅전기`, 이 문서만).
