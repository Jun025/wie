## [2026-10-03] LGT 게스트 객체 누수 — 게스트 레지스터·스택을 보수적 GC 루트로 (wie-guest-gc-root-conservative-scan-live-lgt-leak)

**무엇을**: 0418 §1 이 원인만 재고 남긴 라이브 LGT `1b107b96bf4e` 누수를 고쳤다. ⒜ 게스트 `new` 를 jvm 객체 집합에 넣는다. ⒝ 게스트→호스트 호출마다 jvm 프레임을 쌓는다. 그 둘이 여는 위험(게스트만 쥔 객체의 해제)은 ⒞ 수집 때마다 게스트 레지스터·스택·데이터를 보수적으로 훑어 루트로 넣는 것으로 막는다. 디버그 모드 `wie_validate --gc-stress N` 을 더했다.
**판정**: 누수 장면(대국)에서 옛 루트 모델은 10분에 인스턴스 1,120 → 28,431 로 단조 증가한다. 새 모델은 60분 동안 평탄하다(10분 구간 평균 2,307 · 2,378 · 2,405 · 2,405 · 2,410 · 2,401). 스트레스 모드(최대 매 호출 수집 · 해제 블록 독 채우기) 위반 0. 0418 의 «Loading» 정체는 r3 퇴행이 **아니다** — 같은 키로 main 빌드에서도 간헐 재현된다(§5).
**사용자 영향**: 문자열을 매 프레임 만드는 LGT 게임을 오래 해도 메모리가 차지 않는다. `1b107b96bf4e` 는 약 40분이면 힙이 차서 게임이 멈췄다.

증적: `~/orchestrator/reports/evidence/wie-guest-gc-root-conservative-scan-live-lgt-leak/`. 타이틀은 sha12 로만 적는다.

### 1. 무엇이 루트였나 — 0418 §1 의 두 갈래(코드 재확인)

- ⒜ `java_instantiate`(게스트 `new`)는 `ClassDefinition::instantiate` 를 바로 불러 jvm 의 `all_objects` 에 들어가지 않는다. 수집 대상이 아니다. #445 는 그 객체가 쥔 자식(`StringBuffer` 의 `char[]`)이 먼저 해제되지 않도록 전역 참조로 붙들었다. 그래서 자식까지 영원히 산다.
- ⒝ `JavaMethodProxy::call`(게스트가 부른 호스트 메서드)은 jvm 프레임을 쌓지 않는다. 호스트가 만든 객체는 rustjava `instantiate_class` 가 «지금 맨 위 프레임»의 지역 변수에 넣는다. 게임 루프는 `run()` 한 호출 안에서 돌므로 그 프레임은 끝나지 않는다.
- 둘 다 «게스트가 쥔 참조를 수집기가 못 본다»는 한 문제의 다른 면이다. 둘 중 하나만 고치면 그 객체는 게스트가 아직 쓰는데 해제된다(be08 의 `Expected object, got Int` 와 같은 모양).

### 2. 처방

| 자리 | 바꾼 것 | 왜 그 자리 |
|---|---|---|
| ⒜ `interface.rs` `java_instantiate` | `jvm.shallow_clone` 으로 객체 집합에 등록하고 원본을 버린다. #445 의 전역 참조를 없앴다 | `instantiate_class` 도 등록하지만 jvm 쪽 클래스 초기화(`ensure_initialized`)를 함께 돌린다. LGT 는 초기화를 게스트 콜백(`java_initialize_class`)으로 따로 몬다. `shallow_clone` 은 초기화 없이 등록하는 유일한 공개 경로다 |
| ⒝ `method.rs` `JavaMethodProxy::call` · `interface.rs` `handle_java_system_svc` | 호출마다 `push_native_frame` / `pop_frame` | 호스트가 만든 객체는 그 호출 동안만 프레임에 묶인다. 게스트가 계속 쥐는 것은 ⒞가 본다 |
| ⒞ `jvm_support/guest_roots.rs` | `GuestRoots` — JVM 마다 전역 참조 하나. 수집기가 그 «필드»를 물을 때 스캔한다 | `jvm` 크레이트(crates.io)에는 수집 훅이 없다. 수집 지점은 여럿이다(midp `Display` paint · LGT 컨텍스트 자원 읽기 · rustjava `System.gc`/`Runtime.gc`). 지점마다 훅을 달면 하나만 빠져도 use-after-free 다. 전역 참조는 **모든** 수집이 반드시 지나는 자리다 |

**스캔 대상**(`ArmCore::guest_root_words` + LGT 쪽):
- 레지스터: 지금 엔진 값, 스레드마다 저장된 문맥, 그리고 `run_function` 이 중첩 호출 동안 비켜 둔 문맥. 마지막 것이 없으면 «호스트 메서드가 받은 인자»가 사라진다. 그 메서드가 게스트 코드를 부르면 r0~r3 이 덮이기 때문이다(시험 `guest_root_words_see_registers_a_nested_call_put_aside`).
- 스레드 스택: 위 문맥들의 SP 중 그 스택 안에서 가장 낮은 것부터 꼭대기까지. SP 아래는 죽은 값이라 읽지 않는다.
- 쓰기 가능 ELF 섹션(`.data`·`.bss`)과 예외 장부(스레드별 보류 예외 · try 프레임이 저장한 레지스터).

**매칭**: 인스턴스마다 머리 블록(12바이트)과 필드 저장 블록을 범위로 등록한다(끝 포함). 그래서 객체 포인터뿐 아니라 필드 저장 포인터·원소 주소·루프 끝 포인터도 루트가 된다. AOT 코드는 `ptr_fields` 나 원소 주소만 쥔 채 호출을 넘길 수 있다(시험 `a_guest_word_into_an_object_keeps_it_alive` 의 네 경우).
**방향**: 보수적이다. 객체처럼 보이는 죽은 값은 한 회 더 산다(과보존). 진짜 참조를 놓치는 것(과수거)만 위반이다.
**같이 고친 것**: WIPI `Display.callSerially(r, timeout)` 의 지연 작업이 `Runnable` 을 전역 참조 없이 들고 잤다. 지금까지는 ⒝의 프레임 고정 덕에 살아 있었다. 전역 참조로 바꿨다(rustjava `Thread.start` 와 같은 방식).
**하지 않은 것**: 스택 밖 힙의 게스트 네이티브 블록(malloc)은 훑지 않는다. LGT 자바 게임에서 그런 블록이 자바 객체의 유일한 참조인 경로를 찾지 못했다. 문자열 리터럴 캐시는 `nativeStrings` Vector 가 루트다. §3 스트레스에서도 위반 0이다.

### 3. 안전 — 스트레스 모드 (`wv` = 이 브랜치 release `wie_validate`)

`wie_validate --gc-stress N`: N번째 게스트→호스트 호출마다 수집한다. 수집이 해제한 블록은 `0xdeaddead` 로 채운다. 게스트가 호스트에 넘기는 참조가 산 인스턴스가 아니면 `dangling` 으로 센다. 놓친 루트는 두 길로 드러난다 — `0xdeaddead` 주소 접근 오류, 또는 `dangling` 계수.

- 처음 실행에서 수집기 자체의 조건 하나가 드러났다. jvm 부트스트랩 중(`Jvm::new` 가 부트스트랩 클래스의 `java/lang/Class` 를 만들기 전) 수집하면 rustjava `Class::java_class` 가 panic 한다. 제품 수집 지점은 모두 부팅 뒤라 해당하지 않는다. 스트레스 수집은 `GuestRoots` 를 설치한 뒤에만 한다.

| 실행 | 대상 | 결과 | 강제 수집 | dangling · `0xdeaddead` · panic |
|---|---|---|---|---|
| N=8 · 기본 27키 · 90초 | 라이브 LGT 5 + 가드 `ddd885583b15` | 6종 PASS · `4ece6eeeaa04` UNMEASURED(무스트레스와 같은 모양) | 2,254 ~ 52,264 | **0 · 0 · 0** |
| N=1(매 호출) · 90초 | 같은 6종 | 6종 PASS(매우 느림 · paints 1~1,606) | 21,705 ~ 90,032 | **0 · 0 · 0** |
| N=4 · 600초 · 대국 | `1b107b96bf4e`(§4 키) | UNMEASURED(키 일부 미전달) · paints 339 | 58,209 | **0 · 0 · 0** |
| N=4 · 600초 · 마을 | `a30bbe008b5e`(`docs/keys/battlemonster-village.keys`) | UNMEASURED · paints 870 | 166,726 | **0 · 0 · 0** |
| N=1 · `keydraw_lgt` `--inject --expect-last-frame` | 고정물 | PASS · paints 55 | 5,006 | **0 · 0 · 0** |

KTF 가드 `49ade89578c5` 는 스트레스 대상이 아니다(수집 0 · LGT 전용). 표는 증적 `stress-*.txt`.

### 4. 누수 장면 — 전/후(같은 장면 · 같은 계측)

- **재현 경로**: `docs/keys/lgt-1b107b96bf4e-match.keys`(공지 → 제목 → 메뉴 1 → 채널 → 대국 → 카드 내기 반복). 0418 의 60분 «후» 실행이 장면에 못 들어간 이유는 §5 다.
- **계측**: 수집마다 그 순간의 LGT 인스턴스 수(해제 전) — `RUST_LOG=wie_lgt::runtime::java::jvm_support::guest_roots=debug` 의 `guest roots R of N instances` 줄. 커밋된 디버그 줄이다.
- **전**: 이 브랜치에서 ⒜⒝만 되돌린 빌드(전역 참조 고정 + 프레임 없음 · 스캔과 계측은 그대로 · 커밋 안 함). 대국 10분: **1,120 → 28,431** 로 단조 증가했다. 0418 LIVEPROBE 의 1.7k → 61k(2.5분 · 다른 장면)와 같은 모양이다.
- **후**: 60분(`--keep-timeout --timeout 3600` · load1 20~40): 수집 3,926회 · 인스턴스 최소 849 · 중앙 2,384 · p95 2,562 · 최대 3,904. 10분 구간 평균은 위와 같다. 대국은 ROUND 13 → 34 로 계속 진행했다(증적 `after3600-sheet.png`). OOM 0 · panic 0 · Java 예외 6(전과 같다). 결과는 UNMEASURED 다 — 부하로 키 1,818 중 1,652 만 전달됐다. 판정과 무관하다.
- 곡선 `curve.png` · 원자료 `old600-curve.csv` · `after3600-curve.csv`.

### 5. «Loading» 정체 — r3 퇴행이 아니다

0418 의 60분 «후» 실행은 채널 → 대국 소개(«Loading · Press Any Key»)에서 50분 동안 멈췄다. 같은 키(§4 머리)로 빌드 셋을 돌렸다.

| 빌드 | 정체 / 실행 |
|---|---|
| main(`4ac38566` · r3 전) | **1 / 5** |
| r3(#452 빌드) | 0 / 1 (+ 0418 60분 1 / 1) |
| 이 브랜치 | 0 / 2 |
| 이 브랜치 ⒜⒝ 되돌림 | 3 / 4 (2회는 t090·t120 프레임이 같아 자동 중단한 판정) |

⇒ **main 에서도 난다 — r3 퇴행이 아니다.** 정체하면 키가 닿아도 화면이 바뀌지 않고 paint 와 paint 수집이 멈춘다. 결과 JSON 이 남은 정체 실행 둘(main 1 · 0418 60분)은 Java 예외가 하나 더(7) 기록됐다(통과 실행은 모두 6). 원인은 재지 않았다 → 후속.

### 6. 퇴행 · 속도

- 라이브 LGT 5 + 가드 2(기본 27키 · 30초 · main ↔ 이 브랜치 · 교대 실행): 결과·`stop`·`content`·Java 예외 수가 7종 모두 같다. paints 도 같은 범위다(`13d7e3c21856` 160 ↔ 202 · `1b107b96bf4e` 142 ↔ 128 · 나머지 ±10). `49ade89578c5` 는 main 쪽이 max-ticks 로 굶었다(81 ↔ 1,039) — LGT 변경과 무관한 KTF 이고 아래 속도 짝에서 같은 범위다.
- 속도(`--pacing 8` · 60초 창 · 교대 2회 · load1 20~45):

| sha12 | main | 이 브랜치 |
|---|---|---|
| `a30bbe008b5e` paints/s | 19.2 · 19.4 | 18.2 · 19.3 |
| `a30bbe008b5e` GC ms(60초) | 255 · 218 | 167 · 125 |
| `1b107b96bf4e` paints/s | 18.7 · 18.8 | 18.7 · 18.7 |
| `1b107b96bf4e` GC ms(60초) | 484 · 463 | 135 · 130 |
| `ddd885583b15` paints/68초 | 3,692 · 4,062 | 3,910 · 3,853 |
| `49ade89578c5`(KTF) paints/68초 | 1,955 · 2,729 | 2,312 · 2,772 |

  스캔 비용으로 느려진 것은 없다. 수집 시간은 오히려 준다 — 살아 있는 객체가 적어 표시할 것이 적다. `ddd885583b15`·`49ade89578c5` 는 `pacing.paints` 가 0이라(midp `Display` 밖에서 그린다) 실행 전체 paints 로 쟀다.

### 7. KTF — 설계만

KTF 는 ⒝만 있다. `java_new` 는 `instantiate_class` 라 객체 집합에 들어가지만, 게스트→호스트 호출(`wie-ktf` `method.rs` 의 프록시)이 프레임을 쌓지 않아 바깥 프레임에 묶인다. 같은 훅을 붙이려면 ① KTF 인스턴스 블록 장부(`class_instance.rs` instantiate/destroy) ② 프록시·SVC 프레임 ③ KTF 용 `GuestRoots`(KTF 예외·컨텍스트 구조 + 이미지 쓰기 영역)가 든다. `ArmCore::guest_root_words` 는 이미 공용이다. KTF 누수를 잰 실측이 아직 없고 가드(`49ade89578c5`)가 KTF 라서 이 회차에서 넣지 않았다 — 먼저 KTF 장시간 인스턴스 곡선을 재야 한다.

### 8. 게이트

- `cargo fmt --check` · `cargo clippy --all -D warnings`(stable · beta) · `cargo clippy --target wasm32-unknown-unknown -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` **663 pass / 0 fail**(`CARGO_INCREMENTAL=0` — 첫 실행은 증분 캐시 파일이 사라져 rustc ICE) · `npm run build:wasm` rc=0 · `check-engine-contract` OK · `npm run audit` PASSED.
- 러너 블록(release `wie_validate` · 이 브랜치): draw · helloworld ×2 · text PASS. keydraw ×2 `--inject --expect-last-frame` 는 load 40 에서 max-ticks 로 UNMEASURED(main 도 같음) → `--max-ticks 100000000000` 로 PASS · rc=0(paints 79 · 55 — main 과 같음).
- **되돌리면 red**(증적 `revert-*.log`): ⒜ 되돌림 → `a_guest_new_nothing_points_at_is_collected_with_what_it_held` · ⒝ → `what_a_host_method_made_for_the_guest_is_collected_once_dropped` · 스캔 끔 → `a_guest_word_into_an_object_keeps_it_alive`(과수거) · 중첩 문맥 장부 끔 → `guest_root_words_see_registers_a_nested_call_put_aside` · 스택 스캔 끔 → `guest_root_words_see_a_thread_stack_from_its_sp_up`. be08 시험(`a_collection_keeps_what_a_guest_new_allocated`)은 이제 게스트 레지스터로 Stack 을 쥔다.
- **기준선**: 측정 바이너리는 `cad17188`(#451) 위 이 브랜치다. 그 뒤 #452(0418 r3)가 먼저 착지해 그 위로 rebase 했다(충돌 0). #452 가 LGT 에서 바꾼 것은 힙 고갈 때의 오류 경로와 ABI 행 하나라 위 측정 경로와 겹치지 않는다. 시험·clippy 는 rebase 한 트리에서 다시 돌렸다.
- 측정 규율: emulator 실행은 전부 `build-slot run` 1개씩 · `nohup &` 없음 · 끝에 내 프로세스 0. 시작 때 `host-load-guard --status --recovered` 는 rc=1(load 81 · idle 0)이었다 — 폭을 1로 두고 진행했다. census 락은 쓰지 않았다(전수 아님).

### 9. 후속

| 군집 | 수 | 계급 | 크기 |
|---|---|---|---|
| `1b107b96bf4e` 대국 소개 간헐 정체(§5) | 1 | ⒝ · worklog p0 | M |
| `1b107b96bf4e` 채널 «3.전적» → `Unimplemented: java/lang/String vtable index 29` 로 게임 스레드 사망(main 실측) | 1 | ⒝ · worklog p1 | S |
| KTF 같은 훅(§7) — 먼저 KTF 장시간 곡선 측정 | — | 측정 | M |

### 10. 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 78회 / 34쌍 · SUFFIX-ATTACHED 5회 / 3쌍. 전부 이 회차가 고친 파일에 **이미 있던** 줄(`init.rs`·`exception.rs`·`interface.rs`·`jvm_support.rs`·`wie_validate.rs` 의 기존 주석·시험 행)이다. SUFFIX-ATTACHED 3쌍은 더 긴 다른 제목이다. 이 회차가 더한 줄의 게임 이름은 0이다(`git diff origin/main...HEAD` 의 `+` 줄 대조 · 초안의 한 곳은 sha12 로 바꿨다).

<!-- corpus-name-inflow v1 subjects=20 tree=2478424e4de0f980 B=78/34 P=9/2 S=5/3 -->
