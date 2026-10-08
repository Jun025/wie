use alloc::{boxed::Box, string::String as RustString, vec, vec::Vec};

use bytemuck::cast_vec;

use jvm::{Array, ClassInstanceRef, JavaChar, JavaValue, Jvm, Result as JvmResult, runtime::JavaLangString};

use jvm_class_proto::{JavaFieldProto, JavaMethodProto, TypeConverter};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_backend::canvas::{ArgbPixel, Canvas as BackendCanvas, Clip, PixelType, Rgb8Pixel, TextAlignment, VecImageBuffer};
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Font, Image};

bitflags::bitflags! {
    struct Anchor: i32 {
        const HCENTER = 1;
        const VCENTER = 2;
        const LEFT = 4;
        const RIGHT = 8;
        const TOP = 16;
        const BOTTOM = 32;
        const BASELINE = 64;
    }
}

impl TypeConverter<Anchor> for Anchor {
    fn to_rust(_: &Jvm, raw: JavaValue) -> Anchor {
        let raw: i32 = raw.into();
        Anchor::from_bits_retain(raw)
    }

    fn from_rust(_: &Jvm, rust: Anchor) -> JavaValue {
        rust.bits().into()
    }
}

impl From<Anchor> for TextAlignment {
    fn from(anchor: Anchor) -> Self {
        if anchor.contains(Anchor::HCENTER) {
            TextAlignment::Center
        } else if anchor.contains(Anchor::RIGHT) {
            TextAlignment::Right
        } else {
            TextAlignment::Left
        }
    }
}

// class javax.microedition.lcdui.Graphics
pub struct Graphics;

#[allow(clippy::too_many_arguments)]
impl Graphics {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Graphics",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/Image;)V",
                    Self::init_with_image,
                    MethodAccessFlags::empty(),
                ),
                JavaMethodProto::new("reset", "()V", Self::reset, MethodAccessFlags::empty()),
                // WIPI wrapper bridge only; this is not part of the MIDP Graphics API.
                JavaMethodProto::new("setXORMode", "(Z)V", Self::set_xor_mode, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("getFont", "()Ljavax/microedition/lcdui/Font;", Self::get_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setColor", "(I)V", Self::set_color, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setColor", "(III)V", Self::set_color_by_rgb, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFont", "(Ljavax/microedition/lcdui/Font;)V", Self::set_font, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillRect", "(IIII)V", Self::fill_rect, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillRoundRect", "(IIIIII)V", Self::fill_round_rect, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillArc", "(IIIIII)V", Self::fill_arc, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawLine", "(IIII)V", Self::draw_line, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawRect", "(IIII)V", Self::draw_rect, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawRoundRect", "(IIIIII)V", Self::draw_round_rect, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawArc", "(IIIIII)V", Self::draw_arc, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawChar", "(CIII)V", Self::draw_char, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawChars", "([CIIIII)V", Self::draw_chars, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "drawSubstring",
                    "(Ljava/lang/String;IIIII)V",
                    Self::draw_substring,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "drawImage",
                    "(Ljavax/microedition/lcdui/Image;III)V",
                    Self::draw_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "drawRegion",
                    "(Ljavax/microedition/lcdui/Image;IIIIIIII)V",
                    Self::draw_region,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setClip", "(IIII)V", Self::set_clip, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clipRect", "(IIII)V", Self::clip_rect, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getColor", "()I", Self::get_color, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getClipX", "()I", Self::get_clip_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getClipY", "()I", Self::get_clip_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getClipWidth", "()I", Self::get_clip_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getClipHeight", "()I", Self::get_clip_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTranslateX", "()I", Self::get_translate_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getTranslateY", "()I", Self::get_translate_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("translate", "(II)V", Self::translate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("drawRGB", "([IIIIIIIZ)V", Self::draw_rgb, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setGrayScale", "(I)V", Self::set_gray_scale, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillTriangle", "(IIIIII)V", Self::fill_triangle, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("copyArea", "(IIIIIII)V", Self::copy_area, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRedComponent", "()I", Self::get_red_component, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getGreenComponent", "()I", Self::get_green_component, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getBlueComponent", "()I", Self::get_blue_component, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getGrayScale", "()I", Self::get_gray_scale, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getDisplayColor", "(I)I", Self::get_display_color, MethodAccessFlags::PUBLIC),
                // ponytail: DOTTED is accepted and drawn solid. Dash the line plotters if a title's look depends on it.
                JavaMethodProto::new("setStrokeStyle", "(I)V", Self::set_stroke_style, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getStrokeStyle", "()I", Self::get_stroke_style, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("strokeStyle", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("img", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("clipX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("clipY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("clipWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("clipHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("translateX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("translateY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("color", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("xorMode", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("font", "Ljavax/microedition/lcdui/Font;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init_with_image(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::<init>({this:?}, {image:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let width: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getWidth", "()I", ())
            .await?;
        let height: i32 = jvm
            .invoke_virtual(&image, "javax/microedition/lcdui/Image", "getHeight", "()I", ())
            .await?;

        jvm.put_field(&mut this, "img", "Ljavax/microedition/lcdui/Image;", image).await?;

        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;

        let _: () = jvm.invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "reset", "()V", ()).await?;

        Ok(())
    }

    async fn reset(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::reset({this:?})");

        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;

        jvm.put_field(&mut this, "clipX", "I", 0).await?;
        jvm.put_field(&mut this, "clipY", "I", 0).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height).await?;
        jvm.put_field(&mut this, "translateX", "I", 0).await?;
        jvm.put_field(&mut this, "translateY", "I", 0).await?;
        jvm.put_field(&mut this, "color", "I", 0).await?;
        jvm.put_field(&mut this, "xorMode", "Z", false).await?;
        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;
        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", font).await?;

        Ok(())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getFont({this:?})");

        jvm.get_field(&this, "font", "Ljavax/microedition/lcdui/Font;").await
    }

    async fn set_color(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, rgb: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setColor({this:?}, {rgb})");

        jvm.put_field(&mut this, "color", "I", rgb).await?;

        Ok(())
    }

    async fn set_color_by_rgb(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, r: i32, g: i32, b: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setColor({this:?}, {r}, {g}, {b})");

        let rgb = (r << 16) | (g << 8) | b;

        jvm.put_field(&mut this, "color", "I", rgb).await?;

        Ok(())
    }

    async fn set_xor_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, xor_mode: bool) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setXORMode({this:?}, {xor_mode})");

        jvm.put_field(&mut this, "xorMode", "Z", xor_mode).await?;

        Ok(())
    }

    async fn set_font(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, font: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setFont({this:?}, {font:?})");

        let font = if font.is_null() {
            jvm.invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await?
        } else {
            font
        };
        jvm.put_field(&mut this, "font", "Ljavax/microedition/lcdui/Font;", font).await
    }

    async fn set_clip(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setClip({this:?}, {x}, {y}, {width}, {height})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        // clip fields hold absolute coordinates; negative w/h must clamp to 0 or `Self::clip()`'s
        // u32 cast produces a huge clip that copy_area's i64 extension treats as unbounded
        jvm.put_field(&mut this, "clipX", "I", x + translate_x).await?;
        jvm.put_field(&mut this, "clipY", "I", y + translate_y).await?;
        jvm.put_field(&mut this, "clipWidth", "I", width.max(0)).await?;
        jvm.put_field(&mut this, "clipHeight", "I", height.max(0)).await?;

        Ok(())
    }

    async fn clip_rect(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::clipRect({this:?}, {x}, {y}, {width}, {height})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let current_clip = Self::clip(jvm, &this).await?;
        let rect = Clip {
            x: x + translate_x,
            y: y + translate_y,
            width: width.max(0) as _,
            height: height.max(0) as _,
        };

        let new_clip = current_clip.intersect(&rect);

        jvm.put_field(&mut this, "clipX", "I", new_clip.x).await?;
        jvm.put_field(&mut this, "clipY", "I", new_clip.y).await?;
        jvm.put_field(&mut this, "clipWidth", "I", new_clip.width as i32).await?;
        jvm.put_field(&mut this, "clipHeight", "I", new_clip.height as i32).await?;

        Ok(())
    }

    async fn fill_round_rect(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        arc_width: i32,
        arc_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::fillRoundRect({this:?}, {x}, {y}, {width}, {height}, {arc_width}, {arc_height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        canvas.fill_round_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            arc_width as _,
            arc_height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn fill_arc(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_angle: i32,
        arc_angle: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::fillArc({this:?}, {x}, {y}, {width}, {height}, {start_angle}, {arc_angle})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        canvas.fill_arc(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            start_angle,
            arc_angle,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn fill_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::fillRect({this:?}, {x}, {y}, {width}, {height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        canvas.fill_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as _,
            height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn draw_rect(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32, width: i32, height: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawRect({this:?}, {x}, {y}, {width}, {height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        // MIDP drawRect outlines an area width+1 pixels wide and height+1 tall
        canvas.draw_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as u32 + 1,
            height as u32 + 1,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn draw_char(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        ch: JavaChar,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawChar({this:?}, {ch}, {x}, {y}, {})", anchor.0);

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let string = RustString::from_utf16(&[ch]).unwrap();

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            context.system().platform().font(),
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
        );

        Ok(())
    }

    async fn draw_chars(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        length: i32,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawChar({this:?}, {chars:?}, {offset}, {length}, {x}, {y})");

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let chars = jvm.load_array(&chars, offset as _, length as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            context.system().platform().font(),
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
        );

        Ok(())
    }

    async fn draw_string(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawString({this:?}, {string:?}, {x}, {y}, {})",
            anchor.0
        );

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            context.system().platform().font(),
            &string,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
        );

        Ok(())
    }
    async fn draw_substring(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawSubstring({this:?}, {string:?}, {offset}, {len}, {x}, {y})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_text(
            context.system().platform().font(),
            &substring,
            (translate_x + x) as _,
            (translate_y + y) as _,
            anchor.into(),
            Rgb8Pixel::to_color(color as _),
            clip,
        );

        Ok(())
    }

    async fn draw_line(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x1: i32, y1: i32, x2: i32, y2: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawLine({this:?}, {x1}, {y1}, {x2}, {y2})");

        let color: i32 = jvm.get_field(&this, "color", "I").await?;
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x1 = x1 + translate_x;
        let y1 = y1 + translate_y;
        let x2 = x2 + translate_x;
        let y2 = y2 + translate_y;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw_line(x1 as _, y1 as _, x2 as _, y2 as _, Rgb8Pixel::to_color(color as _), clip);

        Ok(())
    }

    async fn draw_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        img: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawImage({this:?}, {img:?}, {x}, {y}, {})", anchor.0);

        if img.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        let src_image = Image::image(jvm, &img).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let x_delta = if anchor.contains(Anchor::HCENTER) {
            -((src_image.width() / 2) as i32)
        } else if anchor.contains(Anchor::RIGHT) {
            -(src_image.width() as i32)
        } else {
            0
        };

        let y_delta = if anchor.contains(Anchor::VCENTER) {
            -((src_image.height() / 2) as i32)
        } else if anchor.contains(Anchor::BOTTOM) {
            -(src_image.height() as i32)
        } else {
            0
        };

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x = translate_x + x + x_delta;
        let y = translate_y + y + y_delta;

        let clip = Self::clip(jvm, &this).await?;

        canvas.draw(x as _, y as _, src_image.width(), src_image.height(), &*src_image, 0, 0, clip);

        Ok(())
    }

    async fn draw_region(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        img: ClassInstanceRef<Image>,
        src_x: i32,
        src_y: i32,
        width: i32,
        height: i32,
        transform: i32,
        x: i32,
        y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawRegion({this:?}, {img:?}, {src_x}, {src_y}, {width}, {height}, {transform}, {x}, {y}, {})",
            anchor.0
        );

        if img.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "img is null").await);
        }

        let src_image = Image::image(jvm, &img).await?;

        // TRANS_ROT90/ROT270/MIRROR_ROT90/MIRROR_ROT270 swap the destination's width and height;
        // the anchor applies to the transformed region (JSR-118 Graphics.drawRegion).
        let (dst_width, dst_height) = if matches!(transform, 4..=7) { (height, width) } else { (width, height) };

        let x_delta = if anchor.contains(Anchor::HCENTER) {
            -dst_width / 2
        } else if anchor.contains(Anchor::RIGHT) {
            -dst_width
        } else {
            0
        };

        let y_delta = if anchor.contains(Anchor::VCENTER) {
            -dst_height / 2
        } else if anchor.contains(Anchor::BOTTOM) {
            -dst_height
        } else {
            0
        };

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x = translate_x + x + x_delta;
        let y = translate_y + y + y_delta;

        let clip = Self::clip(jvm, &this).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;
        draw_transformed(&mut *canvas, &*src_image, (src_x, src_y, width, height), transform, x, y, clip);

        Ok(())
    }

    async fn draw_round_rect(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        arc_width: i32,
        arc_height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawRoundRect({this:?}, {x}, {y}, {width}, {height}, {arc_width}, {arc_height})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        // MIDP drawRoundRect outlines an area width+1 pixels wide and height+1 tall
        canvas.draw_round_rect(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as u32 + 1,
            height as u32 + 1,
            arc_width as _,
            arc_height as _,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn draw_arc(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        start_angle: i32,
        arc_angle: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::drawArc({this:?}, {x}, {y}, {width}, {height}, {start_angle}, {arc_angle})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let clip = Self::clip(jvm, &this).await?;

        if width < 0 || height < 0 {
            return Ok(());
        }

        // MIDP drawArc covers an area width+1 pixels wide and height+1 tall
        canvas.draw_arc(
            (translate_x + x) as _,
            (translate_y + y) as _,
            width as u32 + 1,
            height as u32 + 1,
            start_angle,
            arc_angle,
            Rgb8Pixel::to_color(rgb as _),
            clip,
        );

        Ok(())
    }

    async fn get_color(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getColor({this:?})");

        let color: i32 = jvm.get_field(&this, "color", "I").await?;

        Ok(color)
    }

    async fn get_clip_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipX({this:?})");

        let clip_x: i32 = jvm.get_field(&this, "clipX", "I").await?;
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;

        Ok(clip_x - translate_x)
    }

    async fn get_clip_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipY({this:?})");

        let clip_y: i32 = jvm.get_field(&this, "clipY", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        Ok(clip_y - translate_y)
    }

    async fn get_clip_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipWidth({this:?})");

        let clip_width: i32 = jvm.get_field(&this, "clipWidth", "I").await?;

        Ok(clip_width)
    }

    async fn get_clip_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getClipHeight({this:?})");

        let clip_height: i32 = jvm.get_field(&this, "clipHeight", "I").await?;

        Ok(clip_height)
    }

    async fn get_translate_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getTranslateX({this:?})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;

        Ok(translate_x)
    }

    async fn get_translate_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Graphics>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Graphics::getTranslateY({this:?})");

        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        Ok(translate_y)
    }

    async fn translate(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Graphics>, x: i32, y: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::translate({this:?}, {x}, {y})");

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        jvm.put_field(&mut this, "translateX", "I", translate_x + x).await?;
        jvm.put_field(&mut this, "translateY", "I", translate_y + y).await?;

        Ok(())
    }

    async fn draw_rgb(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Graphics>,
        rgb_data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        scan_length: i32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        process_alpha: bool,
    ) -> JvmResult<()> {
        tracing::debug!(
            "javax.microedition.lcdui.Graphics::drawRGB({this:?}, {rgb_data:?}, {offset}, {scan_length}, {x}, {y}, {width}, {height}, {process_alpha})"
        );

        // TODO proper scanlength support
        let pixel_data: Vec<i32> = jvm.load_array(&rgb_data, offset as _, (width * height) as _).await?;

        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;

        let x = translate_x + x;
        let y = translate_y + y;

        let clip = Self::clip(jvm, &this).await?;

        if process_alpha {
            let src_image = VecImageBuffer::<ArgbPixel>::from_raw(width as _, height as _, cast_vec(pixel_data));
            canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0, clip);
        } else {
            let src_image = VecImageBuffer::<Rgb8Pixel>::from_raw(width as _, height as _, cast_vec(pixel_data));
            canvas.draw(x as _, y as _, width as _, height as _, &src_image, 0, 0, clip);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn fill_triangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::fillTriangle({this:?}, {x1}, {y1}, {x2}, {y2}, {x3}, {y3})");

        let rgb: i32 = jvm.get_field(&this, "color", "I").await?;
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;
        let clip = Self::clip(jvm, &this).await?;
        let mut canvas = Self::canvas(jvm, &mut this).await?;

        let color = Rgb8Pixel::to_color(rgb as _);
        for (y, left, right) in triangle_spans((x1, y1), (x2, y2), (x3, y3), clip.y, clip.y.saturating_add(clip.height as i32)) {
            canvas.fill_rect(translate_x + left, translate_y + y, (right - left + 1) as u32, 1, color, clip);
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn copy_area(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        dest_x: i32,
        dest_y: i32,
        anchor: Anchor,
    ) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::copyArea({this:?}, {x}, {y}, {width}, {height}, {dest_x}, {dest_y})");

        if width <= 0 || height <= 0 {
            return Ok(());
        }
        let dx = if anchor.contains(Anchor::HCENTER) {
            -width / 2
        } else if anchor.contains(Anchor::RIGHT) {
            -width
        } else {
            0
        };
        let dy = if anchor.contains(Anchor::VCENTER) {
            -height / 2
        } else if anchor.contains(Anchor::BOTTOM) {
            -height
        } else {
            0
        };
        let translate_x: i32 = jvm.get_field(&this, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&this, "translateY", "I").await?;
        let clip = Self::clip(jvm, &this).await?;
        let mut canvas = Self::canvas(jvm, &mut this).await?;
        canvas.copy_area(
            translate_x + dest_x + dx,
            translate_y + dest_y + dy,
            translate_x + x,
            translate_y + y,
            width as u32,
            height as u32,
            clip,
        );

        Ok(())
    }

    async fn get_red_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok((jvm.get_field::<i32>(&this, "color", "I").await? >> 16) & 0xff)
    }

    async fn get_green_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok((jvm.get_field::<i32>(&this, "color", "I").await? >> 8) & 0xff)
    }

    async fn get_blue_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(jvm.get_field::<i32>(&this, "color", "I").await? & 0xff)
    }

    // the luminance of the current colour, as the RI computes it
    async fn get_gray_scale(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let c: i32 = jvm.get_field(&this, "color", "I").await?;

        Ok((((c >> 16) & 0xff) * 76 + ((c >> 8) & 0xff) * 150 + (c & 0xff) * 29) >> 8)
    }

    // a 24-bit display shows every colour as asked
    async fn get_display_color(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, color: i32) -> JvmResult<i32> {
        Ok(color & 0xffffff)
    }

    async fn set_stroke_style(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, style: i32) -> JvmResult<()> {
        if style != 0 && style != 1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid stroke style").await);
        }
        jvm.put_field(&mut this, "strokeStyle", "I", style).await
    }

    async fn get_stroke_style(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "strokeStyle", "I").await
    }

    async fn set_gray_scale(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, value: i32) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Graphics::setGrayScale({this:?}, {value})");

        let color = (value << 16) | (value << 8) | value;

        jvm.put_field(&mut this, "color", "I", color).await?;

        Ok(())
    }

    pub(crate) async fn canvas(jvm: &Jvm, this: &mut ClassInstanceRef<Graphics>) -> JvmResult<Box<dyn BackendCanvas>> {
        let image = Self::image(jvm, this).await?;
        let mut canvas = Image::canvas(jvm, &image).await?;
        let xor_mode: bool = jvm.get_field(this, "xorMode", "Z").await?;

        canvas.set_xor_mode(xor_mode);

        Ok(canvas)
    }

    pub async fn image(jvm: &Jvm, this: &mut ClassInstanceRef<Graphics>) -> JvmResult<ClassInstanceRef<Image>> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "img", "Ljavax/microedition/lcdui/Image;").await?;

        if !image.is_null() {
            Ok(image)
        } else {
            let width = jvm.get_field(this, "width", "I").await?;
            let height = jvm.get_field(this, "height", "I").await?;

            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    [width, height],
                )
                .await?;

            jvm.put_field(this, "img", "Ljavax/microedition/lcdui/Image;", image.clone()).await?;

            Ok(image)
        }
    }

    pub async fn clip(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Clip> {
        let x: i32 = jvm.get_field(this, "clipX", "I").await?;
        let y: i32 = jvm.get_field(this, "clipY", "I").await?;
        let width: i32 = jvm.get_field(this, "clipWidth", "I").await?;
        let height: i32 = jvm.get_field(this, "clipHeight", "I").await?;

        Ok(Clip {
            x: x as _,
            y: y as _,
            width: width as _,
            height: height as _,
        })
    }
}

// Horizontal spans (y, left, right — inclusive) covering the triangle, for rows in [y_from, y_to).
// Each row takes the pixel centres inside the edges, so two triangles sharing an edge neither gap
// nor overlap along it; a degenerate triangle still covers its line.
fn triangle_spans(a: (i32, i32), b: (i32, i32), c: (i32, i32), y_from: i32, y_to: i32) -> Vec<(i32, i32, i32)> {
    let top = a.1.min(b.1).min(c.1).max(y_from);
    let bottom = a.1.max(b.1).max(c.1).min(y_to - 1);
    let edges = [(a, b), (b, c), (c, a)];
    let mut spans = Vec::new();
    for y in top..=bottom {
        let (mut left, mut right) = (f64::MAX, f64::MIN);
        for ((x0, y0), (x1, y1)) in edges {
            let (lo, hi) = (y0.min(y1), y0.max(y1));
            if y < lo || y > hi {
                continue;
            }
            if y0 == y1 {
                left = left.min(x0.min(x1) as f64);
                right = right.max(x0.max(x1) as f64);
            } else {
                let x = x0 as f64 + (y - y0) as f64 * (x1 - x0) as f64 / (y1 - y0) as f64;
                left = left.min(x);
                right = right.max(x);
            }
        }
        if left <= right {
            spans.push((y, round(left), round(right)));
        }
    }

    spans
}

fn round(x: f64) -> i32 {
    // no_std: f64::round is not in core
    if x >= 0.0 { (x + 0.5) as i32 } else { (x - 0.5) as i32 }
}

// Where pixel (x, y) of a w×h region drawn with `transform` comes from, in the untransformed region.
pub(crate) fn region_source_point(transform: i32, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    match transform {
        1 => (x, h - 1 - y),         // TRANS_MIRROR_ROT180
        2 => (w - 1 - x, y),         // TRANS_MIRROR
        3 => (w - 1 - x, h - 1 - y), // TRANS_ROT180
        4 => (y, x),                 // TRANS_MIRROR_ROT270
        5 => (y, h - 1 - x),         // TRANS_ROT90
        6 => (w - 1 - y, x),         // TRANS_ROT270
        7 => (w - 1 - y, h - 1 - x), // TRANS_MIRROR_ROT90
        _ => (x, y),
    }
}

// Draws the (sx, sy, w, h) region of `src` with `transform`, its transformed top-left at canvas (x, y).
pub(crate) fn draw_transformed(
    canvas: &mut dyn BackendCanvas,
    src: &dyn wie_backend::canvas::Image,
    (sx, sy, w, h): (i32, i32, i32, i32),
    transform: i32,
    x: i32,
    y: i32,
    clip: Clip,
) {
    if transform == 0 {
        canvas.draw(x, y, w.max(0) as _, h.max(0) as _, src, sx, sy, clip);
    } else if w > 0 && h > 0 {
        let region = transformed_region(src, sx, sy, w, h, transform);
        let (out_w, out_h) = if matches!(transform, 4..=7) { (h, w) } else { (w, h) };
        canvas.draw(x, y, out_w as _, out_h as _, &region, 0, 0, clip);
    }
}

// Copies the (sx, sy, w, h) region of `src` into a new buffer laid out as `transform` asks
// (javax.microedition.lcdui.game.Sprite TRANS_* values). Pixels outside `src` stay transparent.
pub(crate) fn transformed_region(
    src: &dyn wie_backend::canvas::Image,
    sx: i32,
    sy: i32,
    w: i32,
    h: i32,
    transform: i32,
) -> VecImageBuffer<ArgbPixel> {
    use wie_backend::canvas::ImageBuffer;

    let (out_w, out_h) = if matches!(transform, 4..=7) { (h, w) } else { (w, h) };
    let mut out = VecImageBuffer::<ArgbPixel>::new(out_w as _, out_h as _);
    for y in 0..out_h {
        for x in 0..out_w {
            let (rx, ry) = region_source_point(transform, x, y, w, h);
            let (px, py) = (sx + rx, sy + ry);
            if px >= 0 && py >= 0 && (px as u32) < src.width() && (py as u32) < src.height() {
                out.put_pixel(x, y, src.get_pixel(px, py));
            }
        }
    }

    out
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstance, ClassInstanceRef, Jvm, Result as JvmResult};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::javax::microedition::lcdui::Image, get_protos};

    #[test]
    fn triangle_spans_cover_rows_within_the_clip() {
        use super::triangle_spans;

        let spans = triangle_spans((0, 0), (4, 0), (0, 4), i32::MIN, i32::MAX);
        assert_eq!(spans, [(0, 0, 4), (1, 0, 3), (2, 0, 2), (3, 0, 1), (4, 0, 0)]);
        assert_eq!(triangle_spans((0, 0), (4, 0), (0, 4), 1, 3), [(1, 0, 3), (2, 0, 2)]);
        assert_eq!(triangle_spans((0, 2), (5, 2), (9, 2), i32::MIN, i32::MAX), [(2, 0, 9)]); // degenerate: a line
    }

    #[test]
    fn test_graphics() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (100, 100),
                )
                .await?;

            let graphics = jvm
                .new_class(
                    "javax/microedition/lcdui/Graphics",
                    "(Ljavax/microedition/lcdui/Image;)V",
                    (image.clone(),),
                )
                .await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x00ff00,))
                .await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 100, 100))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;

            assert_eq!(backend_image.width(), 100);
            assert_eq!(backend_image.height(), 100);

            assert_eq!(backend_image.colors()[0].r, 0x00);
            assert_eq!(backend_image.colors()[0].g, 0xff);
            assert_eq!(backend_image.colors()[0].b, 0x00);

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 1, 1))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xf00faa,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setXORMode", "(Z)V", (true,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 1, 1))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(0, 0);
            assert_eq!(color.r, 0x12 ^ 0xf0);
            assert_eq!(color.g, 0x34 ^ 0x0f);
            assert_eq!(color.b, 0x56 ^ 0xaa);

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setXORMode", "(Z)V", (false,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0x123456,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 1, 1))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setXORMode", "(Z)V", (true,))
                .await?;

            let mut rgb_data = jvm.instantiate_array("I", 1).await?;
            jvm.store_array(&mut rgb_data, 0, vec![0x00ff0000i32]).await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawRGB",
                    "([IIIIIIIZ)V",
                    (rgb_data, 0, 1, 0, 0, 1, 1, true),
                )
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(0, 0);
            assert_eq!(color.r, 0x12);
            assert_eq!(color.g, 0x34);
            assert_eq!(color.b, 0x56);

            Ok(())
        })
    }

    async fn new_graphics(jvm: &Jvm) -> JvmResult<(ClassInstanceRef<Image>, Box<dyn ClassInstance>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (100, 100),
            )
            .await?;

        let graphics = jvm
            .new_class(
                "javax/microedition/lcdui/Graphics",
                "(Ljavax/microedition/lcdui/Image;)V",
                (image.clone(),),
            )
            .await?;

        Ok((image, graphics))
    }

    #[test]
    fn test_clip_follows_translate() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (10, 10))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, 5, 5))
                .await?;

            let clip_x: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                .await?;
            let clip_y: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                .await?;
            assert_eq!(clip_x, 0);
            assert_eq!(clip_y, 0);

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (-10, -10))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xff0000,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 100, 100))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for (x, y) in [(10, 10), (14, 14)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00));
            }
            for (x, y) in [(9, 9), (15, 15)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_clip_rect_intersects_in_translated_coords() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, 20, 20))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (10, 10))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "clipRect", "(IIII)V", (0, 0, 5, 5))
                .await?;

            let clip_x: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                .await?;
            let clip_y: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                .await?;
            let clip_width: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                .await?;
            let clip_height: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                .await?;
            assert_eq!(clip_x, 0);
            assert_eq!(clip_y, 0);
            assert_eq!(clip_width, 5);
            assert_eq!(clip_height, 5);

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (-10, -10))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xff0000,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 100, 100))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(10, 10);
            assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00));
            for (x, y) in [(9, 9), (15, 15)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_empty_clip_draws_nothing() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, 5, 5))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "clipRect", "(IIII)V", (20, 20, 5, 5))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xff0000,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 100, 100))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for (x, y) in [(0, 0), (2, 2), (21, 21)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_negative_clip_is_empty() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (0, 0, -5, -5))
                .await?;

            let clip_width: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                .await?;
            let clip_height: i32 = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                .await?;
            assert_eq!(clip_width, 0);
            assert_eq!(clip_height, 0);

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xff0000,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "fillRect", "(IIII)V", (0, 0, 100, 100))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for (x, y) in [(0, 0), (50, 50)] {
                let color = backend_image.get_pixel(x, y);
                assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            }

            Ok(())
        })
    }

    #[test]
    fn test_draw_rect_is_inclusive() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setColor", "(I)V", (0xff0000,))
                .await?;
            // MIDP: drawRect(2, 2, 0, 3) draws a 1px-wide, 4px-tall vertical line
            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "drawRect", "(IIII)V", (2, 2, 0, 3))
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            for y in 2..=5 {
                let color = backend_image.get_pixel(2, y);
                assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00), "y={y}");
            }
            let color = backend_image.get_pixel(2, 6);
            assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));
            let color = backend_image.get_pixel(3, 2);
            assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));

            Ok(())
        })
    }

    #[test]
    fn test_draw_rgb_follows_translate() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (image, graphics) = new_graphics(&jvm).await?;

            let _: () = jvm
                .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "translate", "(II)V", (10, 10))
                .await?;

            let mut rgb_data = jvm.instantiate_array("I", 1).await?;
            jvm.store_array(&mut rgb_data, 0, vec![0x00ff0000i32]).await?;
            let _: () = jvm
                .invoke_virtual(
                    &graphics,
                    "javax/microedition/lcdui/Graphics",
                    "drawRGB",
                    "([IIIIIIIZ)V",
                    (rgb_data, 0, 1, 0, 0, 1, 1, false),
                )
                .await?;

            let backend_image = Image::image(&jvm, &image).await?;
            let color = backend_image.get_pixel(10, 10);
            assert_eq!((color.r, color.g, color.b), (0xff, 0x00, 0x00));
            let color = backend_image.get_pixel(0, 0);
            assert_eq!((color.r, color.g, color.b), (0x00, 0x00, 0x00));

            Ok(())
        })
    }
}
