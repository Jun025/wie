# STEP report — LGT native-backed JVM (checkpoints 1–8)

Goal: run the AOT-compiled LGT app (BattleMonster `00025C2B`) on wie's JVM, toward
`startApp` → `paint(Graphics)` (title). Branch `feat/lgt-java-interface-bridge`
(local only). PoC `LgtJvmShared` kept LGT-specific per #1232; shared
`wie_wipi_java`/`wie_midp` classes **not** modified.

## Status summary

| item | result |
|---|---|
| cp1–2: app classes registered; methods run as real ARM | ✅ |
| cp3: `java_load_classes` tables; native↔platform bridge | ✅ |
| cp4–5: per-class platform vtables; Runtime vtable wall crossed | ✅ |
| cp6: two-level virtual dispatch | ✅ |
| cp7: stdlib `0x32` = native allocator | ✅ |
| **cp8: java `0xf` = native object allocator; `new StringBuffer()` constructs** | ✅ |
| reach | `Game.a()` → constructs StringBuffers → `stringBuffer.vtable[19]()` |
| next blocker | StringBuffer's per-class vtable (idx 19 collides with Graphics global slot) |
| `paint`/title | ❌ not yet |
| clet (`test_helloworld`) | ✅ | clippy | ✅ |

cp1–7 are in the git history; this revision covers **checkpoint 8**.

## Checkpoint 8 — construction sequence RE + java `0xf` (object allocator)

### The cp8 hypothesis (field round-trip) was wrong — RE found the real cause

The task hypothesised the `new StringBuffer()` null-`this` came from a field
store/load mismatch. Disassembling the actual sequence (`0xfc00` / `0x4740`):
```
r0 = java(0x64 / 0x0f)();    // .data trampoline 0x140452c -> resolver -> (table 0x64, index 0xf)
StringBuffer.<init>(r0);      // r0 is the object's `this`
```
The `this` is the **return of java-interface import `0xf`**, not a field. And `0xf`
fell into the cp5 **generic no-op java stub** (returns 0) — so `this` was 0. **Not a
field-storage problem.**

### Fix — java `0xf` = native object allocator

Routed java-interface `0xf` to `LgtJvmShared::alloc_native_object` (added cp7): it
returns a pending guest object, and the **`<init>` trampoline binds** it to a JVM
instance of the constructed class (the native-`new` ↔ JVM-object path). 

Result:
```
stdlib new (0x32) -> 0x48840090
StringBuffer.<init>()V  this_raw=0x48840090 this_actual="java/lang/StringBuffer"   ✓
... a second StringBuffer constructs too ...
```
The new'd object now reaches `<init>` with a correct `this` — **cp8's goal,
achieved via the actual mechanism** (object allocator), not field unification.

### Next blocker — StringBuffer per-class vtable (idx 19)

After construction the native does (`0x4784`): `r3=[stringBuffer]; bx [r3 + 0x4c]`
— `stringBuffer.vtable[19](arg)`, an **append-like** call. The StringBuffer's
`+0x00` is the global vtable, whose slot 19 is `Graphics.setXORMode` (Graphics'
imported global position) → `NoSuchMethodError: StringBuffer.setXORMode`.

StringBuffer's virtual methods are **not imported** (`java_load_classes` lists only
Graphics/Card/Display/… methods), so its per-class vtable index 19 collides with
Graphics'. This is the **same per-class platform-vtable wall as java/lang**
(Runtime): a platform class the AOT calls by hardcoded vtable index whose layout is
not in the app data. Resolving it needs per-class vtables built from a platform
vtable-index spec (or `wie` method lists, with index matching) — **not forced**
(would diverge).

확정 / 추정 / 미해결:
- **확정**: the `new StringBuffer()` null-`this` was java `0xf` (object allocator)
  no-op'd, not a field issue; implementing `0xf` fixes it.
- **추정**: StringBuffer `vtable[19]` ≈ `append` (called `this` + 1 arg).
- **미해결**: per-class vtables for platform classes whose methods aren't imported
  (StringBuffer, java/lang `{Object,Runtime,System}`); field-storage unification
  (cp3 item 4) for app-declared fields (not the cause here, still future work).

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# trace: ... -> Game.a() -> StringBuffer.<init> (this_actual=java/lang/StringBuffer)
# -> NoSuchMethodError StringBuffer.setXORMode (vtable[19] collision).
```

## Remaining work
1. **Per-class platform vtables for non-imported-method classes** (StringBuffer,
   java/lang). This is the dominant recurring wall: build each platform class's
   vtable (index→method) — needs the WIPI/ez-i vtable spec or a `wie`-method-list
   reconstruction validated by observed `(class, index)` calls.
2. Field-storage unification (cp3 item 4) for app-declared fields.
3. Continue the stdlib/java-runtime tail as functions appear.
With per-class platform vtables, `Game.a()` should finish building app state →
`a.startApp` → Card `o` → `paint(Graphics)` toward the title.
