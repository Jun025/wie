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
}

pub struct DefaultTaskRunner;

#[async_trait::async_trait]
impl TaskRunner for DefaultTaskRunner {
    async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        future.await
    }
}
