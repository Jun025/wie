use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Font, lwc::TextComponent};

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
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_height, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // A text box starts one line tall: the javadoc's (String, int, int) constructor calls its h
    // 「초기 컴포넌트의 높이값」, so this one has an initial height too, and the API-only class files do
    // not give it — one line of the default font plus a pixel above and below (TextComponent.paint
    // centres the text in it). The width stays 0 until something configures the box. d448aee68157's
    // form layout() only places a box that already has a size, so a 0 × 0 box was never placed.
    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::<init>({this:?}, {data:?}, {constraint:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/TextComponent", "<init>", "()V", ()).await?;
        TextComponent::set_constraint(jvm, &mut this, constraint).await?;

        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("org/kwis/msp/lcdui/Font", "getDefaultFont", "()Lorg/kwis/msp/lcdui/Font;", ())
            .await?;
        let font_height: i32 = jvm.invoke_virtual(&font, "org/kwis/msp/lcdui/Font", "getHeight", "()I", ()).await?;
        jvm.put_field(&mut this, "h", "I", font_height + 2).await
    }

    // (data, constraints, h) — the javadoc's order: the last int is the box's initial height.
    // ponytail: the javadoc grows the box past h when the text needs more lines; one line is all
    // TextComponent.paint draws, so h is kept as given.
    async fn init_with_height(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<TextBoxComponent>,
        data: ClassInstanceRef<String>,
        constraint: i32,
        h: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.TextBoxComponent::<init>({this:?}, {data:?}, {constraint}, {h})");

        let _: () = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/TextBoxComponent",
                "<init>",
                "(Ljava/lang/String;I)V",
                (data, constraint),
            )
            .await?;
        jvm.put_field(&mut this, "h", "I", h).await
    }
}
