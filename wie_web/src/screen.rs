use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};

use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use wie_backend::{Screen, canvas::Image};
use wie_util::Result;

/// Shared "the core wants the display updated" signal. The core calls
/// [`Screen::request_redraw`] only after it has finished composing a frame; the
/// host ([`crate::WieEmulator::tick`]) consumes the flag and delivers exactly
/// one `Event::Redraw`. Driving repaints off this flag — instead of forcing one
/// every animation frame — means we only ever blit COMPLETE frames, which fixes
/// the flicker some titles showed (a forced mid-compose blit could expose a
/// cleared-but-not-yet-drawn framebuffer).
pub type RedrawFlag = Arc<AtomicBool>;

/// Browser screen backed by a 2D `<canvas>` context.
///
/// Double-buffered: the core's RGBA framebuffer is written with `putImageData`
/// onto an OFFSCREEN back-buffer canvas, then composited onto the visible canvas
/// with a single `drawImage`. Writing `putImageData` straight to the on-screen
/// canvas could let the compositor present a half-updated surface when the page
/// is otherwise idle (the reported "flickers only while NOT scrolling" symptom);
/// `drawImage` from the back buffer is one atomic composited blit, so the
/// visible canvas only ever shows complete frames. Scaling is CSS-only
/// (`image-rendering: pixelated`).
pub struct WebScreen {
    ctx: CanvasRenderingContext2d,
    back_canvas: HtmlCanvasElement,
    back_ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
    redraw: RedrawFlag,
}

// wasm32 in the browser is single-threaded; the JS handle is never sent across
// threads. This mirrors the existing `wie_cli` audio-sink precedent.
unsafe impl Send for WebScreen {}
unsafe impl Sync for WebScreen {}

impl WebScreen {
    pub fn new(
        ctx: CanvasRenderingContext2d,
        back_canvas: HtmlCanvasElement,
        back_ctx: CanvasRenderingContext2d,
        width: u32,
        height: u32,
        redraw: RedrawFlag,
    ) -> Self {
        Self {
            ctx,
            back_canvas,
            back_ctx,
            width,
            height,
            redraw,
        }
    }
}

impl Screen for WebScreen {
    fn request_redraw(&self) -> Result<()> {
        // Signal the host loop; it will deliver one Event::Redraw on the next tick.
        self.redraw.store(true, Ordering::Release);
        Ok(())
    }

    fn paint(&self, image: &dyn Image) {
        let width = image.width();
        let height = image.height();
        if width == 0 || height == 0 {
            return;
        }

        let colors = image.colors();
        let mut rgba = alloc_rgba(colors.len());
        for color in &colors {
            // Force opaque alpha: the core treats the framebuffer as XRGB
            // (see wie_cli's softbuffer path). A literal 0 alpha would make the
            // canvas pixel transparent and the screen would render blank.
            rgba.push(color.r);
            rgba.push(color.g);
            rgba.push(color.b);
            rgba.push(0xff);
        }

        let image_data = match ImageData::new_with_u8_clamped_array_and_sh(Clamped(&rgba), width, height) {
            Ok(data) => data,
            Err(_) => return,
        };
        // 1) write the finished frame to the offscreen back buffer …
        if self.back_ctx.put_image_data(&image_data, 0.0, 0.0).is_err() {
            return;
        }
        // 2) … then present it in one atomic composited blit.
        let _ = self.ctx.draw_image_with_html_canvas_element(&self.back_canvas, 0.0, 0.0);
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

fn alloc_rgba(pixel_count: usize) -> Vec<u8> {
    Vec::with_capacity(pixel_count * 4)
}
