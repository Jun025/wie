use alloc::{string::String as RustString, vec};

use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics};
use wie_util::keypad::{self, MULTITAP_MS, Mode};

const NAME: &str = "com/xce/lcdui/XTextField";
const DESCRIPTOR: &str = "Lcom/xce/lcdui/XTextField;";
const CLEAR: i32 = 8;
// javax.microedition.lcdui.TextField
const CONSTRAINT_MASK: i32 = 0xFFFF;
const NUMERIC: i32 = 2;
const PHONENUMBER: i32 = 3;
const DECIMAL: i32 = 5;

// class com.xce.lcdui.XTextField
//
// The measured callers (ec2f8f2e02a2 · 2f5246006bd8 · f2ae515201f2) forward every key to keyPressed
// and never repaint afterwards; ec2f8f2e02a2 copies getText() into its name only inside paint, and
// its OK refuses an empty name. So a key that changes the text repaints the field itself, or the
// title never sees it (a key that changes nothing does not). Digit keys are multitap upper-case Latin (wie_util::keypad, as SKVM's
// TextComponentHandler) — the same key within MULTITAP_MS cycles the last letter — and NUMERIC-like
// constraints take the digit. CLEAR deletes. keyReleased has nothing to do: a cycle ends by time
// or by another key.
// ponytail: upper-case Latin only, no Hangul/mode key — no measured title asks for one. `0` is a
// space (keypad's E.161 table, shared with TextComponentHandler); a title that must refuse a
// blank-only name checks that itself.
//
// 14a62a8521a0's name box never calls keyPressed here: its wrapper focuses the field and offers every
// key to TextComponentHandler.keyPressed instead, and never calls setTextComponent. So the field that
// holds focus is kept in `focused`, and the handler gives it a key when no TextComponent is registered.
// No other title in the corpus calls the handler's keyPressed on a field of this class (measured),
// so their keys still arrive here exactly once.
pub struct XTextField;

impl XTextField {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/XTextField",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                    Self::init,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getMaxSize", "()I", Self::get_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getText", "()Ljava/lang/String;", Self::get_text, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("hasFocus", "()Z", Self::has_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("inputChar", "(C)V", Self::input_char, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)V", Self::key_pressed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)V", Self::key_released, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyRepeated", "(I)V", Self::key_repeated, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setBounds", "(IIII)V", Self::set_bounds, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFocus", "(Z)V", Self::set_focus, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxSize", "(I)V", Self::set_max_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setText", "(Ljava/lang/String;)V", Self::set_text, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("constraints", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("focus", "Z", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("x", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("y", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("width", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("height", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("canvas", "Ljavax/microedition/lcdui/Canvas;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapKey", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapChar", "C", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapTime", "J", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("focused", "Lcom/xce/lcdui/XTextField;", FieldAccessFlags::STATIC),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<String>,
        max_size: i32,
        constraints: i32,
        canvas: ClassInstanceRef<Canvas>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::<init>({this:?}, {text:?}, {max_size}, {constraints}, {canvas:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        if text.is_null() || canvas.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "text and canvas must not be null").await);
        }
        if max_size < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must not be negative").await);
        }

        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "maxSize", "I", max_size).await?;
        jvm.put_field(&mut this, "constraints", "I", constraints).await?;
        jvm.put_field(&mut this, "canvas", "Ljavax/microedition/lcdui/Canvas;", canvas).await?;

        Ok(())
    }

    async fn truncate_text(jvm: &Jvm, text: ClassInstanceRef<String>, max_size: i32) -> JvmResult<ClassInstanceRef<String>> {
        let length: i32 = jvm.invoke_virtual(&text, "java/lang/String", "length", "()I", ()).await?;
        if length > max_size {
            jvm.invoke_virtual(&text, "java/lang/String", "substring", "(II)Ljava/lang/String;", (0, max_size))
                .await
        } else {
            Ok(text)
        }
    }

    async fn get_max_size(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "maxSize", "I").await
    }

    async fn get_text(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("com.xce.lcdui.XTextField::getText({this:?})");
        jvm.get_field(&this, "text", "Ljava/lang/String;").await
    }

    async fn has_focus(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "focus", "Z").await
    }

    async fn input_char(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: JavaChar) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::inputChar({this:?}, {key})");

        Self::append(jvm, &mut this, key).await?;
        Ok(())
    }

    /// Appends unless the field is full; ends any multitap cycle either way. Returns whether the text changed.
    async fn append(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, key: JavaChar) -> JvmResult<bool> {
        jvm.put_field(this, "tapKey", "I", 0).await?;
        let mut text = Self::text(jvm, this).await?;
        let max_size: i32 = jvm.get_field(this, "maxSize", "I").await?;
        if text.chars().count() as i32 >= max_size {
            return Ok(false);
        }
        text.push(char::from_u32(key as u32).unwrap_or(char::REPLACEMENT_CHARACTER));
        Self::put_text(jvm, this, &text).await?;
        Ok(true)
    }

    async fn set_focus(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, focus: bool) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setFocus({this:?}, {focus})");
        jvm.put_field(&mut this, "focus", "Z", focus).await?;

        if focus {
            jvm.put_static_field(NAME, "focused", DESCRIPTOR, this).await?;
        }
        Ok(())
    }

    /// The field that last took focus, or null. It may have lost focus since; take_key checks.
    pub async fn focused(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Self>> {
        jvm.get_static_field(NAME, "focused", DESCRIPTOR).await
    }

    async fn set_bounds(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setBounds({this:?}, {x}, {y}, {width}, {height})");

        if width < 0 || height < 0 {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "width and height must not be negative")
                .await);
        }

        jvm.put_field(&mut this, "x", "I", x).await?;
        jvm.put_field(&mut this, "y", "I", y).await?;
        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await
    }

    async fn key_pressed(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyPressed({this:?}, {key_code})");

        Self::take_key(jvm, context, this, key_code).await.map(|_| ())
    }

    /// keyPressed's work; returns whether the text changed (a changed field has repainted itself).
    pub async fn take_key(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<bool> {
        let focus: bool = jvm.get_field(&this, "focus", "Z").await?;
        if !focus {
            return Ok(false);
        }

        let changed = match key_code {
            0x30..=0x39 => Self::multitap(jvm, context, &mut this, key_code).await?,
            CLEAR => {
                jvm.put_field(&mut this, "tapKey", "I", 0).await?;
                let mut text = Self::text(jvm, &this).await?;
                let changed = text.pop().is_some();
                if changed {
                    Self::put_text(jvm, &mut this, &text).await?;
                }
                changed
            }
            32..=126 => Self::append(jvm, &mut this, key_code as JavaChar).await?,
            _ => false,
        };
        if changed {
            Self::repaint(jvm, context, this).await?;
        }
        Ok(changed)
    }

    /// Returns whether the text changed.
    async fn multitap(jvm: &Jvm, context: &mut WieJvmContext, this: &mut ClassInstanceRef<Self>, key_code: i32) -> JvmResult<bool> {
        let constraints: i32 = jvm.get_field(this, "constraints", "I").await?;
        let mode = if matches!(constraints & CONSTRAINT_MASK, NUMERIC | PHONENUMBER | DECIMAL) {
            Mode::Digit
        } else {
            Mode::Upper
        };
        let now = context.system().platform().now().raw() as i64;
        let tap_key: i32 = jvm.get_field(this, "tapKey", "I").await?;
        let tap_time: i64 = jvm.get_field(this, "tapTime", "J").await?;
        let again = tap_key == key_code && now - tap_time < MULTITAP_MS;
        let mut text = Self::text(jvm, this).await?;
        let max_size: i32 = jvm.get_field(this, "maxSize", "I").await?;

        let mut tokens = if again {
            vec![char::from_u32(jvm.get_field::<JavaChar>(this, "tapChar", "C").await? as u32).unwrap_or(' ')]
        } else {
            vec![]
        };
        let edit = keypad::press(mode, &mut tokens, (key_code - 0x30) as u8, again);
        // Judged on the length AFTER the edit: a letter cycle replaces in place, but a digit has
        // nothing to cycle and appends even when pressed again. A full field takes no new
        // character, and the refused key starts no cycle that would rewrite the last one.
        if (text.chars().count() - edit.delete + edit.insert.len()) as i32 > max_size {
            jvm.put_field(this, "tapKey", "I", 0).await?;
            return Ok(false);
        }
        for _ in 0..edit.delete {
            text.pop();
        }
        text.extend(edit.insert);
        Self::put_text(jvm, this, &text).await?;
        jvm.put_field(this, "tapKey", "I", key_code).await?;
        jvm.put_field(this, "tapChar", "C", tokens[0] as JavaChar).await?;
        jvm.put_field(this, "tapTime", "J", now).await?;
        Ok(true)
    }

    async fn text(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<RustString> {
        let text: ClassInstanceRef<String> = jvm.get_field(this, "text", "Ljava/lang/String;").await?;
        JavaLangString::to_rust_string(jvm, &text).await
    }

    async fn put_text(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: &str) -> JvmResult<()> {
        let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(jvm, text).await?.into();
        jvm.put_field(this, "text", "Ljava/lang/String;", text).await
    }

    async fn key_repeated(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyRepeated({this:?}, {key_code})");

        // A held digit would cycle its letters at the repeat rate.
        if (0x30..=0x39).contains(&key_code) {
            return Ok(());
        }
        Self::key_pressed(jvm, context, this, key_code).await
    }

    async fn key_released(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key_code: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::keyReleased({this:?}, {key_code})");

        Ok(())
    }

    async fn paint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::paint({this:?}, {graphics:?})");

        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let x: i32 = jvm.get_field(&this, "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "y", "I").await?;
        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;
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

        let _: () = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "clipRect",
                "(IIII)V",
                (x, y, width, height),
            )
            .await?;
        let draw_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "drawString",
                "(Ljava/lang/String;III)V",
                (text, x, y, 20),
            )
            .await;
        let restore_result: JvmResult<()> = jvm
            .invoke_virtual(
                &graphics,
                "javax/microedition/lcdui/Graphics",
                "setClip",
                "(IIII)V",
                (clip_x, clip_y, clip_width, clip_height),
            )
            .await;
        draw_result?;
        restore_result
    }

    async fn repaint(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::repaint({this:?})");

        let canvas: ClassInstanceRef<Canvas> = jvm.get_field(&this, "canvas", "Ljavax/microedition/lcdui/Canvas;").await?;
        let x: i32 = jvm.get_field(&this, "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "y", "I").await?;
        let width: i32 = jvm.get_field(&this, "width", "I").await?;
        let height: i32 = jvm.get_field(&this, "height", "I").await?;
        jvm.invoke_virtual(&canvas, "javax/microedition/lcdui/Canvas", "repaint", "(IIII)V", (x, y, width, height))
            .await
    }

    async fn set_max_size(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, max_size: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setMaxSize({this:?}, {max_size})");

        if max_size < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "maxSize must not be negative").await);
        }

        let text: ClassInstanceRef<String> = jvm.get_field(&this, "text", "Ljava/lang/String;").await?;
        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await?;
        jvm.put_field(&mut this, "maxSize", "I", max_size).await
    }

    async fn set_text(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, text: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.XTextField::setText({this:?}, {text:?})");

        if text.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "text is null").await);
        }

        let max_size: i32 = jvm.get_field(&this, "maxSize", "I").await?;
        let text = Self::truncate_text(jvm, text, max_size).await?;
        jvm.put_field(&mut this, "tapKey", "I", 0).await?;
        jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, JavaChar, JavaError, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_midp::classes::javax::microedition::lcdui::{Canvas, Graphics, Image};

    use super::{super::TextComponentHandler, XTextField};

    struct TrackingCanvas;
    struct TrackingGraphics;

    impl TrackingCanvas {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/TrackingCanvas",
                parent_class: Some("javax/microedition/lcdui/Canvas"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("repaint", "(IIII)V", Self::repaint, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("repaintCount", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("repaintHeight", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "javax/microedition/lcdui/Canvas", "<init>", "()V", ()).await
        }

        async fn repaint(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            x: i32,
            y: i32,
            width: i32,
            height: i32,
        ) -> JvmResult<()> {
            let count: i32 = jvm.get_field(&this, "repaintCount", "I").await?;
            jvm.put_field(&mut this, "repaintCount", "I", count + 1).await?;
            jvm.put_field(&mut this, "repaintX", "I", x).await?;
            jvm.put_field(&mut this, "repaintY", "I", y).await?;
            jvm.put_field(&mut this, "repaintWidth", "I", width).await?;
            jvm.put_field(&mut this, "repaintHeight", "I", height).await
        }

        async fn paint(
            _jvm: &Jvm,
            _context: &mut WieJvmContext,
            _this: ClassInstanceRef<Self>,
            _graphics: ClassInstanceRef<wie_midp::classes::javax::microedition::lcdui::Graphics>,
        ) -> JvmResult<()> {
            Ok(())
        }
    }

    impl TrackingGraphics {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "javax/microedition/lcdui/TrackingGraphics",
                parent_class: Some("javax/microedition/lcdui/Graphics"),
                interfaces: vec![],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("drawString", "(Ljava/lang/String;III)V", Self::draw_string, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("observedClipX", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("observedClipY", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("observedClipWidth", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("observedClipHeight", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let image: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createImage",
                    "(II)Ljavax/microedition/lcdui/Image;",
                    (20, 20),
                )
                .await?;
            jvm.invoke_special(
                &this,
                "javax/microedition/lcdui/Graphics",
                "<init>",
                "(Ljavax/microedition/lcdui/Image;)V",
                (image,),
            )
            .await
        }

        async fn draw_string(
            jvm: &Jvm,
            _context: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            _string: ClassInstanceRef<String>,
            _x: i32,
            _y: i32,
            _anchor: i32,
        ) -> JvmResult<()> {
            let clip_x: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                .await?;
            let clip_y: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                .await?;
            let clip_width: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                .await?;
            let clip_height: i32 = jvm
                .invoke_virtual(&this, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                .await?;
            jvm.put_field(&mut this, "observedClipX", "I", clip_x).await?;
            jvm.put_field(&mut this, "observedClipY", "I", clip_y).await?;
            jvm.put_field(&mut this, "observedClipWidth", "I", clip_width).await?;
            jvm.put_field(&mut this, "observedClipHeight", "I", clip_height).await
        }
    }

    #[test]
    fn text_state_input_limit_and_repaint_work_as_one_flow() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [XTextField::as_proto(), TrackingCanvas::as_proto(), TrackingGraphics::as_proto()].into(),
            ]),
            |jvm| async move {
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                let initial: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "abc").await?.into();
                let field: ClassInstanceRef<XTextField> = jvm
                    .new_class(
                        "com/xce/lcdui/XTextField",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (initial, 5, 0, canvas.clone()),
                    )
                    .await?
                    .into();

                let max_size: i32 = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "getMaxSize", "()I", ()).await?;
                let focused: bool = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "hasFocus", "()Z", ()).await?;
                assert_eq!(max_size, 5);
                assert!(!focused);

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyPressed", "(I)V", ('z' as i32,))
                    .await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abc");

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setFocus", "(Z)V", (true,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (3, 5, 40, 12))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyPressed", "(I)V", ('d' as i32,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "keyRepeated", "(I)V", ('e' as i32,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "inputChar", "(C)V", ('f' as JavaChar,))
                    .await?;

                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "abcde");
                assert!(
                    jvm.invoke_virtual::<_, bool>(&field, "com/xce/lcdui/XTextField", "hasFocus", "()Z", ())
                        .await?
                );

                for bounds in [(9, 8, -1, 4), (7, 6, 4, -1)] {
                    let bounds_result: JvmResult<()> = jvm
                        .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", bounds)
                        .await;
                    let Err(JavaError::JavaException(exception)) = bounds_result else {
                        panic!("XTextField.setBounds accepted a negative size");
                    };
                    assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));
                }

                let _: () = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "setMaxSize", "(I)V", (3,)).await?;
                let replacement: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "wxyz").await?.into();
                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (replacement,))
                    .await?;
                let text: ClassInstanceRef<String> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "getText", "()Ljava/lang/String;", ())
                    .await?;
                assert_eq!(JavaLangString::to_rust_string(&jvm, &text).await?, "wxy");

                // 'd' and 'e' each repainted the field; this is the third.
                let _: () = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "repaint", "()V", ()).await?;
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 3);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintX", "I").await?, 3);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintY", "I").await?, 5);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintWidth", "I").await?, 40);
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintHeight", "I").await?, 12);

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (1, 2, 0, 0))
                    .await?;

                let negative_result: JvmResult<()> = jvm.invoke_virtual(&field, "com/xce/lcdui/XTextField", "setMaxSize", "(I)V", (-1,)).await;
                let Err(JavaError::JavaException(exception)) = negative_result else {
                    panic!("XTextField.setMaxSize accepted a negative value");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

                let null_text = ClassInstanceRef::<String>::new(None);
                let null_result: JvmResult<()> = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setText", "(Ljava/lang/String;)V", (null_text,))
                    .await;
                let Err(JavaError::JavaException(exception)) = null_result else {
                    panic!("XTextField.setText accepted null");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    /// ec2f8f2e02a2's name screen: the title forwards each key to keyPressed, never repaints, reads
    /// getText() only in paint, and its OK refuses an empty name — so typing must change the text
    /// AND repaint, or the screen never moves on.
    #[test]
    fn digit_keys_type_multitap_and_repaint() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [XTextField::as_proto(), TrackingCanvas::as_proto(), TrackingGraphics::as_proto()].into(),
            ]),
            |jvm| async move {
                const X: &str = "com/xce/lcdui/XTextField";
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                async fn field(jvm: &Jvm, canvas: &ClassInstanceRef<Canvas>, max: i32, constraints: i32) -> JvmResult<ClassInstanceRef<XTextField>> {
                    let empty: ClassInstanceRef<String> = JavaLangString::from_rust_string(jvm, "").await?.into();
                    let field: ClassInstanceRef<XTextField> = jvm
                        .new_class(
                            X,
                            "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                            (empty, max, constraints, canvas.clone()),
                        )
                        .await?
                        .into();
                    let _: () = jvm.invoke_virtual(&field, X, "setFocus", "(Z)V", (true,)).await?;
                    Ok(field)
                }
                async fn press(jvm: &Jvm, field: &ClassInstanceRef<XTextField>, keys: &[i32]) -> JvmResult<alloc::string::String> {
                    for &key in keys {
                        let _: () = jvm.invoke_virtual(field, X, "keyPressed", "(I)V", (key,)).await?;
                        let _: () = jvm.invoke_virtual(field, X, "keyReleased", "(I)V", (key,)).await?;
                    }
                    let text: ClassInstanceRef<String> = jvm.invoke_virtual(field, X, "getText", "()Ljava/lang/String;", ()).await?;
                    JavaLangString::to_rust_string(jvm, &text).await
                }

                // ec2f8f2e02a2: `new XTextField("", 5, 0, this)`.
                let name = field(&jvm, &canvas, 5, 0).await?;
                // 2 → A, 2 again → B (cycled in place), 3 → D.
                assert_eq!(press(&jvm, &name, &[0x32, 0x32, 0x33]).await?, "BD");
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 3);
                // CLEAR deletes and ends the cycle: 3 again is a new letter.
                assert_eq!(press(&jvm, &name, &[8, 0x33]).await?, "BD");
                // A held digit does not cycle.
                let _: () = jvm.invoke_virtual(&name, X, "keyRepeated", "(I)V", (0x33,)).await?;
                assert_eq!(press(&jvm, &name, &[]).await?, "BD");
                // setText ends the cycle too; a full field takes nothing and rewrites nothing.
                let wxyz: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "WXYZ").await?.into();
                let _: () = jvm.invoke_virtual(&name, X, "setText", "(Ljava/lang/String;)V", (wxyz,)).await?;
                assert_eq!(press(&jvm, &name, &[0x33, 0x34, 0x34]).await?, "WXYZD");
                // Only keys that changed the text repainted: 2 2 3 · CLEAR · 3 · 3 (4 4 were refused).
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 6);

                // 2f5246006bd8's 9-digit field: `new XTextField("", 9, 2, canvas)` (NUMERIC).
                let number = field(&jvm, &canvas, 9, 2).await?;
                assert_eq!(press(&jvm, &number, &[0x30, 0x31, 0x31]).await?, "011");
                // A full NUMERIC field: the same digit again within the window must not append.
                let short = field(&jvm, &canvas, 3, 2).await?;
                assert_eq!(press(&jvm, &short, &[0x31, 0x32, 0x33, 0x33]).await?, "123");

                // inputChar ends a cycle: the 2 after it is a new letter, not a rewrite of 'X'.
                let direct = field(&jvm, &canvas, 5, 0).await?;
                let _: () = jvm.invoke_virtual(&direct, X, "keyPressed", "(I)V", (0x32,)).await?;
                let _: () = jvm.invoke_virtual(&direct, X, "inputChar", "(C)V", ('X' as JavaChar,)).await?;
                assert_eq!(press(&jvm, &direct, &[0x32]).await?, "AXA");
                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    /// 14a62a8521a0's name box: `new XTextField("", 64, 0, canvas)`, `setFocus(true)`, then every key
    /// goes only to `TextComponentHandler.keyPressed` (no setTextComponent). The focused field must
    /// take it; once focus leaves, the handler hands keys back again.
    #[test]
    fn handler_keys_reach_the_focused_field() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                crate::get_protos().into(),
                [TrackingCanvas::as_proto(), TrackingGraphics::as_proto()].into(),
            ]),
            |jvm| async move {
                const X: &str = "com/xce/lcdui/XTextField";
                const H: &str = "com/xce/lcdui/TextComponentHandler";
                let handler: ClassInstanceRef<TextComponentHandler> = jvm
                    .invoke_static(H, "getTextComponentHandler", "()Lcom/xce/lcdui/TextComponentHandler;", ())
                    .await?;
                async fn press(jvm: &Jvm, handler: &ClassInstanceRef<TextComponentHandler>, key: i32) -> JvmResult<bool> {
                    jvm.invoke_virtual(handler, H, "keyPressed", "(I)Z", (key,)).await
                }
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                let empty: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "").await?.into();
                let name: ClassInstanceRef<XTextField> = jvm
                    .new_class(
                        X,
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (empty, 64, 0, canvas.clone()),
                    )
                    .await?
                    .into();
                let text = async |jvm: &Jvm| -> JvmResult<alloc::string::String> {
                    let text: ClassInstanceRef<String> = jvm.invoke_virtual(&name, X, "getText", "()Ljava/lang/String;", ()).await?;
                    JavaLangString::to_rust_string(jvm, &text).await
                };

                // Not focused yet: nothing to type into.
                assert!(!press(&jvm, &handler, 0x35).await?);
                let _: () = jvm.invoke_virtual(&name, X, "setFocus", "(Z)V", (true,)).await?;
                // 5 5 → K, 2 → A, CLEAR, 6 → M. Each is consumed and repaints the canvas.
                for key in [0x35, 0x35, 0x32, 8, 0x36] {
                    assert!(press(&jvm, &handler, key).await?);
                }
                assert_eq!(text(&jvm).await?, "KM");
                assert_eq!(jvm.get_field::<i32>(&canvas, "repaintCount", "I").await?, 5);
                // A key the field does not take (UP) goes back to the title.
                assert!(!press(&jvm, &handler, 141).await?);

                // The title's mode-key path refocuses (setFocus false then true) — still the same field.
                let _: () = jvm.invoke_virtual(&name, X, "setFocus", "(Z)V", (false,)).await?;
                assert!(!press(&jvm, &handler, 0x32).await?);
                assert_eq!(text(&jvm).await?, "KM");
                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn paint_uses_the_bounds_clip_and_restores_the_original_clip() {
        let result = run_jvm_test(
            Box::new([
                wie_midp::get_protos().into(),
                [XTextField::as_proto(), TrackingCanvas::as_proto(), TrackingGraphics::as_proto()].into(),
            ]),
            |jvm| async move {
                let canvas: ClassInstanceRef<Canvas> = jvm.new_class("test/TrackingCanvas", "()V", ()).await?.into();
                let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "long text").await?.into();
                let field: ClassInstanceRef<XTextField> = jvm
                    .new_class(
                        "com/xce/lcdui/XTextField",
                        "(Ljava/lang/String;IILjavax/microedition/lcdui/Canvas;)V",
                        (text, 20, 0, canvas),
                    )
                    .await?
                    .into();
                let graphics: ClassInstanceRef<Graphics> = jvm.new_class("javax/microedition/lcdui/TrackingGraphics", "()V", ()).await?.into();

                let _: () = jvm
                    .invoke_virtual(&field, "com/xce/lcdui/XTextField", "setBounds", "(IIII)V", (3, 5, 4, 6))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", "setClip", "(IIII)V", (1, 2, 10, 11))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &field,
                        "com/xce/lcdui/XTextField",
                        "paint",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        (graphics.clone(),),
                    )
                    .await?;

                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipX", "I").await?, 3);
                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipY", "I").await?, 5);
                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipWidth", "I").await?, 4);
                assert_eq!(jvm.get_field::<i32>(&graphics, "observedClipHeight", "I").await?, 6);
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipX", "()I", ())
                        .await?,
                    1
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipY", "()I", ())
                        .await?,
                    2
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipWidth", "()I", ())
                        .await?,
                    10
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&graphics, "javax/microedition/lcdui/Graphics", "getClipHeight", "()I", ())
                        .await?,
                    11
                );

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
