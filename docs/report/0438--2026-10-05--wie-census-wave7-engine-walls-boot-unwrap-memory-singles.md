## [2026-10-05] 7차 엔진 벽 — KTF DB 슬롯 11 = 남은 저장 공간 · InitParam4 +8 = `throw e` · 부팅 panic → 오류 (wie-census-wave7-engine-walls-boot-unwrap-memory-singles)

**무엇을**: `docs/report/0432` §6 군집표 중 주인 없는 벽을 보았다. 고친 것 셋:
- KTF DB 슬롯 11(헤더 이름 `MC_dbGetRecordSize`)을 «남은 저장 공간(바이트)»으로 답한다(슬롯 12 와 같은 함수).
- KTF `InitParam4` +8(`wipi_types` 의 `unk1`, 종전 0)에 «만들어 둔 예외 객체를 던진다» 서비스를 넣었다.
- KTF 부팅에서 `KtfClassLoader`/`JarFile` 생성 예외가 `unwrap()` 로 호스트 panic 이 되던 것을 오류로 돌려준다.

**왜**: 운영자 지시(2026-09-30 · 10-04). 0432 §6 의 «1씩»·«5»·«2» 행에 주인 티켓이 없었다.

**사용자 영향**: 지원 현황 388/24/17 → **391/23/15**(playable 3 · not-yet → limited 2). §6 표. 타이틀은 sha12 로만 적는다.

### 1. 부팅 `unwrap JavaException` 5종 — panic 은 없앴고, 부팅하게 만들지는 못했다

| sha12 | 원인(실측) | 이 회차 |
|---|---|---|
| `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` | `client.bin` 이 재배치 표로 시작하는 다른 이미지 ABI. 막는 것은 `MNInterface`(30칸 이상 · 자료 0) | 0340 «구현하지 않는다» 판정 유지(다시 열 조건 셋 모두 미충족). 오류만 `panic … unwrap()` → `tick error … Unsupported KTF client.bin layout` |
| `60bd6cbc5936` `dab2d537f3ef` | jar 자리가 OMA DRM `odcf` 컨테이너(0330) | 안내 규칙(`drm`, #455)이 이미 compat 에 있다(«암호로 잠긴 파일…»). 오류만 panic → `ZipException: … EOCD` |

- 원인 API: `wie-ktf/src/runtime/java/jvm_support.rs` 의 `JarFile`·`KtfClassLoader` 생성자. 생성자가 client.bin 을 읽으며 Java 예외를 던지는데, 호출부가 `.unwrap()` 했다. 이제 `JvmSupport::to_wie_err` 로 스택 트레이스를 담은 오류가 된다.
- 효과는 «진단 문구»뿐이다. 다섯 타이틀의 compat 행은 바꾸지 않았다.

### 2. KTF DB 슬롯 11 — 호출부 역어셈으로 «남은 저장 공간»

종전에는 `gen_stub(11, "MC_dbGetRecordSize")` 라 부르는 순간 `Unimplemented` 로 멈췄다. 헤더(`docs/reference/WIPIHeader.h`) 시그니처는 `MC_dbGetRecordSize(fd)` 이다. 그러나 KTF DB 표는 헤더 순서와 다르다(4 = lseek · 5 = stat · 12 = 남은 공간, `database.rs` 의 기존 주석).

| 호출부(관측) | sha12 | 인자 | 결과를 어떻게 쓰나 |
|---|---|---|---|
| 저장 함수 `0x143454`(같은 엔진 두 판 — 바이트 동일) | `4a4d2ac046f7` `f981d228b757` | 인자 준비 없음(r0 = 직전 비교 결과 0) | 저장 파일이 없을 때만 부른다. `if (slot11() >= 저장 길이) 쓴다; else 버리고 0` — 부호 있는 비교(`bge`) |
| 부팅 `0x10c590` | `3c658a46bbfb` | 인자 준비 없음 | `> 0x176f` 이면 «공간 있음» 깃발 = 1 |

- 세 호출부 모두 **인자 없이 부르고 바이트 수와 비교**한다. 이것은 슬롯 12 의 계약(인자 없음 · 남은 바이트, 기존 호출자 0x100·0x1200 비교)과 같다. 그래서 같은 함수(`list_databases`)를 붙였다.
- 실측 경로: `4a4d2ac046f7` 은 진단 빌드(커밋 0)로 DB 서비스의 `lr` 을 찍어 `MC_dbExists` 래퍼(`0x11e640`)를 찾았다. 그 래퍼가 쓰는 표 GOT 칸(`0x298`)으로 `[표, #0x2c]` 를 읽는 곳을 훑어 한 곳을 찾았다. `3c658a46bbfb` 는 진단 빌드의 `lr` 이 곧 호출부다.
- 시험: `database_slot_11_answers_free_storage`. 되돌리면 red(`Unimplemented("11: MC_dbGetRecordSize")`).

### 3. `InitParam4` +8 — «이미 만든 예외를 던진다»

`5267badf20b3`(부팅) · `55aadf368b8e`(장시간)이 `jump native address is null` 로 멈췄다. 두 이미지는 같은 던지기 도우미를 갖는다(`0x1385a8` · `0x147f10`, 바이트 동일).
- 도우미의 동작: `jump_2(e, 0, *(param4 + 8))`.
- `param4` 는 `fn_init` 의 다섯째 인자다. 이미지가 전역에 저장하는 것을 `0x137766` 에서 확인했다.
- +4 는 `fn_java_throw` 다. 같은 파일의 다른 도우미가 `jump_2(이름, 0, *(param4 + 4))` 로 부른다. 우리 구현은 그 인자를 **클래스 이름 문자열**로 읽는다.
- +8 은 0389 가 실측한 대로 `new Exception` 바로 뒤에 **객체**를 받는다.
- ⇒ +4 = «이름으로 만들어 던진다», +8 = «만들어 둔 것을 던진다»(`athrow`).

구현: `InitSvcId::JavaThrowInstance`. 받은 원시 포인터를 `JavaClassInstance` 로 감싸 기존 `JavaMethod::handle_exception` 에 넘긴다(잡는 블록이 있으면 그리로 가고, 없으면 `JavaException`). `null` 은 `athrow` 처럼 `NullPointerException` 을 던진다.
- 시험 `init_param_4_wires_throw_instance`: 되돌리면(`unk1: 0`) red.
- 시험 `throw_instance_throws_the_instance_it_is_given`: 받은 인스턴스가 그대로 올라온다 · null → NPE.

### 4. 장시간 주소 0/8 2종 — 재현 0

| sha12 | census(0432) | 이 회차 재현 시도(같은 장시간 키) | 결과 |
|---|---|---|---|
| `7da00ecd4804` | 19키 «주소 8» | 240초 1 · 400초 1(진단 빌드: 폴트 PC·레지스터 로그) | 0/2 — 339·568키까지 오류 없음 |
| `5a59f62d1f1a` | 60키 «주소 0» | 같음 | 0/2 — 같음(그림 축 «magenta 키 미적용» FAIL 은 별개 판정) |

- 진단 빌드는 쓰기 폴트(`memory_error`)와 낮은 PC 점프(`pc < 0x1000`) 둘 다에서 PC·LR·r0~r7 을 찍게 했다(커밋 0 · 되돌림). 두 번 다 닿지 않았다.
- 재현이 안 되므로 «쓰기를 삼키는» 처방은 넣지 않았다. 티켓 「실기 근거 없이 쓰기를 삼키지 마라」를 따랐다.
- 0391 의 근거(`7da00ecd4804`: 빈 이름 `""` 으로 `MC_knlGetResourceID` → -12)가 지금까지의 유일한 실측이다. 빈 이름이 어디서 오는지는 여전히 모른다.
- 크기: 재현 조건(부하·키 시점)을 먼저 찾아야 한다 — M.

### 5. 단발 7

| sha12 | 벽 | 이 회차 |
|---|---|---|
| `4a4d2ac046f7` | `MC_dbGetRecordSize` | §2 로 고침 |
| `55aadf368b8e` `5267badf20b3` | null 네이티브 점프 | §3 으로 고침 |
| `3151fdc167b6` | KTF 커널 36(`MXUserMemInterf` 이름 조회) | 0389 판정 그대로 — 반환 표의 slot0 이후 뜻을 모른다. 이번에 새 근거 없음 |
| `71d1d8235bd1`(SKT) | `NoClassDefFoundError: m/V3` | 0389 판정 그대로 — `m/V3`·`m/A3`·`m/XO_World` 3D 라이브러리(MBAC/MTRA 렌더러) |
| `01f05f8231f4`(LGT) | 그림 0 | 미조사. census 경고: LGT wipi_c `unk1`·`unk7`·`unk10`·`unk11` 스텁 · `MC_imHandleInput` 스텁 — 이 중 무엇을 기다리는지 재지 못했다 |
| `8b899f410f5d` | 검은 화면 | §3 으로 풀렸다 — §6(0435 의 «풀림»은 이 핀 main 에서 재현 안 됨) |

### 6. 전/후 측정 — 18종 짝 · 고친 5종 장시간 짝

- 1회차(2026-10-05 01:37~01:45 · guard rc=0 7배치): 5종 짝.
- 재배차(04:55~06:42 · 스윕 전체를 `build-slot run --long` 한 임대로 · 안쪽 맨 실행 · 동시 ≤3 · 배치마다 guard rc=0): 나머지 13종 A/B 짝 + 장시간 L 5종 × 2빌드.
- base = `origin/main` `9b4eb62e` · after = `c132e55b`. 그 뒤 커밋은 동작을 바꾸지 않는다(§9 의 `wie_validate` 스레드 · 시험 진단).

| sha12 | base | after | compat |
|---|---|---|---|
| `3c658a46bbfb` | A/B `Unimplemented: 11` FAIL · 그림 5 · L 부팅 FAIL | input ok(새 11) · 소리 ok · **L 600초 생존**(854/900 · 그림 11,559) | limited → **playable** |
| `4a4d2ac046f7` | A PASS · L 600초 생존 | A PASS · L 600초 생존 | limited → **playable**(longplay `no` 의 근거였던 슬롯 11 벽을 고쳤고, 이 핀 L 이 생존) |
| `f981d228b757` | A PASS · L 생존 | A PASS · L 생존 | 이미 playable — 변화 없음 |
| `5267badf20b3` | **boot FAIL**(null 네이티브 점프 · A·L 둘 다) | boot ok · render ok · input none(그림 1~2) · L 생존(그림 2) · 무음 | not-yet → **limited** |
| `55aadf368b8e` | A input ok · L 생존 | A input ok · L 생존 | limited → **playable**(근거 = 0432 의 null 점프 벽을 §3 이 고쳤고, 이 핀 L 이 생존) |
| `8b899f410f5d` | A/B 그림 2 · A `max-ticks` 9/27 · input none | **A PASS 27/27 · 그림 556/1004 · input ok(새 27) · 소리 ok** — 로그에 `java_throw_instance` | not-yet → **limited** |
| `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` | boot `panic … unwrap()` | boot `tick error … Unsupported KTF client.bin layout`(panic 0) | 변화 없음 |
| `60bd6cbc5936` `dab2d537f3ef` | boot `panic … unwrap()` | boot `ZipException … EOCD`(panic 0) | 변화 없음(잠김 안내 그대로) |
| 라이브 LGT 5 · 가드 2 | — | boot·render·input 축 7종 모두 같다 | 변화 없음 |

- **퇴행 0**:
  - `13d7e3c21856`(LGT)의 A 가 1회차에서 «PASS → `max-ticks` 15/27» 이었다. 판정 축은 같고, 변경은 KTF 경로뿐이다.
  - 나머지 6종은 결과·`stop` 까지 같거나 같은 축에서 흔들렸다(`4ece6eeeaa04` `a30bbe008b5e` 는 양쪽 다 `max-ticks`).
- `8b899f410f5d` 는 0435 가 «풀었다»고 적은 타이틀이다. 그러나 이 핀 main 에서도 그림 2 에서 멈췄다(A·B 둘 다). 이 회차의 `throw e` 서비스가 그 경로를 지난다(after 로그 `java_throw_instance` 1회). ⇒ 0435 의 «풀림»은 이 핀에서 재현되지 않았다. 이 회차의 고침으로 귀속한다.
- `4a4d2ac046f7` · `55aadf368b8e` 의 base 도 이번 L 에서 생존했다 — 두 벽은 키 시점에 따라 닿기도 하고 안 닿기도 한다(0432 §3-2). 그래서 «base 가 이번엔 죽었다»는 대비가 없다. playable 로 올린 근거는 둘이다: ① 0432 가 실측한 그 벽의 기전을 §2·§3 이 고쳤다 ② after 의 600초 L 이 생존했다.
- 재측 안 한 축: speed(종전 값 유지) · progress(재지 않음).

### 7. 측정 조건
- base = `origin/main` `9b4eb62e` 릴리스 · after = 이 브랜치 `c132e55b` 릴리스.
- 인자는 census 와 같다. 프로브 A = `--inject --keep-timeout --timeout 30 --relaunch 1 --pacing 8`. B = `… --inject-keys 0 --shot-every 1`. L = `LONG_KEYS` 60회 · 600초 · `--shot-every 20`. 판정도 `judge()` 와 같다(입력 = A 에 B 에 없는 프레임이 있나).
- 재배차분은 스윕 전체를 `~/orchestrator-live/bin/build-slot run --long` 한 임대로 감쌌고, 안쪽 실행은 맨 명령이다(PR #472 의 규칙).
- **census 도구를 쓰지 않았다.** 다른 레인의 진도 census 가 호스트 락을 90분 넘게 쥐고 있었다. 대신 `wie_validate` 를 직접 돌렸다. 한 번에 3개 이하 · 모두 `build-slot run` 경유 · 배치마다 `host-load-guard --status --recovered` rc=0 을 기다렸다. 진척은 `~/scratch/w7/progress.log`(로컬).
- 진도(`progress`) 축은 재지 않았다(§8).
- 예외: 첫 비공식 한 쌍(30초 A 3타이틀 × 2빌드)은 guard 가 «holding» 일 때 돌렸다. 그 숫자는 §6 표에 쓰지 않았다. 세 타이틀 모두 rc=0 배치에서 다시 쟀다(§6).

### 8. 후속

| 수 | 벽 | 계급 | 크기 |
|---|---|---|---|
| 17 | input none(잠금 1 제외) — 짝 2회 + 키 순서 4종 조사 | 측정(드라이버 `~/scratch/w7/inputstudy.py` 준비 · 미실행) | S(약 120회 × 30초) |
| 2 | 장시간 주소 0/8 — 재현 조건 | 엔진(재현 먼저) | M |
| 3 | 재배치 표 client.bin | 엔진 — 0340 판정 유지 | L |
| 1 | `01f05f8231f4` LGT 그림 0 | 엔진(스텁 중 무엇을 기다리나) | M |
| 1 | `3151fdc167b6` 커널 36 표 | 엔진(자료 0) | M~L |
| 1 | `71d1d8235bd1` SKT 3D | 엔진(3D 렌더러) | L |
| 1 | GUI 호스트(`wie_cli` 창)의 Windows 주 스레드 스택 여유 — 미측정(검증기는 §9 로 풀렸다) | 호스트 | S |

### 9. Windows CI 빨간색 — 주 스레드 스택 넘침(재배차에서 고침)

첫 제출 head `57e6bc02` 는 `rust_ci (windows-latest, stable)` 만 red 였다.
- push 3번 모두 red(3/3). 같은 시각 main · 다른 PR · Windows beta 는 green.
- 시험: `wie_cli/tests/validate_profile_out.rs` — 디버그 `wie_validate` 가 JSON 한 줄도 내지 못했다.

**원인(실측)**: 그 시험이 stderr 를 버리고 있었다. 그래서 종료 상태와 stderr 꼬리를 남기게 하고 다시 돌렸다(`a0362451`):
`exit code: 0xc00000fd` · `thread 'main' has overflowed its stack` — 키 없는 실행과 `--profile-out` 실행 모두.
- `wie_validate` 는 에뮬레이터를 **주 스레드**에서 돌렸다.
- Windows 주 스레드는 1 MiB 고정이다. CI 의 `RUST_MIN_STACK` 은 새로 띄우는 스레드에만 적용된다.
- §1 의 오류 경로(`to_wie_err` 를 부팅 future 안에서 기다린다)가 그 한도를 넘겼다.

**처음 시도는 틀렸다 — 남긴다.** macOS `ulimit -s` 이분 탐색으로 디버그 부팅의 주 스레드 필요량을 쟀다:

| 빌드 | 필요량 |
|---|---|
| main | 1016K |
| 첫 제출 | 1028K |
| 오류 서식기 future 를 `Box::pin` 한 판(`2ecc71f0`) | 1016K |

그런데 Windows stable 은 그 판에서도 red 였다. ⇒ macOS 수치는 Windows MSVC 디버그 프레임을 대신하지 못한다. 박스는 되돌렸다.

**고침**: `wie_validate` 가 에뮬레이터를 크기 지정 스레드(8 MiB = macOS/Linux 주 스레드와 같은 값)에서 돌린다.
- `std::thread::scope` + `spawn_scoped` 를 쓴다. `run()` 의 panic 은 종전처럼 그 안의 `catch_unwind` 가 잡는다. 바깥으로 나온 panic 은 `resume_unwind` 로 그대로 올린다.
- 이제 주 스레드 한도와 무관하다: macOS `ulimit -s 512` 에서도 PASS(전에는 출력 0).
- 되돌리면 red = Windows stable 의 위 실패(4 push).
- GUI 호스트(`wie_cli` 의 창)는 바꾸지 않았다. winit 이 주 스레드를 요구하기 때문이다. 그쪽 Windows 스택은 이 회차가 재지 않았다(§8).

### 10. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 336쌍 + SUFFIX-ATTACHED 15쌍. 이 회차가 쓴 줄 중 게임 이름은 0이다 — BOUNDED 의 나머지는 `compat.json` 의 기존 제목 값(1행만 값 수정 · 제목 무변경)과, 손댄 `wie-ktf` 세 파일에 이미 있던 주석(이 회차의 `+` 줄 0)이다. 타이틀은 sha12 로만 적었다.

<!-- corpus-name-inflow v1 subjects=14 tree=4f35016eeed2a44c B=725/336 P=1/1 S=35/15 -->
