use alloc::vec;

use alloc::vec::Vec;

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new("setString", "(Ljava/lang/String;)V", Self::set_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMaxLength", "(I)V", Self::set_max_length, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getString", "()Ljava/lang/String;", Self::get_string, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyNotify", "(II)Z", Self::key_notify, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("m_cPos", "I", FieldAccessFlags::PROTECTED),
                // Read directly by 레전드오브마스터 and 훼밀리마트타이쿤; the LGT linker reports it as
                // `TextComponent.iModeI`, which is where the descriptor comes from. Nothing in
                // wie writes it -- imHandler owns the mode -- so it stays at its default.
                JavaFieldProto::new("iMode", "I", FieldAccessFlags::PROTECTED),
                // Same story as iMode: read directly by 서든어택포켓 (`maxLength I`) and
                // 훼밀리마트타이쿤 (`m_td [C`). setMaxLength is a no-op stub, so maxLength is
                // not written here; m_td stays null until a key reaches keyNotify below.
                JavaFieldProto::new("maxLength", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("m_td", "[C", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("imHandler", "Lorg/kwis/msp/lcdui/InputMethodHandler;", FieldAccessFlags::PROTECTED),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<TextComponent>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.TextComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        // TODO constant. 0: CONSTRAINT_ANY
        let im_handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (0,)).await?;

        jvm.put_field(&mut this, "imHandler", "Lorg/kwis/msp/lcdui/InputMethodHandler;", im_handler)
            .await?;

        Ok(())
    }

    async fn set_max_length(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>, max_length: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextComponent::<init>({this:?}, {max_length})");

        Ok(())
    }

    async fn set_string(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>, data: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextComponent::setString({this:?}, {data:?})");

        Ok(())
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub org.kwis.msp.lwc.TextComponent::getString({this:?})");

        // Typed text wins; until something is typed the old "temp" answer stays, so titles that
        // never route a key here read exactly what they read before.
        let typed: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&this, "m_td", "[C").await?;
        if !typed.is_null() {
            return Ok(jvm.new_class("java/lang/String", "([C)V", (typed,)).await?.into());
        }
        let result = JavaLangString::from_rust_string(jvm, "temp").await?;

        Ok(result.into())
    }

    // Minimal input: a digit press appends it, CLR removes the last character — the keys a
    // ShellComponent hands the focused text widget. The text is kept in m_td, the canonical
    // buffer field (see its comment). No multi-tap letters, and no length cap: setMaxLength is
    // still a no-op. 1 = KEY_PRESSED, -16 = CLR as net.wie.CardCanvas sends them.
    async fn key_notify(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<TextComponent>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::keyNotify({this:?}, {type}, {key})");

        // Every key answers true, as Component's stub always did: 0c67145b11df calls this directly
        // before its form is shown, and the answer it reads there stays the same.
        let is_digit = (0x30..=0x39).contains(&key);
        if r#type != 1 || (!is_digit && key != -16) {
            return Ok(true);
        }

        let typed: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&this, "m_td", "[C").await?;
        let mut text: Vec<JavaChar> = if typed.is_null() {
            Vec::new()
        } else {
            let length = jvm.array_length(&typed).await?;
            jvm.load_array(&typed, 0, length).await?
        };
        if is_digit {
            text.push(key as JavaChar);
        } else {
            text.pop();
        }
        let mut buffer = jvm.instantiate_array("C", text.len()).await?;
        jvm.store_array(&mut buffer, 0, text).await?;
        jvm.put_field(&mut this, "m_td", "[C", buffer).await?;

        Ok(true)
    }
}
