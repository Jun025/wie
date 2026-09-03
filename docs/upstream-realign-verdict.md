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
★**`Jun025/RustJava`** 다. ★**단 그쪽도 «private» 이 아니다**(실측 `isPrivate=false`) — 정확한 이름은
★**«멈춰 있는 «핀» 의존»**이고(§4 · ★**정정 2026-09-04 §8-8** — 초판은 이것을 「멈춰 있는 **fork** 의존」이라
적었으나 ★**멈춘 것은 rev `c66f08d` 이지 저장소가 아니다**: `Jun025/RustJava` `origin/main` 은 2026-08-27 까지
**52커밋** 전진했다), 그것이 이 문서 제목의 문장과도 정합한다.

★★**[2026-08-27 정정 · 게이트② 반려 승계] 「그러니 지금 당장 싸게 버릴 수 있다」는 초판의 문장은 «참이 아니었다».**
`Cargo.toml` base 의존에 `rev` 가 없어 `[patch]` 를 지우면 의존이 **47커밋 앞으로 끌려가고**,
그 전진면에 우리 호출부 ★**218곳(⑴209 ⑵6 ⑶3)**을 깨뜨리는 공개 API 변경 **3종**이 실재한다(§4-B).
⇒ P1 은 `size: S`·`risk: low` 가 **아니고**, 갈래가 **둘**이며 각각 대가가 다르다(§6-P1 의 ⒜⒝).
★**이 문서는 그 둘을 나란히 적을 뿐 고르지 않는다 — 선택은 총괄 몫이다.**

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

## 4. ★진짜 사슬 — `Jun025/RustJava` `[patch]` 표는 «멈춰 있는 «핀» 의존»이다

> ★★**[정정 2026-09-04 · §8-8]** 이 절 전체가 «핀 `c66f08d`(2026-07-07)»에 대해 참이고
> ★**«저장소 `Jun025/RustJava`»에 대해서는 거짓이다** — 그 main 은 그 뒤 **52커밋** 전진했다(dlunch **+29** 흡수).
> 아래에서 「우리 fork」로 읽히는 자리는 전부 ★**「우리가 소비하는 판본 = 핀」**으로 읽어라.

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
★**«기능·파일 축»에서 upstream RustJava 의 진부분집합**이며, ★**그 «핀»은 2026-07-07 에 멈춰 있다**
(★저장소가 아니다 — §8-8).

★★**축을 반드시 명시해서 읽어라 — 초판은 이 한정 없이 «진부분집합»이라고만 적었고, 거기서 P1 견적이 어긋났다.**
★**`jvm` 크레이트 «공개 API» 축에서는 진부분집합이 «아니다»**: 우리 fork 의 12커밋 중 2건이
`jvm/src/jvm.rs`(+33/−1)·`jvm/src/thread.rs`·`jvm/src/garbage_collector.rs` 를 만졌고, `pub fn` **4종**
(`set_current_java_thread`·`current_java_thread`·`add_pending_java_thread`·`remove_pending_java_thread`)을
**추가**했다(실측: `bee850f` 에 0 · `c66f08d` 에 1). ⇒ ★**「기능은 부분집합인데 API 는 아니다」가 정확한 문장이고,
그 어긋남이 §4-B 의 파열을 낳는다.**

★★**그런데 이것도 «private 의존»이 아니다 — 초판이 여기서 같은 실수를 반복했다.**
실측(2026-08-27): ★**`Jun025/RustJava` 도 `isPrivate=false`**(`isFork=true` · parent `dlunch`).
⇒ 초판은 「private 축이 틀린 축이다」를 논지로 세워 놓고 **그 틀린 라벨을 RustJava 로 옮겨 붙였다**
(`Jun025/wie` 에는 `isPrivate` 를 실제로 조회해 교정했으면서 RustJava 에는 같은 조회를 하지 않았다 —
게이트② §6 지적). ★**정확한 이름은 「불필요한 private 의존」이 아니라 «멈춰 있는 «핀» 의존»이다**
(★2026-09-04 재정정 — 초판은 「fork 의존」이라 적었고 그것도 절반만 참이었다 · §8-8).
그리고 그것이 이 문서 제목의 문장(「private 이라서가 아니라 전진하지 않아서」)과도 **비로소 정합한다.**
⇒ 실제로 우리를 upstream 에서 떼어놓고 있던 사슬은 **`[patch]` 표 한 덩어리**다.

## 4-B. ★★P1 의 실제 비용 — `[patch]` 제거는 «13줄 삭제»가 아니다

> ★**이 절은 게이트② 반려(`reports/wie-upstream-realign-verdict.review.md` §5)로 «추가»된 것이다.**
> 초판에는 **이 축이 통째로 없었다.** 아래 수는 전부 **이 회차가 직접 다시 잰 값**이다(옮겨 적지 않았다).

### 4-B-1. 왜 파열이 생기는가 — `rev` 가 없다

`Cargo.toml` 의 **base 의존에는 `rev` 가 없다**(실측 `Cargo.toml:49-53`):

```toml
java_class_proto = { git = "https://github.com/dlunch/RustJava.git" }   # rev 없음
java_constants   = { git = "https://github.com/dlunch/RustJava.git" }
java_runtime     = { git = "https://github.com/dlunch/RustJava.git" }
jvm              = { git = "https://github.com/dlunch/RustJava.git" }
jvm_rust         = { git = "https://github.com/dlunch/RustJava.git" }
```

⇒ ★**`[patch]`(`Cargo.toml:76-81`) 를 지우면 `Cargo.lock` 의 `Jun025` source 가 무효가 되어
`dlunch/RustJava` main HEAD(`ba5797b` · 우리 fork 기준 47커밋 앞)로 «재해결»된다.**
★**«원래 자리로 돌아가는» 것이 아니라 «47커밋 앞으로 끌려가는» 것이다** — 이것이 초판이 놓친 기전이다.

### 4-B-2. 그 판본과 우리 코드 사이의 컴파일 파열 **3종** (★이 회차 재측정)

| # | 파열 | ours (`c66f08d`) | upstream (`ba5797b`) | wie 파열 지점 |
|---|---|---|---|---|
| ⑴ | `Jvm::invoke_virtual` 에 `class_name: &str` **인자 추가** | `jvm.rs:282` **4인자** | `jvm.rs:345` **5인자** | ★**209곳 / 37파일** |
| ⑵ | `current_class_loader` 가 ★**비공개로 닫혔다** | `jvm.rs:718` `pub async fn` | `jvm.rs:1147` `async fn` | ★**6곳** |
| ⑶ | `attach_thread` 가 **arity + async 둘 다** 바뀜 | `jvm.rs:665` `pub fn attach_thread(&self)` | `jvm.rs:1087` `pub async fn attach_thread(&self, Option<Box<dyn ClassInstance>>)` | ★**3곳** |

★**⑴의 209 는 «해부해서» 얻은 수다 — 옮겨 적지 않았다.** 맨 `grep -c 'invoke_virtual'` 은 **210** 을 주는데,
그 1건은 `wie_lgt/src/runtime/java/native_jvm.rs:926` 의 ★**doc 주석**(「trampoline that \`invoke_virtual\`s that
method *by name*」)이고 호출부가 아니다. `\.invoke_virtual` 로 좁히면 **209 / 37파일**이고 게이트② 수와 일치한다.
크레이트별: `wie_wipi_java` **124** · `wie_midp` **43** · `wie_skvm` **24** · `wie_ktf` **10** · `wie_lgt` **5** ·
`wie_jvm_support` **2** · `wie_j2me` **1**.

⑵의 6곳(실측): `wie_lgt/src/runtime/wipi_c/context.rs:98,110` · `wie_lgt/src/emulator.rs:114` ·
`wie_midp/src/classes/javax/microedition/lcdui/image.rs:116` · `wie_ktf/src/runtime/wipi_c/context.rs:93,106`.
⑶의 3곳(실측): `wie_lgt/src/runtime/wipi_c/context.rs:81` · `wie_midp/src/classes/net/wie/launcher.rs:64` ·
`wie_ktf/src/runtime/wipi_c/context.rs:76`.

★★**⑵가 가장 나쁘다 — «기계적»이 아니다.** upstream `jvm.rs` 의 `pub fn` 전수를 훑어도
★**현재 클래스로더를 얻는 공개 대체 API 가 없다**(공개면에 남은 유일한 class-loader 언급은
`Jvm::new(bootstrap_class_loader, …)` 생성자 인자뿐이고, 그것은 «부트스트랩»이지 «현재»가 아니다).
⇒ **대체 경로를 «설계»해야 한다.** ⑴의 209 도 순수 기계 치환이 아니다 — 각 호출의 **«선언 클래스»를 알아야**
새 인자를 채울 수 있으므로 **의미 작업**이다.

★**그리고 이 3종은 «전수»가 아니다** — 47커밋 안의 다른 파열은 **세지 않았다**(§7-4 참조).

### 4-B-3. ★그래서 `size`·`risk` 를 다시 적는다

| | 초판 | **정정** |
|---|---|---|
| `size` | `S` | ★**`L`**(갈래 ⒜ 채택 시) / **`S`**(갈래 ⒝ 채택 시 — §6-P1 참조) |
| `risk` | `low` | ★**`med`** |
| 범위 | 「`Cargo.toml` 13줄 삭제 + `Cargo.lock` 재해결 + `AGENTS.md` 개정」 | ★**그것 «아니다»** — 위 ⑴209 ⑵6 ⑶3 의 처분이 **본체**다 |

**근거 1줄**: ★**`[patch]` 삭제는 의존을 «원위치»시키는 것이 아니라 47커밋 앞으로 «전진»시키고,
그 전진면에 우리 호출부 218곳(209+6+3)을 깨뜨리는 공개 API 변경이 3종 실재하기 때문이다.**

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

### ⒜-부록 — P1(`[patch]` 제거)의 KTF **동작** 회귀 위험

> ★★★**이 표는 «자»가 하나뿐이다 — 「KTF 동작 회귀」만 재고 「API 호환」은 재지 않는다.**
> ★**이 표를 P1 비용의 «상한»으로 읽지 마라.** 아래 두 값이 아무리 작아도 §4-B 의 컴파일 파열
> ⑴209곳 ⑵6곳 ⑶3곳 중 **무엇도 움직이지 않는다** — 그 파열은 `wie_ktf` 의 diff 크기나 크레이트 LOC 로는
> **구조적으로 보이지 않는다**(`jvm` 크레이트의 **공개 API 형태**가 축이기 때문이다).
> ★**초판이 정확히 그 오독을 했다** — 이 표만 보고 P1 을 `size: S`·`risk: low` 로 적었고,
> 게이트②가 그 자리를 반려했다(`reports/wie-upstream-realign-verdict.review.md` §5).
> ⇒ **P1 의 실제 비용은 §4-B 와 §6-P1 을 봐라. 이 표는 그 비용의 «다른 축»이다.**

`[patch]` 의 명목이 «KTF panic→exception hardening» 이므로 「빼면 KTF 동작이 깨지지 않나」가 정당한 반문이다.
★**그 «동작» 위험면은 작다**:

| 축 | 값 |
|---|---|
| 우리 `wie_ktf` 고유 변경 **전량** | **5파일 +30/−13** (`init.rs`·`java/interface.rs`·`java/jvm_support.rs`·`ktf_class_loader.rs`·`wipi_c/method_table.rs`) |
| `wie_ktf` 크레이트 규모 | ours **4,484** LOC ↔ upstream **4,934** LOC (파일 수는 30 동수) |
| RustJava 쪽 하드닝 **기능 축** | §4 대로 **upstream 이 전 축에서 같거나 많다** |

⇒ ★**KTF «동작» 축에서 우리가 앞선 자리는 없다.** ★**그러나 이것은 P1 이 싸다는 뜻이 «아니다»** —
비용은 동작이 아니라 **API 형태**에서 나온다(§4-B).

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

### P1 — `Jun025/RustJava` fork 이탈 · ★**갈래가 «둘»이고 크기가 갈린다** · risk **med**

> ★★**초판은 여기를 `size: S` · `risk: low` · 선행 없음 · 「`Cargo.toml` 13줄 삭제 + `Cargo.lock` 재해결」로
> 적고 「이것만으로 충족된다 · 먼저 간다」로 headline 했다. ★그 문장들은 «참이 아니었다» — §4-B 참조.**
> 게이트②가 그 자리를 반려했고(`…review.md` §5), 이 절은 그 정정이다.

★★**아래 ⒜⒝ 는 «나란히» 적는다. 이 회차는 «고르지 않는다» — 선택은 총괄 몫이다.**

#### ⒜ 파열을 P1 범위에 «포함»한다 — `size: L` · risk **med**

`[patch]` 제거 → `dlunch/RustJava@ba5797b` 로 재해결 → ★**§4-B 의 ⑴209 ⑵6 ⑶3 = 218곳을 처분**한다.
★★**[정정 2026-09-04 · §8-4⑶] 이 수는 «≥222» 다** — 초판이 놓친 파열 2종이 더 있다(`ClassDefinition::{interface_names,prepare}` **2곳** · `822504b`(+21)의 `Runtime::exit` **1곳** · `from_classfile` 오류형 **1곳**). ★그리고 오늘 `dlunch/RustJava` HEAD 는 `ba5797b` 가 아니라 `bd42427`(**+59**)다 — `rev` 를 안 박으면 목적지가 «움직인다».

- **대가**: ⑴209곳은 각 호출의 «선언 클래스»를 알아야 하는 **의미 작업**이고, ⑵6곳은
  ★**공개 대체 API 가 없어 «설계»가 필요**하다. ★**47커밋 안의 다른 파열은 아직 세지도 않았다**
  ⇒ **218 은 하한이지 상한이 아니다.**
- **얻는 것**: ★**upstream RustJava 최신(2026-08-16)을 그대로 탄다** — 12커밋 하드닝을 **잃지 않는다**
  (게이트②가 12건 전건이 upstream 에 실재함을 커밋 단위로 재현했다). fork 의존 **완전 소멸**.
- ★**여러 회차로 쪼개라** — 한 PR 에 218곳을 넣으면 검수가 불가능하다.

#### ⒝ base 의존을 `rev` 로 못박아 «fork 이탈»만 먼저 한다 — `size: S` · risk **med**

`[patch]` 를 지우는 대신 `Cargo.toml:49-53` 의 base 의존 5줄에 `rev = "bee850f…"`(= fork 시점)를
**박는다**. `Jun025/RustJava` 참조는 사라지고 의존은 `dlunch/RustJava` 한 곳이 된다.

- ★**API 작업 «0곳»이다**(이 회차 실측): `bee850f` 에서 ⑴`invoke_virtual` 은 **4인자**(`jvm.rs:278`) ·
  ⑵`current_class_loader` 는 **`pub`**(`jvm.rs:686`) · ⑶`attach_thread` 는 **`pub fn …(&self)`**(`jvm.rs:660`)
  ⇒ ★**세 파열이 «전부 부재»한다.** `bee850f`↔`c66f08d` 의 `jvm` 공개 API 차이는 우리가 **추가**한
  `pub fn` 4종뿐이고, ★**wie 는 그 4종을 «0곳» 호출한다**(실측) ⇒ 제거해도 호출부가 깨지지 않는다.
- ★★**대가 — «12커밋 하드닝을 잃는다». 그리고 그 손실이 «조용하다».**
  잃는 것(실측 목록): `panic→exception hardening for KTF` · `System.arraycopy`/`Class.forName` null 가드 ·
  `Thread.currentThread()` 참조 동일성 + pending-thread GC 루트 · `StringBuffer.append(char[],int,int)` NPE ·
  `StringBuffer.insert` · `Vector.copyInto`/`capacity` · `DataInputStream.readUnsignedByte` ·
  `Timer.schedule(TimerTask,long)` · `java.lang.Byte` · `File.length()` 미존재 파일 0 · `TimeZone.getAvailableIDs`.
  ★★**이것들은 전부 «Java 레벨» 동작이라 «4게이트가 green 인 채로» 사라진다** — 컴파일도 되고
  `cargo test --all` 도 통과한다. ★**드러나는 곳은 코퍼스(P2)뿐이고, 증상은 게임이 특정 지점에서 죽는 것이다.**
  ⇒ ★**⒝ 를 고른다면 P2 를 «선택»이 아니라 «필수 후속»으로 묶어야 한다.**
- ★**미측정 1건(정직하게)**: 12축 전건이 upstream 에 실재하므로(게이트② 재현) ★**하드닝은 있고 API 파열은
  없는 «중간 rev»가 `dlunch/RustJava` 안에 존재할 수 있다.** 그 rev 를 찾으면 ⒝ 의 대가가 **0 에 가까워진다.**
  ★**이 회차는 그것을 재지 않았다**(47커밋을 rev 단위로 이분해야 한다 — 그 자체가 별도 측정이다).
  ⇒ **총괄이 ⒝ 를 검토한다면 이 측정을 먼저 발권할 값이 있다.**

#### 두 갈래 공통

`AGENTS.md` Constraint 8 개정(★「not stale duplication」문장을 §4 실측으로 교체 — 근거가 반증된 조항을
남겨 두면 다음 사람이 되돌린다) · **검증** = 4게이트 + `cargo test --all` + `wie_lgt`/`wie_ktf` helloworld.
★**단 ⒝ 에서는 4게이트 green 이 «안전»을 뜻하지 않는다**(위 대가 참조).

★★**초판의 두 문장을 이렇게 대체한다**:
- ~~「이것만으로도 운영자 지시의 «불필요한 private 의존을 버려»는 충족된다」~~
  → ★**「⒜⒝ 어느 쪽도 «13줄»로는 충족되지 않는다. ⒜는 218곳의 처분을, ⒝는 12커밋 하드닝 상실의
  수용을 각각 «대가»로 요구한다. 그리고 애초에 그것은 «private 의존»이 아니라 «멈춰 있는 «핀» 의존»이다»(§4 · §8-8).**
- ~~「나머지와 독립이므로 먼저 간다」~~
  → ★**「fork 이탈이라는 «목적»은 여전히 나머지와 독립이다. 그러나 «순서»는 자명하지 않다」** —
  ⒜를 고르면 P1 이 **가장 비싼 회차**가 되어 ★**「가장 싼 판정 측정(P2)」이 「가장 비싼 선행」 뒤에 서게 된다.**
  ⇒ ★**P2 의 `선행: P1` 배치를 총괄이 재검토하라**(게이트② §6 도 같은 지적). ★**이 회차는 결정하지 않는다.**

### P2 — upstream base 코퍼스 회귀 측정 · size **M** · risk **low**(측정 전용) · 선행 P1
★**이 회차가 못 한 단 하나의 측정이고, ⒟ 전체의 go/no-go 다.**
`upstream/main` 체크아웃 + 우리 `scripts/smoke_gate.sh` 로 **코퍼스가 있는 머신에서**
`ktf 190 / lgt 52 / skt 50` 재측정. 산출물은 **차이표 한 장**(신규 PASS · 신규 FAIL).
★**제품 코드 변경 0.** ★FAIL 이 나온 타이틀은 그대로 P3 의 작업목록이 된다.

### P3 — 재정렬 집행 · size **L** · risk **med** · 선행 P2
⑴`wie_web` → `wie_featurephone` 개명(계약·`build-wasm.sh`·`check-engine-contract.mjs` 동반)
⑵`upstream/main` 을 base 로 삼고 ③ 오버레이 재적용 ⑶`compile_model.rs` 122줄 이식
⑷② 1,134줄 중 upstream 에 이미 있는 것을 걷어내고 남은 것만 재적용
⑸★**엔트리포인트 규약 정합**(§3-5) — upstream `LgtEmulator` 는 아카이브에서 **`application.jar`** 를 찾고
   우리는 `00000000.jar` 를 그대로 넘긴다. ★**«부수 발견»으로 흘리지 말고 작업목록의 «리터럴 항목»으로 둔다** —
   본문 표현대로 「조용히 깨질 자리」이고, 조용한 것은 목록에 없으면 잊힌다(게이트② §6 지적).
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
⇒ ★**이 repo 의 PR 은 `--squash` 로 머지하면 실패한다**(★2026-09-04 정정 — 초판은 「P3 의 PR」이라 좁게 적었으나
`~/orchestrator/contracts/upstream-sync-repos.conf` 는 **repo `wie` 전체**를 등재하고 `bin/queue-lint` 검사 22 가
그 repo 의 **모든** `*-merge` 티켓에 `merge_strategy:` 선언을 요구한다 — 「P3 한정」으로 읽히면 **P1 착지 회차**에서 규율이 빠진다).** squash 는 upstream 계보를 평평하게 만들어
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
4. ★★**[게이트②가 추가시킨 4항] `[patch]` 제거 시의 «API 호환» 축** — §4-B.
   ★★**그리고 이 항목은 «앞의 셋과 계급이 다르다». 흐리지 마라.**
   ⑴⑵⑶은 **구조적 부재**다 — 코퍼스가 이 머신에 없고(Constraint 9 로 반입 불가) 브라우저 툴체인이 없다.
   ★**4항은 «구조적 부재가 아니다» — 이 머신에서, 이 회차에, `grep -c` 두 번이면 잴 수 있었다.**
   실제로 이번 정정 회차가 **그 두 번의 `grep`** 으로 209·6·3 을 전부 얻었다(외부 자원 0 · 네트워크 0 · 수 분).
   ⇒ ★**초판이 이 축을 빠뜨린 것은 «못 잰 것»이 아니라 «잴 생각을 못 한 것»이다.**
   ★**근인**: 「위험면」을 `wie_ktf` **diff 크기**와 **크레이트 LOC** 라는 **동작 축의 자**로만 쟀고
   (§5-⒜부록), ★**그 자로는 API 형태 변화가 «구조적으로 보이지 않는다»** — 그런데 「작다」로 단정했다.
   ⇒ ★★**다음 회차를 위한 규율**: 「의존을 바꾼다」는 제안의 위험면은 ★**«그 의존의 공개 API 형태»**로 재라.
   diff 크기·LOC·기능 커버리지는 **전부 다른 축**이고, 셋 다 작아도 API 는 깨진다.
   ★**「측정하지 못한 것」에 올리기 전에 «정말 못 재는가»를 한 번 물어라** — 이 항목은 그 질문을 안 해서
   목록에조차 오르지 못했다(게이트②의 자기 지적을 그대로 옮긴다).

## 8. P2 — go/no-go 측정 회차 (2026-09-03 · `wie-upstream-realign-p2-gate-measurement-before-p1`)

> ★**총괄 결정으로 P2 를 P1 «보다 먼저» 돌렸다**(「가장 싼 판정 측정이 가장 비싼 선행 뒤에 서는 것은
> 순서가 뒤집힌 것이다」). 이 절은 그 회차의 산출이고, §6-P2 · §7-1 · §6-P1 의 「미측정 1건」에 답한다.
> ★**제품 코드 변경 0 · upstream 발신 0 · `Cargo.toml` 무접촉**(아래 API 프로브는 전부
> `git worktree` 격리 체크아웃에서 돌고 제거됐다 — 이 저장소 diff 에 남지 않는다).
>
> ★★**[2026-09-04 반려 승계 `-fix`] 이 절은 «두 번 쓰였다». §8-4⑶·⑷ 와 §8-6 은 개정판이고
> §8-8 이 그 근인이다** — 초판은 `dlunch/RustJava` `bee850f`+N 위에만 계단을 세우고
> ★**우리 fork 자신의 `origin/main` 을 한 번도 재지 않았다.** 그 결과 ⑴계단에 칸 하나가 빠지고
> ⑵`822504b`(+21)의 파열 2종을 놓쳐 +33·+46·+47 칸의 수가 낮았으며 ⑶EUC-KR 문장의 주체가 틀렸다.
> ★**권고도 `fe5d116`(+16) → `5b84dd1`(+33) 로 바뀌었다**(§8-6).

**측정 시각 2026-09-03 23:00~23:15 KST · `origin/main` `ec1b7027` · `upstream/main` `6cafdb0e`**
(`git rev-list --count`: behind **1,089** / ahead **194** · `merge-base` 여전히 `fa641a8a` — §1 의 1,067/192 는
2026-08-27 값이고, ★**이 수는 조회 시각과 함께 읽어라**).

### 8-1. ★★결론 먼저 — **P2 는 이 머신에서 측정 불가다. 그리고 사유가 «둘»이다**

| # | 코퍼스 | 측정 가능? | 사유 |
|---|---|---|---|
| — | `ktf` 190 | ★**불가** | 아래 ⒜ ⒝ **둘 다** |
| — | `lgt` 52 | ★**불가** | 〃 |
| — | `skt` 50 | ★**불가** | 〃 |
| — | `test_data/helloworld_{ktf,lgt}.zip` (커밋된 픽스처 2건) | **가능** | §8-2 |

**⒜ 코퍼스 부재 — 구조적**(§5 · §7-1 이 이미 적은 축, 이 회차 재확인).
`find ~ -maxdepth 4 -name game_lab` → **0건**, `~/work`·`~/Documents` 깊이 5 에 `ktf`/`skt` 디렉터리 **0건**.
게임 바이트는 Constraint 9 로 repo 에 들어올 수 없다 ⇒ 이 머신에서는 **영구히** 못 잰다.

**⒝ ★★러너 부재 — 이 회차가 «새로» 찾은 축이고, §6-P2 의 처방을 그대로는 집행할 수 없게 만든다.**
§6-P2 는 「`upstream/main` 체크아웃 + 우리 `scripts/smoke_gate.sh`」라고 적었다. ★**그 조합은 성립하지 않는다**:

| 축 | ours `ec1b7027` | upstream `6cafdb0e` |
|---|---|---|
| 게이트 스크립트 | `scripts/smoke_gate.sh` | ★**없다**(`scripts/` 자체가 없고 `.github/scripts/release/` 3건뿐) |
| 헤드리스 러너 | `wie_cli/src/bin/wie_validate.rs` **772줄** | ★**없다** — `wie_cli` 크레이트 자체가 없다 |
| 네이티브 호스트 | `wie_cli` | 루트 패키지 `wie`(`src/main.rs`) + `wie-app`(Tauri) |
| 그 바이너리의 인자 | `--timeout`·`--inject`·JSON `"result":"PASS\|FAIL"` | `filename`·`--debug`·`--profile-out`·`--midi-device`·`--list-midi-devices` — ★**타임아웃 없음 · JSON 판정 출력 없음 · 창을 띄운다** |
| 크레이트 디렉터리 명명 | `wie_ktf`… (밑줄) | `wie-ktf`… (하이픈) — ★**전면 개명됐다** |

`scripts/smoke_gate.sh` 는 `BIN=target/debug/wie_validate` 와 `cargo build -p wie_cli --bin wie_validate` 에
하드코딩돼 있다 ⇒ upstream 체크아웃에서는 **빌드 대상 자체가 해결되지 않는다.**
⇒ ★★**P2 를 실행하려면 «코퍼스가 있는 머신»만으로는 부족하고 `wie_validate` 772줄을 upstream 의
개명된 크레이트 위로 이식하는 선행 작업이 필요하다.** §6-P2 의 `size: M` 은 그 몫을 세지 않았다.

★**그래서 이 회차는 차이표를 «지어내지 않았다»** — 티켓이 허용한 대로 「측정 불가」를 판정으로 적는다.

**무엇이 있으면 잴 수 있나(한 줄)**: ★`game_lab/working/{ktf,lgt,skt}` 코퍼스가 있는 머신 **그리고**
그 머신에서 `wie_validate` 를 upstream 크레이트 이름 위로 이식한 판본 — **둘 다** 있어야 한다.

### 8-2. 잴 수 있는 범위에서 잰 것 — 차이표 1장 (★코퍼스가 아니다)

양쪽 저장소가 **각자 갖고 있는** helloworld 픽스처 통합테스트를 그대로 돌렸다.

| | 명령 | ktf | lgt | 벽시계 |
|---|---|---|---|---|
| ours `ec1b7027` | `RUST_MIN_STACK=4194304 cargo test -p wie_ktf -p wie_lgt --test test_helloworld` | ★**ok** (0.53s) | ★**ok** (0.27s) | 1m16s (캐시 적중) |
| upstream `6cafdb0e` | `… cargo test -p wie-ktf -p wie-lgt --test test_helloworld` | ★**ok** (3.72s) | ★**ok** (0.70s) | 7m42s (콜드) |

⇒ **신규 FAIL 0 · 신규 PASS 0**(2/2 ↔ 2/2). §3-5 가 lgt 1건에 대해 얻은 신호를 ktf 로 넓힌 것이고,
★**그 절의 경고를 여기서도 반복한다 — 이 신호의 크기를 부풀리지 마라.** helloworld 는 stdout 한 줄이고
회귀 기준선 **292건**에 대한 답이 **아니다**. 이것은 «go/no-go» 가 아니라 «최소 안전 신호»다.

### 8-3. 「12커밋 하드닝 상실」의 크기 — 열거와 판정

`Jun025/RustJava` `bee850f..c66f08d` = **정확히 12커밋**(실측). 각 커밋이 upstream 의 어느 rev 에서
대응물을 갖는지를 트리 프로브로 쟀다(`git show <rev>:<파일>` + 심볼 계수 · 옮겨 적지 않았다).

> ★★**[2026-09-04 정정 · `-fix`] 열이 «둘» 늘었다.** 초판은 `bee850f`·`fe5d116`·`ba5797b` 만 쟀고
> ★**권고가 실제로 겨눌 두 칸(`5b84dd1`(+33) · `Jun025/RustJava` `origin/main`)을 재지 않았다.**
> ★그리고 **행 5 의 `ba5797b` 칸이 «있음»으로 잘못 적혀 있었다** — 정밀 프로브로는 `pending` **0건**이다(아래 주).

| # | 우리 커밋 | 축 | `bee850f`(+0) | `fe5d116`(+16) | ★`5b84dd1`(+33) | ★**fork main**(`8c1238b`) | `ba5797b`(+47) |
|---|---|---|---|---|---|---|---|
| 1 | `1f0e52e` | panic→exception (NFE 클래스 · `Integer.parseInt`) | 없음 | ★**있음** | 있음 | ★**있음** | 있음 |
| 1' | 〃 | `String` null NPE | 없음(기저 4) | 없음 | 있음 | ★**있음** | 있음 |
| 1'' | 〃 | `ByteArrayInputStream` null NPE | 없음 | 없음 | 없음 | ★**없음** | 있음 |
| 2a | `0f4f0bb` | `System.arraycopy` null NPE | 없음 | 없음 | 없음 | ★**없음** | 있음 |
| 2b | 〃 | `Class.forName` CNFE | 없음 | ★**있음** | 있음 | ★**있음** | 있음 |
| 3 | `9970ab6` | `Thread.currentThread()` 참조 동일성 | 없음 | 없음 | 있음(= `f9a315e` **+17**) | ★**있음** | 있음 |
| 4 | `45ff9c2` | `StringBuffer.append(char[],int,int)` NPE | 없음 | 없음 | 없음 | ★**없음** | 있음 |
| 5 | `daee53b` | pending-thread GC 루트 | 없음 | 없음 | 없음 | ★**없음** | ★**없음**(정정 · 아래 주) |
| 6 | `cd4804f` | `DataInputStream.readUnsignedByte` | 없음 | 없음 | 있음 | ★**있음** | 있음 |
| 7 | `58c1525` | `Vector.copyInto`/`capacity` | 없음 | 없음 | 있음 | ★**있음** | 있음 |
| 8 | `9cfc346` | `StringBuffer.insert(I,String)` | 없음 | 없음 | 없음 | ★**없음** | 있음 |
| 9 | `3cb4d7d` | `Timer.schedule(TimerTask,long)` 1회성 | 없음 | 없음 | 없음 | ★**없음** | 있음 |
| 10 | `d61cf07` | `java.lang.Byte` | 없음 | 없음 | 있음 | ★**있음** | 있음 |
| 11 | `9be0ea3` | `File.length()` 미존재 → 0 | 없음 | ★**있음** | 있음 | ★**있음** | 있음 |
| 12 | `c66f08d` | `TimeZone.getAvailableIDs` | 없음 | 없음 | 있음 | ★**있음** | 있음 |

⇒ 상실: `bee850f` **12/12** · `fe5d116` ≈**10.5/12** · ★`5b84dd1` ≈**4/12** ·
★**fork main ≈4/12**(= +33 과 같다 — dlunch 조상이 **+29** 라 그렇다) · `ba5797b` ≈**0~1/12**.
★**칸 F 와 +33 이 이 축에서 «구별되지 않는다»는 것이 §8-6 권고의 근거 절반이다.**

★**주 — 행 5 의 한계(숨기지 않는다)**: `pending` 문자열 프로브로는 `5b84dd1`·`95ebc5c`·`ba5797b`·fork main
**전부 0건**이다. ★**그러나 upstream 이 «다른 설계로 같은 창을 닫았을» 수 있다**(우리 `daee53b` 는
`Thread.start()` 후 spawn 태스크가 attach 하기 전 구간을 GC 루트로 잡는데, upstream 은 `f9a315e` 에서
attach 를 async 로 바꿔 그 창 자체를 없앴을 수 있다). ⇒ ★**이 한 축은 «프로브로 판정 불가»** 로 남긴다 —
그래서 위 상실 수가 「4」가 아니라 「≈4」다. ★초판이 이 칸을 «있음»으로 단정한 것이 오류였다.

★★**「그중 코퍼스 통과에 영향이 있는 것이 몇 건인가」 — 답은 「모른다」이고, «왜 모르는지»가 §8-1 이다.**
이 12축은 전부 **게임 바이트코드가 그 Java 메서드를 부를 때** 발화한다. 우리 Rust 는 그 메서드들을
직접 호출하지 않으므로 ★**이 저장소 안의 어떤 정적 자로도 순위를 매길 수 없다**(호출자는 게임이다).
⇒ 코퍼스 없이는 «종류»만 알고 «빈도»를 모른다. **종류는 전부 crash 급이다**(미등록 클래스 →
`ClassNotFoundException` · 미구현 메서드 → 미해결 · null → panic).

### 8-4. ★★§6-P1 의 「미측정 1건」에 답한다 — **「하드닝은 있고 API 파열은 없는 중간 rev」는 «존재하지 않는다»**

§6-P1 은 「그 rev 를 찾으면 ⒝ 의 대가가 0 에 가까워진다」고 적고 재지 않았다. 이 회차가 **쟀다**.

**⑴ 알려진 파열 3종의 도입 지점**(`bee850f..ba5797b` 47커밋을 rev 단위로 훑어 `jvm/src/jvm.rs` 의
시그니처가 «바뀌는 커밋»만 뽑았다):

| 파열 | 도입 rev | 위치 | wie 호출부 |
|---|---|---|---|
| ⑶ `attach_thread` arity+async | `f9a315e` (#175) | ★**+17** | 3곳 |
| ⑵ `current_class_loader` 비공개화 | `7dc1b90` | ★**+34** | 6곳(공개 대체 없음) |
| ⑴ `invoke_virtual` `class_name` 인자 | `ba5797b` (#201) | ★**+47 = 마지막 커밋** | 209곳 |

⇒ 여기까지만 보면 **+16 까지가 「API 안전 천장」**으로 보인다. ★**그래서 실제로 컴파일해 봤다.**

**⑵ ★그리고 «네 번째» 파열이 나왔다 — 컴파일이 아니면 안 보였다.**
격리 워크트리에서 base 5줄을 `rev = "fe5d116"`(+16)로 핀하고 `[patch]` 를 지운 뒤
`cargo check --workspace --all-targets` → ★**rc=101**:

```
error[E0046]: not all trait items implemented, missing: `interface_names`, `prepare`
  --> wie_ktf/src/runtime/java/jvm_support/class_definition.rs:270   (impl ClassDefinition for JavaClassDefinition)
  --> wie_lgt/src/runtime/java/native_jvm.rs:440                     (impl ClassDefinition for LgtClassDefinition)
```

트리 프로브로 도입 지점을 좁혔다: `ClassDefinition::interface_names` = ★**`ebd9c03` (+1)** ·
`ClassDefinition::prepare` = ★**`07fc404` (+2)**.
⇒ ★★**API 안전 천장은 `bee850f` «자신»이다. upstream 은 우리 fork 시점 «바로 다음 커밋»에서
`ClassDefinition` 트레이트를 넓혔고, 그 뒤로 API 파열 없는 rev 는 하나도 없다.**
★**§6-P1 의 가설은 반증됐다 — ⒝ 의 대가는 0 에 가까워지지 않는다.**

**⑶ ★그 대신 «비용 계단»이 드러났다 — 이것이 이 회차의 실질 산출이다.**

> ★★**[2026-09-04 정정 · 게이트② 반려 승계 `-fix`] 초판의 계단은 «칸 하나»를 빼먹었고 «두 칸의 수»가 낮았다.**
> 빠진 칸은 ★**`Jun025/RustJava` 자신의 `origin/main`** 이다 — 초판은 headline 산출을 전부
> `dlunch/RustJava` `bee850f`+N 위에만 세우고 ★**우리 fork 의 main 을 «한 번도 재지 않았다»**(근인 = §8-8).
> 아래 표는 그 칸을 넣고 **네 축을 같은 자로** 다시 잰 것이다. 명령은 §8-4⑸에 전문을 적었다.

| # | 핀 | fork 의존 | ⒜얻는 커밋 | ⒝누적 API 파열(호출부) | ⒞하드닝 상실 | ⒟`Cargo.toml` 변경 |
|---|---|---|---|---|---|---|
| 1 | `bee850f`(+0) | ★**소멸** | 0 | 없음 — **0** | **12/12** | base 5줄 `rev` + `[patch]` 12줄 삭제 |
| 2 | `fe5d116`(+16) | ★**소멸** | upstream **16** | `ClassDefinition::{interface_names,prepare}` — ★**2 impl(컴파일 측정)** | ≈**10.5/12** | 〃 |
| 3 | `5b84dd1`(+33) | ★**소멸** | upstream **33** | + `attach_thread`(3) + ★**`Runtime::exit`(1)** + ★**`from_classfile` 오류형(1)** ⇒ ★**≥7(컴파일 측정)** | ≈**4/12** | 〃 |
| ★**F** | ★**`Jun025/RustJava` `origin/main`**(`8c1238b`) | ★**유지** | **52**(= dlunch **29** + 우리 **23**) | ★**≥7 — 3번 칸과 «에러가 글자까지 같다»(컴파일 측정)** | ≈**4/12** | ★**핀 5줄 bump(+5/−5)** |
| 4 | `95ebc5c`(+46) | 소멸 | upstream 46 | + `current_class_loader`(6 · **공개 대체 없음 ⇒ 설계**) ⇒ **≥13** | ≈**0~1/12** | 〃 |
| 5 | `ba5797b`(+47) | 소멸 | upstream 47 | + `invoke_virtual`(**209**) ⇒ ★**≥222** | ≈**0~1/12** | 〃 |

★★**초판 대비 무엇이 바뀌었나 — 셋이다.**
⑴★**칸 F 가 «없었다».** ⑵★**3번 칸이 「+3 = 5곳」이 아니라 「≥7」이다** — `822504b`(**+21** · #180
「Harden JVM runtime correctness」)가 ★**두 종류를 더 깨뜨린다**: `java_runtime::Runtime` 트레이트가
`fn exit(&self, i32)` 를 얻었고(`wie_jvm_support/src/runtime.rs:142` impl **1곳**),
`ClassDefinitionImpl::from_classfile` 의 오류형이 `JavaError` → `ClassDefinitionError` 로 바뀌었다
(`wie_jvm_support/src/jvm_implementation.rs:46` **1곳**). ★초판은 +33·+46 을 **grep 값**으로만 적었고
그래서 **이 두 종류를 놓쳤다** — 이번에 `cargo check` 로 잡았다. ⑶따라서 4·5번 칸도 각각 **≥13 · ≥222** 로 올라간다
(§4-B 의 「218」은 초판이 **220** 으로 고쳤고, 이번에 ★**222** 가 된다 — ★그래도 여전히 **하한**이다).

★★**칸 F 의 판정 — 「가장 값싼 칸」일 것 같았고, 실측은 «아니었다».**
게이트② 검수자가 「그 칸이 가장 값싼 칸일 가능성이 높다」고 지적했고 그 지적은 **옳게 제기됐다**.
그러나 실제로 `cargo check` 를 돌리자 ★**3번 칸(`5b84dd1`)과 «똑같은 두 에러»에서 멈췄다**
(`wie_jvm_support` E0308 + E0046 `exit` · 크레이트·줄번호까지 동일).
근인: fork main 의 dlunch 조상은 `3296139` = **bee850f+29** 이고, 파열원 `822504b` 는 **+21** 이라
★**F 도 3번 칸도 그것을 «똑같이» 포함한다.**
⇒ ★★**F 는 3번 칸과 ⒝⒞ 가 같으면서 ⒟만 싸고(5줄 bump) «fork 의존을 유지한다»** ⇒
★**3번 칸이 F 를 «지배»한다**(같은 값에 fork 이탈이 덤이다). ★그래서 F 는 권고가 아니다 —
**배제 근거를 «부재»가 아니라 «수»로 적었다.**

★**계단의 «컴파일로 측정된» 칸은 이제 셋이다**: +16 · +33 · F. +46·+47 의 호출부 수(6·209)는 여전히
§4-B 의 **grep 값**이고, ★**+38 이후 칸에는 lockfile 축의 추가 대가가 있다**(§8-7-3).
⇒ ★**209 는 여전히 «마지막 한 커밋»에 몰려 있다.** P1 은 「222 아니면 0」의 동전던지기가 아니라
★**0 → 2 → ≥7 → ≥13 → ≥222 의 계단**이고, **앞 세 칸을 따로 착지시킬 수 있다.**

**⑷ ★★[2026-09-04 정정] EUC-KR panic 수정 — «주체»가 틀렸다. 「우리 fork」가 아니라 「우리 핀」이다.**
`8ac70cb`(**+5** · #162 「Fix `DataInputStream.readUTF()` panic on non-UTF-8 (EUC-KR) input」)은
`RustString::from_utf8(buf)` 의 `.unwrap()` 을 EUC-KR 폴백으로 바꾼다.
- ★**핀 `c66f08d`(브랜치 `origin/wie-ktf-hardening` · 2026-07-07)의 같은 파일 203줄은 여전히 `.unwrap()` 이다** — 여기까지는 참이다.
- ★★**그러나 `Jun025/RustJava` `origin/main` 에서는 «이미 고쳐져 있다»**: `8ac70cb` 가 그 조상이고
  (`git merge-base --is-ancestor 8ac70cb origin/main` **rc=0**), `read_utf` 은 **modified-UTF-8 디코더로 다시 쓰였다**
  (`origin/main:…/data_input_stream.rs:155~` · `from_utf8` **0건** · `.unwrap()` **0건**).
⇒ ★**초판의 「우리 fork 는 «한국어 인코딩 panic 수정»을 놓치고 있다」는 «핀에 대해 참이고 fork 에 대해 거짓»이다.**
정확한 문장: ★**「우리가 «소비하는 판본»이 그 수정을 놓치고 있다」.** 이 수정은 **+5 라 API 파열 «앞»에 있다.**

**⑸ ★이 절의 수를 낸 명령**(다음 사람이 그대로 다시 칠 수 있게 · 트리 `~/work/otterpebble/rustjava`
· ref 최종 fetch **2026-09-04 00:03** · ★**이 회차는 fetch 하지 않았다 = 읽기만 했다**):

```sh
# ⒜ 계보
git rev-list --count bee850f..origin/main                      # 52
git rev-list --left-right --count origin/main...upstream/main  # 23  30
git merge-base origin/main upstream/main                       # 3296139 = bee850f+29
git rev-list --count origin/main..c66f08d                      # 12   (핀은 main 밖)
git for-each-ref --contains c66f08d --format='%(refname:short)' # origin/wie-ktf-hardening
# ⒝ API — 시그니처 3종 + 컴파일
git show origin/main:jvm/src/jvm.rs | /usr/bin/grep -nE 'pub async fn invoke_virtual|fn current_class_loader|fn attach_thread'
git show origin/main:jvm/src/class_definition.rs | /usr/bin/grep -nE 'fn (interface_names|prepare)\b'
#   격리 워크트리에서 핀만 8c1238b 로 bump → cargo check --workspace --all-targets  → rc=101
# ⒞ 하드닝 13행 트리 프로브(§8-3 과 같은 자) — bee850f / fe5d116 / 5b84dd1 / 95ebc5c / ba5797b / origin/main / c66f08d
# ⒟ git diff --stat Cargo.toml                                  # 핀 bump = 5 insertions(+), 5 deletions(-)
```

★**⒞ 의 판정 근거(칸 F)**: 13행 중 **보유 9**(1 NFE · 1' `String` NPE · 2b CNFE · 3 thread identity ·
6 `readUnsignedByte` · 7 `copyInto` · 10 `Byte` · 11 `File.length` · 12 `getAvailableIDs`) ·
★**부재 4**(4 `StringBuffer.append` NPE 가드 · 5 pending-thread GC 루트 · 8 `StringBuffer.insert` ·
9 `Timer.schedule(TimerTask,long)` 1회성) · **부분 2**(1' `ByteArrayInputStream` NPE **부재** ·
2a `System.arraycopy` NPE **부재**) ⇒ 상실 ≈ **4/12**.
★**검수자 실측(≈3/12)과 갈린다** — 그쪽은 8·9 를 분류하지 않았다. ★**내 수도 프로브값이다**:
축 5 는 `95ebc5c`·`ba5797b` 에서도 `pending` **0건**이라 ★**upstream 이 «다른 설계로 같은 창을 닫았을»
가능성이 있다** ⇒ 이 한 축은 **프로브로 판정 불가**로 남긴다(그래서 4가 아니라 「≈4」다).

### 8-5. ★부수 실측 — upstream 은 이제 RustJava 를 «git 이 아니라 crates.io» 로 쓴다

`upstream/main` 의 `Cargo.toml`·`Cargo.lock` 실측: `jvm 0.1.1` · `jvm-bytecode 0.1.1` ·
`jvm-class-proto 0.1.1` · `jvm-types 0.1.1` · `rustjava-runtime 0.1.1` — 전부
`source = "registry+https://github.com/rust-lang/crates.io-index"`. `[patch]` 표 **없음**.
⇒ ★**⒜ 의 목적지(`dlunch/RustJava` git HEAD)는 «upstream 이 실제로 쓰는 것»이 아니다.**
정합하려면 크레이트 **개명**까지 따라가야 한다(`java_class_proto`→`jvm-class-proto` ·
`java_runtime`→`rustjava-runtime` · `java_constants`/`jvm_rust` 는 소멸/분할).
★**이 몫도 §4-B 의 220 에 세어져 있지 않다.**

### 8-6. ★P1 갈래 권고 (한 줄) — ★**「집행하지 않는다」**

> ★★**[2026-09-04 개정] 권고가 `fe5d116`(+16) → `5b84dd1`(+33) 로 «바뀌었다».**
> **왜 바뀌었나(한 줄)**: 초판 권고의 값어치를 「upstream 커밋 **16건**을 산다」로 적었는데,
> ★**그 16건은 이미 `Jun025/RustJava` 의 main 안에 있었다**(`git merge-base --is-ancestor fe5d116 origin/main`
> **rc=0**) — 즉 ★**그 칸이 «사 온다»고 말한 것의 상당수는 fork 가 이미 갖고 있고 «핀»만 못 보고 있었다.**
> 그 사실을 넣어 계단을 다시 재자 ★**+33 칸이 대가 대비 가장 많이 산다**는 것이 수로 드러났다.

> ★**권고: ⒝ 를 고르되 핀을 `5b84dd1`(+33)로 잡아라 — 컴파일로 측정된 대가는 ★≥7곳
> (`ClassDefinition` 2 impl + `attach_thread` 3 + `Runtime::exit` 1 + `from_classfile` 오류형 1)이고,
> 그 값으로 upstream 커밋 33건을 사면서 하드닝 상실을 ★**≈10.5/12 → ≈4/12** 로 내리고
> `Jun025/RustJava` 의존은 소멸시킨다.**

- **왜 +16 이 아닌가(수로)**: +16 → +33 의 추가 대가는 ★**호출부 «약 5곳»**(2 → ≥7)인데,
  그 값으로 사는 것은 ★**하드닝 6.5/12**(상실 10.5 → 4) + upstream 커밋 17건이다.
  ⇒ ★**같은 자로 재면 +33 이 압도적으로 싸다.**
- **왜 칸 F(fork main 핀 bump)가 아닌가(수로)**: F 와 +33 은 ⒝⒞ 가 **같다**(같은 두 에러에서 멈춘다 ·
  상실 ≈4/12). 다른 것은 ⒟(F 가 5줄로 더 싸다)와 ★**fork 의존**(F 는 **유지**, +33 은 **소멸**)뿐이다.
  ⇒ ★**+33 이 F 를 지배한다** — 같은 값에 「fork 이탈」이 덤이다. ★**그리고 fork 이탈은 P1 의 «목적» 자체다.**
  ※F 를 「목적에 안 맞아서」로만 접지 않았다 — **대가가 같다는 것을 먼저 쟀다**(§8-4⑶).
- ★★**대가를 같은 줄에 적는다 — 이 권고를 채택하면 잃는 것.**
  ⒜★**하드닝 상실 ≈4/12 는 «미측정»이 아니라 «측정»이다**(§8-4⑸의 13행 프로브). 잃는 축(실측):
  `StringBuffer.append([CII)` null 가드 · pending-thread GC 루트 · `StringBuffer.insert` ·
  `Timer.schedule(TimerTask,long)` · `ByteArrayInputStream` null NPE · `System.arraycopy` null NPE.
  ⒝★★**그 손실이 «참인지»를 이 집에서는 확인할 수 없다.** §6-P1⒝ 는 그 조용한 손실을 근거로
  「⒝ 를 고른다면 P2 를 «선택»이 아니라 «필수 후속»으로 묶어야 한다」고 요구했는데,
  ★**§8-1 이 그 P2 를 «이 머신에서 영구 불가»로 판정했다**(사유 둘 — 코퍼스 + 러너).
  ⇒ ★★**권고를 채택하면 이 ≈4/12 상실은 «영구 미검증»으로 남는다.** 4게이트도 `cargo test --all` 도
  green 인 채로 사라지고, 드러나는 곳은 코퍼스뿐이며 그 코퍼스가 없다.
  ⇒ ★**그래서 채택은 「싸니까」가 아니라 「그 미검증을 감수한다」는 «결정»이어야 한다.**
- **근거의 성격**: ★**API 축·하드닝 축은 «측정»이다**(`cargo check` rc=101 ×3 · 시그니처 rev 훑기 ·
  13행 트리 프로브). ★**「그 상실이 게임을 깨뜨리는가」만 «미측정»이다**(§8-1). ⇒ ★**흐리지 마라 —
  초판은 이 둘을 「코퍼스 축 = 미측정」 한 줄로 접어 «측정된 손실»을 미측정으로 강등했다.**
- ★★**총괄 결정 ②의 전제를 정정해야 한다**: 「P2 가 조용한 손실의 «크기»를 답한다」였는데,
  ★**크기는 이미 여기서 «측정»됐고**(≈4/12 · 축 이름까지) ★**P2 가 답할 것은 «아픔»뿐인데
  그 P2 가 이 머신에서 영구 불가다.** ⇒ 결정 ②를 P2 뒤로 계속 미루면 **무기한 대기**가 된다.
- **⒜ 를 권하지 않는 이유**: **≥222곳**(하한) + §8-5 의 crates.io 개명 + `current_class_loader` 의
  «공개 대체 API 없음»(설계). ★**그러나 ⒜ 를 «버리는» 것이 아니다** — 위 계단의 종점이 ⒜ 이고,
  `5b84dd1` 핀은 그 계단의 **세 번째 칸**이다.
- **⒝(`bee850f`)를 권하지 않는 이유**: 대가 0 처럼 보이지만 ★**얻는 것도 0** 이고 상실이 **12/12** 다.
  ★**핀을 «2026-06 에 멈춰 있는 rev»로 바꾸는 것**이라 §4 가 진단한 병(§8-8 의 «핀» 축)이 그대로 남는다.

### 8-7. ★이 회차가 «측정하지 못한» 것

1. ★**코퍼스 292건** — §8-1(사유 둘). **바뀐 것 없음**, 다만 사유가 하나 늘었다.
2. ★**어느 칸도 «완전한» 파열 집합을 못 냈다 — 전부 «측정된 하한»이다.** `cargo check` 는 첫 실패
   크레이트에서 멈추므로 그 뒤 크레이트에 **도달하지 못한다**: +16 은 `wie_ktf`·`wie_lgt` 에서 멈춰
   `wie_cli`·`wie_web` 미도달 · ★**+33 과 칸 F 는 «더 앞»인 `wie_jvm_support` 에서 멈춰
   `wie_ktf`·`wie_lgt` 조차 미도달**(그 둘의 `ClassDefinition` 2 impl + `attach_thread` 3곳은
   ★**시그니처로 확정한 값**이지 컴파일이 세어 준 값이 아니다 — 그래서 「≥7」이다).
   상한은 앞 실패를 채우고 다시 돌려야 나오고, 그것은 P1 의 첫 행동이지 측정이 아니다.
3. ★**+46·+47 칸의 파열 집합** — 그 두 칸의 호출부 수(6·209)는 §4-B 의 **grep 값**이고 컴파일로
   확인하지 않았다(+16·+33·F 는 이번에 컴파일로 확인했다).
   ★**+46(`95ebc5c`) 프로브는 «컴파일에 도달조차 못 했다» — 의존 해결에서 죽는다.** 3회 시도 전부:
   ⒜lock 유지 → `regex-syntax` 충돌(upstream `#193` 의 `regex ^1.13.1` ↔ 우리 `tracing-subscriber 0.3.20`
   이 잠근 `regex-syntax 0.8.10`) ⒝`Cargo.lock` 삭제 → `smaf_player` 해결 불가 ⒞`cargo update regex-syntax`
   → 이번엔 `regex-automata` 충돌. ⇒ ★★**이것이 +38 이후 칸의 «세지 않은 대가» 한 건이다** —
   그 칸을 밟으려면 `tracing-subscriber` 계열까지 함께 올려야 하고, 그 몫은 **≥222 에도 없다**.
4. **AOT-Java 렌더 벽 · 웹 계약 왕복** — §7-2·§7-3 그대로. 바뀐 것 없음.

5. ★★**[2026-09-04 추가] 칸 F 의 «우리 23커밋» 쪽 파열은 아직 못 봤다.** F 의 컴파일은
   `822504b`(dlunch +21)에서 온 두 에러에서 멈췄으므로, `Jun025/RustJava` 가 **자기 23커밋으로 바꾼
   공개 API** 가 더 있어도 이번 실행은 그것을 **보지 못한다**. ⇒ F 의 「≥7」은 3번 칸의 「≥7」보다
   ★**불확실성이 «더» 크다** — 배제 근거를 F 의 «상한»에 걸지 않은 이유가 이것이다(지배 논거는
   ⒟·fork 의존 축이지 「F 가 더 비싸다」가 아니다).
6. ★**`dlunch/RustJava` HEAD 는 «움직인다» — `+47` 은 그 자체가 낡은 좌표다.** 이 회차 실측
   (2026-09-04 · 로컬 ref, fetch 없음): `upstream/main` = `bd42427`(2026-08-31) = `bee850f`**+59**
   ⇒ ★**`[patch]` 를 지우고 base 에 `rev` 를 «안» 박으면 오늘 해결되는 곳은 +47 이 아니라 +59 다.**
   §4-B 가 「47커밋 전진」이라 쓴 것은 **2026-08-27 의 좌표**다. ★**계단의 칸은 전부 `rev` 를 «박는»
   전제 위에 있으므로 이 이동은 계단을 흔들지 않는다** — 흔드는 것은 「rev 를 안 박는」 경로뿐이다.

### 8-8. ★★「멈춰 있는 fork 의존」을 «핀» 과 «fork» 로 가른다 (2026-09-04 · 이 오류의 근인)

> ★**이 한 문장이 §8-4⑶ 의 빠진 칸과 §8-4⑷ 의 틀린 주체를 «둘 다» 낳았다.** 그래서 여기 따로 적는다.

`Jun025/RustJava` 에는 서로 다른 두 가지가 있고, §4 의 초판은 그 둘을 **한 이름으로 불렀다**:

| | 무엇 | 실측(2026-09-04) | 「멈춰 있다」가 참인가 |
|---|---|---|---|
| ★**핀** | `Cargo.toml` `[patch]` 가 가리키는 rev `c66f08d` | **2026-07-07** · 브랜치 `origin/wie-ktf-hardening` · **main 밖**(`origin/main..c66f08d` = **12**) | ★**참** — 60일째 그대로다 |
| ★**fork** | `Jun025/RustJava` 저장소의 `origin/main` | `8c1238b` **2026-08-27** · `bee850f..origin/main` = **52** · dlunch 조상 **+29** · upstream 대비 **23 ahead / 30 behind** | ★**거짓** — 전진했다 |

⇒ ★★**정확한 문장은 「멈춰 있는 fork 의존」이 아니라 ★«멈춰 있는 «핀» 의존»이다.**
fork 는 그동안 upstream 을 **29커밋** 당겨 왔고, 우리가 그것을 **못 보고 있었을 뿐**이다.

★**그리고 이 문서는 그 사실을 «알고 있었다»** — §6-P0 축이 「RustJava 는 S1~S4 를 착지시키고도
`merge-base` 가 fork 시점 그대로였다」고 쓴다. ★**즉 fork main 이 전진한 것을 «인용하면서»
§4·§8 은 fork 를 «2026-07-07 에 얼어 있는 것»으로 다뤘다.** 그 두 서술이 한 문서 안에서
서로를 반증하고 있었고, 60일 된 rev 하나가 저장소 전체의 이름이 됐다.

★★**다음 사람을 위한 규율**: ★**「fork 가 멈췄다」를 말하기 전에 «무엇이» 멈췄는지 적어라 —
`rev` 인가, 브랜치인가, 저장소인가.** 셋은 서로 다른 속도로 움직인다.
그리고 ★**핀은 «참조»이지 «저장소»가 아니다** — 핀이 낡은 것은 **우리 쪽 부작위**이지 fork 의 정체가 아니다.

★**이 정정이 닿는 자리**(전건 · 이 회차가 훑어서 고쳤다): §0 한 줄 결론의 「멈춰 있는 fork 의존」 ·
§4 제목과 본문 2곳(「2026-07-07 에 멈춰 있다」·「정확한 이름은 …」) · §6-P1 두 갈래 공통 문단의 같은 표현 ·
§8-4⑷(EUC-KR 주체) · §8-6(권고 근거) · `STATE.md` `## 다음` P1 · `REPORT.md` 2026-09-03 항목.

### 8-9. ★Contract 4 — P1 갈래(⒜ `L` ↔ ⒝ `S`)의 «구조»는 바뀌지 않는다 (수로 적는다)

칸 F 를 넣어도 §6-P1 의 두 갈래 **구조**는 그대로다. ★**왜인지를 수로 적는다**:

- ★**F 는 «세 번째 갈래»가 아니라 ⒝ 안의 «핀 선택지» 하나다.** ⒝ 의 정의는 「base 를 `rev` 로 못박아
  fork 이탈만 먼저 한다」인데, F 는 **fork 이탈을 하지 않는다**(핀이 여전히 `Jun025/RustJava` 를 가리킨다)
  ⇒ ⒝ 의 정의를 만족하지 않고, 그렇다고 ⒜(파열 218~222곳 처분)도 아니다.
  ⇒ ★**F 는 갈래가 아니라 «비교 대상»이고, 비교 결과는 §8-6 대로 «지배당함»이다.**
- ★**⒜ 의 수는 바뀌었다**: 218 → ★**≥222**(§8-4⑶). ★**갈래 판정은 안 바뀐다** — 209 가 여전히
  전체의 **94%** 이고 여전히 **마지막 한 커밋**(`ba5797b` #201)에 몰려 있다.
- ★**⒝ 의 수는 «하나»가 아니게 됐다**: 초판은 ⒝ 를 「`bee850f` 핀 = 상실 12/12」로만 적었는데,
  같은 갈래 안에서 핀을 옮기면 상실이 **12/12 → ≈10.5 → ≈4 → ≈0~1** 로 내려가고 대가가
  **0 → 2 → ≥7 → ≥13** 으로 오른다. ⇒ ★**⒝ 는 «점»이 아니라 «곡선»이다** — 그것이 이 회차가
  §6-P1 에 더한 유일한 구조 변화이고, 권고는 그 곡선 위의 **한 점**(+33)을 고른 것이다.
- ★**총괄 결정(282회차)은 그대로다** — 「P1 갈래는 P2 결과 «뒤»에 고른다」. ★**이 회차는 P1 을
  착수하지 않았다**(`Cargo.toml`·`Cargo.lock`·호출부 **무접촉** · 프로브는 전부 격리 워크트리에서
  돌고 제거됐다). ★다만 §8-6 이 적은 대로 **그 「뒤」는 오지 않는다** — P2 가 이 머신에서 영구 불가다.

## 9. P1 집행 — 총괄 채택(2026-09-04) · 핀 `5b84dd1`(+33) · `wie-upstream-realign-p1-execute-pin-plus33-and-cost-hardening-port`

> ★★**§8-6 의 권고가 «채택»됐고 이 절은 그 집행 기록이다.** 총괄 결정(2026-09-04): 갈래 **⒝** · 핀 **`5b84dd1`**.
> ⇒ ★**`Jun025/RustJava` `[patch]` 표는 «없어졌다».** §4 가 「진짜 사슬」이라 부른 것이 끊겼다.
> ★**§8 의 계단·차이표를 다시 만들지 않았다** — 이 절은 그 위에 «실제로 든 값»만 적는다.

### 9-1. ★API 파열 — 예상 **≥7** ↔ 실제 **11개소 / 7파일**

| # | 파열 | §8-4⑶ 예상 | ★실제 | 개소 |
|---|---|---|---|---|
| ⑴ | `ClassDefinitionImpl::from_classfile` 오류형 `JavaError` → `ClassDefinitionError` | 1 | **1** | `wie_jvm_support/src/jvm_implementation.rs` |
| ⑵ | `java_runtime::Runtime` 트레이트에 `fn exit(&self, i32)` 추가 | 1 | **1** | `wie_jvm_support/src/runtime.rs` |
| ⑶ | `Jvm::attach_thread` **async + `Option<Java Thread>`** | 3 | **3** | `wie_midp`·`wie_ktf`·`wie_lgt` |
| ⑷ | `ClassDefinition::{interface_names, prepare}` | 2 impl | **2 impl** | `wie_ktf/.../class_definition.rs` · `wie_lgt/.../native_jvm.rs` |
| ★⑸ | **`ClassInstance::{identity, shallow_clone}`** | ★**예상에 없었다** | **3 impl** | `wie_ktf` 객체·배열 · `wie_lgt` |
| ★⑹ | **`ArrayClassInstance: ClassInstance` 로 승격**(→ `destroy`/`class_definition`/`equals` 가 그 트레이트에서 빠짐) | ★**예상에 없었다** | **1 impl 재구조화** | `wie_ktf/.../array_class_instance.rs` |

⇒ ★★**「≥7」은 하한이었고 실제는 11 이다.** ★**그 하한 표기가 옳았다** — §8-7-2 가 「모든 칸의 파열 수는
«측정된 하한»이다 · `cargo check` 는 첫 실패 크레이트에서 멈춘다」고 미리 적었고, ⑸⑹은 정확히
`wie_jvm_support` 가 통과한 «뒤에야» 보이는 자리였다.

★**⑸ 는 «시그니처만 채우면 끝나는» 파열이 아니었다** — `shallow_clone` 은 `Object.clone()` 의 구현체다.
KTF·LGT 인스턴스는 **게스트 메모리**에 살아서 Rust 구조체를 복제하면 같은 주소를 가리킨다(=복제본에 쓰면
원본이 바뀐다) ⇒ **새 게스트 객체를 할당하고 필드 블록을 복사**하도록 구현했다. LGT 쪽은 `instantiate` 안의
동기 할당 클로저를 `alloc_guest_object` 로 빼내 두 곳이 같은 모양을 쓰게 했다.
★**`identity` 는 반대로 «공짜»였다** — KTF·LGT 객체는 게스트 주소가 곧 정체성이라 `ptr_raw`/`guest_ptr` 그대로다.

★**⑷ 의 `interface_names` 는 «값을 모른다»는 것을 시끄럽게 만들었다**: KTF 클래스 서술자에 `ptr_interfaces`
필드가 있으나 wie 가 정의하는 클래스는 전부 0 을 쓰고 그 표의 형식은 해독돼 있지 않다. ⇒ 빈 목록을 돌려주되
**0 이 아니면 `tracing::warn!`** 을 찍는다(그때만 답이 틀리기 때문이다). LGT 는 애초에 그 필드가 없다.

### 9-2. ★★하드닝 6축 — ⒤사라졌나 ⒥이식 비용 ⒥⒦ 잠글 수 있나 · 그리고 **고른 갈래**

★**⒤ 13행 프로브를 새 핀에서 재실행했다**(§8-4⑸ 와 같은 자). §8-6 이 예고한 6축이 전부 «프로브상» 0 이 됐다 — 추가 손실 0.
★★**[2026-09-04 정정] 그중 축 5 는 «프로브가 틀린 것»이지 사라진 것이 아니다**(아래 정정 항목) ⇒ ★**실제 상실은 5축이다.**

| 축 | ⒤새 핀에서 | ⒥이식 비용(원본 커밋) | ⒦코퍼스 없이 잠글 수 있나 | **처분** |
|---|---|---|---|---|
| 1'' `ByteArrayInputStream` null NPE | **사라짐**(1→0) | 4줄 | ✔ | ★**이식** |
| 2a `System.arraycopy` null NPE | **사라짐**(2→0) | 5줄 | ✔ | ★**이식** |
| 4 `StringBuffer.append([CII)` null NPE | **사라짐**(1→0) | 4줄 | ✔ | ★**이식** |
| 8 `StringBuffer.insert(I,String)` | **사라짐**(1→0) | **53줄** + 시험 36줄 | ✔ | **미이식** |
| 9 `Timer.schedule(TimerTask,long)` | **사라짐**(1→0) | **17줄** + 시험 26줄 | △(시계 의존) | **미이식** |
| 5 pending-thread GC 루트 | ★**«사라지지 않았다»** — 프로브가 틀렸다(아래 정정) | — | — | ★**이식 불요** |

★★**갈래 = ⒝ «일부 이식». 고른 이유는 «줄 수»가 아니라 «실패의 등급»이다.**
- ★**이식한 3축은 «호스트 프로세스가 죽는다»** — 새 핀의 코드는 null `ClassInstanceRef` 를 곧장 역참조한다.
  ★**실측으로 재현했다**(9-3 개악 대조): `jvm/src/class_instance.rs:108` 의 `Option::unwrap()` 패닉.
  ⇒ 에뮬레이터가 통째로 죽고, 게스트는 잡을 수 없다.
- ★**미이식 2축(8·9)은 «메서드 부재»다** — 실패가 Java 레벨 해석 오류로 나와 **시끄럽고 잡을 수 있다.**
  ⇒ ★**같은 «하드닝»이라는 이름 아래 등급이 다르다.** 줄 수(53·17)보다 이 차이가 결정적이다.
- ★★**[2026-09-04 정정 · 게이트② 반려] 축 5 의 「사라짐 · 불가 · 영구 미복구」는 «참이 아니었다».**
  새 핀은 **같은 창을 다른 설계로 이미 닫아 놓았다**(이 회차가 소스로 재확인):
  `java/lang/Thread::start` 가 `jvm.new_global_ref(&this)` 로 만든 **`GlobalRef<Thread>`** 를
  `ThreadStartProxy` 가 **필드로 들고** spawn 콜백 전 구간을 산다(`thread.rs:194`·`:256`) ·
  `garbage_collector.rs::determine_garbage` 가 **`global_references` 를 루트로 순회**한다(`:34`) ·
  `global_ref.rs` 의 `impl Drop` 이 콜백 종료 시 해제한다(`:33`).
  ⇒ ★**fork 의 `add/remove_pending_java_thread` 와 «의미가 같다» — 보호는 그대로 있다.**
  ★★**왜 틀렸나 — 이것이 재발 방지의 핵심 문장이다: 13행 프로브는 «`pending` 이라는 fork 의 «식별자»»를
  세지 «보호»를 세지 않는다.** 이름이 바뀌면 «사라졌다»로 읽힌다.
  ★**§8-4⑸ 는 그것을 알고 「축 5 는 프로브로 판정 «불가»」로 정직하게 남겼는데, 이 절의 초판이 그것을
  「사라짐(2→0) · 불가 · 영구 미복구」로 «올렸다».** ⇒ ★**총괄에게 «없는 리스크»를 올린 것이고 그것이 더 나쁘다.**
  ※원본 커밋 계수(`daee53b` 34 insertions · `garbage_collector.rs` 7 + `jvm.rs` 18 = 25)는 **정확하다** — 그 수는 둔다.
  ⇒ ★**이 회차가 «갚지 못한» 값은 축 5 가 아니라 «미이식 2축(8·9)»뿐이다.**

★★**그리고 이식이 «가능»했던 이유를 적어 둔다 — 이 회차의 구조적 발견이다.**
`JvmRuntime::find_rustjar_class`(**wie 쪽 코드**)가 `java_runtime::get_runtime_class_proto(class)` 로 프로토를
받아 JVM 에 넘긴다. `JavaClassProto` 의 필드는 전부 `pub` 다 ⇒ ★**wie 가 그 사이에서 메서드 본문을 감쌀 수 있다.**
⇒ ★**「하드닝을 되돌리려면 fork 가 필요하다」는 참이 아니었다** — §8-6 은 그 가능성을 재지 않았다.

### 9-3. ★이식분의 잠금 — `wie_jvm_support/src/hardening.rs` (본문 103줄 · 시험 99줄 · 배선 19줄)

- `harden(&mut proto) -> usize` 가 프로토를 지나가며 `NullArgGuard` 로 본문을 감싼다. **적용 개수를 돌려주고**,
  못 찾으면 `tracing::error!` 를 찍는다 — ★**조용히 안 걸리는 것이 이 하드닝이 처음 사라진 방식**이기 때문이다.
- 시험 3종(**전부 `run_jvm_test` = 게임 파일 0**): ⒜`every_guard_is_actually_applied`(적용 수 == 1) ⒝세 null 인자가
  **NPE 로 온다** ⒞null 아닌 인자는 동작 불변(`arraycopy` 왕복).
- ★★**개악 대조**: `harden` 을 `return 0` 으로 무력화하면 ⒜는 `left: 0 / right: 1` 로, ⒝는
  ★**`jvm/src/class_instance.rs:108:32: called Option::unwrap() on a None value` 패닉**으로 red 가 된다.
  ⇒ ★**가드가 막고 있는 것이 «정확히 그 패닉»임을 코퍼스 없이 보였다.**

### 9-4. ★fork 의존 소멸 (실측)

`Cargo.toml` `[patch]` 절 **0** · `Cargo.lock` 의 `Jun025` 문자열 **0건** ·
`Cargo.lock` 의 RustJava source 6행 전건 `git+https://github.com/dlunch/RustJava.git?rev=5b84dd1c039f613f…` ·
`cargo tree -p wie_jvm_support` 의 `java_class_proto`·`java_constants`·`java_runtime`·`jvm`·`jvm_rust` **전건 dlunch@5b84dd1**.
※`Cargo.toml` 에 남은 `Jun025` 1건은 **주석**(무엇이 왜 없어졌는지를 다음 사람에게 알리는 문장)이다.

### 9-5. ★검증의 «한계» — green 을 하드닝 보존의 증거로 읽지 마라

4게이트 전건 green · `cargo test --all` **133 passed / 0 failed**(직전 130 + 이 회차 시험 3) ·
`wie_ktf`·`wie_lgt` helloworld **둘 다 ok**.
★★**그러나 §8-6 이 못박은 그대로다 — 「4게이트도 `cargo test --all` 도 green 인 채로 사라진다」.**
⇒ ★**green 은 «회귀 없음»의 증거이지 «하드닝 보존»의 증거가 아니다.** 보존의 증거는 오직 9-2 의 프로브와
9-3 의 개악 대조뿐이고, ★**축 8·9 에 대해서는 그 증거가 «없다»**(★**축 5 는 목록에서 빠졌다 — 2026-09-04 정정 · §9-2**).
★**코퍼스 축(P2)은 여전히 이 머신에서 불가**다(§8-1) — 이 회차가 그것을 바꾸지 않았다.

## 10. 미이식 하드닝 2축의 처분 (2026-09-04 · `wie-unported-hardening-two-axes-decide-with-a-corpus-probe`)

> 운영자 채택 제안 **`2026-09-04-upstream-realign-p1-pin-plus33#p0`** — 「이식하지 않은 2축을 «닫을지 말지» 결정하라」.
> 총괄 결정: ★**둘을 «한 덩어리»로 보지 않고 한 번의 값싼 측정으로 가른다.**
> ★**결론: 둘 다 이식했다.** 근거는 아래 «두 번째» 측정이다.

### 10-1. ★첫 측정 — 우리가 «가진» 아카이브의 상수풀 프로브 (결론을 내지 못했다)

저장소가 가진 게스트 아카이브 전수를 `.class` 상수풀의 `Methodref`/`InterfaceMethodref` 로 훑었다
(★문자열 grep 이 아니라 **상수풀 파싱** · 실행 0):
`test_data/draw_j2me.jar` · `test_data/helloworld_ktf.zip` · `test_data/helloworld_lgt.zip` ·
`docs/reference/AromaWIPI_classes.zip` ⇒ ★**표본 `.class` 226개.**

| 축 | 정확 일치 호출 |
|---|---|
| `java/util/Timer.schedule(Ljava/util/TimerTask;J)V` | **0건** |
| `java/lang/StringBuffer.insert(ILjava/lang/String;)…` | **1건** — ★그러나 그 1건은 `AromaWIPI` 라이브러리 **자신의 `StringBuffer.class`**(다른 오버로드가 이 것에 위임한다) ⇒ ★**게스트 호출부는 0건** |

★★**그리고 이 표본으로는 답이 «나오지 않는다» — 그 사실을 먼저 적는다.** 226개 중
`test_data` 3건은 **우리가 직접 만든 hello-world/draw** 이고, `AromaWIPI` 는 **게임이 아니라 플랫폼 라이브러리**다.
⇒ ★**「게임이 이 메서드를 부르는가」에 대한 표본이 «사실상 0» 이다.** 규칙 ⒝(둘 다 0 ⇒ 미이식)을
여기서 적용했다면 그것은 **측정이 아니라 표본 부재를 근거로 삼는 것**이었다.

### 10-2. ★★두 번째 측정 — «보호»를 세라: 플랫폼 표면 + ★**fork 커밋 로그의 실제 게임 이름**

★**⒜ 플랫폼이 게스트에게 «약속»하는 표면**(`AromaWIPI_classes.zip` 의 선언 메서드 파싱):

| 클래스 | 플랫폼이 선언 | 우리 핀(`5b84dd1`)이 가진 것 |
|---|---|---|
| `java/lang/StringBuffer.insert` | ★**9개 오버로드**(`(I,Object)`·`(I,String)`·`(I,[C)`·`(I,Z)`·`(I,C)`·`(I,I)`·`(I,J)`·`(I,F)`·`(I,D)`) | ★**0개** |
| `java/util/Timer` | `schedule(TT,J)` · `schedule(TT,Date)` · `schedule(TT,JJ)` · `schedule(TT,Date,J)` · `scheduleAtFixedRate(TT,JJ)` · `scheduleAtFixedRate(TT,Date,J)` · ★`cancel()` | `schedule(TT,JJ)` · `scheduleAtFixedRate(TT,JJ)` **뿐** |

★**⒝ 그리고 «누가 실제로 불렀는지»가 남아 있었다 — 버린 fork 의 커밋 메시지에.**
```
3cb4d7d  feat(java_runtime): Timer.schedule(TimerTask, long) one-shot overload
         "Trace-specified as method-not-found on 소울카드마스터2."
9cfc346  feat(java_runtime): StringBuffer.insert(I, String)
         "Trace-specified as method-not-found on 미니고치."
```
⇒ ★★★**둘 다 «실제 타이틀에서 관측된» 호출이다.** 코퍼스가 있던 시절의 관측이고, 여기서 어떤 프로브를 돌려도
그것을 대체하지 못한다. ⇒ ★**규칙 ⒜(「어느 한쪽이라도 호출이 있다 ⇒ 이식」)가 «둘 다»에 걸린다.**

★★**이것이 §9-2 의 판단을 정정한다** — P1 회차는 두 축을 「실패 등급이 낮다(메서드 부재는 시끄럽다)」로 미뤘고
검수는 「게스트가 부르는지는 코퍼스가 없어 못 쟀다」로 남겼는데, ★**그 증거는 우리 의존성의 이력 안에 이미 있었다.**
★**아무도 «거기»를 보지 않았다.** — §8-4⑸ 의 프로브가 「식별자를 센다」였던 것과 같은 형태의 실수다.

### 10-3. 처분 — 둘 다 이식(`wie_jvm_support/src/hardening.rs`) · 시험은 코퍼스 비의존

- `Timer.schedule(TimerTask,long)` — 핀의 `(TT,J,J)` 에 **period 0** 으로 위임한다(`TimerThread` 가 `period > 0` 일 때만 반복한다).
  ★**대체 수단이 없다**: 아무 period 나 주면 1회성 태스크가 «영원히 반복»된다.
- `StringBuffer.insert(int,String)` — fork 구현을 **그대로 이식**(재유도하지 않았다). 두 사양이 잃기 쉽다 —
  ★null String 은 문자 네 개 `"null"` 을 넣고 ★범위 밖 offset 은 던진다(`StringIndexOutOfBoundsException` 은
  이 런타임에 없는 클래스라 등록된 상위형 `IndexOutOfBoundsException`).
- ★**둘 다 «감싸기»가 아니라 «추가»다** — `add()` 는 핀이 나중에 같은 메서드를 갖게 되면 ★**덮지 않고 `tracing::error!` 로 신고**한다.
- **시험**(게임 파일 0 · 시계 비의존): `wie_jvm_support/tests/absent_methods.rs` 2건 ·
  ★Timer 는 «시계를 기다리지 않고» `TimerThread` 가 읽는 두 필드(`period`·`nextExecutionTime`)로 단언한다.
  ★**개악 대조**: `add()` 를 `return false` 로 무력화하면 행동 시험 **2/2 red** + 계수 시험 red.

### 10-4. ★남아 있는 «부분 표면» — 재개 조건을 «수»로 단다

이 회차가 닫은 것은 ★**`insert` 9개 중 1개 · `Timer` 5개 중 3개**다. 나머지는 **일부러** 두었다 —
이름이 붙은 타이틀이 없는 것을 사양만 보고 넣으면 «어느 것을 게임이 부르는지»를 추측하는 일이 된다.
★★**재개 조건(「나중에」가 아니다)**: ★**누락 오버로드를 지목한 게스트 실패 «1건» 관측.**
`wie_validate` 가 그 실패를 **디스크립터가 들어간 해석 오류**로 보고하므로 **트레이스 한 건이면 다음 대상이 정해진다.**
※부수 발견: `Timer.cancel()` 도 핀에 **없다**(플랫폼은 선언한다). ★이 회차 범위 밖이라 손대지 않았다 — 같은 재개 조건이 적용된다.
