//! Headless game validator for batch triage.
//!
//! Boots a game with a windowless platform, drives the emulator for a bounded
//! number of ticks (faithfully replaying the request_redraw -> Redraw flow the
//! windowed CLI relies on), and classifies the result:
//!
//!   PASS  -> emulator exited cleanly, or rendered at least one frame, with no
//!            error or panic within the time/tick budget.
//!   FAIL  -> tick returned an error, the emulator panicked, or the budget was
//!            exhausted without a clean exit or any rendered frame (hang/black).
//!
//! Emits one JSON line on stdout and exits 0 (PASS) / 1 (FAIL). Optionally
//! writes a PNG of the last rendered frame for visual spot-checks.
//!
//! This is a triage tool, not a correctness oracle: a headless run cannot prove
//! a game is visually correct or audible. It catches crashes/hangs/black boots.

extern crate alloc;

use std::{
    fs::{self, File},
    io::BufWriter,
    panic::{AssertUnwindSafe, catch_unwind},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant as StdInstant, SystemTime, UNIX_EPOCH},
};

use clap::Parser;

use wie_backend::{
    AudioSink, Database, DatabaseRepository, Emulator, Event, Filesystem, Instant, Options, Platform, RecordId, Screen, canvas::Image, extract_zip,
};
use wie_j2me::J2MEEmulator;
use wie_ktf::KtfEmulator;
use wie_lgt::LgtEmulator;
use wie_skt::SktEmulator;
use wie_util::Result as WieResult;

use test_utils::MemoryFilesystem;

// ── headless screen (captures last frame, counts paints) ─────────────────────

struct HeadlessScreen {
    width: u32,
    height: u32,
    paints: AtomicU64,
    redraw_requested: AtomicBool,
    last_frame: Mutex<Option<Vec<u32>>>,
}

impl Screen for HeadlessScreen {
    fn request_redraw(&self) -> WieResult<()> {
        self.redraw_requested.store(true, Ordering::SeqCst);
        Ok(())
    }

    fn paint(&self, image: &dyn Image) {
        let data = image
            .colors()
            .iter()
            .map(|x| ((x.a as u32) << 24) | ((x.r as u32) << 16) | ((x.g as u32) << 8) | (x.b as u32))
            .collect::<Vec<_>>();
        *self.last_frame.lock().unwrap() = Some(data);
        self.paints.fetch_add(1, Ordering::SeqCst);
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

// ── no-op audio (never panics, unlike test_utils' TestAudioSink) ─────────────

struct HeadlessAudioSink;

impl AudioSink for HeadlessAudioSink {
    fn play_wave(&self, _channel: u8, _sampling_rate: u32, _wave_data: &[i16]) {}
    fn midi_note_on(&self, _channel_id: u8, _note: u8, _velocity: u8) {}
    fn midi_note_off(&self, _channel_id: u8, _note: u8, _velocity: u8) {}
    fn midi_program_change(&self, _channel_id: u8, _program: u8) {}
    fn midi_control_change(&self, _channel_id: u8, _control: u8, _value: u8) {}
}

// ── in-memory database ───────────────────────────────────────────────────────

type DbKey = (String, String);
type DbStore = std::collections::HashMap<DbKey, std::collections::HashMap<RecordId, Vec<u8>>>;

#[derive(Default)]
struct MemDbRepository {
    store: Arc<Mutex<DbStore>>,
}

#[async_trait::async_trait]
impl DatabaseRepository for MemDbRepository {
    async fn open(&self, _system: &wie_backend::System, name: &str, app_id: &str) -> Box<dyn Database> {
        let key = (app_id.to_string(), name.to_string());
        self.store.lock().unwrap().entry(key.clone()).or_default();
        Box::new(MemDatabase {
            store: self.store.clone(),
            key,
        })
    }

    async fn exists(&self, _system: &wie_backend::System, name: &str, app_id: &str) -> bool {
        self.store.lock().unwrap().contains_key(&(app_id.to_string(), name.to_string()))
    }

    async fn delete(&self, _system: &wie_backend::System, name: &str, app_id: &str) -> bool {
        self.store.lock().unwrap().remove(&(app_id.to_string(), name.to_string())).is_some()
    }
}

struct MemDatabase {
    store: Arc<Mutex<DbStore>>,
    key: DbKey,
}

#[async_trait::async_trait]
impl Database for MemDatabase {
    async fn next_id(&self) -> RecordId {
        let store = self.store.lock().unwrap();
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
        self.store.lock().unwrap().get(&self.key)?.get(&id).cloned()
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let mut store = self.store.lock().unwrap();
        store.entry(self.key.clone()).or_default().insert(id, data.to_vec());
        true
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        self.store
            .lock()
            .unwrap()
            .get_mut(&self.key)
            .is_some_and(|records| records.remove(&id).is_some())
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        self.store
            .lock()
            .unwrap()
            .get(&self.key)
            .map(|records| records.keys().copied().collect())
            .unwrap_or_default()
    }
}

// ── headless platform ────────────────────────────────────────────────────────

struct HeadlessPlatform {
    screen: Arc<HeadlessScreen>,
    fs: MemoryFilesystem,
    db: MemDbRepository,
    stdout: Arc<Mutex<Vec<u8>>>,
    exited: Arc<AtomicBool>,
}

impl Platform for HeadlessPlatform {
    fn screen(&self) -> &dyn Screen {
        self.screen.as_ref()
    }

    fn now(&self) -> Instant {
        let since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        Instant::from_epoch_millis(since.as_millis() as _)
    }

    fn database_repository(&self) -> &dyn DatabaseRepository {
        &self.db
    }

    fn filesystem(&self) -> &dyn Filesystem {
        &self.fs
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(HeadlessAudioSink)
    }

    fn write_stdout(&self, buf: &[u8]) {
        self.stdout.lock().unwrap().extend_from_slice(buf);
    }

    fn write_stderr(&self, _buf: &[u8]) {}

    fn exit(&self) {
        self.exited.store(true, Ordering::SeqCst);
    }

    fn vibrate(&self, _duration_ms: u64, _intensity: u8) {}
}

// ── CLI ──────────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(about = "Headless game validator for batch triage")]
struct Args {
    /// Path to the game file (.zip / .jar / .jad)
    filename: String,
    /// Wall-clock budget in seconds before giving up
    #[arg(long, default_value_t = 20)]
    timeout: u64,
    /// Maximum number of emulator ticks before giving up. High by default so the
    /// wall-clock --timeout is the real budget; this is only an infinite-loop
    /// backstop. Heavy LGT games can need millions of ticks to reach first paint.
    #[arg(long, default_value_t = 50_000_000)]
    max_ticks: u64,
    /// Write a PNG of the last rendered frame here
    #[arg(long)]
    screenshot: Option<PathBuf>,
}

const SCREEN_W: u32 = 240;
const SCREEN_H: u32 = 320;

fn main() {
    // Honor RUST_LOG for debugging (logs to stderr, separate from the JSON result on stdout).
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();
    let start = StdInstant::now();

    let result = run(&args);
    let elapsed_ms = start.elapsed().as_millis();

    // Emit a single JSON line for the batch wrapper to parse.
    let json = format!(
        "{{\"file\":{:?},\"platform\":{:?},\"result\":{:?},\"reason\":{:?},\"ticks\":{},\"paints\":{},\"ms\":{}}}",
        args.filename,
        result.platform,
        if result.passed { "PASS" } else { "FAIL" },
        result.reason,
        result.ticks,
        result.paints,
        elapsed_ms
    );
    println!("{json}");

    std::process::exit(if result.passed { 0 } else { 1 });
}

struct Outcome {
    platform: String,
    passed: bool,
    reason: String,
    ticks: u64,
    paints: u64,
}

fn run(args: &Args) -> Outcome {
    let screen = Arc::new(HeadlessScreen {
        width: SCREEN_W,
        height: SCREEN_H,
        paints: AtomicU64::new(0),
        redraw_requested: AtomicBool::new(false),
        last_frame: Mutex::new(None),
    });
    let exited = Arc::new(AtomicBool::new(false));
    let stdout = Arc::new(Mutex::new(Vec::new()));

    let platform = Box::new(HeadlessPlatform {
        screen: screen.clone(),
        fs: MemoryFilesystem::new(),
        db: MemDbRepository::default(),
        stdout: stdout.clone(),
        exited: exited.clone(),
    });

    // ── load & route (mirrors wie_cli/src/main.rs) ──────────────────────────
    let buf = match fs::read(&args.filename) {
        Ok(b) => b,
        Err(e) => return fail("unknown", format!("read error: {e}"), 0, 0),
    };

    let load = catch_unwind(AssertUnwindSafe(|| build_emulator(platform, &args.filename, buf)));
    let (mut emulator, platform_name) = match load {
        Ok(Ok(v)) => v,
        Ok(Err((name, e))) => return fail(&name, format!("load error: {e}"), 0, 0),
        Err(p) => return fail("unknown", format!("load panic: {}", panic_message(&p)), 0, 0),
    };

    // ── drive ───────────────────────────────────────────────────────────────
    let timeout = Duration::from_secs(args.timeout);
    let loop_start = StdInstant::now();
    let mut ticks = 0u64;
    let mut run_err: Option<String> = None;

    while !exited.load(Ordering::SeqCst) {
        if loop_start.elapsed() > timeout || ticks >= args.max_ticks {
            break;
        }

        let step = catch_unwind(AssertUnwindSafe(|| {
            emulator.tick()?;
            // Faithfully reproduce the windowed flow: the emulator paints in
            // response to the Redraw event it requested via request_redraw.
            if screen.redraw_requested.swap(false, Ordering::SeqCst) {
                emulator.handle_event(Event::Redraw);
            }
            Ok::<(), wie_util::WieError>(())
        }));

        match step {
            Ok(Ok(())) => {}
            Ok(Err(e)) => {
                run_err = Some(format!("tick error: {e}"));
                break;
            }
            Err(p) => {
                run_err = Some(format!("panic: {}", panic_message(&p)));
                break;
            }
        }
        ticks += 1;
    }

    let paints = screen.paints.load(Ordering::SeqCst);

    // ── screenshot ──────────────────────────────────────────────────────────
    if let Some(path) = &args.screenshot
        && let Some(frame) = screen.last_frame.lock().unwrap().as_ref()
        && let Err(e) = save_png(path, frame, SCREEN_W, SCREEN_H)
    {
        eprintln!("screenshot write failed: {e}");
    }

    // ── classify ─────────────────────────────────────────────────────────────
    if let Some(reason) = run_err {
        return fail(&platform_name, reason, ticks, paints);
    }
    if exited.load(Ordering::SeqCst) {
        return pass(&platform_name, "clean exit".into(), ticks, paints);
    }
    if paints >= 1 {
        return pass(&platform_name, "booted + rendered".into(), ticks, paints);
    }
    fail(
        &platform_name,
        "no frame rendered and no exit within budget (hang/black screen)".into(),
        ticks,
        paints,
    )
}

#[allow(clippy::type_complexity)]
fn build_emulator(platform: Box<dyn Platform>, filename: &str, buf: Vec<u8>) -> std::result::Result<(Box<dyn Emulator>, String), (String, String)> {
    let options = Options {
        enable_gdbserver: false,
        profile: None,
    };

    if filename.ends_with("zip") {
        let files = extract_zip(&buf).map_err(|e| ("unknown".to_string(), format!("{e}")))?;
        if KtfEmulator::loadable_archive(&files) {
            let e = KtfEmulator::from_archive(platform, files, options).map_err(|e| ("ktf".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "ktf".into()))
        } else if LgtEmulator::loadable_archive(&files) {
            let e = LgtEmulator::from_archive(platform, files, options).map_err(|e| ("lgt".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "lgt".into()))
        } else if SktEmulator::loadable_archive(&files) {
            let e = SktEmulator::from_archive(platform, files).map_err(|e| ("skt".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "skt".into()))
        } else {
            Err(("unknown".to_string(), "unrecognized zip archive (no __adf__/app_info/.msd)".to_string()))
        }
    } else if filename.ends_with("jad") {
        let jar_filename = filename.replace(".jad", ".jar");
        let jar = fs::read(&jar_filename).map_err(|e| ("j2me".to_string(), format!("jar read: {e}")))?;
        let jar_name = jar_filename[jar_filename.rfind('/').unwrap_or(0) + 1..].to_owned();
        let e = J2MEEmulator::from_jad_jar(platform, buf, jar_name, jar).map_err(|e| ("j2me".to_string(), format!("{e}")))?;
        Ok((Box::new(e), "j2me".into()))
    } else if filename.ends_with("jar") {
        let name = filename[filename.rfind('/').unwrap_or(0) + 1..].to_owned();
        let stem = name.trim_end_matches(".jar");
        if KtfEmulator::loadable_jar(&buf) {
            let e = KtfEmulator::from_jar(platform, &name, buf, stem, stem, None, options).map_err(|e| ("ktf".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "ktf".into()))
        } else if LgtEmulator::loadable_jar(&buf) {
            let e = LgtEmulator::from_jar(platform, &name, buf, stem, stem, None, options).map_err(|e| ("lgt".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "lgt".into()))
        } else if SktEmulator::loadable_jar(&buf) {
            let e = SktEmulator::from_jar(platform, &name, buf, stem, None).map_err(|e| ("skt".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "skt".into()))
        } else {
            let e = J2MEEmulator::from_jar(platform, &name, buf).map_err(|e| ("j2me".to_string(), format!("{e}")))?;
            Ok((Box::new(e), "j2me".into()))
        }
    } else {
        Err(("unknown".to_string(), "unknown file extension".to_string()))
    }
}

fn save_png(path: &PathBuf, frame: &[u32], width: u32, height: u32) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut rgba = Vec::with_capacity(frame.len() * 4);
    for px in frame {
        rgba.push((px >> 16) as u8); // r
        rgba.push((px >> 8) as u8); // g
        rgba.push(*px as u8); // b
        rgba.push((px >> 24) as u8); // a
    }
    let file = BufWriter::new(File::create(path)?);
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgba)?;
    Ok(())
}

fn panic_message(p: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

fn pass(platform: &str, reason: String, ticks: u64, paints: u64) -> Outcome {
    Outcome {
        platform: platform.to_string(),
        passed: true,
        reason,
        ticks,
        paints,
    }
}

fn fail(platform: &str, reason: String, ticks: u64, paints: u64) -> Outcome {
    Outcome {
        platform: platform.to_string(),
        passed: false,
        reason,
        ticks,
        paints,
    }
}
