use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.GMenubarForm -- KTF's own form toolkit, like ChoiceText; no API document is in
// this repo. bfa8ec352451 (KTF AOT) fails its first class load on it at boot.
pub struct GMenubarForm;

impl GMenubarForm {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GMenubarForm",
            parent_class: Some("org/kwis/msp/lwc/FormComponent"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC)],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GMenubarForm::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/FormComponent", "<init>", "()V", ()).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    // bfa8ec352451's boot loads an app class that extends this one; dropping it from get_protos()
    // is that title's boot panic again, and it has to stay an lwc form to be shown like one.
    #[test]
    fn g_menubar_form_is_an_lwc_form() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let form = jvm.new_class("com/ktf/kfc/GMenubarForm", "()V", ()).await?;
            assert!(jvm.is_instance(&*form, "org/kwis/msp/lwc/FormComponent"));

            Ok(())
        })
    }
}
