mod lbmp;

use alloc::{borrow::Cow, boxed::Box, string::ToString, vec, vec::Vec};
use core::mem::size_of;

use ab_glyph::{Font, FontRef, ScaleFont};
use bytemuck::{Pod, cast_slice, pod_collect_to_vec};
use image::ImageReader;
use num_traits::{Num, Zero};

use wie_util::{Result, WieError};

use self::lbmp::decode_lbmp;

lazy_static::lazy_static! {
    static ref FONT: FontRef<'static> = FontRef::try_from_slice(include_bytes!("../../fonts/neodgm.ttf")).unwrap();
}

pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Image: Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes_per_pixel(&self) -> u32;
    fn get_pixel(&self, x: i32, y: i32) -> Color;
    fn raw(&self) -> Cow<'_, [u8]>;
    fn colors(&self) -> Vec<Color>;
}

pub trait ImageBuffer: Send {
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]);
}

#[allow(clippy::too_many_arguments)]
pub trait Canvas: Send {
    fn image(&self) -> &dyn Image;
    fn draw(&mut self, dx: i32, dy: i32, w: u32, h: u32, src: &dyn Image, sx: i32, sy: i32, clip: Clip);
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color);
    fn draw_text(&mut self, string: &str, x: i32, y: i32, text_alignment: TextAlignment, color: Color);
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip);
    fn draw_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: u32, arc_angle: u32, color: Color, clip: Clip);
    fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip);
    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip);
    fn fill_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: u32, arc_angle: u32, color: Color, clip: Clip);
    fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip);
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
}

pub trait PixelType: Send {
    type DataType: Copy + Pod + Num + Send;
    fn from_color(color: Color) -> Self::DataType;
    fn to_color(raw: Self::DataType) -> Color;
}

pub struct Rgb332Pixel;

impl PixelType for Rgb332Pixel {
    type DataType = u8;

    fn from_color(color: Color) -> Self::DataType {
        let r = ((color.r as u16 * 7 + 127) / 255) as u8;
        let g = ((color.g as u16 * 7 + 127) / 255) as u8;
        let b = ((color.b as u16 * 3 + 127) / 255) as u8;

        (r << 5) | (g << 2) | b
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = (raw >> 5) & 0x7;
        let g = (raw >> 2) & 0x7;
        let b = raw & 0x3;

        Color {
            a: 0xff,
            r: r * 36,
            g: g * 36,
            b: b * 85,
        }
    }
}

pub struct Rgb565Pixel;

impl PixelType for Rgb565Pixel {
    type DataType = u16;

    fn from_color(color: Color) -> Self::DataType {
        let r = (color.r as u16) >> 3;
        let g = (color.g as u16) >> 2;
        let b = (color.b as u16) >> 3;

        (r << 11) | (g << 5) | b
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = ((raw >> 11) & 0x1f) as u8;
        let g = ((raw >> 5) & 0x3f) as u8;
        let b = (raw & 0x1f) as u8;

        let r = ((r as u32 * 255 + 15) / 31) as u8;
        let g = ((g as u32 * 255 + 31) / 63) as u8;
        let b = ((b as u32 * 255 + 15) / 31) as u8;

        Color { a: 0xff, r, g, b }
    }
}

pub struct Rgb8Pixel;

impl PixelType for Rgb8Pixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let b = (raw & 0xff) as u8;

        Color { a: 0xff, r, g, b }
    }
}

pub struct ArgbPixel;

impl PixelType for ArgbPixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        ((color.a as u32) << 24) | ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let a = ((raw >> 24) & 0xff) as u8;
        let r = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let b = (raw & 0xff) as u8;

        Color { a, r, g, b }
    }
}

pub struct AbgrPixel;

impl PixelType for AbgrPixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        ((color.a as u32) << 24) | ((color.b as u32) << 16) | ((color.g as u32) << 8) | color.r as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let a = ((raw >> 24) & 0xff) as u8;
        let b = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let r = (raw & 0xff) as u8;

        Color { a, r, g, b }
    }
}

pub struct VecImageBuffer<T>
where
    T: PixelType,
{
    width: u32,
    height: u32,
    data: Vec<T::DataType>,
}

impl<T> VecImageBuffer<T>
where
    T: PixelType,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![T::DataType::zero(); (width * height) as usize],
        }
    }

    pub fn from_raw(width: u32, height: u32, raw: Vec<T::DataType>) -> Self {
        Self { width, height, data: raw }
    }
}

impl<T> Image for VecImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn bytes_per_pixel(&self) -> u32 {
        size_of::<T::DataType>() as u32
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        let raw = self.data[((y as u32) * self.width + (x as u32)) as usize];

        T::to_color(raw)
    }

    fn raw(&self) -> Cow<'_, [u8]> {
        cast_slice(&self.data).into()
    }

    fn colors(&self) -> Vec<Color> {
        self.data.iter().map(|&x| T::to_color(x)).collect()
    }
}

impl<T> ImageBuffer for VecImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
            return;
        }

        let raw = T::from_color(color);

        self.data[((y as u32) * self.width + (x as u32)) as usize] = raw;
    }

    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]) {
        for (i, color) in colors.iter().enumerate() {
            let x = x + (i as i32 % (width as i32));
            let y = y + (i as i32 / (width as i32));

            if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
                continue;
            }

            let raw = T::from_color(*color);

            self.data[((y as u32) * self.width + (x as u32)) as usize] = raw;
        }
    }
}

pub struct ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    image_buffer: T,
}

impl<T> ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    pub fn new(image_buffer: T) -> Self {
        Self { image_buffer }
    }

    pub fn into_inner(self) -> T {
        self.image_buffer
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || (x as u32) >= self.image_buffer.width() || (y as u32) >= self.image_buffer.height() {
            return;
        }
        let bg = self.image_buffer.get_pixel(x, y);
        let factor = color.a as f32 / 255.0;

        let computed_color = Color {
            a: 0xff,
            r: (color.r as f32 * factor + bg.r as f32 * (1.0 - factor)) as u8,
            g: (color.g as f32 * factor + bg.g as f32 * (1.0 - factor)) as u8,
            b: (color.b as f32 * factor + bg.b as f32 * (1.0 - factor)) as u8,
        };

        self.put_pixel(x, y, computed_color);
    }
}

#[allow(clippy::too_many_arguments)]
impl<T> Canvas for ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    fn image(&self) -> &dyn Image {
        &self.image_buffer
    }

    fn draw(&mut self, dx: i32, dy: i32, w: u32, h: u32, src: &dyn Image, sx: i32, sy: i32, clip: Clip) {
        for y in 0..(h as i32) {
            for x in 0..(w as i32) {
                if sx + x < 0 || sy + y < 0 || sx + x >= src.width() as i32 || sy + y >= src.height() as i32 {
                    continue;
                }
                if dx + x < 0 || dy + y < 0 || dx + x >= self.image_buffer.width() as i32 || dy + y >= self.image_buffer.height() as i32 {
                    continue;
                }
                if dx + x < clip.x || dx + x >= clip.x + (clip.width as i32) || dy + y < clip.y || dy + y >= clip.y + (clip.height as i32) {
                    continue;
                }

                // TODO blend multiple pixels at once for performance
                self.blend_pixel(dx + x, dy + y, src.get_pixel(sx + x, sy + y));
            }
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        if x1 == x2 && y1 == y2 {
            self.blend_pixel(x1 as _, y1 as _, color);
            return;
        }

        // bresenham's line drawing
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
            self.blend_pixel(x as _, y as _, color);

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_text(&mut self, string: &str, x: i32, y: i32, text_alignment: TextAlignment, color: Color) {
        let size = 10.0; // TODO
        let font = FONT.as_scaled(FONT.pt_to_px_scale(size).unwrap());

        let total_width = string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>();
        let x = match text_alignment {
            TextAlignment::Left => x,
            TextAlignment::Center => x - (total_width / 2.0) as i32,
            TextAlignment::Right => x - total_width as i32,
        };

        let mut position = 0.0;
        for c in string.chars() {
            if c.is_control() {
                continue;
            }

            let glyph = font.scaled_glyph(c);
            let h_advance = font.h_advance(glyph.id);

            if let Some(outlined_glyph) = font.outline_glyph(glyph) {
                outlined_glyph.draw(|glyph_x: u32, glyph_y, c| {
                    let bounds = outlined_glyph.px_bounds();
                    self.blend_pixel(
                        x + (glyph_x as f32 + bounds.min.x + position) as i32,
                        y + (glyph_y as f32 + bounds.min.y + size) as i32,
                        Color {
                            a: (c * 255.0) as u8,
                            r: color.r,
                            g: color.g,
                            b: color.b,
                        },
                    )
                });
            }

            position += h_advance;
        }
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip) {
        // TODO use put_pixels
        for x in x..x + (w as i32) {
            if x < 0 || x >= self.image_buffer.width() as i32 {
                continue;
            }
            if x < clip.x || x >= clip.x + clip.width as i32 {
                continue;
            }
            if y < 0 || y >= self.image_buffer.height() as i32 {
                continue;
            }
            if y < clip.y || y >= clip.y + clip.height as i32 {
                continue;
            }

            self.put_pixel(x, y, color);
            self.put_pixel(x, y + (h as i32) - 1, color);
        }
        for y in y..y + (h as i32) {
            if x < 0 || x >= self.image_buffer.width() as i32 {
                continue;
            }
            if x < clip.x || x >= clip.x + clip.width as i32 {
                continue;
            }
            if y < 0 || y >= self.image_buffer.height() as i32 {
                continue;
            }
            if y < clip.y || y >= clip.y + clip.height as i32 {
                continue;
            }

            self.put_pixel(x, y, color);
            self.put_pixel(x + (w as i32) - 1, y, color);
        }
    }

    fn draw_arc(&mut self, x: i32, y: i32, w: u32, h: u32, _start_angle: u32, _arc_angle: u32, color: Color, clip: Clip) {
        // TODO unimplemented
        self.draw_rect(x, y, w, h, color, clip);
    }

    fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, _arc_width: u32, _arc_height: u32, color: Color, clip: Clip) {
        // TODO unimplemented
        self.draw_rect(x, y, w, h, color, clip);
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip) {
        // TODO use put_pixels
        for y in y..y + (h as i32) {
            for x in x..x + (w as i32) {
                if x >= self.image_buffer.width() as i32 || y >= self.image_buffer.height() as i32 {
                    continue;
                }
                if x < clip.x || x >= clip.x + clip.width as i32 || y < clip.y || y >= clip.y + clip.height as i32 {
                    continue;
                }
                self.put_pixel(x, y, color);
            }
        }
    }

    fn fill_arc(&mut self, x: i32, y: i32, w: u32, h: u32, _start_angle: u32, _arc_angle: u32, color: Color, clip: Clip) {
        // TODO unimplemented
        self.fill_rect(x, y, w, h, color, clip);
    }

    fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, _arc_width: u32, _arc_height: u32, color: Color, clip: Clip) {
        // TODO unimplemented
        self.fill_rect(x, y, w, h, color, clip);
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.image_buffer.put_pixel(x, y, color)
    }
}

pub struct Clip {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Clip {
    pub fn intersect(&self, other: &Clip) -> Clip {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x + (self.width as i32)).min(other.x + (other.width as i32)) - x;
        let height = (self.y + (self.height as i32)).min(other.y + (other.height as i32)) - y;

        Clip {
            x,
            y,
            width: width as _,
            height: height as _,
        }
    }
}

pub fn decode_image(data: &[u8]) -> Result<Box<dyn Image>> {
    extern crate std; // XXX

    use std::io::Cursor;

    if data.len() >= 4 && data[0] == b'L' && data[1] == b'B' && data[2] == b'M' && data[3] == b'P' {
        return decode_lbmp(data);
    }

    let image = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()
        .map_err(|x| WieError::FatalError(x.to_string()))?
        .decode()
        .map_err(|x| WieError::FatalError(x.to_string()))?;
    let rgba = image.into_rgba8();

    let data = rgba.pixels().flat_map(|x| [x.0[2], x.0[1], x.0[0], x.0[3]]).collect::<Vec<_>>();

    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(
        rgba.width(),
        rgba.height(),
        pod_collect_to_vec(&data),
    )) as Box<_>)
}

pub fn string_width(string: &str, pt_size: f32) -> f32 {
    let font = FONT.as_scaled(FONT.pt_to_px_scale(pt_size).unwrap());

    string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>()
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;

    use wie_util::Result;

    use crate::canvas::{Clip, Image, ImageBufferCanvas, decode_image};

    use super::{AbgrPixel, ArgbPixel, Canvas, Color, PixelType, Rgb8Pixel, Rgb332Pixel, Rgb565Pixel, VecImageBuffer};

    // 2x2 RGBA PNG: (255,0,0,255) (0,255,0,128) / (0,0,255,255) (255,255,255,0)
    const PNG_2X2: [u8; 78] = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00,
        0x02, 0x08, 0x06, 0x00, 0x00, 0x00, 0x72, 0xb6, 0x0d, 0x24, 0x00, 0x00, 0x00, 0x15, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0xf8, 0xcf,
        0xc0, 0xf0, 0x1f, 0x08, 0x1b, 0x18, 0x80, 0x34, 0x08, 0x30, 0x00, 0x00, 0x43, 0xd3, 0x08, 0x79, 0xc9, 0x15, 0x1c, 0x0f, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn test_decode_image_short_input_returns_err() {
        for data in [&[] as &[u8], &[b'L'], &[b'L', b'B'], &[b'L', b'B', b'M']] {
            let result = decode_image(data);
            assert!(result.is_err(), "expected Err for {}-byte input", data.len());
        }
    }

    #[test]
    fn test_decode_lbmp_truncated_header_returns_err() {
        // valid tag but header shorter than 24 bytes
        for len in 4..24 {
            let mut data = Vec::new();
            data.extend_from_slice(b"LBMP");
            data.resize(len, 0);

            let result = decode_image(&data);
            assert!(result.is_err(), "expected Err for {len}-byte LBMP input");
        }
    }

    #[test]
    fn test_decode_image_png() -> Result<()> {
        let image = decode_image(&PNG_2X2)?;

        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);
        assert_eq!(image.bytes_per_pixel(), 4);

        let expected = [(255, 255, 0, 0), (128, 0, 255, 0), (255, 0, 0, 255), (0, 255, 255, 255)];
        for (i, &(a, r, g, b)) in expected.iter().enumerate() {
            let color = image.get_pixel((i % 2) as i32, (i / 2) as i32);
            assert_eq!((color.a, color.r, color.g, color.b), (a, r, g, b), "pixel {i}");
        }

        // raw is stored BGRA per pixel (little-endian ArgbPixel)
        let raw = image.raw();
        assert_eq!(&raw[0..4], &[0, 0, 255, 255]);
        assert_eq!(&raw[4..8], &[0, 255, 0, 128]);

        Ok(())
    }

    #[test]
    fn test_decode_lbmp_rgb332() -> Result<()> {
        let mut data = Vec::new();
        data.extend_from_slice(b"LBMP"); // descriptor
        data.extend_from_slice(&8u32.to_le_bytes()); // type
        data.extend_from_slice(&2u32.to_le_bytes()); // width
        data.extend_from_slice(&2u32.to_le_bytes()); // height
        data.extend_from_slice(&4u32.to_le_bytes()); // size
        data.extend_from_slice(&0u32.to_le_bytes()); // mask
        data.extend_from_slice(&[0b111_000_00, 0b000_111_00, 0b000_000_11, 0xff]);

        let image = decode_image(&data)?;

        assert_eq!(image.width(), 2);
        assert_eq!(image.height(), 2);
        assert_eq!(image.bytes_per_pixel(), 1);

        let expected = [(252, 0, 0), (0, 252, 0), (0, 0, 255), (252, 252, 255)];
        for (i, &(r, g, b)) in expected.iter().enumerate() {
            let color = image.get_pixel((i % 2) as i32, (i / 2) as i32);
            assert_eq!((color.r, color.g, color.b), (r, g, b), "pixel {i}");
        }

        Ok(())
    }

    fn gray(v: u8) -> Color {
        Color { a: 0xff, r: v, g: v, b: v }
    }

    #[test]
    fn test_rgb332_roundtrip() {
        // (input gray value, expected raw, expected roundtrip rgb)
        let cases = [
            (0, 0b000_000_00, (0, 0, 0)),
            (18, 0b000_000_00, (0, 0, 0)),
            (19, 0b001_001_00, (36, 36, 0)),
            (128, 0b100_100_10, (144, 144, 170)),
            (255, 0b111_111_11, (252, 252, 255)),
        ];

        for (v, raw, (r, g, b)) in cases {
            assert_eq!(Rgb332Pixel::from_color(gray(v)), raw, "from_color({v})");
            let color = Rgb332Pixel::to_color(raw);
            assert_eq!((color.a, color.r, color.g, color.b), (0xff, r, g, b), "to_color({raw:#010b})");
        }
    }

    #[test]
    fn test_rgb565_roundtrip() {
        let cases = [
            (0, (0, 0, 0)),
            (18, (16, 16, 16)),
            (19, (16, 16, 16)),
            (128, (132, 130, 132)),
            (255, (255, 255, 255)),
        ];

        for (v, (r, g, b)) in cases {
            let color = Rgb565Pixel::to_color(Rgb565Pixel::from_color(gray(v)));
            assert_eq!((color.a, color.r, color.g, color.b), (0xff, r, g, b), "roundtrip({v})");
        }
    }

    #[test]
    fn test_rgb8_roundtrip() {
        for v in [0, 18, 19, 128, 255] {
            let color = Rgb8Pixel::to_color(Rgb8Pixel::from_color(gray(v)));
            assert_eq!((color.a, color.r, color.g, color.b), (0xff, v, v, v), "roundtrip({v})");
        }
    }

    #[test]
    fn test_argb_abgr_roundtrip() {
        for v in [0, 18, 19, 128, 255] {
            let input = Color {
                a: v,
                r: v,
                g: 255 - v,
                b: v,
            };

            let color = ArgbPixel::to_color(ArgbPixel::from_color(input));
            assert_eq!((color.a, color.r, color.g, color.b), (v, v, 255 - v, v), "argb roundtrip({v})");

            let color = AbgrPixel::to_color(AbgrPixel::from_color(input));
            assert_eq!((color.a, color.r, color.g, color.b), (v, v, 255 - v, v), "abgr roundtrip({v})");
        }
    }

    #[test]
    fn test_canvas() -> Result<()> {
        let image_buffer = VecImageBuffer::<ArgbPixel>::new(10, 10);
        let mut canvas = ImageBufferCanvas::new(image_buffer);

        let clip = Clip {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        canvas.fill_rect(0, 0, 10, 10, Color { r: 0, g: 0, b: 0, a: 255 }, clip);

        let image_buffer = canvas.into_inner();
        let raw = image_buffer.raw();

        assert_eq!(raw.len(), 10 * 10 * 4);
        for i in 0..10 * 10 {
            assert_eq!(raw[i * 4], 0);
            assert_eq!(raw[i * 4 + 1], 0);
            assert_eq!(raw[i * 4 + 2], 0);
            assert_eq!(raw[i * 4 + 3], 255);
        }

        Ok(())
    }
}
