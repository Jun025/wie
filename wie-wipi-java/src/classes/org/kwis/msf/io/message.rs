use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.Message
pub struct Message;

impl Message {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/Message",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            // The two members 간호사타이쿤2 references; both descriptors match the authoritative
            // `org/kwis/msf/io/Message.class` in `docs/reference/AromaWIPI_classes.zip`. That class
            // has two further constructors and seventeen more accessors (getAddress, getLength,
            // getDate, ...) which no corpus title references, so they are not here.
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;[B)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getData", "()[B", Self::get_data, MethodAccessFlags::PUBLIC),
            ],
            // Field names are the real class's. `data` backs getData. `addr` has no reader in the
            // corpus and is here only because it is where the constructor's own argument belongs --
            // dropping it would silently discard the address this class exists to carry.
            fields: vec![
                JavaFieldProto::new("addr", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("data", "[B", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        addr: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msf.io.Message::<init>({this:?}, {addr:?}, {data:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "addr", "Ljava/lang/String;", addr).await?;
        jvm.put_field(&mut this, "data", "[B", data).await?;

        Ok(())
    }

    async fn get_data(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Array<i8>>> {
        tracing::debug!("org.kwis.msf.io.Message::getData({this:?})");

        jvm.get_field(&this, "data", "[B").await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::Message;

    // getData has to hand back the bytes the constructor was given -- a stub returning null is
    // what a caller would misread as an empty message.
    #[test]
    fn get_data_returns_the_payload_the_constructor_was_given() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let mut payload = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut payload, 0, [1i8, 2, 3]).await?;

            let addr = JavaLangString::from_rust_string(&jvm, "010").await?;
            let message: ClassInstanceRef<Message> = jvm
                .new_class("org/kwis/msf/io/Message", "(Ljava/lang/String;[B)V", (addr, payload))
                .await?
                .into();

            let data = jvm.invoke_virtual(&message, "org/kwis/msf/io/Message", "getData", "()[B", ()).await?;
            assert_eq!(jvm.load_array::<i8>(&data, 0, 3).await?, [1i8, 2, 3]);

            Ok(())
        })
    }
}
