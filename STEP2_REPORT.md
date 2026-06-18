# STEP report — LGT native-backed JVM (checkpoints 1–3)

Goal: run the AOT-compiled LGT app (BattleMonster `00025C2B`) on wie's JVM —
register its native classes, dispatch its methods to real ARM, and bridge calls
into the platform classes (`Jlet`/`Card`/`Graphics`/`Display`/`java/lang/*`).

Branch `feat/lgt-java-interface-bridge` (local only). Builds on the RE pass
(`docs/lgt_native_classes.md`, `native_class.rs`). PoC object model agreed in
Discussion #1232 (LGT-specific, not over-engineered).

## Status summary

| item | result |
|---|---|
| `NoClassDefFoundError: Game` resolved | ✅ |
| `Game` instantiated; app methods run as real ARM | ✅ |
| `java_load_classes` fills the platform method/field tables | ✅ |
| native → platform calls dispatch by name | ✅ (Jlet.<init>, getDefaultDisplay, getRuntime, System.gc, BackLight.alwaysOn, Graphics.drawLine, …) |
| `a.<init>`→`Jlet.<init>` wires the Jlet; `a.startApp` reached as real ARM | ✅ |
| Card `o`.paint(Graphics) / title screen | ❌ blocked on per-class platform vtables |
| clet regression (`test_helloworld`) | ✅ | clippy | ✅ |

## Checkpoint 1 — register native classes ✅
`register_app_classes` scans `.data` for class headers, parses them, and registers
all 20 app classes (superclass-dependency order). No-op for clet apps.

## Checkpoint 2 — ARM-backed objects + real dispatch ✅
Custom `ClassDefinition`/`ClassInstance`/`Method`/`Field`. Instances are guest
object blocks (`this+0x08` -> field array). `LgtMethod::run` marshals `this`+args
into `r0..r3`, `run_function(code_ptr)`, marshals the return.

## Checkpoint 3 — `java_load_classes` + native↔platform bridge ✅ (core), ⏹ (per-class vtables)

### Trampoline design (`native_jvm.rs`)

- **Shared runtime** `LgtJvmShared`: an instance registry (`guest_ptr ↔ ClassInstance`),
  the trampoline table, and the virtual-method-table base.
- **Object bridge**: `value_to_guest` turns a JVM value into the guest word the AOT
  code expects — an app instance yields its `guest_ptr`; a platform object gets a
  freshly-allocated **proxy block** registered in the map; primitives pass raw.
  `guest_to_value` is the inverse (a guest pointer round-trips to its JVM object).
- **`install_platform_tables`** (the real `java_load_classes`): reads the imported-
  class table and each class's `virtual/static` method ranges; for every requested
  method it creates a **native→platform trampoline** (an SVC stub in
  `SVC_CATEGORY_JAVA_TRAMPOLINE`) and writes the stub pointer into the fixed-offset
  output table the AOT code reads — `static_method_offsets[idx*4]` /
  `virtual_method_offsets[idx*4]` (index = the global method-array index, matching
  the AOT's baked offset). `field_offsets[idx]` gets a distinct guest slot.
- **`handle_java_trampoline`**: on a native→platform call, reads `this` (`r0`, via
  the instance registry) + args (`r1..`, per the descriptor), invokes the matching
  `wie_wipi_java`/`wie_midp` method by name+descriptor (`<init>`→invoke_special,
  static→invoke_static, else invoke_virtual), and marshals the return into `r0`.
- **Vtable word**: every object carries the `virtual_method_offsets` base at
  `+0x00`, so the AOT's virtual dispatch `r3=[this]; bx [r3 + idx*4]` lands in the
  table.

### Reach (verified by trace)
```
java_load_classes: filled 128 method slots, 1 field slot
new Game -> Game.<init> [real ARM]
  -> a.<init> [real ARM] -> trampoline Jlet.<init>()V (wires currentJlet/Display/EventQueue)
  -> trampoline BackLight.alwaysOn, Runtime.getRuntime, System.gc, Display.getDefaultDisplay,
     Graphics.drawLine, Component.getHeight ...
-> a.startApp([Ljava/lang/String;)V [real ARM]
```
Static/special platform dispatch is correct (methods invoked by their real names).

### Next stop — per-class platform vtables

The AOT virtual-dispatches some platform methods through **hardcoded vtable
indices** baked per the *original* platform's class layout, e.g.
`Runtime.getRuntime().<vtable 14>()` and `Component.getHeight()`. Our single global
`virtual_method_offsets` table can't disambiguate index 14 across classes (it holds
`Graphics.drawLine` at 14), so the call resolves to the wrong method on a `Runtime`
(`NoSuchMethodError: Runtime.drawLine`).

A PoC fallback (swallow `NoSuchMethodError`, return 0) advances execution to
`a.startApp`, but then **hangs**: `Component.getHeight()` returns 0, so the native
layout code loops forever. This confirms (and matches the maintainer's warning)
that no-op/0 returns diverge — **correct per-class platform vtables are required**:
each platform class needs its own vtable whose slot *i* is that class's actual
vtable method *i* (inherited + declared), matching the original LGT vtable
ordering. The import tables alone don't encode that ordering (e.g. `Runtime`
declares 0 virtual methods yet a vtable-14 call is made), so this needs a
platform vtable-index spec / per-class vtable construction. Nothing is drawn yet
(the hang precedes any `paint`).

### Recommended next step
1. Build a per-class vtable for each imported platform class: resolve its full
   virtual-method list in vtable order (Object + supers + declared) against the
   `wie_wipi_java`/`wie_midp` definitions; store each class's vtable base.
2. Set each object's `+0x00` to its class's vtable base (instead of the single
   global table); proxies use the platform class's vtable.
3. Remove the `NoSuchMethodError→0` fallback once vtables are correct.
4. Re-run: `getHeight()` returns the real height, `startApp` builds the Card `o`,
   pushes it, and `paint(Graphics)` is reached → toward the title screen.

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# look for: "filled 128 method slots", the trampoline platform calls, "a.startApp",
# then the per-class-vtable warning / layout-loop hang.
```

## Module layout
- `native_class.rs` — read-only descriptor parser (RE pass).
- `native_jvm.rs` — ARM-backed object model, native↔platform bridge, trampolines,
  `java_load_classes` table fill, class registration.
- `init.rs` — captures `.data`, registers the trampoline handler + app classes,
  threads `LgtJvmShared`.
- `interface.rs` / `svc_ids.rs` — java-interface imports incl. runtime-helper stubs.
