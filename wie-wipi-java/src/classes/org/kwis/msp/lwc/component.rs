use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use rustjava_runtime::classes::java::{lang::Object, util::Vector};

use crate::classes::{
    net::wie::ShellCard,
    org::kwis::msp::{lcdui::Graphics, lwc::EventListener},
};

// The javadoc's EventListener.eventNotify types (FOCUS_NOTIFY 1 · KEY_NOTIFY 3).
const FOCUS_NOTIFY: i32 = 1;
pub const KEY_NOTIFY: i32 = 3;

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
                JavaMethodProto::new("isShown", "()Z", Self::is_shown, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setBackground", "(I)V", Self::set_background, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getBackground", "()I", Self::get_background, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setForeground", "(I)V", Self::set_foreground, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getForeground", "()I", Self::get_foreground, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setEventListener",
                    "(Lorg/kwis/msp/lwc/EventListener;Ljava/lang/Object;)V",
                    Self::set_event_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("bg", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("fg", "I", FieldAccessFlags::PRIVATE),
                // Protected in the javadoc, and AOT subclasses read them directly: d448aee68157's
                // text box died on `Field xI not found from org/kwis/msp/lwc/Component` right after
                // its first name entry. Left at 0, the same origin and size getX/getY/getWidth/
                // getHeight report, since nothing is laid out here.
                JavaFieldProto::new("x", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("y", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("w", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("h", "I", FieldAccessFlags::PROTECTED),
                // setEventListener's registrations, as flat (component, listener, obj) triples. Static,
                // not the canonical evtListener/evtListenerObj instance fields: an lwc instance field
                // shifts an LGT AOT subclass's offsets (AnnunciatorComponent.shownHeight, net.wie.ShellCard).
                JavaFieldProto::new(
                    "evtListeners",
                    "Ljava/util/Vector;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.Component::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        // -1 is the javadoc's own "no background" value (setBackground: 「지정을 해제 할경우 -1값」).
        // The canonical per-widget default is not in the API-only class files, so none is invented.
        jvm.put_field(&mut this, "bg", "I", -1).await?;
        // The javadoc gives fg no value at all (「기본값은 각 컴포넌트에 따라 다르게」), so the API's only
        // documented "no colour" value is reused rather than a colour invented: -1 (setBackground, paintContent).
        jvm.put_field(&mut this, "fg", "I", -1).await?;

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

    // Kept, not drawn — the same as setBackground. ca7fa8ade8ad calls setForeground(I) at boot, right after it.
    async fn set_foreground(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fg: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setForeground({this:?}, {fg:#x})");

        jvm.put_field(&mut this, "fg", "I", fg).await
    }

    async fn get_foreground(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.Component::getForeground({this:?})");

        jvm.get_field(&this, "fg", "I").await
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

    // Recorded so a shell can hand keys to it (ShellComponent::keyNotify). focusNotify is not
    // called: it is still a stub, and no measured title overrides it. The listeners of the component
    // losing and the one gaining the focus are told (FOCUS_NOTIFY, arg1 0/1); with nothing behind
    // them to suppress, their answer is not read.
    async fn set_focus(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setFocus({this:?})");

        let old = ShellCard::focus(jvm).await?;
        if !old.is_null() && old.identity() == this.identity() {
            return Ok(());
        }
        ShellCard::set_focus(jvm, this.clone()).await?;
        if !old.is_null() {
            Self::notify_listener(jvm, &old, FOCUS_NOTIFY, 0, 0, 0).await?;
        }
        Self::notify_listener(jvm, &this, FOCUS_NOTIFY, 1, 0, 0).await?;

        Ok(())
    }

    // The javadoc: the listener sees every event first, and returning true means it took the event.
    // Delivered today: keys (ShellCard and ShellComponent::keyNotify, before keyNotify) and focus
    // moves (setFocus). Not delivered: SHOW_NOTIFY and POINTER_NOTIFY — no lwc child is shown or
    // pointed at here. A null listener removes the registration.
    async fn set_event_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<EventListener>,
        obj: ClassInstanceRef<Object>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.Component::setEventListener({this:?}, {listener:?}, {obj:?})");

        let listeners = Self::listeners(jvm).await?;
        if let Some(at) = Self::listener_index(jvm, &listeners, &this).await? {
            for _ in 0..3 {
                let _: () = jvm.invoke_virtual(&listeners, "java/util/Vector", "removeElementAt", "(I)V", (at,)).await?;
            }
        }
        if listener.is_null() {
            return Ok(());
        }
        let entry: [ClassInstanceRef<Object>; 3] = [this.instance.into(), listener.instance.into(), obj];
        for value in entry {
            let _: () = jvm
                .invoke_virtual(&listeners, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (value,))
                .await?;
        }

        Ok(())
    }

    /// Hands an event to `component`'s EventListener, if it has one; true means the listener took it.
    pub async fn notify_listener(jvm: &Jvm, component: &ClassInstanceRef<Self>, r#type: i32, arg1: i32, arg2: i32, arg3: i32) -> JvmResult<bool> {
        let listeners = Self::listeners(jvm).await?;
        let Some(at) = Self::listener_index(jvm, &listeners, component).await? else {
            return Ok(false);
        };
        let listener: ClassInstanceRef<EventListener> = jvm
            .invoke_virtual(&listeners, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (at + 1,))
            .await?;
        let obj: ClassInstanceRef<Object> = jvm
            .invoke_virtual(&listeners, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (at + 2,))
            .await?;

        jvm.invoke_virtual(
            &listener,
            "org/kwis/msp/lwc/EventListener",
            "eventNotify",
            "(IIIILjava/lang/Object;)Z",
            (r#type, arg1, arg2, arg3, obj),
        )
        .await
    }

    // ponytail: linear scan by identity, and a registration outlives its component unless the guest
    // clears it — a screen registers one or two. A map keyed by component when that stops holding.
    async fn listener_index(jvm: &Jvm, listeners: &ClassInstanceRef<Vector>, component: &ClassInstanceRef<Self>) -> JvmResult<Option<i32>> {
        let size: i32 = jvm.invoke_virtual(listeners, "java/util/Vector", "size", "()I", ()).await?;
        for at in (0..size).step_by(3) {
            let entry: ClassInstanceRef<Object> = jvm
                .invoke_virtual(listeners, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (at,))
                .await?;
            if entry.identity() == component.identity() {
                return Ok(Some(at));
            }
        }

        Ok(None)
    }

    async fn listeners(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Vector>> {
        let listeners: ClassInstanceRef<Vector> = jvm
            .get_static_field("org/kwis/msp/lwc/Component", "evtListeners", "Ljava/util/Vector;")
            .await?;
        if !listeners.is_null() {
            return Ok(listeners);
        }

        let listeners: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        jvm.put_static_field("org/kwis/msp/lwc/Component", "evtListeners", "Ljava/util/Vector;", listeners.clone())
            .await?;

        Ok(listeners)
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

    // Shown means "on the display", and the only component this layer puts on the display is a shell
    // on its net.wie.ShellCard — the same test repaint uses. A child of a shown shell answers false:
    // children are never laid out or drawn here. 0c67145b11df asks its shell from paint.
    async fn is_shown(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.Component::isShown({this:?})");

        Ok(ShellCard::find(jvm, &this).await?.is_some())
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

    // Still false although setFocus is now recorded: only a plain ShellComponent hands keys to the
    // focused widget, and games that see false keep handling keys themselves — which is what the
    // titles measured before the focus was recorded relied on. Answering truthfully is its own change.
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

    /// d448aee68157's wall after its first name entry: an AOT subclass reads the protected `x`
    /// directly and died on `Field xI not found from org/kwis/msp/lwc/Component`.
    #[test]
    fn position_and_size_fields_resolve_on_a_subclass_and_match_the_getters() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
            for field in ["x", "y", "w", "h"] {
                let value: i32 = jvm.get_field(&shell, field, "I").await?;
                assert_eq!(value, 0, "{field}");
            }

            Ok(())
        })
    }

    /// ca7fa8ade8ad's next boot wall: setForeground(I), the line after setBackground. Same shape — the
    /// -1 start must come from Component.<init> because ShellComponent skips ContainerComponent.<init>.
    #[test]
    fn set_foreground_is_kept_and_get_foreground_starts_unset() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
            let fg: i32 = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "getForeground", "()I", ())
                .await?;
            assert_eq!(fg, -1);

            let _: () = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "setForeground", "(I)V", (0x00654321,))
                .await?;
            let fg: i32 = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "getForeground", "()I", ())
                .await?;
            let bg: i32 = jvm
                .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "getBackground", "()I", ())
                .await?;
            assert_eq!((fg, bg), (0x00654321, -1));

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
