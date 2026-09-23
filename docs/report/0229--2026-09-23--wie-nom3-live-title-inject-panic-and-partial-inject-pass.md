## [2026-09-23] 놈3 입력 panic 의 이름은 «게스트가 자기 함수표를 throw 한다» + 부분 주입을 UNMEASURED 로 (wie-nom3-live-title-inject-panic-and-partial-inject-pass)

**무엇을**: ⑴`java_is_class_assignable` 이 게스트 레지스터에서 온 클래스 포인터를 `unwrap()` 으로 역참조하다
호스트 프로세스 전체를 죽이던 것을 고쳤다(`JavaClassDefinition::try_name`). ⑵`--inject` 의 UNMEASURED 술어를
`input_steps == 0` → `input_steps < input_steps_total` 로 넓혔다.

**왜**: 놈3 는 featurephone.otterpebble.com 에 등재된 **라이브 타이틀**이고(이 회차 실측: `broken/lgt/놈3.zip`
sha256 `b475b639…` = 티켓이 인용한 배포 번들 해시와 일치), 방향키를 누르면 약 4번에 1번 죽었다.

**사용자 영향**: 놈3 가 입력에 죽지 않는다 — `--inject` **16/16 PASS · 27/27 스텝 · FAIL 0**.
같은 `unwrap` 무리를 공유하는 **모든 LGT 타이틀**이 함께 산다(패닉은 프로세스 전체를 죽였다).

---

## ① 놈3 입력 panic — 사슬 전체를 «실측으로» 세웠다

### ⑴ `0x104C02B4` 의 이름 — 티켓 §필수①

★**주소가 고정이라는 단서가 맞았다.** 사슬은 다섯 걸음이고 전부 실측이다:

| # | 관측 | 도구 |
|---|---|---|
| 1 | 게스트가 **SVC `0x19`(`ThrowException`)** 를 `R0 = 0x1500a9c` 로 실행한다 | 예외 경로 프로브 |
| 2 | `0x1500a9c` = ★**`.bss + 0xa9c`** (`.text@0x1000` · `.data@0x1400000` · `.bss@0x1500000`) | `RUST_LOG=wie_lgt=debug` 의 `Section` 줄 |
| 3 | 그 8워드가 ★**전부 `0x7100xxxx`** = `FUNCTIONS_BASE(0x71000000)` 의 SVC 스텁 ⇒ ★**객체가 아니라 «함수 포인터표»다** | 게스트 메모리 덤프 |
| 4 | 게스트가 그것을 예외 인스턴스로 읽어 클래스 워드 ★**`0x104C02B4`** 를 얻고 SVC `0x18`(`IsClassAssignable`)에 넘긴다 | 인자 프로브 |
| 5 | `0x104C02B4` 는 ★**LGT 어느 영역에도 매핑돼 있지 않다**(이미지 3섹션 · 힙 `0x40000000–0x50000000` · 전역 `0x7fff0000`) ⇒ `raw()` 가 `InvalidMemoryAccess(273416884)` | `PROBE-NAME raw() FAILED ptr_raw=0x104c02b4` |

⇒ ★**이름**: 「게스트가 `.bss+0xa9c` 의 **자기 SVC 스텁 표**를 예외 참조로 `athrow` 하고, 그 첫 워드를
클래스 헤더로 읽은 값이 `0x104C02B4` 다」. ★**쓰레기 포인터가 아니라 «잘못된 종류의 객체»다.**

★**게스트 호출 자리도 잡았다**: `LR 0x31315` ⇒ `<Base>+0x30310` ← `<Base>+0x17760`,
`IP=0x19`(스텁이 싣는 SVC 번호), `R1/R2`(`0x4904d960`/`0x4904d940`) = 바로 뒤 `IsClassAssignable` 의
`ptr_class_name`/`ptr_fields` 와 **동일**.

★★**정직하게 남긴다 — «왜 게스트가 그것을 throw 하는가»는 못 밝혔다.** 그것은 AOT 컴파일된 게스트 코드
안(`<Base>+0x30310`)에 있고, 이 회차의 범위를 넘는다. 티켓이 허용한 **「이름 + 최소재현」**까지가 산출물이다.

### ⑵ jvm-trait 회차와의 관계 — 티켓 §필수②의 ⒜/⒝

★**⒜ 다 — 그 회차가 «덮지 않은» 자리**다. `wie-lgt-jvm-trait-…-panics`(머지)는
`NativeJavaValueCodec::object_from_raw` 를 **게이트**했고, 자기 회신에서 실측 프레임이
`object_from_raw ← JavaClassInstance::get_field` 임을 밝혔다.
★이번 자리는 **`ClassDefinition::name` → `descriptor()` → `raw()`** 로 **다른 트레이트·다른 호출 사슬**이고,
그 회차의 실측 프레임에 **등장하지 않는다**. ⇒ 같은 «오류 통로 없는 트레이트» 계급의 **다음 자리**다.

### ⑶ 처방 — `unwrap`→`?` «만» 하지 않았다

티켓 §필수②의 경고(「패닉이 조용한 실패로 바뀌면 더 나쁘다」)를 그대로 받았다. 그래서 ★**되돌리는 값이
«거짓 성공»이 아니라 «자바 의미론상 옳은 답»인 자리**를 골랐다:

- `JavaClassDefinition::try_name()` — `name()` 과 **같은 읽기**, 다만 `Result`. `name()` 은 `self.try_name().unwrap()` 로
  남겨 ★**기존 호출부의 동작을 한 글자도 바꾸지 않는다**(이 런타임이 스스로 등록한 클래스에는 언제나 성공한다).
- `java_is_class_assignable` 에서만 그것을 쓰고, 읽지 못하면 ★**`0`(= not assignable)** 을 돌려준다.
  ★**이 함수는 «catch 절 일치 검사»다** — 읽을 수 없는 클래스에 대해 「일치한다」고 말할 근거가 없으므로
  **「일치하지 않는다」가 이 런타임이 할 수 있는 정직한 답**이고, 게스트의 **자기 핸들러 탐색이 계속 돈다**.
- ★**같은 함수 안의 비대칭이 결함의 형태였다**: `ptr_class_name`(게스트 포인터)은 `?` 로 전파하는데
  `ptr_class`(게스트 포인터)는 패닉이었다. 둘의 신뢰 등급을 같게 맞춘 것이 이 diff다.
- ★**조용하지 않다** — `tracing::error!` 로 주소와 대상 클래스명을 남긴다.

### ⑷ 놈3 전/후 — 티켓 §필수② 의 16회

| | n/N | 비고 |
|---|---|---|
| **전**(`origin/main` 무수정 release) | ★**FAIL 2/8** | 죽은 스텝 `11_LEFT`·`10_OK` — ★**서로 다른데 주소는 전건 `0x104C02B4`** |
| 전(선행 회차 합산 · 티켓 인용) | FAIL 5/16 | 합산 **7/24 ≈ 29%** |
| **후** | ★**FAIL 0/16** · 전건 `PASS 27/27` | rc=0 · 재현율 25% 라 16회 FAIL 0 의 우연 확률 ≈ 1% |

★**«고쳐서 사라진 것»과 «살아남게 된 것»을 구별한다**: 게스트의 잘못된 `athrow` 는 **그대로 난다**.
한 판에서 **46,222회** 관측됐고 그래도 `PASS 27/27` 이었다. ⇒ ★**25% 는 «발생률»이 아니라 «그 상태에 들어가는 비율»**
이었다 — 종전에는 **첫 1회가 곧 프로세스 사망**이라 판당 1회로 보였을 뿐이다.

---

## ② 부분 주입 — 코퍼스를 «재고» 골랐다 (티켓 §실측② ⒝)

**코퍼스 실측**(2026-09-23 · 한 번이라도 `survived input sequence` 를 낸 **177종** 중 **175종 실행** ·
★**loadavg 77–98** = 절단이 가장 잘 나는 조건 · ①의 수정이 들어간 바이너리):

| 축 | 수 |
|---|---|
| PASS | 145 |
| 그중 `survived input sequence` 주장 | 143 — ★**전건 27/27** |
| ★**부분 주입인데 «주장»까지 한 것** | ★**0** |
| ★**0스텝 주장** | ★**0** |
| 기존 술어가 잡은 UNMEASURED | **3** — ★**전건 `0/27` 이고 `stop = clean exit`** |
| 부분 주입 PASS | **2** — `케로로미니게임 10/27` · `삼국지연의2 2/27` · ★**둘 다 `stop/reason = clean exit`** |

★★**이 표가 설계를 뒤집었다.** 기존 술어(`== 0`)는 ★**같은 종료 원인(`clean exit`)에 대해 `0/27` 은 «미측정»,
`10/27` 은 «PASS»** 라고 말한다 — 누락된 경우가 아니라 ★**자기모순**이다. 부분 주입은 종료 원인과 무관하게
부분 측정이다. ⇒ ★**「총계 미달 전건」(`input_steps < input_steps_total`) 을 골랐다.**
※티켓이 준 두 선택지(「0 초과 & 총계 미달」 ↔ 「총계 미달 전건」)는 기존 `==0` 절과 합치면 ★**같은 집합**이다 —
술어 하나로 쓰는 쪽을 골랐다.

★**대가를 숨기지 않는다**: 이 코퍼스에서 **바뀌는 것은 정확히 2종**이고 `PASS → UNMEASURED` 다.
★**`FAIL` 로 뒤집은 것은 0** — 티켓이 못박은 경계(「PASS 를 FAIL 로 뒤집는 것이 목표가 아니다」) 그대로다.
★**기존 UNMEASURED 3종은 그대로 UNMEASURED** 다(좁히지 않았다 — `!exited` 로 한정하는 안은 그 3종을 전부
풀어 버려서 **기각**했다).

★**오늘의 이득은 0 이고, 그것도 적는다**: 부분 주입이 «주장»과 함께 온 사례는 이 코퍼스에 없다.
남긴 이유는 ⒜위 자기모순 해소 ⒝원 결함의 형태(`--max-ticks` 백스톱)는 **틱으로 절단**하는데 지금 코퍼스에
스크립트 중간에 백스톱을 태울 만큼 빠른 타이틀이 없을 뿐이라는 것(AGENTS.md 가 이미 「크기가 아니라 형태」로 적어 둔 축).

★**실동작 확인**: `삼국지연의2` → ★`UNMEASURED 2/27 · rc=2 · "--inject delivered 2/27 input steps (run ended: clean exit)"`.
`케로로미니게임` 은 **저부하 재실행에서 27/27 PASS** 였다 ⇒ ★**부분 주입은 부하 의존이다** — 그래서 고부하에서
10/27 을 «전량 통과»로 적던 종전 동작이 실제 위험이었다.

### 라이브 4종 등재 회차의 실제 스텝 수 — 티켓 §실측② ⒜

★**4종 전부 «전량»이었다 — 부분으로 등재된 것은 없다.**

| 타이틀 | 등재 근거의 스텝 | 출처 |
|---|---|---|
| 놈3 · 메이플스토리2007 · 현영맞고2006 | **28/28**(스텝 샷 각 28장) | `…-confirm-three-renderable-aot-java-titles.done.md` |
| 체스마스터 | **28스텝 전건 실행** | `…-register-chessmaster-aot-java.done.md` |

★★**그러나 «아슬아슬했다»**: 체스마스터 회차의 **첫 `--inject` 실행은 0스텝 공허 PASS** 였고
(`--shotdir` 0장 · `ms:6214`), 그 회차가 **손으로 알아채** 백스톱을 올려 다시 쟀다. ⇒ 그 형태가 PR #256 이
기계로 닫은 그것이고, 이번 회차는 **부분까지** 닫았다.

---

## 회귀 — 라이브 4종 전건

| 타이틀 | 부팅 `--timeout 120` | 주입(등재 회차 파라미터) |
|---|---|---|
| 놈3 | PASS `p791 dc3 lf=true` | ★PASS **27/27** `p173 dc99` |
| 메이플스토리2007 | PASS `p500 dc63 lf=true` | PASS **27/27** `p633 dc512` |
| 현영맞고2006 | PASS `p403 dc62 lf=true` | PASS **27/27** `p281 dc512` |
| 체스마스터 | PASS `p2 dc36 lf=true` | PASS **27/27** `p12 dc41` |

★**놈3 는 고치는 대상이면서 회귀 대상이다** — 부팅 축 PASS 유지 확인.
★`paints`·`distinct_colors` 절대값을 «개선»으로 읽지 마라(부하 의존 · 아래 위생 절).

## 게이트

4관문 + `cargo +beta clippy` **전건 rc=0**(`cargo test --all` **46 suites · 실패 0**).
엔진 코드를 만졌으므로 **러너 블록** 전건: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me(--timeout 5)` **PASS** ·
`keydraw_lgt --inject --expect-last-frame` **PASS 27/27 p28** · `git status test_data/` **clean**.
clet 회귀 `test_helloworld`(+`_unmodified_archive`·`_jar_under_p_prefix`) **ok**.
문서 검사기 7종(`worklog-json`·`docs-report-serial`·`doc-liveness-parity`·`worklog-coverage`·`inflow-marker`·
`checker-census`·`engine-contract`) 전건 rc=0.

★**`keydraw_ktf` 가 1회 FAIL 했다 — 숨기지 않는다.** `paints 20`(무부하 범위 48–55) · **loadavg 96** ·
AGENTS.md §The four gates 의 4단 절차대로 **재실행 5회 → 4 PASS**(`p39–52`, 무부하 범위 안) · FAIL 2회는 `p20`·`p27`.
★**구조적으로도 이 diff 가 닿지 않는다**: 엔진 수정은 **`wie-lgt/` 전용**인데 그 고정물은 **KTF** 이고,
`input_steps` 는 **전 실행 27/27** 이라 ②의 게이트가 발화한 적도 없다.
★**`origin/main` 재빌드로 ⑶단계를 밟지는 않았다**(이 부하에서 릴리스 빌드 1회가 26–41분) — 그 한계를 그대로 적는다.

## 시험 — 개악 대조로 «양방향으로» 진다

| | 개악 | 결과 |
|---|---|---|
| **M1** | ②술어를 옛 `== 0` 으로 | `zero_injected_steps_is_not_a_pass_test` **FAILED** |
| **M2** | ②호출부가 `input_steps` 대신 `input_steps_total` 을 받게 | `the_gate_is_handed_the_delivered_count_test` **FAILED** |
| **M3** | ①`try_name` 을 옛 `unwrap` 판으로 | `try_name_reports_an_unreadable_class_instead_of_panicking` **FAILED**(`:347` 패닉) |

원문 복구 후 전건 ok. ★M3 의 시험은 **놈3 가 실제로 낸 주소 `0x104c02b4`** 를 그대로 쓴다.

## 측정 위생

- `uptime` load1: 착수 **24.8** → 놈3 16회 **55→84** → 코퍼스 스윕 **83→98** → 회귀 **41–72** → 러너 블록 **96–102**.
- ★**기아 아님을 «판정 불변»으로 보였다**: 놈3 16회에서 `paints` 가 **149 → 2** 로 무너지는 동안
  판정은 **16/16 PASS · 27/27** 로 한 번도 움직이지 않았다(AGENTS.md 의 「수가 아니라 판정을 읽어라」 그대로).
- ★`ticks`·`distinct_colors` 절대값을 근거로 쓰지 않았다 · ★`--inject` 는 `--timeout` 을 덮는다(러너 블록 외 미사용).
- ★25% 는 **n/N 으로만** 적었다(전 2/8 · 선행 5/16 · 후 0/16).

## 계측 코드

★**최종 diff 에 프로브 0** — `PROBE` 문자열 전수 검색 0건. 값을 하지 못한 변경(스레드 교차 가설 등)은 남기지 않았다.
※그 가설도 적어 둔다: `SUPPORT_CONTEXT_BASE` 의 예외 프레임 스택은 **전 게스트 스레드 공유**라 교차를 의심했는데,
실측에서 실패 순간의 `frame_sp == cur_sp` 라 ★**이 결함의 원인이 아니다**(공유 자체는 사실이므로 후속 제안으로 남긴다).

## 손대지 않은 것

`lgt_java_abi.toml` **무접촉**(형제 P0·P1 소관) · 배틀몬스터 무접촉 · `smoke_gate.sh` 무접촉 ·
`otterpebble`·`tower`·원장 무접촉 · 게임 바이트 커밋 0 · `game_lab/` 커밋 0 · 새 검사기·CI 스텝 0.
