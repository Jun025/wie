use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.lcdui.TextComponentHandler
//
// The handset's text-input overlay. The one caller measured (9a2cf5ffc9d3, paint path) gates every
// other use on `isLoaded()` — `invokestatic isLoaded; ifne …; return` — so reporting "not loaded"
// is the whole contract it needs: no input-method indicator is drawn. 14a62a8521a0 takes the handler
// unconditionally in its text field's constructor, asks it for the input mode and offers it every
// key, so those are here: one shared handler, mode 0, and no key consumed (false hands the key back
// to the title). getTextComponent is still absent — no measured caller.
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
                JavaMethodProto::new("keyPressed", "(I)Z", Self::key_event, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyReleased", "(I)Z", Self::key_event, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("keyRepeated", "(I)Z", Self::key_event, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![JavaFieldProto::new(
                "instance",
                "Lcom/xce/lcdui/TextComponentHandler;",
                FieldAccessFlags::PRIVATE | FieldAccessFlags::STATIC,
            )],
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

    async fn key_event(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, key: i32) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::key*({this:?}, {key})");

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::ClassInstanceRef;
    use test_utils::run_jvm_test;

    use crate::get_protos;

    use super::TextComponentHandler;

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
}
