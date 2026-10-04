use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Card, Display, Graphics},
    lwc::{Component, KEY_NOTIFY, ShellComponent},
};

// class net.wie.ShellCard
//
// The Card a shown lwc ShellComponent sits on. lwc has no screen of its own in wie, so show()
// pushes one of these onto the Display and every repaint the shell asks for goes down the same
// Card → CardCanvas path the Card games already use. The link lives here, not on the shell: a field
// on an lwc class would change the instance layout an LGT AOT subclass computes its own fields from.
pub struct ShellCard;

impl ShellCard {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/ShellCard",
            parent_class: Some("org/kwis/msp/lcdui/Card"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/lwc/ShellComponent;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![
                JavaFieldProto::new("shell", "Lorg/kwis/msp/lwc/ShellComponent;", FieldAccessFlags::PRIVATE),
                // The one lwc component that has the input focus (Component.setFocus). Static and here,
                // not an instance field on an lwc class, for the layout reason above; one is enough
                // because only the shell on top of the display receives keys.
                JavaFieldProto::new(
                    "focus",
                    "Lorg/kwis/msp/lwc/Component;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, shell: ClassInstanceRef<ShellComponent>) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::<init>({this:?}, {shell:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lcdui/Card", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "shell", "Lorg/kwis/msp/lwc/ShellComponent;", shell).await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::paint({this:?}, {g:?})");

        let shell = Self::shell(jvm, &this).await?;
        jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", (g,))
            .await
    }

    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("net.wie.ShellCard::keyNotify({this:?}, {type}, {key})");

        // The shell's own EventListener sees the key before the shell does (Component.setEventListener).
        let shell = Self::shell(jvm, &this).await?;
        let component: ClassInstanceRef<Component> = shell.clone().instance.into();
        if Component::notify_listener(jvm, &component, KEY_NOTIFY, r#type, key, 0).await? {
            return Ok(true);
        }
        jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "keyNotify", "(II)Z", (r#type, key))
            .await
    }

    async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
        tracing::debug!("net.wie.ShellCard::showNotify({this:?}, {show})");

        let shell = Self::shell(jvm, &this).await?;
        jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "showNotify", "(Z)V", (show,))
            .await
    }

    async fn shell(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<ShellComponent>> {
        jvm.get_field(this, "shell", "Lorg/kwis/msp/lwc/ShellComponent;").await
    }

    pub async fn focus(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Component>> {
        jvm.get_static_field("net/wie/ShellCard", "focus", "Lorg/kwis/msp/lwc/Component;").await
    }

    pub async fn set_focus(jvm: &Jvm, component: ClassInstanceRef<Component>) -> JvmResult<()> {
        jvm.put_static_field("net/wie/ShellCard", "focus", "Lorg/kwis/msp/lwc/Component;", component)
            .await
    }

    /// The ShellCard currently on the default Display for `component`, if it is a shown shell.
    // ponytail: linear scan of the card stack — it holds one or two cards in practice.
    pub async fn find(jvm: &Jvm, component: &ClassInstanceRef<Component>) -> JvmResult<Option<ClassInstanceRef<Card>>> {
        if component.is_null() {
            return Ok(None);
        }

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
            .await?;
        if display.is_null() {
            return Ok(None);
        }
        let card_canvas = Display::card_canvas(jvm, &display).await?;
        let cards = jvm.get_field(&card_canvas, "cards", "Ljava/util/Vector;").await?;
        let length: i32 = jvm.invoke_virtual(&cards, "java/util/Vector", "size", "()I", ()).await?;

        for i in 0..length {
            let card: ClassInstanceRef<Card> = jvm
                .invoke_virtual(&cards, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (i,))
                .await?;
            if card.is_null() || !jvm.is_instance(&**card, "net/wie/ShellCard") {
                continue;
            }
            let shell: ClassInstanceRef<ShellComponent> = jvm.get_field(&card, "shell", "Lorg/kwis/msp/lwc/ShellComponent;").await?;
            if !shell.is_null() && shell.identity() == component.identity() {
                return Ok(Some(card));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::{Object, String};
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_backend::Platform;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::lcdui::Display as MidpDisplay;
    use wie_util::Result;

    use crate::{
        classes::{
            net::wie::{CardCanvas, ShellCard, WIPIKeyCode},
            org::kwis::msp::{
                lcdui::{Display, Graphics},
                lwc::Component,
            },
        },
        get_protos,
    };

    // The shape 학교가는길 has: a ShellComponent subclass that overrides paint/keyNotify/showNotify
    // and never touches a Card.
    struct TestShell;

    impl TestShell {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TestShell",
                parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PROTECTED),
                    JavaMethodProto::new("showNotify", "(Z)V", Self::show_notify, MethodAccessFlags::PROTECTED),
                ],
                fields: vec![
                    JavaFieldProto::new("paintCount", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("keyCount", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("showCount", "I", FieldAccessFlags::PRIVATE),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await
        }

        async fn bump(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, field: &str) -> JvmResult<()> {
            let count: i32 = jvm.get_field(this, field, "I").await?;
            jvm.put_field(this, field, "I", count + 1).await
        }

        async fn paint(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: ClassInstanceRef<Graphics>) -> JvmResult<()> {
            Self::bump(jvm, &mut this, "paintCount").await
        }

        async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, _: i32, _: i32) -> JvmResult<bool> {
            Self::bump(jvm, &mut this, "keyCount").await?;
            Ok(false)
        }

        async fn show_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, show: bool) -> JvmResult<()> {
            if show { Self::bump(jvm, &mut this, "showCount").await } else { Ok(()) }
        }
    }

    // Records the region repaint a Card forwards, like card.rs's TestCanvas, but is a real
    // CardCanvas so paint and key dispatch still run.
    struct SpyCardCanvas;

    impl SpyCardCanvas {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/SpyCardCanvas",
                parent_class: Some("net/wie/CardCanvas"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("repaintX", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("repaintY", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("repaintWidth", "I", FieldAccessFlags::PRIVATE),
                    JavaFieldProto::new("repaintHeight", "I", FieldAccessFlags::PRIVATE),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "net/wie/CardCanvas", "<init>", "()V", ()).await
        }

        async fn repaint(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, w: i32, h: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "repaintX", "I", x).await?;
            jvm.put_field(&mut this, "repaintY", "I", y).await?;
            jvm.put_field(&mut this, "repaintWidth", "I", w).await?;
            jvm.put_field(&mut this, "repaintHeight", "I", h).await?;
            // Forward, so a serviceRepaints that only paints pending regions (#345) still sees one.
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (x, y, w, h))
                .await
        }
    }

    fn test_jlet() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "test/TestJlet",
            parent_class: Some("org/kwis/msp/lcdui/Jlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // A default Display on a card canvas of class `canvas_class`, the way a running Jlet has one.
    async fn install_display(jvm: &Jvm, canvas_class: &str) -> JvmResult<ClassInstanceRef<CardCanvas>> {
        let canvas: ClassInstanceRef<CardCanvas> = jvm.new_class(canvas_class, "()V", ()).await?.into();
        let midp_display: ClassInstanceRef<MidpDisplay> = jvm.new_class("javax/microedition/lcdui/Display", "()V", ()).await?.into();
        let _: () = jvm
            .invoke_virtual(
                &midp_display,
                "javax/microedition/lcdui/Display",
                "setCurrent",
                "(Ljavax/microedition/lcdui/Displayable;)V",
                (canvas.clone(),),
            )
            .await?;
        let mut display: ClassInstanceRef<Display> = jvm.instantiate_class("org/kwis/msp/lcdui/Display").await?.into();
        jvm.put_field(&mut display, "cardCanvas", "Lnet/wie/CardCanvas;", canvas.clone()).await?;
        jvm.put_field(&mut display, "midpDisplay", "Ljavax/microedition/lcdui/Display;", midp_display)
            .await?;
        let mut jlet = jvm.instantiate_class("test/TestJlet").await?;
        jvm.put_field(&mut jlet, "dis", "Lorg/kwis/msp/lcdui/Display;", display).await?;
        jvm.put_static_field("org/kwis/msp/lcdui/Jlet", "currentJlet", "Lorg/kwis/msp/lcdui/Jlet;", jlet)
            .await?;

        Ok(canvas)
    }

    // The game's listener: counts action calls and keeps the last (component, object) pair.
    struct TestListener;

    impl TestListener {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TestListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["org/kwis/msp/lwc/ActionListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new(
                        "action",
                        "(Lorg/kwis/msp/lwc/Component;Ljava/lang/Object;)V",
                        Self::action,
                        MethodAccessFlags::PUBLIC,
                    ),
                ],
                fields: vec![
                    JavaFieldProto::new("count", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("cmp", "Lorg/kwis/msp/lwc/Component;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("o", "Ljava/lang/Object;", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn action(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            cmp: ClassInstanceRef<Component>,
            o: ClassInstanceRef<Object>,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "count", "I").await?;
            jvm.put_field(&mut this, "count", "I", count + 1).await?;
            jvm.put_field(&mut this, "cmp", "Lorg/kwis/msp/lwc/Component;", cmp).await?;
            jvm.put_field(&mut this, "o", "Ljava/lang/Object;", o).await
        }
    }

    // A guest EventListener: records every eventNotify and takes (returns true for) one key code.
    struct TestEventListener;

    impl TestEventListener {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TestEventListener",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["org/kwis/msp/lwc/EventListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("eventNotify", "(IIIILjava/lang/Object;)Z", Self::event_notify, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("focusGained", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("keys", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("lastKey", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("take", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("o", "Ljava/lang/Object;", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        #[allow(clippy::too_many_arguments)] // the arity is the Java descriptor's
        async fn event_notify(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            r#type: i32,
            arg1: i32,
            arg2: i32,
            _: i32,
            o: ClassInstanceRef<Object>,
        ) -> JvmResult<bool> {
            jvm.put_field(&mut this, "o", "Ljava/lang/Object;", o).await?;
            match (r#type, arg1) {
                (1, 1) => {
                    let n: i32 = jvm.get_field(&this, "focusGained", "I").await?;
                    jvm.put_field(&mut this, "focusGained", "I", n + 1).await?;
                    Ok(false)
                }
                (3, 1) => {
                    let n: i32 = jvm.get_field(&this, "keys", "I").await?;
                    jvm.put_field(&mut this, "keys", "I", n + 1).await?;
                    jvm.put_field(&mut this, "lastKey", "I", arg2).await?;
                    let take: i32 = jvm.get_field(&this, "take", "I").await?;
                    Ok(arg2 == take)
                }
                _ => Ok(false),
            }
        }
    }

    // Reverting show() or Component.repaint to the old logged stubs turns this red: nothing reaches
    // the Display, so paintCount stays 0.
    #[test]
    fn shown_shell_component_is_painted_through_the_display() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestShell::as_proto(), SpyCardCanvas::as_proto(), test_jlet()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let canvas = install_display(&jvm, "test/SpyCardCanvas").await?;

                let shell: ClassInstanceRef<TestShell> = jvm.new_class("test/TestShell", "()V", ()).await?.into();
                let count = |name: &'static str| {
                    let shell = shell.clone();
                    let jvm = jvm.clone();
                    async move { jvm.get_field::<i32>(&shell, name, "I").await }
                };
                let card_count = || {
                    let canvas = canvas.clone();
                    let jvm = jvm.clone();
                    async move { jvm.invoke_virtual::<_, i32>(&canvas, "net/wie/CardCanvas", "countCard", "()I", ()).await }
                };

                let is_shown = || {
                    let shell = shell.clone();
                    let jvm = jvm.clone();
                    async move {
                        jvm.invoke_virtual::<_, bool>(&shell, "org/kwis/msp/lwc/Component", "isShown", "()Z", ())
                            .await
                    }
                };
                // Not shown yet: repaint has nowhere to go and must not throw.
                assert!(!is_shown().await?);
                let _: () = jvm
                    .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "repaint", "(IIII)V", (0, 0, 10, 10))
                    .await?;
                assert_eq!(card_count().await?, 0);

                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;
                assert_eq!(card_count().await?, 1, "show() twice puts the shell on the display once");
                assert_eq!(count("showCount").await?, 1);
                assert!(is_shown().await?, "isShown asks the same question repaint does");

                // The shell answers the screen it is on: be08d047cbae repaints (0, 0, getWidth(),
                // getHeight()) after every key, and Component's 0 × 0 made that repaint cover nothing.
                let display: ClassInstanceRef<Display> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
                    .await?;
                for method in ["getWidth", "getHeight"] {
                    let size: i32 = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/Component", method, "()I", ()).await?;
                    let expected: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", method, "()I", ()).await?;
                    assert!(size > 0, "shell {method} = {size}");
                    assert_eq!(size, expected);
                }
                // …but the status strip, a ShellComponent this layer never draws, takes no room.
                let strip = jvm.new_class("org/kwis/msp/lwc/AnnunciatorComponent", "(Z)V", (false,)).await?;
                let strip_height: i32 = jvm.invoke_virtual(&strip, "org/kwis/msp/lwc/Component", "getHeight", "()I", ()).await?;
                assert_eq!(strip_height, 0);

                // repaint(IIII) reaches the canvas as a region repaint (the shell's card sits at 0,0)…
                let _: () = jvm
                    .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "repaint", "(IIII)V", (1, 2, 3, 4))
                    .await?;
                let region = [
                    jvm.get_field::<i32>(&canvas, "repaintX", "I").await?,
                    jvm.get_field::<i32>(&canvas, "repaintY", "I").await?,
                    jvm.get_field::<i32>(&canvas, "repaintWidth", "I").await?,
                    jvm.get_field::<i32>(&canvas, "repaintHeight", "I").await?,
                ];
                assert_eq!(region, [1, 2, 3, 4]);

                // …and serviceRepaints lands in the shell's own paint.
                let painted = count("paintCount").await?;
                let _: () = jvm
                    .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "serviceRepaints", "()V", ())
                    .await?;
                assert_eq!(count("paintCount").await?, painted + 1);

                let _: () = jvm.invoke_virtual(&canvas, "net/wie/CardCanvas", "keyPressed", "(I)V", (42,)).await?;
                assert_eq!(count("keyCount").await?, 1);

                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "hide", "()V", ()).await?;
                assert_eq!(card_count().await?, 0);
                assert!(!is_shown().await?);

                Ok(())
            },
        )
    }

    /// A shown status strip takes 24 rows off the top of a 240-wide screen: d1e0badfce82 shows it, then
    /// makes its Card and sizes a 32-row table by `getHeight() / 10 + 1` — 320 rows overran it on
    /// every paint. A card made before the strip is shown keeps the whole screen.
    #[test]
    fn shown_strip_moves_new_cards_below_it_on_a_240_wide_screen() -> Result<()> {
        shown_strip_moves_new_cards_below_it(240, 320, 24)
    }

    /// 176-wide screens lose 18 rows: 202 = 220 − 18 is the commonest full-width image height there.
    #[test]
    fn shown_strip_moves_new_cards_below_it_on_a_176_wide_screen() -> Result<()> {
        shown_strip_moves_new_cards_below_it(176, 220, 18)
    }

    fn shown_strip_moves_new_cards_below_it(w: i32, h: i32, rows: i32) -> Result<()> {
        let platform = TestPlatform::new();
        platform.screen().resize(w as u32, h as u32)?;
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestShell::as_proto(), test_jlet()]).into_boxed_slice();
        run_jvm_test_with_system(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            Box::new(platform),
            move |jvm, _| async move {
                let _ = install_display(&jvm, "net/wie/CardCanvas").await?;
                let display: ClassInstanceRef<Display> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Display", "getDefaultDisplay", "()Lorg/kwis/msp/lcdui/Display;", [])
                    .await?;
                let bounds = |card: ClassInstanceRef<Object>| {
                    let jvm = jvm.clone();
                    async move {
                        let mut v = [0; 3];
                        for (slot, method) in v.iter_mut().zip(["getY", "getWidth", "getHeight"]) {
                            *slot = jvm.invoke_virtual::<_, i32>(&card, "org/kwis/msp/lcdui/Card", method, "()I", ()).await?;
                        }
                        Ok::<_, jvm::JavaError>(v)
                    }
                };
                let new_card = || {
                    let jvm = jvm.clone();
                    async move {
                        let shell = jvm.new_class("test/TestShell", "()V", ()).await?;
                        jvm.new_class("net/wie/ShellCard", "(Lorg/kwis/msp/lwc/ShellComponent;)V", (shell,)).await
                    }
                };

                assert_eq!(bounds(new_card().await?.into()).await?, [0, w, h]);

                let strip = jvm.new_class("org/kwis/msp/lwc/AnnunciatorComponent", "(Z)V", (false,)).await?;
                let _: () = jvm
                    .invoke_virtual(&strip, "org/kwis/msp/lwc/AnnunciatorComponent", "show", "()V", ())
                    .await?;

                let height: i32 = jvm.invoke_virtual(&display, "org/kwis/msp/lcdui/Display", "getHeight", "()I", ()).await?;
                assert_eq!(height, h - rows);
                assert_eq!(bounds(new_card().await?.into()).await?, [rows, w, h - rows]);

                Ok(())
            },
        )
    }

    /// 0c67145b11df's name form, built in the title's order: two text boxes, a ChoiceText and a
    /// button on a GFormComponent, the form on a plain ShellComponent, focus on the first box. Every
    /// key used to stop at the shell; now DOWN walks the focus to the button and FIRE calls the
    /// listener — once, with the button and the registered object. On the way, digits typed into the
    /// first box come back from getString and RIGHT moves the ChoiceText selection.
    #[test]
    fn shell_walks_focus_to_the_button_and_fire_calls_its_listener() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestListener::as_proto(), test_jlet()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                // The display a running title has: the form's repaint after each press goes looking for it.
                let _ = install_display(&jvm, "net/wie/CardCanvas").await?;
                let form: ClassInstanceRef<Component> = jvm.new_class("com/ktf/kfc/GFormComponent", "()V", ()).await?.into();
                let mut widgets: Vec<ClassInstanceRef<Component>> = Vec::new();
                for _ in 0..2 {
                    let text = JavaLangString::from_rust_string(&jvm, "").await?;
                    widgets.push(
                        jvm.new_class("org/kwis/msp/lwc/TextBoxComponent", "(Ljava/lang/String;I)V", (text, 0))
                            .await?
                            .into(),
                    );
                }
                let mut choices = jvm.instantiate_array("Ljava/lang/String;", 2).await?;
                let choice_strings = [
                    JavaLangString::from_rust_string(&jvm, "a").await?,
                    JavaLangString::from_rust_string(&jvm, "b").await?,
                ];
                jvm.store_array(&mut choices, 0, choice_strings).await?;
                widgets.push(
                    jvm.new_class("com/ktf/kfc/ChoiceText", "([Ljava/lang/String;)V", (choices,))
                        .await?
                        .into(),
                );
                let image: ClassInstanceRef<Object> = None.into();
                let label: ClassInstanceRef<Object> = None.into();
                let button: ClassInstanceRef<Component> = jvm
                    .new_class(
                        "org/kwis/msp/lwc/ButtonComponent",
                        "(Ljava/lang/String;Lorg/kwis/msp/lcdui/Image;)V",
                        (label, image),
                    )
                    .await?
                    .into();
                widgets.push(button.clone());
                for widget in &widgets {
                    let _: i32 = jvm
                        .invoke_virtual(
                            &form,
                            "com/ktf/kfc/GFormComponent",
                            "addComponent",
                            "(Lorg/kwis/msp/lwc/Component;IIII)I",
                            (widget.clone(), 0, 0, 0, 0),
                        )
                        .await?;
                }
                let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
                let _: i32 = jvm
                    .invoke_virtual(
                        &shell,
                        "org/kwis/msp/lwc/ContainerComponent",
                        "addComponent",
                        "(Lorg/kwis/msp/lwc/Component;)I",
                        (form,),
                    )
                    .await?;
                let listener = jvm.new_class("test/TestListener", "()V", ()).await?;
                let tag: ClassInstanceRef<Object> = jvm.new_class("java/lang/Object", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &button,
                        "org/kwis/msp/lwc/ButtonComponent",
                        "setActionListener",
                        "(Lorg/kwis/msp/lwc/ActionListener;Ljava/lang/Object;)V",
                        (listener.clone(), tag.clone()),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&widgets[0], "org/kwis/msp/lwc/Component", "setFocus", "()V", ())
                    .await?;

                // Press and release, the way net.wie.CardCanvas delivers a key.
                let key = |code: WIPIKeyCode| {
                    let shell = shell.clone();
                    let jvm = jvm.clone();
                    async move {
                        for r#type in [1, 2] {
                            let _: bool = jvm
                                .invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "keyNotify", "(II)Z", (r#type, code as i32))
                                .await?;
                        }
                        JvmResult::Ok(())
                    }
                };
                let count = || {
                    let listener = listener.clone();
                    let jvm = jvm.clone();
                    async move { jvm.get_field::<i32>(&listener, "count", "I").await }
                };

                key(WIPIKeyCode::FIRE).await?;
                assert_eq!(count().await?, 0, "FIRE on the text box is not the button's");

                // Digits and CLR edit the focused text box (천지인 Hangul first: ㄱ ㅣ ㆍ ㄱ, CLR takes the
                // final back), '*' moves to English; getString reads it back.
                use WIPIKeyCode::*;
                for code in [NUM4, NUM1, NUM2, NUM4, CLEAR, STAR, NUM2] {
                    key(code).await?;
                }
                let typed: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&widgets[0], "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &typed).await?, "가A");

                // RIGHT on the ChoiceText moves the selection the listener reads.
                key(WIPIKeyCode::DOWN).await?;
                key(WIPIKeyCode::DOWN).await?;
                key(WIPIKeyCode::RIGHT).await?;
                let selected: i32 = jvm
                    .invoke_virtual(&widgets[2], "com/ktf/kfc/ChoiceText", "getSelectedIndex", "()I", ())
                    .await?;
                assert_eq!(selected, 1);

                key(WIPIKeyCode::DOWN).await?;
                key(WIPIKeyCode::FIRE).await?;
                assert_eq!(count().await?, 1);
                let cmp: ClassInstanceRef<Component> = jvm.get_field(&listener, "cmp", "Lorg/kwis/msp/lwc/Component;").await?;
                let o: ClassInstanceRef<Object> = jvm.get_field(&listener, "o", "Ljava/lang/Object;").await?;
                assert_eq!(cmp.identity(), button.identity());
                assert_eq!(o.identity(), tag.identity());

                // DOWN past the last leaf wraps to the first; UP from there comes back to the button.
                key(WIPIKeyCode::DOWN).await?;
                key(WIPIKeyCode::FIRE).await?;
                assert_eq!(count().await?, 1);
                key(WIPIKeyCode::UP).await?;
                key(WIPIKeyCode::FIRE).await?;
                assert_eq!(count().await?, 2);

                Ok(())
            },
        )
    }

    /// 65ef7052f528's ID entry: a text field on a shell, the listener registered on the field, show(),
    /// and no setFocus anywhere — the title waits for its listener to hear OK. Each piece reddens one
    /// assertion when reverted: setEventListener (the method is missing), show's first focus (no
    /// FOCUS_NOTIFY, no keys), the listener-first dispatch (a taken key still edits the field).
    #[test]
    fn event_listener_on_a_shown_text_field_hears_focus_and_keys_first() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestEventListener::as_proto(), test_jlet()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let canvas = install_display(&jvm, "net/wie/CardCanvas").await?;
                let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
                let text: ClassInstanceRef<String> = None.into();
                let field: ClassInstanceRef<Component> = jvm
                    .new_class("org/kwis/msp/lwc/TextFieldComponent", "(Ljava/lang/String;I)V", (text, 0))
                    .await?
                    .into();
                let _: i32 = jvm
                    .invoke_virtual(
                        &shell,
                        "org/kwis/msp/lwc/ContainerComponent",
                        "addComponent",
                        "(Lorg/kwis/msp/lwc/Component;)I",
                        (field.clone(),),
                    )
                    .await?;
                let mut listener = jvm.new_class("test/TestEventListener", "()V", ()).await?;
                jvm.put_field(&mut listener, "take", "I", WIPIKeyCode::NUM1 as i32).await?;
                let tag: ClassInstanceRef<Object> = jvm.new_class("java/lang/Object", "()V", ()).await?.into();
                let _: () = jvm
                    .invoke_virtual(
                        &field,
                        "org/kwis/msp/lwc/Component",
                        "setEventListener",
                        "(Lorg/kwis/msp/lwc/EventListener;Ljava/lang/Object;)V",
                        (listener.clone(), tag.clone()),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;

                let field_ = |name: &'static str| {
                    let listener = listener.clone();
                    let jvm = jvm.clone();
                    async move { jvm.get_field::<i32>(&listener, name, "I").await }
                };
                let typed = || {
                    let field = field.clone();
                    let jvm = jvm.clone();
                    async move {
                        let s: ClassInstanceRef<String> = jvm
                            .invoke_virtual(&field, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
                            .await?;
                        JvmResult::Ok(JavaLangString::to_rust_string(&jvm, &s).await?)
                    }
                };
                let press = |code: WIPIKeyCode| {
                    let canvas = canvas.clone();
                    let jvm = jvm.clone();
                    async move {
                        jvm.invoke_virtual::<_, ()>(&canvas, "net/wie/CardCanvas", "keyPressed", "(I)V", (code as i32,))
                            .await
                    }
                };

                assert_eq!(field_("focusGained").await?, 1, "show() gave the field the focus and told its listener");
                let o: ClassInstanceRef<Object> = jvm.get_field(&listener, "o", "Ljava/lang/Object;").await?;
                assert_eq!(o.identity(), tag.identity());

                // Taken by the listener: the field never sees it.
                press(WIPIKeyCode::NUM1).await?;
                assert_eq!((field_("keys").await?, field_("lastKey").await?), (1, WIPIKeyCode::NUM1 as i32));
                assert_eq!(typed().await?, "");

                // Not taken: the listener hears it, then the field types it.
                press(WIPIKeyCode::NUM2).await?;
                assert_eq!(field_("keys").await?, 2);
                assert_ne!(typed().await?, "");

                // A null listener unregisters.
                let none: ClassInstanceRef<Object> = None.into();
                let _: () = jvm
                    .invoke_virtual(
                        &field,
                        "org/kwis/msp/lwc/Component",
                        "setEventListener",
                        "(Lorg/kwis/msp/lwc/EventListener;Ljava/lang/Object;)V",
                        (none.clone(), none),
                    )
                    .await?;
                press(WIPIKeyCode::NUM1).await?;
                assert_eq!(field_("keys").await?, 2);

                Ok(())
            },
        )
    }

    /// 0c67145b11df's OK with an empty box: the button's listener calls FormComponent.setFocus(box) on
    /// the GFormComponent. Reverting either the method or GFormComponent's FormComponent parent makes
    /// the call fail to resolve; after it the next key types into that box.
    #[test]
    fn form_component_set_focus_on_a_g_form_moves_the_keys_to_that_box() -> Result<()> {
        let fixture: Box<[WieJavaClassProto]> = Vec::from([test_jlet()]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let _ = install_display(&jvm, "net/wie/CardCanvas").await?;
                let form: ClassInstanceRef<Component> = jvm.new_class("com/ktf/kfc/GFormComponent", "()V", ()).await?.into();
                let mut boxes: Vec<ClassInstanceRef<Component>> = Vec::new();
                for _ in 0..2 {
                    let text = JavaLangString::from_rust_string(&jvm, "").await?;
                    let widget: ClassInstanceRef<Component> = jvm
                        .new_class("org/kwis/msp/lwc/TextBoxComponent", "(Ljava/lang/String;I)V", (text, 0))
                        .await?
                        .into();
                    let _: i32 = jvm
                        .invoke_virtual(
                            &form,
                            "com/ktf/kfc/GFormComponent",
                            "addComponent",
                            "(Lorg/kwis/msp/lwc/Component;IIII)I",
                            (widget.clone(), 0, 0, 0, 0),
                        )
                        .await?;
                    boxes.push(widget);
                }
                let shell = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?;
                let _: i32 = jvm
                    .invoke_virtual(
                        &shell,
                        "org/kwis/msp/lwc/ContainerComponent",
                        "addComponent",
                        "(Lorg/kwis/msp/lwc/Component;)I",
                        (form.clone(),),
                    )
                    .await?;
                let _: () = jvm.invoke_virtual(&boxes[0], "org/kwis/msp/lwc/Component", "setFocus", "()V", ()).await?;

                let _: () = jvm
                    .invoke_virtual(
                        &form,
                        "org/kwis/msp/lwc/FormComponent",
                        "setFocus",
                        "(Lorg/kwis/msp/lwc/Component;)V",
                        (boxes[1].clone(),),
                    )
                    .await?;
                assert_eq!(ShellCard::focus(&jvm).await?.identity(), boxes[1].identity());

                let _: bool = jvm
                    .invoke_virtual(
                        &shell,
                        "org/kwis/msp/lwc/ShellComponent",
                        "keyNotify",
                        "(II)Z",
                        (1, WIPIKeyCode::NUM4 as i32),
                    )
                    .await?;
                let mut typed = Vec::new();
                for widget in &boxes {
                    let s: ClassInstanceRef<String> = jvm
                        .invoke_virtual(widget, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
                        .await?;
                    typed.push(JavaLangString::to_rust_string(&jvm, &s).await?.is_empty());
                }
                assert_eq!(typed, [true, false], "the key went to the box setFocus named");

                // A null component is ignored rather than clearing the focus.
                let none: ClassInstanceRef<Component> = None.into();
                let _: () = jvm
                    .invoke_virtual(
                        &form,
                        "org/kwis/msp/lwc/FormComponent",
                        "setFocus",
                        "(Lorg/kwis/msp/lwc/Component;)V",
                        (none,),
                    )
                    .await?;
                assert_eq!(ShellCard::focus(&jvm).await?.identity(), boxes[1].identity());

                Ok(())
            },
        )
    }
}
