use alloc::boxed::Box;

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
        if let Some(window) = web_sys::window() {
            let _ = window.navigator().vibrate_with_duration(duration_ms as u32);
        }
    }
}
