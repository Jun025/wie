use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.ChoiceText -- KTF's own form toolkit; no API document is in this repo, so the
// shape comes only from what 0c67145b11df (KTF AOT) asks of it on its name-entry form:
//
// - `<init>([Ljava/lang/String;)V` is the only method it resolves (`get_java_method`).
// - The instance goes straight into `GFormComponent.addComponent(Lorg/kwis/msp/lwc/Component;IIII)I`,
//   so it is an lwc Component. Which Component subclass the real one extends is not known.
//
// The choices are not kept: no getter is asked for, and lwc widgets are not drawn by this layer
// (Component::paint is a no-op). A title that reads the selection back needs more than this.
pub struct ChoiceText;

impl ChoiceText {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/ChoiceText",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "<init>",
                "([Ljava/lang/String;)V",
                Self::init,
                MethodAccessFlags::PUBLIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, choices: ClassInstanceRef<Array<String>>) -> JvmResult<()> {
        tracing::warn!("stub com.ktf.kfc.ChoiceText::<init>({this:?}, {choices:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::lwc::Component, get_protos};

    // 0c67145b11df's form, in its order: new ChoiceText(String[]), new GFormComponent(), then
    // addComponent(choice, x, y, w, h). Dropping either class from get_protos() or making
    // ChoiceText not a Component turns this red.
    #[test]
    fn choice_text_goes_into_a_g_form_component() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let mut choices = jvm.instantiate_array("Ljava/lang/String;", 2).await?;
            let a = JavaLangString::from_rust_string(&jvm, "a").await?;
            let b = JavaLangString::from_rust_string(&jvm, "b").await?;
            jvm.store_array(&mut choices, 0, [a, b]).await?;

            let choice: ClassInstanceRef<Component> = jvm
                .new_class("com/ktf/kfc/ChoiceText", "([Ljava/lang/String;)V", (choices,))
                .await?
                .into();
            assert!(jvm.is_instance(&**choice, "org/kwis/msp/lwc/Component"));
            let form = jvm.new_class("com/ktf/kfc/GFormComponent", "()V", ()).await?;
            let index: i32 = jvm
                .invoke_virtual(
                    &form,
                    "com/ktf/kfc/GFormComponent",
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;IIII)I",
                    (choice.clone(), 120, 183, 36, 15),
                )
                .await?;
            assert_eq!(index, 0);

            let back: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &form,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    (0,),
                )
                .await?;
            assert_eq!(back.identity(), choice.identity());

            Ok(())
        })
    }
}
