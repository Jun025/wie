//! Dropping the emulator frees it. Java objects that reference each other — a static holding an
//! instance of its own class, `Writer.lock = this`, `Display` ↔ its current `Canvas` — outlive the
//! JVM, since the `jvm` crate frees an object only when its last `Arc` goes, and their classes hold
//! the `System` and through it the platform. Until the JVM cut those cycles on drop, every boot
//! leaked ~0.5 MiB — on the web, per title switch and per import check.

use std::sync::Arc;

use test_utils::TestPlatform;
use wie_backend::{Emulator, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_util::Result;

#[test]
fn dropped_emulator_frees_its_platform() -> Result<()> {
    let archive = extract_zip(include_bytes!("../../test_data/draw_j2me.zip"))?;
    let jar = archive["draw_j2me.jar"].clone();

    // Mid-boot (a title switched away from) and after the guest painted and settled.
    for ticks in [1, 300] {
        let probe = Arc::new(());
        let held = probe.clone();
        let platform = TestPlatform::with_event_handler(move |_| {
            let _ = &held;
        });
        let mut emulator = J2MEEmulator::from_jar(Box::new(platform), "draw_j2me.jar", jar.clone())?;
        for _ in 0..ticks {
            emulator.tick()?;
        }
        drop(emulator);
        assert_eq!(Arc::strong_count(&probe), 1, "the platform outlived its emulator after {ticks} ticks");
    }
    Ok(())
}
