use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class java.io.UnavailableException — a WIPI class (AromaWIPI `java/io/UnavailableException.class`:
// public, extends RuntimeException, `<init>()V` and `<init>(String)V`) the RustJava runtime does not carry.
// KTF titles that reference it abort class-catalog init with NoClassDefFoundError without it.
pub struct UnavailableException;

impl UnavailableException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "java/io/UnavailableException",
            parent_class: Some("java/lang/RuntimeException"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("java.io.UnavailableException::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/RuntimeException", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("java.io.UnavailableException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/RuntimeException", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::runtime::JavaLangString;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_unavailable_exception_is_a_runtime_exception() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let message = JavaLangString::from_rust_string(&jvm, "unavailable").await?;
            let instance = jvm.new_class("java/io/UnavailableException", "(Ljava/lang/String;)V", (message,)).await?;
            assert!(jvm.is_instance(&*instance, "java/lang/RuntimeException"));

            Ok(())
        })
    }
}
