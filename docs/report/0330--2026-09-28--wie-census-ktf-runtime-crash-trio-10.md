## [2026-09-28] KTF 런타임 중단 3형 — 네이티브 predicate 는 «jar 바이트코드가 client.bin 을 가린 것» · java_throw 스택 넘침은 «힙 소진 예외의 재귀» · getNextEvent 4종은 미식별 콜백이 아니라 각기 다른 원인 (wie-census-ktf-runtime-crash-trio-10)

**무엇을**
- `KtfClassLoader.loadClass` 재정의 — jar 가 `.class` 로도 싣는 이름은 **client.bin 이 먼저**(나머지는 종전대로 parent-first). jar 의 `.class` 목록은 `KtfJvmSupport::init` 이 client.bin 을 찾는 김에 같이 모은다.
- 게스트 힙이 가득 차 인스턴스를 못 만들면 **미리 만들어 둔 `OutOfMemoryError`**(`KtfClassLoader.outOfMemoryError` 정적 필드)를 던진다 — `instantiate`·`instantiate_array` 공통 `KtfJvmSupport::instantiation_error`.
  `JavaMethod::run` 의 `jvm.exception(…)`(내부 `unwrap` 두 개) 자리는 할당 실패를 그대로 돌려주는 `KtfJvmSupport::wie_error` 로.
- 시험 2 — `test_load_class_prefers_client_bin_over_jar_bytecode` · `test_allocation_failure_throws_reserved_out_of_memory_error`.

**왜**
- ⒜ `3c658a46bbfb` `5a59f62d1f1a` `c4b90400f9fe`: KTF zip 286개(working·broken) 중 **이 3개만** jar 에 `.class`(원본 `Clet.class`·`Clet$CletCard.class`)와 client.bin 을 **함께** 싣는다(281개는 client.bin 만 · 2개는 둘 다 없음 — 실측). client.bin 에도 같은 `Clet` 이 AOT 로 있다(문자열 `Clet$CletCard`·`paintClet` 등 12건). parent-first 라 시스템 로더가 jar 바이트코드를 먼저 집었고 → 메인 클래스가 네이티브가 아니라 `class_definition_raw` 의 `unwrap` 이 부팅에서 panic 했다(`jvm_support.rs:223`). 앵커만 넘기면 그 바이트코드 `Clet` 의 `native` 메서드가 게스트 객체를 요구해 `object_to_raw` downcast 가 다음 panic 이었다 — **같은 원인의 두 증상**이다.
- ⒝ `6c96c5050b2b` `c94d64777926` `e9fac881e602`: `java_throw` 재귀가 **아니다**. macOS 크래시 리포트(스택 86프레임)의 꼭대기는 `JavaArrayClassDefinition::instantiate_array` 7겹 — 할당 실패 → `jvm.exception` → `Throwable.fillInStackTrace` 의 문자열 → `instantiate_array` 실패 → … 가 호스트 스택을 다 쓴 것이다.
  힙이 찬 까닭: 게임이 한 번의 paint 안에서 NPE 를 던지고 잡기를 **43,276회**(5분) 반복하는 동안 GC(paint 끝에서만 돈다)가 한 번도 돌지 않았다 — NPE 시작 뒤 `Destroy` 로그 **0줄**.
  #350 의 notifyDestroyed 재귀와는 **다른 모양**이다(그쪽은 게스트 메서드 사이의 무한 호출).
- ⒞ «getNextEvent 주소 0 · 미식별 콜백»은 **한 원인이 아니다**. 전부 타이머 콜백(`MC_knlSetTimer`)이 `getNextEvent` 안에서 도는 곳이라 같은 줄에 찍혔을 뿐이고, 게스트 결함 PC 를 떠 보니 넷이 다 다르다(아래 표). 이번 회차는 고치지 않았다.

**사용자 영향**: `5a59f62d1f1a` `c4b90400f9fe` 부팅·화면·조작 통과. `6c96c5050b2b` `c94d64777926` `e9fac881e602` 는 **탭(호스트)이 죽지 않는다** — 둘은 여전히 멈추지만 오류 안내로 멈춘다.

### 전/후(전건 · `--inject --timeout 30` · 장시간은 `long.keys` 600초)
| sha12 | 형 | 전 | 후 | 다음 벽 |
|---|---|---|---|---|
| `5a59f62d1f1a` | ⒜ | 부팅 panic(`unwrap` None) | **PASS** 27/27 · paints 44 | — (장시간 미측정) |
| `c4b90400f9fe` | ⒜ | 부팅 panic | **PASS** 27/27 · paints 189–197 | — (장시간 미측정) |
| `3c658a46bbfb` | ⒜ | 부팅 panic | 부팅 진행 · paints 4 · FAIL | `Unimplemented: 11: MC_dbGetRecordSize` — KTF DB 슬롯 11, 인자 배치 미측정(이 저장소의 DB 슬롯 이름은 KTF 의미와 어긋난 전례가 있어 추측 구현하지 않았다) |
| `c94d64777926` | ⒝ | 약 5분에 **SIGABRT**(호스트 스택 넘침) | 306초에 `tick error` — 예약 OOM · 조작 434/900 | 한 paint 안 NPE 무한 포획(원인 미식별: 무엇이 null 인가) |
| `e9fac881e602` | ⒝ | SIGABRT | 289초에 `tick error` — 예약 OOM · 조작 409/900 | 위와 같은 게임의 다른 판 · 같은 벽 |
| `6c96c5050b2b` | ⒝ | SIGABRT(전수) · 재현 1회는 `NoSuchFieldError [C.midpImage` | 600초 생존 · 조작 854/900 · paints 4,554 | 재현이 흔들린다 — 부하 의존 |
| `155972cac664` | ⒞ | FAIL(주소 0) | 동일 | DB 존재 판정: 첫 실행에 `MC_dbExists` 가 `FirstRun`·`Certification`·`Config` 모두 0 인데 게임은 `Config` 크기 0 으로 할당해 역참조(PC `0x148a32`). zip 의 `P/` 에 그 세 파일이 있다 — KTF DB 가 패키지 `P/` 를 못 본다 |
| `30c7bd6fb01b` | ⒞ | FAIL(주소 0 · 때로 PC=4) | 흔들림(이번 재측 PASS) | 없는 저장 슬롯 파일 → `MC_knlCalloc(0)` = NULL(상류 `4b3bbef2` 의 의도된 규칙) → `ldr r1,[r0]`(PC `0x137e0e`) |
| `568c339a8c07` | ⒞ | 부팅 FAIL(주소 72) | 동일 | 타이머 콜백이 `MC_grpFillRect` 직후 `[[sp+0x3c]+8]` = `0x4a` 를 포인터로 읽는다(PC `0x119896`) — 그 구조체를 누가 채우는지 미식별 |
| `2b1ed0c8d061` | ⒞ | 전수 FAIL(주소 0) | 이번 재측 전/후 PASS | 재현 안 됨(부하 의존) |

★⒞ 에서 버린 가설: 「`MC_knlUnsetTimer` 가 스텁이라 해제된 타이머가 발화한다」 — `568c339a8c07` 은 타이머 1개·해제 0회로 죽는다. 스텁 자체는 사실이다(`kernel.rs` `unset_timer`).

### 퇴행
전(`main f12e16e8`) / 후 같은 부하에서 `--inject --timeout 30`:
- 라이브 LGT 5종 `13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684` — **5/5 PASS → PASS**.
- 가드 `49ade89578c5` `ddd885583b15` — **PASS → PASS**.
- KTF playable 표본 10(census 등급 순 15번째마다) — 9 PASS → PASS. `888702965551` 은 후 1회 FAIL(네이티브 PC `0x7100883a` 주소 24)이었고 **전/후 각 3회 재측 6/6 PASS** — 굶음. 이 변경이 닿는 경로(jar `.class` 3종 · 할당 실패)에 이 타이틀은 없다.

### 변이(되돌리면 red)
- `load_class` 의 jar-class 분기를 끈다 → `test_load_class_prefers_client_bin_over_jar_bytecode` red(`org/kwis/msf/io/Network` 가 부트스트랩 쪽).
- `instantiate_array` 를 `jvm.exception(…)` 로 되돌린다 → `test_allocation_failure_throws_reserved_out_of_memory_error` red(던져진 것이 예약 OOM 이 아니다).

### 한계
- ⒝ 는 «탭이 죽는다 → 게임이 멈춘다»로 **한 계급 내렸을 뿐**이다. NPE 루프가 진짜 벽이고, 실기에서는 GC 가 할당 실패 때 돌아 힙이 차지 않았을 것이다. 여기서 할당 실패 때 GC 를 돌리지 않은 이유: KTF 게스트는 ARM 스택·레지스터에 객체 포인터를 쥐고 있고 GC 는 그것을 못 본다 — 게스트 실행 중 수거는 살아 있는 객체를 지운다.
- 예약 OOM 은 JVM 하나에 1개이고 공유된다(JVM 들의 관례). `jar_name` 이 없는 시험 JVM 에는 없다 → 종전 경로.
