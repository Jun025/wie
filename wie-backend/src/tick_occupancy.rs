//! Where a browser host's frame goes — emulating, waiting inside a tick for a guest's paced wake, or
//! handed back to the host — on a simulated vsync and clock, so the split is a ratio no machine's
//! load moves (docs/report/0556). `cargo test -p wie-backend tick_occupancy -- --nocapture` prints
//! the table. Its tests also hold the host-wake path (`Platform::wake_margin_ms`) to the paced
//! sleep's timing.

extern crate std;

use alloc::{sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};
use std::{println, string::String};

use spin::Mutex;

use crate::{
    executor::Executor,
    frame_pacer::FramePacer,
    pacing::Pacing,
    speed_clock::{AchievedSpeed, SpeedClock, max_wait},
    task::{SleepFuture, YieldFuture},
};

// Wall µs a clock read costs; waiting for a wake is nothing but reads.
const READ_US: u64 = 1;
// Wall µs of emulation between two yields of a busy guest.
const SLICE_US: u64 = 500;
// Wall µs of emulation in one game frame of a sleeping guest (update + paint): 2-4ms, varying, so
// a sleep does not lock onto the display's phase — a fixed one made sleep(60) at 2x land in the
// same gap after every budget, and that is a property of the simulation, not of a game.
const FRAME_SLICES: (u64, u64) = (4, 8);
// The same of a heavy one: 15-20ms, more than a 60Hz tick's budget, so every game frame is cut by
// one. Where a wake tick may run is what this shape is for: with a budget of its own from wherever
// its timer fired it ran past the next vsync, and the game lost 1.5-11% of its frames at 60Hz.
const HEAVY_FRAME_SLICES: (u64, u64) = (30, 40);
// Wall µs a frame callback runs after its vsync, at most.
const CALLBACK_LATE_US: u64 = 1_000;
// Wall µs of one poll.
const POLL_WORK_US: u64 = 50;
const SECONDS: u64 = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Shape {
    // Never sleeps: a title that wants more CPU than a frame has.
    Busy,
    // `FRAME_SLICES` of work then `sleep(n)`, for ever (메이플스토리2007's sleep(60)).
    Sleep(u64),
    // `HEAVY_FRAME_SLICES` of work then `sleep(n)`.
    HeavySleep(u64),
    // An event thread polling every 1ms for a key that never comes (`POLL_SLEEP_MS`).
    Menu,
}

// The host's wake timer: fires this many µs after it was due, uniformly up to the bound.
struct Jitter(u64, u64);

impl Jitter {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        if self.1 == 0 { 0 } else { (self.0 >> 33) % (self.1 + 1) }
    }
}

#[derive(Default, Debug)]
struct Split {
    wall_us: u64,
    emulate_us: u64,
    wait_us: u64,
    ticks: u64,
    // How late each `sleep(n)` woke, guest ms past `n`.
    late: Vec<u64>,
    // `AchievedSpeed` at the end, fed every tick as the browser host feeds it.
    achieved: f64,
    // The host's wake ticks: all, those in which the guest did not run, those still running at the
    // next vsync, and the longest.
    wake_ticks: u64,
    wake_ticks_idle: u64,
    wake_ticks_past_vsync: u64,
    wake_tick_max_us: u64,
}

impl Split {
    fn busy(&self) -> f64 {
        (self.emulate_us + self.wait_us) as f64 / self.wall_us as f64
    }

    fn pct(&self, us: u64) -> f64 {
        us as f64 * 100.0 / self.wall_us as f64
    }

    fn late_max(&self) -> u64 {
        self.late.iter().copied().max().unwrap_or(0)
    }

    fn late_p50(&self) -> u64 {
        let mut late = self.late.clone();
        late.sort_unstable();
        late.get(late.len() / 2).copied().unwrap_or(0)
    }
}

/// Runs `shape` at `speed` on a display of `period_us`, ticking once per vsync with the budget
/// `FramePacer` gives that display. `margin`: `None` is today's host; `Some(ms)` a host that ends a
/// tick on a wake further off than `ms` and comes back for it on a timer late by up to `jitter_us`.
fn run(shape: Shape, speed: f64, period_us: u64, margin: Option<u64>, jitter_us: u64) -> Split {
    let wall = Arc::new(AtomicU64::new(1_000_000));
    let emulated = Arc::new(AtomicU64::new(0));
    let late = Arc::new(Mutex::new(Vec::new()));
    let clock = Arc::new(SpeedClock::new());
    clock.set_speed(1_000.0, speed);
    let mut executor = Executor::new();

    let work = {
        let (wall, emulated) = (wall.clone(), emulated.clone());
        move |us: u64| {
            wall.fetch_add(us, Ordering::SeqCst);
            emulated.fetch_add(us, Ordering::SeqCst);
        }
    };
    let guest_now = {
        let (wall, clock) = (wall.clone(), clock.clone());
        move || clock.now(wall.load(Ordering::SeqCst) as f64 / 1000.0)
    };
    let (task_executor, task_late, task_now) = (executor.clone(), late.clone(), guest_now.clone());
    let slices = if let Shape::HeavySleep(_) = shape {
        HEAVY_FRAME_SLICES
    } else {
        FRAME_SLICES
    };
    let mut frame_work = Jitter(3, slices.1 - slices.0);
    executor.spawn(move || async move {
        // Not `loop`: a task that never returns has type `!`, which `spawn` does not take.
        for _ in 0.. {
            match shape {
                Shape::Busy => {
                    work(SLICE_US);
                    YieldFuture::new().await;
                }
                Shape::Sleep(ms) | Shape::HeavySleep(ms) => {
                    for _ in 0..slices.0 + frame_work.next() {
                        work(SLICE_US);
                        YieldFuture::new().await;
                    }
                    // `System::sleep`: the deadline counts from the call.
                    let due = task_now() + ms;
                    SleepFuture::new(due, &task_executor).await;
                    task_late.lock().push(task_now().raw().saturating_sub(due.raw()));
                }
                Shape::Menu => {
                    work(POLL_WORK_US);
                    // `System::sleep` at `POLL_SLEEP_MS`: a poll, which keeps no tick alive.
                    SleepFuture::toward(task_now() + 1, None, &task_executor).await;
                }
            }
        }
    });

    let budget = FramePacer::budget_for_period_us(period_us);
    let max_wait = margin.map(|ms| max_wait(ms, speed));
    let mut jitter = Jitter(7, jitter_us);
    let mut callback_late = Jitter(11, CALLBACK_LATE_US);
    let mut split = Split::default();
    let start = wall.load(Ordering::SeqCst);
    let mut achieved = AchievedSpeed::default();
    // As `System::tick_for` keeps them. `budget`: `None` is a wake tick, the rest of the last frame
    // tick (`Executor::resume_tick`). Returns the µs emulated.
    let mut pacing = Pacing::default();
    let mut frame_end = 0.0;
    let mut tick = |executor: &mut Executor, split: &mut Split, pacing: &mut Pacing, budget: Option<u64>| {
        let (w0, e0) = (wall.load(Ordering::SeqCst), emulated.load(Ordering::SeqCst));
        let read = {
            let (wall, clock) = (wall.clone(), clock.clone());
            move || clock.now(wall.fetch_add(READ_US, Ordering::SeqCst) as f64 / 1000.0)
        };
        let begin = w0 as f64 / 1000.0;
        match budget {
            Some(budget) => {
                frame_end = begin + budget as f64;
                executor.tick_for(read, clock.budget(budget), max_wait).unwrap()
            }
            None => executor.resume_tick(read, max_wait).unwrap(),
        }
        let (w1, e1) = (wall.load(Ordering::SeqCst), emulated.load(Ordering::SeqCst));
        split.emulate_us += e1 - e0;
        split.wait_us += (w1 - w0) - (e1 - e0);
        split.ticks += 1;
        let end = w1 as f64 / 1000.0;
        pacing.tick_ended(executor.wants_cpu(), executor.next_due(), executor.next_wake());
        let due = pacing.due().map(|due| clock.wall_at(end, due));
        // The browser host's `advance`: a wake tick's budget is what the frame's had left.
        achieved.tick(begin, end, (frame_end - begin).max(0.0), executor.wants_cpu(), due);
        e1 - e0
    };
    for frame in 1..=SECONDS * 1_000_000 / period_us {
        let vsync = start + frame * period_us;
        // A tick that ran past this vsync missed it; the host's next frame is the one after.
        if wall.load(Ordering::SeqCst) >= vsync {
            continue;
        }
        wall.store(vsync + callback_late.next(), Ordering::SeqCst);
        tick(&mut executor, &mut split, &mut pacing, Some(budget));
        // The host-wake loop (crate doc of `wie_featurephone`): after every tick a timer for
        // `wake_in_ms`, which the next frame callback cancels if it comes first.
        while let Some(margin) = margin {
            let Some(due) = pacing.wake() else { break };
            let now = wall.load(Ordering::SeqCst);
            let fire = now + (clock.wake_delay(now as f64 / 1000.0, due, margin) * 1000.0) as u64 + jitter.next();
            if fire >= vsync + period_us {
                break;
            }
            wall.store(fire, Ordering::SeqCst);
            let emulated = tick(&mut executor, &mut split, &mut pacing, None);
            let end = wall.load(Ordering::SeqCst);
            split.wake_ticks += 1;
            split.wake_ticks_idle += u64::from(emulated == 0);
            split.wake_ticks_past_vsync += u64::from(end >= vsync + period_us);
            split.wake_tick_max_us = split.wake_tick_max_us.max(end - fire);
        }
    }
    split.wall_us = wall.load(Ordering::SeqCst) - start;
    split.achieved = achieved.get(speed);
    split.late = core::mem::take(&mut *late.lock());
    executor.clear();
    split
}

const SHAPES: [Shape; 6] = [
    Shape::Busy,
    Shape::Sleep(60),
    Shape::Sleep(20),
    Shape::HeavySleep(60),
    Shape::HeavySleep(20),
    Shape::Menu,
];
const SPEEDS: [f64; 3] = [1.0, 1.5, 2.0];
const PERIODS: [u64; 2] = [16_667, 8_333];
// The margin the browser host is told to use: WebKit's setTimeout, armed from a frame callback,
// fired at most 3ms late (300 samples × 2 runs, headless, 2026-11-11 — docs/report/0556).
const MARGIN_MS: u64 = 3;
const WEBKIT_JITTER_US: u64 = 3_000;

fn row(label: &str, shape: Shape, speed: f64, period_us: u64, s: &Split) -> String {
    std::format!(
        "| {label} | {shape:?} | {speed}x | {}Hz | {:.1} | {:.1} | {:.1} | {:.1} | {} | {} | {} | {:.3} | {} | {} |",
        (1e6 / period_us as f64).round(),
        s.pct(s.emulate_us),
        s.pct(s.wait_us),
        s.pct(s.wall_us - s.emulate_us - s.wait_us),
        s.busy() * 100.0,
        s.late_p50(),
        s.late_max(),
        s.late.len(),
        s.achieved,
        s.wake_ticks,
        s.wake_ticks_idle,
    )
}

#[test]
fn tick_occupancy_table() {
    println!(
        "| host | shape | speed | display | emulate % | wait % | host % | busy % | late p50 | late max | frames | achieved | wake ticks | idle |"
    );
    println!("|---|---|---|---|---|---|---|---|---|---|---|---|---|---|");
    for shape in SHAPES {
        for period in PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                println!("{}", row("before", shape, speed, period, &before));
                println!("{}", row("after", shape, speed, period, &after));
                // The busy guest is the one the margin must not touch: it never sleeps.
                if let Shape::Busy = shape {
                    assert_eq!(before.busy(), after.busy(), "{shape:?} {speed}x {period}");
                }
            }
        }
    }
    println!();
    println!("| margin ms | timer jitter ms | Sleep(60) 1x 60Hz wait % | late p50 | late max |");
    println!("|---|---|---|---|---|");
    for jitter in [3_000, 10_000] {
        for margin in [0, 1, 2, 3, 4, 6] {
            let s = run(Shape::Sleep(60), 1.0, 16_667, Some(margin), jitter);
            println!(
                "| {margin} | {} | {:.1} | {} | {} |",
                jitter / 1000,
                s.pct(s.wait_us),
                s.late_p50(),
                s.late_max()
            );
        }
    }
}

const LIGHT: [Shape; 2] = [Shape::Sleep(60), Shape::Sleep(20)];
const SLEEPERS: [Shape; 4] = [Shape::Sleep(60), Shape::Sleep(20), Shape::HeavySleep(60), Shape::HeavySleep(20)];

#[test]
fn a_paced_sleep_keeps_its_timing_when_the_host_wakes_it() {
    for shape in SLEEPERS {
        for period in PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                let case = std::format!("{shape:?} {speed}x {period}us");
                assert!(before.late.len() > 100, "{case}: {}", before.late.len());
                // As many game frames (a frame either way at the end of the run), and none later than
                // a wake can be on either host: one in the gap a tick leaves the host waits for the
                // next frame callback (plus the clock's last ms). The host's timer firing a full
                // margin late puts a wake at the very end of the budget there too, so the two maxima
                // are a millisecond or two apart either way — not the same sample.
                assert!(
                    after.late.len() * 100 >= before.late.len() * 99,
                    "{case}: {} vs {}",
                    after.late.len(),
                    before.late.len()
                );
                let gap_us = period - FramePacer::budget_for_period_us(period) * 1000 + CALLBACK_LATE_US;
                let bound = (gap_us as f64 * speed / 1000.0) as u64 + 2;
                assert!(
                    before.late_max() <= bound && after.late_max() <= bound,
                    "{case}: late max {} -> {} > {bound}",
                    before.late_max(),
                    after.late_max()
                );
                assert!(
                    after.late_p50() <= before.late_p50(),
                    "{case}: late p50 {} > {}",
                    after.late_p50(),
                    before.late_p50()
                );
                // Extra ticks between frames are not a slower game: `achieved_speed` stays at most
                // `speed` and within 1% of today's (measured when written: -0.7% to +1.1%).
                assert!(
                    after.achieved <= speed && after.achieved >= before.achieved * 0.99,
                    "{case}: achieved {} vs {}",
                    after.achieved,
                    before.achieved
                );
            }
        }
    }
}

#[test]
fn the_host_wake_hands_back_the_wait() {
    let (mut before_us, mut after_us) = (0, 0);
    for shape in SLEEPERS {
        for period in PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                let case = std::format!("{shape:?} {speed}x {period}us");
                // Per game frame, the wait is what a tick waits itself (`max_wait`: the margin, or the
                // 4ms of a host timer's floor), not what is left of the budget.
                let bound = after.late.len() as u64 * (MARGIN_MS + 1) * 1_000;
                assert!(after.wait_us <= bound, "{case}: wait {} > {bound}", after.wait_us);
                // And in no cell more than today's: the host comes back only for a wake today's tick
                // waits for. While it came back for any wake, 5 of the 12 light cells waited more.
                assert!(after.wait_us <= before.wait_us, "{case}: wait {} > {}", after.wait_us, before.wait_us);
                if LIGHT.contains(&shape) {
                    before_us += before.wait_us;
                    after_us += after.wait_us;
                }
            }
        }
    }
    // The light shapes, measured: 13.2s -> 3.6s of 120s. Not lower: where a sleep keeps landing in
    // the gap a tick leaves the host, neither host waits for it (`tick_occupancy_table`).
    assert!(after_us * 10 < before_us * 4, "wait {after_us}us vs {before_us}us");
}

// A 30Hz display too (`FramePacer`'s 30ms budget): the longest a wake tick could hold the page.
const WAKE_PERIODS: [u64; 3] = [33_333, 16_667, 8_333];

#[test]
fn a_wake_tick_ends_inside_its_frame() {
    println!("| shape | speed | display | frames off | frames | wake ticks | idle | past the next vsync | longest ms |");
    println!("|---|---|---|---|---|---|---|---|---|");
    let mut wake_ticks = 0;
    for shape in SLEEPERS {
        for period in WAKE_PERIODS {
            for speed in SPEEDS {
                let before = run(shape, speed, period, None, 0);
                let after = run(shape, speed, period, Some(MARGIN_MS), WEBKIT_JITTER_US);
                let case = std::format!("{shape:?} {speed}x {period}us");
                println!(
                    "| {shape:?} | {speed}x | {}Hz | {} | {} | {} | {} | {} | {:.1} |",
                    (1e6 / period as f64).round(),
                    before.late.len(),
                    after.late.len(),
                    after.wake_ticks,
                    after.wake_ticks_idle,
                    after.wake_ticks_past_vsync,
                    after.wake_tick_max_us as f64 / 1000.0
                );
                // It is the rest of the frame tick (`Executor::resume_tick`), so it ends where that would
                // have: before the next vsync, and the frame after it is not missed.
                assert_eq!(after.wake_ticks_past_vsync, 0, "{case}");
                // And the guest runs in every one: the tick waits as long as the host's timer aims
                // ahead (`max_wait`), so none ends on the wake it was called for.
                assert_eq!(after.wake_ticks_idle, 0, "{case}");
                wake_ticks += after.wake_ticks;
                assert!(
                    after.late.len() * 100 >= before.late.len() * 99,
                    "{case}: {} vs {}",
                    after.late.len(),
                    before.late.len()
                );
            }
        }
    }
    assert!(wake_ticks > 1_000, "{wake_ticks}");
}

#[test]
fn without_the_margin_a_late_timer_makes_the_wake_late() {
    // The margin absorbs the timer's lateness: with none, the wake is as late as its timer.
    let tight = run(Shape::Sleep(60), 1.0, 16_667, Some(0), WEBKIT_JITTER_US);
    let before = run(Shape::Sleep(60), 1.0, 16_667, None, 0);
    assert!(tight.late_p50() > before.late_p50(), "{} vs {}", tight.late_p50(), before.late_p50());
}
