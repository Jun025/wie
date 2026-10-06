## [2026-10-07] KTF 재배치 client.bin 3종 — 적재기 · `MNInterface` 13칸 · `fp` 문맥 (wie-ktf-relocated-client-bin-descriptor-loader-and-mninterface)

**무엇을**: 0340·0455 가 형식만 밝히고 멈춘 KTF 재배치 client.bin 3종(`1d5831e42a8a` `83fc429f9cbe` `b907b0faf483`)의 적재기를 넣었다(`wie-ktf/src/runtime/relocated.rs`).
`MNInterface` 칸은 13개를 구현했다. 칸마다 근거는 이미지 안 C 래퍼의 역어셈이고, 실행 추적으로 확인했다.
이미지가 기대하는 런타임 계약도 함께 맞췄다. `fp` 스레드 문맥, `+0x480` 호스트 워드 6개, 문자열 상수 객체, 필드 오프셋, 예외 복귀가 그것이다.

**왜**: 운영자가 추천 후속 작업 `2026-10-06-skt-3d-and-ktf-reloc-verdict#p1` 을 채택했다(0455 §3 의 남은 셋: 서술자 적재 · `MNInterface` · `fp` 문맥).

**사용자 영향**: 3종 중 2종이 이제 켜지고 화면을 그린다(§6). `83fc429f9cbe` 는 적재까지 되지만 KTF 라이브러리 클래스 `com/ktf/kfc/GProgressBar` 가 없어서 시작 화면 전에 멈춘다(§7).
표준 형식 KTF 타이틀은 바뀌지 않는다. 새 경로는 재배치 이미지를 적재했을 때만 켜진다(`KtfJvmSupport::relocated_abi`).

### 1. 적재 — 0340·0455 위에 더한 것

| 단계 | 내용 | 근거 |
|---|---|---|
| 재배치 | 표(접두 오프셋) 한 번 + GOT `[hdr+0x18, hdr+0x1c)` 한 번 | 0340 가설 1 그대로 |
| 클래스 레코드 | `JavaClass` 0x14 레코드 `count` 개가 **GOT 바로 아래**에 있다. `ptr_next = self+4` · `unk1 = 0x500` · `vtable = 0` | 3종 모두 `count` 전건이 이 조건을 만족(22·12·58) |
| 이름 참조 | 부모·인터페이스·예외표의 클래스는 `(색인 << 1) | 1` 로 `hdr+0x10` 이름표를 가리킨다. 짝수면 레코드 포인터다 | `1d5831e42a8a` 부모 `0x4b → [0x25] java/lang/Object` · `0x123 → [0x91] org/kwis/msp/lcdui/Card`. 코드 쪽 lazy 셀(`ldr r2,[r0]; lsrs r3,r2,#1`)도 같은 표기다 |
| 필드 오프셋 | 인스턴스 필드 오프셋이 **0 부터** 시작하고 `fields_size` 는 제 필드만 센다 | `1d5831e42a8a` 의 `Card` 하위 클래스는 첫 필드 오프셋이 0 이다. 부모 크기만큼 밀지 않으면 `Jlet.dis` 가 게임 필드로 덮였다(`b907b0faf483` `startApp` NPE) |
| 연결 | 이름 참조 해석 → 부모 크기만큼 필드 이동 → `JavaVtable::new` | 표준 KTF 레코드라 기존 `JavaClassDefinition`·`JavaVtable` 을 그대로 쓴다 |
| `init(param)` | `param[0] = get_interface` → `"MNInterface"` 요청 → 표를 돌려준다 | 0340 가설 2 |
| 클래스 로더 | `ExeInterfaceFunctions.fn_get_class` 만 채운다(이름으로 레코드를 찾아 연결) | `KtfClassLoader::find_class` 무변경 |

### 2. `MNInterface` 칸 — 래퍼별 근거 (`1d5831e42a8a` 주소)

각 칸의 뜻은 이미지 자신의 C 래퍼에서 읽었다. 래퍼는 칸을 `[[GOT 전역]] + off` 로 부르고, 실패하면 이름 붙은 예외를 던진다.

| 칸 | 인자 → 결과 | 래퍼 (실패 경로) | 호스트 구현 |
|---|---|---|---|
| `+0x20` | `(클래스 이름, 0)` throw | 모든 래퍼의 실패 경로 `("java/lang/Error", 0)` | `java_throw` |
| `+0x24` | `(예외, 0)` athrow | `0x152bf0` (`0x1000aa` athrow 도우미) | `java_throw_instance` |
| `+0x38` | `(클래스)` → 인스턴스 | `0x15293c` (`0x1000d2` new 도우미) | `java_new` |
| `+0x3c` | `(배열 클래스, 길이)` → 배열 | `0x1529f0` (길이 < 0 이면 `NegativeArraySizeException`) | `java_array_new` |
| `+0x40` | `(out, 클래스 이름)` → 0 | `0x15278c`·`0x152878`·`0x1527ec` (실패 시 throw) | `java_class_load` |
| `+0x44` | `(인스턴스)` → 클래스 | `0x152ad0` aastore 검사(배열의 원소형을 읽으려고) | 인스턴스 `+4` |
| `+0x48` | `(클래스, 인스턴스)` → 바이트 | `0x15297c` instanceof · `0x1529ac` checkcast(0 이면 throw) · aastore | `java_check_type` |
| `+0x54` | `(클래스, 필드 이름)` → 필드 레코드 | `0x1527ec` (정적 필드 셀) | `get_field` |
| `+0x60` | `(클래스)` 초기화 | `0x152904` — 레코드 `+0x12 & 8` 이 없을 때만 부른다 | `register_class` + 비트 8 |
| `+0x64` | `(클래스, 메서드 이름)` → 메서드 레코드 | `0x152878` (호출 셀), `0x152ba8` (인터페이스) | `get_java_method` |
| `+0x6c` | `(원소 클래스)` → 배열 클래스 | `0x1529f0` (`+0x3c` 앞) | 이름 `[L…;` 로 해석 |
| `+0x70` | `(atype × 4, 길이)` → 기본형 배열 | `0x152a64` | `T_*` 표 |
| `+0x74` | `(배열 클래스, 차원 수, 길이들)` multianewarray | `0x152b7c` (`0x10045e` 가 길이를 Java 스택에 push) | 재귀 생성 |

- `+0x70` 의 첫 인자는 호출 지점 191곳에서 `0x10 · 0x20 · 0x24 · 0x28` 네 값뿐이다(정적 · `1d5831e42a8a`). JVM `newarray` 의 `T_BOOLEAN 4 · T_BYTE 8 · T_SHORT 9 · T_INT 10` 의 네 배와 맞는다.
- 그 밖의 칸은 모두 이름 붙은 오류(`MNInterface +0x.. from …`)로 멈추게 했다. 세 타이틀이 위 길에서 부른 칸은 이 13개뿐이다(실행 추적). 0340 이 정적으로 센 10칸은 모두 이 안에 있다.

### 3. 이미지 `+0x480` — 호스트가 채우는 6워드

코드 시작(`hdr+0x14 = +0x480`)의 첫 6워드는 0이다. 도우미들은 `ldr rN, [pc, …]; mov pc, rN` 으로 여기를 거쳐 뛴다. 도우미의 쓰임새로 뜻을 정했다.

| 워드 | 부르는 도우미 | 뜻 | 채운 것 |
|---|---|---|---|
| `+0x480` | 해석된 메서드로 갈 때 모두(`r0` = 메서드 레코드, `r2,r3` push) | invoke | Thumb 트램펄린: `pop {r2,r3}` 후 `fn_body` 로 `bx` |
| `+0x484` | 같은 도우미에서 메서드 `ACC_NATIVE`(+0x16 비트 8)일 때 | invoke native | 같은 트램펄린 |
| `+0x488` | synchronized 메서드 입구(`0x10003a` 기록 뒤) | monitorenter | `monitor_enter` |
| `+0x48c` | synchronized 블록 출구와 예외 경로 | monitorexit | `monitor_exit` |
| `+0x490` | 루프 머리 96곳(`1d5831e42a8a`) · 인자 없음 | 안전점(양보) | 곧장 반환 |
| `+0x494` | synchronized 블록 입구 | monitorenter | `monitor_enter` |

### 4. `fp` 스레드 문맥과 스택

도우미가 `fp` 에서 읽는 칸: `+0x24` = Java `sp` 임시 보관 · `+0x2c` = 예외 기록 사슬 · `+0x30` = 함수표(기록에 복사) · `+0x34` = 네이티브 스택 · `+0x38` = JVM 문맥(`+0xc` = vtable 표, 표준과 같다).
이미지는 같은 문맥을 `hdr+0x20` 칸으로도 읽는다(`GOT[0x180] = +0x20`). 그래서 이 칸을 `KtfJvmSupport::set_current_thread_context` 가 함께 갱신한다.

- 호스트가 메서드를 부를 때마다 `fp` 를 문맥에 맞추고 `+0x30/+0x34/+0x38` 을 채운다. Java 프레임은 그 아래 `0x4000` 부터 시작한다(`relocated::enter`).
- **중첩 진입 실측**: `new` 도우미는 Java `sp` 를 `+0x24` 에 두고 네이티브 스택에서 런타임을 부른다. 그 안에서 `<clinit>` 이 돌면 새 Java 프레임은 **보관된 `sp` 아래**에 둬야 한다.
  처음에 「현재 `sp` 아래」로 뒀다. 그랬더니 `Card` 하위 클래스 생성자의 저장된 `lr` 이 호스트 스텁의 `push {r4}` 에 덮였다. 감시 주소를 두고 실측해 찾았다. `+0x24` 와 `+0x34` 는 진입마다 저장하고 되돌린다.

### 5. 문자열 상수 · 예외

- **문자열 상수**: 이미지에는 미리 만든 `String`·`char[]` 객체가 들어 있다(215 · 289 · 208개 · 3종 전건 같은 꼴). 차이는 둘이다. vtable 칸을 `char[]` = 0 · `String` = 5 로 고정해 뒀다. 그리고 필드 포인터가 `self+4` 라, 우리 인스턴스가 클래스를 두는 `+4` 와 겹친다.
  이미지는 객체를 필드 포인터로만 읽는다. 그래서 주소와 필드는 그대로 두고 머리만 우리 꼴로 바꿨다. `char[]` 는 필드를 제자리에 두고 머리를 밖으로 뺐다(가리키는 것은 그 `String` 뿐이다).
- **예외 기록**: 0x10003a·0x10004c·0x100072 가 만든다. 표준 기록과 다른 점은 셋이다. try 의 현재 pc 가 `+0x10` 에 있고, catch 는 예외를 `+0x38` 에서 읽고, `+0x18` 부터 `sp, lr, r4–r7, r8, sb, -, sl` 순서다.
  찾은 처리기로 돌아가는 것은 `setjmp` 와 같다. try 도우미의 반환 지점으로 가고, `r0` 에 처리기 번호(예외표 `target`)를 담는다. 메서드가 그 번호로 분기한다.
  예외표의 클래스도 이름 참조다(§1).
- **복귀 경로의 결함(고침)**: 카테고리 SVC 처리기가 `()` 를 돌려주면 `RegisteredFunctionHolder` 가 `pc` 를 `lr` 로 다시 쓴다. 안쪽 함수가 정한 재개 지점(`next_pc`)이 그래서 사라졌다.
  재배치 경로는 이 때문에 catch 를 «찾고도» 호출 다음 줄로 돌아갔다(`b907b0faf483` 이 세이브 없음 예외 뒤 검은 화면). 처리기를 `JumpTo(현재 pc)` 로 바꿨다(`java::resume_here`).
  표준 경로의 JAVA 카테고리도 같은 처리기를 쓴다. 그 안쪽 함수는 언제나 `lr` 을 쓰거나 오류를 돌려주므로 동작은 같다.
- 기록이 바깥 진입의 프레임이면(`+0x18 sp ≥ +0x34`) 여기서 잡지 않고 Java 예외로 넘긴다. 그러면 바깥 진입이 잡는다.

### 6. 측정

`playability-census.mjs run --only probe --jobs 2`(long 풀 · 호스트 잠금) 으로 같은 28종을 전(`origin/main` `72fda07e`) → 후(이 브랜치) 순서로 쟀다. load1 은 8~11 이다.
대상은 재배치 3종, 가드 2종(`49ade89578c5` `ddd885583b15`), 예외가 많은 KTF 2종(`e9fac881e602` `dbd078113b97` — 이번에 예외 경로를 건드렸다), 힙 회귀 1종(`8d8c24b7c198`), 표준 KTF playable 20종(sha 순 앞 20)이다.

| 묶음 | 종 | A/B 판정·stop·content·소리가 바뀐 것 |
|---|---|---|
| `1d5831e42a8a` | 1 | FAIL(error · `Unsupported KTF client.bin layout`) · paints 0 → **PASS(deadline) · paints 251 · 소리 있음 · 예외 0** |
| `b907b0faf483` | 1 | 같은 FAIL → **PASS(deadline) · paints 296** · 예외 3(첫 실행이라 세이브가 없다 — 게임이 잡는다) · 소리 없음 |
| `83fc429f9cbe` | 1 | 같은 FAIL → FAIL(`GProgressBar` 없음 → `startApp` NPE) · paints 0 → 1 |
| 가드 2 · 예외 2 · 힙 1 · 표준 KTF 20 | 25 | 0 |

- 마지막 화면: `1d5831e42a8a` 는 슈팅 스테이지 진행 중이다(적·탄·점수 표시). `b907b0faf483` 는 인트로 이야기 글이다.
- 예외 수가 다른 칸은 하나다. `0e6cd188729e` A 의 Java 예외가 102 → 0 이다. 같은 시각에 다시 잰 짝에서는 전 0·102, 후 0·0 이었다. 전 바이너리도 0 과 102 를 오가므로 시점 차로 본다(첫 예외 `NullPointerException: image is null`).
- 러너 줄(후 바이너리): `draw_j2me` · `helloworld_ktf/lgt` · `text_j2me --timeout 5` PASS. `keydraw_ktf/lgt --inject --expect-last-frame` 은 첫 실행이 `UNMEASURED · stop max-ticks` 였다(load1 11). AGENTS 지시대로 `--max-ticks` 를 올리자 전·후 모두 PASS · rc=0 이었다(paints 79/80 · 55/55).

**6-1. longplay 와 compat**: 후 바이너리로 `--only long`(600초 · 기본 키)을 두 종에 돌렸다. 둘 다 600초 내내 그렸고 끝까지 오류가 없었다(paints 4,998 · 5,992).
`1d5831e42a8a` 는 소리가 났고(plays 3), `b907b0faf483` 는 소리가 0이었다.
`b907b0faf483` 의 마지막 화면은 타이틀의 「HOW TO PLAY」 메뉴다. 기본 키로는 본 게임 시작까지 확인하지 못했다.
compat 의 두 행은 `playability-census.mjs report` 가 이 측정에서 계산한 축을 그대로 옮겼다. 공개 스키마에 맞춰 `silent` 는 `no` 로 적고 `progress n/a` 는 뺐다.
- `1d5831e42a8a`: not-yet → **playable**(모든 축 ok).
- `b907b0faf483`: not-yet → **playable** · sound `no`(「소리가 나지 않을 수 있어요.」).
- `83fc429f9cbe` 행은 바꾸지 않았다. 여전히 시작하는 도중에 멈춘다.

### 7. 남은 벽

| 대상 | 벽 | 크기 |
|---|---|---|
| `83fc429f9cbe` | KTF 라이브러리 클래스 `com/ktf/kfc/GProgressBar` 미구현이다. `load_java_class` 가 실패해 예외가 나고, 게임이 그것을 잡은 뒤 null 진행 막대로 `startApp` 에서 NPE 가 난다. 이미지 이름표에서 진행 막대용으로 보이는 이름은 `setMaximum(I)Z` · `setValue(I)Z` 다(어느 생성자인지는 정하지 않았다) | M — 새 UI 클래스 |

### 8. 검증

- 시험 4개(`relocated::tests`): 부모 이름 참조와 필드 이동 · 문자열 상수 입양 · 중첩 진입 스택 · catch 재개.
- 되돌리면 red 5 변이 전건: 필드 이동 생략 · 부모를 이름 대신 `Object` 로 · 상수 머리 미교체 · 보관 `sp` 무시(`max`) · 처리기 반환을 `lr` 로. 각각 해당 시험이 FAILED 였고, 원상에서는 4/4 통과했다.

게임 파일명 유입: 도구 기본 실행(브랜치 전체)은 BOUNDED 333쌍 · SUFFIX-ATTACHED 16쌍이다. 거의 전부 `docs/player-data/compat.json` 안이다(계약상 제목 목록).
compat.json 을 뺀 이 회차 파일만 넘기면 BOUNDED 3쌍 · SUFFIX-ATTACHED 1쌍이다. 모두 이 회차가 고친 파일에 원래 있던 줄이다(`adf.rs` 시험 · `interface.rs`·`jvm_support.rs` 주석). 이 회차가 더한 줄에는 0이다.
