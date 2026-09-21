use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

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
            ],
            fields: vec![JavaFieldProto::new("mode", "I", FieldAccessFlags::PRIVATE)],
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
}
