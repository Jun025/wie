use alloc::{boxed::Box, collections::BTreeMap, format, vec, vec::Vec};

use jvm::{
    Jvm,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};
use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{AsyncCallable, Event, Instant, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, write_generic};
use wie_wipi_c::{WIPICContext, WIPICMethodBody};

// `alloc`/`free` live outside the impl so the free(NULL) guard can be tested with a bare
// `ArmCore` — building an `LgtWIPICContext` needs a full `Jvm`, which is why the guard went
// untested (and silently vanished in the base swap #161) the first time.
fn alloc_indirect(core: &mut ArmCore, size: WIPICWord) -> Result<WIPICIndirectPtr> {
    let address = Allocator::alloc(core, size + size_of::<WIPICWord>() as WIPICWord)?;
    write_generic(core, address, size)?;

    Ok(WIPICIndirectPtr(address + size_of::<WIPICWord>() as WIPICWord))
}

fn free_indirect(core: &mut ArmCore, memory: WIPICIndirectPtr) -> Result<()> {
    // Freeing a null handle is a defined no-op (C `free(NULL)` / `MC_knlFree(NULL)`).
    // 메탈슬러그 서바이벌 calls `MC_grpDestroyOffScreenFrameBuffer(0)` at boot; without this the
    // `- 4` below underflows and panics the host. Pre-swap `0942b0fb` had this guard; the upstream
    // base swap (#161) replaced the file and dropped it.
    if memory.0 == 0 {
        return Ok(());
    }

    let base_address = memory.0 - size_of::<WIPICWord>() as WIPICWord;

    let size: WIPICWord = read_generic(core, base_address)?;
    Allocator::free(core, base_address, size + size_of::<WIPICWord>() as WIPICWord)
}

// A Clet title's frames never reach MIDP `Display`'s paint, which is where garbage is collected: it
// draws through `MC_grpFlushLcd` from its own `MC_knlSetTimer` callback. 4fdbd64c9fbd's collections
// stopped once its title card gave way to the timer loop and every Java object made after that
// stayed: the 16-byte heap bucket kept about 50,750 of every 65,536 allocations until its 524,288
// slots were full, and the 600-second run ended in an `unwrap` panic on the next allocation.
// So a timer callback collects too, on the same schedule as `Display` (wie-midp `GC_INTERVAL_MS`,
// `GC_COST_SHARE`): at most once a second, and never more than a twentieth of the time between.
const GC_INTERVAL_MS: u64 = 1000;
const GC_COST_SHARE: u64 = 20;

/// When an emulator's last timer collection ran and what it cost. Keyed by `ArmCore::id`, not held
/// by the context: a context is built afresh for every SVC (`handle_wipic_svc`), so a per-context
/// clock was always new and collected on every timer tick — measured 754 collections in 110 s on
/// 236c7da689f6 where the schedule allows ~110.
/// ponytail: an entry outlives its core; a later core with the same id finds a stale `last` and
/// waits at most one interval longer for its first collection.
#[derive(Clone, Copy, Default)]
struct GcClock {
    last: u64,
    cost: u64,
}

static GC_CLOCKS: spin::Mutex<BTreeMap<usize, GcClock>> = spin::Mutex::new(BTreeMap::new());

impl GcClock {
    fn due(&self, now: u64) -> bool {
        now.saturating_sub(self.last) >= GC_INTERVAL_MS.max(self.cost * GC_COST_SHARE)
    }

    fn of(id: usize) -> Self {
        GC_CLOCKS.lock().get(&id).copied().unwrap_or_default()
    }

    fn record(id: usize, last: u64, cost: u64) {
        GC_CLOCKS.lock().insert(id, GcClock { last, cost });
    }
}

// mostly same as ktf's one, can we merge those?
#[derive(Clone)]
pub struct LgtWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm,
}

impl LgtWIPICContext {
    // A Java exception on these paths (fe76e641bb3d: an allocation failure) is an error, not a panic.
    async fn resource_stream(&self, name: &str) -> Result<Option<Box<dyn jvm::ClassInstance>>> {
        let stream = match JavaLangClassLoader::get_system_class_loader(&self.jvm).await {
            Ok(class_loader) => JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await,
            Err(error) => Err(error),
        };
        match stream {
            Ok(stream) => Ok(stream),
            Err(error) => Err(JvmSupport::to_wie_err(&self.jvm, error).await),
        }
    }

    async fn collect_garbage(&self) -> Result<()> {
        match self.jvm.collect_garbage() {
            Ok(_) => Ok(()),
            Err(error) => Err(JvmSupport::to_wie_err(&self.jvm, error).await),
        }
    }

    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
    }

    async fn collect_garbage_if_due(&mut self) -> Result<()> {
        let (id, now) = (self.core.id(), self.system.platform().now().raw());
        if !GcClock::of(id).due(now) {
            return Ok(());
        }
        self.collect_garbage().await?;
        let cost = self.system.platform().now().raw().saturating_sub(now);
        GcClock::record(id, now, cost);
        self.system.pacing().collected_garbage(cost);

        Ok(())
    }
}

#[async_trait::async_trait]
impl WIPICContext for LgtWIPICContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
        Allocator::alloc(&mut self.core, size)
    }

    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr> {
        alloc_indirect(&mut self.core, size)
    }

    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()> {
        free_indirect(&mut self.core, memory)
    }

    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()> {
        Allocator::free(&mut self.core, address, size)?;

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord> {
        Ok(memory.0)
    }

    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord> {
        self.core.run_function(address, args).await
    }

    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()> {
        struct SpawnProxy {
            context: LgtWIPICContext,
            callback: WIPICMethodBody,
        }

        impl AsyncCallable<Result<()>> for SpawnProxy {
            async fn call(mut self) -> Result<()> {
                self.context.jvm.attach_thread(None).await.unwrap();
                self.callback.call(&mut self.context, Box::new([])).await?;
                self.context.jvm.detach_thread().unwrap();

                Ok(())
            }
        }

        self.system.spawn(SpawnProxy {
            context: self.clone(),
            callback,
        });

        Ok(())
    }

    async fn get_resource_size(&self, name: &str) -> Result<Option<usize>> {
        if let Some(stream) = self.resource_stream(name).await? {
            return match self.jvm.invoke_virtual(&stream, "java/io/InputStream", "available", "()I", ()).await {
                Ok(available) => Ok(Some(i32::max(available, 0) as usize)),
                Err(error) => Err(JvmSupport::to_wie_err(&self.jvm, error).await),
            };
        }
        self.collect_garbage().await?;

        Ok(self.system.filesystem().size(name).await)
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        if let Some(stream) = self.resource_stream(name).await? {
            return match JavaIoInputStream::read_until_end(&self.jvm, &stream).await {
                Ok(data) => Ok(data),
                Err(error) => Err(JvmSupport::to_wie_err(&self.jvm, error).await),
            };
        }

        let Some(size) = self.system.filesystem().size(name).await else {
            return Err(WieError::FatalError(format!("Missing resource: {name}")));
        };
        let mut data = vec![0; size];
        let read = self.system.filesystem().read(name, 0, size, &mut data).await.unwrap_or(0);
        data.truncate(read);

        self.collect_garbage().await?;

        Ok(data)
    }

    fn set_timer(&mut self, due: Instant, pace_from: u64, callback: WIPICMethodBody) {
        let context = self.clone();

        self.system().event_queue().push(Event::guest_timer(due, pace_from, move || {
            let mut context = context.clone();

            async move {
                callback.call(&mut context, Box::new([])).await?;
                context.collect_garbage_if_due().await
            }
        }))
    }
}

impl ByteRead for LgtWIPICContext {
    fn read_bytes(&self, address: WIPICWord, result: &mut [u8]) -> wie_util::Result<usize> {
        self.core.read_bytes(address, result)
    }
}

impl ByteWrite for LgtWIPICContext {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> wie_util::Result<()> {
        self.core.write_bytes(address, data)
    }
}

#[cfg(test)]
mod tests {
    use wipi_types::wipic::WIPICIndirectPtr;

    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::Result;

    use super::{alloc_indirect, free_indirect};

    /// The timer path's collection schedule: once a second, stretched while a collection is costly,
    /// and one clock per emulator however many contexts ask. A per-context clock collected on every
    /// tick (236c7da689f6: 754 collections in 110 s); without `due` a Clet never collects at all
    /// (4fdbd64c9fbd filled its heap).
    #[test]
    fn timer_collection_waits_a_second_and_for_its_cost() {
        use super::GcClock;

        let id = usize::MAX - 1;
        GcClock::record(id, 10_000, 0);
        assert!(!GcClock::of(id).due(10_999));
        assert!(GcClock::of(id).due(11_000));
        GcClock::record(id, 10_000, 100);
        assert!(!GcClock::of(id).due(11_999));
        GcClock::record(usize::MAX - 2, 11_500, 0);
        assert!(!GcClock::of(usize::MAX - 2).due(12_000));
        assert!(GcClock::of(id).due(12_000));
        assert!(GcClock::of(usize::MAX - 2).due(12_500), "another emulator has its own clock");
    }

    /// `free(NULL)` is a no-op, and a real handle still frees.
    ///
    /// The guard is what keeps 메탈슬러그 서바이벌 booting (`MC_grpDestroyOffScreenFrameBuffer(0)`).
    /// It had no test, so the base swap (#161) dropped it without a single red; #265 restored it,
    /// still untested. Removing the guard makes `0 - 4` underflow, which panics this test.
    #[test]
    fn free_null_handle_is_a_no_op_and_real_handle_still_frees() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        free_indirect(&mut core, WIPICIndirectPtr(0))?;

        // The non-null path is unchanged: alloc → free → the same block is handed out again.
        let first = alloc_indirect(&mut core, 16)?;
        free_indirect(&mut core, first)?;
        let again = alloc_indirect(&mut core, 16)?;
        assert_eq!(again.0, first.0);

        Ok(())
    }
}
