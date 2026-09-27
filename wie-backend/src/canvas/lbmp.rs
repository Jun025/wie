use alloc::{boxed::Box, format, vec::Vec};

use bytemuck::{Pod, Zeroable, from_bytes, pod_collect_to_vec};

use wie_util::{Result, WieError};

use crate::canvas::{ArgbPixel, Image, Rgb332Pixel, Rgb565Pixel, VecImageBuffer};

// lcd bitmap file format for skvm

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LbmpHeader {
    descriptor: u32,
    r#type: u32,
    width: u32,
    height: u32,
    size: u32,
    mask: u32,
}

pub fn decode_lbmp(data: &[u8]) -> Result<Box<dyn Image>> {
    if data.len() < 24 {
        return Err(WieError::FatalError(format!("Truncated LBMP header: {} bytes", data.len())));
    }

    let header: &LbmpHeader = from_bytes(&data[0..24]);
    let data = &data[24..];

    if header.r#type == 3 {
        // unsupported grayscale
        return Err(WieError::Unimplemented(format!("Unsupported grayscale type {}", header.r#type)));
    }

    Ok(if header.r#type == 2 {
        decode_gray2(header, data)?
    } else if header.r#type == 8 {
        Box::new(VecImageBuffer::<Rgb332Pixel>::from_raw(header.width, header.height, data.to_vec()))
    } else if header.r#type == 16 {
        Box::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
            header.width,
            header.height,
            pod_collect_to_vec(data),
        ))
    } else {
        return Err(WieError::Unimplemented(format!("Unsupported type {}", header.r#type)));
    })
}

// Type 2: 2-bit gray in LCD page layout, measured on f6fe2adc8cce's 11 type-2 files (2026-09-27).
// Each plane is `size` = width × ceil(height / 8) bytes; byte `(y / 8) * width + x` holds a column
// of 8 pixels, bit `y % 8` (LSB = top). Plane 0 is the high bit, plane 1 the low bit, and 3 is black
// (a URL banner is set in both planes on white). With `mask` set a third plane
// follows whose set bits are transparent (the rounded corners of a key legend).
// Plane order is the measured part: plane 0 as high bit gives 21–32% lower total variation on all
// four photographic puzzle tiles than the reverse — the smoother reading of a photo is the right one.
fn decode_gray2(header: &LbmpHeader, data: &[u8]) -> Result<Box<dyn Image>> {
    let (width, height, size) = (header.width as usize, header.height as usize, header.size as usize);
    let planes = if header.mask != 0 { 3 } else { 2 };
    if size != width * height.div_ceil(8) || data.len() < size * planes {
        return Err(WieError::FatalError(format!(
            "Truncated LBMP type 2: {width}x{height} size {size}, {} bytes for {planes} planes",
            data.len()
        )));
    }

    let bit = |plane: usize, x: usize, y: usize| (data[plane * size + (y / 8) * width + x] >> (y % 8)) & 1;
    let pixels: Vec<u32> = (0..height)
        .flat_map(|y| (0..width).map(move |x| (x, y)))
        .map(|(x, y)| {
            let level = 255 - (bit(0, x, y) * 2 + bit(1, x, y)) as u32 * 85;
            let alpha = if planes == 3 && bit(2, x, y) == 1 { 0 } else { 0xff };
            (alpha << 24) | (level << 16) | (level << 8) | level
        })
        .collect();

    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(header.width, header.height, pixels)))
}
