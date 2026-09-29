use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::{Object, String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.io.Connector
// wie has no network, so every open fails the way a phone out of coverage does: an IOException
// the title's own handler already expects. Without the class the lookup itself is fatal.
pub struct Connector;

impl Connector {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/Connector",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;)Ljavax/microedition/io/Connection;",
                    Self::open,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;I)Ljavax/microedition/io/Connection;",
                    Self::open_mode,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;IZ)Ljavax/microedition/io/Connection;",
                    Self::open_mode_timeouts,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn open(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<Object>> {
        tracing::warn!("javax.microedition.io.Connector::open({name:?}) — no network");

        Err(jvm.exception("java/io/IOException", "no network").await)
    }

    async fn open_mode(jvm: &Jvm, context: &mut WieJvmContext, name: ClassInstanceRef<String>, _mode: i32) -> Result<ClassInstanceRef<Object>> {
        Self::open(jvm, context, name).await
    }

    async fn open_mode_timeouts(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        _mode: i32,
        _timeouts: bool,
    ) -> Result<ClassInstanceRef<Object>> {
        Self::open(jvm, context, name).await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::{Object, String};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    // The SKT title that found this wall opens a socket inside a key handler and catches IOException.
    #[test]
    fn open_throws_io_exception() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "socket://127.0.0.1:1").await?.into();
            let opened: JvmResult<ClassInstanceRef<Object>> = jvm
                .invoke_static(
                    "javax/microedition/io/Connector",
                    "open",
                    "(Ljava/lang/String;)Ljavax/microedition/io/Connection;",
                    (name,),
                )
                .await;
            let Err(JavaError::JavaException(exception)) = opened else {
                panic!("Connector.open succeeded without a network");
            };
            assert!(jvm.is_instance(&*exception, "java/io/IOException"));

            Ok(())
        })
    }
}
