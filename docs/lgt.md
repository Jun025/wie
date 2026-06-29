# LGT Platform Architecture

LGT (LG Telecom) is a carrier, and LGT devices shipped with their own WIPI implementation. Currently only **Clet** (C native apps) execution is implemented. Java app support is not yet implemented.

## App Structure

An LGT app consists of:
- A JAR containing:
  - `binary.mod` — an ARM ELF executable
  - App resources
- An `app_info` file (separate from the JAR) — app descriptor (AID, PID, MClass)

### Clets

C native apps compiled as standard ARM ELF binaries. Unlike KTF's raw binary format, LGT uses proper ELF with section headers, allowing standard loading at specified addresses.

### Java Apps

LGT Java apps are **AOT-compiled to ARM native code inside `binary.mod`** — the same model
as KTF's `client.bin`, not `.class` files run on a Rust JVM. (H1, confirmed by RE across
`docs/lgt_abi.md` cp1–cp48: class/method metadata structures live in the ELF `.data` segment,
method bodies are ARM code at `.text` pointers; `register_app_classes` scans `.data` and
registers each as a JVM class whose methods dispatch to ARM via `core.run_function`.) The app
is an **ez-i** (LGE's WIPI Java profile) Jlet built on `org.kwis.msp.lcdui`.

## Platform Interfaces

LGT uses its own WIPI-side import-table mechanism instead of KTF's direct callback approach.

### Import Table System

During initialization, the native binary receives platform callbacks for import resolution:
- one callback identifies an import table
- another resolves a function pointer from a table ID and function index

The binary uses these callbacks to resolve each platform function it needs. Known tables:

| Table ID | Purpose |
|----------|---------|
| `0x1fb`  | WIPI C functions (kernel, graphics, etc.) |
| `0x64`   | Java interface functions |
| `0x1`    | C standard library (memcpy, strlen, etc.) |

### WIPI C Interface

Provides the LGT-side WIPI C surface (kernel, graphics, database, timer, etc.), but delivered through the import table rather than a named interface pointer.

### Standard Library

LGT-specific: provides C standard library functions (memcpy, strlen, etc.) that the native binary expects from the platform. KTF binaries include these in their own binary; LGT imports them.

## Initialization Sequence

1. Platform parses `binary.mod` as ELF, loads sections into memory at their specified addresses
2. Calls the ELF entrypoint with platform-owned initialization blocks
   - one of these blocks contains the import-resolution callbacks
3. The binary stores the import-resolution callbacks and uses them on demand when platform functions are needed
4. The binary returns a pointer to a structure containing its initialization entry
5. Platform calls that initialization entry to start the app

## Key Differences from KTF

| Aspect | KTF WIPI | LGT WIPI |
|--------|----------|----------|
| Binary format | Raw ARM (`client.bin`) | ELF (`binary.mod`) |
| Function binding | Direct callback pointers | Import table lookup |
| Java integration | AOT-compiled into ARM binary | **AOT-compiled into ARM ELF** (`binary.mod`) — same model (H1) |
| C stdlib | Included in binary | Provided by platform |
| Per-frame render driver | app self-loop (`Thread.run` game loop) | **`EventQueue` `TIMER_EVENT(21)` at frame cadence** drives the app's own `getNextEvent` loop (cp55) — *see below* |

## How We Emulate This

- **ARM execution**: Same `wie_core_arm::ArmCore` as KTF.
- **ELF loading**: Uses the `elf` crate to parse sections and load them at their specified addresses.
- **Import table**: Rust callbacks map `(table_id, function_index)` pairs to registered function addresses for WIPI C, Java interface, and stdlib functions.
- **JVM**: Clets run on `RustJavaJvmImplementation` (pure Rust JVM). **AOT-Java apps** run on the LGT
  native-JVM bridge (`runtime/java/native_jvm.rs`): class/method metadata is parsed from `binary.mod`
  `.data`, methods dispatch to ARM bodies via `core.run_function`, and `new`→`<init>` binds the guest
  object block to a JVM instance (`alloc_native_object`/`bind_pending`).

## The §7 wall: AOT-Java per-frame render driver (open)

KTF AOT-Java titles render because the **app** spawns its own game-loop thread (`Thread.run`)
that does logic + `repaint()` each frame; wie drives it via the cooperative scheduler. LGT ez-i
apps drive per-frame work off the **`org.kwis.msp.lcdui.EventQueue`**: the platform posts a
`TIMER_EVENT` (type **21**) at frame cadence, and the app's own `getNextEvent` loop reads
`event[0]` and self-dispatches (`dispatchEvent` is a stub in ez-i — see
`docs/ezi_dispatch_reference.md`). **wie never posts `TIMER_EVENT(21)`**, so the app's
`getNextEvent` loop blocks forever and never ticks (cp55). *(Earlier cp42/52 wrongly modelled this
as "the runtime dispatches a method on a registered bare handle"; cp55 corrects it.)*

cp55 confirmed this in-binary (BattleMonster's dispatcher @0x831xx switches `event[0]`∈{17,19,21},
routing TIMER 21 → a card-update call) and by experiment: posting `[21,…]` at cadence unblocked the
loop (159 per-frame iterations, `paint()` ran each frame). So the per-frame driver **is**
implementable in wie (LGT-AOT-gated `TIMER_EVENT` cadence) — it is *not* proprietary.

Measured consequences (배틀몬스터, the one title reaching this wall):
- The app sets its displayable via native `import 0x21`, never `Display.pushCard`, so the MSP
  `CardCanvas` card-vector stays empty and `CardCanvas.paint` draws nothing (cp48).
- The draw gate `o.g` is set only by the card's **update** method (`i.b`), which the MSP `paint`
  contract never calls; the app's update/paint methods each run **once** at boot, never per-frame.
- Driving the update method per-frame externally (cp49 probe, reverted) → `Game.b` returns 0
  idempotently, **0 draws**; forcing the gate + paint (cp28) → background `fillRect`/`setColor`
  only, **no sprites** — sprite load is further gated by the scene-state machine
  (`field[0x74]`, see `docs/FOLLOWUP_ISSUE.md`).

A coordinated reconstruction (cp50) wired the **paint** pipeline: routing the native `0x21(_,Card,
Jlet)` registration to `Display.pushCard(card)` and driving `Display.handlePaintEvent` per frame
runs `CardCanvas.paint → card.paint` every frame. But three walls still block any draw:
- **(A) multi-subclass binding:** the card binds to its **base** Card subclass (`o`), not its true
  type (e.g. `i`), because wie only sees the platform `Card.<init>` (app `<init>`s are bypassed) and
  the `new` primitive carries no class handle — so the subclass update method that sets the draw gate
  is unreachable (`NoSuchMethodError`).
- **(B) JVM-field vs guest-field:** AOT field reads hit guest memory, so the gate must be set by the
  real ARM update method, not a JVM-side `put_field`.
- **(C) scene-state:** even with the gate open, only background `fillRect`/`setColor` draws; sprites
  need the scene-state machine (`field[0x74]`) to advance (see `docs/FOLLOWUP_ISSUE.md`).

So the missing pieces are app-internal obfuscated RE (per-`new`-site true subclass; the subclass
update method/args) plus the ez-i runtime's scene-state inputs — recoverable only from the ez-i
native runtime / LGE Xceed VM / a device execution trace. Full RE trail: `docs/lgt_abi.md` §7 and
checkpoints cp37–cp50.
