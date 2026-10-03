use alloc::vec;

use alloc::vec::Vec;

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
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
                JavaMethodProto::new("getMaxLength", "()I", Self::get_max_length, MethodAccessFlags::PUBLIC),
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
                // 73f3a21e981c (`m_td [C`). setMaxLength keeps its value here; m_td starts empty
                // (see init) and keyNotify below fills it.
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

        // Never null: 9789fec50f39's TextFieldComponent subclass reads `m_td.length` in its own
        // keyNotify before handing the key on, so a null buffer was an NPE on the first key.
        let empty = jvm.instantiate_array("C", 0).await?;
        jvm.put_field(&mut this, "m_td", "[C", empty).await?;

        Ok(())
    }

    // Kept so getMaxLength can answer it: d448aee68157's name box calls getMaxLength()I on the
    // first key it receives, after setMaxLength(5). Nothing caps the typed text yet (keyNotify).
    async fn set_max_length(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<TextComponent>, max_length: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setMaxLength({this:?}, {max_length})");

        jvm.put_field(&mut this, "maxLength", "I", max_length).await
    }

    async fn get_max_length(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getMaxLength({this:?})");

        jvm.get_field(&this, "maxLength", "I").await
    }

    // m_td is the text, whoever wrote it: setString, a typed key, or a guest subclass directly.
    // 9789fec50f39 calls setString("") and reads getString() back every frame until they agree;
    // with setString a no-op and getString answering "temp", that never settled and the
    // allocations exhausted the guest heap.
    async fn set_string(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<TextComponent>,
        data: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::setString({this:?}, {data:?})");

        let chars: ClassInstanceRef<Array<JavaChar>> = if data.is_null() {
            jvm.instantiate_array("C", 0).await?.into()
        } else {
            jvm.invoke_virtual(&data, "java/lang/String", "toCharArray", "()[C", ()).await?
        };
        jvm.put_field(&mut this, "m_td", "[C", chars).await
    }

    async fn get_string(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<TextComponent>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.lwc.TextComponent::getString({this:?})");

        let text: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&this, "m_td", "[C").await?;

        Ok(jvm.new_class("java/lang/String", "([C)V", (text,)).await?.into())
    }

    // Minimal input: a digit press appends it, CLR removes the last character — the keys a
    // ShellComponent hands the focused text widget. The text is kept in m_td, the canonical
    // buffer field (see its comment). No multi-tap letters, and no length cap: setMaxLength only
    // records the value. 1 = KEY_PRESSED, -16 = CLR as net.wie.CardCanvas sends them.
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

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaChar, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    /// d448aee68157's name box: setMaxLength(5), then getMaxLength()I on the first key it receives —
    /// the method was missing, and the tick died on it.
    #[test]
    fn get_max_length_answers_what_set_max_length_kept() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let null: ClassInstanceRef<()> = ClassInstanceRef::new(None);
            let text_box = jvm
                .new_class("org/kwis/msp/lwc/TextBoxComponent", "(Ljava/lang/String;I)V", (null, 0))
                .await?;
            let _: () = jvm
                .invoke_virtual(&text_box, "org/kwis/msp/lwc/TextBoxComponent", "setMaxLength", "(I)V", (5,))
                .await?;
            let max: i32 = jvm
                .invoke_virtual(&text_box, "org/kwis/msp/lwc/TextBoxComponent", "getMaxLength", "()I", ())
                .await?;
            assert_eq!(max, 5);

            Ok(())
        })
    }

    /// 9789fec50f39's name box: its TextFieldComponent subclass reads `m_td.length` on the first
    /// key, before anything is typed, and its frame loop calls setString("") then reads getString()
    /// back until the two agree.
    #[test]
    fn text_lives_in_m_td_from_construction_and_set_string_keeps_it() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let null: ClassInstanceRef<()> = ClassInstanceRef::new(None);
            let field = jvm
                .new_class("org/kwis/msp/lwc/TextFieldComponent", "(Ljava/lang/String;I)V", (null, 0))
                .await?;
            let td: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&field, "m_td", "[C").await?;
            assert!(!td.is_null(), "m_td is null before any key");
            assert_eq!(jvm.array_length(&td).await?, 0);

            let text = JavaLangString::from_rust_string(&jvm, "ab").await?;
            let _: () = jvm
                .invoke_virtual(
                    &field,
                    "org/kwis/msp/lwc/TextFieldComponent",
                    "setString",
                    "(Ljava/lang/String;)V",
                    (text,),
                )
                .await?;
            let read = jvm
                .invoke_virtual(&field, "org/kwis/msp/lwc/TextComponent", "getString", "()Ljava/lang/String;", ())
                .await?;
            assert_eq!(JavaLangString::to_rust_string(&jvm, &read).await?, "ab");

            Ok(())
        })
    }
}
