//! `log`'s compile-time ceiling for the native hosts (`wie_cli/Cargo.toml`): release builds drop trace
//! only, debug builds drop nothing. `cargo test --all` runs the debug half; the release half is
//! `cargo test --release -p wie_cli --test log_static_level`.
#![cfg(not(target_arch = "wasm32"))]

#[test]
fn release_builds_compile_out_log_trace_only() {
    let expected = if cfg!(debug_assertions) {
        log::LevelFilter::Trace
    } else {
        log::LevelFilter::Debug
    };
    assert_eq!(log::STATIC_MAX_LEVEL, expected);
}
