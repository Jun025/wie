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
- **JVM**: Clets run on `RustJavaJvmImplementation` (pure Rust JVM). **AOT-Java apps** are loaded
  into that same JVM by `net/wie/LgtClassLoader` (`runtime/java/classes/net/wie/lgt_class_loader.rs`),
  a real `java.lang.ClassLoader` subclass: it reads the class/method metadata out of `binary.mod`
  `.data` and hands back ordinary JVM classes whose methods dispatch to ARM bodies via
  `core.run_function` (`runtime/java/jvm_support/`).
  *(★Corrected 2026-09-21. This bullet described `runtime/java/native_jvm.rs` and an
  `alloc_native_object`/`bind_pending` guest-object binding; that file does not exist in this tree —
  upstream `cc652b1d` replaced the bridge. The difference is not cosmetic: the old model is what the
  §7 section below reasons from, which is why that section now carries a superseded banner.)*

## The §7 wall: AOT-Java per-frame render driver (SUPERSEDED — see the banner)

> **★SUPERSEDED 2026-09-21 by upstream `cc652b1d` (2026-08-04, "Implement LGT Java AOT runtime").
> Everything below is the fork-engine record as of 2026-07; it was true then and its *conclusion* is
> not true now.** Read it for the reverse-engineering trail, not for what wie does today.
>
> **What changed.** This section's model is that the app self-dispatches off a `TIMER_EVENT(21)`
> that wie never posts, so nothing ever paints. That model describes an engine that no longer
> exists: `native_jvm.rs` was replaced by `runtime/java/jvm_support/` + `net/wie/LgtClassLoader`,
> which loads the AOT app's classes into the real RustJava JVM as ordinary JVM classes. The string
> `TIMER_EVENT` now appears **0 times** in the tree.
>
> **What is measured today** (2026-09-21, `--inject`, release `wie_validate` at `394fde8b`; full
> table in `docs/report/0210`). Three of the corpus's 18 unique AOT-Java titles render:
> `메이플스토리2007` (up to 121 paints / 512 distinct colours — title screen *with sprites*, then an
> in-game intro scene), `현영맞고2006` (154 paints / 512 colours), `놈3` (69 paints / 93 colours).
> A `RUST_LOG=debug` trace of `메이플스토리2007` shows the loop the section says is blocked actually
> running: `net.wie.EventQueue::getNextEvent` **73×**, `dispatchEvent` **73×**,
> `net.wie.CardCanvas::paint` **21×**, `Graphics::drawImage` **193×**, `Image::createImage` **44×** —
> driven by wie's `RepaintEvent(41)`, not by a TIMER 21 the app self-dispatches. The trace also shows
> `org.kwis.msp.lcdui.Display::pushCard` **1×**, which directly falsifies the cp48 bullet below
> ("never `Display.pushCard`, so the card-vector stays empty").
>
> **What is NOT claimed.** The other 15 unique titles still render nothing — but they die at *boot*
> (ticks 0–3, `NoClassDefFoundError` / `Invalid memory access`), which is **upstream of** this wall,
> exactly as cp43 said. Nothing reaches §7 to test it, so "§7 is fixed" is not measured and is not
> asserted here. **배틀몬스터 in particular is no longer "the one title reaching this wall"** — it
> stops at boot tick 2 on `net.wie.WieError: Invalid memory access; address: 0`, so whether §7 would
> still block it is **unknown**. The cp-numbered record below is kept verbatim; none of it is deleted.

KTF AOT-Java titles render because the **app** spawns its own game-loop thread (`Thread.run`)
that does logic + `repaint()` each frame; wie drives it via the cooperative scheduler. LGT ez-i
apps drive per-frame work off the **`org.kwis.msp.lcdui.EventQueue`**: the platform posts a
`TIMER_EVENT` (type **21**) at frame cadence, and the app's own `getNextEvent` loop reads
`event[0]` and self-dispatches (`dispatchEvent` is a stub in ez-i — see
`docs/reference/ezi_dispatch_reference.md`). **wie never posts `TIMER_EVENT(21)`**, so the app's
`getNextEvent` loop blocks forever and never ticks (cp55). *(Earlier cp42/52 wrongly modelled this
as "the runtime dispatches a method on a registered bare handle"; cp55 corrects it.)*

cp55 confirmed this in-binary (BattleMonster's dispatcher @0x831xx switches `event[0]`∈{17,19,21},
routing TIMER 21 → a card-update call) and by experiment: posting `[21,…]` at cadence unblocked the
loop (159 per-frame iterations, `paint()` ran each frame). So the per-frame driver **is**
implementable in wie (LGT-AOT-gated `TIMER_EVENT` cadence) — it is *not* proprietary.

Measured consequences (배틀몬스터, the one title reaching this wall — ★as of 2026-07; it now stops
at boot tick 2, see the banner):
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
