## [2026-10-04] LGT ABI 2행 — String `concat` 29 · Vector `lastElement` 25, 둘 다 호출부 역어셈 (wie-vtable-unimplemented-string29-stack25)

무엇을: `wie-lgt/data/lgt_java_abi.toml` 에 두 행을 넣었다. `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 같은 두 행을 단언했다.
왜: `1b107b96bf4e` 는 채널 화면에서 NUM3(전적)을 누르면 게임 스레드가 죽었다(0421 §9). `be08d047cbae` 는 보조 스레드가 18초마다 죽었다(0425 §4).
사용자 영향: 앞 게임은 전적 화면이 열린다. 뒤 게임은 보조 스레드가 죽지 않고 대화를 지나 게임 안 상점 화면까지 간다.

### 1. 주소 읽는 법

LGT `binary.mod` 는 ELF32 다. `objdump -d --triple=armv5te`(Thumb 이미지는 `thumbv5te`)로 바로 읽힌다. 경고 줄의 `lr` 이 objdump 주소와 같다. 파일 오프셋으로 읽으면 0x1000 이 아니라 **`lr - 0x1000 + 0x34`**(ELF 헤더 0x34 바이트)다. 0425 §4 의 «lr `0x8c168` · 파일 `0x8b168..0x8b198`» 는 그 0x34 를 빼고 읽어 호출 지점이 lr 보다 뒤에 있는 것처럼 적혔다. 결론(replace 30)은 그대로다 — 범위 끝이 실제 반환 지점이다.

### 2. ① `1b107b96bf4e` — `java/lang/String` 29 = `concat(String)`

- 재현(main release): 채널 목록이 뜬 뒤(키 13번째) NUM3. 경고 `java/lang/String vtable index 29: this=0x488495d0 r1=0x48849750 r2=0x488495d0 lr=0x5d7a9` → `FAIL · stop error`.
- 호출부(Thumb, 반환 `0x5d7a8` = `0x5d7a4` 의 `bl`): r0 = 필드에서 읽은 String. r1 = 바로 앞 slot-4(`toString`) 디스패치의 결과. 결과는 **같은 필드에 되쓴다** — `f = f.<29>(x.toString())`. `[r3,#0x78]` 디스패치는 이미지에 5곳(`0x5d7a2` `0x5d8d8` `0x5d9d6` `0x5dad4` `0x5df9a`)이고, 다섯 모두 같은 모양이다.
- String 인자 하나에 String 반환이다. CLDC 순서(28 substring(II) · 29 concat · 30 replace)에서 그 모양은 concat 뿐이다. 양옆 28·30 은 이미 측정된 행이다.

### 3. ② `be08d047cbae` — `java/util/Stack` 25 = Vector `lastElement()`

- 호출부(ARM, `lr 0x1388e0`, 함수 `0x138854`): Stack 을 `this.<field>+8 -> +4` 에서 읽고 null 검사한다. `[r3,#0x40]`(15 · size)를 부르고 `> 0` 일 때만 계속한다. 같은 Stack 을 다시 읽어 **r0 만** 두고 `ldr ip,[r3,#0x68]`(= 4 + 4×25)를 부른다. r1 = `0x32` 는 몇 줄 위 `mov r1,#50`(필드 저장값)의 잔여이고 인자가 아니다. 결과는 null 검사 → 게스트 클래스 비교 → 그 객체의 메서드 호출이다: `if (s.size() > 0) { o = s.<25>(); … }`.
- 고정된 firstElement 24 와 removeElementAt 27 사이에서 인자 없는 객체 반환은 lastElement 뿐이다(setElementAt 26 은 인자 둘·void). 행은 Vector 에 넣었다 — Stack 은 상속한다.

### 4. 전/후

| 대상 | 조건 | 전(main `0e9fbaeb`) | 후 |
|---|---|---|---|
| ① `1b107b96bf4e` | 채널 NUM3 · 60초 | FAIL · stop error · String 29 1회 | **PASS 2/2** · 전적(Record) 화면 표시 · Java 예외 6 → 6 |
| ② `be08d047cbae` | 진도 v2 인자 그대로 · 420초(300 + 재시작 120) | Stack 25 uncaught **12**(전부 `lr 0x1388e0` · 약 18초 간격) · paints 2,357 | uncaught **0** · paints 4,150 · Java 예외 16 → 16 |

- ② 화면: 전은 대화 뒤 일시정지 메뉴로 되돌아가고 마지막 프레임 일부가 빈다. 후는 «진짜 게임을 시작해 볼까?» 대화를 지나 아이템 상점(1/12)까지 간다. 그 상점에 180초부터 290초까지 머문다 — 원인은 재지 않았다(§6).
- ② 후 실행은 `cargo test --all` 과 시간이 겹쳤다. uncaught 수는 부하와 무관한 결정적 사망이라 판정에 영향이 없다. paints 는 부하 때문에 오히려 불리한 쪽이다.

### 5. 퇴행

- A 프로브(기본 27키 · 30초 · `--relaunch 1 --pacing 8` · 전·후 교대): 라이브 LGT 5(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) + 가드 2(`49ade89578c5` `ddd885583b15`) + `73f3a21e981c`. **8/8 짝** · FAIL 0 → 0 · content 8 → 8 · Java 예외 수 8종 모두 같다. 판정이 다른 행 둘(`b475b6399684` stop · `73f3a21e981c` UNMEASURED→PASS)은 둘 다 `max-ticks` 굶음이다 — 측정 중 가드는 `holding`(idle ≈0 · load 21)이었다.
- 되돌리면 red: concat 행 제거 · lastElement 행 제거 → 각각 `abi_rows_cover_the_indexes_titles_actually_dispatch_on` FAILED(2/2).

### 6. 게이트 · 측정 조건

- `cargo fmt --check` · `cargo clippy --all -D warnings`(stable · beta) · `cargo clippy --target wasm32-unknown-unknown -D warnings` · `RUST_MIN_STACK=4194304 cargo test --all` **672 pass / 0 fail**(`CARGO_INCREMENTAL=0` — 첫 실행은 증분 캐시로 rustc ICE, 0421 §8 과 같다).
- 러너 블록(후 release): draw · helloworld ×2 · text PASS. keydraw ×2 `--inject --expect-last-frame` 는 `--max-ticks 100000000000` 로 PASS · rc=0(paints 79 · 55 — 0421 과 같다).
- 에뮬레이터는 전부 `build-slot run` 1잡씩 · `nohup &` 없음 · 끝에 내 프로세스 0. census 락은 쓰지 않았다(전수 아님 · 진도 v2 인자를 `wie_validate` 에 그대로 줬다). 시작 때 `host-load-guard --status --recovered` rc=0.
- RustJava(`5b84dd1`)는 `String.concat`·`Vector.lastElement` 를 이미 갖고 있다 — upstream 변경 없음.

### 7. 후속

- `be08d047cbae` 아이템 상점 체류(§4) — 진도 v2 키로 나가지 못하는지, 게임 쪽 대기인지 먼저 잰다. 이번 회차 관측 1판뿐이라 제안 카드로 올리지 않았다.
- compat.json 은 바꾸지 않았다(진도 축은 600초 정의 · 이번 측정은 420초 1판).

### 8. 게임 이름 유입

`corpus-name-inflow`: BOUNDED 28쌍 · SUFFIX-ATTACHED 4쌍. 전부 이 회차가 손댄 두 파일(`lgt_java_abi.toml` · `jvm_support.rs`)에 **원래 있던** 주석이다. 이 회차가 더한 줄(`git diff origin/main...HEAD` 의 `+` 줄)은 코퍼스 stem 451개와 겹침 **0**이다 — 새 주석은 sha 앞 12자만 쓴다.
