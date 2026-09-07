//! Boots a J2ME guest inside `cargo test --all` — the axis that lived only in the
//! browser job until now.
//!
//! Why this exists, measured not assumed. On 2026-09-04 a RustJava pin bump was
//! green on all four gates while `draw_j2me.jar` died with `NoClassDefFoundError`
//! on the first tick (`AGENTS.md` §Definition of Done records it). Nothing in
//! `cargo test --all` booted a J2ME guest — measured on this branch's base:
//! ten `*/tests/*.rs` files, zero of them naming `wie_j2me`. The net that does
//! cover it is `scripts/contract-roundtrip.mjs` Scenario C, and it runs in ONE
//! browser job; if that job stops running the axis disappears with it. This file
//! is the second net, and it is deliberately NOT a replacement — the browser job
//! is untouched.
//!
//! What it asserts, and why that and not pixels: `TestScreen` keeps no
//! framebuffer, so the pixel question stays the browser's (Scenario C asserts real
//! canvas pixels). What is answerable here is the coarser question the incident
//! actually lost — *did a frame ever get composed* — so the assertion is a paint
//! COUNT, not "nothing threw". "Nothing threw" would also pass for a guest that
//! dies before painting, which is precisely the 2026-09-04 shape.
//!
//! ── The fixture is a zip that holds the jar, and that is the repo's convention ──
//! `scripts/make-draw-fixture.mjs` builds `test_data/draw_j2me.jar`, and that jar
//! is deliberately NOT committed: `*.jar` is git-ignored and
//! `scripts/audit-no-leak.sh:128` rejects any TRACKED `*.jar` (Constraint 9). So
//! this test cannot `include_bytes!` it. The committed fixture is therefore
//! `test_data/draw_j2me.zip`, a zip whose single entry is that jar — exactly the
//! shape of the four fixtures already committed here (`keydraw_ktf.zip` holds
//! `00000000.jar` inside it). No blocklist is loosened and no file is renamed to
//! duck one; the archive is opened with the same `extract_zip` every other guest
//! test already uses.
//! Regenerate: `node scripts/make-draw-fixture.mjs`, then re-zip the jar. The
//! generator is deterministic (measured: identical md5 across runs), but verify a
//! regeneration by re-running this test, not by diffing the zip — the same rule
//! `scripts/make-wipi-keydraw-fixture.sh` states for its own fixtures.

use std::sync::{Arc, Mutex, atomic::Ordering};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Event, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_util::Result;

/// The guest paints from `DrawCanvas::paint`, so one composed frame proves the
/// whole chain reached the screen: jar -> class load -> MIDlet startApp ->
/// setCurrent -> the emulator's paint loop.
#[test]
pub fn j2me_guest_boots_and_paints() -> Result<()> {
    let platform = TestPlatform::new();
    let paints = platform.paint_counter();
    let redraw = platform.redraw_flag();

    let archive = extract_zip(include_bytes!("../../test_data/draw_j2me.zip"))?;
    let jar = archive
        .get("draw_j2me.jar")
        .expect("test_data/draw_j2me.zip must hold draw_j2me.jar — regenerate it with scripts/make-draw-fixture.mjs")
        .clone();

    let mut emulator = J2MEEmulator::from_jar(Box::new(platform), "draw_j2me.jar", jar)?;

    // The fixture never exits — it paints and waits. Stop at the first composed
    // frame rather than burning the whole budget; the cap is only a hang backstop.
    // ★The `Event::Redraw` reply is REQUIRED, not decoration: the core composes a
    // frame only in response to the redraw it asked for. Without this the loop
    // sees 0 paints forever (measured). Same shape as wie_validate's run loop.
    let mut ticks = 0;
    while paints.load(Ordering::SeqCst) == 0 && ticks < 10_000 {
        emulator.tick()?;
        if redraw.swap(false, Ordering::SeqCst) {
            emulator.handle_event(Event::Redraw);
        }
        ticks += 1;
    }

    assert!(
        paints.load(Ordering::SeqCst) > 0,
        "the J2ME guest never composed a frame in {ticks} ticks — it booted but nothing reached the screen"
    );

    Ok(())
}

/// The *failure* branches of the same `Image.createImage(String)` call the test
/// above exercises on its success path.
///
/// Why a second test and not two more asserts in the first: this one reads guest
/// stdout, which needs `TestPlatform::with_event_handler`, and the paint test
/// deliberately takes the plain constructor. Keeping them apart also keeps the
/// failure diagnostics separate — "never painted" and "wrong exception type" are
/// different bugs.
///
/// What is actually locked here is the TYPE the guest receives, not just "it did
/// not crash". `scripts/make-draw-fixture.mjs` catches each branch NARROWLY
/// (`java/io/IOException` for the absent name, `java/lang/IllegalArgumentException`
/// for the undecodable bytes — both read from `wie_midp/.../lcdui/image.rs`, not
/// guessed). So there are two independent ways this test goes red:
///   • the branch throws a DIFFERENT type -> the narrow handler does not catch,
///     the throw leaves `startApp`, the boot aborts, and no frame is ever painted;
///   • the branch stops throwing at all -> control falls through the `goto`, the
///     marker is never printed, and the `contains` below fails.
/// Before this test, both of those were silent: nothing observed either branch.
#[test]
pub fn j2me_createimage_failure_branches_report_expected_exceptions() -> Result<()> {
    let stdout = Arc::new(Mutex::new(Vec::<u8>::new()));
    let sink = stdout.clone();
    let platform = TestPlatform::with_event_handler(move |event| {
        if let TestPlatformEvent::Stdout(buf) = event {
            sink.lock().unwrap().extend_from_slice(&buf);
        }
    });
    let paints = platform.paint_counter();
    let redraw = platform.redraw_flag();

    let archive = extract_zip(include_bytes!("../../test_data/draw_j2me.zip"))?;
    let jar = archive
        .get("draw_j2me.jar")
        .expect("test_data/draw_j2me.zip must hold draw_j2me.jar — regenerate it with scripts/make-draw-fixture.mjs")
        .clone();

    let mut emulator = J2MEEmulator::from_jar(Box::new(platform), "draw_j2me.jar", jar)?;

    // Same loop shape as the paint test: both markers are printed by `startApp`,
    // which runs before the first frame, so stopping at the first paint is enough
    // and keeps this test as cheap as that one.
    let mut ticks = 0;
    while paints.load(Ordering::SeqCst) == 0 && ticks < 10_000 {
        emulator.tick()?;
        if redraw.swap(false, Ordering::SeqCst) {
            emulator.handle_event(Event::Redraw);
        }
        ticks += 1;
    }

    let out = String::from_utf8_lossy(&stdout.lock().unwrap().clone()).into_owned();
    assert!(
        out.contains("imgerr:missing"),
        "createImage on a name absent from the jar did not surface as java.io.IOException to the guest \
         (expected the marker `imgerr:missing` on stdout after {ticks} ticks; guest stdout was {out:?})"
    );
    assert!(
        out.contains("imgerr:broken"),
        "createImage on bytes that decode to nothing did not surface as java.lang.IllegalArgumentException \
         to the guest (expected the marker `imgerr:broken` on stdout after {ticks} ticks; guest stdout was {out:?})"
    );

    Ok(())
}
