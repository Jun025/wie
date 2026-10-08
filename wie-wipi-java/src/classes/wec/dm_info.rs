use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class wec.DMInfo -- handset OEM extension, like `OEMDevice`: not WIPI, in no SDK here, and named
// by exactly one corpus title (a10a1f02b41b, KTF). Its boot asks «이벤트 수신을 하시겠습니까?
// 예/아니오» (the publisher's marketing opt-in). Either answer loads this class; without it the load
// failed and the title drew «플랫폼 예외 · 새 버전을 확인하세요 · Error» and never left it.
//
// What the title asks of it, measured by running it (a missing member is a fatal «not found»):
// the static `getDMInfo()` and `gethandsetMIN()` on the result -- nothing else on either answer.
// Then it shows «수신동의/수신거부 등록 중», its `Network.connect()` fails here, and it carries on
// to the title screen and the game.
//
// MIN is the handset's phone number. There is no handset, so the answer is "" (no number) --
// not null, which the title would dereference, and not an invented number.
pub struct DMInfo;

impl DMInfo {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "wec/DMInfo",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                // Not referenced by the title -- `getDMInfo()` constructs through it.
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PRIVATE),
                JavaMethodProto::new(
                    "getDMInfo",
                    "()Lwec/DMInfo;",
                    Self::get_dm_info,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("gethandsetMIN", "()Ljava/lang/String;", Self::get_handset_min, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn get_dm_info(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<ClassInstanceRef<DMInfo>> {
        tracing::warn!("stub wec.DMInfo::getDMInfo()");

        Ok(jvm.new_class("wec/DMInfo", "()V", ()).await?.into())
    }

    async fn get_handset_min(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("stub wec.DMInfo::gethandsetMIN({this:?})");

        Ok(JavaLangString::from_rust_string(jvm, "").await?.into())
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    use super::DMInfo;

    // The path a10a1f02b41b takes after either opt-in answer: the static getter, then the number.
    #[test]
    fn get_dm_info_then_an_empty_handset_min() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let info: ClassInstanceRef<DMInfo> = jvm.invoke_static("wec/DMInfo", "getDMInfo", "()Lwec/DMInfo;", ()).await?;
            let min: ClassInstanceRef<String> = jvm
                .invoke_virtual(&info, "wec/DMInfo", "gethandsetMIN", "()Ljava/lang/String;", ())
                .await?;

            assert!(!min.is_null());
            assert_eq!(JavaLangString::to_rust_string(&jvm, &min).await?, "");

            Ok(())
        })
    }
}
