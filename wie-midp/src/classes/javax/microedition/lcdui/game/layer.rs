use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// abstract class javax.microedition.lcdui.game.Layer — position, size and visibility shared by
// Sprite and TiledLayer (JSR-118). The constructor is package-private: only those two extend it.
pub struct Layer;

impl Layer {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/Layer",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(II)V", Self::init, MethodAccessFlags::empty()),
                JavaMethodProto::new("setPosition", "(II)V", Self::set_position, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("move", "(II)V", Self::r#move, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getX", "()I", Self::get_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getY", "()I", Self::get_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getWidth", "()I", Self::get_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getHeight", "()I", Self::get_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setVisible", "(Z)V", Self::set_visible, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("isVisible", "()Z", Self::is_visible, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new_abstract(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("x", "I", FieldAccessFlags::empty()),
                JavaFieldProto::new("y", "I", FieldAccessFlags::empty()),
                JavaFieldProto::new("width", "I", FieldAccessFlags::empty()),
                JavaFieldProto::new("height", "I", FieldAccessFlags::empty()),
                JavaFieldProto::new("visible", "Z", FieldAccessFlags::empty()),
            ],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, width: i32, height: i32) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;
        jvm.put_field(&mut this, "visible", "Z", true).await?;

        Ok(())
    }

    async fn set_position(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "x", "I", x).await?;
        jvm.put_field(&mut this, "y", "I", y).await?;

        Ok(())
    }

    async fn r#move(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, dx: i32, dy: i32) -> JvmResult<()> {
        let x: i32 = jvm.get_field(&this, "x", "I").await?;
        let y: i32 = jvm.get_field(&this, "y", "I").await?;
        jvm.put_field(&mut this, "x", "I", x.wrapping_add(dx)).await?;
        jvm.put_field(&mut this, "y", "I", y.wrapping_add(dy)).await?;

        Ok(())
    }

    async fn get_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "x", "I").await
    }

    async fn get_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "y", "I").await
    }

    async fn get_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "width", "I").await
    }

    async fn get_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "height", "I").await
    }

    async fn set_visible(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, visible: bool) -> JvmResult<()> {
        jvm.put_field(&mut this, "visible", "Z", visible).await
    }

    async fn is_visible(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        jvm.get_field(&this, "visible", "Z").await
    }
}
