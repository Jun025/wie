//! Headless game validator for batch triage.
//!
//! Boots a game with a windowless platform, drives the emulator for a bounded
//! number of ticks (faithfully replaying the request_redraw -> Redraw flow the
//! windowed CLI relies on), and classifies the result:
//!
//!   PASS       -> emulator exited cleanly, or rendered at least one frame, with no
//!                 error or panic within the time/tick budget.
//!   FAIL       -> tick returned an error, the emulator panicked, or the budget was
//!                 exhausted without a clean exit or any rendered frame (hang/black).
//!   UNMEASURED -> the run ended before the thing being asked about was fully exercised.
//!                 Today that is exactly one case: `--inject` delivered FEWER input steps
//!                 than the script holds (see the `--inject` section below).
//!
//! Emits one JSON line on stdout and exits 0 (PASS) / 1 (FAIL) / 2 (UNMEASURED).
//! When the SVC stub space runs out that line (a FAIL) is written at that moment, because
//! the process may then abort — see `SVC_STUB_EXHAUSTED_MESSAGE`.
//! Optionally writes a PNG of the last rendered frame for visual spot-checks.
//!
//! This is a triage tool, not a correctness oracle: a headless run cannot prove
//! a game is visually correct or audible. It catches crashes/hangs/black boots.
//!
//! ── Why UNMEASURED is a third verdict and not just FAIL ──────────────────────
//! The two errors are not the same size. FAIL means "the game broke"; on this axis
//! that claim, made about a title that was never given an input, is as wrong as the
//! PASS it replaces — it would block a healthy title from registration just as the
//! PASS registered an unproven one. UNMEASURED says the only true thing: the run did
//! not get far enough to have an opinion. Exit 2 for it is this tree's existing
//! convention for that state (AGENTS.md, `ktf-image-sweep.py`: "Exit 2 is 'could not
//! measure', never 'found nothing'"), and it is fail-closed on BOTH axes a caller
//! might read: the exit code is non-zero, and `result` is not the string `PASS` that
//! all three in-tree callers grep for.
//!
//! ── `--inject`: the input axis is only measured if inputs actually ran ───────
//! The drive loop stops at whichever of four things comes first — a clean guest exit,
//! the `--max-ticks` backstop, the wall-clock deadline, or an error. The injection
//! SCHEDULE is wall-clock, so a fast title can burn the TICK backstop before the
//! first key is due and end with the whole script unfired. That used to report
//! `PASS ... survived input sequence` over zero delivered keys and zero `--shotdir`
//! frames — measured 2026-09-22 on a real title, and nearly used as registration
//! evidence. Raising the backstop's default does not close it: the next title that
//! is faster reopens it, which is the shape of the defect rather than its size.
//!
//! So the run reports what it did instead of what it was asked to do: `input_steps`
//! / `input_steps_total` count the keys actually delivered, `stop` names which of the
//! four ended the run, and a PASS that delivered fewer than all of them becomes UNMEASURED.
//!
//! That gate reads `input_steps < input_steps_total`, not `== 0`, since 2026-09-23. A run cut
//! off at 10 of 27 keys measured ten keys; calling it `survived input sequence` is the same
//! overclaim as calling zero that, only smaller. The corpus measurement behind the widening,
//! and the two titles it costs, are recorded on `inject_unmeasured` itself.
//!
//! Four opt-in flags change the schedule; with none given it is byte-for-byte the old one
//! (`plan_schedule_defaults_unchanged_test`):
//!
//!   --keys SCRIPT     replace the 27-key script (file path or inline list; grammar on `parse_keys`)
//!   --inject-keys N   inject only the first N script keys (input_steps_total = N)
//!   --keep-timeout    end at --timeout, not the schedule-derived deadline
//!   --shot-every S    extra --shotdir frame every S seconds, labelled `tNNN.N`
//!
//! They exist to pair a keyed run with an unkeyed one on the SAME budget — "is a title that
//! paints 3 frames waiting for a key, or stuck?" (docs/report/0233 needed a scratch patch).
//! `--keys` exists because the fixed script reaches a DIFFERENT screen under load: a menu that
//! wants a key 10 s in gets whatever the 0.6 s grid has queued by then. Three battlemonster
//! rounds each carried the same uncommitted `WIE_KEYS` patch to pin a path to the village;
//! `docs/examples/keys/battlemonster-village.keys` is that path. Under `--keys` the 120 s cap
//! on the schedule-derived deadline is lifted — the script's length IS the requested budget.
//!
//! ── Two content axes, same predicate, different scope ────────────────────────
//! `content`            — `has_content` ORed over EVERY painted frame ("did the game
//!                         ever draw something?"). This is what PASS/FAIL uses.
//! `last_frame_content` — the same predicate on the LAST painted frame only ("is the
//!                         screen a user ends up looking at non-blank?").
//!
//! They disagree exactly when a good frame is drawn and then OVERPAINTED, which the
//! any-frame axis cannot see by construction. Measured 2026-09-05 on LGT: 30 paints
//! arrived with an all-black image and the final frame was black, yet
//! `wie_validate --inject` reported PASS with `content: true` — the browser
//! round-trip was the only thing that caught it.
//!
//! The three content-RICHNESS metrics have the same shape and, since 2026-09-06, the same
//! pairing: `distinct_colors` / `nondominant_pct` / `center_nonuniform_pct` are each a MAX
//! over every painted frame (`fetch_max`, monotonic), so a later frame cannot pull them back
//! down — structurally the same blind spot. `last_frame_distinct_colors` /
//! `last_frame_nondominant_pct` / `last_frame_center_nonuniform_pct` are the same three
//! computed on the final frame alone, ONCE per run. They are REPORTED, never gated — not
//! even under `--expect-last-frame`, which reads the boolean pair below and no richness
//! field at all (`last_frame_gate_fails` takes three bools). Unlike that pair these are
//! numbers, and a number needs a threshold to fail anything. No threshold is defined here on
//! purpose — the point is to make one CHOOSABLE from data instead of invented.
//!
//! `last_frame_content` is REPORT-ONLY BY DEFAULT and becomes a gate only when the
//! caller passes `--expect-last-frame`. Two reasons it cannot be gated
//! unconditionally, both measured rather than assumed: a fixture may legitimately
//! end on a blank frame (helloworld_lgt draws nothing at all and exits cleanly),
//! and without `--inject` a key-driven fixture is black until a key arrives.
//!
//! ── Why the expectation lives on the COMMAND LINE and nowhere else ───────────
//! Gating needs a per-fixture expectation, and the axis is `fixture x mode`, not
//! `fixture`: measured 2026-09-06 on this tree,
//!
//!   helloworld_ktf.zip              PASS  content false  last_frame_content false
//!   helloworld_lgt.zip              PASS  content false  last_frame_content false
//!   keydraw_ktf.zip --inject        PASS  content true   last_frame_content TRUE
//!   keydraw_lgt.zip --inject        PASS  content true   last_frame_content TRUE
//!   keydraw_ktf.zip                 FAIL  content false  last_frame_content false
//!   keydraw_lgt.zip                 FAIL  content false  last_frame_content false
//!
//! — the same fixture expects a blank final frame in one mode and a drawn one in
//! the other. `--inject` is already a flag on that command line, so the mode half
//! of the key is *there and nowhere else*. A sidecar file next to the fixture, or
//! a fixture-name table inside this binary, would put the other half somewhere
//! else and make the pair a SECOND SOURCE OF TRUTH that drifts the moment a
//! fixture is renamed, a mode is added, or the tool is pointed at a file it has
//! never heard of. The flag keeps both halves in one place: the invocation.
//!
//! The cost is honest and worth stating: an opt-in flag only gates the callers
//! that pass it. It buys nothing for an invocation that forgets — but neither
//! would a sidecar the caller never reads. Callers today are `scripts/smoke_gate.sh`,
//! `scripts/lgt_render_probe.sh` and the AGENTS.md runner block; no workflow runs
//! this binary, so `--expect-last-frame` is a tool for those callers, not a CI gate
//! by itself.

extern crate alloc;

use std::{
    fs::{self, File},
    io::{BufWriter, LineWriter, Write as _},
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
    AudioSink, Database, DatabaseRepository, Emulator, Event, Filesystem, Font, FramePacer, Instant, KeyCode, Options, Platform, ProfileCallback,
    ProfileSample, RecordId, Screen, canvas::Image, extract_zip,
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

/// Does this frame contain content, i.e. >=2 distinct pixel values (as opposed to a
/// uniform blank/black screen)?
///
/// Deliberately shared by BOTH content axes so they differ only in SCOPE, never in
/// predicate: `saw_content` ORs it over every painted frame, `last_frame_content`
/// applies it to the final frame alone. Comparing two axes that also disagreed on the
/// test would prove nothing.
fn has_content(data: &[u32]) -> bool {
    match data.first() {
        Some(first) => data.iter().any(|p| p != first),
        None => false,
    }
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
    fn resize(&self, _width: u32, _height: u32) -> WieResult<()> {
        Ok(())
    }

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
        if has_content(&data) {
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
    fn send(&self, _command: wie_backend::AudioCommand) {}
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
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn Database> {
        let key = (app_id.to_string(), name.to_string());
        self.store.lock().unwrap().entry(key.clone()).or_default();
        Box::new(MemDatabase {
            store: self.store.clone(),
            key,
        })
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        self.store.lock().unwrap().contains_key(&(app_id.to_string(), name.to_string()))
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        self.store.lock().unwrap().remove(&(app_id.to_string(), name.to_string())).is_some()
    }

    async fn usage(&self, _app_id: &str) -> u64 {
        0
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
    font: Font,
}

impl Platform for HeadlessPlatform {
    // Carries a real font, for the same reason `wie_featurephone`'s `WebPlatform` does: any guest
    // that draws text reaches this, and an `unimplemented!()` here panics the *validator* rather
    // than the game. That is not a theoretical distinction — it was the single largest failure
    // signature in `game_lab/broken/`: games that booted and painted dozens of frames were filed
    // as broken the moment they drew their first string. Measured 2026-09-17 over a stratified
    // 15-game re-run, `panic … : not implemented` was 6/15, and 4 of those 6 had already painted
    // 31–135 frames. The bytes are the same `assets/neodgm.ttf` the browser host embeds, so the
    // two hosts lay text out identically; keep them in sync.
    fn font(&self) -> &Font {
        &self.font
    }

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
    /// Inject only the first N keys of the script (default: all of them). With
    /// `--keep-timeout` this pairs a keyed run against an unkeyed baseline on the
    /// same budget — `--inject-keys 0` vs `1` is "is it waiting for a key?".
    #[arg(long, requires = "inject")]
    inject_keys: Option<usize>,
    /// Key script replacing the built-in 27 keys: a file path, or the script inline.
    /// Whitespace-separated `NAME[:GAP[:HOLD]]` steps, `#` starts a comment; NAME is a
    /// key (`OK UP DOWN LEFT RIGHT LSOFT RSOFT CLR STAR HASH NUM0`..`NUM9`) or `WAIT`
    /// (no key, still a shot). GAP = seconds until the next step (default --action-secs),
    /// HOLD = seconds the key stays down (default 0.15). Lifts the 120 s deadline cap.
    #[arg(long, requires = "inject", value_parser = parse_keys)]
    keys: Option<KeyScript>,
    /// Under `--inject`, end at `--timeout` instead of the deadline derived from
    /// the key schedule, so runs injecting different key counts get the same budget.
    #[arg(long, default_value_t = false, requires = "inject")]
    keep_timeout: bool,
    /// Also write a `--shotdir` frame every SECS of wall time (`<stem>__tNNN.N.png`),
    /// independent of key steps — the way to see whether the screen still moves.
    #[arg(long, requires = "shotdir", value_parser = positive_secs)]
    shot_every: Option<f64>,
    /// Require the LAST painted frame to be non-blank, i.e. gate on
    /// `last_frame_content`. OFF by default so existing verdicts are unchanged;
    /// pass it for the fixture+mode combinations where a blank final screen is a
    /// defect (e.g. `keydraw_lgt.zip --inject`, but NOT `helloworld_lgt.zip`,
    /// which draws nothing at all and exits cleanly). This is the per-fixture
    /// expectation the module header says gating needs — see there for why it
    /// lives here rather than in a sidecar or a table.
    #[arg(long, default_value_t = false)]
    expect_last_frame: bool,
    /// Add the guest's stdout to the JSON line as `guest_stdout` (+ a
    /// `guest_stdout_truncated` flag). OFF by default, and the default output is
    /// byte-identical to before: with the flag absent neither key is emitted.
    ///
    /// It is opt-in because of Constraint 9, not because of cost. The guest's
    /// bytes never appear here — but whatever the guest *prints* does, and a real
    /// title can print a path. AGENTS.md's own smoke-gate note draws that line
    /// explicitly ("identifiers and expected status only, never paths or bytes"),
    /// so "this is not game bytes" mitigates the constraint rather than exempting
    /// it. Output is capped at GUEST_STDOUT_MAX_BYTES and truncation is reported,
    /// but nothing here can tell a path from any other string — do not paste this
    /// output into the repo or a shared log when running against a real game.
    #[arg(long, default_value_t = false)]
    guest_stdout: bool,
    /// Write the ARM core's sampling profile here in flamegraph-folded form
    /// (`0x<outer>;…;0x<pc> <count>`, one line per stack per flushed batch — the
    /// same format as `wie --profile-out`). OFF by default: without it no file is
    /// created and the JSON line is unchanged. KTF/LGT only; J2ME/SKT have no ARM
    /// core, so the file stays empty. Stacks are guest addresses, not game bytes.
    #[arg(long)]
    profile_out: Option<PathBuf>,
    /// Tick as a host with this display rate would: each tick gets the budget
    /// `FramePacer` derives from a `1 / HZ` s frame (60 -> 14ms, 120 -> 5ms)
    /// instead of the engine default (14ms). OFF by default, so existing runs are
    /// unchanged; this loop never sleeps, so it changes how finely a tick slices
    /// the guest, not how often ticks come.
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=1000))]
    frame_hz: Option<u32>,
}

/// Cap on the `guest_stdout` field, in bytes of the lossy-decoded text.
///
/// Sized from what the field is for: the committed fixtures print two short
/// markers, and a useful diagnostic is a handful of lines or a stack trace. 4 KiB
/// holds that with room to spare while keeping one JSON line pasteable and
/// bounding how much a chatty guest can dump into a caller's log.
const GUEST_STDOUT_MAX_BYTES: usize = 4096;

/// Escape a string for a JSON string literal.
///
/// Hand-rolled on purpose: `wie_cli` has no JSON dependency and this is the whole
/// of the format's string grammar. **`{:?}` is not a substitute** — Rust's Debug
/// renders a control byte as `\u{1}`, which JSON does not accept.
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Decode the collected guest stdout, cap it, and escape it for the JSON line.
///
/// Returns the escaped body and whether it was cut. Truncation is signalled by a
/// **separate key**, never by a marker inside the text: the text is the one part
/// of this line the guest controls, so an in-band marker would be forgeable by
/// the very source being bounded.
fn guest_stdout_field(raw: &[u8], max: usize) -> (String, bool) {
    let text = String::from_utf8_lossy(raw);
    let mut end = text.len().min(max);
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    (json_escape(&text[..end]), end < text.len())
}

// ── Java exceptions the runtime raised (report-only) ─────────────────────────
//
// Measured 2026-09-22 (docs/worklog/2026-09-22-aot-java-swallowed-io-stream-exception.json):
// four LGT-AOT titles all read `no frame rendered`, and what actually happened in each is
// that the runtime raised exactly one Java exception which the guest caught in its own
// handler and carried on without drawing. Finding that took `RUST_LOG` and counting
// `jvm::jvm: throwing java exception` lines by hand. This puts the count and the first one
// on the JSON line, so the next run shows it without a log.
//
// ★Where it comes from, and why this does not touch `jvm`: that crate is a crates.io
// dependency (`jvm 0.1.1`, not ours to edit), and every exception it raises goes through
// `Jvm::exception`, whose first statement is
// `tracing::info!("throwing java exception: {} {message}", r#type)` — target `jvm::jvm`.
// So a `tracing` layer on this side sees each one, with no upstream change. The test
// `java_exception_tally_sees_a_real_jvm_exception_test` drives the real `Jvm::exception`
// rather than a hand-written event, so if upstream rewords or re-levels that line, the
// test goes red instead of this field silently reading 0.
//
// ★What it does NOT see: an exception the guest constructs itself (`new` + `athrow` in
// bytecode) never passes through `Jvm::exception`. It counts what the RUNTIME raised —
// which is the class that hid behind `no frame rendered` — not every throw.
//
// ★Named `java_exceptions`, not `swallowed_exceptions`: the line fires at raise time and
// cannot tell whether the guest caught it. Whether the run ended in an error is `reason`'s
// job; this field says only that the runtime raised these.
//
// ★Reported, never gated. A healthy title can raise and catch as part of normal play —
// 배틀몬스터 raised IllegalMonitorState 288 times before the monitor fix — so a count
// has no threshold that means "broken". `passed` is decided without reading it.

/// Longest `first` kept, in bytes. A class name plus a message is well under this; the cap
/// only bounds a message that happens to embed guest data.
const JAVA_EXCEPTION_FIRST_MAX_BYTES: usize = 256;

/// Prefix of the `jvm` crate's `Jvm::exception` log line (jvm 0.1.1, `src/jvm.rs`).
const JVM_EXCEPTION_PREFIX: &str = "throwing java exception: ";

#[derive(Default)]
struct JavaExceptionTally {
    count: AtomicU64,
    /// `"<class>: <message>"` of the first one, as logged.
    first: Mutex<Option<String>>,
}

impl JavaExceptionTally {
    fn record(&self, line: &str) {
        let Some(rest) = line.strip_prefix(JVM_EXCEPTION_PREFIX) else {
            return;
        };
        self.count.fetch_add(1, Ordering::SeqCst);
        let mut first = self.first.lock().unwrap();
        if first.is_none() {
            // The log is `"{type} {message}"`; a JVM class name has no space in it.
            *first = Some(match rest.split_once(' ') {
                Some((class, message)) => format!("{class}: {message}"),
                None => rest.to_string(),
            });
        }
    }

    /// The JSON value: `{"count":N,"first":"<class>: <message>"|null,"first_truncated":bool}`.
    fn json(&self) -> String {
        let count = self.count.load(Ordering::SeqCst);
        match self.first.lock().unwrap().as_deref() {
            Some(first) => {
                let (body, truncated) = guest_stdout_field(first.as_bytes(), JAVA_EXCEPTION_FIRST_MAX_BYTES);
                format!("{{\"count\":{count},\"first\":\"{body}\",\"first_truncated\":{truncated}}}")
            }
            None => format!("{{\"count\":{count},\"first\":null,\"first_truncated\":false}}"),
        }
    }
}

// ── Stub hits and SVC stub space (report-only) ───────────────────────────────
//
// Several 2026-09-24 PASSes stand on minimal stubs that log a warning and hand back a default
// (`wec.SYSTheme::saveItem` warns and returns 0), and a default of 0 has already walked titles
// into a NULL dereference further on. Without a count, "PASS" and "PASS because a stub said 0"
// read the same. So the line carries how many stubs the run hit, and which.
//
// ★What counts as a stub is the convention the tree already has, not a new one: a WARN event
// whose message starts with `stub ` (any target). That is 278 of the 343 `warn!` sites. Not
// counted, on purpose: the 8 `debug!("stub …")` lwc/lcdui constructors (demoted because they
// are expected), and warnings that do not say `stub` (LGT `unk2/3/4` in `stdlib.rs`, skvm's
// `unsupported com.xce.io.XFile::…`, KTF's `Unknown {name}`). Rewording one of those to `stub …`
// is how it joins the count; the name is the text after `stub ` up to `(` or whitespace.
//
// ★SVC stub space: `ArmCore::make_svc_stub` carves 16-byte trampolines out of a fixed 64 KiB
// region, 4,096 of them, and fails when it runs out. `wie_core_arm::svc_stub_high_water()` is
// read at line-writing time — a counter, not a per-stub trace event: an earlier draft enabled
// TRACE on `wie_core_arm::core` for this, which raises `tracing`'s global level hint and halved
// `keydraw_lgt` paints (51/34/48 without it vs 10/10/15 with, same load, 2026-09-25). The one
// event the layer does listen for — exhaustion — is INFO, a level `jvm::jvm` already enables.
//
// ★Reported, never gated — like `java_exceptions`, `passed` is decided without reading these.

/// How many distinct stubs `stub_hits.first` lists.
const STUB_HITS_TOP: usize = 5;
/// Longest stub name kept, in bytes.
const STUB_NAME_MAX_BYTES: usize = 128;
/// The existing stub warning convention (`tracing::warn!("stub …")`).
const STUB_PREFIX: &str = "stub ";

#[derive(Default)]
struct StubHitTally {
    count: AtomicU64,
    by_name: Mutex<std::collections::BTreeMap<String, u64>>,
}

impl StubHitTally {
    fn record(&self, line: &str) {
        let Some(rest) = line.strip_prefix(STUB_PREFIX) else {
            return;
        };
        let name = &rest[..rest.find(|c: char| c == '(' || c.is_whitespace()).unwrap_or(rest.len())];
        self.count.fetch_add(1, Ordering::SeqCst);
        *self.by_name.lock().unwrap().entry(name.to_string()).or_default() += 1;
    }

    /// `{"count":N,"distinct":D,"first":[{"name":"…","count":n}, …]}` — `first` is the top
    /// `STUB_HITS_TOP` by hits, ties broken by name so the line is deterministic.
    fn json(&self) -> String {
        let by_name = self.by_name.lock().unwrap();
        let mut top: Vec<(&String, &u64)> = by_name.iter().collect();
        top.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let first = top
            .iter()
            .take(STUB_HITS_TOP)
            .map(|(name, count)| {
                format!(
                    "{{\"name\":\"{}\",\"count\":{count}}}",
                    guest_stdout_field(name.as_bytes(), STUB_NAME_MAX_BYTES).0
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"count\":{},\"distinct\":{},\"first\":[{first}]}}",
            self.count.load(Ordering::SeqCst),
            by_name.len()
        )
    }
}

/// `{"used":U,"capacity":C|null}` off `wie_core_arm`'s process-wide high-water mark. `capacity`
/// is `null` when no ARM core bound a stub (J2ME) — there was no stub space to measure.
fn svc_stub_slots_json() -> String {
    match wie_core_arm::svc_stub_high_water() {
        0 => "{\"used\":0,\"capacity\":null}".to_string(),
        used => format!("{{\"used\":{used},\"capacity\":{}}}", wie_core_arm::SVC_STUB_CAPACITY),
    }
}

/// Everything the validator's `tracing` layer counts for the JSON line.
#[derive(Default)]
struct Tallies {
    java_exceptions: JavaExceptionTally,
    stub_hits: StubHitTally,
    /// Set once the SVC stub space ran out and `on_svc_stub_exhausted` has run.
    svc_stub_exhausted: AtomicBool,
    /// Writes the result line at the moment of exhaustion — see `SVC_STUB_EXHAUSTED_MESSAGE`.
    on_svc_stub_exhausted: std::sync::OnceLock<ExhaustedHook>,
}

type ExhaustedHook = Box<dyn Fn(&Tallies) + Send + Sync>;

/// Message of `ArmCore::make_svc_stub`'s exhaustion trace.
///
/// ★Why the line is written from inside the layer: measured 2026-09-25 with the space capped at
/// 1,000 stubs, KTF hits an `unwrap()` on the error and the run ends as an ordinary FAIL, but
/// LGT recurses until `fatal runtime error: stack overflow, aborting` (rc 134) and `main` never
/// prints — a stack overflow cannot be caught. So the line goes out when the space runs out, and
/// the abort stays. Exactly one line either way: `main` stays silent afterwards unless its own
/// verdict would differ (not observed), in which case both print and the in-tree parsers'
/// "매치 N건(기대 1)" warning fires instead of a verdict changing silently.
const SVC_STUB_EXHAUSTED_MESSAGE: &str = "SVC stub space exhausted";

/// Target of `ArmCore::make_svc_stub`'s trace line.
const SVC_STUB_TARGET: &str = "wie_core_arm::core";

/// The `tracing` layer that feeds [`Tallies`]. Installed with its OWN filter (`jvm::jvm` and the
/// SVC stub target at INFO, WARN everywhere else — never TRACE/DEBUG, see the stub-hits header) so it sees those events whatever
/// `RUST_LOG` says, while the stderr logger keeps honouring `RUST_LOG` exactly as before.
struct TallyLayer(Arc<Tallies>);

impl<S: tracing::Subscriber> tracing_subscriber::Layer<S> for TallyLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        struct Message(Option<String>);
        impl tracing::field::Visit for Message {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn core::fmt::Debug) {
                if field.name() == "message" {
                    self.0 = Some(format!("{value:?}"));
                }
            }
            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                if field.name() == "message" {
                    self.0 = Some(value.to_string());
                }
            }
        }
        let mut message = Message(None);
        event.record(&mut message);
        let Some(line) = message.0 else {
            return;
        };
        let (target, level) = (event.metadata().target(), *event.metadata().level());
        if level == tracing::Level::INFO && target == "jvm::jvm" {
            self.0.java_exceptions.record(&line);
        }
        if level == tracing::Level::WARN {
            self.0.stub_hits.record(&line);
        }
        if level == tracing::Level::INFO
            && target == SVC_STUB_TARGET
            && line == SVC_STUB_EXHAUSTED_MESSAGE
            && !self.0.svc_stub_exhausted.swap(true, Ordering::SeqCst)
            && let Some(write_line) = self.0.on_svc_stub_exhausted.get()
        {
            write_line(&self.0);
        }
    }
}

fn tally_layer<S>(tallies: Arc<Tallies>) -> impl tracing_subscriber::Layer<S>
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    use tracing_subscriber::Layer;
    TallyLayer(tallies).with_filter(
        tracing_subscriber::filter::Targets::new()
            .with_target("jvm::jvm", tracing::Level::INFO)
            .with_target(SVC_STUB_TARGET, tracing::Level::INFO)
            .with_default(tracing::Level::WARN),
    )
}

/// The process subscriber `main` installs: the stderr logger under `stderr_filter` (`RUST_LOG`)
/// plus the tallies. The logger's filter is per-layer, not global, so the tallies still see
/// their events when `RUST_LOG` is unset — a global filter would drop them first.
fn validator_subscriber(stderr_filter: tracing_subscriber::EnvFilter, tallies: Arc<Tallies>) -> impl tracing::Subscriber + Send + Sync + 'static {
    use tracing_subscriber::{Layer, layer::SubscriberExt};
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr).with_filter(stderr_filter))
        .with(tally_layer(tallies))
}

const SCREEN_W: u32 = 240;
const SCREEN_H: u32 = 320;

fn main() {
    // Honor RUST_LOG for debugging (logs to stderr, separate from the JSON result on stdout).
    let tallies = Arc::new(Tallies::default());
    {
        use tracing_subscriber::util::SubscriberInitExt;
        validator_subscriber(tracing_subscriber::EnvFilter::from_default_env(), tallies.clone()).init();
    }

    let args = Args::parse();
    let start = StdInstant::now();

    {
        let filename = args.filename.clone();
        // The exhaustion line carries no `guest_stdout` even under `--guest-stdout`: the guest's
        // buffer lives in `main` and the hook runs mid-run, so this line's schema is the default one.
        let _ = tallies.on_svc_stub_exhausted.set(Box::new(move |tallies: &Tallies| {
            use std::io::Write;
            let line = result_line(&filename, &svc_stub_exhausted_outcome(), tallies, start.elapsed().as_millis());
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{line}");
            let _ = stdout.flush();
        }));
    }

    // Hoisted so main can read what the guest printed after `run` returns; `run`
    // hands the same handle to HeadlessPlatform::write_stdout.
    let guest_out = Arc::new(Mutex::new(Vec::new()));
    let result = run(&args, guest_out.clone());
    let elapsed_ms = start.elapsed().as_millis();

    // Emit a single JSON line for the batch wrapper to parse.
    let json = result_line(&args.filename, &result, &tallies, elapsed_ms);
    // Appended, never interleaved, and only when asked: with the flag absent the
    // line above keeps every pre-existing key in the same order with the same value
    // (`stub_hits`/`svc_stub_slots` are new keys, inserted just before `ms`).
    // Both in-tree parsers read this line with `grep -o` over the WHOLE line
    // (smoke_gate.sh's `"result":"[^"]*"`, lgt_render_probe.sh's `"<key>":[0-9a-z.]*`)
    // plus `tail -1`, so a guest that printed `"result":"PASS"` would otherwise win
    // the match by being later on the line. `json_escape` is what stops that: the
    // quotes become `\"`, so the literal `"result":"` those patterns need never
    // appears inside the payload. That is asserted below, not assumed.
    let json = if args.guest_stdout {
        let (body, truncated) = guest_stdout_field(&guest_out.lock().unwrap(), GUEST_STDOUT_MAX_BYTES);
        format!(
            "{}{}",
            &json[..json.len() - 1],
            format_args!(",\"guest_stdout\":\"{body}\",\"guest_stdout_truncated\":{truncated}}}")
        )
    } else {
        json
    };
    if tallies.svc_stub_exhausted.load(Ordering::SeqCst) && result.verdict() == svc_stub_exhausted_outcome().verdict() {
        eprintln!("wie_validate: result line already written when the SVC stub space ran out; not writing a second one");
    } else {
        println!("{json}");
    }

    std::process::exit(result.exit_code());
}

/// What the line written at SVC stub exhaustion says. `platform` is not known to the layer.
fn svc_stub_exhausted_outcome() -> Outcome {
    fail(
        "unknown",
        format!("{SVC_STUB_EXHAUSTED_MESSAGE} (line written at exhaustion; see stderr)"),
        0,
        0,
        false,
    )
}

/// The one JSON result line. `main` writes it at the end of a run; the SVC-stub-exhaustion hook
/// writes it early, because on that path the process may abort before `main` gets there.
fn result_line(file: &str, result: &Outcome, tallies: &Tallies, elapsed_ms: u128) -> String {
    format!(
        "{{\"file\":{:?},\"platform\":{:?},\"result\":{:?},\"reason\":{:?},\"stop\":{:?},\
         \"input_steps\":{},\"input_steps_total\":{},\
         \"ticks\":{},\"paints\":{},\"content\":{},\
         \"last_frame_content\":{},\
         \"distinct_colors\":{},\"nondominant_pct\":{:.1},\"center_nonuniform_pct\":{:.1},\
         \"last_frame_distinct_colors\":{},\"last_frame_nondominant_pct\":{:.1},\"last_frame_center_nonuniform_pct\":{:.1},\
         \"java_exceptions\":{},\"stub_hits\":{},\"svc_stub_slots\":{},\"ms\":{}}}",
        file,
        result.platform,
        result.verdict(),
        result.reason,
        result.stop,
        result.input_steps,
        result.input_steps_total,
        result.ticks,
        result.paints,
        result.content,
        result.last_frame_content,
        result.distinct_colors,
        result.nondominant_bp as f64 / 100.0,
        result.center_nonuniform_bp as f64 / 100.0,
        result.last_frame_distinct_colors,
        result.last_frame_nondominant_bp as f64 / 100.0,
        result.last_frame_center_nonuniform_bp as f64 / 100.0,
        tallies.java_exceptions.json(),
        tallies.stub_hits.json(),
        svc_stub_slots_json(),
        elapsed_ms
    )
}

struct Outcome {
    platform: String,
    passed: bool,
    /// The run never exercised the axis it was asked about, so neither PASS nor FAIL
    /// is true of it. Takes precedence over `passed` in both the JSON and the exit
    /// code — and `passed` is forced false alongside it, so a later reader of that
    /// field alone cannot mistake this for a pass. See the module header.
    unmeasured: bool,
    reason: String,
    /// Which of the four loop terminators ended the run: `clean exit`, `max-ticks`,
    /// `deadline`, `error`. Reported on EVERY run, not just failing ones — "it ended
    /// early" and "it ran to the end of the schedule" were previously indistinguishable
    /// in the output, which is what let a backstop-truncated run look complete.
    stop: &'static str,
    /// Input steps actually delivered to the guest, out of the scripted total. Both are
    /// 0 without `--inject`. A number, not a boolean, because the round that needed it
    /// needed to write down "28 steps ran" — a flag cannot say that.
    input_steps: u64,
    input_steps_total: u64,
    ticks: u64,
    paints: u64,
    content: bool,
    /// Same predicate as `content`, but scoped to the LAST painted frame instead of
    /// ORed over all of them. Measure-only UNLESS `--expect-last-frame` is passed,
    /// in which case a false value turns a PASS into a FAIL. See the module header
    /// for why the two can disagree and why the expectation lives on the command line.
    last_frame_content: bool,
    // Content-richness metrics (measure-only; do not affect passed). See HeadlessScreen.
    // These are the MAX over every painted frame, so they carry the same structural blind
    // spot `content` does: fetch_max is monotonic, and a later frame cannot pull the number
    // back down. The `last_frame_*` triple below is the same three metrics scoped to the
    // final frame — the pair is what makes "drew something rich, then overpainted with
    // near-nothing" visible. Measure-only as well: NO threshold is defined for them and
    // none is invented here (a number without a threshold cannot fail a run).
    distinct_colors: u64,
    nondominant_bp: u64,
    center_nonuniform_bp: u64,
    last_frame_distinct_colors: u64,
    last_frame_nondominant_bp: u64,
    last_frame_center_nonuniform_bp: u64,
}

impl Outcome {
    fn verdict(&self) -> &'static str {
        if self.unmeasured {
            "UNMEASURED"
        } else if self.passed {
            "PASS"
        } else {
            "FAIL"
        }
    }

    fn exit_code(&self) -> i32 {
        match self.verdict() {
            "PASS" => 0,
            "UNMEASURED" => 2,
            _ => 1,
        }
    }
}

fn run(args: &Args, stdout: Arc<Mutex<Vec<u8>>>) -> Outcome {
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

    let platform = Box::new(HeadlessPlatform {
        screen: screen.clone(),
        fs: MemoryFilesystem::new(),
        db: MemDbRepository::default(),
        stdout: stdout.clone(),
        exited: exited.clone(),
        // `.expect` and not `?`: `run` returns `Outcome`, and these bytes are a compile-time
        // constant, so a failure here means the committed asset is corrupt — a build-wide fault,
        // not a per-game one. `wie-backend`'s own `text_layout.rs` unwraps the same bytes.
        font: Font::try_from_static(include_bytes!("../../../assets/neodgm.ttf")).expect("assets/neodgm.ttf failed to parse"),
    });

    // ── load & route (mirrors wie_cli/src/main.rs) ──────────────────────────
    let buf = match fs::read(&args.filename) {
        Ok(b) => b,
        Err(e) => return fail("unknown", format!("read error: {e}"), 0, 0, false),
    };

    let profile = match args.profile_out.as_deref().map(profile_callback).transpose() {
        Ok(p) => p,
        Err(e) => return fail("unknown", format!("--profile-out: {e}"), 0, 0, false),
    };
    let load = catch_unwind(AssertUnwindSafe(|| build_emulator(platform, &args.filename, buf, profile)));
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

    let (schedule, deadline_secs, input_steps_total) = plan_schedule(args);
    let deadline = Duration::from_secs_f64(deadline_secs.max(1.0));

    // ── drive ───────────────────────────────────────────────────────────────
    let loop_start = StdInstant::now();
    let mut ticks = 0u64;
    let tick_budget = args.frame_hz.map(|hz| FramePacer::budget_for_period_us(1_000_000 / u64::from(hz)));
    let mut run_err: Option<String> = None;
    let mut phase = String::from("boot");
    let mut sched_idx = 0usize;
    let mut input_steps = 0u64;

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
                    // Counted on the DOWN half (one per scripted step) and BEFORE dispatch:
                    // a step that panics the guest was still delivered, and `input_steps` has
                    // to agree with the `panic on input '06_OK'` reason naming step 6.
                    if *down {
                        input_steps += 1;
                    }
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
            match tick_budget {
                Some(budget) => emulator.tick_for(budget)?,
                None => emulator.tick()?,
            }
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
    // Derived from the loop's own exit state rather than set at each `break`: the two budget
    // breaks share one condition, so a flag would just restate `ticks >= args.max_ticks` and
    // could drift from it.
    let stop = stop_cause(run_err.is_some(), exited.load(Ordering::SeqCst), ticks, args.max_ticks);

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
    // classification below is byte-for-byte the existing behaviour; the ONE thing
    // that can still change the verdict afterwards is `--expect-last-frame`, and
    // only in the PASS -> FAIL direction (see just past the last_frame_content
    // assignment). An earlier revision of this comment said the PASS/FAIL logic was
    // byte-for-byte existing, full stop; that stopped being true on 2026-09-06.
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

    // The last frame is what a user actually sees. `content` cannot answer for it: it is
    // an ANY-frame OR, so a run that draws correctly and is then overpainted with a blank
    // screen still reports `content: true`.
    outcome.last_frame_content = screen.last_frame.lock().unwrap().as_deref().is_some_and(has_content);

    // Order is load-bearing and NOT alphabetical: every field that merely RECORDS what
    // happened is filled before the one thing that JUDGES. That covers BOTH richness trios —
    // the `last_frame_*` one just below and the whole-run `max_*` one after it. The gate reads
    // three bools — `args.expect_last_frame`, `outcome.passed` (set by pass()/fail() above) and
    // `outcome.last_frame_content` (set just above) — and no richness field, so today the
    // blocks commute. That is exactly why the order needs writing down: widening the gate to
    // read a richness field would otherwise compile, run, and read a zero, because the fields
    // are Default::default() until the blocks below fill them. Both trios are pinned ahead of
    // the gate by `richness_is_recorded_before_the_gate_judges_test`.
    //
    // Same three richness metrics, scoped to the final frame. Computed ONCE per run, not per
    // painted frame: the last frame is already held in memory, so this is one extra pass over
    // one frame at the end of a run that painted `paints` of them.
    let (lf_colors, lf_nondominant, lf_center) = match screen.last_frame.lock().unwrap().as_deref() {
        Some(frame) => frame_richness(frame, SCREEN_W, SCREEN_H),
        None => (0, 0, 0),
    };
    outcome.last_frame_distinct_colors = lf_colors;
    outcome.last_frame_nondominant_bp = lf_nondominant;
    outcome.last_frame_center_nonuniform_bp = lf_center;

    // The whole-run peaks of those same three metrics. Plain atomic loads off `screen`: nothing
    // between here and the gate writes them, which is why moving them above the gate is a pure
    // reordering — see the ordering comment above.
    outcome.distinct_colors = screen.max_distinct_colors.load(Ordering::SeqCst);
    outcome.nondominant_bp = screen.max_nondominant_bp.load(Ordering::SeqCst);
    outcome.center_nonuniform_bp = screen.max_center_nonuniform_bp.load(Ordering::SeqCst);

    // Same rule as the richness trios: recorded before anything judges. These three are what
    // the gate below reads, so here the order is not merely conventional.
    outcome.stop = stop;
    outcome.input_steps = input_steps;
    outcome.input_steps_total = input_steps_total;

    // `else if`, not a second `if`: a run that injected nothing has no last frame worth
    // gating either, and "last frame blank" would be a narrower, more misleading reason
    // than "the input never ran". UNMEASURED therefore wins over --expect-last-frame.
    if inject_unmeasured(args.inject, outcome.passed, outcome.input_steps, outcome.input_steps_total) {
        outcome.unmeasured = true;
        // Forced false alongside `unmeasured` so no reader of `passed` alone can mistake
        // this for a pass — the same fail-closed reflex as the exit code.
        outcome.passed = false;
        outcome.reason = format!(
            "--inject delivered {}/{} input steps (run ended: {}) — input survival NOT measured (otherwise: {})",
            outcome.input_steps, outcome.input_steps_total, outcome.stop, outcome.reason
        );
    } else if last_frame_gate_fails(args.expect_last_frame, outcome.passed, outcome.last_frame_content) {
        outcome.passed = false;
        outcome.reason = format!("last frame blank, but --expect-last-frame was given (otherwise: {})", outcome.reason);
    }

    outcome
}

/// The `--inject` key script plus any `--shot-every` frames, as a time-sorted schedule,
/// with the run's deadline and the number of keys it will try to deliver. With none of
/// `--inject-keys`/`--keep-timeout`/`--shot-every` given this is the historical
/// schedule unchanged (asserted by `plan_schedule_defaults_unchanged_test`).
fn plan_schedule(args: &Args) -> (Vec<(f64, ScheduledEv)>, f64, u64) {
    let mut schedule: Vec<(f64, ScheduledEv)> = Vec::new();
    let mut deadline_secs = args.timeout as f64;
    let mut input_steps_total = 0u64;
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
        let default: Vec<KeyStep> = script
            .iter()
            .map(|&(key, name)| KeyStep {
                key: Some(key),
                name: name.into(),
                gap: None,
                hold: None,
            })
            .collect();
        let script = args.keys.as_ref().map_or(&default[..], |k| &k.0[..]);
        let script = &script[..args.inject_keys.map_or(script.len(), |n| n.min(script.len()))];
        input_steps_total = script.iter().filter(|s| s.key.is_some()).count() as u64;
        schedule.push((args.boot_secs, ScheduledEv::Shot("00_boot".into())));
        let mut t = args.boot_secs + 0.3;
        for (i, step) in script.iter().enumerate() {
            let label = format!("{:02}_{}", i + 1, step.name);
            let gap = step.gap.unwrap_or(args.action_secs);
            if let Some(kc) = step.key {
                schedule.push((t, ScheduledEv::Key(kc, true, label.clone())));
                schedule.push((t + step.hold.unwrap_or(0.15), ScheduledEv::Key(kc, false, label.clone())));
            }
            schedule.push((t + gap - 0.05, ScheduledEv::Shot(label)));
            t += gap;
        }
        if !args.keep_timeout {
            // Hard cap against runaway — but not on a caller-written script, whose length is the budget asked for.
            deadline_secs = if args.keys.is_some() { t + 1.0 } else { (t + 1.0).min(120.0) };
        }
    }
    if let Some(every) = args.shot_every {
        let mut k = every;
        while k < deadline_secs {
            schedule.push((k, ScheduledEv::Shot(format!("t{k:05.1}"))));
            k += every;
        }
        // Stable sort: same-instant events keep their scripted order.
        schedule.sort_by(|a, b| a.0.total_cmp(&b.0));
    }
    (schedule, deadline_secs, input_steps_total)
}

/// Same writer as the GUI host's `profile_callback` (`src/lib.rs`): leaf-first stacks reversed
/// into root-first folded lines.
fn profile_callback(path: &std::path::Path) -> std::io::Result<ProfileCallback> {
    let writer = Mutex::new(LineWriter::new(File::create(path)?));
    Ok(Box::new(move |batch: Vec<ProfileSample>| {
        let mut writer = writer.lock().unwrap();
        for sample in batch {
            let folded: Vec<String> = sample.stack.iter().rev().map(|pc| format!("0x{pc:x}")).collect();
            let _ = writeln!(writer, "{} {}", folded.join(";"), sample.count);
        }
    }))
}

/// One `--inject` step. `key: None` is `WAIT`; `gap`/`hold` `None` = the defaults.
#[derive(Clone)]
struct KeyStep {
    key: Option<KeyCode>,
    name: String,
    gap: Option<f64>,
    hold: Option<f64>,
}

#[derive(Clone)]
struct KeyScript(Vec<KeyStep>);

/// `--keys`: a readable file is the script; anything else is the script inline.
/// Fail-closed on an unknown name — a typo silently becoming a no-op is a different path.
fn parse_keys(v: &str) -> std::result::Result<KeyScript, String> {
    let text = if std::path::Path::new(v).is_file() {
        std::fs::read_to_string(v).map_err(|e| format!("{v}: {e}"))?
    } else {
        v.to_owned()
    };
    let mut steps = Vec::new();
    for tok in text.lines().flat_map(|l| l.split('#').next().unwrap_or("").split_whitespace()) {
        let mut it = tok.split(':');
        let name = it.next().unwrap_or("");
        let key = match name {
            "WAIT" => None,
            "OK" => Some(KeyCode::OK),
            "UP" => Some(KeyCode::UP),
            "DOWN" => Some(KeyCode::DOWN),
            "LEFT" => Some(KeyCode::LEFT),
            "RIGHT" => Some(KeyCode::RIGHT),
            "LSOFT" => Some(KeyCode::LEFT_SOFT_KEY),
            "RSOFT" => Some(KeyCode::RIGHT_SOFT_KEY),
            "CLR" => Some(KeyCode::CLEAR),
            "STAR" => Some(KeyCode::STAR),
            "HASH" => Some(KeyCode::HASH),
            "NUM0" => Some(KeyCode::NUM0),
            "NUM1" => Some(KeyCode::NUM1),
            "NUM2" => Some(KeyCode::NUM2),
            "NUM3" => Some(KeyCode::NUM3),
            "NUM4" => Some(KeyCode::NUM4),
            "NUM5" => Some(KeyCode::NUM5),
            "NUM6" => Some(KeyCode::NUM6),
            "NUM7" => Some(KeyCode::NUM7),
            "NUM8" => Some(KeyCode::NUM8),
            "NUM9" => Some(KeyCode::NUM9),
            _ => return Err(format!("unknown key {name:?} in {tok:?}")),
        };
        let gap = it.next().map(positive_secs).transpose()?;
        let hold = it.next().map(positive_secs).transpose()?;
        if it.next().is_some() {
            return Err(format!("{tok:?}: expected NAME[:GAP[:HOLD]]"));
        }
        steps.push(KeyStep {
            key,
            name: name.into(),
            gap,
            hold,
        });
    }
    if steps.is_empty() {
        return Err("empty key script".into());
    }
    Ok(KeyScript(steps))
}

fn positive_secs(s: &str) -> std::result::Result<f64, String> {
    match s.parse::<f64>() {
        Ok(v) if v > 0.0 && v.is_finite() => Ok(v),
        _ => Err(format!("expected a positive number of seconds, got {s:?}")),
    }
}

enum ScheduledEv {
    /// key code, is_down, step label
    Key(KeyCode, bool, String),
    /// screenshot with step label
    Shot(String),
}

#[allow(clippy::type_complexity)]
fn build_emulator(
    platform: Box<dyn Platform>,
    filename: &str,
    buf: Vec<u8>,
    profile: Option<ProfileCallback>,
) -> std::result::Result<(Box<dyn Emulator>, String), (String, String)> {
    let options = Options {
        enable_gdbserver: false,
        profile,
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
        let jar_name = base_name(&jar_filename);
        let e = J2MEEmulator::from_jad_jar(platform, buf, jar_name, jar).map_err(|e| ("j2me".to_string(), format!("{e}")))?;
        Ok((Box::new(e), "j2me".into()))
    } else if filename.ends_with("jar") {
        let name = base_name(filename);
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

/// The file name without its directories. Was `path[path.rfind('/').unwrap_or(0) + 1..]`, which on
/// Windows (`C:\…\x.jar`, no `/`) kept the whole path minus its first character — the J2ME boot then
/// panicked on that name — and dropped the first letter of any bare name (`x.jar` → `.jar`).
fn base_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map_or_else(|| path.to_owned(), |n| n.to_string_lossy().into_owned())
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

/// Does `--expect-last-frame` turn this verdict into a FAIL?
///
/// Deliberately ONE-DIRECTIONAL: it can only turn a PASS into a FAIL, never rescue
/// a FAIL. An already-failing run has a more specific reason than "the last frame
/// was blank" (a panic, a hang, the magenta color-key check), and letting the gate
/// touch it would either lose that reason or — if the sign were ever flipped —
/// hand a PASS to a run that crashed. Extracted from `run` so that invariant is
/// asserted by `cargo test --all` rather than by reading the call site.
fn last_frame_gate_fails(expect_last_frame: bool, passed: bool, last_frame_content: bool) -> bool {
    expect_last_frame && passed && !last_frame_content
}

/// Does this `--inject` run have no input verdict to report?
///
/// ONE-DIRECTIONAL for the same reason `last_frame_gate_fails` is: it only demotes a
/// PASS. A FAIL already knows something — the guest errored, panicked, or never drew —
/// and relabelling that as "not measured" would lose a real finding. Zero steps plus a
/// PASS is the only combination where the verdict is about an axis the run never touched.
///
/// The count is what makes this work, and the reason it is a count and not a
/// `saw_any_input` flag: `10/27` vs `27/27` is the number the reporting round has to write down.
/// Extracted from `run` so `cargo test --all` asserts the invariant instead of a reader
/// of the call site — the failure that matters here is folding a shortfall back into PASS, which
/// leaves every name and field in place and only changes the answer.
///
/// The comparison is `<`, not `== 0`, and the corpus is why. Measured 2026-09-23 over the 175
/// titles that had ever reported `survived input sequence` (loadavg 77-98, i.e. the condition
/// most likely to truncate a run): the three UNMEASURED verdicts were all `0/27` **under
/// `clean exit`**, and the only partial deliveries — `10/27` and `2/27` — were `clean exit`
/// too, and were reported as PASS. So the old `== 0` line called the same terminator, with
/// the same "only part of the script ran" story, unmeasured at zero and a pass at ten. That
/// is the inconsistency, not a missing case: partial injection IS partial measurement however
/// the run ended. Widening costs exactly those two titles, PASS -> UNMEASURED, never -> FAIL.
/// No partial ever arrived carrying the `survived input sequence` claim, so this buys nothing
/// on today's corpus; it is kept because the backstop shape that produced the original defect
/// truncates by tick count, which no title here is currently fast enough to hit mid-script.
fn inject_unmeasured(inject: bool, passed: bool, input_steps: u64, input_steps_total: u64) -> bool {
    inject && passed && input_steps < input_steps_total
}

/// Which of the drive loop's four terminators ended the run.
///
/// Precedence is the loop's own: an error breaks out immediately, a clean guest exit ends
/// the `while`, and the two budget breaks share one condition — so when both budgets are
/// spent the TICK backstop is named, because it is the one a caller can misread as a
/// completed run. `deadline` is the fall-through: under `--inject` that deadline is derived
/// from the schedule, so it means "the script ran out", which `input_steps` disambiguates.
fn stop_cause(errored: bool, exited: bool, ticks: u64, max_ticks: u64) -> &'static str {
    if errored {
        "error"
    } else if exited {
        "clean exit"
    } else if ticks >= max_ticks {
        "max-ticks"
    } else {
        "deadline"
    }
}

/// `stop` starts as `error` in both constructors, and `run` overwrites it for every run
/// that reached the drive loop. The only outcomes that keep this value are the three
/// pre-loop `fail()` returns — a read error, a load error, a load panic — for which
/// `error` is the right answer.
const STOP_BEFORE_LOOP: &str = "error";

fn pass(platform: &str, reason: String, ticks: u64, paints: u64, content: bool) -> Outcome {
    Outcome {
        platform: platform.to_string(),
        passed: true,
        unmeasured: false,
        stop: STOP_BEFORE_LOOP,
        input_steps: 0,
        input_steps_total: 0,
        reason,
        ticks,
        paints,
        content,
        last_frame_content: false,
        distinct_colors: 0,
        nondominant_bp: 0,
        center_nonuniform_bp: 0,
        last_frame_distinct_colors: 0,
        last_frame_nondominant_bp: 0,
        last_frame_center_nonuniform_bp: 0,
    }
}

fn fail(platform: &str, reason: String, ticks: u64, paints: u64, content: bool) -> Outcome {
    Outcome {
        platform: platform.to_string(),
        passed: false,
        unmeasured: false,
        stop: STOP_BEFORE_LOOP,
        input_steps: 0,
        input_steps_total: 0,
        reason,
        ticks,
        paints,
        content,
        last_frame_content: false,
        distinct_colors: 0,
        nondominant_bp: 0,
        center_nonuniform_bp: 0,
        last_frame_distinct_colors: 0,
        last_frame_nondominant_bp: 0,
        last_frame_center_nonuniform_bp: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GUEST_STDOUT_MAX_BYTES, HeadlessPlatform, HeadlessScreen, RICHNESS_COLOR_CAP, SCREEN_H, SCREEN_W, base_name, frame_richness,
        guest_stdout_field, has_content, inject_unmeasured, json_escape, last_frame_gate_fails, stop_cause,
    };
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64},
    };
    use test_utils::MemoryFilesystem;
    use wie_backend::Platform;

    use super::{JavaExceptionTally, StubHitTally, Tallies, tally_layer, validator_subscriber};

    /// Runs `body` with only the tally layer installed, and returns the tallies.
    fn with_tally(body: impl FnOnce()) -> Arc<Tallies> {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        let tally = Arc::new(Tallies::default());
        let guard = tracing_subscriber::registry().with(tally_layer(tally.clone())).set_default();
        body();
        drop(guard);
        tally
    }

    /// The field exists to replace hand-counting `jvm::jvm: throwing java exception` lines,
    /// so the test drives the REAL `jvm::Jvm::exception` (crates.io `jvm`, not ours) through a
    /// bare JVM rather than emitting a look-alike event. If upstream rewords, re-targets or
    /// re-levels that line, this goes red instead of the JSON silently reporting 0.
    ///
    /// Note what the tally is fed by: the subscriber is the thread-local default set by
    /// `with_tally`, and `run_jvm_test` ticks the JVM on this same thread.
    #[test]
    fn java_exception_tally_sees_a_real_jvm_exception_test() {
        let tally = with_tally(|| {
            test_utils::run_jvm_test(Box::new([]), |jvm| async move {
                let _ = jvm.exception("java/io/IOException", "first one").await;
                let _ = jvm.exception("java/lang/IllegalStateException", "second").await;
                Ok(())
            })
            .unwrap();
        });
        assert_eq!(tally.java_exceptions.count.load(core::sync::atomic::Ordering::SeqCst), 2);
        assert_eq!(
            tally.java_exceptions.json(),
            r#"{"count":2,"first":"java/io/IOException: first one","first_truncated":false}"#
        );
    }

    /// The tests around this one prove the layer counts; this pins that `main` actually installs it
    /// and prints it. Without it, dropping the subscriber from `main` would leave every test green and
    /// the field reading `{"count":0,...}` on every run — indistinguishable from "nothing raised".
    #[test]
    fn java_exception_layer_is_installed_and_printed_by_main_test() {
        let src = include_str!("wie_validate.rs");
        let main_body = &src[src.find(concat!("fn ", "main() {")).unwrap()..src.find(concat!("struct ", "Outcome {")).unwrap()];
        for needle in [
            concat!(
                "validator_subscriber(tracing_subscriber::EnvFilter::from_default_env()",
                ", tallies.clone())"
            ),
            concat!("tallies.java_exceptions", ".json(),"),
            concat!("\\\"java_exceptions", "\\\":{},"),
            concat!("tallies.stub_hits", ".json(),"),
            concat!("\\\"stub_hits", "\\\":{},"),
            concat!("svc_stub_slots", "_json(),"),
            concat!("\\\"svc_stub_slots", "\\\":{},"),
        ] {
            assert_eq!(
                main_body.matches(needle).count(),
                1,
                "main() no longer wires the exception tally: {needle}"
            );
        }
    }

    /// `main`'s whole subscriber, with the stderr filter `RUST_LOG` unset gives. The tally must still
    /// count: turning the logger's per-layer filter back into a global `EnvFilter` drops `jvm::jvm`
    /// INFO before any layer sees it, and every other test here stays green on that regression.
    #[test]
    fn java_exception_tally_counts_under_main_subscriber_with_rust_log_unset_test() {
        use tracing_subscriber::util::SubscriberInitExt;
        let tally = Arc::new(Tallies::default());
        let guard = validator_subscriber(tracing_subscriber::EnvFilter::new(""), tally.clone()).set_default();
        test_utils::run_jvm_test(Box::new([]), |jvm| async move {
            let _ = jvm.exception("java/io/IOException", "unset").await;
            Ok(())
        })
        .unwrap();
        drop(guard);
        assert_eq!(tally.java_exceptions.count.load(core::sync::atomic::Ordering::SeqCst), 1);
    }

    /// The control: a JVM that raises nothing reports 0 and `null`, not a stale or invented value.
    #[test]
    fn java_exception_tally_is_zero_when_nothing_is_raised_test() {
        let tally = with_tally(|| {
            test_utils::run_jvm_test(Box::new([]), |jvm| async move {
                let _ = jvm.new_class("java/lang/Object", "()V", ()).await?;
                Ok(())
            })
            .unwrap();
        });
        assert_eq!(tally.java_exceptions.json(), r#"{"count":0,"first":null,"first_truncated":false}"#);
    }

    /// Only the `Jvm::exception` line counts, and `first` is escaped + capped like guest stdout —
    /// the message can carry guest text, and this line is grep-parsed by in-tree callers.
    #[test]
    fn java_exception_tally_filters_escapes_and_caps_test() {
        let tally = JavaExceptionTally::default();
        tally.record("Instantiate java/lang/Object");
        tally.record("No such method: a.b:()V");
        assert_eq!(tally.json(), r#"{"count":0,"first":null,"first_truncated":false}"#);

        tally.record(r#"throwing java exception: java/io/IOException "result":"PASS""#);
        tally.record("throwing java exception: java/lang/Error later");
        assert_eq!(
            tally.json(),
            r#"{"count":2,"first":"java/io/IOException: \"result\":\"PASS\"","first_truncated":false}"#
        );
        assert!(!tally.json().contains(r#""result":""#));

        let long = JavaExceptionTally::default();
        long.record(&format!("throwing java exception: java/lang/Error {}", "x".repeat(1000)));
        assert!(long.json().ends_with(r#""first_truncated":true}"#));
        assert!(long.json().len() < 400);
    }

    /// Only WARN `stub …` lines count — any target — and `first` is the top 5 by hits, escaped.
    /// The `debug!("stub …")` constructors and non-`stub` warnings stay out.
    #[test]
    fn stub_hit_tally_counts_warn_stub_lines_only_test() {
        let tally = with_tally(|| {
            tracing::warn!(target: "wie_wipi_java::x", "stub wec.SYSTheme::saveItem({:?})", 1);
            tracing::warn!("stub wec.SYSTheme::saveItem(2)");
            tracing::warn!("stub unk7(0x1, 0x2)");
            tracing::warn!("stub unk2");
            tracing::warn!("stub unk3-12");
            tracing::warn!("stub a\"b(1)");
            tracing::warn!("stub z()");
            tracing::debug!("stub org.kwis.msp.lwc.Component::<init>()");
            tracing::error!("stub not-a-warning()");
            tracing::warn!("unk4(0x0, 0x0, 0x0, 0x0)");
        });
        assert_eq!(
            tally.stub_hits.json(),
            r#"{"count":7,"distinct":6,"first":[{"name":"wec.SYSTheme::saveItem","count":2},{"name":"a\"b","count":1},{"name":"unk2","count":1},{"name":"unk3-12","count":1},{"name":"unk7","count":1}]}"#
        );
        assert_eq!(StubHitTally::default().json(), r#"{"count":0,"distinct":0,"first":[]}"#);
    }

    /// The exhaustion event (only from the core's target) makes the hook run exactly once, however many times the runtime retries
    /// (LGT recurses on it until the stack overflows), and records the slots as full.
    #[test]
    fn svc_stub_exhaustion_runs_the_line_hook_once_test() {
        use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
        let tally = Arc::new(Tallies::default());
        let calls = Arc::new(AtomicU64::new(0));
        let hook_calls = calls.clone();
        let _ = tally.on_svc_stub_exhausted.set(Box::new(move |_: &Tallies| {
            hook_calls.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
        }));
        let guard = tracing_subscriber::registry().with(tally_layer(tally.clone())).set_default();
        for _ in 0..3 {
            tracing::info!(target: "wie_core_arm::core", "SVC stub space exhausted");
            tracing::info!(target: "elsewhere", "SVC stub space exhausted");
        }
        drop(guard);
        assert_eq!(calls.load(core::sync::atomic::Ordering::SeqCst), 1);
        assert!(tally.svc_stub_exhausted.load(core::sync::atomic::Ordering::SeqCst));
    }

    /// `HeadlessPlatform::font()` shipped for two months as `unimplemented!()`, so every guest that
    /// drew a string panicked the *validator* and `classify.sh` recorded that as the game's fault —
    /// the largest failure signature in `game_lab/broken/`. Nothing in this workspace caught it:
    /// `cargo test --all` never reached `font()`, and the AGENTS.md runner block was green
    /// throughout because `drawString` appeared in 0 of the 6 committed fixtures.
    ///
    /// `test_data/text_j2me.jar` closed that hole for the *runner*, but the runner is local-only —
    /// it runs from a developer's shell and, once a week, from `doc-liveness.yml`. This test is the
    /// per-PR half: `cargo test --all` runs it on all six matrix legs, so re-breaking this one
    /// method reddens the next PR instead of waiting for the weekly job.
    ///
    /// **It guards one method, not the text path.** A regression anywhere else between
    /// `Graphics.drawString` and the screen is invisible here — that breadth is what the fixture
    /// and the runner line buy, and this test deliberately does not duplicate them.
    ///
    /// The assertion goes *through* `text_layout`, which is the path a guest's `drawString`
    /// actually takes, rather than stopping at "the call returned": a font that is present but
    /// carries no usable metrics measures every string at 0, and that would pass a
    /// did-not-panic check while still drawing nothing.
    #[test]
    fn headless_platform_font_measures_text_test() {
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
        let platform = HeadlessPlatform {
            screen,
            fs: MemoryFilesystem::new(),
            db: Default::default(),
            stdout: Arc::new(Mutex::new(Vec::new())),
            exited: Arc::new(AtomicBool::new(false)),
            font: wie_backend::Font::try_from_static(include_bytes!("../../../assets/neodgm.ttf")).expect("assets/neodgm.ttf failed to parse"),
        };

        // Through the trait, not the field: the defect was in the `Platform` impl.
        let font = Platform::font(&platform);
        // Not vacuous: the same call on an empty string measures 0, so `> 0` discriminates.
        assert_eq!(wie_backend::text_layout::minimum_width(font, "", 12.0), 0);
        for text in ["wie", "가"] {
            let width = wie_backend::text_layout::minimum_width(font, text, 12.0);
            assert!(width > 0, "minimum_width({text:?}) = {width} — the font carries no usable metrics");
        }
    }

    /// The payload is the one part of the JSON line the guest controls, and both
    /// in-tree parsers `grep -o` over the whole line and take `tail -1`. So a guest
    /// that prints a fake verdict must not be able to produce the byte sequence
    /// those patterns look for. Escaping is what prevents it — assert that here
    /// rather than trusting the reasoning in the comment at the call site.
    #[test]
    fn escaped_guest_output_cannot_forge_a_field_the_parsers_match_test() {
        let hostile = b"\"result\":\"PASS\" \"paints\":999";
        let (body, truncated) = guest_stdout_field(hostile, GUEST_STDOUT_MAX_BYTES);
        assert!(!truncated);
        // smoke_gate.sh: grep -o '"result":"[^"]*"'
        assert!(!body.contains("\"result\":\""), "forged verdict survived escaping: {body}");
        // lgt_render_probe.sh: field() greps "\"<name>\":[0-9a-z.]*"
        assert!(!body.contains("\"paints\":"), "forged metric survived escaping: {body}");
        // The text is still readable — escaping, not stripping.
        assert!(body.contains("result"));
    }

    #[test]
    fn json_escape_covers_the_string_grammar_test() {
        assert_eq!(json_escape("a\"b\\c"), "a\\\"b\\\\c");
        assert_eq!(json_escape("l1\nl2\r\tx"), "l1\\nl2\\r\\tx");
        // A control byte must become \u00XX, not Rust Debug's \u{1} (invalid JSON).
        assert_eq!(json_escape("\u{1}"), "\\u0001");
        // Non-ASCII passes through as UTF-8; JSON does not require escaping it.
        assert_eq!(json_escape("한"), "한");
    }

    /// The name handed to the emulators is the file name alone, on every host.
    #[test]
    fn base_name_strips_directories_and_keeps_bare_names_whole_test() {
        assert_eq!(base_name("test_data/draw_j2me.jar"), "draw_j2me.jar");
        assert_eq!(base_name("draw_j2me.jar"), "draw_j2me.jar");
        #[cfg(windows)]
        assert_eq!(base_name(r"C:\Temp\x\draw_j2me.jar"), "draw_j2me.jar");
    }

    #[test]
    fn guest_stdout_is_capped_and_says_so_test() {
        let (body, truncated) = guest_stdout_field(&vec![b'x'; GUEST_STDOUT_MAX_BYTES + 1], GUEST_STDOUT_MAX_BYTES);
        assert_eq!(body.len(), GUEST_STDOUT_MAX_BYTES);
        assert!(truncated, "over-cap input must report truncation");

        let (body, truncated) = guest_stdout_field(b"short", GUEST_STDOUT_MAX_BYTES);
        assert_eq!(body, "short");
        assert!(!truncated);

        // Cutting mid-character must not panic or emit a partial code point: the
        // cap is in bytes and the payload is arbitrary guest output.
        let (body, truncated) = guest_stdout_field("한글".as_bytes(), 4);
        assert!(truncated);
        assert_eq!(body, "한");
    }

    #[test]
    fn last_frame_gate_only_turns_pass_into_fail_test() {
        // Off by default: every verdict survives untouched, which is what keeps the
        // existing fixture judgements (helloworld_* PASS on a blank final frame)
        // from flipping. This row is the whole reason the flag is opt-in.
        for passed in [true, false] {
            for last in [true, false] {
                assert!(!last_frame_gate_fails(false, passed, last), "flag off must never fire: {passed} {last}");
            }
        }
        // Declared, and the last frame is blank -> fail. Measured equivalent:
        // keydraw_lgt.zip --inject with the is_clet_card normalization reverted.
        assert!(last_frame_gate_fails(true, true, false));
        // Declared and satisfied -> untouched.
        assert!(!last_frame_gate_fails(true, true, true));
        // Already failing -> the gate does NOT speak, in either direction. A sign
        // flip here would hand a PASS to a crashed run, so it is asserted, not
        // left to the reader of the call site.
        assert!(!last_frame_gate_fails(true, false, false));
        assert!(!last_frame_gate_fails(true, false, true));
    }

    /// A `--inject` run that delivered ZERO keys has no input verdict, and used to report one:
    /// `PASS ... survived input sequence` over 0 delivered steps and 0 `--shotdir` frames,
    /// because the `--max-ticks` backstop ended the run before the wall-clock schedule was due.
    ///
    /// The mutation this is aimed at is the quiet one: keep the counter, keep the field, keep
    /// the name — and fold `0` back into PASS. Every row below fails under that edit, and the
    /// two `passed: false` rows pin the other direction, where a sign flip would relabel a real
    /// crash as "not measured" and lose it.
    #[test]
    fn zero_injected_steps_is_not_a_pass_test() {
        // Without --inject there is no input axis to be unmeasured about, at any count.
        for passed in [true, false] {
            for steps in [0, 1, 27] {
                assert!(!inject_unmeasured(false, passed, steps, 27), "flag off must never fire: {passed} {steps}");
            }
        }
        // The defect, exactly: injection asked for, nothing delivered, and it "passed".
        assert!(inject_unmeasured(true, true, 0, 27));
        // A SHORTFALL is the same claim at a different size — one key or twenty-six, the run
        // measured part of the script and the verdict may only speak about that part. These two
        // rows are the widening; under the old `== 0` predicate both were PASS.
        assert!(inject_unmeasured(true, true, 1, 27));
        assert!(inject_unmeasured(true, true, 26, 27));
        // The whole script ran -> the axis has a real verdict and this gate is silent.
        assert!(!inject_unmeasured(true, true, 27, 27));
        // Already failing -> silent in BOTH directions. A FAIL at 0 steps is a boot-time crash,
        // which is a real finding; demoting it to "not measured" would throw that away.
        assert!(!inject_unmeasured(true, false, 0, 27));
        assert!(!inject_unmeasured(true, false, 27, 27));
        // Total 0 cannot be a shortfall. `>` here would fire on every non-injecting shape that
        // still reaches the gate, which is the sign error a `<`/`!=` swap would introduce.
        assert!(!inject_unmeasured(true, true, 0, 0));
    }

    /// The gate must be handed the DELIVERED count, not the scripted total.
    ///
    /// Measured, not hypothetical: swapping `outcome.input_steps` for
    /// `outcome.input_steps_total` at that one call site restores the whole defect — the
    /// binary printed `input_steps: 0` and `result: PASS`, rc 0 — and the predicate test
    /// above stayed green, because the predicate is still perfect and only its ARGUMENT is
    /// wrong. `run` needs a real emulator, so no behavioural test in this file can reach the
    /// call site; this reads the source instead, exactly as
    /// `richness_is_recorded_before_the_gate_judges_test` does and for the same reason.
    ///
    /// **Two lines produce that argument, and pinning only the call site left the other one
    /// open.** Measured 2026-09-23, by a gate② reviewer and reproduced here: putting the same
    /// swap on the RECORD line three lines above the gate — `outcome.input_steps =
    /// input_steps_total;` — left this file at `15 passed; 0 failed`, and the effect is the
    /// original defect restored *and worse*: the gate eats 27 and can never fire, while the
    /// JSON prints `input_steps: 27/27` on a run that delivered none, so a human reading it
    /// sees "it all ran". Hence the second assertion.
    ///
    /// **Scope, stated rather than implied**: these two assertions pin the two lines that
    /// hand the gate its argument. They do NOT pin how the local `input_steps` is counted —
    /// its `= 0u64` init and its `+= 1` in the dispatch loop are both still free to lie
    /// (checked: mutating the init to `input_steps_total` is green under both assertions).
    /// That half is held by `zero_injected_steps_is_not_a_pass_test`'s predicate rows and by
    /// the two `--inject` fixtures in the AGENTS.md runner block, not by string matching.
    ///
    /// Split with `concat!` so this test's own source is not a second match — the count
    /// assertions are what make that safe.
    #[test]
    fn the_gate_is_handed_the_delivered_count_test() {
        let src = include_str!("wie_validate.rs");
        let call = concat!(
            "inject_unmeasured(args.inject, outcome.passed, outcome.input",
            "_steps, outcome.input_steps_total)"
        );
        assert_eq!(
            src.matches(call).count(),
            1,
            "the gate call site is not unique (or no longer reads `outcome.input_steps`) — \
             a gate fed `input_steps_total` never fires: that field is 27 on every --inject run"
        );
        let record = concat!("outcome.input_steps = input", "_steps;");
        assert_eq!(
            src.matches(record).count(),
            1,
            "the delivered count is no longer what feeds the gate — the record line three \
             lines above the gate must assign `input_steps`, not `input_steps_total`"
        );
    }

    /// The three verdicts must map onto three distinct exit codes, and `unmeasured` must beat
    /// `passed` in both. Asserted rather than read off the two `match`es because they are the
    /// whole of the tool's machine-readable contract: a caller keys on one or the other.
    #[test]
    fn unmeasured_outranks_passed_in_both_readings_test() {
        let mut o = super::fail("lgt", String::new(), 0, 0, false);
        assert_eq!((o.verdict(), o.exit_code()), ("FAIL", 1));
        o.passed = true;
        assert_eq!((o.verdict(), o.exit_code()), ("PASS", 0));
        // `run` forces `passed` false alongside this; set it true here anyway, because the
        // precedence is what is being pinned, not run's housekeeping.
        o.unmeasured = true;
        assert_eq!((o.verdict(), o.exit_code()), ("UNMEASURED", 2));
        // Not the string every in-tree caller greps for — the second fail-closed axis.
        assert_ne!(o.verdict(), "PASS");
    }

    /// `stop` answers "why did the run end", and the answer a caller can misread is
    /// `max-ticks`: it looks like a completed run in every other field.
    #[test]
    fn stop_cause_names_the_terminator_test() {
        // Error outranks everything — it is the only one that says the guest misbehaved.
        assert_eq!(stop_cause(true, true, 50, 50), "error");
        assert_eq!(stop_cause(true, false, 0, 50), "error");
        // A clean guest exit outranks the budgets: the loop's `while` condition ends first.
        assert_eq!(stop_cause(false, true, 50, 50), "clean exit");
        // Budget breaks share one condition in the loop, so when both are spent the tick
        // backstop is named — it is the one that silently truncates a wall-clock schedule.
        assert_eq!(stop_cause(false, false, 50, 50), "max-ticks");
        assert_eq!(stop_cause(false, false, 51, 50), "max-ticks");
        // Under budget and still stopped: the wall-clock deadline ran out.
        assert_eq!(stop_cause(false, false, 49, 50), "deadline");
    }

    /// Both richness trios — `last_frame_*` and the whole-run `max_*` — must be filled BEFORE
    /// `last_frame_gate_fails` is consulted.
    ///
    /// This reads the source rather than the behaviour on purpose, and the reason is the
    /// whole point of the lock: the gate takes three bools and no richness field, so today
    /// the blocks commute and **no behavioural test can tell the orders apart**. A
    /// behavioural assertion here would pass in both orders — it would look like a lock and
    /// hold nothing. What is actually being defended is the next edit: widening the gate to
    /// read a richness field compiles and runs in either order, and in the wrong one it reads
    /// a `Default::default()` zero instead of the measured value.
    ///
    /// The needles are split with `concat!` so this test's own source does not contain them —
    /// otherwise it would match itself. The count assertions are what make that safe: if a
    /// needle ever appears twice, this fails loudly instead of comparing the wrong position.
    #[test]
    fn richness_is_recorded_before_the_gate_judges_test() {
        let src = include_str!("wie_validate.rs");
        // One needle per trio. Pinning only the last-written one would leave the other free to
        // drift below the gate while this still passed — which is exactly the hole that let a
        // widened gate read a zero under a green suite.
        let records = [
            concat!("outcome.last_frame_center", "_nonuniform_bp = lf_center;"),
            concat!(
                "outcome.center_nonuniform_bp = screen.max_center",
                "_nonuniform_bp.load(Ordering::SeqCst);"
            ),
        ];
        let judge = concat!("if last_frame_gate", "_fails(args.expect_last_frame,");
        assert_eq!(
            src.matches(judge).count(),
            1,
            "gate call is not unique — the position below would be arbitrary"
        );
        for record in records {
            assert_eq!(
                src.matches(record).count(),
                1,
                "richness assignment is not unique — the position below would be arbitrary: {record}"
            );
            assert!(
                src.find(record).unwrap() < src.find(judge).unwrap(),
                "the gate is consulted before the richness fields are filled — see the ordering comment in run(): {record}"
            );
        }
    }

    #[test]
    fn has_content_is_the_two_distinct_values_test() {
        assert!(!has_content(&[]));
        assert!(!has_content(&[0x000000]));
        assert!(!has_content(&[0x112233; 64])); // uniform, any color — still blank
        assert!(has_content(&[0x000000, 0x000001]));
    }

    #[test]
    fn the_two_axes_disagree_exactly_on_overpaint() {
        // The shape this axis exists for: good frame, then a blank one on top.
        let good = vec![0x000000, 0xFFFFFF];
        let blank = vec![0x000000, 0x000000];

        // `content` is an OR over the run …
        let any_frame = [&good, &blank].iter().any(|f| has_content(f));
        // … `last_frame_content` looks only at what is left on screen.
        let last_frame = has_content(&blank);

        assert!(any_frame, "any-frame axis must still see the good frame");
        assert!(!last_frame, "last-frame axis must see the overpaint");
        assert_ne!(any_frame, last_frame, "this disagreement IS the blind spot being closed");

        // And they must AGREE when nothing overpaints — otherwise the new axis would
        // just be noise on every healthy run.
        assert_eq!([&good, &good].iter().any(|f| has_content(f)), has_content(&good));
    }

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

    fn plan(argv: &[&str]) -> (Vec<(f64, super::ScheduledEv)>, f64, u64) {
        use clap::Parser;
        let args = super::Args::try_parse_from(["wie_validate"].iter().chain(argv).chain(&["g.zip"])).unwrap();
        super::plan_schedule(&args)
    }

    fn shots(schedule: &[(f64, super::ScheduledEv)]) -> Vec<String> {
        schedule
            .iter()
            .filter_map(|(_, e)| match e {
                super::ScheduledEv::Shot(l) => Some(l.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn plan_schedule_defaults_unchanged_test() {
        // No --inject: nothing scheduled, deadline = --timeout.
        let (s, d, n) = plan(&[]);
        assert!(s.is_empty());
        assert_eq!((d, n), (20.0, 0));
        // --inject alone: 27 keys (down+up+shot each) + boot shot, schedule-derived 20.0 s.
        let (s, d, n) = plan(&["--inject"]);
        assert_eq!((s.len(), n), (27 * 3 + 1, 27));
        assert!((d - 20.0).abs() < 1e-9, "deadline {d}");
    }

    #[test]
    fn plan_schedule_new_flags_test() {
        // One key, budget pinned to --timeout.
        let (s, d, n) = plan(&["--inject", "--inject-keys", "1", "--keep-timeout", "--timeout", "60"]);
        assert_eq!((s.len(), d, n), (1 + 3, 60.0, 1));
        // Zero keys on the same budget is the unkeyed baseline.
        let (s, d, n) = plan(&["--inject", "--inject-keys", "0", "--keep-timeout", "--timeout", "60"]);
        assert_eq!((s.len(), d, n), (1, 60.0, 0));
        // More than the script holds clamps rather than panics.
        assert_eq!(plan(&["--inject", "--inject-keys", "99"]).2, 27);
        // Fixed-time shots, time-sorted in among the key events.
        let (s, _, _) = plan(&[
            "--inject",
            "--inject-keys",
            "1",
            "--keep-timeout",
            "--timeout",
            "12",
            "--shotdir",
            "x",
            "--shot-every",
            "5",
        ]);
        assert_eq!(shots(&s), ["00_boot", "01_OK", "t005.0", "t010.0"].map(String::from));
        assert!(s.windows(2).all(|w| w[0].0 <= w[1].0), "schedule not time-sorted");
        // Without --inject, shots run to --timeout.
        assert_eq!(shots(&plan(&["--shotdir", "x", "--shot-every", "10"]).0), ["t010.0"].map(String::from));
    }

    fn keys(schedule: &[(f64, super::ScheduledEv)]) -> Vec<(f64, bool, String)> {
        schedule
            .iter()
            .filter_map(|(t, e)| match e {
                super::ScheduledEv::Key(_, d, l) => Some((*t, *d, l.clone())),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn keys_flag_replaces_the_script_test() {
        // Ignoring --keys would schedule the 27 defaults — every assertion here would go red.
        let (s, d, n) = plan(&["--inject", "--keys", "UP:2 WAIT:3 # comment\nNUM5:1:0.5"]);
        assert_eq!(n, 2, "WAIT is not an input step");
        assert_eq!(shots(&s), ["00_boot", "01_UP", "02_WAIT", "03_NUM5"].map(String::from));
        let k = keys(&s);
        assert_eq!(
            k.iter().map(|x| (x.1, x.2.as_str())).collect::<Vec<_>>(),
            [(true, "01_UP"), (false, "01_UP"), (true, "03_NUM5"), (false, "03_NUM5")]
        );
        // boot 2.5 + 0.3, then gaps 2 + 3 before NUM5, held 0.5; deadline = end + 1.
        assert!((k[2].0 - 7.8).abs() < 1e-9 && (k[3].0 - 8.3).abs() < 1e-9, "{k:?}");
        assert!((d - 9.8).abs() < 1e-9, "deadline {d}");
        // A written script is its own budget: no 120 s cap. The default path keeps it.
        assert!((plan(&["--inject", "--keys", "OK:200"]).1 - 203.8).abs() < 1e-9);
        assert_eq!(plan(&["--inject", "--action-secs", "10"]).1, 120.0);
        // --inject-keys still takes a prefix.
        assert_eq!(plan(&["--inject", "--keys", "OK UP DOWN", "--inject-keys", "2"]).2, 2);
    }

    #[test]
    fn keys_flag_spelling_the_defaults_is_the_default_schedule_test() {
        let names = "OK OK LSOFT NUM5 DOWN OK DOWN OK UP OK LEFT OK RIGHT OK NUM5 LSOFT RSOFT DOWN DOWN OK UP OK STAR HASH NUM1 OK OK";
        let (a, _, an) = plan(&["--inject"]);
        let (b, _, bn) = plan(&["--inject", "--keys", names]);
        assert_eq!((keys(&a), shots(&a), an), (keys(&b), shots(&b), bn));
    }

    #[test]
    fn keys_flag_rejects_bad_scripts_and_reads_files_test() {
        use clap::Parser;
        for bad in ["OKK", "OK:0", "OK:1:2:3", "", "# only a comment"] {
            assert!(
                super::Args::try_parse_from(["wie_validate", "--inject", "--keys", bad, "g"]).is_err(),
                "{bad:?} should be rejected"
            );
        }
        assert!(
            super::Args::try_parse_from(["wie_validate", "--keys", "OK", "g"]).is_err(),
            "--keys requires --inject"
        );
        let example = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/keys/battlemonster-village.keys");
        let (s, _, n) = plan(&["--inject", "--keys", example]);
        assert_eq!(n, 97);
        assert_eq!(shots(&s)[95], "95_OK");
    }

    #[test]
    fn new_flags_require_their_partner_test() {
        use clap::Parser;
        for argv in [
            &["wie_validate", "--inject-keys", "1", "g"][..],
            &["wie_validate", "--keep-timeout", "g"],
            &["wie_validate", "--shot-every", "5", "g"],
            &["wie_validate", "--shotdir", "x", "--shot-every", "0", "g"],
        ] {
            assert!(super::Args::try_parse_from(argv).is_err(), "{argv:?} should be rejected");
        }
    }
}
