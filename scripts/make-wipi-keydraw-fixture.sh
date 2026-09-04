#!/usr/bin/env bash
# Rebuild the WIPI "keydraw" fixtures that the KTF/LGT key-reach tests boot.
#
#   test_data/keydraw_ktf.zip   test_data/keydraw_lgt.zip
#
# Why this script exists: the committed KTF/LGT fixtures are ARM guest binaries,
# not something you can hand-edit. `helloworld_{ktf,lgt}.zip` arrived with no
# recorded recipe, so nobody could make a *different* one — which is exactly why
# key delivery on those two paths went unasserted for so long. This file is the
# recipe, and the guest source below is the fixture: reading it tells you what
# the assertion is really watching.
#
# What the fixture does: on keydown it prints `key:<code>` and paints a bar
# `<code>` pixels wide. `<code>` is the WIPI code the guest was handed, so the
# tests can name the exact integer instead of "an event arrived". Two evidence
# axes on purpose — stdout for the headless Rust tests (the test harness's
# TestScreen keeps no framebuffer), pixels for a future browser scenario.
#
# Requirements (all local; nothing is pushed anywhere):
#   - network access to clone dlunch/wipi
#   - a nightly toolchain with rust-src   (`-Zbuild-std` needs both)
#       rustup toolchain install nightly && rustup component add rust-src --toolchain nightly
#   - the thumbv4t-none-eabi target is built from source by -Zbuild-std, so it
#     does NOT need `rustup target add`.
#
# Usage:  scripts/make-wipi-keydraw-fixture.sh
#
# NOTE the output is not byte-reproducible (the build embeds paths), so this
# does not overwrite blindly-equal files — it just rewrites both zips. Verify a
# regeneration by re-running the tests, not by diffing the zips:
#   RUST_MIN_STACK=4194304 cargo test -p wie_ktf --test test_key_reach
#   RUST_MIN_STACK=4194304 cargo test -p wie_lgt --test test_key_reach
set -euo pipefail

# Pinned to the same revision Cargo.toml takes `wipi_types` from — a moving
# upstream would silently change what the fixture is.
WIPI_REV=068312d
WIPI_URL=https://github.com/dlunch/wipi.git

root=$(cd "$(dirname "$0")/.." && pwd)
S=$(mktemp -d)
trap 'rm -rf "${S:?S unset}"' EXIT

echo "== clone $WIPI_URL @ $WIPI_REV"
git clone -q "$WIPI_URL" "$S/wipi"
git -C "$S/wipi" checkout -q "$WIPI_REV"

echo "== inject the keydraw example"
mkdir -p "$S/wipi/examples/resources/keydraw"
cat > "$S/wipi/examples/src/keydraw.rs" <<'RS'
#![cfg_attr(not(test), no_main)]
#![no_std]
extern crate alloc;

use wipi::{
    app::App,
    event::KeyCode,
    framebuffer::{Color, Framebuffer},
    println,
    wipi_main,
};

const BAR_H: i32 = 8;
const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };

pub struct KeyDrawApp {
    width: i32,
}

impl App for KeyDrawApp {
    // Repaints from scratch every frame, so the pixel count is a function of the
    // last key alone — no union-across-frames bookkeeping like the J2ME fixture.
    fn on_paint(&mut self) {
        let mut fb = Framebuffer::screen_framebuffer();
        let (w, h) = (fb.width() as i32, fb.height() as i32);
        fb.fill_rect(0, 0, w, h, BLACK);
        if self.width > 0 {
            fb.fill_rect(0, 0, self.width, BAR_H, WHITE);
        }
    }

    fn on_keydown(&mut self, key_code: KeyCode) {
        // For digits and symbols this IS the WIPI code the host delivered, so
        // the assertion pins a value rather than a proxy. The named keys are
        // negative in the WIPI space and cannot be a bar width, so they get a
        // distinct positive slot each — say so when you assert on them.
        self.width = match key_code {
            KeyCode::Key0 => 48,
            KeyCode::Key1 => 49,
            KeyCode::Key2 => 50,
            KeyCode::Key3 => 51,
            KeyCode::Key4 => 52,
            KeyCode::Key5 => 53,
            KeyCode::Key6 => 54,
            KeyCode::Key7 => 55,
            KeyCode::Key8 => 56,
            KeyCode::Key9 => 57,
            KeyCode::Hash => 35,
            KeyCode::Star => 42,
            KeyCode::Up => 101,
            KeyCode::Down => 102,
            KeyCode::Left => 103,
            KeyCode::Right => 104,
            KeyCode::Ok => 105,
            KeyCode::Back => 106,
            KeyCode::Call => 107,
            KeyCode::End => 108,
            KeyCode::Unknown(_) => 199,
        };
        println!("key:{}", self.width);
        Framebuffer::screen_framebuffer().request_repaint();
    }
}

#[wipi_main]
pub fn main() -> KeyDrawApp {
    KeyDrawApp { width: 0 }
}
RS

cat >> "$S/wipi/examples/Cargo.toml" <<'TOML'

[[bin]]
name = "keydraw"
path = "src/keydraw.rs"
TOML

cd "$S/wipi"
for carrier in ktf lgt; do
  echo "== build + archive: $carrier"
  cargo -Zbuild-std=core,alloc build -q -p examples --bin keydraw \
    --target thumbv4t-none-eabi --features "$carrier" --profile examples --no-default-features
  cargo run -q -p wipi-archiver -- "$carrier" \
    target/thumbv4t-none-eabi/examples/keydraw Clet 00000000 PD000000 ./examples/resources/keydraw \
    > "target/keydraw_$carrier.zip"
  cp "target/keydraw_$carrier.zip" "$root/test_data/keydraw_$carrier.zip"
  echo "   -> test_data/keydraw_$carrier.zip ($(wc -c < "$root/test_data/keydraw_$carrier.zip") bytes)"
done

echo "OK — fixtures rebuilt. Re-run the key-reach tests to verify."
