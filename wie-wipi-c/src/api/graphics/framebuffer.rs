use alloc::{borrow::Cow, boxed::Box, vec, vec::Vec};
use core::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable, cast_slice, cast_slice_mut};

use wipi_types::wipic::{WIPICFramebuffer, WIPICIndirectPtr, WIPICWord};

use wie_backend::canvas::{ArgbPixel, Canvas, Clip, Color, Image, ImageBuffer, ImageBufferCanvas, PixelType, Rgb8Pixel, Rgb565Pixel, VecImageBuffer};
use wie_util::{Result, WieError};

use crate::context::WIPICContext;

// same 256MB as wie_core_arm's HEAP_SIZE; not referenced directly to avoid the dependency
const MAX_FRAMEBUFFER_BYTES: u32 = 0x1000_0000;

fn buffer_size(width: u32, height: u32, bytes_per_pixel: u32) -> Result<(u32, u32)> {
    let bpl = width.checked_mul(bytes_per_pixel).ok_or(WieError::AllocationFailure)?;
    let size = bpl.checked_mul(height).ok_or(WieError::AllocationFailure)?;
    if size > MAX_FRAMEBUFFER_BYTES {
        return Err(WieError::AllocationFailure);
    }

    Ok((size, bpl))
}

pub struct FrameBuffer(pub WIPICFramebuffer);

impl FrameBuffer {
    pub fn empty() -> Self {
        Self(WIPICFramebuffer {
            width: 0,
            height: 0,
            bpl: 0,
            bpp: 0,
            buf: WIPICIndirectPtr(0),
        })
    }

    pub fn new(context: &mut dyn WIPICContext, width: WIPICWord, height: WIPICWord, bpp: WIPICWord) -> Result<Self> {
        let bytes_per_pixel = bpp / 8;

        let (size, bpl) = buffer_size(width, height, bytes_per_pixel)?;
        let buf = context.alloc(size)?;

        Ok(Self(WIPICFramebuffer {
            width,
            height,
            bpl,
            bpp: bytes_per_pixel * 8,
            buf,
        }))
    }

    pub fn from_image(context: &mut dyn WIPICContext, image: &dyn Image) -> Result<Self> {
        let (size, bpl) = buffer_size(image.width(), image.height(), image.bytes_per_pixel())?;
        let buf = context.alloc(size)?;

        context.write_bytes(context.data_ptr(buf)?, &image.raw())?;

        Ok(Self(WIPICFramebuffer {
            width: image.width(),
            height: image.height(),
            bpl,
            bpp: image.bytes_per_pixel() * 8,
            buf,
        }))
    }

    fn data<T: Pod>(&self, context: &dyn WIPICContext) -> Result<Vec<T>> {
        let (size, _) = buffer_size(self.0.width, self.0.height, self.0.bpp / 8)?;
        let mut buf = vec![T::zeroed(); size as usize / size_of::<T>()];
        context.read_bytes(context.data_ptr(self.0.buf)?, cast_slice_mut(&mut buf))?;

        Ok(buf)
    }

    pub fn image(&self, context: &mut dyn WIPICContext) -> Result<Box<dyn Image>> {
        Ok(match self.0.bpp {
            16 => Box::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
                self.0.width as _,
                self.0.height as _,
                self.data(context)?,
            )),
            32 => Box::new(VecImageBuffer::<ArgbPixel>::from_raw(
                self.0.width as _,
                self.0.height as _,
                self.data(context)?,
            )),
            _ => unimplemented!("Unsupported pixel format: {}", self.0.bpp),
        })
    }

    /// A canvas over `region` only (already clipped to the framebuffer): just those pixels are read from
    /// guest memory and written back. Drawing must stay inside `region` — pass it as the clip.
    /// Copying the whole buffer per call was 65% of one LGT title's frame (docs/report/0501).
    pub fn canvas<'a>(&'a self, context: &'a mut dyn WIPICContext, region: Clip) -> Result<FramebufferCanvas<'a>> {
        let canvas: Box<dyn Canvas> = match self.0.bpp {
            16 => Box::new(ImageBufferCanvas::new(self.window::<Rgb565Pixel>(context, region)?)),
            32 => Box::new(ImageBufferCanvas::new(self.window::<ArgbPixel>(context, region)?)),
            _ => unimplemented!("Unsupported pixel format: {}", self.0.bpp),
        };

        Ok(FramebufferCanvas {
            framebuffer: self,
            context,
            canvas,
            region,
            flushed: false,
        })
    }

    /// Guest address and byte length of each row of `region`; rows are `width * bpp` apart, as `data()` reads them.
    fn region_rows(&self, context: &dyn WIPICContext, region: Clip) -> Result<impl Iterator<Item = (u32, usize)> + use<>> {
        let bytes_per_pixel = self.0.bpp / 8;
        let (_, stride) = buffer_size(self.0.width, self.0.height, bytes_per_pixel)?;
        let base = context.data_ptr(self.0.buf)? + region.y as u32 * stride + region.x as u32 * bytes_per_pixel;
        let row_bytes = (region.width * bytes_per_pixel) as usize;
        // full-width rows are contiguous: one transfer
        let (count, len) = if region.width == self.0.width {
            (1, row_bytes * region.height as usize)
        } else {
            (region.height, row_bytes)
        };

        Ok((0..count).map(move |row| (base + row * stride, len)))
    }

    fn window<P: PixelType>(&self, context: &dyn WIPICContext, region: Clip) -> Result<WindowImageBuffer<P>> {
        let mut data = vec![<P::DataType as Zeroable>::zeroed(); (region.width * region.height) as usize];
        let bytes = cast_slice_mut::<_, u8>(&mut data);
        let mut offset = 0;
        for (address, len) in self.region_rows(context, region)? {
            context.read_bytes(address, &mut bytes[offset..offset + len])?;
            offset += len;
        }

        Ok(WindowImageBuffer {
            width: self.0.width,
            height: self.0.height,
            region,
            data,
        })
    }

    fn write_region(&self, context: &mut dyn WIPICContext, region: Clip, bytes: &[u8]) -> Result<()> {
        let mut offset = 0;
        for (address, len) in self.region_rows(context, region)? {
            context.write_bytes(address, &bytes[offset..offset + len])?;
            offset += len;
        }

        Ok(())
    }

    pub fn write(&self, context: &mut dyn WIPICContext, data: &[u8]) -> Result<()> {
        context.write_bytes(context.data_ptr(self.0.buf)?, data)
    }

    pub fn pixel_to_color(&self, pixel: WIPICWord) -> Color {
        match self.0.bpp {
            16 => Rgb565Pixel::to_color(pixel as u16),
            _ => Rgb8Pixel::to_color(pixel),
        }
    }
}

pub struct FramebufferCanvas<'a> {
    framebuffer: &'a FrameBuffer,
    context: &'a mut dyn WIPICContext,
    canvas: Box<dyn Canvas>,
    region: Clip,
    flushed: bool,
}

impl FramebufferCanvas<'_> {
    pub fn flush(mut self) -> Result<()> {
        self.flushed = true;

        self.framebuffer.write_region(self.context, self.region, &self.canvas.image().raw())
    }
}

// best-effort fallback for canvases dropped without an explicit flush
impl Drop for FramebufferCanvas<'_> {
    fn drop(&mut self) {
        if self.flushed {
            return;
        }

        tracing::warn!("framebuffer canvas dropped without explicit flush; write-back errors will be lost");

        if let Err(err) = self.framebuffer.write_region(self.context, self.region, &self.canvas.image().raw()) {
            tracing::error!("Failed to flush framebuffer canvas: {err}");
        }
    }
}

/// A framebuffer-sized image that holds only `region`'s pixels. Bounds, and so every clipping decision
/// a canvas makes, are the whole framebuffer's; pixels outside `region` read as zero and drop writes.
/// `raw()` is `region`'s pixels row by row — what `write_region` takes back.
struct WindowImageBuffer<P: PixelType> {
    width: u32,
    height: u32,
    region: Clip,
    data: Vec<P::DataType>,
}

impl<P: PixelType> WindowImageBuffer<P> {
    fn index(&self, x: i32, y: i32) -> Option<usize> {
        let (x, y) = (x.checked_sub(self.region.x)?, y.checked_sub(self.region.y)?);
        (x >= 0 && y >= 0 && (x as u32) < self.region.width && (y as u32) < self.region.height)
            .then(|| (y as u32 * self.region.width + x as u32) as usize)
    }
}

impl<P: PixelType + 'static> Image for WindowImageBuffer<P> {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn bytes_per_pixel(&self) -> u32 {
        size_of::<P::DataType>() as u32
    }

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        P::to_color(self.index(x, y).map_or(<P::DataType as Zeroable>::zeroed(), |i| self.data[i]))
    }

    fn raw(&self) -> Cow<'_, [u8]> {
        cast_slice(&self.data).into()
    }

    fn colors(&self) -> Vec<Color> {
        self.data.iter().map(|&x| P::to_color(x)).collect()
    }
}

impl<P: PixelType + 'static> ImageBuffer for WindowImageBuffer<P> {
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if let Some(i) = self.index(x, y) {
            self.data[i] = P::from_color(color);
        }
    }

    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]) {
        for (i, color) in colors.iter().enumerate() {
            self.put_pixel(x + (i as i32 % width as i32), y + (i as i32 / width as i32), *color);
        }
    }

    fn xor_pixel(&mut self, x: i32, y: i32, color: Color) {
        if let Some(i) = self.index(x, y) {
            self.data[i] = P::xor_color(self.data[i], color);
        }
    }
}

impl Deref for FramebufferCanvas<'_> {
    type Target = Box<dyn Canvas>;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for FramebufferCanvas<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}

#[cfg(test)]
mod test {
    use wie_util::WieError;

    use crate::context::test::TestContext;

    use super::FrameBuffer;

    #[test]
    fn test_new_overflow_returns_error() {
        let mut context = TestContext::new();

        assert!(matches!(
            FrameBuffer::new(&mut context, 0x10000, 0x10000, 32),
            Err(WieError::AllocationFailure)
        ));
    }

    #[test]
    fn test_new_over_heap_limit_returns_error() {
        let mut context = TestContext::new();

        assert!(matches!(
            FrameBuffer::new(&mut context, 0x4000, 0x4000, 32),
            Err(WieError::AllocationFailure)
        ));
    }

    #[test]
    fn test_new_zero_height_bpl_overflow_returns_error() {
        let mut context = TestContext::new();

        assert!(matches!(
            FrameBuffer::new(&mut context, 0xffff_ffff, 0, 32),
            Err(WieError::AllocationFailure)
        ));
    }

    #[test]
    fn test_new_normal_size_ok() {
        let mut context = TestContext::new();

        for bpp in [16, 32] {
            let framebuffer = FrameBuffer::new(&mut context, 100, 100, bpp).unwrap();
            assert_eq!(framebuffer.0.width, 100);
            assert_eq!(framebuffer.0.height, 100);
            assert_eq!(framebuffer.0.bpl, 100 * bpp / 8);
            assert_eq!(framebuffer.0.bpp, bpp);
            let pixels = (0..100 * 100 * bpp / 8).map(|i| i as u8).collect::<alloc::vec::Vec<_>>();
            framebuffer.write(&mut context, &pixels).unwrap();
            assert_eq!(&*framebuffer.image(&mut context).unwrap().raw(), pixels.as_slice());
            let canvas = framebuffer
                .canvas(
                    &mut context,
                    wie_backend::canvas::Clip {
                        x: 0,
                        y: 0,
                        width: 100,
                        height: 100,
                    },
                )
                .unwrap();
            assert_eq!(&*canvas.image().raw(), pixels.as_slice());
            canvas.flush().unwrap();
            assert_eq!(&*framebuffer.image(&mut context).unwrap().raw(), pixels.as_slice());
        }
    }
}
