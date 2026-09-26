## [2026-09-26] 배틀몬스터 zip 의 고아 저장 플래그를 적재하지 않는다 — 처음 실행은 새로하기부터 (wie-2026-09-26-battlemonster-orphan-save-flag-exclude-adopt-p0)

**무엇을**: `LgtEmulator::from_archive` 에 타이틀별 제외 목록(`ORPHAN_ENTRIES`)을 두었다. 앱 jar 의 MD5 가
배틀몬스터 것이고 아카이브에 `master.sav` 가 없으면 `mastercom.sav` 를 가상층에 올리지 않는다.
채택 제안: `2026-09-26-battlemonster-continue-orphan-save-flag#p0`(0272).

**왜**: zip 사본의 `mastercom.sav` byte[3]=01(«저장 있음»)인데 `master.sav` 가 없어, 처음 켠 사용자도 이어하기로
들어가 0 상태 마을에서 캐릭터가 움직이지 않았다(0272).

**사용자 영향**: 처음 켜면 메뉴 커서가 «1 새로하기»이고 새로하기 대화상자가 «게임을 새로 시작합니다»다.
자기 저장이 있는 사용자(웹 = IndexedDB)는 그대로 이어하기가 된다 — 플랫폼 FS 가 가상층보다 먼저 읽힌다.
다른 타이틀은 동작 무변경.

### 반증 먼저

- ⓐ 같은 기구 없음 — `origin/main` `5434ab0a` 의 `load` 는 zip 전 파일을 `add_virtual` 한다. 제외 코드 0.
- ⓑ 재현 성립 — `main` 빌드 · 빈 저장소에서 메뉴 커서 «2 이어하기» · 새로하기 → «저장된 데이타가 있습니다».

### 키: 왜 zip sha256 이 아니라 jar MD5 인가

`from_archive` 는 압축 해제된 파일만 받고 zip 바이트를 보지 못한다(웹 `build_emulator` · `wie_validate` 둘 다
`extract_zip` 뒤에 부른다). jar 는 타이틀 빌드를 가리키고 `md5` 는 이미 작업공간 의존성이다(`wie-core-arm` 의
바이너리 패치 표와 같은 관용). 등재 키(zip sha256 `a30bbe00…`)는 주석으로 옆에 적었다.
`unless_present: master.sav` 로, 같은 jar 라도 진짜 저장 데이터를 실은 zip 이면 플래그를 남긴다.

### 검증(헤드리스 · 저장소 밖 스크래치 패치 = 디스크 FS + 대화형 키 · 커밋 안 함)

| 실행 | 결과 |
|---|---|
| main · 빈 저장소 | 커서 «2 이어하기» · 새로하기 → «저장된 데이타가 있습니다» |
| 수정 · 빈 저장소 | `Not mounting orphan save flag mastercom.sav` · 읽기 FNF → 게임이 `03 01 32 00 …`(byte[3]=00) 생성 · 커서 «1 새로하기» · «게임을 새로 시작합니다» → 예 → 인트로 |
| 수정 · 자기 저장(0272 회차 저장본) | 이어하기 «저장데이타를 불러옵니다» → `master.sav` 1,900 B 읽음 → 마을 · 변한 화소 무입력 965 ↔ RIGHT 42,330 · RIGHT 50,588 · LEFT 49,701 |
| 메이플스토리2007 main ↔ 수정 | 둘 다 PASS 27/27 · 드롭 로그 0 · 아카이브 `maplecfg.do`·`maplesave1.do`·`maplesave2.do` 를 양쪽 다 연다 |

- ★첫 수정 빌드는 MD5 한 바이트를 잘못 옮겨 적었다(`0x20` → `0x2b`) — 드롭 0 · 화면 무변경. 단위 테스트는 합성 표라
  못 잡았고 헤드리스 실행이 잡았다 ⇒ 이 빌드가 음성 대조다(키가 틀리면 아무것도 안 한다).
- 메이플 `paints` 는 부하(load 97~140)로 124 ↔ 586 이 흔들린다 — 판정 근거로 쓰지 않았다.
- 고정 키 스케줄은 이 부하에서 메뉴 전환 중 키를 삼켰다 ⇒ 판정은 전부 대화형 제어(키 사이 6~30 s 대기)로 했다.

### 한계

- 셸(otterpebble) 실브라우저에서는 재지 않았다 — 웹 호스트도 같은 `from_archive` 를 부른다는 코드 판정.
- 배틀몬스터는 셸에서 아직 aot-java 게이트(미등재)로 막혀 있다 — 이 변경은 그 게이트를 건드리지 않는다.

증거(저장소 밖 · 게임 바이트 0): `~/orchestrator/reports/evidence/wie-2026-09-26-battlemonster-orphan-save-flag-exclude-adopt-p0/`.

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus <로컬 코퍼스>`): BOUNDED 쌍 — 전부 `배틀몬스터`(이 회차 대상 · 0272 가 이미 적음) ·
`메이플스토리2007`(대조 타이틀 · 0272·0208 이 이미 적음) · `영웅서기4`(`emulator.rs` 의 기존 테스트 줄 — 이 회차가 쓰지 않았다) ·
SUFFIX-ATTACHED 2회/2쌍 — 둘 다 `배틀몬스터는`(조사 «는» · 진짜 언급).
