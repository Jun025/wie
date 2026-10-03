use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    sync::Arc,
    vec::Vec,
};
use core::time::Duration;

use spin::Mutex;

use jvm::{ClassDefinition, ClassInstance, Field, JavaValue, Jvm, Result as JvmResult};
use jvm_bytecode::{ArrayClassDefinitionImpl, ClassDefinitionError, ClassDefinitionImpl};
use jvm_types::FieldAccessFlags;
use rustjava_runtime::{
    File, FileDescriptorId, FileOpenOptions, FileSize, FileStat, FileType, IOError, IOResult, RT_RUSTJAR, Runtime, SpawnCallback,
    get_runtime_class_proto,
};

use wie_backend::{AsyncCallable, System};
use wie_util::WieError;

use crate::{JvmImplementation, JvmSupport, WIE_RUSTJAR, WieJavaClassProto, WieJvmContext, jvm_implementation::InheritedMethods};

mod file;

use file::FileImpl;

const STDOUT_FD: u32 = 1;
const STDERR_FD: u32 = 2;

struct FileTableInner {
    files: BTreeMap<u32, Box<dyn File>>,
    next_id: u32,
}

impl FileTableInner {
    fn new() -> Self {
        Self {
            files: BTreeMap::new(),
            next_id: 3, // 0=stdin, 1=stdout, 2=stderr
        }
    }

    fn add(&mut self, file: Box<dyn File>) -> FileDescriptorId {
        let id = self.next_id;
        self.next_id += 1;
        self.files.insert(id, file);
        FileDescriptorId::new(id)
    }
}

#[derive(Clone)]
struct StdoutFile {
    system: System,
}

#[async_trait::async_trait]
impl File for StdoutFile {
    async fn read(&mut self, _buf: &mut [u8]) -> IOResult<usize> {
        Err(IOError::Unsupported)
    }

    async fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.system.platform().write_stdout(buf);

        Ok(buf.len())
    }

    async fn seek(&mut self, _pos: FileSize) -> IOResult<()> {
        Err(IOError::Unsupported)
    }

    async fn tell(&self) -> IOResult<FileSize> {
        Err(IOError::Unsupported)
    }

    async fn set_len(&mut self, _len: FileSize) -> IOResult<()> {
        Err(IOError::Unsupported)
    }

    async fn metadata(&self) -> IOResult<FileStat> {
        Err(IOError::Unsupported)
    }
}

#[derive(Clone)]
struct StderrFile {
    system: System,
}

#[async_trait::async_trait]
impl File for StderrFile {
    async fn read(&mut self, _buf: &mut [u8]) -> IOResult<usize> {
        Err(IOError::Unsupported)
    }

    async fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        self.system.platform().write_stderr(buf);

        Ok(buf.len())
    }

    async fn seek(&mut self, _pos: FileSize) -> IOResult<()> {
        Err(IOError::Unsupported)
    }

    async fn tell(&self) -> IOResult<FileSize> {
        Err(IOError::Unsupported)
    }

    async fn set_len(&mut self, _len: FileSize) -> IOResult<()> {
        Err(IOError::Unsupported)
    }

    async fn metadata(&self) -> IOResult<FileStat> {
        Err(IOError::Unsupported)
    }
}

/// Every class this runtime defined, to cut the reference cycles among its objects once the JVM is
/// dropped.
///
/// The `jvm` crate frees an object only when its last `Arc` goes, so any cycle outlives the JVM: a
/// static holding an instance of its own class (the instance holds its class), `Writer.lock = this`,
/// `Display` ↔ the current `Canvas`. Each boot leaked those objects and, through their classes'
/// context, the `System` and its platform (~0.5 MiB a boot, measured). Nothing can run Java once the
/// JVM is gone, so every object reachable from a static is emptied then.
#[derive(Clone, Default)]
pub struct DefinedClasses(Arc<Mutex<Vec<Box<dyn ClassDefinition>>>>);

impl DefinedClasses {
    fn record(&self, class: JvmResult<Box<dyn ClassDefinition>>) -> JvmResult<Box<dyn ClassDefinition>> {
        // Only classes whose objects live in the `jvm` crate: KTF and LGT define theirs in guest memory.
        if let Ok(class) = &class
            && (**class).as_any().is::<InheritedMethods>()
        {
            self.0.lock().push(class.clone());
        }
        class
    }

    pub fn sever(&self) {
        // Taken, not iterated in place: these definitions hold this list through their context.
        let mut classes = core::mem::take(&mut *self.0.lock());
        let by_name = classes.iter().map(|x| (x.name(), x.clone())).collect::<BTreeMap<_, _>>();

        let mut pending = Vec::new();
        for class in &mut classes {
            for field in class.fields() {
                if field.access_flags().contains(FieldAccessFlags::STATIC)
                    && let Some(value) = take_object(class.get_static_field(&*field), &*field)
                {
                    pending.push(value);
                    let _ = class.put_static_field(&*field, JavaValue::Object(None));
                }
            }
        }

        let mut seen = BTreeSet::new();
        while let Some(mut object) = pending.pop() {
            if !seen.insert(object.identity()) {
                continue;
            }
            let definition = object.class_definition();
            let name = definition.name();
            if (*definition).as_any().is::<ArrayClassDefinitionImpl>() {
                let Some(array) = object.as_array_instance_mut() else { continue };
                if name.starts_with("[L") || name.starts_with("[[") {
                    let length = array.length();
                    let values = array.load(0, length).unwrap_or_default();
                    pending.extend(values.into_iter().filter_map(|x| if let JavaValue::Object(x) = x { x } else { None }));
                    let _ = array.store(0, (0..length).map(|_| JavaValue::Object(None)).collect());
                }
                continue;
            }
            // An instance holds the definition `InheritedMethods` wraps.
            if !(*definition).as_any().is::<ClassDefinitionImpl>() {
                continue;
            }
            let mut class = by_name.get(&name);
            while let Some(definition) = class {
                for field in definition.fields() {
                    if !field.access_flags().contains(FieldAccessFlags::STATIC)
                        && let Some(value) = take_object(object.get_field(&*field), &*field)
                    {
                        pending.push(value);
                        let _ = object.put_field(&*field, JavaValue::Object(None));
                    }
                }
                class = definition.super_class_name().and_then(|x| by_name.get(&x));
            }
        }
    }
}

fn take_object(value: JvmResult<JavaValue>, field: &dyn Field) -> Option<Box<dyn ClassInstance>> {
    let descriptor = field.descriptor();
    if !(descriptor.starts_with('L') || descriptor.starts_with('[')) {
        return None;
    }
    match value {
        Ok(JavaValue::Object(x)) => x,
        _ => None,
    }
}

#[derive(Clone)]
pub struct JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    system: System,
    implementation: T,
    protos: Arc<Mutex<Vec<WieJavaClassProto>>>,
    file_table: Arc<Mutex<FileTableInner>>,
    pub(crate) classes: DefinedClasses,
}

impl<T> JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    pub fn new(system: System, implementation: T, protos: Box<[Box<[WieJavaClassProto]>]>) -> Self {
        let mut file_table = FileTableInner::new();
        file_table.files.insert(STDOUT_FD, Box::new(StdoutFile { system: system.clone() }));
        file_table.files.insert(STDERR_FD, Box::new(StderrFile { system: system.clone() }));

        Self {
            system,
            implementation,
            protos: Arc::new(Mutex::new(protos.into_vec().into_iter().flat_map(|x| x.into_vec()).collect())),
            file_table: Arc::new(Mutex::new(file_table)),
            classes: DefinedClasses::default(),
        }
    }
}

#[async_trait::async_trait]
impl<T> Runtime for JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    async fn sleep(&self, duration: Duration) {
        self.system.guest_sleep(duration.as_millis() as _).await;
    }

    async fn r#yield(&self) {
        self.system.guest_yielded();
        self.system.yield_now().await;
    }

    fn spawn(&self, jvm: &Jvm, callback: Box<dyn SpawnCallback>) {
        struct SpawnProxy {
            jvm: Jvm,
            callback: Box<dyn SpawnCallback>,
        }

        impl AsyncCallable<Result<(), WieError>> for SpawnProxy {
            async fn call(self) -> Result<(), WieError> {
                let result = self.callback.call().await;
                if let Err(err) = result {
                    self.jvm.attach_thread(None).await.unwrap();

                    let result = Err(JvmSupport::to_wie_err(&self.jvm, err).await);

                    self.jvm.detach_thread().unwrap();

                    return result;
                }

                Ok(())
            }
        }

        self.system.spawn(SpawnProxy { jvm: jvm.clone(), callback });
    }

    /// `System.exit(int)` — the guest asked to stop. dlunch/RustJava's own host
    /// calls `std::process::exit` here; we cannot (the emulator is a library on
    /// both hosts), so we route it through the platform's normal shutdown path —
    /// the same one WIPI `MC_knlExit` uses, which is what `wie_featurephone`'s sticky
    /// `has_exited()` observes. The status code is not propagated: neither host
    /// surface has a place to put it.
    fn exit(&self, status: i32) {
        tracing::debug!("Runtime::exit({status})");

        self.system.platform().exit();
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn current_task_id(&self) -> u64 {
        self.system.current_task_id()
    }

    fn stdin(&self) -> IOResult<FileDescriptorId> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> IOResult<FileDescriptorId> {
        Ok(FileDescriptorId::new(STDOUT_FD))
    }

    fn stderr(&self) -> IOResult<FileDescriptorId> {
        Ok(FileDescriptorId::new(STDERR_FD))
    }

    async fn open(&self, path: &str, options: FileOpenOptions) -> IOResult<FileDescriptorId> {
        tracing::debug!("open({path:?}, {options:?})");

        let file = FileImpl::new(self.system.clone(), path, options).await?;
        Ok(self.file_table.lock().add(Box::new(file)))
    }

    fn get_file(&self, fd: FileDescriptorId) -> IOResult<Box<dyn File>> {
        self.file_table.lock().files.get(&fd.id()).cloned().ok_or(IOError::NotFound)
    }

    fn close_file(&self, fd: FileDescriptorId) {
        self.file_table.lock().files.remove(&fd.id());
    }

    async fn unlink(&self, _path: &str) -> IOResult<()> {
        Err(IOError::Unsupported)
    }

    async fn metadata(&self, path: &str) -> IOResult<FileStat> {
        if path.is_empty() || path.ends_with("/") {
            return Ok(FileStat {
                size: 0,
                r#type: FileType::Directory,
            });
        }

        let size = self.system.filesystem().size(path).await.ok_or(IOError::NotFound)?;

        Ok(FileStat {
            size: size as _,
            r#type: FileType::File,
        })
    }

    async fn find_rustjar_class(&self, jvm: &Jvm, classpath: &str, class: &str) -> JvmResult<Option<Box<dyn ClassDefinition>>> {
        let class = class.replace('.', "/");

        if classpath == RT_RUSTJAR {
            let proto = get_runtime_class_proto(&class);
            if let Some(mut proto) = proto {
                // Re-add the null guards the dropped RustJava fork carried (see
                // `hardening`): the proto passes through here before the class is
                // defined, so no fork is needed to wrap a method body.
                crate::hardening::harden(&mut proto);

                return Ok(Some(self.classes.record(
                    self.implementation.define_class_rust(jvm, proto, Box::new(self.clone()) as Box<_>).await,
                )?));
            }
        } else if classpath == WIE_RUSTJAR {
            let proto_index = self.protos.lock().iter().position(|x| x.name == class);
            if let Some(proto_index) = proto_index {
                let proto = self.protos.lock().remove(proto_index);
                let context = Box::new(WieJvmContext::new(&self.system));

                return Ok(Some(
                    self.classes
                        .record(self.implementation.define_class_rust(jvm, proto, context as Box<_>).await)?,
                ));
            }
        }

        Ok(None)
    }

    async fn define_class(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        match ClassDefinitionImpl::from_classfile(data) {
            Ok(class) => self.classes.record(InheritedMethods::wrap(jvm, class).await),
            Err(ClassDefinitionError::InvalidClassFile) => Err(jvm.exception("java/lang/ClassFormatError", "Invalid class file").await),
            Err(ClassDefinitionError::UnsupportedClassVersion(version)) => Err(jvm
                .exception(
                    "java/lang/UnsupportedClassVersionError",
                    &format!("Unsupported class file version {version}"),
                )
                .await),
            Err(ClassDefinitionError::Verification) => Err(jvm.exception("java/lang/VerifyError", "Bytecode verification failed").await),
            Err(ClassDefinitionError::UnsupportedFeature(feature)) => Err(jvm
                .exception(
                    "java/lang/UnsupportedOperationException",
                    &format!("Unsupported class file feature: {feature}"),
                )
                .await),
        }
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_array_class(_jvm, element_type_name).await
    }
}
