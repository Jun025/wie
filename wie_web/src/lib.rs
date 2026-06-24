//! Browser entry point for the wie emulator.
//!
//! Game bytes arrive here as a `Uint8Array` that the user picked locally; they
//! are injected straight into the emulator core in wasm memory and never touch
//! the network. There is no `fetch`, `XMLHttpRequest`, `WebSocket`, or upload of
//! any kind in this crate — the only data leaving wasm is the rendered frame
//! (to the canvas) and audio samples (to WebAudio).
//!
//! The crate is compiled only for `wasm32`; on every other target it is empty so
//! that native workspace jobs keep building.
#![cfg(target_arch = "wasm32")]

extern crate alloc;

mod audio;
mod database;
mod filesystem;
mod platform;
mod screen;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, GainNode, HtmlCanvasElement};

use wie_backend::{Emulator, Event, KeyCode, Options, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;

use crate::database::{DbStore, WebDatabaseRepository};
use crate::filesystem::{FsStore, WebFilesystem};
use crate::platform::WebPlatform;
use crate::screen::{RedrawFlag, WebScreen};

const SEP: char = '\u{1}';

/// Installs a panic hook that surfaces Rust panics in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// A running emulator instance, owned by JS for the lifetime of one loaded app.
#[wasm_bindgen]
pub struct WieEmulator {
    inner: Box<dyn Emulator>,
    fs_store: FsStore,
    db_store: DbStore,
    redraw: RedrawFlag,
}

#[wasm_bindgen]
impl WieEmulator {
    /// Construct an emulator from an uploaded file.
    ///
    /// * `filename` — the original name (its extension picks the loader).
    /// * `data` — the raw file bytes (kept entirely in wasm memory).
    /// * `canvas` — the `<canvas>` the framebuffer is blitted onto.
    /// * `audio_ctx` — an already-resumed `AudioContext`, or `null` for silence.
    /// * `width` / `height` — emulator screen size (240×320 is the usual default).
    #[wasm_bindgen(constructor)]
    pub fn new(
        filename: &str,
        data: Vec<u8>,
        canvas: &HtmlCanvasElement,
        audio_ctx: Option<AudioContext>,
        gain: Option<GainNode>,
        width: u32,
        height: u32,
    ) -> Result<WieEmulator, JsValue> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| JsValue::from_str("failed to get 2d context"))?
            .ok_or_else(|| JsValue::from_str("canvas has no 2d context"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("unexpected context type"))?;
        canvas.set_width(width);
        canvas.set_height(height);

        // Offscreen back buffer for double-buffered presentation (see WebScreen).
        let document = canvas.owner_document().ok_or_else(|| JsValue::from_str("canvas has no owner document"))?;
        let back_canvas = document
            .create_element("canvas")
            .map_err(|_| JsValue::from_str("failed to create back buffer"))?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| JsValue::from_str("back buffer is not a canvas"))?;
        back_canvas.set_width(width);
        back_canvas.set_height(height);
        let back_ctx = back_canvas
            .get_context("2d")
            .map_err(|_| JsValue::from_str("failed to get back 2d context"))?
            .ok_or_else(|| JsValue::from_str("back canvas has no 2d context"))?
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .map_err(|_| JsValue::from_str("unexpected back context type"))?;

        let fs_store: FsStore = Arc::new(Mutex::new(Default::default()));
        let db_store: DbStore = Arc::new(Mutex::new(Default::default()));
        // Start "true" so the very first composed frame is shown even if a title
        // somehow paints before its first request_redraw.
        let redraw: RedrawFlag = Arc::new(AtomicBool::new(true));

        let platform = Box::new(WebPlatform::new(
            WebScreen::new(ctx, back_canvas, back_ctx, width, height, redraw.clone()),
            WebFilesystem::new(fs_store.clone()),
            WebDatabaseRepository::new(db_store.clone()),
            audio_ctx,
            gain,
        ));

        let options = Options {
            enable_gdbserver: false,
            profile: None,
        };

        let inner = build_emulator(platform, filename, data, options).map_err(|e| JsValue::from_str(&e))?;

        Ok(WieEmulator {
            inner,
            fs_store,
            db_store,
            redraw,
        })
    }

    /// Advance the emulator one tick. Call this from `requestAnimationFrame`.
    ///
    /// We deliver `Event::Redraw` (which drives the actual blit) ONLY when the
    /// core asked for it via `Screen::request_redraw` — i.e. after it finished a
    /// frame. Forcing a redraw every animation frame could blit a half-composed
    /// framebuffer and made some titles flicker.
    pub fn tick(&mut self) -> Result<(), JsValue> {
        self.inner.tick().map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        if self.redraw.swap(false, Ordering::AcqRel) {
            self.inner.handle_event(Event::Redraw);
        }
        Ok(())
    }

    pub fn key_down(&mut self, code: &str) {
        if let Some(key) = parse_key(code) {
            self.inner.handle_event(Event::Keydown(key));
        }
    }

    pub fn key_up(&mut self, code: &str) {
        if let Some(key) = parse_key(code) {
            self.inner.handle_event(Event::Keyup(key));
        }
    }

    pub fn key_repeat(&mut self, code: &str) {
        if let Some(key) = parse_key(code) {
            self.inner.handle_event(Event::Keyrepeat(key));
        }
    }

    /// Snapshot the in-memory save filesystem as a plain JS object
    /// (`{ "aidpath": Uint8Array }`) for the JS layer to persist in
    /// IndexedDB. Game files are never in this store.
    pub fn export_fs(&self) -> Result<Object, JsValue> {
        let obj = Object::new();
        for ((aid, path), data) in self.fs_store.lock().unwrap().iter() {
            let key = format!("{aid}{SEP}{path}");
            let value = Uint8Array::from(data.as_slice());
            Reflect::set(&obj, &JsValue::from_str(&key), &value)?;
        }
        Ok(obj)
    }

    /// Restore a previously exported save snapshot. Replaces the current store.
    pub fn import_fs(&self, snapshot: &Object) -> Result<(), JsValue> {
        let mut store = self.fs_store.lock().unwrap();
        store.clear();
        let keys = Object::keys(snapshot);
        for key in keys.iter() {
            let key_str = key.as_string().unwrap_or_default();
            let Some((aid, path)) = key_str.split_once(SEP) else {
                continue;
            };
            let value = Reflect::get(snapshot, &key)?;
            let bytes = Uint8Array::new(&value).to_vec();
            store.insert((aid.to_owned(), path.to_owned()), bytes);
        }
        Ok(())
    }

    /// True once any save data exists, so JS knows whether to persist.
    pub fn has_saves(&self) -> bool {
        !self.fs_store.lock().unwrap().is_empty() || !self.db_store.lock().unwrap().is_empty()
    }

    /// Export ALL save state — both the RMS database records and the save
    /// filesystem — as one opaque binary blob (`WIESAV01`). Unlike `export_fs`,
    /// this also captures RMS records, so titles that persist via the record
    /// store keep their progress. The blob is what the JS layer stores in
    /// IndexedDB and (base64-encoded) syncs to the server as opaque save bytes —
    /// the server never learns which game it belongs to.
    pub fn export_saves(&self) -> Vec<u8> {
        let db = self.db_store.lock().unwrap();
        let fs = self.fs_store.lock().unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(SAVE_MAGIC);

        let dbs: Vec<_> = db.iter().filter(|(_, recs)| !recs.is_empty()).collect();
        put_u32(&mut out, dbs.len() as u32);
        for ((app_id, name), recs) in dbs {
            put_str(&mut out, app_id);
            put_str(&mut out, name);
            put_u32(&mut out, recs.len() as u32);
            for (id, data) in recs {
                put_u32(&mut out, *id);
                put_bytes(&mut out, data);
            }
        }

        put_u32(&mut out, fs.len() as u32);
        for ((aid, path), data) in fs.iter() {
            put_str(&mut out, aid);
            put_str(&mut out, path);
            put_bytes(&mut out, data);
        }
        out
    }

    /// Restore an opaque blob produced by `export_saves`, replacing the current
    /// save state. A malformed/empty blob is ignored (returns `false`).
    pub fn import_saves(&self, blob: &[u8]) -> bool {
        let Some((dbs, files)) = parse_saves(blob) else {
            return false;
        };
        *self.db_store.lock().unwrap() = dbs;
        *self.fs_store.lock().unwrap() = files;
        true
    }
}

const SAVE_MAGIC: &[u8; 8] = b"WIESAV01";

fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}
fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_u32(out, b.len() as u32);
    out.extend_from_slice(b);
}
fn put_str(out: &mut Vec<u8>, s: &str) {
    put_bytes(out, s.as_bytes());
}

#[allow(clippy::type_complexity)]
fn parse_saves(
    blob: &[u8],
) -> Option<(
    std::collections::BTreeMap<(String, String), std::collections::BTreeMap<wie_backend::RecordId, Vec<u8>>>,
    std::collections::BTreeMap<(String, String), Vec<u8>>,
)> {
    use std::collections::BTreeMap;
    if blob.len() < 8 || &blob[..8] != SAVE_MAGIC {
        return None;
    }
    let mut pos = 8usize;
    let take_u32 = |buf: &[u8], pos: &mut usize| -> Option<u32> {
        let end = pos.checked_add(4)?;
        if end > buf.len() {
            return None;
        }
        let v = u32::from_le_bytes(buf[*pos..end].try_into().ok()?);
        *pos = end;
        Some(v)
    };
    let take_bytes = |buf: &[u8], pos: &mut usize| -> Option<Vec<u8>> {
        let len = take_u32(buf, pos)? as usize;
        let end = pos.checked_add(len)?;
        if end > buf.len() {
            return None;
        }
        let v = buf[*pos..end].to_vec();
        *pos = end;
        Some(v)
    };
    let take_str = |buf: &[u8], pos: &mut usize| -> Option<String> { String::from_utf8(take_bytes(buf, pos)?).ok() };

    let mut dbs = BTreeMap::new();
    let db_count = take_u32(blob, &mut pos)?;
    for _ in 0..db_count {
        let app_id = take_str(blob, &mut pos)?;
        let name = take_str(blob, &mut pos)?;
        let rec_count = take_u32(blob, &mut pos)?;
        let mut recs = BTreeMap::new();
        for _ in 0..rec_count {
            let id = take_u32(blob, &mut pos)?;
            let data = take_bytes(blob, &mut pos)?;
            recs.insert(id, data);
        }
        dbs.insert((app_id, name), recs);
    }
    let mut files = BTreeMap::new();
    let file_count = take_u32(blob, &mut pos)?;
    for _ in 0..file_count {
        let aid = take_str(blob, &mut pos)?;
        let path = take_str(blob, &mut pos)?;
        let data = take_bytes(blob, &mut pos)?;
        files.insert((aid, path), data);
    }
    Some((dbs, files))
}

fn build_emulator(platform: Box<WebPlatform>, filename: &str, data: Vec<u8>, options: Options) -> Result<Box<dyn Emulator>, String> {
    let name = &filename[filename.rfind('/').map(|i| i + 1).unwrap_or(0)..];

    if filename.ends_with("zip") {
        let files = extract_zip(&data).map_err(|e| format!("{e:?}"))?;
        if KtfEmulator::loadable_archive(&files) {
            Ok(Box::new(
                KtfEmulator::from_archive(platform, files, options).map_err(|e| format!("{e:?}"))?,
            ))
        } else if LgtEmulator::loadable_archive(&files) {
            Ok(Box::new(
                LgtEmulator::from_archive(platform, files, options).map_err(|e| format!("{e:?}"))?,
            ))
        } else if SktEmulator::loadable_archive(&files) {
            Ok(Box::new(SktEmulator::from_archive(platform, files).map_err(|e| format!("{e:?}"))?))
        } else {
            Err("Unknown archive format".to_owned())
        }
    } else if filename.ends_with("jar") {
        let name_without_ext = name.trim_end_matches(".jar");
        if KtfEmulator::loadable_jar(&data) {
            Ok(Box::new(
                KtfEmulator::from_jar(platform, name, data, name_without_ext, name_without_ext, None, options).map_err(|e| format!("{e:?}"))?,
            ))
        } else if LgtEmulator::loadable_jar(&data) {
            Ok(Box::new(
                LgtEmulator::from_jar(platform, name, data, name_without_ext, name_without_ext, None, options).map_err(|e| format!("{e:?}"))?,
            ))
        } else if SktEmulator::loadable_jar(&data) {
            Ok(Box::new(
                SktEmulator::from_jar(platform, name, data, name_without_ext, None).map_err(|e| format!("{e:?}"))?,
            ))
        } else {
            Ok(Box::new(J2MEEmulator::from_jar(platform, name, data).map_err(|e| format!("{e:?}"))?))
        }
    } else if filename.ends_with("jad") {
        Err("A .jad needs its companion .jar — please upload the .jar file instead.".to_owned())
    } else {
        Err("Unknown file format (expected .jar or .zip)".to_owned())
    }
}

/// Map a frontend key name to a core [`KeyCode`]. Names match the `KeyCode`
/// variants so the JS remapping UI can use the same vocabulary.
fn parse_key(code: &str) -> Option<KeyCode> {
    Some(match code {
        "UP" => KeyCode::UP,
        "DOWN" => KeyCode::DOWN,
        "LEFT" => KeyCode::LEFT,
        "RIGHT" => KeyCode::RIGHT,
        "OK" => KeyCode::OK,
        "LEFT_SOFT_KEY" => KeyCode::LEFT_SOFT_KEY,
        "RIGHT_SOFT_KEY" => KeyCode::RIGHT_SOFT_KEY,
        "CLEAR" => KeyCode::CLEAR,
        "CALL" => KeyCode::CALL,
        "HANGUP" => KeyCode::HANGUP,
        "VOLUME_UP" => KeyCode::VOLUME_UP,
        "VOLUME_DOWN" => KeyCode::VOLUME_DOWN,
        "NUM0" => KeyCode::NUM0,
        "NUM1" => KeyCode::NUM1,
        "NUM2" => KeyCode::NUM2,
        "NUM3" => KeyCode::NUM3,
        "NUM4" => KeyCode::NUM4,
        "NUM5" => KeyCode::NUM5,
        "NUM6" => KeyCode::NUM6,
        "NUM7" => KeyCode::NUM7,
        "NUM8" => KeyCode::NUM8,
        "NUM9" => KeyCode::NUM9,
        "HASH" => KeyCode::HASH,
        "STAR" => KeyCode::STAR,
        _ => return None,
    })
}
