# Plan: wie engine artifact publish → otterpebble propagation (발신부)

> Session: F3 · wie repo only. Do not modify otterpebble/qts/RustJava. Coupling = build-artifact boundary only.
> SoT for this repo: AGENTS.md · CLAUDE.md · .github/workflows · scripts/build-wasm.sh · Cargo.toml.

## Goal
When the wie engine improves (fresh WASM · new device backend · new format) and lands on `main`, publish a
versioned, hash-pinned WASM artifact and signal otterpebble via `repository_dispatch` so
`featurephone.otterpebble.com` serves the new engine **with no human step** (federation ①: publish → dispatch →
receiver pin-bump → featurephone deploy). This session builds the **sender** only; otterpebble owns the receiver.

## Receiver contract (otterpebble-owned · measured read-only · DO NOT change — match it)
`apps/featurephone/.github/workflows/wie-artifact-receive.yml`:
- **event_type**: `wie-artifact-published`
- **client_payload** (all required unless noted): the receiver validates presence of each and
  `curl -fSL` downloads the two URLs **unauthenticated**, then `sha256sum -c`, then bumps
  `apps/featurephone/public/engine/manifest.json` and commits (→ ci.yml deploys).

| payload field | meaning | source (this repo) |
|---|---|---|
| `version` | crate version → manifest `crateVersion` | `Cargo.toml` workspace.package.version (`0.0.1`) |
| `wieHead` | engine commit → manifest `wieHead` + commit msg | `git rev-parse HEAD` |
| `wasmUrl` | public download URL for the wasm | GitHub Release asset `browser_download_url` |
| `glueUrl` | public download URL for the glue | GitHub Release asset `browser_download_url` |
| `wasmSha256` | integrity → receiver fail-closed on mismatch | `sha256sum web/src/wasm/wie_web_bg.wasm` |
| `glueSha256` | integrity | `sha256sum web/src/wasm/wie_web.js` |
| `confirmedPlatforms` (optional) | display hint | `["KTF","SKT"]` (featurephone plan) |

- **Public repo** (`Jun025/wie` is PUBLIC) → Release asset `browser_download_url` is curl-able without auth →
  matches the receiver's plain `curl -fSL`. This is why we publish to a GitHub Release (not R2/private storage).

## Build path (reuse existing repo conventions — measured)
- `scripts/build-wasm.sh` (already in repo): `cargo build --target wasm32-unknown-unknown --release -p wie_web`
  → `wasm-bindgen --target web --out-dir web/src/wasm --out-name wie_web` → optional `wasm-opt -Oz`.
  Outputs `web/src/wasm/{wie_web_bg.wasm, wie_web.js}` (git-ignored build artifacts — no large binaries in git).
- CI toolchain mirrors `web.yml`: `dtolnay/rust-toolchain` (wasm32 target) + `taiki-e/install-action wasm-bindgen-cli@0.2.108`
  (pinned to Cargo.lock) + binaryen (for wasm-opt). Fresh build from current source — never the stale committed dist.

## Milestones
- **M1 build+publish**: `.github/workflows/publish-artifact.yml`. Trigger = push to `main` (paths: Rust sources +
  Cargo + build script + this workflow) + `workflow_dispatch`. Build fresh WASM → sha256 → `gh release create
  engine-<shortsha>` with the two files as assets (idempotent: skip if the tag already exists).
- **M2 dispatch**: after release, `gh api repos/Jun025/otterpebble/dispatches` with `event_type=wie-artifact-published`
  and the payload above. Token = `secrets.OTTERPEBBLE_DISPATCH_TOKEN` (a PAT/fine-grained token with
  `contents:write` on **otterpebble**, since the default `GITHUB_TOKEN` only reaches this repo). **Absent token →
  skip dispatch** (build+release still run) = dormant, activates the moment the token is injected. No plaintext token.
- **M3 verify**: local `scripts/build-wasm.sh` run (proves fresh build + hashes) + YAML validate + confirm dispatch
  dormant-without-token.

## Trigger cadence (tunable · documented)
- Default = **every `main` push touching engine sources** → featurephone always serves the latest working engine.
  wie is a **public repo → free GitHub Actions**, so build frequency has no cost. The receiver only bumps when the
  artifact sha changes (always true per commit — that is the intent: fresh engine). featurephone deploy is a cheap
  static direct-upload.
- If churn is undesired later, switch the trigger to release **tags** (`on: push: tags: [engine-v*]`) for deliberate
  releases — a one-line change. Recorded here as the alternative.

## Human console remnant (surface honestly — no false completion)
- **Inbound PAT**: create a token with `contents:write` (repository_dispatch) on `Jun025/otterpebble`, register it
  as wie repo secret `OTTERPEBBLE_DISPATCH_TOKEN`. Until then: build+release run, **dispatch skips (dormant)**.
- otterpebble receiver is already in place (this session does not touch it).

## Boundary / invariants
- wie repo only. Artifacts cross the boundary; **no source vendoring**. No game files produced/bundled (WASM only).
- CI gate (`rust.yml`: fmt/clippy/test matrix) must stay green — this change is workflow-YAML-only (no Rust delta).
