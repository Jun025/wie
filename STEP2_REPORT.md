# STEP report — LGT native-backed JVM (checkpoints 1–4)

Goal: run the AOT-compiled LGT app (BattleMonster `00025C2B`) on wie's JVM —
register its native classes, dispatch its methods to real ARM, and bridge calls
into the platform classes, toward `startApp` → `paint(Graphics)` (title screen).

Branch `feat/lgt-java-interface-bridge` (local only). PoC `LgtJvmShared` kept
LGT-specific per Discussion #1232. RE basis: `docs/lgt_native_classes.md`.

## Status summary

| item | result |
|---|---|
| `NoClassDefFoundError` resolved; app methods run as real ARM | ✅ (cp1–2) |
| `java_load_classes` fills the platform method/field tables | ✅ (cp3) |
| native → platform static/special dispatch (by name) | ✅ (cp3) |
| native → platform **virtual** dispatch via **per-class vtables** | ✅ implemented (cp4) |
| reach | ⏹ clean null at `Runtime.vtable[14]` — an un-imported `java/lang` virtual method |
| `paint(Graphics)` / title screen | ❌ blocked on the platform vtable-index spec |
| clet regression (`test_helloworld`) | ✅ | clippy | ✅ |

cp1–3 are summarized in the git history; this revision focuses on **checkpoint 4**.

## Checkpoint 4 — per-class platform vtables

### Investigation (the vtable-index ordering source)

The AOT virtual-dispatches platform methods as `r3 = [this]; bx [r3 + idx*4]`
(read the object's vtable pointer at `+0x00`, branch through a fixed index). The
question was whether `idx` indexes (A) the `java_load_classes` virtual-method
array position, (B) a standard inheritance-ordered vtable, or (C) something else.

Evidence gathered (logging each trampoline's intended method **and the actual
class of `this`**):

```
trampoline -> Jlet.<init>()V                 this_actual = Game            ✓ (Game is-a Jlet)
trampoline -> BackLight.alwaysOn()V          this_actual = None (static)   ✓
trampoline -> Runtime.getRuntime()...        this_actual = None (static)   ✓
trampoline -> Graphics.drawLine(IIII)V       this_actual = java/lang/Runtime  ✗
trampoline -> Graphics.getClipX()I           this_actual = java/lang/Runtime  ✗
trampoline -> Component.getHeight()I         this_actual = Game               ✗
```

The `virtual_methods` array is laid out **per class**, each class's methods at
distinct positions, and the same name recurs at different positions for different
classes:

```
Component.getHeight @ 1     Card.{serviceRepaints@2, repaint@3, getHeight@4, getWidth@5}
Graphics.{getClipHeight@10 … drawLine@14 … setClip@22}    Display.{pushCard@23, removeAllCards@24}
Jlet.notifyDestroyed @ 25   Image.{getGraphics@26, getHeight@27, getWidth@28}
```

So `getHeight` is at array index 1 (Component) **and** 4 (Card) **and** 27 (Image):
the index is a **per-class vtable index**, not a global one. With a single shared
table, `Runtime.getRuntime().<idx 14>()` resolved to `Graphics.drawLine` (the
occupant of global slot 14) and was invoked on a `Runtime` — confirming the
conflation. **Conclusion (확정): per-class vtables are required.**

### Implementation

`java_load_classes` (`install_platform_tables`) now builds, for **every** imported
platform class, a zeroed guest vtable (`VTABLE_WORDS`) and places that class's own
virtual methods at their global indices. A **platform proxy** object's `+0x00`
points to its class's vtable (`class_vtables[name]`); **app** objects keep the
union global table (they extend the lcdui hierarchy, so their dispatch spans many
imported classes' methods). Static/special calls keep the by-name offset-table
path from cp3. The no-op/0 fallback was removed (it diverges — `getHeight()→0`
loops the layout).

Effect: a `Runtime` proxy now carries its **own** (empty) vtable, so
`[runtime+0 + 0x38]` reads **0** — a clean null at the unresolvable index — instead
of misfiring onto `drawLine`.

### Reach
```
new Game -> Game.<init> [real ARM]
  -> trampoline Jlet.<init>()V (this=Game)         [real platform Jlet ctor]
  -> trampoline BackLight.alwaysOn (static)
  -> trampoline Runtime.getRuntime() (static) -> Runtime proxy
  -> [runtime.vtable + 0x38] == 0  => clean null  (Invalid memory access; address 0)
```

### Unresolved (미해결) — needs the platform vtable-index spec

The next call is `Runtime.getRuntime().<vtable index 14>()` — a **`java/lang/Runtime`
virtual method** (behaviorally a startup memory query: `freeMemory`/`totalMemory`).
`Runtime`, `System`, and `Object` declare **0 virtual methods** in the app's import
tables (`freeMemory`/`totalMemory`/`gc` do not appear anywhere in `virtual_methods`),
yet the AOT calls them by hardcoded vtable index. Their vtable layout therefore
**cannot be derived from the app's data** — it requires the original LGT platform's
per-class vtable-index assignment for the `java/lang/*` (and base `Object`)
classes.

Per the task's honesty requirement, this is left as a clean, loud failure (no
forced advance). Evidence for the maintainer — `(class, vtable index)` pairs the
app calls but the import data cannot resolve:

| object class | vtable index (byte off) | likely method | in import data? |
|---|---|---|---|
| `java/lang/Runtime` | 14 (`0x38`) | `freeMemory`/`totalMemory` | no (Runtime vmc=0) |
| `java/lang/System`/Object | (reached after the above) | — | no |

Imported lcdui-hierarchy classes (`Graphics`, `Card`, `Display`, `Image`,
`Component`, `Jlet`) **are** resolvable by index (their methods are in
`virtual_methods` at the indices the AOT uses, e.g. `drawLine@14`); they are on the
`paint` path and should work once execution gets past the `java/lang` wall.

확정 / 추정 / 미해결:
- **확정**: per-class vtable indices (not global); the lcdui classes' indices match
  their `virtual_methods` array positions.
- **추정**: `Runtime.vtable[14]` is `freeMemory`/`totalMemory` (startup memory query).
- **미해결**: the vtable layout (index→method) for `java/lang/{Object,Runtime,System}`
  — absent from app data; needs the platform spec.

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# trace shows the per-class vtable dispatch and the clean null at Runtime.vtable[14].
```

## Module layout
- `native_class.rs` — read-only descriptor parser.
- `native_jvm.rs` — ARM-backed objects, native↔platform bridge, trampolines,
  `java_load_classes` table + **per-class vtable** construction, class registration.
- `init.rs` / `interface.rs` / `svc_ids.rs` — wiring, java-interface imports.

## Remaining work (beyond this pass)
1. Obtain/define the `java/lang/{Object,Runtime,System}` vtable-index layout (the
   WIPI/ez-i platform spec) and fill those per-class vtables.
2. Field-storage unification (cp3 item 4) — JVM `get/put_field` ↔ the guest field
   array — so native-written and JVM-read fields agree on shared fields.
3. With both, expect `startApp` → Card `o` → `paint(Graphics)` toward the title.
