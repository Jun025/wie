use alloc::vec;

use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_util::keypad::{self, MULTITAP_MS, Mode, Op};

use super::TextComponent;

// MIDP key codes as this engine delivers them (wie-midp event_queue).
const CLEAR: i32 = 8;
const LEFT: i32 = 142;
const RIGHT: i32 = 145;
// javax.microedition.lcdui.TextField
const CONSTRAINT_MASK: i32 = 0xFFFF;
const NUMERIC: i32 = 2;
const PHONENUMBER: i32 = 3;
const DECIMAL: i32 = 5;

// ponytail: upper-case Latin only (wie_util::keypad's E.161 table); no case/Hangul mode switch until
// a title needs one — the only title typing here (85f03ca7389e) takes a Latin name.

// class com.xce.lcdui.TextComponentHandler
//
// The handset's text-input overlay. The one caller measured (9a2cf5ffc9d3, paint path) gates every
// other use on `isLoaded()` — `invokestatic isLoaded; ifne …; return` — so reporting "not loaded"
// is the whole contract it needs: no input-method indicator is drawn. 14a62a8521a0 takes the handler
// unconditionally in its text field's constructor, asks it for the input mode and offers it every
// key, but never registers a field — so with nothing registered no key is consumed (false hands the
// key back to the title) and that title sees exactly what it saw before this was an input method.
//
// 85f03ca7389e's text fields register themselves with setTextComponent on focus, and their
// keyPressed is only `handler.keyPressed(key); pop` — the handler is the sole writer of text. So
// with a field registered this is a minimal multitap input method: a digit key inserts a character
// through the field's own TextComponent methods, the same key again within MULTITAP_MS cycles it in
// place (replace), CLEAR deletes and LEFT/RIGHT go to moveCursor. NUMERIC-like constraints take the
// digit itself. Hangul composition and mode switching are not here. The field calls clear() whenever
// it resets its text or moves the cursor; that ends the current multitap cycle. getTextComponent is
// still absent — no measured caller reaches it.
pub struct TextComponentHandler;

impl TextComponentHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponentHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new("isLoaded", "()Z", Self::is_loaded, MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC),
                JavaMethodProto::new(
                    "getTextComponentHandler",
                    "()Lcom/xce/lcdui/TextComponentHandler;",
                    Self::get_text_component_handler,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getInputMode", "()I", Self::get_input_mode, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyPressed", "(I)Z", Self::key_pressed, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)Z", Self::key_event, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyRepeated", "(I)Z", Self::key_event, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setTextComponent",
                    "(Lcom/xce/lcdui/TextComponent;)V",
                    Self::set_text_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("clear", "()V", Self::clear, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new(
                    "instance",
                    "Lcom/xce/lcdui/TextComponentHandler;",
                    FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
                ),
                JavaFieldProto::new("component", "Lcom/xce/lcdui/TextComponent;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapKey", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapChar", "C", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("tapTime", "J", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn is_loaded(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::isLoaded()");

        Ok(false)
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn get_text_component_handler(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<Self>> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::getTextComponentHandler()");

        const NAME: &str = "com/xce/lcdui/TextComponentHandler";
        const DESCRIPTOR: &str = "Lcom/xce/lcdui/TextComponentHandler;";
        let instance: ClassInstanceRef<Self> = jvm.get_static_field(NAME, "instance", DESCRIPTOR).await?;
        if !instance.is_null() {
            return Ok(instance);
        }
        let instance: ClassInstanceRef<Self> = jvm.new_class(NAME, "()V", ()).await?.into();
        jvm.put_static_field(NAME, "instance", DESCRIPTOR, instance.clone()).await?;

        Ok(instance)
    }

    async fn get_input_mode(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.lcdui.TextComponentHandler::getInputMode({this:?})");

        Ok(0)
    }

    async fn key_pressed(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::keyPressed({this:?}, {key})");

        const NAME: &str = "com/xce/lcdui/TextComponent";
        let component: ClassInstanceRef<TextComponent> = jvm.get_field(&this, "component", "Lcom/xce/lcdui/TextComponent;").await?;
        if component.is_null() {
            return Ok(false);
        }

        match key {
            CLEAR => {
                let _: () = jvm.invoke_virtual(&component, NAME, "delete", "()V", ()).await?;
                // The letter being cycled may be the one just deleted: replace would write at caret-1.
                jvm.put_field(&mut this, "tapKey", "I", 0).await?;
            }
            LEFT | RIGHT => {
                // moveCursor calls our clear() itself and repaints.
                return jvm.invoke_virtual(&component, NAME, "moveCursor", "(I)V", (key,)).await.map(|()| true);
            }
            0x30..=0x39 => {
                let constraints: i32 = jvm.invoke_virtual(&component, NAME, "getConstraints", "()I", ()).await?;
                let mode = if matches!(constraints & CONSTRAINT_MASK, NUMERIC | PHONENUMBER | DECIMAL) {
                    Mode::Digit
                } else {
                    Mode::Upper
                };

                // Latin renders as typed, so the letter being cycled is the whole composition.
                let now = context.system().platform().now().raw() as i64;
                let tap_key: i32 = jvm.get_field(&this, "tapKey", "I").await?;
                let tap_time: i64 = jvm.get_field(&this, "tapTime", "J").await?;
                let again = tap_key == key && now - tap_time < MULTITAP_MS;
                let mut tokens = if again {
                    vec![char::from_u32(jvm.get_field::<JavaChar>(&this, "tapChar", "C").await? as u32).unwrap_or(' ')]
                } else {
                    vec![]
                };
                let edit = keypad::press(mode, &mut tokens, (key - 0x30) as u8, again);
                let before: i32 = jvm.invoke_virtual(&component, NAME, "size", "()I", ()).await?;
                for op in edit.ops() {
                    let _: () = match op {
                        Op::Insert(c) => jvm.invoke_virtual(&component, NAME, "insert", "(C)V", (c as JavaChar,)).await?,
                        Op::Replace(c) => jvm.invoke_virtual(&component, NAME, "replace", "(C)V", (c as JavaChar,)).await?,
                        Op::Delete => jvm.invoke_virtual(&component, NAME, "delete", "()V", ()).await?,
                    };
                }
                let after: i32 = jvm.invoke_virtual(&component, NAME, "size", "()I", ()).await?;
                // A full field ignores insert; cycling then would rewrite the last character instead.
                let typed = after > before || (again && edit.delete > 0);
                jvm.put_field(&mut this, "tapKey", "I", if typed { key } else { 0 }).await?;
                jvm.put_field(&mut this, "tapChar", "C", tokens[0] as JavaChar).await?;
                jvm.put_field(&mut this, "tapTime", "J", now).await?;
            }
            _ => return Ok(false),
        }
        let _: () = jvm.invoke_virtual(&component, NAME, "repaint", "()V", ()).await?;

        Ok(true)
    }

    async fn key_event(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::key*({this:?}, {key})");

        Ok(false)
    }

    async fn set_text_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<TextComponent>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::setTextComponent({this:?}, {component:?})");

        jvm.put_field(&mut this, "component", "Lcom/xce/lcdui/TextComponent;", component).await?;
        jvm.put_field(&mut this, "tapKey", "I", 0).await
    }

    async fn clear(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::clear({this:?})");

        jvm.put_field(&mut this, "tapKey", "I", 0).await
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use jvm::{ClassInstanceRef, JavaChar, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
    use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

    use crate::get_protos;

    use super::{TextComponent, TextComponentHandler};

    /// 9a2cf5ffc9d3 died on `NoClassDefFoundError: com/xce/lcdui/TextComponentHandler` in paint.
    #[test]
    fn text_component_handler_reports_not_loaded() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let loaded: bool = jvm.invoke_static("com/xce/lcdui/TextComponentHandler", "isLoaded", "()Z", ()).await?;
            assert!(!loaded);

            // 14a62a8521a0's text field constructor.
            let descriptor = "()Lcom/xce/lcdui/TextComponentHandler;";
            let a: ClassInstanceRef<TextComponentHandler> = jvm
                .invoke_static("com/xce/lcdui/TextComponentHandler", "getTextComponentHandler", descriptor, ())
                .await?;
            let b: ClassInstanceRef<TextComponentHandler> = jvm
                .invoke_static("com/xce/lcdui/TextComponentHandler", "getTextComponentHandler", descriptor, ())
                .await?;
            assert!(!a.is_null() && a.equals(&**b)?);
            let mode: i32 = jvm
                .invoke_virtual(&a, "com/xce/lcdui/TextComponentHandler", "getInputMode", "()I", ())
                .await?;
            assert_eq!(mode, 0);
            let consumed: bool = jvm
                .invoke_virtual(&a, "com/xce/lcdui/TextComponentHandler", "keyReleased", "(I)Z", (53,))
                .await?;
            assert!(!consumed);
            Ok(())
        });
        assert!(result.is_ok(), "{result:?}");
    }

    // The shape of 85f03ca7389e's text-field inner class: implements the interface, one method shown.
    struct InputMethodImpl;

    impl InputMethodImpl {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/InputMethodImpl",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["com/xce/lcdui/TextComponent"],
                methods: vec![
                    JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
            jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
        }

        async fn size(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>) -> JvmResult<i32> {
            Ok(7)
        }
    }

    /// 85f03ca7389e lost its game thread on `NoClassDefFoundError: com/xce/lcdui/TextComponent` when
    /// it built a text field; then the field registers itself and clears the handler.
    #[test]
    fn text_component_implementor_loads_and_registers() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [InputMethodImpl::as_proto()].into()]),
            |jvm| async move {
                let component: ClassInstanceRef<TextComponent> = jvm.new_class("test/InputMethodImpl", "()V", ()).await?.into();
                assert!(jvm.is_instance(&**component, "com/xce/lcdui/TextComponent"));
                let size: i32 = jvm.invoke_virtual(&component, "com/xce/lcdui/TextComponent", "size", "()I", ()).await?;
                assert_eq!(size, 7);

                let handler: ClassInstanceRef<TextComponentHandler> = jvm
                    .invoke_static(
                        "com/xce/lcdui/TextComponentHandler",
                        "getTextComponentHandler",
                        "()Lcom/xce/lcdui/TextComponentHandler;",
                        (),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &handler,
                        "com/xce/lcdui/TextComponentHandler",
                        "setTextComponent",
                        "(Lcom/xce/lcdui/TextComponent;)V",
                        (component,),
                    )
                    .await?;
                let null = ClassInstanceRef::<TextComponent>::new(None);
                let _: () = jvm
                    .invoke_virtual(
                        &handler,
                        "com/xce/lcdui/TextComponentHandler",
                        "setTextComponent",
                        "(Lcom/xce/lcdui/TextComponent;)V",
                        (null,),
                    )
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&handler, "com/xce/lcdui/TextComponentHandler", "clear", "()V", ())
                    .await?;
                Ok(())
            },
        );
        assert!(result.is_ok(), "{result:?}");
    }

    // A text field that keeps its caret at the end, like 85f03ca7389e's while the name is typed.
    struct EditField;

    impl EditField {
        fn as_proto() -> WieJavaClassProto {
            WieJavaClassProto {
                name: "test/EditField",
                parent_class: Some("java/lang/Object"),
                interfaces: vec!["com/xce/lcdui/TextComponent"],
                methods: vec![
                    JavaMethodProto::new("<init>", "(II)V", Self::init, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("getConstraints", "()I", Self::get_constraints, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("size", "()I", Self::size, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("insert", "(C)V", Self::insert, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("replace", "(C)V", Self::replace, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("delete", "()V", Self::delete, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("moveCursor", "(I)V", Self::move_cursor, MethodAccessFlags::PUBLIC),
                    JavaMethodProto::new("repaint", "()V", Self::repaint, MethodAccessFlags::PUBLIC),
                ],
                fields: vec![
                    JavaFieldProto::new("text", "Ljava/lang/String;", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("constraints", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("maxSize", "I", FieldAccessFlags::PUBLIC),
                    JavaFieldProto::new("moved", "I", FieldAccessFlags::PUBLIC),
                ],
                access_flags: ClassAccessFlags::PUBLIC,
            }
        }

        async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, constraints: i32, max_size: i32) -> JvmResult<()> {
            let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
            Self::set(jvm, &mut this, alloc::string::String::new()).await?;
            jvm.put_field(&mut this, "constraints", "I", constraints).await?;
            jvm.put_field(&mut this, "maxSize", "I", max_size).await
        }

        async fn get(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<alloc::string::String> {
            let text: ClassInstanceRef<String> = jvm.get_field(this, "text", "Ljava/lang/String;").await?;
            JavaLangString::to_rust_string(jvm, &text).await
        }

        async fn set(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, text: alloc::string::String) -> JvmResult<()> {
            let text: ClassInstanceRef<String> = JavaLangString::from_rust_string(jvm, &text).await?.into();
            jvm.put_field(this, "text", "Ljava/lang/String;", text).await
        }

        async fn get_constraints(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            jvm.get_field(&this, "constraints", "I").await
        }

        async fn size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
            Ok(Self::get(jvm, &this).await?.len() as i32)
        }

        async fn insert(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, c: JavaChar) -> JvmResult<()> {
            let mut text = Self::get(jvm, &this).await?;
            if (text.len() as i32) < jvm.get_field::<i32>(&this, "maxSize", "I").await? {
                text.push(c as u8 as char);
            }
            Self::set(jvm, &mut this, text).await
        }

        async fn replace(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, c: JavaChar) -> JvmResult<()> {
            let mut text = Self::get(jvm, &this).await?;
            text.pop();
            text.push(c as u8 as char);
            Self::set(jvm, &mut this, text).await
        }

        async fn delete(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
            let mut text = Self::get(jvm, &this).await?;
            text.pop();
            Self::set(jvm, &mut this, text).await
        }

        async fn move_cursor(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, key: i32) -> JvmResult<()> {
            jvm.put_field(&mut this, "moved", "I", key).await
        }

        async fn repaint(_: &Jvm, _: &mut WieJvmContext, _: ClassInstanceRef<Self>) -> JvmResult<()> {
            Ok(())
        }
    }

    /// 85f03ca7389e's name field only forwards keys (`handler.keyPressed(key); pop`): the text in it
    /// is whatever this handler inserts. Unregistered, keys are handed back untouched.
    #[test]
    fn registered_field_takes_multitap_and_digits() {
        let result = run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), [EditField::as_proto()].into()]),
            |jvm| async move {
                const H: &str = "com/xce/lcdui/TextComponentHandler";
                let handler: ClassInstanceRef<TextComponentHandler> = jvm
                    .invoke_static(H, "getTextComponentHandler", "()Lcom/xce/lcdui/TextComponentHandler;", ())
                    .await?;
                async fn press(jvm: &Jvm, handler: &ClassInstanceRef<TextComponentHandler>, key: i32) -> JvmResult<bool> {
                    jvm.invoke_virtual(handler, H, "keyPressed", "(I)Z", (key,)).await
                }
                async fn register(jvm: &Jvm, handler: &ClassInstanceRef<TextComponentHandler>, field: &ClassInstanceRef<EditField>) -> JvmResult<()> {
                    let field = ClassInstanceRef::<TextComponent>::new(field.instance.clone());
                    jvm.invoke_virtual(handler, H, "setTextComponent", "(Lcom/xce/lcdui/TextComponent;)V", (field,))
                        .await
                }
                let (j, h) = (&jvm, &handler);

                // Nothing registered: 14a62a8521a0's situation — every key goes back to the title.
                assert!(!press(j, h, 0x35).await?);

                let name: ClassInstanceRef<EditField> = jvm.new_class("test/EditField", "(II)V", (0, 3)).await?.into();
                register(j, h, &name).await?;
                // 5 → J, 5 again → K (cycled in place), 2 → A, CLEAR → deletes.
                for key in [0x35, 0x35, 0x32] {
                    assert!(press(j, h, key).await?);
                }
                assert_eq!(EditField::get(&jvm, &name).await?, "KA");
                assert!(press(j, h, 8).await?);
                assert_eq!(EditField::get(&jvm, &name).await?, "K");
                // CLEAR ended the cycle: 2 again is a new letter, not a rewrite of K.
                assert!(press(j, h, 0x32).await?);
                assert_eq!(EditField::get(&jvm, &name).await?, "KA");
                // clear() (the field resets or moves its caret) ends the cycle too.
                let _: () = jvm.invoke_virtual(&handler, H, "clear", "()V", ()).await?;
                assert!(press(j, h, 0x32).await?);
                assert_eq!(EditField::get(&jvm, &name).await?, "KAA");
                // Full field: insert is ignored, and pressing again must not rewrite the last letter.
                assert!(press(j, h, 0x34).await?);
                assert!(press(j, h, 0x34).await?);
                assert_eq!(EditField::get(&jvm, &name).await?, "KAA");
                assert!(press(j, h, 142).await?);
                assert_eq!(jvm.get_field::<i32>(&name, "moved", "I").await?, 142);
                assert!(!press(j, h, 0x23).await?); // '#': no mode switch here

                // NUMERIC (85f03ca7389e's 11-digit field) takes the digit itself.
                let number: ClassInstanceRef<EditField> = jvm.new_class("test/EditField", "(II)V", (2, 11)).await?.into();
                register(j, h, &number).await?;
                for key in [0x30, 0x31, 0x31] {
                    assert!(press(j, h, key).await?);
                }
                assert_eq!(EditField::get(&jvm, &number).await?, "011");

                // Unregistered again (focus left): keys go back to the title.
                register(j, h, &ClassInstanceRef::new(None)).await?;
                assert!(!press(j, h, 0x35).await?);
                Ok(())
            },
        );
        assert!(result.is_ok(), "{result:?}");
    }
}
