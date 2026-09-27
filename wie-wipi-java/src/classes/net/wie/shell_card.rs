use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{
    lcdui::{Card, Display, Graphics},
    lwc::{Component, ShellComponent},
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
            fields: vec![JavaFieldProto::new(
                "shell",
                "Lorg/kwis/msp/lwc/ShellComponent;",
                FieldAccessFlags::PRIVATE,
            )],
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

        let shell = Self::shell(jvm, &this).await?;
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

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::lcdui::Display as MidpDisplay;
    use wie_util::Result;

    use crate::{
        classes::{
            net::wie::CardCanvas,
            org::kwis::msp::lcdui::{Display, Graphics},
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

    // Reverting show() or Component.repaint to the old logged stubs turns this red: nothing reaches
    // the Display, so paintCount stays 0.
    #[test]
    fn shown_shell_component_is_painted_through_the_display() -> Result<()> {
        let test_jlet = WieJavaClassProto {
            name: "test/TestJlet",
            parent_class: Some("org/kwis/msp/lcdui/Jlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        let fixture: Box<[WieJavaClassProto]> = Vec::from([TestShell::as_proto(), SpyCardCanvas::as_proto(), test_jlet]).into_boxed_slice();
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), fixture]),
            |jvm| async move {
                let canvas: ClassInstanceRef<CardCanvas> = jvm.new_class("test/SpyCardCanvas", "()V", ()).await?.into();
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

                // Not shown yet: repaint has nowhere to go and must not throw.
                let _: () = jvm
                    .invoke_virtual(&shell, "org/kwis/msp/lwc/Component", "repaint", "(IIII)V", (0, 0, 10, 10))
                    .await?;
                assert_eq!(card_count().await?, 0);

                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&shell, "org/kwis/msp/lwc/ShellComponent", "show", "()V", ()).await?;
                assert_eq!(card_count().await?, 1, "show() twice puts the shell on the display once");
                assert_eq!(count("showCount").await?, 1);

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

                Ok(())
            },
        )
    }
}
