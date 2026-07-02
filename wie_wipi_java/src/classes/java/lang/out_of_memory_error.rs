use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class java.lang.OutOfMemoryError
// Not provided by the bundled java_runtime; KTF `MExe_init` preloads it via
// load_java_class during init and aborts if unresolved. Register it as the spec
// VirtualMachineError subclass.
pub struct OutOfMemoryError;

impl OutOfMemoryError {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "java/lang/OutOfMemoryError",
            parent_class: Some("java/lang/VirtualMachineError"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("java.lang.OutOfMemoryError::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/VirtualMachineError", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("java.lang.OutOfMemoryError::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/VirtualMachineError", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
