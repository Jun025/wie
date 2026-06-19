# STEP report — LGT native-backed JVM (checkpoints 1–9)

Goal: run the AOT-compiled LGT app (BattleMonster `00025C2B`) on wie's JVM, toward
`startApp` → `paint(Graphics)` (title). Branch `feat/lgt-java-interface-bridge`
(local only). PoC `LgtJvmShared` kept LGT-specific per #1232; shared
`wie_wipi_java`/`wie_midp` classes **not** modified.

## Status summary

| item | result |
|---|---|
| cp1–2: app classes registered; methods run as real ARM | ✅ |
| cp3: `java_load_classes` tables; native↔platform bridge | ✅ |
| cp4–5: per-class vtables; Runtime vtable wall crossed | ✅ |
| cp6: two-level virtual dispatch | ✅ |
| cp7: stdlib `0x32` = native allocator | ✅ |
| cp8: java `0xf` = native object allocator; `new StringBuffer()` constructs | ✅ |
| cp9: per-class platform vtable / native-object investigation | ⏹ STOP (B) — **superseded by cp10** |
| cp10: StringBuffer wall crossed — String factory + per-class vtable + append bridge | ✅ |
| cp11: native-instantiated object — first investigation → STOP (B) | ⏹ |
| cp12: P3 re-attack — import thunks decoded, `r8` is an app object w/ compiled-away class | ⏹ |
| cp13: static-type identification of `r8` — disproven (AOT slot reuse) | ⏹ |
| cp14: forward-probe — `r8` non-critical; collisions gated by offset-table dispatch | ⏹ |
| cp15: reserved-slot-0 global vtable — offset-table off-by-one fixed; FULL BOOT + paint loop | ✅ |
| cp16: diagnosis — data-load gated by uninitialised instance field_offsets | ⏹ |
| **cp17: inheritance-aware instance field_offsets — data load + 240×320 back-buffer run** | ✅ |
| **cp18: Thread.start per-class slot + pending-new pass-through — game-loop thread spawns a.run()** | ✅ |
| cp19: diagnosis — a.run() returns early on `getInstance(a)` run-flag (no-op import) | ⏹ |
| cp20: implement `getInstance` (java-interface `0xc`) singletons — a.run enters loop + back-buffer Graphics | ✅ |
| cp21: diagnosis — a.run is a one-shot; render gated by AOT-runtime import substrate | ⏹ |
| **cp22: substrate RE — no draws anywhere; imports overloaded/stack-ptr args, resist single-shot RE (STOP D, stuck)** | ⏹ |
| `paint`/title | ◑ full setup (data, back-buffer, getGraphics, Cards, RNG); render driver never starts → zero draw calls |
| clet (`test_helloworld`) | ✅ | clippy | ✅ |
| clet (`test_helloworld`) | ✅ | clippy | ✅ |

## Checkpoint 22 — substrate RE: no draws; imports resist single-shot RE (STOP D, stuck)

Pushed on the cp21 substrate. Two firm results + a refined wall.

### Confirmed: the render driver never starts (zero draw calls)
Across a full run, **no** `drawImage`/`fillRect`/`drawString`/`drawLine`/`copyArea`/
`flushLcd`/`setColor` — anywhere. Every platform call is **setup**: class `<init>`s,
`getInstance`, `Card.getHeight/getWidth`, `Component.getHeight`,
`AnnunciatorComponent.show`, `Random.<init>`, `Math.min`, the StringBuffer/data load,
`getGraphics`. So the game completes setup but its per-frame render driver never runs.

### a.run is structurally one-shot (not import-gated)
a.run's body main path returns **unconditionally** (`b 0x1ff4` → `b 0x2100` → return),
so it is not a loop regardless of the imports. Confirmed at runtime (a.run RETURNED;
its `0x55` fires exactly once). So the render driver is *not* a.run's loop — it is the
object a.run allocates (`stdlib 0x32`) and hands to `0x57`/`0x21`; whatever per-frame
mechanism should invoke it (timer/thread/callback) is **never observed** (no timer
call, no second thread, no `pushCard`).

### The substrate imports resist confident single-shot RE
Call-site args (a3 is the dispatch thunk, ignore it):
| import | observed args | shape |
|---|---|---|
| `0x12` | `(0, 0, sp=0x4010022c)` | stack ptr only |
| `0x1f` | `(0, obj/type, count)` e.g. `(0,obj,4)`, `(0,0xda958,0x38)` | obj/type + count |
| `0x21` | `(obj, 0, sp)` | obj + stack ptr |
| `0x22` | `(0, n, …)` | small ints |
| `0x55` | **`(a-sing, code@0x1ad4, 0)`** *and* **`(0, 4, 8)`** (different call sites: a.run vs `i.Q`) | **overloaded** |
| `0x56` | `(this, code@0x1ad4, 0)` | obj + code |
| `0x57` | `(this, code@0x1ad4, 0)` and `(this, obj, this)` | obj + code/obj |

- `0x12/0x21` pass a **stack pointer** (`0x40100…`), and `0x1f/0x22` pass obj+count —
  the signature of **GC / safepoint / exception-frame** primitives, which are
  plausibly *safe* as no-ops in an emulator without GC (consistent with the game not
  crashing). So these are likely **not** the render blocker.
- `0x55/0x56/0x57` carry `code@0x1a24` (an a-singleton **state initialiser**: sets
  fields `[9],[13],[14],[15],[18]…`, allocates a sub-object, does **not** draw). But
  `0x55` is **overloaded** — `i.Q` calls `0x55(0, 4, 8)` (a1=4 is not a code pointer) —
  so it is **not** a uniform "run code@a1". They look like a try/synchronized triple
  (handler code passed, no-op-safe if no exception) rather than the render trigger.

### Why STOP (condition D — stuck, 2 passes)
The substrate is **not** a single missing function: it is a batch of AOT-runtime
primitives with overloaded, context-dependent args (stack ptrs, codes, objects) that
**cannot be implemented from the call sites alone without guessing** — and a guess/
blanket is disallowed (and risky: these touch GC/exception/threading). The render
driver's per-frame mechanism is also unobserved. Two passes (cp21, cp22) produced
diagnosis but no safe functional fix. This needs a deeper, different RE angle.

### Recommended next angle (cp23)
- RE the **import dispatcher** `0xe31d8`/`0xe31a8` (behind the `bl`+`.word table/idx`
  thunks) — it may reveal the intended ABI/semantics of the whole java-interface table
  at once (vs guessing per index).
- Or trace the **object a.run registers** (`stdlib 0x32` alloc → `0x57`/`0x21`): bind
  it, see what methods are later invoked on it, to identify the per-frame driver.
- Or look for a **timer**/repaint registration path the game expects wie to drive
  (the game never `pushCard`s; it blits a back-buffer — find what triggers the blit).

## Checkpoint 21 — a.run is a one-shot; render gated by the AOT-runtime import substrate

After cp20 `a.run` enters its body. Full RE of the body (`0x1f10`–`0x2128`) shows it
is **not** a continuous render loop — it is `if (run-flag) { body }` and the body's
main path **returns** (`0x1ff4 → 0x1ffc → 0x2100 → 0x2140`). Confirmed at runtime:
`a.run RETURNED -> 0`, after which the **main thread spins at ~99% CPU** on
`CardCanvas.paint` (whose `cards` vector is empty → draws nothing).

### What a.run's body does (main path), and the blocker
```
import 0x55(a-singleton, code@0x1a24, 0)   ; result discarded
import 0x56(this,        code@0x1a24, 0)   ; result discarded
import 0x1f(0)
obj = stdlib_0x32()                        ; native allocator (cp7) — non-null
import 0x57(this, obj, this)               ; result discarded
import 0x21(obj)                           ; result discarded
return
```
- `code@0x1a24` (passed to 0x55/0x56) is a **state initialiser** for the a-singleton:
  it `getInstance(a)`s, sets fields `[9]=0,[13]=1,[14]=1,[15]=new(),[18]=3,…`, and
  does **not** draw. So 0x55/0x56 carry an init routine the runtime is meant to run
  (or register); as no-ops, that init never happens.
- `obj` (allocated, then handed to 0x57/0x21) looks like a runtime object the game
  **registers** for its real per-frame driver (timer/thread/callback). 0x57/0x21 are
  no-ops, so nothing is registered → no render driver runs.

So `a.run` only **sets up** the game's run substrate through a batch of java-interface
imports — **`0x55, 0x56, 0x57` (code/object lifecycle), `0x1f, 0x20, 0x21, 0x22`
(object alloc/init/register/finalise), `0x12` (a loop-continue gate elsewhere in the
body: `cmp r0,#0; bne …` — returning 0 exits)** — all currently no-op stubs. Without
them the game's actual render driver never starts.

### Render model: back-buffer blit, not pushCard (confirmed)
The game never calls `Display.pushCard` (no `push_card` in any trace) — so the empty
`CardCanvas` is expected. It renders by drawing to its **240×320 back-buffer Image**
(`getGraphics` obtained) and blitting to screen — driven by the substrate above.

### Next (cp22) — identify + implement the substrate imports (P2), no guessing
RE each of `0x55/0x56/0x57` and `0x1f/0x20/0x21/0x22`, `0x12` from their call sites
(args, result use, the dispatcher) and the AOT runtime — they are the object/thread/
timer/monitor/exception primitives. In particular determine whether 0x55/0x56 *run*
or *register* `code@0x1a24`, and what `obj` (0x57/0x21) is (a timer/thread for the
per-frame loop). Implement per evidence so the run substrate starts → the game draws
to the back-buffer and blits → title pixels. This is a multi-import batch, not a
single fix; a guess/blanket is disallowed.

## Checkpoint 20 — getInstance singletons; a.run enters its loop

cp19's run-gate is `a.run: obj = getInstance(a); if (obj.field[8] != 0) loop`.
RE of `getInstance` (`func@0x1908` → `func@0x18ac`): it is **java-interface import
`0xc`** — `import_0xc(class_handle, registry)` returns the class's canonical
singleton instance, which the AOT dereferences (`obj.field[..]`). Left as a no-op it
returned **0**, so every `getInstance` produced an inconsistent phantom: `a.startApp`
wrote the run-flag into one, `a.run` read another → flag always 0 → loop self-gated
off. (Memory IS shared across the spawned thread — confirmed: `a.Display` written on
the main thread is visible to `a.run`.)

### Fix (P2/P5)
`import 0xc = singleton_instance(class_handle)` now returns a **stable, cached**
instance per class descriptor handle — lazily instantiated as a bound app object
(with its guest field array), shared via `LgtJvmShared.singletons` across threads.

### Verified
- `getInstance(a-handle 0x1400df4)` → stable `a` singleton `@0x48840020`.
- `a.startApp` writes its `field[8] = 0x48840010` (the Game instance); `a.run` reads
  the **same** `field[8]` (non-zero) → **enters the game loop** (`0x1f38`).
- The loop obtains the back-buffer Graphics (`Image.getGraphics()`).

### Next (cp21) — the game loop's imports
`a.run`'s loop body (`0x1f38`) calls java-interface imports **`0x55`, `0x56`,
`0x57`** (new), plus `0x1f/0x12/0x20/0x21/0x22` and app fns `0xe2c50`/`0x235c`. They
are no-op stubs; the loop runs **~1 iteration then exits** (not spinning — each of
0x55/56/57 fires 1–2×). Args: `0x55(a-singleton, code@0x1ad4, 0)`,
`0x56(this, code@0x1ad4, 0)`, `0x57(this, obj, this)` — shaped like
register-callback / monitor / event-poll primitives. Identify 0x55–0x57 (and whether
`func@0xe2c50` is the per-frame render/update gated by them), implement per P2, so the
loop iterates and renders → `pushCard` + `drawImage` → the title.

## Checkpoint 17–19 — data load, back-buffer, game thread; now at the run-flag

A productive arc on top of cp15's full boot. Each step was an evidence-based,
no-guess fix; the app now executes deep into its own startup.

### cp17 — inheritance-aware instance `field_offsets` (data load + back-buffer)
cp16's gate (`a.startApp`: `if (Game.field[field_offsets[148]] != 0) skip-setup`)
was wrong because `field_offsets` was filled for static fields only, so every
instance ref aliased slot 0. Fix: `register_app_classes` now computes each app
class's instance-field object slots = **(app-ancestor field count) + declared
index** (inherited-first flat guest layout; platform ancestors → 0, their fields
live JVM-side) and stores them; `java_load_classes` segments the `fields` array
(grouped by owning app class — verified offline: 150/150 matched, e.g. o's 11
fields, then l's 56, …) and writes `field_offsets[k] = slot`. The gate now reads
the real (null) `a.Display`, so the setup branch runs: **`Game.a()` data load
executes** (StringBuffer filenames ×18) and the game **builds its 240×320
back-buffer** (`Image.createImage(240,320)`).

### cp18 — game-loop thread (Thread.start + r8 pass-through)
`a.startApp` then spawns the loop thread; two direct-`vtable[N]` collisions:
- **r8 back-buffer probe** (cp11/14): `i.<init>` does `r8 = new(); raw-init;
  if(r8) r8.vtable[11]()` with the result discarded. r8 is a native-`new`'d app
  object (class compiled away); slot 11 misroutes to `Graphics.getClipHeight`.
  Pass it through (return 0), scoped to `pending_new` unbound objects (bound
  platform calls never reach it) — the cp14-proven discarded probe.
- **Thread.start** (P1 per-class slot): `t = new Thread(this); t.vtable[11]()`
  then returns (result discarded; the Runnable is Jlet base `a`, `run()` = game
  loop) ⇒ `vtable[11] = start()V`. Added to `known_java_lang_vtable`.

Result: `Thread.start` spawns **`a.run()` (the game loop) on a thread** — confirmed
dispatched (`native a.run()V code=0x1f10`).

### cp19 — the run-flag (next task)
`a.run()` body (`0x212c`): `obj = getInstance@0x1908(); if (obj.field[8] != 0)
loop else return`. RE of `0x1908`/`0x18ac`: it returns **class `a`'s descriptor
handle** `0x1400df4`, and the gate reads **`a`'s header word at +0x20** — a
class-level "running"/state flag, not an instance field. It is **0**, so `a.run()`
returns immediately and nothing renders. (Header `+0x20` is one of the descriptor
words documented as "mostly zero" / unconfirmed — likely a class static the AOT
stores in the header.) Next: find where this flag is set (a class-static write to
`0x1400da8+0x20`) and why it has not happened — likely a static-field-storage path
(the header-static mechanism, cf. cp17's instance-field work). Then `a.run()` enters
its loop and should `pushCard` / draw the title.

### Verified layout tables (cumulative, this arc)
- Global vtable: method-ref `r` → physical slot `r+1`, `offset[r]=r`, dispatch
  `vtable[offset[r]+1]` (reserved slot 0). Per-class overrides (physical/absolute):
  Runtime `freeMemory@13, gc@14`; StringBuffer `toString@5, append(String)@19`;
  **Thread `start@11`**.
- Instance fields: object slot = app-ancestor field count + declared index
  (inherited-first). e.g. Game object: `a`'s fields 0–3, Game's 4–8
  (`a.Display`→0, Game's `e:Le;`→8).

## Checkpoint 16 — full boot achieved; next gate = instance field_offsets

cp15 (reserved-slot-0) was a turning point: the app now **boots completely**, runs
`Game.<init>` → `a.startApp`, sets `Display.setCurrent(CardCanvas)`, and reaches the
wie MIDP **main event loop** driving `CardCanvas.paint(Graphics)` steadily with **no
crash**. The off-by-one that masked everything is gone (see cp15). This exposed the
app's *true* control flow and the next, precisely-located blocker.

### The "no title yet" cause (RE of `a.startApp`@`0x1ad8`)
`CardCanvas.paint` iterates `cards` (a Vector) and paints each; the app populates it
via `Display.getDefaultDisplay().pushCard(card)`. The app **never calls pushCard**, so
the canvas is empty. Why: `a.startApp` begins with a gate:
```
r3 = field_offsets[148]          ; 0x15006f4 + 0x128  (halfword idx 148)
ip = Game.field[r3]              ; read an instance field of `this`(Game)
if (ip != 0) goto 0x1d60         ; "already initialised" -> skip setup, run per-frame app.b
else        run app.a (0x1b58)   ; first run -> data load + card setup
```
`field_offsets[148]` is the resolved object-slot of an **instance field reference**.
Dumping the `fields` array: **`FIELDREF[148] = name "a", type "Lorg/kwis/msp/lcdui/Display;"`**
— i.e. the inherited `Display` field of app base class `a`. The gate is literally "is
my Display already set?". On first run it must read **null** so the setup branch runs.

But `install_platform_tables` only fills `field_offsets` for **static** fields (small
indices); **instance** field refs (like 148) are left **0**. So the read becomes
`Game.field[0]`, which in the AOT's own-fields-first layout is `Game`'s field 0
(`"a":Lj;`, set non-null by `<init>`) — *not* the Display field. The gate sees non-null,
wrongly takes the "already initialised" branch, **skips the data load + card setup**,
and falls straight through to the per-frame `app.b` and the (empty) event loop.

This is the long-deferred **cp3 item 4 (field unification)**: the app addresses
instance fields through `field_offsets[K]` (object slot of field-ref K), but the table
isn't populated for instance fields. A prior *blanket identity* fill regressed
`a.startApp` (noted in code), so the correct fix is **inheritance-aware**:
`field_offsets[K] = (sum of ancestor field counts) + declared_index` for field-ref K's
owning class, consistent with how the AOT lays out and writes the same fields.

### Verified mapping facts (ground truth, cp15/cp16 instrumentation)
- Global vtable is uniform: method-ref `r` → physical slot `r+1`; `offset[r]=r`;
  dispatch `vtable[offset[r]+1]=vtable[r+1]=ref r`. Confirmed for platform
  (`show`=ref6→slot7, `read`=ref7→slot8) **and** app (`b()V`=ref99→slot100,
  `a()V`=ref100→slot101) methods. So cp15 is correct for both; the pre-cp15 data-load
  was the off-by-one calling the wrong (similar) app method.
- `a.startApp` two dispatches: `@0x1b58` index 100 → `app.a` (data load, gated),
  `@0x1d68` index 99 → `app.b` (per-frame).
- `FIELDREF[148]` = inherited `a.Display`; the data-load gate.

### Next task (resume here — concrete, no external spec)
Populate `field_offsets` for **instance** field references with inheritance-aware
object slots (decode the per-class instance-field range in the import class table +
the app class field tables + parent chain). Then the gate reads the real (null)
Display field, runs the data load + `pushCard`, and `CardCanvas.paint` draws the app's
`o` Card → the title. Risk: must match the AOT field layout exactly (own-first vs
inherited-first) — verify by behaviour, not blanket identity.

## Checkpoint 14 — forward-probe past `r8`: the wall is Object's vtable layout

Since `r8`'s class is unrecoverable (cp13) but `r8.vtable[11]`'s result is *discarded*
(RE-confirmed), I ran a **temporary investigation** pass: return the discarded probe's
value instead of NPE-ing, to see whether `r8` is actually critical and to map the path
forward. (The pass was reverted — a blanket default is a disallowed no-op; it was only
to gather evidence.)

### Result: `r8` is NOT critical; the game keeps going
With `getClipWidth(r8)` returning a value (discarded anyway), execution continued well
past `r8`: it constructed more `Card`s, an `AnnunciatorComponent`, and began reading
files — i.e. `r8`'s `vtable[11]` is a genuine no-op-equivalent here. **`r8` is not a
real blocker**; it is one instance of the per-class-vtable collision pattern.

### The next stop is the SAME pattern on a *bound* object (tractable in principle)
```
LGT trampoline id=119 -> AnnunciatorComponent.<init>(Z)  this_raw=0x48840420   (bound OK)
LGT trampoline id=7   -> File.read([B)I  this_raw=0x48840420  this_actual=AnnunciatorComponent
  => NoSuchMethodError AnnunciatorComponent.read([B)I
```
The app `new`s an `AnnunciatorComponent` (static slot 18 = its `<init>`, correctly
bound), then calls `obj.vtable[idx]` where `idx` comes from the `.bss`
`virtual_method_offsets` table (`r2=[0x15009ac+0xc]`) and the call uses `vtable[r2+1]`.
That resolves to **global slot 7 = `File.read` (g7)**, but the object is an
`AnnunciatorComponent` whose own virtual is `show` (g6). Classic global-by-name
collision — `AnnunciatorComponent` needs a **per-class vtable** so its slot maps to its
own/inherited method, not `File.read`.

### The single fact that unblocks ALL of these: Object's vtable layout
Every collision (StringBuffer@19 [solved cp10 by behaviour], Runtime@13/14 [cp5], `r8`
slot 11, AnnunciatorComponent) is a per-class vtable whose index layout is
`[Object virtuals] ++ [intermediate platform parents] ++ [own virtuals]`. The one
missing constant is **`java/lang/Object`'s virtual-method count and order**:
- **Object vtable size ≈ 12 (slots 0–11)** — derived: cp5 saw `Runtime` (extends
  Object) own methods `freeMemory@13, gc@14` ⇒ `totalMemory@12` ⇒ Object occupies 0–11.
- **slot 5 = `toString`** — cp10: `StringBuffer.toString` is called at vtable index 5.
- **slot 11 = a *void* Object method** — `r8`'s `vtable[11]` result is discarded; with
  Object size 12, slot 11 is Object's last virtual (a void one — `wait`/`notify*`/
  `finalize`-class).

With Object's 12-slot layout (method→index) known, per-class vtables for `r8`,
`AnnunciatorComponent`, and the rest can be built **without guessing** (P1):
`[Object 0–11] ++ [parent imported methods at their java_load_classes indices] ++ [own
native code_ptrs]`, set on each object's `+0x00`. This is one bounded table, not a
per-class spec.

### Why STOP-B here (single fact, P3/P4/P5 exhausted)
- P3: re-RE'd repeatedly (cp11–14) — found the o-instance/Card chain, the thunk format,
  and now the forward path; the pattern is understood, not mysterious.
- P4: `r8`'s class is provably erased (cp13: stored in an int-declared reused slot).
- P5: the generic `0xf` allocator can't be type-intercepted; factories build fields.
- Forward-probe shows the blocker is **not** `r8` specifically but the **shared Object
  vtable layout** gating every per-class vtable. That single constant
  (`java/lang/Object` method→index, ~12 slots) is the precise external fact needed.
  Pinning slot 11 / AnnunciatorComponent's slot by guessing the method is the
  disallowed move.

### Evidence — observed `(class, vtable index)` needing Object's layout
| class | observed slot | what it should be | basis |
|---|---|---|---|
| `java/lang/StringBuffer` | 5 / 19 | `toString` / `append(String)` | solved cp10 (behaviour) |
| `java/lang/Runtime` | 13 / 14 | `freeMemory` / `gc` | solved cp5 (Object size 12) |
| `r8` (erased app class) | 11 | a void Object virtual (result discarded) | cp14 probe |
| `AnnunciatorComponent` | `r2+1`→7 | its own/inherited method (not `File.read`) | cp14 trace |

## Checkpoint 13 — identify `r8` by static type (P4): disproven, AOT erases the type

Per the directive, I exhausted P4 (a/b/c) + P5 to identify `r8`'s class from static
type info where it flows. Result: the type is **not in any reliable static position**.

### Where `r8` flows (full RE of `i.<init>`@`0x1c348`)
`r8 = [0x140452c]()` = import `0xf` (the class-agnostic allocator), then
`helper@0x1adc8(r8, this_i)` fills its fields, then it is stored and `vtable[11]`'d:
```
r5 = func@0xd8640()         ; getInstance-style: starts from o's handle 0x1403e08
                            ;   (= o header 0x1403dbc + 0x4c), tail-calls -> the o INSTANCE
str r8, [[r5+8] + 0x14]     ; r8 -> o-instance app-field array, offset 0x14 = index 5
if (r8 != null) r8.vtable[11]()   ; result DISCARDED
```
`o` = `class o`, which **extends `org/kwis/msp/lcdui/Card`** (the game's title Card).
So `r8` is stored as a field of the game's Card instance.

### P4(a) — owning field descriptor: **misleading (slot reuse)**
The store offset `0x14` = `o`'s **own app-declared field index 5**. `o`'s field table
(descriptor): index 5 = name `f`, **type `I` (int)**. But `r8` is an *object*. The AOT
**reuses the int-declared slot for an object reference** — native code is untyped, so
the Java field descriptor (`I`) does **not** type `r8`. P4(a) is not just unavailable
here, it is actively misleading.

### P4(b) — typed method argument: **none**
`r8` is only ever passed to (i) the untyped `helper@0x1adc8` (a compiler codegen helper,
no signature) and (ii) `vtable[11]` (the collision). It is never an argument to a
descriptor-typed app/platform method ⇒ no signature to read.

### P4(c) — field-fingerprint: **non-unique**
`helper@0x1adc8` writes `r8`'s field-array words 1–4 (results of imports `0xe`/`0x10`,
unknown return type). Matching against all 19 app descriptors: the only class with
*object* fields at indices 1–4 is `Game` (and `r8`≠`Game`, the Jlet); if `0xe`/`0x10`
return ints, the match is ambiguous across many classes. No unique identification.

### P5 — factory interception: **cannot mint `r8`**
`r8` comes from the generic import `0xf` allocator (no type). Imports `0xe`/`0x10` build
`r8`'s *fields*, not `r8`. So there is no typed factory to intercept for `r8` itself
(unlike cp10's String factory `0x9`).

### `vtable[11]` is a *void* method (decisive, theory confirmed)
Both branches of `if (r8!=null) r8.vtable[11]()` discard the result
(`bx ip` → method; next insn `ldr r0,[fp,#-0x2c]` overwrites `r0` with `this`). A
discarded call to `Graphics.getClipWidth()I` (an `int` getter, global ref `g11`) is
meaningless ⇒ `vtable[11]` for `r8`'s real class is a **void** method; the global
by-name table misroutes slot 11 to `getClipWidth`. `r8` is an app object whose
per-class vtable slot 11 is some inherited/own void virtual.

### Narrowed single-fact wall (B)
Everything is solved except: **`r8`'s class identity, which the AOT erased** (allocated
class-agnostically; stored into an int-declared, reused field slot; constructed by a
shared codegen helper; never passed to a typed method). The static-type strategy is
disproven for this object. The minimal external fact that would unblock:
> For `org/kwis/msp/lcdui/Card`'s subclass hierarchy, **what method is at per-class
> vtable index 11** (a void method on the game's Card-field object), and/or a way to
> bind the natively-`new`'d object stored at Card-instance field `f`. Equivalently:
> the Card/Component/Object vtable layout for the single slot 11.

This is one slot, not a full layout — but it is genuinely not derivable from the app's
own data (the type is erased at every observable point). Reporting per the STOP-B rule
after exhausting P3+P4+P5.

### Reference data gathered this checkpoint
- Import-thunk format `[str lr,[sp,#-4]! | bl disp | .word table=0x64 | .word index]`:
  `0x140452c`→`0xf` (alloc), `0x140451c`→`0xe`, `0x140453c`→`0x10`, `0x140466c`→`0x54`.
- `func@0xd8640`/`0xd85e4` = the `o`-singleton (Card) getInstance accessor
  (`o` handle `0x1403e08`).
- `class o` extends `org/kwis/msp/lcdui/Card`; `o`'s field 5 = `f:I` (reused for `r8`).

## Checkpoint 12 — P3 re-attack on the cp11 wall: identify `r8`'s class

Per the cp9→cp10 precedent (STOP-B was premature), I re-RE'd the cp11 blocker to try
to **identify** `r8`'s class empirically (no guess binding). The dig produced a lot of
new ground but the specific identity is **compiled away** — refined STOP-B below.

### New ground (all confirmed)
**Import thunk format.** Each `.data` import slot is a 16-byte thunk
`[0xe52de004 = str lr,[sp,#-4]!] [bl dispatcher] [.word table] [.word index]`. Reading
the table/index words decodes every slot:
| slot | table | index | = |
|---|---|---|---|
| `0x140452c` | `0x64` | `0xf`  | native allocator (`new`) |
| `0x140451c` | `0x64` | `0xe`  | factory A (builds `r8` fields) |
| `0x140453c` | `0x64` | `0x10` | factory B (builds `r8` fields) |
| `0x140466c` | `0x64` | `0x54` | per-method entry helper (stubbed no-op) |
| `0x14045fc` | `0x64` | `0x22` | (an object-finalise/`0x21` pair helper) |

**`i.<init>`@`0x1c348` → `r8` provenance.** `r8 = [0x140452c]()` = **import `0xf`**, the
**class-agnostic** allocator: at all three observed call sites (`i.<init>`,
StringBuffer, here) its `r0` is leftover from the prior call — it takes **no class/size
argument**. So `r8` carries no type. `helper@0x1adc8` then fills `r8`'s fields via
imports `0xe`/`0x10` (currently no-op), and `i.<init>` does
`if (r8!=null) r8.vtable[11]()`.

**`r8` is an app object, not a Graphics (decisive).** Global vtable slot 11 = Graphics
`getClipWidth()I` (an `int` getter — see the imported-class table below). But the call
site **discards the result** (`bx ip` → `getClipWidth`; the next insn
`ldr r0,[fp,#-0x2c]` overwrites `r0`). A discarded `int` getter is meaningless ⇒
`r8`'s **real** class has a **void** method at vtable index 11, and the global by-name
table misroutes it to `getClipWidth` — exactly the StringBuffer@19 collision class.
So `r8` needs a **per-class vtable**, and the slot-11 method is some inherited/own
**void** virtual of `r8`'s class.

### Why the class is not empirically recoverable (refined STOP-B)
- The allocator is class-agnostic (proven), so there is no type tag at `new`.
- The factories `0xe`/`0x10` build `r8`'s *fields*, not `r8`; they don't name its class.
- App `<init>`/constructors are **raw-native codegen** (`helper@0x1adc8` is a compiler
  helper, not a Java method), invisible to the bridge — no observation point.
- Back-deriving the class from the observed *void slot 11* requires the **platform
  base-class vtable layout** (Object/Component/Card slot counts): the app classes
  extend `Card`/`Component`, and index 11 in *their* per-class vtable maps to a method
  only if that base layout is known. This is the **same external spec cp9 flagged** —
  now proven to also gate app-object virtual dispatch (cp11 anticipated this; cp12
  proves it with the void-slot-11 evidence). Guessing the layout / the class is the
  disallowed move.

### The precise maintainer question
Provide (or RE from the LGT/ez-i runtime) the **vtable layout of the platform base
classes** `java/lang/Object`, `org/kwis/msp/lwc/Component`, `org/kwis/msp/lcdui/Card`
(method → index). With it: (1) hardcoded indices like `vtable[11]` on app objects
resolve to the right method, and (2) per-class **native** vtables (ARM code pointers)
can be built for app objects so app→app/app→self virtual dispatch needs no JVM
round-trip or runtime class tag. Absent it, only the global-ref slots that happen to
coincide (lcdui-hierarchy inherited methods) work — app-specific indices collide.

### Reference — imported platform classes (from `java_load_classes`, cp12 dump)
Global virtual refs `gN` are the global-vtable slots; getClipWidth = `g11`.
```
[4]  org/kwis/msp/lwc/Component   virt: g1 getHeight
[5]  org/kwis/msp/lcdui/Card      virt: g2 serviceRepaints, g3 repaint, g4 getHeight, g5 getWidth
[12] org/kwis/msp/lcdui/Graphics  virt: g10 getClipHeight, g11 getClipWidth, g12 getClipY,
       g13 getClipX, g14 drawLine, g15 drawRect, g16 drawImage, g17 setColor, g18 fillRect,
       g19 setXORMode, g20 setColor, g21 getColor, g22 setClip
[20] org/kwis/msp/lcdui/Display   virt: g23 pushCard, g24 removeAllCards
[23] org/kwis/msp/lcdui/Jlet      virt: g25 notifyDestroyed
[26] org/kwis/msp/lcdui/Image     virt: g26 getGraphics, g27 getHeight, g28 getWidth
```
The app `new`s `r8` and calls `vtable[11]` on it — but `r8` is not in the lcdui import
hierarchy at that slot (its slot 11 is a void method), so the global table is wrong for it.

## Checkpoint 11 — native-instantiated platform object → STOP (condition B)

After cp10 the game runs its data-load loop and reaches `Graphics` setup, then stops:
```
LGT UNBOUND this for org/kwis/msp/lcdui/Graphics.getClipWidth:
    this_raw=0x48840400 pending_new=true vtable_word=0x4d85a000 (global=0x4d85a000)
java/lang/NullPointerException: getClipWidth   (at Game.a -> i.<init>)
```

### Airtight diagnosis (RE of `i.<init>`@`0x1c348`, helper@`0x1adc8`)
`i.<init>` does `r8 = new(); helper@0x1adc8(r8, …); if (r8!=null) r8.vtable[11]()`
(offset 0x2c → slot 11 → `getClipWidth`; the result is discarded — a null-guarded
virtual call). Instrumentation confirms `r8` (`0x48840400`):
- **`pending_new = true`** — it was produced by the native allocator (stdlib `0x32` /
  java `0xf`, both → `alloc_native_object`) and **never bound to a JVM class**.
- **`vtable_word = global`** — so `vtable[11]` resolves through the global
  by-name table to `Graphics.getClipWidth`, invoked on an unregistered `this` → NPE.

`helper@0x1adc8` is a **compiler codegen helper** (not a Java `<init>`): it fills
`r8`'s fields with sub-objects from two platform factories (import slots
`0x140451c`, `0x140453c`). So the app constructs `r8` **entirely in native code** —
the platform `<init>` trampoline (which `bind_pending` hooks) never fires.

### Why this is a structural wall (B), not an empirical one
- The native allocator is **class-agnostic**: its `r0` is leftover from the prior
  call (verified in both the StringBuffer and `i.<init>` sites), so the object's
  class identity is **not present at allocation**.
- App `<init>`/constructors run as **raw native ARM**, invisible to the bridge —
  there is no observation point to learn an object's class. (Game and the Cards were
  bindable only because they go through the JVM / a platform `<init>` trampoline.)
- `getClipWidth` is meaningful only on a real **Graphics with a backing**. The app
  expects native `new` + native init to yield a working platform Graphics; wie's
  Graphics needs proper construction (Image/screen backing). Binding `r8` to any
  class by guesswork is exactly the disallowed move (추측/블랭킷 → divergence).

Resolving this needs a **maintainer/design decision on the native-object model**:
how a natively-`new`'d platform object (e.g. Graphics) is recognised and bound to a
wie instance with the correct backing — e.g. (a) make the LGT allocator carry/record
a class tag, (b) a per-class native-vtable of ARM code pointers for app→app/app→self
dispatch (needs the app/platform vtable layout — the same external dependency cp9
flagged), or (c) intercept the specific platform factory imports
(`0x140451c`/`0x140453c`) to mint bound objects. Each is a structural change, not a
single empirical function.

### Evidence table — `(class, vtable index)` / native-object binding
| site | observation | basis | status |
|---|---|---|---|
| `Graphics.getClipWidth` | called on `pending_new` `r8`, global vtable slot 11 | runtime warn + `i.<init>` disasm | ❌ unbound native object |
| native allocator (`0x32`/`0xf`) | class-agnostic (no class/size arg) | call-site RE (StringBuffer + `i.<init>`) | confirmed |
| app `<init>` (`helper@0x1adc8`) | raw native codegen helper, builds composite | disasm `0x1adc8` | confirmed |

확정 / 추정 / 미해결:
- **확정**: `r8` is a native-allocated, never-bound object; the global by-name vtable
  cannot serve it; the allocator carries no class identity; app constructors are
  invisible to the bridge.
- **추정**: `r8` is intended to be a platform `Graphics` (getClipWidth target) the app
  `new`s and inits natively; many more such native-instantiated objects likely follow.
- **미해결 (needs maintainer / design)**: the native-object model for natively-`new`'d
  platform objects (class binding + correct vtable/backing). Also still pending from
  cp9: platform per-class vtable spec, app field unification (cp3 item 4).

## Checkpoint 10 — StringBuffer wall CROSSED (supersedes cp9 STOP-B)

Re-ran the cp9 RE (playbook P3) and found the STOP-B conclusion was premature. Both
"walls" were empirically solvable, no external spec or guessing required.

### What the AOT actually does (RE of `0x4720`, `0x1834`)
`Game.a()` builds resource filenames `"txt/" + arg + ".dat"` via StringBuffer:
```
r6 = new()                      ; allocator import @0x140452c
StringBuffer.<init>(r6)         ; .bss table [r4+0x160], trampoline id=189
s  = String factory(const[26])  ; func@0x1834 -> import 0x9; const[26]="txt/"
r6.vtable[19](r6, s)            ; offset 0x4c -> append(String), chained x3
r6.vtable[5]()                  ; offset 0x14 -> toString() -> String
```
- **vtable[19] = `append(String)`**: behaviour-confirmed — the arg is a String from
  the constant pool, the result is re-`append`ed twice (builder chain), then
  `toString`'d. Index is StringBuffer's *own* class-vtable slot, not a global ref.
- **vtable[5] = `toString()`**: the result is read as a String.
- **`func@0x1834`** = string-constant loader: reads `const[idx]={len:u16, u16 chars}`
  and calls **java-interface import `0x9`** = a native **String factory**
  `(ctx, char_ptr, count, out_slot)`. Identified from the import-resolution log:
  `0x9(0x1400154, 0xe7512, 4, …)` with char data matching the pool ("txt/").

### Fix (three small, evidence-grounded pieces)
1. **String factory** (`interface.rs`): java-interface imports now route by
   `function_index` through `SVC_CATEGORY_JAVA_INTERFACE` (the SVC id *is* the index),
   so each keeps its identity. Import `0x9` reads the UTF-16 chars, builds a
   `java/lang/String`, and registers it behind a guest proxy
   (`register_platform_object`) so it round-trips back to the JVM String when used as
   an argument. The "native String isn't a JVM object" wall dissolves: the factory
   *is* where native Strings are born, so it just makes JVM ones.
2. **Per-class StringBuffer vtable** (`known_java_lang_vtable`): slot 19 →
   `append(String)`, slot 5 → `toString()`. `bind_pending` now rewrites the guest
   object's `+0x00` vtable word to the per-class vtable at `<init>` time (the native
   allocator set the global one before the class was known).
3. **Synthetic `append(String)`** (`handle_java_trampoline`): wie's StringBuffer has
   `append([CII)` but not `append(String)`/`append(Object)`; the trampoline
   special-cases it, reading the String's chars and forwarding to `append([CII)`
   (no shared-class edits — rule-compliant). `append(null)` appends "null".

### Result
`new StringBuffer(); …append…toString()` now produces real filenames —
`txt/mon_info.dat`, `txt/SUB_QST_INFO.dat`, `txt/upgrade_attr.dat`,
`txt/mon_attr_init.dat`, `txt/gradePoint.dat`, … — i.e. the game's data-load loop
runs. Execution advances well past the cp9 stop into `Graphics` setup.

### New stop (next checkpoint, NOT a wall)
`org/kwis/msp/lcdui/Graphics.getClipWidth()I` with `this_actual=None`. The `this`
(`r8`) is a fresh object from the allocator import `0x140452c` (java `0xf` `new`)
that is used as a Graphics without a platform `<init>` binding it, so it is not in the
instance map. RE of `0x1c604`/`0x1adc8` (what `new`s it and what `func@0x1adc8`
initialises) is the next step — same empirical loop, no external input expected.

## Checkpoint 9 — per-class platform vtables: investigation → STOP (B)  [SUPERSEDED]

The dominant blocker is per-class vtables for platform classes the AOT calls by a
hardcoded vtable index but which `java_load_classes` does not list (StringBuffer,
java/lang/{Object,Runtime,System}). I investigated whether they can be reconstructed
without the platform spec.

### Method enumeration is available
`java_runtime::loader::get_runtime_class_proto(name)` (public) + `wie_wipi_java`/
`wie_midp::get_protos()` expose every platform class's method list — so a
**standard-order** vtable (`[reserved slot 0] ++ [ancestor virtuals] ++ [own
virtuals]`, proto declaration order, with override dedup) can be built.

### Validation result: standard order does NOT generalize
- **Runtime — matches (coincidentally).** Object has 11 virtual methods; with a
  reserved slot 0 its vtable size is 12. Runtime (extends Object) own virtuals are
  `totalMemory, freeMemory, gc` → slots 12, 13, 14. This matches the cp5 observation
  (`freeMemory@13`, `gc@14`). ✅
- **StringBuffer — disproven.** Disassembly of `new StringBuffer(); sb.append(...)`
  (`0x4740`) shows the native calls `stringBuffer.vtable[19](this, arg)`. The arg
  comes from `0x1834`, which is a **string-constant loader** (reads the pool at
  `0x140019c`), so `vtable[19] = append(Ljava/lang/String;)Ljava/lang/StringBuffer;`
  (behaviour-confirmed). But in wie's `StringBuffer`, `append(String)` is the **first
  own virtual** → standard-order vtable index **12**, not 19. **So wie's method order
  ≠ the AOT's order**; the Runtime match was a 3-method coincidence.

→ The per-class vtable **order** for platform classes is **not derivable** from the
app data or wie's protos. It needs the original **LGT/ez-i platform vtable-index
spec** (external). Per-slot empirical RE (cp5-style) can pin *individual* observed
slots without guessing, but is not a general solution (each class has many slots).

### A second, compounding wall: native objects aren't JVM objects
Even the one confirmed slot can't be exercised: `vtable[19] = append(String)`'s
argument is a **native** String produced by the string-constant loader (`0x1834`),
which is **not** registered as a JVM object. Marshalling it would yield `null`. So
StringBuffer (and string constants, and other natively-created objects) need a
general **native↔JVM object/String bridge** (read native object state → JVM
instance) — substantial new infrastructure, beyond a single function.

### Why STOP (condition B)
Clean, no-guess forward progress to `paint` is blocked on **two large items that
need information/infrastructure beyond the app**: (1) the external per-class
vtable-index spec for platform classes (wie order disproven), and (2) a native↔JVM
object/String bridge. The only remaining "advance" would be risky guessing of
vtable orders — explicitly disallowed (divergence/regression already seen at cp7/cp8
with blanket fills). Reporting for maintainer input per the autopilot stop rule.

### Evidence table — `(class, vtable index)` the AOT calls vs. derivability

| class | vtable idx | method (confirmed/inferred) | basis | wie standard-order idx | status |
|---|---|---|---|---|---|
| `java/lang/Runtime` | 13 | `freeMemory()J` | cp5 usage | 13 (Object=12 +1) | ✅ matches |
| `java/lang/Runtime` | 14 | `gc()V` | cp5 usage | 14 | ✅ matches |
| `java/lang/StringBuffer` | 19 | `append(Ljava/lang/String;)` | `0x1834` = string-const loader; arg is a String; result chained | 12 | ❌ order mismatch |
| platform String args | — | native String, not a JVM object | `0x1834` returns a native obj | — | ❌ needs bridge |

확정 / 추정 / 미해결:
- **확정**: method enumeration is available; Runtime standard-order matches; for
  StringBuffer the AOT order (`append(String)@19`) ≠ wie order (`@12`);
  `vtable[19]=append(String)` (behaviour-confirmed via the string-constant loader).
- **추정**: other platform classes likely also diverge from wie order (StringBuffer
  shows it is not reliable).
- **미해결 (needs maintainer / external spec)**: per-class vtable-index layout for
  platform classes; native↔JVM object/String bridge; (also still pending: app field
  unification cp3 item 4, Blocker A Jlet Component methods).

## Reproduce (current, post-cp10)
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# Game.a() -> data-load loop: new StringBuffer().append("txt/").append(name)
#   .append(".dat").toString() now builds real filenames (txt/mon_info.dat, ...).
# Then i.<init> -> r8 = new(); native init; r8.getClipWidth() with r8 unbound
#   (pending_new) -> NullPointerException. See cp11 "LGT UNBOUND this" warn.
```

## Recommended next steps (need a decision / external input)
The cp9 items below were partly resolved by cp10 and reframed by cp11:

1. **Platform base-class vtable layout (cp12 — the live blocker).** `r8` is an app
   object whose hardcoded `vtable[11]` (a *void* method) collides with the global
   by-name table's `getClipWidth`. Its class isn't tagged at `new` (the allocator is
   class-agnostic — proven) and is compiled away. To resolve, provide the vtable layout
   (method→index) of `java/lang/Object`, `org/kwis/msp/lwc/Component`,
   `org/kwis/msp/lcdui/Card`; then app objects can get **per-class native vtables** (ARM
   code pointers) so `vtable[11]` dispatches correctly without a runtime class tag or
   JVM round-trip. (Factories `0xe`/`0x10` build `r8`'s *fields*, not `r8`, so
   intercepting them — cp11 option c — does not mint `r8`.)
2. **Native↔JVM String bridge — DONE (cp10).** The native String factory is
   java-interface import `0x9`; it now mints real `java/lang/String`s. (Generalising
   to other native objects is subsumed by item 1.)
3. **StringBuffer per-class vtable — DONE (cp10).** Slot 19 = `append(String)` (synth
   via `append([CII)`), slot 5 = `toString()`; object vtable rebound at `<init>`. The
   cp9 "order not derivable" worry was moot: the index is the per-class slot, pinned
   empirically per playbook P1 — no full-layout spec was needed for the observed slots.
4. Platform per-class vtable spec (cp9) is still the general fallback for *other*
   classes; app field unification (cp3 item 4) and Blocker A remain pending.
