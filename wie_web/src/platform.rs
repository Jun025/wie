use alloc::boxed::Box;

use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{AudioContext, GainNode};

use wie_backend::{AudioSink, DatabaseRepository, Filesystem, Instant, Platform, Screen};

use crate::audio::WebAudioSink;
use crate::database::WebDatabaseRepository;
use crate::filesystem::WebFilesystem;
use crate::screen::WebScreen;

/// Browser implementation of the host `Platform` abstraction.
///
/// Everything the emulator core needs from the outside world is satisfied here
/// without OS threads, sockets, or a real filesystem — keeping the single
/// cooperative `tick()` model the core was designed for on wasm.
pub struct WebPlatform {
    screen: WebScreen,
    filesystem: WebFilesystem,
    database_repository: WebDatabaseRepository,
    audio_ctx: Option<AudioContext>,
    // JS-owned master gain node (output = gain → destination). Audio is routed
    // through it so the UI volume slider is the single source of truth.
    gain: Option<GainNode>,
}

// Single-threaded browser runtime; the only non-Send field is the JS audio
// handle, which never crosses a thread boundary.
unsafe impl Send for WebPlatform {}
unsafe impl Sync for WebPlatform {}

impl WebPlatform {
    pub fn new(
        screen: WebScreen,
        filesystem: WebFilesystem,
        database_repository: WebDatabaseRepository,
        audio_ctx: Option<AudioContext>,
        gain: Option<GainNode>,
    ) -> Self {
        Self {
            screen,
            filesystem,
            database_repository,
            audio_ctx,
            gain,
        }
    }
}

impl Platform for WebPlatform {
    fn screen(&self) -> &dyn Screen {
        &self.screen
    }

    fn now(&self) -> Instant {
        Instant::from_epoch_millis(js_sys::Date::now() as u64)
    }

    fn database_repository(&self) -> &dyn DatabaseRepository {
        &self.database_repository
    }

    fn filesystem(&self) -> &dyn Filesystem {
        &self.filesystem
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(WebAudioSink::new(self.audio_ctx.clone(), self.gain.clone()))
    }

    fn write_stdout(&self, buf: &[u8]) {
        web_sys::console::log_1(&String::from_utf8_lossy(buf).as_ref().into());
    }

    fn write_stderr(&self, buf: &[u8]) {
        web_sys::console::warn_1(&String::from_utf8_lossy(buf).as_ref().into());
    }

    fn exit(&self) {
        web_sys::console::log_1(&"[wie] emulator requested exit".into());
    }

    fn vibrate(&self, duration_ms: u64, _intensity: u8) {
        // `navigator.vibrate` is ABSENT on iOS (every browser is WebKit) and in
        // many desktop / insecure contexts. The web-sys `vibrate_with_duration`
        // binding is not a catching binding, so on those hosts it throws
        // `TypeError: navigator.vibrate is not a function`, which propagates out
        // of `tick()` and aborts the game (the reported crash on 영웅서기4).
        //
        // Look the method up reflectively and invoke it through
        // `Function::call1`, which returns a `Result` and swallows any JS
        // exception. Vibration is best-effort hardware feedback; its absence (or
        // a throwing implementation) is normal and must be a silent no-op that
        // never interrupts emulation — including under rapid repeated calls.
        let Some(window) = web_sys::window() else { return };
        let navigator = window.navigator();
        let Ok(vibrate) = Reflect::get(navigator.as_ref(), &JsValue::from_str("vibrate")) else {
            return;
        };
        if let Some(func) = vibrate.dyn_ref::<Function>() {
            let _ = func.call1(navigator.as_ref(), &JsValue::from_f64(duration_ms as f64));
        }
    }
}
