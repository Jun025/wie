use alloc::{boxed::Box, vec::Vec};
use core::{future::Future, pin::Pin};

pub type RecordId = u32;

#[async_trait::async_trait]
pub trait Database: Send {
    async fn next_id(&self) -> RecordId;
    async fn add(&mut self, data: &[u8]) -> RecordId;
    async fn get(&self, id: RecordId) -> Option<Vec<u8>>;
    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool;
    async fn delete(&mut self, id: RecordId) -> bool;

    async fn get_record_ids(&self) -> Vec<RecordId>;
}

#[async_trait::async_trait]
pub trait DatabaseRepository {
    /// Create an empty store if it does not already exist.
    async fn create(&self, name: &str, app_id: &str);
    /// Access records, creating the store if it is missing.
    /// fork: upstream (#1575) made this non-creating and added `create`. Our hosts and
    /// `test-utils` keep creating here: our `wie-wipi-c` database slots open a store to create it,
    /// and what a guest then sees from `exists` is what the support list was measured on.
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn Database>;
    async fn exists(&self, name: &str, app_id: &str) -> bool;
    async fn delete(&self, name: &str, app_id: &str) -> bool;
    /// Returns the bytes occupied by all databases owned by `app_id`.
    /// fork: upstream dropped this (#1575); our `wie-wipi-c` database slots still report free space
    /// from it. Hosts that do not track it report 0 — everything free. Written out by hand: a
    /// default `async fn` here would need `Self: Sync` and could not be called on `dyn`.
    fn usage<'a, 'b, 'c>(&'a self, _app_id: &'b str) -> Pin<Box<dyn Future<Output = u64> + Send + 'c>>
    where
        'a: 'c,
        'b: 'c,
        Self: 'c,
    {
        Box::pin(async { 0 })
    }
}
