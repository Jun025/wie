## [2026-09-04] 키 입력이 게스트에 «도달했는지»를 행동으로 단언한다 — 왕복 검사 Scenario D (wie-featurephone-keypress-reaches-guest-behavioral-axis)
- **무엇을**: `scripts/make-draw-fixture.mjs`(픽스처가 `keyPressed()` 를 **override** 하고 받은 MIDP 코드만큼 넓은 막대를 그린다 · 정적 필드 1개 + 메서드 1개 + `paint` 분기) · `scripts/contract-roundtrip.mjs`(**Scenario D** 3체크 + `tickLoop` 의 `untilPainted` 불리언을 `until(pixels)` 술어로 일반화) · `STATE.md`·`REPORT.md`·`docs/worklog/2026-09-04-featurephone-keypress-reaches-guest.json`. ★**제품 코드 변경 0** — `wie_web/src/lib.rs` 는 개악 대조에만 쓰고 **되돌렸다**(`git diff` 0). ★CI 워크플로 변경 **0**(두 파일 다 `engine-contract.yml` 의 `paths-filter` 목록에 **이미** 있다).
- **왜**: 운영자 채택 제안 `2026-07-22--featurephone-engine-contract-selftest#p0` — 「왕복 검사는 키를 눌러 보지만 «예외가 안 났다»만 확인하고, 그 키가 실제로 게임까지 전달됐는지는 보지 않는다」.
- **★⑴ 종전에 무엇을 쟀나(제안의 전제 확인)**: ⒜`contract-roundtrip.mjs` Scenario A 는 `contract.keyVocabulary` 전건에 `key_down`/`key_up` 을 돌리고 **`try/catch` 로 «throw 안 함»만** 단언한다(`A: key vocabulary down/up sweep (no throw)`). ⒝「소스 핀 하나」 = `scripts/check-engine-contract.mjs` §4 — `wie_web/src/lib.rs` 의 **`fn parse_key` 본문만 잘라** 어휘마다 `"UP" => KeyCode::UP` **좌우 쌍**을 정규식으로 확인한다(좌변만 보면 `"UP" => KeyCode::DOWN` 을 통과시키므로 쌍으로 본다). ★**부정확 1건 정정**: 제안 문안의 「키 **18종**」은 실제로 **20종**이다(`featurephone-engine-contract.json` `keyVocabulary` 실측) — 결론은 안 바뀌지만 수는 고친다.
- **★⑵ 대표 키 3종과 이유**(★어휘 전건으로 넓히지 않았다 — 제안이 「대표 키 한정」): `LEFT_SOFT_KEY`(MIDP **6** · 소프트키 · 셸의 메뉴/뒤로) · `NUM5`(**53** · 숫자 · ASCII 값대) · `UP`(**141** · 방향 · MIDP `Canvas` 명명키 141~148 대 · 셸 D-pad). ★**코드 값이 세 갈래로 갈리고 오름차순**이라 ⑴세 계급을 각각 덮고 ⑵막대가 프레임 간 합집합이어도 기대값이 **정확히** 일치한다.
- **★⑶ 어떻게 «행동»으로 재나**: 픽스처 `DrawCanvas`(J2ME MIDlet)가 `keyPressed(int)` 를 override 해 받은 코드를 정적 필드에 넣고 `repaint()` 를 부른다. `paint` 는 기존 32×32 사각형에 더해 **폭 = 그 코드**인 8px 막대를 그린다 ⇒ 캔버스 비검정 픽셀 = `1024 + code*8`. ★**「키가 왔다」가 아니라 「게스트가 «바로 그 코드»를 봤다」를 단언한다** — 전달 경로 어디서 코드가 바뀌어도 red 다. 경로 실측: `Event::Keydown` → `net.wie.EventQueue.getNextEvent` → `Display.handleKeyEvent` → `Canvas.handleKeyEvent` → `invoke_virtual keyPressed(I)V`.
- **★★⑷ 개악 대조 — 이 회차의 성공 판정**: `wie_web/src/lib.rs` 의 `key_down` 이 `parse_key` 는 하되 **이벤트를 버리게** 고치고 wasm 을 다시 빌드했다.

  | 축 | M1(전달 경로 절단) | M2(오배선 `"UP" => KeyCode::DOWN`) |
  |---|---|---|
  | 소스 핀(`check-engine-contract.mjs`) | ★**48 pass / 0 위반 · rc=0 — 못 잡는다** | ✗ 1 위반 — **잡는다** |
  | Scenario A(기존 「예외 없음」) | ★**✓ 20 codes — 못 잡는다** | (좌동 성격 — no-throw 축) |
  | ★**Scenario D(신설)** | ★**✗ 3건 red**(전부 `1024 px, expected 1072/1448/2152`) | 잡는다(141 ≠ 146) |

  ⇒ ★**제안의 전제는 «참»이었다** — 전달이 통째로 끊겨도 종전 두 축은 **둘 다 green** 이다.
- **★⑸ 오탐 0**: 개악을 되돌리고 다시 빌드한 뒤 정적 **48 pass / 0 위반** · 왕복 **29/29**(종전 26건 전건 통과 + D 3건). `npm run audit` PASSED. 4게이트 전건 green(`cargo test --all` **130 passed / 0 failed**).
- **사용자 영향**: 없음(검사만). 엔진·웹 셸·배포 산출물·의존성 무변경. 얻는 것은 「키가 게임에 안 닿는 회귀」가 **아티팩트 발행 전에** CI 를 red 로 만든다는 것이다.
- **★남는 구멍(한계)**: ⑴★**대표 3종만 «도달»을 재고 나머지 17종은 여전히 «예외 없음»만 본다** — 그 17종의 유일한 탐지축은 소스 핀이고, 소스 핀은 M1 형태(전달 절단)를 **못 잡는다**. ⑵Scenario D 는 **J2ME 경로**만 지난다 — KTF·LGT 의 키 경로는 여전히 단언 밖이다(Scenario C 의 그리기 한계와 같은 형태). ⑶`key_repeat` 은 계약 `methods` 에 없어 재지 않았다. ⑷MIDP 코드 3개가 JS 쪽에 리터럴로 박힌다 — `wie_midp` 의 `MIDPKeyCode` 와 이중 기재지만, 어긋나면 **시끄럽게** 실패하므로 조용한 드리프트는 아니다.
