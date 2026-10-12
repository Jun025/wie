use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use rustjava_runtime::classes::java::util::Vector;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::{lcdui::Graphics, lwc::Component};

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent;

impl ContainerComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/ContainerComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PROTECTED),
                JavaMethodProto::new(
                    "addComponent",
                    "(Lorg/kwis/msp/lwc/Component;)I",
                    Self::add_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    Self::get_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getNumberOfComponent", "()I", Self::get_number_of_component, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("removeComponent", "(I)V", Self::remove_component_index, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "removeComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    Self::remove_component,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("removeAllComponents", "()V", Self::remove_all_components, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("paint", "(Lorg/kwis/msp/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PROTECTED),
            ],
            fields: vec![JavaFieldProto::new("children", "Ljava/util/Vector;", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("stub org.kwis.msp.lwc.ContainerComponent::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/Component", "<init>", "()V", ()).await?;

        Ok(())
    }

    // The children are kept so getComponent(I) can hand them back — ca7fa8ade8ad builds a
    // DialogComponent and reads its component back with getComponent(0), and paint draws them. No
    // stock layout places them (see Component::layout). The AromaWIPI javadoc says addComponent puts the child "on
    // top" (맨 위에) of a stack getComponent indexes, so it goes at the end and the return value
    // is its index. Not checked: the javadoc's NullPointerException for null and
    // IllegalArgumentException for a child that already has a parent — no parent is tracked here.
    async fn add_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, component: ClassInstanceRef<Component>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::addComponent({this:?}, {component:?})");

        let children = Self::children(jvm, this).await?;
        let _: () = jvm
            .invoke_virtual(&children, "java/util/Vector", "addElement", "(Ljava/lang/Object;)V", (component,))
            .await?;
        let size: i32 = jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await?;

        Ok(size - 1)
    }

    // Out of range is null, not an exception: the AromaWIPI javadoc says so
    // ("인덱스가 유효한 영역을 벗어나는 경우에는 null을 돌려줍니다").
    async fn get_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Component>> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::getComponent({this:?}, {index})");

        let children = Self::children(jvm, this).await?;
        let size: i32 = jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await?;
        if index < 0 || index >= size {
            return Ok(None.into());
        }

        jvm.invoke_virtual(&children, "java/util/Vector", "elementAt", "(I)Ljava/lang/Object;", (index,))
            .await
    }

    async fn get_number_of_component(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::getNumberOfComponent({this:?})");

        let children = Self::children(jvm, this).await?;
        jvm.invoke_virtual(&children, "java/util/Vector", "size", "()I", ()).await
    }

    async fn remove_component_index(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeComponent({this:?}, {index})");

        let children = Self::children(jvm, this).await?;
        jvm.invoke_virtual(&children, "java/util/Vector", "removeElementAt", "(I)V", (index,))
            .await
    }

    async fn remove_component(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        component: ClassInstanceRef<Component>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeComponent({this:?}, {component:?})");

        let children = Self::children(jvm, this).await?;
        let _: bool = jvm
            .invoke_virtual(&children, "java/util/Vector", "removeElement", "(Ljava/lang/Object;)Z", (component,))
            .await?;

        Ok(())
    }

    // The javadoc: "모든 컴포넌트를 삭제합니다." ca7fa8ade8ad calls it on a ShellComponent, which
    // inherits it from here.
    async fn remove_all_components(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::removeAllComponents({this:?})");

        let children = Self::children(jvm, this).await?;
        jvm.invoke_virtual(&children, "java/util/Vector", "removeAllElements", "()V", ()).await
    }

    // A container draws its children, each in its own box: validate first (the javadoc: validate →
    // layout, which a guest subclass may override to configure the children), then every child's paint
    // translated to its (x, y). d448aee68157's form paints its dialog and calls this as super.paint, and
    // its layout() is what puts the name box at (56, 93). ponytail: no clip to the child's box — a
    // TextComponent draws inside its own; clip when a child is measured drawing past it.
    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.lwc.ContainerComponent::paint({this:?}, {g:?})");

        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/lwc/Component", "validate", "()V", ()).await?;
        Self::paint_children(jvm, this.instance.into(), g).await
    }

    /// Every child's paint, translated to the child's (x, y). Also what a ShellComponent draws.
    pub async fn paint_children(jvm: &Jvm, container: ClassInstanceRef<Component>, g: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        let count: i32 = jvm
            .invoke_virtual(&container, "org/kwis/msp/lwc/ContainerComponent", "getNumberOfComponent", "()I", ())
            .await?;
        for i in 0..count {
            let child: ClassInstanceRef<Component> = jvm
                .invoke_virtual(
                    &container,
                    "org/kwis/msp/lwc/ContainerComponent",
                    "getComponent",
                    "(I)Lorg/kwis/msp/lwc/Component;",
                    (i,),
                )
                .await?;
            if child.is_null() {
                continue;
            }
            let x: i32 = jvm.get_field(&child, "x", "I").await?;
            let y: i32 = jvm.get_field(&child, "y", "I").await?;
            let _: () = jvm
                .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "translate", "(II)V", (x, y))
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &child,
                    "org/kwis/msp/lwc/Component",
                    "paint",
                    "(Lorg/kwis/msp/lcdui/Graphics;)V",
                    (g.clone(),),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&g, "org/kwis/msp/lcdui/Graphics", "translate", "(II)V", (-x, -y))
                .await?;
        }

        Ok(())
    }

    // Created on first use, not in <init>: ShellComponent's constructors chain to Component.<init>
    // directly, and a guest subclass may too, so an eagerly created list would be missing on
    // exactly those containers.
    async fn children(jvm: &Jvm, mut this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Vector>> {
        let children: ClassInstanceRef<Vector> = jvm.get_field(&this, "children", "Ljava/util/Vector;").await?;
        if !children.is_null() {
            return Ok(children);
        }

        let children: ClassInstanceRef<Vector> = jvm.new_class("java/util/Vector", "()V", ()).await?.into();
        jvm.put_field(&mut this, "children", "Ljava/util/Vector;", children.clone()).await?;

        Ok(children)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec, vec::Vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};
    use jvm_class_proto::JavaMethodProto;
    use jvm_types::{ClassAccessFlags, MethodAccessFlags};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
    use wie_util::Result;

    use crate::{
        classes::org::kwis::msp::{
            lcdui::{Graphics, Image},
            lwc::Component,
        },
        get_protos,
    };

    const CONTAINER: &str = "org/kwis/msp/lwc/ContainerComponent";

    async fn label(jvm: &Jvm) -> JvmResult<ClassInstanceRef<Component>> {
        Ok(jvm.new_class("org/kwis/msp/lwc/LabelComponent", "()V", ()).await?.into())
    }

    async fn get(jvm: &Jvm, container: &ClassInstanceRef<Component>, index: i32) -> JvmResult<ClassInstanceRef<Component>> {
        jvm.invoke_virtual(container, CONTAINER, "getComponent", "(I)Lorg/kwis/msp/lwc/Component;", (index,))
            .await
    }

    async fn count(jvm: &Jvm, container: &ClassInstanceRef<Component>) -> JvmResult<i32> {
        jvm.invoke_virtual(container, CONTAINER, "getNumberOfComponent", "()I", ()).await
    }

    /// ca7fa8ade8ad's wall: children were dropped, so getComponent had nothing to return. On a
    /// ShellComponent on purpose — its constructors skip ContainerComponent.<init>, which is why the
    /// list is made on first use.
    #[test]
    fn added_children_come_back_by_index_and_leave_on_remove() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let shell: ClassInstanceRef<Component> = jvm.new_class("org/kwis/msp/lwc/ShellComponent", "()V", ()).await?.into();
            assert_eq!(count(&jvm, &shell).await?, 0);
            assert!(get(&jvm, &shell, 0).await?.is_null());

            let mut labels = Vec::new();
            for expected in 0..3 {
                let child = label(&jvm).await?;
                let index: i32 = jvm
                    .invoke_virtual(&shell, CONTAINER, "addComponent", "(Lorg/kwis/msp/lwc/Component;)I", (child.clone(),))
                    .await?;
                assert_eq!(index, expected);
                labels.push(child);
            }
            assert_eq!(count(&jvm, &shell).await?, 3);
            for (index, child) in labels.iter().enumerate() {
                assert_eq!(get(&jvm, &shell, index as i32).await?.identity(), child.identity());
            }
            // Out of range is null per the javadoc, on both sides.
            assert!(get(&jvm, &shell, 3).await?.is_null());
            assert!(get(&jvm, &shell, -1).await?.is_null());

            let _: () = jvm.invoke_virtual(&shell, CONTAINER, "removeComponent", "(I)V", (0,)).await?;
            let _: () = jvm
                .invoke_virtual(
                    &shell,
                    CONTAINER,
                    "removeComponent",
                    "(Lorg/kwis/msp/lwc/Component;)V",
                    (labels[2].clone(),),
                )
                .await?;
            assert_eq!(count(&jvm, &shell).await?, 1);
            assert_eq!(get(&jvm, &shell, 0).await?.identity(), labels[1].identity());

            // ca7fa8ade8ad's next wall: removeAllComponents()V on a ShellComponent.
            let _: () = jvm.invoke_virtual(&shell, CONTAINER, "removeAllComponents", "()V", ()).await?;
            assert_eq!(count(&jvm, &shell).await?, 0);
            assert!(get(&jvm, &shell, 0).await?.is_null());

            Ok(())
        })
    }

    /// The exact call ca7fa8ade8ad makes: build a dialog around a label, then ask for child 0.
    #[test]
    fn dialog_component_holds_its_component_as_child_zero() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let content = label(&jvm).await?;
            let title: ClassInstanceRef<String> = None.into();
            let dialog: ClassInstanceRef<Component> = jvm
                .new_class(
                    "org/kwis/msp/lwc/DialogComponent",
                    "(Lorg/kwis/msp/lwc/Component;Ljava/lang/String;I)V",
                    (content.clone(), title, 1),
                )
                .await?
                .into();

            assert_eq!(count(&jvm, &dialog).await?, 1);
            assert_eq!(get(&jvm, &dialog, 0).await?.identity(), content.identity());

            Ok(())
        })
    }

    // d448aee68157's form, as measured: a FormComponent subclass whose layout() configures each text
    // box to a fixed place — but only one that already has a size (a 0 × 0 box is configured as is).
    const NAME_FORM: &str = "test/NameForm";
    const PLACES: [(i32, i32, i32, i32); 2] = [(4, 2, 24, 12), (4, 18, 24, 12)];

    fn name_form_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: NAME_FORM,
            parent_class: Some("org/kwis/msp/lwc/FormComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", name_form_init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("layout", "()V", name_form_layout, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn name_form_init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Component>) -> JvmResult<()> {
        jvm.invoke_special(&this, "org/kwis/msp/lwc/FormComponent", "<init>", "()V", ()).await
    }

    async fn name_form_layout(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Component>) -> JvmResult<()> {
        for (i, (x, y, w, h)) in PLACES.into_iter().enumerate() {
            let child = get(jvm, &this, i as i32).await?;
            let mut size = 0;
            for getter in ["getX", "getY", "getWidth", "getHeight"] {
                size |= jvm
                    .invoke_virtual::<_, i32>(&child, "org/kwis/msp/lwc/Component", getter, "()I", ())
                    .await?;
            }
            let place = if size == 0 { (0, 0, 0, 0) } else { (x, y, w, h) };
            let _: () = jvm
                .invoke_virtual(
                    &child,
                    "org/kwis/msp/lwc/Component",
                    "configure",
                    "(IIIII)V",
                    (place.0, place.1, place.2, place.3, 3),
                )
                .await?;
        }

        Ok(())
    }

    /// d448aee68157's name box took keys but was never drawn (Component::paint was a stub), so the
    /// title looked like it took no input. The form paints itself and calls super.paint; that has to
    /// validate (→ the form's own layout(), which places the box) and draw each box in its place: the
    /// background, then the text. A password box draws its text as stars. Reverting any of
    /// configure, validate → layout, the text box's initial height, ContainerComponent.paint or
    /// TextComponent.paint leaves the boxes undrawn.
    #[test]
    fn a_form_lays_out_through_its_own_layout_and_paints_its_text_boxes() -> Result<()> {
        run_jvm_test(
            Box::new([wie_midp::get_protos().into(), get_protos().into(), Box::new([name_form_proto()])]),
            |jvm| async move {
                const BG: i32 = 0xff24aa;
                let form: ClassInstanceRef<Component> = jvm.new_class(NAME_FORM, "()V", ()).await?.into();
                for (constraint, text) in [(2, "ab"), (0, "**")] {
                    let null: ClassInstanceRef<()> = ClassInstanceRef::new(None);
                    let text_box: ClassInstanceRef<Component> = jvm
                        .new_class("org/kwis/msp/lwc/TextBoxComponent", "(Ljava/lang/String;I)V", (null, constraint))
                        .await?
                        .into();
                    let _: () = jvm
                        .invoke_virtual(&text_box, "org/kwis/msp/lwc/Component", "setBackground", "(I)V", (BG,))
                        .await?;
                    let text = JavaLangString::from_rust_string(&jvm, text).await?;
                    let _: () = jvm
                        .invoke_virtual(&text_box, "org/kwis/msp/lwc/TextComponent", "setString", "(Ljava/lang/String;)V", (text,))
                        .await?;
                    let _: i32 = jvm
                        .invoke_virtual(&form, CONTAINER, "addComponent", "(Lorg/kwis/msp/lwc/Component;)I", (text_box,))
                        .await?;
                }

                let image: ClassInstanceRef<Image> = jvm
                    .invoke_static("org/kwis/msp/lcdui/Image", "createImage", "(II)Lorg/kwis/msp/lcdui/Image;", (32, 32))
                    .await?;
                let g: ClassInstanceRef<Graphics> = jvm
                    .invoke_virtual(&image, "org/kwis/msp/lcdui/Image", "getGraphics", "()Lorg/kwis/msp/lcdui/Graphics;", ())
                    .await?;
                let _: () = jvm
                    .invoke_virtual(
                        &form,
                        "org/kwis/msp/lwc/Component",
                        "paint",
                        "(Lorg/kwis/msp/lcdui/Graphics;)V",
                        (g.clone(),),
                    )
                    .await?;

                let pixel = |x: i32, y: i32| {
                    let (jvm, g) = (jvm.clone(), g.clone());
                    async move {
                        jvm.invoke_virtual::<_, i32>(&g, "org/kwis/msp/lcdui/Graphics", "getPixel", "(II)I", (x, y))
                            .await
                    }
                };
                let mut boxes = Vec::new();
                for (x, y, w, h) in PLACES {
                    let mut rows = Vec::new();
                    for dy in 0..h {
                        for dx in 0..w {
                            rows.push(pixel(x + dx, y + dy).await? & 0xffffff);
                        }
                    }
                    assert!(
                        rows.iter().filter(|&&p| p == BG).count() > rows.len() / 2,
                        "box at ({x}, {y}) has no background"
                    );
                    assert!(rows.iter().any(|&p| p != BG), "box at ({x}, {y}) has no text");
                    boxes.push(rows);
                }
                assert_eq!(boxes[0], boxes[1], "the password box draws one star per character");
                // Outside the boxes nothing was drawn.
                assert_eq!(pixel(0, 0).await? & 0xffffff, 0);
                assert_eq!(pixel(31, 31).await? & 0xffffff, 0);

                Ok(())
            },
        )
    }

    /// configure keeps what its mask names (javadoc: POS_MASK moves, SIZE_MASK resizes) and the
    /// getters read it back.
    #[test]
    fn configure_keeps_position_and_size_by_mask() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let c = label(&jvm).await?;
            let configure = |x: i32, y: i32, w: i32, h: i32, mask: i32| {
                let (jvm, c) = (jvm.clone(), c.clone());
                async move {
                    jvm.invoke_virtual::<_, ()>(&c, "org/kwis/msp/lwc/Component", "configure", "(IIIII)V", (x, y, w, h, mask))
                        .await
                }
            };
            let read = || {
                let (jvm, c) = (jvm.clone(), c.clone());
                async move {
                    let mut v = [0; 4];
                    for (slot, getter) in v.iter_mut().zip(["getX", "getY", "getWidth", "getHeight"]) {
                        *slot = jvm.invoke_virtual(&c, "org/kwis/msp/lwc/Component", getter, "()I", ()).await?;
                    }
                    JvmResult::Ok(v)
                }
            };

            configure(1, 2, 3, 4, 3).await?;
            assert_eq!(read().await?, [1, 2, 3, 4]);
            configure(5, 6, 7, 8, 1).await?;
            assert_eq!(read().await?, [5, 6, 3, 4]);
            configure(9, 9, 7, 8, 2).await?;
            assert_eq!(read().await?, [5, 6, 7, 8]);

            Ok(())
        })
    }
}
