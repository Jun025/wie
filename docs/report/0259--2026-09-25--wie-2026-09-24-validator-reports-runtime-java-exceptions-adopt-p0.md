## [2026-09-25] 서든어택포켓(lgt)은 세이브 «앞»에서 막히지 않는다 — 30초 예산이 부팅 전에 끝났고, 실제 정지는 `DataOutputStream 14` 미구현이 여는 «같은 catch 로 되던지기» 무한 루프다 (wie-2026-09-24-validator-reports-runtime-java-exceptions-adopt-p0)

**무엇을** — 코드 변경 0. 두 제안(`2026-09-24-validator-reports-runtime-java-exceptions#p0` ·
`2026-09-24-file-preserves-filenotfound-exception-type#p1`)의 공통 전제
「`java_exceptions` count 0 · 저장 분기 앞에서 멈춘다」를 예산을 늘린 짝지은 재측과 일회용 probe 로 반증했다.

**왜** — 0242 의 `count 0` 은 «예외가 아닌 다른 계급의 벽»으로, 0231 은 «저장 분기 앞»으로 읽혔다.
둘 다 «더 앞에 벽이 있다»를 가정했다. 실측은 **예산 부족 + 뒤에 숨은 무한 루프**를 가리킨다.

### ⒝ 예산 짝 재측 — `count 0` 은 30초 기아였다

origin/main `2349a614` · release `wie_validate` · `RUST_LOG=info` · 두 파일(md5 동일 `3d06675a…` —
`broken/lgt/서든어택포켓.zip` · `_dup/broken/lgt/lgt 서든어택 포켓.zip`). 60 s·240 s 4개 동시 기동(load1 74.86 → 143.89),
30 s 는 9분 뒤 단독(load1 159~166):

| 예산 | 파일 | ticks | paints | java_exceptions | stop |
|---|---|---|---|---|---|
| 30 s | 서든어택포켓 | **2** | 0 | **count 0** · null | deadline |
| 60 s | 서든어택포켓 | 179 | 0 | **count 2** · `java/io/FileNotFoundException: File not found` | deadline |
| 60 s | lgt 서든어택 포켓 | 187 | 0 | count 2 · 동일 | deadline |
| 240 s | 서든어택포켓 | 1470 | 0 | count 2 · 동일 | deadline |
| 240 s | lgt 서든어택 포켓 | 1477 | 0 | count 2 · 동일 | deadline |

```
… AnnunciatorComponent::show()
                                   ← 29.2 ~ 38.9 s 공백 (이 구간 ticks ≈ 1~2)
… Display::getDisplay(null)
… HandsetProperty::getSystemProperty(PHONENUMBER)
jvm::jvm: throwing java exception: java/io/FileNotFoundException File not found
jvm::jvm: throwing java exception: net/wie/WieError Unimplemented: java/io/DataOutputStream vtable index 14
```

부팅에서 `FileNotFoundException` 까지 벽시계 **35~41 s** 이고 그중 29~39 s 가 위 공백 한 구간이다.
`--timeout 30` 은 load 와 무관하게 공백 안에서 끝난다(재현: ticks 2 · count 0). 공백 동안 ticks 가 거의 오르지 않는다 —
executor tick 은 8 ms 상한인데 한 tick 이 끝나지 않는다 = 게스트가 양보 없이 돈다. ★**게스트는 저장 경로에 닿는다**
(세이브 없음 → `DataOutputStream` 을 쓰고 닫는 자리).

### ⒜ 게스트 athrow — «있다», 그리고 그것이 정지다

LGT 게스트의 `athrow` 는 SVC `ThrowException`(0x21)으로 들어오고 그 갈래는 **어떤 로그 레벨로도 남지 않는다**
(`handle_java_system_svc`에 tracing 0 · `wie_core_arm` 의 trace 는 등록 함수 호출만 찍는다) ⇒ `RUST_LOG=trace` 로는
원리적으로 배제할 수 없어 probe 를 심었다(non-LTO release · 최종 diff 0줄). 90 s 한 회:

```
PROBE push   frame=0x4a85cd80 prev=0x0        lr=0x1bdc
java/io/FileNotFoundException File not found
PROBE unwind exc=0x4885a2e0 into frame=0x4a85cd80 resume_lr=0x1bdc
PROBE assignable java/io/FileNotFoundException -> java/lang/Exception = true
PROBE push   frame=0x4a875280 prev=0x4a85cd80 lr=0x21e0      ← catch 뒤 pop 없이 새 try
net/wie/WieError Unimplemented: java/io/DataOutputStream vtable index 14
PROBE unwind exc=0x4885ab60 into frame=0x4a875280 resume_lr=0x21e0
PROBE assignable net/wie/WieError -> java/lang/Exception = false
PROBE guest athrow 0x4885ab60 class=Ok("net/wie/WieError") lr=0x2344
PROBE unwind exc=0x4885ab60 into frame=0x4a875280 resume_lr=0x21e0   ← 같은 프레임으로 되돌아간다
PROBE assignable net/wie/WieError -> java/lang/Exception = false
PROBE guest athrow 0x4885ab60 … (이하 반복)
```

`guest athrow` **133,498회** · 전부 같은 객체 `0x4885ab60` · 같은 `lr=0x2344` · 첫 줄 벽 직후부터 데드라인까지(40.7 s).
⇒ `catch (Exception)` 처리기가 `WieError`(Exception 아님)를 못 잡고 되던지는데, 그 프레임이 아직 현재 프레임이라
`exception::unwind` 가 **같은 처리기로 되돌려 보낸다** — JVM 에서 처리기 안의 throw 는 그 try 에 잡히지 않는다.
(`WieError` 직후의 레지스터 덤프는 스레드 사망이 아니라 `Unimplemented` 발생 시점의 출력이다 — 그 뒤 13만 회가 같은 스레드다.)

### 판정 — 대기 루프·스텁·athrow·기아 중

| 후보 | 판정 |
|---|---|
| 대기 루프 | **아니다** — 기다리는 것이 아니라 초당 수천 회 되던진다 |
| 스텁 반환 | **발단** — `DataOutputStream` 14 칸 부재 → `WieError` |
| 게스트 athrow | **정지 그 자체** — 위 되던지기 루프(런타임의 unwind 가 처리기 프레임을 건너뛰지 않음) |
| 기아 | **0242 의 `count 0` 만** — 30 s 예산이 부팅 공백을 못 넘는다 |

### 같은 루프가 LGT «hang» 5종 전부에 있다

probe 로 LGT 91파일(`working/lgt`+`broken/lgt` · `--timeout 30` · 6병렬)을 쟀고, unwind 가 한 번이라도 난 9종을
`--timeout 60` 으로 다시 쟀다. «unwind 직후 첫 프레임 사건»을 분류했다(주소 재사용 오염을 피하려 매번 초기화):

| 타이틀 | result | 같은 프레임 재진입(60 s) | 발단 |
|---|---|---|---|
| 간호사타이쿤2 | FAIL | 602,363 | `DataInputStream vtable index 22` |
| 서든어택포켓 | FAIL | 499,011 | `DataOutputStream vtable index 14` |
| 턴 | FAIL | 621,042 | `DataOutputStream vtable index 14` |
| 훼밀리마트타이쿤 | FAIL | 597,554 | `StringBuffer vtable index 22` |
| 학교가는길 | FAIL | 601,292 | `Timer vtable index 12` |
| 붕어빵타이쿤3 · 놈3 · 일지매영웅전기 · 현영맞고2006 | PASS | 0 | — |

(재진입 수는 `MARK unwind` 줄 수 — 첫 unwind 1~2회를 포함한 근사다.)
⇒ ★**이 5종의 `no frame rendered (hang/black screen)` 는 «미구현 칸» 한 개씩을 13만~60만 회 되던지기로 가린 것이다.**

★**그런데 «unwind 가 프레임을 pop 하면 된다»는 틀린 수리다** — 같은 측정에서 unwind 직후 첫 사건이
**게스트 자신의 `pop`(같은 프레임) 11회**(PASS 4종 포함) · **pop 없이 새 `push` 4회**로 갈렸다.
unwind 가 pop 하면 앞의 11회가 **이중 pop**(바깥 프레임 해제)이 된다. 맞는 모양은 JVM 의미론 —
«처리기에 진입한 프레임으로는 다시 unwind 하지 않는다(건너뛰고 바깥으로)» — 이고, 이것은 한 칸짜리 행이 아니라
런타임 의미론 변경이라 이 회차에서 하지 않았다(후속 제안 p0).

### 두 관측의 조건 차이(형제 `wie-2026-09-24-baos-tobytearray-and-string-indexof-slots-adopt-p0` 대조)

형제가 인용한 0232 는 `RUST_LOG=info wie_validate --timeout 90` 으로 `BAOS 16 → DataOutputStream 14` 를 봤다 —
**90 s 는 공백(≤ 39 s)을 넘고, 30 s(0242)는 못 넘는다.** 60 s(0231)는 `FileNotFoundException` 까지 닿았다.
0231 의 「BAOS 16 벽이 60 s 안에 나타나지 않았다」는 그 트리(`2ec002f7`)에 BAOS 16 행이 없었으므로(`70a67095` 비조상)
나타났어야 할 벽이다 — `java_exceptions`(eb1a123f) 이전이라 `RUST_LOG` 없이는 `Unimplemented` 가 보이지 않았다는 것이
유력하나 그 회차 원문이 없어 **판정 보류**다.

### 수리 — 하지 않았다

벽의 발단은 한 칸짜리 행(`java/io/OutputStream` `close()V = 14` 추정)이지만 **형제 P0 가 정확히 그 행을 소유**한다
(티켓 경계 ⓖ 「남의 행은 무접촉」). 되던지기 루프는 한 칸이 아니다 ⇒ 둘 다 판정만 넘긴다.
★형제의 짝 재측은 **`--timeout` ≥ 60** 이어야 한다(30 s 는 이 타이틀에서 무엇도 재지 못한다).

### 검증

최종 diff 는 문서 2파일뿐(probe 는 non-LTO 별 target 에서만 빌드 · 원상 복구 후 `git diff -- wie-lgt` 0줄).
`cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 ·
`cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0 ·
`RUST_MIN_STACK=4194304 cargo test --all` rc=0 · **436 passed / 0 failed** ·
`cargo clippy --workspace --all-targets` rc=0 · 경고 **16**(0231·0242 와 같은 수 · 전건 기존 test 타깃) ⇒ 증가 0.
AGENTS.md 러너 블록은 돌리지 않았다 — 엔진 코드 무변경.

**게임 파일명 유입**(`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` 실행값): BOUNDED·SUFFIX-ATTACHED 는
아래 표식 그대로다. 전건 이 회차의 측정 대상 타이틀 이름(표·판정 문장)이고 파일 바이트·경로 밖 유입은 없다.
