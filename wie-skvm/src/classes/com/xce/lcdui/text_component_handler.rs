use alloc::vec;

use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.lcdui.TextComponentHandler
//
// The handset's text-input overlay. The one caller measured (9a2cf5ffc9d3, paint path) gates every
// other use on `isLoaded()` — `invokestatic isLoaded; ifne …; return` — so reporting "not loaded"
// is the whole contract it needs: no input-method indicator is drawn. getTextComponentHandler /
// getTextComponent / getInputMode are behind that gate and deliberately absent.
pub struct TextComponentHandler;

impl TextComponentHandler {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponentHandler",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "isLoaded",
                "()Z",
                Self::is_loaded,
                MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn is_loaded(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::debug!("com.xce.lcdui.TextComponentHandler::isLoaded()");

        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;

    use crate::get_protos;

    /// 9a2cf5ffc9d3 died on `NoClassDefFoundError: com/xce/lcdui/TextComponentHandler` in paint.
    #[test]
    fn text_component_handler_reports_not_loaded() {
        let result = run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let loaded: bool = jvm.invoke_static("com/xce/lcdui/TextComponentHandler", "isLoaded", "()Z", ()).await?;
            assert!(!loaded);
            Ok(())
        });
        assert!(result.is_ok(), "{result:?}");
    }
}
