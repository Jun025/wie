use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_util::keypad::{self, Edit, MULTITAP_MS, Mode, Op};

use crate::classes::org::kwis::msp::lcdui::InputMethodListener;

// class org.kwis.msp.lcdui.InputMethodHandler
pub struct InputMethodHandler;

const KEY_PRESSED: i32 = 1; // org.kwis.msp.lcdui.EventQueue.KEY_PRESSED — what titles pass here (measured)
const CLEAR: i32 = -16; // the WIPI CLR key code (net/wie/CardCanvas)
const INSERT: i32 = -1;
const REPLACE: i32 = 0;
const DELETE: i32 = 1;
// The mode numbers. Only 3 is measured: 9789fec50f39 and b22a7fcfb406 both setCurrentMode(3) on a
// name prompt, 9789fec50f39 erases every digit typed there but keeps Hangul, and b22a7fcfb406's own
// strip «가 A a 1» highlights 가 at 3. The rest follow that strip's order as changeCurrentModeToNext
// steps it (3 → 0 → 1 → 2 → 3). ponytail: 0..2 are assumed (docs/report/0428 §2) — no reference here
// numbers the modes; the stripped AromaWIPI classes leave them to native code.
const MODES: [Mode; 4] = [Mode::Upper, Mode::Lower, Mode::Digit, Mode::Hangul];
const HANGUL_MODE: i32 = 3;
// TextComponent: CONSTRAINT_NUMBER = 1; PASSWORD and PHONENUMBER are digits too (javadoc).
const NUMERIC_CONSTRAINTS: [i32; 3] = [1, 2, 5];

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
                // The keys of the word being composed (wie_util::keypad tokens).
                JavaFieldProto::new("composing", "[C", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraint: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::<init>({this:?}, {constraint})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "constraint", "I", constraint).await?;
        // A handset starts a text field in Korean.
        jvm.put_field(&mut this, "mode", "I", HANGUL_MODE).await?;

        Ok(())
    }

    // setCurrentMode already claims success, so a getter returning a constant would contradict
    // it. Storing the mode keeps the pair consistent; nothing below this layer consumes it.
    async fn set_current_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, mode: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::setCurrentMode({this:?}, {mode})");

        jvm.put_field(&mut this, "mode", "I", mode).await?;
        Self::end_composition(jvm, &mut this).await?;

        Ok(true)
    }

    // Steps through MODES and wraps, which keeps getCurrentMode inside a range a title can index a
    // label table with. lwc TextComponent calls this on '*'.
    pub async fn change_current_mode_to_next(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::changeCurrentModeToNext({this:?})");

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        jvm.put_field(&mut this, "mode", "I", (mode + 1).rem_euclid(MODES.len() as i32)).await?;
        Self::end_composition(jvm, &mut this).await
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

    // The handsets' automaton (wie_util::keypad): multi-tap Latin, 천지인 Hangul, or digits for the
    // numeric constraints; CLR takes back the last key. Each change goes to the listener's
    // `notifyTextChanged(chars, len, pMode)` (pMode insert -1 / replace 0 / delete 1 — javadoc), one
    // call per character: a Hangul key can rewrite one syllable and add the next (각 + ㅏ → 가가).
    //
    // Argument order is (keyCode, type), not the (type, key) the stub was declared with: the javadoc
    // says `notifyKeyInput(int keyCode, int type)` with type = EventQueue.KEY_PRESSED/RELEASED
    // (docs/reference/AromaWIPI_javadoc.zip, org/kwis/msp/lcdui/InputMethodHandler.html), and a KTF
    // title's calls log as (53, 1) — '5' pressed (docs/report/0393 §4). A delete passes len 1 over a
    // one-slot array holding 0: the javadoc gives `len` as «처리할 문자의 갯수» and pMode 1 as delete,
    // so the count is what a listener acts on; there is no character to hand over.
    //
    // Why it exists: the stub consumed nothing and never called the listener, so a title that asks
    // for a name through this handler could never get past the prompt — a KTF title's shop-name screen
    // answered every key with «at least 1 character» for 30 minutes (1e43e2e0055f, progress census
    // 2026-09-30).
    async fn notify_key_input(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32, r#type: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.lcdui.InputMethodHandler::notifyKeyInput({this:?}, {key}, {type})");

        let listener: ClassInstanceRef<InputMethodListener> = jvm.get_field(&this, "listener", "Lorg/kwis/msp/lcdui/InputMethodListener;").await?;
        if listener.is_null() {
            return Ok(false); // javadoc: no listener -> false, nothing done
        }
        if r#type != KEY_PRESSED {
            return Ok(key == CLEAR || (b'0' as i32..=b'9' as i32).contains(&key));
        }
        let Some(edit) = Self::compose(jvm, context, this, key, None).await? else {
            return Ok(false);
        };

        // Resolved on the listener's own class: the interface proto here declares no methods.
        let class = listener.class_definition().name();
        for op in edit.ops() {
            let (c, mode) = match op {
                Op::Insert(c) => (c, INSERT),
                Op::Replace(c) => (c, REPLACE),
                Op::Delete => ('\0', DELETE),
            };
            let mut array = jvm.instantiate_array("C", 1).await?;
            jvm.store_array(&mut array, 0, vec![c as JavaChar]).await?;
            let _: () = jvm
                .invoke_virtual(&listener, &class, "notifyTextChanged", "([CII)V", (array, 1, mode))
                .await?;
        }

        Ok(true)
    }

    /// One pressed key through the automaton: the edit to make before the caret, or None for a key
    /// it does not type (only digits and CLR are). `text` — the field's text before the caret, when
    /// the caller has it: if it no longer ends with the composition (the title rewrote it), the
    /// composition is over and this key starts a new one.
    pub async fn compose(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        key: i32,
        text: Option<&[JavaChar]>,
    ) -> JvmResult<Option<Edit>> {
        let constraint: i32 = jvm.get_field(&this, "constraint", "I").await?;
        let mode = if NUMERIC_CONSTRAINTS.contains(&constraint) {
            Mode::Digit
        } else {
            MODES[jvm.get_field::<i32>(&this, "mode", "I").await?.rem_euclid(MODES.len() as i32) as usize]
        };
        let composing: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&this, "composing", "[C").await?;
        let mut tokens: Vec<char> = if composing.is_null() {
            Vec::new()
        } else {
            let length = jvm.array_length(&composing).await?;
            let chars: Vec<JavaChar> = jvm.load_array(&composing, 0, length).await?;
            chars.into_iter().filter_map(|c| char::from_u32(c as u32)).collect()
        };
        if let Some(text) = text {
            let shown: Vec<JavaChar> = keypad::render(mode, &tokens).iter().map(|&c| c as JavaChar).collect();
            if !text.ends_with(&shown) {
                tokens.clear();
            }
        }

        let edit = if key == CLEAR {
            jvm.put_field(&mut this, "lastKey", "I", 0).await?;
            keypad::back(mode, &mut tokens).unwrap_or(Edit {
                delete: 1,
                insert: Vec::new(),
            })
        } else if (b'0' as i32..=b'9' as i32).contains(&key) {
            let now = context.system().platform().now().raw() as i64;
            let last_key: i32 = jvm.get_field(&this, "lastKey", "I").await?;
            let last_at: i64 = jvm.get_field(&this, "lastAt", "J").await?;
            jvm.put_field(&mut this, "lastKey", "I", key).await?;
            jvm.put_field(&mut this, "lastAt", "J", now).await?;
            keypad::press(
                mode,
                &mut tokens,
                (key - b'0' as i32) as u8,
                last_key == key && now - last_at < MULTITAP_MS,
            )
        } else {
            return Ok(None);
        };

        let mut composing = jvm.instantiate_array("C", tokens.len()).await?;
        jvm.store_array(&mut composing, 0, tokens.into_iter().map(|c| c as JavaChar).collect::<Vec<_>>())
            .await?;
        jvm.put_field(&mut this, "composing", "[C", composing).await?;

        Ok(Some(edit))
    }

    async fn end_composition(jvm: &Jvm, this: &mut ClassInstanceRef<Self>) -> JvmResult<()> {
        let empty = jvm.instantiate_array("C", 0).await?;
        jvm.put_field(this, "composing", "[C", empty).await?;
        jvm.put_field(this, "lastKey", "I", 0).await
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
                -1 => text.extend(got.iter().map(|&c| char::from_u32(c as u32).unwrap())),
                0 => {
                    text.pop();
                    text.extend(got.iter().map(|&c| char::from_u32(c as u32).unwrap()));
                }
                _ => {
                    text.pop();
                }
            }
            let text = JavaLangString::from_rust_string(jvm, &text.into_iter().collect::<String>()).await?;
            jvm.put_field(&mut this, "text", "Ljava/lang/String;", text).await
        }
    }

    async fn typed(jvm: &Jvm, constraint: i32, mode: i32, keys: &[i32]) -> JvmResult<String> {
        let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (constraint,)).await?;
        let _: bool = jvm
            .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "setCurrentMode", "(I)Z", (mode,))
            .await?;
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
            let keys = |s: &str| s.bytes().map(|b| if b == b'<' { -16 } else { b as i32 }).collect::<Vec<_>>();
            assert_eq!(typed(&jvm, 0, 0, &keys("223")).await?, "BD"); // English upper multi-tap: 2 2 -> B
            assert_eq!(typed(&jvm, 0, 0, &keys("46<")).await?, "G"); // CLR deletes
            assert_eq!(typed(&jvm, 0, 1, &keys("44")).await?, "h");
            // Mode 3 is Hangul (천지인): ㄱ ㅣ ㆍ ㄱ ㆍ ㅣ — the last two strokes make ㅓ and move
            // the final ㄱ on, which takes a replace and an insert in one key.
            assert_eq!(typed(&jvm, 0, 3, &keys("412421")).await?, "가거");
            assert_eq!(typed(&jvm, 0, 3, &keys("4124<")).await?, "가"); // CLR takes back the last stroke
            assert_eq!(typed(&jvm, 1, 3, &keys("22")).await?, "22");
            assert_eq!(typed(&jvm, 0, 2, &keys("22")).await?, "22"); // the number mode // CONSTRAINT_NUMBER types digits in any mode

            // d448aee68157 asks for the mode by the other name. A new handler starts in Hangul.
            let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (0,)).await?;
            let mode: i32 = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "getCurrentMode", "()I", ())
                .await?;
            assert_eq!(mode, 3);
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
