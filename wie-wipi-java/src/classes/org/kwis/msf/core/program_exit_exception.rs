use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.core.ProgramExitException
//
// KTF's AOT runtime names this class in its table of exceptions it throws itself, next to
// NullPointer/Arithmetic/ClassCast; all three titles of the 2026-09-27 census wall carry it
// (33f3e7669599 · ca7fa8ade8ad · a10a1f02b41b), and a10a1f02b41b once stopped on it not resolving.
// Parent and constructors are the ones `docs/reference/AromaWIPI_classes.zip` declares.
pub struct ProgramExitException;

impl ProgramExitException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/core/ProgramExitException",
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
        tracing::debug!("org.kwis.msf.core.ProgramExitException::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/RuntimeException", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("org.kwis.msf.core.ProgramExitException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/RuntimeException", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    /// The class a10a1f02b41b's runtime once failed to resolve — and a RuntimeException, so a
    /// guest's `catch (RuntimeException e)` catches it the way the reference hierarchy says.
    #[test]
    fn program_exit_exception_resolves_as_a_runtime_exception() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let exception = jvm.new_class("org/kwis/msf/core/ProgramExitException", "()V", ()).await?;
            assert!(jvm.is_instance(&*exception, "java/lang/RuntimeException"));

            Ok(())
        })
    }
}
