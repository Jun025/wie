use alloc::{string::String as RustString, vec};

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_backend::canvas;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.lcdui.Font
pub struct Font;

impl Font {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Font",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, Default::default()),
                JavaMethodProto::new("getBaselinePosition", "()I", Self::get_baseline_position, Default::default()),
                JavaMethodProto::new("stringWidth", "(Ljava/lang/String;)I", Self::string_width, Default::default()),
                JavaMethodProto::new("substringWidth", "(Ljava/lang/String;II)I", Self::substring_width, Default::default()),
                JavaMethodProto::new("charWidth", "(C)I", Self::char_width, Default::default()),
                JavaMethodProto::new("charsWidth", "([CII)I", Self::chars_width, Default::default()),
                JavaMethodProto::new(
                    "getFont",
                    "(III)Ljavax/microedition/lcdui/Font;",
                    Self::get_font,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "getDefaultFont",
                    "()Ljavax/microedition/lcdui/Font;",
                    Self::get_default_font,
                    MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("FACE_SYSTEM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_MONOSPACE", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FACE_PROPORTIONAL", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_PLAIN", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_BOLD", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_ITALIC", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("STYLE_UNDERLINED", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_SMALL", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_MEDIUM", "I", FieldAccessFlags::STATIC),
                JavaFieldProto::new("SIZE_LARGE", "I", FieldAccessFlags::STATIC),
            ],
            access_flags: Default::default(),
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("javax.microedition.lcdui.Font::<clinit>");

        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_SYSTEM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_MONOSPACE", "I", 32).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "FACE_PROPORTIONAL", "I", 64)
            .await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_PLAIN", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_BOLD", "I", 1).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_ITALIC", "I", 2).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "STYLE_UNDERLINED", "I", 4).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_MEDIUM", "I", 0).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_SMALL", "I", 8).await?;
        jvm.put_static_field("javax/microedition/lcdui/Font", "SIZE_LARGE", "I", 16).await?;

        Ok(())
    }

    async fn init(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Font>) -> JvmResult<()> {
        tracing::warn!("stub javax.microedition.lcdui.Font::<init>({this:?})");

        Ok(())
    }

    async fn get_height(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getHeight");

        Ok(12) // TODO: hardcoded
    }

    // The stub font has no real ascent metric; baseline is derived from getHeight as
    // height * 4 / 5 (floored). MIDP baseline ≈ ascent, which is most of the font
    // height minus descent, so 4/5 is a reasoned approximation (e.g. height 12 → 9).
    async fn get_baseline_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("javax.microedition.lcdui.Font::getBaselinePosition");

        let height: i32 = jvm.invoke_virtual(&this, "getHeight", "()I", ()).await?;

        Ok(height * 4 / 5)
    }

    async fn get_default_font(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getDefaultFont");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn get_font(jvm: &Jvm, _: &mut WieJvmContext, face: i32, style: i32, size: i32) -> JvmResult<ClassInstanceRef<Font>> {
        tracing::warn!("stub javax.microedition.lcdui.Font::getFont({face:?}, {style:?}, {size:?})");

        let instance = jvm.new_class("javax/microedition/lcdui/Font", "()V", []).await?;

        Ok(instance.into())
    }

    async fn string_width(jvm: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, string: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::stringWidth({string:?})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;

        Ok(canvas::string_width(&string, 10.0) as _)
    }

    async fn substring_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        _: ClassInstanceRef<Self>,
        string: ClassInstanceRef<String>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::substringWidth({string:?}, {offset:?}, {len:?})");

        let string = JavaLangString::to_rust_string(jvm, &string).await?;
        let substring = string.chars().skip(offset as usize).take(len as usize).collect::<RustString>();

        Ok(canvas::string_width(&substring, 10.0) as _)
    }

    async fn char_width(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>, char: JavaChar) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::charWidth({char:?})");

        let string = RustString::from_utf16(&[char]).unwrap();

        Ok(canvas::string_width(&string, 10.0) as _)
    }

    async fn chars_width(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        _: ClassInstanceRef<Self>,
        chars: ClassInstanceRef<Array<JavaChar>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub javax.microedition.lcdui.Font::charsWidth({chars:?}, {offset:?}, {len:?})");

        let chars = jvm.load_array(&chars, offset as _, len as _).await?;
        let string = RustString::from_utf16(&chars).unwrap();

        Ok(canvas::string_width(&string, 10.0) as _)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::javax::microedition::lcdui::Font, get_protos};

    #[test]
    fn test_baseline_position() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let font: ClassInstanceRef<Font> = jvm
                .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
                .await?;

            let height: i32 = jvm.invoke_virtual(&font, "getHeight", "()I", ()).await?;
            let baseline: i32 = jvm.invoke_virtual(&font, "getBaselinePosition", "()I", ()).await?;

            // baseline = floor(height * 4 / 5), strictly inside the font box
            assert_eq!(baseline, height * 4 / 5);
            assert_eq!(baseline, 9); // stub height 12 → 9
            assert!(baseline < height);

            Ok(())
        })
    }
}
