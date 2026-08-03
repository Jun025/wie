# wie 현재 상태 (02_status)

> **기준일 2026-07-14** (공급망 절만 **2026-08-01** 갱신 — 아래 «공급망 추적 대장») · 레포 실측 기반(워크플로 실행 로그·커밋·베이스라인). 이 파일이 KB 의 최신 현황이다 — 매 작업 세션(main 반영) 시 재생성.

## 자동발행 파이프라인 (연방 ① 발신부) — **완전 라이브(dispatch 포함)**

`.github/workflows/publish-artifact.yml` (2026-07-08 라이브, dispatch 는 2026-07-09 PAT 재주입으로 활성):

- main push(엔진 소스 경로: `**/*.rs`·`Cargo.toml`·`Cargo.lock`·`build-wasm.sh`) → **fresh** `wie_web` WASM 빌드(wasm-bindgen 0.2.108 핀 + wasm-opt) → GitHub Release `engine-<shortsha>` 에 `wie_web_bg.wasm` + `wie_web.js` 발행(sha256 메타 포함, 같은 커밋 재실행 멱등).
- 최신 릴리스: **`engine-d7b5b02`**(2026-07-10).
- 이어서 otterpebble 에 `repository_dispatch`(event `wie-artifact-published`, payload: version·wieHead·wasmUrl/glueUrl·sha256·confirmedPlatforms `["KTF","SKT","LGT"]`) → 리시버(otterpebble 소유 `wie-artifact-receive.yml`)가 featurephone 재배포. (SoT: `publish-artifact.yml` dispatch 스텝 payload.)
- ~~PAT 403 잔여~~ — **해소 확인(2026-07-10)**: d7b5b024 발행 런에서 "Dispatch to otterpebble" 스텝 success. 구 403(2026-07-08)은 fine-grained PAT 권한 부족이었고 재발급·재주입으로 종결.

## 엔진 웹 계약 (소비자 featurephone 이 의존 — 변경 = 기획 사안)

원본: `wie_web/src/lib.rs`(wasm-bindgen 표면). 2026-07 현행:

- **생성자**: `new WieEmulator(filename, data: Uint8Array, canvas, audioCtx?, gain?, width, height)` — 7인자, 오디오 2개는 옵션(무음 시 undefined).
- **렌더**: 엔진이 **캔버스에 직접 blit**(더블버퍼 putImageData→drawImage, CSS pixelated 스케일). 소비자는 `requestAnimationFrame` 에서 `tick()` 만 호출.
- **입력**: `key_down/key_up/key_repeat(code)` — 키코드 문자열: `UP DOWN LEFT RIGHT OK LEFT_SOFT_KEY RIGHT_SOFT_KEY CLEAR CALL HANGUP VOLUME_UP VOLUME_DOWN NUM0`~`NUM9 HASH STAR`(미지 코드는 무시).
- **종료**: `has_exited(): boolean` — **additive getter(2026-07-10, featurephone 요청·셸소유 확정 계약변경)**. 코어가 정상 종료를 요청한 순간(= `[wie] emulator requested exit` 로그 경로, 현행 도달점 WIPI `MC_knlExit`) true 로 플립, **sticky**(한번 true 면 인스턴스 수명 내내 유지). 플립 후 `tick()` 은 **안전한 no-op**(코어 미전진, `Ok` 반환 — rAF 루프가 플래그 관찰 전에 더 돌아도 무해). 런타임 실패 = 기존대로 `tick()` 이 예외를 던짐 → 셸 구분법: **clean exit = getter true·throw 없음 / 실패 = throw**. 세이브는 플립 후에도 읽기 가능 — 매 `tick` 후 getter 폴링, true 시 `export_saves()` 로 persist 후 `free()`. 구(폐지) has_exited 의 제거를 revert 한 것이 아니라 현 모델 위 순수 additive(기존 소비자 무손상).
- **오디오**: JS 가 사용자 제스처에서 만든 `AudioContext`+`GainNode` 를 주입, PCM 은 WebAudio 로 갭리스 스케줄, MIDI 는 무음 스텁.
- **세이브**: `has_saves()` / `export_saves()`(불투명 `WIESAV01` 블롭: RMS+FS) / `import_saves(blob)` / `export_fs`·`import_fs`. 해제는 `free()`.
- **로더**: `.zip`→KTF→LGT→SKT 순 판별, `.jar`→KTF→LGT→SKT→J2ME 폴백, `.jad` 는 거부(.jar 요구). `platform_kind()` 가 `"KTF"|"LGT"|"SKT"|"J2ME"` 반환.
- **LGT 컴파일모델**: `lgt_compile_model(): string | null` — **additive getter(2026-07-13, featurephone 옵션1 확정·셸소유 계약변경)**. LGT 타이틀이면 `"clet"`(WIPI-C, wie 렌더 가능) 또는 `"aot-java"`(AOT-Java, 부팅되나 §7 벽으로 미렌더), **비-LGT(KTF/SKT/J2ME)는 `null`**(개념 부적용 — wasm-bindgen `Option<String>`→JS null, 셸이 `=== "aot-java"` 한 줄로 차단 판정·나머지는 falsy 통과라 구분 최단). 판별근거=앱 자신의 import thunk 정적 스캔(`binary.mod` ELF 실행섹션에서 `bl`+`.word 0x64` java-interface thunk 유무; 0x64⇒aot-java, 부재⇒clet). **생성자에서 1회 산정, 인스턴스 수명 내 불변 — 첫 `tick()` 전 즉시 유효**(로드 성공 직후 호출 가능). read-only 조회, 런타임 무영향. 코퍼스 실측: working/lgt 54 전부 clet, broken/lgt 24 aot-java(deep-assets 알려진 24종과 정확 일치)+22 clet, ambiguous 0. `platform_kind()` 등 기존 표면 전부 무변경 순수 additive. 권고 셸 패턴: `"aot-java"` 업로드 시점 실행 차단+"준비 중" 안내, `"clet"`만 실행.

## CI 현황

- **Rust CI**(`rust.yml`): 3-OS 매트릭스, `fmt --check` + `clippy -D warnings`(wasm32 타깃 포함) + 전체 테스트 — **green**(clippy red 해소됨).
- **coverage**(`coverage.yml`): `CODECOV_TOKEN` 없으면 업로드만 skip(fork-safe) — green.
- **Web**(`web.yml`): wasm 빌드 + Cloudflare Pages 배포, Pages 프로젝트(`wie-web`) 부재 시 자가 재생성(`pages project create || true`).
- **Security audit**(`rust-audit.yaml`, 매일 schedule + `workflow_dispatch`): **schedule 경로 정정(2026-07-22, `[wie-security-audit-schedule-red]`)**. 종전 KB 의 "green(2026-07-10 해소)" 서술은 **실측과 불일치한 오독이었다**: `gh run list -w "Security audit"` 보존 이력의 **schedule 런 28건 전부(2026-06-25~07-22) failure**(예: 29887989781·29798146955), 유일 success 는 2026-07-10 **workflow_dispatch** 1건(같은 날 같은 커밋의 schedule 런 29067652864 는 failure). 원인은 취약점이 아니라 **fork+Issues 비활성**: 이 repo 는 `dlunch/wie` fork 라 Issues 가 기본 off 이고 repo-레벨 disable 은 토큰 권한으로 못 넘긴다. 그런데 종전 `rustsec/audit-check` 액션이 경고 2건을 **GitHub Issue 로 올리려다** `Issues has been disabled in this repository.` 로 매 schedule 런에서 죽었다. 선행 커밋 `7405c50b` 이 붙인 `issues: write` 는 애초에 성립 불가한 처방이었고, 같은 커밋이 KB 를 red→green 으로 바꾼 것이 오독의 출처. **정정 조치**: 리포팅 경로를 `cargo audit` 직접 실행으로 전환(Issue 생성 안 함 → fork 에서 성립), 무효 권한 `issues/checks: write` 제거. **부류 분리 실증(로컬)**: 경고만 있는 현 상태 = exit 0(green), 취약점 존재 시(quick-xml ignore 제거) = exit 1(red) — 종료코드로 구분. ★정확한 성격 규정: 이 잡은 **공급망 차단 게이트가 아니다**(`.github/workflows/` 전수에 `workflow_run` 소비 참조 0건, publish-artifact 독립 구동). 탐지(cargo audit) 자체는 정상 동작하며 Actions 탭에 red 로 보였으므로 무성 실패도 아니었다 — 실제로 죽었던 것은 **알림 채널(Issue 생성)과 신호 대 잡음비**다. **PENDING 2건**: ① `quick-xml` 0.39.2(RUSTSEC-2026-0194·0195, XML DoS) — 패치는 0.41.0에만 존재하나 유일 소비자 `wayland-scanner`(최신 0.31.10)가 `^0.39` 요구로 상위 차단. 빌드타임 proc-macro 가 vendored 신뢰 XML 만 파싱해 공격면 없음 — 근거 명시 후 개별 `--ignore` 처리, wayland-scanner 가 ≥0.41 채택 시 ignore 제거. ② `ttf-parser` 0.25.1 unmaintained(RUSTSEC-2026-0192, patched 버전 없음)·`spin` 0.12.0 yanked — 둘 다 **비게이팅 경고**(로그에는 보이나 exit 0). skrifa 이행은 `ab_glyph`(최신 0.2.32도 ttf-parser 의존) 교체가 선행돼야 해 보류. **로드맵에 미래 트랙으로 등재(아래 5번)** — 매일 audit 경고가 잊히지 않게 추적.
- **RUSTSEC 권고 6건 일괄 판정(2026-07-31, `[wie-rustsec-advisory-sweep-batch1]`)** — upstream `dlunch/wie` 이슈 #1254·#1252·#1251·#1250·#1238·#1161 전건. 결론: **취약점 0건 · ignore 0건**(`cargo audit` 무-플래그 exit 0).
  - ★**quick-xml 상위 차단 해소(#1252 RUSTSEC-2026-0195 · #1251 RUSTSEC-2026-0194)**: 07-22 기재의 «wayland-scanner 0.31.10 이 `^0.39` 요구» 는 **더 이상 사실이 아니다** — `wayland-scanner` **0.31.11 이 `quick-xml ^0.41` 채택**. `cargo update -p wayland-scanner` 만으로 quick-xml **0.39.2 → 0.41.0**(패치 경계 `>=0.41.0`) 상향, 잠금 변동 **정확히 2 패키지**. 이에 따라 `rust-audit.yaml` 의 `--ignore RUSTSEC-2026-0194/0195` **제거** — 이제 이 잡은 **ignore 없이** 돈다.
  - **#1254(crossbeam-epoch RUSTSEC-2026-0204) · #1238(memmap2 RUSTSEC-2026-0186) · #1161(rand RUSTSEC-2026-0097)** — **전부 이미 패치 버전**(0.9.20 / 0.9.11 / 0.10.1 = 각 권고의 `patched` 경계와 동일)이라 07-31 이전부터 `cargo audit` 무-검출이었다. **도달성도 별도 확인**(«버전만 봤다» 금지 조항): crossbeam 은 `crossbeam::channel` 만 사용(epoch 의 `Atomic`/`Shared` 를 `fmt` 로 출력하는 경로 0건, `wie_core_arm/src/gdb.rs`·`engine/debugged_arm32_cpu.rs`) · `memmap2` 는 우리 소스 참조 0건(winit/softbuffer/sctk 내부 전용, 취약 함수 `advise_range`/`flush_range` 미호출) · `rand` 는 rodio 경유 간접 의존이고 권고의 전제인 «`rand::rng()` 를 부르는 커스텀 `log` 로거» 부재(우리는 `tracing_subscriber::fmt`).
  - **#1250(ttf-parser RUSTSEC-2026-0192 unmaintained)** — **수용 유지**. `patched = []`(버전업으로 안 풀림)이고 `ab_glyph` 는 **최신 0.2.32 도 `owned_ttf_parser ^0.25` 의존**(crates.io 실측 07-31)이라 선행요건 미충족. 비게이팅 경고로 남기고 아래 5번 트랙에서 계속 추적.
  - ※`spin` 0.12.0 yanked(최신 0.12.2)는 이 6건 밖이라 **손대지 않음** — 배치2 후보. **⇒ 아래 «공급망 추적 대장» A-3 으로 등재됨(2026-08-01).**

## ★공급망 추적 대장 (RUSTSEC 경고 · 미해소 이월분) — 2026-08-01 등재 `[wie-rustsec-advisory-sweep-batch2]`

> **이 절이 이월분의 정본이다.** 배치1(2026-07-31)까지는 이월 목록이 완료 보고서 본문에만 존재해
> 추적 대상이 아니었다(게이트② 사후 감사 [4] «등재 없음 = 유실» 지적). 여기로 옮겨 등재한다.
> **매일 도는 `cargo audit`(`rust-audit.yaml`)의 비게이팅 경고가 곧 이 표다** — 경고 수와 이 표의
> 행 수가 어긋나면 둘 중 하나가 낡은 것이다.

### A. 권고·공급망 3건 — **전건 비게이팅 경고**(`cargo audit` 무-플래그 **exit 0 · vulnerabilities 0**)

실측 기준: `cargo-audit 0.22.2` · advisory-db `685d32fd`(2026-07-31) · `Cargo.lock` 435 패키지 ·
2026-08-01 `main` 형상. **억제(`--ignore`·`audit.toml`) 0건** — 이 0은 억제로 만든 0이 아니다.

| # | 대상 | 권고 | 성격 | 설치 버전 → 패치 | 판정 | 다음 행동 |
|---|---|---|---|---|---|---|
| **A-1** | `ttf-parser` | RUSTSEC-2026-0192 | `unmaintained`(informational) | `0.25.1` → **`patched = []`** | **수용 유지** — 버전업으로 안 풀린다 | 아래 로드맵 5번(ab_glyph→skrifa)에서 계속 추적. **착수 아님** |
| **A-2** | `event-listener` | RUSTSEC-2026-0221 | `unsound`(informational) | `5.4.1` → **`>= 5.4.2`** (**버전축 affected**) | **도달 불가**(근거 아래 ⓐ) — 상향은 가능 | `cargo update -p event-listener` **1패키지** 이동(dry-run 실증). **별건 발권 대상** |
| **A-3** | `spin` | **권고 없음**(yanked) | 취약점 아님 — 레지스트리 신호 | `0.12.0`(yanked) → `0.12.2` | **도달 불가**(근거 아래 ⓑ) — 상향은 가능 | `cargo update -p spin` **1패키지** 이동(dry-run 실증). **별건 발권 대상** |

#### ⓐ A-2 `event-listener` RUSTSEC-2026-0221 — «버전이 같으니 패치됨» 이 아니라 «도달하지 않는다»

권고 원문(advisory-db `685d32fd`): `patched = [">= 5.4.2"]` · `unaffected = ["< 5.1.0"]` ·
`informational = "unsound"`. 설치본 `5.4.1` 은 **두 경계 사이 = affected 구간**이다.
★**버전축만 보면 «해당됨»** 이므로, 판정 근거는 전적으로 도달성이다.

권고가 지목한 것은 **`StackSlot<'_, T>`**(= `listener!` 매크로가 만드는 스택 할당 리스너)의
무조건적 `Send`/`Sync` 구현이며, 성립 전제는 **`Event::with_tag` 로 세운 `!Send` 태그 타입**이
스레드를 넘는 것이다. 두 축 모두 부재 —

- **의존 경로**: `Cargo.lock` 역의존 파싱 결과 소비자는 **`jvm 0.0.1` 단 하나**
  (= `[patch]` 로 고정한 `Jun025/RustJava` fork `c66f08d4`). 다른 크레이트 **0건**.
- **fork 실사용**(체크아웃 원본 grep): `jvm/src/jvm.rs:12 use event_listener::{Event, EventListener};`
  뿐이고, 실제 사용은 `Event::new()`(→ 태그 `()` = `Send + Sync`) + `.listen()`(→ **힙** `EventListener`)
  2곳(`jvm.rs:475 object_listen`, `753 get_or_create_monitor`).
  **`listener!` 0건 · `StackSlot` 0건 · `Event::with_tag` 0건**
  (`listener!|with_tag|StackSlot` 전수 grep 2건은 전부 `classfile/src/constant_pool.rs` 의
  **`parse_with_tag`** — JVM 상수풀 파서로 무관).
- **우리 repo 소스**: `event[-_]listener|EventListener` grep **22건 — 전건이 «에뮬레이트되는 자바
  클래스 이름»** (`org/kwis/msp/lwc/EventListener`, `org/kwis/msp/lcdui/JletEventListener`)이고
  Rust 크레이트와 **무관**. ★이름 충돌이라 순진한 grep 은 «22건 도달» 로 오독된다 — **실 도달 0건**.

⇒ **취약 타입이 의존 폐포 어디에서도 생성되지 않고**(1축), **성립 전제인 `!Send` 태그도 부재**(2축).
독립 2축이 각각 판정을 지지한다. 상향(`5.4.2`)은 `jvm` 의 요구 `^5.4` 안이라 무마찰이지만,
**이 티켓은 판정·등재 범위이므로 lockfile 무변경**으로 남긴다.

#### ⓑ A-3 `spin` 0.12.0 yanked — ★**yanked 는 취약점이 아니다**(성격부터 가른다)

- **취약점이 아닌 이유**: RustSec 에 `spin` 권고는 3건뿐이고 **셋 다 우리 설치본에 미해당** —
  RUSTSEC-2019-0013 `patched = [">= 0.5.2"]` · RUSTSEC-2023-0031 `patched = [">= 0.9.8"]`
  (둘 다 `0.12.0` 이 이미 상회) · RUSTSEC-2019-0031(`unmaintained`)은 `unaffected = [">= 0"]` 로
  **무력화된 이력 항목**. `yanked` 는 advisory-db 가 아니라 **crates.io 인덱스 상태**이며,
  `cargo audit` 도 이를 `warning: yanked` 로만 보고한다(**exit 0**). 즉 **«알려진 취약점» 신호가 아니다.**
- **그럼에도 문제일 수 있는 이유**(무시하면 안 되는 축): yanked 는 «저자가 이 버전을 쓰지 말라고
  선언» 이다. 실측한 yank 형상은 **광범위·동시다발**이다 —
  `0.9.7 · 0.9.8 · 0.10.0 · 0.11.0 · 0.12.0 · 0.12.1` 이 모두 yanked 이고,
  **2026-07-13 하루에 6개 라인의 패치본이 동시 발행**(`0.7.2 · 0.8.1 · 0.9.9 · 0.10.1 · 0.11.1 · 0.12.2`).
  전 라인 백포트 팬아웃은 통상 **건전성 결함**의 형상이다.
- **실제 사유 확정**(추정하지 않고 원문을 떴다 — `.crate` 2본 내려받아 diff): 패키징된
  `CHANGELOG.md` 가 사유를 명시한다. **`0.12.1`** = `lock_api` 인터페이스의 `RwLock::try_upgrade`
  경로 unsoundness(+ `LazyLock` 의 `Sync` 구현 완화) · **`0.12.2`** = `Once::force_into_inner` ·
  `Once::try_into_inner` · `Once::into_inner_unchecked` unsoundness.
  ※ 이 사유는 **GitHub `mvdnes/spin-rs` 에서는 확인되지 않는다** — 그 repo 의 최신 커밋은
  2026-05-14, CHANGELOG 최신 항목은 `0.11.0` 이라 **0.12.x 이력 자체가 없다**(발행은 공동 소유자
  `zesterer` 경유). ★**사유 확인은 crates.io 패키지 본문이 유일 경로**라는 점을 기록해 둔다.
- **⇒ 도달 불가 판정**: 우리는 `spin` 을 **`default-features = false`, `features = ["spin_mutex","rwlock"]`**
  로 쓴다(`Cargo.toml:46`, 워크스페이스 6 크레이트가 이를 상속). spin 의 feature 표상
  **`once` 미활성 ⇒ `Once` 자체가 컴파일되지 않고**(0.12.2 결함 부재),
  **`lock_api` 미활성 ⇒ `dep:lock_api_crate` 미링크**(0.12.1 결함 부재),
  `lazylock = ["once"]` 도 미활성. **기계 확증**: `Cargo.lock` 의 `spin` 블록에 **의존 항목이 아예 없다**
  (트리의 `lock_api` 는 winit 계열 `parking_lot` 소비분이고 spin 과 무관).
  소스 실사용도 `spin::` 전수 **13건 = `Mutex` 12 + `RwLock`/`RwLockWriteGuard` 1** 뿐이고,
  `Once` **0 · `lock_api` 0 · `upgradable_read`/`try_upgrade` 0 · `*into_inner*` 0**.
- **결론**: **취약점 아님 · 결함 도달 불가**. 다만 yanked 의존은 «저자 비권장 + 신규 lockfile 생성 시
  재현 불가» 라는 **위생 부채**이므로 상향 자체는 하는 게 맞다 — **별건 발권**.

### B. upstream `dlunch/wie` 이슈 9건 — 배치1 이월분 (2026-08-01 **재조회 실측**)

배치1 done §5(b) 의 표를 **그대로 옮긴 뒤 현재 상태만 재확인**했다(목록 신규 작성 아님).
★**우리는 이 repo 에 close·label 권한이 없다**(`gh api repos/dlunch/wie` → `pull:true` 외 전부 false ·
`Jun025/wie` 는 fork 라 Issues 비활성). 따라서 이 표는 **추적용**이며 upstream 조작은 하지 않는다.

| # | 제목 | 성격(배치1 분류) | **2026-08-01 재조회** | 분류 판정 |
|---|---|---|---|---|
| **1292** | LGT `java_import_10`(import table 0x64) — blocks `Display.setCurrent()` | 엔진 결함 · **최우선 후보** | ★**CLOSED/COMPLETED**(2026-07-31, 작성자 `jjongjjongs` 자진 종결) | ★**변경** — 아래 ※ 참조 |
| **1260** | 영웅서기5 LGT — Unknown SVC id 1409 | 엔진 결함(미구현 SVC) | OPEN(2026-07-27 갱신) | **유지** — 실질 개발 대상 |
| **1122** | Wie 실행중 게임 정지 현상 | 크래시/행 — 재현 필요 | OPEN(2026-05-10, 정체) | **유지** — 실질 개발 대상 |
| **1240** | 서울타이쿤2 | 게임 호환 리포트 | **OPEN/REOPENED**(2026-07-27) — KTF 기동 성공 PR #1291 이 upstream 에 올라옴 | **유지**(호환 리포트) · upstream 에서 진행 중 |
| **1130** | 레이카르나 LGT | 게임 호환 리포트 | OPEN(2026-03-22, 코멘트 0) | **유지** |
| **980** | Sideloading download data | 기능 요청 | OPEN(2025-09-17) | **유지** |
| **1152** | Feat.req. Offline portable version | 기능 요청 | OPEN(2026-04-29) | **유지** |
| **1253** | 메이플스토리 지원 예정 문의 | **문의**(코드 변경 아님) | OPEN(2026-07-18) | **유지 — 구현 대상에서 분리** |
| **1127** | 와일드프론티어 지원 가능 문의 | **문의**(코드 변경 아님) | OPEN(2026-04-30) | **유지 — 구현 대상에서 분리** |

- ⇒ **분류 갱신**: 배치1 이 «실질 개발 = 1292·1260·1122» 라 했으나 **1292 가 종결**되어
  현재 실질 개발 후보는 **1260 · 1122 2건**이다. 문의 2건(1253·1127) 분리는 **유지**.
- ※★**1292 종결이 우리 쪽 블로커를 해소한 것은 아니다.** 그 이슈의 실체는 **트랙② §7 벽**
  (LGT AOT-Java 의 `0x64` ordinal→native 등록표)과 같은 문제이고, upstream 코멘트에서 메인테이너가
  **저작권 사유로 펌웨어/공식 에뮬레이터 리버싱을 하지 않는다**고 명시했다(2026-07-27, `dlunch`).
  즉 **«upstream 에서 닫혔다» = «해결됐다» 가 아니라 «upstream 이 다루지 않는다»** 에 가깝다 —
  아래 트랙② «실기 트레이스 필요 · 동결» 판단은 **그대로 유효**하다.
- **본 티켓의 범위는 분류·등재까지다 — 구현 0건**(코드 변경 없음). 개발은 건별 발권 대상.

## 두 트랙 — 엔진 정상화 현황

**트랙 ① 타이틀 회수(모드 A — 자율주행 진행 중)**
- **회귀 베이스라인**(`scripts/smoke_gate_baseline.tsv`): **292 타이틀 부팅+렌더 PASS — KTF 190 / LGT 52 / SKT 50**(2-run 교집합 검증, 게임파일 미포함·식별자만). 게이트는 부팅+렌더만 판정(입력 생존은 비게이팅 어드바이저리). 구 스냅샷(2026-07-02) 202 대비 **+90**.
- **커버리지 오딧 승격(2026-07-13)**: 코퍼스 전량(ktf190/lgt54/skt50=294) 대비 기존 261 등재분을 대조해 "PASS-both 이나 미등재" 후보 **31종(전부 SKT)** 특정 → 각 타이틀 SKT 2-run 독립 실행(A/B 각 SKT50/50 전수 PASS)으로 교집합 승격, **261→292**. 후보가 SKT 뿐인 이유: KTF 코퍼스 190=베이스라인 190(전량 등재 완료), LGT 코퍼스 54 중 52 PASS 전량 등재(FAIL 2 제외). 승격 후 전체 코퍼스 2-run 재검증 — 두 런 모두 **292 전수 회귀-0·absent 0**. baseline.tsv 데이터 등재만(엔진 런타임 무변경, 회귀-0 자명).
- **제외 2건(등재 금지)**: `lgt/놈ZERO` = 기지의 per-game FAIL(누락 blit SVC 아님, 게임별 near-blank). `lgt/하이브리드` = 선재 핀 이슈(널점프 inject runaway) **PENDING·미접촉** — 승격/수정 금지.
- **d7b5b024(sec/audit-green 머지) 게이트 소급 확정(2026-07-10)**: 코퍼스 복귀 후 2-run 실측 — 두 런 모두 **베이스라인 261 전수 회귀-0**(294 중 292 PASS, FAIL 2건은 두 런 동일한 비-베이스라인 LGT 타이틀 놈ZERO·하이브리드). crossbeam-epoch 0.9.20 등 lockfile 패치 상향의 회귀-블라인드 해소. ② has_exited getter 변경은 wasm32 전용 `wie_web` 한정 — 네이티브 `wie_validate` 바이너리 sha256 동일 실증(재컴파일 미발생, wie_cli 의존트리 무관)으로 동일 2-run 이 변경 후 트리에도 유효.
- 최근 리듬(git log): WIPI-Java/MIDP 메서드 보강 · RustJava 포크 핀 상승(트랙2 클러스터 다수 귀속: readUnsignedByte·TimeZone·Byte 등) · 결정적 실행기(BTreeMap 폴링·스레드 스케줄링 = 구 트랙1 반영)로 232→261.
- dispatch 의 `confirmedPlatforms` 는 **KTF·SKT·LGT**(2026-07-14 LGT 승격). SKT 는 코퍼스 50종 전량 베이스라인 등재. **LGT 는 clet 52종 confirmed** — 셸이 `lgt_compile_model()==="aot-java"` 로 AOT-Java 24종을 사전 제외하므로 confirmed 는 **clet 서브셋 정식 지원**을 의미(AOT 24종은 §7 동결·"준비 중", 렌더 가능 승격 아님). J2ME 는 웹 로더 폴백 지원.

**트랙 ② §7 벽 — LGT AOT-Java 렌더(모드 B — 외부 산출물 대기)**
- LGT AOT-Java 24종은 렌더 0 유지. 바이너리-측 조사는 cp59 로 완결: per-frame 구동은 TIMER_EVENT(21) 모델로 확정(구현 가능), 유일 블로커는 **0x64 ordinal→native 등록표**. 오프라인 획득 소진 증명(AromaWIPI 비공번호 — `docs/reference/lgt_0x64_ordinal_table.md`) → **실기 트레이스 필요**. 도착 시 4단계 즉시 활성화 스캐폴드 커밋됨(기본 비활성·회귀 0). 요약: `10_deep-assets.md`, 원문: `docs/lgt_abi.md` §7·§8.

**LGT confirmed 승격 선결조건 = AOT-Java graceful 제외 신호 (2026-07-13 조사, 미구현)**
- **Q1 실패모드 실측**(배틀몬스터 2빌드, wie_validate=웹과 동일 `LgtEmulator::from_archive` 경로): 진짜 AOT-Java 는 **silent blank(throw 없음)** — paints=1·content=false·max_ticks(5천만) 완주, `run_err` 미발생(reason="only blank/uniform frames"). 부팅은 성공(`registered 20 app classes` + import table `0x64` 다수)하나 §7 렌더 드라이버 부재로 조용히 검은화면. **최악 케이스**: 셸의 현행 실패감지(tick() throw / has_exited)로 감지 불가. ※ 대조: broken/lgt 의 clet 미완성분(영웅서기4=WIPIC SVC 111, 붉은보석=stdlib 0x3f7)은 **throw** 하고, 제노니아2 는 이제 PASS — 즉 broken 폴더는 AOT 전용 아님, 실패모드는 서브셋별로 갈림.
- **Q2 판별 신호 가용성**: clet↔AOT 구분은 컨테이너/파일명으론 불가(양쪽 다 jar 안 `binary.mod` + `app_info`, MClass 필드는 대부분 공란이라 비신뢰). **판별점 2개 실재**: ⓐ **정적(로드 전)** — `binary.mod` ELF import thunk 의 `0x64`(Java-interface) 참조 유무. deep-assets 가 16바이트 thunk 패턴으로 24/102 정적 특정 완료해 **로드 전 파일 바이트만으로 판별 가능** 실증됨. ⓑ **부팅 극초기(첫 tick 전, `load_native` 내)** — `register_app_classes` 반환 non-empty(AOT 는 `.data` 에 class descriptor, clet 은 없음) + 첫 import table `0x64`(AOT) vs `0x1fb`(WIPI-C clet). 현 로더는 `loadable_archive`(app_info 존재)·`loadable_jar`(binary.mod 존재)로 **LGT 판정만** 하고 컴파일모델은 표면에 미노출 — `platform_kind()="LGT"` 한 단계 아래 정보는 내부에 존재하나 셸이 못 봄.
- **최소 additive 제안(구현 금지·형태만)**: `platform_kind()` 무변경 유지 + ▸옵션1(선호, 정적) 별도 판별 getter 예 `lgt_compile_model() -> "clet"|"aot-java"` — 로더가 binary.mod 의 0x64 thunk 정적 스캔(추출기 기존)으로 셋, 셸이 "aot-java"면 사전 "미지원 서브셋" 안내 후 제외. ▸옵션2(로드실패 명시화) AOT 감지 시(class descriptor non-empty && 렌더드라이버 부재) 로드 단계에서 명시적 `WieError` 반환해 현행 silent-blank 를 explicit-throw 로 승격 — 단 런타임 동작 변경이라 계약·회귀 검토 필요. 둘 다 기존 표면 무변경 후방호환.
- **승격 안전성 판정**: Q1(AOT=silent blank, 감지불가) + Q2(판별신호 명확히 가용) 종합 → **"명시적 caveat + graceful 제외 신호 선행 필요"**. additive 신호를 노출하고 셸이 AOT 서브셋을 사전 제외하면 승격 안전(그때 clet 서브셋만 confirmedPlatforms 승격).
- **LGT confirmed 승격 4단계 = 전체 완료(2026-07-14)**: **① 엔진 getter 노출 = 완료(2026-07-13, #33)** — `lgt_compile_model()` 라이브(엔진 웹 계약 참조, 옵션1 정적 0x64 thunk 스캔). **② 셸 배선(featurephone "aot-java" 차단+안내) = 완료**. **③ clet-only 재검증 = 완료**. **④ confirmedPlatforms 에 LGT 승격 = 완료(2026-07-14)** — `publish-artifact.yml` dispatch payload `["KTF","SKT"]`→`["KTF","SKT","LGT"]`(발신부 메타데이터 한정, 런타임 무변경). **범위 불변식**: clet 52종의 platform-level confirmed 선언일 뿐 AOT-Java 24종을 렌더 가능으로 만들지 않음 — 셸이 aot-java 를 사전 제외하므로 confirmed=clet 서브셋 정식 지원, AOT 24종은 §7 동결 유지.

## 로드맵 위치 · 잔여

1. ~~dispatch PAT 권한 수정~~ — **완료(2026-07-10 확인)**: d7b5b024 발행 런 dispatch success, 자동 전파 완전 라이브.
2. ~~security audit red 해소~~ — **schedule 경로 정정 완료(2026-07-22, `[wie-security-audit-schedule-red]`)**: 2026-07-10 "해소" 는 dispatch 런 오독이었고 schedule 은 전건 failure 였다(원인 fork+Issues 비활성). `cargo audit` 직접 실행으로 전환해 알림 채널 의존 제거 — 취약점=red / 경고=green 부류 분리 확립. 잔여 PENDING 중 **quick-xml 상위 차단은 2026-07-31 해소**(wayland-scanner 0.31.11 이 `^0.41` 채택 → 상향 후 `--ignore` 제거, `[wie-rustsec-advisory-sweep-batch1]`); 남은 것은 ttf-parser unmaintained(아래 5번)·spin yanked·event-listener unsound 3건이며, **2026-08-01 부터 «공급망 추적 대장» A 표가 정본**이다(`[wie-rustsec-advisory-sweep-batch2]`) — 전건 비게이팅 경고(exit 0)·억제 0. ~~코퍼스 복귀 시 261 재확인 권장~~ → **소급 확정 완료(2026-07-10, 트랙① 참조)**.
3. 트랙 ① 지속(292+) · **LGT clet 확정 승격은 AOT-Java graceful 제외 신호 선행 필요**(트랙② 하단 조사 참조 — Q1 silent blank·Q2 판별신호 가용, 유보 권고) · 플레이키 타이틀(입력 타이밍) 분류.
4. 트랙 ② 는 실기 트레이스 확보(사람/외부) 전까지 동결 — 재조사 금지 목록 준수(`10_deep-assets.md` 가드레일).
5. **[미래 트랙 — 착수 아님] ab_glyph→skrifa 폰트스택 이행**: `ttf-parser` RUSTSEC-2026-0192(unmaintained) 해소의 선행요건. `ab_glyph` 가 최신(0.2.32)까지 ttf-parser 에 의존하므로 폰트 렌더 스택 자체를 skrifa 계열로 교체해야 경고가 사라진다. 지금 구현 착수 금지 — 매일 audit 의 비게이팅 경고를 이 항목으로 추적(ab_glyph 의 탈-ttf-parser 릴리스 또는 skrifa 직접 이행 타당성 재평가 시 활성화). = 대장 **A-1**.
6. **[발권 대기 — 미착수] 공급망 위생 상향 2건**: **A-2** `cargo update -p event-listener`(5.4.1→5.4.2, RUSTSEC-2026-0221 해소) · **A-3** `cargo update -p spin`(0.12.0→0.12.2, yanked 해소). 둘 다 dry-run 실측 **1패키지 이동**이고 도달성 판정은 이미 «도달 불가» 로 끝나 있다(대장 ⓐ·ⓑ) — 즉 **긴급도 0, 위생 부채**. `[wie-rustsec-advisory-sweep-batch2]` 는 **판정·등재까지**라 lockfile 을 건드리지 않았다. 각각 소티켓으로 발권할 것.
7. **[발권 대기 — 미착수] upstream 이월 이슈**: 실질 개발 후보 **#1260**(LGT Unknown SVC id 1409) · **#1122**(실행중 정지 재현). 대장 B 표 참조 — 건별 발권 대상이며 문의 2건(#1253·#1127)은 구현 대상 아님.
