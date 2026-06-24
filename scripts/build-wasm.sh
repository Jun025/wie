#!/usr/bin/env bash
#
# Reproducible wasm build for the wie web frontend.
#
# 1. compile the wie_web cdylib for wasm32
# 2. run wasm-bindgen (--target web) to emit the ES-module glue + bindings wasm
# 3. optionally shrink with wasm-opt if binaryen is installed
#
# Output lands in web/src/wasm/ which the Vite app imports. The generated files
# are build artifacts and are git-ignored.
#
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version 0.2.108   # must match Cargo.lock
#   (optional) brew install binaryen / npm i -g binaryen   # for wasm-opt
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

OUT_DIR="web/src/wasm"
WASM_IN="target/wasm32-unknown-unknown/release/wie_web.wasm"

echo "==> cargo build (wasm32, release)"
cargo build --target wasm32-unknown-unknown --release -p wie_web

echo "==> wasm-bindgen"
wasm-bindgen --target web --out-dir "$OUT_DIR" --out-name wie_web "$WASM_IN"

if command -v wasm-opt >/dev/null 2>&1; then
  echo "==> wasm-opt -Oz"
  wasm-opt -Oz -o "$OUT_DIR/wie_web_bg.wasm" "$OUT_DIR/wie_web_bg.wasm"
else
  echo "==> wasm-opt not found, skipping (optional)"
fi

echo "==> done. bundle:"
ls -la "$OUT_DIR"
