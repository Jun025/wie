#![allow(clippy::too_many_arguments)]

use alloc::{string::String, vec};

use wie_backend::canvas::{Clip, Color, Image, PixelType, Rgb8Pixel};
use wie_util::{Result, read_null_terminated_string_bytes};

use crate::{WIPICContext, api::graphics::FrameBuffer};

pub fn read_text(context: &dyn WIPICContext, address: u32, length: i32) -> Result<Option<String>> {
    let bytes = if length == -1 {
        read_null_terminated_string_bytes(context, address)?
    } else if length >= 0 {
        let mut bytes = vec![0; length as usize];
        context.read_bytes(address, &mut bytes)?;
        bytes
    } else {
        return Ok(None);
    };

    Ok(Some(encoding_rs::EUC_KR.decode(&bytes).0.into_owned()))
}

/// `x, y, width, height` as an area for `write_canvas`, in i64 so guest extremes cannot overflow.
fn area(x: i32, y: i32, width: u32, height: u32) -> Option<[i64; 4]> {
    Some([x as i64, y as i64, x as i64 + width as i64, y as i64 + height as i64])
}

/// Run `operation` on the part of `framebuffer` it can touch: `area` (`[x0, y0, x1, y1)`, `None` = anywhere)
/// within `clip` and the buffer. Only that part goes between guest memory and the canvas, and the
/// operation gets it as its clip, so nothing outside it can change.
fn write_canvas<F>(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, area: Option<[i64; 4]>, clip: Clip, operation: F) -> Result<()>
where
    F: FnOnce(&mut dyn wie_backend::canvas::Canvas, Clip),
{
    let [ax0, ay0, ax1, ay1] = area.unwrap_or([i64::MIN, i64::MIN, i64::MAX, i64::MAX]);
    let x0 = ax0.max(clip.x as i64).max(0);
    let y0 = ay0.max(clip.y as i64).max(0);
    let x1 = ax1.min(clip.x as i64 + clip.width as i64).min(framebuffer.0.width as i64);
    let y1 = ay1.min(clip.y as i64 + clip.height as i64).min(framebuffer.0.height as i64);
    if x0 >= x1 || y0 >= y1 {
        return Ok(());
    }
    let region = Clip {
        x: x0 as i32,
        y: y0 as i32,
        width: (x1 - x0) as u32,
        height: (y1 - y0) as u32,
    };

    let mut canvas = framebuffer.canvas(context, region)?;
    operation(&mut **canvas, region);
    canvas.flush()
}

pub fn put_pixel(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, x: i32, y: i32, color: Color, clip: Clip) -> Result<()> {
    write_canvas(context, framebuffer, area(x, y, 1, 1), clip, |canvas, clip| {
        canvas.put_pixel(x, y, color, clip)
    })
}

pub fn fill_rect(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Color,
    clip: Clip,
) -> Result<()> {
    write_canvas(context, framebuffer, area(x, y, width, height), clip, |canvas, clip| {
        canvas.fill_rect(x, y, width, height, color, clip)
    })
}

pub fn draw_line(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    color: Color,
    clip: Clip,
) -> Result<()> {
    // bresenham stays inside the endpoints' box, also after the canvas clips the segment to the image
    let line = Some([x1.min(x2) as i64, y1.min(y2) as i64, x1.max(x2) as i64 + 1, y1.max(y2) as i64 + 1]);
    write_canvas(context, framebuffer, line, clip, |canvas, clip| {
        canvas.draw_line(x1, y1, x2, y2, color, clip)
    })
}

pub fn draw_rect(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    color: Color,
    clip: Clip,
) -> Result<()> {
    write_canvas(context, framebuffer, area(x, y, width, height), clip, |canvas, clip| {
        canvas.draw_rect(x, y, width, height, color, clip)
    })
}

pub fn draw_arc(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    start_angle: i32,
    arc_angle: i32,
    color: Color,
    clip: Clip,
) -> Result<()> {
    write_canvas(context, framebuffer, area(x, y, width, height), clip, |canvas, clip| {
        canvas.draw_arc(x, y, width, height, start_angle, arc_angle, color, clip)
    })
}

pub fn fill_arc(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    start_angle: i32,
    arc_angle: i32,
    color: Color,
    clip: Clip,
) -> Result<()> {
    write_canvas(context, framebuffer, area(x, y, width, height), clip, |canvas, clip| {
        canvas.fill_arc(x, y, width, height, start_angle, arc_angle, color, clip)
    })
}

pub fn draw_image(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    image: &dyn Image,
    source_x: i32,
    source_y: i32,
    clip: Clip,
) -> Result<()> {
    write_canvas(context, framebuffer, area(x, y, width, height), clip, |canvas, clip| {
        canvas.draw(x, y, width, height, image, source_x, source_y, clip)
    })
}

pub fn copy_area(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source_x: i32,
    source_y: i32,
    clip: Clip,
) -> Result<()> {
    let image = framebuffer.image(context)?;
    write_canvas(context, framebuffer, area(x, y, width, height), clip, |canvas, clip| {
        canvas.draw(x, y, width, height, &*image, source_x, source_y, clip)
    })
}

pub fn copy_framebuffer(
    context: &mut dyn WIPICContext,
    destination: &FrameBuffer,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    source: &FrameBuffer,
    source_x: i32,
    source_y: i32,
    clip: Clip,
) -> Result<()> {
    let image = source.image(context)?;
    write_canvas(context, destination, area(x, y, width, height), clip, |canvas, clip| {
        canvas.draw(x, y, width, height, &*image, source_x, source_y, clip)
    })
}

pub fn draw_text(context: &mut dyn WIPICContext, framebuffer: &FrameBuffer, string: &str, x: i32, y: i32, color: Color, clip: Clip) -> Result<()> {
    let font = context.system().platform().font().clone();
    // glyph extents are the font's business: the whole clip
    write_canvas(context, framebuffer, None, clip, |canvas, clip| {
        canvas.draw_text(&font, string, x, y, wie_backend::canvas::TextAlignment::Left, color, clip)
    })
}

fn rgb_row_bytes(width: i32, stride: i32) -> Option<usize> {
    if width <= 0 || stride <= 0 {
        return None;
    }

    let row_bytes = (width as usize).checked_mul(4)?;
    (stride as usize >= row_bytes).then_some(row_bytes)
}

pub fn get_rgb_pixels(
    context: &mut dyn WIPICContext,
    image: &dyn Image,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    destination: u32,
    destination_bpl: i32,
) -> Result<()> {
    if height <= 0 {
        return Ok(());
    }
    let Some(row_bytes) = rgb_row_bytes(width, destination_bpl) else {
        return Ok(());
    };

    let mut row = vec![0; row_bytes];
    for row_index in 0..height {
        for column in 0..width {
            let source_x = x.wrapping_add(column);
            let source_y = y.wrapping_add(row_index);
            let color = if source_x < 0 || source_y < 0 || source_x >= image.width() as i32 || source_y >= image.height() as i32 {
                Color { a: 0, r: 0, g: 0, b: 0 }
            } else {
                image.get_pixel(source_x, source_y)
            };
            let offset = column as usize * 4;
            row[offset..offset + 4].copy_from_slice(&Rgb8Pixel::from_color(color).to_le_bytes());
        }
        let address = destination
            .checked_add(
                (row_index as u32)
                    .checked_mul(destination_bpl as u32)
                    .ok_or(wie_util::WieError::AllocationFailure)?,
            )
            .ok_or(wie_util::WieError::AllocationFailure)?;
        context.write_bytes(address, &row)?;
    }
    Ok(())
}

pub fn set_rgb_pixels(
    context: &mut dyn WIPICContext,
    framebuffer: &FrameBuffer,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    source: u32,
    source_bpl: i32,
    clip: Clip,
) -> Result<()> {
    if height <= 0 {
        return Ok(());
    }
    let Some(row_bytes) = rgb_row_bytes(width, source_bpl) else {
        return Ok(());
    };
    let Some(total_bytes) = row_bytes.checked_mul(height as usize) else {
        return Ok(());
    };

    let mut pixels = vec![0; total_bytes];
    for row_index in 0..height {
        let address = source
            .checked_add(
                (row_index as u32)
                    .checked_mul(source_bpl as u32)
                    .ok_or(wie_util::WieError::AllocationFailure)?,
            )
            .ok_or(wie_util::WieError::AllocationFailure)?;
        let offset = row_index as usize * row_bytes;
        context.read_bytes(address, &mut pixels[offset..offset + row_bytes])?;
    }

    // guest coordinates wrap (`wrapping_add`), so there is no one box: the whole clip
    write_canvas(context, framebuffer, None, clip, |canvas, clip| {
        for row_index in 0..height {
            let row_offset = row_index as usize * row_bytes;
            let row = &pixels[row_offset..row_offset + row_bytes];
            for column in 0..width {
                let offset = column as usize * 4;
                let rgb = u32::from_le_bytes([row[offset], row[offset + 1], row[offset + 2], row[offset + 3]]);
                canvas.put_pixel(x.wrapping_add(column), y.wrapping_add(row_index), Rgb8Pixel::to_color(rgb), clip);
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use wie_backend::canvas::{Clip, Color};
    use wie_util::{ByteRead, ByteWrite, Result};

    use crate::{
        api::graphics::{
            FrameBuffer,
            primitives::{fill_rect, get_rgb_pixels, put_pixel, set_rgb_pixels},
        },
        context::{WIPICContext, test::TestContext},
    };

    #[test]
    fn drawing_primitives_write_the_guest_framebuffer() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 4, 4, 16)?;
        let clip = Clip {
            x: 0,
            y: 0,
            width: 4,
            height: 4,
        };
        let red = Color {
            a: 0xff,
            r: 0xff,
            g: 0,
            b: 0,
        };

        fill_rect(&mut context, &framebuffer, 1, 1, 2, 2, red, clip)?;
        put_pixel(&mut context, &framebuffer, 0, 0, red, clip)?;

        let image = framebuffer.image(&mut context)?;
        assert_eq!(image.get_pixel(0, 0).r, 255);
        assert_eq!(image.get_pixel(1, 1).r, 255);
        assert_eq!(image.get_pixel(3, 3).r, 0);
        Ok(())
    }

    #[test]
    fn rgb_pixels_use_the_wipi_little_endian_layout() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 1, 1, 16)?;
        put_pixel(
            &mut context,
            &framebuffer,
            0,
            0,
            Color {
                a: 0xff,
                r: 0xff,
                g: 0,
                b: 0,
            },
            Clip {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        )?;
        let image = framebuffer.image(&mut context)?;

        get_rgb_pixels(&mut context, &*image, 0, 0, 1, 1, 0x1000, 4)?;
        let mut bytes = [0; 4];
        context.read_bytes(0x1000, &mut bytes)?;
        assert_eq!(u32::from_le_bytes(bytes), 0x00ff_0000);
        Ok(())
    }

    #[test]
    fn rgb_row_bytes_rejects_overflow_and_short_strides() {
        assert_eq!(super::rgb_row_bytes(i32::MAX, i32::MAX), None);
        assert_eq!(super::rgb_row_bytes(4, 15), None);
        assert_eq!(super::rgb_row_bytes(4, 16), Some(16));
    }

    #[test]
    fn set_rgb_pixels_wraps_guest_coordinates() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 1, 1, 16)?;
        let source = context.alloc(8)?;
        context.write_bytes(context.data_ptr(source)?, &[0, 0, 0, 0, 0, 0, 0, 0])?;

        set_rgb_pixels(
            &mut context,
            &framebuffer,
            i32::MAX,
            0,
            2,
            1,
            source.0,
            8,
            Clip {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
        )
    }
}

/// Every primitive against the canvas it used to run on — the whole buffer read, drawn, written back —
/// over a patterned buffer, so a region too small (lost pixels) or a write outside it (changed pixels)
/// both show. Covers clips, off-buffer and extreme coordinates, a source with transparent pixels
/// (the colour key arrives as alpha 0), and offscreen buffers narrower and wider than the source.
#[cfg(test)]
mod region_tests {
    use alloc::{boxed::Box, vec::Vec};

    use wie_backend::canvas::{ArgbPixel, Canvas, Clip, Color, Image, ImageBufferCanvas, PixelType, Rgb565Pixel, VecImageBuffer};
    use wie_util::Result;

    use super::*;
    use crate::context::test::TestContext;

    struct Lcg(u32);
    impl Lcg {
        fn next(&mut self) -> u32 {
            self.0 = self.0.wrapping_mul(1_103_515_245).wrapping_add(12345);
            self.0 >> 8
        }
        fn coord(&mut self, size: u32) -> i32 {
            match self.next() % 16 {
                0 => i32::MIN + (self.next() % 4) as i32,
                1 => i32::MAX - (self.next() % 4) as i32,
                _ => (self.next() % (size + 8)) as i32 - 4,
            }
        }
        fn len(&mut self, size: u32) -> u32 {
            if self.next() % 16 == 0 {
                u32::MAX - self.next() % 4
            } else {
                self.next() % (size + 4)
            }
        }
        fn color(&mut self) -> Color {
            let v = self.next();
            Color {
                a: 0xff,
                r: v as u8,
                g: (v >> 8) as u8,
                b: (v >> 16) as u8,
            }
        }
    }

    fn reference<P: PixelType + 'static>(raw: &[u8], w: u32, h: u32, op: &dyn Fn(&mut dyn Canvas)) -> Vec<u8> {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<P>::from_raw(w, h, bytemuck::pod_collect_to_vec(raw)));
        op(&mut canvas);
        canvas.image().raw().into_owned()
    }

    fn sweep(bpp: u32, w: u32, h: u32, seed: u32) -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, w, h, bpp)?;
        let mut rng = Lcg(seed);
        let pattern: Vec<u8> = (0..w * h * bpp / 8).map(|_| rng.next() as u8).collect();

        // the source: half its pixels transparent, sized off the target's
        let (sw, sh) = (w / 2 + 3, h + 2);
        let source: Box<dyn Image> = Box::new(VecImageBuffer::<ArgbPixel>::from_raw(
            sw,
            sh,
            (0..sw * sh)
                .map(|i| {
                    ArgbPixel::from_color(Color {
                        a: if i % 2 == 0 { 0 } else { 0xff },
                        ..rng.color()
                    })
                })
                .collect(),
        ));
        let source_framebuffer = FrameBuffer::from_image(&mut context, &*source)?;

        for round in 0..400 {
            framebuffer.write(&mut context, &pattern)?;
            let clip = if rng.next() % 3 == 0 {
                Clip {
                    x: 0,
                    y: 0,
                    width: w,
                    height: h,
                }
            } else {
                Clip {
                    x: rng.coord(w),
                    y: rng.coord(h),
                    width: rng.len(w),
                    height: rng.len(h),
                }
            };
            let (x, y, x2, y2) = (rng.coord(w), rng.coord(h), rng.coord(w), rng.coord(h));
            let (cw, ch) = (rng.len(w), rng.len(h));
            let (sx, sy) = (rng.coord(sw), rng.coord(sh));
            let color = rng.color();
            let kind = round % 10;

            let op = |canvas: &mut dyn Canvas| match kind {
                0 => canvas.put_pixel(x, y, color, clip),
                1 => canvas.fill_rect(x, y, cw, ch, color, clip),
                2 => canvas.draw_line(x, y, x2, y2, color, clip),
                3 => canvas.draw_rect(x, y, cw, ch, color, clip),
                4 => canvas.draw_arc(x, y, cw % 64, ch % 64, sx, sy, color, clip),
                5 => canvas.fill_arc(x, y, cw % 64, ch % 64, sx, sy, color, clip),
                6 | 8 => canvas.draw(x, y, cw, ch, &*source, sx, sy, clip),
                7 => {
                    let image = canvas.image();
                    let copy = VecImageBuffer::<ArgbPixel>::from_raw(w, h, image.colors().into_iter().map(ArgbPixel::from_color).collect());
                    canvas.draw(x, y, cw, ch, &copy, sx, sy, clip)
                }
                _ => {}
            };
            let expected = match bpp {
                16 => reference::<Rgb565Pixel>(&pattern, w, h, &op),
                _ => reference::<ArgbPixel>(&pattern, w, h, &op),
            };

            match kind {
                0 => put_pixel(&mut context, &framebuffer, x, y, color, clip)?,
                1 => fill_rect(&mut context, &framebuffer, x, y, cw, ch, color, clip)?,
                2 => draw_line(&mut context, &framebuffer, x, y, x2, y2, color, clip)?,
                3 => draw_rect(&mut context, &framebuffer, x, y, cw, ch, color, clip)?,
                4 => draw_arc(&mut context, &framebuffer, x, y, cw % 64, ch % 64, sx, sy, color, clip)?,
                5 => fill_arc(&mut context, &framebuffer, x, y, cw % 64, ch % 64, sx, sy, color, clip)?,
                6 => draw_image(&mut context, &framebuffer, x, y, cw, ch, &*source, sx, sy, clip)?,
                7 => copy_area(&mut context, &framebuffer, x, y, cw, ch, sx, sy, clip)?,
                8 => copy_framebuffer(&mut context, &framebuffer, x, y, cw, ch, &source_framebuffer, sx, sy, clip)?,
                _ => {}
            }

            let actual = framebuffer.image(&mut context)?.raw().into_owned();
            assert!(actual == expected, "bpp {bpp} {w}x{h} round {round} kind {kind}");
        }
        Ok(())
    }

    #[test]
    fn drawing_through_a_region_matches_drawing_the_whole_buffer() -> Result<()> {
        for (seed, (w, h)) in [(64, 48), (9, 11), (31, 7), (1, 1)].into_iter().enumerate() {
            sweep(16, w, h, seed as u32 + 1)?;
            sweep(32, w, h, seed as u32 + 101)?;
        }
        Ok(())
    }
}
