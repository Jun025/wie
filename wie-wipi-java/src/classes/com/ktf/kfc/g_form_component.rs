use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lwc::Component;

// class com.ktf.kfc.GFormComponent -- the form ChoiceText is placed on. Same source as ChoiceText:
// 0c67145b11df resolves `<init>()V` and `addComponent(Lorg/kwis/msp/lwc/Component;IIII)I`, adds four
// children (two text boxes, a ChoiceText, a button) and then passes the form itself to
// `ShellComponent.addComponent(Lorg/kwis/msp/lwc/Component;)I`. A Component that holds children is
// a ContainerComponent here; that it is the real parent is an inference, not a document.
pub struct GFormComponent;

impl GFormComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GFormComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;IIII)I",
                    Self::add_component,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GFormComponent::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/ContainerComponent", "<init>", "()V", ())
            .await
    }

    // The four ints read as a box (x, y, w, h): the title passes (110,135,60,17), (110,160,60,17),
    // (120,183,36,15) and (107,300,22,8). They are dropped -- this layer lays nothing out -- and the
    // child is kept the way ContainerComponent keeps one, so the return value is its index.
    #[allow(clippy::too_many_arguments)] // the arity is the Java descriptor's
    async fn add_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub com.ktf.kfc.GFormComponent::addComponent({this:?}, {component:?}, {x}, {y}, {width}, {height})");

        jvm.invoke_special(
            &this,
            "org/kwis/msp/lwc/ContainerComponent",
            "addComponent",
            "(Lorg/kwis/msp/lwc/Component;)I",
            (component,),
        )
        .await
    }
}
