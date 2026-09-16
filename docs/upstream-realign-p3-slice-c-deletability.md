# 조각 C — 「upstream 이 이미 가졌다」 ≠ 「지금 base 에서 지울 수 있다」

> 회차 티켓 `wie-p3-slice-c-drop-hunks-already-upstream` · 채택 제안
> `docs/worklog/2026-09-16-p3-remaining-slices-plan.json#p2`.
> 입력 = 조각 B 의 분류표(`docs/upstream-realign-p3-slice-b-triage.md` · PR **#158**, 이 문서 작성 시점 **미머지**).
> ★**이 회차는 B 의 판정을 뒤집지 않는다** — B 는 「머지가 우리 쪽을 버려도 되는가」를 옳게 답했다.
> 이 문서는 **그 다음 질문**에 답한다: ★**「그 행을 «오늘 우리 base 에서» 지울 수 있는가.」**
>
> ★**측정 트리**: `origin/main c77998ad` ↔ `upstream/main 44fbf265` ↔ `merge-base fa641a8a`
> (2026-09-16 · `git fetch` 직후). 머지 예행은 격리 워크트리에서 돌고 `--abort` + `worktree remove --force` 로 제거됐다.

## 0. 결론 — B 의 폐기 21행은 «세 계급»으로 갈린다

| 그룹 | 행 | 오늘 base 에서 | 이 회차의 처분 |
|---|---|---|---|
| **A** | #2~#8 (7파일) | ★**지울 수 있다 — 기능 델타가 «0 임을 실측했다»** | ★**지웠다**(묶음1) |
| **B** | #17~#21 (5행) | ★★**구조적으로 못 지운다** — 컴파일·린트·픽스처가 «실제로» 막는다 | ★**손대지 않았다** |
| **C** | #1 · #9~#16 (9행) | 컴파일은 되나 ★**검증이 «공허하다»** | ★**보류**(운영자 판정 2026-09-16) |

★★**그래서 이 회차의 실제 산출은 「21행을 지웠다」가 아니라 ★「21행 중 7행만이 오늘 지울 수 있는 것이고,
나머지 14행이 «왜» 아닌지를 수로 보였다」** 이다.

## 1. ★그룹 B — 「upstream 도 갖고 있다」의 «이유»가 중복이 아니다

B 표는 #17~#21 을 「upstream 이 이미 갖는다」로 옳게 분류했다. ★**그러나 양쪽이 같은 코드를 가진 이유는
«우리가 베꼈다»가 아니라 «양쪽이 같은 RustJava 트레이트를 따랐다»** 이다. 우리 핀은
`dlunch/RustJava@5b84dd1`(+33)이고, 그 트레이트가 이 코드를 **요구한다**.

되돌려 봤다(`git checkout fa641a8a -- <경로>` = 우리 델타 제거) — **측정값이다**:

| 행 | 되돌리면 | 막는 축 |
|---|---|---|
| #18 `array_class_instance.rs` · #19 `class_instance.rs` · #20 `wipi_c/context.rs` | ★**`cargo check --all` 오류 10건** | `error[E0046]: not all trait items implemented, missing: identity, shallow_clone` · `error[E0407]: method destroy is not a member of trait ArrayClassInstance` ×3 · `error[E0277]: the trait bound JavaArrayClassInstance: ClassInstance is not satisfied` ×4 · ★`error[E0061]: this method takes 1 argument but 0 arguments were supplied`(= `attach_thread(None)`) |
| #17 `wie_ktf/…/java/interface.rs` | ★**beta clippy red** | `error: this function has a #[must_use] attribute with no message, but returns a type already considered as #[must_use]` ⇒ `could not compile wie_ktf` |
| #21 `wie_midp/…/lcdui/image.rs` | ★**`draw_j2me.jar` FAIL** | `panicked at wie_midp/src/classes/javax/microedition/lcdui/image.rs:117:100: called Option::unwrap() on a None value` → `"result":"FAIL"` · `paints 0` |

근거 인용(핀의 트레이트 정의 · `jvm/src/class_instance.rs:14-19`):

```rust
pub trait ClassInstance: Sync + Send + AsAny + Debug + DynHash + DynClone + 'static {
    fn destroy(self: Box<Self>);
    fn identity(&self) -> usize;                                  // ★기본 구현 «없다»
    fn shallow_clone(&self) -> Result<Box<dyn ClassInstance>>;    // ★기본 구현 «없다»
    fn class_definition(&self) -> Box<dyn ClassDefinition>;
    fn equals(&self, other: &dyn ClassInstance) -> Result<bool>;
```

⇒ ★★**이 다섯은 조각 D 의 머지가 «upstream 쪽을 고르면» 저절로 해소된다 — 선행 삭제로 줄일 수 있는 것이 아니다.**
★#21 은 특히 주의: **우리 로컬 픽스처가 이미 그 경로를 덮고 있다**(`AGENTS.md` 러너 블록이 적은
Scenario C-img 축). 「upstream 에 있으니 지워도 된다」로 읽으면 **오늘 바로 깨진다.**

## 2. ★★그룹 C — 컴파일은 되지만 «게이트가 그것을 보지 않는다»

제안 #p2 의 검증 논거는 「우리 현재 base 위에서 지우면 **4게이트 + 5픽스처가 전부 살아** 검증이 강하다」였다.
★**그 문장을 시험했고, 이 9행에서는 «거짓»이다.** 되돌린 뒤(= 지운 것과 같은 상태) 실측:

| 축 | 되돌리기 «전» | 되돌린 «후» | 차이 |
|---|---|---|---|
| `cargo check --all` | 오류 0 | 오류 **0** | 없음 |
| `RUST_MIN_STACK=4194304 cargo test --all` | **179 passed · 0 failed** | **179 passed · 0 failed** | ★**없음** |
| 5픽스처 | 전건 PASS | 전건 PASS(`paints` 1/0/0/55/55 동일) | ★**없음** |

그런데 사라지는 것은 **Java 가시 메서드 18개 + 클래스 1종**이다(`JavaMethodProto::new` 계수):

| 파일 | ours | base | 잃는 메서드 |
|---|---:|---:|---|
| `db/data_base.rs` | 14 | 11 | `getSizeAvailable` · `getRecordSize` · `selectRecord(I[BI)V` |
| `io/file.rs` | 13 | 11 | `openOutputStream` · `openDataOutputStream` |
| `io/file_system.rs` | 9 | 6 | `isFile(…;I)Z` · `isDirectory(…;)Z` · `mkdir(…;)V` |
| `lcdui/card.rs` | 17 | 13 | `<init>(Z)V` · `move(II)V` · `resize(II)V` · `getDisplay` |
| `lcdui/display.rs` | 15 | 12 | `popCard` · `getBitsPerPixel` · `callSerially(…Runnable;I)V` |
| `lcdui/font.rs` | 10 | 9 | `getBaselinePosition` |
| `media/clip.rs` | 8 | 7 | `getVolume` |
| `media/base_clip.rs` | 5 | 4 | `setBuffer([BI)Z` |
| **합계** | | | ★**18** |
| `lcdui/image_observer.rs` (#1) | — | — | ★**클래스 `org.kwis.msp.lcdui.ImageObserver` 등록이 통째로 사라진다** |

★★**제안이 스스로 제시한 완화책은 이 9행에서 «구조적으로 공허하다».** 제안 문안은
「지우기 전 `grep` 으로 우리 안의 호출자 0건을 먼저 보이고, 0건이 아니면 지우지 마라」인데,
★**이것들은 `JavaMethodProto` 로 등록된 «게스트가 부르는» 메서드라 Rust 호출자는 «정의상» 0** 이다.
전수 계수(17심볼):

```
get_size_available 0 · get_record_size 1 · select_record_into 0 · open_output_stream 0 ·
open_data_output_stream 0 · is_file_with_flag 0 · is_directory_no_flag 0 · mkdir_no_flag 0 ·
init_bool 0 · move_to 0 · get_display 0 · pop_card 0 · get_bits_per_pixel 0 ·
call_serially_with_param 0 · get_baseline_position 0 · get_volume 1 · set_buffer 1
```

⇒ ★**그 술어는 「지워도 안전하다」와 「유일한 호출자가 우리가 시험할 수 없는 게임이다」를 «구별하지 못한다».**
292 코퍼스가 이 머신에 없다는 계획 §4-2 의 그 한계가 정확히 여기서 값을 매긴다.

## 3. ★대가 비교 — 「무엇을 사고 무엇을 파는가」

머지 예행을 **실제로 다시 돌려** 수를 얻었다(계획 §6-C 의 요구: 「74 → N 을 적어라」):

| 상태 | 미해결 | 내역 |
|---|---:|---|
| 원본(`c77998ad`) | **74** | UU 35 · AU 19 · UD 17 · AA 3 |
| ★**묶음1 착지 후(이 PR)** | ★**67** | UU 35 · **AU 12** · UD 17 · AA 3 |
| (가정) 묶음1 + 그룹 C | 59 | **UU 34** · AU 12 · **UD 10** · AA 3 |

⇒ 그룹 C 가 사는 것은 **충돌 8건**, 파는 것은 **탐지되지 않는 WIPI 메서드 18개 + 클래스 1종**이다.
★**그리고 그 8건은 전부 `UD`/`UU` 라 조각 D 가 «upstream 쪽을 고르면» 그냥 해소된다** — upstream 판본이
그 18개 메서드를 **전부 갖고 있기 때문**이다(조각 B 가 열어서 확인한 그것).

★★**결정(2026-09-16 운영자): 그룹 C 는 지우지 않는다.** 근거 셋:
⑴위 검증 공허 ⑵★**복원이 보장되지 않는다** — 조각 D 는 조각 A 에 걸려 있고, A 는 **no-go 신호**를 들고 있다
(`keydraw_lgt` 가 upstream base 에서 FAIL · 계획 §2). D 가 무산되면 18개는 **영구 손실**이다.
⑶사는 것(8)이 남는 것(59)에 비해 작다 — ★**67 이든 59 든 머지는 여전히 사람이 한 번에 읽을 크기가 아니다.**

## 4. ★그룹 A — 「지웠다」의 근거는 «읽은 것»이 아니라 «잰 것»

`wie_wipi_java/src/classes/java/**` 7파일(#2~#8)은 ★**upstream 축이 아니라 «우리 base 축»에서 이미 죽어 있었다.**

```sh
# 삭제 «전» 원본 트리에서, get_protos() 를 전부 적재한 채
jvm.new_class("java/lang/VirtualMachineError", "()V", ())  →  InstantiationError
```

★우리 사본의 `access_flags` 는 `Default::default()`(비-ABSTRACT)이고
`java_runtime@5b84dd1 .../java/lang/virtual_machine_error.rs:23` 은 **`ClassAccessFlags::ABSTRACT`** 다.
⇒ ★**그 예외는 «런타임 정의가 이겼다»의 직접 증거다.** 이유는 `wie_jvm_support/src/lib.rs` 가
`java.class.path` 를 **`RT_RUSTJAR : WIE_RUSTJAR : <jar>`** 순으로 세우기 때문이다.

★**시간 순서**(왜 예전엔 옳았나): 사본 등록 `5603a7f9` **2026-07-02**(당시 Jun025/RustJava 포크 핀에 그 넷이 없었다)
↔ 핀 이동 `1762a32c` **2026-09-04**(`dlunch/RustJava@5b84dd1`, `java_runtime/src/loader.rs:26/36/80/96` 이 넷 다 등록).
⇒ ★**그 사이에 죽었고, 아무도 알아채지 못했다.**

★**잠금 축**: `wie_wipi_java/tests/preload_classes_come_from_the_runtime.rs` —
⑴네 클래스가 resolve 되고(KTF `MExe_init` 이 요구하는 것) ⑵그것이 **런타임의** 판본임을 ABSTRACT 판별자로 단언한다.
★**핀이 그 넷 중 하나를 잃으면 «게스트가 부팅 중 abort» 하는 게 아니라 «여기가 붉는다».**

## 5. ★못 재는 것 — 「무엇이 있으면 잴 수 있는가」와 함께

| 못 재는 것 | 있으면 잴 수 있는 것 |
|---|---|
| ★**그룹 C 의 18개 메서드를 실제로 부르는 타이틀이 몇인가** — Rust 호출자 계수는 이 축을 «정의상» 못 본다 | `game_lab/working/{ktf,lgt,skt}` 292 코퍼스(계획 §4-2 · human-step 후보) |
| ★**그룹 A 삭제가 「5 KTF 타이틀」에 무해한가** — 원 커밋 `5603a7f9` 의 그 주장 | 같은 코퍼스. ★**대체 증거는 §4 의 `InstantiationError` 실측**이고, 그것은 「사본이 애초에 안 쓰였다」를 말하므로 무해가 «따라 나온다» |
| ★**#21 을 지워도 되는 upstream base 에서의 동작** | 조각 D 이후의 재측 — 지금 트리에서는 `draw_j2me` 가 **즉시 FAIL** 한다 |
| ★**67 이 조각 D 에서 실제로 몇으로 떨어지는가** | 조각 D 의 머지 그 자체. ★이 문서의 59/67 은 **예행 수**이지 착지 수가 아니다 |
