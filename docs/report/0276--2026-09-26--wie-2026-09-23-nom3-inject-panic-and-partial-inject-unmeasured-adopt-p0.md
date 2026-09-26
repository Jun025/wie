## [2026-09-26] 놈3 가 던지던 «자기 SVC 스텁 표»는 게임 버그가 아니다 — `Thread.sleep` 의 catch 가 오염된 pending 을 되던진 것 · 공유 예외 프레임 체인(p1 로 이미 닫힘) (wie-2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured-adopt-p0)

**무엇을**: 0229 가 이름만 세우고 남긴 게스트 코드 `<Base>+0x30310` 를 디스어셈블했다. 코드 변경은 `interface.rs` 주석 1곳(틀린 원인 서술 정정)뿐이다.

**왜**: 제안 `2026-09-23-nom3-inject-panic-and-partial-inject-unmeasured#p0` — 「그 코드가 무엇을 하려던 것인가만 읽고 멈춘다」.

**사용자 영향**: 없음(원인이 이미 닫혀 있음을 확인한 회차). 놈3 의 잘못된 `athrow` 는 현 main 에서 **0/96**.

---

### ⑴ 디스어셈블 — `binary.mod`(stripped ELF · `.text@0x1000` ⇒ `<Base>` = `0x1000`)

`0x57dd8` 는 `bx r3` 베니어, `.data 0x14021xx` 는 ARM 임포트 썽크(`push {lr}; bl resolver; .word 0x64; .word idx`),
`0x57638` 은 `(1, 0x32)` 썽크 = `PendingException`. 슬롯 이름은 링크 시점 프로브로 실측(아래 ⑶).

```
0x312d0  push {r4,r5,lr} ; sub sp,#8 ; str r0,[sp,#4] ; str r1,[sp]   ; f(long ms) — r0:r1 = 100L
0x312d8  movs r0,#0xc ; bl *0x1402264                 ; (0x64,0x54) Unk54
0x312e0  movs r0,#0   ; bl *0x14021d4                 ; (0x64,0x1f) PushExceptionFrame   ← setjmp; landing = 0x312e8
0x312e8  bl 0x57638                                   ; PendingException
0x312ee  subs r4,r0,#0 ; beq 0x31316                  ; 0 ⇒ try 본문
0x312f2  ldr r3,=0x1500960 ; ldr r3,[r3,#0x6c] ; bl   ; *(0x15009cc) = InterruptedException 클래스 게터
0x312fa  ldr r3,[r4] ; ldr r2,[r0,#8] ; ldr r0,[r3] ; ldr r1,[r2,#8]
0x31302  bl *0x1402164                                ; (0x64,0x12) IsClassAssignable(e.class, "InterruptedException")
0x31308  cmp r0,#0 ; bne 0x3133e                      ; 맞으면 삼킨다
0x3130c  adds r0,r4,#0 ; bl *0x14021f4                ; (0x64,0x21) ThrowException(e) — 되던지기. LR = 0x31315
0x31316  ldr r4,=0x1500960 ; ... ldr r3,[r4+0x134] ; bl   ; *(0x1500a94) = Thread 클래스 게터(초기화)
0x31326  adds r4,r4,#0x13c ; ldr r3,[r4] ; r0,r1 = 인자 ; bl   ; ★r4 = 0x1500a9c · *(0x1500a9c) = Thread.sleep(J)V
0x31336  movs r0,#0 ; bl *0x14021e4                   ; (0x64,0x20) PopExceptionFrame
0x3133e  add sp,#8 ; pop {r4,r5} ; pop {r0} ; bx r0
```

호출부 `0x1875a–0x18760`: `movs r0,#0x64 ; movs r1,#0 ; ldr r3,=0x312d1 ; bl` = `f(100L)`, 루프 안(`0x1876a b 0x186b2`).

⇒ ★**원문**: `try { Thread.sleep(100L); } catch (InterruptedException e) {}` — **정상 자바**다.
되던지는 값은 `PendingException` 의 반환이고, 관측된 `0x1500a9c` 는 ★**try 본문이 `r4` 에 남기는 `Thread.sleep` 임포트 슬롯 주소**다.
«표»가 아니라 «슬롯 하나의 주소»이고, 그 8워드가 전부 `0x7100xxxx` 인 것은 이웃 슬롯(`yield`·`Thread.<init>`·`Clip` …)이기 때문이다.

### ⑵ 판정 — ★**에뮬레이터 결함**(게임 버그 아님) · **이미 닫힘**(`0c196a94`, p1 스레드별 예외 프레임)

| 바이너리 | `Unreadable thrown class` 발화 실행 | `ptr=0x1500a9c` | panic |
|---|---|---|---|
| 현 main(`5434ab0a` + 프로브) | ★**0 / 96** | 0 | 0 |
| 같은 트리 · **스레드 키만 상수 0**(= p1 이전 공유 체인 · p1 자신의 변이) | ★**4 / 48** | — | 0 |

- 대조는 **한 줄** 차이다(`current_thread_key` → `0`). 0229-fix 가 p1 이전에 잰 값도 **4/48** — 일치.
- 4/48 기준에서 0/96 이 우연일 확률 ≈ `(44/48)^96` ≈ 2×10⁻⁴.
- ★공유 체인 실행 30 에서 `Unreadable` 직전 줄은 **실제 힙 예외 `0x48845bd0` 가 `LR 0x31315` 에서 tid 3 으로 되던져지는 것**이다 —
  남의 스레드 프레임이 복원돼 그 스레드가 `sleep` 의 catch 로 뛰어든 형태(0251 의 교차 pop 관측과 같은 기전).
- ★**못 잰 것**: pending 이 정확히 어느 쓰기에서 `0x1500a9c` 가 되는지 — 프로브가 실행당 40줄 상한이라 씨앗 줄을 놓쳤다.
  이미 닫힌 결함이라 상한을 풀고 재빌드(이 부하에서 20–55분)하지 않았다.

### ⑶ 재현 명령(원문)

```
# 디스어셈블 — 게임 바이트는 스크래치에만
unzip -o -q game_lab/broken/lgt/놈3.zip -d "$S/x"; unzip -o -q "$S/x/00015E3D.jar" binary.mod -d "$S/m"
uv run -q --with capstone python3 "$S/tdis.py" 0x312d0 0x31340     # .text 파일 오프셋 0x34 · Thumb
# 실측 — 프로브(커밋 안 함) 빌드 후
RUST_LOG=warn,wie_lgt::runtime::java::interface=trace target/release/wie_validate --inject "$S/n3.zip"
```

`scripts/ktf-image-sweep.py` 는 쓰지 않았다 — `IMAGE_BASE = 0x100000`(KTF) 고정이라 LGT ELF(`.text@0x1000`)에 맞지 않는다.
프로브: `handle_java_system_svc`·`handle_java_svc` 의 `JavaException` 분기(ptr·lr·r4·tid) + `link_class_members` 의 `.bss 0x1500900–0x1500b00` 기입(슬롯 → 클래스.메서드). 최종 diff 에 없음.

### ⑷ 정정한 것

`interface.rs` `java_is_class_assignable` 주석이 원인을 「초기화 안 된 static 을 던진다」로 적고 있었다 — 틀렸다. 동작 무변경(`Ok(0)` 가드 유지 — 다른 오염 경로에 대한 방어로 여전히 값한다).

**게이트**(2026-09-26, loadavg 70~150): fmt rc0 · clippy `--all -D warnings` rc0 · wasm clippy rc0 · `cargo +beta clippy` rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **454 passed / 0 failed**(47 suites) · `clippy --workspace --all-targets` 경고 **16**(= 0251 기준 16). 엔진 동작 diff 0(주석뿐)이라 러너 블록은 돌리지 않았다.

**게임 파일명 유입**: `node scripts/corpus-name-inflow.mjs --corpus <코퍼스>` ⇒ BOUNDED 13회/9쌍(전부 관측 대상 타이틀명 놈3) · SUFFIX-ATTACHED 1회/1쌍 — `interface.rs` 에 **이미 있던** 「서든어택포켓」(= 다른 제목, 이 회차가 쓰지 않았다). 게임 바이트 0.
