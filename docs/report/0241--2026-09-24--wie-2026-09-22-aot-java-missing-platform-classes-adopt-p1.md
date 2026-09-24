## [2026-09-24] `Jlet.getCurrentJlet()` — `getActiveJlet` 의 별명임을 KEmulator 바이트코드로 세우고 위임했다 (wie-2026-09-22-aot-java-missing-platform-classes-adopt-p1)

채택 제안 `2026-09-22-aot-java-missing-platform-classes#p1` 의 판정 회차다. ★**판정 = «별명이다» — 근거가 섰으므로 구현했다.**

### ⒜ 벽은 현 main 에도 있다

`origin/main` `4dea6c03` · `wie_validate --timeout 15` 간호사타이쿤2(LGT):
`FAIL · stop error · ticks 2 · paints 0` — `Unable to resolve non-virtual method org/kwis/msp/lcdui/Jlet.getCurrentJlet()Lorg/kwis/msp/lcdui/Jlet;`.
«non-virtual» 이므로 게임은 이것을 static 으로 해결한다.

### ⒝ 근거 — 코퍼스 안의 유일한 구현체

`game_lab/` 전체 zip·jar(2단 중첩까지) 바이트 검색: `getCurrentJlet` 을 담은 것 **300**, `getActiveJlet` **299**, 둘 다 **299**.
그중 게임 쪽은 전부 ARM(`client.bin*`·`binary.mod`)이라 호출 형태를 바이트코드로 읽을 수 없다.
★**바이트코드로 읽히는 것은 `vendor_sdk/ezi` 의 KEmulator jar 둘(`KEmulator-mmpp.jar`·`kemulator-debug.jar`)뿐이고, 둘의 `Jlet.class` 는 md5 `cf8299cd…` 로 동일(= 1개 출처)** 이다. 디스어셈블:

```
field  0xa  activeJlet : Lorg/kwis/msp/lcdui/Jlet;      (private static)
method 0x9  getActiveJlet  ()Lorg/kwis/msp/lcdui/Jlet;   getstatic activeJlet; areturn
method 0x9  getCurrentJlet ()Lorg/kwis/msp/lcdui/Jlet;   getstatic activeJlet; areturn
method 0x9  getJletFromPID (I)Lorg/kwis/msp/lcdui/Jlet;  getstatic activeJlet; areturn
```

⇒ `getCurrentJlet` 은 `public static` 이고 본문이 `getActiveJlet` 과 **바이트 동일**하다. 게임이 static 으로 해결한다는 ⒜와도 맞는다.
★**한계**: KEmulator 는 에뮬레이터이지 실기가 아니다 — 출처 1개. 실기 ez-i 가 둘을 다르게 구현했다면 이 근거는 그것을 못 본다.

### 구현

`wie-wipi-java/.../lcdui/jlet.rs` 에 `getCurrentJlet` 등재 1건 — 기존 `get_active_jlet` 에 위임(새 함수 0). `getJletFromPID` 는 부르는 게임을 보지 못해 넣지 않았다.

### before / after · 양방향 변이

| | result · stop | ticks | paints | 벽 |
|---|---|---|---|---|
| before (main) | FAIL · error | 2 | 0 | `getCurrentJlet` 해결 실패 |
| after ×2 | FAIL · deadline | 1014 / 1120 | 0 | `no frame rendered` |
| 변이(등재 제거) | FAIL · error | 2 | 0 | 원래 벽 그대로 |

단위 테스트 `get_current_jlet_is_get_active_jlet`: 원본 ok · 등재 제거 시 `NoSuchMethodError` 로 FAILED.
after 에서 게임은 `DataBase.openDataBase`·`RecordStore`·LWC 컴포넌트까지 가고 한 프레임도 그리지 않는다 — 다음 벽은 no-frame 계급이다(이 회차 범위 밖).

### 게임 파일명 유입

`node scripts/corpus-name-inflow.mjs --corpus ~/work/otterpebble/wie/game_lab`(모집단 stem 451 · 대상 3파일):
**BOUNDED 9회/4쌍 · SUFFIX-ATTACHED 9회/4쌍 · PREFIX 0**(이 절 자신의 언급 2회 포함). BOUNDED 는 전부 대상 타이틀 이름 `간호사타이쿤2` 자체(0238 이 월드장기체스를 적은 것과 같은 계급 — 이름이지 바이트가 아니다).
SUFFIX-ATTACHED 9건은 손으로 갈랐다: 전부 stem `간호사타이쿤` 뒤에 `2` 가 붙은 같은 9곳 = «더 긴 다른 제목»이고 그 제목이 위 BOUNDED 다 — 새 이름 0.

<!-- corpus-name-inflow v1 subjects=3 tree=151173856796a172 B=9/4 P=0/0 S=9/4 -->
