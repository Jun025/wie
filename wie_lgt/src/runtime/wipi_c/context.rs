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
        let address = Allocator::alloc(&mut self.core, size + size_of::<WIPICWord>() as WIPICWord)?;
        write_generic(&mut self.core, address, size)?;

        Ok(WIPICIndirectPtr(address + size_of::<WIPICWord>() as WIPICWord))
    }

    fn free(&mut self, memory: WIPICIndirectPtr) -> Result<()> {
        // Freeing a null handle is a defined no-op (C `free(NULL)` / `MC_knlFree(NULL)`).
        // Games double-destroy or destroy a never-created framebuffer (e.g.
        // `MC_grpDestroyOffScreenFrameBuffer(0)`); without this the `- 4` below underflows.
        if memory.0 == 0 {
            return Ok(());
        }

        let base_address = memory.0 - size_of::<WIPICWord>() as WIPICWord;

        let size: WIPICWord = read_generic(&self.core, base_address)?;
        Allocator::free(&mut self.core, base_address, size + size_of::<WIPICWord>() as WIPICWord)
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
                // +33 pin: async + Option<Java Thread>. A WIPI callback thread is
                // host-spawned and has no Java Thread object, so `None`.
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
        // get_system_class_loader, NOT current_class_loader: the latter goes private one
        // commit past our pin. Both DO resolve this resource today — but not for the
        // reason an earlier revision of this comment gave. It said "the two coincide
        // wherever there is no Java frame": the clause is true in general and the
        // PREMISE IS FALSE HERE. These calls run inside net/wie/CletWrapper::startApp,
        // which drives the ARM core synchronously (core.run_function(start_clet)), so a
        // Java frame is on the stack the whole time. They agree anyway because
        // CletWrapper/CletWrapperCard are registered with NO class loader at all —
        // jvm.register_class(.., None) in wie_lgt/src/runtime/wipi_c.rs — so
        // current_class_loader finds the calling class has no loader and falls back to
        // get_system_class_loader itself. Measured 2026-09-06: with all four sites on
        // current_class_loader the resource tests still pass, and a probe here printed
        //   current=<java.net.URLClassLoader>  system=<java.net.URLClassLoader>
        // — the same class, as the fallback predicts. Be precise about what that probe
        // does NOT show: "no frame" and "frame whose class has no loader" both end in
        // that fallback, so the printout cannot separate them. The frame's presence is
        // established from source instead — start_app drives the ARM core synchronously
        // at clet_wrapper.rs (core.run_function(start_clet)), so this call happens inside
        // the Java method. The KTF probe, where the two objects DIFFER, is the direct
        // refutation of the old premise.
        // The KTF twins coincide for a DIFFERENT reason (their class HAS a loader,
        // KtfClassLoader, which delegates to the system loader as its parent). One
        // sentence cannot cover both, which is exactly how the false premise survived at
        // four sites.
        // Covered since 2026-09-06 by test_data/keydraw_{ktf,lgt}.zip, which bundles res.bin
        // and reads it at boot: wie_{ktf,lgt}/tests/test_resource_reach.rs asserts the guest
        // gets `res:9:602` back, and a planted panic!() here now fails that test (per-site
        // drill, all four sites). Before that fixture the same drill left the suite at
        // 150 passed — these lines had never run. Per-site table:
        // docs/upstream-realign-verdict.md §8-4(3)-b.
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if let Some(stream) = stream {
            let available: i32 = self.jvm.invoke_virtual(&stream, "available", "()I", ()).await.unwrap();
            return Ok(Some(available as _));
        }

        Ok(self.system.filesystem().size(name).await)
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        // get_system_class_loader, NOT current_class_loader — same reason, same measurement
        // and same coverage as get_resource_size above; read that block. In short: a Java
        // frame IS on the stack here (so the old "wherever there is no Java frame" premise
        // is false), and the two agree only because the calling class has no loader.
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
