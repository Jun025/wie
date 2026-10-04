## [2026-10-05] 7차 엔진 벽 — KTF DB 슬롯 11 = 남은 저장 공간 · InitParam4 +8 = `throw e` · 부팅 panic → 오류 (wie-census-wave7-engine-walls-boot-unwrap-memory-singles)

**무엇을**: `docs/report/0432` §6 군집표 중 주인 없는 벽을 보았다. 고친 것 셋:
- KTF DB 슬롯 11(헤더 이름 `MC_dbGetRecordSize`)을 «남은 저장 공간(바이트)»으로 답한다(슬롯 12 와 같은 함수).
- KTF `InitParam4` +8(`wipi_types` 의 `unk1`, 종전 0)에 «만들어 둔 예외 객체를 던진다» 서비스를 넣었다.
- KTF 부팅에서 `KtfClassLoader`/`JarFile` 생성 예외가 `unwrap()` 로 호스트 panic 이 되던 것을 오류로 돌려준다.

**왜**: 운영자 지시(2026-09-30 · 10-04). 0432 §6 의 «1씩»·«5»·«2» 행에 주인 티켓이 없었다.

**사용자 영향**: §6 표. 타이틀은 sha12 로만 적는다.

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
| `8b899f410f5d` | 검은 화면 | 이미 0435(176폭 상태줄 18행)가 풀었다(그쪽 짝 재측 2/2). compat 행은 그 회차가 갱신하지 않았다 — §6 |

### 6. 전/후 측정 — 잰 것만 적는다

guard rc=0 배치는 01:37~01:45 의 7배치(21회)뿐이었다. 그 뒤 01:46 부터 제출 때까지 guard 는 «saturated/holding/recovering» 이었다(다른 레인의 부하 · idle 0~1%). 그래서 아래 다섯 타이틀만 짝이 있다. 나머지 짝과 장시간 L 은 재지 못했다(§8).

| sha12 | 축 | base `9b4eb62e` | after `c132e55b` |
|---|---|---|---|
| `3c658a46bbfb` | boot · render · input · sound | ok · ok · **none**(새 프레임 0) · 무음 — A/B 둘 다 `Unimplemented: 11` FAIL · 그림 5 | ok · ok · **ok**(새 프레임 11) · **ok**(midi 234) — 예외 0 · 그림 133/143 |
| `1d5831e42a8a` | boot | fail — `panic … unwrap() … JavaException` | fail — `tick error … Unsupported KTF client.bin layout`(panic 0) |
| `13d7e3c21856`(라이브 LGT) | input | ok(새 27) · A PASS | ok(새 14) · A `max-ticks` 15/27 |
| `1b107b96bf4e`(라이브 LGT) | input | ok(새 25) · `max-ticks` 26/27 | ok(새 25) · `max-ticks` 26/27 |
| `49ade89578c5`(가드) | input | ok · PASS | ok · PASS |

- 퇴행 0. `13d7e3c21856` 의 «A PASS → `max-ticks`»는 LGT 타이틀이다. 이 변경은 KTF 경로만 바꾼다(DB 표 · `InitParam4` · KTF 부팅). 그래서 퇴행으로 읽지 않았다. 판정 축(boot·render·input)은 같다.
- `3c658a46bbfb` compat 행: input `no`→`ok` · sound `no`→`ok` · longplay `no`→`unknown`. 종전 `no` 는 30초 프로브의 FAIL 에서 나온 값이었고, 이 핀에서 장시간은 재지 않았다. status 는 `limited` 그대로다.
- guard «holding» 중에 돌린 비공식 1회(§7 예외)도 방향이 같았다: `1d5831e42a8a`·`60bd6cbc5936` panic → 오류 · `3c658a46bbfb` 그림 5 → 125.
- **고쳤지만 이 회차에 짝이 없는 것**: `4a4d2ac046f7` `f981d228b757`(슬롯 11 은 저장할 때만 닿는다 — 장시간 L 필요) · `5267badf20b3` `55aadf368b8e`(§3). 근거는 역어셈과 시험뿐이다. 그래서 compat 행·소식에 넣지 않았다.

### 7. 측정 조건
- base = `origin/main` `9b4eb62e` 릴리스 · after = 이 브랜치 `c132e55b` 릴리스.
- 인자는 census 와 같다. 프로브 A = `--inject --keep-timeout --timeout 30 --relaunch 1 --pacing 8`. B = `… --inject-keys 0 --shot-every 1`. L = `LONG_KEYS` 60회 · 600초 · `--shot-every 20`. 판정도 `judge()` 와 같다(입력 = A 에 B 에 없는 프레임이 있나).
- **census 도구를 쓰지 않았다.** 다른 레인의 진도 census 가 호스트 락을 90분 넘게 쥐고 있었다. 대신 `wie_validate` 를 직접 돌렸다. 한 번에 3개 이하 · 모두 `build-slot run` 경유 · 배치마다 `host-load-guard --status --recovered` rc=0 을 기다렸다. 진척은 `~/scratch/w7/progress.log`(로컬).
- 진도(`progress`) 축은 재지 않았다(§8).
- 예외: 첫 비공식 한 쌍(30초 A 3타이틀 × 2빌드)은 guard 가 «holding» 일 때 돌렸다. 그 숫자는 §6 표에 쓰지 않았다. `3c658a46bbfb`·`1d5831e42a8a` 는 rc=0 배치에서 다시 쟀다. `60bd6cbc5936` 은 다시 재지 못했다.

### 8. 후속

| 수 | 벽 | 계급 | 크기 |
|---|---|---|---|
| 17 | input none(잠금 1 제외) — 짝 2회 + 키 순서 4종 조사 | 측정(드라이버 `~/scratch/w7/inputstudy.py` 준비 · 미실행) | S(약 120회 × 30초) |
| 2 | 장시간 주소 0/8 — 재현 조건 | 엔진(재현 먼저) | M |
| 3 | 재배치 표 client.bin | 엔진 — 0340 판정 유지 | L |
| 1 | `01f05f8231f4` LGT 그림 0 | 엔진(스텁 중 무엇을 기다리나) | M |
| 1 | `3151fdc167b6` 커널 36 표 | 엔진(자료 0) | M~L |
| 1 | `71d1d8235bd1` SKT 3D | 엔진(3D 렌더러) | L |

### 9. 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 334쌍 + SUFFIX-ATTACHED 15쌍. 이 회차가 쓴 줄 중 게임 이름은 0이다 — BOUNDED 의 나머지는 `compat.json` 의 기존 제목 값(1행만 값 수정 · 제목 무변경)과, 손댄 `wie-ktf` 세 파일에 이미 있던 주석(이 회차의 `+` 줄 0)이다. 타이틀은 sha12 로만 적었다.
