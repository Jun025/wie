use alloc::{boxed::Box, collections::BTreeMap, string::String, sync::Arc, vec::Vec};

use spin::Mutex;

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

#[derive(Clone)]
pub struct KtfWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm, // We need jvm to access resource in jvm. TODO is there better way to do this?
    resources: ResourceCache,
}

/// Resource bytes by name, shared by every context of one WIPI-C runtime.
///
/// Reading a resource through the JVM leaves its stream and byte array as guest-heap garbage,
/// so each read ends in a full `collect_garbage` — about 5.4ms natively on 영웅서기4 (KTF), and
/// titles call `MC_knlGetResourceID` + `MC_knlGetResource` for the same name back to back and
/// again on every screen change. Uncached, a menu transition spent 58-65% of its frame in those
/// collections, inside one SVC where ARM preemption cannot split it
/// (docs/report/ for wie-ktf-hero4-290ms-frame-after-key-input). Resources are read-only archive
/// entries, so each is read — and collected after — once. Missing names are not cached.
pub type ResourceCache = Arc<Mutex<BTreeMap<String, Vec<u8>>>>;

impl KtfWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm, resources: ResourceCache) -> Self {
        Self {
            core,
            system,
            jvm,
            resources,
        }
    }

    async fn resource(&self, name: &str) -> Result<Option<Vec<u8>>> {
        if let Some(data) = self.resources.lock().get(name) {
            return Ok(Some(data.clone()));
        }

        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm)
            .await
            .map_err(|err| WieError::FatalError(alloc::format!("Failed to get class loader for resource {name:?}: {err:?}")))?;
        let stream = match JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await {
            Ok(stream) => stream,
            Err(err) => {
                tracing::error!("Java exception while opening resource: name={name:?}, error={err:?}");
                return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
            }
        };

        let data = match stream {
            Some(stream) => match JavaIoInputStream::read_until_end(&self.jvm, &stream).await {
                Ok(data) => Some(data),
                Err(err) => {
                    tracing::error!("Java exception while reading resource: name={name:?}, error={err:?}");
                    return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
                }
            },
            None => None,
        };
        if let Err(err) = self.jvm.collect_garbage() {
            return Err(JvmSupport::to_wie_err(&self.jvm, err).await);
        }

        if let Some(data) = &data {
            self.resources.lock().insert(name.into(), data.clone());
        }

        Ok(data)
    }
}

#[async_trait::async_trait]
impl WIPICContext for KtfWIPICContext {
    fn alloc_raw(&mut self, size: WIPICWord) -> Result<WIPICWord> {
        Allocator::alloc(&mut self.core, size)
    }

    fn alloc(&mut self, size: WIPICWord) -> Result<WIPICIndirectPtr> {
        let ptr = Allocator::alloc(&mut self.core, size + 12)?; // all allocation has indirect pointer
        write_generic(&mut self.core, ptr, ptr + 4)?;
        write_generic(&mut self.core, ptr + 4, size)?;

        Ok(WIPICIndirectPtr(ptr))
    }

    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()> {
        // Freeing a null handle is a defined no-op (C `free(NULL)` / `MC_knlFree(NULL)`), as in LGT's
        // `free_indirect`. A KTF title's first-run exit calls `MC_grpDestroyOffScreenFrameBuffer(0)`.
        if memory.0 == 0 {
            return Ok(());
        }
        let size: u32 = read_generic(&self.core, memory.0 + 4)?;
        Allocator::free(&mut self.core, memory.0, size + 12)?;

        Ok(())
    }

    fn free_raw(&mut self, address: WIPICWord, size: WIPICWord) -> Result<()> {
        Allocator::free(&mut self.core, address, size)?;

        Ok(())
    }

    fn data_ptr(&self, memory: WIPICIndirectPtr) -> Result<WIPICWord> {
        let base: WIPICWord = read_generic(&self.core, memory.0)?;

        Ok(base + 8) // all data has offset of 8 bytes
    }

    fn system(&mut self) -> &mut System {
        &mut self.system
    }

    async fn call_function(&mut self, address: WIPICWord, args: &[WIPICWord]) -> Result<WIPICWord> {
        self.core.run_function(address, args).await
    }

    fn spawn(&mut self, callback: WIPICMethodBody) -> Result<()> {
        struct SpawnProxy {
            context: KtfWIPICContext,
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
        Ok(self.resource(name).await?.map(|data| data.len()))
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        self.resource(name)
            .await?
            .ok_or_else(|| WieError::FatalError(alloc::format!("Resource disappeared before read: {name:?}")))
    }

    fn set_timer(&mut self, due: Instant, pace_from: u64, callback: WIPICMethodBody) {
        let context = self.clone();

        self.system().event_queue().push(Event::guest_timer(due, pace_from, move || {
            let mut context = context.clone();

            async move {
                callback.call(&mut context, Box::new([])).await?;
                Ok(())
            }
        }))
    }
}

impl ByteRead for KtfWIPICContext {
    fn read_bytes(&self, address: WIPICWord, result: &mut [u8]) -> wie_util::Result<usize> {
        self.core.read_bytes(address, result)
    }
}

impl ByteWrite for KtfWIPICContext {
    fn write_bytes(&mut self, address: WIPICWord, data: &[u8]) -> wie_util::Result<()> {
        self.core.write_bytes(address, data)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc};
    use core::{
        future::Future,
        mem::size_of,
        pin::Pin,
        sync::atomic::{AtomicBool, Ordering},
    };

    use bytemuck::Zeroable;

    use wipi_types::wipic::WIPICIndirectPtr;

    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{ByteWrite, Result, WieError, write_generic};
    use wie_wipi_c::WIPICContext;

    use test_utils::TestPlatform;

    use crate::runtime::{
        java::{
            interface::java_throw_instance,
            jvm_support::{KtfJvmSupport, KtfJvmThreadContext},
        },
        svc_ids::WIPICTableId,
        wipi_c::method_table::get_method_body,
    };

    use super::KtfWIPICContext;

    async fn new_context(system: &mut System) -> Result<KtfWIPICContext> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;
        let mut registers = core.save_context();
        registers.sp = Allocator::alloc(&mut core, 0x100)? + 0x100;
        core.restore_context(&registers);
        let ptr_thread_context = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
        write_generic(&mut core, ptr_thread_context, KtfJvmThreadContext::zeroed())?;
        KtfJvmSupport::set_current_thread_context(&mut core, ptr_thread_context)?;
        let (jvm, _) = KtfJvmSupport::init(&mut core, system, None).await?;

        Ok(KtfWIPICContext::new(core, system.clone(), jvm, Default::default()))
    }

    fn run(body: fn(KtfWIPICContext) -> Pin<Box<dyn Future<Output = Result<()>> + Send>>) -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            body(new_context(&mut system_clone).await?).await?;
            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }

    // A KTF title's first-run exit destroys an offscreen frame buffer it never created.
    #[test]
    fn free_of_null_is_a_no_op() -> Result<()> {
        run(|mut context| Box::pin(async move { context.free(WIPICIndirectPtr(0)) }))
    }

    // Interface4 slot 0 is the header's record-database open, not the unnamed-table stub.
    #[test]
    fn interface4_slot_0_opens_a_record_database() -> Result<()> {
        run(|mut context| {
            Box::pin(async move {
                let name = context.alloc_raw(16)?;
                context.write_bytes(name, b"SaveData\0")?;
                let open = get_method_body(WIPICTableId::Interface4, 0).unwrap();
                let args = [name, 0xeec, 0, 1];
                match open.call(&mut context, Box::new(args)).await {
                    Err(WieError::Unimplemented(message)) => panic!("Interface4[0] is still a stub: {message}"),
                    result => {
                        result?;
                    }
                }

                Ok(())
            })
        })
    }

    // Database slot 11 is free storage in bytes, not the header's `MC_dbGetRecordSize(fd)`: its callers
    // pass nothing and keep a save only when the answer is at least the save's length.
    #[test]
    fn database_slot_11_answers_free_storage() -> Result<()> {
        run(|mut context| {
            Box::pin(async move {
                let slot11 = get_method_body(WIPICTableId::Database, 11).unwrap();
                let slot12 = get_method_body(WIPICTableId::Database, 12).unwrap();
                let free = slot11.call(&mut context, Box::new([0; 4])).await?.results[0] as i32;
                assert_eq!(free, slot12.call(&mut context, Box::new([])).await?.results[0] as i32);
                assert!(free > 0x176f, "a boot check reads {free} as «not enough storage»");

                Ok(())
            })
        })
    }

    // `throw e` (InitParam4 +8) hands over the instance the guest built. With no handler registered it
    // reaches the host as that very instance; `throw null` becomes a NullPointerException.
    #[test]
    fn throw_instance_throws_the_instance_it_is_given() -> Result<()> {
        run(|context| {
            Box::pin(async move {
                let (mut core, mut jvm) = (context.core.clone(), context.jvm.clone());
                let exception = jvm.new_class("java/lang/RuntimeException", "()V", ()).await.unwrap();
                let raw = KtfJvmSupport::class_instance_raw(&exception);

                let thrown = java_throw_instance(&mut core, &mut jvm, raw, 0).await;
                assert!(matches!(thrown, Err(WieError::JavaException(x)) if x == raw));
                let thrown = java_throw_instance(&mut core, &mut jvm, 0, 0).await;
                assert!(matches!(thrown, Err(WieError::JavaException(x)) if x != 0 && x != raw));

                Ok(())
            })
        })
    }
}
