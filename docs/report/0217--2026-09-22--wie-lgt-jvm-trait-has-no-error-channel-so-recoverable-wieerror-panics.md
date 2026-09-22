## [2026-09-22] 죽은 객체 참조는 «패닉»이 아니라 «null» 이다 — 레전드오브마스터 2파일 panic → PASS (wie-lgt-jvm-trait-has-no-error-channel-so-recoverable-wieerror-panics)

### 무엇을

`NativeJavaValueCodec::object_from_raw` 의 반환을 `Box<dyn ClassInstance>` → **`Option<Box<…>>`** 로 바꿨다.
게스트 워드가 «살아 있는 객체»를 가리키지 않으면 `None` 을 돌려주고, `decode_word` 는 그것을
**워드 0 과 똑같이** `JavaValue::Object(None)` 으로 읽는다(+`tracing::warn!` 으로 거부한 워드를 남긴다).
SVC 핸들러 경로(`LgtJvmSupport::class_instance_from_raw`)는 **null 이 아니라 `WieError`** 를 돌려준다 —
그쪽 호출부 9곳은 전부 `wie_util::Result` 를 반환하므로 tick 오류로 살아난다.

### 왜 — ★«어느 프레임에서 실패하는가»가 판정을 정한다

티켓은 `value.rs::object_from_raw` 와 `class_instance.rs::class_definition` 을 후보로 지목했다.
★**둘 다 아니었다.** `RUST_BACKTRACE=1` 로 실측한 실제 프레임(두 타이틀 모두 **동일**):

```
object_from_raw                                   ← wie-lgt/.../value.rs:24
  ← JavaClassInstance::get_field                  ← impl ClassInstance (sync · jvm::Result)
    ← ⒜ Jvm::get_field  ←  Card::repaint_with_area        (스파이더맨3)
      ⒝ jvm::garbage_collector::find_reachable_objects  (레전드오브마스터)
        ← Jvm::collect_garbage ← Display::handle_paint_event
```

★**그 프레임에는 오류를 넣을 자리가 «없다» — 두 겹으로.**
⑴`jvm::Result<T> = Result<T, JavaError>` 이고 `JavaError` 의 변종은 **`JavaException(Box<dyn ClassInstance>)`
단 하나**다(실측 `jvm-0.1.1/src/error.rs`). 그 인스턴스는 `Jvm::exception` 으로만 만들 수 있고
그것은 **async + `&Jvm`** 인데 `ClassInstance::get_field` 은 **sync 이고 `Jvm` 을 갖지 않는다.**
⑵설령 ⑴이 풀려도 ★**`find_reachable_objects` 는 반환형이 `()` 이고 `get_field`·`load`·`get_static_field`
전부를 `.unwrap()` 한다** ⇒ GC 경로에서 올린 오류는 **그 자리에서 다시 패닉이 된다.**
(`ClassInstance::class_definition` 은 아예 `Result` 가 없다 — 이것도 같은 축이다.)

⇒ ★**「호출부에서 `WieError` 로 되돌린다」(티켓 ⒝)는 이 프레임에서 «불가»다.**
그래서 오류를 올리는 대신 **게이트에서 거부**했다 — `object_from_raw` 는 `wie-jvm-support` 의
**우리 트레이트**이고, 모든 참조가 그 한 곳을 지나며, ★**아직 `Option` 을 표현할 수 있는 마지막 프레임**이다.
`jvm` 트레이트 시그니처는 **한 줄도 건드리지 않았다**(upstream 병합면 증가 0).

★**직전 회차가 버린 diff 와 다른 점**: 그 회차는 `object_from_raw` 의 `unwrap` 만 지워
«패닉이 한 프레임 이동»했다(→ `class_definition`). 이번에는 **깨진 인스턴스를 만들지 않는다** ⇒
위쪽 프레임이 그것을 볼 일이 없다.

### ★⑴ 두 자리에서 «무엇이» 실패하는가 — 티켓 §필수①

티켓의 의심(「`(0)` 과 `(3321430483)` 은 다른 원인일 수 있다」)은 **맞았고, 예상보다 더 갈린다.**

| 타이틀 | 문면 | 무엇이 0/쓰레기인가 | 이름 |
|---|---|---|---|
| 레전드오브마스터(2파일) | `InvalidMemoryAccess(0)` | 인스턴스 헤더의 **`ptr_dispatch_table` 이 0** (`ptr_raw` 는 0 이 아니다 — `decode_word` 가 `raw != 0` 로 이미 걸렀다. 주소 0 을 낼 수 있는 읽기는 `class()` 의 **둘째** `read_generic` 뿐이다) | **죽은/미초기화 인스턴스 헤더**를 GC 도달성 순회가 밟는다 |
| 스파이더맨3 | `InvalidMemoryAccess(3338…)` | ★**주소가 아니라 «밀리초 시계값»이다** | `Card.canvas`(`Ljavax/microedition/lcdui/Canvas;`) 워드에 **시간값**이 들어 있다 = Card **필드 레이아웃 불일치** |

★**둘째 줄은 추론이 아니라 실측이다.** 같은 바이너리로 **62초 간격** 두 번 돌렸다:

```
wall  1790038975 → 1790039037   (+62 s)
addr  3332581741 → 3332644028   (+62287)
```

⇒ ★**값이 벽시계와 1:1 로 전진한다. 포인터가 아니다.** 4회 관측 전건 다른 값(3321430483 · 3332385528 ·
3332501851 · 3338699669)이라 「쓰레기 포인터」로 읽으면 원인을 놓친다. ★**이 축은 이 회차 밖이다**
(`Card` 필드 워드 인덱스 축 — 후속 추천 ②).

### ★⑵ ⒜/⒝ 판정 — 티켓 §필수②

★**«둘 다»가 정확한 답이고, 축이 갈린다.**

| 축 | 판정 | 근거 |
|---|---|---|
| **복구 가능한 실패가 패닉이 되는 것을 막는 일** | ★**⒝ — 우리 쪽에서 닫았다** | 게이트(`object_from_raw`)가 우리 트레이트이고, 거기서 거부하면 위쪽 프레임이 깨진 인스턴스를 못 본다. 트레이트 무접촉 |
| **sync `jvm` 트레이트 impl 이 호스트 오류를 «보고»하는 일** | ★**⒜ — upstream** | 위 ⑴⑵의 두 겹. 이 회차가 고칠 수 없고 제안문까지가 경계다 |

★**티켓 전제 1건 정정**: 티켓은 「`[patch]` 로 `Jun025/RustJava` 를 물린다」고 썼는데 ★**그 `[patch]` 표는
2026-09-04 에 삭제됐다**(`docs/upstream-realign-verdict.md` §9 · 이 저장소 `Cargo.toml` 에 `[patch]` **0건**).
`jvm` 은 **crates.io `0.1.1`**(`Cargo.lock` checksum 확인)이다.
⇒ ★**⒜ 의 비용이 「rev 핀 이동」이 아니라 「upstream 착지 + crates.io 릴리스」다** — 그것이 ⒝ 를 고른 두 번째 이유다.

### ★⑶ upstream 제안 — 인접 2건과 «같은 결론»이므로 «합쳐서» 낸다 (티켓 §필수④)

티켓이 지목한 두 제안을 읽었다. **같은 장애물에 독립으로 닿아 있다**(전문 인용):

- `RustJava/docs/worklog/2026-09-20-string-array-hiding-overflows-stack.json#p0` —
  「A re-entrancy guard has to decide what to return when it trips, and the honest answer is a
  non-Java error — **which `JavaError` cannot currently express (single variant, carries a
  ClassInstance)**. That is the same obstacle the sibling round hit, so **the two should probably be
  decided together** rather than separately.」
- `…-name-the-missing-bootstrap-class.json#p0` — 「Measured cost of the alternative: **`JavaError` has a
  single variant holding a `Box<dyn ClassInstance>`** …」

⇒ ★**셋이 같은 한 줄을 요구한다: `JavaError` 에 비-Java(호스트) 변종.**
★**이 회차가 보태는 것은 «게스트에게 보이는 대가»다** — 앞의 둘은 자기 진단/재귀 축이고,
여기서는 **실제 타이틀 2건이 호스트를 죽였다.** 그리고 **둘째 반**(`find_reachable_objects` 가
`Result` 를 전파해야 한다 · `Jvm::collect_garbage` 는 이미 `Result<usize>` 다)은 앞의 둘에 **없다.**
정본 = `docs/worklog/2026-09-22-lgt-object-reference-gate.json#p0`(`target: RustJava jvm/src/error.rs …`).

★**임시 완화 가능성(§필수④)**: 「가능하다」가 아니라 ★**이번에 그것을 «했다»** — 위 ⒝ 축이 그 완화다.
upstream 이 닫히면 남는 일은 **게이트의 `None` 을 호스트 오류로 «승격»** 하는 것뿐이고,
그때도 게이트 위치는 그대로다.

### ★⑷ 레전드오브마스터가 더 가는가 — 티켓 §필수③ · 짝지어 실측

같은 분에 번갈아 실행(`--inject` · release · before = `origin/main` + PR #252 병합 · after = 그것 + 이 diff):

| 타이틀 | BEFORE | AFTER |
|---|---|---|
| **lgt 레전드 오브 마스터** | `FAIL` t36 p1 · `panic during 'boot'` | ★**`PASS` t50000000 p2 dc33 lf=true** |
| **레전드오브마스터** | `FAIL` t36 p1 · `panic during 'boot'` | ★**`PASS` t50000000 p2 dc33 lf=true** |
| **스파이더맨3** | `FAIL` **t1** p0 dc0 · `panic during 'boot'` | `FAIL` t50000000 **p1 dc1** · `only blank/uniform frames` |

★**레전드오브마스터 2파일은 `PASS`(booted + rendered + survived input sequence)로 넘어갔다.**
★**「렌더한다」고 쓰지 않는다** — paints 는 **2**이고 `--inject` 예산 전체를 돈다. 시각적 정합은 안 봤다.
★**스파이더맨3 은 «벽이 바뀌었다»**: 부팅에서 죽던 것이 부팅을 통과해 **1프레임을 그리고** 이후
`no frame rendered` 계열로 간다 ⇒ 다음 회차의 입력이 생겼다(후속 추천 ②).

★★**측정 위생 — `ticks`·step 라벨을 절대값으로 인용하지 마라.** 같은 before 바이너리가
같은 파일에서 **`panic during '21_UP'` / `'14_OK'` / `'11_LEFT'` / `'04_NUM5'` / `'boot'`** 로 움직였고
`ticks` 도 36~46 이었다. ★**읽은 축은 «판정 + 벽 문면»이다.** load1 은 아래 §부하.

### 회귀 — `game_lab/broken/lgt` **46파일 전건 before/after 짝지어**

- JSON 을 내는 **44쌍**(2쌍 = `LGT 크로이센`·`크로이센` **NO-JSON · before·after 둘 다** = 선재·무관)
- **판정 + 벽 문면 유지 41/44** · 변경 **3건**(전부 위 표) · ★**`PASS → non-PASS` 0건**
- ★`당구마스터2010` 은 **«유지»로 센다**: 벽이 `Unknown lgt stdlib import: 0x3f9` 로 **동일**하고
  step 라벨만 `07_DOWN ↔ 08_OK` 로 다르다(위 비결정성과 같은 형태 · t482 ↔ t442)
- ★**렌더 3종**(PR #989 로 사용자에게 열려 있음) 재확인:

| | BEFORE | AFTER |
|---|---|---|
| 놈3 | PASS p108 **dc99** | PASS p105 **dc99** |
| 메이플스토리2007 | FAIL · `Thread vtable index 13` | FAIL · ★**같은 벽** |
| 현영맞고2006 | PASS p264 **dc512** | PASS p150 **dc512** |

★`현영맞고2006` 의 paints 264 → 150 은 **이 타이틀의 알려진 변동폭**(`ticks` 0.43M~50M) 안이고
판정·`distinct_colors` 는 불변이다. ★**그것을 회귀로 읽지 않는다 — 그리고 「회귀 없음」이라 단정하지도 않는다.**

### 게이트

4게이트 + `cargo +beta clippy --all -D warnings` **전건 rc=0**(`cargo test --all` 46 suites 전건 ok · 실패 0).
엔진 코드를 만졌으므로 **러너 블록**도 돌렸다:
`draw_j2me`·`helloworld_ktf`·`helloworld_lgt`·`text_j2me`(`--timeout 5`) **PASS** ·
`keydraw_ktf`·`keydraw_lgt` **PASS · content true · rc=0 · paints 55**(무부하 범위 48~55 안) ·
`git status test_data/` **clean**(픽스처 재생성 바이트 안정). clet 회귀 `test_helloworld` 포함.

### 시험은 개악 대조로 «진다»

`value.rs` 의 `a_reference_to_a_dead_instance_decodes_to_null` — **양방향**이라 한쪽 개악으로 통과하지 못한다:

| 개악 | 결과 |
|---|---|
| **M1** 게이트를 종전 `instance.class().unwrap()` 으로 되돌림 | **FAILED** — `value.rs:26` 에서 패닉(= 이 회차가 고친 그 형태) |
| **M2** `object_from_raw` 가 항상 `None` | **FAILED** — 살아 있는 인스턴스 단정(`:111`)이 진다 |
| 원문 | ok |

### 사용자 영향

LGT 게스트가 **살아 있지 않은 객체 참조**를 넘겼을 때 **호스트 엔진 인스턴스가 죽지 않는다**(패닉 → null).
`wie_validate` 는 이전에도 패닉을 잡아 정상 JSON 을 냈으므로 **진단 손실은 0 이었지만**, 사용자 셸에서는
패닉이 **복구 불가**였다 — 그쪽이 이 변경의 실제 수혜다. 그리고 `레전드오브마스터` 2파일이 `PASS` 로 넘어갔다.

★**대가를 숨기지 않는다**: 깨진 참조가 **조용한 null** 이 되므로 게스트는 NPE 를 보게 된다(Java 계약 안 ·
catch 가능). 그 대신 **거부한 워드를 `tracing::warn!` 으로 남긴다** — 스파이더맨3 의 Card 레이아웃 축은
정확히 그 로그로 찾았다. ★**진단을 지우지 않는다.**

### 손대지 않은 것

- ★`Runtime`/`Thread`/`String` **미배치 vtable 인덱스** — 별 P0 소관(`wie-unplaced-vtable-index-family-…`).
  메이플스토리2007 이 이 회차 전후로 **같은 벽**에 서는 것이 그 증거다.
- ★`서든어택포켓`·`턴` hang · `SD한국전쟁` init struct · `NoClassDefFoundError` 클러스터 — 각자 다른 계급.
- ★`RustJava` repo **직접 무접촉** — 제안문까지가 경계다(worklog `target:` 로 표기).
- `otterpebble`·`tower`·원장 무접촉. 게임 바이트 커밋 **0** · `game_lab/` 커밋 **0** · 새 검사기·CI 스텝 **0**.
- ★`impl ClassInstance` 의 나머지 `unwrap()` 무리(`destroy`·`shallow_clone`·`class_definition`·
  `get_field` 의 주소 계산)는 **그대로 두었다** — 이 회차가 잰 두 프레임이 아니고,
  ★그것들을 지우는 일은 위 ⒜(upstream) 없이는 «패닉을 한 프레임 옮기는» 그 실패형이다.

### 부하와 예산

- **load1**: 조사·빌드 구간 **60~112** · 짝지어 스윕 구간 **21~34** · 러너 블록 **21~25**.
- **예산**: `--inject` **기본값** = `2.5 + 0.3 + 27×0.6 + 1.0` = **20.0 s**.
  ★`--timeout` 은 `--inject` 경로에서 덮어써지므로 쓰지 않았다(`text_j2me` 만 `--timeout 5`).
- ★FAIL 을 자기 diff 탓으로 읽기 전에 AGENTS.md 4단계를 밟았다 — 이번 3건은 **손대지 않은 트리
  (before 바이너리)에서 반대 방향으로 재현**되므로 starvation 이 아니다.
