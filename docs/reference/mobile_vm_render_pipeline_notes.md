# Mobile-VM render/event pipeline — general conceptual notes

**Scope & limits (read first).** This is *general* feature-phone Java-VM architecture
context to orient porting work — it is **NOT** the LGT ez-i `0x64` java-interface ABI
spec, and it does **not** supply the missing `0x64` ordinal→native table (see
`docs/lgt_abi.md` and `ORDINAL_*` reports). Nothing here is authoritative for wie's
runtime; no ordinal/handle may be bound on the basis of these notes. No copyrighted
document is reproduced — this is a plain-language summary of widely-documented mobile-VM
concepts (MIDP/WIPI/GVM-class platforms), cross-checked only against files already in
`docs/reference/` (`ezi_dispatch_reference.md`, `WIPIHeader.h`).

## Per-frame drive (event model)
On these platforms the *platform* owns the frame clock: it posts a **timer event** at a
frame cadence into an event queue; the **application's own loop** dequeues it and calls the
app's update/paint. There is no platform "update()" callback — the displayable exposes only
`paint(Graphics)` + input notifies (confirmed for ez-i in `ezi_dispatch_reference.md`
§Card/EventQueue). So a headless/emulated host that never posts the timer event leaves the
app's dequeue loop blocked → no frames. (This is why wie's LGT §7 work centers on an
LGT-AOT-gated `TIMER_EVENT(21)` driver; see docs/lgt_abi.md cp55.)

## Render path (buffers)
Typical order: **resource decode** (image bytes → raster) → draw into an **offscreen/virtual
back buffer** → composite/blit to the **front buffer** → **flush to LCD** (double-buffered).
The WIPI-C surface in `WIPIHeader.h` mirrors this: `MC_grpCreateOffscreenFrameBuffer` /
`MC_grpGetScreenFrameBuffer` (buffers), `MC_grpDrawImage`/`MC_grpFillRect` (raster ops),
`MC_grpFlushLcd` (present). The scene will not advance to first draw until the resource-load
primitive returns a *real* handle (a no-op returning 0 blocks the decode→raster step).

## Why this doesn't unblock §7 by itself
The concepts above fix the *shape* of the pipeline, not the *numbering*: which `0x64`
ordinal is `createImage` vs `getScreenFrameBuffer` vs a query is a property of the native
runtime's import registration order, absent from both the concepts here and the app binary
(see ORDINAL_TRIANGULATION / ORDINAL_DUALFORM). Required artifact remains external (native
registry in registration order, or a named-bytecode dual title, or a device import trace).
