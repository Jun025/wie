use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class java.lang.VirtualMachineError
// Not provided by the bundled java_runtime; KTF `MExe_init` preloads it (and its
// OutOfMemoryError subclass) via load_java_class during init, and aborts (returns
// 0xffffffff) if the class can't be resolved. Register it as a plain Error subclass.
pub struct VirtualMachineError;

impl VirtualMachineError {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "java/lang/VirtualMachineError",
            parent_class: Some("java/lang/Error"),
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
        tracing::debug!("java.lang.VirtualMachineError::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Error", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("java.lang.VirtualMachineError::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/Error", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
