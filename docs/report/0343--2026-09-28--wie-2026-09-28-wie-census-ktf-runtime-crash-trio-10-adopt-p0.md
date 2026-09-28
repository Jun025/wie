## [2026-09-28] KTF catch 블록의 `e` 가 0 — 한 paint 안 NPE 무한 포획의 원인은 에뮬레이터 (wie-2026-09-28-wie-census-ktf-runtime-crash-trio-10-adopt-p0)

**무엇을**
- `JavaMethod::handle_exception` 이 게스트 catch 로 되감기 직전, 던져진 예외 객체를 예외 핸들러 레코드의 `+0x10` 칸(`wipi_types` 이름 `unk3`)에 쓴다.
- 시험 `test_catch_handler_receives_thrown_exception` — 예외 표 1행(0..10 → 0x1dd)과 등록된 핸들러로 되감기 결과와 그 칸 값을 본다.

**왜 — 무엇이 null 이었나(계측)**
`java_throw` 에 임시 계측(레지스터·스택 덤프, 커밋 안 함)을 달고 `e9fac881e602` 를 무작위 키 900개(시드 2)로 돌려 재현했다(211번째 키 · 약 140초).
- **첫 NPE**: 호출 지점 `0x128068`(client.bin) — `c.paint(Lorg/kwis/msp/lcdui/Graphics;)V` 의 try 본문에서 `this.ay`(필드 `ay:Lq;`, 오프셋 0x2c) 를 읽어 그 위에 invokevirtual 을 부른다. `ay` 가 null 이라 런타임 헬퍼 `0x16fed0` 이 NPE 를 던진다. 이 한 번은 게임의 try/catch 가 받도록 되어 있다(`paint` 의 예외 표 1행: `0..0x1da → 0x1dd`).
- **그 뒤 43,000여 회**: 전부 호출 지점 `0x1281ba` — catch 블록(`target 0x1dd`)이 핸들러 레코드 `+0x10`(스택 `sp+0x24`)을 `e` 로 읽어 가상 메서드를 부른다. 그 칸이 **0** 이라 두 번째 NPE 가 난다. catch 블록은 `current_pc` 를 옮기지 않으므로 여전히 try 범위(`< 0x1da`) 안이고, 같은 핸들러가 다시 잡는다 → 끝없이 반복.
- 그 칸을 쓰는 게스트 코드는 client.bin 전체에 **0곳**이다(핸들러 등록 `0x171698` 은 `ptr_method`·`ptr_this`·`old_handler`·`current_pc=0`·`ptr_functions` 만 쓴다). 실기에서는 단말 런타임의 throw 가 채웠고, 에뮬레이터가 그 자리를 대신하면서 빠뜨렸다. 상류(`dlunch/wie` main)도 같다.
- 버린 가설: GC 가 살아 있는 객체를 수거 — 수거는 필드를 null 로 만들지 않는다. 스텁 null 반환 — 반복 NPE 의 receiver 는 스텁 반환값이 아니라 핸들러 레코드 칸이다.
- **`ay` 가 왜 null 인지는 특정하지 못했다.** 게임이 바로 그 경우를 `paint` 전체 try/catch 로 감싸 두었으므로, 이 수정 뒤에는 한 번 잡히고 넘어간다(아래).

**사용자 영향**: 몇 분 플레이 뒤 멈추던 두 판이 멈추지 않는다.

### 전/후(같은 부하 · 동시 실행 · `--inject --boot-secs 6 --keys <무작위 900 · 0.66초>` · release)
| sha12 | 전(main `1dd9f81c`) | 후 |
|---|---|---|
| `e9fac881e602` 시드 2 | **FAIL** 305번째 키 `tick error`(예약 OOM) · `java_throw` **43,288** | **PASS** 600초 · 900/900 · paints 8,368 · `java_throw` **5**(세 차례, 각각 잡히고 진행) |
| `c94d64777926` 시드 2 | PASS 600초(이 부하에선 그 장면에 닿지 않음 — 재현은 타이밍 의존) | — |

`c94d64777926` 과 `e9fac881e602` 의 client.bin 은 재배치 후 **바이트 동일**(`wie-ktf-dump` 산출물 `cmp`)이다 — 같은 코드 경로.

### 퇴행(전/후 동시 · `--inject` 기본 예산 · release)
KTF working 190개 중 25번째마다 8종:
`1065985081fc` PASS→PASS · `1793f87924d4` PASS→PASS · `8801ab57a0ee` PASS→PASS · `dbd078113b97` PASS→PASS ·
`5bff8d2168f7` UNMEASURED→PASS · `0e72b6bc12bb` `e09aca27c132` `fd3bf717db21` UNMEASURED→UNMEASURED(양쪽 다 키가 덜 전달됨 — 부하).
PASS 가 FAIL 로 간 것 0. 러너 줄(draw·helloworld×2·keydraw×2 `--inject --expect-last-frame`·text) 전부 PASS · keydraw rc=0.

### 변이(되돌리면 red)
- 새 `write_generic` 을 지운다 → `test_catch_handler_receives_thrown_exception` red(`the catch block reads e from this slot`).

### 한계
- catch 가 **다른 프레임**에서 잡혀야 하는 경우(현재 핸들러 표에 맞는 행이 없으면 `ptr_old_handler` 로 올라가지 않고 곧장 호스트 오류)는 이번 범위 밖이다 — 이 게임 경로는 같은 프레임에서 잡는다.
- `ay` 의 null 출처 미특정(위).

게임 파일명 유입(`corpus-name-inflow --corpus ~/work/otterpebble/wie/game_lab`): BOUNDED 1회/1쌍 · SUFFIX-ATTACHED 0회/0쌍. BOUNDED 1회는 수정 파일 `jvm_support.rs` 의 기존 줄(main 에 이미 있음)이다 — 이 회차의 추가 줄 중 일치 0.

<!-- corpus-name-inflow v1 subjects=5 tree=c6fa5c7c564e8999 B=1/1 P=0/0 S=0/0 -->
