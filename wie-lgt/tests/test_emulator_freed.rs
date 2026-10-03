//! Dropping the emulator frees it. The core, the system and the JVM hold clones of one another
//! (SVC handler contexts, tasks, the JVM's function table, the guest-root scan's global reference);
//! until `Drop for LgtEmulator` cut those cycles, every boot leaked one core — on the web, one per
//! title switch and per import check, until the tab was reloaded.
//!
//! Alone in its file on purpose: `live_cores` counts the whole process, and other tests in the
//! same binary boot cores in parallel.

use test_utils::TestPlatform;
use wie_backend::{Emulator, Options, extract_zip};
use wie_core_arm::live_cores;
use wie_lgt::LgtEmulator;
use wie_util::Result;

fn boot(ticks: usize) -> Result<()> {
    let archive = extract_zip(include_bytes!("data/helloworld_lgt.zip"))?;
    let options = Options {
        enable_gdbserver: false,
        profile: None,
    };
    let mut emulator = LgtEmulator::from_archive(Box::new(TestPlatform::new()), archive, options)?;
    for _ in 0..ticks {
        emulator.tick()?;
    }
    Ok(())
}

#[test]
fn dropped_emulator_frees_its_core() -> Result<()> {
    // Mid-boot (a title switched away from) and after the guest ran its course.
    for ticks in [1, 50] {
        boot(ticks)?;
        assert_eq!(live_cores(), 0, "a core outlived its emulator after {ticks} ticks");
    }
    Ok(())
}
