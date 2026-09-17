## [2026-09-17] base swap 이 남긴 고아 파일을 전수로 셌다 — **4건**이고, **3건은 되살리면 red** 다(추정 아님) (wie-adopt-orphaned-files-after-base-swap-census)

**무엇을**: 채택 제안 `2026-09-17-revive-orphaned-preload-guard#p0` 의 이행.
★**세는 것이 산출물**이고 계약이 「되살리는 것은 **하나**」로 못박았다 ⇒ `git mv` **1건** · 삭제 **0** · 경로 재배치 **0** · 새 검사기 **0**.

**왜**: 2026-09-16 base swap 이 크레이트 디렉터리를 하이픈으로 개명하면서 **언더바 디렉터리가 남았다**.
직전 회차가 그중 «검사» 하나를 살렸지만, ★**나머지가 몇 개이고 어떤 상태인지는 아무도 세지 않았다.**

**사용자 영향**: 없다(제품 코드 0줄). 바뀐 것은 ★**J2ME 게스트 부팅 검사가 `cargo test --all` 에서 «돈다»**는 것이다.

### 1. 술어와 그 술어가 «틀리는» 두 자리
제안이 준 술어 = 「workspace member 가 아닌 디렉터리의 `.rs`」. 그대로 쓰되 ★**두 군데를 고쳐야 했다**.
```
소유자 = cargo metadata --no-deps 의 각 패키지 manifest 디렉터리
모집단 = git ls-files 의 *.rs        →  316건 · member 소유 302 · ★member 밖 14
```

| member 밖 14건 | 수 | 판정 |
|---|---|---|
| 루트 패키지 `wie`(`src/*.rs` 6 + `src/main.rs` 포함 · `tests/font.rs`) | **7** | ★**고아 아님** — 소유자가 `.` 이라 단순 술어가 놓친다. cargo 가 `lib`/`bin`/`test` 타깃으로 **본다** |
| `wie-app/` | **3** | ★**고아 아님** — `Cargo.toml` 이 **exclude 를 주석으로 정당화**한 의도된 예외(gtk/webkit2gtk 뿌리) |
| ★**개명 잔재** | **4** | ★**진짜 고아** |

★**제안 자신이 이 함정을 예고했다**(「«고아»의 정의가 애매하면 오탐이 난다 — `wie-app` 이 그렇다」). 실제로 났고, **둘**이었다.

### 2. 진짜 고아 4건 — ★**cargo 타깃 0건**으로 「있는데 안 돈다」를 기계로 확정
| 파일 | 짝 member | cargo test 타깃 | ★되살리면? (실측) |
|---|---|---|---|
| `wie_j2me/tests/test_boot.rs` | `wie-j2me` | **0** | ★**green — 2 passed** ⇒ **이번에 되살린 하나** |
| `wie_jvm_support/tests/absent_string_buffer_insert.rs` | `wie-jvm-support` | **0** | ★**red `E0061`** — `this method takes 5 arguments but 4 were supplied` |
| `wie_jvm_support/tests/absent_timer_schedule.rs` | `wie-jvm-support` | **0** | ★**red `E0432`** — `unresolved import java_class_proto` |
| `wie_midp/tests/create_image_missing_name_message.rs` | `wie-midp` | **0** | ★**red `E0061`** |

★**「전부 되살리면 red 폭풍」이 실측으로 참이다 — 4건 중 3건.** 그리고 ★**추정하지 않았다**:
각각을 제 하이픈 크레이트로 **복사해 실행**하고 제거했다(커밋 0).
★**셋의 원인이 «다르다»**: `E0061` 둘은 상류 `invoke_virtual` 인자 드리프트(직전 회차가 같은 한 줄로 고친 그것) ·
`E0432` 하나는 **의존 미선언**(빌드 배선). ⇒ 처방이 갈리므로 **한 티켓으로 묶으면 안 된다**(후속 제안에 적었다).

### 3. 되살린 1건 — ★**「돈다」와 「잡는다」를 «둘 다» 보였다**
`git mv wie_j2me/tests/test_boot.rs → wie-j2me/tests/test_boot.rs` · ★**파일 내용 0줄 수정**
(`include_bytes!("../../test_data/draw_j2me.zip")` 의 상대 깊이가 같아 그대로 해결된다).

**돈다**: `cargo test -p wie-j2me --test test_boot` → `2 passed; 0 failed` ·
cargo 타깃 조회 `test_boot` **0 → 1** · 통합테스트 타깃 총 **7 → 8**.

**잡는다**(★개악 대조 · **제품 호출부**):
```
wie-j2me/src/emulator.rs   fn tick() { self.system.tick() }  →  Ok(())   // 프레임 0장 합성
  개악 → ★2 FAILED (rc=101 · tests/test_boot.rs:74 · :134 패닉)
  복원 → ★2 passed        (복원본은 git show HEAD:… 와 바이트 동일)
```
★**이 개악이 그 파일 머리주석이 지목한 «그 갈래»다** — 「아무것도 안 던졌다」는 **여전히 참인데**
«프레임이 한 장도 안 나온» 형상, 즉 2026-09-04 사고의 모양. 페인트 **카운트**로 단언했기에 잡힌다.

### 4. 회귀 — 전건 합산(★`tail` 미사용)
4게이트 + `cargo +beta clippy` **전부 rc=0** · `cargo test --all` **rc=0** =
★**44타깃 · 388 passed · 0 failed · 0 ignored**(부활 전 **43 / 386** 대비 ★**+1 타깃 · +2 테스트**) ·
node 검사기 **8/8 rc=0** · `npm run audit` **PASSED**.

### 남긴 한계
⑴★**검사기를 만들지 않았다** — 제안의 tradeoff 가 예고한 오탐이 **이 회차에서 실제로 났다**(루트 7 · `wie-app` 3).
예외를 **어디에 선언할지**가 먼저이고, 그 결정 없이 검사기를 만들면 오탐이 굳는다.
⑵★술어는 **`.rs` 만** 본다 — 개명 잔재가 다른 형태로 남았으면 못 본다. ※이번 3개 언더바 디렉터리에는 `.rs` 외 추적 파일이 **0건**이라 «이번에는» 빈틈이 없었다.
⑶남은 3건은 각각 **코드 수정**이 필요하고 그것은 별 회차다(계약 3).
