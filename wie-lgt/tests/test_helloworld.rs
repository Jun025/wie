//! LGT entrypoint-discovery cases.
//!
//! ── Where `test_helloworld_jar_named_application` went ───────────────────────
//! A 2026-09-19 machine audit listed that name as "lost in the crate rename".
//! Measured rather than assumed: **its assertion is not lost** — it is inside
//! `test_helloworld` below, which renames the fixture's `00000000.jar` to
//! `application.jar` before booting. Reverting discovery to the old
//! `format!("{aid}.jar")` form reddens `test_helloworld`, so that case is covered.
//! The audit diffed *function names*, and a name can disappear into another test.
//!
//! ── What was actually missing, and is added below ────────────────────────────
//! The old pair was deliberately asymmetric: one case renamed the jar (sees a
//! name-based regression) and one left it AID-named (cannot). Today **both**
//! existing cases move the jar — `application.jar` and `P/00000000.jar` — so the
//! shape an archive actually ships in was not booted by anything.
//! `test_helloworld_unmodified_archive` restores that baseline.
//!
//! ★Its unique power is narrow, and saying so is the honest version: under
//! content-based discovery it passes whenever the others do. It fails alone only
//! for a regression specific to the untouched path — e.g. a discovery rule that
//! skipped AID-named jars. It is here because nothing else boots the fixture as
//! shipped, not because it is a strong net.

use std::{
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

#[test]
pub fn test_helloworld() -> Result<()> {
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

    let mut archive = extract_zip(include_bytes!("data/helloworld_lgt.zip"))?;
    assert_eq!(LgtEmulator::archive_id(&archive).as_deref(), Some("PD000000"));
    let jar = archive.remove("00000000.jar").unwrap();
    archive.insert("application.jar".into(), jar);
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

    let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
    assert_eq!(stdout_str, "Hello, world!");

    Ok(())
}

#[test]
pub fn test_helloworld_jar_under_p_prefix() -> Result<()> {
    // Archives can store the entrypoint jar under a `P/` directory. `load()` strips that prefix
    // from every filesystem key, so the classpath name must be stripped the same way, or the
    // entrypoint fails to load with "Missing binary.mod in P/...".
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

    let mut archive = extract_zip(include_bytes!("data/helloworld_lgt.zip"))?;
    let jar = archive.remove("00000000.jar").unwrap();
    archive.insert("P/00000000.jar".into(), jar);
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

    let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
    assert_eq!(stdout_str, "Hello, world!");

    Ok(())
}

#[test]
pub fn test_helloworld_unmodified_archive() -> Result<()> {
    // The archive exactly as it ships: the entrypoint jar is still `00000000.jar`,
    // which is also the AID. Every other case in this file renames or re-parents
    // it first, so without this one nothing exercises the untouched shape.
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

    let archive = extract_zip(include_bytes!("data/helloworld_lgt.zip"))?;
    assert!(
        archive.contains_key("00000000.jar"),
        "the fixture must still ship its jar AID-named, or this case is testing nothing"
    );
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

    let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
    assert_eq!(stdout_str, "Hello, world!");

    Ok(())
}
