//! Does a KTF guest actually get its bundled RESOURCE back?
//!
//! Until this test the answer was unknown, not "yes": the four host-side resource
//! sites — `wie_ktf/.../wipi_c/context.rs` and `wie_lgt/.../wipi_c/context.rs`, two
//! each (`get_resource_size`, `read_resource`) — were reached by nothing in the
//! repo. Measured 2026-09-06: a planted `panic!()` at all four left
//! `cargo test --all` at **150 passed / 0 failed** and every fixture at PASS,
//! because the committed fixtures shipped an EMPTY resources directory. Code that
//! is never executed is not "working"; it is unmeasured.
//!
//! Evidence axis is the guest's own stdout, same as `test_key_reach` and for the
//! same reason (the harness's `TestScreen` keeps no framebuffer). The fixture reads
//! `res.bin` at boot and prints `res:<size>:<byte-sum>`.
//!
//! **Both numbers are load-bearing, and that is the point of asserting the pair.**
//! `size` is what `MC_knlGetResourceID` returned, which is the host's
//! `get_resource_size`; the sum can only be computed from bytes `MC_knlGetResource`
//! actually delivered, which is the host's `read_resource`. Asserting only the size
//! would leave the second site as unmeasured as it was before.
//!
//! The payload is `WIE-RES-1` (9 ASCII bytes, 87+73+69+45+82+69+83+45+49 = 602),
//! written by `scripts/make-wipi-keydraw-fixture.sh`. Change it there and here
//! together — the script says so at the line that writes the file.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Options, extract_zip};
use wie_ktf::KtfEmulator;
use wie_util::Result;

/// The fixture never exits (it waits for keys), so the loop is bounded. The read
/// happens at boot, well before this budget — the same budget `test_key_reach`
/// uses to reach its first keydown.
const BOOT_TICKS: usize = 400_000;

/// `res:<size>:<sum>` for the committed payload. See the module header.
const EXPECTED: &str = "res:9:602";

#[test]
pub fn bundled_resource_reaches_the_ktf_guest() -> Result<()> {
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let exited = Arc::new(AtomicBool::new(false));

    let stdout_clone = stdout.clone();
    let exited_clone = exited.clone();
    let platform = Box::new(TestPlatform::with_event_handler(move |event| match event {
        TestPlatformEvent::Stdout(buf) => stdout_clone.lock().unwrap().extend(buf),
        TestPlatformEvent::Exit => exited_clone.store(true, Ordering::SeqCst),
    }));

    let archive = extract_zip(include_bytes!("../../test_data/keydraw_ktf.zip"))?;
    let mut emulator = KtfEmulator::from_archive(
        platform,
        archive,
        Options {
            enable_gdbserver: false,
            profile: None,
        },
    )?;

    let mut seen = String::new();
    for _ in 0..BOOT_TICKS {
        emulator.tick()?;
        seen = String::from_utf8_lossy(&stdout.lock().unwrap().clone()).into_owned();
        if seen.contains("res:") {
            break;
        }
    }

    // `res:err` is a real outcome the guest prints rather than panicking, so name it
    // separately: "the call failed" and "the call never happened" are different bugs.
    assert!(
        !seen.contains("res:err"),
        "the KTF guest reached the resource API but it FAILED — guest stdout was {seen:?}"
    );
    assert!(
        seen.contains(EXPECTED),
        "the KTF guest did not read res.bin as {EXPECTED} (size from get_resource_size, sum from read_resource) — guest stdout was {seen:?}"
    );

    Ok(())
}
