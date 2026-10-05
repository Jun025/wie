mod array_class_definition;
mod array_class_instance;
mod class_definition;
mod class_instance;
mod classes;
mod field;
mod guest_roots;
mod jvm_implementation;
mod method;
mod name;
mod value;
mod vtable;

use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeSet, format, sync::Arc};
use core::mem::{offset_of, size_of};
use jvm_implementation::KtfJvmImplementation;

use bytemuck::{Pod, Zeroable};
use futures::TryFutureExt;

use jvm::{ClassDefinition, ClassInstance, ClassInstanceRef, Field, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_types::FieldAccessFlags;
use rustjava_runtime::classes::java::{
    lang::Throwable,
    util::{Enumeration, jar::JarEntry},
};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmSupport;
use wie_midp::classes::javax::microedition::midlet::MIDlet;
use wie_util::{Result, WieError, read_generic, read_null_terminated_table, write_generic};

use wipi_types::ktf::{ExeInterfaceFunctions, InitParam2, java::JavaClass as RawJavaClass};

use self::{
    array_class_instance::JavaArrayClassInstance,
    classes::net::wie::{ClassLoaderContext, KtfClassLoader},
    name::JavaFullName,
};
use super::interface::register_java_interface_svc_handler;

pub use self::{
    array_class_definition::JavaArrayClassDefinition,
    class_definition::JavaClassDefinition,
    class_instance::JavaClassInstance,
    method::{JavaMethod, JavaMethodResult},
    vtable::JavaVtable,
};

pub type KtfJvmWord = u32;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct KtfJvmThreadContext {
    unk: [u32; 8],
    current_java_exception_handler: u32,
    // A native body can hand its result back here instead of in r0: KTF AID 0103BF27
    // writes `+0x24 = 2` and the int to `+0x28`, leaving an unrelated constant in r0.
    // Before these fields existed those two stores landed past the end of this allocation.
    pub(crate) native_result_type: u32,
    pub(crate) native_result: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct KtfJvmSupportContext {
    ptr_vtables_base: u32,
    ptr_current_jvm_thread_context: u32,
}

const SUPPORT_CONTEXT_BASE: u32 = 0x7fff0000;

pub struct KtfJvmSupport;

impl KtfJvmSupport {
    pub async fn init(core: &mut ArmCore, system: &mut System, jar_name: Option<&str>) -> Result<(Jvm, Box<dyn ClassInstance>)> {
        let jvm_context = InitParam2 {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_java_vtables: [0; 128],
        };
        let ptr_jvm_context = Allocator::alloc(core, size_of::<InitParam2>() as u32)?;
        write_generic(core, ptr_jvm_context, jvm_context)?;

        write_generic(
            core,
            SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_vtables_base) as u32,
            ptr_jvm_context + 12,
        )?;

        let protos = [wie_wipi_java::get_protos().into(), wie_midp::get_protos().into()];
        let jvm_implementation = KtfJvmImplementation::new(core);
        let jvm = JvmSupport::new_jvm(system, jar_name, Box::new(protos), &[], jvm_implementation.clone()).await?;
        guest_roots::install(&jvm, core);
        register_java_interface_svc_handler(core, &jvm)?;

        let system_class_loader: Box<dyn ClassInstance> = jvm
            .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", [])
            .await
            .unwrap();

        // used in tests
        if jar_name.is_none() {
            return Ok((jvm, system_class_loader));
        }

        // find client.bin
        let jar_name_java = JavaLangString::from_rust_string(&jvm, jar_name.unwrap()).await.unwrap();
        let jar_file = jvm
            .new_class("java/util/jar/JarFile", "(Ljava/lang/String;)V", (jar_name_java,))
            .or_else(async |error| Err(JvmSupport::to_wie_err(&jvm, error).await))
            .await?;
        let entries: ClassInstanceRef<Enumeration> = jvm
            .invoke_virtual(&jar_file, "java/util/jar/JarFile", "entries", "()Ljava/util/Enumeration;", [])
            .await
            .unwrap();

        // Also collect the jar's `.class` entries: 3 KTF titles ship their source `Clet.class`
        // next to the AOT client.bin that holds the same classes, and the loader must pick client.bin.
        let mut binary_name = None;
        let mut jar_classes = BTreeSet::new();
        loop {
            let has_more_elements: bool = jvm
                .invoke_virtual(&entries, "java/util/Enumeration", "hasMoreElements", "()Z", [])
                .await
                .unwrap();
            if !has_more_elements {
                break;
            }

            let entry: ClassInstanceRef<JarEntry> = jvm
                .invoke_virtual(&entries, "java/util/Enumeration", "nextElement", "()Ljava/lang/Object;", [])
                .await
                .unwrap();
            let name = jvm
                .invoke_virtual(&entry, "java/util/jar/JarEntry", "getName", "()Ljava/lang/String;", [])
                .await
                .unwrap();
            let name_rust = JavaLangString::to_rust_string(&jvm, &name).await.unwrap();

            if let Some(class_name) = name_rust.strip_suffix(".class") {
                jar_classes.insert(class_name.to_owned());
            } else if binary_name.is_none() && name_rust.starts_with("client.bin") {
                binary_name = Some(name);
            }
        }
        let binary_name = binary_name.ok_or_else(|| WieError::FatalError("client.bin not found".into()))?;

        let class_loader_class = JavaClassDefinition::new(
            core,
            &jvm,
            KtfClassLoader::as_proto(),
            Box::new(ClassLoaderContext {
                core: core.clone(),
                system: system.clone(),
                jar_classes: Arc::new(jar_classes),
            }) as Box<_>,
            jvm_implementation.java_functions(),
        )
        .await?;

        jvm.register_class(Box::new(class_loader_class), None).await.unwrap();

        let class_loader = jvm
            .new_class(
                "net/wie/KtfClassLoader",
                "(Ljava/lang/ClassLoader;Ljava/lang/String;II)V",
                (
                    system_class_loader,
                    binary_name,
                    ptr_jvm_context as i32,
                    (SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_current_jvm_thread_context) as u32) as i32,
                ),
            )
            // The constructor loads client.bin, so an unsupported image (a relocation-table layout,
            // `docs/report/0340`) and a jar that is not a zip (an OMA DRM container) both throw here.
            // That is the title's error to report, not a host panic.
            .or_else(async |error| Err(JvmSupport::to_wie_err(&jvm, error).await))
            .await?;

        Ok((jvm, class_loader))
    }

    // Native initialization must have linked the class catalog before registration.
    pub(crate) async fn register_static_classes(
        core: &mut ArmCore,
        jvm: &Jvm,
        class_loader: Box<dyn ClassInstance>,
        main_class_name: &str,
    ) -> Result<()> {
        let ptr_functions: i32 = jvm
            .get_field(&class_loader, "nativeFunctions", "I")
            .or_else(async |error| Err(JvmSupport::to_wie_err(jvm, error).await))
            .await?;
        let functions: ExeInterfaceFunctions = read_generic(core, ptr_functions as u32)?;
        let predicate = functions.fn_is_native_class_address;
        if predicate == 0 {
            return Ok(());
        }

        let main_class = jvm
            .resolve_class(main_class_name)
            .or_else(async |error| Err(JvmSupport::to_wie_err(jvm, error).await))
            .await?;
        let mut ptr_class = Self::class_definition_raw(&*main_class.definition)?;
        if core.run_function::<u32>(predicate, &[ptr_class]).await? == 0 {
            return Ok(());
        }

        let class_size = size_of::<RawJavaClass>() as u32;
        while core.run_function::<u32>(predicate, &[ptr_class - class_size]).await? != 0 {
            ptr_class -= class_size;
        }
        while core.run_function::<u32>(predicate, &[ptr_class]).await? != 0 {
            let class = JavaClassDefinition::from_raw(ptr_class, core);
            let name = class.name()?;
            if !jvm.has_class(&name) && class.fields()?.any(|field| field.access_flags().contains(FieldAccessFlags::STATIC)) {
                jvm.register_class(Box::new(class), Some(class_loader.clone()))
                    .or_else(async |error| Err(JvmSupport::to_wie_err(jvm, error).await))
                    .await?;
            }
            ptr_class += class_size;
        }

        Ok(())
    }

    pub(crate) async fn disable_midp_paint(jvm: &Jvm) -> JvmResult<()> {
        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        let display = MIDlet::display(jvm, &midlet).await?;

        jvm.invoke_virtual(&display, "javax/microedition/lcdui/Display", "disablePaint", "()V", ())
            .await
    }

    /// The error for a guest instance that could not be allocated.
    ///
    /// Building an exception allocates too, so on a full guest heap `jvm.exception` failed the same
    /// way and recursed — instance, exception, `fillInStackTrace`'s string, instance — until the
    /// *host* stack overflowed and took the whole tab down (3 KTF titles in the 2026-09-27 census,
    /// after minutes of play). A full heap throws the `OutOfMemoryError` allocated up front
    /// instead, the way a JVM keeps one in reserve; it stays a Java exception the guest may catch.
    pub(crate) async fn instantiation_error(jvm: &Jvm, error: WieError, kind: &str) -> JavaError {
        tracing::error!("Failed to instantiate {kind}: {error}");
        if matches!(error, WieError::AllocationFailure)
            && let Ok(reserved) = jvm
                .get_static_field::<ClassInstanceRef<Throwable>>("net/wie/KtfClassLoader", "outOfMemoryError", "Ljava/lang/OutOfMemoryError;")
                .await
            && !reserved.is_null()
        {
            return JavaError::JavaException(reserved.into());
        }

        Self::wie_error(jvm, &format!("Failed to instantiate {kind}: {error}")).await
    }

    /// `jvm.exception("net/wie/WieError", message)` without its `unwrap`s: when the exception
    /// itself cannot be allocated, the allocation's own error is returned instead of a host panic.
    pub(crate) async fn wie_error(jvm: &Jvm, message: &str) -> JavaError {
        let exception = async {
            let message = JavaLangString::from_rust_string(jvm, message).await?;
            jvm.new_class("net/wie/WieError", "(Ljava/lang/String;)V", (message,)).await
        };
        match exception.await {
            Ok(x) => JavaError::JavaException(x),
            Err(x) => x,
        }
    }

    pub fn class_definition_raw(definition: &dyn ClassDefinition) -> Result<u32> {
        Ok(if let Some(x) = definition.as_any().downcast_ref::<JavaClassDefinition>() {
            x.ptr_raw
        } else {
            let class = definition.as_any().downcast_ref::<JavaArrayClassDefinition>().unwrap();

            class.class.ptr_raw
        })
    }

    pub fn class_from_raw(core: &ArmCore, ptr_class: u32) -> JavaClassDefinition {
        JavaClassDefinition::from_raw(ptr_class, core)
    }

    pub fn read_name(core: &ArmCore, ptr_name: u32) -> Result<JavaFullName> {
        JavaFullName::from_ptr(core, ptr_name)
    }

    #[allow(clippy::borrowed_box)]
    pub fn class_instance_raw(instance: &Box<dyn ClassInstance>) -> u32 {
        if let Some(x) = instance.as_any().downcast_ref::<JavaClassInstance>() {
            x.ptr_raw
        } else {
            let instance = instance.as_any().downcast_ref::<JavaArrayClassInstance>().unwrap();

            instance.class_instance.ptr_raw
        }
    }

    pub fn get_vtable_index(core: &mut ArmCore, class: &JavaClassDefinition) -> Result<u32> {
        // TODO remove context
        let context_data: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
        let ptr_vtables = read_null_terminated_table(core, context_data.ptr_vtables_base)?;

        let ptr_vtable = class.ptr_vtable()?;

        for (index, &current_ptr_vtable) in ptr_vtables.iter().enumerate() {
            if ptr_vtable == current_ptr_vtable {
                return Ok(index as _);
            }
        }

        let index = ptr_vtables.len();
        write_generic(core, context_data.ptr_vtables_base + (index * size_of::<u32>()) as u32, ptr_vtable)?;

        Ok(index as _)
    }

    pub fn current_java_exception_handler(core: &mut ArmCore) -> Result<u32> {
        let ptr_thread_context = Self::current_thread_context(core)?;
        let thread_context: KtfJvmThreadContext = read_generic(core, ptr_thread_context)?;

        Ok(thread_context.current_java_exception_handler)
    }

    /// `(thread context, the caller's handler)` — written back to that same context by
    /// `leave_exception_scope`, whichever thread is current by then. None if it cannot be read.
    pub fn enter_exception_scope(core: &mut ArmCore) -> Option<(u32, u32)> {
        let ptr_thread_context = Self::current_thread_context(core).ok()?;
        let saved = Self::current_java_exception_handler(core).ok()?;
        Self::write_java_exception_handler(core, ptr_thread_context, 0).ok()?;
        Some((ptr_thread_context, saved))
    }

    pub fn leave_exception_scope(core: &mut ArmCore, scope: Option<(u32, u32)>) {
        if let Some((ptr_thread_context, saved)) = scope {
            let _ = Self::write_java_exception_handler(core, ptr_thread_context, saved);
        }
    }

    fn write_java_exception_handler(core: &mut ArmCore, ptr_thread_context: u32, ptr_handler: u32) -> Result<()> {
        write_generic(
            core,
            ptr_thread_context + offset_of!(KtfJvmThreadContext, current_java_exception_handler) as u32,
            ptr_handler,
        )
    }

    pub fn set_current_java_exception_handler(core: &mut ArmCore, ptr_handler: u32) -> Result<()> {
        let ptr_thread_context = Self::current_thread_context(core)?;
        Self::write_java_exception_handler(core, ptr_thread_context, ptr_handler)
    }

    pub fn set_current_thread_context(core: &mut ArmCore, ptr_thread_context: u32) -> Result<()> {
        write_generic(
            core,
            SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_current_jvm_thread_context) as u32,
            ptr_thread_context,
        )
    }

    pub fn current_thread_context(core: &ArmCore) -> Result<u32> {
        let context_data: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;

        Ok(context_data.ptr_current_jvm_thread_context)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
    use core::{
        mem::size_of,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    };

    use bytemuck::Zeroable;
    use jvm::{
        ClassInstanceRef, JavaValue, Jvm,
        runtime::{JavaLangClass, JavaLangString},
    };
    use jvm_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::Class;
    use wipi_types::ktf::{
        ExeInterfaceFunctions,
        java::{
            JavaClass as RawJavaClass, JavaClassInstance as RawJavaClassInstance, JavaExceptionHandler as RawJavaExceptionHandler,
            JavaFieldDefinition as RawJavaField, JavaMethodDefinition as RawJavaMethod,
            JavaMethodExceptionTableEntry as RawJavaMethodExceptionTableEntry,
        },
    };

    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_jvm_support::native::encode_method_arguments;
    use wie_midp::classes::javax::microedition::{lcdui::Display as MidpDisplay, midlet::MIDlet};
    use wie_util::{Result, WieError, read_generic, write_generic};

    use crate::runtime::java::{JavaSvcFunctions, handle_java_svc, interface};

    use super::{
        ClassLoaderContext, JavaArrayClassInstance, JavaClassDefinition, JavaClassInstance, JavaMethod, KtfClassLoader, KtfJvmSupport,
        KtfJvmThreadContext, value::JavaValueCodec,
    };

    use test_utils::{TestClock, TestPlatform};

    async fn init_jvm(system: &mut System) -> Result<(Jvm, ArmCore)> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let mut context = core.save_context();
        let stack = Allocator::alloc(&mut core, 0x100)?;
        context.sp = stack + 0x100;
        core.restore_context(&context);

        let ptr_thread_context = Allocator::alloc(&mut core, size_of::<KtfJvmThreadContext>() as u32)?;
        write_generic(&mut core, ptr_thread_context, KtfJvmThreadContext::zeroed())?;
        KtfJvmSupport::set_current_thread_context(&mut core, ptr_thread_context)?;

        let (jvm, _) = KtfJvmSupport::init(&mut core, system, None).await?;

        Ok((jvm, core))
    }

    #[test]
    fn test_register_static_classes_keeps_live_guest_roots_without_initializing() -> Result<()> {
        async fn is_native_class(_core: &mut ArmCore, range: &mut core::ops::Range<u32>, address: u32) -> Result<u32> {
            Ok(u32::from(range.contains(&address)))
        }

        async fn count_initialization(_jvm: &Jvm, count: &mut Arc<AtomicUsize>) -> jvm::Result<()> {
            count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let java_functions = JavaSvcFunctions::default();
            core.register_svc_handler(5, handle_java_svc, &java_functions)?;
            let loader_class = JavaClassDefinition::new(
                &mut core.clone(),
                &jvm,
                KtfClassLoader::as_proto(),
                Box::new(ClassLoaderContext {
                    core: core.clone(),
                    system: system_clone.clone(),
                    jar_classes: Default::default(),
                }),
                java_functions.clone(),
            )
            .await?;
            for method in loader_class.methods()? {
                let mut raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                raw.fn_body = core.make_svc_stub(5, method.ptr_raw)?;
                write_generic(&mut core, method.ptr_raw, raw)?;
            }
            jvm.register_class(Box::new(loader_class), None).await.unwrap();
            let mut loader = jvm.instantiate_class("net/wie/KtfClassLoader").await.unwrap();
            let ptr_functions = Allocator::alloc(&mut core, size_of::<ExeInterfaceFunctions>() as u32)?;
            let mut functions = ExeInterfaceFunctions::zeroed();
            write_generic(&mut core, ptr_functions, functions)?;
            jvm.put_field(&mut loader, "nativeFunctions", "I", ptr_functions as i32).await.unwrap();

            let missing_name = JavaLangString::from_rust_string(&jvm, "test/Missing").await.unwrap();
            let missing: ClassInstanceRef<Class> = jvm
                .invoke_virtual(
                    &loader,
                    "net/wie/KtfClassLoader",
                    "findClass",
                    "(Ljava/lang/String;)Ljava/lang/Class;",
                    (missing_name,),
                )
                .await
                .unwrap();
            assert!(missing.is_null());

            let class_size = size_of::<RawJavaClass>() as u32;
            let begin = Allocator::alloc(&mut core, class_size * 4)?;
            let end = begin + class_size * 4;
            core.register_svc_handler(6, is_native_class, &(begin..end))?;
            let predicate = core.make_svc_stub(6, 0u32)?;
            let initialization_count = Arc::new(AtomicUsize::new(0));
            let mut classes = Vec::new();
            for (index, name) in ["test/Before", "test/Unused", "test/Main", "test/After"].into_iter().enumerate() {
                let class = JavaClassDefinition::new(
                    &mut core,
                    &jvm,
                    JavaClassProto {
                        name,
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: if index == 0 {
                            vec![JavaMethodProto::new("<clinit>", "()V", count_initialization, MethodAccessFlags::STATIC)]
                        } else {
                            vec![]
                        },
                        fields: vec![JavaFieldProto::new(
                            "root",
                            "Ljava/lang/Object;",
                            if index == 1 {
                                FieldAccessFlags::PUBLIC
                            } else {
                                FieldAccessFlags::STATIC
                            },
                        )],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(initialization_count.clone()),
                    java_functions.clone(),
                )
                .await?;

                // Relocate only the class headers into the native ABI's contiguous table.
                let ptr_class = begin + index as u32 * class_size;
                let mut raw: RawJavaClass = read_generic(&core, class.ptr_raw)?;
                raw.ptr_next = ptr_class + 4;
                write_generic(&mut core, ptr_class, raw)?;
                for field in class.fields()? {
                    let mut raw: RawJavaField = read_generic(&core, field.ptr_raw)?;
                    raw.ptr_class = ptr_class;
                    write_generic(&mut core, field.ptr_raw, raw)?;
                }
                for method in class.methods()? {
                    let mut raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                    raw.ptr_class = ptr_class;
                    raw.fn_body = core.make_svc_stub(5, method.ptr_raw)?;
                    write_generic(&mut core, method.ptr_raw, raw)?;
                }
                classes.push(JavaClassDefinition::from_raw(ptr_class, &core));
            }
            let seed = jvm
                .register_class(Box::new(classes[2].clone()), Some(loader.clone()))
                .await
                .unwrap()
                .unwrap();
            let seed_identity = seed.identity();

            KtfJvmSupport::register_static_classes(&mut core, &jvm, loader.clone(), "test/Main").await?;
            assert!(!jvm.has_class("test/Before"));
            assert!(!jvm.has_class("test/After"));
            functions.fn_is_native_class_address = predicate;
            write_generic(&mut core, ptr_functions, functions)?;
            KtfJvmSupport::register_static_classes(&mut core, &jvm, loader.clone(), "java/lang/Object").await?;
            assert!(!jvm.has_class("test/Before"));
            assert!(!jvm.has_class("test/After"));

            let mut identities = Vec::new();
            for pass in 0..2 {
                KtfJvmSupport::register_static_classes(&mut core, &jvm, loader.clone(), "test/Main").await?;
                assert!(!jvm.has_class("test/Unused"));
                for (index, name) in ["test/Before", "test/Main", "test/After"].into_iter().enumerate() {
                    assert!(jvm.has_class(name));
                    let class = jvm.resolve_class(name).await.unwrap().java_class();
                    let registered_loader = JavaLangClass::class_loader(&jvm, &class).await.unwrap().unwrap();
                    assert_eq!(registered_loader.identity(), loader.identity());
                    if pass == 0 {
                        identities.push(class.identity());
                    } else {
                        assert_eq!(class.identity(), identities[index]);
                    }
                }
                assert_eq!(identities[1], seed_identity);
                assert_eq!(initialization_count.load(Ordering::Relaxed), 0);
            }
            for class in &classes {
                let raw: RawJavaClass = read_generic(&core, class.ptr_raw)?;
                assert_eq!(raw.unk_flag, 8);
            }

            let before_root = classes[0].field("root", "Ljava/lang/Object;", true)?.unwrap();
            let after_root = classes[3].field("root", "Ljava/lang/Object;", true)?.unwrap();
            jvm.pop_frame();
            jvm.collect_garbage().unwrap();
            jvm.push_native_frame();
            let mut objects = Vec::new();
            for _ in 0..3 {
                let object = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
                objects.push(KtfJvmSupport::class_instance_raw(&object));
            }
            classes[0].write_static_field(&before_root, objects[0])?;
            classes[3].write_static_field(&after_root, objects[1])?;
            jvm.pop_frame();
            jvm.collect_garbage().unwrap();
            let instance_size = size_of::<RawJavaClassInstance>() as u32;
            assert!(Allocator::is_allocated(&core, objects[0], instance_size)?);
            assert!(Allocator::is_allocated(&core, objects[1], instance_size)?);
            assert!(!Allocator::is_allocated(&core, objects[2], instance_size)?);

            jvm.push_native_frame();
            let replacement = jvm.new_class("java/lang/Object", "()V", ()).await.unwrap();
            let replacement = KtfJvmSupport::class_instance_raw(&replacement);
            classes[0].write_static_field(&before_root, replacement)?;
            jvm.pop_frame();
            jvm.collect_garbage().unwrap();
            assert!(!Allocator::is_allocated(&core, objects[0], instance_size)?);
            assert!(Allocator::is_allocated(&core, objects[1], instance_size)?);
            assert!(Allocator::is_allocated(&core, replacement, instance_size)?);

            classes[0].write_static_field(&before_root, 0)?;
            classes[3].write_static_field(&after_root, 0)?;
            jvm.collect_garbage().unwrap();
            assert!(!Allocator::is_allocated(&core, objects[1], instance_size)?);
            assert!(!Allocator::is_allocated(&core, replacement, instance_size)?);
            assert_eq!(initialization_count.load(Ordering::Relaxed), 0);
            jvm.push_native_frame();
            jvm.ensure_initialized(&jvm.resolve_class("test/Before").await.unwrap()).await.unwrap();
            assert_eq!(initialization_count.load(Ordering::Relaxed), 1);
            jvm.pop_frame();

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    // An allocation the guest heap cannot hold throws the reserved OutOfMemoryError, allocating nothing:
    // building a fresh exception there needs the heap too, and recursed until the host stack overflowed.
    #[test]
    fn test_allocation_failure_throws_reserved_out_of_memory_error() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, core) = init_jvm(&mut system_clone).await?;
            let loader_class = JavaClassDefinition::new(
                &mut core.clone(),
                &jvm,
                KtfClassLoader::as_proto(),
                Box::new(ClassLoaderContext {
                    core: core.clone(),
                    system: system_clone.clone(),
                    jar_classes: Default::default(),
                }),
                JavaSvcFunctions::default(),
            )
            .await?;
            jvm.register_class(Box::new(loader_class), None).await.unwrap();
            let reserved = jvm.new_class("java/lang/OutOfMemoryError", "()V", ()).await.unwrap();
            jvm.put_static_field(
                "net/wie/KtfClassLoader",
                "outOfMemoryError",
                "Ljava/lang/OutOfMemoryError;",
                reserved.clone(),
            )
            .await
            .unwrap();

            // 1 GiB of ints: more than the whole guest heap.
            let Err(jvm::JavaError::JavaException(thrown)) = jvm.instantiate_array("I", 0x1000_0000).await else {
                panic!("an array larger than the heap must not allocate");
            };
            assert_eq!(thrown.identity(), reserved.identity());

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_array_classes_carry_object_vtable() -> Result<()> {
        // 6c9f969f089f calls equals(Object) on an int[] through the class's vtable; an array
        // class with none sent it to address 0. Each array class also needs its own vtable slot:
        // with a null vtable pointer every one of them took the index the next class then reused.
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;

            let mut indexes = Vec::new();
            for name in ["[I", "[B"] {
                let class = jvm.resolve_class(name).await.unwrap();
                let class = class
                    .definition
                    .as_any()
                    .downcast_ref::<super::JavaArrayClassDefinition>()
                    .unwrap()
                    .class
                    .clone();
                let vtable = super::vtable::JavaVtable::from_raw(&core, class.ptr_vtable()?);
                assert_ne!(class.ptr_vtable()?, 0, "{name} has no vtable");
                assert!(
                    vtable.find_method("equals", "(Ljava/lang/Object;)Z")?.is_some(),
                    "{name} vtable lacks equals"
                );
                indexes.push(KtfJvmSupport::get_vtable_index(&mut core, &class)?);
            }
            assert_ne!(indexes[0], indexes[1]);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    // A class the jar also ships as bytecode comes from client.bin; every other class stays parent-first.
    #[test]
    fn test_load_class_prefers_client_bin_over_jar_bytecode() -> Result<()> {
        async fn get_class(core: &mut ArmCore, natives: &mut Arc<Vec<(&'static str, u32)>>, ptr_name: u32) -> Result<u32> {
            let name = alloc::string::String::from_utf8(wie_util::read_null_terminated_string_bytes(core, ptr_name)?).unwrap();
            Ok(natives.iter().find(|(x, _)| *x == name).map_or(0, |(_, ptr)| *ptr))
        }

        // Both names resolve through the parent too (bootstrap protos, not loaded by init).
        const SHADOWED: &str = "org/kwis/msf/io/Network";
        const PLAIN: &str = "org/kwis/msf/io/Message";

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let java_functions = JavaSvcFunctions::default();
            core.register_svc_handler(5, handle_java_svc, &java_functions)?;
            let loader_class = JavaClassDefinition::new(
                &mut core.clone(),
                &jvm,
                KtfClassLoader::as_proto(),
                Box::new(ClassLoaderContext {
                    core: core.clone(),
                    system: system_clone.clone(),
                    jar_classes: Arc::new([SHADOWED.into()].into()),
                }),
                java_functions.clone(),
            )
            .await?;
            for method in loader_class.methods()? {
                let mut raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                raw.fn_body = core.make_svc_stub(5, method.ptr_raw)?;
                write_generic(&mut core, method.ptr_raw, raw)?;
            }
            jvm.register_class(Box::new(loader_class), None).await.unwrap();
            let mut loader = jvm.instantiate_class("net/wie/KtfClassLoader").await.unwrap();
            let system_loader: Box<dyn jvm::ClassInstance> = jvm
                .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", [])
                .await
                .unwrap();
            jvm.put_field(&mut loader, "parent", "Ljava/lang/ClassLoader;", system_loader)
                .await
                .unwrap();

            let mut natives = Vec::new();
            for name in [SHADOWED, PLAIN] {
                let class = JavaClassDefinition::new(
                    &mut core,
                    &jvm,
                    JavaClassProto {
                        name,
                        parent_class: Some("java/lang/Object"),
                        interfaces: vec![],
                        methods: vec![],
                        fields: vec![],
                        access_flags: ClassAccessFlags::PUBLIC,
                    },
                    Box::new(()),
                    java_functions.clone(),
                )
                .await?;
                natives.push((name, class.ptr_raw));
            }
            core.register_svc_handler(7, get_class, &Arc::new(natives.clone()))?;
            let ptr_functions = Allocator::alloc(&mut core, size_of::<ExeInterfaceFunctions>() as u32)?;
            let mut functions = ExeInterfaceFunctions::zeroed();
            functions.fn_get_class = core.make_svc_stub(7, 0u32)?;
            write_generic(&mut core, ptr_functions, functions)?;
            jvm.put_field(&mut loader, "nativeFunctions", "I", ptr_functions as i32).await.unwrap();

            for (name, native) in natives {
                assert!(!jvm.has_class(name));
                let java_name = JavaLangString::from_rust_string(&jvm, name).await.unwrap();
                let _: ClassInstanceRef<Class> = jvm
                    .invoke_virtual(
                        &loader,
                        "net/wie/KtfClassLoader",
                        "loadClass",
                        "(Ljava/lang/String;)Ljava/lang/Class;",
                        (java_name,),
                    )
                    .await
                    .unwrap();
                let loaded = KtfJvmSupport::class_definition_raw(&*jvm.resolve_class(name).await.unwrap().definition)?;
                assert_eq!(loaded == native, name == SHADOWED, "{name}");
            }

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_jvm_support() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, _core) = init_jvm(&mut system_clone).await?;

            let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("net/wie/WIPIMIDlet", "()V", ()).await.unwrap().into();
            let display: ClassInstanceRef<MidpDisplay> = MIDlet::display(&jvm, &midlet).await.unwrap();
            let paint_disabled: bool = jvm.get_field(&display, "paintDisabled", "Z").await.unwrap();
            assert!(!paint_disabled);

            KtfJvmSupport::disable_midp_paint(&jvm).await.unwrap();

            let paint_disabled: bool = jvm.get_field(&display, "paintDisabled", "Z").await.unwrap();
            assert!(paint_disabled);

            let string1 = JavaLangString::from_rust_string(&jvm, "test1").await.unwrap();
            let string2 = JavaLangString::from_rust_string(&jvm, "test2").await.unwrap();
            assert!(!string1.class_definition().fields().is_empty());

            let string3 = jvm
                .invoke_virtual(
                    &string1,
                    "java/lang/String",
                    "concat",
                    "(Ljava/lang/String;)Ljava/lang/String;",
                    [string2.into()],
                )
                .await
                .unwrap();

            assert_eq!(JavaLangString::to_rust_string(&jvm, &string3).await.unwrap(), "test1test2");

            let mut array = jvm.instantiate_array("S", 10).await.unwrap();
            jvm.store_array(&mut array, 0, (0..10i16).collect::<Vec<_>>()).await.unwrap();
            let temp: Vec<i16> = jvm.load_array(&array, 5, 4).await.unwrap();

            assert_eq!(temp, vec![5, 6, 7, 8]);

            let reference_array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            assert!(reference_array.class_definition().fields().is_empty());

            // test 64bit parameter passing
            let date = jvm.new_class("java/util/Date", "(J)V", (0x12345678_abcdef01i64,)).await.unwrap();
            let time: i64 = jvm.invoke_virtual(&date, "java/util/Date", "getTime", "()J", ()).await.unwrap();

            assert_eq!(time, 0x12345678_abcdef01);

            let calendar = jvm.new_class("java/util/GregorianCalendar", "()V", ()).await.unwrap();
            assert!(jvm.is_instance(&*calendar, "java/util/Calendar"));
            let cloneable = jvm.resolve_class("java/lang/Cloneable").await.unwrap();
            assert!(cloneable.definition.access_flags().contains(ClassAccessFlags::INTERFACE));

            done_clone.store(true, Ordering::Relaxed);

            Ok(())
        });

        loop {
            system.tick()?;
            if done.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_non_native_method_through_native_entry() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let mut chars = jvm.instantiate_array("C", 4).await.unwrap();
            jvm.store_array(&mut chars, 0, vec![0x41u16, 0xd654, 0xc7a5, 0x42]).await.unwrap();
            let class = jvm.resolve_class("java/lang/String").await.unwrap();
            let class = class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap();
            let method = class.method("valueOf", "([CII)Ljava/lang/String;", true)?.unwrap();
            let raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
            assert!(!MethodAccessFlags::from_bits_truncate(raw.access_flags).contains(MethodAccessFlags::NATIVE));
            assert_eq!(raw.exception_table_count, 0);
            assert_ne!(raw.fn_body_native_or_exception_table, 0);

            let args = vec![chars.clone().into(), 1.into(), 2.into()];
            let java_result = method.run(args.clone().into_boxed_slice()).await?;
            let java_result = Box::<dyn jvm::ClassInstance>::from(java_result);
            assert_eq!(JavaLangString::to_rust_string(&jvm, &java_result).await.unwrap(), "화장");

            let codec = JavaValueCodec::new(&core);
            let words = encode_method_arguments(&codec, &args);
            let ptr_args = Allocator::alloc(&mut core, words.len() as u32 * 4)?;
            for (index, word) in words.iter().enumerate() {
                write_generic(&mut core, ptr_args + index as u32 * 4, *word)?;
            }
            // KTF AOT callers can use the native argument-buffer ABI even when
            // the host implementation's Java prototype is not marked native.
            let result: u32 = core.run_function(raw.fn_body_native_or_exception_table, &[0, ptr_args]).await?;
            let result: Box<dyn jvm::ClassInstance> = Box::new(JavaClassInstance::from_raw(result, &core));
            assert_eq!(JavaLangString::to_rust_string(&jvm, &result).await.unwrap(), "화장");

            write_generic(&mut core, ptr_args + 4, (-1i32) as u32)?;
            let result = core.run_function::<u32>(raw.fn_body_native_or_exception_table, &[0, ptr_args]).await;
            assert!(matches!(result, Err(WieError::JavaException(_))));
            Allocator::free(&mut core, ptr_args, words.len() as u32 * 4)?;

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_native_method_entry_points() -> Result<()> {
        struct ReturnWords([u32; 2]);

        impl wie_core_arm::RunFunctionResult<ReturnWords> for ReturnWords {
            fn get(core: &ArmCore) -> Self {
                Self([core.read_param(0).unwrap(), core.read_param(1).unwrap()])
            }
        }

        let clock = TestClock::new();
        clock.set(0x12345678_9abcdef0);
        let mut system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let runtime = jvm.new_class("java/lang/Runtime", "()V", ()).await.unwrap();
            let mut source = jvm.instantiate_array("I", 4).await.unwrap();
            let mut destination = jvm.instantiate_array("I", 4).await.unwrap();
            jvm.store_array(&mut source, 0, vec![11i32, 22, 33, 44]).await.unwrap();

            for (class_name, name, descriptor, is_static, args, expected) in [
                (
                    "java/lang/Runtime",
                    "totalMemory",
                    "()J",
                    false,
                    vec![JavaValue::from(runtime)],
                    vec![0x100000, 0],
                ),
                ("java/lang/System", "currentTimeMillis", "()J", true, vec![], vec![0x9abcdef0, 0x12345678]),
                (
                    "java/lang/System",
                    "arraycopy",
                    "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                    true,
                    vec![source.into(), 1.into(), destination.clone().into(), 0.into(), 2.into()],
                    vec![0],
                ),
            ] {
                let class = jvm.resolve_class(class_name).await.unwrap();
                let class = class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap();
                let method = class.method(name, descriptor, is_static)?.unwrap();
                let raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
                assert!(MethodAccessFlags::from_bits_truncate(raw.access_flags).contains(MethodAccessFlags::NATIVE));
                assert_ne!(raw.fn_body, 0);
                assert_ne!(raw.fn_body_native_or_exception_table, 0);
                assert_ne!(raw.fn_body, raw.fn_body_native_or_exception_table);

                for native_entry in [false, true] {
                    jvm.store_array(&mut destination, 0, vec![0i32; 4]).await.unwrap();
                    let codec = JavaValueCodec::new(&core);
                    let actual = if native_entry {
                        let result = method.run(args.clone().into_boxed_slice()).await?;
                        encode_method_arguments(&codec, &[result])
                    } else {
                        let mut params = vec![0];
                        params.extend(encode_method_arguments(&codec, &args));
                        let result = core.run_function::<ReturnWords>(raw.fn_body, &params).await?;
                        result.0[..expected.len()].to_vec()
                    };
                    assert_eq!(actual, expected, "{name}, native_entry={native_entry}");
                    if name == "arraycopy" {
                        assert_eq!(jvm.load_array::<i32>(&destination, 0, 4).await.unwrap(), vec![22, 33, 0, 0]);
                    }
                }

                // The JB interface `call_native` slot writes the result back into the argument
                // container; a `J` caller reads both words from it (귀신사냥2007 KTF).
                let codec = JavaValueCodec::new(&core);
                let words = encode_method_arguments(&codec, &args);
                let container = Allocator::alloc(&mut core, (words.len().max(2) * 4) as u32)?;
                for (i, word) in words.iter().enumerate() {
                    write_generic(&mut core, container + (i * 4) as u32, *word)?;
                }
                let _ = interface::call_native(&mut core, &mut (), raw.fn_body_native_or_exception_table, container).await?;
                let slot: [u32; 2] = [read_generic(&core, container)?, read_generic(&core, container + 4)?];
                assert_eq!(slot[..expected.len()], expected[..], "{name}, call_native");
            }

            done_clone.store(true, Ordering::Relaxed);
            clock.advance(16);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_call_native_reads_result_published_in_thread_context() -> Result<()> {
        use wie_util::ByteWrite;

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (_jvm, mut core) = init_jvm(&mut system_clone).await?;
            let ptr_thread_context = KtfJvmSupport::current_thread_context(&core)?;
            let ptr_slot = ptr_thread_context + core::mem::offset_of!(KtfJvmThreadContext, native_result_type) as u32;

            // The KTF AID 0103BF27 shape: tag 2 at +0x24, the int at +0x28, an unrelated constant in r0.
            let publishing = Allocator::alloc(&mut core, 24)?;
            let mut code = Vec::new();
            for half in [0x4a03u16, 0x2302, 0x6253, 0x4b03, 0x6293, 0x2007, 0x4770, 0x46c0] {
                // ldr r2,=ctx; movs r3,#2; str r3,[r2,#0x24]; ldr r3,=value; str r3,[r2,#0x28]; movs r0,#7; bx lr
                code.extend_from_slice(&half.to_le_bytes());
            }
            code.extend_from_slice(&ptr_thread_context.to_le_bytes());
            code.extend_from_slice(&0x1234_5678u32.to_le_bytes());
            core.write_bytes(publishing, &code)?;

            // An r0-only native: movs r0,#7; bx lr
            let plain = Allocator::alloc(&mut core, 4)?;
            core.write_bytes(plain, &[0x07, 0x20, 0x70, 0x47])?;

            let container = Allocator::alloc(&mut core, 8)?;

            let _ = interface::call_native(&mut core, &mut (), publishing | 1, container).await?;
            let slot: [u32; 2] = read_generic(&core, container)?;
            assert_eq!(slot, [0x1234_5678, 0], "published int wins over r0");

            // An outer native's pending result must survive an inner call, and an r0-only native
            // must not be answered by it.
            write_generic(&mut core, ptr_slot, [2u32, 0xdead, 0])?;
            let _ = interface::call_native(&mut core, &mut (), plain | 1, container).await?;
            let slot: u32 = read_generic(&core, container)?;
            assert_eq!(slot, 7, "a tag left by an outer call does not answer this one");
            let restored: [u32; 3] = read_generic(&core, ptr_slot)?;
            assert_eq!(restored, [2, 0xdead, 0], "the outer call's slot is put back");

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_catch_handler_receives_thrown_exception() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;

            // One catch-all entry covering pc 0..10, target 0x1dd — the shape of the AOT frame
            // whose handler calls a method on `e`.
            let ptr_entry = Allocator::alloc(&mut core, size_of::<RawJavaMethodExceptionTableEntry>() as u32)?;
            write_generic(
                &mut core,
                ptr_entry,
                RawJavaMethodExceptionTableEntry {
                    from_pc: 0,
                    to_pc: 10,
                    target: 0x1dd,
                    ptr_class: 0,
                },
            )?;
            let ptr_table = Allocator::alloc(&mut core, 4)?;
            write_generic(&mut core, ptr_table, ptr_entry)?;
            let ptr_method = Allocator::alloc(&mut core, size_of::<RawJavaMethod>() as u32)?;
            let mut method = RawJavaMethod::zeroed();
            method.fn_body_native_or_exception_table = ptr_table;
            method.exception_table_count = 1;
            write_generic(&mut core, ptr_method, method)?;

            let ptr_functions = Allocator::alloc(&mut core, 8)?;
            write_generic(&mut core, ptr_functions + 4, 0x1234u32)?;
            let ptr_handler = Allocator::alloc(&mut core, size_of::<RawJavaExceptionHandler>() as u32)?;
            let mut handler = RawJavaExceptionHandler::zeroed();
            handler.ptr_method = ptr_method;
            handler.current_pc = 5;
            handler.ptr_functions = ptr_functions;
            write_generic(&mut core, ptr_handler, handler)?;

            let ptr_thread_context = KtfJvmSupport::current_thread_context(&core)?;
            let mut thread_context: KtfJvmThreadContext = read_generic(&core, ptr_thread_context)?;
            thread_context.current_java_exception_handler = ptr_handler;
            write_generic(&mut core, ptr_thread_context, thread_context)?;

            let exception = jvm.new_class("java/lang/NullPointerException", "()V", ()).await.unwrap();
            let ptr_exception = KtfJvmSupport::class_instance_raw(&exception);
            let result = JavaMethod::handle_exception(&mut core, &jvm, exception).await;
            assert!(matches!(
                result,
                Err(WieError::JavaExceptionUnwind {
                    target: 0x1dd,
                    next_pc: 0x1234,
                    ..
                })
            ));

            let handler: RawJavaExceptionHandler = read_generic(&core, ptr_handler)?;
            assert_eq!(handler.unk3, ptr_exception, "the catch block reads `e` from this slot");

            done_clone.store(true, Ordering::Relaxed);

            Ok(())
        });

        loop {
            system.tick()?;
            if done.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }

    /// A catch in the CALLER's frame: the innermost record's table does not cover its pc, so the
    /// throw walks `ptr_old_handler` to the frame that does, and that record becomes current — the
    /// inner frame is gone (docs/report/0445).
    #[test]
    fn test_throw_walks_to_the_caller_frames_catch() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;

            let record = |core: &mut ArmCore, from_pc: u32, to_pc: u32, target: u32, current_pc: u32, ptr_old_handler: u32| -> Result<u32> {
                let ptr_entry = Allocator::alloc(core, size_of::<RawJavaMethodExceptionTableEntry>() as u32)?;
                write_generic(
                    core,
                    ptr_entry,
                    RawJavaMethodExceptionTableEntry {
                        from_pc,
                        to_pc,
                        target,
                        ptr_class: 0,
                    },
                )?;
                let ptr_table = Allocator::alloc(core, 4)?;
                write_generic(core, ptr_table, ptr_entry)?;
                let ptr_method = Allocator::alloc(core, size_of::<RawJavaMethod>() as u32)?;
                let mut method = RawJavaMethod::zeroed();
                method.fn_body_native_or_exception_table = ptr_table;
                method.exception_table_count = 1;
                write_generic(core, ptr_method, method)?;

                let ptr_functions = Allocator::alloc(core, 8)?;
                write_generic(core, ptr_functions + 4, 0x1234u32)?;
                let ptr_handler = Allocator::alloc(core, size_of::<RawJavaExceptionHandler>() as u32)?;
                let mut handler = RawJavaExceptionHandler::zeroed();
                handler.ptr_method = ptr_method;
                handler.current_pc = current_pc;
                handler.ptr_old_handler = ptr_old_handler;
                handler.ptr_functions = ptr_functions;
                write_generic(core, ptr_handler, handler)?;
                Ok(ptr_handler)
            };
            // caller: try covers pc 0..10, it is at pc 5 · callee: its try covers 0..10, it is at pc 20 (outside)
            let outer = record(&mut core, 0, 10, 0x2ee, 5, 0)?;
            let inner = record(&mut core, 0, 10, 0x1dd, 20, outer)?;

            let ptr_thread_context = KtfJvmSupport::current_thread_context(&core)?;
            let mut thread_context: KtfJvmThreadContext = read_generic(&core, ptr_thread_context)?;
            thread_context.current_java_exception_handler = inner;
            write_generic(&mut core, ptr_thread_context, thread_context)?;

            let exception = jvm.new_class("java/lang/NullPointerException", "()V", ()).await.unwrap();
            let result = JavaMethod::handle_exception(&mut core, &jvm, exception).await;
            assert!(
                matches!(result, Err(WieError::JavaExceptionUnwind { target: 0x2ee, context_base, .. }) if context_base == outer + 24),
                "the caller's catch, not a host error"
            );
            assert_eq!(KtfJvmSupport::current_java_exception_handler(&mut core)?, outer);

            // A JVM→guest call starts with an empty chain and hands the caller's back on the way out,
            // so a throw that escapes the call cannot leave the dead frames' records registered.
            let scope = KtfJvmSupport::enter_exception_scope(&mut core);
            assert_eq!(KtfJvmSupport::current_java_exception_handler(&mut core)?, 0);
            KtfJvmSupport::set_current_java_exception_handler(&mut core, inner)?; // a frame that never unregistered
            KtfJvmSupport::leave_exception_scope(&mut core, scope);
            assert_eq!(KtfJvmSupport::current_java_exception_handler(&mut core)?, outer);

            done_clone.store(true, Ordering::Relaxed);

            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    // The scope is entered by the JVM→guest boundary itself (`Method::run`), not by its callers: the
    // body sees an empty chain however deep the caller's was, and the caller's is back afterwards.
    #[test]
    fn test_jvm_call_runs_in_its_own_exception_scope() -> Result<()> {
        async fn probe(_jvm: &Jvm, core: &mut ArmCore) -> jvm::Result<i32> {
            Ok(KtfJvmSupport::current_java_exception_handler(core).unwrap() as i32)
        }

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let java_functions = JavaSvcFunctions::default();
            core.register_svc_handler(5, handle_java_svc, &java_functions)?;
            let class = JavaClassDefinition::new(
                &mut core.clone(),
                &jvm,
                JavaClassProto {
                    name: "test/Probe",
                    parent_class: Some("java/lang/Object"),
                    interfaces: vec![],
                    methods: vec![JavaMethodProto::new("probe", "()I", probe, MethodAccessFlags::STATIC)],
                    fields: vec![],
                    access_flags: ClassAccessFlags::PUBLIC,
                },
                Box::new(core.clone()),
                java_functions.clone(),
            )
            .await?;
            let method = class.method("probe", "()I", true)?.unwrap();
            let mut raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;
            raw.fn_body = core.make_svc_stub(5, method.ptr_raw)?;
            write_generic(&mut core, method.ptr_raw, raw)?;

            KtfJvmSupport::set_current_java_exception_handler(&mut core, 0x1234)?; // the caller's chain
            let seen = <JavaMethod as jvm::Method>::run(&method, &jvm, Box::new([])).await.unwrap();
            assert!(matches!(seen, JavaValue::Int(0)), "the body ran under the caller's chain: {seen:?}");
            assert_eq!(KtfJvmSupport::current_java_exception_handler(&mut core)?, 0x1234);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    #[test]
    fn test_exception_class_matches_raw_class_and_vtable() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;

            let exception = jvm.new_class("java/lang/NullPointerException", "()V", ()).await.unwrap();
            let null_pointer_class = jvm
                .resolve_class("java/lang/NullPointerException")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();
            let runtime_exception_class = jvm
                .resolve_class("java/lang/RuntimeException")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();
            let illegal_argument_class = jvm
                .resolve_class("java/lang/IllegalArgumentException")
                .await
                .unwrap()
                .definition
                .as_any()
                .downcast_ref::<JavaClassDefinition>()
                .unwrap()
                .clone();

            assert!(JavaMethod::exception_class_matches(&core, &jvm, &*exception, 0)?);
            assert!(JavaMethod::exception_class_matches(&core, &jvm, &*exception, null_pointer_class.ptr_raw)?);
            assert!(JavaMethod::exception_class_matches(
                &core,
                &jvm,
                &*exception,
                null_pointer_class.ptr_vtable()?
            )?);
            assert!(JavaMethod::exception_class_matches(
                &core,
                &jvm,
                &*exception,
                runtime_exception_class.ptr_vtable()?
            )?);
            assert!(!JavaMethod::exception_class_matches(
                &core,
                &jvm,
                &*exception,
                illegal_argument_class.ptr_vtable()?
            )?);

            let ptr_exception = KtfJvmSupport::class_instance_raw(&exception);
            let result = JavaMethod::handle_exception(&mut core, &jvm, exception).await;
            assert!(matches!(result, Err(WieError::JavaException(ptr)) if ptr == ptr_exception));

            done_clone.store(true, Ordering::Relaxed);

            Ok(())
        });

        loop {
            system.tick()?;
            if done.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_long_array_store_load() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, core) = init_jvm(&mut system_clone).await?;

            let values = vec![i64::MIN, -1, 0x12345678_9abcdef0, i64::MAX];

            let mut array = jvm.instantiate_array("J", 4).await.unwrap();
            jvm.store_array(&mut array, 0, values.clone()).await.unwrap();
            let loaded: Vec<i64> = jvm.load_array(&array, 0, 4).await.unwrap();

            assert_eq!(loaded, values);

            // guard against store/load flipping words symmetrically: check raw guest memory layout
            let array_instance = JavaArrayClassInstance::from_raw(KtfJvmSupport::class_instance_raw(&array), &core);
            let mut raw = [0u8; 8];
            array_instance.load_raw(16, &mut raw)?;
            assert_eq!(raw, 0x12345678_9abcdef0u64.to_le_bytes());

            done_clone.store(true, Ordering::Relaxed);

            Ok(())
        });

        loop {
            system.tick()?;
            if done.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }

    #[test]
    fn test_double_array_store_load() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);

        let done = Arc::new(AtomicBool::new(false));

        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, _core) = init_jvm(&mut system_clone).await?;

            let values = vec![f64::MIN_POSITIVE, -1.5, f64::MAX];

            let mut array = jvm.instantiate_array("D", 3).await.unwrap();
            jvm.store_array(&mut array, 0, values.clone()).await.unwrap();
            let loaded: Vec<f64> = jvm.load_array(&array, 0, 3).await.unwrap();

            let to_bits = |x: &Vec<f64>| x.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
            assert_eq!(to_bits(&loaded), to_bits(&values));

            done_clone.store(true, Ordering::Relaxed);

            Ok(())
        });

        loop {
            system.tick()?;
            if done.load(Ordering::Relaxed) {
                break;
            }
        }

        Ok(())
    }

    // A context in which no guest register points anywhere: what survives a collection is then what
    // the guest roots it is given, plus the jvm's own roots.
    fn clear_guest_registers(core: &mut ArmCore) {
        let mut context = core.save_context();
        (context.r0, context.r1, context.r2, context.r3, context.r4, context.r5, context.r6) = (0, 0, 0, 0, 0, 0, 0);
        (context.r7, context.r8, context.sb, context.sl, context.fp, context.ip, context.lr) = (0, 0, 0, 0, 0, 0, 0);
        core.restore_context(&context);
    }

    // 8d8c24b7c198: the `String` a host method made for the guest is garbage once the guest drops it.
    // Without a frame per call it was rooted in the frame of the host call the guest code ran under —
    // for a game loop, one that never returns — and the guest heap ran out after ~7.5 minutes.
    #[test]
    fn test_what_a_host_method_made_for_the_guest_is_collected_once_dropped() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let class = jvm.resolve_class("java/lang/String").await.unwrap();
            let class = class.definition.as_any().downcast_ref::<JavaClassDefinition>().unwrap();
            let method = class.method("valueOf", "(I)Ljava/lang/String;", true)?.unwrap();
            let raw: RawJavaMethod = read_generic(&core, method.ptr_raw)?;

            jvm.push_native_frame(); // the long-lived host frame a game loop runs under
            let ptr_string: u32 = core.run_function(raw.fn_body, &[0, 1234]).await?;
            let string: Box<dyn jvm::ClassInstance> = Box::new(JavaClassInstance::from_raw(ptr_string, &core));
            assert_eq!(JavaLangString::to_rust_string(&jvm, &string).await.unwrap(), "1234");
            drop(string);
            clear_guest_registers(&mut core);

            jvm.collect_garbage().unwrap();
            assert!(!Allocator::is_allocated(&core, ptr_string, size_of::<RawJavaClassInstance>() as u32)?);
            jvm.pop_frame();

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    // The same for the guest's own `new`: `instantiate_class` roots it in the top frame, which the
    // init SVC now pushes per call.
    #[test]
    fn test_a_guest_new_the_guest_dropped_is_collected() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            crate::runtime::init::register_init_svc_handler(&mut core, &jvm)?;
            let java_new = core.make_svc_stub(crate::runtime::SVC_CATEGORY_INIT, crate::runtime::svc_ids::InitSvcId::JavaNew)?;
            let java_array_new = core.make_svc_stub(crate::runtime::SVC_CATEGORY_INIT, crate::runtime::svc_ids::InitSvcId::JavaArrayNew)?;
            let class = jvm.resolve_class("java/lang/Object").await.unwrap();
            let ptr_class = KtfJvmSupport::class_definition_raw(&*class.definition)?;

            jvm.push_native_frame(); // the long-lived host frame a game loop runs under
            let ptr_object: u32 = core.run_function(java_new, &[ptr_class]).await?;
            let ptr_array: u32 = core.run_function(java_array_new, &[b'I' as u32, 4]).await?;
            clear_guest_registers(&mut core);

            jvm.collect_garbage().unwrap();
            assert!(!Allocator::is_allocated(&core, ptr_object, size_of::<RawJavaClassInstance>() as u32)?);
            assert!(!Allocator::is_allocated(&core, ptr_array, size_of::<RawJavaClassInstance>() as u32)?);
            jvm.pop_frame();

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    // The other direction, where a miss is a use-after-free: a guest word keeps its object alive — a
    // pointer to it, or into its field storage (AOT code may keep only `ptr_fields` or an element
    // address across a call), or one past that storage's end (a loop's end pointer).
    #[test]
    fn test_a_guest_word_into_an_object_keeps_it_alive() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            let instance_size = size_of::<RawJavaClassInstance>() as u32;
            for holder in ["object", "storage", "element", "storage end"] {
                jvm.push_native_frame();
                let array = jvm.instantiate_array("I", 4).await.unwrap();
                let ptr_array = KtfJvmSupport::class_instance_raw(&array);
                drop(array);
                jvm.pop_frame();
                // KTF storage: the vtable-index word, the length, then the elements.
                let ptr_fields: u32 = read_generic(&core, ptr_array)?;
                let word = match holder {
                    "object" => ptr_array,
                    "storage" => ptr_fields,
                    "element" => ptr_fields + 8 + 2 * 4,
                    _ => ptr_fields + 8 + 4 * 4,
                };
                clear_guest_registers(&mut core);
                let mut context = core.save_context();
                context.r5 = word;
                core.restore_context(&context);

                jvm.collect_garbage().unwrap();
                assert!(
                    Allocator::is_allocated(&core, ptr_array, instance_size)?,
                    "kept by a guest word at its {holder}"
                );

                clear_guest_registers(&mut core);
                jvm.collect_garbage().unwrap();
                assert!(
                    !Allocator::is_allocated(&core, ptr_array, instance_size)?,
                    "freed once that word is gone ({holder})"
                );
            }

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }

    /// KTF `MC_grpCopyFrameBuffer`/`MC_grpDrawImage` run the proc set with `MC_grpSetContext(PIXELOP)`
    /// as `proc(src, dst, param1)` on 16-bit pixels (docs/report/0446).
    ///
    /// Two titles key glyphs and sprites this way; the blits ignored the context and drew the key as
    /// magenta. The proc here is written for the test: `src == 0xf81f ? dst : src`.
    #[test]
    fn test_wipic_blits_run_the_context_pixel_op() -> Result<()> {
        use crate::runtime::{
            SVC_CATEGORY_WIPIC,
            svc_ids::{WIPICGraphicsMethodId, WIPICTableId},
            wipi_c::register_wipic_svc_handler,
        };

        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let mut system_clone = system.clone();
        system.spawn(async move || {
            let (jvm, mut core) = init_jvm(&mut system_clone).await?;
            register_wipic_svc_handler(&mut core, &system_clone, &jvm)?;
            let svc = |core: &mut ArmCore, id| core.make_svc_stub(SVC_CATEGORY_WIPIC, WIPICTableId::Graphics.function_id(id));
            let (init, set) = (svc(&mut core, WIPICGraphicsMethodId::InitContext)?, svc(&mut core, WIPICGraphicsMethodId::SetContext)?);
            let (copy, draw) = (svc(&mut core, WIPICGraphicsMethodId::CopyFrameBuffer)?, svc(&mut core, WIPICGraphicsMethodId::DrawImage)?);

            let proc_code = Allocator::alloc(&mut core, 14)?;
            // movs r3,#0xf8; lsls r3,r3,#8; adds r3,#0x1f; cmp r0,r3; bne +0; movs r0,r1; bx lr
            write_generic(&mut core, proc_code, [0x23f8u16, 0x021b, 0x331f, 0x4298, 0xd100, 0x0008, 0x4770])?;

            // KTF handles are indirect: `*handle` is a block whose data starts at +8.
            let indirect = |core: &mut ArmCore, data: &[u32]| -> Result<u32> {
                let block = Allocator::alloc(core, 8 + data.len() as u32 * 4)?;
                for (i, &word) in data.iter().enumerate() {
                    write_generic(core, block + 8 + i as u32 * 4, word)?;
                }
                let handle = Allocator::alloc(core, 4)?;
                write_generic(core, handle, block)?;
                Ok(handle)
            };
            let pixels = |core: &ArmCore, buf: u32| -> Result<u32> { read_generic(core, read_generic::<u32, _>(core, buf)? + 8) };
            let src_buf = indirect(&mut core, &[0x1234_f81f])?; // [key, 0x1234]
            let dst_buf = indirect(&mut core, &[0xaaaa_aaaa])?;
            let src = indirect(&mut core, &[2, 1, 4, 16, src_buf])?;
            let dst = indirect(&mut core, &[2, 1, 4, 16, dst_buf])?;
            let fill = |core: &mut ArmCore, buf: u32| -> Result<()> { write_generic(core, read_generic::<u32, _>(core, buf)? + 8, 0xaaaa_aaaau32) };

            let record = Allocator::alloc(&mut core, 52)?;
            let _: u32 = core.run_function(init, &[record]).await?;
            let _: u32 = core.run_function(copy, &[dst, 0, 0, 2, 1, src, 0, 0, record]).await?;
            assert_eq!(pixels(&core, dst_buf)?, 0x1234_f81f, "no pixel op: a plain copy");

            fill(&mut core, dst_buf)?;
            let _: u32 = core.run_function(set, &[record, 5, proc_code | 1]).await?; // PixelopIdx
            let _: u32 = core.run_function(copy, &[dst, 0, 0, 2, 1, src, 0, 0, record]).await?;
            assert_eq!(pixels(&core, dst_buf)?, 0x1234_aaaa, "the key keeps dst, the rest takes src");

            // An image record starts with its framebuffer: 2x1, 32bpp, [opaque magenta, #123456].
            let argb = indirect(&mut core, &[0xffff_00ff, 0xff12_3456])?;
            let image = indirect(&mut core, &[2, 1, 8, 32, argb])?;
            fill(&mut core, dst_buf)?;
            let _: u32 = core.run_function(draw, &[dst, 0, 0, 2, 1, image, 0, 0, record]).await?;
            assert_eq!(pixels(&core, dst_buf)?, 0x11aa_aaaa, "#ff00ff is the key; #123456 lands as RGB565 0x11aa");

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });
        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }
        Ok(())
    }
}
