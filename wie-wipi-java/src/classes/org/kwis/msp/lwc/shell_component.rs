use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    net::wie::ShellCard,
    org::kwis::msp::lcdui::{Card, Display},
    org::kwis::msp::lwc::Component,
};

// class org.kwis.msp.lwc.ShellComponent
pub struct ShellComponent;

impl ShellComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ShellComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(IIII)V", Self::init_with_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setWorkComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::set_work_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hide", "()V", Self::hide, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTitle", "(Ljava/lang/String;)V", Self::set_title_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTitle", "(Lorg/kwis/msp/lwc/Component;)V", Self::set_title, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTitle", "()Lorg/kwis/msp/lwc/Component;", Self::get_title, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("cmpTitle", "Lorg/kwis/msp/lwc/Component;", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ShellComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_size(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ShellComponent::<init>({this:?}, {x}, {y}, {width}, {height})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn set_work_component(
        _: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.ShellComponent::setWorkComponent({this:?}, {component:?})");

        Ok(())
    }

    // show()/hide() put the shell on and off the default Display through a net.wie.ShellCard, which
    // is what gives it a screen: without this a ShellComponent game's paint is never called.
    async fn show(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::show({this:?})");

        let component: ClassInstanceRef<Component> = this.clone().instance.into();
        if ShellCard::find(jvm, &component).await?.is_some() {
            return Ok(());
        }

        let card: ClassInstanceRef<Card> = jvm
            .new_class("net/wie/ShellCard", "(Lorg/kwis/msp/lwc/ShellComponent;)V", (this,))
            .await?
            .into();
        let display = Self::default_display(jvm).await?;
        jvm.invoke_virtual(
            &display,
            "org/kwis/msp/lcdui/Display",
            "pushCard",
            "(Lorg/kwis/msp/lcdui/Card;)V",
            (card,),
        )
        .await
    }

    async fn hide(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::hide({this:?})");

        let component: ClassInstanceRef<Component> = this.instance.into();
        let Some(card) = ShellCard::find(jvm, &component).await? else {
            return Ok(());
        };
        let display = Self::default_display(jvm).await?;
        let _: bool = jvm
            .invoke_virtual(
                &display,
                "org/kwis/msp/lcdui/Display",
                "removeCard",
                "(Lorg/kwis/msp/lcdui/Card;)Z",
                (card,),
            )
            .await?;

        Ok(())
    }

    // The javadoc's getTitle returns the title *component*, and the canonical ShellComponent refers
    // to LabelComponent, so a string title becomes a label held in cmpTitle (the canonical field).
    // Kept, not drawn: no lwc widget is painted by this layer. 33f3e7669599 calls setTitle(String)
    // after its 14th key.
    async fn set_title_string(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        title: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::setTitle({this:?}, {title:?})");

        let label: ClassInstanceRef<Component> = jvm
            .new_class("org/kwis/msp/lwc/LabelComponent", "(Ljava/lang/String;)V", (title,))
            .await?
            .into();

        Self::set_title(jvm, context, this, label).await
    }

    async fn set_title(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, title: ClassInstanceRef<Component>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::setTitle({this:?}, {title:?})");

        jvm.put_field(&mut this, "cmpTitle", "Lorg/kwis/msp/lwc/Component;", title).await
    }

    async fn get_title(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Component>> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::getTitle({this:?})");

        jvm.get_field(&this, "cmpTitle", "Lorg/kwis/msp/lwc/Component;").await
    }

    async fn default_display(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Display>> {
        jvm.invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::lwc::Component, get_protos};

    /// 33f3e7669599's wall after its 14th key: setTitle(String) on a ShellComponent. The title comes
    /// back from getTitle as a LabelComponent; setTitle(Component) replaces it with that component.
    #[test]
    fn set_title_string_keeps_a_label_and_set_title_component_replaces_it() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
            let none: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &shell,
                    "org/kwis/msp/lwc/ShellComponent",
                    "getTitle",
                    "()Lorg/kwis/msp/lwc/Component;",
                    (),
                )
                .await?;
            assert!(none.is_null());

            let text = JavaLangString::from_rust_string(&jvm, "title").await?;
            let _: () = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "setTitle", "(Ljava/lang/String;)V", (text,))
                .await?;
            let title: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &shell,
                    "org/kwis/msp/lwc/ShellComponent",
                    "getTitle",
                    "()Lorg/kwis/msp/lwc/Component;",
                    (),
                )
                .await?;
            assert!(!title.is_null());
            assert!(jvm.is_instance(&**title, "org/kwis/msp/lwc/LabelComponent"));

            let other = jvm.new_class("org/kwis/msp/lwc/LabelComponent", "()V", ()).await?;
            let _: () = jvm
                .invoke_virtual(
                    &shell,
                    "org/kwis/msp/lwc/ShellComponent",
                    "setTitle",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    (other.clone(),),
                )
                .await?;
            let title: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &shell,
                    "org/kwis/msp/lwc/ShellComponent",
                    "getTitle",
                    "()Lorg/kwis/msp/lwc/Component;",
                    (),
                )
                .await?;
            assert_eq!(title.identity(), other.identity());

            Ok(())
        })
    }
}
