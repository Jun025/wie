# STEP report — LGT native-backed JVM (checkpoints 1–6)

Goal: run the AOT-compiled LGT app (BattleMonster `00025C2B`) on wie's JVM, toward
`startApp` → `paint(Graphics)` (title). Branch `feat/lgt-java-interface-bridge`
(local only). PoC `LgtJvmShared` kept LGT-specific per Discussion #1232; shared
`wie_wipi_java`/`wie_midp` classes are **not** modified.

## Status summary

| item | result |
|---|---|
| cp1–2: app classes registered; methods run as real ARM | ✅ |
| cp3: `java_load_classes` tables; native↔platform bridge | ✅ |
| cp4–5: per-class platform vtables; Runtime vtable wall crossed | ✅ |
| **cp6: two-level virtual dispatch (the a.startApp fix)** | ✅ implemented + validated |
| reach | `a.startApp` → `Game.a()` **native ARM**; deep into app logic |
| next blocker | stdlib import `0x32` (50) requested by `Game.a()` |
| `paint`/title | ❌ not yet (blocked behind the stdlib tail) |
| clet regression (`test_helloworld`) | ✅ | clippy | ✅ |

cp1–5 are in the git history; this revision covers **checkpoint 6**.

## Checkpoint 6 — two-level virtual dispatch

### Re-RE of the methodref index space (work item 2)

From the `.bss` table layout (addresses from the `java_load_classes` args):
- `static_method_offsets` `0x1500820`: **99 × u32** (direct function pointers).
- `virtual_method_offsets` `0x15009ac`: **102 × halfword** (an INDEX table).
- `field_offsets` `0x15006f4`: **150 × halfword**.

The `virtual_methods` INPUT array is **larger than the 29 I'd parsed**: refs **0–28
= platform methods** (the imported classes), refs **29–100 = the app's own virtual
methods** (obfuscated names `b,c,…`), e.g. ref 100 = app `a()V`. So the methodref
space spans platform + app virtual methods.

### Model (decoded in cp5, implemented here)

The AOT dispatches virtuals two ways:
```
idx = virtual_method_offsets[ref]; obj.vtable[idx]()   (indirect, ref baked)
obj.vtable[hardcoded]()                                (direct, index baked)
```
`virtual_method_offsets` is a **halfword index table**, `obj+0x00` is a **separate
pointer vtable**. cp3–4 wrote pointers into the offset table and pointed `obj+0`
there — which served direct calls but fed garbage indices to indirect calls (the
`a.startApp` hang).

### Implementation (`install_platform_tables`)

- **One global pointer vtable**: `slot[ref] = trampoline` that `invoke_virtual`s
  `virtual_methods[ref]` **by name** on `this`. Because dispatch is by name, this
  single vtable serves *every* object: a platform proxy → the wie method; an app
  object → its native ARM method. Every object's `+0x00` points here.
- `virtual_method_offsets[ref*2] = ref` (halfword **identity** — the vtable index is
  the method's global array position, which the direct hardcoded indices also use:
  `drawLine@14`, `getHeight@1/4/27`).
- `static_method_offsets[i*4]` = direct function pointer (unchanged).
- **java/lang override**: classes the AOT calls by a hardcoded index that collides
  with another class's slot (Runtime 13/14) get a per-class vtable = a copy of the
  global one with the [`known_java_lang_vtable`] slots overridden.

### Validation (trace — indirect & direct both consistent)
```
a.startApp([Ljava/lang/String;)V               [native ARM]
  trampoline ref100 -> app.a()V, this=Game      [indirect: virtual_method_offsets[100]=100
                                                  -> global_vtable[100] -> invoke_virtual(Game,"a")]
  dispatch -> native Game.a()V @0x11dc           [resolved to the app's own ARM method ✓]
```
Direct calls still resolve (`Runtime.gc/freeMemory`@13/14, `Display.getDefaultDisplay`,
…). The `a.startApp` hang is gone; boot now runs the app's real logic.

## Reach & next blocker

`Game.a()` (native) constructs app objects (`e`, `j`, …), stores them into its fields,
and — via a `.data` lazy-bind trampoline — requests **C stdlib import `0x32` (50)**:
```
dispatch -> native Game.a()V @0x11dc
get_import_table(0x1); get_import_function(0x1, 50)
=> Unknown lgt stdlib import: 0x32
```
`0x32` is far below the known WIPI stdlib range (`0x3f6`–`0x424`: `strlen`, `memcpy`,
`memset`, …), so its identity isn't established. Per the task, it is **not** stubbed
blindly (a wrong C-function stub diverges). This is the next, incremental blocker —
a missing native-library function, not a structural issue.

(`Component.getHeight` on the Jlet, the cp5 "Blocker A", is **not** reached on this
path yet — `a.startApp` goes through `Game.a()` first.)

확정 / 추정 / 미해결:
- **확정**: two-level dispatch model; methodref space (102 virtual refs, 0–28
  platform + 29–100 app); the global-by-name vtable serves all objects.
- **추정**: `Runtime` 13/14 = `freeMemory`/`gc` (from cp5; still valid).
- **미해결**: stdlib import `0x32`; full `java/lang/{Object,System}` vtable layout;
  Blocker A (`Jlet` Component methods); field-storage unification (cp3 item 4).

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# trace: a.startApp -> ref100 app.a() -> Game.a() native -> get_import_function(0x1, 50)
# -> Unknown lgt stdlib import: 0x32.
```

## Remaining work (incremental)
1. Identify/implement stdlib import `0x32` (and the rest of the low-index stdlib
   tail) requested by the app's native code.
2. Blocker A: `Jlet` Component methods (`getHeight`/`getWidth`) — handle in the LGT
   PoC layer (synthetic methods on app classes / delegation), without touching
   shared classes.
3. Field-storage unification (cp3 item 4).
4. `java/lang/{Object,System}` vtable slots as they appear (extend
   `known_java_lang_vtable`).
With these, expect `a.startApp` → Card `o` → `paint(Graphics)` toward the title.
