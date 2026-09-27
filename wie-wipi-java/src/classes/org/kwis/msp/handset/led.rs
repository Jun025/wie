use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.LED
pub struct LED;

impl LED {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/LED",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "set",
                "(I)V",
                Self::set,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // There is no LED on a browser or desktop host, so the state is dropped like BackLight's.
    async fn set(_: &Jvm, _: &mut WieJvmContext, state: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.LED::set({state})");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    /// Two KTF titles loaded this class at boot and died on `NoClassDefFoundError`; the first call
    /// either makes is `LED.set(int)` (measured 2026-09-27, `Method set(I)V not found`).
    #[test]
    fn led_set_is_callable() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let _: () = jvm.invoke_static("org/kwis/msp/handset/LED", "set", "(I)V", (0,)).await?;

            Ok(())
        })
    }
}
