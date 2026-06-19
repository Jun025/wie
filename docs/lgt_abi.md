# LGT / ez-i Java-app ABI

Reverse-engineered ABI for running an **AOT-compiled LGT Java app** (ez-i / Xceed
toolchain) on wie's JVM. Each Java class is emitted as a native ARM record: methods
are real ARM code in `.text`, class/field/method metadata lives in `.data`. The app
talks to the platform through the "java-interface" import module (table `0x64`).

Scope: the PoC lives entirely in `wie_lgt` (`LgtJvmShared`, LGT-specific per #1232);
shared `wie_midp` / `wie_wipi_java` classes are **not** modified. This document is the
consolidated ABI; see `docs/lgt_native_classes.md` for the byte-level descriptor RE.
The `cpNN` tags below mark the reverse-engineering milestone each fact was pinned at.

Notation: addresses are from the ez-i reference app (`.text` `0x1000..0xe7800`,
`.data` `0x1400000..`, `.bss` `0x1500000..`). Anything not pinned by RE is labelled
**(추정)**.

---

## 1. Module / SVC layout

The native code reaches the platform two ways: **import thunks** (16-byte stubs that
trap into wie via SVC) and **trampolines** (per-method SVC stubs wie installs into the
vtables). wie routes everything through SVC categories (`wie_lgt/src/runtime.rs`):

| category | id | handler | role |
|---|---|---|---|
| `SVC_CATEGORY_INIT` | 1 | `init.rs` | boot imports (`java_unk0/5/9/11/12`, `load_classes`, `new`, `0x54`) |
| `SVC_CATEGORY_WIPIC` | 3 | `wipi_c.rs` | WIPI-C clet ABI (framebuffer/graphics) — used by the clet path |
| `SVC_CATEGORY_STDLIB` | 5 | `stdlib.rs` | libc-ish (`strcpy`, `memcpy`, `time`, …) + native `new` (`0x32`) |
| `SVC_CATEGORY_JAVA_TRAMPOLINE` | 7 | `native_jvm.rs` | per-method vtable/static stubs → JVM invoke |
| `SVC_CATEGORY_JAVA_INTERFACE` | 9 | `java/interface.rs` | java-interface imports routed by index (the SVC id **is** the import index) |

### Import thunk format (16 bytes, in `.text`)

```
str  lr, [sp, #-4]!     ; save return
bl   <dispatcher>       ; 0xe31a8 (resolves index, traps to platform)
.word <table>           ; table id (1 = stdlib, 0x64 = java-interface)
.word <index>           ; function index within the table
```

The dispatcher (`0xe31a8`) reads the index word that follows the `bl` and returns to
the caller via the saved `ip` after the platform call. wie resolves a thunk on first
use via `get_import_table(0x64)` / `get_import_function(0x64, <index>)`
(`init.rs`), which hands back the SVC stub for that index. Example: the thunk at
`0xe2c50` has `.word 1, .word 0x32` ⇒ the stdlib **`new`** primitive — so an app call
that `bx`-es to `0xe2c50` is allocating an object.

---

## 2. Native class descriptor format

(Full RE in `docs/lgt_native_classes.md`; mirrored by `native_class.rs`. 283/283
method code pointers validated inside `.text`.)

```
Class header (at H):
  +0x00  tag           (0x21 / 0x31 observed)
  +0x08  ptr_name      -> cstring (obfuscated single letters: "Game", "a", "o", "i", …)
  +0x10  ptr_parent    -> platform class-name cstring, OR another class header H'
  +0x18  access_flags  (java access bits | 0x20000 app marker)
  +0x38  ptr_methods   -> [count:u32, MethodRecord[count]]
  +0x3c  ptr_fields    -> [count:u32, FieldRecord[count]]

Class handle (at H + 0x4c): { 0, 0, H }    <- every member's ptr_class points here,
                                              i.e. handle indirection: record -> handle -> header

MethodRecord (28 bytes):
  +0x00 ptr_class(=handle)  +0x04 ptr_name   +0x08 ptr_signature
  +0x0c access_flags        +0x10 num_locals +0x14 code_ptr(-> .text)   +0x18 unk

FieldRecord (20 bytes):
  +0x00 ptr_class(=handle)  +0x04 ptr_name   +0x08 ptr_type
  +0x0c access_flags        +0x10 index      (declared index within the class)
```

**Class handle indirection**: members carry `ptr_class = H + 0x4c` (the handle), not
`H`. The handle's `+0x08` points back to `H`. `getInstance` and singleton lookups take
a handle and resolve `handle → header → name` (`parse_native_class_from_handle`).

### App class graph (reference app, from `.data` scan)

```
Jlet (platform)
  └─ a            (a.run @0x1f10, a.startApp @0x1ad8, a.b(Lo;I)V @0x2200)
       └─ Game    (Game.a @0x11dc data-load, Game.b @0x1484)
org/kwis/msp/lcdui/Card (platform)
  └─ o
       ├─ d, e, j, l
       └─ b └─ i
```
Class `a`'s descriptor handle = `0x1400df4` (header `0x1400da8`).

---

## 3. Boot sequence

`get_java_interface_method` (`interface.rs`) maps the boot imports; the app drives them
in order during startup:

| import | wie fn | meaning |
|---|---|---|
| `0x03` | `java_unk0` | register main-class metadata `(name="Game", _, flag="true")` |
| `0x07` | `java_unk5` | register the app's OWN native classes (handle array @ a0) |
| `0x06` | `java_unk12` | paired with `0x07` (same struct ptr) — role unconfirmed (추정) |
| `0x14` | `java_load_classes` | declare IMPORTED platform classes + resolve dispatch offsets (see §4) |
| `0x82` | `java_unk9` | boot hook, arg always 0 (추정: lifecycle marker) |
| `0x83` | `java_unk11` | invoke-static `org/kwis/msp/lcdui/Main.main(argv)`, `argv[0]="Game"` |
| `0x0f` | `JavaNewObject` | native object allocator (`obj = 0xf(...); obj.<init>()`) |
| `0x54` | `java_interface_unk84` | per-method entry helper (stack/safepoint check) — no-op (추정) |

`0x83` boots the app's Jlet through the **shared lcdui Main path**
(`invoke_lcdui_main(jvm, "Game")`), identical to the WIPI-C clet boot
(`net/wie/CletWrapper`). That wraps `Game` in `WIPIMIDlet`, creates `net.wie.CardCanvas`,
and `Display.setCurrent(CardCanvas)` — so wie's MIDP paint loop begins ticking (empty)
from here.

---

## 4. Global virtual dispatch (two-level vtable, reserved slot 0)

`java_load_classes` (`0x14`) hands the platform parallel arrays describing imported
classes and the method/field refs they use. `install_platform_tables` builds the
dispatch tables the AOT code reads.

**AOT virtual call shape:**
```
idx = virtual_method_offsets[ref]      ; u16 logical index
bx   vtable[idx + 1]                   ; ldr ip,[r3, #4] after add r3,r3,idx<<2
```
The `+1` (`ldr ip,[r3,#4]`) means **physical slot 0 is reserved** and methods start at
slot 1. wie therefore:

- allocates `global_vtable` of `VTABLE_REFS+1 = 129` words (`VTABLE_REFS = 128`),
- for each method-ref `r` with a real `(...)`-signature: installs a trampoline at
  **physical slot `r+1`** and sets `virtual_method_offsets[r] = r` (the logical index),
- so `vtable[offset[r] + 1] = vtable[r + 1]` dispatches ref `r` correctly.

Verified by RE: pre-fix, `AnnunciatorComponent.show@ref6` misrouted to `vtable[7]` =
`File.read`; the reserved-slot-0 fix corrected it (cp15). Every object's `+0x00` word
points at `global_vtable` (app objects and platform proxies alike).

### Per-class override vtables

A few `java/lang` classes are dispatched by the AOT at **hardcoded physical slots**
that collide with the global identity table. For those, wie copies the global vtable
and overrides the known slots (`known_java_lang_vtable`), storing the result in
`class_vtables[name]`; `bind_pending` repoints the object's `+0x00` to it once the
class is known. Indices are **physical** (reserved slot already baked in):

| class | physical slot → method |
|---|---|
| `java/lang/Runtime` | 13 → `freeMemory()J`, 14 → `gc()V` |
| `java/lang/StringBuffer` | 5 → `toString()`, 19 → `append(Ljava/lang/String;)Ljava/lang/StringBuffer;` |
| `java/lang/Thread` | 11 → `start()V` |

(These slots are empirically identified — **추정** where not cross-checked against a
second call site. Runtime/StringBuffer/Thread are all confirmed by a
working call.)

### Static methods / fields

Per imported class, static methods get **direct** trampoline pointers written into
`static_method_offsets[smo+j]`; static-field slots get an identity fill into
`field_offsets[sfo+j]`.

---

## 5. Instance object model + field layout

```
guest object block (alloc_native_object / native `new`):
  +0x00  vtable word      -> global_vtable (or a per-class override after <init>)
  +0x04  0
  +0x08  ptr_fields       -> field array (256 words, zeroed)
```

`OBJ_HEADER_SIZE = 0x0c`, `OBJ_PTR_FIELDS_OFFSET = 0x08`, `FIELD_ARRAY_WORDS = 256`.

**Instance field addressing** (AOT): `obj.field[ field_offsets[K] ]`. Left all-zero,
every field-ref aliases slot 0 (this broke `a.startApp`'s "is Display set?" gate —
cp16/cp17). wie computes an **inheritance-aware, inherited-first flat layout**:

> object slot = (count of fields in all app-ancestor classes) + declared field index

Platform ancestors contribute 0 fields to the guest layout (their state lives on the
JVM side). `register_app_classes` computes each class's `(name, type, slot)` list into
`app_field_layouts`; `install_platform_tables` then segments the flat `fields` ref
array by matching each window to a class's exact field set and writes the resolved slot
into `field_offsets[k]`. Field-record parse offsets: name `+0x04`, type `+0x08`,
declared index `+0x10` (all 150 reference-app fields matched after fixing an initial
`+0x00` vs `+0x04` off-by-4).

### Object binding lifecycle

- `new` (stdlib `0x32` / java `0xf`) → `alloc_native_object`: allocates the block with
  the **global** vtable word, inserts the ptr into `pending_new` (**unbound** — no JVM
  class yet).
- The `<init>` trampoline calls `bind_pending(ptr, class_name)`: removes it from
  `pending_new`, repoints `+0x00` to a per-class vtable if one exists, resolves the
  class, and creates the instance. App classes reuse the guest block as an
  `LgtClassInstance`; platform classes are instantiated by the JVM keyed on the ptr.
- `getInstance` (java-interface `0xc`) → `singleton_instance(class_handle)`: cached;
  resolves handle → name, instantiates a bound app instance, returns its guest ptr.
  Must be stable across calls/threads (per-class state like `a.run`'s run-flag at
  `obj+0x20` is shared through it — cp20).

> Roadmap — field unification: an `LgtClassInstance`'s JVM-side fields and the guest
> field array at `guest_ptr` are currently separate stores. They should be unified so a
> field written by ARM code and the same field read via the JVM agree, by mapping each
> JVM field through the `field_offsets` slot map onto the guest array. Not yet needed:
> for the current reach (boot + setup) no field is written on one side and read on the
> other.

---

## 6. java-interface import table (status)

The SVC id **is** the import index. Implemented imports get a real handler; everything
else is logged and returns 0. Three tiers:

### Implemented

| idx | name | semantics |
|---|---|---|
| `0x9` | String factory | `(ctx, utf16_ptr, count, out_slot)` → materialise `java/lang/String` from constant-pool UTF-16, return a guest ptr bound to it (cp10) |
| `0xc` | `getInstance(handle)` | canonical singleton instance for a class handle (cp20) |
| `0xf` | native `new` | allocate guest object; `<init>` binds it (cp8) |
| `0x54` | method-entry helper | resolved first in every native method with a small constant — stack/safepoint check (추정), no-op |
| boot: `0x03 0x06 0x07 0x14 0x82 0x83` | see §3 | |

### No-op-safe (evidence-identified primitives; documented no-ops)

These are called with shapes consistent with GC / safepoint / exception / sync
bookkeeping; leaving them as `→ 0` has caused no regression across full boot + setup.

| idx | observed args | likely role (추정) |
|---|---|---|
| `0x12` | `(0, 0, sp)` | stack-ptr only — safepoint/exception frame |
| `0x1f` | `(0, obj/type, count)` | type/count — GC root or array bookkeeping |
| `0x21` | `(obj, 0, sp)` **and** `(new_obj, …)` | register/track object (see §7 — driver path uses this) |
| `0x22` | `(0, n, …)` | small ints — counter/flag |
| `0xb`,`0xd` | `(handle, …)` pair | per-class bookkeeping invoked inside helper `0x1908` (36× each) |

> Note: `0x21` is listed here because its *common* uses are benign object tracking, but
> it is **also** on the render-driver registration path (§7) where a no-op is *not*
> sufficient. The distinction is the call site, not the index.

### Unresolved — render driver (needs ez-i ABI; see §7)

| idx | observed | blocker |
|---|---|---|
| `0x55` | `(a-singleton, code@0x1ad4, 0)` / `(0,4,8)` (overloaded) | registers app "carried code"; replaying it is inert (cp23) |
| `0x56` | `(this, code@0x1ad4, 0)` | registers app callback/code |
| `0x57` | `(this, …)` in `a.b(Lo;I)V` | called once at setup; a0 = app `this`, not a card (cp25) |
| `0x21` (driver use) | `(new_obj, …)` in `a.run` and `a.b` | hands an **unbound** native object to the platform |

---

## 7. Render model + the one open question

### What the WIPI render model says (public)

- `DisplayProxy.flush()` / `flush(x,y,w,h)` blits a double-buffered back-buffer to
  screen; `isDoubleBuffered()` exists.
- `org.kwis.msp.lcdui.EventDispatcher` (`DisplayProxy.evtDisp`) drives the tick.
- WIPI-C clet: the platform calls `paintClet(x,y,w,h)` per frame. Java equivalent =
  the displayable's per-frame paint + flush.
- wie already drives WIPI-C clets; `CardCanvas.paint` already loops every frame.

### What the reference app actually does (RE, cp21–cp25)

- The app does **not** use `pushCard`/`CardCanvas` normally. It runs full setup
  (data load → 240×320 back-buffer → `getGraphics` → Cards/RNG/Thread) and then **0
  draw calls** are issued.
- `a.run` (the game thread's `run`) is a **confirmed one-shot**: `getInstance(0xe)` →
  helper → check `obj.field[8]` → `0x55(obj)`, `0x56(this)`, `0x1f(0)`, then **returns**
  via the epilogue at `0x2140`. No frame loop inside it.
- `a.b(Lo;I)V` runs **once** at setup: `new(0x32)` → bare object `r4`; `0x57(a0=this)`;
  `0x21(a0=r4)`. The "helper" `0xe2c50` is itself the `new` import thunk.
- The object handed to `0x21` (the render driver / "card") comes straight from `new`:
  it has only the **global vtable word**, is in `pending_new`, and has **no `<init>`**
  before registration ⇒ **no JVM class, no descriptor, no per-class vtable**.

### Why this is blocked

P4 (static-type identification) has nothing to latch onto: the registered object is an
opaque ez-i-native handle with **no bound per-frame method to call**. The per-frame
invocation is the **ez-i runtime's** job — its event loop calls the registered native
object's paint entry each frame — and wie does not emulate that loop (replaying the
carried code `0x55/0x56` is inert, cp23). The app registers nothing into wie's MIDP
card system, so there is no safe wie-side connection point. Calling any native vtable
slot here would be a guess (forbidden).

**cp26 — the app `Card.paint` path is reachable but empty (experiment, reverted).**
The app's `Card` subclasses *are* known JVM classes: `o` (extends
`org/kwis/msp/lcdui/Card`) has a real `paint(Lorg/kwis/msp/lcdui/Graphics;)V`
(`@0xd8d70`, draws on its Graphics arg), inherited by `d`/`e`/`j`/`l`, and each is
created as a bound singleton via `getInstance` (`0xc`). A one-shot experiment pushed the
bound `o`-card into wie's `Display`/`CardCanvas`: **`o.paint` then ticked every frame on
the back-buffer** (the Card.paint path wires cleanly into wie's existing tick) — but
issued **zero draw calls**. `o.paint` ran without error and made no `Graphics`/draw
calls at all, i.e. it took its empty-state early-out. Root cause is the same wall: the
`getInstance` singletons are **empty shells** (zeroed fields); the live title-screen
state lives in the ez-i-native (unbound) objects, not in the JVM-bound cards. So
correctly ticking `Card.paint` still paints nothing. This **excludes the app
`Card.paint` instance path** and re-points at the same missing piece below.

### The single missing answer (for the maintainer)

> In ez-i, when an app `new`s a bare native object and hands it to platform import
> `0x21` (and registers app callbacks via `0x55`/`0x56`), **which registered object's
> which native entry point does the ez-i runtime invoke each frame to paint**, and how
> does its back-buffer reach the screen (the ez-i equivalent of `DisplayProxy.flush`)?
> Equivalently: what is the ez-i native displayable/clet ABI that `0x21`/`0x55`/`0x56`
> bind — so wie can call that per-frame entry from its existing paint tick?

With that, the connection is: on each wie paint tick, invoke the registered object's
per-frame entry with a Graphics, then map its flush to wie's `present`/screen blit —
all from the `wie_lgt` / `LgtJvmShared` side, without touching shared classes.

---

## 8. Current reach

| stage | state |
|---|---|
| app classes registered, methods run as real ARM | ✅ |
| boot (`0x64` java-interface → `Main.main` → Jlet) | ✅ |
| two-level vtable + per-class overrides + instance field layout | ✅ |
| `getInstance` singletons, `Thread.start`, game thread spawns `a.run` | ✅ |
| data load → 240×320 back-buffer → `getGraphics` → Cards/RNG | ✅ |
| app `Card.paint` ticked in wie's loop (cp26 experiment) | ◑ wires in & runs per-frame, but the bound card is an empty shell → **0 draws** (§7) |
| **per-frame render driver** | ⛔ blocked on ez-i render-tick ABI (§7) — **0 draw calls** |
| clet regression (`test_helloworld`) / `clippy -p wie_lgt` | ✅ clean |
