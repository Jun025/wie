//! Does a key press actually REACH an LGT guest?
//!
//! `test_helloworld` proves the LGT boot path; it says nothing about input,
//! because that fixture prints and exits before any key can arrive. The browser
//! round-trip covers this with pixels (Scenario F), but that job sits behind
//! `engine-contract.yml`'s `dorny/paths-filter`, so a diff that touches no
//! filtered path never runs it. This test is the unconditional half: `cargo test
//! --all` runs it on every matrix leg.
//!
//! ── Why this file had to be written twice ────────────────────────────────────
//! It existed once. `b52ed661` created `wie_ktf/tests/test_key_reach.rs` and
//! `wie_lgt/tests/test_key_reach.rs` together — that round's reply said it covered
//! "두 경로 다 … 부분 완료가 아니라 전건". The 2026-09-16 base swap
//! (`wie_lgt` → `wie-lgt`) carried the KTF pair across and left the LGT pair
//! behind: the swap's own rounds fix `wie-ktf/tests/test_key_reach.rs` **by name**
//! (`8ff418ed`, and again in `docs/report/0123`) and never mention the LGT one, and
//! the orphan census that followed (`docs/report/0153`) lists four leftover files,
//! none of them these. So the pair was neither carried nor adjudicated — it simply
//! stopped existing, and `keydraw_lgt.zip` was left with no unconditional reader.
//! Restored from the KTF file's current shape rather than from `b52ed661`, because
//! that original predates two measured fixes the KTF side has since received.
//!
//! ── The evidence axis, and why it is load-immune ─────────────────────────────
//! The guest's own **stdout**, not canvas pixels: `TestScreen` retains no
//! framebuffer, and stdout lets the assertion name the exact integer the guest was
//! handed. `test_data/keydraw_lgt.zip` prints `key:<code>` on keydown; it is built
//! by `scripts/make-wipi-keydraw-fixture.sh` from the same guest source as the KTF
//! fixture, so `KeyCode::Key5 => 53` is one table shared by both carriers.
//!
//! ★The loop below is bounded by **ticks**, not by wall clock, and that is
//! load-bearing. The same fixture run through `wie_validate --inject` fails under
//! load — measured 2026-09-18 at loadavg 180, `paints` 12-22 against an idle range
//! of 48-55 — because that path has a fixed wall-clock deadline that a busy machine
//! eats. A tick budget cannot be starved that way: the host advances the guest the
//! same number of steps whether the machine is idle or not. Do not "improve" this
//! into a timeout, and do not assert on `paints`; the reasoning for the latter is
//! in the KTF file's header and applies unchanged here.

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};

use test_utils::{TestPlatform, TestPlatformEvent, guest_line_complete};
use wie_backend::{Emulator, Event, KeyCode, Options, extract_zip};
use wie_lgt::LgtEmulator;
use wie_util::Result;

/// The fixture never exits, so the loop here is bounded.
const DELIVER_TICKS: usize = 400_000;

#[test]
pub fn key_press_reaches_the_lgt_guest() -> Result<()> {
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

    // Queued before the first tick on purpose: the event queue holds it until
    // the guest is up, which keeps this test off a "how long is boot?" guess.
    // NUM5 is deliberate: its WIPI code is 53, so the guest printing "key:53"
    // pins the value end to end instead of just "something arrived".
    emulator.handle_event(Event::Keydown(KeyCode::NUM5));

    let mut seen = String::new();
    for _ in 0..DELIVER_TICKS {
        emulator.tick()?;
        seen = String::from_utf8_lossy(&stdout.lock().unwrap().clone()).into_owned();
        // `contains("key:")` is NOT enough — see `guest_line_complete`'s doc
        // comment, which both stdout-polling tests got wrong independently.
        if guest_line_complete(&seen, "key:") {
            break;
        }
    }

    assert!(
        seen.contains("key:53"),
        "NUM5 did not reach the LGT guest as WIPI code 53 — guest stdout was {seen:?}"
    );

    Ok(())
}
