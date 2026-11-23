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
                JavaFieldProto::new(
                    "IMG_COPY",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "IMG_AND",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "IMG_OR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "IMG_XOR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
                JavaFieldProto::new("height2", "I", FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::<clinit>()");

        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_COPY", "I", 0).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_AND", "I", 1).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_OR", "I", 2).await?;
        jvm.put_static_field("com/xce/lcdui/XDisplay", "IMG_XOR", "I", 3).await?;

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

    // KEmulator: `Graphics2D.captureLCD(x, y, width, height)` drawn into `image.getGraphics()` at
    // (0, 0, TOP | LEFT); `graphics` is unused. Null or empty arguments stay the no-op they were.
    #[allow(clippy::too_many_arguments)]
    async fn copy_lcd(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        graphics: ClassInstanceRef<Graphics>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XDisplay::copyLCD({graphics:?}, {image:?}, {x}, {y}, {width}, {height})");

        if image.is_null() || width <= 0 || height <= 0 {
            return Ok(());
        }

        let captured: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "com/skt/m/Graphics2D",
                "captureLCD",
                "(IIII)Ljavax/microedition/lcdui/Image;",
                (x, y, width, height),
            )
            .await?;
        let target: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        jvm.invoke_virtual(
            &target,
            "javax/microedition/lcdui/Graphics",
            "drawImage",
            "(Ljavax/microedition/lcdui/Image;III)V",
            (captured, 0, 0, 20),
        )
        .await
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
    // NoSuchMethodError. A logged no-op: what it clears (and with what) is unknown.
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
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, JavaValue, Result as JvmResult};
    use jvm_types::ClassAccessFlags;
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};

    use wie_jvm_support::WieJavaClassProto;
    use wie_midp::classes::javax::microedition::{
        lcdui::{Graphics, Image},
        midlet::MIDlet,
    };

    use crate::classes::com::{skt::m::Graphics2D, xce::lcdui::Toolkit};

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

    // From upstream, on our drawImageEx (which forwards to Graphics2D.drawImage and ignores the second
    // image): upstream also runs every mode with an explicit destination image, which ours does not draw into.
    #[test]
    fn image_regions_apply_modes_translation_clipping_and_overlap() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Graphics2D::as_proto(), XDisplay::as_proto()].into()]),
            |jvm| async move {
                let mut pixels = jvm.instantiate_array("I", 8).await?;
                jvm.store_array(
                    &mut pixels,
                    0,
                    [
                        0xffabcdefu32 as i32,
                        0,
                        0,
                        0,
                        0xffee1122u32 as i32,
                        0xff0ff055u32 as i32,
                        0x00ffffff,
                        0xffee1122u32 as i32,
                    ],
                )
                .await?;
                let source: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createRGBImage",
                        "([IIIZ)Ljavax/microedition/lcdui/Image;",
                        (pixels, 4, 2, true),
                    )
                    .await?;
                for (mode, expected) in [(0, 0x0ff055), (1, 0x023054), (2, 0x1ff457), (3, 0x1dc403)] {
                    for explicit_destination in [false] {
                        let destination: ClassInstanceRef<Image> = jvm
                            .invoke_static(
                                "javax/microedition/lcdui/Image",
                                "createImage",
                                "(II)Ljavax/microedition/lcdui/Image;",
                                (5, 3),
                            )
                            .await?;
                        let target_graphics: ClassInstanceRef<Graphics> = jvm
                            .invoke_virtual(
                                &destination,
                                "javax/microedition/lcdui/Image",
                                "getGraphics",
                                "()Ljavax/microedition/lcdui/Graphics;",
                                (),
                            )
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&target_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&target_graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 5, 3))
                            .await?;
                        let graphics_image: ClassInstanceRef<Image> = if explicit_destination {
                            jvm.invoke_static(
                                "javax/microedition/lcdui/Image",
                                "createImage",
                                "(II)Ljavax/microedition/lcdui/Image;",
                                (5, 3),
                            )
                            .await?
                        } else {
                            destination.clone()
                        };
                        let graphics: ClassInstanceRef<Graphics> = jvm
                            .invoke_virtual(
                                &graphics_image,
                                "javax/microedition/lcdui/Image",
                                "getGraphics",
                                "()Ljavax/microedition/lcdui/Graphics;",
                                (),
                            )
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (1, 1))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (1, 0, 2, 1))
                            .await?;
                        let _: () = jvm
                            .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setXORMode", "(Z)V", (true,))
                            .await?;
                        let argument_destination = if explicit_destination {
                            destination.clone()
                        } else {
                            ClassInstanceRef::new(None)
                        };
                        let _: () = jvm
                            .invoke_static(
                                "com/xce/lcdui/XDisplay",
                                "drawImageEx",
                                "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                                vec![
                                    graphics.clone().into(),
                                    argument_destination.into(),
                                    0.into(),
                                    0.into(),
                                    source.clone().into(),
                                    0.into(),
                                    1.into(),
                                    4.into(),
                                    1.into(),
                                    mode.into(),
                                ],
                            )
                            .await?;
                        let image = Image::image(&jvm, &destination).await?;
                        for y in 0..3 {
                            for x in 0..5 {
                                let color = image.get_pixel(x, y);
                                let rgb = (i32::from(color.r) << 16) | (i32::from(color.g) << 8) | i32::from(color.b);
                                assert_eq!(
                                    rgb,
                                    if (x, y) == (2, 1) { expected } else { 0x123456 },
                                    "mode={mode}, explicit={explicit_destination}, pixel=({x},{y})"
                                );
                            }
                        }
                        if explicit_destination {
                            let pixel = Image::image(&jvm, &graphics_image).await?.get_pixel(2, 1);
                            assert_eq!((pixel.r, pixel.g, pixel.b), (255, 255, 255));
                        }
                        assert_eq!(
                            jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getTranslateX", "()I", ())
                                .await?,
                            1
                        );
                        assert_eq!(
                            jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                                .await?,
                            1
                        );
                        assert!(
                            jvm.get_field::<bool>(&graphics, "javax/microedition/lcdui/Graphics", "xorMode", "Z")
                                .await?
                        );
                    }
                }
                // Upstream also pins IMG_AND [0x0f, 0x03, 0x11, 0x00] and IMG_OR [0x0f, 0x3f, 0x77, 0xff] here; ours
                // blends front to back, so on an image drawn onto itself those two smear.
                for (mode, expected) in [(0, [0x0f, 0x0f, 0x33, 0x55]), (3, [0x0f, 0x3c, 0x66, 0xff])] {
                    let image: ClassInstanceRef<Image> = jvm
                        .invoke_static(
                            "javax/microedition/lcdui/Image",
                            "createImage",
                            "(II)Ljavax/microedition/lcdui/Image;",
                            (4, 1),
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
                    let mut pixels = jvm.instantiate_array("I", 4).await?;
                    jvm.store_array(&mut pixels, 0, [0x0f0f0f, 0x333333, 0x555555, 0xaaaaaa]).await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &graphics,
                            "javax/microedition/lcdui/Graphics",
                            "drawRGB",
                            "([IIIIIIIZ)V",
                            (pixels, 0, 4, 0, 0, 4, 1, false),
                        )
                        .await?;
                    let _: () = jvm
                        .invoke_static(
                            "com/xce/lcdui/XDisplay",
                            "drawImageEx",
                            "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IILjavax/microedition/lcdui/Image;IIIII)V",
                            vec![
                                graphics.into(),
                                image.clone().into(),
                                1.into(),
                                0.into(),
                                image.clone().into(),
                                0.into(),
                                0.into(),
                                4.into(),
                                1.into(),
                                mode.into(),
                            ],
                        )
                        .await?;
                    let image = Image::image(&jvm, &image).await?;
                    for (x, expected) in expected.into_iter().enumerate() {
                        let pixel = image.get_pixel(x as i32, 0);
                        assert_eq!((pixel.r, pixel.g, pixel.b), (expected, expected, expected), "overlap mode={mode}, x={x}");
                    }
                }
                Ok(())
            },
        )
        .unwrap();
    }

    // From upstream, on our refresh: ours asks the host for a redraw (upstream paints the screen image
    // there and then), so what is pinned is that the pending repaint survives it.
    #[test]
    fn toolkit_graphics_survives_midlet_startup_and_refresh_preserves_pending_repaint() {
        let midlet_proto = WieJavaClassProto {
            name: "test/RefreshMidlet",
            parent_class: Some("javax/microedition/midlet/MIDlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        };
        run_jvm_test_with_system(
            Box::new([
                wie_midp::get_protos().into(),
                [XDisplay::as_proto(), Toolkit::as_proto(), midlet_proto].into(),
            ]),
            Box::new(TestPlatform::new()),
            move |jvm, _system| async move {
                let graphics: ClassInstanceRef<Graphics> = jvm
                    .get_static_field("com/xce/lcdui/Toolkit", "graphics", "Ljavax/microedition/lcdui/Graphics;")
                    .await?;
                let midlet: ClassInstanceRef<MIDlet> = jvm.instantiate_class("test/RefreshMidlet").await?.into();
                let _: () = jvm
                    .invoke_special(&midlet, "javax/microedition/midlet/MIDlet", "<init>", "()V", ())
                    .await?;
                let display = MIDlet::display(&jvm, &midlet).await?;
                let screen_graphics: ClassInstanceRef<Graphics> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Display",
                        "getScreenGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                assert!(graphics.equals(&**screen_graphics)?);
                for pending in [false, true] {
                    if pending {
                        let _: () = jvm
                            .invoke_virtual(&display, "javax/microedition/lcdui/Display", "repaint", "(IIII)V", (0, 0, 320, 240))
                            .await?;
                    }
                    let _: () = jvm.invoke_static("com/xce/lcdui/XDisplay", "refresh", "(IIII)V", (3, 4, 2, 1)).await?;
                    assert_eq!(
                        jvm.get_field::<bool>(&display, "javax/microedition/lcdui/Display", "repaintPending", "Z")
                            .await?,
                        pending
                    );
                }
                Ok(())
            },
        )
        .unwrap();
    }
}
