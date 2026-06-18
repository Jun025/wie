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
| **cp9: per-class platform vtable / native-object investigation** | ⏹ **STOP (condition B)** |
| `paint`/title | ❌ blocked — external spec / major infra needed |
| clet (`test_helloworld`) | ✅ | clippy | ✅ |

## Checkpoint 9 — per-class platform vtables: investigation → STOP (B)

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

## Reproduce
```sh
cargo build -p wie_cli
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=error \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
# ... Game.a() -> new StringBuffer() (constructs) -> stringBuffer.vtable[19] ->
# NoSuchMethodError StringBuffer.setXORMode (global-slot collision; the AOT wants
# append(String) at vtable 19, which wie's order places at 12).
```

## Recommended next steps (need a decision / external input)
1. **Platform vtable-index spec.** Obtain (or reverse-engineer from the LGT/ez-i
   runtime) the per-class vtable layout for StringBuffer and java/lang/*; fill the
   per-class vtables from it. Without it, only per-slot empirical RE (slow, bounded
   to observed calls) is no-guess-safe.
2. **Native↔JVM object/String bridge.** Decide how natively-allocated objects
   (string constants, StringBuffer, etc.) map to JVM instances (e.g. read native
   object fields on demand, or intern native Strings into JVM Strings).
3. Then resume: app field unification (cp3 item 4), Blocker A (Jlet Component
   methods), and the stdlib/java-runtime tail, toward Card `o` / `paint`.
