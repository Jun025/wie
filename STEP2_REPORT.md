# STEP report — LGT native-backed JVM (checkpoints 1–7)

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
| cp6: two-level virtual dispatch (the a.startApp fix) | ✅ |
| **cp7: stdlib `0x32` = native object allocator (`new`)** | ✅ implemented |
| reach | `a.startApp` → `Game.a()` native → `new StringBuffer()` |
| next blocker | `StringBuffer.<init>` called with **null `this`** (field-route, not the allocator) |
| `paint`/title | ❌ not yet |
| clet regression (`test_helloworld`) | ✅ | clippy | ✅ |

cp1–6 are in the git history; this revision covers **checkpoint 7**.

## Checkpoint 7 — stdlib `0x32` (native object allocator)

### Identification (no guessing)

1. **Dependency**: `wipi`/`wipi_types` define no stdlib index map (only
   `ImportModule::WIPIC=0x1fb`); the reference invokes platform methods by name. The
   known `StdlibSvcId` range is `0x3f6`–`0x424`, far from `0x32`. → unavailable.
2. **Disassembly**: `0xe2c50` is the lazy-bind trampoline for `(table=1, index=0x32)`
   (`str lr; bl resolver; .word 1; .word 0x32`).
3. **Behavioral probe** (logged args, returned 0): the next event was
   `StringBuffer.<init>()` with a **null `this`**. So the AOT pattern is
   `obj = 0x32(...); obj.<init>()` → **stdlib `0x32` is the native object allocator
   (`new`)**, not a leaf libc function.

### Implementation

- `LgtJvmShared::alloc_native_object`: allocate a guest object block (header +
  zeroed field array, vtable word at `+0x00`), mark it **pending**; stdlib `0x32`
  returns it.
- The **`<init>` trampoline binds** a pending native object to a JVM instance of the
  constructed class: an app class → an `LgtClassInstance` reusing the guest block; a
  platform class → JVM-instantiated, keyed by the guest pointer. (This is the
  native-`new` ↔ JVM-object integration.)
- Threaded `LgtJvmShared` into the stdlib handler.
- Fixed `virtual_method_offsets` to write the identity index only for real method
  refs (avoids overshooting the ~102-entry table).

### Reach
```
a.startApp -> Game.a() [native ARM]
  stdlib new (0x32) -> 0x48840070           (allocator implemented; past the fatal)
  java/lang/StringBuffer.<init>()V  this_raw=0x0   <- BLOCKER
```

### Next blocker — `StringBuffer.<init>` with null `this`

`new StringBuffer()` reaches `StringBuffer.<init>` with `this_raw = 0`. Crucially
this is **independent of the allocator**: the allocator returned a valid block
(`0x48840070`), but the `this` passed to `<init>` is **0**, arriving via a
field/local route, not the allocator's return. So the object reference is being
**lost across a field store/load** — a field-storage / construction-flow issue.

A blanket identity `field_offsets` fill was tried to make native field round-trips
self-consistent, but it **regressed `a.startApp`** (crashed earlier at address 0):
the field semantics are subtler than identity (some refs are inherited platform
fields whose values live JVM-side, not in the guest field array). So this is the
**field-storage unification problem (cp3 item 4)**, now on the critical path, and
needs dedicated RE of the construction sequence (how the `new`'d object reference
flows to `<init>`).

확정 / 추정 / 미해결:
- **확정**: stdlib `0x32` = native object allocator (`obj=0x32(); obj.<init>()`).
- **추정**: the allocator's size argument (impl uses a fixed generous block; args
  `(0,8,1)` not fully pinned).
- **미해결**: `StringBuffer.<init>` null `this` (field-route); field-storage
  unification (cp3 item 4); Blocker A (`Jlet` Component methods); remaining
  `java/lang/{Object,System}` vtable slots.

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# trace: ... -> Game.a() -> "stdlib new (0x32) -> 0x..." ->
# StringBuffer.<init>() this_raw=0x0 (NullPointerException).
```

## Remaining work
1. RE how the `new`'d object reference reaches `<init>` (field store/load) and fix
   field-storage so native-written object refs are read back correctly — likely the
   general field-unification (cp3 item 4), now blocking.
2. Continue the stdlib/native-runtime tail as functions appear.
3. Blocker A (`Jlet` Component methods) in the LGT PoC layer; remaining `java/lang`
   vtable slots.
With these, expect `Game.a()` to finish constructing app state → `a.startApp` →
Card `o` → `paint(Graphics)` toward the title.
