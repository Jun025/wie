# feat(lgt): native-backed JVM foundation for AOT-compiled LGT/ez-i Java apps

## Summary

Adds a foundation in `wie_lgt` for running **AOT-compiled LGT Java apps** (ez-i / Xceed
toolchain), where each Java class is emitted as native ARM code with `.data` metadata
rather than JVM bytecode. The app boots through wie's existing lcdui `Main.main` path
and runs its real native methods, dispatched through reconstructed platform tables.

Reference app: BattleMonster (`00025C2B`). The app reaches **full boot + game setup**
(class registration, platform dispatch, data load, 240×320 back-buffer, `getGraphics`,
Cards/RNG/Thread). The **per-frame render driver does not yet run** — it depends on the
ez-i displayable/clet tick ABI, which is not derivable from the app binary and is the
one documented open question (see below).

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
- **Docs**: `docs/lgt_abi.md` (consolidated ABI), `docs/lgt_native_classes.md`
  (descriptor RE). `STEP2_REPORT.md` / `BRIDGE_REPORT.md` are the checkpoint
  derivation logs.

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
- ⛔ **Title pixels not yet drawn** — 0 draw calls; blocked on the render-tick ABI.

## Deferred (out of scope for this PR)

**ez-i render-tick ABI.** The app `new`s a bare native object, hands it to platform
import `0x21`, and registers callbacks via `0x55`/`0x56`. That object is **unbound**
(no `<init>`, no JVM class), so there is no identifiable per-frame method to call — the
ez-i runtime is what invokes the registered object's paint each frame, and wie does not
emulate that loop. `a.run` is a confirmed one-shot (it returns after registration). The
single missing fact, recorded in `docs/lgt_abi.md` §7:

> Which registered object's native entry does the ez-i runtime invoke per frame to
> paint, and how does its back-buffer flush to screen (the ez-i `DisplayProxy.flush`
> equivalent / the displayable/clet ABI that `0x21`/`0x55`/`0x56` bind)?

With that, the per-frame call can be driven from `LgtJvmShared` on wie's existing paint
tick — no shared-class changes needed. PoC `LgtJvmShared` stays LGT-specific (#1232).

## Verification

- `cargo test -p wie_ktf test_helloworld` (clet regression) — **pass**.
- `cargo clippy -p wie_lgt` — **clean**.
- `cargo build --workspace` — **builds**.
- Diff contains no ROMs / logs / `.DS_Store` / local task notes (`.gitignore` updated).
