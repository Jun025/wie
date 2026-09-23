## [2026-09-23] 배틀몬스터의 두 증상은 하나였다 — `java/lang/Object` index 5 = `notify()V`, 그리고 `IllegalMonitorStateException` 288건은 그 결과였다 (wie-battlemonster-object-notify-slot-and-monitor-ownership)

### 무엇을

`wie-lgt/data/lgt_java_abi.toml` 의 `java/lang/Object` 행에 **한 줄**을 넣었다
(`notify ()V = 5`) + 그 행을 고정하는 시험 1행. 본문 diff 는 데이터 1행·시험 1행이고
나머지는 주석이다.

★**배틀몬스터가 두 파일 전건 `FAIL → PASS`** 로 바뀌었다(`paints 3` · `content true`).
`체스마스터` 에 이어 **두 번째로 그리는 LGT AOT-Java 타이틀**이다.

| 파일 | 무엇 |
|---|---|
| `wie-lgt/data/lgt_java_abi.toml` | `java/lang/Object` 에 `notify()V = 5` + 도출·측정 근거 주석 |
| `wie-lgt/src/runtime/java/jvm_support.rs` | `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 그 행 추가 |

### ⑴ 총괄 가설 「index 5 = `notify()`」 — ★**확인**(단 잔여를 숨기지 않는다)

**⒜ 계수가 강제한다.** `vtable_size = 10` 이고 CLDC 1.1 `java/lang/Object` 의 가상 메서드는
**아홉**이다 — `getClass hashCode equals toString notify notifyAll wait(J) wait(JI) wait()`.
아홉이 열 칸에 들어가면 한 칸이 남고, **측정 앵커 `getClass=1 · equals=3 · toString=4` 는
run 이 1 에서 시작할 때만 셋 다 맞는다** ⇒ index 0 이 남는 칸이고 5~9 는 선언 순서로 고정된다.

★**`rustjava-runtime` 은 «열한» 개를 선언한다**(실측 `src/classes/java/lang/object.rs`):
위 아홉 + `clone()Ljava/lang/Object;`(PROTECTED|NATIVE) + `finalize()V`(PROTECTED).
그 둘은 **CLDC 1.1 에 없다** ⇒ 티켓이 지적한 「11 ↔ 10 의 어긋남」은 ★**하나가 아니라 둘이 빠지는
것**으로 닫힌다(그래서 남는 칸이 정확히 1개다). 둘 다 이 ABI 에 행이 없고 미배치로 남는다.

**⒝ 호출부 관측이 후보를 다섯 → 셋으로 줄인다**(필수① · 계측 후 되돌림).
미배치 스텁과 모니터 임포트에 임시 계측을 넣고 배틀몬스터를 돌렸다:

```
07:13:29.831489  PROBE_MONITOR enter 0x488453d0
07:13:29.831499  PROBE_VTABLE class=Game index=5 r0=0x488453d0 r1=0x49855940 r2=0xffffffff r3=0x4a85aa80 lr=0x2308
```

★**게스트가 «10 마이크로초 전에 모니터를 잡은 바로 그 객체»에 index 5 를 건다** ⇒
`synchronized (o) { o.???(); }` 형태이고, index 5 는 **모니터 계열**이다(`hashCode` 배제).
★**인자 레지스터는 힙 포인터·`-1`·힙 포인터** = 호출자 잔재이지 `long` 이 아니다 —
AAPCS 는 `this` 가 r0 이면 64비트 인자를 **r2:r3** 에 싣는데 그 쌍은
`0x4a85aa80_ffffffff` 로 타임아웃이 될 수 없는 값이다 ⇒ **`wait(J)`·`wait(JI)` 배제.**
⇒ 남는 셋 = **`notify()` · `notifyAll()` · `wait()`**(전부 무인자·void).

**⒞ 셋 중 선언 순서가 `notify()` 를 고르고, ★`notify()` 는 «틀려도 가장 싼» 쪽이다.**
`notifyAll()` 자리에 서면 하나만 깨우고, `wait()` 자리에 서면 일찍 돌아온다.
반대로 **`wait()` 를 `notify()` 자리에 넣으면 스레드가 영원히 잠든다** — 티켓이 경고한 조용한 hang
이다. 그래서 7·8·9 는 **비워 뒀다.**

★**남는 위험을 적는다**: 만약 5 가 실제로 `wait()` 였다면 우리는 **기다려야 할 자리에서 안 기다린다.**
그러나 그 경우 게스트는 `while (!ready) wait();` 로 **바쁜 루프**를 돌 것이고, 실측은 그 반대다 —
배틀몬스터는 **그리고 나서 렌더까지 간다**(⑷). 그리고 그 시점 네이티브 스레드는 **1개뿐**이었다
(스레드 2는 288건 캐스케이드 «뒤»에 생긴다) ⇒ 그 자리에서 `wait()` 였다면 깨울 주체가 없다.

### ⑵ 288건의 출처 — ★**이름은 `Jvm::monitor_exit`(java-interface 임포트 `0x57`)이고, 원인은 index 5 다**

★**티켓의 의심 축(「모니터 임포트가 no-op 이라 획득이 엔진에 도달하지 않는다」)은 «반증»됐다.**
`0x56 MonitorEnter`·`0x57 MonitorExit` 는 **둘 다 배선돼 있고**(`wie-lgt/src/runtime/java/interface.rs`),
실측에서 **획득이 엔진에 도달한다**(`PROBE_MONITOR enter` **4건**).

| 측정 | 값 |
|---|---|
| `monitor_enter` 도달 | **4** |
| `monitor_exit` 도달 | **292** |
| `IllegalMonitorStateException` | **288** = 292 − 4 |

★**그리고 시간축이 인과를 고정한다**: `Game vtable index 5` 예외가 `08:45:08.192` 에 나고,
**첫 IMSE 가 .193** — 1 밀리초 뒤다. 288건 전부가 그 뒤 **203 ms** 안에 들어오고, 전건 같은 객체
(`0x488453d0` = 그 `Game` 인스턴스)다. 예외가 풀리며 그 프레임의 `monitorexit` 핸들러들이
**소유하지 않은 모니터를 반복해서 놓으려 한** 것이다.

⇒ ★**두 증상은 하나의 병이다**(총괄 판단이 맞았다). 그리고 ★**index 5 를 채우자 288 → «0»** 이다
(고친 뒤 `throwing java exception` 문면 **전건 0건**). ★**따로 고칠 것이 남아 있지 않다.**

※`Jvm` 에서 IMSE 를 낼 수 있는 자리는 셋(`monitor_exit`·`object_wait_prepare`·`object_notify`)인데,
뒤 둘은 `Object.wait/notify` 를 거치므로 **그 칸이 스텁인 동안에는 도달 불가**다 —
그래서 288건은 구조적으로 `monitor_exit` 하나뿐이다.

### ⑶ 채운 칸과 «비워 둔» 칸

`java/lang/Object` 행에 넣은 것은 **index 5 한 줄**이다.
**2 · 6 · 7 · 8 · 9 는 도출되지만 쓰지 않았다** — `java/lang/Runtime` index 10 을 비워 둔 것과 같은
이유다: 빈 칸은 **자기 이름을 말하고**, 틀린 칸은 말하지 않는다.
2(`hashCode()I`)를 채우면 게스트에게 **그럴듯한 틀린 정수**를, 7·8·9 를 채우면 **hang** 을 건넨다.
★**어느 타이틀도 아직 그 칸들을 부르지 않았다.**

### ⑷ 배틀몬스터 전/후

| 파일 | before | after |
|---|---|---|
| `배틀몬스터.zip` | **FAIL** · `Unimplemented: Game vtable index 5` + `IllegalMonitorStateException` ×288 · p0 | ★**PASS** · `booted + rendered` · **p3 · content true** · `stop: max-ticks` |
| `lgt 배틀몬스터.zip` | **FAIL** · 같은 문면 · p0 | ★**PASS** · 같음 · **p3 · content true** |

★**«기아가 아님»의 증명은 정지 사유가 진다**(필수③): after 는 `stop: "max-ticks"` 로 끝난다 —
**시간이 아니라 tick 상한 5,000만에 닿았다**(74.9 s · 17.5 s). 굶은 실행은 예산에서 죽지 상한에서
죽지 않는다. before 는 정확히 그 반대였다(`stop: "deadline"` · 120 s 에 tick **2,668**).
★**`ticks`·`paints` 를 절대값으로 인용하지 않는다** — 판정은 `result`·`content`·`stop` 으로 했다.

### ⑸ 회귀 — ★**깨지지 않았다**

**라이브 4종**(featurephone.otterpebble.com 등재분) — 짝지어 전건 양쪽 `PASS`:

| 타이틀 | before | after |
|---|---|---|
| 놈3 | PASS p199 | PASS p288 |
| 메이플스토리2007 | PASS p134 | PASS p139 |
| 현영맞고2006 | PASS p100 | PASS p131 |
| 체스마스터 | PASS p2 | PASS p1 |

**LGT 코퍼스 46파일 짝지은 스윕**(`--timeout 30`): 판정·벽 **유지 41/46** · **`PASS → non-PASS` 0건** ·
PASS **11 → 12**. 바뀐 5건의 처분:

- **배틀몬스터 2파일** — 의도한 대상(FAIL → PASS).
- ★**영웅서기4 `PASS → FAIL` 은 부하다 — 변경이 아니다.** 짝지어 15회씩 재측:
  저부하(load 34~40)에서 **before 6/6 · after 6/6 PASS**, 고부하(load 62~69)에서
  **before 1/6 · after 0/6**. ★**before 도 같이 굶는다** ⇒ AGENTS.md §The four gates 가 적은
  「starved run」이고, 직전 회차도 이 타이틀을 「양쪽에서 흔들린다」로 기록했다.
- **당구마스터2010** — 스윕의 before 값이 부하 산물이었다. 짝지어 3회씩 재측하니
  **양쪽 다** `Unknown lgt stdlib import` 로 동일. 변경 아님.
- ★**학교가는길 — `ax vtable index 30` → `29`.** 같은 클래스·같은 자리에서 죽고 **숫자만** 1 내려갔다
  (양쪽 결정론적 · 3/3 재현). `java/lang/Object` 의 미배치 꼬리가 **8 → 7** 로 줄어 그 아래 표가
  한 칸 당겨진 것이다. ⇒ ★**이 문면의 숫자는 «게스트의 ABI 인덱스»가 아니라 «우리 표의 위치»다.**
  형제 티켓(`ax vtable index 30`)은 이제 **29** 로 읽어야 한다.

**게이트**: `cargo fmt --all -- --check` · `cargo clippy --all -- -D warnings` ·
`cargo clippy --target wasm32-unknown-unknown -- -D warnings` · `cargo +beta clippy --all -- -D warnings` ·
`RUST_MIN_STACK=4194304 cargo test --all` — 전건 rc=0.
러너 블록(`draw_j2me` · `helloworld_ktf` · `helloworld_lgt` · `text_j2me` · `keydraw_ktf` ·
`keydraw_lgt`) 전건 PASS.

### 왜

미배치 vtable 칸은 **그 칸을 부른 타이틀을 죽이는 데서 끝나지 않는다** — 이번 것은 죽으면서
288건의 2차 예외를 만들었고, 그 2차 증상이 «모니터 기계의 독립된 병»처럼 보였다.
★**증상 두 개를 각각 발권하면 없는 결함을 고치게 된다.** 시간축(1 ms 간격)과 계수(292 − 4 = 288)가
그것을 가른 자리다.

### 사용자 영향

★**배틀몬스터가 그린다.** 지금까지 LGT AOT-Java 중 화면까지 간 것은 체스마스터 하나였고, 이제 둘이다.
등재된 4종은 전건 그대로다.

### 게임 파일명 유입 — `node scripts/corpus-name-inflow.mjs` 를 «실행해서» 적는다

**유입 29건(BOUNDED)** · **판단 필요 5쌍(SUFFIX-ATTACHED)** · PREFIX-EMBEDDED 1건.
게임 «바이트»는 0이고, 들어온 것은 **제목 문자열**뿐이다 — 이 저장소가 증거를 적는 기존 방식 그대로이고
(ABI 표는 이미 타이틀 이름을 각 행의 근거로 적는다), Constraint 9 가 금지하는 것은 게임 바이트다.

SUFFIX-ATTACHED 5쌍의 **내 판정**(기계가 못 가르는 바구니라 손으로 가른다):
**4쌍은 진짜 언급**이고 전부 **한국어 조사**(…의 / …가 / …를 / …는)가 붙은 형태다.
**1쌍만 «더 긴 다른 제목»** 이다 — 짧은 stem 하나가 그것을 접두로 갖는 **다른 코퍼스 stem** 안에 들어
있는 것이고, 그 문장은 **이 회차가 건드리지 않은 기존 주석**이다.

<!-- corpus-name-inflow v1 subjects=4 tree=c670ed397e5438bc B=48/29 P=1/1 S=12/5 -->
