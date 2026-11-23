use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{
    io::{DataInputStream, DataOutputStream, InputStream, OutputStream},
    lang::String,
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::io::Connection;

// class javax.microedition.io.Connector
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
                    Self::open_with_mode,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;IZ)Ljavax/microedition/io/Connection;",
                    Self::open_with_mode_and_timeouts,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openInputStream",
                    "(Ljava/lang/String;)Ljava/io/InputStream;",
                    Self::open_input_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataInputStream",
                    "(Ljava/lang/String;)Ljava/io/DataInputStream;",
                    Self::open_data_input_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openOutputStream",
                    "(Ljava/lang/String;)Ljava/io/OutputStream;",
                    Self::open_output_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataOutputStream",
                    "(Ljava/lang/String;)Ljava/io/DataOutputStream;",
                    Self::open_data_output_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // No protocol is supported yet. fork: the three `open` overloads fail the way a phone out of coverage does — a
    // plain IOException, which is what the support list was measured with — where upstream throws
    // ConnectionNotFoundException (as the stream helpers below still do).
    async fn open(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?})");

        Err(jvm.exception("java/io/IOException", "no network").await)
    }

    async fn open_with_mode(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, mode: i32) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?}, {mode})");

        Err(jvm.exception("java/io/IOException", "no network").await)
    }

    async fn open_with_mode_and_timeouts(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        mode: i32,
        timeouts: bool,
    ) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?}, {mode}, {timeouts})");

        Err(jvm.exception("java/io/IOException", "no network").await)
    }

    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<InputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openInputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_data_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<DataInputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openDataInputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<OutputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openOutputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_data_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<DataOutputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openDataOutputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
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
