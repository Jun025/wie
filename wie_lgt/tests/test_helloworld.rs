//! The second test here is the only thing that holds the LGT entrypoint-name contract.
//!
//! `LgtEmulator::from_archive` used to build the entrypoint name as `format!("{}.jar", aid)`, so
//! the container name and the AID had to agree — upstream `wie-lgt/src/emulator.rs:50-53` finds it
//! by content instead (any `*.jar` whose zip holds `binary.mod`) and its own
//! `wie-lgt/tests/test_helloworld.rs:34-35` renames the fixture's `00000000.jar` to
//! `application.jar` to prove exactly that. Reverting the discovery to the `format!` form reddens
//! `test_helloworld_jar_named_application` and leaves `test_helloworld` green, which is the whole
//! point: the fixture's jar happens to be AID-named, so the AID-named test cannot see the break.

use std::{
    collections::BTreeMap,
    sync::Arc,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use test_utils::{TestPlatform, TestPlatformEvent};
use wie_backend::{Emulator, Options, extract_zip};
use wie_lgt::LgtEmulator;
use wie_util::Result;

fn run_to_exit(archive: BTreeMap<String, Vec<u8>>) -> Result<String> {
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let exited = Arc::new(AtomicBool::new(false));

    let stdout_clone = stdout.clone();
    let exited_clone = exited.clone();
    let event_handler = move |event| match event {
        TestPlatformEvent::Stdout(buf) => {
            stdout_clone.lock().unwrap().extend(buf);
        }
        TestPlatformEvent::Exit => {
            exited_clone.store(true, Ordering::SeqCst);
        }
    };

    let platform = Box::new(TestPlatform::with_event_handler(event_handler));

    let mut emulator = LgtEmulator::from_archive(
        platform,
        archive,
        Options {
            enable_gdbserver: false,
            profile: None,
        },
    )?;

    while !exited.load(Ordering::SeqCst) {
        emulator.tick()?;
    }

    Ok(String::from_utf8(stdout.lock().unwrap().clone()).unwrap())
}

#[test]
pub fn test_helloworld() -> Result<()> {
    let archive = extract_zip(include_bytes!("../../test_data/helloworld_lgt.zip"))?;

    assert_eq!(run_to_exit(archive)?, "Hello, world!");

    Ok(())
}

#[test]
pub fn test_helloworld_jar_named_application() -> Result<()> {
    let mut archive = extract_zip(include_bytes!("../../test_data/helloworld_lgt.zip"))?;
    let jar = archive.remove("00000000.jar").expect("fixture must hold 00000000.jar");
    archive.insert("application.jar".into(), jar);

    assert_eq!(run_to_exit(archive)?, "Hello, world!");

    Ok(())
}

#[test]
pub fn test_helloworld_jar_under_p_prefix() -> Result<()> {
    // `load()` strips `P/` from every filesystem key, so an archive holding its entrypoint as
    // `P/<name>.jar` boots only if the classpath name is stripped the same way — this test holds
    // that agreement (the committed fixtures are all top-level, so they cannot see it break).
    let mut archive = extract_zip(include_bytes!("../../test_data/helloworld_lgt.zip"))?;
    let jar = archive.remove("00000000.jar").expect("fixture must hold 00000000.jar");
    archive.insert("P/00000000.jar".into(), jar);

    assert_eq!(run_to_exit(archive)?, "Hello, world!");

    Ok(())
}
