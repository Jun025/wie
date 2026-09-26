## [2026-09-27] 학교가는길 · 놈3 «안 그리는» 판의 첫 대기 지점 — 둘 다 입력 대기가 아니다 · Timer 12 · Stack 32 등재 (wie-lgt-no-frame-hang-first-wait-point-measure)

**무엇을**: 제안 `2026-09-24-lgt-link-appended-vtable-slot-propagation#p0`(학교가는길) ·
`2026-09-25-lgt-exception-frames-per-thread#p0`(놈3) 채택. 두 타이틀의 «첫 프레임 전에 무엇을 기다리나»를 스레드별로 쟀다.
코드 변경은 `wie-lgt/data/lgt_java_abi.toml` 두 줄(`java/util/Timer` 12 · `java/util/Stack` 32)뿐이다.

**왜**: 두 제안 모두 「배틀몬스터 입력 대기 군집에 합류시키기 전에 대기 원인을 한 번 재라」였다.

**사용자 영향**: 학교가는길이 부트 즉시 죽던 오류(`Unimplemented: java/util/Timer vtable index 12`)를 넘어 첫 화면(빈 화면)까지 간다.
아직 그리지는 못한다 — 다음 벽은 아래 ⑴-c. 놈3 은 변화 없음(측정만).

### 판정

| 타이틀 | 첫 프레임 전에 멈춘 곳 | 배틀몬스터 «입력 대기» 군집? |
|---|---|---|
| 학교가는길 | ★**대기가 아니다.** `origin/main` 에서는 hang 이 아니라 부트 0.16s 의 하드 실패(Timer 12 → 등재 후 Stack 32). 두 줄을 넣으면 **게임 루프 `TimerTask`(`c.run()`)가 첫 호출에서 죽는다** — 호스트 `TimerTask` 필드가 게스트 하위 클래스의 필드 칸을 덮는다(⑴-c). 스레드가 남지 않아 실행기가 빈 채로 돈다 | **아니다 — 별 원인**(ABI 칸 2개 + 필드 배치) |
| 놈3 | ★**모니터 교착(deadlock).** 스레드 3 은 A→B, 이벤트 스레드 4 는 B→A 순서로 잡는다. 스레드 3 이 A 를 잡은 뒤 B 를 청하기 전에 스레드 4 가 B 를 먼저 잡으면 둘 다 영원히 선다(FAIL 3/3 같은 모양) | **아니다 — 별 원인**(잠금 순서 역전 + 실행기 스케줄) |

### ⑴ 학교가는길

- 대전제 재측(`origin/main` `3d912985` release): `FAIL · stop error · ticks 4 · 160ms` 3/3 —
  `net.wie.WieError: Unimplemented: java/util/Timer vtable index 12`. 제안이 본 `no frame rendered`(t1231~2198)는
  #312 의 base `5434ab0a` 에서 3/3 재현했다(같은 명령). ⇒ **그 사이 hang 이 하드 실패로 바뀌었다.**
  근거는 이 회차가 가르지 않았다(#292 unwind 수리가 가장 가까운 후보다).
- ⑴-a **Timer 12** — 스크래치 프로브(`handle_missing_java_vtable_entry` 에서 lr · 인자 6개 · 인자의 클래스, 미커밋) 3/3:
  `lr=0x25c0 params=[Timer, <게스트 c 인스턴스>, 0, 0, 0x64, 0]` = 객체 1 + long 2(지연 0 · 주기 100).
  CLDC 1.1 Timer 는 Object(10칸) 뒤 `schedule(TJ) schedule(TDate) schedule(TJJ) schedule(TDateJ) scheduleAtFixedRate(TJJ)
  scheduleAtFixedRate(TDateJ) cancel()` 순이라 12 = `schedule(TimerTask,JJ)V`. 같은 모양인 14(`scheduleAtFixedRate`)와는 순서만이 가른다 —
  틀려도 100ms 주기의 고정지연/고정비율 차이다. 등재 후 호스트가 계산한 `nextExecutionTime − lastScheduled` = 105ms ⇒ long 단어 순서도 맞다.
- ⑴-b **Stack 32** — Timer 를 넣자 3/3 `Unimplemented: java/util/Stack vtable index 32`(lr `0x138f24`, 인자 1개).
  Vector 의 기존 4 앵커(size 15 · elementAt 23 · removeElementAt 27 · insertElementAt 28)가 CLDC 1.1 Vector 22메서드 순서와 전부 맞아
  Vector = 32칸 ⇒ Stack 자기 칸은 32 부터 `push pop peek empty search`. 32 = `push(Object)` · 인자 1개 모양도 맞다.
  등재 후 런타임 로그 `java.util.Stack::push(0x488412d0, 0x488423d0)` 정상.
- ⑴-c **다음 벽 — TimerTask 필드 덮어쓰기**(수정하지 않았다 · 제안). 두 줄 뒤 판 = `FAIL · max-ticks · paints 1 · only blank` 3/3.
  스레드 2(`Timer$TimerThread`)가 첫 틱에 `c.run()` 에서 `Invalid memory access; address 0xde07b454` 로 죽는다.
  - 게스트 `c.run()` 은 `ldr r3,[this,#8]; ldr r1,[r3,#0x18]; ldr r3,[r1]` — **자기 필드 word 6** 을 객체로 읽는다.
  - 직전에 호스트가 `Put field c.lastScheduledExecutionTime:J = 1790431442004`(= `0x1a0_de07b454`) — ★**죽은 주소가 그 값의 하위 단어다.**
  - 필드 프로브: 게스트 `c` 서술자 `total=7 own=0`(메타데이터에 드러난 자기 필드 없음) ⇒ 폰의 TimerTask 는 **6 단어**
    (J2SE 배치 lock · state · nextExecutionTime J · period J 와 같은 수)이고 word 6 은 `c` 의 숨은 자기 필드다.
    호스트 `rustjava-runtime 0.1.1` TimerTask 는 `lastScheduledExecutionTime J` 를 더해 **8 단어** ⇒ 매 틱 word 6 을 덮고 word 7 은 인스턴스 밖에 쓴다.
  - ABI 파일의 고정 필드 칸으로는 못 푼다(6 단어 안에 long 3 개 + 2 단어가 안 들어간다) — 런타임 쪽 결정이라 S 가 아니다.

### ⑵ 놈3

- 대전제 재측: 조용한 판(load1 4.4–16.3) 순차 **0/80** no-frame. 스스로 부하를 걸어(병렬 12) main 무계측 **0/180**(load1 63–119) ·
  계측 **0/180**. ★**그러나 «main 에서 사라졌다»로 읽지 않았다** — 양성 대조가 실패했다: FAIL 이 잡혔던 #312 base `5434ab0a` 도 같은 방식으로
  **0/180**(load1 51–150). 차이는 굶주림 정도였다(#312 판 paints 중앙 **40** ↔ 이 방식 140–179). ⇒ 병렬 24 로 올려 **3/216** 을 잡았다.
- 스크래치 모니터 프로브(`java_monitor_enter`/`exit` 에 `MON req/got/exit <obj> lr=`, 미커밋) — FAIL 3건이 **같은 순서**다(시각은 h17):

| t | 스레드 3(java · `setPriority(1)`) | 스레드 4(이벤트·그리기, native) |
|---|---|---|
| +0.000 | B `0x488461e0` 획득(lr `0x1c7bd`) → `sleep(80)` 을 B 를 쥔 채 | |
| +0.0004 | | B 청함(lr `0x3143d`) — 대기 |
| +0.273 | B 놓음 → ★**곧바로 B 재획득**(lr `0x1ce19`) · 놓음 → A `0x48845d70` 획득(lr `0x31507`) | |
| +0.273 | | ★B 획득 → A 청함(lr `0x1e1e7`) — **영원히** |
| +0.274 | B 청함(lr `0x3143d`) — **영원히** | |

  그 뒤 18초 동안 움직이는 것은 스레드 2 의 `Thread.sleep(100)` 뿐이다(0265 의 «B 모양» · 0276 의 «돌지 않고 서 있다»와 맞는다).
- PASS 대조(계측 1판)는 같은 코드 자리를 다른 순서로 지난다: 스레드 3 이 A → B(`0x3143d`) → A 재진입(`0x1e1e7`)까지 **스레드 4 가 끼기 전에** 끝낸다.
  ⇒ 게임 코드에 잠금 순서 역전(스레드 3: A→B · 스레드 4: B→A)이 있고, 창은 «스레드 3 이 A 를 잡은 뒤 B 를 청하기 전»이다.
- 왜 우리 쪽에서 창이 열리나(측정이 아니라 읽은 사실): `jvm 0.1.1` `Monitor::exit` 는 대기자에게 **넘겨주지 않고 알리기만** 한다 —
  놓은 스레드가 같은 poll 안에서 바로 다시 잡는다(+0.273 의 B 재획득). 실행기는 `setPriority` 를 보지 않는다(스레드 3 은 우선순위 1).
  폰처럼 놓는 순간 대기자(스레드 4)가 B 를 받았다면 A 는 비어 있어 교착이 안 난다. ★**어느 쪽을 고칠지는 가르지 않았다**(아래 한계).

### 한계

- 놈3 FAIL n=3, 전부 병렬 24 의 자가 부하에서. 조용한 판 재현율은 이 회차에서 0/80, 0265 의 1/83 과 같은 크기다.
- 학교가는길의 hang→하드 실패 전환 커밋은 이분 탐색하지 않았다.
- 크로이센은 두 바이너리 모두 JSON 을 내지 않았다(기존 · 이 회차 무관).

### 검증

- LGT 91 파일 스윕(`--timeout 10`, main `3d912985` ↔ 이 브랜치): 바뀐 타이틀 **1** — 학교가는길(`Unimplemented` → `only blank`). 나머지 동일 · PASS 76 / FAIL 14 양쪽 같음.
- 네 게이트 + beta clippy: fmt OK · clippy/wasm clippy/beta clippy 경고 0 · `RUST_MIN_STACK=4194304 cargo test --all` 47 스위트 479 passed 0 failed.
- 러너 블록: draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS · keydraw_ktf/lgt `PASS paints 55 rc=0`(load1 ~115).
- 증거(저장소 밖 · 게임 바이트 0): `~/orchestrator/reports/evidence/wie-lgt-no-frame-hang-first-wait-point-measure/` —
  놈3 판별 tsv(조용 80 · 병렬 12×4 조건 · 병렬 24) · FAIL 3 로그 · PASS 모니터 로그 · 학교가는길 등재 후 로그 · 91 스윕 표 · 루프 스크립트 · 프로브 설명.

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus <game_lab>`): 수는 아래 표식의 B(BOUNDED)·S(SUFFIX-ATTACHED) 다 — ★S 도 0 이 아니다. 대상이 `lgt_java_abi.toml` «파일 전체»라 기존 주석의 타이틀(배틀몬스터·서든어택포켓·스파이더맨3 등, 리니지가 이미 적은 라이브 타이틀)이 함께 세어진다. 이 회차가 새로 쓴 이름은 `학교가는길`·`놈3`(이 회차의 대상)과 `배틀몬스터`(비교 군집 이름)뿐이다.

<!-- corpus-name-inflow v1 subjects=3 tree=497a2d7e664fd3e2 B=71/22 P=4/2 S=12/6 -->
