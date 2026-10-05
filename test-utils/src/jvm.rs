use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};

use jvm::{Jvm, Result as JvmResult};

use wie_backend::{DefaultTaskRunner, Platform, System};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation, WieJavaClassProto};
use wie_util::{Result, WieError};

use crate::{TestPlatform, TestPlatformEvent};

// TODO macro?
pub fn run_jvm_test<T, F>(protos: Box<[Box<[WieJavaClassProto]>]>, func: T) -> Result<()>
where
    T: FnOnce(Jvm) -> F + Send + 'static,
    F: Future<Output = JvmResult<()>> + Send,
{
    run_jvm_test_with_system(protos, Box::new(TestPlatform::new()), move |jvm, _system| func(jvm))
}

pub fn run_jvm_test_with_system<T, F>(protos: Box<[Box<[WieJavaClassProto]>]>, platform: Box<dyn Platform>, func: T) -> Result<()>
where
    T: FnOnce(Jvm, System) -> F + Send + 'static,
    F: Future<Output = JvmResult<()>> + Send,
{
    let mut system = System::new(platform, "", "", DefaultTaskRunner);

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();
    let system_clone = system.clone();

    system.spawn(async move || {
        let jvm = JvmSupport::new_jvm(&system_clone, None, protos, &[], RustJavaJvmImplementation).await?;
        func(jvm, system_clone).await.unwrap();

        done_clone.store(true, Ordering::Relaxed);

        Ok::<_, WieError>(())
    });

    loop {
        system.tick()?;
        if done.load(Ordering::Relaxed) {
            break;
        }
    }

    Ok(())
}

/// Runs `func` on a guest thread until the guest exits, and reports whether `func` came back.
/// A guest exit does not return to the guest (`System::exit_from_guest`), so a test of an exit path
/// cannot wait for `func` to finish the way [`run_jvm_test_with_system`] does. Panics if the guest
/// has not exited after `ticks` ticks.
pub fn run_jvm_test_until_exit<T, F>(protos: Box<[Box<[WieJavaClassProto]>]>, ticks: usize, func: T) -> Result<bool>
where
    T: FnOnce(Jvm) -> F + Send + 'static,
    F: Future<Output = JvmResult<()>> + Send,
{
    let exited = Arc::new(AtomicBool::new(false));
    let exited_clone = exited.clone();
    let platform = TestPlatform::with_event_handler(move |event| {
        if matches!(event, TestPlatformEvent::Exit) {
            exited_clone.store(true, Ordering::SeqCst);
        }
    });
    let mut system = System::new(Box::new(platform), "", "", DefaultTaskRunner);

    let returned = Arc::new(AtomicBool::new(false));
    let returned_clone = returned.clone();
    let system_clone = system.clone();
    system.spawn(async move || {
        let jvm = JvmSupport::new_jvm(&system_clone, None, protos, &[], RustJavaJvmImplementation).await?;
        func(jvm).await.unwrap();
        returned_clone.store(true, Ordering::SeqCst);

        Ok::<_, WieError>(())
    });

    for _ in 0..ticks {
        system.tick()?;
        if exited.load(Ordering::SeqCst) {
            // A few more: a guest thread that wrongly comes back gets the chance to.
            for _ in 0..10 {
                system.tick()?;
            }
            return Ok(returned.load(Ordering::SeqCst));
        }
    }
    panic!("the guest did not exit within {ticks} ticks");
}
