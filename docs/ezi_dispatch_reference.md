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
