## [2026-10-07] 진도 4차 KTF — stuck 70 분류 · 미측정 17 중 10 측정 · 엔진 벽 3군집 수정 (wie-progression-wave4-stuck-ktf-classify-and-engine-walls)

**무엇을**: KTF playable 중 `progress` 가 `stuck`(70) 이거나 없는(17) 타이틀을 다뤘다. 미측정 10종을 정책 v2 600초 P·P2 로 쟀다. stuck 70 은 마지막 화면과 첫 결함 지점으로 나눴다. 엔진 벽 3군집을 고쳤다.

**왜**: 운영자 지시(2026-09-29·30 «continue»). 셸이 진도 배지를 보여 주므로 «막힘»이 이용자에게 보인다. KTF 가 그중 70종이다.

**사용자 영향**:
- KTF 3종이 막힌 자리를 지난다.
  - `0c67145b11df`: 이름 칸에 네 글자를 넘게 치면 게임 스레드가 죽었다. 이제 이름을 받고 본편으로 간다.
  - `09a6a300994d`: 프로필 입력 화면에서 모든 키가 예외를 냈다. 이제 칸 사이를 옮겨 다닌다.
  - `33f3e7669599`: 프로필 확인(«1.응») 뒤 처음 단계로 되돌아가기만 했다. 이제 두뇌 게임 본편으로 간다.
- 미측정이던 KTF 10종의 진도 배지가 생긴다(ok 4 · stuck 6).
- `689c491586b9` 에 «시작 때 통신사 서버 접속이 필요하다»는 안내 한 줄이 붙는다.

### 1. 범위와 측정 조건

| 엔진 | sha | 쓴 곳 |
|---|---|---|
| main | `f5c02609` | 미측정 P·P2 · 고치기 전 짝 |
| 이 브랜치 | `49228d89` 의 release(`wv_h2`·`wv_h3`) | 고친 뒤 짝 · 현 핀 짧은 재현 |

- 미측정 17 중 측정 10. 나머지 7은 진도 후보가 아니어서 재지 못했다(§2).
- stuck 70 은 이 회차에 600초로 다시 재지 못했다. 이유는 둘이다.
  - census 는 호스트 잠금 하나를 쓴다. 형제 레인(`wie-3`·`wie-4`)의 진도 측정과 번갈아 써야 했다.
  - long 빌드 슬롯 2칸도 상시 만석이었다. 그래서 17:32 → 19:58 동안 미측정 묶음 하나(P 10 + P2 7)만 돌았다.
- 그래서 stuck 70 의 분류 근거는 두 겹이다(§3).
  - 구핀 census: 로컬에 남은 가장 최근 P·P2(2026-09-30 ~ 10-06 핀). 마지막 화면 7~9장을 직접 봤다.
  - 현 핀 짧은 재현: 이 브랜치 빌드로 정책 v2 키를 100~150초 돌렸다(`--relaunch 8` · 10초마다 한 장). 예외·오류 줄과 화면을 봤다.
  - ★짧은 재현은 «600초의 마지막 3분의 1에 새 화면이 있는가»를 답하지 못한다. 그래서 현 핀에서 진행이 보인 타이틀은 ok 로 올리지 않았다. ⒟ «재측 필요»로 남겼다.
- 측정 조건:
  - 드라이버는 `build-slot run --long` 임대 1개로 감쌌다. 단계마다 `host-load-guard --status --recovered` 를 최대 15분 기다렸다. 회복이 없으면 jobs 2 로 돌렸다(P 17:32 jobs 2).
  - P 17:32–18:55(load1 64 → 50 · idle 0%). P2 19:10–19:58(load1 13 → 9 · idle 1% → 36%). 타이틀별 load1 은 P 20~50 · P2 9~25 다.
  - 짧은 재현과 짝 측정은 `build-slot run`(short) 1~2개씩이었다. `nohup` 0.
  - 드라이버는 19:59 에 손으로 내렸다. 내린 뒤 `w4k` 프로세스 0 을 확인했다.
  - 고친 3종의 600초 P·P2(이 브랜치 빌드)를 둘째 드라이버로 걸었다. long 슬롯을 20:02–20:30 기다렸다. 그 뒤 census 호스트 잠금을 20:30–21:10 기다렸다(형제 레인의 프로브 census 가 쥐고 있었다). 21:10 에 내렸다. 잰 판 0 · 남은 프로세스 0 이다.

### 2. 미측정 17

| 판정 | 수 | sha12 |
|---|---|---|
| ok | 4 | `bc94ba53677b`(P 정체 10초) `6eb93824daf8`(20) `5267badf20b3`(180) `89c214dbd15d`(P 210 → P2 10) |
| stuck(P2 동의) | 6 | `8c71be3ad26d`(480/430) `1cdea1985955`(220/290) `3b5b98afa890`(490/490) `d552e095ddcf`(340/340) `0eb19d9bbe7a`(270/280) `3151fdc167b6`(340/340) |
| 후보 아님 — 로컬 L 이 옛 핀의 오류 | 3 | `55aadf368b8e`(옛 L «jump native address is null») `4a4d2ac046f7`(옛 L «Unimplemented 11: MC_dbGetRecordSize» — main 에 이미 있다) `8d8c24b7c198`(옛 L 예외) |
| 후보 아님 — 로컬 L 없음/미완 | 2 | `3c658a46bbfb`(L 없음) `5a59f62d1f1a`(L UNMEASURED) |
| 다른 티켓 | 2 | `1d5831e42a8a` `b907b0faf483` — 오늘 #499 로 켜졌다. 로컬 L 이 없다 |

- census 는 boot·render·longplay 가 ok 인 타이틀만 진도를 잰다. 로컬 out 에 이 핀의 L 이 없어 옛 핀 L 을 옮겨 썼다. 그래서 위 7종이 빠졌다. compat 는 7종 모두 longplay ok 라 L 을 이 핀으로 다시 재면 후보가 된다(§후속).
- stuck 6종의 마지막 화면(P2):
  - `8c71be3ad26d`: 타이틀에서 정지. 타이틀 위 숨은 `ShellComponent`(71,151,74×16)에 `TextBoxComponent`(최대 5자)가 포커스를 갖는다. 키가 그 칸으로 간다. 게임이 `getString().charAt(4)` 를 불러 `StringIndexOutOfBoundsException: index 4` 가 난다 ⇒ ⒝ 남김(§5).
  - `1cdea1985955` · `3b5b98afa890`: 타이틀 메뉴와 도움말 ⇒ ⒜.
  - `d552e095ddcf`: 메뉴 → «정보이용료 안내» → 타이틀로 돈다 ⇒ ⒜.
  - `0eb19d9bbe7a`: 옵션 화면 ⇒ ⒜.
  - `3151fdc167b6`: 지도/«EXIT-CLR» 화면 ⇒ ⒜.

### 3. stuck 70 분류

| 계급 | 수 |
|---|---|
| ⒝ 엔진 벽 — 고침(§4) | 3 |
| ⒝ 엔진 벽 — 다른 티켓 소관(`wie-progression-engine-walls-r4-…`) | 3 |
| ⒝ 엔진 의심 — 남김(§5) | 2 |
| ⒞ 통신 | 6 |
| ⒜ 키 루프 한계(메뉴·안내·슬롯·입력값) | 21 |
| ⒟ 판정 보류 — 현 핀 짧은 재현에서 진행 · 600초 재측 필요 | 35 |

- 근거 열: «구» = 구핀 census 마지막 화면. «현» = 현 핀 짧은 재현.
- 이 표는 «현 핀 600초 판정»이 아니다. ⒟ 35종은 옛 핀 이후 엔진 수정(#458 이후)으로 풀렸을 가능성이 큰 무리다. ok 로 올리지 않았다.

| sha12 | 계급 | 첫 결함 지점 / 마지막 화면 |
|---|---|---|
| `0c67145b11df` | ⒝ 고침 | 이름 칸에 5자 이상 → 게임 스레드 `ArrayIndexOutOfBoundsException: 16 > 8`(§4-1) |
| `09a6a300994d` | ⒝ 고침 | 프로필 칸의 키마다 `removeComponent(0)` → `0 >= 0`(§4-2) |
| `33f3e7669599` | ⒝ 고침 | 키 처리에서 `Calendar.getInstance(null)` → `NullPointerException: timeZone`(§4-3) |
| `96dc32e781d3` | ⒝ 타 티켓 | 3차 r6 §3-1(객체 머리 낱말) |
| `f2280c6699a0` | ⒝ 타 티켓 | 3차 r6 §3-1(타이머 콜백 `free`) |
| `c5b3f6835d00` | ⒝ 타 티켓 | 힙 고갈 |
| `c3057f46c59b` | ⒝ 의심 | 현: 게임 스레드 `c.run` 이 `ArrayIndexOutOfBoundsException: 98 > 54` 로 죽는다. `data/text1.txt`(54바이트)를 `InputStreamReader` 로 읽은 직후 `System.arraycopy(…, 98)` 이다 |
| `a10a1f02b41b` | ⒝ 의심 | 현: `load_java_class(wec/DMInfo)` 실패 → «플랫폼 예외 · 새 버전을 확인하세요 · Error» 화면 |
| `3185174d2121` | ⒞ | 현: «스토리 모드» → `FileSystem.exists` → `org.kwis.msf.io.Network.connect()` = -1 → 글자 없는 보라 상자에서 정지 |
| `30c7bd6fb01b` | ⒞ | 현: «게임빌 매니아 접속» → «서버 접속 중…» |
| `689c491586b9` | ⒞ | 현: «네트워크 접속에 실패 하였습니다 · 다시 시작하세요» → 재실행 8회 |
| `dbd078113b97` | ⒞ | 현: 랭킹 메뉴 «통화료 부과» → «통신장애입니다» · 이벤트 처리 NPE 16회 |
| `77c2d0bcd435` | ⒞ | 현: «네트워크 문제로 서버에 접속하지 못하였습니다» |
| `568c339a8c07` | ⒞ | 3차 r6: «네트워크 접속에 실패하였습니다» |
| `739c7657c1f2` | ⒜ | 현: 이어하기 빈 슬롯(3차 r1 레시피 ok) |
| `c95bf2740c69` | ⒜ | 현: 도움말 |
| `8c6821fab969` | ⒜ | 현: 게임 문의 화면 |
| `db8ef04a6504` | ⒜ | 현: 속도 체크 → 메뉴·조작 안내 |
| `5aa438fbdc87` | ⒜ | 현: «게임방법» 화면(구: 추가 다운로드 안내) |
| `49f2734f17d8` | ⒜ | 현: 환경설정 |
| `a20c2044305c` | ⒜ | 현: 상태창 |
| `0f9e1026724d` | ⒜ | 현: 아이템 조합 «재료가 부족합니다» |
| `d5e996a53118` | ⒜ | 현: 불러오기 EMPTY ↔ 메뉴 |
| `cbf36fee9f63` | ⒜ | 현: 기계 구입 창(가격 1,000,000 > 소지 50,000). `NumberFormatException "_4"` 175회는 게임 자료 문자열의 `substring(2)` 이다 |
| `fe184f834bc7` | ⒜ | 현: 옵션 |
| `e09aca27c132` | ⒜ | 현: 생일 입력 «입력날짜오류» |
| `f5bd7a91a107` | ⒜ | 현: 저장 선택 창 |
| `ea35907b22a4` | ⒜ | 현: LOAD 빈 슬롯 · 조작법 |
| `1e43e2e0055f` | ⒜ | 현: 일정 선택 «이미 선택된 일정» |
| `0392263fbb85` | ⒜ | 현: 불러오기 슬롯 |
| `3ccc6cf147d2` | ⒜ | 현: 환경설정 ↔ 종료 확인 |
| `4288d8c1c6ac` | ⒜ | 현: 환경설정 |
| `4b6eaa69e056` | ⒜ | 현: «모든 데이터가 삭제됩니다» 예/아니오 |
| `69e516bb2ffa` | ⒜ | 현: 메뉴 |
| `9e16cc54d0ab` | ⒜ | 현: 환경설정 · 조작법 |
| `135d1291501f` | ⒟ | 구: 메뉴 · 현: 이야기 → 지도. `/bg_wa1.png` 없음 1회는 jar 에 원래 없는 파일이다 |
| `145f760b2f4a` | ⒟ | 구: 메뉴 · 현: 성 이야기 |
| `155586ece7f8` | ⒟ | 구: 메뉴 · 현: 이야기 → 지도 |
| `174237758542` | ⒟ | 구: 메뉴 · 현: 속도 체크 → 튜토리얼 |
| `0cc4ef7ede37` | ⒟ | 구: 메뉴 → 옵션 · 현: 사격 훈련 |
| `e538cbb4e687` | ⒟ | 구: 정보이용료 안내 정지 · 현: 안내 → 야구 메뉴 → 타격 |
| `5a0fa312a793` | ⒟ | 현: 횡스크롤 전투 |
| `cb7c7f87f9e6` | ⒟ | 현: 이야기 → 가게 메뉴 |
| `75f002ba70e3` | ⒟ | 구: 추가 다운로드 안내 후 종료 · 현: 이야기 → 체육관 |
| `44c292be4e4c` | ⒟ | 현: 퀘스트 등록 |
| `ed6ad7318ac9` | ⒟ | 현: 이야기 → 필드 |
| `5d3ba49eccf7` | ⒟ | 구: 그림 3회 · 현: 던전 |
| `859aa864797b` | ⒟ | 구: 로고 정지 · 현: 이야기 → 본편 |
| `46b2238f87a6` | ⒟ | 현: 당구 |
| `6af589a88cf9` | ⒟ | 현: 이야기 → 마을 |
| `33801c1ba14f` | ⒟ | 현: 본편 |
| `8d2828ab8a3f` | ⒟ | 현: 이야기 |
| `c181cae84146` | ⒟ | 현: 이야기 → 본편 |
| `d448aee68157` | ⒟ | 현: 이름 입력 → 부엌 |
| `4df05a4dc452` | ⒟ | 현: 튜토리얼 지도 |
| `6d63b4025bef` | ⒟ | 현: 이야기 |
| `ab64a56b2b44` | ⒟ | 현: 튜토리얼 |
| `65bace1623a7` | ⒟ | 현: 마을 |
| `4decaeed58b1` | ⒟ | 구: `Invalid memory access; address: 0` · 현: RPG 진행 |
| `f7752f9124e8` | ⒟ | 구: 추가 다운로드 · 현: 이야기 → 방 |
| `182fa44210dc` | ⒟ | 현: 이야기(이벤트 처리 AIOOBE 6회 · 진행은 계속) |
| `75e6050fe272` | ⒟ | 현: 화투 |
| `4fcd4b74020e` | ⒟ | 현: 지도 |
| `965eee81e442` | ⒟ | 현: 쿠키 가게 |
| `d9384b388ea5` | ⒟ | 현: 이야기 → 본편 |
| `36acdf213c33` | ⒟ | 현: 이야기 → «저장된 게임이 없으므로 새로 시작» |
| `0e72b6bc12bb` | ⒟ | 구: 추가 다운로드 · 현: 이야기 → 전투 지도 |
| `b1ec149b354c` | ⒟ | 현: 피자 가게 |
| `f12d97040c33` | ⒟ | 구: `StringIndexOutOfBoundsException` · 현: 맞고(그림 2,748) |
| `8a33aafc06a2` | ⒟ | 현: 이야기 → 퍼즐 |

- ⒟ 가운데 5종(`135d` `145f` `1555` `1742` `0cc4`)은 구핀 마지막 화면이 메뉴(⒜ 모양)였다. 현 핀은 본편까지 간다.
- 이 회차에 ⒜ 레시피를 새로 확인한 타이틀은 없다. 그래서 `knownIssues_ko` «진행 요령»은 0줄이다.

### 4. 고친 엔진 벽

#### 4-1. lwc `TextComponent` 가 `setMaxLength` 를 지키지 않았다 (`0c67145b11df`)
- 막힌 지점(main): 이름 칸 둘이 `setMaxLength(4)` 다. 정책 키가 글자를 계속 넣는다. 다섯째 이후 게임 스레드가 `ArrayIndexOutOfBoundsException: 16 > 8` 로 죽는다. 폼이 닫히지 않는다.
- 고침: `keyNotify` 가 넣을 글자를 더하면 상한을 넘을 때 그 키를 버린다(상한이 0 이하이면 제한 없음).
- 근거: AromaWIPI 의 `TextComponent.class` 문자열에 `"Max Length Over"` 가 있다. 실기도 상한을 지킨다.
- 전/후(90초 정책 · 같은 시각 짝): 전 예외 1 · 같은 이름 폼 → 후 예외 0 · 이름을 받고 본편(집 · 대화)으로 간다. 200초 단발도 본편까지 갔다.

#### 4-2. lwc `ShellComponent.setWorkComponent` 가 빈 스텁이었다 (`09a6a300994d`)
- 막힌 지점(main): 프로필 폼이 `setWorkComponent(텍스트 칸)` 다음 키마다 `getNumberOfComponent()` 와 `removeComponent(0)` 을 부른다. 셸에 자식이 없어 키마다 `ArrayIndexOutOfBoundsException: 0 >= 0` 이다(90초에 13회).
- 고침: 작업 컴포넌트를 셸의 자식으로 넣는다. 이미 자식이면 다시 넣지 않는다.
- 한계(ponytail): 두 번째 다른 컴포넌트를 넣으면 앞 것을 바꾸지 않고 자식이 하나 더 생긴다. 바꾸려면 앞 것을 기억할 필드가 필요하다. 필드를 더하면 AOT 하위 클래스의 필드 오프셋이 밀린다(기존 주석과 같은 이유).
- 전/후(90초 짝): 예외 13 → 0 · 서로 다른 화면 5 → 8. 칸 사이 이동(그룹 이름 → 생년월일)이 보인다.

#### 4-3. `Calendar.getInstance(null)` 이 NPE 였다 (`33f3e7669599`)
- 막힌 지점(main): 프로필 확인 «1.응» 의 키 처리에서 게임이 `Calendar.getInstance((TimeZone) null)` 를 부른다. 그 앞에 `TimeZone` 호출은 없다(디버그 추적). 핀은 `NullPointerException: timeZone` 을 던진다. 게임은 1단계(별명)로 돌아간다.
- 고침: `wie_jvm_support::hardening` 에서 그 메서드를 감싼다. 시간대가 null 이면 `TimeZone.getDefault()` 를 넣는다. 핀은 바꾸지 않는다.
- 한계(ponytail): null 이 기본 시간대(여기서는 GMT)라는 가정이다. 다른 시간대를 바라는 게임이면 시계가 밀리지 예외는 아니다.
- 전/후(150초 짝): `timeZone` NPE 6 → 0 · 서로 다른 화면 10 → 14. 후는 두뇌 게임 본편(별자리 · 도형)까지 간다.

#### 4-4. 되돌리면 red
- `text_component::tests::keys_compose_hangul_and_a_rewritten_text_starts_a_new_word`: 상한 검사를 끄면 FAILED.
- `shell_component::tests::work_component_is_a_child_the_guest_can_remove_by_index`: `setWorkComponent` 를 빈 몸으로 되돌리면 FAILED.
- `hardening::tests::calendar_get_instance_with_a_null_zone_uses_the_default_zone` · `every_guard_is_actually_applied`: `java/util/Calendar` 가지를 빼면 둘 다 FAILED.
- 셋 다 확인했다.

### 5. 고치지 못한 것 — 4군집 미달

- 티켓은 «최소 4군집»을 요구했다. 이 회차는 **3군집**이다.
- 넷째 후보로 아래를 끝까지 따라갔다. 셋 다 고칠 근거까지 가지 못했다.
  - `8c71be3ad26d`: 게임이 `getString().charAt(4)` 를 부르는데 문자열이 1자다. 실기의 `TextComponent` 는 `m_td`·`charCount`·`m_cPos` 를 따로 둔다(AromaWIPI 클래스 문자열). 실기에서 `getString()` 이 무엇을 돌려주는지는 바이트코드를 풀어야 한다. 이 맥에는 JRE 가 없어 `javap` 를 쓰지 못했다.
  - `c3057f46c59b`: 54바이트 자료를 읽은 뒤 98을 복사한다. 98이 어디서 왔는지(게임 계산인지, 읽기 반환값인지) 가리지 못했다.
  - `a10a1f02b41b`: `wec/DMInfo` 가 이미지에 이름만 있다. 메서드 이름이 없어 무엇을 구현할지 정하지 못했다. DRM 성격인지도 확인하지 못했다.
- `3185174d2121` 은 `Network.connect()` 실패 경로라 ⒞ 로 두었다. 다만 안내 상자에 글자가 그려지지 않는다(`drawString` 0회). 엔진 탓인지는 가리지 않았다.

### 6. 퇴행

- `setWorkComponent` 를 쓰는 코퍼스 12종(정적 검사: 내부 jar 에 문자열 `setWorkComponent`)을 같은 시각 짝으로 쟀다(정책 90초 · 전 main · 후 이 브랜치).
  - 9종은 판정·화면 수가 같다(차이 ±1).
  - `36b82cb67723`: 화면 수 4 → 2. 두 판 모두 같은 정보 화면에 머문다. 차이는 커서 깜박임이다.
  - `65ef7052f528`: 후가 재실행 1회. 다시 3짝을 쟀다. 전 1·4·0 · 후 1·2·1 회로 양쪽에 다 난다.
- 라이브 LGT 5(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) + 가드 2(`49ade89578c5` `ddd885583b15`): 정책 60초 짝.
  - 판정 7/7 같다. 예외 수 같다. 그림 수 차 ±3% 이내.
  - `1b107b96bf4e` 첫 짝만 화면 수 5 → 2 였다. 그 짝은 전만 재실행 1회였다. 다시 2짝을 쟀다. 둘 다 같은 값(2/2 · 5/5)이다.
- 러너 블록(이 브랜치 release): draw · helloworld ×2 · text PASS.
  - keydraw ×2 는 기본 `--max-ticks` 로 `UNMEASURED`(rc=2 · `max-ticks`)였다. `--max-ticks 100000000000` 로 PASS · rc=0(그림 79 · 55)이다. 3차 r1 과 같은 처리다.

### 7. compat.json

- 방법: 이 브랜치의 `compat.json`(= `origin/main` `f5c02609` 의 파일) 위에 잰 행의 키만 얹었다. 도구는 로컬 스크립트(행을 `sha256` 앞 12자 + `platform` 으로 찾고, `axes.progress` 와 `knownIssues_ko` 만 고친다)다. 다시 쓴 파일은 원본과 바이트 동일한 직렬화(들여쓰기 1칸)다.
- 바뀐 행 11:
  - `axes.progress` 없음 → ok 4 · 없음 → stuck 6(§2).
  - `689c491586b9` `knownIssues_ko` 1줄 추가(⒞).
- 고친 3종의 `progress` 는 바꾸지 않았다(아직 `stuck`). 이 브랜치 빌드의 600초 P·P2 는 시작하지 못했다(§1). §4 의 근거는 90~200초 짝뿐이다.
- `docs/player-updates/2026-10-07-ktf-input-forms-pass.json` 1건(3종).

### 8. 게이트

- `cargo fmt --check` OK.
- `cargo clippy --all -D warnings` · wasm32 · `+beta` 모두 rc=0.
- `RUST_MIN_STACK=4194304 cargo test --all` **732 passed / 0 failed**.
- `npm run build:wasm` rc=0 · `check-engine-contract` 113 pass · `npm run audit` PASSED · `player-data.mjs` OK.

## 후속 군집

| 무엇 | 수 | 첫 벽 | 추정 계급 | 크기 |
|---|---|---|---|---|
| 현 핀 600초 P·P2 재측(⒟ 35 + 고친 3) | 38 | — | 측정 | 묶음당 P ≈ 1시간(jobs 2~3) + P2 |
| 미측정 7 의 L 재측 → P·P2 | 7 | 로컬 L 이 옛 핀 | 측정 | L 600초 + P·P2 |
| `8c71be3ad26d` 숨은 `TextBoxComponent` 의 `charAt(4)` | 1 | `TextComponent.getString()` 의 실기 의미 | ⒝ — lwc | S~M(AromaWIPI 바이트코드 해독이 먼저) |
| `c3057f46c59b` 게임 스레드 `98 > 54` | 1 | `data/text1.txt` 읽기 뒤 `arraycopy` | ⒝ 의심 — `InputStreamReader` | S~M |
| `a10a1f02b41b` `wec/DMInfo` 없음 | 1 | `load_java_class` 실패 | ⒝ 의심 — 플랫폼 클래스 | 미상(메서드 미확인 · DRM 여부 미확인) |
| `3185174d2121` 접속 실패 뒤 빈 안내 상자 | 1 | `Network.connect()` = -1 | ⒞(엔진 여부 미확인) | S |

### 9. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`(이 브랜치 ↔ `origin/main`): BOUNDED 724회/335쌍 + SUFFIX-ATTACHED 36회/16쌍.
- 전부 기존 줄이다 — `compat.json` 의 `title` 값과 `hardening.rs`·`text_component.rs` 의 기존 주석. 이 회차가 더한 줄에서는 0이다(회차 문서·소식·`hardening.rs`·`shell_component.rs` 를 따로 재면 BOUNDED 2쌍 = `hardening.rs` 의 기존 주석 · SUFFIX 0).
- 이 문서는 타이틀을 sha12 로만 적었다.


<!-- corpus-name-inflow v1 subjects=6 tree=d8681e48e0d6540d B=724/335 P=0/0 S=36/16 -->
