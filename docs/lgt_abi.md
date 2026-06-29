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
| `java/lang/String` | 35 → `toCharArray()[C` (cp30) |

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
> other. (cp27 confirmed this for the render path: `o.paint` reads, and `o.k` writes,
> the *same* guest field-array slot — both ARM-side — so the title-render wall is **not**
> a field-store split. See §7.)

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

**cp27 — render-field provenance, traced (diagnosis only).** cp26's "empty shell" was
an inference from `g==0`; cp27 traced the actual writer of the one field that gates
`o.paint`, to classify the wall as app-side (fixable) vs platform-side (maintainer).

- *Gate field S.* `o.paint` (`@0xd8d70`) early-outs on `bl 0xd8640` → `getInstance(o)`
  (`0xd85e4`, import idx `0xc`, confirmed) returning the **o class singleton**; the gate
  is that singleton's guest field-array slot 6 (`[obj+8][+0x18]`) = **`o.g : I`** (int,
  declared index 6; `o` extends the platform `Card`, so slot = declared index). The gate
  is the *class singleton*, not `this`.
- *Writers of `o.g`.* Found in the app ARM — both inside one method, **`o.k()V`
  (`@0xda7f8`)**: `0xda8a0` writes `g ← 0` (reset), `0xdb240` writes `g ← 1` (set, from a
  literal). Both reach the singleton via `bl 0xd8640` (`getInstance(o)`), i.e. they write
  **the same singleton and the same guest field-array store that `o.paint` reads**. No
  JVM-side writer exists (the AOT writes its own fields with ARM `str`, not a JVM
  putfield). Data source is a literal `0/1` (a game state flag — not a `.dat` load, not a
  native-import return).
- *Instance identity.* gate object = writer target = the instance cp26 pushed = the
  `getInstance(o)` singleton (`0x48840130`) — all the same. The unbound native objects
  carried by `0x21`/`0x57` are separate, but `o.paint` never reads them (it gates on the
  class singleton).
- *Why `o.g` stays 0.* `o.k()V` (the writer) is a **virtual method with no static call
  site** (no `bl 0xda7f8`, no pointer-word ref anywhere in `.text`) — it is only reachable
  via the two-level vtable dispatch, i.e. called by the game's state machine each
  frame/step. In the reachable run only `o.paint` is dispatched on `o`; `o.k` never runs,
  so `g` is never set.

**Classification — platform-side (§7), now traced, not inferred.** The wall is *not* a
field-store split (§5: writer and reader use the same guest array) and *not* an
instance mis-pick (cp26 pushed the very singleton `o.paint` gates on). The `o.g` writer
exists and targets the right store/instance; it just never runs, because the virtual
`o.k()V` that holds it is driven by the game state machine, which only advances under a
per-frame tick. That tick is the ez-i runtime's job — the same missing piece below.
So **field unification (§5) and "find the live instance" are *not* fixes** for the title
render; the one missing thing remains the ez-i per-frame drive.

**cp28 — force `o.g = 1`: the render path works end-to-end (experiment, reverted).**
A one-shot surgical probe pushed the `o`-singleton into wie's tick (cp26 wiring) and
wrote `1` into its gate slot (`[obj+8][+0x18]`, the `o.g` cp27 traced). Result:
`o.paint` **passed the early-out and drew on the back-buffer** — 39 real draw calls
(**21× `Graphics.setColor`, 18× `Graphics.fillRect`**) through wie's standard paint
event (`Display.handlePaintEvent → CardCanvas.handlePaintEvent(Graphics) → o.paint`),
which presents the buffer to the screen. This **proves `o.g` is the genuine render gate
and the whole render path (gate → `o.paint` → `Graphics` → back-buffer → flush) works**;
the only thing missing for those draws is *setting `o.g`* (i.e. running the virtual
`o.k()V`, i.e. the ez-i per-frame drive).
Caveat: `o.paint` did not run to completion — after the 39 draws it hit a *separate*
`NoSuchMethodError: java/lang/String.e()V` (diagnosed in cp29 below — **not** a second
empty-state field gate; draws did occur). The draws are background/box fills
(`fillRect`), not yet the title sprite/text (`drawImage`/`drawString`), consistent with
`o.paint` aborting partway. Experiment reverted (force-g=1 is a test hook, not a fix);
only the finding is recorded.

**cp29 — the `String.e()V` abort is an ez-i `java/lang/String` vtable slot (platform,
STOP).** Traced the cp28 caveat under the same harness. The receiver is a *genuine*
`java/lang/String` (`"LOADING..."`, just made by the String factory): a draw-text
wrapper `B(Graphics, String)` (`@0x100d8`) does `setColor` → `s.vtable[slot 35]()` →
draw. So the String is the intended argument, **not** a mis-bound app object. The crash
is that the app calls `java/lang/String`'s hardcoded **physical vtable slot 35**
(ref 34), but the app's import data declares `java/lang/String` with **`vmc=0`** (zero
imported virtual methods) — so wie has no per-class String vtable and the global slot 35
holds an unrelated *app* method (`e()V`), giving `String.e()` → `NoSuchMethodError`.
The correct slot-35 method is an entry of **ez-i's own `java/lang/String` vtable**, which
is **not present in the app binary** (`vmc=0`; ez-i provides it). This is the same shape
as the `java/lang/*` per-class vtables (Runtime/StringBuffer/Thread, cp4–6/cp10).
cp29 initially over-classified this as maintainer-gated from a single call site; cp30
corrected and resolved it by RE.

**cp30 — String physical slot 35 = `toCharArray()[C`, RE-confirmed (fix).** Fully
disassembling the draw-text wrapper `B(Graphics, String)` (`@0x100d8`) and its draw
helper (`@0x10228`): `B` calls `s.vtable[35]()` (no args) and the helper then iterates
the result `r` as a char array — `data = [r+8]; len = [data]; for i in 0..len { char =
[data + 4 + i*2] }` (a per-char glyph loop, bitmap font). The only no-arg, char-array-
returning String method is **`toCharArray()[C`**. Added as a per-class String override
(physical slot 35) in `known_java_lang_vtable`. Behaviour-confirmed under the force-g=1
harness: `String.e()V NoSuchMethodError` is **gone**, `String.toCharArray()[C` is now
dispatched at that call site, and `o.paint` runs **without fatal** (cp28 aborted there).
So the platform-side mis-classification was wrong; this was an ordinary `java/lang/*`
vtable slot, RE'd like the others.

**cp31 — char-array guest marshalling (fix), but the glyph loop is gated elsewhere.**
The glyph loop reads a `char[]` as `data = [arr+8]; len = [data]; char = [data+4+i*2]`
(u16 LE). wie marshalled the `toCharArray` result via `register_platform_object`, whose
proxy has `[ptr+8]=0` (empty). cp31 adds `materialize_char_array` (in
`handle_java_trampoline`'s result path, scoped to `[C`): it allocates a guest data
block `{u32 len, u16 chars[len]}` and points the object's `+0x08` at it — the exact
layout RE'd above. Behaviour-confirmed: the `[C` return is materialised, and the block
is correct (logged `char[] len=10 text="LOADING..."`; unit-tested via
`write_char_array_block`).

*(cp31's note that the glyph loop "never enters" was wrong — an `lr` mis-calc; cp32
shows it runs.)* *cp32 target (one line):* find why the glyph loop, once entered,
draws no glyphs.

**cp32 — char-array marshalling confirmed working; glyph render is the real gap.**
Re-traced with corrected `lr`s. The colour "gate" at `0x10298` is **not** a gate:
`g.vtable[ref21]()` = `Graphics.getColor()I` (id 21); both branches (`== 0x00ffffff`
and the `setColor(~c)` else-path) **fall through to `0x102c0`**, so the glyph loop is
always entered. And it **runs**: the loop body's `import 0x55` at `0x102d0` fires
`lr=0x102dc` **30×** (3 frames × 10 chars), reading `data=0x49048000`, `len=10`, chars
`0x4c='L'`… — i.e. `materialize_char_array` works end-to-end and the loop consumes
"LOADING...". (`register_platform_object` readback confirmed `[obj+8]=data`, `[data]=10`.)
What's missing is **glyph drawing**, traced in cp33 below.

**cp33 — the glyph-draw fn runs but has no font image (platform/font gap).** The
glyph-draw fn `@0x109b4` *is* called per char (each char's `0xb(a1=char, a2=x)` lookup
then the fn's body fire). It branches on its first arg `r6` (the font image): at
`0x10b1c cmp r6,#0; 0x10b2c bne 0x10b44`, `r6 != 0` → `r6.vtable[r2]()` (the blit /
drawImage), `r6 == 0` → `import 0x22` (a no-op fallback). Measured: every char takes the
**`r6 == 0`** path — `import 0x22` fires at `lr=0x10b40` once per char — so **no
drawImage is ever issued**. (The earlier `getColor`/clip branches are *not* the skip;
they all reach the glyph fn.) Root cause **(C): the font glyph image is absent** (`r6 ==
0`) — the bitmap-font sheet the AOT expects to blit each glyph from is not present
guest-side, so the draw falls back to a no-op. char data is correct ("LOADING..." is
read); the glyphs just have no font to draw from.

This is a font/image-marshalling task (its own checkpoint), not a one-liner.

**cp34 — glyph blit mechanism RE'd; contract not yet complete → cp35 plan.** The
glyph-draw fn `@0x109b4` has two paths, gated on its font-image arg `r6`:
- **`r6 != 0`** → `g.drawImage(font_sheet, …)` with **`src_x = (char - 0x21) * 10`,
  `width = 10`** (`0x10ac0`: `r3 = char - 0x21; r4 = r3 * 10`) — i.e. a fixed-pitch
  10px ASCII glyph grid blitted from a font sheet (`char 0x21='!'` is grid origin).
- **`r6 == 0`** (measured) → `import 0x22(a0=0, a1=0x11264, a2=x)`. `a1=0x11264` is a
  **.text thunk → `0x10fb0`** (an app native fn that itself calls `getInstance`/imports),
  i.e. `0x22` here is a **carried-code/callback shape** (a1 = a function pointer), and
  wie no-ops it → the native font render never runs. x advances +6 per char (14/20/26…).

So the per-char render is *not* a one-liner: with no font image (`r6==0`), the app falls
to a native font path (`0x22` → fn `0x10fb0`) that wie doesn't run. **Two unknowns block
a confident impl** (so no code this checkpoint, per the "no half-guess" rule):
1. *Why `r6==0`.* Statically `r6` traces to the `Graphics` arg (≠0), but it's measured
   as 0 at the blit — needs a guest-register read (the static trace can't resolve a
   mid-fn reassignment). Determines path (i) JVM-image vs (ii) native.
2. *What `0x22`/`0x10fb0` actually do* — is `0x22` "run carried code a1" (cp23 showed
   replaying carried code can be inert), or a blit primitive? And does `0x10fb0` render
   a glyph to the back-buffer, and from what font data?

*cp35 plan:* (a) guest-register probe at the `0x22` site to fix `r6`/the font image; (b)
RE `0x10fb0`; (c) implement the confirmed path. (Resolved in cp35 below.)

**cp35 — both unknowns resolved; the font path is platform-gated (§7), STOP.** A
one-shot `dump_reg_stack` probe at the per-char `import 0x22` (filtered `a1=0x11264,
a2=14`) settled both:
- **Unknown 1 (corrected cp33/cp34):** `r6` is **not** the font image — `R6 = 0x48840550
  = g (the back-buffer Graphics, ≠0)`. The font image is `import 0x22`'s **a0**
  (`R0 = [singleton.field5] = 0`), measured **0** every char (with `R7=0`, `SB=x`,
  `SL=char`, e.g. `0x4c='L'`). So the **font sheet image is absent/null**, and the glyph
  draw falls back to the native `0x22` path.
- **Unknown 2:** `0x10fb0` (the `0x22` a1 fn) is **`strb` into an object's field array**
  (`[r2+4..+0xb]`, 8 bytes) with **no `Graphics`/drawImage vtable call** — i.e.
  bookkeeping, not an on-screen blit. So the native path renders nothing (cp23-style
  inert), even if run.
- **No font load happens.** A full `debug` run shows `Image.createImage` called **once**
  — the 240×320 **back-buffer** — and **no** `createImage`/`getResource` for a font
  sheet anywhere. So path (i) JVM-image has no load site in the reachable run, and path
  (ii) native is inert.

**Classification: platform-side (§7).** Either the font sheet would be loaded by a later
init step the game never reaches (a.run is one-shot — same per-frame-drive gap as the
render driver), or the font is an ez-i-native resource the `0x22`/`0x10fb0` runtime draws
(which wie doesn't emulate). Both are the §7 missing piece, not an app-side one-liner;
forcing a font would be a guess. *cp36 (one line):* once the per-frame drive (§7) runs,
re-check whether the game then loads the font sheet via `createImage` (→ path i becomes
implementable, `g.drawImage(sheet, src_x=(char-0x21)*10, w=10)`); until then the title
text is blocked on the same §7 gate as the rest of the live render state.

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

cp28 narrows what this drive must accomplish: wie's paint→`Graphics`→back-buffer→flush
path already works (forcing one gate flag made `o.paint` draw to screen). The missing
piece is purely **advancing the game state machine each frame** so the per-card render
flags like `o.g` get set.

**cp36 — driving the state methods from wie does NOT substitute for the ez-i tick
(confirmed).** Tested whether wie can just call the state-advance methods each frame
instead of the ez-i runtime:
- *Correction to cp27:* the `o.g` writer is **not** the registered virtual `o.k()V`.
  `o.k @0xda7f8` is a short method (returns at `0xda85c`, no `+0x18`/`o.g` store) — it
  copies a singleton field. The real `o.g` writer is an **unregistered helper
  `@0xda870`** (not in any class's method table): `getInstance(o)` → reset `o.g=0` (and
  siblings) → conditionally `o.g=1` at `0xdb240`.
- *Experiment (reverted):* drove both, 3× each, reading `o.g` after each call. `o.k()`
  (JVM `invoke_virtual`) → `o.g` stays `0`. `fn@0xda870` (native `run_function`) → `o.g`
  stays `0` too: the helper runs and resets `o.g=0`, but its conditional `o.g=1` branch
  (`0xdb240`) is **not taken** — that branch depends on accumulated game state
  (load-complete / timer / input / card-transition), not satisfiable by calling the
  method in isolation.
- *Conclusion:* **a single JVM/native method drive cannot advance the state** — `o.g=1`
  needs the whole game loop's accumulated conditions, which is exactly the ez-i per-frame
  drive. So substituting wie method-calls for the ez-i tick is ruled out; the §7 entry
  (the real per-frame driver the ez-i runtime invokes) is the single remaining gate, the
  same one that gates the live render state, `o.g`, and the font load. Maintainer path.

**cp37 — the registered "carried code" is one-time INIT, not the per-frame step;
sustained drive of it is inert (experiment, reverted).** Directly attacked the §7 gate by
synthesizing an ez-i per-frame drive in `wie_lgt` (LGT-Java-gated): capture the carried-code
pointer the app registers, then invoke it once per ~frame from a spawned task paced by
`system.sleep` (so wie's clock advances and the MIDP paint/event loop interleaves). New facts,
all RE-confirmed against the app binary (`binary.mod`; 배틀몬스터 **is** the reference app —
`.text 0x1000..0xe7800`, `.data 0x1400000`, matching this doc):

- *AOT method bodies are ARM (A32), not Thumb.* The init entrypoint is entered Thumb
  (`entry+1`), but every class method / carried-code body is 32-bit ARM (`mov ip,sp;
  stmfd sp!,{…,lr}` prologue). `run_function` selects mode by `address & 1`, so the even
  code pointers run as ARM (consistent with methods already working).
- *The registered callback is a single entry `0x1ad4`.* Runtime trace of `a.run`: it calls
  `0x55(obj=0x48840020, a1=0x1ad4, 0)`, `0x56(this=0x48840010, a1=0x1ad4, 0)`,
  `0x57(this, 0x1ad4, 0)`, `0x21(newobj=0x48840550, 0x1ad4, 0)` — **all four carry the same
  code pointer `a1=0x1ad4`**, with distinct `.data` stash slots in `a3` (`0x140467c/8c/9c`).
  `0x1ad4` is `b 0x1a24`.
- *`0x1a24` is straight-line INIT, arg-ignoring and idempotent (full disasm).* It does
  `getInstance(0xa)` (immediately `mov r0,#0xa`, discarding any incoming arg), `bl 0x1908`,
  then unconditionally `str`s constants into the singleton's field array
  (`[+0x24]=0, [+0x34]=1, [+0x38]=1, [+0x3c]=…, [+0x48]=3, [+0x44/0x4c..0x60]=0`) and
  returns. **No conditional branch, no per-frame state read.** Re-running it just re-sets
  the same constants — semantically a one-time "start/init" callback (the ez-i analog of
  the clet `startClet`), **not** a frame step.
- *Experiment result (reverted):* the loop captured `0x1ad4` and drove it every ~33 ms
  with no error — and the screen **stayed black** (`content:false`, 1 paint). So a *sustained*
  drive of the registered carried code is **inert** (extends cp23's one-shot finding), and
  driving init code per-frame is also semantically wrong (it would re-init a progressed
  game). Reverted: the carried code is not the per-frame entry, so capturing/driving it is
  dead weight and a (small) regression risk to the other AOT games.

*Refined classification.* The per-frame entry the ez-i runtime invokes is **not** any
pointer the app hands to `0x55/0x56/0x57` (those are its init/lifecycle callbacks). Per §7
it is the runtime's invocation of a **method on the registered native object** (the unbound
`0x21` object `0x48840550`) through the **ez-i native-displayable vtable ABI** — which lives
in the LGT platform, **not in the app binary**, so its slot/signature cannot be derived from
`binary.mod` and choosing one would be a guess (forbidden). cp37 thus narrows the §7 question
to its sharpest form below and **rules out the most natural non-guess avenue** (drive the
registered callback). Genuinely maintainer/platform-ABI-gated.

*cp38 starting point (one line):* find the per-frame entry by locating who, in a real LGT
device trace, calls into the `0xda870→0xdb240` (`o.g`) writer chain — i.e. RE the LGT/ez-i
platform's native-displayable dispatch (the absent caller), not the app; the app side is
exhausted.

**cp38 — the `o.g=1` store is UNCONDITIONAL; the gate is *which method runs*, and the
o.g-setter is a card method reached only by vtable dispatch (decode + live values, reverted).**
cp36 attributed the stuck `o.g` to an unsatisfied predicate "needing accumulated state."
cp38 decoded the actual control flow and it is **not** a predicate at the store — it is a
*never-dispatched method*. Two corrections + hard evidence:

- *The store `0xdb240` is unconditional.* `str r4,[r3,#0x18]` (`r4=1`, `r3=[o_singleton+8]`
  = the `o.g` slot) sits in a self-contained function whose **prologue is `0xdb200`**; there
  is **no conditional branch between `0xdb200` and `0xdb240`** that can skip it. So `o.g=1`
  **iff `fn@0xdb200` is entered**. `fn@0xdb200` is a private helper ("show/activate card N":
  takes a card-id in `r0`, does `o[0x30]=max(o[0x30],id)`, `o.g=1`, `o[0x70]=0`); it has
  **no method-table entry** — it is called via 4 `.text` literal-pool `ldr;bx` sites.
- *cp36 drove the WRONG function.* `fn@0xda870` (which cp36 drove and saw `o.g` stay 0) is
  the **resetter**: prologue `0xda870`, it does `getInstance(o)` (`bl 0xd8640`) then
  `str 0,[…+0x18]` (`o.g=0`) and **returns at `0xda940`** — it never reaches `0xdb240`. So
  driving it can only ever *clear* `o.g`. cp36's "needs accumulated state" conclusion is
  **withdrawn**.
- *Who calls the setter (the real gate), with exact predicates.* The 2 reachable
  literal-pool call sites of `fn@0xdb200` are inside two **registered card-`i` methods**
  (`i` extends `b` … extends `Card`):
  - `i.b(III)V` (`@0x2d6b4`, rec `0x14020d0`): `cmp r7,#0; bne skip` where `r7` = its **3rd
    int arg `p3`**. So **`o.g=1` iff `i.b(_, _, 0)` is invoked** (then `0xdb200(card=3)`).
  - `i.a()V` (`@0x6fac4`, rec `0x1402c4c`): `cmp [fp-0x30], o[0x74]; bne skip` (a local vs
    the `o` field at `+0x74`); if equal → `0xdb200(card=5)` then `(9)`.
  Both `i.a`/`i.b` have **no direct `bl` anywhere in `.text`** — only their method-table
  entry — so they are reached **only by virtual (vtable) dispatch**, i.e. by the game state
  machine / ez-i event loop, never by a static app call.
- *Live values (temporary diag, reverted).* At the natural boot stop the `o` singleton
  exists (`o@0x48840130`) with **`o.g=0, o[0x30]=0, o[0x74]=0`**; a full debug trace shows
  **only 5 app dispatches in the entire boot** — `Game.<init>/a/b`, `a.startApp`, `a.run` —
  and **zero card methods** (`i/o/b/d/e/j/l`) ever run. Driving `i.b(0,0,0)` via the JVM
  **flips `o.g` to `1` in a single call** (then `i.b` errors later in the same font/`String`
  path as cp28–35 — the `o.g` store already happened); `i.a()` runs clean and `o.g` stays 1.

*Label table (the §2.2 deliverable):*

| input that gates `o.g=1` | what it is | writer / supplier | label |
|---|---|---|---|
| `fn@0xdb200` is entered | "show card N" helper; the unconditional `o.g=1` store | the 2 card-`i` call sites below | — |
| `i.b` arg `p3 == 0` | 3rd int param of `i.b(III)V` | **`i.b`'s caller** (state machine, via vtable) — no static app caller | **PLATFORM** (per-frame/event dispatch) |
| `i.a` local `== o[0x74]` | a value computed in `i.a` vs `o` field `+0x74` | `i.a`'s own body once `i.a` is dispatched; `o[0x74]=0` at boot | **PLATFORM** (`i.a` itself is dispatch-only) |
| `i.a` / `i.b` are invoked | the card update methods themselves | **ez-i runtime vtable dispatch of the current card** (absent in `binary.mod`) | **PLATFORM** |

*Verdict — §7 wall HARD-confirmed, with a sharper shape.* Every gate resolves to the same
thing: the **card-`i` update methods (`i.a`/`i.b`) are never dispatched** because nothing
drives the ez-i per-frame/per-event loop. The `o.g` store is healthy and reachable by a
**single** legitimate method call (so it is **not** the unsatisfiable accumulated-state
predicate cp36 supposed); the lone missing piece is the runtime's natural dispatch of the
current card's update method. Driving `i.a`/`i.b` out-of-band sets `o.g` but is **forcing**
(it jumps to a card irrespective of game logic / args), the same class as force-`g`, so it is
**not** an APP-drivable precondition wie has legitimately completed. **No (b) candidate — pure
(a).** Encouragingly, the gap is now a *normal card-method dispatch* (vtable + a "current
displayable / update entry" protocol), closer to wie's existing `Card.paint` tick than to an
opaque native handle: the open question narrows from "what native entry?" to **"what is the
ez-i protocol for choosing the current displayable and dispatching its per-frame update method
(the analog of `i.a`/`i.b`) — which method/slot, what args (e.g. `i.b`'s `p3`), what cadence
(frame vs key/timer event)?"** That protocol is platform-side; the app side is exhausted.

**cp39 — `a.run` IS the game loop; it exits because the current displayable (`a.field[0x5c]`)
is 0, not because of the run-flag. Premise correction + concrete next gate (decode + live,
reverted).** A follow-up hypothesised the per-frame driver was a `notifyEvent` override fed by
wie's existing `Event::Notify → CardCanvas.handleNotifyEvent → card.notifyEvent` path. **That
premise is false** and a sharper, more concrete blocker was found:

- *No `notifyEvent` override exists.* The string `notifyEvent` is **absent** from `binary.mod`.
  The app keeps the platform-override names it *does* implement — `o.paint`,
  `o.keyNotify(II)Z`, `a.run()V`, `a.startApp([Ljava/lang/String;)V` (method-record scan) — so
  cp38's `i.b` (name literally `b`) is **not** a `notifyEvent` override; pushing `Event::Notify`
  would hit wie's default `Card.notifyEvent` and never reach it. The `i.a`/`i.b` o.g-setters are
  app-internal methods the **game's own loop** dispatches, not platform event callbacks.
- *`a.run` (`@0x1f10`) is the real per-frame loop (full disasm).* Shape:
  `r5 = getInstance(a)` (helper `0x1908→0x18ac`, class handle `0x1400df4` = `a`);
  `while (a.field[0x20] != 0) { …body…; cur = a.field[0x5c]; if (cur == 0) { call [0x14045fc];
  return; } else { cur.vtable[off]()  // 0x2128: per-frame update dispatch } }`. So each
  iteration dispatches the **current displayable's** update method — exactly the
  `i.a`/`i.b`-shaped call cp38 wanted — **iff a current displayable exists**.
- *Live values at boot stop (temp diag, reverted).* `a` singleton `@0x48840020`:
  **`field[0x20]` (run-flag) = `0x48840010` (SET, non-zero)**, but **`field[0x5c]` (current
  displayable) = `0`**. So `a.run` does **not** stall on the run-flag — it bails every iteration
  at `cur == 0` (via `[0x14045fc]`) **before** the per-frame dispatch. (`o` singleton:
  `o.g(field[0x18]) = 0` as expected.)
- *Where `a.field[0x5c]` should come from.* It is the "current displayable" the game establishes
  at startup (a.startApp invokes static `Display.getDefaultDisplay()` and constructs Cards —
  `Card.<init>` ×5, `getDefaultDisplay` ×2 in the boot trampoline trace). In wie it ends up `0`
  on the `a` singleton. The single `str […,#0x5c]` inside the `a`-class code is `0x1c3c` in
  a.startApp, but it targets a *different* object's field array (not the `a` singleton), so the
  writer of **`a_singleton.field[0x5c]`** is **not yet pinned** — that is the next concrete step.

*Verdict.* The wall narrows again and for the first time points at a **wie-side startup wiring
gap rather than an absent per-frame ABI**: `a.run` is present, is the loop, and *would* dispatch
the current card's update each iteration — but `a.field[0x5c]` (current displayable) is never set
on the `a` singleton, so the loop self-exits before the first update. This is a **potential (b)
APP-drivable/wie-fixable candidate** (set/propagate the current displayable into the `a`
singleton's `field[0x5c]` through the legitimate startup path), **not yet confirmed**: it hinges
on pinning the `a_singleton.field[0x5c]` writer and whether wie's `Display.setCurrent`/`pushCard`
path is supposed to feed it. Not implemented this turn (would need that pin; guessing forbidden).
No forcing, no shipped code. *cp40 start:* find the writer of `a_singleton.field[0x5c]` (scan the
"set current displayable" path — likely a `Display`/`Jlet` static the AOT calls whose result the
game stores; check whether wie returns a usable object there) → decide (b) wiring vs (a) wall.

**cp40 — the `getInstance(a)` ↔ `currentJlet` identity split is real but is NOT the cause;
`field[0x5c]` is 0 on BOTH Jlet objects (hypothesis refuted, reverted).** Tested the hypothesis
that `a.run` reads `field[0x5c]` from a *different* `a` instance than the one a.startApp wrote it
on (a `getInstance` returns-fresh-instance bug). Live dump at boot stop:

| object | guest_ptr | `field[0x20]` (run-flag) | `field[0x5c]` (current displayable) |
|---|---|---|---|
| `getInstance(a)` (a.run reads this) | `0x48840020` | `0x48840010` (set) | **`0`** |
| `Jlet.currentJlet` (boot Jlet `this`) | `0x48840010` | `0x48840120` (set) | **`0`** |

- *Identity split confirmed:* `getInstance(a)` (`0x48840020`) **is** a distinct fresh instance
  from the running `currentJlet` (`0x48840010`) — `singleton_instance` instantiates a new object
  rather than returning the live Jlet. (A real latent issue, but see below.)
- *…but it is not the cause:* **`field[0x5c]` = 0 on *both*** objects. Unifying
  `getInstance(a) → currentJlet` would make `a.run` read `currentJlet`, whose `field[0x5c]` is
  **also 0** — the loop would still bail. The current displayable is **never set on either Jlet**.
- *2.2(a) — the `0x1c3c` `str[…,#0x5c]` in a.startApp targets a THIRD object*, not `this`/the
  Jlet: `r6 = fn@0xe3274(a.field[0x14]); r4 = [r6+8]; str r0,[r4+0x5c]`. So a.startApp does not
  write the Jlet's `field[0x5c]` at all. Per the task's own stop rule (third-object ⇒ stop), the
  identity-unification fix is **not implemented** (it cannot start the loop, and would change
  `getInstance` semantics the cp20 run-flag sharing relies on — a regression risk for no gain).

*Refined blocker (cp41 start).* `a.run`'s loop gate `a.field[0x5c]` (current displayable) is
never written on the `a`/Jlet object during boot — not by a.startApp (writes a third object), not
by anything reachable (live = 0). So the open question is **which method is supposed to store the
initial displayable into the Jlet's `field[0x5c]`, and why it never runs / stores 0 in wie** —
i.e. trace the app's "set current card/displayable" call (the AOT analog of
`Display.setCurrent`/`pushCard`) and find the `str […,#0x5c]` whose base resolves (live) to
`0x48840020` or `0x48840010`. That writer (or the platform call feeding its value) is the real
gate; the `getInstance` identity split is a side issue to revisit only if it turns out the writer
*does* target a third instance that `a.run` should but doesn't read.

**cp41 — CORRECTION: `a.run` is ONE-SHOT, not a loop; the `field[0x5c]` gate (cp39) is dead
code. The cp39/40/41 "a.run loop" line is withdrawn; the wall reverts to §7 (cp37).** Before
acting on the cp41 H1/H2 hypotheses, the `a.run` control flow was re-traced from `binary.mod`
**and** cross-checked against the live import trace — and cp39's premise is **false**.

- *Executed path (CFG + live, conclusive).* `a.run@0x1f10` enters at `b 0x212c` (run-flag check:
  `getInstance(a).field[0x20] != 0` → true) → body `0x1f38`. The body calls imports `0x55`
  (`lr=0x1f44`), `0x56` (`0x1f58`), `0x1f` (`0x1f68`), then **`new()` at `0x1f68`** which
  **succeeds** (`r0 != 0`) → `0x1f7c` → import `0x57` (`0x1f94`) → `b 0x1ff4` → import `0x21`
  (`0x48840550`, `lr=0x2108`) → **`0x2108: b 0x2140` = return**. Live trace: each of these imports
  fires **exactly once**, `0x21` at `lr=0x2108` exactly once, and **no import fires at any
  `lr` in `0x2000..0x20f8`** — so the `field[0x5c]` check at `0x20e8` is **never reached** (it sits
  on the `new`-**failed** branch `0x1f9c→…`, not taken because `new` always succeeds).
- *Consequence.* cp39's "`a.run` = `while(field[0x20]){…; if field[0x5c]==0 exit}`" mis-read the
  CFG: the loop-back (`0x213c`) and the `field[0x5c]` exit (`0x20f8`) are both on the unreached
  `new`-failed path. `a.run` actually does **one** registration pass (carried code via
  `0x55/0x56/0x57`, an object via `0x21`) and returns — i.e. exactly the one-shot registrar of
  **cp37**. cp40's identity split (`getInstance(a)=0x48840020` ≠ `currentJlet=0x48840010`, both
  `field[0x5c]=0`) is **factually true but irrelevant**: `field[0x5c]` is never read.
- *H1/H2 not applicable.* Both hypotheses assumed `a.run` reads `field[0x5c]` and loops; since it
  does neither, there is nothing to wire (H1) or unify (H2) for this gate. Not implemented.

*Where this leaves the wall — back to §7 (cp37), re-confirmed.* The per-frame driver is **not**
`a.run` (one-shot). It is the **ez-i runtime driving the object `a.run` registered via `0x21`**
(`0x48840550`, a bare `new`'d native handle) plus the carried code `0x1ad4` (`0x55/0x56/0x57`).
cp37 already showed driving `0x1ad4` is inert (it is init code), and the `0x21` object has no
bound method to call (guessing its vtable slot is forbidden). So the single open question is the
original §7 one: **which entry of which registered object does the ez-i runtime invoke each
frame** — a platform-ABI fact absent from `binary.mod`. *Meta-note:* cp38's `o.g`/`i.b` decode
still stands (the `o.g=1` store and its card-`i` callers are real); what cp39–41 add is the
**negative** result that `a.run` is not the loop and `field[0x5c]` is not the gate — pin
control-flow against an execution trace before building on it.

**cp42 — the `0x21`-registered objects are BARE native handles (no class, global vtable); no
known entry is dispatchable on them → §7 wall confirmed *empirically*, not just inferred
(diagnostic, reverted).** Characterised every object the app hands to import `0x21` at the moment
of registration (temporary classifier on the live guest object):

- *All ten `0x21` registrations are bare.* Each `a0` is `vtable=0x4010022c` (**the GLOBAL identity
  vtable**, not a per-class Card vtable), `pending_new=true`, **not in the instance map** (no JVM
  class — never `<init>`'d). The a.run one (`0x48840550`) is identical. So 3.2 resolves to
  **(ii) bare** for every candidate: there is **no** mapping to app overrides (`o.paint`/`i.a`/`i.b`)
  or to any known platform Card slot *on these objects*.
- *No known entry can be driven on them (3.3 not executable).* A bare object has no JVM class, so
  `invoke_virtual` is impossible; and its only vtable is the global identity table, whose slots are
  platform-method-**by-name** trampolines that `invoke_virtual` on `this` — which NPEs/no-ops for a
  classless `this`. wie's existing trampoline path already **no-ops any vtable slot call on a
  `pending_new` object** (the cp14 "discarded probe" branch). So every candidate per-frame entry on
  the registered object is, by construction, inert in wie — there is nothing to legitimately drive.
- *The behaviour lives in the platform registry, not the object.* `0x21(obj, code=0x1ad4, slot)`
  hands the platform an (object, carried-code, `.data` slot) triple; `0x1ad4` is idempotent init
  (cp37), and the per-frame behaviour is whatever the **ez-i runtime's** registry/dispatch does
  with these triples — code that is **not in `binary.mod`** (the app only *registers*; the loop and
  the choice of which entry to call each frame are the platform's).

*Verdict — §7 platform-ABI wall, empirically established (3.4).* The registered render driver is a
bare opaque handle with no descriptor and no bound method; the ez-i per-frame dispatch is a fact of
the LGT/ez-i platform runtime, absent from the app binary and from public WIPI docs. Reaching it
from wie would require **fabricating** the dispatch (which entry/slot, which object, what cadence)
— forbidden. This closes the app-side investigation: cp37→cp42 have exhausted what `binary.mod`
can yield. **External escalation path:** obtain the LGT/ez-i platform ABI for the native
displayable/clet registry (the `0x21`/`0x55`/`0x56`/`0x57` semantics and the runtime's per-frame
invocation) — e.g. an LGT SDK/runtime reference or a device-side trace of the real platform — then
wie can emulate that registry + per-frame call from the `wie_lgt` side. Until then 배틀몬스터 (and
the other AOT-Java titles that reach this same gate) stay at **0 draw calls**.

---

## 7b. AOT-Java title sweep (cp43)

Headless `wie_validate` sweep of all 39 `broken/lgt` titles to map the LGT-Java (AOT) set
beyond 배틀몬스터 and bucket each by its blocker. AOT-Java = the app registers native class
descriptors (`registered N app classes` > 0); the rest are WIPI-C clets. **17 AOT-Java titles**
found. **Result: 0 of them render; 배틀몬스터 is the only one that even *reaches* §7 — the other
16 die earlier, at boot, on app-side walls.** (The 6 LGT titles that *do* render — 리듬페스티발,
메이플스토리 도적편, 블레이드마스터3, 아니마, 창세기전3ep1, 판타지포에버3 — are **WIPI-C clets,
not AOT-Java**, and were already passing.)

| bucket | count | titles | blocker (raw validator/JVM error) |
|---|---|---|---|
| **[§7]** per-frame ABI wall | 1 | 배틀몬스터 | reaches the wall (cp37–42); `content:false`, 0 draw |
| **[X-paint]** card bound to platform `Card` | 7 | 간호사타이쿤2, 당신은골프왕, 레전드오브마스터, 붕어빵타이쿤3, 서든어택포켓, 슈퍼액션히어로, 현영맞고2006 | `AbstractMethodError: Abstract paint(Lorg/kwis/msp/lcdui/Graphics;)V` via `CardCanvas.paint → card.paint` |
| **[X-vtable]** misrouted hardcoded vtable slot | 8 | 놈3, 메이플스토리2007, 스파이더맨3, 월드장기체스, 체스마스터, 턴, 학교가는길, 훼밀리마트타이쿤 | `NoSuchMethodError` on nonsense pairings: `Thread.serviceRepaints`, `Vector.setVolume`, `Card.d:(II)V`, `String.getWidth`, `Card.A:()V`, `Card.startEngine`, `ShellComponent.getWidth`, `String.setString` |
| **[X-class]** missing app class | 1 | 일지매영웅전기 | `NoClassDefFoundError: atdata.JimaeMD` |

*Two boot walls, both app-side but not one-liners (and all §7-gated even once past boot):*

- **[X-paint] (7) — card-instance binding.** The app card classes (e.g. `GameCanvas`/`Title`/`Logo`)
  *do* declare a concrete `paint` (verified: `paint` method records present in the binary), but the
  instance pushed to `CardCanvas`'s card vector is bound to the **platform `Card`** (whose `paint` is
  abstract), so dispatch hits the abstract method. Root cause: the most-derived `<init>` that wie sees
  is the platform `Card.<init>` (the app subclass `<init>` runs as native ARM and chains to it), so
  `bind_pending` binds to `Card`, losing the app-subclass identity. 배틀몬스터 avoids this because its
  cards come via `getInstance` (handle→class), not `new`+`Card.<init>`. A fix means resolving the
  app subclass at card-bind time — a core change to the bind/dispatch path (regression risk to the
  working `getInstance` path and to clets), so **not attempted this turn**.
- **[X-vtable] (8) — per-class vtable overrides.** Same family as cp4–6/cp29–30 (`known_java_lang_vtable`):
  the AOT calls a class's method by a hardcoded *physical* vtable slot that collides with the global
  identity table, and wie lacks the override for that `(class, slot)`. Each title needs its own slot
  RE'd to the correct method (like cp30 did for `String[35]=toCharArray`). Per-title work.

**Verdict (cp43).** The AOT-Java title set is fully mapped: **0 render**, 1 at §7, 16 boot-walled in
three clusters. Crucially, every cluster is **upstream of §7** — fixing a boot wall only moves a title
to the same §7 per-frame-ABI wall 배틀몬스터 sits at (cp42). So no AOT-Java title can reach `draw>0`
without the §7 platform ABI, regardless of the boot fixes. The boot clusters are nonetheless real,
pinned, app-side next-steps for *reaching* §7: **[X-paint]** = resolve app card subclass at bind time;
**[X-vtable]** = RE each colliding `(class, slot)` and extend `known_java_lang_vtable`. No fix shipped
this turn (both are risky/RE-heavy and §7-gated downstream); buckets recorded for future work.

**cp44 — [X-paint] and [X-vtable-`Card`] are ONE root cause (card-instance binding); a probe fix
advances games past it but reveals a *cascade* of further boot walls (not §7) and regresses clets
(experiment, reverted).** Measured (not inferred) where an X-vtable title goes once past its boot wall.

- *Unified root cause.* In 체스마스터 the app declares class `a` extending `org/kwis/msp/lcdui/Card`
  with concrete `paint(Graphics)V`, `A()V`, `d(IIII)V` (method records present), and `b`/`c` extend
  `Object`. `MobiChess.startApp` `new`s the card and calls the **platform** `Card.<init>` directly (the
  app subclass has no own `<init>`), so `bind_pending` binds the object to platform `Card`; the later
  `card.A()` then dispatches on `Card` → `NoSuchMethodError: Card.A`. The `AbstractMethodError: paint`
  cluster ([X-paint], 7) is the **same** mechanism — `paint` is declared *abstract* on the platform
  `Card`, so it surfaces as `AbstractMethodError` instead of `NoSuchMethodError`. So [X-paint] (7) +
  [X-vtable-`Card`] (스파이더맨3 `Card.d`, 체스마스터 `Card.A`, 턴 `Card.startEngine`) ≈ **10 titles, one
  card-binding bug.** (배틀몬스터 avoids it: its cards come via `getInstance` (handle→class), not
  `new`+`Card.<init>`.)
- *Probe fix (reverted).* When exactly one app class directly extends a platform class, redirect
  `bind_pending` to that subclass (체스마스터: `Card`→`a`). It **works and advances**: 체스마스터 boots
  past `Card.A` deep into `startApp` (PC `0x1626`→`0x19a8e`); 스파이더맨3 advances `Card.d`→
  `NoSuchMethodError c.show()V`; 슈퍼액션히어로 → `h.getNumberOfRecords()I`. But:
  - **Not §7.** 체스마스터's new endpoint is `Allocation failure` after a ~461 s alloc loop in `startApp`
    — a *different, deeper* wall, reached **before** `a.run`/§7. The others hit **more** `NoSuchMethod`
    walls (a per-method/per-class cascade), also pre-§7.
  - **Regresses clets.** 제노니아1 (WIPI-C clet, normally PASS `content=true`) **hangs >30 s** with the
    fix and **recovers to PASS on revert** — confirmed regression (clets register descriptors that
    populate the redirect map and mis-bind). So the naive unique-subclass redirect is **unsafe**.
- *Measurement verdict.* cp43's "all 16 are §7-gated" is **not confirmed** for the X-paint/X-vtable
  titles: they are blocked by a **cascade of boot walls** (card-binding → more vtable/NoSuchMethod →
  allocation) that is **measured to not cleanly reach §7**. Fixing one wall reveals the next.
  Recommendation: **do not** ship the unique-subclass redirect (clet regression) and **do not** pursue
  the 8-title full vtable RE (each title is a multi-wall cascade, not one slot). The tractable next
  step is a **clet-safe** card-binding fix (bind a `new`+platform-`Card.<init>` object to its app
  subclass *without* touching the clet path — e.g. gate on app-class registration being non-empty AND
  exclude the descriptor-scan false positives clets trip), then re-measure the cascade depth. Until
  then these titles stay boot-walled; battle's §7 remains the only *confirmed* §7 case.

**cp45 — the card-binding fix IS clet-safe (cp44's "regression" was load, not the fix); kept in
tree, push held. It advances 9/10 binding titles and MEASURES 4 more to reach §7 (still 0 render).**
The cp44 fix is made permanent in the tree (push held — 0 new render, and a 체스마스터 alloc-hang to
weigh first): in `bind_pending`, when exactly one app class directly extends the platform class
being constructed, bind the guest object to that app subclass instead of the platform class. The
map (`platform_to_app_subclass`) is populated only from registered **app** classes, so it is empty
for WIPI-C clets and the redirect is inert on the clet path.

- *cp44's clet "regression" was a false alarm.* 제노니아1 registers **0 app classes** ⇒ the redirect
  map is empty ⇒ the fix is provably inert for it. Re-tested run **alone** (not contending with the
  leftover 체스마스터 ~461 s alloc-loop processes that were starving the cp44 batch): 제노니아1 **PASS
  `content=true` d=9, no hang**, identical to baseline. The cp44 hang was host load, not the fix.
- *Regression gate — 0 (all run alone).* Clets 제노니아1 (d=9) / 그랜드체이스 (d=2) PASS; 하이브리드 /
  배틀몬스터 unchanged (`content:false`, §7); baseline 막시민편 (d=3) and the rendering clets 아니마
  (36) / 창세기전3ep1 (257) / 블레이드마스터3 (27) / 판타지포에버3 (4) / 메이플스토리 도적편 (34) /
  리듬페스티발 (15) all PASS with identical `distinct_colors`. `getInstance` path (battle) untouched.
  `cargo test --workspace` = 33 suites green; `clippy --workspace` clean.
- *Cascade endpoints, measured with the fix (35 s watchdog):* the redirect advances **9 of 10**
  binding titles past the card-binding wall:
  - **(b) reach §7** (the battle-type blank screen, `content:false` d≈1): **턴, 레전드오브마스터,
    서든어택포켓, 현영맞고2006** — so these 4 are now *measured* (not inferred) to be §7-gated, like
    battle. cp43's "all §7" inference is **confirmed for these**.
  - **(c) deeper app-side wall**: 체스마스터 → `Allocation failure` (~461 s alloc loop in `startApp`;
    a separate wie/app alloc issue, not the binding); 스파이더맨3 → `NoSuchMethodError c.show()V`;
    슈퍼액션히어로 → `h.getNumberOfRecords()I`; 붕어빵타이쿤3 → `d.show()V`; 간호사타이쿤2 →
    `NetClient.getWidth()I`. A further per-method cascade, still pre-§7.
  - **unchanged**: 당신은골프왕 (multiple app `Card` subclasses ⇒ no *unique* redirect; needs
    per-instance class resolution, not covered).
  - **(a) render: 0** — no title reaches `draw>0`; every advanced title hits either §7 or a deeper wall.
- *Net.* The fix is a correct, clet-safe bug fix that turns 4 titles' §7-gating from inference into
  measurement and pushes 5 others to deeper, named app-side walls — real forward progress, but **no
  new on-screen render** (all still black: §7 or the next wall). Two open follow-ups, both app-side:
  (1) the 체스마스터 `startApp` alloc loop (bound or fix the runaway allocation); (2) per-instance class
  resolution for multi-`Card`-subclass titles (당신은골프왕). The §7 titles (battle + the 4 newly
  measured) remain blocked on the external platform ABI (cp42).

**cp46 — the 체스마스터 alloc-loop diagnosed + a runaway guard added; binding fix + guard pushed
(every title now strictly better-or-equal).** Diagnosed the ~461 s hang the cp45 binding fix
exposed in 체스마스터, then bounded it so it fast-fails.

- *Diagnosis (measured).* Past the card-binding wall, 체스마스터's `startApp` enters a loop whose body
  is `obj = new(); import 0x1f(0, code=0x1b252, obj, slot)` — the ez-i object/carried-code
  registration — and it **never terminates**: in 5 s of debug trace it ran **29 238×** (`import 0x1f`
  + `stdlib new`, 1:1). wie no-ops `import 0x1f` (registration), and the loop's termination depends on
  the runtime side wie doesn't emulate, so it `new`s ~2.6M objects over ~461 s until the allocator
  (which slows as the heap fills, ~5800→~1100 obj/s) finally OOMs. Type (ii)/(iii): a non-terminating
  loop wie can't close (same §7-family gap — the registration semantics live platform-side), **not**
  the binding fix being wrong.
- *Guard (cp46).* `alloc_native_object` (the AOT `new` primitive, stdlib `0x32` / java `0xf`) now caps
  native-object count at `NATIVE_OBJECT_LIMIT = 16384` and returns a fatal error past it (returning
  NULL doesn't help — the loop ignores `new`'s result and just spins faster). So a runaway loop
  **fast-fails (~4 s, surfaced "tick error")** instead of hanging ~461 s. The cap is far above any
  real boot: every AOT title measured creates **< 1000** native objects at boot (battle ≈ 12; the §7
  titles 턴/레전드오브마스터/서든어택포켓/현영맞고2006 all < 1000), so it never trips a legitimate game.
  Clets don't use `alloc_native_object` (WIPI-C alloc path), so it is inert for them.
- *Verification (run alone, no host-load contamination).* 체스마스터: **4 s fast-fail** (`runaway
  guard`) vs the old 461 s hang. §7 titles still reach the blank screen (`content:false`, unaffected
  by the cap). Regression 0: clets 제노니아1 (d=9) / 그랜드체이스 (2) / 하이브리드 PASS-as-baseline;
  battle unchanged; baseline 막시민편 (3); renderers 아니마 (36) / 창세기전3ep1 (257) / 판타지포에버3
  (4) / 메이플스토리 도적편 (34) all identical. `fmt`/`clippy --workspace`/`test --workspace` (33
  suites) green.
- *Shipped.* With the alloc-hang bounded, the cp45 card-binding fix + this guard are **pushed
  together**: every title is now strictly better-or-equal (체스마스터 461 s hang → 4 s fail; 9/10
  binding titles advanced; 0 regressions). Still **0 new on-screen render** — the advanced titles sit
  at §7 (4, platform-ABI) or deeper per-method walls. Remaining app-side follow-ups unchanged:
  multi-`Card`-subclass per-instance resolution (당신은골프왕), and the per-method `NoSuchMethod`
  cascade (스파이더맨3 `c.show`, 슈퍼액션히어로 `h.getNumberOfRecords`, etc.).

**cp47 — 놈ZERO "garbage calloc size" is NOT a wie marshalling/struct-offset bug: ground-truth
file extraction disproves it; the clet renders menus+text+textured bg.** A round had reframed the
놈ZERO failure as a single Rust↔ARM boundary defect — a struct field offset / endianness /
pointer-width / ABI constant that supplies garbage where an integer size belongs (the
`calloc(0x72657473)` ≈ "ster" / "monster" signature), unifying "garbage size" and "garbage decode
pixels" under one root. **Measurement disproves this framing on every count:**

- *The crash reports were stale.* `game_lab/reports/놈ZERO.json` (SVC `0x415`, `getNextEvent`) and
  `제노니아1.json` (SVC `412`) predate landed fixes (`memmove 0x415`, `getNextEvent` null-guard,
  `calloc` OOM→NULL, `ListDatabases 412`). **Current headless 놈ZERO = PASS**: boots, 153 paints,
  34 distinct colors, survives the full 27-key inject sequence with no crash/panic. Screens progress
  LGT loading-frame → menus with legible Korean bitmap-font text + a textured (`.pzx`) background.
  Not garbage stripes, not black.
- *The "garbage size" is the game's own data, supplied byte-perfectly.* Trace at the `calloc` site:
  the game `stream_read`s `table/cur_figure.dat` (1036 B) into a buffer, then reads the dword at
  **buffer+4** as a size → `calloc(0x01010101)`. Extracting the real `table/cur_figure.dat` from the
  jar and hexdumping it: bytes are `0d 3c 11 01 | 01 01 01 01 | …` — **offset 4 genuinely is
  `0x01010101`**. wie's DB `stream_read` AND `MC_knlGetResource` both deliver these exact bytes
  (verified against the jar). The sibling `string/popup.zt1` has offset+4 = `0x1274` (its real zlib
  uncompressed length, header `78 9c`) and the game `calloc`s that correctly. ⇒ The game's resource
  loader applies the `.zt1` `[complen][uncomplen][zlib]` convention to a **raw `.dat` table**, reads
  a stat byte-run (`01 01 01 01`) as a length, over-allocs ~16 MB, and **frees it immediately**
  (observed). No struct offset, no endianness, no pointer-width, no ABI error — the marshalling is
  correct. The `0x72657473`="ster" variant (different file/branch, "처음부터 시작") is the **same
  mechanism**; the existing `calloc` OOM→NULL guard is the correct WIPI `malloc` contract for it.
- *Size and pixels share NO root.* 놈ZERO issues **zero blit SVCs** (no `DrawImage`/`PutPixel`/
  `SetRgbPixels`/`CreateImage`) — only `FillRect`/`DrawRect` + direct writes to a framebuffer pointer
  from `GetScreenFrameBuffer`/`CreateOffScreenFrameBuffer`, with `GetPixelFromRGB` (569×) for color.
  That path is format-consistent 16bpp RGB565 across `get_pixel_from_rgb` / `get_display_info` /
  `FRAMEBUFFER_DEPTH` / `FrameBuffer` storage / `flush_lcd` (re-confirms prior "stride/FB normal").
- *Differential (제노니아1 clean vs 놈ZERO broken) inverts.* In current headless, 제노니아1 sits at
  the **same** green LGT loading frame and renders **less** than 놈ZERO, which advances to textured
  menus. So 놈ZERO (clet) is not the regressed one; the framing that drove "여러 라운드 교착" was the
  stale-report bug-attribution, now corrected.

*Verdict (cp47).* No confirmed wie-side defect behind the "garbage size." Per the no-blind-fix /
preserve-passing-games rule, **0 code changed** (a blind special-case would risk the now-passing
state for no proven gain). Remaining open item is **full `.pzx`/`.ft2` sprite fidelity** (a visual,
on-device judgment) — advancing it needs either the **`.pzx`/`.ft2` format spec** (absent from
repo/env → external, hold) or **dynamic ARM memory-watchpoint tracing** (not exposed by wie's
`ArmCore`), exactly as cp35/cp42 flagged. Doc-only checkpoint; no game behavior changes.

**cp48 — live LGT/AOT re-baseline + the §7 "free MSP option" exhausted by trace: the per-frame
driver is an ez-i UPDATE-tick dispatch, NOT the MSP paint contract (branch (a), code 0).**

*STEP 1 — live matrix (no cached reports; current code, headless boot + 27-key inject).* The cp43
buckets **hold** — the validator now wraps each inner error as `native dispatch <App>.startApp/<init>
@…: <inner>`, but the `<inner>` is the cp43 error verbatim (e.g. 놈3 = `NoSuchMethodError
Thread.serviceRepaints` = cp43 [X-vtable]; 당신은골프왕 = `AbstractMethodError paint` = [X-paint];
일지매영웅전기 = `NoClassDefFoundError atdata.JimaeMD` = [X-class]). **AOT-Java: 0/17 draw.**
배틀몬스터 alone reaches §7 (`paints:1`, `content:false`, blank). **Clets (live):** 놈ZERO ✅230p/69c,
그랜드체이스 ✅228p/512c, 라테일 ✅147p/316c, 게임빌2010슈퍼사커 ✅280p/4c. **Two pre-existing
clet issues surfaced (NOT introduced here — this round changed 0 code):** 하이브리드 = blank at boot
(`content:false`) + null-jump (`PC=0`) in `EventQueue.getNextEvent` on first OK; 제노니아1 = runaway
single-`tick()` spin under inject (host-load-sensitive, same family as 체스마스터/cp46). Both are
clet-path, pinned for a dedicated round.

*STEP 2 — free MSP option (in-repo WIPI 1.1.1 `org.kwis.msp.lcdui`) exhausted on 배틀몬스터 (the only
§7 title).* The Java contract is intact and live: `Display.<init>` makes `net/wie/CardCanvas` the MIDP
current Displayable; each repaint runs `Display.handlePaintEvent → Canvas.handlePaintEvent →
CardCanvas.paint → ∀ card: card.paint(g)`. Trace of 배틀몬스터 shows this loop **runs** (2×
`handlePaintEvent`/`CardCanvas.paint`) but produces **0 `Graphics` draw calls** because:
- The app **never calls `Display.pushCard`** (0 occurrences) — so `CardCanvas.cards` is empty. It
  constructs `Card::<init>` ×10 (handles `0x…120/170/190/1b0/4f0…`) and instead hands a displayable to
  the ez-i runtime via **native `import 0x21`** — which wie implements as a **no-op** (only `0x9`
  string-factory / `0xc` getInstance are live). `0x21` is overloaded: 8 calls `(obj,0,const)`, one
  `(0x48840540, Card 0x48840120, Jlet 0x48840010)` @lr `0x227c`, one `(0x48840550, code 0x1ad4, 0)`
  @lr `0x2108` (= a.run carried code, cp41).
- *Why wiring `0x21(_,Card,_) → CardCanvas.pushCard(Card)` would still not render (the key result):*
  the app's draw is gated on **`o.g`**, set **only** by the card's **update** method `i.b(_,_,0)`
  (cp38) — **not** by `paint`. The MSP dispatch contract only ever invokes `card.paint(g)` on repaint;
  it never invokes the card's update method. Trace confirms `Game.<init>/a/b` + `a.startApp/a.run` each
  run **exactly once** at boot (one-shot, cp41) — `Game.b` (=`i.b`, the `o.g` setter) fires once at
  init and **never per-frame**. So even a fully-wired pushCard yields `paint` with `o.g=0` ⇒ 0 draws.
  No forcing-free MSP path dispatches `i.b` per frame.

*Verdict — branch (a), platform-ABI-gated, code 0.* The in-repo MSP reference defines the **paint**
contract but not the **per-frame update-tick dispatch** the app actually depends on; that is ez-i
runtime-private and its consumer is absent from `binary.mod` (cp42). **★ The precise missing fact:**
on each frame the ez-i runtime invokes — *on which registered object* (the `0x21` a0 handle e.g.
`0x48840550`, or the carried `Card` a1 `0x48840120`?) — *which method* (the card update `i.b`? via
which vtable slot, given the registered object is a bare global-vtable handle with no JVM class, cp42)
— *with which arguments* (cp38 saw `i.b(_,_,0)`; p1/p2/p3 unconfirmed) — *at which cadence* (every
repaint / fixed timer / vsync?). **Resolvable only by:** the ez-i SDK *native* runtime (the
`org.kwis.msp` platform impl, not the Java API stubs) / the LGE Xceed VM runtime / a real-device
execution trace of an AOT title. Doc-only checkpoint; no code, no game behavior changes.

## 8. Current reach

| stage | state |
|---|---|
| app classes registered, methods run as real ARM | ✅ |
| boot (`0x64` java-interface → `Main.main` → Jlet) | ✅ |
| two-level vtable + per-class overrides + instance field layout | ✅ |
| `getInstance` singletons, `Thread.start`, game thread spawns `a.run` | ✅ |
| data load → 240×320 back-buffer → `getGraphics` → Cards/RNG | ✅ |
| app `Card.paint` ticked in wie's loop (cp26 experiment) | ◑ wires in & runs per-frame, but `o.paint` gates on `o.g` which its (never-run) virtual `o.k()V` writer would set → **0 draws** (cp27, §7) |
| render path with `o.g` forced to 1 (cp28 experiment) | ✅ `o.paint` draws (21× setColor, 18× fillRect) to back-buffer + flushes → **render path works end-to-end**; only "set `o.g`" (ez-i per-frame drive) is missing (§7) |
| `java/lang/String` slot 35 = `toCharArray()[C` (cp30) | ✅ per-class override added; `String.e` abort gone, `o.paint` runs without fatal. Title text still blocked on char-array guest marshalling (cp31) |
| char-array guest marshalling (cp31) | ✅ `materialize_char_array` → `{u32 len, u16 chars}` at `[arr+8]` (RE'd, unit-tested; `len=10 "LOADING..."`) |
| glyph loop runs, consumes chars (cp32) | ✅ confirmed: loop runs 30× (3 frames × 10 chars), reads "LOADING..."; the `0x10298` "gate" is just `getColor` (both paths fall through) |
| glyph-draw fn runs; no font image (cp33) | ◑ `@0x109b4` is called per char but takes its `r6==0` (no font image) path → `import 0x22` no-op, **0 drawImage**. Root cause: bitmap-font sheet absent guest-side |
| glyph blit mechanism RE'd (cp34) | ◑ blit = `g.drawImage(sheet, src_x=(char-0x21)*10, w=10)`; font path via `import 0x22(a0=font_img, a1=0x11264→fn 0x10fb0)` |
| font path resolved → platform-gated (cp35) | ⛔ probe: `r6=g` (not the font img — cp33/4 corrected); font img = `0x22` a0 = **0** every char; `0x10fb0` = strb bookkeeping (no draw); **no font `createImage`** in the reachable run (only the 240×320 back-buffer). Font load/native render is §7-gated, not an app one-liner |
| wie can't substitute the ez-i tick (cp36) | ⛔ ~~`fn@0xda870` driven → `o.g` stays 0~~ — **corrected by cp38**: `fn@0xda870` is the `o.g=0` *resetter*, not the setter; the "accumulated state" conclusion is withdrawn |
| registered carried code is INIT, not the frame step (cp37) | ⛔ `0x55/0x56/0x57/0x21` all register one entry `0x1ad4→0x1a24` = straight-line idempotent init (full disasm; arg-ignoring). Synthesized per-frame drive of it ran clean but **stayed black** (inert, extends cp23). Per-frame entry is a method on the `0x21` object via the platform's native-displayable ABI (absent from `binary.mod`). Reverted |
| `o.g=1` store decoded; gate = un-dispatched card method (cp38) | ⛔ store `0xdb240` is **unconditional** within `fn@0xdb200` ("show card"); reached only from card `i.b(_,_,0)` / `i.a()`, which are **vtable-dispatch-only**. Boot dispatches **5 methods, zero card methods**; `o.g=0` at stop; driving `i.b(0,0,0)` flips `o.g=1` in one call (forcing, reverted). Gap = absent ez-i dispatch of the current card's update method (§7), not an unsatisfiable predicate |
| ~~`a.run` is the loop; exits on null current displayable (cp39)~~ | **WITHDRAWN by cp41** — CFG mis-trace. Still valid from cp39: **no `notifyEvent` override exists** (the app keeps `o.paint`/`o.keyNotify`/`a.run`/`a.startApp` names). |
| ~~`getInstance(a)`≠`currentJlet` (cp40)~~ | identity split is **true but irrelevant** (cp41): `field[0x5c]` is never read by `a.run`. Recorded for the record only. |
| **CORRECTION: `a.run` is one-shot; `field[0x5c]` is dead code (cp41)** | ⛔ CFG + live: `a.run` body takes the `new`-succeeds path (`0x55/0x56/0x57/0x21`, one each) → returns at `0x2108`; the `field[0x5c]` check (`0x20e8`) is on the unreached `new`-failed branch. Reverts the wall to §7/cp37: per-frame driver = ez-i runtime invoking the `0x21`-registered object (`0x48840550`) — platform-ABI, absent from `binary.mod` |
| `0x21` objects are BARE handles → §7 wall confirmed empirically (cp42) | ⛔ all 10 `0x21`-registered objects = GLOBAL vtable, no JVM class, `pending_new` (live classifier). No `invoke_virtual` possible; global-vtable slots no-op on a classless `this` (cp14). The per-frame entry is platform-runtime ABI not in `binary.mod`. **App-side exhausted (cp37→42); external escalation: obtain LGT/ez-i platform ABI** |
| AOT-Java title sweep: 17 titles, 0 render (cp43) | ⛔ 1 at §7 (battle), 7 [X-paint] `AbstractMethodError paint`, 8 [X-vtable] misrouted `NoSuchMethod`, 1 [X-class] `NoClassDef`. All boot walls upstream of §7 |
| [X-paint]+[X-vtable-`Card`] = card-binding; cascade not §7 (cp44) | ◑ measured: app card `new`+platform `Card.<init>` binds to `Card`, losing app subclass (paint/A/d fail). Unique-subclass bind-redirect **advances** titles but hits a **cascade** of further boot walls. (cp44's "clet regression" was host load, not the fix — see cp45) |
| clet-safe card-binding fix + runaway alloc guard, **pushed** (cp45/cp46) | ✅ `bind_pending` redirects a `new`+platform-`Card.<init>` object to the unique app subclass; map empty for clets ⇒ inert on clet path. Regression 0 (제노니아1 d=9 no hang, all renderers identical, `test --workspace` green). Measures 턴/레전드오브마스터/서든어택포켓/현영맞고2006 to §7; 5 others to deeper walls. **0 new render** (still §7/next-wall) |
| **per-frame render driver** | ⛔ blocked on ez-i render-tick ABI (§7) — **0 draw calls** (AOT-Java path) |
| 놈ZERO (clet) renders; "garbage size" not a wie bug (cp47) | ✅ boots, 153 paints, menus+text+textured `.pzx` bg, survives full inject. `calloc(0x01010101)`/"ster" = game loader reading byte-correct `cur_figure.dat` (offset+4 = `01 01 01 01` in the jar; verified). Size ≠ pixel root; FB path RGB565-consistent. Open: `.pzx`/`.ft2` fidelity (external spec / watchpoint-gated) |
| live re-baseline + §7 free-MSP option exhausted (cp48) | ⛔ AOT 0/17 draw (cp43 buckets hold); 배틀몬스터 at §7 blank. App sets displayable via native `import 0x21` (no-op in wie), never `Display.pushCard`; even wired, draw-gate `o.g` is set by card update `i.b` (cp38), which MSP `card.paint` never calls ⇒ 0 draws without forcing. Missing ez-i fact: which registered obj / which method+slot / which args / which cadence the runtime ticks. **External: ez-i native runtime / Xceed VM / device trace.** Code 0. (Pre-existing clet issues noted: 하이브리드 blank+null-OK, 제노니아1 inject spin) |
| clet regression (`test_helloworld`) / `clippy -p wie_lgt` | ✅ clean |
