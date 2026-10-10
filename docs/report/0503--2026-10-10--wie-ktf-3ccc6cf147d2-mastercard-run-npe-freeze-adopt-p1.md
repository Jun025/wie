## [2026-10-10] KTF `3ccc6cf147d2` 캠프 지도 뒤 정지 — paint 가 게임 스레드의 이미지 적재 한가운데로 들어갔다 (wie-ktf-3ccc6cf147d2-mastercard-run-npe-freeze-adopt-p1)

**무엇을**: 0502 §5 가 주인 없이 남긴 «캠프 지도 뒤 능력치·? 상자 화면 정지»를 쟀고 고쳤다. KTF·LGT 에서 paint 도 키처럼, 다른 게스트 스레드가 «코드 한가운데서» 잘려 있으면 그 스레드가 호스트 호출에 닿기를 기다린 뒤 시작한다(상한 250ms · 0430 의 키 장치 그대로). 그리고 명령 예산이 `svc` 위에서 다한 스레드도 «잘린» 것으로 센다.
**판정**: null 은 엔진이 채워야 할 값이 **아니다.** 게임 자신의 paint 경로가 그 배열을 비운다. 엔진 탓은 «그 paint 를 게임 스레드의 적재 한가운데에 끼워 넣은 것»이다(§2).
**사용자 영향**: `3ccc6cf147d2`·`4288d8c1c6ac` 에서 시나리오 모드의 무대 소개가 더는 멈추지 않는다(짝 재측 main 정지 5/28 ↔ 0/28 · §3). 다른 KTF·LGT 게임은 paint 가 그 순간에 걸렸을 때만 그만큼 늦게 그려진다(§4).

증적: `~/orchestrator/reports/evidence/wie-ktf-3ccc6cf147d2-mastercard-run-npe-freeze-adopt-p1/`. 타이틀은 sha12 로만 적는다.

### 1. 재현 — 두 타이틀 모두 같은 경합이다

- 0502 는 `3ccc6cf147d2` 만 멈추고 `4288d8c1c6ac` 은 «같은 NPE 인데 계속 간다»고 적었다. 이 회차 첫 실행은 **반대**였다: `4288d8c1c6ac` 이 같은 화면(약 t030)에서 멈췄고 `3ccc6cf147d2` 은 무대까지 갔다.
- ⇒ 타이틀 차이가 아니라 시점 차이다. 둘은 같은 게임의 화면 크기 두 판이다(같은 클래스 · 같은 경로).
- 정지의 모양: `MasterCard.run()` 스레드가 NPE 로 끝난다 → 화면은 그 프레임에 머문다 → 남은 스레드가 돌기만 해 `ticks` 가 빨리 닳는다(`stop: max-ticks` · 약 34초).

### 2. 원인 — 스크래치 계측(커밋 0 · 증적 `scratch-probes.md`)

1. **어디서 throw**: KTF AOT 코드의 배열 읽기 도우미(`0x160ea8` · 배열이 0 이면 `java/lang/NullPointerException` 을 이름으로 던진다)가 `0x112e34` 에서 불렸다. 그 함수는 메서드 표로 찾은 `TalkImgLoad()V` 다.
2. **무엇이 null**: `this.imgSpeechChar` 배열이다. 같은 함수가 그 배열을 새로 만들고(`0x112b30`) 칸 0·1·2 를 읽은 **다음**, 칸 3 을 읽을 때 필드가 0 이었다.
3. **누가 0 으로**: 필드 블록 워드를 엔진에서 감시했다. 0 을 쓴 것은 **다른 스레드**의 `TalkImgRelease()V`(`0x112fd0`)다. 호출 사슬은 `paint → UpdateScene → UpdateGameScene → RenderBriefing → ReleaseImg → TalkImgRelease` 다. 적재 쪽 사슬은 `MasterCard.run → … KeyEventBriefing → InitStage → LoadStage → InitCharacter → TalkImgLoad` 다. 두 쓰기의 간격은 7~11ms 였다.
   - 기각: 필드 오프셋 충돌(런타임 필드 표에서 두 필드는 `0x1c4`·`0x1ec` 로 다르다) · 과수거(객체는 등록돼 있고 필드 블록도 그대로다).
4. **왜 paint 가 그 사이에 들어갔나** — 두 겹:
   - paint 는 0463 부터 «도는 동안» 다른 스레드를 붙든다. 그러나 **시작하기 전에** 다른 스레드가 코드 한가운데서 잘려 있는지는 보지 않았다. 키만 기다렸다(0430).
   - 그 기다림만 넣은 빌드는 여전히 멈췄다(12회 중 5 · 다른 묶음 12회 중 2). 계측하니 paint 가 시작할 때 `others_preempted()` 가 거짓이었다. 원인은 `ArmCore::run_function` 이다. 명령 예산이 **`svc` 위에서** 다하면 엔진은 `Svc` 로 멈추고, 그 양보에는 `preempted` 가 서지 않았다(0430 은 `Yield` 만 셌다). 그 스레드는 호스트 호출을 아직 시작도 안 한 채, 프레임 한가운데서 잘려 있다.

### 3. 처방과 짝 재측

| 자리 | 바꾼 것 |
|---|---|
| `wie-core-arm` `ArmCore::run_function` | 예산이 다해 양보할 때 멈춘 이유가 `Svc` 여도 `preempted` 를 세운다 |
| `wie-midp` `Display::paint_serialized` | `handlePaintEvent` 앞에서 잘린 스레드를 최대 250ms 기다린다. 바깥 키 처리가 이미 붙들고 있으면 기다리지 않는다 |
| 〃 `wait_for_guest_threads` | 키와 paint 가 같은 기다림을 쓴다(상한 `KEY_WAITS_FOR_GUEST_MS`) |

- NPE 를 삼키지 않았다. 게임의 적재·해제 코드는 그대로 돈다. 바뀐 것은 paint 가 «언제» 시작하느냐뿐이다.
- 되돌리면 red: `a_budget_that_runs_out_on_an_svc_counts_as_preempted`(core 변경 제거 → FAILED) · `paint_waits_for_a_thread_sliced_out_mid_code`(paint 기다림 제거 → FAILED).

짝 재측(release · `origin/main` `a3da330d` ↔ 이 브랜치 · 3개 동시 · `host-load-guard --recovered` rc=0 후 시작 · 증적 `pair-3ccc-4288.tsv`). 정지 판정 = `MasterCard.run` NPE 와 `paints` 급감·`frozen_tail_steps` > 0 이 함께 나온 실행:

| 묶음 | 키 | main 정지 | 이 브랜치 정지 |
|---|---|---|---|
| A · 40초 | OK 12번(2초 간격) | `3ccc` 0/8 · `4288` **3/8** | 0/8 · 0/8 |
| B · 90초 · `--max-ticks 1e9` | `game_lab/recipes-progress/3ccc6cf147d2.keys` | `3ccc` **1/6** · `4288` **1/6** | 0/6 · 0/6 |
| 합 | | **5/28** | **0/28** |

- 동작 불변(정지 안 한 실행끼리): 묶음 B `4288` paints main 1,190~1,270 ↔ 이 브랜치 1,118~1,242 · `3ccc` main 1,283~1,555 ↔ 1,543~1,570. 둘 다 무대에서 걷고 물풍선을 놓는다.
- 묶음 A 의 이 브랜치 `3ccc` 1회가 `stop: max-ticks` 인데 39.9초 · paints 696 · NPE 0 이다. 정상 진행이 기한 직전에 예산을 다 쓴 것이다(main 정지 실행은 약 34초 · paints 약 470).

### 4. 퇴행 — KTF 16 + LGT 11 짝(기본 27키 `--inject` · main·이 브랜치 동시 · 증적 `sweep-27.tsv`)

- 결과(PASS/UNMEASURED) 27/27 동일 · `last_frame_content` 27/27 동일 · paints 비 중앙값 **0.995**.
- 따로 본 3종:
  - `2c2ba3b84b98`(226 → 20): 두 빌드 모두 27키가 8번째에서 종료 메뉴를 고르는 경우와 아닌 경우로 갈린다. 3회씩 재측 = main clean exit 2/3 · 이 브랜치 2/3.
  - `6eb93824daf8`(644 → 609): 3회 재측 main 771~781 · 이 브랜치 706~997 — 잡음.
  - `a540945188ca`(**436 → 350 · 4회 재측 모두 −18~22%**): 진짜다. paint 362번 중 80번이 44~56ms 기다렸다(게임 스레드가 프레임 하나를 그만큼 쉬지 않고 계산한다 · 증적 `a540945188ca-paint-waits.txt`). 2초 간격 화면은 두 빌드가 같다(도움말 화면 · `a540945188ca-main-vs-fix.png`). 빠진 paint 는 게임 프레임이 아니라 «계산 도중의 덧 paint» 다. 단, 이 판정은 그 한 화면에서만 본 것이다.
- 어느 쪽 변경이 그 감소를 냈나(`a540945188ca` 4회): main 429~448 · core 만 438~456 · paint 기다림만 373~381 · 둘 다 346~360. ⇒ paint 기다림이다. 그리고 core 변경이 기다림을 더 정확하게(조금 더 자주) 만든다.
- 전체 runner 줄(AGENTS §Definition of Done): `draw_j2me` · `helloworld_ktf` · `helloworld_lgt` PASS · `keydraw_ktf`/`keydraw_lgt` `--inject --expect-last-frame` PASS rc=0 · `text_j2me` PASS.

### 5. 한계

- paint 를 기다리게 한 것은 «단말에서는 paint 가 다른 스레드의 프레임 한가운데로 들어가지 않는다»는 0430 의 전제를 paint 로 넓힌 것이다. 실기의 스케줄러는 재지 못했다.
- 그 대가로 프레임 하나를 오래 계산하는 KTF·LGT 게임은 paint 수가 준다(`a540945188ca` −20%). 이번 표본 27종에서 그런 게임은 그 하나였다. census 속도 축으로 전체를 다시 재지는 않았다.
- SKT·J2ME 는 바뀌지 않는다(`TaskRunner` 기본값 · `others_preempted` 거짓).
