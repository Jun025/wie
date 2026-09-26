## [2026-09-26] 서든어택포켓 부팅의 «무양보 구간»은 게스트 연산이 아니라 우리 링커 — `JavaVtable::read` 의 선형 대조를 색인으로, 그 tick 5.4~10.6배 단축 (wie-surdenattack-boot-30s-no-yield-measure)

**무엇을** — 채택 제안 `2026-09-25-surdenattack-pocket-wall-is-dos14-not-pre-save#p1`(부팅의 30초 무양보 구간에서 무엇을 도는지
잰다). 판정은 **우리 스텁/링커 비용**. `wie-lgt/src/runtime/java/jvm_support/vtable.rs` 의 `JavaVtable::read` 한 곳을 고쳤다 —
vtable 항목마다 «알려진 모든 메서드»를 선형으로 훑던 대조를 호출당 한 번 만드는 target→method 색인으로 바꿨다(첫 일치 우선 ·
`target()` 오류 메서드 제외 · ABI 폴백 그대로 ⇒ 결과 동일).

**왜** — 0260 은 `AnnunciatorComponent::show()` ↔ `Display::getDisplay` 사이 29~39 s 동안 ticks 가 1~2 라는 것만 적고
정체를 판정하지 않았다(그 회차 limits). 그 공백이 `--timeout 30` 측정(0242)을 `count 0` 으로 만들어 벽 위치를 오독하게 했다.

### 대전제 재측 — origin/main `5434ab0a`

| 빌드 | 측정 | 값 |
|---|---|---|
| release(LTO) | `RUST_LOG=info --timeout 60`, 줄마다 경과초 | show() **1.27 s** → getDisplay **3.88 s** (공백 2.6 s) · `FileNotFoundException` 4.17 s · ticks 2338 · count 2 |
| release(LTO) | `--timeout 30` × 4 (원본 3 · `_dup` 1) | **4/4 count 2** · 벽 문면 동일(`FileNotFoundException` → `DataOutputStream 14`) |
| release(LTO) | `--max-ticks N` 의 `ms` | N=1 **263** · N=2 **3,849** · 3 6,992 · 4 3,764 · 5 2,443 · 8 4,272 |

⇒ 공백은 **둘째 tick 한 번**이다(N=1→2 에서 수 초가 붙고 이후는 load 요동). 0260 의 29~39 s 는 LTO release 에서는
재현되지 않았다(2.4~7.0 s). **non-LTO release 에서는 재현된다**(아래 A/B 의 base 5.4~45.5 s) — 0260 의 probe 가 non-LTO
빌드였다는 점과 맞지만, 0260 표 자체의 빌드 프로필은 원문에 없어 **귀속은 보류**한다. `2349a614..5434ab0a` 82커밋 중
`jvm_support`·`interface.rs` 를 건드린 것은 0 이다.

### 무엇을 도는가 — `sample`(macOS) 로 둘째 tick 을 찍었다

게스트 ARM 은 `INSTRUCTIONS_PER_YIELD = 10_000` 마다 양보하고 executor tick 은 예산(현 14 ms)을 넘으면 끊는다 ⇒
**tick 이 수 초 끝나지 않으면 그 시간은 게스트가 아니라 양보 없는 호스트 Rust 코드다.** LTO release 2 s 표본(855개):

```
LgtEmulator::tick → … java_start_application → Jvm::execute_method → (게스트 <clinit>/main 사슬)
  → handle_java_system_svc → java_register_class → LgtJvmSupport::register_generated_class
    → ArmCore::run_function → handle_java_system_svc → java_link → link_class_members
      → LgtJvmSupport::virtual_method_index            772 / 855  (90.3%)
top of stack: Arm32CpuEngine::mem_read 228 · JavaMethod::target 211 · memmove 159 · JavaVtable::read(try_fold) 146
```

- `link_class_members` 는 클래스의 **가져온 가상·인터페이스 메서드마다** `virtual_method_index` 를 부른다.
- 그것은 매번 `vtable_entries` → `JavaVtable::read` 로 표를 새로 읽는데, 항목 하나당 계층의 **모든 메서드**를 훑으며
  `JavaMethod::target()` 을 부르고, 그것이 **메서드 구조체를 게스트 메모리에서 통째로 읽는다**(`read_generic` → memmove).
- ⇒ 링크 한 건 = 가져온 메서드 수 × 항목 수 × 메서드 수의 게스트 읽기. 그 곱이 양보 없이 한 tick 안에서 돈다.

| 후보 | 판정 |
|---|---|
| 게스트 정상 연산 | **아니다** — 표본의 90% 가 호스트 `virtual_method_index` |
| 우리 스텁/링커 비용 | **이것** — `JavaVtable::read` 의 O(항목 × 메서드) 게스트 읽기 |
| 선점 불가 | 구조상 그렇다(호스트 async 는 게스트 명령 계수 밖이라 양보점이 없다) — 그러나 비용을 줄이면 된다 |

### 수리 A/B — 같은 트리 non-LTO release 두 벌, 교대 6쌍 (load1 103~121)

`--timeout 60 --max-ticks 2` 의 `ms`(= 부팅부터 둘째 tick 끝까지):

| 쌍 | base | fix | 배 |
|---|---|---|---|
| 1 | 45,483 | 4,737 | 9.6 |
| 2 | 27,907 | 5,196 | 5.4 |
| 3 | 27,573 | 2,610 | 10.6 |
| 4 | 17,772 | 2,278 | 7.8 |
| 5 | 7,436 | 1,106 | 6.7 |
| 6 | 5,436 | 998 | 5.4 |

fix 쪽 둘째 tick 은 1 s 표본에 28개만 잡혔다(그 사이 run 이 끝났다) — 남은 몫은 이 회차에서 더 쪼개지 않았다.

### 등가성 — LGT 코퍼스 91파일 짝(`working/lgt` 54 + `broken/lgt` 37 · `--timeout 30` · base·fix 동시 기동 · 4쌍 병렬)

| base → fix | 수 |
|---|---|
| PASS → PASS | 70 |
| FAIL → FAIL | 19 |
| FAIL → PASS | 1 (놈3) |
| PASS → FAIL | **0** |
| 둘 다 결과 줄 없음 | 1 (크로이센) |

- 놈3: 30 s 에서 base 만 `no frame rendered` — `--timeout 60` 짝 재측은 **둘 다 PASS**(ticks 1082/1077 · paints 189/190) ⇒ base 의 기아.
- 크로이센: base·fix **둘 다** 호스트 `stack overflow` rc=134 — 기존 결함, 이 변경과 무관.
- `result`·`content`·`java_exceptions.first` 가 갈린 쌍은 위 놈3 하나뿐이다.

### 측정 예산 권고(벽 오독 방지)

`java_exceptions.count 0` 에 **ticks 가 한 자리**면 그 run 은 벽을 잰 것이 아니라 부팅 한 tick 안에서 끝난 것이다 —
벽을 읽기 전에 `--timeout 60` 이상으로 다시 재고, `--max-ticks 2` 의 `ms` 로 부팅 tick 길이를 먼저 본다. 이 수리 뒤
서든어택포켓의 부팅 tick 은 1~5 s(non-LTO)라 30 s 로도 벽에 닿지만, 같은 링크 비용은 가져온 메서드가 많은 모든
LGT 타이틀에 붙으므로 권고는 타이틀 무관하다.

### 검증

`cargo fmt --all -- --check` rc=0 · `cargo clippy --all -- -D warnings` rc=0 ·
`cargo clippy --target wasm32-unknown-unknown -- -D warnings` rc=0 · `cargo +beta clippy --all -- -D warnings` rc=0 ·
`RUST_MIN_STACK=4194304 cargo test --all` rc=0 · **454 passed / 0 failed**.
AGENTS.md 러너 블록(엔진 변경) — `draw_j2me`·`helloworld_ktf`·`helloworld_lgt` PASS · `keydraw_ktf`·`keydraw_lgt`
`--inject --expect-last-frame` PASS rc=0 · `text_j2me --timeout 5` PASS.
★이 변경을 잠그는 시험은 없다 — 의미 불변 성능 변경이라 등가성 근거는 위 91파일 짝이다.

**게임 파일명 유입**(`node scripts/corpus-name-inflow.mjs --corpus <game_lab>` 실행값): BOUNDED·SUFFIX-ATTACHED 는
아래 표식 그대로다. 전건 이 회차 측정 대상 타이틀 이름(표·판정 문장·코드 주석 1곳)이고 SUFFIX-ATTACHED 는
「서든어택」⊂「서든어택포켓」 과 「서든어택포켓의」(조사) — 파일 바이트·경로 유입은 없다.
