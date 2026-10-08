use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.GForm -- the same undocumented toolkit as GMenubarForm. f07cbc782828 builds one
// with `<init>(IIII)V` (124, 214, 65, 17 in its startApp) and hands it to
// `GTextField.<init>(Lcom/ktf/kfc/GMenubarForm;…)`, which is the only evidence of its parent.
// ponytail: the box is not kept — nothing reads it back, and the title draws its own screens.
pub struct GForm;

impl GForm {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GForm",
            parent_class: Some("com/ktf/kfc/GMenubarForm"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "(IIII)V", Self::init, MethodAccessFlags::PUBLIC)],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GForm::<init>({this:?}, {x}, {y}, {width}, {height})");

        jvm.invoke_special(&this, "com/ktf/kfc/GMenubarForm", "<init>", "()V", ()).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    // f07cbc782828 passes a GForm where GTextField asks for a GMenubarForm; without the class its
    // startApp throws on the first load and the title never paints.
    #[test]
    fn g_form_is_a_menubar_form() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let form = jvm.new_class("com/ktf/kfc/GForm", "(IIII)V", (124, 214, 65, 17)).await?;
            assert!(jvm.is_instance(&*form, "com/ktf/kfc/GMenubarForm"));

            Ok(())
        })
    }
}
