## [2026-10-06] 진도 3차 (4회차) — `ad`·`ae` 60종 측정 · KTF 접속 경로 벽(Net 슬롯 34 · `MC_utilInetAddrInt`) · lwc 키 잡기(`grabKey`·`setGrabKeyListener`) (wie-progression-wave3-remaining-playable-and-stuck-34-r4)

3회차(`docs/report/0447` · PR #480)의 이어하기다. 남은 몫은 `chunk.ad`…`ai` 159종의 600초 진도 판정, 막힘 계급, 엔진 벽 수정이었다.
**부분 완료다.** `ad`·`ae` 60종을 쟀고 `af`…`ai` 99종은 못 쟀다(§1). 엔진 벽은 2군집(2종)을 고쳤다(§3·§4).

### 1. 측정 — 측정 엔진 = `1bc1317c` 빌드(release `wie_validate`)

- 착수(10-05 21:5x) 시 #480 은 **미착지**였다(`gh pr view 480` → OPEN). 그때 `origin/main` 은 `91e3ced8` 이었고, `1bc1317c..91e3ced8` 에는 엔진 변경이 없다(#479 = 스크립트·문서 5파일). 그래서 3회차의 `1bc1317c` 빌드를 그대로 이어 썼다(같은 `--out` → 잰 것은 건너뜀).
- 회차 도중(10-06 새벽) #480·#481·#482·#483 이 착지했다. 그 넷이 이 회차가 잰 행에 닿는 곳(§5)은 표에 표시했다.
- 조건은 3회차와 같다: 정책 v2 600초 + 재기동 120초 · P 뒤 `p2list` 가 고른 P2 짝 · 스윕을 `build-slot run --long` 임대로 감쌌다(안쪽 실행은 맨 명령) · census 호스트 잠금 · 착수 전 `host-load-guard --status --recovered` rc=0 · `nohup` 0.

| 단계 | 시각 | jobs | load1 (시작 → 끝) | idle (시작 → 끝) | 임대 |
|---|---|---|---|---|---|
| P `ad`(30) | 21:53–23:53 | 3 | 4.7 → 7.7 | 49% → 72% | ① |
| P2 `ad`(17) | 23:53–00:55 | 3 | 7.7 → 21.2 | 71% → 0% | ① |
| P `ae`(30) | 00:55–02:55 | 3 | 21.2 → 10.0 | 1% → 54% | ① |
| 고친 벽 짧은 확인(단발) | 03:2x–03:5x | 1 | — | — | short |
| P2 `ae`(13) | 03:20–04:08 | 3 | 12.9 → 9.0 | 6% → 15% | ② |
| 고친 2종 P·P2(이 PR 빌드) | 04:08–04:32 | 2 | 8.9 → 8.5 | 78% → 46% | ③ |

- 타이틀별 `load1`(census 열): `ad` 최소 10 · 중앙 15 · 최대 68 / `ae` 최소 10 · 중앙 16 · 최대 166.
- 02:55 에 P `ae` 가 끝난 자리에서 드라이버를 멈췄다(자기 pid). 남은 시간에 P2 `ae` 와 고친 2종 판정을 넣으려고 임대를 다시 잡았다(②③). ②는 다른 레인의 long 임대가 먼저 있어 22분 기다렸다.

| 묶음 | 모집단 | 잰 수 | ok | stuck | error | n/a | 미측정 사유 |
|---|---|---|---|---|---|---|---|
| `ad` | 30 | 30 | 21 | 8 | 1 | 0 | — |
| `ae` | 30 | 30 | 18 | 11 | 1 | 0 | — |
| `af`…`ai` | 99 | 0 | — | — | — | — | 회차 시간 |
| **합** | 159 | 60 | 39 | 19 | 2 | 0 | |

- stuck 은 P2 짝이 같을 때만 stuck 이다. `ae` 에서 P stuck 12종 중 `249e655147a1` 1종은 P2 가 움직여(stall 210 → 50) ok 다.
- 위 표는 `1bc1317c` 빌드 값이다. `85e94babc247`(`ad` error)은 이 PR 빌드 값으로 compat 에 들어간다(§4). `974e0df9ab1e`(`ad` stuck)은 compat 에 넣지 **않았다**(§5).

### 2. 막힘 계급(stuck·error 21 · 600초 화면을 직접 봤다)

| 계급 | 수 | sha12 | 근거 |
|---|---|---|---|
| ⒜ 정책 한계 | 16 | `ad`: `cbf36fee9f63` `a20c2044305c` `75e6050fe272` `e09aca27c132` `d5e996a53118` `182fa44210dc` `0f9e1026724d` · `ae`: `36acdf213c33` `0392263fbb85` `4fcd4b74020e` `3ccc6cf147d2` `4288d8c1c6ac` `d9384b388ea5` `b1ec149b354c` `965eee81e442` `4b6eaa69e056` | 메인 메뉴 · 타이틀(«press any key») · 설정 화면 · 스토리 대사 창. 전부 정상으로 그려지고 정책 키가 그 화면을 못 넘는다 |
| ⒝ 엔진 벽 — 고침 | 1 | `85e94babc247` | §4 |
| ⒝ 엔진 벽 — main 이 고침 | 1 | `974e0df9ab1e` | «설치(또는 실행) 공간이 부족합니다. 2103KB…» 로 20초에 끝난다. 이 빌드의 `MC_dbListDataBase` 답이 1 MB 였다. #482 가 16 MB 로 올려 main 에 착지했다(§5) |
| ⒝ 엔진 벽 — 남김 | 2 | `f2280c6699a0` · `568c339a8c07` | `f228`: `07_OK` 에서 `EventQueue.getNextEvent` 안 `Invalid memory access`(P·P2 같다). `568c`: 화면에 글자만 그려지고 바탕 그림이 없다(600초 내내) · 700초 검은 화면. 렌더 쪽 벽으로 보이나 원인은 안 쟀다 |
| ⒜? PixelOp 타이틀 | 1 | `23919eb33365` | 600초 화면이 회색 잡음 위 메뉴다. 3회차가 찾은 «픽셀 연산을 거는 KTF 5종» 중 하나이고, 이 빌드에는 #480(KTF 블릿 픽셀 연산)이 없다 ⇒ 판정이 #480 착지 후 달라질 수 있다 |

- 3회차가 남긴 `30c7bd6fb01b`(⒞) · `96dc32e781d3`(⒝): `30c7` 은 §3 에서 다뤘다. `96dc` 는 남김이다 — `paint` 안 `Invalid memory access` 주소 `0x71790820` 이 SVC 스텁 영역(`0x7100xxxx`) 근처이고 PC 도 네이티브 함수(`0x71008e4a`)라 어느 네이티브가 엉뚱한 포인터를 읽는지부터 가려야 한다. 이 회차는 거기까지 못 갔다.

### 3. ⒞ `30c7bd6fb01b` — KTF 접속 경로가 «죽음 → 접속 대기»로

**증상.** «게임빌 매니아에 접속하시겠습니까?»에서 «예» → Net 표 슬롯 34 `Unimplemented` 로 40.8초에 죽었다(3회차).

**확인한 것.** 슬롯 34 의 r0 은 이미지 안 문자열 `kt68wipiwicgsfr.magicn.com:27090`(덤프 `0x17b3f0`) · r1 = `0xff` 다. 호출부는 정적으로 못 찾았다 — `ktf-image-sweep.py slots` 는 `+0x40` 까지의 오프셋만 잡고(이 이미지 1,099 호출 중 0x88 없음), 그 문자열을 가리키는 리터럴 워드도 이미지에 없다.
그래서 동적으로 쟀다. 슬롯 34 가 `-1`(`MC_netSocket` 과 같은 «네트워크 없음» 규약)을 돌려주게 하고 같은 키 재생을 돌리니 게임이 그 다음 호출로 갔다:

```
Net slot 34(…, 0xff) -> -1 (no network)
inet_addr("218.145.70.36")          ← util 슬롯 4 MC_utilInetAddrInt — 여기서 또 Unimplemented 였다
MC_utilHtons(20106)
stub MC_netConnect(0x1215cd, 0x19af44)   ← 기존 스텁: 1틱 뒤 콜백에 M_E_ERROR
```

⇒ 슬롯 34 의 반환값은 이 경로에서 **분기에 쓰이지 않는다**(무엇을 돌려줘도 `MC_utilInetAddrInt` 로 간다). 3회차 검수가 지적한 «반환 규약을 모르는 채 성공을 돌려주면 거짓 ok» 위험은 여기서는 성립하지 않는다. 그래도 «성공»이 아니라 `-1` 을 골랐다(네트워크가 없다는 사실과 같은 쪽).

**고친 것.**
- KTF Net 슬롯 34 → `-1`(로그에 문자열을 남긴다).
- `MC_utilInetAddrInt`: LGT 의 `inet_addr`(WIPIC 904 · 점 네 개 십진 → 네트워크 순서, 아니면 `INADDR_NONE`)를 `wie-wipi-c` 의 `api::util` 로 옮겨 KTF util 슬롯 4 와 함께 쓴다. LGT 는 같은 함수를 가리킬 뿐이다(LGT 시험 `inet_addr` 단언 그대로 통과).

**결과.** 600초 판정에서 더 죽지 않는다(P: `PASS` · deadline 720초). 화면은 «네트워크 · 서버 접속 중…» 에 머문다 — 접속이 불가능한 환경에서의 정직한 상태다. 계급은 ⒞ 그대로다(죽음 → 대기).
- ★#481(`KTF finally 재던짐 무한 루프 — 연결 실패 뒤 멈춤`)이 이 회차 도중 main 에 착지했다. 이 판정 빌드에는 없다. 그 수정이 «서버 접속 중» 대기를 바꾸는지는 다음 회차 main 빌드 측정이 답한다.

| 개악 | 결과 |
|---|---|
| 슬롯 34 줄 삭제 | `test_net_slot_34_fails_without_network` FAILED — `Unimplemented(… Net, function 34 …)` |
| util 슬롯 4 를 종전 `gen_stub` 으로 | 같은 시험 FAILED — `Unimplemented("4: MC_utilInetAddrInt")` |

### 4. ⒝ `85e94babc247` — lwc `ShellComponent` 키 잡기

**증상.** 결과 화면(«조금만 더!!!»)에서 «다시하기»(NUM1)를 누르면 `Method grabKey(I)V not found from org/kwis/msp/lwc/ShellComponent` 로 죽었다(P·P2 같은 `56_NUM1`).

**확인한 것.** 메서드를 넣으면 다음 줄에서 `setGrabKeyListener(Lorg/kwis/msp/lwc/GrabKeyListener;Ljava/lang/Object;)V` 가 없어 죽는다. 콜백 서명은 게임의 `client.bin` 문자열에서 읽었다: `grabKeyNotify (IILjava/lang/Object;)Z`.
게임이 리스너를 거는 이상 «안 죽게만»(빈 메서드)으로는 잡은 키가 리스너에 안 간다. 그래서 전달까지 넣었다.

**고친 것.**
- `ShellComponent.grabKey(I)` · `ungrabKey(I)` · `setGrabKeyListener(GrabKeyListener, Object)`.
- 상태는 `net.wie.ShellCard` 의 static 필드에 둔다(잡은 키 `[Z` 128칸 · 리스너 · param). lwc 클래스에 인스턴스 필드를 더하면 LGT AOT 하위 클래스의 필드 오프셋이 밀린다는 그 파일의 기존 규율(`focus` 와 같은 자리)을 따랐다. 키 색인은 `key & 127` — WIPI 키 코드(-16…57)끼리 겹치지 않는다.
- `ShellCard.keyNotify`: 잡은 키면 `grabKeyNotify(type, key, param)` 를 먼저 부르고, 리스너가 받으면 거기서 끝낸다. 아니면 종전 경로(EventListener → 셸 `keyNotify`).
- `GrabKeyListener` 인터페이스에 그 추상 메서드를 적었다(`EventListener` 와 같은 모양). 빈 인터페이스로는 `invoke_virtual` 이 `NoSuchMethodError` 를 낸다(시험에서 실측).
- ponytail: 잡기 집합은 셸마다가 아니라 하나다 — 키를 받는 것은 화면 맨 위 셸 하나뿐이다.

**결과.** 600초 판정에서 더 죽지 않는다(P: `PASS` · deadline 720초). 짧은 확인(80초)에서 «다시하기» 뒤 판이 이어지고 콤보(×2·×3)가 오르는 화면을 봤다.

| 개악 | 결과 |
|---|---|
| `ShellCard.keyNotify` 의 잡기 분기를 끔 | `grabbed_key_goes_to_the_grab_key_listener_first` FAILED(«a grabbed key stops at the listener») |

### 5. 회차 도중 착지한 main 변경과 이 회차 행

| main 변경 | 이 회차 행에 닿는 곳 | 처리 |
|---|---|---|
| #480 KTF 블릿 픽셀 연산 | PixelOp 5종 중 `503d5d2f196b`(`ad` ok) · `23919eb33365`(`ae` stuck) | 표시만. `1bc1317c` 값으로 넣었다 |
| #482 KTF 여유 저장 공간 1 → 16 MB | `974e0df9ab1e` | compat 에 **넣지 않았다**(main 값 유지). 고치기 전 빌드의 stuck 을 적으면 착지한 수정을 덮는다 |
| #481 KTF finally 재던짐 | `30c7bd6fb01b` 의 «서버 접속 중» 대기와 관계 가능 | 표시만 |
| #483 LGT CreateImage | 이 회차 행은 전부 KTF | 없음 |

### 6. compat 행 단위 비교

`origin/main` `2e0ac37e` ↔ head(`sha256+platform` 키 · axes 하위 키 단위):

```
rows 429 -> 429 changed by field set {"axes.progress":59}
  (none)->ok 40 · (none)->stuck 19
compat-revert: OK — main 에서 받은 행을 받기 전 값으로 되돌린 필드 0 (fork bdf58a8e → mb 2e0ac37e) · 착지 기준 바뀐 행 59
```

- 59 = `ad`·`ae` 60 − `974e0df9ab1e`(§5). ok 40 = `1bc1317c` ok 39 + `85e94babc247`(이 PR 빌드). stuck 19 = `1bc1317c` stuck 18(`974e` 제외) + error 1(`f2280c6699a0` · error 는 stuck 으로 적는다).
- `30c7bd6fb01b` 은 3회차가 이미 stuck 으로 적었고 이 PR 빌드 판정도 stuck 이라 바뀐 행이 아니다.

### 7. 이어하기

- `cd ~/scratch/w7prog/r3 && ~/orchestrator-live/bin/build-slot run --long -- bash drive.sh` — 같은 `--out` 이라 잰 것은 건너뛰고 `af` 부터 돈다. ★그때의 main(#480·#481·#482·#483·이 PR 포함)을 다시 빌드하고 `drive.sh` 의 `BIN`·`--pin` 을 바꿔라. 그러면 `ad`·`ae` 와 `af`… 의 빌드가 갈린다 — 표에 빌드 sha 를 나눠 적어라.
- `23919eb33365`(PixelOp) · `30c7bd6fb01b`(#481 관계) 는 새 main 빌드로 다시 재면 판정이 바뀔 수 있다.
- 남긴 ⒝: `f2280c6699a0`(getNextEvent 안 메모리 접근) · `568c339a8c07`(바탕 그림 없음) · `96dc32e781d3`(paint 안 메모리 접근).
