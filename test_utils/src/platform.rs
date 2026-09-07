use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};

use hashbrown::HashMap;
use spin::Mutex;
use wie_backend::{AudioSink, Database, DatabaseRepository, Filesystem, Instant, Platform, RecordId, Screen, canvas::Image};
use wie_util::Result;

use crate::filesystem::MemoryFilesystem;

static TEST_EPOCH: AtomicU64 = AtomicU64::new(0);

pub enum TestPlatformEvent {
    Stdout(Vec<u8>),
    Exit,
}

pub struct TestPlatform {
    screen: TestScreen,
    event_handler: Option<Box<dyn Fn(TestPlatformEvent) + Sync + Send>>,
    fs: Arc<MemoryFilesystem>,
    db: Arc<MemoryDatabaseRepository>,
}

impl Default for TestPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl TestPlatform {
    pub fn new() -> Self {
        Self {
            screen: TestScreen::default(),
            event_handler: None,
            fs: Arc::new(MemoryFilesystem::default()),
            db: Arc::new(MemoryDatabaseRepository::default()),
        }
    }

    pub fn with_event_handler<T>(event_handler: T) -> Self
    where
        T: Fn(TestPlatformEvent) + Sync + Send + 'static,
    {
        Self {
            screen: TestScreen::default(),
            event_handler: Some(Box::new(event_handler)),
            fs: Arc::new(MemoryFilesystem::default()),
            db: Arc::new(MemoryDatabaseRepository::default()),
        }
    }
}

impl TestPlatform {
    /// Frames the guest actually composed — the positive half of a boot assertion.
    /// Take these BEFORE boxing the platform; the emulator consumes the value.
    pub fn paint_counter(&self) -> Arc<AtomicUsize> {
        self.screen.counter()
    }

    /// Set when the core has a frame ready. The caller must reply with
    /// `Event::Redraw`, or no paint ever happens (see `TestScreen`).
    pub fn redraw_flag(&self) -> Arc<AtomicBool> {
        self.screen.redraw_flag()
    }
}

impl Platform for TestPlatform {
    fn screen(&self) -> &dyn Screen {
        &self.screen
    }

    fn now(&self) -> Instant {
        let epoch = TEST_EPOCH.fetch_add(8, Ordering::SeqCst);
        Instant::from_epoch_millis(epoch) // TODO
    }

    fn database_repository(&self) -> &dyn DatabaseRepository {
        self.db.as_ref()
    }

    fn filesystem(&self) -> &dyn Filesystem {
        self.fs.as_ref()
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(TestAudioSink)
    }

    fn write_stdout(&self, buf: &[u8]) {
        if let Some(event_handler) = &self.event_handler {
            (event_handler)(TestPlatformEvent::Stdout(buf.to_vec()))
        }
    }

    fn write_stderr(&self, _buf: &[u8]) {}

    fn exit(&self) {
        if let Some(event_handler) = &self.event_handler {
            (event_handler)(TestPlatformEvent::Exit);
        }
    }

    fn vibrate(&self, _duration_ms: u64, _intensity: u8) {}
}

type DatabaseKey = (String, String);
type DatabaseStore = HashMap<DatabaseKey, HashMap<RecordId, Vec<u8>>>;

#[derive(Default)]
struct MemoryDatabaseRepository {
    store: Arc<Mutex<DatabaseStore>>,
}

#[async_trait::async_trait]
impl DatabaseRepository for MemoryDatabaseRepository {
    async fn open(&self, _system: &wie_backend::System, name: &str, app_id: &str) -> Box<dyn Database> {
        let key = (app_id.to_string(), name.to_string());
        self.store.lock().entry(key.clone()).or_default();
        Box::new(MemoryDatabase {
            store: self.store.clone(),
            key,
        })
    }

    async fn exists(&self, _system: &wie_backend::System, name: &str, app_id: &str) -> bool {
        self.store.lock().contains_key(&(app_id.to_string(), name.to_string()))
    }

    async fn delete(&self, _system: &wie_backend::System, name: &str, app_id: &str) -> bool {
        self.store.lock().remove(&(app_id.to_string(), name.to_string())).is_some()
    }
}

struct MemoryDatabase {
    store: Arc<Mutex<DatabaseStore>>,
    key: DatabaseKey,
}

#[async_trait::async_trait]
impl Database for MemoryDatabase {
    async fn next_id(&self) -> RecordId {
        let store = self.store.lock();
        let records = store.get(&self.key);
        let mut id = 1;
        while records.is_some_and(|records| records.contains_key(&id)) {
            id += 1;
        }
        id
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        let id = self.next_id().await;
        self.set(id, data).await;
        id
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        self.store.lock().get(&self.key)?.get(&id).cloned()
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let mut store = self.store.lock();
        store.entry(self.key.clone()).or_default().insert(id, data.to_vec());
        true
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        self.store.lock().get_mut(&self.key).is_some_and(|records| records.remove(&id).is_some())
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        self.store
            .lock()
            .get(&self.key)
            .map(|records| records.keys().copied().collect())
            .unwrap_or_default()
    }
}

pub struct TestAudioSink;

impl AudioSink for TestAudioSink {
    fn play_wave(&self, _channel: u8, _sampling_rate: u32, _wave_data: &[i16]) {
        todo!()
    }

    fn midi_note_on(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        todo!()
    }

    fn midi_note_off(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        todo!()
    }

    fn midi_program_change(&self, _channel_id: u8, _program: u8) {
        todo!()
    }

    fn midi_control_change(&self, _channel_id: u8, _control: u8, _value: u8) {
        todo!()
    }
}

/// Counts `paint` calls so a headless test can assert the guest actually reached
/// the screen, not merely that nothing threw.
///
/// Why a count and not a framebuffer: the pixel question already has an owner —
/// the browser round-trip asserts real canvas pixels. What `cargo test --all`
/// could not answer at all was the coarser one, "did a frame ever get composed",
/// and that is exactly the axis a J2ME guest lost on 2026-09-04 (four gates green
/// while the guest died with NoClassDefFoundError before the first paint).
/// Records the two screen signals a headless test needs, so it can assert the
/// guest reached the screen instead of merely "nothing threw".
///
/// ★The redraw flag is load-bearing, not bookkeeping: painting is a REQUEST/REPLY
/// loop. The core calls `request_redraw()` when it has a frame ready and only
/// composes (`Screen::paint`) once the host feeds an `Event::Redraw` back. A test
/// that ticks without replaying that reply never sees a paint — measured here
/// first (0 paints in 10,000 ticks) before this flag existed. `wie_validate` runs
/// exactly this loop (`wie_cli/src/bin/wie_validate.rs`, "faithfully reproduce the
/// windowed flow"); this is the same shape, not a new one.
///
/// The pixel question stays with the browser round-trip, which asserts real canvas
/// pixels. What this answers is the coarser one: was a frame ever composed.
#[derive(Default)]
pub struct TestScreen {
    paints: Arc<AtomicUsize>,
    redraw_requested: Arc<AtomicBool>,
}

impl TestScreen {
    /// Shared handles, so a caller can keep observing after the platform has been
    /// boxed and handed to the emulator (which takes it by value).
    pub fn counter(&self) -> Arc<AtomicUsize> {
        self.paints.clone()
    }

    pub fn redraw_flag(&self) -> Arc<AtomicBool> {
        self.redraw_requested.clone()
    }
}

impl Screen for TestScreen {
    fn request_redraw(&self) -> Result<()> {
        self.redraw_requested.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn paint(&self, _image: &dyn Image) {
        self.paints.fetch_add(1, Ordering::SeqCst);
    }

    fn width(&self) -> u32 {
        320
    }

    fn height(&self) -> u32 {
        240
    }
}
