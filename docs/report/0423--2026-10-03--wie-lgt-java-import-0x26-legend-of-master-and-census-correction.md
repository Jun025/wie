## [2026-10-03] LGT import 0x26 = ClassCastException · 배열 `checkcast` 의 대상이 Class 객체로 온다 — 735a579d82ac 필드 진입 (wie-lgt-java-import-0x26-legend-of-master-and-census-correction)

**무엇을**: ⑴LGT Java 시스템 import `0x26` 을 `RaiseClassCastException`(SVC id 38)으로 등재했다. ⑵import `0x12`(`IsClassAssignable`)가 대상 인자로 **`java/lang/Class` 객체**를 받으면 그 이름으로 판정한다(종전: 언제나 문자열로 읽었다).
**왜**: 735a579d82ac(LGT · aot-java)가 스토리 뒤 필드에 들어가면 `Fatal error: Unknown lgt java import: 0x26` 로 죽었다(otterpebble#1285 게이트② 권고 1). 그런데 0x26 만 넣으면 **죽는 대신 예외가 쏟아진다** — 아래 «진짜 원인».
**사용자 영향**: 필드에서 인물이 그려지고 이야기가 이어진다. 공개 데이터는 이 행에 진도 축 `ok` 를 더했다(나머지 축은 이미 `ok` 였고, 이제 그것이 필드 너머에서도 참이다).

### 0x26 판정 — binary.mod(ELF · **ARM** · 기호 없음)
- 이 타이틀은 Thumb 이 아니라 ARM 이다(`--triple=armv5te` 로만 읽힌다).
- 트램폴린 `{str lr,[sp,#-4]!; bl resolver; 0x64; 0x26}` = `0x1406a48`. `.text` 리터럴 참조 **78곳**, 전부 같은 모양:
  `x = …; if (x != null) { cls = x->vtable[0]; if (!IsClassAssignable(cls, T)) import_0x26(); }` — `cmp r0,#0` 직후 호출이라 **r0 = 0 그대로, 인자 없음**. ⇒ `checkcast` 실패 = `ClassCastException`. 메시지는 비운다(대상 클래스가 넘어오지 않는다).
- 이웃 번호와도 맞는다: `0x22` NPE · `0x23` AIOOBE · `0x25` ArithmeticException.

### 진짜 원인 — 실패해서는 안 되는 cast 였다
0x26 만 등재한 판(임시 계측, 커밋 안 함)으로 OK×40(간격 3초)을 돌리자 `ClassCastException` **744건**, 전부 한 호출부(`0xe3050`)에서 `[C` 를 **빈 이름**과 비교했다.
- 그 호출부는 130개 `IsClassAssignable` 호출 중 **유일한 배열형**이다: `r1 = GetArrayType(1, 0, 5)`(import `0x0e` · 5 = char) 의 **반환값을 그대로** 넘긴다. 나머지 129곳은 `T.descriptor.name` 문자열을 넘긴다.
- 이 런타임의 `GetArrayType` 은 `java/lang/Class` 객체를 돌려주고(`InstantiateArray` 0x10 이 그것을 받는다 — 243곳, 일관), 그 객체를 문자열로 읽으면 ""다. ⇒ `(char[]) x` 가 **언제나** 실패했다.
- 처리: `LgtJvmSupport::class_object_at` 이 그 워드가 `java/lang/Class` 인스턴스인지 **조용히** 본다(문자열 포인터면 클래스 읽기가 실패하거나 이름이 다르다 — `object_from_raw` 를 쓰지 않은 이유는 매 호출 warn). 맞으면 `JavaLangClass::name` 으로 판정, 아니면 종전대로 문자열.

### 실측 — release `wie_validate`, 전(`bfc82215` = origin/main)·후 동시 짝
| | 전 | 후 |
|---|---|---|
| OK×40(3초) `--shotdir` | (0x26 만 넣은 판) PASS · 예외 744 · 고유 화면 17 · **끝 10장 1종** — 필드에 인물 없음, 정지 | PASS · 예외 0 · 고유 37 · **끝 10장 10종** — 인물 · 대사 진행 |
| census L(필드 레시피 접두 + 600초) | **FAIL error** · 키 16에서 `Unknown lgt java import: 0x26` | UNMEASURED deadline(722/940) · 예외 0 · 타이머 샷 29 중 **28종** |
| census 프로브 A·B(30초) | A 3종 · B 2종 | A 5종 · B 2종 — 두 판 모두 input `ok` |
| census L(정책 루프 · 레시피 없음) | (bd2337ff 판: 29장 3종) | 29장 3종 — 타이틀 메뉴를 맴돈다 |
| census 진도(정책 키 · 600초) | (L 실패로 대상 아님) | **ok** · 754/754 · 71장 59종 · NPE 6(게임이 처리, 실행 PASS) |

- ★**종전 `input`·`longplay ok` 가 왜 «틀린 ok» 였나**: 정책 루프 L 은 타이틀 메뉴(게임문의 화면 등)를 벗어나지 못해 필드에 닿지 않는다 — bd2337ff 판도 29장 3종. 축 정의상 값은 맞았지만 실패 지점을 볼 수 없었다. 필드 레시피를 붙인 L 이 전 판에서 키 16에 죽는 것으로 그것을 보였다.
- 공개 데이터: 레시피로 잰 값은 싣지 않는다(`docs/contracts/featurephone-public-data.md` progress 행). 실은 `progress: ok` 는 **정책 키** 진도 판이다. 레시피 파일은 `game_lab/recipes-progress/` (git 밖).
- 측정 조건: census 호스트 락을 6차 전수 재측(`wie-census-wave6-full-recensus-at-main-all-axes`, 수 시간 점유)이 쥐고 있어 `WIE_CENSUS_LOCK` 을 이 회차 스크래치로 돌렸다 — 단건 · `--jobs 1` · 전 단계 `build-slot run`, 짝 실행이라 동시 2개. `host-load-guard` rc=1(포화) 구간이라 폭을 2 이하로 뒀다. load1 18~40(마지막 진도 끝무렵 100+).
- 6차 회차와의 충돌: 그 회차는 PR 이 없고(핀 `4ac38566`), r2 티켓이 «0x26 티켓의 손 변경을 덮지 마라» 를 이미 적었다 ⇒ 이 행은 이 PR 이 진다.

### 회귀 — import 0x26 보유 LGT 8종 중 나머지 7 + 가드 2
코퍼스 LGT binary.mod 91 중 0x26 보유 8종(대상 포함). 나머지 7(`fd8f52f3abf7` `be08d047cbae` `5d0ba9495592` `13d7e3c21856` `70d709c40e10` `73f3a21e981c` `61ed69520fd3`) + 가드 `49ade89578c5` `ddd885583b15` 를 census `--only probe` 전·후 짝으로: 9종 모두 결과·stop·Java 예외 수 동일, `report` 축(status·boot·render·input·longplay) **diff 0**.

### 시험(되돌리면 red)
`class_assignable_takes_an_array_class_object_as_target_and_0x26_raises_class_cast`: `[C` 인스턴스의 클래스 ↔ `GetArrayType(1,0,5)` = 1 · `GetArrayType(1,0,10)`(`[I`) = 0 · 문자열 `[C`/`java/lang/Object` = 1 · `java/lang/String` = 0 · import 0x26 해석 · 처리기가 `java/lang/ClassCastException` 을 게스트로 던진다.
변이 2건 모두 red(실측): Class 객체 판독 제거 → `left: 0 right: 1` · 표에서 `0x26` 행 삭제 → `Unknown lgt java import: 0x26`. SVC id 왕복 시험의 마지막 id 를 `RaiseClassCastException` 으로 옮겼다.

### 게이트
- 네 게이트 + beta: fmt OK · clippy / wasm clippy / beta clippy rc0 · `RUST_MIN_STACK=4194304 cargo test --all` **51 스위트 666 passed 0 failed**(첫 판은 `target/debug/incremental` 의 `.pre-lto.bc` 결손 ICE — 그 크레이트 증분 디렉터리를 지우고 재실행).
- 러너: draw_j2me · helloworld_ktf · helloworld_lgt · text_j2me PASS · keydraw_ktf/lgt `--inject --expect-last-frame` 기본 max-ticks 에서 `UNMEASURED max-ticks` → `--max-ticks 1000000000` 에서 둘 다 **PASS · rc0**(paints 80 · 55).

게임 이름 유입(`scripts/corpus-name-inflow.mjs --corpus game_lab`): BOUNDED 774/355쌍 · SUFFIX-ATTACHED 40/18 — 전부 대상 파일 «전체»의 기존 문면이다(`compat.json` 의 `title`·`fileTitle` 필드, `interface.rs`·`jvm_support.rs` 의 기존 주석). 이 회차가 추가한 줄에는 게임 이름 0(코드·주석·소식 모두 sha12 만).

<!-- corpus-name-inflow v1 subjects=6 tree=ccbb073f07d6dd3a B=774/355 P=2/1 S=40/18 -->
