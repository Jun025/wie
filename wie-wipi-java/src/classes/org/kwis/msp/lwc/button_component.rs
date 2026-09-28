use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use rustjava_runtime::classes::java::lang::{Object, String};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Image, lwc::ActionListener};

// class org.kwis.msp.lwc.ButtonComponent
//
// Parent, the two constructors, setActionListener and the field names are the ones
// `docs/reference/AromaWIPI_classes.zip` declares. 33f3e7669599 constructs one with
// (String, Image) and registers an ActionListener on it. Like the other lwc widgets here it is
// not drawn and not laid out (Component::paint · getWidth stay the no-op/0 answers), and a key
// press does not fire the listener — keyNotify is still Component's. The label, image and
// listener are only kept, so a later measured wall can use them without a new field.
pub struct ButtonComponent;

impl ButtonComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ButtonComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/Image;)V",
                    Self::init_with_string_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setActionListener",
                    "(Lorg/kwis/msp/lwc/ActionListener;Ljava/lang/Object;)V",
                    Self::set_action_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("str", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("img", "Lorg/kwis/msp/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("l", "Lorg/kwis/msp/lwc/ActionListener;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("o", "Ljava/lang/Object;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_string_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        image: ClassInstanceRef<Image>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::<init>({this:?}, {string:?}, {image:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "str", "Ljava/lang/String;", string).await?;
        jvm.put_field(&mut this, "img", "Lorg/kwis/msp/lcdui/Image;", image).await?;

        Ok(())
    }

    async fn set_action_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<ActionListener>,
        object: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ButtonComponent::setActionListener({this:?}, {listener:?}, {object:?})");

        jvm.put_field(&mut this, "l", "Lorg/kwis/msp/lwc/ActionListener;", listener).await?;
        jvm.put_field(&mut this, "o", "Ljava/lang/Object;", object).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::Object;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::{lcdui::Image, lwc::ActionListener},
        get_protos,
    };

    /// 33f3e7669599's wall: the class was missing, then its (String, Image) constructor. The
    /// listener is the one the guest registers; a null image is what a text-only button passes.
    #[test]
    fn button_component_constructs_with_string_and_image_and_keeps_its_listener() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let text = JavaLangString::from_rust_string(&jvm, "OK").await?;
            let image: ClassInstanceRef<Image> = None.into();
            let button = jvm
                .new_class(
                    "org/kwis/msp/lwc/ButtonComponent",
                    "(Ljava/lang/String;Lorg/kwis/msp/lcdui/Image;)V",
                    (text.clone(), image),
                )
                .await?;
            assert!(jvm.is_instance(&*button, "org/kwis/msp/lwc/Component"));

            let listener: ClassInstanceRef<ActionListener> = None.into();
            let tag: ClassInstanceRef<Object> = jvm.new_class("java/lang/Object", "()V", ()).await?.into();
            let _: () = jvm
                .invoke_virtual(
                    &button,
                    "org/kwis/msp/lwc/ButtonComponent",
                    "setActionListener",
                    "(Lorg/kwis/msp/lwc/ActionListener;Ljava/lang/Object;)V",
                    (listener, tag.clone()),
                )
                .await?;

            let kept: ClassInstanceRef<Object> = jvm.get_field(&button, "o", "Ljava/lang/Object;").await?;
            assert_eq!(kept.identity(), tag.identity());
            let label: ClassInstanceRef<Object> = jvm.get_field(&button, "str", "Ljava/lang/String;").await?;
            assert_eq!(label.identity(), text.identity());

            Ok(())
        })
    }
}
