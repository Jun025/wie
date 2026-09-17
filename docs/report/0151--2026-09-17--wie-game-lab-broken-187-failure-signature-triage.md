## [2026-09-17] 헤드리스 검증기가 글자를 그리는 순간 스스로 패닉했다 — `broken/` «표본» 최대 서명이 게임 버그가 아니었다 (wie-game-lab-broken-187-failure-signature-triage)

### 무엇을

`wie_cli/src/bin/wie_validate.rs` 의 `HeadlessPlatform::font()` 가 `unimplemented!()` 였다. 게스트가
텍스트를 그리는 순간 **게임이 아니라 검증기가** 패닉하고, `game_lab/classify.sh` 는 그 결과를 해당 게임의
`broken/` 분류로 기록한다. 브라우저 호스트(`wie_featurephone/src/platform.rs:58`)가 이미 쓰는
`assets/neodgm.ttf` 를 같은 방식으로 실어 그 칸을 채웠다. **1파일 · +16/−3 · 제품 엔진 로직 0줄.**

### 왜

운영자가 `game_lab/` 에 코퍼스를 배치했고 분류가 돌아 `broken/` **187** · `working/` **294** 가 쌓였는데,
「187 이 몇 가지 원인인가」를 아무도 오늘 기준으로 세지 않았다. 세어 보니 **최대 군집이 게임 결함이 아니라
하니스 결손**이었다 — 부팅해서 **31~135 프레임**을 그린 게임이 첫 문자열에서 broken 으로 접혔다.

**A/B 실측**(같은 3게임 × 3런 · 직렬 · 동일 인자):

| | `panic … : not implemented` | PASS |
|---|---|---|
| 수정 전 | **8 / 9 런** | 1 / 9 |
| 수정 후 | **0 / 9 런** | 8 / 9 |

★**판정 축을 «서명»으로 잡은 이유**: 같은 트리·같은 게임에서 PASS/FAIL 이 부하에 따라 뒤집힌다
(이 저장소가 `AGENTS.md` §Definition of Done 에 「past that ceiling the verdict is not invariant」로 이미
기록한 그 현상). 실제로 되돌림 대조에서 `엑스맨`·`(KTF)투스워즈` 는 **수정 없이도 PASS** 했다. ⇒
「고쳤더니 PASS 가 됐다」는 **안전한 주장이 아니고**, 「그 패닉 서명이 사라졌다」는 안전하다.

### 사용자 영향

`broken/` 로 분류돼 있던 게임 여러 개가 실제로는 **부팅·렌더·입력 시퀀스를 완주**한다. 재검증에서
`MBC무한도전`·`엑스맨`·`투스워즈`·`(KTF)투스워즈`·`탁재훈신맞고2007`·`2006영혼낚시` **6타이틀**이 PASS 로 관측됐다.
★**다만 「6게임이 고쳐졌다」로 읽지 마라** — 고친 것은 «검증기»이고, 그 게임들이 실제로 플레이 가능한지는
이 회차가 판정하지 않았다(`visual correctness NOT checked` 는 이 도구가 스스로 붙이는 문구다).

### 한계 — 숨기지 않는다

- ★**기존 `reports/` 1,312건은 «오늘 트리를 설명하지 않는다».** 그 데이터는 `rustjava rev c66f08d`(git 핀)
  시절 것이고 오늘은 base swap 이후 crates.io `rustjava-runtime ^0.1.1` 이다. 15게임 표본 재실행에서 런타임
  메시지 형식까지 달라졌다(`java/lang/…` → `java.lang.…`). ⇒ 이 회차의 서명 표는 **7월 데이터의 표**이고,
  오늘 기준 전수 재census 는 **하지 않았다**(187 직렬 재실행 비용).
- ★`working/` 표본 5건 중 LGT **2건이 FAIL** 했는데, 되돌림 대조에서 **수정 없이도 같은 FAIL** 이거나
  (`레이카르나`) **수정 없이 PASS**(`일지매영웅전기2`)였다 ⇒ **이 변경이 만든 회귀가 아니다**(부하 flake + 오늘 base 의 LGT 상태).
- ★남은 14개 버킷은 **표로만** 남겼다 — 군집당 티켓 1장이 다음 입력이다.

### ★[게이트② 반려 승계 · 2026-09-18 `-fix`] 고친 두 문면

- ★★**이 수정에는 «자동 회귀 가드»가 없다 — 그것이 정직한 값이고, 적지 않은 것이 결손이었다.**
  `font()` 를 `unimplemented!()` 로 되돌린 트리에서 **러너 블록 5종이 전건 PASS·rc=0** 이다
  (`draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`keydraw_ktf`·`keydraw_lgt` · 뒤 둘은 `content true`·`last true`·paints 55).
  ★**그 트리가 «고장나 있었다»는 증거는 따로 있다** — 같은 빌드로 `game_lab/broken/ktf/(KTF)투스워즈` 는
  **2/2런 `panic during 'boot': not implemented`** 였고, 복원 후 같은 게임은 PASS 다. ⇒ 개악은 살아 있었고 **러너가 못 본 것**이다.
  ★**근인 둘**: ⒜커밋된 픽스처 **6개 중 «문자열을 그리는» 것이 0개**(zip 원문·전개 양쪽 `drawString` **0/6** ·
  생성기 3종 `make-draw-fixture.mjs`·`make-resize-fixture.mjs`·`make-wipi-keydraw-fixture.sh` 텍스트 API **0건**)
  ⒝`cargo test --all` 이 쓰는 `TestPlatform` 은 ★**이미 진짜 폰트를 든다**(`test-utils/src/platform.rs:139`)
  ⇒ 다른 impl 인 `HeadlessPlatform` 의 결손과 **구조적으로 만나지 않는다**. 유일한 감시자는 `.gitignore` 된 `game_lab/` 이고 **CI 에 없다**
  ⇒ ★**이 결손이 두 달을 산 이유가 그것이다.** 진짜 처방(문자열을 그리는 픽스처 1개)은 **제안 카드**로 넘겼다.
- ★★**worklog `proposals[1].why` 의 「남은 `unimplemented!()`」 목록이 틀렸었다 — 재측해 교체했다.**
  술어를 먼저 적는다: ★**주석·문자열 리터럴을 지운 뒤에도 매크로가 남고, `test` 를 언급하는 `#[cfg(...)]` 항목의
  «중괄호 범위 밖»일 것.**(파일의 `cfg(test)` 줄 «위»라는 근사는 모듈이 여럿이면 깨진다. 중괄호는 ★**리터럴을 지운 사본**에서 세야 한다 —
  `unimplemented!("Unsupported pixel format: {bpp}")` 가 정확히 그 함정이다.)

  | 자리 | 매크로 | 판정 |
  |---|---|---|
  | `wie-backend/src/system/event_queue.rs:60` | `unimplemented!("…")` | ★**생산 경로** |
  | `wie-midp/src/classes/javax/microedition/lcdui/image.rs:211,225` | `unimplemented!("…")` | ★**생산 경로** |
  | `wie-wipi-c/src/api/graphics/framebuffer.rs:89,105` | `unimplemented!("…")` | ★**생산 경로** |
  | `wie-core-arm/src/core.rs:465` | ★`todo!()` | **생산 경로**(`cfg(test)` 는 `:677`) |
  | `wie-backend/src/system/file_system.rs:233,236,242,248` | `unimplemented!()` | ✕ `#[cfg(test)]` **:160-417** |
  | `wie-wipi-c/src/context.rs:155,163,179` | ★`todo!()` | ✕ `#[cfg(test)]` **:82-198** |
  | `wie-wipi-c/src/api/graphics.rs:77,78` | `todo!()` | ✕ **주석 처리된 죽은 줄** |

  ⇒ 전수 **13 = 생산 6(`unimplemented!` 5 + `todo!` 1) · test 게이트 7 · 주석뿐 3**(세 번째는 `wie_validate.rs:351` 산문 포함).
  ★★**근인은 «검색어»다** — `unimplemented!()`/`todo!()` 라는 **빈 괄호 리터럴**로 찾으면 종전 목록의 네 파일이 **정확히** 나오고,
  ★진짜 위험한 다섯은 전부 **인자를 가진** `unimplemented!("…")` 라 그 검색에 **원리적으로 걸리지 않는다**.
  ⇒ 한 줄의 검색어가 ⑴매크로 이름을 섞고 ⑵`cfg(test)` 를 못 보고 ⑶가장 위험한 형태를 통째로 놓쳤다.
