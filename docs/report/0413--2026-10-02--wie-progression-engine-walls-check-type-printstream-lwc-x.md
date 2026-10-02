## [2026-10-02] 진도 축 엔진 벽 3건 — `java_check_type` 가설 반증 · LGT GC 가 게스트 `new` 의 자식을 지운다 · `PrintStream` 27 · lwc 필드 `x` (wie-progression-engine-walls-check-type-printstream-lwc-x)

**무엇을**: 0393 §6 이 남긴 엔진 벽 셋을 쟀다. ① KTF `java_check_type(unk≠0)` 가 «항상 참»이라 AIOOBE/NPE 가 난다는 의심. ② LGT `PrintStream` 27 · 게스트 `TimerTask` 11. #425 에서 그 행을 넣으면 호스트 panic 이 났다. ③ lwc `Component` 필드 `x`.
**판정**: ① **가설 반증 — 고칠 것 없음.** ② **고쳤다.** panic 의 원인은 ABI 행이 아니었다. LGT 수집기가 게스트 `new` 객체가 들고 있는 호스트 할당물을 지웠다. 행 셋을 더했다(`TimerTask.cancel` 11 · `PrintStream.println(Z)` 27 · `String.getChars` 12). ③ **고쳤다**(필드 4개 선언). 회차 중 #444 가 착지해 키가 입력칸에 닿자 같은 타이틀의 다음 벽 `TextComponent.getMaxLength()I` 가 드러났고, 그것도 고쳤다(§3).
**사용자 영향**: 타이틀 세 종이 막힌 지점을 넘었다(§4). 로고 뒤 멈춤 → 시작 메뉴, 안내 화면 → 선택 화면, 이름 입력 뒤 멈춤 → 본편. 그러나 600초 진도 축으로는 세 종 모두 아직 `stuck` 이다 — 벽을 넘은 뒤 다음 자리에서 멈춘다. 그래서 `compat.json` 은 바꾸지 않았다(§8).

증적: `~/orchestrator/reports/evidence/wie-progression-engine-walls-check-type-printstream-lwc-x/`. 타이틀은 sha12 로만 적는다.

### 1. ① KTF `java_check_type` — 무엇을 검사하나

임시 계측을 넣고 `01e2715ba07a`·`9789fec50f39` 를 60초 돌렸다. 계측은 호출 lr, 인자, 호출부 코드, 그리고 guest r4 가 가리키는 배열을 찍었다(커밋 안 함).

- **호출부는 하나다.** 두 이미지 모두 모든 호출(각 651 · 258회)이 같은 lr 로 돌아온다(`0x15bb73` · `0x17cbf7`). 그 함수를 역어셈하면 `aastore` 도우미다. `(array r0, index r1, value r2)` 를 받아 null·범위를 검사하고, value 가 null 이 아니면 `java_check_type` 을 부른 뒤 `str r6,[r3,#8]` 로 저장한다. 결과가 0 일 때만 `ArrayStoreException` 쪽으로 간다.
- **`unk` 는 인자가 아니다.** 직전 계산 `r2 = 배열 헤더 >> 5`(= vtable 색인 × 4)가 r2 에 남은 것이다. 실측값 `0x9c`·`0xd8`·`0x100`… 은 배열 헤더 `0x1380`·`0x1b00` 과 정확히 맞는다. 그래서 «unk≠0 → 1» 분기는 모든 호출에서 탄다.
- **`ptr_class` 는 원소 클래스가 아니다.** 게스트는 `[[T+8+색인] + 0x14]` 로 원소 타입을 구하는데, 이 엔진에서 배열 클래스는 vtable 표에 자리가 없다(`ptr_vtable = 0` → `get_vtable_index` 가 표 끝 색인을 준다). 그래서 다음에 등록된 클래스의 칸을 읽는다. 계산값이 실제 인자와 같았다(`r0calc = 0x49048140`).
- ⇒ «항상 참»이 할 수 있는 일은 **`ArrayStoreException` 을 한 번도 던지지 않는 것**뿐이다. AIOOBE·NPE 를 만들 수 없다. 올바른 프로그램에서는 그 검사가 늘 통과하므로 결과도 같다. 배열에 vtable 칸을 주는 수정은 득 볼 타이틀이 없어 하지 않았다.
- **`9789fec50f39` 의 실제 사인**: 「게임시작」 직후 게임 스레드(`ac.run`)가 NPE 로 죽는다. 던지는 곳은 `0x17c684` 도우미다(가상 호출 전 null 검사). 해석된 참조(`0x19c72c`)는 `org/kwis/msp/lcdui/Image` 메서드(vtable 0x14)이므로, **null 인 `Image` 에 메서드를 불렀다.** 어느 정적·필드가 null 로 남았는지는 추적하지 않았다(L · worklog 제안 p0).
- **`01e2715ba07a`**: 300초 진도 정책 실행에서 AIOOBE 가 나오지 않았다(전·후 같음 · 새 화면 6 · 60초 이후 정지). 0393 이 본 AIOOBE 는 580초 근처다. 이번 회차에서 사인을 재지 않았다.

### 2. ② LGT `TimerTask` 11 — panic 은 수집기였다

- **호출부**(`be08d047cbae` `binary.mod` · lr `0x23a8` · objdump): `if (timer != null) { timer.<16>(); timer = null; } if (task != null) { task.<11>(); task = null; }`. 인자 없이 부르고 결과는 버린다. CLDC 순서(`run` 10 · `cancel` 11 · `scheduledExecutionTime` 12)와 같으니 `cancel()Z` 다.
- **#425 의 panic 재현**(`Expected object, got Int(…)`): 행을 넣고 `RUST_BACKTRACE=1` 로 돌리면 `TimerThread.run` → 게스트 `c.run` → `Stack.pop` → `peek` → `Vector.elementAt` → `load_array` 에서 터진다. 게스트 `Stack` 의 `elementData` 가 `[I` 로 읽혔다.
- **추적**: `Stack.<init>` 이 `[Ljava/lang/Object;` 를 넣은 직후 수집이 한 번 돈다. 그 배열이 `destroy` 되고, 같은 주소가 `int[]` 로 다시 할당됐다. `Stack` 자신은 지워지지 않았다. 메모리 전수 검색으로 보면 `Stack` 을 가리키는 워드는 하나뿐이다 — 게스트 클래스 `w` 인스턴스의 워드 1(참조 비트맵에 표시됨). 그 `w` 를 가리키는 워드는 게스트 메모리 어디에도 없다(레지스터에만 있다).
- **원인**: `java_instantiate`(게스트 `new`)는 `definition.instantiate` 로 객체를 만든다. 그래서 jvm 객체 집합에 들어가지 않는다 — 지워지지 않지만 **루트도 아니다.** 그 생성자가 호스트에서 만든 배열은 객체 집합에 들어간다. 게스트 레지스터·스택만 그 객체를 쥐고 있으면, 수집기에게 배열은 고아다.
- **처방**(`wie-lgt/src/runtime/java/interface.rs` `java_instantiate`): 게스트 `new` 마다 전역 참조를 하나 만들고 놓지 않는다(`mem::forget`). 게스트 객체는 원래 영원히 살아 있으므로, 이 변경은 «게스트가 아직 닿을 수 있는 것»만 살린다. 대가는 두 가지다. 객체마다 맵 항목 1개가 생기고, 수집 1회당 추적 대상이 게스트 객체 수만큼 는다. LGT 의 수집은 그림 경로에서 돈다(`wie-midp` `Display` · 1초 간격 · 비용의 20배 간격 상한으로 시간 ≤5%). 그래서 추적 비용이 늘면 간격이 스스로 벌어진다.
- **잔류 실측**(임시 힙 사용량 계측 · 같은 트리에서 이 줄만 끄고 켠 두 바이너리 · 진도 정책 300초): 버킷 사용 바이트 `ddd885583b15` 1,749,764 → 1,749,796 · `a30bbe008b5e` 287,424 → 292,892 · `73f3a21e981c` 1,156,092 → 1,366,776(+210KB · 버킷 영역 128MB 중). 리스트 영역은 세 종 모두 ±0.05%. 그림 수 9,423 → 9,475 · 5,879 → 5,878 · 7,141 → 7,466 — 수집 비용 증가는 보이지 않는다.
- 행 둘 다 필요하다. 행만 넣으면 panic, 루트만 넣으면 `Unimplemented: c vtable index 11` 이다.

### 3. ② `PrintStream` 27 · 그 다음 `String` 12 · ③ lwc `x`

- **`PrintStream` 27**(`73f3a21e981c` · r1 = 1): DataOutputStream 주석의 계산(checkError 15 · setError 16 · print ×9 = 17..25 · println() 26)대로 27 은 `println(Z)V` 다. 측정 앵커 `println(String)` 34 와도 맞는다(27..34 가 println 8개). 0232 의 «설명 안 되는 두 칸»은 이 계산이 이미 checkError·setError 로 풀어 둔 것이다.
- 그 다음 벽은 **`String` 12**(lr `0xbe114` · 역어셈)였다. `r1 = 0`, `r2 = 직전 slot-10(length) 결과`, `r3 = 필드의 배열`, 스택 1워드 0 → `s.getChars(0, s.length(), buf, 0)` 이다. 규칙상 charAt 11 과 getBytes 14 사이 12 에 놓인다. 함께 넣었다.
- **lwc 필드 `x`**(`d448aee68157`): `Fatal error: Field xI@182 not found from org/kwis/msp/lwc/Component`. javadoc(`AromaWIPI_javadoc.zip` Component.html)의 protected 필드 가운데 위치·크기 `x y w h` 를 선언했다. 값은 0 으로 두었다 — getX/getY/getWidth/getHeight 가 이미 0 을 돌려주니 같은 답이다. KTF 게스트 하위 클래스의 자기 필드는 실기 부모 크기(인스턴스 필드 12워드) 뒤에서 시작하므로, 호스트 부모가 2 → 6 워드가 되어도 겹치지 않는다.
- 이 벽은 #444(lwc 폼 키 전달)가 다루지 않았다. 그 PR 은 `component.rs` 의 `setFocus` 만 바꿨다. 회차 중 #444 가 착지해 그 위로 rebase 했다(충돌 0).
- **rebase 뒤 다음 벽 `TextComponent.getMaxLength()I`**: #444 로 키가 포커스 입력칸에 닿자, 같은 타이틀의 이름칸이 첫 키에서 `getMaxLength()I` 를 불렀다. 그 메서드가 없어 `Fatal error: Method getMaxLength()I@247 not found` 로 그 틱이 죽었다(600초 실행 FAIL · 91번째 키). 앞서 `setMaxLength(5)` 를 부르므로, `setMaxLength` 가 `maxLength` 필드에 값을 남기고 `getMaxLength` 가 그 값을 돌려주게 했다. 입력 길이를 자르지는 않는다(keyNotify 그대로). `maxLength` 를 직접 읽는 두 타이틀(그 필드 주석)은 이제 0 대신 게임이 정한 값을 읽는다.

### 4. 전/후

바이너리: 전 = `origin/main 5cbd635e` release · 후 = 이 브랜치(같은 base) release. 각 쌍을 같은 분에 교대로, `build-slot` 경유 1잡씩 돌렸다. 진도 정책 키 300초(census `progress()` 와 같은 인자 · 판정은 census 의 `fingerprint`/`progressCurve` 를 그대로 잘라 썼다).

| sha12 | 막힌 지점(전) | 다음 지점(후) | 진도 300초(전 → 후) |
|---|---|---|---|
| `73f3a21e981c` LGT | 로고 뒤 `PrintStream 27` 로 게임 스레드 사망 · paints 107 | 시작 메뉴 위 «서버 접속 실패» 대화상자 · 그 OK 키에서 `NullPointerException at c_bf.keyNotify` · paints 7,673 | stuck(새 화면 1) → stuck(새 화면 4 · 110초까지) |
| `be08d047cbae` LGT | 안내 화면 · `c vtable index 11` 로 사망 | 선택 화면(«…골라봐»). 키는 `ShellCard.keyNotify` 까지 닿는데 화면이 변하지 않는다 | stuck(1) → stuck(1) |
| `d448aee68157` KTF | 매장 이름 입력 뒤 `Field xI` Fatal | 본편 집 안(먹이 고르기 화면) | stuck(새 화면 5 · 60초까지) → **ok**(새 화면 10 · 260초까지) |
| `01e2715ba07a` KTF | — (①: 고칠 것 없음) | 같다 | stuck(6) → stuck(6) |
| `9789fec50f39` KTF | — (①: 사인은 null `Image`) | 같다 | stuck(1) → stuck(1) |

- **`d448aee68157` 600초**(축 정의와 같은 길이 · rebase + `getMaxLength` 바이너리): PASS · 새 화면 11 · 마지막 새 화면 290초 ⇒ **stall 310초 = stuck**. 본편 방 화면(잠든 펫)에서 정책 키로는 더 변하지 않는다 — ⒜ 로 본다. 300초 표의 `ok` 는 길이가 짧아서 나온 값이다.
- rebase 뒤(#444 포함) 120초 재측: `73f3a21e981c` 같은 대화상자 · `be08d047cbae` 같은 선택 화면 — 위 표와 같다.
- ★세 종 모두 진도 축으로는 아직 `stuck` 이다. 벽 하나를 넘은 뒤 다음 자리에서 멈췄다.

### 5. 퇴행

- 30초 A 프로브(census 와 같은 인자 · `--pacing 8 --relaunch 1`): 라이브 LGT 5(`13d7e3c21856` `1b107b96bf4e` `4ece6eeeaa04` `a30bbe008b5e` `b475b6399684`) + 가드 2(`49ade89578c5` `ddd885583b15`) + `739c7657c1f2` + 대상 5 = 13종, 전·후 각 1판. **FAIL 0 → 0**, `content` 13/13 → 13/13. 판정은 PASS 1 → 3(`73f3a21e981c` `be08d047cbae` `b475b6399684`), 나머지는 양쪽 모두 `UNMEASURED · stop max-ticks` 다(census A 인자에는 `--max-ticks` 가 없어 기본 5천만에 닿는다 — 전·후 같은 조건). 그림 수 차이는 load1 31~124 범위에서 ±15% 이내다.
- 러너 블록: draw · helloworld ×2 · text PASS. keydraw ×2 는 release 에서 `UNMEASURED · max-ticks` 이고, `--max-ticks 1e9` 이면 PASS 27/27 이다(0393 과 같다 · 이 diff 무관).
- 게이트(rebase 뒤): `cargo fmt --check` · `cargo clippy --all -D warnings` · wasm32 clippy · `cargo +beta clippy --all -D warnings` 모두 rc=0. `RUST_MIN_STACK=4194304 cargo test --all` 51 스위트 649 통과 0 실패. `npm run build:wasm` rc=0. `check-engine-contract` 113 pass / 0 위반.

### 6. 되돌리면 red

| 수정 | 시험 | 되돌림 결과(실측) |
|---|---|---|
| 게스트 `new` 루트 | `wie-lgt` `a_collection_keeps_what_a_guest_new_allocated` — 게스트 `new` 로 만든 `Stack`, 프레임을 닫고 수집, push/pop | `forget` 줄 주석 → FAILED `the guest's Stack still points at elementData` |
| `getChars` 12 · `println(Z)` 27 · `TimerTask.cancel` 11 | `abi_rows_cover_the_indexes_titles_actually_dispatch_on` 행 3개 | `getChars` 행 삭제 → FAILED `java/lang/String vtable index 12 is empty` |
| lwc `x y w h` | `wie-wipi-java` `position_and_size_fields_resolve_on_a_subclass_and_match_the_getters` | `x` 줄 삭제 → `NoSuchFieldError` |
| `getMaxLength` · `setMaxLength` 보관 | `get_max_length_answers_what_set_max_length_kept` | `setMaxLength` 저장 삭제 → `left: 0 right: 5` |

### 7. 측정 조건 — 지키지 못한 것

- `host-load-guard --status` 는 각 실행 직전 rc=0 이었다(아니면 60초 대기 · 실행 스크립트에 넣었다).
- ★**census 락은 잡지 않았다.** 회차 내내 다른 레인의 `--only progress` 실행(55종 × 600초 · jobs 3)이 `/tmp/wie-playability-census.lock` 을 쥐고 있었다. 그래서 `census.mjs` 를 쓰지 않고 `build-slot` 경유 1잡 직렬로 돌렸다. 여섯 축 전수(census `run`)는 그 락이 필요해 돌리지 않았다. 위 표의 축은 A 프로브(boot·render·input)와 진도뿐이다.
- 전/후 진도 표는 300초 단판이다. 축 정의(600초 · stuck 은 짝 재측)와 같은 조건으로 잰 것은 `d448aee68157` 600초 1판뿐이다.

### 8. compat.json — 바꾸지 않았다

진도 축이 바뀐 행이 없다. 600초로 잰 `d448aee68157` 은 `stuck` 이고, 나머지 둘은 300초에서도 `stuck` 이다. `status`·여섯 축은 이번 회차가 재지 않았다(§7). 게임별 `changes` 는 `docs/player-updates/2026-10-02-{logo-wall-printstream-getchars,notice-wall-timer-cancel,name-entry-text-box}.json` 에서 빌드 때 파생된다(계약 §2).

### 9. 게임 이름 유입

`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` 결과(10파일): 유입 39쌍(BOUNDED) · 판단 필요 6쌍(SUFFIX-ATTACHED). **이 diff 가 새로 들인 이름은 0이다.** 두 바구니 모두 이 diff 가 건드린 파일(`lgt_java_abi.toml`·`jvm_support.rs`·`interface.rs`·`text_component.rs`)에 원래 있던 이름이다. 새로 쓴 주석·문서·소식은 sha12 만 쓴다. 고친 `text_component.rs` 필드 주석 한 줄의 이름은 sha12 로 바꿨다. `git diff origin/main` 의 코드 `+` 줄에서 3자 이상 한글 연속은 0건이다. 문서의 «게임시작»·«…골라봐» 는 화면 문구다.

<!-- corpus-name-inflow v1 subjects=10 tree=a96dfb8edeb6ba28 B=118/39 P=2/1 S=16/6 -->
