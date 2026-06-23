# feat(lgt): native-backed JVM foundation for AOT-compiled LGT/ez-i Java apps

## Summary

Adds a foundation in `wie_lgt` for running **AOT-compiled LGT Java apps** (ez-i / Xceed
toolchain), where each Java class is emitted as native ARM code with `.data` metadata
rather than JVM bytecode. The app boots through wie's existing lcdui `Main.main` path
and runs its real native methods, dispatched through reconstructed platform tables.

Reverse-engineered against one ez-i reference app. It reaches **boot + game setup +
first render**: class registration, platform dispatch, data load, 240×320 back-buffer,
`getGraphics`, Cards/RNG/Thread, and — after implementing the show-card import and
driving the card lifecycle — the card's `o.paint` runs each frame and **draws to the
back-buffer** (`fillRect`/`setColor`). The full title (logo/sprites/text) is still in
progress, blocked on an unbound-class vtable misdispatch in the scene resource setup
(see below).

The PoC keeps everything LGT-specific in `LgtJvmShared` (per #1232); shared
`wie_midp` / `wie_wipi_java` classes are **not modified**.

## What's added

- **Native class descriptor parser** (`runtime/java/native_class.rs`, read-only):
  decodes the ez-i class/method/field records in `.data` (handle indirection, 28-byte
  method records, 20-byte field records). Full byte layout in
  `docs/lgt_native_classes.md`.
- **Boot path** (`runtime/init.rs`, `runtime/java/interface.rs`): the `0x64`
  java-interface module — register main metadata (`0x03`), app classes (`0x07`),
  imported platform classes + offset resolution (`0x14`), and invoke-static
  `Main.main` (`0x83`) into the shared lcdui boot.
- **Two-level virtual dispatch** (`runtime/java/native_jvm.rs`): global vtable with a
  **reserved slot 0** (`vtable[virtual_method_offsets[ref] + 1]`), per-class override
  vtables for `java/lang` classes the AOT calls by hardcoded slot
  (Runtime/StringBuffer/Thread), and **inheritance-aware instance field layout**.
- **Object model**: native `new` primitive (stdlib `0x32` / java `0xf`) +
  `<init>`-trampoline binding to JVM instances, `getInstance` singletons
  (java-interface `0xc`), and the native String factory (`0x9`).
- **Unit tests** (`cargo test -p wie_lgt`, 5 tests): the descriptor parser against a
  hand-encoded fixture (header offsets, 28/20-byte record strides, in-`.text`
  code-pointer invariant, handle indirection); the reserved-slot-0 vtable model
  (install slot == dispatch slot for every ref, no slot-0 use, no collisions) and the
  per-class override slots; and the inheritance-aware field layout on a known
  hierarchy. Two pure helpers (`physical_vtable_slot`, `compute_field_layouts`) were
  extracted to make the core invariants testable without a live app.
- **Docs**: `docs/lgt_abi.md` (consolidated, reverse-engineered ABI) and
  `docs/lgt_native_classes.md` (descriptor byte layout).

## Architecture (one paragraph)

Native methods run as real ARM under `wie_core_arm`. They reach the platform via
import thunks (`SVC_CATEGORY_JAVA_INTERFACE`) and per-method trampolines
(`SVC_CATEGORY_JAVA_TRAMPOLINE`) that bridge into wie's JVM. wie reconstructs the
dispatch tables the AOT code reads — a global vtable with reserved slot 0, per-class
overrides for hardcoded `java/lang` slots, and an inherited-first instance field
layout — so virtual/static calls and field access route correctly. Objects are bare
guest blocks (vtable word + 256-word field array) bound to JVM instances at `<init>`.

## Behavior

- ✅ App classes registered; native methods execute as real ARM.
- ✅ Boot: `0x64` java-interface → `Main.main` → `Game` Jlet → `CardCanvas` →
  `Display.setCurrent` (wie's MIDP paint loop ticks).
- ✅ Platform dispatch (two-level vtable, per-class overrides, instance fields),
  `getInstance` singletons, `Thread.start`, game thread spawns `a.run`.
- ✅ Game setup: data load → 240×320 back-buffer → `getGraphics` → Cards/RNG.
- ✅ **Render: the card draws.** Implementing java-interface `0x57` (show-card) +
  driving the card lifecycle (scene-enter `i.a` once, per-frame step `i.aE`) sets the
  `o.g` render gate and runs `o.paint` each frame through wie's MIDP paint loop →
  **`fillRect` / `setColor` draws to the back-buffer** (background fills). First pixels.
- ◑ Full title (logo / sprites / text) not yet complete — blocked on a `vtable[24]`
  misdispatch in the scene resource setup (an unbound-class object's hardcoded vtable
  slot misroutes to `Display.pushCard`; the cp14/25 unbound-vtable problem).

## Deferred (out of scope for this PR)

**ez-i render driver — resolved (cp38–39, `docs/lgt_abi.md` §7).** The §7 open question
("which entry does ez-i invoke per frame") turned out to be a single no-op'd import, not
an unknown ABI: `a.run` hands the platform the title card via java-interface `0x57`
(show-card / `Display.setCurrent`), which wie left as a no-op, so the card was never
pushed and `o.paint` never ran. `LgtJvmShared::show_card` now rebinds the card to its app
class and `pushCard`s it; `drive_card_step` runs the genuine card lifecycle from the
paint tick. `o.g` is set by the app's **own** scene setup (`i.a`'s `@0xdb200` writer,
not a force), and `o.paint` draws. All LGT-specific (`LgtJvmShared`); shared classes
untouched (#1232).

**Remaining for the full title.** The scene resource setup (`i.a(Z)V`) makes a hardcoded
`vtable[24]` call on an object whose class isn't in the import tables, which wie's global
by-name vtable misroutes — the same unbound-vtable issue tracked since cp14/25. Resolving
it (and confirming the precise title scene state) unblocks sprite/image/text rendering.

## Verification

- `cargo test -p wie_lgt` — **5 passed** (descriptor parser, vtable model, field layout).
- `cargo test -p wie_ktf test_helloworld` (clet regression) — **pass**.
- `cargo clippy --workspace --tests` — **clean**.
- `cargo build --workspace` — **builds**.
- Diff contains no ROMs / logs / `.DS_Store` / local task notes (`.gitignore` updated).
