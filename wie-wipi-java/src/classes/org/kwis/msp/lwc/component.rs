use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{net::wie::ShellCard, org::kwis::msp::lcdui::Graphics};

// class org.kwis.msp.lwc.Component
pub struct Component;

impl Component {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/Component",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("focusNotify", "(Z)V", Self::focus_notify, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("configure", "(IIIII)V", Self::configure, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFocus", "()V", Self::set_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getX", "()I", Self::get_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getY", "()I", Self::get_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("validate", "()V", Self::layout, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "(IIII)V", Self::repaint_region, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("serviceRepaints", "()V", Self::service_repaints, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasFocus", "()Z", Self::has_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setBackground", "(I)V", Self::set_background, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getBackground", "()I", Self::get_background, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new("bg", "I", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.Component::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        // -1 is the javadoc's own "no background" value (setBackground: 「지정을 해제 할경우 -1값」).
        // The canonical per-widget default is not in the API-only class files, so none is invented.
        jvm.put_field(&mut this, "bg", "I", -1).await?;

        Ok(())
    }

    // Kept, not drawn: this layer paints no lwc widget (Component::paint is a no-op), so the colour
    // only comes back from getBackground. ca7fa8ade8ad calls setBackground(I) at boot.
    async fn set_background(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, bg: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setBackground({this:?}, {bg:#x})");

        jvm.put_field(&mut this, "bg", "I", bg).await
    }

    async fn get_background(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getBackground({this:?})");

        jvm.get_field(&this, "bg", "I").await
    }

    async fn key_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, chr: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::keyNotify({this:?}, {type:?}, {chr:?})");

        Ok(true)
    }

    async fn focus_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::focusNotify({this:?}, {focus:?})");

        Ok(())
    }

    async fn show_notify(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::showNotify({this:?}, {show:?})");

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn configure(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32, mask: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::configure({this:?}, {x}, {y}, {w}, {h}, {mask})");

        Ok(())
    }

    async fn set_focus(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::setFocus({this:?})");

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getHeight({this:?})");

        Ok(0)
    }

    // Paired with getHeight, which has returned 0 since this layer landed: the lwc widgets are
    // never laid out here, so there is no measured width to report and inventing one would be a
    // guess a game could lay out against.
    async fn get_width(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getWidth({this:?})");

        Ok(0)
    }

    // The position half of the same answer: nothing is laid out, so every component sits at the
    // origin. ae749cc5a777 calls ShellComponent.getX() at boot.
    async fn get_x(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getX({this:?})");

        Ok(0)
    }

    async fn get_y(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::getY({this:?})");

        Ok(0)
    }

    // No-op for the same reason: this layer never lays children out. a10a1f02b41b calls layout() on an
    // AnnunciatorComponent at boot and validate() — the same request, also bound here — on its first key.
    async fn layout(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::layout/validate({this:?})");

        Ok(())
    }

    // A shown ShellComponent sits on a net.wie.ShellCard, so its repaint is that card's repaint —
    // the same Card → CardCanvas → request_redraw path Card games take. A component that is not a
    // shown shell has nowhere to be drawn (children are never laid out here), so it stays a no-op.
    async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::repaint({this:?})");

        match ShellCard::find(jvm, &this).await? {
            Some(card) => jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "()V", ()).await,
            None => Ok(()),
        }
    }

    // The region-bounded repaint 학교가는길 calls every timer tick.
    async fn repaint_region(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::repaint({this:?}, {x}, {y}, {width}, {height})");

        match ShellCard::find(jvm, &this).await? {
            Some(card) => {
                jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "repaint", "(IIII)V", (x, y, width, height))
                    .await
            }
            None => Ok(()),
        }
    }

    async fn service_repaints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::serviceRepaints({this:?})");

        match ShellCard::find(jvm, &this).await? {
            Some(card) => jvm.invoke_virtual(&card, "org/kwis/msp/lcdui/Card", "serviceRepaints", "()V", ()).await,
            None => Ok(()),
        }
    }

    // What ShellCard calls when the guest shell does not override paint (a shell that only hosts a
    // work component): lwc's own widgets are not drawn here, so there is nothing to put on screen.
    async fn paint(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::paint({this:?}, {g:?})");

        Ok(())
    }

    // Reported as false rather than tracked: setFocus and focusNotify are both stubs that keep
    // no state, so a stored flag would claim a focus this layer never actually grants. Games
    // that see false keep handling keys themselves, which is what the other lwc stubs assume.
    async fn has_focus(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lwc.Component::hasFocus({this:?})");

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::runtime::JavaLangString;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    /// Three census walls, each a missing lwc method a KTF title called at boot or on its first key:
    /// `LabelComponent.<init>(String)` (33f3e7669599 · ca7fa8ade8ad), `ShellComponent.getX()` (ae749cc5a777)
    /// and `AnnunciatorComponent.layout()`/`validate()` (a10a1f02b41b). Each is called the way the guest does — on
    /// the subclass — so a method that only resolves on some other class still fails here.
    #[test]
    fn lwc_label_with_text_get_x_y_and_layout_resolve_on_the_classes_games_call() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let label = JavaLangString::from_rust_string(&jvm, "label").await?;
            let _ = jvm
                .new_class("org/kwis/msp/lwc/LabelComponent", "(Ljava/lang/String;)V", (label,))
                .await?;

            let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
            let x: i32 = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "getX", "()I", ()).await?;
            let y: i32 = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "getY", "()I", ()).await?;
            assert_eq!((x, y), (0, 0));

            let annunciator = jvm.new_class("org/kwis/msp/lwc/AnnunciatorComponent", "(Z)V", (true,)).await?;
            for method in ["layout", "validate"] {
                let _: () = jvm
                    .invoke_virtual(&annunciator, "org/kwis/msp/lwc/AnnunciatorComponent", method, "()V", ())
                    .await?;
            }

            Ok(())
        })
    }

    /// ca7fa8ade8ad's boot wall: setBackground(I) on a Component. Called on a ShellComponent — whose
    /// constructor skips ContainerComponent.<init> — so the -1 default must come from Component.<init>.
    #[test]
    fn set_background_is_kept_and_get_background_starts_unset() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
            let bg: i32 = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "getBackground", "()I", ())
                .await?;
            assert_eq!(bg, -1);

            let _: () = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "setBackground", "(I)V", (0x00123456,))
                .await?;
            let bg: i32 = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "getBackground", "()I", ())
                .await?;
            assert_eq!(bg, 0x00123456);

            Ok(())
        })
    }
}
