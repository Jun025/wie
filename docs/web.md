# Web service (client emulator + accounts/saves backend)

A browser build of the wie emulator plus a thin Cloudflare backend for
**accounts** and **save sync**. The defining rule:

> Game handling is **100% client-side**. Game bytes — and the device's
> game-ownership metadata (filenames, content hashes, "which games this device
> has") — never leave the browser. The server stores ONLY ⓐ account info,
> ⓑ opaque save data, and ⓒ inquiry text. BYOF only.

## Layout

```
wie_web/            # Rust: browser host adapter (additive; core untouched)
  src/lib.rs        #   #[wasm_bindgen] entry — WieEmulator
                    #     load/tick/input, export_saves/import_saves (opaque RMS+FS blob)
  src/platform.rs   #   wie_backend::Platform impl for the browser
  src/screen.rs     #   Screen  -> <canvas> putImageData (RGBA, opaque alpha)
  src/audio.rs      #   AudioSink -> WebAudio PCM (MIDI = silent stub)
  src/filesystem.rs #   Filesystem -> in-memory, aid-isolated (saves only)
  src/database.rs   #   DatabaseRepository -> in-memory, app-isolated (RMS)
scripts/build-wasm.sh   # cargo build (wasm32) -> wasm-bindgen -> wasm-opt
web/                # Vite + React + TypeScript + Tailwind SPA
  src/App.tsx           # tabbed shell: Library / Player / Cloud saves / Inquiry / Account
  src/components/       # GameLibrary, Player, VirtualPad, KeyRemap, AuthPanel, CloudSaves, InquiryForm
  src/hooks/useAuth.ts  # session state
  src/lib/emulator.ts   # wasm glue + rAF loop + opaque-blob autosave
  src/lib/library.ts    # IndexedDB: device-local game library + save cache (NO network)
  src/lib/api.ts        # the ONLY module that calls the server
  src/lib/saveSync.ts   # opaque save push/pull by user alias
  src/lib/keymap.ts     # keyboard -> KeyCode map (localStorage)
functions/          # Cloudflare Pages Functions (the backend)
  _lib/               # crypto (PBKDF2 + HMAC sessions), session gate, rate limit
  api/auth/*          # register / login / logout / me
  api/saves/*         # owner-scoped CRUD (cross-user access -> 404)
  api/devices.js      # other-device view: slot counts only, no game identity
  api/inquiries/*     # text only; binary/multipart attachments rejected (415)
migrations/0001_init.sql   # users / sessions / saves / inquiries / rate_limits
wrangler.toml       # Pages config: output web/dist, D1 binding DB
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

Frontend dev / build:

```sh
cd web
npm install
npm run dev      # Vite dev server (frontend only)
npm run build    # wasm build + tsc + vite build -> web/dist/
```

Full stack locally (frontend + Functions + D1), from the repo root:

```sh
npm install                         # ops tools (wrangler, playwright)
cp .dev.vars.example .dev.vars      # set SESSION_SECRET (openssl rand -hex 32)
(cd web && npm ci && npm run build) # -> web/dist
npm run db:migrate:local            # local D1
npm run dev                         # wrangler pages dev web/dist  (http://localhost:8788)
npm run audit                       # no-leak self-audit
```

See `docs/CLOUDFLARE_SETUP.md` for the (user-side) Cloudflare dashboard steps.

## Backend

- **Auth**: PBKDF2-HMAC-SHA256 passwords (210k iterations, per-user salt — the
  strong KDF natively in the Workers WebCrypto runtime); HMAC-SHA256-signed
  session cookie (`HttpOnly; Secure; SameSite=Lax`) backed by a `sessions` row
  for revocation/expiry; best-effort D1 rate limiting keyed by a salted IP hash.
- **Saves**: every query is owner-scoped (`WHERE user_id = ?`); a cross-user id
  returns 404. The payload is an opaque base64 snapshot; the slot is identified
  by a **user alias**, never a game title/filename/hash.
- **Inquiries**: text only — non-JSON bodies and base64-smuggled game/binary
  magic are rejected (415).
- **Devices**: returns per-alias save-slot aggregates only; it cannot reveal
  what games a device has.

## Device-local game library

`web/src/lib/library.ts` keeps uploaded game bytes + metadata in IndexedDB so a
game runs on later visits **without re-uploading**: list, click-to-run, capacity
cap, per-item/all delete. This store, and everything in it (bytes, names,
hashes), is device-only — `library.ts` performs zero network calls.

## Save sync

The emulator exports an opaque `WIESAV01` blob covering **both** RMS records and
the save filesystem. It is cached locally per game-hash and, when the user
chooses, pushed to D1 under a user alias. The hash↔alias mapping lives only in
the browser, so the server never learns which game a save belongs to.

## Cloudflare Pages

- Build command: `cd web && npm ci && npm run build`; output: `web/dist`.
- Functions are auto-detected from `functions/`; D1 is bound as `DB`.
- **No COOP/COEP headers required** — the core is single-threaded (no
  `SharedArrayBuffer`/wasm threads), so cross-origin isolation is unnecessary.
- Pages build images lack Rust, so CI (`.github/workflows/web.yml`) prebuilds the
  wasm and deploys the static `web/dist` + `functions/` (Direct Upload). The
  deploy step is skipped until the user adds the `CLOUDFLARE_API_TOKEN` secret.

## Measured bundle size

| stage | size |
| --- | --- |
| `wie_web_bg.wasm` (after `wasm-opt -Oz`) | ~6.9 MB raw |
| gzip | ~2.1 MB |
| app JS (`index-*.js`) | ~74 KB gzip |
| app CSS | ~4.5 KB gzip |

The wasm carries the full ARM core, the RustJava JVM, all four platform runtimes
(KTF/LGT/SKT/J2ME) and the embedded `neodgm` font. One cached download per
visitor.

## Privacy / BYOF — what leaves the browser

- **Never**: game bytes, game filenames, content hashes, or a "games on this
  device" list. There is no `fetch`/`XHR`/`WebSocket`/`sendBeacon` of game data
  anywhere in the Rust or TypeScript code (enforced by `scripts/audit-no-leak.sh`).
- **Only**: account info, opaque save payloads + a user alias, and inquiry text.

## Verification

Driven by a real headless Chrome (`scripts/verify-browser.mjs`): a graphical
J2ME MIDlet renders in-browser via wasm through the BYOF → IndexedDB library →
run flow; keyboard input reaches the game; and the captured network trace shows
0 off-origin requests and 0 game bytes in any request body. Screenshots in
`docs/verification/`. Backend auth / owner-isolation / inquiry-rejection /
rate-limit are checked against a local D1 via `wrangler pages dev`.
