## [2026-09-30] 진도 축 신설 — «죽지 않음»이 아니라 «계속 앞으로 가나»를 잰다 · 막힘 원인 엔진 수정 4건 (wie-progression-axis-30min-playthrough-census)

**무엇을**: `playability-census.mjs` 에 `--only progress` 단계(진도 축)를 넣고, 플레이 가능 판정 350종 가운데 102종을 진행 정책 키로 10~30분 돌렸다. 막힌 화면을 직접 보고 ⒜정책 부족 ⒝엔진 탓 ⒞게임 한계(+⒟측정 한계)로 갈랐고, ⒝ 중 넷을 고쳤다.
**왜**: 운영자 지시(2026-09-30) — 「플레이 가능한 게임도 수십 분 직접 플레이해 계속 진도를 나갈 수 있는지 검토하고, 부족하면 조치하라」. 기존 `longplay` 축은 «10분 키 주입 동안 죽지 않았다»만 잰다. 메뉴를 맴돌거나 조용히 멈춰도 `ok` 였다.
**사용자 영향**: 네 가지가 고쳐졌다. ① 텍스트 파일을 한 번에 읽는 게임이 메뉴에서 «게임 시작»을 눌러도 시작되지 않던 문제(라이브 등재 LGT `1b107b96bf4e` 포함). ② 이름 입력칸에 글자가 들어가지 않아 입력 화면에서 영원히 멈추던 문제. ③ 타이틀 화면에서 더 넘어가지 않던 문제. ④ LGT 두 종의 게임 스레드가 죽던 첫 벽. 셸이 보여 주는 등급(`status`)은 바꾸지 않았다(§6).

증적: 프레임 PNG·전사는 저장소 밖(`~/scratch/wie-progress-2026-09-30/`)에만 있다. 이 문서는 sha12 만 적는다. 게임 이름·바이트는 0 이다.

### 1. 진도 축 — 무엇을 재나

| 항목 | 값 |
|---|---|
| 입력 | 진행 정책 키 ~30초 한 주기: 확인(OK·5·1·왼쪽 소프트키 1회)으로 안내·메뉴를 넘기고, 방향키 누르기·길게 누르기, 5 연타. **CLR·오른쪽 소프트키는 쓰지 않는다**(대부분 «뒤로/종료») · 종료하면 3회까지 다시 켠다(DB 유지) |
| 표본 | 10초마다 한 장(`--shot-every 10`) |
| 새 화면 | 16×16 칸 평균 휘도 지문. 지금까지 본 **모든** 화면과 8칸 이상이 32단계 넘게 다르면 새 화면 |
| 판정 | 마지막 새 화면 이후 남은 시간(`stall`)이 실행 시간의 1/3 이상(하한 180초)이면 `stuck` · FAIL 줄이면 `error` · 그 밖 `ok` |
| 확정 | `stuck` 은 **같은 바이너리로 짝 재측(`--as P2`)** 해 둘 다 `stuck` 일 때만 확정. P2 가 없으면 `n/a` — 추측하지 않는다 |
| 레시피 | 제목별 키 파일을 **머리말**로 붙이고 그 뒤에 정책을 잇는다(`--titles` 셋째 열) |

**왜 PNG 해시가 아니라 휘도 지문인가 — 첫 판이 틀렸다.** 첫 판은 PNG sha256 으로 «새 화면»을 셌다. 30분 대표 실행에서 두 종이 거짓 `ok` 로 나왔다. 빈 슬롯 선택창의 커서 빛(`739c7657c1f2`, 해시 27종)과 쿠폰 입력칸의 글자 순환(`13d7e3c21856`, 해시 76종)이다. 둘 다 30분 내내 같은 화면이었다. 지문은 문턱을 두 벌 대조해 골랐다. 24/3 은 슬롯 선택창을 여전히 통과시켰다. 32/8 은 두 종과 얼어붙은 메뉴(`1b107b96bf4e`)를 `stuck` 으로 잡았다. 실제로 진행하는 두 종(1300초·430초까지 새 화면)은 `ok` 로 남겼다.
**알고 남긴 한계**(코드 `ponytail:` 주석): 커서만 판 위를 도는 보드 게임은 `ok` 로 읽힌다(체스 한 종 관측). 반대로 한 화면 안에서 계속 플레이하는 게임(횡스크롤 한 구간, 테니스 연습)은 `stuck` 으로 읽힐 수 있다 — §3 의 ⒟.

**가상 시간이 아니다.** `wie_validate` 는 벽시계로 돈다. 10분 = 실시간 10분이고, 부하가 크면 게임이 느려져 «정체»처럼 보인다. 그래서 짝 재측을 판정 조건으로 넣었다. 호스트 부하(load1)는 P 9~190, P2 29~139 이었다.

### 2. 표본과 결과

전수(350종 × 30분 = 5 jobs 로 약 35시간)는 이 회차 시간(8시간) 안에 들어가지 않는다. 그래서 1차를 이렇게 끊었다.
- **대표 8종 × 30분**: 라이브 LGT 5 `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684` · 가드 2 `49ade89578c5` `ddd885583b15` · KTF `739c7657c1f2`.
- **정체 후보 84종 × 10분**: 기존 전수(`bd2337ff`)의 longplay 스크린샷(20초 간격 30장)에서 끝 5장 이상이 같았거나 도중 종료한 것.
- **대조군 10종 × 10분**: 나머지 playable 258종에서 정렬 순서로 26번째마다.

| 무리 | 수 | P(1차) ok / stuck | P2(짝 재측) | 진도 축 |
|---|---|---|---|---|
| 대표 | 8 | 3 / 5 | 5 모두 stuck | ok 3 · stuck 5 |
| 정체 후보 | 84 | 12 / 72 | 72 중 70 stuck · 2 ok | ok 14 · stuck 70 |
| 대조군 | 10 | 3 / 7 | 7 모두 stuck | ok 3 · stuck 7 |
| **계** | **102** | **18 / 84** | **82 stuck · 2 ok** | **ok 20 · stuck 82 · error 0** |

- 짝 재측에서 뒤집힌 것은 2종이다. `b42e4242866b`(마을 플레이)와 `d5a86e344611`(글자 퀴즈)은 부하 차이로 새 화면 시점이 달라졌다. 나머지 82종은 두 번 다 `stuck` 이었다.
- 대표 `ok` 3종: `49ade89578c5`(가드 · 30분 중 36 화면) · `4ece6eeeaa04`(라이브) · `ddd885583b15`(가드).

**대조군이 말하는 것**: 대조군 10종 중 7종이 `stuck` 이었다. longplay 선별이 정체를 잘 가려내지 못했고, 범용 정책으로 10분 돌리면 **대부분 어딘가에서 멈춘다**. 막힌 곳은 대개 ⒜ — 이 표는 «엔진이 막는다»가 아니라 «범용 키로는 여기까지 간다»로 읽어야 한다. 전수의 진도 `stuck` 비율을 이 102종으로 외삽하지 마라(선별 편향).

### 3. 분류 — 막힌 프레임을 직접 봤다

확정 `stuck` 82종의 분포(프레임 4장 — 시작·1/3·2/3·끝 — 을 한 줄로 이어 붙여 모두 눈으로 봤다. `?` 는 보고도 못 가른 것):

| 분류 | 수 | 뜻 |
|---|---|---|
| ⒜ 정책 부족 | 35 | 특정 키·순서가 필요하다(뒤로 가기·항목 번호·착수·글자 입력) |
| ⒝ 엔진 탓(확인) | 10 | 전사에 엔진 오류가 남았거나 stub 이 원인으로 확인됐다 — 7종에서 벽을 넘겼다(§4) |
| ⒝? 엔진 의심 | 6 | 오류는 있으나 원인 미확정(§8) |
| ⒞ 게임 한계 | 10 | 추가 다운로드·통신 전용 |
| ⒞? | 2 | 통신 추정(다운로드 화면 고정 · 시작 직후 종료) |
| ⒟ 측정 한계 | 11 | 플레이는 계속되나 한 장면 안이다 |
| ? 미조사 | 8 | 흰/검은 화면 · 로고에서 그리기 정지 · 전사에 오류 없음 |

전체 목록(sha12 · 1차/짝 재측 정체 초 · 분류 · 막힌 지점):

| sha12 | 통신사 | P | P2 | 축 | 분류 | 막힌 지점 |
|---|---|---|---|---|---|---|
| b42e4242866b | skt | stuck 280s | ok 190s | ok | d | 마을 플레이(한 화면) |
| d5a86e344611 | ktf | stuck 250s | ok 180s | ok | a | 글자 퀴즈 — 입력 정책 없음 |
| 01e2715ba07a | ktf | stuck 580s | stuck 580s | stuck | b? | 게임 스레드 ArrayIndexOutOfBounds(GameCanvas.run) · java_check_type(unk≠0 → 항상 참) 의심 · 미확정 |
| 0865be217bde | ktf | stuck 480s | stuck 480s | stuck | b? | 가게 이름 입력 뒤 정지 — 입력기 추정 · 미확정 |
| 08aa799c11b0 | skt | stuck 560s | stuck 560s | stuck | a | «선물하기»(통신) 메뉴로만 들어감 — 첫 항목 OK 필요 |
| 09a6a300994d | ktf | stuck 550s | stuck 550s | stuck | b | 프로필 입력(닉네임) — 입력기 stub(1e43 과 같은 원인 추정) |
| 0e72b6bc12bb | ktf | stuck 540s | stuck 540s | stuck | c | «추가 다운로드» 안내 → 종료 |
| 13d7e3c21856 | lgt | stuck 1750s | stuck 1770s | stuck | a | 쿠폰번호 입력 화면(글자는 들어감) — 입력/취소 순서 필요 |
| 155972cac664 | ktf | stuck 560s | stuck 550s | stuck | a | 미니맵 없음 팝업 순환 |
| 1793f87924d4 | ktf | stuck 590s | stuck 590s | stuck | c? | «다운로드중 1/11» 고정 — 추가 다운로드(통신) 추정 |
| 1b107b96bf4e | lgt | stuck 1770s | stuck 1770s | stuck | b | InputStreamReader 짧은 읽기 → 메뉴 키 전부 StringIndexOutOfBounds(«게임시작» 불가) |
| 1cf2e6076079 | ktf | stuck 600s | stuck 600s | stuck | b? | «Data 인스톨에 실패했습니다» 뒤 종료 — KTF DB stream_write 17회 뒤 ListRecords |
| 1e43e2e0055f | ktf | stuck 500s | stuck 510s | stuck | b | 매장 이름 입력 — 입력기 stub |
| 2d66945008c1 | skt | stuck 490s | stuck 490s | stuck | a | 능력치 배분 화면 |
| 2fc792485d91 | ktf | stuck 200s | stuck 340s | stuck | a | 예/아니오 순환 |
| 31c90441f639 | skt | stuck 460s | stuck 260s | stuck | d | 마을 화면(이동 중) |
| 38277d63b0ba | skt | stuck 590s | stuck 590s | stuck | ? | 흰 화면 2장 뒤 그리기 없음(SKT) — 미조사 |
| 3bafa1f1eed2 | skt | stuck 550s | stuck 550s | stuck | a | 배 메뉴 팝업 순환 |
| 41466fc7f709 | ktf | stuck 560s | stuck 560s | stuck | a | 불러오기 «No Data» — CLR 필요 |
| 44a4228cca51 | ktf | stuck 570s | stuck 580s | stuck | a | GAME OVER 화면 순환 |
| 44b6356d13f8 | skt | stuck 600s | stuck 600s | stuck | c? | 시작 직후 종료(SKT · 화면 15장) — 미조사 |
| 4503f7e3a825 | skt | stuck 310s | stuck 400s | stuck | d | 자동 전투 순환 |
| 4d6f8e78cf92 | ktf | stuck 420s | stuck 380s | stuck | a | 미용실 메뉴 순환 |
| 4df05a4dc452 | ktf | stuck 560s | stuck 560s | stuck | a | 도움말(CLR exit) |
| 5028b8a5d19f | ktf | stuck 600s | stuck 600s | stuck | b? | «Data 인스톨에 실패했습니다» 뒤 종료 — 1cf2 와 같은 경로 |
| 55c453b46254 | skt | stuck 550s | stuck 550s | stuck | a | 캐릭터 선택 커서만 이동 |
| 595a443e1c16 | lgt | stuck 530s | stuck 450s | stuck | d | 테니스 연습 모드(플레이 중이나 한 화면) |
| 5a0fa312a793 | ktf | stuck 250s | stuck 260s | stuck | d | 액션 플레이(한 방) |
| 5aa438fbdc87 | ktf | stuck 540s | stuck 540s | stuck | c | «추가 다운로드» 안내 → 종료 |
| 5d3ba49eccf7 | ktf | stuck 590s | stuck 590s | stuck | b | Display.callSerially(r,timeout) stub 이 Runnable 을 버림 → 타이틀에서 정지 |
| 640428a9cf9e | skt | stuck 590s | stuck 590s | stuck | c | «연결상태가 좋지 않습니다» — 통신 전용 메뉴 |
| 65bace1623a7 | ktf | stuck 270s | stuck 260s | stuck | ? | 거의 빈 화면 — 미조사 |
| 689c491586b9 | ktf | stuck 590s | stuck 590s | stuck | c | «네트워크 접속에 실패» 뒤 종료 |
| 69e516bb2ffa | ktf | stuck 290s | stuck 580s | stuck | a | 메뉴 순환 |
| 7089dec0e8df | skt | stuck 580s | stuck 580s | stuck | a | 메뉴 대화상자 순환 |
| 739c7657c1f2 | ktf | stuck 1640s | stuck 1410s | stuck | a | «이어하기» 빈 슬롯(데이터없음) — CLR/새로하기 필요 · 레시피 있음 |
| 73f3a21e981c | lgt | stuck 590s | stuck 590s | stuck | b | LGT ABI Stack 35 미등재 → 게임 스레드 사망(다음 벽 PrintStream 27) |
| 75f002ba70e3 | ktf | stuck 590s | stuck 590s | stuck | c | «추가 다운로드시 통신료» 안내 → 종료 |
| 77c2d0bcd435 | ktf | stuck 580s | stuck 590s | stuck | b? | KTF «Invalid memory access; address 132» — 미조사 |
| 78bd51675574 | skt | stuck 330s | stuck 350s | stuck | d | 필드 이동(한 지도) |
| 7f40060723fc | skt | stuck 460s | stuck 470s | stuck | a | 전략 메뉴 대화상자 |
| 7f73422ba19b | ktf | stuck 590s | stuck 590s | stuck | a | 안내 텍스트 페이지 순환 |
| 85f03ca7389e | skt | stuck 590s | stuck 590s | stuck | b | SKVM com/xce/lcdui/TextComponent 클래스 없음 → 게임 스레드 사망 |
| 916aea39fa36 | skt | stuck 270s | stuck 250s | stuck | d | 보드 화면 진행 |
| 9789fec50f39 | ktf | stuck 590s | stuck 590s | stuck | b? | 게임 스레드 NullPointerException(ac.run) — java_check_type(unk≠0) 의심 · 미확정 |
| 9a2cf5ffc9d3 | skt | stuck 550s | stuck 550s | stuck | b | 게임 스레드 NPE(String.toCharArray on null) — InputStreamReader 수정 뒤 사라짐 |
| 9aa31965cd10 | skt | stuck 530s | stuck 530s | stuck | c | «접속» 필요 메뉴 |
| 9babd9789bae | ktf | stuck 370s | stuck 340s | stuck | a | 대화상자 순환 |
| a10a1f02b41b | ktf | stuck 590s | stuck 590s | stuck | c | «통신 에러» 화면 |
| a30bbe008b5e | lgt | stuck 1780s | stuck 1780s | stuck | a | 정보 화면(게임문의)에서 CLR 없이 못 나옴 — 레시피 있음 |
| a540945188ca | ktf | stuck 570s | stuck 520s | stuck | ? | 빈 갈색 칸 화면 — 미조사 |
| af7d82e5e239 | lgt | stuck 210s | stuck 330s | stuck | d | 어두운 방 플레이 |
| b129770bae26 | ktf | stuck 470s | stuck 580s | stuck | a | 대화상자 |
| b475b6399684 | lgt | stuck 1680s | stuck 1780s | stuck | a | 도움말 페이지(취소:BACK) — CLR 필요 |
| be08d047cbae | lgt | stuck 590s | stuck 590s | stuck | b | LGT ABI Timer 16 미등재 → 게임 스레드 사망(다음 벽 guest TimerTask 11) |
| bf54c05e58a9 | skt | stuck 590s | stuck 590s | stuck | ? | 검은 화면(SKT) — 미조사 |
| c16d6e6ab5c3 | ktf | stuck 570s | stuck 210s | stuck | a | 메뉴·정보 순환 |
| c6cadf75c454 | skt | stuck 590s | stuck 590s | stuck | a | 정보 화면 — CLR 필요 |
| c95bf2740c69 | ktf | stuck 550s | stuck 550s | stuck | a | 아이템 메뉴 순환 |
| ca7fa8ade8ad | ktf | stuck 590s | stuck 590s | stuck | a | 전투 대화상자 순환 |
| ccb45e6b8d80 | skt | stuck 570s | stuck 570s | stuck | a | 능력치 표 순환 |
| cf067c5fd952 | ktf | stuck 510s | stuck 530s | stuck | a | 메뉴 대화상자 |
| cf5249e75df3 | skt | stuck 480s | stuck 530s | stuck | a | 메뉴 대화상자 |
| d1dce4a36141 | skt | stuck 540s | stuck 540s | stuck | a | 대화상자 순환 |
| d1e0badfce82 | ktf | stuck 580s | stuck 580s | stuck | ? | 메뉴 뒤 빈 그라데이션 화면 — 미조사 |
| d234152636c8 | ktf | stuck 550s | stuck 550s | stuck | a | 메뉴 순환 |
| d4188f8ef8c4 | ktf | stuck 440s | stuck 370s | stuck | a | 메뉴 페이지 순환 |
| d448aee68157 | ktf | stuck 550s | stuck 540s | stuck | b | 입력기 getCurrentInputMode/changeCurrentModeToNext 미정의(다음 벽 lwc.Component 필드 x) |
| d607a2622126 | ktf | stuck 470s | stuck 470s | stuck | a | 전략 메뉴 대화상자 |
| d647131cdc8d | ktf | stuck 400s | stuck 340s | stuck | a | 바둑/오목판 — 착수 정책 없음 |
| d9afc4db742c | ktf | stuck 590s | stuck 590s | stuck | ? | 로고(gameloft)에서 정지 — 미조사 |
| e538cbb4e687 | ktf | stuck 590s | stuck 590s | stuck | c | 추가 통신료 안내 화면 |
| e68b1c8aef85 | lgt | stuck 590s | stuck 590s | stuck | ? | 흰 화면 — 미조사 |
| ea35907b22a4 | ktf | stuck 590s | stuck 590s | stuck | ? | 검은 화면 — 미조사 |
| ed6ad7318ac9 | ktf | stuck 290s | stuck 450s | stuck | d | 횡스크롤 플레이(한 구간) |
| f12d97040c33 | ktf | stuck 590s | stuck 590s | stuck | b | InputStreamReader 짧은 읽기(1b107 과 같은 게임의 KTF 판) |
| f2ae515201f2 | skt | stuck 500s | stuck 390s | stuck | d | 전술 전투 화면(진행 중이나 한 화면) |
| f44271803135 | ktf | stuck 590s | stuck 590s | stuck | c | «CONNECTING» 고정 — 통신 |
| f5bd7a91a107 | ktf | stuck 510s | stuck 520s | stuck | a | 데이터 없음 대화상자 |
| f7752f9124e8 | ktf | stuck 540s | stuck 540s | stuck | c | «추가 다운로드» 안내 |
| f81658d4ad1a | skt | stuck 240s | stuck 290s | stuck | d | 퍼즐 플레이 |
| fb7a86ce425b | skt | stuck 310s | stuck 450s | stuck | a | 메뉴 순환 |
| fba094100d13 | skt | stuck 580s | stuck 580s | stuck | a | 불러오기 «NO SAVE DATA» — CLR 필요 |
| fe184f834bc7 | ktf | stuck 260s | stuck 260s | stuck | d | 러너 플레이(한 구간) |


- **⒜ 정책 부족 — 가장 큰 군집은 «뒤로 가기 없음»**: 정보·도움말·빈 «이어하기» 슬롯·«No Data» 불러오기 화면에 들어가면 CLR 없이는 나올 수 없다(`a30bbe008b5e` `b475b6399684` `739c7657c1f2` `41466fc7f709` `4df05a4dc452` `fba094100d13` `c6cadf75c454` …). 둘째는 숫자키가 메뉴 항목을 직접 고르는 게임이다. `1b107b96bf4e` 는 6이 «게임종료»였다. 셋째는 메뉴·대화상자 순환이다.
- **⒞ 게임 한계 — 통신**: «추가 다운로드» 안내 뒤 종료(`0e72b6bc12bb` `5aa438fbdc87` `75f002ba70e3` `f7752f9124e8`), «통신 에러»·«CONNECTING»·«네트워크 접속에 실패».
- **⒟ 측정 한계**: 한 장면 안에서 계속 플레이한다(횡스크롤 한 구간, 러너, 퍼즐, 테니스 연습). 16×16 지문으로는 새 화면이 아니다.
- **?(미조사)**: 흰/검은 화면에서 그리기를 멈춘 것들. 전사에 오류가 남지 않았다.

### 4. ⒝ 고친 것 — 전/후

| 원인 | 고친 곳 | 걸린 타이틀(관측) | 전 → 후 |
|---|---|---|---|
| `InputStreamReader.read(char[],0,759)` 가 **6자**를 돌려준다(10바이트 조각 하나 디코드 후 `break`). EUC-KR 글자가 조각 경계에서 갈리면 U+FFFD 로 깨진다 | `wie-jvm-support/src/hardening.rs` — 핀의 메서드 본문을 바꿔 끼운다(JDK `StreamDecoder` 규칙: 뭔가 읽었고 더 읽으면 막히면 멈춘다 · 진짜 미완성 글자만 남긴다) | `1b107b96bf4e`(LGT) `f12d97040c33`(KTF) `9a2cf5ffc9d3`(SKT) | 메뉴 «게임시작»에서 키마다 `StringIndexOutOfBounds` → 튜토리얼·게임 진입 · 고스톱 판 진행 · 이야기·게임 화면 진행(게임 스레드 NPE 사라짐) |
| `InputMethodHandler.notifyKeyInput` 가 키를 받지 않고 리스너도 부르지 않는 stub | `wie-wipi-java/.../lcdui/input_method_handler.rs` — 멀티탭(ITU E.161)·CLR 삭제·숫자 제한자, `notifyTextChanged(chars,len,pMode)`(javadoc) · `getCurrentInputMode` · `changeCurrentModeToNext` | `1e43e2e0055f` `d448aee68157`(KTF) · `09a6a300994d` `0865be217bde` 같은 형태 | 매장 이름 입력 «최소 1자» 무한 반복 → 게임 본편(2007년 1월 지도) · `d448` 은 벽 두 개를 넘어 다음 벽 `lwc.Component` 필드 `x` |
| `org.kwis.msp.lcdui.Display.callSerially(Runnable,int)` 가 Runnable 을 버리는 stub | `wie-wipi-java/.../lcdui/display.rs` — `timeout` ms 뒤 이벤트 큐에 넣는다(한정 없는 판으로 넘김 → `run()` 은 이벤트 스레드) | `5d3ba49eccf7`(KTF) | 10분 동안 타이틀 3장 → 메뉴·이야기·로딩·던전 플레이 |
| LGT Java ABI 에 `Stack` 35 · `Timer` 16 이 없다 → `Unimplemented … vtable index` 로 게임 스레드 사망 | `wie-lgt/data/lgt_java_abi.toml` — CLDC 1.1 선언 순서로 `empty()Z`=35 · `cancel()V`=16(**호출부 역어셈 안 함 · 순서 유도**) | `73f3a21e981c` `be08d047cbae`(LGT) | 화면상 진전 없음. 다음 벽으로 이동: `PrintStream` 27 · 게스트 `TimerTask` 하위 11 |

**수정 바이너리로 다시 돌린 결과(F, 이 브랜치 · 10분 정책)** — 벽은 넘었다. 그 뒤 ⒜ 로 다시 멈춘 것이 대부분이라 진도 축은 그대로 `stuck` 이다.

| sha12 | 수정 전(P · P2) | 수정 후(F) | 막힌 지점 → 다음 지점 |
|---|---|---|---|
| `1b107b96bf4e` | 30분 새 화면 3 · 1770초 정지 | 10분 새 화면 6 | 메뉴 키마다 예외 → 튜토리얼 → 고스톱 판(판 안 대화상자에서 ⒜) |
| `f12d97040c33` | 새 화면 1 | 새 화면 4 | 메뉴 → 판 진행(같은 게임 KTF 판) |
| `9a2cf5ffc9d3` | 새 화면 4 · 게임 스레드 NPE | 새 화면 7 | 이야기 → 마을(«원정 실패» 대화 순환 ⒜) |
| `1e43e2e0055f` | 매장 이름 «최소 1자» 순환 | 새 화면 8 | 이름 입력 → 본편 지도 → 번호 선택 화면(⒜) |
| `d448aee68157` | 새 화면 4 | 새 화면 6 | 입력기 메서드 없음 두 번 → `lwc.Component` 필드 `x` 없음(⒝ · §8) |
| `5d3ba49eccf7` | 10분 새 화면 1(타이틀) | 새 화면 9 | 타이틀 → 메뉴·이야기·로딩 → 던전 전투(처치 0/7 → 1/7 · ⒟) |
| `73f3a21e981c` | 로고 · `Stack` 35 | 로고 · `PrintStream` 27 | 화면상 진전 없음(다음 벽) |
| `be08d047cbae` | 안내 화면 · `Timer` 16 | 안내 화면 · 게스트 `TimerTask` 11 | 화면상 진전 없음(다음 벽) |
| `9789fec50f39` | 새 화면 1 | 새 화면 1 | 변화 없음 — 이번 수정과 무관(§8 `java_check_type` 의심) |

**넣었다가 뺀 것 하나**: `TimerTask.cancel()Z`=11 행을 넣으면 `be08d047cbae` 가 둘째 키에서 **호스트 panic**(`Expected object, got Int`)으로 바뀌었다. 행을 빼고 그 벽은 남겼다. 순서 유도만으로 넣지 않을 행의 실례다.

**«되돌리면 red»**: 네 수정 모두 단위 시험이 있다. 수정을 빼면 각각 실패한다(실측).
- `reader_read_tests::one_read_fills_the_buffer_from_a_resource_stream` — 핀 본문이면 540 대신 8을 돌려준다.
- `reader_read_tests::a_character_split_across_requests_is_carried_whole`(게이트② F4 로 추가) — 앞에 1바이트를 덧대 2자 요청의 경계가 늘 한글 음절 가운데 걸린다. 보류를 끄면(`complete_prefix` 우회) `x�〕�가…` 로 red 다.
- `input_method_handler::tests::keys_reach_the_listener_as_text`
- `display::tests::timed_call_serially_queues_the_runnable_after_the_timeout` — stub 이면 큐가 늘지 않는다.
- ABI 두 행: `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 목록에 추가했다(빈 슬롯이면 panic).

### 5. 대표 타이틀 심층 플레이

| sha12 | 무엇 | 정책 30분(P · P2) | 레시피 + 정책(F · 수정 바이너리) |
|---|---|---|---|
| `a30bbe008b5e` | 라이브 LGT | stuck · 새 화면 2 · 정보 화면(«게임문의»)에 갇힘 ⒜ | **ok** · 30분 새 화면 25 · 새로하기 → 집 안 → 마을 지도(1510초까지 새 화면) · 키 1626/1971 |
| `739c7657c1f2` | KTF | stuck · 새 화면 5 · 빈 «이어하기» 슬롯 ⒜ | **ok** · 30분 새 화면 21 · 새로하기 → 이야기 → 마을 → 장비 창(1240초까지 새 화면) · 키 1890/1890 PASS |
| `1b107b96bf4e` | 라이브 LGT | stuck · 새 화면 3 · ⒝(§4) | 수정 뒤 10분 새 화면 6 · 고스톱 판 안 대화상자 ⒜ |
| `13d7e3c21856` | 라이브 LGT | stuck · 새 화면 5 · 쿠폰 입력칸 ⒜ | 10분 새 화면 3 · 같은 자리 |
| `b475b6399684` | 라이브 LGT | stuck · 새 화면 5 · 도움말 ⒜ | 10분 새 화면 2 · 같은 자리 |
| `4ece6eeeaa04` | 라이브 LGT | ok · 새 화면 3(1330초까지) — 체스판 커서(§1 한계) | – |
| `49ade89578c5` | 가드 KTF | ok · 새 화면 36(1300초까지) | – |
| `ddd885583b15` | 가드 LGT | ok | – |

레시피가 있는 두 종은 **막힌 원인이 정책(⒜)이었음을 레시피가 증명했다**. 같은 바이너리 계열로 정책만 30분 돌리면 `stuck` 이고, 저장소의 레시피로 문을 열면 30분 내내 새 화면이 나온다.

**60분은 채우지 못했다.** 대표 8종은 정책으로 30분(P), 확정 재측으로 30분(P2)을 돌렸다. 레시피가 있는 두 종(`a30bbe008b5e` `docs/keys/battlemonster-village.keys` · `739c7657c1f2` `docs/keys/ktf-739c7657c1f2-town.keys`)은 레시피 + 정책으로 30분을 수정 바이너리로 돌렸다. 스테이지 2 도달과 «세이브 후 이어하기»는 **이 도구로 판정하지 않았다**. 화면 지문은 «어느 스테이지인가»를 모른다. 세이브 쓰기 횟수도 아직 `wie_validate` 가 내지 않는다(§8).

### 6. compat.json · 셸

- `axes.progress` 는 **선택 축**으로 넣었다(`scripts/player-data.mjs` 의 `EXTRA_AXES`). 잰 행에만 싣는다. ★**공개 값은 `ok`·`stuck`** 이다(`EXTRA_AXIS_VALUES` · 계약 문서 progress 행). census `error` 도 `stuck` 으로 싣고, `n/a` 는 키를 싣지 않는다. 여섯 축의 `no`·`unknown` 어휘는 쓰지 않는다.
- ★**어휘를 바꾼 이유(게이트② F3)**: 첫 판은 `stuck→no` 로 실었다. 그런데 셸 otterpebble main(#1244 · 6e9ba983)의 `PROGRESS_UI` 는 `ok`·`stuck` 만 그리고, 모르는 값이면 칸을 숨긴다(`progressOf`). 그대로 배포됐다면 **막힌 행은 전부 안 보이고 «계속 진행» 행만 보였을 것이다.** 좋은 쪽만 보이는 편향이다. 셸 가져오기(`compat-import.mjs`)는 여섯 축만 검사하므로 `stuck` 값을 그대로 통과시킨다.
- `status` 규칙은 **바꾸지 않았다**. 제안(운영자 문안 결정): 확정 `stuck` 을 `limited` 로 내리지 **말 것**. 확정 `stuck` 의 다수가 ⒜(범용 정책의 한계)와 ⒟(측정 한계)다. 게임이 멈춘 것이 아니다. 내린다면 ⒝·⒞ 로 분류된 것만, 그리고 `knownIssues_ko` 문장과 함께 내려야 한다.
- `docs/player-data/compat.json` 갱신 방법: 이 회차에 잰 102행에만 `axes.progress` 를 넣었다. 값은 P/P2(`f44c6bcd` 기준 · 수정 전) 판정 **그대로**다. ★공개 분포는 **`ok` 20 · `stuck` 82** 로, §2 표의 진도 축과 같다. 첫 판은 레시피 F 2종을 `ok` 로 덮어 22/80 이었다. 레시피는 «정책 키»라는 축 정의 밖이라 덮지 않는다(F8). origin/main 병합 뒤 main 의 compat.json 위에 같은 102행을 다시 적용했다(#423·#421 착지분 보존). 나머지 행·`enginePin`·`status` 는 그대로다.

### 7. 퇴행 확인

- 라이브 LGT 5 + 가드 2 + `739c7657c1f2` 를 30초 A 프로브(전수와 같은 인자)로 다시 돌렸다. `wv-fix`·최종 바이너리 모두 **8/8 PASS → PASS**, `content` 참이다. 그림 수는 load1 ~70~130 에서 오르내린다.
- 러너 블록(AGENTS.md): `draw_j2me` `helloworld_ktf` `helloworld_lgt` `text_j2me` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 릴리스 바이너리에서 `UNMEASURED · stop max-ticks` 였다. 판정 절차 ⑴~⑶을 따랐다. `--max-ticks 1000000000` 이면 **수정 전(`wv-main`)·후 모두** PASS, 27/27 키, 55장, 마지막 화면 내용 있음이다. 기본 5천만 틱을 빠른 릴리스 빌드가 먼저 태운 것이고, 이 diff 와 무관하다.
- 네 관문: `cargo fmt --check` · `clippy --all -D warnings` · `clippy --target wasm32-unknown-unknown -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` · `cargo +beta clippy --all -D warnings` 모두 rc=0 이다.

### 8. 후속(남긴 군집)

| 군집 | 수 | 계급 | 크기 | 근거 |
|---|---|---|---|---|
| «Data 인스톨에 실패» 뒤 종료 — KTF DB `stream_write` 17회(1.7MB) → `close` → `MC_dbListRecords(fd, buf, 1)` 에서 실패로 판정 | 2 (`1cf2e6076079` `5028b8a5d19f`) | ⒝ 추정 | M | 두 종 모두 census 에서 `playable` 이지만 실제로는 매 부팅 3초 안에 종료한다 — **등급 거짓 양성** |
| KTF `java_check_type(…, unk≠0)` 가 항상 참 → 게임 스레드 AIOOBE/NPE 의심 | 2 (`01e2715ba07a` `9789fec50f39`) | ⒝ 의심 | L(역어셈 필요) | `interface.rs` 의 `// TODO is it correct?` |
| SKVM `com/xce/lcdui/TextComponent` 없음 | 1 (`85f03ca7389e`) | ⒝ | M | `NoClassDefFoundError` · 게임 스레드 사망 |
| LGT ABI `PrintStream` 27 · 게스트 `TimerTask` 11 | 2 (`73f3a21e981c` `be08d047cbae`) | ⒝ | S~M | 호출부 역어셈으로 행 확정 필요(순서 유도만으로 넣은 11 은 panic) |
| 입력기 다음 벽 `lwc.Component` 필드 `x` | 1 (`d448aee68157`) | ⒝ | S | `Field xI@182 not found` |
| 진행 정책 v2 — 정체 뒤에만 CLR 1회 | ⒜ 36 중 다수 | 도구 | M | 고정 키 스크립트로는 «막혔을 때만 뒤로»를 못 쓴다 |
| 세이브 쓰기 횟수·RMS 를 진도 축에 | – | 도구 | S | 과제 문안의 측정 항목 중 이번에 안 낸 것 |

### 9. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` 결과(15파일): 유입 360쌍(BOUNDED) · 판단 필요 19쌍(SUFFIX-ATTACHED). **이 diff 가 새로 들인 이름은 0**이다. 두 바구니 모두 이 diff 가 건드린 파일에 **원래 있던** 이름이다 — `docs/player-data/compat.json` 의 공개 제목(이 회차는 `progress` 키만 더했다), `lgt_java_abi.toml`·`jvm_support.rs`·`hardening.rs` 의 기존 주석. 새로 쓴 주석·문서·소식은 sha12 만 쓴다(`git diff origin/main` 의 `+` 줄에서 3자 이상 한글 연속은 메뉴 문구 «게임시작» 1건뿐).

### 10. 게이트② 반려 처분(-fix 회차)

| # | 처분 |
|---|---|
| F1 main 충돌 | `git merge origin/main`(upstream 동기 repo라 rebase 는 하지 않는다). compat.json 은 main 쪽을 받고 102행 `progress` 를 `runs.jsonl` 판정으로 다시 적용했다 — 102행 · main 착지분 보존 |
| F2 연번 | 0390 → **0393**(`--next-serial`). AGENTS.md·worklog 참조를 갱신했다 |
| F3 어휘 | 공개 값 `ok`·`stuck`(§6). 분포 ok 20 · stuck 82 |
| F4 경계 시험 | 위 §4 새 시험 · 개악 red |
| F5 짝이 FAIL | `progressAxis` 는 짝이 실제로 움직였을 때만 `ok`, FAIL/없음이면 `n/a` 다. selftest 1줄을 더했다 |
| F6 재생성 | import(`fromCensus`)가 `n/a` 면 키를 싣지 않는다. selftest 가 `progress: 'n/a'` 입력을 본다 |
| F7 timeout 전 | 0 ms · 10분 두 호출을 같이 넣고, 앞엣것이 들어간 뒤에도 뒤엣것은 밖인지 본다. sleep 삭제·stub 복귀 둘 다 red(테스트 시계는 읽을 때마다 8 ms 가는 가상 시계라 «20 ms 뒤»는 구별력이 없었다) |
| F8 22/80 | 레시피로 덮지 않는다 → 20/82(§6) |
| F9 인자 순서 | 주석에 javadoc 경로와 측정 로그 `(53, 1)` 을 달았다. delete 의 `len=1` 은 javadoc «처리할 문자의 갯수»와 대조했다 |
| F10 0 반환 | **안 고쳤다** — `InputStream.read(b,off,len>0)` 계약은 1바이트 이상 또는 -1 이라 0 은 깨진 스트림에서만 난다. 거기서 다시 돌면 무한 루프가 되므로 돌려받은 값을 그대로 둔다 |
| F11 동시 실행 | `--only progress` 의 기본 `--jobs` 를 ncpu/4 로 내렸다(명시값은 종전대로 ncpu 상한) |


<!-- corpus-name-inflow v1 subjects=15 tree=8024b2bb137f381d B=821/360 P=0/0 S=49/19 -->
