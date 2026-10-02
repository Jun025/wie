## [2026-10-02] 진도 벽 3차 — LGT 힙 고갈은 panic 대신 `OutOfMemoryError` · 누수 원인 실측 · `DataOutputStream` 24 · SKT 패키지 화면 크기 (wie-progression-walls-r3-live-lgt-heap-panic-manual-page-image-null)

**무엇을**: 0415 §5·§8 이 남긴 벽 다섯을 쟀다. ① 라이브 LGT `1b107b96bf4e` 43분 힙 고갈 → 호스트 panic ② 라이브 LGT `b475b6399684` 설명서 마지막 쪽 무반응 ③ SKT `38277d63b0ba` `img is null` ④ SKT `bf54c05e58a9` 240폭 그림 찾기 ⑤ ⒝? 셋.
**판정**: ① panic 은 **고쳤다**(고갈하면 예약 `OutOfMemoryError` 를 던진다). 누수는 **원인을 쟀고 고치지 않았다** — GC 루트 모델 변경(L)이다(§1). ② **고쳤다**(ABI 행 1). ③ r2 와 **다른 뿌리**다 — 그림 탓이 아니라 그림이 로드되기 전 첫 paint 다(§3). ④ **고쳤다**(패키지 내용으로 화면 크기 선택 · 같은 규칙에 1종 더 걸린다). ⑤ 분류만 했다(§5).
**사용자 영향**: 설명서에서 멈추던 게임이 게임 화면으로 들어간다. 작은 화면용 SKT 게임 2종이 제 크기로 뜬다(검은 화면 → 메뉴·이야기 · 빠진 제목 그림 → 나옴). 40분 넘게 하는 LGT 게임은 힙이 차도 탭이 panic 으로 죽지 않는다 — 다만 힙은 여전히 찬다(§1).

증적: `~/orchestrator/reports/evidence/wie-progression-walls-r3-live-lgt-heap-panic-manual-page-image-null/`. 타이틀은 sha12 로만 적는다.

### 1. ① `1b107b96bf4e` — panic 의 출구는 막았고, 누수는 GC 루트 모델이다

**panic**: 0415 실행(2564초)의 stderr 끝은 `LGT host error unbuildable, dropped: net/wie/WieError …` 3회 → `class_instance.rs:109` `unwrap` panic 이다. `host_error` 가 오류를 만들지 못하면 주소 0 인스턴스를 대신 돌려줬고, 호스트 코드가 그 클래스를 읽다 죽었다.
**처방**(`wie-lgt/src/runtime/java/jvm_support.rs`): JVM 기동 때 `OutOfMemoryError` 하나를 미리 만들어 `net/wie/LgtClassLoader.outOfMemoryError`(정적 필드 = GC 루트)에 둔다. KTF 가 이미 쓰는 방식이다(`KtfJvmSupport::instantiation_error`).
- 객체·배열 할당이 `AllocationFailure` 면 그 예약 객체를 던진다. 할당이 없는 Java 예외라 게스트가 잡을 수 있다.
- `host_error` 가 오류를 못 만들면 주소 0 대신 예약 객체를 돌려준다. 클래스를 읽어도 panic 하지 않는다. `unwind` 가 다음 예외를 호스트 오류로 끝내는 동작은 그대로다.
- 시험 `exhausted_heap_throws_the_reserved_out_of_memory_error`(기존 `exhausted_heap_ends_array_instantiation_with_a_host_error` 를 바꿈). 1 GiB 배열과 가득 찬 힙 모두 예약 객체가 나와야 하고, 못 만든 오류의 대역은 `java/lang/OutOfMemoryError` 로 읽혀야 한다. **되돌리면 red**: 주소 0 대역으로 되돌리면 `class_definition()` 에서 panic 한다. `allocation_error` 를 빼면 identity 단언이 진다.

**누수 실측**(임시 계측 3종 · 커밋 안 함 · #446 진도 정책 v2 로 재현 · `wie_validate` = #446 head `cae0cce7` + 계측):
- 힙 버킷 사용량(65,536 할당마다): 한 장면에서 16바이트 버킷이 **1.5분에 +31k** 로 늘었다(8·64바이트 버킷도 같은 비율). 최대 **214,806 / 524,288** 까지 갔다가 장면이 바뀌거나 재기동하면 내려간다. 그 속도면 한 장면에서 약 25~40분이면 16바이트 버킷이 찬다 — 0415 의 2564초와 맞는다.
- 이번 60분 재현(0415 와 같은 정책 · origin/main 위)은 **panic 없이 끝났다**(PASS · paints 64,684). 키 타이밍이 달라 그 장면에 머문 시간이 짧았다. 그래서 «40분에 반드시 죽는다»가 아니라 «그 장면에 오래 있으면 죽는다»가 실측이다.
- 살아 있는 jvm 객체 수(클래스별 · 생성 −파괴): 메뉴·상점 장면은 **1.7k 로 평탄**하다. 누수 장면은 **1.7k → 61k(2.5분)** 로 는다. 상위 클래스는 `[C` 24,711 · `String` 22,384 · `StringBuffer` 8,228 이다. `Font`(매 프레임 `getDefaultFont`)는 169 로 평탄했다 — 누수가 아니다.
- 살아 있는 블록의 할당 지점(lr): 상위가 게스트 코드 주소 10여 곳(`0x48979` · `0x4bcd9` · `0x5966f` …)에서 온 12바이트 블록이다. 각 2.3~3.9k 개다 = **게스트 `new`**(문자열 이어 붙이기의 `StringBuffer`)이다.
- **원인 두 갈래**(코드):
  ⒜ 게스트 `new`(`java_instantiate`)는 jvm 객체 집합에 넣지 않아 **애초에 수거되지 않는다**. #445 부터는 전역 참조로도 붙들어, 그 `char[]` 까지 산다(#445 이전 0415 실행에서도 panic 이 났으니 #445 가 원인은 아니다).
  ⒝ 게스트가 부른 호스트 메서드(`JavaMethodProxy::call`)는 프레임을 쌓지 않는다. 그래서 그 안에서 만든 객체(`toString` 의 `String` 등)는 rustjava `instantiate_class` 가 **바깥 호스트 프레임의 지역 변수**에 넣는다. 게스트 루프가 그 호출 안에서 도는 동안 풀리지 않는다. KTF 도 같은 모양이다.
- **고치지 않은 이유**: 두 갈래 모두 «게스트 스택·레지스터가 쥔 참조를 수집기가 못 본다»에서 나온다. 고치려면 수집 직전에 게스트 스레드 스택과 레지스터를 보수적으로 훑어 루트로 넣어야 한다. 그다음 게스트 `new` 를 객체 집합에 넣고, 프록시 호출마다 프레임을 쌓았다 내린다. 수집 호출 지점(midp `Display` · LGT/KTF 컨텍스트 · `Runtime.gc`)에 훅이 필요하다. 루트를 하나라도 놓치면 be08 같은 use-after-free 가 된다. **크기 L** · 후속 제안 p0. 힙 상한은 건드리지 않았다.
- **60분 «후» 실행**(같은 정책 · #446 head + 이 브랜치 커밋): PASS · panic 0 · paints 11,032. 그러나 t≈400초부터 끝까지 «Loading · Press Any Key» 화면에 머물렀고 탈출 키 58회로도 넘지 못했다. 누수 장면에는 들어가지 않았다. ⇒ **이 실행은 «60분 무panic»의 증거이지 «누수 장면을 견딘다»의 증거가 아니다.** 같은 두 바이너리(수정 전 · 후)를 600초 짝으로 돌리면 프레임이 시점마다 같다(정보 화면 → 메뉴 → 본편). 경고 집합도 같다. `1b107b96bf4e` 는 `DataOutputStream` 24 를 한 번도 부르지 않는다(4개 실행 stderr 계수 0). 그래서 그 Loading 정체는 키 타이밍에 따른 경로 차이로 읽는다 — 원인은 재지 않았다.

### 2. ② `b475b6399684` — 설명서 OK 가 `DataOutputStream` 24 에서 게임 스레드를 죽였다

- 손 키 재현(OK 3초 간격): 설명서 두 번째 쪽에서 `Unimplemented: java/io/DataOutputStream vtable index 24` (r1 = 힙 객체 · lr `0x31b3f`) → `Uncaught exception in thread` 다. 화면은 남고 키는 아무 곳에도 닿지 않는다. 0415 가 본 «마지막 쪽 무반응»이 이것이다(쪽 번호는 키 타이밍에 따라 다르다).
- **24 = `writeUTF(Ljava/lang/String;)V`**: 15에서 CLDC 1.1 선언 순서로 센다. `writeChar` 18 · `writeFloat` 21 · `writeDouble` 22 · `writeChars` 23 · `writeUTF` 24. CLDC 에는 `writeBytes` 가 없다(J2SE 에는 있어 `writeUTF` 가 25 가 된다). 이미 잰 `DataInputStream` 이 J2SE 순서를 반증한다 — `readUTF` 32 가 `readDouble` 31 바로 다음이다(J2SE 면 그 사이에 `readLine` 이 있다). 행·주석 = `wie-lgt/data/lgt_java_abi.toml`.
- 후: 같은 키로 설명서를 넘어 스킨 모음판 → 게임 화면까지 간다(증적 `b475-after-keys.png`). 시험 `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 에 행 추가 — **되돌리면 red**(행을 빼면 그 단언이 «index 24 is empty» 로 진다).
- 진도 축(v1 정책 600초 · census `--only progress`): **stuck** 이다. 막힌 곳이 설명서 → **스킨 모음판**으로 옮겨 갔다. 정책 키가 판 위 커서만 돌리고 들어가지 않는다(⒜ 정책 계급 · 증적 `b475-after-progress600.png`). OK 만 누르면 들어간다(위 손 키 재현).

### 3. ③ `38277d63b0ba` — r2 와 다른 뿌리: 그림을 로드하기 전에 첫 paint 가 그림을 그린다

r2(`wie-progression-engine-walls-r2-null-image-dialog-npe-key`)의 `9789fec50f39` 는 **KTF** 다. 이것은 **SKT** 이고 사인도 다르다. 그래서 stand down 하지 않고 쟀다.
- 게임 구조(javap): `run()` 이 `repaint(); wait();` 를 돈다. `paint()` 끝의 `notify()` 가 그것을 깨운다. 로딩은 `paint()` 안에서 단계별로 한다.
- 첫 paint 가 로딩 막대를 그리며 `drawImage(E[i])` 를 부른다(53,163). 그때 `E[i]` 는 아직 null 이다 → `NullPointerException: img is null` → `notify()` 에 닿지 못한다 → `run()` 이 영원히 `wait()` 한다 → 흰 화면에 로딩 막대 한 조각(증적 `38277-paint-before-load.png`). 그 전까지 `createImage(String)` 호출은 0회다.
- 실기에서 이 순서가 어떻게 통과하는지(SKVM 의 null 그림 그리기 동작인지, 첫 paint 시점 차이인지)는 재지 않았다. MIDP 명세는 null 이면 NPE 다. 그래서 `drawImage(null)` 을 묵인하는 수정은 근거 없이 하지 않았다. **⒝ · M** · 후속 표.

### 4. ④ `bf54c05e58a9` — SKT 패키지가 가진 그림 크기로 화면을 고른다

- 코드(javap): `Canvas.getWidth()` 로 `<176 → _120 · 240×160 → _120 · <240 → _176 · 그 밖 → _240` 을 고른다. 패키지에는 `_120`·`_176` 만 있다 → 240폭에서 `Resource not found: /title/main_logo_240.png` → `img is null` → 검은 화면.
- **규칙**(`wie-skt/src/emulator.rs` `package_display_size`): 클래스 상수 풀의 문자열 중 `<stem>_<폭>.<확장자>`(폭 ∈ 120·128·176·240)가 jar 에 **없고**, 같은 stem 의 더 좁은 폭이 jar 에 **있으면**, 그 중 가장 넓은 폭이 패키지가 만들어진 화면이다. 화면이 그보다 넓으면 `Screen::resize` 한다(KTF `DisplaySize:` 와 같은 경로 · 브라우저 호스트는 실제로 바뀐다 · `wie_validate` 는 no-op). 높이는 폭별 흔한 패널(120·128→160 · 176→220)이다 — 패키지가 높이를 말하지 않는다(`ponytail:` 주석).
- 타이틀 하드코딩 없음. 코퍼스 462 패키지를 같은 술어로 훑으면 **2종**이 걸린다(둘 다 SKT): `bf54c05e58a9`(240 없음 → 176) · `78bd51675574`(176·240 없음 → 120). KTF·LGT 는 0종이다.
- 시험 `display_size_is_the_widest_variant_a_package_holds_beside_a_missing_one`: 상수 풀 파서(long 이 두 칸)와 규칙 4경우. **되돌리면 red**(규칙을 바꾸면 단언이 진다).
- 측정: `wie_validate` 의 화면이 240×320 고정이라, 화면 상수만 바꾼 임시 빌드(176×220 · 120×160 · 커밋 안 함)로 잰다. 같은 키 30초:
  - `bf54c05e58a9` 240: paints 24 · 예외 2 · 검은 화면 → 176×220: paints 287 · 예외 0 · 로고 → 메뉴 → 새 게임 → 이야기 화면.
  - `78bd51675574` 240: 예외 879(`/img_size/240.png` 매 프레임) · 제목 그림 빈칸 · 필드까지는 간다 → 120×160: 예외 0 · 제목 그림 · 메뉴 · 필드.
  - 진도 축 600초(census `--only progress` · v1 정책 · 176×220 임시 빌드): `bf54c05e58a9` 새 화면 **1 → 15** · 마지막 새 화면 **10초 → 250초**(마을 → 건물 안 대화). 판정은 여전히 `stuck` 이다(정체 350초 ≥ 200). `78bd51675574` 는 census 락을 다른 레인이 쥐고 있어 **재지 못했다**(손 키 30초 실측만).

### 5. ⑤ ⒝? 셋 — 분류

| sha12 | 플랫폼 | 실측 | 계급 · 크기 |
|---|---|---|---|
| `44b6356d13f8` | SKT | `install.dat` 를 `RandomAccessFile` 로 열다 `FileNotFoundException` → 클래스 `a` 가 「인증 되지 않은 컨텐츠 입니다. 프로그램을 종료합니다.」를 그리고 `System.exit(-1)`(javap). 앞의 `option.dat` FNF 는 게임이 잡는다 | **⒞** 구매 단말 인증(설치 토큰). census 의 «잠긴 파일» 판별은 XCE `SecureUtil` 경로만 알아 이것을 못 본다 — 판별 확장 S |
| `d1e0badfce82` | KTF | 환경설정 뒤 매 paint 마다 게스트가 `ArrayIndexOutOfBoundsException` 을 던진다(`java_throw` pc `0x1453f8` · 90초에 489회) → 배경 그라데이션만 남는다 | ⒝ · M(역어셈 추적) |
| `a540945188ca` | KTF | 90초 동안 경고는 `java_check_type` 스텁뿐이고 예외 0 · 폼 칸 테두리만 그려지고 글자가 없다 | ⒝? · M(그리기 경로 추적) |

### 6. 퇴행

- `RUST_MIN_STACK=4194304 cargo test --all` **651 pass / 0 fail** · fmt · clippy stable·wasm32·beta `-D warnings` · `npm run build:wasm` rc=0 · `check-engine-contract` OK · `npm run audit` PASSED.
- 러너 블록: draw · helloworld ×2 · keydraw ×2(`--inject --expect-last-frame` rc=0) · text 모두 PASS.
- 라이브 LGT 5 + 가드 2(기본 27키 · 30초 · `origin/main` `890ae6e1` 빌드 ↔ 이 브랜치 · 결과 · paints · 예외):

| sha12 | 전 | 후 |
|---|---|---|
| `13d7e3c21856` | UNMEASURED(max-ticks) · 130 · 0 | UNMEASURED(max-ticks) · 133 · 0 |
| `1b107b96bf4e` | UNMEASURED · 109 · 6 | UNMEASURED · 108 · 6 |
| `4ece6eeeaa04` | UNMEASURED · 9 · 0 | UNMEASURED · 8 · 0 |
| `a30bbe008b5e` | UNMEASURED · 88 · 1 | UNMEASURED · 87 · 1 |
| `b475b6399684` | UNMEASURED · 245 · 1 | UNMEASURED · 484 · 1 |
| `49ade89578c5`(KTF 가드) | UNMEASURED · 76 · 0 | UNMEASURED · 71 · 0 |
| `ddd885583b15`(가드) | PASS · 813 · 0 | PASS · 863 · 0 |

  `UNMEASURED` 는 키 일정보다 tick 상한이 먼저 닿은 것이다(0415 와 같은 모양). 전부 `content true` 다. 바뀐 것은 `b475b6399684` 의 paints 뿐이다(설명서를 넘어 그림이 더 많다).

### 7. `compat.json`

손으로 바꾼 행이 **없다**. 잰 값이 기존 값과 같다 — `b475b6399684` `progress` stuck(설명서 → 스킨판) · `bf54c05e58a9` stuck(검은 화면 → 마을 · 176 임시 빌드). `78bd51675574` 는 진도를 재지 못했다. P2 짝 재측은 하지 않았다. 고친 내용은 `docs/player-updates/` 2건이 `changes[]` 로 파생한다(빌드 몫).

### 8. 후속(수 · 계급 · 크기)

| 군집 | 수 | 계급 | 크기 |
|---|---|---|---|
| LGT 게스트 스택을 GC 루트로(게스트 `new` 수거 · 프록시 호출 프레임) — `1b107b96bf4e` 누수 | 1(+ 문자열을 매 프레임 만드는 LGT 전부) | ⒝ · worklog p0 | L |
| `38277d63b0ba` 그림 로드 전 첫 paint 의 `drawImage(null)` | 1 | ⒝ · worklog p1 | M |
| `d1e0badfce82` 환경설정 뒤 매 paint AIOOBE | 1 | ⒝ | M |
| `a540945188ca` 폼 글자 없음 | 1 | ⒝? | M |
| census «잠긴 파일» 판별에 설치 토큰 인증 종료 추가 — `44b6356d13f8` | 1 | ⒞ 표기 | S |
| `b475b6399684` 스킨 모음판을 넘는 키 레시피 | 1 | ⒜ | S |
| `78bd51675574` 진도 축 재측(120×160) | 1 | 측정 | S |

### 9. 유입

`node scripts/corpus-name-inflow.mjs --corpus <로컬 코퍼스>`: BOUNDED 107회 / 31쌍 · SUFFIX-ATTACHED 14회 / 4쌍. 전부 이 회차가 고친 파일에 **이미 있던** 줄(`lgt_java_abi.toml` · `jvm_support.rs` 의 기존 주석·시험 행)이다. 이 회차가 더한 줄의 게임 이름은 0이다. 고쳐 쓴 주석 한 줄은 이름을 sha12 로 바꿨다.

<!-- corpus-name-inflow v1 subjects=1332 tree=ee98759ee3709d60 B=107/31 P=2/2 S=14/4 -->
