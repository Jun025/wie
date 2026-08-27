# upstream 재정렬 판정 (2026-08-27)

> 회차 티켓 `wie-upstream-realign-verdict`. **판정 문서다 — 이 회차는 집행하지 않는다.**
> 운영자 지시(2026-08-27): 「이들은 모두 public github 를 포크해서 가져온건데 굳이 내가 private 으로
> 따로 해야하는건지 의구심이 든다. 불필요한 private 의존을 버려 양질의 기능들과 최신화된 기능들을
> 사용하는 방향으로 개선하라.」
>
> ★**측정 트리 명시**: 아래 수치는 전부 `origin/main`(`250d7e4c`) ↔ `upstream/main`(`73938944`) 을
> 2026-08-27 에 `git fetch` 한 뒤 잰 값이다. 워킹트리(`@live`) 기준 값은 하나도 쓰지 않았다.

## 0. 한 줄 결론

**⒟ — fork 를 «배포·제품 오버레이»로 남기고 엔진은 upstream 을 쓴다.** ⒜(upstream 위 재정렬)의
변형이되, 「①을 버린다」가 아니라 ★**「우리가 애초에 엔진을 들고 있을 이유가 남아 있지 않다」**가
판정의 형태다. 근거는 §2~§4 의 실측이고, 대가는 §5 에 **수로** 적었다.

★**그리고 운영자의 의구심은 정확하되 축이 하나 더 있다** — 문제는 `Jun025/wie` 가 아니라
★**`Jun025/RustJava`** 다. 그쪽이 진짜 «불필요한 private 의존»이고(§4), 나머지와 **독립적으로**
지금 당장 버릴 수 있다.

## 1. 재측정값 (티켓 실측표의 재확인)

| 축 | 티켓 기재 | 이 회차 재측정 | 판정 |
|---|---|---|---|
| fork 공통조상 | `fa641a8a` 2026-06-10 | `fa641a8a` 2026-06-10 | 일치 |
| `origin/main` behind | 1,067 | **1,067** | 일치 |
| `origin/main` ahead | 192 | **192** | 일치 |
| 고유 코드 변경(문서·CI·md 제외) | 148파일 +17,890/−127 | **148파일 +17,890/−127** | 일치 |
| `Jun025/wie` 공개 여부 | `isPrivate=false` | **`isPrivate=false`** · `isFork=true` · parent `dlunch/wie` | 일치 |
| upstream 활동 | 2026-08-27 push | upstream head `73938944` **2026-08-27** | 일치 |

**추가 측정 — 전체 축(문서·CI 포함)**: 220파일 **+25,690 / −164**.
**upstream 1,067커밋의 구성**: ★**dependabot 721건(67.6%)** · 사람 커밋 ~346건
(Inseok Lee 324 · 그 외 22). ⇒ ★**「1,067」은 착수 규모를 과대표시한다** — §5-⒝ 에서 다시 쓴다.

## 2. 과제 ⑴ — 우리 192커밋의 3분류 (파일·줄 수)

분류 규칙을 먼저 적는다(인상 배제): `origin/main` 이 `fa641a8a` 대비 바꾼 **모든 파일**을
⑴upstream 에 지금 존재하는가 ⑵공통조상에 있었는가 ⑶upstream 도 그 뒤 손댔는가 로 교차한 뒤,
경로로 갈래를 붙였다.

### 2-1. 전체 (220파일)

| 갈래 | 파일 | +줄 | −줄 | 줄 비중 |
|---|---:|---:|---:|---:|
| ① 중복 — `wie_lgt` | 12 | 2,426 | 41 | 9.4% |
| ① 중복 — `wie_web` | 7 | 1,065 | 0 | 4.1% |
| ② 고유·가치 (엔진) | 47 | 1,908 | 61 | 7.4% |
| ③ 로컬 스캐폴딩 | 154 | 20,291 | 62 | **79.0%** |
| **합계** | **220** | **25,690** | **164** | |

### 2-2. Rust 만 (64파일 · 위 표에서 문서·CI·web/functions 를 걷어낸 것)

| 갈래 | 파일 | +줄 | −줄 | Rust 내 비중 |
|---|---:|---:|---:|---:|
| ① `wie_lgt` — upstream 이 자기 구현으로 착지 | 12 | 2,426 | 41 | 45.4% |
| ① `wie_web` — upstream 이 자기 브라우저 호스트 보유 | 6 | 1,011 | 0 | 18.9% |
| ② 엔진 고유·가치 | 45 | 1,134 | 61 | 21.2% |
| ③ fork 전용 도구 (`wie_validate`) | 1 | 772 | 0 | 14.5% |
| **합계** | **64** | **5,343** | **102** | |

★**①의 비율 = Rust 기준 3,437줄 / 5,343줄 = 64.3%** (파일 기준 18/64 = 28.1%).
★전체 축으로 보면 3,491 / 25,690 = **13.6%** 인데, ★**이 두 수를 섞어 쓰지 마라** —
전체 축이 낮은 이유는 우리가 잘해서가 아니라 **분모의 79%가 엔진이 아닌 웹 제품**이기 때문이다.

### 2-3. ③ 이 79%라는 사실이 이 판정의 실제 무게중심

`③ 로컬 스캐폴딩` 20,291줄의 내역: **웹 제품 표면 77파일 +12,337줄**
(`web/` React 앱 · `functions/` Cloudflare Pages Functions · `migrations/` D1 · `scripts/` · `wrangler.toml` ·
`package*.json`) + 문서·CI·원장 나머지. upstream 에는 **대응물이 아예 없고, 있을 이유도 없다**
(upstream 은 emulator 이고 우리는 그 위의 **서비스**다).

⇒ ★**fork 의 존재 이유는 «엔진을 따로 들고 있는 것»이 아니라 «엔진 위에 제품을 얹은 것»이었다.**
그 제품(79%)은 upstream 이 뭘 하든 **그대로 남는다.** 버려지는 후보는 엔진 쪽 3,437줄뿐이다.

### 2-4. 교차검증 — 파일 존재/충돌 행렬

| 상태 | 파일 | 뜻 |
|---|---:|---|
| upstream 에 없음 · 공통조상에도 없음 | 163 | 우리만 만든 것(웹 제품·문서·`wie_lgt` 신규·`wie_web` 신규) |
| upstream 에 있음 · upstream **도** 수정 | **43** | ★**충돌면.** 재정렬 시 실제로 손으로 풀어야 하는 자리 |
| upstream 에 있음 · upstream 미수정 | 10 | 깨끗이 얹힌다 |
| upstream 이 **독립적으로 같이 만듦** | 4 | `package.json`·`package-lock.json`·`wie_web/Cargo.toml`·`lcdui/image_observer.rs` |

★마지막 행이 ①의 축소판이다 — **같은 파일을 양쪽이 따로 만들었다.**

## 3. 과제 ⑵-대표사례 — 우리 LGT ↔ upstream `wie_lgt` (코드 대조)

★**DoD 2 요구대로 «문면 대조»가 아니라 두 트리를 열어서 잰 값만 적는다.**

### 3-1. 결정적 수치 — LGT Java 런타임 import 테이블(`0x64`) 구현 개수

`wie_lgt/src/runtime/java/interface.rs` 의 디스패치 arm 을 양쪽에서 세었다.

| | 구현 index 수 | 목록 |
|---|---:|---|
| **ours** (`origin/main`) | **8** | `0x03 0x06 0x07 0x0f 0x14 0x54 0x82 0x83` |
| **upstream** (`73938944`) | **31** | 위 8개 **전부 포함** + 23개 |
| upstream 에만 | **23** | `0x09 0x0a 0x0b 0x0c 0x0d 0x0e 0x10 0x11 0x12 0x13 0x1f 0x20 0x21 0x22 0x23 0x25 0x55 0x56 0x57 0x61 0xe1 0xe2 0xfa` |
| **ours 에만** | ★**0** | — |

★★**우리 구현은 upstream 의 «진부분집합»이다. 우리만 하는 것이 하나도 없다.**

그 23개가 무엇인지가 중요하다(upstream `docs/lgt.md` 의 표 + 코드):
`0x1f/0x20/0x21` 예외 핸들러 프레임 push/pop/throw · `0x22` NPE · `0x23` 배열 인덱스 예외 ·
`0x25` 산술 예외 · `0x56/0x57` 모니터 enter/exit · `0x0b/0x0c/0x0d` 클래스 등록·해석·초기화 ·
`0x0e/0x10/0x11` 배열 클래스·1차원·다차원 배열 생성 · `0x12` assignable 검사 ·
`0x09` UTF-16 문자열 리터럴 캐시 · `0x0a` 인터페이스 디스패치 테이블 · `0x61/0xfa` 참조배열 저장.

⇒ ★**즉 「예외·모니터·배열·클래스 초기화」라는 JVM 의 뼈대가 우리 쪽엔 통째로 없다.**
우리 8개는 전부 `InitSvcId::JavaUnk*`·`JavaInterfaceUnk*` 라는 **미상 이름**이고, 미등재 index 는
`java_interface_stub` 하나로 흘려보낸다. upstream 은 31개 전부에 **의미 이름**이 붙어 있다.

### 3-2. 크레이트 규모·구조

| | LOC | 구조 |
|---|---:|---|
| ours | 4,091 | `native_jvm.rs` **1,436**(파일 머리에 스스로 «PoC» 라 적혀 있다) + `native_class.rs` 346 |
| upstream | **7,741** | `jvm_support/` **모듈 9개**(`class_definition` 948 · `method` 457 · `vtable` 218 · `array_class_instance` 229 · `class_instance` 168 · `field` 119 · `array_class_definition` 62 · `jvm_implementation` 60 · `value` 38) + `jvm_support.rs` 1,167 + `exception.rs` 142 + `abi.rs` 58 + `classes/net/wie/lgt_class_loader.rs` 148 |

★구조 차이가 기능 차이다: upstream 은 `wie_jvm_support::JvmImplementation` 을 구현해 **`wie_ktf` 와 같은
경로**로 붙는다. 우리 것은 `BTreeMap<u32, Box<dyn ClassInstance>>` 곁테이블 + SVC 트램폴린으로 **옆에 붙였다.**

★`abi.rs` 는 ABI 표를 코드 밖 `data/lgt_java_abi.toml`(115줄 · 클래스 14종)로 빼 **데이터로** 읽는다.
우리 쪽 대응물은 소스에 박혀 있다.

### 3-3. LGT 그래픽 — 우리에겐 파일 자체가 없다

upstream `wie_lgt/src/runtime/wipi_c/graphics.rs` = **1,095줄 · LGT 전용 그래픽 구현 + 단위테스트 5개**
(annunciator 높이에 따른 application view 기하 · 오프스크린 프레임버퍼 · backing/view 모델 ·
`set_display_property` · 컨텍스트 정규화).

우리 `wie_lgt/src/runtime/wipi_c.rs` 는 **전부 공용 `wie_wipi_c::api::graphics` 로 흘린다.**
upstream 은 `graphics::`(LGT 전용)와 `shared_graphics::`(공용)를 **갈라서** 건다.

측정된 기능 차 1건: ★**`DrawArc`/`FillArc` — upstream 은 건다, 우리는 `grep` 결과 0건이다.**

### 3-4. SVC 표

| | `svc_ids.rs` | init SVC |
|---|---:|---|
| ours | 327줄 | 16개 — `Unk0`·`JavaUnk7`·`JavaUnk1`… **대부분 미상** |
| upstream | 373줄 | 4개 — `ImportTable`·`ImportFunction`·**`SetDisplayProperty`**·**`ApplicationJarPath`** (전부 규명됨) |

★수가 적은 쪽이 진 것이 아니다 — upstream 은 java 트램폴린을 init SVC 가 아니라 `abi.rs`+`jvm_support` 로
옮겼기 때문에 init 이 4개로 **줄어든** 것이고, 그 4개는 **전부 이름이 밝혀져 있다.**

### 3-5. ★같은 픽스처로 «실제로 돌려» 봤다 (문면 대조 아님)

양쪽 다 `test_data/helloworld_lgt.zip` 과 `wie_lgt/tests/test_helloworld.rs` 를 갖고 있어 직접 실행했다.

| | 명령 | 결과 |
|---|---|---|
| ours (`250d7e4c`, 이 워킹트리) | `RUST_MIN_STACK=4194304 cargo test -p wie_lgt --test test_helloworld` | ★**ok** (1 passed · 0.82s · 빌드 9m02s) |
| upstream (`73938944`, `git worktree` 격리 체크아웃) | 동일 | ★**ok** (1 passed · 0.95s · 빌드 25m21s) |

⇒ ★★**둘 다 통과한다.** 이것이 ⒟ 채택에 필요한 «최소 안전 신호»다 — upstream 판본이 LGT 기본 경로를
**깨뜨리지 않았다**는 것을 «문면이 아니라 실행으로» 확인했다.
★**그러나 이 신호의 크기를 부풀리지 마라**: helloworld 는 **stdout 한 줄**을 볼 뿐 **아무것도 그리지 않는다.**
회귀 기준선 292건(그중 LGT **52건**)에 대한 답은 **아니다** — §7-1 · §6-P2 가 그 자리다.

★**차이 1건 발견**: upstream 판 테스트는 아카이브의 `00000000.jar` 를 **`application.jar` 로 개명**한 뒤
`LgtEmulator::from_archive` 에 넘긴다. 우리 판은 그대로 넘긴다.
⇒ **엔트리포인트 규약이 갈렸다.** 재정렬 시 컨테이너 명명 규약을 맞춰야 한다(작은 일이지만 **조용히 깨지는** 자리다).

## 4. ★진짜 «불필요한 private 의존» — `Jun025/RustJava` `[patch]` 표

이것이 이 회차의 **가장 실행하기 쉬운** 발견이고, 재정렬과 **독립적**이다.

`Cargo.toml` 의 `[patch."https://github.com/dlunch/RustJava.git"]` 는 5개 크레이트를
`Jun025/RustJava` rev `c66f08d4`(**2026-07-07**)로 돌린다. `AGENTS.md` Constraint 8 은 그 근거를
「KTF panic→exception hardening — it is not stale duplication」이라고 적어 두었다.

★**그 문장을 실측으로 검증했다.** 두 체크아웃(`~/.cargo/git/checkouts/`)을 직접 비교했다:
ours `c66f08d`(2026-07-07) ↔ upstream `ba5797b`(**2026-08-16**).

| 축 | ours (Jun025 fork) | upstream (dlunch) | 판정 |
|---|---:|---:|---|
| `.rs` 파일 수 | 216 | **439** | |
| ★**우리에만 있는 `.rs` 파일** | ★**0** | — | ★**우리 fork 는 파일을 하나도 더하지 않는다** |
| `java/lang/String` NPE 가드 | 6 | **10** | upstream 이 더 많다 |
| `java/lang/StringBuffer` NPE 가드 | 1 | **7** | upstream 이 더 많다 |
| `java/io/ByteArrayInputStream` NPE 가드 | 1 | **3** | upstream 이 더 많다 |
| `NumberFormatException` 참조 | 6 | **15** | upstream 이 더 많다 |
| `byte_value` / `short_value` / `long_value` | 2 / 2 / 2 | **6 / 6 / 8** | upstream 이 더 많다 |
| `readUnsignedByte` | 구현 있음 | 구현 있음 **+ `java/io/DataInput` 인터페이스** | upstream 이 더 많다 |
| `current_thread` | 4 | 4 | 동수 |

⇒ ★★**Constraint 8 의 「not stale duplication」은 «측정으로 반증됐다».**
우리 fork 가 존재 이유로 든 **모든** 축에서 upstream 이 **같거나 더 많다**. 우리 fork 는
**upstream RustJava 의 진부분집합**이며, 2026-07-07 에 멈춰 있다.

★**이것이 운영자가 물은 그 「불필요한 private 의존」이다.** `Jun025/wie` 는 이미 public 이고
(`isPrivate=false`), 실제로 우리를 upstream 에서 떼어놓고 있던 사슬은 **`[patch]` 표 한 덩어리**다.

## 5. 갈래 판정 — 무엇을 «버리는지»를 먼저 센다

★티켓 요구: 「어느 쪽이든 «버려지는 것»을 명시적으로 세어 적어라」.

### ⒜ upstream 위로 재정렬 (③만 얹는 얇은 오버레이)

**⒜ 를 문자 그대로 집행하면 버려지는 것(측정치)**
- `wie_lgt` 우리 구현 **12파일 2,426줄** — §3 대로 upstream 의 진부분집합. ★**기능 손실 0**,
  단 `compile_model.rs` **122줄**(clet ↔ aot-java 정적 판별)은 upstream 에 **대응물이 없고**
  웹 계약이 **실제로 소비한다**(실측: `wie_web/src/lib.rs` 가 `wie_lgt::detect_compile_model` 을
  import 해 `lgt_compile_model` 로 노출 · `wie_lgt/src/lib.rs` 가 `pub use` 로 내보낸다)
  ⇒ **이식 대상**이지 폐기 대상이 아니다.
- `wie_web` 우리 구현 **6파일 1,011줄** — ★★**그리고 여기가 ⒜ 를 «그대로» 채택하면 안 되는 지점이다.**
  ⒜ 의 「③만 얹는다」를 문자대로 읽으면 `wie_web` 은 ③이 아니라 엔진 쪽이라 버려지는데,
  ★**이건 «중복»이 아니다.** upstream `wie_web` 은
  webpack+TS 단독 앱(`src/rust/` + `src/ts/`, `crate-type=["cdylib"]`)이고, 우리 것은
  **featurephone 소비 계약**(`WieEmulator` 클래스 · `constructorArity=7` · `tick`/`key_down`/
  `platform_kind`/`lgt_compile_model`/`export_saves`/`import_saves`/`has_saves`/`has_exited`)을
  wasm-bindgen 으로 노출한다. ⇒ ★**드롭인 대체가 «불가»하다. 이름만 충돌한다.**
- **`AGENTS.md` Constraint 7**(「`wie_web` 은 wasm32 밖에서 빈 라이브러리」)은 upstream 판본과 **형상이 다르다**:
  ours `[lib] crate-type=["cdylib","rlib"]` + 실코드 전량 `cfg(target_arch="wasm32")` 게이트(그래서
  네이티브 `cargo test --all` 이 green) ↔ upstream `crate-type=["cdylib"]` · `path="src/rust/lib.rs"` ·
  게이트 없음. ⇒ ★**같은 이름의 서로 다른 크레이트다.** Constraint 7 은 폐기 대상이 아니라 **개명한
  크레이트로 따라가야 할 조항**이다.

**대가(정직하게)**
- ★**회귀 기준선 292건이 전부 «미검증»이 된다.** `scripts/smoke_gate_baseline.tsv` =
  **ktf 190 · lgt 52 · skt 50 = 292 PASS**. 이 값은 **우리 엔진**(② 1,134줄 하드닝 포함)에서 잰 것이다.
- ★★**그 검증을 이 회차에서 할 수 없다** — `game_lab/` 코퍼스가 **이 머신에 없다**(실측: 부재).
  게임 바이트는 Constraint 9 로 repo 에 들어올 수 없으므로 **구조적으로** 여기서 못 잰다.
  ⇒ **이것이 ⒜/⒟ 의 유일한 실질 게이트다.** §6-P2 참조.

**얻는 것**
- upstream LGT Java import **8 → 31** · LGT 전용 그래픽 **0 → 1,095줄** · arc 그리기 획득 ·
  `wie_backend/canvas.rs` **709 → 1,647줄**(XOR 모드 · `copy_area` · round-rect · 클리핑 시맨틱 수정) ·
  `wipi_java` 클래스 표면 대폭 확대(아래) · `wie_app`(Tauri 데스크톱) · `wie_ktf_dump` 신규.

  | 파일 | 공통조상 | ours | upstream | 메서드 ours→up |
  |---|---:|---:|---:|---|
  | `lcdui/card.rs` | 175 | 222 | **1,159** | 17 → **32** |
  | `msp/db/data_base.rs` | 238 | 289 | **615** | 14 → **22** |
  | `lcdui/display.rs` | 223 | 265 | **527** | 15 → **29** |
  | `msp/io/file.rs` | 221 | 259 | **543** | 13 → **16** |

  ★**공통조상 열을 같이 봐라** — 우리도 늘렸지만(175→222), upstream 은 **1,159** 로 갔다.

### ⒜-부록 — P1(`[patch]` 제거)의 KTF 회귀 위험을 수로 재어 둔다

`[patch]` 의 명목이 «KTF panic→exception hardening» 이므로 「빼면 KTF 가 깨지지 않나」가 정당한 반문이다.
★**그 위험면을 재어 보면 작다**:

| 축 | 값 |
|---|---|
| 우리 `wie_ktf` 고유 변경 **전량** | **5파일 +30/−13** (`init.rs`·`java/interface.rs`·`java/jvm_support.rs`·`ktf_class_loader.rs`·`wipi_c/method_table.rs`) |
| `wie_ktf` 크레이트 규모 | ours **4,484** LOC ↔ upstream **4,934** LOC (파일 수는 30 동수) |
| RustJava 쪽 하드닝 | §4 대로 **upstream 이 전 축에서 같거나 많다** |

⇒ ★**KTF 축에서도 우리가 앞선 자리가 없다.** P1 의 1차 판정은 4게이트 + `ktf`/`lgt` helloworld 로 서고,
코퍼스 회귀는 P2 에 합류시키면 된다. 되돌리기는 **`[patch]` 13줄 복구**로 끝난다.

### ⒝ 단계 머지 (RustJava `docs/upstream-sync-approach.md` 선례 이식)

**비용 추정 — ★RustJava 비율을 그대로 쓰면 안 되는 이유부터 적는다.**
RustJava 선례는 **33커밋 / 4회차** = 8.25커밋/회차. 그 비율을 1,067에 곱하면 **129회차**가 나온다.
★**그 수는 틀렸다.** 두 가지가 다르다:
1. **1,067 중 721(67.6%)이 dependabot** 이다. 의존성 범프는 **커밋 단위로 풀 대상이 아니라
   `Cargo.lock` 한 번의 재해결로 접힌다.** 실질은 사람 커밋 **~346건**.
2. ★**비용을 정하는 것은 커밋 수가 아니라 «충돌면»이다.** 우리도 upstream 도 손댄 파일 = **43개**이고,
   그 43개를 건드린 upstream 커밋은 **599건**이다. ⇒ 단계 머지는 **43개 파일을 599커밋만큼의
   변화폭에 대해 손으로 화해시키는 일**이다.

**그런데 그 43개 중 엔진 파일의 대부분은 §3·§5-⒜ 표대로 «upstream 이 더 많은» 자리다.**
⇒ ★**단계 머지는 «버릴 것을 한 커밋씩 정성껏 보존하는» 작업이 된다.** 비용은 ⒜보다 크고 산출물은 더 나쁘다.

**버리는 것**: 없음(그것이 문제다 — ①의 3,437줄을 **살려서 화해**시키느라 비용을 쓴다).

### ⒞ 현상유지

채택하려면 「upstream 최신 기능을 안 쓰는 것이 낫다」를 세워야 한다. ★**세워지지 않는다**:
LGT import 8/31 · LGT 그래픽 0/1,095 · canvas 709/1,647 · RustJava fork 가 upstream 의 진부분집합.
★**우리 쪽이 앞선 축을 «하나도» 찾지 못했다**(`wie_midp/.../font.rs` 메서드 10 vs 9 가 유일한 예외이고,
이는 ② 로 upstream 에 보낼 대상이지 fork 를 유지할 근거가 아니다).

★**단 ⒞ 에 부분적 진실이 하나 있다**: **지금 당장은 우리 것이 «돈다»**(292건 기준선 · helloworld ok).
upstream 이 낫다는 것과 **우리 기준선을 upstream 이 통과한다는 것은 다른 명제**다. §6-P2 가 그 간극이다.

### ⒟ ★**채택** — fork = 배포·제품 오버레이, 엔진 = upstream

⒜의 변형이되 세 축을 **분리해서** 처분한다. 분리가 핵심인 이유: 세 축의 **위험도와 게이트가 전부 다르다**.

| 축 | 처분 | 게이트 |
|---|---|---|
| `[patch]` RustJava fork | ★**즉시 제거** | 없음 — 4게이트 + helloworld 로 지금 검증 가능 |
| `wie_lgt` ① 3,437줄 | **upstream 판본으로 교체** (단 `compile_model.rs` 122줄 이식) | ★코퍼스 회귀 실행 |
| `web/`·`functions/`·`migrations/`·`wie_web` ③ 20,291줄 | **그대로 유지** (`wie_web` 은 **개명**) | 계약 왕복검사 |

★**`wie_web` 개명이 왜 필요한가**: upstream 이 같은 이름의 크레이트를 자기 용도로 쓰고 있어
디렉터리가 정면 충돌한다. 우리 호스트를 예컨대 `wie_featurephone` 으로 옮기면 upstream `wie_web` 과
**공존**하고, 그 순간 upstream 은 **fast-forward 가능한 base** 가 된다. Constraint 7 도 그 크레이트로 따라간다.

## 6. 집행 후속 티켓 초안 (★이 회차는 집행하지 않는다)

### P1 — `[patch]` RustJava fork 제거 · size **S** · risk **low** · 선행 없음
`Cargo.toml` 의 `[patch]` 13줄 삭제 + `Cargo.lock` 재해결 + `AGENTS.md` Constraint 8 개정
(★「not stale duplication」문장을 §4 실측으로 교체 — 근거가 반증된 조항을 남겨 두면 다음 사람이 되돌린다).
**검증**: 4게이트 + `cargo test --all` + `wie_lgt`/`wie_ktf` helloworld.
★**이것만으로도 운영자 지시의 「불필요한 private 의존을 버려」는 충족된다.** 나머지와 독립이므로 **먼저 간다.**
※`wie_ktf` 회귀 위험은 실재한다(패치의 원래 명목이 KTF hardening) — ★그래서 P1 은 **코퍼스 없이도**
`ktf` helloworld + 4게이트로 1차 판정하고, 코퍼스 회귀는 P2 에 합류시킨다.

### P2 — upstream base 코퍼스 회귀 측정 · size **M** · risk **low**(측정 전용) · 선행 P1
★**이 회차가 못 한 단 하나의 측정이고, ⒟ 전체의 go/no-go 다.**
`upstream/main` 체크아웃 + 우리 `scripts/smoke_gate.sh` 로 **코퍼스가 있는 머신에서**
`ktf 190 / lgt 52 / skt 50` 재측정. 산출물은 **차이표 한 장**(신규 PASS · 신규 FAIL).
★**제품 코드 변경 0.** ★FAIL 이 나온 타이틀은 그대로 P3 의 작업목록이 된다.

### P3 — 재정렬 집행 · size **L** · risk **med** · 선행 P2
⑴`wie_web` → `wie_featurephone` 개명(계약·`build-wasm.sh`·`check-engine-contract.mjs` 동반)
⑵`upstream/main` 을 base 로 삼고 ③ 오버레이 재적용 ⑶`compile_model.rs` 122줄 이식
⑷② 1,134줄 중 upstream 에 이미 있는 것을 걷어내고 남은 것만 재적용.
★**여러 회차로 쪼개라 — 한 PR 에 넣지 마라.**

### P4 — ② 를 upstream 에 PR · size **M** · risk **low** · P3 와 병행 가능
★**IP 방침 선 안쪽만**(#1239 · 2026-06-29 「공개 문서 기반으로만 구현 · 펌웨어 리버스 계획 없음」).

**선 안쪽 (보낼 수 있다)**
- `wie_wipi_java` 우리만 있는 클래스 **10종**: `java/io/{InterruptedIOException,UnsupportedEncodingException}` ·
  `java/lang/{OutOfMemoryError,VirtualMachineError}` · `msp/lcdui/InputMethodListener` ·
  `msp/lwc/{ActionListener,FormComponent,GrabKeyListener,LabelComponent}` · `msp/media/MediaUnsupportedException`.
  ★전부 **WIPI 공개 API 스텁**이다.
- `wie_backend/src/canvas.rs` 의 **+149줄 = 전부 단위테스트 9개**(구현 아님 — 실측). ★테스트는 IP 축과 무관.
- `wie_midp/.../font.rs` 의 메서드 1건(우리 10 ↔ upstream 9).

**선 바깥 (fork 에 남긴다)**
- `docs/lgt_abi.md` 1,482줄 · `docs/lgt_native_classes.md` 228줄 · `docs/reference/` 7건
  (`WIPIHeader.h` 2,123줄 · `ezi_native_surface.txt` 1,363줄 · `AromaWIPI_*.zip` · `lgt_0x64_ordinal_table.md`).
  ★**리버스엔지니어링 산출물이라 메인테이너가 선언한 선 밖이다.** 보내지 마라.

### ★★P0 축 — 계보가 전진하는지를 DoD 에 박아라 (RustJava 선례)
동시 발권 `rustjava-upstream-sync-squash-defeats-convergence` 실측: RustJava 는 S1~S4 를 착지시키고도
`merge-base` 가 fork 시점 그대로였고 근인 후보가 게이트③ **`--squash`** 다.
★**wie 도 같은 규율 아래 있다** — 실측: 우리 192커밋에 **upstream 동기화 시도 0건**, `merge-base` 는
**2026-06-10 에서 한 번도 움직인 적이 없다.**
⇒ ★**P3 의 PR 은 `--squash` 로 머지하면 실패한다.** squash 는 upstream 계보를 평평하게 만들어
`merge-base` 를 `fa641a8a` 에 그대로 못박는다 — 다음 회차가 **또 1,067커밋 뒤**에서 시작한다.
⇒ P3 티켓 DoD 에 **리터럴로** 넣어라:
```
머지 후 `git merge-base origin/main upstream/main` 이 fa641a8a 가 아니어야 한다.
그대로면 그 회차는 실패다 — 게이트③ --squash 예외를 먼저 받아라.
```

## 7. 이 회차가 «측정하지 못한» 것 (숨기지 않는다)

1. ★**upstream base 에서 우리 292건 기준선이 통과하는가** — 코퍼스 부재로 구조적 불가. **P2 가 진다.**
2. **AOT-Java 렌더 벽(`docs/lgt.md` §7)이 upstream 에서 풀렸는가** — 우리 문서는 「boots but does not
   render」이고 도달 타이틀은 **배틀몬스터 1건**이다. upstream 은 그 축의 커밋을 17건 쌓았고
   (`#1343` GC 중 AOT static 보존 · `#1352` ABI 디스패치 · `#1368` 그래픽/런타임 호환) 예외·모니터·배열까지
   구현했다. ★**그러나 「그래서 렌더가 되는가」는 그 타이틀로 직접 돌려야 답이 나온다 — P2 에서 함께 재라.**
3. **웹 계약 왕복이 upstream base 에서 성립하는가** — `contract-roundtrip.mjs` 는 브라우저 툴체인이 필요하다. **P3.**
