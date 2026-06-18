# STEP 2 — LGT native-backed JVM (implementation pass)

Goal: register the LGT java app's native classes with the JVM and dispatch their
methods to ARM code, so `Main.main` can instantiate `Game` (break
`NoClassDefFoundError`) and drive `Game`/`startApp`.

Reference app: BattleMonster `00025C2B`. Branch: `feat/lgt-java-interface-bridge`
(local only). Builds on the RE pass (`docs/lgt_native_classes.md`,
`wie_lgt/src/runtime/java/native_class.rs`).

## Status summary

| ask | result |
|---|---|
| `NoClassDefFoundError: Game` resolved | ✅ yes |
| `Game` instantiated (`new Game()`) | ✅ yes (`Game.<init>` invoked) |
| `Game`/app `startApp` reached | ✅ yes — `a.startApp([Ljava/lang/String;)V` is invoked |
| Title screen / first battle | ❌ not yet — needs real ARM dispatch (see blocker) |
| Clet path regression (`test_helloworld`) | ✅ still passes |

Boot now runs to a **stable MIDP event loop** with no crash; the screen is blank
because the native `startApp` body is still a stub.

## Checkpoint 1 — register native classes ✅

`wie_lgt/src/runtime/java/native_jvm.rs`:
- `scan_class_headers`: walks the app's `.data` segment (range captured from the
  ELF in `load_executable`) for class headers matching the documented layout.
- `build_class_definition`: converts each parsed `LgtNativeClass` into a
  `jvm_rust::ClassDefinitionImpl` — fields from the field records, methods from the
  method records with a custom `MethodBody` (`LgtNativeMethodBody`). Built directly
  via `ClassDefinitionImpl::new` + `MethodImpl::from_method_proto` so runtime
  (obfuscated) names work without `&'static`.
- `register_app_classes`: registers all classes in **superclass-dependency order**
  (`register_class` resolves the parent eagerly, so e.g. `a` must precede `Game`,
  `o` before `b/d/e/j/l`). Platform parents (`Jlet`, `Card`, `Object`) resolve via
  the bootstrap loader.

Wired in `init.rs` before the app initializer runs. No-op for clet apps (their
`.data` has no class descriptors), so the clet path is unaffected.

Result: all **20** app classes register
(`a c f g h k m n o p r s Game b d e i j l q`), `resolve_class` finds `Game` via the
registered-classes map, and `Game.<init>` is invoked — `NoClassDefFoundError` gone.

## Checkpoint 2 — method dispatch ⚠️ (interim; real ARM dispatch blocked)

**Blocker (structural):** real dispatch — marshal JVM args → ARM `r0..r3`,
`core.run_function(code_ptr)`, marshal the return — requires the app's **objects to
be ARM-memory-backed**, because:
- every instance method receives `this` as a guest pointer, and native bodies
  operate on it directly. Disassembly of `Game.<init>` (`0x10c8`):
  ```
  mov ip, sp ; push {r4,r5,r6,fp,ip,lr,pc} ; ldr r3,[pc,#0xe0]
  mov r4, r0        ; r4 = this
  mov r0, #0xc ; mov lr, pc ; bx r3   ; call runtime helper
  ```
- object args/returns must marshal to/from guest pointers. In `wie_ktf` this works
  because its `JavaClassInstance` is ARM-backed (`ptr_raw`); our classes use
  `jvm_rust::ClassInstanceImpl`, which has no guest-memory backing.

So real dispatch needs a custom **ARM-backed `ClassInstance`/`ClassDefinition`**
(allocate a guest object block on `instantiate`, lay fields out per the descriptor
`index`, dispatch methods via `run_function`, marshal values like
`wie_ktf/.../jvm_support/value.rs`). This is the KTF-scale object-model port and is
the **pending architectural decision** with the maintainer, so it is intentionally
not implemented here.

**Interim implemented** (isolated in `LgtNativeMethodBody::call`, easy to remove):
a parameterless `<init>` is chained to its superclass `<init>` through the JVM.
Recursively this runs `Game.<init>` → `a.<init>` → `org/kwis/msp/lcdui/Jlet.<init>`
(the real platform constructor), which calls `setCurrentJlet(this)` and creates the
`Display`/`EventQueue`. All other native methods remain logging stubs returning a
type-appropriate default.

Observed boot with the interim:
```
register 20 app classes
new Game -> Game.<init> -> a.<init> -> Jlet.<init> (setCurrentJlet(Game), Display, EventQueue)
Launcher.startMIDlet -> WIPIMIDlet.startApp -> a.startApp([Ljava/lang/String;)V  [stub]
Launcher spawns event loop -> EventQueue.getNextEvent/dispatchEvent looping (no crash)
```

## Checkpoint 3 — `java_load_classes` offset tables ❌ not started

Filling the `*_offsets` output tables (so native code can call platform methods like
`Graphics`/`Image`/`Display` by resolved index) only becomes exercisable once native
bodies actually run (checkpoint 2 real dispatch). Deferred behind the object model.

## Checkpoint 4 — wire + advance to title ⚠️ partial

The java-app path is wired and boots to the running event loop. Reaching the title
screen requires the native `a.startApp` (`0x1ad8`) to execute: it creates the Card
subclass `o` (which has `paint(Lorg/kwis/msp/lcdui/Graphics;)V`@`0xd8d70`,
`keyNotify`), pushes it on the `Display`, and starts the game threads. That is real
ARM dispatch → blocked as above.

## Furthest point reached & next stop

- **Furthest:** `new Game()` succeeds; the platform `Jlet` is initialised; the app's
  `startApp` is invoked; the MIDP event loop runs steadily with no crash.
- **Next stop / cause:** the screen stays blank because `a.startApp` and all other
  native bodies are stubs. The single blocker for everything beyond this is the
  **ARM-backed instance object model** (custom `ClassInstance`/`ClassDefinition` +
  value marshaling), which is the pending structural decision.

### Recommended next step (once the object model is agreed)

1. Add `LgtClassInstance` (guest object block; `instantiate` allocates via
   `Allocator`, field offsets from descriptor `index`) and a `LgtClassDefinition`
   (or extend `ClassDefinitionImpl` to allocate guest backing).
2. Implement `LgtNativeMethodBody::call` for real: marshal `this`+args into `r0..r3`
   (+stack), `run_function(code_ptr)`, marshal the return (mirror
   `wie_ktf/.../value.rs`). Remove the interim `<init>` chain.
3. Resolve & fill `java_load_classes` `*_offsets` so native→platform calls dispatch.
4. Re-run; expect `a.startApp` → push Card `o` → `paint(Graphics)` → toward title.

## Reproduce

```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=debug,wie_core_arm=warn \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# look for: "registered 20 app classes", the <init> chain, WIPIMIDlet.startApp,
# a.startApp stub, then EventQueue.getNextEvent/dispatchEvent looping.
```

## Module layout (kept separate per maintainer-pending design)

- `wie_lgt/src/runtime/java/native_class.rs` — read-only descriptor parser (RE pass).
- `wie_lgt/src/runtime/java/native_jvm.rs` — class registration + method bodies
  (this pass). The real object model would slot in here / a sibling module.
- `wie_lgt/src/runtime/init.rs` — captures `.data` range, registers app classes
  before the app initializer.
