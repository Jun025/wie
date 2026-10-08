use alloc::{boxed::Box, vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::Image as BackendImage;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{
    Graphics, Image,
    game::TiledLayer,
    graphics::{draw_transformed, region_source_point},
};

// class javax.microedition.lcdui.game.Sprite (JSR-118)
//
// Frames are cut from `image` left-to-right, top-to-bottom. `seq` is null while the default
// sequence (0..rawFrames) is in force. `transform` is a Sprite.TRANS_* value; the sprite keeps
// its reference pixel in place when the transform changes, as the reference implementation does.
pub struct Sprite;

const LAYER: &str = "javax/microedition/lcdui/game/Layer";

impl Sprite {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/Sprite",
            parent_class: Some(LAYER),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/Image;)V",
                    Self::init_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("<init>", "(Ljavax/microedition/lcdui/Image;II)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljavax/microedition/lcdui/game/Sprite;)V",
                    Self::init_copy,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "setImage",
                    "(Ljavax/microedition/lcdui/Image;II)V",
                    Self::set_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setFrame", "(I)V", Self::set_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getFrame", "()I", Self::get_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRawFrameCount", "()I", Self::get_raw_frame_count, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getFrameSequenceLength",
                    "()I",
                    Self::get_frame_sequence_length,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("nextFrame", "()V", Self::next_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("prevFrame", "()V", Self::prev_frame, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setFrameSequence", "([I)V", Self::set_frame_sequence, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("defineReferencePixel", "(II)V", Self::define_reference_pixel, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setRefPixelPosition", "(II)V", Self::set_ref_pixel_position, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRefPixelX", "()I", Self::get_ref_pixel_x, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRefPixelY", "()I", Self::get_ref_pixel_y, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setTransform", "(I)V", Self::set_transform, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "defineCollisionRectangle",
                    "(IIII)V",
                    Self::define_collision_rectangle,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/Sprite;Z)Z",
                    Self::collides_with_sprite,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/TiledLayer;Z)Z",
                    Self::collides_with_tiled_layer,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/Image;IIZ)Z",
                    Self::collides_with_image,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("frameWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("frameHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("rawFrames", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("seq", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("seqIndex", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("transform", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("refX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("refY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collX", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collY", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("collHeight", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init_image(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>) -> JvmResult<()> {
        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let width: i32 = jvm.get_field(&image, "w", "I").await?;
        let height: i32 = jvm.get_field(&image, "h", "I").await?;

        Self::init(jvm, context, this, image, width, height).await
    }

    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        frame_width: i32,
        frame_height: i32,
    ) -> JvmResult<()> {
        let _: () = jvm.invoke_special(&this, LAYER, "<init>", "(II)V", (frame_width, frame_height)).await?;
        Self::load_image(jvm, &mut this, image, frame_width, frame_height).await?;

        Ok(())
    }

    async fn init_copy(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, other: ClassInstanceRef<Self>) -> JvmResult<()> {
        if other.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "sprite is null").await);
        }
        let width: i32 = jvm.get_field(&other, "width", "I").await?;
        let height: i32 = jvm.get_field(&other, "height", "I").await?;
        let _: () = jvm.invoke_special(&this, LAYER, "<init>", "(II)V", (width, height)).await?;

        for name in [
            "x",
            "y",
            "frameWidth",
            "frameHeight",
            "rawFrames",
            "seqIndex",
            "transform",
            "refX",
            "refY",
            "collX",
            "collY",
            "collWidth",
            "collHeight",
        ] {
            let value: i32 = jvm.get_field(&other, name, "I").await?;
            jvm.put_field(&mut this, name, "I", value).await?;
        }
        let visible: bool = jvm.get_field(&other, "visible", "Z").await?;
        jvm.put_field(&mut this, "visible", "Z", visible).await?;
        let image: ClassInstanceRef<Image> = jvm.get_field(&other, "image", "Ljavax/microedition/lcdui/Image;").await?;
        jvm.put_field(&mut this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        let seq: ClassInstanceRef<Array<i32>> = jvm.get_field(&other, "seq", "[I").await?;
        if !seq.is_null() {
            let values: Vec<i32> = jvm.load_array(&seq, 0, jvm.array_length(&seq).await?).await?;
            Self::store_sequence(jvm, &mut this, &values).await?;
        }

        Ok(())
    }

    async fn load_image(
        jvm: &Jvm,
        this: &mut ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        frame_width: i32,
        frame_height: i32,
    ) -> JvmResult<()> {
        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let image_width: i32 = jvm.get_field(&image, "w", "I").await?;
        let image_height: i32 = jvm.get_field(&image, "h", "I").await?;
        if frame_width < 1 || frame_height < 1 || image_width % frame_width != 0 || image_height % frame_height != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid frame size").await);
        }

        let raw_frames = (image_width / frame_width) * (image_height / frame_height);
        let old_raw_frames: i32 = jvm.get_field(this, "rawFrames", "I").await?;

        jvm.put_field(this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(this, "frameWidth", "I", frame_width).await?;
        jvm.put_field(this, "frameHeight", "I", frame_height).await?;
        jvm.put_field(this, "rawFrames", "I", raw_frames).await?;
        jvm.put_field(this, "collX", "I", 0).await?;
        jvm.put_field(this, "collY", "I", 0).await?;
        jvm.put_field(this, "collWidth", "I", frame_width).await?;
        jvm.put_field(this, "collHeight", "I", frame_height).await?;
        // fewer frames than before: the old sequence may point past the end, so fall back to the default one
        if raw_frames < old_raw_frames {
            jvm.put_field(this, "seq", "[I", None).await?;
            jvm.put_field(this, "seqIndex", "I", 0).await?;
        }

        let transform: i32 = jvm.get_field(this, "transform", "I").await?;
        let (width, height) = if matches!(transform, 4..=7) {
            (frame_height, frame_width)
        } else {
            (frame_width, frame_height)
        };
        jvm.put_field(this, "width", "I", width).await?;
        jvm.put_field(this, "height", "I", height).await?;

        Ok(())
    }

    async fn set_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        frame_width: i32,
        frame_height: i32,
    ) -> JvmResult<()> {
        // keep the reference pixel where it is on screen
        let (ref_x, ref_y) = Self::ref_pixel(jvm, &this).await?;
        Self::load_image(jvm, &mut this, image, frame_width, frame_height).await?;
        Self::place_ref_pixel(jvm, &mut this, ref_x, ref_y).await
    }

    async fn store_sequence(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, values: &[i32]) -> JvmResult<()> {
        let mut seq = jvm.instantiate_array("I", values.len()).await?;
        jvm.store_array(&mut seq, 0, values.to_vec()).await?;
        jvm.put_field(this, "seq", "[I", seq).await
    }

    async fn sequence_length(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        let seq: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "seq", "[I").await?;
        if seq.is_null() {
            jvm.get_field(this, "rawFrames", "I").await
        } else {
            Ok(jvm.array_length(&seq).await? as i32)
        }
    }

    async fn raw_frame(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<i32> {
        let seq: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "seq", "[I").await?;
        let index: i32 = jvm.get_field(this, "seqIndex", "I").await?;
        if seq.is_null() {
            Ok(index)
        } else {
            Ok(jvm.load_array(&seq, index as _, 1).await?[0])
        }
    }

    async fn set_frame(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, index: i32) -> JvmResult<()> {
        if index < 0 || index >= Self::sequence_length(jvm, &this).await? {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "frame index out of range").await);
        }
        jvm.put_field(&mut this, "seqIndex", "I", index).await
    }

    async fn get_frame(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "seqIndex", "I").await
    }

    async fn get_raw_frame_count(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "rawFrames", "I").await
    }

    async fn get_frame_sequence_length(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Self::sequence_length(jvm, &this).await
    }

    async fn next_frame(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let index: i32 = jvm.get_field(&this, "seqIndex", "I").await?;
        let length = Self::sequence_length(jvm, &this).await?.max(1);
        jvm.put_field(&mut this, "seqIndex", "I", (index + 1) % length).await
    }

    async fn prev_frame(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        let index: i32 = jvm.get_field(&this, "seqIndex", "I").await?;
        let length = Self::sequence_length(jvm, &this).await?.max(1);
        jvm.put_field(&mut this, "seqIndex", "I", (index + length - 1) % length).await
    }

    async fn set_frame_sequence(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        seq: ClassInstanceRef<Array<i32>>,
    ) -> JvmResult<()> {
        jvm.put_field(&mut this, "seqIndex", "I", 0).await?;
        if seq.is_null() {
            return jvm.put_field(&mut this, "seq", "[I", None).await;
        }

        let values: Vec<i32> = jvm.load_array(&seq, 0, jvm.array_length(&seq).await?).await?;
        let raw_frames: i32 = jvm.get_field(&this, "rawFrames", "I").await?;
        if values.is_empty() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "empty frame sequence").await);
        }
        if values.iter().any(|&f| f < 0 || f >= raw_frames) {
            return Err(jvm.exception("java/lang/ArrayIndexOutOfBoundsException", "frame out of range").await);
        }
        // copied: the game may reuse its array, and the sequence must not change under the sprite
        Self::store_sequence(jvm, &mut this, &values).await
    }

    async fn define_reference_pixel(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        jvm.put_field(&mut this, "refX", "I", x).await?;
        jvm.put_field(&mut this, "refY", "I", y).await
    }

    // The reference pixel, in painter's coordinates.
    async fn ref_pixel(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<(i32, i32)> {
        let x: i32 = jvm.get_field(this, "x", "I").await?;
        let y: i32 = jvm.get_field(this, "y", "I").await?;
        let transform: i32 = jvm.get_field(this, "transform", "I").await?;
        let (dx, dy) = Self::transformed_ref(jvm, this, transform).await?;

        Ok((x + dx, y + dy))
    }

    async fn transformed_ref(jvm: &Jvm, this: &ClassInstanceRef<Self>, transform: i32) -> JvmResult<(i32, i32)> {
        let ref_x: i32 = jvm.get_field(this, "refX", "I").await?;
        let ref_y: i32 = jvm.get_field(this, "refY", "I").await?;
        let frame_width: i32 = jvm.get_field(this, "frameWidth", "I").await?;
        let frame_height: i32 = jvm.get_field(this, "frameHeight", "I").await?;

        Ok(destination_point(transform, ref_x, ref_y, frame_width, frame_height))
    }

    async fn place_ref_pixel(jvm: &Jvm, this: &mut ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        let transform: i32 = jvm.get_field(this, "transform", "I").await?;
        let (dx, dy) = Self::transformed_ref(jvm, this, transform).await?;
        jvm.put_field(this, "x", "I", x - dx).await?;
        jvm.put_field(this, "y", "I", y - dy).await
    }

    async fn set_ref_pixel_position(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, x: i32, y: i32) -> JvmResult<()> {
        Self::place_ref_pixel(jvm, &mut this, x, y).await
    }

    async fn get_ref_pixel_x(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(Self::ref_pixel(jvm, &this).await?.0)
    }

    async fn get_ref_pixel_y(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        Ok(Self::ref_pixel(jvm, &this).await?.1)
    }

    async fn set_transform(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, transform: i32) -> JvmResult<()> {
        if !(0..=7).contains(&transform) {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid transform").await);
        }
        let (ref_x, ref_y) = Self::ref_pixel(jvm, &this).await?;
        jvm.put_field(&mut this, "transform", "I", transform).await?;

        let frame_width: i32 = jvm.get_field(&this, "frameWidth", "I").await?;
        let frame_height: i32 = jvm.get_field(&this, "frameHeight", "I").await?;
        let (width, height) = if matches!(transform, 4..=7) {
            (frame_height, frame_width)
        } else {
            (frame_width, frame_height)
        };
        jvm.put_field(&mut this, "width", "I", width).await?;
        jvm.put_field(&mut this, "height", "I", height).await?;

        Self::place_ref_pixel(jvm, &mut this, ref_x, ref_y).await
    }

    async fn define_collision_rectangle(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> JvmResult<()> {
        if width < 0 || height < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "negative collision size").await);
        }
        jvm.put_field(&mut this, "collX", "I", x).await?;
        jvm.put_field(&mut this, "collY", "I", y).await?;
        jvm.put_field(&mut this, "collWidth", "I", width).await?;
        jvm.put_field(&mut this, "collHeight", "I", height).await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, mut graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }
        let visible: bool = jvm.get_field(&this, "visible", "Z").await?;
        if !visible {
            return Ok(());
        }

        let state = SpriteState::load(jvm, &this).await?;
        let image = state.backend_image(jvm).await?;
        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let mut canvas = Graphics::canvas(jvm, &mut graphics).await?;

        draw_transformed(
            &mut *canvas,
            &*image,
            (state.frame_x, state.frame_y, state.frame_width, state.frame_height),
            state.transform,
            translate_x.wrapping_add(state.x),
            translate_y.wrapping_add(state.y),
            clip,
        );

        Ok(())
    }

    async fn collides_with_sprite(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        other: ClassInstanceRef<Self>,
        pixel_level: bool,
    ) -> JvmResult<bool> {
        if other.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "sprite is null").await);
        }
        if !Self::both_visible(jvm, &this, &other).await? {
            return Ok(false);
        }

        let a = SpriteState::load(jvm, &this).await?;
        let b = SpriteState::load(jvm, &other).await?;
        let Some(rect) = intersect(a.collision_rect(), b.collision_rect()) else {
            return Ok(false);
        };
        if !pixel_level {
            return Ok(true);
        }

        let a_image = a.backend_image(jvm).await?;
        let b_image = b.backend_image(jvm).await?;
        Ok(any_pixel(rect, |x, y| a.opaque_at(&*a_image, x, y) && b.opaque_at(&*b_image, x, y)))
    }

    async fn collides_with_tiled_layer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        layer: ClassInstanceRef<TiledLayer>,
        pixel_level: bool,
    ) -> JvmResult<bool> {
        if layer.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "layer is null").await);
        }
        if !Self::both_visible(jvm, &this, &layer).await? {
            return Ok(false);
        }

        let sprite = SpriteState::load(jvm, &this).await?;
        let tiles = TiledLayer::state(jvm, &layer).await?;
        let Some(rect) = intersect(sprite.collision_rect(), tiles.bounds()) else {
            return Ok(false);
        };

        let sprite_image = if pixel_level { Some(sprite.backend_image(jvm).await?) } else { None };
        let tile_image = if pixel_level { Some(tiles.backend_image(jvm).await?) } else { None };

        let (cell_width, cell_height) = (tiles.cell_width, tiles.cell_height);
        let first_col = (rect.0 - tiles.x) / cell_width;
        let last_col = (rect.0 + rect.2 - 1 - tiles.x) / cell_width;
        let first_row = (rect.1 - tiles.y) / cell_height;
        let last_row = (rect.1 + rect.3 - 1 - tiles.y) / cell_height;
        for row in first_row..=last_row {
            for col in first_col..=last_col {
                let tile = tiles.tile_at(col, row);
                if tile == 0 {
                    continue;
                }
                let (Some(sprite_image), Some(tile_image)) = (&sprite_image, &tile_image) else {
                    return Ok(true);
                };
                let cell = (tiles.x + col * cell_width, tiles.y + row * cell_height, cell_width, cell_height);
                let Some(overlap) = intersect(rect, cell) else {
                    continue;
                };
                let (tile_x, tile_y) = tiles.tile_origin(tile);
                let hit = any_pixel(overlap, |x, y| {
                    sprite.opaque_at(&**sprite_image, x, y) && tile_image.get_pixel(tile_x + x - cell.0, tile_y + y - cell.1).a != 0
                });
                if hit {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn collides_with_image(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        x: i32,
        y: i32,
        pixel_level: bool,
    ) -> JvmResult<bool> {
        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let visible: bool = jvm.get_field(&this, "visible", "Z").await?;
        if !visible {
            return Ok(false);
        }

        let sprite = SpriteState::load(jvm, &this).await?;
        let width: i32 = jvm.get_field(&image, "w", "I").await?;
        let height: i32 = jvm.get_field(&image, "h", "I").await?;
        let Some(rect) = intersect(sprite.collision_rect(), (x, y, width, height)) else {
            return Ok(false);
        };
        if !pixel_level {
            return Ok(true);
        }

        let sprite_image = sprite.backend_image(jvm).await?;
        let other = Image::image(jvm, &image).await?;
        Ok(any_pixel(rect, |px, py| {
            sprite.opaque_at(&*sprite_image, px, py) && other.get_pixel(px - x, py - y).a != 0
        }))
    }

    async fn both_visible<T>(jvm: &Jvm, a: &ClassInstanceRef<Self>, b: &ClassInstanceRef<T>) -> JvmResult<bool> {
        let a_visible: bool = jvm.get_field(a, "visible", "Z").await?;
        let b_visible: bool = jvm.get_field(b, "visible", "Z").await?;

        Ok(a_visible && b_visible)
    }
}

// Where (x, y) of the untransformed w×h frame lands once `transform` is applied — the inverse of
// `region_source_point`. Used for the reference pixel and the collision rectangle.
fn destination_point(transform: i32, x: i32, y: i32, w: i32, h: i32) -> (i32, i32) {
    match transform {
        1 => (x, h - 1 - y),
        2 => (w - 1 - x, y),
        3 => (w - 1 - x, h - 1 - y),
        4 => (y, x),
        5 => (h - 1 - y, x),
        6 => (y, w - 1 - x),
        7 => (h - 1 - y, w - 1 - x),
        _ => (x, y),
    }
}

type Rect = (i32, i32, i32, i32);

fn intersect(a: Rect, b: Rect) -> Option<Rect> {
    let x = a.0.max(b.0);
    let y = a.1.max(b.1);
    let right = (a.0 as i64 + a.2 as i64).min(b.0 as i64 + b.2 as i64);
    let bottom = (a.1 as i64 + a.3 as i64).min(b.1 as i64 + b.3 as i64);
    if right <= x as i64 || bottom <= y as i64 {
        return None;
    }

    Some((x, y, (right - x as i64) as i32, (bottom - y as i64) as i32))
}

fn any_pixel(rect: Rect, mut hit: impl FnMut(i32, i32) -> bool) -> bool {
    (rect.1..rect.1 + rect.3).any(|y| (rect.0..rect.0 + rect.2).any(|x| hit(x, y)))
}

struct SpriteState {
    image: ClassInstanceRef<Image>,
    x: i32,
    y: i32,
    frame_x: i32,
    frame_y: i32,
    frame_width: i32,
    frame_height: i32,
    transform: i32,
    collision: Rect,
}

impl SpriteState {
    async fn load(jvm: &Jvm, this: &ClassInstanceRef<Sprite>) -> JvmResult<Self> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "image", "Ljavax/microedition/lcdui/Image;").await?;
        let image_width: i32 = jvm.get_field(&image, "w", "I").await?;
        let frame_width: i32 = jvm.get_field(this, "frameWidth", "I").await?;
        let frame_height: i32 = jvm.get_field(this, "frameHeight", "I").await?;
        let columns = (image_width / frame_width).max(1);
        let frame = Sprite::raw_frame(jvm, this).await?;

        Ok(Self {
            image,
            x: jvm.get_field(this, "x", "I").await?,
            y: jvm.get_field(this, "y", "I").await?,
            frame_x: (frame % columns) * frame_width,
            frame_y: (frame / columns) * frame_height,
            frame_width,
            frame_height,
            transform: jvm.get_field(this, "transform", "I").await?,
            collision: (
                jvm.get_field(this, "collX", "I").await?,
                jvm.get_field(this, "collY", "I").await?,
                jvm.get_field(this, "collWidth", "I").await?,
                jvm.get_field(this, "collHeight", "I").await?,
            ),
        })
    }

    async fn backend_image(&self, jvm: &Jvm) -> JvmResult<Box<dyn BackendImage>> {
        Image::image(jvm, &self.image).await
    }

    // The collision rectangle in painter's coordinates, after the transform.
    fn collision_rect(&self) -> Rect {
        let (cx, cy, cw, ch) = self.collision;
        if cw == 0 || ch == 0 {
            return (self.x, self.y, 0, 0);
        }
        let a = destination_point(self.transform, cx, cy, self.frame_width, self.frame_height);
        let b = destination_point(self.transform, cx + cw - 1, cy + ch - 1, self.frame_width, self.frame_height);

        (self.x + a.0.min(b.0), self.y + a.1.min(b.1), (a.0 - b.0).abs() + 1, (a.1 - b.1).abs() + 1)
    }

    // Whether the painted sprite has an opaque pixel at painter's (x, y), within its collision rectangle.
    fn opaque_at(&self, image: &dyn BackendImage, x: i32, y: i32) -> bool {
        let (local_x, local_y) = (x - self.x, y - self.y);
        let (w, h) = if matches!(self.transform, 4..=7) {
            (self.frame_height, self.frame_width)
        } else {
            (self.frame_width, self.frame_height)
        };
        if local_x < 0 || local_y < 0 || local_x >= w || local_y >= h {
            return false;
        }
        let (fx, fy) = region_source_point(self.transform, local_x, local_y, self.frame_width, self.frame_height);
        let (cx, cy, cw, ch) = self.collision;
        if fx < cx || fy < cy || fx >= cx + cw || fy >= cy + ch {
            return false;
        }
        let (px, py) = (self.frame_x + fx, self.frame_y + fy);
        if px < 0 || py < 0 || px as u32 >= image.width() || py as u32 >= image.height() {
            return false;
        }

        image.get_pixel(px, py).a != 0
    }
}

#[cfg(test)]
mod test {
    use super::{destination_point, intersect};
    use crate::classes::javax::microedition::lcdui::graphics::region_source_point;

    #[test]
    fn destination_point_inverts_region_source_point() {
        let (w, h) = (5, 3);
        for transform in 0..8 {
            let (out_w, out_h) = if (4..=7).contains(&transform) { (h, w) } else { (w, h) };
            for y in 0..h {
                for x in 0..w {
                    let (dx, dy) = destination_point(transform, x, y, w, h);
                    assert!(dx >= 0 && dy >= 0 && dx < out_w && dy < out_h, "transform {transform}");
                    assert_eq!(region_source_point(transform, dx, dy, w, h), (x, y), "transform {transform}");
                }
            }
        }
    }

    #[test]
    fn intersect_rects() {
        assert_eq!(intersect((0, 0, 10, 10), (5, 5, 10, 10)), Some((5, 5, 5, 5)));
        assert_eq!(intersect((0, 0, 10, 10), (10, 0, 5, 5)), None);
        assert_eq!(intersect((0, 0, 10, 10), (0, 0, i32::MAX, i32::MAX)), Some((0, 0, 10, 10)));
    }
}
