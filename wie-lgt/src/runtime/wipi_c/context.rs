use alloc::{boxed::Box, format, vec, vec::Vec};

use jvm::{
    Jvm,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};
use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{AsyncCallable, Event, Instant, System};
use wie_core_arm::{Allocator, ArmCore};
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

// mostly same as ktf's one, can we merge those?
#[derive(Clone)]
pub struct LgtWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm,
}

impl LgtWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
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
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if let Some(stream) = stream {
            let available: i32 = self
                .jvm
                .invoke_virtual(&stream, "java/io/InputStream", "available", "()I", ())
                .await
                .unwrap();
            return Ok(Some(available as _));
        }
        self.jvm.collect_garbage().unwrap();

        Ok(self.system.filesystem().size(name).await)
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if let Some(stream) = stream {
            return Ok(JavaIoInputStream::read_until_end(&self.jvm, &stream).await.unwrap());
        }

        let Some(size) = self.system.filesystem().size(name).await else {
            return Err(WieError::FatalError(format!("Missing resource: {name}")));
        };
        let mut data = vec![0; size];
        let read = self.system.filesystem().read(name, 0, size, &mut data).await.unwrap_or(0);
        data.truncate(read);

        self.jvm.collect_garbage().unwrap();

        Ok(data)
    }

    fn set_timer(&mut self, due: Instant, callback: WIPICMethodBody) {
        let context = self.clone();

        self.system().event_queue().push(Event::timer(due, move || {
            let mut context = context.clone();

            async move {
                callback.call(&mut context, Box::new([])).await?;
                Ok(())
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
