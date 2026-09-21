use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lwc::Component;

// The three constants doModal's return value is drawn from. Values read out of the
// ConstantValue attributes of `org/kwis/msp/lwc/DialogComponent.class` in
// `docs/reference/AromaWIPI_classes.zip` -- not guessed, and not the same numbers as the
// TYPE_* they pair with (TYPE_NONE/OK/OK_CANCEL are 0/1/2, DLG_TIMEOUT/OK/CANCEL are 10/11/12).
const TYPE_NONE: i32 = 0;
const DLG_TIMEOUT: i32 = 10;
const DLG_OK: i32 = 11;

// class org.kwis.msp.lwc.DialogComponent
pub struct DialogComponent;

impl DialogComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/DialogComponent",
            // ShellComponent, per the AromaWIPI javadoc's hierarchy
            // (Object -> Component -> ContainerComponent -> ShellComponent -> DialogComponent).
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                // The three members 붕어빵타이쿤3 / 당신은골프왕 / 슈퍼액션히어로 actually reference.
                // Their `binary.mod` pools carry the first and third in all three titles, and
                // setButtonString in the latter two; every descriptor below matches the
                // authoritative class. The other six public methods of the real class
                // (setType, setTimeout, getTimeout, getActionState, show, layout) and both other
                // constructors are referenced by no title in the corpus, so they are not here.
                JavaMethodProto::new(
                    "<init>",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setButtonString",
                    "(ILjava/lang/String;)V",
                    Self::set_button_string,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("doModal", "()I", Self::do_modal, MethodAccessFlags::PUBLIC),
            ],
            // Backs doModal's return value; the real class has this field under this name.
            fields: vec![JavaFieldProto::new("type", "I", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        title: ClassInstanceRef<String>,
        dialog_type: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.DialogComponent::<init>({this:?}, {component:?}, {title:?}, {dialog_type})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "type", "I", dialog_type).await?;

        Ok(())
    }

    async fn set_button_string(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        button_type: i32,
        button_string: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.DialogComponent::setButtonString({this:?}, {button_type}, {button_string:?})");

        Ok(())
    }

    // Nothing here draws the dialog or waits for a key, so doModal returns the outcome the
    // real one would have produced had the user done nothing. For TYPE_NONE that is exact --
    // it has no buttons and can only ever end in DLG_TIMEOUT. For the two button types it is a
    // choice: DLG_OK stands for the affirmative button, which is what lets a guest blocked on a
    // confirmation carry on. A guest that reads this as "the user pressed OK" when the user
    // pressed nothing is the known cost, and it is preferred to DLG_CANCEL only because
    // cancelling is what stops a boot-time dialog from proceeding.
    async fn do_modal(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let dialog_type: i32 = jvm.get_field(&this, "type", "I").await?;

        tracing::warn!("stub org.kwis.msp.lwc.DialogComponent::doModal({this:?}) type={dialog_type}");

        Ok(if dialog_type == TYPE_NONE { DLG_TIMEOUT } else { DLG_OK })
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::{DLG_OK, DLG_TIMEOUT, DialogComponent};

    async fn new_dialog(jvm: &Jvm, dialog_type: i32) -> JvmResult<ClassInstanceRef<DialogComponent>> {
        let title = JavaLangString::from_rust_string(jvm, "title").await?;
        let component: ClassInstanceRef<crate::classes::org::kwis::msp::lwc::Component> = None.into();

        Ok(jvm
            .new_class(
                "org/kwis/msp/lwc/DialogComponent",
                "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
                (component, title, dialog_type),
            )
            .await?
            .into())
    }

    async fn do_modal_for_type(jvm: &Jvm, dialog_type: i32) -> JvmResult<i32> {
        let dialog = new_dialog(jvm, dialog_type).await?;

        jvm.invoke_virtual(&dialog, "org/kwis/msp/lwc/DialogComponent", "doModal", "()I", ())
            .await
    }

    // TYPE_NONE has no buttons, so DLG_TIMEOUT is its only real outcome; the other two types
    // must not collapse onto it.
    #[test]
    fn do_modal_returns_timeout_only_for_the_button_less_type() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            assert_eq!(do_modal_for_type(&jvm, 0).await?, DLG_TIMEOUT);
            assert_eq!(do_modal_for_type(&jvm, 1).await?, DLG_OK);
            assert_eq!(do_modal_for_type(&jvm, 2).await?, DLG_OK);

            Ok(())
        })
    }

    // The constructor is reached through ShellComponent, so a broken parent chain shows up as a
    // failure to treat the dialog as the Component the guest passes it around as.
    #[test]
    fn dialog_component_is_a_component() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let dialog = new_dialog(&jvm, 1).await?;

            assert!(jvm.is_instance(dialog.as_ref(), "org/kwis/msp/lwc/ShellComponent"));
            assert!(jvm.is_instance(dialog.as_ref(), "org/kwis/msp/lwc/Component"));

            Ok(())
        })
    }
}
