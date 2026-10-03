## [2026-10-03] 에뮬레이터 해제 시 코어↔JVM 순환 절단 — 타이틀 전환마다 한 대분씩 쌓이던 누수 (wie-lgt-core-jvm-cycle-leaks-emulator-per-title-switch)

0423 검수 §3 의 후속. 에뮬레이터를 놓아도 코어·시스템·JVM 이 서로를 붙잡아 아무것도 해제되지 않았다.

### 무엇을 — 방식과 근거

**`Weak` 가 아니라 해제 시 명시 철거**다. `jvm::Jvm` 은 `Arc<JvmInner>` 하나뿐이고 약한 참조 API 가 없다(jvm 0.1.1
`src/jvm.rs` — `Weak`·`downgrade` 0건). 그리고 순환은 SVC 핸들러 하나가 아니었다 — 하나씩 끊으며 코어 `Arc` 강한 참조 수를 쟀다
(임시 계측 · 커밋 안 함 · `keydraw_lgt --restart-at 2`):

| 끊은 것 | 옛 코어 강한 참조 |
|---|---|
| executor 작업 + SVC 핸들러 + 스레드 | **1,557** |
| + JVM 함수표(`JavaSvcFunctions` — 프록시가 `Jvm` 을 쥐고 JVM 이 표를 쥔다) | **4** |
| + guest_roots 전역 참조(`mem::forget` 이 `GlobalReferences`·인스턴스를 영구 보유) | **2** = 에뮬레이터 자신 + `LgtTaskRunner` ⇒ 해제 |

- `System::teardown` — executor 작업 전부, 화면 합성기 비움.
- `ArmCore::teardown` — `on_teardown` 훅(JVM 함수표 비우기) 실행 → SVC 핸들러 → 스레드. 전부 잠금 밖에서 drop(해제가 코어를 다시 잠근다).
- `Drop for {Lgt,Ktf,Skt,J2ME}Emulator` 가 위를 부른다. LGT 는 이어서 `LgtJvmSupport::forget_core`.
- ★**같은 PR 에서 장부 정리**: `guest_roots::forget` 이 `BLOCKS`·`REGIONS`·`INSTALLED` 의 그 코어 항목을, `JavaHostField::forget_core` 가
  `HOST_FIELD_VALUES` 를 지운다(넷째 장부 — 같은 코어 id 키라 같은 오염이 가능했다). `INSTALLED` 는 이제 전역 참조를 들고 있다가 `forget` 에서 놓는다.
- `wie_core_arm::live_cores()` — 살아 있는 코어 수. `wie_validate` 가 `--relaunch`/`--restart-at` 때 `live_cores` 로 싣는다.

**KTF·SKT 판정**: KTF 는 **같은 순환**(JVM 함수표 · SVC 핸들러 · 작업)이 있어 같은 PR 에서 끊었다. SKT·J2ME 는 ArmCore 가 없어 코어↔JVM 순환은 없고,
작업 순환만 `System::teardown` 으로 끊었다 — **잔여 누수가 남는다**(아래 브라우저 표 · 후속 제안 1건).

### 수치

**`--restart-at`**(release · 2부팅): `live_cores` main 측정 불가(계측 없음 · 0423 검수 실측 DROP 0) → **1** (LGT·KTF). DROP ≥1 충족.

**`--relaunch N`**(release · `helloworld_lgt` · 최대 RSS): N=0 17.0 / 16.8 MiB · N=10 **31.5** / 50.0 · N=30 **58.3** / 114.1 (PR / main).
네이티브 RSS 의 잔여 증가(부팅당 ~1.4 MiB)는 `live_cores`=1 이라 코어가 아니다 — 아래 wasm 이 평탄하므로 이 프로세스의 malloc 반환 특성으로 읽었다(재지 않았다).

**브라우저**(headless Chromium · `wie_featurephone` wasm · 한 페이지 · 전환 = 생성 → 120틱 → `free()` · 가져오기 = 6틱 → `free()`(`validateGame` 과 같은 모양) ·
`memory.buffer.byteLength`):

| 파일 | main | PR |
|---|---|---|
| keydraw_lgt | 1회 262 → 12회 3,091 → 가져오기 3회째 3,863 MiB → **`RuntimeError: unreachable`(OOM · 4,096 MiB)** | 전환 12 + 가져오기 12 내내 **262.4 MiB** |
| keydraw_ktf | 263 → 3,095 → 3,867 → **OOM** | 내내 **263.1 MiB** |
| draw_j2me.jar | 6.1 → 13.6 → 21.8 MiB(부팅당 ~0.7) | 6.1 → 11.1 → 16.4 MiB(부팅당 ~0.45) |

⇒ ★**main 은 한 탭에서 LGT·KTF 타이틀을 16번째 띄우는 순간 죽는다**(한 대 = 게스트 힙 256 MiB). PR 은 평탄.
J2ME 는 줄었지만 남는다 — 순수 JVM 경로의 잔여 순환이다(후속).

**해제 후 재부팅 use-after-free**: debug `--inject --restart-at 8 --gc-stress 50` keydraw_lgt **PASS · restarted · live_cores 1 · collections 159 · dangling 0** ·
keydraw_ktf PASS · live_cores 1. release `--relaunch 10 --gc-stress 20` helloworld_lgt PASS · collections 304 · dangling 0.

**되돌리면 red**: `wie-{lgt,ktf}/tests/test_emulator_freed.rs`(파일당 시험 1개 — `live_cores` 가 프로세스 전역이라) —
`self.core.teardown()` 주석 처리 시 두 crate 모두 `a core outlived its emulator after 1 ticks` · guest_roots `forget` 이 전역 참조를 놓지 않게 하면 LGT 같은 실패.
`guest_roots::tests::forget_drops_every_entry_of_the_core` 가 장부 정리를 잡는다.

### 사용자 영향

한 탭에서 게임을 바꿔 가며 띄우거나 파일을 가져올 때마다 에뮬레이터 한 대분(약 256 MiB)이 쌓이던 것이 사라졌다 — LGT·KTF 는 15번쯤 바꾸면 탭이 멈췄다.
