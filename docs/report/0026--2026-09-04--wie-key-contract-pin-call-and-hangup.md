## [2026-09-04] «통화»·«종료» 두 키를 세 표 전부에서 잠갔다 — 「소비자와 함께 롤아웃」은 필요 없었다 (wie-key-contract-pin-call-and-hangup)
- **무엇을**: `docs/contracts/featurephone-engine-contract.json` — `keyVocabulary` +2(`CALL`·`HANGUP`) · `keyMidpCodes` +2(10 · -1) · `keyWipiCodes` +2(-10 · -11) + `keyVocabularyNote` 신설. ★**검사기 0줄 · 제품 코드 0줄.**
- **왜**: 운영자 채택 제안 `2026-09-04-ktf-third-key-table-pin#p0`. 화면의 «통화»·«종료» 버튼은 **지금도 눌리는데** 그 둘만 세 표 어디에서도 안 잠겨 있었다.
- **★⑴ 제안이 스스로 붙인 선행 조건 2개를 «실측으로» 해소했다**: ⒜「§4b 접촉이므로 형제 회차 판단이 선행」 → §4b(`6b4bf150`)·§4c(`b02467fc`) **둘 다 `origin/main` 조상** · 열린 PR **0건** · 잔여 티켓 **0건** ⇒ 닫혀 있다(그리고 실제로 §4b 코드는 **한 줄도 안 만졌다** — 계약 데이터만 늘었다). ⒝★**「어휘 변경은 소비자 계약이라 otterpebble 과 같은 롤아웃」은 «과했다»** — 실측: otterpebble 이 이 계약 파일을 참조하는 곳 **0건**이고 수신 워크플로는 아티팩트 **sha256 검증 + 핀 범프**만 한다. ⇒ **교차 저장소 롤아웃 불요**(그 문장은 직전 회차가 근거 없이 적은 것이고 이번에 상류를 재서 지웠다).
- **★⑵ 「검사기 로직 변경 0」을 주장하지 않고 «확인»했다**: 세 블록의 루프 머리가 각각 `contract.keyVocabulary` · `Object.entries(contract.keyMidpCodes)` · `Object.entries(contract.keyWipiCodes)` 를 순회하고, 블록마다 «어휘 커버리지» 역검사가 붙어 있다 ⇒ 계약 데이터만 늘리면 검사가 자동으로 는다. 실측 **88 → 94 pass**(+6 = 2키 × 3표), 검사기 diff **0**.
- **★★⑶ 개악 3건 — «표마다 하나씩» 제품 실물에 심었다**: §4 `"CALL" => KeyCode::HANGUP` → `key mapping miswired` · §4b `KeyCode::CALL => Self::HANGUP` → `as -1 … contract pins 10` · §4c `MIDPKeyCode::CALL => Self::HANGUP` → `as -11 … contract pins -10`. ★**셋이 각각 하나씩만 울었다** — 한 표만 red 였다면 나머지 둘은 안 잠긴 것이다. 무개악 **94 pass / 0 violation**.
- **★⑷ 왕복이 자동으로 넓어졌다**: Scenario A 가 어휘를 순회하므로 스윕이 **20 → 22 codes**(29/29 유지) ⇒ 두 키가 `key_down`/`key_up` 을 실제로 통과하는 것도 브라우저에서 보인다.
- **사용자 영향**: 화면의 «통화»·«종료» 버튼이 «다른 키로 나가는» 오작동이 착지 전에 잡힌다. 엔진 동작·배포 산출물 변경 0.
- **★남는 구멍**: `VOLUME_UP`·`VOLUME_DOWN` 2행은 **일부러** 열어 뒀다 — 셸이 보내지 않아 «도달 불가»이고, 넣으면 계약이 쓰이지 않는 표면을 약속하게 된다(세 표 24행 중 **22행** 잠김). 게임 액션 표 2종은 별 티켓 몫이다.

