use alloc::{boxed::Box, string::String, vec, vec::Vec};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{Instant, System};
use wie_util::{ByteRead, ByteWrite, Result};

use crate::{
    WIPICMethodBody,
    method::{ParamConverter, ResultConverter},
};

#[async_trait::async_trait]
pub trait WIPICContext: ByteRead + ByteWrite + Send + Sync {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord>;
    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr>;
    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()>;
    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()>;
    fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord>;
    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord>;
    fn system(&mut self) -> &mut System;
    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()>;
    async fn java_property(&self, name: &str) -> Result<Option<String>>;
    async fn get_resource_size(&self, name: &str) -> Result<Option<usize>>;
    async fn read_resource(&self, name: &str) -> Result<Vec<u8>>;
    /// `pace_from`: `wie_backend::Event::guest_timer`.
    fn set_timer(&mut self, due: Instant, pace_from: u64, callback: WIPICMethodBody);
    /// Whether the title's graphics context is all words (KTF) rather than the packed record
    /// `wipi_types` describes. See `api::graphics::read_grp_ctx`.
    fn wide_graphics_context(&self) -> bool {
        false
    }
}

pub struct WIPICResult {
    pub results: Vec<WIPICWord>,
}

impl ParamConverter<WIPICWord> for WIPICWord {
    fn convert(_: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICWord {
        raw
    }
}

impl ParamConverter<WIPICIndirectPtr> for WIPICIndirectPtr {
    fn convert(_: &mut dyn WIPICContext, raw: WIPICWord) -> WIPICIndirectPtr {
        WIPICIndirectPtr(raw)
    }
}

impl ParamConverter<i32> for i32 {
    fn convert(_: &mut dyn WIPICContext, raw: WIPICWord) -> i32 {
        raw as _
    }
}

impl ResultConverter<u64> for u64 {
    fn convert(_: &mut dyn WIPICContext, result: u64) -> WIPICResult {
        WIPICResult {
            results: vec![result as u32, (result >> 32) as u32],
        }
    }
}

impl ResultConverter<WIPICWord> for WIPICWord {
    fn convert(_: &mut dyn WIPICContext, result: WIPICWord) -> WIPICResult {
        WIPICResult { results: vec![result] }
    }
}

impl ResultConverter<WIPICIndirectPtr> for WIPICIndirectPtr {
    fn convert(_: &mut dyn WIPICContext, result: WIPICIndirectPtr) -> WIPICResult {
        WIPICResult { results: vec![result.0] }
    }
}

impl ResultConverter<i32> for i32 {
    fn convert(_: &mut dyn WIPICContext, result: i32) -> WIPICResult {
        WIPICResult { results: vec![result as _] }
    }
}

impl ResultConverter<()> for () {
    fn convert(_: &mut dyn WIPICContext, _: ()) -> WIPICResult {
        WIPICResult { results: Vec::new() }
    }
}

/// Longest a spawned callback waits for the other guest threads to reach a host call — a key's cap
/// (`KEY_WAITS_FOR_GUEST_MS`, docs/report/0430).
const CALLBACK_WAITS_FOR_GUEST_MS: u32 = 250;

/// Runs a callback `WIPICContext::spawn` put on its own guest thread (clip status, socket) the way a
/// key and a paint run: not in the middle of another guest thread's code, and not sliced into by one.
/// 8b01d4ae6af4's event pump checks a busy flag and then dequeues; a clip callback that came in while
/// the main thread was sliced between the two ran the main thread's record, the main thread ran the
/// callback's, both drove the title's one script runner at once and it ended in a stack overflow
/// (docs/report/0591). The wait is capped: past it the callback runs anyway, as it did before.
pub async fn call_spawned(context: &mut dyn WIPICContext, callback: &WIPICMethodBody) -> Result<()> {
    let mut waited = 0;
    while waited < CALLBACK_WAITS_FOR_GUEST_MS && context.system().guest_others_preempted() {
        context.system().sleep(1).await;
        waited += 1;
    }
    if waited > 0 {
        tracing::debug!("spawned callback waited {waited}ms for a guest thread sliced out mid-code");
    }

    context.system().guest_hold_others(true);
    let result = callback.call(context, Box::new([])).await;
    context.system().guest_hold_others(false);

    result.map(|_| ())
}

#[cfg(test)]
pub mod test {
    use alloc::{boxed::Box, format, string::String, sync::Arc, vec::Vec};

    use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

    use wie_backend::{Instant, System};
    use wie_util::{ByteRead, ByteWrite, Result, WieError};

    use super::{WIPICContext, WIPICMethodBody};

    const TEST_MEMORY_SIZE: usize = 0x20000;
    const TEST_ALLOC_START: usize = 0x10000;
    const GLOBAL_DATA_BASE: u32 = 0x7fff0000;

    pub struct TestContext {
        memory: [u8; TEST_MEMORY_SIZE],
        global_data: [u8; 0x10000],
        last_alloc: usize,
        system: Option<System>,
        resources: Vec<(String, Vec<u8>)>,
        pub timers: Vec<(Instant, u64, WIPICMethodBody)>,
        pub calls: Vec<WIPICWord>,
        pub call_args: Vec<(WIPICWord, Vec<WIPICWord>)>,
        pub spawned: Vec<WIPICMethodBody>,
        pub wide_graphics_context: bool,
        /// Guest functions a test registers (upstream's double); an address not listed here answers 0.
        pub functions: Vec<(WIPICWord, Arc<WIPICMethodBody>)>,
    }

    impl TestContext {
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self {
            Self {
                memory: [0; TEST_MEMORY_SIZE],
                global_data: [0; 0x10000],
                last_alloc: TEST_ALLOC_START,
                system: None,
                resources: Vec::new(),
                timers: Vec::new(),
                calls: Vec::new(),
                call_args: Vec::new(),
                spawned: Vec::new(),
                wide_graphics_context: false,
                functions: Vec::new(),
            }
        }

        pub fn with_system(system: System) -> Self {
            Self {
                memory: [0; TEST_MEMORY_SIZE],
                global_data: [0; 0x10000],
                last_alloc: TEST_ALLOC_START,
                system: Some(system),
                resources: Vec::new(),
                timers: Vec::new(),
                calls: Vec::new(),
                call_args: Vec::new(),
                spawned: Vec::new(),
                wide_graphics_context: false,
                functions: Vec::new(),
            }
        }

        pub fn with_resource(mut self, name: &str, data: &[u8]) -> Self {
            self.resources.push((String::from(name), data.to_vec()));
            self
        }
    }

    #[async_trait::async_trait]
    impl WIPICContext for TestContext {
        fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
            let address = self.last_alloc;
            self.last_alloc += size as usize;

            Ok(address as WIPICWord)
        }

        fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr> {
            Ok(WIPICIndirectPtr(Self::alloc_raw(self, size)?))
        }

        fn free(&mut self, _memory: WIPICIndirectPtr) -> Result<()> {
            Ok(())
        }

        fn free_raw(&mut self, _address: WIPICWord, _size: WIPICWord) -> Result<()> {
            Ok(())
        }

        fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord> {
            Ok(memory.0)
        }

        async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord> {
            self.calls.push(address);
            self.call_args.push((address, args.to_vec()));
            let Some(callback) = self.functions.iter().find(|(ptr, _)| *ptr == address).map(|(_, f)| f.clone()) else {
                return Ok(0);
            };
            let result = callback.call(self, args.to_vec().into_boxed_slice()).await?;
            Ok(result.results[0])
        }

        fn system(&mut self) -> &mut System {
            self.system.as_mut().unwrap()
        }

        fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()> {
            self.spawned.push(callback);
            Ok(())
        }

        async fn java_property(&self, _name: &str) -> Result<Option<String>> {
            Ok(None)
        }

        async fn get_resource_size(&self, name: &str) -> Result<Option<usize>> {
            Ok(self.resources.iter().find(|(x, _)| x == name).map(|(_, data)| data.len()))
        }

        async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
            self.resources
                .iter()
                .find(|(x, _)| x == name)
                .map(|(_, data)| data.clone())
                .ok_or_else(|| WieError::FatalError(format!("Missing test resource: {name}")))
        }

        fn set_timer(&mut self, due: Instant, pace_from: u64, callback: WIPICMethodBody) {
            self.timers.push((due, pace_from, callback));
        }

        fn wide_graphics_context(&self) -> bool {
            self.wide_graphics_context
        }
    }

    /// An out-of-range access is an ERROR here, not a panic.
    ///
    /// The real backend (`ArmCore`) returns `WieError::InvalidMemoryAccess` for
    /// an address it cannot map, so a double that indexes the slice directly
    /// diverges from production exactly on the error path — which makes the
    /// fail-soft branches of this crate (every `read_null_terminated_string_bytes`
    /// whose failure is *meant* to be recoverable) untestable, and turns a guest
    /// pointer outside `TEST_MEMORY_SIZE` into a harness crash. Measured
    /// 2026-09-19: `sort_records`'s own test passes `0x13184c` — a real address
    /// from a register dump, and ~9.5x this double's memory.
    fn range(address: u32, len: usize) -> wie_util::Result<core::ops::Range<usize>> {
        let start = address as usize;
        let end = start.checked_add(len).ok_or(wie_util::WieError::InvalidMemoryAccess(address))?;
        if end > TEST_MEMORY_SIZE {
            return Err(wie_util::WieError::InvalidMemoryAccess(address));
        }
        Ok(start..end)
    }

    impl ByteWrite for TestContext {
        fn write_bytes(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
            if address >= GLOBAL_DATA_BASE {
                let offset = (address - GLOBAL_DATA_BASE) as usize;
                let global = self.global_data.get_mut(offset..offset + data.len());
                global.ok_or(wie_util::WieError::InvalidMemoryAccess(address))?.copy_from_slice(data);
            } else {
                self.memory[range(address, data.len())?].copy_from_slice(data);
            }

            Ok(())
        }
    }

    impl ByteRead for TestContext {
        fn read_bytes(&self, address: u32, result: &mut [u8]) -> wie_util::Result<usize> {
            if address >= GLOBAL_DATA_BASE {
                let offset = (address - GLOBAL_DATA_BASE) as usize;
                let global = self.global_data.get(offset..offset + result.len());
                result.copy_from_slice(global.ok_or(wie_util::WieError::InvalidMemoryAccess(address))?);
            } else {
                result.copy_from_slice(&self.memory[range(address, result.len())?]);
            }

            Ok(result.len())
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::{
        future::Future,
        pin::Pin,
        sync::atomic::{AtomicBool, AtomicU32, Ordering},
    };

    use test_utils::TestPlatform;
    use wie_backend::{AsyncCallable, System, TaskRunner};
    use wie_util::{Result, WieError};
    use wipi_types::wipic::WIPICWord;

    use crate::{MethodBody, WIPICMethodBody};

    use super::{ResultConverter, WIPICContext, WIPICResult, call_spawned, test::TestContext};

    /// Another guest thread stays sliced out mid-code for the first `left` asks; one hold flag.
    struct SlicedRunner {
        left: Arc<AtomicU32>,
        held: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl TaskRunner for SlicedRunner {
        async fn run(&self, future: Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
            future.await
        }

        fn others_preempted(&self) -> bool {
            let left = self.left.load(Ordering::Relaxed);
            self.left.store(left.saturating_sub(1), Ordering::Relaxed);
            left > 0
        }

        fn hold_others(&self, on: bool) -> bool {
            self.held.swap(on, Ordering::Relaxed)
        }
    }

    /// Records (asks left, held) when the guest callback runs.
    struct Probe {
        left: Arc<AtomicU32>,
        held: Arc<AtomicBool>,
        seen: Arc<Seen>,
    }

    #[derive(Default)]
    struct Seen {
        ran: AtomicBool,
        left: AtomicU32,
        held: AtomicBool,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for Probe {
        async fn call(&self, _: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            self.seen.ran.store(true, Ordering::Relaxed);
            self.seen.left.store(self.left.load(Ordering::Relaxed), Ordering::Relaxed);
            self.seen.held.store(self.held.load(Ordering::Relaxed), Ordering::Relaxed);
            Ok(WIPICResult { results: Vec::new() })
        }
    }

    // 8b01d4ae6af4: a clip callback ran the main thread's record from inside its sliced event pump.
    #[test]
    fn a_spawned_callback_waits_for_a_sliced_thread_and_runs_held() {
        let (left, held) = (Arc::new(AtomicU32::new(3)), Arc::new(AtomicBool::new(false)));
        let runner = SlicedRunner {
            left: left.clone(),
            held: held.clone(),
        };
        let mut system = System::new(Box::new(TestPlatform::new()), "pid", "aid", runner);
        let seen = Arc::new(Seen::default());
        let callback: WIPICMethodBody = Box::new(Probe {
            left,
            held: held.clone(),
            seen: seen.clone(),
        });

        struct Spawned(TestContext, WIPICMethodBody);
        impl AsyncCallable<Result<()>> for Spawned {
            async fn call(mut self) -> Result<()> {
                call_spawned(&mut self.0, &self.1).await
            }
        }
        system.spawn(Spawned(TestContext::with_system(system.clone()), callback));
        for _ in 0..100 {
            if seen.ran.load(Ordering::Relaxed) {
                break;
            }
            system.tick().unwrap();
        }

        assert!(seen.ran.load(Ordering::Relaxed));
        assert_eq!(seen.left.load(Ordering::Relaxed), 0, "ran only once the other thread reached a host call");
        assert!(seen.held.load(Ordering::Relaxed), "ran holding the others");
        assert!(!held.load(Ordering::Relaxed), "and let go after");
    }

    #[test]
    fn convert_u64_splits_low_and_high_words() {
        let mut context = TestContext::new();

        let result = <u64 as ResultConverter<u64>>::convert(&mut context, 0x1122_3344_5566_7788);

        assert_eq!(result.results, vec![0x5566_7788, 0x1122_3344]);
    }
}
