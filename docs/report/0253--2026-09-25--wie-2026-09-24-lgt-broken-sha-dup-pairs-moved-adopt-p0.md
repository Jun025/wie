## [2026-09-25] `game_lab/broken/ktf` sha256 중복 8쌍 — 태그 붙은 쪽을 `game_lab/_dup/` 로 옮겼다 (wie-2026-09-24-lgt-broken-sha-dup-pairs-moved-adopt-p0)

**무엇을** — `game_lab/broken/ktf` 의 바이트 동일(sha256) 쌍 8개에서 통신사 태그가 붙은 이름(`(KTF)…`·`kt …`·`ktf …`)을
`game_lab/_dup/broken/ktf/` 로 **옮겼다**(삭제 0). #270(`718a9402`)의 `broken/lgt` 9쌍과 같은 경로·같은 규칙이다.
`game_lab/` 은 git-ignored 라 이 PR 의 diff 는 이 문서와 worklog 뿐이다. 옮김 자체는 로컬 코퍼스에서 일어났다.

**왜** — 제안 `2026-09-24-lgt-broken-sha-dup-pairs-moved#p0`. 착수 시 sha256 을 전수 재측했고 **8쌍 그대로**였다.
같은 바이너리를 두 번 돌리면 센서스 수가 부푼다.

| 남긴 쪽 (`broken/ktf/`) | 옮긴 쪽 (`_dup/broken/ktf/`) | sha256 앞 12 |
|---|---|---|
| 아스팔트4.zip | (KTF)아스팔트4.zip | 1b3b4868d46e |
| 물가에돌튕기기2.zip | kt 물가에 돌튕기기2.zip | 6c96c5050b2b |
| 메이플스토리 궁수편.zip | (KTF)메이플스토리 궁수편.zip | 89c214dbd15d |
| 원더즈_-_영웅의길.zip | kt 윈더즈 영웅의길.zip | afa64c89eecc |
| 2006월드올림픽.zip | (KTF)2006월드올림픽.zip | bc94ba53677b |
| 코에이 삼국지 영걸전.zip | (KTF)삼국지 영걸전.zip | c4b90400f9fe |
| 미니러비.zip | (KTF)미니러비.zip | d448aee68157 |
| 생과일타이쿤파이널.zip | ktf 생과일타이쿤파이널.zip | fe481fc815ab |

**모호한 두 쌍은 참조 수로 골랐다.** 두 쌍 모두 한쪽에만 통신사 태그(`kt `·`(KTF)`)가 있다. 그래서 #270 규칙(태그 없는
이름을 남긴다)과 참조 수가 같은 쪽을 가리킨다. 세는 방식은 NFC 정규화 후 부분문자열 계수다(디스크 이름은 NFD).

| 이름 | repo(origin/main) + orchestrator tasks/queue | game_lab reports + census-map + README |
|---|---|---|
| `원더즈_-_영웅의길` | 2 | **8** |
| `kt 윈더즈 영웅의길` | 2 | 5 |
| `코에이 삼국지 영걸전` | 2 | **11** |
| `(KTF)삼국지 영걸전` | 2 | 8 |

repo 쪽 2건은 양쪽 모두 원 worklog 와 이 티켓의 산문 인용이다(경로로 쓰는 명령이 아니다).

**전/후** (sha256 전수 · 로컬 코퍼스)

| 축 | 전 | 후 |
|---|---|---|
| `broken/ktf` zip | 104 | 96 |
| `broken/**` zip | 178 | 170 |
| `broken/**` 고유 sha | 170 | 170 |
| `broken/**` 중복 sha 그룹 (`.gitkeep` 제외) | 8 | 0 |
| `game-lab-recensus.sh --dry-run` 대상 | 178 | 170 |
| `census-map.tsv` 데이터 행 (재생성) | 179 (`files=178`) | 171 (`files=170 stems=167`) |

옮긴 8파일마다 `broken/ktf/` 에 같은 sha 가 정확히 1개 남아 있다. 재생성한 `census-map.tsv` 에 옮긴 이름은 0회 나온다.

**참조 깨짐 0** — 옮긴 8개 이름을 `.zip` 경로로 쓰는 곳: repo(origin/main 추적 파일) **0** ·
orchestrator `tasks/`·`queue/` **0** · `queue/`·`running/` 에서 `broken/ktf` 를 인용하는 티켓은 이 티켓 하나다.
repo 안의 `(KTF)미니러비` 인용 1건(report 0238 표 셀 「미니러비·(KTF)미니러비」)은 경로가 아니라 이름 나열이다.

**사용자 영향** — 없음(엔진·웹 무변경). 코퍼스 기반 수(스윕·센서스)가 ktf 에서 8건 덜 부푼다.
이제 `broken/**` 의 파일 수와 고유 수가 같다(170 = 170).
