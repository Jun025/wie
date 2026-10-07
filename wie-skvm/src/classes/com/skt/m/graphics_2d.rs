use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::{Clip, Color, PixelType, Rgb8Pixel};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::{
    lcdui::{Graphics, Image},
    midlet::MIDlet,
};

// class com.skt.m.Graphics2D
pub struct Graphics2D;

impl Graphics2D {
    const DRAW_COPY: i32 = 0;
    const DRAW_AND: i32 = 1;
    const DRAW_OR: i32 = 2;
    const DRAW_XOR: i32 = 3;

    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Graphics2D",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Graphics;)V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new(
                    "getGraphics2D",
                    "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                    Self::get_graphics2d,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "captureLCD",
                    "(IIII)Ljavax/microedition/lcdui/Image;",
                    Self::capture_lcd,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "drawImage",
                    "(IILjavax/microedition/lcdui/Image;IIIII)V",
                    Self::draw_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "createMaskableImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    Self::create_maskable_image,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getPixel", "(II)I", Self::get_pixel, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getPixelMask", "(II)Z", Self::get_pixel_mask, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("invertRect", "(IIII)V", Self::invert_rect, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPixel", "(III)V", Self::set_pixel, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setPixelMask", "(IIZ)V", Self::set_pixel_mask, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "DRAW_COPY",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DRAW_AND",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DRAW_OR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new(
                    "DRAW_XOR",
                    "I",
                    FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL,
                ),
                JavaFieldProto::new("graphics", "Ljavax/microedition/lcdui/Graphics;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::<clinit>()");

        for (name, value) in [
            ("DRAW_COPY", Self::DRAW_COPY),
            ("DRAW_AND", Self::DRAW_AND),
            ("DRAW_OR", Self::DRAW_OR),
            ("DRAW_XOR", Self::DRAW_XOR),
        ] {
            jvm.put_static_field("com/skt/m/Graphics2D", name, "I", value).await?;
        }

        Ok(())
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::<init>({this:?}, {graphics:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        jvm.put_field(&mut this, "graphics", "Ljavax/microedition/lcdui/Graphics;", graphics)
            .await?;

        Ok(())
    }

    async fn get_graphics2d(jvm: &Jvm, _context: &mut WieJvmContext, graphics: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.skt.m.Graphics2D::getGraphics2D({graphics:?})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let instance = jvm
            .new_class("com/skt/m/Graphics2D", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
            .await?;

        Ok(instance.into())
    }

    // KEmulator's Graphics2D.captureLCD crops the screen back buffer: `Image.createImage(backBuffer, x, y, w, h, 0)`.
    // Ours is the current Display's screen image — what the last paint presented.
    async fn capture_lcd(jvm: &Jvm, _context: &mut WieJvmContext, x: i32, y: i32, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.skt.m.Graphics2D::captureLCD({x}, {y}, {width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        let midlet: ClassInstanceRef<MIDlet> = jvm
            .get_static_field("javax/microedition/midlet/MIDlet", "currentMIDlet", "Ljavax/microedition/midlet/MIDlet;")
            .await?;
        if midlet.is_null() {
            return Ok(image);
        }
        let display = MIDlet::display(jvm, &midlet).await?;
        if display.is_null() {
            return Ok(image);
        }
        let mut screen_graphics: ClassInstanceRef<Graphics> = jvm
            .invoke_virtual(
                &display,
                "javax/microedition/lcdui/Display",
                "getScreenGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;
        let screen = Graphics::image(jvm, &mut screen_graphics).await?;
        let screen = Image::image(jvm, &screen).await?;

        let mut canvas = Image::canvas(jvm, &image).await?;
        let clip = Clip {
            x: 0,
            y: 0,
            width: width as _,
            height: height as _,
        };
        canvas.draw(0, 0, width as _, height as _, &*screen, x, y, clip);

        Ok(image)
    }

    #[allow(clippy::too_many_arguments)]
    async fn draw_image(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        tx: i32,
        ty: i32,
        src: ClassInstanceRef<Image>,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::drawImage({this:?}, {tx}, {ty}, {src:?}, {sx}, {sy}, {sw}, {sh}, {mode})");

        if src.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        if ![Self::DRAW_COPY, Self::DRAW_AND, Self::DRAW_OR, Self::DRAW_XOR].contains(&mode) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid draw mode").await);
        }

        if sw < 0 || sh < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must not be negative")
                .await);
        }

        if sw == 0 || sh == 0 {
            return Ok(());
        }

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let src_image = Image::image(jvm, &src).await?;

        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        let (dx, dy) = (tx.wrapping_add(translate_x), ty.wrapping_add(translate_y));

        // KEmulator: DRAW_AND / DRAW_OR read the target and source RGB, combine them per pixel
        // (`dst & src` / `dst | src`) and draw the result opaque. A transparent source pixel leaves the
        // target alone, as DRAW_XOR does here. Combining 8-bit channels is exact for every pixel type:
        // their 8-bit expansions are bit replications of the native fields.
        if mode == Self::DRAW_AND || mode == Self::DRAW_OR {
            for j in 0..sh {
                for i in 0..sw {
                    let (src_x, src_y) = (sx.wrapping_add(i), sy.wrapping_add(j));
                    if src_x < 0 || src_y < 0 || src_x as u32 >= src_image.width() || src_y as u32 >= src_image.height() {
                        continue;
                    }
                    let src_color = src_image.get_pixel(src_x, src_y);
                    let (x, y) = (dx.wrapping_add(i), dy.wrapping_add(j));
                    let Some(dst_color) = canvas.get_pixel(x, y) else {
                        continue;
                    };
                    if src_color.a == 0 {
                        continue;
                    }
                    let op = |d: u8, s: u8| if mode == Self::DRAW_AND { d & s } else { d | s };
                    let color = Color {
                        a: 0xff,
                        r: op(dst_color.r, src_color.r),
                        g: op(dst_color.g, src_color.g),
                        b: op(dst_color.b, src_color.b),
                    };
                    canvas.put_pixel(x, y, color, clip);
                }
            }
            return Ok(());
        }

        canvas.set_xor_mode(mode == Self::DRAW_XOR);

        canvas.draw(dx, dy, sw as _, sh as _, &*src_image, sx, sy, clip);

        Ok(())
    }

    async fn create_maskable_image(jvm: &Jvm, _context: &mut WieJvmContext, width: i32, height: i32) -> JvmResult<ClassInstanceRef<Image>> {
        tracing::debug!("com.skt.m.Graphics2D::createMaskableImage({width}, {height})");

        if width <= 0 || height <= 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must be positive")
                .await);
        }

        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (width, height),
            )
            .await?;

        Ok(image)
    }

    async fn get_pixel(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<i32> {
        tracing::debug!("com.skt.m.Graphics2D::getPixel({this:?}, {x}, {y})");

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let canvas = Image::canvas(jvm, &image).await?;

        Ok(canvas
            .get_pixel(x.wrapping_add(translate_x), y.wrapping_add(translate_y))
            .map(|color| ((color.r as i32) << 16) | ((color.g as i32) << 8) | color.b as i32)
            .unwrap_or(0))
    }

    async fn get_pixel_mask(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Graphics2D::getPixelMask({this:?}, {x}, {y})");
        Ok(false)
    }

    async fn invert_rect(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::invertRect({this:?}, {x}, {y}, {width}, {height})");

        if width < 0 || height < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must not be negative")
                .await);
        }

        if width == 0 || height == 0 {
            return Ok(());
        }

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        canvas.invert_rect(
            x.wrapping_add(translate_x),
            y.wrapping_add(translate_y),
            width as u32,
            height as u32,
            clip,
        );

        Ok(())
    }

    async fn set_pixel(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, color: i32) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Graphics2D::setPixel({this:?}, {x}, {y}, {color})");

        let mut graphics: ClassInstanceRef<Graphics> = jvm.get_field(&this, "graphics", "Ljavax/microedition/lcdui/Graphics;").await?;
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let image = Graphics::image(jvm, &mut graphics).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        canvas.put_pixel(
            x.wrapping_add(translate_x),
            y.wrapping_add(translate_y),
            Rgb8Pixel::to_color(color as u32),
            clip,
        );

        Ok(())
    }

    async fn set_pixel_mask(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, x: i32, y: i32, mask: bool) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.Graphics2D::setPixelMask({this:?}, {x}, {y}, {mask})");
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use alloc::vec;

    use jvm::{Array, ClassInstanceRef, JavaError, Jvm, Result as JvmResult};
    use jvm_class_proto::JavaMethodProto;
    use jvm_types::{ClassAccessFlags, MethodAccessFlags};
    use test_utils::{TestPlatform, run_jvm_test, run_jvm_test_with_system};
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

    use wie_midp::classes::javax::microedition::{
        lcdui::{Graphics, Image},
        midlet::MIDlet,
    };

    use crate::classes::com::{skt::m::SISImage, xce::lcdui::XDisplay};

    use super::Graphics2D;

    #[test]
    fn graphics_2d_copy_xor_clip_and_pixel_operations_use_the_target_canvas() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Graphics2D::as_proto()].into()]),
            |jvm| async move {
                let target: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (4, 1),
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
                let graphics_2d: ClassInstanceRef<Graphics2D> = jvm
                    .invoke_static(
                        "com/skt/m/Graphics2D",
                        "getGraphics2D",
                        "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                        (graphics.clone(),),
                    )
                    .await?;

                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (1, 0))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, 2, 1))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics_2d, "com/skt/m/Graphics2D", "setPixel", "(III)V", (0, 0, 0x123456))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics_2d, "com/skt/m/Graphics2D", "setPixel", "(III)V", (2, 0, 0xffffff))
                    .await?;
                let pixel: i32 = jvm
                    .invoke_virtual(&graphics_2d, "com/skt/m/Graphics2D", "getPixel", "(II)I", (0, 0))
                    .await?;
                assert_eq!(pixel, 0x123456);

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
                    .invoke_virtual(&source_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xf00faa,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&source_graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 1, 1))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&source_graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xabcdef,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&source_graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (1, 0, 1, 1))
                    .await?;

                let copy_mode: i32 = jvm.get_static_field("com/skt/m/Graphics2D", "DRAW_COPY", "I").await?;
                let xor_mode: i32 = jvm.get_static_field("com/skt/m/Graphics2D", "DRAW_XOR", "I").await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics_2d,
                        "com/skt/m/Graphics2D",
                        "drawImage",
                        "(IILjavax/microedition/lcdui/Image;IIIII)V",
                        (0, 0, source.clone(), 0, 0, 2, 1, copy_mode),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &graphics_2d,
                        "com/skt/m/Graphics2D",
                        "drawImage",
                        "(IILjavax/microedition/lcdui/Image;IIIII)V",
                        (0, 0, source, 0, 0, 1, 1, xor_mode),
                    )
                    .await?;

                let target_image = Image::image(&jvm, &target).await?;
                let untouched = target_image.get_pixel(0, 0);
                assert_eq!((untouched.r, untouched.g, untouched.b), (0, 0, 0));
                let xored = target_image.get_pixel(1, 0);
                assert_eq!((xored.r, xored.g, xored.b), (0, 0, 0));
                let copied = target_image.get_pixel(2, 0);
                assert_eq!((copied.r, copied.g, copied.b), (0xab, 0xcd, 0xef));
                let clipped = target_image.get_pixel(3, 0);
                assert_eq!((clipped.r, clipped.g, clipped.b), (0, 0, 0));

                let _: () = jvm
                    .invoke_virtual(&graphics_2d, "com/skt/m/Graphics2D", "invertRect", "(IIII)V", (0, 0, 4, 1))
                    .await?;
                let target_image = Image::image(&jvm, &target).await?;
                let inverted = target_image.get_pixel(1, 0);
                assert_eq!((inverted.r, inverted.g, inverted.b), (0xff, 0xff, 0xff));
                let outside_clip = target_image.get_pixel(3, 0);
                assert_eq!((outside_clip.r, outside_clip.g, outside_clip.b), (0, 0, 0));

                let _: () = jvm
                    .invoke_virtual(&graphics_2d, "com/skt/m/Graphics2D", "invertRect", "(IIII)V", (0, 0, 0, 1))
                    .await?;
                let negative_size: JvmResult<()> = jvm
                    .invoke_virtual(&graphics_2d, "com/skt/m/Graphics2D", "invertRect", "(IIII)V", (0, 0, -1, 1))
                    .await;
                let Err(JavaError::JavaException(exception)) = negative_size else {
                    panic!("Graphics2D.invertRect accepted a negative width");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                Ok(())
            },
        )
        .unwrap();
    }

    async fn image_with_pixels(jvm: &Jvm, colors: &[i32]) -> JvmResult<(ClassInstanceRef<Image>, ClassInstanceRef<Graphics>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (colors.len() as i32, 1),
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
        for (x, color) in colors.iter().enumerate() {
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (*color,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (x as i32, 0, 1, 1))
                .await?;
        }
        Ok((image, graphics))
    }

    // DRAW_AND / DRAW_OR were a warning and no pixels (c107462e5f8a d1dce4a36141 f12984cd0d37).
    #[test]
    fn graphics_2d_and_or_modes_combine_target_and_source_inside_the_clip() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [Graphics2D::as_proto()].into()]),
            |jvm| async move {
                let (source, _) = image_with_pixels(&jvm, &[0x0f0ff0, 0x0f0ff0, 0x0f0ff0]).await?;
                for (mode_name, expected) in [("DRAW_AND", 0x030330), ("DRAW_OR", 0x3f3ff3)] {
                    let (target, graphics) = image_with_pixels(&jvm, &[0x333333, 0x333333, 0x333333]).await?;
                    let _: () = jvm
                        .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, 2, 1))
                        .await?;
                    let graphics_2d: ClassInstanceRef<Graphics2D> = jvm
                        .invoke_static(
                            "com/skt/m/Graphics2D",
                            "getGraphics2D",
                            "(Ljavax/microedition/lcdui/Graphics;)Lcom/skt/m/Graphics2D;",
                            (graphics,),
                        )
                        .await?;
                    let mode: i32 = jvm.get_static_field("com/skt/m/Graphics2D", mode_name, "I").await?;
                    let _: () = jvm
                        .invoke_virtual(
                            &graphics_2d,
                            "com/skt/m/Graphics2D",
                            "drawImage",
                            "(IILjavax/microedition/lcdui/Image;IIIII)V",
                            (0, 0, source.clone(), 0, 0, 3, 1, mode),
                        )
                        .await?;

                    let target_image = Image::image(&jvm, &target).await?;
                    let rgb = |x| {
                        let c = target_image.get_pixel(x, 0);
                        ((c.r as i32) << 16) | ((c.g as i32) << 8) | c.b as i32
                    };
                    assert_eq!((rgb(0), rgb(1), rgb(2)), (expected, expected, 0x333333), "{mode_name}");
                }

                Ok(())
            },
        )
        .unwrap();
    }

    struct TestMIDlet;

    impl TestMIDlet {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "TestMIDlet",
                parent_class: Some("javax/microedition/midlet/MIDlet"),
                interfaces: vec![],
                methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC)],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/midlet/MIDlet", "<init>", "()V", ()).await
        }
    }

    // captureLCD returned a blank image (ec2f8f2e02a2 090877d7a3e0 7089dec0e8df).
    #[test]
    fn graphics_2d_capture_lcd_copies_the_screen_image() -> wie_util::Result<()> {
        run_jvm_test_with_system(
            Box::new([
                wie_midp::get_protos().into(),
                [Graphics2D::as_proto(), XDisplay::as_proto(), TestMIDlet::as_proto()].into(),
            ]),
            Box::new(TestPlatform::new()),
            |jvm, _system| async move {
                let midlet: ClassInstanceRef<MIDlet> = jvm.new_class("TestMIDlet", "()V", ()).await?.into();
                let display = MIDlet::display(&jvm, &midlet).await?;
                let screen: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(
                        &display,
                        "javax/microedition/lcdui/Display",
                        "getScreenGraphics",
                        "()Ljavax/microedition/lcdui/Graphics;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&screen, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x12ab34,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&screen, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (5, 7, 1, 1))
                    .await?;

                let captured: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "com/skt/m/Graphics2D",
                        "captureLCD",
                        "(IIII)Ljavax/microedition/lcdui/Image;",
                        (5, 7, 2, 2),
                    )
                    .await?;
                let captured = Image::image(&jvm, &captured).await?;
                assert_eq!((captured.width(), captured.height()), (2, 2));
                let hit = captured.get_pixel(0, 0);
                assert_eq!((hit.r, hit.g, hit.b), (0x12, 0xab, 0x34));
                let miss = captured.get_pixel(1, 1);
                assert_eq!((miss.r, miss.g, miss.b), (0, 0, 0));

                // XDisplay.copyLCD is captureLCD drawn into the image at (0, 0).
                let copy: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (3, 3),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_static(
                        "com/xce/lcdui/XDisplay",
                        "copyLCD",
                        "(Ljavax/microedition/lcdui/Graphics;Ljavax/microedition/lcdui/Image;IIII)V",
                        (screen, copy.clone(), 4, 6, 2, 2),
                    )
                    .await?;
                let copy = Image::image(&jvm, &copy).await?;
                let hit = copy.get_pixel(1, 1);
                assert_eq!((hit.r, hit.g, hit.b), (0x12, 0xab, 0x34));

                Ok(())
            },
        )
    }

    #[test]
    fn sis_image_placeholder_validates_ranges_and_paints_without_panicking() {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), [SISImage::as_proto()].into()]),
            |jvm| async move {
                let mut data: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 3).await?.into();
                jvm.store_array(&mut data, 0, [1i8, 2, 3]).await?;

                let _: () = jvm.invoke_static("com/skt/m/SISImage", "createBuffer", "(II)V", (1, 1)).await?;
                for sizes in [(0, 1), (1, 0)] {
                    let invalid_size: JvmResult<()> = jvm.invoke_static("com/skt/m/SISImage", "createBuffer", "(II)V", sizes).await;
                    let Err(JavaError::JavaException(exception)) = invalid_size else {
                        panic!("SISImage.createBuffer accepted a zero buffer size");
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                }

                let required: i32 = jvm
                    .invoke_static("com/skt/m/SISImage", "getRequiredBufferSize", "([BII)I", (data.clone(), 1, 2))
                    .await?;
                assert_eq!(required, 0);

                let sis_image: ClassInstanceRef<SISImage> = jvm
                    .invoke_static("com/skt/m/SISImage", "createSISImage", "([BII)Lcom/skt/m/SISImage;", (data.clone(), 0, 3))
                    .await?;
                let max_frame_id: i32 = jvm.invoke_virtual(&sis_image, "com/skt/m/SISImage", "getMaxFrameID", "()I", ()).await?;
                let max_object_id: i32 = jvm.invoke_virtual(&sis_image, "com/skt/m/SISImage", "getMaxObjectID", "()I", ()).await?;
                assert_eq!((max_frame_id, max_object_id), (0, 0));

                let frame: ClassInstanceRef<Image> = jvm
                    .invoke_virtual(&sis_image, "com/skt/m/SISImage", "getFrame", "(I)Ljavax/microedition/lcdui/Image;", (0,))
                    .await?;
                let object: ClassInstanceRef<Image> = jvm
                    .invoke_virtual(
                        &sis_image,
                        "com/skt/m/SISImage",
                        "getObject",
                        "(IZ)Ljavax/microedition/lcdui/Image;",
                        (0, false),
                    )
                    .await?;
                assert!(frame.is_null());
                assert!(object.is_null());

                let invalid_frame: JvmResult<ClassInstanceRef<Image>> = jvm
                    .invoke_virtual(&sis_image, "com/skt/m/SISImage", "getFrame", "(I)Ljavax/microedition/lcdui/Image;", (1,))
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_frame else {
                    panic!("SISImage.getFrame accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let invalid_object: JvmResult<ClassInstanceRef<Image>> = jvm
                    .invoke_virtual(
                        &sis_image,
                        "com/skt/m/SISImage",
                        "getObject",
                        "(IZ)Ljavax/microedition/lcdui/Image;",
                        (-1, false),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_object else {
                    panic!("SISImage.getObject accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let target: ClassInstanceRef<Image> = jvm
                    .invoke_static(
                        "javax/microedition/lcdui/Image",
                        "createImage",
                        "(II)Ljavax/microedition/lcdui/Image;",
                        (1, 1),
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
                let _: () = jvm
                    .invoke_virtual(
                        &sis_image,
                        "com/skt/m/SISImage",
                        "paintFrame",
                        "(Ljavax/microedition/lcdui/Graphics;III)V",
                        (graphics.clone(), 0, 0, 0),
                    )
                    .await?;

                let invalid_frame: JvmResult<()> = jvm
                    .invoke_virtual(
                        &sis_image,
                        "com/skt/m/SISImage",
                        "paintFrame",
                        "(Ljavax/microedition/lcdui/Graphics;III)V",
                        (graphics.clone(), 1, 0, 0),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_frame else {
                    panic!("SISImage.paintFrame accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let invalid_object: JvmResult<()> = jvm
                    .invoke_virtual(
                        &sis_image,
                        "com/skt/m/SISImage",
                        "paintObject",
                        "(Ljavax/microedition/lcdui/Graphics;IIIZ)V",
                        (graphics.clone(), 1, 0, 0, false),
                    )
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_object else {
                    panic!("SISImage.paintObject accepted an out-of-range ID");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                let _: () = jvm
                    .invoke_virtual(
                        &sis_image,
                        "com/skt/m/SISImage",
                        "paintObject",
                        "(Ljavax/microedition/lcdui/Graphics;IIIZ)V",
                        (graphics, 0, 0, 0, false),
                    )
                    .await?;

                let invalid_range: JvmResult<ClassInstanceRef<SISImage>> = jvm
                    .invoke_static("com/skt/m/SISImage", "createSISImage", "([BII)Lcom/skt/m/SISImage;", (data, 2, 2))
                    .await;
                let Err(JavaError::JavaException(exception)) = invalid_range else {
                    panic!("SISImage.createSISImage accepted an invalid byte range");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/ArrayIndexOutOfBoundsException"));

                let null_data = ClassInstanceRef::<Array<i8>>::new(None);
                let null_result: JvmResult<i32> = jvm
                    .invoke_static("com/skt/m/SISImage", "getRequiredBufferSize", "([BII)I", (null_data, 0, 0))
                    .await;
                let Err(JavaError::JavaException(exception)) = null_result else {
                    panic!("SISImage.getRequiredBufferSize accepted null data");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                Ok(())
            },
        )
        .unwrap();
    }
}
