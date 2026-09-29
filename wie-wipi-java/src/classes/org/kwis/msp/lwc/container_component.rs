use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use rustjava_runtime::classes::java::util::Vector;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::lwc::Component;

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
    // DialogComponent and reads its component back with getComponent(0). Only the list is kept:
    // this layer still does not lay children out or draw them (Component::getX/getWidth stay 0,
    // Component::paint stays a no-op). The AromaWIPI javadoc says addComponent puts the child "on
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
    use alloc::{boxed::Box, vec::Vec};

    use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::lwc::Component, get_protos};

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
}
