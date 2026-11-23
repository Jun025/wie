//! Browser entry point for the wie emulator.
//!
//! Game bytes arrive here as a `Uint8Array` that the user picked locally; they
//! are injected straight into the emulator core in wasm memory and never touch
//! the network. There is no `XMLHttpRequest`, `WebSocket`, or upload of any kind
//! in this crate, and exactly one `fetch`: a GET of the soundfont URL the host
//! passes to the constructor (`audio.rs` — no URL, no request), which carries no
//! game data. The only data leaving wasm is the rendered frame (to the canvas)
//! and audio samples (to WebAudio).
//!
//! The crate is compiled only for `wasm32`; on every other target it is empty so
//! that native workspace jobs keep building.
//!
//! # Host wakes (opt-in, 2026-11-11 · docs/report/0556)
//!
//! A guest that sleeps on a beat (`Thread.sleep(60)`, a WIPI timer) is woken on time by `tick`
//! waiting for the wake inside its budget — reading the clock in a loop. How much of such a game's
//! cost that wait is depends on where its wakes fall in the frame: simulated at 60Hz, about two
//! thirds of the engine's CPU in 4 shapes of 6 and about a tenth in the other 2; at 120Hz 7-43%
//! (`wie-backend`'s `tick_occupancy`). A host that calls `set_wake_margin(ms)` once takes that wait
//! over: from then on a tick ends on a wake further off than `ms` — and than the 4ms a nested
//! browser timer cannot fire inside — and the host comes back for it. What the guest gets is the
//! same in number either way — as many wakes, as many game frames, as many timer fires — and only
//! the waiting moves to the host. Where inside a host frame each of them falls is not the same: a
//! 1ms timer another thread arms while the event loop waits for a key fires as often, and later on
//! average (beside a 10ms beat 3.8 -> 6.2 guest ms at 1x, 4.0 -> 8.7 at 2x; docs/report/0556 §9).
//! A host that never calls it runs exactly as before.
//!
//! That holds because a tick hands over only a wake nothing polls before. While it waits for a
//! wake itself, a tick also runs every poll beside it a millisecond apart: a 1ms WIPI timer, one
//! that already fired its share of the frame, `callSerially`, a thread in a `sleep(1)` loop. With
//! such work due before the wake the tick goes on waiting itself, margin or not — a host that came
//! back only for the wake ran those 8-50% less often (docs/report/0556 §9). So a game with a timer
//! beside a thread that sleeps on a beat gains little or nothing from the margin, and loses nothing.
//!
//! - **After every `tick` and `wake_tick`**: `clearTimeout` the pending wake timer, read
//!   `wake_in_ms()`, and if it is a number, `setTimeout(() => emu.wake_tick(), it)`. Arm it from
//!   there and not from a chain of timers of your own: WebKit fired a timer armed from a frame
//!   callback at most 3ms late, a long chain of nested timers up to 7ms (measured headless
//!   2026-11-11), and `ms` must cover that lateness or the wake is late by the difference.
//! - **`wake_in_ms()` is a number only for a wake the last `tick` would itself have waited for** —
//!   a sleep or a paced timer due inside that tick's budget. `undefined` = no timer, and the next
//!   `tick` serves the guest as it does on a host without a margin: it wants CPU; or its wake falls
//!   after that budget; or what it waits for is polled, not paced — a key, `callSerially`, a 1ms
//!   WIPI timer, a timer that already fired its share this frame. Coming back for those ran such
//!   timers 2-17 times as fast as the host's frames allow (docs/report/0556 §8). Or the tick waited
//!   for the wake itself, so did not end on it: a poll has work before that wake (above); or it
//!   was nearer than a tick waits — `ms`, and never less than 4ms.
//! - **`wake_tick` is not a frame but the rest of the last `tick`**: it ends where that tick's
//!   budget ended — never past the next vsync, whatever the display — is the same host frame to a
//!   WIPI timer, and leaves the frame-interval pacing (`FramePacer`) alone. Called after that
//!   budget is spent it does nothing. Keep calling `tick` from `requestAnimationFrame`.
//! - **A repaint still reaches the game once per host frame**, where that frame's tick is over:
//!   after the `tick` or `wake_tick` that leaves no wake to come back for, or — the timer having
//!   lost to the next frame — before the next `tick`. The host has nothing to do for it. Handed on
//!   after every one of them, a game that repaints and then sleeps 20ms was painted 1971 times in
//!   20s at 2x where a host without a margin paints it 1154 times (docs/report/0556 §10).
//! - **`ms` = 3** is the measured value for WebKit; a browser whose timers run later wants more,
//!   at the cost of up to `ms` of waiting per wake. `undefined` turns it off again. A timer later
//!   than `ms` is a slower game, not only a late wake: simulated `sleep(20)` at 2x on 60Hz ran 719
//!   game frames in 10s without a margin and with timers up to 3ms late, and 644 / 611 / 602
//!   with timers up to 8 / 12 / 20ms late — where `achieved_speed()` still read
//!   1.85-1.93 (a device whose timers alone run late, as in a low-power mode).
//! - **Paused or hidden**: cancel the timer along with the frame loop; resume both together.
//! - **`set_speed`**: the delay is in wall time and changes with the speed — re-read `wake_in_ms()`
//!   and re-arm.
#![cfg(target_arch = "wasm32")]

extern crate alloc;

mod audio;
mod database;
mod filesystem;
mod platform;
mod screen;

use alloc::sync::Arc;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use js_sys::{Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use web_sys::{AudioContext, GainNode, HtmlCanvasElement};

use wie_backend::{AchievedSpeed, Emulator, Event, FramePacer, Instant, KeyCode, Options, SpeedClock, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::{LgtEmulator, detect_compile_model};
use wie_skt::SktEmulator;

use crate::database::{DbStore, WebDatabaseRepository};
use crate::filesystem::{FsStore, WebFilesystem};
use crate::platform::{HostWake, WebPlatform};
use crate::screen::{FrameCount, RedrawFlag, WebScreen};

const SEP: char = '\u{1}';

/// Installs a panic hook that surfaces Rust panics in the browser console.
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Which game a file is, without booting it: `{ platform, model, contentKey }`, or `undefined` when no loader takes it.
// One line, as on `achieved_speed`: this comment is copied into wie_web.js. The definition is
// `wie_identify::identify`; `data` is copied in and read only — the caller's buffer is never written.
#[wasm_bindgen]
pub fn identify(filename: &str, data: &[u8]) -> Result<JsValue, JsValue> {
    let Some(identity) = wie_identify::identify(filename, data) else {
        return Ok(JsValue::UNDEFINED);
    };
    let text = |value: Option<&str>| value.map_or(JsValue::NULL, JsValue::from_str);
    let object = Object::new();
    Reflect::set(&object, &"platform".into(), &identity.platform.into())?;
    Reflect::set(&object, &"model".into(), &text(identity.model))?;
    Reflect::set(&object, &"contentKey".into(), &text(identity.content_key.as_deref()))?;
    Ok(object.into())
}

/// A running emulator instance, owned by JS for the lifetime of one loaded app.
#[wasm_bindgen]
pub struct WieEmulator {
    inner: Box<dyn Emulator>,
    fs_store: FsStore,
    db_store: DbStore,
    redraw: RedrawFlag,
    // Frames `WebScreen` has presented (see `frames_painted`).
    frames: FrameCount,
    // Sticky clean-exit flag, set by `WebPlatform::exit` when the core requests
    // a normal shutdown (the "[wie] emulator requested exit" path).
    exited: Arc<AtomicBool>,
    platform_kind: &'static str,
    // Turns the interval between `tick` calls (one per `requestAnimationFrame`) into the tick
    // budget, so a 120Hz display does not get a 60Hz frame's worth of emulation per frame.
    pacer: FramePacer,
    // For LGT titles, the statically-detected compile model ("clet"/"aot-java");
    // `None` for non-LGT platforms. Set once at construction, never changes.
    lgt_compile_model: Option<&'static str>,
    // Play speed: the guest clock `WebPlatform::now` reads (see `set_speed`).
    clock: Arc<SpeedClock>,
    // What `speed` actually comes to on this device (see `achieved_speed`).
    achieved: AchievedSpeed,
    // Host wakes (crate doc): the margin and the wake-tick flag `WebPlatform` reports to the engine;
    // the wall ms at which the last frame tick's budget ends; and the wake the last tick ended on
    // that the host is to come back for (`Pacing::wake`).
    host_wake: Arc<HostWake>,
    frame_end: f64,
    wake: Option<Instant>,
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
    /// * `soundfont_url` — optional sf2/sf3 the audio sink fetches in the background from boot.
    ///   Every MIDI play of the session renders through it, the first included (a play waits for it,
    ///   bounded); if it fails or is too late, the whole session is FM. Omitted (or a build without
    ///   the soundfont prelude) = the built-in FM synth only, the pre-soundfont behaviour.
    // Positional because the JS constructor shape is the pinned engine contract
    // (docs/contracts/featurephone-engine-contract.json `constructorShape`).
    #[allow(clippy::too_many_arguments)]
    #[wasm_bindgen(constructor)]
    pub fn new(
        filename: &str,
        data: Vec<u8>,
        canvas: &HtmlCanvasElement,
        audio_ctx: Option<AudioContext>,
        gain: Option<GainNode>,
        width: u32,
        height: u32,
        soundfont_url: Option<String>,
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
        let frames = FrameCount::default();
        let exited: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        let clock = Arc::new(SpeedClock::new());
        let host_wake = Arc::new(HostWake {
            margin_ms: AtomicU64::new(u64::MAX),
            in_wake_tick: AtomicBool::new(false),
        });

        let platform = Box::new(
            WebPlatform::new(
                WebScreen::new(ctx, back_canvas, back_ctx, width, height, redraw.clone(), frames.clone()),
                WebFilesystem::new(fs_store.clone()),
                WebDatabaseRepository::new(db_store.clone()),
                audio_ctx,
                gain,
                soundfont_url,
                exited.clone(),
                clock.clone(),
                host_wake.clone(),
            )
            .map_err(|e| JsValue::from_str(&format!("{e:?}")))?,
        );

        let options = Options {
            enable_gdbserver: false,
            aot: None, // interpreter only: adopting upstream's ARM AOT is a separate, measured round
            profile: None,
        };

        let (inner, platform_kind, lgt_compile_model) = build_emulator(platform, filename, data, options).map_err(|e| JsValue::from_str(&e))?;

        Ok(WieEmulator {
            inner,
            fs_store,
            db_store,
            redraw,
            frames,
            exited,
            platform_kind,
            lgt_compile_model,
            pacer: FramePacer::new(),
            clock,
            achieved: AchievedSpeed::default(),
            host_wake,
            frame_end: 0.0,
            wake: None,
        })
    }

    /// The detected emulator backend ("KTF" / "LGT" / "SKT" / "J2ME"). A runtime
    /// label for error diagnostics only — NOT game-file identity, never sent out.
    pub fn platform_kind(&self) -> String {
        self.platform_kind.to_owned()
    }

    /// For an LGT title, the statically-detected compile model — `"clet"`
    /// (WIPI-C) or `"aot-java"` (AOT-compiled Java). Returns `null` for every
    /// non-LGT platform (KTF / SKT / J2ME), where the notion does not apply.
    ///
    /// Detected once at construction from the app's own import thunks (read-only,
    /// no execution) and never changes for this instance. Because it is set in
    /// the constructor it is valid immediately — a shell can call it right after
    /// `new WieEmulator(...)` succeeds, before the first `tick()`.
    ///
    /// **This getter reports how the app was compiled. It does not report whether
    /// wie can render it, and it never could** — an earlier revision of this doc
    /// said `"aot-java"` "boots but does not yet render (the §7 wall)" and
    /// recommended blocking the whole model at upload. That recommendation rested
    /// on a 2026-07 sweep whose premise upstream `cc652b1d` (2026-08-04, "Implement
    /// LGT Java AOT runtime") superseded. Re-measured 2026-09-21 over the local
    /// corpus's 24 AOT-Java files (18 unique): **3 render** — `메이플스토리2007`
    /// reaches its title screen and in-game intro, `현영맞고2006` its title screen,
    /// `놈3` its splash — while the other 15 die at *boot*, before anything a
    /// render path could reach. So the model is not a rendering predicate in
    /// either direction. A shell that wants a runnability denylist owns it
    /// per-title; see `docs/lgt-fork-notes.md` §7 and `docs/project-kb/02_status.md`.
    pub fn lgt_compile_model(&self) -> Option<String> {
        self.lgt_compile_model.map(str::to_owned)
    }

    /// Advance the emulator one tick. Call this from `requestAnimationFrame`.
    ///
    /// We deliver `Event::Redraw` (which drives the actual blit) ONLY when the
    /// core asked for it via `Screen::request_redraw` — i.e. after it finished a
    /// frame. Forcing a redraw every animation frame could blit a half-composed
    /// framebuffer and made some titles flicker.
    ///
    /// Once [`has_exited`](Self::has_exited) is true this is a safe no-op that
    /// returns `Ok(())` — the core is never advanced past its requested exit, so
    /// a render loop that keeps firing before JS observes the flag cannot crash.
    pub fn tick(&mut self) -> Result<(), JsValue> {
        self.advance(false)
    }

    /// Opt in to host wakes: a tick waits at most `margin_ms` for a guest's wake. See crate doc.
    pub fn set_wake_margin(&mut self, margin_ms: Option<u32>) {
        self.host_wake.margin_ms.store(margin_ms.map_or(u64::MAX, u64::from), Ordering::Relaxed);
    }

    /// Ms until `wake_tick` should run; `undefined` = no wake the last tick would have waited for.
    /// See crate doc.
    pub fn wake_in_ms(&self) -> Option<f64> {
        let margin = self.host_wake.margin_ms()?;
        let due = self.wake.filter(|_| !self.exited.load(Ordering::Acquire))?;
        Some(self.clock.wake_delay(js_sys::Date::now(), due, margin))
    }

    /// The rest of the last `tick`, for `wake_in_ms`'s timer. See crate doc.
    pub fn wake_tick(&mut self) -> Result<(), JsValue> {
        self.advance(true)
    }

    fn advance(&mut self, wake: bool) -> Result<(), JsValue> {
        if self.exited.load(Ordering::Acquire) {
            return Ok(());
        }
        // The pacer works in wall time; the executor measures its budget on the guest clock, which
        // runs `speed` times as fast, so the same wall-clock slice is `speed` times the guest budget.
        let begin = js_sys::Date::now();
        let budget = if wake {
            // Not a frame — the pacer would read the interval as a faster display — but the rest of
            // the last one's tick: the engine ends it where that tick's budget did
            // (`Platform::in_wake_tick`), so what that budget had left is all this can be.
            (self.frame_end - begin).max(0.0) as u64
        } else {
            let budget = self.pacer.begin(begin as u64);
            self.frame_end = begin + budget as f64;
            budget
        };
        // A redraw the frame before still holds: its tick ended on a wake this host never came back
        // for (a timer that lost to the next frame), so that frame is over without its wake tick.
        if !wake && self.wake.is_some() {
            self.hand_on_redraw();
        }
        self.host_wake.in_wake_tick.store(wake, Ordering::Relaxed);
        let result = self.inner.tick_for(self.clock.budget(budget));
        let end = js_sys::Date::now();
        if !wake {
            self.pacer.end(end as u64);
        }
        // Taken every tick, so the window is this tick; nothing else here reads the pacing counters.
        let pacing = self.inner.take_pacing();
        self.wake = pacing.wake();
        // The guest's next due work, on the wall clock.
        let due = pacing.due().map(|due| self.clock.wall_at(end, due));
        self.achieved.tick(begin, end, budget as f64, pacing.starved_ticks() > 0, due);
        result.map_err(|e| JsValue::from_str(&format!("{e:?}")))?;
        // Once per host frame, as the engine hands on a guest's own repaint (`System::request_redraw`):
        // not after a tick the host comes back for, or a game that asks before and after that wake
        // is painted twice in the frame.
        if self.wake.is_none() {
            self.hand_on_redraw();
        }
        Ok(())
    }

    fn hand_on_redraw(&mut self) {
        if self.redraw.swap(false, Ordering::AcqRel) {
            self.inner.handle_event(Event::Redraw);
        }
    }

    /// Set the play speed: the game runs `speed` times as fast as real time. Continuous in
    /// `[1.0, 2.0]` (not quantized); values outside are clamped, NaN and infinities are 1.0.
    /// Returns the speed applied. Every guest timer follows (sleeps, WIPI/MIDP timers, the clock a
    /// game reads); changing it mid-game never makes the guest clock jump or run backwards. Each
    /// tick still takes the same wall-clock slice of the frame, so where the CPU cannot keep up the
    /// game runs as fast as it can. Audio pitch and tempo are not changed.
    pub fn set_speed(&mut self, speed: f64) -> f64 {
        self.achieved.reset();
        self.clock.set_speed(js_sys::Date::now(), speed)
    }

    /// The current play speed (1.0 until `set_speed` is called).
    pub fn speed(&self) -> f64 {
        self.clock.speed()
    }

    /// The play speed actually achieved over the last 2s; at most `speed()`. See `AchievedSpeed`.
    // Kept to one line: wasm-bindgen copies this comment into wie_web.js, which has a byte budget
    // (contract `artifacts.maxBytes`). The definition and its floor are on `wie_backend::AchievedSpeed`.
    pub fn achieved_speed(&self) -> f64 {
        self.achieved.get(self.clock.speed())
    }

    /// Frames the game has shown so far (wraps at 2^32); its change over time is the game's frames/s.
    // One line, as above. `achieved_speed` bottoms out near 0.84 for a game short of CPU however
    // short; this is the «how short»: an LGT title read 15 -> 4 frames/s with that figure flat
    // (docs/report/0557).
    pub fn frames_painted(&self) -> u32 {
        self.frames.load(Ordering::Relaxed)
    }

    /// True once the emulator core requested a NORMAL shutdown (the
    /// "[wie] emulator requested exit" path, e.g. WIPI `MC_knlExit`). Sticky:
    /// once true it stays true for the lifetime of this instance.
    ///
    /// This is how JS distinguishes a clean exit (flag flips true, `tick` keeps
    /// resolving as a no-op) from a failure (`tick` throws). Save state remains
    /// readable after the flip — poll this after each `tick` and, when true,
    /// persist via `export_saves` before tearing the instance down.
    pub fn has_exited(&self) -> bool {
        self.exited.load(Ordering::Acquire)
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

// Returns the loaded emulator, the detected platform kind ("KTF"/"LGT"/"SKT"/
// "J2ME"), and — for LGT only — the statically-detected compile model
// ("clet"/"aot-java"), else `None`. The kind is a backend/runtime label (NOT
// game-file identity); it is surfaced only in on-screen error diagnostics,
// never sent to the server.
#[allow(clippy::type_complexity)]
fn build_emulator(
    platform: Box<WebPlatform>,
    filename: &str,
    data: Vec<u8>,
    options: Options,
) -> Result<(Box<dyn Emulator>, &'static str, Option<&'static str>), String> {
    let name = &filename[filename.rfind('/').map(|i| i + 1).unwrap_or(0)..];

    if filename.ends_with("zip") {
        let files = extract_zip(&data).map_err(|e| format!("{e:?}"))?;
        if KtfEmulator::loadable_archive(&files) {
            Ok((
                Box::new(KtfEmulator::from_archive(platform, files, options).map_err(|e| format!("{e:?}"))?),
                "KTF",
                None,
            ))
        } else if LgtEmulator::loadable_archive(&files) {
            // Detect the compile model before `files` is moved into the loader.
            let model = detect_compile_model(&files).map(|m| m.as_str());
            Ok((
                Box::new(LgtEmulator::from_archive(platform, files, options).map_err(|e| format!("{e:?}"))?),
                "LGT",
                model,
            ))
        } else if SktEmulator::loadable_archive(&files) {
            Ok((
                Box::new(SktEmulator::from_archive(platform, files).map_err(|e| format!("{e:?}"))?),
                "SKT",
                None,
            ))
        } else {
            Err("Unknown archive format".to_owned())
        }
    } else if filename.ends_with("jar") {
        let name_without_ext = name.trim_end_matches(".jar");
        if KtfEmulator::loadable_jar(&data) {
            Ok((
                Box::new(
                    KtfEmulator::from_jar(platform, name, data, name_without_ext, name_without_ext, None, options).map_err(|e| format!("{e:?}"))?,
                ),
                "KTF",
                None,
            ))
        } else if LgtEmulator::loadable_jar(&data) {
            // The jar is itself a zip; its extracted contents hold binary.mod.
            let model = extract_zip(&data).ok().and_then(|inner| detect_compile_model(&inner)).map(|m| m.as_str());
            Ok((
                Box::new(
                    LgtEmulator::from_jar(platform, name, data, name_without_ext, name_without_ext, None, options).map_err(|e| format!("{e:?}"))?,
                ),
                "LGT",
                model,
            ))
        } else if SktEmulator::loadable_jar(&data) {
            Ok((
                Box::new(SktEmulator::from_jar(platform, name, data, name_without_ext).map_err(|e| format!("{e:?}"))?),
                "SKT",
                None,
            ))
        } else {
            Ok((
                Box::new(J2MEEmulator::from_jar(platform, name, data).map_err(|e| format!("{e:?}"))?),
                "J2ME",
                None,
            ))
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
