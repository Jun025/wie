## [2026-10-09] J2ME 에 표준 키코드(-1~-7) 전달 — 데모와 함께 (wie-j2me-standard-keycodes-with-demo)

**무엇을** — `Canvas` 경계에서 **J2ME 일 때만** 키값을 표준 코드로 바꿔 준다: 위 -1 · 아래 -2 · 왼쪽 -3 ·
오른쪽 -4 · 확인 -5 · 소프트 L/R -6/-7 · CLR -8 · 통화 -10 · 종료 -11(SKVM 의 -1 이 표준에서는 «위»라 노키아
종료키 값으로 옮겼다). 숫자·`*`·`#` 은 그대로. `getGameAction`·`getKeyCode`·`getKeyName` 도 같은 표로 대칭이다.
표시는 `Canvas.standardKeyCodes`(정적 필드)이고 `wie-j2me` 만 켠다 — SKVM(같은 `wie-midp` 사용)·KTF·LGT 는 무변경.
이벤트 큐는 계속 SKT 값을 싣는다(Screen·List·Gauge·`GameCanvas.getKeyStates` 가 그것을 읽는다) — 번역은
`Canvas.handleKeyEvent` 가 `keyPressed/Released/Repeated` 로 넘기는 한 곳뿐.
데모(`demo/arcade-common`)는 종료키를 -11 로 바꿨고, `-1` 은 `getKeyCode(UP) != -1`(= 옛 엔진)일 때만 종료로 읽는다
— 새 jar 는 옛 엔진·새 엔진 둘 다에서 맞게 돈다. 동물줄맞춤 2.1.0 → **2.1.1**.

**왜** — #517 실측: 노키아·소니에릭슨용 게임은 방향키를 -1~-4, 소프트키를 -6/-7 로 기다리는데 wie 는 SKT 값
(위 141 · 소프트 6/7)을 보내 방향키가 먹지 않았다(숫자키·`getGameAction` 쓰는 게임만 움직였다).

**사용자 영향** — 일반 자바 게임에서 방향키·소프트키가 먹는다. 통신사 게임은 그대로.

### 실측 (release `wie_validate` · 옛 = origin/main `82959a5b` · 증거 `~/orchestrator/reports/evidence/wie-j2me-standard-keycodes-with-demo/`)
| 실행 | 키 | 옛 엔진 | 새 엔진 |
|---|---|---|---|
| sperm-race | `NUM5 NUM5 NUM5 RIGHT RIGHT` | RIGHT 두 장 = 직전과 같은 해시 `16627bf767` ×3 | `16627bf767` → `04b49d3315` → `670c35976b`(새 화면 2장) |
| j3de | `RIGHT×3 UP LEFT` | 전 장 `fd6c683439` | 같음 — 게임 자신의 «script error» 화면에서 멈춰 `keyPressed` 상태에 못 닿는다(엔진 키와 무관 · 미해결). `keyPressed` 의 `lookupswitch -4,-3,-2,-1,50,52,54,56` 은 javap 로 확인 |
| 데모 옛 jar(`9a18bf3f`) | 메뉴에서 `UP` | — | **「나가시겠어요?」** 가 뜬다(`b4c81982a4`) — 데모를 안 바꾸면 생길 일 |
| 데모 새 jar(`88f99a9a`) | 메뉴에서 `UP` | 커서가 «게임 나가기»로(`9ada4dfad4`) | 같은 해시 `9ada4dfad4` — 종료 확인 없음 |
| 데모 playthrough(136키) | `playthrough.keys` | 새 jar PASS 136/136 · clean exit | 새 jar PASS 136/136 · clean exit · 옛 jar 는 35키째 종료(UNMEASURED) |

- 통신사 코퍼스 smoke gate(294 · 3분할 병렬 · release 바이너리): **294 PASS / 0 FAIL · baseline 292 확인 · 퇴행 0**.
- 시험: `j2me_canvas_gets_standard_key_codes_and_skvm_does_not` — 번역을 지우면 J2ME 절반이, 무조건 번역하면 SKVM 절반이 red(번역 제거로 red 확인).
- 데모 빌드 2회 동일 sha256 `88f99a9a0de36695acf7cdd0bf57b33cd3433112b309c1daccb36279cdb4085f` · 58,752 B(openjdk 17).

### 계약(Constraint 3) — 같은 PR
- `docs/contracts/featurephone-engine-contract.json` 에 **`keyJ2meCodes`**(J2ME 게스트가 받는 정수 22개) 신설. `keyMidpCodes` 는 SKVM 게스트 기준으로 그대로 참이다.
- `check-engine-contract.mjs` **§4e**: `to_standard` 의 각 갈래를 `keyJ2meCodes` 에 대조(갈래 없는 키는 `_ => code` 이므로 `keyMidpCodes` 값과 같아야 함) + `from_standard` 가 각 갈래를 되돌리는지. 갈래 값 하나 바꾸면 «miswired», 역표 하나 바꾸면 «asymmetric» 으로 red(둘 다 손으로 확인).
- 브라우저 왕복 **Scenario D** 는 J2ME 게스트라 이제 `keyJ2meCodes`(-6 · -1 · 53)를 기대한다. 음수 폭은 아무것도 안 그리므로 픽스처가 막대 폭에 `KEY_CODE_BIAS = 16` 을 더한다(`make-draw-fixture.mjs` 수출 · `keyBarPixels` 가 반영). 키는 코드 오름차순으로 정렬해 쓴다. 로컬 `contract-roundtrip.mjs` **67/67**.
  `test_data/draw_j2me.zip` 은 새 jar 로 다시 묶었다(옛 zip 의 날짜·압축 속성 그대로 — 옛 jar 로 같은 방법을 쓰면 옛 zip 과 바이트 동일함을 먼저 확인).
- 셸은 키 «이름»만 보내므로 otterpebble 쪽 코드 변경은 없다.

### 배포 순서 — ★엔진보다 데모 jar 가 먼저
엔진 머지는 곧바로 셸에 배포된다(otterpebble `wie-artifact-receive.yml` 이 핀을 직접 커밋·배포). 셸의 데모 jar 는
otterpebble `apps/featurephone/public/demo/` 에 커밋된 옛 바이트(`9a18bf3f`)라 **새 엔진 + 옛 jar = 위쪽 키가 종료 확인**이다.
새 jar 는 옛 엔진에서도 맞게 돌므로, otterpebble 에 새 jar(`88f99a9a`)·`demo.ts` sha256/bytes 를 **먼저** 등재하고 그다음 이 PR 을 머지한다.

### 한계
- `getGameAction` 은 숫자키(2/4/6/8/5)를 게임 액션으로 접지 않는다(종전과 같음 — 단말마다 다르다).
- 볼륨 키(13/14)는 번역하지 않았다(표준 값 없음).
