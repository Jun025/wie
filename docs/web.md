# Web frontend (client-only)

A browser build of the wie emulator. It is **100% client-side**: the user picks
a game file locally, the bytes are injected straight into wasm memory, and the
emulator runs entirely in the tab. There is no server component and game files
never leave the browser.

## Layout

```
wie_web/            # Rust: browser host adapter (additive; core untouched)
  src/lib.rs        #   #[wasm_bindgen] entry — WieEmulator (load/tick/input/saves)
  src/platform.rs   #   wie_backend::Platform impl for the browser
  src/screen.rs     #   Screen  -> <canvas> putImageData (RGBA, opaque alpha)
  src/audio.rs      #   AudioSink -> WebAudio PCM (MIDI = silent stub)
  src/filesystem.rs #   Filesystem -> in-memory, aid-isolated (saves only)
  src/database.rs   #   DatabaseRepository -> in-memory, app-isolated
scripts/build-wasm.sh   # cargo build (wasm32) -> wasm-bindgen -> wasm-opt
web/                # Vite + React + TypeScript + Tailwind SPA
  src/lib/emulator.ts   # wasm glue + rAF loop + IndexedDB save persistence
  src/lib/keymap.ts     # keyboard -> KeyCode map (localStorage)
  src/lib/idb.ts        # IndexedDB (saves only)
  src/components/       # VirtualPad, KeyRemap
```

`wie_web` compiles to an **empty library on non-wasm targets**, so native
workspace jobs (`cargo build`, `cargo test --all`, `cargo clippy --all`) are
unaffected. All real code is behind `#[cfg(target_arch = "wasm32")]`.

## Build & run

Prerequisites (one-time):

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.108   # must match Cargo.lock
# optional, for size: brew install binaryen  (or: npm i -g binaryen)
```

Develop:

```sh
cd web
npm install
npm run wasm     # builds wie_web -> web/src/wasm/ (git-ignored)
npm run dev      # Vite dev server
```

Production build (static):

```sh
cd web
npm run build    # runs wasm build, tsc, vite build -> web/dist/
```

## Cloudflare Pages

Static deploy of `web/dist/`:

- Build command: `npm run build` (run from `web/`)
- Output directory: `web/dist`
- **No COOP/COEP headers required.** The emulator core is single-threaded and
  uses no `SharedArrayBuffer`, wasm threads, or `Atomics` cross-thread, so
  cross-origin isolation is unnecessary.
- The wasm build step needs the Rust toolchain + `wasm-bindgen-cli` in the build
  environment. If the Pages build image lacks Rust, build `web/src/wasm/`
  beforehand (CI artifact) and deploy the prebuilt `web/dist/`.

## Measured bundle size

| stage | size |
| --- | --- |
| `wie_web_bg.wasm` (after `wasm-opt -Oz`) | ~6.9 MB raw |
| gzip | ~1.98 MB |
| JS glue (`wie_web.js`) | ~20 KB |

The wasm carries the full ARM core, the RustJava JVM, all four platform runtimes
(KTF/LGT/SKT/J2ME) and the embedded `neodgm` font. One cached download per
visitor.

## Privacy / BYOF

- Game bytes flow: `<input type=file>` → `ArrayBuffer` → `Uint8Array` →
  `WieEmulator` constructor (wasm memory). They are never fetched/uploaded.
- Only **saves** are persisted, to IndexedDB, in this browser. Game files are
  not persisted and not part of any snapshot.
- There is no `fetch` / `XMLHttpRequest` / `WebSocket` / `sendBeacon` of game
  data anywhere in the Rust or TypeScript code.
