use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;

use rustjava_runtime::classes::java::lang::String;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.LabelComponent
pub struct LabelComponent;

impl LabelComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/LabelComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_label, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<LabelComponent>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?})");
        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        Ok(())
    }

    // The label text is not kept: lwc widgets are not drawn by this layer (Component::paint).
    // 33f3e7669599 and ca7fa8ade8ad construct one with a label.
    async fn init_with_label(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<LabelComponent>, label: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.LabelComponent::<init>({this:?}, {label:?})");
        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;
        Ok(())
    }
}
