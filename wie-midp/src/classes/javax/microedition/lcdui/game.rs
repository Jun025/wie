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
