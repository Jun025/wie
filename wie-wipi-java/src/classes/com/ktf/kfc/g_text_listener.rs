use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.ktf.kfc.GTextListener -- despite the name, an object a GTextField hands out and the title
// configures: f07cbc782828 calls `setIMEModes([I)V` on what `getGTextListener()` returned, so it is
// a class with that method, not an interface the title implements.
// ponytail: the modes are not kept — the text field does no input-method switching of its own.
pub struct GTextListener;

impl GTextListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GTextListener",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setIMEModes", "([I)V", Self::set_ime_modes, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GTextListener::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await
    }

    async fn set_ime_modes(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, modes: ClassInstanceRef<Array<i32>>) -> JvmResult<()> {
        tracing::warn!("stub com.ktf.kfc.GTextListener::setIMEModes({this:?}, {modes:?})");

        Ok(())
    }
}
