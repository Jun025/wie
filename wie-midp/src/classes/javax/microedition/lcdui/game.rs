mod game_canvas;
mod layer;
mod layer_manager;
mod sprite;
mod tiled_layer;

pub use game_canvas::GameCanvas;
pub use layer::Layer;
pub use layer_manager::LayerManager;
pub use sprite::Sprite;
pub use tiled_layer::TiledLayer;

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstance, ClassInstanceRef, Jvm, Result as JvmResult};

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::javax::microedition::lcdui::Image, get_protos};

    const G: &str = "javax/microedition/lcdui/Graphics";

    async fn image(jvm: &Jvm, w: i32, h: i32) -> JvmResult<(ClassInstanceRef<Image>, Box<dyn ClassInstance>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (w, h),
            )
            .await?;
        let graphics = jvm.new_class(G, "(Ljavax/microedition/lcdui/Image;)V", (image.clone(),)).await?;

        Ok((image, graphics))
    }

    #[allow(clippy::borrowed_box)] // invoke_virtual takes &Box
    async fn fill(jvm: &Jvm, g: &Box<dyn ClassInstance>, color: i32, rect: (i32, i32, i32, i32)) -> JvmResult<()> {
        let _: () = jvm.invoke_virtual(g, G, "setColor", "(I)V", (color,)).await?;
        jvm.invoke_virtual(g, G, "fillRect", "(IIII)V", (rect.0, rect.1, rect.2, rect.3)).await
    }

    async fn rgb(jvm: &Jvm, image: &ClassInstanceRef<Image>, x: i32, y: i32) -> JvmResult<(u8, u8, u8)> {
        let c = Image::image(jvm, image).await?.get_pixel(x, y);

        Ok((c.r, c.g, c.b))
    }

    // A 2-frame strip (red | green) drawn as a mirrored Sprite and as a TiledLayer, composed by a LayerManager.
    #[test]
    fn sprite_tiled_layer_and_layer_manager_paint() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let (sheet, sheet_g) = image(&jvm, 4, 2).await?;
            fill(&jvm, &sheet_g, 0xff0000, (0, 0, 2, 2)).await?;
            fill(&jvm, &sheet_g, 0x00ff00, (2, 0, 2, 2)).await?;
            fill(&jvm, &sheet_g, 0x0000ff, (0, 1, 1, 1)).await?; // marks frame 0's bottom-left

            let sprite = jvm
                .new_class(
                    "javax/microedition/lcdui/game/Sprite",
                    "(Ljavax/microedition/lcdui/Image;II)V",
                    (sheet.clone(), 2, 2),
                )
                .await?;
            let frames: i32 = jvm
                .invoke_virtual(&sprite, "javax/microedition/lcdui/game/Sprite", "getRawFrameCount", "()I", ())
                .await?;
            assert_eq!(frames, 2);

            let tiles = jvm
                .new_class(
                    "javax/microedition/lcdui/game/TiledLayer",
                    "(IILjavax/microedition/lcdui/Image;II)V",
                    (3, 1, sheet.clone(), 2, 2),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&tiles, "javax/microedition/lcdui/game/TiledLayer", "setCell", "(III)V", (0, 0, 2))
                .await?;
            let animated: i32 = jvm
                .invoke_virtual(&tiles, "javax/microedition/lcdui/game/TiledLayer", "createAnimatedTile", "(I)I", (1,))
                .await?;
            assert_eq!(animated, -1);
            let _: () = jvm
                .invoke_virtual(&tiles, "javax/microedition/lcdui/game/TiledLayer", "setCell", "(III)V", (2, 0, -1))
                .await?;

            // sprite over the empty middle cell, mirrored: frame 0's blue bottom-left lands bottom-right
            let _: () = jvm
                .invoke_virtual(&sprite, "javax/microedition/lcdui/game/Sprite", "setTransform", "(I)V", (2,))
                .await?;
            let _: () = jvm
                .invoke_virtual(&sprite, "javax/microedition/lcdui/game/Layer", "setPosition", "(II)V", (2, 0))
                .await?;

            let manager = jvm.new_class("javax/microedition/lcdui/game/LayerManager", "()V", ()).await?;
            let _: () = jvm
                .invoke_virtual(
                    &manager,
                    "javax/microedition/lcdui/game/LayerManager",
                    "append",
                    "(Ljavax/microedition/lcdui/game/Layer;)V",
                    (sprite.clone(),),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(
                    &manager,
                    "javax/microedition/lcdui/game/LayerManager",
                    "append",
                    "(Ljavax/microedition/lcdui/game/Layer;)V",
                    (tiles.clone(),),
                )
                .await?;

            let (screen, screen_g) = image(&jvm, 6, 2).await?;
            let _: () = jvm
                .invoke_virtual(
                    &manager,
                    "javax/microedition/lcdui/game/LayerManager",
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;II)V",
                    (screen_g, 0, 0),
                )
                .await?;

            assert_eq!(rgb(&jvm, &screen, 0, 0).await?, (0, 0xff, 0)); // cell 0 = tile 2
            assert_eq!(rgb(&jvm, &screen, 2, 0).await?, (0xff, 0, 0)); // sprite frame 0
            assert_eq!(rgb(&jvm, &screen, 2, 1).await?, (0xff, 0, 0));
            assert_eq!(rgb(&jvm, &screen, 3, 1).await?, (0, 0, 0xff)); // mirrored
            assert_eq!(rgb(&jvm, &screen, 4, 1).await?, (0, 0, 0xff)); // cell 2 = animated -> tile 1, unmirrored

            // reference pixel stays put across the transform; collision against the tiled layer
            let ref_x: i32 = jvm
                .invoke_virtual(&sprite, "javax/microedition/lcdui/game/Sprite", "getRefPixelX", "()I", ())
                .await?;
            assert_eq!(ref_x, 3); // ref (0,0) mirrored in a 2-wide frame = x + 1
            let hit: bool = jvm
                .invoke_virtual(
                    &sprite,
                    "javax/microedition/lcdui/game/Sprite",
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/TiledLayer;Z)Z",
                    (tiles.clone(), false),
                )
                .await?;
            assert!(!hit); // the cell under the sprite is empty
            let _: () = jvm
                .invoke_virtual(&sprite, "javax/microedition/lcdui/game/Layer", "move", "(II)V", (1, 0))
                .await?;
            let hit: bool = jvm
                .invoke_virtual(
                    &sprite,
                    "javax/microedition/lcdui/game/Sprite",
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/TiledLayer;Z)Z",
                    (tiles, true),
                )
                .await?;
            assert!(hit);

            Ok(())
        })
    }
}

#[cfg(test)]
mod upstream_test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, JavaValue, Jvm, Result as JvmResult};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::lcdui::{
            Graphics, Image,
            game::{Layer, LayerManager, Sprite, TiledLayer},
        },
        get_protos,
    };

    // upstream's `sprite.rs` test (dlunch/wie #1601), run against our game package: the two were written
    // separately for the same API, and this is the check that they agree.
    const CLASS: &str = "javax/microedition/lcdui/game/Sprite";
    const LAYER: &str = "javax/microedition/lcdui/game/Layer";

    const TILED_LAYER: &str = "javax/microedition/lcdui/game/TiledLayer";
    const LAYER_MANAGER: &str = "javax/microedition/lcdui/game/LayerManager";
    const RED: (u8, u8, u8) = (0xff, 0, 0);
    const BLUE: (u8, u8, u8) = (0, 0, 0xff);
    // fork: untouched pixels — upstream's BLANK; a new image is transparent black here (image.rs).
    const BLANK: (u8, u8, u8) = (0, 0, 0);

    async fn new_canvas(jvm: &Jvm) -> JvmResult<(ClassInstanceRef<Image>, ClassInstanceRef<Graphics>)> {
        let image: ClassInstanceRef<Image> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Image",
                "createImage",
                "(II)Ljavax/microedition/lcdui/Image;",
                (100, 100),
            )
            .await?;
        let graphics = jvm
            .invoke_virtual(
                &image,
                "javax/microedition/lcdui/Image",
                "getGraphics",
                "()Ljavax/microedition/lcdui/Graphics;",
                (),
            )
            .await?;

        Ok((image, graphics))
    }

    async fn pixel(jvm: &Jvm, image: &ClassInstanceRef<Image>, x: i32, y: i32) -> JvmResult<(u8, u8, u8)> {
        let color = Image::image(jvm, image).await?.get_pixel(x, y);

        Ok((color.r, color.g, color.b))
    }

    #[test]
    fn sprites_and_tiled_layers_paint_and_collide() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            // Two 2x2 frames: only the upper-left pixel of the first is opaque (red), the second is solid blue.
            let (red, blue) = (0xffff0000u32 as i32, 0xff0000ffu32 as i32);
            let mut rgb = jvm.instantiate_array("I", 8).await?;
            jvm.store_array(&mut rgb, 0, [red, 0, blue, blue, 0, 0, blue, blue]).await?;
            let frames: ClassInstanceRef<Image> = jvm
                .invoke_static(
                    "javax/microedition/lcdui/Image",
                    "createRGBImage",
                    "([IIIZ)Ljavax/microedition/lcdui/Image;",
                    (rgb, 4, 2, true),
                )
                .await?;

            let sprite: ClassInstanceRef<Sprite> = jvm
                .new_class(CLASS, "(Ljavax/microedition/lcdui/Image;II)V", (frames.clone(), 2, 2))
                .await?
                .into();
            let other: ClassInstanceRef<Sprite> = jvm
                .new_class(CLASS, "(Ljavax/microedition/lcdui/game/Sprite;)V", (sprite.clone(),))
                .await?
                .into();
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, CLASS, "getRawFrameCount", "()I", ()).await?, 2);

            // Mirroring keeps the reference pixel at (10, 10), which moves the sprite one pixel to the left.
            let _: () = jvm.invoke_virtual(&sprite, LAYER, "setPosition", "(II)V", (10, 10)).await?;
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "setTransform", "(I)V", (2,)).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, LAYER, "getX", "()I", ()).await?, 9);
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, CLASS, "getRefPixelX", "()I", ()).await?, 10);

            // The mirrored sprite is opaque only at (10, 10); the other sprite only at its upper-left corner.
            for (x, pixel_level, expected) in [(10, true, true), (11, false, false), (9, false, true), (9, true, false)] {
                let _: () = jvm.invoke_virtual(&other, LAYER, "setPosition", "(II)V", (x, 10)).await?;
                let collides: bool = jvm
                    .invoke_virtual(
                        &sprite,
                        CLASS,
                        "collidesWith",
                        "(Ljavax/microedition/lcdui/game/Sprite;Z)Z",
                        (other.clone(), pixel_level),
                    )
                    .await?;
                assert_eq!(collides, expected, "x={x} pixel_level={pixel_level}");
            }
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "nextFrame", "()V", ()).await?;
            assert!(
                jvm.invoke_virtual::<_, bool>(
                    &sprite,
                    CLASS,
                    "collidesWith",
                    "(Ljavax/microedition/lcdui/game/Sprite;Z)Z",
                    (other.clone(), true)
                )
                .await?
            );

            // Sequence [1, 0]: stepping back from entry 0 wraps to entry 1, which is the first raw frame.
            let mut sequence = jvm.instantiate_array("I", 2).await?;
            jvm.store_array(&mut sequence, 0, [1, 0]).await?;
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "setFrameSequence", "([I)V", (sequence,)).await?;
            let _: () = jvm.invoke_virtual(&sprite, CLASS, "prevFrame", "()V", ()).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&sprite, CLASS, "getFrame", "()I", ()).await?, 1);

            let (canvas, graphics) = new_canvas(&jvm).await?;
            let _: () = jvm
                .invoke_virtual(&sprite, CLASS, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics.clone(),))
                .await?;
            assert_eq!(pixel(&jvm, &canvas, 10, 10).await?, RED);
            assert_eq!(pixel(&jvm, &canvas, 9, 10).await?, BLANK);

            // Three cells at (20, 10): empty, solid blue, and an animated tile showing the red corner tile.
            let layer: ClassInstanceRef<TiledLayer> = jvm
                .new_class(TILED_LAYER, "(IILjavax/microedition/lcdui/Image;II)V", (3, 1, frames.clone(), 2, 2))
                .await?
                .into();
            let _: () = jvm.invoke_virtual(&layer, LAYER, "setPosition", "(II)V", (20, 10)).await?;
            let _: () = jvm.invoke_virtual(&layer, TILED_LAYER, "setCell", "(III)V", (1, 0, 2)).await?;
            let animated: i32 = jvm.invoke_virtual(&layer, TILED_LAYER, "createAnimatedTile", "(I)I", (1,)).await?;
            assert_eq!(animated, -1);
            let _: () = jvm.invoke_virtual(&layer, TILED_LAYER, "setCell", "(III)V", (2, 0, animated)).await?;
            assert_eq!(jvm.invoke_virtual::<_, i32>(&layer, TILED_LAYER, "getCell", "(II)I", (2, 0)).await?, -1);

            let _: () = jvm
                .invoke_virtual(
                    &layer,
                    TILED_LAYER,
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;)V",
                    (graphics.clone(),),
                )
                .await?;
            for (x, expected) in [(20, BLANK), (22, BLUE), (24, RED), (25, BLANK)] {
                assert_eq!(pixel(&jvm, &canvas, x, 10).await?, expected, "x={x}");
            }
            let _: () = jvm.invoke_virtual(&layer, TILED_LAYER, "setAnimatedTile", "(II)V", (animated, 2)).await?;
            let _: () = jvm
                .invoke_virtual(&layer, TILED_LAYER, "paint", "(Ljavax/microedition/lcdui/Graphics;)V", (graphics,))
                .await?;
            assert_eq!(pixel(&jvm, &canvas, 25, 10).await?, BLUE);

            // At x = 21 the other sprite's only opaque pixel is over the empty cell; at x = 22 it is over the blue one.
            for (x, pixel_level, expected) in [(17, false, false), (21, false, true), (21, true, false), (22, true, true)] {
                let _: () = jvm.invoke_virtual(&other, LAYER, "setPosition", "(II)V", (x, 10)).await?;
                let collides: bool = jvm
                    .invoke_virtual(
                        &other,
                        CLASS,
                        "collidesWith",
                        "(Ljavax/microedition/lcdui/game/TiledLayer;Z)Z",
                        (layer.clone(), pixel_level),
                    )
                    .await?;
                assert_eq!(collides, expected, "x={x} pixel_level={pixel_level}");
            }
            // The image's red corner is at its upper-left, so it meets the sprite's opaque pixel only at x = 22.
            for (x, pixel_level, expected) in [(22, true, true), (23, false, true), (23, true, false)] {
                let collides: bool = jvm
                    .invoke_virtual(
                        &other,
                        CLASS,
                        "collidesWith",
                        "(Ljavax/microedition/lcdui/Image;IIZ)Z",
                        (frames.clone(), x, 10, pixel_level),
                    )
                    .await?;
                assert_eq!(collides, expected, "x={x} pixel_level={pixel_level}");
            }

            // The sprite at (22, 10) is above the tiled layer; the 4x2 view window at (20, 10) is drawn at (50, 50).
            let manager: ClassInstanceRef<LayerManager> = jvm.new_class(LAYER_MANAGER, "()V", ()).await?.into();
            for appended in [JavaValue::from(other.clone()), JavaValue::from(layer.clone())] {
                let _: () = jvm
                    .invoke_virtual(&manager, LAYER_MANAGER, "append", "(Ljavax/microedition/lcdui/game/Layer;)V", [appended])
                    .await?;
            }
            assert_eq!(jvm.invoke_virtual::<_, i32>(&manager, LAYER_MANAGER, "getSize", "()I", ()).await?, 2);
            let top: ClassInstanceRef<Layer> = jvm
                .invoke_virtual(&manager, LAYER_MANAGER, "getLayerAt", "(I)Ljavax/microedition/lcdui/game/Layer;", (0,))
                .await?;
            assert_eq!(top.identity(), other.identity());

            let _: () = jvm
                .invoke_virtual(&manager, LAYER_MANAGER, "setViewWindow", "(IIII)V", (20, 10, 4, 2))
                .await?;
            let (canvas, graphics) = new_canvas(&jvm).await?;
            let _: () = jvm
                .invoke_virtual(
                    &manager,
                    LAYER_MANAGER,
                    "paint",
                    "(Ljavax/microedition/lcdui/Graphics;II)V",
                    (graphics.clone(), 50, 50),
                )
                .await?;
            for (x, expected) in [(50, BLANK), (52, RED), (53, BLUE), (54, BLANK)] {
                assert_eq!(pixel(&jvm, &canvas, x, 50).await?, expected, "x={x}");
            }
            for (getter, expected) in [("getTranslateX", 0), ("getClipX", 0), ("getClipWidth", 100)] {
                let value: i32 = jvm
                    .invoke_virtual(&graphics, "javax/microedition/lcdui/Graphics", getter, "()I", ())
                    .await?;
                assert_eq!(value, expected, "{getter}");
            }

            Ok(())
        })
    }
}
