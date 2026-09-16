## [2026-09-16] 조각 A — upstream base 에서 `keydraw_lgt` 가 깨지는 원인 규명 (wie-p3-slice-a-keydraw-lgt-breaks-on-upstream-base)

**무엇을** — 조사 전용(제품 코드 **0줄**). 원인을 **이름으로** 댔고, ★**27줄 치환 한 번으로 PASS 를 되돌려**
그 이름이 맞음을 보였다. 채택 제안 `2026-09-16-p3-remaining-slices-plan#p0`.

**왜** — 이 FAIL 이 코퍼스 없이 얻은 **유일한 no-go 신호**이고 조각 **D**(base swap)의 게이트다.

> ★**[정정 2026-09-16 · 게이트② `request-changes` → `-fix`]** 원인 판정은 **그대로**(검수자가 전건 재현했다).
> 고친 것은 **«불확실성의 주소»** 둘이다 — ⑴패닉 자리를 `fb.buf`(+16) 읽기로 적었는데 **핸들 역참조**다
> ⑵대안을 「`ptr_image` 한 겹 아래」로 적었는데 화면 FB 는 **`ptr_image: 0`** 이고 실제 간접은 **두 겹**이다.

### 원인 — 한 줄

★★**upstream `wie-lgt/src/runtime/wipi_c.rs` 가 graphics SVC **27개**를 LGT 전용 구현
(`wie-lgt/src/runtime/wipi_c/graphics.rs` · **1,095줄**)으로 보내고, 그 구현이 게스트에게
«다른 레코드 ABI»를 준다.** 도입 커밋 = ★**`9a88423b` (2026-08-23) `Fix LGT graphics and runtime
compatibility (#1368)`** — 2026-09-13 회차가 엔트리포인트 규약에서 인용한 **그 PR 과 같다**.

| | ours | upstream |
|---|---|---|
| `MC_grpGetScreenFrameBuffer` 구현 | `wie_wipi_c::api::graphics`(공용) | `wie_lgt::runtime::wipi_c::graphics`(LGT 전용) |
| 반환 레코드 | `WIPICFramebuffer{width,height,bpl,bpp,**buf**}` **20B** · 픽셀 포인터 **+16** | `LgtFramebuffer{owned_image,ptr_graphics,ptr_image,screen_kind}` **16B** · ★**`buf` 필드 없음** |

게스트(`keydraw_lgt.zip`)는 **dlunch 의 게스트 SDK `wipi`** 로 빌드됐고, 그 게스트가 **스스로 패닉**한다.
게스트 자신이 그렇게 말한다(`MC_knlPrintk` 로 뽑은 전문):

```
res: 9 : 602
panicked at wipi/src/framebuffer.rs : 149 : 18 : null reference produced
```

★★**그 패닉 자리는 «핸들 자신»의 역참조다 — `fb.buf`(+16) 읽기가 «아니다».**
게스트 SDK 체크아웃(`wipi-05c13da594aa95be/068312d`) 실측:

| 줄 | 코드 | 이 패닉을 낼 수 있나 |
|---|---|---|
| `:149` `read_fb` | `unsafe { &*(deref_indirect_ptr(self.raw) as *const _) }` (`&` 가 **18열**) | ★**낸다** — 파일 전체에서 `&*` 는 ★**이 한 자리뿐**(실측 `grep -c '&\*'` = **1**) |
| `:153` `buffer_ptr` | `deref_indirect_ptr(fb.buf)` ← **+16 은 여기다** | ★**못 낸다** — **raw 포인터를 돌려준다**(널이어도 «참조»를 만들지 않는다. 널이면 `set_pixel :46-52` 의 `*pixel_ptr = …` 에서 **와일드 쓰기**가 난다) |

⇒ ★**패닉은 «+16 읽기»보다 «한 단계 앞»에서 난다 — `fb.buf` 는 «읽히기 전»이다.**
★★**그러므로 「게스트가 +16 을 읽어 레코드 끝 너머에서 0 을 얻는다」는 «추론»이고 이 회차가 «측정하지 않았다».**
측정된 것은 셋이다 — ⒜두 레코드 레이아웃이 다르다 ⒝게스트가 `framebuffer.rs:149:18` 에서 널 참조로 패닉한다
⒞27줄을 공용으로 되돌리면 PASS 다. ★**조각 D 의 결정은 그 셋만으로 그대로 선다.**

그 패닉 뒤 게스트가 **주소 0 으로 분기**해 호스트가 `Undefined instruction`(PC=0x0 · 전 레지스터 0)을 낸다 —
`net/wie/CletWrapperCard.paint` 스택은 **증상이지 원인이 아니다**.

★★**「레코드만 20B 로 맞추면 된다」는 선택지가 «아니다»** — ⑶-3 의 **202 단일 치환이 FAIL 불변**이라 축은
**계열 전체**이고, 게이트② 검수자가 `get_screen_framebuffer` 를 **진짜 20B 백킹**(`buf` 가 실픽셀을 가리킨다 ·
양쪽 non-null)으로 고쳐 돌려도 ★**같은 `149:18` 에서 같은 문구로 FAIL** 이었다.
※그 실험 자체는 **결정적이지 않다**(치환이 upstream 의 `resolve_framebuffer` 를 함께 깨므로 `paints 0` 이
flush 축으로도 설명된다) — 그래서 «20B 로 맞추면 된다»의 **반증**이 아니라 **미지지**로 적는다.

### 이 주장을 무는 것 — 치환 실험

`wie-lgt/src/runtime/wipi_c.rs` 의 `=> graphics::` **27건**을 `=> wie_wipi_c::api::graphics::` 로 바꾸고
upstream 을 그대로 빌드하면:

| | 결과 |
|---|---|
| upstream 원본 | **FAIL · paints 0** (3/3) |
| ★**27줄 치환 upstream** | ★**PASS · paints 55 · content true** (2/2) |
| ours(대조군) | **PASS · paints 55/55/47 · rc=0** (3/3) |

### 아닌 것 — 배제한 가설 넷 (전부 실행으로)

1. **함수 포인터 등록 실패 아님** — `CletWrapperCard::<init>` 인자가 양쪽 **`0x25619` 동일**,
   `paint` 시점 재측도 **`0x25619`**(프로브). `paint` 본체는 두 트리가 **바이트 동일**.
2. **upstream 이 추가한 `graphics::init_process_state`/`set_use_annunciator` 2줄 아님** — 꺼도 **FAIL 불변**.
3. **단일 함수 아님** — `GetScreenFramebuffer`(202) **한 줄만** 공용으로 돌려도 **FAIL 불변**
   (같은 `149:18` 패닉) ⇒ **계열 전체**가 축이다.
4. **공용 구현이 갈린 것 아님** — `get_screen_framebuffer` 공용 판본은 두 트리 **diff 0**.

### 범위 — LGT 그리기 경로 «한정»

| 픽스처 | upstream |
|---|---|
| `keydraw_ktf` (그린다) | **PASS · paints 55** — upstream 도 **공용** 구현으로 보낸다(트레이스 확인) |
| `helloworld_lgt` (안 그린다) | **PASS** — LGT 부팅 자체는 산다 |
| `keydraw_lgt` (그린다) | ★**FAIL** |

### ★«픽스처 편향인가» — 계획이 남긴 질문에 답한다(단정하지 않는다)

계획 §6-A 는 「그 픽스처는 우리가 만든 것이라 우리 경로에 유리하게 편향됐을 수 있다」를 «못 재는 것»으로 적었다.
★**이 회차가 그 축을 «조금» 움직였다** — 우리 자신의 리버스 문서 `docs/lgt_abi.md:930` 이 **실제 LGT clet
타이틀(놈ZERO)** 관측으로 이렇게 적는다:

> 놈ZERO issues **zero blit SVCs** … only `FillRect`/`DrawRect` + **direct writes to a framebuffer pointer
> from `GetScreenFrameBuffer`/`CreateOffScreenFrameBuffer`**

⇒ ★**실제 LGT clet 도 「`GetScreenFrameBuffer` 가 준 포인터에 픽셀을 직접 쓴다」** — 우리 픽스처와 **같은 모양**이다.
★★**그러나 이것으로 「upstream 이 실게임도 깬다」를 «확정하지 않는다»** — 실제 clet 이 **upstream 의 간접 체인을
걷는** 것이 진짜 LGT ABI 라면 upstream 이 옳고 **우리 픽스처가 대표성이 없다**. ★**가르는 것은 코퍼스뿐이다**
(`game_lab/working/lgt/` 52건).

★★**그 체인의 «주소»를 정확히 적는다 — 화면 FB 에 「`ptr_image` 한 겹 아래」는 «없다».**
upstream `wie-lgt/src/runtime/wipi_c/graphics.rs` 실측:

| | 값 | 근거 |
|---|---|---|
| **화면** FB(`get_screen_framebuffer`) | ★**`ptr_image: 0`** | `:374-381` 의 `LgtFramebuffer{owned_image:0, ptr_graphics: ptr_screen_view.0, ptr_image: 0, screen_kind}` |
| **오프스크린** FB | `ptr_image = ptr_backing` | `:402-407` — 단 그 `ptr_backing` 이 가리키는 것은 ★**`WIPICFramebuffer`**(`create_backing` `:243-252`)이지 `LgtImage` 가 아니다 |
| `LgtImage{ptr_image, ptr_framebuffer}` **8B** | 이 경로에 **안 쓰인다** | `wipi-types-0.0.2/src/lgt/wipic.rs:67-70` · 유일한 생성처는 이미지 디코드(`graphics.rs:915-927`) |

⇒ ★**실제 간접은 «두 겹»이다**: `핸들 → LgtFramebuffer.ptr_graphics → LgtGraphicsView.ptr_backing`
(`create_view` `:255-266`) `→ WIPICFramebuffer.buf → 픽셀`.

★★**그러므로 코퍼스가 생기면 물을 질문은 이것이다**(리터럴로 박는다):
> **실제 clet 이 `GetScreenFrameBuffer` 반환값을 `WIPICFramebuffer` 로 «직접» 읽나, 아니면
> `ptr_graphics → view.ptr_backing` 을 걷나.**

★「게임이 `ptr_image` 를 보나」로 물으면 **틀린 질문**이다 — 화면 경로에서 그 필드는 **0** 이다.

### 부수 발견 1건 — 이 결함의 원인은 «아니다»

공용 `FrameBuffer::new` 가 upstream 에서 바뀌었다: `bpl = width * bytes_per_pixel` → `buffer_size()` 가
계산한 `bpl`(정렬 포함으로 보인다). ★**KTF 가 그 판본으로 PASS 하므로 이 회차의 원인이 아니다.**
조각 **B**(② hunk 분류)가 `wie_wipi_c` 를 볼 때 **이 축을 알고 보라**.

### 사용자 영향

없음(문서 전용). ★**조각 D 가 「무엇을 결정해야 하는가」가 «27줄 배선»으로 좁혀졌다** — 종전에는
「LGT 가 왜인지 깨진다」였다.

### 다음 조각에 넘기는 것

★**조각 D** — base swap 시 `wie-lgt/src/runtime/wipi_c.rs` 의 graphics 27줄을 **어느 쪽으로 둘지가 결정 항목**이다
(upstream LGT 전용 유지 ⇒ 우리 픽스처·SDK 기반 게스트가 깨진다 / 공용으로 되돌림 ⇒ upstream 의 LGT RE 1,095줄을 버린다).
★**그 결정은 코퍼스 없이는 «안전하게» 내릴 수 없다** — 이 회차가 그 사실을 수로 고정했다.
★**조각 B** — 위 `bpl` 축.
