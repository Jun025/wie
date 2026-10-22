//! The engine's pacing, measured end to end on a committed game loop — the guard against the
//! two speed regressions this fork shipped silently (#291: a KTF clet 35 -> 17fps; #338: an LGT
//! title 20 -> 12fps). Both were found by a player first, and #291's arrived with an upstream
//! base swap: the unit tests beside each fix lock one function each, and a sync that rewrites
//! the path around them can keep those tests green. This one watches the outcome instead.
//!
//! The guest (`scripts/make-pace-fixture.mjs`) runs #338's loop: repaint, spin on
//! `Thread.yield` until its paint lands, sleep to a 50ms deadline. The clock is a
//! `TestClock::stepping(1)` — it moves 1ms per read — so the numbers below count the engine's
//! own clock reads and do not move with host load; the same run is the same on any CI leg.
//!
//! What each assertion stands for, and the shape that turns it red (mutation log: the round's
//! report, `docs/report/`):
//! - frame period vs the 50ms the guest asks for — any added wait in the loop;
//! - paints that started in a later host tick than their repaint — #338's tick-end delivery;
//! - repaint -> paint latency — a `getNextEvent` that no longer looks at the queue while it waits
//!   (#338's second half). A deadline loop absorbs that wait inside its 50ms, so the period alone
//!   does not see it;
//! - guest sleep lateness — a wake held past its deadline (rounded to a host frame, say);
//! - GCs per paint — #338's full collection on every paint.
//!
//! Not covered here: WIPI `MC_knlSetTimer` (a J2ME guest has none) — #291's fixed 16ms wait is
//! `timer_due_mid_slice_fires_without_waiting_for_the_whole_slice` in wie-midp.

use std::sync::atomic::Ordering;

use test_utils::{TestClock, TestPlatform};
use wie_backend::{Emulator, Event, extract_zip};
use wie_j2me::J2MEEmulator;
use wie_util::Result;

const PERIOD_MS: u64 = 50; // scripts/make-pace-fixture.mjs PERIOD_MS
const WARMUP_MS: u64 = 2_000;
const WINDOW_MS: u64 = 10_000;

/// A `Pacing::summary_json` value; a percentile over no samples (`null`) reads as `u64::MAX`, so
/// "nothing was measured" fails a bound instead of passing it.
fn field(json: &str, key: &str) -> u64 {
    let tail = &json[json.find(&format!("\"{key}\":")).unwrap_or_else(|| panic!("{key} missing: {json}")) + key.len() + 3..];
    tail[..tail.find([',', '}']).unwrap()].parse().unwrap_or(u64::MAX)
}

// The fixture on a stepping clock, past its warm-up: the emulator, the clock, and one host tick that
// answers a redraw request the way wie_validate's loop does.
fn boot_pace_fixture() -> Result<(J2MEEmulator, TestClock, impl Fn(&mut J2MEEmulator) -> Result<()>)> {
    let clock = TestClock::stepping(1);
    let platform = TestPlatform::with_clock(clock.clone());
    let redraw = platform.redraw_flag();
    let archive = extract_zip(include_bytes!("../../test_data/pace_j2me.zip"))?;
    let jar = archive
        .get("pace_j2me.jar")
        .expect("test_data/pace_j2me.zip must hold pace_j2me.jar — regenerate it with scripts/make-pace-fixture.mjs")
        .clone();
    let mut emulator = J2MEEmulator::from_jar(Box::new(platform), "pace_j2me.jar", jar)?;

    let tick = move |emulator: &mut J2MEEmulator| -> Result<()> {
        emulator.tick()?;
        // wie_validate's host loop: answer a host redraw request with the event.
        if redraw.swap(false, Ordering::SeqCst) {
            emulator.handle_event(Event::Redraw);
        }
        Ok(())
    };
    while clock.peek() < WARMUP_MS {
        tick(&mut emulator)?;
    }
    emulator.take_pacing();
    Ok((emulator, clock, tick))
}

fn run_pace_fixture() -> Result<String> {
    let (mut emulator, clock, tick) = boot_pace_fixture()?;
    let now = || clock.peek();
    let from = now();
    while now() < from + WINDOW_MS {
        tick(&mut emulator)?;
    }
    Ok(emulator.take_pacing().summary_json())
}

#[test]
fn a_deadline_game_loop_runs_at_the_period_it_asks_for() -> Result<()> {
    let pacing = run_pace_fixture()?;
    println!("pacing {pacing}");

    let paints = field(&pacing, "paints");
    let redraws = field(&pacing, "redraws");
    let cross = field(&pacing, "redraw_cross_tick");
    let redraw_p95 = field(&pacing, "redraw_p95");
    let sleep_late_p95 = field(&pacing, "sleep_late_p95");
    let gcs = field(&pacing, "gcs");

    // Actual frame period against the guest's 50ms. 0.9 is the ticket's line for "slow".
    let period = WINDOW_MS as f64 / paints.max(1) as f64;
    assert!(
        PERIOD_MS as f64 / period >= 0.9,
        "the loop asks for {PERIOD_MS}ms frames and got {period:.1}ms ({paints} paints in {WINDOW_MS}ms): {pacing}"
    );
    assert!(
        redraws * 10 >= paints * 9,
        "the loop's paints should nearly all answer its repaints: {pacing}"
    );
    assert!(
        cross * 10 <= redraws,
        "{cross} of {redraws} paints started a host tick after their repaint — the engine is holding repaints \
         to the end of a tick again (#338): {pacing}"
    );
    // Measured on this tree: p95 4 and 0. A getNextEvent that ignores the queue made it 21 and 4.
    assert!(
        redraw_p95 <= 8,
        "repaint -> paint p95 {redraw_p95}ms — something is holding the paint event while the guest spins: {pacing}"
    );
    assert!(
        sleep_late_p95 <= 5,
        "guest sleeps woke {sleep_late_p95}ms late (p95) on a host that ticks back to back: {pacing}"
    );
    assert!(
        gcs * 10 <= paints,
        "{gcs} GCs over {paints} paints — a collection per paint is #338's other half: {pacing}"
    );
    Ok(())
}

#[test]
fn a_tick_that_ends_on_a_guest_sleep_reports_its_wake_as_due() -> Result<()> {
    // `AchievedSpeed` (wie-backend speed_clock.rs) reads `Pacing::due` to tell a light game on a slow
    // host — its wake passes before the next tick — from one waiting for a key. The fixture sleeps
    // to a 50ms deadline, and its MIDP event thread meanwhile polls the queue every 1ms: the wake is
    // the due, the polls are not (a poll would put the due 1ms ahead, and every tick would read late).
    let (mut emulator, clock, tick) = boot_pace_fixture()?;
    let (mut ticks, mut leads) = (0u64, Vec::new());
    let from = clock.peek();
    while clock.peek() < from + WINDOW_MS {
        tick(&mut emulator)?;
        ticks += 1;
        let now = clock.peek();
        if let Some(due) = emulator.take_pacing().due() {
            leads.push(due.raw() as i64 - now as i64);
        }
    }
    leads.sort();
    // Measured on this tree: all 600 ticks report a due, 1..=35ms ahead, 400 of them past a 16ms
    // host frame (a 39ms sleep spans three 16ms ticks). A tick that drops the executor's wake
    // reports none; one that counts the 1ms polls reports every due 1ms ahead.
    let frames = WINDOW_MS / PERIOD_MS;
    let far = leads.iter().filter(|&&x| x > 16).count() as u64;
    assert!(
        far >= frames,
        "{far} ticks (of {ticks}, {} with a due) saw the guest's wake past a host frame over {frames} frames — \
         the due is no longer the sleep's wake (dropped, or a 1ms poll in its place): {leads:?}",
        leads.len()
    );
    assert!(
        leads.iter().all(|&x| (1..=PERIOD_MS as i64).contains(&x)),
        "a due is the guest's next wake, within its {PERIOD_MS}ms period and not in the past: {leads:?}"
    );
    Ok(())
}
