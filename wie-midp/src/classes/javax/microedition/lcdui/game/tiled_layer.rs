use alloc::{boxed::Box, vec, vec::Vec};

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_backend::canvas::Image as BackendImage;
use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::lcdui::{Graphics, Image};

// class javax.microedition.lcdui.game.TiledLayer (JSR-118)
//
// `cells` is row-major; 0 is empty, n > 0 is static tile n (1-based, left-to-right then
// top-to-bottom in `image`), n < 0 is animated tile n whose static tile is `animated[-n - 1]`.
pub struct TiledLayer;

impl TiledLayer {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/game/TiledLayer",
            parent_class: Some("javax/microedition/lcdui/game/Layer"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(IILjavax/microedition/lcdui/Image;II)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("createAnimatedTile", "(I)I", Self::create_animated_tile, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setAnimatedTile", "(II)V", Self::set_animated_tile, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getAnimatedTile", "(I)I", Self::get_animated_tile, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setCell", "(III)V", Self::set_cell, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCell", "(II)I", Self::get_cell, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("fillCells", "(IIIII)V", Self::fill_cells, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCellWidth", "()I", Self::get_cell_width, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getCellHeight", "()I", Self::get_cell_height, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getColumns", "()I", Self::get_columns, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getRows", "()I", Self::get_rows, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "setStaticTileSet",
                    "(Ljavax/microedition/lcdui/Image;II)V",
                    Self::set_static_tile_set,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("paint", "(Ljavax/microedition/lcdui/Graphics;)V", Self::paint, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("image", "Ljavax/microedition/lcdui/Image;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cellWidth", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cellHeight", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("columns", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("rows", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("staticTiles", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("cells", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("animated", "[I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("animatedCount", "I", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn init(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        columns: i32,
        rows: i32,
        image: ClassInstanceRef<Image>,
        tile_width: i32,
        tile_height: i32,
    ) -> JvmResult<()> {
        if columns < 1 || rows < 1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid size").await);
        }
        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/lcdui/game/Layer",
                "<init>",
                "(II)V",
                (columns * tile_width, rows * tile_height),
            )
            .await?;
        jvm.put_field(&mut this, "columns", "I", columns).await?;
        jvm.put_field(&mut this, "rows", "I", rows).await?;
        let cells = jvm.instantiate_array("I", (columns * rows) as _).await?;
        jvm.put_field(&mut this, "cells", "[I", cells).await?;
        let animated = jvm.instantiate_array("I", 4).await?;
        jvm.put_field(&mut this, "animated", "[I", animated).await?;

        Self::load_tile_set(jvm, &mut this, image, tile_width, tile_height).await
    }

    async fn load_tile_set(
        jvm: &Jvm,
        this: &mut ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        tile_width: i32,
        tile_height: i32,
    ) -> JvmResult<()> {
        if image.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "image is null").await);
        }
        let image_width: i32 = jvm.get_field(&image, "w", "I").await?;
        let image_height: i32 = jvm.get_field(&image, "h", "I").await?;
        if tile_width < 1 || tile_height < 1 || image_width % tile_width != 0 || image_height % tile_height != 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "invalid tile size").await);
        }

        jvm.put_field(this, "image", "Ljavax/microedition/lcdui/Image;", image).await?;
        jvm.put_field(this, "cellWidth", "I", tile_width).await?;
        jvm.put_field(this, "cellHeight", "I", tile_height).await?;
        jvm.put_field(this, "staticTiles", "I", (image_width / tile_width) * (image_height / tile_height))
            .await
    }

    async fn set_static_tile_set(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        image: ClassInstanceRef<Image>,
        tile_width: i32,
        tile_height: i32,
    ) -> JvmResult<()> {
        let old_tiles: i32 = jvm.get_field(&this, "staticTiles", "I").await?;
        Self::load_tile_set(jvm, &mut this, image, tile_width, tile_height).await?;
        let columns: i32 = jvm.get_field(&this, "columns", "I").await?;
        let rows: i32 = jvm.get_field(&this, "rows", "I").await?;
        jvm.put_field(&mut this, "width", "I", columns * tile_width).await?;
        jvm.put_field(&mut this, "height", "I", rows * tile_height).await?;

        // fewer tiles than before: the cells may name tiles that no longer exist, so start empty
        let new_tiles: i32 = jvm.get_field(&this, "staticTiles", "I").await?;
        if new_tiles < old_tiles {
            let cells = jvm.instantiate_array("I", (columns * rows) as _).await?;
            jvm.put_field(&mut this, "cells", "[I", cells).await?;
            jvm.put_field(&mut this, "animatedCount", "I", 0).await?;
        }

        Ok(())
    }

    async fn check_static_tile(jvm: &Jvm, this: &ClassInstanceRef<Self>, tile: i32) -> JvmResult<()> {
        let tiles: i32 = jvm.get_field(this, "staticTiles", "I").await?;
        if tile < 0 || tile > tiles {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "invalid static tile").await);
        }

        Ok(())
    }

    async fn check_tile(jvm: &Jvm, this: &ClassInstanceRef<Self>, tile: i32) -> JvmResult<()> {
        if tile >= 0 {
            return Self::check_static_tile(jvm, this, tile).await;
        }
        let count: i32 = jvm.get_field(this, "animatedCount", "I").await?;
        if -tile > count {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "invalid animated tile").await);
        }

        Ok(())
    }

    async fn create_animated_tile(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, static_tile: i32) -> JvmResult<i32> {
        Self::check_static_tile(jvm, &this, static_tile).await?;

        let count: i32 = jvm.get_field(&this, "animatedCount", "I").await?;
        let mut animated: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "animated", "[I").await?;
        let capacity = jvm.array_length(&animated).await?;
        if count as usize == capacity {
            let mut values: Vec<i32> = jvm.load_array(&animated, 0, capacity).await?;
            values.resize(capacity * 2, 0);
            animated = jvm.instantiate_array("I", values.len()).await?.into();
            jvm.store_array(&mut animated, 0, values).await?;
            jvm.put_field(&mut this, "animated", "[I", animated.clone()).await?;
        }
        jvm.store_array(&mut animated, count as _, vec![static_tile]).await?;
        jvm.put_field(&mut this, "animatedCount", "I", count + 1).await?;

        Ok(-(count + 1))
    }

    async fn set_animated_tile(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32, static_tile: i32) -> JvmResult<()> {
        Self::check_static_tile(jvm, &this, static_tile).await?;
        if index >= 0 {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "invalid animated tile").await);
        }
        Self::check_tile(jvm, &this, index).await?;
        let mut animated: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "animated", "[I").await?;

        jvm.store_array(&mut animated, (-index - 1) as _, vec![static_tile]).await
    }

    async fn get_animated_tile(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, index: i32) -> JvmResult<i32> {
        if index >= 0 {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "invalid animated tile").await);
        }
        Self::check_tile(jvm, &this, index).await?;
        let animated: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "animated", "[I").await?;

        Ok(jvm.load_array(&animated, (-index - 1) as _, 1).await?[0])
    }

    async fn cell_index(jvm: &Jvm, this: &ClassInstanceRef<Self>, col: i32, row: i32) -> JvmResult<usize> {
        let columns: i32 = jvm.get_field(this, "columns", "I").await?;
        let rows: i32 = jvm.get_field(this, "rows", "I").await?;
        if col < 0 || row < 0 || col >= columns || row >= rows {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "cell out of range").await);
        }

        Ok((row * columns + col) as usize)
    }

    async fn set_cell(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, col: i32, row: i32, tile: i32) -> JvmResult<()> {
        let index = Self::cell_index(jvm, &this, col, row).await?;
        Self::check_tile(jvm, &this, tile).await?;
        let mut cells: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "cells", "[I").await?;

        jvm.store_array(&mut cells, index, vec![tile]).await
    }

    async fn get_cell(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, col: i32, row: i32) -> JvmResult<i32> {
        let index = Self::cell_index(jvm, &this, col, row).await?;
        let cells: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "cells", "[I").await?;

        Ok(jvm.load_array(&cells, index, 1).await?[0])
    }

    #[allow(clippy::too_many_arguments)]
    async fn fill_cells(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        col: i32,
        row: i32,
        num_cols: i32,
        num_rows: i32,
        tile: i32,
    ) -> JvmResult<()> {
        if num_cols < 0 || num_rows < 0 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "negative size").await);
        }
        if num_cols == 0 || num_rows == 0 {
            return Ok(());
        }
        Self::cell_index(jvm, &this, col, row).await?;
        Self::cell_index(jvm, &this, col + num_cols - 1, row + num_rows - 1).await?;
        Self::check_tile(jvm, &this, tile).await?;

        let columns: i32 = jvm.get_field(&this, "columns", "I").await?;
        let mut cells: ClassInstanceRef<Array<i32>> = jvm.get_field(&this, "cells", "[I").await?;
        for r in row..row + num_rows {
            jvm.store_array(&mut cells, (r * columns + col) as _, vec![tile; num_cols as usize])
                .await?;
        }

        Ok(())
    }

    async fn get_cell_width(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "cellWidth", "I").await
    }

    async fn get_cell_height(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "cellHeight", "I").await
    }

    async fn get_columns(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "columns", "I").await
    }

    async fn get_rows(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        jvm.get_field(&this, "rows", "I").await
    }

    async fn paint(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, mut graphics: ClassInstanceRef<Graphics>) -> JvmResult<()> {
        if graphics.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "graphics is null").await);
        }
        let visible: bool = jvm.get_field(&this, "visible", "Z").await?;
        if !visible {
            return Ok(());
        }

        let state = Self::state(jvm, &this).await?;
        let tile_image = state.backend_image(jvm).await?;

        let translate_x: i32 = jvm.get_field(&graphics, "translateX", "I").await?;
        let translate_y: i32 = jvm.get_field(&graphics, "translateY", "I").await?;
        let clip = Graphics::clip(jvm, &graphics).await?;
        let mut canvas = Graphics::canvas(jvm, &mut graphics).await?;

        // only the cells under the clip: a level map is often many screens wide
        let origin_x = translate_x as i64 + state.x as i64;
        let origin_y = translate_y as i64 + state.y as i64;
        let (cw, ch) = (state.cell_width as i64, state.cell_height as i64);
        let first_col = ((clip.x as i64 - origin_x).div_euclid(cw)).max(0);
        let last_col = ((clip.x as i64 + clip.width as i64 - 1 - origin_x).div_euclid(cw)).min(state.columns as i64 - 1);
        let first_row = ((clip.y as i64 - origin_y).div_euclid(ch)).max(0);
        let last_row = ((clip.y as i64 + clip.height as i64 - 1 - origin_y).div_euclid(ch)).min(state.rows as i64 - 1);

        for row in first_row..=last_row {
            for col in first_col..=last_col {
                let tile = state.tile_at(col as i32, row as i32);
                if tile == 0 {
                    continue;
                }
                let (sx, sy) = state.tile_origin(tile);
                canvas.draw(
                    (origin_x + col * cw) as i32,
                    (origin_y + row * ch) as i32,
                    state.cell_width as u32,
                    state.cell_height as u32,
                    &*tile_image,
                    sx,
                    sy,
                    clip,
                );
            }
        }

        Ok(())
    }

    pub(super) async fn state(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<TiledState> {
        let image: ClassInstanceRef<Image> = jvm.get_field(this, "image", "Ljavax/microedition/lcdui/Image;").await?;
        let cells: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "cells", "[I").await?;
        let animated: ClassInstanceRef<Array<i32>> = jvm.get_field(this, "animated", "[I").await?;
        let animated_count: i32 = jvm.get_field(this, "animatedCount", "I").await?;
        let image_width: i32 = jvm.get_field(&image, "w", "I").await?;
        let cell_width: i32 = jvm.get_field(this, "cellWidth", "I").await?;

        Ok(TiledState {
            x: jvm.get_field(this, "x", "I").await?,
            y: jvm.get_field(this, "y", "I").await?,
            cell_width,
            cell_height: jvm.get_field(this, "cellHeight", "I").await?,
            columns: jvm.get_field(this, "columns", "I").await?,
            rows: jvm.get_field(this, "rows", "I").await?,
            tile_columns: (image_width / cell_width).max(1),
            cells: jvm.load_array(&cells, 0, jvm.array_length(&cells).await?).await?,
            animated: jvm.load_array(&animated, 0, animated_count as _).await?,
            image,
        })
    }
}

pub(super) struct TiledState {
    pub x: i32,
    pub y: i32,
    pub cell_width: i32,
    pub cell_height: i32,
    columns: i32,
    rows: i32,
    tile_columns: i32,
    cells: Vec<i32>,
    animated: Vec<i32>,
    image: ClassInstanceRef<Image>,
}

impl TiledState {
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        (self.x, self.y, self.columns * self.cell_width, self.rows * self.cell_height)
    }

    // The static tile shown at (col, row) — animated tiles resolved — or 0 for empty / out of range.
    pub fn tile_at(&self, col: i32, row: i32) -> i32 {
        if col < 0 || row < 0 || col >= self.columns || row >= self.rows {
            return 0;
        }
        let cell = self.cells[(row * self.columns + col) as usize];
        if cell < 0 {
            self.animated.get((-cell - 1) as usize).copied().unwrap_or(0)
        } else {
            cell
        }
    }

    // Top-left of static tile `tile` (1-based) in the tile image.
    pub fn tile_origin(&self, tile: i32) -> (i32, i32) {
        let index = tile - 1;

        (
            (index % self.tile_columns) * self.cell_width,
            (index / self.tile_columns) * self.cell_height,
        )
    }

    pub async fn backend_image(&self, jvm: &Jvm) -> JvmResult<Box<dyn BackendImage>> {
        Image::image(jvm, &self.image).await
    }
}
