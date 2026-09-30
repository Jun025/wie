## [2026-09-30] 3차 군집 — LGT Thread 폰 배치 · 호스트 panic → Java 예외 · SKT MIDlet 표시 필드 (wie-census-wave3-lgt-runtime-robustness)

**무엇을**: 0380 §4 의 두 군집(LGT JVM 지원 층 panic 4 · `class_instance` unwrap panic 5)을 재현해 원인 계급을 가렸다. 셋째 군집(LGT 미식별 SVC 8)은 #419(`wie-2026-09-29-lgt-wipic-svc-eight-unknown-ids-adopt-p0`)가 이미 열려 있어 손대지 않았다.
**왜**: 호스트 panic 은 탭 전체가 죽는다. Java 예외면 게임이 잡을 수 있고, 못 잡아도 오류 문구가 남는다.
**사용자 영향**: 9종 중 5종이 30초 조작 3/3 PASS 로 바뀌었다(2종은 0380 의 «bucket.rs» 군집이던 `517ed32c92d6` 포함). 나머지 4종은 panic 이 사라지고 다음 벽이 보인다.

### 1. 재현 — 원인은 «해제된 인스턴스»가 아니었다

전 = `origin/main` + 진단 로그 1줄(`wv-detect`) · 30초 `--inject --pacing 8 --relaunch 1` · 3판씩 · load1 110~330.

| sha12 | 첫 벽(전) | 재현 | 원인 계급 |
|---|---|---|---|
| `a16f08d025eb` `2a57e33133b5` | LGT GC 가 `class 0x0` 이름 읽기 panic | 3/3 · 2/3 | **호스트가 인스턴스 밖에 씀** — 아래 §2 |
| `517ed32c92d6` | `allocator/bucket.rs` 범위 밖 panic(0380 의 다른 군집) | 2/3 | 같은 원인 |
| `85e94babc247` | `Clip(String, byte[])` 의 `array_length(null)` | 3/3 | 호스트가 null 을 역참조 |
| `c7f543c73b91` | WIPI `Graphics.setFont(null)` | 2/3 | 호스트가 null 을 역참조 |
| `0262a4fe3389` | SKT `Toolkit.<clinit>` 에서 `getDisplay` 가 null | 3/3 | **저장 칸 공유** — §3 |
| `1eaa92092bee` `fe76e641bb3d` | LGT `JavaException` unwrap(부모 클래스 해석 · 리소스 읽기) | 1/3 · 3/3 | 안쪽은 **힙 목록 헤더 손상** — §4 |
| `0cc4ef7ede37` `249e655147a1` | 없음 | 0/3 | 30초 창에서 재현 안 됨(0380 에서는 장시간 1판) |

### 2. LGT `java/lang/Thread` 는 폰에서 4단어다

- 탐침(스크래치 · 미커밋): 인스턴스 헤더 첫 단어를 0 으로 만드는 쓰기를 잡았다 — `Allocator::free` 는 한 번도 그 주소를 풀지 않았고, 쓴 쪽은 rustjava `Thread.<init>` 의 `put_field(daemon)` 이었다.
- `HOSSXNet`(`a16f`) · `atdata/a`(`2a57`) 는 `Thread` 하위 클래스이고 선언 필드 0 · 전체 **4단어** · 참조 비트맵이 **2·3번 단어**를 가리킨다. rustjava 의 `Thread` 는 `id J`(0–1) · `target`(2) · `name`(3) 뒤에 `priority`·`interrupted`·`started`·`alive`·`daemon` 을 4–8번에 둔다 ⇒ **인스턴스 밖 5단어**에 썼다.
- 처방: `data/lgt_java_abi.toml` 의 `java/lang/Thread` 에 그 5개를 `host_field` 로(TimerTask 0296 과 같은 모양). 참조 둘은 인스턴스에 남아 GC 가 본다.
- 전수: `prepare_generated` 가 «호스트 상위 클래스가 폰 배치보다 크다»를 `tracing::error` 로 말하게 했고, LGT 79종을 30초씩 돌렸다 — 걸린 것은 **`Thread` 3종뿐**(`a16f` `2a57` base 4 · `517ed` base 7). `517ed` 는 숨은 자기 단어 3개가 `priority` 등에 덮였고, 그것이 0380 의 «bucket.rs 범위 밖» 군집의 한 타이틀이었다. 짝 `6bc7f65e3022` 는 전·후 모두 30초 안에 재현되지 않았다.
- 시험 `thread_keeps_the_phone_layout` — 행을 지우면 red.

### 3. SKT `MIDlet` 의 표시 필드 이름

- 0382 ⚠ 가 적은 그대로다: `jvm-bytecode` 는 인스턴스 저장을 (이름, 서술자, 플래그)로 잡아, 게임의 private `display` 와 `MIDlet` 의 private `display` 가 **한 칸**이다. `0262` 의 생성자가 자기 `display` 를 null 로 두자 MIDlet 쪽도 null 이 됐다.
- 처방: 네이티브 필드 이름을 `wieDisplay` 로. 게임은 `MIDlet` 의 private 필드를 쓸 수 없으므로 이름은 계약이 아니다. 기존 시험의 가리는 필드를 **private** 로 바꿔(0382 가 피해 간 경우) 이름을 되돌리면 red.
- 결과: `0262` 부팅 panic 3/3 → PASS 3/3.

### 4. `1eaa` · `fe76` 의 안쪽 — 화면 버퍼 끝을 넘는 쓰기(못 고쳤다)

- panic 은 사라졌다(부모 해석 실패는 그 Java 예외를 되던지고, 리소스 경로는 `WieError`). 대신 `net/wie/WieError: Allocation failure` 로 끝난다.
- 힙이 찬 것이 아니다: 실패 시점 목록 할당기 블록 18개, 18번째 헤더가 `0xFFFFFFFF`(`fe76`) · `0xFF000000`(`1eaa`)로 덮여 있어 걷기가 끝 너머로 뛴다(남은 공간 ~127MB).
- 헤더 쓰기 탐침: 두 타이틀 모두 **`MC_grpGetScreenFrameBuffer` 가 만든 0x25800 버퍼 바로 뒤**를 쓴다 — `fe76` 은 stdlib `memset`(480바이트 = 한 줄), `1eaa` 는 게스트 `strh`(`pc=0x1232e` Thumb).
- **한 줄 여유를 주면 쓰는 주소가 정확히 그만큼 밀린다**(`0x403431d0` → `0x403433b0`) ⇒ 게임은 `height` 가 아니라 **할당 크기**에서 주소를 얻는다. 그래서 여유는 소용이 없고 되돌렸다. 다음 회차는 그 주소를 만드는 호출(버퍼 핸들 앞 크기 단어를 읽는지)을 봐야 한다.

### 5. 전/후

후 = 이 PR head · 같은 조건 3판(load1 ~140). 전 열은 §1.

| sha12 | 전 | 후 | 다음 벽 |
|---|---|---|---|
| `2a57e33133b5` | PASS 1 · panic 2 | **PASS 3** | — |
| `517ed32c92d6` | PASS 1 · panic 2 | **PASS 3** | — |
| `85e94babc247` | panic 3 | **PASS 3** | 게임이 NPE 를 잡는다 |
| `c7f543c73b91` | PASS 1 · panic 2 | **PASS 3** | — |
| `0262a4fe3389` | 부팅 panic 3 | **PASS 3** | — |
| `a16f08d025eb` | 부팅 panic 3 | panic 0 · 한 색 화면(PASS 1 · FAIL 2 · paints 3) | 그림이 3장에서 멈춤 |
| `1eaa92092bee` | panic | `WieError` 3 | §4 |
| `fe76e641bb3d` | panic 3 | `WieError` 3 | §4 |
| `0cc4ef7ede37` `249e655147a1` `6bc7f65e3022` | PASS | PASS | — |
| 가드 `49ade89578c5` `ddd885583b15` | PASS 3 · PASS 2 + FAIL 1 | PASS 3 · PASS 3(첫 짝 판은 전·후 모두 PASS 2 + «no frame» 1 — 같은 시각 같은 모양이라 부하) | 퇴행 0 |

후 39판 전부 panic 0.

### 6. 되돌리면 red

`thread_keeps_the_phone_layout`(host_field 행 삭제) · `define_class_rust_rethrows_an_unresolvable_superclass`(부모 unwrap 복원) · `display_is_the_midlets_own_field_not_a_subclasss`(이름을 `display` 로) · `test_null_data_throws_null_pointer_exception`(null 검사 삭제) · `test_set_font_null_is_the_default_font`(null 분기 삭제) — 5개 모두 rc=101 확인. 리소스 경로 `unwrap` → `WieError` 는 시험이 없다(Java 예외를 일으키려면 할당 실패가 필요하다) — `fe76` 전/후가 증거다.

### 7. 데이터

`docs/player-data/compat.json` — **행 단위 갱신**. 방법: 이 13종만 담은 스크래치 corpus(심링크)로 `playability-census run --bin <이 head release>`(6축 · 장시간 600초) → `report --pin d1918719` → `player-data.mjs import` 로 공개 어휘에 맞춘 뒤, 원래 파일을 되살리고 **해당 sha 의 `status`·`axes`·`knownIssues_ko` 만** 바꿔 끼웠다(`title`·`fileTitle`·`changes` 는 그대로 · 같은 1칸 들여쓰기). 파일 머리의 `enginePin` 은 나머지 행이 잰 `3c34efee` 그대로다.
- 바뀐 행: `0262a4fe3389` not-yet → **playable** · `2a57e33133b5` `517ed32c92d6` `85e94babc247` `c7f543c73b91` `0cc4ef7ede37` `249e655147a1` limited → **playable**(장시간 무오류) · `a16f08d025eb` not-yet 유지(장시간 축 no → unknown) · `1eaa92092bee` `fe76e641bb3d` limited 유지.
- 가드 `49ade89578c5`(소리 ok → silent) · `ddd885583b15` 는 **기존 값을 유지**했다. `49ade` 소리를 전·후 빌드 2판씩 짝 재측하니 넷 다 재생 1회 · MIDI 4649 로 같았다 — 전수 1판의 0 은 흔들림이다.
- 합계 playable 349 → **356** · limited 50 → 44 · not-yet 30 → 29.
- `0cc4ef7ede37` `249e655147a1` 은 이 회차가 원인을 고친 타이틀이 아니다(30초 창에서 재현 0) — 새 빌드 장시간 1판이 무오류였다는 뜻일 뿐이라 이용자 소식은 내지 않았다.

### 8. 게이트

fmt · clippy `-D warnings`(stable · beta · wasm32) rc=0 · `RUST_MIN_STACK=4194304 cargo test --all` rc=0(51 묶음) · 러너 블록 PASS(`keydraw_*` 는 release 바이너리라 기본 `--max-ticks` 에 먼저 닿아 UNMEASURED — `origin/main` 빌드도 같다 · `--max-ticks` 를 올리면 두 종 PASS rc=0).
