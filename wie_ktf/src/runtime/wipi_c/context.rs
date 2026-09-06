use alloc::{boxed::Box, vec::Vec};

use jvm::{
    Jvm,
    runtime::{JavaIoInputStream, JavaLangClassLoader},
};
use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_backend::{AsyncCallable, Event, Instant, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{ByteRead, ByteWrite, Result, read_generic, write_generic};
use wie_wipi_c::{WIPICContext, WIPICMethodBody};

#[derive(Clone)]
pub struct KtfWIPICContext {
    core: ArmCore,
    system: System,
    jvm: Jvm, // We need jvm to access resource in jvm. TODO is there better way to do this?
}

impl KtfWIPICContext {
    pub fn new(core: ArmCore, system: System, jvm: Jvm) -> Self {
        Self { core, system, jvm }
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
        // PREMISE IS FALSE HERE. A KTF app runs entirely under
        // invoke_static("org/kwis/msp/lcdui/Main", "main"), so a Java frame is always on
        // the stack and current_class_loader hands back KtfClassLoader — a DIFFERENT
        // object from the system loader. It still finds the file because KtfClassLoader
        // overrides no findResource (its proto declares only <init>/findClass) and is
        // constructed with the system class loader as its PARENT (jvm_support.rs), while
        // ClassLoader::getResource asks the parent before findResource.
        // Measured 2026-09-06, two ways. (1) A probe here printed
        //   current=<net.wie.KtfClassLoader>  system=<java.net.URLClassLoader>
        // — DIFFERENT objects, which is the direct refutation of "no Java frame".
        // (2) With all four sites switched to current_class_loader the resource tests
        // still pass, so the two really do agree; cutting KtfClassLoader's parent link
        // breaks KTF while LGT keeps passing. NOTE that second cut is blunt, not a clean
        // isolation: KtfClassLoader::init loads client.bin through the same
        // getResource->parent path, so KTF then fails on the SHIPPED form too. It shows
        // the parent link carries KTF resources; it does not by itself separate the two
        // loader APIs. (1) is what does that.
        // The LGT twins coincide for a DIFFERENT reason (their class is registered with
        // no loader at all, so the fallback branch runs). One sentence cannot cover both,
        // which is exactly how the false premise survived at four sites.
        // Covered since 2026-09-06 by test_data/keydraw_{ktf,lgt}.zip, which bundles res.bin
        // and reads it at boot: wie_{ktf,lgt}/tests/test_resource_reach.rs asserts the guest
        // gets `res:9:602` back, and a planted panic!() here now fails that test (per-site
        // drill, all four sites). Before that fixture the same drill left the suite at
        // 150 passed — these lines had never run. Per-site table:
        // docs/upstream-realign-verdict.md §8-4(3)-b.
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name).await.unwrap();

        if stream.is_none() {
            return Ok(None);
        }

        let available: i32 = self.jvm.invoke_virtual(&stream.unwrap(), "available", "()I", ()).await.unwrap();

        Ok(Some(available as _))
    }

    async fn read_resource(&self, name: &str) -> Result<Vec<u8>> {
        // get_system_class_loader, NOT current_class_loader — same reason, same measurement
        // and same coverage as get_resource_size above; read that block. In short: a Java
        // frame IS on the stack here (so the old "wherever there is no Java frame" premise
        // is false), and the two agree only because KtfClassLoader delegates to its parent.
        let class_loader = JavaLangClassLoader::get_system_class_loader(&self.jvm).await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&self.jvm, &class_loader, name)
            .await
            .unwrap()
            .unwrap();

        Ok(JavaIoInputStream::read_until_end(&self.jvm, &stream).await.unwrap())
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
