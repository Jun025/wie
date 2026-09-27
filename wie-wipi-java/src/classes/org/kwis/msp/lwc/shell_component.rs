use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
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
            ],
            fields: vec![],
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

    async fn default_display(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Display>> {
        jvm.invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await
    }
}
