# LGT Clet→Java Bridge — Implementation Report

Target: 배틀몬스터 (BattleMonster), AID `00025C2B`, LGT WIPI / ez-i.
Branch: `feat/lgt-java-interface-bridge`. Local-only (no remote work performed).

## TL;DR

- **Before:** boot stopped immediately at `java_unk11` → `Err(Unimplemented("LGT Java apps are not implemented yet"))`.
- **After:** the full LGT java-interface boot sequence is decoded and traced; `java_unk11`
  now performs the real `invoke-static org/kwis/msp/lcdui/Main.main(String[])`, reusing a
  shared boot helper with the WIPI-C clet path. Boot now advances **into the JVM** and
  stops at a new, precisely understood point:
  `java/lang/NoClassDefFoundError: Game at org/kwis/msp/lcdui/Main.main`.
- **Remaining blocker:** the app's own classes (incl. `Game`) are AOT-compiled native ARM
  code registered via `java_unk5` (import `0x07`). Running the game requires a
  **native-backed JVM class bridge** (register each native class as a JVM class whose
  methods dispatch into ARM code) — the same kind of subsystem as `wie_ktf/.../jvm_support/`.
  This was scoped out as too large/risky to implement speculatively in one pass; it is
  fully characterized below as the next step.
- **No regression:** the WIPI-C clet path is unchanged in behavior; `cargo test -p wie_lgt`
  (`test_helloworld`, a clet app) still passes.

## How to reproduce

```sh
# build
cargo build -p wie_cli

# run (opens a 240x320 window; boot trace on stderr)
RUST_LOG=wie_lgt=debug,wie=error,wie_core_arm=warn \
  cargo run -p wie_cli -- /absolute/path/to/00025C2B.jar
```

Observed terminal endpoint:

```
java_unk11: invoke-static org/kwis/msp/lcdui/Main.main argv=["Game", "", "true", "true"]
ERROR wie: Fatal error:
java/lang/NoClassDefFoundError: Game
	at org/kwis/msp/lcdui/Main.main([Ljava/lang/String;)V
```

## Decoded calling conventions (import table `0x64`)

The `binary.mod` is a 1 MB ARM ELF; the jar contains **no `.class` files**. The application
is an **AOT-compiled Java program** whose classes are emitted as native ARM code. It talks to
the platform through import module `0x64`. Boot-time call order and decoded semantics:

| order | import | handler | args (decoded) |
|------:|:------:|:--------|:---------------|
| 1 | `0x03` | `java_unk0` | `(main_class_name="Game", params_ptr, flag="true")` — registers main-class metadata |
| 2 | (`0x1fc/0x1ff/0x201`, fn `0x03`) | `java_unk1/2/3` | same `("Game", _, _)` tuple across 3 aux modules |
| 3 | `0x07` | `java_unk5` | `(own_class_table, aux)` — **app's own native classes** |
| 4 | `0x14` | `java_load_classes` | imported platform classes + offset-resolution tables |
| 5 | `0x82` | `java_unk9` | `(0)` — boot hook |
| 6 | `0x06` | `java_unk12` | `(own_class_table)` — same ptr as `java_unk5.a0` |
| 7 | `0x83` | `java_unk11` | **invoke-static** `Main.main(argv)` |

### `java_unk11` (0x83) — the boot invoke (IMPLEMENTED)

```
a0 = char*  class name        -> "org/kwis/msp/lcdui/Main"
a1 = 0                        (unused; implicit method "main")
a2 = u32    argc              -> 4
a3 = char** argv              -> ["Game", "", "true", "true"]
```

This is exactly analogous to the WIPI-C clet boot, which calls the same
`org/kwis/msp/lcdui/Main.main` with `["net/wie/CletWrapper"]`. `argv[0]` is the
application's main Jlet class name. Implementation: read `class_name`/`argv`, validate the
target is `Main`, and call the shared `invoke_lcdui_main(jvm, argv[0])`.

### `java_load_classes` (0x14) — imported platform classes (DECODED, not yet wired)

11 arguments. `classes` is a count-prefixed `LgtJavaImportedClass[]` (stride 24 B). For
BattleMonster: **30 imported platform classes**, all already implemented in Rust under
`wie_wipi_java` / `wie_midp`:

```
org/kwis/msp/media/{Player,Clip,Volume,Vibrator}
org/kwis/msp/lcdui/{Card,Graphics,Display,Image,Jlet,JletWrapper}
org/kwis/msp/lwc/{Component,AnnunciatorComponent}
org/kwis/msp/io/File, org/kwis/msf/io/{Socket,URL,Network}
org/kwis/msp/handset/{HandsetProperty,BackLight}
java/lang/{IllegalArgumentException,Runtime,Object,Exception,Thread,
           System,Math,StringBuffer,String}
java/io/{DataOutputStream,DataInputStream}, java/util/Random
```

Each entry carries `(static_field_off/cnt, virtual_method_off/cnt, static_method_off/cnt)`
indexing the `fields/static_fields/virtual_methods/static_methods` arrays (arrays of
`{ptr_name, ptr_type}` pairs). The trailing `*_offsets` arguments point at **writable app
RAM** (~`0x15006f4`) — these are **outputs**: the platform is meant to fill them with
resolved method indices / vtable offsets so the native code can dispatch into platform
methods. Note `JletWrapper` exposes exactly **6 static methods**, mirroring `CletFunctions`'
6 lifecycle hooks (start/pause/resume/destroy/paint/handleEvent).

### `java_unk5` (0x07) — the app's OWN native classes (DECODED, **this is the blocker**)

```
a0 -> [count, 0, desc0, desc1, ... desc_{count-1}, <name strings...>]
```

For BattleMonster, `count = 16`. Each `desc_i` points to a per-class descriptor in app RAM
(~`0x1404xxx`) that embeds **native ARM code pointers** for the method bodies (observed in
the text region, e.g. `0xd7fd0`, `0x83e60`, `0x1b520`, `0xdbb70`). Class/method/field names
are **AOT-obfuscated to single characters** (`"1"`, `"!"`, `"a"`, `"aJ"`, `"bP"`, …); only
the public entry name `"Game"` survives, delivered out-of-band via `java_unk0`/`java_unk11`.

`Main.main` instantiates `argv[0]` (`new_class("Game")`), so until these 16 native classes
are registered as JVM classes, the JVM raises `NoClassDefFoundError: Game`.

## Changes in this branch

- `wie_lgt/src/runtime/init.rs` — thread `(System, Jvm)` into the init SVC handler context so
  java-interface handlers can call the JVM (`register_init_svc_handler(core, system, jvm)`).
- `wie_lgt/src/runtime/wipi_c.rs` — extract `invoke_lcdui_main(jvm, main_class_name)` from
  `clet_register`; clet path now calls it with `"net/wie/CletWrapper"` (behavior unchanged).
- `wie_lgt/src/runtime/java/interface.rs` — decode + document every `0x64` import; implement
  `java_unk11` as the real `Main.main` invoke-static via the shared helper; concise
  `tracing::debug!` decoding for `java_unk0/5/9/12` and `java_load_classes`.
- `.gitignore` — ignore test ROMs / game binaries / logs (`_roms/`, `*.mod`, `*.sav`, `*.log`,
  `00025C2B*`).

## Next step to reach title → first battle

Implement a **native-backed JVM class registration** for the LGT java interface, porting the
approach used by `wie_ktf/src/runtime/java/jvm_support/`:

1. In `java_unk5` (and/or `java_load_classes`), parse each native class descriptor into
   `{ name, super, fields[], methods[(name, sig, native_ptr)] }`.
2. Register each as a JVM `ClassDefinition` whose method bodies are native stubs that marshal
   JVM args → ARM `r0..r3` and `core.run_function(native_ptr, …)` (mirroring how
   `CletWrapperCard`/`startApp` already calls `start_clet`).
3. Fill the `java_load_classes` `*_offsets` output tables so the native code can dispatch into
   the Rust-implemented platform classes (the reverse direction).
4. Re-run; `Main.main` should then instantiate `Game`, `startMIDlet` should drive
   `Game.startApp`, and execution should proceed toward the title screen. Sound/rendering are
   lower priority per task scope.

The full descriptor byte layout still needs a small amount of additional reverse engineering
(field/method counts, signature encoding, super-class linkage); the tracing added here
(`RUST_LOG=wie_lgt=debug`) dumps the raw descriptors to support that.
