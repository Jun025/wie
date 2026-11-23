extern crate alloc;

// fork: the `cli` dependencies are native-only in our `Cargo.toml` (the wasm lint gate builds this
// package with default features), so on wasm32 this binary is an empty `main`.
#[cfg(not(target_arch = "wasm32"))]
mod cli;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<()> {
    cli::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {}
