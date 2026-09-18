## [2026-09-18] 개명에서 유실된 LGT 도달 시험 2개를 되살렸다 — ★**되살린 것이 «문다»를 개악 2칸으로 보였다** (wie-restore-lgt-key-and-resource-reach-tests-lost-in-rename)

**무엇을**: `wie-lgt/tests/test_key_reach.rs`·`test_resource_reach.rs` **신설 2파일**.
★**제품 코드 0줄** · ktf 쪽 **무접촉** · `contract-roundtrip.mjs`·Scenario F 필터 **무접촉** ·
생성기·`test_data/*.zip` **무접촉** · 새 워크플로 **0** · ★**`STATE.md` 무접촉**.

**왜**: `b52ed661` 이 ktf·lgt 도달 시험을 **둘 다** 만들었는데(그 회차 회신: 「두 경로 다 … 부분 완료가 아니라 **전건**」)
**2026-09-16 base swap 이 ktf 만 데려왔다.**

**사용자 영향**: 없다(시험 코드뿐). `cargo test --all` 이 다리당 **약 4.4 + 3.7 = 8.1초** 길어진다.

### ⓐ 결손을 내가 쟀다 — ★**「일부러 뺐다」는 흔적이 없다**

- `cargo metadata --no-deps` 시험 타깃 전수(8건)에 ★**lgt 도달 시험 0건**(ktf 둘은 실재).
- ★**삭제 커밋이 «없다»** — `git log --diff-filter=D` 가 **빈 출력**이다. base swap 이 트리를 통째로 갈아서
  「지워진」 것이 아니라 **새 트리에 안 들어온** 것이다(마지막으로 존재한 커밋 `c0f48c66`).
- ★**base swap 회차들은 ktf 두 시험을 «이름 대어» 고쳤다**(`8ff418ed` 「test_key_reach 탈출 조건 교정」 ·
  `docs/report/0123` 이 ktf 두 파일을 함께 수정) — ★**lgt 쪽 언급은 0건**.
- ★**뒤이은 고아 인구조사(`docs/report/0153`)의 «진짜 고아 4건»에도 없다**
  (`wie_j2me/tests/test_boot.rs`·`wie_jvm_support/tests/*` 둘·`wie_midp/tests/*`).
  ⇒ 인구조사 시점엔 **파일이 아예 없었다**(고아는 «있는데 안 도는» 것이다).
⇒ ★**판정: 「고의」의 증거가 없고 「누락」의 증거가 셋이다.** ⓒ 는 여기서 끝나지 않는다.

### ⓑ ★「무조건 축 0」을 «정확하게» 고쳐 적는다 — 티켓 문면보다 «좁다»

Scenario F 는 `if: steps.changes.outputs.engine == 'true'`(`engine-contract.yml:391`) 뒤에 있다. 그런데
★**그 필터의 «첫 항목»이 `**/*.rs` 다** ⇒ ★**Rust 를 건드리는 PR 은 전부 F 를 돌린다.**
⇒ 진짜 사각은 「LGT 가 무방비」가 아니라 ★**«비-Rust diff»**(문서·워크플로·설정 전용 PR)다.

★★**그러나 더 큰 비대칭이 따로 있었다 — 그것이 이 회차의 실제 소득이다**:

| 축 | 어디서 도는가 | 조건 |
|---|---|---|
| Scenario F / F-res(종전 유일) | ★**`ubuntu-latest` «하나»**(`engine-contract.yml:63`) | paths 필터 뒤 |
| ★**되살린 시험 2개** | ★**macos · ubuntu · windows × stable · beta = 6다리** | ★**무조건** |

⇒ ★**LGT 키·리소스 도달은 «Windows·macOS 에서 한 번도 검사된 적이 없었다».**

### ⑵ 단언은 «정확한 정수»다 (계약 1⑵)

```rust
assert!(seen.contains("key:53"),
    "NUM5 did not reach the LGT guest as WIPI code 53 — guest stdout was {seen:?}");

const EXPECTED: &str = "res:9:602";
assert!(!seen.contains("res:err"), …);          // 「실패했다」와 「일어나지 않았다」를 가른다
assert!(seen.contains(EXPECTED), "… (size from get_resource_size, sum from read_resource) …");
```
★`> 0` 류 느슨한 단언 **0**. `53` 은 생성기의 `KeyCode::Key5 => 53`(두 캐리어 공용 표) ·
`9:602` 는 `WIE-RES-1` 9바이트의 바이트합이고 ★**두 수를 «함께» 단언해야 `get_resource_size` 와
`read_resource` 가 «둘 다» 덮인다**(크기만 보면 둘째 사이트가 종전처럼 미측정으로 남는다).

### ⑶ ★★부하 내성은 «설계»다 — 관용을 그대로 물려받았다

★**되살린 시험은 «틱 상한» 루프다**(`DELIVER_TICKS`/`BOOT_TICKS = 400_000` · `guest_line_complete` 로 탈출) —
★**벽시계 예산이 아니다.** 원 회차가 본 거짓 FAIL 은 `wie_validate --inject` 경로의 것이고, 그쪽은
**고정 벽시계 deadline** 이라 바쁜 기계가 그것을 먹는다(loadavg 180 에서 `paints` 12~22 ↔ idle 48~55).
★**틱 예산은 그렇게 굶길 수 없다** — 호스트는 기계가 한가하든 아니든 **같은 수의 스텝**을 진행시킨다.
⇒ ★**`paints` 단언을 넣지 않았다**(ktf 헤더가 2026-09-07 에 측정으로 기각한 그것) · 두 파일 헤더에
「타임아웃으로 «개선»하지 마라」를 못박았다.

### 양방향 — ★**«문다»가 합격선이다**(개악은 백업+`trap` · 커밋 0)

| 개악(제품 호출부) | 결과 |
|---|---|
| `wie-lgt/.../wipi_c/context.rs` `get_resource_size` 가 **1 크게** 답한다 | ★**FAILED** — `guest stdout was "res:10:602\n"` · 메시지가 **어느 사이트인지 이름 댄다** |
| `wie-lgt/.../net/wie/clet_wrapper_card.rs` `key_notify` 가 **`key + 1`** 을 건넨다 | ★**FAILED** — `guest stdout was "res:9:602\nkey:54\n"` |
| 원복 후 | ★**둘 다 ok**(`git status` 에 소스 변경 0) |

★**두 개악이 «서로 다른 LGT 고유 경로»다** — 하나는 리소스 API, 하나는 Clet 키 디스패치.

★★**⒞ 부하 대조 — 이 티켓에만 있는 축**:

| 상태 | loadavg | 결과 |
|---|---|---|
| 주변 부하 | 178 | ★**ok** 1.15s / 1.58s |
| ★**인위적 부하 16 스피너 투입** | ★**193** | ★**ok** 2.72s / 2.27s |

⇒ ★**부하에서 느려지되 «뒤집히지 않는다».** 같은 부하대(180)에서 러너 경로는 **FAIL** 한다 —
★**그 대조가 ⑶ 설계의 증거다.**

### 대가 — ★**「잃는 것이 없다」가 아니다**

- ★**모든 PR 이 느려진다**: 두 시험 **약 8.1초/다리** × **6다리**. self-hosted 러너 **1대**라 형제 PR 이 그만큼 기다린다.
- ★**무조건 축이라 flaky 가 되면 «모든 PR 을 막는다»** — 그래서 ⑶ 을 실제로 쟀다. ★그러나 부하 상한을 «찾은» 것은 아니다:
  193 까지 green 임을 보였을 뿐이고 ★**더 높은 부하에서의 거동은 모른다.**
- ★**되살리지 «않는» 쪽의 대가**(비교): LGT 키·리소스 회귀가 ⑴**비-Rust diff 에서 안 잡히고**
  ⑵★**Windows·macOS 에서 영영 안 잡힌다.** ⇒ 두 대가를 나란히 놓고 **되살리는 쪽**을 골랐다.
- ★**Scenario F 를 무조건으로 바꾸지 «않았다»** — 범위 밖이고(계약 3) 그쪽이 훨씬 비싸다(wasm 빌드+playwright).
