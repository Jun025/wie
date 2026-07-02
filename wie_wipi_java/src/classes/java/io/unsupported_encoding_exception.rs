use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class java.io.UnsupportedEncodingException — not in bundled java_runtime; registered so KTF init preload resolves it.
pub struct UnsupportedEncodingException;

impl UnsupportedEncodingException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "java/io/UnsupportedEncodingException",
            parent_class: Some("java/io/IOException"),
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
        tracing::debug!("java.io.UnsupportedEncodingException::<init>({this:?})");
        let _: () = jvm.invoke_special(&this, "java/io/IOException", "<init>", "()V", ()).await?;
        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("java.io.UnsupportedEncodingException::<init>({this:?}, {message:?})");
        let _: () = jvm
            .invoke_special(&this, "java/io/IOException", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;
        Ok(())
    }
}
