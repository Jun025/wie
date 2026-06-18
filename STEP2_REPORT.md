# STEP 2 — LGT native-backed JVM (implementation pass)

Goal: register the LGT java app's native classes with the JVM and dispatch their
methods to **real ARM code**, so the app's own AOT methods (`Game.<init>`,
`a.<init>`, `a.startApp`, …) execute natively.

Reference app: BattleMonster `00025C2B`. Branch: `feat/lgt-java-interface-bridge`
(local only). Builds on the RE pass (`docs/lgt_native_classes.md`,
`wie_lgt/src/runtime/java/native_class.rs`).

Approach agreed in Discussion #1232: an LGT-specific PoC object model is fine; this
pass covers dispatch of the **app's own** methods. Calls into platform classes go
through the `java_load_classes` method/offset tables — that is checkpoint 3.

## Status summary

| item | result |
|---|---|
| `NoClassDefFoundError: Game` resolved | ✅ yes |
| `Game` instantiated (`new Game()`) | ✅ yes |
| App methods run as **real ARM** | ✅ yes — `Game.<init>` → `a.<init>` execute natively |
| Next stop | ⏹ platform method-table call in `a.<init>` (= **checkpoint 3**) |
| Clet path regression (`test_helloworld`) | ✅ still passes |
| clippy | ✅ clean |

The earlier interim (`<init>` super-chaining through the JVM, which reached a blank
event loop) has been **removed** in favour of real dispatch. With real dispatch the
boot now stops earlier — at the first platform call — which is the correct,
expected checkpoint-2 endpoint.

## Checkpoint 1 — register native classes ✅

`register_app_classes` (in `native_jvm.rs`) scans the app's `.data` (range captured
from the ELF in `load_executable`) for class headers, parses each via
`native_class.rs`, and registers all **20** app classes with the JVM in
superclass-dependency order (`register_class` resolves the parent eagerly). No-op
for clet apps (no descriptors in `.data`), so the clet path is unaffected.
`resolve_class` then finds `Game` → `NoClassDefFoundError` gone.

## Checkpoint 2 — ARM-backed object model + real dispatch ✅

### Object model (`wie_lgt/src/runtime/java/native_jvm.rs`)

Custom `jvm` trait impls (pure-Rust metadata; no guest reflection, unlike wie_ktf):

- **`LgtClassDefinition`** — `name`/`super`/`access`, method/field lookup, static
  fields (Rust map). `instantiate()` allocates a **guest object block**: a 12-byte
  header whose **`+0x08` points to a zeroed field array**. This matches the layout
  the AOT code requires — observed `r1 = [this, #8]; str rX, [r1, idx<<2]`.
- **`LgtClassInstance`** — identified by its guest pointer (`guest_ptr`). JVM-side
  `get_field`/`put_field` use a separate Rust map (enough for the platform
  `Jlet`/`Display` glue that touches inherited fields). Unifying that with the guest
  field array requires the platform field-offset table → checkpoint 3.
- **`LgtMethod::run`** — real dispatch:
  1. marshal JVM args → ARM `r0..r3` (+stack): `this` (args[0]) and object args
     become guest pointers; primitives become raw words (`marshal_arg`).
  2. `core.run_function(code_ptr, &params)`.
  3. marshal the return per the descriptor's return type (`marshal_return`).
- **`LgtField`** — name/descriptor/access.

### Runtime helpers (lazily resolved during dispatch)

Native bodies resolve AOT-runtime helpers through the java-interface import table
(`0x64`) *while executing*. Observed during `Game.<init>`: imports `0x54`, then
`0xb`/`0xc`/`0xd`. These are stubbed (no-ops returning 0) so dispatch advances:
`0x54` has a dedicated handler; other unknown java imports route to a generic logged
no-op (`java_interface_stub`). The `.data` trampoline at `0x140466c` (real app code)
runs as-is. (Implement these properly as they prove to need real behaviour.)

### Reach (verified by ARM trace)

```
register 20 app classes
new Game -> LGT instantiate Game (guest object) -> invoke <init>
LGT dispatch Game.<init>()V code=0x10c8       <- real ARM
   helper(0xc)=import 0x54 [stub], 0x1908, ...
   bx 0x194c  -> a.<init> (superclass ctor)   <- real ARM
       helper(0xb) [stub] returns to 0x1970
       ldr ip,[r4,#0x108] ; bx ip  with r4=0x1500820  -> bx 0
=> java/lang/Error "Invalid memory access; address: 0"
```

So `Game.<init>` and its superclass `a.<init>` run as real ARM. Execution stops in
`a.<init>` at `bx [0x1500820 + 0x108]`: `0x1500820` is the platform method table in
`.bss` (the `java_load_classes` `static_method_offsets` output), still **null**
because `java_load_classes` is a stub. This null call is the next blocker.

## Checkpoint 3 (next) — platform method/field offset tables

The single remaining blocker is exactly what the maintainer flagged: external
(platform) Java API references are **vtable/offset-indexed**. `java_load_classes`
must populate the `.bss` tables so native calls like `bx [0x1500820 + N]` and field
accesses via `[0x15006f4 + idx]` resolve to the platform implementations
(`org/kwis/msp/lcdui/{Jlet,Card,Graphics,Image,Display}`, `java/lang/*`, …). Each
slot needs to point at a trampoline that re-enters the JVM and invokes the
corresponding platform method/field (decoded from the `java_load_classes` imported-
class tables in `docs/lgt_native_classes.md`). Once wired:
`a.<init>` → `Jlet.<init>` (real) → `setCurrentJlet`/Display/EventQueue → `startApp`
→ Card `o` `paint(Graphics)` → toward the title screen.

## Reproduce

```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# look for: "registered 20 app classes", "LGT dispatch Game.<init>()V",
# then the Error at the platform method-table call (address 0).
```

## Module layout (kept separate per the agreed PoC design)

- `native_class.rs` — read-only descriptor parser (RE pass).
- `native_jvm.rs` — ARM-backed object model + real dispatch + class registration.
- `init.rs` — captures `.data` range, registers app classes before the initializer;
  routes the lazily-resolved runtime-helper imports.
- `interface.rs` / `svc_ids.rs` — java-interface import handlers/ids incl. the
  runtime-helper stubs.
