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

# wasm-opt is optional. An OLD binaryen can fail to parse wasm emitted by a
# recent rustc/wasm-bindgen ("Fatal: error parsing wasm"), so we optimize into a
# temp file and only swap it in on success — a failure ships the (larger but
# valid) unoptimized bundle instead of breaking the build.
if command -v wasm-opt >/dev/null 2>&1; then
  echo "==> wasm-opt -Oz"
  if wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int \
      -o "$OUT_DIR/wie_web_bg.opt.wasm" "$OUT_DIR/wie_web_bg.wasm"; then
    mv "$OUT_DIR/wie_web_bg.opt.wasm" "$OUT_DIR/wie_web_bg.wasm"
  else
    echo "==> wasm-opt failed (old binaryen?) — shipping unoptimized wasm"
    rm -f "$OUT_DIR/wie_web_bg.opt.wasm"
  fi
else
  echo "==> wasm-opt not found, skipping (optional)"
fi

echo "==> done. bundle:"
ls -la "$OUT_DIR"
