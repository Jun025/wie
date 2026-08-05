# Emulator Architecture

This document describes the emulator itself: how the top-level crates fit together, how an app moves from file detection to execution, and how the supporting libraries are used.

## High-Level Shape

The repository is organized around three layers:

1. **Host layer**
   - owns the window, audio output, filesystem access, clock, stdout/stderr, and app database
   - abstracted through the `wie_backend::Platform` trait
   - consumed by a private web frontend in the main product, and by `wie_cli` for local development
2. **Runtime layer**
   - owns task scheduling, event delivery, audio playback scheduling, and platform-neutral services used by emulated apps
   - represented primarily by `wie_backend`
3. **Execution layer**
   - runs platform-specific app code
   - split between the Java runtime (`wie_jvm_support`) and the ARM emulator (`wie_core_arm`)

Platform crates such as `wie_ktf`, `wie_lgt`, `wie_skt`, and `wie_j2me` sit on top of those layers and choose which execution model to use.

## Crate Map

The repo is one Cargo workspace (the emulator engine) plus a Cloudflare Pages web service that
embeds the engine as WASM. Nothing above the engine is allowed to reach back into it.

```
                    ┌─ hosts ────────────────────────────────────────┐
  wie_cli  (native desktop: winit/softbuffer/rodio/midir)            │
  wie_web  (browser: wasm-bindgen; empty lib on non-wasm targets)    │
                    └───────────────┬────────────────────────────────┘
                                    │ implements wie_backend::Platform
                    ┌───────────────▼────────────────────────────────┐
  wie_backend       │ Platform / Screen / AudioSink / Filesystem /   │
                    │ Database traits, canvas, executor, event queue │
                    └───────────────┬────────────────────────────────┘
        ┌───────────────────────────┼───────────────────────────┐
  wie_ktf / wie_lgt / wie_skt / wie_j2me   ← per-carrier emulator entry points
        └───────────┬───────────────┴──────────────┬────────────┘
  wie_core_arm (ARM32 CPU + binary patches)   wie_jvm_support (RustJava bridge)
  wie_wipi_c / wie_wipi_java / wie_midp / wie_skvm  ← the emulated API surfaces
                                    │
                              wie_util (Result/WieError, byte read/write helpers)
```

### Crate roles

| Crate | Role |
|---|---|
| `wie_util` | `Result`/`WieError`, `ByteRead`/`ByteWrite`, generic memory read/write helpers. Bottom of the stack — depends on nothing in-tree. |
| `wie_backend` | The host-abstraction boundary. Owns the `Platform`, `Screen`, `AudioSink`, `Filesystem`, `Database`, `TaskRunner`, `Emulator` traits plus canvas/font/zip/audio services. Every host implements these. |
| `wie_core_arm` | ARM32 interpretation (`arm32_cpu`), the emulated-function calling convention, and the `data/binary_patches.toml`-driven per-game patch table. |
| `wie_jvm_support` | Bridges the emulated Java world onto the patched RustJava JVM (`JvmImplementation`, class-proto plumbing). |
| `wie_wipi_c` | WIPI C API (`WIPICContext`): graphics, kernel, media, network shims called from ARM code. |
| `wie_wipi_java`, `wie_midp`, `wie_skvm` | The emulated Java class libraries — WIPI (`org.kwis.msp.*`), MIDP (`javax.microedition.*`), and SK-VM respectively. |
| `wie_ktf`, `wie_lgt`, `wie_skt`, `wie_j2me` | Per-platform entry points: archive layout, boot/relocation, and which API surfaces get wired. `wie_ktf`/`wie_lgt` carry the heavy reverse-engineered runtimes. |
| `wie_cli` | Native host. Also ships `src/bin/wie_validate.rs`, a headless PASS/FAIL triage runner for batch game validation. |
| `wie_web` | Browser host. Compiles to an empty library off `wasm32` on purpose, so native workspace jobs stay green. |
| `wie_ktf_dump` | Dev-only binary: dumps a KTF game's relocated `client.bin` for IDA/Ghidra. |
| `test_utils` | Shared in-memory `Platform`/`Filesystem`/`Database`/JVM fixtures for tests. |

### Non-Rust surfaces

| Path | Role |
|---|---|
| `web/` | React 19 + Vite + Tailwind frontend. `web/src/wasm/` holds the engine artifact (`wie_web.js` glue + `wie_web_bg.wasm`) that `build-wasm.sh` generates — git-ignored, never committed (`web/.gitignore`). |
| `functions/` | Cloudflare Pages Functions (the API). `functions/_lib/` is import-only (Pages does not route `_`-prefixed paths); `functions/api/` is the routed surface. |
| `migrations/` | D1 SQL migrations, auto-applied to the prod DB on `main` pushes. |
| `scripts/` | Build and gate scripts: `build-wasm.sh`, `check-engine-contract.mjs`, `contract-roundtrip.mjs`, `audit-no-leak.sh`, `verify-browser.mjs`, `smoke_gate.sh`, `lgt_render_probe.sh`. |
| `docs/` | This file, per-platform RE notes (`ktf.md`, `lgt.md`, `lgt_abi.md`), `contracts/` (the featurephone consumer pin), `project-kb/`, `verification/` screenshots, `worklog/`. |
| `data/`, `fonts/`, `test_data/` | Binary-patch table, the bundled `neodgm.ttf`, and the hello-world fixtures the tests and the round-trip gate boot. |

## Why Most Crates Are `no_std`

Most crates in the repository are written as `no_std` crates so the core emulator logic is not tied to a native desktop runtime.

This supports a few goals:
- keeping the execution core portable across native and wasm targets
- avoiding accidental dependence on OS facilities that do not exist in browser-hosted builds
- forcing platform interaction to go through explicit abstractions such as `Platform`, `System`, and the ARM/JVM bridges
- making the host-specific layer narrow, so desktop CLI and the private web frontend can share the same emulator core

In practice, host-facing crates such as `wie_cli` can use `std`, while the emulator core, runtime services, and platform implementations stay mostly `no_std` with `alloc`.

## Startup Flow

The repository includes `wie_cli` as a local development helper for driving the emulator from the desktop. The main user-facing frontend lives in a separate private web interface repository.

In this repository, the observable startup flow is:

1. `wie_cli` reads the input file and identifies the archive format.
2. It constructs the matching platform emulator (`KtfEmulator`, `LgtEmulator`, `SktEmulator`, or `J2MEEmulator`).
3. The selected emulator builds a `wie_backend::System`, injects the host `Platform`, and starts the platform-specific boot sequence.
4. The main loop repeatedly calls `Emulator::tick()` and forwards host events into the emulator.

This means `wie_cli` is mostly a development-oriented host adapter and format dispatcher; the real platform behavior lives in the platform crates, and the same backend abstractions are intended to be driven by the web frontend as well.

## Core Crates

### `wie_backend`

`wie_backend` is the shared runtime used by all emulators.

It provides:
- `System`, the central runtime object shared across tasks and platform code
- a small cooperative `Executor`
- the app-visible event queue
- a virtual filesystem populated from the input archive
- the audio manager
- host abstraction through the `Platform` trait

`System::spawn()` schedules async work on the internal executor, while `TaskRunner` decides where that work should actually run.

For pure Java platforms such as SKT and J2ME, `DefaultTaskRunner` runs futures directly. For ARM-backed platforms such as KTF and LGT, the platform crate provides a task runner that moves the future into an ARM thread context through `ArmCore::run_in_thread()`.

These async tasks are not just a convenience abstraction. For ARM-backed platforms they model the original runtime's thread model. This is especially important for the wasm target, where native OS threads are not available, so historical ARM-thread behavior has to be represented in the cooperative async/task system instead.

### `wie_core_arm`

`wie_core_arm` is the ARM execution engine wrapper.

It is responsible for:
- mapping and loading executable data into emulated memory
- running ARM functions through `run_function()`
- maintaining thread-local ARM state
- dispatching registered Rust callbacks when ARM code reaches trampoline addresses

This crate is the bridge that makes mixed Rust/ARM execution possible. Platform runtimes use it both to run native binaries and to expose host/runtime services back to native code as registered functions.

### `wie_jvm_support`

`wie_jvm_support` builds and configures the Java runtime used by the project.

It provides:
- `JvmSupport::new_jvm()` for common JVM setup
- the `JvmImplementation` abstraction for choosing how classes are defined
- shared runtime glue between `System` and the Java VM

There are two important JVM modes in the repository:
- `RustJavaJvmImplementation`: loads normal Java classfiles and Rust-defined class prototypes
- platform-specific implementations such as KTF's custom JVM path, where class and method metadata come from native binary structures instead of `.class` files

### API crates: `wie_midp`, `wie_wipi_java`, `wie_wipi_c`, `wie_skvm`

These crates implement the API surfaces visible to emulated applications.

- `wie_midp` provides MIDP classes such as LCDUI, RMS, media, and the launcher used by J2ME-style apps
- `wie_wipi_java` provides Java-side WIPI classes, largely implemented on top of the MIDP layer from `wie_midp`
- `wie_wipi_c` provides C-side WIPI method glue
- `wie_skvm` provides SKVM-specific Java APIs

They are mostly collections of Rust-defined Java class prototypes and method implementations that plug into the JVM created by `wie_jvm_support`.

## Platform Execution Models

### Pure JVM platforms

`wie_j2me` and `wie_skt` run entirely inside the Java runtime.

- no ARM core is created
- the emulator creates a JVM with the needed API prototypes
- startup code resolves the entry class and invokes the expected Java entry point

### Mixed JVM + ARM platforms

`wie_ktf` and `wie_lgt` use `ArmCore` in addition to the JVM.

They both:
- create an ARM core
- create a `System` whose tasks execute inside ARM thread contexts
- load native code into ARM memory
- expose Rust functions back to ARM code through registered callbacks

They differ in how their native binaries are structured and how Java integration works:
- KTF reads Java metadata and AOT-compiled method bodies from `client.bin`
- LGT currently focuses on ELF-native execution and import-table based native integration

## Rust/ARM Boundary

The project relies heavily on callback registration.

At the boundary:
- Rust registers a function with `ArmCore`
- ARM code receives the synthetic address of that function
- when ARM execution reaches that address, `wie_core_arm` suspends normal ARM stepping and calls back into Rust
- Rust can then inspect emulated registers and memory, perform host-side work, and return values back into the ARM context

This pattern is used for:
- WIPI C functions
- Java bridge trampolines
- native import resolution
- allocation, exception, and class-loading helpers

## Async Model

The emulator uses a small cooperative async runtime from `wie_backend` rather than Tokio or async-std.

The important split is:
- `Executor` owns task scheduling and sleeping
- `TaskRunner` decides how a task is executed

That separation lets the same `System` abstraction support:
- direct Rust execution for pure JVM platforms
- ARM-thread-backed execution for platforms that require thread-local ARM state

In practice, this means the async runtime is also the portability layer that preserves thread-like behavior on targets that cannot expose real native threads.

## Audio Path

Audio is funneled through `wie_backend::system::audio`.

The current implementation focuses on SMAF:
- app code requests audio through Java or WIPI APIs
- backend audio stores the raw SMAF data and assigns a handle
- playback parses SMAF into timestamped events
- those events are emitted to the host `AudioSink` as PCM wave playback or MIDI messages

On CLI, the host implementation uses `rodio` for wave output and `midir` for MIDI output.

## Custom Supporting Libraries

Some dependencies are external at the Cargo level but are logically part of this project and maintained by the same author.

### RustJava

Workspace dependencies such as `jvm`, `java_runtime`, `java_class_proto`, and `jvm_rust` come from the RustJava project.

In this repository they provide:
- the Java VM core
- class loading and runtime support
- Rust-defined Java class prototypes
- standard Java runtime classes used by the emulated environments

### `wipi` / `wipi_types`

`wipi_types` is part of the `wipi` project. It provides binary structure definitions and ABI-level constants for WIPI-related formats and native interfaces.

In practice it is the contract layer between reverse-engineered native binaries and the Rust implementation: pointers read from emulated memory are decoded using these shared structure definitions.

The `wipi` project also serves as the homebrew SDK for building applications runnable in this emulator, so the type definitions and ABI knowledge used here are closely related to the app-side development toolchain as well.

### `smaf` / `smaf_player`

SMAF support is also provided by custom libraries. SMAF here refers to Yamaha's Synthetic music Mobile Application Format (MMF). The emulator uses these libraries to parse MMF/SMAF assets into timed events, which are then translated into host-side PCM or MIDI operations.

## Design Bias

The project is structured around matching the behavior of historical mobile runtimes rather than forcing all platforms into a single abstraction.

That is why:
- each platform crate owns its own boot sequence
- KTF and LGT have different native integration models
- API crates are shared where possible, but platform-specific runtime glue stays platform-specific
- the JVM and ARM layers are kept separate and joined only through explicit bridge code
