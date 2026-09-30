mod framebuffer;
mod grp_context;
mod image;
pub mod primitives;

pub use framebuffer::FrameBuffer;
pub use grp_context::WIPICGraphicsContextIdx;
pub use image::decode_image_framebuffer;

use alloc::vec::Vec;
use core::mem::size_of;

use wie_backend::{
    Event,
    canvas::{ArgbPixel, Clip, Color, Image, ImageBuffer, PixelType, Rgb565Pixel, string_width},
};
use wie_util::{Result, read_generic, write_generic};

use wipi_types::wipic::{WIPICDisplayInfo, WIPICFramebuffer, WIPICGraphicsContext, WIPICImage, WIPICIndirectPtr, WIPICWord};

use crate::context::WIPICContext;

use self::image::create_wipi_image;

const FRAMEBUFFER_DEPTH: u32 = 16; // XXX hardcode to 16bpp as some game requires 16bpp framebuffer
const SCREEN_FRAMEBUFFER_PTR: u32 = 0x7fff1000;

pub async fn get_screen_framebuffer(context: &mut dyn WIPICContext, a0: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetScreenFrameBuffer({a0:#x})");

    let framebuffer_ptr: u32 = read_generic(context, SCREEN_FRAMEBUFFER_PTR)?;
    if framebuffer_ptr != 0 {
        return Ok(WIPICIndirectPtr(framebuffer_ptr));
    }

    let (width, height) = {
        let platform = context.system().platform();
        let screen = platform.screen();
        (screen.width(), screen.height())
    };

    let framebuffer = FrameBuffer::new(context, width, height, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer.0)?;
    write_generic(context, SCREEN_FRAMEBUFFER_PTR, memory.0)?;

    Ok(memory)
}

/// Keeps the native screen framebuffer and the Java screen image one picture, at each Java paint
/// (KTF Java mode).
///
/// On the handset they are one memory, so a title may draw natively, call `Card.repaint`, and never
/// `MC_grpFlushLcd` (docs/report/0361). Here they are two buffers, synced both ways per paint: native
/// pixels changed since the last sync go onto the Java image, and Java pixels changed since then go
/// into the native framebuffer. A title that draws in Java and merely holds the framebuffer keeps
/// its picture; one that clears the screen in Java and redraws natively in the same colour — which
/// no value diff sees — still gets its native picture back.
// ponytail: a pixel both sides changed between two paints goes to native whatever the real order
// was; a per-draw dirty log would settle it if a title ever shows it.
#[derive(Default)]
pub struct ScreenFramebufferSync {
    native: Vec<u8>,
    java: Vec<u8>,
}

impl ScreenFramebufferSync {
    /// Returns whether any native pixel went onto `target`.
    pub fn compose(&mut self, context: &mut dyn WIPICContext, current: &dyn Image, target: &mut dyn ImageBuffer) -> Result<bool> {
        let handle: u32 = read_generic(context, SCREEN_FRAMEBUFFER_PTR)?;
        if handle == 0 {
            return Ok(false);
        }

        self.compose_framebuffer(context, WIPICIndirectPtr(handle), current, target)
    }

    fn compose_framebuffer(
        &mut self,
        context: &mut dyn WIPICContext,
        handle: WIPICIndirectPtr,
        current: &dyn Image,
        target: &mut dyn ImageBuffer,
    ) -> Result<bool> {
        let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(handle)?)?);
        let native = framebuffer.image(context)?;
        let mut native_raw = native.raw().into_owned();
        let java_raw = current.raw().into_owned();
        if self.native.len() != native_raw.len() || self.java.len() != java_raw.len() {
            // first sight: all of native counts as changed, black included
            self.native = native_raw.iter().map(|x| !x).collect();
            self.java = java_raw.clone();
        }

        let (nb, jb) = (native.bytes_per_pixel() as usize, current.bytes_per_pixel() as usize);
        let (nw, jw) = (native.width() as usize, current.width() as usize);
        let width = nw.min(jw);
        let height = native.height().min(current.height()) as usize;
        let mut mirrored = false;
        let mut drawn = false;
        for y in 0..height {
            let at_native = |x: usize| (y * nw + x) * nb..(y * nw + x + 1) * nb;
            let at_java = |x: usize| (y * jw + x) * jb..(y * jw + x + 1) * jb;
            let mut x = 0;
            while x < width {
                let start = x;
                while x < width && native_raw[at_native(x)] != self.native[at_native(x)] {
                    x += 1;
                }
                if x > start {
                    let colors: Vec<Color> = (start..x).map(|x| native.get_pixel(x as _, y as _)).collect();
                    target.put_pixels(start as _, y as _, (x - start) as _, &colors);
                    drawn = true;
                    continue;
                }

                if java_raw[at_java(x)] != self.java[at_java(x)] {
                    let color = current.get_pixel(x as _, y as _);
                    let pixel = match nb {
                        2 => bytemuck::bytes_of(&Rgb565Pixel::from_color(color)).to_vec(),
                        _ => bytemuck::bytes_of(&ArgbPixel::from_color(color)).to_vec(),
                    };
                    native_raw[at_native(x)].copy_from_slice(&pixel);
                    mirrored = true;
                }
                x += 1;
            }
        }

        if mirrored {
            framebuffer.write(context, &native_raw)?;
        }
        self.native = native_raw;
        self.java = current.raw().into_owned();

        Ok(drawn)
    }
}

pub async fn init_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpInitContext({p_grp_ctx:#x})");

    let grp_ctx = WIPICGraphicsContext::default();
    write_generic(context, p_grp_ctx, grp_ctx)?;
    Ok(())
}

pub async fn set_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpSetContext({p_grp_ctx:#x}, {op:?}, {pv:#x})");

    let mut grp_ctx: WIPICGraphicsContext = read_generic(context, p_grp_ctx)?;
    match op {
        WIPICGraphicsContextIdx::ClipIdx => {
            let clip: [i32; 4] = read_generic(context, pv)?;
            grp_ctx.clip = clip.map(|value| value as u16);
        }
        WIPICGraphicsContextIdx::FgPixelIdx => {
            grp_ctx.fgpxl = pv as _;
        }
        WIPICGraphicsContextIdx::BgPixelIdx => {
            grp_ctx.bgpxl = pv as _;
        }
        WIPICGraphicsContextIdx::TransPixelIdx => {
            grp_ctx.transpxl = pv as _;
        }
        WIPICGraphicsContextIdx::AlphaIdx => {
            grp_ctx.alpha = pv as _;
            // grp_ctx.pixel_op_func_ptr = todo!();
            // grp_ctx.param1 = todo!();
        }
        WIPICGraphicsContextIdx::PixelopIdx => {
            grp_ctx.pixel_op_func_ptr = pv;
        }
        WIPICGraphicsContextIdx::PixelParam1Idx => {
            grp_ctx.param1 = pv;
        }
        WIPICGraphicsContextIdx::FontIdx => {
            grp_ctx.font = pv;
        }
        WIPICGraphicsContextIdx::StyleIdx => {
            grp_ctx.style = pv;
        }
        WIPICGraphicsContextIdx::OffsetIdx => {
            let offset: [i32; 2] = read_generic(context, pv)?;
            grp_ctx.offset = offset.map(|value| value as u16);
        }
        _ => {
            tracing::warn!("MC_grpSetContext({p_grp_ctx:#x}, {op:?}, {pv:#x}): ignoring invalid op");
        }
    }
    write_generic(context, p_grp_ctx, grp_ctx)?;

    Ok(())
}

pub async fn get_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpGetContext({p_grp_ctx:#x}, {op:?}, {pv:#x})");

    let grp_ctx: WIPICGraphicsContext = read_generic(context, p_grp_ctx)?;
    match op {
        WIPICGraphicsContextIdx::ClipIdx => write_generic(context, pv, grp_ctx.clip.map(|value| i32::from(value as i16)))?,
        WIPICGraphicsContextIdx::FgPixelIdx => write_generic(context, pv, grp_ctx.fgpxl)?,
        WIPICGraphicsContextIdx::BgPixelIdx => write_generic(context, pv, grp_ctx.bgpxl)?,
        WIPICGraphicsContextIdx::TransPixelIdx => write_generic(context, pv, grp_ctx.transpxl)?,
        WIPICGraphicsContextIdx::AlphaIdx => write_generic(context, pv, grp_ctx.alpha)?,
        WIPICGraphicsContextIdx::PixelopIdx => write_generic(context, pv, grp_ctx.pixel_op_func_ptr)?,
        WIPICGraphicsContextIdx::PixelParam1Idx => write_generic(context, pv, grp_ctx.param1)?,
        WIPICGraphicsContextIdx::FontIdx => write_generic(context, pv, grp_ctx.font)?,
        WIPICGraphicsContextIdx::StyleIdx => write_generic(context, pv, grp_ctx.style)?,
        WIPICGraphicsContextIdx::OffsetIdx => write_generic(context, pv, grp_ctx.offset.map(|value| i32::from(value as i16)))?,
        _ => tracing::warn!("MC_grpGetContext({p_grp_ctx:#x}, {op:?}, {pv:#x}): ignoring invalid op"),
    }

    Ok(())
}

pub async fn put_pixel(context: &mut dyn WIPICContext, dst_fb: WIPICIndirectPtr, x: i32, y: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpPutPixel({:#x}, {x}, {y}, {p_gctx:?})", dst_fb.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst_fb)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::put_pixel(
        context,
        &framebuffer,
        x as _,
        y as _,
        color,
        Clip {
            x: 0,
            y: 0,
            width: framebuffer.0.width,
            height: framebuffer.0.height,
        },
    )
}

pub async fn fill_rect(context: &mut dyn WIPICContext, dst_fb: WIPICIndirectPtr, x: i32, y: i32, w: i32, h: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpFillRect({:#x}, {x}, {y}, {w}, {h}, {p_gctx:#x})", dst_fb.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst_fb)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::fill_rect(context, &framebuffer, x, y, w as u32, h as u32, color, clip)
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    start_angle: i32,
    arc_angle: i32,
    p_gctx: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpDrawArc({:#x}, {x}, {y}, {w}, {h}, {start_angle}, {arc_angle}, {p_gctx:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_arc(context, &framebuffer, x, y, w as u32, h as u32, start_angle, arc_angle, color, clip)
}

#[allow(clippy::too_many_arguments)]
pub async fn fill_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    start_angle: i32,
    arc_angle: i32,
    p_gctx: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFillArc({:#x}, {x}, {y}, {w}, {h}, {start_angle}, {arc_angle}, {p_gctx:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::fill_arc(context, &framebuffer, x, y, w as u32, h as u32, start_angle, arc_angle, color, clip)
}

pub async fn create_image(
    context: &mut dyn WIPICContext,
    ptr_image: WIPICWord,
    image_data: WIPICIndirectPtr,
    offset: u32,
    len: u32,
) -> Result<WIPICWord> {
    tracing::debug!("MC_grpCreateImage({ptr_image:#x}, {:#x}, {offset}, {len})", image_data.0);

    let image = create_wipi_image(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<WIPICImage>() as WIPICWord)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(1) // MC_GRP_IMAGE_DONE
}

pub async fn destroy_image(context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyImage({:#x})", image.0);

    context.free(image)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_image(
    context: &mut dyn WIPICContext,
    framebuffer: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    image: WIPICIndirectPtr,
    sx: i32,
    sy: i32,
    graphics_context: WIPICWord,
) -> Result<()> {
    tracing::debug!(
        "MC_grpDrawImage({:#x}, {dx}, {dy}, {w}, {h}, {:#x}, {sx}, {sy}, {graphics_context:#x})",
        framebuffer.0,
        image.0
    );

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);
    let image: WIPICImage = read_generic(context, context.data_ptr(image)?)?;

    let src_image = FrameBuffer(image.img).image(context)?;
    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    primitives::draw_image(context, &framebuffer, dx, dy, w as u32, h as u32, &*src_image, sx, sy, clip)
}

pub async fn flush_lcd(
    context: &mut dyn WIPICContext,
    i: WIPICWord,
    framebuffer: WIPICIndirectPtr,
    x: WIPICWord,
    y: WIPICWord,
    w: WIPICWord,
    h: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFlushLcd({i:#x}, {:#x}, {x:#x}, {y:#x}, {w:#x}, {h:#x})", framebuffer.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);

    let src_canvas = framebuffer.image(context)?;

    let platform = context.system().platform();
    let screen = platform.screen();

    screen.paint(&*src_canvas);

    Ok(())
}

pub async fn get_pixel_from_rgb(_context: &mut dyn WIPICContext, r: i32, g: i32, b: i32) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetPixelFromRGB({r:#x}, {g:#x}, {b:#x})");
    if (r > 0xff) || (g > 0xff) | (b > 0xff) {
        tracing::debug!("MC_grpGetPixelFromRGB({r:#x}, {g:#x}, {b:#x}): value clipped to 8 bits");
    }

    let color = Rgb565Pixel::from_color(Color {
        a: 0xff,
        r: r as u8,
        g: g as u8,
        b: b as u8,
    });

    Ok(color as WIPICWord)
}

pub async fn get_rgb_from_pixel(context: &mut dyn WIPICContext, pixel: i32, r: WIPICWord, g: WIPICWord, b: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_grpGetRGBFromPixel({pixel}, {r:#x}, {g:#x}, {b:#x})");

    let color = Rgb565Pixel::to_color(pixel as u16);

    write_generic(context, r, color.r as i32)?;
    write_generic(context, g, color.g as i32)?;
    write_generic(context, b, color.b as i32)?;

    Ok(pixel)
}

pub async fn get_display_info(context: &mut dyn WIPICContext, reserved: WIPICWord, out_ptr: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetDisplayInfo({reserved:#x}, {out_ptr:#x})");

    assert_eq!(reserved, 0);

    let platform = context.system().platform();
    let screen = platform.screen();

    let info = WIPICDisplayInfo {
        bpp: FRAMEBUFFER_DEPTH,
        depth: 16,
        width: screen.width(),
        height: screen.height(),
        bpl: 2 * screen.width(),
        color_type: 1, // 1==MC_GRP_DIRECT_COLOR_TYPE
        red_mask: 0xf800,
        green_mask: 0x7e0,
        blue_mask: 0x1f,
    };

    write_generic(context, out_ptr, info)?;
    Ok(1)
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_area(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpCopyArea({:#x}, {dx}, {dy}, {w}, {h}, {x}, {y}, {pgc:#x})", dst.0);

    if w < 0 || h < 0 {
        tracing::warn!("Skipping negative dimension");

        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    primitives::copy_area(context, &framebuffer, dx, dy, w as u32, h as u32, x, y, clip)
}

pub async fn create_offscreen_framebuffer(context: &mut dyn WIPICContext, w: i32, h: i32) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpCreateOffScreenFrameBuffer({w}, {h})");

    let framebuffer = FrameBuffer::new(context, w as _, h as _, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer.0)?;

    Ok(memory)
}

pub async fn destroy_offscreen_framebuffer(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyOffScreenFrameBuffer({:#x})", framebuffer.0);

    context.free(framebuffer)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_frame_buffer(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    src: WIPICIndirectPtr,
    sx: i32,
    sy: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!(
        "MC_grpCopyFrameBuffer({:#x}, {dx}, {dy}, {w}, {h}, {:#x}, {sx}, {sy}, {pgc:#x})",
        dst.0,
        src.0
    );

    let src_framebuffer = FrameBuffer(read_generic(context, context.data_ptr(src)?)?);
    let dst_framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    primitives::copy_framebuffer(context, &dst_framebuffer, dx, dy, w as u32, h as u32, &src_framebuffer, sx, sy, clip)
}

pub async fn get_font(_: &mut dyn WIPICContext, face: i32, size: i32, style: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFont({face}, {size}, {style})");

    Ok(0)
}

pub async fn get_font_height(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontHeight({font})");

    Ok(12)
}

pub async fn get_font_ascent(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontAscent({font})");

    Ok(10)
}

pub async fn get_font_descent(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontDescent({font})");

    Ok(2)
}

pub async fn get_string_width(context: &mut dyn WIPICContext, font: i32, ptr_string: WIPICWord, length: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetStringWidth({font}, {ptr_string:#x}, {length})");

    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(0);
    };
    Ok(string_width(context.system().platform().font(), &string, 10.0) as i32)
}

pub async fn draw_string(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    ptr_string: WIPICWord,
    length: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpDrawString({:#x}, {x}, {y}, {ptr_string:#x}, {length}, {pgc:#x})", dst.0);

    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(());
    };

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, pgc)?;

    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_text(context, &framebuffer, &string, x, y, color, clip)
}

pub async fn repaint(context: &mut dyn WIPICContext, lcd: i32, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    tracing::debug!("MC_grpRepaint({lcd}, {x}, {y}, {width}, {height})");

    let platform = context.system().platform();
    let screen = platform.screen();
    screen.request_redraw().unwrap();

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn get_rgb_pixels(
    context: &mut dyn WIPICContext,
    src: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    pd: WIPICWord,
    ipl: i32,
) -> Result<()> {
    tracing::debug!("MC_grpGetRGBPixels({:#x}, {x}, {y}, {w}, {h}, {pd:#x}, {ipl})", src.0);
    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(src)?)?);
    let image = framebuffer.image(context)?;
    primitives::get_rgb_pixels(context, &*image, x, y, w, h, pd, ipl)
}

#[allow(clippy::too_many_arguments)]
pub async fn set_rgb_pixels(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    psrc: WIPICWord,
    ibpl: i32,
    _pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpSetRGBPixels({:#x}, {x}, {y}, {w}, {h}, {psrc:#x}, {ibpl})", dst.0);
    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };
    primitives::set_rgb_pixels(context, &framebuffer, x, y, w, h, psrc, ibpl, clip)
}

pub async fn get_image_framebuffer(_context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetImageFrameBuffer({:#x})", image.0);

    // WIPICImage starts with `img: WIPICFramebuffer` at offset 0,
    // so the image handle doubles as a framebuffer handle.
    Ok(image)
}

pub async fn get_image_property(context: &mut dyn WIPICContext, image: WIPICIndirectPtr, property: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetImageProperty({:#x}, {property})", image.0);

    let image: WIPICImage = read_generic(context, context.data_ptr(image)?)?;

    Ok(match property {
        4 => image.img.width as _,
        5 => image.img.height as _,
        _ => {
            tracing::warn!("unknown property {property}");
            0
        }
    })
}

pub async fn draw_rect(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x: i32, y: i32, w: i32, h: i32, pgc: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpDrawRect({:#x}, {x}, {y}, {w}, {h}, {pgc:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, pgc)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_rect(context, &framebuffer, x, y, w as u32, h as u32, color, clip)
}

pub async fn draw_line(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x1: i32, y1: i32, x2: i32, y2: i32, pgc: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpDrawLine({:#x}, {x1}, {y1}, {x2}, {y2}, {pgc:#x})", dst.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, pgc)?;
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width as _,
        height: framebuffer.0.height as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_line(context, &framebuffer, x1, y1, x2, y2, color, clip)
}

pub async fn post_event(context: &mut dyn WIPICContext, id: i32, r#type: i32, param1: i32, param2: i32) -> Result<i32> {
    tracing::debug!("MC_grpPostEvent({id}, {type}, {param1}, {param2})");

    context.system().event_queue().push(Event::Notify { r#type, param1, param2 });

    Ok(0)
}

/// Read a `WIPICFramebuffer` from an indirect handle, or `None` if the handle is
/// null (0).
///
/// LGT clets have been observed handing a 0 framebuffer handle to the framebuffer
/// info accessors below mid-render (놈ZERO, reached after repeated input). A 0
/// handle resolves to guest address 0, so the unconditional struct read faulted
/// ("Invalid memory access; address: 0") and killed the whole VM. A null handle is
/// invalid input, not a fatal condition on real hardware, so the accessors treat it
/// as an empty framebuffer and return a benign default, letting the game's input
/// loop continue. Valid (non-zero) handles are read byte-for-byte as before, so no
/// normal game's behaviour changes.
fn read_framebuffer_or_null(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<Option<WIPICFramebuffer>> {
    if framebuffer.0 == 0 {
        tracing::warn!("WIPI-C framebuffer accessor called with null (0) handle; treating as empty framebuffer");
        return Ok(None);
    }

    Ok(Some(read_generic(context, context.data_ptr(framebuffer)?)?))
}

// it's not documented api, but lgt apps gets pointer via api call
pub async fn get_framebuffer_pointer(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<WIPICWord> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_POINTER({:#x})", framebuffer.0);

    let Some(framebuffer) = read_framebuffer_or_null(context, framebuffer)? else {
        return Ok(0);
    };

    Ok(framebuffer.buf.0)
}

pub async fn get_framebuffer_width(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_WIDTH({:#x})", framebuffer.0);

    let Some(framebuffer) = read_framebuffer_or_null(context, framebuffer)? else {
        return Ok(0);
    };

    Ok(framebuffer.width as _)
}

pub async fn get_framebuffer_height(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_HEIGHT({:#x})", framebuffer.0);

    let Some(framebuffer) = read_framebuffer_or_null(context, framebuffer)? else {
        return Ok(0);
    };

    Ok(framebuffer.height as _)
}

pub async fn get_framebuffer_bpl(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPL({:#x})", framebuffer.0);

    let Some(framebuffer) = read_framebuffer_or_null(context, framebuffer)? else {
        return Ok(0);
    };

    Ok(framebuffer.bpl as _)
}

pub async fn get_framebuffer_bpp(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPP({:#x})", framebuffer.0);

    // Fall back to the default depth (not 0) so a caller deriving a pixel stride
    // from bpp on a null framebuffer doesn't divide by zero.
    let Some(framebuffer) = read_framebuffer_or_null(context, framebuffer)? else {
        return Ok(FRAMEBUFFER_DEPTH as _);
    };

    Ok(framebuffer.bpp as _)
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use wie_backend::canvas::VecImageBuffer;

    use crate::{MethodImpl, context::test::TestContext};

    use super::*;

    #[futures_test::test]
    async fn context_values_are_written_to_the_output_pointer() -> Result<()> {
        let mut context = TestContext::new();
        let ptr_context = context.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
        let input = context.alloc_raw(16)?;
        let output = context.alloc_raw(20)?;
        init_context(&mut context, ptr_context).await?;

        let set = set_context.into_body();
        let get = get_context.into_body();
        for (op, value) in [
            (1, 0x12345678),
            (2, 0x87654321),
            (3, 0xff00ff),
            (4, 128),
            (5, 0x1001),
            (6, 42),
            (7, 8),
            (8, 1),
        ] {
            set.call(&mut context, Box::new([ptr_context, op, value])).await?;
            write_generic(&mut context, output, [0xccccccccu32; 5])?;
            get.call(&mut context, Box::new([ptr_context, op, output])).await?;
            assert_eq!(
                read_generic::<[u32; 5], _>(&context, output)?,
                [value, 0xcccccccc, 0xcccccccc, 0xcccccccc, 0xcccccccc]
            );
        }

        write_generic(&mut context, input, [-5i32, -8, 176, 220])?;
        set.call(&mut context, Box::new([ptr_context, 0, input])).await?;
        write_generic(&mut context, output, [999i32; 5])?;
        get.call(&mut context, Box::new([ptr_context, 0, output])).await?;
        assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-5, -8, 176, 220, 999]);

        write_generic(&mut context, input, [-12i32, 34])?;
        set.call(&mut context, Box::new([ptr_context, 10, input])).await?;
        write_generic(&mut context, output, [999i32; 5])?;
        get.call(&mut context, Box::new([ptr_context, 10, output])).await?;
        assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-12, 34, 999, 999, 999]);

        get.call(&mut context, Box::new([ptr_context, 0xff, output])).await?;
        assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-12, 34, 999, 999, 999]);
        Ok(())
    }

    #[derive(Default)]
    struct Spans(Vec<(i32, i32, usize, u8)>);

    impl ImageBuffer for Spans {
        fn put_pixel(&mut self, _: i32, _: i32, _: Color) {
            unreachable!()
        }
        fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]) {
            assert_eq!(width as usize, colors.len());
            self.0.push((x, y, colors.len(), colors[0].r));
        }
        fn xor_pixel(&mut self, _: i32, _: i32, _: Color) {
            unreachable!()
        }
    }

    // native changes go to the Java image; Java changes go into native, so a later native redraw in
    // the same colour is a change again
    #[test]
    fn screen_sync_is_two_way() -> Result<()> {
        let mut context = TestContext::new();
        let framebuffer = FrameBuffer::new(&mut context, 4, 2, 16)?;
        let handle = context.alloc(size_of::<WIPICFramebuffer>() as _)?;
        let data = context.data_ptr(handle)?;
        write_generic(&mut context, data, framebuffer.0)?;
        let mut sync = ScreenFramebufferSync::default();
        let white = Color {
            a: 0xff,
            r: 0xff,
            g: 0xff,
            b: 0xff,
        };
        let red = Rgb565Pixel::from_color(Color {
            a: 0xff,
            r: 0xff,
            g: 0,
            b: 0,
        });
        let mut java = VecImageBuffer::<ArgbPixel>::new(4, 2);
        let native = |context: &mut TestContext| -> Result<Vec<u16>> {
            let raw = framebuffer.image(context)?.raw().into_owned();
            Ok(bytemuck::pod_collect_to_vec(&raw))
        };

        let mut spans = Spans::default();
        assert!(sync.compose_framebuffer(&mut context, handle, &java, &mut spans)?);
        assert_eq!(spans.0, [(0, 0, 4, 0), (0, 1, 4, 0)]);

        // the returned flag is what a Clet paint presents on: nothing native changed, nothing drawn
        let mut spans = Spans::default();
        assert!(!sync.compose_framebuffer(&mut context, handle, &java, &mut spans)?);
        assert!(spans.0.is_empty());

        framebuffer.write(&mut context, bytemuck::cast_slice(&[0u16, red, red, 0, 0, 0, 0, red]))?;
        assert!(sync.compose_framebuffer(&mut context, handle, &java, &mut spans)?);
        assert_eq!(spans.0, [(1, 0, 2, 0xff), (3, 1, 1, 0xff)]);

        // Java clears one pixel white: it reaches native, and nothing goes back to Java
        let mut spans = Spans::default();
        java.put_pixel(0, 1, white);
        assert!(!sync.compose_framebuffer(&mut context, handle, &java, &mut spans)?);
        assert!(spans.0.is_empty());
        assert_eq!(native(&mut context)?[4], 0xffff);

        // native redraws it black — the value it had before Java's clear — and that is a change
        framebuffer.write(&mut context, bytemuck::cast_slice(&[0u16, red, red, 0, 0, 0, 0, red]))?;
        sync.compose_framebuffer(&mut context, handle, &java, &mut spans)?;
        assert_eq!(spans.0, [(0, 1, 1, 0)]);
        Ok(())
    }
}
