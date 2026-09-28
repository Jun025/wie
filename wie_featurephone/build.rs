//! Embeds the optional soundfont prelude (spessasynth_core, bundled by
//! `scripts/build-soundfont-prelude.mjs`) into the crate as a string, next to `audio_worklet.js`.
//!
//! `scripts/build-wasm.sh` builds the bundle and passes its path in `WIE_SOUNDFONT_PRELUDE`. Every
//! other build (native `cargo test --all`, the wasm clippy gate, anyone running `cargo build` by
//! hand) has no node toolchain in the loop, so it gets an empty prelude — and an empty prelude
//! means the sink never fetches a soundfont and plays FM, exactly as before the prelude existed.
//! A published artifact without it is caught by `scripts/check-engine-contract.mjs`
//! (contract `soundfontPrelude`), not here.
use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=WIE_SOUNDFONT_PRELUDE");
    let out = PathBuf::from(env::var("OUT_DIR").expect("cargo sets OUT_DIR")).join("soundfont_prelude.js");
    let prelude = match env::var("WIE_SOUNDFONT_PRELUDE") {
        Ok(path) if !path.is_empty() => {
            println!("cargo:rerun-if-changed={path}");
            // Asked for and missing is a broken build, not a silent FM-only one.
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("WIE_SOUNDFONT_PRELUDE={path}: {error}"))
        }
        _ => String::new(),
    };
    fs::write(out, prelude).expect("write soundfont_prelude.js into OUT_DIR");
}
