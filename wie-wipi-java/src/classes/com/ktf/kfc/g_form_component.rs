use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::{
    net::wie::ShellCard,
    org::kwis::msp::{
        lcdui::{Graphics, Image},
        lwc::Component,
    },
};

// class com.ktf.kfc.GFormComponent -- the form ChoiceText is placed on. Same source as ChoiceText:
// 0c67145b11df resolves `<init>()V` and `addComponent(Lorg/kwis/msp/lwc/Component;IIII)I`, adds four
// children (two text boxes, a ChoiceText, a button) and then passes the form itself to
// `ShellComponent.addComponent(Lorg/kwis/msp/lwc/Component;)I`. The parent is FormComponent: the same
// title calls `FormComponent.setFocus(Lorg/kwis/msp/lwc/Component;)V` on this form, and on KTF a method
// the receiver's class does not inherit has no vtable slot — under ContainerComponent the call jumped
// to address 0. The class is still not in the reference zip, so that call is the only evidence.
//
// The box each child is added with is the only placement this layer has, so the form keeps it and
// paints its widgets there: a cleared box with the typed text or the selected choice, the button's
// image, and a red rectangle around the focus. The title draws the labels and background itself.
pub struct GFormComponent;

// MIDP Graphics.TOP | LEFT, which org.kwis.msp.lcdui.Graphics forwards as is.
const TOP_LEFT: i32 = 0x10 | 0x04;

impl GFormComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/ktf/kfc/GFormComponent",
            parent_class: Some("org/kwis/msp/lwc/FormComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;IIII)I",
                    Self::add_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
            ],
            // (x, y, w, h) per child, in child order.
            // ponytail: not kept in step with removeComponent — the title never removes from its form.
            fields: vec![JavaFieldProto::new("bounds", "[I", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GFormComponent::<init>({this:?})");

        jvm.invoke_special(&this, "org/kwis/msp/lwc/FormComponent", "<init>", "()V", ()).await
    }

    // The four ints read as a box (x, y, w, h): the title passes (110,135,60,17), (110,160,60,17),
    // (120,183,36,15) and (107,300,22,8). The child is kept the way ContainerComponent keeps one,
    // so the return value is its index.
    #[allow(clippy::too_many_arguments)] // the arity is the Java descriptor's
    async fn add_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.ktf.kfc.GFormComponent::addComponent({this:?}, {component:?}, {x}, {y}, {width}, {height})");

        let index: i32 = jvm
            .invoke_special(
                &this,
                "org/kwis/msp/lwc/ContainerComponent",
                "addComponent",
                "(Lorg/kwis/msp/lwc/Component;)I",
                (component,),
            )
            .await?;

        let mut bounds = Self::bounds(jvm, &this).await?;
        bounds.resize(index as usize * 4, 0);
        bounds.extend([x, y, width, height]);
        let mut array = jvm.instantiate_array("I", bounds.len()).await?;
        jvm.store_array(&mut array, 0, bounds).await?;
        jvm.put_field(&mut this, "bounds", "[I", array).await?;

        Ok(index)
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("com.ktf.kfc.GFormComponent::paint({this:?}, {g:?})");

        let bounds = Self::bounds(jvm, &this).await?;
        let focus = ShellCard::focus(jvm).await?;
        for (index, b) in bounds.as_chunks::<4>().0.iter().enumerate() {
            let child: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &this,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    (index as i32,),
                )
                .await?;
            if child.is_null() {
                continue;
            }
            let (x, y, w, h) = (b[0], b[1], b[2], b[3]);
            let focused = !focus.is_null() && focus.identity() == child.identity();

            if jvm.is_instance(&**child, "org/kwis/msp/lwc/ButtonComponent") {
                let image: ClassInstanceRef<Image> = jvm.get_field(&child, "img", "Lorg/kwis/msp/lcdui/Image;").await?;
                if !image.is_null() {
                    let _: () = jvm
                        .invoke_virtual(
                            &g,
                            "org/kwis/msp/lcdui/Graphics",
                            "drawImage",
                            "(Lorg/kwis/msp/lcdui/Image;III)V",
                            (image, x, y, TOP_LEFT),
                        )
                        .await?;
                }
            } else {
                // The title's own card does not redraw this area when the form repaints, so each box
                // is cleared here — otherwise the last focus rectangle and the last choice stay on screen.
                Self::set_color(jvm, &g, 0xffffff).await?;
                let _: () = jvm
                    .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "fillRect", "(IIII)V", (x, y, w, h))
                    .await?;
                Self::set_color(jvm, &g, 0).await?;
                if jvm.is_instance(&**child, "org/kwis/msp/lwc/TextComponent") {
                    let typed: ClassInstanceRef<Array<JavaChar>> = jvm.get_field(&child, "m_td", "[C").await?;
                    if !typed.is_null() {
                        let length = jvm.array_length(&typed).await? as i32;
                        let _: () = jvm
                            .invoke_virtual(
                                &g,
                                "org/kwis/msp/lcdui/Graphics",
                                "drawChars",
                                "([CIIIII)V",
                                (typed, 0, length, x + 2, y + 1, TOP_LEFT),
                            )
                            .await?;
                    }
                } else if jvm.is_instance(&**child, "com/ktf/kfc/ChoiceText") {
                    let choices: ClassInstanceRef<Array<String>> = jvm.get_field(&child, "choices", "[Ljava/lang/String;").await?;
                    let selected: i32 = jvm.get_field(&child, "selected", "I").await?;
                    if !choices.is_null() && (selected as usize) < jvm.array_length(&choices).await? {
                        let choice: ClassInstanceRef<String> = jvm.load_array(&choices, selected as usize, 1).await?.remove(0);
                        if !choice.is_null() {
                            let _: () = jvm
                                .invoke_virtual(
                                    &g,
                                    "org/kwis/msp/lcdui/Graphics",
                                    "drawString",
                                    "(Ljava/lang/String;III)V",
                                    (choice, x + 2, y + 1, TOP_LEFT),
                                )
                                .await?;
                        }
                    }
                }
            }

            // Around a box, which every paint redraws in black or red. A button has no box: its mark
            // goes on the image's edge, so the next paint's image covers it once the focus moves on.
            let is_button = jvm.is_instance(&**child, "org/kwis/msp/lwc/ButtonComponent");
            if focused || !is_button {
                Self::set_color(jvm, &g, if focused { 0xff0000 } else { 0 }).await?;
                let rect = if is_button { (x, y, w - 1, h - 1) } else { (x - 1, y - 1, w + 1, h + 1) };
                let _: () = jvm.invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "drawRect", "(IIII)V", rect).await?;
            }
        }

        Ok(())
    }

    async fn set_color(jvm: &Jvm, g: &ClassInstanceRef<Graphics>, color: i32) -> JvmResult<()> {
        jvm.invoke_virtual(g, "org/kwis/msp/lcdui/Graphics", "setColor", "(I)V", (color,)).await
    }

    async fn bounds(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<i32>> {
        let array: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "bounds", "[I").await?;
        if array.is_null() {
            return Ok(Vec::new());
        }
        let length = jvm.array_length(&array).await?;
        jvm.load_array(&array, 0, length).await
    }
}
