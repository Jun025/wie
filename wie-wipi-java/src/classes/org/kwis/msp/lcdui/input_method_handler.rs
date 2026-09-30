use alloc::vec;

use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::InputMethodListener;

// class org.kwis.msp.lcdui.InputMethodHandler
pub struct InputMethodHandler;

const KEY_PRESSED: i32 = 1; // org.kwis.msp.lcdui.EventQueue.KEY_PRESSED — what titles pass here (measured)
const CLEAR: i32 = -16; // the WIPI CLR key code (net/wie/CardCanvas)
const INSERT: i32 = -1;
const REPLACE: i32 = 0;
const DELETE: i32 = 1;
const MULTITAP_MS: i64 = 1000;
// Korean · English upper · English lower · number: the four the handsets' mode indicator cycled.
// ponytail: a count, not a measured table — nothing here reads which mode is which.
const MODE_COUNT: i32 = 4;
// TextComponent: CONSTRAINT_NUMBER = 1; PASSWORD and PHONENUMBER are digits too (javadoc).
const NUMERIC_CONSTRAINTS: [i32; 3] = [1, 2, 5];
const MULTITAP: [&str; 10] = [" ", ".,-", "ABC", "DEF", "GHI", "JKL", "MNO", "PQRS", "TUV", "WXYZ"];

impl InputMethodHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/InputMethodHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setCurrentMode", "(I)Z", Self::set_current_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCurrentMode", "()I", Self::get_current_mode, MethodAccessFlags::PUBLIC),
                // d448aee68157 dies on `getCurrentInputMode()I not found` (progress census 2026-09-30).
                // The javadoc gives it the same meaning as getCurrentMode, so it reads the same field.
                JavaMethodProto::new("getCurrentInputMode", "()I", Self::get_current_mode, MethodAccessFlags::PUBLIC),
                // …and then on `changeCurrentModeToNext()V not found` (same census, after the row above).
                JavaMethodProto::new(
                    "changeCurrentModeToNext",
                    "()V",
                    Self::change_current_mode_to_next,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("hideSymbolCard", "()V", Self::hide_symbol_card, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("notifyKeyInput", "(II)Z", Self::notify_key_input, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setInputMethodListener",
                    "(Lorg/kwis/msp/lcdui/InputMethodListener;)V",
                    Self::set_input_method_listener,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("mode", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("listener", "Lorg/kwis/msp/lcdui/InputMethodListener;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("constraint", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lastKey", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("lastAt", "J", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tap", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraint: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::<init>({this:?}, {constraint})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "constraint", "I", constraint).await?;

        Ok(())
    }

    // setCurrentMode already claims success, so a getter returning a constant would contradict
    // it. Storing the mode keeps the pair consistent; nothing below this layer consumes it.
    async fn set_current_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, mode: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::setCurrentMode({this:?}, {mode})");

        jvm.put_field(&mut this, "mode", "I", mode).await?;

        Ok(true)
    }

    // Every mode types the same Latin multi-tap here (notifyKeyInput), so the order of modes is not
    // observable beyond the number itself; it steps through MODE_COUNT values and wraps, which keeps
    // getCurrentMode inside a range a title can index a label table with.
    async fn change_current_mode_to_next(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::changeCurrentModeToNext({this:?})");

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        jvm.put_field(&mut this, "mode", "I", (mode + 1).rem_euclid(MODE_COUNT)).await
    }

    async fn get_current_mode(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::getCurrentMode({this:?})");

        jvm.get_field(&this, "mode", "I").await
    }

    // wie draws no symbol card, so hiding one is already the resting state.
    async fn hide_symbol_card(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::hideSymbolCard({this:?})");

        Ok(())
    }

    // A multi-tap keypad, which is what the handsets' automata did for Latin letters: a digit key
    // inserts the first letter of its group, the same key again within MULTITAP_MS replaces it with
    // the next, CLR deletes. Each change goes to the listener's `notifyTextChanged(chars, len, pMode)`
    // (pMode insert -1 / replace 0 / delete 1 — javadoc). Numeric constraints insert the digit.
    //
    // Why it exists: the stub consumed nothing and never called the listener, so a title that asks
    // for a name through this handler could never get past the prompt — a KTF title's shop-name screen
    // answered every key with «at least 1 character» for 30 minutes (1e43e2e0055f, progress census
    // 2026-09-30). ponytail: no Hangul automaton and no mode switching — every mode types Latin.
    // A name the player can type is the unblock; a Korean automaton is the upgrade if a title
    // rejects Latin names.
    async fn notify_key_input(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32, r#type: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::notifyKeyInput({this:?}, {key}, {type})");

        let listener: ClassInstanceRef<InputMethodListener> = jvm.get_field(&this, "listener", "Lorg/kwis/msp/lcdui/InputMethodListener;").await?;
        if listener.is_null() {
            return Ok(false); // javadoc: no listener -> false, nothing done
        }
        if r#type != KEY_PRESSED {
            return Ok(key == CLEAR || (b'0' as i32..=b'9' as i32).contains(&key));
        }

        let (chars, mode) = if key == CLEAR {
            jvm.put_field(&mut this, "lastKey", "I", 0).await?;
            (vec![], DELETE)
        } else if let Some(group) = (b'0' as i32..=b'9' as i32).contains(&key).then(|| MULTITAP[(key - b'0' as i32) as usize]) {
            let constraint: i32 = jvm.get_field(&this, "constraint", "I").await?;
            if NUMERIC_CONSTRAINTS.contains(&constraint) {
                (vec![key as JavaChar], INSERT)
            } else {
                let now = context.system().platform().now().raw() as i64;
                let last_key: i32 = jvm.get_field(&this, "lastKey", "I").await?;
                let last_at: i64 = jvm.get_field(&this, "lastAt", "J").await?;
                let again = last_key == key && now - last_at < MULTITAP_MS;
                let tap = if again { jvm.get_field::<i32>(&this, "tap", "I").await? + 1 } else { 0 };
                jvm.put_field(&mut this, "lastKey", "I", key).await?;
                jvm.put_field(&mut this, "lastAt", "J", now).await?;
                jvm.put_field(&mut this, "tap", "I", tap).await?;
                let letters = group.as_bytes();
                (
                    vec![letters[tap as usize % letters.len()] as JavaChar],
                    if again { REPLACE } else { INSERT },
                )
            }
        } else {
            return Ok(false);
        };

        let mut array = jvm.instantiate_array("C", chars.len().max(1)).await?;
        jvm.store_array(&mut array, 0, chars.clone()).await?;
        // Resolved on the listener's own class: the interface proto here declares no methods.
        let class = listener.class_definition().name();
        let _: () = jvm
            .invoke_virtual(&listener, &class, "notifyTextChanged", "([CII)V", (array, 1, mode))
            .await?;

        Ok(true)
    }

    // wie has no input method, so the listener is never called back; it is kept so the
    // registration is observable to anything that later reads it.
    async fn set_input_method_listener(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<InputMethodListener>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::setInputMethodListener({this:?}, {listener:?})");

        jvm.put_field(&mut this, "listener", "Lorg/kwis/msp/lcdui/InputMethodListener;", listener)
            .await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, string::String, vec, vec::Vec};

    use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::get_protos;

    // A title's listener: keeps the text the way the javadoc's pMode says (insert -1 / replace 0 / delete 1).
    struct Recorder;
    impl Recorder {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/Recorder",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["org/kwis/msp/lcdui/InputMethodListener"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("notifyTextChanged", "([CII)V", Self::changed, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PUBLIC)],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }
        async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
            let empty = JavaLangString::from_rust_string(jvm, "").await?;
            jvm.put_field(&mut this, "text", "Ljava/lang/String;", empty).await
        }
        async fn changed(
            jvm: &Jvm,
            _: &mut WieJvmContext,
            mut this: ClassInstanceRef<Self>,
            chars: ClassInstanceRef<Array<JavaChar>>,
            len: i32,
            mode: i32,
        ) -> JvmResult<()> {
            let text = JavaLangString::to_rust_string(jvm, &jvm.get_field(&this, "text", "Ljava/lang/String;").await?).await?;
            let mut text: Vec<char> = text.chars().collect();
            let got: Vec<JavaChar> = jvm.load_array(&chars, 0, len as usize).await?;
            match mode {
                -1 => text.extend(got.iter().map(|&c| c as u8 as char)),
                0 => {
                    text.pop();
                    text.extend(got.iter().map(|&c| c as u8 as char));
                }
                _ => {
                    text.pop();
                }
            }
            let text = JavaLangString::from_rust_string(jvm, &text.into_iter().collect::<String>()).await?;
            jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await
        }
    }

    async fn typed(jvm: &Jvm, constraint: i32, keys: &[i32]) -> JvmResult<String> {
        let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (constraint,)).await?;
        let listener = jvm.new_class("test/Recorder", "()V", ()).await?;
        let _: () = jvm
            .invoke_virtual(
                &handler,
                "org/kwis/msp/lcdui/InputMethodHandler",
                "setInputMethodListener",
                "(Lorg/kwis/msp/lcdui/InputMethodListener;)V",
                (listener.clone(),),
            )
            .await?;
        for &key in keys {
            let consumed: bool = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "notifyKeyInput", "(II)Z", (key, 1))
                .await?;
            assert!(consumed, "key {key} not consumed");
        }
        JavaLangString::to_rust_string(jvm, &jvm.get_field(&listener, "text", "Ljava/lang/String;").await?).await
    }

    // 1e43e2e0055f's shop-name prompt: the stub consumed no key and called no listener, so the
    // name stayed empty forever. Keys arrive as ASCII digits with KEY_PRESSED (measured: (53, 1)).
    #[test]
    fn keys_reach_the_listener_as_text() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into(), Box::new([Recorder::as_proto()])]), |jvm| async move {
            assert_eq!(typed(&jvm, 0, &[b'2' as i32, b'2' as i32, b'3' as i32]).await?, "BD"); // multi-tap: 2 2 -> B
            assert_eq!(typed(&jvm, 0, &[b'4' as i32, b'6' as i32, -16]).await?, "G"); // CLR deletes
            assert_eq!(typed(&jvm, 1, &[b'2' as i32, b'2' as i32]).await?, "22"); // CONSTRAINT_NUMBER types digits

            // d448aee68157 asks for the mode by the other name.
            let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (0,)).await?;
            let _: bool = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "setCurrentMode", "(I)Z", (3,))
                .await?;
            let mode: i32 = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "getCurrentInputMode", "()I", ())
                .await?;
            assert_eq!(mode, 3);
            let _: () = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "changeCurrentModeToNext", "()V", ())
                .await?;
            let mode: i32 = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "getCurrentMode", "()I", ())
                .await?;
            assert_eq!(mode, 0, "wraps after the last mode");

            // No listener: the javadoc's false, and nothing done.
            let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (0,)).await?;
            let consumed: bool = jvm
                .invoke_virtual(
                    &handler,
                    "org/kwis/msp/lcdui/InputMethodHandler",
                    "notifyKeyInput",
                    "(II)Z",
                    (b'5' as i32, 1),
                )
                .await?;
            assert!(!consumed);

            Ok(())
        })
    }
}
