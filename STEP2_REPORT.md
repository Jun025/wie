# STEP report — LGT native-backed JVM (checkpoints 1–5)

Goal: run the AOT-compiled LGT app (BattleMonster `00025C2B`) on wie's JVM, toward
`startApp` → `paint(Graphics)` (title screen). Branch `feat/lgt-java-interface-bridge`
(local only). PoC `LgtJvmShared` kept LGT-specific per Discussion #1232.

## Status summary

| item | result |
|---|---|
| cp1–2: app classes registered; methods run as real ARM | ✅ |
| cp3: `java_load_classes` fills tables; native↔platform bridge | ✅ |
| cp4: per-class platform vtables (proxies) | ✅ |
| cp5: **Runtime vtable wall crossed** → `a.startApp` reached as real ARM | ✅ |
| `paint(Graphics)` / title | ❌ — two newly-localized blockers below |
| clet regression (`test_helloworld`) | ✅ | clippy | ✅ |

cp1–4 are in the git history; this revision covers **checkpoint 5** (the `java/lang`
vtable investigation) and the two blockers it exposed.

## Checkpoint 5 — the `java/lang` vtable wall

### Investigation (ordering source)

1. **Dependency spec (cheapest):** searched `wipi`/`wipi_types`. **No `java/lang`
   vtable-index layout exists** — the reference invokes platform methods **by name**
   (`java_invoke_special(c"…", c".()V+<init>", …)`), not by index. So BattleMonster's
   AOT (ez-i) baked indices that aren't recorded in the dependency. → unavailable.
2. **Reconstruct from wie:** wie's `java/lang/Object` has 11 virtual methods; the LGT
   platform's must have ~13 (Runtime's methods are called at indices 13/14). The
   counts don't match, so wie's order can't be used directly.
3. **Empirical (decisive for the method):** disassembling `Game.<init>`:
   - `0x5538`: `getRuntime().<vtable 14>()` — **result discarded** ⇒ a void method ⇒ `gc()`.
   - `0x1144`: `getRuntime().<vtable 13>()` — **result used** (passed to `0xdb8f0`) ⇒ a
     value-returning memory query ⇒ `freeMemory()`.
   This is the classic `getRuntime().gc(); … freeMemory()` startup memory check.

### Implementation (추정, evidence-grounded)

`known_java_lang_vtable()` places the empirically-identified slots into a `java/lang`
class's per-class vtable — currently `Runtime{13: freeMemory()J, 14: gc()V}`. These
are **estimates** at the **observed** indices, not a derived spec.

### Result
```
Game.<init> -> Jlet.<init> (real) -> BackLight.alwaysOn
  -> getRuntime().gc()        [vtable 14, void]
  -> getRuntime().freeMemory()[vtable 13, value used]   ← startup memory check passes
-> a.startApp([Ljava/lang/String;)V  [real ARM]
```
The Runtime vtable wall (the pass's primary objective) is **crossed**; `a.startApp`
now runs as real ARM.

## Two blockers exposed inside `a.startApp` (next work)

### Blocker A — wie/LGT class-hierarchy gap (`Jlet.getHeight`)
`a.startApp` calls `Component.getHeight()I` on the `Game`(Jlet) object. In the LGT
platform the Jlet is a Component/Canvas (has `getHeight`); in wie `Jlet → Object`,
so `invoke_virtual` fails: `NoSuchMethodError: Game.getHeight`. A one-line
delegating `Jlet.getHeight`/`getWidth` resolves it (tried, then reverted to avoid a
speculative shared-class edit and because it then hits Blocker B). This is a small,
tractable platform-hierarchy reconciliation.

### Blocker B — virtual dispatch is **two-level** (model correction, 확정 via disasm)
The biggest finding: my cp3/cp4 virtual model is structurally wrong. In `a.startApp`:
```
r3 = 0x15009ac (virtual_method_offsets)
ldrsh r2, [r3, #0xc8]      ; r2 = virtual_method_offsets[ref]  -- a HALFWORD index
r3 = [sb]                  ; r3 = [this+0]  -- the object's own vtable (pointers)
add r3, r3, r2, lsl #2     ; &vtable[idx]
ldr ip, [r3, #4]; bx ip    ; call obj.vtable[idx]
```
So the AOT does **two-level** virtual dispatch:
`idx = virtual_method_offsets[methodref]` (a **halfword index table**), then
`obj.vtable[idx]()` where `obj+0` is a **separate pointer vtable**.

My code instead wrote 4-byte *pointers* into `virtual_method_offsets` and pointed
`obj+0` at that same table. This happens to serve **direct hardcoded-index** calls
(`obj.vtable[const]` — e.g. `Runtime`@13/14, `Component.getHeight`@1, which is why
cp3–5 worked), but **breaks index-table calls** (`a.startApp` reads
`virtual_method_offsets` as halfwords → garbage index → the observed hang after
`getHeight`).

Correct model for the next pass:
- `virtual_method_offsets[methodref*2]` (halfword) = the method's vtable index.
- `static_method_offsets[i*4]` (word) = direct function pointer (already correct).
- `field_offsets[i*2]` (halfword) = field slot (already correct).
- `obj+0` = a **separate** per-(class) pointer vtable, `vtable[idx] = trampoline`.
- Open question (needs RE): the `methodref` index space (`0xc8` ⇒ ref 100) is larger
  than the 29-entry `virtual_methods` input array — the ref→method mapping that
  `java_load_classes` must honour when filling `virtual_method_offsets` is not yet
  decoded.

## Evidence table — `(java/lang class, vtable index)` the AOT calls (maintainer Q)

| object class | vtable index | inferred method | basis | status |
|---|---|---|---|---|
| `java/lang/Runtime` | 13 | `freeMemory()J` | result used as value | 추정 (placed) |
| `java/lang/Runtime` | 14 | `gc()V` | result discarded (void) | 추정 (placed) |
| `java/lang/System`/Object | (after the above) | — | reached later | 미해결 |

확정 / 추정 / 미해결:
- **확정**: no java/lang vtable spec in the dependency; virtual dispatch is two-level
  (index table + object pointer vtable).
- **추정**: `Runtime.vtable[13]=freeMemory`, `[14]=gc` (from usage); validated by
  reaching `a.startApp`.
- **미해결**: the full `java/lang/{Object,Runtime,System}` vtable layout, and the
  `methodref → method` index-space mapping for `virtual_method_offsets`.

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# trace: Runtime.gc()/freeMemory() dispatch, then a.startApp, then NoSuchMethodError
# Game.getHeight (Blocker A). Disasm shows the two-level dispatch (Blocker B).
```

## Remaining work
1. Rework virtual dispatch to the two-level model (Blocker B); decode the
   `virtual_method_offsets` methodref index space.
2. Reconcile the wie/LGT platform hierarchy (Blocker A: `Jlet` as Component/Canvas).
3. Obtain the `java/lang/{Object,Runtime,System}` vtable layout (platform spec) to
   replace the 추정 placements.
4. Field-storage unification (cp3 item 4).
With these, expect `a.startApp` → Card `o` → `paint(Graphics)` toward the title.
