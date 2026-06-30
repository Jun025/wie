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
    AudioSink, Database, DatabaseRepository, Emulator, Event, Filesystem, Instant, KeyCode, Options, Platform, RecordId, Screen, canvas::Image,
    extract_zip,
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
    /// Set once any painted frame contains >=2 distinct pixel values, i.e. the
    /// game drew real content rather than a uniform blank/black screen.
    saw_content: AtomicBool,
    /// Largest magenta-pixel count seen in any single painted frame (the color-key
    /// 0xFF00FF leaking through), tracked across the whole run since the menu that
    /// shows it may scroll away before the final frame.
    max_magenta_px: AtomicU64,
    /// Content-richness metrics, each tracked as the MAX over all painted frames
    /// (the "richest" frame the run ever produced — content may scroll in/out).
    /// These distinguish a real game frame from a chrome-only blank (a UI shell:
    /// a status bar + an empty canvas box + a thin border) which the coarse
    /// `saw_content` >=2-colors test mis-reads as content. Measured for every game;
    /// see `frame_richness`.
    ///
    /// Distinct color count in the richest frame (capped at RICHNESS_COLOR_CAP).
    max_distinct_colors: AtomicU64,
    /// Fraction (basis points, 0..10000) of the richest frame NOT equal to its single
    /// most common color — i.e. how much of the screen is non-background.
    max_nondominant_bp: AtomicU64,
    /// Fraction (basis points) of the CENTER region (chrome-excluded: skips the top
    /// status bar, bottom soft-key strip, and side borders) that differs from that
    /// region's dominant color. A chrome-only shell has a uniform (empty) center, so
    /// this stays ~0; a real game draws content into the center, so it rises.
    max_center_nonuniform_bp: AtomicU64,
}

/// Distinct-color counting stops here (a real game frame blows past this; the cap
/// just bounds the per-frame set size).
const RICHNESS_COLOR_CAP: usize = 512;

/// Compute the three richness metrics for one frame: (distinct color count capped at
/// RICHNESS_COLOR_CAP, non-dominant fraction in basis points, center-region
/// non-uniform fraction in basis points). Pure function, unit-tested.
fn frame_richness(data: &[u32], width: u32, height: u32) -> (u64, u64, u64) {
    use std::collections::HashMap;

    if data.is_empty() {
        return (0, 0, 0);
    }

    // Whole-frame distinct colors + dominant color.
    let mut counts: HashMap<u32, u32> = HashMap::new();
    for &p in data {
        if counts.len() < RICHNESS_COLOR_CAP || counts.contains_key(&p) {
            *counts.entry(p).or_insert(0) += 1;
        }
    }
    let distinct = counts.len() as u64;
    let dominant = counts.values().copied().max().unwrap_or(0) as u64;
    let total = data.len() as u64;
    let nondominant_bp = ((total - dominant) * 10000) / total;

    // Center region: skip top 28% (status bar / title chrome), bottom 12% (soft-key
    // strip), and 12% side margins (borders). What's left is where real gameplay /
    // menu content lives; a chrome-only shell leaves it uniform.
    let (w, h) = (width as usize, height as usize);
    let x0 = w * 12 / 100;
    let x1 = w - x0;
    let y0 = h * 28 / 100;
    let y1 = h - h * 12 / 100;
    let mut center: HashMap<u32, u32> = HashMap::new();
    let mut center_total: u64 = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let idx = y * w + x;
            if idx < data.len() {
                *center.entry(data[idx]).or_insert(0) += 1;
                center_total += 1;
            }
        }
    }
    let center_bp = {
        let center_dominant = center.values().copied().max().unwrap_or(0) as u64;
        ((center_total - center_dominant) * 10000).checked_div(center_total).unwrap_or(0)
    };

    (distinct, nondominant_bp, center_bp)
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
        if let Some(first) = data.first()
            && data.iter().any(|p| p != first)
        {
            self.saw_content.store(true, Ordering::SeqCst);
        }
        let magenta = data
            .iter()
            .filter(|&&p| ((p >> 16) & 0xff) > 200 && (p & 0xff) > 200 && ((p >> 8) & 0xff) < 60)
            .count() as u64;
        self.max_magenta_px.fetch_max(magenta, Ordering::SeqCst);

        let (distinct, nondominant_bp, center_bp) = frame_richness(&data, self.width, self.height);
        self.max_distinct_colors.fetch_max(distinct, Ordering::SeqCst);
        self.max_nondominant_bp.fetch_max(nondominant_bp, Ordering::SeqCst);
        self.max_center_nonuniform_bp.fetch_max(center_bp, Ordering::SeqCst);

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
    /// Enable scripted input injection (press confirm/soft keys, navigate with
    /// arrows + select) after boot, watching for crash/panic/hang/blank screen
    /// across the whole sequence. NOTE: this only proves the game does not die on
    /// input; it cannot judge whether the on-screen result is visually correct.
    #[arg(long, default_value_t = false)]
    inject: bool,
    /// Directory for time-series screenshots (one per input step; filename
    /// encodes the step + key). Requires --inject.
    #[arg(long)]
    shotdir: Option<PathBuf>,
    /// Seconds to let the game boot before the first injected input.
    #[arg(long, default_value_t = 2.5)]
    boot_secs: f64,
    /// Seconds per injected input step (press, then settle + screenshot).
    #[arg(long, default_value_t = 0.6)]
    action_secs: f64,
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
        "{{\"file\":{:?},\"platform\":{:?},\"result\":{:?},\"reason\":{:?},\"ticks\":{},\"paints\":{},\"content\":{},\
         \"distinct_colors\":{},\"nondominant_pct\":{:.1},\"center_nonuniform_pct\":{:.1},\"ms\":{}}}",
        args.filename,
        result.platform,
        if result.passed { "PASS" } else { "FAIL" },
        result.reason,
        result.ticks,
        result.paints,
        result.content,
        result.distinct_colors,
        result.nondominant_bp as f64 / 100.0,
        result.center_nonuniform_bp as f64 / 100.0,
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
    content: bool,
    // Content-richness metrics (measure-only; do not affect passed). See HeadlessScreen.
    distinct_colors: u64,
    nondominant_bp: u64,
    center_nonuniform_bp: u64,
}

fn run(args: &Args) -> Outcome {
    let screen = Arc::new(HeadlessScreen {
        width: SCREEN_W,
        height: SCREEN_H,
        paints: AtomicU64::new(0),
        redraw_requested: AtomicBool::new(false),
        last_frame: Mutex::new(None),
        saw_content: AtomicBool::new(false),
        max_magenta_px: AtomicU64::new(0),
        max_distinct_colors: AtomicU64::new(0),
        max_nondominant_bp: AtomicU64::new(0),
        max_center_nonuniform_bp: AtomicU64::new(0),
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
        Err(e) => return fail("unknown", format!("read error: {e}"), 0, 0, false),
    };

    let load = catch_unwind(AssertUnwindSafe(|| build_emulator(platform, &args.filename, buf)));
    let (mut emulator, platform_name) = match load {
        Ok(Ok(v)) => v,
        Ok(Err((name, e))) => return fail(&name, format!("load error: {e}"), 0, 0, false),
        Err(p) => return fail("unknown", format!("load panic: {}", panic_message(&p)), 0, 0, false),
    };

    // ── input schedule (scripted fuzzing) ────────────────────────────────────
    let stem = std::path::Path::new(&args.filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("game")
        .to_string();

    let mut schedule: Vec<(f64, ScheduledEv)> = Vec::new();
    let mut deadline_secs = args.timeout as f64;
    if args.inject {
        // Confirm keys (OK/soft-key/keypad-5) interleaved with directional
        // navigation + select — covers single-button and two-button menus.
        let script: &[(KeyCode, &str)] = &[
            (KeyCode::OK, "OK"),
            (KeyCode::OK, "OK"),
            (KeyCode::LEFT_SOFT_KEY, "LSOFT"),
            (KeyCode::NUM5, "NUM5"),
            (KeyCode::DOWN, "DOWN"),
            (KeyCode::OK, "OK"),
            (KeyCode::DOWN, "DOWN"),
            (KeyCode::OK, "OK"),
            (KeyCode::UP, "UP"),
            (KeyCode::OK, "OK"),
            (KeyCode::LEFT, "LEFT"),
            (KeyCode::OK, "OK"),
            (KeyCode::RIGHT, "RIGHT"),
            (KeyCode::OK, "OK"),
            (KeyCode::NUM5, "NUM5"),
            (KeyCode::LEFT_SOFT_KEY, "LSOFT"),
            (KeyCode::RIGHT_SOFT_KEY, "RSOFT"),
            (KeyCode::DOWN, "DOWN"),
            (KeyCode::DOWN, "DOWN"),
            (KeyCode::OK, "OK"),
            (KeyCode::UP, "UP"),
            (KeyCode::OK, "OK"),
            (KeyCode::STAR, "STAR"),
            (KeyCode::HASH, "HASH"),
            (KeyCode::NUM1, "NUM1"),
            (KeyCode::OK, "OK"),
            (KeyCode::OK, "OK"),
        ];
        schedule.push((args.boot_secs, ScheduledEv::Shot("00_boot".into())));
        let mut t = args.boot_secs + 0.3;
        for (i, (kc, name)) in script.iter().enumerate() {
            let label = format!("{:02}_{name}", i + 1);
            schedule.push((t, ScheduledEv::Key(*kc, true, label.clone())));
            schedule.push((t + 0.15, ScheduledEv::Key(*kc, false, label.clone())));
            schedule.push((t + args.action_secs - 0.05, ScheduledEv::Shot(label)));
            t += args.action_secs;
        }
        deadline_secs = (t + 1.0).min(120.0); // hard cap against runaway
    }
    let deadline = Duration::from_secs_f64(deadline_secs.max(1.0));

    // ── drive ───────────────────────────────────────────────────────────────
    let loop_start = StdInstant::now();
    let mut ticks = 0u64;
    let mut run_err: Option<String> = None;
    let mut phase = String::from("boot");
    let mut sched_idx = 0usize;

    while !exited.load(Ordering::SeqCst) {
        let elapsed = loop_start.elapsed();
        if elapsed > deadline || ticks >= args.max_ticks {
            break;
        }

        // Fire any scheduled input/screenshot events that are now due.
        while sched_idx < schedule.len() && schedule[sched_idx].0 <= elapsed.as_secs_f64() {
            match &schedule[sched_idx].1 {
                ScheduledEv::Key(kc, down, label) => {
                    phase = label.clone();
                    let ev = if *down { Event::Keydown(*kc) } else { Event::Keyup(*kc) };
                    if let Err(p) = catch_unwind(AssertUnwindSafe(|| emulator.handle_event(ev))) {
                        run_err = Some(format!("panic on input '{label}': {}", panic_message(&p)));
                        break;
                    }
                }
                ScheduledEv::Shot(label) => {
                    if let Some(dir) = &args.shotdir
                        && let Some(frame) = screen.last_frame.lock().unwrap().as_ref()
                    {
                        let _ = save_png(&dir.join(format!("{stem}__{label}.png")), frame, SCREEN_W, SCREEN_H);
                    }
                }
            }
            sched_idx += 1;
        }
        if run_err.is_some() {
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
                run_err = Some(format!("tick error during '{phase}': {e}"));
                break;
            }
            Err(p) => {
                run_err = Some(format!("panic during '{phase}': {}", panic_message(&p)));
                break;
            }
        }
        ticks += 1;
    }

    let paints = screen.paints.load(Ordering::SeqCst);
    let content = screen.saw_content.load(Ordering::SeqCst);

    // ── final screenshot (back-compat single frame) ───────────────────────────
    if let Some(path) = &args.screenshot
        && let Some(frame) = screen.last_frame.lock().unwrap().as_ref()
        && let Err(e) = save_png(path, frame, SCREEN_W, SCREEN_H)
    {
        eprintln!("screenshot write failed: {e}");
    }

    // Peak fraction of any frame that was the magenta color-key (RGB ~255,0,255).
    // A large magenta area means a sprite/offscreen blit didn't drop the key.
    // Calibrated above the levels normal screens reach, so the regression-baseline
    // games do not false-fail. This catches the color-key class; it cannot judge
    // subtler glyph/graphic correctness (that stays a human check).
    let magenta_frac = screen.max_magenta_px.load(Ordering::SeqCst) as f64 / (SCREEN_W * SCREEN_H) as f64;

    // ── classify ─────────────────────────────────────────────────────────────
    // NOTE: the richness metrics below are MEASURE-ONLY — they are recorded in the
    // JSON for triage but deliberately do NOT influence PASS/FAIL (phase A1). The
    // PASS/FAIL logic is byte-for-byte the existing behaviour.
    let mut outcome = if let Some(reason) = run_err {
        fail(&platform_name, reason, ticks, paints, content)
    } else if exited.load(Ordering::SeqCst) {
        pass(&platform_name, "clean exit".into(), ticks, paints, content)
    } else if magenta_frac >= 0.15 {
        fail(
            &platform_name,
            format!("render: magenta color-key not applied ({:.0}% of frame)", magenta_frac * 100.0),
            ticks,
            paints,
            content,
        )
    } else if content {
        let reason = if args.inject {
            "booted + rendered + survived input sequence (visual correctness NOT checked)"
        } else {
            "booted + rendered"
        };
        pass(&platform_name, reason.into(), ticks, paints, content)
    } else if paints >= 1 {
        fail(&platform_name, "only blank/uniform frames (black screen)".into(), ticks, paints, content)
    } else {
        fail(&platform_name, "no frame rendered (hang/black screen)".into(), ticks, paints, content)
    };

    outcome.distinct_colors = screen.max_distinct_colors.load(Ordering::SeqCst);
    outcome.nondominant_bp = screen.max_nondominant_bp.load(Ordering::SeqCst);
    outcome.center_nonuniform_bp = screen.max_center_nonuniform_bp.load(Ordering::SeqCst);
    outcome
}

enum ScheduledEv {
    /// key code, is_down, step label
    Key(KeyCode, bool, String),
    /// screenshot with step label
    Shot(String),
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

fn pass(platform: &str, reason: String, ticks: u64, paints: u64, content: bool) -> Outcome {
    Outcome {
        platform: platform.to_string(),
        passed: true,
        reason,
        ticks,
        paints,
        content,
        distinct_colors: 0,
        nondominant_bp: 0,
        center_nonuniform_bp: 0,
    }
}

fn fail(platform: &str, reason: String, ticks: u64, paints: u64, content: bool) -> Outcome {
    Outcome {
        platform: platform.to_string(),
        passed: false,
        reason,
        ticks,
        paints,
        content,
        distinct_colors: 0,
        nondominant_bp: 0,
        center_nonuniform_bp: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{RICHNESS_COLOR_CAP, frame_richness};

    const W: u32 = 240;
    const H: u32 = 320;

    fn solid(color: u32) -> Vec<u32> {
        vec![color; (W * H) as usize]
    }

    #[test]
    fn blank_screen_is_not_rich() {
        let (distinct, nondominant, center) = frame_richness(&solid(0x000000), W, H);
        assert_eq!(distinct, 1);
        assert_eq!(nondominant, 0);
        assert_eq!(center, 0);
    }

    #[test]
    fn chrome_only_shell_has_empty_center() {
        // A green status bar (top 12%) + a thin green border around an otherwise
        // all-white canvas: the 게임빌/놈ZERO blank-but-PASS pattern. The chrome adds
        // a couple of colors and some non-dominant pixels, but the CENTER region is
        // uniform white, so center_nonuniform must stay ~0.
        let (w, h) = (W as usize, H as usize);
        let mut frame = vec![0x00FF_FFFFu32; w * h]; // white
        for y in 0..h {
            for x in 0..w {
                let bar = y < h * 12 / 100;
                let border = x == 0 || x == w - 1 || y == 0 || y == h - 1;
                if bar || border {
                    frame[y * w + x] = 0x009A_CD00; // lime-green chrome
                }
            }
        }
        let (distinct, _nondominant, center) = frame_richness(&frame, W, H);
        assert!(distinct <= 3, "chrome-only should have very few colors, got {distinct}");
        assert_eq!(center, 0, "chrome-only center must be uniform (empty canvas)");
    }

    #[test]
    fn textured_center_is_rich() {
        // A frame whose center carries a multi-colour pattern (a real game scene):
        // distinct colors high and center_nonuniform well above zero.
        let (w, h) = (W as usize, H as usize);
        let mut frame = vec![0x0000_0000u32; w * h];
        for y in 0..h {
            for x in 0..w {
                // a noisy gradient-ish pattern across the whole frame
                frame[y * w + x] = (((x * 7 + y * 13) % 251) as u32) << 8 | ((x ^ y) as u32 & 0xFF);
            }
        }
        let (distinct, nondominant, center) = frame_richness(&frame, W, H);
        assert_eq!(distinct as usize, RICHNESS_COLOR_CAP, "rich frame should hit the color cap");
        assert!(nondominant > 5000, "rich frame: most pixels non-dominant, got {nondominant}bp");
        assert!(center > 5000, "rich frame: center is non-uniform, got {center}bp");
    }
}
