## [2026-10-04] KTF 게스트 GC 루트 — LGT #454 이식 (wie-ktf-guest-gc-roots-port-from-lgt)

**무엇을**: KTF 의 게스트→호스트 호출마다 jvm 프레임을 쌓고, 게스트가 레지스터·스택·이미지에만 쥔 포인터는 0421 의 보수적 스캔으로 살린다. 스캔·인스턴스 장부·`--gc-stress` 는 LGT 안에 있던 것을 `wie-jvm-support::guest_roots` 로 옮겨 두 통신사가 같은 코드를 쓴다.
**판정**: `8d8c24b7c198` 600초 — 이 브랜치는 3회 모두 살아남는다(게임 스레드 사망 0 · 꼬리 동결 0). 프레임 지역 수는 600초 내내 **2,723 근처에서 평탄**하다. 같은 계측에서 프레임만 뺀 빌드는 **3,278,186** 까지 오르고 478초에 8바이트 버킷 524,288칸을 다 써서 죽는다(이분 탐색 회신이 잰 그 모양). KTF 전수 짝 결과는 §5, 스트레스는 §6.
**사용자 영향**: 게임 루프를 도는 KTF 자바 게임을 오래 해도 메모리가 차지 않는다. `8d8c24b7c198` 은 약 7~8분이면 게임 스레드가 죽어 화면이 굳었다.

증적: `~/orchestrator/reports/evidence/wie-ktf-guest-gc-roots-port-from-lgt/`. 타이틀은 sha12 로만 적는다.

### 1. 원인 (이분 탐색 회신 `reports/wie-longplay-regression-8d8c24b7c198-bisect.done.md` 그대로)

`JavaMethodProxy::call`(게스트가 부른 호스트 메서드)이 프레임을 쌓지 않는다. 그 안의 `new_class`·`instantiate_array`·`invoke_*` 반환 참조는 rustjava 가 «지금 맨 위 프레임» 지역 변수에 넣는다. 게임 루프는 영원히 돌아오지 않는 호스트 호출 안에서 돌므로 그 프레임의 지역 변수는 줄지 않는다. KTF 의 게스트 `new`(`java_new` · `java_array_new`)도 `instantiate_class` 라 같은 자리에 쌓인다.

### 2. 처방

| 자리 | 바꾼 것 |
|---|---|
| `wie-ktf` `method.rs` `JavaMethodProxy::call` | 호출마다 `push_native_frame` / `pop_frame` |
| `wie-ktf` `init.rs` `handle_init_svc` | `JavaThrow` · `JavaNew` · `JavaArrayNew` 만 프레임으로 감싼다. `GetInterface`·`Alloc` 은 C 쪽에서도 불려 jvm 스레드가 아닐 수 있어 건드리지 않았다 |
| `wie-ktf` `interface.rs` `handle_java_interface_svc` | 전부 프레임으로 감싼다(LGT `handle_java_system_svc` 와 같다). `register_java_string` 이 부를 때마다 만드는 문자열이 여기 쌓였다 |
| `wie-ktf` `class_instance.rs` | 인스턴스 생성·해제 한 곳(`instantiate` · `destroy`)에서 블록 장부 등록·해제 · 스트레스 독 채우기 |
| `wie-ktf` `init.rs` `load_native` | 이미지 전체(`data + bss`)를 스캔 영역으로 등록. KTF client.bin 은 섹션 표가 없어 코드·데이터를 가를 수 없다 — 코드 낱말이 포인터처럼 보이면 그 객체가 한 회 더 산다(과보존 방향) |
| `wie-ktf` `jvm_support/guest_roots.rs` | KTF 쪽 어댑터 — 레지스터·스택(`ArmCore::guest_root_words`)과 인스턴스 해독 |
| `wie-ktf` `emulator.rs` `Drop` | `guest_roots::forget` — 해제된 코어 id 를 다음 코어가 물려받지 않게 |

**공용화**: 0421 의 `wie-lgt/.../guest_roots.rs` 에서 통신사와 무관한 부분(블록 장부 · 영역 · `GuestRoots` 전역 참조 · 스캔 · 스트레스 모드)을 `wie-jvm-support/src/guest_roots.rs` 로 옮겼다. 통신사는 `GuestMemory` 트레잇(코어 id · 루트 낱말 · 인스턴스 해독) 하나만 구현한다. `wie-jvm-support` 는 `wie-core-arm` 에 의존하지 않는다(`docs/architecture.md` «JVM 과 ARM 층은 브리지 코드로만 잇는다») — 그래서 코어 대신 id 와 트레잇을 받는다. LGT 동작에서 바뀐 것은 하나다: 스캔이 걸러 내는 범위를 고정 상수(`0x4000_0000..0x5000_0000`) 대신 «장부의 첫 블록 시작 ~ 마지막 블록 끝»으로 잡는다. 같은 블록을 찾으므로 결과는 같고, 힙 상수를 jvm 층이 알 필요가 없다. LGT 시험(0421 의 되돌리면 red 3종 포함)은 그대로 통과한다.

**하지 않은 것**: 게스트 malloc 블록(`Alloc` SVC · WIPI-C `MC_knlAlloc`)은 훑지 않는다 — LGT 와 같은 한계(`guest_roots.rs` 머리 `ponytail:`). §5·§6 에서 그런 경로로 객체를 잃은 타이틀은 나오지 않았다. 예외 처리기(`JavaExceptionHandler.unk3` 에 쓰는 예외 객체)는 따로 등록하지 않았다 — 처리기는 try 블록의 지역 구조라 스택 스캔이 본다. 스트레스에서도 `dangling` 0 이다.

### 3. `8d8c24b7c198` 600초 (이분 탐색 탐침과 같은 인자 · `--keys long.keys --keep-timeout --timeout 600 --relaunch 1`)

| 실행 | 결과 | paints | 게임 스레드 사망 · `Allocation failure` | 꼬리(t440~t580 샷 md5) |
|---|---|---|---|---|
| 이 브랜치 1 | UNMEASURED(854/900 키 · deadline) | 86,076 | 0 · 0 | 8장 모두 다름 |
| 이 브랜치 2 | 같음 | 61,786 | 0 · 0 | 모두 다름 |
| 이 브랜치 3 | 같음 | 76,431 | 0 · 0 | 모두 다름 |
| 이 브랜치 + 지역 계측 | 같음 | 86,286 | 0 · 0 | — |
| 프레임만 뺀 빌드 + 지역 계측 | **FAIL 478.3초** | 47,659 | `buckets 8:524288/524288` → `uncaught exception in thread` | — |
| main(`739ee854`) | UNMEASURED | 42,458 | 0 · 0 | 모두 다름 |

- UNMEASURED 는 키 900개 중 854개만 닿은 것이다(이분 탐색의 «생존» 실행도 같은 854/900). 판정과 무관하다.
- main 이 이번 600초를 버틴 것은 부하(load1 40~60) 아래 일을 덜 했기 때문이다 — paints 42,458 은 이분 탐색이 잰 OOM 시점(445.7초 × 98.9 paints/s ≈ 44,100 · 492.6초 × 95.0 ≈ 46,800)에 못 미친다. 이분 탐색에서는 600초 실행 전부(«생존»으로 집계된 것 포함)가 그 시점에 힙이 찼다. 이 브랜치는 그 두 배에 가까운 86,076 paints 를 넘겼다.

**인스턴스 곡선**(수집마다 KTF 인스턴스 수 · `RUST_LOG=wie_jvm_support::guest_roots=debug` · 1분 구간 중앙):

| 분 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 |
|---|---|---|---|---|---|---|---|---|---|---|
| 이 브랜치 2 | 5,103 | 10,135 | 9,826 | 9,863 | 7,870 | 7,340 | 7,592 | 8,061 | 7,646 | 7,273 |
| 이 브랜치 3 | 5,109 | 9,844 | 7,488 | 8,512 | 9,015 | 10,051 | 10,150 | 10,269 | 10,212 | 10,205 |
| 프레임만 뺀 빌드 | 9,831 | 36,279 | 55,935 | 82,879 | 121,633 | 163,586 | 168,026 | 197,666 | 217,449 | 233,217 |

**프레임 지역 수**(이분 탐색과 같은 방법 — 스크래치 `[patch.crates-io] jvm` 이 수집마다 스레드별 지역 수를 찍는다 · 커밋 안 함): 이 브랜치는 전 스레드 합이 600초 내내 2,719~2,728 이다. 프레임만 뺀 빌드는 454,973(1분) → 1,473,132(3분) → 2,962,926(7분) → 3,278,186(OOM 직전)이다.

### 4. 되돌리면 red

`wie-ktf` 시험 셋(증적 `revert-*.log`):
- 프록시 프레임 제거 → `test_what_a_host_method_made_for_the_guest_is_collected_once_dropped`
- init SVC 프레임 제거 → `test_a_guest_new_the_guest_dropped_is_collected`
- 스캔 제거(`guest_roots::install` 빼기) → `test_a_guest_word_into_an_object_keeps_it_alive`(과수거 — 객체 · 필드 저장 · 원소 · 저장 끝 네 경우)

각 되돌림은 자기 시험 하나만 red 로 만든다. 인터페이스 SVC 프레임은 시험이 없다 — 거기서 만들어지는 것(문자열 등록)은 `nativeStrings` Vector 가 붙드는 것이라 «수거된다»로 관측할 수 없고, 쌓이는 것은 지역 변수 항목뿐이다(§3 계측이 그 축을 잰다).

### 5. KTF 전수 짝 (기본 27키 · `--inject` · main ↔ 이 브랜치 · 타이틀마다 순서 교대 · `build-slot` · 동시 ≤3)

대상: `game_lab/working/ktf` 190 파일 = **sha 170종**(같은 바이트가 이름만 다른 20쌍). 타이틀마다 main · 이 브랜치 · 이 브랜치 `--gc-stress 8` 셋을 차례로 돌렸다. 같은 sha 의 두 파일이 동시에 돌아 결과 파일을 서로 덮은 20종과, 1차에서 갈린 타이틀 25종은 `--max-ticks` 를 풀고 다시 쟀다(확인 회차 45종). 증적 `sweep.tsv`(sha12 만).

| | main | 이 브랜치 |
|---|---|---|
| PASS | 96 | 97 |
| UNMEASURED | 74 | 73 |
| FAIL | 0 | 0 |
| 나빠짐(FAIL 로 · 내용 잃음 · Java 예외 증가) | — | **1** (`3d38a46becd9` · 아래) |
| paints 비(브랜치/main) | — | 중앙 1.000 · p10 0.939 · p90 1.059 |

- UNMEASURED 는 거의 전부 `max-ticks`(부하 load1 40~60 아래 27키 전에 틱 상한)다. 1차에서 결과가 갈린 7종은 양방향이었고(main 쪽이 나은 것 2 · 브랜치 쪽이 나은 것 5), 상한을 풀고 다시 재면 7종 모두 같은 결과다.
- **`3d38a46becd9`**: 확인 회차에서 main 은 예외 0 · 브랜치는 `NullPointerException: image is null` 3건이었다. 같은 조건으로 더 쟀다 — main 은 22회 중 10회가 «paints 132 · 예외 0», 12회가 «paints 82 · 예외 3~4»이고, 브랜치는 19회 모두 후자다. 갈리는 지점은 부팅 직후다: 로딩 막대에서 메뉴로 스스로 넘어가느냐, 키를 기다리느냐(후자에서 첫 paint 가 아직 없는 이미지를 그리려다 NPE 를 잡고 넘어간다). 두 모드 모두 게임 화면(«GAME OVER»·메뉴)까지 간다.
  **수거 문제가 아니다** — ⑴ 스캔이 모든 인스턴스를 살리게 한 빌드(아무것도 해제하지 않는다)도 5회 모두 후자다 ⑵ 프레임 없이 장부·스캔만 켠 빌드도 3회 모두 후자다 ⑶ 스트레스(8호출마다 수집 · 해제 블록 독 채우기)에서는 오히려 예외 0 · `dangling` 0 이다 ⑷ 코덱의 «산 인스턴스 아님» 경고가 0 이다 — `image` 는 해제된 참조가 아니라 진짜 null 이다. main 에도 있는 부팅 경합이 장부·스캔 비용(부팅 20초 수집 18회에 main 37~44ms → 57~58ms)으로 한쪽에 기운 것이다.

**그 밖에 본 것**(나빠짐이 아닌 것):
- `65ef7052f528` 은 1차에서 브랜치만 22번째 키에서 `Method setEventListener(Lorg/kwis/msp/lwc/EventListener;Ljava/lang/Object;)V not found from org/kwis/msp/lwc/Component` 로 끝났다. 다시 재면 main·브랜치·스트레스 모두 PASS(각 2회 · 1회). 그 메서드는 `wie-wipi-java` 어디에도 없다 — 키가 그 화면에 닿으면 빌드와 무관하게 난다. 같은 모양이 스트레스의 `0c67145b11df`(`FormComponent.setFocus(Component)` 없음)다. → worklog 제안 1건.

### 6. 스트레스 (`--gc-stress N`)

- **전수 N=8**(§5 와 같은 170종 · 기본 27키 · 확인 회차는 `--keep-timeout --timeout 90`): 강제 수집 1,073,574회(타이틀당 816 ~ 26,704 · 중앙 4,618) · **`dangling` 0 · `0xdeaddead` 0 · panic 0**. 결과는 PASS 143 · UNMEASURED 26(`clean exit` 16 · `max-ticks` 10) · FAIL 1(`0c67145b11df` — 위 미구현 메서드). `clean exit` 은 키가 «종료» 메뉴에 닿은 게임 자신의 종료이고, 같은 일이 main·브랜치에서도 양방향으로 나온다(스트레스가 느려 키가 다른 화면에 닿는다).
- 1차(기본 마감 ~20초)에서 FAIL `no frame rendered` 였던 17종은 스트레스가 느려 첫 화면을 못 그린 것이다 — 90초로 다시 재면 모두 그린다.
- **`keydraw_ktf` N=1**(매 호출 수집): PASS · paints 79 · 수집 6,479 · `dangling` 0.
- **`8d8c24b7c198` N=4 · 600초**(§3 과 같은 키): UNMEASURED(853/900 키 · deadline) · paints 2,143 · 강제 수집 104,338 · `dangling` 0 · `0xdeaddead`·panic·게임 스레드 사망 0.

### 7. 게이트

- `cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0(beta `1.100.0-beta.3`, `rustup toolchain install beta` 로 갱신 확인) · `cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0 **684 pass / 0 fail**(`CARGO_INCREMENTAL=0`).
- 러너 블록(debug `cargo run`): draw · helloworld ×2 · text PASS · keydraw ×2 `--inject --expect-last-frame` PASS · rc=0(paints 79 · 55).
- `npm run build:wasm` rc=0 · `check-engine-contract` 113 pass / 0 violation · `npm run audit` PASSED.
- 측정 바이너리는 이 브랜치 첫 커밋 직전 트리다. 그 뒤 바뀐 것은 `roots()` 의 범위 계산 한 줄(블록이 하나뿐일 때 빈 집합을 내던 것)뿐이다 — 인스턴스마다 블록이 둘이라 측정 경로에서는 같은 값이다.
- 측정 규율: 착수 때 `host-load-guard --status --recovered` rc=0(load 5.7). 에뮬레이터 실행은 전부 `build-slot run` · 동시 ≤3 · `nohup` 없음 · 끝에 내 프로세스 0. 인공 부하 없음(load1 40~60 은 다른 레인).

### 8. 후속

없음 — 아래 회신에 적은 한계(게스트 malloc 블록)는 관측된 결함이 없다.

### 9. 유입

`node scripts/corpus-name-inflow.mjs`: BOUNDED 64회 / 31쌍 · SUFFIX-ATTACHED 5회 / 3쌍. 전부 이 회차가 고친 파일에 **이미 있던** 줄(`wie-ktf` `interface.rs`·`jvm_support.rs`, `wie-lgt` `interface.rs`·`jvm_support.rs` 등의 기존 주석·시험 행)이다. SUFFIX-ATTACHED 3쌍은 더 긴 다른 제목이다. 이 회차가 더한 줄의 게임 이름은 0이다(`git diff origin/main...HEAD` 의 `+` 줄 대조).

<!-- corpus-name-inflow v1 subjects=26 tree=54f4bfcc6df386da B=64/31 P=10/2 S=5/3 -->
