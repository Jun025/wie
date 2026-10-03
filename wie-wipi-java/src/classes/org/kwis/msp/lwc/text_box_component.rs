use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use super::TextComponent;

// class org.kwis.msp.lwc.TextBoxComponent
pub struct TextBoxComponent;

impl TextBoxComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/TextBoxComponent",
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_max_length, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextBoxComponent::<init>({this:?}, {data:?}, {constraint:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/TextComponent", "<init>", "()V", ()).await?;
        TextComponent::set_constraint(jvm, &mut this, constraint).await
    }

    // (text, maxLength, constraint), following MIDP's TextBox(title, text, maxSize, constraints)
    // ordering -- the middle int is routed to setMaxLength, which 붕어빵타이쿤3 also calls on this
    // class directly. setMaxLength only records its value, but the constraint now picks the input
    // mode (1/2/5 type digits), so a swapped reading would show as a field typing the wrong kind of
    // character. Unmeasured: no title reaching this constructor has been typed into.
    async fn init_with_max_length(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        max_length: i32,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.TextBoxComponent::<init>({this:?}, {data:?}, {max_length}, {constraint:?})");

        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/TextBoxComponent",
                "<init>",
                "(Ljava/lang/String;I)V",
                (data, constraint),
            )
            .await?;

        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/lwc/TextComponent", "setMaxLength", "(I)V", (max_length,))
            .await?;

        Ok(())
    }
}
