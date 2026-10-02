use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::net::wie::WIPIKeyCode;

// class com.ktf.kfc.ChoiceText -- KTF's own form toolkit; no API document is in this repo, so the
// shape comes only from what 0c67145b11df (KTF AOT) asks of it on its name-entry form:
//
// - `<init>([Ljava/lang/String;)V` is the only method it resolves (`get_java_method`).
// - The instance goes straight into `GFormComponent.addComponent(Lorg/kwis/msp/lwc/Component;IIII)I`,
//   so it is an lwc Component. Which Component subclass the real one extends is not known.
//
// - Its button listener reads `getSelectedIndex()I` back, so the choices and the selection are kept.
//   LEFT/RIGHT presses move the selection (wrapping) once the shell hands this widget the focus; which
//   keys the real one uses is not documented — those two are the ones UP/DOWN focus moves leave free.
//   Nothing is drawn (Component::paint is a no-op).
pub struct ChoiceText;

impl ChoiceText {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/ChoiceText",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "([Ljava/lang/String;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getSelectedIndex", "()I", Self::get_selected_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("choices", "[Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("selected", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, choices: ClassInstanceRef<Array<String>>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.ChoiceText::<init>({this:?}, {choices:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "choices", "[Ljava/lang/String;", choices).await
    }

    async fn get_selected_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.ktf.kfc.ChoiceText::getSelectedIndex({this:?})");

        jvm.get_field(&this, "selected", "I").await
    }

    // 1 = KEY_PRESSED as net.wie.CardCanvas sends it. Every key answers true, as Component's stub did.
    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.ktf.kfc.ChoiceText::keyNotify({this:?}, {type}, {key})");

        let step = match key {
            x if x == WIPIKeyCode::LEFT as i32 => -1,
            x if x == WIPIKeyCode::RIGHT as i32 => 1,
            _ => return Ok(true),
        };
        let choices: ClassInstanceRef<Array<String>> = jvm.get_field(&this, "choices", "[Ljava/lang/String;").await?;
        if r#type != 1 || choices.is_null() {
            return Ok(true);
        }
        let length = jvm.array_length(&choices).await? as i32;
        if length == 0 {
            return Ok(true);
        }
        let selected: i32 = jvm.get_field(&this, "selected", "I").await?;
        jvm.put_field(&mut this, "selected", "I", (selected + step).rem_euclid(length)).await?;

        Ok(true)
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
