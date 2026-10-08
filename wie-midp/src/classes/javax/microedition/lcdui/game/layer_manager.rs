use alloc::{vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Graphics, game::Layer};

// class javax.microedition.lcdui.game.LayerManager (JSR-118)
//
// Index 0 is the layer nearest the viewer, so paint goes from the last index to the first.
pub struct LayerManager;

const GRAPHICS: &str = "javax/microedition/lcdui/Graphics";

impl LayerManager {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/LayerManager",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "append",
                    "(Ljavax/microedition/lcdui/game/Layer;)V",
                    Self::append,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "insert",
                    "(Ljavax/microedition/lcdui/game/Layer;I)V",
                    Self::insert,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "remove",
                    "(Ljavax/microedition/lcdui/game/Layer;)V",
                    Self::remove,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getLayerAt",
                    "(I)Ljavax/microedition/lcdui/game/Layer;",
                    Self::get_layer_at,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("getSize", "()I", Self::get_size, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setViewWindow", "(IIII)V", Self::set_view_window, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;II)V",
                    Self::paint,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![
                JavaFieldProto::new("layers", "[Ljavax/microedition/lcdui/game/Layer;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("viewX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("viewY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("viewWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("viewHeight", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        Self::store(jvm, &mut this, Vec::new()).await?;
        jvm.put_field(&mut this, "viewWidth", "I", i32::MAX).await?;
        jvm.put_field(&mut this, "viewHeight", "I", i32::MAX).await
    }

    // ponytail: the whole list is reloaded and rewritten on each change; games hold a handful of
    // layers and change them rarely, so a size field with spare capacity would buy nothing.
    async fn load(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<Vec<ClassInstanceRef<Layer>>> {
        let layers: ClassInstanceRef<Array<ClassInstanceRef<Layer>>> =
            jvm.get_field(this, "layers", "[Ljavax/microedition/lcdui/game/Layer;").await?;
        let length = jvm.array_length(&layers).await?;

        jvm.load_array(&layers, 0, length).await
    }

    async fn store(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, layers: Vec<ClassInstanceRef<Layer>>) -> JvmResult<()> {
        let mut array = jvm.instantiate_array("Ljavax/microedition/lcdui/game/Layer;", layers.len()).await?;
        jvm.store_array(&mut array, 0, layers).await?;

        jvm.put_field(this, "layers", "[Ljavax/microedition/lcdui/game/Layer;", array).await
    }

    async fn without(jvm: &Jvm, this: &ClassInstanceRef<Self>, layer: &ClassInstanceRef<Layer>) -> JvmResult<Vec<ClassInstanceRef<Layer>>> {
        if layer.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "layer is null").await);
        }
        let mut layers = Self::load(jvm, this).await?;
        layers.retain(|l| l.instance != layer.instance);

        Ok(layers)
    }

    async fn append(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, layer: ClassInstanceRef<Layer>) -> JvmResult<()> {
        let mut layers = Self::without(jvm, &this, &layer).await?;
        layers.push(layer);

        Self::store(jvm, &mut this, layers).await
    }

    async fn insert(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, layer: ClassInstanceRef<Layer>, index: i32) -> JvmResult<()> {
        let mut layers = Self::without(jvm, &this, &layer).await?;
        if index < 0 || index as usize > layers.len() {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "index out of range").await);
        }
        layers.insert(index as usize, layer);

        Self::store(jvm, &mut this, layers).await
    }

    async fn remove(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, layer: ClassInstanceRef<Layer>) -> JvmResult<()> {
        let layers = Self::without(jvm, &this, &layer).await?;

        Self::store(jvm, &mut this, layers).await
    }

    async fn get_layer_at(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<ClassInstanceRef<Layer>> {
        let layers = Self::load(jvm, &this).await?;
        match layers.into_iter().nth(index.max(0) as usize) {
            Some(layer) if index >= 0 => Ok(layer),
            _ => Err(jvm.exception("java/lang/IndexOutOfBoundsException", "index out of range").await),
        }
    }

    async fn get_size(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        let layers: ClassInstanceRef<Array<ClassInstanceRef<Layer>>> =
            jvm.get_field(&this, "layers", "[Ljavax/microedition/lcdui/game/Layer;").await?;

        Ok(jvm.array_length(&layers).await? as i32)
    }

    async fn set_view_window(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        if width < 0 || height < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "negative view size").await);
        }
        jvm.put_field(&mut this, "viewX", "I", x).await?;
        jvm.put_field(&mut this, "viewY", "I", y).await?;
        jvm.put_field(&mut this, "viewWidth", "I", width).await?;
        jvm.put_field(&mut this, "viewHeight", "I", height).await
    }

    // As the reference implementation: the view window's top-left lands at (x, y), drawing is
    // clipped to the window, and the Graphics' translation and clip are restored afterwards.
    async fn paint(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        graphics: ClassInstanceRef<Graphics>,
        x: i32,
        y: i32,
    ) -> JvmResult<()> {
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }
        let view_x: i32 = jvm.get_field(&this, "viewX", "I").await?;
        let view_y: i32 = jvm.get_field(&this, "viewY", "I").await?;
        let view_width: i32 = jvm.get_field(&this, "viewWidth", "I").await?;
        let view_height: i32 = jvm.get_field(&this, "viewHeight", "I").await?;

        let clip_x: i32 = jvm.invoke_virtual(&graphics, GRAPHICS, "getClipX", "()I", ()).await?;
        let clip_y: i32 = jvm.invoke_virtual(&graphics, GRAPHICS, "getClipY", "()I", ()).await?;
        let clip_width: i32 = jvm.invoke_virtual(&graphics, GRAPHICS, "getClipWidth", "()I", ()).await?;
        let clip_height: i32 = jvm.invoke_virtual(&graphics, GRAPHICS, "getClipHeight", "()I", ()).await?;

        let (dx, dy) = (x.wrapping_sub(view_x), y.wrapping_sub(view_y));
        let _: () = jvm.invoke_virtual(&graphics, GRAPHICS, "translate", "(II)V", (dx, dy)).await?;
        // the default window is Integer.MAX_VALUE wide; translated it would overflow, and it clips nothing
        if view_width != i32::MAX || view_height != i32::MAX {
            let _: () = jvm
                .invoke_virtual(&graphics, GRAPHICS, "clipRect", "(IIII)V", (view_x, view_y, view_width, view_height))
                .await?;
        }

        for layer in Self::load(jvm, &this).await?.into_iter().rev() {
            let visible: bool = jvm.get_field(&layer, "visible", "Z").await?;
            if visible {
                let _: () = jvm
                    .invoke_virtual(
                        &layer,
                        "javax/microedition/lcdui/game/Layer",
                        "paint",
                        "(Ljavax/microedition/lcdui/Graphics;)V",
                        (graphics.clone(),),
                    )
                    .await?;
            }
        }

        let _: () = jvm
            .invoke_virtual(&graphics, GRAPHICS, "translate", "(II)V", (dx.wrapping_neg(), dy.wrapping_neg()))
            .await?;
        let _: () = jvm
            .invoke_virtual(&graphics, GRAPHICS, "setClip", "(IIII)V", (clip_x, clip_y, clip_width, clip_height))
            .await?;

        Ok(())
    }
}
