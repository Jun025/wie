use alloc::boxed::Box;
use core::{future::Future, pin::Pin};

use wie_util::Result;

#[async_trait::async_trait]
pub trait TaskRunner: Sync + Send {
    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()>;

    /// Whether another guest thread is suspended mid-code rather than at a host call. Only a
    /// runner whose guest threads the host can slice reports it.
    fn others_preempted(&self) -> bool {
        false
    }

    /// While `on`, the current guest thread's code runs as one piece against the others' up to its
    /// next blocking host call.
    fn hold_others(&self, _on: bool) {}
}

pub struct DefaultTaskRunner;

#[async_trait::async_trait]
impl TaskRunner for DefaultTaskRunner {
    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        future.await
    }
}
