# ez-i / LGT WIPI-Java reference v2 — authoritative (AromaSoft WIPI 1.1.1)

Updates v1 with the **authoritative platform library**. AromaSoft built the LGT WIPI / ez-i
("자바스테이션") platform (confirmed in the official WIPI 2004 deck: *모바일 표준 플랫폼 개발 —
아로마소프트*; *Ez-Java / LGT / Aromasoft*). So **`AromaWIPI_classes.zip` IS the `org.kwis.msp.*`
implementation that BattleMonster's COD/AOTC compiled against** — the 0x64 java-interface imports
are the AOT-compiled equivalents of calls into these exact classes.

## Reference files (commit alongside this doc)

- **`AromaWIPI_classes.zip`** — WIPI 1.1.1 platform class library (bytecode): the real
  `org.kwis.msp.{lcdui,lwc,db,io,handset,media}`, `org.kwis.msf.{core,io}`. Decompile for exact
  method behavior/return values.
- **`AromaWIPI_javadoc.zip`** — official API docs (signatures, params, returns) for every class.
- **`WIPIHeader.h`** — the WIPI **C** API (`MC_grp*`, `MC_knl*`, `MC_GrpFrameBuffer`, `MC_db*`,
  `MC_net*`, …) = the **0x1fb** (WIPI-C) table spec.
- (from SK-VM bundle) `KEmulator-mmpp.jar` — cross-check; `midp3.exe` — native phoneME+ez-i.

## 1. Event model — CONFIRMED (`org.kwis.msp.lcdui.EventQueue`)

`int[15]` ring buffer. `EVENT_SIZE=15`, **`KEY_EVENT=17`, `POINTER_EVENT=19`, `TIMER_EVENT=21`**.
Methods: `getNextEvent(int[])` (blocking dequeue), `postEvent(int[])`, `postEvent(int,int[])`,
`dispatchEvent(int[])`, **`hookEvent(int,JletEventListener)`**, **`setSystemEventListener(int,
SystemEventListener)`**, `removeSystemEventListener(int)`. `Jlet`: `ACTIVE=11/PAUSED=13/DESTROYED=15`.

## 2. §7 driver — CONFIRMED implementable (cp55)

The platform posts `TIMER_EVENT(21)` at frame cadence; the **game's own** `getNextEvent` loop reads
`event[0]` and self-dispatches (BattleMonster dispatcher @0x831xx switches `event[0] in {17,19,21}`,
routing TIMER -> card-update). wie never posts `TIMER_EVENT(21)` -> loop blocks -> 0 draw. cp55 proved
posting `[21,...]` at ~50ms unblocks it (159 iterations, `paint()` per frame). **Implement LGT-AOT-gated
TIMER post.** Cadence: KEmulator `_repaintInterval`/`j2lStyleFpsLimit`; confirm against gameplay video.

## 3. Consumed no-op 0x64 imports -> platform-method mapping (cp53/54 -> resolve via classes.zip)

The 0x64 table is the AOT image of `org.kwis.msp.*` calls. Map each cp54 import by its **usage
signature** to the method below, then implement per the decompiled class / javadoc. Anchor the
index->method numbering off the already-known ones (`0x9`=string, `0xc`=getInstance/getDefaultDisplay).

| import (cp54 sig) | strong candidate(s) | notes |
|---|---|---|
| `0xd (obj,code_ptr,n)` | **`Display.callSerially(Runnable,int)`** | deferred/timer callback -> ties the TIMER loop (s2). Also cf. `EventQueue.hookEvent`/`setSystemEventListener`. cp37 "carried-code 0x1ad4" = the Runnable. |
| `0xb (data,ptr,n)` void | `Display.add/setJletEventListener`, `grabKey(int,listener)`, `EventQueue.hookEvent` | event/listener registration (return ignored). |
| `0xe (1,0,size)->handle` | **`Image.createImage(int,int)`** (mutable) / `createImage(byte[],int,int)` | returns an Image handle stored in a field; null-checked. |
| `0x10 (handle,idx)->field` | `Image.getGraphics()/getWidth()/getHeight()`, `Display.get*` | accessor on a handle. |
| `0x12 (0,0,outbuf)->bool` | `Display.isColor/hasPointerEvents/isDoubleBuffered/where`, or a resource-exists query | branch-on-return; pick by the outbuf usage. |
| `0x1f (0,code/size,n)` | `Card`/displayable register, or `Image` resource | ez-i register; cross-ref classes.zip. |
| `0x22 (0,idx,n)` font | **`Image.createImage(String)/loadImage(String,obs)/getResourceAsStream(String)`** | cp33-35 font-sheet path: the font Image is created here; currently no-op->a0=0->no glyph blit. |

Resource/sprite load (the `field[0x74]` scene-state inputs, cp53) flows through `Image.*` +
`org.kwis.msp.io.File.openInputStream` + `Kernel`/DB (`org.kwis.msp.db.DataBase`). The scene won't
advance past state 8 until the import that loads its resources returns a real handle (not 0).

## 4. Platform API surface (from javadoc — anchors for mapping)

- **Display** (the `0x21` registration target, cp50): `getDefaultDisplay`, `getDisplay(String)`,
  **`pushCard(Card)`**, `popCard`, `removeCard`, `countCard`, **`callSerially(Runnable[,int])`**,
  `flush`, `getWidth/Height`, `isColor`, `numColors`, `hasPointerEvents`, `grabKey/ungrabKey`,
  `set/add/removeJletEventListener`, `where`.
- **Card**: `paint(Graphics)`, `keyNotify(int,int)`, `pointerNotify(int,int,int)`,
  `showNotify(boolean)`, `repaint([iiii])`, `serviceRepaints`, `isShown`, `getDisplay`. (No
  "update" method -> game update is app-internal `i.a/i.b`, driven by the TIMER event it reads.)
- **Image**: `createImage(byte[],int,int)`, `createImage(int,int)`, `createImage(String)`,
  `loadImage(String,obs)`, `getResourceAsStream(String)`, `getGraphics`, `getWidth/Height`,
  `isAnimated`, `play/stop`.
- **Kernel** (`org.kwis.msf.core`): `execute/load`, `getPrgID/getAMID/getParentPrgID`, `getPrgInfo`,
  `stop`, `getAccessLevel`.

## 5. Plan (cp56)

1. LGT-AOT-gated `TIMER_EVENT(21)` cadence driver on `net/wie/EventQueue` (s2). Clet/KTF/SKT inert.
2. Decompile `AromaWIPI_classes.zip` (+ javadoc) -> resolve each consumed 0x64 import (s3 table),
   implement those whose semantics are confirmed (esp. `Image.createImage`/resource-load so
   `field[0x74]` advances and sprites load; `callSerially` for the timer).
3. Validate vs the title oracle (cream bg + BATTLE Monster logo + sprites) and the video cadence.

---

# v3 addendum — the 0x64 ordinal is NOT in any host emulator (and the native surface)

## A. midp3.exe / WIPIEmul.exe RE is a DEAD END for the ordinal table (confirmed)

`lgt_abi.md` (wie's own RE) is explicit: the 0x64 java-interface table is a **global flat ordinal**
— "the SVC id **is** the import index" (anchors: `0x9`=string, `0xc`=getInstance, `0xf/0x32`=new,
`0x21`=pushCard-class, `0x54`=entry-helper). That ordinal numbering is an **on-device AOT-runtime /
COD-link-time** artifact.

The host emulators do **not** contain it — verified:
- **midp3.exe** is a phoneME **bytecode interpreter** (`@_interpreter`, `SlowInterpret()`,
  `Interpreter$Engine`) — resolves natives by name.
- **WIPIEmul.exe** registers natives **by name+descriptor** (JNI-style, e.g.
  `(Ljava/lang/Runnable;)V+callSerially`) — no global ordinal table.
- KEmulator-mmpp is pure-Java bytecode likewise.

⇒ Do not spend a round RE-ing these for the ordinal numbering; it isn't there. The ordinal→method
map can only come from: (B1) an ez-i title in **both** JAR + `binary.mod` form (correlate named
bytecode calls ↔ indexed AOT imports → derive the shared ordinal table), or (B2) a device
import-table trace — **or** be discovered empirically in wie (see C).

## B. The complete native surface (deliverable: `ezi_native_surface.txt`)

Extracted the full java-interface native registry from WIPIEmul.exe: **1200 native methods** (155 on
the render path) as `(descriptor)+name`. This is the **candidate set** the 0x64 ordinal enumerates.
Render-path natives that match the cp54 unknown signatures:

| cp54 import (sig) | native candidate(s) (exact descriptor) |
|---|---|
| `0xe (1,0,size)->handle` | `(II)Lorg/kwis/msp/lcdui/Image;+createImage`, `(Ljava/lang/String;)I+loadImage0` (returns int handle) |
| `0x22 (0,idx,n)` font | `loadImage0(String)I`, `(Ljava/lang/String;)[B+getResource`, `(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Image;+createImage` |
| `0xd (obj,code,n)` | **`(II)V+setEventTimer`** (← per-frame timer!), `(Ljava/lang/Runnable;I)V+callSerially`, `(I)V+callSeriallyRunnable`, `()V+postCallSeriallyEvent` |
| `0x12 (0,0,outbuf)->bool` | `()Z+isColor`, `()Z+hasPointerEvents`, `()Z+hasRepeatEvents` |
| `0xb (data,ptr,n)` void | `(ILorg/kwis/msp/lcdui/JletEventListener;)V+hookEvent`, `setSystemEventListener`, `addJletEventListener` |
| `0x10 (handle,idx)->field` | `()I+getWidth/getHeight`, `()Lorg/kwis/msp/lcdui/Graphics;+getGraphics`, clip getters |

**★ `setEventTimer(int,int)`** is the explicit per-frame driver native: game calls it, platform posts
`TIMER_EVENT(21)` at that interval. Pairs with §2. Also note low-level primitives
`createImage0/1/Basic/Ext`, `decodeNextImage*`, `loadImage0` — the AOT imports map to these **native
primitives**, not the public Java wrappers.

## C. The path that works NOW: in-wie empirical reference-guided probe

Because the candidate semantics are known (above) and the oracle is sharp (`field[0x74]` advance /
real draw), the unknown consumed indices `{0xb,0xd,0xe,0x10,0x12,0x1f,0x22}` can be cracked **inside
wie** by structured probing — implement a candidate's real semantics on an index, run, and KEEP it
only if the game's **own** code then advances (return flows through app logic to a real scene-advance).
That game-driven advance is the signature of a *correct* import — categorically different from forcing
a gate. Bounded set + known candidates + clear oracle = tractable without the device.
