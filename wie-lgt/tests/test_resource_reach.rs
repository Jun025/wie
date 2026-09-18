//! Does an LGT guest actually get its bundled RESOURCE back?
//!
//! The KTF sibling's header explains why this axis exists at all — four host-side
//! resource sites were reached by nothing in the repo, and a planted `panic!()` at
//! all four left `cargo test --all` green. **Two of those four are LGT's**
//! (`wie-lgt/.../wipi_c/context.rs`: `get_resource_size`, `read_resource`), and they
//! are the pair this file covers. Until it was restored, they were unconditionally
//! unmeasured again: the browser round-trip's Scenario F-res reaches them, but that
//! job is behind `engine-contract.yml`'s paths filter.
//!
//! ── Why this file had to be written twice ────────────────────────────────────
//! Same history as `test_key_reach.rs` next to it: created alongside the KTF one,
//! dropped by the 2026-09-16 base swap, never adjudicated. That file's header
//! carries the evidence; it is not repeated here.
//!
//! **Both numbers are load-bearing, and that is the point of asserting the pair.**
//! `size` is what the host's `get_resource_size` returned; the sum can only be
//! computed from bytes `read_resource` actually delivered. Asserting only the size
//! would leave the second site as unmeasured as it was before.
//!
//! The payload is `WIE-RES-1` (9 ASCII bytes, 87+73+69+45+82+69+83+45+49 = 602),
//! written by `scripts/make-wipi-keydraw-fixture.sh` — the same line that produces
//! the KTF fixture, so the two carriers share one expected value by construction.
//! Change it there and in both test files together.
//!
//! ★Bounded by **ticks**, not wall clock — see `test_key_reach.rs`'s header for the
//! measurement behind that choice. Do not convert it to a timeout.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use test_utils::{TestPlatform, TestPlatformEvent, guest_line_complete};
use wie_backend::{Emulator, Options, extract_zip};
use wie_lgt::LgtEmulator;
use wie_util::Result;

/// The fixture never exits (it waits for keys), so the loop is bounded. The read
/// happens at boot, well before this budget — the same budget `test_key_reach`
/// uses to reach its first keydown.
const BOOT_TICKS: usize = 400_000;

/// `res:<size>:<sum>` for the committed payload. See the module header.
const EXPECTED: &str = "res:9:602";

#[test]
pub fn bundled_resource_reaches_the_lgt_guest() -> Result<()> {
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let exited = Arc::new(AtomicBool::new(false));

    let stdout_clone = stdout.clone();
    let exited_clone = exited.clone();
    let platform = Box::new(TestPlatform::with_event_handler(move |event| match event {
        TestPlatformEvent::Stdout(buf) => stdout_clone.lock().unwrap().extend(buf),
        TestPlatformEvent::Exit => exited_clone.store(true, Ordering::SeqCst),
    }));

    let archive = extract_zip(include_bytes!("../../test_data/keydraw_lgt.zip"))?;
    let mut emulator = LgtEmulator::from_archive(
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
        // Wait for a COMPLETE line, not the `res:` prefix — see the KTF sibling.
        if guest_line_complete(&seen, "res:") {
            break;
        }
    }

    // `res:err` is a real outcome the guest prints rather than panicking, so name it
    // separately: "the call failed" and "the call never happened" are different bugs.
    assert!(
        !seen.contains("res:err"),
        "the LGT guest reached the resource API but it FAILED — guest stdout was {seen:?}"
    );
    assert!(
        seen.contains(EXPECTED),
        "the LGT guest did not read res.bin as {EXPECTED} (size from get_resource_size, sum from read_resource) — guest stdout was {seen:?}"
    );

    Ok(())
}
