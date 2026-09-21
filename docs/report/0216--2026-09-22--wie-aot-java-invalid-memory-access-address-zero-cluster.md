## [2026-09-22] `Invalid memory access; address: 0` 는 «하나의 벽»이 아니라 «세 개»였다 — 둘을 근원에서 닫고, 배틀몬스터의 것은 이름을 붙였다 (wie-aot-java-invalid-memory-access-address-zero-cluster)

### 무엇을

LGT AOT-Java 9종이 공유하던 문면 `net.wie.WieError: Invalid memory access; address: 0` 을
**어느 코드 줄이 내는지**까지 계측으로 갈랐다. 하나인 줄 알았던 벽이 **셋**이었고, 그중 둘은
근원에서 닫혔다. 엔진 3파일 · 시험 2건. 본문 +30 / −7(시험 제외).

| 파일 | 무엇 |
|---|---|
| `wie-lgt/src/runtime/java/jvm_support/vtable.rs` | 배치 불가한 선언 가상 메서드마다 vtable 슬롯을 **예약** |
| `wie-lgt/src/runtime/java/interface.rs` | 임포트 표의 **빈 슬롯**(name·descriptor 둘 다 null)을 건너뛴다 + 시험 |
| `wie-lgt/src/runtime/init.rs` | 게스트 진입점이 init 구조체를 안 내놓은 경우를 **이름 붙여** 보고 |
| `wie-lgt/src/runtime/java/jvm_support.rs` | 시험 1건(`java/lang/Runtime` vtable 이 index 13 을 덮는가) |

### ⑴ 그 문면을 내는 «소스 위치» — 다섯 자리이고, 이 클러스터가 밟은 것은 셋이다

`WieError::InvalidMemoryAccess(0)` 를 낼 수 있는 자리는 전부 다음 다섯이다(전수 실측):

| 자리 | 뜻 |
|---|---|
| `wie-core-arm/src/engine/arm32_cpu.rs` `if pc < 0x1000` | 게스트가 **주소 0 으로 분기**했다 |
| 같은 파일 `arm32cpu_memory.memory_error` | 게스트 **적재/저장**이 미매핑 주소를 짚었다 |
| `wie-util/src/lib.rs` `read_generic` / `read_null_terminated_string_bytes` / `write_generic` / `write_null_terminated_string_bytes` 의 `address == 0` 가드 | **Rust 쪽**이 널 포인터를 역참조했다 |

★**문면은 다섯을 구별하지 못한다** — `Display` 가 주소 숫자만 찍는다(`wie-util/src/lib.rs`).
그래서 이 회차는 다섯 자리에 **임시 라벨**을 박아 각 타이틀을 다시 돌렸다(계측 후 전량 되돌림).

| 타이틀 | 라벨 | 즉 무엇이 0 인가 |
|---|---|---|
| **배틀몬스터** · 학교가는길 · 체스마스터 | `PROBE-BRANCH pc=0x0` | ★**게스트가 «vtable 슬롯»에서 읽은 함수 포인터** |
| 서든어택포켓 · 스파이더맨3 · 훼밀리마트타이쿤 · 레전드오브마스터 · 턴 | `PROBE-READ_CSTR` | ★**임포트 표 슬롯의 «멤버 이름 포인터»** |
| SD한국전쟁 | `PROBE-READ_GENERIC` | ★**`InitParam1.ptr_init_struct`** |

★**티켓이 「유일한 공짜 단서 = `LgtClassLoader.findClass` 프레임 ⇒ 거기서 시작하라」고 했는데,
그 프레임을 가진 두 종(서든어택포켓·스파이더맨3)은 배틀몬스터와 «같은 근인이 아니다».**
그 프레임을 좇았으면 배틀몬스터에는 닿지 못했다.

### ⑵ 배틀몬스터 — «무엇이 0 인가»의 이름

★**`java/lang/Runtime` 의 vtable 엔트리 index 13** 이고, 그것은 ★**wie 가 그 클래스에 만든
10칸짜리 표의 «밖»** 이다. 게스트는 그 한 칸 너머를 읽어 0 을 얻고 거기로 분기했다.

게스트 코드(`binary.mod` `.text` @`0x5588`, ARM 모드 · `lr=0x5598` 이 가리킨 자리):

```
0x5588: ldr r3, [r3]         ; r3 = instance->ptr_dispatch_table
0x558c: ldr ip, [r3, #0x38]  ; 0x38 = 4*(13+1) ⇒ vtable 엔트리 index 13
0x5590: mov lr, pc
0x5594: bx  ip               ; ip == 0
```

계측 시점의 그 표 전문(라벨 프로브가 읽어 온 값):

```
r3=0x49854200 words=[ptr_class, 0x710000c1, 0x71000011, 0x710000d1, 0x71000031, 0x71000051,
                     0x710000e1, 0x710000f1, 0x71000101, 0x71000111, 0x71000121, 0, 0, 0, …]
ptr_class → name="java/lang/Runtime"
```

= **엔트리 0~9 만 실재**(=`lgt_java_abi.toml` 의 `java/lang/Object` `vtable_size = 10`),
`JavaVtable::allocate` 가 잡은 크기는 `(10+1)*4 = 0x2c` 바이트인데 게스트는 **`+0x38`** 을 읽는다.

★**근원**: `JavaVtable::build_methods` 는 선언된 가상 메서드 중 **ABI 행도 상속 자리도 없는 것**을
`continue` 로 **버렸다**. `java/lang/Runtime` 은 `data/lgt_java_abi.toml` 에 **행이 없고**, 그 클래스의
가상 메서드는 `totalMemory()J` · `freeMemory()J` · `gc()V` · `exit(I)V` **넷**이다 ⇒ 넷 다 버려져
표가 Object 의 10칸에서 멈췄다.

★**처방은 「널 가드」가 아니라 「자리를 비워 두기」다** — 버리는 대신 **슬롯을 예약**한다.
`JavaVtable::write` 는 target 0 을 이미 `SVC_CATEGORY_MISSING_JAVA_VTABLE_ENTRY` 스텁으로 바꾸므로,
예약된 칸은 **자기 이름을 말하는 벽**이 된다.

```
before: net.wie.WieError: Invalid memory access; address: 0
after : net.wie.WieError: Unimplemented: java/lang/Runtime vtable index 13
```

★**index 13 이 «어느 메서드인가»는 이 회차가 말하지 않는다 — 추측으로 채우지 않았다.**
좁힌 것까지만 적는다: ⒜네 후보 중 **`exit(I)V` 는 제외**된다(위 게스트 코드가 인자 레지스터를
세팅하지 않는다) ⒝그 호출은 **꼬리 호출이고 결과가 그대로 반환**된다(`0x5598: ldm sp,…; bx lr`)
⒞앞선 두 호출이 클래스 게터와 `getRuntime()` 자리다. ⇒ 남는 것은 `totalMemory`/`freeMemory`/`gc`.
★**고르려면 LGT 의 ordinal 표가 필요하고 그것은 티켓이 범위 밖으로 명시한 축이다.**
★**순서를 지어내 채우면 최악의 경우 `exit(I)V` 를 부르게 된다 — 그래서 스텁으로 남겼다.**

### ⑶ 배틀몬스터 `ticks`·`paints` 전/후 — ★**바뀌지 않았다. 부풀리지 않는다**

| | ticks | paints | 벽 |
|---|---|---|---|
| before | **2** | **0** | `Invalid memory access; address: 0` |
| after | **2** | **0** | `Unimplemented: java/lang/Runtime vtable index 13` |

★**이 회차가 배틀몬스터에 대해 산 것은 «진척»이 아니라 «이름»이다.**
티켓 §필수②⒜ 가 허용한 그 경로이고, 그 문면이 다음 회차의 입력이다.
★**화면은 여전히 안 나온다.** 같은 벽을 **학교가는길·체스마스터**가 공유하므로,
그 한 칸을 채우면 **3종이 동시에** 다음으로 간다.

### ⑷ 보조 표본 2종은 «같은 근인이 아니다» — 티켓의 전제를 뒤집는다

`서든어택포켓`·`스파이더맨3` 의 0 은 **`LgtClassLoader.findClass` → `java_link_public_class` →
`link_class_members` → `read_member_name_and_descriptor`** 의 **멤버 이름 포인터**다.
라벨 프로브가 슬롯 좌표까지 찍었다 — 전건 **`instance_field` 임포트 표**이고, 전건 **범위 «안»**이다:

| 타이틀 | 클래스 | 널 슬롯 | 요청 범위 |
|---|---|---|---|
| 서든어택포켓 | `SAttack` | index **6** | offset 5 · count 9 |
| 스파이더맨3 | `a` | index **2** | offset 0 · count 75 |
| 훼밀리마트타이쿤 | `c_bf` | index **8** | offset 4 · count 21 |
| 레전드오브마스터 | `f` | index **51** | offset 1 · count 412 |
| 턴 | `GameCanvas` | index **10** | offset 2 · count 12 |

★**표 «끝을 넘어선 것»이 아니라 표 «가운데 구멍»이다** — 그리고 name·descriptor 가 **둘 다 0** 인
깨끗한 빈 쌍이다. ⇒ LGT 컴파일러가 남긴 **빈 슬롯**이고, 링커는 그 칸을 **건너뛰어야** 한다.
`read_member_name_and_descriptor` 가 `Option` 을 돌려주게 하고 다섯 루프가 `continue` 한다.

★**「건너뛰기」와 「0 을 써 넣기」를 «실측으로» 갈랐다** — 빈 칸에 word index 0 을 써 보는 대조군을
돌렸고 5종 결과가 **전건 동일**이었다. ⇒ 그 칸을 게스트가 쓰지 않는다는 증거이고, 그래서
**아무것도 쓰지 않는 쪽**(더 짧은 diff)을 골랐다.

### ⑸ 벽이 옮겨 간 곳 — 문면 그대로 (release `wie_validate` · `--inject` 기본 예산 20.0s)

| 타이틀 | before | after |
|---|---|---|
| **배틀몬스터**(2파일) | t2 p0 · `Invalid memory access; address: 0` | t2 p0 · ★`Unimplemented: java/lang/Runtime vtable index 13` |
| 학교가는길 | t3 p0 · 같은 문면 | t3 p0 · ★같은 `Runtime` index 13 |
| 체스마스터 | t1 p0 · 같은 문면 | t1 p0 · ★같은 `Runtime` index 13 |
| 훼밀리마트타이쿤 | t2 p0 · 같은 문면 | t2 p0 · ★`Unimplemented: java/lang/String vtable index 19` |
| 서든어택포켓(2파일) | t1 p0 | ★**t116 / t835** p0 · `no frame rendered (hang/black screen)` |
| 턴(2파일) | t1 p0 | ★**t204 / t924** p0 · `no frame rendered (hang/black screen)` |
| 레전드오브마스터(2파일) | t2 p0 | ★**t41 / t46 · paints 1** · `panic during '09_UP'/'12_OK': Result::unwrap() on Err(InvalidMemoryAccess(0))` |
| 스파이더맨3 | t1 p0 | t1 p0 · `panic during 'boot': Result::unwrap() on Err(InvalidMemoryAccess(3321430483))` |
| SD한국전쟁 | t0 p0 · `Invalid memory access; address: 0` | t0 p0 · ★`LGT entrypoint returned without publishing an init struct (ptr_init_struct is null)` |

★**부팅을 통과한 것이 4종이고, 그중 레전드오브마스터는 «처음으로 한 번 그린다»(paints 0 → 1).**
★**그러나 «렌더한다»고 쓰지 않는다** — 1 프레임이고 곧 패닉한다. 부풀리지 마라.

★★**새로 드러난 것이 «패닉»이다 — 숨기지 않는다.** `스파이더맨3`·`레전드오브마스터`가 옛 벽을
지난 뒤 `impl ClassInstance for JavaClassInstance` 의 `unwrap()` 무리
(`value.rs` `object_from_raw` · `class_instance.rs` `class_definition`)에 닿는다. 그 자리들은
`jvm` 트레이트에 오류 통로가 없어 복구 가능한 `WieError` 를 패닉으로 바꾼다.
★**이 회차가 만든 결함이 아니라 «도달 가능해진» 선재 결함**이고, ★**관측 손실은 없다** —
`wie_validate` 가 패닉을 잡아 `panic during '<step>'` 으로 **정상 JSON** 을 낸다(실측).
★**고치지 않았다**: 통로 없는 트레이트 메서드에서 «무엇을 돌려줄 것인가»가 별개 설계 결정이고,
그 객체 자체가 쓰레기 참조(`0xC5F3_…`)라 가려도 다음 줄에서 죽는다. ⇒ 후속 티켓 몫.
★실제로 `object_from_raw` 의 `unwrap` 만 제거해 봤고 **패닉이 한 프레임 이동했을 뿐**이라
그 변경은 **버렸다**(측정으로 값을 못 한 diff 는 남기지 않는다).

### 회귀 — `game_lab/broken/lgt` 46파일 전건 before/after 짝지어 실행

★**before 바이너리를 «이 회차가 직접 만들어» 같은 분(minute)에 번갈아 돌렸다**(AGENTS.md 의
짝지음 규율). 패치를 되돌려 빌드 → `/tmp/wie_before`, 적용해 빌드 → `/tmp/wie_after`.

- **판정·벽이 그대로**: 36/46 · **바뀐 것 10건**은 전부 위 표의 대상이다.
- **`PASS → non-PASS` 1건**, `non-PASS → PASS` 0건.
- **`LGT 크로이센`·`크로이센` 은 before·after 둘 다 `NO-JSON`** — 선재이고 이 회차와 무관.

★★**그 1건(`현영맞고2006`)은 ★재현되지 않았다 — 부하 축이다.**
스윕 회차: before `PASS t35.2M p144` ↔ after `FAIL t14.8M p139`
(`tick error during '16_LSOFT': Invalid memory access; address: 0`, PC=네이티브 스텁).
★**같은 짝을 25쌍 더 돌렸다 — before 26/26 PASS · after 25/26 PASS**(부하 load1 **27~97** 구간 전반).
⇒ ★**1/26 대 0/26 은 이 표본 수에서 구별되지 않는다**(Fisher 정확검정 p=1.0).
★**「회귀 없음」이라고 단정하지 않는다 — 「이 n 에서는 구별되지 않는다」가 잰 것의 전부다.**
★AGENTS.md 4단계를 그대로 밟았다: ⑴여러 번 재실행 ⑵`paints` 를 정상 범위와 대조(139 ↔ 144·155·241 —
기아 형태가 아니다) ⑶**손대지 않은 쪽에서 재현 시도**(26회 전건 PASS) ⑷`uptime` 기록.

### 측정 위생 — 호스트 부하와 예산

- **load1**: 계측 구간 전반 **23.6 → 202.5** 사이를 오갔다(딴 세션이 같은 맥에서 빌드 중이었다).
  스윕 실행 구간 **52~97** · 현영맞고 재측 구간 **27~97**.
- **예산**: 전 회차 `--inject` **기본값**(= `2.5 + 0.3 + 27×0.6 + 1.0` = **20.0s**). `--timeout` 은
  `--inject` 경로에서 덮어쓰이므로 쓰지 않았다.
- ★**`ticks` 를 절대값으로 인용하지 않는다** — 같은 타이틀·같은 바이너리에서 `현영맞고2006` 이
  **0.43M ~ 50M** 으로 움직였다(26회). 이 회차가 읽은 축은 **판정과 벽 문면**이다.
- ★**진단 자체는 부하에 무관했다** — 세 근인 전건이 **debug 빌드에서 tick 0~3 에 결정론적으로**
  재현됐다(9종 전건 · 2회 이상). 부하가 흔든 것은 렌더까지 가는 타이틀뿐이다.

### 게이트

`cargo fmt --all -- --check` · `cargo clippy --all -D warnings` · `cargo clippy --target
wasm32-unknown-unknown -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` · `cargo +beta
clippy --all -D warnings` — **전건 rc=0**.
엔진 코드를 만졌으므로 **러너 블록도 돌렸다**: `draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·
`text_j2me` **PASS**, `keydraw_ktf`·`keydraw_lgt` **PASS · content true · rc=0 · paints 55**(양쪽 다
무부하 범위 48~55 안).

### 시험 — 둘 다 개악 대조로 «진다»는 것을 확인했다

| 시험 | 잠그는 것 | 개악 대조 |
|---|---|---|
| `jvm_support.rs::unplaceable_virtual_methods_still_occupy_vtable_slots` | `java/lang/Runtime` vtable 이 **index 13 을 덮고** 전 칸이 non-zero | 예약 한 줄 삭제 → **FAIL**: 「vtable is **10** entries, does not cover index 13」 |
| `interface.rs::empty_import_slot_reads_as_none` | 널 쌍 슬롯이 `None` · 양옆은 `Some` | 조건을 `false` 로 → **FAIL** |

★첫 시험은 **실제 클래스(`java/lang/Runtime`)** 로 잠근다 — 합성 클래스로 잠그면 이 버그가
「ABI 행이 없는 실클래스」에서 나온다는 사실이 시험 밖으로 새어 나간다.

### 손대지 않은 것

- `NoClassDefFoundError` 클러스터 · `일지매영웅전기` · §7 per-frame 모델 · `0x64` ordinal 표 획득 —
  전부 티켓이 범위 밖으로 명시한 축이다.
- `game_lab/` 커밋 **0** · 게임 바이트 커밋 **0** · 새 검사기/CI 스텝 **0** · main 무접촉 · 머지 0.
- 계측용 임시 라벨(`wie-core-arm`·`wie-util`·`wie-lgt` 7파일)과 임시 시험 하네스는 **전량 되돌렸다**
  (`git status` 로 확인 · 최종 diff 는 위 4파일뿐).
