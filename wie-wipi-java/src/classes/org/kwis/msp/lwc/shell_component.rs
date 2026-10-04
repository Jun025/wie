use alloc::{boxed::Box, vec, vec::Vec};

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    net::wie::{ShellCard, WIPIKeyCode},
    org::kwis::msp::lcdui::{Card, Display, Graphics},
    org::kwis::msp::lwc::{Component, KEY_NOTIFY},
};

// Component.keyNotify's type values (javadoc: KEY_PRESSED, KEY_RELEASED, KEY_REPEATED) as
// net.wie.CardCanvas sends them.
const KEY_PRESSED: i32 = 1;

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
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hide", "()V", Self::hide, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTitle", "(Ljava/lang/String;)V", Self::set_title_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTitle", "(Lorg/kwis/msp/lwc/Component;)V", Self::set_title, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTitle", "()Lorg/kwis/msp/lwc/Component;", Self::get_title, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
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
        let _: () = jvm
            .invoke_virtual(
                &display,
                "org/kwis/msp/lcdui/Display",
                "pushCard",
                "(Lorg/kwis/msp/lcdui/Card;)V",
                (card,),
            )
            .await?;

        // A shell shown with none of its leaves focused gives the focus to the first one that takes
        // input (labels do not), the traversal order keyNotify's UP/DOWN already walks. Without it
        // keys reach no widget: 65ef7052f528's ID entry shows a text field it never calls setFocus on,
        // and waits for its EventListener to hear OK. A title that sets the focus itself keeps it.
        let mut leaves = Vec::new();
        Self::collect_leaves(jvm, component, &mut leaves).await?;
        let focus = ShellCard::focus(jvm).await?;
        if !focus.is_null() && leaves.iter().any(|leaf| leaf.identity() == focus.identity()) {
            return Ok(());
        }
        let Some(first) = leaves
            .into_iter()
            .find(|leaf| !jvm.is_instance(&***leaf, "org/kwis/msp/lwc/LabelComponent"))
        else {
            return Ok(());
        };
        jvm.invoke_virtual(&first, "org/kwis/msp/lwc/Component", "setFocus", "()V", ()).await
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

    // The javadoc: a key goes to the component that has the focus (setFocus). This layer lays nothing
    // out, so "next" is the next leaf in add order: UP/DOWN presses move the focus there, every other
    // key goes to the focused leaf. 0c67145b11df's name form is two text boxes, a ChoiceText and a
    // button, focus on the first box. A shell whose leaves do not hold the focus does nothing, as before.
    // Always true, as before: the return value is what net.wie.CardCanvas reads to pass the key on.
    // ponytail: UP/DOWN are never offered to the leaf first — no measured leaf uses them.
    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::keyNotify({this:?}, {type}, {key})");

        let mut leaves = Vec::new();
        Self::collect_leaves(jvm, this.clone().instance.into(), &mut leaves).await?;
        if leaves.is_empty() {
            return Ok(true);
        }
        let focus = ShellCard::focus(jvm).await?;
        let Some(at) = leaves.iter().position(|leaf| !focus.is_null() && leaf.identity() == focus.identity()) else {
            return Ok(true);
        };

        // The focused leaf's EventListener sees the key first (Component.setEventListener); a key it
        // takes neither moves the focus nor reaches the leaf.
        if Component::notify_listener(jvm, &leaves[at], KEY_NOTIFY, r#type, key, 0).await? {
            return Ok(true);
        }

        let step = match key {
            x if x == WIPIKeyCode::UP as i32 => -1,
            x if x == WIPIKeyCode::DOWN as i32 => 1,
            _ => 0,
        };
        if step != 0 {
            if r#type == KEY_PRESSED {
                let next = (at as i32 + step).rem_euclid(leaves.len() as i32) as usize;
                let _: () = jvm
                    .invoke_virtual(&leaves[next], "org/kwis/msp/lwc/Component", "setFocus", "()V", ())
                    .await?;
            }
        } else {
            let _: bool = jvm
                .invoke_virtual(&leaves[at], "org/kwis/msp/lwc/Component", "keyNotify", "(II)Z", (r#type, key))
                .await?;
        }
        // The press may have moved the focus or changed a widget, so the form is drawn again — the
        // whole display, since the title's own card under this one paints the form's background. A
        // press that took the shell off the display finds nothing to repaint.
        if r#type == KEY_PRESSED {
            let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "repaint", "()V", ()).await?;
        }

        Ok(true)
    }

    // A shell that does not draw itself (the guest did not override paint) lets its children draw:
    // ShellCard calls this. Every lwc child but com.ktf.kfc.GFormComponent still draws nothing.
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::paint({this:?}, {g:?})");

        let count: i32 = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/ContainerComponent", "getNumberOfComponent", "()I", ())
            .await?;
        for i in 0..count {
            let child: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &this,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    (i,),
                )
                .await?;
            if !child.is_null() {
                let _: () = jvm
                    .invoke_virtual(
                        &child,
                        "org/kwis/msp/lwc/Component",
                        "paint",
                        "(Lorg/kwis/msp/lcdui/Graphics;)V",
                        (g.clone(),),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn collect_leaves(jvm: &Jvm, component: ClassInstanceRef<Component>, leaves: &mut Vec<ClassInstanceRef<Component>>) -> JvmResult<()> {
        let count: i32 = jvm
            .invoke_virtual(&component, "org/kwis/msp/lwc/ContainerComponent", "getNumberOfComponent", "()I", ())
            .await?;
        for i in 0..count {
            let child: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &component,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    (i,),
                )
                .await?;
            if child.is_null() {
                continue;
            }
            if jvm.is_instance(&**child, "org/kwis/msp/lwc/ContainerComponent") {
                Box::pin(Self::collect_leaves(jvm, child, leaves)).await?;
            } else {
                leaves.push(child);
            }
        }

        Ok(())
    }

    // A shell is the screen it is shown on, so it answers the display's size where Component's stub
    // answers 0. be08d047cbae reads its shell's size once and repaints (0, 0, width, height) after
    // every key; at 0 × 0 that repaint covered nothing and the screen never moved off the first
    // frame. ponytail: a shell built with (IIII) also answers the display size — no field keeps
    // its own, because a field here would shift an LGT AOT subclass's field offsets.
    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::getWidth({this:?})");

        let display = Self::default_display(jvm).await?;
        if display.is_null() {
            return Ok(0);
        }
        jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getWidth", "()I", ()).await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ShellComponent::getHeight({this:?})");

        let display = Self::default_display(jvm).await?;
        if display.is_null() {
            return Ok(0);
        }
        jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", ()).await
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
