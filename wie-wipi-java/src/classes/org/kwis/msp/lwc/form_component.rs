use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lwc::Component;

// class org.kwis.msp.lwc.FormComponent
pub struct FormComponent;

impl FormComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/FormComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Z)V", Self::init_with_vertical, Default::default()),
                JavaMethodProto::new(
                    "setFocus",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::set_focus,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("vertical", "Z", FieldAccessFlags::PRIVATE)],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<FormComponent>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.FormComponent::<init>({this:?})");
        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await?;
        Ok(())
    }

    // Kept, not laid out: this layer does no form layout, so bVertical only sits in a field.
    // ca7fa8ade8ad calls it off the boot thread. The javadoc gives `FormComponent()` no default direction,
    // so `()V` leaves the field at the JVM default rather than inventing one.
    async fn init_with_vertical(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<FormComponent>, vertical: bool) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::<init>({this:?}, {vertical})");
        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await?;
        jvm.put_field(&mut this, "vertical", "Z", vertical).await?;
        Ok(())
    }

    // The javadoc gives this one no text; the class keeps a cmpFocus, so it is read as "give c the
    // focus" — c's own setFocus(), dispatched virtually so a guest override still runs, which is
    // where the shell's key routing and c's EventListener look. 0c67145b11df's name form calls it
    // from its OK button with the box left empty. A null c is ignored rather than clearing the focus.
    async fn set_focus(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<FormComponent>, c: ClassInstanceRef<Component>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.FormComponent::setFocus({this:?}, {c:?})");

        if c.is_null() {
            return Ok(());
        }
        jvm.invoke_virtual(&c, "org/kwis/msp/lwc/Component", "setFocus", "()V", ()).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    /// ca7fa8ade8ad's wall after setForeground: `FormComponent.<init>(Z)V`, called off the boot thread.
    #[test]
    fn form_component_boolean_constructor_keeps_vertical() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            for vertical in [true, false] {
                let form = jvm.new_class("org/kwis/msp/lwc/FormComponent", "(Z)V", (vertical,)).await?;
                let kept: bool = jvm.get_field(&form, "vertical", "Z").await?;
                assert_eq!(kept, vertical);
                let bg: i32 = jvm
                    .invoke_virtual(&form, "org/kwis/msp/lwc/Component", "getBackground", "()I", ())
                    .await?;
                assert_eq!(bg, -1);
            }

            Ok(())
        })
    }
}
