## [2026-09-28] 전수 점검 KTF `KtfClassLoader` 생성 예외 8종 — 원인 4갈래 판정 · 2종 수리 · 머리 표 포맷 명명 (wie-census-ktf-classloader-header-table-8)

**무엇을**
- `wie-wipi-java` — `java/io/UnavailableException` 등록(WIPI 클래스 · `RuntimeException` 하위 · `<init>()V`·`<init>(String)V`, AromaWIPI `java/io/UnavailableException.class` 그대로). RustJava 런타임에 없어 클래스 목록 초기화가 `NoClassDefFoundError` 로 멈췄다.
- `wie-ktf/src/emulator.rs` — `jar_filename()`: `<AID>.jar` 가 없고 아카이브에 `.jar` 가 **딱 하나**면 그것을 쓴다. 0개·여럿이면 종전 이름을 그대로 둔다.
- `wie-ktf/src/adf.rs` `is_relocation_prefixed()` + `runtime/init.rs` — 재배치 표로 시작하는 `client.bin` 을 알아보고 **이름 있는 오류**로 멈춘다. 종전에는 표를 코드로 실행해 `Invalid memory access; address: 28` 이 났다.

**왜**: 전수 점검(`docs/report/0321`) 군집 「boot:fail · `jvm_support.rs` JavaException」 8종. 티켓 관측은 「5종이 머리 표」였지만, 실측하니 원인이 넷으로 갈렸다.

**사용자 영향**: KTF 1종이 부팅·화면·조작까지 된다. KTF 1종은 시작 화면·조작까지 가지만 간헐적인 다음 벽이 있다(아래). 나머지 6종은 그대로이고, 그 이유를 이름 붙여 남겼다.

### 판정 — 8종이 4갈래
| sha12 | 원인(실측) | 이번 회차 |
|---|---|---|
| `0392263fbb85` | `NoClassDefFoundError: java/io/UnavailableException` | **수리** |
| `d552e095ddcf` | `__adf__` AID 와 아카이브 속 jar 이름이 다르다 → `JarFile` 생성 `FileNotFoundException` | **수리**(다음 벽 있음) |
| `1d5831e42a8a` `83fc429f9cbe` `b907b0faf483` | `client.bin` 이 재배치 표로 시작하는 다른 이미지 ABI | 포맷 확정 · 명명 오류 |
| `bfa8ec352451` | `NoClassDefFoundError: com/ktf/kfc/GMenubarForm` — KTF 단말 UI 라이브러리 `com/ktf/kfc/*` 6클래스 | 범위 밖(참조 자료 0) |
| `60bd6cbc5936` `dab2d537f3ef` | jar 자리의 파일이 zip 이 아니라 OMA DRM `odcf` 컨테이너(암호화) | 범위 밖(복호 불가) |

티켓의 「머리 표 5종」은 **3종**이다. 나머지 2종(`0392…`·`bfa8…`)은 `client.bin` 첫 16바이트가 working KTF 190종 전건과 같은 표준 진입 스텁(`04e0c046 24020420`)이다.

### 머리 표 포맷(3종) — 확정한 것 / 못 한 것
- **컨테이너**: `[u32 bss][u32 count][count × u32 이미지 오프셋, 오름차순][이미지]`. 첫 워드가 파일 이름 `client.binN` 의 N 과 같다(3/3: 64·64·40). 오프셋이 가리키는 워드(이미지 0x0·0x8·0xc…)는 전부 이미지 안 포인터값이다.
- **표준 포맷과의 관계**: 표준 `client.bin` 의 진입 스텁(0x0–0x63)도 **같은 표**(0x70 부터 count+오프셋)를 읽어 이미지를 재배치한다. 그다음 이미지 머리 `[재배치2 시작, 끝, bss 시작, bss 끝, WipiExe]` 로 bss 를 0 으로 채우고 `WipiExe` 를 돌려준다. 머리 표 포맷은 그 스텁이 **없고**, 이미지 머리도 다르다(`+0x20`·`+0x28` = `0x13580001`, `+0x24` = Thumb 함수, `+0x0c` = `MNInterface`·예외 클래스 이름 표).
- **못 한 것 — 이미지 ABI**: 세 이미지 모두 `WIPI_exe` 문자열이 **0건**이다. 코드는 `sl`(r10)을 GOT 기준으로 쓰는 위치 독립 코드다(`ldr r3,[pc]; mov sl,r3; add sl,pc` 후 `[sl+off]`). 표준 로더의 `WipiExe → ExeInterface → fn_init` 경로가 없다. 메서드 이름 표기(`()V+init_…`)는 표준과 같다. ⇒ 재배치만 해서는 진입점이 없다. 이 ABI(진입·인터페이스 표·`sl` 설정)를 푸는 것은 별 회차다. 이번에는 추측으로 실행하지 않고 **이름 붙은 오류**로 멈추게 했다.
- 검출이 표준 190종을 잘못 잡을 수 없는 이유: 표준 파일의 첫 워드는 스텁 `0x46c0e004` 라 파일 이름 bss 와 같을 수 없다.

### 전/후 (wie_validate · release · host load1 167–223)
| sha12 | 전 | 후 · 부팅/화면 | 후 · 조작(`--inject`) | 후 · 장시간 |
|---|---|---|---|---|
| `0392263fbb85` | FAIL · `NoClassDefFoundError: java/io/UnavailableException` | **PASS** · paints 147 · distinct 139 | **PASS** · 27/27 · paints 39 · distinct 80 | 아래 「장시간」 |
| `d552e095ddcf` | FAIL · `FileNotFoundException` | 4회 중 **PASS 2** (paints 15·19) · **panic 2** | **PASS** · 27/27 · paints 16 · distinct 6 | 미측(다음 벽이 간헐) |
| `1d5831e42a8a` | FAIL · `Invalid memory access; address: 28` | FAIL · `Unsupported KTF client.bin layout: client.bin64 starts with a relocation table…` | — | — |
| `83fc429f9cbe` | 같음 | FAIL · 같은 명명 오류(`client.bin64`) | — | — |
| `b907b0faf483` | 같음 | FAIL · 같은 명명 오류(`client.bin40`) | — | — |
| `bfa8ec352451` | FAIL · `com/ktf/kfc/GMenubarForm` | 불변 | — | — |
| `60bd6cbc5936` `dab2d537f3ef` | FAIL · `ZipException: Could not find EOCD` | 불변 | — | — |

**`d552e095ddcf` 의 다음 벽**: 게스트가 `NumberFormatException: For input string: ""` 을 잡은 뒤 가끔 `new String((char[]) null)` 에 닿는다. 그러면 RustJava 런타임 `String.<init>([C…)` 가 NPE 대신 **호스트 패닉**을 낸다(`jvm-0.1.1/src/class_instance.rs:108:32` · 백트레이스 `rustjava_runtime::classes::java::lang::string::String::init_with_char_array`). #350 의 `Clip::player` 널 가드와 같은 줄이지만 다른 호출자다. 가드 자리는 `wie_jvm_support/src/hardening.rs`(핀이 안 싣는 널 가드)다. 별 회차다.

### 장시간
`0392263fbb85` `--timeout 600`(입력 없음): **PASS** · paints 2967 · distinct 139 · stop deadline · Java 예외 0 · 패닉 0.

### 퇴행
라이브 LGT 5종 + 가드 2종. 같은 바이너리 조건(`--timeout 20`)으로 전/후를 쟀다:

| sha12 | 전 | 후 |
|---|---|---|
| `13d7e3c21856` | PASS 97/63 | PASS 101/63 |
| `1b107b96bf4e` | PASS 113/62 | PASS 123/62 |
| `4ece6eeeaa04` | PASS 2/36 | PASS 2/36 |
| `a30bbe008b5e` | PASS 100/19 | PASS 84/19 |
| `b475b6399684` | PASS 32/2 | PASS 19/2 |
| `49ade89578c5` | FAIL no frame (deadline) | FAIL no frame (deadline) — 동일 |
| `ddd885583b15` | FAIL no frame (deadline) | FAIL no frame (deadline) — 동일 |

(paints/distinct. paints 는 부하에 따라 움직이는 하한이다 — AGENTS.md §runner.)

### 시험 · 변이
- 새 시험 3: `test_unavailable_exception_is_a_runtime_exception`(wie-wipi-java) · `jar_filename_falls_back_to_the_only_jar` · `relocation_prefixed_client_bin_is_detected`(wie-ktf).
- 변이(실측 · 각각 되돌린 뒤 green 확인):
  - M1 `get_protos()` 에서 등록 제거 → 시험 1 **FAILED**
  - M2 폴백 갈래를 `expected` 로 → 시험 2 **FAILED**
  - M3 검출 함수 `return false` → 시험 3 **FAILED**
- 게이트: `cargo fmt --check` OK · `clippy --all -D warnings` OK · wasm32 clippy OK · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(실패 0).
- 진입점 호출(`init.rs`)을 직접 잡는 시험은 없다. 3종의 실행 결과 문구(위 표)가 그 증거다.
