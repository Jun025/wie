## [2026-09-26] 배틀몬스터 `class_definition.rs:801` 패닉 — 재현 0/27 · GC 해제 후 사용 가설 반증 · 패닉 문구만 보강 (wie-2026-09-26-class-definition-801-unwrap-panic-adopt-p1)

**무엇을**: 배틀몬스터 필드 이동 중 간헐로 난 `ClassDefinition::name` 의 `try_name().unwrap()` →
`InvalidMemoryAccess(0)` 패닉의 출처를 찾으려 했다. 재현이 되지 않았다. 원인을 모른 채 봉합하지 않았고,
대신 다음 발생이 «class → descriptor → name 중 어느 고리가 끊겼는지» 말하도록 패닉 문구만 바꿨다.

**왜**: 기존 증거(`reports/evidence/wie-battlemonster-continue-save-restores-wrong-position/panic_class_definition_801.txt`)는
`InvalidMemoryAccess(0)` 한 줄뿐이고 백트레이스가 없다. 0 번지 읽기는 세 경우(클래스 포인터 0 · 디스크립터 0 · 이름 포인터 0)가
같은 문구를 낸다 — 그래서 호출부를 좁힐 수 없었다.

**사용자 영향**: 없음(동작 불변). 패닉은 여전히 패닉이다. 문구만
`LGT class 0x… (descriptor Ok(0x…)|Err(…)) has no readable name: …` 로 바뀐다.

### 재현 (0/27)

| 빌드 | 경로 | 판 | 패닉 | 비고 |
|---|---|---|---|---|
| main `5434ab0a` + #317 | 숲 짧은 경로(`OK:15 OK:15 N1:10 N1:50 OK:8×6` · 검수자가 main 에서 `:801` 을 본 `04_N1` 구간) | 10 | 0 | 전부 기존 벽 `StringBuffer vtable index 22` 에서 멈춤 |
| #292 head `2d17aff4` | `keys_field_move.txt`(마을 → 집, UP·RIGHT·LEFT·UP 포함 103 키) | 11 | 0 | 전부 `PASS` · `last_frame_content true` · `first null` |
| #292 head `2d17aff4` | 대화형 ctl 경로(`tovillage.sh` + RIGHT RIGHT UP UP UP) | 6 | 0 | **6/6 이 `h1` 부근에서 `a.run()` / by zero 로 게임 스레드 사망** → 이동 단계에 도달 못 함 |

진단 빌드는 `name()` 실패 시 `ptr_raw`·클래스 워드·`ptr_descriptor` 를 찍게 해 두었다(`diag` 0/27). #317 의
값 코덱 warn(「does not point at a live instance」)도 main 판 10개에서 0 이다. 계수 원문은
`reports/evidence/wie-2026-09-26-class-definition-801-unwrap-panic-adopt-p1/runs.txt`.

### 반증한 가설: `Runtime.gc()` 가 게스트가 쥔 객체를 해제한다

근거가 있어 보였다. 배틀몬스터는 `java/lang/Runtime` vtable 13(= `gc`, `lgt_java_abi.toml`)을 부르고,
rustjava `collect_garbage` 의 루트는 rustjava 스레드 프레임·정적 필드·전역 참조뿐이라 ARM 스택/레지스터에만 있는 참조를 모른다.
해제된 객체의 메모리가 재사용되면 클래스 워드가 0 이 될 수 있다.

측정: `JavaClassInstance::destroy` 에서 해제 주소를 기록하고 (a) 이후 `from_raw` 로 다시 들어오면 경고, (b) 해제 대신
클래스 워드를 `0xDEAD0000` 으로 독살해 게스트가 건드리면 그 주소로 죽게 했다. 5판에서 destroy **12,834~52,844회/판**, 해제 후 사용 **0**,
`0xdead0000`/`3735879680` 출현 **0**. ⇒ 이 경로에서는 GC 가 살아 있는 객체를 해제하지 않는다. 가설 기각.

### 남긴 것

- `class_definition.rs` `name()`: `unwrap()` → 포인터 체인을 담은 `panic!`. 시험 `name_panic_names_the_broken_link`
  (`from_raw(0)` → `LGT class 0x0 (descriptor Err(InvalidMemoryAccess(0))) …`). 이전 코드로는 문구가 달라 실패한다.
- 에러 전파로 바꾸지 않았다 — 티켓 tradeoff 그대로, 출처를 모른 채 바꾸면 원인을 숨긴다.

### 한계

- 원 발생 두 건: #292 대화형 세션 1/2(`InvalidMemoryAccess(0)` · 사람이 친 경로)와 #306 검수자의 main 숲 교대 스크립트 1/2
  (`InvalidMemoryAccess(541552863 = 0x204770df)` · `04_N1`). 뒤엣것은 같은 키의 앞 10개로 main+#317 에서 10판 쟀고 0 이다.
  대화형 쪽은 ctl 경로로 흉내 냈으나 현 #292 head 에서 이동 전에 `/ by zero` 로 멈춰 해당 구간에 닿지 못했다.
- 부하 41~159(loadavg)에서 쟀다. 타이밍 의존이면 부하가 판정을 바꿀 수 있다.
- 재개 조건: 다음 `:801` 발생 로그(새 문구)가 `class 0x0` 인지, `descriptor Ok(0)` 인지, 이름 포인터인지를 알려 주면 그 갈래의 호출부를 본다.
