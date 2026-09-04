//! Does a key press actually REACH a KTF guest?
//!
//! `test_helloworld` proves the KTF boot path; it says nothing about input,
//! because that fixture prints and exits before any key can arrive. The
//! browser round-trip's Scenario D asserts key delivery for J2ME only — the
//! WIPI path is different code (Clet event dispatch, not the MIDP event queue),
//! so nothing covered it.
//!
//! Evidence axis here is the guest's own **stdout**, not canvas pixels: the
//! test harness's `TestScreen` does not retain a framebuffer, and stdout lets
//! the assertion name the exact integer the guest was handed rather than a
//! pixel count. `test_data/keydraw_ktf.zip` prints `key:<code>` on keydown
//! (and paints a bar of that width, for the browser axis). It is built from
//! `scripts/make-wipi-keydraw-fixture.sh`; no game files are involved.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Event, KeyCode, Options, extract_zip};
use wie_ktf::KtfEmulator;
use wie_util::Result;

/// The fixture never exits, so the loop here is bounded.
const DELIVER_TICKS: usize = 400_000;

#[test]
pub fn key_press_reaches_the_ktf_guest() -> Result<()> {
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

    // Queued before the first tick on purpose: the event queue holds it until
    // the guest is up, which keeps this test off a "how long is boot?" guess.
    // NUM5 is deliberate: its WIPI code is 53, so the guest printing "key:53"
    // pins the value end to end instead of just "something arrived".
    emulator.handle_event(Event::Keydown(KeyCode::NUM5));

    let mut seen = String::new();
    for _ in 0..DELIVER_TICKS {
        emulator.tick()?;
        seen = String::from_utf8_lossy(&stdout.lock().unwrap().clone()).into_owned();
        if seen.contains("key:") {
            break;
        }
    }

    assert!(
        seen.contains("key:53"),
        "NUM5 did not reach the KTF guest as WIPI code 53 — guest stdout was {seen:?}"
    );

    Ok(())
}
