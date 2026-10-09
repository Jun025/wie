## [2026-10-10] 진도 6차 — 진행 요령 2차: 1차가 못 끝낸 40종 · 요령 23줄 (wie-progression-wave6-play-tips-round2-remaining-40)

**무엇을**: 진도 5차(0482)가 시간 안에 못 끝낸 40종을 다시 몰았다. 내역은 미해결 10 · 미시도 5 · 600초 미측정 2 · 레시피 막힘 8 · 팁 없음 11 · ⒝⒞ 4다. 대상은 23행이다. 13행은 600초 판의 마지막 3분의 1 이 실제 놀이였다. 10행은 놀이에 «들어가는 법»이 이 핀에서 화면으로 확인됐다. 이 23행에 `playTips_ko` 1줄씩을 달았다. 짝(P2)으로 확정된 9행은 `axes.progress` 를 `stuck` → `ok` 로 바꿨다. 엔진은 고치지 않았다.

**왜**: 운영자 지시(2026-09-30 · 10-09 «이어서 필요한 후속 작업들을 완전자율주행으로»). 수십 분을 해도 계속 나아갈 수 있게 하고 안내를 최신으로 맞춘다. 출처는 0482 §1 «Acceptance 미달»이다.

**사용자 영향**: 23개 게임의 지원 현황 상세 «진행 요령»에 어떤 키로 들어가고 넘어가는지 한 줄이 생긴다. 그중 9개는 진도 배지가 «자동으론 막힘»에서 «계속 진행»으로 바뀐다(셸 `PROGRESS_UI` · otterpebble `e0a38e9be`). 요령대로 했을 때 10분 동안 두 번 다 실제 놀이가 이어졌다. 나머지 14개는 배지가 그대로다.

### 1. 측정 조건

- 엔진: `origin/main` `7ca47380` 의 release `wie_validate`. 1차 핀 `74c9d915` 이후 SKVM `XTextField` 입력(0489), LGT Clet 타이머 수거(0484·0490), KTF 카드 겹침(0491)이 들어왔다.
- 탐색: 서브에이전트 3개가 그룹마다 에뮬레이터를 하나씩만 돌렸다(동시 3). 단발 실행은 150초 이하이고 타이틀당 5판 이하다. 그룹 a 50분, c 62분, b 72분이 걸렸다.
- 레시피 길이를 보정했다. 1차 레시피 다수가 실제로는 약 575초에서 끝났다. 키 간격 기본값은 `--action-secs` 0.6초인데, 그 뒤로는 census 정책 키가 메뉴를 열었다. 이번에는 34개 레시피를 모두 ≥ 820초(600 + 재시작 120 + 여유)가 되게 마지막 루프 줄로 채웠다.
- 판정: `bash scripts/census-drive.sh … --only progress --progress 600 --titles <34> --as P`(정책 v2). 4종(`1eaa92092bee` `fe76e641bb3d` `87b04639cdfe` `fb80e97cbc57`)은 이 핀에서 A·B·L 을 새로 쟀다(`--only probe` → `--only long`). 나머지 30종은 1차 `outF` 의 A·B·L 을 옮겨 썼다.
- ★«ok» 를 그대로 믿지 않았다. 34종 모두 t400~t600 화면을 직접 봤다(0482 와 같은 기준). 화면 새로움 판정은 커서나 창 깜박임을 진행과 가르지 못하기 때문이다. 실측 예:
  - `0266ca417880` 은 P `ok`(stall 70)인데 서버 선택 확인창의 커서만 600초 동안 오갔다.
  - `8c6821fab969` 도 `ok` 였지만 옥상 조작 안내에 멈춰 있었다.
- P2(짝): P `ok` 이면서 화면도 놀이인 12종만 `--as P2` 로 다시 쟀다. census 는 11종을 `ok`·1종을 `stuck` 으로 냈다. 눈으로 9종이 놀이였다(§4).
- 자원:
  - long(4종): 23:38 첫 임대에서 대기 0. 탐색과 겹쳐 `--jobs 1`.
  - P(34종): 00:19~02:43 · `--jobs 3` · census-drive 6회.
  - P2(12종): 02:45~03:57 · `host-load-guard` 포화(rc=1)라 `--jobs 2`.
  - `nohup` 0. P2 첫 기동을 맨 `&` 로 띄웠다가 같은 분에 그 4개 프로세스(드라이버·임대·census·에뮬레이터 2)를 pid 로 거두고 도구 백그라운드로 다시 띄웠다. 그 2종의 부분 결과는 지웠다. 끝난 뒤 내 `wv_7ca47380` 프로세스는 0이다.
- 레시피: 비커밋 `game_lab/recipes-progress/<sha12>.keys` 40개(키 이름만 · 머리 주석에 이유). 증적(화면 낱장 모음·탐색 결과 표)은 저장소 밖 `~/orchestrator/reports/evidence/wie-progression-wave6-play-tips-round2-remaining-40/` 에 있다.

### 2. 전건 표(40)

판정:
- **팁 · 놀이 확인**: 레시피 P 의 마지막 3분의 1 이 놀이다.
- **팁 · 들어가는 법**: 놀이에 들어가는 길이 이 핀의 판에서 화면으로 확인됐다. 다만 600초를 놀이로 버티지는 못한다. 요령 문장은 «들어가는 법»만 말한다.
- **들어감 · 요령 안 실음**: §3 기준에 따라 싣지 않았다.
- **막힘 · 이유**: 넘어가지 못한 자리를 화면 근거로 적었다.
- ⒝ = 엔진 첫 결함 지점. ⒞ = 서버.

| sha12 | 통신사 | 1차 | 이번 판정 | P(600초) | 근거(마지막 3분의 1 · 또는 탐색) | 근거 화면 | 요령 |
|---|---|---|---|---|---|---|---|
| `0392263fbb85` | KTF | 팁 없음(600초 미유지) | 팁 · 놀이 확인 | ok stall=0 | t400-600 tactical battle field continues | `tail-0392263fbb85-P.png` | 처음 메뉴에서 아래 방향키로 «새로하기»를 고른 뒤 확인 키로 이야기를 넘기면 전투가 시작돼요. |
| `0f9e1026724d` | KTF | P 막힘(340) | 팁 · 놀이 확인 | ok stall=0 | t400-540 story + castle walking · t550-570 menu/slot (census escapes) · t580-600 field | `tail-0f9e1026724d-P.png` | 처음 화면에서 확인 키를 누르고 새로 하겠냐고 묻는 창에서 «예»를 고른 뒤 이야기를 넘기면 성 안을 방향키로 다닐 수 있어요. |
| `1045007289d8` | LGT | P 막힘(260) | 팁 · 놀이 확인 | ok stall=0 | t400-600 village walking · one shop dialog t430 | `tail-1045007289d8-P.png` | 생일은 숫자 키로 넣고 확인 키를 누른 뒤 «게임시작»을 고르고, 이야기는 오른쪽 소프트키로 넘기면 마을을 방향키로 걸어 다닐 수 있어요. |
| `1eaa92092bee` | LGT | 600초 미측정 | 팁 · 놀이 확인 | ok stall=30 | t400-600 side-scroll field fighting · minimap t570+ | `tail-1eaa92092bee-P.png` | 처음 메뉴에서 «게임시작»에 확인 키를 누르고 이야기를 확인 키로 넘기면 들판에서 방향키로 움직이며 싸울 수 있어요. |
| `4503f7e3a825` | SKT | 미해결 | 팁 · 놀이 확인 | ok stall=70 | t400-600 walking town rooms · town map t530-590 | `tail-4503f7e3a825-P.png` | 처음 메뉴에서 확인 키를 눌러 시작하고, 싸움에서는 확인 키를 계속 누르면 기술이 나가며 싸움이 이어져요. |
| `49f2734f17d8` | KTF | 미해결 | 팁 · 놀이 확인 | ok stall=40 | t400-600 city map building · month reports 10월→11월→2월 | `tail-49f2734f17d8-P.png` | «게임시작»에서 아래 방향키로 «무한모드»를 고르고 확인 키로 시작한 뒤 숫자 7 키를 누르면 방향키로 도로를 깔 수 있고, 결산 창은 CLR 키로 닫아요. |
| `4b6eaa69e056` | KTF | 미시도 | 팁 · 놀이 확인 | ok stall=10 | t400-600 Go-Stop hands one after another · money changes | `tail-4b6eaa69e056-P.png` | 처음 메뉴에서 «새로하기»로 시나리오 모드를 시작하고 확인 키로 상대를 고르면 방향키와 확인 키로 화투를 칠 수 있어요. |
| `976141a9525b` | SKT | 팁 없음 | 팁 · 놀이 확인 | ok stall=160 | t400-600 side-on village field, character walks along the street | `tail-976141a9525b-P.png` | 처음 화면부터 확인 키를 계속 눌러 이야기를 넘기면 마을 길을 방향키로 걸어 다닐 수 있어요. |
| `af7d82e5e239` | LGT | 팁 없음 | 팁 · 놀이 확인 | ok stall=30 | t400-600 room investigation, the examine lines change (machine, glass, floor) | `tail-af7d82e5e239-P.png` | 처음 안내와 동의 창에서 확인 키를 누르고 «게임시작»에서 «처음부터»를 고른 뒤 이야기를 넘기면 방 안을 방향키로 둘러보며 확인 키로 조사할 수 있어요. |
| `c6cadf75c454` | SKT | 팁 없음 | 팁 · 놀이 확인 | ok stall=10 | t400-600 village and forest walking | `tail-c6cadf75c454-P.png` | 처음 메뉴에서 «1 시작하기»와 난이도를 확인 키로 고르고 이야기를 넘기면 마을을 걸어 다닐 수 있어요. |
| `cbf36fee9f63` | KTF | 미해결 | 팁 · 놀이 확인 | ok stall=100 | t400-600 claw-machine rounds one after another, money falls 25,000→23,000 · two short menu boxes | `tail-cbf36fee9f63-P.png` | 처음 메뉴에서 «새로하기»를 고르고 이야기를 넘긴 뒤 마을에서 오른쪽 방향키를 한 번 눌러 인형 가게로 들어가면 확인 키로 인형뽑기를 할 수 있어요. |
| `e09aca27c132` | KTF | 미해결 | 팁 · 놀이 확인 | ok stall=40 | t400-600 Go-Stop hands, results screens, back to new hands | `tail-e09aca27c132-P.png` | 메뉴에서 숫자 1 키로 «새로하기»를 고르고 생일을 숫자 키로 넣은 뒤 «저장하시겠습니까?»에서 왼쪽 방향키를 누르고 확인 키를 누르면 상대를 골라 화투를 칠 수 있어요. |
| `5aa438fbdc87` | KTF | 팁 없음 | 팁 · 놀이 확인(P stuck) | stuck stall=380 | t400-600 walking one village square, NPC talk boxes · census stuck 380 (no new screen) | `tail-5aa438fbdc87-P.png` | 처음 화면에서 확인 키를 계속 눌러 이야기를 넘기면 마을에서 방향키로 걸어 다닐 수 있어요. |
| `739c7657c1f2` | KTF | 팁 없음 | 팁 · 들어가는 법 | stuck stall=410 | t400-600 town with «아직 수행할 수 없습니다» box ↔ quest list window | `tail-739c7657c1f2-P.png` | 처음 메뉴에서 아래 방향키로 «새로하기»를 고르고 저장 칸을 확인 키로 정한 뒤 초기화 확인은 숫자 1 키로 넘기면 거리를 방향키로 다닐 수 있어요. |
| `db8ef04a6504` | KTF | 팁 없음 | 팁 · 들어가는 법 | stuck stall=230 | t400-600 inventory window open | `tail-db8ef04a6504-P.png` | 처음 화면들을 확인 키로 넘기고 «모험의 시작»을 고른 뒤 대화를 넘기면 들판에서 방향키로 움직일 수 있어요. |
| `ea35907b22a4` | KTF | 팁 없음 | 팁 · 들어가는 법 | ok stall=90 | t400-600 the same field battle, units swing but nothing ends (as in round 1) | `tail-ea35907b22a4-P.png` | 처음 메뉴에서 위 방향키로 «새로시작»을 고른 뒤 확인 키로 이야기를 넘기면 지도와 전투가 이어져요. |
| `d5e996a53118` | KTF | P 막힘(400) | 팁 · 들어가는 법 | ok stall=180 | t400-600 settings screen (sound/vibration/speed) after a return to the title | `tail-d5e996a53118-P.png` | 새 게임 확인 창은 커서가 «NO»에 있으니 왼쪽 방향키로 «YES»를 고른 뒤 확인 키를 누르면 이야기와 1단계가 시작돼요. |
| `0266ca417880` | LGT | ⒝ | 팁 · 들어가는 법 | ok stall=70 | census run never passes the server-select dialog (서버 간 이동 불가 확인창 · 예/취소 cursor toggles t30..t710) — recipe timing-sensitive | `tail-0266ca417880-P.png` | 처음 화면부터 확인 키를 누르다가 서버 선택 확인 창이 나오면 왼쪽 방향키로 «예»를 고른 뒤 확인 키를 눌러야 이야기로 넘어가요. |
| `1e43e2e0055f` | KTF | 미해결 | 팁 · 들어가는 법 | stuck stall=510 | t400-600 map tutorial box «깃발이 있는 위치로 이동하세요» repeating (P stuck 510) | `tail-1e43e2e0055f-P.png` | «게임시작»을 고른 뒤 ＊ 키로 이야기를 건너뛰고, 매장 이름 칸에서 숫자 2 키를 천천히 두 번 눌러 글자를 넣은 다음 확인 키를 누르면 미션을 고르고 지도로 들어가요. |
| `2f5246006bd8` | SKT | 미해결 | 팁 · 들어가는 법 | stuck stall=460 | t400-600 battle map, same frame (unit cursor waits for a command the loop never gives) | `tail-2f5246006bd8-P.png` | «새로하기»에서 이름 화면은 숫자 5 키와 위 방향키로 넘기고, 그 뒤로는 방향키 없이 확인 키만 누르면 임무를 받아 전투 지도로 들어가요. |
| `ec2f8f2e02a2` | SKT | ⒝ | 팁 · 들어가는 법 | stuck stall=460 | t400-600 «이것 선택할래?» pick box over a list, same frame | `tail-ec2f8f2e02a2-P.png` | 처음 화면에서 확인 키로 새 게임을 고르고 «예»를 누른 뒤, 이름 화면에서 숫자 키로 글자를 넣고 확인 키를 누르면 이야기가 시작돼요. |
| `f12984cd0d37` | SKT | 미해결 | 팁 · 들어가는 법 | stuck stall=230 | t400-600 unit/command screen, cursor only | `tail-f12984cd0d37-P.png` | 저장 칸 목록 맨 위 줄에서 오른쪽 방향키로 «이어하기»를 «새로하기»로 바꾼 뒤 확인 키를 누르면 이야기가 시작돼요. |
| `4288d8c1c6ac` | KTF | 미시도 | 팁 · 들어가는 법 | ok stall=0 | t400-580 title menu with «데이터가 존재하지 않습니다» box (load with no save) · t590 help text · t600 stage | `tail-4288d8c1c6ac-P.png` | 처음 메뉴에서 «시나리오 모드»를 고른 뒤 «이어하기»가 아니라 «새로하기»에 확인 키를 누르면 이야기를 지나 무대가 시작돼요. |
| `8c6821fab969` | KTF | 팁 없음 | 들어감 · 요령 안 실음 | ok stall=0 | t400-600 rooftop tutorial «점프 후, 상 키를 누르면 앞뒤 흔들기» never cleared (P ok is the figure's idle motion) — 옥상 조작 안내(점프 뒤 위 키)를 못 넘긴다 — 점프 키를 못 찾았다. 들어가는 법만 실으면 막힌 길로 안내한다 | `tail-8c6821fab969-P.png` |  |
| `a20c2044305c` | KTF | 팁 없음 | 들어감 · 요령 안 실음 | stuck stall=360 | t400-600 character status window open — 들어가는 길이 확인 키 두 번 — 자동 키가 이미 하는 일이라 요령이 아니다 | `tail-a20c2044305c-P.png` |  |
| `52edc54e2b23` | SKT | P 막힘(500) | 들어감 · 요령 안 실음 | stuck stall=420 | t400-600 forest village field, character pinned at the left edge, same screen — 확인 키만 — 같은 사유 | `tail-52edc54e2b23-P.png` |  |
| `fe76e641bb3d` | LGT | 600초 미측정 | 들어감 · 요령 안 실음 | ok stall=190 | t400-600 one room, the same talk line repeating — 확인 키만 — 같은 사유 | `tail-fe76e641bb3d-P.png` |  |
| `6e93f26fa2f5` | SKT | P 막힘(450) | 들어감 · 요령 안 실음 | stuck stall=380 | t400-600 lost-match screen «1.재도전» with a tip line, no key moves it — 확인 키만 — 같은 사유 | `tail-6e93f26fa2f5-P.png` |  |
| `66959afab216` | SKT | P 막힘(340) | 들어감 · 요령 안 실음 | stuck stall=200 | t400-600 field with the status window open the whole time — «NewGame» 확인 키만 — 같은 사유 | `tail-66959afab216-P.png` |  |
| `bf54c05e58a9` | SKT | 팁 없음 | 들어감 · 요령 안 실음 | ok stall=60 | t400-520 town walking with talk lines · t530-600 menu then «Quest» window stays open to the end — 확인 키만 — 같은 사유 | `tail-bf54c05e58a9-P.png` |  |
| `e3276ce8557c` | SKT | P 막힘(480) | 들어감 · 요령 안 실음 | stuck stall=480 | t400-600 race track barely moves, «1.계속하기» pause window t560 — 확인 키만 — 같은 사유 | `tail-e3276ce8557c-P.png` |  |
| `a0436eb1ddbb` | SKT | 미시도 | 들어감 · 요령 안 실음 | stuck stall=360 | t400-600 item window «빈슬롯 입니다» open over the park map — 확인 키만 — 같은 사유 | `tail-a0436eb1ddbb-P.png` |  |
| `fb80e97cbc57` | SKT | 미시도 | 들어감 · 요령 안 실음 | stuck stall=220 | t400-570 two corridor rooms back and forth · t580-600 equipment window — 확인 키만 — 같은 사유 | `tail-fb80e97cbc57-P.png` |  |
| `87b04639cdfe` | LGT | 미해결 | 들어감 · 요령 안 실음 | stuck stall=220 | t400-600 rhythm play screen, lanes light on keys but no notes and score stays — no song progress visible — 두 키 시험은 한 번호 + «아니오» 로 넘지만 P 600초에 음표가 안 내려온다 — 연주가 확인되지 않았다 | `tail-87b04639cdfe-P.png` |  |
| `3eb73c20bbae` | SKT | P 막힘(470) | 막힘 · 이유 | — | 사진 찍기 판 뒤 가게 화면 · 확인 키는 «보유 금액 부족» 창만 · CLR·방향키로 못 나간다 · 돈 버는 길 미발견 · 예외 0 | `explore-3eb73c20bbae-b1.png` |  |
| `1f0d7e81336a` | SKT | 미해결 | 막힘 · 이유 | — | «동료 2인 선택» 에서 좌우는 화살표를 옮기나 OK(짧게·길게·연타)·숫자·소프트키·＊·＃ 모두 고르기 표시가 안 생긴다 · CLR 는 앞 창 · 예외 0 · 스텁 흔적 없음 | `explore-1f0d7e81336a-b2.png` |  |
| `21ffd1c61ebd` | LGT | 미해결 | 막힘 · 이유 | — | Chapter 1 컷신이 넘겨지지 않아(NUM0 «SKIP» 무효) 150초 안에 «캡틴관리» 튜토리얼 화면까지 못 간다 — 아이콘 시험 못 함 | `explore-21ffd1c61ebd-r2b.png` |  |
| `14a62a8521a0` | SKT | ⒝ 의심 | ⒝ 엔진 | — | 캐릭터 생성 «이름 입력» 상자 · 숫자 멀티탭·OK·방향키 무반응, 글자 0 · 캐릭터는 움직인다 · stderr 0 · 스텁 목록에 XTextField 없음 — 0489 가 고친 경로가 아니다 | `explore-14a62a8521a0-b1.png` |  |
| `3ccc6cf147d2` | KTF | 미시도 | ⒝ 엔진 | — | 캠프 지도 뒤 능력치·? 상자 화면(약 30초)에서 정지 · 모든 키 무반응 · stderr 첫 줄 «Uncaught exception … java.lang.NullPointerException at BnB/MasterCard.run()V»(1회) · «무한모드» 는 시나리오 클리어 전 잠김 · 같은 NPE 가 4288d8c1c6ac 에서도 나지만 그쪽은 계속 간다 | `explore-3ccc6cf147d2-r2b.png` |  |
| `0fdf45ec169f` | LGT | ⒞ | ⒞ 서버 | — | 이용 안내 뒤 «접속 중입니다… 서버에 연결중입니다» 에서 t020~t060 진행바만 · 키 무반응 · 예외 0 | `explore-0fdf45ec169f-smoke.png` |  |

- 합계: 팁 · 놀이 확인 13 · 팁 · 들어가는 법 10 · 들어감 · 요령 안 실음 11 · 막힘 · 이유 3 · ⒝ 2 · ⒞ 1 = 40.
- 1차 대비:
  - ⒝ 3 가운데 둘은 풀렸다. `ec2f8f2e02a2` 는 0489 이후 이름 화면이 숫자 멀티탭을 받는다. `0266ca417880` 은 0490 으로 7분 힙 고갈의 주인이 정해졌다. 이번 판은 서버 선택 창에서 멈춰 그 7분에 닿지 않았다. 그래서 고갈이 다시 나지 않는지는 이 회차가 보지 못했다.
  - `14a62a8521a0` 은 그대로다.
  - 미시도 `3ccc6cf147d2` 는 새 ⒝ 다.

### 3. 요령 싣는 기준(티켓 할 일 3)

티켓은 «600초 생존은 게임 실력 축일 수 있다» 고 했다. 그래서 «들어가는 법»만 확인된 행에 요령을 싣는 것이 이용자에게 맞는지 판정했다. 실은 조건은 넷이다.

1. 들어가는 길을 이 핀(`7ca47380`)의 판에서 화면으로 봤다. 그룹 a 가 1차 화면으로 정한 3종은 이번 P 의 t0~t400 화면으로 다시 확인했다(`head-*.png`).
2. 그 길에 자동 키가 못 고르는 한 수가 있다. 예: 특정 메뉴 항목, 기본 커서가 «아니오»·«NO» 인 확인창, 숫자 키, ＊ 키, 방향키를 누르면 안 되는 자리. 확인 키만 누르면 되는 길은 싣지 않는다. 정책 키가 이미 그렇게 누르므로 그 줄은 요령이 아니다. 해당 9종은 §2 표의 «확인 키만».
3. 문장은 «들어가는 법»만 말하고 «계속 된다»고 주장하지 않는다.
4. 그 길 끝이 우리가 이유를 모르는 벽이면 싣지 않는다. 막힌 길로 안내하기 때문이다. 해당: `8c6821fab969` 옥상 조작 안내 · `87b04639cdfe` 연주 미확인.

진도 배지(`progress`)는 이 기준으로 바꾸지 않는다. 배지를 바꾸는 근거는 §4 의 짝뿐이다.

### 4. P2(짝) · `axes.progress`

| sha12 | P | P2 | P2 마지막 3분의 1(눈) | `progress` |
|---|---|---|---|---|
| `0392263fbb85` | ok stall=0 | ok stall=160 | t400-600 «불러오기» slot list, cursor only (after a lost battle, as round 1) | stuck 그대로 |
| `0f9e1026724d` | ok stall=0 | stuck stall=300 | t400-600 equipment upgrade window «재련석이 부족합니다» ↔ castle hall | stuck 그대로 |
| `1045007289d8` | ok stall=0 | ok stall=30 | t400-540 village, talking to villagers · t550-600 magic shop buy window | stuck → **ok** |
| `1eaa92092bee` | ok stall=30 | ok stall=40 | t400-550 field fighting · t560-600 minimap | stuck → **ok** |
| `4503f7e3a825` | ok stall=70 | ok stall=90 | t400-600 town rooms walking · town map t510-570 | stuck → **ok** |
| `49f2734f17d8` | ok stall=40 | ok stall=10 | t400-600 city map, month reports 8월→9월→… advance | stuck → **ok** |
| `4b6eaa69e056` | ok stall=10 | ok stall=0 | t400-600 Go-Stop hands one after another | stuck → **ok** |
| `976141a9525b` | ok stall=160 | ok stall=150 | t400-600 village street, character walks (same as P) | stuck → **ok** |
| `af7d82e5e239` | ok stall=30 | ok stall=120 | t400-600 «CHARACTER» profile screen paging | stuck 그대로 |
| `c6cadf75c454` | ok stall=10 | ok stall=100 | t400-600 village walking | stuck → **ok** |
| `cbf36fee9f63` | ok stall=100 | ok stall=110 | t400-600 claw-machine rounds, money 25,600→23,400 | stuck → **ok** |
| `e09aca27c132` | ok stall=40 | ok stall=0 | t400-600 Go-Stop hands, results, new hands | stuck → **ok** |

- 바꾼 행 **9**: P `ok` · P2 `ok` 이고 두 판의 마지막 3분의 1 이 모두 놀이였다. 근거 화면은 `tail-<sha12>-P.png` · `tail-<sha12>-P2.png` 다.
- 안 바꾼 행 3:
  - `0f9e1026724d` 는 P2 가 `stuck` 이다.
  - `0392263fbb85` · `af7d82e5e239` 는 P2 가 `ok` 이지만 화면은 «불러오기» 커서와 인물 소개 넘기기였다. 짝이 진행을 확인하지 못했다.
- ★공개 값 계약을 이 PR 에서 함께 고쳤다(`docs/contracts/featurephone-public-data.md` §progress). 종전 문장은 «레시피로 잰 결과는 싣지 않는다»였다. 이제 예외가 하나 있다. 같은 행에 그 레시피를 옮긴 `playTips_ko` 가 함께 실리고, 짝 두 판이 `ok` 이며, 사람이 화면을 확인했을 때다. 요령을 따라 하는 이용자에게 참인 배지만 싣는다는 뜻이다.
- 한계: census `report` 의 `progressAxis()` 는 P 가 `ok` 이면 P2 없이 `ok` 를 낸다. 그래서 이번 out 디렉터리로 `report` 를 다시 돌리면 눈으로 걸러 낸 행(P `ok` 인데 메뉴였던 7)도 `ok` 가 된다. 이 회차의 compat 는 `report` 재생성이 아니라 행 단위 수정이다. 같은 out 으로 재생성하지 마라.

### 5. ⒝ 엔진 벽(할 일 4 · 주인 확인)

- `ec2f8f2e02a2` · `0266ca417880`: 주인 있음. 0489(`wie-skt-xtextfield-key-input-name-screen`) · 0490·0492.
- `14a62a8521a0`: 주인 없음.
  - 0489 §1 이 «이 회차에서 재지 않았다»고 남긴 그 이름 화면이다.
  - 첫 결함 지점: 캐릭터 생성 «이름 입력» 상자에서 숫자 멀티탭·OK·방향키를 눌러도 글자가 0이다. 캐릭터는 움직인다.
  - stderr 0줄 · 예외 0 · `stub_hits` 에 `XTextField` 가 없다. ⇒ 0489 가 고친 경로가 아니다.
- `3ccc6cf147d2`: 주인 없음. 이전 회차(0452·0470)는 정책 한계(⒜)로만 적었다.
  - 첫 결함 지점: 캠프 지도 뒤 능력치·? 상자 화면(약 30초)에서 정지하고 이후 모든 키에 무반응이다.
  - stderr 첫 결함 줄은 `Uncaught exception … java.lang.NullPointerException at BnB/MasterCard.run()V`(1회)다.
  - 같은 NPE 가 `4288d8c1c6ac`(같은 계열)에서도 나지만 그쪽은 계속 간다. 그래서 이 NPE 하나가 정지의 원인이라고는 말하지 못한다.
  - 클래스는 jar 가 아니라 AOT `client.bin` 에 있다. 정적으로 읽으려면 ARM 역어셈이 필요해 이 회차 범위 밖이다.
- 두 건 모두 worklog 카드(§6)로 남긴다. 엔진 코드 변경 0.

### 6. compat · 소식 · 셸

- `compat.json`: 23행에 `playTips_ko` 1줄씩 추가(23행 모두 원래 키 없음). 9행 `axes.progress` `stuck` → `ok`(§4). `knownIssues_ko`·`status` 는 무변경.
- census `HAND_TIP` 에 같은 23줄을 넣었다. 다음 census `report` 가 같은 값을 다시 만든다.
- 키 이름은 기존 요령 19줄의 표기를 따랐다(«확인 키» · «숫자 n 키» · «왼쪽/오른쪽 소프트키» · 방향키). 셸에서 확인한 것(otterpebble `e0a38e9be`):
  - 키보드 지도 라벨(`lib/keyboard-layout.ts` `VALUE_LABEL`)은 «확인» · «왼쪽 소프트키» · «＊» 다.
  - 화면 버튼은 «OK» · «CLR» 다.
  - 그래서 CLR 은 «CLR 키», 별표는 «＊ 키» 로 적었다.
- `docs/player-updates/2026-10-10-progression-play-tips-round2.json`(improvement · 23 sha).

### 7. 후속

- `14a62a8521a0` 이름 상자 입력 경로 · `3ccc6cf147d2` 30초 정지 — worklog 카드.
- 막힘 3(`3eb73c20bbae` 가게 돈 · `1f0d7e81336a` 동료 확정 · `21ffd1c61ebd` 넘길 수 없는 컷신)은 카드로 만들지 않았다. 엔진 결함 근거가 없고(예외 0), 플레이 방법을 모른다는 «질문»이라서다.
