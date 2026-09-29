use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lcdui::InputMethodListener;

// class org.kwis.msp.lcdui.InputMethodHandler
pub struct InputMethodHandler;

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
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, constraint: i32) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lcdui.InputMethodHandler::<init>({this:?}, {constraint})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    // setCurrentMode already claims success, so a getter returning a constant would contradict
    // it. Storing the mode keeps the pair consistent; nothing below this layer consumes it.
    async fn set_current_mode(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, mode: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::setCurrentMode({this:?}, {mode})");

        jvm.put_field(&mut this, "mode", "I", mode).await?;

        Ok(true)
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

    // wie has no input method, so no key is consumed by one: false hands every key back to the
    // title's own handler.
    async fn notify_key_input(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, key: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.lcdui.InputMethodHandler::notifyKeyInput({this:?}, {type}, {key})");

        Ok(false)
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
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    // A KTF title registers a listener before its text-entry screen; without the method that is fatal.
    #[test]
    fn listener_and_key_input_are_accepted() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let handler = jvm.new_class("org/kwis/msp/lcdui/InputMethodHandler", "(I)V", (0,)).await?;
            let listener: ClassInstanceRef<super::InputMethodListener> = jvm.new_class("java/lang/Object", "()V", ()).await?.into();
            let _: () = jvm
                .invoke_virtual(
                    &handler,
                    "org/kwis/msp/lcdui/InputMethodHandler",
                    "setInputMethodListener",
                    "(Lorg/kwis/msp/lcdui/InputMethodListener;)V",
                    (listener,),
                )
                .await?;
            let consumed: bool = jvm
                .invoke_virtual(&handler, "org/kwis/msp/lcdui/InputMethodHandler", "notifyKeyInput", "(II)Z", (1, 53))
                .await?;
            assert!(!consumed);

            Ok(())
        })
    }
}
