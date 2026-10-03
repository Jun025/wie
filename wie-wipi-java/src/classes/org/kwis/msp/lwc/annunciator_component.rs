use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent;

impl AnnunciatorComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/AnnunciatorComponent",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("show", "()V", Self::show, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_size, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<AnnunciatorComponent>, a0: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({this:?}, {a0})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn show(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show()");

        Ok(())
    }

    // The status strip is never drawn here (show above is a stub), so it takes no room — the answer
    // Component gave before ShellComponent learned to answer the screen size. b475b6399684 reads the
    // strip's height twice at boot; answered with the screen height it stopped painting after two
    // frames (30-second probe, 2 of 2 runs), where 0 keeps it at ~240 paints.
    async fn get_size(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<AnnunciatorComponent>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.AnnunciatorComponent::getWidth/getHeight({this:?})");

        Ok(0)
    }
}
