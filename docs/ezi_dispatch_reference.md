# ez-i / LGT WIPI-Java per-frame dispatch — reference (from real emulators)

Extracted from two reference implementations of the LGT ez-i / DownTown platform:

- **KEmulator-mmpp.jar** (pure Java; `SK-VM, ez-i 에뮬레이터`) — contains
  `org/kwis/msp/lcdui/{Jlet,EventQueue,JletStateChangeException}` + the host MIDP
  dispatch core `emulator/EventQueue` + `emulator/lcdui/*`.
- **midp3.exe** (`LGT_MIDP_Emulator`) — native LGT ez-i MIDP runtime, **phoneME-derived**
  (`com/sun/midp/...`) with LGT ez-i extensions (`initializeEzi`, `comeToEzI`, `EZI_*`,
  `LGTSMSParser`, `setEziPasswd`).

Both are **host re-implementations that run the JAR (bytecode)** form, not the AOT
`binary.mod`. So they give the **platform Java-API contract + dispatch semantics**, which is
exactly the §7 unknown — not the on-device 0x64 native import index table (that still needs
mapping, see §4).

---

## 1. The ez-i event model (CONFIRMED, concrete constants)

`org.kwis.msp.lcdui.EventQueue` — a producer/consumer **ring buffer of `int[15]` events**:

| const | value |
|---|---|
| `EVENT_SIZE` | **15** (each event is `int[15]`) |
| `KEY_EVENT` | **17** |
| `POINTER_EVENT` | **19** |
| `TIMER_EVENT` | **21** |

Methods (all `([I)V`):
- `postEvent([I])` — enqueue a 15-int event (producer; platform side).
- `getNextEvent([I])` — **blocking dequeue** into the caller's array (`wait()`/`notifyAll`,
  `System.arraycopy`). ← this is the method wie crashed in (`net/wie/EventQueue.getNextEvent`).
- `dispatchEvent([I])` — **EMPTY stub** in this impl. ⇒ the routing to the card is **NOT** in
  the platform class; the **game's own run loop** reads `event[0]` (the type) and dispatches itself.

`org.kwis.msp.lcdui.Jlet` lifecycle (CONFIRMED):
- states `ACTIVE=11 / PAUSED=13 / DESTROYED=15`; fields `eventQueue`, static `activeJlet`.
- `setActiveJlet / getActiveJlet / getJletFromPID(I) / getCurrentJlet / getCurrentProgramID`,
  `startApp([String]) / pauseApp / resumeApp / destroyApp(Z) / notifyDestroyed`,
  `getAppProperty(String) / getEventQueue() / removeAllResource(I)`.

## 2. The per-frame CADENCE (CONFIRMED, KEmulator core)

`emulator/EventQueue` (the host MIDP dispatch core) drives frames with a **screen timer**:
- fields: `screenTimer:java.util.Timer`, `screenTimerTask:TimerTask` (`ScreenTimerTask extends
  TimerTask`), `_repaintInterval`, `_fpsLimiter`, `j2lStyleFpsLimit`, `repaintPending`,
  `repaintX/Y/W/H`.
- `ScreenTimerTask.run()` is `schedule`d at a fixed interval → posts repaint/timer →
  `queueRepaint → internalRepaint → serviceRepaints → paint`.
- i.e. a **fixed-interval timer** posts the per-frame tick; cadence = the fps limiter.

## 3. ★ Re-diagnosis of wie's §7 (the model cp37–54 had WRONG)

cp42/cp52 assumed §7 = "the ez-i runtime selects & dispatches a method on a registered **bare
handle**." The reference shows that is **not** the model. The real model:

1. The platform **posts a `TIMER_EVENT` (type 21) into the EventQueue at frame cadence** (the
   screen timer), plus `KEY_EVENT` (17) on keypress, `POINTER_EVENT` (19) on touch.
2. The **game's own loop** calls `getNextEvent([I])`, reads `event[0]`, and dispatches itself:
   `TIMER_EVENT → advance scene-state + repaint`, `KEY_EVENT → key handler`.
3. The card update (`i.a`/`i.b` → sets `o.g`) and scene-state advance (`field[0x74]`) are driven
   **by the game off the TIMER_EVENT**, not by a runtime method-dispatch on a bare handle.

**⇒ The actual wie gap: wie never posts `TIMER_EVENT(21)` into its `EventQueue` at a frame
cadence.** So the game's `getNextEvent` loop never receives the per-frame tick → `i.a`/`i.b`
never run → `o.g=0`, `field[0x74]` stuck at 8 → 0 draws. cp50 wired `pushCard→paint` (the PAINT
half) but not the **TIMER_EVENT update half**.

This is **implementable inside wie without the proprietary runtime**: post a 15-int event with
`event[0]=21` at cadence, deliver it via the game's existing `getNextEvent` path; the game's own
code does the scene-advance + paint. (This is reconstruction of the standard ez-i EventQueue
timer model — confirmed by two reference emulators — not forcing.)

Open: the exact `event[1..14]` payload for `TIMER_EVENT` (timer id? elapsed ms?) and cadence ms.
Recover the payload from the game's `getNextEvent` consumer disasm; recover/confirm cadence from
KEmulator's `_repaintInterval`/`j2lStyleFpsLimit` and the gameplay video frame rate.

## 4. Still to map: the consumed no-op 0x64 imports

cp53/54's `{0xb,0xd,0xe,0x10,0x12,0x1f,0x22}` are AOT-call equivalents of platform Java methods.
The reference emulators implement those methods (kwis + the phoneME `com/sun/midp` + ez-i
extensions). Map LGT `0x64` index → `org.kwis.msp.*`/ez-i method **by the app's usage signature**
(cp54 table) cross-referenced against the **decompiled reference implementations**, then implement
each import per the reference's semantics. Note `0xd=(obj,code_ptr,n)` is plausibly the
**event-handler / timer-callback registration** that wires §3's TIMER path — re-interpret it in
the EventQueue model.

## 5. Files

- `KEmulator-mmpp.jar` → `org/kwis/msp/lcdui/*`, `emulator/EventQueue*`, `emulator/lcdui/*`,
  `emulator/Emulator*`, `javax/microedition/lcdui/*`. Decompile these (CFR/procyon/jadx).
- `midp3.exe` → native phoneME+ez-i; strings/IDA/Ghidra for the ez-i additions (`initializeEzi`,
  `EZI_*`) and, if present, the AOT/native dispatch — the closest artifact to wie's on-device ABI.
