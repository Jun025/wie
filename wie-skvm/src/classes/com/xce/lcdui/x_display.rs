use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

use crate::classes::com::skt::m::Graphics2D;

// class com.xce.lcdui.XDisplay
pub struct XDisplay;

impl XDisplay {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XDisplay",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("refresh", "(IIII)V", Self::refresh, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "copyLCD",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IIII)V",
                    Self::copy_lcd,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "drawImageEx",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                    Self::draw_image_ex,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "clear",
                    "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;II)V",
                    Self::clear,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("width", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height2", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::<clinit>()");

        // TODO: temp
        jvm.put_static_field("com/xce/lcdui/XDisplay", "width", "I", 240).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "height", "I", 320).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "height2", "I", 320).await?;

        Ok(())
    }

    async fn refresh(_jvm: &Jvm, context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XDisplay::refresh({x}, {y}, {width}, {height})");

        let platform = context.system().platform();
        let screen = platform.screen();
        screen.request_redraw().unwrap();

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn copy_lcd(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XDisplay::copyLCD({graphics:?}, {image:?}, {x}, {y}, {width}, {height})",);

        Ok(())
    }

    // Argument names and behaviour from KEmulator's XDisplay (gfx, image, tx, ty, srcImage, sx, sy, sw, sh, mode):
    // `image` is unused and the call forwards to Graphics2D.drawImage. fb80e97cbc57 passes null for `image`.
    #[allow(clippy::too_many_arguments)]
    async fn draw_image_ex(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        _image: ClassInstanceRef<Image>,
        tx: i32,
        ty: i32,
        src: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::drawImageEx({graphics:?}, {tx}, {ty}, {src:?}, {sx}, {sy}, {sw}, {sh}, {mode})");

        let graphics_2d: ClassInstanceRef<Graphics2D> = jvm
            .invoke_static(
                "com/skt/m/Graphics2D",
                "getGraphics2D",
                "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                (graphics,),
            )
            .await?;
        jvm.invoke_virtual(
            &graphics_2d,
            "com/skt/m/Graphics2D",
            "drawImage",
            "(IILjavax/microedition/lcdui/Image;IIIII)V",
            (tx, ty, src, sx, sy, sw, sh, mode),
        )
        .await
    }

    // No API document for this in the repo; 6e93f26fa2f5 calls it while loading a game and stops on
    // NoSuchMethodError. A logged no-op like copyLCD above: what it clears (and with what) is unknown.
    async fn clear(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.lcdui.XDisplay::clear({graphics:?}, {image:?}, {x}, {y})");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaValue, Result as JvmResult};
    use test_utils::run_jvm_test;

    use wie_midp::classes::javax::microedition::lcdui::{Graphics, Image};

    use crate::classes::com::skt::m::Graphics2D;

    use super::XDisplay;

    // fb80e97cbc57 calls exactly this descriptor with null as the second Image and modes 0/1; without
    // it the title stops on the logo with NoSuchMethodError. The source region lands at (tx, ty).
    #[test]
    fn draw_image_ex_draws_the_source_region_at_the_target_and_ignores_the_second_image() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Graphics2D::as_proto(), XDisplay::as_proto()].into()]),
            |jvm| async move {
                let target: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (3, 1),
                    )
                    .await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &target,
                        "javax/microedition/lcdui/Image",
                        "getGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let source: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (2, 1),
                    )
                    .await?;
                let source_graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &source,
                        "javax/microedition/lcdui/Image",
                        "getGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&source_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xabcdef,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&source_graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (1, 0, 1, 1))
                    .await?;

                let _: () = jvm
                    .invoke_static(
                        "com/xce/lcdui/XDisplay",
                        "drawImageEx",
                        "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                        [
                            graphics.into(),
                            JavaValue::Object(None),
                            JavaValue::Int(2),
                            JavaValue::Int(0),
                            source.into(),
                            JavaValue::Int(1),
                            JavaValue::Int(0),
                            JavaValue::Int(1),
                            JavaValue::Int(1),
                            JavaValue::Int(0),
                        ],
                    )
                    .await?;

                let target_image = Image::image(&jvm, &target).await?;
                let untouched = target_image.get_pixel(1, 0);
                assert_eq!((untouched.r, untouched.g, untouched.b), (0, 0, 0));
                let drawn = target_image.get_pixel(2, 0);
                assert_eq!((drawn.r, drawn.g, drawn.b), (0xab, 0xcd, 0xef));

                Ok(())
            },
        )
        .unwrap();
    }

    // 6e93f26fa2f5 resolves exactly this descriptor while loading a game; without it the load ends
    // in NoSuchMethodError and the title never reaches play.
    #[test]
    fn clear_is_resolvable_with_the_descriptor_a_title_calls() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [XDisplay::as_proto()].into()]),
            |jvm| async move {
                let image: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (4, 4),
                    )
                    .await?;
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &image,
                        "javax/microedition/lcdui/Image",
                        "getGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let r: JvmResult<()> = jvm
                    .invoke_static(
                        "com/xce/lcdui/XDisplay",
                        "clear",
                        "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;II)V",
                        (graphics, image, 0, 0),
                    )
                    .await;
                assert!(r.is_ok());

                Ok(())
            },
        )
        .unwrap();
    }
}
